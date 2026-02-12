# Feature Specification: OSCAL Catalog Groups and Controls

**Feature Branch**: `009-catalog-groups-controls`
**Created**: 2026-02-11
**Status**: Draft
**Input**: PRD `docs/PRD/009-prd-catalog-groups-controls.md` (WI-9)
**Depends On**: WI-5 (Domain Model), WI-7 (UUID Generation), WI-8 (Citation Extraction)

---

## Clarifications

### Session 2026-02-11

- Q: Which collision resolution strategy for section abbreviation collisions — numeric suffix, longer abbreviation, or hybrid? → A: Numeric suffix (first section keeps base abbreviation, subsequent collisions get `AC2`, `AC3`, etc.)
- Q: How should control titles be derived from requirement text — what strategy and character cap? → A: First sentence (up to first `.`/`!`/`?`), capped at 120 characters with `...` suffix if exceeded.
- Q: Should the MVP handle nested sections as recursive OSCAL groups or flatten them? → A: Flat mapping only — top-level sections become groups; child sections' requirements fold into the parent group's controls.

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Generate Catalog Groups from Policy Sections (Priority: P1)

The system converts policy document sections into OSCAL Catalog groups, preserving the organizational structure of the source document. Each policy section becomes a named, identifiable group in the output.

> As a developer working on FORGE, I want policy sections to map to OSCAL Catalog groups so that the generated Catalog reflects the organizational structure of the source policy document.

**Why this priority**: Groups are the top-level organizational structure in a Catalog. Without groups, controls have no container and the Catalog has no hierarchy. This is the foundational structure that all other Catalog elements attach to.

**Independent Test**: Provide a policy document with 3 sections and invoke the Catalog builder; verify the output contains 3 groups with correct titles and identifiers.

**Acceptance Scenarios**:

1. **Given** a policy document with sections "Access Control", "Data Protection", and "Incident Response", **When** the Catalog builder runs, **Then** the output contains 3 groups with identifiers derived from section titles (e.g., `access-control`, `data-protection`, `incident-response`) and matching titles.
2. **Given** a policy section with nested child sections, **When** the Catalog builder runs, **Then** the hierarchy is flattened: only the top-level section becomes a group, and child sections' requirements are included in the parent group's controls.

---

### User Story 2 — Generate Catalog Controls from Policy Requirements (Priority: P1)

Each policy requirement is mapped to an OSCAL control within its parent group, making every individual requirement addressable as a discrete control in the Catalog.

> As a developer working on FORGE, I want policy requirements to map to OSCAL controls so that each individual requirement is addressable as a discrete control in the Catalog.

**Why this priority**: Controls are the fundamental addressable unit in OSCAL. Every downstream operation (profiling, component mapping, assessment) operates on controls. Without controls, the Catalog has structure but no content.

**Independent Test**: Provide a policy document with 2 sections containing 3 and 4 requirements respectively, invoke the Catalog builder, and verify 7 controls are distributed across 2 groups with correct identifiers.

**Acceptance Scenarios**:

1. **Given** a policy document with a section "Access Control" containing 3 requirements, **When** the Catalog builder runs, **Then** the "access-control" group contains 3 controls with identifiers `POL-AC-001`, `POL-AC-002`, `POL-AC-003`.
2. **Given** a policy requirement with a stable identifier from the UUID generation stage, **When** mapped to a control, **Then** the control's UUID matches the requirement's stable identifier.

---

### User Story 3 — Generate Human-Readable Control IDs from Section Context (Priority: P1)

Control identifiers follow a deterministic naming pattern derived from section and requirement context, making controls human-readable and traceable to their source policy section.

> As a compliance engineer, I want control IDs to follow a meaningful naming convention (e.g., `POL-AC-001`) so that controls are human-readable and traceable to their source policy section.

**Why this priority**: Human-readable control IDs are essential for usability. They enable compliance engineers to quickly identify which policy section a control belongs to and its position within that section.

**Independent Test**: Provide a policy document with a section titled "Access Control" and 2 requirements, verify the generated control IDs are `POL-AC-001` and `POL-AC-002`.

**Acceptance Scenarios**:

1. **Given** a section titled "Access Control" with 3 requirements, **When** generating control IDs, **Then** IDs are `POL-AC-001`, `POL-AC-002`, `POL-AC-003`.
2. **Given** a section titled "Incident Response and Recovery" with 1 requirement, **When** generating control IDs, **Then** the ID follows the pattern (e.g., `POL-IRR-001`) using an abbreviation derived from the section title.
3. **Given** two sections with potentially colliding abbreviations, **When** generating control IDs, **Then** IDs remain unique across the entire Catalog.

---

### User Story 4 — Serialize Catalog to JSON (Priority: P1)

The assembled Catalog structure is serialized to valid JSON output conforming to OSCAL conventions, producing the first machine-readable OSCAL artifact from the pipeline.

> As a developer working on FORGE, I want the Catalog to be serialized as valid JSON so that the output is machine-readable and consumable by OSCAL-compatible tools.

