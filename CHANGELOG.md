# Changelog

## [0.1.1] - Unreleased

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
