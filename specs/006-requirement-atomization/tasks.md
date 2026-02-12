# Tasks: Requirement Atomization

**Input**: Design documents from `/specs/006-requirement-atomization/`
**Prerequisites**: plan.md (required), spec.md (required), prd.md, ar.md, sec.md, data-model.md, contracts/atomize-api.md, quickstart.md

**Tests**: Included — TDD is mandatory per constitution Principle IV. Tests MUST be written first and MUST fail before implementation.

**Organization**: Tasks grouped by user story. Each story is independently testable after foundational phase.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Add dependencies and create module structure for atomization feature.

- [X] T001 Add `regex = "1"` dependency to `Cargo.toml` under `[dependencies]`. After adding, run pre-addition security checks per constitution Principle XI: `cargo audit`, `cargo deny check advisories` (if cargo-deny configured), verify license is MIT/Apache-2.0, and run `cargo tree -p regex` to review transitive dependencies. Also add `tracing = "0.1"` for observability (constitution Principle IX).
- [X] T002 [P] Create `src/parse/atomize.rs` module file and add `pub mod atomize;` to `src/parse/mod.rs`; re-export public types via `pub use atomize::{AtomizationResult, atomize_document, atomize_requirement, preliminary_id};` in `src/parse/mod.rs`
- [X] T003 [P] Create test fixture reference files: `tests/fixtures/compound_statements.txt` (compound statements with expected split counts from AC-1, AC-2, AC-3 of PRD), `tests/fixtures/atomic_statements.txt` (atomic statements from AC-3, AC-4), and `tests/fixtures/edge_cases.txt` (EC-1 through EC-10 scenarios). Note: these files serve as human-readable reference documentation for test scenarios; actual test data is defined inline in unit tests.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Domain model types and core helper functions that MUST exist before any user story implementation.

**Domain Model Scope**: The domain model types (`PolicyRequirement`, `PolicySection`, `PolicyDocument`) from WI-5 do not yet exist in the codebase (`src/model/mod.rs` is empty). T004 and T005 below create the minimum domain model required for atomization. These types serve as the WI-5 domain model implementation for the fields needed by WI-6 and downstream work items. If WI-5 is later expanded, these types should be extended rather than replaced.

- [X] T004 Define `PolicyRequirement` struct in `src/model/mod.rs` with fields: `stable_id: String` (SHA-256 hex, 64 chars), `text: String` (requirement text), `source_line: usize` (1-based line number), `atom_index: usize` (0-based split position), `parent_text: Option<String>` (original compound text if split). Derive `Debug`, `Clone`, `PartialEq`, `Serialize`. See data-model.md for field validation rules.
- [X] T005 Define `PolicySection` struct (fields: `heading: String`, `requirements: Vec<PolicyRequirement>`) and `PolicyDocument` struct (fields: `title: String`, `sections: Vec<PolicySection>`) in `src/model/mod.rs`. Add `total_requirement_count(&self) -> usize` helper method on `PolicyDocument`. Derive `Debug`, `Clone`, `PartialEq`, `Serialize`.
- [X] T006 [P] Implement `pub fn preliminary_id(text: &str, source_line: usize, atom_index: usize) -> String` in `src/parse/atomize.rs`. Algorithm: construct input as `"{text}|{source_line}|{atom_index}"`, compute SHA-256 hash using `sha2::Sha256`, return 64-char lowercase hex string. See contracts/atomize-api.md for format specification. This function has no domain model dependency and can be implemented in parallel with T004/T005.
- [X] T007 Implement private helper `fn extract_subject(text: &str, first_verb_pos: usize) -> Option<String>` in `src/parse/atomize.rs`. Returns `Some(trimmed_subject)` from `text[0..first_verb_pos].trim()` if non-empty, `None` otherwise. Also implement `fn reconstruct_clause(shared_subject: &str, clause: &str) -> String` that prepends shared subject to clause fragment if clause doesn't already start with it (case-insensitive check). See contracts/atomize-api.md for examples.
- [X] T008 Define `pub struct AtomizationResult` (fields: `requirements: Vec<PolicyRequirement>`, `was_split: bool`, `original_text: Option<String>`) in `src/parse/atomize.rs`. Also implement `build_split_pattern()` as a `static SPLIT_PATTERN: LazyLock<Regex>` using `std::sync::LazyLock` with pattern `\b(and|or)\s+(must|shall|should|will)\b` (case-sensitive). Add constant `const MAX_SPLITS_PER_REQUIREMENT: usize = 50;` (SEC-5, FR-010).

