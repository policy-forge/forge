# Tasks: WI-34 Parameter Extraction

**Input**: Design documents from `specs/034-prd-parameter-extraction/`
**Prerequisites**: plan.md, research.md, data-model.md, contracts/parameter_api.rs, contracts/parameter_types.rs, quickstart.md
**Source**: PRD `docs/PRD/034-prd-parameter-extraction.md`, AR `docs/AR/034-ar-parameter-extraction.md`, SEC `docs/SEC/034-sec-parameter-extraction.md`

**Tests**: TDD is mandatory per Constitution principle IV and PRD technical constraints. Test tasks are included in every user story phase. Each test task MUST be completed and confirmed failing (RED) before the corresponding implementation task begins.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1–US5)
- Exact file paths included in all descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Verify baseline, create module skeleton. The project already builds; setup is lightweight.

- [X] T001 Verify clean build and all tests pass on feature branch: `cargo build && cargo test` (baseline before any changes)
- [X] T002 Create `src/parameter/mod.rs` and `src/parameter/matchers.rs` with minimal stubs (empty `pub mod matchers;` + `todo!()` placeholders) so module compiles
- [X] T003 Add `pub mod parameter;` to `src/lib.rs` (depends on T002)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Domain types and error infrastructure that ALL user stories depend on. No user story work can begin until this phase is complete.

**⚠️ CRITICAL**: Phases 3–7 are all blocked until T004–T007 complete.

- [X] T004 Add `PolicyParameter`, `ParameterType`, `ParameterConstraint`, and `ConstraintType` types (with `#[derive(Debug, Clone, PartialEq, Serialize)]`) to `src/model/mod.rs` per `contracts/parameter_types.rs`
- [X] T005 Add `parameters: Vec<PolicyParameter>` field to `PolicyRequirement` struct in `src/model/mod.rs` (depends on T004; follow the `citations` field pattern)
- [X] T006 Fix all existing tests that construct `PolicyRequirement` by adding `parameters: vec![]` to every struct literal — search across `src/model/mod.rs`, `src/pipeline.rs` tests, and any integration tests; run `cargo test` to confirm all pass
- [X] T007 [P] Add `ForgeError::ParameterExtraction(String)` variant with `#[error("Parameter extraction error: {0}")]` to `src/error.rs`; add `ForgeError::ParameterExtraction(_) => 2` to `exit_code()` match arm (parallel with T004–T006 since different file)

**Checkpoint**: Run `cargo test` — all existing tests pass, new types compile. Foundation ready.

---

## Phase 3: User Story 1 — Extract Time Window Parameters (Priority: P1) 🎯 MVP

**Goal**: Detect "within N days", "after N weeks", "every N months/years" patterns in requirement text and extract them as `PolicyParameter` objects with time window type and appropriate constraint (Minimum for "within"/"after", Exact for "every").

**Independent Test**: `cargo test parameter::matchers::tests::time_window` — pass a string containing "within 30 days" to `TimeWindowMatcher.find_parameters()` and assert a `ParameterMatch` is returned with `value = "30 days"`, `parameter_type = TimeWindow`, `constraint = Some(ParameterConstraint { constraint_type: Minimum, value: "30 days" })`.

### Tests for User Story 1

> **Write tests FIRST. Confirm they FAIL before implementing. (RED)**

- [X] T008 [US1] Write failing tests for `TimeWindowMatcher.find_parameters()` in `src/parameter/matchers.rs` — cover: "within 30 days", "after 6 weeks", "every 90 days", "within 2 years", multi-unit variants; assert `parameter_type`, `value`, `constraint_type`, `label`, `start`/`end` offsets
- [X] T009 [P] [US1] Write failing negative-fixture tests for `TimeWindowMatcher` (SEC-2): assert "Section 3.2" and "NIST SP 800-53" do NOT produce matches; assert bare numbers without qualifier words are not extracted (parallel with T008, same file but different test functions)

### Implementation for User Story 1

