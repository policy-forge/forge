# Feature Specification: Component Implemented Requirements

**Feature Branch**: `015-component-implemented-requirements`
**Created**: 2026-02-13
**Status**: Draft
**Input**: Derived from docs/PRD/015-prd-component-implemented-requirements.md (WI-15)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Map Policy Requirements to Control IDs (Priority: P1)

A compliance engineer converts a policy document into a Component Definition where each policy requirement becomes an implemented-requirement entry linked to a control-id from the specified baseline.

> As a compliance engineer, I want each policy requirement to be mapped to a control-id in the Component Definition so that I can trace which controls my policy addresses and use the output for compliance automation.

**Why this priority**: This is the core function of WI-15. Without control-id mapping, the Component Definition has no compliance value. Every downstream workflow (traceability in WI-16/WI-17, end-to-end pipeline in WI-18) depends on implemented-requirements being populated.

**Independent Test**: Build a Component Definition from a PolicyDocument with 5 requirements and a source profile reference, and verify 5 implemented-requirements are produced, each with a valid control-id and narrative description.

**Acceptance Scenarios**:

1. **Given** a PolicyDocument with 5 PolicyRequirements and a source profile path, **When** generating the Component Definition, **Then** the output contains a `control-implementations` array with one entry whose `source` references the profile path, and 5 `implemented-requirements` entries.
2. **Given** a PolicyRequirement with text "All employees must complete security awareness training annually", **When** mapped to an implemented-requirement, **Then** the `description` field contains the implementation narrative derived from that requirement prose.

---

### User Story 2 - Source Profile Reference in Control Implementations (Priority: P1)

A compliance engineer specifies which baseline profile the Component Definition maps against using the `--source-profile` CLI flag.

> As a compliance engineer, I want to specify the source baseline profile so that the Component Definition's control-implementations correctly reference the profile my organization uses for compliance.

**Why this priority**: The `source` field in `control-implementations` is structurally required to indicate which baseline the implemented-requirements map against. Without it, the control-id values have no context.

**Independent Test**: Generate a Component Definition with `--source-profile ./baselines/nist-800-53-moderate.json` and verify the `source` field in `control-implementations` equals that path.

**Acceptance Scenarios**:

1. **Given** a source profile path of `./baselines/nist-800-53-moderate.json`, **When** generating the Component Definition, **Then** the `control-implementations[0].source` field equals `"./baselines/nist-800-53-moderate.json"`.
2. **Given** no `--source-profile` flag is provided, **When** generating the Component Definition with `--strategy component`, **Then** the system exits with a descriptive error indicating that `--source-profile` is required for the component strategy.

---

### User Story 3 - Deterministic UUIDs for Implemented Requirements (Priority: P1)

Each implemented-requirement and control-implementation receives a deterministic UUID for stability across re-conversions.

> As a developer working on FORGE, I want implemented-requirement UUIDs to be deterministic so that re-converting the same policy produces identical identifiers, enabling meaningful diffs and stable traceability.

**Why this priority**: UUID stability is a cross-cutting requirement (Parent PRD M-8) that must be established at generation time. Non-deterministic UUIDs would break traceability and make diffs meaningless.

**Independent Test**: Generate a Component Definition from the same PolicyDocument twice and verify all UUIDs in `control-implementations` and `implemented-requirements` are identical across runs.

**Acceptance Scenarios**:

1. **Given** the same PolicyDocument and source profile, **When** generating the Component Definition twice, **Then** all `uuid` values in `control-implementations` and `implemented-requirements` are identical.
2. **Given** a PolicyRequirement whose text is substantively changed, **When** re-generating, **Then** the corresponding `implemented-requirement` UUID changes.

---

### User Story 4 - Implementation Narrative with Source Context (Priority: P2)

A compliance reviewer reads the Component Definition and understands where each implementation narrative originated.

> As a compliance reviewer, I want implementation narratives to preserve the original requirement text with source context so that I can audit the mapping without referring back to the original document.

**Why this priority**: While the core mapping (P1) ensures completeness, adding source context (policy section prefix) improves auditability and reviewer confidence. Not a launch blocker but high value.

**Independent Test**: Generate a Component Definition from a policy with known section titles and verify narratives include contextual prefixes referencing the source section.

**Acceptance Scenarios**:

1. **Given** a PolicyRequirement from section "3.1 Access Control" with text "Systems must enforce MFA for privileged access", **When** mapped to an implemented-requirement, **Then** the description preserves the original requirement text with a prefix indicating the source policy section.
2. **Given** a policy document titled "Corporate Security Policy", **When** generating the Component Definition, **Then** the `control-implementations` entry description includes a reference to "Corporate Security Policy".

---

### Edge Cases

