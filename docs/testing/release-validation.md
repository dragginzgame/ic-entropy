# Release validation and CI caches

Shared tooling owns execution, metadata transactions, evidence admission and
delivery recovery. Entropy's Make recipes and thin release adapter own the
active crate, retirement notice, feature modes and installed qualification obligations.

| Flow | Entrypoints and trigger | Guarantee and input scope |
| --- | --- | --- |
| Development | Focused package tests, Clippy and Wasm selections in `AGENTS.md`, after affected product edits | Product behavior and compilation for the changed package, features and callers. |
| Changed tooling | `make toko-tooling-check tools-check`; affected metadata, validation-dispatch, scanner and required-inventory fixtures | Installed registry bytes, prepared tools and actual consumer caller/failure contracts. Run fixtures when their wiring changes. |
| Repository CI | `make ci fmt-check`, ordered rule checks and one history scan, on pushes/PRs | Active-crate feature modes and the empty retirement notice, Wasm, docs and installed qualification on every declared host; one repository secret scan. |
| Release | `make release-patch`, `release-minor` or `release-major`, after explicit release authority | Fresh graph/pin/format admission, two-lock version preparation, the complete native stage on final-version source, commit/tag and Git delivery. |
| Publication | `make publish`, for an affected publishable release | Fresh final-source and Cargo archive/registry admission; the native stage reuses an exact-input proof or executes. |
| Recovery | `make release-resume VERSION=X.Y.Z`, for one saved failed release | Saved source, payload and destination admission; preserve failed evidence. Publication attempts separately require registry reconciliation before any retry. |

Consumer orchestration fixtures do not become unconditional release/upload gates.
The two-lock metadata fixture remains selected in repository CI; generic tooling
regressions stay upstream. A normal release starts fresh, while recovery selects
its saved attempt explicitly.

Active-crate tests, native/Wasm Clippy and Rustdoc cover all combinations of the
optional buffered, metrics and logging features. The metrics cases protect shared
store binding, cancellation/outcome attribution, latency-clock quality and
refill/consumption accounting; run affected modes when these paths or their
dependencies change.
Logging cases protect optional forwarding, native setup refusal, passive backend
refusals and refill transition callbacks outside model custody. The existing Make
leaves feed CI, native release validation and publication admission. Qualification
enables all three features and exercises host-owned metrics and shared IC Logging
storage on PocketIC. The incremental release-stage cost of these feature checks
is not yet measured; record it on the next ordinary delivery.

`make test-required-inventory` checks that unreadable directories fail both the
compilation-cache inventory and the release source observer before an incomplete
receipt or observation can be published. It also verifies readable cache
seal/restore and that readable source reaches dependency observation. Repository
CI selects this focused fixture; it has no release or publication effects.

The release adapter admits graph, pins and formatting before preparation. It
runs native lint, tests, Wasm and installed qualification from the committed
final-version source before tagging/pushing. Publication invokes the same stage;
Toko Tooling's exact-input runner either executes or admits the original proof
with a new admission record. Both Cargo locks and every source-file byte/mode
belong to the observation. Versions are never normalized or projected.

The consumer observer inventories local source (including new files), expanded
registry packages, compiler/sysroot, native headers/linker inputs, executable
bundles/receipts and the PocketIC server. Commands run with an explicit bounded
environment, after fetching missing locked dependencies with the caller's Cargo
settings. Explicit offline mode and other build/simulator overrides retain the
ordinary gate and their original environment. Unqualified hosts also retain the
ordinary gate; observation/custody failures refuse rather than silently skip.
Linux reuse is qualified locally. macOS always executes until separately
qualified. Source changes, missing proof, damaged logs and changed tool inputs
cannot authorize reuse. Fresh package/archive and registry checks remain with
the shared publication runner.

Evidence is under `target/release-validation/stages/attempts/`; each admission
names `executed` or `reused` and its original execution proof. Invalid candidates
are retained with the verifier's diagnostic. Missing/exact-input mismatches take
execution; the changed context inventories explain the mismatch. Interrupted or
failed executions produce no success proof. Do not edit or copy stage evidence
between checkouts or inject it into hosted build caches.

