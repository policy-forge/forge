# Research: Validation Error Reporting (WI-20)

**Date**: 2026-02-14
**Status**: Complete — no NEEDS CLARIFICATION items remain

---

## R-1: jsonschema Crate Error API (v0.41.0)

**Decision**: Use `validator.iter_errors()` which returns an iterator of `jsonschema::ValidationError` objects. Each error exposes `instance_path()` (JSON Pointer to failing value) and `schema_path()` (JSON Pointer within schema). The `to_string()` method provides a human-readable message.

**Rationale**: The existing WI-19 code (`src/validate/mod.rs:141-152`) already uses this exact API pattern:
```rust
validator.iter_errors(json).map(|error| {
    let instance_path = error.instance_path().to_string();
    let schema_path = error.schema_path().to_string();
    SchemaError { message: error.to_string(), instance_path, schema_path }
})
```
WI-20 needs to enhance this mapping by:
1. Converting `instance_path` (JSON Pointer) to JSON Path notation
2. Extracting the actual value from the JSON at the instance path
3. Deriving the expected constraint from the error message and schema path

**Alternatives considered**:
- Using `error.kind` for structured error details (Python API supports this, but Rust `to_string()` is sufficient and simpler)
- Using the `mask` feature for redacting sensitive values (overkill — manual truncation to 100 chars is simpler and already specified in SEC-1)

---

## R-2: JSON Pointer to JSON Path Conversion

**Decision**: Implement a simple `pointer_to_json_path()` function that converts RFC 6901 JSON Pointer notation to JSON Path notation.

**Rationale**: JSON Pointer uses `/` separators (e.g., `/catalog/metadata/uuid`), while JSON Path uses `$.` prefix with `.` separators and `[n]` for array indices (e.g., `$.catalog.metadata.uuid`). JSON Path is more widely recognized by compliance engineers.

**Algorithm**:
1. Handle empty pointer → `"$"` (root)
2. Split by `"/"`
3. Filter empty leading segment
4. Unescape RFC 6901 sequences: replace `~1` with `/`, then `~0` with `~`
5. For numeric segments → `[n]` array index notation
6. For string segments → `.name` property notation
7. Prefix with `"$"`

**Examples**:
| JSON Pointer | JSON Path |
|---|---|
| `` (empty) | `$` |
| `/catalog` | `$.catalog` |
| `/catalog/metadata/uuid` | `$.catalog.metadata.uuid` |
| `/catalog/groups/0/controls/2/id` | `$.catalog.groups[0].controls[2].id` |
| `/catalog/groups/0/controls/5/parts/0/props/3/value` | `$.catalog.groups[0].controls[5].parts[0].props[3].value` |

**Alternatives considered**:
- Using JSON Pointer directly (less familiar to target users per PRD decision log)
- Using jq-style paths (non-standard, less tooling support)

---

## R-3: Extracting Expected and Actual Values

**Decision**: Extract expected constraint from the jsonschema error message string. Extract actual value by navigating the JSON document at the instance path.

**Rationale**: The `jsonschema` crate's `ValidationError::to_string()` produces messages like:
- `"\"not-a-uuid\" is not valid under any of the given schemas"` — actual is embedded in message
- `"\"metadata\" is a required property"` — expected is "required field", actual is "missing"
- `"42 is not of type \"string\""` — expected is "string", actual is "42"

For the actual value, navigate to `instance_path` in the JSON tree and serialize the value. Truncate to 100 characters per SEC-1.

For expected constraint, parse common patterns from the error message:
- "required property" → expected: "required field", actual: "field not present"
- "is not of type" → expected: the type name
- "is not valid under" → expected: "valid schema match"
- "is longer than N" / "is shorter than N" → expected: length constraint
- Pattern/format violations → expected: the pattern/format

**Alternatives considered**:
- Walking the schema alongside the document to extract constraints (overly complex; error message parsing is simpler and sufficient)
- Using a generic error classification system (YAGNI per constitution X)

---

## R-4: Semantic Validation — Orphaned Link Detection

