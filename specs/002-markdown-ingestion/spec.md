# Feature Specification: Markdown Ingestion

**Feature Branch**: `002-markdown-ingestion`
**Created**: 2026-02-11
**Status**: Draft
**Input**: Derived from PRD 002-prd-markdown-ingestion, AR 002-ar-markdown-ingestion, SEC 002-sec-markdown-ingestion

## Clarifications

### Session 2026-02-11

- Q: Should oversized files (>10MB) be warned about but still processed, or rejected outright? → A: Reject by default with a `--max-size` CLI flag to override the limit.
- Q: What format should the ingested content be output in on stdout? → A: JSON object with fields `source_path`, `fingerprint`, and `lines` (array of `{number, text}`).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Read a Markdown Policy File (Priority: P1)

A compliance engineer provides a Markdown policy file path to FORGE and the tool reads it into memory with line-by-line tracking, preparing it for downstream conversion to OSCAL.

> As a compliance engineer, I want to provide a Markdown policy file to FORGE so that it can be processed through the conversion pipeline.

**Why this priority**: This is the entry point to the entire conversion pipeline. No conversion to OSCAL is possible without file ingestion. Every downstream feature depends on this capability.

**Independent Test**: Run `forge convert policy.md` with a valid Markdown file and verify that a JSON object is output to stdout containing `source_path`, `fingerprint`, and `lines` (array of `{number, text}` objects).

**Acceptance Scenarios**:

1. **Given** a valid Markdown file at `policy.md`, **When** running `forge convert policy.md`, **Then** a JSON object is output to stdout containing `source_path`, `fingerprint`, and `lines` fields.
2. **Given** a Markdown file with 100 lines, **When** ingested, **Then** the JSON output contains a `lines` array of 100 objects, each with a `number` field (1-based) and `text` field matching the original line content.
3. **Given** a valid Markdown file, **When** ingested, **Then** the JSON output contains a `fingerprint` field with the SHA-256 hash of the file content.

---

### User Story 2 - Reject Unsupported File Formats (Priority: P1)

A user accidentally provides a non-Markdown file (PDF, DOCX, or other) and receives a clear, actionable error message explaining that only Markdown is supported and suggesting how to convert their document.

> As a compliance engineer, I want clear feedback when I provide an unsupported file format so that I know to pre-convert my document to Markdown using an external tool.

**Why this priority**: Per the project's Markdown-only input decision, clear error messages at the boundary prevent confusion and guide users to the correct workflow. Without this, users would get cryptic failures.

**Independent Test**: Run `forge convert policy.pdf` and verify a descriptive error is returned with a non-zero exit code.

**Acceptance Scenarios**:

1. **Given** a file with a `.pdf` extension, **When** running `forge convert policy.pdf`, **Then** the CLI exits with an error message indicating only Markdown files are supported and suggests external conversion tools (e.g., pandoc, markitdown).
2. **Given** a file with a `.docx` extension, **When** running `forge convert policy.docx`, **Then** the CLI exits with a similar descriptive error and conversion suggestions.
3. **Given** a file with no extension, **When** running `forge convert policy`, **Then** the CLI exits with an error indicating the format is unsupported.
4. **Given** a file with an uppercase `.MD` extension, **When** running `forge convert policy.MD`, **Then** the file is accepted as valid Markdown (case-insensitive detection).

---

### User Story 3 - Handle File Access Errors Gracefully (Priority: P1)

A user provides a path to a file that does not exist or cannot be read, and FORGE returns a clear, descriptive error message with a non-zero exit code.

> As a compliance engineer, I want clear error messages when a file cannot be found or accessed so that I can quickly diagnose and fix the issue.

**Why this priority**: Robust error handling is essential for a CLI tool. Users need actionable feedback for all common failure modes to use the tool effectively.

**Independent Test**: Run `forge convert nonexistent.md` and verify a descriptive "file not found" error with non-zero exit code is returned.

**Acceptance Scenarios**:

1. **Given** a non-existent file path, **When** running `forge convert missing.md`, **Then** the CLI exits with a non-zero exit code and a descriptive "file not found" error message.
2. **Given** a file that exists but is not readable (insufficient permissions), **When** running `forge convert restricted.md`, **Then** the CLI exits with a descriptive permission error.
3. **Given** a file that contains non-UTF-8 encoded content, **When** running `forge convert binary.md`, **Then** the CLI exits with a descriptive encoding error explaining the file must be valid UTF-8 text.

---

### User Story 4 - Handle Oversized Files Safely (Priority: P2)

