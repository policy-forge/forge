//! OSCAL Profile generation for WI-30 and WI-31.
//!
//! Provides `build_profile`, `parse_control_ids`, and `build_modify_section`
//! plus the structs required to produce a valid OSCAL v1.2.0 Profile JSON artifact.

use std::collections::BTreeMap;

use serde::Serialize;
use uuid::Uuid;

use crate::error::ForgeError;
use crate::model::DocumentMetadata;
use crate::oscal::metadata::{OscalMetadata, assemble_metadata};

// ---------------------------------------------------------------------------
// Root wrapper
// ---------------------------------------------------------------------------

/// Root wrapper for OSCAL Profile JSON: `{"profile": {...}}`.
///
/// Serializes with the OSCAL-required `"profile"` root key.
#[derive(Debug, Serialize)]
pub struct ProfileRoot {
    /// The OSCAL Profile object (serialized under key `profile`).
    pub profile: OscalProfile,
}

// ---------------------------------------------------------------------------
// OSCAL Profile model
// ---------------------------------------------------------------------------

/// OSCAL Profile model. Contains metadata and one or more import entries.
///
/// WI-30 produces profiles with exactly one [`ProfileImport`].
/// WI-31 adds the optional [`Modify`] section for parameter tailoring.
#[derive(Debug, Serialize)]
pub struct OscalProfile {
    /// Deterministic UUID v5 — derived from catalog path and selected control IDs.
    pub uuid: Uuid,

    /// OSCAL metadata (title, last-modified, version, oscal-version).
    pub metadata: OscalMetadata,

    /// Import entries: which catalog(s) to draw controls from.
    pub imports: Vec<ProfileImport>,

    /// Optional parameter override section (WI-31).
    /// Absent when no `--set-param` flags are provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modify: Option<Modify>,
}

// ---------------------------------------------------------------------------
// ProfileImport
// ---------------------------------------------------------------------------

/// A single entry in the Profile's `imports[]` array.
///
/// Valid OSCAL v1.2.0 import shapes:
/// - Include mode: `href` + `include-controls`
/// - Exclude mode: `href` + `include-all: {}` + `exclude-controls`
///   (The OSCAL schema requires `include-all` when using `exclude-controls` alone.)
#[derive(Debug, Serialize)]
pub struct ProfileImport {
    /// URI reference to the source Catalog (stored as-is from `--catalog`).
    pub href: String,

    /// Include all controls from the catalog (`--exclude` mode).
    ///
    /// Required by the OSCAL v1.2.0 Profile schema when `exclude-controls` is present.
    #[serde(skip_serializing_if = "Option::is_none", rename = "include-all")]
    pub include_all: Option<IncludeAll>,

    /// Controls to include (from `--include`).
    #[serde(skip_serializing_if = "Option::is_none", rename = "include-controls")]
    pub include_controls: Option<Vec<ControlSelection>>,

    /// Controls to exclude (from `--exclude`).
    #[serde(skip_serializing_if = "Option::is_none", rename = "exclude-controls")]
    pub exclude_controls: Option<Vec<ControlSelection>>,
}

/// Marker struct for `include-all: {}` — include all controls from the catalog.
///
/// Per the OSCAL v1.2.0 Profile schema, this is an empty object that signals
/// the profile should import every control from the referenced catalog.
#[derive(Debug, Serialize)]
pub struct IncludeAll {}

// ---------------------------------------------------------------------------
// ControlSelection
// ---------------------------------------------------------------------------

/// A control selection with a list of control IDs.
///
/// Used for both `include-controls` and `exclude-controls`.
#[derive(Debug, Serialize)]
pub struct ControlSelection {
    /// Control identifiers to include or exclude.
    #[serde(rename = "with-ids")]
    pub with_ids: Vec<String>,
}

// ---------------------------------------------------------------------------
// SelectionMode
// ---------------------------------------------------------------------------

