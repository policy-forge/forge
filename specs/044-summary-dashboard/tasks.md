# Tasks: Summary Dashboard (044)

**Input**: Design documents from `/specs/044-summary-dashboard/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/summary.rs, PRD, AR, SEC

**Tests**: TDD mandatory per constitution (Principle IV). Tests are written first and must fail before implementation.

**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Module Structure)

**Purpose**: Create the summary module skeleton and register it in the crate

- [x] T001 Create `src/summary/mod.rs` with `ConversionStatistics` struct, `ValidationStatus` enum, `Default` impls, `mapping_coverage()` method, `count_sections()`, `count_requirements()`, and `count_catalog_controls()` helper functions per `contracts/summary.rs`
- [x] T002 [P] Create `src/summary/format.rs` with `format_summary_dashboard()` (box-drawing table with all fields: strategy, output, elapsed, sections, requirements, controls, mapping coverage, validation status with up to 3 error messages), `format_elapsed()`, and ANSI color helpers using `std::io::IsTerminal`
- [x] T003 Add `pub mod summary;` to `src/lib.rs`

---

## Phase 2: Foundational (Core Infrastructure + TDD)

**Purpose**: Unit tests for all summary module functions, pipeline return type changes, CLI flag addition

**CRITICAL**: Must complete before user story wiring can begin

### Tests (write first, must FAIL)

- [x] T004 [P] Write unit tests for `ConversionStatistics::mapping_coverage()` in `src/summary/mod.rs` — cases: zero requirements → 0.0, partial (12/15 → 80.0), full (15/15 → 100.0), exceeds (18/15 → 120.0). Use approximate f64 comparison with epsilon
- [x] T005 [P] Write unit tests for `count_sections()` and `count_requirements()` in `src/summary/mod.rs` — cases: empty doc, flat sections, nested sections with children, multiple requirements per section
- [x] T006 [P] Write unit tests for `count_catalog_controls()` in `src/summary/mod.rs` — cases: empty catalog, single group with controls, nested groups
- [x] T007 [P] Write unit tests for `format_elapsed()` in `src/summary/format.rs` — cases: sub-second (0.42s), multi-second (3.2s), over-minute (1m 23s), zero duration
- [x] T008 [P] Write unit tests for `format_summary_dashboard()` in `src/summary/format.rs` — both color and no-color modes; assert key substrings present ("FORGE Conversion Summary", "Sections parsed:", "Requirements:", "Controls generated:", "Mapping coverage:", "Validation:", box-drawing chars ┌ ─ ┐ │ └ ┘). Use insta snapshot tests for exact formatting
- [x] T009 [P] Write unit test for `format_summary_dashboard()` with `ValidationStatus::Failed` and 3+ error messages in `src/summary/format.rs` — assert up to 3 messages displayed with "and N more..." overflow

### Implementation

- [x] T010 Implement all functions in `src/summary/mod.rs` until T004, T005, T006 tests pass
- [x] T011 Implement all functions in `src/summary/format.rs` until T007, T008, T009 tests pass

### Pipeline and CLI Changes

- [x] T012 Modify `src/pipeline.rs`: change `run_catalog_pipeline` return type from `Result<(), ForgeError>` to `Result<ConversionStatistics, ForgeError>` — populate stats from `PolicyDocument` (sections, requirements), catalog controls, and validation result; set strategy to "catalog"
- [x] T013 Modify `src/pipeline.rs`: change `run_component_pipeline` return type from `Result<(), ForgeError>` to `Result<ConversionStatistics, ForgeError>` — populate stats from `PolicyDocument` (sections, requirements), implemented-requirements count, and validation result; set strategy to "component"
- [x] T014 Update all existing tests that match on `Ok(())` from `run_catalog_pipeline` and `run_component_pipeline` to use `Ok(_)` or destructure the stats — files: `src/pipeline.rs` (mod tests), `tests/catalog_pipeline_test.rs`, `tests/component_pipeline_test.rs`, `tests/pipeline_test.rs`, and any other integration tests calling these functions
- [x] T015 Add `--summary` flag (bool) to the `Convert` variant in `src/cli/mod.rs` — use `#[arg(long)]`; pass through to `convert::execute`
- [x] T016 Update `convert::execute` signature in `src/cli/convert.rs` to accept `summary: bool` parameter; update the match arm in `src/cli/mod.rs` `execute()` to pass the new field
- [x] T017 Write CLI parsing test in `src/cli/mod.rs` — verify `--summary` flag is accepted and parsed correctly; verify it defaults to false when omitted

