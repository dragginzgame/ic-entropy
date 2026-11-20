//! Optional fixed event forwarding into the host's canonical IC Logging store.
//! The host owns storage, timestamps, retention, reporting and authorization.

#[cfg(test)]
pub(crate) mod tests;

use crate::{Error, source};
use ic_logging::Level;
use std::cell::Cell;

/// Host callback compatible with Canic's `logging::record` function.
///
/// The source and message contain only fixed adapter text. The callback must
/// return normally and must not reenter mutating Entropy operations. It may read
/// adapter status. The host handles any log append refusal; logging supplies no
/// entropy admission or recovery policy.
pub type LogSink = fn(Level, &'static str, &str);

thread_local! {
    static SINK: Cell<Option<LogSink>> = const { Cell::new(None) };
}

/// Select, replace or disable the host's sink without allocating a private log.
///
/// Call from the host's synchronous lifecycle setup before acquisition starts.
/// `None` disables forwarding. Repeated setup leaves the host's retained rows
/// intact; normal upgrades need a fresh binding. Queries never change the sink.
/// See [`LogSink`] for the callback contract.
///
/// # Errors
/// Returns [`Error::NonReplicated`] outside replicated Wasm execution.
pub fn set_log_sink(sink: Option<LogSink>) -> Result<(), Error> {
    source::require_replicated()?;
    SINK.set(sink);

    Ok(())
}

///
/// Event
///
/// Fixed adapter diagnostic selected for forwarding to the host log sink.
///

#[derive(Clone, Copy)]
pub(super) enum Event {
    Suspended,
    FreshResumed,
    TransportFailed,
    MalformedReply,
    StaleReply,
    #[cfg(feature = "buffered")]
    Reconstructed,
    #[cfg(feature = "buffered")]
    RetryRequested,
    #[cfg(feature = "buffered")]
    SchedulingFailed,
    #[cfg(feature = "buffered")]
    RefillPaused,
    #[cfg(feature = "buffered")]
    RefillRecovered,
}

pub(super) fn source_error(error: Error) {
    let event = match error {
        Error::Transport => Event::TransportFailed,
        Error::MalformedReply => Event::MalformedReply,
        Error::Stale => Event::StaleReply,
        _ => return,
    };
    record(event);
}

pub(super) fn record(event: Event) {
    let Some(sink) = SINK.get() else {
        return;
    };
    let (level, message) = match event {
        Event::Suspended => (Level::Info, "Entropy suspended"),
        Event::FreshResumed => (Level::Info, "Fresh requests resumed"),
        Event::TransportFailed => (Level::Warn, "Entropy acquisition transport failed"),
        Event::MalformedReply => (Level::Warn, "Entropy acquisition reply malformed"),
        Event::StaleReply => (Level::Debug, "Stale entropy reply discarded"),
        #[cfg(feature = "buffered")]
        Event::Reconstructed => (Level::Info, "Entropy buffer reconstructed"),
        #[cfg(feature = "buffered")]
        Event::RetryRequested => (Level::Info, "Entropy refill retry requested"),
        #[cfg(feature = "buffered")]
        Event::SchedulingFailed => (Level::Warn, "Entropy refill scheduling failed"),
        #[cfg(feature = "buffered")]
        Event::RefillPaused => (Level::Warn, "Entropy refill paused after eight failures"),
        #[cfg(feature = "buffered")]
        Event::RefillRecovered => (Level::Info, "Entropy refill recovered"),
    };
    sink(level, "ic-entropy", message);
}
