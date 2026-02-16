# Research: Export Subcommand (WI-29)

**Date**: 2026-02-15
**Status**: Complete

## Research Questions

### RQ-1: XML Deserialization Strategy

**Question**: How should OSCAL XML artifacts be deserialized into the internal OSCAL model? The codebase currently has XML serialization (manual `quick_xml::Writer`) but no XML deserialization.

**Decision**: Use `quick-xml`'s `serde` feature for XML deserialization via `quick_xml::de::from_str()`.

**Rationale**:
- quick-xml 0.37 supports serde deserialization when the `serde` feature is enabled
- All OSCAL model structs already derive `serde::Deserialize` with appropriate `#[serde(rename = "...")]` annotations (e.g., `last-modified`, `back-matter`, `component-definition`)
- The existing XML serializer creates elements with names matching these serde rename annotations
- Avoids implementing a full manual XML deserialization Reader (hundreds of lines, error-prone)
- Constitution principle X (YAGNI) favors the simplest approach

**Alternatives Considered**:
1. **Manual quick-xml Reader deserialization** — Mirroring the Writer-based serializer with a Reader-based deserializer. Rejected: too much code for a thin orchestration layer; high maintenance burden.
2. **Intermediate JSON conversion** — Convert XML to JSON via an external tool, then use serde_json. Rejected: adds complexity; may lose fidelity; requires an XML-to-JSON conversion strategy.

**Implementation Notes**:
- Enable `serde` feature on quick-xml in `Cargo.toml`: `quick-xml = { version = "0.37", features = ["serde"] }`
- Implement `deserialize_catalog_from_xml()` and `deserialize_component_from_xml()` wrapper functions
- XML namespace (`xmlns="http://csrc.nist.gov/ns/oscal/1.0"`) on root element: quick-xml's serde mode treats `xmlns` as a regular attribute; it does not interfere with element name matching
- The `@` prefix convention for XML attributes in quick-xml serde: OSCAL model structs don't use `@`-prefixed fields, so all fields are deserialized as child elements (correct for OSCAL JSON-derived structure)
- The `$text` / `$value` convention for text content in quick-xml serde: prose fields in `OscalPart` may need attention if XML text content doesn't map cleanly
- `Vec<serde_json::Value>` in `DocumentaryComponent.control_implementations`: this JSON-specific type may not deserialize cleanly from XML. Mitigation: for the export use case, control implementations from XML input should be empty or represented as a generic structure. Test this edge case explicitly.

**Risks**:
- quick-xml serde may not handle all OSCAL XML structural patterns (e.g., mixed content, attributes vs elements). Mitigation: comprehensive test fixtures for each model type.
- If serde deserialization fails for specific patterns, fall back to a custom deserialization function for those specific fields.

---

### RQ-2: Validation Strategy for Non-JSON Formats

**Question**: The existing validation infrastructure (`validate_artifact()`) operates on `serde_json::Value`. How should output validation work for XML and YAML target formats?

**Decision**: Validate via JSON intermediate representation regardless of target format.

**Rationale**:
- OSCAL schema validation uses JSON Schema (OSCAL v1.2.0 JSON schemas are embedded)
- The `validate_artifact()` function takes a `serde_json::Value` — this is the only validation path
- After serializing to the target format, the content is already correct (the internal model was validated). But to catch serialization bugs, we validate the internal model as JSON before serializing to the final format.
- This matches the existing pipeline pattern in `pipeline.rs`: `validate_catalog_json()` serializes to JSON, validates the JSON, then serializes to the target format.

**Implementation**:
1. Deserialize input to internal model
2. Serialize model to JSON (`serde_json::to_value()`)
3. Validate JSON value against OSCAL schema
4. If valid, serialize model to target format
5. Write output

**Alternatives Considered**:
1. **Validate after final serialization** (re-parse target format to JSON, then validate). Rejected: unnecessary round-trip; validates the same model twice.
2. **No validation for non-JSON formats** (only validate JSON output). Rejected: PRD M-4 requires validation for all target formats.
3. **XML Schema (XSD) validation for XML output**. Rejected: no XSD validation infrastructure exists; YAGNI.

---

### RQ-3: OSCAL Model Type Detection During Deserialization

**Question**: When deserializing an unknown OSCAL artifact, how do we determine whether it's a Catalog or Component Definition?

**Decision**: Try-deserialize pattern — attempt each model type in order, use the first that succeeds.

**Rationale**:
- For JSON: use the existing `detect_model_type()` function which inspects top-level keys (`"catalog"` vs `"component-definition"`)
- For YAML: same approach — deserialize YAML to `serde_json::Value` first, then use `detect_model_type()`
- For XML: inspect the root element name before full deserialization

**Implementation**:
```
fn detect_and_deserialize(content: &str, format: OscalFormat) -> Result<OscalModel, ForgeError> {
    match format {
        Json => {
            let value: serde_json::Value = serde_json::from_str(content)?;
            let model_type = detect_model_type(&value)?;
            match model_type {
                Catalog => Ok(OscalModel::Catalog(serde_json::from_value(value)?)),
                ComponentDefinition => Ok(OscalModel::Component(serde_json::from_value(value)?)),
            }
        }
        Yaml => {
            let value: serde_json::Value = serde_yaml::from_str(content)?;
            let model_type = detect_model_type(&value)?;
            // deserialize from the value
        }
        Xml => {
            // Inspect root element name, then deserialize with appropriate type
        }
    }
}
```

