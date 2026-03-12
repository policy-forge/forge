# Feature Specification: oscal-cli Round-Trip Validation

**Feature Branch**: `037-oscal-cli-round-trip`
**Created**: 2026-03-12
**Status**: Draft
**Input**: Derived from 037-prd-oscal-cli-round-trip, 037-ar-oscal-cli-round-trip, 037-sec-oscal-cli-round-trip

## Clarifications

### Session 2026-03-12

- Q: What format and location should the divergence log use? → A: JSON file written to a configurable output path (default: `divergences.json` in the working directory)
- Q: Which OSCAL fields are treated as unordered arrays in semantic comparison? → A: `props`, `links`, `parts`
- Q: Use `assert_json_diff` crate or custom comparison logic? → A: Custom `serde_json::Value` recursive tree walker; no `assert_json_diff` dependency
- Q: What timeout applies to oscal-cli subprocess invocations? → A: 30 seconds per invocation; treat timeout as a hard error, not a skip
- Q: Is C-1 (`forge validate --round-trip` CLI flag) in scope for this WI? → A: Explicitly deferred to a later WI; tracked as a GitHub issue

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Verify FORGE Output Matches Reference Tool Conversion (Priority: P1)

A developer runs the round-trip validation suite to confirm that FORGE-generated OSCAL artifacts are interoperable with the NIST reference toolchain.

> As a developer working on FORGE, I want to automatically compare FORGE output against the NIST reference tool's canonical conversion so that I can identify and fix any divergences before users encounter interoperability issues.

**Why this priority**: This is the core deliverable of the feature. Without automated round-trip comparison, there is no systematic assurance that FORGE-generated artifacts will be correctly consumed by other OSCAL-compliant tools. This story must exist for any subsequent work to be meaningful.

**Independent Test**: Generate an OSCAL Catalog artifact with FORGE, convert it through the reference tool (JSON → XML → JSON), and compare the original output with the round-tripped result. Verify the comparison produces a clear pass/fail indication with divergence details.

**Acceptance Scenarios**:

1. **Given** a FORGE-generated OSCAL Catalog artifact, **When** converting it through the reference tool (JSON → XML → JSON), **Then** the round-tripped artifact is semantically equivalent to the original, or all divergences are reported with their locations.
2. **Given** a FORGE-generated Component Definition artifact, **When** converting it through the reference tool (JSON → XML → JSON), **Then** the round-tripped artifact is semantically equivalent to the original, or all divergences are reported with their locations.
3. **Given** two artifacts that differ only in field ordering, **When** running semantic comparison, **Then** the comparison reports no divergences (field ordering is an acceptable variation).

---

### User Story 2 — Document and Classify Divergences (Priority: P1)

A developer reviews documented divergences to determine whether FORGE or the reference tool is the source of the difference, and what action to take.

> As a developer working on FORGE, I want divergences between FORGE output and reference tool conversion to be clearly documented and classified so that I can prioritize fixes and communicate known differences to users.

**Why this priority**: Discovering divergences is only useful if they are understood and categorized. Without classification, a developer cannot determine which divergences require FORGE fixes versus which are reference tool behaviors or acceptable variations.

**Independent Test**: Run the round-trip validation against a known divergent artifact and verify the divergence report includes the field location, expected value, actual value, and a classification of the divergence type.

**Acceptance Scenarios**:

1. **Given** a round-trip comparison that identifies divergences, **When** reviewing the divergence report, **Then** each divergence includes the field location, expected value, actual value, and classification (FORGE fix needed / reference tool difference / acceptable variation).
2. **Given** a completed validation run, **When** reviewing the divergence log, **Then** every divergence has a resolution status (fixed, accepted, or reported upstream).

---

### User Story 3 — Round-Trip Across All Three OSCAL Formats (Priority: P2)

A developer validates that FORGE output survives a full three-format round-trip cycle, confirming format-agnostic interoperability.

> As a developer working on FORGE, I want round-trip validation across all three OSCAL serialization formats (JSON, XML, YAML) so that I can verify FORGE output is interoperable regardless of which format downstream tools consume.

**Why this priority**: The two-format round-trip (JSON → XML → JSON) validates the most common path. Full format coverage provides broader assurance but is not required for an initial passing baseline. Delivers additional confidence once the two-format validation is established.

**Independent Test**: Convert FORGE output through JSON → XML → YAML → JSON using the reference tool and verify semantic equivalence at the final step.

**Acceptance Scenarios**:

1. **Given** a FORGE-generated Catalog artifact, **When** converting JSON → XML → YAML → JSON via the reference tool, **Then** the final artifact is semantically equivalent to the original FORGE output.
2. **Given** a FORGE-generated Component Definition artifact, **When** performing the same full three-format round-trip, **Then** the final artifact is semantically equivalent to the original FORGE output.

---

### Edge Cases

