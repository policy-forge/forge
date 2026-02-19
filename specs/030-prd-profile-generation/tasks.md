---
description: "Task list for 030-prd-profile-generation"
---

# Tasks: 030-prd-profile-generation — OSCAL Profile Generation

**Input**: specs/030-prd-profile-generation/plan.md, spec.md, data-model.md, contracts/, research.md
**Branch**: `030-prd-profile-generation`
**Tests**: TDD mandatory — tests written FIRST and must FAIL before implementation begins (>90% coverage target)

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- **[TDD RED]**: Write test first; verify it FAILS before writing implementation

---

## Phase 1: Setup (File Structure)

**Purpose**: Register the new module in the existing crate so compilation succeeds before any logic is written.

- [X] T001 Add `pub mod profile;` to src/oscal/mod.rs and re-export `OscalProfile`, `ProfileRoot`, `ProfileImport`, `ControlSelection`, `SelectionMode`, `build_profile`, `parse_control_ids`
- [X] T002 Create src/oscal/profile.rs with empty module stubs (struct declarations with `todo!()` stubs or unit-empty impls so the crate compiles)

**Checkpoint**: `cargo build` compiles with no errors before any logic exists

---

## Phase 2: Foundational (Core Types + ID Parsing)

**Purpose**: OSCAL Profile type definitions and `parse_control_ids` — required by every user story. No user story work begins until this phase is complete.

**⚠️ CRITICAL**: Phases 3–5 are blocked until T003–T006 are complete.

- [X] T003 [P] Write unit tests for OSCAL type serialization in src/oscal/profile.rs: `ProfileRoot` serializes with `"profile"` root key; `ProfileImport` with include path produces `"include-controls"` key and omits `"exclude-controls"`; `ProfileImport` with exclude path produces `"exclude-controls"` key and omits `"include-controls"`; `ControlSelection` serializes `with_ids` as `"with-ids"` [TDD RED — must fail before T005]
- [X] T004 [P] Write unit tests for `parse_control_ids` in src/oscal/profile.rs: trims whitespace, deduplicates (order-preserving, first occurrence kept), removes empty tokens after trim, errors on empty string input, handles single ID with no comma [TDD RED — must fail before T006]
- [X] T005 [P] Implement `ProfileRoot`, `OscalProfile`, `ProfileImport`, `ControlSelection`, `SelectionMode` structs in src/oscal/profile.rs with all serde attributes from contracts/profile_types.rs (makes T003 pass)
- [X] T006 [P] Implement `parse_control_ids` in src/oscal/profile.rs: split on `,`, trim each token, filter empty, dedup order-preserving, return `ForgeError::InvalidArgument` if result is empty (makes T004 pass)

**Checkpoint**: `cargo test --lib` passes T003–T006; `parse_control_ids` and all type serialization tests green

---

## Phase 3: User Story 1 — Include-Based Control Selection (Priority: P1) 🎯 MVP

**Goal**: `forge profile --catalog <path> --include <ids>` produces valid OSCAL v1.2.0 Profile JSON to stdout.

**Independent Test**: Run `forge profile --catalog /tmp/test-catalog.json --include POL-AC-001,POL-AC-002` and verify stdout contains `imports[0].include-controls[0].with-ids = ["POL-AC-001","POL-AC-002"]` and `imports[0].href = "/tmp/test-catalog.json"`.

### Tests for User Story 1

- [X] T007 [US1] Write unit tests for `build_profile` include path in src/oscal/profile.rs: `href` matches `catalog_path` argument; `include_controls` is `Some` with correct `with_ids`; `exclude_controls` is `None`; `metadata.title == "Policy Baseline Profile"`; `metadata.oscal_version == "1.2.0"`; security test: serialized JSON contains no catalog content beyond href; security test: href stored exactly as provided (no normalization) [TDD RED]
- [X] T008 [P] [US1] Write integration tests for --include path and general CLI validation in tests/profile_generation_test.rs: happy path (AC-2), missing catalog returns `Io` error (AC-8), no selection flag returns `InvalidArgument` error (EC-3; general validation, not US1-specific), `forge profile --help` output contains `--catalog`, `--include`, and `--exclude` (AC-1) [TDD RED]

### Implementation for User Story 1

