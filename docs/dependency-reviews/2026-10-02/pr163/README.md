# PR 163: pdf-extract

This records a bounded agent-assisted differential review at the immutable head
in [receipt.json](receipt.json), with base `aef0ab24f77559593b6d0b4fe5027f8a5eaa857e`. The retained final
snapshot shows OPEN/BLOCKED; this is historical state, not a current merge claim.

Pdf-extract 0.12.0 → 0.12.1 changes only its lock version/checksum. Its sole
executable delta clears cached fonts at page entry, fixing cross-page reuse of
the same resource name for different fonts/ToUnicode maps. Dependency edges,
feature definitions and build behavior are unchanged; neither version declares
a Rust minimum. A second agent independently verified the source differential
and its reachability from Forge's extract_text(path) call. No new actionable
regression was identified. Existing blank-PDF tests do not demonstrate the fix;
a deterministic two-page font-map fixture remains a meaningful verification gap.
Baseline malformed-PDF panic, extraction/output limits, input-path reopening and
recursive/same-page Form-resource behavior are observations requiring dedicated
validation before any accepted security finding. They are not fixed by this patch.

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
