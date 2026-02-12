# Tasks: OSCAL Metadata Assembly

**Input**: Design documents from `/specs/011-oscal-metadata/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/oscal-metadata.rs, quickstart.md

**Tests**: TDD is mandatory per Constitution Principle IV. All tests are written before implementation and must fail initially.

**Organization**: Tasks are grouped by user story. US1+US4 are combined because US4 (shared function) is inherently satisfied by implementing US1's `assemble_metadata` as a single shared function.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Dependencies)

**Purpose**: Add required crate dependencies before any code is written

- [x] T001 Add `v4` feature to existing `uuid` dependency and add `chrono` crate with `serde` feature in `Cargo.toml`
- [x] T002 Run `cargo audit` to verify `chrono` and updated `uuid` have no RustSec advisories (Constitution XI)

**Checkpoint**: `cargo check` passes with new dependencies available; `cargo audit` clean

---

## Phase 2: Foundational (Types & Module Wiring)

**Purpose**: Define types, constant, and module structure that ALL user stories depend on

**CRITICAL**: No user story work can begin until this phase is complete

- [x] T003 Create `src/oscal/metadata.rs` with `OSCAL_VERSION` constant, `OscalMetadata` struct (with serde derives and `#[serde(rename)]` for `last-modified` and `oscal-version`), `MetadataOptions` struct (with `Default` derive), and `assemble_metadata` function stub returning `todo!()` — per contract in `specs/011-oscal-metadata/contracts/oscal-metadata.rs`
- [x] T004 [P] Update `src/oscal/mod.rs` to declare `pub mod metadata;` and re-export `metadata::{assemble_metadata, MetadataOptions, OscalMetadata, OSCAL_VERSION}`
- [x] T005 [P] Update `src/lib.rs` to add `pub use oscal::{OscalMetadata, assemble_metadata};`

**Checkpoint**: `cargo check` passes — types are visible from `forge::oscal::metadata::*` and `forge::OscalMetadata`

---

## Phase 3: User Story 1 + User Story 4 — Core Metadata Assembly (Priority: P1) MVP

**Goal**: Implement the single shared `assemble_metadata` function that produces all five required OSCAL metadata fields. US4 (shared function across artifact types) is satisfied by this being a single, reusable function.

**Independent Test**: Call `assemble_metadata` with a `DocumentMetadata` and `MetadataOptions` overrides; verify all five fields are correct.

### TDD Tests for US1 (write FIRST, must FAIL before implementation)

> **NOTE: Write these tests FIRST in `src/oscal/metadata.rs` `#[cfg(test)] mod tests`, ensure they FAIL before implementation (T012)**

- [x] T006 [US1] Write test `assemble_with_overrides_uses_injected_values` — inject fixed UUID and timestamp via `MetadataOptions`, assert both are returned unchanged in `src/oscal/metadata.rs`
- [x] T007 [P] [US1] Write test `assemble_populates_title_from_document_metadata` — set title to "Access Control Policy", assert `metadata.title` matches in `src/oscal/metadata.rs`
- [x] T008 [P] [US1] Write test `assemble_populates_version_from_document_metadata` — set version to "2.1", assert `metadata.version` matches in `src/oscal/metadata.rs`
- [x] T009 [P] [US1] Write test `assemble_sets_oscal_version_to_1_2_0` — assert `metadata.oscal_version` equals "1.2.0" in `src/oscal/metadata.rs`
- [x] T010 [P] [US1] Write test `assemble_generates_utc_timestamp` — call without override, parse `last_modified`, assert timezone is UTC in `src/oscal/metadata.rs`
- [x] T011 [P] [US1] Write test `serialization_produces_correct_json_field_names` — serialize `OscalMetadata` to JSON, assert keys include `last-modified` and `oscal-version` (hyphenated) in `src/oscal/metadata.rs`

### Implementation for US1

- [x] T012 [US1] Implement `assemble_metadata` function body in `src/oscal/metadata.rs` — replace `todo!()` with production logic per contract: unwrap `MetadataOptions` with defaults, emit `tracing::warn!` for empty title (EC-1), construct `OscalMetadata` with `Uuid::new_v4()` / `Utc::now()` / cloned title+version / `OSCAL_VERSION`. Add `#[instrument(skip(doc_metadata), level = "debug")]` attribute for observability (Constitution IX).
- [x] T013 [US1] Run `cargo test --lib -- oscal::metadata` and verify all US1 tests (T006–T011) pass

