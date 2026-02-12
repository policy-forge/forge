# Tasks: OSCAL Catalog Groups and Controls

**Input**: Design documents from `/specs/009-catalog-groups-controls/`
**Prerequisites**: plan.md (required), spec.md (required), data-model.md, contracts/catalog.rs, research.md, quickstart.md
**PRD**: `docs/PRD/009-prd-catalog-groups-controls.md`
**AR**: `docs/AR/009-ar-catalog-groups-controls.md`
**SEC**: `docs/SEC/009-sec-catalog-groups-controls.md`

**Tests**: TDD is mandatory per Constitution Principle IV. Every function follows Red-Green-Refactor.

**Organization**: Tasks grouped by user story. All stories are P1 but have natural dependencies: US1 (groups) → US3 (control IDs) → US2 (controls) → US4 (serialization).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4)
- All source paths relative to repository root

---

## Phase 1: Setup

**Purpose**: Create the catalog module file and wire it into the existing module structure

- [X] T001 Create `src/oscal/catalog.rs` with module skeleton and add `pub mod catalog;` declaration with public re-exports in `src/oscal/mod.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Error handling infrastructure and OSCAL type definitions needed by ALL user stories

**CRITICAL**: No user story work can begin until this phase is complete

- [X] T002 TDD: Write failing test for `ForgeError::CatalogBuild` display message, then add `#[error("Catalog build error: {0}")] CatalogBuild(String)` variant to `ForgeError` in `src/error.rs`
- [X] T003 Define OSCAL struct types (`CatalogEnvelope`, `OscalCatalog`, `OscalMetadata`, `OscalGroup`, `OscalControl`) with `#[derive(Debug, Clone, Serialize)]` and serde rename/skip attributes per contracts/catalog.rs in `src/oscal/catalog.rs`

**Checkpoint**: Error variant and OSCAL types compiled. User story implementation can begin.

---

## Phase 3: User Story 1 — Generate Catalog Groups from Policy Sections (Priority: P1)

**Goal**: Map each `PolicySection` to an `OscalGroup` with slugified group ID and matching title, preserving section order.

**Independent Test**: Provide a `PolicyDocument` with 3 sections; verify output contains 3 groups with correct IDs (`access-control`, `data-protection`, `incident-response`) and titles.

**Traces to**: M-1, M-2, S-1 (group ID collision: T008a), AC-1, EC-1, EC-2, EC-4, SEC-3

### Tests for User Story 1

> **NOTE: Write tests FIRST, ensure they FAIL before implementation (Constitution IV)**

- [X] T004 [P] [US1] TDD: Write tests for `generate_group_id()` — "Access Control Policies" → "access-control-policies", "Data Protection & Privacy" → "data-protection-privacy", "3.1 — Incident Response" → "3-1-incident-response", consecutive hyphens collapsed, leading/trailing hyphens trimmed in `src/oscal/catalog.rs`
- [X] T005 [P] [US1] TDD: Write tests for `generate_group_id()` edge cases — special characters and non-ASCII safely slugified (EC-4, SEC-3), empty title fallback to "group-{index}" in `src/oscal/catalog.rs`

### Implementation for User Story 1

- [X] T006 [US1] Implement `generate_group_id(section_title: &str) -> String` — lowercase, replace non-alphanumeric with hyphens, collapse consecutive hyphens, trim (handles SEC-3, EC-4) in `src/oscal/catalog.rs`
- [X] T007 [US1] TDD: Write tests for group mapping in `build_catalog()` — 3 sections → 3 groups with correct IDs and titles (AC-1), zero sections → empty groups (EC-1), section with zero requirements → group with empty controls (EC-2) in `src/oscal/catalog.rs`
- [X] T008[US1] Implement initial `build_catalog(&PolicyDocument) -> Result<OscalCatalog, ForgeError>` producing `OscalCatalog` with groups mapped from `document.sections`, placeholder UUID and metadata (D-5) in `src/oscal/catalog.rs`
- [X] T008a [US1] TDD: Write tests for group ID collision detection — two sections with titles that slugify identically (e.g., "Data Protection" and "Data Protection!") → first keeps base ID, subsequent get numeric suffix ("data-protection", "data-protection-2"). Implement collision tracking with `HashMap<String, usize>` in `build_catalog()` (S-1) in `src/oscal/catalog.rs`

