# Data Model: Assessment Plan Scaffolding — Controls (WI-41)

**Feature Branch**: `041-assessment-plan-controls`
**Phase**: 1 — Design
**Date**: 2026-03-12

---

## Overview

The Assessment Plan builder introduces a single new Rust module (`src/oscal/assessment_plan.rs`)
with typed structs for the OSCAL Assessment Plan JSON structure. All structs use `serde`
with `#[serde(rename)]` annotations to produce OSCAL-compliant hyphenated JSON keys.

No new database tables, no new file formats, no new external services.

---

## Rust Structs

### `AssessmentPlanEnvelope`

Top-level JSON envelope. Serializes to `{"assessment-plan": {...}}`.

```rust
#[derive(Debug, Clone, Serialize)]
pub struct AssessmentPlanEnvelope {
    #[serde(rename = "assessment-plan")]
    pub assessment_plan: AssessmentPlan,
}
```

**Relationships**: Contains exactly one `AssessmentPlan`.

---

### `AssessmentPlan`

OSCAL Assessment Plan root object.

```rust
#[derive(Debug, Clone, Serialize)]
pub struct AssessmentPlan {
    pub uuid: String,                        // UUID v5, deterministic from (control_ids + ssp_href)
    pub metadata: ApMetadata,                // From assemble_metadata (WI-11)
    #[serde(rename = "import-ssp")]
    pub import_ssp: ImportSsp,               // From --import-ssp CLI flag
    #[serde(rename = "reviewed-controls")]
    pub reviewed_controls: ReviewedControls, // Populated from conversion output
}
```

**Validation rules**:
- `uuid`: non-empty, valid UUID v5
- `metadata`: populated via `assemble_metadata()` shared function
- `import_ssp.href`: non-empty (validated before building)
- `reviewed_controls.control_selections`: non-empty when controls exist; may be empty with warning

---

### `ApMetadata`

OSCAL metadata for the Assessment Plan. A subset of `OscalMetadata` re-serialized with
AP-specific field values.

```rust
#[derive(Debug, Clone, Serialize)]
pub struct ApMetadata {
    pub title: String,                // "Assessment Plan for {policy_title}"
    #[serde(rename = "last-modified")]
    pub last_modified: String,        // ISO 8601 UTC timestamp
    pub version: String,              // "1.0.0" (static)
    #[serde(rename = "oscal-version")]
    pub oscal_version: String,        // "1.2.0" (OSCAL_VERSION constant)
}
```

**Note**: Uses the existing `assemble_metadata()` function internally to construct values,
then maps to `ApMetadata` for serialization with correct field names.

---

### `ImportSsp`

Reference to the System Security Plan being assessed. Href is passed through verbatim.

```rust
#[derive(Debug, Clone, Serialize)]
pub struct ImportSsp {
    pub href: String,  // From --import-ssp CLI flag; non-empty validated before use
}
```

**Validation rules**:
- `href`: must be non-empty (checked in `build_assessment_plan`)

---

### `ReviewedControls`

Container defining the assessment scope. Wraps one `ApControlSelection` covering all controls.

```rust
#[derive(Debug, Clone, Serialize)]
pub struct ReviewedControls {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,               // "Controls derived from {policy_title} for assessment review."
    #[serde(rename = "control-selections")]
    pub control_selections: Vec<ApControlSelection>,
}
```

**Validation rules**:
- `control_selections`: always one entry (single selection group per WI-41 scope)
- `description`: always populated (FR-009 SHOULD → implemented as optional field, always set)

---

### `ApControlSelection`

A single control-selection group containing all include-controls.

```rust
#[derive(Debug, Clone, Serialize)]
pub struct ApControlSelection {
    #[serde(rename = "include-controls")]
    pub include_controls: Vec<ApIncludeControl>,
}
```

**Validation rules**:
- `include_controls`: may be empty (zero-controls edge case); deduplicated + sorted before building

---

