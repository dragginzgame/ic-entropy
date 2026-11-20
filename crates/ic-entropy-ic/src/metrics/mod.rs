//! Fixed operational observations and host-supplied heap storage.
//! No entropy contents, reporting endpoint, timer recording or reset authority.

#[cfg(test)]
mod tests;

use crate::{Error, source};
use ic_metrics::{MeasurementCounter, MeasurementHistogram};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

///
/// AcquisitionMetrics
///
/// One class of source requests, counted when its future is first polled in
/// replicated execution. Returned errors and cancellations are separate outcomes.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AcquisitionMetrics {
    /// Calls polled in replicated execution, including busy/disabled refusals.
    pub attempts: MeasurementCounter,
    /// Calls that returned a newly acquired seed.
    pub succeeded: MeasurementCounter,
    /// Calls that returned an error, including admission and stale-reply refusals.
    pub failed: MeasurementCounter,
    /// Polled calls dropped before returning; never-polled futures are not attempts.
    pub cancelled: MeasurementCounter,
    /// Completed source-call latency in nanoseconds, including transport/framing
    /// errors and stale replies. Busy/disabled refusals and cancellations add no sample.
    /// Inclusive bounds are 1/10/100 ms and 1/10/60/300 s, plus an overflow bucket.
    pub latency_ns: MeasurementHistogram<7>,
    /// Otherwise eligible latency samples whose clock was unavailable or went backward.
    pub latency_unavailable: MeasurementCounter,
}

impl Default for AcquisitionMetrics {
    fn default() -> Self {
        Self {
            attempts: MeasurementCounter::ZERO,
            succeeded: MeasurementCounter::ZERO,
            failed: MeasurementCounter::ZERO,
            cancelled: MeasurementCounter::ZERO,
            latency_ns: MeasurementHistogram::new([
                1_000_000,
                10_000_000,
                100_000_000,
                1_000_000_000,
                10_000_000_000,
                60_000_000_000,
                300_000_000_000,
            ])
            .expect("fixed increasing latency bounds"),
            latency_unavailable: MeasurementCounter::ZERO,
        }
    }
}

///
/// Metrics
///
/// Aggregate observations for one volatile adapter incarnation. Suspension,
/// reconstruction and repeated binding preserve them; normal upgrades start fresh.
/// Counters retain ic-metrics' sticky actual-overflow flags. These values are
/// diagnostic recorder state, not Wire DTOs or authority to draw or retry.
/// Observations follow IC message rollback; they are not a durable effect log.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Metrics {
    /// Explicit fresh_seed source calls; callers own decision-commitment timing.
    pub fresh: AcquisitionMetrics,
    /// Background source calls; zero when buffering is not selected.
    pub refill: AcquisitionMetrics,
    /// Source calls refused because another request held custody.
    pub busy_refusals: MeasurementCounter,
    /// Source calls refused while acquisition was suspended.
    pub disabled_refusals: MeasurementCounter,
    /// Source calls returning a transport failure or unknown outcome.
    pub transport_failures: MeasurementCounter,
    /// Source calls returning a framing failure.
    pub malformed_replies: MeasurementCounter,
    /// Source calls returning after their lifecycle generation was invalidated.
    pub stale_replies: MeasurementCounter,
    /// Seeds removed from the pool, including shared-generator reseeding.
    pub seeds_consumed: MeasurementCounter,
    /// Bytes successfully written by the adapter's shared generator. Consumer-owned
    /// core generators are outside this observation boundary.
    pub bytes_generated: MeasurementCounter,
    /// Successful shared-generator calls, including empty draws.
    pub draws_succeeded: MeasurementCounter,
    /// Replicated shared-generator calls returning an error.
    pub draw_refusals: MeasurementCounter,
    /// Replicated public take_seed calls returning an error.
    pub take_refusals: MeasurementCounter,
    /// Current-generation refill attempts admitted after a previous failure.
    pub refill_retries: MeasurementCounter,
    /// Current-generation failure streaks that exhausted the automatic retry budget.
    pub refill_pauses: MeasurementCounter,
    /// Successful current-generation refills that ended a nonzero failure streak.
    pub refill_recoveries: MeasurementCounter,
    /// Explicit failure-budget resets admitted by the model, even if subsequent
    /// timer reconciliation fails. This does not promise delivery.
    pub refill_restarts: MeasurementCounter,
}

///
/// MetricsStorage
///
/// Inert heap storage retained by a host alongside its other metric sources.
/// Only Entropy records into it; hosts can project snapshots into their own
/// reporting contracts. No persistence or in-place reset is supplied.
///

#[derive(Default)]
pub struct MetricsStorage(Cell<Metrics>);

impl MetricsStorage {
    /// Read the same counters the adapter records, without recording or allocating.
    #[must_use]
    pub fn snapshot(&self) -> Metrics {
        self.0.get()
    }

    fn update(&self, record: impl FnOnce(&mut Metrics)) {
        let mut metrics = self.0.get();
        record(&mut metrics);
        self.0.set(metrics);
    }
}

thread_local! {
    static STORAGE: RefCell<Option<Rc<MetricsStorage>>> = const { RefCell::new(None) };
}

