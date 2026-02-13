# Tasks: OSCAL Component Definition Structure

**Input**: Design documents from `/specs/014-component-definition-structure/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/component_definition.rs, quickstart.md
**Branch**: `014-component-definition-structure`

**Tests**: Required — TDD mandatory per constitution (IV), spec SC-005 (>90% coverage), and project testing rules.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

**Note on AR Guardrail Override**: The AR recommends `serde_json::Value` builder pattern, but research R-1 found the actual Catalog builder uses typed structs with `#[derive(Serialize)]`. This plan follows the **actual codebase pattern** (typed structs) per research.md R-1. See `specs/014-component-definition-structure/research.md` for full rationale.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add shared constants and error variants that all downstream tasks depend on.

- [x] T001 [P] Add `COMPONENT_NAMESPACE` UUID v5 namespace constant (derived from `Uuid::new_v5(&FORGE_NAMESPACE_UUID, b"component")`) with hardcoded bytes and derivation unit test in `src/uuid.rs`
- [x] T002 [P] Add `ComponentDefinitionBuild(String)` error variant to `ForgeError` enum with display message `"Component definition build error: {0}"` and add unit test for display formatting in `src/error.rs`

**Checkpoint**: Shared infrastructure ready — `COMPONENT_NAMESPACE` and error variant available for the builder module.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Register the new module, define type contracts, and wire re-exports. MUST be complete before any user story tests or implementation can compile.

**Warning**: No user story work can begin until this phase is complete.

- [x] T003 Add `pub mod component_definition;` declaration and public re-exports (`ComponentDefinitionEnvelope`, `ComponentDefinition`, `ComponentDefinitionMetadata`, `DocumentaryComponent`, `build_component_definition`, `DEFAULT_COMPONENT_TITLE`) in `src/oscal/mod.rs`
- [x] T004 Create `src/oscal/component_definition.rs` with type struct definitions (`ComponentDefinitionEnvelope`, `ComponentDefinition`, `ComponentDefinitionMetadata`, `DocumentaryComponent`), `DEFAULT_COMPONENT_TITLE` constant, and stub `build_component_definition` function returning `todo!()` — all per `contracts/component_definition.rs`
- [x] T005 Add public re-exports for Component Definition types (`ComponentDefinitionEnvelope`, `ComponentDefinition`, `ComponentDefinitionMetadata`, `DocumentaryComponent`, `build_component_definition`) in `src/lib.rs`

**Checkpoint**: Module registered, types defined, re-exports wired. `cargo check` passes (stub function compiles). User story implementation can now begin.

---

## Phase 3: User Story 1 — Generate Component Definition Structure (Priority: P1) :dart: MVP

**Goal**: Build a `ComponentDefinitionEnvelope` from a `PolicyDocument` with correct root key, metadata, one documentary component of type `"policy"`, deterministic UUID v5, and optional back matter.

**Independent Test**: Given a PolicyDocument with title "Corporate Security Policy" and version "2.0", `build_component_definition` returns a `ComponentDefinitionEnvelope` whose serialized JSON has root key `"component-definition"`, correct metadata, and a `components` array with one documentary component of type `"policy"`.

**Traces to**: M-1, M-2, M-3, M-4, M-5, M-6, S-1, S-2, SEC-1 through SEC-4, EC-1 through EC-5, AC-1 through AC-5

### Tests for User Story 1 (TDD — write FIRST, verify they FAIL)

> **NOTE: Write these tests FIRST. Run `cargo test` after each test task to confirm tests FAIL (compile errors from `todo!()` stub are expected). Implementation follows in T012–T013.**

