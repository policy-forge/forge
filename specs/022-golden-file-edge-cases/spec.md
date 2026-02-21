# Feature Specification: Golden File Edge Cases

**Feature Branch**: `022-golden-file-edge-cases`  
**Created**: 2026-02-21  
**Status**: Ready for Implementation  
**Input**: User description: "use docs/PRD/022-prd-golden-file-edge-cases.md to start working on the next SPEC. The worktree/branch should start with 022 since that is the PRD #."

## Clarifications

### Session 2026-02-21

- Q: What exact defaults should FR-006 use for missing metadata (title/version/author)? → A: `version = "0.0.0"`, `title = input filename stem`, `author = "Unknown"`, with one warning per missing field.
- Q: Which edge cases are "applicable" for dual-strategy validation in FR-011/SC-005? → A: Dual-strategy applies to EC-1, EC-2, EC-3, EC-4, EC-5, EC-6, EC-7, and EC-10; EC-9 is strategy-agnostic and validated once.
- Q: How should "substantively changed requirement" be defined for FR-008/SC-004? → A: Any non-whitespace change in normative requirement text is substantive and triggers a new stable identifier.
- Q: What canonical indicator should FR-009 require for malformed citations? → A: Preserve citation and attach `prop name="url-status"` with value `unvalidated` on the back-matter resource.
- Q: How should FR-003 define "descriptive failure feedback" so tests stay stable while wording evolves? → A: Include cause category, offending input/path, and one remediation hint; tests assert required substrings rather than exact full messages.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Actionable Failure Feedback (Priority: P1)

A compliance engineer submits flawed or missing policy input and needs immediate, clear feedback about why conversion cannot proceed.

**Why this priority**: If users cannot understand failure causes, they cannot fix inputs and trust in the conversion workflow drops quickly.

**Independent Test**: Evaluate headingless and missing-file edge case scenarios and confirm each returns a clear, actionable failure outcome.

**Acceptance Scenarios**:

1. **Given** policy content with no identifiable section headings, **When** conversion is requested, **Then** conversion fails with a message that includes the cause category, the offending input/path, and one remediation hint.
2. **Given** a file path that does not exist, **When** conversion is requested, **Then** conversion fails with a message that includes the cause category, the offending input/path, and one remediation hint.

---

### User Story 2 - Boundary Input Handling (Priority: P1)

A compliance engineer converts imperfect but legitimate policy content and expects robust handling of compound statements, empty sections, and missing metadata.

**Why this priority**: These conditions are common in real policy documents and must be handled consistently to avoid false failures and rework.

**Independent Test**: Evaluate compound/atomic, empty-section, and missing-metadata scenarios and compare outcomes with approved expected results.

**Acceptance Scenarios**:

1. **Given** content containing both compound and atomic normative statements, **When** conversion is requested, **Then** compound statements are separated into distinct requirements while atomic statements remain single requirements.
2. **Given** sections with no normative statements, **When** conversion is requested, **Then** those sections are represented as empty result groups and a warning is produced.
3. **Given** missing title, version, or author metadata, **When** conversion is requested, **Then** defaults are applied as `title = input filename stem`, `version = "0.0.0"`, `author = "Unknown"`, and one warning is emitted per missing field.

---

### User Story 3 - Stable Identifier Integrity (Priority: P1)

A compliance engineer re-converts revised policy text and needs identifier behavior that distinguishes formatting-only edits from true requirement changes.

**Why this priority**: Reliable identifier behavior is essential for traceability, audit review, and meaningful document diffs.

**Independent Test**: Compare identifier outcomes across whitespace-only variants and substantive-content variants of the same source.

**Acceptance Scenarios**:

1. **Given** two source variants that differ only by whitespace, **When** both are converted, **Then** all stable identifiers remain unchanged.
2. **Given** two source variants where normative requirement text differs by any non-whitespace change, **When** both are converted, **Then** the changed requirement receives a new stable identifier and a warning is produced.

---

### User Story 4 - Consistent Edge Case Coverage Across Strategies (Priority: P2)

A compliance engineer expects edge case outcomes to remain consistent across both supported conversion strategies, including malformed citation and validation-error scenarios.

**Why this priority**: Strategy-specific inconsistencies create unpredictable behavior and increase downstream verification effort.

**Independent Test**: Execute each applicable edge case scenario through both conversion strategies and compare each result to its approved expected outcome.

**Acceptance Scenarios**:

