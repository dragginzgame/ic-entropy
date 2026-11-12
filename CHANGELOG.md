# Changelog

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
