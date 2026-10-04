# F10 sealed epochs: Windows fixture correction and fresh measurement

Windows CI at `b3e3143a8d176b4571e277a67eb6c9bce9f7fc7e` refused the epoch CLI
test target during strict linting: `Fixture.request` was unused when the supported
publisher tests were excluded. Run `37222779876`, job `111496302707`, did not run
Windows tests. The existing unsupported-platform control now decodes the actual
saved `append.json` and compares its complete value with `fixture.request` before
invoking the refusal path. This two-line fixture assertion changes no production
behavior, adds no test registration, and suppresses no warning. Ordinary fixture
JSON decoding supplies no duplicate-key admission guarantee.

The [machine receipt](2026-10-04-f10-epochs-windows-fixture-verification-v1.json)
binds the corrected source, complete declaration comparisons, native aliases and
actual closed jobs. The [initial pre-PR receipt](2026-10-04-f10-epochs-integration-verification.md)
and [initial machine projection](2026-10-04-f10-epochs-integration-verification.json)
remain byte-for-byte preserved, including their earlier failure history. This
successor records a new generation; it does not relabel earlier executions.

## Fresh actual corrected-source checks

Rust 1.99.0 on `aarch64-apple-darwin`, locked dependencies, all features, offline
resolution and two build jobs were used for:

```sh
cargo fmt --all -- --check
cargo clippy --locked --offline --all-targets --all-features -- -D warnings
cargo llvm-cov --workspace --all-features --locked --json --output-path RAW_JSON
```

All three checks passed. The fresh LLVM run started at 18:21:45 UTC and finished
at 18:23:09 UTC on 2026-10-04: **3,407 passed, zero failed, three existing ignored
tests**, across 73 test summaries. The raw output was absent before launch; all
recorded workspace bytes, HEAD and protected tracker configuration were unchanged
during measurement. The eleven exact Rust/schema pins are retained in the machine
receipt. These receipt files and their documentation links are added afterward.

Raw LLVM: **24,542,844 bytes**, SHA-256
`b1ecb9e5e4969cff630c0c7550a0c13eab07ad706d609e27dea3e1582dbfc011`.
Raw export and command logs remain hash-identified machine-local evidence;
the tracked projection preserves complete declaration and alias denominators.

## Docstrings and coverage

The fresh literal census retains the original parent
`e236d85d38bd852b3c9fe00bf9ac6eaab96f4a51` as comparison basis and measures
HEAD `b3e3143a8d176b4571e277a67eb6c9bce9f7fc7e` plus the uncommitted two-line
fixture correction. It covers nine explicit Rust files:
`src/assessment_results/{mod,epoch_report}.rs`, `src/cli/mod.rs`,
`src/evidence_capture.rs`, `src/poam/{mod,source,assessment_epochs,assessment_epochs_cli}.rs`,
and `tests/assessment_epochs_cli_test.rs`; two JSON schemas are pinned inputs.

| Category | Selected documented / total | Whole nine files documented / total |
| --- | ---: | ---: |
| Functions | 328 / 328 | 461 / 523 |
| Types | 62 / 62 | 126 / 127 |
| Fields, variants and tuple payload members | 329 / 329 | 783 / 871 |
| Literal module declarations | 6 / 6 | 28 / 43 |

Selected functions include 318 new declarations and ten literal changes, with
195 production, 75 test-configuration and 58 integration declarations; 71 have
literal test registrations. Types, members and modules select new declarations
only. The lexer is not an AST, macro expansion or cfg evaluator. All 1,564
declaration comparisons, two removals, whole-file omissions and 42 whole-file
function cfg ambiguities remain retained. File-level inner docs are separate
from literal module docs. These are scoped counts, not repository-wide 100% docs.

| Whole LLVM metric | Covered / total | Percentage |
| --- | ---: | ---: |
| Lines | 89,459 / 97,614 | 91.65% |
| Functions | 7,951 / 9,490 | 83.78% |
| Regions | 157,659 / 173,706 | 90.76% |
| Instantiations | 11,502 / 17,163 | 67.02% |

Whole export includes production, test and generated code. Branch and MC/DC
denominators are zero and unavailable. Totals happen to match the initial run;
the source pins, raw export hash and actual job generation are new.

Exact literal-header association uses the filename, visibility/modifier header
line and UTF8 column at the first regular `CodeRegion`, without name-only,
nearest-region, closure or `fn`-keyword substitution:

| Scope | Positive | Zero | Unmapped | Total | Positive / zero exact aliases |
| --- | ---: | ---: | ---: | ---: | ---: |
| Selected functions | 327 | 0 | 1 | 328 | 575 / 34 |
| Whole nine files | 518 | 4 | 1 | 523 | 897 / 74 |

A positive header means at least one exact alias entered; it does not prove all
aliases, full bodies, branches or platform behavior. The existing unsupported
publisher control at line 420 of `tests/assessment_epochs_cli_test.rs` remains
unmapped on macOS, with **no local Windows body credit**. The four unchanged
whole-file zero headers are `assessment_results::error`, capture
`CountingWriter::flush` and `BoundedWriter::flush`, and POA&M source
`CanonicalHash::flush`. All 74 zero aliases, 993 scoped nonheader records and
6,892 global zero-count raw indices remain visible.

## Acceptance boundaries

The [command guide](../assessment-results-epochs.md) still defines a partial
PRD 063 S-4 same-context sealed profile. Different context/actor/receipt generations,
open or overlapping windows, branching families, native-only continuity
interoperability, representative assessor workflows and owner decisions remain
open. Generic `forge validate` does not accept Assessment Results; append uses
the vendored OSCAL 1.2.3 AR schema. The final goal-wide documentation review,
dependency-audit acceptance and release authorization remain separate gates.

At the original PR #212 head, Ubuntu/macOS Rust and all three API parity checks
passed; Windows Rust lint failed before tests. Linux OS-denial prerequisites
failed before the producer ran, and supply-chain qualification failed. New hosted
results must be read back at the corrected pushed head; local measurement supplies
no replacement credit for those hosted or acceptance gates.
