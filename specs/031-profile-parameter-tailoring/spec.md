# Feature Specification: Profile Parameter Tailoring

**Feature Branch**: `031-profile-parameter-tailoring`
**Created**: 2026-02-18
**Status**: Draft
**Input**: User description: "use the following documents docs/PRD/031-prd-profile-parameter-tailoring.md, docs/AR/031-ar-profile-parameter-tailoring.md and docs/SEC/031-sec-profile-parameter-tailoring.md to create the specs and plan."

## Clarifications

### Session 2026-02-18

- Q: Should the `modify` section's serialization be explicitly tested for XML and YAML output formats, or is JSON the only format in scope for WI-31's test coverage? → A: JSON only — XML/YAML are inherited via the serialization layer and need no explicit WI-31 test cases.
- Q: When C-2 is implemented and `--set-param` is used without `--include` or `--exclude`, should the command succeed or fail? → A: Non-fatal — emit a warning to stderr, generate the Profile, exit 0.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Set a Single Parameter Value in a Profile (Priority: P1)

A compliance engineer generates a Profile that overrides one default parameter value from the source catalog to match their organization's requirements. For example, they want to change the password rotation interval from the catalog default of "90 days" to their organization's standard of "60 days."

**Why this priority**: This is the core deliverable of the feature. Without the ability to set at least one parameter value, the `--set-param` flag provides no value. It directly fulfills the parent requirement for parameter tailoring in OSCAL baseline generation.

**Independent Test**: Run `forge profile --catalog catalog.json --include POL-AC-001 --set-param POL-AC-001_prm "60 days" --format json` and verify the Profile JSON output contains a `modify` section with a `set-parameters` array holding exactly one entry for `POL-AC-001_prm` with value `"60 days"`.

**Acceptance Scenarios**:

1. **Given** a Profile generation command with `--set-param POL-AC-001_prm "60 days"`, **When** the Profile is generated, **Then** the output JSON contains a `modify.set-parameters` array with `{ "param-id": "POL-AC-001_prm", "values": ["60 days"] }`.
2. **Given** a Profile generation command with no `--set-param` flags, **When** the Profile is generated, **Then** the output JSON does not contain a `modify` section, and output is identical to pre-tailoring behavior.

---

### User Story 2 - Set Multiple Parameters in a Single Command (Priority: P1)

A compliance engineer needs to tailor several parameter values in one Profile generation command. Running a separate command per parameter would be impractical for real-world baselines that commonly require many overrides at once.

**Why this priority**: Real-world compliance baselines require multiple simultaneous parameter overrides. Supporting many `--set-param` flags in one command is essential for practical usability and is an explicit requirement of the parent PRD.

**Independent Test**: Run `forge profile --catalog catalog.json --include POL-AC-001,POL-IR-001 --set-param POL-AC-001_prm "60 days" --set-param POL-IR-001_prm "4 hours" --format json` and verify both overrides appear as separate entries in the `modify.set-parameters` array.

**Acceptance Scenarios**:

1. **Given** a command with two `--set-param` flags for distinct parameter IDs, **When** the Profile is generated, **Then** the `modify.set-parameters` array contains exactly two entries with the correct `param-id` and `values` for each.
2. **Given** a command with three `--set-param` flags for distinct parameter IDs, **When** the Profile is generated, **Then** all three parameter overrides appear in the `modify.set-parameters` array.
3. **Given** a command with two `--set-param` flags for the same parameter ID, **When** the Profile is generated, **Then** exactly one `set-parameters` entry is produced for that ID, with both values combined in the `values` array.

---

### User Story 3 - Generated Profile Is Structurally Valid OSCAL (Priority: P1)

A developer integrating FORGE into a compliance pipeline needs the generated Profile with a `modify` section to be structurally valid OSCAL so that downstream tools can consume it without errors.

**Why this priority**: Structural validity is a prerequisite for downstream tool compatibility and for the upcoming Profile validation work. An invalid Profile structure breaks every consumer of FORGE output.

**Independent Test**: Generate a Profile with one or more `--set-param` flags and inspect the JSON structure. Verify that `modify` appears as a direct child of the `profile` root object (sibling of `imports` and `metadata`), and that each `set-parameters` entry contains exactly the `param-id` (string) and `values` (array of strings) fields required by the OSCAL Profile model.

**Acceptance Scenarios**:

1. **Given** a Profile generated with `--set-param` flags, **When** inspecting the JSON structure, **Then** the `modify` section is a direct child of the `profile` root object, at the same level as `imports` and `metadata`.
2. **Given** a Profile generated with `--set-param` flags, **When** inspecting each `set-parameters` entry, **Then** each entry contains a `param-id` field (string) and a `values` field (array of strings), conforming to the OSCAL Profile model.

---

### Edge Cases