---

### RQ-4: Error Handling for Export

**Question**: The existing `ForgeError::UnsupportedFormat` variant has a message specific to Markdown files ("Only Markdown files (.md, .markdown) are supported"). How should export-specific errors be handled?

**Decision**: Add new `ForgeError` variants specific to the export context.

**New Variants**:
- `ForgeError::ExportUnsupportedExtension { extension: String }` — unrecognized file extension for OSCAL format detection (message: "Unrecognized file extension '.{ext}'. Expected .json, .xml, .yaml, or .yml")
- `ForgeError::ExportNoExtension` — no file extension found on input (message: "No file extension on input file. Cannot determine OSCAL format. Expected .json, .xml, .yaml, or .yml")
- `ForgeError::ExportInvalidOscal { detail: String }` — input file is not a valid OSCAL artifact (message: "Input is not a valid OSCAL artifact: {detail}")
- `ForgeError::ExportEmptyInput { path: PathBuf }` — input file is empty (message: "Input file is empty: '{path}'")

**Rationale**:
- Reusing `ForgeError::UnsupportedFormat` would produce confusing messages ("Only Markdown files supported" when the user provided an OSCAL artifact)
- Each error variant maps to exit code 1 (input/IO errors) for consistency with existing error code mapping
- Constitution principle VIII requires actionable, contextual error messages

**Alternatives Considered**:
1. **Reuse existing variants with generic messages** — `ForgeError::Validation("not a valid OSCAL artifact")`. Rejected: loses specificity and exit code granularity.
2. **Modify existing `UnsupportedFormat`** to be more generic. Rejected: would break existing error message tests for `forge convert`.

---

### RQ-5: Reusing `OutputFormat` vs Creating `OscalFormat`

**Question**: The existing `OutputFormat` enum (json, xml, yaml) is defined in `cli/mod.rs`. The AR specifies an `OscalFormat` enum. Should we reuse `OutputFormat` or create a new enum?

**Decision**: Reuse the existing `OutputFormat` enum from `cli/mod.rs` for the `--format` argument. Add a separate `detect_format()` function that returns `OutputFormat`.

**Rationale**:
- `OutputFormat` already has exactly the right variants (Json, Xml, Yaml) with `ValueEnum` derive for clap
- Creating a duplicate `OscalFormat` enum adds unnecessary code for identical functionality
- The AR's `OscalFormat` was a conceptual name — the existing `OutputFormat` serves the same purpose
- Constitution principle X: don't create abstractions that duplicate what exists

**Implementation**:
- `ExportArgs.format` uses `OutputFormat` (same enum as `Convert` subcommand)
- `detect_format()` returns `OutputFormat`
- Both are consistent with the existing CLI vocabulary

---

### RQ-6: XXE Prevention in XML Deserialization (SEC-1)

**Question**: Does quick-xml's serde deserialization process DTD declarations or expand external entities?

**Decision**: quick-xml does NOT process DTDs or expand external entities by default. Confirmed safe.

**Evidence**:
- quick-xml's default parser treats DTD declarations and entities as opaque events
- `quick_xml::de::from_str()` does not expand `<!ENTITY>` declarations
- The parser emits DTD-related events but the serde deserializer ignores them (they don't map to struct fields)

**Verification**: Write a unit test with an XXE payload in input XML (`<!DOCTYPE foo [<!ENTITY xxe SYSTEM "file:///etc/passwd">]>`) and confirm the entity is not expanded and no file system read occurs. This test is specified in SEC-1.

---

### RQ-7: The `Vec<serde_json::Value>` Problem in ComponentDefinition

**Question**: `DocumentaryComponent.control_implementations` is typed as `Vec<serde_json::Value>`. This JSON-specific type won't deserialize cleanly from XML or YAML.

**Decision**: Deserialize each format via its natural path, using `serde_json::Value` as the intermediate only where needed:

1. **JSON input**: `serde_json::from_str()` to `serde_json::Value` → `detect_model_type()` → `serde_json::from_value::<CatalogEnvelope>()` or `ComponentDefinitionEnvelope`
2. **YAML input**: `serde_yaml::from_str::<serde_json::Value>()` → `detect_model_type()` → `serde_json::from_value()` to typed envelope (YAML→Value works cleanly since `serde_json::Value` implements `Deserialize`)
3. **XML input**: Dedicated XML deserialization structs (`XmlCatalog`, `XmlComponentDefinition`) via `quick_xml::de::from_str()`, then manual conversion to the shared model types. XML root element name determines model type.

**Rationale**:
- The internal model types (CatalogEnvelope, ComponentDefinitionEnvelope) are designed for JSON-first serde
- `serde_json::Value` serves as the common intermediate for JSON and YAML model type detection
- XML requires dedicated deserialization structs due to attribute/element naming differences (e.g., `@uuid`, `@id` attributes vs JSON fields); `quick_xml::de::from_str::<serde_json::Value>()` does not reliably produce the same structure as JSON
- This approach was validated during implementation and all 18 format-pair tests pass

**Test plan**: Create fixtures for each model type in each format and verify round-trip through the export pipeline.
