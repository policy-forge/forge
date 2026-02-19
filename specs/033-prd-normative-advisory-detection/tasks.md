# Tasks: Normative/Advisory Detection (WI-33)

**Input**: Design documents from `/specs/033-prd-normative-advisory-detection/`
**Prerequisites**: plan.md ✅, research.md ✅, data-model.md ✅, contracts/rust-interface.md ✅, quickstart.md ✅
**Tests**: TDD is mandatory per PRD and project constitution — test tasks MUST be written first and verified FAILING before implementation begins.
**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no inter-task dependencies)
- **[Story]**: User story label (US1, US2, US3)
- Exact file paths included in all task descriptions

---

## Phase 1: Setup

**Purpose**: Create the new module file and register it — enables all subsequent phases to compile.

- [X] T001 [P] Create `src/parse/modality.rs` skeleton with a placeholder module comment (empty file; no types yet)
- [X] T002 Add `pub mod modality;` and `pub use modality::annotate_modalities;` to `src/parse/mod.rs`
- [X] T003 [P] Create mixed-modality fixture `tests/fixtures/033-mixed-modality.md` with must/shall/should/may/no-verb requirements (per quickstart.md input example)

---

## Phase 2: Foundational (Domain Model — Blocking Prerequisite)

**Purpose**: Extend the domain model — required before any user story can compile or be tested.

**⚠️ CRITICAL**: No user story implementation can begin until this phase is complete — `Modality` and `PolicyRequirement.modality` must exist first.

- [X] T004 Add `Modality` enum to `src/model/mod.rs` with `#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]` and `#[serde(rename_all = "lowercase")]` (`Normative`, `Advisory` variants per data-model.md)
- [X] T005 Extend `PolicyRequirement` struct in `src/model/mod.rs` with `pub modality: Option<Modality>` field (after the `citations` field per data-model.md)
- [X] T006 Update all existing `PolicyRequirement { ... }` struct literals across test code to add `modality: None` (run `cargo build` to find every compilation error and fix each one)
- [X] T007 Add `pub use model::Modality;` re-export to `src/lib.rs`

**Checkpoint**: `cargo build` passes — foundation ready, user story phases can begin

---

## Phase 3: User Story 1 — Core Modality Detection (Priority: P1) 🎯 MVP

**Goal**: Implement the full detection pass (`detect_modality` + `annotate_modalities`), wire it into the pipeline as Step 7c, and emit modality props in both OSCAL output builders.

**Independent Test**: `cargo test -- parse::modality` passes. Given `tests/fixtures/033-mixed-modality.md`, "must"/"shall" requirements are tagged `normative` and "should"/"may" requirements are tagged `advisory` in catalog JSON output.

### Tests for User Story 1 ⚠️ Write FIRST — Must FAIL Before Implementation

- [X] T008 [P] [US1] Write unit test `detect_modality_normative_verbs`: each of "must", "shall", "will", "required" → `Modality::Normative`, non-empty `matched_verbs` in `src/parse/modality.rs` `#[cfg(test)]` block (AC-1)
- [X] T009 [P] [US1] Write unit test `detect_modality_advisory_verbs`: each of "should", "may", "recommended", "optional" → `Modality::Advisory`, non-empty `matched_verbs` in `src/parse/modality.rs` `#[cfg(test)]` block (AC-2)
- [X] T010 [P] [US1] Write unit test `detect_modality_case_insensitive`: "MUST", "SHALL", "Should", "MAY" all classified correctly in `src/parse/modality.rs` `#[cfg(test)]` block (AC-7, M-6)
- [X] T011 [P] [US1] Write unit test `detect_modality_word_boundary`: "customize", "dismay", "shouldering", "willful" produce no verb matches in `src/parse/modality.rs` `#[cfg(test)]` block (SEC-2)
- [X] T012 [P] [US1] Write unit test `annotate_modalities_all_requirements_populated`: doc with 3 mixed requirements returns all with `modality: Some(_)` in `src/parse/modality.rs` `#[cfg(test)]` block
- [X] T036 [P] [US1] Write unit test `detect_modality_negated_normative_verbs`: "Organizations must not share passwords" → `Modality::Normative`, non-empty `matched_verbs` containing "must" in `src/parse/modality.rs` `#[cfg(test)]` block (EC-1; negated obligations remain normative)
- [X] T037 [P] [US1] Write unit test `detect_modality_required_as_adjective`: "the required configuration" → `Modality::Normative`, non-empty `matched_verbs` in `src/parse/modality.rs` `#[cfg(test)]` block (EC-6; accepted conservative behavior per PRD)

