//! Fixed-capacity FIFO of externally acquired seeds; no allocation or expansion.

use crate::Seed;

/// The pool is full. No existing seed was overwritten.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PoolFull;

/// A bounded queue that returns each inserted seed at most once.
///
/// The caller owns source quality and must not insert repeated seeds. This is
/// buffered randomness, not a guarantee of freshness at the time of consumption.
/// No persistence or clone interface is supplied. Removing a seed clears its
/// queue slot, but this crate makes no compiler-level secure-erasure guarantee.
pub struct SeedPool<const N: usize> {
    seeds: [Seed; N],
    head: usize,
    len: usize,
}

impl<const N: usize> SeedPool<N> {
    /// Create an empty pool. Zero capacity is valid and always refuses insertion.
    pub const fn new() -> Self {
        Self {
            seeds: [[0; 32]; N],
            head: 0,
            len: 0,
        }
    }

    /// Number of seeds currently buffered.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Whether there is no seed available.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Whether another seed would exceed the configured capacity.
    pub const fn is_full(&self) -> bool {
        self.len == N
    }

    /// Append an externally acquired seed without overwriting buffered values.
    ///
    /// # Errors
    /// Returns [`PoolFull`] without mutation when capacity is exhausted.
    pub fn insert(&mut self, seed: Seed) -> Result<(), PoolFull> {
        if self.is_full() {
            return Err(PoolFull);
        }
        let until_end = N - self.head;
        let tail = if self.len >= until_end {
            self.len - until_end
        } else {
            self.head + self.len
        };
        self.seeds[tail] = seed;
        self.len += 1;
        Ok(())
    }

    /// Remove the oldest seed, or return `None` when the pool is empty.
    pub fn take(&mut self) -> Option<Seed> {
        if self.is_empty() {
            return None;
        }
        let seed = core::mem::take(&mut self.seeds[self.head]);
        self.head = if self.head == N - 1 { 0 } else { self.head + 1 };
        self.len -= 1;
        Some(seed)
    }
}

impl<const N: usize> Default for SeedPool<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
