# Feature Specification: OSCAL Catalog Statement Parts & Prose

**Feature Branch**: `010-catalog-statement-parts`
**Created**: 2026-02-12
**Status**: Draft
**Input**: PRD `docs/PRD/010-prd-catalog-statement-parts.md` — WI-10: OSCAL Catalog JSON Statement Parts & Prose

## Clarifications

### Session 2026-02-12

- Q: What is the source signal for guidance parts (S-1/S-2) given PolicyRequirement has no guidance/objective fields? → A: Use `PolicySection.body_text` as guidance prose for controls in that section (no domain model changes needed).
- Q: Which PolicyRequirement fields should become forge: props beyond source-line? → A: Minimal — `forge:source-line` only. `atom_index`, `parent_text`, `nesting_depth` are internal processing artifacts, not meaningful to OSCAL consumers.

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Generate Statement Parts with Prose (Priority: P1)

The Catalog builder populates each control with a statement part containing the policy requirement text, transforming structurally empty controls from WI-9 into content-bearing OSCAL controls.

> As a compliance engineer using FORGE, I want each generated OSCAL control to contain a statement part with the policy requirement prose so that the Catalog communicates actual policy requirements to downstream OSCAL consumers.

**Why this priority**: Without statement parts, the entire Catalog output is semantically empty — controls exist but say nothing about what they require. This is the core deliverable of WI-10.

**Independent Test**: Convert a test Markdown policy through the Catalog builder and verify each control has a `parts[]` array with a statement part whose `prose` matches the source `PolicyRequirement` text.

**Acceptance Scenarios**:

1. **Given** a PolicyRequirement with text "All users must use multi-factor authentication for privileged access", **When** the Catalog builder generates the control, **Then** the control has a `parts[]` array containing one entry with `name: "statement"` and `prose` equal to the requirement text. *(AC-1, traces to M-1, M-2)*
2. **Given** a control with ID `POL-AC-001`, **When** the statement part is generated, **Then** the part has `id: "POL-AC-001_smt"` following the `{control-id}_smt` convention. *(AC-2, traces to M-3)*
3. **Given** a WI-9 Catalog builder producing controls with IDs and titles, **When** parts generation is applied, **Then** controls retain their existing IDs and titles and now also have populated `parts[]` arrays. *(AC-5, traces to M-5)*
4. **Given** generated parts JSON, **When** comparing against the OSCAL v1.2.0 parts schema shape, **Then** the JSON structure matches the expected OSCAL parts format (each part has `id`, `name`, `prose` fields). *(AC-6, traces to M-6)*

---

### User Story 2 — Handle Multi-Part Controls (Priority: P2)

Controls with guidance content produce additional parts beyond the statement, preserving the full structure of the policy requirement.

> As a compliance engineer, I want controls that have guidance content to include those as separate named parts so that the OSCAL output preserves the full structure of the policy requirement.

**Why this priority**: Multi-part controls are important for complete policy representation but secondary to the core statement part generation.

**Independent Test**: Convert a test Markdown policy containing a requirement with associated guidance text, and verify the control has both a statement part and a guidance part.

**Acceptance Scenarios**:

1. **Given** a PolicySection with non-empty `body_text` containing guidance prose, **When** the Catalog builder generates controls for that section's requirements, **Then** each control has a statement part and a separate guidance part with `name: "guidance"`, `id: "{control-id}_gdn"`, and prose from `body_text`. *(AC-7, traces to S-1)*
2. *(Deferred — AC-8, traces to W-5)* Objective parts with `name: "objective"` and `id: "{control-id}_obj"` are deferred until a domain model signal for objective text exists.

---

### User Story 3 — Structured Data Uses Props Not Remarks (Priority: P1)

Structured metadata about controls is expressed using OSCAL `prop` elements, never placed in `remarks` fields, per NIST guidance.

> As a compliance engineer, I want structured policy metadata stored in OSCAL `prop` elements rather than `remarks` so that the generated output follows NIST guidance and is machine-queryable.

**Why this priority**: NIST explicitly warns against misusing `remarks` for structured data. Violating this guidance would produce non-idiomatic OSCAL that tools and auditors may reject or misinterpret.

**Independent Test**: Convert a test Markdown policy with structured metadata (e.g., source line numbers, requirement identifiers) and verify they appear as `props` elements, not in `remarks`.

**Acceptance Scenarios**:

