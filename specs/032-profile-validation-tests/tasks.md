# Tasks: Profile Validation and Golden-File Tests (WI-32)

**Branch**: `032-profile-validation-tests` | **Date**: 2026-02-18
**Input**: Design documents from `specs/032-profile-validation-tests/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, quickstart.md ✅

**WI-31 Note**: Parameter tailoring (`--set-param`, `modify` section) is NOT implemented. Three tests are `#[ignore]` stubs to be enabled when WI-31 lands.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no shared state)
- **[Story]**: User story this task belongs to (US1, US2, US3)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Obtain the OSCAL v1.2.0 Profile JSON schema — required at compile time in Phase 2.

- [x] T001 Download the OSCAL v1.2.0 Profile JSON schema from `https://github.com/usnistgov/OSCAL/releases/download/v1.2.0/oscal_profile_schema.json` and save to `schemas/oscal_profile_schema.json` (alongside the existing `oscal_catalog_schema.json` and `oscal_component_schema.json`)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can begin.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T002 Extend `src/validate/mod.rs` with the `Profile` variant of `OscalModelType`. **TDD order**: write the unit tests in step (5) first — they reference `OscalModelType::Profile` which does not exist yet, so `cargo build` fails; then implement (1)–(4) until `cargo test` passes. Steps: (1) add `Profile` to the `OscalModelType` enum, (2) add `OscalModelType::Profile => include_str!("../../schemas/oscal_profile_schema.json")` arm to `load_schema()`, (3) add a `json.get("profile").is_some() => Ok(OscalModelType::Profile)` arm to `detect_model_type()` (before the error fallthrough), (4) add `OscalModelType::Profile => write!(f, "profile")` arm to the `Display` impl, (5) add unit tests `load_schema_profile` and `detect_model_type_profile` in the existing `#[cfg(test)]` block: `load_schema_profile` asserts `load_schema(OscalModelType::Profile)` returns `Ok(s)` where `!s.is_empty()`; `detect_model_type_profile` asserts `detect_model_type(&json!({ "profile": {} }))` returns `Ok(OscalModelType::Profile)`

- [x] T003 [P] Add `pub fn normalize_for_snapshot(value: &serde_json::Value) -> serde_json::Value` to `tests/common/mod.rs`. This function recursively normalizes a JSON value: (a) any `String` matching the UUID pattern `^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$` → `"00000000-0000-0000-0000-000000000000"`, (b) the `last-modified` key's string value → `"2026-01-01T00:00:00Z"`, (c) any string starting with `/` or a Windows drive letter pattern (`[A-Za-z]:\`) → `"NORMALIZED_PATH"`, (d) recurse into all `Value::Object` fields and `Value::Array` elements. Note: T003 is independent of T001/T002 and can run in parallel with T002.

**Checkpoint**: Foundation ready — user story implementation can now begin.

---

## Phase 3: User Story 1 — Schema-Valid Profile Output (Priority: P1) 🎯 MVP

**Goal**: Automated confirmation that `forge profile` output (include-only, exclude-only paths) conforms to the OSCAL v1.2.0 Profile JSON schema.

**Independent Test**: `cargo test --test profile_validation_tests schema` — 2 active tests pass, 1 `#[ignore]` test skipped.

### Implementation for User Story 1

