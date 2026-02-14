# Implementation Plan: 019-schema-validation

**Branch**: `019-schema-validation` | **Date**: 2026-02-13 | **Spec**: [PRD](../../docs/PRD/019-prd-schema-validation.md)
**Input**: PRD, AR, SEC from `docs/` (no spec.md — formal artifacts used directly)

## Summary

Integrate the `jsonschema` Rust crate with compile-time-embedded OSCAL v1.2.0 JSON schemas to validate FORGE-generated artifacts. Implements `forge validate <file>` as a standalone subcommand, adds auto-validation as a pipeline gate in `forge convert`, and supports model type auto-detection (Catalog, Component Definition) with a `--schema-type` override.

## Technical Context

**Language/Version**: Rust edition 2024, stable 1.93.0
**Primary Dependencies**: jsonschema (NEW), serde_json 1.x, clap 4.x, thiserror 2.0.18 (all existing except jsonschema)
**Storage**: N/A — in-memory processing only; schemas embedded at compile time via `include_str!`
**Testing**: `cargo test` (unit + integration), TDD mandatory per constitution IV
**Target Platform**: CLI binary (macOS, Linux — fully offline)
**Project Type**: Single Rust binary (existing Cargo workspace)
**Performance Goals**: < 2 seconds for 100-control Catalog validation
**Constraints**: Fully offline (no network); schemas pinned to NIST OSCAL release; no `.unwrap()` in production code
**Scale/Scope**: 2 schema files embedded (~500KB total), 4 new source files, ~8 modified files

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Crate-First Architecture | PASS | Validation module lives in `src/validate/` within the existing `forge` crate. Not large enough to warrant a separate crate — single-purpose module with clear boundaries. |
| II. Rust-First | PASS | No FFI, no unsafe code. Pure Rust validation via jsonschema crate. |
| III. Contract-First | PASS | Types, enums, and error types defined in AR interface specification before implementation. |
| IV. Test-First (TDD) | PASS | Tests written before implementation per plan phasing. |
| V. Complete Implementation | GATE | All tasks must complete before merge. |
| VI. Performance-First | PASS | < 2 seconds target for 100-control Catalog. Benchmark test included. |
| VII. Security-First | PASS | File size limit enforced (SEC-3); no panics on malformed input (SEC-4, SEC-5); `.unwrap()`-free (SEC-7). |
| VIII. Error Handling | PASS | `thiserror` enum `ValidateError` with contextual variants. No `.unwrap()` in production. |
| IX. Observability | PASS | `tracing` at DEBUG for schema compilation time, INFO for validation result. |
| X. Simplicity | PASS | Minimal 4-component architecture (AR justifies each component). No over-abstraction. |
| XI. Current Dependency Policy | PASS | `jsonschema` added at latest stable; MIT license; `cargo audit` verified. |

## Project Structure

### Documentation (this feature)

```text
specs/019-schema-validation/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── contracts/           # Phase 1 output
```

### Source Code (repository root)

```text
schemas/
├── oscal_catalog_schema.json                  # NEW — NIST OSCAL v1.2.0 Catalog schema
└── oscal_component-definition_schema.json     # NEW — NIST OSCAL v1.2.0 Component Definition schema

src/
├── validate/
│   └── mod.rs           # MODIFY — OscalModelType, ValidationResult, SchemaError, ValidateError,
│                        #           detect_model_type(), load_schema(), validate_artifact()
├── cli/
│   ├── mod.rs           # MODIFY — Add --schema-type flag to Validate variant
│   └── validate.rs      # MODIFY — Wire execute() to validation module
├── error.rs             # MODIFY — Add SchemaValidation variant to ForgeError
├── pipeline.rs          # MODIFY — Add auto-validation gate before write_output
└── lib.rs               # MODIFY — Re-export validation types

tests/
├── validate_test.rs     # NEW — Unit-level integration tests for validation module
└── common/mod.rs        # MODIFY — Add test helpers for validation fixtures if needed
```

**Structure Decision**: Single project (existing). Validation module is a new `src/validate/` module within the existing `forge` crate. Schemas stored in `schemas/` at project root.

## Complexity Tracking

No constitution violations to justify. Architecture is minimal:
- 1 new dependency (jsonschema) — justified by AR (hand-implementing JSON Schema is infeasible)
- 4 new components (OscalModelType, detect, load, validate) — minimal for PRD M-1 through M-6
- No new patterns beyond existing module structure

---

## Phase 0: Research

### Research Tasks

| # | Topic | Status |
|---|-------|--------|
| R-1 | jsonschema crate API: `validator_for()`, `iter_errors()`, `ValidationError` fields | RESOLVED |
| R-2 | OSCAL v1.2.0 JSON schemas: download location, self-containedness, `$ref` resolution | RESOLVED |
| R-3 | File size limit enforcement strategy (SEC-3) | RESOLVED |

