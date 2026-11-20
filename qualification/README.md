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

Setup uses the shared Rust-tool installer for IC Testkit 0.110.0's
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
This driver uses raw PocketIC calls and its own Wire 0.27 codecs, so it needs no
additional adapter dependency.
The native test starts the server after its binary is built.
`make qualification-tools-check` admits the prepared server without downloading
or installing it. Cargo may retrieve missing locked dependencies using the caller's
settings; offline mode is an explicit caller choice.

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
- The host-supplied metrics store matches the adapter's observations, preserves
  counters across repeated binding and reconstruction, and refuses replacement.
  Fresh/background requests, cancellation, stale replies, consumption/refusals
  and real latency clocks are observed; upgrade/reinstall start fresh counters.

- The host's canonical IC Logging store contains both host and Entropy events.
  Routine draws/fresh successes add no rows; stale completion and refill recovery
  emit fixed messages. Repeated sink setup preserves history, and upgrade/reinstall
  create fresh logs. The sink reads adapter status without a custody conflict.

The endpoint group is at the top of `entropy-probe/src/lib.rs`, with its explicit
guard and request/reply bounds recorded on each handler. Portable bodies and the
trap reporter also compile for the native Wire client. The HTTP contention
barrier uses Runtime 0.23.23's pricing API on both targets. Native compilation
does not supply an IC pricing environment; the installed PocketIC journey
executes the real barrier call and verifies contention and recovery.
The RPCs use bounded Wire codecs. IC `raw_rand` remains Platform's external
protocol. The locked Protocol and Types supply bounded randomness/HTTP codecs
and identities. The Runtime and Timers graph retains one executor and its
matching guard. The locked Testkit supplies native server and instance ownership;
neither workspace retains the legacy Platform facade. Native qualification's
Host suppliers are bound by `Cargo.lock`; the setup helper's own supplier graph
is bound by its build receipt. These remain outside the product dependency graph.
Native model tests retain failure-budget, cancellation and stale-reply cases.
The installed contention fixture holds a simulator-controlled HTTP reply only
to keep the fresh request's IC invocation alive while its source claim is held.
Management `raw_rand` is real and is not mocked. The HTTP codec/dependencies are
private qualification inputs, not additional product dependencies. The fixture
checks contention and recovery; it does not claim full installed backoff/pause
coverage or arbitrary host cancellation behavior.
Arbitrary heap snapshot restore, real subnet deployment, consumer integration and
macOS execution are separate claims; a Linux PocketIC pass does not establish them.
