# Tasks: Deterministic UUID Generation

**Input**: Design documents from `/specs/007-uuid-generation/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/uuid-module.md

**Tests**: TDD is mandatory for this feature (per plan.md: "Testing: `cargo test` (TDD mandatory)"). Tests are written FIRST and must FAIL before implementation.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story. All 3 user stories are P1 priority — they share the same underlying functions but validate different behavioral guarantees.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add the uuid dependency and ensure domain model stub types exist for integration.

- [x] T001 Add `uuid` crate at latest stable version to Cargo.toml via `cargo add uuid --features v5`; also add `tracing` crate via `cargo add tracing` (Constitution IX, XI)
- [x] T032 Run pre-addition security checks per Constitution XI: `cargo audit` and `cargo deny check` (if cargo-deny is configured); review `cargo tree -p uuid` for transitive dependency impact
- [x] T003 [P] Define stub domain model types in src/model/mod.rs: `PolicyDocument` (with `sections: Vec<PolicySection>`), `PolicySection` (with `title: String`, `requirements: Vec<PolicyRequirement>`, `children: Vec<PolicySection>`), `PolicyRequirement` (with `text: String`, `source_line: usize`, `nesting_depth: u8`, `stable_id: Option<String>`); derive Debug, Clone, PartialEq on all types; these are minimal stubs for WI-5/WI-6

**Checkpoint**: `cargo build` passes with new dependencies added, security audit clean, and model types compiling

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Create the uuid module file with the FORGE namespace constant and register it. MUST complete before any user story implementation.

**CRITICAL**: No user story work can begin until this phase is complete.

- [x] T004 Create src/uuid.rs with module-level imports (`use uuid::Uuid;` and `use crate::model::*;`) and define `FORGE_NAMESPACE_UUID` as a `pub const Uuid` — generate a new project-specific UUID v4 (via `uuidgen` or equivalent), hardcode as `Uuid::from_bytes([...16 bytes...])` with doc comment explaining purpose and breaking-change warning (S-1, SEC-3)
- [x] T002 Register uuid module in src/lib.rs (add `pub mod uuid;` to module declarations) — must follow T004 so the module file exists

**Checkpoint**: `cargo build` passes; FORGE_NAMESPACE_UUID constant is accessible from `forge::uuid::FORGE_NAMESPACE_UUID`

---

## Phase 3: User Story 1 — Deterministic IDs Across Runs (Priority: P1) MVP

**Goal**: Same policy content always produces identical stable IDs across conversion runs (PRD M-1, M-3, M-4; AC-1, AC-4, AC-5)

**Independent Test**: Generate a UUID from the same requirement text twice and verify both UUIDs are identical. Run `assign_stable_ids` on a PolicyDocument and verify all requirements have `stable_id = Some(...)`.

### TDD Tests for User Story 1

> **Write these tests FIRST, ensure they FAIL before implementation**

- [x] T005 [US1] Write TDD test `test_normalize_for_hashing_basic` in src/uuid.rs `#[cfg(test)] mod tests`: verify `normalize_for_hashing("  foo   bar  ")` returns `"foo bar"`, and `normalize_for_hashing("already clean")` returns `"already clean"` (idempotency)
- [x] T006 [US1] Write TDD test `test_generate_stable_id_determinism` in src/uuid.rs: call `generate_stable_id("All users must use multi-factor authentication")` twice and assert both UUIDs are equal (AC-1, M-4)
- [x] T007 [US1] Write TDD test `test_generate_stable_id_uuid_v5_format` in src/uuid.rs: verify the returned UUID has version nibble = 5 (`uuid.get_version() == Some(Version::Sha1)`) and correct RFC 4122 variant bits (AC-5)

### Implementation for User Story 1

- [x] T008 [US1] Implement `normalize_for_hashing(text: &str) -> String` in src/uuid.rs using `text.split_whitespace().collect::<Vec<&str>>().join(" ")` — must be a pure function with no side effects or I/O (M-2, SEC-4)
- [x] T009 [US1] Implement `generate_stable_id(text: &str) -> Uuid` in src/uuid.rs: normalize text then call `Uuid::new_v5(&FORGE_NAMESPACE_UUID, normalized.as_bytes())` — must be a pure function with no side effects or I/O (M-1, S-2, SEC-4)
- [x] T010 [US1] Verify T005, T006, T007 tests pass with `cargo test uuid`

### Integration Tests for User Story 1

