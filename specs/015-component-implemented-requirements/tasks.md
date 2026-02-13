# Tasks: Component Implemented Requirements (WI-15)

**Input**: Design documents from `/specs/015-component-implemented-requirements/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/interfaces.rs

**Tests**: TDD mandatory (Constitution IV). Test tasks are included and MUST be written first (RED), then implementation (GREEN).

**Organization**: Tasks follow the implementation order from quickstart.md, tagged with user stories. US1+US3 share the core builder; US2 covers CLI integration. US4 (P2) is out of scope for this iteration.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- Include exact file paths in descriptions

## User Story Reference

| Story | Title | Priority | Spec Section |
|-------|-------|----------|-------------|
| US1 | Map Policy Requirements to Control IDs | P1 | spec.md US1 |
| US2 | Source Profile Reference in Control Implementations | P1 | spec.md US2 |
| US3 | Deterministic UUIDs for Implemented Requirements | P1 | spec.md US3 |
| US4 | Implementation Narrative with Source Context | P2 | spec.md US4 (deferred — raw text at P1 scope per clarification) |

---

## Phase 1: Foundational (Blocking Prerequisites)

**Purpose**: Infrastructure changes that ALL user stories depend on — UUID namespaces, visibility fixes, and module skeleton.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T001 [P] Compute and add `CONTROL_IMPL_NAMESPACE` and `IMPL_REQ_NAMESPACE` UUID v5 namespace constants to `src/uuid.rs` using the pattern `Uuid::new_v5(&FORGE_NAMESPACE_UUID, b"control-implementation")` and `Uuid::new_v5(&FORGE_NAMESPACE_UUID, b"implemented-requirement")` — store as hardcoded byte arrays matching the existing `BACK_MATTER_NAMESPACE` and `COMPONENT_NAMESPACE` pattern (see contracts/interfaces.rs for seed strings)
- [x] T002 [P] Change `resolve_abbreviation` visibility from private `fn` to `pub(crate) fn` in `src/oscal/catalog.rs` (line ~344) so the Component builder can reuse section abbreviation collision resolution — no logic change, visibility only
- [x] T003 Create `src/oscal/implemented_requirements.rs` module file with placeholder function signatures from `contracts/interfaces.rs` (build_control_implementations, map_requirement_to_implemented, generate_control_impl_uuid, generate_impl_req_uuid, derive_control_id_or_fallback) and register the module in `src/oscal/mod.rs` — include rustdoc comments on all `pub` items per Constitution III

**Checkpoint**: Foundation ready — `cargo build` should compile with placeholder `todo!()` bodies. User story implementation can now begin.

---

## Phase 2: US1+US3 — Core Builder: Map Requirements + Deterministic UUIDs (Priority: P1) 🎯 MVP

**Goal**: Implement `build_control_implementations` that walks document sections depth-first (same order as Catalog builder), maps each `PolicyRequirement` to an `implemented-requirement` with a deterministic UUID v5 and a `control-id` matching the Catalog builder's scheme (e.g., `POL-AC-001`).

**Independent Test**: Build control-implementations from a PolicyDocument with 5 requirements across 2 sections, verify 5 implemented-requirements are produced with valid UUIDs, correct control-ids, and raw requirement text as descriptions.

**Key Design Decisions** (from research.md):
- R1: Walk sections depth-first using `generate_section_abbreviation` + `resolve_abbreviation` + `generate_control_id` — same as Catalog builder
- R2: UUID seeds — control-impl: `"{source_profile}\0{policy_title}"`, impl-req: `"{stable_id}\0{text}\0{index}"`
- Receives `&PolicyDocument` (not `&[PolicyRequirement]`) for section context access

### Tests for US1+US3 (TDD RED — write these FIRST, ensure they FAIL)

- [x] T004 [P] [US3] Write unit tests for `generate_control_impl_uuid` and `generate_impl_req_uuid` in `src/oscal/implemented_requirements.rs`: verify determinism (same inputs → same UUID), verify different inputs → different UUIDs, verify UUID format is valid v5
- [x] T005 [P] [US1] Write unit tests for `derive_control_id_or_fallback` in `src/oscal/implemented_requirements.rs`: verify normal case produces `POL-{ABBR}-{NNN}` format via `generate_control_id`, verify fallback when `has_stable_id == false` produces `REQ-{zero-padded global_index}` (e.g., `REQ-001`)
- [x] T006 [US1] Write unit tests for `map_requirement_to_implemented` in `src/oscal/implemented_requirements.rs`: verify returned JSON has `uuid`, `control-id`, and `description` fields; verify `description` is the raw requirement text (FR-008); verify `control-id` matches expected format
- [x] T007 [US1] Write unit tests for `build_control_implementations` in `src/oscal/implemented_requirements.rs`: verify returned JSON array has one entry; verify entry has `uuid`, `source`, `description`, and `implemented-requirements` fields; verify `source` equals provided source_profile; verify `description` follows `"Implementation narratives derived from {policy_title}."` pattern (FR-004, S-2); verify implemented-requirements count matches total PolicyRequirements across all sections

### Implementation for US1+US3 (TDD GREEN)

- [x] T008 [P] [US3] Implement `generate_control_impl_uuid` and `generate_impl_req_uuid` in `src/oscal/implemented_requirements.rs` using `Uuid::new_v5` with `CONTROL_IMPL_NAMESPACE` / `IMPL_REQ_NAMESPACE` and the seed formats from data-model.md
- [x] T009 [P] [US1] Implement `derive_control_id_or_fallback` in `src/oscal/implemented_requirements.rs`: call `generate_control_id(abbreviation, req_index_in_section, "POL")` for normal case; return `format!("REQ-{:03}", global_index + 1)` for fallback when `has_stable_id == false`
- [x] T010 [US1] Implement `map_requirement_to_implemented` in `src/oscal/implemented_requirements.rs`: construct `serde_json::json!` object with `uuid` (from `generate_impl_req_uuid`), `control-id`, and `description` (raw requirement text, or placeholder `"No implementation narrative available."` if empty per FR-014)
- [x] T011 [US1] Implement `build_control_implementations` in `src/oscal/implemented_requirements.rs`: walk `document.sections` depth-first, call `generate_section_abbreviation` + `resolve_abbreviation` (from catalog.rs) per section, iterate requirements calling `map_requirement_to_implemented` with generated control-id, assemble JSON array with one `control-implementations` entry containing `uuid` (from `generate_control_impl_uuid`), `source`, `description`, and all `implemented-requirements`

**Checkpoint**: `cargo test` passes for all unit tests in `implemented_requirements.rs`. Core mapping logic is complete and independently testable.

---

## Phase 3: US2 — Source Profile Reference + CLI Integration (Priority: P1)

**Goal**: Add `--source-profile` CLI flag, modify `build_component_definition` to accept it, create `run_component_pipeline`, and wire the component strategy to replace the placeholder error.

**Independent Test**: Run `forge convert policy.md --strategy component --source-profile ./baselines/nist.json` and verify the output Component Definition JSON has `control-implementations[0].source == "./baselines/nist.json"` with populated `implemented-requirements`.

**Key Design Decisions** (from research.md):
- R3: Add `source_profile: Option<&str>` to `build_component_definition` — `None` preserves WI-14 behavior, `Some` injects control-implementations
- R4: `run_component_pipeline` mirrors `run_catalog_pipeline` stages 1-9, diverges at stage 10
- R5: Runtime validation in `convert.rs` (not clap-level) for descriptive error messages

### Tests for US2 (TDD RED)

- [x] T012 [US2] Write unit tests for `build_component_definition` with `source_profile` parameter in `src/oscal/component_definition.rs`: verify `None` produces empty `control-implementations` (backward compatible with WI-14 tests); verify `Some("./baseline.json")` produces populated `control-implementations` with correct `source` field

### Implementation for US2

- [x] T013 [US2] Modify `build_component_definition` signature to accept `source_profile: Option<&str>` in `src/oscal/component_definition.rs`: when `Some`, call `build_control_implementations` and assign to `component.control_implementations`; when `None`, keep empty vec
- [x] T014 [US2] Update all existing callers of `build_component_definition` to pass `None` for `source_profile` — ~19 test call sites in `src/oscal/component_definition.rs`, plus re-exports in `src/oscal/mod.rs` and `src/lib.rs`; all must pass `None` as second argument to maintain backward compatibility
- [x] T015 [P] [US2] Add `--source-profile` field as `Option<String>` with `#[arg(long)]` to `Commands::Convert` in `src/cli/mod.rs`
- [x] T016 [US2] Add `run_component_pipeline` function in `src/pipeline.rs` mirroring `run_catalog_pipeline` stages 1-9 (ingest through citation extraction), then stage 10: call `build_component_definition(document, Some(source_profile))`, serialize to JSON, and write output
- [x] T017 [US2] Wire component strategy in `src/cli/convert.rs`: add runtime validation (strategy == Component && source_profile.is_none() → `ForgeError::Validation("--source-profile is required when using --strategy component")`; strategy == Component && source_profile == Some("") → `ForgeError::Validation("--source-profile must not be empty")`); replace placeholder error with `run_component_pipeline` call

