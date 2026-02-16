# Contract: XML Serializer Module

**Feature**: 026-xml-output
**Module**: `src/export/xml_serializer.rs`
**Date**: 2026-02-15

## Module Purpose

Provides OSCAL XML serialization functions that convert existing OSCAL model structs into valid OSCAL v1.2.0 XML strings. All XML is produced via `quick-xml::Writer` — no string concatenation.

## Constants

```rust
/// OSCAL XML namespace URI — applied to root element only.
pub const OSCAL_NS: &str = "http://csrc.nist.gov/ns/oscal/1.0";

/// XML indentation: 2 spaces per level.
const INDENT_SIZE: usize = 2;
```

## Public Functions

### `serialize_catalog_to_xml`

```rust
/// Serialize an OSCAL Catalog to a valid OSCAL v1.2.0 XML string.
///
/// Produces a complete XML document with:
/// - XML declaration (`<?xml version="1.0" encoding="UTF-8"?>`)
/// - Root `<catalog>` element with OSCAL namespace and uuid attribute
/// - Child elements in XSD-prescribed order: metadata, group*, back-matter?
///
/// # Arguments
/// * `catalog` — Reference to the `OscalCatalog` to serialize
///
/// # Returns
/// * `Ok(String)` — Valid OSCAL XML string
/// * `Err(ForgeError::Serialization)` — If XML writing or UTF-8 conversion fails
///
/// # Security
/// * All text content is XML-escaped via quick-xml (SEC-3)
/// * No DTD or entity declarations are emitted (SEC-1)
/// * No string concatenation is used for XML construction (SEC-5)
pub fn serialize_catalog_to_xml(catalog: &OscalCatalog) -> Result<String, ForgeError>;
```

**Input**: `&OscalCatalog` (from `src/oscal/catalog.rs`)
**Output**: Complete XML document string
**Error**: `ForgeError::Serialization(String)` wrapping quick-xml or UTF-8 errors

### `serialize_component_definition_to_xml`

```rust
/// Serialize an OSCAL Component Definition to a valid OSCAL v1.2.0 XML string.
///
/// Produces a complete XML document with:
/// - XML declaration (`<?xml version="1.0" encoding="UTF-8"?>`)
/// - Root `<component-definition>` element with OSCAL namespace and uuid attribute
/// - Child elements in XSD-prescribed order: metadata, component*, back-matter?
///
/// # Arguments
/// * `component_def` — Reference to the `ComponentDefinition` to serialize
///
/// # Returns
/// * `Ok(String)` — Valid OSCAL XML string
/// * `Err(ForgeError::Serialization)` — If XML writing or UTF-8 conversion fails
pub fn serialize_component_definition_to_xml(
    component_def: &ComponentDefinition,
) -> Result<String, ForgeError>;
```

**Input**: `&ComponentDefinition` (canonical struct name from `src/oscal/component_definition.rs`; note: PRD interface contract uses `OscalComponentDefinition` but the actual Rust struct is `ComponentDefinition`)
**Output**: Complete XML document string
**Error**: `ForgeError::Serialization(String)`

## Internal Helper Functions

These are `pub(crate)` or private functions shared across serializers:

### `write_metadata`

```rust
/// Write OSCAL metadata elements in XSD order.
///
/// Writes: <metadata> → title, last-modified, version, oscal-version → </metadata>
///
/// Handles both `OscalMetadata` (from catalog.rs inline struct with String fields)
/// and `ComponentDefinitionMetadata` (from component_definition.rs).
fn write_metadata<W: Write>(
    writer: &mut Writer<W>,
    title: &str,
    last_modified: &str,
    version: &str,
    oscal_version: &str,
) -> Result<(), ForgeError>;
```

### `write_group`

```rust
/// Write a single OSCAL group element with children in XSD order.
///
/// Writes: <group id="..."> → title, prop*, link*, control* → </group>
fn write_group<W: Write>(
    writer: &mut Writer<W>,
    group: &OscalGroup,
) -> Result<(), ForgeError>;
```

### `write_control`

```rust
/// Write a single OSCAL control element with children in XSD order.
///
/// Writes: <control id="..."> → title, prop*, link*, part* → </control>
fn write_control<W: Write>(
    writer: &mut Writer<W>,
    control: &OscalControl,
) -> Result<(), ForgeError>;
```

### `write_part`

```rust
/// Write a single OSCAL part element with children in XSD order.
///
/// Writes: <part id="..." name="..."> → prop*, <p>prose</p>, part* → </part>
fn write_part<W: Write>(
    writer: &mut Writer<W>,
    part: &OscalPart,
) -> Result<(), ForgeError>;
```

