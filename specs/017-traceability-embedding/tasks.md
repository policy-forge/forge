# Tasks: Traceability Embedding (WI-17)

**Input**: Design documents from `/specs/017-traceability-embedding/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/trace_embedding.rs
**Branch**: `017-traceability-embedding`

**Tests**: TDD is mandatory per Constitution IV. Test tasks are included and must be written before implementation.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single Rust crate**: `src/` at repository root
- Test modules: inline `#[cfg(test)] mod tests` per Rust convention

---

## Phase 1: Setup

**Purpose**: Create the trace_embedding module with constants and wire it into the OSCAL module tree.

- [x] T001 Create `src/oscal/trace_embedding.rs` with 5 constants (`FORGE_TRACE_NS`, `PROP_SOURCE_FILE`, `PROP_SOURCE_SECTION`, `PROP_SOURCE_LINE`, `LINK_REL_SOURCE`) and add `pub mod trace_embedding;` to `src/oscal/mod.rs` with appropriate re-exports

**Checkpoint**: `cargo check` passes. Constants are importable from `forge::oscal::trace_embedding`.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Structural changes to existing types that ALL user stories depend on. Helper functions shared by both Catalog and Component Definition trace embedding.

**CRITICAL**: No user story work can begin until this phase is complete.

### Struct Extensions

- [x] T002 Extend `OscalProp` with `ns: Option<String>` field (between `name` and `value`, with `#[serde(skip_serializing_if = "Option::is_none")]`), update the doc comment, and fix the construction site at `src/oscal/parts.rs:198` to add `ns: None`. Also replace `build_control_props` body (line 194) with `vec![]` since trace props are now added by `embed_trace_in_catalog` post-processing. Update existing tests in `src/oscal/parts.rs` that assert `forge:source-line` props.
- [x] T003 [P] Extend `OscalGroup` with `props: Vec<OscalProp>` and `links: Vec<OscalLink>` fields (after `title`, before `controls`, both with `#[serde(skip_serializing_if = "Vec::is_empty")]`), update the construction site at `src/oscal/catalog.rs:357` to add `props: vec![], links: vec![]`, and make `collect_requirements_with_section` (line 277) `pub(crate)` instead of private. Add `use crate::oscal::back_matter::OscalLink;` import if not present.
- [x] T004 [P] Extend `DocumentaryComponent` with `props: Vec<OscalProp>` field (after `description`, before `control_implementations`, with `#[serde(skip_serializing_if = "Vec::is_empty")]`) and update the construction site at `src/oscal/component_definition.rs:146` to add `props: vec![]`. Add `use crate::oscal::parts::OscalProp;` import if not present.
- [x] T005 Verify `cargo test` passes after structural changes — all existing tests must remain green

### Helper Functions (TDD)

- [x] T006 TDD: `encode_href_path` in `src/oscal/trace_embedding.rs` — write tests first covering: `%` → `%25`, space → `%20`, `#` → `%23`, empty string returns empty, normal path unchanged, combined special chars (e.g., `"my file#1%.md"` → `"my%20file%231%25.md"`). Then implement: sequential `.replace` with `%` encoded FIRST to avoid double-encoding.
- [x] T007 TDD: `build_trace_props` in `src/oscal/trace_embedding.rs` — write tests first asserting: returns exactly 3 `OscalProp` instances; props are ordered `source-file`, `source-section`, `source-line`; all have `ns: Some(FORGE_TRACE_NS)`; prop names use module constants; values match inputs; `source-line` is `line_number.to_string()`. Then implement per contract.
- [x] T008 TDD: `build_trace_link` in `src/oscal/trace_embedding.rs` — write tests first asserting: returns `OscalLink` with `rel: "source"`, `text: None`; href format is `"<encoded_path>#line=<n>"`; file path with special chars is percent-encoded via `encode_href_path`. Then implement per contract (depends on T006 for `encode_href_path`).
- [x] T009 Verify `cargo test` passes — all helper function tests green, no regressions

**Checkpoint**: Foundation ready. `OscalProp` has `ns`, `OscalGroup` has `props`/`links`, `DocumentaryComponent` has `props`. Helper functions `build_trace_props`, `build_trace_link`, `encode_href_path` are tested and working. All existing tests pass.

---

