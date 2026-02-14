# Quickstart: End-to-End Component Definition Pipeline

**Feature**: 018-component-pipeline
**Date**: 2026-02-13

## Usage

### Full pipeline with source profile (primary use case)

```bash
forge convert policy.md --strategy component --source-profile baseline.json --format json
```

Produces OSCAL Component Definition JSON to stdout with:
- Documentary component (type: "policy")
- Control-implementations with implemented-requirements mapped to control IDs
- Traceability props (source-file, source-section, source-line) on each implemented-requirement
- Back matter resources for citations

### Without source profile (unmapped requirements)

```bash
forge convert policy.md --strategy component --format json
```

Emits a warning to stderr:
```
WARN: --source-profile not provided; control-id mapping will be skipped. The generated Component Definition will have empty control-implementations.
```

Produces a valid Component Definition with empty `control-implementations` array.

### Default format (JSON inferred)

```bash
forge convert policy.md --strategy component --source-profile baseline.json
```

`--format` defaults to `json` when omitted.

### Output to file

```bash
forge convert policy.md --strategy component --source-profile baseline.json --output component-def.json
```

### Error cases

```bash
# Non-existent source profile → descriptive error, exit 1
forge convert policy.md --strategy component --source-profile nonexistent.json

# Non-existent output directory → descriptive error, exit 1
forge convert policy.md --strategy component --output /bad/dir/out.json

# Empty input file → descriptive error, exit 1
forge convert empty.md --strategy component
```

## Development

### Run tests

```bash
# All tests
cargo test

# Component pipeline integration tests only
cargo test --test component_pipeline_test

# CLI integration tests only
cargo test --test cli_integration

# Unit tests only
cargo test --lib
```

### Build

```bash
cargo build
cargo run -- convert tests/fixtures/full_policy.md --strategy component --source-profile ./baselines/nist.json --format json
```

### Lint

```bash
cargo fmt --check
cargo clippy -- -D warnings
```
