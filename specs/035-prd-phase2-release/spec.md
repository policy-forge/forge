# Feature Specification: Phase 2 Integration Testing & v0.2.0 Release

**Feature Branch**: `035-prd-phase2-release`
**Created**: 2026-02-19
**Status**: Active
**Input**: Derived from PRD `docs/PRD/035-prd-phase2-release.md` (WI-35)

---

## Clarifications

### Session 2026-02-19

- Q: Should the Component Definition ↔ XML round-trip test normalize (clear `control_implementations`) before semantic equivalence comparison, consistent with WI-28? → A: Yes — normalize before XML comparison, matching the WI-28 pattern.
- Q: For EC-3 (`--set-param` with non-existent param ID), should the integration test assert error or warning behavior? → A: Exit 0 with a warning on stderr (permissive: param is added to Profile regardless of whether the ID exists in the Catalog).
- Q: Should `integration_regression.rs` use new insta snapshots, structural assertions only, or be skipped in favour of existing `golden_file_tests.rs`? → A: Structural assertions only — assert key invariants (uuid present, oscal-version = "1.2.0", groups/controls non-empty) without duplicating the existing insta snapshot infrastructure.

---

## User Scenarios & Testing

### User Story 1 — Verify Multi-Format Round-Trip (Priority: P1)

A compliance engineer converts a policy to OSCAL in all three formats and confirms they are semantically equivalent.

> As a compliance engineer, I want to convert my policy to OSCAL in JSON, XML, and YAML formats and confirm that converting between formats preserves all content so that I can trust any format for my workflow.

**Why this priority**: Multi-format output (S-3, S-4) is a key Phase 2 deliverable. Round-trip fidelity is the fundamental correctness guarantee for format support.

**Independent Test**: Convert a sample policy to JSON, then use `forge export` to convert to XML and YAML, then convert each back to JSON and compare for semantic equivalence.

**Acceptance Scenarios**:

1. **Given** an OSCAL Catalog in JSON format, **When** converting to XML via `forge export` and back to JSON, **Then** the resulting JSON is semantically equivalent to the original.
2. **Given** an OSCAL Catalog in JSON format, **When** converting to YAML via `forge export` and back to JSON, **Then** the resulting JSON is semantically equivalent to the original.
3. **Given** an OSCAL Component Definition in JSON format, **When** round-tripping through XML and YAML, **Then** semantic equivalence is preserved for both paths.

---

### User Story 2 — End-to-End Profile Generation with Tailoring (Priority: P1)

A compliance engineer generates an OSCAL Profile that selects controls and sets parameters from an existing Catalog.

> As a compliance engineer, I want to run `forge profile --catalog catalog.json --include POL-AC-001,POL-AC-002 --set-param password-length 16` and receive a valid OSCAL Profile so that I can define a tailored baseline for my organization.

**Why this priority**: Profile generation (S-5) with include/exclude and parameter setting is the MS-6 exit criteria.

**Independent Test**: Generate a Catalog from a sample policy, then generate a Profile selecting specific controls and setting parameters. Validate the Profile against the OSCAL schema.

**Acceptance Scenarios**:

1. **Given** a valid OSCAL Catalog with 10 controls, **When** running `forge profile --catalog catalog.json --include POL-AC-001,POL-AC-002 --format json`, **Then** a valid OSCAL Profile JSON is produced with `imports[].include-controls` referencing the specified control IDs.
2. **Given** a Profile generation with `--set-param password-length 16`, **When** inspecting the output, **Then** the Profile contains a `modify.set-parameters` entry with the specified parameter ID and value.
3. **Given** a generated Profile, **When** running `forge validate` against it, **Then** schema validation passes with zero errors.
4. **Given** Profile generation with `--exclude` instead of `--include`, **When** inspecting the output, **Then** the Profile contains `imports[].exclude-controls` referencing the excluded control IDs.

---

### User Story 3 — Verify Normative/Advisory Tagging Across Formats (Priority: P2)

A compliance engineer confirms that normative and advisory tagging is preserved in all output formats.

> As a compliance engineer, I want to see normative ("must"/"shall") and advisory ("should"/"may") requirements tagged with `prop` annotations in JSON, XML, and YAML output so that I can filter requirements by obligation level regardless of output format.

