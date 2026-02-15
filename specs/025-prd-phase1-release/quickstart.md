# Quickstart: WI-25 Phase 1 Release

**Branch**: `025-prd-phase1-release` | **Date**: 2026-02-14

## Prerequisites

- Rust 1.93.0+ (edition 2024) installed
- All WI-1 through WI-24 merged to branch
- `cargo test` baseline passing (660+ tests)

## Implementation Sequence

### Phase 1: Integration Tests (Days 1-2)

```bash
# 1. Verify baseline
cargo test
cargo fmt --check
cargo clippy -- -D warnings

# 2. Write new integration tests in tests/ directory
# Files to create/modify:
#   tests/e2e_release_test.rs   — MS-4 verification tests
#   tests/cli_integration.rs    — Help text and verbosity tests (extend existing)
```

**New integration tests to write:**
1. `test_e2e_catalog_convert_and_validate` — Full pipeline: convert MD → Catalog JSON → validate
2. `test_e2e_component_convert_and_validate` — Full pipeline: convert MD → Component Def → validate
3. `test_help_text_completeness` — All subcommands and flags in `--help`
4. `test_verbose_output_shows_pipeline_stages` — `--verbose` prints INFO messages
5. `test_quiet_output_suppresses_info` — `--quiet` suppresses non-essential output
6. `test_error_message_consistency` — Error format for all failure modes
7. `test_deterministic_output` — Same input produces identical output
8. `test_metadata_fields_present` — uuid, title, last-modified, version, oscal-version in output
9. `test_citations_in_back_matter` — Citations → back matter, not prose
10. `test_traceability_props_present` — Trace metadata in generated artifacts

### Phase 2: CLI Polish (Day 2-3)

```bash
# Review and enhance help text
# Edit: src/cli/mod.rs — clap derive attributes
#   - Enhance #[command(about = "...")] with richer description
#   - Add long_about for forge --help
#   - Ensure all args have clear help text
```

**Polish checklist:**
- [ ] `forge --help` — comprehensive top-level description
- [ ] `forge convert --help` — all options described with examples
- [ ] `forge validate --help` — artifact path and options described
- [ ] `--verbose` description mentions "pipeline stage information"
- [ ] `--quiet` description mentions "only OSCAL artifact output"
- [ ] Error messages reviewed for consistency and actionability

### Phase 3: README Update (Day 3)

```bash
# Update README.md:
# 1. Mark all Phase 1 status items as "Done"
# 2. Add Usage section with 3 verified examples
# 3. Add Quick Start section
# 4. Verify each example runs successfully:
cargo run -- convert tests/fixtures/sample_policy.md --strategy catalog --format json
cargo run -- convert tests/fixtures/sample_policy.md --strategy component --format json
# Save output and validate:
cargo run -- convert tests/fixtures/sample_policy.md --strategy catalog --format json --output /tmp/test-catalog.json
cargo run -- validate /tmp/test-catalog.json
```

### Phase 4: Release Setup (Day 4)

```bash
# Install and configure cargo-dist
cargo install cargo-dist
cargo dist init
# Select targets: linux-x64, macos-x64, macos-arm64, windows-x64
# Commit generated files: dist-workspace.toml, .github/workflows/release.yml
```

### Phase 5: Release Gate (Day 5)

```bash
# MS-4 Exit Criteria Verification
cargo fmt --check              # 0 violations
cargo clippy -- -D warnings    # 0 warnings
cargo test                     # All tests pass
cargo bench                    # <30s for 50-page fixture

# Verify golden-file accuracy >95%
cargo test golden_file         # Check accuracy in output

# Verify forge validate works
cargo run -- validate tests/fixtures/golden/*/expected-catalog.json

# All pass? Tag the release
git tag v0.1.0
git push --tags
```

## Key Constraints

- **NO new features** — integration, polish, and release only
- **NO new dependencies** unless required for bug fixes
- Tag v0.1.0 **ONLY** after all MS-4 criteria verified
- README examples must be **tested against the actual binary** before committing
- Error messages must not expose internal Rust paths or module names (SEC-4)
