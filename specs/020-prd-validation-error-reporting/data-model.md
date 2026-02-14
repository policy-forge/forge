# Data Model: Validation Error Reporting (WI-20)

**Date**: 2026-02-14
**Source**: PRD 020, AR 020, SEC 020

---

## Entities

### ValidationErrorCategory

**Purpose**: Classifies validation errors by source (PRD M-6).

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| (enum variant) | `Schema` | Error from JSON Schema validation | — |
| (enum variant) | `Semantic` | Error from semantic validation (orphaned links, missing references) | — |

**Derives**: `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Serialize`, `Deserialize`, `Hash`

**Relationships**: Referenced by `ValidationError.category`

---

### ValidationError

**Purpose**: A single validation error with full context for actionable reporting (PRD M-1).

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| `category` | `ValidationErrorCategory` | Schema or Semantic classification | Required (PRD M-6) |
| `path` | `String` | JSON Path to offending field (e.g., `$.catalog.metadata.uuid`) | Required; JSON Path notation, not JSON Pointer (AR decision) |
| `message` | `String` | Human-readable error description | Required; must not contain raw crate messages (SEC-2) |
| `expected` | `String` | What schema/rule expected (e.g., "required string field") | Required (PRD M-1) |
| `actual` | `String` | What was found (e.g., "field not present") | Required; truncated to 100 content chars with "..." appended (103 chars total) (SEC-1) |

**Derives**: `Debug`, `Clone`, `Serialize`, `Deserialize`, `PartialEq`

**Validation Rules**:
- `actual` field MUST be truncated to 100 content characters with `"..."` appended when longer (103 characters total) (SEC-1)
- `path` MUST use JSON Path notation (`$.field.name`), not JSON Pointer (`/field/name`)
- `message` MUST NOT contain raw `jsonschema` crate error text (SEC-2)
- `message` MUST NOT contain Rust module paths, type names, or stack traces (SEC-4)

**Relationships**:
- Contained in `ValidationReport.errors`
- Created by `format_schema_error()` (for schema errors)
- Created by `SemanticValidator` methods (for semantic errors)

---

### ValidationReport

**Purpose**: Aggregated validation result with all errors and summary counts (PRD M-2, S-2).

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| `artifact_path` | `String` | Path to the validated artifact | Required |
| `is_valid` | `bool` | Whether artifact passed all validation | `true` iff `errors.is_empty()` |
| `errors` | `Vec<ValidationError>` | All collected errors (both schema and semantic) | May be empty |
| `schema_error_count` | `usize` | Count of errors where `category == Schema` | Must equal `errors.iter().filter(Schema).count()` (SEC-8) |
| `semantic_error_count` | `usize` | Count of errors where `category == Semantic` | Must equal `errors.iter().filter(Semantic).count()` (SEC-8) |

**Derives**: `Debug`, `Clone`, `Serialize`, `Deserialize`

**Invariants**:
- `is_valid == errors.is_empty()` — always consistent
- `schema_error_count + semantic_error_count == errors.len()` — always consistent (SEC-8)

**Validation Rules**:
- Report MUST contain ALL errors from both schema and semantic passes (PRD M-2)
- Report MUST NOT be truncated even with 50+ errors (PRD EC-2)

**Relationships**:
- Contains `Vec<ValidationError>`
- Rendered by `render_text_report()` and `render_json_report()`
- Built by validation orchestrator after both passes complete

---

### OscalModelType (Existing — WI-19)

**Purpose**: Identifies which OSCAL model type to validate against.

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| (enum variant) | `Catalog` | OSCAL Catalog model | — |
| (enum variant) | `ComponentDefinition` | OSCAL Component Definition model | — |

**Note**: Already exists in `src/validate/mod.rs`. No changes needed for WI-20.

---

## State Transitions

```
Loading → SchemaValidation → SemanticValidation → ReportAssembly → OutputRendering
                                                                         ↓
                                                                  Valid (exit 0) | Invalid (exit non-zero)
```

Validation is a single-pass operation with no persistent state. Each invocation:
1. Loads artifact → validates schema → validates semantics → assembles report → renders output
2. No state persists between invocations

---

## Relationships Diagram

```
ValidationReport
├── artifact_path: String
├── is_valid: bool
├── errors: Vec<ValidationError>
│   └── ValidationError
│       ├── category: ValidationErrorCategory (Schema | Semantic)
│       ├── path: String (JSON Path)
│       ├── message: String
│       ├── expected: String
│       └── actual: String (max 100 chars)
├── schema_error_count: usize
└── semantic_error_count: usize
```

---

## Migration from WI-19 Types

| WI-19 Type | WI-20 Type | Migration |
|------------|------------|-----------|
| `SchemaError` | `ValidationError` (category: Schema) | Replace; SchemaError fields map to ValidationError fields via `format_schema_error()` |
| `ValidationResult` | `ValidationReport` | Replace; ValidationResult.errors maps to formatted ValidationReport.errors |
| `SchemaError.message` | `ValidationError.message` | Reformatted from raw crate message to user-friendly text |
| `SchemaError.instance_path` | `ValidationError.path` | Converted from JSON Pointer to JSON Path via `pointer_to_json_path()` |
| `SchemaError.schema_path` | (used internally) | Used by `format_schema_error()` to derive `expected` constraint; not exposed in output |

**Backward Compatibility**: WI-19's `SchemaError` and `ValidationResult` types are replaced by the new types. The `validate_artifact()` function signature is unchanged — it still returns raw errors. The new formatting layer wraps the existing function.
