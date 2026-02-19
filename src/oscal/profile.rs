//! OSCAL Profile generation for WI-30.
//!
//! Provides [`build_profile`] and [`parse_control_ids`] plus the structs required
//! to produce a valid OSCAL v1.2.0 Profile JSON artifact.

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
    pub profile: OscalProfile,
}

// ---------------------------------------------------------------------------
// OSCAL Profile model
// ---------------------------------------------------------------------------

/// OSCAL Profile model. Contains metadata and one or more import entries.
///
/// WI-30 produces profiles with exactly one [`ProfileImport`].
#[derive(Debug, Serialize)]
pub struct OscalProfile {
    /// UUID v4 — unique per generation.
    pub uuid: Uuid,

    /// OSCAL metadata (title, last-modified, version, oscal-version).
    pub metadata: OscalMetadata,

    /// Import entries: which catalog(s) to draw controls from.
    pub imports: Vec<ProfileImport>,
}

// ---------------------------------------------------------------------------
// ProfileImport
// ---------------------------------------------------------------------------

/// A single entry in the Profile's `imports[]` array.
///
/// Exactly one of `include_controls` or `exclude_controls` is `Some`.
#[derive(Debug, Serialize)]
pub struct ProfileImport {
    /// URI reference to the source Catalog (stored as-is from `--catalog`).
    pub href: String,

    /// Controls to include (from `--include`).
    #[serde(skip_serializing_if = "Option::is_none", rename = "include-controls")]
    pub include_controls: Option<Vec<ControlSelection>>,

    /// Controls to exclude (from `--exclude`).
    #[serde(skip_serializing_if = "Option::is_none", rename = "exclude-controls")]
    pub exclude_controls: Option<Vec<ControlSelection>>,
}

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
    Include,
    Exclude,
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
/// * `mode` — Whether the IDs represent included or excluded controls.
///
/// # Errors
///
/// * `ForgeError::InvalidArgument` — if `control_ids` is empty.
///
/// # Guardrails
///
/// * Does NOT read or parse the source Catalog file.
/// * Does NOT generate a `modify` section (WI-31 scope).
#[tracing::instrument(skip_all)]
pub fn build_profile(
    catalog_path: &str,
    control_ids: Vec<String>,
    mode: SelectionMode,
) -> Result<OscalProfile, ForgeError> {
    if control_ids.is_empty() {
        return Err(ForgeError::InvalidArgument("control_ids must not be empty".to_string()));
    }

    let doc_meta = DocumentMetadata {
        title: "Policy Baseline Profile".to_string(),
        version: "1.0.0".to_string(),
        ..Default::default()
    };
    let metadata = assemble_metadata(&doc_meta, None)?;

    let selection = ControlSelection { with_ids: control_ids };
    let import = match mode {
        SelectionMode::Include => ProfileImport {
            href: catalog_path.to_string(),
            include_controls: Some(vec![selection]),
            exclude_controls: None,
        },
        SelectionMode::Exclude => ProfileImport {
            href: catalog_path.to_string(),
            include_controls: None,
            exclude_controls: Some(vec![selection]),
        },
    };

    Ok(OscalProfile { uuid: Uuid::new_v4(), metadata, imports: vec![import] })
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

    // ── T003: Type serialization tests ──────────────────────────────────────

    #[test]
    fn profile_root_serializes_with_profile_key() {
        let profile =
            build_profile("/tmp/catalog.json", vec!["AC-1".to_string()], SelectionMode::Include)
                .unwrap();
        let root = ProfileRoot { profile };
        let json = serde_json::to_value(&root).unwrap();
        assert!(json.get("profile").is_some(), "must have 'profile' root key");
        assert!(json.get("catalog").is_none());
    }

    #[test]
    fn profile_import_include_produces_include_controls_key() {
        let profile =
            build_profile("/tmp/cat.json", vec!["POL-1".to_string()], SelectionMode::Include)
                .unwrap();
        let import = &profile.imports[0];
        let json = serde_json::to_value(import).unwrap();
        assert!(json.get("include-controls").is_some());
        assert!(json.get("exclude-controls").is_none());
        assert_eq!(json["href"], "/tmp/cat.json");
    }

    #[test]
    fn profile_import_exclude_produces_exclude_controls_key() {
        let profile =
            build_profile("/tmp/cat.json", vec!["POL-1".to_string()], SelectionMode::Exclude)
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
    fn build_profile_href_matches_catalog_path() {
        let profile = build_profile(
            "/abs/path/catalog.json",
            vec!["AC-1".to_string()],
            SelectionMode::Include,
        )
        .unwrap();
        assert_eq!(profile.imports[0].href, "/abs/path/catalog.json");
    }

    #[test]
    fn build_profile_include_sets_include_controls_none_exclude() {
        let profile =
            build_profile("/tmp/c.json", vec!["AC-1".to_string()], SelectionMode::Include).unwrap();
        assert!(profile.imports[0].include_controls.is_some());
        assert!(profile.imports[0].exclude_controls.is_none());
        let ids = &profile.imports[0].include_controls.as_ref().unwrap()[0].with_ids;
        assert_eq!(ids, &["AC-1"]);
    }

    #[test]
    fn build_profile_metadata_title_and_oscal_version() {
        let profile =
            build_profile("/tmp/c.json", vec!["AC-1".to_string()], SelectionMode::Include).unwrap();
        assert_eq!(profile.metadata.title, "Policy Baseline Profile");
        assert_eq!(profile.metadata.oscal_version, "1.2.0");
    }

    #[test]
    fn build_profile_security_no_catalog_content_in_json() {
        let profile =
            build_profile("/tmp/c.json", vec!["AC-1".to_string()], SelectionMode::Include).unwrap();
        let root = ProfileRoot { profile };
        let json_str = serde_json::to_string(&root).unwrap();
        // Profile JSON must only reference the catalog by href, not embed content
        assert!(!json_str.contains("\"groups\""));
        assert!(!json_str.contains("\"controls\""));
        assert!(json_str.contains("/tmp/c.json"));
    }

    #[test]
    fn build_profile_security_href_stored_as_is() {
        let path = "/absolute/path/to catalog.json";
        let profile =
            build_profile(path, vec!["AC-1".to_string()], SelectionMode::Include).unwrap();
        assert_eq!(profile.imports[0].href, path);
    }

    // ── T013: build_profile exclude path ────────────────────────────────────

    #[test]
    fn build_profile_exclude_sets_exclude_controls_none_include() {
        let profile =
            build_profile("/tmp/c.json", vec!["POL-AC-003".to_string()], SelectionMode::Exclude)
                .unwrap();
        assert!(profile.imports[0].exclude_controls.is_some());
        assert!(profile.imports[0].include_controls.is_none());
        let ids = &profile.imports[0].exclude_controls.as_ref().unwrap()[0].with_ids;
        assert_eq!(ids, &["POL-AC-003"]);
    }

    #[test]
    fn build_profile_exclude_json_omits_include_controls_key() {
        let profile =
            build_profile("/tmp/c.json", vec!["X-1".to_string()], SelectionMode::Exclude).unwrap();
        let json = serde_json::to_value(&profile.imports[0]).unwrap();
        assert!(json.get("exclude-controls").is_some());
        assert!(json.get("include-controls").is_none());
    }
}