A user provides a very large file and FORGE rejects it by default to prevent excessive resource consumption. Users can override the limit with the `--max-size` CLI flag.

> As a compliance engineer, I want FORGE to handle unexpectedly large files safely so that the tool does not consume excessive memory or hang.

**Why this priority**: While policy documents are typically small (<1MB), protecting against edge cases with very large files prevents poor user experience and resource exhaustion. This is a safety measure rather than a core feature.

**Independent Test**: Provide a file exceeding the size threshold and verify it is rejected with a descriptive error. Then retry with `--max-size 20` to verify the override works.

**Acceptance Scenarios**:

1. **Given** a file exceeding the default size limit (10MB), **When** running `forge convert huge.md`, **Then** the CLI rejects the file with a non-zero exit code and a descriptive error message stating the file exceeds the 10MB limit.
2. **Given** a file exceeding the default size limit (10MB), **When** running `forge convert huge.md --max-size 20`, **Then** the file is accepted and processed normally because the user raised the limit to 20MB.

---

### Edge Cases

- What happens when the input file is empty (0 bytes)? The system returns a valid but empty result (not an error).
- What happens when the file has a `.MARKDOWN` extension (uppercase)? It is recognized as Markdown (case-insensitive detection).
- What happens when the file path contains special characters or spaces? The system handles them correctly via standard path handling.
- What happens when the file is a symlink to a Markdown file? The system follows the symlink and reads the target file.
- What happens when the user provides a directory path instead of a file? The system returns a descriptive error.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST read a Markdown file from a given filesystem path and output its content to stdout as a JSON object with fields: `source_path` (string), `fingerprint` (string, SHA-256), and `lines` (array of `{number: integer, text: string}`).
- **FR-002**: System MUST detect input format by file extension (`.md`, `.markdown`) using case-insensitive matching.
- **FR-003**: System MUST reject unsupported file formats with a descriptive error message that suggests external conversion tools for common formats (PDF, DOCX).
- **FR-004**: System MUST preserve line number tracking, mapping each line of content to its original 1-based line number in the source file.
- **FR-005**: System MUST return a non-zero exit code and descriptive error message when the input file does not exist.
- **FR-006**: System MUST return a non-zero exit code and descriptive error message when the input file is not readable (permission denied).
- **FR-007**: System MUST validate that the input file is valid UTF-8 text and return a descriptive encoding error for non-UTF-8 files.
- **FR-008**: System MUST compute a SHA-256 fingerprint of the file content for downstream traceability and change detection.
- **FR-009**: System MUST record the original source file path alongside the ingested content.
- **FR-010**: System SHOULD check file size before reading and reject files exceeding a configurable limit (default: 10MB) with a non-zero exit code. The limit is overridable via a `--max-size` CLI flag (value in MB).
- **FR-011**: System SHOULD verify the path refers to a regular file (not a directory, device, or other special file) before reading.

### Key Entities

- **IngestedDocument**: Represents a Markdown file that has been read from the filesystem. Contains the original file path, content fingerprint (SHA-256 hash), and a collection of source lines with their line numbers. This is the output of the ingestion process and the input to all downstream pipeline stages.
- **SourceLine**: Represents a single line from the source document. Contains the 1-based line number and the text content of that line (without trailing newline). Enables traceability from any downstream artifact back to the exact source line.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of valid Markdown files (`.md`, `.markdown`, case-insensitive) are read successfully and their content is output to stdout as valid JSON.
- **SC-002**: All failure modes (file not found, permission denied, unsupported format, invalid encoding) produce descriptive, actionable error messages with non-zero exit codes.
- **SC-003**: Line number tracking is 100% accurate — every line in the ingested document maps to its correct 1-based line number in the source file.
- **SC-004**: The same file content always produces the same fingerprint (hash), enabling reliable change detection.
- **SC-005**: Empty files (0 bytes) are handled gracefully as valid input, not errors.
- **SC-006**: Error messages for unsupported formats include suggestions for external conversion tools.

### Assumptions

- Input files are UTF-8 encoded Markdown text.
- File sizes are reasonable for in-memory processing (policy documents are typically under 1MB).
- The file size limit defaults to 10MB and is overridable via the `--max-size` CLI flag.
- Users have standard filesystem permissions on the files they provide.
- The ingestion layer only reads files — it does not parse Markdown structure (headings, sections, tables). Structural extraction is a separate downstream capability.

### Dependencies

- **Requires**: Project scaffolding (001-project-scaffolding) — CLI skeleton, error type definitions, module structure.
- **Blocks**: Structural extraction of headings/sections, clause/table extraction, and all downstream pipeline features.
