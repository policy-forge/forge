# Implementation Plan: Summary Dashboard

**Branch**: `044-summary-dashboard` | **Date**: 2026-03-10 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/044-summary-dashboard/spec.md`

## Summary

Add a `--summary` flag to `forge convert` that prints a box-drawing formatted dashboard to stderr after conversion, showing: sections parsed, requirements extracted, controls generated, validation status (with up to 3 error messages), mapping coverage percentage, elapsed time, strategy, and output path. Statistics are collected via a `ConversionStatistics` accumulator struct populated at pipeline stage boundaries. ANSI colors are used with automatic terminal detection based on `stderr` terminal capability. No new crate dependencies required (uses `std::io::IsTerminal`).

## Technical Context

**Language/Version**: Rust, Edition 2024, stable 1.93.0
**Primary Dependencies**: clap 4.x (CLI), serde 1.0.228, serde_json 1.0.149, thiserror 2.0.18, tracing 0.1.44 — all existing
**Storage**: N/A — dashboard writes to stderr, no persistence
**Testing**: `cargo test` + insta 1.46.3 (snapshot testing for dashboard format), tempfile 3.25.0
**Target Platform**: Cross-platform CLI (macOS, Linux, Windows)
**Project Type**: Single Rust crate
**Performance Goals**: <1% overhead from statistics collection (counter increments only)
**Constraints**: No new crate dependencies; `--summary` must not alter conversion behavior
**Scale/Scope**: XS-sized feature — 1 new module (~200 lines), modifications to 3 existing files

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Crate-First Architecture | PASS | New `src/summary/` module within existing crate; no new crates |
| II. Rust-First Implementation | PASS | Pure stable Rust; no unsafe code |
| III. Contract-First Development | PASS | `ConversionStatistics` struct and `format_summary_dashboard` contract defined in spec and AR |
| IV. Test-First Development | PASS | TDD mandatory per plan; unit tests for stats, formatting, coverage calc |
| V. Complete Requirement Delivery | PASS | All Must-Have (M-1 through M-7) and Should-Have (S-1 through S-3) covered by planned tasks |
| VI. Performance and Scope Discipline | PASS | Feature explicitly out-of-scope for benchmarking (per PRD W-3); no speculative benchmark work added |
| VII. Security-First Design | PASS | SEC-1 through SEC-3 mapped to tasks; no sensitive data in output |
| VIII. Error Handling Standards | PASS | Zero-division guard for coverage; validation "Not run" fallback; tests use substring assertions |
| IX. Observability and Debuggability | PASS | Statistics logged at DEBUG level; dashboard output is deterministic and snapshot-testable |
| X. Simplicity and Pragmatism | PASS | Extends existing CLI and pipeline; minimal new code; no new frameworks |
| XI. Dependency Policy | PASS | No new dependencies; uses `std::io::IsTerminal` (stable since Rust 1.70) for terminal detection |
| Quality Gates | PASS | `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check` required |

## Project Structure

### Documentation (this feature)

```text
specs/044-summary-dashboard/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── summary.rs       # ConversionStatistics struct and format function signatures
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── summary/
│   ├── mod.rs           # NEW: ConversionStatistics struct, ValidationStatus enum, mapping_coverage()
│   └── format.rs        # NEW: format_summary_dashboard(), ANSI color helpers, box-drawing output
├── cli/
│   ├── mod.rs           # MODIFY: Add --summary flag to Convert variant
│   └── convert.rs       # MODIFY: Add summary param, wire stats collection, print dashboard after write
├── pipeline.rs          # MODIFY: Return stats from pipeline functions (or accept &mut stats)
├── lib.rs               # MODIFY: Add pub mod summary
└── ...

tests/
├── summary_dashboard_test.rs  # NEW: Unit + integration tests for dashboard
└── ...
```

**Structure Decision**: Single Rust crate with a new `src/summary/` module containing the `ConversionStatistics` struct and formatting logic. This follows the existing pattern of feature modules (`src/validate/`, `src/export/`, `src/parameter/`).

## Complexity Tracking

No constitution violations. No complexity justification needed.

## Design Decisions

### D1: Statistics Collection Strategy

**Decision**: Pipeline functions return `ConversionStatistics` alongside their existing results rather than accepting a `&mut` reference.

**Rationale**: The current pipeline functions (`run_catalog_pipeline`, `run_component_pipeline`) return `Result<(), ForgeError>`. Changing the return type to include stats is a cleaner functional approach than threading a mutable reference. However, since the pipelines call `prepare_document` (shared stages) and then strategy-specific stages, the simplest approach is to:
1. Have `prepare_document` return the `PolicyDocument` (unchanged)
2. Count sections/requirements from the returned `PolicyDocument` after `prepare_document`
3. Count controls from the generated OSCAL artifacts
4. Capture validation status from the existing validation step

This avoids modifying pipeline internals — statistics are derived from existing return values.

### D2: Pipeline Signature Changes

**Decision**: Modify `run_catalog_pipeline` and `run_component_pipeline` to return `Result<ConversionStatistics, ForgeError>` instead of `Result<(), ForgeError>`. Callers that don't need stats simply ignore the return value. `ConversionStatistics` is always populated (cheap counter increments) regardless of whether `--summary` is set.

**Alternatives rejected**:
- *Wrapper functions* (`run_catalog_pipeline_with_stats`): Rejected — adds indirection without benefit since stats collection is zero-cost.
- *Orchestration in `convert::execute`*: Rejected — would require extracting pipeline internals and duplicating stage orchestration logic.

### D3: Terminal Detection for Colors

**Decision**: Use `std::io::IsTerminal` trait on `std::io::stdout()` to detect interactive terminals. Available since Rust 1.70, no new dependency needed.

### D4: Validation Error Messages in Dashboard

**Decision**: Store up to 3 validation error messages in `ConversionStatistics`. The existing `ValidationReport` has an `errors()` method returning all errors. We capture the first 3 error messages from that report.

### D5: Elapsed Time Measurement

**Decision**: Wrap the pipeline execution in `std::time::Instant::now()` / `elapsed()` in `convert::execute`. The elapsed time covers the full pipeline from `prepare_document` through file write.
