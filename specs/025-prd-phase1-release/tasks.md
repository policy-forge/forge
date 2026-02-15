# Tasks: WI-25 Phase 1 Release

**Input**: Design documents from `/specs/025-prd-phase1-release/`
**Prerequisites**: plan.md, PRD `docs/PRD/025-prd-phase1-release.md`, AR `docs/AR/025-AR-phase1-release.md`, SEC `docs/SEC/020-sec-validation-error-reporting.md`

**Organization**: Tasks are grouped by user story (from PRD) to enable independent implementation and testing of each story. This is a testing/polish/release sprint — no new features.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1–US5)
- All file paths are relative to repository root

---

## Phase 1: Setup

**Purpose**: Verify baseline and ensure test infrastructure is ready

- [x] T001 Run baseline verification: `cargo fmt --check && cargo clippy -- -D warnings && cargo test` — confirm 660+ tests pass with zero violations
- [x] T002 Verify E2E test fixtures exist in tests/fixtures/ — confirm sample_policy.md (or equivalent policy fixture with sections, requirements, and citations) is available for integration tests

**Checkpoint**: Baseline green — integration test development can begin

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Create the new integration test file that all user story phases depend on

**Warning**: No user story work can begin until this phase is complete

- [x] T003 Create tests/e2e_release_test.rs with module skeleton: shared helper functions for building binary path (`env!("CARGO_BIN_EXE_forge")`), running forge convert/validate via `std::process::Command`, parsing JSON output, and loading test fixtures from tests/fixtures/

**Checkpoint**: Foundation ready — user story implementation can now begin

---

## Phase 3: User Story 1 — End-to-End Catalog Conversion (Priority: P1) MVP

**Goal**: Verify the complete Markdown → OSCAL Catalog → validate pipeline works end-to-end, covering parent PRD M-1, M-3, M-5, M-6, M-7, M-8, M-9, M-10, M-11 and acceptance criteria AC-1 through AC-10.

**Independent Test**: Run `cargo test e2e_release` and verify all catalog-related integration tests pass. Run `forge convert policy.md --strategy catalog --format json --output /tmp/catalog.json && forge validate /tmp/catalog.json` manually.

### Integration Tests for User Story 1

- [x] T004 [P] [US1] Write test_m1_structural_hierarchy_extraction in tests/e2e_release_test.rs — convert sample MD, verify groups present in Catalog JSON matching source heading hierarchy (M-1, AC-1)
- [x] T005 [P] [US1] Write test_m3_valid_oscal_catalog_json in tests/e2e_release_test.rs — convert MD, parse output as JSON, verify top-level catalog structure with groups, controls, metadata, back-matter (M-3, AC-3)
- [x] T006 [P] [US1] Write test_m5_metadata_fields_present in tests/e2e_release_test.rs — verify uuid, title, last-modified, version, oscal-version fields present in catalog metadata (M-5, AC-5)
- [x] T007 [P] [US1] Write test_m6_validate_generated_catalog in tests/e2e_release_test.rs — convert MD to temp file, run `forge validate` on output, verify exit code 0 and success message (M-6, AC-6)
- [x] T008 [P] [US1] Write test_m7_json_output_format in tests/e2e_release_test.rs — convert MD, verify output is valid JSON parseable by serde_json (M-7, AC-7)
- [x] T009 [P] [US1] Write test_m8_deterministic_uuids in tests/e2e_release_test.rs — convert same input twice, compare both outputs byte-for-byte to verify deterministic UUIDs (M-8, AC-8)
- [x] T010 [P] [US1] Write test_m9_citations_in_back_matter in tests/e2e_release_test.rs — convert MD with citations, verify back_matter.resources array contains citation entries with rlinks (M-9, AC-9)
- [x] T011 [P] [US1] Write test_m10_traceability_props in tests/e2e_release_test.rs — convert MD, verify trace props (source-heading, source-document) on controls (M-10, AC-10)
- [x] T012 [P] [US1] Write test_m11_no_arbitrary_remarks in tests/e2e_release_test.rs — convert MD, verify no arbitrary remarks fields on controls that contain unstructured prose (M-11)
- [x] T013 [P] [US1] Write test_error_message_missing_file in tests/cli_integration.rs — run `forge convert nonexistent.md`, verify descriptive error with file path, no panic or stack trace (S-3, AC-11, SEC-4)
- [x] T014 [P] [US1] Write test_error_message_invalid_json_for_validate in tests/cli_integration.rs — run `forge validate` on a non-JSON file, verify descriptive error message (EC-5, SEC-4)
- [x] T015 [US1] Verify golden-file accuracy >95% by running `cargo test golden_file` in tests/golden_file_tests.rs — confirm accuracy metric meets MS-4 threshold (M-3)
- [x] T016 [US1] Run `cargo test` and verify all US-1 integration tests pass with zero failures

