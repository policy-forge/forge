# Research: Assessment Plan Scaffolding — Controls (WI-41)

**Feature Branch**: `041-assessment-plan-controls`
**Phase**: 0 — Research & Unknown Resolution
**Date**: 2026-03-12

---

## Summary

All technical unknowns resolved by codebase inspection. No new dependencies required.
All infrastructure (UUID v5, metadata assembly, serde_json builder, clap CLI) already exists.

---

## Decision Log

### D-1: Control ID collection mechanism

**Decision**: Collect control IDs from the primary artifact after it is built, not during
building. Expose two pure helper functions — one for Catalog, one for Component Definition.

**Rationale**: The pipeline already builds the full artifact (Catalog or Component Definition)
before writing output. After `build_catalog()` returns an `OscalCatalog`, iterating
`catalog.groups[].controls[].id` is a one-liner. For Component Definition,
iterate `components[].control_implementations[].implemented_requirements[].control_id`.
This is zero-coupling and requires no changes to the Catalog or Component Definition
builders themselves.

**Alternatives considered**:
- Collect during `build_catalog` (invasive — adds a side-channel output parameter to existing functions)
- Read the output JSON file (breaks in-memory pipeline pattern; adds I/O dependency)

**Codebase evidence**: `src/oscal/catalog.rs` — `OscalCatalog` struct has
`pub groups: Vec<OscalGroup>` → `pub controls: Vec<OscalControl>` → `pub id: String`.
Each control ID is already a `String` in the form `"POL-AC-001"`. The
`component_definition.rs` has `components[].control_implementations[].implemented_requirements`
where each entry has a `control_id` field.

---

### D-2: UUID v5 namespace for Assessment Plan elements

**Decision**: Add `ASSESSMENT_PLAN_NAMESPACE` constant to `src/uuid.rs`, derived as
`Uuid::new_v5(&FORGE_NAMESPACE_UUID, b"assessment-plan")`.

**Rationale**: Follows the established namespace pattern in `src/uuid.rs`:
`BACK_MATTER_NAMESPACE`, `COMPONENT_NAMESPACE`, `CONTROL_IMPL_NAMESPACE`,
`IMPL_REQ_NAMESPACE`. Each is a named constant with a derivation comment and a
corresponding test that verifies the derivation. Two UUIDs need namespacing:
1. **Document UUID**: `generate_stable_id` with input
   `format!("assessment-plan|{}|{}", sorted_control_ids.join(","), ssp_href)`
   — changes when control set or SSP reference changes (satisfies FR-008, SC-003).
2. **Control-selection UUID**: `Uuid::new_v5(&ASSESSMENT_PLAN_NAMESPACE, ...)`
   with same seed input as document UUID.

**Alternatives considered**: Reusing `FORGE_NAMESPACE_UUID` directly — would work but
loses the semantic namespacing that makes debugging easier and matches the pattern.

---

### D-3: AP output path derivation

**Decision**: AP output path = `{output_dir}/{policy_stem}-assessment-plan.json` where:
- `output_dir` = parent directory of `--output` path if provided, else `.` (cwd)
- `policy_stem` = input file stem (e.g., `policy.md` → `policy`)
- Always JSON (no XML/YAML variant for AP in WI-41 scope)

**Rationale**: Consistent with how `batch::output_naming::derive_output_paths` computes
output paths from input stems. Uses the input stem (not the output path stem) because the
input stem is the natural identifier of the policy being assessed.

**Implementation**: A small helper `fn derive_ap_output_path(input: &Path, output: Option<&Path>) -> PathBuf`
in `src/pipeline.rs` or `src/cli/convert.rs`.

---

### D-4: Batch mode handling

**Decision**: In batch mode (2+ input files), if `--import-ssp` is provided, emit a
`tracing::warn!` and skip AP generation — identical to the `--stable-id-baseline` behavior.

**Rationale**: WI-41 spec and PRD describe single-file AP generation. Batch AP generation
is not mentioned in any requirement. Silently ignoring is consistent with the existing
`--stable-id-baseline` warning in `src/cli/convert.rs` (`execute_dispatch`). Per YAGNI,
batch AP support is deferred to a future work item if needed.

---

### D-5: Pipeline function signature extension

**Decision**: Extend `run_catalog_pipeline` and `run_component_pipeline` signatures to
accept `import_ssp_href: Option<&str>` as a final parameter. When `Some(href)`, AP is
generated and written after the primary artifact. When `None`, pipeline behavior is
unchanged (backward compatible).

**Rationale**: Minimal, additive change. All existing callers (batch orchestrator) can pass
`None`. Single-file `execute()` in `src/cli/convert.rs` passes the flag value.

**Codebase evidence**: `src/pipeline.rs` — both functions already take all other options
as explicit parameters. The `run_catalog_pipeline` signature:
```rust
pub fn run_catalog_pipeline(
    input_path: &Path,
    output_path: Option<&Path>,
    max_size_bytes: u64,
    format: &OutputFormat,
) -> Result<ConversionStatistics, ForgeError>
```
Adding `import_ssp_href: Option<&str>` as last parameter is non-breaking for all
direct callers (there are 3: `cli/convert.rs`, `pipeline.rs` tests, and batch orchestrator).

---

### D-6: ForgeError variant for AP build failures

**Decision**: Add `ForgeError::AssessmentPlanBuild(String)` variant in `src/error.rs`,
mapped to exit code 2 (parse/structure errors), consistent with `CatalogBuild` and
`ComponentDefinitionBuild`.

**Rationale**: Consistent error taxonomy. Allows precise error messages for AP-specific
failures (e.g., empty SSP href validation inside the builder).

---

### D-7: metadata.version value in Assessment Plan

**Decision**: Set `metadata.version = "1.0.0"` (static string, hardcoded in builder).

**Rationale**: Confirmed by clarification Q3 (2026-03-12). The `assemble_metadata`
function takes a `DocumentMetadata` which has a `version` field. We construct a synthetic
`DocumentMetadata` for the AP with `version: "1.0.0".to_string()` and `title: format!("Assessment Plan for {policy_title}")`.

---

### D-8: Empty SSP validation location

**Decision**: Validate `import_ssp_href` is non-empty inside `build_assessment_plan()`,
not at the CLI layer. Return `ForgeError::Validation("--import-ssp must not be empty")`.

**Rationale**: CLI-level validation (clap `required = true`) handles the missing-flag case.
The empty-string case is a runtime check. Placing it in the builder makes it testable
as a pure unit test without CLI setup. Consistent with `resolve_source_profile()` pattern
in `src/cli/convert.rs` (validates non-empty after the flag is present).

---

## No-Op Decisions (Already Clear)

| Topic | Resolution | Source |
|-------|------------|--------|
| OSCAL version | `"1.2.0"` — `OSCAL_VERSION` constant in `metadata.rs` | `src/oscal/metadata.rs:11` |
| Metadata assembly | Call `assemble_metadata(&doc_meta, None)` | `src/oscal/metadata.rs` |
| UUID v5 generation | `generate_stable_id(text)` via `FORGE_NAMESPACE_UUID` | `src/uuid.rs:138` |
| JSON serialization | `serde_json::to_string_pretty(&envelope)` | Established pattern |
| Deduplication order | Sort + dedup before building include-controls | AR guardrail |
| Control-ids source (Catalog) | `catalog.groups[].controls[].id` | `src/oscal/catalog.rs` |
| No new dependencies | All needed crates already in `Cargo.toml` | Verified |
| Test file location | `tests/assessment_plan_test.rs` (integration) + inline unit tests | Convention |