**Checkpoint**: `forge convert policy.md --strategy component --source-profile ./baseline.json` produces valid Component Definition JSON with populated control-implementations. All existing `--strategy catalog` tests still pass.

---

## Phase 4: Edge Cases & Validation (EC-1 through EC-5, SEC-1 through SEC-5)

**Purpose**: Handle all edge cases from spec.md and security requirements from SEC review. Each task includes both the test (RED) and the handler (GREEN).

- [x] T018 [P] [US1] Add test and `tracing::warn!` for zero requirements producing empty `implemented-requirements` array in `src/oscal/implemented_requirements.rs` — given a PolicyDocument with zero PolicyRequirements, `build_control_implementations` returns a valid array with one entry whose `implemented-requirements` is empty, plus a warning log (EC-1, FR-013)
- [x] T019 [P] [US1] Add test and placeholder handler for empty requirement text in `src/oscal/implemented_requirements.rs` — given a PolicyRequirement with `text == ""`, `map_requirement_to_implemented` sets `description` to `"No implementation narrative available."` (EC-3, FR-014, SEC-5)
- [x] T020 [P] [US1] Add test and fallback for missing `stable_id` in `src/oscal/implemented_requirements.rs` — given a PolicyRequirement with `stable_id == None`, `derive_control_id_or_fallback` returns `REQ-{zero-padded global_index + 1}` and emits `tracing::warn!` (EC-2)
- [x] T021 [P] [US2] Add test and validation for empty `--source-profile` string in `src/cli/convert.rs` — given `--source-profile ""` with `--strategy component`, the CLI exits with `ForgeError::Validation("--source-profile must not be empty")` (EC-4, SEC-3, SEC-4)
- [x] T022 [US3] Add test for identical text at different positions producing distinct UUIDs in `src/oscal/implemented_requirements.rs` — given two PolicyRequirements with identical `text` but different `atom_index`, verify their implemented-requirement UUIDs differ (EC-5, FR-012)
- [x] T023 [US1] Add test verifying `description` field uses raw requirement text and NOT `remarks` field in `src/oscal/implemented_requirements.rs` — verify no `remarks` key in output JSON (SEC-1, SEC-2)

