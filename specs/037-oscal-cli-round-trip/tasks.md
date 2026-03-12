# Tasks: oscal-cli Round-Trip Validation (WI-37)

**Input**: Design documents from `/specs/037-oscal-cli-round-trip/`
**Prerequisites**: plan.md ✅, spec.md ✅, prd.md ✅, ar.md ✅, sec.md ✅, data-model.md ✅, contracts/round_trip.rs ✅, quickstart.md ✅

**TDD**: Mandatory per constitution principle IV and PRD technical constraints. All unit tests are written first, must FAIL, then implementation makes them pass.

**Organization**: Tasks grouped by user story to enable independent implementation and testing.

---

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no shared dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- Exact file paths included in every task

---

## Phase 1: Setup

**Purpose**: Create the `round_trip` module skeleton so all subsequent phases have a home.

- [x] T001 Create `src/round_trip/` directory with empty stub files: `mod.rs`, `divergence.rs`, `rules.rs`, `comparator.rs`, `chain.rs`, `log.rs` — each file contains only a module-level doc comment
- [x] T002 Register `pub mod round_trip;` in `src/lib.rs` alongside existing modules (`oscal_cli`, `validate`, etc.)

**Checkpoint**: `cargo build` compiles without errors — empty module compiles successfully.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Data structures and oscal-cli extension that ALL user stories require. Nothing else can start until this phase is complete.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T003 Add `OscalFormat` enum (`Json`, `Xml`, `Yaml`) with `to_cli_flag()` method to `src/oscal_cli/mod.rs` (matches contract in `specs/037-oscal-cli-round-trip/contracts/round_trip.rs`)
- [x] T004 Add `ConvertArgs` struct (input_path, output_path, output_format: OscalFormat, timeout: Duration) and `ConvertResult` struct (output_path, warnings: Vec<String>) to `src/oscal_cli/mod.rs` — sequence after T003 (same file)
- [x] T005 Add `convert(&self, args: &ConvertArgs) -> Result<ConvertResult, ForgeError>` method to `OscalCliInvoke` trait in `src/oscal_cli/mod.rs` — add `ForgeError::OscalCliTimeout` variant if not yet present
- [x] T006 Implement `convert()` in `ProcessInvoker` in `src/oscal_cli/invoker.rs`: build argument array `["convert", "--to=<fmt>", "<input>", "<output>"]`, spawn with per-invocation timeout, check exit code, return `ConvertResult` with stderr warnings on success or `ForgeError` on failure (inherits SEC-036 subprocess security: argument arrays, no shell strings, absolute paths)
- [x] T007 [P] Implement `Divergence`, `DivergenceClass` (`ForgeFix`, `OscalCliDiff`, `Acceptable`), `ResolutionStatus` (`Fixed`, `Accepted`, `ReportedUpstream`), and `RoundTripResult` structs/enums in `src/round_trip/divergence.rs` with derives matching the contract: `Debug, Clone, Serialize, Deserialize, PartialEq` on `Divergence` (which includes `pub resolution: Option<ResolutionStatus>`); `Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq` on `DivergenceClass` and `ResolutionStatus`; `Debug, Serialize` on `RoundTripResult` — do NOT add `#[serde(skip_serializing_if)]` to `resolution`; `None` must serialize as `"resolution": null` so unresolved divergences are visible in the log (PRD M-6, AC-6)
- [x] T008 [P] Implement `OscalComparisonRules` struct with `Default` impl in `src/round_trip/rules.rs`: `unordered_array_paths = ["props", "links", "parts"]`, `ignored_paths = []`

**Checkpoint**: `cargo test --lib` passes (empty tests), `cargo build` compiles — all foundational types are available.

---

## Phase 3: User Story 1 — Verify FORGE Output Matches Reference Tool Conversion (Priority: P1) 🎯 MVP

**Goal**: Automated comparison of FORGE output against oscal-cli's canonical conversion, with integration tests for Catalog and Component Definition artifacts. Zero unresolved FORGE-caused divergences.

**Independent Test**: `cargo test round_trip` (unit tests, no oscal-cli needed) + `cargo test --test oscal_cli_round_trip` (integration tests, skips gracefully if oscal-cli absent). All unit tests green; integration tests either pass or skip.

### Unit Tests for User Story 1 (TDD — write first, verify FAIL before implementing)

