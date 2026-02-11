# Tasks: Structural Extraction — Clauses & Tables

**Input**: Design documents from `/specs/004-structural-extraction-clauses/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/api.rs
**Architecture**: AR-004 Option 1 — Event-Based Pattern Matching with Depth Counter
**Security**: SEC-004 requirements (SEC-1 through SEC-7)

**Tests**: TDD is MANDATORY per Constitution Principle IV. Tests are written before implementation.

**Organization**: Tasks grouped by user story. US3 (section association) is **deferred to WI-5** per AR-004 implementation guardrails.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US4)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Create module structure, define types from contract, promote shared utilities

- [X] T001 Promote `build_line_starts()` and `offset_to_line()` from `fn` to `pub(crate) fn` in `src/parse/mod.rs`
- [X] T002 Create `src/parse/clauses.rs` with type definitions (`ListType`, `ExtractedListItem`, `ExtractedTable`, `ExtractedParagraph`, `ExtractedContent`) matching `contracts/api.rs` and derive macros (`Debug`, `Clone`, `PartialEq`, `Serialize`; `Copy` + `Eq` for `ListType`)
- [X] T003 Add `pub mod clauses;` declaration and re-export clause types (`ListType`, `ExtractedListItem`, `ExtractedTable`, `ExtractedParagraph`, `ExtractedContent`, `extract_clauses`) from `src/parse/mod.rs`
- [X] T004 Add `extract_clauses` function skeleton in `src/parse/clauses.rs` returning `Result<ExtractedContent, ForgeError>` with `todo!()` body; verify `cargo build` compiles and existing tests pass (`cargo test --lib`)

**Checkpoint**: Module structure compiles, all existing WI-3 tests still pass, types are importable via `forge::parse::*`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Scaffold the parser infrastructure that all user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T005 Write unit test `test_empty_document_returns_empty_content` in `src/parse/clauses.rs` `#[cfg(test)]` module: empty string input produces `ExtractedContent` with all empty vectors *(SEC-1, EC-1)* — verify test FAILS against `todo!()` skeleton from T004
- [X] T006 Write unit test `test_document_with_only_headings_returns_empty_content` in `src/parse/clauses.rs`: document with headings but no lists/tables/paragraphs produces empty `ExtractedContent` *(SEC-1)* — verify test FAILS
- [X] T007 Implement parser setup inside `extract_clauses` in `src/parse/clauses.rs`: create `Options::ENABLE_TABLES` *(SEC-7)*, instantiate `Parser::new_ext(content, options)`, build line-starts table via `build_line_starts(content)`, initialize empty `ExtractedContent`, and return `Ok(result)` — T005 and T006 tests now pass
- [X] T008 Verify `cargo test --lib` passes with all new and existing tests, `cargo clippy -- -D warnings` clean

**Checkpoint**: Foundation ready — `extract_clauses` handles empty/heading-only documents, parser infrastructure in place

---

## Phase 3: User Story 1 — Extract List Items (Priority: P1) 🎯 MVP

**Goal**: Extract ordered and unordered list items with text, source line numbers, nesting depth, and list type

**Independent Test**: Parse a Markdown document with ordered/unordered/nested lists and verify each item has correct text, list type, nesting depth, and source line number

**AR Guardrails**: Use depth counter (not indentation parsing) *(SEC-4, SEC-6)*. Strip inline formatting to plain text *(SEC-3)*. Exclude code blocks and blockquotes within list items *(EC-8)*.

### Tests for User Story 1 (TDD — write FIRST, verify FAIL)

