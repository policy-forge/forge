# Tasks: Profile Parameter Tailoring (WI-31)

**Input**: Design documents from `/specs/031-profile-parameter-tailoring/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/rust-api.md ✅, prd.md ✅, ar.md ✅, sec.md ✅
**Branch**: `031-profile-parameter-tailoring`

**Tests**: Included — TDD is **mandatory** per PRD Technical Constraints ("TDD mandatory; unit tests for modify section construction and CLI argument parsing") and plan.md Constitution Gate IV.

**Organization**: Tasks grouped by user story. US1 holds all core infrastructure (new types, function, CLI, execute). US2 and US3 are test-only phases — the Phase 3 BTreeMap and serde types satisfy their requirements automatically.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no shared in-progress dependencies)
- **[Story]**: Which user story this task belongs to
- Exact file paths included in every description

---

## Phase 1: Setup

**Purpose**: Verify current baseline and identify any blocked state before implementation begins.

- [X] T001 Run `cargo check --tests` from repo root and identify any pre-existing compile errors blocking implementation

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Fix pre-existing compile errors from WI-33/WI-34 model additions (`parameters` field). These MUST be resolved before any WI-31 code can be integrated.

**⚠️ CRITICAL**: No user story work can begin until `cargo build` passes.

- [X] T002 Fix `PolicyRequirement` struct literal in the `///` doctest in `src/parse/atomize.rs`: add `modality: None, parameters: vec![]` fields to the struct initializer (missing fields from WI-33/WI-34)
- [X] T003 [P] Fix `req()` test helper function in `src/parse/modality.rs` test module: add `parameters: vec![]` to the `PolicyRequirement` struct initializer (approximately line 178; `modality: None` is already present)
- [X] T004 Run `cargo build` to confirm all compile errors are resolved and the codebase builds cleanly

**Checkpoint**: `cargo build` passes — user story implementation can now begin.

---

## Phase 3: User Story 1 — Set a Single Parameter Value (Priority: P1) 🎯 MVP

**Goal**: A compliance engineer can run `forge profile --catalog catalog.json --include POL-AC-001 --set-param POL-AC-001_prm "60 days" --format json` and receive a Profile JSON containing a `modify.set-parameters` array with exactly one correct entry.

**Independent Test**: Run the command above and verify the output JSON contains `"modify": { "set-parameters": [{ "param-id": "POL-AC-001_prm", "values": ["60 days"] }] }`.

### Tests for User Story 1 (TDD — write first, confirm RED before implementing)

> **NOTE**: Write these tests FIRST. They must **compile** (after adding struct skeletons) and **fail at runtime** before proceeding to implementation.

- [X] T005 [US1] Write unit tests for `build_modify_section` in `src/oscal/profile.rs` covering: empty input → `None`, single `(id, value)` pair → `Some(Modify)`, value containing spaces (e.g., `"60 days"`) → preserved as a single string, empty-string value `""` → entry with `values: [""]`
- [X] T006 [US1] Write `/// # Examples` doctest for `build_modify_section` in `src/oscal/profile.rs` demonstrating the single-param case and the empty-input → `None` case

### Implementation for User Story 1