- [X] T010 [US1] Implement `ParameterMatch` struct and `ParameterMatcher` trait in `src/parameter/matchers.rs` per `contracts/parameter_api.rs` (depends on T008 failing)
- [X] T011 [US1] Implement `TimeWindowMatcher` struct with `static TIME_WINDOW: LazyLock<Regex>` using bounded pattern `(?i)(?P<qualifier>within|after|every)\s+(?P<value>\d+)\s+(?P<unit>days?|weeks?|months?|years?)` in `src/parameter/matchers.rs`; map "every" → `Exact`, "within"/"after" → `Minimum`; label = `"{qualifier} {value} {unit}"` (SEC-3: bounded quantifiers only; SEC-4: `LazyLock` for one-time compilation)
- [X] T012 [US1] Run `cargo test parameter::matchers::tests::time_window` — confirm GREEN; run negative fixture tests — confirm no false positives

**Checkpoint**: `cargo test parameter::matchers::tests` — `TimeWindowMatcher` tests pass, negative fixtures pass.

---

## Phase 4: User Story 2 — Extract Threshold Parameters (Priority: P1)

**Goal**: Detect "at least N", "minimum N", "no fewer than N", "no less than N" (minimum constraint) and "no more than N", "maximum N", "at most N" (maximum constraint) in requirement text.

**Independent Test**: `cargo test parameter::matchers::tests::threshold` — pass "at least 128-bit" to `ThresholdMatcher.find_parameters()` and assert `value = "128-bit"`, `constraint_type = Minimum`; pass "no more than 15 minutes" and assert `constraint_type = Maximum`.

### Tests for User Story 2

> **Write tests FIRST. Confirm they FAIL before implementing. (RED)**

- [X] T013 [US2] Write failing tests for `ThresholdMatcher.find_parameters()` in `src/parameter/matchers.rs` — cover: minimum patterns ("at least 128-bit", "minimum 12 characters", "no fewer than N", "no less than N") and maximum patterns ("no more than 15 minutes", "maximum 5", "at most 3"); verify `constraint_type`, `value`, `label`

### Implementation for User Story 2

- [X] T014 [US2] Implement `ThresholdMatcher` struct with two `LazyLock<Regex>` patterns (minimum: `(?i)(?P<qualifier>at\s+least|minimum|no\s+fewer\s+than|no\s+less\s+than)\s+(?P<value>\d+[\w-]*)` and maximum: `(?i)(?P<qualifier>no\s+more\s+than|maximum|at\s+most)\s+(?P<value>\d+[\w-]*)`) in `src/parameter/matchers.rs`; map to `Threshold` type and `Minimum`/`Maximum` constraints respectively; label = `"{qualifier} {value}"` (e.g., `"at least 128-bit"`, `"no more than 15 minutes"`) (SEC-3, SEC-4)
- [X] T015 [US2] Run `cargo test parameter::matchers::tests::threshold` — confirm GREEN; verify `cargo test` — all prior tests still pass

**Checkpoint**: All time window and threshold matcher tests pass.

---

## Phase 5: User Story 3 — Extract Frequency Parameters (Priority: P1)

**Goal**: Detect "at least annually", "quarterly", "monthly", "weekly", "daily", "biannually", "semi-annually" and extract with frequency type. Bare frequency words (without "at least") → `Exact` constraint; "at least {word}" → `Minimum` constraint.

**Independent Test**: `cargo test parameter::matchers::tests::frequency` — "at least annually" → `value = "annually"`, `constraint_type = Minimum`; "quarterly" → `value = "quarterly"`, `constraint_type = Exact`.

### Tests for User Story 3

> **Write tests FIRST. Confirm they FAIL before implementing. (RED)**

- [X] T016 [US3] Write failing tests for `FrequencyMatcher.find_parameters()` in `src/parameter/matchers.rs` — cover: qualified ("at least annually", "at least monthly") → `Minimum`; bare words ("quarterly", "weekly", "daily", "biannually", "semi-annually") → `Exact`; verify `parameter_type = Frequency`, `value`, `label`

### Implementation for User Story 3

