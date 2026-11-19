# Release validation and CI caches

Shared tooling owns execution, metadata transactions, evidence admission and
delivery recovery. Entropy's Make recipes and thin release adapter own the
product packages, feature modes and installed qualification obligations.

| Flow | Entrypoints and trigger | Guarantee and input scope |
| --- | --- | --- |
| Development | Focused package tests, Clippy and Wasm selections in `AGENTS.md`, after affected product edits | Product behavior and compilation for the changed package, features and callers. |
| Changed tooling | `make toko-tooling-check tools-check`; affected metadata, validation-dispatch and scanner fixtures | Installed registry bytes, prepared tools and actual consumer caller/failure contracts. Run fixtures when their wiring changes. |
| Repository CI | `make ci fmt-check`, workflow pin checks and one history scan, on pushes/PRs | Both product packages and adapter modes, Wasm, docs and installed qualification on every declared host; one repository secret scan. |
| Release | `make release-patch`, `release-minor` or `release-major`, after explicit release authority | Fresh graph/pin/format admission, two-lock version preparation, the complete native stage on final-version source, commit/tag and Git delivery. |
| Publication | `make publish`, for an affected publishable release | Fresh final-source and Cargo archive/registry admission; the native stage reuses an exact-input proof or executes. |
| Recovery | `make release-resume VERSION=X.Y.Z`, for one saved failed release | Saved source, payload and destination admission; preserve failed evidence. Publication attempts separately require registry reconciliation before any retry. |

Consumer orchestration fixtures do not become unconditional release/upload gates.
The two-lock metadata fixture remains selected in repository CI; generic tooling
regressions stay upstream. A normal release starts fresh, while recovery selects
its saved attempt explicitly.

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

Run `bash scripts/release/test-validation.sh` for consumer dispatch, failure and
cache recovery checks. The real Linux stage must also demonstrate execution,
reuse and invalidation through `scripts/release/native-stage.sh`.

Deployment latency runs from the authorized request through pushed GitHub refs
and registry availability. Use existing release/publication attempts to report
preparation, validation, packaging, Git delivery, upload and registry waiting.
Local source changes cannot establish deployment savings. The next authorized
release supplies the before/after comparison for [Entropy #1](https://github.com/dragginzgame/ic-entropy/issues/1); no benchmark release
or additional recurring gate is required. Hosted cache hit/miss and macOS times
remain separate, pending CI evidence.

## Target selection

The adapter supports native refusal/model tests, and the qualification driver
imports the probe's shared Wire types on native hosts. With Runtime 0.23.20 and
Timers 0.46.9, removing the following IC boundaries fails those supported builds,
even after selecting the same dependency features on native:

| Boundary | Concrete native failure |
| --- | --- |
| Adapter source operation and replication admission | Runtime omits `raw_rand::request` and `raw`. |
| Buffered IC watchdog | Timers omits `Watchdog`, `initialize` and `bind_watchdog`; Runtime omits `system` and `executor`. |
| Qualification HTTP contention barrier | Runtime omits `raw` and `Call::execute` from the native client library. |
| Qualification trap reporter | Runtime omits `system::trap` from the native client library. |
| Consumer-owned example's admission check | Runtime omits `raw`; the native example must refuse before accessing streams. |

The corresponding native fallbacks preserve explicit refusal. Shared custody,
refill transitions and cancellation cleanup compile without target selection.
The qualification endpoint group lives at the top of its library root. Its
portable handler bodies compile for the native Wire client too; Runtime's macros
select only their generated IC exports for Wasm. Only the HTTP barrier and trap
reporter retain target conditions, at the functions calling the missing APIs.
The native unit-test harness needs no extra target predicate. Qualification's
Testkit/Tokio declarations use ordinary dev-dependencies: the shipped Wasm library
does not compile those dependencies, and its strict Wasm check passes without a
Cargo target predicate.

Review these boundaries with the owning native adapter tests, example Clippy,
qualification all-target Clippy and shipped Wasm library check. The retained
guards protect missing IC APIs and supported callers; warnings on an unsupported
host are not the reason for them.
