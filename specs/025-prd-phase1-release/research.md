# Research: WI-25 Phase 1 Release

**Branch**: `025-prd-phase1-release` | **Date**: 2026-02-14

## R1: Integration Test Architecture — `assert_cmd` vs `std::process::Command`

### Decision
**Keep existing `std::process::Command` pattern** from `tests/cli_integration.rs`.

### Rationale
- The existing test suite already uses `std::process::Command` with `env!("CARGO_BIN_EXE_forge")` across 33KB of integration tests in `cli_integration.rs`
- Adding `assert_cmd` + `predicates` would create two competing patterns and require refactoring existing tests or maintaining inconsistency
- The existing pattern is working well (660 tests passing) and provides equivalent capability
- Constitution Principle X (Simplicity/YAGNI): adding a new dependency for the same capability is unjustified

### Alternatives Considered
- **`assert_cmd` + `predicates`**: Cleaner assertion syntax but introduces new dev-dependencies and a competing pattern. AR recommended this but existing tests already use `std::process::Command` effectively.

---

## R2: `cargo-dist` Configuration for v0.1.0 Release

### Decision
**Use `cargo-dist`** (latest stable) to generate GitHub Actions workflows for cross-platform binary builds.

### Rationale
- `cargo-dist` automates the generation of complex cross-platform CI workflows
- Produces binaries for Linux x86_64, macOS x86_64 + ARM64, and Windows x86_64
- One-time setup via `cargo dist init` generates `dist-workspace.toml` and `.github/workflows/release.yml`
- Aligns with AR Option 1 (selected)
- Standard in the Rust ecosystem for CLI tool releases

### Configuration Steps
1. Install: `cargo install cargo-dist`
2. Initialize: `cargo dist init` — select targets (linux-x64, macos-x64, macos-arm64, windows-x64)
3. Generated files: `dist-workspace.toml`, `.github/workflows/release.yml`
4. Commit generated workflow files
5. Release is triggered by pushing a git tag matching `v*`

### Alternatives Considered
- **Manual GitHub Actions**: Full control but complex cross-compilation maintenance
- **Source-only release**: Simplest but requires Rust toolchain from users

---

## R3: Parent PRD M-requirement Test Coverage Mapping

### Decision
Map each M-requirement to existing and new integration tests for traceability.

### Current Coverage Analysis

| Parent M-Req | Description | Existing Test Coverage | Gap |
|-------------|-------------|----------------------|-----|
| M-1 | Markdown input → structural hierarchy | `cli_integration.rs`, `catalog_pipeline_test.rs` | Need explicit E2E traceability test |
| M-2 | Atomize compound statements | `atomize_integration.rs` | Covered |
| M-3 | Valid OSCAL Catalog generation | `catalog_pipeline_test.rs`, `golden_file_tests.rs` | Need schema validation assertion |
| M-4 | Valid Component Definition generation | `component_pipeline_test.rs` | Need schema validation assertion |
| M-5 | OSCAL metadata fields present | `catalog_pipeline_test.rs` | Need explicit metadata field checks |
| M-6 | `forge validate` works | `validate_test.rs` | Covered |
| M-7 | JSON output format | `cli_integration.rs` | Covered |
| M-8 | Stable UUIDs across re-conversions | `fixture_determinism_test.rs` | Covered |
| M-9 | Citations → back matter | `catalog_pipeline_test.rs` | Need explicit assertion |
| M-10 | Traceability links | `trace_integration.rs` | Covered |
| M-11 | No arbitrary data in remarks | `golden_file_tests.rs` | Need explicit assertion |

### New Integration Tests Required
1. **E2E milestone-4 (MS-4) verification test**: Explicit test that runs full pipeline + validate and asserts pass (MS-4 = Milestone 4 exit criteria, distinct from M-4 requirement)
2. **Metadata completeness test**: Assert all 5 required metadata fields present
3. **Cross-pipeline schema validation**: Generate both Catalog and Component Def, then validate each
4. **Help text completeness test**: Assert all subcommands and flags appear in --help output
5. **Error message consistency test**: Test error format for missing file, invalid input, schema violations

---

## R4: README Update Strategy

### Decision
Replace current "Planned" status markers with "Done" and add a comprehensive Usage section with verified examples.

### Rationale
- Current README shows "Planned" for components that are now complete (Catalog pipeline, Component Definition, Traceability, Validate, Export)
- Usage examples must be verified by running them against the built binary before committing (PRD S-2, R-4)
- README is the "front door" for community adoption (Vision goal G-3)

### README Structure
1. Update Status table — mark all Phase 1 stages as "Done"
2. Add Usage section with 3 tested examples:
   - `forge convert policy.md --strategy catalog --format json`
   - `forge convert policy.md --strategy component --format json`
   - `forge validate catalog.json`
3. Add Quick Start section (PRD C-1)
4. Update Project Structure to reflect current layout
5. Keep Installation section current

---

## R5: CLI Help Text Review

### Decision
Review and enhance all `--help` text using clap derive `about`, `help`, and `long_help` attributes.

### Current State Assessment
- `forge --help`: Shows `convert` and `validate` subcommands, `--verbose`, `--quiet`, `--version` ✓
- `forge convert --help`: Shows `<INPUT>`, `--strategy`, `--format`, `--output`, `--max-size`, `--source-profile` ✓
- `forge validate --help`: Shows `<INPUT>`, `--schema-type`, `--format` ✓
- Help text descriptions are present but may need enhancement for clarity and completeness

### Polish Items
1. Enhance top-level `about` text with more descriptive information
2. Add `long_about` for `forge --help` with pipeline overview
3. Ensure `--verbose` and `--quiet` descriptions mention what they control (pipeline stages vs suppression)
4. Add examples to `long_help` where appropriate (clap supports `help_heading`)

---

## R6: Security Requirements from SEC-020

### Decision
Incorporate SEC-1 through SEC-8 from the security review as verification checkpoints in the integration test suite.

### Relevant Security Controls
| SEC-Req | Description | Verification |
|---------|-------------|-------------|
| SEC-1 | Actual values truncated to 100 chars | Already unit-tested in formatter |
| SEC-2 | Raw jsonschema errors never exposed | Already unit-tested |
| SEC-3 | JSON report contains only defined fields | Already unit-tested |
| SEC-4 | No internal Rust paths in error messages | Integration test needed |
| SEC-5 | No external URL following | Already unit-tested |
| SEC-6 | Malformed JSON pointers handled gracefully | Already unit-tested |
| SEC-7 | Auto-validate errors to stderr only | Integration test needed |
| SEC-8 | Error count accuracy | Already unit-tested |

### Integration Test Additions for SEC
- SEC-4: Run `forge validate` on invalid artifact, verify no Rust module paths in output
- SEC-7: Run `forge convert` with `--verbose`, verify errors go to stderr not stdout
