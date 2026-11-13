use super::*;

#[test]
fn dropped_refill_reopens_demand_but_cannot_clear_replacement_instance() {
    BUFFER.with_borrow_mut(|state| state.reset(true, 10));
    let generation = BUFFER.with_borrow_mut(|state| state.begin(10)).unwrap();
    drop(refill_cancellation(generation, 10));
    assert_eq!(status().consecutive_failures, 1);
    assert_eq!(status().refill, RefillState::Scheduled);
    let next = status().next_refill_ns;
    let generation = BUFFER.with_borrow_mut(|state| state.begin(next)).unwrap();
    let old = refill_cancellation(generation, next);
    BUFFER.with_borrow_mut(|state| state.reset(true, next));
    assert!(BUFFER.with_borrow_mut(|state| state.begin(next)).is_some());
    drop(old);
    assert_eq!(status().refill, RefillState::Pending);
    assert_eq!(status().consecutive_failures, 0);
}

#[test]
fn cancelling_refill_future_before_or_after_poll_reopens_demand_once() {
    use std::{
        future::{Future, pending},
        task::{Context, Waker},
    };

    for poll_first in [false, true] {
        BUFFER.with_borrow_mut(|state| state.reset(true, 10));
        let generation = BUFFER.with_borrow_mut(|state| state.begin(10)).unwrap();
        let cancellation = refill_cancellation(generation, 10);
        let mut refill = Box::pin(async move {
            let cancellation = cancellation;
            pending::<()>().await;
            DropGuard::dismiss(cancellation);
        });
        if poll_first {
            assert!(
                refill
                    .as_mut()
                    .poll(&mut Context::from_waker(Waker::noop()))
                    .is_pending()
            );
        }
        assert_eq!(status().refill, RefillState::Pending);
        drop(refill);
        assert_eq!(status().refill, RefillState::Scheduled);
        assert_eq!(status().consecutive_failures, 1);
        assert_eq!(status().last_error, Some(Error::Transport));
    }
}

#[test]
fn completed_refill_dismissal_preserves_the_next_attempt() {
    for result in [Ok([37; 32]), Err(Error::Transport)] {
        BUFFER.with_borrow_mut(|state| state.reset(true, 10));
        let generation = BUFFER.with_borrow_mut(|state| state.begin(10)).unwrap();
        let cancellation = refill_cancellation(generation, 10);
        BUFFER.with_borrow_mut(|state| state.finish(generation, result, 11));
        let next = status().next_refill_ns;
        assert!(BUFFER.with_borrow_mut(|state| state.begin(next)).is_some());
        let pending = status();
        DropGuard::dismiss(cancellation);
        assert_eq!(status(), pending);
        assert_eq!(status().refill, RefillState::Pending);
    }
}

#[cfg(panic = "unwind")]
#[test]
fn unwinding_refill_reopens_demand_once() {
    BUFFER.with_borrow_mut(|state| state.reset(true, 10));
    let generation = BUFFER.with_borrow_mut(|state| state.begin(10)).unwrap();
    let result = std::panic::catch_unwind(|| {
        let _cancellation = refill_cancellation(generation, 10);
        panic!("cancel refill while unwinding");
    });
    assert!(result.is_err());
    assert_eq!(status().refill, RefillState::Scheduled);
    assert_eq!(status().consecutive_failures, 1);
    assert_eq!(status().last_error, Some(Error::Transport));
}
