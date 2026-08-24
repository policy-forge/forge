# Implementation Plan: OSCAL v1.2.3 Compatibility

**Branch**: `054-oscal-1-2-3-compatibility` | **Date**: 2026-08-23 | **Spec**: `specs/054-oscal-1-2-3-compatibility/spec.md`

## Summary

Replace the mixed OSCAL schema baseline with nine pinned, pristine v1.2.3 assets; verify them offline from a JSON manifest; update generated metadata; enforce and report supported declarations separately from the schema used; then gate all currently emitted models and formats while retaining immutable legacy inputs.

## Technical Context

**Language/Version**: Rust edition 2024, stable 1.93.0
**Primary Dependencies**: existing `serde_json`, `sha2`, `jsonschema`, `quick-xml`, `serde_yaml`, `tempfile`, `tracing`
**Storage**: vendored schema and fixture files only
**Testing**: Rust unit/integration tests, existing golden/snapshot suites, optional `xmllint --nonet` and oscal-cli lanes
**Target Platform**: Linux, macOS, Windows CLI
**Project Type**: single Rust crate
**Constraints**: offline runtime, pristine upstream bytes, additive output contracts, no new models or dependencies

## Constitution Check

- Crate-first: extend `src/validate`, `src/round_trip`, and existing serializers only.
- Contract-first: validation and round-trip fields are defined before implementation.
- Test-first: each behavior begins with a focused failing test.
- Complete delivery: tasks map FR-001 through FR-013 and PRD M-1 through M-25.
- Security-first: integrity and remote-reference tests fail closed.
- Simplicity: retain schema paths and existing validator/serializer patterns.
- Quality gates: full test, strict Clippy, format, provenance, and CI matrix.

## Project Structure

```text
schemas/
├── oscal-schema-manifest.json
├── oscal_catalog_schema.json
├── oscal_component_schema.json
└── oscal_profile_schema.json
src/
├── oscal/metadata.rs
├── validate/{mod.rs,error_types.rs,report.rs}
├── cli/{validate.rs,export.rs}
└── round_trip/{divergence.rs,log.rs}
tests/
├── fixtures/schemas/{oscal_assessment-plan_schema.json,oscal_ssp_schema.json}
├── fixtures/xsd/{oscal_catalog_schema.xsd,oscal_component_schema.xsd,oscal_profile_schema.xsd,oscal_complete_schema.xsd}
├── schema_provenance_test.rs
└── oscal_1_2_3_compatibility_test.rs
```

## Delivery Phases

1. Lock inventory, manifest contract, legacy fixture labels, and baseline tests.
2. Vendor/verify official assets; update shared metadata and supported-version enforcement/reporting.
3. Gate and correct current Catalog, Component, Profile, AP, and SSP serializers; refresh generated fixtures only.
4. Extend round-trip evidence, documentation, CI, and future-upgrade verification.

## Complexity Tracking

No constitutional violations or new dependencies are planned.