**Checkpoint**: All unit tests pass. Pipeline returns stats. CLI accepts `--summary`. No dashboard printed yet.

---

## Phase 3: User Story 1 — View Conversion Statistics (Priority: P1) MVP

**Goal**: User runs `forge convert --summary` and sees sections parsed, requirements extracted, and controls generated counts.

**Independent Test**: Run `forge convert policy.md --strategy catalog --summary --output out.json` on a known fixture and verify stdout includes counts for sections, requirements, and controls.

**Traces to**: FR-001, FR-002, FR-003, FR-004, FR-007, FR-008, FR-009, M-1, M-2, M-3, M-4, M-7

### Tests

- [x] T018 [US1] Write integration test in `tests/summary_dashboard_test.rs` — run catalog pipeline with `--summary` on `tests/fixtures/sample_policy.md`, capture stdout, assert it contains "Sections parsed:", "Requirements:", "Controls generated:" with correct counts matching the fixture
- [x] T019 [US1] Write integration test in `tests/summary_dashboard_test.rs` — run catalog pipeline WITHOUT `--summary`, assert stdout does NOT contain "FORGE Conversion Summary" (FR-008: default behavior unchanged)
- [x] T020 [P] [US1] Write integration test in `tests/summary_dashboard_test.rs` — run catalog pipeline with `--summary`, verify the output artifact file is identical to a run without `--summary` (FR-009: flag does not alter artifact)

### Implementation

- [x] T021 [US1] Wire dashboard printing in `src/cli/convert.rs` — when `summary` is true: wrap pipeline call with `Instant::now()`/`elapsed()`, receive `ConversionStatistics` from pipeline, set `output_path` and `elapsed`, call `format_summary_dashboard()` with `std::io::stdout().is_terminal()` for color detection, print result to stdout after artifact write
- [x] T022 [US1] Verify T018, T019, T020 pass — run `cargo test summary_dashboard`

**Checkpoint**: `forge convert --summary` prints dashboard with core statistics. MVP functional.

---

## Phase 4: User Story 2 — View Validation Status (Priority: P1)

**Goal**: User sees validation status (PASSED, Not run) in the summary dashboard.

**Independent Test**: Run `forge convert --summary` and verify stdout includes validation status matching actual validation result.

**Traces to**: FR-005, M-5, SEC-3, EC-2, EC-5

### Tests

- [x] T023 [US2] Write unit test in `src/summary/format.rs` — verify `ValidationStatus::Passed` renders as "PASSED" (with green ANSI when color enabled)
- [x] T024 [P] [US2] Write unit test in `src/summary/format.rs` — verify `ValidationStatus::NotRun` renders as "Not run"
- [x] T025 [P] [US2] Write unit test in `src/summary/format.rs` — verify `ValidationStatus::Failed` with 5 errors renders count + first 3 messages + "and 2 more..."
- [x] T026 [US2] Write integration test in `tests/summary_dashboard_test.rs` — run catalog pipeline with `--summary` on valid fixture, assert stdout contains "Validation:" followed by "PASSED"

### Implementation

- [x] T027 [US2] Ensure `src/pipeline.rs` correctly sets `validation_status` to `Passed` on successful validation and captures up to 3 error messages from `ValidationReport` on failure (verify existing wiring from T012/T013 handles this)
- [x] T028 [US2] Verify T023, T024, T025, T026 pass

**Checkpoint**: Dashboard shows validation status. Both P1 stories complete.

---

## Phase 5: User Story 3 — View Mapping Coverage (Priority: P2)

**Goal**: User sees mapping coverage percentage with raw counts in the dashboard.

