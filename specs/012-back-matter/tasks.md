# Tasks: OSCAL Back Matter Generation

**Input**: Design documents from `/specs/012-back-matter/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/back_matter.rs
**TDD**: Mandatory per Constitution IV — tests before implementation

**Organization**: Tasks grouped by user story. All 3 stories are P1 but ordered by logical dependency: US1 (resources) → US2 (links) → US3 (remarks audit).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story (US1, US2, US3)
- Exact file paths included in descriptions

---

## Phase 1: Setup

**Purpose**: Add dependency, error variant, namespace constant, and input contract struct

- [X] T001 Add `url` dependency to `Cargo.toml` (`url = "2"` under `[dependencies]`)
- [X] T002 [P] Add `BackMatter(String)` variant to `ForgeError` enum in `src/error.rs` with display message `"Back matter error: {0}"` and corresponding unit test
- [X] T003 [P] Add `BACK_MATTER_NAMESPACE` constant to `src/uuid.rs` — compute `Uuid::new_v5(&FORGE_NAMESPACE_UUID, b"back-matter")`, extract bytes, hardcode as `pub const BACK_MATTER_NAMESPACE: Uuid = Uuid::from_bytes([...])` with breaking-change doc comment
- [X] T004 [P] Add `Citation` struct to `src/model/mod.rs` with fields: `id: String`, `text: String`, `url: Option<String>`, `source_requirement_id: Option<String>` — derive `Debug, Clone, Serialize`

**Checkpoint**: `cargo check` passes with new types available

---

## Phase 2: Foundational (Struct Definitions & Module Wiring)

**Purpose**: Create the back matter module with all OSCAL struct types and wire into crate

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T005 Create `src/oscal/back_matter.rs` with struct definitions from `specs/012-back-matter/contracts/back_matter.rs`: `BackMatter`, `BackMatterResource`, `ResourceCitation`, `Rlink`, `OscalLink`, `Prop` — all with serde derives and `skip_serializing_if` annotations per contract. Add `todo!()` stubs for `generate_back_matter` and `generate_control_links` function signatures
- [X] T006 Register module in `src/oscal/mod.rs`: add `pub mod back_matter;` and re-export key types (`BackMatter`, `BackMatterResource`, `OscalLink`, `generate_back_matter`, `generate_control_links`)
- [X] T007 Add re-exports to `src/lib.rs`: re-export `Citation` from model and back matter public API types from oscal

**Checkpoint**: `cargo check` passes — all types importable, function stubs compile

---

## Phase 3: User Story 1 — Citations Appear as Back Matter Resources (Priority: P1) 🎯 MVP

**Goal**: Every extracted citation produces a correctly structured OSCAL back matter resource with deterministic UUID, proper title, and correct classification (URL → rlinks, bibliographic → citation.text, malformed → rlinks + prop annotation)

**Independent Test**: Convert 3 citations (2 URLs, 1 bibliographic) and verify all 3 appear in resources with correct structure

### Tests for User Story 1 (TDD — RED phase)

> **Write these tests FIRST, ensure they FAIL before implementation**

- [X] T008 [US1] Write tests for URL-based citations → rlinks resources in `src/oscal/back_matter.rs`: valid `https://` URL produces resource with `rlinks[].href` matching URL; URL with `.pdf` extension gets `media-type: "application/pdf"` (S-2); URL with query params/fragments preserved in href (EC-2, M-2)
- [X] T009 [US1] Write tests for bibliographic citations → citation.text resources in `src/oscal/back_matter.rs`: citation with `url: None` produces resource with `citation.text` matching citation text (M-3); long text (>500 chars) preserved without truncation (EC-3)
- [X] T010 [US1] Write tests for deterministic UUID v5 and title derivation in `src/oscal/back_matter.rs`: same citation content → same UUID across calls (M-4, AC-3); title = citation text when text available (M-5); title = full URL for URL-only citations (M-5 clarification); title prefers text over URL when both present
- [X] T011 [US1] Write tests for malformed/empty/non-http URL handling in `src/oscal/back_matter.rs`: malformed URL (e.g., `"not a url"`) → resource with rlinks preserving URL + `prop name="url-status" value="unvalidated"` (M-8, SEC-3); `url: Some("")` treated as malformed (EC-6, SEC-4); `ftp://`, `mailto:`, `javascript:`, `data:` schemes → unvalidated prop (clarification); resource map still populated for malformed URL citations. **Security note**: `javascript:` and `data:` schemes pose XSS risks if rendered as clickable links — downstream consumers must treat `url-status: "unvalidated"` resources as untrusted.
- [X] T012 [US1] Write tests for edge cases in `src/oscal/back_matter.rs`: zero citations → `Ok((empty vec, empty map))` (EC-1); two identical citations → same UUID in resource map (EC-5); resource includes description when `source_requirement_id` is present (S-1); function returns `Result` — validate error path for citation with empty id

