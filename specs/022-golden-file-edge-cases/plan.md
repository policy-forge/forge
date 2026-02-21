# Implementation Plan: Golden File Edge Cases (WI-22)

**Branch**: `022-golden-file-edge-cases` | **Date**: 2026-02-21 | **Spec**: [spec.md](./spec.md)
**Input**: [spec.md](./spec.md) · [PRD](../../docs/PRD/022-prd-golden-file-edge-cases.md) · [AR](../../docs/AR/022-ar-golden-file-edge-cases.md) · [SEC](../../docs/SEC/022-sec-golden-file-edge-cases.md)

## Summary

Implement WI-22 by extending the existing WI-21 golden-file test harness with edge-case fixtures and assertions for EC-1 through EC-10 (excluding EC-8). The plan adds helper-driven tests for descriptive failures, warning-inclusive outputs, malformed citation retention, stable ID comparison pairs, and multi-issue validation reporting, with dual-strategy coverage for EC-1/2/3/4/5/6/7/10 and single strategy-agnostic coverage for EC-9. It also includes supplemental Should-Have coverage fixtures for unusual citation placement and parameter-like content preservation.

## Technical Context

**Language/Version**: Rust edition 2024, stable 1.93.0  
**Primary Dependencies**: Existing `serde_json`, `insta` (`json` feature), `tempfile`, `regex`, `tracing`; no new crate dependencies required  
**Storage**: File-based fixtures and expected outputs under `tests/fixtures/edge-cases/`; snapshots under `tests/snapshots/`  
**Testing**: `cargo test` integration tests + `insta` snapshots + schema/validation checks; quality gates via `cargo fmt --check` and `cargo clippy -- -D warnings`  
**Target Platform**: Local CLI and CI (macOS/Linux/Windows)  
**Project Type**: Single Rust crate (`forge`) with integration tests  
**Performance Goals**: Performance benchmarking is explicitly out of scope for this feature (FR-012, PRD W-2)  
**Constraints**: Markdown-only fixtures; EC-8 excluded; failure assertions use required substrings; metadata defaults are fixed (`title` filename stem, `version` `0.0.0`, `author` `Unknown`); malformed citations must keep `prop name="url-status" value="unvalidated"`; non-whitespace normative text changes must trigger new stable IDs  
**Scale/Scope**: 9 parent edge cases total (8 dual-strategy + 1 strategy-agnostic), plus 2 supplemental Should-Have scenarios for citation placement and parameter-like content

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Gate | Status | Notes |
|------|--------|-------|
| Constitution-derived gates available | ✅ Pass | `.specify/memory/constitution.md` is ratified and provides enforceable planning and implementation constraints |
| Test-first workflow | ✅ Pass | PRD/AR require TDD behavior for edge-case harness additions |
| Harness consistency | ✅ Pass | Selected architecture extends WI-21 golden harness rather than replacing framework |
| Scope boundary compliance | ✅ Pass | EC-8 excluded; PDF/DOCX and performance benchmarking remain out of scope |
| Security posture | ✅ Pass | SEC document marks feature as test-only, no new attack surface |
| Dependency policy | ✅ Pass | No new production dependencies; reuse existing dev/test stack |

**Gate Result**: ✅ PASS — proceed to Phase 0 research.

## Project Structure

### Documentation (this feature)

```text
specs/022-golden-file-edge-cases/
├── plan.md                               # This file (/speckit.plan output)
├── research.md                           # Phase 0 output
├── data-model.md                         # Phase 1 output
├── quickstart.md                         # Phase 1 output
├── checklists/
│   ├── requirements.md                   # Spec quality + final gate evidence
│   └── actionability-review.md           # SC-006 stakeholder actionability evidence
├── contracts/
│   └── rust-test-harness.md              # Phase 1 output (Rust helper contract)
└── tasks.md                              # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
tests/
├── golden_file_tests.rs                  # EXISTING WI-21 harness (reused)
├── golden_edge_case_tests.rs             # NEW: WI-22 edge-case test suite
├── fixtures/
│   ├── golden/                           # EXISTING WI-21 fixtures
│   └── edge-cases/                       # NEW WI-22 fixtures
│       ├── ec01-no-headings/
│       │   ├── input.md
│       │   └── expected-error.txt
│       ├── ec02-compound-atomic/
│       │   ├── input.md
│       │   ├── expected-catalog.json
│       │   └── expected-component-definition.json
│       ├── ec03-empty-sections/
│       │   ├── input.md
│       │   ├── expected-catalog.json
│       │   └── expected-warnings.txt
│       ├── ec04-missing-metadata/
│       │   ├── input.md
│       │   ├── expected-catalog.json
│       │   └── expected-warnings.txt
│       ├── ec05-whitespace-only/
│       │   ├── input-original.md
│       │   └── input-whitespace-variant.md
│       ├── ec06-substantive-change/
│       │   ├── input-original.md
│       │   ├── input-changed.md
│       │   └── expected-warnings.txt
│       ├── ec07-malformed-citation/
│       │   ├── input.md
│       │   └── expected-catalog.json
│       ├── ec09-file-not-found/
│       │   └── expected-error.txt
│       ├── ec10-multiple-errors/
│           ├── input.md
│           └── expected-errors.txt
│       ├── ec-citation-unusual-positions/
│       │   ├── input.md
│       │   ├── expected-catalog.json
│       │   └── expected-component-definition.json
│       └── ec-parameter-like-content/
│           ├── input.md
│           ├── expected-catalog.json
│           └── expected-component-definition.json
└── snapshots/
    ├── golden_edge_case_tests__ec02_catalog.snap
    ├── golden_edge_case_tests__ec02_component.snap
    ├── golden_edge_case_tests__strategy_matrix_catalog.snap
    ├── golden_edge_case_tests__strategy_matrix_component.snap
    └── golden_edge_case_tests__ec10_validation_errors.snap
```

