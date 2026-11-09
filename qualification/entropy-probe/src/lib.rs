//! Private Wire protocol and standalone canister for installed entropy qualification.

#[cfg(target_arch = "wasm32")]
mod canister;

use serde::{Deserialize, Serialize};

pub const INIT: u64 = 1;
pub const INSPECT: u64 = 2;
pub const TAKE: u64 = 3;
pub const DRAW: u64 = 4;
pub const FRESH: u64 = 5;
pub const SUSPEND: u64 = 6;
pub const RESUME: u64 = 7;
pub const QUERY_REFUSAL: u64 = 8;
pub const OVERSIZED_REFUSAL: u64 = 9;

pub const LIMITS: ic_wire::Limits = ic_wire::Limits {
    bytes: 4096,
    allocation: 65536,
    nodes: 4096,
    depth: 8,
};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub seeds: u32,
    pub remaining: u64,
    pub idle: bool,
    pub disabled: bool,
}
