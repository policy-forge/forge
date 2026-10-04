# Evidence overlay integration verification

This record covers the final local F09 evidence-overlay slice measured from six Rust source files at comparison and measurement head `4e578584644aadfcd5a736877ec6822314dea784`. The worktree contained the explicit overlay changes; the head alone does not identify those working bytes. Exact file hashes below bind the measurement. This is a source and verification record, with full F09 interoperability and acceptance still open.

The [machine audit](plans/2026-10-04-f09-evidence-overlay-integration-verification.json) retains every before and after lexical row once, all omission locators, exact native aliases, raw zero records, cfg qualifications and authenticated job/source bindings. It separates original source, historical proposals and final measured source. The proposed metadata publication omits its own resulting file hash; the TEMP packet index pins its exact bytes independently.

## Documentation scope

Selected means a new declaration or a changed literal signature, body, documentation, attribute, registration or scope against the exact frozen original before bytes. The three existing shared files have real before copies. The three new files have absent-before entries. An unchanged legacy whole-file inventory does not establish selected completion.

| Literal declaration dimension | Selected documented/total | Whole documented/total | Whole omissions |
| --- | ---: | ---: | ---: |
| Functions | 87/87 | 115/272 | 157 |
| Types | 18/18 | 63/81 | 18 |
| Members | 83/83 | 478/682 | 204 |
| Modules | 4/4 | 10/21 | 11 |

The selected named-function cohort contains 84 new declarations and 3 existing hooks: CLI `execute`, `validate_oscal_json_value` and `export_json_value`. The two export helpers changed visibility; their bodies remain exact. The selected function scope labels are 33 production-lexical, 30 cfg-test and 24 integration-test declarations. These include 30 literal test registrations. Final integration helpers `fixture_evidence_rows` and `assert_complete_source_provenance` are included, rather than inheriting the earlier 85-function denominator.

The text lexer does not perform Rust cfg/type/macro expansion. It retains 19 selected and 36 whole-function cfg qualifications. Platform and compound attributes remain explicit. Outer module declarations form the module denominator; inner file Rustdoc occurrences remain separate. Documentation counts do not independently approve historical authored primitives, numeric policy or authored controls.

## Actual final native observations

Root's closed full-workspace, all-feature LLVM V2 job passed 3,295 tests with 0 failures and 3 ignored across 71 result summaries. Source bytes remained unchanged during the command, and all six before/after job pins match this exact final source generation. Strict all-feature/all-target Clippy V4 also passed on the same source. No compiler, product, control or native execution was performed by this metadata reader.

| Exact named header observation | Positive | Zero | Unmapped | Exact emitted aliases | Zero aliases |
| --- | ---: | ---: | ---: | ---: | ---: |
| Selected 87 | 85 | 1 | 1 | 134 | 6 |
| Whole six files 272 | 256 | 5 | 11 | 388 | 42 |

Association requires the exact original source filename, literal visibility/modifier header line and one-based UTF-8 column, and the first regular `CodeRegion`. It uses no nearest region, generated closure, name-only match or fn-keyword substitution. Every alias is retained, including zero aliases beneath an otherwise positive declaration. Selected aliases are 128 positive and 6 zero; whole aliases are 346 positive and 42 zero. A positive header does not establish full-body, line, branch or platform coverage.

The selected zero is `LimitedWriter::flush` at `src/linkage/overlay.rs:941`, with exact alias entry counts [0, 0]. The selected unmapped declaration is `overlay_unsupported_platform_does_not_publish_or_emit_stdout` at `tests/linkage_overlay_test.rs:699`; its unsupported-platform cfg is absent from this macOS execution. All 5 whole zero and 11 whole unmapped declarations remain in the audit, including legacy and platform-specific occurrences.

The raw export contains 19,995 function records and 6,610 zero records. Its 399 scoped regular records that are not literal named headers remain separate; 0 raw records lack a regular region. These raw/compiler denominators differ from named source declarations.

| Whole-export metric | Covered/total |
| --- | ---: |
| Lines | 84,166/92,052 (91.433103029%) |
| Functions | 7,501/8,937 (83.931968222%) |
| Regions | 147,580/162,615 |
| Instantiations | 10,626/16,005 |

Branch and MC/DC denominators are both zero and therefore unavailable. Whole-export totals include production, tests and generated code; they are not selected-production-only percentages or fresh Windows execution.

## Exact measured source

| Path | Bytes | SHA-256 |
| --- | ---: | --- |
| `src/cli/mod.rs` | 117,582 | `3d8928a8dac23db0e37ae0240038f93e807b5183184d40f99ea7719c34605ab8` |
| `src/cli/export.rs` | 46,310 | `10d3fe062d2f87d88fd5a62409b33ab4c7454feb83f0b35286b1d71be99c5b97` |
| `src/linkage/mod.rs` | 93,031 | `9b8945c71fd309a1efa47cb3eab64c2b9c135f533406862293a803d7bae85e2d` |
| `src/linkage/overlay.rs` | 69,428 | `8f6b94e1913cfc83a35db50a0357262c5e5f26576c3101ead7ad20de40bbf9d7` |
| `src/linkage/overlay_cli.rs` | 1,874 | `15bf2334859cc00a9e21b9ef8aa2bb819807e7e0ae08dcc863e271435d8ec58c` |
| `tests/linkage_overlay_test.rs` | 33,763 | `b9415232c43435d52a5a5c67e57dca0badf01eb637b4ba7880af463fa9bf5e39` |

The raw LLVM V2 export is `eac1e2da380c406b5b7ca4cb56d3f7de5394b33c1df191367e642e73279ac7f8` (22,836,343 bytes). The machine audit pins the exact raw export, full log, closed job, strict Clippy job/log, original source capture and reviewer tools. The textual lexer identity is `c22358f90295420d6fdd5418890031855c0fc595e1a76ce47c82140493858118`. Complete literal offsets, separate fn-keyword positions, bodies/signature hashes and occurrence qualifications are retained.

## Preserved history and remaining gates

The producer V1 review predicted two control defects without executing them: a JSON-row fixture exceeded its projection ceiling by one byte, and held captured owners prevented the intended Windows deletion. Separate source successors adjusted the fixture to `MAX_RECORD - 3` and dropped actual owners after copied assertions, before deletion. The final source also preserves narrow doc/numeric-separator lint fixes, two documented integration-helper extractions and equivalent semicolon additions. V1 producer/review sources, pre-lint LLVM V1 and all lint/control predecessors remain byte-exact historical evidence; their observations are not relabelled as final source.

This slice preserves the export-only integer-represented JSON prerequisite. Decimal/exponent f64 representations, including harmless `1.0` or `1e0` spellings, remain refused. Decoded-value equality does not establish arbitrary numeric losslessness. Original OSCAL values can retain sensitive prose, embedded content and metadata; generated-field minimization does not promise blanket redaction.

Full F09 model/numeric/interoperability, consumer and owner/human acceptance remain open. This audit provides no current native Windows evidence, F04 OS-denial qualification, actor authority, release or goal-wide acceptance. Final required enabled commit-hook and hosted measurements are separate evidence not yet represented by this local LLVM/Clippy record. No future hook or hosted result is inferred.
