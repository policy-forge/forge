# Data Model: End-to-End Catalog Pipeline (WI-13)

## Overview

WI-13 introduces no new domain data model entities. This is a pure integration sprint that wires together existing domain model types (WI-5) and OSCAL Catalog types (WI-9 through WI-12). A new error variant (`ForgeError::Serialization`) is added for JSON serialization failures — this is infrastructure, not a domain entity.

## Existing Entities Used

### Input Domain

| Entity | Module | Role in Pipeline |
|--------|--------|-----------------|
| `IngestedDocument` | `ingest` | Step 1 output: raw file content with line tracking |
| `SectionNode` | `parse` | Step 3 output: hierarchical heading tree |
| `ExtractedContent` | `parse::clauses` | Step 4 output: list items, tables, paragraphs |
| `PolicyDocument` | `model` | Steps 5-7: domain model with sections + requirements |
| `PolicySection` | `model` | Nested sections with requirements and body text |
| `PolicyRequirement` | `model` | Individual atomic requirement with stable_id |
| `DocumentMetadata` | `model` | Source document metadata (title, version, author) |
| `Citation` | `model` | Citation reference (not yet populated by WI-8) |

### Output Domain

| Entity | Module | Role in Pipeline |
|--------|--------|-----------------|
| `CatalogEnvelope` | `oscal::catalog` | JSON wrapper: `{"catalog": {...}}` |
| `OscalCatalog` | `oscal::catalog` | Root catalog with uuid, metadata, groups, back_matter |
| `OscalGroup` | `oscal::catalog` | Group mapped from PolicySection |
| `OscalControl` | `oscal::catalog` | Control mapped from PolicyRequirement |
| `OscalPart` | `oscal::parts` | Statement/guidance parts on controls |
| `OscalProp` | `oscal::parts` | Metadata properties on controls |
| `OscalMetadata` (catalog) | `oscal::catalog` | Placeholder metadata with String fields |
| `OscalMetadata` (real) | `oscal::metadata` | Real metadata with Uuid + DateTime fields |
| `BackMatter` | `oscal::back_matter` | Back matter resources (empty until WI-8) |
| `OscalLink` | `oscal::back_matter` | Control-to-resource links |

### New Type (error handling only)

| Entity | Module | Purpose |
|--------|--------|---------|
| `ForgeError::Serialization` | `error` | New variant for JSON serialization failures |

## Data Flow

```
[Markdown File]
    |
    v
IngestedDocument --> String (content)
    |                   |
    |     +-------------+------------------+
    |     v             v                  |
    | SectionNode[]  ExtractedContent      |
    |     |             |                  |
    |     +-------------+                  |
    |           |                          |
    v           v                          |
PolicyDocument (assembled) <---------------+
    |
    v
PolicyDocument (atomized) --> new copy, compound reqs split
    |
    v
PolicyDocument (with UUIDs) --> mutated, stable_ids populated
    |
    +----------------------------------------------+
    v                                              v
OscalCatalog (groups/controls)     OscalMetadata (real)
    |                                              |
    +----------------------------------------------+
    |
    v
CatalogEnvelope (final assembly: catalog + real metadata + back_matter)
    |
    v
JSON String (pretty-printed)
    |
    v
[stdout or file]
```

## Validation Rules

No new validation rules. All validation handled by existing pipeline stages.

## State Transitions

No new state transitions. Pipeline is a linear transformation:
`File -> IngestedDocument -> PolicyDocument -> PolicyDocument (atomized) -> PolicyDocument (with IDs) -> OscalCatalog -> JSON`
