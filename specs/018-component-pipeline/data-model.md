# Data Model: End-to-End Component Definition Pipeline

**Feature**: 018-component-pipeline
**Date**: 2026-02-13

## Summary

No new types are introduced. WI-18 is a pipeline wiring task that composes existing types. All entities below are already defined and tested.

## Entities (Existing — No Changes)

### PolicyDocument (input)
- **Location**: `src/model/mod.rs`
- **Fields**: `id: String`, `metadata: DocumentMetadata`, `sections: Vec<PolicySection>`
- **Produced by**: `prepare_document()` (shared pipeline)
- **Consumed by**: `build_component_definition()`

### ComponentDefinitionEnvelope (output)
- **Location**: `src/oscal/component_definition.rs`
- **Fields**: `component_definition: ComponentDefinition`
- **Serializes to**: `{"component-definition": {...}}`

### ComponentDefinition
- **Location**: `src/oscal/component_definition.rs`
- **Fields**: `uuid: String`, `metadata: ComponentDefinitionMetadata`, `components: Vec<DocumentaryComponent>`, `back_matter: Option<BackMatter>`

### DocumentaryComponent
- **Location**: `src/oscal/component_definition.rs`
- **Fields**: `uuid: String`, `component_type: String` ("policy"), `title: String`, `description: String`, `props: Vec<OscalProp>`, `control_implementations: Vec<serde_json::Value>`

### ForgeError (unchanged)
- **Location**: `src/error.rs`
- **Relevant variants**: `Validation(String)`, `ComponentDefinitionBuild(String)`, `Serialization(String)`, `Io`

## Interface Changes

### `run_component_pipeline` signature change

```rust
// BEFORE:
pub fn run_component_pipeline(
    input_path: &Path,
    output_path: Option<&Path>,
    max_size_bytes: u64,
    source_profile: &str,          // required &str
) -> Result<(), ForgeError>

// AFTER:
pub fn run_component_pipeline(
    input_path: &Path,
    output_path: Option<&Path>,
    max_size_bytes: u64,
    source_profile: Option<&str>,  // optional
) -> Result<(), ForgeError>
```

### `OutputFormat` clap attribute change

```rust
// BEFORE:
#[arg(long)]
format: OutputFormat,

// AFTER:
#[arg(long, default_value = "json")]
format: OutputFormat,
```

## Relationships

```
PolicyDocument
  └── build_component_definition(doc, source_profile, trace_links, source_file)
        ├── assemble_metadata(doc.metadata) → ComponentDefinitionMetadata
        ├── build_control_implementations(doc, profile, source_file) → [ControlImplementation]
        │     └── map_requirement_to_implemented(req, control_id, ...) → implemented-requirement JSON
        │           ├── build_trace_props(source_file, section, line) → [OscalProp]
        │           └── build_trace_link(source_file, line) → OscalLink
        ├── collect_all_citations(sections) → [Citation]
        │     └── generate_back_matter(citations) → BackMatter
        └── ComponentDefinitionEnvelope
```

## Validation Rules

| Rule | Location | Spec Req |
|------|----------|----------|
| `--source-profile` path must exist (if provided) | `cli/convert.rs` | SEC-3 |
| `--source-profile` must be a regular file (if provided) | `cli/convert.rs` | SEC-3 |
| `--source-profile` must not be empty string | `cli/convert.rs` | Existing |
| Output parent directory must exist | `pipeline.rs:write_output` | SEC-5 |
| Input file must be valid Markdown | `ingest/mod.rs` | Existing |
