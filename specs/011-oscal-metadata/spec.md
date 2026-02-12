# Feature Specification: OSCAL Metadata Assembly

**Feature Branch**: `011-oscal-metadata`
**Created**: 2026-02-11
**Status**: Draft
**Input**: WI-11 from FORGE Product Roadmap. Generate the OSCAL metadata section (uuid, title, version, last-modified, oscal-version) required for valid OSCAL documents. Source of truth: `docs/PRD/011-prd-oscal-metadata.md`.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Generate OSCAL Artifact with Required Metadata (Priority: P1)

When FORGE generates an OSCAL artifact, the metadata section is automatically populated with all required fields so that the artifact passes schema validation and is interoperable with other OSCAL-compliant tools.

> As a compliance engineer, I want generated OSCAL artifacts to include all required metadata fields so that the artifacts pass schema validation and are interoperable with other OSCAL-compliant tools.

**Why this priority**: Metadata is required by the OSCAL specification. Without it, no artifact is valid. This is a blocking dependency for the end-to-end Catalog pipeline (WI-13) and all downstream OSCAL generation.

**Independent Test**: Generate an OSCAL Catalog from a test Markdown policy and verify the metadata object contains `uuid`, `title`, `last-modified`, `version`, and `oscal-version` with correct values.

**Acceptance Scenarios**:

1. **Given** a PolicyDocument with title "Access Control Policy" and version "2.1", **When** generating an OSCAL Catalog, **Then** `metadata.title` equals "Access Control Policy" and `metadata.version` equals "2.1". *(AC-3, AC-4)*
2. **Given** any OSCAL artifact generation, **When** inspecting the metadata, **Then** `metadata.uuid` is a valid UUID v4 string. *(AC-1)*
3. **Given** any OSCAL artifact generation, **When** inspecting the metadata, **Then** `metadata.oscal-version` equals "1.2.0". *(AC-6)*
4. **Given** any OSCAL artifact generation, **When** inspecting the metadata, **Then** `metadata.last-modified` is a valid ISO 8601 UTC timestamp. *(AC-5)*

---

### User Story 2 - Unique Artifact UUIDs Per Generation (Priority: P1)

Each time an OSCAL artifact is generated, it receives a unique UUID v4, distinguishing it from other artifact instances even from the same source document.

> As a compliance engineer, I want each generated OSCAL artifact to have a unique instance UUID so that I can distinguish different generation runs and track artifact provenance over time.

**Why this priority**: UUID v4 per artifact instance is required by OSCAL. Using the same UUID across different generation runs would violate the specification's intent for artifact identification and could cause conflicts when artifacts are consumed by downstream tools.

**Independent Test**: Generate two OSCAL Catalogs from the same Markdown policy and verify they have different `metadata.uuid` values.

**Acceptance Scenarios**:

1. **Given** the same PolicyDocument, **When** generating two OSCAL artifacts in succession, **Then** each has a distinct `metadata.uuid`. *(AC-2)*
2. **Given** a generated artifact UUID, **When** parsing it, **Then** it conforms to UUID v4 format (version nibble = 4, variant bits correct). *(AC-1)*

---

### User Story 3 - Metadata from PolicyDocument Defaults (Priority: P2)

When the source PolicyDocument has missing or default metadata, the metadata assembly provides sensible fallbacks so that artifacts are always generated with valid metadata even from incomplete source documents.

> As a developer working on FORGE, I want the metadata assembly to handle missing source metadata gracefully so that artifacts are always generated with valid metadata even from incomplete source documents.

**Why this priority**: Real-world policy documents may lack explicit version or title metadata. The system must produce valid OSCAL regardless.

**Independent Test**: Generate an OSCAL Catalog from a Markdown file with no frontmatter and verify metadata fields use appropriate defaults.

**Acceptance Scenarios**:

