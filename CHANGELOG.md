# Changelog

## [0.2.0] - 2026-11-13

### ⚠️ Breaking

- Consumer builds now require Rust 1.100.0; upgrade the compiler before adopting
  this release ([#2 — Adopt Rust 1.100.0 and the common lint policy](https://github.com/dragginzgame/ic-entropy/issues/2)).

### 🔧 Changed

- Adopt Toko Tooling 0.40.1 and its common lint policy in both workspaces.
  Validate native and Wasm Rustdoc with strict warnings; retain IC-local futures
  without a universal `Send` requirement.
- Adopt Platform 0.32.0 for entropy acquisition and the qualification canister.
  Keep Runtime 0.21.3 and the probe's direct Types 0.19.5 paired with published
  Timers 0.43.4; revisit Runtime 0.22.0 and direct Types 0.20.0 with the next
  compatible Timers/Runtime adoption. Platform's codec-owned Types 0.20.0 stays
  behind its serialization boundary.
- Supplier review: Testkit 0.101.0, Wire 0.21.5, ChaCha 0.10.0, Serde 1.0.229
  and Tokio 1.54.1 are current. Defer the optional Build 0.73.0 to 0.83.0
  release-editor migration to the next release-tooling maintenance batch.

## [0.1.12] - 2026-11-12

### 🐛 Fixed

- Save admitted compiled tools before PocketIC acquisition; retry recognized
  transient simulator download failures with a bounded policy, retain redacted
  setup diagnostics, and let all CI hosts finish independently
  ([#1](https://github.com/dragginzgame/ic-entropy/issues/1)).

## [0.1.11] - 2026-11-12

### 🔧 Changed

- Adopt Toko Tooling 0.39.18 in both workspaces.
- Validate committed final-version native inputs once and admit exact-input
  evidence again at publication; retain fresh source, archive and registry gates.
  Add admitted compiled-tool and bounded compilation caches in CI
  ([#1](https://github.com/dragginzgame/ic-entropy/issues/1)).
- Refresh locked Metrics 0.29.12, Timers 0.43.4 and qualification Wire 0.21.5.

## [0.1.10] - 2026-11-12

### 🔧 Changed

- Adopt Wire 0.21.4, Platform 0.31.3, Runtime 0.21.3 and Types 0.19.5;
  qualify Host Artifacts, FS and Process 0.76.5 through Testkit.
- Adopt Toko Tooling 0.39.17 and migrate generator, pool and qualification
  canister modules to its directory-module layout.
- Supplier review: Testkit 0.101.0 remains current. Defer the optional IC Build
  0.83.0 release-editor migration to the next release-tooling maintenance batch.

## [0.1.9] - 2026-11-12

### 🔧 Changed

- Adopt Toko Tooling 0.39.15 in both workspaces, including bounded LOC report
  labels and the shared `make deps-update` command for both workspaces.
- Adopt IC Testkit 0.101.0 in the qualification workspace and server setup;
  migrate the installed lifecycle test to its upstream asynchronous PocketIC client.
- Adopt qualification Wire 0.21.2, Platform 0.31.2, Runtime 0.21.2 and
  Types 0.19.4; confirm Metrics 0.29.11 through Timers.
- Refresh the product and qualification locks to Timers 0.43.2 and the current
  compatible Platform runtime stack.
- Supplier review: Serde 1.0.229 and Tokio 1.54.1 are current. Defer the
  IC Build 0.83.0 release-editor migration to the next release-tooling
  maintenance batch.

## [0.1.8] - 2026-11-11

### 🔧 Changed

- Adopt Toko Tooling 0.39.10 in both workspaces.
- Adopt IC Platform 0.31.0 and Types 0.19.3 in the adapter and installed
  qualification workspace.
- Refresh the product lockfile to IC Timers 0.43.1.

## [0.1.7] - 2026-11-11

### 🔧 Changed

- Adopt Toko Tooling 0.39.9 in both workspaces.
- Adopt IC Timers 0.43.0 and Platform 0.30.0, with Runtime 0.21.1 and
  Types 0.19.2 shared by the adapter and installed qualification probe.

## [0.1.6] - 2026-11-10

### 🔧 Changed

- Adopt Toko Tooling 0.39.8 in both workspaces.

## [0.1.5] - 2026-11-10

### 🔧 Changed

- Adopt Toko Tooling 0.39.6 in both workspaces.

## [0.1.4] - 2026-11-10

### 🔧 Changed

- Adopt Toko Tooling 0.39.2 in both workspaces and explicitly enable its shared
  release commands.

## [0.1.3] - 2026-11-10

### ➕ Added

- Add a compile-checked consumer-owned generator example with separate budgets,
  small draws, failure-preserving reseeding, query refusal and lifecycle disposal.
- Extend standalone installed qualification with real fresh-request cancellation,
  stale callback fencing, replacement-claim custody and timer refill contention
  followed by recovery, without a downstream framework.

## [0.1.2] - 2026-11-10

### ➕ Added

- Qualify independent consumer streams drawn from one seed pool, including
  different expansion budgets, exhaustion and reseeding without affecting peers.

### 🔧 Changed

- Adopt Toko Tooling 0.38.2, IC Timers 0.42.0 and Platform 0.29.2 with
  Runtime 0.21.0 in the product and qualification graphs, retiring the older
  Runtime compatibility exception.

## [0.1.1] - 2026-11-09

### 🔧 Changed

- Update development tooling to Toko Tooling 0.38.0.
- Set the IC adapter's fixed pool capacity to 256 seeds (8 KiB), retaining
  timer-driven replenishment. Document consumer-owned generators with explicit
  per-consumer expansion budgets; the shared convenience stream retains 64 KiB.

### ➕ Added

- Add an independent IC entropy adapter with fresh management requests and
  opt-in timer-driven background refills, bounded retries and volatile lifecycle
  reconstruction. Canic, IcyDB and Auth remain downstream consumers.
- Add standalone PocketIC qualification for background acquisition, refill after
  consumption, query refusal, suspension and upgrade/reinstall reconstruction.

## [0.1.0] - 2026-11-09

### ➕ Added

- Bounded seed buffering and caller-owned ChaCha20 expansion, extracted from
  IcyDB's generator with its existing u128 stream ordering preserved.
- Explicit draw budgets, refusal without stream advancement, and reseeding.
  The core requires no Canic, IC runtime, operating system or allocator.
