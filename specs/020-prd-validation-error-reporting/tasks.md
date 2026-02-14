# Tasks: Validation Error Reporting (WI-20)

**Input**: Design documents from `/specs/020-prd-validation-error-reporting/`
**Prerequisites**: plan.md, PRD, AR, SEC, research.md, data-model.md, contracts/, quickstart.md

**Tests**: TDD is mandatory (Constitution Principle IV). Tests MUST be written first and FAIL before implementation.

**Organization**: Tasks grouped by user story. US1 and US3 can be parallelized (different files). US2 depends on both. US4 depends on all.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Module structure and scaffolding for new validation modules

- [x] T001 Add module declarations for `error_types`, `formatter`, `semantic`, `report` in `src/validate/mod.rs`
- [x] T002 Create empty module files with doc comments: `src/validate/error_types.rs`, `src/validate/formatter.rs`, `src/validate/semantic.rs`, `src/validate/report.rs`
- [x] T003 Add `OutputFormat` enum variant for `--format text|json` to `forge validate` subcommand in `src/cli/mod.rs` (add `ValidateOutputFormat` enum with `Text` and `Json` variants; add `--format` argument to validate command with default `Text`)

**Checkpoint**: `cargo build` succeeds with empty modules declared.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core types that ALL user stories depend on. MUST complete before any story work.

**CRITICAL**: No user story work can begin until this phase is complete.

### Tests for Foundational Types

- [x] T004 [P] Write unit tests for `ValidationErrorCategory` in `src/validate/error_types.rs`: test `Debug`, `Clone`, `Copy`, `PartialEq`, `Serialize`/`Deserialize` round-trip for both `Schema` and `Semantic` variants
- [x] T005 [P] Write unit tests for `ValidationError` in `src/validate/error_types.rs`: test construction with all fields, `Serialize`/`Deserialize` round-trip, test that `actual` field respects 100-char invariant
- [x] T006 [P] Write unit tests for `ValidationReport` in `src/validate/error_types.rs`: test `new()` builder with invariants (`is_valid == errors.is_empty()`, `schema_error_count + semantic_error_count == errors.len()`), test with empty errors, single schema error, single semantic error, mixed errors (SEC-8)
- [x] T007 [P] Write unit tests for `truncate_value()` in `src/validate/formatter.rs`: test short string (no truncation), exactly 100 chars (no truncation), 101+ chars (truncated with "..."), empty string, Unicode at boundary (SEC-1)

### Implementation for Foundational Types

- [x] T008 [P] Implement `ValidationErrorCategory` enum (`Schema`, `Semantic`) with derives (`Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Serialize`, `Deserialize`) and `Display` impl in `src/validate/error_types.rs`
- [x] T009 [P] Implement `ValidationError` struct with fields (`category`, `path`, `message`, `expected`, `actual`) and derives (`Debug`, `Clone`, `Serialize`, `Deserialize`, `PartialEq`) in `src/validate/error_types.rs`
- [x] T010 Implement `ValidationReport` struct with fields (`artifact_path`, `is_valid`, `errors`, `schema_error_count`, `semantic_error_count`), derives (`Debug`, `Clone`, `Serialize`, `Deserialize`), and a `new()` constructor that enforces invariants in `src/validate/error_types.rs`
- [x] T011 Implement `truncate_value(value: &str, max_len: usize) -> String` in `src/validate/formatter.rs` — truncate to `max_len` chars with `"..."` suffix when exceeded (SEC-1)
- [x] T012 Re-export `ValidationErrorCategory`, `ValidationError`, `ValidationReport` from `src/validate/mod.rs` as public API

**Checkpoint**: `cargo test` passes for all foundational types. All invariants enforced.

---

## Phase 3: User Story 1 — Actionable Schema Error Messages (Priority: P1) MVP

**Goal**: Transform raw `jsonschema` crate errors into user-friendly messages with JSON Path, expected constraint, and actual value. (PRD M-1)

**Independent Test**: Run `forge validate` against an artifact with a known schema violation and verify the error message includes the field path, expected constraint, and actual value.

### Tests for User Story 1

