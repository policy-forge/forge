# Implementation Plan: OSCAL Metadata Assembly

**Branch**: `011-oscal-metadata` | **Date**: 2026-02-12 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/011-oscal-metadata/spec.md`

## Summary

Implement the OSCAL metadata assembly function (`assemble_metadata`) in the existing `src/oscal/` module. This function produces an `OscalMetadata` struct containing five required fields (`uuid`, `title`, `last-modified`, `version`, `oscal-version`) from a `DocumentMetadata` reference plus auto-generated values. Shared by all OSCAL artifact generators (Catalog, Component Definition, Profile). Injectable overrides via `MetadataOptions` enable deterministic testing.

**Scope note:** The user input mentioned "configurable roles/parties" but per the spec (W-1) and AR, optional metadata fields (`roles`, `parties`, `responsible-parties`, `locations`) are explicitly deferred. This plan implements only the five required OSCAL metadata fields.

## Technical Context

**Language/Version**: Rust (edition 2024, stable 1.93.0)
**Primary Dependencies**: `uuid` 1.20.0 (add `v4` feature), `chrono` latest stable (NEW), `serde` 1.x (existing), `thiserror` 2.0.18 (existing), `tracing` 0.1.44 (existing)
**Storage**: N/A — in-memory processing only
**Testing**: `cargo test` (unit tests in `#[cfg(test)]` modules, TDD mandatory per constitution IV)
**Target Platform**: CLI (Linux, macOS, Windows)
**Project Type**: Single Rust binary crate with module structure
**Performance Goals**: Trivial — single struct construction (UUID generation + timestamp capture + string copies)
**Constraints**: No system information leakage (SEC-1, SEC-5); UTC timestamps only (SEC-2); no network I/O
**Scale/Scope**: One function, two structs, one constant, ~50 lines of production code + ~150 lines of tests

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Crate-First Architecture | PASS | Code lives in existing `src/oscal/` module within the single `forge` crate — consistent with established project structure |
| II. Rust-First | PASS | Pure Rust, no FFI, no unsafe |
| III. Contract-First | PASS | Structs and function signature defined in AR before implementation |
| IV. Test-First (TDD) | PLAN | Tests written before implementation; injectable overrides enable deterministic assertions |
| V. Complete Implementation | PLAN | All tasks must complete before merge |
| VI. Performance-First | PASS | Trivial computation; no hot path. Benchmark not required (would benchmark UUID generation, which is external crate code) |
| VII. Security-First | PASS | SEC review completed; SEC-1 through SEC-6 constraints documented |
| VIII. Error Handling | PASS | `thiserror` for `ForgeError`; function returns `Result<OscalMetadata, ForgeError>` for API consistency |
| IX. Observability | PASS | `tracing::warn!` for empty title edge case (EC-1); tracing already a dependency |
| X. Simplicity (YAGNI) | PASS | Five fields, one function, no abstraction layers; `MetadataOptions` is minimal test seam |
| XI. Current Dependency Policy | PLAN | `chrono` at latest stable; `uuid` v4 feature added; `cargo audit` required before merge |

**Gate result: PASS** — No violations. Proceed to Phase 0.

## Project Structure

### Documentation (this feature)

```text
specs/011-oscal-metadata/
├── plan.md              # This file
├── research.md          # Phase 0: dependency research
├── data-model.md        # Phase 1: entity definitions
├── quickstart.md        # Phase 1: implementation quickstart
├── contracts/           # Phase 1: type contracts
│   └── oscal-metadata.rs  # Struct/function signatures
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
src/
├── oscal/
│   ├── mod.rs           # MODIFY: Add metadata submodule declaration + re-exports
│   └── metadata.rs      # NEW: OscalMetadata, MetadataOptions, assemble_metadata, OSCAL_VERSION
├── model/
│   └── mod.rs           # READ ONLY: DocumentMetadata (title, version) — input source
├── error.rs             # READ ONLY: ForgeError — return type
└── lib.rs               # MODIFY: Re-export OscalMetadata from oscal module
```

**Structure Decision**: Follows established single-crate module pattern. New code in `src/oscal/metadata.rs` with re-exports through `src/oscal/mod.rs`. No new crates, no new top-level modules.

## Complexity Tracking

No constitution violations to justify.
