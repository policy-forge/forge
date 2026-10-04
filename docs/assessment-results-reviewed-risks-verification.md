# Reviewed-risk authoring integration verification

This record covers the candidate `assessment results export-poam` command and its
maintained POA&M consumers. The final working source was measured at comparison
and measurement head `63db6029e68d93d67ddf3b120157b58111a9121f`; the head alone
does not identify the uncommitted changes. Exact source hashes below bind this
generation. This is engineering evidence for a bounded S-3 slice. Full F10,
independent interoperability, owner and human acceptance remain open.

The [machine audit](plans/2026-10-04-f10-risk-authoring-integration-verification.json)
retains complete before/after lexical declarations, omission and cfg locators,
exact emitted aliases, all zero and unmapped observations, raw totals and actual
job/source bindings. Historical proposals and failed lint jobs remain separate
from the final source. The audit omits its own resulting file hash; a separate
TEMP publication index pins its bytes.

## Documentation scope

The census covers seven explicit Rust paths. All four existing predecessors were
compared with actual `HEAD:path` bytes; four new paths, including the request
schema, were verified absent at that head. Selected functions are new or have a
changed literal signature, body, documentation, attribute, registration or scope.
Selected types, members and modules are new declarations; existing enclosing
containers remain in the whole inventory with their comparison metadata.

| Literal declaration dimension | Selected documented/total | Whole documented/total | Whole omissions |
| --- | ---: | ---: | ---: |
| Functions | 104/104 | 238/290 | 52 |
| Types | 6/6 | 77/77 | 0 |
| Members | 26/26 | 533/618 | 85 |
| Modules | 3/3 | 24/33 | 9 |

The selected cohort contains 103 new named functions and one changed existing
function, CLI `execute`. Its scope labels are 36 production-lexical, 22 cfg-test
and 46 integration-test declarations. There are 44 literal test registrations:
18 producer tests and 26 CLI tests, including one Windows-only test. The actual
macOS campaign executed the 18 producer and 25 applicable CLI tests.

The lexer reads Rust text; it does not resolve compiler cfg, types or macro
expansion. All 13 whole-function cfg ambiguities and 23 inner Rustdoc occurrences
are retained. Outer `mod` declarations form the module denominator. There are no
removed declarations, raw-only changes or macro-literal declarations in this
comparison. Documentation completeness does not establish source correctness.

## Actual final tests and native observations

The final full-workspace, all-feature LLVM V1 job passed 3,338 tests with zero
failures and three ignored tests across 72 result summaries. All eight source
pins matched before and after the command, and the working bytes stayed
unchanged. Formatting V3 and strict all-target/all-feature Clippy V3 also passed
on this exact source generation.

The new controls exercise actual init scaffolds and captured native originals,
then export into maintained `poam check --workflow` and `poam build`. They verify
official native-schema validation, complete schedule counts, original source
links and hashes, ordered values and nulls, explicit authorship, closed-schema
and duplicate-key refusals, stale selections, source changes between commands,
bounded growth and no-replace publication. The two planned work items produce
four schedule rows, two overdue and two due soon, while retaining all five
selected-result source objects. Export exits zero; the valid action-required
check/build operations retain their own exit one.

| Exact named header observation | Positive | Zero | Unmapped | Exact aliases | Zero aliases |
| --- | ---: | ---: | ---: | ---: | ---: |
| Selected 104 | 101 | 2 | 1 | 139 | 16 |
| Whole seven files 290 | 285 | 4 | 1 | 470 | 41 |

Association uses the exact original filename, literal visibility/modifier header
line and one-based UTF-8 column, and the first regular `CodeRegion`. It uses no
nearest region, name-only match, generated closure or fn-keyword substitution.
All aliases remain present, including zero aliases beneath positive declarations.
Selected aliases are 123 positive and 16 zero; whole aliases are 429 positive and
41 zero. A positive header establishes entry, not full-body or branch coverage.

The selected zeros are `CountingWriter::flush` at
`src/poam/risk_authoring.rs:488` and `BoundedWriter::flush` at line 528, each with
two exact zero aliases. The selected unmapped declaration is
`windows_keeps_supported_stdout_and_refuses_file_publication` at
`tests/risk_authoring_cli_test.rs:1269`, absent from this macOS build under
`cfg(windows)`. Two additional whole-file zero declarations are the existing
capture writers' `flush` methods. Every omission remains in the audit.

