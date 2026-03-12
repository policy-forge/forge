# Data Model: 043-diff-report

**Date**: 2026-03-12 | **Branch**: `043-diff-report`

## Module: `src/diff/types.rs`

Types are primarily data structs/enums with small helper methods (e.g., accessors, `Display`). Core comparison logic lives in `engine.rs`.

---

### `ArtifactType`

Detected from the root key of the loaded OSCAL JSON.

```rust
pub enum ArtifactType {
    Catalog,              // root key: "catalog"
    ComponentDefinition,  // root key: "component-definition"
}
```

**Detection rule**: Check `json["catalog"]` first; if present → `Catalog`. Check `json["component-definition"]`; if present → `ComponentDefinition`. If neither → `DiffError("not a recognized OSCAL artifact")`.

---

### `ControlSnapshot`

The set of diffable fields captured from a single control or implemented-requirement during extraction. This is the unit of comparison.

```rust
pub struct ControlSnapshot {
    /// The stable human-assigned identifier (e.g., "POL-AC-001").
    /// Primary matching key across versions.
    pub control_id: String,

    /// UUID of this control/implemented-requirement.
    /// For Catalog artifacts from FORGE: always empty string (uuid is skip_serializing).
    /// For Component Definition artifacts: populated from ir["uuid"].
    pub uuid: String,

    /// Control title (from control["title"] in Catalog; None for Component Definition).
    pub title: Option<String>,

    /// Implementation narrative for Component Definition implemented-requirements
    /// (from ir["description"]). None for Catalog controls.
    /// FieldChange.field_name = "description" when this field differs.
    pub description: Option<String>,

    /// Statement prose for Catalog controls (from control["parts"][statement]["prose"]).
    /// Empty for Component Definition (which uses `description` instead).
    /// FieldChange.field_name = "statement[N]" for index N when a prose entry differs.
    pub parts_prose: Vec<String>,
}
```

**Notes**:
- `title` is `None` for Component Definition snapshots (implemented-requirements have no title field).
- `description` is `Some(narrative)` for Component Definition snapshots; `None` for Catalog.
- `parts_prose` for Catalog captures all statement-part prose strings in order; empty for Component Definition.
- Field labels in `FieldChange.field_name`: `"title"` | `"description"` | `"statement[N]"`.

---

### `FieldChange`

A single field-level difference within a changed control.

```rust
pub struct FieldChange {
    /// Field name as displayed in the report (e.g., "title", "description", "statement[0]").
    pub field_name: String,

    /// Previous value.
    pub old_value: String,

    /// Updated value.
    pub new_value: String,
}
```

---

### `DiffEntry`

A single comparison result for one control-id.

```rust
pub enum DiffEntry {
    /// Control present in new but absent from old.
    Added {
        control_id: String,
        new_uuid: String,
    },

    /// Control present in old but absent from new.
    Removed {
        control_id: String,
        old_uuid: String,
    },

    /// Control present in both with different content.
    /// When uuid_changed=true AND field_changes is non-empty, UUID and field changes co-occur
    /// on the same entry (no separate UuidChanged entry is emitted).
    Changed {
        control_id: String,
        old_uuid: String,
        new_uuid: String,
        /// True when the UUID changed (same control-id, different UUID) AND field values also changed.
        uuid_changed: bool,
        field_changes: Vec<FieldChange>,
    },

    /// UUID changed but all diffable field values are identical.
    /// Only emitted when uuid differs AND field_changes would be empty.
    UuidChanged {
        control_id: String,
        old_uuid: String,
        new_uuid: String,
    },
}
```

**Classification rules (from spec clarification 2026-03-12)**:
- UUID differs AND fields differ → `Changed { uuid_changed: true, field_changes: [...] }`
- UUID differs AND fields same → `UuidChanged { ... }`
- UUID same AND fields differ → `Changed { uuid_changed: false, field_changes: [...] }`
- UUID same AND fields same → Unchanged (not stored in entries)

---

### `DiffSummary`

Aggregate counts derived from the entries list.

```rust
pub struct DiffSummary {
    pub total_old: usize,
    pub total_new: usize,
    pub added: usize,
    pub removed: usize,
    pub changed: usize,    // includes Changed{uuid_changed: true/false}
    pub unchanged: usize,
    pub uuid_changes: usize, // count of UuidChanged entries only
}
```

**Count semantics**:
- `uuid_changes` = count of `DiffEntry::UuidChanged` entries only (NOT `Changed{uuid_changed:true}`)
- `changed` = count of `DiffEntry::Changed` entries (whether or not uuid_changed)
- `total_old` + `total_new` = counts from respective HashMaps before comparison

---

### `DiffReport`

The complete result of comparing two artifacts.

```rust
pub struct DiffReport {
    /// Path to the old artifact file.
    pub old_file: String,

    /// Path to the new artifact file.
    pub new_file: String,

    /// Detected artifact type (both files must be the same type).
    pub artifact_type: ArtifactType,

    /// Categorized diff entries, sorted by control_id ascending.
    pub entries: Vec<DiffEntry>,

    /// Aggregate counts.
    pub summary: DiffSummary,
}
```

---

## Entity Relationships

```
DiffReport
├── old_file: String
├── new_file: String
├── artifact_type: ArtifactType
├── summary: DiffSummary (derived from entries)
└── entries: Vec<DiffEntry>  (sorted by control_id)
    ├── Added       { control_id, new_uuid }
    ├── Removed     { control_id, old_uuid }
    ├── Changed     { control_id, old_uuid, new_uuid, uuid_changed, field_changes }
    │   └── FieldChange { field_name, old_value, new_value }  (1..n)
    └── UuidChanged { control_id, old_uuid, new_uuid }
```

```
(Internal only — not in DiffReport)
ControlSnapshot { control_id, uuid, title, description, parts_prose }
  → used during extraction + comparison
  → not stored in final DiffReport
```

---

## Validation Rules

| Entity | Rule |
|--------|------|
| `ControlSnapshot.control_id` | Must be non-empty; used as HashMap key |
| `DiffReport.entries` | Sorted by `control_id` (ascending, lexicographic) |
| `DiffSummary.total_old` | `= added + changed + unchanged + removed` (from old perspective) |
| `DiffSummary.total_new` | `= added + changed + unchanged` (from new perspective) |
| `DiffEntry::Changed.field_changes` | Non-empty (if empty, classify as `UuidChanged`) |
| Both input files | Must be same `ArtifactType` |

---

## Extraction Paths

### Catalog (`"catalog"` root key)

```
catalog.groups[].controls[]           → extract OscalControl-shaped objects
catalog.groups[].groups[].controls[]  → recursive (arbitrary depth)

Per control:
  control_id  = json["id"].as_str()
  uuid        = json["uuid"].as_str().unwrap_or("")   // usually empty in FORGE output
  title       = json["title"].as_str()
  parts_prose = json["parts"][*]["prose"] where json["parts"][*]["name"] == "statement"
```

### Component Definition (`"component-definition"` root key)

```
component-definition.components[*].control-implementations[*].implemented-requirements[*]

Per implemented-requirement:
  control_id  = ir["control-id"].as_str()
  uuid        = ir["uuid"].as_str().unwrap_or("")     // present in FORGE output
  title       = None
  description = Some(ir["description"].as_str().unwrap_or(""))  // FieldChange label: "description"
  parts_prose = []                                               // unused for ComponentDefinition
```