- [X] T009 [US1] Implement `build_profile` in src/oscal/profile.rs: accept `catalog_path: &str`, `control_ids: Vec<String>`, `mode: SelectionMode`; build `OscalProfile` using `assemble_metadata` with `DocumentMetadata { title: "Policy Baseline Profile".to_string(), version: "1.0.0".to_string(), ..Default::default() }`; populate `include_controls` or `exclude_controls` based on `mode`; error if `control_ids` is empty (makes T007 pass)
- [X] T010 [P] [US1] Add `Profile { catalog: PathBuf, include: Option<String>, exclude: Option<String>, format: OutputFormat, output: Option<PathBuf> }` variant to `Commands` enum in src/cli/mod.rs with `#[arg(long, conflicts_with = "exclude")]` on `include` and `#[arg(long, conflicts_with = "include")]` on `exclude`
- [X] T011 [US1] Create src/cli/profile.rs with `pub fn execute(catalog: &PathBuf, include: Option<&str>, exclude: Option<&str>, format: &OutputFormat, output: Option<&Path>) -> Result<(), ForgeError>`: validate exactly one of include/exclude is `Some` (clap handles both-provided case; this handles neither); check `catalog.exists()` returning `ForgeError::Io`; call `parse_control_ids`; determine `SelectionMode`; call `build_profile`; wrap in `ProfileRoot`; serialize with `serde_json::to_string_pretty`; write to stdout (makes T008 integration tests compilable)
- [X] T012 [US1] Add `Commands::Profile { catalog, include, exclude, format, output } => profile::execute(catalog, include.as_deref(), exclude.as_deref(), format, output.as_deref())` dispatch arm to `execute()` in src/cli/mod.rs and add `pub mod profile;` to src/cli/mod.rs (makes T008 integration tests pass)

**Checkpoint**: `cargo test` passes all US1 tests; `forge profile --catalog /tmp/catalog.json --include POL-AC-001` outputs valid Profile JSON to stdout

---

## Phase 4: User Story 2 — Exclude-Based Control Selection (Priority: P1)

**Goal**: `forge profile --catalog <path> --exclude <ids>` produces valid OSCAL v1.2.0 Profile JSON with `exclude-controls`.

**Independent Test**: Run `forge profile --catalog /tmp/test-catalog.json --exclude POL-AC-003` and verify stdout contains `imports[0].exclude-controls[0].with-ids = ["POL-AC-003"]` and no `include-controls` key.

### Tests for User Story 2

- [X] T013 [P] [US2] Write unit tests for `build_profile` exclude path in src/oscal/profile.rs: `exclude_controls` is `Some` with correct `with_ids`; `include_controls` is `None`; JSON output omits `"include-controls"` key entirely [TDD RED]
- [X] T014 [P] [US2] Write integration test for --exclude happy path in tests/profile_generation_test.rs: AC-3 scenario — `forge profile --catalog catalog.json --exclude POL-AC-003` produces `exclude-controls` with `with-ids: ["POL-AC-003"]` [TDD RED]

### Implementation for User Story 2

- [X] T015 [US2] Complete the Exclude path: if T009's `SelectionMode::Exclude` branch is incomplete, implement it now; verify `execute()` in src/cli/profile.rs passes `SelectionMode::Exclude` when `exclude` is `Some`; run `cargo test` to confirm T013 and T014 pass (makes T013, T014 pass)

**Checkpoint**: `cargo test` passes all US1 and US2 tests; both `--include` and `--exclude` paths produce correct OSCAL Profile JSON

---

## Phase 5: User Story 3 — File Output (Priority: P2)

**Goal**: `forge profile --catalog <path> --include <ids> --output baseline.json` writes valid Profile JSON to the specified file instead of stdout.

**Independent Test**: Run `forge profile --catalog /tmp/test-catalog.json --include POL-AC-001 --output /tmp/baseline.json` and verify `/tmp/baseline.json` exists and contains valid OSCAL Profile JSON.

### Tests for User Story 3

- [X] T016 [US3] Write integration test for --output file creation in tests/profile_generation_test.rs: `forge profile --catalog catalog.json --include POL-AC-001 --output <tempfile>` — verify file exists and contains valid Profile JSON (AC-6); also verify no --output writes to stdout (AC-7) [TDD RED]

### Implementation for User Story 3

- [X] T017 [US3] Add `--output` file-writing to src/cli/profile.rs execute(): when `output` is `Some(path)`, write serialized JSON to file using `std::fs::write` (reusing export.rs pattern); when `None`, print to stdout (makes T016 pass)

**Checkpoint**: `cargo test` passes all US1, US2, and US3 tests; file output and stdout output both work correctly

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Edge case coverage, error message quality, and final verification gates.