- [X] T007 [US1] Add `Modify` struct with `#[derive(Debug, Serialize)]` and `#[serde(rename = "set-parameters")]` on `set_parameters: Vec<SetParameter>` to `src/oscal/profile.rs`; add `///` rustdoc on struct and field (Constitution V — rustdoc required for all public API items)
- [X] T008 [US1] Add `SetParameter` struct with `#[derive(Debug, Serialize)]`, `#[serde(rename = "param-id")]` on `param_id: String`, and `values: Vec<String>` field to `src/oscal/profile.rs`; add `///` rustdoc on struct and its fields (Constitution V)
- [X] T009 [US1] Add `build_modify_section(param_overrides: &[(String, String)]) -> Option<Modify>` skeleton returning `todo!()` to `src/oscal/profile.rs` — confirm T005 tests compile then panic/fail RED (SEC-6: must be a pure function)
- [X] T010 [US1] Implement `build_modify_section` body in `src/oscal/profile.rs`: annotate with `#[tracing::instrument(skip_all, fields(param_count = param_overrides.len()))]` (Constitution IX, SEC-6); early-return `None` for empty input (M-6, SEC-3); use `BTreeMap<String, Vec<String>>` with `entry().or_default().push()` for deterministic aggregation (S-1, S-2, SEC-4); collect into `Vec<SetParameter>`; return `Some(Modify { set_parameters })`
- [X] T011 [US1] Add `modify: Option<Modify>` field with `#[serde(skip_serializing_if = "Option::is_none")]` to `OscalProfile` struct in `src/oscal/profile.rs` (M-5, M-6)
- [X] T012 [US1] Extend `build_profile` signature in `src/oscal/profile.rs` to add `param_overrides: &[(String, String)]` parameter; update body to call `build_modify_section(param_overrides)` and assign the result to `OscalProfile.modify`
- [X] T013 [US1] Update all existing `build_profile` call sites in `src/cli/profile.rs` and test modules within `src/oscal/profile.rs` to pass `&[]` as the new `param_overrides` argument (backward compat — empty slice produces no `modify` section)
- [X] T014 [US1] Write CLI parse test in `tests/profile_param_test.rs` verifying that `--set-param POL-AC-001_prm "60 days"` produces a `set_params` `Vec<String>` of `["POL-AC-001_prm", "60 days"]` in `Commands::Profile` (SEC-2)
- [X] T015 [US1] Add `set_params: Vec<String>` field with `#[arg(long = "set-param", num_args = 2, action = clap::ArgAction::Append, value_names = ["PARAM_ID", "VALUE"])]` to `Commands::Profile` in `src/cli/mod.rs` (M-1, SEC-2)
- [X] T016 [US1] Update the `Commands::Profile` dispatch arm in `src/cli/mod.rs` to destructure `set_params` and pass it to `profile::execute`
- [X] T017 [US1] Write integration test in `tests/profile_param_test.rs` for `profile::execute` with a single `--set-param` flag verifying the output JSON contains the correct `modify.set-parameters` entry (AC-1)
- [X] T018 [US1] Add `set_params: &[String]` parameter to `execute` in `src/cli/profile.rs`; implement private `parse_set_param_pairs(set_params: &[String]) -> Vec<(String, String)>` helper using `chunks_exact(2)` (clap's `num_args = 2` guarantees even length)
- [X] T019 [US1] In `execute` in `src/cli/profile.rs`: pass `parse_set_param_pairs(set_params)` result to `build_profile`; add `tracing::info!(param_count = pairs.len(), "profile: {} parameter override(s) specified")` after computing pairs (Constitution IX); implement C-2 warning — when `set_params` is non-empty and both `include` and `exclude` are `None`, emit `eprintln!("warning: --set-param specified without --include or --exclude; the Profile will have no control imports")` and `tracing::warn!` then return `InvalidArgument`
- [X] T020 [US1] Write insta snapshot test in `tests/profile_param_test.rs` for the complete Profile JSON output with `--set-param POL-AC-001_prm "60 days"`; run `INSTA_UPDATE=always cargo test` to accept the initial snapshot

**Checkpoint**: Run the US1 Independent Test. Verify output matches the documented JSON shape in `contracts/rust-api.md`.

---

## Phase 4: User Story 2 — Set Multiple Parameters in a Single Command (Priority: P1)

**Goal**: A compliance engineer can specify multiple `--set-param` flags (including the same param-id twice) and receive a Profile with all entries present, alphabetically ordered, and duplicate IDs aggregated.

**Independent Test**: Run `forge profile --catalog catalog.json --include POL-AC-001,POL-IR-001 --set-param POL-AC-001_prm "60 days" --set-param POL-IR-001_prm "4 hours" --format json` and verify `modify.set-parameters` contains two entries in alphabetical order. Run the same command twice and verify byte-for-byte identical output (S-2, SEC-4).

> **NOTE**: No new implementation code is required for US2. The `BTreeMap` in `build_modify_section` (T010) handles aggregation and alphabetical ordering automatically. These tasks write and confirm tests only.

### Tests for User Story 2 (TDD — confirm tests PASS with Phase 3 implementation)

- [X] T021 [US2] Write unit tests for `build_modify_section` in `src/oscal/profile.rs` covering: two distinct param-ids → two entries in alphabetical order, ten distinct param-ids → all ten entries in alphabetical order, non-alphabetical input order → output in alphabetical order by param-id (EC-5, EC-6, S-2)
- [X] T022 [P] [US2] Write unit test for duplicate param-id aggregation in `src/oscal/profile.rs`: two pairs with the same param-id and different values → single `SetParameter` entry with `values: ["v1", "v2"]` (EC-2, S-1)
- [X] T023 [P] [US2] Write CLI parse test in `tests/profile_param_test.rs` for two `--set-param` flags: verifies `set_params` `Vec<String>` contains four elements `["id1", "val1", "id2", "val2"]`
- [X] T024 [US2] Write integration test in `tests/profile_param_test.rs` for two distinct `--set-param` flags: verify both entries present in `modify.set-parameters` and verify alphabetical ordering by `param-id` (AC-2, S-2)
- [X] T025 [P] [US2] Write integration test in `tests/profile_param_test.rs` for duplicate param-id aggregation: `--set-param prm1 "60 days" --set-param prm1 "quarterly"` produces a single entry with `values: ["60 days", "quarterly"]` (EC-2, S-1)
- [X] T026 [US2] Write insta snapshot test in `tests/profile_param_test.rs` for multi-param Profile JSON (two distinct params); run `INSTA_UPDATE=always cargo test` to accept snapshot — verify alphabetical ordering is captured in snapshot

**Checkpoint**: Run the US2 Independent Test. Verify determinism by running the same command twice and confirming byte-for-byte identical output.

---

## Phase 5: User Story 3 — Structurally Valid OSCAL (Priority: P1)

**Goal**: The generated Profile with a `modify` section has correct OSCAL JSON structure: `modify` is a sibling of `imports` and `metadata` under the `profile` root, each `set-parameters` entry has `param-id` (string) and `values` (array of strings), and no `modify` key appears when no `--set-param` flags are provided.

**Independent Test**: Generate a Profile with `--set-param` and deserialize the JSON; assert `profile.modify.set-parameters[0]` exists with correct fields. Generate a Profile without `--set-param` and assert the JSON has no `"modify"` key (AC-3, AC-4, AC-5, SEC-3, SEC-5).

> **NOTE**: No new implementation code is required for US3. Structure is guaranteed by the serde types defined in Phase 3 (T007, T008, T011).

### Tests for User Story 3 (TDD — confirm tests PASS with Phase 3 implementation)

- [X] T027 [US3] Write test in `tests/profile_param_test.rs` verifying JSON structure: deserialize Profile output and assert `modify` appears as a direct child of the `profile` root object at the same level as `imports` and `metadata` (M-5, AC-4)
- [X] T028 [P] [US3] Write test in `tests/profile_param_test.rs` verifying each `set-parameters` entry: for each entry in the deserialized Profile's `modify.set-parameters`, assert `param-id` is a string and `values` is an array of strings per OSCAL v1.2.0 Profile model (M-3, AC-5, SEC-5)
- [X] T029 [P] [US3] Write backward-compat regression test in `tests/profile_param_test.rs`: run `forge profile` without any `--set-param` flags and assert the output JSON does NOT contain a `"modify"` key — identical to WI-30 behavior (M-6, AC-3, SEC-3)
- [X] T030 [US3] Write determinism test in `tests/profile_param_test.rs`: run `forge profile` twice with identical `--set-param` inputs and assert the resulting JSON strings are byte-for-byte identical (S-2, SEC-4)

**Checkpoint**: All three user stories are independently testable and passing. The Profile output with `modify` section is structurally valid OSCAL v1.2.0.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Quality gates required for merge per plan.md pre-merge checklist and Constitution Principle IV.

- [X] T031 [P] Run `cargo clippy -- -D warnings` and fix any linting issues in all modified files (`src/oscal/profile.rs`, `src/cli/mod.rs`, `src/cli/profile.rs`, `src/parse/atomize.rs`, `src/parse/modality.rs`)
- [X] T032 [P] Run `cargo fmt`; re-run `cargo fmt --check` to confirm clean formatting
- [X] T033 Run `cargo test --workspace` to verify the complete test suite passes (unit tests, integration tests, snapshot tests)
- [X] T034 [P] Run `cargo test --doc` to verify all doctests pass (including the new `build_modify_section` doctest from T006)
- [X] T035 Validate `quickstart.md` end-to-end: build the binary with `cargo build` and manually run each example command in `specs/031-profile-parameter-tailoring/quickstart.md` using the WI-30 fixture catalog; confirm output JSON matches the documented shapes
- [X] T036 [P] Run `cargo llvm-cov --lib` (or `cargo tarpaulin --lib`) targeting `src/oscal/profile.rs` and verify line coverage on `build_modify_section` is ≥90%; fail the quality gate if below threshold (PRD Technical Verification, Constitution IV)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — run immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 — **BLOCKS all user stories**
- **Phase 3 (US1)**: Depends on Phase 2 (`cargo build` must pass)
- **Phase 4 (US2)**: Depends on Phase 3 (tests verify Phase 3 BTreeMap implementation)
- **Phase 5 (US3)**: Depends on Phase 3 (tests verify Phase 3 serde structure)
- **Phase 6 (Polish)**: Depends on Phases 3, 4, and 5

### Within Phase 3 (US1) — Sequential Dependencies

```
T005, T006 (write tests)
    ↓
T007, T008 (add structs — types needed for tests to compile)
    ↓
T009 (build_modify_section skeleton — tests compile, fail RED)
    ↓
T010 (implement — tests pass GREEN)
    ↓
T011, T012 (OscalProfile extend, build_profile extend)
    ↓
T013 (update all callers to pass &[])
    ↓
T014 (CLI parse test) → T015 (CLI arg) → T016 (dispatch update)
    ↓
T017 (integration test) → T018 (execute sig + helper) → T019 (pass pairs + C-2)
    ↓
T020 (snapshot test)
```

### User Story Dependencies

- **US1 (Phase 3)**: No dependency on US2 or US3 — independently deliverable as MVP
- **US2 (Phase 4)**: Depends on Phase 3 implementation (all tests verify already-written code)
- **US3 (Phase 5)**: Depends on Phase 3 implementation (all tests verify already-written code)
- US2 and US3 can proceed in **parallel** after Phase 3 completes

### Parallel Opportunities

| Tasks | Parallelizable? | Reason |
|-------|-----------------|--------|
| T002 + T003 | ✅ Yes | Different files (`atomize.rs`, `modality.rs`) |
| T007 + T008 | ✅ Yes | Different struct declarations, same file (additive) |
| T031 + T032 + T034 + T036 | ✅ Yes | Different quality tools, read-only analysis |
| T022 + T023 (US2) | ✅ Yes | Different test cases / different units |
| T025 (US2) + T027 (US3 start) | ✅ Yes | Independent test cases |
| T028 + T029 (US3) | ✅ Yes | Different test cases in same file |

---

## Parallel Example: Phase 3 US1 (after T009 skeleton exists)

```bash
# After T009 (skeleton added), T010 runs alone (implementation), then:

# After T010 passes (GREEN), launch in parallel:
Task: "Extend OscalProfile with modify field (T011) in src/oscal/profile.rs"
Task: "Write CLI parse test (T014) in tests/profile_param_test.rs"

# After T011 + T012 complete, launch in parallel:
Task: "Implement execute set_params param (T018) in src/cli/profile.rs"
Task: "Write integration test (T017) in tests/profile_param_test.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Verify baseline
2. Complete Phase 2: Fix compile errors — **CRITICAL**
3. Complete Phase 3: User Story 1 (all 16 tasks)
4. **STOP and VALIDATE**: `cargo test` passes; run US1 Independent Test manually
5. This delivers the core feature — single `--set-param` flag producing correct Profile JSON

### Incremental Delivery

1. Phase 1 + Phase 2 → Build passes (prerequisite)
2. Phase 3 (US1) → Single-param tailoring works ✅ (MVP!)
3. Phase 4 (US2) → Multiple-param + aggregation confirmed by tests ✅
4. Phase 5 (US3) → Structural validity confirmed by tests ✅
5. Phase 6 → Pre-merge quality gates pass ✅

### TDD Discipline (Per Plan Constitution Gate IV)

For every implementation task (T010, T012, T015, T016, T018, T019):

1. Write test first (RED — compile or runtime fail)
2. Add minimal implementation
3. Confirm test PASSES (GREEN)
4. Move to next task

**Never implement before confirming the test fails.**

---

## Notes

- `[P]` tasks operate on different files or independent units with no in-progress dependencies
- `[US1]`, `[US2]`, `[US3]` labels map tasks to PRD acceptance criteria and user stories
- Snapshot tests use `insta` crate (already in dev-dependencies); accept initial snapshots with `INSTA_UPDATE=always cargo test`
- The C-2 warning (T019) emits to stderr — integration tests should assert on stdout only
- `parse_set_param_pairs` uses `chunks_exact(2)` — clap's `num_args = 2` guarantees even-length `Vec` so remainder is always empty
- Pre-merge gates (plan.md): `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test --workspace`, `cargo test --doc`
- Do NOT validate param-id values against the source catalog — that is WI-32's responsibility (AR guardrail, SEC-1)
- Do NOT implement `alter` directives or `merge` section — stay focused on `set-parameters` only (PRD W-2, W-4, AR guardrail)
- SEC requirements traceability: SEC-2 → T014/T015; SEC-3 → T029; SEC-4 → T021/T022/T024; SEC-5 → T027/T028; SEC-6 → T009/T010
