# Tasks: Internal Domain Model

**Input**: Design documents from `/specs/005-domain-model/`
**Prerequisites**: plan.md (required), spec.md (required), prd.md, ar.md, sec.md, data-model.md, contracts/rust-interfaces.md, quickstart.md

**Tests**: TDD mandatory per constitution principle IV and AR implementation guardrails. All tests MUST be written and verified to FAIL before implementation.

**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2)
- Exact file paths included in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add new dependency and create module file structure

- [x] T001 Add `serde_yaml` dependency to Cargo.toml (required for YAML frontmatter parsing per AR selected approach)
- [x] T002 Create `src/model/frontmatter.rs` and `src/model/assemble.rs` module files, declare as submodules in `src/model/mod.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Define core domain model type definitions that MUST exist before any user story work

**CRITICAL**: No user story work can begin until this phase is complete

- [x] T003 Define `PolicyRequirement`, `PolicySection`, `DocumentMetadata`, and `PolicyDocument` structs with `#[derive(Debug, Clone)]` and doc comments in `src/model/mod.rs` — fields per contracts/rust-interfaces.md: PolicyRequirement {stable_id: Option\<String\>, text: String, source_line: usize, nesting_depth: u8}, PolicySection {title: String, heading_level: u8, source_line: usize, body_text: Option\<String\>, children: Vec\<PolicySection\>, requirements: Vec\<PolicyRequirement\>}, DocumentMetadata {title: String, version: String, author: Option\<String\>, date: Option\<String\>, source_path: PathBuf, content_hash: Option\<String\>}, PolicyDocument {id: String, metadata: DocumentMetadata, sections: Vec\<PolicySection\>}
- [x] T004 Write unit tests for struct construction (all fields populated), Debug derive output, Clone derive, and edge cases (empty sections vec, None optional fields, empty requirements vec) in `src/model/mod.rs`

**Checkpoint**: Domain model types defined and tested — user story implementation can now begin

---

## Phase 3: User Story 1 — Build PolicyDocument from Extracted Data (Priority: P1) MVP

**Goal**: Assemble a complete PolicyDocument from ingestion (WI-2), section extraction (WI-3), and clause extraction (WI-4) outputs, with metadata from YAML frontmatter or fallback values.

**Independent Test**: Construct IngestedDocument/SectionNode/ExtractedContent test inputs, call `assemble_document`, and verify all sections and requirements are present with correct metadata.

**Traces to**: M-1, M-2, M-3, M-4, M-5, S-1, S-2, C-1, SEC-1, SEC-2, SEC-3, SEC-5, SEC-6, SEC-7, AC-1 through AC-5, EC-1 through EC-4

### Tests for User Story 1 (TDD — Write FIRST, verify they FAIL)

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T005 [P] [US1] Write failing tests for `parse_frontmatter` in `src/model/frontmatter.rs`: (1) valid YAML with title+version+author+date returns Some(FrontmatterData) with all fields, (2) valid YAML with only title+version returns Some with author=None date=None, (3) content with no frontmatter returns None, (4) malformed YAML returns None (fault-tolerant per SEC-1, SEC-6), (5) empty string returns None (SEC-3)
- [x] T006 [P] [US1] Write failing tests for `map_sections` in `src/model/assemble.rs`: (1) flat 3 sections produce 3 PolicySections with correct titles/levels, (2) nested H1->H2->H3 produces correct parent-child tree, (3) section with no matching list items has empty requirements vec (EC-2), (4) list items are associated with correct parent section by line range, (5) empty sections vec returns empty vec
- [x] T007 [US1] Write failing tests for `assemble_document` in `src/model/assemble.rs`: (1) happy path with YAML frontmatter populates title="Security Policy" version="1.0" (AC-2), (2) no frontmatter with H1 "Access Control Policy" falls back to title from H1 and version "0.0.0" (AC-3), (3) no frontmatter and no headings falls back to filename as title and version "0.0.0" (EC-1), (4) structurally empty document returns Ok with empty sections (EC-3, SEC-2), (5) malformed YAML frontmatter returns Ok with fallback metadata and warning to stderr (EC-4, SEC-1), (6) 3 sections with 10 total requirements assembled correctly (AC-1, AC-5)

### Implementation for User Story 1

