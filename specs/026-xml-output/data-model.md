# Data Model: OSCAL XML Output (026)

**Feature**: 026-xml-output
**Date**: 2026-02-15
**Status**: Complete

## Overview

No new data model is introduced. This feature adds an XML serialization layer for existing OSCAL model structs. The data model below documents the mapping from Rust struct fields to OSCAL XML elements and attributes.

## Entity-to-XML Mapping

### OscalCatalog → `<catalog>`

**Source**: `src/oscal/catalog.rs:31` (`OscalCatalog`)
**XML Root Element**: `<catalog xmlns="http://csrc.nist.gov/ns/oscal/1.0" uuid="...">`

| Rust Field | Type | XML Representation | XML Position | XSD Order |
|------------|------|--------------------|--------------|-----------|
| `uuid` | `String` | Attribute `uuid` | `<catalog uuid="...">` | attribute |
| `metadata` | `OscalMetadata` | Child element `<metadata>` | 1st child | 1 |
| `groups` | `Vec<OscalGroup>` | Repeated `<group>` elements | after metadata | 4 |
| `back_matter` | `Option<BackMatter>` | Child element `<back-matter>` | last child | 5 |

**Notes**: `param` and standalone `control` elements (positions 2-3 in XSD) are not currently produced by FORGE's catalog builder. Groups contain all controls.

### OscalGroup → `<group>`

**Source**: `src/oscal/catalog.rs:46` (`OscalGroup`)

| Rust Field | Type | XML Representation | XML Position | XSD Order |
|------------|------|--------------------|--------------|-----------|
| `id` | `String` | Attribute `id` | `<group id="...">` | attribute |
| `title` | `String` | Child element `<title>` | 1st child | 1 |
| `props` | `Vec<OscalProp>` | Repeated `<prop>` elements | after title | 3 |
| `links` | `Vec<OscalLink>` | Repeated `<link>` elements | after props | 4 |
| `controls` | `Vec<OscalControl>` | Repeated `<control>` elements | last | 7 |

**Notes**: `param`, `part`, and nested `group` elements (XSD positions 2, 5, 6) are not currently produced by FORGE.

### OscalControl → `<control>`

**Source**: `src/oscal/catalog.rs:64` (`OscalControl`)

| Rust Field | Type | XML Representation | XML Position | XSD Order |
|------------|------|--------------------|--------------|-----------|
| `id` | `String` | Attribute `id` | `<control id="...">` | attribute |
| `uuid` | `String` | **SKIPPED** (not serialized in JSON either) | — | — |
| `title` | `String` | Child element `<title>` | 1st child | 1 |
| `props` | `Vec<OscalProp>` | Repeated `<prop>` elements | after title | 3 |
| `links` | `Vec<OscalLink>` | Repeated `<link>` elements | after props | 4 |
| `parts` | `Vec<OscalPart>` | Repeated `<part>` elements | after links | 5 |

**Notes**: `uuid` is marked `#[serde(skip_serializing)]` in the catalog schema — OSCAL catalog XSD does not allow uuid on controls. `param` (XSD position 2) and nested `control` (XSD position 6) are not currently produced by FORGE.

### OscalPart → `<part>`

**Source**: `src/oscal/parts.rs:26` (`OscalPart`)

| Rust Field | Type | XML Representation | XML Position | XSD Order |
|------------|------|--------------------|--------------|-----------|
| `id` | `String` | Attribute `id` | `<part id="...">` | attribute |
| `name` | `String` | Attribute `name` | `<part name="...">` | attribute |
| `prose` | `String` | Child `<p>` element(s) | after props | 3 (blockElementGroup) |
| `parts` | `Vec<OscalPart>` | Repeated `<part>` elements | after prose | 4 |
| `props` | `Vec<OscalProp>` | Repeated `<prop>` elements | after title | 2 |

**Notes**: `title` (XSD position 1) and `link` (XSD position 5) are not currently produced on parts by FORGE. The `ns` and `class` attributes are optional in XSD and not currently set by FORGE. Prose maps to `<p>` element(s) within the blockElementGroup.