## Phase 3: User Story 1 — Catalog Control Trace Embedding (Priority: P1) MVP

**Goal**: Every generated OSCAL control in Catalog output contains 3 namespaced trace props (`source-file`, `source-section`, `source-line`) and 1 source link (`rel: "source"`, `href: "<file>#line=<n>"`).

**Independent Test**: Generate an OSCAL Catalog from a policy document and inspect any control's `props` and `links` arrays for trace metadata.

**Reqs**: M-1, M-2, M-6, M-8, SEC-4, SC-001, SC-002, SC-006

### Tests (write FIRST, must FAIL before implementation)

- [x] T010 [US1] Write unit tests for `embed_trace_in_catalog` control annotation in `src/oscal/trace_embedding.rs`: build a minimal `OscalCatalog` with 1 group containing 2 controls, build a `TraceLinkCollection` with matching entries (keyed by `control.uuid`), call `embed_trace_in_catalog`, assert each control has exactly 3 additional props with correct names/ns/values and 1 additional link with correct rel/href. Also test: control with no matching trace link gets no props/links added (partial trace data support). Also test: function logs annotated count at `tracing::debug!`.

### Implementation

- [x] T011 [US1] Implement `embed_trace_in_catalog` in `src/oscal/trace_embedding.rs` — iterate `catalog.groups` → `group.controls`, look up `trace_links.by_oscal_element(control.uuid)`, if found: append `build_trace_props(file, section, line)` to `control.props` and `build_trace_link(file, line)` to `control.links`. Also handle group annotation: derive `source-section` from first child control's trace link `section_title`, add to `group.props`. If no child controls have trace links, skip group prop. Log annotated counts via `tracing::debug!`.
- [x] T012 [US1] Integrate `embed_trace_in_catalog` in catalog pipeline at `src/pipeline.rs:115` — after `build_catalog` returns and before envelope assembly: make `catalog` mutable, call `embed_trace_in_catalog(&mut catalog, &trace_links)`. The `catalog.groups` field is already moved into the envelope at line 144, so the mutation must happen before that point.
- [x] T013 [US1] Integration test: create a test Markdown file with at least 2 sections each containing requirements, run `run_catalog_pipeline`, deserialize the output JSON, verify every control has 3 trace props (correct ns, names, values matching source locations) and 1 source link (correct rel, href with line number). Verify `source-file` prop value matches the input path.

**Checkpoint**: Catalog controls have full trace provenance. `cargo test` passes. SC-001, SC-002, SC-006 satisfied.

---

## Phase 4: User Story 2 — Component Definition Trace Embedding (Priority: P1)

**Goal**: Every generated `implemented-requirement` in Component Definition output contains 3 trace props and 1 source link. The documentary component has a `source-file` prop.

**Independent Test**: Generate a Component Definition and inspect any `implemented-requirement`'s props/links and the documentary component's props.

**Reqs**: M-3, M-4, M-5, M-6, M-8, SEC-4, SC-003, SC-004, SC-005, SC-006

### Tests (write FIRST, must FAIL before implementation)

- [x] T014 [P] [US2] Write unit tests for `map_requirement_to_implemented` with trace data in `src/oscal/implemented_requirements.rs`: call with `source_file` and `section_title` params, assert the returned JSON `Value` has `"props"` array with 3 trace props (correct names, ns, values) and `"links"` array with 1 source link (correct rel, href). Test with `source_line: 42` to verify `source-line` value is `"42"`.
- [x] T015 [P] [US2] Write unit test for `DocumentaryComponent` source-file prop in `src/oscal/component_definition.rs`: build a component definition with a known input path, verify the documentary component's `props` array contains exactly 1 prop with `name: "source-file"`, `ns: Some(FORGE_TRACE_NS)`, and `value` matching the input path.

### Implementation

