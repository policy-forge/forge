# Implementation Plan: WI-25 Phase 1 Release

**Branch**: `025-prd-phase1-release` | **Date**: 2026-02-14 | **Spec**: N/A (PRD-driven)
**Input**: PRD `docs/PRD/025-prd-phase1-release.md`, AR `docs/AR/025-AR-phase1-release.md`, SEC `docs/SEC/020-sec-validation-error-reporting.md`

## Summary

WI-25 is the Phase 1 release gate for FORGE v0.1.0. It integrates all 24 preceding work items, verifies end-to-end correctness through comprehensive integration tests, polishes the CLI user experience (help text, verbose/quiet flags, error messages), updates the README with verified usage examples, and configures `cargo-dist` for automated release packaging. The release is tagged only after all MS-4 exit criteria are verified: all M-requirements passing, golden-file accuracy >95%, `forge validate` working, and all CI gates green.

## Technical Context

**Language/Version**: Rust (edition 2024, stable 1.93.0)
**Primary Dependencies**: clap 4.x, serde 1.x, serde_json 1.x, pulldown-cmark 0.13.x, uuid 1.20.0, chrono 0.4, sha2 0.10.9, thiserror 2.0.18, tracing 0.1.44, anyhow 1.x, jsonschema 0.41.0, url 2.5 — all existing, no new runtime dependencies
**Dev Dependencies**: criterion 0.8.2, insta 1.46.3, tempfile 3.25.0 — all existing. NEW: `assert_cmd` and `predicates` NOT added (see research.md R1 — keep existing `std::process::Command` pattern)
**Build Tool**: cargo-dist (NEW — build/release tool only, not a runtime dependency)
**Storage**: Filesystem (read input .md, write output .json)
**Testing**: `cargo test` (unit + integration + golden-file + snapshot)
**Target Platform**: Linux x86_64, macOS x86_64 + ARM64, Windows x86_64
**Project Type**: Single Rust binary crate
**Performance Goals**: 50-page policy document conversion <30s (established in WI-24, verified by benchmarks)
**Constraints**: No new features; integration, polish, and release only; no new runtime dependencies

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Crate-First Architecture | PASS | Single binary crate established in WI-1; WI-25 does not change architecture |
| II. Rust-First | PASS | All code is Rust; no FFI or unsafe added |
| III. Contract-First Development | PASS | No new contracts; WI-25 verifies existing contracts end-to-end |
| IV. Test-First Development | PASS | Integration tests written before verification claims; TDD cycle followed |
| V. Complete Implementation | PASS | All prior WI tasks complete; WI-25 tasks will be completed before merge |
| VI. Performance-First Design | PASS | <30s benchmark verified (WI-24); no performance regression expected |
| VII. Security-First Design | PASS | SEC-020 review incorporated; SEC-1 through SEC-8 verified |
| VIII. Error Handling Standards | PASS | thiserror in library, anyhow in binary; error messages reviewed for consistency |
| IX. Observability & Debuggability | PASS | tracing wired via --verbose/--quiet; structured logging in place |
| X. Simplicity & Pragmatism | PASS | No new features; minimal changes; existing patterns reused |
| XI. Current Dependency Policy | PASS | No new runtime dependencies; cargo-dist is build-only tool |

**Gate Result**: PASS — no violations. Proceeding to implementation.

## Project Structure

### Documentation (this feature)

