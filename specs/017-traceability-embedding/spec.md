# Feature Specification: Traceability Embedding

**Feature Branch**: `017-traceability-embedding`
**Created**: 2026-02-13
**Status**: Draft
**Input**: PRD: docs/PRD/017-prd-traceability-embedding.md (WI-17)
**Depends On**: WI-16 (016-traceability-model), WI-15 (015-component-implemented-requirements)
**Blocks**: WI-18 (018-end-to-end-component-pipeline)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Discover Source Location from OSCAL Control (Priority: P1)

An auditor inspects a generated OSCAL Catalog and needs to verify the source of a specific control. Each control must contain prop and link elements indicating its source policy file, section, and line number so that the auditor can trace any control back to its authoritative policy text without needing a separate report.

**Why this priority**: This is the core traceability capability required by Parent PRD M-10. Without it, generated OSCAL artifacts are opaque and unauditable. This alone delivers a viable MVP — any Catalog consumer can discover provenance for every control.

**Independent Test**: Generate an OSCAL Catalog from a policy document and inspect any control's `props` and `links` arrays for source location metadata.

**Acceptance Scenarios**:

1. **Given** a Markdown policy document with a requirement at line 42 in section "3.1 Access Control", **When** converting to OSCAL Catalog, **Then** the generated control has a `prop` with `name: "source-file"`, a `prop` with `name: "source-line"` and `value: "42"`, and a `prop` with `name: "source-section"` and `value: "3.1 Access Control"`, all using the FORGE trace namespace.
2. **Given** the same generated control, **When** inspecting its `links` array, **Then** there is a `link` with `rel: "source"` and an `href` referencing the source file and line in the format `<file>#line=<n>`.
3. **Given** a generated Catalog with trace props, **When** looking up a control by its ID and reading its source-line prop, **Then** the source location (file, section, line) is unambiguous and correct.

---

### User Story 2 - Discover Source Location from Component Definition (Priority: P1)

A compliance engineer inspects a generated Component Definition and needs to verify which policy text backs each implemented-requirement. Each implemented-requirement must contain props and links indicating its source policy section and line so that the engineer can verify the mapping between policy text and control implementation narratives.

**Why this priority**: Component Definition traceability is equally critical for Parent PRD M-10 and is the primary MS-3 deliverable alongside Catalog traceability.

**Independent Test**: Generate an OSCAL Component Definition and inspect any implemented-requirement's `props` and `links` for source location metadata.

**Acceptance Scenarios**:

1. **Given** a policy document and a baseline profile reference, **When** converting to Component Definition, **Then** each `implemented-requirement` has `prop` elements for `source-file`, `source-section`, and `source-line` using the FORGE trace namespace.
2. **Given** the same Component Definition, **When** inspecting the documentary component itself, **Then** the component has a `prop` with `name: "source-file"` indicating the policy document it was derived from.
3. **Given** a generated implemented-requirement, **When** inspecting its `links` array, **Then** there is a `link` with `rel: "source"` pointing to the source document and location.

---

### User Story 3 - Verify No Trace Data in Remarks (Priority: P1)

A compliance engineer needs confidence that FORGE does not misuse OSCAL `remarks` fields for trace metadata. Trace metadata must be stored exclusively in prop and link elements, never in remarks fields, so that generated artifacts comply with NIST OSCAL guidance.

**Why this priority**: Parent PRD M-11 explicitly prohibits storing arbitrary data in remarks. Violating this makes artifacts non-compliant with NIST guidance.

**Independent Test**: Generate OSCAL artifacts and verify that no `remarks` field contains trace metadata (file paths, line numbers, section references).

**Acceptance Scenarios**:

1. **Given** any generated OSCAL artifact (Catalog or Component Definition), **When** inspecting all `remarks` fields, **Then** none contain source file paths, line numbers, or section identifiers — all such data is in `props` or `links` exclusively.

---

### User Story 4 - Group-Level Source Annotation (Priority: P2)

An auditor reviewing a Catalog's structure wants to see which source section a group of controls corresponds to, so that the organizational mapping between the policy and the OSCAL grouping is visible.

**Why this priority**: Adds structural traceability context at the group level, making the Catalog more navigable, but individual control traceability (US-1) is sufficient for core compliance verification.

