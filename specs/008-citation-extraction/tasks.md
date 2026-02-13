# Tasks: Citation and Reference Extraction (WI-8)

**Input**: Design documents from `/specs/008-citation-extraction/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/citation_extraction.rs

**Tests**: TDD is mandatory per Constitution Principle IV. Tests are written before implementation within each user story phase.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Module initialization and project structure

- [X] T001 Create src/citation.rs module skeleton with doc comments, imports (`regex::Regex`, `std::sync::LazyLock`, `crate::model::Citation`, `crate::error::ForgeError`, `uuid::Uuid`, `crate::uuid::FORGE_NAMESPACE_UUID`), and placeholder function signatures matching contracts/citation_extraction.rs
- [X] T002 Add `pub mod citation;` declaration and `pub use citation::extract_citations;` re-export to src/lib.rs

---

## Phase 2: Foundational (Domain Model Extension)

**Purpose**: Extend domain model with citations support — MUST complete before any user story

**⚠️ CRITICAL**: No citation extraction work can begin until this phase is complete

- [X] T003 Add `PartialEq` derive to `Citation` struct and add `pub citations: Vec<Citation>` field to `PolicyRequirement` in src/model/mod.rs
- [X] T004 [P] Update test helpers and constructors in src/model/mod.rs and src/model/assemble.rs to include `citations: vec![]`
- [X] T005 [P] Update test helpers and constructors in src/parse/atomize.rs and src/parse/clauses.rs to include `citations: vec![]`
- [X] T006 [P] Update test helpers and constructors in src/uuid.rs, src/oscal/catalog.rs, and src/oscal/parts.rs to include `citations: vec![]`
- [X] T007 [P] Update bench tests in benches/atomize.rs and benches/uuid_benchmark.rs to include `citations: vec![]` in PolicyRequirement construction
- [X] T008 Verify `cargo test --workspace` passes with zero regressions and `cargo clippy --workspace -- -D warnings` passes

**Checkpoint**: Domain model ready — citation extraction implementation can now begin

---

## Phase 3: User Story 1 — Extract Inline URLs (Priority: P1) 🎯 MVP

**Goal**: Detect inline URLs (http://, https://) in requirement text, extract them into Citation objects, strip from prose, and normalize whitespace.

**Independent Test**: Parse a requirement containing an inline URL, run citation extraction, verify the URL appears in a Citation and is removed from the prose.

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T009 [US1] Write unit tests for URL extraction in src/citation.rs: single URL extraction (AC-1), multiple URLs in one requirement (AC-2), URL in parentheses extracted without parens (EC-4), duplicate URLs produce separate Citations (EC-5), no-citations text unchanged (EC-1), whitespace normalization after stripping (EC-2)

### Implementation for User Story 1

- [X] T010 [US1] Define `LazyLock<Regex>` URL pattern `https?://[^\s\)\]>,;]+` as static in src/citation.rs (SEC-1, SEC-2, SEC-7)
- [X] T011 [US1] Implement `generate_citation_id(requirement_id: &str, citation_text: &str) -> String` using UUID v5 with `FORGE_NAMESPACE_UUID` namespace, hashing `"{requirement_id}:{citation_text}"` in src/citation.rs (R-1)
- [X] T012 [US1] Implement `extract_citations_from_text(requirement_id: &str, text: &str) -> Result<(String, Vec<Citation>), ForgeError>` with URL regex matching, byte range tracking for overlap detection, and Citation construction in src/citation.rs
- [X] T013 [US1] Implement prose cleanup in `extract_citations_from_text`: replace matched text with space, collapse consecutive spaces, trim, normalize punctuation artifacts (double commas, trailing commas before periods) in src/citation.rs (R-9)
- [X] T014 [US1] Verify all US1 tests pass with `cargo test citation`

**Checkpoint**: URL extraction works independently — inline URLs are detected, extracted, and prose is cleaned

---

## Phase 4: User Story 2 — Extract Bibliographic References (Priority: P1)

**Goal**: Detect bibliographic references (NIST SP, ISO, RFC, FIPS) with optional revision and section suffixes, and extract them as Citation objects with descriptive text.

**Independent Test**: Parse a requirement referencing "NIST SP 800-53 Rev 5", run citation extraction, verify a Citation captures the reference text.

### Tests for User Story 2

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T015 [P] [US2] Write unit tests for bibliographic extraction in src/citation.rs: NIST SP with Rev and Section suffix (AC-6), ISO standard number, RFC number, FIPS number, multiple standards in one requirement producing separate Citations

### Implementation for User Story 2

- [X] T016 [US2] Define `LazyLock<Regex>` bibliographic pattern `\b(?:NIST\s+SP|ISO|RFC|FIPS)\s+[\d]+[-\w.]*(?:\s+Rev\.?\s*\d+)?(?:,?\s+Section\s+[\w.-]+)?` as static in src/citation.rs (SEC-3)
- [X] T017 [US2] Add bibliographic pattern matching to `extract_citations_from_text` with priority-based overlap detection: URL matches processed first, bibliographic matches skip overlapping byte ranges (R-5) in src/citation.rs
- [X] T018 [US2] Verify all US2 tests pass with `cargo test citation`