- [x] T016 [US2] Modify `map_requirement_to_implemented` in `src/oscal/implemented_requirements.rs` (line 86): add `source_file: &str` and `section_title: &str` parameters; import and call `build_trace_props(source_file, section_title, requirement.source_line)` and `build_trace_link(source_file, requirement.source_line)` from `trace_embedding`; add `"props"` and `"links"` arrays to the `serde_json::json!` output.
- [x] T017 [US2] Update `build_control_implementations` in `src/oscal/implemented_requirements.rs` (line 35): change import from `collect_requirements` to `collect_requirements_with_section`; accept `source_file: &str` parameter; iterate `(requirement, section)` pairs; pass `source_file` and `section.title` to `map_requirement_to_implemented`. Update the call site and all callers.
- [x] T018 [US2] Add `source_file: &str` parameter to `build_component_definition` in `src/oscal/component_definition.rs` (line 108); pass it to `build_control_implementations`; add `source-file` prop to `DocumentaryComponent` construction: `props: vec![OscalProp { name: PROP_SOURCE_FILE.to_string(), ns: Some(FORGE_TRACE_NS.to_string()), value: source_file.to_string() }]`. Import constants from `trace_embedding`.
- [x] T019 [US2] Update `run_component_pipeline` in `src/pipeline.rs` (line 184): pass `&input_path.display().to_string()` as `source_file` to `build_component_definition`. Update the function call to include the new parameter.
- [x] T020 [US2] Integration test: create a test Markdown file and baseline profile, run `run_component_pipeline`, deserialize the output JSON, verify: (a) documentary component has `source-file` prop matching input path, (b) every `implemented-requirement` has 3 trace props and 1 source link with correct values.

**Checkpoint**: Component Definition has full trace provenance. `cargo test` passes. SC-003, SC-004, SC-005 satisfied.

---

## Phase 5: User Story 3 — No Trace Data in Remarks (Priority: P1)

**Goal**: Verify that no generated OSCAL artifact contains trace metadata (file paths, line numbers, section references) in any `remarks` field.

**Independent Test**: Generate OSCAL artifacts and grep all `remarks` fields for trace-like content.

**Reqs**: M-7, SEC-1, SEC-2, SC-007

- [x] T021 [P] [US3] Write test in `src/oscal/trace_embedding.rs`: generate a full catalog (with trace embedding) from a test policy document, serialize to JSON string, parse with `serde_json`, recursively walk the JSON tree collecting all values under `"remarks"` keys, assert none contain any of: the source file path, the section title, line number strings matching source lines. This verifies M-7 and SEC-1/SEC-2 for Catalog output.
- [x] T022 [P] [US3] Write test in `src/oscal/implemented_requirements.rs` or `src/oscal/component_definition.rs`: generate a full component definition, serialize to JSON, recursively walk for `"remarks"` keys, assert none contain trace metadata. This verifies M-7 and SEC-2 for Component Definition output.

**Checkpoint**: SEC-1, SEC-2, M-7 verified. No trace data leaks into remarks fields.

---

## Phase 6: User Story 4 — Group-Level Source Annotation (Priority: P2)

**Goal**: Every Catalog group that has traceable child controls receives a `source-section` prop indicating the corresponding source section.

**Independent Test**: Generate a Catalog from a hierarchical policy document and inspect group elements for `source-section` props.

**Reqs**: S-1, S-2, EC-4, SC-008

- [x] T023 [US4] Write unit test in `src/oscal/trace_embedding.rs`: build a catalog with 2 groups — one with traceable controls, one with no traceable controls (no matching trace links). Call `embed_trace_in_catalog`. Assert: (a) first group has `source-section` prop with correct section title and `ns: Some(FORGE_TRACE_NS)`, (b) second group has NO `source-section` prop (EC-4). Verify prop value uses hierarchical section path (S-2).
- [x] T024 [US4] Write unit test verifying that the `source-section` prop value is derived from the first child control's trace link `section_title`, not the group's own `title` field.
- [x] T025 [US4] Integration test: generate a catalog from a multi-section policy, verify each group element in the JSON output has a `source-section` prop matching the section heading. Verify groups with no requirements have no `source-section` prop.

**Checkpoint**: Group-level traceability complete. S-1, S-2, EC-4 verified.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final validation, code quality, and security checks.

