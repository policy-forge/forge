# Tasks: Traceability Report (WI-38)

**Input**: Design documents from `/specs/038-traceability-report/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/trace_interfaces.rs

**Tests**: Included — Constitution Principle IV mandates test-first development.

**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root (Rust crate)

---

## Phase 1: Setup

**Purpose**: Module structure and shared infrastructure

- [X] T001 Create trace module directory and root module in `src/trace/mod.rs` with submodule declarations (`pub mod report; pub mod extractor; pub mod walker; pub mod resolver; pub mod formatter;`)
- [X] T002 Register trace module in `src/lib.rs` — add `pub mod trace;`
- [X] T003 Add `TraceUnsupportedArtifact { detail: String }` variant to `ForgeError` in `src/error.rs` and map it to exit code 2 in `exit_code()`
- [X] T004 Verify setup compiles: `cargo build`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Data structures and extractor that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### Tests

- [X] T005 [P] Write unit tests for `TraceMetadata`, `TraceEntry`, `TraceSummary` construction and `TraceSummary` coverage calculation (0%, partial, 100%, empty) in `src/trace/report.rs` `#[cfg(test)]` module
- [X] T006 [P] Write unit tests for `extract_trace_metadata()` in `src/trace/extractor.rs` `#[cfg(test)]` module — cases: control with all 3 trace props → `Some(TraceMetadata)`, element with no props → `None`, element with only `source-section` (group) → `Some` with `source_line == 0`, element with non-trace props only → `None`, element with unparseable `source-line` value → `None`
- [X] T007 [P] Write unit tests for `strip_control_chars()` in `src/trace/extractor.rs` `#[cfg(test)]` module — cases: string with ANSI escape → stripped, string with tab/newline → preserved, clean string → unchanged, empty string → empty

### Implementation

- [X] T008 [P] Implement `TraceMetadata`, `TraceEntry`, `TraceSummary`, `TraceReport` structs in `src/trace/report.rs` per contracts/trace_interfaces.rs. Include `TraceSummary::from_entries(entries: &[TraceEntry]) -> TraceSummary` constructor.
- [X] T009 [P] Implement `strip_control_chars()` in `src/trace/extractor.rs` — filter bytes 0x00-0x1F excluding 0x0A and 0x09 (SEC-5, FR-012)
- [X] T010 Implement `extract_trace_metadata()` in `src/trace/extractor.rs` — scan `element["props"]` array for props with `ns == FORGE_TRACE_NS`, extract `source-file`, `source-section`, `source-line`. Import constants from `crate::oscal::trace_embedding`. Return `None` if no `source-section` found.
- [X] T011 Verify all foundational tests pass: `cargo test trace`

**Checkpoint**: Data structures and extractor ready. User story implementation can now begin.

---

## Phase 3: User Story 1 — Produce Traceability Report (Priority: P1) 🎯 MVP

**Goal**: `forge trace catalog.json --source policy.md` produces a column-aligned text table mapping each OSCAL element to its source location.

**Independent Test**: Generate an OSCAL Catalog from a sample policy with trace metadata, run `forge trace`, verify structured table with one row per element showing ID, type, section, line.

**Requirements**: M-1, M-2, M-3, M-4, M-5, FR-001 through FR-005, FR-009, FR-010, FR-012, SEC-2, SEC-3, SEC-4

### Tests for User Story 1

- [X] T012 [P] [US1] Write unit tests for `detect_artifact_type()` in `src/trace/walker.rs` `#[cfg(test)]` module — cases: JSON with `"catalog"` key → `ArtifactType::Catalog`, JSON with `"component-definition"` key → `ArtifactType::ComponentDefinition`, JSON with neither → `Err(TraceUnsupportedArtifact)`
- [X] T013 [P] [US1] Write unit tests for `walk_catalog_elements()` in `src/trace/walker.rs` `#[cfg(test)]` module — cases: catalog with 2 groups and 3 controls → 5 entries, groups have element_type "group" with section but no line, controls have element_type "control" with full metadata, empty catalog → 0 entries
- [X] T014 [P] [US1] Write unit tests for `walk_compdef_elements()` in `src/trace/walker.rs` `#[cfg(test)]` module — cases: compdef with 1 component and 3 implemented-requirements → 3 entries with element_type "implemented-requirement", empty components → 0 entries
- [X] T015 [P] [US1] Write unit tests for `format_trace_table()` in `src/trace/formatter.rs` `#[cfg(test)]` module — use `insta` snapshot tests for: report with 3 mapped entries (verify column alignment), report with unmapped entries (verify `[unmapped]` markers), report with group entries (verify `—` for Source Line), empty report
- [X] T016 [P] [US1] Write unit test for `ForgeError::TraceUnsupportedArtifact` display message and exit code in `src/error.rs` `#[cfg(test)]` module

