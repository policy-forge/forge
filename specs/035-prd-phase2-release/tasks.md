# Tasks: Phase 2 Integration Testing & v0.2.0 Release

**Input**: Design documents from `/specs/035-prd-phase2-release/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/ ✅
**Branch**: `035-prd-phase2-release`

**Tests**: This sprint IS the tests. Tasks produce 21 new integration test functions across 4 files (18 original + T043 EC-1, T044 EC-2, T045 EC-4). All tests verify existing Phase 2 feature behavior (no implementation code changes). Follow: write test → run → fix root cause if failing → confirm green.

**Organization**: Tasks grouped by user story. US1/US2/US5 are P1 (critical path). US3/US4 are P2. US6 (release) executes last.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no shared state)
- **[Story]**: User story [US1]–[US6] from spec.md
- All test file paths relative to repo root

---

## Phase 1: Setup (Baseline Verification)

**Purpose**: Confirm the pre-sprint baseline is clean before adding integration tests.

- [X] T001 Run `cargo build` and verify the `forge` binary compiles with all Phase 2 features (XML/YAML output, `forge export`, `forge profile`, `forge validate`)
- [X] T002 Run `cargo test` and record baseline pass count (expected: 1049+ passed, 0 failed, 3 ignored)

**Checkpoint**: Baseline confirmed — integration test development can begin.

---

## Phase 2: Foundational (Shared Test Infrastructure)

**Purpose**: Establish shared helpers used across multiple integration test files. Must complete before Phase 3+.

**⚠️ CRITICAL**: Parallel tasks here set up shared utilities; complete both before Phase 3.

- [X] T003 [P] Verify `forge::testing::assert_semantic_equivalence` is accessible as a library import in integration tests by adding a minimal test-only use of the symbol in `tests/integration_round_trip.rs` (scaffold only, delete after confirm)
- [X] T004 [P] Smoke-test all six format conversion pairs of `forge export` from the CLI using `tests/fixtures/golden/small/input.md`: first run `forge convert tests/fixtures/golden/small/input.md --strategy catalog --format json/xml/yaml` to produce `catalog.json`, `catalog.xml`, and `catalog.yaml`, then test all six export pairs (json→xml, json→yaml, xml→json, xml→yaml, yaml→json, yaml→xml); confirm exit 0 for each pair (generating intermediate XML/YAML first ensures non-JSON input files exist for reverse-direction pairs)

**Checkpoint**: Foundation ready — all four integration test files can be developed in parallel.

---

## Phase 3: User Story 1 — Verify Multi-Format Round-Trip (Priority: P1) 🎯 MVP

**Goal**: Cross-cutting integration tests that verify `forge export` round-trips preserve semantic equivalence for both Catalog and Component Definition across all three format pairs.

**Independent Test**: `cargo test integration_round_trip` → 5 tests pass.

**Clarification applied**: Component Definition ↔ XML round-trip normalizes `control_implementations` before comparison (clears the field on both sides, matching WI-28 pattern). YAML round-trip asserts full equivalence.

### Implementation for User Story 1

- [X] T005 [P] [US1] Create `tests/integration_round_trip.rs` with module-level doc comment, `forge_bin()` helper (using `env!("CARGO_BIN_EXE_forge")`), `create_temp_file()` helper, and `use forge::testing::assert_semantic_equivalence`
- [X] T006 [P] [US1] Implement `catalog_json_xml_json_round_trip` in `tests/integration_round_trip.rs`: run `forge convert tests/fixtures/golden/small/input.md --strategy catalog --format json` → temp JSON file; `forge export <json> --format xml` → temp XML file; `forge export <xml> --format json` → temp round-tripped JSON; assert `assert_semantic_equivalence` returns `is_equivalent: true` (traces M-1, AC-1)
- [X] T007 [P] [US1] Implement `catalog_json_yaml_json_round_trip` in `tests/integration_round_trip.rs`: same flow via YAML path; assert semantic equivalence (traces M-1, AC-2)
- [X] T008 [US1] Implement `component_definition_json_xml_json_round_trip` in `tests/integration_round_trip.rs`: convert `tests/fixtures/full_policy.md` → component JSON; export → XML → JSON; normalize both sides by clearing `control_implementations` before calling `assert_semantic_equivalence` (traces M-1, EC-5, clarification Q1)
- [X] T009 [US1] Implement `component_definition_json_yaml_json_round_trip` in `tests/integration_round_trip.rs`: same component via YAML path; assert full semantic equivalence including `control_implementations` (traces M-1, EC-5)
- [X] T043 [P] [US1] Implement `empty_groups_catalog_round_trip` in `tests/integration_round_trip.rs`: construct a minimal OSCAL Catalog JSON string with metadata and one group whose `controls` array is empty (`[]`); write to a temp file; `forge export <json> --format xml` → temp XML; `forge export <xml> --format json` → round-tripped JSON; assert `assert_semantic_equivalence` returns `is_equivalent: true` — verifies EC-1 (empty groups do not corrupt round-trip)
- [X] T010 [US1] Run `cargo test integration_round_trip -- --nocapture` and confirm all 5 tests pass (T006, T007, T008, T009, T043); fix any round-trip failures in upstream serialization if found (do not add new features — only fix defects)

**Checkpoint**: US1 complete — `cargo test integration_round_trip` → 5 passed, 0 failed (4 round-trip tests + T043 EC-1 empty groups test).

---

## Phase 4: User Story 2 — End-to-End Profile Generation with Tailoring (Priority: P1)

**Goal**: Integration tests that verify the full `forge profile` pipeline (generate → validate) with include, exclude, set-param, and multi-format output.

**Independent Test**: `cargo test integration_profile_e2e` → 6 tests pass.

### Implementation for User Story 2

- [X] T011 [P] [US2] Create `tests/integration_profile_e2e.rs` with `forge_bin()` helper and `catalog_from_policy()` helper that runs `forge convert <golden-small> --strategy catalog --format json --output <temp>.json` and returns the temp path
- [X] T012 [P] [US2] Implement `profile_include_produces_valid_oscal` in `tests/integration_profile_e2e.rs`: generate catalog via T011 helper; parse catalog JSON and extract the first two control IDs from `catalog.groups[0].controls[*].id` (use `.as_array()` on the controls field); run `forge profile --catalog <path> --include <extracted-id-1>,<extracted-id-2> --format json`; parse output; assert `profile.imports[0].include-controls[0].with-ids` contains the extracted IDs (traces M-2, AC-3)
- [X] T013 [P] [US2] Implement `profile_exclude_produces_valid_oscal` in `tests/integration_profile_e2e.rs`: generate catalog; run `forge profile --catalog <path> --exclude <control-ids> --format json`; assert `profile.imports[0].exclude-controls[0].with-ids` is present (traces S-1)
- [X] T014 [P] [US2] Implement `profile_set_param_produces_modify_section` in `tests/integration_profile_e2e.rs`: generate catalog; run `forge profile --catalog <path> --include <id> --set-param password-length 16 --format json`; assert `profile.modify.set-parameters` contains an entry with `param-id: "password-length"` and value `"16"` (traces M-3, AC-4)
- [X] T015 [US2] Implement `profile_passes_schema_validation` in `tests/integration_profile_e2e.rs`: generate profile to temp file; run `forge validate <profile-path>`; assert exit 0 and stdout contains "Valid" (traces M-4, AC-5)
- [X] T044 [P] [US2] Implement `profile_include_nonexistent_id_produces_error` in `tests/integration_profile_e2e.rs`: generate catalog; run `forge profile --catalog <path> --include NONEXISTENT-CONTROL-999 --format json`; assert the exit code is non-zero and stderr or stdout contains a descriptive error message referencing the unknown control ID (not a panic/unwrap crash) (traces EC-2)
- [X] T016 [US2] Implement `profile_xml_yaml_formats` in `tests/integration_profile_e2e.rs`: generate catalog; run `forge profile --format xml --output <temp>.xml` → assert exit 0, file non-empty, and file contents contain `<profile` element; run `forge profile --format yaml --output <temp>.yaml` → assert exit 0, parse via `serde_yaml_ng::from_str::<serde_json::Value>`, assert the resulting `profile.uuid` is a non-empty string (traces S-2)
- [X] T017 [US2] Run `cargo test integration_profile_e2e -- --nocapture` and confirm all 6 tests pass (T012, T013, T014, T015, T044, T016); fix any profile generation defects if found

**Checkpoint**: US2 complete — `cargo test integration_profile_e2e` → 6 passed, 0 failed (5 profile E2E tests + T044 EC-2 error path test).

---

## Phase 5: User Story 5 — Phase 1 Regression Verification (Priority: P1)

**Goal**: Lightweight structural regression tests confirming Phase 2 development has not corrupted Phase 1 pipeline output. Uses structural assertions only (no new insta snapshots — existing `golden_file_tests.rs` covers snapshot regression).

**Independent Test**: `cargo test integration_regression` → 3 tests pass.

**Clarification applied**: Structural assertions only — assert invariants (uuid present, oscal-version = "1.2.0", groups/controls non-empty). Allow Phase 2 additive `prop` and `param` elements. Do not snapshot.

### Implementation for User Story 5

- [X] T018 [P] [US5] Create `tests/integration_regression.rs` with `forge_bin()` helper and doc comment explaining structural-assertion strategy (no insta snapshots; additive Phase 2 props/params are allowed)
- [X] T019 [P] [US5] Implement `phase1_catalog_structure_regression` in `tests/integration_regression.rs`: run `forge convert tests/fixtures/golden/small/input.md --strategy catalog --format json`; parse output; assert `catalog.uuid` is a non-empty string, `catalog.metadata.oscal-version` == "1.2.0", `catalog.groups` is non-empty, at least one group has `controls` (traces M-6, AC-9)
- [X] T020 [P] [US5] Implement `phase1_component_structure_regression` in `tests/integration_regression.rs`: **prerequisite** — assert `Path::new("tests/fixtures/sample_profile.json").exists()` at test start with a clear error if absent; run `forge convert tests/fixtures/full_policy.md --strategy component --source-profile tests/fixtures/sample_profile.json --format json`; assert `component-definition.uuid` present, `component-definition.components` non-empty, first component `type` == "policy" (traces M-6, AC-9)
- [X] T021 [US5] Implement `phase1_validate_still_passes` in `tests/integration_regression.rs`: convert `tests/fixtures/golden/small/input.md` → catalog JSON to temp file; run `forge validate <path>`; assert exit 0 and stdout contains "Valid" (traces M-6, AC-9)
- [X] T022 [US5] Run `cargo test integration_regression -- --nocapture` and confirm all 3 tests pass

**Checkpoint**: US5 complete — `cargo test integration_regression` → 3 passed, 0 failed.

---

## Phase 6: User Stories 3 & 4 — Cross-Feature Verification (Priority: P2)

**Goal**: Integration tests verifying that normative/advisory `prop` annotations and `param` elements from Phase 2 enrichment passes (WI-33, WI-34) are present in all output formats and survive format round-trips. US3 and US4 share a single test file.

**Independent Test**: `cargo test integration_cross_feature` → 7 tests pass.

### Implementation for User Stories 3 & 4

- [X] T023 [P] [US3] Create `tests/integration_cross_feature.rs` with `forge_bin()` helper and `MIXED_POLICY` const fixture containing: normative language ("Systems must enforce MFA", "Passwords must be changed within 90 days"), advisory language ("Administrators should review logs weekly"), and parameterized requirements ("within 90 days", "up to 8 hours"); **before finalizing the fixture**, verify these phrasings trigger WI-33 normative/advisory detection and WI-34 parameter extraction by checking the regex patterns in `src/parse/` — adjust trigger phrases if needed to guarantee param extraction in T027–T029
- [X] T024 [P] [US3] Implement `normative_props_present_in_json` in `tests/integration_cross_feature.rs`: convert `MIXED_POLICY` → JSON; traverse all controls; assert at least one control has `props` containing `{"name": "modality", "value": "normative"}` and at least one has `{"name": "modality", "value": "advisory"}` (traces M-5, AC-6)
- [X] T025 [P] [US3] Implement `normative_props_survive_xml_round_trip` in `tests/integration_cross_feature.rs`: convert `MIXED_POLICY` → JSON file; `forge export` → XML; `forge export` → JSON; compare modality props between original and round-tripped (collect all `prop[name=modality]` values from each; assert identical sets) (traces M-5, AC-8)
- [X] T026 [P] [US3] Implement `normative_props_survive_yaml_round_trip` in `tests/integration_cross_feature.rs`: same flow via YAML; assert modality props identical after round-trip (traces M-5, AC-8)
- [X] T045 [US3] Implement `atomized_normative_advisory_each_gets_correct_prop` in `tests/integration_cross_feature.rs`: add a compound sentence to `MIXED_POLICY` containing both normative and advisory clauses in one statement (e.g., "Systems must enforce MFA and administrators should review logs"); convert to JSON; verify the atomizer produced separate controls (controls count >= 2 for this section); assert at least one control has `prop[name=modality, value=normative]` AND at least one has `prop[name=modality, value=advisory]` — each atomized control carries its own distinct modality prop (traces EC-4, M-5)
- [X] T027 [P] [US4] Implement `param_elements_present_in_json` in `tests/integration_cross_feature.rs`: convert `MIXED_POLICY` → JSON; traverse all controls; assert at least one control has a non-empty `params` array with an entry containing `id` and `values` (traces M-5, AC-7)
- [X] T028 [P] [US4] Implement `param_elements_survive_xml_round_trip` in `tests/integration_cross_feature.rs`: convert `MIXED_POLICY` → JSON file; `forge export` → XML; `forge export` → JSON; collect all `param.id` + `param.values` from original and round-tripped; assert identical sets (traces M-5, AC-8)
- [X] T029 [US4] Implement `param_elements_survive_yaml_round_trip` in `tests/integration_cross_feature.rs`: same via YAML path; assert param IDs and values identical after round-trip (traces M-5, AC-8)
- [X] T030 [US3] Run `cargo test integration_cross_feature -- --nocapture` and confirm all 7 tests pass (T024, T025, T026, T045, T027, T028, T029); fix any prop/param serialization defects in upstream XML/YAML serializers if found

**Checkpoint**: US3 + US4 complete — `cargo test integration_cross_feature` → 7 passed, 0 failed (6 cross-feature tests + T045 EC-4 atomized prop test).

---

## Phase 7: User Story 6 — Tag and Publish v0.2.0 (Priority: P1)

**Goal**: Execute the four-gate quality check, prepare release artifacts, and create the `v0.2.0` git tag on a fully verified commit.

**Independent Test**: `git tag -l v0.2.0` exists; `forge --version` reports `0.2.0`; CI passes.

**⚠️ GATE**: All phases 3–6 must be complete and passing before Phase 7 begins. Tag only after all four quality gates pass (test + clippy + fmt + deny).

### Implementation for User Story 6

- [X] T031 [US6] Run complete test suite: `cargo test` — must report 0 failures (all 1049+ existing + 21 new integration tests: T006–T009, T043, T012–T016, T044, T019–T021, T024–T029, T045); record total pass count
- [X] T032 [US6] Run `cargo clippy -- -D warnings` — must report 0 warnings; fix any warnings in integration test files if found
- [X] T033 [US6] Run `cargo fmt --check` — must report 0 violations; run `cargo fmt` if needed then re-check; also run `cargo deny check` (deny.toml exists at repo root) — must report 0 deny violations; fix any license or advisory violations before proceeding (traces constitution Principle XI)
- [X] T034 [US6] Update `Cargo.toml` version field from `"0.1.0"` to `"0.2.0"` in `/Cargo.toml`
- [X] T035 [US6] Create `CHANGELOG.md` at repo root documenting v0.2.0 Phase 2 features: WI-26 (XML output), WI-27 (YAML output), WI-28 (round-trip testing), WI-29 (`forge export`), WI-30 (Profile generation), WI-31 (parameter tailoring), WI-32 (Profile validation), WI-33 (normative/advisory detection), WI-34 (parameter extraction), WI-35 (Phase 2 integration testing) (traces S-3)
- [X] T036 [US6] Review `forge profile --help` and `forge export --help` output; assert each of the following flags has a non-empty, human-readable description (not empty or "TODO"): for `profile` — `--include`, `--exclude`, `--set-param`, `--format`, `--output`; for `export` — `--format`, `--output`; update `#[arg(help = "...")]` annotations in `src/cli/` for any missing or inadequate descriptions (traces S-4) [note: removed [P] marker — T036 modifies src/cli/ files staged by T038; must complete before T038]
- [X] T037 [US6] Re-run all four quality gates after T034–T036: `cargo test && cargo clippy -- -D warnings && cargo fmt --check && cargo deny check` — all must pass with 0 issues; only proceed to T038 when all four gates report zero failures/warnings/violations
- [X] T038 [US6] Stage and commit release preparation: `git add Cargo.toml Cargo.lock CHANGELOG.md src/` then `git commit -m "release: bump version to 0.2.0 for Phase 2 milestone"`
- [X] T039 [US6] Create annotated release tag: `git tag v0.2.0` (traces M-7, AC-10)
- [X] T040 [US6] Verify release: run `cargo run -- --version` and confirm output contains `0.2.0`; run `git tag -l v0.2.0` and confirm tag exists (traces M-8, AC-10)

