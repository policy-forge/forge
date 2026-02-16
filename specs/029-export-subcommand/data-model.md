# Data Model: Export Subcommand (WI-29)

**Date**: 2026-02-15
**Status**: Complete

## Overview

The export subcommand introduces NO new persistent data model entities. It operates entirely on existing OSCAL model types (`CatalogEnvelope`, `ComponentDefinitionEnvelope`) and adds only a wrapper enum for the export pipeline.

## Existing Entities (Reused)

### CatalogEnvelope

- **Location**: `src/oscal/catalog.rs`
- **Purpose**: Top-level JSON envelope `{"catalog": {...}}`
- **Fields**: `catalog: OscalCatalog`
- **Serde**: Derives `Serialize`, `Deserialize`
- **Used for**: Deserialization from JSON/YAML/XML, serialization to all formats

### ComponentDefinitionEnvelope

- **Location**: `src/oscal/component_definition.rs`
- **Purpose**: Top-level JSON envelope `{"component-definition": {...}}`
- **Fields**: `component_definition: ComponentDefinition`
- **Serde**: Derives `Serialize`, `Deserialize`; rename `component-definition`
- **Used for**: Deserialization from JSON/YAML/XML, serialization to all formats

### OutputFormat

- **Location**: `src/cli/mod.rs`
- **Purpose**: Enum representing output format (Json, Xml, Yaml)
- **Derives**: `ValueEnum`, `Clone`, `Debug`
- **Used for**: CLI `--format` argument, format routing in pipeline

## New Entities

### OscalModel (Export Pipeline Wrapper)

- **Location**: `src/cli/export.rs` (or `src/export/mod.rs`)
- **Purpose**: Wrapper enum to hold a deserialized OSCAL model during the export pipeline without knowing the model type at compile time
- **Lifetime**: Transient — exists only during a single export invocation; not persisted
- **Scope limitation**: Currently supports only Catalog and ComponentDefinition model types. Additional OSCAL types (Profile, SSP, SAP, SAR, POAM) can be added by introducing new variants and extending `deserialize_oscal()` / `serialize_oscal()` match arms.
- **Variants**:
  - `Catalog(CatalogEnvelope)` — holds a deserialized OSCAL Catalog
  - `Component(ComponentDefinitionEnvelope)` — holds a deserialized Component Definition

```rust
pub enum OscalModel {
    Catalog(CatalogEnvelope),
    Component(ComponentDefinitionEnvelope),
}
```

**Validation Rules**: The inner envelope must pass OSCAL schema validation (checked in `validate_oscal_model()`).

**State Transitions**: None. This is a value type with no state machine.

### ExportArgs (CLI Arguments)

- **Location**: `src/cli/mod.rs` (as part of `Commands::Export` variant)
- **Purpose**: CLI argument struct for the export subcommand
- **Fields**:
  - `input: PathBuf` — positional, path to input OSCAL artifact
  - `format: OutputFormat` — required `--format` flag
  - `output: Option<PathBuf>` — optional `--output` flag
- **Validation**: `input` must exist and be readable; `format` is constrained by clap `ValueEnum`; if `output` is `Some(path)`, the write is attempted via `std::fs::write()` and filesystem errors (e.g., parent directory missing, permission denied) are propagated as `ForgeError::Io`

### ForgeError Variants (Additions)

- **Location**: `src/error.rs`
- **New Variants**:

| Variant | Fields | Error Message | Exit Code |
|---------|--------|---------------|-----------|
| `ExportUnsupportedExtension` | `extension: String` | "Unrecognized file extension '.{ext}'..." | 1 |
| `ExportNoExtension` | `path: PathBuf` | "No file extension on input file..." | 1 |
| `ExportInvalidOscal` | `detail: String` | "Input is not a valid OSCAL artifact: {detail}" | 1 |
| `ExportEmptyInput` | `path: PathBuf` | "Export input file is empty: '{path}'" | 1 |
| `Serialization` | `String` | "JSON/XML/YAML serialization failed: {detail}" | 1 |
| `SchemaValidation` | `String` | "N validation error(s): {details}" | 1 |

**Note**: `Serialization` and `SchemaValidation` are existing variants reused for deserialization/serialization failures and validation errors respectively.

## Entity Relationships

```
ExportArgs (CLI input)
    │
    ├── input: PathBuf ──→ File System (read)
    ├── format: OutputFormat ──→ Serializer selection
    └── output: Option<PathBuf> ──→ File System (write) or stdout
         │
         ▼
OscalModel (transient wrapper)
    ├── Catalog(CatalogEnvelope) ──→ OscalCatalog → OscalGroup → OscalControl
    └── Component(ComponentDefinitionEnvelope) ──→ ComponentDefinition → DocumentaryComponent
         │
         ▼
validate_oscal_model(&model) ──→ validate_artifact(json_value, model_type) ──→ ValidationResult
         │
         ▼
Serializer (OutputFormat routing)
    ├── Json: serde_json::to_string_pretty()
    ├── Xml: serialize_catalog_to_xml() / serialize_component_definition_to_xml()
    └── Yaml: serialize_to_yaml()
```

## Data Flow Summary

```
Input File (JSON/XML/YAML)
    → Read to String
    → Detect Format (extension-based)
    → Deserialize to OscalModel variant
    → Validate (serialize to JSON Value → OSCAL schema check)
    → Serialize to target format
    → Write to stdout or file
```

No data is persisted, cached, or transmitted over a network. All operations are in-memory transformations of OSCAL artifacts.
