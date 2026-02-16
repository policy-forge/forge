# Research: OSCAL XML Output (026)

**Feature**: 026-xml-output
**Date**: 2026-02-15
**Status**: Complete

## Research Questions

### RQ-1: quick-xml API Patterns for OSCAL XML Serialization

**Decision**: Use `quick-xml::Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2)` for streaming XML construction with 2-space indentation.

**Rationale**: `quick-xml` provides a low-level, high-performance XML writer that gives precise control over element ordering, attribute placement, and namespace handling — all critical for OSCAL XSD compliance. The `Writer` API uses event-based construction (`Event::Start`, `Event::End`, `Event::Text`, `Event::Decl`) which maps naturally to recursive OSCAL element traversal.

**Key API Patterns**:
- `BytesDecl::new("1.0", Some("UTF-8"), None)` — XML declaration
- `BytesStart::new("element-name")` + `.push_attribute(("key", "value"))` — element with attributes
- `BytesText::new("content")` — text content (auto-escapes `<`, `>`, `&`, `"`, `'`)
- `BytesEnd::new("element-name")` — closing tag
- `Writer::create_element("name").with_attribute(("k", "v")).write_text_content(text)` — convenience API for simple elements
- `writer.into_inner().into_inner()` — extract byte buffer after writing

**Alternatives Considered**:
- `serde-xml-rs`: Abandoned (no updates since 2022); cannot control element ordering; poor namespace support. Rejected.
- `xml-rs::EventWriter`: Similar control to quick-xml but 2-5x slower and less actively maintained. Rejected.
- `quick-xml` serde feature (`se::Serializer`): Cannot guarantee OSCAL element ordering through serde derive. Rejected — manual construction required.

### RQ-2: OSCAL XML Element Ordering (XSD Sequence Definitions)

**Decision**: All XML elements must be written in the exact order prescribed by the OSCAL v1.2.0 XSD sequence definitions. Element ordering verified against actual XSD schema files (`oscal_catalog_schema.xsd`, `oscal_component_schema.xsd`).

**Rationale**: OSCAL XML schemas use `<xs:sequence>` which mandates a specific ordering. Out-of-order elements fail XSD validation even if they are individually well-formed. This is the primary reason derive-based serialization is insufficient.

**Element Ordering Reference** (extracted from OSCAL v1.1.2 XSD — compatible with v1.2.0):

| Element | Child Ordering (XSD sequence) | Attributes |
|---------|-------------------------------|------------|
| `catalog` | metadata, param*, control*, group*, back-matter? | uuid |
| `group` | title, param*, prop*, link*, part*, group*, control* | id |
| `control` | title, param*, prop*, link*, part*, control* | id |
| `part` | title?, prop*, (prose/p)*, part*, link* | id, name, ns, class |
| `prop` | remarks? | name(req), uuid, ns, value(req), class, group |
| `link` | text? | href(req), rel, media-type, resource-fragment |
| `metadata` | title, published?, last-modified, version, oscal-version, revisions?, ... | — |
| `back-matter` | resource* | — |
| `resource` | title?, description?, prop*, document-id*, citation?, rlink*, ... | uuid |
| `component-definition` | metadata, import-component-definition*, component*, capability*, back-matter? | uuid |
| `component` | title, description, purpose?, prop*, link*, ... | uuid, type |

**Key Mapping Rules**:
- JSON plural arrays (`groups`, `controls`, `props`) → repeated singular XML elements (`group`, `control`, `prop`) — NO wrapper elements
- JSON `uuid` field → XML attribute on parent element
- JSON `id` field → XML attribute on parent element
- JSON `prose` field → XML `<p>` element(s) within blockElementGroup
- JSON hyphenated names (`last-modified`, `oscal-version`, `back-matter`) → XML element names with same hyphenation
- `prop` has `name`, `value`, `ns`, `uuid` as XML **attributes** (not child elements), with optional `remarks` child
- `link` has `href`, `rel`, `media-type` as XML **attributes**, with optional `text` child
- `rlink` has `href`, `media-type` as XML **attributes**

**Alternatives Considered**:
- Guessing element ordering from JSON key ordering: Unreliable, JSON object keys are unordered. Rejected.
- Using OSCAL Metaschema tooling to auto-generate ordering: Adds build-time dependency on Java/Node.js toolchain. Rejected — manual ordering from XSD is sufficient and verified.

### RQ-3: XSD Validation Approach

**Decision**: Use `xmllint --schema <xsd_path> --noout <xml_file>` via `std::process::Command` for OSCAL XSD validation in integration tests.

**Rationale**: Pure-Rust XSD validation crates are immature and incomplete. `xmllint` (part of `libxml2`) is pre-installed on macOS (`/usr/bin/xmllint`) and available via package managers on Linux. It is the standard tool for XSD validation and is used by the OSCAL community. Using it in integration tests provides authoritative validation without adding fragile Rust dependencies.

**Implementation Approach**:
1. Download OSCAL v1.1.2 XSD schemas to `tests/fixtures/xsd/` (committed to repo or downloaded in CI)
2. In integration tests, serialize OSCAL model to XML string, write to temp file
3. Run `xmllint --schema tests/fixtures/xsd/oscal_catalog_schema.xsd --noout temp.xml`
4. Check exit code: 0 = valid, non-zero = validation errors
5. Parse stderr for error details on failure
6. Skip tests if `xmllint` is not available (CI ensures it's present)

**Alternatives Considered**:
- Pure-Rust XSD validation (`xmlschema` crate): Immature, incomplete XSD support, limited maintenance. Rejected.
- oscal-cli Java tool: Requires JVM installation, heavyweight for CI. Rejected for unit tests, possible for CI-only validation.
- Skip validation entirely: Unacceptable for compliance tooling (PRD M-5). Rejected.

### RQ-4: OSCAL Namespace Handling

**Decision**: Add `xmlns="http://csrc.nist.gov/ns/oscal/1.0"` as an attribute on the root element only (`<catalog>` or `<component-definition>`). No namespace prefix is used (default namespace).

**Rationale**: OSCAL XML uses a default (unprefixed) namespace declaration on the root element. All child elements inherit this namespace. Adding the namespace to every child element would be redundant and produce unnecessarily verbose XML. The `quick-xml` attribute API handles namespace declaration as a regular attribute: `root.push_attribute(("xmlns", OSCAL_NS))`.

**Alternatives Considered**:
- Prefix-based namespace (`oscal:catalog`): Not used by NIST OSCAL examples. Rejected.
- Namespace on every element: Redundant, verbose, non-standard for OSCAL. Rejected.

### RQ-5: Prose-to-XML Mapping

**Decision**: Map the JSON `prose` field to a single `<p>` element within the OSCAL blockElementGroup. Multi-paragraph prose (containing `\n\n`) maps to multiple `<p>` elements.

**Rationale**: In OSCAL XML, the `prose` field from JSON corresponds to inline markup within a `<part>` element's blockElementGroup. The simplest correct mapping is to wrap each paragraph in a `<p>` element. The `quick-xml` `BytesText` type handles XML escaping of special characters within the prose text.

**Alternatives Considered**:
- Raw text without `<p>` wrapper: Would violate the OSCAL XSD which expects blockElementGroup content. Rejected.
- Full Markdown-to-XHTML conversion: Over-engineering for this WI; FORGE prose is already plain text or simple Markdown. Rejected — simple `<p>` wrapping is sufficient for XSD compliance.
