# Tasks: Structural Extraction — Headings

**Input**: Design documents from `/specs/003-structural-extraction-headings/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/extract_sections.rs
**AR**: docs/AR/003-ar-structural-extraction-headings.md (implementation guardrails)
**SEC**: docs/SEC/003-sec-structural-extraction-headings.md (SEC-1 through SEC-4)

**Tests**: Included — Constitution Principle IV (TDD) is NON-NEGOTIABLE. Tests MUST be written before implementation.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Add pulldown-cmark dependency and verify the project builds cleanly.

- [x] T001 Add `pulldown-cmark = "0.13"` dependency to Cargo.toml and run `cargo build` to verify compilation
- [x] T002 Run `cargo test` to verify all existing tests still pass with the new dependency

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Define the `SectionNode` contract and shared helper functions before any user story implementation (Constitution Principle III: Contract-First).

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T003 Define `SectionNode` struct with rustdoc comments and `#[derive(Debug, Clone, PartialEq)]` in `src/parse/mod.rs` per the contract in `specs/003-structural-extraction-headings/contracts/extract_sections.rs`. Fields: `title: String`, `heading_level: u8`, `source_line: usize`, `body_text: Option<String>`, `children: Vec<SectionNode>`
- [x] T004 Define `extract_sections(content: &str) -> Result<Vec<SectionNode>, ForgeError>` function signature as a stub returning `Ok(vec![])` in `src/parse/mod.rs`, with full rustdoc from the contract
- [x] T005 [P] Implement `fn heading_level_to_u8(level: HeadingLevel) -> u8` helper in `src/parse/mod.rs` that converts pulldown-cmark's `HeadingLevel` enum to `u8` (1-6) using a match expression per research R-3
- [x] T006 [P] Implement `fn build_line_starts(content: &str) -> Vec<usize>` and `fn offset_to_line(offset: usize, line_starts: &[usize]) -> usize` helper functions in `src/parse/mod.rs` per research R-4 (pre-computed line-starts table with binary search via `partition_point`)
- [x] T007 Write unit tests for `offset_to_line` in `src/parse/mod.rs`: verify line 1 at offset 0, correct line for multi-line content, offset at newline boundary, offset at end of content, empty content

**Checkpoint**: SectionNode contract defined, helpers tested and working. Ready for user story implementation.

---

## Phase 3: User Story 1 — Extract Section Hierarchy (Priority: P1) 🎯 MVP

**Goal**: Parse Markdown headings (H1-H6) into a hierarchical tree with correct parent-child relationships, heading titles, levels, and 1-based source line numbers.

**Independent Test**: Parse a Markdown document with nested headings (H1 through H3+) and verify the resulting tree has correct parent-child relationships, heading titles, levels, and source line numbers.

**Traces to**: M-1, M-2, M-3 | AC-1, AC-2, AC-3 | SC-001, SC-002, SC-003

### Tests for User Story 1 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation (Constitution Principle IV)**

- [x] T008 [P] [US1] Write failing test: single H1 heading produces one root `SectionNode` with correct title, `heading_level=1`, and `source_line=1` in `src/parse/mod.rs`
- [x] T009 [P] [US1] Write failing test: H1 followed by H2 produces a tree where H2 is a child of H1 in `src/parse/mod.rs`
- [x] T010 [P] [US1] Write failing test: H1 → H2 → H3 produces a three-level nested tree where H3 is child of H2 which is child of H1 in `src/parse/mod.rs`
- [x] T011 [P] [US1] Write failing test: heading `## Access Control` at a known line number produces `SectionNode { title: "Access Control", heading_level: 2, source_line: <expected> }` in `src/parse/mod.rs` (AC-2)
- [x] T012 [P] [US1] Write failing test: document with 5 headings at various levels produces 5 section nodes in the correct hierarchy in `src/parse/mod.rs` (AC-1)

### Implementation for User Story 1

- [x] T013 [US1] Implement `extract_sections` core algorithm in `src/parse/mod.rs` using the stack-based tree construction from AR-003: iterate `pulldown_cmark::Parser::new(content).into_offset_iter()`, detect `Event::Start(Tag::Heading { level, .. })` events, collect title text until `Event::End(TagEnd::Heading(_))`, compute source line via `offset_to_line`, manage `Vec<(u8, SectionNode)>` explicit stack with pop-to-parent semantics, and drain stack to root list at end. Body text accumulation is deferred to US3 — set `body_text: None` for now. (SEC-3: explicit stack, not recursion)
- [x] T014 [US1] Run `cargo test parse` and verify all US1 tests (T008-T012) pass

**Checkpoint**: Basic heading extraction works. Single headings, nested headings, and multi-level hierarchies produce correct trees with accurate titles, levels, and line numbers.

---

## Phase 4: User Story 2 — Handle Irregular Heading Levels (Priority: P2)

**Goal**: Handle documents with skipped heading levels, multiple H1s, headings starting at deep levels, empty headings, and documents with no headings — all without panicking or losing sections.