- [X] T017 [US3] Implement `FrequencyMatcher` struct with `static FREQUENCY: LazyLock<Regex>` using pattern `(?i)(?:at\s+least\s+)?(?P<value>annually|quarterly|monthly|weekly|daily|biannually|semi-annually)` in `src/parameter/matchers.rs`; presence of "at least" prefix → `Minimum`, absent → `Exact`; label = `"at least {value}"` when prefix present, `"{value}"` when bare (e.g., `"at least annually"`, `"quarterly"`) (SEC-3, SEC-4)
- [X] T018 [US3] Run `cargo test parameter::matchers::tests::frequency` — confirm GREEN; verify all prior tests pass

**Checkpoint**: Time window, threshold, and frequency matcher tests all pass. P1 matchers complete.

---

## Phase 6: User Story 4 — Extract Quantity Parameters (Priority: P2)

**Goal**: Detect "no fewer than 3 factors", "at least 2 generations", "minimum 3 authentication factors" (qualifier + digit + countable unit noun) and extract as `Quantity` type with `Minimum` constraint.

**Independent Test**: `cargo test parameter::matchers::tests::quantity` — "no fewer than 3 authentication factors" → `value = "3"`, `parameter_type = Quantity`, `constraint_type = Minimum`.

### Tests for User Story 4

> **Write tests FIRST. Confirm they FAIL before implementing. (RED)**

- [X] T019 [US4] Write failing tests for `QuantityMatcher.find_parameters()` in `src/parameter/matchers.rs` — cover: "no fewer than 3 factors", "at least 2 generations", "minimum 8 password characters"; verify `parameter_type = Quantity`, `constraint_type = Minimum`, `value`, `label`

### Implementation for User Story 4

- [X] T020 [US4] Implement `QuantityMatcher` struct with `static QUANTITY: LazyLock<Regex>` using pattern `(?i)(?P<qualifier>at\s+least|no\s+fewer\s+than|minimum)\s+(?P<value>\d+)\s+(?P<unit>\w+)` in `src/parameter/matchers.rs`; map to `Quantity` type and `Minimum` constraint; label = `"{qualifier} {value} {unit}"` (SEC-3, SEC-4)
- [X] T021 [US4] Run `cargo test parameter::matchers::tests::quantity` — confirm GREEN; verify all prior tests pass

**Checkpoint**: All four matchers (TimeWindow, Threshold, Frequency, Quantity) pass their TDD test suites.

---

## Phase 7: User Story 5 — Link Parameters to Parent Controls (Priority: P1)

**Goal**: Orchestrate all matchers to extract parameters from a full `PolicyDocument`, replace matched spans with OSCAL insertion placeholders, emit `OscalParam` elements within the correct `OscalControl.params` array, and wire the pass into the pipeline.

**Independent Test**: `cargo test parameter::tests::control_with_params` — create a `PolicyDocument` with one `PolicyRequirement` (stable_id "POL-AC-001", text "Passwords must be changed within 30 days"), call `extract_parameters()`, then run `build_catalog()`, and assert the resulting JSON contains `"params"` with id "POL-AC-001_prm_0" nested within control "pol-ac-001".

**Depends on**: All Phase 3–6 matchers must exist (at minimum TimeWindowMatcher for the independent test above).

### Tests for User Story 5

> **Write tests FIRST. Confirm they FAIL before implementing. (RED)**

