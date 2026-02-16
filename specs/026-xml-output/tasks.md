# Tasks: OSCAL XML Output (026)

**Input**: Design documents from `/specs/026-xml-output/`
**Prerequisites**: plan.md, data-model.md, contracts/xml-serializer.md, research.md, quickstart.md
**PRD**: `docs/PRD/026-prd-xml-output.md` (MoSCoW requirements M-1–M-7, S-1–S-3)
**AR**: `docs/AR/026-ar-xml-output.md` (Option 1: quick-xml manual construction)
**SEC**: `docs/SEC/026-sec-xml-output.md` (SEC-1–SEC-7)

**Tests**: TDD is mandatory per constitution principle IV. Tests are included.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add `quick-xml` dependency and create module scaffolding

- [X] T001 Add `quick-xml` dependency to Cargo.toml (latest stable, MIT)
- [X] T002 [P] Create `src/export/xml_serializer.rs` module file with `OSCAL_NS` and `INDENT_SIZE` constants, and declare it in `src/export/mod.rs`
- [X] T003 [P] Download OSCAL v1.2.0 XSD schemas to `tests/fixtures/xsd/` (oscal_catalog_schema.xsd, oscal_component_schema.xsd, oscal_complete_schema.xsd)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Implement shared XML helper functions that all serializers depend on

**CRITICAL**: No user story work can begin until these helpers exist

- [X] T004 Write unit tests for `write_metadata` helper in `src/export/xml_serializer.rs` — test that metadata fields (title, last-modified, version, oscal-version) are written as child elements in XSD order
- [X] T005 Implement `write_metadata` helper function in `src/export/xml_serializer.rs` per contracts/xml-serializer.md — writes `<metadata>` with child elements in XSD order using `quick_xml::Writer`
- [X] T006 [P] Write unit tests for `write_prop` helper — test that `<prop name="..." value="..." />` is emitted as self-closing element with attributes; test optional `ns` attribute
- [X] T007 [P] Implement `write_prop` helper function in `src/export/xml_serializer.rs` — writes `<prop>` element with name, value as attributes and optional ns attribute
- [X] T008 [P] Write unit tests for `write_link` helper — test that `<link href="..." rel="...">` is emitted with attributes and optional `<text>` child
- [X] T009 [P] Implement `write_link` helper function in `src/export/xml_serializer.rs` — writes `<link>` element with href/rel as attributes, optional text child
- [X] T010 Write unit tests for `write_part` helper — test that `<part id="..." name="...">` includes props, `<p>` prose elements, and nested parts in XSD order
- [X] T011 Implement `write_part` helper function in `src/export/xml_serializer.rs` — writes `<part>` with id/name attributes, child props, `<p>` prose, and recursive nested parts
- [X] T012 Write unit tests for `write_back_matter` and `write_resource` helpers — test resource with uuid attribute, title, description, props, citation, rlinks in XSD order
- [X] T013 Implement `write_back_matter`, `write_resource` helpers in `src/export/xml_serializer.rs` — writes `<back-matter>` with child `<resource>` elements including title, description, props, citation/text, rlinks
- [X] T014 [P] Write unit tests for `map_xml_err` and `map_utf8_err` error mapping functions in `src/export/xml_serializer.rs` — verify quick-xml and UTF-8 errors wrap to `ForgeError::Serialization`
- [X] T015 [P] Implement `map_xml_err` and `map_utf8_err` error mapping functions in `src/export/xml_serializer.rs`

**Checkpoint**: All shared helpers ready — user story implementation can now begin

---

## Phase 3: User Story 1 — Convert Policy to OSCAL XML (Priority: P1) MVP

**Goal**: `forge convert policy.md --strategy catalog --format xml` produces valid OSCAL Catalog XML; `--strategy component --format xml` produces valid Component Definition XML
**PRD**: M-1, M-2, M-3, M-4, M-7 | AC-1, AC-2, AC-6
**Independent Test**: Run `forge convert` with `--format xml` and verify output contains XML declaration, OSCAL namespace, and correct element structure

### Tests for User Story 1

> **NOTE: Write tests FIRST, ensure they FAIL before implementation**

- [X] T016 [P] [US1] Write unit tests for `write_control` helper in `src/export/xml_serializer.rs` — test that `<control id="...">` includes title, props, links, parts in XSD order; verify `uuid` field is NOT serialized (per data-model.md)
- [X] T017 [P] [US1] Write unit tests for `write_group` helper in `src/export/xml_serializer.rs` — test that `<group id="...">` includes title, props, links, controls in XSD order
- [X] T018 [P] [US1] Write unit tests for `serialize_catalog_to_xml` in `src/export/xml_serializer.rs` — test complete catalog XML with XML declaration, OSCAL namespace, uuid attribute, metadata, groups, controls, back-matter
- [X] T019 [P] [US1] Write unit tests for `write_component` helper in `src/export/xml_serializer.rs` — test that `<component uuid="..." type="...">` includes title, description, props in XSD order; verify control_implementations is SKIPPED
- [X] T020 [P] [US1] Write unit tests for `serialize_component_definition_to_xml` in `src/export/xml_serializer.rs` — test complete component definition XML with XML declaration, OSCAL namespace, uuid attribute, metadata, components, back-matter

