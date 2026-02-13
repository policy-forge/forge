# Implementation Plan: Component Implemented Requirements

**Branch**: `015-component-implemented-requirements` | **Date**: 2026-02-13 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/015-component-implemented-requirements/spec.md`
**AR**: [015-ar-component-implemented-requirements.md](../../docs/AR/015-ar-component-implemented-requirements.md)
**SEC**: [015-sec-component-implemented-requirements.md](../../docs/SEC/015-sec-component-implemented-requirements.md)

## Summary

Extend the WI-14 Component Definition builder to populate `control-implementations[]` with `implemented-requirements` mapped 1:1 from `PolicyRequirement`s. Each implemented-requirement carries a deterministic UUID v5, a `control-id` matching the Catalog builder's scheme (e.g., `POL-AC-001`), and the raw requirement prose as the implementation narrative. The `--source-profile` CLI flag provides the baseline profile reference for the `source` field. All new code lives in the existing `forge` crate, extending `src/oscal/` with a new `implemented_requirements.rs` module.

## Technical Context

**Language/Version**: Rust (edition 2024, stable 1.93.0)
**Primary Dependencies**: uuid 1.20.0 (v5 feature), serde 1.x, serde_json 1.x, clap 4.x, thiserror 2.0.18, tracing 0.1.44 — all existing, no new dependencies
**Storage**: N/A — in-memory processing only
**Testing**: `cargo test`, TDD mandatory (Constitution IV)
**Target Platform**: CLI (macOS/Linux)
**Project Type**: Single crate (`forge`)
**Performance Goals**: N/A — in-memory JSON assembly; no hot paths requiring benchmarks
**Constraints**: OSCAL v1.2.0 compliance; no new dependencies (Constitution XI); no mutation of existing WI-14 output structure (note: `build_component_definition` gains an `Option<&str>` parameter — callers updated for backward compatibility)
**Scale/Scope**: Documents with 0–1000s of PolicyRequirements; single control-implementations entry per source profile

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Crate-First Architecture | PASS | Single crate, extends existing `oscal` module — no new crate needed |
| II. Rust-First | PASS | All Rust, no FFI, no unsafe |
| III. Contract-First Development | MUST DO | Define function signatures, types, and error variants before implementation |
| IV. Test-First Development | MUST DO | Write tests before implementation; RED-GREEN-REFACTOR |
| V. Complete Implementation | MUST DO | All tasks in tasks.md must be complete before merge |
| VI. Performance-First Design | PASS | In-memory JSON assembly; no hot paths requiring benchmarks |
| VII. Security-First Design | PASS | No unsafe, no secrets; `--source-profile` validated for presence (SEC-3, SEC-4) |
| VIII. Error Handling Standards | MUST DO | Use `thiserror` `ForgeError` variants; descriptive errors for missing/empty `--source-profile` |
| IX. Observability | MUST DO | `tracing::warn!` for zero requirements (FR-013), empty text (FR-014), missing stable_id (EC-2) |
| X. Simplicity | PASS | Direct 1:1 mapping, no over-engineering; prose used as-is |
| XI. Current Dependency Policy | PASS | No new dependencies required |

**Gate Result**: PASS — No violations. Proceed to Phase 0.

## Project Structure

### Documentation (this feature)

```text
specs/015-component-implemented-requirements/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── interfaces.rs    # Rust function signatures and types
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
src/
├── cli/
│   ├── mod.rs                  # [MODIFY] Add --source-profile flag to Convert command
│   └── convert.rs              # [MODIFY] Wire component strategy to pipeline
├── oscal/
│   ├── mod.rs                  # [MODIFY] Re-export new module
│   ├── catalog.rs              # [MODIFY] Make resolve_abbreviation pub(crate)
│   ├── component_definition.rs # [MODIFY] Accept source_profile param; inject control-implementations
│   └── implemented_requirements.rs  # [NEW] Core WI-15 logic
├── uuid.rs                     # [MODIFY] Add CONTROL_IMPL_NAMESPACE, IMPL_REQ_NAMESPACE constants
├── pipeline.rs                 # [MODIFY] Add run_component_pipeline function
└── error.rs                    # [VERIFY] ForgeError::ComponentDefinitionBuild variant exists

tests/
├── common/mod.rs               # [VERIFY] Test helpers sufficient
└── component_pipeline_test.rs  # [NEW] Integration test for component pipeline
```

**Structure Decision**: Single crate, extending the existing `src/oscal/` module family. The new `implemented_requirements.rs` follows the established pattern (cf. `catalog.rs`, `component_definition.rs`, `back_matter.rs`, `parts.rs`, `metadata.rs`). No new crates or projects needed.

## Complexity Tracking

> No violations — no complexity justifications needed.
