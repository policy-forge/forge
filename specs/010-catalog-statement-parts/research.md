# Research: OSCAL Catalog Statement Parts & Prose

**Branch**: `010-catalog-statement-parts` | **Date**: 2026-02-12

## Overview

No NEEDS CLARIFICATION items in the Technical Context. All dependencies are existing crates, the OSCAL parts structure is well-documented, and the integration point (WI-9 `build_catalog`) is clear from the codebase.

Research focused on three areas: (1) confirming OSCAL v1.2.0 parts JSON structure, (2) mapping current codebase integration points, and (3) validating the guidance text sourcing decision.

---

## R-1: OSCAL v1.2.0 Parts JSON Structure

**Decision**: Parts use `id`, `name`, `prose` as top-level string fields. Nested sub-parts and props are optional arrays. Field names are lowercase, no hyphens.

**Rationale**: Confirmed from OSCAL Catalog JSON schema and NIST SP 800-53 examples. The parts structure within a control is:

```json
{
  "id": "POL-AC-001",
  "uuid": "...",
  "title": "...",
  "parts": [
    {
      "id": "POL-AC-001_smt",
      "name": "statement",
      "prose": "Requirement text here."
    },
    {
      "id": "POL-AC-001_gdn",
      "name": "guidance",
      "prose": "Guidance text here."
    }
  ],
  "props": [
    {
      "name": "forge:source-line",
      "value": "42"
    }
  ]
}
```

**Alternatives considered**: None — OSCAL v1.2.0 structure is prescriptive.

**Serde implications**:
- `OscalPart.parts` and `OscalPart.props` use `#[serde(skip_serializing_if = "Vec::is_empty")]` to omit when empty (matching OSCAL convention of omitting empty arrays).
- `OscalControl.parts` is NOT skip-serialized (every control MUST have parts per FR-001).
- `OscalControl.props` uses `#[serde(skip_serializing_if = "Vec::is_empty")]` (not all controls have props, e.g., when source_line is 0).
- No field renaming needed — `id`, `name`, `prose`, `parts`, `props` are already the correct OSCAL JSON field names.

---

## R-2: WI-9 Catalog Builder Integration Points

**Decision**: Extend `OscalControl` in `src/oscal/catalog.rs` with `parts: Vec<OscalPart>` and `props: Vec<OscalProp>` fields. Call `build_control_parts` and `build_control_props` inside the `build_catalog` function's requirement loop.

**Rationale**: The current `build_catalog` function (catalog.rs:266) iterates sections and requirements. It already has access to:
- `section` — provides `body_text` for guidance parts
- `req` — the `PolicyRequirement` with `text` and `source_line`
- `control_id` — generated via `generate_control_id`

The integration requires:
1. Pass `section.body_text.as_deref()` to `build_control_parts` for guidance
2. Call `build_control_props(req)` for props
3. Set `parts` and `props` on the `OscalControl` struct

**Current OscalControl shape** (catalog.rs:53-60):
```rust
pub struct OscalControl {
    pub id: String,
    pub uuid: String,
    pub title: String,
    // WI-10 adds:
    // pub parts: Vec<OscalPart>,
    // pub props: Vec<OscalProp>,
}
```

**Alternatives considered**: Inline parts generation directly in `build_catalog` (rejected — AR Option 2 was not selected due to testability concerns).

---

## R-3: Guidance Text Sourcing from PolicySection.body_text

**Decision**: Use `PolicySection.body_text` as guidance prose for all controls within that section. When `body_text` is `Some(text)` and `text` is non-empty, generate a guidance part.

**Rationale**: Clarification session confirmed this approach. `PolicySection.body_text` (model/mod.rs:90) is `Option<String>` containing non-list-item prose between headings. This naturally represents explanatory/guidance content that accompanies the normative requirements in list items.

**Behavior**:
- `body_text` is `None` or `Some("")` → no guidance part generated
- `body_text` is `Some("non-empty text")` → guidance part with `name: "guidance"`, `id: "{control-id}_gdn"`, `prose: body_text`
- All requirements in the section share the same guidance text (the section's body text)

**Alternatives considered**: Add `guidance_text` field to `PolicyRequirement` (rejected — requires domain model changes not in scope). Defer guidance entirely (rejected — clarification chose Option A).

---

## R-4: Props Scope Decision

**Decision**: Only `forge:source-line` is emitted as a prop. Other `PolicyRequirement` fields (`atom_index`, `parent_text`, `nesting_depth`) are internal processing artifacts and not emitted.

**Rationale**: Clarification session confirmed minimal approach. Source line is meaningful to external OSCAL consumers for traceability. Other fields are internal to FORGE's processing pipeline.

**Behavior**:
- `source_line > 0` → `OscalProp { name: "forge:source-line", value: source_line.to_string() }`
- `source_line == 0` → no prop generated (EC-6)

**Alternatives considered**: Emit all metadata fields as props (rejected — internal artifacts not meaningful to OSCAL consumers).