- [x] T009 [P] [US1] Write unit tests for `compare_oscal_json` in `src/round_trip/comparator.rs`: (a) identical documents → empty divergence list; (b) documents differing only in JSON object field order → empty list; (c) scalar value difference → single divergence with correct `/json_path`, expected, actual; (d) missing key in actual → divergence; (e) extra key in actual → divergence; (f) unordered array (`props`) with reordered elements matched by `uuid` → empty list; (g) unordered array with element missing by `uuid` → divergence; (h) empty array `[]` in expected vs. absent key in actual → single divergence with `classification: Acceptable` (EC-2); (i) key present in expected but absent in actual where value is not recognized by oscal-cli → divergence captured with full path (EC-5) — verify all FAIL before T010
- [x] T010 [P] [US1] Write unit tests for `run_round_trip_chain` using an in-memory `MockInvoker` (stub that returns canned `ConvertResult` without spawning processes) in `src/round_trip/chain.rs`: (a) happy path — chain calls `convert()` exactly 3 times (JSON→XML, XML→YAML, YAML→JSON) and returns the final output path; (b) timeout hard-error — `MockInvoker` returns `Err(ForgeError::OscalCliTimeout)` on the second conversion step → `run_round_trip_chain` returns `Err(ForgeError::OscalCliTimeout)` (NOT a graceful skip; FR-010) — verify both FAIL before T011

### Implementation for User Story 1

- [x] T011 [US1] Implement `compare_oscal_json` in `src/round_trip/comparator.rs`: recursive `serde_json::Value` tree walk, track full RFC 6901 path, compare objects as unordered key sets, handle `unordered_array_paths` with `uuid`→`name`+`ns`→positional fallback matching, compare primitives by type and value — all T009 unit tests must pass
- [x] T012 [US1] Implement `run_round_trip_chain` in `src/round_trip/chain.rs`: create intermediate filenames in `temp_dir` (artifact.xml, artifact.yaml, artifact-rt.json), call `invoker.convert()` three times with independent `timeout` per invocation (SEC-6), use `tempfile`-owned cleanup (SEC-2, SEC-3, SEC-4), log each step at DEBUG level — T010 unit test must pass
- [x] T013 [US1] Add public re-exports to `src/round_trip/mod.rs` for Phase-3-available symbols only: `pub use divergence::{Divergence, DivergenceClass, ResolutionStatus, RoundTripResult}; pub use rules::OscalComparisonRules; pub use comparator::compare_oscal_json; pub use chain::run_round_trip_chain;` — do NOT add `write_divergence_log` here; that re-export is added in T017 once the function exists (forward-referencing a non-existent function causes a compile error)
- [x] T014 [US1] Create `tests/oscal_cli_round_trip.rs` with helper `fn skip_if_no_oscal_cli() -> Option<ProcessInvoker>` using `PathDetector`; write integration tests: `catalog_json_xml_yaml_json_round_trip` (generate Catalog via FORGE pipeline, run `run_round_trip_chain`, call `compare_oscal_json` with `OscalComparisonRules::default()`, assert zero `ForgeFix`/`OscalCliDiff` divergences); `component_json_xml_yaml_json_round_trip` (same for Component Definition); `round_trip_skip_when_oscal_cli_unavailable` (mock unavailable detector, verify returns early without failure) — covers S-2 and FR-009 (both require conditional skip; same behavior, one task) (SEC-5, SC-001, SC-002, SC-005)
- [x] T015 [US1] Run `cargo test --test oscal_cli_round_trip`, inspect all divergences in output, fix any FORGE-caused divergences (`ForgeFix` classification) in the FORGE JSON serialization layer (`src/oscal/` serializers or equivalent output formatters) — **scope constraint**: fixes are limited to FORGE's own serialization code; do not modify the comparator or reclassify divergences to hide them; **stopping condition**: if a `ForgeFix` divergence cannot be resolved within a reasonable investigation window, escalate to `OscalCliDiff` or `Acceptable` only after confirming this via the OSCAL spec, and document the reasoning in the divergence `description`; repeat until SC-001 and SC-002 satisfied (FR-005/M-5)

**Checkpoint**: `cargo test round_trip` (unit) and `cargo test --test oscal_cli_round_trip` (integration) all pass or skip — User Story 1 is independently verifiable.

---

## Phase 4: User Story 2 — Document and Classify Divergences (Priority: P1)

**Goal**: All discovered divergences written to a structured JSON divergence log with classification and resolution status. Log writer is unit-tested with insta snapshots.

**Independent Test**: Run `cargo test round_trip::log` to verify log writer unit tests pass. After integration tests, confirm `divergences.json` contains all divergences with `json_path`, `expected`, `actual`, `classification`, and `description`.

