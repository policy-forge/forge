# Implementation Plan: Assessment Plan Scaffolding — Controls (WI-41)

**Branch**: `041-assessment-plan-controls` | **Date**: 2026-03-12 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/041-assessment-plan-controls/spec.md`

---

## Summary

Add `--import-ssp <path>` to `forge convert`. When provided, the pipeline collects control
IDs from the generated Catalog or Component Definition, calls `build_assessment_plan()`, and
writes `{policy-stem}-assessment-plan.json` to the same output directory. The builder reuses
`assemble_metadata` (WI-11) and `generate_stable_id` (WI-7). No new dependencies. No new
subcommand. Fully backward compatible when `--import-ssp` is omitted.

---

## Technical Context

**Language/Version**: Rust, Edition 2024, stable 1.93.0
**Primary Dependencies**: clap 4.x, serde 1.0.228, serde_json 1.0.149, uuid 1.20.0, chrono 0.4, thiserror 2.0.18, tracing 0.1.44 — all existing in `Cargo.toml`; **no new dependencies**
**Storage**: Local filesystem — reads Markdown, writes JSON (AP always JSON regardless of `--format`)
**Testing**: `cargo test` — unit tests inline + integration tests in `tests/assessment_plan_test.rs`
**Target Platform**: Local CLI (macOS/Linux)
**Project Type**: Single Rust crate
**Performance Goals**: N/A — explicitly out of scope for this exploratory feature
**Constraints**: No new dependencies; TDD mandatory; `cargo clippy -- -D warnings` + `cargo fmt --check` must pass; no new `unsafe` code

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Crate-First Architecture | ✅ | New module `src/oscal/assessment_plan.rs` within existing crate |
| II. Rust-First Implementation | ✅ | Pure stable Rust, no `unsafe` |
| III. Contract-First Development | ✅ | `contracts/assessment_plan.rs` defines all public interfaces |
| IV. Test-First Development | ✅ | TDD mandatory per spec; all ACs and ECs have corresponding test tasks |
| V. Complete Requirement Delivery | ✅ | All M-1 through M-7, S-1, S-2 covered by tasks below |
| VI. Performance and Scope Discipline | ✅ | No benchmark tasks; performance explicitly out of scope |
| VII. Security-First Design | ✅ | SEC-2 (empty SSP validation), SEC-3 (deduplication), SEC-4 (UUID v5) all in tasks |
| VIII. Error Handling Standards | ✅ | `ForgeError::AssessmentPlanBuild` + `ForgeError::Validation` for actionable errors |
| IX. Observability and Debuggability | ✅ | `tracing::info!` for control count + SSP href; `tracing::warn!` for zero controls |
| X. Simplicity and Pragmatism | ✅ | Extends existing pipeline functions; no new subcommand; reuses all infrastructure |
| XI. Dependency Policy | ✅ | Zero new dependencies |

**Post-design re-check**: All principles satisfied. No violations requiring justification.

---

## Project Structure

### Documentation (this feature)

```text
specs/041-assessment-plan-controls/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   └── assessment_plan.rs  # Phase 1 output — interface contract
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code Changes

```text
src/
├── error.rs                          # MODIFY: add AssessmentPlanBuild variant
├── uuid.rs                           # MODIFY: add ASSESSMENT_PLAN_NAMESPACE constant
├── oscal/
│   ├── mod.rs                        # MODIFY: pub mod assessment_plan; re-export
│   ├── assessment_plan.rs            # NEW: build_assessment_plan, structs, helpers
│   └── catalog.rs                    # MODIFY: add collect_control_ids_from_catalog
│   └── component_definition.rs       # MODIFY: add collect_control_ids_from_component_def
├── pipeline.rs                       # MODIFY: add import_ssp_href param to run_*_pipeline
└── cli/
    ├── mod.rs                        # MODIFY: add --import-ssp flag to Convert
    └── convert.rs                    # MODIFY: add import_ssp field to ConvertOptions; wire AP output

tests/
└── assessment_plan_test.rs           # NEW: integration tests for AC-1..AC-7, EC-1..EC-5
```

**Structure Decision**: Single-crate, single-project layout. No new crates. All changes
are additive extensions to existing modules except `assessment_plan.rs` (new file).