### Implementation for User Story 1

- [X] T017 [P] [US1] Implement `detect_artifact_type()` in `src/trace/walker.rs` — check top-level JSON keys `"catalog"` and `"component-definition"`, return `Err(ForgeError::TraceUnsupportedArtifact)` if neither found
- [X] T018 [P] [US1] Implement `walk_catalog_elements()` in `src/trace/walker.rs` — walk `catalog.groups[]` → yield group entries (id from `group.id`, type "group"), then `group.controls[]` → yield control entries (id from `control.id`, type "control"). Call `extract_trace_metadata()` for each element. Skip parts.
- [X] T019 [P] [US1] Implement `walk_compdef_elements()` in `src/trace/walker.rs` — walk `component-definition.components[].control-implementations[].implemented-requirements[]` → yield entries (id from `control-id`, type "implemented-requirement"). Call `extract_trace_metadata()` for each.
- [X] T020 [US1] Implement `format_trace_table()` in `src/trace/formatter.rs` — two-pass approach: first pass computes max column widths across all entries, second pass renders header + separator + padded rows + summary footer. Apply `strip_control_chars()` to source-derived strings. Show `[unmapped]` for unmapped entries, `—` for groups with no line number.
- [X] T021 [US1] Implement `generate_trace_report()` in `src/trace/mod.rs` — orchestrate: validate files exist (return `ForgeError::FileNotFound`), read artifact JSON (return `ForgeError::Parse` on invalid JSON), detect type, walk elements, compute summary, return `TraceReport`. Do NOT check staleness yet (deferred to US4).
- [X] T022 [US1] Implement CLI handler `execute()` in `src/cli/trace.rs` — call `generate_trace_report()`, call `format_trace_table()`, print to stdout. Handle errors.
- [X] T023 [US1] Add `Trace` variant to `Commands` enum in `src/cli/mod.rs` with fields: `artifact: PathBuf` (positional), `#[arg(long)] source: PathBuf` (required), `#[arg(long)] output: Option<PathBuf>`. Add dispatch to `execute()` function.
- [X] T024 [US1] Create test fixture: minimal OSCAL Catalog JSON with trace metadata (2 groups, 4 controls, WI-17 props) in `tests/fixtures/catalog-with-trace.json` and corresponding source policy `tests/fixtures/trace-sample-policy.md`
- [X] T025 [US1] Create test fixture: minimal OSCAL Component Definition JSON with trace metadata (1 component, 3 implemented-requirements) in `tests/fixtures/compdef-with-trace.json`
- [X] T026 [US1] Write integration test in `tests/trace_report_test.rs` — test `generate_trace_report()` with catalog fixture, verify entry count, mapped status, element IDs, and element types
- [X] T027 [US1] Write integration test in `tests/trace_report_test.rs` — test `generate_trace_report()` with compdef fixture, verify implemented-requirement entries
- [X] T028 [US1] Write integration test for error cases in `tests/trace_report_test.rs` — missing artifact file → `FileNotFound`, invalid JSON → `Parse`, unsupported type → `TraceUnsupportedArtifact`
- [X] T029 [US1] Run quality gates: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`

**Checkpoint**: `forge trace` produces correct table output for Catalog and Component Definition artifacts. Core MVP is functional.

---

## Phase 4: User Story 2 — Save Traceability Report to File (Priority: P2)

**Goal**: `forge trace catalog.json --source policy.md --output trace-report.txt` writes the report to a file.

**Independent Test**: Run with `--output` flag, verify file created with same content as stdout output.

**Requirements**: S-1, FR-007

### Tests for User Story 2

- [X] T030 [P] [US2] Write integration test in `tests/trace_report_test.rs` — run with `--output` to a tempfile, verify file content matches `format_trace_table()` output
- [X] T031 [P] [US2] Write integration test in `tests/trace_report_test.rs` — run without `--output`, verify output goes to returned string (stdout path)

### Implementation for User Story 2

- [X] T032 [US2] Update `execute()` in `src/cli/trace.rs` — if `output` is `Some(path)`, write formatted table to file using `std::fs::write()` instead of printing to stdout. Report success message to stderr when writing to file.
- [X] T033 [US2] Run quality gates: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`