**Checkpoint**: `build_catalog` produces groups from sections with unique group IDs. Controls are empty placeholders.

---

## Phase 4: User Story 3 — Generate Human-Readable Control IDs from Section Context (Priority: P1)

**Goal**: Control IDs follow deterministic `POL-{ABBR}-{NNN}` pattern derived from section titles with collision resolution.

**Independent Test**: Provide a section titled "Access Control" with 2 requirements; verify IDs are `POL-AC-001` and `POL-AC-002`.

**Traces to**: M-4, M-8, S-3, AC-2, AC-6, EC-3, EC-6, SEC-2

> **Note**: US3 is implemented before US2 because control creation (US2) depends on having the ID generation algorithm (US3) in place.

### Tests for User Story 3

- [X] T009 [P] [US3] TDD: Write tests for `generate_section_abbreviation()` — "Access Control" → "AC", "Incident Response and Recovery" → "IRR", "Data Protection" → "DP", "Physical and Environmental Security" → "PES", stop words filtered ("a", "an", "and", "the", "of", "for", "in", "to"), empty-after-filtering fallback to first 2 chars in `src/oscal/catalog.rs`
- [X] T010 [P] [US3] TDD: Write tests for `generate_control_id()` — `("AC", 0, "POL")` → "POL-AC-001", `("DP", 4, "POL")` → "POL-DP-005", zero-padded to 3 digits, >999 extends naturally `("AC", 999, "POL")` → "POL-AC-1000" (EC-6) in `src/oscal/catalog.rs`

### Implementation for User Story 3

- [X] T011 [US3] Implement `generate_section_abbreviation(section_title: &str) -> String` — split words, remove stop words, take first char uppercase, fallback to first 2 chars in `src/oscal/catalog.rs`
- [X] T012 [US3] Implement `generate_control_id(abbreviation: &str, requirement_index: usize, prefix: &str) -> String` — format `{prefix}-{abbr}-{NNN}` with 0-based→1-based, min 3-digit zero-pad in `src/oscal/catalog.rs`
- [X] T013 [US3] TDD: Write tests for abbreviation collision resolution — "Access Control" and "Application Configuration" both yield "AC", first keeps "AC", second gets "AC2", third collision gets "AC3" (EC-3, S-3) in `src/oscal/catalog.rs`
- [X] T014 [US3] Implement abbreviation collision tracking with `HashMap<String, usize>` and numeric suffix resolution in `build_catalog()`, verify all control IDs globally unique (M-8, SEC-2) in `src/oscal/catalog.rs`

**Checkpoint**: Control ID generation is deterministic, collision-safe, and globally unique.

---

## Phase 5: User Story 2 — Generate Catalog Controls from Policy Requirements (Priority: P1)

**Goal**: Map each `PolicyRequirement` to an `OscalControl` with UUID from `stable_id`, derived title, and generated control ID within its parent group.

**Independent Test**: Provide 2 sections with 3 and 4 requirements; verify 7 controls distributed across 2 groups with correct IDs.

**Traces to**: M-3, M-5, M-6, S-2, AC-2, AC-3, AC-4, EC-5, EC-7, SEC-1, D-3

### Tests for User Story 2

- [X] T015 [P] [US2] TDD: Write tests for `derive_control_title()` — "Systems shall require MFA. Additional info." → "Systems shall require MFA.", no punctuation → full text, >120 chars → truncated to 120 + "..." (EC-7), exclamation/question marks as sentence-enders in `src/oscal/catalog.rs`
- [X] T016 [P] [US2] TDD: Write tests for `collect_requirements()` — flat section returns own requirements, nested children returns parent requirements first then children depth-first, order preserved (D-3, S-2) in `src/oscal/catalog.rs`
- [X] T017 [P] [US2] TDD: Write test for missing `stable_id` — `PolicyRequirement` with `stable_id: None` causes `build_catalog()` to return `ForgeError::CatalogBuild` with descriptive message (EC-5, SEC-1, M-6) in `src/oscal/catalog.rs`