**Why this priority**: S-7 (normative/advisory detection) must work across all output formats.

**Independent Test**: Convert a policy with mixed normative and advisory language to all three formats and verify `prop` annotations are present and correct in each.

**Acceptance Scenarios**:

1. **Given** a policy containing "Systems must enforce MFA" and "Administrators should review logs", **When** converting to JSON, XML, and YAML, **Then** each output contains `prop` annotations with `name: "modality"` and values `"normative"` or `"advisory"`.
2. **Given** a round-trip through XML and back to JSON, **When** inspecting the normative/advisory `prop` annotations, **Then** they are preserved identically.

---

### User Story 4 — Verify Parameter Extraction Across Formats (Priority: P2)

A compliance engineer confirms that extracted parameters appear correctly in all output formats.

> As a compliance engineer, I want policy parameters (time windows, thresholds) to appear as OSCAL `param` elements in JSON, XML, and YAML output so that I can manage configurable values in my compliance toolchain regardless of format choice.

**Why this priority**: S-8 (parameter extraction) must be verified across all output formats.

**Independent Test**: Convert a policy with parameterized requirements to all three formats and verify `param` elements appear correctly in each.

**Acceptance Scenarios**:

1. **Given** a policy containing "Passwords must be changed within 90 days", **When** converting to JSON, XML, and YAML, **Then** each output contains an OSCAL `param` element representing the "90 days" time window.
2. **Given** a round-trip through YAML and back to JSON, **When** inspecting `param` elements, **Then** parameter IDs, values, and constraints are preserved.

---

### User Story 5 — Phase 1 Regression Verification (Priority: P1)

A developer confirms that all Phase 1 (v0.1.0) functionality still works correctly after Phase 2 changes.

> As a developer working on FORGE, I want to verify that all Phase 1 acceptance criteria (AC-1 through AC-10) still pass after Phase 2 development so that I can confirm v0.2.0 is a strict superset of v0.1.0 with no regressions.

**Why this priority**: A release that breaks existing functionality is unacceptable.

**Independent Test**: Run the full Phase 1 test suite and verify all tests pass. Run `forge convert` with Catalog and Component Definition strategies and verify key structural invariants hold (uuid present, oscal-version = "1.2.0", groups/controls non-empty). Insta-based golden-file regression is covered by the existing `golden_file_tests.rs`; `integration_regression.rs` uses structural assertions only.

**Acceptance Scenarios**:

1. **Given** the complete Phase 1 test suite, **When** running `cargo test`, **Then** all Phase 1 tests pass with zero failures.
2. **Given** a sample Markdown policy, **When** running `forge convert policy.md --strategy catalog --format json`, **Then** the output matches the Phase 1 golden-file baseline (accounting for expected additive changes from S-7/S-8).

---

### User Story 6 — Tag and Publish v0.2.0 (Priority: P1)

A developer tags the v0.2.0 release after all integration tests pass.

> As a developer working on FORGE, I want to tag v0.2.0 in the repository after all Phase 2 exit criteria are met so that there is a clear, reproducible release marker for the Phase 2 milestone.

**Why this priority**: The version tag is the MS-6 deliverable.

**Independent Test**: Verify the `v0.2.0` git tag exists, points to a commit where all tests pass, and `forge --version` reports `0.2.0`.

**Acceptance Scenarios**:

1. **Given** all integration tests passing, **When** creating the `v0.2.0` git tag, **Then** the tag is created on a commit where `cargo test` passes with zero failures and `cargo clippy -- -D warnings` reports zero warnings.
2. **Given** the tagged release, **When** running `forge --version`, **Then** the output includes `0.2.0`.

---

### Edge Cases