- [x] T013 [P] [US1] Write unit tests for `pointer_to_json_path()` in `src/validate/formatter.rs`: test empty pointer → `"$"`, simple path `/catalog` → `"$.catalog"`, nested path `/catalog/metadata/uuid` → `"$.catalog.metadata.uuid"`, array indices `/groups/0/controls/2/id` → `"$.groups[0].controls[2].id"`, deeply nested (6+ levels), malformed pointer (no leading slash) handles gracefully without panic (SEC-6)
- [x] T014 [P] [US1] Write unit tests for `format_schema_error()` in `src/validate/formatter.rs`: test formatting of missing required field error (expected: "required field", actual: "field not present"), wrong type error (expected: type name, actual: actual type), invalid pattern error (expected: pattern, actual: value), verify actual value is truncated to 100 chars (SEC-1), verify raw crate message is NOT passed through (SEC-2), verify no Rust module paths in output (SEC-4)

### Implementation for User Story 1

- [x] T015 [US1] Implement `pointer_to_json_path(pointer: &str) -> String` in `src/validate/formatter.rs` — split by `/`, filter empty segments, convert numeric segments to `[n]` array notation, join with `.`, prefix with `$` (SEC-6: handle malformed pointers gracefully)
- [x] T016 [US1] Implement helper `extract_actual_value(json: &serde_json::Value, pointer: &str) -> String` in `src/validate/formatter.rs` — navigate JSON tree at instance_path, serialize value, truncate to 100 chars via `truncate_value()` (SEC-1)
- [x] T017 [US1] Implement `format_schema_error(raw_error: &jsonschema::ValidationError, json: &serde_json::Value) -> ValidationError` in `src/validate/formatter.rs` — convert instance_path via `pointer_to_json_path()`, extract expected from error message patterns, extract actual via `extract_actual_value()`, construct `ValidationError` with `category: Schema` (SEC-2: never pass raw crate message)
- [x] T018 [US1] Verify all T013/T014 tests pass with `cargo test` for formatter module

**Checkpoint**: Schema errors can be formatted into actionable `ValidationError` structs with JSON Path, expected, and actual.

---

## Phase 4: User Story 3 — Semantic Validation (Priority: P1)

**Goal**: Detect orphaned back-matter links and missing control-id references beyond schema compliance. (PRD M-3, M-4)

**Independent Test**: Create an OSCAL artifact with an orphaned back-matter link and verify `forge validate` reports it as a semantic error.

**Note**: This phase can be executed in PARALLEL with Phase 3 (US1) since it operates on different files.

### Tests for User Story 3

- [x] T019 [P] [US3] Write unit tests for `check_orphaned_links()` in `src/validate/semantic.rs`: test artifact with orphaned link (link href `#uuid` not in back-matter resources) → error reported with path and UUID; test artifact with valid links (no errors); test artifact with no back-matter section but links with `#uuid` hrefs → all reported as orphaned (PRD EC-3); test artifact with multiple orphaned links → all reported; test artifact with no links → no errors (PRD EC-4); verify no external URLs are followed (SEC-5)
- [x] T020 [P] [US3] Write unit tests for `check_missing_references()` in `src/validate/semantic.rs`: test component definition with empty `control-id` → error reported; test component definition with valid control-ids → no errors; test component definition with no implemented-requirements → no errors; test catalog (not component) → skip check gracefully

### Implementation for User Story 3

- [x] T021 [US3] Implement `check_orphaned_links(json: &serde_json::Value) -> Vec<ValidationError>` in `src/validate/semantic.rs` — collect resource UUIDs from `back-matter.resources[].uuid` into HashSet, recursively walk JSON tree tracking current path, for each `href` starting with `#` check UUID exists in set, emit `ValidationError` with `category: Semantic` for orphans (SEC-5: do NOT follow external URLs)
- [x] T022 [US3] Implement `check_missing_references(json: &serde_json::Value, model_type: OscalModelType) -> Vec<ValidationError>` in `src/validate/semantic.rs` — for `ComponentDefinition`: walk `implemented-requirements` and check `control-id` fields are non-empty strings (a control-id is invalid if it is empty, whitespace-only, or contains only punctuation; valid control-ids are non-empty strings with at least one alphanumeric character); for `Catalog`: no-op (return empty vec); emit `ValidationError` with `category: Semantic`
- [x] T023 [US3] Implement `SemanticValidator::validate(json: &serde_json::Value, model_type: OscalModelType) -> Vec<ValidationError>` in `src/validate/semantic.rs` — orchestrate `check_orphaned_links()` + `check_missing_references()`, combine results
- [x] T024 [US3] Verify all T019/T020 tests pass with `cargo test` for semantic module