### Implementation for User Story 2

- [X] T018 [US2] Implement `derive_control_title(requirement_text: &str) -> String` — find first `.`/`!`/`?`, extract first sentence, trim, truncate to 120 chars + "..." if needed in `src/oscal/catalog.rs`
- [X] T019 [US2] Implement `collect_requirements(section: &PolicySection) -> Vec<&PolicyRequirement>` — recursively collect from section and children depth-first in `src/oscal/catalog.rs`
- [X] T020 [US2] Complete `build_catalog()` control mapping: validate `stable_id` is `Some` (SEC-1), copy to `uuid` (M-6), derive title (M-5), generate control ID (M-4), preserve requirement ordering (M-3) in `src/oscal/catalog.rs`
- [X] T021 [US2] TDD: Write test verifying `build_catalog()` produces correct controls — 2 sections with 3+4 requirements → 7 controls across 2 groups, correct IDs, UUIDs, and titles (AC-2, AC-3, AC-4) in `src/oscal/catalog.rs`

**Checkpoint**: `build_catalog` produces complete groups with controls. All mapping logic in place.

---

## Phase 6: User Story 4 — Serialize Catalog to JSON (Priority: P1)

**Goal**: Serialize the assembled `OscalCatalog` to valid OSCAL-compliant JSON with `{"catalog": {...}}` envelope.

**Independent Test**: Build a complete Catalog, serialize, verify valid JSON with correct field names and round-trip integrity.

**Traces to**: M-7, AC-5, D-5, D-6, SC-004

### Tests for User Story 4

- [X] T022 [P] [US4] TDD: Write tests for JSON serialization — `CatalogEnvelope` produces `{"catalog": {...}}` root key (D-6), correct field names (`groups`, `controls`, `id`, `title`, `uuid`, `last-modified`, `oscal-version`), placeholder metadata values (D-5), empty groups omitted via `skip_serializing_if` (AC-5) in `src/oscal/catalog.rs`
- [X] T023 [P] [US4] TDD: Write round-trip test — serialize `CatalogEnvelope` to JSON string, parse back to `serde_json::Value`, verify field structure matches expected OSCAL schema in `src/oscal/catalog.rs`

### Implementation for User Story 4

- [X] T024 [US4] Verify `CatalogEnvelope` serialization produces valid OSCAL JSON via `serde_json::to_string_pretty()` — no code changes expected if structs from T003 are correct; fix serde attributes if any field name mismatches in `src/oscal/catalog.rs`
- [X] T025 [US4] Add `tracing` DEBUG-level logging in `build_catalog()` for group count, control count, and abbreviation collision events per Constitution Principle IX in `src/oscal/catalog.rs`

**Checkpoint**: Full pipeline produces valid OSCAL Catalog JSON. All user stories functional.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Integration testing, validation, and quality gates

- [X] T026 [P] Write end-to-end integration test with realistic multi-section `PolicyDocument` (5 sections, 20 requirements) validating complete Catalog output — all sections mapped (SC-001), all requirements mapped (SC-002), zero duplicate IDs (SC-003), deterministic output (SC-006) in `src/oscal/catalog.rs`
- [X] T027 [P] Write test verifying global control ID uniqueness with abbreviation collisions across 5+ sections (AC-6, M-8, SC-003, SEC-2) in `src/oscal/catalog.rs`
- [X] T028 Run `cargo clippy -- -D warnings` and `cargo fmt --check` — zero warnings, zero formatting violations
- [X] T029 Validate `specs/009-catalog-groups-controls/quickstart.md` code examples compile and match actual public API in `src/oscal/catalog.rs`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 — BLOCKS all user stories
- **US1 Groups (Phase 3)**: Depends on Phase 2 — first user story
- **US3 Control IDs (Phase 4)**: Depends on Phase 2 — can run in parallel with US1 for helper functions, but `build_catalog()` integration depends on Phase 3
- **US2 Controls (Phase 5)**: Depends on Phase 3 (groups exist) and Phase 4 (control IDs exist)
- **US4 Serialization (Phase 6)**: Depends on Phase 5 (complete Catalog structure)
- **Polish (Phase 7)**: Depends on all user stories complete

