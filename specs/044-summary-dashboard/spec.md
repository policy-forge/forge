# Feature Specification: Summary Dashboard

**Feature Branch**: `044-summary-dashboard`
**Created**: 2026-03-10
**Status**: Draft
**Input**: User description: "Summary dashboard for forge convert --summary flag showing conversion statistics"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - View Conversion Statistics (Priority: P1)

A compliance engineer runs `forge convert` with the `--summary` flag and sees a concise summary of what the conversion pipeline produced: how many sections were parsed, how many requirements were extracted, and how many controls were generated. This gives immediate feedback on conversion completeness without manually inspecting the output artifact.

**Why this priority**: This is the core value of the feature. Without statistics collection and display, the dashboard has nothing to show. All other stories build on this foundation.

**Independent Test**: Run `forge convert policy.md --strategy catalog --summary` on a known policy document and verify that stdout includes counts for sections parsed, requirements extracted, and controls generated.

**Acceptance Scenarios**:

1. **Given** a policy document with 5 sections and 15 requirements, **When** running `forge convert policy.md --strategy catalog --summary`, **Then** stdout shows sections parsed as 5, requirements extracted as 15, and controls generated as the actual count.
2. **Given** a successful conversion with `--summary`, **When** viewing stdout, **Then** the statistics are printed after the output file path confirmation as a distinct formatted section.
3. **Given** a conversion run without `--summary`, **When** viewing stdout, **Then** no dashboard is printed and behavior is unchanged from current.

---

### User Story 2 - View Validation Status (Priority: P1)

A compliance engineer sees whether the generated OSCAL artifact passed schema validation as part of the summary, eliminating the need for a separate validation step.

**Why this priority**: Validation status is a critical quality indicator. Users need to know immediately whether the output is usable by downstream compliance tools.

**Independent Test**: Run `forge convert policy.md --strategy catalog --summary` and verify that stdout includes validation status (e.g., "PASSED", "FAILED (3 errors)", or "Not run").

**Acceptance Scenarios**:

1. **Given** a conversion that produces valid OSCAL, **When** using `--summary`, **Then** stdout shows validation status as "PASSED".
2. **Given** a conversion that produces OSCAL with validation warnings, **When** using `--summary`, **Then** stdout shows validation status as "PASSED with N warnings".
3. **Given** validation infrastructure is not available, **When** using `--summary`, **Then** stdout shows validation status as "Not run".

---

### User Story 3 - View Mapping Coverage (Priority: P2)

A compliance engineer sees what percentage of extracted requirements were successfully mapped to OSCAL representations, highlighting completeness gaps that need attention.

**Why this priority**: Mapping coverage provides a quality signal — 100% means every requirement became a control or implemented-requirement, while lower coverage indicates gaps needing investigation. Less critical than raw counts and validation status but valuable for iterative refinement workflows.

**Independent Test**: Convert a policy where 12 of 15 requirements produce controls, run with `--summary`, and verify stdout shows the coverage percentage with raw counts.

**Acceptance Scenarios**:

1. **Given** a conversion where 12 of 15 requirements produce OSCAL controls, **When** using `--summary`, **Then** stdout shows mapping coverage as "80.0% (12/15)".
2. **Given** a conversion where all requirements produce controls, **When** using `--summary`, **Then** stdout shows mapping coverage as "100.0%".
3. **Given** a conversion where zero requirements are extracted (empty document), **When** using `--summary`, **Then** stdout shows mapping coverage as "0.0% (0/0)" with a warning.

---

### User Story 4 - View Conversion Context (Priority: P2)

A compliance engineer sees the conversion strategy used and output file path in the summary, providing complete context alongside the statistics.

**Why this priority**: Strategy and output path provide context that helps interpret the statistics. Useful but not essential for quality assessment.

**Independent Test**: Run `forge convert policy.md --strategy catalog --summary` and verify the dashboard includes the strategy name and output file path.

**Acceptance Scenarios**:

1. **Given** a conversion with `--strategy catalog`, **When** using `--summary`, **Then** the dashboard shows "catalog" as the strategy.
2. **Given** a conversion that writes to `output/catalog.json`, **When** using `--summary`, **Then** the dashboard shows the output file path.

---

### Edge Cases

