# Tasks: Assessment Plan Scaffolding — Controls (WI-41)

**Input**: Design documents from `/specs/041-assessment-plan-controls/`
**Prerequisites**: plan.md ✅, spec.md ✅, prd.md ✅, ar.md ✅, sec.md ✅, data-model.md ✅, contracts/assessment_plan.rs ✅, quickstart.md ✅

**Tests**: TDD is **mandatory** per plan.md constraints. Test tasks are included for all ACs and ECs.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no unmet dependencies)
- **[Story]**: Which user story this task belongs to ([US1], [US2], [US3])
- **TDD rule**: Write test first (RED), implement (GREEN), then verify
- Exact file paths are shown for each task

---

## Phase 1: Setup (Read Existing Interfaces)

**Purpose**: Confirm integration points before modifying any file. No new project structure is needed — this is a single Rust crate with all dependencies already present.

- [X] T001 Verify that src/pipeline.rs exports `run_catalog_pipeline` and `run_component_pipeline`, that src/cli/mod.rs defines `Commands::Convert`, and that src/error.rs has no pre-existing `AssessmentPlanBuild` variant — note any signature divergence from contracts/assessment_plan.rs as a blocker before proceeding

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Error type and UUID namespace constant that all subsequent phases depend on. Both are foundational — no user story work can begin without them.

**⚠️ CRITICAL**: Phases 3–5 cannot start until this phase is complete.

- [X] T002 Add `ForgeError::AssessmentPlanBuild(String)` variant with `#[error("Assessment plan build error: {0}")]` and map to exit code 2 in `exit_code()`, plus inline unit tests for `Display` and `exit_code` in src/error.rs
- [X] T003 [P] Add `pub const ASSESSMENT_PLAN_NAMESPACE: Uuid` constant derived from `Uuid::new_v5(&FORGE_NAMESPACE_UUID, b"assessment-plan")` with derivation comment and inline verification test `assessment_plan_namespace_matches_derivation` in src/uuid.rs

**Checkpoint**: `cargo test --lib` passes — error type and UUID namespace are verified.

---

## Phase 3: User Story 1 — Generate AP with Reviewed Controls (Priority: P1) 🎯 MVP

**Goal**: Produce a structurally valid OSCAL Assessment Plan JSON with reviewed-controls populated from conversion output control IDs.

**Independent Test**: Call `build_assessment_plan` with 10 control IDs and verify `reviewed-controls.control-selections[0].include-controls` contains all 10. Verify `metadata.title`, `metadata.version = "1.0.0"`, `metadata.oscal-version = "1.2.0"`, and `reviewed-controls.description` references the policy title.

### Tests for User Story 1 (Write FIRST — must FAIL before T005)

- [X] T004 [US1] Write failing inline unit tests for AC-1 (root key `"assessment-plan"`), AC-2 (metadata fields present and correct), AC-4 (reviewed-controls with 10 control IDs in include-controls), AC-7 (reviewed-controls.description references policy title), EC-1 (zero controls → empty include-controls array), EC-3 (duplicate IDs → deduplicated output), EC-4 (JSON parseable with hyphenated root key), EC-5/unit (call builder twice with different `control_ids` → different UUIDs) in src/oscal/assessment_plan.rs — **Rust TDD ordering**: create a minimal stub module with empty struct skeletons and `todo!()` function bodies first so tests compile, then write the failing tests against those stubs, then proceed to T005–T007 for full implementation; **EC-1 tracing**: assert the output `include-controls` array is empty — `tracing::warn!` emission is verified by E2E log inspection only, no in-process subscriber required; **EC-5 scope**: this is the unit-level EC-5 test (direct builder call); T017 covers the same invariant at integration level (full CLI invocation)

### Implementation for User Story 1