- When the reference tool is not available in the environment, round-trip validation skips gracefully with a clear warning message — it does not count as a test failure.
- When a reference tool subprocess invocation exceeds 30 seconds, it is treated as a hard error (not a graceful skip) and fails the test with a timeout message.
- When FORGE output contains empty collections and the reference tool omits them entirely, the comparison classifies this as an acceptable variation rather than a divergence.
- When the reference tool reorders collection elements, the comparison handles unordered matching for `props`, `links`, and `parts` — the OSCAL fields where element order is not mandated.
- When a divergence occurs in a deeply nested field, the reported location path is complete and precise (for example: `catalog.groups[0].controls[2].parts[0].prose`).
- When FORGE output includes fields not recognized by the reference tool, the divergence is captured and reported with the full field path.

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST convert FORGE-generated OSCAL Catalog artifacts through a multi-format conversion cycle (at minimum JSON → XML → JSON) using the reference tool and compare the result with the original output.
- **FR-002**: The system MUST convert FORGE-generated Component Definition artifacts through the same multi-format conversion cycle and compare the result with the original output.
- **FR-003**: The comparison MUST use semantic equivalence — it MUST tolerate acceptable variations in field ordering and whitespace without reporting them as divergences. The following OSCAL array fields MUST be compared without regard to element order: `props`, `links`, `parts`.
- **FR-004**: Each divergence MUST be reported with: its field location, the expected value, the actual value, and a human-readable description. The `description` field is authored by the integration test investigator (not auto-generated by the comparator); the comparator emits an empty string as the initial value, which the investigator fills in during the reclassification pass.
- **FR-005**: All divergences where FORGE is non-conformant MUST be resolved (FORGE output corrected) before the feature is considered complete. A divergence may be reclassified as `OscalCliDiff` or `Acceptable` — rather than fixed — only after verifying via the OSCAL specification that the behavior is correct or expected; such reclassification requires a documented rationale in the divergence `description` field.
- **FR-006**: A divergence log MUST document all discovered divergences, their classification, and their resolution status. The log MUST be written as a JSON file to a configurable output path (default: `divergences.json` in the working directory).
- **FR-007**: The validation SHOULD support a full three-format round-trip cycle (JSON → XML → YAML → JSON) in addition to the two-format cycle.
- **FR-OUT-C1**: A `forge validate --round-trip` CLI flag is explicitly out of scope for this WI. It is deferred to a future work item (tracked as a GitHub issue).
- **FR-008**: The divergence log SHOULD classify each divergence as one of: "FORGE fix needed", "reference tool difference", or "acceptable variation".
- **FR-009**: The validation SHOULD execute as part of the automated test suite, conditioned on reference tool availability — skipping gracefully when the reference tool is not present.
- **FR-010**: Each reference tool subprocess invocation MUST have a 30-second timeout; exceeding this limit MUST be treated as a hard error, not a graceful skip.

### Key Entities

- **Round-Trip Result**: Represents the outcome of a single validation run for one artifact type. Captures: artifact type (Catalog or Component Definition), whether the run passed, and the list of divergences found.
- **Divergence**: A single difference between original FORGE output and the round-tripped artifact. Captures: field location, expected value, actual value, classification (FORGE fix / reference tool difference / acceptable variation), and resolution status (fixed / accepted / reported upstream).

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: FORGE-generated Catalog artifacts survive a JSON → XML → JSON round-trip with zero unresolved FORGE-caused divergences.
- **SC-002**: FORGE-generated Component Definition artifacts survive a JSON → XML → JSON round-trip with zero unresolved FORGE-caused divergences.
- **SC-003**: FORGE-generated Catalog and Component Definition artifacts survive a full JSON → XML → YAML → JSON round-trip with zero unresolved FORGE-caused divergences.
- **SC-004**: All discovered divergences are documented in the divergence log with classification and resolution status — 100% coverage, no undocumented divergences.
- **SC-005**: Automated validation executes without failure in environments where the reference tool is available, and skips cleanly without blocking in environments where it is not.

---

## Assumptions

- The reference tool (oscal-cli) is installed and available in the development and CI environment, as established by the preceding integration work (WI-36).
- The reference tool supports JSON → XML, XML → YAML, and YAML → JSON conversions for both Catalog and Component Definition models.
- When FORGE and the reference tool produce different output, FORGE is assumed non-conformant unless investigation proves otherwise.
- Semantic equivalence comparison can tolerate field ordering differences and whitespace variations, focusing only on structural and value-level differences.
- The comparison implementation uses a custom `serde_json::Value` recursive tree walker; the `assert_json_diff` crate is NOT used.
- `run_round_trip_chain` always performs all three conversion steps (JSON → XML → YAML → JSON). User Story 1 and User Story 3 use the same chain; the distinction is that US3 (SC-003) explicitly verifies the three-format outcome rather than just the pass/fail result of a two-format subset.
