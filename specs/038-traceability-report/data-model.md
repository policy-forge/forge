# Data Model: Traceability Report (WI-38)

## Entities

### TraceMetadata

Extracted from an OSCAL element's WI-17 trace props.

| Field | Type | Source | Notes |
|-------|------|--------|-------|
| source_file | String | `PROP_SOURCE_FILE` prop value | Filename only (no path) |
| source_section | String | `PROP_SOURCE_SECTION` prop value | Section title |
| source_line | usize | `PROP_SOURCE_LINE` prop value (parsed) | 1-based line number |

**Validation**: `source_line` must parse as valid `usize` from string. If unparseable, treat element as unmapped.

### TraceEntry

A single row in the traceability report.

| Field | Type | Source | Notes |
|-------|------|--------|-------|
| element_id | String | OSCAL element `id` (controls, groups) or `control-id` (impl-reqs) | Human-readable ID |
| element_type | String | Derived from JSON structure | One of: `"group"`, `"control"`, `"implemented-requirement"` |
| trace | Option\<TraceMetadata\> | Extracted from element props | `None` → unmapped |

**Derived fields** (for display):
- `is_mapped`: `trace.is_some()`
- Source Section display: `trace.map(|t| t.source_section).unwrap_or("[unmapped]")`
- Source Line display: `trace.map(|t| t.source_line.to_string()).unwrap_or("[unmapped]")`, or `"—"` for groups with section but no line

### TraceSummary

Aggregate statistics computed from `Vec<TraceEntry>`.

| Field | Type | Computation |
|-------|------|-------------|
| total_elements | usize | `entries.len()` |
| mapped_elements | usize | `entries.iter().filter(\|e\| e.trace.is_some()).count()` |
| unmapped_elements | usize | `total_elements - mapped_elements` |
| coverage_percent | f64 | `(mapped_elements as f64 / total_elements as f64) * 100.0` (0.0 if total is 0) |

### TraceReport

The complete report structure.

| Field | Type | Source |
|-------|------|--------|
| artifact_path | PathBuf | CLI argument |
| source_path | PathBuf | CLI `--source` argument |
| artifact_type | String | Detected from JSON top-level key: `"catalog"` or `"component-definition"` |
| entries | Vec\<TraceEntry\> | Built by walker |
| summary | TraceSummary | Computed from entries |
| source_stale | bool | `true` if source mtime > OSCAL metadata.last-modified |

## Relationships

```
TraceReport 1──* TraceEntry
TraceReport 1──1 TraceSummary
TraceEntry  0..1──1 TraceMetadata
```

## State Transitions

N/A — the trace report is a read-only, stateless computation. No lifecycle or state machine.

## Data Flow

```
OSCAL Artifact (JSON file)
  │
  ├─ detect_artifact_type() → "catalog" | "component-definition"
  │
  ├─ walk_catalog_elements() ──┐
  │   or                       ├─ Vec<(element_id, element_type, &Value)>
  ├─ walk_compdef_elements() ──┘
  │
  ├─ For each element:
  │   └─ extract_trace_metadata(&Value) → Option<TraceMetadata>
  │       └─ Match props where ns == FORGE_TRACE_NS
  │
  ├─ Build Vec<TraceEntry>
  │
  ├─ Compute TraceSummary
  │
  ├─ Check source staleness (mtime vs metadata.last-modified)
  │
  └─ TraceReport
       │
       └─ format_trace_table() → String (aligned text table)
            │
            └─ stdout or --output file
```

## Input Artifacts (Read-Only)

### OSCAL Catalog JSON (envelope structure)

```json
{
  "catalog": {
    "uuid": "...",
    "metadata": {
      "title": "...",
      "last-modified": "2026-01-15T10:30:00Z",
      "version": "1.0",
      "oscal-version": "1.2.0"
    },
    "groups": [
      {
        "id": "access-control",
        "title": "Access Control",
        "props": [
          { "name": "source-section", "ns": "https://forge.policy-forge.github.io/ns/trace", "value": "Access Control" }
        ],
        "controls": [
          {
            "id": "POL-AC-001",
            "title": "...",
            "props": [
              { "name": "source-file", "ns": "https://forge.policy-forge.github.io/ns/trace", "value": "policy.md" },
              { "name": "source-section", "ns": "https://forge.policy-forge.github.io/ns/trace", "value": "Access Control" },
              { "name": "source-line", "ns": "https://forge.policy-forge.github.io/ns/trace", "value": "42" }
            ],
            "links": [
              { "href": "policy.md#line=42", "rel": "source" }
            ]
          }
        ]
      }
    ]
  }
}
```

### OSCAL Component Definition JSON (envelope structure)

```json
{
  "component-definition": {
    "uuid": "...",
    "metadata": {
      "title": "...",
      "last-modified": "2026-01-15T10:30:00Z",
      "version": "1.0",
      "oscal-version": "1.2.0"
    },
    "components": [
      {
        "uuid": "...",
        "type": "policy",
        "title": "...",
        "control-implementations": [
          {
            "uuid": "...",
            "source": "...",
            "implemented-requirements": [
              {
                "uuid": "...",
                "control-id": "POL-AC-001",
                "description": "...",
                "props": [
                  { "name": "source-file", "ns": "https://forge.policy-forge.github.io/ns/trace", "value": "policy.md" },
                  { "name": "source-section", "ns": "https://forge.policy-forge.github.io/ns/trace", "value": "Access Control" },
                  { "name": "source-line", "ns": "https://forge.policy-forge.github.io/ns/trace", "value": "42" }
                ],
                "links": [
                  { "href": "policy.md#line=42", "rel": "source" }
                ]
              }
            ]
          }
        ]
      }
    ]
  }
}
```
