# Feature Specification: Golden-File Test Suite — Core

**Branch**: `021-prd-golden-file-tests` | **Date**: 2026-02-14
**Source PRD**: [docs/PRD/021-prd-golden-file-tests.md](../../docs/PRD/021-prd-golden-file-tests.md)
**Source AR**: [docs/AR/021-ar-golden-file-tests.md](../../docs/AR/021-ar-golden-file-tests.md)

---

## Summary

Implement a golden-file regression test suite that compares the FORGE pipeline's actual OSCAL JSON output against hand-verified expected outputs for Markdown policy fixtures of varying complexity. Uses `insta` snapshot testing with custom UUID/timestamp normalization. Measures extraction accuracy against a >95% target (MS-4 exit criterion).

## Requirements (from PRD)

### Must Have

| ID | Requirement |
|----|-------------|
| M-1 | 3+ Markdown fixtures: small (1 section, 3-5 reqs), medium (3-5 sections, 10-15 reqs), complex (5+ sections, 20+ reqs with citations) |
| M-2 | Hand-verified expected OSCAL Catalog JSON golden files passing OSCAL v1.2.0 schema validation |
| M-3 | Hand-verified expected OSCAL Component Definition JSON golden files passing OSCAL v1.2.0 schema validation |
| M-4 | Normalize non-deterministic fields (UUIDs, `last-modified` timestamps) before comparison |
| M-5 | Clear diff output showing JSON path and divergent values on failure |
| M-6 | All tests runnable via `cargo test` and passing in CI |
| M-7 | Test both `--strategy catalog` and `--strategy component` for each fixture |
| M-8 | Measure and report extraction accuracy >= 95% per fixture and overall (see research.md R-7: inclusive threshold) |
| M-9 | Golden files validate all parent PRD Must Haves (M-1 through M-11) |

### Should Have

| ID | Requirement |
|----|-------------|
| S-1 | Golden file update mechanism (env var or `cargo insta review`) |
| S-2 | Determinism verification (identical output across consecutive runs) |
| S-3 | Accuracy report identifies each missed requirement by stable ID |

### Could Have

| ID | Requirement |
|----|-------------|
| C-1 | Partial golden-file matching (assert on JSON subtrees) |
| C-2 | Fixture with baseline profile reference for component-first testing |

## Architecture Decisions (from AR)

- **Framework**: `insta` crate with `json` feature for snapshot testing
- **Normalization**: Custom `normalize_for_comparison()` function replaces UUIDs with placeholder, `last-modified` with fixed timestamp, sorts map keys
- **Test Style**: Integration tests calling FORGE library API directly (no CLI binary spawning)
- **Fixture Storage**: `tests/fixtures/golden/{small,medium,complex}/`
- **Dual Files**: Raw `.json` for schema validation + `.snap` for insta comparison
- **Accuracy**: Count controls in expected vs actual, report percentage and missed IDs

## Security (from SEC)

Security review not required — test infrastructure only. No user-facing functionality, no data ingestion, no external exposure. Fixtures are synthetic test data.

## Constraints

- Tests must run via `cargo test` without external tools or network access
- JSON output must use ordered/sorted keys for deterministic serialization
- Expected golden files must pass OSCAL v1.2.0 schema validation
- Normalization must be idempotent
- Test functions call library API directly — no process spawning

## Dependencies

- **Requires**: WI-13 (Catalog pipeline), WI-18 (Component pipeline), WI-19 (Schema validation)
- **Blocks**: WI-22 (Edge case tests), WI-25 (Phase 1 release)