**Checkpoint**: US6 complete — v0.2.0 tagged on a fully verified commit. MS-6 exit criteria met.

---

## Final Phase: Polish & Cross-Cutting Concerns

**Purpose**: Optional hardening steps after the release tag is created.

- [ ] T041 [P] Run `cargo doc --workspace --no-deps` — confirm documentation builds without warnings; fix any missing `rustdoc` items in new test helper functions if flagged
- [ ] T042 [P] Run `cargo audit` — confirm no active RustSec advisories in the dependency tree; address any CRITICAL/HIGH advisories before distributing

---

## Dependencies & Execution Order

### Phase Dependencies

```
Phase 1 (Setup)
  └─> Phase 2 (Foundational) ── T003 + T004 [parallel]
        └─> Phase 3 (US1) ─────── T005–T010
        └─> Phase 4 (US2) ─────── T011–T017   } can start simultaneously
        └─> Phase 5 (US5) ─────── T018–T022   } after Phase 2 complete
        └─> Phase 6 (US3+US4) ─── T023–T030   }
              ↓ (all phases 3–6 complete)
        Phase 7 (US6 Release) ─── T031–T040 [sequential gates]
              ↓
        Final (Polish) ─────────── T041–T042 [parallel]
```

### User Story Dependencies

- **US1 (P1)**: Depends on Phase 2 only. No dependency on US2/US5.
- **US2 (P1)**: Depends on Phase 2 only. No dependency on US1/US5.
- **US5 (P1)**: Depends on Phase 2 only. No dependency on US1/US2.
- **US3 (P2)**: Depends on Phase 2 only. Shares `integration_cross_feature.rs` with US4 — implement in sequence within the file.
- **US4 (P2)**: Follows US3 within the same file (T027–T030 after T023–T026).
- **US6 (P1 gate)**: Depends on ALL of US1, US2, US3, US4, US5 being complete and passing.