- [ ] T009 [US1] Write failing test `test_ordered_list_three_items` in `src/parse/clauses.rs`: 3-item ordered list produces 3 `ExtractedListItem`s with `ListType::Ordered`, correct text, `nesting_depth: 0`, and correct source line numbers *(AC-1, M-1, M-4)*
- [ ] T010 [US1] Write failing test `test_unordered_list_three_items` in `src/parse/clauses.rs`: 3-item bullet list produces 3 `ExtractedListItem`s with `ListType::Unordered`, correct text, and source line numbers *(AC-2, M-2, M-4)*
- [ ] T011 [US1] Write failing test `test_nested_list_three_levels` in `src/parse/clauses.rs`: nested list (ordered > unordered > ordered) produces items with `nesting_depth` 0, 1, and 2 respectively, with correct `list_type` at each level *(AC-5, M-5)*
- [ ] T012 [US1] Write failing test `test_mixed_list_types` in `src/parse/clauses.rs`: document with both ordered and unordered lists produces items with correct `list_type` for each *(M-1, M-2)*
- [ ] T013 [US1] Write failing test `test_deeply_nested_list_six_levels` in `src/parse/clauses.rs`: 6-level nested list produces items with `nesting_depth` 0 through 5 *(EC-2, SEC-4)*
- [ ] T014 [US1] Write failing test `test_inline_formatting_stripped` in `src/parse/clauses.rs`: list item with `**bold**`, `*italic*`, `[link](url)`, and `` `code` `` produces plain text without markup *(EC-4, SEC-3)*
- [ ] T015 [US1] Write failing test `test_multi_paragraph_list_item` in `src/parse/clauses.rs`: list item with continuation paragraph concatenates text; code block within item is excluded *(EC-8)*
- [ ] T016 [US1] Write failing test `test_empty_list_item_excluded` in `src/parse/clauses.rs`: list item with no text content (only whitespace) is not included in results

### Implementation for User Story 1

- [ ] T017 [US1] Implement list extraction in `extract_clauses` in `src/parse/clauses.rs`: `Vec<ListType>` stack for depth tracking, `Start(List)`/`End(List)` push/pop, `Start(Item)`/`End(Item)` accumulation, `nesting_depth = (stack.len() - 1).min(255) as u8` *(SEC-4 saturation)*, `offset_to_line` for source line numbers *(M-4)*
- [ ] T018 [US1] Implement inline text accumulation in `src/parse/clauses.rs`: handle `Text`, `Code`, and `SoftBreak` events within list items; `SoftBreak` → space; ignore formatting tags (`Start(Emphasis)`, `Start(Strong)`, `Start(Link)`, etc.) *(SEC-3)*
- [ ] T019 [US1] Implement `exclude_depth` counter in `src/parse/clauses.rs`: increment on `Start(CodeBlock|BlockQuote)` within items, decrement on end; only accumulate text when `exclude_depth == 0` *(EC-8)*
- [ ] T020 [US1] Verify all US1 tests pass: `cargo test extract_clauses --lib` — all T009-T016 tests green
- [ ] T021 [US1] Run `cargo clippy -- -D warnings` and `cargo fmt --check` — zero warnings, formatting clean

**Checkpoint**: All list extraction works — ordered, unordered, nested, mixed types, inline stripping, multi-paragraph items. US1 independently testable.

---

## Phase 4: User Story 2 — Extract Table Content (Priority: P2)

**Goal**: Extract GFM tables preserving headers, data rows, cell content, and source line number

**Independent Test**: Parse a Markdown document with tables and verify table structure (headers, rows, cells, source line) is preserved

**AR Guardrails**: Use `Options::ENABLE_TABLES` *(SEC-7)*. Empty cells → empty strings *(SEC-2)*.

### Tests for User Story 2 (TDD — write FIRST, verify FAIL)

- [ ] T022 [US2] Write failing test `test_basic_table_three_columns_five_rows` in `src/parse/clauses.rs`: table with 3 headers and 5 data rows produces `ExtractedTable` with correct `headers`, `rows`, and `source_line` *(AC-3, M-3, M-4)*
- [ ] T023 [US2] Write failing test `test_table_source_line` in `src/parse/clauses.rs`: table at a known line position records correct `source_line` *(AC-4, M-4)*
- [ ] T024 [US2] Write failing test `test_table_empty_cells` in `src/parse/clauses.rs`: table with empty cells produces empty strings for those cells *(EC-5, SEC-2)*
- [ ] T025 [US2] Write failing test `test_table_header_only_no_data_rows` in `src/parse/clauses.rs`: header-only table produces `ExtractedTable` with populated `headers` and empty `rows` *(EC-3)*
- [ ] T026 [US2] Write failing test `test_table_inline_formatting_stripped` in `src/parse/clauses.rs`: table cells with bold/italic/code produce plain text *(SEC-3)*
- [ ] T027 [US2] Write failing test `test_multiple_tables` in `src/parse/clauses.rs`: document with 2 tables produces 2 `ExtractedTable`s in document order *(M-3)*

