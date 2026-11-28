//! Private Wire protocol and standalone canister for installed entropy qualification.

use ic_entropy::{self as entropy, Error, RefillState};
use ic_platform_runtime as runtime;
use ic_wire::{Message, Wire, evolution::inline::Inline};
use std::{
    cell::RefCell,
    future::{Future, poll_fn},
    num::NonZeroUsize,
    rc::Rc,
    task::Poll,
};

// init
// Reconstruct fresh volatile entropy after installation.
// guard  : guard (unrestricted private probe)
// limits : request Wire LIMITS (4096 bytes); no reply
#[runtime::init(
    runtime = runtime,
    guard = guard,
    decode = decode::<{ INIT }>,
    max_bytes = 4096,
    on_error = fail
)]
fn init(_input: ()) {
    reconstruct_entropy();
}

// post_upgrade
// Reconstruct entropy after an upgrade without restoring seeds or stream state.
// guard  : guard (unrestricted private probe)
// limits : request Wire LIMITS (4096 bytes); no reply
#[runtime::post_upgrade(
    runtime = runtime,
    guard = guard,
    decode = decode::<{ INIT }>,
    max_bytes = 4096,
    on_error = fail
)]
fn post_upgrade(_input: ()) {
    reconstruct_entropy();
}

// inspect
// Observe readiness and budget without consuming randomness.
// guard  : guard (unrestricted private probe)
// limits : request Wire LIMITS (4096 bytes); reply Wire LIMITS (4096 bytes)
#[runtime::query(
    runtime = runtime,
    name = "inspect",
    guard = guard,
    decode = decode::<{ INSPECT }>,
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
    decode = decode::<{ TAKE }>,
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
    decode = decode::<{ DRAW }>,
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
    decode = decode::<{ FRESH }>,
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
    decode = decode::<{ SUSPEND }>,
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
    decode = decode::<{ RESUME }>,
    encode = encode_unit::<{ RESUME }>,
    max_bytes = 4096,
    on_error = fail
)]
fn resume(_input: ()) {
    reconstruct_entropy();
}

// metrics
// Read host-owned metrics and verify that the adapter shares the same store.
// guard  : guard (unrestricted private probe)
// limits : request Wire LIMITS (4096 bytes); reply Wire LIMITS (4096 bytes)
#[runtime::query(
    runtime = runtime,
    name = "metrics",
    guard = guard,
    decode = decode::<{ METRICS }>,
    encode = encode::<_, { METRICS }>,
    max_bytes = 4096,
    on_error = fail
)]
fn metrics(_input: ()) -> MetricState {
    let host = ENTROPY_METRICS.with(|storage| storage.snapshot());
    let current = entropy::metrics().unwrap();
    let overflowed = [
        host.fresh.attempts,
        host.fresh.succeeded,
        host.fresh.failed,
        host.fresh.cancelled,
        host.fresh.latency_unavailable,
        host.refill.succeeded,
        host.refill.failed,
        host.seeds_consumed,
        host.bytes_generated,
        host.draws_succeeded,
        host.draw_refusals,
        host.busy_refusals,
        host.stale_replies,
    ]
    .into_iter()
    .any(|counter| counter.total_overflowed())
        || host.fresh.latency_ns.aggregate().samples_overflowed();

    MetricState {
        shared: host == current,
        overflowed,
        fresh_attempts: host.fresh.attempts.total(),
        fresh_succeeded: host.fresh.succeeded.total(),
        fresh_failed: host.fresh.failed.total(),
        fresh_cancelled: host.fresh.cancelled.total(),
        fresh_latency_samples: host.fresh.latency_ns.aggregate().samples(),
        fresh_latency_unavailable: host.fresh.latency_unavailable.total(),
        refill_succeeded: host.refill.succeeded.total(),
        refill_failed: host.refill.failed.total(),
        seeds_consumed: host.seeds_consumed.total(),
        bytes_generated: host.bytes_generated.total(),
        draws_succeeded: host.draws_succeeded.total(),
        draw_refusals: host.draw_refusals.total(),
        busy_refusals: host.busy_refusals.total(),
        stale_replies: host.stale_replies.total(),
    }
}

// metrics_binding
// Refuse a different live metrics store without erasing host observations.
// guard  : guard (unrestricted private probe)
// limits : request Wire LIMITS (4096 bytes); reply Wire LIMITS (4096 bytes)
#[runtime::update(
    runtime = runtime,
    name = "metrics_binding",
    guard = guard,
    decode = decode::<{ METRICS_BINDING }>,
    encode = encode::<_, { METRICS_BINDING }>,
    max_bytes = 4096,
    on_error = fail
)]
fn metrics_binding(_input: ()) -> bool {
    let before = entropy::metrics();

    entropy::initialize_with_metrics(Rc::default()) == Err(Error::Busy)
        && entropy::metrics() == before
}