---

## Phase 0: Research

**Status**: Complete. See [research.md](research.md) for all decisions.

Key resolved decisions:
- **D-1**: Collect control IDs from built artifact after construction (pure iteration, no pipeline changes to builders)
- **D-2**: `ASSESSMENT_PLAN_NAMESPACE = Uuid::new_v5(&FORGE_NAMESPACE_UUID, b"assessment-plan")`
- **D-3**: AP output path = `{output_dir}/{policy_stem}-assessment-plan.json`
- **D-4**: Batch mode: emit warning + skip AP generation (like `--stable-id-baseline`)
- **D-5**: Extend pipeline signatures with `import_ssp_href: Option<&str>` (backward compatible)
- **D-6**: `ForgeError::AssessmentPlanBuild(String)` → exit code 2
- **D-7**: `metadata.version = "1.0.0"` (static)
- **D-8**: Validate empty SSP inside `build_assessment_plan()` (testable as pure unit)

---

## Phase 1: Design

**Status**: Complete. See [data-model.md](data-model.md), [contracts/assessment_plan.rs](contracts/assessment_plan.rs), [quickstart.md](quickstart.md).

### Key Design Decisions

**Module placement**: `src/oscal/assessment_plan.rs` — alongside `catalog.rs` and
`component_definition.rs`. Follows the established pattern for OSCAL artifact builders.

**No schema validation for AP**: PRD W-3 explicitly defers AP schema validation to a future
work item. The builder produces structurally correct JSON without schema validation.

**Metadata assembly**: Constructs a synthetic `DocumentMetadata` with
`title = "Assessment Plan for {policy_title}"`, `version = "1.0.0"`, then calls
`assemble_metadata(&doc_meta, None)`. Maps result fields to `ApMetadata` for serialization.

**UUID seed**: `format!("assessment-plan|{}|{}", sorted_ids.join(","), ssp_href)` hashed
with `FORGE_NAMESPACE_UUID`. Changes when control set or SSP reference changes (satisfies FR-008, SC-003).

**Deduplication + ordering**: Inside `build_assessment_plan`, sort + dedup `control_ids`
before building `include-controls`. Preserves natural string ordering (lexicographic).

---

## Implementation Sequence

The following sequence respects all dependencies between tasks. Each step is independently
testable before proceeding.

### Step 1 — Error type extension
**File**: `src/error.rs`
**Change**: Add `ForgeError::AssessmentPlanBuild(String)` variant; map to exit code 2.
**Test**: Unit test for `Display` and `exit_code` (inline in `error.rs` tests).

### Step 2 — UUID namespace constant
**File**: `src/uuid.rs`
**Change**: Add `ASSESSMENT_PLAN_NAMESPACE` constant with derivation comment.
**Test**: Verification test `assessment_plan_namespace_matches_derivation()` (inline).

### Step 3 — Assessment Plan builder (core)
**File**: `src/oscal/assessment_plan.rs` (new file)
**Contents**:
- `AssessmentPlanEnvelope`, `AssessmentPlan`, `ApMetadata`, `ImportSsp`, `ReviewedControls`,
  `ApControlSelection`, `ApIncludeControl` structs with serde annotations
- `build_assessment_plan(control_ids, import_ssp_href, policy_title) -> Result<AssessmentPlanEnvelope, ForgeError>`
- `derive_ap_output_path(input, primary_output) -> PathBuf`
- Inline unit tests for AC-1, AC-2, AC-4, AC-7, EC-1, EC-3, EC-4, EC-5 (see tasks.md T004); AC-3 and EC-2 tests are in the same file but written in T011 as part of US2 test setup
**Dependency**: Step 1 (ForgeError), Step 2 (ASSESSMENT_PLAN_NAMESPACE)

### Step 4 — Control ID collectors
**File**: `src/oscal/catalog.rs` (add function)
**Change**: Add `pub fn collect_control_ids_from_catalog(catalog: &OscalCatalog) -> Vec<String>`

