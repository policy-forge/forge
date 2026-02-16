# Tasks: Export Subcommand (WI-29)

**Input**: Design documents from `/specs/029-export-subcommand/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/export.rs, PRD, AR, SEC

**Tests**: TDD is mandatory per constitution principle IV. Tests are included before implementation tasks.

**Organization**: Tasks are grouped by user story (from PRD) to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Dependencies, error types, and test fixtures

- [X] T001 Enable `serde` feature on `quick-xml` dependency in Cargo.toml (`quick-xml = { version = "0.37", features = ["serde"] }`)
- [X] T002 [P] Add export-specific `ForgeError` variants (`ExportUnsupportedExtension`, `ExportNoExtension`, `ExportInvalidOscal`, `ExportEmptyInput`) and exit code mapping (all → exit code 1) in src/error.rs per contracts/export.rs
- [X] T003 [P] Create OSCAL JSON test fixture files in tests/fixtures/export/ by copying and trimming from tests/fixtures/golden/small/expected-catalog.json and tests/fixtures/golden/small/expected-component-definition.json

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story

**CRITICAL**: No user story work can begin until this phase is complete

- [X] T004 Write unit tests for `detect_format()` in src/cli/export.rs: .json→Json, .xml→Xml, .yaml→Yaml, .yml→Yaml, unknown extension→ExportUnsupportedExtension, no extension→ExportNoExtension
- [X] T005 Implement `detect_format(path: &Path) -> Result<OutputFormat, ForgeError>` in src/cli/export.rs to pass T004 tests
- [X] T006 Define `OscalModel` enum (Catalog/Component variants) in src/cli/export.rs per data-model.md
- [X] T007 Add `Export` variant to `Commands` enum in src/cli/mod.rs with fields: `input: PathBuf`, `#[arg(long, value_enum)] format: OutputFormat`, `#[arg(long)] output: Option<PathBuf>`
- [X] T008 Register `pub mod export;` in src/cli/mod.rs and create src/cli/export.rs module file with `execute()` skeleton function
- [X] T009 Wire `Commands::Export` dispatch in `execute()` function in src/cli/mod.rs to call `export::execute(input, format, output)`

**Checkpoint**: `cargo build` succeeds; `forge export --help` shows usage; detect_format tests pass

---

## Phase 3: User Story 1 — Convert OSCAL JSON to XML (Priority: P1) MVP

**Goal**: `forge export catalog.json --format xml` produces valid OSCAL XML to stdout

**Independent Test**: Run `forge export tests/fixtures/export/catalog.json --format xml` and verify valid OSCAL XML output

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T010 [US1] Write unit test: `deserialize_oscal()` with JSON catalog content returns `OscalModel::Catalog` in src/cli/export.rs
- [X] T011 [US1] Write unit test: `export_artifact()` with JSON catalog fixture → XML output string contains `<catalog` and `xmlns` in src/cli/export.rs

### Implementation for User Story 1

- [X] T012 [US1] Implement `deserialize_oscal(content: &str, format: OutputFormat) -> Result<OscalModel, ForgeError>` for JSON format in src/cli/export.rs — parse to `serde_json::Value`, call `detect_model_type()`, then `serde_json::from_value()` into appropriate envelope type
- [X] T013 [US1] Implement `serialize_oscal(model: &OscalModel, format: OutputFormat) -> Result<String, ForgeError>` in src/cli/export.rs — dispatch to `serde_json::to_string_pretty()`, `serialize_catalog_to_xml()` / `serialize_component_definition_to_xml()`, or `serialize_to_yaml()` based on format and model variant
- [X] T014 [US1] Implement `validate_oscal_model(model: &OscalModel) -> Result<(), ForgeError>` in src/cli/export.rs — serialize model to `serde_json::Value`, call `validate::validate_artifact()`, map errors to `ForgeError::SchemaValidation`
- [X] T015 [US1] Implement `export_artifact(input_path: &Path, target_format: OutputFormat, output: Option<&Path>) -> Result<(), ForgeError>` in src/cli/export.rs — read file, detect format, deserialize, validate, serialize, call `pipeline::write_output()`
- [X] T016 [US1] Wire `execute()` in src/cli/export.rs to call `export_artifact()` with parsed args
- [X] T017 [US1] Write unit test: JSON component-definition fixture → XML output in src/cli/export.rs; verify T010 and T011 pass