**Checkpoint**: Domain model and helper infrastructure ready — user story implementation can now begin.

---

## Phase 3: User Story 1 — Split Compound Policy Statement (Priority: P1) :dart: MVP

**Goal**: Detect compound policy statements containing conjunctions paired with normative verbs and split them into individual atomic requirements with shared subject reconstruction.

**Independent Test**: Pass compound statements like "Systems must enforce MFA and must require complex passwords" through the atomizer and verify correct number of atomic requirements with accurate text.

**Traces to**: PRD M-1, M-2, M-5, M-6, M-7; FR-001, FR-002, FR-005, FR-006, FR-010; SEC-5

### Tests for User Story 1 (TDD — write first, verify they FAIL)

- [X] T009 [US1] Write unit tests in `src/parse/atomize.rs` `#[cfg(test)] mod tests` for two-part compound splitting: (1) "Systems must enforce MFA and must require complex passwords" → 2 parts with shared subject "Systems" (AC-1), (2) "The organization shall review access logs and shall revoke inactive accounts within 30 days" → 2 parts with shared subject "The organization" (AC-2), (3) verify `source_line` preserved from parent on each atomic requirement (AC-7, FR-005), (4) verify `parent_text` set to original compound text, (5) verify `was_split == true` and `original_text == Some(original)`
- [X] T010 [US1] Write unit tests in `src/parse/atomize.rs` for multi-conjunction, mixed-conjunction, and "or" splitting: (1) "All employees must complete security training and must acknowledge the acceptable use policy or must request a waiver" → 3 parts (EC-3, FR-007/S-1), (2) "Systems shall enforce MFA and shall require passwords" → 2 parts with "shall" verb (EC-6), (3) "must X and must Y or must Z" → 3 parts with mixed conjunctions (FR-008/S-2), (4) "Systems must implement MFA or must use certificate-based authentication" → 2 parts with "or" conjunction (EC-2), (5) verify sequential `atom_index` values (0, 1, 2)
- [X] T011 [US1] Write unit test in `src/parse/atomize.rs` for max split count enforcement: generate a statement with 51 "and must Y" repetitions (>50 splits), verify `was_split == false` and statement preserved as-is with `requirements.len() == 1` (EC-9, FR-010, SEC-5)

### Implementation for User Story 1

- [X] T012 [US1] Implement `pub fn atomize_requirement(requirement: &PolicyRequirement) -> Result<AtomizationResult, ForgeError>` core splitting logic in `src/parse/atomize.rs`. Algorithm: (1) match `SPLIT_PATTERN` regex against `requirement.text`, (2) if no match → return as-is (single requirement, `was_split=false`), (3) if match → count splits, check `> MAX_SPLITS_PER_REQUIREMENT` → preserve as-is + `eprintln!` warning (placeholder until T023 adds `tracing`), (4) find position of first normative verb in text, extract shared subject via `extract_subject()`, (5) split text at each conjunction+verb boundary, (6) reconstruct each clause with shared subject via `reconstruct_clause()`, (7) assign `preliminary_id()` to each, preserve `source_line`, set `parent_text`. See contracts/atomize-api.md for detailed algorithm.
- [X] T013 [US1] Implement `pub fn atomize_document(document: PolicyDocument) -> Result<PolicyDocument, ForgeError>` in `src/parse/atomize.rs`. Iterate over all `PolicySection`s, for each `PolicyRequirement` call `atomize_requirement()`, replace compound requirements with atomic parts, return updated `PolicyDocument`. Total requirement count must be >= original count (FR-006, M-6).
- [X] T014 [US1] Write integration tests in `tests/atomize_integration.rs` for `atomize_document()`: (1) document with 1 compound + 1 atomic requirement → total count increases by 1 (AC-8), (2) verify split requirements have sequential `atom_index` and shared `source_line`, (3) verify atomic requirement preserved unchanged, (4) empty document with zero sections → returned unchanged (EC-8). This file tests the public API only, per constitution Principle IV testing stack (integration tests in `tests/` directory).

**Checkpoint**: Compound statements are correctly split. Run `cargo test` — all US1 tests should pass.

---

## Phase 4: User Story 2 — Preserve Atomic Statements (Priority: P1)

**Goal**: Atomic (single-obligation) statements pass through the atomizer completely unchanged — no text modification, no false splits.

**Independent Test**: Pass atomic statements like "All systems must enforce MFA" and verify exactly one requirement returned with unmodified text.

**Traces to**: PRD M-3; FR-003; SEC-9

### Tests for User Story 2 (TDD — write first, verify they FAIL or pass from US1 implementation)

