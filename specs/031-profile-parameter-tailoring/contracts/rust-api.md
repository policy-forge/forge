# Rust API Contract: 031 Profile Parameter Tailoring

**Version**: 1.0 | **Date**: 2026-02-18 | **Branch**: `031-profile-parameter-tailoring`

## Public API (`src/oscal/profile.rs`)

### New Types

```rust
/// OSCAL Profile `modify` section with `set-parameters` array.
pub struct Modify {
    pub set_parameters: Vec<SetParameter>,
}

/// Single parameter override in `modify.set-parameters`.
pub struct SetParameter {
    pub param_id: String,
    pub values: Vec<String>,
}
```

### New Function

```rust
/// Build the Profile `modify` section from (param_id, value) pairs.
///
/// Returns `None` for empty input (no `modify` section in output — backward compat).
/// Aggregates duplicate param-ids; sorts alphabetically by param-id.
pub fn build_modify_section(param_overrides: &[(String, String)]) -> Option<Modify>;
```

### Modified Function

```rust
/// Extended with `param_overrides` parameter (WI-31).
/// Empty slice → no `modify` section (identical to WI-30 output).
pub fn build_profile(
    catalog_path: &str,
    control_ids: Vec<String>,
    mode: SelectionMode,
    param_overrides: &[(String, String)],
) -> Result<OscalProfile, ForgeError>;
```

## CLI Interface Contract

```
forge profile
  --catalog <path>               Required: path to OSCAL Catalog JSON
  [--include <comma-ids>]        Mutually exclusive with --exclude
  [--exclude <comma-ids>]        Mutually exclusive with --include
  [--set-param <id> <value>]...  Repeatable: zero or more parameter overrides
  [--format json]                Default: json (only json supported in WI-31)
  [--output <path>]              Optional: file path; default stdout
```

**`--set-param` semantics**:
- Takes exactly two arguments per occurrence: `<param-id>` and `<value>`
- Repeatable: `--set-param prm1 "v1" --set-param prm2 "v2"` produces two entries
- Same `param-id` twice: values combined into one entry (`values: ["v1", "v2"]`)
- Values containing spaces must be shell-quoted: `--set-param prm "60 days"`
- Empty value string is valid: `--set-param prm ""`

## JSON Output Invariants

1. `"modify"` key is **absent** when no `--set-param` flags are provided
2. `"modify"` key is **present** when one or more `--set-param` flags are provided
3. `"set-parameters"` is always a non-empty array when `"modify"` is present
4. Entries in `"set-parameters"` are ordered **alphabetically by `"param-id"`**
5. Each entry contains exactly `"param-id"` (string) and `"values"` (array of strings)
6. Same inputs always produce **byte-for-byte identical** output

## Error Conditions

| Condition | Behavior |
|-----------|----------|
| Both `--include` and `--exclude` provided | `ForgeError::InvalidArgument` (clap `conflicts_with` catches first) |
| Neither `--include` nor `--exclude` with no `--set-param` | `ForgeError::InvalidArgument` |
| Neither `--include` nor `--exclude` with `--set-param` (C-2) | Warning to stderr; continue with no imports |
| Catalog path does not exist | `ForgeError::FileNotFound` |
| Output file write failure | `ForgeError::Io` |
| JSON serialization failure | `ForgeError::Serialization` |

## Test Contract

| Scenario | Expected Result |
|----------|----------------|
| `--set-param POL-AC-001_prm "60 days"` | `modify.set-parameters` has one entry |
| Two distinct `--set-param` flags | `modify.set-parameters` has two entries, alphabetically ordered |
| Same `param-id` twice | Single entry with two-element `values` array |
| No `--set-param` flags | No `"modify"` key in output |
| `--set-param` with space in value | Value preserved intact as single string |
| `--set-param prm ""` | Entry generated with `values: [""]` |
| Ten distinct `--set-param` flags | All ten entries in `set-parameters` |
| Same inputs twice | Byte-for-byte identical output |