### Within Each User Story

- Scaffold file first (T005, T011, T018, T023) → then individual test functions
- Each test function: write → `cargo test <name>` → fix if failing → confirm green
- `cargo test integration_<story>` checkpoint before marking story complete
- Fix defects in upstream Phase 2 code if found; do not add new features

---

## Parallel Execution Examples

### After Phase 2: Run US1, US2, US5 in parallel

```text
Agent A: Tasks T005–T009, T043, T010 (tests/integration_round_trip.rs — US1, includes EC-1)
Agent B: Tasks T011–T015, T044, T016–T017 (tests/integration_profile_e2e.rs — US2, includes EC-2)
Agent C: Tasks T018–T022 (tests/integration_regression.rs — US5)
Agent D: Tasks T023–T026, T045, T027–T030 (tests/integration_cross_feature.rs — US3+US4, includes EC-4)
```

Each agent works in a distinct file — no merge conflicts.

### Within US1: Parallel test scaffolding

```text
Task: T006 — catalog_json_xml_json_round_trip
Task: T007 — catalog_json_yaml_json_round_trip
(Both write to same file; write T005 scaffold first, then T006+T007 in parallel)
```

### Release gates (sequential — cannot parallelize)

```text
T031 cargo test → T032 cargo clippy → T033 cargo fmt --check
→ T034 version bump → T035 CHANGELOG → T036 help review
→ T037 re-run gates → T038 commit → T039 tag → T040 verify
```