```text
specs/025-prd-phase1-release/
├── plan.md              # This file
├── research.md          # Phase 0 output — research findings
├── data-model.md        # Phase 1 output — N/A, no new models
├── quickstart.md        # Phase 1 output — implementation guide
└── tasks.md             # Phase 2 output (created by /speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── main.rs              # Entry point (verbose/quiet filter wiring)
├── lib.rs               # Module declarations and public API
├── error.rs             # ForgeError enum (thiserror)
├── pipeline.rs          # E2E pipeline orchestration
├── uuid.rs              # Deterministic UUID v5 generation
├── citation.rs          # Citation extraction
├── cli/
│   ├── mod.rs           # CLI definition (clap derive) — POLISH TARGET
│   ├── convert.rs       # Convert subcommand handler
│   └── validate.rs      # Validate subcommand handler
├── ingest/mod.rs        # File ingestion and fingerprinting
├── parse/
│   ├── mod.rs           # Heading hierarchy extraction
│   ├── clauses.rs       # Clause extraction
│   └── atomize.rs       # Compound requirement splitting
├── model/
│   ├── mod.rs           # PolicyDocument, PolicySection, PolicyRequirement
│   ├── frontmatter.rs   # YAML frontmatter parsing
│   ├── assemble.rs      # Pipeline assembly
│   └── trace.rs         # TraceLink model
├── oscal/
│   ├── mod.rs           # OSCAL module declarations
│   ├── catalog.rs       # Catalog builder
│   ├── parts.rs         # Statement parts
│   ├── metadata.rs      # OSCAL metadata assembly
│   └── back_matter.rs   # Back matter resources
├── export/mod.rs        # JSON export
└── validate/
    ├── mod.rs           # Schema validation
    ├── error_types.rs   # Validation error types
    ├── formatter.rs     # Error formatting (SEC-1 truncation)
    ├── report.rs        # Validation report
    └── semantic.rs      # Semantic validation

tests/
├── cli_integration.rs          # CLI tests — EXTEND for help/verbosity
├── catalog_pipeline_test.rs    # Catalog E2E — verify for M-req coverage
├── component_pipeline_test.rs  # Component E2E — verify for M-req coverage
├── golden_file_tests.rs        # Golden-file accuracy — verify >95%
├── validate_test.rs            # Validation tests — verify for M-6
├── atomize_integration.rs      # Atomization tests — verify for M-2
├── trace_integration.rs        # Traceability tests — verify for M-10
├── fixture_determinism_test.rs # Determinism tests — verify for M-8
├── fixture_validity_test.rs    # Fixture validity tests
├── adversarial_input_test.rs   # Adversarial input tests
├── pipeline_test.rs            # Pipeline tests
├── e2e_release_test.rs         # NEW: MS-4 verification integration tests
├── common/                     # Test helpers
├── fixtures/                   # Test fixtures
└── snapshots/                  # Insta snapshots

README.md                       # UPDATE: Usage examples, status table
dist-workspace.toml             # NEW: cargo-dist configuration
.github/workflows/release.yml   # NEW: Release CI workflow (cargo-dist generated)
```

**Structure Decision**: Single binary crate (established in WI-1). No structural changes in WI-25. New files are integration tests, README updates, and cargo-dist configuration.

## Implementation Phases

### Phase 1: Integration Tests for MS-4 Verification (PRD M-1, M-2, M-3, M-4, M-7)

**Goal**: Write integration tests that explicitly verify each parent PRD M-requirement end-to-end.

**New file**: `tests/e2e_release_test.rs`

Tests to implement:

1. **`test_m1_structural_hierarchy_extraction`** — Convert sample MD → verify groups present in Catalog JSON (M-1, AC-1)
2. **`test_m2_atomize_compound_statements`** — Convert MD with "must X and must Y" → verify separate controls (M-2, AC-2)
3. **`test_m3_valid_oscal_catalog_json`** — Convert → parse as JSON → verify catalog structure (M-3, AC-3)
4. **`test_m4_valid_component_definition`** — Convert with component strategy → verify structure (M-4, AC-4)
5. **`test_m5_metadata_fields_present`** — Verify uuid, title, last-modified, version, oscal-version (M-5, AC-5)
6. **`test_m6_validate_generated_catalog`** — Convert → write to temp file → `forge validate` → success (M-6, AC-6)
7. **`test_m6_validate_generated_component`** — Same for Component Definition (M-6, AC-6)
8. **`test_m7_json_output_format`** — Verify valid JSON output (M-7, AC-7)
9. **`test_m8_deterministic_uuids`** — Convert same input twice → compare UUIDs (M-8, AC-8)
10. **`test_m9_citations_in_back_matter`** — Convert MD with citations → verify back_matter.resources (M-9, AC-9)
11. **`test_m10_traceability_props`** — Convert → verify trace props on controls (M-10, AC-10)
12. **`test_m11_no_arbitrary_remarks`** — Convert → verify no arbitrary remarks fields (M-11)

