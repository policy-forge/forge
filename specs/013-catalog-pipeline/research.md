# Research: End-to-End Catalog Pipeline (WI-13)

## R1: Pipeline Stage Interface Compatibility

**Question**: Do all 12 pipeline stage functions have compatible signatures for sequential composition?

**Finding**: Yes, with two caveats:

1. **Immutability inconsistency**: `atomize_document(&PolicyDocument) -> Result<PolicyDocument>` returns a new document (immutable), but `assign_stable_ids(&mut PolicyDocument)` mutates in place. The orchestrator must handle both patterns.

2. **Content reconstruction**: Between ingest and parse, `IngestedDocument.reconstruct_content()` must be called to get the raw Markdown string. This is an existing pattern used in `pipeline_test.rs` and `cli/convert.rs`.

**Decision**: Accept both patterns. The orchestrator handles ownership transitions explicitly.

## R2: OscalMetadata Type Conflict

**Question**: How should the two `OscalMetadata` types be reconciled?

**Finding**: Two types exist:
- `oscal::catalog::OscalMetadata` (placeholder): `{title: String, last_modified: String, version: String, oscal_version: String}`
- `oscal::metadata::OscalMetadata` (real): `{uuid: Uuid, title: String, last_modified: DateTime<Utc>, version: String, oscal_version: String}`

The `OscalCatalog` struct uses the placeholder type. `build_catalog()` fills it with dummy values.

**Decision**: Map real metadata fields to placeholder strings in the orchestrator. Do NOT modify existing types (AR guardrail). Use `uuid.to_string()` for UUID and `DateTime::to_rfc3339()` for timestamp conversion. This is documented technical debt — a future WI should unify the two metadata types to restore compile-time guarantees.

**Alternatives considered**:
- Replace placeholder type in OscalCatalog with real type: Rejected (modifies existing WI-9 code)
- Create a `From` impl: Rejected (adds coupling, violates YAGNI)

## R3: WI-8 Citation Extraction Status

**Question**: Is citation extraction available for the pipeline?

**Finding**: Not available. The `Citation` struct exists. `generate_back_matter(&[Citation])` can process them. But NO extraction function exists. WI-8 is "In Progress" on a separate track.

**Decision**: Pass `&[]` to `generate_back_matter()`. Produces empty back matter. Set `back_matter: None`. Valid OSCAL output. Pipeline will integrate WI-8 when available.

## R4: OSCAL JSON Envelope Structure

**Question**: What is the expected top-level JSON structure?

**Finding**: `CatalogEnvelope` already exists in `oscal::catalog`, producing `{"catalog": {...}}`.

**Decision**: Use `CatalogEnvelope` directly. No additional wrapper needed.

## R5: CLI Flag Design

**Question**: Should `--strategy` and `--format` be required or have defaults?

**Finding**: Spec EC-4 and EC-5 require errors when either flag is omitted.

**Decision**: Make `--strategy` required (remove `Option`), make `--format` required (remove `default_value`). Validate `--strategy component` in handler with descriptive error per S-3.

## R6: ForgeError Serialization Variant

**Decision**: Add `ForgeError::Serialization(String)` variant. Use for `serde_json::to_string_pretty` failures.

## R7: Output Path Validation

**Finding**: SEC-6 and EC-3 require validation.

**Decision**: Check `path.parent().map(|p| p.exists())` before writing. Return descriptive error if parent dir does not exist. Write-permission validation is deferred — Rust's `std::fs::write` already produces a descriptive `io::Error` if the directory is not writable.

## R8: Pretty Print Default

**Finding**: S-1 specifies pretty-printed JSON by default.

**Decision**: Use `serde_json::to_string_pretty()`. No `--compact` flag in WI-13 (C-1 deferred).
