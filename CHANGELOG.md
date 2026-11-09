# Changelog

## [0.1.0] - Unreleased

### ➕ Added

- Bounded seed buffering and caller-owned ChaCha20 expansion, extracted from
  IcyDB's generator with its existing u128 stream ordering preserved.
- Explicit draw budgets, refusal without stream advancement, and reseeding.
  The core requires no Canic, IC runtime, operating system or allocator.
