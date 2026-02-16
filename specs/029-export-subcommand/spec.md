# Feature Specification: Export Subcommand

**Feature Branch**: `029-export-subcommand`
**Created**: 2026-02-15
**Status**: Complete
**Input**: PRD `docs/PRD/029-prd-export-subcommand.md` (WI-29)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Convert OSCAL JSON to XML (Priority: P1)

A compliance engineer has a valid OSCAL Catalog in JSON format and needs to provide it to a GRC tool that requires XML input.

**Why this priority**: JSON-to-XML conversion is the most common cross-format need. This is the core use case and directly satisfies parent PRD US-5 AC-2.

**Independent Test**: Run `forge export catalog.json --format xml` and verify the output is a valid OSCAL XML document semantically equivalent to the input JSON.

**Acceptance Scenarios**:

1. **Given** a valid OSCAL Catalog JSON file, **When** running `forge export catalog.json --format xml`, **Then** a valid OSCAL XML representation is produced to stdout.
2. **Given** a valid OSCAL Catalog JSON file and `--output catalog.xml`, **When** running `forge export catalog.json --format xml --output catalog.xml`, **Then** the XML output is written to the specified file path.

---

### User Story 2 - Convert OSCAL JSON to YAML (Priority: P1)

A compliance engineer needs a human-readable YAML representation of an OSCAL artifact for review or manual editing.

**Why this priority**: YAML is preferred for human review. Supporting JSON-to-YAML alongside JSON-to-XML completes the multi-format story.

**Independent Test**: Run `forge export catalog.json --format yaml` and verify the output is a valid OSCAL YAML document semantically equivalent to the input JSON.

**Acceptance Scenarios**:

1. **Given** a valid OSCAL Catalog JSON file, **When** running `forge export catalog.json --format yaml`, **Then** a valid OSCAL YAML representation is produced.
2. **Given** a valid OSCAL Component Definition JSON file, **When** running `forge export component.json --format yaml`, **Then** the YAML output contains the same semantic content as the JSON input.

---

### User Story 3 - Convert Between Any Format Pair (Priority: P1)

A user needs to convert between any combination of JSON, XML, and YAML formats.

**Why this priority**: Full format flexibility is required for complete multi-format export capability.

**Independent Test**: Run `forge export catalog.xml --format json` and verify the output is a valid OSCAL JSON document.

**Acceptance Scenarios**:

1. **Given** a valid OSCAL XML artifact, **When** running `forge export artifact.xml --format json`, **Then** a valid OSCAL JSON representation is produced.
2. **Given** a valid OSCAL YAML artifact, **When** running `forge export artifact.yaml --format xml`, **Then** a valid OSCAL XML representation is produced.
3. **Given** a valid OSCAL JSON artifact, **When** running `forge export artifact.json --format json`, **Then** the output is a valid (potentially re-formatted) JSON representation.

---

### User Story 4 - Validate Output After Conversion (Priority: P1)

The exported artifact must be validated against the target format's OSCAL schema to ensure correctness.

**Why this priority**: Validation after conversion is essential and is a core differentiator over generic format conversion tools.

**Independent Test**: Run `forge export artifact.json --format xml` on a valid input and verify the output passes OSCAL schema validation; run on an invalid input and verify an error is reported.

**Acceptance Scenarios**:

1. **Given** a valid OSCAL artifact, **When** exporting to any target format, **Then** the output passes OSCAL v1.2.0 schema validation.
2. **Given** an invalid or non-OSCAL input file, **When** running `forge export invalid.json --format xml`, **Then** a descriptive error is reported indicating the input is not a valid OSCAL artifact.

---

### Edge Cases

- EC-1: When input file has no extension → report descriptive error listing supported extensions (.json, .xml, .yaml, .yml)
- EC-2: When file extension doesn't match content (e.g., `.json` containing XML) → deserialization fails with a descriptive error (the extension-detected format is authoritative; no content-based fallback). FR-007 content-based detection applies only when the extension is missing/unrecognized (S-1, deferred).
- EC-3: Same-format export (e.g., JSON→JSON) → re-serialize and validate (normalization pass)
- EC-4: `--output` path points to read-only location → report filesystem error
- EC-5: Input file is empty (0 bytes) → report empty file error
- EC-6: Input is valid JSON but not valid OSCAL → distinguish "not JSON" from "not OSCAL"
- EC-7: `--format` not provided → CLI reports required argument, exits non-zero

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001** (M-1): System MUST provide `forge export <input> --format <json|xml|yaml>` subcommand
- **FR-002** (M-2): System MUST auto-detect input format from file extension (`.json`, `.xml`, `.yaml`, `.yml`)
- **FR-003** (M-3): System MUST deserialize and re-serialize preserving semantic equivalence
- **FR-004** (M-4): System MUST validate output against OSCAL v1.2.0 schemas
- **FR-005** (M-5): System MUST support `--output <path>` for file output (default: stdout)
- **FR-006** (M-6): System MUST report descriptive error and non-zero exit code for invalid input
- **FR-007** (S-1): System SHOULD support content-based format detection as fallback
- **FR-008** (S-2): System SHOULD report detected formats in verbose mode
- **FR-009** (S-3): Same-format export SHOULD re-serialize and validate (normalization)

### Key Entities

- **OutputFormat**: Existing enum representing JSON, XML, YAML serialization formats (reused from `src/cli/mod.rs` per research.md RQ-5; AR's `OscalFormat` name was superseded)
- **ExportArgs**: CLI argument struct for the export subcommand
- **CatalogEnvelope / ComponentDefinitionEnvelope**: Existing OSCAL model envelope types for deserialization

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All 9 format combinations (3x3) produce valid OSCAL output — 100% coverage
- **SC-002**: Exported artifacts pass OSCAL v1.2.0 schema validation — 100% pass rate
- **SC-003**: Round-trip fidelity: export→re-import produces semantically equivalent model. Verified by deserializing the exported output back through `deserialize_oscal()` and comparing the resulting model fields; covered by the 18 format-pair tests in `src/cli/export.rs` which export and then re-read the output.
- **SC-004**: Test coverage exceeds 90% for export module
- **SC-005**: Format conversion of 100KB–1MB artifact completes in under 1 second