**Checkpoint**: User Story 1 complete — full Catalog E2E pipeline verified with M-requirement traceability

---

## Phase 4: User Story 2 — End-to-End Component Definition Conversion (Priority: P1)

**Goal**: Verify the Markdown → OSCAL Component Definition pipeline works end-to-end, covering parent PRD M-2, M-4, M-6.

**Independent Test**: Run `cargo test e2e_release` and verify all component-related integration tests pass. Run `forge convert policy.md --strategy component --format json` and verify valid Component Definition output.

### Integration Tests for User Story 2

- [x] T017 [P] [US2] Write test_m2_atomize_compound_statements in tests/e2e_release_test.rs — convert MD with "must X and must Y" compound statements, verify separate controls in output (M-2, AC-2)
- [x] T018 [P] [US2] Write test_m4_valid_component_definition in tests/e2e_release_test.rs — convert with component strategy, verify component-definition structure with components and implemented-requirements (M-4, AC-4)
- [x] T019 [P] [US2] Write test_m6_validate_generated_component in tests/e2e_release_test.rs — convert to Component Definition temp file, run `forge validate` on output, verify exit code 0 (M-6, AC-6)
- [x] T020 [US2] Run `cargo test` and verify all US-2 integration tests pass with zero failures

**Checkpoint**: User Stories 1 AND 2 complete — both conversion strategies verified end-to-end

---

## Phase 5: User Story 3 — CLI Help and Discoverability (Priority: P1)

**Goal**: Verify all `--help` output is comprehensive and accurate for first-time users, covering PRD M-5 and acceptance criteria AC-5.

**Independent Test**: Run `forge --help`, `forge convert --help`, `forge validate --help` and verify each displays complete, accurate usage information including all subcommands, arguments, and options.

### Integration Tests for User Story 3

- [x] T021 [P] [US3] Write test_help_text_lists_all_subcommands in tests/cli_integration.rs — run `forge --help`, verify output contains "convert", "validate", "--verbose", "--quiet" (M-5, AC-5, EC-3)
- [x] T022 [P] [US3] Write test_convert_help_lists_all_options in tests/cli_integration.rs — run `forge convert --help`, verify output contains input, --strategy, --format, --output, --source-profile, --max-size (M-5)
- [x] T023 [P] [US3] Write test_validate_help_lists_all_options in tests/cli_integration.rs — run `forge validate --help`, verify output contains input path, --schema-type, --format (M-5)
- [x] T024 [P] [US3] Write test_version_flag in tests/cli_integration.rs — run `forge --version`, verify output contains "forge" and version string

### CLI Polish for User Story 3

- [x] T025 [US3] Enhance #[command(about)] with richer FORGE description and add #[command(long_about)] with pipeline overview in src/cli/mod.rs
- [x] T026 [US3] Review and enhance help attributes on all convert subcommand args (strategy, format, output, max-size, source-profile) in src/cli/mod.rs
- [x] T027 [US3] Review and enhance help attributes on all validate subcommand args (input, schema-type, format) in src/cli/mod.rs
- [x] T028 [US3] Run `cargo test` and verify all US-3 tests pass — help text accurate after polish

