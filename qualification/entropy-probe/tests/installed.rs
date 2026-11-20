//! Real management randomness and global-timer delivery, without a consumer framework.

use entropy_probe::*;
use ic_testkit::{
    pocket_ic::{
        CanisterId as Principal, PocketIcBuilder,
        common::rest::{CanisterHttpReply, CanisterHttpResponse, MockCanisterHttpResponse},
        nonblocking::PocketIc,
    },
    server::{Deletion, Limits as ServerLimits, PreparedServer, Server},
};
use serde::de::DeserializeOwned;
use std::{path::PathBuf, time::Duration};

const EXPECTED_POOL_CAPACITY: u32 = 256;

fn prepared_server() -> PreparedServer {
    let directory =
        PathBuf::from(std::env::var_os("ENTROPY_POCKET_IC_DIRECTORY").expect(
            "run make install-qualification-tools and select the prepared server directory",
        ));

    PreparedServer::check(&directory).unwrap()
}

#[test]
fn prepared_server_is_admitted() {
    prepared_server();
}

fn request(id: u64) -> Vec<u8> {
    ic_wire::encode(
        &(),
        ic_wire::Encoding::Frame {
            method: id,
            direction: ic_wire::Direction::Request,
        },
        LIMITS,
    )
    .unwrap()
}

async fn update<T: DeserializeOwned>(pic: &PocketIc, pid: Principal, name: &str, id: u64) -> T {
    let bytes = pic
        .update_call(pid, Principal::anonymous(), name, request(id))
        .await
        .unwrap();

    ic_wire::decode(
        &bytes,
        ic_wire::Encoding::Frame {
            method: id,
            direction: ic_wire::Direction::Response,
        },
        LIMITS,
    )
    .unwrap()
}

async fn query<T: DeserializeOwned>(pic: &PocketIc, pid: Principal, name: &str, id: u64) -> T {
    let bytes = pic
        .query_call(pid, Principal::anonymous(), name, request(id))
        .await
        .unwrap();

    ic_wire::decode(
        &bytes,
        ic_wire::Encoding::Frame {
            method: id,
            direction: ic_wire::Direction::Response,
        },
        LIMITS,
    )
    .unwrap()
}

async fn inspect(pic: &PocketIc, pid: Principal) -> State {
    query(pic, pid, "inspect", INSPECT).await
}

async fn metrics(pic: &PocketIc, pid: Principal) -> MetricState {
    let state: MetricState = query(pic, pid, "metrics", METRICS).await;
    assert!(state.shared);
    assert!(!state.overflowed);

    state
}

async fn logs(pic: &PocketIc, pid: Principal) -> Vec<(String, String)> {
    query(pic, pid, "logs", LOGS).await
}

fn has_log(rows: &[(String, String)], source: &str, message: &str) -> bool {
    rows.iter().any(|row| row.0 == source && row.1 == message)
}

async fn advance(pic: &PocketIc, seconds: u64) {
    pic.advance_time(Duration::from_secs(seconds)).await;
    for _ in 0..5 {
        pic.tick().await;
    }
}

async fn full_pool(pic: &PocketIc, pid: Principal) -> State {
    for _ in 0..EXPECTED_POOL_CAPACITY + 40 {
        let state = inspect(pic, pid).await;
        if state.seeds == EXPECTED_POOL_CAPACITY && state.idle {
            return state;
        }
        advance(pic, 1).await;
    }

    panic!(
        "refill did not reach idle capacity: {:?}",
        inspect(pic, pid).await
    );
}