**Independent Test**: Parse documents with irregular heading patterns and verify reasonable trees are produced with no lost sections and no panics.

**Traces to**: M-4 | AC-4 | EC-1, EC-2, EC-3, EC-4, EC-6 | SC-004 | SEC-1, SEC-2

### Tests for User Story 2 ⚠️

> **NOTE: Write these tests FIRST. Some may already pass if the stack algorithm handles them naturally — that's OK. Verify each test's behavior explicitly.**

- [x] T015 [P] [US2] Write test: H1 followed directly by H3 (skipping H2) places H3 as child of H1 in `src/parse/mod.rs` (AC-4, SEC-1)
- [x] T016 [P] [US2] Write test: document with multiple H1 headings produces separate top-level sections in `src/parse/mod.rs` (EC-4)
- [x] T017 [P] [US2] Write test: document starting with H3 (no preceding H1 or H2) makes H3 a top-level section in `src/parse/mod.rs` (EC-2, SEC-1)
- [x] T018 [P] [US2] Write test: document with no headings returns empty `Vec<SectionNode>` in `src/parse/mod.rs` (EC-1, SEC-2)
- [x] T019 [P] [US2] Write test: empty heading text (bare `##` with no title) produces section with empty string title in `src/parse/mod.rs` (EC-3)
- [x] T020 [P] [US2] Write test: document with only one heading produces a single-node tree in `src/parse/mod.rs` (EC-6)
- [x] T021 [P] [US2] Write test: heading level sequence H1 → H3 → H2 correctly pops H3 back and makes H2 a sibling of (not child of) H3 in `src/parse/mod.rs`

### Implementation for User Story 2

- [x] T022 [US2] Review US2 test results — if any tests fail, fix the stack pop-to-parent logic in `extract_sections` in `src/parse/mod.rs` to correctly handle all irregular nesting patterns
- [x] T023 [US2] Run `cargo test parse` and verify all US1 + US2 tests pass (no regressions)

**Checkpoint**: Irregular heading patterns handled. All edge cases (EC-1 through EC-4, EC-6) pass. SEC-1 and SEC-2 verified.

---

## Phase 5: User Story 3 — Capture Section Body Content (Priority: P3)

**Goal**: Capture the text content between headings and associate it with the appropriate section's `body_text` field. Text before the first heading is discarded per assumption A-4.

**Independent Test**: Parse a document where headings have body text between them and verify each section node contains the correct body content.

**Traces to**: S-1 | EC-5 | A-3, A-4

### Tests for User Story 3 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before extending the body text accumulation logic.**

- [x] T024 [P] [US3] Write failing test: heading `## Scope` followed by two paragraphs of text captures both paragraphs in `body_text` in `src/parse/mod.rs` (S-1 scenario 1)
- [x] T025 [P] [US3] Write failing test: heading with no content before the next heading has `body_text: None` in `src/parse/mod.rs` (S-1 scenario 2)
- [x] T026 [P] [US3] Write failing test: text before the first heading is discarded (not associated with any section) in `src/parse/mod.rs` (EC-5, A-4)
- [x] T027 [P] [US3] Write failing test: inline code in body text is preserved (e.g., `` `foo` `` appears in body_text) in `src/parse/mod.rs`

### Implementation for User Story 3

- [x] T028 [US3] Extend `extract_sections` in `src/parse/mod.rs` to accumulate body text: track `in_heading` boolean state, and between headings accumulate `Event::Text(s)` → append `s`, `Event::Code(s)` → append `` `s` `` (with backticks per R-6), `Event::SoftBreak` → append `\n`, and `Event::HardBreak` → append `\n` events into the current stack top's `body_text` field. Trim trailing whitespace on finalized body_text. Convert empty accumulated body to `None`.
- [x] T029 [US3] Run `cargo test parse` and verify all US1 + US2 + US3 tests pass (no regressions)

**Checkpoint**: Body text capture works. Each section contains the text content between its heading and the next heading. Pre-heading text is discarded.

---

## Phase 6: User Story 4 — Verify Section Tree via Debug Output (Priority: P4)

**Goal**: Ensure the section tree can be inspected as human-readable debug output for development-time verification.

**Independent Test**: Run section extraction on a sample document and verify a human-readable representation including nesting, titles, levels, and line numbers is produced.

**Traces to**: S-2

- [x] T030 [US4] Write test verifying `SectionNode`'s `Debug` output includes title, heading_level, source_line, body_text, and children fields for a nested document in `src/parse/mod.rs`
- [x] T031 [US4] Verify `format!("{:#?}", sections)` produces readable indented output for a multi-level tree in `src/parse/mod.rs` (this should already work via `#[derive(Debug)]` — confirm with test)

**Checkpoint**: Debug output is human-readable and shows all SectionNode fields including nesting structure.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Pipeline integration, comprehensive validation, and quality gates.