- [x] T026 [P] SEC-5 audit: search all source files in `src/oscal/trace_embedding.rs`, `src/oscal/implemented_requirements.rs`, and `src/oscal/component_definition.rs` for raw string literals `"source-file"`, `"source-section"`, `"source-line"`, `"source"` used as prop names or link rels — all must use constants (`PROP_SOURCE_FILE`, `PROP_SOURCE_SECTION`, `PROP_SOURCE_LINE`, `LINK_REL_SOURCE`). Fix any violations.
- [x] T027 [P] Run `cargo clippy -- -D warnings` and `cargo fmt --check` — fix any warnings or formatting issues
- [x] T028 Run full `cargo test` suite — all tests must pass (unit + integration)
- [x] T029 Validate quickstart.md examples: generate a catalog and component definition from a sample policy, compare JSON structure against `specs/017-traceability-embedding/quickstart.md` examples for structural consistency (prop names, namespace, link format)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 (constants needed for helpers) — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Phase 2 completion
- **US2 (Phase 4)**: Depends on Phase 2 completion — independent of Phase 3 (US1)
- **US3 (Phase 5)**: Depends on Phase 3 AND Phase 4 (needs both artifact types generated with trace data)
- **US4 (Phase 6)**: Depends on Phase 3 (group annotation is implemented in `embed_trace_in_catalog`, tested separately here)
- **Polish (Phase 7)**: Depends on all previous phases

### User Story Dependencies

- **US1 (P1)**: After Phase 2 — no dependencies on other stories
- **US2 (P1)**: After Phase 2 — independent of US1 (uses same helper functions but different embedding approach)
- **US3 (P1)**: After US1 AND US2 — verifies both artifact types
- **US4 (P2)**: After US1 — tests group behavior of the same `embed_trace_in_catalog` function

### Within Each Phase

- Tests MUST be written and FAIL before implementation (TDD)
- Struct changes before helper functions (Phase 2)
- Helper functions before embedding functions (Phase 2 → Phase 3/4)
- Embedding before pipeline integration
- Unit tests before integration tests

### Parallel Opportunities

- **Phase 2**: T003 and T004 can run in parallel (different files: catalog.rs vs component_definition.rs)
- **Phase 3 + Phase 4**: US1 and US2 can run in parallel after Phase 2 (different files, different embedding approaches)
- **Phase 4**: T014 and T015 can run in parallel (different test files)
- **Phase 5**: T021 and T022 can run in parallel (different test scopes)
- **Phase 7**: T026 and T027 can run in parallel

---

## Parallel Example: US1 + US2 After Foundational

```text
# After Phase 2 completes, launch US1 and US2 in parallel:

Agent A (US1 - Catalog):
  T010 → T011 → T012 → T013

Agent B (US2 - Component Definition):
  T014, T015 (parallel) → T016 → T017 → T018 → T019 → T020

# Then converge for US3:
  T021, T022 (parallel)
```

---

## Implementation Strategy

### MVP First (US1 Only)

1. Complete Phase 1: Setup (T001)
2. Complete Phase 2: Foundational (T002–T009)
3. Complete Phase 3: US1 — Catalog Controls (T010–T013)
4. **STOP and VALIDATE**: Generate a catalog from a real policy → verify trace props/links in JSON
5. This alone delivers core Catalog traceability (Parent PRD M-10 for Catalog)

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. Add US1 → Catalog traceability verified → MVP
3. Add US2 → Component Definition traceability verified → Full P1 coverage
4. Add US3 → No-remarks invariant verified → Security sign-off
5. Add US4 → Group-level annotation → Feature complete
6. Polish → Code quality, SEC-5 audit, quickstart validation

### Key Architecture Notes

- **Catalog**: Post-processing via `embed_trace_in_catalog()` after `build_catalog()` — uses `TraceLinkCollection.by_oscal_element(control.uuid)` for O(1) lookup
- **Component Definition**: Inline injection during construction — source data available at `map_requirement_to_implemented` call site
- **Shared helpers**: `build_trace_props`, `build_trace_link`, `encode_href_path` in `trace_embedding.rs` — used by both Catalog and Component Definition code paths
- **Clean replacement**: Old `forge:source-line` prop removed (build_control_props returns `vec![]`); new namespaced props added by trace embedding

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests FAIL before implementing (TDD cycle: RED → GREEN → REFACTOR)
- Commit after each phase completion
- All trace prop names MUST use module constants (SEC-5) — never raw string literals
- All trace props MUST have `ns: Some(FORGE_TRACE_NS)` (M-6, SEC-4)
- No trace data in `remarks` fields (M-7, SEC-1, SEC-2)