#[tokio::test]
async fn background_refill_and_lifecycle() {
    let path = PathBuf::from(
        std::env::var_os("ENTROPY_PROBE_WASM")
            .expect("run make test-installed to build and select the probe artifact"),
    );
    let wasm = std::fs::read(path).unwrap();
    let mut server = Server::start(
        prepared_server(),
        ServerLimits {
            startup: Duration::from_secs(30),
            lifetime: Duration::from_secs(180),
            stdout_bytes: 1024 * 1024,
            stderr_bytes: 1024 * 1024,
        },
    )
    .unwrap();
    let owned = server
        .instance(
            PocketIcBuilder::new().with_application_subnet(),
            Duration::from_secs(120),
        )
        .await
        .unwrap();
    let pic = owned.client();
    let pid = pic.create_canister().await;
    pic.add_cycles(pid, 10_000_000_000_000).await;
    pic.install_canister(pid, wasm.clone(), request(INIT), None)
        .await;

    // Only queries/time/ticks precede readiness: the pool must fill in the background.
    let full = full_pool(pic, pid).await;
    assert!(metrics(pic, pid).await.refill_succeeded >= u128::from(EXPECTED_POOL_CAPACITY));
    assert!(update::<bool>(pic, pid, "metrics_binding", METRICS_BINDING).await);
    let initial_logs = logs(pic, pid).await;
    assert_eq!(initial_logs.len(), 2);
    assert!(has_log(
        &initial_logs,
        "entropy-probe",
        "Host lifecycle setup"
    ));
    assert!(has_log(
        &initial_logs,
        "ic-entropy",
        "Entropy buffer reconstructed"
    ));
    assert_eq!(full.remaining, 0);
    advance(pic, 30).await;
    assert_eq!(inspect(pic, pid).await, full);
    assert!(query::<bool>(pic, pid, "query_refusal", QUERY_REFUSAL).await);
    assert_eq!(inspect(pic, pid).await, full);

    let (_, after_take): ([u8; 32], State) = update(pic, pid, "take", TAKE).await;
    assert_eq!(after_take.seeds, EXPECTED_POOL_CAPACITY - 1);
    assert!(!after_take.idle);
    full_pool(pic, pid).await;
    let (bytes, budget): (ic_wire::ByteBuf, u64) = update(pic, pid, "draw", DRAW).await;
    assert_eq!(bytes.len(), 1024);
    assert_eq!(budget, 65536 - 1024);
    assert!(update::<bool>(pic, pid, "oversized_refusal", OVERSIZED_REFUSAL).await);
    let consumption = metrics(pic, pid).await;
    assert_eq!(consumption.seeds_consumed, 2);
    assert_eq!(consumption.bytes_generated, 1024);
    assert_eq!(consumption.draws_succeeded, 1);
    assert_eq!(consumption.draw_refusals, 1);

    // A fresh call must not spend the buffered generator or consume pool seeds.
    let before_fresh = full_pool(pic, pid).await;
    let before = metrics(pic, pid).await;
    let _: [u8; 32] = update(pic, pid, "fresh", FRESH).await;
    assert_eq!(inspect(pic, pid).await, before_fresh);
    assert_eq!(logs(pic, pid).await, initial_logs);
    let after = metrics(pic, pid).await;
    assert_eq!(after.fresh_attempts, before.fresh_attempts + 1);
    assert_eq!(after.fresh_succeeded, before.fresh_succeeded + 1);
    assert_eq!(after.seeds_consumed, before.seeds_consumed);
    assert_eq!(after.bytes_generated, before.bytes_generated);
    assert_eq!(
        after.fresh_latency_samples,
        before.fresh_latency_samples + 1
    );
    assert_eq!(after.fresh_latency_unavailable, 0);

    assert!(update::<bool>(pic, pid, "cancel", CANCEL).await);
    assert_eq!(inspect(pic, pid).await, before_fresh);
    let cancelled = metrics(pic, pid).await;
    assert_eq!(cancelled.fresh_attempts, after.fresh_attempts + 2);
    assert_eq!(cancelled.fresh_cancelled, after.fresh_cancelled + 1);
    assert_eq!(cancelled.fresh_succeeded, after.fresh_succeeded + 1);
    assert!(update::<bool>(pic, pid, "stale", STALE).await);
    assert!(inspect(pic, pid).await.disabled);
    assert!(has_log(
        &logs(pic, pid).await,
        "ic-entropy",
        "Stale entropy reply discarded"
    ));
    let stale = metrics(pic, pid).await;
    assert_eq!(stale.fresh_attempts, cancelled.fresh_attempts + 3);
    assert_eq!(stale.stale_replies, cancelled.stale_replies + 1);
    assert_eq!(stale.busy_refusals, cancelled.busy_refusals + 1);

    let held = pic
        .submit_call(pid, Principal::anonymous(), "contend", request(CONTEND))
        .await
        .unwrap();
    let mut observed = false;
    for _ in 0..20 {
        advance(pic, 1).await;
        let state: SourceState = query(pic, pid, "source_state", SOURCE_STATE).await;
        if state.failures > 0 && state.busy {
            assert_eq!(state.seeds, 0);
            observed = true;
            break;
        }
    }
    assert!(
        observed,
        "timer refill did not encounter held source custody"
    );
    let barrier = pic
        .get_canister_http()
        .await
        .into_iter()
        .find(|call| call.url == "https://entropy-probe.invalid/hold")
        .expect("held HTTP barrier");
    pic.mock_canister_http_response(MockCanisterHttpResponse {
        subnet_id: barrier.subnet_id,
        request_id: barrier.request_id,
        response: CanisterHttpResponse::CanisterHttpReply(CanisterHttpReply {
            status: 200,
            headers: Vec::new(),
            body: Vec::new(),
        }),
        additional_responses: Vec::new(),
    })
    .await;
    let reply = pic.await_call(held).await.unwrap();
    assert!(
        ic_wire::decode::<bool>(
            &reply,
            ic_wire::Encoding::Frame {
                method: CONTEND,
                direction: ic_wire::Direction::Response
            },
            LIMITS
        )
        .unwrap()
    );
    full_pool(pic, pid).await;
    let recovered: SourceState = query(pic, pid, "source_state", SOURCE_STATE).await;
    assert!(!recovered.busy);
    assert_eq!(recovered.failures, 0);
    assert!(metrics(pic, pid).await.refill_failed > 0);

    assert!(has_log(
        &logs(pic, pid).await,
        "ic-entropy",
        "Entropy refill recovered"
    ));
    let before_suspend = metrics(pic, pid).await;
    update::<()>(pic, pid, "suspend", SUSPEND).await;
    advance(pic, 10).await;
    assert_eq!(
        inspect(pic, pid).await,
        State {
            seeds: 0,
            remaining: 0,
            idle: false,
            disabled: true,
        }
    );
    update::<()>(pic, pid, "resume", RESUME).await;
    full_pool(pic, pid).await;
    assert_eq!(
        metrics(pic, pid).await.fresh_attempts,
        before_suspend.fresh_attempts
    );
    let retained_logs = logs(pic, pid).await;
    assert!(has_log(&retained_logs, "ic-entropy", "Entropy suspended"));
    assert!(has_log(
        &retained_logs,
        "ic-entropy",
        "Entropy refill recovered"
    ));
    assert_eq!(
        retained_logs
            .iter()
            .filter(|row| row.1 == "Host lifecycle setup")
            .count(),
        2
    );
    let _: (ic_wire::ByteBuf, u64) = update(pic, pid, "draw", DRAW).await;

    // Upgrades start a fresh heap and reconstruct the provider and entropy state.
    pic.stop_canister(pid, None).await.unwrap();
    pic.upgrade_canister(pid, wasm.clone(), request(INIT), None)
        .await
        .unwrap();
    pic.start_canister(pid, None).await.unwrap();
    assert_eq!(inspect(pic, pid).await.remaining, 0);
    assert_eq!(metrics(pic, pid).await.fresh_attempts, 0);
    assert_eq!(metrics(pic, pid).await.bytes_generated, 0);
    assert_eq!(logs(pic, pid).await, initial_logs);
    full_pool(pic, pid).await;
    let (_, budget): (ic_wire::ByteBuf, u64) = update(pic, pid, "draw", DRAW).await;
    assert_eq!(budget, 65536 - 1024);
    pic.reinstall_canister(pid, wasm, request(INIT), None)
        .await
        .unwrap();
    assert_eq!(full_pool(pic, pid).await.remaining, 0);
    assert_eq!(metrics(pic, pid).await.bytes_generated, 0);
    assert_eq!(logs(pic, pid).await, initial_logs);
    assert_eq!(
        owned.close(Duration::from_secs(30)).await,
        Deletion::Attempted
    );
    let stopped = server.stop().unwrap();
    assert_eq!(stopped.abandoned, 0);
    assert!(stopped.process.status.is_some());
    assert!(stopped.process.cleanup.is_empty());
}
