# Implementation Plan: Structural Extraction — Headings

**Branch**: `003-structural-extraction-headings` | **Date**: 2026-02-11 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/003-structural-extraction-headings/spec.md`
**AR**: [003-ar-structural-extraction-headings.md](../../docs/AR/003-ar-structural-extraction-headings.md)
**SEC**: [003-sec-structural-extraction-headings.md](../../docs/SEC/003-sec-structural-extraction-headings.md)

## Summary

Extract Markdown heading elements (H1-H6) from ingested content into a hierarchical `Vec<SectionNode>` tree using pulldown-cmark's event-based parser with a stack-based single-pass O(n) algorithm. Each `SectionNode` records heading title, level, source line number, optional body text, and child sections. The implementation handles irregular heading nesting (skipped levels, multiple H1s, documents starting with deep headings) without panicking or losing sections.

## Technical Context

**Language/Version**: Rust edition 2024, stable 1.93.0
**Primary Dependencies**: pulldown-cmark 0.13.x (new), clap 4, thiserror 2.0.18, serde 1.x, serde_json 1.x, sha2 0.10.x (existing)
**Storage**: N/A — in-memory processing only
**Testing**: `cargo test` (unit tests in-module, integration tests in `tests/`)
**Target Platform**: macOS/Linux CLI
**Project Type**: Single Rust binary crate
**Performance Goals**: O(n) in document size; all 25 example policy documents parsed successfully
**Constraints**: No panics on any input; explicit stack (not call-stack recursion) per SEC-3
**Scale/Scope**: Typical policy documents 1-100 headings, up to 10MB content (bounded by WI-2)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Crate-First Architecture | PASS (with note) | This is a single-crate project. The `parse` module is the boundary. No new crate needed — the project is not a workspace. |
| II. Rust-First | PASS | Pure Rust implementation. No FFI, no unsafe code. |
| III. Contract-First Development | PASS | `SectionNode` struct and `extract_sections` function signature defined in AR before implementation. See [contracts/](./contracts/). |
| IV. Test-First Development | PASS | TDD cycle required. Tests written before implementation per constitution and AR guardrails. |
| V. Complete Implementation | PASS | All tasks must be complete before merge. |
| VI. Performance-First Design | PASS | O(n) single-pass algorithm. No premature optimization needed. |
| VII. Security-First Design | PASS | SEC-1 through SEC-4 incorporated. No unsafe code. Explicit stack prevents stack overflow (SEC-3). |
| VIII. Error Handling Standards | PASS | Uses existing `ForgeError::Parse` variant with `thiserror`. Empty docs return empty Vec (not error). |
| IX. Observability & Debuggability | PASS | `#[derive(Debug)]` on `SectionNode`. Tracing deferred per AR. |
| X. Simplicity & Pragmatism | PASS | Minimal implementation — single function, one struct, one helper. No premature abstractions. |
| XI. Current Dependency Policy | PASS | pulldown-cmark 0.13.x is latest stable. Must run `cargo audit` before adding. |

**Gate result: PASS — no violations.**

## Project Structure

### Documentation (this feature)

```text
specs/003-structural-extraction-headings/
├── plan.md              # This file
├── research.md          # Phase 0 output — pulldown-cmark API research
├── data-model.md        # Phase 1 output — SectionNode entity model
├── quickstart.md        # Phase 1 output — developer quickstart
├── contracts/           # Phase 1 output — Rust type contracts
│   └── extract_sections.rs
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
src/
├── cli/
│   ├── mod.rs           # CLI dispatch (existing)
│   └── convert.rs       # Convert handler — wire extract_sections call (modify)
├── error.rs             # ForgeError enum (existing, no changes needed)
├── ingest/
│   └── mod.rs           # IngestedDocument (existing, consume only — DO NOT MODIFY)
├── parse/
│   └── mod.rs           # SectionNode struct + extract_sections function (implement here)
├── model/
│   └── mod.rs           # Empty stub (DO NOT MODIFY — WI-5 scope)
├── lib.rs               # Module declarations (existing)
└── main.rs              # Entry point (existing, no changes)

tests/
└── cli_integration.rs   # Existing integration tests
```

**Structure Decision**: Single Rust binary crate (not a workspace). All heading extraction code lives in `src/parse/mod.rs`. This follows the existing project layout where each module (`ingest/`, `parse/`, `model/`, etc.) has a `mod.rs`. No new modules or crates are created.

## Complexity Tracking

No constitution violations to justify. The implementation is minimal:
- 1 struct (`SectionNode`)
- 1 public function (`extract_sections`)
- 3 helper functions (`heading_level_to_u8`, `build_line_starts`, `offset_to_line`)
- 1 new dependency (`pulldown-cmark`)
