# Tasks: Traceability Model (WI-16)

**Input**: Design documents from `/specs/016-traceability-model/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, quickstart.md, PRD, AR, SEC
**Branch**: `016-traceability-model`

**Tests**: TDD is mandatory per project constitution (Principle IV). Tests are written first (RED), then implementation (GREEN).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Register the trace module and create the file skeleton

- [x] T001 Add `pub mod trace;` declaration to src/model/mod.rs and create empty src/model/trace.rs with module-level doc comment

---

## Phase 2: US3 — Source Location Metadata (Priority: P1) 🎯 MVP

**Goal**: Define SourceLocation, TraceLink, and TraceError data types that form the foundation of the traceability model

**Independent Test**: Construct SourceLocation and TraceLink instances, verify all fields are stored correctly, verify serialization round-trips, verify TraceError displays the correct message

**Traces to**: PRD M-1, M-2, S-1; AR Interface Definitions; SEC SEC-1, SEC-2; spec FR-001, FR-002, FR-012

### Tests for US3

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T002 [US3] Write unit tests for SourceLocation: construction with all fields, Debug/Clone/PartialEq/Eq derives, Serialize/Deserialize round-trip, empty section_title edge case (EC-4) in src/model/trace.rs
- [x] T003 [US3] Write unit tests for TraceLink: construction with all four fields plus embedded SourceLocation, Debug/Clone/Serialize/Deserialize derives in src/model/trace.rs
- [x] T004 [US3] Write unit test for TraceError::DuplicateElement variant: verify Display output matches "Duplicate OSCAL element ID: {element_id} already recorded" in src/model/trace.rs

### Implementation for US3

- [x] T005 [US3] Implement SourceLocation struct with derives (Debug, Clone, PartialEq, Eq, Serialize, Deserialize) and fields: file_path (PathBuf), section_title (String), line_number (usize) in src/model/trace.rs
- [x] T006 [US3] Implement TraceLink struct with derives (Debug, Clone, Serialize, Deserialize) and fields: requirement_stable_id (String), oscal_json_path (String), oscal_element_id (String), source_location (SourceLocation) in src/model/trace.rs
- [x] T007 [US3] Implement TraceError enum with #[derive(Debug, thiserror::Error)] and DuplicateElement { element_id: String } variant in src/model/trace.rs

**Checkpoint**: SourceLocation, TraceLink, and TraceError compile; all T002–T004 tests pass (`cargo test trace`)

---

## Phase 3: US1 + US2 — Bidirectional Lookup (Priority: P1)

**Goal**: Implement TraceLinkCollection with dual-store architecture: `links` Vec for insertion-order iteration + `by_requirement` HashMap<String, Vec<TraceLink>> for O(1) grouped forward lookup + `by_oscal_element` HashMap<String, usize> for O(1) reverse lookup

**Independent Test**: Record trace links, verify forward lookup returns all links for a requirement, verify reverse lookup returns the correct single link, verify duplicate detection, verify graceful empty/None returns for missing lookups

**Traces to**: PRD M-3, M-4, M-5, S-2, S-3; AR Option 1 algorithm; SEC SEC-3, SEC-4, SEC-5; spec FR-003 through FR-011

### Tests for US1 + US2

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T008 [US1] [US2] Write unit tests for TraceLinkCollection::new() returns empty, len() == 0, is_empty() == true in src/model/trace.rs
- [x] T009 [US1] [US2] Write unit tests for record(): happy path appends link and updates both indexes, duplicate oscal_element_id returns Err(TraceError::DuplicateElement) (EC-1, SEC-3) in src/model/trace.rs
- [x] T010 [US2] Write unit tests for by_requirement(): returns matching TraceLinks for known stable_id, returns empty slice for unknown stable_id (EC-2, SEC-4), returns multiple links when one requirement maps to multiple OSCAL elements in src/model/trace.rs
- [x] T011 [US1] Write unit tests for by_oscal_element(): returns Some(&TraceLink) for known element_id, returns None for unknown element_id (EC-3, SEC-5) in src/model/trace.rs
- [x] T012 [US1] [US2] Write unit tests for iter(): returns all links in insertion order (S-2), empty collection yields empty iterator in src/model/trace.rs

### Implementation for US1 + US2

- [x] T013 [US1] [US2] Implement TraceLinkCollection struct with private fields (links: Vec<TraceLink> for insertion-order iteration, by_requirement: HashMap<String, Vec<TraceLink>> for grouped forward lookup, by_oscal_element: HashMap<String, usize> for reverse lookup into links), derive Debug and Default, implement new() via Default in src/model/trace.rs
- [x] T014 [US1] [US2] Implement record(&mut self, link: TraceLink) -> Result<(), TraceError>: check duplicate in by_oscal_element, clone link into by_requirement grouped Vec, append original to links Vec, insert index into by_oscal_element reverse index. Consider adding #[instrument(skip(self))] for observability (Constitution IX). In src/model/trace.rs
- [x] T015 [US2] Implement by_requirement(&self, stable_id: &str) -> &[TraceLink]: return &self.by_requirement[stable_id] as contiguous slice from grouped Vec, or empty slice &[] if not found in src/model/trace.rs
- [x] T016 [US1] Implement by_oscal_element(&self, element_id: &str) -> Option<&TraceLink>: look up index in reverse HashMap, return Some(&links[index]) or None in src/model/trace.rs
- [x] T017 [US1] [US2] Implement iter() -> impl Iterator<Item = &TraceLink>, len() -> usize, is_empty() -> bool convenience methods in src/model/trace.rs

**Checkpoint**: All T008–T012 tests pass; TraceLinkCollection provides O(1) bidirectional lookup (`cargo test trace`)

---

## Phase 4: US4 — Catalog Generation Trace Capture (Priority: P1)

**Goal**: Instrument the Catalog builder to automatically record a TraceLink for every generated control, linking it to the source requirement's file path, section title, and line number

**Independent Test**: Run build_catalog() with a test PolicyDocument containing multiple sections and requirements, verify the TraceLinkCollection contains one TraceLink per control with correct source locations and oscal_json_paths

**Traces to**: PRD M-6, M-8; AR Integration Point (Catalog Builder); spec FR-006, FR-008; AC-4, AC-6

### Tests for US4

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T018 [P] [US4] Write integration test: build_catalog() with Some(&mut TraceLinkCollection), verify one TraceLink per control, verify oscal_json_path format "catalog.groups[{g}].controls[{c}]", verify source_location fields match PolicyRequirement.source_line / PolicySection.title / DocumentMetadata.source_path in tests/trace_integration.rs

### Implementation for US4

- [x] T019 [US4] Modify build_catalog() signature in src/oscal/catalog.rs to accept additional parameter trace_links: Option<&mut crate::model::trace::TraceLinkCollection>; add use declaration for trace types
- [x] T020 [US4] Inside the control creation loop (after OscalControl push at ~line 310) in src/oscal/catalog.rs, call trace_links.record(TraceLink { requirement_stable_id: stable_id, oscal_json_path: format!("catalog.groups[{idx}].controls[{req_idx}]"), oscal_element_id: stable_id (control uuid), source_location: SourceLocation { file_path: document.metadata.source_path, section_title: section.title, line_number: req.source_line } }) when trace_links is Some. Note: for catalog controls, oscal_element_id equals requirement_stable_id (both are the stable UUID); the dual-index becomes meaningful when component definition uses distinct implemented-requirement UUIDs.
- [x] T021 [US4] Update run_catalog_pipeline() in src/pipeline.rs: create TraceLinkCollection::new() before Step 8, pass Some(&mut trace_links) to build_catalog(), log trace link count with tracing::info! after build completes
- [x] T022 [US4] Update all existing callers and tests of build_catalog() across the codebase to pass None as the trace_links parameter for backward compatibility

**Checkpoint**: T018 integration test passes; existing catalog pipeline tests still pass (`cargo test`)

---

## Phase 5: US5 — Component Definition Generation Trace Capture (Priority: P1)

**Goal**: Instrument the Component Definition builder to record trace links for implemented-requirement elements (stubbed if WI-15 not yet merged)

**Independent Test**: Run build_component_definition() with Some(&mut TraceLinkCollection), verify trace links are recorded for any implemented-requirements produced (or collection remains empty if WI-15 not merged)

**Traces to**: PRD M-7, M-8; AR Integration Point (Component Builder); spec FR-007, FR-008; AC-5

### Tests for US5

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T023 [P] [US5] Write integration test: build_component_definition() with Some(&mut TraceLinkCollection), verify trace links are recorded per implemented-requirement (if present), or verify empty collection if no implemented-requirements exist yet in tests/trace_integration.rs

### Implementation for US5

- [x] T024 [US5] Modify build_component_definition() signature in src/oscal/component_definition.rs to accept additional parameter trace_links: Option<&mut crate::model::trace::TraceLinkCollection>; add use declaration for trace types
- [x] T025 [US5] Add trace link recording at the implemented-requirement creation point (if WI-15 loop exists) or add a TODO comment marking the future integration point in src/oscal/component_definition.rs
- [x] T026 [US5] Update all existing callers and tests of build_component_definition() across the codebase to pass None as the trace_links parameter for backward compatibility

**Checkpoint**: T023 integration test passes; existing component definition tests still pass (`cargo test`)

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Module re-exports, observability, and final validation

- [x] T027 Add re-exports for SourceLocation, TraceLink, TraceLinkCollection, and TraceError in src/lib.rs (add to existing `pub use model::{...}` line)
- [x] T028 Run `cargo clippy -- -D warnings` and fix any warnings
- [x] T029 Run `cargo fmt --check` and fix any formatting issues
- [x] T030 Run `cargo test` full suite — verify all existing + new tests pass with zero failures
- [x] T031 Validate quickstart.md code examples match the implemented API signatures

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **US3 (Phase 2)**: Depends on Phase 1 (trace module must exist) — BLOCKS Phases 3, 4, 5
- **US1+US2 (Phase 3)**: Depends on Phase 2 (data types must exist) — BLOCKS Phases 4, 5
- **US4 (Phase 4)**: Depends on Phase 3 (collection must be implemented)
- **US5 (Phase 5)**: Depends on Phase 3 (collection must be implemented)
- **US4 and US5 are independent of each other** — can proceed in parallel after Phase 3
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **US3 (Phase 2)**: Foundation — no story dependencies; BLOCKS US1, US2, US4, US5
- **US1+US2 (Phase 3)**: Depends on US3; BLOCKS US4, US5
- **US4 (Phase 4)**: Depends on US1+US2; independent of US5
- **US5 (Phase 5)**: Depends on US1+US2; independent of US4

### Within Each User Story

- Tests MUST be written and FAIL before implementation (TDD: RED → GREEN)
- Struct definitions before collection methods
- Collection methods before integration
- Core implementation before integration with existing code
- Story complete before moving to next priority

### Parallel Opportunities

- **Phase 4 (US4) and Phase 5 (US5)** can run in parallel after Phase 3 completes — they modify different files (catalog.rs vs. component_definition.rs)
- **T018 and T023** (integration test writing) can be parallelized — they write to the same file but test independent scenarios
- Within Phase 2, test tasks T002/T003/T004 test different types and can be written as a batch
- Within Phase 3, test tasks T008–T012 test different methods and can be written as a batch

---

## Parallel Example: Phase 4 + Phase 5

```bash
# After Phase 3 is complete, launch both integration stories in parallel:

