# Data Model: OSCAL Component Definition Structure

**Phase 1 output** | **Date**: 2026-02-12

## Entities

### ComponentDefinitionEnvelope

JSON envelope producing `{"component-definition": {...}}` at the top level.

| Field | Type | Serde | Required | Source |
|-------|------|-------|----------|--------|
| `component_definition` | `ComponentDefinition` | `#[serde(rename = "component-definition")]` | Yes | Constructed by builder |

### ComponentDefinition

OSCAL Component Definition root structure.

| Field | Type | Serde | Required | Source |
|-------|------|-------|----------|--------|
| `uuid` | `String` | -- | Yes | `Uuid::new_v4().to_string()` from `assemble_metadata` |
| `metadata` | `ComponentDefinitionMetadata` | -- | Yes | Mapped from `assemble_metadata` return |
| `components` | `Vec<DocumentaryComponent>` | -- | Yes | Built from PolicyDocument |
| `back_matter` | `Option<BackMatter>` | `#[serde(rename = "back-matter", skip_serializing_if = "Option::is_none")]` | No | From `generate_back_matter` when citations exist |

### ComponentDefinitionMetadata

Metadata fields mapped from the shared `OscalMetadata` (metadata.rs). Uses String fields for consistency with the Catalog builder's approach.

| Field | Type | Serde | Required | Source |
|-------|------|-------|----------|--------|
| `title` | `String` | -- | Yes | `doc.metadata.title` |
| `last_modified` | `String` | `#[serde(rename = "last-modified")]` | Yes | `assemble_metadata().last_modified.to_rfc3339()` |
| `version` | `String` | -- | Yes | `doc.metadata.version` (default `"0.0.0"`) |
| `oscal_version` | `String` | `#[serde(rename = "oscal-version")]` | Yes | `OSCAL_VERSION` constant (`"1.2.0"`) |

### DocumentaryComponent

A component within the Component Definition of type `"policy"`.

| Field | Type | Serde | Required | Source |
|-------|------|-------|----------|--------|
| `uuid` | `String` | -- | Yes | UUID v5 from `COMPONENT_NAMESPACE` + `"{title}\0{version}"` (null-separated) |
| `component_type` | `String` | `#[serde(rename = "type")]` | Yes | Always `"policy"` |
| `title` | `String` | -- | Yes | `doc.metadata.title` or `"Untitled Policy Document"` |
| `description` | `String` | -- | Yes | Template: `"Documentary component representing the {title} policy document."` |
| `control_implementations` | `Vec<serde_json::Value>` | `#[serde(rename = "control-implementations")]` | Yes (empty) | Empty vec (placeholder for WI-15) |

## Relationships

```text
ComponentDefinitionEnvelope 1--1 ComponentDefinition
ComponentDefinition 1--1 ComponentDefinitionMetadata
ComponentDefinition 1--* DocumentaryComponent (exactly 1 for now)
ComponentDefinition 1--0..1 BackMatter (existing type from back_matter.rs)
```

## Validation Rules

- `ComponentDefinition.uuid`: Valid UUID v4 string (from `assemble_metadata`)
- `ComponentDefinition.components`: Exactly 1 entry (A-2)
- `DocumentaryComponent.uuid`: Deterministic UUID v5 -- same input always produces same output
- `DocumentaryComponent.component_type`: Always `"policy"` (M-3)
- `DocumentaryComponent.title`: Non-empty; defaults to `"Untitled Policy Document"` when PolicyDocument title is empty (EC-1)
- `DocumentaryComponent.description`: Non-empty; always uses template format
- `DocumentaryComponent.control_implementations`: Empty array (W-1)
- `ComponentDefinitionMetadata.version`: Defaults to `"0.0.0"` when PolicyDocument version is empty (EC-2)
- `ComponentDefinitionMetadata.oscal_version`: Always `"1.2.0"` (M-2)

## Constants

| Name | Location | Value | Purpose |
|------|----------|-------|---------|
| `COMPONENT_NAMESPACE` | `src/uuid.rs` | `Uuid::new_v5(&FORGE_NAMESPACE_UUID, b"component")` | UUID v5 namespace for documentary component IDs |
| `DEFAULT_COMPONENT_TITLE` | `src/oscal/component_definition.rs` | `"Untitled Policy Document"` | Fallback title for empty PolicyDocument title |
