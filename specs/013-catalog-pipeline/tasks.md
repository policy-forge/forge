# Tasks: End-to-End Catalog Pipeline (WI-13)

**Input**: Design documents from `/specs/013-catalog-pipeline/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/pipeline.rs
**Tests**: Included — project constitution mandates TDD (Principle IV).
**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Exact file paths included in descriptions

## Phase 1: Setup

**Purpose**: Minimal changes that unblock pipeline implementation

- [X] T001 [P] Add `ForgeError::Serialization(String)` variant with `#[error("Serialization error: {0}")]` and unit test to src/error.rs (D5, R6)
- [X] T002 [P] Add `pub mod pipeline;` declaration to src/lib.rs (D4)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Test fixture required by US1 and US3

**⚠️ CRITICAL**: US1 smoke test (T005) cannot run without this fixture

- [X] T003 Create test fixture tests/fixtures/full_policy.md — YAML frontmatter (title: "Sample Security Policy", version: "1.0.0"), 3 top-level sections (Access Control, Data Protection, Incident Response), 10+ requirements across sections, at least 1 compound statement ("Systems must X and must Y") for atomization testing

**Checkpoint**: Foundation ready — user story implementation can begin

---

## Phase 3: User Story 1 — Core Pipeline Conversion (Priority: P1) 🎯 MVP

**Goal**: `run_catalog_pipeline` converts a Markdown policy to valid OSCAL Catalog JSON

**Independent Test**: Call `run_catalog_pipeline` programmatically with full_policy.md fixture and verify valid JSON output containing catalog object with metadata, groups, and controls

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T004 [P] [US1] Write unit tests for write_output in src/pipeline.rs — file creation with correct content (M-5), error when parent directory does not exist (SEC-6, EC-3)
- [X] T005 [US1] Write end-to-end smoke test in tests/catalog_pipeline_test.rs — call run_catalog_pipeline with tests/fixtures/full_policy.md, capture output to temp file, parse JSON, assert presence of `catalog` object with `metadata`, `groups`, and controls; assert top-level JSON contains only expected OSCAL keys (no extraneous data per SEC-1) (M-7, AC-6, SEC-1)

### Implementation for User Story 1

- [X] T006 [US1] Implement write_output function in src/pipeline.rs — if output_path is None, println JSON to stdout; if Some, validate parent dir exists (return ForgeError::Validation if not), then write JSON to file (M-4, M-5, SEC-6)
- [X] T007 [US1] Implement run_catalog_pipeline orchestrator in src/pipeline.rs — wire 13 pipeline steps: (1) ingest_file(input_path, max_size_bytes), (2) reconstruct_content(), (3) extract_sections(&content), (4) extract_clauses(&content), (5) assemble_document(&ingested, &sections, &clauses), (6) atomize_document(&document), (7) assign_stable_ids(&mut document), (8) build_catalog(&document), (9) assemble_metadata(&doc.metadata, None), (10) generate_back_matter(&[]) — empty citations stub per D2, (11) assemble CatalogEnvelope mapping real metadata fields to placeholder strings per D1 (uuid→to_string, last_modified→to_rfc3339, title/version/oscal_version directly), (12) serde_json::to_string_pretty(&envelope) with ForgeError::Serialization on failure, (13) write_output(&json, output_path) (M-1, M-6, S-1)

**Checkpoint**: Pipeline converts Markdown to OSCAL JSON — core value delivered

---

## Phase 4: User Story 2 — CLI Integration & File Output (Priority: P1)

**Goal**: `forge convert policy.md --strategy catalog --format json [--output path]` works end-to-end via CLI

**Independent Test**: Run forge binary with test fixture and required flags, verify OSCAL Catalog JSON on stdout and in output file

### Tests for User Story 2

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T008 [P] [US2] Write CLI integration tests for pipeline output in tests/cli_integration.rs — (a) stdout: run with --strategy catalog --format json, assert valid JSON with `catalog` key (AC-3); (b) file: run with --output flag, assert file contains OSCAL JSON, verify output file has default permissions (not elevated) on Unix via std::os::unix::fs::PermissionsExt (AC-4, SEC-2); (c) overwrite: run twice with same --output path, assert second run overwrites first (EC-7)

