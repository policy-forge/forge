# Feature Specification: Deterministic UUID Generation

**Feature Branch**: `007-uuid-generation`
**Created**: 2026-02-11
**Status**: Draft
**Input**: User description: "Read docs/PRD/007-prd-uuid-generation.md as the source of truth for this feature's requirements. This work item implements deterministic UUID generation for OSCAL elements, ensuring stable identifiers across repeated conversions of the same source content."

**Dependencies**:
- Requires: 005-prd-domain-model (PolicyRequirement.stable_id field)
- Requires: 006-prd-requirement-atomization (atomized requirements)
- Parallel with: 008-prd-citation-extraction
- Blocks: 009-prd-catalog-groups-controls

**Related Documents**:
- Parent PRD: docs/FORGE_PRD.md (M-8, AC-8, EC-5, EC-6, Spike-4)
- PRD: docs/PRD/007-prd-uuid-generation.md
- Product Vision: docs/FORGE_PRODUCT_VISION.md (Principle P-3: Deterministic and auditable)

## Problem Statement

After requirement atomization (WI-6), each PolicyRequirement has a `stable_id` field that is None. Without deterministic identifiers, every conversion run produces different UUIDs for the same requirements, making diffs between conversion runs meaningless, breaking traceability across re-conversions, and violating product principle P-3 (Deterministic and auditable). Users cannot trust that identical policy content will produce identical OSCAL output, undermining confidence in the conversion pipeline and preventing meaningful change tracking over time.

## User Scenarios & Testing

### User Story 1 - Deterministic IDs Across Runs (Priority: P1)

A compliance engineer converts the same policy document twice and expects identical OSCAL output, including identical requirement identifiers.

**Why this priority**: Deterministic output is a core product principle (P-3). Without stable IDs, all downstream OSCAL generation produces non-reproducible output that cannot be meaningfully compared or tracked. This is the foundation of trust in the conversion pipeline.

**Independent Test**: Convert a test policy document twice with the same content. Compare the stable_id values on all PolicyRequirements and verify they are identical. This can be tested with a single test policy file and delivers the core value proposition: reproducibility.

**Acceptance Scenarios**:

1. **Given** a PolicyRequirement with text "All users must use multi-factor authentication", **When** UUID v5 is generated twice with the same namespace, **Then** both generated UUIDs are identical
2. **Given** two separate conversion runs of the same Markdown policy file, **When** comparing the stable_id values on corresponding PolicyRequirements, **Then** all stable_ids are identical across runs

---

### User Story 2 - Whitespace Normalization (Priority: P1)

A compliance engineer makes whitespace-only edits to a policy document (reformatting, extra spaces, trailing newlines) and expects stable IDs to remain unchanged.

**Why this priority**: Without normalization, trivial formatting changes would generate new UUIDs, creating noise in diffs and undermining trust in the stability guarantee. This is essential for practical use of the tool with real-world policy documents that undergo formatting changes.

**Independent Test**: Generate a UUID for a requirement, then modify only the whitespace in the requirement text (add leading/trailing spaces, collapse double spaces, change indentation) and regenerate. Verify the UUID is unchanged. This demonstrates robustness against formatting variations.

**Acceptance Scenarios**:

1. **Given** a requirement "Users must change passwords every 90 days", **When** the text is changed to "  Users  must  change  passwords  every  90  days  " (extra whitespace), **Then** the generated UUID is identical to the original
2. **Given** a requirement with a trailing newline, **When** the trailing newline is removed, **Then** the generated UUID is unchanged
3. **Given** a requirement with mixed tabs and spaces, **When** all whitespace is normalized to single spaces, **Then** the generated UUID is identical

---

### User Story 3 - Substantive Change Detection (Priority: P1)

When a requirement's text is substantively changed, the stable ID changes to reflect the new content, enabling change detection through identifier comparison.

**Why this priority**: If substantive changes did not alter the UUID, the system would be unable to detect content drift between policy revisions. Users need confidence that meaningful changes are reflected in ID changes.

**Independent Test**: Generate a UUID for a requirement, then make a substantive text change (alter a word, add a clause) and regenerate. Verify the UUID is different. This validates that the system can detect real policy changes.

**Acceptance Scenarios**:

1. **Given** a requirement "All users must use MFA", **When** the text is changed to "All administrators must use MFA", **Then** the generated UUID is different from the original
2. **Given** a requirement "Passwords must be at least 8 characters", **When** the text is changed to "Passwords must be at least 12 characters", **Then** the generated UUID is different

---

### Edge Cases