1. **Given** a control with structured metadata (e.g., source line number 42, original requirement ID), **When** the Catalog builder generates the control, **Then** the metadata appears as `prop` entries with appropriate `name` and `value` fields (e.g., `name: "forge:source-line"`, `value: "42"`). *(AC-3, traces to M-4)*
2. **Given** a generated Catalog with multiple controls, **When** inspecting all controls, **Then** no control has arbitrary structured data stored in `remarks` fields. *(AC-4, traces to M-4)*

---

### User Story 4 — Preserve Multi-Paragraph Prose (Priority: P2)

Controls whose requirement text spans multiple paragraphs preserve paragraph breaks in the OSCAL prose field.

> As a compliance engineer, I want multi-paragraph requirement text to retain its paragraph structure in the OSCAL prose field so that the formatted output matches the original policy document.

**Why this priority**: Preserving paragraph structure ensures the OSCAL output faithfully represents the original policy text without information loss.

**Independent Test**: Convert a test policy containing a requirement with multiple paragraphs and verify the prose field preserves paragraph breaks.

**Acceptance Scenarios**:

1. **Given** a PolicyRequirement with text spanning two or more paragraphs, **When** the Catalog builder generates the statement part, **Then** the `prose` field preserves paragraph breaks from the original text. *(Traces to S-3)*

---

### Acceptance Criteria

| AC ID | Requirement | User Story | Criterion |
|-------|-------------|------------|-----------|
| AC-1 | M-1, M-2 | US-1 | Control has `parts[]` with statement entry whose `prose` matches requirement text |
| AC-2 | M-3 | US-1 | Statement part has `id: "{control-id}_smt"` |
| AC-3 | M-4 | US-3 | Control has `prop` with `name: "forge:source-line"` and correct value |
| AC-4 | M-4 | US-3 | No control stores structured data in `remarks` fields |
| AC-5 | M-5 | US-1 | Controls retain WI-9 IDs and titles and now also have populated `parts[]` |
| AC-6 | M-6 | US-1 | JSON structure matches OSCAL v1.2.0 parts format (`id`, `name`, `prose`) |
| AC-7 | S-1 | US-2 | Control has both statement and guidance part with `name: "guidance"` |
| AC-8 | S-2 | US-2 | *(Deferred)* Objective part with `name: "objective"` — deferred pending domain model signal |

### Edge Cases

- **EC-1** (M-2): When a PolicyRequirement has empty text, the statement part has an empty string prose field and a warning is emitted.
- **EC-2** (M-2): When a PolicyRequirement has only whitespace text, the statement part's prose preserves that whitespace verbatim (no trimming per SEC-1 direct copy), and a warning is emitted to indicate effectively empty content (analogous to EC-1 for whitespace-only input).
- **EC-3** (S-3): When prose contains Markdown formatting (bold, links, inline code), the formatting is preserved as-is in the prose field.
- **EC-4** (M-3): When a control ID contains special characters, the part ID is still correctly formed using the `{control-id}_smt` convention.
- **EC-5** (C-1): When a requirement has sub-items (a, b, c), sub-items are concatenated into the statement prose. Nested sub-parts with `name: "item"` are deferred to a future WI if C-1 is prioritized.
- **EC-6** (M-4): When a PolicyRequirement has a `source_line` of 0 (the domain model sentinel for "unknown line" since `source_line` is `usize`, not `Option`), no `forge:source-line` prop is generated for that control.

## Requirements *(mandatory)*

### Functional Requirements

**Must Have (MVP, launch blockers):**

- **FR-001** (M-1): Each generated OSCAL control MUST contain a `parts[]` array with at least one part having `name: "statement"`. *(Traces to: Parent PRD M-3)*
- **FR-002** (M-2): The statement part MUST have a `prose` field populated from the `PolicyRequirement.text` content. *(Traces to: Parent PRD M-3)*
- **FR-003** (M-3): Each statement part MUST have an `id` following the convention `{control-id}_smt`. *(Traces to: Parent PRD M-3)*
- **FR-004** (M-4): Structured metadata MUST be expressed as `prop` elements on controls, not stored in `remarks` fields. For WI-10, the only prop is `forge:source-line` (from `PolicyRequirement.source_line`); other domain model fields (`atom_index`, `parent_text`, `nesting_depth`) are internal artifacts and not emitted as props. *(Traces to: Parent PRD M-11)*
- **FR-005** (M-5): Parts generation MUST integrate with the existing Catalog builder from WI-9, extending controls that already have IDs and titles. *(Traces to: Parent PRD M-3)*
- **FR-006** (M-6): Generated parts MUST produce valid OSCAL v1.2.0 JSON structure for the `parts` array within controls. *(Traces to: Parent PRD M-3)*

