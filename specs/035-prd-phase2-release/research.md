# Research: Phase 2 Integration Testing & v0.2.0 Release

**Feature**: WI-35 | **Date**: 2026-02-19
**Status**: Complete — all unknowns resolved

---

## Overview

WI-35 is a pure integration testing and release sprint. All Phase 2 features (WI-26–WI-34) are already implemented. No new external technologies are introduced. The research phase confirms existing infrastructure is sufficient and documents decisions already made in the PRD and AR.

---

## Decision 1: Integration Testing Framework

**Decision**: Use standard `cargo test` with integration test modules in `tests/` directory.

**Rationale**:
- Consistent with WI-25 (Phase 1 release) and all other WIs — no learning curve.
- `env!("CARGO_BIN_EXE_forge")` provides reliable binary path for subprocess invocation.
- `cargo nextest` compatible for faster local runs.
- No additional test framework crates needed (`assert_cmd`, `trycmd`, etc. — all spike-rejected in WI-25 research).

**Alternatives considered**: `assert_cmd` crate (rejected: adds dependency for no meaningful benefit given existing subprocess helpers); `trycmd` snapshot testing (rejected: overly rigid for cross-feature scenarios that need dynamic assertions).

---

## Decision 2: Test File Organization

**Decision**: Modular test files, one per cross-cutting concern (4 files).

**Rationale**:
- Enables `cargo test integration_round_trip` to run a single category.
- Keeps each file focused and < 400 lines.
- Easier to maintain as Phase 3 adds features.

**Alternatives considered**: Single monolithic `tests/phase2_release.rs` (AR Option 3 — rejected: becomes unwieldy, mixes concerns); folding into existing test files (rejected: existing files belong to specific WIs and should not accumulate cross-cutting concerns).

---

## Decision 3: Round-Trip Equivalence Verification

**Decision**: Use `serde_json::Value` deserialized comparison, not string comparison.

**Rationale**:
- JSON key ordering is not guaranteed to be stable across serialization/deserialization.
- XML uses element ordering which may differ from JSON field ordering.
- `serde_json::Value` comparison is semantic, not syntactic.
- Pattern already established in `tests/round_trip_test.rs` via `forge::testing::assert_semantic_equivalence`.

**Alternatives considered**: String diff comparison (rejected: brittle, fails on semantically equivalent documents with different ordering); `insta` snapshot (useful for golden-file regression, not for dynamic cross-format equivalence).

---

## Decision 4: Cross-Feature Test Fixtures

**Decision**: Use a new synthetic fixture combining normative language, advisory language, and parameterized requirements.

**Rationale**:
- Existing fixtures (`full_policy.md`, golden/medium) contain normative language but were not designed to exercise all three Phase 2 features simultaneously.
- A purpose-built fixture ensures deterministic, well-understood test inputs.
- Small fixture keeps test execution fast.

**Fixture design** (embedded in test file as `const`):
```markdown
# Access Control
- Systems must enforce multi-factor authentication
- Administrators should review access logs weekly
- Passwords must be changed within 90 days
- Sessions may be extended up to 8 hours

# Data Protection
- All data at rest must be encrypted using AES-256
- Encryption keys should be rotated annually
```
This fixture produces: normative props (`must`, `shall`), advisory props (`should`, `may`), time-window params (90 days, 8 hours), nominal params (AES-256).

---

## Decision 5: Regression Strategy for Phase 2 Additive Changes

**Decision**: Assert presence of expected core fields; allow new Phase 2 `prop` and `param` elements; do NOT assert their absence.

**Rationale**:
- Phase 2 enrichment passes (WI-33 modality detection, WI-34 parameter extraction) add `prop` and `param` elements that Phase 1 golden files do not contain.
- Phase 1 golden-file tests (in `golden_file_tests.rs`) use `insta` snapshot comparison. If snapshots were updated during WI-33/WI-34, they already reflect Phase 2 additions — no changes needed.
- If snapshots were NOT updated, the regression tests will fail, revealing which golden files need updating. This is the desired behavior.

**Alternative considered**: Update all Phase 1 golden files before writing regression tests (rejected: circular — regression tests should reveal unexpected changes, not be pre-satisfied).

---

## Decision 6: Release Workflow

**Decision**: Manual three-gate sequence: `cargo test` → `cargo clippy` → `cargo fmt --check` → version bump → CHANGELOG → commit → tag.

**Rationale**:
- Consistent with WI-25 pattern.
- Single developer; automation overhead not justified (constitution Principle X: YAGNI).
- `scripts/release.sh` (AR Option 2) rejected: adds maintenance burden for trivial automation.

---

## Infrastructure Survey

### Existing Test Infrastructure Available for Reuse

| Component | Location | Status |
|-----------|----------|--------|
| Binary invocation helper | `tests/e2e_release_test.rs:forge_bin()` | Copy/adapt |
| Temp file creation helper | `tests/e2e_release_test.rs:create_temp_file()` | Copy/adapt |
| Semantic equivalence assertion | `src/testing/` (`forge::testing::assert_semantic_equivalence`) | Import directly |
| Round-trip deserialization | `forge::export::xml_deserializer`, `forge::export::yaml` | Import directly |
| Schema validation | `forge validate` subprocess | Subprocess invocation |
| Golden file fixtures | `tests/fixtures/golden/{small,medium,complex}/` | Load with `std::fs::read_to_string` |
| Comprehensive policy fixture | `tests/fixtures/full_policy.md` | Load directly |
| Sample profile | `tests/fixtures/sample_profile.json` | Load directly |

### Current Test Status

```
cargo test: 1049 passed, 3 ignored (31 suites, 3.51s)
cargo build: 12 crates compiled, 0 errors
cargo clippy: (TBD — run before release)
cargo fmt --check: (TBD — run before release)
Cargo.toml version: 0.1.0 (bump to 0.2.0 in Step 3.2)
CHANGELOG.md: does not exist (create in Step 3.3)
```

### Phase 2 Feature Status (at sprint start)

| WI | Feature | Status |
|----|---------|--------|
| WI-26 | XML output (`--format xml`) | ✅ Implemented + tested |
| WI-27 | YAML output (`--format yaml`) | ✅ Implemented + tested |
| WI-28 | Round-trip testing infrastructure | ✅ Implemented + tested |
| WI-29 | `forge export` subcommand | ✅ Implemented + tested |
| WI-30 | Profile generation (`forge profile`) | ✅ Implemented + tested |
| WI-31 | Profile parameter tailoring (`--set-param`) | ✅ Implemented + tested |
| WI-32 | Profile validation + golden files | ✅ Implemented + tested |
| WI-33 | Normative/advisory detection (modality props) | ✅ Implemented + tested |
| WI-34 | Parameter extraction (`param` elements) | ✅ Implemented + tested |

---

## NEEDS CLARIFICATION: None

All technical questions are resolved. No open research items.
