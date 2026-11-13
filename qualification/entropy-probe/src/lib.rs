//! Private Wire protocol and standalone canister for installed entropy qualification.

#[cfg(target_arch = "wasm32")]
mod canister;

use serde::{Deserialize, Serialize};

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
