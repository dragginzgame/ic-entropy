#![doc = include_str!("../README.md")]

#[cfg(feature = "buffered")]
mod buffered;
mod generator;
#[cfg(feature = "logging")]
mod logging;
#[cfg(feature = "metrics")]
mod metrics;
mod pool;
mod source;

#[cfg(feature = "buffered")]
pub use buffered::{
    BYTE_BUDGET, MAX_DRAW_BYTES, POOL_CAPACITY, RefillState, Status, fill_bytes, reconstruct,
    retry_refill, status, take_seed,
};
pub use generator::{Exhausted, Generator};
#[cfg(feature = "logging")]
pub use logging::{LogSink, set_log_sink};
#[cfg(feature = "metrics")]
pub use metrics::{AcquisitionMetrics, Metrics, MetricsStorage, initialize_with_metrics, metrics};
pub use pool::{PoolFull, SeedPool};
pub use source::fresh_seed;

/// One full seed supplied by a cryptographically secure entropy source.
///
/// The length alone cannot establish entropy quality. Do not seed with clocks,
/// counters, identifiers, or a small set of possible values.
pub type Seed = [u8; 32];

///
/// Error
///
/// Failure to acquire or consume the explicitly requested quality of randomness.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// Mutation requires replicated Wasm execution.
    NonReplicated,
    /// The host has not enabled buffering or has suspended it.
    Disabled,
    /// No initial buffered entropy has arrived yet.
    NotReady,
    /// No seed or sufficient generator budget is currently available.
    Exhausted,
    /// The requested local draw exceeds the fixed per-call work limit.
    DrawTooLarge,
    /// Source custody, active buffering or already-bound metrics storage conflicts
    /// with the requested operation.
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
///
/// # Errors
/// Refuses non-replicated execution or a failed timer cancellation.
///
/// # Panics
/// Traps if the logical lifecycle generation is exhausted. With `logging`, also
/// propagates a panic from the configured host sink.
pub fn suspend() -> Result<(), Error> {
    source::require_replicated()?;
    source::invalidate(true);
    #[cfg(feature = "buffered")]
    buffered::suspend()?;
    #[cfg(feature = "logging")]
    logging::record(logging::Event::Suspended);

    Ok(())
}

/// Reopen fresh requests after a fresh-only host's lifecycle reconstruction.
///
/// Buffered hosts use `reconstruct` instead. No request or timer is started here.
/// Refuses while buffering is active, so feature unification cannot resume an old stream.
///
/// # Errors
/// Returns [`Error::NonReplicated`] outside replicated execution, or [`Error::Busy`]
/// while buffering remains active.
///
/// # Panics
/// Traps if the logical lifecycle generation is exhausted. With `logging`, also
/// propagates a panic from the configured host sink.
pub fn resume_fresh_requests() -> Result<(), Error> {
    source::require_replicated()?;
    #[cfg(feature = "buffered")]
    if buffered::status().refill != RefillState::Disabled {
        return Err(Error::Busy);
    }
    source::invalidate(false);
    #[cfg(feature = "logging")]
    logging::record(logging::Event::FreshResumed);

    Ok(())
}
