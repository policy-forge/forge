# Quickstart: Validation Error Reporting (WI-20)

**Date**: 2026-02-14
**Prerequisite**: WI-19 (schema validation integration) completed

---

## Overview

WI-20 enhances FORGE's validation error reporting with actionable messages. The implementation adds 4 new modules to `src/validate/` and modifies 3 existing files.

## Implementation Order

Follow this sequence (dependencies flow top-to-bottom):

### Phase A: Core Types (no dependencies)

1. **`src/validate/error_types.rs`** — Define `ValidationErrorCategory`, `ValidationError`, `ValidationReport`
   - Derive Serialize/Deserialize for JSON output
   - Implement `ValidationReport::new()` builder with invariant enforcement
   - TDD: Write tests for invariants (`is_valid == errors.is_empty()`, count consistency)

### Phase B: Formatter (depends on Phase A)

2. **`src/validate/formatter.rs`** — Implement `pointer_to_json_path()` and `format_schema_error()`
   - `pointer_to_json_path()`: Convert JSON Pointer → JSON Path notation
   - `truncate_value()`: Truncate actual values to 100 chars (SEC-1)
   - `format_schema_error()`: Transform raw jsonschema error into `ValidationError`
   - TDD: Write tests for path conversion (simple, array indices, deeply nested, empty)
   - TDD: Write tests for truncation (short value, exactly 100, over 100)
   - TDD: Write tests for error formatting (missing field, wrong type, invalid enum)

### Phase C: Semantic Validator (depends on Phase A)

3. **`src/validate/semantic.rs`** — Implement `SemanticValidator`
   - `check_orphaned_links(json: &Value) -> Vec<ValidationError>`: Walk JSON tree, find `href="#uuid"` references, check back-matter
   - `check_missing_references(json: &Value, model_type: OscalModelType) -> Vec<ValidationError>`: Check control-id references in implemented-requirements
   - `validate(&self, json: &Value, model_type: OscalModelType) -> Vec<ValidationError>`: Orchestrate semantic checks
   - TDD: Write tests for orphaned links (found, none, no back-matter, multiple)
   - TDD: Write tests for missing references (missing, valid, empty list)

### Phase D: Report Renderers (depends on Phase A)

4. **`src/validate/report.rs`** — Implement `render_text_report()` and `render_json_report()`
   - Text format: summary line + categorized error list
   - JSON format: serde serialization of ValidationReport
   - TDD: Write tests for valid report, single error, mixed categories, 50+ errors

### Phase E: Integration (depends on Phases A-D)

5. **`src/validate/mod.rs`** — Add `run_full_validation()` orchestrator
   - `pub fn run_full_validation(artifact_path: &str, json: &Value, model_type: OscalModelType) -> Result<ValidationReport, ValidateError>`
   - Call `load_schema()` + `jsonschema::validator_for()` for raw schema errors
   - Transform via `format_schema_error()`
   - Call `SemanticValidator` for semantic errors
   - Combine into `ValidationReport`
   - Re-export new public types

6. **`src/cli/mod.rs`** — Add `--format text|json` to validate subcommand

7. **`src/cli/validate.rs`** — Use `run_full_validation()` + renderers
   - Replace raw error output with `render_text_report()` or `render_json_report()`

8. **`src/pipeline.rs`** — Update auto-validation in catalog and component pipelines
   - Replace raw `SchemaError` formatting with `ValidationReport`
   - Add semantic validation to auto-validation
   - Use `render_text_report()` for stderr output on failure

## Key Files Quick Reference

| File | Status | Changes |
|------|--------|---------|
| `src/validate/error_types.rs` | NEW | ValidationError, ValidationErrorCategory, ValidationReport |
| `src/validate/formatter.rs` | NEW | pointer_to_json_path(), format_schema_error(), truncate_value() |
| `src/validate/semantic.rs` | NEW | SemanticValidator, check_orphaned_links(), check_missing_references() |
| `src/validate/report.rs` | NEW | render_text_report(), render_json_report() |
| `src/validate/mod.rs` | MODIFY | Add run_full_validation(), re-export new types, add module declarations |
| `src/cli/mod.rs` | MODIFY | Add --format flag to validate subcommand |
| `src/cli/validate.rs` | MODIFY | Use run_full_validation() + renderers |
| `src/pipeline.rs` | MODIFY | Update auto-validation to use ValidationReport |

## Build Verification

After each phase, verify:
```bash
cargo fmt --check        # Formatting
cargo clippy -- -D warnings  # Linting
cargo test               # All tests pass
```

## Security Checklist (from SEC 020)

- [ ] SEC-1: Actual values truncated to 100 chars in `truncate_value()`
- [ ] SEC-2: Raw crate messages never exposed — verify `format_schema_error()` tests
- [ ] SEC-3: JSON report contains only defined fields — verify serialization tests
- [ ] SEC-4: No Rust module paths in error messages — verify integration tests
- [ ] SEC-5: No external URL following — verify `check_orphaned_links()` tests
- [ ] SEC-6: Malformed JSON pointers handled gracefully — verify `pointer_to_json_path()` tests
- [ ] SEC-7: Auto-validation errors to stderr only — verify pipeline integration tests
- [ ] SEC-8: Error counts consistent — verify `ValidationReport` invariant tests
