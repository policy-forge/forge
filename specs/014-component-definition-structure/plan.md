# Implementation Plan: OSCAL Component Definition Structure

**Branch**: `014-component-definition-structure` | **Date**: 2026-02-12 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/014-component-definition-structure/spec.md`

## Summary

Build an OSCAL Component Definition JSON builder (`build_component_definition`) in the `oscal` module that produces a `component-definition` root with document-level metadata (reusing WI-11 `assemble_metadata`), a `components` array containing one documentary component of type `"policy"`, and optional back matter (reusing WI-12). The documentary component has a deterministic UUID v5 generated from the PolicyDocument's title and version. This is the structural foundation for the component-first conversion strategy (`--strategy component`), enabling WI-15 (implemented-requirements) and WI-16 (traceability).

## Technical Context

**Language/Version**: Rust edition 2024, stable 1.93.0
**Primary Dependencies**: serde 1.x, serde_json 1.x, uuid 1.20.0 (v4+v5 features), chrono 0.4, thiserror 2.0.18, tracing 0.1.44 -- all existing, no new dependencies
**Storage**: N/A (in-memory processing only)
**Testing**: `cargo test` with TDD workflow; unit tests in-module, integration tests in `tests/` directory
**Target Platform**: CLI tool (macOS, Linux)
**Project Type**: Single Rust crate
**Performance Goals**: N/A -- single-pass in-memory transformation
**Constraints**: No new dependencies; must reuse WI-11 metadata and WI-12 back matter; typed structs (consistent with actual Catalog builder pattern)
**Scale/Scope**: Single PolicyDocument -> single Component Definition JSON

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Crate-First Architecture | PASS | Adds to existing `oscal` module within the `forge` crate, same boundary as Catalog builder |
| II. Rust-First | PASS | Pure Rust, no FFI, no unsafe |
| III. Contract-First | PASS | Types and error variants defined before implementation (see data-model.md, contracts/) |
| IV. Test-First | PASS | TDD mandatory -- tests written before implementation |
| V. Complete Implementation | PASS | All tasks must be complete before merge |
| VI. Performance-First | N/A | Simple in-memory transformation; no hot paths |
| VII. Security-First | PASS | SEC review completed (Low risk); no new inputs, no network, no secrets |
| VIII. Error Handling | PASS | Add `ComponentDefinitionBuild` variant to `ForgeError` using `thiserror` |
| IX. Observability | PASS | Not yet needed per AR; add `tracing` spans in future sprint |
| X. Simplicity | PASS | YAGNI -- build structural envelope only; no traits, no abstractions beyond what Catalog uses |
| XI. Current Dependencies | PASS | No new dependencies; all existing crates at current stable versions |

## Project Structure

### Documentation (this feature)

```text
specs/014-component-definition-structure/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── component_definition.rs  # Type contracts
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── oscal/
│   ├── mod.rs                     # Add component_definition module + re-exports
│   ├── component_definition.rs    # NEW: Component Definition builder (this feature)
│   ├── catalog.rs                 # Existing Catalog builder (pattern reference)
│   ├── metadata.rs                # Existing shared metadata assembly (reused)
│   ├── back_matter.rs             # Existing back matter generation (reused)
│   └── parts.rs                   # Existing control parts (not used)
├── uuid.rs                        # Add COMPONENT_NAMESPACE constant
├── error.rs                       # Add ComponentDefinitionBuild variant
├── lib.rs                         # Add re-exports for new types
└── pipeline.rs                    # NOT modified (WI-18 handles pipeline wiring)
```

**Structure Decision**: Single new file `src/oscal/component_definition.rs` following the same pattern as `src/oscal/catalog.rs`. This keeps the oscal module organized by artifact type with one file per builder.

## Constitution Re-check (Post Phase 1 Design)

| Principle | Status | Design Impact |
|-----------|--------|---------------|
| I. Crate-First | PASS | Single new file in existing `oscal` module; no new crate boundary |
| III. Contract-First | PASS | `contracts/component_definition.rs` defines all types before implementation |
| VII. Security-First | PASS | SEC-1 through SEC-4 covered by unit tests (no remarks, default title, default version) |
| VIII. Error Handling | PASS | `ComponentDefinitionBuild(String)` via thiserror; `Result` return type on builder |
| X. Simplicity | PASS | Research R-1 found typed structs ARE the actual Catalog pattern; no added abstraction. No trait, no Value builder -- just structs mirroring catalog.rs |
| XI. Current Dependencies | PASS | Zero new dependencies added by design |

**GATE: PASS** -- No violations detected. Proceed to `/speckit.tasks`.

## Complexity Tracking

No constitution violations to justify. The implementation follows the simplest approach that meets requirements.
