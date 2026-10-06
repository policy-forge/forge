# Integrated Assessment Results baseline guard verification

The current stack now refuses a schema-valid Assessment Results baseline with
more than one result epoch before extracting snapshots or changing the supplied
report. The CLI exits `2` before artifact/report publication, including with
`--fail-on never`. One-result comparisons keep their existing behavior. This
integrates the prerequisite retained in draft #176 at `3b0b5a294810cf5a8b25d9f37c661c31d763b33f`
onto source basis `8342b92f963031866cd40ceefea56418c00c9e2a`; shared current
CLI, README and usage changes are preserved. Original draft receipts remain
historical and are not transferred to this execution.

See the [CLI contract](assessment-results.md),
[integration plan](plans/2026-10-03-f10-baseline-guard-integration.md) and
[source-bound report](plans/2026-10-03-f10-baseline-integration-verification.json).
The report uses format `forge.f10-integrated-baseline-verification/2` and was
derived before drafting. Its SHA-256 is
`0c1fd69b8b0f57ea1e3af8a31174dd112c86da3563850741d0831e4dcf0afd64`.

## Actual current-source checks

Rust 1.99.0, the existing locked dependency graph and default local development
target were used. No new dependency was introduced.

| Check | Actual outcome |
| --- | --- |
| Tests before applying the guard | Both plural API/CLI regressions failed, unchanged source; API accepted plural input and CLI returned `1` instead of invalid-input `2` |
| Full Assessment Results integration suite | 16 passed, zero failed/ignored |
| Strict all-target/all-feature Clippy | Passed with `-D warnings` |
| Formatting | `cargo fmt --check` passed |
| Fresh whole-workspace/all-feature LLVM suite | 2,995 passed, zero failed, three ignored across 68 summaries |

The three added registered tests use the real builder and schema validator.
Four plural fixtures combine populated or conclusion-free additional epochs in
both result orders, with distinct stable keys and UUIDs. The CLI test completes
32 combinations across those fixtures, any/never exit gates, absent/existing
binary sentinels, and file/stdout artifact modes. It requires unchanged source
and destination bytes, no stdout artifact, and no file/report publication.
All 32 separately retained synthetic CLI captures reconcile to exit `2` and
empty stdout, with four distinct baseline hashes. The direct API test preserves
the whole prepopulated report for all four fixtures. A one-result control checks
whole artifact/report equality through the API and both CLI exit gates.
These combinations are not extra registered tests or authentic assessor epochs.

Each of the four passing jobs records identical before/after pins for all seven
integrated source/document paths. The original two regression failures are
retained without pass credit. The enabled commit hook is a separate delivery
check; its actual result belongs to the immutable draft description and receipt.
This guide does not predict that result.

## Documentation and coverage denominators

All nine selected declarations have adjacent Rustdoc: two production functions
and seven added test/helper functions. Exact native filename, declaration line
and UTF-8 byte column map all nine to a positive first regular LLVM CodeRegion.
All 11 matching instances remain recorded, including two zero instances; a
positive header does not imply every instantiation or function body executed.

The complete two-file lexical cohort has 33 named declarations and nine with
Rustdoc, retaining 24 legacy omissions. It has 32 positive headers and one zero
(`baseline::bounded`), with zero unmapped headers. Its 12 top-level literal
`#[test]` attributes are distinct from the suite's 16 registered runtime tests,
which also include four common-module tests. The initial pure collector missed
one multiline attribute when classifying a historical test. Version 2 corrects
that metadata from identical source and runtime inputs; version 1 remains
preserved and adds no execution.

The native export reports 6,536/7,675 covered functions and 72,547/79,766 covered
lines. These mixed production/test/generated summaries are not whole-repository
production-only coverage. The baseline file's native line summary is 311/361;
LLVM does not supply a separate integration-file line summary in this export.
All raw whole-export zero-function indices, named zero/unmapped maps, matching
instances and available file summaries remain in the report. Branch and MC/DC
denominators are zero and unavailable. Header observations do not establish
full-body, branch, platform or acceptance coverage.

## Remaining goal work

This guard does not implement F10 reviewed-risk handoff or plural epochs. The
successor needs explicit kind/key/UUID/result/fingerprint selection into the
F07/F08 POA&M contract, without invented owners or dates, and bounded per-epoch
context, identity, output, report and baseline correspondence. Keep the refusal
until those semantics and actual workflows are implemented.

Interoperability, dependency/owner decisions, qualified Linux OS denial,
Windows/platform/crash/rollback, independent security/privacy,
assistive-technology/participant judgments and verified 2.0.0 release provenance
remain separate gates. The user's full integrated documentation review across
all completed roadmap work remains open. Local controls, source metadata and
draft PRs confer no acceptance, task closure, merge or publication.
