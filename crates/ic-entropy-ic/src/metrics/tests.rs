use super::*;

#[test]
fn host_storage_is_shared_idempotent_and_cannot_be_replaced() {
    assert!(metrics().is_none());
    let host = Rc::new(MetricsStorage::default());
    bind(Rc::clone(&host)).unwrap();
    Acquisition::start_at(Rc::clone(&host), Kind::Fresh, Some(10)).finish_at(Ok(()), Some(20));
    let before = host.snapshot();
    bind(Rc::clone(&host)).unwrap();
    assert_eq!(metrics(), Some(before));
    assert_eq!(host.snapshot(), before);
    assert_eq!(bind(Rc::default()), Err(Error::Busy));
    assert_eq!(metrics(), Some(before));
    assert!(Rc::ptr_eq(&storage(), &host));
}

#[test]
fn native_binding_refuses_without_selecting_or_mutating_storage() {
    let host = Rc::new(MetricsStorage::default());
    assert_eq!(
        initialize_with_metrics(Rc::clone(&host)),
        Err(Error::NonReplicated)
    );
    assert!(metrics().is_none());
    assert_eq!(host.snapshot(), Metrics::default());
}

#[test]
fn completions_refusals_and_cancellation_have_distinct_accounting() {
    let host = Rc::new(MetricsStorage::default());
    Acquisition::start_at(Rc::clone(&host), Kind::Fresh, Some(10))
        .finish_at(Ok(()), Some(1_000_010));
    for error in [
        Error::Transport,
        Error::MalformedReply,
        Error::Stale,
        Error::Busy,
        Error::Disabled,
    ] {
        Acquisition::start_at(Rc::clone(&host), Kind::Fresh, Some(10))
            .finish_at(Err(error), Some(1_000_010));
    }
    drop(Acquisition::start_at(
        Rc::clone(&host),
        Kind::Fresh,
        Some(10),
    ));
    let before = host.snapshot();
    let never_polled = async {
        let _observation = Acquisition::start_at(Rc::clone(&host), Kind::Fresh, Some(10));
    };
    drop(never_polled);
    assert_eq!(host.snapshot(), before);
    assert_eq!(before.fresh.attempts.exact_total(), Ok(7));
    assert_eq!(before.fresh.succeeded.exact_total(), Ok(1));
    assert_eq!(before.fresh.failed.exact_total(), Ok(5));
    assert_eq!(before.fresh.cancelled.exact_total(), Ok(1));
    assert_eq!(before.fresh.latency_ns.aggregate().samples(), 4);
    assert_eq!(before.fresh.latency_ns.bucket_counts()[0], 4);
    assert_eq!(before.fresh.latency_unavailable.exact_total(), Ok(0));
    for counter in [
        before.transport_failures,
        before.malformed_replies,
        before.stale_replies,
        before.busy_refusals,
        before.disabled_refusals,
    ] {
        assert_eq!(counter.exact_total(), Ok(1));
    }
    assert_eq!(before.refill, AcquisitionMetrics::default());
}

#[test]
fn absent_or_backward_clocks_are_not_zero_latency_samples() {
    let host = Rc::new(MetricsStorage::default());
    for (start, end) in [(None, None), (Some(10), Some(9)), (Some(10), None)] {
        Acquisition::start_at(Rc::clone(&host), Kind::Fresh, start).finish_at(Ok(()), end);
    }
    Acquisition::start_at(Rc::clone(&host), Kind::Fresh, Some(10)).finish_at(Ok(()), Some(10));
    let counters = host.snapshot();
    assert_eq!(counters.fresh.latency_unavailable.exact_total(), Ok(3));
    assert_eq!(counters.fresh.latency_ns.aggregate().samples(), 1);
    assert_eq!(counters.fresh.latency_ns.aggregate().maximum(), Some(0));
}

#[test]
fn overflow_is_preserved_in_the_host_snapshot() {
    let host = Rc::new(MetricsStorage::default());
    host.update(|metrics| metrics.fresh.attempts.add(u128::MAX));
    assert!(!host.snapshot().fresh.attempts.total_overflowed());
    Acquisition::start_at(Rc::clone(&host), Kind::Fresh, Some(0)).finish_at(Ok(()), Some(1));
    assert!(host.snapshot().fresh.attempts.total_overflowed());
    assert_eq!(host.snapshot().fresh.attempts.total(), u128::MAX);
    assert_eq!(host.snapshot().fresh.succeeded.exact_total(), Ok(1));
}

#[cfg(feature = "buffered")]
#[test]
fn background_requests_do_not_become_explicit_fresh_requests() {
    let host = Rc::new(MetricsStorage::default());
    Acquisition::start_at(Rc::clone(&host), Kind::Refill, Some(0)).finish_at(Ok(()), Some(1));
    let counters = host.snapshot();
    assert_eq!(counters.refill.attempts.exact_total(), Ok(1));
    assert_eq!(counters.refill.succeeded.exact_total(), Ok(1));
    assert_eq!(counters.fresh, AcquisitionMetrics::default());
}