**Checkpoint**: Semantic errors (orphaned links, missing references) are detected and returned as `ValidationError` structs.

---

## Phase 5: User Story 2 — Report All Errors (Priority: P1)

**Goal**: Collect ALL validation errors (schema + semantic) in a single pass and present them with categorization and summary. (PRD M-2, M-6, S-1, S-2)

**Independent Test**: Run `forge validate` against an artifact with 3+ distinct schema violations and verify all errors are reported with category labels.

**Depends on**: Phase 3 (US1) and Phase 4 (US3) — needs both formatters and semantic validator.

### Tests for User Story 2

- [x] T025 [P] [US2] Write unit tests for `render_text_report()` in `src/validate/report.rs`: test valid report → "Valid" message; test single schema error → formatted with path/expected/actual; test mixed errors (2 schema + 1 semantic) → grouped by category with summary line "3 schema errors, 1 semantic error" (PRD S-2); test 50+ errors → all rendered without truncation (PRD EC-2); test deeply nested path displayed fully (PRD EC-6)
- [x] T026 [P] [US2] Write unit tests for `render_json_report()` in `src/validate/report.rs`: test valid report → JSON with `is_valid: true`, empty errors array; test report with errors → proper JSON structure matching `ValidationReport` schema; verify JSON contains only defined fields (SEC-3); test round-trip: render → parse → compare original report
- [x] T027 [P] [US2] Write unit tests for `run_full_validation()` in `src/validate/mod.rs`: test with valid minimal catalog → `is_valid: true`, no errors; test with invalid catalog (missing metadata) → schema errors formatted with JSON Path; test with artifact containing orphaned links → semantic errors included; test with artifact having both schema and semantic errors → both categories present, counts correct (SEC-8); test error messages do not contain raw crate text (SEC-2)

### Implementation for User Story 2

- [x] T028 [US2] Implement `render_text_report(report: &ValidationReport) -> String` in `src/validate/report.rs` — format summary line with counts (PRD S-2), group errors by category ("Schema Errors:" / "Semantic Errors:"), number each error, show path + message + expected + actual for each
- [x] T029 [US2] Implement `render_json_report(report: &ValidationReport) -> String` in `src/validate/report.rs` — serialize `ValidationReport` via `serde_json::to_string_pretty()` (SEC-3: only defined fields)
- [x] T030 [US2] Implement `run_full_validation(artifact_path: &str, json: &serde_json::Value, model_type: OscalModelType) -> Result<ValidationReport, ValidateError>` in `src/validate/mod.rs` — call `load_schema()` + `jsonschema::validator_for()` to build a validator, iterate raw `jsonschema::ValidationError`s via `iter_errors()`, transform each through `format_schema_error()` (NOTE: must use raw crate errors directly, NOT `validate_artifact()`, because `validate_artifact()` converts to `SchemaError` losing the crate-level error context needed by `format_schema_error()`), call `SemanticValidator::validate()` for semantic errors, combine into `ValidationReport` via `new()` constructor
- [x] T031 [US2] Update `src/cli/validate.rs` to use `run_full_validation()` instead of direct `validate_artifact()` call — on valid: print "Valid" + exit 0; on invalid: render report via `render_text_report()` (or `render_json_report()` if `--format json`) to stderr + exit non-zero
- [x] T032 [US2] Wire `--format` argument from `src/cli/mod.rs` into validate execute function — pass format choice to determine text vs JSON rendering
- [x] T033 [US2] Verify all T025/T026/T027 tests pass with `cargo test`

**Checkpoint**: `forge validate` reports ALL errors (schema + semantic) with category labels, summary, and `--format json` support.

---

## Phase 6: User Story 4 — Auto-Validation in forge convert (Priority: P1)

**Goal**: `forge convert` automatically validates its output and fails with actionable errors if the generated artifact is invalid. (PRD M-5)