**Checkpoint**: File output works. Stories 1 and 2 are independently testable.

---

## Phase 5: User Story 3 — Verify Complete Coverage (Priority: P2)

**Goal**: Report flags unmapped elements and includes a coverage summary with total/mapped/unmapped/percentage.

**Independent Test**: Generate report from artifact where one control lacks trace metadata, verify "unmapped" flag and summary showing <100% coverage.

**Requirements**: M-6, S-2, FR-006

### Tests for User Story 3

- [X] T034 [P] [US3] Write unit tests for coverage summary display in `src/trace/formatter.rs` `#[cfg(test)]` module — verify summary line format: `"Summary: N elements, M mapped, U unmapped (X.X% coverage)"` for 100%, partial, and 0% cases
- [X] T035 [P] [US3] Create test fixture: OSCAL Catalog JSON where 1 control has no trace props in `tests/fixtures/catalog-partial-trace.json`
- [X] T036 [P] [US3] Write integration test in `tests/trace_report_test.rs` — generate report from partial-trace fixture, verify unmapped control shows `[unmapped]`, summary shows correct counts and <100% coverage

### Implementation for User Story 3

- [X] T037 [US3] Verify that `format_trace_table()` in `src/trace/formatter.rs` already renders `[unmapped]` for entries with `trace: None` and appends summary line — this was built in Phase 3 (T020). If not complete, finish unmapped rendering and summary footer now.
- [X] T038 [US3] Create test fixture: OSCAL Catalog JSON with NO trace metadata on any element in `tests/fixtures/catalog-no-trace.json`
- [X] T039 [US3] Write integration test in `tests/trace_report_test.rs` — generate report from no-trace fixture, verify ALL elements are unmapped, coverage is 0%, warning about absence of trace data is present
- [X] T040 [US3] Run quality gates: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`

**Checkpoint**: Coverage gaps are visible. Unmapped elements are flagged. Summary statistics are correct.

---

## Phase 6: User Story 4 — Source Integrity Warning (Priority: P3)

**Goal**: Warn when source policy file appears modified since OSCAL artifact generation.

**Independent Test**: Modify source policy after generating OSCAL artifact, run `forge trace`, verify staleness warning.

**Requirements**: S-3, FR-008, FR-011, SEC-7

### Tests for User Story 4

- [X] T041 [P] [US4] Write unit tests for `check_source_staleness()` in `src/trace/resolver.rs` `#[cfg(test)]` module — cases: source mtime newer than OSCAL last-modified → `true`, source mtime older → `false`, unparseable OSCAL timestamp → `false` (graceful fallback), missing OSCAL last-modified → `false`
- [X] T042 [P] [US4] Write unit tests for `validate_line_reference()` in `src/trace/resolver.rs` `#[cfg(test)]` module — cases: line within range → `true`, line beyond range → `false`, line == 0 → `true` (groups), empty file → `false` for any line > 0

### Implementation for User Story 4

- [X] T043 [P] [US4] Implement `check_source_staleness()` in `src/trace/resolver.rs` — parse OSCAL `metadata.last-modified` with `chrono::DateTime::parse_from_rfc3339()`, get source file mtime via `std::fs::metadata().modified()`, convert to `chrono::DateTime<Utc>`, compare. Return `false` on any parse/IO error.
- [X] T044 [P] [US4] Implement `validate_line_reference()` in `src/trace/resolver.rs` — simple bounds check: `line_number == 0 || line_number <= source_line_count`
- [X] T045 [US4] Integrate staleness check into `generate_trace_report()` in `src/trace/mod.rs` — extract `metadata.last-modified` from parsed JSON, call `check_source_staleness()`, set `report.source_stale` field
- [X] T046 [US4] Integrate line validation into walker or report builder — for each mapped entry, call `validate_line_reference()` against source file line count (read source file once, count lines). If out of range, append a per-entry warning marker (e.g., `"⚠"` suffix on Source Line column).
- [X] T047 [US4] Update `format_trace_table()` in `src/trace/formatter.rs` — if `report.source_stale`, prepend a warning line: `"Warning: Source file may have been modified since conversion (source is newer than artifact)"`
- [X] T048 [US4] Write integration test in `tests/trace_report_test.rs` — create a tempfile source with mtime set to future, verify staleness warning appears in formatted output
- [X] T049 [US4] Run quality gates: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`

**Checkpoint**: Source integrity warnings work. All 4 user stories are complete and independently testable.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final quality assurance across all stories

- [X] T050 [P] Write CLI parsing test in `src/cli/mod.rs` `#[cfg(test)]` module — verify `forge trace artifact.json --source policy.md` parses correctly, verify `--output` optional flag, verify missing `--source` fails
- [X] T051 [P] Run `insta` snapshot review for all formatter snapshots: `cargo insta review`
- [X] T052 Verify all edge cases from spec are covered: missing artifact file, missing source file, invalid JSON, unsupported type, no trace metadata, line out of range, control characters in source content
- [X] T053 Final quality gates: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Setup — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Foundational — core MVP
- **US2 (Phase 4)**: Depends on US1 (uses `format_trace_table()` and `generate_trace_report()`)
- **US3 (Phase 5)**: Depends on US1 (extends coverage display built in US1)
- **US4 (Phase 6)**: Depends on US1 (extends report builder with staleness check)
- **Polish (Phase 7)**: Depends on all user stories complete

