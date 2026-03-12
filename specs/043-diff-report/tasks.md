# Tasks: OSCAL Diff Report

**Input**: Design documents from `/specs/043-diff-report/`
**Prerequisites**: plan.md ✅, spec.md ✅, prd.md ✅, ar.md ✅, sec.md ✅, contracts/diff.rs ✅, data-model.md ✅

**Tests**: Included — TDD mandatory per project constitution (plan.md Phase 1 status).

**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- Exact file paths included in every description

---

## Phase 1: Setup (Module Structure)

**Purpose**: Create the `src/diff/` module directory with a stub entry point.

- [x] T001 Create stub `src/diff/mod.rs` (empty, no exports yet) to initialize the module directory

**Checkpoint**: `src/diff/` directory exists in the project tree

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core types and error variants that ALL user stories depend on. No story work can begin until this phase is complete.

**⚠️ CRITICAL**: `DiffEntry`, `ControlSnapshot`, and `ForgeError` variants must exist before any engine, extractor, formatter, or CLI code can compile.

- [x] T002 Define `ArtifactType`, `ControlSnapshot`, `FieldChange`, `DiffEntry`, `DiffSummary`, `DiffReport` in `src/diff/types.rs` (matches `contracts/diff.rs` exactly)
- [x] T003 Add `ForgeError::DiffHasChanges` and `ForgeError::DiffError(String)` variants to `src/error.rs`
- [x] T004 Add `pub mod diff;` declaration to `src/lib.rs`

**Checkpoint**: Foundation ready — `cargo build` compiles with empty `src/diff/mod.rs`; all user story phases can now proceed.

---

## Phase 3: User Story 1 — Compare Two Conversion Outputs (Priority: P1) 🎯 MVP

**Goal**: `forge diff old-catalog.json new-catalog.json` loads two Catalog JSONs, detects artifact type, extracts controls, classifies Added/Removed/Changed entries (including co-occurring UUID changes in `Changed`), builds a `DiffReport`, and prints a human-readable report to stdout. Exit code 0 = no changes, 1 = changes found, 2 = error.

**Independent Test**: Given `old.json` (10 controls) and `new.json` (12 controls, 2 new, 1 changed description), running `forge diff old.json new.json` prints a report with "Added: 2, Changed: 1" in the summary and exits with code 1.

### Tests for User Story 1 (TDD — write FIRST, confirm they FAIL before implementing)

- [x] T005 [US1] Write failing unit tests for `extract_controls` Catalog path (flat group, nested groups, empty groups, missing `id`, and controls nested ≥2 levels deep to verify FR-007 recursive traversal) in `src/diff/extractor.rs` `#[cfg(test)]`
- [x] T006 [P] [US1] Write failing unit tests for `compare_controls` covering AC-2 (added), AC-3 (removed), AC-4 (changed with field detail — including title, description, and statement[N] field_name labels), EC-1 (identical/no changes), EC-2 (all added), EC-3 (all removed), EC-6 (title-only change), EC-7 (same UUID different content → `Changed{uuid_changed:false}`), and a bulk-change scenario (≥10 simultaneous changes — verifies summary counts and sort order at scale) in `src/diff/engine.rs` `#[cfg(test)]`
- [x] T007 [P] [US1] Write failing unit tests for `format_diff_report` covering AC-6 (readable output), AC-8 (summary section with counts), "No differences found" case in `src/diff/formatter.rs` `#[cfg(test)]`
- [x] T008 [P] [US1] Write failing unit tests for `diff_artifacts` error handling covering AC-7/EC-4 (type mismatch error), EC-5 (missing file error), invalid JSON error, non-OSCAL JSON error, and AC-1 (valid Catalog diff produces `DiffReport`) in `src/diff/mod.rs` `#[cfg(test)]` (use stable substring assertions per Constitution Principle VIII — do not assert full error message strings)

### Implementation for User Story 1