### Unit Tests for User Story 2 (TDD — write first, verify FAIL before implementing)

- [x] T016 [P] [US2] Write unit test for `write_divergence_log` with `insta` snapshot in `src/round_trip/log.rs`: (a) clean pass result (`passed: true`, empty divergences) → snapshot of written JSON; (b) result with one `ForgeFix` divergence (`resolution: Some(Fixed)`) and one `Acceptable` divergence (`resolution: Some(Accepted)`) → snapshot includes both `classification` and `resolution` fields (PRD M-6, AC-6); (c) result with `resolution: None` divergence → snapshot shows `"resolution": null` — snapshots stored at `src/round_trip/snapshots/`; run `cargo insta review` to accept baseline; verify all FAIL before T017

### Implementation for User Story 2

- [x] T017 [US2] Implement `write_divergence_log(result: &RoundTripResult, output_path: &Path) -> Result<(), ForgeError>` in `src/round_trip/log.rs`: serialize `RoundTripResult` via `serde_json::to_writer_pretty`, create or overwrite file at `output_path`, return `ForgeError::Io` on failure — T016 insta snapshots (in `src/round_trip/snapshots/`) must pass after `cargo insta review` accepts them; commit snapshot files alongside implementation; **also** add `pub use log::write_divergence_log;` to `src/round_trip/mod.rs` (deferred from T013 — safe to add now that the function exists)
- [x] T018 [US2] Inspect `divergences.json` output from integration tests; add a `reclassify` helper in `tests/oscal_cli_round_trip.rs` that post-processes the `Vec<Divergence>` returned by `compare_oscal_json` — for each divergence, override `classification` (if the initial `ForgeFix` is wrong after investigation) and set `resolution` (`Some(Fixed)` after FORGE fix, `Some(Accepted)` for acceptable variations, `Some(ReportedUpstream)` for oscal-cli differences) with a descriptive `description`; reclassification is a manual annotation in the test body, NOT in the comparator (S-3/FR-008, M-6/AC-6) — call `reclassify` BEFORE passing divergences to `write_divergence_log`
- [x] T019 [US2] Update `catalog_json_xml_yaml_json_round_trip` and `component_json_xml_yaml_json_round_trip` integration tests in `tests/oscal_cli_round_trip.rs` to call `write_divergence_log` (after `reclassify` from T018) with a `tempfile`-scoped output path; assert: file exists, is valid JSON, each divergence entry contains `json_path`, `expected`, `actual`, `classification`, `description`, and `resolution` fields — `resolution` may be `null` for auto-classified `Acceptable` divergences (EC-2), but must be non-null for all `ForgeFix` and `OscalCliDiff` entries (SC-004 satisfied when zero unresolved `ForgeFix` remain)

**Checkpoint**: `cargo test round_trip::log` passes; integration tests write a valid `divergences.json` with all divergences classified — User Story 2 is independently verifiable.

---

## Phase 5: User Story 3 — Round-Trip Across All Three OSCAL Formats (Priority: P2)

**Goal**: Explicitly confirm that the JSON → XML → YAML → JSON three-format round-trip produces zero unresolved FORGE-caused divergences for both Catalog and Component Definition (SC-003).

**Independent Test**: `cargo test --test oscal_cli_round_trip` — test names `catalog_json_xml_yaml_json_round_trip` and `component_json_xml_yaml_json_round_trip` pass (or skip gracefully when oscal-cli unavailable).

- [x] T020 [P] [US3] Add a `MockInvoker`-based unit test in `src/round_trip/chain.rs` that verifies all three intermediate files are created in `temp_dir` with correct extensions (`artifact.xml`, `artifact.yaml`, `artifact-rt.json`) — this makes US3 independently testable without oscal-cli; then confirm `catalog_json_xml_yaml_json_round_trip` integration test passes (or skips) and SC-003 is met; fix any newly discovered YAML-step divergences
- [x] T021 [P] [US3] Confirm `component_json_xml_yaml_json_round_trip` integration test passes (or skips) for SC-003 — investigate and fix any YAML-step-specific divergences not caught in Phase 3; update `reclassify` helper in `tests/oscal_cli_round_trip.rs` if new divergences are discovered
- [x] T022 [US3] Log INFO-level summary after each integration test run in `tests/oscal_cli_round_trip.rs`: emit divergence count, classification breakdown (ForgeFix/OscalCliDiff/Acceptable counts), and pass/fail status — confirms SC-003, SC-004 observability requirement