### R-1: jsonschema Crate API

**Decision**: Use `jsonschema` crate latest stable (currently 0.26.x+)
**Key API**:
- `jsonschema::validator_for(&schema_value)` → `Result<Validator, ValidationError>` — compiles schema once
- `validator.iter_errors(&instance)` → iterator of `ValidationError` — collects ALL errors (PRD M-5)
- `validator.is_valid(&instance)` → `bool` — quick check
- `ValidationError` has `instance_path` (JSON pointer) and `schema_path` — fulfills PRD S-2
- Error `Display` provides human-readable message
**Rationale**: Active maintenance, MIT license, supports Draft 2020-12 and Draft-07 (covers OSCAL)
**Alternatives**: `valico` (less maintained), custom validator (infeasible per AR)

### R-2: OSCAL v1.2.0 JSON Schemas

**Decision**: Download from NIST OSCAL GitHub release v1.2.0 (tag `v1.2.0`), specifically:
- `oscal_catalog_schema.json`
- `oscal_component-definition_schema.json`
**Key finding**: NIST publishes each model schema as a self-contained file with all `$id`/`$ref` definitions inlined. No external `$ref` resolution needed at runtime.
**Pin strategy**: Store in `schemas/` directory; document the NIST release tag/commit hash in a `schemas/README.md` or inline comment.
**Rationale**: Self-contained schemas work directly with `include_str!` + `serde_json::from_str` + `jsonschema::validator_for`

### R-3: File Size Limit (SEC-3)

**Decision**: Enforce 50MB file size limit in `forge validate` before reading file content, consistent with SEC-3 recommendation and existing `--max-size` pattern in `forge convert`.
**Implementation**: Check `std::fs::metadata(path)?.len()` before `std::fs::read_to_string`. Default 50MB, no CLI override in this WI (can be added later if needed).
**Rationale**: Prevents OOM on extremely large inputs; serde_json parsing is the memory-intensive step.

---

## Phase 1: Design & Contracts

### Data Model

See `specs/019-schema-validation/data-model.md` (generated separately).

**Core types** (from AR interface definitions):

```rust
/// Supported OSCAL model types for schema validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OscalModelType {
    Catalog,
    ComponentDefinition,
}

/// Result of schema validation.
#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub model_type: OscalModelType,
    pub errors: Vec<SchemaError>,
}

/// A single schema validation error.
#[derive(Debug)]
pub struct SchemaError {
    pub message: String,
    pub instance_path: Option<String>,
    pub schema_path: Option<String>,
}

/// Errors from validation operations.
#[derive(Debug, thiserror::Error)]
pub enum ValidateError {
    #[error("Failed to read artifact file: {path}")]
    FileRead { path: PathBuf, #[source] source: std::io::Error },

    #[error("Failed to parse JSON: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Unable to detect OSCAL model type from JSON structure")]
    UnknownModelType,

    #[error("Schema compilation failed for {model_type}: {message}")]
    SchemaCompilation { model_type: String, message: String },

    #[error("Artifact file is too large ({size_mb:.1}MB, limit: {limit_mb}MB)")]
    FileTooLarge { size_mb: f64, limit_mb: u64 },
}
```

### API Contracts

**Public functions** (from AR):

```rust
pub fn detect_model_type(json: &Value) -> Result<OscalModelType, ValidateError>;
pub fn load_schema(model_type: OscalModelType) -> Result<Value, ValidateError>;
pub fn validate_artifact(json: &Value, model_type: OscalModelType) -> Result<ValidationResult, ValidateError>;
```

**CLI changes**:

```rust
// In Commands enum — add --schema-type to Validate
Validate {
    input: PathBuf,
    #[arg(long, value_enum)]
    schema_type: Option<SchemaType>,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum SchemaType {
    Catalog,
    ComponentDefinition,
}
```

**Pipeline integration point** (in `run_catalog_pipeline` and `run_component_pipeline`):
- After `serde_json::to_string_pretty(&envelope)`, parse back to `Value`
- Call `validate_artifact(&value, model_type)`
- If invalid → return `ForgeError::SchemaValidation(...)`, do NOT write output
- If valid → proceed to `write_output`

### ForgeError Extension

```rust
// Add to ForgeError enum
#[error("Schema validation failed: {0}")]
SchemaValidation(String),
```

---

## Phase 2: Implementation Tasks

> **Note**: Task generation via `/speckit.tasks` will produce the full `tasks.md`. Below is the planned task breakdown for reference.

### Task Group 1: Schema Acquisition & Embedding