- [x] T004 [P] [US1] Create `tests/profile_validation_tests.rs` with the following content: (1) a `const CATALOG_JSON: &str` containing minimal OSCAL catalog JSON with 10 controls (ids: AC-1 through AC-10) per the fixture design in `specs/032-profile-validation-tests/data-model.md` Entity 3, (2) a helper `fn make_catalog_file() -> tempfile::NamedTempFile` that writes `CATALOG_JSON` to a temp file and returns it (keep the return value alive in the test to prevent premature deletion), (3) test `schema_include_only`: create catalog tempfile, call `build_profile(catalog_path_str, vec!["AC-1".into(), "AC-2".into()], SelectionMode::Include)`, unwrap to get an `OscalProfile`, serialize to `serde_json::Value` via `serde_json::to_value()`, call `validate_artifact(&value, OscalModelType::Profile)` (from `forge::validate`), assert `result.is_valid == true` and `result.errors.is_empty()`, (4) test `schema_exclude_only`: same pattern but `SelectionMode::Exclude` with `vec!["AC-10".into()]`, (5) `#[ignore]` stub `schema_with_set_param` with body `todo!("Enable when WI-31 (--set-param) is implemented")`. Note: PRD S-4 (schema error messages include JSON path) is satisfied by WI-19's `jsonschema` crate — `ValidationError` carries `instance_path`; verify this when a validation error is present by asserting the error context is non-empty.

**Checkpoint**: US1 complete — `forge profile` output is confirmed schema-valid for include and exclude paths.

---

## Phase 4: User Story 2 — Golden-File Regression Tests (Priority: P1)

**Goal**: Insta snapshot tests that lock down exact normalized Profile JSON for include-only and exclude-only scenarios, catching any unintended output changes.

**Independent Test**: `cargo test --test profile_golden_file_tests` — 2 active snapshot tests pass; 1 `#[ignore]` skipped.

### Implementation for User Story 2

- [x] T005 [P] [US2] Create `tests/profile_golden_file_tests.rs` with the following content: (1) re-use the `make_catalog_file()` helper and `CATALOG_JSON` const — define them here too or factor into `tests/common/mod.rs` if needed, (2) test `golden_include_only`: create catalog tempfile, call `build_profile(catalog_path_str, vec!["AC-1".into(), "AC-2".into(), "AC-3".into()], SelectionMode::Include)`, serialize to `serde_json::Value`, call `common::normalize_for_snapshot(&value)`, then `insta::assert_json_snapshot!("golden_include_only", &normalized)`, (3) test `golden_exclude_only`: same pattern with `SelectionMode::Exclude` and `vec!["AC-9".into(), "AC-10".into()]`, snapshot name `"golden_exclude_only"`, (4) `#[ignore]` stub `golden_include_with_params` with body `todo!("Enable when WI-31 (--set-param) is implemented")`

- [x] T006 [US2] Generate and accept the initial insta snapshots: (1) run `cargo test --test profile_golden_file_tests` — the 2 active tests will fail on first run and write `.snap.new` files to `tests/snapshots/`, (2) run `cargo insta accept` to approve the generated snapshots, (3) verify that `tests/snapshots/profile_golden_file_tests__golden_include_only.snap` and `tests/snapshots/profile_golden_file_tests__golden_exclude_only.snap` exist and contain normalized JSON (no raw UUIDs, no timestamps, no absolute paths), (4) run `cargo test --test profile_golden_file_tests` again to confirm both active tests now pass

**Checkpoint**: US2 complete — golden-file regression safety net is in place for include-only and exclude-only Profile generation.

---

## Phase 5: User Story 3 — Edge Case Coverage (Priority: P1)

**Goal**: Tests covering boundary conditions (empty selection, all controls, duplicate IDs, conflicting flags, invalid catalog, nonexistent control ID) and the AC-12 end-to-end scenario.

**Independent Test**: `cargo test --test profile_validation_tests edge e2e` — 7 active tests pass, 1 `#[ignore]` skipped.

### Implementation for User Story 3