**Checkpoint**: URL + bibliographic extraction both work — core P1 extraction functionality complete

---

## Phase 5: User Story 5 — Pipeline Enrichment Function (Priority: P1)

**Goal**: Implement document-level enrichment that walks the full PolicyDocument tree, integrates into the conversion pipeline, and processes all requirements.

**Independent Test**: Pass a PolicyDocument with multiple requirements containing URLs and bibliographic refs, verify all requirements are processed with citations attached and prose cleaned.

### Tests for User Story 5

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T019 [P] [US5] Write unit tests for document-level extraction in src/citation.rs: multiple sections each with requirements, nested subsections, empty document with zero requirements (EC-7), requirements with mixed citation types (URL + bibliographic in same requirement)

### Implementation for User Story 5

- [X] T020 [US5] Implement `extract_citations(document: &mut PolicyDocument) -> Result<(), ForgeError>` with recursive section walking (consistent with `assign_stable_ids_to_section` pattern in src/uuid.rs) in src/citation.rs
- [X] T021 [US5] Add tracing instrumentation: `#[tracing::instrument]` on `extract_citations`, DEBUG-level log of citation count per requirement in src/citation.rs
- [X] T022 [US5] Integrate citation extraction into conversion pipeline in src/cli/convert.rs: call `extract_citations(&mut document)` after UUID assignment (`assign_stable_ids`) and before OSCAL generation
- [X] T023 [US5] Write integration test: end-to-end PolicyDocument with multiple citation types through `extract_citations`, verifying all requirements have citations populated and text cleaned in src/citation.rs
- [X] T024 [US5] Verify all US5 tests pass and pipeline produces correct output with `cargo test`

**Checkpoint**: Full pipeline works — documents flow through citation extraction end-to-end. MVP complete (URL + bibliographic + pipeline).

---

## Phase 6: User Story 3 — Handle Malformed URLs (Priority: P2)

**Goal**: Detect scheme-less URLs (www.example.com) and preserve them as Citations for downstream validation by back_matter's `classify_url`.

**Independent Test**: Parse a requirement with "www.example.com/policy", verify a Citation is created with `url: Some("www.example.com/policy")`.

### Tests for User Story 3

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T025 [P] [US3] Write unit tests for scheme-less URL detection in src/citation.rs: `www.example.com/policy` extracted as Citation with url field populated (EC-3), scheme-less URL in parentheses, scheme-less URL alongside a full URL in same requirement

### Implementation for User Story 3

- [X] T026 [US3] Define `LazyLock<Regex>` scheme-less URL pattern `\bwww\.[^\s\)\]>,;]+` as static in src/citation.rs (R-7)
- [X] T027 [US3] Add scheme-less URL matching to `extract_citations_from_text`: process after full URL matches, skip overlapping byte ranges, set `url: Some(matched_text)` on Citation in src/citation.rs
- [X] T028 [US3] Verify all US3 tests pass with `cargo test citation`

**Checkpoint**: Malformed/scheme-less URLs are detected and preserved for downstream back_matter validation

---

## Phase 7: User Story 4 — Detect Cross-References (Priority: P2)

**Goal**: Detect internal cross-references (Section X.Y, Appendix A, Table N) and extract as Citation objects without a URL.

**Independent Test**: Parse a requirement containing "See Section 3.2", verify a Citation is created with `text = "Section 3.2"` and `url = None`.

### Tests for User Story 4

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T029 [P] [US4] Write unit tests for cross-reference detection in src/citation.rs: "Section 3.2" extracted (AC-7), "Appendix A" extracted, "Table 2" extracted, lowercase "section" NOT matched (EC-6), cross-ref alongside URL in same requirement

### Implementation for User Story 4

- [X] T030 [US4] Define `LazyLock<Regex>` cross-reference pattern `\b(?:Section|Appendix|Table)\s+[\dA-Z]+(?:\.\d+)*\b` as static in src/citation.rs (SEC-4)
- [X] T031 [US4] Add cross-reference matching to `extract_citations_from_text`: process after URL and bibliographic matches, skip overlapping byte ranges, set `url: None` on Citation (R-5) in src/citation.rs
- [X] T032 [US4] Verify all US4 tests pass with `cargo test citation`

**Checkpoint**: All four citation types (URL, bibliographic, scheme-less URL, cross-reference) are detected and extracted

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Verification, performance, security, and quality gates

