# Implementation Plan: Traceability Model

**Branch**: `016-traceability-model` | **Date**: 2026-02-13 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/016-traceability-model/spec.md`

## Summary

Define the `TraceLink`, `SourceLocation`, and `TraceLinkCollection` data structures that provide bidirectional traceability between source policy requirements and generated OSCAL elements. The collection uses a `Vec<TraceLink>` canonical store with dual `HashMap` indexes for O(1) forward (requirement -> OSCAL elements) and reverse (OSCAL element -> requirement) lookups. Integration hooks are added to the Catalog builder (WI-9) and Component Definition builder (WI-14/WI-15) to capture trace links at generation time. No new external dependencies required.

## Technical Context

**Language/Version**: Rust edition 2024, stable 1.93.0
**Primary Dependencies**: serde 1.x (Serialize/Deserialize), thiserror 2.0.18 (error types), tracing 0.1.44 (logging) -- all existing
**Storage**: N/A -- in-memory processing only (process lifetime)
**Testing**: `cargo test` (unit tests in `#[cfg(test)]` blocks + integration tests in `tests/`)
**Target Platform**: CLI tool (Linux, macOS, Windows)
**Project Type**: Single Rust binary with library crate
**Performance Goals**: O(1) amortized bidirectional lookups via HashMap indexing
**Constraints**: No new external dependencies; standard library HashMap/Vec only for indexes
**Scale/Scope**: Hundreds to low thousands of trace links per conversion run

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Crate-First Architecture | PASS | New `trace` submodule in existing `model` module -- appropriate since TraceLink is a domain model type |
| II. Rust-First | PASS | Pure Rust, no FFI, no unsafe code needed |
| III. Contract-First Development | PASS | Interface contract defined in AR. Data model and contracts defined in Phase 1 before implementation |
| IV. Test-First Development | PASS | TDD mandatory. Tests written before implementation per task breakdown |
| V. Complete Implementation | PASS | All tasks must be complete before merge |
| VI. Performance-First Design | PASS | O(1) HashMap lookups per AR decision. No performance concerns for expected scale |
| VII. Security-First Design | PASS | No unsafe code. SEC review rates risk as Low. Data minimization applied |
| VIII. Error Handling Standards | PASS | `TraceError` enum with `thiserror`. Graceful empty/None returns for missing lookups |
| IX. Observability | PASS | `tracing` for trace link count logging. Debug derives on all types |
| X. Simplicity & Pragmatism | PASS | Minimal data structures using std library types. No external graph library. YAGNI applied |
| XI. Current Dependency Policy | PASS | No new dependencies. All existing deps are current |

**Gate Result**: PASS -- no violations.

## Project Structure

### Documentation (this feature)

```text
specs/016-traceability-model/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
src/
├── model/
│   ├── mod.rs           # MODIFY: add `pub mod trace;` declaration
│   ├── trace.rs         # NEW: SourceLocation, TraceLink, TraceError, TraceLinkCollection
│   ├── assemble.rs      # existing
│   └── frontmatter.rs   # existing
├── oscal/
│   ├── catalog.rs       # MODIFY: add TraceLinkCollection parameter to build_catalog
│   ├── component_definition.rs  # MODIFY: add TraceLinkCollection parameter
│   └── ...              # existing (unchanged)
├── pipeline.rs          # MODIFY: create TraceLinkCollection, pass to builders
├── lib.rs               # MODIFY: re-export trace types
└── ...                  # existing (unchanged)

tests/
└── trace_integration.rs # NEW: integration tests for trace link capture in pipelines
```

**Structure Decision**: Single project structure. New `trace.rs` submodule added to the existing `model` module per AR guidance. The trace model is a domain concept that belongs alongside `PolicyDocument`, `PolicySection`, and `PolicyRequirement`.

## Complexity Tracking

No constitution violations to justify. The implementation uses only standard library types and follows the simplest approach from the AR (Option 1: Adjacency List with Dual HashMap Indexes).