1. **Given** a PolicyDocument with no explicit version (defaulting to "0.0.0"), **When** generating metadata, **Then** `metadata.version` equals "0.0.0". *(AC-8)*
2. **Given** a PolicyDocument whose title was derived from the first H1 heading, **When** generating metadata, **Then** `metadata.title` reflects that heading text. *(AC-8)*

---

### User Story 4 - Shared Metadata Function Across Artifact Types (Priority: P1)

The metadata assembly is a single shared capability reusable by all OSCAL artifact generators (Catalog, Component Definition, Profile) so that metadata is consistent and not duplicated.

> As a developer working on FORGE, I want metadata assembly to be a shared function so that all OSCAL artifact types produce consistent metadata without code duplication.

**Why this priority**: All current and future OSCAL artifact generators (WI-9, WI-14, WI-30, etc.) need metadata. A single shared function avoids divergence and maintenance burden.

**Independent Test**: Verify that Catalog, Component Definition, and Profile generation code paths all invoke the same metadata assembly function.

**Acceptance Scenarios**:

1. **Given** Catalog, Component Definition, or Profile generation, **When** inspecting the metadata assembly code path, **Then** all artifact types use the same metadata assembly function. *(AC-7)*

---

### Edge Cases

- **EC-1** (M-2): When the PolicyDocument title is an empty string, `metadata.title` is set to an empty string and a warning is emitted.
- **EC-2** (M-3): When the PolicyDocument version is "0.0.0" (the default for missing metadata), `metadata.version` is "0.0.0".
- **EC-3** (M-4): When the system clock returns an unexpected value, the timestamp is still formatted as valid ISO 8601 UTC.
- **EC-4** (M-1): When generating many artifacts in rapid succession, each receives a unique UUID v4 (no collisions).
- **EC-5** (M-2): When the PolicyDocument title contains special characters (quotes, ampersands, Unicode), `metadata.title` preserves them faithfully.

## Requirements *(mandatory)*

### Functional Requirements

#### Must Have (M) -- MVP, launch blockers

- **M-1**: The metadata assembly MUST generate a `uuid` field containing a valid UUID v4 string, unique per artifact generation. *(Traces to: Parent PRD M-5, M-8)*
- **M-2**: The metadata assembly MUST populate the `title` field from the PolicyDocument's title value. *(Traces to: Parent PRD M-5)*
- **M-3**: The metadata assembly MUST populate the `version` field from the PolicyDocument's version value. *(Traces to: Parent PRD M-5)*
- **M-4**: The metadata assembly MUST generate a `last-modified` field containing a valid ISO 8601 timestamp (UTC) representing the artifact generation time. *(Traces to: Parent PRD M-5)*
- **M-5**: The metadata assembly MUST set `oscal-version` to "1.2.0". *(Traces to: Parent PRD M-5)*
- **M-6**: The metadata assembly MUST be a shared function reusable by all OSCAL artifact generators (Catalog, Component Definition, Profile). *(Traces to: Parent PRD M-3, M-4, M-5)*

#### Should Have (S) -- High value, not blocking

- **S-1**: The metadata assembly SHOULD accept an optional override for `last-modified` to support deterministic testing (inject a fixed timestamp instead of generating one at call time).
- **S-2**: The metadata assembly SHOULD accept an optional override for `uuid` to support deterministic testing (inject a fixed UUID instead of random generation).

#### Could Have (C) -- Nice to have, if time permits

- **C-1**: The metadata assembly COULD include a `published` field set to the same timestamp as `last-modified` for first-generation artifacts.
- **C-2**: The metadata assembly COULD include a `revisions` array with a single entry for the current generation.

#### Won't Have (W) -- Explicitly deferred

- **W-1**: Optional metadata fields (`roles`, `parties`, `responsible-parties`, `locations`) -- *Reason: Not required by OSCAL; add when organizational context features are introduced.*
- **W-2**: Metadata `remarks` -- *Reason: Per Parent PRD M-11, arbitrary data should not be stored in remarks fields.*
- **W-3**: Metadata `props` and `links` -- *Reason: Deferred to WI-12 (back matter) and WI-17 (traceability) where props/links are contextually appropriate.*

