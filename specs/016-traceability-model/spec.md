# Feature Specification: Traceability Model

**Feature Branch**: `016-traceability-model`
**Created**: 2026-02-13
**Status**: Draft
**Input**: User description: "Traceability TraceLink Model — mapping OSCAL elements to source policy locations"

## Clarifications

### Session 2026-02-13

- Q: Should `oscal_json_path` use JSON Pointer (RFC 6901) or dot-notation format? → A: Dot-notation (e.g., `catalog.groups[0].controls[2]`) — more human-readable and matches all existing examples.

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Trace OSCAL Element Back to Source (Priority: P1)

A compliance engineer inspects a generated OSCAL Catalog and needs to verify which policy section produced a specific control. They look up the OSCAL element's identifier and receive the exact file path, section title, and line number where the source requirement originated.

> As a compliance engineer, I want every generated OSCAL element to be traceable back to its source policy section and line number so that I can verify correctness and satisfy audit requirements.

**Why this priority**: This is the core purpose of the traceability model. The parent PRD requirement M-10 explicitly requires traceability from every generated OSCAL element back to its source. Product principle P-2 ("Traceability is non-negotiable") makes this the highest-priority capability.

**Independent Test**: Generate OSCAL from a test policy document, retrieve the trace link collection, and verify that every OSCAL element identifier has an associated source location with valid file path, section title, and line number.

**Acceptance Scenarios**:

1. **Given** a policy document converted to an OSCAL Catalog, **When** looking up any control's element identifier in the trace link collection, **Then** a trace link is returned containing the source file path, section title, and line number where the requirement originated.
2. **Given** a policy document converted to an OSCAL Component Definition, **When** looking up any implemented-requirement's element identifier in the trace link collection, **Then** a trace link is returned with the correct source location.

---

### User Story 2 — Trace Source Requirement Forward to OSCAL Elements (Priority: P1)

A developer or compliance engineer needs to verify that a specific policy requirement was correctly mapped into OSCAL output. They look up the requirement's stable identifier and see all OSCAL elements that were generated from it, confirming nothing was dropped during conversion.

> As a developer, I want to look up a policy requirement's stable identifier and find all OSCAL elements it generated so that I can verify complete coverage and confirm no requirements were dropped during conversion.

**Why this priority**: Bidirectional traceability is essential for completeness verification. If a requirement exists in the source but has no forward trace link, it was silently dropped — a critical defect that violates the "no orphan elements" principle.

**Independent Test**: Generate OSCAL from a test policy, query the trace link collection by a known requirement stable identifier, and verify it returns the expected OSCAL element identifiers and logical paths.

**Acceptance Scenarios**:

1. **Given** a requirement with a known stable identifier that maps to a Catalog control, **When** querying the trace link collection by that stable identifier, **Then** the result includes the OSCAL control's element identifier and logical path.
2. **Given** a requirement that maps to both a Catalog control and a Component Definition implemented-requirement, **When** querying by that requirement's stable identifier, **Then** both OSCAL elements are returned.

---

### User Story 3 — Capture Source Location Metadata (Priority: P1)

The system must record precise source locations so that downstream consumers (traceability embedding, trace reports) have accurate provenance data. Source locations include the file path, section title, and line number — enabling users to navigate directly to the originating text.

> As a compliance engineer, I want source locations to include the file path, section title, and line number so that I can navigate directly to the originating text in the source policy document.

**Why this priority**: Source location precision is the foundation of meaningful traceability. Without file path, section, and line number, trace links are vague and un-navigable.

**Independent Test**: Construct a source location from a known file, section, and line, then verify all three fields are accurately stored and retrievable.

**Acceptance Scenarios**:

1. **Given** a requirement extracted from file "policy.md", section "Access Control", line 42, **When** stored as a source location within a trace link, **Then** the source location fields return "policy.md", "Access Control", and 42 respectively.
2. **Given** a trace link with a populated source location, **When** displaying the trace link for debugging or reporting, **Then** the output includes all three source location fields in a human-readable format.

