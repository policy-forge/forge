# Data Model: OSCAL Catalog Statement Parts & Prose

**Branch**: `010-catalog-statement-parts` | **Date**: 2026-02-12

## Entities

### OscalPart (NEW)

An OSCAL control part representing a discrete piece of content (statement, guidance, objective).

| Field | Type | Required | Validation | Notes |
|-------|------|----------|------------|-------|
| `id` | `String` | Yes | Non-empty; follows `{control-id}_{suffix}` convention | e.g., `POL-AC-001_smt` |
| `name` | `String` | Yes | One of: `"statement"`, `"guidance"`, `"objective"`, `"item"` | OSCAL-standard part names |
| `prose` | `String` | Yes (always present; may be empty string per EC-1) | Direct copy of source text (SEC-1) | Human-readable requirement text |
| `parts` | `Vec<OscalPart>` | No | Omitted from JSON when empty | Nested sub-parts (C-1/C-2, if implemented) |
| `props` | `Vec<OscalProp>` | No | Omitted from JSON when empty | Properties on this part |

**Serde annotations**: `#[serde(skip_serializing_if = "Vec::is_empty")]` on `parts` and `props`.

### OscalProp (NEW)

An OSCAL property for structured key-value metadata on controls or parts.

| Field | Type | Required | Validation | Notes |
|-------|------|----------|------------|-------|
| `name` | `String` | Yes | Non-empty; FORGE-specific names use `forge:` prefix | e.g., `forge:source-line` |
| `value` | `String` | Yes | Non-empty | e.g., `"42"` |

### OscalControl (MODIFIED — extends WI-9)

Two new fields added to the existing struct.

| Field | Type | Required | Validation | Notes |
|-------|------|----------|------------|-------|
| `id` | `String` | Yes | *Existing (WI-9)* | e.g., `POL-AC-001` |
| `uuid` | `String` | Yes | *Existing (WI-9)* | From `PolicyRequirement.stable_id` |
| `title` | `String` | Yes | *Existing (WI-9)* | Derived from requirement text |
| `parts` | `Vec<OscalPart>` | Yes | Must contain at least one part with `name: "statement"` (FR-001) | **NEW in WI-10** |
| `props` | `Vec<OscalProp>` | No | Omitted from JSON when empty | **NEW in WI-10** |

**Serde annotations**: `parts` is NOT skip-serialized (always present). `props` uses `#[serde(skip_serializing_if = "Vec::is_empty")]`.

## Relationships

```text
OscalControl 1──* OscalPart    (parts[])     — every control has ≥1 part (specifically ≥1 statement part per FR-001)
OscalControl 1──* OscalProp    (props[])     — 0 or more metadata props
OscalPart    1──* OscalPart    (sub-parts[]) — optional nesting (C-1)
OscalPart    1──* OscalProp    (props[])     — optional props on parts
```

## Source Mappings

| OSCAL Output | Source Field | Transformation |
|-------------|-------------|----------------|
| `OscalPart.prose` (statement) | `PolicyRequirement.text` | Direct copy (SEC-1) |
| `OscalPart.prose` (guidance) | `PolicySection.body_text` | Direct copy when present and non-empty |
| `OscalPart.prose` (objective) | *(deferred — W-5)* | No domain model signal exists; deferred to future WI |
| `OscalPart.prose` (item) | *(deferred — C-1/C-2)* | Nested sub-parts for enumerated sub-items; deferred |
| `OscalPart.id` | `OscalControl.id` + suffix | `format!("{}_{}", control_id, suffix)` |
| `OscalPart.name` | Determined by part type | `"statement"`, `"guidance"`, `"objective"` (deferred), `"item"` (deferred) |
| `OscalProp.name` | Constant | `"forge:source-line"` |
| `OscalProp.value` | `PolicyRequirement.source_line` | `source_line.to_string()` (only when > 0) |

## Part ID Suffix Convention

| Part Type | Suffix | Example |
|-----------|--------|---------|
| Statement | `_smt` | `POL-AC-001_smt` |
| Guidance | `_gdn` | `POL-AC-001_gdn` |
| Objective | `_obj` | `POL-AC-001_obj` (deferred) |
| Item (sub-part) | `_smt.a` | `POL-AC-001_smt.a` (C-1, if implemented) |