**Independent Test**: Introduce a deliberate generation bug that produces invalid OSCAL and verify `forge convert` fails with validation errors rather than writing invalid output.

**Depends on**: Phase 5 (US2) — needs the full validation orchestrator and renderers.

### Tests for User Story 4

- [x] T034 [P] [US4] Write integration test for catalog pipeline auto-validation in `src/pipeline.rs` (or `tests/`): test that `run_catalog_pipeline` with a valid Markdown input produces valid output (no validation errors); test that validation errors in generated output use `ValidationReport` format with JSON Path notation (not raw crate messages)
- [x] T035 [P] [US4] Write integration test for component pipeline auto-validation in `src/pipeline.rs` (or `tests/`): same pattern as T034 for `run_component_pipeline`
- [x] T036 [P] [US4] Write integration test verifying that auto-validation errors go to stderr only and do NOT contaminate stdout output (SEC-7)

### Implementation for User Story 4

- [x] T037 [US4] Update `run_catalog_pipeline()` in `src/pipeline.rs` (lines 153-173) to use `run_full_validation()` instead of raw `validate_artifact()` — replace raw `SchemaError` formatting with `render_text_report()` output to stderr; include semantic validation in auto-validation; fail with non-zero exit code if any errors (PRD M-5, EC-7: do NOT write output file)
- [x] T038 [US4] Update `run_component_pipeline()` in `src/pipeline.rs` (lines 225-247) to use `run_full_validation()` — same pattern as T037 for component pipeline
- [x] T039 [US4] Verify all T034/T035/T036 tests pass with `cargo test`; verify no partial output written on validation failure (PRD EC-7)

**Checkpoint**: `forge convert` auto-validates output with enhanced error reporting. Invalid output never reaches the user.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Edge cases, security hardening, and final quality gates

- [x] T040 [P] Add CLI-level integration test: `forge validate` with a valid artifact prints "Valid" and process exits 0 (PRD EC-1) — distinct from T025 unit-level renderer test; place in `src/cli/validate.rs` tests
- [x] T041 [P] Add edge case test: artifact with 50+ errors reports all without truncation, summary count correct (PRD EC-2) in `src/validate/report.rs` tests
- [x] T042 [P] Add edge case test: deeply nested JSON Path (6+ levels) displayed without truncation in rendered report output (PRD EC-6) — distinct from T013 unit test of `pointer_to_json_path()`; place in `src/validate/report.rs` tests
- [x] T043 [P] Add edge case test: `forge convert` valid artifact passes silently, output written normally (PRD EC-5) in `src/pipeline.rs` tests
- [x] T044 [P] Add documentation note about `--format json` error report sensitivity: error reports inherit the sensitivity classification of the input artifact (SEC finding F1); add to `forge validate --help` output or user documentation
- [x] T045 Run `cargo fmt --check` — zero formatting violations
- [x] T046 Run `cargo clippy -- -D warnings` — zero warnings
- [x] T047 Run full `cargo test` — all tests pass
- [x] T048 Verify test coverage exceeds 80% for `src/validate/error_types.rs`, `src/validate/formatter.rs`, `src/validate/semantic.rs`, `src/validate/report.rs`

**Checkpoint**: All PRD requirements (M-1 through M-6, S-1, S-2), acceptance criteria (AC-1 through AC-8), edge cases (EC-1 through EC-7), and security requirements (SEC-1 through SEC-8) satisfied.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 — BLOCKS all user stories
- **US1 (Phase 3)** and **US3 (Phase 4)**: Both depend on Phase 2 — can run IN PARALLEL
- **US2 (Phase 5)**: Depends on Phase 3 AND Phase 4 (needs formatter + semantic validator)
- **US4 (Phase 6)**: Depends on Phase 5 (needs full orchestrator + renderers)
- **Polish (Phase 7)**: Depends on all user stories

### Dependency Graph

```
Phase 1 (Setup)
    │
    ▼
Phase 2 (Foundational)
    │
    ├──────────────┐
    ▼              ▼
Phase 3 (US1)  Phase 4 (US3)   ◄── PARALLEL
    │              │
    └──────┬───────┘
           ▼
    Phase 5 (US2)
           │
           ▼
    Phase 6 (US4)
           │
           ▼
    Phase 7 (Polish)
```