CI restores pinned tool downloads before bootstrap, then installs and admits
the pinned host tools so the shared
`prepare-rust-toolchain.sh --toolchain-file rust-toolchain.toml` entrypoint can
parse the compiler policy. That file owns the channel, profile, components and
Wasm target. Each host verifies the effective rustc/Cargo versions and reports
installed components and targets before dependency fetching. Qualify this setup
with the workflow's `Prepare selected compiler` step; Linux execution does not
establish execution on either macOS runner.

CI uses the shared `vendor/toko-tooling/.github/actions/prepare-common-tools`
action with custom Testkit/editor
recipes included in the cache key and ordered setup/check hooks. Restored Cargo
tools always pass their existing receipt and byte checks; missing or corrupt
generations retain the installer's rebuild/refusal behavior. Common tool cache
keys include compiler, native build environment, catalogs and installer inputs.
Compiled-tool setup and checks include Testkit's `prepare_server` example, while PocketIC acquisition
is a separate CI step. Successfully admitted compiled tools can therefore be
saved before a transient simulator download failure. Explicit simulator setup
uses curl's bounded transient retries with sanitized download diagnostics.
Checksum, preparation and cancellation failures retain their failure status;
the consumer explicitly downloads Testkit's selected asset, and Testkit owns
authentication and bounded decompression. CI
uploads redacted setup diagnostics and does not cancel other hosts on failure.