**Independent Test**: Convert a policy where 12 of 15 requirements produce controls, verify stdout shows "80.0% (12/15)".

**Traces to**: FR-006, FR-012, M-6, SEC-2, EC-1, EC-6

### Tests

- [x] T029 [US3] Write integration test in `tests/summary_dashboard_test.rs` — verify dashboard output contains "Mapping coverage:" with percentage and raw counts for the fixture
- [x] T030 [P] [US3] Write unit test in `src/summary/format.rs` — verify zero requirements case renders "0.0% (0/0)" with a warning indicator
- [x] T031 [P] [US3] Write unit test in `src/summary/format.rs` — verify >100% coverage case (e.g., 18/15) renders correctly as "120.0% (18/15)"

### Implementation

- [x] T032 [US3] Ensure `format_summary_dashboard()` renders mapping coverage with correct formatting: percentage to 1 decimal place, raw counts in parentheses, warning for 0/0 case, yellow color for >100% — verify all tests pass

**Checkpoint**: Dashboard shows mapping coverage with edge cases handled.

---

## Phase 6: User Story 4 — View Conversion Context (Priority: P2)

**Goal**: User sees strategy, output path, and elapsed time in the dashboard with ANSI colors.

**Independent Test**: Run `forge convert --strategy catalog --summary` and verify dashboard includes strategy name, output path, and elapsed time.

**Traces to**: FR-010, FR-011, FR-013, FR-014, S-1, S-2, S-3

### Tests

- [x] T033 [US4] Write integration test in `tests/summary_dashboard_test.rs` — verify dashboard contains "Strategy:" with "catalog", and "Output:" with the output file path
- [x] T034 [P] [US4] Write unit test in `src/summary/format.rs` — verify elapsed time appears in dashboard with correct format (e.g., "0.42s")
- [x] T035 [P] [US4] Write unit test in `src/summary/format.rs` — verify ANSI color codes present when `use_color=true` and absent when `use_color=false`

### Implementation

- [x] T036 [US4] Verify `src/cli/convert.rs` correctly sets `strategy` (from CLI arg) and `output_path` (from `--output` or "stdout") on `ConversionStatistics` before passing to formatter — verify T033, T034, T035 pass

**Checkpoint**: Dashboard is fully featured with context, colors, and elapsed time.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Security verification, component pipeline coverage, quality gates

