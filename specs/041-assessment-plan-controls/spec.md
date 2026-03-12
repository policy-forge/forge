# Feature Specification: Assessment Plan Scaffolding — Controls

**Feature Branch**: `041-assessment-plan-controls`
**Created**: 2026-03-12
**Status**: Draft
**Input**: Derived from 041-prd-assessment-plan-controls, 041-ar-assessment-plan-controls, 041-sec-assessment-plan-controls

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Generate Assessment Plan with Reviewed Controls (Priority: P1)

A compliance engineer generates an Assessment Plan skeleton that lists all controls from their converted policy as reviewed-controls, giving assessors a structured, complete starting point.

> As a compliance engineer, I want to generate an Assessment Plan skeleton with reviewed-controls populated from my policy conversion output so that assessors have a structured starting point that covers all policy-derived controls.

**Why this priority**: This is the core deliverable — producing the reviewed-controls structure that defines the scope of assessment. Without it the Assessment Plan has no control coverage and delivers no value to assessors.

**Independent Test**: Convert a policy with 10 controls and generate an Assessment Plan. Verify that the reviewed-controls section contains all 10 control identifiers and is structurally valid.

**Acceptance Scenarios**:

1. **Given** a conversion output with 10 controls and an SSP reference path, **When** generating the Assessment Plan, **Then** the output contains a reviewed-controls section listing all 10 control identifiers.
2. **Given** a conversion output with controls, **When** generating the Assessment Plan, **Then** the output includes required document metadata: a title, a last-modified timestamp, a version, and the OSCAL version identifier.
3. **Given** a policy titled "Corporate Security Policy", **When** generating the Assessment Plan, **Then** the reviewed-controls section includes a description referencing "Corporate Security Policy".

---

### User Story 2 — Link the SSP Reference (Priority: P1)

A compliance engineer specifies the System Security Plan that the Assessment Plan will reference, so the generated artifact correctly links to the system context being assessed.

> As a compliance engineer, I want to specify the SSP reference for the Assessment Plan so that the generated artifact correctly links to the system context being assessed.

**Why this priority**: The SSP reference is a required structural element of the OSCAL Assessment Plan — without it the plan has no system context and is structurally incomplete.

**Independent Test**: Generate an Assessment Plan with a specific SSP path and verify the output's SSP reference field matches that path exactly. Also verify that providing an empty string as the SSP path produces a clear, actionable error.

**Acceptance Scenarios**:

1. **Given** an SSP path of `./ssp/system-ssp.json`, **When** generating the Assessment Plan, **Then** the output's SSP reference field equals `"./ssp/system-ssp.json"`.
2. **Given** no SSP path is provided, **When** running `forge convert`, **Then** AP generation is skipped and the command completes normally without producing an Assessment Plan file.
3. **Given** an empty string provided as the SSP path, **When** attempting to generate an Assessment Plan, **Then** a descriptive error message is shown indicating the SSP path is invalid.

---

### User Story 3 — Deterministic Assessment Plan UUIDs (Priority: P2)

A developer re-generates an Assessment Plan from the same input and receives identical identifiers, enabling meaningful diffs and stable references across runs.

> As a developer working on FORGE, I want Assessment Plan identifiers to be deterministic so that re-generating from the same input produces identical values, enabling meaningful diffs and stable references in downstream tools.

**Why this priority**: Identifier stability is a cross-cutting quality requirement maintained across all generated OSCAL artifacts. Without it, each re-generation breaks downstream references and makes diffs noisy.

**Independent Test**: Generate an Assessment Plan from the same input twice and verify all identifiers are identical across both runs. Then change one control in the input and verify the relevant identifiers update accordingly.

**Acceptance Scenarios**:

1. **Given** the same conversion output and SSP reference, **When** generating the Assessment Plan twice, **Then** all identifiers are identical across both runs.
2. **Given** a change in the control set, **When** re-generating the Assessment Plan, **Then** the document-level identifier and affected control-selection identifiers reflect the changed input.

---

### Edge Cases

