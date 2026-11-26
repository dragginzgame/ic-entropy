# Changelog

## [0.7.0] - 2026-11-26

### ⚠️ Breaking

- `LogSink` now accepts `ic_logging::Level` from Logging 0.26. Hosts using the
  `logging` feature must update their sink and shared logger to Logging 0.26.
  [Logging 0.26 release notes](https://github.com/dragginzgame/ic-observability/blob/d295cef690a9046c050f09ad085d4a2d458ef692/releases/logging/CHANGELOG.md#0260---2026-11-26)

### 🔧 Tooling

- Adopt Metrics 0.30.23 and Platform Protocol 0.3.11 and retain the Timers
  0.46.17 lockfile refresh in both workspaces. Entropy, Timers and the installed
  probe share Platform Runtime and its guard at 0.23.27.

## [0.6.3] - 2026-11-26

### 🩹 Fixed

- Use Timers 0.46.16 so buffered refill can retry when the replica rejects a timer
  self-call before execution because the canister is low on cycles or stopped.
  [Timers 0.46.16 release notes](https://github.com/dragginzgame/ic-scheduling/blob/ba5e544107d24fe545906965cb5a147adfc18e27/releases/timers/CHANGELOG.md#04616---2026-11-26)

### 🔧 Tooling

- Adopt Toko Tooling 0.46.5. Use `make deps-list` for the product workspace and
  `make deps-list DEPENDENCY_MANIFEST=qualification/Cargo.toml` to include
  the qualification workspace.
- Update Logging to 0.25.21 and Metrics to 0.30.22. Align
  Platform and Wire declarations with the reviewed locks and preserve one Runtime executor.
- Update Testkit and its PocketIC preparer to 0.110.5 and the release metadata
  editor to 0.1.4.

### 🧪 Testing

- Check Cargo network settings in the release fixture on both Linux and macOS,
  where native validation follows different evidence-reuse rules.

## [0.6.2] - 2026-11-25

### 🔧 Tooling

- Refresh the locked Platform Runtime and Timers versions in both workspaces
  and update Wire in the installed qualification probe.

## [0.6.1] - 2026-11-24

### 🔧 Tooling

- Adopt Wire 0.30.0 in both sides of installed qualification and align Platform
  declarations with Runtime/Macros/Guard 0.23.25, Protocol 0.3.8 and Types
  0.21.4. Keep one Runtime executor and guard across Entropy, Timers and the probe.
- Adopt Testkit 0.110.4 for qualification and its prepared server helper, with
  locked Host Artifacts/Fs 0.80.8 and Process 0.81.4. Preserve the existing lock
  refreshes, including current Timers 0.46.13. Defer the optional metadata editor
  0.1.3 refresh to the next installer-maintenance batch.
- Adopt Toko Tooling 0.46.3 in both workspace catalogs and the authenticated
  snapshot, including corrected coordinated-release dependency admission.
- Capture compiler bootstrap and both locked dependency fetch failures through
  shared Toko Tooling setup diagnostics. Retain bounded, sanitized hidden logs
  and original statuses in the always-run 14-day setup artifact; report its link,
  missing evidence and upload failures explicitly.
- Exercise the actual workflow commands with forced setup/fetch failures and
  local artifact selection, including redaction, bounds and collector failures.

## [0.6.0] - 2026-11-20

### ⚠️ Breaking

- Consolidate IC acquisition, allocation-free `Generator` and `SeedPool`, optional
  buffering, metrics and logging into `ic-entropy` with Rust path `ic_entropy`.
  Platform Runtime and upstream ChaCha20 are normal dependencies; the package
  no longer promises `no_std` support.
- Publish `ic-entropy-ic` as an empty retirement notice linking to `ic-entropy`,
  with no APIs, features, forwarding dependency or compatibility exports. Remove
  the unpublished core package instead of publishing another crate name.
- Rename the fixed log source and refill watchdog prefix to `ic-entropy`; update
  host filters and imports using the README migration instructions.

### 🔧 Tooling

- Keep one active runtime qualification target and check the empty retirement
  package directly. Prepare the root and qualification locks together without
  adding the retirement notice to the canister dependency graph.
- Update package checks and release fixtures for the consolidated API and the
  empty retirement artifact.

## [0.5.10] - 2026-11-20

### ➕ Added

- Add independent opt-in `ic-entropy-ic/logging` using IC Logging 0.25.20.
  Forward fixed lifecycle, acquisition failure, scheduling and refill transition
  messages to a host-configured sink compatible with Canic's shared logger.
  Hosts own log storage and policy; no entropy contents enter messages.
- Add opt-in `ic-entropy-ic/metrics` using IC Metrics 0.30.21. Hosts can supply
  shared `Rc<MetricsStorage>` with the same binding pattern as Timers and Canic's
  typed metrics store; the adapter owns recording and hosts own reporting and
  incarnation identity. No Canic dependency or reporting endpoint is introduced.
- Observe explicit/background acquisition outcomes, cancellations, bounded
  completion latency, source refusals, buffered seed/byte consumption and refill
  retry/pause/recovery transitions. Preserve observations across reconstruction
  and refuse switching live stores.

### 🧪 Testing

- Cover instrumented and uninstrumented fresh/buffered modes and qualify shared
  storage, consumption, cancellation, stale replies and fresh upgrade/reinstall
  counters in the standalone PocketIC journey. Check shared canonical log storage,
  quiet successful draws, retained lifecycle history and fresh-heap logs.

### 🔧 Tooling

- Adopt authenticated Toko Tooling 0.46.0 in both workspaces and the installed
  snapshot. Enforce inherited dependency declarations, canonical IC dependency
  and Candid/CBOR source boundaries, and Rust type headers through shared checks.
- Restore pinned tool downloads before CI bootstrap, share ordered native
  validation between CI and release, and prove lint failure stops later work
  under parallel Make. Keep one repository secret scan and all three host jobs.
- Apply the current Rust type documentation and function layout rules; keep
  logging test helpers in test source and share refill recovery decisions.
- Registry review confirms current locked direct suppliers. Retain the separately
  prepared metadata editor 0.1.0 and Testkit preparer 0.110.0; defer optional
  0.1.2/0.110.3 CLI refreshes to the next installer-maintenance batch.
- Qualify the locked Timers 0.46.13 and qualification Wire 0.27.1 selections
  with the optional metrics integration.

## [0.5.9] - 2026-11-20

### 🔧 Tooling

- Adopt authenticated Toko Tooling 0.45.22 in both workspaces and the installed
  snapshot; audit required lint settings and inheritance in all five Cargo
  manifests.
- Adopt IC Wire 0.27.0 for the private qualification probe and native driver,
  retaining the existing framed schemas and request/reply limits.
- Include the reviewed lockfile updates to Timers 0.46.12 in both workspaces
  and qualification's native C compiler helper `cc` 1.11.3.
- Retain prepared metadata editor 0.1.0 and Testkit server preparer 0.110.0;
  revisit their optional 0.1.2/0.110.3 upgrades in the next installer-maintenance
  batch.

### 🧹 Cleanup

- Narrow internal buffered suspension and failure-budget visibility, separate
  the documented example's return value, and correct stale qualification and
  core installation documentation.

### 🩹 Fixed

- Refuse incomplete cache and release-source inventories when directory traversal
  reports an unreadable input. Exercise refusal and readable recovery through a
  focused regression fixture selected by repository CI.

## [0.5.8] - 2026-11-20

### 🧹 Cleanup

- Compile the buffered driver, replication check and contention probe on both
  targets using Platform Runtime 0.23.23 and Timers 0.46.10 or later. Remove the
  consumer target guards following [Platform #34 — Expose management observations and prepared call pricing to native type-checking](https://github.com/dragginzgame/ic-platform/issues/34).

### 🔧 Tooling

- Refresh qualification's locked Host Artifacts/Fs from 0.80.6 to 0.80.7.
- Retain Wire 0.26.1, Toko Tooling 0.45.21, prepared metadata editor 0.1.0 and
  Testkit server preparer 0.110.0. Defer optional 0.27.0/0.45.22/0.1.2/0.110.3
  refreshes to the next tooling-maintenance batch; other first-party locked
  suppliers remain current.

## [0.5.7] - 2026-11-19

### 🔧 Tooling

- Prepare CI's selected compiler through the shared Toko Tooling installer and
  `rust-toolchain.toml`, including rustfmt, Clippy and the Wasm target. Retain
  compiler/Cargo version checks and report installed components and targets on
  every host. Fix [Entropy #4 — Use the shared Rust toolchain installer in CI](https://github.com/dragginzgame/ic-entropy/issues/4).
- Include the reviewed lockfile refresh to Platform Runtime/guard/macros 0.23.23,
  Types 0.21.4 and Timers 0.46.11 across product and qualification; qualification
  also selects Wire 0.26.1, Testkit 0.110.3, Host Artifacts/Fs 0.80.6 and Host
  Process 0.81.3.
- Retain prepared metadata editor 0.1.0 and Testkit server preparer 0.110.0;
  defer optional 0.1.2/0.110.3 setup-tool refreshes to the next tooling-maintenance
  batch. Toko Tooling 0.45.21 and Protocol 0.3.7 remain current.

## [0.5.6] - 2026-11-19

### 🔧 Tooling

- Adopt IC Wire 0.26.0 for the qualification probe and native driver, retaining
  their bounded framed RPC schemas.
- Adopt Toko Tooling 0.45.21, Platform Runtime/macros/guard 0.23.22 and Protocol
  0.3.7 across product and qualification; preserve locked Timers 0.46.10.
- Keep fresh-request custody and the trap reporter unconditional, remove obsolete
  native lint expectations, and align endpoint guard/budget headers and return
  spacing with the current hygiene rules. Remaining native primitive exports
  are tracked in [Platform #34 — Expose management observations and prepared call pricing to native type-checking](https://github.com/dragginzgame/ic-platform/issues/34).
- Retain metadata editor 0.1.0 and Testkit server preparer 0.110.0; defer optional
  0.1.1/0.110.2 setup-tool refreshes to the next tooling-maintenance batch.

## [0.5.5] - 2026-11-19

### 🔧 Changed

- Use published Platform Runtime 0.23.20's bounded `raw_rand` operation in both
  workspaces, retaining the 300-second wait, source custody and lifecycle fencing.
  Remove the native transport stand-in and redundant production Protocol/Types
  dependencies. Fix [Entropy #3 — Adopt the Platform raw_rand primitive while preserving source custody](https://github.com/dragginzgame/ic-entropy/issues/3).

### 🔧 Tooling

- Adopt Toko Tooling 0.45.18 in both workspaces and the authenticated snapshot.
- Refresh locked Timers to 0.46.9 in both workspaces and qualification Wire to
  0.25.2.
- Group qualification endpoints at the library root with explicit access and
  bounds headers. Compile portable handlers on native hosts, narrowing Wasm
  conditions to the HTTP contention barrier and trap reporter.
- Retain metadata editor 0.1.0 and Testkit server preparer 0.110.0; defer their
  optional 0.1.1/0.110.2 setup refreshes to the next tooling-maintenance batch.

## [0.5.4] - 2026-11-19

### 🔧 Tooling

- Adopt Toko Tooling 0.45.16 in both workspaces and the authenticated snapshot.
- Refresh locked Timers to 0.46.8 across both workspaces, and qualification's Wire
  to 0.25.1, Testkit to 0.110.2, Host Artifacts/Fs to 0.80.5 and Host Process to 0.81.2.
- Retain the prepared metadata editor 0.1.0 and Testkit server preparer 0.110.0;
  defer their optional 0.1.1/0.110.2 refreshes to the next tooling-maintenance batch.
- Remove redundant target predicates from qualification dev-dependencies and the
  native unit-test harness. Document the IC API build failures that require the
  remaining transport, watchdog, canister-binding and example admission guards.

## [0.5.3] - 2026-11-19

### 🔧 Tooling

- Adopt Toko Tooling 0.45.13 in both workspaces and the authenticated snapshot.
- Refresh qualification's locked Testkit to 0.110.1 and Host Artifacts/Fs to
  0.80.4 and Host Process to 0.81.1.
- Retain the prepared metadata editor 0.1.0 and Testkit server preparer 0.110.0.
  Defer their optional 0.1.1/0.110.1 refreshes and Tooling 0.45.15 to the next
  tooling-maintenance batch; their reviewed changes do not block this release.

### 🩹 Fixed

- Omit bare Bearer credentials and labelled payloads from retained scanner setup
  diagnostics, redact API-key environment values, and neutralize workflow commands
  in report output. Exercise these refusals in Linux CI while retaining live logs.

### 🔧 Changed

- Keep portable refill transitions and cancellation cleanup compiled on native
  targets, with scoped expectations for their IC-only production callers.
- Refresh locked Protocol 0.3.6, Runtime 0.23.18, Types 0.21.3 and Timers 0.46.7
  across product and qualification, retaining one executor and matching guard.

## [0.5.2] - 2026-11-18

### 🔧 Tooling

- Retain bounded, sanitized scanner setup diagnostics and original failure status
  in CI, with a linked artifact even when installation fails before scanning.
- Update native qualification's TLS dependency lock to Zeroize 1.9.2.
- Adopt Testkit 0.110.0 for qualification and its setup helper, including its
  required Host Process 0.81.0 ownership implementation.
- Adopt Wire 0.25.0 for the qualification canister and native driver, retaining
  their framed RPC schemas and explicit codec limits.
- Adopt Toko Tooling 0.45.8 in both workspaces and the installed snapshot, with
  streamed input hashing and detailed phase timings for release validation.
- Preserve caller-selected Cargo network settings during metadata and validation;
  prepare locked inputs before sealing reusable validation evidence.
- Bound PocketIC download attempts and update the setup fixture for Testkit's
  current archive-preparation interface, preserving failure diagnostics and status.

### 🔧 Changed

- Refresh locked Runtime 0.23.17 and Timers 0.46.6 in the product and qualification
  workspaces, retaining one executor and its matching guard.
- Isolate Wasm transport and timer driving from the adapter's shared custody model,
  document public refusals and lifecycle panics, and classify qualification-only
  code and dependency roles for tooling.

## [0.5.1] - 2026-11-17

### 🔧 Changed

- Adopt Protocol 0.3.5 in the product and qualification workspaces, preserving
  bounded management randomness and HTTP framing. Types remains current at
  0.21.2.
- Adopt Timers 0.46.3 and Runtime 0.23.16 in both workspace graphs, keeping one
  executor and its matching macros/guard cohort.

### 🔧 Tooling

- Use standalone `ic-cargo-metadata` 0.1.0 for release manifest and lock rendering,
  replacing the legacy Build metadata editor.
- Bound CI jobs with measured host-specific timeout budgets and retain streamed
  validation diagnostics alongside the existing setup logs.
- Adopt Testkit 0.108.0 for native qualification and its setup helper, with Host
  Artifacts/Fs 0.80.3 and Host Process 0.80.4 supplying asset and process ownership.

## [0.5.0] - 2026-11-16

### ⚠️ Breaking

- Adopt Timers 0.46.0 and Runtime 0.23.14 with one executor. Hosts must align
  direct Runtime/Timers dependencies and stop/drain before upgrading. Entropy
  acquisition, bounded buffering and consumer-owned stream APIs are unchanged.

### 🔧 Changed

- Move management randomness framing to Protocol 0.2.7 and Principal identities
  to Types 0.21.2. Remove the legacy Platform facade from both workspaces.
- Migrate installed qualification to Testkit 0.107.3's explicit server ownership
  and authenticated PocketIC preparation with Wire 0.24.3, retaining real
  source-custody and lifecycle checks.

### 🔧 Tooling

- Adopt Toko Tooling 0.44.11 in both workspaces, correcting dependency-pin
  admission for the exact tooling selection and private qualification version.
- Retain the Build 0.73.0 release editor; defer its optional 0.87.3 refresh to
  the next dependency-maintenance batch.

## [0.4.0] - 2026-11-15

### ⚠️ Breaking

- Adopt Timers 0.45.9 and its retained watchdog capabilities, with the required
  Runtime 0.22.5 executor. Hosts must align direct Timers dependencies to 0.45;
  entropy APIs, bounded refill admission and lifecycle ownership are unchanged.

### 🔧 Changed

- Migrate the independent qualification probe and driver to Wire 0.24.1's
  root codec APIs with explicit request/response framing and existing limits.
- Adopt Platform 0.33.0 in the IC adapter and qualification probe, with Runtime
  0.22.5 and Types 0.20.3. Preserve the raw management-randomness contract and one
  executor across acquisition, timers and the probe.

### 🔧 Tooling

- Adopt Toko Tooling 0.44.4, including corrected LOC depth reporting and stricter
  release timing admission. Keep qualification lock planning and the
  existing direct release workflow and two-lock metadata transaction.
- Adopt IC Testkit 0.106.0, its matching server CLI and required Host 0.79
  dependencies. Retain fresh-instance qualification and raw PocketIC calls;
  the retired pooling and diagnostic APIs require no driver migration.

## [0.3.1] - 2026-11-14

### 🔧 Tooling

- Adopt Toko Tooling 0.43.0, including CDPATH-independent hooks and the `toko-cloc`
  PATH launcher replacing `make cloc`.
- Adopt IC Testkit 0.104.0 and its matching server CLI for installed qualification;
  retain raw PocketIC calls and the probe's independently selected Wire codecs.

### 🔧 Changed

- Refresh locked Platform to 0.32.2, Runtime/macros/guard to 0.22.2, Types to
  0.20.2 and Timers to 0.44.1; retain one executor across acquisition, timers and
  the qualification probe. Qualification also selects Wire 0.22.1 and Host 0.77.2.
- Retain the qualified Build 0.73.0 release editor; defer the optional 0.84.1
  refresh to the next dependency-maintenance batch.

## [0.3.0] - 2026-11-13

### ⚠️ Breaking

- Adopt Timers 0.44.0 and Runtime 0.22.1 with its exact matching executor guard.
  Canister hosts must align direct Runtime/Timers dependencies to these generations;
  align direct Platform Types to 0.20 when exchanging Runtime Principal values.
  Entropy seed and draw APIs remain unchanged.

### 🔧 Changed

- Adopt Toko Tooling 0.42.5 and its offline `make deps-list` inventory, including
  release bootstrap support without display-only dependency aliases.
- Defer optional Runtime 0.22.2, Types 0.20.2, Wire 0.22.1 and Build 0.84.1
  refreshes to the next dependency-maintenance batch; this release retains the
  selected compatible runtime and qualification graphs and Build 0.73.0 editor.
- Isolate installed tooling under `vendor/toko-tooling/`; update Make, CI, hooks
  and release callers while keeping repository-owned scripts outside that tree.
- Adopt Platform 0.32.1, qualification Wire 0.22.0 and Testkit 0.102.0, including
  its matching server setup and Host 0.77 dependencies. Align qualification Types
  to 0.20.1 and retain one Runtime executor across acquisition, timers and the probe.
- Enforce common Cargo dependency/metadata lints and Rustdoc table checks in
  product and qualification workspaces.
- Enforce Clippy warnings through Cargo's warning mode on fresh and reused
  artifacts, retaining explicit strict Rustdoc checks.
- Remove unused product and qualification dependency aliases to keep Cargo and
  Clippy output clear while retaining both product packages in release validation.
- Select the adapter's Runtime dependency for Wasm and buffered builds; native
  fresh-only builds no longer carry an unused IC execution dependency.
- Use Rust 1.100's standard cleanup guard for cancelled background refills,
  preserving stale-generation refusal and single failure accounting without a
  custom destructor or mutable completion flag.

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