**Independent Test**: Generate an OSCAL Catalog from a policy with hierarchical sections and inspect group elements for source section props.

**Acceptance Scenarios**:

1. **Given** a Markdown policy document with a heading "3. Access Controls" that contains sub-requirements, **When** converting to OSCAL Catalog, **Then** the corresponding group element has a `prop` with `name: "source-section"` and the hierarchical section path.
2. **Given** a group that has no direct source section (a synthetic grouping), **When** inspecting its props, **Then** no `source-section` prop is present (no empty or placeholder values).

---

### Edge Cases

- **EC-1** (M-1): When a policy requirement spans multiple lines, `source-line` records the starting line of the requirement.
- **EC-2** (M-1): When a policy requirement was atomized from a compound statement (WI-6), each resulting control gets the source-line of the original compound statement.
- **EC-3** (M-8): When two different requirements originate from the same source line (e.g., atomized compound statement), both controls have the same `source-line` value but distinct control IDs, maintaining bidirectional traceability.
- **EC-4** (S-1): When a group has no direct source section (e.g., a synthetic grouping), it receives no `source-section` prop rather than an empty or placeholder value.
- **EC-5** (M-6): When a prop name like `source-file` collides with a future NIST-defined prop name, the FORGE namespace disambiguates the two.
- **EC-6** (M-7): When the source policy file path contains special characters (spaces, unicode), the `source-file` prop value preserves the exact user-provided path (no normalization) and the `link` href properly percent-encodes it per RFC 3986.

## Clarifications

### Session 2026-02-13

- Q: How should WI-17 handle the transition from the existing prefix-based `forge:source-line` prop (no `ns` field on `OscalProp`) to the namespace-based prop model required by M-6? → A: Extend `OscalProp` with an optional `ns` field and replace the existing `forge:source-line` with the new namespaced `source-line` prop (no dual/legacy props).
- Q: What format should the `source-file` prop value use — preserve user-provided path, normalize to relative, or strip to filename? → A: Preserve the path exactly as provided by the user on the CLI (no normalization).
- Q: Should `OscalGroup` gain both `props` and `links` fields (like `OscalControl`) or only the minimal `props` field for S-1? → A: Add both `props` and `links` fields to `OscalGroup` for structural consistency with `OscalControl`.

## Requirements *(mandatory)*

### Functional Requirements

#### Must Have (MVP, launch blockers)

- **M-1**: Every generated OSCAL control (in Catalog output) SHALL have `prop` elements recording `source-file`, `source-section`, and `source-line` from its originating policy requirement.
- **M-2**: Every generated OSCAL control SHALL have a `link` element with `rel: "source"` pointing to the source document and location using the format `<file>#line=<n>`.
- **M-3**: Every generated `implemented-requirement` (in Component Definition output) SHALL have `prop` elements recording `source-file`, `source-section`, and `source-line`.
- **M-4**: Every generated `implemented-requirement` SHALL have a `link` element with `rel: "source"` pointing to the source document and location.
- **M-5**: The documentary component element itself SHALL have a `prop` element recording the `source-file` from which it was derived.
- **M-6**: All trace-related props SHALL use the FORGE namespace (`ns: "https://forge.policy-forge.github.io/ns/trace"`) to avoid collisions with NIST-defined prop names.
- **M-7**: No trace metadata (file paths, section names, line numbers) SHALL appear in any `remarks` field in generated artifacts.
- **M-8**: Bidirectional traceability SHALL be verifiable: given any generated OSCAL element with trace props, the source location is unambiguous; given a source section, the corresponding OSCAL element ID(s) can be determined from the artifact's props.

#### Should Have (high value, not blocking)

- **S-1**: Group elements in Catalog output SHOULD have `prop` elements recording the source section they map to.
- **S-2**: Prop values for `source-section` SHOULD use the section's hierarchical path (e.g., "3.1 Access Control") rather than just the immediate heading text, to disambiguate sections with identical titles.

#### Could Have (nice to have, if time permits)

- **C-1**: A `prop` with `name: "source-hash"` containing a content hash of the source text, enabling consumers to detect if the source has changed since generation.

#### Won't Have (explicitly deferred)