- [x] T008 [P] [US1] Implement `FrontmatterData` serde deserialize struct and `parse_frontmatter` function in `src/model/frontmatter.rs` — detect "---\n" delimiters, extract YAML between them, deserialize with `serde_yaml::from_str` (handle errors gracefully, NO `unwrap()` per SEC-6), return `Option<FrontmatterData>` with None on failure
- [x] T009 [P] [US1] Implement `map_sections` function in `src/model/assemble.rs` — recursively convert `Vec<SectionNode>` to `Vec<PolicySection>`, associate `ExtractedListItem`s with sections by line-range heuristic (requirement belongs to section if source_line falls within [section.source_line, next_sibling.source_line)), convert matching items to `PolicyRequirement` with stable_id=None (SEC-5: no silent drops)
- [x] T010 [US1] Implement `assemble_document` function in `src/model/assemble.rs` — reconstruct content from `IngestedDocument.lines`, call `parse_frontmatter`, resolve metadata with fallback chain (frontmatter.title -> first H1 heading -> filename stem; frontmatter.version -> "0.0.0"; author/date from frontmatter only), map `IngestedDocument.fingerprint` to `DocumentMetadata.content_hash`, call `map_sections`, construct `PolicyDocument` with id from filename stem, emit `eprintln!` warning for malformed YAML (SEC-1), return `Ok(PolicyDocument)` (M-5)
- [x] T011 [US1] Re-export `assemble_document` as public API from `src/model/mod.rs` and verify `cargo test` passes for all US1 tests

**Checkpoint**: User Story 1 fully functional — PolicyDocument can be assembled from extraction outputs

---

## Phase 4: User Story 2 — Preserve Source Traceability in Domain Model (Priority: P1)

**Goal**: Verify every domain model element preserves its source line number for downstream traceability compliance.

**Independent Test**: Construct a PolicyDocument from test inputs with known line numbers and verify each PolicySection and PolicyRequirement has exact source_line values matching extraction inputs.

**Traces to**: M-6, SEC-4, SEC-5, AC-4

### Tests for User Story 2

> **NOTE: These tests verify traceability guarantees from the assembly implementation**

- [x] T012 [P] [US2] Write traceability tests verifying `PolicySection.source_line` matches `SectionNode.source_line` for sections at known line numbers (10, 20, 35) after assembly in `src/model/assemble.rs` (AC-4, SEC-4)
- [x] T013 [P] [US2] Write traceability tests verifying `PolicyRequirement.source_line` matches `ExtractedListItem.source_line` for requirements at known line numbers (15, 22, 30) after assembly in `src/model/assemble.rs` (AC-4, SEC-4)
- [x] T014 [US2] Write data completeness test verifying `assemble_document` preserves ALL sections and requirements without silent drops — input section count equals output section count, input list item count equals total output requirement count across all sections in `src/model/assemble.rs` (SEC-5)

**Checkpoint**: User Stories 1 AND 2 verified — full assembly with traceability guarantees

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: CLI integration, integration testing, and quality validation

- [x] T015 Wire `assemble_document` into CLI convert pipeline in `src/cli/convert.rs` — after ingestion and section extraction, add clause extraction via `extract_clauses`, then call `assemble_document`, update output to include PolicyDocument with human-readable summary (section count, requirement count) for CLI output per S-1
- [x] T016 Write integration test for full pipeline in `tests/pipeline_test.rs` — read a real Markdown fixture file, call `ingest_file` -> `extract_sections` -> `extract_clauses` -> `assemble_document`, verify PolicyDocument has non-empty sections, requirements with valid source_lines, and correct metadata
- [x] T017 [P] Run `cargo clippy -- -D warnings` and fix any warnings across all new and modified files
- [x] T018 [P] Run `cargo fmt --check` and ensure all new and modified files pass formatting
- [x] T019 Validate all security requirements (SEC-1 through SEC-7) are covered by existing tests — cross-reference each SEC requirement with its test location

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational phase completion
- **User Story 2 (Phase 4)**: Depends on User Story 1 completion (traceability verification requires working assembly)
- **Polish (Phase 5)**: Depends on User Stories 1 and 2 being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) — No external story dependencies
- **User Story 2 (P1)**: Depends on User Story 1 assembly implementation — traceability tests verify assembly behavior

### Within Each User Story

- Tests MUST be written and FAIL before implementation (TDD mandatory)
- `FrontmatterData` struct before `parse_frontmatter` function
- `parse_frontmatter` before `assemble_document` (internal dependency)
- `map_sections` before `assemble_document` (internal dependency)
- `assemble_document` before re-export
- Story complete before moving to next phase

### Parallel Opportunities

- **Phase 3 Tests**: T005 and T006 can run in parallel (different files: frontmatter.rs vs assemble.rs)
- **Phase 3 Implementation**: T008 and T009 can run in parallel (different files: frontmatter.rs vs assemble.rs)
- **Phase 4 Tests**: T012 and T013 can run in parallel (independent test functions)
- **Phase 5 Quality**: T017 and T018 can run in parallel (independent quality checks)

