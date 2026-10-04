# Local POA&M evidence inspection verification

This checkpoint binds the local evidence-inspection slice to final source V4,
lexical/native collector V3, the closed whole-workspace LLVM V2 run and the
normal enabled hook V1. Earlier source and failed or incomplete checkpoints
remain separate. These observations do not close product or human acceptance.

The [inspection guide](poam-evidence-inspection.md) explains the command and its
input and output boundaries. The slice inspects every declared closure-evidence
assertion against explicit bindings and actual current local originals. A
valid report does not authenticate evidence, approve remediation, admit a
native terminal transition or resolve D064.

## Exact source scope

The comparison basis is `0006cd28c5f1bed9ebdd80f443dacd7014f38335`. The final
source capture is `/private/tmp/forge-f08-s4-final-source-v4/index.json (SHA-256 43b3d9ee1cef53ef5b88d462d46e85334e1d1b41a39d28b49dc7e636be68c8eb; 294,758 bytes)`; the lexical reader is
`/private/tmp/forge-f08-fresh-evidence-s4-final-census-peer-v3/index.json (SHA-256 4fb5128f5283afac0d21486fb80655b361579d321ea2a05295a2b21e538f4c47; 18,895 bytes)`; the machine verification record is
`docs/plans/2026-10-04-f08-evidence-inspection-integration-verification.json`. All 13 final Rust files match the authentic LLVM, final format, all-feature
Clippy and enabled-hook before/after source pins. The machine record has
SHA-256 `8888aec93d44acf31c0111add742a9862f4b93b249bc819a7f359b771738912d` and is 7,592,748 bytes.
Its own pin is bound here after publication; it is not a self-hash inside that record.

The explicit Rust cohort contains these 13 files:

| File | Scope |
| --- | --- |
| `src/evidence_capture.rs` | Shared capture leases, proofs and before-growth budgets |
| `src/linkage/fresh.rs` | Confined local originals, absence/directory proofs and complete fresh linkage |
| `src/mapping/inventory.rs` | Borrowed captured native inventory consumer |
| `src/poam/source.rs` | Captured source consumer; existing source loader remains separate |
| `src/poam/workflow_evidence.rs` | Closed binding parser and complete inspection report |
| `src/assessment_results/context.rs` | Documented forwarding port; existing loader bodies preserved |
| `src/assessment_results/context_captured.rs` | Private sealed-byte context consumer and cfg controls |
| `src/cli/mod.rs` | Command arguments and dispatch |
| `src/lib.rs` | Private module wiring |
| `src/linkage/mod.rs` | Fresh linkage module wiring |
| `src/poam/mod.rs` | Inspection/CLI module wiring |
| `src/poam/workflow_evidence_cli.rs` | Input and output preflight, rechecks and publication |
| `tests/poam_foundation_test.rs` | Additive real native/input/CLI integration controls |

These whole-file counts cover this explicit cohort, not the repository. The
selected cohort comprises literal new or changed declarations against the
frozen basis, including registration, attributes, signatures, bodies, docs and
qualified scope changes. Production, cfg-test and integration-test occurrences
remain separate; anonymous/generated compiler records do not become named
source declarations.

## Documentation inventory

| Dimension | Selected documented / total | Whole documented / total | Final omissions |
| --- | --- | --- | --- |
| Named functions and methods | `212 / 212` | `400 / 589` | `0 selected; 189 whole omission locators retained in the machine record` |
| Types | `44 / 44` | `93 / 121` | `0 selected; 28 whole omission locators retained in the machine record` |
| Fields, variants and tuple payload members | `200 / 213` | `603 / 876` | `13 selected tuple payloads; 273 whole omission locators retained` |
| Literal module declarations | `9 / 10` | `60 / 81` | `1 selected outer declaration; 21 whole omission locators retained` |

The machine census retains every declaration position, qualified name,
source pin, cfg classification, before/after hash and omission locator. File or
module inner `//!` docs are counted separately from an outer declaration's
adjacent docs. A documented type or variant does not silently document each
tuple payload. The text lexer is not Rust type checking, cfg expansion or macro
expansion; ambiguous occurrences stay qualified in the data.

The preserved predecessor recorded 211/211 selected function docs, 44/44 types,
200/213 members and 9/10 outer module declarations. Its 13 member omissions are
new tuple payloads in `LocalObservation`, `CapturedLocal`, `OpenedLocal` and
`SourceRefs`. The new CLI `tests` module has an adjacent inner doc, retained
separately from its missing outer doc. These predecessor counts and omissions
stay historical. The final table is the fresh exact recount: 211 new
functions plus one changed existing dispatch, `crate::cli::execute`. No selected
function or type doc is missing; the 13 new tuple payload doc gaps and one outer
module doc gap remain open.