**Should Have (high value, not blocking):**

- **FR-007** (S-1): Multi-part controls SHOULD support guidance parts with `name: "guidance"` and ID convention `{control-id}_gdn` when the parent `PolicySection.body_text` is present (non-None, non-empty).
- **FR-009** (S-3): Part generation SHOULD handle multi-paragraph prose by preserving paragraph breaks in the `prose` field.

**Could Have (nice to have, if time permits):**

- **FR-010** (C-1): Nested sub-parts within statement parts for controls that have multiple enumerated sub-requirements (e.g., `(a)`, `(b)`, `(c)` sub-items).
- **FR-011** (C-2): A `name: "item"` sub-part pattern for individual list items within a statement part.

**Won't Have (explicitly deferred):**

- **W-1**: Parameter (`param`) extraction and embedding within parts — deferred to WI-34.
- **W-2**: Back matter resource links within part prose — deferred to WI-12.
- **W-3**: Automatic classification of statement vs. guidance vs. objective from natural language analysis — deferred; initial implementation relies on document structure.
- **W-4**: Schema validation of generated parts — deferred to WI-19.
- **W-5** (formerly S-2/FR-008): Objective parts with `name: "objective"` and ID convention `{control-id}_obj` — deferred; no domain model signal exists for objective text. Revisit when domain model evolves.

### Key Entities

- **Part**: An OSCAL construct within a control representing a discrete piece of content (statement, guidance, objective). Contains `id`, `name`, and `prose` fields. May contain nested sub-parts and props.
- **Prop**: An OSCAL property element used for structured key-value data on controls or parts. Contains `name` and `value` fields. Preferred over remarks for additional data per NIST guidance.
- **Control**: An OSCAL element representing a single security requirement or policy statement. Contains parts, props, links, and parameters. Produced by the WI-9 Catalog builder with IDs and titles; extended by this WI with parts content.
- **PolicyRequirement**: Domain model entity (from WI-5) containing the source requirement text, source line information, and identifiers consumed by the parts builder.

### Assumptions

- [A-1] WI-9 produces controls with IDs and titles that this feature can extend with `parts[]` content.
- [A-2] `PolicyRequirement.text` from the domain model (WI-5) contains the raw requirement text suitable for use as OSCAL prose.
- [A-3] Guidance text is sourced from `PolicySection.body_text` (non-list-item prose between headings). Objective text has no current domain model signal and is deferred until one exists.
- [A-4] OSCAL v1.2.0 part naming conventions (`statement`, `guidance`, `objective`) are stable and well-documented.

### Dependencies

- **Requires**: WI-9 (009-catalog-groups-controls) — Catalog groups and controls structure this feature extends.
- **Requires**: WI-5 (005-domain-model) — PolicyRequirement text content consumed by this feature.
- **Parallel with**: WI-11 (OSCAL metadata), WI-12 (back matter and link patterns).
- **Blocks**: WI-13 (013-catalog-pipeline) — end-to-end Catalog pipeline.

### Scope Boundaries

**In Scope:**
- Control `parts[]` array generation with `name: "statement"` parts
- Populating `prose` fields from `PolicyRequirement` text content
- Multi-part controls: guidance parts, objective parts where applicable
- Part IDs following OSCAL conventions (e.g., `{control-id}_smt`)
- Using `props` for structured data that does not belong in prose or remarks
- Ensuring no arbitrary data is stored in OSCAL `remarks` fields

**Out of Scope:**
- Catalog group and control structure (groups[], controls[], control IDs) — completed in WI-9
- OSCAL metadata assembly (uuid, title, last-modified, version, oscal-version) — deferred to WI-11
- Back matter resources and link patterns — deferred to WI-12
- End-to-end pipeline wiring — deferred to WI-13
- Parameter extraction and `param` elements — deferred to WI-34
- Schema validation of generated output — deferred to WI-19

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of generated controls contain at least one statement part with `name: "statement"` and non-empty prose matching the source PolicyRequirement text.
- **SC-002**: 100% of statement part IDs follow the `{control-id}_smt` naming convention.
- **SC-003**: Zero instances of structured metadata (source line numbers, original identifiers) stored in `remarks` fields across all generated controls.
- **SC-004**: All generated parts produce valid OSCAL v1.2.0 JSON structure (correct field names, types, and nesting).
- **SC-005**: When guidance or objective text is available, the corresponding named parts are generated with correct IDs (`{control-id}_gdn`, `{control-id}_obj`).
- **SC-006**: Multi-paragraph requirement text retains paragraph structure in the prose field without information loss.