- [x] T007 [US3] Add 7 tests (6 active + 1 ignored) to `tests/profile_validation_tests.rs`: (1) `edge_empty_include_list`: call `build_profile(path, vec![], SelectionMode::Include)`, assert `Err` is returned with a descriptive error message (e.g., `ForgeError::InvalidArgument` or similar), (2) `edge_all_controls_include`: call with all 10 IDs (`vec!["AC-1".into(), ..., "AC-10".into()]`), assert `Ok`, serialize, call `validate_artifact()`, assert `is_valid`, (3) `edge_duplicate_control_ids`: call with `vec!["AC-1".into(), "AC-1".into(), "AC-2".into()]`, assert `Ok`, parse the output JSON, navigate to `profile.imports[0].include-controls[0].with-ids`, assert it contains exactly 2 entries (deduplication is idempotent), (4) `edge_both_flags_returns_error`: call the profile CLI handler with both `include: Some("AC-1")` and `exclude: Some("AC-10")`, assert `Err` containing a message referencing mutual exclusivity, (5) `edge_invalid_catalog_path`: call `build_profile("/tmp/nonexistent-catalog-99999.json", vec!["AC-1".into()], SelectionMode::Include)`, assert `Err`, (6) `edge_nonexistent_control_id`: call with a control ID that does not exist in the catalog (e.g., `vec!["FAKE-999".into()]`), assert `Err` with a message identifying the unknown ID, (7) `#[ignore]` stub `edge_conflicting_set_param` with body `todo!("Enable when WI-31 (--set-param) is implemented")`

- [x] T008 [US3] Add test `e2e_ac12_profile_generation` to `tests/profile_validation_tests.rs`. Add a doc comment: `/// Verifies parent PRD AC-12: given a policy Catalog with multiple controls, forge profile with include/exclude flags generates a valid OSCAL Profile`. Test body: create catalog tempfile, call `build_profile(catalog_path_str, vec!["AC-1".into(), "AC-2".into(), "AC-3".into(), "AC-4".into(), "AC-5".into()], SelectionMode::Include)`, assert `Ok`, serialize to `serde_json::Value`, call `validate_artifact(&value, OscalModelType::Profile)`, assert `result.is_valid == true` and `result.errors.is_empty()`, assert the JSON contains `"include-controls"` key in the imports section, assert `profile.imports[0].include-controls[0].with-ids` has exactly 5 entries

**Checkpoint**: US3 complete — all boundary conditions have defined, tested outcomes.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final quality gate before PR submission.

- [x] T009 Run `cargo test` and verify zero test failures. Expected outcome: 13 active tests pass; 3 `#[ignore]` tests skipped (`schema_with_set_param`, `golden_include_with_params`, `edge_conflicting_set_param`). If any active test fails, investigate root cause and fix before proceeding. Do NOT mark complete if any active test is failing.
- [x] T010 [P] Run `cargo clippy -- -D warnings` and resolve any warnings in the new or modified files: `src/validate/mod.rs`, `tests/common/mod.rs`, `tests/profile_validation_tests.rs`, `tests/profile_golden_file_tests.rs`
- [x] T011 [P] Run `cargo fmt --check` and resolve any formatting issues in modified files

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: T002 depends on T001 (compile-time `include_str!` requires schema file); T003 is independent of T001/T002 but logically grouped here
- **User Story Phases (Phases 3–5)**: All depend on T002 + T003 completion
- **Polish (Phase 6)**: Depends on all user story phases being complete

### User Story Dependencies

- **US1 (Phase 3)**: Can start after T002 + T003 — independent of US2 (different file)
- **US2 (Phase 4)**: Can start after T002 + T003 — independent of US1 (different file: `profile_golden_file_tests.rs`)
- **US3 (Phase 5)**: Adds to `tests/profile_validation_tests.rs` (same file as US1) — complete T004 before T007/T008 to avoid conflicts

### Critical Sequential Constraints

| Task | Depends On | Reason |
|------|-----------|--------|
| T002 | T001 | `include_str!("../../schemas/oscal_profile_schema.json")` fails to compile without the file |
| T004 | T002, T003 | Uses `OscalModelType::Profile` and `validate_artifact()` |
| T005 | T002, T003 | Uses `OscalModelType::Profile` and `normalize_for_snapshot()` |
| T006 | T005 | Tests must exist before `cargo insta accept` |
| T007, T008 | T004 | Add to the same file created in T004 |
| T009-T011 | T004-T008 | All tests must be written before final verification |

