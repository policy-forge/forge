# Tasks: YAML Output (WI-27)

**Input**: Design documents from `/specs/027-yaml-output/`
**Prerequisites**: plan.md (required), contracts/yaml_serializer.rs (required), research.md, data-model.md, quickstart.md
**Tests**: Required (TDD mandatory per constitution principle IV)
**Organization**: Tasks grouped by user story for independent implementation and testing

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## User Stories (from PRD 027-prd-yaml-output.md)

> **Numbering note:** Tasks.md user stories (US1-US4) are task-oriented groupings that do NOT correspond 1:1 with PRD user stories (PRD US-1 through US-4). PRD US-3 (forge export) is deferred to WI-29; PRD US-4 (stdout/file) is covered by US1/US2. Tasks.md US3 and US4 are derived from PRD requirements (M-3) and security review (SEC-1..SEC-5) respectively.

| Story | Title | Priority | PRD Trace | PRD User Story | Independent Test |
|-------|-------|----------|-----------|----------------|------------------|
| US1 | YAML Catalog Convert | P1 (MVP) | M-1, M-4, M-5 | PRD US-1 | `forge convert policy.md --strategy catalog --format yaml` produces valid YAML with all OSCAL metadata |
| US2 | YAML Component Convert | P1 | M-2, M-4, M-5 | PRD US-2 | `forge convert policy.md --strategy component --format yaml` produces valid YAML |
| US3 | Semantic Equivalence | P2 | M-3 | PRD US-1 AC-2, US-2 AC-2 | JSON and YAML outputs deserialize to identical `serde_json::Value` |
| US4 | Security Verification | P3 | SEC-1..SEC-5 | — | Adversarial inputs produce safe, properly quoted YAML with no type tags |

**Note**: US1 and US2 are implemented together in Phase 3 (same pipeline changes enable both strategies). PRD US-3 (forge export) is deferred to WI-29 (W-5). PRD US-4 (stdout/file) is covered by US1/US2 via existing `write_output` infrastructure.

---

## Phase 1: Setup

**Purpose**: Create module structure for YAML serializer

- [X] T001 Create `src/export/yaml.rs` with module-level doc comment skeleton and placeholder function signatures that return `todo!()` per contract in `specs/027-yaml-output/contracts/yaml_serializer.rs`
- [X] T002 Update `src/export/mod.rs` to declare `pub mod yaml;` and re-export `yaml::serialize_to_yaml` and `yaml::deserialize_from_yaml`

---

## Phase 2: Foundational — YAML Serializer Module (TDD)

**Purpose**: Implement the core YAML serialization functions that ALL user stories depend on

**CRITICAL**: No user story work can begin until this phase is complete

### Tests (RED)

- [X] T003 Write unit tests for `serialize_to_yaml` and `deserialize_from_yaml` in `tests/yaml_serializer_test.rs` — test cases: (1) serialize a simple struct to YAML string, (2) deserialize YAML string back to `serde_json::Value`, (3) round-trip serialize then deserialize produces equivalent Value, (4) serialize error returns `ForgeError::Serialization`, (5) deserialize invalid YAML returns `ForgeError::Serialization`. Use `serde_json::json!` macro for test fixtures. Verify tests FAIL (RED) against `todo!()` stubs.

### Implementation (GREEN)

- [X] T004 Implement `serialize_to_yaml<T: Serialize>` and `deserialize_from_yaml<T: DeserializeOwned>` in `src/export/yaml.rs` per contract `specs/027-yaml-output/contracts/yaml_serializer.rs` — wrap `serde_yaml::to_string()` and `serde_yaml::from_str()` with `ForgeError::Serialization` error mapping
- [X] T005 Run `cargo test --test yaml_serializer_test` to verify all unit tests pass (GREEN)

**Checkpoint**: YAML serializer module complete and tested. User story implementation can begin.

---

## Phase 3: User Stories 1 & 2 — YAML Convert for Catalog and Component (Priority: P1) MVP

**Goal**: Enable `forge convert --format yaml` for both catalog (US1/PRD US-1) and component (US2/PRD US-2) strategies