- What happens when zero requirements are extracted (empty or unparseable document)? Mapping coverage shows "0.0% (0/0)" with a warning; other statistics show 0.
- What happens when the conversion fails with an error before completion? The summary is not printed; only the error message is shown.
- What happens when `--strategy component` is used? "Controls generated" reflects implemented-requirements count, not Catalog controls.
- What happens when atomization produces more controls than input requirements? Mapping coverage can exceed 100% and is displayed as-is (e.g., "120.0% (18/15)").
- What happens when `--summary` is combined with different output formats (XML, YAML)? The summary is always plain text to stdout regardless of output format.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST accept a `--summary` flag on the `forge convert` subcommand.
- **FR-002**: When `--summary` is provided, the system MUST print the number of sections parsed from the input document to stdout.
- **FR-003**: When `--summary` is provided, the system MUST print the number of requirements extracted to stdout.
- **FR-004**: When `--summary` is provided, the system MUST print the number of controls generated (Catalog controls or Component Definition implemented-requirements, depending on strategy) to stdout.
- **FR-005**: When `--summary` is provided, the system MUST print the validation status (passed, passed with warnings, failed with error count, or not run) to stdout.
- **FR-006**: When `--summary` is provided, the system MUST print the mapping coverage percentage (controls generated / requirements extracted * 100) to stdout, along with raw counts.
- **FR-007**: The summary dashboard MUST be printed after the conversion artifact is written to the output file, as a visually distinct formatted section.
- **FR-008**: When `--summary` is not provided, no dashboard output MUST be printed and conversion behavior MUST be unchanged.
- **FR-009**: The `--summary` flag MUST NOT alter the conversion pipeline's behavior — the generated artifact MUST be identical with or without the flag.
- **FR-010**: The summary dashboard SHOULD include the conversion strategy used (catalog or component).
- **FR-011**: The summary dashboard SHOULD include the output file path.
- **FR-012**: When zero requirements are extracted, the system MUST handle the mapping coverage calculation without errors and display "0.0% (0/0)".

### Key Entities

- **ConversionStatistics**: Represents aggregate counts collected during a single conversion run — sections parsed, requirements extracted, controls generated, validation status (passed/failed/warnings/not run), validation error and warning counts, conversion strategy name, and output file path.
- **ValidationStatus**: Represents the outcome of OSCAL schema validation — one of: passed, passed with warnings, failed, or not run.
- **Mapping Coverage**: A derived metric calculated as (controls generated / requirements extracted) * 100, representing the percentage of requirements with an OSCAL representation.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can assess conversion quality at a glance within 5 seconds of conversion completion when using `--summary`.
- **SC-002**: All statistics (sections parsed, requirements extracted, controls generated) match the actual pipeline behavior with 100% accuracy for any input document.
- **SC-003**: Mapping coverage percentage matches manual calculation (controls / requirements * 100) for all test fixtures.
- **SC-004**: Validation status accurately reflects the actual validation result in all cases (passed, warnings, failed, not available).
- **SC-005**: The summary dashboard is visually distinct and readable, with clear labels and aligned values.
- **SC-006**: Conversion performance is not degraded — the `--summary` flag adds negligible overhead (statistics collection is counter increments, not additional processing).
- **SC-007**: Default behavior (without `--summary`) is completely unchanged — no output differences, no behavioral changes.

## Assumptions

- The conversion pipeline can be instrumented to collect statistics at stage boundaries (after ingestion, after extraction, after generation, after validation) without a separate analysis pass.
- Validation infrastructure from previous work items (WI-19+) is available, or the system gracefully falls back to "Not run" status.
- The `--summary` flag does not affect artifact output — statistics are purely additive stdout content printed after the artifact is written to file.
- Mapping coverage is calculated as a simple ratio (controls / requirements); some requirements may legitimately produce more or fewer than one control due to atomization.
- No new external dependencies are needed — formatting uses standard string operations with box-drawing Unicode characters.

## Dependencies

- **Requires**: WI-35 (Phase 2 integration testing) — ensures the conversion pipeline is complete and instrumentation points are stable.
- **Parallel With**: WI-36 (oscal-cli integration), WI-40 (batch conversion), WI-41 (Assessment Plan controls), WI-43 (diff report).

## Scope Boundaries

**In Scope:**
- `--summary` flag on `forge convert`
- Collecting and displaying: sections parsed, requirements extracted, controls generated, validation status, mapping coverage
- Human-readable formatted text output to stdout

**Out of Scope:**
- Persistent storage of statistics (database or file) — stdout only
- Historical tracking or trend analysis across runs — single-run statistics only
- Web-based or GUI dashboard — CLI stdout only
- Performance benchmarking or timing metrics — content statistics focus
- Structured JSON output of statistics (deferred as a future enhancement)
- Automatic remediation suggestions based on low coverage — report only