- [X] T005 [US1] Define all AP structs with `#[derive(Debug, Clone, Serialize)]` and OSCAL-compliant serde rename annotations: `AssessmentPlanEnvelope`, `AssessmentPlan`, `ApMetadata`, `ImportSsp`, `ReviewedControls`, `ApControlSelection`, `ApIncludeControl` in src/oscal/assessment_plan.rs
- [X] T006 [US1] Implement `pub fn build_assessment_plan(control_ids: &[String], import_ssp_href: &str, policy_title: &str) -> Result<AssessmentPlanEnvelope, ForgeError>` — validate non-empty href, sort+dedup control IDs, call `assemble_metadata`, generate UUID v5 from seed `format!("assessment-plan|{}|{}", sorted_ids.join(","), ssp_href)`, build and return `AssessmentPlanEnvelope` in src/oscal/assessment_plan.rs — **S-2**: call the shared `assemble_metadata()` function from WI-11 to produce the metadata block; do not duplicate metadata assembly inline
- [X] T007 [US1] Implement `pub fn derive_ap_output_path(input: &Path, primary_output: Option<&Path>) -> PathBuf` — output filename `{input_stem}-assessment-plan.json` in parent dir of `primary_output` or `./` if `None` in src/oscal/assessment_plan.rs — include two inline unit tests: (a) `primary_output = None` → `PathBuf::from("./policy-assessment-plan.json")`; (b) `primary_output = Some(Path::new("out/catalog.json"))` → `PathBuf::from("out/policy-assessment-plan.json")`
- [X] T008 [P] [US1] Write failing inline unit test for `collect_control_ids_from_catalog` (groups → controls depth-first), then implement `pub fn collect_control_ids_from_catalog(catalog: &OscalCatalog) -> Vec<String>` in src/oscal/catalog.rs
- [X] T009 [P] [US1] Write failing inline unit test for `collect_control_ids_from_component_def` (all implemented-requirements across all components), then implement `pub fn collect_control_ids_from_component_def(envelope: &ComponentDefinitionEnvelope) -> Vec<String>` in src/oscal/component_definition.rs
- [X] T010 [US1] Add `pub mod assessment_plan;` declaration and re-export `build_assessment_plan`, `derive_ap_output_path`, `AssessmentPlanEnvelope` in src/oscal/mod.rs

**Checkpoint**: `cargo test --lib` passes — US1 unit tests green. `build_assessment_plan` and both collectors are independently callable.

---

## Phase 4: User Story 2 — Link the SSP Reference (Priority: P1)

**Goal**: Wire `--import-ssp <path>` CLI flag through the pipeline to the AP builder, producing `import-ssp.href` in the output. When `--import-ssp` is omitted, AP generation is skipped and convert completes normally (backward compatible). Provide an actionable error for empty-string SSP path values.

**Independent Test**: Run `forge convert policy.md --strategy catalog --import-ssp ./ssp/system-ssp.json` and verify `import-ssp.href = "./ssp/system-ssp.json"`. Run without `--import-ssp` and verify AP generation is skipped and convert succeeds. Verify that `--import-ssp ""` exits with a descriptive validation error.

### Tests for User Story 2 (Write FIRST — must FAIL before T013)

- [X] T011 [US2] Write failing inline unit tests for AC-3 (`import-ssp.href` equals provided path) and EC-2 (empty or whitespace-only href → `ForgeError::Validation`) in src/oscal/assessment_plan.rs — **assertion style**: use substring checks (e.g. `.to_string().contains("import-ssp")`) not full-message equality (constitution VIII); **Note**: AC-5 (--import-ssp omitted → AP skipped, convert succeeds) is a pipeline/CLI behavior, not a builder-level behavior — it is covered at integration level by T019
- [X] T012 [P] [US2] Write failing CLI parsing unit tests for `--import-ssp`: flag present with value, flag absent (`import_ssp` is `None`), and flag with empty string (validates clap sees `Some("")`) in src/cli/mod.rs

### Implementation for User Story 2