**Checkpoint**: User Story 3 complete — all --help output is comprehensive and discoverable

---

## Phase 6: User Story 4 — Verbose and Quiet Output Control (Priority: P2)

**Goal**: Verify --verbose and --quiet flags work correctly for debugging and scripted workflows, covering PRD S-1 and acceptance criteria AC-8, AC-9.

**Independent Test**: Run `forge -v convert policy.md --strategy catalog --format json` and verify pipeline stage INFO messages appear on stderr. Run with `-q` and verify only the OSCAL artifact appears on stdout.

### Integration Tests for User Story 4

- [x] T029 [P] [US4] Write test_verbose_flag_shows_pipeline_stages in tests/cli_integration.rs — run with --verbose, verify stderr contains pipeline stage messages like "Ingesting" or tracing output (S-1, AC-8, SEC-7)
- [x] T030 [P] [US4] Write test_quiet_flag_suppresses_output in tests/cli_integration.rs — run with --quiet, verify stderr has no informational messages, only OSCAL artifact on stdout (S-1, AC-9)
- [x] T031 [P] [US4] Write test_verbose_quiet_conflict_error in tests/cli_integration.rs — run with both --verbose and --quiet, verify clear conflict error message and non-zero exit code (S-1, EC-4)

### Verification for User Story 4

- [x] T032 [US4] Verify verbose→debug, quiet→error, default→warn filter mapping in src/main.rs — confirm tracing_subscriber filter level wiring is correct
- [x] T033 [US4] Verify --verbose description says "Enable verbose output showing pipeline stage information" and --quiet says "Suppress all non-essential output" in src/cli/mod.rs
- [x] T034 [US4] Run `cargo test` and verify all US-4 tests pass with zero failures

**Checkpoint**: User Story 4 complete — verbose/quiet output control verified

---

## Phase 7: User Story 5 — README Usage Examples (Priority: P2)

**Goal**: Update README with verified usage examples so new users can start using FORGE immediately, covering PRD S-2, C-1 and acceptance criteria AC-10.

**Independent Test**: Copy each README example, run it against the built binary with sample data, and verify it produces the described output.

### Implementation for User Story 5

- [x] T035 [US5] Update status table marking all Phase 1 pipeline stages (Ingestion, Structural Extraction, Domain Model, Atomization, UUID Generation, Citation Extraction, Catalog Pipeline, Component Pipeline, Traceability, Schema Validation, Error Handling, Performance) as "Done" in README.md
- [x] T036 [US5] Add Usage section with verified examples for: `forge convert policy.md --strategy catalog --format json`, `forge convert policy.md --strategy component --format json`, and `forge validate catalog.json` in README.md (S-2, AC-10)
- [x] T037 [US5] Add Quick Start section with single-command example and note about sample policy file location (tests/fixtures/sample_policy.md) in README.md (C-1, EC-7)
- [x] T038 [US5] Update Project Structure section to reflect current source layout (all modules in src/, test files in tests/) in README.md
- [x] T039 [US5] Verify each README example runs against built binary: `cargo run -- convert tests/fixtures/sample_policy.md --strategy catalog --format json` and other examples produce expected output without errors

**Checkpoint**: User Story 5 complete — README is accurate with working examples

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Error message audit, security verification, release packaging, and final gate

### Error Message Audit (S-3, SEC-4)

- [x] T040 [P] Audit all ForgeError variants for consistent "Error: {descriptive message}" format in src/error.rs
- [x] T041 [P] Audit error handling paths for actionable guidance (e.g., "file not found: {path}") in src/cli/convert.rs
- [x] T042 [P] Audit error handling paths for actionable guidance in src/cli/validate.rs
- [x] T043 Verify no internal Rust module paths (e.g., "src::parse::clauses") appear in user-facing error messages across src/error.rs, src/cli/convert.rs, src/cli/validate.rs (SEC-4)

