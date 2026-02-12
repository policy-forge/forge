# Tasks: OSCAL Catalog Statement Parts & Prose

**Input**: Design documents from `/specs/010-catalog-statement-parts/`
**Prerequisites**: plan.md (required), spec.md (required), prd.md, ar.md, sec.md, data-model.md, contracts/, research.md, quickstart.md

**Tests**: TDD mandatory per Constitution IV. Tests are written FIRST and must FAIL before implementation.

**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single Rust binary crate**: `src/`, tests inline via `mod tests`

---

## Phase 1: Setup

**Purpose**: No project initialization needed — extending existing single-crate Rust project. All dependencies (serde, serde_json, thiserror, tracing) are already present.

No setup tasks required.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Define new types, update module structure, and extend `OscalControl` so user story implementation can begin.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T001 Create `src/oscal/parts.rs` with `OscalPart` and `OscalProp` struct definitions including serde derives, `skip_serializing_if` annotations per data-model.md, and rustdoc comments on all public items (Constitution I)
- [x] T002 Update `src/oscal/mod.rs` to add `pub mod parts;` declaration and re-export `OscalPart`, `OscalProp`, `build_control_parts`, `build_control_props`, `generate_part_id`
- [x] T003 [P] Update `src/lib.rs` to re-export `OscalPart` and `OscalProp` from the `oscal` module
- [x] T004 [P] Extend `OscalControl` in `src/oscal/catalog.rs` with `parts: Vec<OscalPart>` (always serialized, FR-001) and `props: Vec<OscalProp>` (skip when empty) fields, adding necessary imports from `super::parts`
- [x] T005 Update all existing WI-9 catalog tests in `src/oscal/catalog.rs` to construct `OscalControl` with `parts: vec![]` and `props: vec![]` so the test suite compiles and passes

**Checkpoint**: Foundation ready — all new types defined, module structure updated, existing tests passing. User story implementation can begin.

---

## Phase 3: User Story 1 — Generate Statement Parts with Prose (Priority: P1) 🎯 MVP

**Goal**: Each generated OSCAL control contains a `parts[]` array with a statement part whose `prose` matches the source `PolicyRequirement.text`.

**Independent Test**: Build a catalog from a test `PolicyDocument` and verify every control has a `parts[]` array with at least one entry: `name: "statement"`, `id: "{control-id}_smt"`, `prose` matching source text.

**Traces to**: M-1, M-2, M-3, M-5, M-6, AC-1, AC-2, AC-5, AC-6, SEC-1

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T006 [P] [US1] Write unit tests for `generate_part_id` in `src/oscal/parts.rs`: normal cases (`"POL-AC-001" + "smt"` → `"POL-AC-001_smt"`), various suffixes (`_gdn`, `_obj`), and special characters in control ID (EC-4)
- [x] T007 [P] [US1] Write unit tests for `build_control_parts` (statement-only, `guidance_text: None`) in `src/oscal/parts.rs`: verify returns exactly one part with `name: "statement"`, correct `id` via `_smt` suffix, `prose` matching `requirement.text` exactly (SEC-1, AC-1, AC-2)

### Implementation for User Story 1

- [x] T008 [US1] Implement `generate_part_id` function in `src/oscal/parts.rs` — format `"{control_id}_{suffix}"`
- [x] T009 [US1] Implement `build_control_parts` function in `src/oscal/parts.rs` — generate statement part from `PolicyRequirement.text` with `name: "statement"`, direct text copy (SEC-1), pass `guidance_text` parameter through (guidance logic deferred to US2)
- [x] T010 [US1] Write integration test in `src/oscal/catalog.rs` verifying `build_catalog` produces controls with populated `parts[]` arrays containing statement parts (AC-5, AC-6)
- [x] T011 [US1] Integrate `build_control_parts` into `build_catalog` in `src/oscal/catalog.rs` — call with `(control_id, req, None)` for guidance_text and `props: vec![]` placeholder; verify all existing and new tests pass. *(Implementation note: Combined with T015 and T019 during Phase 2 — final code passes `section.body_text.as_deref()` and `build_control_props(req)` directly.)*

**Checkpoint**: Every control in a built catalog has a statement part with correct ID and prose. US1 independently testable.

---

## Phase 4: User Story 3 — Structured Data Uses Props Not Remarks (Priority: P1) 🎯 MVP

**Goal**: Structured metadata (source line numbers) appears as `prop` elements with `forge:` namespace prefix. No `remarks` field exists on `OscalControl`.

**Independent Test**: Build a catalog from a test `PolicyDocument` with known `source_line` values and verify `props[]` contains `forge:source-line` entries. Verify no `remarks` field in serialized JSON.

**Traces to**: M-4, AC-3, AC-4, SEC-3, SEC-5, EC-6