**Why this priority**: JSON serialization is what makes the Catalog usable. Without it, the internal representation has no external value. This is the bridge from internal processing to OSCAL output.

**Independent Test**: Build a complete Catalog structure, serialize it, and verify the output is valid JSON with correct field names matching OSCAL conventions.

**Acceptance Scenarios**:

1. **Given** a complete Catalog structure with groups and controls, **When** serialized to JSON, **Then** the output is valid, parseable JSON with correct field names (`groups`, `controls`, `id`, `title`, `uuid`).
2. **Given** a serialized Catalog, **When** parsed back into a data structure, **Then** all field values match the original structure (round-trip integrity).

---

### Edge Cases

- **EC-1** (M-1): When a policy document has zero sections, then the Catalog's groups collection is empty (valid Catalog with no groups).
- **EC-2** (M-3): When a section has zero requirements, then the group has an empty controls collection.
- **EC-3** (M-4): When two sections have titles that produce the same abbreviation (e.g., "Access Control" and "Application Configuration" both yielding "AC"), then the first section retains `AC` and subsequent collisions receive a numeric suffix (`AC2`, `AC3`, etc.), ensuring all control IDs remain unique.
- **EC-4** (M-2): When a section title contains special characters or non-ASCII text, then the group ID is properly normalized to contain only lowercase alphanumeric characters and hyphens.
- **EC-5** (M-6): When a requirement's stable identifier is missing (UUID generation not yet run), then the builder reports an error identifying the affected requirement.
- **EC-6** (M-4): When a section has more than 999 requirements, then control IDs extend beyond 3 digits (e.g., `POL-AC-1000`).
- **EC-7** (M-5): When the first sentence of requirement text exceeds 120 characters, then the control title is truncated to 120 characters with a `...` suffix appended.

---

## Requirements *(mandatory)*

### Functional Requirements

#### Must Have (M) — MVP, launch blockers

- **M-1**: The Catalog builder SHALL map each policy section in the domain model to an OSCAL group, preserving section order. *(Traces to: Parent PRD M-3)*
- **M-2**: Each OSCAL group SHALL have an identifier derived from the section title (e.g., `access-control` from "Access Control Policies") and a title matching the section title. *(Traces to: Parent PRD M-3)*
- **M-3**: The Catalog builder SHALL map each policy requirement within a section to an OSCAL control in the parent group's controls collection, preserving requirement order. *(Traces to: Parent PRD M-3)*
- **M-4**: Each OSCAL control SHALL have a human-readable identifier generated from section context and requirement index (e.g., `POL-AC-001`). *(Traces to: Parent PRD M-3)*
- **M-5**: Each OSCAL control SHALL have a title derived from the requirement text: extract the first sentence (up to the first `.`, `!`, or `?`), capped at 120 characters with a `...` suffix if the first sentence exceeds that limit. *(Traces to: Parent PRD M-3)*
- **M-6**: Each OSCAL control SHALL include a UUID sourced from the requirement's stable identifier field (populated by WI-7). *(Traces to: Parent PRD M-8)*
- **M-7**: The Catalog builder SHALL serialize the assembled Catalog structure to valid JSON output. *(Traces to: Parent PRD M-3, M-7)*
- **M-8**: Control IDs SHALL be unique across the entire generated Catalog. *(Traces to: Parent PRD M-3)*

#### Should Have (S) — High value, not blocking

- **S-1**: Group IDs SHALL be unique across the Catalog; the builder SHALL detect and resolve collisions by appending a numeric suffix.
- **S-2**: The Catalog builder SHALL handle nested policy section children by flattening: only top-level sections become groups, and child sections' requirements are included in the parent group's controls. Recursive nested OSCAL group generation is deferred to a future work item.
- **S-3**: The control ID abbreviation (e.g., `AC` from "Access Control") SHALL be derived via a deterministic abbreviation algorithm that produces stable results across runs. On abbreviation collision, the first section retains the base abbreviation and subsequent collisions receive a numeric suffix (e.g., `AC`, `AC2`, `AC3`).

#### Could Have (C) — Nice to have, if time permits

- **C-1**: The Catalog builder COULD accept an optional prefix parameter (default: `POL`) for control IDs, allowing customization (e.g., `SEC-AC-001` instead of `POL-AC-001`).
- **C-2**: The Catalog builder COULD emit warnings for sections with zero requirements (empty groups).

#### Won't Have (W) — Explicitly deferred

- **W-1**: Control statement parts and prose — *Deferred to WI-10 (statement parts)*
- **W-2**: OSCAL metadata assembly — *Deferred to WI-11 (metadata)*
- **W-3**: Back matter resources — *Deferred to WI-12 (back matter)*
- **W-4**: OSCAL JSON schema validation — *Deferred to WI-19+*
- **W-5**: CLI integration (`--strategy catalog --format json`) — *Deferred to WI-13 (end-to-end pipeline wiring)*
- **W-6**: XML or YAML output — *Deferred to later work items*

### Acceptance Criteria