/// Whether the Profile includes or excludes the specified controls.
///
/// `Include` → populates `include_controls`.
/// `Exclude` → populates `exclude_controls`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    /// Include the specified control IDs in the Profile.
    Include,
    /// Exclude the specified control IDs from the Profile.
    Exclude,
}

// ---------------------------------------------------------------------------
// Modify / SetParameter (WI-31)
// ---------------------------------------------------------------------------

/// OSCAL Profile `modify` section containing parameter overrides.
///
/// When serialized, this struct produces `{"set-parameters": [...]}` per the
/// OSCAL v1.2.0 Profile model. Added by WI-31.
#[derive(Debug, Serialize)]
pub struct Modify {
    /// Array of parameter override entries.
    #[serde(rename = "set-parameters")]
    pub set_parameters: Vec<SetParameter>,
}

/// Single parameter override in `modify.set-parameters`.
///
/// Serializes as `{"param-id": "<id>", "values": ["<v1>", ...]}`.
#[derive(Debug, Serialize)]
pub struct SetParameter {
    /// The parameter identifier.
    #[serde(rename = "param-id")]
    pub param_id: String,

    /// One or more values assigned to this parameter.
    pub values: Vec<String>,
}

// ---------------------------------------------------------------------------
// build_modify_section
// ---------------------------------------------------------------------------

/// Build the Profile `modify` section from `(param_id, value)` pairs.
///
/// Returns `None` for empty input — no `"modify"` key in the serialized output,
/// preserving backward compatibility with WI-30. Aggregates duplicate `param_id`
/// values into a single [`SetParameter`] entry with multiple `values`. Entries
/// are sorted alphabetically by `param_id` for deterministic output.
///
/// # Examples
///
/// ```
/// use forge::oscal::profile::build_modify_section;
///
/// // Empty input → None (no modify section)
/// assert!(build_modify_section(&[]).is_none());
///
/// // Single pair → Some(Modify) with one entry
/// let pairs = vec![("POL-AC-001_prm".to_string(), "60 days".to_string())];
/// let modify = build_modify_section(&pairs).unwrap();
/// assert_eq!(modify.set_parameters.len(), 1);
/// assert_eq!(modify.set_parameters[0].param_id, "POL-AC-001_prm");
/// assert_eq!(modify.set_parameters[0].values, vec!["60 days"]);
/// ```
#[tracing::instrument(skip_all, fields(param_count = param_overrides.len()))]
pub fn build_modify_section(param_overrides: &[(String, String)]) -> Option<Modify> {
    if param_overrides.is_empty() {
        return None;
    }

    let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (id, value) in param_overrides {
        map.entry(id.clone()).or_default().push(value.clone());
    }

    let set_parameters: Vec<SetParameter> =
        map.into_iter().map(|(param_id, values)| SetParameter { param_id, values }).collect();

    Some(Modify { set_parameters })
}

// ---------------------------------------------------------------------------
// build_profile
// ---------------------------------------------------------------------------