### Release Packaging (C-2)

- [x] T044 Create .github/workflows/release.yml with cross-platform build matrix — configure targets: linux-x64, macos-x64, macos-arm64, windows-x64 (manual workflow instead of cargo-dist — simpler for single-binary project)
- [x] T045 Review generated .github/workflows/release.yml — verify workflow triggers on tag push matching `v*` and builds all target platforms

### Release Gate Verification (M-6, M-7)

- [x] T046 Run MS-4 exit criteria verification checklist: (1) cargo fmt --check = 0 violations, (2) cargo clippy -- -D warnings = 0 warnings, (3) cargo test = all pass, (4) golden-file >95%, (5) forge validate works on generated artifacts, (6) all M-1 through M-11 verified, (7) all AC-1 through AC-10 verified, (8) README examples verified, (9) forge --version = "forge 0.1.0"
- [x] T047 Run full quality gate suite (`cargo fmt --check && cargo clippy -- -D warnings && cargo test`) and confirm zero violations before merge

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup (T001 baseline green) — BLOCKS all user stories
- **US-1 (Phase 3)**: Depends on Foundational (T003 test skeleton) — **MVP, do first**
- **US-2 (Phase 4)**: Depends on Foundational (T003) — can run in parallel with US-1
- **US-3 (Phase 5)**: Depends on Foundational (T003) — can run in parallel with US-1/US-2
- **US-4 (Phase 6)**: Depends on Foundational (T003) — can run in parallel with US-1/US-2/US-3
- **US-5 (Phase 7)**: Depends on US-3 completion (CLI polish finalized before documenting) — run after Phase 5
- **Polish (Phase 8)**: Depends on all user stories being complete

### User Story Dependencies

- **US-1 (P1)**: After Foundational → No dependencies on other stories — **MVP scope**
- **US-2 (P1)**: After Foundational → Independent of US-1 (different test file sections, different strategy)
- **US-3 (P1)**: After Foundational → Independent (tests/cli_integration.rs extensions + src/cli/mod.rs polish)
- **US-4 (P2)**: After Foundational → Independent (tests/cli_integration.rs extensions + src/main.rs verification)
- **US-5 (P2)**: After US-3 (CLI polish must be finalized before documenting help text in README)

### Within Each User Story

- Integration tests written first (verify requirement coverage)
- Implementation/polish tasks after tests (fix any gaps found by tests)
- Verification run last (cargo test for the story)

### Parallel Opportunities

- All Setup tasks (T001, T002) can run in parallel
- All [P]-marked tests within a user story can run in parallel (same test file, different test functions)
- US-1, US-2, US-3, and US-4 can all start in parallel once Foundational completes
- Error audit tasks (T040, T041, T042) can run in parallel (different source files)
- US-5 README tasks (T035, T036, T037, T038) operate on different sections but same file — run sequentially

---

## Parallel Example: User Story 1

```bash
# After T003 (foundational) is complete, launch all US-1 tests in parallel:
Task: "Write test_m1_structural_hierarchy_extraction in tests/e2e_release_test.rs"
Task: "Write test_m3_valid_oscal_catalog_json in tests/e2e_release_test.rs"
Task: "Write test_m5_metadata_fields_present in tests/e2e_release_test.rs"
Task: "Write test_m6_validate_generated_catalog in tests/e2e_release_test.rs"
Task: "Write test_m7_json_output_format in tests/e2e_release_test.rs"
Task: "Write test_m8_deterministic_uuids in tests/e2e_release_test.rs"
Task: "Write test_m9_citations_in_back_matter in tests/e2e_release_test.rs"
Task: "Write test_m10_traceability_props in tests/e2e_release_test.rs"
Task: "Write test_m11_no_arbitrary_remarks in tests/e2e_release_test.rs"
# CLI error tests in parallel (different file):
Task: "Write test_error_message_missing_file in tests/cli_integration.rs"
Task: "Write test_error_message_invalid_json_for_validate in tests/cli_integration.rs"
```

