# Release validation and CI caches

The release adapter admits graph, pins and formatting before preparation. It
runs native lint, tests, Wasm and installed qualification from the committed
final-version source before tagging/pushing. Publication invokes the same stage;
Toko Tooling's exact-input runner either executes or admits the original proof
with a new admission record. Both Cargo locks and every source-file byte/mode
belong to the observation. Versions are never normalized or projected.

The consumer observer inventories local source (including new files), expanded
registry packages, compiler/sysroot, native headers/linker inputs, executable
bundles/receipts and the PocketIC server. Commands run with an explicit bounded
environment. Unknown build/simulator overrides and unqualified hosts retain the
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
retention. No scanner findings, executable downloads or installer archives are
uploaded. Installation keeps its original exit status; collector failures are
recorded separately and also fail setup. The always-run artifact/report steps
include the selected hidden log, print a 4 KiB excerpt, link the 14-day artifact
and explicitly fail on missing evidence or upload failure. Setup has 3 minutes
and upload has 1 minute within the secrets job's existing 5-minute budget.
Run `bash scripts/ci/test-scanner-setup.sh` for offline caller failure checks;
these do not establish hosted artifact upload/retrieval.

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
