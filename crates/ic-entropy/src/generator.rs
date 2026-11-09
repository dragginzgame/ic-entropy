//! Explicitly owned ChaCha20 state with a caller-selected byte budget.

use crate::Seed;
use rand_chacha::{
    ChaCha20Rng,
    rand_core::{Rng, SeedableRng},
};

/// A request would exceed the remaining byte budget; no bytes were consumed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Exhausted;

/// Deterministic pseudorandom expansion of a full seed, bounded by output bytes.
///
/// Not clonable or serializable: copying or restoring state can replay outputs.
/// Hosts must obtain a fresh seed after a restart or restore. The byte budget is
/// an operational reseeding limit, not a count of remaining entropy bits.
pub struct Generator {
    rng: ChaCha20Rng,
    remaining: u64,
}

impl Generator {
    /// Start a stream from externally acquired entropy and an explicit budget.
    ///
    /// A zero budget is valid and refuses every nonempty draw. Separate consumers
    /// must receive independent seeds; identical seeds reproduce identical streams.
    pub fn from_seed(seed: Seed, byte_budget: u64) -> Self {
        Self {
            rng: ChaCha20Rng::from_seed(seed),
            remaining: byte_budget,
        }
    }

    /// Number of output bytes available before the host must replace the seed.
    pub const fn remaining_bytes(&self) -> u64 {
        self.remaining
    }

    /// Fill the entire output, or leave both it and the stream unchanged.
    ///
    /// # Errors
    /// Returns [`Exhausted`] when the output exceeds the remaining budget.
    pub fn fill_bytes(&mut self, output: &mut [u8]) -> Result<(), Exhausted> {
        let bytes = u64::try_from(output.len()).map_err(|_| Exhausted)?;
        self.reserve(bytes)?;
        self.rng.fill_bytes(output);
        Ok(())
    }

    /// Draw a 128-bit value using IcyDB's existing high-word, low-word order.
    ///
    /// # Errors
    /// Returns [`Exhausted`] without advancing the stream if fewer than 16 bytes remain.
    pub fn next_u128(&mut self) -> Result<u128, Exhausted> {
        self.reserve(16)?;
        let hi = u128::from(self.rng.next_u64());
        let lo = u128::from(self.rng.next_u64());
        Ok((hi << 64) | lo)
    }

    /// Replace the stream with a freshly acquired seed and a new byte budget.
    ///
    /// This is replacement, not entropy mixing. Reusing a seed replays its stream.
    pub fn reseed(&mut self, seed: Seed, byte_budget: u64) {
        *self = Self::from_seed(seed, byte_budget);
    }

    fn reserve(&mut self, bytes: u64) -> Result<(), Exhausted> {
        self.remaining = self.remaining.checked_sub(bytes).ok_or(Exhausted)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
