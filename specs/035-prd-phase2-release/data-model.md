# Data Model: Phase 2 Integration Testing & v0.2.0 Release

**Feature**: WI-35 | **Date**: 2026-02-19

---

## Overview

**No new data models are introduced in WI-35.** This sprint is integration testing and release only. The entities below are the Phase 2 data models under test, documented here for reference by test authors.

---

## Entities Under Test

### PolicyRequirement (Phase 2 enriched)

Located in `src/model/mod.rs`.

```rust
pub struct PolicyRequirement {
    pub stable_id: Option<String>,       // Deterministic ID (v5 UUID from text+line)
    pub text: String,                    // Atomic requirement text
    pub source_line: usize,             // Source Markdown line number
    pub body_text: Option<String>,       // Section prose (if section-level)
    pub atom_index: usize,              // Position within parent compound statement
    pub parent_text: Option<String>,    // Original compound text before atomization
    pub citations: Vec<Citation>,       // Bibliographic/URL references
    pub modality: Option<Modality>,     // WI-33: normative | advisory | None
    pub parameters: Vec<PolicyParameter>, // WI-34: extracted OSCAL params
}
```

**State transitions**: `None` → `Normative | Advisory` (via modality detection pass, WI-33). Parameters extracted in separate pass (WI-34). Both enrichments happen in-memory before OSCAL serialization.

### Modality (WI-33)

```rust
pub enum Modality {
    Normative,   // "must", "shall", "will", "required"
    Advisory,    // "should", "may", "recommended"
}
```

**OSCAL mapping**: → `prop { name: "modality", value: "normative" | "advisory" }` on the corresponding OSCAL `control`.

### PolicyParameter (WI-34)

```rust
pub struct PolicyParameter {
    pub id: String,           // Deterministic parameter ID
    pub label: String,        // Human-readable label
    pub value: String,        // Extracted value (e.g., "90 days", "AES-256")
    pub class: ParameterClass, // TimeWindow | Threshold | Nominal
}
```

**OSCAL mapping**: → `param { id, label, values[value] }` on the corresponding OSCAL `control`.

---

## OSCAL Models Under Test

### OSCAL Catalog (JSON/XML/YAML)

Key fields verified in round-trip and cross-feature tests:

```json
{
  "catalog": {
    "uuid": "<uuid-v4>",
    "metadata": {
      "title": "...",
      "last-modified": "...",
      "version": "...",
      "oscal-version": "1.2.0"
    },
    "groups": [{
      "id": "<uuid-v5>",
      "title": "...",
      "controls": [{
        "id": "<uuid-v5>",
        "title": "...",
        "props": [
          { "name": "modality", "value": "normative" },   // WI-33
          { "name": "source-file", "value": "..." },
          { "name": "source-section", "value": "..." },
          { "name": "source-line", "value": "..." }
        ],
        "params": [                                        // WI-34
          { "id": "...", "label": "...", "values": ["..."] }
        ],
        "parts": [{ "name": "statement", "prose": "..." }]
      }]
    }]
  }
}
```

### OSCAL Profile (JSON/XML/YAML)

Key fields verified in profile E2E tests:

```json
{
  "profile": {
    "uuid": "<uuid-v4>",
    "metadata": { ... },
    "imports": [{
      "href": "<catalog-path>",
      "include-controls": [{ "with-ids": ["POL-AC-001", "POL-AC-002"] }]
      // OR: "exclude-controls": [{ "with-ids": ["..."] }]
    }],
    "modify": {
      "set-parameters": [{
        "param-id": "password-length",
        "values": ["16"]
      }]
    }
  }
}
```

---

## Serialization Format Notes

### XML Format (WI-26)

- Element names use OSCAL kebab-case conventions (e.g., `component-definition`, `include-controls`)
- `prop` elements serialize with `name` and `value` attributes
- `param` elements serialize with `id` and `label` attributes, child `value` elements
- Round-trip note: `control-implementations` within Component Definitions may be excluded from XML serialization (see `normalize_component_envelope` in `round_trip_test.rs`)

### YAML Format (WI-27)

- Field names identical to JSON (OSCAL convention; both use camelCase or kebab-case per spec)
- `serde_yaml_ng` handles the serialization; `serde` derive macros ensure field alignment
- Round-trip: semantic equivalence expected; deserialized `Value` comparison ignores ordering differences

---

## Validation

### OSCAL Schema (WI-19, WI-32)

- `schemas/oscal_catalog_schema.json` — validates generated Catalogs
- `schemas/oscal_component_schema.json` — validates generated Component Definitions
- `schemas/oscal_profile_schema.json` — validates generated Profiles
- Invoked via `forge validate <artifact-path>` subprocess in integration tests
