//! Volatile buffered demand, source-attempt custody and bounded draw state.
//! No transport, scheduler, persistence, query mutation or seed serialization.

#[cfg(test)]
mod tests;

use super::{BYTE_BUDGET, MAX_DRAW_BYTES, POOL_CAPACITY, RefillState, Status};
use crate::Error;
use ic_entropy::{Generator, SeedPool};

const MAX_FAILURES: u8 = 8;
const SECOND: u64 = 1_000_000_000;

///
/// RefillChange
///
/// Current-generation refill transition reported after releasing model custody.
///

pub(super) enum RefillChange {
    Unchanged,
    Paused,
    Recovered,
}

///
/// Model
///
/// Volatile pool, generator and bounded refill policy for one lifecycle instance.
///

pub(super) struct Model {
    generation: u64,
    enabled: bool,
    pending: bool,
    failures: u8,
    next_ns: u64,
    seeded: bool,
    last_error: Option<Error>,
    pool: SeedPool<POOL_CAPACITY>,
    generator: Option<Generator>,
}

impl Model {
    pub(super) const fn new() -> Self {
        Self {
            generation: 0,
            enabled: false,
            pending: false,
            failures: 0,
            next_ns: 0,
            seeded: false,
            last_error: None,
            pool: SeedPool::new(),
            generator: None,
        }
    }

    pub(super) fn reset(&mut self, enabled: bool, now: u64) {
        let generation = self
            .generation
            .checked_add(1)
            .expect("entropy generation exhausted");
        *self = Self::new();
        self.generation = generation;
        self.enabled = enabled;
        self.next_ns = now;
    }

    pub(super) fn status(&self) -> Status {
        let refill = if !self.enabled {
            RefillState::Disabled
        } else if self.failures == MAX_FAILURES {
            RefillState::Paused
        } else if self.pending {
            RefillState::Pending
        } else if self.pool.is_full() {
            RefillState::Idle
        } else {
            RefillState::Scheduled
        };

        Status {
            buffered_seeds: self.pool.len(),
            remaining_bytes: self
                .generator
                .as_ref()
                .map_or(0, Generator::remaining_bytes),
            refill,
            consecutive_failures: self.failures,
            next_refill_ns: self.next_ns,
            last_error: self.last_error,
        }
    }

    pub(super) fn retry(&mut self, now: u64) -> Result<(), Error> {
        if !self.enabled {
            return Err(Error::Disabled);
        }

        if self.pending {
            return Err(Error::Busy);
        }
        self.failures = 0;
        self.last_error = None;
        self.next_ns = now;
        #[cfg(feature = "metrics")]
        crate::metrics::record(crate::metrics::Event::RefillRestarted);

        Ok(())
    }

    const fn unavailable(&self) -> Error {
        if self.failures == MAX_FAILURES {
            Error::RetryLimit
        } else if self.seeded {
            Error::Exhausted
        } else {
            Error::NotReady
        }
    }

    pub(super) fn take_seed(&mut self) -> Result<[u8; 32], Error> {
        if !self.enabled {
            return Err(Error::Disabled);
        }

        let result = self.pool.take().ok_or_else(|| self.unavailable());
        #[cfg(feature = "metrics")]
        if result.is_ok() {
            crate::metrics::record(crate::metrics::Event::SeedConsumed);
        }

        result
    }

    pub(super) fn fill_bytes(&mut self, output: &mut [u8]) -> Result<(), Error> {
        if !self.enabled {
            return Err(Error::Disabled);
        }

        if output.len() > MAX_DRAW_BYTES {
            return Err(Error::DrawTooLarge);
        }

        if output.is_empty() {
            return Ok(());
        }

        let required = u64::try_from(output.len()).map_err(|_| Error::DrawTooLarge)?;
        if self
            .generator
            .as_ref()
            .is_none_or(|generator| generator.remaining_bytes() < required)
        {
            let seed = self.take_seed()?;
            self.generator = Some(Generator::from_seed(seed, BYTE_BUDGET));
        }

        self.generator
            .as_mut()
            .expect("admitted generator")
            .fill_bytes(output)
            .map_err(|_| Error::Exhausted)
    }

    pub(super) fn begin(&mut self, now: u64) -> Option<u64> {
        if self.status().refill != RefillState::Scheduled || now < self.next_ns {
            return None;
        }
        self.pending = true;
        #[cfg(feature = "metrics")]
        if self.failures > 0 {
            crate::metrics::record(crate::metrics::Event::RefillRetry);
        }

        Some(self.generation)
    }

    pub(super) fn finish(
        &mut self,
        generation: u64,
        result: Result<[u8; 32], Error>,
        now: u64,
    ) -> RefillChange {
        if generation != self.generation || !self.pending {
            return RefillChange::Unchanged;
        }
        self.pending = false;

        match result {
            Ok(seed) => {
                // One claim and one bounded source result: nothing else inserts into the pool.
                self.pool
                    .insert(seed)
                    .expect("refill claim reserves a free pool slot");
                let change = if self.failures > 0 {
                    RefillChange::Recovered
                } else {
                    RefillChange::Unchanged
                };
                #[cfg(feature = "metrics")]
                if matches!(change, RefillChange::Recovered) {
                    crate::metrics::record(crate::metrics::Event::RefillRecovered);
                }

                self.seeded = true;
                self.failures = 0;
                self.last_error = None;
                self.next_ns = now.saturating_add(SECOND);

                change
            }
            Err(error) => {
                self.failures += 1;
                #[cfg(feature = "metrics")]
                if self.failures == MAX_FAILURES {
                    crate::metrics::record(crate::metrics::Event::RefillPaused);
                }
                self.last_error = Some(error);
                let seconds = (1_u64 << (self.failures - 1)).min(60);
                self.next_ns = now.saturating_add(seconds * SECOND);

                if self.failures == MAX_FAILURES {
                    RefillChange::Paused
                } else {
                    RefillChange::Unchanged
                }
            }
        }
    }
}