- **EC-1**: When a PolicyDocument has zero PolicyRequirements, the `implemented-requirements` array is empty and a warning is emitted.
- **EC-2**: When a PolicyRequirement has no stable_id, a fallback control-id is generated using the index-based format `REQ-{zero-padded index}` (e.g., `REQ-001`).
- **EC-3**: When a PolicyRequirement has empty text, the implemented-requirement description defaults to a placeholder indicating no narrative available.
- **EC-4**: When `--source-profile` is an empty string, the system exits with an error indicating an invalid profile path.
- **EC-5**: When two PolicyRequirements have identical text but different source locations, they receive distinct UUIDs (the UUID seed incorporates source location or index).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST include a `control-implementations[]` array with at least one entry in the Component Definition when PolicyRequirements are present.
- **FR-002**: Each `control-implementations` entry MUST include a deterministic `uuid` field.
- **FR-003**: Each `control-implementations` entry MUST include a `source` field set to the value provided by the user via the `--source-profile` flag.
- **FR-004**: Each `control-implementations` entry MUST include a `description` field summarizing the implementation context, referencing the policy document title.
- **FR-005**: Each `control-implementations` entry MUST include an `implemented-requirements[]` array populated from PolicyRequirements in the domain model.
- **FR-006**: Each `implemented-requirement` entry MUST include a deterministic `uuid` field generated using UUID v5.
- **FR-007**: Each `implemented-requirement` entry MUST include a `control-id` field derived from the requirement's control identifier.
- **FR-008**: Each `implemented-requirement` entry MUST include a `description` field containing the raw PolicyRequirement text as the implementation narrative (no transformation or prefix at P1 scope).
- **FR-009**: The `--source-profile` flag MUST be required when using `--strategy component`; omitting it MUST produce a descriptive error message.
- **FR-010**: The implementation narrative SHOULD preserve the original requirement text with minimal transformation, prefixed with context indicating the source policy section.
- **FR-011**: All PolicyRequirements in the input MUST be mapped to implemented-requirements with no requirements lost during conversion.
- **FR-012**: Two PolicyRequirements with identical text but different source locations MUST receive distinct UUIDs.
- **FR-013**: A PolicyDocument with zero PolicyRequirements MUST produce an empty `implemented-requirements` array and emit a warning.
- **FR-014**: A PolicyRequirement with empty text MUST produce an implemented-requirement with a placeholder description.

### Key Entities

- **ControlImplementation**: Groups implemented-requirements under a single source baseline reference. Contains a uuid, source profile reference, description, and a collection of implemented-requirements.
- **ImplementedRequirement**: Maps a single PolicyRequirement to an OSCAL control-id with an implementation narrative. Contains a uuid, control-id, and description.
- **PolicyRequirement** *(existing)*: The domain model struct representing an individual policy requirement extracted from source text. Contains stable_id, text, and source location.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of PolicyRequirements from the input document are represented as implemented-requirements in the output Component Definition.
- **SC-002**: Generating the same Component Definition from identical inputs produces identical UUIDs across all runs, with zero variance.
- **SC-003**: The `source` field in every `control-implementations` entry exactly matches the user-provided `--source-profile` value.
- **SC-004**: Omitting `--source-profile` with `--strategy component` produces a clear, actionable error message — no silent failures or partial output.
- **SC-005**: Each implementation narrative faithfully represents the original requirement prose, enabling compliance reviewers to audit mappings without referring to the source document.
- **SC-006**: All generated output conforms to the OSCAL v1.2.0 Component Definition structure for `control-implementations` and `implemented-requirements`.

## Assumptions

- WI-14 provides a working Component Definition builder with documentary component structure that this feature extends.
- The `control-id` for each implemented-requirement is derived from the PolicyRequirement's stable_id or the mapping scheme established in the Catalog generation path (WI-9/WI-10).
- UUID v5 generation from WI-7 is available for generating deterministic UUIDs.
- A single `control-implementations` entry is sufficient for mapping all requirements against one baseline profile; multiple baselines are not required at this stage.
- The `--source-profile` flag may already be defined as a CLI argument placeholder from prior work items.

## Dependencies

- **Requires**: WI-14 (component definition structure), WI-7 (UUID generation), WI-9 (catalog groups/controls for control-id scheme)
- **Blocks**: WI-17 (traceability embedding), WI-18 (component pipeline)
- **Parallel With**: WI-16 (traceability model)

## Clarifications

### Session 2026-02-13

- Q: Should the P1 (MUST) implementation narrative be the raw requirement text, or include a section-context prefix? → A: Raw requirement text for P1 (MUST); section-context prefix deferred to P2/S-1 (SHOULD).
- Q: What format should the fallback control-id use when stable_id is absent? → A: Index-based: `REQ-{zero-padded index}` (e.g., `REQ-001`, `REQ-002`).

## Scope Boundaries

**In Scope**:
- Populating `control-implementations[]` with source profile reference and implemented-requirements
- Mapping PolicyRequirements to implemented-requirement entries with control-id linking
- Generating deterministic UUIDs for control-implementations and implemented-requirements
- Generating implementation narratives from PolicyRequirement prose
- Consuming the `--source-profile` CLI flag

**Out of Scope**:
- Documentary component structure (type, title, description) — completed in WI-14
- TraceLink model and source-to-OSCAL element mapping — deferred to WI-16
- Embedding trace metadata as props/links — deferred to WI-17
- End-to-end component pipeline wiring — deferred to WI-18
- OSCAL metadata assembly — completed in WI-11
- Back matter resource generation — completed in WI-12
- Validation of control-ids against the source profile's actual controls — deferred to WI-19
- Multiple components per Component Definition — deferred
- `set-parameters` within implemented-requirements — deferred to WI-34
