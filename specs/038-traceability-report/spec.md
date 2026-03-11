# Feature Specification: Traceability Report

**Feature Branch**: `038-traceability-report`
**Created**: 2026-03-10
**Status**: Draft
**Input**: Derived from PRD 038, AR 038, and SEC 038 — `forge trace` subcommand for mapping OSCAL elements to source policy locations

## Clarifications

### Session 2026-03-10

- Q: FR-003 specifies a "Source Paragraph" column, but WI-17 only embeds `source-file`, `source-section`, and `source-line` props — no paragraph metadata exists. How should the report handle this? → A: Drop the Source Paragraph column entirely from the report. The table columns are: OSCAL Element ID, Element Type, Source Section, Source Line.
- Q: FR-008 requires source hash comparison, but WI-17 does not embed a source file hash in OSCAL artifacts. How should WI-38 handle source integrity checking? → A: Compare source file mtime against OSCAL `metadata.last-modified` as a basic staleness heuristic. No WI-17 changes required.
- Q: WI-17 embeds full trace props (source-file, source-section, source-line) on controls but only source-section on groups (no line number). How should groups appear in the report? → A: Include groups as rows with section title populated and Source Line shown as "—" (not applicable). Groups count toward total elements and coverage (mapped if they have a source-section prop).
- Q: FR-002 lists "part" as an element type, but WI-17 does not embed trace metadata on parts. Should parts appear in the report? → A: Exclude parts from the report. Only walk groups, controls, and implemented-requirements — the element types that WI-17 actually traces.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Produce Traceability Report (Priority: P1)

A compliance engineer generates a traceability report to audit the mapping between an OSCAL artifact and the source policy document. The engineer runs `forge trace catalog.json --source policy.md` and receives a structured table that maps every OSCAL element (group, control, implemented-requirement) to its originating source policy location — including the section title and line number.

**Why this priority**: This is the core purpose of WI-38 and directly satisfies the parent PRD requirement S-6 ("The CLI shall produce a traceability report mapping source policy locations to OSCAL element identifiers") and User Story US-7. Without this subcommand, traceability metadata embedded by WI-17 is inaccessible to users.

**Independent Test**: Generate an OSCAL Catalog from a sample policy (with trace metadata embedded by WI-17), run `forge trace catalog.json --source policy.md`, and verify the output is a structured table with one row per OSCAL element showing element ID, element type, source section, and source line number.

**Acceptance Scenarios**:

1. **Given** an OSCAL Catalog JSON with 3 groups and 10 controls containing trace metadata, **When** running `forge trace catalog.json --source policy.md`, **Then** a structured table is produced with 13 rows (3 groups + 10 controls), each showing the OSCAL element ID, type, source section title, and line number.
2. **Given** an OSCAL Component Definition JSON with 5 implemented-requirements containing trace metadata, **When** running `forge trace compdef.json --source policy.md`, **Then** a structured table is produced with rows for each implemented-requirement, each mapped to its source location.

---

### User Story 2 - Save Traceability Report to File (Priority: P2)

A compliance engineer saves the traceability report to a file for inclusion in audit documentation packages. The engineer specifies an output path and the report is written there instead of to the terminal.

**Why this priority**: File output is essential for practical audit workflows, but the core functionality (stdout output) is higher priority.

**Independent Test**: Run `forge trace catalog.json --source policy.md --output trace-report.txt` and verify the file is created with the same structured table content that would appear on stdout.

**Acceptance Scenarios**:

1. **Given** `--output trace-report.txt` is specified, **When** running `forge trace`, **Then** the file `trace-report.txt` is created containing the structured table.
2. **Given** the `--output` flag is omitted, **When** running `forge trace`, **Then** the structured table is printed to stdout.

---

### User Story 3 - Verify Complete Coverage (Priority: P2)

A compliance engineer verifies that every OSCAL element has a source mapping. Any elements lacking trace metadata are flagged as "unmapped" so the engineer can identify gaps in the conversion provenance.

**Why this priority**: Completeness of traceability is essential for audit confidence. Gaps undermine trust in the conversion and must be visible.

**Independent Test**: Generate a traceability report from an artifact where one control has no trace metadata and verify the report flags it as unmapped with a summary warning indicating incomplete coverage.

**Acceptance Scenarios**:

1. **Given** an OSCAL Catalog where one control lacks trace metadata, **When** running `forge trace`, **Then** the report includes a row for that control marked as "unmapped" and a summary warning indicating incomplete coverage.
2. **Given** an OSCAL Catalog where all controls have trace metadata, **When** running `forge trace`, **Then** the report summary indicates 100% coverage.

---

### User Story 4 - Source Integrity Warning (Priority: P3)

> *Traces to PRD S-3 ("The report should warn if the source policy file appears to have been modified since conversion")*

A compliance engineer is warned when the source policy file may have been modified since the OSCAL artifact was generated, so that stale line number references are identified before being relied upon for audit.

**Why this priority**: Important for data integrity, but the core report generation and coverage detection take precedence.

**Independent Test**: Modify the source policy after generating an OSCAL artifact, run `forge trace`, and verify a warning is emitted about the source file being newer than the artifact.

**Acceptance Scenarios**:

1. **Given** a source policy file whose modification time is newer than the OSCAL artifact's `metadata.last-modified` timestamp, **When** running `forge trace`, **Then** a warning is displayed indicating the source policy may have been modified since conversion.
2. **Given** a source policy file whose modification time is older than or equal to the OSCAL artifact's `metadata.last-modified` timestamp, **When** running `forge trace`, **Then** no source integrity warning is displayed.

---

### Edge Cases

- What happens when the OSCAL artifact file does not exist? The tool exits with a non-zero status code and a descriptive error message identifying the missing file.
- What happens when the source policy file does not exist? The tool exits with a descriptive error indicating the source file path is invalid.
- What happens when the OSCAL artifact contains no trace metadata at all (e.g., generated without WI-17)? All elements are flagged as "unmapped" and a warning is emitted about the absence of trace data.
- What happens when a trace metadata reference points to a line number beyond the source file's length? The entry is flagged with a "source modified" warning for that specific element.
- What happens when the OSCAL artifact is neither a Catalog nor a Component Definition? The tool exits with an error indicating the artifact type is unsupported.
- What happens when the OSCAL artifact is invalid JSON? The tool exits with a descriptive parsing error.
- What happens when source content contains terminal control characters (ANSI escape sequences)? Control characters (ASCII 0x00-0x1F, excluding newline and tab) are stripped from source-derived content before embedding in the report output to prevent terminal injection.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide a `forge trace <artifact> --source <policy>` subcommand that produces a traceability report mapping OSCAL elements to source policy locations.
- **FR-002**: System MUST map each OSCAL element (group, control, implemented-requirement) to its source section title and line number. Parts are excluded — WI-17 does not embed trace metadata on parts. Groups that have only a `source-section` prop (no line number) are shown with Source Line as "—" and are considered mapped.
- **FR-003**: System MUST output the report as a structured, column-aligned table with columns: OSCAL Element ID, Element Type, Source Section, Source Line.
- **FR-004**: System MUST support both Catalog and Component Definition OSCAL artifact types.
- **FR-005**: System MUST extract trace metadata from props/links embedded in the OSCAL artifact by WI-17.
- **FR-006**: System MUST flag elements without trace metadata as "unmapped" in the report and include a coverage summary showing total elements, mapped elements, unmapped elements, and coverage percentage.
- **FR-007**: System MUST support an `--output <path>` flag to write the report to a file instead of stdout.
- **FR-008**: System MUST warn when the source policy file appears to have been modified after the OSCAL artifact was generated, by comparing the source file's modification time against the OSCAL artifact's `metadata.last-modified` timestamp.
- **FR-009**: System MUST validate that both input files (artifact and source) exist before processing and exit with descriptive errors if either is missing.
- **FR-010**: System MUST exit with a descriptive error when the OSCAL artifact is invalid JSON or an unsupported artifact type.
- **FR-011**: System MUST handle source line number references that exceed the actual source file length by flagging the affected entry with a "source modified" warning rather than crashing.
- **FR-012**: System MUST strip ASCII control characters (0x00-0x1F, excluding newline 0x0A and tab 0x09) from source-derived content before embedding in the report output.

### Key Entities

- **TraceReport**: The complete traceability report containing the artifact path, source path, artifact type, a collection of trace entries, and a coverage summary.
- **TraceEntry**: A single mapping between an OSCAL element (identified by ID and type) and its source location (section, line number). Includes a mapping status indicating whether trace metadata was found.
- **TraceSummary**: Aggregate statistics for the report — total elements, mapped elements, unmapped elements, and coverage percentage.
- **TraceMetadata**: Source location information extracted from an OSCAL element's trace props/links — section title and line number (matching WI-17's `source-file`, `source-section`, `source-line` props under the `https://forge.policy-forge.github.io/ns/trace` namespace).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Compliance engineers can generate a traceability report from any OSCAL Catalog or Component Definition artifact in a single command invocation.
- **SC-002**: 100% of OSCAL elements in an artifact appear in the report — either mapped to a source location or flagged as unmapped. No elements are silently omitted.
- **SC-003**: Source location references (section, line number) in the report match the actual source policy document locations with 100% accuracy when the source file has not been modified since conversion.
- **SC-004**: The report output is readable and well-formatted in a standard terminal (80+ column width) with aligned columns and clear separation between entries.
- **SC-005**: The tool provides clear, actionable error messages for all failure modes (missing files, invalid JSON, unsupported artifact type) that enable the user to resolve the issue without consulting documentation.
- **SC-006**: Coverage gaps are immediately visible — a compliance engineer can identify which OSCAL elements lack source provenance within 30 seconds of viewing the report.

## Assumptions

- WI-17 has embedded trace metadata (props/links) in OSCAL artifacts, and the metadata format is stable and documented.
- The trace metadata includes sufficient information to resolve source section and line number references.
- The source policy document is available at the path specified by `--source` and ideally has not been modified since conversion.
- The `forge trace` subcommand can be added to the existing clap CLI structure following established patterns.
- The trace metadata prop/link naming convention follows the format established by WI-17: props `source-file`, `source-section`, `source-line` under namespace `https://forge.policy-forge.github.io/ns/trace`, and links with rel `source` and href `<file>#line=<n>`.
