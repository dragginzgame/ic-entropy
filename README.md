# IC Entropy

Entropy primitives and an optional IC acquisition service, upstream of Canic,
IcyDB, Auth and other consumers.

| Package | Responsibility |
| --- | --- |
| `ic-entropy` | Allocation-free `no_std` seed pool and bounded upstream ChaCha20 expansion. No Platform, Timers or consumer-framework dependencies. |
| `ic-entropy-ic` | Fresh Platform `raw_rand` requests, plus opt-in background buffering through IC Timers. Hosts compose lifecycle hooks and retain consumer admission policy. |

`SeedPool<N>` buffers externally acquired 32-byte seeds in FIFO order, consuming
each entry once and refusing insertion when full. `Generator` expands one seed
with an explicit byte budget and atomic refusal on exhaustion. It preserves
IcyDB's high-word/low-word u128 construction; `fill_bytes` supplies smaller values.

## Background acquisition

Enable `ic-entropy-ic`'s `buffered` feature and call
`ic_entropy_ic::reconstruct()` from the host's existing synchronous init and
post-upgrade participants after runtime restoration. One retained IC Timers
watchdog fills a 256-seed pool through Platform `raw_rand`. It idles when full
and resumes after consumption. Successful refills are spaced at least one second
apart, with bounded backoff and an explicit retry after eight consecutive failures.

Each consumer calls `take_seed()` and owns an `ic_entropy::Generator` with its own
explicit output-byte budget. The pool capacity is fixed; expansion policy is
consumer-selected. Handle `NotReady` and `Exhausted`; reconstruction schedules background
work rather than synchronously obtaining entropy. For fresh-after-commitment
randomness, use `fresh_seed().await` after the decision commitment instead.

See the [IC adapter setup and lifecycle contract](crates/ic-entropy-ic/README.md).
Downstream adoption is separate from this package's release.
Canic's existing local adapter must be replaced during migration;
consumers must not run both services or retain a second refill implementation.

## Entropy and lifecycle

Hosts acquire seeds for the standalone core, choose budgets and compose lifecycle.
Neither buffering nor deterministic expansion increases the entropy of a seed.
Use full cryptographically secure seeds; never substitute clocks/counters, log
seeds, or restore an old stream for new security-sensitive draws. Separate
consumers need independent seeds. Reset and acquire fresh entropy after upgrades
or restores. An output-byte budget describes usage, not entropy bits.

The IC adapter's snapshot contract requires suspension before capture and complete
host/shared-provider reconstruction after restore. A restored heap alone is not
safe to resume. Canister state is not assumed confidential from subnet operators,
and no TEE protection is claimed.

## Development

Rust 1.100.0 with rustfmt and Clippy; Wasm uses `wasm32-unknown-unknown`.
`make test`, `make lint`, `make check-wasm`, and `make doc-check` aggregate the core and adapter,
including the adapter's fresh-only and buffered selections. For focused work,
select `-p ic-entropy` or `-p ic-entropy-ic` and the appropriate feature set.
Wasm lint checks and Rustdoc use strict warnings, as do native lint checks.
Linux checks do not establish native macOS or installed IC execution.

The independent [qualification workspace](qualification/README.md) exercises real
management calls and timers on PocketIC without a framework dependency. Prepare
it with `make install-qualification-tools`, then run `make qualification-check`
and `make test-installed`. CI runs this journey on its native host matrix.

The selected repository license is the [Toko Studios Network Licence](LICENSE.md).
Contributions follow the canonical [contribution policy](CONTRIBUTING.md).

The published core is available as `ic-entropy = "0.1.0"`.
