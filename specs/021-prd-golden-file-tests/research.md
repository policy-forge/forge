# Research: Golden-File Test Suite — Core

**Branch**: `021-prd-golden-file-tests` | **Date**: 2026-02-14

---

## R-1: Snapshot Testing Framework Selection

**Decision**: Use `insta` crate (with `json` feature) for snapshot management.

**Rationale**: `insta` is the industry-standard Rust snapshot testing crate, listed in the constitution's testing stack (Principle IV). It provides:
- JSON-structural diffs via `assert_json_snapshot!` (satisfies PRD M-5)
- `cargo insta review` for interactive golden-file updates (satisfies PRD S-1)
- Automatic snapshot storage in `snapshots/` directories tracked in VCS
- CI-compatible — `cargo insta test` fails on pending snapshots (satisfies PRD M-6)

**Alternatives considered**:
- **Custom `serde_json` framework** (AR Option 2): Requires building comparison, diff reporting, and update mechanisms from scratch. More code, inferior diff quality.
- **`expect-test`** (AR Option 3): Inline snapshots impractical for multi-KB OSCAL JSON documents.
- **`assert_json_diff`**: Good for comparison but lacks snapshot management, update workflow, and the ergonomic `cargo insta` CLI.

---

## R-2: Non-Deterministic Field Normalization Strategy

**Decision**: Walk JSON tree recursively, replacing UUID-format strings with a fixed placeholder (`00000000-0000-0000-0000-000000000000`) and `last-modified` field values with a fixed timestamp (`2026-01-01T00:00:00Z`). Sort all map keys alphabetically.

**Rationale**: The FORGE pipeline uses deterministic UUID v5 generation (WI-7), but timestamps are inherently non-deterministic (`chrono::Utc::now()`). Normalizing both provides resilience against future UUID logic changes (PRD M-4). Key sorting ensures deterministic serialization (PRD technical constraint).

**Alternatives considered**:
- **insta redactions**: `insta` has a `redactions` feature for inline field masking, but it requires specifying each field path explicitly. Custom normalization via tree-walking is more robust for fields that appear at arbitrary depths.
- **Regex on serialized string**: Fragile — could match UUID-like strings inside prose content. Tree-walking with type awareness is safer.
- **Normalize only timestamps**: Would rely entirely on deterministic UUIDs. If UUID generation changes, all tests break. Normalizing both is more resilient.

---

## R-3: Fixture Design Strategy

**Decision**: Three complexity tiers — small, medium, complex — each in `tests/fixtures/golden/<tier>/input.md` with corresponding `expected-catalog.json` and `expected-component-definition.json`.

**Rationale**: Multiple complexity levels catch different bug categories:
- **Small** (1 section, 3-5 reqs): Validates basic structural mapping, Catalog/Component generation, metadata, JSON format, stable UUIDs (M-1, M-3, M-4, M-5, M-7, M-8)
- **Medium** (3-5 sections, 10-15 reqs, 1-2 citations): Validates multi-section hierarchy, back matter, traceability (adds M-9, M-10)
- **Complex** (5+ sections, 20+ reqs, citations, cross-refs, compound statements): Validates all M-1 through M-11 including atomization (M-2) and prop/link usage (M-11)

**Alternatives considered**:
- **Single comprehensive fixture**: Harder to diagnose failures; mixes concerns.
- **More than 3 levels**: Diminishing returns; WI-22 handles edge cases.
- **Reuse existing `full_policy.md`**: Possible for medium tier, but dedicated fixtures allow precise control over what each exercises.

---

## R-4: Accuracy Measurement Approach

**Decision**: Count expected controls/implemented-requirements by extracting control IDs from both expected and actual JSON. Accuracy = (intersection count / expected count) × 100. Report per-fixture and aggregate. Identify missed requirements by listing control IDs present in expected but absent in actual.

**Rationale**: Control-level granularity is the natural unit for OSCAL artifacts. For Catalog strategy, count controls across all groups. For Component strategy, count implemented-requirements in the documentary component. This aligns with PRD M-8 (>95% target) and S-3 (identify missed requirements by ID).

**Alternatives considered**:
- **Requirement text comparison**: Too brittle — minor prose changes would count as failures even if the requirement is correctly mapped.
- **Structural subtree matching**: Over-engineered for the accuracy metric; better suited for C-1 (partial matching, deferred).
- **Full JSON equality after normalization**: This is what insta does — accuracy measurement is complementary, not a replacement.

---

## R-5: Test Organization Pattern

**Decision**: Single integration test file `tests/golden_file_tests.rs` containing all golden-file test functions, with helper functions for normalization and accuracy measurement defined inline. No separate crate or module directory needed.

**Rationale**: Follows existing test patterns in the codebase (e.g., `tests/catalog_pipeline_test.rs`, `tests/component_pipeline_test.rs`). Keeps test infrastructure simple per constitution Principle X. Helper functions are small enough to live in one file.

**Alternatives considered**:
- **Separate `tests/golden/` module directory**: More structure than needed for ~6 test functions + 2 helper functions. Can refactor if tests grow significantly.
- **Shared helpers in `tests/common/`**: Possible but adds coupling. Normalization and accuracy are specific to golden-file tests.

---

## R-6: `insta` Crate Version and Features

**Decision**: Use `insta` 1.46.3 (latest stable as of 2026-02-14) with `json` feature enabled. Add as dev-dependency only. Pin exact version per constitution Principle XI.

**Rationale**: The `json` feature enables `assert_json_snapshot!` which provides structural JSON comparison and JSON-aware diffing. Latest stable ensures no known vulnerabilities. Dev-dependency means it doesn't affect the production binary.

**Verification**: `insta` is Apache-2.0 licensed (constitution-compliant), actively maintained, widely used in the Rust ecosystem.

---

## R-7: Extraction Accuracy Threshold

**Decision**: The extraction accuracy threshold is **>= 95.0% (inclusive)**. A fixture with exactly 95.0% accuracy passes.

**Rationale**: PRD M-8 states ">95%" and EC-4 flags the boundary ambiguity ("clarify if >=95% or strictly >95%"). Inclusive (>=) is the pragmatic choice: achieving exactly 95.0% demonstrates the pipeline meets the target. Strictly >95% would reject 95.0% (e.g., 19/20 = 95.0%), which is unnecessarily punitive given the metric's purpose is to confirm "at least 95% of requirements are correctly extracted."

**Alternatives considered**:
- **Strictly >95%**: Would require 96%+ effectively for small fixture counts (e.g., 20 reqs → must get 20/20 since 19/20 = 95.0% would fail). Over-constraining.
- **>= 95.5%**: Arbitrary — no basis for non-round threshold.
