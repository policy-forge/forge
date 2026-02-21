# Tasks: Golden File Edge Cases (WI-22)

**Input**: Design documents from `specs/022-golden-file-edge-cases/`  
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/rust-test-harness.md`, `quickstart.md`  
**Source**: Primary requirements from `docs/PRD/022-prd-golden-file-edge-cases.md`, architecture constraints from `docs/AR/022-ar-golden-file-edge-cases.md`, security context from `docs/SEC/022-sec-golden-file-edge-cases.md`

**Tests**: Explicitly required. This feature is test infrastructure and mandates TDD-style task ordering (tests written/failing before implementation assertions are finalized).

**Organization**: Tasks are grouped by user story for independent implementation and verification.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Parallelizable (different files, no dependency on unfinished tasks)
- **[Story]**: User story label (`[US1]` ... `[US7]`) for story-phase tasks only
- Every task includes an exact file path

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create WI-22 edge-case test scaffolding and fixture layout.

- [x] T001 Create edge-case integration test scaffold in `tests/golden_edge_case_tests.rs`
- [x] T002 [P] Create fixture mapping guide for WI-22 scenarios in `tests/fixtures/edge-cases/README.md`
- [x] T003 [P] Create initial edge-case fixture directory tree under `tests/fixtures/edge-cases/`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared helpers and guardrails required by all user stories.

**⚠️ CRITICAL**: No user-story phase should begin before these tasks complete.

- [x] T004 Implement shared `run_catalog` and `run_component` execution helpers in `tests/golden_edge_case_tests.rs`
- [x] T005 Implement fixture loaders for expected JSON and expected substring files in `tests/golden_edge_case_tests.rs`
- [x] T006 Implement `assert_edge_case_error` helper (non-zero exit + substring checks) in `tests/golden_edge_case_tests.rs`
- [x] T007 Implement warning assertion helper for `expected-warnings.txt` expectations in `tests/golden_edge_case_tests.rs`
- [x] T008 Implement stable ID extraction and comparison helpers in `tests/golden_edge_case_tests.rs`
- [x] T009 Implement validation aggregation helper for multi-issue checks in `tests/golden_edge_case_tests.rs`
- [x] T010 Define strategy applicability constants (dual-strategy vs strategy-agnostic) in `tests/golden_edge_case_tests.rs`
- [x] T011 Add foundational smoke test for fixture-contract completeness in `tests/golden_edge_case_tests.rs`

**Checkpoint**: WI-22 harness utilities are ready; user stories can proceed.

---

## Phase 3: User Story 1 - No Identifiable Headings (EC-1) (Priority: P1) 🎯 MVP

**Goal**: Ensure headingless policy input fails with actionable, descriptive feedback.

**Independent Test**: Run EC-1 tests in `tests/golden_edge_case_tests.rs` and verify failure diagnostics include cause category, offending path/input, remediation hint, and non-zero exit behavior. Dual-strategy parity is enforced in US7.

### Tests for User Story 1

- [x] T012 [P] [US1] Create headingless policy fixture in `tests/fixtures/edge-cases/ec01-no-headings/input.md`
- [x] T013 [P] [US1] Define required EC-1 error substrings in `tests/fixtures/edge-cases/ec01-no-headings/expected-error.txt`
- [x] T014 [US1] Add failing EC-1 baseline failure tests in `tests/golden_edge_case_tests.rs`

### Implementation for User Story 1

- [x] T015 [US1] Implement EC-1 assertions for FR-003 descriptive failure contract in `tests/golden_edge_case_tests.rs` (strategy matrix enforcement deferred to US7)

**Checkpoint**: US1 is independently testable and enforces actionable error behavior.

---

## Phase 4: User Story 2 - Compound Statement Atomization (EC-2) (Priority: P1)

**Goal**: Validate that compound normative statements split correctly while atomic statements stay singular.

**Independent Test**: Run EC-2 behavior tests in `tests/golden_edge_case_tests.rs`; compare output to expected golden artifacts and verify compound vs atomic handling.

### Tests for User Story 2

- [x] T016 [P] [US2] Create compound-and-atomic fixture input in `tests/fixtures/edge-cases/ec02-compound-atomic/input.md`
- [x] T017 [P] [US2] Add expected catalog output for EC-2 in `tests/fixtures/edge-cases/ec02-compound-atomic/expected-catalog.json`
- [x] T018 [P] [US2] Add expected component output for EC-2 in `tests/fixtures/edge-cases/ec02-compound-atomic/expected-component-definition.json`
- [x] T019 [US2] Add failing EC-2 behavior tests in `tests/golden_edge_case_tests.rs`

### Implementation for User Story 2

- [x] T020 [US2] Implement EC-2 assertions for compound split and atomic preservation in `tests/golden_edge_case_tests.rs`
- [x] T021 [US2] Add EC-2 snapshot artifacts in `tests/snapshots/golden_edge_case_tests__ec02_catalog.snap` and `tests/snapshots/golden_edge_case_tests__ec02_component.snap`

**Checkpoint**: US2 passes independently for both strategies.

---

## Phase 5: User Story 3 - Empty Sections and Missing Metadata (EC-3, EC-4) (Priority: P1)

**Goal**: Validate graceful handling of empty sections and deterministic metadata defaults with warnings.

**Independent Test**: Run EC-3/EC-4 tests in `tests/golden_edge_case_tests.rs`; verify empty-group output, default metadata values, and one warning per missing metadata field.

### Tests for User Story 3

- [x] T022 [P] [US3] Create empty-sections fixture input in `tests/fixtures/edge-cases/ec03-empty-sections/input.md`
- [x] T023 [P] [US3] Add expected EC-3 catalog output in `tests/fixtures/edge-cases/ec03-empty-sections/expected-catalog.json`
- [x] T024 [P] [US3] Add expected EC-3 warning substrings in `tests/fixtures/edge-cases/ec03-empty-sections/expected-warnings.txt`
- [x] T025 [P] [US3] Create missing-metadata fixture input in `tests/fixtures/edge-cases/ec04-missing-metadata/input.md`
- [x] T026 [P] [US3] Add expected EC-4 catalog output with default title/version/author in `tests/fixtures/edge-cases/ec04-missing-metadata/expected-catalog.json`
- [x] T027 [P] [US3] Add expected EC-4 warning substrings (one per missing field) in `tests/fixtures/edge-cases/ec04-missing-metadata/expected-warnings.txt`
- [x] T028 [US3] Add failing EC-3 and EC-4 tests in `tests/golden_edge_case_tests.rs`

### Implementation for User Story 3

- [x] T029 [US3] Implement EC-3/EC-4 assertions enforcing FR-006 defaults and warning granularity in `tests/golden_edge_case_tests.rs`

**Checkpoint**: US3 verifies warning-inclusive output behavior independently.

---

## Phase 6: User Story 4 - Identifier Stability (EC-5, EC-6) (Priority: P1)

**Goal**: Guarantee stable ID continuity for whitespace-only edits and stable ID rotation for non-whitespace normative text changes.

**Independent Test**: Run EC-5/EC-6 stability tests in `tests/golden_edge_case_tests.rs`; confirm all IDs unchanged for whitespace edits and changed IDs + warning for substantive edits.

### Tests for User Story 4

- [x] T030 [P] [US4] Create EC-5 whitespace pair fixtures in `tests/fixtures/edge-cases/ec05-whitespace-only/input-original.md` and `tests/fixtures/edge-cases/ec05-whitespace-only/input-whitespace-variant.md`
- [x] T031 [P] [US4] Create EC-6 substantive-change pair fixtures in `tests/fixtures/edge-cases/ec06-substantive-change/input-original.md` and `tests/fixtures/edge-cases/ec06-substantive-change/input-changed.md`
- [x] T032 [P] [US4] Add expected EC-6 warning substrings in `tests/fixtures/edge-cases/ec06-substantive-change/expected-warnings.txt`
- [x] T033 [US4] Add failing EC-5 and EC-6 stability tests in `tests/golden_edge_case_tests.rs`

### Implementation for User Story 4

- [x] T034 [US4] Implement EC-5 stable ID equality assertions for whitespace-only changes in `tests/golden_edge_case_tests.rs`
- [x] T035 [US4] Implement EC-6 stable ID change assertions for non-whitespace normative edits in `tests/golden_edge_case_tests.rs`

**Checkpoint**: US4 validates stable identifier integrity independently.

---

## Phase 7: User Story 5 - Malformed Citations and Validation Errors (EC-7, EC-9, EC-10) (Priority: P1)

**Goal**: Preserve malformed citations with explicit unvalidated marker, handle missing files descriptively, and report all validation issue types together.

**Independent Test**: Run EC-7/EC-9/EC-10 tests in `tests/golden_edge_case_tests.rs`; verify citation back-matter prop contract, strategy-agnostic missing-file failure contract, and aggregated schema+semantic issue reporting.

### Tests for User Story 5

- [x] T036 [P] [US5] Create malformed-citation fixture input in `tests/fixtures/edge-cases/ec07-malformed-citation/input.md`
- [x] T037 [P] [US5] Add expected EC-7 catalog output containing `prop name="url-status" value="unvalidated"` in `tests/fixtures/edge-cases/ec07-malformed-citation/expected-catalog.json`
- [x] T038 [P] [US5] Add expected EC-9 error substrings in `tests/fixtures/edge-cases/ec09-file-not-found/expected-error.txt`
- [x] T039 [P] [US5] Create EC-10 multi-issue validation fixture input in `tests/fixtures/edge-cases/ec10-multiple-errors/input.md`
- [x] T040 [P] [US5] Add expected EC-10 schema+semantic issue substrings in `tests/fixtures/edge-cases/ec10-multiple-errors/expected-errors.txt`
- [x] T041 [US5] Add failing EC-7, EC-9, and EC-10 tests in `tests/golden_edge_case_tests.rs`

### Implementation for User Story 5

- [x] T042 [US5] Implement EC-7 back-matter citation retention assertions in `tests/golden_edge_case_tests.rs`
- [x] T043 [US5] Implement EC-9 strategy-agnostic failure assertions in `tests/golden_edge_case_tests.rs`
- [x] T044 [US5] Implement EC-10 aggregated validation assertions (all issue categories reported) in `tests/golden_edge_case_tests.rs`

**Checkpoint**: US5 independently validates malformed-citation and validation-error behavior.

---

## Phase 8: User Story 7 - Both Strategies Tested (Priority: P1)

**Goal**: Enforce full dual-strategy coverage for all strategy-applicable edge cases and single-run validation for EC-9.

**Independent Test**: Run strategy-matrix tests in `tests/golden_edge_case_tests.rs`; verify EC-1/2/3/4/5/6/7/10 run under both strategies and EC-9 runs once.

### Tests for User Story 7

- [x] T045 [US7] Add explicit strategy applicability matrix for EC scenarios in `tests/golden_edge_case_tests.rs`
- [x] T046 [US7] Add failing strategy-matrix coverage tests in `tests/golden_edge_case_tests.rs`

### Implementation for User Story 7

- [x] T047 [US7] Implement matrix runner and per-strategy expected artifact assertions in `tests/golden_edge_case_tests.rs`
- [x] T048 [US7] Add strategy-matrix snapshot artifacts in `tests/snapshots/golden_edge_case_tests__strategy_matrix_catalog.snap` and `tests/snapshots/golden_edge_case_tests__strategy_matrix_component.snap`

**Checkpoint**: US7 confirms coverage parity across strategies.

---

## Phase 9: User Story 6 - Citation Extraction and Parameter-Like Content (Priority: P2)

**Goal**: Add edge fixtures for unusual citation placement and parameter-like text preservation without corruption.

**Independent Test**: Run US6 tests in `tests/golden_edge_case_tests.rs`; verify citations extract correctly from unusual positions and parameter-like prose remains intact in output.

### Tests for User Story 6

- [x] T049 [P] [US6] Create unusual citation placement fixture in `tests/fixtures/edge-cases/ec-citation-unusual-positions/input.md`
- [x] T050 [P] [US6] Add expected catalog/component outputs for unusual citation placement in `tests/fixtures/edge-cases/ec-citation-unusual-positions/expected-catalog.json` and `tests/fixtures/edge-cases/ec-citation-unusual-positions/expected-component-definition.json`
- [x] T051 [P] [US6] Create parameter-like content fixture in `tests/fixtures/edge-cases/ec-parameter-like-content/input.md`
- [x] T052 [P] [US6] Add expected catalog/component outputs for parameter-like preservation in `tests/fixtures/edge-cases/ec-parameter-like-content/expected-catalog.json` and `tests/fixtures/edge-cases/ec-parameter-like-content/expected-component-definition.json`
- [x] T053 [US6] Add failing US6 tests for unusual citation positions and parameter-like prose preservation in `tests/golden_edge_case_tests.rs`

### Implementation for User Story 6

- [x] T054 [US6] Implement US6 assertions and snapshot checks in `tests/golden_edge_case_tests.rs`

**Checkpoint**: US6 (P2) passes independently and does not regress P1 stories.

---

## Phase 10: Polish & Cross-Cutting Concerns

**Purpose**: Finalize cross-story quality checks, stakeholder actionability evidence, and documentation.

- [x] T055 [P] Add fixture integrity coverage for WI-22 directories in `tests/fixture_validity_test.rs`, including explicit FR-012 guards (no EC-8 fixture path and no performance-benchmark test artifacts)
- [x] T056 [P] Add schema/validation regression checks for WI-22 expected outputs in `tests/validate_test.rs`
- [x] T057 [P] Add cross-story traceability comments linking FR IDs to test cases in `tests/golden_edge_case_tests.rs`
- [x] T058 [P] Update WI-22 execution steps and verification commands in `specs/022-golden-file-edge-cases/quickstart.md`
- [x] T059 [P] Update plan acceptance checklist with WI-22 test completion evidence in `specs/022-golden-file-edge-cases/plan.md`
- [x] T060 Finalize and normalize WI-22 snapshot set in `tests/snapshots/golden_edge_case_tests__ec10_validation_errors.snap`
- [x] T061 Run `cargo test` and record WI-22 pass evidence in `specs/022-golden-file-edge-cases/checklists/requirements.md`
- [x] T062 Run `cargo clippy -- -D warnings` and record lint pass evidence in `specs/022-golden-file-edge-cases/checklists/requirements.md`
- [x] T063 Run `cargo fmt --check` and record formatting pass evidence in `specs/022-golden-file-edge-cases/checklists/requirements.md`
- [x] T064 [P] Review fixture realism against WI-21 style guardrails and document outcomes in `tests/fixtures/edge-cases/README.md`
- [x] T065 [P] Create stakeholder actionability review rubric for failure scenarios in `specs/022-golden-file-edge-cases/checklists/actionability-review.md`
- [x] T066 Execute actionability review and record SC-006 score (>=95%) in `specs/022-golden-file-edge-cases/checklists/actionability-review.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies
- **Phase 2 (Foundational)**: Depends on Phase 1 and blocks all stories
- **Phases 3-9 (User Stories)**: Depend on Phase 2 completion
- **Phase 10 (Polish)**: Depends on completion of selected user stories