- What happens when a `--set-param` value contains spaces (e.g., `"60 days"`)? The value must be preserved as a single string in the `values` array — not split on whitespace.
- What happens when the same parameter ID is provided twice via two `--set-param` flags with different values? Both values are aggregated into a single `set-parameters` entry with a combined `values` array (e.g., `values: ["val1", "val2"]`).
- What happens when no `--set-param` flags are provided alongside `--include` or `--exclude`? The Profile is generated with the `imports` section only and no `modify` section, preserving the behavior of the prior release.
- What happens when a `--set-param` value is an empty string? The entry is still generated with `values: [""]`; an empty string is a valid OSCAL parameter value.
- What happens when ten `--set-param` flags are provided with distinct parameter IDs? All ten entries appear in the `set-parameters` array.
- What ordering do multiple `set-parameters` entries follow? Entries are ordered alphabetically by `param-id`, producing deterministic output across repeated invocations with the same inputs.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The `forge profile` subcommand MUST accept a repeatable `--set-param <id> <value>` flag, where each occurrence takes exactly two arguments: a parameter identifier and a value string.
- **FR-002**: When one or more `--set-param` flags are provided, the generated Profile MUST include a `modify` section containing a `set-parameters` array.
- **FR-003**: Each entry in the `set-parameters` array MUST contain a `param-id` field (the parameter identifier) and a `values` field (an array of one or more strings).
- **FR-004**: Multiple `--set-param` flags with distinct parameter IDs MUST produce one entry per distinct ID in the `set-parameters` array.
- **FR-005**: The `modify` section MUST be positioned as a direct child of the `profile` root object, at the same level as `imports` and `metadata`.
- **FR-006**: When no `--set-param` flags are provided, the generated Profile MUST NOT include a `modify` section, preserving backward compatibility with the prior release.
- **FR-007**: When multiple `--set-param` flags specify the same parameter ID, their values MUST be aggregated into a single `set-parameters` entry with a combined `values` array.
- **FR-008**: The `set-parameters` entries MUST be ordered deterministically (alphabetically by `param-id`) so that the same inputs always produce identical output.
- **FR-009** *(C-2, optional)*: When `--set-param` flags are provided without any `--include` or `--exclude` flags, the command SHOULD emit a non-fatal warning to stderr and continue generating the Profile (exit 0).

### Key Entities *(include if feature involves data)*

- **Profile**: An OSCAL document that selects controls from one or more catalogs and tailors them into a baseline. Contains `uuid`, `metadata`, `imports`, and optionally a `modify` section.
- **Modify**: The section within a Profile that holds amendments to imported controls. Contains a `set-parameters` array.
- **SetParameter**: A single tailoring instruction within the `modify` section. Identified by a `param-id` and carries one or more override `values`.
- **Parameter ID**: An opaque string identifier referencing a parameter defined in the source catalog. Treated as a pass-through by FORGE — not validated against the catalog at this stage.
- **Parameter Value**: A string override for the default value of a catalog parameter (e.g., `"60 days"`, `"quarterly"`). May contain spaces. Treated as an opaque string.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A compliance engineer can generate a Profile with parameter overrides using a single `forge profile` command with one or more `--set-param` flags, without additional post-processing steps.
- **SC-002**: All `--set-param` values containing spaces (e.g., `"60 days"`) are preserved intact in the output — zero values are truncated or split.
- **SC-003**: When ten `--set-param` flags with distinct parameter IDs are provided, all ten entries appear in the generated Profile's `set-parameters` array — 100% of specified overrides are present.
- **SC-004**: Repeated invocations with identical inputs produce byte-for-byte identical Profile output — deterministic output rate is 100%.
- **SC-005**: A Profile generated without `--set-param` flags is identical in structure and content to a Profile generated by the prior release — zero regressions in existing behavior.
- **SC-006**: The generated Profile with a `modify` section passes structural inspection confirming all required OSCAL fields (`param-id`, `values`) are present in every `set-parameters` entry — 0 malformed entries.

## Assumptions

- The Profile generation core (WI-30 — control inclusion/exclusion via the `imports` section) is already complete and provides the base Profile structure.
- Each `--set-param` occurrence accepts exactly two arguments: a parameter identifier and a value string. No key=value syntax or file-based bulk input is required in this release.
- Parameter IDs are opaque strings; FORGE does not validate them against the source catalog at this stage. Catalog-aware validation is a future concern.
- The OSCAL v1.2.0 Profile `modify.set-parameters` structure is the target output format.
- If the same parameter ID is specified multiple times, values are combined into a single `set-parameters` entry's `values` array. This is intentional aggregation behavior.
- No new external dependencies are required; all serialization and CLI parsing reuses existing libraries.
- XML and YAML output for the `modify` section is handled automatically by the existing serialization layer (WI-26/WI-27); no explicit XML/YAML test cases are in scope for WI-31.