- [X] T015 [US2] Write unit tests in `src/parse/atomize.rs` for atomic statement preservation: (1) "All systems must enforce MFA" → 1 requirement, text unchanged (AC-3, FR-003), (2) "Passwords must be at least 12 characters" → 1 requirement, text unchanged (AC-4), (3) verify `was_split == false`, `original_text == None`, `atom_index == 0`, `parent_text == None` (SEC-9)
- [X] T016 [US2] Write unit tests in `src/parse/atomize.rs` for non-splitting edge cases: (1) "Systems must encrypt and store data securely" → preserved as-is, no normative verb after "and" (EC-1), (2) "Systems must implement logging and monitoring" → preserved as-is (EC-5), (3) empty string "" and whitespace-only "   " → preserved as-is without error (EC-7, SEC-3), (4) "Systems MUST enforce MFA and MUST require passwords" → preserved as-is, case-sensitive matching (EC-10)

### Implementation for User Story 2

- [X] T017 [US2] Verify and refine the no-match pass-through path in `atomize_requirement()` in `src/parse/atomize.rs`: ensure (1) original `text` is returned byte-for-byte unchanged (SEC-9), (2) `source_line` matches input, (3) `atom_index == 0`, (4) `parent_text == None`, (5) `stable_id` is set via `preliminary_id()`. Fix any edge case failures found by T015/T016 tests.

**Checkpoint**: Atomic statements pass through unchanged. All US1 + US2 tests pass.

---

## Phase 5: User Story 3 — Assign Preliminary IDs to Atomic Requirements (Priority: P2)

**Goal**: Each atomic requirement receives a deterministic preliminary SHA-256 ID that is unique per requirement and identical across repeated runs.

**Independent Test**: Atomize the same compound statement twice and verify preliminary IDs are identical across runs. Verify different atom indices produce different IDs.

**Traces to**: PRD M-4; FR-004; SEC-8; AC-5, AC-6

### Tests for User Story 3 (TDD — write first, verify they FAIL or pass from earlier implementation)

- [X] T018 [US3] Write unit tests in `src/parse/atomize.rs` for preliminary ID determinism: (1) call `preliminary_id("Systems must enforce MFA", 42, 0)` twice → identical output (AC-6, FR-004), (2) verify output is exactly 64 characters and all lowercase hex (SEC-8), (3) atomize the same compound statement twice via `atomize_requirement()` → all `stable_id` values match across runs (AC-5)
- [X] T019 [US3] Write unit tests in `src/parse/atomize.rs` for preliminary ID uniqueness: (1) `preliminary_id("text", 42, 0)` vs `preliminary_id("text", 42, 1)` → different IDs (different atom_index), (2) `preliminary_id("text", 42, 0)` vs `preliminary_id("text", 43, 0)` → different IDs (different source_line), (3) compound statement split into N parts → all N `stable_id` values are unique

### Implementation for User Story 3

- [X] T020 [US3] Verify `preliminary_id()` is correctly integrated into both `atomize_requirement()` and `atomize_document()` in `src/parse/atomize.rs`: (1) all returned requirements (split and non-split) have non-empty 64-char hex `stable_id`, (2) IDs use format `"{text}|{source_line}|{atom_index}"` as SHA-256 input per contracts/atomize-api.md, (3) non-split requirements use `atom_index=0`

**Checkpoint**: All requirements have deterministic, unique preliminary IDs. All US1 + US2 + US3 tests pass.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Security hardening, observability, performance benchmarks, documentation, code quality, and mutation testing.