**Extend file**: `tests/cli_integration.rs`

13. **`test_verbose_flag_shows_pipeline_stages`** — Run with `--verbose`, verify pipeline stage messages in stderr (S-1, AC-8)
14. **`test_quiet_flag_suppresses_output`** — Run with `--quiet`, verify minimal output (S-1, AC-9)
15. **`test_verbose_quiet_conflict_error`** — Both flags → clear error (S-1, EC-4)
16. **`test_help_text_lists_all_subcommands`** — `--help` contains convert, validate, --verbose, --quiet (M-5, AC-5, EC-3)
17. **`test_convert_help_lists_all_options`** — All args listed (M-5)
18. **`test_validate_help_lists_all_options`** — All args listed (M-5)
19. **`test_error_message_missing_file`** — Descriptive error, no panic (S-3, AC-11)
20. **`test_error_message_invalid_json_for_validate`** — Descriptive error (EC-5)
21. **`test_version_flag`** — `forge --version` outputs version string

**Extend file**: `tests/golden_file_tests.rs`

22. **Verify golden-file accuracy >95%** — Ensure existing suite covers M-3 threshold

### Phase 2: CLI Polish (PRD M-5, S-1, S-3)

**File**: `src/cli/mod.rs`

1. Enhance `#[command(about = "...")]` to provide richer description of FORGE
2. Add `#[command(long_about = "...")]` with pipeline overview for `forge --help`
3. Review and enhance `help` attributes on all args for clarity
4. Verify `--verbose` description says "Enable verbose output showing pipeline stage information"
5. Verify `--quiet` description says "Suppress all non-essential output (only OSCAL artifact)"
6. Verify `conflicts_with` properly produces clear error message for `--verbose --quiet`

**File**: `src/main.rs`

7. Verify verbose/quiet filter mapping (already: verbose→debug, quiet→error, default→warn)
8. Ensure tracing output goes to stderr (already: `.with_writer(std::io::stderr)`)

**File**: Error messages audit across `src/error.rs`, `src/cli/convert.rs`, `src/cli/validate.rs`

9. Review all `ForgeError` variants for consistent format: `"Error: {descriptive message}"`
10. Verify error messages include actionable guidance (e.g., "file not found: {path}")
11. Verify no internal Rust paths or module names in user-facing errors (SEC-4)

### Phase 3: README Update (PRD S-2, C-1)

**File**: `README.md`

1. Update Status table — mark all Phase 1 stages as "Done"
2. Add Usage section with verified examples:
   - Catalog conversion: `forge convert policy.md --strategy catalog --format json`
   - Component conversion: `forge convert policy.md --strategy component --format json`
   - Validation: `forge validate catalog.json`
   - Verbose/quiet: `forge -v convert ...` / `forge -q convert ...`
3. Add Quick Start section (PRD C-1) with single-command example
4. Update Project Structure section to reflect current layout
5. Run each example against built binary to verify accuracy before committing
6. Add note about sample policy file location (`tests/fixtures/sample_policy.md`)

### Phase 4: Release Packaging (PRD M-6, C-2)

1. Install `cargo-dist`: `cargo install cargo-dist`
2. Run `cargo dist init` — configure target platforms
3. Review and commit generated files:
   - `dist-workspace.toml`
   - `.github/workflows/release.yml`