### Parallel Opportunities

- **T002 and T003**: Different files (`src/validate/mod.rs` vs `tests/common/mod.rs`) — run simultaneously
- **T004 and T005**: Different files (`tests/profile_validation_tests.rs` vs `tests/profile_golden_file_tests.rs`) — run simultaneously after Foundational phase
- **T010 and T011**: Independent CLI commands — run simultaneously

---

## Parallel Example: Foundational Phase

```bash
# After T001 completes, launch both in parallel:
Task A: "Extend OscalModelType with Profile variant in src/validate/mod.rs" (T002)
Task B: "Add normalize_for_snapshot() to tests/common/mod.rs" (T003)
```

## Parallel Example: US1 + US2 Creation

```bash
# After T002 + T003 both complete, launch in parallel:
Task A: "Create tests/profile_validation_tests.rs with schema tests" (T004)
Task B: "Create tests/profile_golden_file_tests.rs with golden-file tests" (T005)
```

---

## Implementation Strategy

### MVP First (US1 Only)

1. Complete Phase 1: T001 — download schema
2. Complete Phase 2: T002 + T003 — foundational infrastructure
3. Complete Phase 3: T004 — schema validation tests
4. **STOP and VALIDATE**: `cargo test --test profile_validation_tests schema` → 2 active tests pass, 1 ignored
5. Continue to US2 (golden files) and US3 (edge cases)

### Incremental Delivery

| Step | Tasks | Validation Gate |
|------|-------|-----------------|
| Foundation | T001 → T002 + T003 | `cargo build` succeeds; OscalModelType::Profile compiles |
| US1 | T004 | `cargo test schema_include_only schema_exclude_only` — both pass |
| US2 | T005 → T006 | `cargo test golden_include_only golden_exclude_only` — both pass |
| US3 | T007 → T008 | `cargo test edge e2e` — 7 active tests pass |
| Polish | T009 → T010 + T011 | `cargo test && cargo clippy && cargo fmt --check` — all green |

### WI-31 Stub Policy

Three tests are `#[ignore]`-annotated stubs. Each MUST have:
- `#[ignore]` attribute
- Body: `todo!("Enable when WI-31 (--set-param) is implemented")`
- Comment: `// TODO(WI-31): remove #[ignore] when --set-param is implemented`

When WI-31 lands: remove `#[ignore]`, implement the test body, run `cargo insta accept` for the new golden snapshot (US2 stub only).

---

## AR Guardrails (enforced throughout)

| Guardrail | Source |
|-----------|--------|
| DO NOT build a new validation framework — use `validate_artifact()` from WI-19 | AR + research.md D-8 |
| DO NOT build a new golden-file framework — use `insta::assert_json_snapshot!()` | AR + spec clarification |
| DO NOT modify `tests/golden_file_tests.rs` — WI-21 infrastructure is off-limits | AR Implementation Guardrails |
| DO NOT modify `src/oscal/profile.rs` or `src/cli/profile.rs` — test-only WI | AR + plan.md guardrails |
| DO NOT use `run_full_validation()` — use `validate_artifact()` to skip SemanticValidator | research.md D-8 |
| DO NOT hard-code UUIDs or timestamps in snapshots — use `normalize_for_snapshot()` | data-model.md Entity 2 |

---

## Notes

- [P] tasks operate on different files with no shared state — safe to parallelize
- [Story] label maps each task to a specific PRD user story for traceability
- All 3 user stories are P1 — all must be complete before PR merge
- `cargo insta accept` in T006 is the only interactive step; it requires reviewing generated snapshot content to confirm normalization worked
- Snapshot files (`tests/snapshots/*.snap`) must be committed to git — they are the golden-file fixtures
- 13 active test functions total (2 unit in src/ + 11 integration in tests/); 3 `#[ignore]` stubs; 2 new integration test files; 1 source file modified; 1 test utility added; 2 new `.snap` files