## Parallel Example: User Stories 1–4 (Team Strategy)

```bash
# After Foundational (T003) completes, four agents can work in parallel:
Agent A: Phase 3 (US-1) — Catalog E2E tests in tests/e2e_release_test.rs
Agent B: Phase 4 (US-2) — Component E2E tests in tests/e2e_release_test.rs
Agent C: Phase 5 (US-3) — CLI help tests in tests/cli_integration.rs + polish in src/cli/mod.rs
Agent D: Phase 6 (US-4) — Verbose/quiet tests in tests/cli_integration.rs + verify src/main.rs
# Note: Agents B, C, D should avoid conflicting edits in shared files
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T002)
2. Complete Phase 2: Foundational (T003)
3. Complete Phase 3: User Story 1 — Catalog E2E (T004–T016)
4. **STOP and VALIDATE**: All 9 catalog M-requirement tests pass, golden-file >95%, error tests pass
5. This alone verifies M-1, M-3, M-5, M-6, M-7, M-8, M-9, M-10, M-11 for Catalog strategy

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. US-1 (Catalog E2E) → Core pipeline verified → **MVP**
3. US-2 (Component E2E) → Both strategies verified
4. US-3 (CLI Help) → Help text polished
5. US-4 (Verbose/Quiet) → Output control verified
6. US-5 (README) → User documentation complete
7. Polish → Error audit, release packaging, final gate
8. **Tag v0.1.0 ONLY after T046 and T047 pass**

---

## Traceability Summary

| PRD Req | User Story | Task IDs | Verification |
|---------|-----------|----------|-------------|
| M-1 | US-1 | T004 | test_m1_structural_hierarchy_extraction |
| M-2 | US-2 | T017 | test_m2_atomize_compound_statements |
| M-3 | US-1 | T005, T015 | test_m3_valid_oscal_catalog_json, golden-file >95% |
| M-4 | US-2 | T018 | test_m4_valid_component_definition |
| M-5 | US-1, US-3 | T006, T021–T024 | test_m5_metadata_fields_present, help text tests |
| M-6 | US-1, US-2 | T007, T019, T046 | test_m6_validate_generated_catalog/component |
| M-7 | US-1 | T008 | test_m7_json_output_format |
| S-1 | US-4 | T029–T031 | verbose/quiet/conflict tests |
| S-2 | US-5 | T036 | README usage examples |
| S-3 | US-1 | T013, T014, T040–T043 | Error message tests + audit |
| C-1 | US-5 | T037 | Quick Start section |
| C-2 | — | T044, T045 | Release packaging with manual workflow |
| SEC-4 | US-1 | T013, T014, T043 | No Rust paths in errors |
| SEC-7 | US-4 | T029 | Errors to stderr |
| M-8 (parent) | US-1 | T009 | test_m8_deterministic_uuids |
| M-9 (parent) | US-1 | T010 | test_m9_citations_in_back_matter |
| M-10 (parent) | US-1 | T011 | test_m10_traceability_props |
| M-11 (parent) | US-1 | T012 | test_m11_no_arbitrary_remarks |

---

## Notes

- [P] tasks = different files or independent test functions, no dependencies
- [Story] label maps task to specific user story for traceability
- All integration tests use existing `std::process::Command` pattern (research R1 — no assert_cmd)
- No new runtime dependencies — manual release workflow requires no additional tools
- Tag v0.1.0 ONLY after ALL MS-4 exit criteria verified (T046)
- README examples MUST be tested against actual binary before committing (PRD R-4)
- Error messages must not expose internal Rust paths or module names (SEC-4)