- [x] T011 [US1] Write TDD test `test_assign_stable_ids_all_populated` in src/uuid.rs: create a PolicyDocument with 3 requirements across 2 sections (some nested), call `assign_stable_ids`, assert all `stable_id` fields are `Some(...)` and no `None` remains (AC-4, M-3)
- [x] T012 [US1] Write TDD test `test_assign_stable_ids_nested_sections` in src/uuid.rs: create a PolicyDocument with 3 levels of nesting, call `assign_stable_ids`, assert requirements at all depths have `stable_id` populated (EC-5)
- [x] T013 [US1] Implement `assign_stable_ids(document: &mut PolicyDocument)` and helper `assign_stable_ids_to_section(section: &mut PolicySection)` in src/uuid.rs: recursively walk all sections and requirements, call `generate_stable_id` on each requirement's text, set `stable_id = Some(uuid.to_string())` (M-3)
- [x] T014 [US1] Verify T011, T012 tests pass with `cargo test uuid`

**Checkpoint**: User Story 1 fully functional — `cargo test uuid` passes all determinism and coverage tests

---

## Phase 4: User Story 2 — Whitespace Normalization (Priority: P1)

**Goal**: Whitespace-only edits to requirement text do not change the stable ID (PRD M-2; AC-2; EC-1, EC-2, EC-3; SEC-1, SEC-2)

**Independent Test**: Generate a UUID for a requirement, modify only whitespace, regenerate, and verify the UUID is unchanged.

### Tests for User Story 2

> **NOTE**: Implementation already exists from US1 (normalize_for_hashing + generate_stable_id). These tests validate the whitespace resilience guarantee.

- [x] T015 [P] [US2] Write test `test_whitespace_resilience_extra_spaces` in src/uuid.rs: verify `generate_stable_id("Users must change passwords every 90 days")` equals `generate_stable_id("  Users  must  change  passwords  every  90  days  ")` (AC-2)
- [x] T016 [P] [US2] Write test `test_whitespace_resilience_trailing_newline` in src/uuid.rs: verify `generate_stable_id("Requirement text")` equals `generate_stable_id("Requirement text\n")` (AC-2)
- [x] T017 [P] [US2] Write test `test_whitespace_resilience_mixed_tabs_newlines` in src/uuid.rs: verify `generate_stable_id("foo bar")` equals `generate_stable_id("foo\t\n\nbar")` (EC-2)
- [x] T018 [P] [US2] Write test `test_edge_case_empty_text` in src/uuid.rs: verify `generate_stable_id("")` returns a valid UUID and `normalize_for_hashing("")` returns `""` (EC-1, SEC-1)
- [x] T019 [P] [US2] Write test `test_edge_case_whitespace_only` in src/uuid.rs: verify `generate_stable_id("   ")` produces the same UUID as `generate_stable_id("")` and `normalize_for_hashing("   ")` returns `""` (EC-1)
- [x] T020 [P] [US2] Write test `test_edge_case_unicode_whitespace` in src/uuid.rs: verify text with non-breaking space (`\u{00A0}`), em space (`\u{2003}`) is normalized to single spaces and produces same UUID as ASCII-whitespace equivalent (EC-3, SEC-2)
- [x] T021 [US2] Verify all US2 tests pass with `cargo test uuid`

**Checkpoint**: User Story 2 verified — all whitespace resilience and edge case tests pass

---

## Phase 5: User Story 3 — Substantive Change Detection (Priority: P1)

**Goal**: Substantive text changes produce different stable IDs, enabling change detection (PRD M-5; AC-3; EC-4)

**Independent Test**: Generate UUIDs for two requirements with different substantive text and verify the UUIDs differ.

### Tests for User Story 3

> **NOTE**: Implementation already exists from US1 (generate_stable_id). These tests validate the content sensitivity guarantee.

- [x] T022 [P] [US3] Write test `test_substantive_change_different_uuid` in src/uuid.rs: verify `generate_stable_id("All users must use MFA")` does NOT equal `generate_stable_id("All administrators must use MFA")` (AC-3, M-5)
- [x] T023 [P] [US3] Write test `test_substantive_change_numeric` in src/uuid.rs: verify `generate_stable_id("Passwords must be at least 8 characters")` does NOT equal `generate_stable_id("Passwords must be at least 12 characters")` (AC-3)
- [x] T024 [P] [US3] Write test `test_no_false_collisions` in src/uuid.rs: generate UUIDs for 5+ distinct requirement texts and verify all UUIDs are unique (EC-4)
- [x] T025 [US3] Verify all US3 tests pass with `cargo test uuid`