### `write_prop`

```rust
/// Write a single OSCAL prop element.
///
/// Writes: <prop name="..." value="..." [ns="..."] />
/// All values are XML attributes. Empty element (self-closing).
fn write_prop<W: Write>(
    writer: &mut Writer<W>,
    prop: &OscalProp,
) -> Result<(), ForgeError>;
```

### `write_link`

```rust
/// Write a single OSCAL link element.
///
/// When `link.text` is `None`: emit self-closing `<link href="..." rel="..." />`
/// using `writer.write_event(Event::Empty(...))`.
/// When `link.text` is `Some(t)`: emit `<link href="..." rel="..."><text>t</text></link>`
/// using `Event::Start` + text child + `Event::End`.
fn write_link<W: Write>(
    writer: &mut Writer<W>,
    link: &OscalLink,
) -> Result<(), ForgeError>;
```

### `write_back_matter`

```rust
/// Write OSCAL back-matter element with resource children.
///
/// Writes: <back-matter> → resource* → </back-matter>
fn write_back_matter<W: Write>(
    writer: &mut Writer<W>,
    back_matter: &BackMatter,
) -> Result<(), ForgeError>;
```

### `write_resource`

```rust
/// Write a single OSCAL resource element with children in XSD order.
///
/// Writes: <resource uuid="..."> → title?, description?, prop*, citation?, rlink* → </resource>
fn write_resource<W: Write>(
    writer: &mut Writer<W>,
    resource: &BackMatterResource,
) -> Result<(), ForgeError>;
```

### `write_component`

```rust
/// Write a single OSCAL component element with children in XSD order.
///
/// Writes: <component uuid="..." type="..."> → title, description, prop* → </component>
fn write_component<W: Write>(
    writer: &mut Writer<W>,
    component: &DocumentaryComponent,
) -> Result<(), ForgeError>;
```

## Error Handling

All quick-xml write errors and UTF-8 conversion errors are wrapped in `ForgeError::Serialization(String)`:

```rust
// Pattern for wrapping quick-xml errors
fn map_xml_err(e: quick_xml::Error) -> ForgeError {
    ForgeError::Serialization(format!("XML write error: {e}"))
}

fn map_utf8_err(e: std::string::FromUtf8Error) -> ForgeError {
    ForgeError::Serialization(format!("XML UTF-8 conversion error: {e}"))
}
```

## Security Contracts

| SEC ID | Contract | Enforcement |
|--------|----------|-------------|
| SEC-1 | No DTD declarations or entity definitions in output | Never emit `Event::DocType`; unit test verifies absence |
| SEC-2 | User content cannot alter XML structure | All text via `BytesText` (auto-escaped); all attributes via `.push_attribute()` |
| SEC-3 | XML special characters escaped in text content | `BytesText::new()` auto-escapes `<`, `>`, `&` |
| SEC-4 | XML attributes escaped via quick-xml API | `.push_attribute()` auto-escapes attribute values |
| SEC-5 | No string concatenation for XML construction | All XML via `quick_xml::Writer` events; code review enforced |
| SEC-6 | Only OSCAL namespace on root element | Hardcoded `OSCAL_NS` constant; no user-derived namespace values |
| SEC-7 | XSD validation in integration tests | `xmllint --schema` in test suite |

## Scope Note

The AR (026-ar-xml-output.md) defines a `serialize_to_xml(artifact: &OscalArtifact)` generic dispatch function. This is **not implemented in WI-26** because the pipeline already dispatches by strategy (catalog vs component). A generic dispatch may be added in a future WI when Profile serialization (S-3, WI-30+) is implemented.

## Integration Points

### Pipeline Integration (`src/pipeline.rs`)

```rust
// In run_catalog_pipeline or equivalent:
match format {
    OutputFormat::Json => serde_json::to_string_pretty(&envelope)?,
    OutputFormat::Xml => {
        serialize_catalog_to_xml(&envelope.catalog)?
    },
    _ => return Err(ForgeError::Serialization("Unsupported format".into())),
}
```

### CLI Integration (`src/cli/convert.rs`)

Remove the XML rejection guard:
```rust
// BEFORE (current):
if !matches!(format, OutputFormat::Json) {
    return Err(anyhow!("Only JSON format is currently supported"));
}

// AFTER:
if matches!(format, OutputFormat::Yaml) {
    return Err(anyhow!("YAML format is not yet supported (see WI-27)"));
}
```
