#![doc = include_str!("../README.md")]

#[cfg(feature = "buffered")]
mod buffered;
mod source;

#[cfg(feature = "buffered")]
pub use buffered::{
    BYTE_BUDGET, MAX_DRAW_BYTES, POOL_CAPACITY, RefillState, Status, fill_bytes, reconstruct,
    retry_refill, status, take_seed,
};
pub use source::fresh_seed;

/// Failure to acquire or consume the explicitly requested quality of randomness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// Mutating draws and source requests require replicated Wasm execution.
    NonReplicated,
    /// The host has not enabled buffering or has suspended it.
    Disabled,
    /// No initial buffered entropy has arrived yet.
    NotReady,
    /// No seed or sufficient generator budget is currently available.
    Exhausted,
    /// The requested local draw exceeds the fixed per-call work limit.
    DrawTooLarge,
    /// Another fresh source request is already outstanding.
    Busy,
    /// The management request failed or timed out; no automatic fresh retry.
    Transport,
    /// Platform refused the external raw_rand response.
    MalformedReply,
    /// The lifecycle instance changed while a request was outstanding.
    Stale,
    /// The bounded refill failure budget is exhausted; explicit retry is needed.
    RetryLimit,
    /// Timers refused the adapter's registration or control request.
    Scheduling,
}

/// Invalidate pending source replies before snapshot/restore or shutdown.
///
/// With `buffered`, also discard all buffered seeds/stream state and cancel refill.
/// This installs no lifecycle exports. The host must call it before taking a snapshot.
pub fn suspend() -> Result<(), Error> {
    source::require_replicated()?;
    source::invalidate(true);
    #[cfg(feature = "buffered")]
    buffered::suspend()?;
    Ok(())
}

/// Reopen fresh requests after a fresh-only host's lifecycle reconstruction.
///
/// Buffered hosts use `reconstruct` instead. No request or timer is started here.
/// Refuses while buffering is active, so feature unification cannot resume an old stream.
pub fn resume_fresh_requests() -> Result<(), Error> {
    source::require_replicated()?;
    #[cfg(feature = "buffered")]
    if buffered::status().refill != RefillState::Disabled {
        return Err(Error::Busy);
    }
    source::invalidate(false);
    Ok(())
}