| File and header line | Selected member without adjacent Rustdoc |
| --- | --- |
| `src/evidence_capture.rs:194` | `evidence_capture::LocalObservation::Present::0` |
| `src/evidence_capture.rs:196` | `evidence_capture::LocalObservation::Absent::0` |
| `src/linkage/fresh.rs:106` | `linkage::fresh::CapturedLocal::Present::0` |
| `src/linkage/fresh.rs:106` | `linkage::fresh::CapturedLocal::Present::1` |
| `src/linkage/fresh.rs:108` | `linkage::fresh::CapturedLocal::Absent::0` |
| `src/linkage/fresh.rs:114` | `linkage::fresh::OpenedLocal::Present::0` |
| `src/linkage/fresh.rs:114` | `linkage::fresh::OpenedLocal::Present::1` |
| `src/linkage/fresh.rs:114` | `linkage::fresh::OpenedLocal::Present::2` |
| `src/linkage/fresh.rs:114` | `linkage::fresh::OpenedLocal::Present::3` |
| `src/linkage/fresh.rs:116` | `linkage::fresh::OpenedLocal::Absent::0` |
| `src/linkage/fresh.rs:116` | `linkage::fresh::OpenedLocal::Absent::1` |
| `src/linkage/fresh.rs:116` | `linkage::fresh::OpenedLocal::Absent::2` |
| `src/poam/workflow_evidence.rs:124` | `poam::workflow_evidence::SourceRefs::0` |

The selected outer-module omission is `src/poam/workflow_evidence_cli.rs:123`
(`tests`); its separate inner `//!` comment is at line 124.

The function scope counts are 144 production-lexical, 52 literal cfg-test
and 16 integration-test occurrences. The 144 lexical label includes 13
`#[cfg(all(test, unix))]` ambiguities in the captured-context test module:
nine registered controls and four helpers. They are qualified test occurrences,
not an assertion of 144 production functions. All 13 rows and their literal
attributes are retained in the machine record; 52 whole-file cfg ambiguities
remain qualified separately. The lexer also retains 64 inner-doc occurrences.

## Observed entries and coverage limits

Final raw LLVM: `/private/tmp/forge-f08-s4-final-integration-llvm-whole-v2.json`, SHA-256
`8fb908858498efa04e03f8ba7459d4c398192da2fc146a52cc9f95b4d236d30a`, `22,415,934` bytes. Final job:
`/private/tmp/forge-f19-browser-f08-s4-final-integration-llvm-whole-v2.json (SHA-256 589229bc2f0ba6faa2a39bea9e826c3912d8d3557d576282f3d33679b3624ca7; 61,588 bytes)`; final log: `/private/tmp/forge-f19-browser-f08-s4-final-integration-llvm-whole-v2.log (SHA-256 7f1e9f9efe9e8d490db37ba8cfa373f666047cf52c130ea9382e7fb0f9bc5140; 281,640 bytes)`. The exact 13
source pins match that job. Coverage uses its explicit dedicated
`CARGO_LLVM_COV_TARGET_DIR`; the recorder's ambient `cargo_target_dir` field
does not select the LLVM target directory.

| Named entry cohort | Positive | Zero | Unmapped | All aliases / zero aliases |
| --- | --- | --- | --- | --- |
| Selected declarations | `203` | `2` | `7` | `403 / 29` |
| Whole 13-file declarations | `564` | `8` | `17` | `968 / 101` |

Association uses the exact original source filepath and the first regular
CodeRegion at the declaration header's one-based line and UTF-8 byte column.
Every exact named alias and scoped nonheader first-regular record, plus all
global raw-zero indices, remains in the machine data; the complete raw LLVM
export is separately pinned. No nearest closure or adjacent function substitutes for an absent
header. A positive named row means at least one exact alias had a positive entry
count; it does not make its zero aliases positive or prove every body path ran.

Whole LLVM totals remain a separate compiler denominator: lines
`82,925 / 90,785`, functions
`7,397 / 8,794`, regions
`145,234 / 160,207`, raw function records
`19,535`, including `6,408` zeros.
Entry observation, line/region coverage and complete body coverage are different
claims. Branch and MC/DC coverage remain unavailable unless an authentic
nonzero denominator is supplied; this slice makes no such claim.

The earlier 3258-test LLVM-V1 checkpoint retained an unentered captured-context
`plan_tasks` row, two unentered writer `flush` rows and platform-specific unmapped
rows. The genuine nested-task/duplicate-task control passed in its own one-test
run and is present in final LLVM V2. `context_captured::plan_tasks` at line 560
now has exact alias entry counts `[0, 6]`: its named row is positive, while its
zero alias remains zero. The older raw export and associations stay immutable.
The following final selected gaps remain; they have not been replaced by nearby
closures or another platform's function.

| Final selected gap | Header line | Qualification |
| --- | --- | --- |
| `evidence_capture::impl io : : Write for CountingWriter::flush` | 521 | Zero entry; all exact aliases zero |
| `evidence_capture::impl io : : Write for BoundedWriter::flush` | 545 | Zero entry; all exact aliases zero |
| `linkage::fresh::directory_identity` | 328 | Unmapped; `#[cfg(windows)]` |
| `linkage::fresh::directory_identity` | 340 | Unmapped; `#[cfg(not(any(unix, windows)))]` |
| `linkage::fresh::open_windows` | 438 | Unmapped; `#[cfg(windows)]` |
| `linkage::fresh::open_root` | 449 | Unmapped; `#[cfg(windows)]` |
| `linkage::fresh::open_local` | 484 | Unmapped; `#[cfg(windows)]` |
| `linkage::fresh::open_root` | 531 | Unmapped; `#[cfg(not(any(unix, windows)))]` |
| `linkage::fresh::open_local` | 537 | Unmapped; `#[cfg(not(any(unix, windows)))]` |