- **EC-1**: When a requirement contains only whitespace, the normalized text is an empty string, and the UUID is still generated deterministically (UUID v5 of empty string is well-defined)
- **EC-2**: When a requirement has mixed newlines, tabs, and spaces, all are collapsed to single spaces and the UUID is the same as a cleanly-formatted version
- **EC-3**: When a requirement has Unicode whitespace characters (e.g., non-breaking space, em space), they are treated as whitespace and collapsed per Rust's split_whitespace behavior
- **EC-4**: When two different requirements have different text, the generated UUIDs are different (no false collisions for reasonable input sizes)
- **EC-5**: When a PolicyDocument has nested sections with requirements at multiple levels, all requirements at all nesting depths receive a stable_id

## Requirements

### Functional Requirements

- **FR-001 (M-1)**: System MUST generate UUID v5 identifiers for PolicyRequirements using a fixed FORGE namespace UUID and the normalized requirement text as the name *(Traces to: Parent PRD M-8)*
- **FR-002 (M-2)**: System MUST normalize requirement text before hashing by trimming leading and trailing whitespace and collapsing all internal whitespace runs to a single space *(Traces to: Parent PRD EC-5)*
- **FR-003 (M-3)**: System MUST populate PolicyRequirement.stable_id with the generated UUID v5 string for every atomized requirement *(Traces to: Parent PRD M-8)*
- **FR-004 (M-4)**: System MUST produce identical UUIDs for identical requirement text across separate conversion runs *(Traces to: Parent PRD AC-8)*
- **FR-005 (M-5)**: System MUST produce different UUIDs when requirement text is substantively changed *(Traces to: Parent PRD EC-6)*
- **FR-006 (S-1)**: The FORGE namespace UUID SHOULD be defined as a well-documented constant in the codebase with a comment explaining its purpose and the consequence of changing it
- **FR-007 (S-2)**: The UUID generation function SHOULD accept any string input (not just PolicyRequirement), enabling reuse for other content-addressed identifiers in later work items
- **FR-008 (C-1)**: System COULD log (at debug level) the normalized text and generated UUID for each requirement to aid debugging
- **FR-009 (W-1)**: System will NOT provide CLI warning when a requirement's stable ID changes between conversions (deferred to WI-43 diff report capability)
- **FR-010 (W-2)**: System will NOT implement UUID v4 generation for OSCAL artifact-level identifiers (deferred to WI-11 OSCAL metadata)
- **FR-011 (W-3)**: System will NOT implement case-insensitive normalization or Unicode normalization beyond whitespace handling (out of scope; may be revisited based on user feedback)
- **FR-012 (W-4)**: System will NOT persist stable IDs to a local cache or database (UUID v5 is deterministic by design; no persistence needed)

### Key Entities

- **FORGE_NAMESPACE_UUID**: A fixed UUID constant specific to FORGE that serves as the namespace parameter for all UUID v5 generation. Changing this value is a breaking change that alters all generated stable_ids.
- **PolicyRequirement.stable_id**: An Option<String> field (defined in WI-5) that holds the deterministic UUID v5 identifier. Populated from None to Some(uuid_string) by this feature.
- **Normalized Text**: The requirement text after trimming leading/trailing whitespace and collapsing internal whitespace runs to single spaces. This normalized form is used as the "name" parameter for UUID v5 generation.

## Success Criteria

### Measurable Outcomes

- **SC-001**: 100% of PolicyRequirements generated from identical source content across multiple conversion runs have identical stable_id values (determinism verification)
- **SC-002**: 100% of PolicyRequirements with whitespace-only text changes produce identical stable_id values (normalization verification)
- **SC-003**: 100% of PolicyRequirements with substantive text changes produce different stable_id values (sensitivity verification)
- **SC-004**: All PolicyRequirements in a PolicyDocument have stable_id populated (no None values remain after UUID generation)
- **SC-005**: Generated UUIDs conform to RFC 4122 UUID v5 format (version nibble = 5, variant bits correct)

### Assumptions

- **A-1**: The `uuid` Rust crate (MIT/Apache-2.0) supports UUID v5 generation and is the selected tool per the parent PRD tool evaluation
- **A-2**: A single fixed FORGE namespace UUID will be defined as a constant and used for all requirement UUID generation
- **A-3**: Content normalization consists of trimming leading/trailing whitespace and collapsing internal runs of whitespace to single spaces. No other normalization (e.g., case folding, punctuation normalization) is required
- **A-4**: The stable_id field on PolicyRequirement is Option<String> (defined in WI-5) and will be populated as Some(uuid_string) after this work item
- **A-5**: Rust's split_whitespace method handles Unicode whitespace correctly for normalization purposes

### Out of Scope

- UUID v4 generation for OSCAL artifact-level identifiers (deferred to WI-11)
- CLI warning on stable ID changes between conversions (deferred to WI-43 with diff/comparison capability)
- Citation extraction (running in parallel as WI-8)
- OSCAL catalog/control generation that consumes stable IDs (deferred to WI-9)
- Case-insensitive or advanced Unicode normalization beyond whitespace handling