---

### User Story 4 — Collect Trace Links During Catalog Generation (Priority: P1)

During the Catalog generation pipeline, the system must automatically record trace links for every control element created. This ensures traceability is captured at generation time — the only moment when the mapping context (which requirement produced which OSCAL element) is available.

> As a system operator, I want trace links to be automatically captured during Catalog generation so that traceability data is recorded at the point of creation and is not lost.

**Why this priority**: If trace links are not captured at generation time, the mapping between source requirements and OSCAL elements is permanently lost. This is a blocking requirement for all downstream traceability features.

**Independent Test**: Run the Catalog generation pipeline with a test policy document and verify the resulting trace link collection contains one trace link per generated control with accurate source locations.

**Acceptance Scenarios**:

1. **Given** a policy document processed through the Catalog generation pipeline, **When** generation completes, **Then** the trace link collection contains a trace link for every generated control element.
2. **Given** a Catalog with 5 controls generated from 5 requirements, **When** inspecting the trace link collection, **Then** all 5 trace links have valid source locations pointing back to the correct sections of the source document.

---

### User Story 5 — Collect Trace Links During Component Definition Generation (Priority: P1)

During the Component Definition generation pipeline, the system must automatically record trace links for every implemented-requirement element created.

> As a system operator, I want trace links to be automatically captured during Component Definition generation so that every implemented-requirement element is traceable to its source policy requirement.

**Why this priority**: Component Definition generation is the second major OSCAL output pathway. Without trace link capture here, half of the generated OSCAL output would lack traceability.

**Independent Test**: Run the Component Definition generation pipeline with a test policy document and verify the resulting trace link collection contains one trace link per generated implemented-requirement with accurate source locations.

**Acceptance Scenarios**:

1. **Given** a policy document processed through the Component Definition generation pipeline, **When** generation completes, **Then** the trace link collection contains a trace link for every generated implemented-requirement element.
2. **Given** a Component Definition with 3 implemented-requirements, **When** inspecting the trace link collection, **Then** all 3 trace links correctly map back to their respective source requirements.

---

### Edge Cases

- What happens when looking up a requirement stable identifier that has no trace links in the collection? The system returns an empty result (not an error or crash).
- What happens when looking up an OSCAL element identifier that has no trace link? The system returns a "not found" result (not an error or crash).
- What happens when attempting to record a trace link with a duplicate OSCAL element identifier that already exists? The system rejects the duplicate and returns an error, since each OSCAL element maps to exactly one source requirement.
- What happens when a requirement has no parent section title (e.g., a top-level requirement outside any heading)? The source location section title defaults to an empty string.
- What happens when a single source requirement generates multiple OSCAL elements (e.g., one Catalog control and one Component implemented-requirement)? Both trace links are stored and the forward lookup returns all of them.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST define a trace link data structure that maps a policy requirement's stable identifier to the corresponding OSCAL element's logical path, element identifier, and source location in the original document.
- **FR-002**: The system MUST define a source location data structure capturing the source file path, section title, and line number (1-based) for each traced requirement.
- **FR-003**: The system MUST define a trace link collection container that aggregates trace links and supports bidirectional lookup.
- **FR-004**: The trace link collection MUST support forward lookup — given a requirement stable identifier, return all associated trace links (source-to-OSCAL direction).
- **FR-005**: The trace link collection MUST support reverse lookup — given an OSCAL element identifier, return the associated trace link (OSCAL-to-source direction).
- **FR-006**: The system MUST capture trace links during Catalog generation by recording a trace link for each generated control element at the point of creation.
- **FR-007**: The system MUST capture trace links during Component Definition generation by recording a trace link for each generated implemented-requirement element at the point of creation.
- **FR-008**: Source locations MUST be populated from the policy requirement's source line, the containing section title, and the document's source file path as defined in the domain model (WI-5).
- **FR-009**: The trace link collection MUST reject duplicate OSCAL element identifiers — each OSCAL element maps to exactly one source requirement. Attempting to record a duplicate returns an error.
- **FR-010**: The trace link collection SHOULD support enumeration of all trace links for reporting or batch processing purposes.
- **FR-011**: The trace link collection SHOULD provide summary statistics (total count and empty check) for inspection and diagnostics.
- **FR-012**: Trace links and source locations SHOULD support inspection, cloning, and serialization for testing and future persistence needs.