The raw export contains 20,258 function records, including 6,682 zero records.
Its 567 scoped regular records that are not literal named headers remain
separate; zero raw records lack a regular region.

| Whole-export metric | Covered/total |
| --- | ---: |
| Lines | 84,946/92,858 (91.479463266%) |
| Functions | 7,596/9,051 (83.924428240%) |
| Regions | 149,174/164,307 |
| Instantiations | 10,759/16,210 |

Branch and MC/DC denominators are zero and therefore unavailable. Whole-export
totals include production, test and generated code; they are not percentages
for selected production code alone or evidence of a Windows run.

## Exact measured source

| Path | Bytes | SHA-256 |
| --- | ---: | --- |
| `src/cli/mod.rs` | 118,535 | `a03cba3a8833b49e073a6a52ce1e6293df0dc452ab734c58bf487bae15d83a95` |
| `src/evidence_capture.rs` | 37,174 | `cf4723f68d72869385e4ebe4667b954e95a29963ce096a289bfa78cf4196d859` |
| `src/poam/mod.rs` | 14,289 | `7ea1820da97a329f6f49cc1b081ba9ebdffa31e39140ae93a925da69063f3f3a` |
| `src/poam/workflow.rs` | 64,125 | `27471b72c055e16aab65c18218a6d952fade128ca5f3c312c9092b4488406cd3` |
| `src/poam/risk_authoring.rs` | 44,744 | `ae854e38a659706e3a91cc89d18c1b16f94a15ffc128263e2aea7f5880006a83` |
| `src/poam/risk_authoring_cli.rs` | 3,077 | `ee6d85df1727e7718f164c084d095604eddc70460ae3d083d9bcda3cd335bd06` |
| `tests/risk_authoring_cli_test.rs` | 54,279 | `e4179e010566ad1288a3713445be46da053c2c9152d0a6cf0cefa0f578e7296a` |
| `schemas/forge.poam-risk-authoring-1.schema.json` | 13,193 | `c441441e589c42a9a777f58b959d594416fdfc72ece9829aa5fda68c8414bd25` |

Raw LLVM V1 is `d84c1fb672a8fedf1d9ffe0d9764b01f78f5acc86680289cf156217993067f97`
(23,117,219 bytes). The lexer is
`c22358f90295420d6fdd5418890031855c0fc595e1a76ce47c82140493858118`.
The machine audit also pins the complete log, closed job, formatter and strict
Clippy evidence, actual original source capture and reviewed tooling.

## Preserved history and remaining gates

Producer V1 lacked the required final decoded-value equality check and two
controls did not isolate their stated predicates. V2 supplied the final strict
encoding oracle and corrected the destination/duplicate-key controls. V3 added
existing-destination metadata preflight before original capture. Independent
source review of those new deltas found no further material issue; it excludes
the reviewers' own historical source and controls from independent approval.

Clippy V1 found test-code boolean, borrowing and empty-assertion style issues.
The first correction exposed an empty `Value` slice type-inference error in
Clippy V2. The final typed assertion and equivalent test-only corrections passed
Clippy V3 and the fresh whole-workspace LLVM campaign. Earlier 18/25 passing
test receipts retain their earlier source identities and are not relabelled as
final-source evidence. Historical packets remain byte-exact.

Seven originals are held by one actual capture session and rechecked immediately
before output. This slice has no deterministic test hook that mutates all seven
within preparation; generic capture tests do not supply that missing canary.
Destination preflight admits missing metadata without claiming atomic absence;
the maintained no-replace publisher handles later creators. The shared projection
ceiling is a conservative encoded-size policy, not an allocator or heap proof.

Full S-3 acceptance, S-4 epochs and recurring-risk semantics, downstream
interoperability, owner/assessor/pilot judgments, D064 terminal disposition and
dependency/runtime qualification remain open. The final goal-wide documentation
review and verified 2.0.0 release candidate are separate required work. The
enabled commit hook and exact-head hosted checks are later receipts, not results
inferred from this pre-PR measurement.
