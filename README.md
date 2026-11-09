# IC Entropy

Small, allocation-free `no_std` entropy primitives, independent of Canic and IcyDB.

- `SeedPool<N>` buffers at most N externally acquired 32-byte seeds in FIFO order.
  Each entry is consumed once; a full pool refuses insertion without overwriting.
- `Generator` expands one seed through upstream ChaCha20, with an explicit byte
  budget and atomic refusal on exhaustion. It preserves IcyDB's existing u128
  high-word/low-word construction. `fill_bytes` also supplies smaller values.

The host obtains seeds (IC Platform's `raw_rand` or native OS entropy), chooses
budgets, and schedules retries/reseeding. No acquisition, timer, job, global state,
persistence or management-canister dependency is hidden in the core. A Canic
adapter can own that composition without constraining the core's dependency graph.
The pool and generator are separate: consuming buffered raw bytes is not the same
contract as expanding a seed. Neither provides a fresh-after-commitment draw.

Use full cryptographically secure seeds. Four bits of uncertainty cannot become
16 secure bits by expansion. Output byte budgets describe usage, not entropy bits.
Never substitute clocks/counters for entropy, log seeds, or restore an old stream
for new security-sensitive draws. Reusing a seed replays output. Reset and acquire
fresh entropy after upgrades/restores. Canister state is not assumed confidential
from subnet operators, and no TEE protection is claimed by this crate.

## Development

Rust 1.99.1 with rustfmt and Clippy; Wasm checks use wasm32-unknown-unknown.
Run `make test`, `make lint`, and `make check-wasm`. Native macOS execution remains
required before claiming that host qualified. See [source provenance](crates/ic-entropy/PROVENANCE.md) and [license](LICENSE).

Add `ic-entropy = "0.1.0"` to use the crate from crates.io.
