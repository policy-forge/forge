# Data Model: 028 Multi-Format Round-Trip Testing

**Date**: 2026-02-15

## Overview

This work item introduces no new domain entities. It adds two supporting data types for semantic equivalence comparison and XML deserialization capability for existing model types.

## New Types

### EquivalenceResult

Structured result from comparing two OSCAL documents for semantic equivalence.

| Field | Type | Description |
|-------|------|-------------|
| `is_equivalent` | `bool` | Whether the two documents are semantically equivalent |
| `differences` | `Vec<EquivalenceDiff>` | List of differences found; empty if equivalent |

**Relationships**: Contains zero or more `EquivalenceDiff` entries.
**Validation**: `is_equivalent` must be `true` if and only if `differences` is empty.
**Location**: `src/testing/semantic_eq.rs`

### EquivalenceDiff

A single difference found during semantic comparison.

| Field | Type | Description |
|-------|------|-------------|
| `path` | `String` | JSON Pointer-style path (e.g., `/catalog/metadata/title`) |
| `description` | `String` | Human-readable description of the difference |
| `expected` | `Option<String>` | String representation of expected value (original document) |
| `actual` | `Option<String>` | String representation of actual value (round-tripped document) |

**Relationships**: Contained within `EquivalenceResult.differences`.
**Validation**: `path` must be a valid JSON Pointer prefix. `expected` is `None` for extra keys; `actual` is `None` for missing keys.
**Location**: `src/testing/semantic_eq.rs`

## Existing Types Modified

### OscalProp (`src/oscal/parts.rs`)

**Change**: Add serde annotations for XML attribute deserialization compatibility.

Current fields: `name`, `value`, `ns` (all `String` / `Option<String>`).

XML attributes (`<prop name="..." value="..." ns="..." />`) require special handling for `quick-xml` serde deserialization. The exact annotation strategy depends on R-6 compatibility testing during implementation.

### OscalLink (`src/oscal/back_matter.rs`)

**Change**: Similar XML attribute annotation may be needed for `href`, `rel` attributes on `<link>` elements.

## Existing Types Used (No Modification)

| Type | Location | Role in Round-Trip |
|------|----------|-------------------|
| `CatalogEnvelope` | `src/oscal/catalog.rs` | JSON deserialization target; XML serialization source |
| `OscalCatalog` | `src/oscal/catalog.rs` | Core model for catalog round-trip |
| `ComponentDefinitionEnvelope` | `src/oscal/component_definition.rs` | JSON deserialization target; XML serialization source |
| `ComponentDefinition` | `src/oscal/component_definition.rs` | Core model for component round-trip |
| `ForgeError` | `src/error.rs` | Error propagation from serialization/deserialization |

## Entity Relationship Diagram

```text
EquivalenceResult
├── is_equivalent: bool
└── differences: Vec<EquivalenceDiff>
    └── EquivalenceDiff
        ├── path: String
        ├── description: String
        ├── expected: Option<String>
        └── actual: Option<String>

Round-Trip Flow (no new entities):
  CatalogEnvelope (JSON) ──serialize──> XML String ──deserialize──> CatalogEnvelope ──to_value──> serde_json::Value
  CatalogEnvelope (JSON) ──serialize──> YAML String ──deserialize──> CatalogEnvelope ──to_value──> serde_json::Value
```