- [X] T018 Write integration tests for mutual exclusivity (AC-9), duplicate ID deduplication (EC-4), whitespace trimming (EC-2), single-ID no-comma (EC-1), empty --include string (EC-5), `--format json` produces JSON output (S-1) in tests/profile_generation_test.rs
- [X] T019 [P] Run `cargo clippy -- -D warnings` and fix any warnings in src/oscal/profile.rs and src/cli/profile.rs
- [X] T020 [P] Run `cargo fmt --check` and fix any formatting issues across all modified files
- [X] T021 [P] Add `#[tracing::instrument(skip_all)]` to `build_profile` and `parse_control_ids` in src/oscal/profile.rs and a `tracing::info!` log in `execute()` in src/cli/profile.rs after successful generation (log catalog path and number of selected controls at INFO level; required by AR observability spec and Constitution §IX)
- [X] T022 [P] Verify all public items in src/oscal/profile.rs (`build_profile`, `parse_control_ids`, `OscalProfile`, `ProfileImport`, `ControlSelection`, `ProfileRoot`, `SelectionMode`) and src/cli/profile.rs (`execute`) have rustdoc comments matching the contract definitions in contracts/profile_types.rs and contracts/cli_profile.rs (required by Constitution §I)

**Checkpoint**: All 5 verification gates pass (cargo test, clippy, fmt, tracing instrumented, rustdoc complete)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 — **BLOCKS all user stories**
- **US1 (Phase 3)**: Depends on Phase 2 — no dependency on US2/US3
- **US2 (Phase 4)**: Depends on Phase 2 — no dependency on US3; `build_profile` implemented in US1 Phase 3 (T009) is required before T015
- **US3 (Phase 5)**: Depends on Phase 3 (needs execute() from T011 before adding file output in T017)
- **Polish (Phase 6)**: Depends on all prior phases

### User Story Dependencies

- **US1 (P1)**: Starts after Phase 2 — no dependency on US2/US3
- **US2 (P1)**: T013/T014 can start after Phase 2; T015 requires T009 (build_profile) to be complete
- **US3 (P2)**: Requires T011 (execute() skeleton) before T017 can extend it

### Parallel Opportunities Within Phases

- Phase 2: T003 ↔ T004 in parallel; T005 ↔ T006 in parallel (after their respective test pairs)
- Phase 3: T007 and T008 (tests) can be written in parallel; T010 (Commands variant) can be done in parallel with T007–T009
- Phase 4: T013 ↔ T014 in parallel (both integration tests for US2)
- Phase 6: T019 ↔ T020 ↔ T021 ↔ T022 in parallel

---

## Parallel Example: Phase 2 Foundational

```bash
# Write both test sets in parallel (different concerns, same file but independent):
Task: "T003 — Serialization tests for ProfileRoot/ProfileImport/ControlSelection"
Task: "T004 — parse_control_ids tests (trim, dedup, empty-error)"

# Once tests are written and FAILING, implement in parallel:
Task: "T005 — Implement type structs with serde attributes"
Task: "T006 — Implement parse_control_ids"
```

## Parallel Example: Phase 3 (US1)

```bash
# Write test skeletons in parallel:
Task: "T007 — build_profile unit tests (include path)"
Task: "T008 — Integration tests for --include, missing-catalog, no-selection"

# Implement in parallel (different files):
Task: "T010 — Profile variant in Commands enum (src/cli/mod.rs)"
# After T009 (build_profile):
Task: "T011 — execute() in src/cli/profile.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T002)
2. Complete Phase 2: Foundational (T003–T006)
3. Complete Phase 3: US1 — Include path (T007–T012)
4. **STOP and VALIDATE**: `forge profile --catalog catalog.json --include POL-AC-001,POL-AC-002` produces correct OSCAL Profile JSON

### Incremental Delivery

1. **Setup + Foundational** → types compile, parse_control_ids tested
2. **Add US1** → include-based selection works end-to-end (MVP!)
3. **Add US2** → exclude-based selection works
4. **Add US3** → file output works
5. **Polish** → clippy/fmt clean, edge cases covered

### TDD Order (MANDATORY)

Within each phase:
1. Write test → verify it FAILS (`cargo test` shows red)
2. Write minimal implementation → verify test PASSES (`cargo test` shows green)
3. Refactor → verify tests still pass

---

## Notes

- `[P]` tasks = different files or independent concerns, no blocking dependencies
- `[Story]` maps each task to a specific user story for traceability
- **No new `Cargo.toml` dependencies** — all crates already present
- **Do NOT read or parse the source Catalog file** — guardrail from AR-030
- **Do NOT generate a `modify` section** — deferred to WI-31
- Catalog existence check is in `cli/profile.rs`, not in `build_profile`
- Commit after each task or logical group
- Stop at each phase checkpoint to validate independently
