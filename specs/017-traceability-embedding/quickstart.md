# Quickstart — Traceability Embedding (WI-17)

**Date**: 2026-02-13

## Overview

WI-17 embeds source provenance metadata into generated OSCAL JSON artifacts. Every control (Catalog) and implemented-requirement (Component Definition) receives 3 namespaced props + 1 source link, enabling bidirectional traceability between OSCAL elements and their source policy text.

## Usage

No new CLI flags. Trace embedding is automatic when generating OSCAL artifacts:

```bash
# Catalog — trace props/links are embedded automatically
cargo run -- catalog input.md -o catalog.json

# Component Definition — trace props/links are embedded automatically
cargo run -- component input.md --source-profile ./baseline.json -o component.json
```

## Output Examples

### Catalog Control (after WI-17)

```json
{
  "id": "POL-AC-001",
  "uuid": "a1b2c3d4-...",
  "title": "All users must authenticate.",
  "props": [
    {
      "name": "source-file",
      "ns": "https://forge.policy-forge.github.io/ns/trace",
      "value": "policies/access-control.md"
    },
    {
      "name": "source-section",
      "ns": "https://forge.policy-forge.github.io/ns/trace",
      "value": "3.1 Access Control"
    },
    {
      "name": "source-line",
      "ns": "https://forge.policy-forge.github.io/ns/trace",
      "value": "42"
    }
  ],
  "links": [
    {
      "rel": "source",
      "href": "policies/access-control.md#line=42"
    }
  ],
  "parts": [...]
}
```

### Catalog Group (after WI-17)

```json
{
  "id": "access-control",
  "title": "Access Control",
  "props": [
    {
      "name": "source-section",
      "ns": "https://forge.policy-forge.github.io/ns/trace",
      "value": "Access Control"
    }
  ],
  "controls": [...]
}
```

### Component Definition — Documentary Component (after WI-17)

```json
{
  "uuid": "e5f6a7b8-...",
  "type": "policy",
  "title": "Corporate Security Policy",
  "description": "Documentary component representing the Corporate Security Policy policy document.",
  "props": [
    {
      "name": "source-file",
      "ns": "https://forge.policy-forge.github.io/ns/trace",
      "value": "policies/access-control.md"
    }
  ],
  "control-implementations": [...]
}
```

### Implemented Requirement (after WI-17)

```json
{
  "uuid": "f9a0b1c2-...",
  "control-id": "POL-AC-001",
  "description": "All users must authenticate using MFA.",
  "props": [
    {
      "name": "source-file",
      "ns": "https://forge.policy-forge.github.io/ns/trace",
      "value": "policies/access-control.md"
    },
    {
      "name": "source-section",
      "ns": "https://forge.policy-forge.github.io/ns/trace",
      "value": "3.1 Access Control"
    },
    {
      "name": "source-line",
      "ns": "https://forge.policy-forge.github.io/ns/trace",
      "value": "42"
    }
  ],
  "links": [
    {
      "rel": "source",
      "href": "policies/access-control.md#line=42"
    }
  ]
}
```

## API Usage (for library consumers)

### Building Trace Props

```rust
use forge::oscal::trace_embedding::{
    FORGE_TRACE_NS, build_trace_props, build_trace_link,
};

// Build 3 namespaced trace props
let props = build_trace_props("policy.md", "Access Control", 42);
assert_eq!(props.len(), 3);
assert_eq!(props[0].name, "source-file");
assert_eq!(props[0].ns, Some(FORGE_TRACE_NS.to_string()));
assert_eq!(props[0].value, "policy.md");

// Build 1 source link
let link = build_trace_link("policy.md", 42);
assert_eq!(link.rel, "source");
assert_eq!(link.href, "policy.md#line=42");
```

### Embedding Trace in Catalog (post-processing)

```rust
use forge::oscal::trace_embedding::embed_trace_in_catalog;
use forge::model::trace::TraceLinkCollection;

// After building catalog with trace link collection
let mut trace_links = TraceLinkCollection::new();
let mut catalog = build_catalog(&document, Some(&mut trace_links))?;

// Inject trace props/links into all controls and groups
embed_trace_in_catalog(&mut catalog, &trace_links);

// catalog.groups[0].controls[0].props now has 3 trace props
// catalog.groups[0].controls[0].links now has 1 source link
// catalog.groups[0].props now has source-section prop
```

## Key Constants

| Constant | Value | Purpose |
|----------|-------|---------|
| `FORGE_TRACE_NS` | `https://forge.policy-forge.github.io/ns/trace` | Namespace URI for all FORGE trace props |
| `PROP_SOURCE_FILE` | `source-file` | Source file path prop name |
| `PROP_SOURCE_SECTION` | `source-section` | Source section title prop name |
| `PROP_SOURCE_LINE` | `source-line` | Source line number prop name |
| `LINK_REL_SOURCE` | `source` | Link relationship type |

## Migrating from `forge:source-line`

WI-17 replaces the old prefix-based `forge:source-line` prop with three namespaced props. If you were consuming the old prop:

**Before** (WI-10):
```json
{"name": "forge:source-line", "value": "42"}
```

**After** (WI-17):
```json
[
  {"name": "source-file", "ns": "https://forge.policy-forge.github.io/ns/trace", "value": "policy.md"},
  {"name": "source-section", "ns": "https://forge.policy-forge.github.io/ns/trace", "value": "Access Control"},
  {"name": "source-line", "ns": "https://forge.policy-forge.github.io/ns/trace", "value": "42"}
]
```

Consumers should filter props by `ns == "https://forge.policy-forge.github.io/ns/trace"` to find FORGE trace props.