### Implementation for User Story 1

- [X] T013 [US1] Define `ModalityResult` struct in `src/parse/modality.rs` with fields: `modality: Modality`, `matched_verbs: Vec<String>`, `is_default: bool`, `has_conflict: bool` (per contracts/rust-interface.md)
- [X] T014 [US1] Define `static NORMATIVE_PATTERN: LazyLock<Regex>` and `static ADVISORY_PATTERN: LazyLock<Regex>` in `src/parse/modality.rs` using patterns `(?i)\b(must|shall|will|required)\b` and `(?i)\b(should|may|recommended|optional)\b` (SEC-3, SEC-4)
- [X] T015 [US1] Implement `detect_modality(requirement: &PolicyRequirement) -> ModalityResult` in `src/parse/modality.rs` covering normative-only and advisory-only classification cases (AC-1, AC-2, AC-7; edge cases added in Phase 5)
- [X] T016 [US1] Implement `annotate_modalities(document: PolicyDocument) -> Result<PolicyDocument, ForgeError>` in `src/parse/modality.rs` iterating all sections/requirements and emitting `tracing::debug!` for matched verbs per requirement (SEC-1)
- [X] T017 [US1] Add Step 7c `annotate_modalities` call in `prepare_document` in `src/pipeline.rs` immediately after the citation extraction step (Step 7b), propagating `?` on error
- [X] T018 [P] [US1] Add modality prop (`OscalProp { name: "modality".to_string(), ns: None, value: "normative"|"advisory", class: None }`) to `OscalControl.props` in `src/oscal/catalog.rs` when `requirement.modality` is `Some(_)` (AC-3, AC-4, M-3)
- [X] T019 [P] [US1] Add modality prop to implemented-requirement props in `src/oscal/implemented_requirements.rs` using the same `OscalProp` pattern as T018 (M-5)

**Checkpoint**: `cargo test -- parse::modality` passes — normative/advisory classification works, modality props appear in OSCAL output

---

## Phase 4: User Story 2 — Modality Visible in Output (Priority: P2)

**Goal**: Integration tests confirm modality props are present across the full OSCAL pipeline (catalog and component definition) for a mixed-modality input document.

**Independent Test**: `cargo test -- integration` passes. JSON output from `033-mixed-modality.md` contains `{"name": "modality", "value": "normative"}` or `{"name": "modality", "value": "advisory"}` prop on each control.

### Tests for User Story 2 ⚠️ Write FIRST — Must FAIL Before Implementation

- [X] T020 [P] [US2] Write integration test: load `tests/fixtures/033-mixed-modality.md`, run full pipeline, assert every control in catalog JSON output has a `props` entry with `"name": "modality"` (AC-6)
- [X] T021 [P] [US2] Write integration test: verify modality prop appears on implemented-requirements in component definition output from the same fixture (M-5, AC-10)

### Implementation for User Story 2

- [X] T022 [US2] Run `cargo insta review` to accept and update insta snapshot test expected outputs to include modality props in OSCAL JSON (AC-10) in `tests/snapshots/`
- [X] T023 [US2] Run `cargo test --workspace` and confirm integration tests T020/T021 pass with modality props present in both output paths

**Checkpoint**: Integration tests pass — modality is visible in full OSCAL output for Catalog and Component Definition

---

## Phase 5: User Story 3 — Edge Cases + Warning Behavior (Priority: P2)

**Goal**: `detect_modality` handles requirements with no verb (defaults to normative) and conflicting verbs (normative wins), with `tracing::warn!` emitted for both cases. Every requirement guaranteed to have `Some(modality)` after the enrichment pass.

**Independent Test**: Given `PolicyRequirement { text: "Encrypt all data at rest", ... }`, `detect_modality` returns `ModalityResult { modality: Normative, is_default: true, matched_verbs: [] }`. Warning log emitted by `annotate_modalities` for this requirement.

### Tests for User Story 3 ⚠️ Write FIRST — Must FAIL Before Implementation

