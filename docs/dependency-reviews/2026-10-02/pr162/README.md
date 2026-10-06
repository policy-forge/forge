# PR 162: argon2

This records a bounded agent-assisted differential review at the immutable head
in [receipt.json](receipt.json), with base `aef0ab24f77559593b6d0b4fe5027f8a5eaa857e`. The retained final
snapshot shows OPEN/BLOCKED; this is historical state, not a current merge claim.

Argon2 0.5.3 → 0.6.0, blake2 0.10.6 → 0.11.0 and password-hash 0.5.0 →
0.6.1 introduce phc 0.6.1, ctutils 0.4.2 and cmov 0.5.4. Argon2 now routes to
already-locked cpufeatures 0.3.0; blake2 routes to digest 0.11.3, activating
ctutils through mac. All six new versions declare Rust 1.85. Forge retains
zeroize and defaults (now alloc/getrandom/password-hash), and does not select
parallel or kdf. Its raw Argon2id V0x13 path uses fixed 64 MiB/3 iterations/1 lane,
32-byte tag and 16-byte salt. Fallible allocation and NonNull-backed sequential
segment traversal are material changes requiring full attributable review.
Six upstream raw KAT expected tags are unchanged, but the tests were not run.
KDF scratch-memory erasure is not established by the enabled zeroize feature;
the old and new allocation paths lack an explicit wipe before freeing scratch
memory. This is a baseline observation, not a validated new vulnerability.
No new actionable regression was identified in the bounded review.

Existing September 25 hosted tests passed on Ubuntu, macOS and Windows, and
cargo-audit/cargo-deny passed. Cargo-vet failed with 26
unvetted versions, including twenty shared baseline gaps. See
[CI evidence](ci/) and [full lock changes](Cargo.lock.diff).
No Cargo or candidate execution was performed during review. The baseline F02
graph covers default features and four native targets; it is not a resolved graph
for this candidate. Stable CI does not establish pinned Rust 1.85 or all four
architecture-specific targets.

Predecessor versions are exempted rather than locally source-audited. This
differential cannot inherit unchanged-source approval. No accepted audit or
human signoff is recorded. Current advisory matching and historical review
details are retained in this directory and the [shared packet](../README.md).

The existing ttf-parser 0.25.1 informational maintenance disposition remains:
deny.toml names owner `deps-rotation`, REVIEW-BY 2026-12-31 and removal after
lopdf drops that dependency. Preserve this recorded rationale; a passing advisory
step does not remove the warning or satisfy D069 source-audit acceptance.

D069 signer/import/exception decisions, attributable accepted reviews, candidate
graph/MSRV/target verification and final documentation review remain open.
