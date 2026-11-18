//! IC timer registration and callback driving; demand and custody stay shared.

use super::{BUFFER, RefillState, refill_cancellation, status};
use crate::{Error, source};
use ic_timers::{Lifetime, Observation, Watchdog, WatchdogDemand, WatchdogNext, WatchdogResult};
use std::{cell::RefCell, mem::DropGuard, num::NonZeroU64};

thread_local! {
    static REFILL: RefCell<Option<Watchdog>> = const { RefCell::new(None) };
}

pub(super) fn reconcile(will_draw: bool) -> Result<(), Error> {
    let status = status();
    // Opt-out never creates even an inactive declaration.
    if status.refill == RefillState::Disabled && REFILL.with_borrow(Option::is_none) {
        return Ok(());
    }
    ic_timers::initialize().map_err(|_| Error::Scheduling)?;
    let desired = match status.refill {
        RefillState::Idle if will_draw => WatchdogDemand::At(status.next_refill_ns),
        RefillState::Disabled | RefillState::Idle | RefillState::Paused => WatchdogDemand::Inactive,
        RefillState::Pending => WatchdogDemand::KeepCadence,
        RefillState::Scheduled => WatchdogDemand::At(status.next_refill_ns),
    };
    REFILL
        .with_borrow_mut(|slot| {
            if slot.is_none() {
                *slot = Some(ic_timers::bind_watchdog(
                    ["ic-entropy-ic", "buffer", "refill"],
                    NonZeroU64::new(1_000_000_000).expect("positive refill cadence"),
                    Lifetime::Retained,
                    drive,
                )?);
            }
            slot.as_ref().expect("bound refill watchdog").set(desired)
        })
        .map_err(|_| Error::Scheduling)
}

pub(super) fn now_ns() -> u64 {
    ic_platform_runtime::system::time()
}

fn drive() -> ic_timers::WatchdogResult {
    if let Some(generation) = BUFFER.with_borrow_mut(|state| state.begin(now_ns())) {
        let cancellation = refill_cancellation(generation, now_ns());
        // Timers invokes this synchronous watchdog directly from its self-update
        // export, outside executor polling. Timers owns the dispatch reply; this
        // root only awaits acquisition and never issues a reply of its own.
        ic_platform_runtime::executor::run_update(async move {
            let cancellation = cancellation;
            let result = source::fresh_seed().await;
            BUFFER.with_borrow_mut(|state| state.finish(generation, result, now_ns()));
            DropGuard::dismiss(cancellation);
            // Reconcile even on a stale completion: current demand remains authoritative.
            reconcile(false).expect("entropy refill scheduling");
        })
        .expect("entropy refill task admission");
    }
    let status = status();
    let decision = match status.refill {
        RefillState::Disabled | RefillState::Idle | RefillState::Paused => WatchdogNext::Stop,
        RefillState::Pending => WatchdogNext::Continue,
        RefillState::Scheduled => WatchdogNext::At(status.next_refill_ns),
    };
    WatchdogResult {
        observation: Observation::NoWork,
        next: decision,
    }
}
