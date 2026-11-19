//! Private Wire protocol and standalone canister for installed entropy qualification.

use ic_entropy_ic::{self as entropy, Error, RefillState};
use ic_platform_runtime as runtime;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    future::{Future, poll_fn},
    task::Poll,
};

// init
// Reconstruct fresh volatile entropy after installation.
// guard  : guard (unrestricted private probe)
// limits : request Wire LIMITS (4096 bytes); no reply
#[runtime::init(
    runtime = runtime,
    guard = guard,
    decode = decode::<_, { INIT }>,
    max_bytes = 4096,
    on_error = fail
)]
fn init(_input: ()) {
    entropy::reconstruct().unwrap();
}

// post_upgrade
// Reconstruct entropy after an upgrade without restoring seeds or stream state.
// guard  : guard (unrestricted private probe)
// limits : request Wire LIMITS (4096 bytes); no reply
#[runtime::post_upgrade(
    runtime = runtime,
    guard = guard,
    decode = decode::<_, { INIT }>,
    max_bytes = 4096,
    on_error = fail
)]
fn post_upgrade(_input: ()) {
    entropy::reconstruct().unwrap();
}

// inspect
// Observe readiness and budget without consuming randomness.
// guard  : guard (unrestricted private probe)
// limits : request Wire LIMITS (4096 bytes); reply Wire LIMITS (4096 bytes)
#[runtime::query(
    runtime = runtime,
    name = "inspect",
    guard = guard,
    decode = decode::<_, { INSPECT }>,
    encode = encode::<_, { INSPECT }>,
    max_bytes = 4096,
    on_error = fail
)]
fn inspect(_input: ()) -> State {
    let status = entropy::status();

    State {
        seeds: status.buffered_seeds as u32,
        remaining: status.remaining_bytes,
        idle: status.refill == RefillState::Idle,
        disabled: status.refill == RefillState::Disabled,
    }
}

// take
// Consume one buffered seed and return the resulting pool state.
// guard  : guard (unrestricted private probe)
// limits : request Wire LIMITS (4096 bytes); reply Wire LIMITS (4096 bytes)
#[runtime::update(
    runtime = runtime,
    name = "take",
    guard = guard,
    decode = decode::<_, { TAKE }>,
    encode = encode::<_, { TAKE }>,
    max_bytes = 4096,
    on_error = fail
)]
fn take(_input: ()) -> ([u8; 32], State) {
    let seed = entropy::take_seed().unwrap();

    (seed, inspect(()))
}

// draw
// Draw a bounded buffer and return its remaining expansion budget.
// guard  : guard (unrestricted private probe)
// limits : request Wire LIMITS (4096 bytes); reply Wire LIMITS (4096 bytes)
#[runtime::update(
    runtime = runtime,
    name = "draw",
    guard = guard,
    decode = decode::<_, { DRAW }>,
    encode = encode::<_, { DRAW }>,
    max_bytes = 4096,
    on_error = fail
)]
fn draw(_input: ()) -> (ic_wire::ByteBuf, u64) {
    let mut bytes = vec![0; 1024];
    entropy::fill_bytes(&mut bytes).unwrap();

    (bytes.into(), entropy::status().remaining_bytes)
}

// fresh
// Request new management randomness without consuming buffered entropy.
// guard  : guard (unrestricted private probe)
// limits : request Wire LIMITS (4096 bytes); reply Wire LIMITS (4096 bytes)
#[runtime::update(
    runtime = runtime,
    name = "fresh",
    guard = guard,
    decode = decode::<_, { FRESH }>,
    encode = encode::<_, { FRESH }>,
    max_bytes = 4096,
    on_error = fail
)]
async fn fresh(_input: ()) -> [u8; 32] {
    entropy::fresh_seed().await.unwrap()
}

// suspend
// Discard volatile entropy and stop background acquisition.
// guard  : guard (unrestricted private probe)
// limits : request Wire LIMITS (4096 bytes); reply Wire LIMITS (4096 bytes)
#[runtime::update(
    runtime = runtime,
    name = "suspend",
    guard = guard,
    decode = decode::<_, { SUSPEND }>,
    encode = encode_unit::<{ SUSPEND }>,
    max_bytes = 4096,
    on_error = fail
)]
fn suspend(_input: ()) {
    entropy::suspend().unwrap();
}

// resume
// Reconstruct acquisition and resume background refill.
// guard  : guard (unrestricted private probe)
// limits : request Wire LIMITS (4096 bytes); reply Wire LIMITS (4096 bytes)
#[runtime::update(
    runtime = runtime,
    name = "resume",
    guard = guard,
    decode = decode::<_, { RESUME }>,
    encode = encode_unit::<{ RESUME }>,
    max_bytes = 4096,
    on_error = fail
)]
fn resume(_input: ()) {
    entropy::reconstruct().unwrap();
}

// query_refusal
// Verify that query execution refuses mutating draws without changing output.
// guard  : guard (unrestricted private probe)
// limits : request Wire LIMITS (4096 bytes); reply Wire LIMITS (4096 bytes)
#[runtime::query(
    runtime = runtime,
    name = "query_refusal",
    guard = guard,
    decode = decode::<_, { QUERY_REFUSAL }>,
    encode = encode::<_, { QUERY_REFUSAL }>,
    max_bytes = 4096,
    on_error = fail
)]
fn query_refusal(_input: ()) -> bool {
    let mut bytes = [7; 32];

    entropy::fill_bytes(&mut bytes) == Err(Error::NonReplicated) && bytes == [7; 32]
}