**File**: `src/oscal/component_definition.rs` (add function)
**Change**: Add `pub fn collect_control_ids_from_component_def(envelope: &ComponentDefinitionEnvelope) -> Vec<String>`
**Test**: Unit tests for both collectors (inline in respective files).
**Dependency**: Step 3 (assessment_plan module exists to reference)

### Step 5 — oscal module re-export
**File**: `src/oscal/mod.rs`
**Change**: `pub mod assessment_plan;` + re-export key public items
**Dependency**: Step 3

### Step 6 — Pipeline extension
**File**: `src/pipeline.rs`
**Change**:
- Add `import_ssp_href: Option<&str>` to `run_catalog_pipeline` and `run_component_pipeline`
- After primary artifact written: if `Some(href)`, collect control IDs, call
  `build_assessment_plan`, derive AP path, write AP JSON
- Emit `tracing::warn!` if zero controls
- Emit `tracing::info!` with control count and SSP href
**Test**: Update existing pipeline tests to pass `None` for new parameter.
**Dependency**: Steps 3, 4, 5

### Step 7 — CLI extension
**File**: `src/cli/mod.rs`
**Change**: Add `import_ssp: Option<String>` field to `Commands::Convert` with `#[arg(long)]`

**File**: `src/cli/convert.rs`
**Change**:
- Add `pub import_ssp: Option<&'a str>` to `ConvertOptions`
- In `execute()`: pass `opts.import_ssp` to `run_catalog_pipeline` / `run_component_pipeline`
- In `execute_dispatch()`: if batch mode and `import_ssp.is_some()`, emit warning and pass `None`
**Test**: CLI parsing tests for `--import-ssp` flag (present/absent/empty string); update existing `make_opts` helper.
**Dependency**: Step 6

### Step 8 — Integration tests
**File**: `tests/assessment_plan_test.rs` (new file)
**Contents**: End-to-end tests using `tests/fixtures/sample_policy.md`:
- AP file written when `--import-ssp` provided
- AP file NOT written when `--import-ssp` omitted
- AP contains all control IDs from sample policy
- AP UUID is deterministic across two runs
- Empty `--import-ssp` produces validation error
**Dependency**: Steps 3–7 complete

### Step 9 — Quality gates
```bash
cargo test             # All tests pass
cargo clippy -- -D warnings   # Zero warnings
cargo fmt --check      # No formatting issues
```

---

## Acceptance Criteria Coverage

| AC ID | PRD Req | Step | Test Location |
|-------|---------|------|---------------|
| AC-1 | M-1 | Step 3 | `assessment_plan.rs` unit tests |
| AC-2 | M-2 | Step 3 | `assessment_plan.rs` unit tests |
| AC-3 | M-3 | Step 3 | `assessment_plan.rs` unit tests |
| AC-4 | M-4, M-5 | Step 3 | `assessment_plan.rs` unit tests |
| AC-5 | M-6 | Steps 3, 7 | `assessment_plan.rs` + `cli/mod.rs` tests |
| AC-6 | M-7 | Step 3 | `assessment_plan.rs` unit tests |
| AC-7 | S-1 | Step 3 | `assessment_plan.rs` unit tests |
| EC-1 | M-5 | Step 3 | `assessment_plan.rs` unit tests |
| EC-2 | M-3 | Step 3 | `assessment_plan.rs` unit tests |
| EC-3 | M-5 | Step 3 | `assessment_plan.rs` unit tests |
| EC-4 | M-1 | Step 8 | `tests/assessment_plan_test.rs` |
| EC-5 | M-7 | Steps 3, 8 | `assessment_plan.rs` + integration tests |

SEC requirements:
| SEC Req | Step | Test |
|---------|------|------|
| SEC-2 (empty SSP) | Step 3 | EC-2 test |
| SEC-3 (deduplication) | Step 3 | EC-3 test |
| SEC-4 (UUID v5) | Step 2, 3 | AC-6 test |

---

## Risk Log

| Risk | Mitigation |
|------|------------|
| Pipeline signature change breaks existing callers | All callers pass `None`; addition is backward compatible |
| AP UUID changes on control order variation | Sort control IDs before hashing — order-independent |
| batch mode confusion with `--import-ssp` | Emit warning; skip gracefully (mirrors `--stable-id-baseline` behavior) |
