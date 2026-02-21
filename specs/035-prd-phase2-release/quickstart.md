# Quickstart: Phase 2 Integration Testing & v0.2.0 Release

**Feature**: WI-35 | **Date**: 2026-02-19

---

## Prerequisites

```bash
# Rust toolchain
rustup update stable
rustc --version   # should be >= 1.93.0

# Verify forge builds
cargo build
```

---

## Running the WI-35 Integration Tests

### Run all integration tests (recommended)

```bash
cargo test
```

This runs all 1049+ existing tests plus the new WI-35 integration test modules.

### Run only WI-35 integration tests

```bash
# Round-trip end-to-end tests (M-1)
cargo test integration_round_trip

# Profile end-to-end tests (M-2, M-3, M-4)
cargo test integration_profile_e2e

# Cross-feature tests (M-5: normative props + params across formats)
cargo test integration_cross_feature

# Phase 1 regression tests (M-6)
cargo test integration_regression
```

### Run with test output for debugging

```bash
cargo test integration_cross_feature -- --nocapture 2>&1 | head -100
```

---

## Release Gate Commands

Run in this exact sequence before tagging v0.2.0:

```bash
# Gate 1: All tests pass
cargo test
# Expected: N passed, 0 failed

# Gate 2: No clippy warnings
cargo clippy -- -D warnings
# Expected: (no output)

# Gate 3: Code formatted
cargo fmt --check
# Expected: (no output / exit 0)

# Gate 4: No license or advisory deny violations (constitution Principle XI)
cargo deny check
# Expected: (no output / exit 0)
```

---

## Creating the v0.2.0 Tag

Only after all three gates above pass:

```bash
# 1. Update version
# Edit Cargo.toml: version = "0.2.0"

# 2. Update CHANGELOG.md (create if absent)

# 3. Final quality gate check (four gates)
cargo test && cargo clippy -- -D warnings && cargo fmt --check && cargo deny check

# 4. Commit and tag
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "release: bump version to 0.2.0 for Phase 2 milestone"
git tag v0.2.0

# 5. Verify
cargo run -- --version   # Must print: forge 0.2.0
git tag -l v0.2.0        # Must print: v0.2.0
```

---

## Manual CLI Verification

Test CLI interfaces introduced in Phase 2:

```bash
# Format output tests (WI-26, WI-27)
cargo run -- convert tests/fixtures/full_policy.md --strategy catalog --format xml
cargo run -- convert tests/fixtures/full_policy.md --strategy catalog --format yaml

# Export subcommand (WI-29)
cargo run -- convert tests/fixtures/full_policy.md --strategy catalog --format json --output /tmp/catalog.json
cargo run -- export /tmp/catalog.json --format xml --output /tmp/catalog.xml
cargo run -- export /tmp/catalog.xml --format yaml --output /tmp/catalog.yaml

# Profile generation (WI-30, WI-31)
cargo run -- profile --catalog /tmp/catalog.json --format json --output /tmp/profile.json
cargo run -- validate /tmp/profile.json

# Help text review (S-4)
cargo run -- profile --help
cargo run -- export --help
```

---

## Troubleshooting

### Test fails with "forge binary not found"

```bash
cargo build         # Build the binary first
cargo test          # Binary is now available via CARGO_BIN_EXE_forge
```

### Round-trip test fails with semantic mismatch

Add `-- --nocapture` to see the diff:
```bash
cargo test catalog_json_xml_json_round_trip -- --nocapture
```

Compare the deserialized JSON values to identify which field is not surviving round-trip.

### Schema validation fails on generated Profile

Check the Profile JSON against the schema manually:
```bash
cargo run -- profile --catalog /tmp/catalog.json --format json --output /tmp/profile.json
cargo run -- validate /tmp/profile.json
```

Look for the specific OSCAL schema violation in the output.

### Clippy warning blocks release

Fix the warning in the source file indicated by clippy. Common patterns:
- Unused imports: remove them
- Missing `#[allow]` for intentional patterns: document why and add `#[allow(clippy::...)]`
- Clippy auto-fixes: `cargo clippy --fix`

### fmt check fails

```bash
cargo fmt    # Auto-format all files
cargo fmt --check  # Verify now passes
```