### User Story Dependencies

- **US1 (P1)**: Starts after Phase 2; no story dependency
- **US2 (P1)**: Starts after Phase 2; no story dependency
- **US3 (P1)**: Starts after Phase 2; no story dependency
- **US4 (P1)**: Starts after Phase 2; no story dependency
- **US5 (P1)**: Starts after Phase 2; no story dependency
- **US7 (P1)**: Depends on US1-US5 fixture/test availability
- **US6 (P2)**: Starts after Phase 2; can run after MVP is stable

### Recommended Completion Order

1. Phase 1 -> Phase 2
2. MVP: US1
3. Remaining P1 stories: US2 -> US3 -> US4 -> US5 -> US7
4. P2 enhancement: US6
5. Phase 10 polish

---

## Parallel Execution Examples Per User Story

### US1

```bash
# Run in parallel:
T012 and T013
```

### US2

```bash
# Run in parallel:
T016, T017, T018
```

### US3

```bash
# Run in parallel:
T022, T023, T024, T025, T026, T027
```

### US4

```bash
# Run in parallel:
T030, T031, T032
```

### US5

```bash
# Run in parallel:
T036, T037, T038, T039, T040
```

### US7

```bash
# Mostly sequential in one file:
T045 -> T046 -> T047 (T048 after test output stabilizes)
```

### US6

```bash
# Run in parallel:
T049, T050, T051, T052
```

---

## Implementation Strategy

### MVP First (US1 only)

1. Complete Phases 1-2
2. Complete Phase 3 (US1)
3. Validate US1 independently in `tests/golden_edge_case_tests.rs`
4. Demo actionable failure feedback behavior

### Incremental Delivery

1. Deliver US1 (MVP)
2. Add US2, US3, US4, US5 incrementally (all P1)
3. Add US7 strategy matrix once P1 fixtures are stable
4. Add US6 (P2) as enhancement
5. Finish with Phase 10 quality/documentation tasks

### Parallel Team Strategy

1. One engineer completes Phases 1-2
2. Then parallelize by story owners:
   - Engineer A: US1 + US5
   - Engineer B: US2 + US4
   - Engineer C: US3 + US6
3. Integrate US7 after all required P1 stories land

---

## Notes

- All tasks use strict checklist format: checkbox, task ID, optional `[P]`, required `[USx]` in story phases, and explicit file paths.
- EC-8 and performance benchmarking are explicitly out of scope for WI-22.
- FR-012 scope exclusions are enforced through executable fixture-integrity assertions (T055), not documentation-only checks.
- Failure and warning assertions should use required substrings, not exact full-message matching.
