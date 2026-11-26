# IC Entropy

One IC entropy crate for fresh management randomness, bounded local generation
and optional background buffering, upstream of Canic, IcyDB and Auth.

Use [`ic-entropy`](https://crates.io/crates/ic-entropy). Platform Runtime and
upstream ChaCha20 are normal dependencies. Enable `buffered` for Timers-driven
refill, `metrics` for host-shareable counters, and `logging` for a host log sink.
The crate exports no canister endpoints or lifecycle entrypoints; hosts compose
it into their existing lifecycle and retain consumer admission policy.

## Acquisition and generation

`fresh_seed().await` makes one bounded-wait management `raw_rand` request with
no attached cycles, no cached result and no automatic retry. Call it after the
relevant decision commitment when fresh-after-commitment randomness is required.

With `buffered`, call `ic_entropy::reconstruct()` after restoring the complete
host/shared timer provider during init or post-upgrade. One retained watchdog
fills a 256-seed pool. Failures use bounded backoff and pause after eight
consecutive failures; `retry_refill()` explicitly reopens the budget.

Each consumer can take a seed and retain an `ic_entropy::Generator` with its own
explicit output-byte budget. `SeedPool<N>` and `Generator` are allocation-free
modules within this crate. The shared `fill_bytes()` convenience API uses a
64 KiB per-seed budget and admits at most 1 KiB per call. Buffered seeds and
expanded bytes do not carry fresh-after-commitment guarantees.

See the [package setup, APIs and lifecycle contract](crates/ic-entropy/README.md)
and its [consumer-owned generator example](crates/ic-entropy/examples/consumer_owned.rs).
Metrics share host-owned `Rc<MetricsStorage>`; logging accepts a sink compatible
with Canic's shared logger. Neither integration adds a Canic dependency or
reporting endpoint.

## Entropy and lifecycle

Expansion does not increase seed entropy. Use independent cryptographically
secure seeds, never clocks, counters or replayed output. Output-byte budgets
limit usage; they do not measure remaining entropy bits.

All entropy state is volatile. Suspend before supported snapshot/shutdown and
discard every consumer-owned generator. Reconstruct the complete host/provider
before resuming. Restoring a captured heap alone is not supported. Stop/drain
before upgrading, then reconstruct fresh state. Canister state is not assumed
confidential from subnet operators; no TEE protection is claimed.

## Migration to 0.6

All functionality now belongs to **`ic-entropy`**, with Rust import path
`ic_entropy`. The package requires IC Runtime and `std`; it no longer offers a
separate `no_std` product. Generator and pool operations remain allocation-free.

Replace `ic-entropy-ic` dependencies and `ic_entropy_ic` imports with `ic-entropy`
and `ic_entropy`. Existing generator/pool callers keep their `ic_entropy` paths.
Enable `buffered`, `metrics` and `logging` on `ic-entropy` as needed.

The latest [`ic-entropy-ic`](https://crates.io/crates/ic-entropy-ic) release is an
empty retirement notice linking here. It has no APIs, features or forwarding
dependency. The unpublished `ic-entropy-core` package was removed and will not be
published. The active crate's log source and refill watchdog prefix are
`ic-entropy`; update host filters alongside imports.

## Development

Rust 1.100.0 with rustfmt and Clippy; Wasm uses `wasm32-unknown-unknown`.
`make test`, `make lint`, `make check-wasm` and `make doc-check` aggregate all
eight active-crate feature combinations and directly check the retirement notice.
For focused work, select `-p ic-entropy` and the appropriate features. Native and
Wasm Clippy use `CARGO_BUILD_WARNINGS=deny`; Rustdoc retains strict warnings.

The independent [qualification workspace](qualification/README.md) exercises real
management calls and timers on PocketIC without a framework dependency. Prepare
it with `make install-qualification-tools`, then run `make qualification-check`
and `make test-installed`. CI covers the declared native host matrix; local Linux
checks do not establish macOS execution.

`make check-rules` checks dependencies and source boundaries. Shared tooling lives
under `vendor/toko-tooling/`; repository-owned adapters stay under `scripts/`.
Refresh the authenticated snapshot with `make toko-tooling-update`. Use
`make deps-list` to list product packages and versions. Add
`DEPENDENCY_MANIFEST=qualification/Cargo.toml` to include the qualification probe.
Use `toko-cloc` for source counts and
`toko-cloc --manifest qualification/Cargo.toml` for qualification source counts.

The selected repository license is the [Toko Studios Network Licence](LICENSE.md).
Contributions follow the canonical [contribution policy](CONTRIBUTING.md).