### Tests for User Story 3

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T012 [P] [US3] Write unit tests for `build_control_props` in `src/oscal/parts.rs`: `source_line > 0` → prop with `name: "forge:source-line"` and `value: line.to_string()` (AC-3); `source_line == 0` → empty vec (EC-6); verify `forge:` prefix (SEC-5)
- [x] T013 [P] [US3] Write test in `src/oscal/catalog.rs` verifying serialized `OscalControl` JSON contains no `remarks` field (AC-4) and that `props` is omitted when empty

### Implementation for User Story 3

- [x] T014 [US3] Implement `build_control_props` function in `src/oscal/parts.rs` — emit `OscalProp { name: "forge:source-line", value }` when `source_line > 0`, return empty vec when `source_line == 0`
- [x] T015 [US3] Replace `props: vec![]` placeholder in `build_catalog` in `src/oscal/catalog.rs` with `props: build_control_props(req)` call; verify all tests pass. *(Implementation note: Combined with T011 during Phase 2.)*

**Checkpoint**: Controls carry structured metadata as props. No remarks misuse possible. US1 + US3 form complete MVP.

---

## Phase 5: User Story 2 — Handle Multi-Part Controls (Priority: P2)

**Goal**: Controls with guidance text (from `PolicySection.body_text`) produce an additional guidance part alongside the statement part.

**Independent Test**: Build a catalog from a `PolicyDocument` where a section has non-empty `body_text`, and verify each control in that section has both a statement part and a guidance part.

**Traces to**: S-1, AC-7, FR-007

### Tests for User Story 2

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T016 [P] [US2] Write unit tests for `build_control_parts` with `guidance_text: Some("guidance text")` in `src/oscal/parts.rs`: verify returns 2 parts — statement part first, then guidance part with `name: "guidance"`, `id: "{control-id}_gdn"`, correct prose (AC-7)
- [x] T017 [P] [US2] Write unit test for `build_control_parts` with `guidance_text: Some("")` (empty string) in `src/oscal/parts.rs`: verify returns only statement part (no guidance for empty text)

### Implementation for User Story 2

- [x] T018 [US2] Add guidance part generation logic to `build_control_parts` in `src/oscal/parts.rs` — when `guidance_text` is `Some(text)` and text is non-empty, append guidance part with `name: "guidance"` and `id: "{control-id}_gdn"`
- [x] T019 [US2] Update `build_catalog` in `src/oscal/catalog.rs` to pass `section.body_text.as_deref()` as `guidance_text` parameter to `build_control_parts` (replacing `None`). *(Implementation note: Combined with T011 during Phase 2 — code was wired with final values from the start.)*
- [x] T020 [US2] Write integration test in `src/oscal/catalog.rs` verifying `build_catalog` with a section containing `body_text` produces controls with both statement and guidance parts

**Checkpoint**: Multi-part controls work end-to-end. Guidance text from `PolicySection.body_text` flows into OSCAL guidance parts.

---

## Phase 6: User Story 4 — Preserve Multi-Paragraph Prose (Priority: P2)

**Goal**: Multi-paragraph requirement text retains paragraph structure in the OSCAL prose field.

**Independent Test**: Build a control from a `PolicyRequirement` with `\n\n`-separated paragraphs and verify the prose field preserves the paragraph breaks.

**Traces to**: S-3, FR-009

### Tests for User Story 4

- [x] T021 [US4] Write unit test for multi-paragraph prose preservation in `src/oscal/parts.rs` — `PolicyRequirement.text` with `\n\n` paragraph breaks preserved verbatim in statement part `prose` (direct copy per SEC-1)

### Implementation for User Story 4

- [x] T022 [US4] Verify `build_control_parts` preserves multi-paragraph text via direct copy in `src/oscal/parts.rs` — no transformation needed per SEC-1; add code comment documenting this guarantee

**Checkpoint**: Multi-paragraph prose confirmed preserved. All P2 stories complete.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Edge cases, observability, validation, and final quality checks.

- [x] T023 [P] Write edge case tests in `src/oscal/parts.rs`: EC-1 (empty requirement text → empty prose + tracing::warn), EC-2 (whitespace-only requirement text → trimmed empty prose + tracing::warn), EC-3 (Markdown formatting bold/links/code preserved in prose)
- [x] T024 [P] Write JSON serialization validation test in `src/oscal/catalog.rs` — build full catalog, serialize to JSON, parse back, verify `parts` array structure matches OSCAL v1.2.0 shape (M-6, AC-6) including field names `id`, `name`, `prose`
- [x] T025 Add `tracing::warn` in `build_control_parts` for empty requirement text (EC-1, SEC-2) and `tracing::debug` for part count per control (Constitution IX) in `src/oscal/parts.rs`
- [x] T026 Run `cargo clippy -- -D warnings`, `cargo fmt --check`, `cargo test --doc`, and `cargo doc --no-deps` to verify zero warnings, correct formatting, doc test compilation, and rustdoc completeness
- [x] T027 Run quickstart.md code examples as validation — verify examples from `specs/010-catalog-statement-parts/quickstart.md` produce expected output
- [x] T028 Verify SEC-4 compliance via code review: confirm `build_control_parts`, `build_control_props`, and `generate_part_id` in `src/oscal/parts.rs` are pure functions with no I/O, no file access, no network calls, and no side effects beyond tracing

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: N/A — no setup tasks needed
- **Foundational (Phase 2)**: No external dependencies — can start immediately
- **User Stories (Phase 3+)**: All depend on Foundational phase (Phase 2) completion
  - US1 (Phase 3) and US3 (Phase 4) are both P1/MVP — execute sequentially (US3 depends on US1 integration)
  - US2 (Phase 5) depends on US1 completion (extends build_control_parts)
  - US4 (Phase 6) depends on US1 completion (tests prose handling)
  - US2 and US4 can proceed in parallel after US1
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