**Structure Decision**: Keep single-project Rust layout and extend test infrastructure only. Use one dedicated integration test file for WI-22 while reusing WI-21 normalization/snapshot conventions and avoiding framework churn.

## Phase 0: Research

Phase 0 research is complete in [research.md](./research.md). All planning clarifications are resolved, including metadata defaults, strategy applicability, substantive change semantics, malformed citation annotation, and stable failure feedback assertions.

## Phase 1: Design & Contracts

Phase 1 artifacts are complete:

- [data-model.md](./data-model.md)
- [contracts/rust-test-harness.md](./contracts/rust-test-harness.md)
- [quickstart.md](./quickstart.md)

## Implementation Order

1. Create `tests/fixtures/edge-cases/` scaffold for EC-1/2/3/4/5/6/7/9/10
2. Add helper functions for expected-error, warning capture/assertion, and stability ID comparison
3. Implement EC-1 and EC-9 failure-path tests with substring assertions and non-zero exits
4. Implement EC-2/3/4/7 output golden tests with strategy-specific expected artifacts
5. Implement EC-5/6 paired-fixture stability tests (same IDs for whitespace-only, changed IDs for non-whitespace text edits)
6. Implement EC-10 validation aggregation test ensuring all issue categories are returned
7. Implement supplemental S-1/S-2 edge fixtures (citation placement and parameter-like content preservation)
8. Apply dual-strategy matrix (EC-1/2/3/4/5/6/7/10) and keep EC-9 single strategy-agnostic
9. Validate full suite with `cargo test`, `cargo fmt --check`, and `cargo clippy -- -D warnings`
10. Add explicit fixture-validity assertions that enforce FR-012 scope boundaries (no EC-8 fixtures and no benchmark artifacts)

## Security Requirements Integration

| Source | Requirement | Planned Handling |
|--------|-------------|------------------|
| SEC-022 (N/A review) | Test-only infrastructure; no new attack surface | Keep fixtures synthetic and local-only |
| AR guardrail | Do not hardcode exact full error messages | Use required substring assertions for stable semantics |
| AR guardrail | Do not skip dual-strategy testing when applicable | Explicit matrix in FR-011 and test scaffold |
| AR guardrail | Markdown-only edge cases | Exclude PDF/DOCX and EC-8 |

## Complexity Tracking

No constitution violations require justification.

## Post-Design Constitution Re-Check

| Gate | Status | Notes |
|------|--------|-------|
| Constitution compliance | ✅ Pass | Plan remains aligned with ratified constitution principles |
| Test-first workflow | ✅ Pass | Design is test-centric and maps all FRs to testable artifacts |
| Harness consistency | ✅ Pass | Design extends existing WI-21 test strategy and snapshots |
| Scope boundary compliance | ✅ Pass | EC-8/performance remain excluded and documented |
| Security posture | ✅ Pass | Still test-only; no production attack surface changes |

**Post-Design Gate Result**: ✅ PASS — design gate satisfied (historical milestone before `/speckit.tasks`).

## Phase 2 Planning Readiness

- Phase 0 artifacts: complete
- Phase 1 artifacts: complete
- Agent context update: complete
- Task generation: complete (`tasks.md` present)
- Analysis remediation: complete (scope-boundary executable guard + AR EC-10 traceability alignment)
- Implementation execution: complete (all WI-22 tasks marked done)
- Next step: use `/speckit.analyze` for post-implementation consistency verification as needed

## Implementation Readiness Decision

- Status: Approved for implementation handoff on 2026-02-21
- Evidence basis: constitution gates pass, 100% requirement-task coverage, and no remaining critical/high cross-artifact inconsistencies
- Note: PRD/AR lifecycle headers may remain Draft/Proposed for document governance, while WI-22 execution readiness is tracked by this feature plan and checklist evidence

## WI-22 Acceptance Checklist

- [x] Edge-case fixture set created for EC-1/2/3/4/5/6/7/9/10
- [x] Supplemental S-1/S-2 fixtures added
- [x] Dedicated WI-22 integration suite added in `tests/golden_edge_case_tests.rs`
- [x] Required snapshots present:
  - `tests/snapshots/golden_edge_case_tests__ec02_catalog.snap`
  - `tests/snapshots/golden_edge_case_tests__ec02_component.snap`
  - `tests/snapshots/golden_edge_case_tests__strategy_matrix_catalog.snap`
  - `tests/snapshots/golden_edge_case_tests__strategy_matrix_component.snap`
  - `tests/snapshots/golden_edge_case_tests__ec10_validation_errors.snap`
- [x] FR-012 scope guards implemented in `tests/fixture_validity_test.rs`