- [X] T021 [P] Write adversarial input tests for ReDoS resistance in `src/parse/atomize.rs`: (1) 10KB+ repetitive string ("Systems must X " repeated 1000 times) completes in <1 second (SC-007, SEC-4), (2) deeply nested conjunction patterns complete in linear time, (3) verify Rust `regex` crate's linear-time guarantee holds
- [X] T022 [P] Write Unicode edge case tests in `src/parse/atomize.rs`: (1) zero-width space between "and" and "must" (`\u{200B}`) → handled gracefully (SEC-4), (2) bidirectional override characters in requirement text → no panic or corruption, (3) multi-byte UTF-8 characters in requirement text → preserved correctly
- [X] T023 Replace `eprintln!` warnings from T012 with `tracing::warn!` in `src/parse/atomize.rs` (constitution Principle IX mandates `tracing`, NOT `log`). Implement conjunction-without-verb warning: when `atomize_requirement()` encounters text containing at least one normative verb AND a bare conjunction ("and"/"or") that the regex did not match (suggesting a near-miss compound pattern), emit `tracing::warn!` indicating a potential compound statement not confidently split (FR-009, S-3). Do NOT warn on every occurrence of "and"/"or" — only when the sentence already contains a normative verb, indicating compound structure where a second obligation may have been missed.
- [X] T024 Add `tracing::debug!` summary metrics to `atomize_document()` in `src/parse/atomize.rs`: log total requirements processed, number split, number preserved as-is (FR-011, S-4). Format: `tracing::debug!(total = %total, split = %split_count, preserved = %preserved_count, "Atomization complete")`. Uses `tracing` crate per constitution Principle IX.
- [X] T025 Run `cargo clippy -- -D warnings` and `cargo fmt --check` against all modified files; fix any warnings or formatting issues in `src/parse/atomize.rs`, `src/model/mod.rs`, `tests/atomize_integration.rs`, and `Cargo.toml`. Also perform code review checkpoint for SEC-6 (O(n*m) performance): verify that `atomize_requirement()` is O(m) per statement and `atomize_document()` iterates requirements once (O(n)), confirming overall O(n*m) complexity.
- [X] T026 Run `cargo mutants` mutation testing targeting `src/parse/atomize.rs`; analyze surviving mutants and add targeted unit tests to kill them; target >80% mutation kill rate
- [X] T027 [P] Add `criterion` benchmark for atomization hot paths in `benches/atomize.rs` (constitution Principle VI). Benchmark: (1) `atomize_requirement()` with a compound statement (2-part split), (2) `atomize_requirement()` with an atomic statement (pass-through), (3) `atomize_document()` with 100 requirements (mix of compound and atomic), (4) `preliminary_id()` throughput. Add `criterion = { version = "0.5", features = ["html_reports"] }` to `[dev-dependencies]` in `Cargo.toml` and create `[[bench]]` entry. Establish baseline performance numbers.
- [X] T028 Add `rustdoc` documentation with `# Examples`, `# Errors`, and `# Panics` sections for all public items in `src/parse/atomize.rs` (`AtomizationResult`, `atomize_document`, `atomize_requirement`, `preliminary_id`) and `src/model/mod.rs` (`PolicyRequirement`, `PolicySection`, `PolicyDocument`). Verify with `cargo doc --no-deps` — must build without warnings (constitution Principles III and V).

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup (T001, T002) — BLOCKS all user stories
  - T004, T005 (domain model in `src/model/mod.rs`) can run in parallel with T006 (preliminary_id in `src/parse/atomize.rs`) since they are in different files
  - T007 and T008 depend on T006 completing (same file: `src/parse/atomize.rs`)
  - T008 also depends on T004 (uses `PolicyRequirement` type in `AtomizationResult`)
- **User Story 1 (Phase 3)**: Depends on all Foundational tasks
- **User Story 2 (Phase 4)**: Depends on US1 implementation (T012) since both use the same `atomize_requirement()` function
- **User Story 3 (Phase 5)**: Depends on US1 implementation (T012) since IDs are assigned during atomization
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) — No dependencies on other stories
- **User Story 2 (P1)**: Depends on US1 (T012) — tests verify the no-match path in the same function
- **User Story 3 (P2)**: Depends on US1 (T012) — tests verify ID assignment in the atomization pipeline

### Within Each User Story

- Tests MUST be written FIRST and MUST FAIL before implementation (TDD mandate)
- Helper functions before core logic
- `atomize_requirement()` before `atomize_document()`
- Unit tests before integration tests

### Parallel Opportunities

**Phase 1**:
- T002 (module file in `src/parse/`) and T003 (fixture files in `tests/fixtures/`) can run in parallel — different directories

**Phase 2**:
- T006 (`preliminary_id` in `src/parse/atomize.rs`) can run in parallel with T004/T005 (domain model in `src/model/mod.rs`) — different files
- T007 and T008 are sequential after T006 (same file: `src/parse/atomize.rs`)

**Phase 3-5** (User Stories):
- Tasks within each user story are sequential (same file: `src/parse/atomize.rs`)
- T014 (integration test in `tests/atomize_integration.rs`) is in a different file but depends on T012/T013

**Phase 6** (Polish):
- T021 (ReDoS tests) and T022 (Unicode tests) can run in parallel — independent test functions
- T027 (criterion benchmarks in `benches/atomize.rs`) can run in parallel with T021/T022/T028 — different file
- T023 and T024 are sequential (both modify logging in `atomize_requirement`/`atomize_document`)

---

## Parallel Example: User Story 1