### OscalProp → `<prop>`

**Source**: `src/oscal/parts.rs:59` (`OscalProp`)

| Rust Field | Type | XML Representation | XML Position | XSD Order |
|------------|------|--------------------|--------------|-----------|
| `name` | `String` | Attribute `name` (required) | `<prop name="...">` | attribute |
| `ns` | `Option<String>` | Attribute `ns` (optional) | `<prop ns="...">` | attribute |
| `value` | `String` | Attribute `value` (required) | `<prop value="...">` | attribute |

**Notes**: `prop` in OSCAL XML is primarily an **attribute-bearing element**. The `name` and `value` are XML attributes, NOT child elements. Optional `remarks` child (XSD position 1) is not produced by FORGE. Additional attributes (`uuid`, `class`, `group`) are optional in XSD and not currently set.

### OscalLink → `<link>`

**Source**: `src/oscal/back_matter.rs:74` (`OscalLink`)

| Rust Field | Type | XML Representation | XML Position | XSD Order |
|------------|------|--------------------|--------------|-----------|
| `href` | `String` | Attribute `href` (required) | `<link href="...">` | attribute |
| `rel` | `String` | Attribute `rel` | `<link rel="...">` | attribute |
| `text` | `Option<String>` | Child element `<text>` | 1st child | 1 |

### OscalMetadata (Catalog) → `<metadata>`

**Source**: `src/oscal/catalog.rs` (inline `OscalMetadata` on `OscalCatalog`)

The catalog uses a simplified metadata struct with these fields:

| Rust Field | Type | XML Representation | XML Position | XSD Order |
|------------|------|--------------------|--------------|-----------|
| `title` | `String` | Child element `<title>` | 1st child | 1 |
| `last_modified` | `String` | Child element `<last-modified>` | after title | 3 |
| `version` | `String` | Child element `<version>` | after last-modified | 4 |
| `oscal_version` | `String` | Child element `<oscal-version>` | after version | 5 |

**Notes**: `published` (XSD position 2) and other optional metadata fields are not currently produced by FORGE's catalog metadata.

### OscalMetadata (shared) → `<metadata>`

**Source**: `src/oscal/metadata.rs:30` (`OscalMetadata`)

| Rust Field | Type | XML Representation | XML Position | XSD Order |
|------------|------|--------------------|--------------|-----------|
| `uuid` | `Uuid` | **Not serialized to XML** (metadata uuid is not in XSD) | — | — |
| `title` | `String` | Child element `<title>` | 1st child | 1 |
| `last_modified` | `DateTime<Utc>` | Child element `<last-modified>` | after title | 3 |
| `version` | `String` | Child element `<version>` | after last-modified | 4 |
| `oscal_version` | `String` | Child element `<oscal-version>` | after version | 5 |

### BackMatter → `<back-matter>`

**Source**: `src/oscal/back_matter.rs:17` (`BackMatter`)

| Rust Field | Type | XML Representation | XML Position | XSD Order |
|------------|------|--------------------|--------------|-----------|
| `resources` | `Vec<BackMatterResource>` | Repeated `<resource>` elements | children | 1 |

### BackMatterResource → `<resource>`

**Source**: `src/oscal/back_matter.rs:28` (`BackMatterResource`)

| Rust Field | Type | XML Representation | XML Position | XSD Order |
|------------|------|--------------------|--------------|-----------|
| `uuid` | `Uuid` | Attribute `uuid` | `<resource uuid="...">` | attribute |
| `title` | `String` | Child element `<title>` | 1st child | 1 |
| `description` | `Option<String>` | Child element `<description>` | after title | 2 |
| `citation` | `Option<ResourceCitation>` | Child element `<citation>` | after document-id | 5 |
| `rlinks` | `Vec<Rlink>` | Repeated `<rlink>` elements | after citation | 6 |
| `props` | `Vec<Prop>` | Repeated `<prop>` elements | after description | 3 |