### Implementation for User Story 1

- [X] T021 [US1] Implement `write_control` helper in `src/export/xml_serializer.rs` — writes `<control>` with id attribute, child title/props/links/parts in XSD order
- [X] T022 [US1] Implement `write_group` helper in `src/export/xml_serializer.rs` — writes `<group>` with id attribute, child title/props/links/controls in XSD order
- [X] T023 [US1] Implement `serialize_catalog_to_xml` public function in `src/export/xml_serializer.rs` — complete OSCAL Catalog XML document with declaration, namespace, uuid, metadata, groups, back-matter per contract
- [X] T024 [US1] Implement `write_component` helper in `src/export/xml_serializer.rs` — writes `<component>` with uuid/type attributes, child title/description/props in XSD order
- [X] T025 [US1] Implement `serialize_component_definition_to_xml` public function in `src/export/xml_serializer.rs` — complete OSCAL Component Definition XML document per contract

### CLI & Pipeline Wiring for User Story 1

- [X] T026 [US1] Write unit test in `src/cli/convert.rs` verifying `OutputFormat::Xml` is accepted (no longer rejected) for both catalog and component strategies
- [X] T027 [US1] Modify `src/cli/convert.rs` — replace the XML rejection guard (lines 23-28) with a YAML-only rejection: `if matches!(format, OutputFormat::Yaml) { return Err(...) }`
- [X] T028 [US1] Modify `src/pipeline.rs` — add `format: &OutputFormat` parameter to `run_catalog_pipeline` and `run_component_pipeline`; when `OutputFormat::Xml`, call `serialize_catalog_to_xml` / `serialize_component_definition_to_xml` instead of `serde_json`; skip JSON schema validation for XML output
- [X] T029 [US1] Update `src/cli/convert.rs` — pass `format` parameter through to pipeline functions
- [X] T030 [US1] Write integration test in `tests/xml_catalog_test.rs` — convert `tests/fixtures/sample_policy.md` with `--strategy catalog --format xml` and verify output contains `<?xml`, `xmlns`, `<catalog`, `<metadata>`, `<group>`, `<control>`
- [X] T031 [US1] Write integration test in `tests/xml_component_test.rs` — convert `tests/fixtures/sample_policy.md` with `--strategy component --format xml` and verify output contains `<?xml`, `xmlns`, `<component-definition`, `<component`

**Checkpoint**: `forge convert --format xml` works for both catalog and component strategies

---

## Phase 4: User Story 2 — Semantic Equivalence Between JSON and XML (Priority: P1)

**Goal**: JSON and XML outputs from the same input contain identical data
**PRD**: M-6, M-7 | AC-5
**Independent Test**: Convert same policy with `--format json` and `--format xml`, compare logical content

### Tests for User Story 2

- [X] T032 [P] [US2] Write integration test in `tests/xml_catalog_test.rs` — convert same fixture to JSON and XML; parse both; compare metadata (uuid, title, version, oscal-version, last-modified), group count, control count, prop values, link hrefs, part prose, back-matter resource count
- [X] T033 [P] [US2] Write integration test in `tests/xml_component_test.rs` — convert same fixture to JSON and XML; compare metadata, component uuid/type/title/description, prop values

### Implementation for User Story 2

- [X] T034 [US2] Add snapshot tests using `insta` in `src/export/xml_serializer.rs` — snapshot the full XML output for a representative catalog and component definition to catch regressions in field mapping

**Checkpoint**: Semantic equivalence verified — JSON and XML carry identical data

---

## Phase 5: User Story 3 — XML Schema Validation (Priority: P1)

**Goal**: Generated XML validates against official OSCAL v1.2.0 XSD schemas with zero errors
**PRD**: M-5 | AC-3, AC-4
**SEC**: SEC-7
**Independent Test**: Validate generated XML with `xmllint --schema`

### Tests for User Story 3

- [X] T035 [P] [US3] Write XSD validation integration test in `tests/xml_validation_test.rs` — serialize a catalog from fixture, write to temp file, run `xmllint --schema tests/fixtures/xsd/oscal_catalog_schema.xsd --noout`, assert exit code 0; skip if `xmllint` not available
- [X] T036 [P] [US3] Write XSD validation integration test in `tests/xml_validation_test.rs` — serialize a component definition from fixture, write to temp file, run `xmllint --schema tests/fixtures/xsd/oscal_component_schema.xsd --noout`, assert exit code 0; skip if `xmllint` not available

