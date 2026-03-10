# Feature Specification: Batch Conversion

**Feature Branch**: `040-batch-conversion`
**Created**: 2026-03-10
**Status**: Draft
**Input**: User description: "Batch conversion: support multiple input files in a single forge convert invocation with parallel processing and aggregated status"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Convert Multiple Policy Documents (Priority: P1)

A compliance engineer converts multiple policy documents to OSCAL format in a single command instead of running separate invocations for each file.

**Why this priority**: This is the core purpose of batch conversion and directly enables organizations with multi-document policy suites to work efficiently. Without this, users must run `forge convert` once per file, repeating flags each time.

**Independent Test**: Run `forge convert policy1.md policy2.md policy3.md --strategy catalog --format json --output output/` and verify three OSCAL Catalog JSON files are produced in the `output/` directory with filenames derived from the input filenames.

**Acceptance Scenarios**:

1. **Given** three Markdown policy files and an output directory, **When** the user runs `forge convert policy1.md policy2.md policy3.md --strategy catalog --format json --output output/`, **Then** three OSCAL Catalog JSON files are created in the `output/` directory named `policy1.json`, `policy2.json`, `policy3.json`.
2. **Given** multiple input files and no `--output` flag, **When** the user runs `forge convert policy1.md policy2.md --strategy catalog --format json`, **Then** each OSCAL Catalog JSON is written to an auto-generated filename in the current directory (e.g., `policy1.json`, `policy2.json`).

---

### User Story 2 - Aggregated Status Output (Priority: P1)

A compliance engineer reviews the conversion results for all files in a single summary after batch conversion completes.

**Why this priority**: Without aggregated status, the user cannot quickly determine if all conversions succeeded. Essential for operational confidence when processing multiple documents.

**Independent Test**: Run a batch conversion where one of three files has a parsing error, and verify the aggregated status shows 2 successes and 1 failure with the error message for the failed file.

**Acceptance Scenarios**:

1. **Given** three input files where one contains invalid Markdown structure, **When** running batch conversion, **Then** the aggregated status output on stderr shows 2 successes and 1 failure, with the failure entry including the filename and error message.
2. **Given** all input files are valid, **When** running batch conversion, **Then** the aggregated status shows all files as successful with a summary count (total, succeeded, failed, total time).
3. **Given** a batch where one file fails, **When** checking the CLI exit code, **Then** the exit code is non-zero.

---

### User Story 3 - Parallel Processing for Performance (Priority: P2)

A developer benefits from faster batch conversion when processing a large set of policy documents, because files are processed in parallel.

**Why this priority**: Parallel processing is a performance enhancement that makes batch conversion practical for large document sets. The core batch functionality (US-1, US-2) is higher priority, but parallelism is important for real-world usability.

**Independent Test**: Run a batch conversion of 10 policy files and verify the total time is at least 2x faster than 10 sequential single-file conversions.

**Acceptance Scenarios**:

1. **Given** 10 independent Markdown policy files, **When** running batch conversion, **Then** the total conversion time is at least 2x faster than 10 sequential single-file conversions.
2. **Given** parallel processing is active, **When** one file conversion fails, **Then** other file conversions are not affected and continue to completion.
3. **Given** a `--jobs <n>` flag, **When** the user specifies `--jobs 1`, **Then** files are processed sequentially (no parallelism).

---

### User Story 4 - Glob Pattern Input (Priority: P2)

A compliance engineer uses a glob pattern to select all policy files in a directory for batch conversion, rather than listing each file individually.

**Why this priority**: Glob patterns significantly improve usability for directory-based policy organization, reducing the need to enumerate files manually.

**Independent Test**: Run `forge convert policies/*.md --strategy catalog --format json --output output/` where `policies/` contains 5 Markdown files, and verify 5 output files are produced.

**Acceptance Scenarios**:

1. **Given** a `policies/` directory with 5 Markdown files, **When** running `forge convert policies/*.md --strategy catalog --format json --output output/`, **Then** 5 OSCAL Catalog JSON files are produced in the `output/` directory.
2. **Given** a glob pattern that matches no files, **When** running `forge convert empty_dir/*.md`, **Then** a descriptive error is displayed indicating no files matched.

---

### Edge Cases

- What happens when only one input file is provided? The behavior must be identical to the existing single-file conversion (backward compatible).
- What happens when zero input files are provided (e.g., glob matches nothing)? The CLI exits with a descriptive error indicating no input files.
- What happens when two input files have the same name but different directories (e.g., `dir1/policy.md` and `dir2/policy.md`)? Output files must not collide — a numeric suffix is appended to avoid overwriting (e.g., `policy.json`, `policy_2.json`).
- What happens when `--output` is a file path (not a directory) and multiple inputs are provided? The CLI exits with an error indicating `--output` must be a directory for batch mode.
- What happens when all files in the batch fail? The aggregated status shows all failures and the exit code is non-zero.
- What happens when the `--output` directory does not exist? It is created automatically.
- What happens when the batch contains more than 100 files? A warning is emitted to stderr indicating the large batch size, but processing continues normally.