### Rlink → `<rlink>`

**Source**: `src/oscal/back_matter.rs:60` (`Rlink`)

| Rust Field | Type | XML Representation | XML Position | XSD Order |
|------------|------|--------------------|--------------|-----------|
| `href` | `String` | Attribute `href` (required) | `<rlink href="...">` | attribute |
| `media_type` | `Option<String>` | Attribute `media-type` | `<rlink media-type="...">` | attribute |

### ResourceCitation → `<citation>`

**Source**: `src/oscal/back_matter.rs:53` (`ResourceCitation`)

| Rust Field | Type | XML Representation | XML Position | XSD Order |
|------------|------|--------------------|--------------|-----------|
| `text` | `String` | Child element `<text>` | 1st child | 1 |

### ComponentDefinition → `<component-definition>`

**Source**: `src/oscal/component_definition.rs:36` (`ComponentDefinition`)
**XML Root Element**: `<component-definition xmlns="http://csrc.nist.gov/ns/oscal/1.0" uuid="...">`

| Rust Field | Type | XML Representation | XML Position | XSD Order |
|------------|------|--------------------|--------------|-----------|
| `uuid` | `String` | Attribute `uuid` | `<component-definition uuid="...">` | attribute |
| `metadata` | `ComponentDefinitionMetadata` | Child element `<metadata>` | 1st child | 1 |
| `components` | `Vec<DocumentaryComponent>` | Repeated `<component>` elements | after metadata | 3 |
| `back_matter` | `Option<BackMatter>` | Child element `<back-matter>` | last child | 5 |

**Notes**: `import-component-definition` (XSD position 2) and `capability` (XSD position 4) are not currently produced by FORGE.

### ComponentDefinitionMetadata → `<metadata>`

**Source**: `src/oscal/component_definition.rs:55` (`ComponentDefinitionMetadata`)

| Rust Field | Type | XML Representation | XML Position | XSD Order |
|------------|------|--------------------|--------------|-----------|
| `title` | `String` | Child element `<title>` | 1st child | 1 |
| `last_modified` | `String` | Child element `<last-modified>` | after title | 3 |
| `version` | `String` | Child element `<version>` | after last-modified | 4 |
| `oscal_version` | `String` | Child element `<oscal-version>` | after version | 5 |

### DocumentaryComponent → `<component>`

**Source**: `src/oscal/component_definition.rs:73` (`DocumentaryComponent`)

| Rust Field | Type | XML Representation | XML Position | XSD Order |
|------------|------|--------------------|--------------|-----------|
| `uuid` | `String` | Attribute `uuid` | `<component uuid="...">` | attribute |
| `component_type` | `String` | Attribute `type` | `<component type="...">` | attribute |
| `title` | `String` | Child element `<title>` | 1st child | 1 |
| `description` | `String` | Child element `<description>` | after title | 2 |
| `props` | `Vec<OscalProp>` | Repeated `<prop>` elements | after purpose | 4 |
| `control_implementations` | `Vec<serde_json::Value>` | **SKIPPED** for WI-26 (complex nested structure) | — | — |

**Notes**: `purpose` (XSD position 3) is not currently set by FORGE. `control-implementations` contains nested `serde_json::Value` objects that would require deeper parsing to serialize to XML — this is deferred to a future WI.

## Validation Rules

1. **UUID format**: All UUID attributes must be valid UUID strings (v4 or v5)
2. **Required attributes**: `prop` requires `name` and `value`; `link` requires `href`; `rlink` requires `href`
3. **Element ordering**: All child elements must follow XSD sequence ordering (see tables above)
4. **Namespace**: Root element must declare `xmlns="http://csrc.nist.gov/ns/oscal/1.0"`
5. **XML declaration**: Must include `<?xml version="1.0" encoding="UTF-8"?>`
6. **Text escaping**: All text content must be XML-escaped (`<` → `&lt;`, `>` → `&gt;`, `&` → `&amp;`)

## State Transitions

N/A — XML serialization is a stateless transformation.