**Checkpoint**: `assemble_metadata` produces all five required fields. US4 (shared function, traces to PRD M-6) is satisfied — single function exists at `forge::oscal::metadata::assemble_metadata`.

---

## Phase 4: User Story 2 — Unique Artifact UUIDs Per Generation (Priority: P1)

**Goal**: Verify each artifact generation produces a unique UUID v4, distinct from any other generation run.

**Independent Test**: Call `assemble_metadata` twice with no UUID override; verify the two UUIDs differ and both conform to UUID v4 format.

### TDD Tests for US2

- [x] T014 [P] [US2] Write test `assemble_generates_valid_uuid_v4` — call without override, verify version nibble = 4 and variant bits are correct in `src/oscal/metadata.rs`
- [x] T015 [P] [US2] Write test `assemble_two_calls_produce_different_uuids` — call twice without override, assert UUIDs differ in `src/oscal/metadata.rs`
- [x] T016 [US2] Run `cargo test --lib -- oscal::metadata` and verify all US2 tests (T014–T015) pass

**Checkpoint**: UUID v4 generation is correct and unique per call.

---

## Phase 5: User Story 3 — Metadata from PolicyDocument Defaults (Priority: P2)

**Goal**: Verify metadata assembly handles edge cases — empty title (EC-1), default version "0.0.0" (EC-2), and special characters in title (EC-5).

**Independent Test**: Call `assemble_metadata` with empty title, default version, and special characters; verify graceful handling.

### TDD Tests for US3

- [x] T017 [P] [US3] Write test `assemble_empty_title_passes_through` — pass empty title, assert `metadata.title` is empty string (edge case EC-1; `tracing::warn!` emission verified by code review, not test assertion) in `src/oscal/metadata.rs`
- [x] T018 [P] [US3] Write test `assemble_default_version_passthrough` — pass version "0.0.0", assert `metadata.version` equals "0.0.0" (edge case EC-2) in `src/oscal/metadata.rs`
- [x] T019 [P] [US3] Write test `assemble_special_characters_in_title` — pass title with quotes, ampersands, Unicode, assert `metadata.title` preserves them (edge case EC-5) in `src/oscal/metadata.rs`
- [x] T020 [US3] Run `cargo test --lib -- oscal::metadata` and verify all US3 tests (T017–T019) pass

**Checkpoint**: Edge cases handled correctly. All 11 unit tests pass.

---

## Phase 6: Polish & Verification

**Purpose**: Final quality gates per Constitution and SEC requirements

- [x] T021 Run full test suite `cargo test --lib -- oscal::metadata` — all 11 tests pass
- [x] T022 [P] Run `cargo test --doc` to verify rustdoc examples compile and pass
- [x] T023 [P] Run `cargo clippy -- -D warnings` and fix any warnings in `src/oscal/metadata.rs`
- [x] T024 [P] Run `cargo fmt --check` and fix any formatting issues
- [x] T025 Verify SEC compliance by code review of `src/oscal/metadata.rs`: SEC-1 (no system info), SEC-2 (UTC only), SEC-3 (empty title safe), SEC-4 (default version used as-is), SEC-5 (pure function), SEC-6 (MetadataOptions not CLI-exposed)

**Checkpoint**: All tests pass (lib + doc), no clippy warnings, formatting clean, SEC-1 through SEC-6 verified.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 (`Cargo.toml` updated with new deps)
- **US1+US4 (Phase 3)**: Depends on Phase 2 (types and module wiring in place)
- **US2 (Phase 4)**: Depends on Phase 3 (function implemented) — adds UUID-specific tests
- **US3 (Phase 5)**: Depends on Phase 3 (function implemented) — adds edge case tests
- **Polish (Phase 6)**: Depends on all story phases complete

### User Story Dependencies

- **US1+US4 (P1)**: Can start after Foundational (Phase 2) — MVP
- **US2 (P1)**: Can start after US1+US4 — adds uniqueness tests to already-working implementation
- **US3 (P2)**: Can start after US1+US4 — adds edge case tests; can run in parallel with US2
- US2 and US3 are independent of each other

### Within Each User Story

- Tests MUST be written and FAIL before implementation (TDD, Constitution IV)
- Implementation follows tests
- Verification run confirms tests pass

### Parallel Opportunities