- [X] T022 [US5] Write failing tests for `parameter_id()` in `src/parameter/mod.rs` — assert `parameter_id("POL-AC-001", 0) == "POL-AC-001_prm_0"` and `parameter_id("POL-AC-001", 1) == "POL-AC-001_prm_1"` per `contracts/parameter_api.rs`. Function signature: `fn parameter_id(requirement_id: &str, position: usize) -> String` (two params; value excluded by design per S-3).
- [X] T023 [US5] Write failing tests for `extract_parameters_from_text()` in `src/parameter/mod.rs` — cover: single time window produces one param with placeholder; multi-parameter requirement produces two params in position order; overlap resolution (first-match-wins; ties by longer match); OSCAL placeholder format `{{ insert: param, id-ref: POL-AC-001_prm_0 }}` appears in updated text; requirement with no params unchanged (SEC-5: reverse-order replacement). **Required overlap fixture**: "MFA must require no fewer than 3 authentication factors" → QuantityMatcher wins over ThresholdMatcher (longer match at same start offset) → `parameter_type = Quantity`, `value = "3"`, `constraint_type = Minimum`. Also test "no fewer than 3" without unit noun → ThresholdMatcher wins (only matcher that fires) → `parameter_type = Threshold`.
- [X] T024 [P] [US5] Add `OscalParam` and `OscalParamConstraint` structs (with `#[derive(Debug, Clone, Serialize, Deserialize)]` and appropriate `serde` skip attributes) and `params: Vec<OscalParam>` field to `OscalControl` (before `parts` for OSCAL schema ordering) in `src/oscal/catalog.rs` — parallel with T022/T023 since different file
- [X] T025 [US5] Write failing tests for `extract_parameters()` document-level pass in `src/parameter/mod.rs` — cover: document with 3 parameterized requirements; document with zero requirements (EC-5); requirements without stable_id are skipped; verify `requirement.text` updated and `requirement.parameters` populated
- [X] T026 [US5] Write failing tests for `to_oscal_param()` in `src/parameter/mod.rs` — cover all four parameter types and constraint types; assert correct `id`, `label`, `values[0]`, `constraints[0].description` format (`"minimum: 30 days"`, `"maximum: 15 minutes"`, `"exact: annually"`)

### Implementation for User Story 5

- [X] T027 [US5] Implement `parameter_id()` function in `src/parameter/mod.rs`: `pub fn parameter_id(requirement_id: &str, position: usize) -> String { format!("{requirement_id}_prm_{position}") }` (two params, no value arg; depends on T022 failing)
- [X] T028 [US5] Implement `extract_parameters_from_text()` in `src/parameter/mod.rs`: collect matches from all four matchers; sort by `start` ascending; resolve overlaps (first-match-wins, ties by longer match); replace spans in reverse-order with placeholders; assign deterministic IDs via `parameter_id()`; return `(updated_text, Vec<PolicyParameter>)` (SEC-5; depends on T023 failing)
- [X] T029 [US5] Implement `extract_parameters()` document-level enrichment pass in `src/parameter/mod.rs`: iterate over `document.sections[].requirements`; skip requirements where `stable_id.is_none()`; call `extract_parameters_from_text()`; update `requirement.text` and `requirement.parameters` in-place; add `tracing::info!` for count and `tracing::debug!` for per-parameter details (SEC-1: no info-level logging of values)
- [X] T030 [US5] Implement `to_oscal_param()` in `src/parameter/mod.rs`: map `PolicyParameter` fields to `OscalParam`; format constraint description as `"{constraint_type_lowercase}: {value}"` (depends on T026 failing and T024 complete for `OscalParam` type)
- [X] T031 [US5] Update `build_catalog()` (or control-building function) in `src/oscal/catalog.rs` to emit `params` from `requirement.parameters` by calling `to_oscal_param()` for each parameter; ensure serialization order places `params` before `parts`
- [X] T032 [US5] Wire `crate::parameter::extract_parameters(&mut doc)?` into `prepare_document()` in `src/pipeline.rs` after citation extraction step; add `use crate::parameter;` import
- [X] T033 [US5] Run `cargo test parameter` — confirm GREEN for all parameter module tests; run `cargo test` — confirm full suite passes
- [X] T033a [US5] Update `src/export/xml_serializer.rs` to write `<param>` elements for each `OscalParam` in `OscalControl.params`: emit `<param id="{id}">`, child `<label>`, `<value>`, and `<constraint><description>` elements before the existing `<part>` elements (maintaining OSCAL schema ordering); follow the existing element-writing patterns in the file (SEC-5 compliance: no string concatenation)
- [X] T033b [US5] Write integration test verifying XML output (`--format xml`) contains `<param>` elements within `<control>` for a requirement with extracted parameters; assert `id`, `label`, `value` attributes/elements are present (parallel with T033a once OSCAL types from T024 exist)
- [X] T033c [US5] Verify YAML output (`--format yaml`) includes `params` arrays within controls — since YAML uses serde via `src/export/yaml.rs`, adding `OscalParam` with `#[serde(skip_serializing_if = "Vec::is_empty")]` on `OscalControl.params` (from T024) should serialize automatically; write a targeted integration test to confirm

