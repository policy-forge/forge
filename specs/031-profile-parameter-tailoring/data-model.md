# Data Model: 031 Profile Parameter Tailoring

**Phase**: 1 | **Date**: 2026-02-18 | **Branch**: `031-profile-parameter-tailoring`

## New Types

### `Modify` (`src/oscal/profile.rs`)

Holds the OSCAL Profile `modify` section. WI-31 scope: `set-parameters` only. `alter` directives are explicitly out of scope (W-2).

```rust
/// OSCAL Profile `modify` section.
///
/// Contains parameter value overrides for imported controls.
/// WI-31 scope: `set-parameters` only. `alter` directives are future scope (W-2).
#[derive(Debug, Serialize)]
pub struct Modify {
    /// Parameter value overrides, ordered alphabetically by `param-id`.
    #[serde(rename = "set-parameters")]
    pub set_parameters: Vec<SetParameter>,
}
```

**OSCAL JSON key**: `"set-parameters"` (OSCAL v1.2.0 Profile model)

**Invariant**: `set_parameters` is always non-empty when `Modify` is constructed by `build_modify_section`. An empty `Modify` is not produced.

---

### `SetParameter` (`src/oscal/profile.rs`)

A single parameter override entry in `modify.set-parameters`.

```rust
/// A single parameter override in `modify.set-parameters`.
///
/// `param-id` is an opaque string identifier from the source catalog.
/// Not validated at this stage — catalog-aware validation is WI-32's scope.
#[derive(Debug, Serialize)]
pub struct SetParameter {
    /// Parameter identifier (opaque; not validated against source catalog here).
    #[serde(rename = "param-id")]
    pub param_id: String,

    /// Override value(s) for this parameter.
    ///
    /// Single-element when one `--set-param prm val` is provided.
    /// Multi-element when the same `param-id` appears in multiple `--set-param` flags.
    pub values: Vec<String>,
}
```

**OSCAL JSON keys**: `"param-id"`, `"values"` (OSCAL v1.2.0 Profile model)

**Invariant**: `values` is always non-empty. An empty-string value (`""`) is valid per OSCAL.

---

## Modified Types

### `OscalProfile` (extended — `src/oscal/profile.rs`)

New optional `modify` field added after `imports`. Skipped during serialization when `None` (FR-006).

```rust
#[derive(Debug, Serialize)]
pub struct OscalProfile {
    pub uuid: Uuid,
    pub metadata: OscalMetadata,
    pub imports: Vec<ProfileImport>,

    /// Optional modify section (WI-31). Absent when no `--set-param` flags provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modify: Option<Modify>,
}
```

---

## New Functions

### `build_modify_section` (`src/oscal/profile.rs`)

Pure function: no side effects, no I/O, no panic paths. Returns `None` for empty input (backward compatibility).

```rust
/// Build the Profile `modify` section from `--set-param` pairs.
///
/// Returns `None` if `param_overrides` is empty, ensuring the `modify` key is
/// absent from the Profile JSON when no `--set-param` flags are provided (FR-006).
///
/// Duplicate `param-id` entries are aggregated into a single `SetParameter` with
/// a combined `values` array (FR-007, S-1). Entries are sorted alphabetically by
/// `param-id` for deterministic output (FR-008, S-2).
///
/// # Examples
///
/// ```
/// use forge::oscal::profile::build_modify_section;
///
/// // Single parameter
/// let modify = build_modify_section(&[
///     ("POL-AC-001_prm".to_string(), "60 days".to_string()),
/// ]).unwrap();
/// assert_eq!(modify.set_parameters[0].param_id, "POL-AC-001_prm");
/// assert_eq!(modify.set_parameters[0].values, ["60 days"]);
///
/// // Empty → None
/// assert!(build_modify_section(&[]).is_none());
/// ```
#[tracing::instrument(skip_all, fields(param_count = param_overrides.len()))]
pub fn build_modify_section(param_overrides: &[(String, String)]) -> Option<Modify> {
    if param_overrides.is_empty() {
        return None;
    }
    let mut grouped: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for (id, value) in param_overrides {
        grouped.entry(id.clone()).or_default().push(value.clone());
    }
    let set_parameters = grouped
        .into_iter()
        .map(|(param_id, values)| SetParameter { param_id, values })
        .collect();
    Some(Modify { set_parameters })
}
```

---

### `parse_set_param_pairs` (private helper — `src/cli/profile.rs`)

Converts clap's flattened `Vec<String>` into typed pairs. Private; called only by `execute`.

```rust
/// Convert the flattened `--set-param` Vec into (param_id, value) pairs.
///
/// clap with `num_args = 2` + `ArgAction::Append` produces a flattened Vec:
/// `["id1", "val1", "id2", "val2"]` → `[("id1", "val1"), ("id2", "val2")]`.
///
/// Panics: cannot panic — clap guarantees `num_args = 2` produces an even-length Vec.
fn parse_set_param_pairs(set_params: &[String]) -> Vec<(String, String)> {
    set_params
        .chunks_exact(2)
        .map(|chunk| (chunk[0].clone(), chunk[1].clone()))
        .collect()
}
```

---

## Modified Functions

### `build_profile` (extended signature — `src/oscal/profile.rs`)

New `param_overrides` parameter added. All existing callers must be updated to pass `&[]`.

```rust
pub fn build_profile(
    catalog_path: &str,
    control_ids: Vec<String>,
    mode: SelectionMode,
    param_overrides: &[(String, String)],  // NEW (WI-31)
) -> Result<OscalProfile, ForgeError>
```

### `cli/profile.rs execute` (extended signature)

New `set_params` parameter added. Handles C-2 warning when params provided without selection flags.

```rust
pub fn execute(
    catalog: &Path,
    include: Option<&str>,
    exclude: Option<&str>,
    set_params: &[String],   // NEW (WI-31): flattened --set-param pairs
    format: &OutputFormat,
    output: Option<&Path>,
) -> Result<(), ForgeError>
```

---

## Entity Relationships

```
Commands::Profile (cli/mod.rs)
└── set_params: Vec<String>   (flattened pairs from clap num_args=2 + Append)
        │
        ▼  parse_set_param_pairs()
        Vec<(String, String)>
        │
        ▼  build_profile(..., param_overrides)
OscalProfile
├── uuid: Uuid
├── metadata: OscalMetadata      (unchanged from WI-30)
├── imports: Vec<ProfileImport>  (unchanged from WI-30)
└── modify: Option<Modify>       ← NEW (WI-31)
        └── set_parameters: Vec<SetParameter>
                ├── param_id: String
                └── values: Vec<String>
```

## Serialization Verification

Expected JSON output shape with one `--set-param` flag:

```json
{
  "profile": {
    "uuid": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
    "metadata": { "title": "...", "last-modified": "...", "version": "...", "oscal-version": "1.2.0" },
    "imports": [{ "href": "catalog.json", "include-controls": [{ "with-ids": ["POL-AC-001"] }] }],
    "modify": {
      "set-parameters": [
        { "param-id": "POL-AC-001_prm", "values": ["60 days"] }
      ]
    }
  }
}
```

Expected JSON output shape with no `--set-param` (backward compatible — identical to WI-30):

```json
{
  "profile": {
    "uuid": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
    "metadata": { "title": "...", "last-modified": "...", "version": "...", "oscal-version": "1.2.0" },
    "imports": [{ "href": "catalog.json", "include-controls": [{ "with-ids": ["POL-AC-001"] }] }]
  }
}
```