- [x] T009 [US1] Implement `extract_controls` for `ArtifactType::Catalog` (recursive `groups[].controls[]` traversal per plan.md algorithm) in `src/diff/extractor.rs`
- [x] T010 [US1] Implement `compare_controls` (set-based HashMap comparison: Added/Removed/Changed detection with `field_changes` for title/description/parts_prose — producing FieldChange entries with `field_name` of `"title"`, `"description"`, or `"statement[N]"` respectively; `uuid_changed: bool` flag in `Changed`; sort output by control_id ascending per FR-010) in `src/diff/engine.rs`
- [x] T011 [P] [US1] Implement `build_summary` (count entries by variant, derive `unchanged = total_old - removed - changed - uuid_changes` — controls in old that were neither removed nor matched as changed/uuid-changed) in `src/diff/engine.rs`
- [x] T012 [US1] Implement `format_diff_report` (summary header + Added/Changed/Removed/UUID Stability sections with "(none)" for empty sections per `contracts/diff.rs` format spec) in `src/diff/formatter.rs`
- [x] T013 [US1] Implement `diff_artifacts` orchestration (file existence validation → serde_json parse → root key type detection → same-type validation → `extract_controls` × 2 → `compare_controls` → `build_summary` → return `DiffReport`) in `src/diff/mod.rs` (entries arrive pre-sorted from `compare_controls` per T010; no re-sorting here)
- [x] T014 [US1] Add public re-exports (`pub use diff_artifacts`, `pub use format_diff_report`, `pub mod types`) and `pub mod` declarations in `src/diff/mod.rs`
- [x] T015 [US1] Implement `execute(old_path, new_path) -> Result<bool, ForgeError>` CLI handler (calls `diff_artifacts`, formats report, prints to stdout, returns `Ok(true)` if changes found) in `src/cli/diff.rs`
- [x] T016 [US1] Add `Commands::Diff { old_artifact: PathBuf, new_artifact: PathBuf }` variant and dispatch arm (`Commands::Diff` → `cli::diff::execute` → `Err(DiffHasChanges)` if `Ok(true)`) to `src/cli/mod.rs`
- [x] T017 [US1] Add `Err(ForgeError::DiffHasChanges) => ExitCode::from(1u8)` match arm (no `eprintln`) to `src/main.rs`
- [x] T018 [US1] Add `tracing::info!` for file paths and artifact type; `tracing::debug!` for control counts in `src/cli/diff.rs` and `src/diff/mod.rs`

**Checkpoint**: `forge diff old-catalog.json new-catalog.json` works end-to-end for Catalog artifacts; all US1 tests pass; exit codes 0/1/2 correct.

---

## Phase 4: User Story 2 — Detect ID Stability Changes (Priority: P1)

**Goal**: Extend the diff engine to emit standalone `DiffEntry::UuidChanged` entries when a control's UUID differs but all diffable fields are identical. The formatter shows the "UUID Stability Changes" section with `!` prefix lines. The `DiffSummary.uuid_changes` counter counts only standalone `UuidChanged` entries (NOT `Changed{uuid_changed:true}`).

**Independent Test**: Given two Catalog JSONs where "POL-AC-001" has the same title/description/parts but a different UUID, running `forge diff` shows "UUID Stability Changes: 1" in the summary and `! POL-AC-001  <old_uuid>  →  <new_uuid>` in the UUID section.

> Note: `UuidChanged` variant and `uuid_changed` field in `Changed` are already defined in `src/diff/types.rs` (Phase 2). This phase extends the engine logic and formatter to handle them correctly.

### Tests for User Story 2 (TDD — write FIRST, confirm they FAIL)

- [x] T019 [US2] Write failing unit tests for UUID stability detection covering AC-5 (`UuidChanged` emitted when UUID differs, fields same), "no UUID changes when UUIDs identical" case, and co-occurrence rule (`Changed{uuid_changed:true}` when both UUID and fields differ — does NOT increment `uuid_changes`) in `src/diff/engine.rs` `#[cfg(test)]`
- [x] T020 [P] [US2] Write failing unit tests for `format_diff_report` UUID Stability Changes section (populated case, `(none)` case, `!` prefix format) in `src/diff/formatter.rs` `#[cfg(test)]`

### Implementation for User Story 2

