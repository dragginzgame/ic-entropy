//! Real management randomness and global-timer delivery, without a consumer framework.

use entropy_probe::*;
use ic_platform_types::Principal;
use ic_testkit::{
    pic::PocketIcStartupConfig,
    pocket_ic::{PocketIc, PocketIcBuilder},
};
use serde::de::DeserializeOwned;
use std::{path::PathBuf, time::Duration};

const EXPECTED_POOL_CAPACITY: u32 = 256;

fn request(id: u64) -> Vec<u8> {
    ic_wire::native::encode_request(id, &(), LIMITS).unwrap()
}

async fn update<T: DeserializeOwned>(pic: &PocketIc, pid: Principal, name: &str, id: u64) -> T {
    let bytes = pic
        .update_call(pid, Principal::anonymous(), name, request(id))
        .await
        .unwrap();
    ic_wire::native::decode_response(id, &bytes, LIMITS).unwrap()
}

async fn query<T: DeserializeOwned>(pic: &PocketIc, pid: Principal, name: &str, id: u64) -> T {
    let bytes = pic
        .query_call(pid, Principal::anonymous(), name, request(id))
        .await
        .unwrap();
    ic_wire::native::decode_response(id, &bytes, LIMITS).unwrap()
}

async fn inspect(pic: &PocketIc, pid: Principal) -> State {
    query(pic, pid, "inspect", INSPECT).await
}

async fn advance(pic: &PocketIc, seconds: u64) {
    pic.advance_time(Duration::from_secs(seconds))
        .await
        .unwrap();
    for _ in 0..5 {
        pic.tick().await.unwrap();
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
    let owned = PocketIcStartupConfig::from_env(Duration::from_secs(30))
        .unwrap()
        .build_instance(PocketIcBuilder::new(Duration::from_secs(120)).with_application_subnet())
        .await
        .unwrap();
    let pic = owned.instance();
    let pid = pic.create_canister().await.unwrap();
    pic.add_cycles(pid, 10_000_000_000_000).await.unwrap();
    pic.install_canister(pid, wasm.clone(), request(INIT), None)
        .await
        .unwrap();

    // Only queries/time/ticks precede readiness: the pool must fill in the background.
    let full = full_pool(pic, pid).await;
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

    // A fresh call must not spend the buffered generator or consume pool seeds.
    let before_fresh = full_pool(pic, pid).await;
    let _: [u8; 32] = update(pic, pid, "fresh", FRESH).await;
    assert_eq!(inspect(pic, pid).await, before_fresh);

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
    let _: (ic_wire::ByteBuf, u64) = update(pic, pid, "draw", DRAW).await;

    // Upgrades start a fresh heap and reconstruct the provider and entropy state.
    pic.upgrade_canister(pid, wasm.clone(), request(INIT), None)
        .await
        .unwrap();
    assert_eq!(inspect(pic, pid).await.remaining, 0);
    full_pool(pic, pid).await;
    let (_, budget): (ic_wire::ByteBuf, u64) = update(pic, pid, "draw", DRAW).await;
    assert_eq!(budget, 65536 - 1024);
    pic.reinstall_canister(pid, wasm, request(INIT), None)
        .await
        .unwrap();
    assert_eq!(full_pool(pic, pid).await.remaining, 0);
    owned.close().await.unwrap();
}
