# F10 sealed assessment epochs: measured engineering verification

This is a pre-PR engineering receipt for `forge assessment results append-epoch`,
measured against parent `e236d85d38bd852b3c9fe00bf9ac6eaab96f4a51` on
`codex/f10-sealed-assessment-epochs`. It does not complete PRD 063 S-4 acceptance.
The [machine receipt](2026-10-04-f10-epochs-integration-verification.json) retains
the eleven exact source/schema pins, complete declaration comparisons and
omissions, every associated alias, zero/unmapped entries, and original job identities.
The [command guide](../assessment-results-epochs.md) defines the supported profile.

## Actual execution

The fresh whole-workspace run used Rust 1.99.0 on `aarch64-apple-darwin`, the locked
dependency graph, all features, offline dependency resolution, and two build jobs:

```sh
cargo llvm-cov --workspace --all-features --locked --json --output-path RAW_JSON
```

It completed on 2026-10-04 at 17:39:52 UTC with **3,407 passed, zero failed and
three existing ignored tests**, across 73 reported test summaries. The raw output
did not exist before launch. HEAD, all recorded workspace bytes and protected
tracker configuration were unchanged during the run. Final strict
`cargo clippy --locked --offline --all-targets --all-features -- -D warnings`
passed; the final `cargo fmt --all` run left source bytes unchanged.

The complete raw LLVM JSON is 24,542,837 bytes, SHA-256
`4eb698470ddc9238fe8e769b5e47be4c78635165a5d30033f164aa22e9b29c6f`.
The actual closed job receipt and log are identified in the machine receipt.
Raw files remain machine-local temporary evidence, not remotely published attachments.
The tracked receipt is a bounded metadata projection, not a replacement raw export.

## Docstring census before drafting

The frozen lexical census covers exactly these nine Rust files:
`src/assessment_results/{mod,epoch_report}.rs`, `src/cli/mod.rs`,
`src/evidence_capture.rs`, `src/poam/{mod,source,assessment_epochs,assessment_epochs_cli}.rs`
and `tests/assessment_epochs_cli_test.rs`. The two new JSON schemas are pinned
inputs but are not Rust docstring denominators.

| Category | Selected documented / total | Whole nine files documented / total |
| --- | ---: | ---: |
| Functions | 328 / 328 | 461 / 523 |
| Types | 62 / 62 | 126 / 127 |
| Fields, variants and tuple payload members | 329 / 329 | 783 / 871 |
| Literal module declarations | 6 / 6 | 28 / 43 |

Selected functions include 318 new declarations and ten existing functions with
literal changes. Selected types, members and modules include new declarations
only; changed existing containers remain in the whole denominator and comparison
rows. Selected functions comprise 195 production lexical declarations, 75 under
test configuration and 58 integration declarations. There are 71 selected literal
test registrations; that is a source count, not a test-run total.

The reader is a literal lexer, not a Rust AST, macro expansion, type checker or cfg
evaluator. All 42 whole-file function cfg ambiguities and legacy omissions remain
visible. File-level inner Rustdoc occurrences are counted separately; they do not
replace documentation on literal module declarations. The first census found ten
missing tuple-payload docstrings; the final successor includes their documentation
and preserves the first census. This is not a claim of repository-wide 100% docs.

## Native coverage and exact header association

| Whole LLVM export metric | Covered / total | Percentage |
| --- | ---: | ---: |
| Lines | 89,459 / 97,614 | 91.65% |
| Functions | 7,951 / 9,490 | 83.78% |
| Regions | 157,659 / 173,706 | 90.76% |
| Instantiations | 11,502 / 17,163 | 67.02% |

These export totals include production, tests and generated code. Branch and
MC/DC denominators are both zero and therefore unavailable.

Separately, the literal-header association maps only an exact source filename,
visibility/modifier header line and UTF8 byte column to the first regular LLVM
`CodeRegion`. It refuses nearest-region, name-only, closure and `fn` keyword
substitution. A positive header requires at least one exact alias with a positive
entry count; it does not imply that every alias or every line in the body ran.

| Literal header scope | Positive | Zero | Unmapped | Total | Positive / zero exact aliases |
| --- | ---: | ---: | ---: | ---: | ---: |
| Selected functions | 327 | 0 | 1 | 328 | 575 / 34 |
| Whole nine files | 518 | 4 | 1 | 523 | 897 / 74 |

The unmapped selected header is
`unsupported_publication_never_emits_an_epoch_pair`, at line 420 of
`tests/assessment_epochs_cli_test.rs`. It is guarded by
`not(any(target_os = "linux", target_os = "macos"))`; this macOS run supplies no
Windows or other unsupported-platform credit.

The four whole-file zero-entry headers are unchanged existing functions:
`assessment_results::error`, `CountingWriter::flush`, `BoundedWriter::flush`
in evidence capture, and `CanonicalHash::flush` in POA&M source loading. All 74
zero aliases, 993 nonheader first-region records within the explicit source list,
and 6,892 zero-count raw function indices remain retained in the machine receipt.
Unmatched generated/closure records are not reassigned to named source functions.

## Meaningful controls and preserved failures

Actual focused runs passed 14 report controls, all 30 capture controls (including
nine new controls), 29 producer controls, and 17 supported-platform CLI controls.
They cover real captured inputs, later third/fourth appends, latest-member refusal,
complete count tampering, bounded complete views, native schema validation,
old/nonlatest selection, original-generation drift and no-replace publication.
CLI controls also consume the new native file through maintained POA&M commands.
Fault-injected producer publication controls exercise the private consumer order;
they do not independently establish platform publisher behavior.

Initial compilation/lint failures were corrected without suppressing warnings.
One intended positive test used a raw non-ASCII URI rejected by the pinned native
OSCAL schema before append. Its successor uses an actual descendant companion
path; the failing run remains recorded. The final whole run measures the corrected
source, including final helper and docstring changes. Earlier successful focused
cohorts are not relabeled as executions of that final source.

## Remaining gates

The delivered profile appends sealed same-context epochs and explicit latest-member
risk continuity with fresh identities. Different context/actor/receipt generations,
open or overlapping windows, branching families, and native-only continuity
interoperability remain outside it. Generic public `forge validate` does not accept
Assessment Results; append validates against the vendored OSCAL 1.2.3 AR schema.
Representative assessor workflows, independent downstream interoperability,
recorded owner decisions, dependency-audit acceptance and release authority remain
open. The full goal-wide documentation review remains a final gate.

The parent PR has passing platform/API and installed-Chrome checks, but its hosted
dependency supply-chain and Linux OS-denial prerequisites failed. This local
receipt does not convert those failures into passing gates or acceptance.
