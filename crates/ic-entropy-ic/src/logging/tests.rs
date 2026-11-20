use super::{Event, SINK, record, set_log_sink, source_error};
use crate::Error;
use ic_logging::{EntryInput, Level, Limits, Log};
use std::{cell::RefCell, num::NonZeroUsize};

thread_local! {
    static HOST_LOG: RefCell<Log> = const { RefCell::new(Log::new(Limits {
        max_entries: NonZeroUsize::new(16).unwrap(),
        max_label_bytes: 32,
        max_message_bytes: 160,
        max_text_bytes: 4096,
    })) };
}

fn host_record(level: Level, source: &'static str, message: &str) {
    // A host is allowed to observe status from a sink callback.
    #[cfg(feature = "buffered")]
    let _ = crate::status();
    HOST_LOG.with_borrow_mut(|log| {
        let _ = log.append(EntryInput {
            created_at_secs: 1,
            level,
            source,
            topic: None,
            message,
        });
    });
}

#[test]
fn forwarding_uses_host_storage_and_can_be_disabled() {
    assert!(SINK.get().is_none());
    record(Event::Suspended);
    assert!(HOST_LOG.with_borrow(Log::is_empty));
    SINK.set(Some(host_record));

    // Rebinding never replaces the caller-owned log.
    record(Event::Suspended);
    SINK.set(Some(host_record));
    record(Event::FreshResumed);
    SINK.set(None);
    record(Event::Suspended);
    HOST_LOG.with_borrow(|log| {
        let rows = log.snapshot().entries;
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].message, "Entropy suspended");
        assert_eq!(rows[1].message, "Fresh requests resumed");
        assert!(rows.iter().all(|row| row.source == "ic-entropy-ic"
            && row.level == Level::Info
            && row.topic.is_none()
            && !row.truncated));
    });
}

#[test]
fn native_configuration_refuses_without_changing_the_sink() {
    SINK.set(Some(host_record));
    assert_eq!(set_log_sink(None), Err(Error::NonReplicated));
    record(Event::FreshResumed);
    assert_eq!(HOST_LOG.with_borrow(Log::len), 1);
    SINK.set(None);
}

#[test]
fn only_source_failures_are_logged_and_backend_refusals_are_passive() {
    SINK.set(Some(host_record));
    for error in [Error::Busy, Error::Disabled, Error::NonReplicated] {
        source_error(error);
    }
    assert!(HOST_LOG.with_borrow(Log::is_empty));
    for error in [Error::Transport, Error::MalformedReply, Error::Stale] {
        source_error(error);
    }
    HOST_LOG.with_borrow(|log| {
        let rows = log.snapshot().entries;
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].level, Level::Warn);
        assert_eq!(rows[1].level, Level::Warn);
        assert_eq!(rows[2].level, Level::Debug);
    });

    // Canonical admission rejects regressed timestamps. It remains host policy
    // to discard that diagnostic rather than change Entropy's operation result.
    HOST_LOG.with_borrow_mut(|log| {
        log.append(EntryInput {
            created_at_secs: 2,
            level: Level::Info,
            source: "host",
            topic: None,
            message: "Host event",
        })
        .unwrap();
    });
    source_error(Error::Transport);
    assert_eq!(HOST_LOG.with_borrow(Log::len), 4);
    SINK.set(None);
}

#[cfg(feature = "buffered")]
pub(crate) fn with_test_sink(sink: super::LogSink, action: impl FnOnce()) {
    SINK.set(Some(sink));
    action();
    SINK.set(None);
}