### `ApIncludeControl`

A single control identifier entry within a control-selection group.

```rust
#[derive(Debug, Clone, Serialize)]
pub struct ApIncludeControl {
    #[serde(rename = "control-id")]
    pub control_id: String,  // e.g., "POL-AC-001"
}
```

**Validation rules**:
- `control_id`: non-empty, preserved exactly as received from pipeline (no transformation)

---

## Entity Relationships

```
AssessmentPlanEnvelope
└── AssessmentPlan (1)
    ├── ApMetadata (1)          ← from assemble_metadata (WI-11)
    ├── ImportSsp (1)           ← href from --import-ssp CLI flag
    └── ReviewedControls (1)
        └── ApControlSelection (1..*)  ← WI-41 produces exactly 1
            └── ApIncludeControl (0..*)  ← 0 allowed with warning
```

---

## UUID v5 Derivation

New namespace constant added to `src/uuid.rs`:

```rust
/// Fixed namespace UUID for Assessment Plan identifier generation.
///
/// Derived from `Uuid::new_v5(&FORGE_NAMESPACE_UUID, b"assessment-plan")`.
pub const ASSESSMENT_PLAN_NAMESPACE: Uuid = Uuid::from_bytes([...]);
```

| Identifier | Seed Input | Namespace |
|------------|------------|-----------|
| `assessment-plan.uuid` | `format!("assessment-plan\|{}\|{}", sorted_ids.join(","), ssp_href)` | `FORGE_NAMESPACE_UUID` |
| `control-selections[0]` UUID (if needed) | Same seed as document UUID | `ASSESSMENT_PLAN_NAMESPACE` |

**Determinism guarantee**: Same `control_ids` (sorted) + same `import_ssp_href` → same UUIDs
across all runs (FR-008, SC-003).

---

## Output File Naming

AP output path is always JSON (`.json`), regardless of the primary output `--format` flag.

| Scenario | AP Output Path |
|----------|----------------|
| `forge convert policy.md --strategy catalog --import-ssp ./ssp.json` | `./policy-assessment-plan.json` |
| `forge convert policy.md --strategy catalog --output ./out/catalog.json --import-ssp ./ssp.json` | `./out/policy-assessment-plan.json` |
| Batch mode (2+ inputs) | AP generation skipped; warning emitted |

**Implementation**: `fn derive_ap_output_path(input: &Path, primary_output: Option<&Path>) -> PathBuf`

---

## Control ID Collection Helpers

Two pure functions added to existing modules:

```rust
// src/oscal/assessment_plan.rs (or catalog.rs / component_definition.rs)

/// Collect all control IDs from a built OSCAL Catalog.
/// Returns IDs in declaration order (groups → controls, depth-first).
pub fn collect_control_ids_from_catalog(catalog: &OscalCatalog) -> Vec<String>;

/// Collect all control IDs from a built Component Definition.
/// Returns IDs from all implemented-requirements across all components.
pub fn collect_control_ids_from_component_def(envelope: &ComponentDefinitionEnvelope) -> Vec<String>;
```

Both return `Vec<String>` — deduplication and sorting are performed inside `build_assessment_plan`.

---

## Serialized JSON Example

```json
{
  "assessment-plan": {
    "uuid": "<deterministic-uuid-v5>",
    "metadata": {
      "title": "Assessment Plan for Corporate Security Policy",
      "last-modified": "2026-03-12T00:00:00Z",
      "version": "1.0.0",
      "oscal-version": "1.2.0"
    },
    "import-ssp": {
      "href": "./ssp/system-ssp.json"
    },
    "reviewed-controls": {
      "description": "Controls derived from Corporate Security Policy for assessment review.",
      "control-selections": [
        {
          "include-controls": [
            { "control-id": "POL-AC-001" },
            { "control-id": "POL-AC-002" },
            { "control-id": "POL-DP-001" }
          ]
        }
      ]
    }
  }
}
```
