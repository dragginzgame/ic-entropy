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

Setup uses the shared Rust-tool installer for IC Testkit 0.108.0's
`prepare_server` example with `managed-server`. The setup script explicitly
downloads Testkit's selected archive; Testkit authenticates the archive and
executable and performs bounded decompression. Prepared server bytes live under
`.tools/pocket-ic-managed/`; earlier installations remain intact. Tests verify
the prepared bytes without installing or downloading tools. Curl retries its
recognized transient failures twice. Redacted download and preparation diagnostics
live under `target/setup-pocket-ic/` and are uploaded by CI.
The driver explicitly owns the bounded server lifetime, attempts instance deletion,
then stops and joins the server, checking for abandoned owners and cleanup errors.
Build artifacts live under
`target/qualification/`. `make fmt` and `make fmt-check` cover both workspace roots.
The native driver uses Testkit's upstream PocketIC re-export, including its
Principal identities and async client. Explicit close awaits upstream best-effort
deletion; it does not independently prove remote deletion.
Typed Wire calls live in the separate `ic-testkit-wire` adapter.
This driver uses raw PocketIC calls and its own Wire codecs, so it needs no
additional adapter dependency.
The native test starts the server after its binary is built. The offline
`make qualification-tools-check` runs only prepared-server admission.

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
- Stop/drain, upgrade/start and reinstall reconstruct fresh volatile state and
  refill again.

The RPCs use bounded Wire codecs. IC `raw_rand` remains Platform's external
protocol. The canister selects Protocol 0.3.5 for bounded randomness/HTTP codecs
and Types 0.21.2 identities. Runtime 0.23.16 and Timers 0.46.3 share one exact
Runtime/guard selection in the locks. Testkit 0.108.0 supplies native server and
instance ownership; neither workspace retains the legacy Platform facade.
Native qualification and the setup helper use Host Artifacts/Fs 0.80.3 and
Host Process 0.80.4; these remain outside the product dependency graph.
Native model tests retain failure-budget, cancellation and stale-reply cases.
The installed contention fixture holds a simulator-controlled HTTP reply only
to keep the fresh request's IC invocation alive while its source claim is held.
Management `raw_rand` is real and is not mocked. The HTTP codec/dependencies are
private qualification inputs, not additional product dependencies. The fixture
checks contention and recovery; it does not claim full installed backoff/pause
coverage or arbitrary host cancellation behavior.
Arbitrary heap snapshot restore, real subnet deployment, consumer integration and
macOS execution are separate claims; a Linux PocketIC pass does not establish them.