- [X] T013 [US2] Extend `run_catalog_pipeline` signature with `import_ssp_href: Option<&str>` parameter; after writing primary artifact, if `Some(href)`: call `collect_control_ids_from_catalog`, call `build_assessment_plan`, derive AP path via `derive_ap_output_path`, write AP JSON; emit `tracing::warn!` if zero controls; emit `tracing::info!` with control count and href; update all existing callers to pass `None` in src/pipeline.rs
- [X] T014 [US2] Extend `run_component_pipeline` signature with `import_ssp_href: Option<&str>` parameter using same AP generation logic as `run_catalog_pipeline`; update all existing callers to pass `None` in src/pipeline.rs
- [X] T015 [US2] Add `#[arg(long)] import_ssp: Option<String>` field to `Commands::Convert` struct with doc comment in src/cli/mod.rs
- [X] T016 [US2] Add `pub import_ssp: Option<&'a str>` field to `ConvertOptions`; pass `opts.import_ssp` to `run_catalog_pipeline` and `run_component_pipeline` calls in `execute()`; in `execute_dispatch()`, if batch mode and `import_ssp.is_some()`, emit `tracing::warn!` and pass `None` to pipelines in src/cli/convert.rs — **observability**: the batch-mode `tracing::warn!` emission is verified by E2E log inspection only; no in-process tracing subscriber required in unit tests

**Checkpoint**: `cargo test` passes — US1 + US2 unit tests green. `cargo run -- convert tests/fixtures/sample_policy.md --strategy catalog --import-ssp ./ssp.json` writes an AP file.

---

## Phase 5: User Story 3 — Deterministic Assessment Plan UUIDs (Priority: P2)

**Goal**: Re-generating from identical input produces identical UUIDs across all runs. Changing any input changes the affected UUIDs.

**Independent Test**: Generate AP twice from same input; diff UUIDs (excluding `last-modified`) — no diff. Change one control ID in the input; verify document UUID changes.

### Tests for User Story 3 (Write FIRST — must FAIL before T018)

- [X] T017 [US3] Write failing integration tests for AC-6 (same input × 2 runs → identical UUIDs) and EC-5/integration (changed control set → different AP UUID) in tests/assessment_plan_test.rs — **scope**: full CLI invocations; EC-5 is also covered at unit level in T004 (direct builder calls with different inputs); this test verifies the CLI-to-builder pipeline preserves determinism end-to-end

### Implementation for User Story 3

- [X] T018 [US3] Verify UUID v5 seed formula in `build_assessment_plan` uses `format!("assessment-plan|{}|{}", sorted_ids.join(","), import_ssp_href)` hashed with `FORGE_NAMESPACE_UUID` — if T017 fails, correct the seed string in src/oscal/assessment_plan.rs

**Checkpoint**: `cargo test` passes — all three stories independently verifiable.

---

## Phase 6: Integration Tests & Polish

**Purpose**: End-to-end validation against the full CLI + quality gate enforcement.

- [X] T019 [P] Write integration test: AP file is written to `{output_dir}/{policy_stem}-assessment-plan.json` when `--import-ssp` is provided; AP file is NOT written when `--import-ssp` is omitted in tests/assessment_plan_test.rs
- [X] T020 [P] Write integration test: AP JSON contains all control IDs from `tests/fixtures/sample_policy.md` when converted with catalog strategy and `--import-ssp` in tests/assessment_plan_test.rs
- [X] T021 [P] Write integration test: empty `--import-ssp ""` exits with non-zero code and descriptive error message in tests/assessment_plan_test.rs — assert error output contains `"import-ssp"` or `"empty"` as a stable substring, not full-message equality (constitution VIII SHOULD)
- [X] T022 Run `cargo test` and fix any remaining failures
- [X] T023 [P] Run `cargo clippy -- -D warnings` and fix all warnings
- [X] T024 [P] Run `cargo fmt --check` and fix any formatting issues

**Checkpoint**: All tests green, zero clippy warnings, clean fmt. Feature complete.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 — **BLOCKS Phases 3–5**
- **US1 (Phase 3)**: Depends on Phase 2; no dependency on US2 or US3
- **US2 (Phase 4)**: Depends on Phase 3 (needs `build_assessment_plan`, collectors, `derive_ap_output_path`)
- **US3 (Phase 5)**: Depends on Phase 3 (needs `build_assessment_plan` with UUID v5 seed)
- **Polish (Phase 6)**: Depends on Phases 3–5

### User Story Dependencies