### Implementation for User Story 1 (GREEN phase)

- [X] T013 [US1] Implement `generate_back_matter` function in `src/oscal/back_matter.rs`: iterate citations, classify each (URL parse via `url::Url::parse` + scheme check → valid/malformed/bibliographic), generate UUID v5 using `BACK_MATTER_NAMESPACE` + `normalize_for_hashing` (trims whitespace and collapses internal runs — see `src/uuid.rs::normalize_for_hashing`), build `BackMatterResource` with correct fields per classification, populate `HashMap<String, Uuid>` resource map, emit `tracing::warn!` for malformed URLs
- [X] T014 [US1] Verify all US1 tests pass (GREEN) — run `cargo test --lib oscal::back_matter` and `cargo clippy -- -D warnings`

**Checkpoint**: `generate_back_matter` fully functional — URL, bibliographic, and malformed citations all handled correctly with deterministic UUIDs

---

## Phase 4: User Story 2 — Control Bodies Link to Back Matter Resources (Priority: P1)

**Goal**: Controls that reference citations contain `link` elements with `href="#<resource-uuid>"` and `rel="reference"` pointing to correct back matter resources

**Independent Test**: Generate links for a control referencing 2 citations, verify 2 link elements with correct href values

### Tests for User Story 2 (TDD — RED phase)

- [X] T015 [US2] Write tests for `generate_control_links` in `src/oscal/back_matter.rs`: single citation → one link with `href="#<uuid>"` and `rel="reference"` (M-6, AC-5); two citations → two links with correct hrefs (AC-6); link text populated from citation text when available
- [X] T016 [US2] Write test for orphan reference handling in `src/oscal/back_matter.rs`: citation ID not in resource map → skip link generation, no panic (EC-4); empty citations slice → empty links vec; resource map with extra entries → only requested citation links generated

### Implementation for User Story 2 (GREEN phase)

- [X] T017 [US2] Implement `generate_control_links` function in `src/oscal/back_matter.rs`: for each citation, look up UUID from resource map, create `OscalLink { href: format!("#{uuid}"), rel: "reference".into(), text }`, skip missing citations with `tracing::warn!`
- [X] T018 [US2] Verify all US2 tests pass (GREEN) — run `cargo test --lib oscal::back_matter`

**Checkpoint**: Both `generate_back_matter` and `generate_control_links` work correctly — resource generation and link generation complete

---

## Phase 5: User Story 3 — No Arbitrary Data in Remarks (Priority: P1)

**Goal**: Generated OSCAL output contains zero `remarks` fields — all structured data uses `prop`, `link`, or `resource` patterns per NIST guidance

**Independent Test**: Serialize back matter resources to JSON, verify no `remarks` key appears anywhere in output

### Tests for User Story 3 (TDD — RED/GREEN)

- [X] T019 [US3] Write tests verifying no `remarks` in serialized output in `src/oscal/back_matter.rs`: serialize `BackMatterResource` (URL-based) to JSON → no `"remarks"` key (M-7, SEC-1, SEC-2, AC-7); serialize `BackMatterResource` (bibliographic) to JSON → no `"remarks"` key; serialize `BackMatter` with multiple resources → no `"remarks"` anywhere in JSON string; metadata stored as `prop` annotations, never remarks

### Verification for User Story 3

- [X] T020 [US3] Verify US3 tests pass — structural audit confirming no `remarks` field exists in `BackMatter`, `BackMatterResource`, `ResourceCitation`, `Rlink`, `OscalLink`, or `Prop` structs

**Checkpoint**: All 3 user stories independently verified — back matter generation, control linking, and remarks compliance all passing

---

## Phase 6: Catalog Integration

**Purpose**: Wire back matter types into existing OSCAL catalog structs for end-to-end usage