// logs
// Read a bounded page from the same canonical log used by host observations.
// guard  : guard (unrestricted private probe)
// limits : request Wire LIMITS (4096 bytes); reply Wire LIMITS (4096 bytes)
#[runtime::query(
    runtime = runtime,
    name = "logs",
    guard = guard,
    decode = decode::<{ LOGS }>,
    encode = encode::<_, { LOGS }>,
    max_bytes = 4096,
    on_error = fail
)]
fn logs(_input: ()) -> Vec<(String, String)> {
    HOST_LOG.with_borrow(|log| {
        log.page(
            ic_logging::Filter::default(),
            ic_logging::PageRequest {
                before: None,
                max_results: NonZeroUsize::new(16).unwrap(),
                max_scanned: NonZeroUsize::new(16).unwrap(),
            },
        )
        .entries
        .into_iter()
        .map(|entry| (entry.source.clone(), entry.message.clone()))
        .collect()
    })
}

// query_refusal
// Verify that query execution refuses mutating draws without changing output.
// guard  : guard (unrestricted private probe)
// limits : request Wire LIMITS (4096 bytes); reply Wire LIMITS (4096 bytes)
#[runtime::query(
    runtime = runtime,
    name = "query_refusal",
    guard = guard,
    decode = decode::<{ QUERY_REFUSAL }>,
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
    decode = decode::<{ OVERSIZED_REFUSAL }>,
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

// contend
// Hold a real request while the driver observes exclusive source custody.
// guard  : guard (unrestricted private probe)
// limits : request Wire LIMITS (4096 bytes); reply Wire LIMITS (4096 bytes)
#[runtime::update(
    runtime = runtime,
    name = "contend",
    guard = guard,
    decode = decode::<{ CONTEND }>,
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
    decode = decode::<{ SOURCE_STATE }>,
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
    decode = decode::<{ CANCEL }>,
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
    decode = decode::<{ STALE }>,
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
/// Observe the host-owned metric source.
pub const METRICS: u64 = 15;
/// Verify refusal of a different live metrics store.
pub const METRICS_BINDING: u64 = 16;
/// Observe a bounded page from the host-owned shared log.
pub const LOGS: u64 = 17;

///
/// MetricState
///
/// Bounded host projection of selected metrics; recorder types stay out of Wire.
///

#[derive(Debug, Wire)]
pub struct MetricState {
    /// The host and adapter read identical observations.
    pub shared: bool,
    /// Any projected counter lost exactness; the driver refuses such a report.
    pub overflowed: bool,
    /// Explicit source attempts.
    pub fresh_attempts: u128,
    /// Explicit source successes.
    pub fresh_succeeded: u128,
    /// Explicit source failures.
    pub fresh_failed: u128,
    /// Explicit source cancellations.
    pub fresh_cancelled: u128,
    /// Completed measured explicit calls.
    pub fresh_latency_samples: u64,
    /// Explicit calls without usable latency clocks.
    pub fresh_latency_unavailable: u128,
    /// Background source successes.
    pub refill_succeeded: u128,
    /// Background source failures.
    pub refill_failed: u128,
    /// Seeds consumed by direct takes and generator reseeding.
    pub seeds_consumed: u128,
    /// Shared-generator output bytes.
    pub bytes_generated: u128,
    /// Successful shared-generator calls.
    pub draws_succeeded: u128,
    /// Refused replicated shared-generator calls.
    pub draw_refusals: u128,
    /// Source contention refusals.
    pub busy_refusals: u128,
    /// Stale source completions.
    pub stale_replies: u128,
}

///
/// SourceState
///
/// Observable acquisition state while a real management call is outstanding.
///

#[derive(Debug, Wire)]
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

///
/// State
///
/// Buffered adapter state returned by the probe.
///

#[derive(Debug, PartialEq, Eq, Wire)]
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

thread_local! {
    static HOST_LOG: RefCell<ic_logging::Log> = const {
        RefCell::new(ic_logging::Log::new(ic_logging::Limits {
            max_entries: NonZeroUsize::new(32).unwrap(),
            max_label_bytes: 32,
            max_message_bytes: 160,
            max_text_bytes: 8192,
        }))
    };
    static ENTROPY_METRICS: Rc<entropy::MetricsStorage> = Rc::default();
}

fn host_record(level: ic_logging::Level, source: &'static str, message: &str) {
    // Read the adapter while reporting, just as a composed host may do.
    let _ = entropy::status();
    HOST_LOG.with_borrow_mut(|log| {
        let _ = log.append(ic_logging::EntryInput {
            created_at_secs: runtime::system::time() / 1_000_000_000,
            level,
            source,
            topic: None,
            message,
        });
    });
}

fn reconstruct_entropy() {
    entropy::set_log_sink(Some(host_record)).unwrap();
    host_record(
        ic_logging::Level::Info,
        "entropy-probe",
        "Host lifecycle setup",
    );
    ENTROPY_METRICS.with(|storage| entropy::initialize_with_metrics(Rc::clone(storage)).unwrap());
    entropy::reconstruct().unwrap();
}

fn decode<const ID: u64>(bytes: &[u8]) -> Result<(), &'static str> {
    ic_wire::decode::<Message<()>>(
        bytes,
        ic_wire::Encoding::Frame {
            method: ID,
            direction: ic_wire::Direction::Request,
        },
        LIMITS,
    )
    .map(|Message(value)| value)
    .map_err(|_| "fixture request")
}

fn encode<T: for<'de> Inline<'de>, const ID: u64>(value: T) -> Result<Vec<u8>, &'static str> {
    ic_wire::encode(
        &Message(value),
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