1. **Download OSCAL v1.2.0 JSON schemas** from NIST OSCAL releases
   - Create `schemas/` directory
   - Download `oscal_catalog_schema.json` and `oscal_component-definition_schema.json`
   - Add `schemas/README.md` documenting the NIST release source and commit hash (SEC-9)

2. **Add jsonschema crate to Cargo.toml**
   - `cargo add jsonschema@latest`
   - Run `cargo audit` to verify no advisories
   - Verify license compatibility (MIT)

### Task Group 2: Validation Module Core (TDD)

3. **Define types and error enum** in `src/validate/mod.rs`
   - `OscalModelType`, `ValidationResult`, `SchemaError`, `ValidateError`
   - Add `SchemaValidation` variant to `ForgeError` in `src/error.rs`
   - Re-export from `src/lib.rs`

4. **Implement `detect_model_type()`** with TDD
   - Tests first: catalog detected, component-definition detected, unknown type errors
   - Implementation: check `json.get("catalog")` and `json.get("component-definition")`

5. **Implement `load_schema()`** with TDD
   - Tests first: loads catalog schema, loads component-definition schema, both parse to valid JSON
   - Implementation: `include_str!` + `serde_json::from_str`

6. **Implement `validate_artifact()`** with TDD
   - Tests first: valid catalog passes, invalid catalog returns errors, all errors collected (not just first)
   - Implementation: `jsonschema::validator_for` + `iter_errors` + collect to `Vec<SchemaError>`
   - Verify `instance_path` and `schema_path` populated (PRD S-2)

### Task Group 3: CLI Integration

7. **Add `--schema-type` flag to Validate subcommand** in `src/cli/mod.rs`
   - Add `SchemaType` enum and optional `schema_type` arg to `Validate` variant
   - Update CLI tests for parsing

8. **Wire `forge validate` to validation module** in `src/cli/validate.rs`
   - Read file, check size limit (SEC-3), parse JSON
   - Detect model type (or use override), validate, format output
   - Exit 0 on valid, exit 1 on invalid (PRD M-4, M-5)
   - Edge cases: file not found, empty file (SEC-5), non-JSON (SEC-4), unknown model (SEC-6)

### Task Group 4: Pipeline Integration

9. **Add auto-validation gate to `forge convert`** in `src/pipeline.rs`
   - After serialization, parse back to `Value` (AR: validate serialized JSON, not in-memory model)
   - Call `validate_artifact` with appropriate model type
   - If invalid → return `ForgeError::SchemaValidation` with error summary, do NOT call `write_output` (SEC-8)
   - Add to both `run_catalog_pipeline` and `run_component_pipeline`

### Task Group 5: Integration Tests

10. **Integration test: `forge validate` on valid artifacts**
    - Generate a catalog via `forge convert`, then validate the output
    - Test with both Catalog and Component Definition

11. **Integration test: `forge validate` on invalid artifacts**
    - Hand-crafted JSON missing required fields
    - Verify all errors reported, non-zero exit

12. **Integration test: `forge convert` auto-validation**
    - Verify normal `forge convert` succeeds (output is schema-valid)
    - Verify pipeline integration does not break existing tests

13. **Edge case tests** (SEC-4, SEC-5, SEC-6)
    - Empty file, non-JSON file, unknown model type
    - `--schema-type` override respected (EC-7)

---

## Implementation Guardrails Checklist (from AR)

- [ ] **DO NOT** download schemas at runtime — use `include_str!` at compile time
- [ ] **DO NOT** stop at the first validation error — exhaust iterator to collect all errors
- [ ] **DO NOT** skip auto-validation in `forge convert`
- [ ] **DO NOT** validate the in-memory model — validate the serialized JSON
- [ ] **MUST** support both Catalog and Component Definition model types
- [ ] **MUST** exit with code 0 on valid and non-zero on invalid
- [ ] **MUST** pin schemas to a specific NIST OSCAL release
- [ ] **MUST** use `.unwrap()`-free error handling for schema compilation and validation

## Security Requirements Checklist (from SEC)

- [ ] SEC-3: File size check before parsing (50MB limit)
- [ ] SEC-4: Non-JSON files produce descriptive parse error, not panic
- [ ] SEC-5: Empty files produce descriptive error message
- [ ] SEC-6: Unrecognized top-level keys produce UnknownModelType error with `--schema-type` guidance
- [ ] SEC-7: Schema compilation uses `.unwrap()`-free error handling
- [ ] SEC-8: Auto-validation blocks output writing on failure
- [ ] SEC-9: Embedded schemas sourced from pinned NIST OSCAL release

## Deferred to WI-20

- SEC-1: Truncate raw artifact field values in error output (>100 chars)
- SEC-2: Do not expose raw jsonschema crate error messages to users
