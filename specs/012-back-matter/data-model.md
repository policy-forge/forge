# Data Model: OSCAL Back Matter Generation

**Feature**: 012-back-matter | **Date**: 2026-02-12

## Entity Overview

```text
Citation (input, from WI-8)
    │
    ▼
BackMatter ──contains──▶ Resource[] ──has──▶ Rlink[] (URL citations)
                              │              ResourceCitation (bibliographic)
                              │              Prop[] (annotations)
                              │
OscalControl ──contains──▶ OscalLink[] ──references──▶ Resource (via #uuid)
```

## Entities

### Citation (Input — Domain Model)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | String | Yes | Unique identifier for this citation (enables resource map lookup) |
| text | String | Yes | Citation text (bibliographic reference or description) |
| url | Option\<String\> | No | URL if this is a URL-based citation; None for bibliographic-only |
| source_requirement_id | Option\<String\> | No | stable_id of the PolicyRequirement that references this citation |

**Validation Rules**:
- `id` must be non-empty
- `text` must be non-empty (empty text → warning, skip resource generation per AR error handling)
- `url: Some("")` is treated as malformed (annotated with url-status prop)
- `url: None` → bibliographic citation (citation.text path)

**Location**: `src/model/mod.rs`

### BackMatter (Output — OSCAL)

| Field | Type | Required | Serialization |
|-------|------|----------|---------------|
| resources | Vec\<BackMatterResource\> | Yes | Always present when BackMatter exists |

**Validation Rules**:
- If resources is empty, the entire BackMatter should not be serialized (omit per clarification)

**Location**: `src/oscal/back_matter.rs`

### BackMatterResource (Output — OSCAL)

| Field | Type | Required | Serialization |
|-------|------|----------|---------------|
| uuid | Uuid | Yes | Always serialized |
| title | String | Yes | Always serialized |
| description | Option\<String\> | No | `skip_serializing_if = "Option::is_none"` |
| citation | Option\<ResourceCitation\> | No | `skip_serializing_if = "Option::is_none"` |
| rlinks | Vec\<Rlink\> | No | `skip_serializing_if = "Vec::is_empty"` |
| props | Vec\<Prop\> | No | `skip_serializing_if = "Vec::is_empty"` |

**Validation Rules**:
- `uuid` is deterministic UUID v5 from `BACK_MATTER_NAMESPACE` + normalized citation content
- `title` derived from citation text (preferred) or full URL (fallback for URL-only citations)
- Must have either `citation` (bibliographic) or `rlinks` (URL-based), or both
- Malformed URLs → rlink preserved + `Prop { name: "url-status", value: "unvalidated" }`

**Location**: `src/oscal/back_matter.rs`

### Rlink (Output — OSCAL)

| Field | Type | Required | Serialization |
|-------|------|----------|---------------|
| href | String | Yes | Always serialized |
| media_type | Option\<String\> | No | `skip_serializing_if = "Option::is_none"`, rename to `media-type` |

**Location**: `src/oscal/back_matter.rs`

### ResourceCitation (Output — OSCAL)

| Field | Type | Required | Serialization |
|-------|------|----------|---------------|
| text | String | Yes | Always serialized |

**Location**: `src/oscal/back_matter.rs`

### OscalLink (Output — OSCAL, added to OscalControl)

| Field | Type | Required | Serialization |
|-------|------|----------|---------------|
| href | String | Yes | Format: `#<resource-uuid>` |
| rel | String | Yes | Always `"reference"` |
| text | Option\<String\> | No | `skip_serializing_if = "Option::is_none"` |

**Location**: `src/oscal/back_matter.rs`

### Prop (Output — OSCAL)

| Field | Type | Required | Serialization |
|-------|------|----------|---------------|
| name | String | Yes | Always serialized |
| value | String | Yes | Always serialized |

**Location**: `src/oscal/back_matter.rs`

## Functions

### generate_back_matter

```text
Input:  &[Citation]
Output: Result<(Vec<BackMatterResource>, HashMap<String, Uuid>), ForgeError>
```

- Iterates citations, classifies each as URL-based or bibliographic
- URL validation: `url::Url::parse` + scheme check (http/https only)
- Generates deterministic UUID v5 per resource using `BACK_MATTER_NAMESPACE`
- Returns resources + a resource map (citation.id → resource UUID)

### generate_control_links

```text
Input:  &[Citation], &HashMap<String, Uuid>
Output: Vec<OscalLink>
```

- For each citation, looks up its resource UUID from the map
- Creates `OscalLink { href: "#<uuid>", rel: "reference" }`
- Orphan references (citation not in map) → warning, skip link

## Relationships

| From | To | Cardinality | Relationship |
|------|----|-------------|--------------|
| BackMatter | BackMatterResource | 1:N | contains |
| BackMatterResource | Rlink | 1:N | has (URL-based) |
| BackMatterResource | ResourceCitation | 1:0..1 | has (bibliographic) |
| BackMatterResource | Prop | 1:N | has (annotations) |
| OscalControl | OscalLink | 1:N | contains |
| OscalLink | BackMatterResource | N:1 | references via href |