- [X] T021 Add `back_matter: Option<BackMatter>` field (with `#[serde(rename = "back-matter", skip_serializing_if = "Option::is_none")]`) to `OscalCatalog` in `src/oscal/catalog.rs`
- [X] T022 Add `links: Vec<OscalLink>` field (with `#[serde(skip_serializing_if = "Vec::is_empty")]`) to `OscalControl` in `src/oscal/catalog.rs`
- [X] T023 Write integration test in `src/oscal/back_matter.rs`: create citations → call `generate_back_matter` → call `generate_control_links` → verify link hrefs resolve to resource UUIDs, verify JSON serialization of full `BackMatter` struct produces valid structure, verify zero citations → `None` back matter on catalog
- [X] T024 Fix any existing catalog tests broken by new fields (add default values for `back_matter` and `links` in test helpers)

**Checkpoint**: Existing `cargo test` suite passes with new fields, integration test validates end-to-end flow

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final validation across all user stories

- [X] T025 Run `cargo test` — all tests pass (existing + new)
- [X] T026 Run `cargo clippy -- -D warnings` — zero warnings
- [X] T027 Run `cargo fmt --check` — no formatting issues
- [X] T028 Validate quickstart.md usage pattern compiles: create a test matching the quickstart example code in `specs/012-back-matter/quickstart.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 (`url` dep, error variant, namespace, Citation struct)
- **US1 (Phase 3)**: Depends on Phase 2 (struct definitions must exist for tests to compile)
- **US2 (Phase 4)**: Depends on Phase 3 (needs `generate_back_matter` to produce resource map for link tests)
- **US3 (Phase 5)**: Depends on Phase 3 (needs serializable resources to test JSON output)
- **Catalog Integration (Phase 6)**: Depends on Phases 3 + 4 (needs both functions implemented)
- **Polish (Phase 7)**: Depends on all previous phases

### User Story Dependencies

- **US1 (Phase 3)**: Independent after Phase 2 — the MVP increment
- **US2 (Phase 4)**: Depends on US1 (needs resource map from `generate_back_matter`)
- **US3 (Phase 5)**: Depends on US1 (needs serializable resources); can run in parallel with US2

### Within Each User Story

1. Tests MUST be written and FAIL before implementation (Constitution IV)
2. Implementation makes tests pass (GREEN)
3. Verification checkpoint before moving to next story

### Parallel Opportunities

**Phase 1** — T002, T003, T004 modify different files and can run in parallel:
```
Task: "Add BackMatter error variant in src/error.rs"
Task: "Add BACK_MATTER_NAMESPACE in src/uuid.rs"
Task: "Add Citation struct in src/model/mod.rs"
```

**Phase 5 + Phase 4** — US3 tests (JSON serialization audit) can run in parallel with US2 implementation since they test different concerns on the same resources.

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (4 tasks)
2. Complete Phase 2: Foundational (3 tasks)
3. Complete Phase 3: User Story 1 — citations → back matter resources (7 tasks)
4. **STOP and VALIDATE**: `cargo test --lib oscal::back_matter` passes
5. Back matter resource generation works independently

### Incremental Delivery

1. Setup + Foundational → Types available (Phases 1-2)
2. US1 → Back matter resources generated → Validate (Phase 3, MVP!)
3. US2 → Control links generated → Validate (Phase 4)
4. US3 → No remarks audit → Validate (Phase 5)
5. Catalog Integration → End-to-end wired → Validate (Phase 6)
6. Polish → All checks green (Phase 7)

---

## Requirement Traceability

| Task(s) | Requirement | SEC | AC |
|---------|-------------|-----|-----|
| T008, T013 | M-1, M-2 | — | AC-1 |
| T009, T013 | M-3 | — | AC-2 |
| T010, T013 | M-4, M-5 | — | AC-3, AC-4 |
| T015-T017 | M-6 | — | AC-5, AC-6 |
| T019-T020 | M-7 | SEC-1, SEC-2 | AC-7 |
| T011, T013 | M-8 | SEC-3, SEC-4 | AC-8 |
| T012 | EC-1 thru EC-5 | — | — |
| T008 | S-1, S-2 | — | — |

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- TDD is mandatory: write tests → verify RED → implement → verify GREEN
- Commit after each phase checkpoint
- All back matter tests live in `src/oscal/back_matter.rs` as `#[cfg(test)] mod tests`
- Avoid: `remarks` fields anywhere, UUID v4 for resources, dropping malformed URLs
