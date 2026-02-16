# Quickstart: OSCAL XML Output (026)

**Feature**: 026-xml-output
**Date**: 2026-02-15

## Overview

This guide shows how to use FORGE's OSCAL XML output feature. After implementation, `forge convert` will accept `--format xml` to produce valid OSCAL v1.2.0 XML from Markdown policy documents.

## CLI Usage

### Convert Policy to OSCAL Catalog XML

```bash
# Convert to XML (stdout)
forge convert policy.md --strategy catalog --format xml

# Convert to XML (file output)
forge convert policy.md --strategy catalog --format xml --output catalog.xml
```

### Convert Policy to OSCAL Component Definition XML

```bash
# Convert to Component Definition XML
forge convert policy.md --strategy component --format xml --output component-definition.xml
```

### Compare JSON and XML Output

```bash
# Generate both formats from the same input
forge convert policy.md --strategy catalog --format json --output catalog.json
forge convert policy.md --strategy catalog --format xml --output catalog.xml

# Both outputs contain identical data in different serializations
```

## Expected XML Output

### Catalog XML Structure

```xml
<?xml version="1.0" encoding="UTF-8"?>
<catalog xmlns="http://csrc.nist.gov/ns/oscal/1.0" uuid="550e8400-e29b-41d4-a716-446655440000">
  <metadata>
    <title>Corporate Security Policy</title>
    <last-modified>2026-02-15T12:00:00Z</last-modified>
    <version>2.0</version>
    <oscal-version>1.2.0</oscal-version>
  </metadata>
  <group id="access-control">
    <title>Access Control</title>
    <control id="POL-AC-001">
      <title>All users must use multi-factor authentication.</title>
      <part id="POL-AC-001_smt" name="statement">
        <p>All users must use multi-factor authentication.</p>
      </part>
    </control>
  </group>
  <back-matter>
    <resource uuid="a1b2c3d4-...">
      <title>NIST SP 800-53</title>
      <rlink href="https://nvd.nist.gov/800-53"/>
    </resource>
  </back-matter>
</catalog>
```

### Component Definition XML Structure

```xml
<?xml version="1.0" encoding="UTF-8"?>
<component-definition xmlns="http://csrc.nist.gov/ns/oscal/1.0" uuid="660e8400-...">
  <metadata>
    <title>Corporate Security Policy</title>
    <last-modified>2026-02-15T12:00:00Z</last-modified>
    <version>2.0</version>
    <oscal-version>1.2.0</oscal-version>
  </metadata>
  <component uuid="770e8400-..." type="policy">
    <title>Corporate Security Policy</title>
    <description>Documentary component representing the Corporate Security Policy policy document.</description>
  </component>
</component-definition>
```

## Library Usage (for developers)

### Serialize a Catalog to XML

```rust
use forge::export::xml_serializer::serialize_catalog_to_xml;
use forge::oscal::catalog::OscalCatalog;

// Given an OscalCatalog built by the pipeline...
let xml_string = serialize_catalog_to_xml(&catalog)?;

// xml_string contains the complete OSCAL XML document
assert!(xml_string.starts_with("<?xml"));
assert!(xml_string.contains("xmlns=\"http://csrc.nist.gov/ns/oscal/1.0\""));
```

### Serialize a Component Definition to XML

```rust
use forge::export::xml_serializer::serialize_component_definition_to_xml;
use forge::oscal::component_definition::ComponentDefinition;

let xml_string = serialize_component_definition_to_xml(&component_def)?;
```

## XSD Validation

Generated XML can be validated against OSCAL XSD schemas:

```bash
# Download OSCAL schemas (one-time)
curl -LO https://github.com/usnistgov/OSCAL/releases/download/v1.1.2/oscal-1.1.2.zip
unzip oscal-1.1.2.zip -d oscal-schemas

# Validate catalog XML
xmllint --schema oscal-schemas/xml/schema/oscal_catalog_schema.xsd --noout catalog.xml

# Validate component definition XML
xmllint --schema oscal-schemas/xml/schema/oscal_component_schema.xsd --noout component-definition.xml
```

## Key Differences: JSON vs XML

| Aspect | JSON | XML |
|--------|------|-----|
| Wrapper | `{"catalog": {...}}` | `<catalog xmlns="...">` |
| Arrays | `"groups": [...]` | Repeated `<group>` elements |
| UUID | `"uuid": "..."` (property) | `uuid="..."` (attribute) |
| Props | `{"name": "...", "value": "..."}` | `<prop name="..." value="..."/>` |
| Links | `{"href": "...", "rel": "..."}` | `<link href="..." rel="..."/>` |
| Prose | `"prose": "text"` | `<p>text</p>` |
| Namespace | N/A | `xmlns="http://csrc.nist.gov/ns/oscal/1.0"` |
| Declaration | N/A | `<?xml version="1.0" encoding="UTF-8"?>` |
