# Source provenance

The ChaCha20 generator and high-word/low-word u128 construction originate in
`dragginzgame/icydb`, `crates/icydb-core/src/types/random.rs`, reviewed at commit
`f2e2e50c5ec31dc1be3f9b7e100cb6a782388f99`.
The source file was unchanged relative to that checkout's HEAD. Its SHA-256 was
`ca81bb5ca85a58f4598f01f1779ba3618b424c05f8aa250ba07b0e0ce30e04c0`.

Retain IcyDB's complete Toko Studios Source Available License in `LICENSE`,
including its existing copyright notice. This extraction does not relicense it.
Upstream ChaCha20 remains implemented by `rand_chacha`, under its own license.

Changes: move from a database-owned global to an explicitly owned generator;
leave native acquisition and database lifetime with IcyDB; add byte budgets,
whole-request refusal and explicit seed replacement. Add a separate bounded
FIFO of externally supplied seeds. No cryptographic primitive or custom hash
chain is introduced. Tests compare successive u128 values against the original
construction and cover refusal, replacement, FIFO wraparound and zero capacity.