### Within Each User Story

- Tests MUST be written and FAIL before implementation (TDD — Constitution IV)
- Types/helpers before business logic
- Core implementation before integration
- Story complete before dependents can start

### Parallel Opportunities

- **Phase 2**: T004, T005, T006, T007 (tests) can all run in parallel; T008, T009 (types) can run in parallel
- **Phase 3 + Phase 4**: Entire phases can run in parallel (different files, no shared state)
- **Within US1**: T013, T014 (tests) can run in parallel
- **Within US3**: T019, T020 (tests) can run in parallel
- **Within US2**: T025, T026, T027 (tests) can run in parallel
- **Within US4**: T034, T035, T036 (tests) can run in parallel
- **Phase 7**: T040, T041, T042, T043 (edge case tests) can all run in parallel

---

## Parallel Example: Phases 3 + 4

```bash
# These two phases can execute simultaneously since they touch different files:

# Agent A: US1 — Schema Error Formatting (src/validate/formatter.rs)
Task: "T013 [US1] Tests for pointer_to_json_path()"
Task: "T014 [US1] Tests for format_schema_error()"
Task: "T015 [US1] Implement pointer_to_json_path()"
Task: "T016 [US1] Implement extract_actual_value()"
Task: "T017 [US1] Implement format_schema_error()"

# Agent B: US3 — Semantic Validation (src/validate/semantic.rs)
Task: "T019 [US3] Tests for check_orphaned_links()"
Task: "T020 [US3] Tests for check_missing_references()"
Task: "T021 [US3] Implement check_orphaned_links()"
Task: "T022 [US3] Implement check_missing_references()"
Task: "T023 [US3] Implement SemanticValidator::validate()"
```

---

## Implementation Strategy

### MVP First (US1 Only)

1. Complete Phase 1: Setup (T001-T003)
2. Complete Phase 2: Foundational types (T004-T012)
3. Complete Phase 3: US1 — Actionable schema error messages (T013-T018)
4. **STOP and VALIDATE**: `forge validate` produces actionable schema errors
5. Continue with US3, US2, US4 as incremental delivery

### Incremental Delivery

1. Setup + Foundational → Core types ready
2. Add US1 → Schema errors are actionable → Deploy/Demo (MVP)
3. Add US3 → Semantic validation detects orphaned links/missing refs
4. Add US2 → All errors reported with categories, summary, JSON output
5. Add US4 → `forge convert` auto-validates with enhanced errors
6. Polish → Edge cases, security, coverage

### Parallel Team Strategy

With two agents:
1. Both complete Setup + Foundational together
2. Once Foundational is done:
   - Agent A: US1 (formatter.rs)
   - Agent B: US3 (semantic.rs)
3. Agent A or B: US2 (report.rs + orchestrator) — after both complete
4. Either agent: US4 (pipeline integration)

---

## Traceability

| Task Range | User Story | PRD Requirements | AR Components | SEC Requirements |
|------------|-----------|------------------|---------------|------------------|
| T004-T012 | Foundation | — | ValidationError, ValidationErrorCategory, ValidationReport, truncate_value | SEC-1, SEC-8 |
| T013-T018 | US1 | M-1 | pointer_to_json_path, format_schema_error | SEC-1, SEC-2, SEC-4, SEC-6 |
| T019-T024 | US3 | M-3, M-4 | SemanticValidator, check_orphaned_links, check_missing_references | SEC-5 |
| T025-T033 | US2 | M-2, M-6, S-1, S-2 | render_text_report, render_json_report, run_full_validation | SEC-3, SEC-8 |
| T034-T039 | US4 | M-5 | Pipeline integration | SEC-7 |
| T040-T048 | Polish | EC-1 through EC-7 | — | All SEC |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- ALL user stories are P1 priority — dependency ordering determines sequence, not priority
- TDD is mandatory: write tests FIRST, verify they FAIL, then implement
- Commit after each phase completion
- Stop at any checkpoint to validate independently
- Avoid: raw crate messages in output (SEC-2), actual values > 100 chars (SEC-1), writing output on validation failure (EC-7)