### Implementation for User Story 2

- [ ] T028 [US2] Implement table extraction state machine in `extract_clauses` in `src/parse/clauses.rs`: track `in_table`, `in_header`, `current_row`, `current_cell`, `current_table`; handle `Start(Table)`/`End(Table)`, `Start(TableHead)`/`End(TableHead)`, `Start(TableRow)`/`End(TableRow)`, `Start(TableCell)`/`End(TableCell)` events; accumulate `Text`/`Code` events into cells *(SEC-7)*
- [ ] T029 [US2] Verify all US2 tests pass: `cargo test extract_clauses --lib` — all T022-T027 tests green
- [ ] T030 [US2] Run `cargo clippy -- -D warnings` and `cargo fmt --check` — zero warnings, formatting clean

**Checkpoint**: Table extraction works — headers, rows, empty cells, inline stripping, multiple tables. US1 + US2 independently testable.

---

## Phase 5: User Story 4 — Capture Paragraph Text (Priority: P4)

**Goal**: Extract standalone paragraph text (not inside list items or tables) with source line number

**Independent Test**: Parse a document with paragraphs between headings/lists and verify paragraph content is captured separately from list items and tables

**Note**: User Story 3 (section association, S-1) is **deferred to WI-5** per AR-004 implementation guardrails: "DO NOT try to associate list items with sections in this module." Source line numbers on every element provide the data WI-5 needs for line-range-based association.

### Tests for User Story 4 (TDD — write FIRST, verify FAIL)