4. Verify workflow triggers on tag push matching `v*`

### Phase 5: Release Gate Verification (PRD M-6, M-7)

**MS-4 Exit Criteria Checklist**:

1. [ ] `cargo fmt --check` — 0 violations
2. [ ] `cargo clippy -- -D warnings` — 0 warnings
3. [ ] `cargo test` — All tests pass (unit + integration + golden-file)
4. [ ] `cargo bench` — <30s mean for 50-page fixture
5. [ ] Golden-file accuracy >95%
6. [ ] `forge validate` works on generated artifacts
7. [ ] All M-1 through M-11 verified by integration tests
8. [ ] All AC-1 through AC-10 verified by integration tests
9. [ ] README examples run successfully
10. [ ] `forge --version` outputs `forge 0.1.0`

**Tag only after ALL criteria pass**:
```bash
git tag v0.1.0
git push --tags
```

## Traceability Matrix

| PRD Req | Phase | Implementation | Test |
|---------|-------|---------------|------|
| M-1 | 1 | Existing pipeline | `test_m1_structural_hierarchy_extraction` |
| M-2 | 1 | Existing pipeline | `test_m2_atomize_compound_statements` |
| M-3 | 1 | Existing pipeline | `test_m3_valid_oscal_catalog_json` |
| M-4 | 1 | Existing pipeline | `test_m4_valid_component_definition` |
| M-5 | 1, 2 | CLI help polish | `test_help_text_lists_all_subcommands`, `test_m5_metadata_fields_present` |
| M-6 | 1, 5 | Integration tests + gate | `test_m6_validate_generated_catalog`, `test_m6_validate_generated_component` |
| M-7 | 1 | Existing pipeline | `test_m7_json_output_format` |
| M-8 | 1 | Existing pipeline | `test_m8_deterministic_uuids` |
| M-9 | 1 | Existing pipeline | `test_m9_citations_in_back_matter` |
| M-10 | 1 | Existing pipeline | `test_m10_traceability_props` |
| M-11 | 1 | Existing pipeline | `test_m11_no_arbitrary_remarks` |
| S-1 | 1, 2 | CLI verbose/quiet | `test_verbose_flag_shows_pipeline_stages`, `test_quiet_flag_suppresses_output` |
| S-2 | 3 | README | Manual verification |
| S-3 | 1, 2 | Error message audit | `test_error_message_missing_file`, `test_error_message_invalid_json_for_validate` |
| C-1 | 3 | README Quick Start | Manual verification |
| C-2 | 4 | Release packaging (manual workflow) | Release workflow |
| SEC-4 | 1 | Error message audit | `test_error_message_*` (verify no Rust paths) |
| SEC-7 | 1 | Existing stderr routing | `test_verbose_flag_shows_pipeline_stages` |

## Security Considerations

From SEC-020 review:
- **SEC-1**: Actual values truncated to 100 chars — already implemented and tested
- **SEC-2**: Raw jsonschema errors not exposed — already implemented and tested
- **SEC-4**: No internal Rust paths in errors — verify in integration tests
- **SEC-7**: Errors to stderr, not stdout — verify in integration tests

No new security surface introduced by WI-25.

## Anti-patterns to Avoid

1. **DO NOT** tag v0.1.0 before all MS-4 criteria verified
2. **DO NOT** add new features or refactor during this sprint
3. **DO NOT** write README examples without running them
4. **DO NOT** skip integration tests because unit tests pass
5. **DO NOT** make CLI polish changes without re-running full test suite
6. **DO NOT** add `assert_cmd` or `predicates` — use existing `std::process::Command` pattern

## Complexity Tracking

No constitution violations. No complexity justification required.

## Dependencies

- **Requires**: All WI-1 through WI-24 complete (verified: 660 tests passing)
- **Parallel With**: None
- **Blocks**: WI-26 (XML Output, Phase 2)
- **New Build Tool**: cargo-dist (build/release only)