**Independent Test**: `cargo run -- convert tests/fixtures/sample_policy.md --strategy catalog --format yaml` produces valid YAML output to stdout; `--output /tmp/out.yaml` writes a valid YAML file

### Tests (RED)

- [X] T006 [P] [US1] Write CLI integration tests for `--format yaml` catalog conversion in `tests/cli_integration.rs` — test cases: (1) stdout output starts with valid YAML (no JSON braces), (2) file output with `--output` creates a parseable YAML file, (3) YAML output contains expected OSCAL catalog keys (`catalog`, `uuid`, `metadata`), (4) verify AC-7: all OSCAL required metadata fields present (`uuid`, `title`, `last-modified`, `version`, `oscal-version`). Use existing test fixture patterns from the file. Verify tests FAIL (RED).
- [X] T007 [P] [US2] Write CLI integration tests for `--format yaml` component conversion in `tests/cli_integration.rs` — test cases: (1) stdout YAML output, (2) file YAML output, (3) contains expected component-definition keys, (4) verify AC-7: all OSCAL required metadata fields present, (5) verify EC-4: `--output` to non-existent directory produces descriptive filesystem error. Verify tests FAIL (RED).

### Implementation (GREEN)

- [X] T008 [US1] [US2] Rename `json` parameter to `content` in `write_output` function signature and doc comment in `src/pipeline.rs:21` — no behavior change, just parameter naming for format-agnosticism (R-8)
- [X] T009 [US1] Refactor `run_catalog_pipeline` in `src/pipeline.rs:101`: (1) add `format: &crate::cli::OutputFormat` parameter, (2) replace `serde_json::to_string_pretty(&envelope)` + `serde_json::from_str(&json)` with `serde_json::to_value(&envelope)` for validation, (3) after validation passes add format dispatch: `OutputFormat::Json` calls `serde_json::to_string_pretty(&envelope)`, `OutputFormat::Yaml` calls `crate::export::yaml::serialize_to_yaml(&envelope)`, `OutputFormat::Xml` returns `Err(ForgeError::Validation("XML output format is not yet supported"))`, (4) pass serialized string to `write_output`
- [X] T010 [US2] Refactor `run_component_pipeline` in `src/pipeline.rs:189`: same changes as T009 — add `format: &crate::cli::OutputFormat` parameter, replace JSON roundtrip with `serde_json::to_value` for validation, add format dispatch after validation
- [X] T011 [US1] [US2] Update `src/cli/convert.rs`: (1) remove the non-JSON guard at line 24 (`if !matches!(format, OutputFormat::Json)`), (2) pass `format` as last argument to `run_catalog_pipeline` call at line 31, (3) pass `format` as last argument to `run_component_pipeline` call at line 64
- [X] T012 [US1] [US2] Update all existing call sites and tests that invoke `run_catalog_pipeline` or `run_component_pipeline` to pass the new `format` parameter — check `tests/catalog_pipeline_test.rs`, `tests/component_pipeline_test.rs`, `tests/cli_integration.rs`, `tests/e2e_release_test.rs`, `tests/golden_file_tests.rs`, and any other files calling these functions. Pass `&OutputFormat::Json` to preserve existing behavior.
- [X] T013 [US1] [US2] Run `cargo test` to verify all existing tests still pass AND new `--format yaml` integration tests pass (GREEN)

**Checkpoint**: `forge convert --format yaml` works for both catalog and component strategies (US1 + US2). MVP complete.

---

## Phase 4: User Story 3 — Semantic Equivalence (Priority: P2)

**Goal**: Verify that JSON and YAML outputs are semantically identical for the same input (PRD M-3)

**Independent Test**: Serialize the same OSCAL model to both JSON and YAML, deserialize both to `serde_json::Value`, assert structural equality

### Tests

