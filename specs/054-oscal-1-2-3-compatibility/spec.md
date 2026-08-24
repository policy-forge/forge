# Feature Specification: OSCAL v1.2.3 Compatibility

**Feature Branch**: `054-oscal-1-2-3-compatibility`
**Created**: 2026-08-23
**Status**: In Progress
**Input**: `docs/PRD/054-prd-oscal-1-2-3-compatibility.md`

## User Scenarios & Testing

### User Story 1 - Generate Current OSCAL Artifacts (Priority: P1)

Compliance engineers receive Catalog, Component Definition, Profile, Assessment Plan, and SSP output that declares OSCAL v1.2.3 while retaining the source document version.

**Independent Test**: Generate each existing model and assert `metadata.oscal-version == "1.2.3"`, the source `metadata.version` is unchanged, and the output passes its official v1.2.3 schema gate.

### User Story 2 - Preserve Supported Legacy Inputs (Priority: P1)

Existing Catalog, Component Definition, and Profile inputs declaring v1.2.0 through v1.2.3 continue through their already-supported validation and export paths when compatible with the v1.2.3 schema.

**Independent Test**: Validate and export immutable v1.2.0 fixtures across every existing format path and verify both version fields survive unchanged.

### User Story 3 - Report Honest Compatibility Evidence (Priority: P1)

Auditors can distinguish the document-declared OSCAL version from the schema and optional oscal-cli baselines actually used.

**Independent Test**: Exercise text and JSON validation plus round-trip output and assert stable declared, schema, tool-version, and compatibility-classification fields.

### User Story 4 - Verify the Vendored Standards Supply Chain (Priority: P2)

Maintainers can prove that every runtime and compatibility-test schema is an unmodified official v1.2.3 release asset and can repeat the process for a future release.

**Independent Test**: Run the offline provenance verifier from a clean checkout, then alter one byte in a temporary schema copy and confirm verification fails with the affected asset and expected/actual digest.

### Edge Cases

- Missing, null, numeric, empty, whitespace-only, prerelease, and out-of-range `metadata.oscal-version` values fail without fallback.
- `1.2.10` is rejected by numeric component parsing, not accepted by lexical comparison.
- Ambiguous roots remain an error; version inspection does not choose a model.
- XML namespace and declared OSCAL version remain separate concerns.
- oscal-cli absence is reported as unavailable without blocking core offline gates.
- Any remote JSON `$ref` or XSD import/include fails the provenance gate.

## Requirements

### Functional Requirements

- **FR-001**: Pin `usnistgov/OSCAL` tag `v1.2.3`, release commit `e061961`, and publication date `2026-08-07`.
- **FR-002**: Record URL, path, size, SHA-256, version, format, model, and role for every vendored schema in a machine-readable JSON manifest.
- **FR-003**: Verify schema bytes and prohibit remote references without network access.
- **FR-004**: Embed pristine v1.2.3 Catalog, Component Definition, and Profile JSON schemas and retain AP/SSP JSON plus model XSDs as test-only gates.
- **FR-005**: Generate all five existing output families with the shared OSCAL version `1.2.3` without changing the document version.
- **FR-006**: Accept compatible declarations `1.2.0` through `1.2.3`; reject all other declarations with an actionable unsupported-version diagnostic.
- **FR-007**: Preserve imported OSCAL and document versions across existing export paths.
- **FR-008**: Add model type, declared OSCAL version, schema version used, and supported-input status to validation results without removing existing fields.
- **FR-009**: Add declared/schema/tool baselines and compatibility classification to round-trip results.
- **FR-010**: Gate generated Catalog, Component Definition, and Profile JSON/XML/YAML plus AP/SSP JSON offline.
- **FR-011**: Protect labeled v1.2.0 legacy fixtures from generated-fixture refresh.
- **FR-012**: Document and automate a verify-before-replace future upgrade workflow.
- **FR-013**: Preserve existing size, timeout, parsing, deterministic-ID, and command-scope boundaries; do not add Mapping, AP/SSP commands, runtime downloads, or multi-version schema selection.

### Key Entities

- **SchemaAsset**: One official release file and its immutable provenance fields.
- **SchemaManifest**: The pinned release identity and allowlisted assets.
- **SupportedOscalVersion**: A parsed three-component supported declaration in the inclusive `1.2.0..=1.2.3` policy.
- **ValidationReport**: Existing validation evidence extended with model and version context.
- **RoundTripResult**: Existing divergence evidence extended with schema/tool compatibility context.

## Success Criteria

- **SC-001**: All required generated model/format fixtures pass their v1.2.3 gates.
- **SC-002**: All retained compatible v1.2.0 fixtures pass existing supported paths and preserve declarations.
- **SC-003**: Every vendored schema matches the manifest size and SHA-256; no remote references exist.
- **SC-004**: Every validation and round-trip contract test reports the declared and actual baseline unambiguously.
- **SC-005**: `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`, and provenance verification pass.
