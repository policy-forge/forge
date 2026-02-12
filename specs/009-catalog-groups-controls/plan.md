# Implementation Plan: OSCAL Catalog Groups and Controls

**Branch**: `009-catalog-groups-controls` | **Date**: 2026-02-11 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/009-catalog-groups-controls/spec.md`

## Summary

Implement a pure-function Catalog builder that maps the domain model (`PolicyDocument` → `PolicySection` → `PolicyRequirement`) to OSCAL Catalog JSON (`OscalCatalog` → `OscalGroup` → `OscalControl`). Control IDs follow the `POL-{ABBR}-{NNN}` pattern with deterministic abbreviation derivation and numeric-suffix collision resolution. Group IDs are slugified section titles. Serialize via `serde_json`. No new dependencies required — `serde`, `serde_json`, `tracing`, and `thiserror` are already in `Cargo.toml`.

## Technical Context

**Language/Version**: Rust (edition 2024, stable 1.93.0)
**Primary Dependencies**: serde 1.x, serde_json 1.x, thiserror 2.0.18, tracing 0.1.44, uuid 1.20.0 (all existing)
**Storage**: N/A — in-memory processing only; no persistent storage
**Testing**: `cargo test` (unit tests in-module `#[cfg(test)]`), `cargo clippy -- -D warnings`, `cargo fmt --check`
**Target Platform**: CLI (cross-platform)
**Project Type**: Single crate with modules
**Performance Goals**: N/A — pure data transformation on single-document scale; no latency-sensitive hot path
**Constraints**: Pure function (no side effects, no file I/O, no domain model mutation)
**Scale/Scope**: Single policy document; typical scale is <100 sections, <1000 requirements

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Crate-First Architecture | PASS (pragmatic) | Project uses module-per-feature pattern within single `forge` crate (established by WI-1 through WI-8). New code goes in `src/oscal/` module. No workspace extraction needed at this scale. |
| II. Rust-First | PASS | Pure Rust, no FFI, no unsafe |
| III. Contract-First | PASS | Types and function signatures defined in AR interface contract before implementation |
| IV. Test-First (TDD) | PASS | TDD cycle mandatory; tests written before implementation per constitution |
| V. Complete Implementation | PASS | All tasks must be complete before merge |
| VI. Performance-First | PASS (N/A for hot path) | Pure data transformation; no criterion benchmark required unless profiling reveals a need. Document scale is bounded. |
| VII. Security-First | PASS | SEC review completed (Low risk). No unsafe, no secrets, no network I/O. SEC-1 through SEC-5 addressed. |
| VIII. Error Handling | PASS | `thiserror` for `ForgeError` variants; new `CatalogBuild` variant needed |
| IX. Observability | PASS | `tracing` at DEBUG level for group/control counts and collision resolution |
| X. Simplicity (YAGNI) | PASS | Direct mapping, no intermediate representation, no configurable rules, flat-only MVP |
| XI. Current Dependencies | PASS | All dependencies already at latest stable; no new crates |

**Gate Result: PASS** — No violations. Proceed to Phase 0.

**Post-Phase 1 Re-check: PASS** — Design uses typed structs with serde derives (Contract-First), pure function (Security-First), and direct mapping (Simplicity). No violations introduced.

## Project Structure

### Documentation (this feature)

```text
specs/009-catalog-groups-controls/
├── plan.md              # This file
├── research.md          # Phase 0 output (all NEEDS CLARIFICATION resolved)
├── data-model.md        # Phase 1 output (OSCAL structs and mapping)
├── quickstart.md        # Phase 1 output (developer guide)
├── contracts/
│   └── catalog.rs       # Phase 1 output (interface contract / type signatures)
└── tasks.md             # Phase 2 output (created by /speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── oscal/
│   ├── mod.rs           # Module root: re-exports, submodule declarations
│   └── catalog.rs       # NEW: OscalCatalog/Group/Control structs, build_catalog(),
│                        #       generate_group_id(), generate_section_abbreviation(),
│                        #       generate_control_id(), title derivation
├── error.rs             # MODIFY: Add CatalogBuild variant to ForgeError
├── model/
│   └── mod.rs           # READ ONLY: PolicyDocument, PolicySection, PolicyRequirement
└── lib.rs               # READ ONLY (oscal module already declared)
```

**Structure Decision**: Single crate, new `src/oscal/catalog.rs` submodule within the existing `src/oscal/` module. This follows the established pattern (e.g., `src/parse/clauses.rs`, `src/parse/atomize.rs`).

## Complexity Tracking

No constitution violations to justify.

## Design Decisions

### D-1: Abbreviation Collision Resolution — Numeric Suffix

**Decision**: First section retains base abbreviation (`AC`); subsequent collisions receive numeric suffix (`AC2`, `AC3`, ...).
**Rationale**: Simpler, deterministic, generalizes to N collisions. Clarified in spec session 2026-02-11.

### D-2: Control Title Derivation — First Sentence, 120-char Cap

**Decision**: Extract first sentence (up to first `.`, `!`, or `?`). If first sentence exceeds 120 characters, truncate at 120 and append `...`.
**Rationale**: Semantically meaningful boundaries; practical display width. Clarified in spec session 2026-02-11.

### D-3: Nested Section Handling — Flat Mapping for MVP

**Decision**: Only top-level sections (`document.sections`) become groups. Child sections' requirements are recursively collected and included in the parent group's controls.
**Rationale**: Simpler architecture, avoids recursive group nesting complexity. Clarified in spec session 2026-02-11.

### D-4: Error on Missing stable_id

**Decision**: If any `PolicyRequirement.stable_id` is `None`, `build_catalog` returns `ForgeError::CatalogBuild` with a message identifying the affected requirement.
**Rationale**: SEC-1 requires explicit error, not silent default. WI-7 must run before WI-9.

### D-5: Placeholder Metadata

**Decision**: `OscalCatalog.uuid` set to `"00000000-0000-0000-0000-000000000000"`. `OscalMetadata` fields set to placeholder values (`title: "placeholder"`, `last_modified: "1970-01-01T00:00:00Z"`, `version: "0.0.0"`, `oscal_version: "1.2.0"`).
**Rationale**: WI-11 populates real metadata. Placeholders maintain valid JSON structure.

### D-6: JSON Envelope

**Decision**: The serialized output wraps the catalog in a top-level `{"catalog": {...}}` object, matching OSCAL v1.2.0 conventions shown in the research sample.
**Rationale**: OSCAL JSON uses a root-level model key (e.g., `"catalog"`, `"profile"`). The research sample and NIST examples consistently use this pattern.

## Implementation Order

Following TDD (Red-Green-Refactor) for each step:

1. **Error variant**: Add `CatalogBuild` variant to `ForgeError`
2. **OSCAL structs**: Define `OscalCatalog`, `OscalGroup`, `OscalControl`, `OscalMetadata`, `CatalogEnvelope` with serde derives
3. **generate_group_id()**: Slugify section title → group ID
4. **generate_section_abbreviation()**: Derive abbreviation from section title (initials of significant words, skip stop words)
5. **generate_control_id()**: Format `POL-{ABBR}-{NNN}` with zero-padded index
6. **derive_control_title()**: First sentence extraction with 120-char cap
7. **build_catalog()**: Orchestrate mapping with abbreviation tracking, collision detection, recursive requirement collection
8. **JSON serialization**: `CatalogEnvelope` wrapping `OscalCatalog`, serialize with `serde_json::to_string_pretty`
9. **Integration test**: End-to-end test with realistic `PolicyDocument` input