**Checkpoint**: `forge export catalog.json --format xml` produces valid OSCAL XML. User Story 1 fully functional.

---

## Phase 4: User Story 2 — Convert OSCAL JSON to YAML (Priority: P1)

**Goal**: `forge export catalog.json --format yaml` produces valid OSCAL YAML

**Independent Test**: Run `forge export tests/fixtures/export/catalog.json --format yaml` and verify valid OSCAL YAML output

### Tests for User Story 2

- [X] T018 [US2] Write unit test: `export_artifact()` with JSON catalog fixture → YAML output string starts with `catalog:` in src/cli/export.rs
- [X] T019 [US2] Write unit test: `export_artifact()` with JSON component-definition fixture → YAML output in src/cli/export.rs

### Implementation for User Story 2

- [X] T020 [US2] Verify T018, T019 pass with no new implementation (YAML path already handled by `serialize_oscal()` from T013)

**Checkpoint**: `forge export catalog.json --format yaml` produces valid OSCAL YAML. US1 + US2 both work.

---

## Phase 5: User Story 3 — Convert Between Any Format Pair (Priority: P1)

**Goal**: All 9 format pair combinations (JSON/XML/YAML × JSON/XML/YAML) work correctly

**Independent Test**: Run `forge export catalog.xml --format json` and verify valid OSCAL JSON output

### Tests for User Story 3

> **NOTE: Write tests FIRST, ensure they FAIL before implementation**

- [X] T021 [US3] Create XML and YAML test fixture files in tests/fixtures/export/ — generate catalog.xml by serializing catalog.json fixture through `serialize_catalog_to_xml()`, generate catalog.yaml through `serialize_to_yaml()`, and similarly for component fixtures
- [X] T022 [P] [US3] Write unit tests for `deserialize_catalog_from_xml()` and `deserialize_component_from_xml()` in src/export/xml_deserializer.rs
- [X] T023 [P] [US3] Write unit test: `export_artifact()` XML catalog → JSON in src/cli/export.rs
- [X] T024 [P] [US3] Write unit test: `export_artifact()` YAML catalog → JSON in src/cli/export.rs

### Implementation for User Story 3

- [X] T025 [US3] Implement `deserialize_catalog_from_xml(xml: &str) -> Result<CatalogEnvelope, ForgeError>` and `deserialize_component_from_xml(xml: &str) -> Result<ComponentDefinitionEnvelope, ForgeError>` in src/export/xml_deserializer.rs using `quick_xml::de::from_str()` with serde feature
- [X] T026 [US3] Register `pub mod xml_deserializer;` in src/export/mod.rs and add re-exports for deserialization functions
- [X] T027 [US3] Extend `deserialize_oscal()` in src/cli/export.rs to handle `OutputFormat::Xml` (call xml_deserializer functions) and `OutputFormat::Yaml` (call `deserialize_from_yaml()`)
- [X] T028 [US3] Write comprehensive unit tests for all 9 format pair combinations (3 input × 3 output) with both Catalog and ComponentDefinition model types (18 test cases total) in src/cli/export.rs
- [X] T029 [US3] Write XXE prevention test (SEC-1): input XML with `<!DOCTYPE>` and `<!ENTITY>` declarations does not expand entities in src/export/xml_deserializer.rs

**Checkpoint**: All 9 format pairs work. `forge export` handles any input format → any output format.

---

## Phase 6: User Story 4 — Validate Output After Conversion (Priority: P1)

**Goal**: Invalid input produces descriptive errors; valid input passes OSCAL schema validation

**Independent Test**: Run `forge export invalid.json --format xml` and verify descriptive error with non-zero exit code

### Tests for User Story 4

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T030 [US4] Write test: invalid OSCAL JSON input (valid JSON, missing `catalog`/`component-definition` key) → `ExportInvalidOscal` error in src/cli/export.rs
- [X] T031 [P] [US4] Write test: empty input file (0 bytes) → `ExportEmptyInput` error in src/cli/export.rs
- [X] T032 [P] [US4] Write test: non-existent input file → `FileNotFound` error in src/cli/export.rs
- [X] T033 [P] [US4] Write test: unrecognized extension (`.txt`) → `ExportUnsupportedExtension` error in src/cli/export.rs
- [X] T034 [P] [US4] Write test: no file extension → `ExportNoExtension` error in src/cli/export.rs
- [X] T045 [P] [US4] Write test: file extension doesn't match content (e.g., `.json` file containing XML) → deserialization error with descriptive message (EC-2) in src/cli/export.rs *(ID out of sequence — added during implementation for EC-2 coverage)*