```bash
# Step 1: Write all US1 tests sequentially (same file: src/parse/atomize.rs):
Task: T009 "Two-part compound splitting tests"
Task: T010 "Multi-conjunction + EC-2 splitting tests"
Task: T011 "Max split count test"

# Step 2: Implement core logic (sequential — atomize_requirement before atomize_document):
Task: T012 "Implement atomize_requirement() in src/parse/atomize.rs"
Task: T013 "Implement atomize_document() in src/parse/atomize.rs"

# Step 3: Integration test (different file):
Task: T014 "Integration test for atomize_document() in tests/atomize_integration.rs"
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2)

1. Complete Phase 1: Setup (add regex + tracing deps, create module)
2. Complete Phase 2: Foundational (domain model, helpers, regex pattern)
3. Complete Phase 3: User Story 1 — compound splitting works
4. Complete Phase 4: User Story 2 — atomic preservation verified
5. **STOP and VALIDATE**: Run `cargo test` — all US1 + US2 tests pass
6. The atomizer can now correctly split compound statements and preserve atomic ones

### Incremental Delivery

1. Setup + Foundational → Infrastructure ready
2. Add US1 → Compound splitting works → Core value delivered (MVP!)
3. Add US2 → Atomic preservation verified → Correctness confirmed
4. Add US3 → Preliminary IDs assigned → Pipeline traceability enabled
5. Add Polish → Security hardened, observable, benchmarked, documented → Production ready

### Suggested MVP Scope

**User Stories 1 + 2** (both P1) form the natural MVP:
- Compound statements are correctly split
- Atomic statements are preserved unchanged
- This delivers the core atomization capability

**User Story 3** (P2) adds preliminary IDs, completing pipeline traceability.

---

## Summary

| Metric | Count |
|--------|-------|
| **Total tasks** | 28 |
| **Setup tasks** | 3 |
| **Foundational tasks** | 5 |
| **US1 tasks** | 6 (3 test + 3 implementation) |
| **US2 tasks** | 3 (2 test + 1 implementation) |
| **US3 tasks** | 3 (2 test + 1 implementation) |
| **Polish tasks** | 8 |
| **Parallelizable tasks** | 6 (marked with [P]: T002, T003, T006, T021, T022, T027) |
| **Files created/modified** | 7 (`Cargo.toml`, `src/model/mod.rs`, `src/parse/mod.rs`, `src/parse/atomize.rs`, `tests/atomize_integration.rs`, `tests/fixtures/*`, `benches/atomize.rs`) |

### Format Validation

All 28 tasks follow the required checklist format: `- [ ] [TaskID] [P?] [Story?] Description with file path`

- Setup phase: No story labels (correct)
- Foundational phase: No story labels (correct)
- US1/US2/US3 phases: All tasks have [US1]/[US2]/[US3] labels (correct)
- Polish phase: No story labels (correct)
- All tasks have checkbox, sequential ID, and file paths (correct)
- [P] markers only on tasks in different files with no dependencies (correct)

### Constitution Compliance

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Modular Architecture | Aligned | Feature in dedicated `src/parse/atomize.rs` submodule with explicit public API |
| III. Contract-First | Aligned | T004-T008 define types/contracts before implementation |
| IV. Test-First | Aligned | Tests (T009-T011, T015-T016, T018-T019) written before implementation |
| V. Complete Implementation | Aligned | All 28 tasks must complete before merge |
| VI. Performance-First | Aligned | T027 adds criterion benchmarks |
| VII. Security-First | Aligned | SEC-1 through SEC-10 covered; T001 includes pre-addition security checks |
| VIII. Error Handling | Aligned | Uses `ForgeError` with `thiserror` |
| IX. Observability | Aligned | T023/T024 use `tracing` crate (not `log`) |
| X. Simplicity | Aligned | YAGNI respected — heuristic regex only |
| XI. Dependency Policy | Aligned | T001 includes security checks for new dependency |

---

## Notes

- [P] tasks = different files with no dependencies on incomplete tasks
- [Story] label maps each task to its user story for traceability
- TDD mandate: tests MUST be written and FAIL before implementation
- All unit test source code goes in `src/parse/atomize.rs` inline `#[cfg(test)]` block
- Integration tests go in `tests/atomize_integration.rs` (per constitution Principle IV testing stack)
- Domain model types go in `src/model/mod.rs` (WI-5 prerequisite, created by T004/T005)
- Performance benchmarks go in `benches/atomize.rs` (per constitution Principle VI)
- Commit after each task or logical group
- Stop at any checkpoint to validate the story independently
