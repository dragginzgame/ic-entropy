//! Controlled interleavings around real management requests; no mocked entropy.

use super::*;
use ic_platform::management_canister_types::{HttpMethod, HttpRequestArgs};
use std::{
    future::{Future, poll_fn},
    task::Poll,
};

#[cdk::update(
    decode_with = "decode::<_, { CONTEND }>",
    encode_with = "encode::<_, { CONTEND }>"
)]
async fn contend() -> bool {
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
    let args = HttpRequestArgs {
        url: "https://entropy-probe.invalid/hold".into(),
        max_response_bytes: Some(256),
        method: HttpMethod::GET,
        headers: Vec::new(),
        body: None,
        transform: None,
        is_replicated: None,
        pricing_version: Some(1),
    };
    let cycles = cdk::api::cost_http_request(args.url.len() as u64, 256);
    cdk::call::Call::unbounded_wait(
        ic_platform_types::Principal::management_canister(),
        "http_request",
    )
    .take_raw_args(ic_platform::candid_codec::encode_one(&args, 4096).unwrap())
    .with_cycles(cycles)
    .await
    .unwrap();
    request.await.is_ok()
}

#[cdk::query(
    decode_with = "decode::<_, { SOURCE_STATE }>",
    encode_with = "encode::<_, { SOURCE_STATE }>"
)]
fn source_state() -> SourceState {
    let status = entropy::status();
    SourceState {
        failures: status.consecutive_failures,
        busy: status.last_error == Some(Error::Busy),
        seeds: status.buffered_seeds as u32,
    }
}

#[cdk::update(
    decode_with = "decode::<_, { CANCEL }>",
    encode_with = "encode::<_, { CANCEL }>"
)]
async fn cancel() -> bool {
    let mut request = Box::pin(entropy::fresh_seed());
    poll_fn(|cx| {
        assert!(request.as_mut().poll(cx).is_pending());
        Poll::Ready(())
    })
    .await;
    drop(request);
    entropy::fresh_seed().await.is_ok()
}

#[cdk::update(
    decode_with = "decode::<_, { STALE }>",
    encode_with = "encode::<_, { STALE }>"
)]
async fn stale() -> bool {
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