- **US1 (P1)**: Independent after Phase 2 — no dependency on US2 or US3
- **US2 (P1)**: Depends on US1 (uses the builder and collectors produced in Phase 3)
- **US3 (P2)**: Depends on US1 (tests the builder's UUID determinism from Phase 3)

### Within Each Phase

- TDD rule: Test tasks (T004, T011, T012, T017) MUST be written and verified FAILING before implementation tasks in the same phase
- Within US1: T005 (structs) → T006 (builder) → T007 (path helper) — sequential; T008 and T009 are parallel
- Within US2: T013, T014 (pipeline) must complete before T015, T016 (CLI) can be verified end-to-end
- T019–T021 integration tests can be written in parallel

### Parallel Opportunities

| Group | Tasks | Condition |
|-------|-------|-----------|
| Foundational | T002, T003 | Different files (error.rs, uuid.rs) |
| US1 collectors | T008, T009 | Different files (catalog.rs, component_definition.rs) |
| US2 tests | T011, T012 | Different files (assessment_plan.rs, cli/mod.rs) |
| US2 pipeline | T013, T014 | Same file — sequential within pipeline.rs |
| Polish | T019, T020, T021, T023, T024 | Independent |

---

## Parallel Example: User Story 1

```bash
# After T004 tests are written (RED):

# Run in parallel — different files, no conflict:
Task A: "Write failing test + implement collect_control_ids_from_catalog in src/oscal/catalog.rs"       # T008
Task B: "Write failing test + implement collect_control_ids_from_component_def in src/oscal/component_definition.rs"  # T009

# Sequential — same file:
Task: "Define AP structs in src/oscal/assessment_plan.rs"       # T005
Task: "Implement build_assessment_plan in src/oscal/assessment_plan.rs"  # T006
Task: "Implement derive_ap_output_path in src/oscal/assessment_plan.rs"  # T007
```

---

## Implementation Strategy

### MVP First (User Story 1 Only — ~8 tasks)

1. Complete Phase 1: Setup (T001)
2. Complete Phase 2: Foundational (T002, T003) — **critical blocker**
3. Complete Phase 3: US1 (T004–T010)
4. **STOP and VALIDATE**: `cargo test assessment_plan` passes; `build_assessment_plan` callable from Rust
5. Continue to US2 (Phase 4) to wire the CLI and make it end-to-end testable

### Incremental Delivery

1. Phase 1 + 2 → Foundational types ready
2. Phase 3 → US1 complete → `build_assessment_plan` unit-tested independently (MVP)
3. Phase 4 → US2 complete → CLI wired → end-to-end `forge convert ... --import-ssp` works
4. Phase 5 → US3 complete → UUID determinism verified by integration test
5. Phase 6 → All integration tests + quality gates → ready for PR

### Acceptance Coverage

| AC/EC | PRD Req | Phase | Task |
|-------|---------|-------|------|
| AC-1 | M-1 | Phase 3 | T004, T006 |
| AC-2 | M-2 | Phase 3 | T004, T006 |
| AC-3 | M-3 | Phase 4 | T011, T013 |
| AC-4 | M-4, M-5 | Phase 3 | T004, T006 |
| AC-5 | M-6 | Phase 4+6 | T015, T016, T019 |
| AC-6 | M-7 | Phase 5 | T017, T018 |
| AC-7 | S-1 | Phase 3 | T004, T006 |
| EC-1 | M-5 | Phase 3 | T004, T006 |
| EC-2 | M-3 | Phase 4 | T011, T013 |
| EC-3 | M-5 | Phase 3 | T004, T006 |
| EC-4 | M-1 | Phase 6 | T019, T020 |
| EC-5 | M-7 | Phase 5 | T017, T018 |
| SEC-2 | M-6 | Phase 4 | T011, T013 |
| SEC-3 | M-5 | Phase 3 | T004, T006 |
| SEC-4 | M-7 | Phase 5 | T017, T018 |

---

## Notes

- [P] tasks operate on different files — no merge conflicts
- TDD is not optional: `cargo test` must fail BEFORE implementation for each test task
- All inline tests live in `#[cfg(test)]` blocks at the bottom of their respective source files
- Integration tests live in `tests/assessment_plan_test.rs` (new file)
- AP output is always JSON regardless of primary `--format` flag (design decision D-3)
- Batch mode skips AP generation with a warning — same pattern as `--stable-id-baseline`
- No new Cargo dependencies — all required crates already in Cargo.toml
- Commit after each phase checkpoint to keep history clean
