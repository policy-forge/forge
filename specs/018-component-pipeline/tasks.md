# Tasks: End-to-End Component Definition Pipeline

**Input**: Design documents from `/specs/018-component-pipeline/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, quickstart.md
**Tests**: TDD mandatory per constitution principle IV — tests written before implementation

**Organization**: Tasks grouped by user story. Codebase is ~85% complete; tasks focus on gaps G-1 through G-7 from plan.md.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story (US1, US2, US3, US4)
- Exact file paths in descriptions

---

## Phase 1: Foundational (Cross-Cutting Fixes)

**Purpose**: Address cross-cutting gaps (EC-1, SEC-1, S-3) that affect all user stories. Must complete before story-specific tests.

**Gaps addressed**: G-4 (SEC-1 absolute path), G-5 (EC-1 format default), G-3 (S-3 logging)

- [X] T001 [P] Write unit test verifying OutputFormat defaults to Json when --format is omitted from clap parsing in src/cli/mod.rs (T-EC1-02, EC-1)
- [X] T002 [P] Write integration test verifying source-file prop contains filename only with no path separators in tests/component_pipeline_test.rs (T-SEC1, SEC-1)
- [X] T003 Add default_value = "json" to --format clap attribute in src/cli/mod.rs (G-5, EC-1) — makes T001 pass
- [X] T004 Use input_path.file_name() instead of input_path.display() for source_file parameter in src/pipeline.rs (G-4, SEC-1) — makes T002 pass
- [X] T005 Add tracing::info!() for pipeline stage progress (ingest, build, serialize) in src/pipeline.rs run_component_pipeline function (G-3, S-3)

**Checkpoint**: Cross-cutting fixes complete. All existing tests still pass. `cargo test` green.

---

## Phase 2: US-1 & US-2 — Full Pipeline + Traceability (Priority: P1) — MVP

**Goal**: Verify the already-wired end-to-end pipeline works correctly via CLI integration tests. Core pipeline (M-1 through M-8) is already implemented; these tests provide regression coverage.

**Independent Test**: Run `forge convert tests/fixtures/full_policy.md --strategy component --source-profile <path> --format json` and verify valid Component Definition JSON with documentary component, implemented-requirements, trace props, and back matter.

- [X] T006 [P] [US1] Write CLI integration test: `forge convert --strategy component --source-profile <path> --format json` produces valid Component Definition JSON to stdout with documentary component, control-implementations, OSCAL metadata fields (uuid, title, last-modified, version, oscal-version), and back-matter resources in tests/cli_integration.rs (AC-1, AC-2, AC-3, AC-5, M-3, M-4, M-5, M-7)
- [X] T007 [P] [US1] Write CLI integration test: --format omitted with --strategy component produces JSON output by default in tests/cli_integration.rs (T-EC1, EC-1)
- [X] T008 [P] [US1] Write CLI integration test: --output <path> writes Component Definition JSON to file instead of stdout in tests/cli_integration.rs (AC-6, M-8)
- [X] T009 [P] [US2] Write CLI integration test: output implemented-requirements contain source-file, source-section, source-line props in tests/cli_integration.rs (AC-4, M-6, SEC-1)
- [X] T022 [P] [US1] Write CLI integration test: input Markdown with zero extractable requirements produces Component Definition with empty control-implementations and warning on stderr in tests/cli_integration.rs (EC-2)
- [X] T023 [P] [US1] Write CLI integration test: source profile with no matching control IDs produces implemented-requirements without control-id references in tests/cli_integration.rs (EC-3)

**Checkpoint**: US-1 and US-2 verified via CLI tests including edge cases. All tests pass. MVP complete.

---

## Phase 3: US-3 — Component Strategy Without Source Profile (Priority: P2)

**Goal**: Allow `--strategy component` without `--source-profile`, producing a Component Definition with empty control-implementations and emitting a warning to stderr.

**Independent Test**: Run `forge convert tests/fixtures/full_policy.md --strategy component --format json` without `--source-profile` and verify output has empty control-implementations and stderr contains warning.

**Gaps addressed**: G-1 (optional source-profile), G-7 (update existing test)

### Tests for US-3

> Write these tests FIRST. They will FAIL until implementation tasks are complete.

- [X] T010 [P] [US3] Write integration test: run_component_pipeline with source_profile: None produces Component Definition with empty control-implementations in tests/component_pipeline_test.rs (T-S1-03, AC-7)
- [X] T011 [P] [US3] Write CLI integration test: --strategy component without --source-profile produces valid JSON output and warning on stderr in tests/cli_integration.rs (T-S1-02, SEC-6, AC-7)

### Implementation for US-3

- [X] T012 [US3] Change run_component_pipeline signature from source_profile: &str to source_profile: Option<&str> and remove empty-string validation in src/pipeline.rs (G-1, S-1)
- [X] T013 [US3] Update Strategy::Component handler in src/cli/convert.rs: emit tracing::warn!() when source_profile is None instead of returning error; pass Option<&str> to pipeline; keep empty-string validation error (G-1, S-1, SEC-6)
- [X] T014 [US3] Update existing test convert_strategy_component_shows_rejection_error in tests/cli_integration.rs to expect success with warning instead of error for missing --source-profile (T-UPD-01, G-7)

**Checkpoint**: US-3 complete. `--strategy component` works without `--source-profile`. Warning emitted. All tests pass.

---

## Phase 4: US-4 — Source Profile Validation (Priority: P2)

**Goal**: Validate that `--source-profile` path exists and is a regular file before pipeline execution, producing a descriptive error and non-zero exit on invalid paths.

**Independent Test**: Run `forge convert tests/fixtures/full_policy.md --strategy component --source-profile nonexistent.json` and verify descriptive error and non-zero exit code.

**Gaps addressed**: G-2 (file validation), SEC-3

### Tests for US-4

> Write these tests FIRST. They will FAIL until implementation task is complete.

- [X] T015 [P] [US4] Write CLI integration test: non-existent --source-profile path produces descriptive error and exits non-zero in tests/cli_integration.rs (T-S2-03, AC-8, SEC-3)
- [X] T016 [P] [US4] Write CLI integration test: directory path as --source-profile produces descriptive error and exits non-zero in tests/cli_integration.rs (SEC-3)

### Implementation for US-4

- [X] T017 [US4] Add source-profile file validation in src/cli/convert.rs Strategy::Component handler: check path.exists() and path.is_file() before calling pipeline; return ForgeError::Validation with descriptive message on failure (G-2, SEC-3)

**Checkpoint**: US-4 complete. Invalid source-profile paths produce clear errors. All tests pass.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Final quality gates and validation

- [X] T018 [P] Run cargo fmt --check and fix any formatting issues
- [X] T019 [P] Run cargo clippy -- -D warnings and fix any warnings
- [X] T020 Run full test suite (cargo test) and verify all tests pass including new tests
- [X] T021 Validate quickstart.md scenarios against implemented CLI behavior

---

## Dependencies & Execution Order

### Phase Dependencies

- **Foundational (Phase 1)**: No dependencies — start immediately
- **US-1 & US-2 (Phase 2)**: Depends on Phase 1 (needs EC-1 and SEC-1 fixes for correct assertions)
- **US-3 (Phase 3)**: Depends on Phase 1 (foundational fixes) — independent of Phase 2
- **US-4 (Phase 4)**: Depends on Phase 3 (source_profile must be optional before adding validation for Some case)
- **Polish (Phase 5)**: Depends on all prior phases

### User Story Dependencies

- **US-1 & US-2 (P1)**: Can start after Phase 1. Already implemented — tests verify existing behavior.
- **US-3 (P2)**: Can start after Phase 1. Independent of US-1/US-2 tests.
- **US-4 (P2)**: Depends on US-3 completion (T012/T013 must change handler before T017 adds validation to it).

### Within Each Phase

- TDD: Tests (T001, T002, T010, T011, T015, T016) written and verified to FAIL before implementation
- Implementation tasks make corresponding tests PASS
- All [P] tasks within a phase can run in parallel

### Parallel Opportunities

- **Phase 1**: T001 and T002 in parallel (different files); T003 and T004 can follow in parallel
- **Phase 2**: All 6 tests (T006–T009, T022–T023) in parallel (same file, different test functions)
- **Phase 3**: T010 and T011 in parallel (different files); T012 and T013 sequential (T012 changes signature, T013 updates call site)
- **Phase 4**: T015 and T016 in parallel (same file, different test functions)
- **Phase 5**: T018 and T019 in parallel

---

## Parallel Example: Phase 2

```bash
# Launch all CLI integration tests for US-1 and US-2 together:
Task: "Write CLI test: full pipeline produces valid Component Definition in tests/cli_integration.rs"
Task: "Write CLI test: --format omitted defaults to JSON in tests/cli_integration.rs"
Task: "Write CLI test: --output writes to file in tests/cli_integration.rs"
Task: "Write CLI test: trace props present in output in tests/cli_integration.rs"
Task: "Write CLI test: zero requirements → empty control-implementations in tests/cli_integration.rs"
Task: "Write CLI test: no control IDs → unmapped requirements in tests/cli_integration.rs"
```

---

## Implementation Strategy

### MVP First (Phase 1 + Phase 2)

1. Complete Phase 1: Foundational cross-cutting fixes (5 tasks)
2. Complete Phase 2: US-1 & US-2 verification tests (6 tasks)
3. **STOP and VALIDATE**: `cargo test` — all tests green, core pipeline verified
4. This is the MVP: full pipeline works end-to-end with CLI integration coverage

### Incremental Delivery

1. Phase 1 + Phase 2 → Core pipeline verified (MVP)
2. Add Phase 3 (US-3) → Optional source-profile support → Test independently
3. Add Phase 4 (US-4) → Source profile validation → Test independently
4. Phase 5 → Final quality gates

### Requirement Traceability

| Req ID | Task(s) | Phase |
|--------|---------|-------|
| M-1 | T006 (verify) | 2 |
| M-2 | T013 | 3 |
| M-3 | T006 (verify) | 2 |
| M-4 | T006 (verify) | 2 |
| M-5 | T006 (verify) | 2 |
| M-6 | T009 (verify) | 2 |
| M-7 | T006 (verify) | 2 |
| M-8 | T008 (verify) | 2 |
| S-1 | T010, T011, T012, T013, T014 | 3 |
| S-2 | T015, T016, T017 | 4 |
| S-3 | T005 | 1 |
| EC-1 | T001, T003, T007 | 1, 2 |
| EC-2 | T022 (verify) | 2 |
| EC-3 | T023 (verify) | 2 |
| SEC-1 | T002, T004, T009 | 1, 2 |
| SEC-3 | T015, T016, T017 | 4 |
| SEC-4 | Deferred (R-2, W-3) | — |
| SEC-6 | T011, T013 | 3 |

---

## Notes

- [P] tasks = different files or independent test functions, no dependencies
- [Story] label maps task to user story for traceability
- Codebase is ~85% complete — M-1 through M-8 already implemented; tasks focus on gaps
- Constitution IV (TDD): Tests written and verified to FAIL before implementation
- No new dependencies — all changes use existing crate infrastructure
- 4 source files modified: src/cli/mod.rs, src/cli/convert.rs, src/pipeline.rs, (test files)
- 2 test files modified: tests/component_pipeline_test.rs, tests/cli_integration.rs
- **SEC-4 deferral**: SEC-4 requires validating that `--source-profile` is parseable JSON. The current design treats the profile path as a reference string — profile content is not parsed in the pipeline (per AR anti-pattern guidance: "Don't parse the source profile eagerly at CLI argument parsing time"). JSON content validation is deferred to the WI that implements profile resolution (W-3). See research.md R-2 for rationale.
- **EC-4/EC-5 coverage**: EC-4 (--output to non-existent directory) is already handled by existing `write_output()` validation (SEC-5). EC-5 (--source-profile with --strategy catalog) is inherent behavior — the catalog strategy ignores the flag. Neither requires a new task.
- **--verbose flag**: The `--verbose` CLI flag already exists (src/cli/mod.rs:21-23) and controls tracing subscriber output levels. T005's `tracing::info!()` calls are visible when `--verbose` is set, satisfying S-3.
