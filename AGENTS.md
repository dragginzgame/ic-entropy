# IC Entropy agent instructions

Apply the installed [Toko Tooling baseline](TOKO-TOOLING.md), recorded in
[the snapshot](.toko-tooling.snapshot), for common engineering and action-authority
rules. Keep this overlay focused on local contracts, commands and exceptions.

`ic-entropy` owns IC acquisition and allocation-free generator/pool operations in
one crate. Platform Runtime is a normal dependency; Timers-driven refill, metrics
and logging are optional. Hosts own lifecycle composition and consumer admission.
It may not depend on Canic, IcyDB or Auth. Retain upstream cryptographic
implementations; do not introduce a custom hash chain, entropy mixer or primitive.
Preserve applicable external dependency notices.

`ic-entropy-ic` is an empty retirement notice linking to `ic-entropy`. It exposes
no APIs, features or normal dependencies. Do not add compatibility forwarding.
The unpublished core package has been removed; generator and pool code belongs
in the main crate's modules.

Focused checks select `-p ic-entropy`, with default features and all combinations
of `buffered`, `metrics` and `logging`, for tests, strict all-target Clippy and Wasm
compilation. Use `CARGO_BUILD_WARNINGS=deny` for Clippy. Keep test bodies/helpers
in separate test modules. The Make test/lint/check-wasm targets aggregate these
modes and directly check the empty retirement package. Linux checks do not
establish macOS execution.

The host can supply shared `Rc<MetricsStorage>` before acquisition and owns
reporting and incarnation identity. Logging forwards fixed events through a host
sink into shared IC Logging storage. Neither adds a Canic dependency or endpoint.
`make check-rules` checks dependency declarations and Rust source boundaries in
both roots. `make validate-native` aggregates lint, tests, Wasm, docs and installed
qualification in order; CI and release adapters share this owner.
Installed IC qualification lives in the independent `qualification/` workspace.
Use `make install-qualification-tools`, `make qualification-check`, and
`make test-installed` for its standalone PocketIC lifecycle journey. It consumes
only the active crate; the empty retirement package has no installed-runtime role.
Its private probe version does not follow product releases; prepare both lockfiles
together through the consumer adapter and shared metadata transaction. Format
both roots.

## Tooling selection

The root and qualification catalogs select the same exact Toko Tooling release;
the receipt binds installed registry bytes. Use `make install-host-tools
install-rust-tools install-hooks` for setup. The release editor's selected version
belongs to its setup owner. Standard release commands use the shared runner and
this workspace's thin metadata adapter.
A release includes native tests/lint, formatting, Wasm compilation, pin checks
and source/lock admission. Hosted macOS qualification runs in CI.

## Runtime selection

Entropy, Timers and the standalone Wasm qualification probe must select a
compatible Runtime generation with one executor guard. Their manifests and locks
own exact supplier selections.
