# Research: 019-schema-validation

**Date**: 2026-02-13 | **Branch**: `019-schema-validation`

## R-1: jsonschema Crate API

### Question
Which Rust JSON Schema validation library to use, and how does its API support collecting all errors with instance paths?

### Decision
Use the `jsonschema` crate at latest stable version (0.26.x+).

### Key API Surface

```rust
// Compile schema once (reusable)
let validator = jsonschema::validator_for(&schema_value)?;

// Collect ALL errors (does NOT stop at first)
for error in validator.iter_errors(&instance) {
    eprintln!("Error: {error}");
    eprintln!("Location: {}", error.instance_path);
}

// Boolean shortcut
let is_valid: bool = validator.is_valid(&instance);

// First error only
let result: Result<(), ValidationError> = validator.validate(&instance);
```

### ValidationError Fields
- `Display` → human-readable error message
- `instance_path` → JSON pointer to the failing element (e.g., `/catalog/metadata/title`)
- `schema_path` → JSON pointer within the schema that was violated

### Rationale
- Active maintenance (frequent releases, responsive to issues)
- MIT licensed — compatible with FORGE's MIT license
- Supports Draft 2020-12 and Draft-07 (covers OSCAL schema features)
- `iter_errors()` collects ALL errors per PRD M-5 requirement
- `instance_path` fulfills PRD S-2 requirement
- Pure Rust — memory-safe, no FFI boundaries

### Alternatives Considered
| Library | Why Rejected |
|---------|-------------|
| `valico` | Less actively maintained, fewer features |
| Custom validator | Infeasible — hundreds of schema constraints to encode manually (AR Option 2 rejected) |
| `oscal-cli` shelling | External dependency (Java runtime), breaks offline requirement (AR Option 3) |

---

## R-2: OSCAL JSON Schemas (from v1.2.0 release, Draft-07)

### Question
Where to obtain OSCAL JSON schemas, whether they're self-contained for `$ref` resolution, and how to pin them?

### Decision
Download from NIST OSCAL GitHub release tag `v1.2.0`:
- `oscal_catalog_schema.json` — `$id` references OSCAL v1.2.0
- `oscal_component_schema.json` — `$id` references OSCAL v1.1.3 (model unchanged between releases; this is the filename NIST uses in the v1.2.0 release assets)

### Key Findings
1. **Self-contained**: NIST publishes each model schema as a standalone JSON Schema file with all definitions (`$defs`) inlined. Internal `$ref` pointers (e.g., `#/$defs/...`) resolve within the same file.
2. **No external `$ref`**: Unlike some JSON Schema ecosystems that use external `$ref` URIs, OSCAL schemas bundle everything needed into each model file.
3. **Schema draft**: Both schemas use JSON Schema Draft-07 (`$schema: "http://json-schema.org/draft-07/schema#"`), which is fully supported by the `jsonschema` crate.
4. **Component schema versioning**: The component-definition schema's `$id` references v1.1.3 because NIST did not change this model between v1.1.3 and v1.2.0. The file is the official asset from the v1.2.0 release.

### Pin Strategy
- Store schema files in `schemas/` directory at project root
- Document the NIST release URL and commit hash in `schemas/README.md`
- Schemas are embedded via `include_str!()` at compile time — no runtime download
- Update process: replace files in `schemas/` + rebuild

### Source URLs
- Release page: `https://github.com/usnistgov/OSCAL/releases/tag/v1.2.0`
- Direct download: assets attached to the GitHub release

---

## R-3: File Size Limit Strategy (SEC-3)

### Question
How to enforce a file size limit for `forge validate` to prevent OOM attacks from extremely large JSON inputs?

### Decision
Enforce a 50MB file size limit before reading file content.

### Implementation
```rust
const MAX_VALIDATE_FILE_SIZE: u64 = 50 * 1024 * 1024; // 50MB

fn check_file_size(path: &Path) -> Result<(), ValidateError> {
    let metadata = std::fs::metadata(path)
        .map_err(|e| ValidateError::FileRead { path: path.to_path_buf(), source: e })?;
    let size = metadata.len();
    if size > MAX_VALIDATE_FILE_SIZE {
        return Err(ValidateError::FileTooLarge {
            size_mb: size as f64 / (1024.0 * 1024.0),
            limit_mb: 50,
        });
    }
    Ok(())
}
```

### Rationale
- Consistent with SEC-3 recommendation (50MB)
- `std::fs::metadata()` is a lightweight stat call — no file content read
- Checked before `std::fs::read_to_string()` which is the memory-intensive step
- 50MB is generous for OSCAL artifacts (typical catalogs are <1MB) but protects against adversarial inputs
- No CLI override in this WI — simplicity per constitution X

### Alternatives Considered
- **No limit**: Rejected — SEC-3 explicitly requires file size check
- **10MB limit**: Too restrictive — large OSCAL catalogs (NIST SP 800-53) could approach 5-10MB
- **100MB limit**: Unnecessary — no real-world OSCAL artifact approaches this size