**Checkpoint**: User Story 3 verified — all sensitivity and collision tests pass

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Cross-cutting concerns required by constitution and final verification across all stories

- [x] T033 [P] Add rustdoc documentation with `# Examples` sections to all public items in src/uuid.rs: `FORGE_NAMESPACE_UUID`, `normalize_for_hashing`, `generate_stable_id`, `assign_stable_ids` — follow contracts/uuid-module.md as the specification (Constitution III, V)
- [x] T034 [P] Add `#[tracing::instrument]` attributes to public functions in src/uuid.rs: `normalize_for_hashing`, `generate_stable_id`, `assign_stable_ids` — skip no sensitive fields needed since input is policy text (Constitution IX)
- [x] T026 [P] Add structured tracing for UUID generation in src/uuid.rs: add `use tracing::debug;` and in `assign_stable_ids_to_section`, add `debug!(normalized_text = %normalized_text, uuid = %uuid, "UUID generated")` for each requirement (C-1); `tracing` crate already added in T001 (Constitution IX mandates `tracing`, NOT `log`)
- [x] T035 [P] Add `criterion` benchmark for `generate_stable_id` in benches/uuid_benchmark.rs: benchmark UUID generation for a representative requirement string to establish a performance baseline (Constitution VI); add `criterion` dev-dependency to Cargo.toml
- [x] T027 Run `cargo fmt --check` and fix any formatting issues
- [x] T028 Run `cargo clippy -- -D warnings` and fix any lint warnings
- [x] T029 Run `cargo test` to verify all tests pass (full suite)
- [x] T030 Validate quickstart.md scenarios: verify `generate_stable_id`, `normalize_for_hashing`, and `assign_stable_ids` work as documented in specs/007-uuid-generation/quickstart.md
- [x] T031 Run `cargo mutants` and address any surviving mutants to ensure test quality

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 completion (T001 must complete before T004; T004 must complete before T002)
- **User Story 1 (Phase 3)**: Depends on Phase 2 completion — BLOCKS all other stories
- **User Story 2 (Phase 4)**: Depends on Phase 3 completion (US1 implements the code that US2 validates)
- **User Story 3 (Phase 5)**: Depends on Phase 3 completion (US1 implements the code that US3 validates)
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Depends on Foundational (Phase 2) — implements all core functions
- **User Story 2 (P1)**: Depends on US1 completion — validates whitespace resilience of existing implementation
- **User Story 3 (P1)**: Depends on US1 completion — validates content sensitivity of existing implementation
- **US2 and US3 are independent of each other** and can run in parallel after US1 completes

### Deferred: Pipeline Integration

Pipeline integration (invoking `assign_stable_ids` from `src/cli/convert.rs` after requirement atomization) is **deferred until WI-6 is complete**. The `assign_stable_ids` function is implemented and tested in this WI, but the call site in the conversion pipeline depends on WI-5/WI-6 providing the populated `PolicyDocument` tree.

### Deferred: Crate-First Architecture (Constitution I)

Constitution Principle I (NON-NEGOTIABLE) requires every feature to begin as a standalone crate within a Cargo workspace. The current project is a single binary crate (`forge`), not a workspace. This is a **project-wide structural issue** affecting all features (WI-1 through WI-7), not specific to UUID generation. Migrating to `crates/forge-uuid/` within a workspace is deferred to a dedicated migration work item. Until then, `src/uuid.rs` as a module follows the existing project convention.

### Within Each User Story

- TDD tests MUST be written and FAIL before implementation (US1)
- US2 and US3 are test-only phases (implementation already exists from US1)
- Within US1: normalize_for_hashing before generate_stable_id before assign_stable_ids

### Parallel Opportunities

- **Phase 1**: T001/T032 (sequential: add dep then audit) and T003 can run in parallel (different concerns)
- **Phase 3 (US1)**: Tests T005, T006, T007 can be written in parallel; T011, T012 can be written in parallel
- **Phase 4 (US2)**: All tests T015–T020 can run in parallel (read-only, same file but independent test functions)
- **Phase 5 (US3)**: All tests T022–T024 can run in parallel
- **Phase 6**: T026, T027, T028 can run in parallel
- **After US1**: US2 and US3 phases can execute in parallel