- **US1 (P1)**: Can start after Foundational (Phase 2) — no dependencies on other stories
- **US3 (P1)**: Can start after US1 (Phase 3) — replaces props placeholder in build_catalog
- **US2 (P2)**: Can start after US1 (Phase 3) — extends build_control_parts with guidance logic
- **US4 (P2)**: Can start after US1 (Phase 3) — verifies prose preservation behavior

### Within Each User Story

- Tests MUST be written and FAIL before implementation (TDD, Constitution IV)
- Unit tests before integration tests
- Implementation before integration into build_catalog
- Story complete before moving to next priority

### Task-Level Dependencies

```
T001 → T002 → T003 (lib.rs re-exports)
              → T004 (OscalControl extension) → T005 (fix existing tests)

T005 → T006, T007 (US1 tests, parallel)
T006 → T008 (generate_part_id impl)
T007 → T009 (build_control_parts impl)
T009 → T010 (integration test) → T011 (integrate into build_catalog)

T011 → T012, T013 (US3 tests, parallel)
T012 → T014 (build_control_props impl) → T015 (integrate props into build_catalog)

T011 → T016, T017 (US2 tests, parallel)
T016, T017 → T018 (guidance logic) → T019 (pass body_text) → T020 (integration test)

T011 → T021 (US4 test) → T022 (verify + document)

T020, T022 → T023, T024 (Polish tests, parallel)
           → T025 (observability)
           → T026 (clippy/fmt/doc-tests)
           → T027 (quickstart validation)
           → T028 (SEC-4 code review)
```

### Parallel Opportunities

- **Foundational**: T003 ‖ T004 (after T002 — different files)
- **US1 tests**: T006 ‖ T007 (different functions, same file but independent tests)
- **US3 tests**: T012 ‖ T013 (different files)
- **US2 tests**: T016 ‖ T017 (different test scenarios, same file)
- **After US1**: US3 and (US2 ‖ US4) can proceed — US3 is prioritized as P1/MVP
- **Polish**: T023 ‖ T024 (different files)

---

## Parallel Example: User Story 1

```bash
# Launch US1 tests in parallel (both must FAIL initially):
Task: "Write unit tests for generate_part_id in src/oscal/parts.rs"
Task: "Write unit tests for build_control_parts (statement-only) in src/oscal/parts.rs"

# Then implement sequentially:
Task: "Implement generate_part_id in src/oscal/parts.rs"
Task: "Implement build_control_parts in src/oscal/parts.rs"
Task: "Write integration test in src/oscal/catalog.rs"
Task: "Integrate build_control_parts into build_catalog in src/oscal/catalog.rs"
```

---

## Implementation Strategy

### MVP First (US1 + US3 — Both P1)

1. Complete Phase 2: Foundational (types, module structure, OscalControl extension)
2. Complete Phase 3: US1 — Statement parts with prose (core deliverable)
3. Complete Phase 4: US3 — Props for structured metadata (no remarks)
4. **STOP and VALIDATE**: Every control has statement parts and props. JSON matches OSCAL v1.2.0 shape.
5. MVP complete — controls are content-bearing OSCAL controls

### Incremental Delivery

1. Foundational → Types defined, module wired, existing tests pass
2. US1 → Statement parts populate controls → Test independently → **MVP core**
3. US3 → Props replace remarks for metadata → Test independently → **MVP complete**
4. US2 → Guidance parts from body_text → Test independently → **Enhanced output**
5. US4 → Multi-paragraph prose verified → Test independently → **Full fidelity**
6. Polish → Edge cases, observability, formatting → **Production ready**

### Single Developer Strategy

Execute phases sequentially: Foundational → US1 → US3 → US2 → US4 → Polish. Each phase builds on the previous. Stop at any checkpoint to validate independently.

---

## Notes

- [P] tasks = different files or independent test functions, no dependencies
- [Story] label maps task to specific user story for traceability
- TDD mandatory: write test → see it fail → implement → see it pass → refactor
- Commit after each task or logical group
- SEC-1: Statement prose is ALWAYS a direct copy of `PolicyRequirement.text` — no transformation
- SEC-3: Structured data as `OscalProp`, never in `remarks`
- SEC-4: All functions are pure — no I/O, no side effects (beyond tracing)
- SEC-5: `forge:` prefix for FORGE-specific prop names