# Agent 1 — Catalog Integration (US4):
Task: "Write integration test for catalog trace capture in tests/trace_integration.rs"
Task: "Modify build_catalog() and pipeline to record trace links in src/oscal/catalog.rs, src/pipeline.rs"

# Agent 2 — Component Definition Integration (US5):
Task: "Write integration test for component def trace capture in tests/trace_integration.rs"
Task: "Modify build_component_definition() to accept TraceLinkCollection in src/oscal/component_definition.rs"
```

---

## Implementation Strategy

### MVP First (Phase 1 + Phase 2 + Phase 3)

1. Complete Phase 1: Setup — register trace module
2. Complete Phase 2: US3 — define SourceLocation, TraceLink, TraceError (TDD)
3. Complete Phase 3: US1+US2 — implement TraceLinkCollection with bidirectional lookup (TDD)
4. **STOP and VALIDATE**: `cargo test trace` — all unit tests pass, collection provides O(1) lookups
5. The trace model is usable at this point even without builder integration

### Incremental Delivery

1. Setup + US3 + US1/US2 → Core trace model ready (MVP)
2. Add US4 (Catalog integration) → Trace links captured during catalog generation
3. Add US5 (Component Definition integration) → Trace links captured during component def generation
4. Polish → Re-exports, linting, full validation
5. Each phase adds value without breaking previous phases

### Single Developer Strategy

1. Complete Phases 1–3 sequentially (all in src/model/trace.rs)
2. Complete Phase 4 (Catalog integration — catalog.rs + pipeline.rs)
3. Complete Phase 5 (Component Definition integration — component_definition.rs)
4. Complete Phase 6 (Polish — lib.rs + validation)

---

## Notes

- [P] tasks = different files, no dependencies on incomplete tasks
- [Story] label maps task to specific user story for traceability
- All user stories are P1 priority; execution order follows data dependency (types → collection → integration)
- TDD is mandatory: write tests first (RED), implement (GREEN), refactor (IMPROVE)
- Commit after each phase or logical group
- Stop at any checkpoint to validate story independently
- The component definition integration (US5) may be partially stubbed if WI-15 (Implemented Requirements) is not yet merged — add TODO comment for future integration