## Clarifications

### Session 2026-03-10

- Q: Should FORGE enforce a maximum batch size, and if so, what threshold? → A: Warn on stderr at 100 files but continue processing.
- Q: What minimum speedup ratio should the parallel processing test assert? → A: At least 2x faster than sequential.
- Q: What should the maximum allowed value for --jobs be? → A: 256.
- Q: How should FORGE handle symlinked input files in batch mode? → A: Follow symlinks silently (standard CLI behavior).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST accept multiple input file paths in a single `forge convert` invocation.
- **FR-002**: System MUST independently convert each input file through the existing pipeline (Catalog or Component), producing its own OSCAL artifact.
- **FR-003**: System MUST write each output file to the `--output` directory (when specified) with a filename derived from the input filename (e.g., `policy1.md` produces `policy1.json`).
- **FR-004**: System MUST print an aggregated status summary to stderr after all conversions complete, showing per-file success/failure with error messages for failures.
- **FR-005**: System MUST continue converting remaining files when one file's conversion fails (error isolation).
- **FR-006**: System MUST exit with a non-zero status code if any file in the batch fails conversion.
- **FR-007**: System SHOULD process files in parallel using a thread pool, with parallelism limited to the number of available CPU cores by default.
- **FR-008**: System SHOULD provide a `--jobs <n>` flag allowing the user to control the degree of parallelism (valid range: 0–256; 0 = auto-detect CPU cores, 1–256 = explicit thread count).
- **FR-009**: System SHOULD write each output to an auto-generated filename in the current directory when multiple inputs are provided without `--output`.
- **FR-010**: System SHOULD include total file count, success count, failure count, and total processing time in the aggregated status summary.
- **FR-011**: System MUST maintain backward compatibility — a single input file must behave identically to the existing single-file conversion.
- **FR-012**: System MUST handle output filename collisions (same-name files from different directories) by appending a numeric suffix.
- **FR-013**: System MUST validate all input files exist and are readable before beginning batch processing (fail-fast).
- **FR-014**: System MUST exit with an error when `--output` is a file (not a directory) and multiple inputs are provided.
- **FR-015**: System MUST create the `--output` directory automatically if it does not exist.
- **FR-016**: System SHOULD emit a warning to stderr when the batch contains more than 100 input files, but MUST NOT refuse to process them.

### Key Entities

- **BatchRun**: Represents a single batch conversion invocation, including the conversion strategy, output format, output directory, and parallelism level.
- **FileResult**: The outcome of converting a single file — includes the input path, output path (if successful), success/failure status, error message (if failed), and duration.
- **BatchSummary**: Aggregation of all FileResults — total files, succeeded count, failed count, total duration, and the ordered list of per-file results.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can convert 10 policy documents in a single invocation, with all 10 producing correct OSCAL output files.
- **SC-002**: When one file in a batch of 10 fails, the remaining 9 are still converted successfully.
- **SC-003**: Aggregated status clearly shows per-file results (filename, success/failure, error message) and a summary (total, succeeded, failed, elapsed time).
- **SC-004**: Batch conversion of 10 files completes in at least 2x less time than 10 sequential single-file conversions (parallel processing benefit).
- **SC-005**: Single-file invocations behave identically to pre-batch behavior (zero regression).
- **SC-006**: Output filenames are deterministic and predictable — derived from input filenames with collision avoidance.

## Assumptions

- The existing single-file conversion pipeline (Catalog and Component strategies) is stable and can be invoked multiple times independently without side effects.
- Each input file produces its own independent OSCAL artifact — no cross-document merging or linking.
- The shell expands glob patterns before passing arguments to the CLI (standard POSIX shell behavior), so FORGE receives individual file paths.
- The `--output` flag, when used with multiple inputs, must specify a directory (not a single file path).

## Scope Boundaries

**In Scope:**
- Supporting multiple input files in a single `forge convert` invocation
- Parallel processing of independent conversions
- Aggregated status output showing per-file success/failure results
- Per-file output naming when `--output` specifies a directory
- Glob pattern support (via shell expansion)
- `--jobs` flag for parallelism control

**Out of Scope:**
- Merging multiple policy documents into a single OSCAL artifact
- Cross-document traceability or inter-document linking
- Batch traceability report generation
- Recursive directory traversal for input files
- Progress bars or real-time streaming status
- `--continue-on-error` flag (default is always continue)
- `--dry-run` flag
- JSON-formatted aggregated status output
