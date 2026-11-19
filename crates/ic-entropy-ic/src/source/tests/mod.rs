use super::*;
use ic_platform_runtime::call::{Completion, Outcome, SubmitError};
use std::{
    cell::Cell,
    future::{pending, poll_fn, ready},
    num::NonZeroU32,
    task::{Context, Poll, Waker},
};

fn complete<T>(future: impl Future<Output = T>) -> T {
    let mut future = std::pin::pin!(future);
    match future
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(result) => result,
        Poll::Pending => panic!("fixture must complete"),
    }
}

#[test]
fn source_returns_platform_seeds_without_consumer_zero_policy() {
    let source = Rc::new(RefCell::new(Source::default()));
    for seed in [[37; 32], [0; 32]] {
        assert_eq!(
            complete(request_with(Rc::clone(&source), ready(Ok(seed)))),
            Ok(seed)
        );
        assert!(!source.borrow().pending);
    }
}

#[test]
fn source_maps_platform_framing_errors_and_releases_custody() {
    let source = Rc::new(RefCell::new(Source::default()));
    // Framing validation belongs to Platform. Entropy projects the typed error
    // while releasing its own single-flight claim.
    for error in [
        ic_platform_protocol::raw_rand::Error::Length,
        ic_platform_protocol::raw_rand::Error::Framing,
        ic_platform_protocol::raw_rand::Error::ByteLimit,
    ] {
        assert_eq!(
            complete(request_with(
                Rc::clone(&source),
                ready(Err(raw_rand::Error::Frame(error))),
            )),
            Err(Error::MalformedReply)
        );
        assert!(!source.borrow().pending);
    }
}

#[test]
fn single_flight_refuses_before_polling_second_operation() {
    let source = Rc::new(RefCell::new(Source::default()));
    let claim = Request::claim(Rc::clone(&source)).unwrap();
    let polled = Cell::new(false);
    assert_eq!(
        complete(request_with(Rc::clone(&source), async {
            polled.set(true);
            Ok([37; 32])
        })),
        Err(Error::Busy)
    );
    assert!(!polled.get());
    drop(claim);
    assert!(!source.borrow().pending);
}

#[test]
fn cancellation_releases_claim_and_transport_failures_never_retry() {
    let source = Rc::new(RefCell::new(Source::default()));
    let mut future = Box::pin(request_with(Rc::clone(&source), pending()));
    assert!(matches!(
        future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop())),
        Poll::Pending
    ));
    assert!(source.borrow().pending);
    drop(future);
    assert!(!source.borrow().pending);
    let rejection = |code| match Completion::from_rejection(
        NonZeroU32::new(code).unwrap(),
        "refused".into(),
        None,
    )
    .into_parts()
    .0
    {
        Outcome::Rejected(error) => raw_rand::Error::Rejected(error),
        Outcome::OutcomeUnknown(error) => raw_rand::Error::OutcomeUnknown(error),
        Outcome::Reply(_) => panic!("fixture must reject"),
    };
    for error in [
        raw_rand::Error::Submission(SubmitError::CallbackCapacity),
        rejection(4),
        rejection(6),
    ] {
        let calls = Cell::new(0);
        assert_eq!(
            complete(request_with(Rc::clone(&source), async {
                calls.set(calls.get() + 1);
                Err(error)
            })),
            Err(Error::Transport)
        );
        assert_eq!(calls.get(), 1);
        assert!(!source.borrow().pending);
    }
}

#[test]
fn stale_reply_and_cleanup_cannot_publish_or_release_replacement_claim() {
    for result in [
        Ok([37; 32]),
        Err(raw_rand::Error::Frame(
            ic_platform_protocol::raw_rand::Error::Length,
        )),
        Err(raw_rand::Error::Submission(SubmitError::CallbackCapacity)),
    ] {
        let source = Rc::new(RefCell::new(Source::default()));
        let released = Cell::new(false);
        let mut result = Some(result);
        let mut old = Box::pin(request_with(
            Rc::clone(&source),
            poll_fn(|_| {
                if released.get() {
                    Poll::Ready(result.take().unwrap())
                } else {
                    Poll::Pending
                }
            }),
        ));
        assert!(matches!(
            old.as_mut().poll(&mut Context::from_waker(Waker::noop())),
            Poll::Pending
        ));
        source.borrow_mut().invalidate(false);
        let replacement = Request::claim(Rc::clone(&source)).unwrap();
        released.set(true);
        assert_eq!(complete(old), Err(Error::Stale));
        assert!(source.borrow().pending);
        drop(replacement);
        assert!(!source.borrow().pending);
    }
}

#[test]
fn suspended_source_refuses_without_polling_operation() {
    let source = Rc::new(RefCell::new(Source::default()));
    source.borrow_mut().invalidate(true);
    assert_eq!(
        complete(request_with(Rc::clone(&source), async {
            panic!("suspended operation")
        })),
        Err(Error::Disabled)
    );
    source.borrow_mut().invalidate(false);
    assert_eq!(
        complete(request_with(source, ready(Ok([9; 32])))),
        Ok([9; 32])
    );
}

#[test]
fn native_public_api_refuses_without_mutating_source() {
    let generation = SOURCE.with(|source| source.borrow().generation);
    assert_eq!(complete(fresh_seed()), Err(Error::NonReplicated));
    assert_eq!(crate::suspend(), Err(Error::NonReplicated));
    assert_eq!(SOURCE.with(|source| source.borrow().generation), generation);
}
