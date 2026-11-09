use super::*;
use std::{
    cell::Cell,
    future::{pending, poll_fn, ready},
    task::{Context, Poll, Waker},
};

// Fixed management-canister producer framing: one vec nat8 result.
// Platform owns decoding; the adapter does not need a general Candid dependency.
fn reply(seed: [u8; 32]) -> Vec<u8> {
    let mut bytes = b"DIDL\x01\x6d\x7b\x01\x00\x20".to_vec();
    bytes.extend_from_slice(&seed);
    bytes
}

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
fn source_decodes_platform_protocol_without_consumer_zero_policy() {
    assert_eq!(ic_platform::raw_rand::encode(), b"DIDL\0\0");
    let source = Rc::new(RefCell::new(Source::default()));
    for seed in [[37; 32], [0; 32]] {
        let reply = reply(seed);
        assert_eq!(
            complete(request_with(Rc::clone(&source), ready(Ok(reply)))),
            Ok(seed)
        );
        assert!(!source.borrow().pending);
    }
}

#[test]
fn source_refuses_malformed_truncated_trailing_and_oversized_replies() {
    let source = Rc::new(RefCell::new(Source::default()));
    let valid = reply([37; 32]);
    let mut invalid: Vec<_> = (0..valid.len()).map(|end| valid[..end].to_vec()).collect();
    let mut trailing = valid;
    trailing.push(0);
    let mut short = reply([1; 32]);
    short[9] = 31;
    short.pop();
    let mut long = reply([1; 32]);
    long[9] = 33;
    long.push(1);
    let mut wrong_type = reply([1; 32]);
    wrong_type[6] = 0x71; // text, not nat8
    invalid.extend([trailing, short, long, vec![0; 75], wrong_type]);
    for reply in invalid {
        assert_eq!(
            complete(request_with(Rc::clone(&source), ready(Ok(reply)))),
            Err(Error::MalformedReply)
        );
        assert!(!source.borrow().pending);
    }
}

#[test]
fn single_flight_refuses_before_polling_second_transport() {
    let source = Rc::new(RefCell::new(Source::default()));
    let claim = Request::claim(Rc::clone(&source)).unwrap();
    let polled = Cell::new(false);
    assert_eq!(
        complete(request_with(Rc::clone(&source), async {
            polled.set(true);
            Ok(Vec::new())
        })),
        Err(Error::Busy)
    );
    assert!(!polled.get());
    drop(claim);
    assert!(!source.borrow().pending);
}

#[test]
fn cancellation_releases_claim_and_transport_failure_never_retries() {
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
    let calls = Cell::new(0);
    assert_eq!(
        complete(request_with(Rc::clone(&source), async {
            calls.set(calls.get() + 1);
            Err(Error::Transport)
        })),
        Err(Error::Transport)
    );
    assert_eq!(calls.get(), 1);
    assert!(!source.borrow().pending);
}

#[test]
fn stale_reply_and_cleanup_cannot_publish_or_release_replacement_claim() {
    let source = Rc::new(RefCell::new(Source::default()));
    let released = Cell::new(false);
    let mut old = Box::pin(request_with(
        Rc::clone(&source),
        poll_fn(|_| {
            if released.get() {
                Poll::Ready(Ok(reply([37; 32])))
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

#[test]
fn suspended_source_refuses_without_polling_transport() {
    let source = Rc::new(RefCell::new(Source::default()));
    source.borrow_mut().invalidate(true);
    assert_eq!(
        complete(request_with(Rc::clone(&source), async {
            panic!("suspended transport")
        })),
        Err(Error::Disabled)
    );
    source.borrow_mut().invalidate(false);
    assert_eq!(
        complete(request_with(source, ready(Ok(reply([9; 32]))))),
        Ok([9; 32])
    );
}

#[test]
fn native_public_api_refuses_without_mutating_source() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let generation = SOURCE.with(|source| source.borrow().generation);
        assert_eq!(complete(fresh_seed()), Err(Error::NonReplicated));
        assert_eq!(crate::suspend(), Err(Error::NonReplicated));
        assert_eq!(SOURCE.with(|source| source.borrow().generation), generation);
    }
}