### User Story Dependencies

```
Phase 1: Setup
    ↓
Phase 2: Foundational (error variant + OSCAL structs)
    ↓
Phase 3: US1 - Groups (generate_group_id + build_catalog skeleton)
    ↓                  ↘
Phase 4: US3 - IDs     (generate_section_abbreviation + generate_control_id + collision resolution)
    ↓                  ↙
Phase 5: US2 - Controls (derive_control_title + collect_requirements + build_catalog completion)
    ↓
Phase 6: US4 - JSON (CatalogEnvelope serialization + tracing)
    ↓
Phase 7: Polish (integration tests + quality gates)
```

### Within Each User Story

1. Tests written FIRST (TDD Red phase)
2. Implementation makes tests pass (TDD Green phase)
3. Refactor if needed
4. Helper functions before orchestrator (`build_catalog`)

### Parallel Opportunities

**Phase 3 (US1)**: T004 and T005 can run in parallel (different test groups)
**Phase 4 (US3)**: T009 and T010 can run in parallel (different functions: abbreviation vs. control ID)
**Phase 5 (US2)**: T015, T016, and T017 can run in parallel (different functions: title, collection, validation)
**Phase 6 (US4)**: T022 and T023 can run in parallel (different test aspects)
**Phase 7**: T026 and T027 can run in parallel (different integration test scenarios)

---

## Parallel Example: User Story 3 (Control IDs)

```
# Launch test-writing tasks in parallel:
Task T009: "TDD tests for generate_section_abbreviation() in src/oscal/catalog.rs"
Task T010: "TDD tests for generate_control_id() in src/oscal/catalog.rs"

# After tests written, implement sequentially:
Task T011: "Implement generate_section_abbreviation() in src/oscal/catalog.rs"
Task T012: "Implement generate_control_id() in src/oscal/catalog.rs"

# Then collision resolution:
Task T013: "TDD tests for abbreviation collision resolution in src/oscal/catalog.rs"
Task T014: "Implement collision tracking in build_catalog() in src/oscal/catalog.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (module file)
2. Complete Phase 2: Foundational (error variant + OSCAL structs)
3. Complete Phase 3: US1 — Groups
4. **STOP and VALIDATE**: `build_catalog` produces groups from sections with correct IDs

### Incremental Delivery

1. Setup + Foundational → Types and error handling ready
2. US1 (Groups) → Catalog has groups from sections → Validate independently
3. US3 (Control IDs) → Deterministic ID generation with collision safety → Validate independently
4. US2 (Controls) → Full control mapping with UUID, title, ID → Validate independently
5. US4 (Serialization) → Complete OSCAL JSON output → Validate end-to-end
6. Polish → Integration tests + quality gates → Ready for merge

### Key Constraint Reminders (from AR Implementation Guardrails)

- **DO NOT** add statement parts (`parts[]`) to controls — deferred to WI-10
- **DO NOT** populate real metadata — deferred to WI-11
- **DO NOT** add back matter or links — deferred to WI-12
- **DO NOT** use `serde_json::Value` — use typed structs
- **DO NOT** mutate the `PolicyDocument` — read-only transformation
- **DO NOT** generate UUIDs — use `stable_id` from WI-7
- **MUST** generate unique control IDs following `POL-{ABBR}-{NNN}` pattern
- **MUST** preserve section and requirement ordering
- **MUST** serialize with `serde_json` producing valid JSON

---

## Notes

- All code goes in `src/oscal/catalog.rs` (single file, per established module-per-feature pattern)
- Tests use `#[cfg(test)] mod tests` block inside `src/oscal/catalog.rs` (per project convention)
- No new dependencies — `serde`, `serde_json`, `tracing`, `thiserror` already in `Cargo.toml`
- Domain model is read-only: `&PolicyDocument`, `&PolicySection`, `&PolicyRequirement`
- `src/oscal/mod.rs` currently empty — needs `pub mod catalog;` and re-exports
- `ForgeError` in `src/error.rs` currently has no `CatalogBuild` variant — added in Phase 2
