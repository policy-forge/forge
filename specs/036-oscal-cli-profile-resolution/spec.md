# Feature Specification: oscal-cli Profile Resolution Integration

**Feature Branch**: `036-oscal-cli-profile-resolution`
**Created**: 2026-03-10
**Status**: Draft
**Input**: Derived from docs/PRD/036-prd-oscal-cli-profile-resolution.md, docs/AR/036-ar-oscal-cli-profile-resolution.md, docs/SEC/036-sec-oscal-cli-profile-resolution.md

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Resolve a Profile via oscal-cli (Priority: P1)

A compliance engineer generates an OSCAL Profile with FORGE and wants to resolve it into a flat Catalog baseline. Rather than manually invoking a separate tool, they run `forge resolve <profile.json>` and FORGE delegates the resolution to NIST's oscal-cli, producing a resolved Catalog containing only the selected and tailored controls.

**Why this priority**: Profile Resolution is the critical step between generating a Profile and using it for downstream compliance workflows. This is the core function of this work item.

**Independent Test**: Generate a Profile with `forge profile`, then run `forge resolve profile.json` and verify a resolved Catalog JSON file is produced containing the controls selected by the Profile's import directives.

**Acceptance Scenarios**:

1. **Given** a valid FORGE-generated Profile JSON and oscal-cli installed, **When** running `forge resolve profile.json`, **Then** a resolved Catalog JSON is produced containing the controls selected by the Profile's import directives.
2. **Given** a Profile with modify directives (parameter values set), **When** resolving via `forge resolve`, **Then** the resolved Catalog reflects the modifications.
3. **Given** a valid Profile and `--output resolved.json` specified, **When** running `forge resolve profile.json --output resolved.json`, **Then** the resolved Catalog is written to `resolved.json`.

---

### User Story 2 - Graceful Degradation Without oscal-cli (Priority: P1)

A user runs FORGE on a system where oscal-cli is not installed. FORGE warns them that profile resolution is unavailable and exits gracefully, without crashing or disrupting other FORGE functionality.

**Why this priority**: oscal-cli is an external dependency that FORGE does not control. Users may not have it installed. A hard failure would break the user experience for a feature that is optional to the core conversion pipeline.

**Independent Test**: Remove oscal-cli from PATH, run `forge resolve profile.json`, and verify a descriptive warning is displayed with installation guidance.

**Acceptance Scenarios**:

1. **Given** oscal-cli is not installed or not on PATH, **When** running `forge resolve profile.json`, **Then** a warning message indicates oscal-cli is not found, includes installation guidance, and the command exits with a non-zero exit code (no panic).
2. **Given** oscal-cli is not installed, **When** running any other FORGE command (e.g., `forge convert`), **Then** no warning about oscal-cli is displayed and the command operates normally.

---

### User Story 3 - Handle oscal-cli Execution Errors (Priority: P1)

A user runs profile resolution but oscal-cli encounters an error (invalid Profile input, unsupported version, etc.). FORGE translates the oscal-cli error into a clear, actionable message instead of showing raw Java stack traces.

**Why this priority**: oscal-cli error output can be verbose and Java-stack-trace heavy. Users need actionable error messages from FORGE.

**Independent Test**: Provide an invalid Profile JSON to `forge resolve` and verify FORGE displays a clear error including relevant oscal-cli error detail.

**Acceptance Scenarios**:

1. **Given** an invalid Profile JSON file, **When** running `forge resolve invalid.json`, **Then** a descriptive error indicates the Profile is invalid, including relevant detail from oscal-cli stderr.
2. **Given** oscal-cli exits with a non-zero exit code, **When** FORGE captures the result, **Then** the exit code and a summary of the error are included in FORGE's error output.

---

### User Story 4 - Diagnostic oscal-cli Check (Priority: P2)

A developer troubleshooting FORGE wants to verify the oscal-cli integration status, path, and version.

**Why this priority**: Diagnostic capability is essential for troubleshooting integration issues with the external dependency.

**Independent Test**: Run `forge resolve --check` and verify it prints oscal-cli detection status and version (or "not found" message).

**Acceptance Scenarios**:

1. **Given** oscal-cli is installed, **When** running `forge resolve --check`, **Then** the output displays the oscal-cli version and executable path.
2. **Given** oscal-cli is not installed, **When** running `forge resolve --check`, **Then** the output displays a message indicating oscal-cli was not found with installation guidance.

---

### Edge Cases

- When oscal-cli is on PATH but not executable (permissions issue), a descriptive error indicates a permissions problem.
- When oscal-cli is installed but the `resolve-profile` subcommand is not supported (old version), a descriptive error suggests an upgrade.
- When oscal-cli produces stderr output but exits with code 0 (warnings), the resolution succeeds and warnings are forwarded to the user.
- When the input file exists but is not valid JSON (e.g., a YAML Profile), a descriptive error indicates the expected format.
- When oscal-cli execution exceeds the configured timeout, the process is terminated and a timeout error is displayed.
- When `--output` is omitted, the resolved Catalog is written to a default path derived from the input filename (e.g., `profile-resolved.json`).
- When the input file does not exist, a descriptive error is shown before invoking oscal-cli.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST detect whether oscal-cli is installed and available on the system PATH.
- **FR-002**: System MUST invoke oscal-cli resolve-profile to resolve a given OSCAL Profile into a resolved Catalog.
- **FR-003**: System MUST capture the resolved Catalog output from oscal-cli and write it to a specified output file (or a default path derived from the input filename).
- **FR-004**: When oscal-cli is not installed, system MUST display a descriptive warning message with installation guidance and exit gracefully without panicking.
- **FR-005**: When oscal-cli exits with a non-zero exit code, system MUST display a descriptive error message that includes relevant detail from oscal-cli's stderr output.
- **FR-006**: System MUST provide a `forge resolve` subcommand that accepts a Profile file path and an optional `--output` flag.
- **FR-007**: The `forge resolve` subcommand MUST validate that the input file exists and is a JSON file before invoking oscal-cli.
- **FR-008**: System SHOULD detect the installed oscal-cli version and log it for diagnostic purposes.
- **FR-009**: System SHOULD provide a `--check` flag that reports oscal-cli detection status, version, and path without performing resolution.
- **FR-010**: System SHOULD set a configurable timeout for oscal-cli execution (default: 60 seconds) to prevent indefinite hangs.
- **FR-011**: When oscal-cli is not found, the warning message SHOULD include installation guidance (link to NIST oscal-cli repository).
- **FR-012**: All process arguments MUST be passed via argument arrays. Shell string interpolation MUST NOT be used (command injection prevention).
- **FR-013**: The child process environment SHOULD be filtered to minimize environment variable leakage, using explicit environment variable allowlisting.
- **FR-014**: Input file paths MUST be canonicalized to resolve symlinks and relative paths before passing to oscal-cli.

### Key Entities

- **OscalCliInfo**: Detection result containing availability status, version string (optional), and executable path (optional).
- **ResolveResult**: Invocation result containing the path where the resolved Catalog was written.
- **OscalCliDetector**: Component responsible for detecting oscal-cli on the system PATH and retrieving version information. Abstracted behind a trait for testability.
- **OscalCliInvoker**: Component responsible for executing oscal-cli resolve-profile with proper argument passing, timeout enforcement, and error capture. Abstracted behind a trait for testability.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can resolve a FORGE-generated Profile into a resolved Catalog with a single command (`forge resolve profile.json`) in under 60 seconds for typical profiles.
- **SC-002**: When oscal-cli is not installed, FORGE displays a clear warning with installation guidance and exits gracefully 100% of the time (no panics, no misleading errors).
- **SC-003**: When oscal-cli encounters an error, 100% of error cases produce an actionable FORGE error message that identifies the root cause without requiring the user to interpret raw stderr.
- **SC-004**: The `forge resolve --check` diagnostic command accurately reports oscal-cli availability, version, and path in under 5 seconds.
- **SC-005**: Test coverage for the oscal-cli integration module exceeds 80%, with 100% coverage of graceful degradation and error handling paths.
- **SC-006**: The resolved Catalog output matches the result of invoking oscal-cli directly (FORGE adds no transformation to oscal-cli output).