**Decision**: Walk the JSON tree to find all `href` values starting with `#`, extract the UUID after `#`, and check against the set of `back-matter.resources[].uuid` values.

**Rationale**: OSCAL uses `href="#<uuid>"` to reference back-matter resources. An orphaned link references a UUID that doesn't exist in `back-matter.resources[]`. This is a common authoring error that schema validation cannot catch.

**Algorithm**:
1. Collect resource UUIDs from `json["<model-root>"]["back-matter"]["resources"]` into a `HashSet<String>`
2. If no back-matter or no resources, the set is empty
3. Recursively walk the entire JSON tree
4. Track current JSON Path during traversal
5. For each object with an `"href"` key where the value starts with `"#"`:
   - Extract UUID = value after `"#"`
   - If UUID not in resource set → emit `ValidationError` (category: Semantic)
6. Return collected errors

**Alternatives considered**:
- Only checking `link` elements (misses `href` in other contexts like `import-profile`)
- Following external URLs (explicitly prohibited by SEC-5)

---

## R-5: Semantic Validation — Missing Reference Detection

**Decision**: For Component Definitions, collect `control-id` values from `implemented-requirements` and report them as semantic warnings when no source catalog/profile is available for cross-referencing.

**Rationale**: An `implemented-requirement` references a `control-id` from a catalog or profile. Without the source catalog loaded, we cannot verify these references. However, we can detect obviously invalid references (empty strings, malformed IDs) and report unverifiable references as informational.

**Scope limitation**: Full cross-reference validation requires loading the source catalog/profile (which is available via `--source-profile` in the component pipeline). For WI-20, implement:
1. Check that `control-id` fields are non-empty strings
2. If no back-matter or source profile context is available, skip cross-reference checks gracefully
3. Check for duplicate `control-id` references within the same component

**Alternatives considered**:
- Loading the source catalog/profile to verify control-ids (scope creep — requires network or additional file I/O; deferred)
- Skipping missing reference detection entirely (would leave PRD M-4 unsatisfied)

---

## R-6: Auto-Validation Integration in forge convert

**Decision**: Replace the existing raw error formatting in `pipeline.rs` (catalog: lines 153-173, component: lines 226-247) with the new `ValidationReport`-based error reporting.

**Rationale**: Auto-validation already exists in `pipeline.rs` but uses the WI-19 raw `SchemaError` format. WI-20 enhances this by:
1. Running schema validation through `format_schema_error()` to produce actionable errors
2. Running semantic validation as a second pass
3. Combining both into a `ValidationReport`
4. Using `render_text_report()` to format errors to stderr
5. Failing with non-zero exit code if any errors found

The existing pattern serializes JSON, re-parses to `Value`, validates, and only writes output if valid. WI-20 preserves this flow but upgrades the error presentation.

**Alternatives considered**:
- Validating the in-memory domain model (misses serialization bugs — rejected per PRD anti-pattern)
- Moving auto-validation to the CLI layer (breaks encapsulation — pipeline should guarantee valid output)

---

## R-7: Error Output Format (--format json)

**Decision**: Add `--format text|json` flag to `forge validate` (default: `text`). Serialize `ValidationReport` as JSON via serde for `--format json`.

**Rationale**: PRD S-1 requires machine-parseable structured error output. The `ValidationReport` struct already derives `Serialize`, so JSON output is straightforward via `serde_json::to_string_pretty()`.

**JSON output structure** (mirrors ValidationReport):
```json
{
  "artifact_path": "catalog.json",
  "is_valid": false,
  "errors": [
    {
      "category": "Schema",
      "path": "$.catalog.metadata.uuid",
      "message": "required field missing",
      "expected": "required string field",
      "actual": "field not present"
    }
  ],
  "schema_error_count": 1,
  "semantic_error_count": 0
}
```

**Alternatives considered**:
- SARIF format (PRD C-2 — nice to have, deferred)
- Custom JSON structure (unnecessary — ValidationReport already has the right shape)
