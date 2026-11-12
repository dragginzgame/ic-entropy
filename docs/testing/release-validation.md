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

CI uses the shared `prepare-common-tools` action with custom Testkit/editor
recipes included in the cache key and ordered setup/check hooks. Restored Cargo
tools always pass their existing receipt and byte checks; missing or corrupt
generations retain the installer's rebuild/refusal behavior. Common tool cache
keys include compiler, native build environment, catalogs and installer inputs.

Compilation caches are limited to declared Cargo output directories, never
release/publication evidence. Keys include host/architecture, runner image,
compiler, source, manifests, locks and the tooling snapshot. Saves happen only
after successful default-branch validation, with a 4 GiB limit. Restores verify
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
release supplies the before/after comparison for issue #1; no benchmark release
or additional recurring gate is required. Hosted cache hit/miss and macOS times
remain separate, pending CI evidence.