### Implementation for User Story 4

- [X] T035 [US4] Implement input validation in `export_artifact()` in src/cli/export.rs: check file exists (SEC-3), check non-empty (EC-5), then proceed with deserialization; ensure all error paths return descriptive ForgeError variants
- [X] T036 [US4] Verify all T030–T034, T045 error tests pass with correct error variants and exit codes

**Checkpoint**: All error paths produce descriptive messages. Invalid input is handled gracefully.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Integration tests, edge cases, code quality, and final verification

- [X] T037 [P] Write CLI integration tests in tests/export_integration.rs: test `forge export` binary invocation with valid JSON→XML, JSON→YAML, and invalid input scenarios using `std::process::Command`
- [X] T038 [P] Write `--output` file writing test: verify output is written to specified file path (using tempfile) in src/cli/export.rs
- [X] T039 [P] Write same-format normalization test (EC-3): JSON→JSON re-serializes and validates in src/cli/export.rs
- [X] T040 Add `tracing` verbose logging for format detection, deserialization, validation, and serialization stages in src/cli/export.rs (PRD S-2)
- [X] T041 Run `cargo clippy -- -D warnings` and fix all warnings
- [X] T042 Run `cargo fmt --check` and fix all formatting issues
- [X] T043 Verify test coverage ≥90% for export module (`cargo test` all export tests pass)
- [X] T044 Run `forge export` end-to-end with all 6 fixture files against all 3 formats to confirm 18 conversions succeed
- [X] T046 [P] Write `--output` to read-only path test (EC-4): verify descriptive filesystem error is reported in tests/export_integration.rs
- [X] T047 Run `cargo bench` performance benchmark: export 500KB+ OSCAL catalog fixture through JSON→XML pipeline completes in under 1 second (SC-005, Constitution VI) — add benchmark in benches/export_bench.rs using criterion

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Phase 2 — MVP target
- **US2 (Phase 4)**: Depends on Phase 3 (reuses serialize_oscal from US1)
- **US3 (Phase 5)**: Depends on Phase 3 (extends deserialize_oscal from US1)
- **US4 (Phase 6)**: Depends on Phase 3 (tests error paths of export_artifact from US1)
- **Polish (Phase 7)**: Depends on Phases 3–6

### User Story Dependencies

- **US1 (P1)**: Can start after Phase 2. MVP — delivers JSON→XML.
- **US2 (P1)**: Depends on US1 (shares serialize_oscal). Minimal work — just tests.
- **US3 (P1)**: Depends on US1. Main work: XML deserialization + YAML input.
- **US4 (P1)**: Depends on US1. Main work: error path tests + input validation.
- **US3 and US4 can proceed in parallel** after US1 completes.

### Within Each User Story

- Tests written and FAIL before implementation (TDD)
- Verify tests pass after implementation
- Commit after each checkpoint

### Parallel Opportunities

```
Phase 1:  T002 ──┐
          T003 ──┤ (parallel: different files)
                 │
Phase 2:  T004 → T005 → T006 → T007 → T008 → T009 (sequential within phase)
                 │
Phase 3:  T010, T011 (tests) → T012–T016 (impl) → T017 (verify)
                 │
         ┌───────┴───────┐
Phase 4:  T018–T020      Phase 5: T022, T023, T024 (parallel tests)
  (US2, sequential)        → T025–T029 (impl + verify)
                          │
                     Phase 6: T030, T031, T032, T033, T034, T045 (parallel tests)
                          → T035, T036 (impl + verify)
                 │
Phase 7:  T037, T038, T039, T046 (parallel: different files)
          → T040–T044 (sequential) → T047 (benchmark)
```

---

## Parallel Example: User Story 3