### Key Entities

- **OscalMetadata**: The metadata object attached to every OSCAL artifact. Contains five required fields: `uuid` (UUID v4, unique per generation), `title` (from source document), `last-modified` (ISO 8601 UTC timestamp), `version` (from source document), and `oscal-version` (always "1.2.0").
- **DocumentMetadata**: The existing internal domain model entity (from WI-5) that provides source document properties (`title`, `version`) consumed by the metadata assembly.
- **MetadataOptions**: An optional configuration entity allowing callers to inject specific values for `uuid` and `last-modified` to enable deterministic testing.

## Acceptance Criteria

| AC ID | Requirement | User Story | Given | When | Then |
|-------|-------------|------------|-------|------|------|
| AC-1  | M-1         | US-1, US-2 | Any OSCAL artifact generation | Inspecting `metadata.uuid` | A valid UUID v4 string is present |
| AC-2  | M-1         | US-2       | The same PolicyDocument converted twice | Comparing `metadata.uuid` values | Each generation has a distinct UUID |
| AC-3  | M-2         | US-1       | A PolicyDocument with title "Access Control Policy" | Generating OSCAL metadata | `metadata.title` equals "Access Control Policy" |
| AC-4  | M-3         | US-1       | A PolicyDocument with version "2.1" | Generating OSCAL metadata | `metadata.version` equals "2.1" |
| AC-5  | M-4         | US-1       | Any OSCAL artifact generation | Inspecting `metadata.last-modified` | A valid ISO 8601 UTC timestamp is present |
| AC-6  | M-5         | US-1       | Any OSCAL artifact generation | Inspecting `metadata.oscal-version` | Value equals "1.2.0" |
| AC-7  | M-6         | US-4       | Catalog, Component Definition, or Profile generation | Inspecting metadata assembly code path | All artifact types use the same metadata assembly function |
| AC-8  | M-2, M-3    | US-3       | A PolicyDocument with no frontmatter (title from H1, version "0.0.0") | Generating OSCAL metadata | `metadata.title` reflects the H1 heading text, `metadata.version` equals "0.0.0" |

## Assumptions

- [A-1] The PolicyDocument's `metadata.title` and `metadata.version` fields are populated by the domain model assembly (WI-5) before metadata assembly runs.
- [A-2] OSCAL v1.2.0 metadata schema for required fields is stable; the five targeted fields (`uuid`, `title`, `last-modified`, `version`, `oscal-version`) are the minimum required set.
- [A-3] ISO 8601 timestamps use UTC timezone (e.g., "2026-05-14T10:30:00Z").
- [A-4] UUID v4 (random) is used for artifact instance identification, distinct from UUID v5 (deterministic, content-based) used for stable requirement IDs in WI-7.

## Dependencies

- **Requires**: WI-1 (project scaffolding), WI-5 (domain model -- provides DocumentMetadata struct with `title` and `version` fields)
- **Blocks**: WI-13 (end-to-end Catalog pipeline needs metadata to produce a valid artifact)
- **Parallel With**: WI-9 (Catalog groups/controls), WI-10 (statement parts/prose), WI-12 (back matter)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of generated OSCAL artifacts contain all five required metadata fields (`uuid`, `title`, `last-modified`, `version`, `oscal-version`).
- **SC-002**: 100% of generated `uuid` values conform to UUID v4 format (version nibble = 4, variant bits correct).
- **SC-003**: 100% of generated `last-modified` values are valid ISO 8601 UTC timestamps parseable by standard datetime tools.
- **SC-004**: All OSCAL artifact types (Catalog, Component Definition, Profile) share a single metadata assembly function with no code duplication.
- **SC-005**: All tests pass deterministically using injectable overrides for UUID and timestamp values.