**Checkpoint**: All three-format integration tests pass (or skip gracefully); `divergences.json` shows zero ForgeFix divergences for both artifact types — SC-003 satisfied.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Quality gates and final verification per quickstart.md validation scenarios.

- [x] T023 [P] Run `cargo clippy -- -D warnings` and resolve all warnings in new/modified files (`src/round_trip/*.rs`, `src/oscal_cli/mod.rs`, `src/oscal_cli/invoker.rs`, `tests/oscal_cli_round_trip.rs`)
- [x] T024 [P] Run `cargo fmt --check` and apply `cargo fmt` to all new/modified files — zero formatting violations
- [x] T025 Run `cargo test` (all unit + integration tests) and confirm: all unit tests green; integration tests pass or skip gracefully; `insta` snapshots are committed
- [x] T026 Execute all quickstart.md validation scenarios manually: `cargo test --lib` (unit only), `cargo test --test oscal_cli_round_trip` (integration), inspect `divergences.json` with `jq .` — confirm SC-001 through SC-005 are all satisfied

**Final Checkpoint**: All validation criteria from `plan.md` pass — `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`, SC-001 through SC-005.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 — **BLOCKS all user stories**
- **US1 (Phase 3)**: Depends on Phase 2 — no dependency on US2 or US3
- **US2 (Phase 4)**: Depends on Phase 2 — integrates with US1 integration tests but log writer is independently testable
- **US3 (Phase 5)**: Depends on Phase 3 (chain already does 3 steps) — validates the 3-format path
- **Polish (Phase 6)**: Depends on all user story phases complete

### User Story Dependencies

- **US1 (P1)**: Can start immediately after Phase 2 — no dependency on US2/US3
- **US2 (P1)**: Can start immediately after Phase 2 — `write_divergence_log` is independently unit-testable; T018/T019 require US1 integration tests to exist
- **US3 (P2)**: Depends on US1 Phase 3 complete (chain and integration test infrastructure already built)

### Within Each User Story (TDD Order)

1. Write unit tests → verify they **FAIL**
2. Implement to make tests **PASS**
3. Write integration tests
4. Run integration tests; fix FORGE-caused divergences
5. Story complete → move to next

### Parallel Opportunities (within phases)

- T003 → T004: **Sequential** (both modify `src/oscal_cli/mod.rs`; same file, no parallel)
- T007 and T008: Parallel (different files: `divergence.rs` vs `rules.rs`)
- T009 and T010 (unit tests): Parallel (different test scopes: comparator vs. chain)

---

## Parallel Example: User Story 1 Unit Tests

```bash
# These unit tests can be written in parallel (different test scopes):
Task: "Write comparator unit tests (T009) in src/round_trip/comparator.rs"
Task: "Write chain unit test with MockInvoker (T010) in src/round_trip/chain.rs"
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2 Only)

1. Complete Phase 1: Setup (~5 min)
2. Complete Phase 2: Foundational — CRITICAL, blocks everything
3. Complete Phase 3: US1 — comparator + chain + integration tests + FORGE fix pass
4. Complete Phase 4: US2 — log writer + classification
5. **STOP and VALIDATE**: `cargo test`, inspect `divergences.json`, confirm SC-001, SC-002, SC-004
6. Proceed to US3 (Phase 5) for full three-format coverage

### Incremental Delivery

1. Phase 1 + 2 → Foundational types and oscal-cli convert ready
2. Phase 3 (US1) → Core round-trip comparison works; SC-001, SC-002 satisfied
3. Phase 4 (US2) → Divergence log written; SC-004 satisfied
4. Phase 5 (US3) → Three-format confirmed; SC-003 satisfied
5. Phase 6 → All quality gates pass; feature complete

---

## Notes

- **[P] tasks** = can be launched in parallel (different files, no incomplete dependencies)
- **[Story] label** = maps task to user story for traceability
- **TDD order is mandatory**: tests written → confirmed FAIL → implementation → confirmed PASS
- **Commit after each logical group**: foundational types together, then each story phase
- **SEC requirements** integrated into implementation tasks: SEC-2/SEC-3/SEC-4 in T012 (chain), SEC-5 in T014 (skip test), SEC-6 in T012 (per-invocation timeout)
- **Insta snapshots** from T016/T017 must be committed via `cargo insta review` before marking US2 complete
- **FORGE fix scope** in T015 is bounded: only fix serialization/formatting divergences in the conversion pipeline; do not modify oscal-cli behavior or modify divergences classified as `OscalCliDiff` or `Acceptable`