### Implementation for User Story 3

- [X] T037 [US3] Fix any XSD validation failures discovered by T035/T036 — wrapped `<description>` content in `<p>` tags using `write_markup_element` for OSCAL markup-multiline compliance

**Checkpoint**: Generated XML passes OSCAL XSD validation

---

## Phase 6: User Story 4 — Export Existing JSON to XML (Priority: P2)

**Goal**: Provide XML serialization capability for existing OSCAL JSON artifacts (full `forge export` wiring deferred to WI-29)
**PRD**: S-1 | AC-7
**Independent Test**: Deserialize an OSCAL JSON file and serialize it to XML

> **Note**: This user story provides the serialization capability only. The `forge export` subcommand is wired in WI-29. Here we verify that an existing JSON artifact can be deserialized and re-serialized to valid XML.

### Tests for User Story 4

- [X] T038 [P] [US4] Write integration test in `tests/xml_catalog_test.rs` — read a valid OSCAL Catalog JSON fixture, deserialize to `CatalogEnvelope`, call `serialize_catalog_to_xml`, verify output is valid XML with matching data
- [X] T039 [P] [US4] Write integration test in `tests/xml_component_test.rs` — read a valid OSCAL Component Definition JSON fixture, deserialize to `ComponentDefinitionEnvelope`, call `serialize_component_definition_to_xml`, verify output is valid XML with matching data

### Implementation for User Story 4

- [X] T040 [US4] Ensure `CatalogEnvelope` and `ComponentDefinitionEnvelope` implement `Deserialize` (verify or add `#[derive(Deserialize)]` if missing) in `src/oscal/catalog.rs` and `src/oscal/component_definition.rs` so JSON fixtures can be loaded for XML round-trip
- [X] T054 [P] [US4] Write integration test for EC-8 — attempt to deserialize malformed/invalid JSON as `CatalogEnvelope` and `ComponentDefinitionEnvelope` and verify a descriptive `ForgeError` is returned (not a panic or corrupt XML)

**Checkpoint**: Existing JSON artifacts can be serialized to valid XML; malformed JSON produces descriptive errors

> **Note (S-3 deferred)**: PRD S-3 (serialize OSCAL Profile to XML) is not covered in this WI because the Profile model struct does not yet exist. Profile generation is planned for WI-30+. The AR references `serialize_profile_to_xml` as a future component.

---

## Phase 7: Security Hardening & Edge Cases

**Purpose**: Address SEC requirements and edge cases from PRD/SEC review

### Security Tests (SEC-1 through SEC-6)

- [X] T041 [P] Write unit test in `src/export/xml_serializer.rs` verifying no DTD declarations in output (SEC-1) — serialize a catalog and assert output does not contain `<!DOCTYPE` or `<!ENTITY`
- [X] T042 [P] Write unit test with adversarial input (SEC-2, SEC-3) — create a catalog with control titles/prose containing `<script>alert('xss')</script>`, `&entity;`, `]]>`, `<!-- comment -->` and verify all are XML-escaped in output
- [X] T043 [P] Write unit test verifying attributes are escaped (SEC-4) — create a catalog with a uuid containing `"` or `<` characters and verify quick-xml escapes them
- [X] T044 [P] Write unit test verifying namespace isolation (SEC-6) — create a catalog with control text containing `xmlns=` and verify it does not create a namespace declaration in output; verify only one `xmlns` attribute on root element

### Edge Case Tests (EC-1 through EC-7)

- [X] T045 [P] Write unit test for EC-1 — catalog with empty groups (no controls) produces valid XML with empty `<group>` elements
- [X] T046 [P] Write unit test for EC-5 — catalog with deeply nested groups (3+ levels) via nested parts in controls preserves correct XML hierarchy
- [X] T047 [P] Write unit test for EC-6 — verify pretty-printing uses consistent 2-space indentation and does not add spurious whitespace in text content
- [X] T048 [P] Write unit test for EC-7 — back-matter resource with rlink containing `media-type` attribute serializes correctly as XML attribute
- [X] T055 [P] Write unit test for EC-3 — prop with namespace-prefixed name (e.g., `name="ns:custom-name"`) serializes correctly as `<prop name="ns:custom-name" value="..."/>` without creating spurious namespace declarations

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Final quality checks and documentation