- [ ] T031 [US4] Write failing test `test_standalone_paragraph` in `src/parse/clauses.rs`: document with a paragraph (not inside list/table) produces `ExtractedParagraph` with correct text and `source_line` *(S-2)*
- [ ] T032 [US4] Write failing test `test_paragraph_not_captured_inside_list` in `src/parse/clauses.rs`: paragraph text inside a list item is NOT captured as a standalone paragraph (it goes into the list item's text instead)
- [ ] T033 [US4] Write failing test `test_paragraph_with_inline_formatting` in `src/parse/clauses.rs`: paragraph with bold/italic/links produces plain text *(SEC-3)*
- [ ] T034 [US4] Write failing test `test_multiple_paragraphs` in `src/parse/clauses.rs`: document with 3 paragraphs produces 3 `ExtractedParagraph`s in document order *(S-2)*

### Implementation for User Story 4

- [ ] T035 [US4] Implement paragraph extraction in `extract_clauses` in `src/parse/clauses.rs`: track `in_standalone_paragraph`, accumulate text only when `list_type_stack.is_empty() && !in_table`, handle `Start(Paragraph)`/`End(Paragraph)` events, use `offset_to_line` for source line *(S-2, M-4)*
- [ ] T036 [US4] Verify all US4 tests pass: `cargo test extract_clauses --lib` — all T031-T034 tests green
- [ ] T037 [US4] Run `cargo clippy -- -D warnings` and `cargo fmt --check` — zero warnings, formatting clean

**Checkpoint**: Paragraph extraction works. All user stories (US1, US2, US4) independently testable. US3 deferred to WI-5.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Integration tests, edge case hardening, full verification

- [ ] T038 Write integration test `test_full_document_extraction` in `src/parse/clauses.rs`: document with headings, ordered lists, unordered lists, nested lists, tables, and paragraphs produces correct `ExtractedContent` with all element types in document order
- [ ] T039 Write test `test_document_with_lists_and_tables_mixed` in `src/parse/clauses.rs`: interleaved lists and tables are all captured correctly
- [ ] T040 Write test `test_list_item_with_code_block_excluded` in `src/parse/clauses.rs`: code block inside a list item does not contribute to item text *(EC-8)*
- [ ] T041 Write test `test_nesting_depth_saturates_at_255` in `src/parse/clauses.rs`: verify `nesting_depth` saturates at 255 for absurdly deep nesting *(SEC-4)* — use programmatically generated input
- [ ] T042 Write test `test_restarted_numbered_list` in `src/parse/clauses.rs`: two separate ordered lists in the same document produce independent sets of `ExtractedListItem`s, each starting at `nesting_depth: 0` *(EC-6)*
- [ ] T043 Write test `test_mixed_bullet_markers` in `src/parse/clauses.rs`: bullet list using `-`, `*`, and `+` markers all produce `ListType::Unordered` items *(EC-7)*
- [ ] T044 Write doc tests for `ExtractedListItem`, `ExtractedTable`, `ExtractedParagraph`, `ExtractedContent` matching examples in `contracts/api.rs` — verify `cargo test --doc` passes
- [ ] T045 Run full verification: `cargo build && cargo test --lib && cargo test --doc && cargo clippy -- -D warnings && cargo fmt --check`
- [ ] T046 Run quickstart.md validation commands: verify all commands in `specs/004-structural-extraction-clauses/quickstart.md` succeed

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **US1 — Lists (Phase 3)**: Depends on Foundational (Phase 2)
- **US2 — Tables (Phase 4)**: Depends on Foundational (Phase 2); can run after US1 since both modify `extract_clauses` in same file
- **US4 — Paragraphs (Phase 5)**: Depends on Foundational (Phase 2); can run after US2 since both modify `extract_clauses` in same file
- **Polish (Phase 6)**: Depends on US1, US2, US4 completion

### User Story Dependencies

- **US1 (P1)**: Can start after Phase 2 — no dependencies on other stories
- **US2 (P2)**: Can start after Phase 2 — technically independent but shares `extract_clauses` function with US1; recommended sequential after US1
- **US3 (P3)**: **DEFERRED to WI-5** per AR-004 guardrails — not implemented in this feature
- **US4 (P4)**: Can start after Phase 2 — shares `extract_clauses` function; recommended sequential after US2

### Within Each User Story

- Tests MUST be written and FAIL before implementation (Constitution Principle IV)
- Implementation follows test completion
- Clippy/fmt verification after each story completes
- Story complete before moving to next priority

### Parallel Opportunities

- T002 and T003 can run in parallel (different files: `clauses.rs` vs `mod.rs`)
- Within US1: T009-T016 (test writing) are all in the same file but can be written together
- Within US2: T022-T027 (test writing) can be written together
- Within US4: T031-T034 (test writing) can be written together
- T044 (doc tests) can run in parallel with T038-T041 (integration tests)

---

## Parallel Example: User Story 1

```bash
# Write all US1 tests together (same file, #[cfg(test)] module):
T009: test_ordered_list_three_items
T010: test_unordered_list_three_items
T011: test_nested_list_three_levels
T012: test_mixed_list_types
T013: test_deeply_nested_list_six_levels
T014: test_inline_formatting_stripped
T015: test_multi_paragraph_list_item
T016: test_empty_list_item_excluded

# Verify all fail: cargo test extract_clauses --lib (expect 8 failures)

# Implement list extraction (T017-T019) — sequential within the function

# Verify all pass: cargo test extract_clauses --lib (expect 0 failures)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T004)
2. Complete Phase 2: Foundational (T005-T008)
3. Complete Phase 3: User Story 1 (T009-T021)
4. **STOP and VALIDATE**: `cargo test --lib` — all list extraction tests pass
5. List extraction is functional; tables and paragraphs added incrementally

### Incremental Delivery

1. Setup + Foundational → module compiles, empty extraction works
2. Add US1 (Lists) → test independently → ordered, unordered, nested lists work (MVP!)
3. Add US2 (Tables) → test independently → GFM tables extracted
4. Add US4 (Paragraphs) → test independently → standalone paragraphs captured
5. Polish → integration tests, doc tests, full verification
6. Each story adds extraction capability without breaking previous stories

### Deferred Work

- **US3 (Section Association)**: Deferred to WI-5 per AR-004 guardrails. Source line numbers on every element provide the data WI-5 needs for line-range-based section association.

---

## Notes

- All tasks target a single new file (`src/parse/clauses.rs`) plus minor changes to `src/parse/mod.rs`
- TDD is mandatory per Constitution Principle IV — every test must fail before implementation
- AR-004 guardrails are traced in task descriptions (SEC-*, EC-*, M-*, AC-*)
- No new dependencies are added — pulldown-cmark 0.13.x already present
- `u8` for `nesting_depth` with saturation at 255 *(SEC-4)* — not `usize`
- `Options::ENABLE_TABLES` *(SEC-7)* — not default parser options
- Event-based parsing only *(SEC-6)* — no regex for structural extraction