- [x] T006 [US1] Write unit test `test_happy_path_root_key_and_metadata` verifying: (1) serialized JSON has root key `"component-definition"`, (2) `metadata.title` matches PolicyDocument title, (3) `metadata.version` matches PolicyDocument version, (4) `metadata.oscal-version` is `"1.2.0"`, (5) `metadata.last-modified` is present and non-empty, (6) `uuid` is present and non-empty in `src/oscal/component_definition.rs` (AC-1, AC-2, M-1, M-2, EC-5)
- [x] T007 [US1] Write unit test `test_documentary_component_structure` verifying: (1) `components` array has exactly 1 entry, (2) `type` is `"policy"`, (3) `title` matches PolicyDocument title, (4) `description` matches template `"Documentary component representing the {title} policy document."`, (5) `uuid` is present and non-empty in `src/oscal/component_definition.rs` (AC-3, AC-5, M-3, M-5, M-6)
- [x] T008 [US1] Write unit test `test_component_uuid_determinism` verifying: (1) same PolicyDocument produces same component UUID across two calls, (2) different title produces different UUID, (3) different version produces different UUID, (4) component UUID differs from document-level UUID in `src/oscal/component_definition.rs` (AC-4, M-4, EC-4)
- [x] T009 [US1] Write unit tests for edge cases: `test_empty_title_defaults` verifying component title is `"Untitled Policy Document"` and description uses default title (EC-1, SEC-3); `test_empty_version_defaults` verifying `metadata.version` is `"0.0.0"` (EC-2, SEC-4); `test_empty_sections_produces_valid_output` verifying builder succeeds with no sections (EC-3) in `src/oscal/component_definition.rs`
- [x] T010 [US1] Write unit tests for security requirements: `test_no_remarks_in_output` verifying serialized JSON contains no `"remarks"` key anywhere (SEC-2); `test_no_extra_data_beyond_policy_document` verifying only PolicyDocument-derived and OSCAL-convention fields appear (SEC-1) in `src/oscal/component_definition.rs`
- [x] T011 [US1] Write unit tests for Should Have requirements: `test_empty_control_implementations` verifying `control-implementations` is an empty array in JSON output (S-1); `test_back_matter_included_with_citations` verifying back matter appears when PolicyDocument has citations and is absent when it has none (S-2) in `src/oscal/component_definition.rs`

### Implementation for User Story 1

- [x] T012 [US1] Implement `generate_component_uuid` private helper function using `Uuid::new_v5(&COMPONENT_NAMESPACE, format!("{}{}", title, version).as_bytes())` in `src/oscal/component_definition.rs`
- [x] T013 [US1] Implement `build_component_definition` function: (1) call `assemble_metadata(&document.metadata, None)` for metadata, (2) resolve title with `DEFAULT_COMPONENT_TITLE` fallback, (3) build `DocumentaryComponent` with `generate_component_uuid`, type `"policy"`, template description, empty `control_implementations`, (4) call `generate_back_matter` for citations, (5) assemble `ComponentDefinitionEnvelope` with document UUID from metadata, mapped metadata fields, components vec, and optional back matter in `src/oscal/component_definition.rs`

**Checkpoint**: All US1 tests pass. `cargo test component_definition` succeeds. The Component Definition builder produces correct JSON for all happy path, edge case, and security scenarios.

---

## Phase 4: User Story 2 — Reuse Metadata Assembly Pattern (Priority: P1)

**Goal**: Verify that the Component Definition metadata is structurally consistent with Catalog metadata, confirming WI-11 `assemble_metadata` reuse.

**Independent Test**: Build both a Catalog and a Component Definition from the same PolicyDocument and verify their metadata blocks have the same `oscal-version`, matching title/version derivation, and equivalent field structure.

**Traces to**: M-7, AC-6, US2 acceptance scenarios

### Tests for User Story 2 (TDD — write FIRST, verify they PASS since T013 already implements reuse)

- [x] T014 [US2] Write integration test `test_metadata_consistency_with_catalog` that builds both a `CatalogEnvelope` (via `build_catalog`) and a `ComponentDefinitionEnvelope` (via `build_component_definition`) from the same PolicyDocument, then verifies: (1) both have `oscal-version` of `"1.2.0"`, (2) both have matching `title`, (3) both have matching `version`, (4) both have non-empty `last-modified` timestamps in `src/oscal/component_definition.rs` (AC-6, M-7)
- [x] T015 [US2] Write unit test `test_version_default_consistency` verifying that a PolicyDocument with empty version produces `metadata.version` of `"0.0.0"` in the Component Definition (matching Catalog default behavior) in `src/oscal/component_definition.rs` (EC-2, US2-AC-2)

### Implementation for User Story 2

- [x] T016 [US2] Run US2 tests and verify they pass — no new code expected since `build_component_definition` already calls `assemble_metadata` (T013). If any test fails, fix the metadata mapping in `src/oscal/component_definition.rs`

**Checkpoint**: All US1 and US2 tests pass. Metadata reuse pattern is verified.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Code quality verification and final validation.

