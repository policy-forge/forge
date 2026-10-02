# F07 documentation and test verification

Development verification of the proposed mechanical foundation, performed before
drafting its PR. Base: `aef0ab24f77559593b6d0b4fe5027f8a5eaa857e`. The [machine-readable receipt](poam-foundation-verification.json)
contains exact measured source hashes, uncovered line numbers and raw report hashes.

## Documentation check

The new `poam` module and CLI `PoamCommand`/`PoamInitArgs` enforce
`deny(missing_docs)`. Their authored public API has no missing-doc errors.
`cargo doc --lib --no-deps --locked --offline` passed with both
`rustdoc::broken_intra_doc_links` and `rustdoc::private_intra_doc_links` denied.
The pass filled 38 missing public field/variant entries and expanded API contracts.
Five existing links were corrected without behavior changes.

This compiler check covers the new public API. It does not establish a documentation
percentage for private items or the entire repository. Stable rustdoc's numeric
`--show-coverage` requires unstable options; no bootstrap or nightly override was used.
The [usage guide](poam-foundation.md) and README describe commands and their source-only
boundary. A full documentation review/update remains required at final roadmap integration.

## Measured test coverage

Rust 1.98.1 (`48a229ceaefd4985c50990b14116b6d856af0985`), LLVM 22.1.8,
cargo-llvm-cov 0.8.7, macOS/aarch64. The run included all library unit tests plus
`poam_foundation_test`, `schema_provenance_test` and `assessment_results_test`:
**1,904 passed, 0 failed, 0 ignored**. Shared fixture helpers are included
in those test-run counts; they are not independent acceptance cases.

LCOV instrumented production lines before each module's `cfg(test)` block:

| File | Covered/instrumented lines | Coverage |
| --- | ---: | ---: |
| `src/poam/identity.rs` | 17/17 | 100.00% |
| `src/poam/manifest.rs` | 181/182 | 99.45% |
| `src/poam/mod.rs` | 160/165 | 96.97% |
| `src/poam/report.rs` | 60/64 | 93.75% |
| `src/poam/source.rs` | 904/978 | 92.43% |
| **Combined** | **1322/1406** | **94.03%** |

The denominator contains instrumented production lines, excluding embedded tests,
blank lines and uninstrumented source. It is not whole-repository, branch, path or
platform coverage. Existing shared adapters and enum-registration changes are
exercised but not included in this five-module percentage.

Coverage review added reachable tests for exact companion identities and roots,
all required receipts, result selection beyond the first epoch, cross-epoch refs,
fresh-hash schema/scope/actor/task failures, nested receipt href resolution,
scaffold capture replacement and bounded escaped reports. Remaining uncovered
lines include I/O/serialization failure guards, aggregate-limit rejection paths,
unsupported source variants and a concurrent manifest-replacement window. No
percentage threshold or independent acceptance was inferred from this run.

## Other verification and limits

`cargo clippy --locked --offline --all-targets --all-features -- -D warnings` passed.
The commit hook separately requires formatting, default-feature strict Clippy and
the full test suite; exact commit/hosted-CI results are reported with the PR.
These local receipts use Rust 1.98.1, matching the prior baseline verification.
They do not establish current-stable Rust 1.99 CI success. Existing baseline
Clippy and unvetted dependency failures remain separate F03/release work.

The official POA&M asset SHA-256 is
`f4fd94487408a9589954b5b92d88965c984d4f79b85365cc574b860898759437`; previous schema assets are preserved.
D064, PRD 063 source acceptance, full F08 workflows, Windows publication,
independent consumers and authentic human remediation evaluation remain open.
