# Installed entropy qualification

This independent workspace contains one unpublished `entropy-probe` package:
its Wasm library is a standalone canister and its native integration test drives
PocketIC through IC Testkit. Neither side depends on Canic, Auth or IcyDB.
The product workspace excludes this tree. Its catalog, lockfile and private
version are independent; only the local product entries in its lock advance
when the product release adapter renders both locks in one shared transaction.

From the repository root:

```sh
cargo fetch --locked --manifest-path qualification/Cargo.toml
make install-qualification-tools
make qualification-check
make test-installed
```

Setup uses the shared Rust-tool installer for IC Testkit 0.96.0 and Testkit's
checksum-verified PocketIC setup. Tests admit the prepared selection without
installing or downloading tools. The command-scoped server belongs to Testkit;
the driver closes its own instance. Build artifacts live under
`target/qualification/`. `make fmt` and `make fmt-check` cover both workspace roots.

The journey verifies:

- Startup fills 256 seeds using only queries and clock/timer progress from the
  driver, proving acquisition is background work.
- A full pool stays idle; taking a seed resumes refill.
- Local expansion spends the configured byte budget; oversized draws refuse
  without mutation, and query draws refuse.
- A fresh request does not consume buffered seeds or generator budget.
- Suspension discards the pool/stream and stops refill; explicit reconstruction
  starts acquisition again.
- Upgrade and reinstall reconstruct fresh volatile state and refill again.

The RPCs use bounded Wire codecs. IC `raw_rand` remains Platform's external
protocol. Runtime 0.20.3 matches the product's approved Timers compatibility
exception; the native Testkit side independently uses current Types 0.19.
Native model tests retain failure-budget, cancellation and stale-reply cases.
Arbitrary heap snapshot restore, real subnet deployment, consumer integration and
macOS execution are separate claims; a Linux PocketIC pass does not establish them.