/// Build an `OscalProfile` from CLI arguments.
///
/// # Arguments
///
/// * `catalog_path` — Path to the source Catalog, stored as-is in `imports[0].href`.
/// * `control_ids` — Trimmed, deduplicated control IDs from `--include` or `--exclude`.
///   An empty `Vec` produces a Profile with no imports (C-2 modify-only case).
/// * `mode` — Whether the IDs represent included or excluded controls.
///   Ignored when `control_ids` is empty.
/// * `param_overrides` — `(param_id, value)` pairs from `--set-param` flags (WI-31).
///   Pass `&[]` to produce output identical to WI-30 (no `modify` section).
///
/// # Errors
///
/// Returns `ForgeError` only from metadata assembly or serialization; never from an
/// empty `control_ids` (that produces a Profile with `"imports": []`).
///
/// # Guardrails
///
/// * Does NOT read or parse the source Catalog file.
#[tracing::instrument(skip_all)]
pub fn build_profile(
    catalog_path: &str,
    control_ids: Vec<String>,
    mode: SelectionMode,
    param_overrides: &[(String, String)],
    timestamp_override: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<OscalProfile, ForgeError> {
    let doc_meta = DocumentMetadata {
        title: "Policy Baseline Profile".to_string(),
        version: "1.0.0".to_string(),
        ..Default::default()
    };
    let meta_opts = timestamp_override.map(|ts| crate::oscal::metadata::MetadataOptions {
        timestamp_override: Some(ts),
        ..Default::default()
    });
    let metadata = assemble_metadata(&doc_meta, meta_opts)?;

    // Compute deterministic UUID v5 from normalized inputs so that
    // different input orderings of the same logical profile produce
    // the same UUID.
    let mode_str = match mode {
        SelectionMode::Include => "include",
        SelectionMode::Exclude => "exclude",
    };
    let sanitized_href = crate::io::sanitize_artifact_path(std::path::Path::new(catalog_path));
    let mut sorted_ids = control_ids.clone();
    sorted_ids.sort_unstable();
    let mut sorted_params: Vec<(&str, &str)> =
        param_overrides.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    sorted_params.sort_unstable();
    let mut seed_parts: Vec<&str> = vec![&sanitized_href, mode_str];
    seed_parts.extend(sorted_ids.iter().map(String::as_str));
    for (param_id, value) in &sorted_params {
        seed_parts.push(param_id);
        seed_parts.push(value);
    }
    let seed = seed_parts.join("|");
    let uuid = Uuid::new_v5(&crate::uuid::PROFILE_NAMESPACE, seed.as_bytes());

    let imports = if control_ids.is_empty() {
        vec![]
    } else {
        let selection = ControlSelection { with_ids: control_ids };
        let import = match mode {
            SelectionMode::Include => ProfileImport {
                href: sanitized_href.clone(),
                include_all: None,
                include_controls: Some(vec![selection]),
                exclude_controls: None,
            },
            SelectionMode::Exclude => ProfileImport {
                href: sanitized_href,
                // OSCAL v1.2.0 Profile schema requires `include-all` when using
                // `exclude-controls` alone — an import must include all controls
                // first, then exclude the specified subset.
                include_all: Some(IncludeAll {}),
                include_controls: None,
                exclude_controls: Some(vec![selection]),
            },
        };
        vec![import]
    };

    let modify = build_modify_section(param_overrides);

    Ok(OscalProfile { uuid, metadata, imports, modify })
}

// ---------------------------------------------------------------------------
// parse_control_ids
// ---------------------------------------------------------------------------

/// Parse a comma-separated control ID string into a trimmed, deduplicated Vec.
///
/// # Arguments
///
/// * `raw` — Comma-separated string from `--include` or `--exclude` CLI flag.
///
/// # Returns
///
/// A `Vec<String>` with whitespace trimmed, empty strings removed, and
/// duplicates eliminated (first occurrence preserved, order maintained).
///
/// # Errors
///
/// * `ForgeError::InvalidArgument` — if the resulting Vec is empty.
#[tracing::instrument(skip_all)]
pub fn parse_control_ids(raw: &str) -> Result<Vec<String>, ForgeError> {
    let mut seen = std::collections::HashSet::new();
    let ids: Vec<String> = raw
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .filter(|s| seen.insert(s.clone()))
        .collect();

    if ids.is_empty() {
        return Err(ForgeError::InvalidArgument(
            "No valid control IDs provided — supply at least one non-empty ID".to_string(),
        ));
    }
    Ok(ids)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ── T005/T021/T022: build_modify_section unit tests (WI-31) ─────────────

    #[test]
    fn build_modify_section_empty_returns_none() {
        assert!(build_modify_section(&[]).is_none());
    }

    #[test]
    fn build_modify_section_single_pair() {
        let pairs = vec![("POL-AC-001_prm".to_string(), "60 days".to_string())];
        let modify = build_modify_section(&pairs).unwrap();
        assert_eq!(modify.set_parameters.len(), 1);
        assert_eq!(modify.set_parameters[0].param_id, "POL-AC-001_prm");
        assert_eq!(modify.set_parameters[0].values, vec!["60 days"]);
    }

    #[test]
    fn build_modify_section_value_with_spaces_preserved() {
        let pairs = vec![("prm".to_string(), "at least 60 days".to_string())];
        let modify = build_modify_section(&pairs).unwrap();
        assert_eq!(modify.set_parameters[0].values, vec!["at least 60 days"]);
    }

    #[test]
    fn build_modify_section_empty_string_value() {
        let pairs = vec![("prm".to_string(), String::new())];
        let modify = build_modify_section(&pairs).unwrap();
        assert_eq!(modify.set_parameters[0].values, vec![""]);
    }

    #[test]
    fn build_modify_section_two_distinct_params_alphabetical() {
        let pairs = vec![
            ("zzz_prm".to_string(), "val1".to_string()),
            ("aaa_prm".to_string(), "val2".to_string()),
        ];
        let modify = build_modify_section(&pairs).unwrap();
        assert_eq!(modify.set_parameters.len(), 2);
        // BTreeMap guarantees alphabetical order
        assert_eq!(modify.set_parameters[0].param_id, "aaa_prm");
        assert_eq!(modify.set_parameters[1].param_id, "zzz_prm");
    }

    #[test]
    fn build_modify_section_ten_params_alphabetical() {
        let pairs: Vec<(String, String)> =
            (0..10).map(|i| (format!("prm_{i:02}"), format!("val_{i}"))).collect();
        let modify = build_modify_section(&pairs).unwrap();
        assert_eq!(modify.set_parameters.len(), 10);
        // Verify sorted
        let ids: Vec<&str> = modify.set_parameters.iter().map(|p| p.param_id.as_str()).collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        assert_eq!(ids, sorted);
    }

    #[test]
    fn build_modify_section_non_alphabetical_input_sorted() {
        let pairs = vec![
            ("c_prm".to_string(), "v3".to_string()),
            ("a_prm".to_string(), "v1".to_string()),
            ("b_prm".to_string(), "v2".to_string()),
        ];
        let modify = build_modify_section(&pairs).unwrap();
        assert_eq!(modify.set_parameters[0].param_id, "a_prm");
        assert_eq!(modify.set_parameters[1].param_id, "b_prm");
        assert_eq!(modify.set_parameters[2].param_id, "c_prm");
    }

    #[test]
    fn build_modify_section_duplicate_param_id_aggregated() {
        let pairs = vec![
            ("prm1".to_string(), "60 days".to_string()),
            ("prm1".to_string(), "quarterly".to_string()),
        ];
        let modify = build_modify_section(&pairs).unwrap();
        assert_eq!(modify.set_parameters.len(), 1);
        assert_eq!(modify.set_parameters[0].param_id, "prm1");
        assert_eq!(modify.set_parameters[0].values, vec!["60 days", "quarterly"]);
    }

    #[test]
    fn build_modify_section_serializes_correct_json_keys() {
        let pairs = vec![("my_prm".to_string(), "val".to_string())];
        let modify = build_modify_section(&pairs).unwrap();
        let json = serde_json::to_value(&modify).unwrap();
        assert!(json.get("set-parameters").is_some(), "must use 'set-parameters' key");
        let entry = &json["set-parameters"][0];
        assert!(entry.get("param-id").is_some(), "must use 'param-id' key");
        assert!(entry.get("values").is_some(), "must have 'values' key");
    }

    // ── T003: Type serialization tests ──────────────────────────────────────

    #[test]
    fn profile_root_serializes_with_profile_key() {
        let profile = build_profile(
            "/tmp/catalog.json",
            vec!["AC-1".to_string()],
            SelectionMode::Include,
            &[],
            None,
        )
        .unwrap();
        let root = ProfileRoot { profile };
        let json = serde_json::to_value(&root).unwrap();
        assert!(json.get("profile").is_some(), "must have 'profile' root key");
        assert!(json.get("catalog").is_none());
    }

    #[test]
    fn profile_import_include_produces_include_controls_key() {
        let profile = build_profile(
            "/tmp/cat.json",
            vec!["POL-1".to_string()],
            SelectionMode::Include,
            &[],
            None,
        )
        .unwrap();
        let import = &profile.imports[0];
        let json = serde_json::to_value(import).unwrap();
        assert!(json.get("include-controls").is_some());
        assert!(json.get("exclude-controls").is_none());
        assert_eq!(json["href"], "cat.json");
    }

    #[test]
    fn profile_import_exclude_produces_exclude_controls_key() {
        let profile = build_profile(
            "/tmp/cat.json",
            vec!["POL-1".to_string()],
            SelectionMode::Exclude,
            &[],
            None,
        )
        .unwrap();
        let import = &profile.imports[0];
        let json = serde_json::to_value(import).unwrap();
        assert!(json.get("exclude-controls").is_some());
        assert!(json.get("include-controls").is_none());
    }

    #[test]
    fn control_selection_serializes_with_ids_key() {
        let sel = ControlSelection { with_ids: vec!["AC-1".to_string(), "AC-2".to_string()] };
        let json = serde_json::to_value(&sel).unwrap();
        assert!(json.get("with-ids").is_some());
        assert_eq!(json["with-ids"][0], "AC-1");
    }

    // ── T004: parse_control_ids tests ───────────────────────────────────────

    #[test]
    fn parse_trims_whitespace() {
        let ids = parse_control_ids("  AC-1  ,  AC-2  ").unwrap();
        assert_eq!(ids, vec!["AC-1", "AC-2"]);
    }

    #[test]
    fn parse_deduplicates_order_preserving() {
        let ids = parse_control_ids("AC-1,AC-2,AC-1,AC-3").unwrap();
        assert_eq!(ids, vec!["AC-1", "AC-2", "AC-3"]);
    }

    #[test]
    fn parse_removes_empty_tokens() {
        let ids = parse_control_ids("AC-1,,AC-2").unwrap();
        assert_eq!(ids, vec!["AC-1", "AC-2"]);
    }

    #[test]
    fn parse_errors_on_empty_string() {
        let err = parse_control_ids("").unwrap_err();
        assert!(matches!(err, ForgeError::InvalidArgument(_)));
    }

    #[test]
    fn parse_single_id_no_comma() {
        let ids = parse_control_ids("POL-AC-001").unwrap();
        assert_eq!(ids, vec!["POL-AC-001"]);
    }

    // ── T007: build_profile unit tests ──────────────────────────────────────

    #[test]
    fn build_profile_href_uses_filename_only() {
        let profile = build_profile(
            "/abs/path/catalog.json",
            vec!["AC-1".to_string()],
            SelectionMode::Include,
            &[],
            None,
        )
        .unwrap();
        assert_eq!(profile.imports[0].href, "catalog.json");
        assert!(!profile.imports[0].href.contains('/'));
    }

    #[test]
    fn build_profile_include_sets_include_controls_none_exclude() {
        let profile = build_profile(
            "/tmp/c.json",
            vec!["AC-1".to_string()],
            SelectionMode::Include,
            &[],
            None,
        )
        .unwrap();
        assert!(profile.imports[0].include_controls.is_some());
        assert!(profile.imports[0].exclude_controls.is_none());
        let ids = &profile.imports[0].include_controls.as_ref().unwrap()[0].with_ids;
        assert_eq!(ids, &["AC-1"]);
    }

    #[test]
    fn build_profile_metadata_title_and_oscal_version() {
        let profile = build_profile(
            "/tmp/c.json",
            vec!["AC-1".to_string()],
            SelectionMode::Include,
            &[],
            None,
        )
        .unwrap();
        assert_eq!(profile.metadata.title, "Policy Baseline Profile");
        assert_eq!(profile.metadata.oscal_version, "1.2.0");
    }

    #[test]
    fn build_profile_security_no_catalog_content_in_json() {
        let profile = build_profile(
            "/tmp/c.json",
            vec!["AC-1".to_string()],
            SelectionMode::Include,
            &[],
            None,
        )
        .unwrap();
        let root = ProfileRoot { profile };
        let json_str = serde_json::to_string(&root).unwrap();
        // Profile JSON must only reference the catalog by href, not embed content
        assert!(!json_str.contains("\"groups\""));
        assert!(!json_str.contains("\"controls\""));
        assert!(json_str.contains("c.json"));
    }

    #[test]
    fn build_profile_security_href_uses_filename_only() {
        let path = "/absolute/path/to catalog.json";
        let profile =
            build_profile(path, vec!["AC-1".to_string()], SelectionMode::Include, &[], None)
                .unwrap();
        // sanitize_artifact_path extracts filename, handling spaces
        assert_eq!(profile.imports[0].href, "to catalog.json");
        assert!(!profile.imports[0].href.contains('/'));
    }

    #[test]
    fn profile_import_href_uses_filename_only() {
        let profile = build_profile(
            "/absolute/path/to/catalog.json",
            vec!["AC-1".to_string()],
            SelectionMode::Include,
            &[],
            None,
        )
        .unwrap();
        assert_eq!(profile.imports[0].href, "catalog.json");
        assert!(!profile.imports[0].href.contains('/'));
    }

    // ── T013: build_profile exclude path ────────────────────────────────────

    #[test]
    fn build_profile_exclude_sets_exclude_controls_none_include() {
        let profile = build_profile(
            "/tmp/c.json",
            vec!["POL-AC-003".to_string()],
            SelectionMode::Exclude,
            &[],
            None,
        )
        .unwrap();
        assert!(profile.imports[0].exclude_controls.is_some());
        assert!(profile.imports[0].include_controls.is_none());
        let ids = &profile.imports[0].exclude_controls.as_ref().unwrap()[0].with_ids;
        assert_eq!(ids, &["POL-AC-003"]);
    }

    #[test]
    fn build_profile_exclude_json_omits_include_controls_key() {
        let profile = build_profile(
            "/tmp/c.json",
            vec!["X-1".to_string()],
            SelectionMode::Exclude,
            &[],
            None,
        )
        .unwrap();
        let json = serde_json::to_value(&profile.imports[0]).unwrap();
        assert!(json.get("exclude-controls").is_some());
        assert!(json.get("include-controls").is_none());
    }

    // ── Task 23: deterministic profile UUIDs ─────────────────────────────────

    #[test]
    fn profile_same_inputs_produce_same_uuid() {
        let p1 = build_profile(
            "catalog.json",
            vec!["AC-1".to_string()],
            SelectionMode::Include,
            &[],
            None,
        )
        .unwrap();
        let p2 = build_profile(
            "catalog.json",
            vec!["AC-1".to_string()],
            SelectionMode::Include,
            &[],
            None,
        )
        .unwrap();
        assert_eq!(p1.uuid, p2.uuid, "Same inputs should produce same UUID");
    }

    #[test]
    fn profile_different_inputs_produce_different_uuid() {
        let p1 = build_profile(
            "catalog.json",
            vec!["AC-1".to_string()],
            SelectionMode::Include,
            &[],
            None,
        )
        .unwrap();
        let p2 = build_profile(
            "catalog.json",
            vec!["AC-2".to_string()],
            SelectionMode::Include,
            &[],
            None,
        )
        .unwrap();
        assert_ne!(p1.uuid, p2.uuid, "Different control IDs should produce different UUID");
    }

    #[test]
    fn profile_uuid_is_v5() {
        let profile = build_profile(
            "catalog.json",
            vec!["AC-1".to_string()],
            SelectionMode::Include,
            &[],
            None,
        )
        .unwrap();
        assert_eq!(
            profile.uuid.get_version(),
            Some(uuid::Version::Sha1),
            "Profile UUID should be v5 (SHA1-based)"
        );
    }
}
