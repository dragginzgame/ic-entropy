# Installed entropy qualification

This independent workspace contains one unpublished `entropy-probe` package:
its Wasm library is a standalone canister and its native integration test drives
PocketIC through IC Testkit. Neither side depends on Canic, Auth or IcyDB.
The product workspace excludes this tree. Its catalog, lockfile and private
version are independent; only the local product entries in its lock advance
when the product release adapter renders both locks in one shared transaction.

Deployments qualify the selected locked product and qualification graphs.
Testkit's native Platform and Types generations may differ from the canister's
when their type, protocol and executor boundaries remain valid. Updating product
dependencies does not require an unrelated Testkit upgrade. A failed optional
supplier-upgrade experiment does not invalidate the existing release selection;
required qualification and installed lifecycle checks still run. Prepare any
required migration before freezing release inputs, then reuse admitted validation
at publication while freshly checking the final archives and registry state.

From the repository root:

```sh
cargo fetch --locked --manifest-path qualification/Cargo.toml
make install-qualification-tools
make qualification-check
make test-installed
```

Setup uses the shared Rust-tool installer for IC Testkit 0.104.0 and Testkit's
checksum-verified PocketIC setup. Tests admit the prepared selection without
installing or downloading tools. The command-scoped server belongs to Testkit;
compiled CLI setup is cached separately from simulator acquisition. Explicit
PocketIC setup retries recognized transient curl failures at most three times;
other failures stop immediately. Redacted download diagnostics live under
`target/setup-pocket-ic/` and are uploaded by CI.
The driver closes its own instance. Build artifacts live under
`target/qualification/`. `make fmt` and `make fmt-check` cover both workspace roots.
The native driver uses Testkit's upstream PocketIC re-export, including its
Principal identities and async client. Explicit close awaits upstream best-effort
deletion; it does not acknowledge deletion on a borrowed server.
Testkit 0.104 keeps typed Wire calls in the separate `ic-testkit-wire` adapter.
This driver uses raw PocketIC calls and its own Wire codecs, so it needs no
additional adapter dependency.
The native test binary is built before starting PocketIC, so a cold build cannot
consume the server's idle timeout before the first request.

The journey verifies:

- Startup fills 256 seeds using only queries and clock/timer progress from the
  driver, proving acquisition is background work.
- A full pool stays idle; taking a seed resumes refill.
- Local expansion spends the configured byte budget; oversized draws refuse
  without mutation, and query draws refuse.
- A fresh request does not consume buffered seeds or generator budget.
- Dropping a pending fresh request releases source custody for another request.
- A reply from before suspension is refused as stale and cannot release a
  replacement request's claim or publish seeds into the replacement lifecycle.
- A held fresh request makes the real refill timer report `Busy`; releasing it
  allows automatic refill recovery and clears the failure count.
- Suspension discards the pool/stream and stops refill; explicit reconstruction
  starts acquisition again.
- Upgrade and reinstall reconstruct fresh volatile state and refill again.

The RPCs use bounded Wire codecs. IC `raw_rand` remains Platform's external
protocol. The locks select Platform 0.32.2 for the codec and its Types 0.20 identities;
Runtime 0.22.2 and the probe's direct Types 0.20.2 match Timers 0.44.1's
executor selection. The native Testkit side independently uses Types 0.20.
Native model tests retain failure-budget, cancellation and stale-reply cases.
The installed contention fixture holds a simulator-controlled HTTP reply only
to keep the fresh request's IC invocation alive while its source claim is held.
Management `raw_rand` is real and is not mocked. The HTTP codec/dependencies are
private qualification inputs, not additional product dependencies. The fixture
checks contention and recovery; it does not claim full installed backoff/pause
coverage or arbitrary host cancellation behavior.
Arbitrary heap snapshot restore, real subnet deployment, consumer integration and
macOS execution are separate claims; a Linux PocketIC pass does not establish them.