- [X] T033 [P] Write idempotency test (S-3): run `extract_citations` twice on same document, verify citations and text are identical after second pass in src/citation.rs
- [X] T034 [P] Write performance benchmark (SEC-6): construct document with 1000+ PolicyRequirements containing mixed citation types, verify `extract_citations` completes under 1 second in src/citation.rs
- [X] T035 [P] Write ReDoS resistance tests (SEC-1): test each regex pattern against pathological input strings (long strings of URL-like characters, deeply nested patterns), verify bounded execution time in src/citation.rs
- [X] T036 Verify `cargo fmt --check` passes
- [X] T037 Verify `cargo clippy --workspace -- -D warnings` passes
- [X] T038 Verify `cargo doc --no-deps` builds without warnings
- [X] T039 Verify test coverage >90% for citation module (SC-007) and run quickstart.md validation

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Foundational — creates core extraction infrastructure
- **US2 (Phase 4)**: Depends on US1 — extends extraction function with bibliographic patterns
- **US5 (Phase 5)**: Depends on US1 + US2 — wraps extraction in document walker and pipeline
- **US3 (Phase 6)**: Depends on US1 — extends extraction function with scheme-less URL pattern
- **US4 (Phase 7)**: Depends on US1 — extends extraction function with cross-reference pattern
- **Polish (Phase 8)**: Depends on all user stories being complete

### Execution Order

```text
Phase 1 → Phase 2 → Phase 3 (US1) → Phase 4 (US2) → Phase 5 (US5) → Phase 6 (US3) → Phase 7 (US4) → Phase 8
```

US3 and US4 (both P2) can be done in either order but must follow US1 (they extend the same function). US5 follows US2 to ensure the pipeline wraps all P1 functionality.

### Within Each User Story

- Tests MUST be written and FAIL before implementation (TDD)
- Regex pattern definition before matching logic
- Core implementation before integration
- Verification before moving to next phase

### Parallel Opportunities

- Phase 2: T004, T005, T006, T007 can all run in parallel (different files)
- Phase 4: T015 (US2 tests) can be written while US1 implementation is verified
- Phase 5: T019 (US5 tests) can be written while US2 is verified
- Phase 6: T025 (US3 tests) can be written during any prior phase
- Phase 7: T029 (US4 tests) can be written during any prior phase
- Phase 8: T033, T034, T035 can all run in parallel (independent verification tests)

---

## Parallel Example: Phase 2 (Foundational)

```bash
# Launch all test helper updates in parallel (different files):
Task: "Update test helpers in src/model/mod.rs and src/model/assemble.rs"
Task: "Update test helpers in src/parse/atomize.rs and src/parse/clauses.rs"
Task: "Update test helpers in src/uuid.rs, src/oscal/catalog.rs, src/oscal/parts.rs"
Task: "Update bench tests in benches/atomize.rs and benches/uuid_benchmark.rs"
```

## Parallel Example: Phase 8 (Polish)

```bash
# Launch all verification tests in parallel:
Task: "Write idempotency test in src/citation.rs"
Task: "Write performance benchmark in src/citation.rs"
Task: "Write ReDoS resistance tests in src/citation.rs"
```

---

## Implementation Strategy

### MVP First (Phase 1 + 2 + 3 + 4 + 5)

1. Complete Phase 1: Setup (module skeleton)
2. Complete Phase 2: Foundational (domain model extension)
3. Complete Phase 3: User Story 1 — URL extraction
4. Complete Phase 4: User Story 2 — Bibliographic extraction
5. Complete Phase 5: User Story 5 — Pipeline integration
6. **STOP and VALIDATE**: Full pipeline works with URL + bibliographic extraction
7. This delivers all P1 user stories

### Incremental Delivery

1. Setup + Foundational → Domain model ready
2. Add US1 (URLs) → Test independently → Core extraction working (MVP!)
3. Add US2 (Bibliographic) → Test independently → Two pattern types working
4. Add US5 (Pipeline) → Test end-to-end → Full pipeline operational
5. Add US3 (Malformed URLs) → Test independently → Edge case coverage
6. Add US4 (Cross-references) → Test independently → All patterns complete
7. Polish → Performance, idempotency, quality gates verified

### Key Design Decisions Applied

- **R-1**: Citation IDs use UUID v5 (deterministic, idempotent)
- **R-2**: No `validated` field — back_matter handles URL validation at OSCAL layer
- **R-3**: `&mut PolicyDocument` enrichment pattern (consistent with WI-7)
- **R-5**: Priority order for overlapping matches: URL > scheme-less URL > bibliographic > cross-ref
- **R-7**: Scheme-less URLs get `url: Some(text)` for downstream classification
- **R-8**: All patterns use `LazyLock<Regex>` (compiled once, RE2-style)
- **R-9**: Prose cleanup: strip → collapse whitespace → trim → normalize punctuation

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- All regex patterns MUST use Rust `regex` crate (RE2-style, linear-time) per SEC-1
- No `validated` field on Citation — back_matter.rs handles URL validation downstream (R-2)
- No network I/O — URLs are extracted and validated syntactically only (SEC-8)
- Commit after each phase completion
- Stop at any checkpoint to validate story independently
