# Feature Specification: Multi-Format Round-Trip Testing

**Feature Branch**: `028-round-trip-testing`
**Created**: 2026-02-15
**Status**: Draft
**Input**: Derived from PRD 028-prd-round-trip-testing, AR 028-ar-round-trip-testing, SEC 028-sec-round-trip-testing

## User Scenarios & Testing *(mandatory)*

### User Story 1 - JSON to XML to JSON Round-Trip (Priority: P1)

A developer verifies that converting an OSCAL artifact from JSON to XML and back to JSON produces a semantically equivalent document.

**Why this priority**: JSON is the primary format; XML is the most structurally different. This path exercises the greatest surface area for data loss due to JSON's unordered objects vs. XML's ordered elements.

**Independent Test**: Serialize a known OSCAL Catalog JSON to XML using WI-26's serializer, deserialize the XML back to the internal model, re-serialize to JSON, and compare using semantic equivalence.

**Acceptance Scenarios**:

1. **Given** a valid OSCAL Catalog in JSON, **When** converting JSON → XML → JSON, **Then** the resulting JSON is semantically equivalent to the original.
2. **Given** a valid OSCAL Component Definition in JSON, **When** converting JSON → XML → JSON, **Then** the resulting JSON is semantically equivalent to the original.
3. **Given** an OSCAL artifact with ordered arrays, **When** converting JSON → XML → JSON, **Then** array element order is preserved.

---

### User Story 2 - JSON to YAML to JSON Round-Trip (Priority: P1)

A developer verifies that converting an OSCAL artifact from JSON to YAML and back to JSON produces a semantically equivalent document.

**Why this priority**: YAML type coercion risks (bare "true"/"false" → booleans, numeric strings → numbers) make this a high-risk path for silent data corruption.

**Independent Test**: Serialize a known OSCAL Catalog JSON to YAML, deserialize back to JSON, and compare using semantic equivalence.

**Acceptance Scenarios**:

1. **Given** a valid OSCAL Catalog in JSON, **When** converting JSON → YAML → JSON, **Then** the resulting JSON is semantically equivalent to the original.
2. **Given** a valid OSCAL Component Definition in JSON, **When** converting JSON → YAML → JSON, **Then** the resulting JSON is semantically equivalent to the original.
3. **Given** an OSCAL artifact with YAML-ambiguous strings ("true", "1.0", "null"), **When** converting JSON → YAML → JSON, **Then** those values remain strings.

---

### User Story 3 - XML to YAML to XML Round-Trip (Priority: P2)

A developer verifies that converting between the two non-JSON formats preserves data integrity.

**Why this priority**: The export subcommand (WI-29) allows arbitrary format-to-format conversion. This path must also be verified for completeness.

**Independent Test**: Load a valid OSCAL Catalog in XML, convert to YAML and back to XML, and compare the result using semantic equivalence.

**Acceptance Scenarios**:

1. **Given** a valid OSCAL Catalog in XML, **When** converting XML → YAML → XML, **Then** the resulting XML is semantically equivalent to the original.
2. **Given** an OSCAL artifact in XML with namespace declarations, **When** converting XML → YAML → XML, **Then** OSCAL namespaces are preserved correctly in the output XML.

---

### User Story 4 - Semantic Equivalence Comparison Utility (Priority: P1)

A developer has access to a reusable semantic equivalence comparison utility for use in tests across the project.

**Why this priority**: All round-trip assertions depend on this utility. String comparison produces false negatives due to key ordering/whitespace.

**Independent Test**: Compare two JSON documents with identical content but different key ordering and verify the utility reports them as equivalent.

**Acceptance Scenarios**:

1. **Given** two JSON objects with identical content but different key ordering, **When** comparing, **Then** result is "equivalent".
2. **Given** two JSON objects where one has an extra key, **When** comparing, **Then** result is "not equivalent" with diff.
3. **Given** two JSON objects with a nested value difference, **When** comparing, **Then** result is "not equivalent" with path and values.

---

### Edge Cases

- Empty JSON objects `{}` report as equivalent
- Empty arrays `[]` are preserved through all round-trip paths
- ISO 8601 timestamps remain strings after YAML round-trip
- UUID strings remain strings after all round-trip paths
- Deeply nested objects (5+ levels) are preserved through round-trips
- Numeric strings like "10" or "3.14" remain strings after YAML round-trip
- YAML 1.1 boolean-like strings ("yes", "no", "on", "off") remain strings after YAML round-trip

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Test suite MUST verify JSON → XML → JSON round-trip fidelity for OSCAL Catalog artifacts (PRD M-1)
- **FR-002**: Test suite MUST verify JSON → YAML → JSON round-trip fidelity for OSCAL Catalog artifacts (PRD M-2)
- **FR-003**: Test suite MUST verify JSON → XML → JSON round-trip fidelity for OSCAL Component Definition artifacts (PRD M-3)
- **FR-004**: Test suite MUST verify JSON → YAML → JSON round-trip fidelity for OSCAL Component Definition artifacts (PRD M-4)
- **FR-005**: Semantic equivalence comparison MUST ignore JSON object key ordering (PRD M-5)
- **FR-006**: Semantic equivalence comparison MUST preserve and verify array element ordering (PRD M-6)
- **FR-007**: Test suite MUST produce pass/fail with structural diff on failure showing path and nature of discrepancy (PRD M-7)
- **FR-008**: Semantic equivalence comparison MUST verify all data types are preserved through round-trips (PRD M-8)
- **FR-009**: XML deserialization MUST be implemented for Catalog and Component Definition to enable XML round-trip paths (prerequisite for FR-001, FR-003; gap from WI-26)
- **FR-010**: Semantic equivalence utility MUST be exposed as a reusable module (PRD S-3)
- **FR-011**: Test suite MUST verify XML → YAML → XML round-trip fidelity for OSCAL Catalog artifacts (PRD S-1)
- **FR-012**: Test suite MUST verify XML → YAML → XML round-trip fidelity for OSCAL Component Definition artifacts (PRD S-1)

### Key Entities

- **EquivalenceResult**: Structured comparison result (is_equivalent: bool, differences: Vec<EquivalenceDiff>)
- **EquivalenceDiff**: Single difference with JSON Pointer path, description, expected/actual values

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% semantic equivalence for JSON → XML → JSON round-trip on Catalog and Component Definition
- **SC-002**: 100% semantic equivalence for JSON → YAML → JSON round-trip on Catalog and Component Definition
- **SC-003**: 0 YAML type coercion incidents (all edge-case tests pass)
- **SC-004**: 100% of round-trip tests passing via `cargo test`
- **SC-005**: All clippy and fmt checks pass
- **SC-006**: 100% semantic equivalence for XML → YAML → XML round-trip on Catalog and Component Definition