```bash
# Launch parallel tests for US3 (different test files/scenarios):
Task: "Write unit tests for XML deserialization in src/export/xml_deserializer.rs"  # T022
Task: "Write test: export XML catalog → JSON in src/cli/export.rs"                  # T023
Task: "Write test: export YAML catalog → JSON in src/cli/export.rs"                 # T024

# After tests written, implement sequentially:
Task: "Implement XML deserialization in src/export/xml_deserializer.rs"              # T025
Task: "Register xml_deserializer module in src/export/mod.rs"                        # T026
Task: "Extend deserialize_oscal for XML and YAML in src/cli/export.rs"              # T027
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T003)
2. Complete Phase 2: Foundational (T004–T009)
3. Complete Phase 3: User Story 1 (T010–T017)
4. **STOP and VALIDATE**: `forge export catalog.json --format xml` produces valid XML
5. Commit and tag as MVP

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. Add US1 (JSON→XML) → **MVP!** Commit.
3. Add US2 (JSON→YAML) → Minimal delta. Commit.
4. Add US3 (All format pairs) → Full format support. Commit.
5. Add US4 (Error handling) → Robust error paths. Commit.
6. Polish → Integration tests, clippy, fmt, coverage. Commit.

### Suggested MVP Scope

**US1 only** (Phase 1 + Phase 2 + Phase 3 = 16 tasks). Delivers a working `forge export` for the most common use case (JSON→XML) and validates the entire pipeline architecture.

---

## Traceability

| PRD Req | Task(s) | User Story |
|---------|---------|------------|
| M-1 (CLI subcommand) | T007, T008, T009 | Foundational |
| M-2 (format detection) | T004, T005 | Foundational |
| M-3 (deserialize + reserialize) | T012, T013, T025, T027 | US1, US3 |
| M-4 (validate output) | T014 | US1 |
| M-5 (--output or stdout) | T038 | Polish |
| M-6 (descriptive errors) | T002, T030–T036, T045 | Setup, US4 |
| S-1 (content-based detection) | — | Deferred |
| S-2 (verbose format logging) | T040 | Polish |
| S-3 (same-format normalization) | T039 | Polish |
| SEC-1 (XXE prevention) | T029 | US3 |
| SEC-3 (input existence check) | T035 | US4 |
| SEC-4 (extension-based detection) | T004, T005 | Foundational |
| SEC-5 (no panic on invalid input) | T030–T034, T045 | US4 |
| SEC-7 (output validation) | T014, T028 | US1, US3 |
| EC-2 (format mismatch) | T045 | US4 |
| EC-4 (read-only output) | T046 | Polish |
| SC-003 (round-trip fidelity) | — | Covered by WI-28 |
| EC-7 (missing --format) | T007 | Foundational (clap required arg) |
| SC-005 (performance < 1s) | T047 | Polish |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- TDD is mandatory: write tests first, verify they fail, then implement
- Commit after each checkpoint
- Constitution principle X: single generic pipeline, no 9 separate functions
- AR guardrail: DO NOT merge export into forge convert
- AR guardrail: DO NOT skip output validation
- AR guardrail: DO NOT re-implement serialization (reuse WI-26/WI-27)

## Design Deviations from AR

The following AR interface decisions were refined during research (research.md) and are superseded by contracts/export.rs:

| AR Decision | Implementation Decision | Rationale |
|-------------|------------------------|-----------|
| `OscalFormat` enum | Reuse existing `OutputFormat` enum | RQ-5: Identical variants; avoids duplication (YAGNI) |
| Validate AFTER serialization (`validate_oscal(&output_string, format)`) | Validate BEFORE serialization (`validate_oscal_model(&model)` via JSON intermediate) | RQ-2: OSCAL schemas are JSON-only; validate model as JSON Value before target serialization |
| `ForgeError::UnsupportedFormat` / `InvalidOscal` | `ExportUnsupportedExtension`, `ExportNoExtension`, `ExportInvalidOscal`, `ExportEmptyInput` | RQ-4: Existing `UnsupportedFormat` has Markdown-specific message; new variants provide context-appropriate messages |
| `ExportArgs.verbose: bool` field | No verbose field; use global `-v` flag via tracing | Codebase convention: verbosity is a global CLI concern, not per-subcommand |
| SEC test files (`tests/export_xxe_test.rs`, etc.) | Unit tests in `src/cli/export.rs`; integration tests in `tests/export_integration.rs` | Consistent with existing codebase test organization |
| PRD: "No new data model structs" | `OscalModel` enum introduced as transient pipeline wrapper | Transient wrapper, not a persistent data entity; PRD statement referred to persistent storage |

**Authority**: For implementation, follow contracts/export.rs and this tasks.md. AR and PRD are upstream design documents that capture the original intent.