- [x] T037 [P] Write integration test in `tests/summary_dashboard_test.rs` — run COMPONENT pipeline with `--summary` (using `--strategy component`), verify dashboard shows "component" as strategy and correct counts (SEC-3: flag doesn't alter behavior)
- [x] T038 [P] Write integration test in `tests/summary_dashboard_test.rs` — verify dashboard output contains ONLY aggregate counts, no policy content (SEC-1: data protection)
- [x] T039 [P] Verify `--summary` combined with `--format xml` and `--format yaml` both produce correct dashboard (edge case from spec)
- [x] T044 [P] Write integration test in `tests/summary_dashboard_test.rs` — trigger a conversion error (e.g., non-existent input file) with `--summary` flag and verify NO dashboard output is printed, only the error message (EC-5: conversion failure suppresses summary)
- [x] T045 [P] Write unit test in `src/summary/format.rs` — verify `ValidationStatus::PassedWithWarnings` renders as "PASSED with N warnings" (forward-compatibility coverage for U2)
- [x] T040 Run `cargo clippy -- -D warnings` and fix any warnings in new/modified files
- [x] T041 Run `cargo fmt --check` and fix any formatting issues
- [x] T042 Run full `cargo test` suite and verify zero regressions
- [x] T043 Run quickstart.md validation: execute `forge convert tests/fixtures/sample_policy.md --strategy catalog --summary --output /tmp/test_catalog.json` and verify dashboard output matches expected format

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 — BLOCKS all user stories
- **Phase 3 (US1)**: Depends on Phase 2 — MVP target
- **Phase 4 (US2)**: Depends on Phase 2 — can run in parallel with US1
- **Phase 5 (US3)**: Depends on Phase 2 — can run in parallel with US1/US2
- **Phase 6 (US4)**: Depends on Phase 2 — can run in parallel with US1/US2/US3
- **Phase 7 (Polish)**: Depends on all user stories complete

### User Story Dependencies

- **US1 (P1)**: No dependencies on other stories. MVP scope.
- **US2 (P1)**: No dependencies on other stories. Can run in parallel with US1.
- **US3 (P2)**: No dependencies on other stories. Uses mapping_coverage() from foundational.
- **US4 (P2)**: No dependencies on other stories. Can run in parallel with US3.

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Implementation resolves failing tests
- Checkpoint validates story independently

### Parallel Opportunities

- T001, T002 can run in parallel (different files)
- T004, T005, T006, T007, T008, T009 can all run in parallel (different test functions)
- T012, T013 can run in parallel (different pipeline functions in same file — careful of conflicts)
- T018, T019, T020 can run in parallel (different test functions)
- T023, T024, T025 can run in parallel (different test functions)
- T029, T030, T031 can run in parallel (different test functions)
- T033, T034, T035 can run in parallel (different test functions)
- T037, T038, T039 can run in parallel (different test functions)
- US1, US2, US3, US4 can all proceed in parallel after Phase 2

---

## Parallel Example: Foundational Phase

```
# Launch all unit test tasks in parallel:
T004: mapping_coverage() tests in src/summary/mod.rs
T005: counting helper tests in src/summary/mod.rs
T006: catalog control counting tests in src/summary/mod.rs
T007: format_elapsed() tests in src/summary/format.rs
T008: format_summary_dashboard() snapshot tests in src/summary/format.rs
T009: validation error overflow tests in src/summary/format.rs
```

## Parallel Example: All User Stories After Phase 2

```
# Launch all user story phases in parallel:
Phase 3 (US1): Core statistics wiring + integration tests
Phase 4 (US2): Validation status wiring + integration tests
Phase 5 (US3): Mapping coverage edge case tests
Phase 6 (US4): Context and color wiring + tests
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T003)
2. Complete Phase 2: Foundational (T004–T017)
3. Complete Phase 3: User Story 1 (T018–T022)
4. **STOP and VALIDATE**: `forge convert --summary` prints core statistics
5. Run `cargo test && cargo clippy -- -D warnings && cargo fmt --check`

### Incremental Delivery

1. Phase 1 + Phase 2 → Foundation ready
2. Add US1 → Core dashboard functional (MVP!)
3. Add US2 → Validation status shown
4. Add US3 → Mapping coverage with edge cases
5. Add US4 → Full context, colors, elapsed time
6. Phase 7 → Quality gates, security verification

### Requirement Traceability

| Requirement | Task(s) |
|-------------|---------|
| FR-001 (--summary flag) | T015, T016, T017 |
| FR-002 (sections count) | T001, T005, T012, T018 |
| FR-003 (requirements count) | T001, T005, T012, T018 |
| FR-004 (controls count) | T001, T006, T012, T013, T018 |
| FR-005 (validation status) | T001, T023–T027 |
| FR-006 (mapping coverage) | T001, T004, T029–T032 |
| FR-007 (printed after write) | T021 |
| FR-008 (no dashboard without flag) | T019 |
| FR-009 (no artifact change) | T020 |
| FR-010 (strategy) | T002, T033, T036 |
| FR-011 (output path) | T002, T033, T036 |
| FR-012 (zero requirements) | T004, T030 |
| FR-013 (elapsed time) | T002, T007, T034, T036 |
| FR-014 (ANSI colors) | T002, T035 |
| SEC-1 (no policy content) | T038 |
| SEC-2 (zero-division guard) | T004, T030 |
| SEC-3 (no behavior change) | T020, T037 |
| EC-5 (conversion failure → no summary) | T044 |
| FR-005 (PassedWithWarnings variant) | T045 |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Constitution Principle IV (TDD): All test tasks must be written and fail before implementation
- Constitution Principle VIII: Test assertions use substring matching, not brittle full-message equality
- Use insta snapshot tests for dashboard formatting to catch unintended format changes
- f64 coverage comparisons use epsilon (0.01) per AR anti-pattern guidance