**Checkpoint**: `cargo run -- convert /tmp/test_params.md --strategy catalog --format json` produces OSCAL output with `"params"` arrays inside controls. `--format xml` produces `<param>` elements. `--format yaml` produces `params:` arrays. User Story 5 end-to-end test passes.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Integration tests, security requirement verification, quality gates. Catches issues that span multiple user stories.

- [X] T034 [P] Write idempotence test `parameter::tests::double_extraction_identical` in `src/parameter/mod.rs`: call `extract_parameters()` on a document, clone the result, call again, assert `parameters` and `text` are identical — verifies SEC-6 and S-4 (idempotence) and that OSCAL placeholders don't re-match any pattern
- [X] T035 [P] Write false-positive negative fixture tests in `src/parameter/mod.rs`: assert "Section 3.2", "NIST SP 800-53", "RFC 2119", "per Appendix A" produce zero parameters — verifies SEC-2 and EC-2
- [X] T036 [P] Write `parameter::tests::redos_adversarial` in `src/parameter/mod.rs`: run all four matchers against a 10,000-character adversarial string (all "a"s + " within"), assert completion within a deterministic bound and no panic — verifies SEC-3
- [X] T037 [P] Write edge case tests covering EC-1 (no params → text unchanged), EC-3 (whitespace/punctuation preserved around placeholder), EC-4 (multi-parameter requirement → unique IDs), EC-5 (empty document → no error), EC-6 (bare "quarterly" → Exact), EC-7 ("no less than" → Minimum), EC-9 (weeks/months/years time windows), EC-10 (empty text → no error) in `src/parameter/mod.rs`
- [X] T038 Run quality gates from `quickstart.md`: `cargo fmt --check` (zero violations), `cargo clippy -- -D warnings` (zero warnings), `cargo test` (all pass), `cargo doc --no-deps 2>&1 | grep -E "(warning|error)"` (zero doc warnings). **Additional guardrail checks** (F14): review `src/parameter/` for (a) any NLP/ML imports or calls — must have none per PRD W-1; (b) any ISO 8601 duration normalization (e.g., "P30D") — must have none per PRD W-4; (c) any `.unwrap()` outside `LazyLock::new(|| ...)` static initializers — must have none per constitution Principle VIII.
- [X] T039 Verify `cargo tarpaulin --lib` achieves ≥90% line coverage for `src/parameter/` module; add targeted tests for any uncovered branches
- [X] T040 [P] Write a `criterion` benchmark (`benches/parameter_extraction.rs`) measuring `extract_parameters()` on a synthetic `PolicyDocument` with 500 requirements (mix of parameterized and non-parameterized), each ~100 characters; assert p95 completion ≤1 second on CI hardware; this verifies the PRD performance target (NF-1) with a measurable bound (SEC-3 complementary: `regex` crate linear-time guarantee is demonstrated empirically)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Setup — **BLOCKS all user stories**
- **US1 (Phase 3)**: Depends on Foundational — no story dependencies
- **US2 (Phase 4)**: Depends on Foundational — no story dependencies; can start in parallel with US1 (different structs in matchers.rs)
- **US3 (Phase 5)**: Depends on Foundational — can start in parallel with US1/US2
- **US4 (Phase 6)**: Depends on Foundational — can start in parallel with US1/US2/US3
- **US5 (Phase 7)**: Depends on Foundational + at minimum US1 (TimeWindowMatcher required for independent test); all matchers recommended before integration
- **Polish (Phase 8)**: Depends on US5 completion (full stack required for integration tests)

### User Story Dependencies

| Story | Priority | Blocks | Blocked By | Can Parallel With |
|-------|----------|--------|------------|-------------------|
| US1 (Time Window) | P1 | US5 (integration) | Phase 2 | US2, US3, US4 |
| US2 (Threshold) | P1 | US5 (integration) | Phase 2 | US1, US3, US4 |
| US3 (Frequency) | P1 | US5 (integration) | Phase 2 | US1, US2, US4 |
| US4 (Quantity) | P2 | US5 (integration) | Phase 2 | US1, US2, US3 |
| US5 (OSCAL Link) | P1 | Polish | Phase 2 + US1 (min) | T024 (OSCAL types) |