### Implementation for User Story 2

- [X] T009 [US2] Make --strategy required (remove `Option<Strategy>` wrapper, use `Strategy` directly) and --format required (remove `default_value = "json"`) in src/cli/mod.rs; update execute() to pass `&strategy` instead of `strategy.as_ref()`; update CLI unit tests (parse_convert_subcommand, parse_convert_with_all_options) to include --strategy catalog --format json flags (M-2, M-3, D3, SEC-3)
- [X] T010 [US2] Update src/cli/convert.rs — remove ConvertOutput struct, change execute signature from `_strategy: Option<&Strategy>` to `strategy: &Strategy`, match on strategy: Strategy::Catalog dispatches to `pipeline::run_catalog_pipeline(input, output, max_size_bytes)`, Strategy::Component returns descriptive ForgeError::Validation (S-3); remove old inline pipeline logic (M-1, M-2)
- [X] T011 [US2] Update all existing CLI integration tests in tests/cli_integration.rs — add --strategy catalog --format json flags to: convert_valid_md_outputs_json, convert_pdf_shows_unsupported_format_error, convert_nonexistent_file_shows_not_found_error, convert_directory_shows_not_a_file_error, convert_oversized_file_shows_size_error, convert_oversized_file_with_max_size_override_succeeds; update convert_valid_md_outputs_json assertions to expect OSCAL Catalog JSON structure instead of old ConvertOutput format

**Checkpoint**: CLI convert command works — full user-facing capability

---

## Phase 5: User Story 3 — Smoke Test Validation (Priority: P1)

**Goal**: Comprehensive smoke test verifies all pipeline stages produce correct OSCAL output

**Independent Test**: Run `cargo test catalog_pipeline` and verify all assertions pass

### Tests for User Story 3

- [X] T012 [P] [US3] Add smoke test assertion: number of groups in output matches number of top-level sections in full_policy.md fixture (expect 3) in tests/catalog_pipeline_test.rs (AC-1)
- [X] T013 [P] [US3] Add smoke test assertion: metadata fields correctly populated — title matches frontmatter, version matches frontmatter, oscal-version is "1.2.0", last-modified is valid RFC 3339 timestamp in tests/catalog_pipeline_test.rs (AC-2)
- [X] T014 [P] [US3] Add smoke test assertion: compound requirements atomized into separate controls with unique stable IDs in tests/catalog_pipeline_test.rs (AC-7)
- [X] T015 [US3] Add edge case test: input Markdown with no sections produces catalog with empty groups array and emits a warning to stderr (via tracing::warn!) in tests/catalog_pipeline_test.rs (EC-6)

**Checkpoint**: Smoke test validates all pipeline stages — engineering confidence established

---

## Phase 6: User Story 4 — Descriptive Error Reporting (Priority: P2)

**Goal**: Clear error messages and non-zero exit codes for all invalid inputs

**Independent Test**: Run forge convert with invalid inputs and verify descriptive errors and non-zero exit codes

### Tests for User Story 4

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T016 [P] [US4] Write CLI edge case tests in tests/cli_integration.rs — (a) empty file → non-zero exit, descriptive error (EC-2, SEC-5); (b) --output with non-existent parent dir → non-zero exit, error about invalid path (EC-3, SEC-6); (c) omitted --strategy → clap error indicating --strategy required (EC-4, SEC-3); (d) omitted --format → clap error indicating --format required (EC-5, SEC-3); (e) --strategy component → descriptive rejection error mentioning catalog-only support (S-3)

### Implementation for User Story 4

> **NOTE**: EC-2 is handled by existing ingest stage. EC-3 by write_output (T006). EC-4/EC-5 by clap (T009). Only S-3 needs new implementation — but this was already included in T010 (Strategy::Component match arm). Verify all tests pass.

- [X] T017 [US4] Verify all edge case tests pass — if --strategy component rejection is not yet implemented in T010, add ForgeError::Validation("Only 'catalog' strategy is currently supported. Component support will be available in a future release.") to src/cli/convert.rs (S-3)

