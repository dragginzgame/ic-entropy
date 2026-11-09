# Source provenance

Extracted on 2026-11-09 from the uncommitted `crates/canic-entropy` implementation
in the local `dragginzgame/canic` checkout, recorded there under issue #757.
There is no committed supplying revision for these files. Preserve Canic's MIT
license and `Copyright (c) 2024 Canic contributors` in this package's `LICENSE`.
The dependency `ic-entropy` retains its separate IcyDB-derived source license.

SHA-256 of the source files at extraction, before adaptation:

| Source path | SHA-256 |
| --- | --- |
| `src/lib.rs` | `dd2d2a7d2dbf4735818935c829fc5f5add24ad416a2baaeb153cf2083dd4bcab` |
| `src/source/mod.rs` | `a80fafaec862ce4e2f2c662c0db5d492a283507feea81791e3ca0aea7a724b6d` |
| `src/buffered/mod.rs` | `5311784376b40772505f7b8db0473f84291bff927a762ab986e271a1c3e15d38` |
| `src/buffered/model/mod.rs` | `028d854d3e97a3731ec7e51b286de495eb88a5740de5f6ffde0690b4bac3bb78` |

Changes: move acquisition/refill ownership upstream of consumer frameworks,
select Runtime directly for execution and Platform only for the `raw_rand` codec,
rename the watchdog identity, and use fixed management-protocol fixtures in tests
instead of a general Candid development dependency. No entropy mixer or
cryptographic primitive was added. Consumer lifecycle composition remains external.