### Key Entities

- **TraceLink**: A mapping record connecting a policy requirement's stable identifier to the OSCAL element it produced. Contains the requirement stable identifier, OSCAL logical path (dot-notation format, e.g., `catalog.groups[0].controls[2]`), OSCAL element identifier, and source location.
- **SourceLocation**: The origin coordinates of a policy requirement in its source document. Contains the file path, section title, and line number.
- **TraceLinkCollection**: An aggregation container holding all trace links produced during a single conversion run. Provides bidirectional lookup capability (source-to-OSCAL and OSCAL-to-source) and enforces the uniqueness constraint on OSCAL element identifiers.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of generated OSCAL elements (controls, implemented-requirements) have an associated trace link after generation — no "orphan" elements exist.
- **SC-002**: Every forward lookup by requirement stable identifier returns the complete set of generated OSCAL elements for that requirement, with 100% accuracy.
- **SC-003**: Every reverse lookup by OSCAL element identifier returns the correct source location (file path, section title, line number), with 100% accuracy.
- **SC-004**: Source locations accurately reflect the original document coordinates — file path, section title, and line number all match the source material.
- **SC-005**: Duplicate OSCAL element identifiers are rejected 100% of the time, preventing corrupted traceability data.
- **SC-006**: Lookups for non-existent identifiers return empty/not-found results gracefully — no crashes or unhandled errors under any input conditions.

## Assumptions

- The requirement stable identifier is available from the UUID generation feature (WI-7) by the time trace links are captured. If WI-7 is not yet integrated, a placeholder identifier from the domain model suffices.
- The Catalog generation pipeline (WI-9) and Component Definition pipeline (WI-14/WI-15) can be instrumented to emit trace links during element creation without significant rework.
- A single policy requirement may map to multiple OSCAL elements (one-to-many from source to OSCAL), while each OSCAL element maps to exactly one source requirement (one-to-one from OSCAL to source).
- The OSCAL logical path uses a notation format that is stable across runs for the same input (deterministic generation).
- Trace links are immutable after creation — they are recorded once during generation and never modified.
- The trace link collection is append-only during generation and read-only afterward.

## Dependencies

- **Requires**: Domain Model (WI-5) — provides the policy requirement, section title, and source file path fields used to populate source locations.
- **Requires**: Catalog Generation (WI-9) — the Catalog builder must exist to be instrumented for trace link capture.
- **Requires**: Component Definition Structure (WI-14) — the Component builder must exist to be instrumented.
- **Parallel With**: Implemented Requirements (WI-15) — trace links capture implemented-requirement mappings produced by this feature.
- **Blocks**: Traceability Embedding (WI-17) — needs the trace link collection to embed traceability metadata into OSCAL artifacts.
- **Blocks**: Schema Validation (WI-19) — must account for trace metadata in validated artifacts.

## Risks

- **R-1**: Existing generation pipelines may be difficult to instrument for trace link capture. Mitigation: Design the collection with a simple record-and-go interface that minimizes coupling with existing generators.
- **R-2**: OSCAL logical paths could become invalid if the artifact structure changes between generation and downstream embedding. Mitigation: Compute paths at generation time when structure is known; validate before embedding in WI-17.
- **R-3**: The one-to-many mapping from source requirements to OSCAL elements could create confusion in downstream reports. Mitigation: Document cardinality clearly and ensure the collection interface makes the one-to-many nature explicit.
