# PR 163: independent PDF dependency differential review

Review date: 2026-10-02. Read-only review using engineering:code-review. No Cargo jobs, installs, repository/Git mutations, external posting, accepted audits, or human approvals performed.

## Immutable candidate and evidence

- PR: https://github.com/policy-forge/forge/pull/163
- Head: db34fdfccaf00262e9706e8f3cd294f9d1f3bc01
- Base: aef0ab24f77559593b6d0b4fe5027f8a5eaa857e
- Final live reread confirmed the same head/base, OPEN, MERGEABLE and BLOCKED.
- Forge patch: Cargo.lock only; pdf-extract 0.12.0 to 0.12.1, no dependency edges or other package changes. Cargo.toml remains pdf-extract = "0.12".
- Independently verified old archive SHA256: 417e8fdc940f1d5bc62c5f89864c3a2255f74f69aa353c98509213d67df61e73.
- Independently verified new archive SHA256: e5c4820f5811e424ce037d08493ac87752ecc54339e0f1e40c3d17da93278861.
- Archive VCS commits: b95bf9f6268772d5088f09b0034e488e64294835 to 47b792ba04c5f3b6e478072b2efb8e1a2a06ec7e. The immutable upstream GitHub comparison independently matches the archive source delta: https://github.com/jrmuizel/pdf-extract/compare/b95bf9f6268772d5088f09b0034e488e64294835...47b792ba04c5f3b6e478072b2efb8e1a2a06ec7e.

## Result and changed behavior

No actionable newly introduced Forge regression found in this bounded differential review. This is not a full audit of the predecessor source or an accepted safe-to-deploy judgment.

The sole executable change is src/lib.rs:2408, clearing Processor.font_table at entry to output_doc_inner. Four preceding comment lines explain the resource-name collision. All other packaged source files are identical. Normalized and original manifests change only their package version; packaged Cargo.lock changes only the root pdf-extract version. No new build script, feature, dependency, network call, file operation, or unsafe code is added. Both versions omit a declared rust-version, so a declared MSRV cannot be attributed to this crate.

Forge src/ingest/mod.rs:165-171 directly invokes pdf_extract::extract_text(path). The new crate's src/lib.rs:2219-2227 loads the PDF and uses output_doc; lines 2374-2385 share one Processor across all document pages. The font cache is indexed only by resource name at lines 1709-1725. Clearing it before loading each page's Resources therefore prevents a later page's /F0 from incorrectly using an earlier page's font and ToUnicode mapping. Resources inherited from parent page dictionaries are still resolved after the clear. Graphics/text state is freshly constructed inside process_stream at lines 1581-1601; the previous page's active font does not persist in that state. Per-page caching remains effective within a page.

The update can correct extracted text and downstream derived OSCAL content for affected multi-page PDFs; Forge's raw-file fingerprint remains based on unchanged source bytes. Clearing makes shared fonts parse again on successive pages. That cost is an expected consequence of this small correctness fix; no performance measurement was run.

No regression test accompanies the immutable upstream two-commit change. Forge's existing PDF-specific ingestion test uses a blank PDF and checks OcrNotSupported (base src/ingest/mod.rs:530-535), so its success does not demonstrate the repaired font mapping. A deterministic two-page PDF with different /F0 ToUnicode maps would provide meaningful direct verification. No such fixture execution occurred in this review.

## Existing boundaries and risks retained by this patch

These source observations predate 0.12.1. They do not establish new regressions or independently reproduced vulnerability severity.

1. Forge's input cap covers file metadata before fs::read, not decompressed PDF streams, extracted text, page count, PDF execution time, or recursive Form XObjects. The PDF is then reopened by extract_text(path), rather than parsed from the fingerprinted byte buffer. A concurrently changed local file can therefore escape the metadata cap and make the extracted content differ from the fingerprint. The patch does not alter this flow (base src/ingest/mod.rs:104-132).
2. pdf-extract process_stream retains Content::decode(...).unwrap() at line 1582 and operand indexing/assertions. output_doc_inner still unwraps page dictionaries/content and expects MediaBox. Forge maps returned extraction errors, but its single-file ingestion path has no catch_unwind. Batch orchestration catches unwind panics (base src/batch/orchestrator.rs:126-147); that does not bound allocations or contain process aborts.
3. Nested Form XObjects recursively call process_stream with their own resource dictionaries and the same Processor/font-name cache (pdf-extract lines 1860-1868). Page-entry clearing fixes cross-page collisions, while within-page resource-scope collisions and recursive resource processing are unchanged. get_inherited also recursively follows Parent without a local cycle/depth guard (lines 2352-2360). These observations need dedicated validation before reporting accepted security findings.
4. Clearing cannot supply predecessor audit assurance. Base supply-chain/config.toml:638-640 contains an exemption for pdf-extract 0.12.0; supply-chain/audits.toml has no pdf-extract source audit. The new version needs attributable policy disposition, rather than automatic inheritance of an exemption.

## Advisory and hosted-check evidence

Current primary RustSec tree 117edb3bed98e9be112f277b7615eea3252e7c43 is not truncated. Independent index lookup for pdf-extract and immediate parser/resource dependencies matches lopdf, postscript and ttf-parser records; no pdf-extract record is listed in that snapshot. The parent reviewer also persisted the complete frozen PDF closure lookup in advisory-index-receipt.json and advisory-summaries.json. Its vulnerability/unsound records have patched locked versions; this is source-index comparison, not a fresh cargo-audit run.

- Unchanged lopdf 0.42.0 satisfies RUSTSEC-2026-0187 patched >=0.42.0. That advisory describes stack-overflow aborts in earlier nested-object parsers, which cannot be caught with catch_unwind: https://rustsec.org/advisories/RUSTSEC-2026-0187.
- Unchanged postscript 0.14.1 satisfies RUSTSEC-2021-0017 patched >=0.14.0: https://rustsec.org/advisories/RUSTSEC-2021-0017.html.
- Unchanged ttf-parser 0.25.1 retains informational unmaintained RUSTSEC-2026-0192 with no patched versions: https://rustsec.org/advisories/RUSTSEC-2026-0192. Base deny.toml already ignores it with deps-rotation owner and REVIEW-BY 2026-12-31. This remains a maintenance exception requiring owner disposition under D069; this patch does not remove it.

Hosted CI run 36143881134 is associated with this PR head. Three OS test jobs passed; the supply-chain job's Security audit and License and advisory check steps passed, while Supply-chain audit failed. Existing cargo-vet log lists 21 unvetted versions: 20 shared baseline gaps plus pdf-extract 0.12.1. CI link: https://github.com/policy-forge/forge/actions/runs/36143881134.

Base CI uses Rust stable, not the declared Forge 1.85 floor, and its three OS labels do not independently prove all four architecture-specific targets in the F02 graph. F02 target/feature graph is used only as frozen baseline context; new-version graph was not regenerated here. No fresh candidate execution, malformed-PDF test, corrected font-map test, architecture/MSRV run, or full transitive source audit was performed.

D069 owner disposition, accepted attributable audits, required platform/MSRV evidence and the final full-documentation review remain open. No merge recommendation or release acceptance is recorded.