- **EC-1:** (M-1) When round-tripping a Catalog with empty groups (no controls), then semantic equivalence is still maintained across JSON, XML, and YAML.
- **EC-2:** (M-2) When generating a Profile with `--include` specifying a control ID that does not exist in the Catalog, then a descriptive error is produced.
- **EC-3:** (M-3) When using `--set-param` with a parameter ID that does not exist in the Catalog, then the command exits 0 and emits a warning on stderr. The parameter is still added to `modify.set-parameters` in the Profile output (permissive behavior; OSCAL Profile param IDs are independent of Catalog param IDs).
- **EC-4:** (M-5) When a control has both normative and advisory sub-statements (from atomization), then each resulting control has the correct individual `prop` annotation.
- **EC-5:** (M-1) When round-tripping a Component Definition through XML, then semantic equivalence is maintained for fields that XML preserves; `control_implementations` are excluded from comparison (XML intentionally omits this field — matches WI-28 normalization pattern). YAML round-trip preserves all fields including `control_implementations`.
- **EC-6:** (M-2) When generating a Profile with both `--include` and `--exclude` flags, then the behavior is well-defined per WI-30 design.
- **EC-7:** (M-6) When Phase 2 introduces additive changes to output (e.g., new `prop` annotations), then Phase 1 golden-file tests account for expected additive differences.

---

## Requirements

### Functional Requirements

**Must Have:**
- **M-1:** Multi-format round-trip tests shall pass for JSON→XML→JSON and JSON→YAML→JSON conversions, confirming semantic equivalence for both Catalog and Component Definition artifacts. *(S-3, S-4)*
- **M-2:** An end-to-end test shall verify `forge profile --catalog <path> --include <ids> --format json` produces a valid OSCAL Profile JSON with correct `imports[].include-controls`. *(S-5, AC-12)*
- **M-3:** An end-to-end test shall verify `forge profile --catalog <path> --set-param <id> <value>` produces a Profile with correct `modify.set-parameters` entries. *(S-5, AC-12)*
- **M-4:** Generated Profiles shall pass schema validation against OSCAL v1.2.0 Profile schemas via `forge validate`. *(S-5)*
- **M-5:** Normative/advisory `prop` annotations and `param` elements shall be present and correct in JSON, XML, and YAML output, and shall survive format round-trips. *(S-7, S-8, AC-13)*
- **M-6:** All Phase 1 tests shall pass with zero failures, confirming no regressions from Phase 2 development. *(AC-1 through AC-10)*
- **M-7:** The `v0.2.0` git tag shall be created on a commit where `cargo test` passes, `cargo clippy -- -D warnings` reports zero warnings, and `cargo fmt --check` reports zero violations.
- **M-8:** `forge --version` shall report version `0.2.0` in the tagged release.

**Should Have:**
- **S-1:** Integration tests shall cover Profile generation with `--exclude` (not just `--include`) to verify both selection modes.
- **S-2:** Integration tests shall cover Profile generation in XML and YAML formats (not just JSON).
- **S-3:** CHANGELOG shall be updated with a summary of all Phase 2 features for the v0.2.0 release.
- **S-4:** `--help` text for `forge profile`, `forge export`, and new flags introduced in Phase 2 shall be reviewed for clarity and completeness.

### Key Entities

No new data model introduced. WI-35 tests existing Phase 2 data models:
- **OSCAL Catalog**: Multi-group, multi-control with metadata, props, params
- **OSCAL Component Definition**: Components with control implementations
- **OSCAL Profile**: Imports (include/exclude controls) + modify (set-parameters)
- **PolicyRequirement.modality**: `Modality` enum (`Normative | Advisory | None`) → OSCAL `prop` with `name: "modality"`
- **PolicyRequirement.parameters**: `Vec<PolicyParameter>` → OSCAL `param` elements

---

## Success Criteria

### Measurable Outcomes

- **SC-001:** All 4 integration test categories pass (round-trip, profile E2E, cross-feature, regression).
- **SC-002:** 100% semantic equivalence maintained for Catalog and Component Definition round-trips through XML and YAML.
- **SC-003:** `forge profile` produces schema-valid OSCAL Profiles for include/exclude/set-param combinations.
- **SC-004:** Normative/advisory `prop` annotations survive XML and YAML round-trips with no loss.
- **SC-005:** `param` elements survive XML and YAML round-trips with values and constraints intact.
- **SC-006:** Zero test failures in `cargo test`, zero warnings in `cargo clippy -- -D warnings`, zero violations in `cargo fmt --check`.
- **SC-007:** `v0.2.0` git tag created on a verified commit; `forge --version` reports `0.2.0`.
