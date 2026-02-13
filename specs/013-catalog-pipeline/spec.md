# Feature Specification: End-to-End Catalog Pipeline

**Feature Branch**: `013-catalog-pipeline`
**Created**: 2026-02-12
**Status**: Draft
**Input**: Derived from PRD `docs/PRD/013-prd-catalog-pipeline.md` (WI-13)

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Convert Markdown Policy to OSCAL Catalog JSON (Priority: P1)

A compliance engineer runs the FORGE CLI to convert a Markdown security policy into an OSCAL Catalog JSON document. This is the primary use case — the first time FORGE delivers its core value proposition: transforming human-readable policy documents into machine-readable OSCAL format via a single command.

> As a compliance engineer, I want to run a single command to convert my Markdown security policy into an OSCAL Catalog JSON document so that I can begin using my policy requirements in machine-readable form.

**Why this priority**: This is the MS-2 milestone exit criteria and the primary user scenario from the parent PRD (US-1). It validates that all 12 prior work items integrate correctly and FORGE delivers demonstrable end-to-end value.

**Independent Test**: Run the convert command with a sample Markdown policy containing sections and requirements, and verify a complete OSCAL Catalog JSON is produced on stdout with groups, controls, metadata, and back matter.

**Acceptance Scenarios**:

1. **Given** a Markdown policy document with 3 sections and 10 requirements, **When** running the catalog conversion command, **Then** a complete OSCAL Catalog JSON is produced on stdout with 3 groups, 10 controls, valid metadata, and back matter. *(AC-1: M-1, M-2, M-3)*
2. **Given** a Markdown policy document with YAML frontmatter (title, version), **When** converting to Catalog JSON, **Then** the OSCAL metadata fields (title, version, last-modified, oscal-version "1.2.0") are correctly populated from the document metadata. *(AC-2: M-1)*
3. **Given** a policy with compound statements (e.g., "Systems must X and must Y"), **When** converting, **Then** the compound statements are atomized into separate controls with individual deterministic stable IDs (content-derived UUIDs via WI-7). *(AC-7: M-1)*

---

### User Story 2 — Write OSCAL Catalog to File (Priority: P1)

A compliance engineer directs the OSCAL Catalog output to a file instead of the default terminal output, enabling integration into automated workflows.

> As a compliance engineer, I want to specify an output file path so that I can save the OSCAL Catalog directly to a file without shell redirection.

**Why this priority**: File output is essential for practical use. While terminal output is the default and supports piping, many workflows require a named output file for downstream processing.

**Independent Test**: Run the convert command with an output file path and verify the file is created with valid OSCAL Catalog JSON content.

**Acceptance Scenarios**:

1. **Given** a Markdown policy document and a specified output file path, **When** running the convert command, **Then** the specified file is created containing the OSCAL Catalog JSON. *(AC-4: M-5)*
2. **Given** the output file path is omitted, **When** running the convert command, **Then** the OSCAL Catalog JSON is printed to the terminal. *(AC-3: M-4)*

---

### User Story 3 — Smoke Test End-to-End Pipeline (Priority: P1)

A developer verifies that the full pipeline works end-to-end with a representative sample policy, providing automated confidence that all pipeline stages are correctly integrated.

> As a developer working on FORGE, I want an automated smoke test that converts a sample Markdown policy through the full pipeline so that I can verify all pipeline stages are correctly wired together.

**Why this priority**: The smoke test is the engineering verification that WI-1 through WI-12 are correctly integrated. Without it, there is no automated confidence that the pipeline works as a whole.

**Independent Test**: Run the automated test suite and verify the end-to-end smoke test passes, confirming a sample policy produces valid OSCAL Catalog JSON with expected structure.

**Acceptance Scenarios**:

1. **Given** a sample Markdown policy fixture in the test suite, **When** running the full pipeline programmatically in a test, **Then** the output is valid JSON containing an `oscal-version` field, a `catalog` object with `metadata`, `groups`, and controls. *(AC-6: M-7)*
2. **Given** the smoke test, **When** verifying the output structure, **Then** the number of groups matches the number of top-level sections in the source policy.

---

### User Story 4 — Descriptive Error Reporting (Priority: P2)

A compliance engineer receives clear error messages when something goes wrong during conversion, rather than cryptic failures.

> As a compliance engineer, I want clear error messages and a non-zero exit code when the conversion fails so that I can diagnose and fix issues with my input document.

**Why this priority**: Error feedback is essential for usability but does not block the core conversion flow.

**Independent Test**: Run the convert command with invalid inputs (missing file, empty file, bad flags) and verify descriptive error messages and non-zero exit codes.

**Acceptance Scenarios**:

1. **Given** any pipeline stage fails during conversion, **When** the command completes, **Then** a non-zero exit code and a descriptive error message are produced. *(S-2)*
2. **Given** an unsupported strategy value is provided, **When** running the convert command, **Then** the command rejects the value with a descriptive error indicating only "catalog" is currently supported. *(S-3)*

---

### Edge Cases

