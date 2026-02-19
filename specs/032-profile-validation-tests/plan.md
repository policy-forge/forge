# Implementation Plan: Profile Validation and Golden-File Tests (WI-32)

**Branch**: `032-profile-validation-tests` | **Date**: 2026-02-18 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/032-profile-validation-tests/spec.md`

## Summary

WI-32 adds three automated quality assurance layers for OSCAL Profile generation: (1) **Schema validation** — extends `OscalModelType` with a `Profile` variant, embeds the NIST OSCAL v1.2.0 Profile JSON schema via `include_str!`, and validates generated Profiles against it using the existing `validate_artifact()` infrastructure from WI-19; (2) **Golden-file regression tests** — uses `insta` snapshot testing (consistent with WI-21 Catalog/Component pattern) to lock down expected Profile output for representative scenarios; (3) **Edge case tests** — covers empty control selection, all-controls selection, duplicate IDs, mutually-exclusive flag errors, invalid catalog paths, and malformed JSON inputs.

This is a **test-only** work item with one production code change: adding `OscalModelType::Profile` to `src/validate/mod.rs` (FR-000). No new Profile generation features or CLI flags are introduced. The OSCAL v1.2.0 Profile schema is embedded at compile time from the official NIST release.

**WI-31 status (critical):** Parameter tailoring (`--set-param`, `modify` section) is **NOT YET IMPLEMENTED** in the codebase. Tests requiring WI-31 (FR-003, FR-008, golden-file scenario 3) are included as `#[ignore]`-annotated stubs to be enabled when WI-31 lands.

## Technical Context

**Language/Version**: Rust, Edition 2024, stable 1.93.0
**Primary Dependencies**: `jsonschema` 0.41.0 (already in Cargo.toml), `insta` 1.46.3 with `json` feature (already in Cargo.toml), `tempfile` 3.25.0 (already in Cargo.toml) — **NO NEW DEPENDENCIES REQUIRED**
**Storage**: `schemas/oscal_profile_schema.json` (compile-time embedded via `include_str!`); `tests/snapshots/*.snap` (insta golden files checked into git)
**Testing**: `cargo test`, `cargo insta review` for snapshot management
**Target Platform**: Developer machine and CI (Linux/macOS)
**Project Type**: Single Rust project (test-only addition)
**Performance Goals**: N/A — test suite
**Constraints**: No new crate dependencies; reuse WI-19 `validate_artifact()` and WI-21 insta pattern; test-only (plus minimal `src/validate/mod.rs` extension)
**Scale/Scope**: ~18 test functions across 2 new integration test files; 2–3 new `.snap` files

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Crate-First Architecture | ✅ PASS | Test-only WI; `src/validate/mod.rs` extension is minimal (adds one enum variant + 3 match arms); no new crate |
| II. Rust-First | ✅ PASS | Pure Rust; no FFI; no unsafe |
| III. Contract-First Development | ✅ PASS | No new public API; FR-000 extends existing `OscalModelType` enum |
| IV. Test-First Development | ✅ PASS | This WI is entirely tests; TDD applies by definition |
| V. Complete Implementation | ✅ PASS | All tasks complete before merge |
| VI. Performance-First Design | N/A | Test suite only |
| VII. Security-First Design | ✅ PASS | Test-only; synthetic fixture data; no network access; SEC doc confirms N/A |
| VIII. Error Handling Standards | ✅ PASS | Reuses existing `ValidateError` and `ForgeError` types; no new error types |
| IX. Observability | N/A | Test suite |
| X. Simplicity | ✅ PASS | Strict YAGNI; reuse WI-19/WI-21; no new frameworks; `#[ignore]` pattern for WI-31 stubs |
| XI. Current Dependency Policy | ✅ PASS | No new dependencies; all existing deps are current |

**Gates result**: All applicable gates PASS. No violations requiring justification.