- [x] T032 Wire `extract_sections` into the convert pipeline in `src/cli/convert.rs`: reconstruct content string from `IngestedDocument.lines` by joining line texts with `"\n"` (note: reconstruction is sufficient for pulldown-cmark heading detection; byte-level fidelity with the original file is not required), call `parse::extract_sections(&content)`, and include the section tree in the output (print section count or debug representation alongside the ingested document JSON)
- [x] T033 Write integration test in `src/parse/mod.rs` that parses all 25 `example_data/*.md` policy documents and verifies each produces a non-empty `Vec<SectionNode>` with no errors (SC-005)
- [x] T034 Run `cargo fmt --check` and fix any formatting issues
- [x] T035 Run `cargo clippy -- -D warnings` and fix any warnings
- [x] T036 Run `cargo doc --no-deps` and verify documentation builds without warnings
- [x] T037 Run full `cargo test` to verify all unit and integration tests pass across the workspace

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup (T001, T002) — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Foundational (T003-T007) — this is the MVP
- **US2 (Phase 4)**: Depends on US1 completion (T013 implementation) — tests verify edge cases of same algorithm
- **US3 (Phase 5)**: Depends on US1 completion (T013 implementation) — extends algorithm with body text
- **US4 (Phase 6)**: Depends on US1 completion — tests Debug derive (already present)
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

```text
Phase 1: Setup
    │
    ▼
Phase 2: Foundational
    │
    ▼
Phase 3: US1 (P1) ← MVP
    │
    ├──► Phase 4: US2 (P2)  ─┐
    ├──► Phase 5: US3 (P3)  ─┤  (US2, US3, US4 can run in parallel after US1)
    └──► Phase 6: US4 (P4)  ─┘
                              │
                              ▼
                    Phase 7: Polish
```

### Within Each User Story

1. Tests MUST be written and FAIL before implementation
2. Implementation makes tests pass
3. All previous story tests must still pass (no regressions)

### Parallel Opportunities

- **Phase 2**: T005 and T006 can run in parallel (different helper functions)
- **Phase 3 tests**: T008-T012 can all be written in parallel (same file but independent tests)
- **Phase 4 tests**: T015-T021 can all be written in parallel
- **Phase 5 tests**: T024-T027 can all be written in parallel
- **After US1**: US2, US3, and US4 phases can proceed in parallel (US2 and US3 both extend the same function, so coordinate if parallel)

---

## Parallel Example: User Story 1

```bash
# Write all US1 tests in parallel (all in src/parse/mod.rs #[cfg(test)]):
Task: "T008 [P] [US1] Write failing test: single H1 heading"
Task: "T009 [P] [US1] Write failing test: H1+H2 nesting"
Task: "T010 [P] [US1] Write failing test: H1+H2+H3 deeper nesting"
Task: "T011 [P] [US1] Write failing test: heading title and line number"
Task: "T012 [P] [US1] Write failing test: 5 headings at various levels"

# Then implement (sequential — single function):
Task: "T013 [US1] Implement extract_sections core algorithm"
Task: "T014 [US1] Verify all US1 tests pass"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T002)
2. Complete Phase 2: Foundational (T003-T007)
3. Complete Phase 3: User Story 1 (T008-T014)
4. **STOP and VALIDATE**: Basic heading extraction works for well-formed documents
5. This delivers M-1, M-2, M-3 — the core section hierarchy

### Incremental Delivery

1. Setup + Foundational → Contract defined, helpers tested
2. Add US1 → Basic heading extraction → Test independently (MVP!)
3. Add US2 → Irregular heading handling → Test edge cases (M-4, SEC-1, SEC-2)
4. Add US3 → Body text capture → Test body accumulation (S-1)
5. Add US4 → Debug output verification → Confirm readability (S-2)
6. Polish → Pipeline integration, all 25 example docs, quality gates

### AR Implementation Guardrails Checklist

These guardrails from the AR MUST be verified before completion:

- [x] **DO NOT** extract lists, tables, or clause content (WI-4 scope)
- [x] **DO NOT** create `PolicySection` structs (WI-5 scope)
- [x] **DO NOT** use regex for heading detection — pulldown-cmark events only
- [x] **DO NOT** panic on irregular heading levels — handle gracefully
- [x] **MUST** produce correct parent-child relationships for all heading level combinations
- [x] **MUST** preserve source line numbers on every SectionNode
- [x] **MUST** handle documents with no headings (return empty Vec)
- [x] **MUST** handle documents starting with deep headings (e.g., H3 first)
- [x] **MUST** write tests before implementation (TDD)

---

## Notes

- [P] tasks = different files or independent test functions, no dependencies on incomplete tasks
- [Story] label maps task to specific user story for traceability
- Each user story is independently testable after completion
- Verify tests fail before implementing (Constitution Principle IV)
- Commit after each phase completion
- SEC-3 (explicit stack) and SEC-4 (bounded depth) are verified by code review of the implementation
- The `extract_sections` function is the single implementation target — all user stories add capabilities to this one function