- **EC-1** (M-1): When the input file does not exist, the command exits with a non-zero status code and a descriptive error message.
- **EC-2** (M-1): When the input file is empty (zero bytes), the command exits with an error indicating no content to process.
- **EC-3** (M-5): When the output path is in a non-existent directory, the command exits with an error indicating the output path is invalid.
- **EC-4** (M-2): When the strategy flag is omitted, the command exits with an error indicating the strategy flag is required.
- **EC-5** (M-3): When the format flag is omitted, the command exits with an error indicating the format flag is required.
- **EC-6** (M-1): When the input Markdown has no identifiable sections or requirements, the pipeline produces a Catalog with empty groups and a warning is emitted to stderr via `tracing::warn!`.
- **EC-7** (M-5): When the output path points to an existing file, the file is overwritten with the new output.

## Requirements *(mandatory)*

### Functional Requirements

#### Must Have (M) — MVP, launch blockers

- **M-1**: The convert command SHALL wire the full pipeline — ingest, parse, normalize, map, assemble, serialize — producing OSCAL Catalog JSON from a Markdown input file. *(Traces to: Parent PRD M-3, M-7)*
- **M-2**: The convert command SHALL require a `--strategy` flag to select the conversion strategy (no default). *(Traces to: Parent PRD M-3)*
- **M-3**: The convert command SHALL require a `--format` flag to select the output format (no default). *(Traces to: Parent PRD M-7)*
- **M-4**: The default output destination SHALL be the terminal (stdout) when no output path is specified. *(Traces to: Parent PRD M-7)*
- **M-5**: The convert command SHALL accept an output path flag to write the OSCAL Catalog JSON to a file. *(Traces to: Parent PRD M-7)*
- **M-6**: The output JSON SHALL be a syntactically valid JSON document parseable by any standard JSON parser (RFC 8259, UTF-8 encoded). *(Traces to: Parent PRD M-7)*
- **M-7**: An automated smoke test SHALL verify end-to-end conversion of a sample Markdown policy to OSCAL Catalog JSON, checking for the presence of catalog, metadata, groups, and controls in the output. *(Traces to: Parent PRD AC-3)*

#### Should Have (S) — High value, not blocking

- **S-1**: The output JSON SHOULD be pretty-printed (indented) by default for human readability.
- **S-2**: The pipeline SHOULD produce a non-zero exit code and descriptive error message if any pipeline stage fails (e.g., file not found, parsing failure, assembly error).
- **S-3**: The strategy flag SHOULD reject values other than "catalog" with a descriptive error (component support deferred to WI-18).

#### Could Have (C) — Nice to have, if time permits

- **C-1**: A compact output option COULD produce minified (non-indented) JSON output for reduced file size.
- **C-2**: A dry-run option COULD run the pipeline without producing output, reporting only statistics (sections found, requirements found, controls generated).

#### Won't Have (W) — Explicitly deferred

- **W-1**: Component Definition pipeline (strategy: component) — *Deferred to WI-14 through WI-18*
- **W-2**: XML or YAML output formats — *Deferred to WI-26, WI-27 (Phase 2)*
- **W-3**: Schema validation of output — *Deferred to WI-19 (schema validation sprint)*
- **W-4**: Traceability metadata in output — *Deferred to WI-16, WI-17 (traceability sprints)*
- **W-5**: Profile generation — *Deferred to WI-30 (Phase 2)*

### Key Entities

- **Pipeline**: The full sequence of processing stages that transforms a Markdown policy document into an OSCAL Catalog. Stages: ingest, parse, normalize (atomize + UUID + citations), map (to OSCAL groups/controls), assemble (metadata + back matter), serialize (to JSON).
- **OSCAL Catalog**: The output artifact — a structured collection of controls (requirements) organized into groups, with metadata and back matter, conforming to OSCAL v1.2.0.
- **Policy Document**: The input artifact — a Markdown file containing security policy sections, requirements, and optional YAML frontmatter metadata.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A compliance engineer can convert a Markdown policy to OSCAL Catalog JSON using a single CLI command, producing a complete output in one step.
- **SC-002**: The output document contains all expected structural elements: catalog root, metadata (title, version, oscal-version, last-modified), groups matching source sections, and controls matching source requirements.
- **SC-003**: Compound requirements in the source document are correctly split into individual controls, each with a unique stable identifier, ensuring 100% of atomic requirements are captured.
- **SC-004**: The automated smoke test passes consistently, verifying pipeline integration across all stages (WI-1 through WI-12).
- **SC-005**: The output can be written to a named file or printed to the terminal, with the terminal as the default for composability with other tools.
- **SC-006**: When invalid inputs are provided (missing file, empty file, bad flags), the user receives a clear error message and the command exits with a failure indicator.

## Assumptions

- **A-1**: All upstream pipeline stages (WI-1 through WI-12) are complete and their tests passing before WI-13 begins.
- **A-2**: JSON serialization capabilities are already available from prior work items.
- **A-3**: The OSCAL Catalog structure produced by WI-9 through WI-12 is serializable to JSON without additional transformation.
- **A-4**: Terminal output (stdout) is an acceptable default; file output via an output path flag is additive.

## Dependencies

- **Requires**: WI-9 (Catalog Groups & Controls), WI-10 (Statement Parts & Prose), WI-11 (OSCAL Metadata), WI-12 (Back Matter) — and transitively all of WI-1 through WI-8
- **Blocks**: WI-14 (Component Definition Structure) — component definition builds on the catalog pipeline pattern