---

## Implementation Strategy

### MVP First (US1 Only — after Phase 2)

1. Complete Phase 1: Setup (T001–T002)
2. Complete Phase 2: Foundational (T003–T004)
3. Complete Phase 3: US1 round-trip (T005–T010)
4. **STOP and VALIDATE**: `cargo test integration_round_trip` → 5 passed
5. Proceed to Phase 4 (US2)

### Full Parallel Delivery (single developer, sequential)

1. Phase 1 → Phase 2 → Phase 3 (US1) → Phase 4 (US2) → Phase 5 (US5) → Phase 6 (US3+US4) → Phase 7 (US6) → Final

### Multi-Agent Delivery (after Phase 2)

1. Phase 1 + Phase 2: Single agent
2. Phases 3–6: 4 agents in parallel (one per integration test file)
3. Phase 7 + Final: Single agent (sequential release gates)

---

## Notes

- **No feature code**: WI-35 adds integration tests and release prep only. If a test fails, fix the root cause in the upstream Phase 2 WI (WI-26–WI-34 source files), not by weakening the test assertion.
- **Defects vs. features**: If integration reveals a genuine defect (e.g., `prop` elements lost in XML serialization), fix it. If it reveals missing functionality, escalate — do not implement new features.
- **XML normalization (Q1)**: T008 only — Component Definition XML round-trip clears `control_implementations` on both original and round-tripped before comparison.
- **EC-3 behavior (Q2)**: If testing `--set-param` with unknown ID, assert exit 0 + non-empty stderr warning (permissive behavior); this is not a required integration test task but the behavior is specified if needed.
- **Regression strategy (Q3)**: T019–T021 use structural assertions only. Do not add insta snapshots. The existing `golden_file_tests.rs` already covers snapshot-level regression.
- **AR function name alignment (I1)**: Integration test function names follow tasks.md definitions. The AR interface definitions (e.g., `phase1_acceptance_criteria_pass`, `catalog_golden_file_regression`) were illustrative drafts — the authoritative names are `phase1_catalog_structure_regression` (T019), `phase1_component_structure_regression` (T020), and `phase1_validate_still_passes` (T021) per clarification Q3.
- **`cargo deny check` gate (D1)**: T033 and T037 include `cargo deny check` as a fourth quality gate. `deny.toml` exists at repo root. This satisfies constitution Principle XI (NON-NEGOTIABLE).
- **EC-6 deferred**: `--include` + `--exclude` combination behavior is defined by WI-30 and covered by WI-30 unit tests; no additional integration test required for v0.2.0. Low-priority addition for Phase 3.
- **`cargo geiger` deferred**: WI-35 introduces no new `unsafe` code (test-only sprint); `cargo geiger` audit is deferred to CI configuration in a future sprint.
- **Tag requires confirmation**: T039 creates the `v0.2.0` git tag. Confirm with the user before executing if running autonomously.
- **[P] tasks** = different files, no shared state, safe to parallelize. Note: T036 has had its [P] marker removed — it modifies `src/cli/` files that T038 stages and must complete before T038.
