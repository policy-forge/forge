# PR 164: getrandom

This records a bounded agent-assisted differential review at the immutable head
in [receipt.json](receipt.json), with base `aef0ab24f77559593b6d0b4fe5027f8a5eaa857e`. The retained final
snapshot shows OPEN/BLOCKED; this is historical state, not a current merge claim.

Getrandom 0.4.2 → 0.4.3 retains Rust 1.85, feature definitions and build.rs.
Reviewed native Linux, Windows and Apple backend files are byte-identical.
Changes add wasm64 JS gating and manual WASI p2/p3 bindings. The supported graph
is native; no WASI/wasm assurance is inferred. Seventeen lock versions disappear
(old getrandom plus sixteen binding/code-generation transitives), but wasip2
remains through another path. Resolver effects remove indexmap serde edges and
wit-bindgen's macro edge. Unchanged tempfile 3.27.0 reroutes to already-locked
getrandom 0.3.4, allowed by its >=0.3,<0.5 range. Direct Forge/lopdf/rand/uuid
edges move to 0.4.3. Therefore a new native graph is required; this is broader
than a one-version substitution. No new actionable regression was identified.

Existing September 25 hosted tests passed on Ubuntu, macOS and Windows, and
cargo-audit/cargo-deny passed. Cargo-vet failed with 21
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