### Within Each User Story

1. Write tests → confirm RED (failing)
2. Implement → confirm GREEN (passing)
3. Run full `cargo test` before moving to next story

### Parallel Opportunities (Single Developer)

Within Phase 2:
- T007 (error.rs) runs parallel with T004/T005 (model/mod.rs)

Within Phase 7:
- T024 (oscal/catalog.rs OSCAL types) runs parallel with T022/T023 (parameter/mod.rs tests)

Across Phase 3–6 (multi-developer):
- All four matcher stories can proceed in parallel once Phase 2 completes

---

## Parallel Example: US1 + US2 (Multi-Developer)

```bash
# After Phase 2 completes, launch simultaneously:

# Developer A: User Story 1 (Time Window)
cargo test parameter::matchers::tests::time_window  # Red first
# implement TimeWindowMatcher
cargo test parameter::matchers::tests::time_window  # Green

# Developer B: User Story 2 (Threshold) — different struct, same file
cargo test parameter::matchers::tests::threshold    # Red first
# implement ThresholdMatcher
cargo test parameter::matchers::tests::threshold    # Green
```

---

## Implementation Strategy

### MVP First (P1 Stories Only, Single Developer)

1. Complete Phase 1: Setup (T001–T003)
2. Complete Phase 2: Foundational (T004–T007) — **critical gate**
3. Complete Phase 3: US1 Time Window (T008–T012)
4. Complete Phase 4: US2 Threshold (T013–T015)
5. Complete Phase 5: US3 Frequency (T016–T018)
6. Complete Phase 7: US5 OSCAL Integration (T022–T033) — uses P1 matchers only
7. **STOP and VALIDATE**: Run end-to-end test from `quickstart.md`
8. Continue to Phase 6 (US4 Quantity) and Phase 8 (Polish)

### Full Delivery (All Stories)

Follow phases in order: 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8

Each phase checkpoint verifies that story is independently testable before proceeding.

### Acceptance Criteria Coverage

| AC ID | Story | Task(s) |
|-------|-------|---------|
| AC-1 (time window, "30 days" extracted) | US1 | T008, T010, T011 |
| AC-2 (threshold min, "128-bit") | US2 | T013, T014 |
| AC-3 (threshold max, "15 minutes") | US2 | T013, T014 |
| AC-4 (frequency, "annually") | US3 | T016, T017 |
| AC-5 (OSCAL param elements) | US5 | T026, T030 |
| AC-6 (param linked to control) | US5 | T031 |
| AC-7 (insertion placeholder in prose) | US1/US5 | T023, T028 |
| AC-8 (document-level pass) | US5 | T025, T029 |
| AC-9 (quantity, "3 factors") | US4 | T019, T020 |
| AC-10 (constraint inference) | US2/US5 | T014, T023, T028 |

### Security Requirements Coverage

| SEC ID | Coverage |
|--------|----------|
| SEC-1 (no info logging of param values) | T029 (`extract_parameters` implementation) |
| SEC-2 (qualifier required, no false positives) | T009, T035 (negative fixtures) |
| SEC-3 (bounded quantifiers, no nested repetition) | T011, T014, T017, T020 (all matcher implementations) |
| SEC-4 (LazyLock one-time compilation) | T011, T014, T017, T020 |
| SEC-5 (position-sorted + reverse replacement) | T028 (`extract_parameters_from_text`) |
| SEC-6 (idempotence) | T034 (idempotence test) |

---

## Notes

- [P] tasks = can run in parallel with other tasks in the same phase (different files or independent sections)
- [Story] label maps task to user story for traceability
- TDD workflow is mandatory: RED → GREEN for every implementation pair
- Run `cargo test` after every phase to verify no regressions
- Commit after each phase checkpoint
- `LazyLock<Regex>` must be at module level (not inside functions) — see quickstart.md Troubleshooting
- OSCAL schema requires `params` before `parts` in `OscalControl` struct field order
- OSCAL placeholder format is non-negotiable: `{{ insert: param, id-ref: {id} }}` (double braces per OSCAL markup spec)
