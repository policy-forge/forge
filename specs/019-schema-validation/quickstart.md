# Quickstart: 019-schema-validation

**Date**: 2026-02-13 | **Branch**: `019-schema-validation`

## Prerequisites

- Rust stable 1.93.0+ (`rustup update stable`)
- FORGE project cloned and building (`cargo build`)
- OSCAL v1.2.0 JSON schemas downloaded to `schemas/` directory

## Setup

### 1. Download OSCAL Schemas

```bash
mkdir -p schemas

# Download from NIST OSCAL v1.2.0 release
# (URLs from https://github.com/usnistgov/OSCAL/releases/tag/v1.2.0)
curl -L -o schemas/oscal_catalog_schema.json \
  "https://github.com/usnistgov/OSCAL/releases/download/v1.2.0/oscal_catalog_schema.json"

curl -L -o schemas/oscal_component_schema.json \
  "https://github.com/usnistgov/OSCAL/releases/download/v1.2.0/oscal_component_schema.json"
```

### 2. Add Dependency

```bash
cargo add jsonschema
```

### 3. Build

```bash
cargo build
```

## Usage

### Validate a standalone OSCAL artifact

```bash
# Validate a catalog
cargo run -- validate catalog.json

# Validate a component definition
cargo run -- validate component.json

# Override model type detection
cargo run -- validate artifact.json --schema-type catalog
```

### Auto-validation in convert pipeline

```bash
# Convert generates OSCAL JSON and auto-validates before writing
cargo run -- convert policy.md --strategy catalog --format json --output catalog.json
```

## Development Workflow (TDD)

```bash
# 1. Write tests first
cargo test --lib validate  # Should FAIL (red)

# 2. Implement
# ... write code in src/validate/mod.rs ...

# 3. Run tests
cargo test --lib validate  # Should PASS (green)

# 4. Quality checks
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

## Key Files

| File | Purpose |
|------|---------|
| `schemas/oscal_catalog_schema.json` | NIST OSCAL Catalog JSON Schema (embedded at compile time) |
| `schemas/oscal_component_schema.json` | NIST OSCAL Component Definition JSON Schema (embedded at compile time) |
| `src/validate/mod.rs` | Validation module: types, detect, load, validate |
| `src/cli/validate.rs` | CLI handler for `forge validate` |
| `src/pipeline.rs` | Auto-validation gate in `forge convert` |
| `tests/validate_test.rs` | Integration tests for validation |
