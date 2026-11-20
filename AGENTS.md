# IC Entropy agent instructions

Apply the installed [Toko Tooling baseline](TOKO-TOOLING.md), recorded in
[the snapshot](.toko-tooling.snapshot), for common engineering and action-authority
rules. Keep this overlay focused on local contracts, commands and exceptions.

The core is `no_std`, allocation-free and independent of Canic, IcyDB, Platform,
OS entropy and timers. The separate `ic-entropy-ic` adapter owns IC acquisition
and optional Timers-driven refill; hosts own lifecycle composition and consumer
admission. Neither package may depend on Canic, IcyDB or Auth.
Retain upstream cryptographic implementations; do not introduce a custom hash
chain, entropy mixer or primitive. Preserve applicable external dependency notices.

Focused checks: `cargo test --locked -p ic-entropy`,
`CARGO_BUILD_WARNINGS=deny cargo clippy --locked -p ic-entropy --all-targets`, and
`cargo check --locked -p ic-entropy --target wasm32-unknown-unknown`.
Linux checks do not establish macOS execution. Keep tests in separate modules.
For adapter changes, select `-p ic-entropy-ic` with default fresh-only and
all combinations of `buffered`, `metrics` and `logging` for tests, strict
all-target Clippy and Wasm compilation. Instrumentation stays opt-in;
the host can supply shared `Rc<MetricsStorage>` before acquisition, and owns
reporting and incarnation identity. Logging forwards fixed events through a host
sink into shared IC Logging storage. Neither adds a Canic dependency or endpoint.
The Make test/lint/check-wasm targets aggregate both packages and adapter modes.
`make check-rules` checks dependency declarations and Rust source boundaries in
both roots. `make validate-native` aggregates lint, tests, Wasm, docs and installed
qualification in order; CI and release adapters share this owner.
Installed IC qualification lives in the independent `qualification/` workspace.
Use `make install-qualification-tools`, `make qualification-check`, and
`make test-installed` for its standalone PocketIC lifecycle journey. Its private
probe version does not follow product releases; prepare both lockfiles together
through the consumer adapter and shared metadata transaction. Format both roots.

## Tooling selection

The root and qualification catalogs select the same exact Toko Tooling release;
the receipt binds installed registry bytes. Use `make install-host-tools
install-rust-tools install-hooks` for setup. The release editor's selected version
belongs to its setup owner. Standard release commands use the shared runner and
this workspace's thin metadata adapter.
A release includes native tests/lint, formatting, Wasm compilation, pin checks
and source/lock admission. Hosted macOS qualification runs in CI.

## Runtime selection

The IC adapter, Timers and standalone Wasm qualification probe must select a
compatible Runtime generation with one executor guard. Their manifests and locks
own exact supplier selections.