---

## Parallel Example: User Story 1

```bash
# Launch test writing in parallel (different files):
Task: "Write failing tests for parse_frontmatter in src/model/frontmatter.rs" [T005]
Task: "Write failing tests for map_sections in src/model/assemble.rs" [T006]

# Launch implementation in parallel (different files):
Task: "Implement parse_frontmatter in src/model/frontmatter.rs" [T008]
Task: "Implement map_sections in src/model/assemble.rs" [T009]
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (add serde_yaml, create files)
2. Complete Phase 2: Foundational (define domain model structs, write construction tests)
3. Complete Phase 3: User Story 1 (TDD: write failing tests -> implement -> verify tests pass)
4. **STOP and VALIDATE**: Run `cargo test` — all US1 tests should pass
5. Proceed to Phase 4: User Story 2 (traceability verification)

### Incremental Delivery

1. Setup + Foundational -> Domain model types ready
2. Add User Story 1 -> PolicyDocument can be assembled -> Test independently -> Validate
3. Add User Story 2 -> Traceability guarantees verified -> Test independently -> Validate
4. Polish -> CLI integration, full pipeline integration test, quality checks
5. Each story adds value without breaking previous stories

### TDD Cycle per Component

For each component (frontmatter, section mapping, assembly):
1. Write tests that describe the desired behavior
2. Run tests — verify they FAIL (confirms tests are meaningful)
3. Implement the minimal code to make tests pass
4. Run tests — verify they PASS
5. Refactor if needed (tests must still pass)

---

## Requirement Traceability

| Requirement | Task(s) | Verification |
|-------------|---------|--------------|
| M-1: PolicyDocument struct | T003, T004 | Unit test |
| M-2: PolicySection struct | T003, T004 | Unit test |
| M-3: PolicyRequirement struct (with stable_id placeholder) | T003, T004 | Unit test |
| M-4: DocumentMetadata + frontmatter with fallback | T003, T005, T007, T008, T010 | Unit test |
| M-5: assemble_document wiring WI-2/3/4 | T007, T010 | Unit test |
| M-6: source_line preservation | T003, T012, T013 | Unit test |
| S-1: Debug derive on PolicyDocument | T003, T004 | Unit test |
| S-2: Optional author/date in DocumentMetadata | T003, T005 | Unit test |
| C-1: content_hash from ingestion | T003 | Unit test |
| SEC-1: Fault-tolerant YAML parsing | T005, T008 | Unit test |
| SEC-2: Empty document -> valid PolicyDocument | T004, T007 | Unit test |
| SEC-3: Sensible defaults (filename, "0.0.0") | T005, T007, T010 | Unit test |
| SEC-4: source_line preservation from extraction | T012, T013 | Unit test |
| SEC-5: No silent drops of sections/requirements | T014 | Unit test |
| SEC-6: No unwrap() on serde_yaml | T008 | Code review |
| SEC-7: Option\<T\> for later-WI fields | T003 | Code review |
| AC-1: 3 sections, 10 requirements assembled | T007 | Unit test |
| AC-2: Frontmatter title/version populated | T007 | Unit test |
| AC-3: H1 fallback for title, "0.0.0" default | T007 | Unit test |
| AC-4: source_line accuracy (15, 22, 30) | T012, T013 | Unit test |
| AC-5: Complete assembly from WI-2/3/4 | T007 | Unit test |
| EC-1: No frontmatter + no headings -> filename title | T007 | Unit test |
| EC-2: Section with empty requirements | T006 | Unit test |
| EC-3: Empty document -> empty PolicyDocument | T007 | Unit test |
| EC-4: Malformed YAML -> warning + fallback | T005, T007 | Unit test |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- TDD is MANDATORY: write tests FIRST, verify they FAIL, then implement
- All domain model structs MUST derive `Debug` and `Clone` (AR guardrail)
- MUST NOT include OSCAL-specific fields (no `group_id`, `control_id`, `oscal_version`) — AR guardrail
- MUST NOT generate stable UUIDs — leave `stable_id` as None; WI-7 populates this — AR guardrail
- MUST NOT use `unwrap()` on `serde_yaml` deserialization results — SEC-6
- MUST use `Option<T>` for fields populated by later WIs (stable_id, content_hash) — SEC-7
- Error handling: malformed YAML -> `eprintln!` warning + fallback values, return `Ok(PolicyDocument)` — SEC-1
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