- **W-1**: Human-readable traceability report output — deferred to WI-38/WI-39 (forge trace subcommand).
- **W-2**: Traceability embedding in XML or YAML output formats — deferred to WI-26/WI-27 (Phase 2).
- **W-3**: Traceability across Profile resolution boundaries — deferred to WI-36 (oscal-cli integration).
- **W-4**: Interactive traceability visualization or navigation — out of scope for CLI tool.

### Key Entities

- **TraceLink**: A mapping record produced by WI-16 that connects a source policy location (file, section, line) to a generated OSCAL element (by element ID). Each TraceLink carries the source file path, section title, line number, and the OSCAL element ID it maps to.
- **Prop (OSCAL Property)**: A name/value pair with optional `class`, `ns` (namespace), and `uuid` attributes. Used to attach structured annotations to OSCAL elements. Trace props use three names: `source-file`, `source-section`, `source-line`. The existing `OscalProp` struct must be extended with an optional `ns` field to support namespaced props. The prior `forge:source-line` prefix-based prop is replaced by the namespaced `source-line` prop (no dual/legacy props retained).
- **Link (OSCAL Link)**: A reference to an external or internal resource via `href`, with a `rel` attribute indicating the relationship type. Trace links use `rel: "source"` with href format `<file>#line=<n>`.
- **FORGE Trace Namespace**: The URI `https://forge.policy-forge.github.io/ns/trace` used as the `ns` value on all FORGE-specific trace props to avoid collision with NIST-defined prop names.
- **OscalGroup (structural change)**: Must be extended with `props` and `links` fields (both `Vec`, skip-serialized when empty) for structural consistency with `OscalControl`, enabling S-1 group-level `source-section` props.
- **DocumentaryComponent (structural change)**: Must be extended with a `props` field (`Vec`, skip-serialized when empty) to support M-5 `source-file` prop on the component element.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of generated controls in Catalog output contain all three trace props (`source-file`, `source-section`, `source-line`) with correct values matching their source policy locations.
- **SC-002**: 100% of generated controls in Catalog output contain a `source` link with a valid href pointing to the correct source file and line.
- **SC-003**: 100% of generated implemented-requirements in Component Definition output contain all three trace props with correct values.
- **SC-004**: 100% of generated implemented-requirements contain a `source` link with a valid href.
- **SC-005**: The documentary component contains a `source-file` prop indicating the policy document it was derived from.
- **SC-006**: 100% of trace props use the FORGE trace namespace — zero props with missing or incorrect namespace.
- **SC-007**: Zero `remarks` fields in any generated artifact contain trace metadata (file paths, section names, line numbers).
- **SC-008**: Given any OSCAL element ID in a generated artifact, its source location (file, section, line) is recoverable from its props; given any source location, the corresponding OSCAL element ID(s) are discoverable by scanning artifact props.

## Assumptions

- WI-16 (TraceLink model) provides a complete collection of TraceLinks after OSCAL generation, with each TraceLink containing the source file path, section title, line number, and the OSCAL element ID it maps to.
- WI-15 (implemented-requirements) provides the Component Definition structure with `implemented-requirements` elements that accept `props` and `links` per the OSCAL v1.2.0 schema.
- The OSCAL v1.2.0 schema permits user-defined `prop` names (with a `ns` namespace) and `link` elements with custom `rel` values on controls, groups, components, and implemented-requirements.
- Source line numbers captured during parsing (WI-3/WI-4) remain accurate through atomization (WI-6) and are available in the TraceLink model.

## Risks

| ID   | Risk                                                                              | Likelihood | Impact | Mitigation                                                                                                                                                 |
| ---- | --------------------------------------------------------------------------------- | ---------- | ------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| R-1  | Custom prop names conflict with future OSCAL standard prop names                  | Low        | Med    | Use a FORGE-specific namespace (`ns: "https://forge.policy-forge.github.io/ns/trace"`) to avoid collisions with NIST-defined prop names                    |
| R-2  | Embedded trace props significantly increase artifact file size for large policies  | Low        | Low    | Props are small name/value pairs; even 500 controls with 3 props each add minimal overhead. Monitor in WI-24 benchmarks.                                   |
| R-3  | TraceLink model from WI-16 does not capture all required fields for embedding     | Med        | Med    | Define the expected TraceLink interface contract in this spec; coordinate with WI-16 implementation to ensure fields are present                            |
