# Research: OSCAL v1.2.3 Compatibility

## Decisions

### Provenance format and owner

Use `schemas/oscal-schema-manifest.json`, verified by Rust integration tests in the existing cross-platform CI `test` job. JSON reuses `serde_json`; SHA-256 reuses `sha2`. No dependency or platform-specific checksum command is required.

### Asset layout

- Runtime JSON: retain the three existing paths under `schemas/` so `include_str!` callers do not move.
- Test-only JSON: add Assessment Plan and SSP under `tests/fixtures/schemas/`.
- Test-only XSD: keep Catalog, Component, and Complete under `tests/fixtures/xsd/`; add the model-specific Profile XSD.
- The manifest covers all nine files and marks each `runtime` or `test`.

### Version enforcement

Parse exactly three ASCII-decimal components. Accept only `1.2.0`, `1.2.1`, `1.2.2`, and `1.2.3`. Do not use lexical comparison or accept prefixes/prereleases. Inspect `metadata.oscal-version` after model detection and before schema validation/export.

### Reporting compatibility

Extend the existing `ValidationReport` additively rather than replacing its error contract. Preserve `artifact_path` and `is_valid`; add `model_type`, `declared_oscal_version`, `schema_version_used`, and `supported_input`. Extend `RoundTripResult` similarly and record the detected oscal-cli version plus a stable classification.

### Offline XML gate

Use local model-specific XSDs with `xmllint --nonet` where available. The Rust provenance test independently rejects XSD `schemaLocation` values that resolve remotely, so required CI behavior does not depend on fetching schemas.

## Evidence

- The official v1.2.3 GitHub release API publishes size and `sha256:` digest metadata for every selected asset.
- Current runtime validation embeds three JSON schemas and performs no runtime download.
- Existing `sha2`, `serde_json`, `jsonschema`, `quick-xml`, `serde_yaml`, and `tempfile` dependencies cover the implementation.
- Existing CI already runs tests, strict Clippy, and formatting across the supported platform matrix.

## Rejected Alternatives

- **TOML manifest**: adds parsing surface without a benefit for this machine-owned inventory.
- **Shell-only checksum workflow**: `sha256sum` is not portable across the current macOS/Windows matrix.
- **Parallel v1.2.0 schemas**: contradicts the single-current-baseline policy and increases embedded surface.
- **Metadata-driven downloads**: violates offline and supply-chain requirements.
- **Complete XSD as the only XML gate**: weaker failure localization than model-specific assets.
