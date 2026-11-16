//! Controlled interleavings around real management requests; no mocked entropy.

use super::{Error, decode, encode, entropy, fail, guard, runtime};
use crate::{CANCEL, CONTEND, SOURCE_STATE, STALE, SourceState};
use std::{
    future::{Future, poll_fn},
    task::Poll,
};

#[runtime::update(
    runtime = runtime,
    name = "contend",
    guard = guard,
    decode = decode::<_, { CONTEND }>,
    encode = encode::<_, { CONTEND }>,
    max_bytes = 4096,
    on_error = fail
)]
async fn contend(_input: ()) -> bool {
    entropy::reconstruct().unwrap();
    let mut request = Box::pin(entropy::fresh_seed());
    poll_fn(|cx| {
        assert!(request.as_mut().poll(cx).is_pending());
        Poll::Ready(())
    })
    .await;
    assert_eq!(entropy::fresh_seed().await, Err(Error::Busy));
    // Keep source custody even after the management reply arrives, until the
    // driver has observed a real timer refill refuse with Busy.
    // A real outstanding call keeps the ingress context alive. PocketIC holds
    // this HTTP reply; it does not replace or mock the raw_rand response.
    let url = "https://entropy-probe.invalid/hold";
    let args =
        ic_platform_protocol::http_request::encode_http_get_args_v1(url, None, 256, true, 4096)
            .unwrap();
    let cycles = runtime::raw::cost_http_request_v1(url.len() as u64, 256);
    let completion = runtime::call::Call::new(
        ic_platform_types::Principal::management_canister(),
        "http_request",
        runtime::call::Wait::Unbounded,
    )
    .owned_args(args)
    .cycles(cycles)
    .execute()
    .await
    .unwrap();
    assert!(matches!(
        completion.outcome(),
        runtime::call::Outcome::Reply(_)
    ));
    request.await.is_ok()
}

#[runtime::query(
    runtime = runtime,
    name = "source_state",
    guard = guard,
    decode = decode::<_, { SOURCE_STATE }>,
    encode = encode::<_, { SOURCE_STATE }>,
    max_bytes = 4096,
    on_error = fail
)]
fn source_state(_input: ()) -> SourceState {
    let status = entropy::status();
    SourceState {
        failures: status.consecutive_failures,
        busy: status.last_error == Some(Error::Busy),
        seeds: status.buffered_seeds as u32,
    }
}

#[runtime::update(
    runtime = runtime,
    name = "cancel",
    guard = guard,
    decode = decode::<_, { CANCEL }>,
    encode = encode::<_, { CANCEL }>,
    max_bytes = 4096,
    on_error = fail
)]
async fn cancel(_input: ()) -> bool {
    let mut request = Box::pin(entropy::fresh_seed());
    poll_fn(|cx| {
        assert!(request.as_mut().poll(cx).is_pending());
        Poll::Ready(())
    })
    .await;
    drop(request);
    entropy::fresh_seed().await.is_ok()
}

#[runtime::update(
    runtime = runtime,
    name = "stale",
    guard = guard,
    decode = decode::<_, { STALE }>,
    encode = encode::<_, { STALE }>,
    max_bytes = 4096,
    on_error = fail
)]
async fn stale(_input: ()) -> bool {
    let mut old = Box::pin(entropy::fresh_seed());
    poll_fn(|cx| {
        assert!(old.as_mut().poll(cx).is_pending());
        Poll::Ready(())
    })
    .await;
    entropy::suspend().unwrap();
    entropy::resume_fresh_requests().unwrap();
    let mut replacement = Box::pin(entropy::fresh_seed());
    poll_fn(|cx| {
        assert!(replacement.as_mut().poll(cx).is_pending());
        Poll::Ready(())
    })
    .await;
    assert_eq!(old.await, Err(Error::Stale));
    // Old completion/cleanup must not release the replacement's source claim.
    assert_eq!(entropy::fresh_seed().await, Err(Error::Busy));
    assert_eq!(entropy::status().buffered_seeds, 0);
    replacement.await.is_ok()
}