### User Story Dependencies

- **US1 (P1)**: Can start after Foundational (Phase 2). No story dependencies.
- **US2 (P2)**: Depends on US1 — uses the same report generation + formatting pipeline, adds file output.
- **US3 (P2)**: Depends on US1 — the unmapped rendering and summary are built in US1's formatter, but US3 adds dedicated fixtures and validates coverage correctness.
- **US4 (P3)**: Depends on US1 — adds staleness check and line validation to the existing report builder.

**Note**: US2, US3, and US4 can technically run in parallel after US1 since they modify different files (cli/trace.rs, formatter.rs, resolver.rs). However, US3 overlaps with US1's formatter, so sequential is safer.

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Data structures before logic
- Core implementation before CLI integration
- Unit tests before integration tests
- Story complete before moving to next priority

### Parallel Opportunities

**Phase 2** (Foundational):
- T005, T006, T007 (tests) can all run in parallel
- T008, T009 (structs + strip_control_chars) can run in parallel

**Phase 3** (US1):
- T012, T013, T014, T015, T016 (all US1 tests) can run in parallel
- T017, T018, T019 (walkers) can run in parallel after tests
- T024, T025 (fixtures) can run in parallel with implementation

**Phase 6** (US4):
- T041, T042 (tests) can run in parallel
- T043, T044 (resolver functions) can run in parallel

---

## Parallel Example: User Story 1

```bash
# Launch all US1 tests in parallel (they target different test modules):
Task: "T012 — detect_artifact_type() tests in src/trace/walker.rs"
Task: "T013 — walk_catalog_elements() tests in src/trace/walker.rs"
Task: "T014 — walk_compdef_elements() tests in src/trace/walker.rs"
Task: "T015 — format_trace_table() tests in src/trace/formatter.rs"

# Launch all US1 walkers in parallel (they're independent functions):
Task: "T017 — detect_artifact_type() in src/trace/walker.rs"
Task: "T018 — walk_catalog_elements() in src/trace/walker.rs"
Task: "T019 — walk_compdef_elements() in src/trace/walker.rs"

# Launch both test fixtures in parallel:
Task: "T024 — catalog fixture in tests/fixtures/catalog-with-trace.json"
Task: "T025 — compdef fixture in tests/fixtures/compdef-with-trace.json"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T004)
2. Complete Phase 2: Foundational (T005–T011)
3. Complete Phase 3: User Story 1 (T012–T029)
4. **STOP and VALIDATE**: `forge trace catalog.json --source policy.md` produces correct table
5. Quality gates pass: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`

### Incremental Delivery

1. Setup + Foundational → Data structures and extractor ready
2. Add US1 → `forge trace` produces table to stdout (MVP!)
3. Add US2 → `--output` writes to file
4. Add US3 → Coverage summary validated with partial/no-trace fixtures
5. Add US4 → Staleness warning for modified source files
6. Each story adds value without breaking previous stories

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- All source-derived strings must have control characters stripped (SEC-5) — enforced in formatter
- Reuse WI-17 constants from `src/oscal/trace_embedding.rs` — never hardcode prop names
- `serde_json::Value` for OSCAL parsing — not typed structs (see research.md R-2)
- Groups: show `—` for Source Line, count as mapped if they have `source-section` prop
- Parts: excluded from walking entirely
