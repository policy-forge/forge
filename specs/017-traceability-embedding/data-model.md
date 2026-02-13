# Data Model — Traceability Embedding (WI-17)

**Date**: 2026-02-13
**Status**: Complete

## Entity Changes

### Modified: `OscalProp` (parts.rs:56-63)

**Current** (2 fields):

```rust
pub struct OscalProp {
    pub name: String,
    pub value: String,
}
```

**After** (3 fields):

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OscalProp {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,
    pub value: String,
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | `String` | Yes | Property name (e.g., `"source-file"`) |
| `ns` | `Option<String>` | No | Namespace URI. `None` omits field from JSON. FORGE trace props use `FORGE_TRACE_NS`. |
| `value` | `String` | Yes | Property value as string |

**Serialization order**: `name`, `ns`, `value` — matches OSCAL v1.2.0 convention.

**Migration**: All existing `OscalProp` construction sites must add `ns: None` for non-trace props. The `build_control_props` function (parts.rs:194) is simplified to return `vec![]` since trace props are now added by `embed_trace_in_catalog`.

### Modified: `OscalGroup` (catalog.rs:45-54)

**Current** (3 fields):

```rust
pub struct OscalGroup {
    pub id: String,
    pub title: String,
    pub controls: Vec<OscalControl>,
}
```

**After** (5 fields):

```rust
#[derive(Debug, Clone, Serialize)]
pub struct OscalGroup {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub props: Vec<OscalProp>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<OscalLink>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub controls: Vec<OscalControl>,
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | `String` | Yes | Slugified section title |
| `title` | `String` | Yes | Section title verbatim |
| `props` | `Vec<OscalProp>` | No | Group-level properties (S-1: `source-section`). Omitted when empty. |
| `links` | `Vec<OscalLink>` | No | Group-level links. Omitted when empty. |
| `controls` | `Vec<OscalControl>` | No | Child controls. Omitted when empty. |

**Serialization order**: `id`, `title`, `props`, `links`, `controls` — `props`/`links` before `controls` per OSCAL convention.

### Modified: `DocumentaryComponent` (component_definition.rs:72-90)

**Current** (5 fields):

```rust
pub struct DocumentaryComponent {
    pub uuid: String,
    pub component_type: String,
    pub title: String,
    pub description: String,
    pub control_implementations: Vec<serde_json::Value>,
}
```

**After** (6 fields):

```rust
#[derive(Debug, Clone, Serialize)]
pub struct DocumentaryComponent {
    pub uuid: String,
    #[serde(rename = "type")]
    pub component_type: String,
    pub title: String,
    pub description: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub props: Vec<OscalProp>,
    #[serde(rename = "control-implementations")]
    pub control_implementations: Vec<serde_json::Value>,
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `uuid` | `String` | Yes | Deterministic UUID v5 |
| `component_type` | `String` | Yes | Always `"policy"` |
| `title` | `String` | Yes | Component title |
| `description` | `String` | Yes | Component description |
| `props` | `Vec<OscalProp>` | No | Component-level properties (M-5: `source-file`). Omitted when empty. |
| `control_implementations` | `Vec<Value>` | Yes | Control implementations array |

**Serialization order**: `uuid`, `type`, `title`, `description`, `props`, `control-implementations`.

## New Module: `trace_embedding` (src/oscal/trace_embedding.rs)

### Constants

| Name | Value | Usage |
|------|-------|-------|
| `FORGE_TRACE_NS` | `"https://forge.policy-forge.github.io/ns/trace"` | Namespace for all FORGE trace props (M-6, SEC-4) |
| `PROP_SOURCE_FILE` | `"source-file"` | Prop name for source file path (M-1, M-3, M-5) |
| `PROP_SOURCE_SECTION` | `"source-section"` | Prop name for section title (M-1, M-3, S-1) |
| `PROP_SOURCE_LINE` | `"source-line"` | Prop name for line number (M-1, M-3) |
| `LINK_REL_SOURCE` | `"source"` | Link rel for source references (M-2, M-4) |

### Helper Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `build_trace_props` | `(file: &str, section: &str, line: usize) -> Vec<OscalProp>` | Returns 3 namespaced props: source-file, source-section, source-line |
| `build_trace_link` | `(file: &str, line: usize) -> OscalLink` | Returns 1 link with `rel: "source"`, `href: "<file>#line=<n>"` |
| `encode_href_path` | `(path: &str) -> String` | Percent-encodes `%`, space, `#` in file paths (SEC-3, EC-6) |

### Embedding Function

| Function | Signature | Description |
|----------|-----------|-------------|
| `embed_trace_in_catalog` | `(catalog: &mut OscalCatalog, trace_links: &TraceLinkCollection)` | Walks groups/controls, injects trace props/links from TraceLinkCollection |

## Relationships

```text
TraceLinkCollection (WI-16)
  └── by_oscal_element(control.uuid) -> TraceLink
        └── source_location: SourceLocation
              ├── file_path: PathBuf   → source-file prop value
              ├── section_title: String → source-section prop value
              └── line_number: usize   → source-line prop value + link href fragment

OscalCatalog
  └── groups: Vec<OscalGroup>
        ├── props: [source-section]     (S-1, derived from first child control's trace link)
        └── controls: Vec<OscalControl>
              ├── props: [source-file, source-section, source-line]   (M-1)
              └── links: [rel=source, href=file#line=N]               (M-2)

ComponentDefinitionEnvelope
  └── component_definition
        └── components: [DocumentaryComponent]
              ├── props: [source-file]                                 (M-5)
              └── control_implementations: [JSON]
                    └── implemented-requirements: [JSON]
                          ├── props: [source-file, source-section, source-line]  (M-3)
                          └── links: [rel=source, href=file#line=N]              (M-4)
```

## Validation Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| All trace props must have `ns: Some(FORGE_TRACE_NS)` | M-6, SEC-4 | `build_trace_props` always sets ns |
| Prop names must use constants, never raw literals | SEC-5 | Code review + constant usage in all construction sites |
| No trace data in `remarks` fields | M-7, SEC-1, SEC-2 | No code path writes to remarks; unit test verifies |
| Link href must be percent-encoded | EC-6, SEC-3 | `encode_href_path` called in `build_trace_link` |
| Group source-section only when derivable | EC-4, S-1 | Skip prop when no child controls have trace links |