- [X] T024 [P] [US3] Write unit test `detect_modality_no_verb_defaults_normative`: "Encrypt all data at rest" → `Modality::Normative`, `is_default: true`, `matched_verbs: []` in `src/parse/modality.rs` `#[cfg(test)]` block (AC-8, S-1)
- [X] T025 [P] [US3] Write unit test `detect_modality_conflict_normative_wins`: "Systems must implement and should log" → `Modality::Normative`, `has_conflict: true`, both verbs in `matched_verbs` in `src/parse/modality.rs` `#[cfg(test)]` block (AC-9, S-2)
- [X] T026 [US3] Write unit test `invariant_is_default_and_has_conflict_mutually_exclusive`: for all test inputs, assert `!(result.is_default && result.has_conflict)` in `src/parse/modality.rs` `#[cfg(test)]` block (contracts/rust-interface.md Invariant 2)

### Implementation for User Story 3

- [X] T027 [US3] Extend `detect_modality` in `src/parse/modality.rs` to handle the default case: when neither pattern matches, return `Modality::Normative` with `is_default: true` and empty `matched_verbs` (AC-8, S-1)
- [X] T028 [US3] Extend `detect_modality` in `src/parse/modality.rs` to handle the conflict case: when both patterns match, return `Modality::Normative` with `has_conflict: true` and combined `matched_verbs` from both patterns (AC-9, S-2)
- [X] T029 [US3] Add `tracing::warn!` calls in `annotate_modalities` in `src/parse/modality.rs` for requirements where `result.is_default` is `true` and where `result.has_conflict` is `true`, including the requirement source text at DEBUG level per SEC-1

**Checkpoint**: All edge cases handled — every `PolicyRequirement` receives `Some(modality)` after `annotate_modalities`, warnings emitted for defaults/conflicts

---

## Phase 6: Polish & Verification

**Purpose**: Quality gates and final validation per the Definition of Done in plan.md.

- [X] T030 [P] Run `cargo test --workspace` — all tests pass (Definition of Done)
- [X] T038 [P] Run `cargo nextest run --workspace` — all tests pass (Definition of Done)
- [X] T031 [P] Run `cargo clippy -- -D warnings` — zero clippy warnings (Definition of Done)
- [X] T032 [P] Run `cargo fmt --check` — zero formatting violations (Definition of Done)
- [X] T033 Run `cargo tarpaulin --lib -- parse::modality` — verify ≥90% line coverage for `src/parse/modality.rs` (AR Testing Strategy)
- [X] T034 [P] Run `cargo audit` — no new security advisories introduced (no new dependencies, should pass cleanly) (Definition of Done)
- [X] T035 Validate quickstart.md scenario manually: build and run FORGE against `tests/fixtures/033-mixed-modality.md`, verify JSON output has modality props on each control and WARN logs appear on stderr for the no-verb requirement

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 completion — BLOCKS all user story phases
- **US1 (Phase 3)**: Depends on Phase 2 completion — core MVP functionality
- **US2 (Phase 4)**: Depends on Phase 3 completion — integration tests require modality props in output
- **US3 (Phase 5)**: Depends on Phase 3 completion — extends the `detect_modality` function built in US1
- **Polish (Phase 6)**: Depends on all phases (3, 4, 5) complete

### User Story Dependencies

- **US1 (P1)**: Starts after Foundational (Phase 2) — no dependency on US2 or US3
- **US2 (P2)**: Depends on US1 (needs modality props in output pipeline)
- **US3 (P2)**: Depends on US1 (extends `detect_modality` and `annotate_modalities` from Phase 3)
- US2 and US3 are independent of each other — can proceed in parallel once US1 is complete

### Within Each User Story

- Test tasks MUST be written and FAIL before implementation tasks begin (TDD mandatory)
- T013 (ModalityResult struct) before T015 (detect_modality uses it)
- T014 (patterns) before T015 (detect_modality uses the statics)
- T015 (detect_modality) before T016 (annotate_modalities calls it)
- T016 (annotate_modalities) before T017 (pipeline integration)
- T017 (pipeline) before T018/T019 (OSCAL builders consume annotated requirements)
- T027/T028 (extend detect_modality) before T029 (warnings in annotate_modalities)

### Parallel Opportunities

- Phase 1: T001 and T003 can run in parallel (different files)
- Phase 3 tests: T008, T009, T010, T011, T012, T036, T037 — all parallel (different test functions in same file)
- Phase 3 impl: T018 and T019 — parallel (different OSCAL output files)
- Phase 4 tests: T020 and T021 — parallel (different integration test scenarios)
- Phase 5 tests: T024 and T025 — parallel (different test functions)
- Phase 6: T030, T031, T032, T034, T038 — all parallel (independent verification commands)

---

## Parallel Example: Phase 3 Tests (US1)