- [X] T014 [P] [US3] Write catalog semantic equivalence test in `tests/yaml_equivalence_test.rs` — build a `CatalogEnvelope` using existing pipeline helpers, serialize to JSON via `serde_json::to_string_pretty`, serialize to YAML via `serialize_to_yaml`, parse JSON to `serde_json::Value` via `serde_json::from_str`, parse YAML to `serde_json::Value` via `deserialize_from_yaml`, assert both Values are equal (R-4). Also verify edge cases: (a) empty collections serialize consistently, (b) Unicode text preserved, (c) null/None optional fields handled identically in both formats (PRD EC-1, EC-2, EC-7).
- [X] T015 [P] [US3] Write component definition semantic equivalence test in `tests/yaml_equivalence_test.rs` — same pattern as T014 but using `ComponentDefinitionEnvelope` (includes `Vec<serde_json::Value>` field `control_implementations` to verify R-2 cross-format fidelity). Also verify edge case: deeply nested structures serialize consistently (PRD EC-3).

**Checkpoint**: Semantic equivalence between JSON and YAML verified for both OSCAL model types.

---

## Phase 5: User Story 4 — Security Verification (Priority: P3)

**Goal**: Verify YAML output is safe — no type injection, proper character handling, no information leakage (SEC-1 through SEC-5)

**Independent Test**: Serialize models containing adversarial strings (YAML-special chars, boolean-like words, tag-like patterns), verify output is safe

### Tests

- [X] T016 [P] [US4] Write SEC-1 test in `tests/yaml_security_test.rs`: serialize a `CatalogEnvelope` and `ComponentDefinitionEnvelope` to YAML, assert output does NOT contain `!!` (YAML type tags). Verifies no language-specific tag injection (R-6).
- [X] T017 [P] [US4] Write SEC-2 test in `tests/yaml_security_test.rs`: create an OSCAL model with control titles/descriptions containing YAML-special characters (`:`, `#`, `[`, `]`, `{`, `}`, `---`, `...`), serialize to YAML, verify output is valid YAML that parses back correctly (R-5).
- [X] T018 [P] [US4] Write SEC-3 test in `tests/yaml_security_test.rs`: create an OSCAL model with control titles containing boolean-like words (`yes`, `no`, `true`, `false`, `on`, `off`), serialize to YAML, deserialize to `serde_json::Value`, verify these remain as string scalars (not coerced to YAML booleans).
- [X] T019 [P] [US4] Write SEC-4 test in `tests/yaml_security_test.rs`: serialize an OSCAL model containing adversarial input (strings with embedded YAML directives, multi-line content, unicode edge cases) to both JSON and YAML, deserialize both to `serde_json::Value`, assert structural equality.

**Checkpoint**: All security requirements (SEC-1 through SEC-4) verified. SEC-5 (use serde_yaml::to_string only) is enforced by design — the implementation in `src/export/yaml.rs` calls only `serde_yaml::to_string()`.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Logging, formatting, final verification

- [X] T020 Add `tracing::debug!` for YAML serialization events in `src/export/yaml.rs` and `tracing::info!` for format selection in `src/pipeline.rs` run_catalog_pipeline and run_component_pipeline (PRD S-2 verbose logging)
- [X] T021 [P] Add criterion benchmark for YAML serialization in `benches/pipeline_benchmark.rs` — benchmark `serialize_to_yaml` for both `CatalogEnvelope` and `ComponentDefinitionEnvelope` using realistic test fixtures, compare against JSON serialization time to verify YAML is within same order of magnitude (constitution principle VI, plan performance goal: < 100ms)
- [X] T022 Run `cargo fmt --check` and `cargo clippy -- -D warnings` — fix any formatting or lint issues across all modified and new files
- [X] T022a Verify PRD S-1 (2-space indent): confirm YAML integration test output uses 2-space indentation (serde_yaml_ng default). No custom formatting — reliance on serde_yaml defaults is by design per AR guardrail.
- [X] T023 Run full test suite `cargo test` — verify all tests pass (unit, integration, equivalence, security, CLI)
- [X] T024 Run quickstart.md manual verification steps: (1) `cargo run -- convert tests/fixtures/sample_policy.md --strategy catalog --format yaml`, (2) verify output with `python3 -c "import yaml; ..."`, (3) compare JSON and YAML structural equality

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup — BLOCKS all user stories
- **US1+US2 (Phase 3)**: Depends on Foundational — core implementation
- **US3 (Phase 4)**: Depends on US1+US2 (needs working YAML serialization for equivalence testing)
- **US4 (Phase 5)**: Depends on US1+US2 (needs working YAML serialization for security testing)
- **Polish (Phase 6)**: Depends on US1+US2; US3 and US4 can overlap

