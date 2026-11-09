# IC Entropy agent instructions

Apply the reviewed Toko Tooling baseline in `TOKO-TOOLING.md`.

The core is `no_std`, allocation-free and independent of Canic, IcyDB, Platform,
OS entropy and timers. The separate `ic-entropy-ic` adapter owns IC acquisition
and optional Timers-driven refill; hosts own lifecycle composition and consumer
admission. Neither package may depend on Canic, IcyDB or Auth.
Retain upstream cryptographic implementations; do not introduce a custom hash
chain, entropy mixer or primitive. Preserve the IcyDB source license and provenance.

Focused checks: `cargo test --locked -p ic-entropy`,
`cargo clippy --locked -p ic-entropy --all-targets -- -D warnings`, and
`cargo check --locked -p ic-entropy --target wasm32-unknown-unknown`.
Linux checks do not establish macOS execution. Keep tests in separate modules.
For adapter changes, select `-p ic-entropy-ic` with both default fresh-only and
`--features buffered` for tests, strict all-target Clippy and Wasm compilation.
The Make test/lint/check-wasm targets aggregate both packages and adapter modes.
Installed IC qualification lives in the independent `qualification/` workspace.
Use `make install-qualification-tools`, `make qualification-check`, and
`make test-installed` for its standalone PocketIC lifecycle journey. Its private
probe version does not follow product releases; prepare both lockfiles together
through the consumer adapter and shared metadata transaction. Format both roots.
Initial workspace creation and the IcyDB extraction were approved by the
maintainer on 2026-11-09; this grants no publication or sibling release authority.

## Tooling selection

Select Toko Tooling 0.38.0 exactly in the root catalog and inherit it as the
package's development dependency. The installed receipt binds registry bytes.
The qualification catalog selects the same exact tooling version for its probe.
Do not edit snapshot-owned files locally. Use `make install-host-tools
install-rust-tools install-hooks` for setup. Release metadata uses the reviewed
IC Build 0.73.0 editor; its setup target owns installation. Standard release
commands use the shared runner and this workspace's thin metadata adapter.
A release includes native tests/lint, formatting, Wasm compilation, pin checks
and source/lock admission. Hosted macOS qualification runs in CI.

## Runtime compatibility exception — 2026-11-09

The maintainer approved Runtime 0.20.3 for `ic-entropy-ic` while published
IC Timers 0.41.5 requires Runtime 0.20. Runtime 0.21.0 conflicts with Timers'
executor guard; Cargo rejects both execution owners in one graph. Platform
0.28.0 is selected independently for its runtime-free `raw-rand` codec.
Revisit this exception when a published Timers release supports Runtime 0.21
or newer; preserve one runtime/executor in each deployed graph.
The standalone Wasm qualification probe uses that same approved Runtime selection.