---

## Parallel Example: User Story 2

```bash
# Launch all US2 tests in parallel (all independent, same file):
Task: "Write test_whitespace_resilience_extra_spaces in src/uuid.rs"
Task: "Write test_whitespace_resilience_trailing_newline in src/uuid.rs"
Task: "Write test_whitespace_resilience_mixed_tabs_newlines in src/uuid.rs"
Task: "Write test_edge_case_empty_text in src/uuid.rs"
Task: "Write test_edge_case_whitespace_only in src/uuid.rs"
Task: "Write test_edge_case_unicode_whitespace in src/uuid.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T003)
2. Complete Phase 2: Foundational (T004)
3. Complete Phase 3: User Story 1 (T005–T014)
4. **STOP and VALIDATE**: `cargo test uuid` — all determinism and coverage tests pass
5. This delivers the core value: deterministic UUID generation for all requirements

### Incremental Delivery

1. Setup + Foundational → Module ready
2. User Story 1 → Core implementation + determinism verified → **MVP complete**
3. User Story 2 → Whitespace resilience verified → Confidence in normalization
4. User Story 3 → Change detection verified → Full behavioral guarantee
5. Polish → Debug logging, linting, formatting → Production ready

### Suggested Single-Developer Flow

Since all 3 user stories are P1 and share the same implementation:

1. Phase 1 + Phase 2 (setup)
2. Phase 3 / US1 (TDD + implementation of 3 functions)
3. Phase 4 + Phase 5 / US2 + US3 (additional tests only)
4. Phase 6 (polish)

---

## Traceability

### PRD Requirement → Task Mapping

| PRD Req | Task(s) | Verification |
|---------|---------|-------------|
| M-1 (UUID v5 with FORGE namespace) | T004, T009 | T006, T007 |
| M-2 (normalize text) | T008 | T005, T015–T020 |
| M-3 (populate all stable_ids) | T013 | T011, T012 |
| M-4 (identical text → identical UUID) | T009 | T006 |
| M-5 (substantive change → different UUID) | T009 | T022, T023 |
| S-1 (documented namespace constant) | T004 | Code review |
| S-2 (accepts any &str) | T009 | T006 (uses &str) |
| C-1 (debug logging) | T026 | Manual verification |

### Security Requirement → Task Mapping

| SEC Req | Task(s) | Verification |
|---------|---------|-------------|
| SEC-1 (empty text handling) | T008 | T018 |
| SEC-2 (adversarial whitespace) | T008 | T017, T020 |
| SEC-3 (compile-time constant) | T004 | Code review |
| SEC-4 (pure function) | T008, T009, T013 | Code review + task descriptions require purity |

### Acceptance Criteria → Task Mapping

| AC ID | Task(s) | Test(s) |
|-------|---------|---------|
| AC-1 (determinism) | T009 | T006 |
| AC-2 (whitespace resilience) | T008 | T015, T016, T017 |
| AC-3 (sensitivity) | T009 | T022, T023 |
| AC-4 (all populated) | T013 | T011 |
| AC-5 (UUID v5 format) | T009 | T007 |

### Constitution Principle → Task Mapping

| Principle | Requirement | Task(s) |
|-----------|-------------|---------|
| I. Crate-First | Standalone crate in workspace | Deferred (project-wide migration) |
| III. Contract-First | Rustdoc with examples for all public items | T033 |
| IV. Test-First | TDD: tests before implementation | T005–T007 (fail first), T011–T012 |
| VI. Performance-First | Criterion benchmarks for hot paths | T035 |
| IX. Observability | `tracing` (NOT `log`); `#[instrument]` on public fns | T026 (tracing), T034 (#[instrument]) |
| XI. Dependency Policy | Pre-addition security checks; latest stable versions | T001 (latest via cargo add), T032 (audit) |

---

## Notes

- [P] tasks = different files or independent test functions, no dependencies
- [Story] label maps task to specific user story for traceability
- All tests live in `src/uuid.rs` as `#[cfg(test)] mod tests` (idiomatic Rust unit tests)
- TDD cycle: Write test → verify FAIL → implement → verify PASS
- `model/mod.rs` stub types are minimal — WI-5/WI-6 will expand them later
- FORGE_NAMESPACE_UUID must be generated ONCE during implementation and never changed
- Implementation guardrails from AR: NO UUID v4, NO raw text hashing, NO runtime-configurable namespace, NO over-normalization