The whole cohort totals eight zero and 17 unmapped named rows, including the selected gaps.
All 968 exact aliases remain: 867 positive and 101 zero. The complete export
retains 1,102 scoped non-header compiler records and no records lacking a
regular CodeRegion. Raw zero/generated records are not removed from the
denominator or counted as named declarations. The raw line proportion is
91.3421820785372%; it is not selected-function body coverage.

## Authentic run history

The following closed local observations are retained with their own source
generations. They are not extra unique test credits and are not transferred to
an unmeasured successor:

| Retained job | Actual result |
| --- | --- |
| Initial library run, `core-library-native-v1` | Exit 101; 2242 passed, 5 failed, 0 ignored. This failed checkpoint earns no pass. |
| Corrected library run, `core-library-native-v2` | Exit 0; 2247 passed, 0 failed, 0 ignored. |
| Native/input/CLI integration, `native-integration-controls-v1` | Exit 0; all 7 controls passed. |
| Strict all-target/all-feature Clippy, `core-final-strict-clippy-v4` | Exit 0 with source bytes unchanged during that command. |
| Format check, `core-final-format-check-v2` | Exit 0 with source bytes unchanged during that command. |
| Preserved whole LLVM-V1 run | Exit 0; 3258 passed, 0 failed, 3 existing ignored, 70 summaries. Its raw source-V2 generation stays historical. |
| Final whole LLVM successor | `Exit 0; 3259 passed, 0 failed, 3 existing ignored, 70 summaries` with exact final before/after source equality. |
| Mandatory enabled hook V1 | Exit 0; 3283 passed, 0 failed, 3 existing ignored, 71 summaries, including 24 doctests; all 13 Rust before/after pins match final source V4. |
| Final all-target/all-feature Clippy, `core-final-strict-clippy-v6` | Exit 0; final source V4 unchanged. |
| Final format check, `native-task-control-format-check-v2` | Exit 0; final source V4 unchanged. |

The final enabled hook used `SKIP_FORGE_PRECOMMIT=0` and normal
`FORGE_PRECOMMIT_STRICT=0`; format, required strict lint and full tests passed.
It observed 18 staged owned paths, including the practical guide and navigation
paragraphs. This new verification guide and the subsequently published machine
record were not those 18 staged inputs; final commit/draft delivery and any
later hook are separate observations. Its exact receipt is
`/private/tmp/forge-f19-browser-f08-s4-final-enabled-hook-v1.json (SHA-256 e0d7ca1e4b99b24f8d61ad9863284681c534c0beab1a38ef3967454017da7e52; 62,011 bytes)`; its log is
`/private/tmp/forge-f19-browser-f08-s4-final-enabled-hook-v1.log (SHA-256 e9a2c136775665ab9bdf949a45a2b34693a7254fb9513b82ce1dff0649cf9ffb; 275,213 bytes)`.

Final all-feature Clippy receipt: `/private/tmp/forge-f19-browser-f08-s4-core-final-strict-clippy-v6.json (SHA-256 d245c48da1e4e18418b3c0d1b26bffdc9a41b1b2a13f5bb3053c500d89aede8f; 61,441 bytes)`.
Final format receipt: `/private/tmp/forge-f19-browser-f08-s4-native-task-control-format-check-v2.json (SHA-256 cf33745dea7a0843dc062fd1ba674f0b5795767f1280d67b756b217450e591f3; 61,376 bytes)`.

Raw failures, initial fixture/reader/lint diagnostics, repair deltas and earlier
exports remain preserved. Later success does not rewrite them. Static shape
examples and authored controls receive no executed credit. The seven native
integration controls exercise actual locally built artifact/source/evidence
files and public CLI boundaries; they do not establish remote authority or
general evidence sufficiency.

The final source and result guards must be refreshed if any listed Rust bytes
change. Later navigation/prose edits are separately pinned and do not create
new runtime observations. A new enabled hook is measured separately from an old
LLVM run; historical sources are never relabeled as current.

## Remaining gates

Local macOS development observations do not qualify Linux or Windows S4
execution; those S4 native campaigns have not run in this checkpoint. No
assistive-technology, participant, owner or human evidence acceptance is
inferred. D064 and native terminal/evidence admission remain open, with
`terminal_admitted: false` and `artifact_validated: false` in inspection reports.

This slice does not close all F08 Must/Should requirements, the complete goal,
release qualification or the final integrated documentation gate. The
borrowed-context and historical source/control authorship overlaps are
explicitly excluded from independent correctness approval. Root owns actual
source integration, runtime verification, the enabled hook and draft delivery.