```bash
# Launch all US1 test tasks together — write them all, then verify ALL FAIL:
Task: "T008 - detect_modality_normative_verbs test in src/parse/modality.rs"
Task: "T009 - detect_modality_advisory_verbs test in src/parse/modality.rs"
Task: "T010 - detect_modality_case_insensitive test in src/parse/modality.rs"
Task: "T011 - detect_modality_word_boundary test in src/parse/modality.rs"
Task: "T012 - annotate_modalities_all_requirements_populated test in src/parse/modality.rs"
Task: "T036 - detect_modality_negated_normative_verbs test in src/parse/modality.rs"
Task: "T037 - detect_modality_required_as_adjective test in src/parse/modality.rs"

# After all 5 tests FAIL — proceed to implementation in order:
# T013 → T014 → T015 → T016 → T017

# Then launch T018 and T019 in parallel:
Task: "T018 - modality prop in src/oscal/catalog.rs"
Task: "T019 - modality prop in src/oscal/implemented_requirements.rs"
```

---

## Implementation Strategy

### MVP First (US1 Only)

1. Complete Phase 1: Setup (T001–T003)
2. Complete Phase 2: Foundational (T004–T007) — CRITICAL: blocks everything
3. Complete Phase 3: US1 (T008–T019)
4. **STOP and VALIDATE**: `cargo test -- parse::modality` passes, modality props in OSCAL JSON
5. Run Phase 6 verification against US1 deliverable

### Full Feature Delivery

1. Setup + Foundational → `cargo build` passes (Foundation ready)
2. US1 → Core detection + OSCAL props → `cargo test -- parse::modality` passes (MVP!)
3. US2 → Integration tests + snapshot updates → Full pipeline verified
4. US3 → Edge cases + warnings → Every requirement guaranteed annotated
5. Polish → All quality gates pass → Ready for merge

---

## Acceptance Criteria Coverage

| AC ID | Req | Story | Tasks |
|-------|-----|-------|-------|
| AC-1 | M-1 | US1 | T008, T015 |
| AC-2 | M-2 | US1 | T009, T015 |
| AC-3 | M-3 | US1 | T018 |
| AC-4 | M-3 | US1 | T018 |
| AC-5 | M-4 | US1 | T004, T005 |
| AC-6 | M-5 | US2 | T018, T019, T020, T022 |
| AC-7 | M-6 | US1 | T010, T015 |
| AC-8 | S-1 | US3 | T024, T027 |
| AC-9 | S-2 | US3 | T025, T028 |
| AC-10 | S-3 | US2 | T018, T019, T021, T022 |

## Edge Case Coverage

| EC ID | PRD Edge Case | Tasks |
|-------|---------------|-------|
| EC-1 | "must not"/"shall not" still normative | T036 |
| EC-2 | "MUST" all-caps → normative (case insensitive) | T010 |
| EC-3 | False positive on non-obligation "must" — accepted limitation | N/A (documented limitation) |
| EC-4 | Imperative without RFC 2119 keyword defaults to normative | T024 |
| EC-5 | "may" as month name — word boundary prevents false match | T011 |
| EC-6 | "required" as adjective still classifies normative | T037 |
| EC-7 | Pre-WI-33 documents get `modality: None` (handled by Option<Modality>) | T005, T006 |

## Security Requirements Coverage

| SEC ID | Requirement | Tasks |
|--------|-------------|-------|
| SEC-1 | Requirement text logged at DEBUG only | T016, T029 |
| SEC-2 | Word boundary anchors (`\b`) | T011, T014 |
| SEC-3 | No nested quantifiers (ReDoS prevention) | T014 |
| SEC-4 | Compile once via `LazyLock` | T014 |

---

## Notes

- [P] tasks = different files or independent test functions — run in parallel with no risk of conflicts
- [Story] label traces each task to its user story for independent delivery and testing
- TDD is mandatory: write tests, confirm FAIL, then implement — never write implementation first
- `cargo build` should pass after Phase 2 even before Phase 3 implementation exists (skeleton module + domain model)
- Insta snapshot updates (T022) require running `cargo insta review` interactively — not a pure code edit
- `is_default` and `has_conflict` are mutually exclusive per Invariant 2 in `contracts/rust-interface.md`
- The `will` verb match is intentional despite potential false positives (e.g., "will be reviewed") — accepted per research.md Decision 2
- "May" as a month name is a known limitation with low false-positive risk in atomized requirement text — accepted per EC-5 in PRD