| AC ID | Requirement | User Story | Given | When | Then |
|-------|-------------|------------|-------|------|------|
| AC-1  | M-1, M-2   | US-1       | A policy document with 3 sections ("Access Control", "Data Protection", "Incident Response") | Calling the Catalog builder | Output groups collection has 3 groups with IDs `access-control`, `data-protection`, `incident-response` and matching titles |
| AC-2  | M-3, M-4   | US-2, US-3 | A section "Access Control" with 3 requirements | Calling the Catalog builder | The `access-control` group has 3 controls with IDs `POL-AC-001`, `POL-AC-002`, `POL-AC-003` |
| AC-3  | M-5        | US-2       | A policy requirement with text "Systems shall require MFA for all privileged access" | Calling the Catalog builder | The control's title is derived from the requirement text |
| AC-4  | M-6        | US-2       | A policy requirement with a stable identifier `a1b2c3d4-...` | Calling the Catalog builder | The control's UUID equals `a1b2c3d4-...` |
| AC-5  | M-7        | US-4       | A complete Catalog structure | Serializing to JSON | Valid JSON is produced with correct field names (`groups`, `controls`, `id`, `title`, `uuid`) |
| AC-6  | M-8        | US-3       | A policy document with 5 sections and 20 total requirements | Calling the Catalog builder | All 20 control IDs are unique |

### Key Entities

- **OSCAL Catalog**: The root container representing a structured collection of controls. Contains groups organized hierarchically with placeholder metadata (populated by WI-11).
- **OSCAL Group**: An organizational container within a Catalog that groups related controls together. Each group has a human-readable identifier and title derived from the source policy section.
- **OSCAL Control**: The fundamental addressable unit representing a single security requirement or policy statement. Each control has a human-readable identifier (e.g., `POL-AC-001`), a stable UUID, and a title derived from the requirement text.
- **Control ID**: A human-readable identifier following the pattern `POL-{ABBR}-{NNN}`, where `POL` is a configurable prefix, `ABBR` is derived from the section title, and `NNN` is a zero-padded sequential index within the section.
- **Group ID**: A slug-form identifier derived from the section title (e.g., `access-control` from "Access Control").

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of policy sections in the input are represented as groups in the output, with no sections dropped or duplicated.
- **SC-002**: 100% of policy requirements in the input are represented as controls in the output, with no requirements dropped or duplicated.
- **SC-003**: Zero duplicate control IDs exist across the entire generated Catalog output.
- **SC-004**: The output is valid, parseable JSON that can be round-tripped (serialized and deserialized) without data loss.
- **SC-005**: Control IDs conform 100% to the `POL-{ABBR}-{NNN}` naming pattern across all generated controls.
- **SC-006**: Group identifiers and control identifiers remain stable and deterministic across repeated runs on the same input.

---

## Scope Boundaries

**In Scope:**
- Mapping policy sections to OSCAL groups
- Mapping policy requirements to OSCAL controls within their parent groups
- Generating human-readable control IDs from section context and requirement index
- Generating group IDs from section titles
- Serializing the Catalog structure to JSON
- Handling control ID and group ID collisions

**Out of Scope:**
- Control statement parts and prose (WI-10)
- OSCAL metadata assembly (WI-11)
- Back matter resources (WI-12)
- OSCAL JSON schema validation (WI-19+)
- End-to-end CLI wiring (WI-13)
- Component Definition generation (WI-14)
- XML or YAML output formats (later work items)

---

## Dependencies

- **Requires**: WI-5 (Domain Model — provides PolicyDocument, PolicySection, PolicyRequirement structures), WI-7 (UUID Generation — provides stable identifiers for controls), WI-8 (Citation Extraction — citation model available in domain model)
- **Blocks**: WI-10 (Statement Parts — adds control parts/prose to controls built here), WI-14 (Component Definition — uses Catalog structure as reference)
- **Parallel With**: WI-11 (OSCAL Metadata), WI-12 (Back Matter)

---

## Assumptions

- [A-1] The policy document is fully assembled by upstream work items (WI-5 through WI-8) before the Catalog builder is invoked.
- [A-2] Each policy requirement has a stable identifier populated by WI-7 (UUID generation) that becomes the control's UUID.
- [A-3] Section titles are sufficiently descriptive to derive meaningful group IDs and control ID abbreviations.
- [A-4] The OSCAL Catalog JSON structure follows v1.2.0 conventions (groups contain controls, each with `id` and `title`).
- [A-5] At this stage, controls are skeletal — they have `id`, `title`, and `uuid` but no statement parts (added in WI-10), no metadata (WI-11), and no back matter links (WI-12).

---

## Risks

| ID   | Risk | Likelihood | Impact | Mitigation |
|------|------|------------|--------|------------|
| R-1  | Control ID abbreviation collisions across sections with similar titles | Medium | Medium | Implement collision-detection; append numeric suffix or use longer abbreviation on collision |
| R-2  | Nested section hierarchy does not map cleanly to OSCAL group nesting | Low | Medium | Start with flat group mapping (one level); add nested group support iteratively if needed |
| R-3  | Upstream domain model structure changes break the Catalog builder | Low | Medium | Use the stable domain model interface defined in WI-5; compiler catches breaking changes |