- [x] T017 [P] Run `cargo fmt --check` and fix any formatting issues
- [x] T018 [P] Run `cargo clippy -- -D warnings` and fix any warnings
- [x] T019 Run full test suite `cargo test` and verify all existing tests still pass (no regressions)
- [x] T020 Verify >90% test coverage for the component definition module (SC-005)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately. T001 and T002 are parallel (different files).
- **Foundational (Phase 2)**: Depends on Phase 1 completion (T004 imports `COMPONENT_NAMESPACE` from uuid.rs and `ForgeError` from error.rs). T003→T004→T005 are sequential.
- **User Story 1 (Phase 3)**: Depends on Phase 2 completion. Tests (T006–T011) are sequential within the same file. Implementation (T012→T013) is sequential.
- **User Story 2 (Phase 4)**: Depends on Phase 3 implementation completion (T013). Tests (T014–T015) verify reuse pattern.
- **Polish (Phase 5)**: Depends on all user stories being complete.

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) — no dependency on other stories
- **User Story 2 (P1)**: Depends on US1 implementation (T013) since the metadata integration test requires a working `build_component_definition`

### Within Each User Story

- Tests MUST be written and FAIL before implementation (TDD)
- Implementation follows test completion
- `generate_component_uuid` (T012) before `build_component_definition` (T013) — helper used by builder
- Story complete before moving to next

### Parallel Opportunities

- T001 and T002 can run in parallel (different files, no dependency)
- T017 and T018 can run in parallel (independent lint/format checks)
- US1 and US2 cannot run in parallel (US2 depends on US1 implementation)

---

## Parallel Example: Phase 1

```bash
# Launch both setup tasks in parallel (different files):
Task: "Add COMPONENT_NAMESPACE constant in src/uuid.rs"
Task: "Add ComponentDefinitionBuild error variant in src/error.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T002, parallel)
2. Complete Phase 2: Foundational (T003–T005, sequential)
3. Complete Phase 3: User Story 1 Tests (T006–T011, sequential in same file)
4. Complete Phase 3: User Story 1 Implementation (T012–T013)
5. **STOP and VALIDATE**: Run `cargo test component_definition` — all US1 tests must pass
6. Proceed to Phase 4: User Story 2

### Incremental Delivery

1. Phase 1 + Phase 2 → Foundation ready (types compile, stubs exist)
2. Phase 3 → US1 complete (builder works, all tests pass) — **MVP!**
3. Phase 4 → US2 complete (metadata consistency verified)
4. Phase 5 → Polish (clippy, fmt, coverage)

---

## Requirement Traceability

| Requirement | Task(s) | Type |
|-------------|---------|------|
| M-1 (root key) | T006, T013 | Test + Impl |
| M-2 (metadata) | T006, T009, T013 | Test + Impl |
| M-3 (type policy) | T007, T013 | Test + Impl |
| M-4 (deterministic UUID) | T008, T012 | Test + Impl |
| M-5 (title derivation) | T007, T009, T013 | Test + Impl |
| M-6 (description template) | T007, T013 | Test + Impl |
| M-7 (metadata reuse) | T014, T016 | Test + Verify |
| S-1 (empty control-implementations) | T011, T013 | Test + Impl |
| S-2 (back matter) | T011, T013 | Test + Impl |
| SEC-1 (no extra data) | T010 | Test |
| SEC-2 (no remarks) | T010 | Test |
| SEC-3 (empty title handling) | T009 | Test |
| SEC-4 (missing version handling) | T009 | Test |
| EC-1 (empty title default) | T009 | Test |
| EC-2 (empty version default) | T009, T015 | Test |
| EC-3 (empty sections) | T009 | Test |
| EC-4 (UUID changes with input) | T008 | Test |
| EC-5 (JSON parseable, root key) | T006 | Test |
| AC-1 through AC-6 | T006–T014 | Test |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- All tests are in-module (`#[cfg(test)] mod tests` in `src/oscal/component_definition.rs`)
- Same-file test tasks are NOT marked [P] since they share a single file
- AR guardrail "DO NOT create typed structs" is overridden by research R-1 (actual Catalog pattern uses typed structs)
- AR guardrail "DO NOT populate control-implementations" is respected (S-1: empty array only)
- AR guardrail "MUST produce root key component-definition" is verified in T006
- AR guardrail "MUST set type to policy" is verified in T007
- AR guardrail "MUST derive title from PolicyDocument" is verified in T007
- Commit after each phase or logical group of tasks
