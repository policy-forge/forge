# Implementation Plan: Project Scaffolding

**Branch**: `001-project-scaffolding` | **Date**: 2026-02-11 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `specs/001-project-scaffolding/spec.md`

## Summary

Establish the FORGE project foundation: a single-crate Rust CLI with clap 4.x derive macros for CLI argument parsing, thiserror for structured error types, a flat module hierarchy mirroring the conversion pipeline stages (`cli`, `ingest`, `parse`, `model`, `oscal`, `validate`, `export`), and a CI pipeline enforcing formatting, linting, testing, and dependency security quality gates. All subcommands are stubs only — no business logic is implemented in this work item.

## Technical Context

**Language/Version**: Rust latest stable (edition 2021)
**Primary Dependencies**: clap 4.x (CLI framework, derive macros), thiserror (error type derivation)
**Storage**: N/A — no data persistence in this work item
**Testing**: `cargo test` (unit tests in `#[cfg(test)]` modules, integration tests in `tests/`)
**Target Platform**: Linux, macOS, Windows (cross-platform CLI)
**Project Type**: Single crate (binary + library)
**Performance Goals**: CLI help output in < 500ms (constitution startup target)
**Constraints**: No dependencies beyond clap and thiserror; stubs only, no business logic
**Scale/Scope**: Single developer, 1 sprint (S-1), foundation for 49 subsequent work items

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Crate-First Architecture | JUSTIFIED DEVIATION | AR decision: single crate for scaffolding; workspace extraction deferred to MS-2/MS-4 when justified by real code. Constitution says "every feature begins as standalone crate" for features, not scaffolding. |
| II. Rust-First | PASS | Rust is the sole language; no FFI or unsafe code |
| III. Contract-First Development | PASS | CLI struct (clap derive) and ForgeError enum defined before handler implementation |
| IV. Test-First Development | PASS | TDD mandatory; tests for CLI parsing and error Display before implementation |
| V. Complete Implementation | PASS | All tasks must be complete before merge |
| VI. Performance-First Design | PASS | < 500ms startup; no hot paths in stubs |
| VII. Security-First Design | PASS | No attack surface (no input processing, no network, no data handling) |
| VIII. Error Handling Standards | PASS | thiserror for library error types; clap handles CLI argument errors |
| IX. Observability | DEFERRED | No tracing/logging needed for scaffolding stubs; added when pipeline has observable stages |
| X. Simplicity & Pragmatism | PASS | Single crate, two dependencies, minimal boilerplate — simplest approach meeting all requirements |
| XI. Current Dependency Policy | PASS | clap and thiserror at latest stable versions; cargo audit clean |

**Gate Result**: PASS (1 justified deviation documented in AR, 1 deferred item appropriate for scaffolding scope)

## Project Structure

### Documentation (this feature)

```text
specs/001-project-scaffolding/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── cli-interface.md # CLI argument contract
├── checklists/
│   └── requirements.md  # Specification quality checklist
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── main.rs              # Entry point: CLI parse + subcommand dispatch
├── lib.rs               # Library root: re-exports public API
├── error.rs             # ForgeError enum (thiserror)
├── cli/
│   ├── mod.rs           # Clap Cli struct, Commands enum, subcommand routing
│   ├── convert.rs       # Convert subcommand handler (stub)
│   └── validate.rs      # Validate subcommand handler (stub)
├── ingest/
│   └── mod.rs           # File ingestion module (stub)
├── parse/
│   └── mod.rs           # Markdown parsing module (stub)
├── model/
│   └── mod.rs           # Domain model module (stub)
├── oscal/
│   └── mod.rs           # OSCAL generation module (stub)
├── validate/
│   └── mod.rs           # Schema validation module (stub)
└── export/
    └── mod.rs           # Output serialization module (stub)

tests/
└── cli_integration.rs   # Integration tests for CLI behavior
```

**Structure Decision**: Single crate with flat module layout per AR Option 1. Module directories mirror the conversion pipeline stages. Binary crate (`main.rs`) is a thin wrapper dispatching to the `cli` module. Library code lives in `lib.rs` and sub-modules. This matches the AR architecture diagram and constitution principle X (simplicity).

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Single crate instead of workspace (Principle I) | Scaffolding has only stubs — 6 of 7 modules will be empty. Workspace overhead (7 Cargo.toml files, inter-crate deps) adds boilerplate with no benefit. | AR evaluated Option 2 (workspace from day one) and rejected it: "Massive over-engineering for sprint 1." Workspace extraction planned at MS-2 or MS-4 when module boundaries are proven by real code. |