1. **Given** malformed citation content, **When** conversion is requested, **Then** citation data is retained and its back-matter resource includes `prop name="url-status" value="unvalidated"` rather than being silently discarded.
2. **Given** a scenario containing multiple validation issue types, **When** validation is requested, **Then** all detected issues are reported in one result set.
3. **Given** EC-1, EC-2, EC-3, EC-4, EC-5, EC-6, EC-7, or EC-10, **When** the scenario is evaluated under both strategies, **Then** each strategy matches its approved expected outcome with no unexplained behavioral divergence.

---

### Edge Cases

- Documents with policy prose but no identifiable headings.
- Compound requirements joined by conjunctions and mixed with atomic requirements.
- Sections with zero normative requirements.
- Simultaneously missing metadata fields (title, version, author).
- Whitespace-only edits (spacing, blank lines, trailing spaces).
- Substantive requirement text edits (any non-whitespace change).
- Malformed citation strings that cannot be validated as reliable references.
- Missing source files.
- Multiple validation issue categories present in the same output.
- Applicable scenarios evaluated under both supported conversion strategies.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The specification MUST cover each applicable parent edge case from EC-1 through EC-10, excluding EC-8.
- **FR-002**: Each covered edge case MUST define an expected result type: successful output, warning-inclusive output, or descriptive failure.
- **FR-003**: For headingless source content and missing source files, the system MUST provide failure feedback that includes a cause category, the offending input/path, and one remediation hint; validation MUST assert required substrings rather than exact full-message equality.
- **FR-004**: The system MUST preserve atomic requirements and split compound requirements into independently addressable requirements.
- **FR-005**: The system MUST represent sections with no normative requirements without failing conversion.
- **FR-006**: When metadata is missing, the system MUST apply `title = input filename stem`, `version = "0.0.0"`, and `author = "Unknown"`, and emit one warning per missing field.
- **FR-007**: Stable identifiers MUST remain unchanged for whitespace-only source changes.
- **FR-008**: Stable identifiers MUST change when normative requirement text has any non-whitespace change, and the change MUST be disclosed via warning output.
- **FR-009**: Malformed citation content MUST be retained, and its back-matter resource MUST include `prop name="url-status" value="unvalidated"` rather than being silently removed.
- **FR-010**: When multiple validation issue types are present, the system MUST report all of them in a single validation result.
- **FR-011**: EC-1, EC-2, EC-3, EC-4, EC-5, EC-6, EC-7, and EC-10 MUST each be validated under both supported conversion strategies; EC-9 is strategy-agnostic and MUST be validated once.
- **FR-012**: Scope MUST explicitly exclude scanned-document edge cases and performance benchmarking from this feature.

### Assumptions

- The core golden-file baseline from the preceding work item is available for extension.
- Parent edge-case definitions remain the authoritative scope source for this feature.
- Parallel error-handling work will provide finalized message patterns before release readiness review.

### Dependencies

- Completion of the core golden-file suite that defines baseline fixtures and expected outcomes.
- Availability of validation behavior needed to surface multiple issue types in one run.
- Coordination with the Phase 1 release readiness track that consumes this edge case coverage.

### Key Entities *(include if feature involves data)*

- **Edge Case Scenario**: A traceable boundary-condition test case with a source policy sample, an edge-case identifier, and a declared expected outcome type.
- **Expected Outcome Record**: The approved reference result for a scenario, including output content, warning expectations, or failure expectations.
- **Stability Comparison Pair**: Two related source variants used to verify identifier continuity rules for formatting-only and substantive edits.
- **Strategy Coverage Pair**: A matched set of expected outcomes for the same scenario under each supported conversion strategy.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of applicable parent edge cases (9 of 9) have approved scenarios with expected outcomes.
- **SC-002**: 100% of P1 acceptance scenarios pass in pre-planning validation.
- **SC-003**: 100% of whitespace-only stability comparison pairs preserve all stable identifiers.
- **SC-004**: 100% of non-whitespace-change stability comparison pairs assign a new identifier to each changed requirement.
- **SC-005**: 100% of strategy-applicable edge case scenarios (EC-1, EC-2, EC-3, EC-4, EC-5, EC-6, EC-7, EC-10) show approved outcomes under both supported conversion strategies, and EC-9 passes its single strategy-agnostic validation.
- **SC-006**: In stakeholder review, at least 95% of failure scenarios are rated actionable (reviewers identify a remediation step within 5 minutes).