- [X] T049 Run `cargo fmt --check` and fix any formatting issues across all modified/new files
- [X] T050 Run `cargo clippy -- -D warnings` and fix any warnings in `src/export/xml_serializer.rs` and modified files
- [X] T051 Run full `cargo test` suite and verify zero failures (both existing tests and new XML tests)
- [X] T052 Add DEBUG-level tracing spans in `serialize_catalog_to_xml` and `serialize_component_definition_to_xml` for observability (constitution principle IX) — log serialization start/end with artifact type
- [X] T053 Run quickstart.md validation — execute the CLI commands from `specs/026-xml-output/quickstart.md` against a test fixture and verify they produce expected output
- [X] T056 [P] Add criterion benchmark for `serialize_catalog_to_xml` and `serialize_component_definition_to_xml` in `benches/xml_benchmark.rs` — benchmark with a representative catalog (~50 controls) and component definition; verify < 50ms target per plan.md (constitution principle VI)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 (T001 for quick-xml, T002 for module file)
- **US1 (Phase 3)**: Depends on Phase 2 completion (shared helpers)
- **US2 (Phase 4)**: Depends on US1 (Phase 3) — needs working XML serialization to compare
- **US3 (Phase 5)**: Depends on US1 (Phase 3) — needs XML output to validate; depends on T003 (XSD schemas)
- **US4 (Phase 6)**: Depends on US1 (Phase 3) — needs working serialization functions
- **Security/Edge Cases (Phase 7)**: Depends on Phase 2 (helpers exist)
- **Polish (Phase 8)**: Depends on all previous phases

### User Story Dependencies

- **User Story 1 (P1)**: Depends on Foundational (Phase 2) — no other story dependencies
- **User Story 2 (P1)**: Depends on US1 (needs both JSON and XML output working)
- **User Story 3 (P1)**: Depends on US1 (needs XML output) + T003 (XSD schemas)
- **User Story 4 (P2)**: Depends on US1 (needs serialization functions) — independent of US2/US3

### Within Each User Story

- Tests MUST be written and FAIL before implementation (TDD)
- Helpers before public functions
- Public functions before CLI/pipeline wiring
- Integration tests after implementation

### Parallel Opportunities

- T002 and T003 can run in parallel (different files)
- T006/T007, T008/T009, T014/T015 can run in parallel within Phase 2
- All [P]-marked test tasks within a phase can run in parallel
- US3 and US4 can run in parallel after US1 completes
- All Phase 7 security/edge case tests can run in parallel

---

## Parallel Example: Phase 2 (Foundational)

```bash
# After T004/T005 (write_metadata) complete:
# Launch prop and link helpers in parallel:
Task: "Write tests for write_prop helper" (T006)
Task: "Write tests for write_link helper" (T008)
Task: "Write tests for error mapping" (T014)

# After their tests pass:
Task: "Implement write_prop" (T007)
Task: "Implement write_link" (T009)
Task: "Implement error mappers" (T015)
```

## Parallel Example: User Story 1

```bash
# Launch all US1 unit tests in parallel:
Task: "Write tests for write_control" (T016)
Task: "Write tests for write_group" (T017)
Task: "Write tests for serialize_catalog_to_xml" (T018)
Task: "Write tests for write_component" (T019)
Task: "Write tests for serialize_component_definition_to_xml" (T020)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T003)
2. Complete Phase 2: Foundational helpers (T004–T015)
3. Complete Phase 3: User Story 1 (T016–T031)
4. **STOP and VALIDATE**: `forge convert --format xml` works for catalog and component
5. This delivers the core capability (PRD M-1 through M-4, M-7)

### Incremental Delivery

1. Setup + Foundational → helpers ready
2. US1 → XML convert works → **MVP** (M-1, M-2, M-3, M-4, M-7)
3. US2 → Semantic equivalence verified (M-6)
4. US3 → XSD validation passes (M-5)
5. US4 → JSON-to-XML round-trip ready (S-1)
6. Security + Polish → hardened and production-ready

### Task Count Summary

| Phase | Tasks | Parallel Opportunities |
|-------|-------|----------------------|
| Phase 1: Setup | 3 | 2 parallel (T002, T003) |
| Phase 2: Foundational | 12 | 6 parallel pairs |
| Phase 3: US1 (P1) | 16 | 5 parallel test tasks, 2 parallel integration tests |
| Phase 4: US2 (P1) | 3 | 2 parallel test tasks |
| Phase 5: US3 (P1) | 3 | 2 parallel test tasks |
| Phase 6: US4 (P2) | 4 | 3 parallel test tasks |
| Phase 7: Security/Edge | 9 | All 9 parallel |
| Phase 8: Polish | 6 | 1 parallel (benchmark) |
| **Total** | **56** | |

---

## Notes

- [P] tasks = different files, no dependencies on incomplete tasks
- [Story] label maps task to specific user story for traceability
- TDD is mandatory: write test → verify FAIL → implement → verify PASS
- Commit after each task or logical group
- All XML construction uses `quick-xml::Writer` — never string concatenation (SEC-5, AR guardrail)
- Element ordering must match OSCAL v1.2.0 XSD sequences (see data-model.md tables)
- UUIDs are XML attributes, not child elements (AR guardrail)