**Post-design re-check (Phase 1):** No new dependencies introduced. Normalization helper added to `tests/common/mod.rs` — this is an additive change to an existing file (not a new crate). Constitution still fully compliant.

## Project Structure

### Documentation (this feature)

```text
specs/032-profile-validation-tests/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
schemas/
├── oscal_catalog_schema.json         # existing (WI-19)
├── oscal_component_schema.json       # existing (WI-19)
└── oscal_profile_schema.json         # NEW (FR-000): NIST OSCAL v1.2.0 Profile JSON schema

src/validate/
└── mod.rs                            # MODIFY (FR-000): add Profile variant to OscalModelType,
                                      #   update load_schema(), detect_model_type(), Display

tests/
├── common/
│   └── mod.rs                        # ADD: normalize_for_snapshot() utility function
├── profile_validation_tests.rs       # NEW: schema validation tests + edge case tests
├── profile_golden_file_tests.rs      # NEW: insta snapshot golden-file tests
└── snapshots/
    ├── [existing .snap files]        # unchanged
    ├── profile_golden_file_tests__golden_include_only.snap     # NEW (post cargo insta accept)
    └── profile_golden_file_tests__golden_exclude_only.snap     # NEW (post cargo insta accept)
```

**Structure Decision**: Single project (Option 1). Two new integration test files follow the WI-21 `golden_file_tests.rs` pattern. Shared normalization added to existing `tests/common/mod.rs` (additive). No new crate needed.

## Implementation Guardrails (from AR)

- **DO NOT** build a new validation framework — reuse WI-19 `validate_artifact()` with `OscalModelType::Profile`
- **DO NOT** build a new golden-file framework — use `insta::assert_json_snapshot!()` as in WI-21
- **DO NOT** modify `tests/golden_file_tests.rs` — WI-21 infrastructure is off-limits
- **DO NOT** modify Profile generation code (`src/oscal/profile.rs`, `src/cli/profile.rs`)
- **DO NOT** hard-code UUIDs or timestamps in snapshots — normalize before comparison
- **MUST** add `OscalModelType::Profile` to enum + `load_schema()` + `detect_model_type()` (FR-000)
- **MUST** include at least 2 active golden-file scenarios: include-only, exclude-only
- **MUST** test edge cases: empty selection, all controls (10), conflicting flags (error)
- **MUST** ensure `cargo test` passes with zero failures
- **MUST** use `validate_artifact()` (not `run_full_validation()`) — avoid semantic validation for Profile

## Key Implementation Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Schema validation function | `validate_artifact()` | `run_full_validation()` invokes `SemanticValidator` (Catalog-specific); schema-only validation is correct for Profile |
| WI-31 tests | `#[ignore]` stubs | WI-31 not implemented; preserves test structure for future enablement |
| Normalization location | `tests/common/mod.rs` (additive) | Existing shared utilities location; avoids modifying WI-21 code |
| Profile schema source | NIST v1.2.0 release asset | Same source as catalog/component schemas; consistent versioning |
| Test fixture catalog | Inline `tempfile::NamedTempFile` | WI-30 doesn't parse catalog content; path existence check is all that's needed |
| Edge case EC-6 behavior | Error assertion | `--include` + `--exclude` returns clap error at parse time; not a golden-file scenario |

## Phase 0: Research Output

See [research.md](./research.md) for all decisions and rationale.

**All NEEDS CLARIFICATION resolved:**
- WI-31 status: NOT implemented → `#[ignore]` stubs
- OSCAL Profile schema URL: confirmed at NIST v1.2.0 release
- Normalization utility: `tests/common/mod.rs`
- Validation function: `validate_artifact()` not `run_full_validation()`

## Phase 1: Design Output

See [data-model.md](./data-model.md) for Profile JSON structure, normalization rules, and fixture design.
See [quickstart.md](./quickstart.md) for running tests and managing snapshots.

**No contracts/** needed — this is a test-only WI with no new API contracts.

## Complexity Tracking

> No constitution violations. Complexity tracking section not required.