### User Story Dependencies

- **US1 + US2 (P1)**: Depend on Phase 2 only — no dependencies on other stories. **This is the MVP.** Implemented together in Phase 3.
- **US3 (P2)**: Depends on US1+US2 being functional (needs both JSON and YAML pipelines working for equivalence testing)
- **US4 (P3)**: Depends on US1+US2 being functional (needs YAML output to verify security properties)
- **US3 and US4 can run in parallel** — they test independent properties of the same output

### Within Each Phase

- Tests MUST be written and FAIL (RED) before implementation begins
- Implementation makes tests pass (GREEN)
- Verify at each checkpoint before proceeding

### Parallel Opportunities

- T001, T002 are sequential (T002 depends on T001)
- T006, T007 can run in parallel (different test sections, same file but independent)
- T009, T010 can be done in parallel (different functions in same file, same pattern)
- T014, T015 can run in parallel (different test functions, same file)
- T016, T017, T018, T019 can ALL run in parallel (independent test functions)
- US3 and US4 can run in parallel after US1+US2 completes

---

## Parallel Example: User Story 1

```text
# After Phase 2 checkpoint, launch US1 tests in parallel:
Agent 1: T006 — CLI integration test for catalog YAML in tests/cli_integration.rs
Agent 2: T007 — CLI integration test for component YAML in tests/cli_integration.rs

# Then implement pipeline changes (sequential — same file):
T008 → T009 → T010 → T011 → T012 → T013
```

## Parallel Example: US3 + US4 After US1+US2

```text
# After US1+US2 checkpoint, launch US3 and US4 in parallel:
Agent 1: T014 + T015 — Semantic equivalence tests (US3)
Agent 2: T016 + T017 + T018 + T019 — Security tests (US4)
```

---

## Implementation Strategy

### MVP First (US1 + US2)

1. Complete Phase 1: Setup (T001-T002)
2. Complete Phase 2: Foundational YAML serializer (T003-T005)
3. Complete Phase 3: US1+US2 YAML Convert for both strategies (T006-T013)
4. **STOP and VALIDATE**: Run `forge convert --format yaml` manually for both catalog and component
5. Feature is usable at this point

### Incremental Delivery

1. Setup + Foundational -> YAML serializer ready
2. US1+US2 -> `--format yaml` works for both strategies -> MVP deployable
3. US3 -> Equivalence verified -> Confidence in data fidelity
4. US4 -> Security verified -> Production-ready
5. Polish -> Clean, documented, benchmarked, fully validated

### Key Constraints (from AR Guardrails)

- **DO NOT** write custom YAML formatting logic — use `serde_yaml::to_string()` only
- **DO NOT** add YAML-specific `#[serde]` attributes to OSCAL model structs
- **DO NOT** verify semantic equivalence via string comparison — compare deserialized `serde_json::Value`
- **DO NOT** implement round-trip testing — deferred to WI-28
- **MUST** handle all `serde_yaml` errors with `ForgeError::Serialization`
- **MUST** validate via `serde_json::Value` before serializing to any output format

---

## Notes

- [P] tasks = different files or independent sections, no dependencies
- [Story] label maps task to specific user story for traceability (US1-US4 align with PRD US-1 through US-4)
- TDD is mandatory (constitution IV) — every phase follows RED → GREEN flow
- SEC-5 (no custom formatting) is enforced by design, not by test
- PRD W-5 (`forge export --format yaml`, formerly M-4) is DEFERRED to WI-29 per research R-7
- PRD W-6 (multiline block scalars, formerly S-2) deferred per AR guardrail conflict
- `serde_yaml_ng` v0.10 is already in `Cargo.toml` — no new dependency needed
- All OSCAL model structs already derive `Serialize` — no model changes required
- OSCAL model structs do NOT derive `PartialEq` — semantic equivalence uses `serde_json::Value` comparison (R-4)