// oversized_refusal
// Verify that oversized draws preserve output and buffered state.
// guard  : guard (unrestricted private probe)
// limits : request Wire LIMITS (4096 bytes); reply Wire LIMITS (4096 bytes)
#[runtime::update(
    runtime = runtime,
    name = "oversized_refusal",
    guard = guard,
    decode = decode::<_, { OVERSIZED_REFUSAL }>,
    encode = encode::<_, { OVERSIZED_REFUSAL }>,
    max_bytes = 4096,
    on_error = fail
)]
fn oversized_refusal(_input: ()) -> bool {
    let before = entropy::status();
    let mut bytes = [7; 1025];

    entropy::fill_bytes(&mut bytes) == Err(Error::DrawTooLarge)
        && bytes == [7; 1025]
        && before == entropy::status()
}

// Runtime 0.23.22 omits HTTP request pricing from the native Wire client.
// contend
// Hold a real request while the driver observes exclusive source custody.
// guard  : guard (unrestricted private probe)
// limits : request Wire LIMITS (4096 bytes); reply Wire LIMITS (4096 bytes)
#[cfg(target_arch = "wasm32")]
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

// source_state
// Observe source contention and refill recovery without consuming entropy.
// guard  : guard (unrestricted private probe)
// limits : request Wire LIMITS (4096 bytes); reply Wire LIMITS (4096 bytes)
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

// cancel
// Drop an accepted request and verify that a replacement can acquire the source.
// guard  : guard (unrestricted private probe)
// limits : request Wire LIMITS (4096 bytes); reply Wire LIMITS (4096 bytes)
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

// stale
// Verify that an old completion cannot release a replacement request claim.
// guard  : guard (unrestricted private probe)
// limits : request Wire LIMITS (4096 bytes); reply Wire LIMITS (4096 bytes)
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

/// Lifecycle initialization and post-upgrade request.
pub const INIT: u64 = 1;
/// Inspect buffered adapter state.
pub const INSPECT: u64 = 2;
/// Consume one buffered seed.
pub const TAKE: u64 = 3;
/// Draw 1024 bytes from the shared generator.
pub const DRAW: u64 = 4;
/// Acquire a fresh management-canister seed.
pub const FRESH: u64 = 5;
/// Suspend acquisition and discard buffered state.
pub const SUSPEND: u64 = 6;
/// Reconstruct the adapter and resume timer refill.
pub const RESUME: u64 = 7;
/// Check that queries cannot consume randomness.
pub const QUERY_REFUSAL: u64 = 8;
/// Check that oversized draws preserve output and adapter state.
pub const OVERSIZED_REFUSAL: u64 = 9;
/// Hold a fresh request while checking exclusive source custody.
pub const CONTEND: u64 = 10;
/// Inspect source contention and refill failures.
pub const SOURCE_STATE: u64 = 12;
/// Cancel a pending request and check source recovery.
pub const CANCEL: u64 = 13;
/// Check that an old completion cannot release a replacement request's custody.
pub const STALE: u64 = 14;

/// Observable acquisition state while a real management call is outstanding.
#[derive(Debug, Deserialize, Serialize)]
pub struct SourceState {
    /// Consecutive background refill failures.
    pub failures: u8,
    /// Whether the last refill failed because the source was busy.
    pub busy: bool,
    /// Number of buffered seeds.
    pub seeds: u32,
}

/// Encoding and decoding bounds shared by the probe and its installed driver.
pub const LIMITS: ic_wire::Limits = ic_wire::Limits {
    bytes: 4096,
    allocation: 65536,
    nodes: 4096,
    depth: 8,
};

/// Buffered adapter state returned by the probe.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    /// Number of buffered seeds.
    pub seeds: u32,
    /// Remaining shared-generator output budget in bytes.
    pub remaining: u64,
    /// Whether refill is enabled and has no outstanding request.
    pub idle: bool,
    /// Whether background refill is disabled.
    pub disabled: bool,
}

fn decode<T: DeserializeOwned, const ID: u64>(bytes: &[u8]) -> Result<T, &'static str> {
    ic_wire::decode(
        bytes,
        ic_wire::Encoding::Frame {
            method: ID,
            direction: ic_wire::Direction::Request,
        },
        LIMITS,
    )
    .map_err(|_| "fixture request")
}

fn encode<T: Serialize, const ID: u64>(value: T) -> Result<Vec<u8>, &'static str> {
    ic_wire::encode(
        &value,
        ic_wire::Encoding::Frame {
            method: ID,
            direction: ic_wire::Direction::Response,
        },
        LIMITS,
    )
    .map_err(|_| "fixture response")
}

fn encode_unit<const ID: u64>((): ()) -> Result<Vec<u8>, &'static str> {
    encode::<_, ID>(())
}

/// Admit every caller to the private probe; hosts own production consumer admission.
fn guard() -> Result<(), std::convert::Infallible> {
    Ok(())
}

fn fail<E: std::fmt::Debug>(error: E) -> ! {
    runtime::system::trap(&format!("{error:?}"))
}
