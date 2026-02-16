# Quickstart: Export Subcommand (WI-29)

**Date**: 2026-02-15

## Prerequisites

- Rust 1.93.0+ (stable)
- FORGE codebase on branch `029-export-subcommand`
- WI-26 (XML) and WI-27 (YAML) merged to main (already done)

## Build

```bash
# Verify branch
git branch --show-current  # Should show: 029-export-subcommand

# Build
cargo build

# Verify the export subcommand appears in help
cargo run -- --help
cargo run -- export --help
```

## Usage

### Basic Format Conversion

```bash
# JSON to XML (most common use case)
forge export catalog.json --format xml

# JSON to YAML
forge export catalog.json --format yaml

# XML to JSON
forge export catalog.xml --format json

# YAML to XML
forge export catalog.yaml --format xml
```

### Output to File

```bash
# Write to file instead of stdout
forge export catalog.json --format xml --output catalog.xml

# Convert component definition
forge export component.json --format yaml --output component.yaml
```

### Same-Format Normalization

```bash
# Re-serialize and validate (normalization pass)
forge export catalog.json --format json --output normalized.json
```

### Verbose Mode

```bash
# Show detected format, target format, and pipeline stages
forge -v export catalog.json --format xml
```

## Testing

```bash
# Run all tests
cargo test

# Run only export-related tests
cargo test export

# Run with verbose output
cargo test export -- --nocapture
```

## Key Files

| File | Purpose |
|------|---------|
| `src/cli/mod.rs` | CLI definition (Commands enum with Export variant) |
| `src/cli/export.rs` | Export subcommand handler |
| `src/export/mod.rs` | Export module (serialization + deserialization) |
| `src/export/xml_deserializer.rs` | XML deserialization via quick-xml serde |
| `src/export/xml_serializer.rs` | XML serialization (existing, WI-26) |
| `src/export/yaml.rs` | YAML serialization/deserialization (existing, WI-27) |
| `src/error.rs` | ForgeError enum (new export variants) |
| `src/validate/mod.rs` | Schema validation (reused) |
| `tests/fixtures/export/` | Test fixture files (JSON, XML, YAML) |

## Architecture

```
forge export <input> --format <target> [--output <path>]
    │
    ├─ 1. Read input file
    ├─ 2. Detect input format (file extension)
    ├─ 3. Deserialize to internal OSCAL model
    ├─ 4. Validate model (OSCAL JSON schema)
    ├─ 5. Serialize to target format
    └─ 6. Write to stdout or file
```

The export pipeline is a thin orchestration layer that reuses:
- `serde_json` for JSON deserialization/serialization
- `quick-xml` for XML deserialization (serde feature) and serialization (Writer)
- `serde_yaml` for YAML deserialization/serialization
- `validate_artifact()` for OSCAL schema validation
- `write_output()` for stdout/file output