- [x] T021 [US2] Extend `compare_controls` to emit `DiffEntry::UuidChanged` when UUID differs AND all field comparisons are equal (per `contracts/diff.rs` classification rules) in `src/diff/engine.rs`
- [x] T022 [US2] Extend `build_summary` in `src/diff/engine.rs` to correctly count `uuid_changes`: increment only for standalone `UuidChanged` entries, not for `Changed{uuid_changed:true}` (T019's co-occurrence counter test verifies this)

**Checkpoint**: UUID stability detection works; AC-5 passes; `uuid_changes` counter correct; UUID section renders correctly in report.

---

## Phase 5: User Story 3 — Diff Component Definitions (Priority: P2)

**Goal**: Extend `extract_controls` to support `ArtifactType::ComponentDefinition` by traversing `component-definition.components[].control-implementations[].implemented-requirements[]`, keyed by `control-id`, capturing `uuid` and `description` (mapped to `parts_prose`). All existing diff/format/CLI code works unchanged.

**Independent Test**: Given two Component Definition JSONs with different `implemented-requirements`, running `forge diff old-comp.json new-comp.json` shows correct added/removed/changed counts.

### Tests for User Story 3 (TDD — write FIRST, confirm they FAIL)

- [x] T023 [US3] Write failing unit tests for `extract_controls` Component Definition path (populated components, multiple `control-implementations`, missing `control-id`, empty components) in `src/diff/extractor.rs` `#[cfg(test)]`
- [x] T024 [P] [US3] Write failing unit tests for `diff_artifacts` end-to-end with Component Definition fixtures (two Component Definition JSONs with known changes) in `src/diff/mod.rs` `#[cfg(test)]`

### Implementation for User Story 3

- [x] T025 [US3] Extend `extract_controls` to handle `ArtifactType::ComponentDefinition` (traverse `component-definition.components[].control-implementations[].implemented-requirements[]`; map `control-id` → `ControlSnapshot` with `uuid` and `ir["description"]` → `ControlSnapshot.description`; `title = None`, `parts_prose = vec![]`) in `src/diff/extractor.rs`
- [x] T026 [US3] Extend `diff_artifacts` root-key detection in `src/diff/mod.rs` to recognize `"component-definition"` and route to the `ComponentDefinition` extraction path; confirm T024's end-to-end test passes with this routing

**Checkpoint**: All three user stories work independently; Component Definition diffs produce correct reports.

---

## Phase N: Polish & Cross-Cutting Concerns

**Purpose**: Quality gates, validation, and SEC requirements verification.

- [x] T027 [P] Run `cargo fmt --check` and `cargo clippy -- -D warnings`; fix all violations across `src/diff/`, `src/cli/diff.rs`, `src/error.rs`, `src/lib.rs`, `src/main.rs`
- [x] T028 [P] Run `cargo test` and verify all acceptance criteria pass: AC-1 through AC-8, EC-1 through EC-7, SEC-2 through SEC-7; manually confirm bulk-change scenario (spec.md edge case: many controls changing simultaneously) produces readable output with summary counts at top; verify SEC-1 sensitivity guidance appears in `spec.md` Assumptions and `quickstart.md` (no enforcement required — documentation review only)
- [x] T029 Run `quickstart.md` validation scenarios manually to confirm end-to-end CLI behavior

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 — **BLOCKS all user stories**
- **US1 (Phase 3)**: Depends on Phase 2 completion
- **US2 (Phase 4)**: Depends on Phase 3 completion (extends engine and formatter from US1)
- **US3 (Phase 5)**: Depends on Phase 2 completion only; Phase 3 is not a blocking dependency (shares extractor.rs — typically sequenced after Phase 3 in practice)
- **Polish (Phase N)**: Depends on all desired stories complete

### User Story Dependencies

- **US1 (P1)**: Starts after Phase 2 — no dependency on US2 or US3
- **US2 (P1)**: Starts after US1 (extends `compare_controls` and `format_diff_report` built in US1)
- **US3 (P2)**: Starts after Phase 2 (foundational types required); no dependency on US1 or US2 (extends `extract_controls` only). Typically sequenced after Phase 3 for practical reasons (shared file), but Phase 3 is not a blocking dependency.

### Within Each User Story

1. Write ALL test tasks first; confirm they **FAIL** with `cargo test`
2. Implement in order: types → extraction → engine → formatter → orchestration → CLI
3. Run `cargo test` after each task to track RED → GREEN transition
4. Complete the story before moving to the next priority

### Parallel Opportunities

- T006, T007, T008 can run in parallel (different files, all tests)
- T011 can run in parallel with T010 (same file, but `build_summary` has no dependency on `compare_controls` internals)
- T018 can run in parallel with T015-T017
- T019 and T020 can run in parallel (different files)
- T023 and T024 can run in parallel (different files)
- T027 and T028 can run in parallel

---

## Parallel Example: User Story 1 Tests

```bash
# Launch all US1 tests in parallel (write them all, then run once):
Task: "Write failing extractor tests in src/diff/extractor.rs"  # T005
Task: "Write failing engine tests in src/diff/engine.rs"        # T006
Task: "Write failing formatter tests in src/diff/formatter.rs"  # T007
Task: "Write failing mod tests in src/diff/mod.rs"              # T008
# Then: cargo test -- to confirm all 4 fail (RED phase complete)
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2 Only)

1. Complete Phase 1: Setup (T001)
2. Complete Phase 2: Foundational (T002–T004) — blocks everything
3. Complete Phase 3: User Story 1 (T005–T018) — core diff for Catalogs
4. Complete Phase 4: User Story 2 (T019–T022) — UUID stability
5. **STOP and VALIDATE**: Run `cargo test`; run `quickstart.md` scenarios
6. US3 (Component Definition) is Should Have — ship MVP without it if needed

### Incremental Delivery

1. Setup + Foundational → types and error variants compiled
2. US1 → `forge diff` works for Catalog JSONs (exit 0/1/2 correct)
3. US2 → UUID stability detection surfaced in report
4. US3 → Component Definition support added
5. Polish → quality gates pass, ready for Phase 1 release (WI-25)

---

## Notes

- **[P]** tasks operate on different files or have no blocking dependencies
- **[Story]** label maps each task to its user story for traceability
- TDD is mandatory: every test task must **fail** before its paired implementation task runs
- `DiffEntry::UuidChanged` vs `Changed{uuid_changed:true}` co-occurrence rule is a critical distinction — see `contracts/diff.rs` classification rules and data-model.md
- SEC-2 through SEC-7 are all covered by US1 test tasks (T008 covers SEC-2 through SEC-6; T007 covers SEC-7 via sort-order test)
- `src/diff/mod.rs` is used for both the module stub (T001), foundational wiring (T014), and orchestration (T013) — update it incrementally across phases