- When the conversion output contains zero controls, the Assessment Plan file is still written with an empty `include-controls` array and a warning is emitted indicating there are no controls to assess.
- When the conversion output contains duplicate control identifiers, the reviewed-controls section lists each identifier only once (duplicates are removed).
- When the SSP path value is an empty string, the generator exits with a descriptive error rather than producing an Assessment Plan with an empty reference.
- When a control identifier contains special characters, it is preserved exactly in the output without modification.
- When the conversion output changes, the document-level identifier changes accordingly — it is not fixed.
- When `--import-ssp` is provided alongside batch-mode inputs (2+ input files), AP generation is skipped with a warning and batch conversion completes normally without producing an Assessment Plan file.

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST generate an Assessment Plan artifact with the correct root structure conforming to the OSCAL Assessment Plan model.
- **FR-002**: The Assessment Plan MUST include required document metadata: a document identifier, title, last-modified timestamp, version (set to `"1.0.0"`), and the OSCAL specification version (set to `"1.2.0"`).
- **FR-003**: The Assessment Plan MUST include an SSP reference field set to the value provided by the user via the SSP path option.
- **FR-004**: The `--import-ssp` flag is optional; when omitted, AP generation is skipped and `forge convert` completes normally (backward compatible, per plan.md D-5). When the flag is provided, its value MUST be non-empty.
- **FR-011**: Assessment Plan generation is triggered by passing `--import-ssp <path>` to the existing `convert` command; no separate subcommand is introduced. *(Derived from clarification session 2026-03-12; no corresponding PRD M-# — extends PRD scope.)*
- **FR-012**: The Assessment Plan JSON file MUST be written to the same output directory as the converted artifact, using the filename `<policy-stem>-assessment-plan.json` (where `<policy-stem>` is the input policy filename without extension). *(Derived from clarification session 2026-03-12; no corresponding PRD M-# — extends PRD scope.)*
- **FR-005**: The Assessment Plan MUST include a reviewed-controls section containing a control-selections list.
- **FR-006**: The control-selections list MUST be populated with all control identifiers from the conversion output.
- **FR-007**: Control identifiers MUST be deduplicated before being included in the control-selections list.
- **FR-008**: All document and element identifiers MUST be generated deterministically — the same input must always produce the same identifiers.
- **FR-009**: The reviewed-controls section SHOULD include a description summarising the assessment scope, referencing the source policy title.
- **FR-010**: The metadata assembly SHOULD be consistent with the metadata pattern used across other FORGE-generated OSCAL artifacts.

### Key Entities

- **Assessment Plan**: The top-level OSCAL artifact generated by this feature. Contains document metadata, an SSP reference, and a reviewed-controls section. Identified by a deterministic document-level identifier.
- **SSP Reference**: A link to the System Security Plan being assessed. Contains only the reference path — no SSP file content is read or embedded.
- **Reviewed Controls**: A container within the Assessment Plan that defines the scope of the assessment. Contains one or more control-selection groups.
- **Control Selection**: An entry within reviewed-controls that lists which control identifiers are included in the assessment scope. Populated from the conversion pipeline output.

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of control identifiers from the conversion output appear in the Assessment Plan's reviewed-controls — no controls are lost during the mapping.
- **SC-002**: The SSP reference field in the generated Assessment Plan matches the user-provided value exactly in every case.
- **SC-003**: Generating an Assessment Plan from the same input twice produces identical output — zero identifier differences across runs.
- **SC-004**: Omitting the SSP path causes AP generation to be skipped cleanly — `forge convert` completes normally and no AP file is produced.
- **SC-005**: The generated Assessment Plan is structurally consistent with the OSCAL Assessment Plan model, as verified against NIST reference examples.

---

## Clarifications

### Session 2026-03-12

- Q: How should Assessment Plan generation be exposed in the CLI — new `assess` subcommand vs. extension to `convert`? → A: Extend `convert` with `--import-ssp` flag; AP generated automatically alongside the Catalog/Component Definition when the flag is provided.
- Q: Where is the Assessment Plan JSON file written? → A: Same output directory as the converted artifact, using a derived filename `<policy-stem>-assessment-plan.json`.
- Q: What value should `metadata.version` carry in the generated Assessment Plan? → A: `"1.0.0"` — static initial version, consistent with other FORGE-generated OSCAL artifacts.
- Q: Should the AP file be written when the conversion produces zero controls? → A: Yes — always write the AP file with an empty `include-controls` array and emit a warning; never suppress the output file silently.

## Assumptions

- The conversion pipeline (producing Catalogs or Component Definitions) already extracts a list of control identifiers during its generation process, and those identifiers are available to pass to the Assessment Plan builder.
- The SSP reference is a simple path string provided by the user — no SSP file content is read, parsed, or validated at this stage.
- A single control-selections group containing all controls is sufficient for the initial scaffold; multiple selection groups are a future extension.
- The OSCAL v1.2.0 Assessment Plan model structure is stable and well-documented.
- This feature produces the control-selection portion of the Assessment Plan scaffold only; assessment tasks and subjects are deferred to WI-42.
