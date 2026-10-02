# F06 documentation and test verification

Development checks completed before drafting the artifact-preflight PR. Base:
`aef0ab24f77559593b6d0b4fe5027f8a5eaa857e`. The [machine-readable receipt](suggestion-evaluation-preflight-verification.json)
retains exact measured source/report hashes, denominator and uncovered lines.
All fixtures are synthetic development inputs. Authentic evaluation and acceptance
remain open.

## Documentation check

The new `suggest::eval` module and `SuggestEvalCommand` enforce
`deny(missing_docs)`; their authored public APIs compile without missing docs.
The new command variant and fields are documented. The shared captured-byte
validator documents caller confinement/source-stability responsibilities and
mechanical checks. Strict `cargo doc --lib --no-deps --locked --offline` passes
with broken and private intra-doc links denied. Five pre-existing API links were
corrected without behavior changes.

This checks new authored public documentation, not a whole-repository or private
item documentation percentage. Stable numeric rustdoc coverage is unavailable
without unstable options; no override was used. The [usage guide](suggestion-evaluation-preflight.md)
example was executed against the development binary: one expected/not-run case,
incomplete status, no output files and exit 1. The report keeps all qualification
flags false. A full documentation review/update remains required at final roadmap
integration.

## Measured test coverage

Rust 1.98.1 (`48a229ceaefd4985c50990b14116b6d856af0985`), LLVM 22.1.8,
cargo-llvm-cov 0.8.7, macOS/aarch64. All library unit tests and the
`suggest_eval_preflight_test` / `suggest_validate_cli_test` targets ran:
**1,869 passed, 0 failed, 0 ignored, 0 measured, 0 filtered**. Shared fixture helper
tests are included in that count; they are not independent acceptance cases.

LCOV instrumented production lines before each `cfg(test)` block:

| File | Covered/instrumented lines | Coverage |
| --- | ---: | ---: |
| `eval/capture.rs` | 176/182 | 96.70% |
| `eval/manifest.rs` | 91/91 | 100.00% |
| `eval/preflight.rs` | 238/244 | 97.54% |
| `eval/report.rs` | 66/69 | 95.65% |
| **New preflight modules** | **571/586** | **97.44%** |
| Shared `suggest/validate.rs` | 299/303 | 98.68% |
| New CLI Eval dispatcher | 12/18 | 66.67% |

The `eval/mod.rs` declaration-only file has no instrumented executable lines and
no percentage. Denominators exclude blank/uninstrumented lines and embedded
tests; these are not whole-repository, branch, path or platform percentages.
The shared validator row includes its existing production behavior, not only
changed lines. The new dispatcher scope is recorded exactly in the receipt.

Coverage review added trusted source/evidence cardinality, duplicate source key,
missing request and mutated report empty/duplicate denominator regressions.
Other tests cover exact byte/hash/importer/source binding, ordinary candidate
refusals, complete failed/blocked/tool-error/not-run accounting, opaque approval
and process-mode assertions, disputes, mapping/drafting, UTF-8 boundaries,
retained-copy reconstruction, changed file identity, bounded escaped output,
CLI exit/stdout/privacy behavior and no-replace publication. Windows refusal has
an explicit platform test; this local run does not establish Windows execution.

Remaining uncovered lines are conversion/I/O/provenance guards, repeat-path and
alias checks, and the unused bounded writer flush implementation. Dispatcher
gaps are UTF-8 encoding and stdout failure closures. No coverage threshold or
independent quality/acceptance judgment is inferred.

## Other gates

Strict all-target/all-feature Clippy passed on Rust 1.98.1. The mandatory commit
hook separately runs formatting, strict default-feature all-target Clippy and the
full Rust suite; exact commit and hosted results belong to the PR/tracker receipt.
Current-stable compatibility is independently drafted in PR #169. Dependency
acceptance, the final full documentation review and the full F06 evaluation scope
remain required.