/// Bind host-owned metrics before the first admitted adapter operation.
///
/// Like Timers' shared-storage initialization, repeating the same Rc preserves
/// observations; a different Rc refuses. This starts no entropy request or timer.
/// Without explicit binding, the first recorded operation creates private storage.
/// Supply fresh storage after a normal upgrade and let the host label its lifetime.
///
/// # Errors
/// Returns NonReplicated outside replicated execution, or Busy if different
/// storage has already been selected, including an automatically created store.
pub fn initialize_with_metrics(storage: Rc<MetricsStorage>) -> Result<(), Error> {
    source::require_replicated()?;

    bind(storage)
}

fn bind(storage: Rc<MetricsStorage>) -> Result<(), Error> {
    STORAGE.with_borrow_mut(|slot| match slot {
        Some(current) if !Rc::ptr_eq(current, &storage) => Err(Error::Busy),
        Some(_) => Ok(()),
        None => {
            *slot = Some(storage);

            Ok(())
        }
    })
}

/// Read aggregate diagnostics without creating storage or changing entropy state.
/// Returns None before explicit binding or the first recorded operation.
#[must_use]
pub fn metrics() -> Option<Metrics> {
    STORAGE.with_borrow(|slot| slot.as_ref().map(|storage| storage.snapshot()))
}

fn storage() -> Rc<MetricsStorage> {
    STORAGE.with_borrow_mut(|slot| Rc::clone(slot.get_or_insert_with(Rc::default)))
}

///
/// Kind
///
/// Separates explicit fresh requests from background refill observations.
///

#[derive(Clone, Copy)]
pub(super) enum Kind {
    Fresh,
    #[cfg(feature = "buffered")]
    Refill,
}

fn acquisition(metrics: &mut Metrics, kind: Kind) -> &mut AcquisitionMetrics {
    match kind {
        Kind::Fresh => &mut metrics.fresh,
        #[cfg(feature = "buffered")]
        Kind::Refill => &mut metrics.refill,
    }
}

///
/// Acquisition
///
/// One first-polled call retaining its selected storage through completion or
/// cancellation. Recording cannot release source custody or authorize a retry.
///

pub(super) struct Acquisition {
    storage: Rc<MetricsStorage>,
    kind: Kind,
    started_ns: Option<u64>,
    finished: bool,
}

impl Acquisition {
    pub(super) fn start(kind: Kind) -> Self {
        Self::start_at(storage(), kind, time_ns())
    }

    fn start_at(storage: Rc<MetricsStorage>, kind: Kind, started_ns: Option<u64>) -> Self {
        storage.update(|metrics| acquisition(metrics, kind).attempts.add(1));

        Self {
            storage,
            kind,
            started_ns,
            finished: false,
        }
    }

    pub(super) fn finish(self, result: Result<(), Error>) {
        self.finish_at(result, time_ns());
    }

    fn finish_at(mut self, result: Result<(), Error>, ended_ns: Option<u64>) {
        self.finished = true;
        self.storage.update(|metrics| {
            let calls = acquisition(metrics, self.kind);
            if result.is_ok() {
                calls.succeeded.add(1);
            } else {
                calls.failed.add(1);
            }

            if !matches!(result, Err(Error::Busy | Error::Disabled)) {
                if let Some(elapsed) = self
                    .started_ns
                    .zip(ended_ns)
                    .and_then(|(start, end)| end.checked_sub(start))
                {
                    calls.latency_ns.record(elapsed);
                } else {
                    calls.latency_unavailable.add(1);
                }
            }

            match result {
                Err(Error::Busy) => metrics.busy_refusals.add(1),
                Err(Error::Disabled) => metrics.disabled_refusals.add(1),
                Err(Error::Transport) => metrics.transport_failures.add(1),
                Err(Error::MalformedReply) => metrics.malformed_replies.add(1),
                Err(Error::Stale) => metrics.stale_replies.add(1),
                _ => {}
            }
        });
    }
}

impl Drop for Acquisition {
    fn drop(&mut self) {
        if !self.finished {
            self.storage
                .update(|metrics| acquisition(metrics, self.kind).cancelled.add(1));
        }
    }
}

fn time_ns() -> Option<u64> {
    ic_platform_runtime::raw::in_replicated_execution().then(ic_platform_runtime::system::time)
}

///
/// Event
///
/// An admitted adapter observation recorded by the canonical metrics engine.
///

#[cfg(feature = "buffered")]
#[derive(Clone, Copy)]
pub(super) enum Event {
    SeedConsumed,
    TakeRefused,
    Drawn(usize),
    DrawRefused,
    RefillRetry,
    RefillPaused,
    RefillRecovered,
    RefillRestarted,
}

#[cfg(feature = "buffered")]
pub(super) fn record(event: Event) {
    storage().update(|metrics| match event {
        Event::SeedConsumed => metrics.seeds_consumed.add(1),
        Event::TakeRefused => metrics.take_refusals.add(1),
        Event::Drawn(bytes) => {
            metrics.draws_succeeded.add(1);
            metrics.bytes_generated.add(bytes as u128);
        }
        Event::DrawRefused => metrics.draw_refusals.add(1),
        Event::RefillRetry => metrics.refill_retries.add(1),
        Event::RefillPaused => metrics.refill_pauses.add(1),
        Event::RefillRecovered => metrics.refill_recoveries.add(1),
        Event::RefillRestarted => metrics.refill_restarts.add(1),
    });
}