CI job budgets use four successful runs from 2026-11-13 through 2026-11-16,
including the [latest run](https://github.com/dragginzgame/ic-entropy/actions/runs/43965638971)
and the [oldest sampled run](https://github.com/dragginzgame/ic-entropy/actions/runs/43470720840).
Durations exclude runner queue time. The headroom accommodates cache misses and
runner/download variation; these limits bound hangs rather than speed up passing runs.

| Job/host | Longest observed | Job budget |
| --- | --- | --- |
| Core, Ubuntu 24.04 | 6m 51s | 20m |
| Core, macOS 15 ARM | 8m 58s | 30m |
| Core, macOS 15 Intel | 18m 19s | 45m |
| Secrets | 10s | 5m |

Tool preparation has an 8/12/20-minute step limit for Linux/macOS ARM/macOS
Intel respectively (observed maxima 3m 11s / 4m 15s / 10m 14s). PocketIC setup
has 5 minutes (observed maximum 45s); validation has 15 minutes (observed maximum
4m 8s). Shorter step limits leave time for the `always()` diagnostic uploads,
each bounded to 2 minutes. Validation output streams to the job log and a retained
artifact, with Bash failure and pipeline status preserved. Setup logs retain their
existing redaction. Uploads are best effort if the overall job limit or runner
termination is reached; they do not turn a failed or timed-out step into success.

The secrets job prepares its pinned Gitleaks scanner separately from scanning.
Setup retains only `status.txt` and the hidden `.attempt/install.log` under
`$RUNNER_TEMP/scanner-setup/`; the sanitized log is capped at 64 KiB, with
oversized lines and truncation identified. URLs, known credential fields,
sensitive environment values and private keys are redacted or omitted before
retention. Bare Bearer credentials and labelled payload lines are omitted; report
output neutralizes hosted workflow commands without altering retained log text.
No scanner findings, executable downloads or installer archives are
uploaded. Installation keeps its original exit status; collector failures are
recorded separately and also fail setup. The always-run artifact/report steps
include the selected hidden log, print a 4 KiB excerpt, link the 14-day artifact
and explicitly fail on missing evidence or upload failure. Setup has 3 minutes
and upload has 1 minute within the secrets job's existing 5-minute budget.
Run `bash scripts/ci/test-scanner-setup.sh` for offline caller failure checks;
Linux repository CI selects that fixture once after common tool setup, with a
one-minute budget. It substitutes installer and artifact effects and performs no
secret scan; these checks do not establish hosted artifact upload/retrieval.
The streaming collector retains completed diagnostic lines before setup exits.
Tooling's optional `SetupDiagnostics` helper returns its excerpt after completion,
so it does not yet replace this retained evidence during a stalled setup.

Compilation caches are limited to declared Cargo output directories, never
release/publication evidence. Keys include host/architecture, runner image,
compiler, source, manifests, locks and the tooling snapshot. A fallback restore
can seed outputs from different source or
locks on the same compiler/host image; Cargo rechecks their fingerprints and all
normal tests still run. This never restores stage evidence or authorizes reuse.
Saves happen only after successful default-branch validation, with a 4 GiB limit. Restores verify
the complete file/mode/digest inventory. Missing/corrupt or unsupported caches
discard only those disposable output directories and rebuild normally. The
cache is an acceleration input, never validation evidence.

Run `make test-validation-dispatch` for consumer dispatch, failure, caller Cargo
network settings, cache recovery and parallel Make ordering checks. CI selects
this consumer fixture; it runs substitutes rather than product compilation. The real Linux stage must also demonstrate execution,
reuse and invalidation through `scripts/release/native-stage.sh`.

Deployment latency runs from the authorized request through pushed GitHub refs
and registry availability. Use existing release/publication attempts to report
preparation, validation, packaging, Git delivery, upload and registry waiting.
Local source changes cannot establish deployment savings. The next authorized
release supplies the before/after comparison for [Entropy #1](https://github.com/dragginzgame/ic-entropy/issues/1); no benchmark release
or additional recurring gate is required. Hosted cache hit/miss and macOS times
remain separate, pending CI evidence.

## Target selection

The active crate, consumer example and qualification probe compile the same source on
native and Wasm. Runtime 0.23.23 supplies replication observation, update dispatch
and HTTP pricing on both targets. Outside replicated execution the adapter refuses
requests before claiming source custody. Timers 0.46.10 exposes the watchdog API
to native builds. Installed qualification exercises actual randomness, timer
dispatch and HTTP contention on the IC.

## Rule checks and ordered validation

`make check-rules` is the shared source/declaration admission for CI and release.
It uses the installed Toko Tooling checkers for pins with Cargo inheritance,
canonical IC/CBOR/hash dependency ownership in both workspaces, lexical Candid/CBOR
references in `crates` and `qualification/entropy-probe`, and cross-module type
headers. Private type layout, macro expansion and semantic ownership still need
review. The empty Candid allowance policy grants no external DTO exceptions.
These checks catch boundary drift before compilation and run when the selected
source, declarations or rule tooling change. On 2026-11-20 the complete leaf took
0.54 seconds on the local Linux host with prepared tools and locked metadata. This is local rule-check cost, not a
release or hosted-CI timing; revisit it when widening the selected roots.

Entropy ships libraries with no supported public canister Wire surface. The
qualification probe's RPCs are private test contracts. Public Wire baseline
comparison is therefore not selected; adding a supported public API would require
baseline review and export matching against its release Wasm.

`make validate-native` owns the existing product/native roster. It completes
native Clippy before tests, Wasm and docs, then qualification lint/docs before
building and running the installed probe. Separate recursive Make phases preserve
that order with parallel scheduling. CI adds rule checks first and consumer
fixtures afterward; release/publication use the same native target through their
existing exact-input stage. No producer regression matrix is added.

## Consolidated crate and retirement notice

From 0.6, `ic-entropy` owns acquisition, generation and pooling. The product
workspace also retains `ic-entropy-ic` solely to publish its empty retirement
notice. The latter has no normal dependencies, features or APIs. Native/Wasm
lint, compilation and Rustdoc check that artifact directly; it is not linked into
the installed probe. The probe qualifies the active crate from its local manifest
and version. The shared metadata transaction advances both product entries in
the root lock and only the active product entry in the qualification lock,
leaving the private probe version and unrelated suppliers unchanged.