- T004 and T005 can run in parallel (different files: `mod.rs` vs `lib.rs`)
- All US1 tests T007–T011 can run in parallel (same file but independent test functions)
- US2 tests T014–T015 can run in parallel
- US3 tests T017–T019 can run in parallel
- US2 (Phase 4) and US3 (Phase 5) can run in parallel after US1 implementation
- T022, T023, and T024 can run in parallel (doc tests vs clippy vs fmt)

---

## Parallel Example: US1 Tests

```bash
# Launch all US1 tests in parallel (same file, independent functions):
Task: "T007 Write test assemble_populates_title_from_document_metadata in src/oscal/metadata.rs"
Task: "T008 Write test assemble_populates_version_from_document_metadata in src/oscal/metadata.rs"
Task: "T009 Write test assemble_sets_oscal_version_to_1_2_0 in src/oscal/metadata.rs"
Task: "T010 Write test assemble_generates_utc_timestamp in src/oscal/metadata.rs"
Task: "T011 Write test serialization_produces_correct_json_field_names in src/oscal/metadata.rs"
```

## Parallel Example: US2 + US3 After Implementation

```bash
# US2 and US3 test phases can run in parallel after US1 implementation:
Task: "Write US2 UUID uniqueness tests in src/oscal/metadata.rs"
Task: "Write US3 edge case tests in src/oscal/metadata.rs"
```

---

## Implementation Strategy

### MVP First (US1+US4 Only)

1. Complete Phase 1: Setup (update Cargo.toml)
2. Complete Phase 2: Foundational (types, module wiring)
3. Complete Phase 3: US1+US4 (TDD tests → implement → verify)
4. **STOP and VALIDATE**: `cargo test --lib -- oscal::metadata` — all 6 core tests pass (T006–T011)
5. Function is usable by downstream work items (WI-9, WI-13)

### Incremental Delivery

1. Setup + Foundational → Types and module structure ready
2. US1+US4 → Core function works, all 5 fields correct → MVP complete
3. US2 → UUID uniqueness verified → Confidence in artifact identity
4. US3 → Edge cases verified → Robustness for real-world documents
5. Polish → Quality gates pass → Ready for merge

### Requirement Traceability

| Task | Requirements | Acceptance Criteria | SEC |
|------|-------------|--------------------|----|
| T002 | XI (dep security) | — | — |
| T003 | M-1–M-6, S-1, S-2 | — | — |
| T006 | S-1, S-2 | — | — |
| T007 | M-2 | AC-3 | SEC-3 |
| T008 | M-3 | AC-4 | SEC-4 |
| T009 | M-5 | AC-6 | — |
| T010 | M-4 | AC-5 | SEC-2 |
| T011 | M-2, M-5 | — | — |
| T012 | M-1–M-6, S-1, S-2 | AC-1–AC-7 | SEC-1, SEC-5 |
| T014 | M-1 | AC-1 | — |
| T015 | M-1 | AC-2 | — |
| T017 | M-2 | AC-8 | SEC-3 |
| T018 | M-3 | AC-8 | SEC-4 |
| T019 | M-2 | — | — |
| T022 | III (contract docs) | — | — |
| T025 | — | — | SEC-1–SEC-6 |

---

## Notes

- [P] tasks = different files or independent test functions, no dependencies
- [Story] label maps task to specific user story for traceability
- US4 (spec.md) traces to PRD requirement M-6; combined with US1 because the shared function is the implementation of US1
- All test code lives in `#[cfg(test)] mod tests` within `src/oscal/metadata.rs` (Rust convention)
- Tests use `MetadataOptions` with fixed UUID/timestamp for deterministic assertions (S-1, S-2)
- `DocumentMetadata` must be constructed with all required fields (use `Default::default()` then override)
- The function returns `Result<OscalMetadata, ForgeError>` but is currently infallible (always `Ok`)
- EC-3 ("unexpected clock value") has no task — `chrono::Utc::now()` always returns a valid `DateTime<Utc>`; this edge case is covered by crate guarantee and requires no test
- T017 (`assemble_empty_title_passes_through`) tests the return value only; the `tracing::warn!` emission for EC-1 is verified by code review in T025, not by test assertion (avoids adding `tracing-test` dependency for a single warn)
- Total tasks: 25 (T001–T025). Total production code: ~50 lines. Total test code: ~150 lines.
