# IC Entropy agent instructions

Apply the reviewed Toko Tooling baseline in `TOKO-TOOLING.md`.

The core is `no_std`, allocation-free and independent of Canic, IcyDB, Platform,
OS entropy and timers. Hosts own entropy acquisition, scheduling and lifecycle.
Retain upstream cryptographic implementations; do not introduce a custom hash
chain, entropy mixer or primitive. Preserve the IcyDB source license and provenance.

Focused checks: `cargo test --locked -p ic-entropy`,
`cargo clippy --locked -p ic-entropy --all-targets -- -D warnings`, and
`cargo check --locked -p ic-entropy --target wasm32-unknown-unknown`.
Linux checks do not establish macOS execution. Keep tests in separate modules.
Initial workspace creation and the IcyDB extraction were approved by the
maintainer on 2026-11-09; this grants no publication or sibling release authority.

## Tooling selection

Select Toko Tooling 0.37.4 exactly in the root catalog and inherit it as the
package's development dependency. The installed receipt binds registry bytes.
Do not edit snapshot-owned files locally. Use `make install-host-tools
install-rust-tools install-hooks` for setup. Release metadata uses the reviewed
IC Build 0.73.0 editor; its setup target owns installation. Standard release
commands use the shared runner and this workspace's thin metadata adapter.
A release includes native tests/lint, formatting, Wasm compilation, pin checks
and source/lock admission. Hosted macOS qualification runs in CI.
