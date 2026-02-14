# Implementation Plan: Golden-File Test Suite — Core

**Branch**: `021-prd-golden-file-tests` | **Date**: 2026-02-14 | **Spec**: [spec.md](./spec.md)
**Input**: PRD [docs/PRD/021-prd-golden-file-tests.md](../../docs/PRD/021-prd-golden-file-tests.md), AR [docs/AR/021-ar-golden-file-tests.md](../../docs/AR/021-ar-golden-file-tests.md), SEC [docs/SEC/021-sec-golden-file-tests.md](../../docs/SEC/021-sec-golden-file-tests.md)

## Summary

Implement an end-to-end golden-file regression test suite for the FORGE pipeline using `insta` snapshot testing with custom UUID/timestamp normalization. Three Markdown fixtures (small, medium, complex) are tested against both catalog and component strategies, producing 6 comparison tests plus extraction accuracy measurement (>95% target). Tests run as integration tests via `cargo test` calling the library API directly.

## Technical Context

**Language/Version**: Rust edition 2024, stable 1.93.0
**Primary Dependencies**: serde_json 1.x (existing), insta (NEW — latest stable, `json` feature, dev-dependency), regex 1.x (existing, for UUID pattern matching)
**Storage**: N/A — file-based test fixtures (read-only); insta `.snap` files generated alongside tests
**Testing**: `cargo test` (integration tests in `tests/`), `cargo insta review` for snapshot management
**Target Platform**: Development/CI (Linux, macOS)
**Project Type**: Single (Rust binary + library)
**Performance Goals**: N/A — test infrastructure (tests should complete in seconds, not minutes)
**Constraints**: No network access; OSCAL v1.2.0 schema compliance; hand-verified golden files only
**Scale/Scope**: 3 fixture tiers × 2 strategies = 6 golden-file tests + accuracy assertions + normalization unit tests

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Crate-First Architecture | ✅ PASS | Test infrastructure lives in `tests/` directory — standard Rust integration test location. No new crate needed for test-only code. |
| II. Rust-First | ✅ PASS | Pure Rust implementation. No FFI, no unsafe. |
| III. Contract-First Development | ✅ PASS | Interface contracts defined in AR (§Interface Definitions) and `contracts/golden_file_api.rs`. Types (`AccuracyReport`), functions (`normalize_for_comparison`, `measure_accuracy`), and constants defined before implementation. |
| IV. Test-First Development | ✅ PASS | This IS test infrastructure. TDD applies to the helper functions (normalization, accuracy measurement) — write unit tests for those first, then implement. |
| V. Complete Implementation | ✅ PASS | All tasks must be complete before merge. Verified via task checklist. |
| VI. Performance-First Design | ✅ N/A | Test infrastructure — no production performance requirements. Tests should not be unreasonably slow. |
| VII. Security-First Design | ✅ N/A | Per SEC review — test infrastructure only, no attack surface, synthetic fixtures only. |
| VIII. Error Handling Standards | ✅ PASS | Test failures produce descriptive messages. Missing fixture files produce clear panics with file paths. Pipeline errors propagate as test failures with full context. |
| IX. Observability & Debuggability | ✅ N/A | Test infrastructure — accuracy reports serve as observability. |
| X. Simplicity & Pragmatism | ✅ PASS | Using `insta` (existing ecosystem tool) instead of building custom framework. Single test file. Minimal new code. |
| XI. Current Dependency Policy | ✅ PASS | `insta` added at latest stable version as dev-dependency. Apache-2.0 license. No known advisories. |

**Gate Result**: ✅ ALL PASS — No violations. Proceed to implementation.

## Project Structure

### Documentation (this feature)

```text
specs/021-prd-golden-file-tests/
├── plan.md              # This file
├── spec.md              # Feature specification (synthesized from PRD/AR/SEC)
├── research.md          # Phase 0: research decisions
├── data-model.md        # Phase 1: data model and type definitions
├── quickstart.md        # Phase 1: developer quickstart guide
├── contracts/           # Phase 1: interface contracts
│   └── golden_file_api.rs
└── tasks.md             # Phase 2 output (NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
tests/
├── golden_file_tests.rs              # NEW: Integration test entry point
│                                     #   - normalize_for_comparison()
│                                     #   - measure_accuracy() + AccuracyReport
│                                     #   - golden_{small,medium,complex}_{catalog,component} tests
│                                     #   - Unit tests for normalization and accuracy helpers
├── fixtures/
│   └── golden/                       # NEW: Golden-file fixture directory
│       ├── small/
│       │   ├── input.md              # 1 section, 3-5 simple requirements
│       │   ├── expected-catalog.json # Hand-verified, schema-valid
│       │   └── expected-component-definition.json
│       ├── medium/
│       │   ├── input.md              # 3-5 sections, 10-15 requirements, citations
│       │   ├── expected-catalog.json
│       │   └── expected-component-definition.json
│       └── complex/
│           ├── input.md              # 5+ sections, 20+ requirements, citations, cross-refs
│           ├── expected-catalog.json
│           └── expected-component-definition.json
└── snapshots/                        # Auto-generated by insta (tracked in VCS)
    └── golden_file_tests__*.snap

Cargo.toml                            # MODIFIED: Add insta dev-dependency
```

**Structure Decision**: Single integration test file (`tests/golden_file_tests.rs`) following existing project patterns. Fixtures in `tests/fixtures/golden/`. No new crate — test-only code does not warrant a crate boundary.

## Complexity Tracking

No violations to justify — all constitution gates pass.

## Post-Design Constitution Re-Check

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Crate-First | ✅ PASS | No new crate. Test code in standard location. |
| III. Contract-First | ✅ PASS | Contracts defined in `contracts/golden_file_api.rs` and `data-model.md`. |
| IV. Test-First | ✅ PASS | Normalization and accuracy helpers will be TDD'd. Golden-file tests themselves ARE the tests. |
| X. Simplicity | ✅ PASS | ~200 lines of test code + fixtures. Using proven ecosystem tools. |
| XI. Dependency Policy | ✅ PASS | `insta` is latest stable, Apache-2.0, actively maintained, dev-dependency only. |

**Post-Design Gate Result**: ✅ ALL PASS — Design is constitution-compliant. Ready for task generation (`/speckit.tasks`).

---

## Implementation Summary

### Phase 0 Artifacts
- [research.md](./research.md) — 6 research decisions documented (framework selection, normalization, fixture design, accuracy measurement, test organization, insta version)

### Phase 1 Artifacts
- [data-model.md](./data-model.md) — Entity definitions (normalization, AccuracyReport, fixture layout, extraction logic)
- [contracts/golden_file_api.rs](./contracts/golden_file_api.rs) — Function signatures, types, and constants
- [quickstart.md](./quickstart.md) — Developer guide for running, updating, and creating golden-file tests

### Phase 2 (next: `/speckit.tasks`)
- `tasks.md` — Actionable, dependency-ordered implementation tasks