**Checkpoint**: All error paths produce clear, descriptive messages

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final validation across all user stories

- [X] T018 [P] Run `cargo clippy -- -D warnings` and fix any issues
- [X] T019 [P] Run `cargo fmt --check` and fix any formatting issues
- [X] T020 Run `cargo test` and verify all tests pass (unit + integration + smoke)
- [X] T021 Validate quickstart.md — run documented commands (`cargo run -- convert policy.md --strategy catalog --format json`) and verify expected OSCAL JSON output structure

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS US1 smoke test
- **US1 (Phase 3)**: Depends on Phase 1 (T001 for Serialization variant, T002 for pipeline module) and Phase 2 (T003 for fixture)
- **US2 (Phase 4)**: Depends on US1 — CLI dispatch needs pipeline to exist
- **US3 (Phase 5)**: Depends on US1 — smoke test validates pipeline output; can run in parallel with US2
- **US4 (Phase 6)**: Depends on US2 — error tests exercise CLI flags
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

- **US1 (P1)**: Can start after Phase 2 — no dependencies on other stories
- **US2 (P1)**: Depends on US1 (pipeline must exist for CLI dispatch)
- **US3 (P1)**: Depends on US1 (smoke test validates pipeline); can run in parallel with US2
- **US4 (P2)**: Depends on US2 (error tests need CLI wiring and required flags)

### Within Each User Story

- Tests MUST be written and FAIL before implementation (TDD)
- Implement minimal code to pass tests
- Verify tests pass after implementation
- Story complete before moving to next priority

### Parallel Opportunities

- **Phase 1**: T001 + T002 can run in parallel (different files)
- **Phase 3**: T004 + T005 can run in parallel (different files: src/pipeline.rs vs tests/catalog_pipeline_test.rs)
- **Phase 4**: T008 is independent (tests can be written before implementation)
- **Phase 5**: T012 + T013 + T014 can run in parallel (independent test assertions)
- **Phase 7**: T018 + T019 can run in parallel (independent checks)
- **Cross-phase**: US2 (Phase 4) and US3 (Phase 5) can start in parallel after US1 completes

---

## Parallel Example: User Story 1

```bash
# Launch unit tests and smoke test in parallel (TDD — write tests first):
Task: "Write unit tests for write_output in src/pipeline.rs"
Task: "Write smoke test in tests/catalog_pipeline_test.rs"

# Then implement sequentially (write_output before orchestrator):
Task: "Implement write_output function in src/pipeline.rs"
Task: "Implement run_catalog_pipeline orchestrator in src/pipeline.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (2 tasks — trivial)
2. Complete Phase 2: Foundational (1 task — fixture)
3. Complete Phase 3: User Story 1 (4 tasks — pipeline + tests)
4. **STOP and VALIDATE**: Run pipeline programmatically, verify OSCAL JSON output
5. Core value delivered — all 12 pipeline stages wired together

### Incremental Delivery

1. Setup + Foundational → Infrastructure ready
2. **US1** → Pipeline works programmatically → Core value (MVP)
3. **US2** → CLI works → User-facing capability
4. **US3** → Smoke test validates → Engineering confidence (MS-2 exit criteria met)
5. **US4** → Error reporting → Usability polish
6. Each story adds value without breaking previous stories

---

## Notes

- [P] tasks = different files, no dependencies on incomplete tasks
- [Story] label maps task to specific user story for traceability
- AR guardrail: DO NOT modify existing pipeline stage code (WI-2 through WI-12) — call functions only
- AR guardrail: DO NOT use trait objects or dynamic dispatch — direct function calls
- D1 (OscalMetadata bridging): Map real metadata fields to placeholder strings in orchestrator
- D2 (WI-8 stub): Pass `&[]` to generate_back_matter, produces empty back matter
- Existing tests in tests/cli_integration.rs will break when --strategy/--format become required (T011 fixes them)
- Existing test convert_valid_md_outputs_json asserts old ConvertOutput format — must update for OSCAL Catalog format (T011)