**Checkpoint**: All edge cases handled. `cargo test` passes. Zero requirements, empty text, missing stable_id, empty source-profile, and identical-text-different-position scenarios are all covered.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Integration testing, cross-artifact consistency verification, and final quality checks.

- [x] T024 Create integration test in `tests/component_pipeline_test.rs` that exercises the full component pipeline end-to-end: read a test Markdown policy fixture, run through `run_component_pipeline`, verify output JSON structure matches OSCAL Component Definition with populated `control-implementations` and `implemented-requirements`
- [x] T025 Add cross-artifact consistency test verifying control-ids match between `build_catalog` and `build_control_implementations` for the same `PolicyDocument` — use a shared test fixture where all requirements have `stable_id` present, generate both Catalog and Component Definition, extract control-ids, assert they are identical (note: EC-2 fallback cases where `stable_id` is `None` are excluded from this assertion since the fallback format `REQ-{index}` intentionally differs from the Catalog's section-based scheme)
- [x] T026 Run `cargo test --workspace`, `cargo clippy -- -D warnings`, and `cargo fmt --check` — fix any failures, warnings, or formatting violations

---

## Dependencies & Execution Order

### Phase Dependencies

- **Foundational (Phase 1)**: No dependencies — can start immediately
- **US1+US3 (Phase 2)**: Depends on Phase 1 completion (needs UUID namespaces, pub(crate) resolve_abbreviation, module skeleton)
- **US2 (Phase 3)**: Depends on Phase 2 completion (needs build_control_implementations to exist before injecting into component_definition)
- **Edge Cases (Phase 4)**: Depends on Phase 2 and Phase 3 completion (tests exercise code from both phases)
- **Polish (Phase 5)**: Depends on all previous phases

### User Story Dependencies

- **US1 (Map Requirements)**: Core story — Phase 2 implements mapping logic, Phase 3 integrates it, Phase 4 handles edge cases
- **US2 (Source Profile)**: Depends on US1 — needs build_control_implementations before wiring CLI
- **US3 (Deterministic UUIDs)**: Co-implemented with US1 — UUID generation is part of the mapping function
- **US4 (P2)**: Deferred — raw text narrative for P1, section-context prefix deferred per clarification

### Within Each Phase

- Tests (TDD RED) MUST be written and FAIL before implementation (TDD GREEN)
- Tasks marked [P] within a phase can run in parallel
- Non-[P] tasks depend on preceding tasks in that phase

### Parallel Opportunities

- Phase 1: T001, T002 can run in parallel (different files)
- Phase 2 Tests: T004, T005 can run in parallel (independent test functions)
- Phase 2 Impl: T008, T009 can run in parallel (independent functions)
- Phase 4: T018, T019, T020, T021 can run in parallel (independent edge cases)

---

## Parallel Example: Phase 2 (Core Builder)

```bash
# Launch tests in parallel (TDD RED):
Task: "Write UUID generation helper tests in src/oscal/implemented_requirements.rs" (T004)
Task: "Write derive_control_id_or_fallback tests in src/oscal/implemented_requirements.rs" (T005)

# Then sequential tests that depend on understanding of helpers:
Task: "Write map_requirement_to_implemented tests" (T006)
Task: "Write build_control_implementations tests" (T007)

# Launch independent implementations in parallel (TDD GREEN):
Task: "Implement UUID helpers" (T008)
Task: "Implement derive_control_id_or_fallback" (T009)

# Then sequential implementations:
Task: "Implement map_requirement_to_implemented" (T010)
Task: "Implement build_control_implementations" (T011)
```

---

## Implementation Strategy

### MVP First (Phase 1 + Phase 2 Only)

1. Complete Phase 1: Foundational infrastructure
2. Complete Phase 2: Core builder with tests
3. **STOP and VALIDATE**: `cargo test` passes, control-ids are correct, UUIDs are deterministic
4. This gives you a working `build_control_implementations` function even without CLI integration

### Incremental Delivery

1. Phase 1 → Foundation ready
2. Phase 2 → Core mapping works (unit testable) — **MVP**
3. Phase 3 → CLI integration works (end-to-end testable)
4. Phase 4 → All edge cases handled
5. Phase 5 → Integration tests, cross-artifact consistency, quality checks

### Key Verification Points

After Phase 2: `cargo test --lib` passes all implemented_requirements tests
After Phase 3: `forge convert policy.md --strategy component --source-profile ./baseline.json` produces valid JSON
After Phase 4: All edge cases from spec.md EC-1 through EC-5 are covered
After Phase 5: `cargo test --workspace && cargo clippy -- -D warnings && cargo fmt --check` all pass

---

## Notes

- [P] tasks = different files or independent functions, no dependencies
- [Story] label maps task to specific user story for traceability
- TDD is mandatory (Constitution IV) — write tests FIRST, ensure they FAIL, then implement
- US4 (P2) is explicitly deferred — raw requirement text at P1 scope per clarification session
- The AR proposed `control_id_from_stable_id` is **NOT used** — per research.md R1, control-ids must be section-aware, matching the Catalog builder's scheme
- `resolve_abbreviation` from `catalog.rs` is reused as `pub(crate)` — no logic duplication
- All UUIDs use dedicated namespaces: `CONTROL_IMPL_NAMESPACE` and `IMPL_REQ_NAMESPACE` (not shared with other element types)
