//! Profile schema validation and edge case tests (WI-32, US1 + US3).
//!
//! Tests confirm that `forge profile` output (include-only and exclude-only)
//! conforms to the OSCAL v1.2.3 Profile JSON schema using the existing
//! `validate_artifact()` infrastructure from WI-19.

use std::io::Write as _;

use tempfile::NamedTempFile;

use forge::oscal::profile::{ProfileRoot, SelectionMode, build_profile, parse_control_ids};
use forge::validate::{OscalModelType, validate_artifact};

// ---------------------------------------------------------------------------
// Shared fixture
// ---------------------------------------------------------------------------

/// Minimal OSCAL catalog JSON with 10 controls (AC-1 through AC-10).
///
/// The catalog content is NOT parsed by `build_profile()` — only the path is
/// used as the `imports[0].href`. The file must exist on disk so the CLI-level
/// `execute()` path can check existence.
const CATALOG_JSON: &str = r#"{
  "catalog": {
    "uuid": "a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d",
    "metadata": {
      "title": "Test Catalog",
      "last-modified": "2026-01-01T00:00:00Z",
      "version": "1.0",
      "oscal-version": "1.2.0"
    },
    "controls": [
      {"id": "AC-1",  "title": "Access Control Policy"},
      {"id": "AC-2",  "title": "Account Management"},
      {"id": "AC-3",  "title": "Access Enforcement"},
      {"id": "AC-4",  "title": "Information Flow Enforcement"},
      {"id": "AC-5",  "title": "Separation of Duties"},
      {"id": "AC-6",  "title": "Least Privilege"},
      {"id": "AC-7",  "title": "Unsuccessful Logon Attempts"},
      {"id": "AC-8",  "title": "System Use Notification"},
      {"id": "AC-9",  "title": "Previous Logon Notification"},
      {"id": "AC-10", "title": "Concurrent Session Control"}
    ]
  }
}"#;

/// Write `CATALOG_JSON` to a temporary file and return the file handle.
///
/// Keep the returned `NamedTempFile` alive for the duration of the test to
/// prevent premature deletion.
fn make_catalog_file() -> NamedTempFile {
    let mut f = NamedTempFile::with_suffix(".json").expect("Failed to create temp file");
    f.write_all(CATALOG_JSON.as_bytes()).expect("Failed to write catalog JSON");
    f.flush().expect("Failed to flush temp file");
    f
}

// ---------------------------------------------------------------------------
// US1: Schema validation tests (T004)
// ---------------------------------------------------------------------------

/// Include-only Profile passes OSCAL v1.2.3 schema validation.
#[test]
fn schema_include_only() {
    let catalog = make_catalog_file();
    let catalog_path = catalog.path().to_string_lossy().to_string();

    let profile = build_profile(
        &catalog_path,
        vec!["AC-1".into(), "AC-2".into()],
        SelectionMode::Include,
        &[],
        None,
    )
    .expect("build_profile should succeed for include mode");

    let root = ProfileRoot { profile };
    let value = serde_json::to_value(&root).expect("Serialization must succeed");

    let result = validate_artifact(&value, OscalModelType::Profile)
        .expect("validate_artifact should not return a validation framework error");

    assert!(
        result.is_valid,
        "Include-only Profile must be schema-valid. Errors: {:?}",
        result.errors
    );
    assert!(result.errors.is_empty(), "Expected zero schema errors, got: {:?}", result.errors);
}

/// Exclude-only Profile passes OSCAL v1.2.3 schema validation.
#[test]
fn schema_exclude_only() {
    let catalog = make_catalog_file();
    let catalog_path = catalog.path().to_string_lossy().to_string();

    let profile =
        build_profile(&catalog_path, vec!["AC-10".into()], SelectionMode::Exclude, &[], None)
            .expect("build_profile should succeed for exclude mode");

    let root = ProfileRoot { profile };
    let value = serde_json::to_value(&root).expect("Serialization must succeed");

    let result = validate_artifact(&value, OscalModelType::Profile)
        .expect("validate_artifact should not return a validation framework error");

    assert!(
        result.is_valid,
        "Exclude-only Profile must be schema-valid. Errors: {:?}",
        result.errors
    );
    assert!(result.errors.is_empty(), "Expected zero schema errors, got: {:?}", result.errors);
}

// Note: PRD S-4 (schema error messages include JSON path) is satisfied by WI-19's
// `jsonschema` crate — `ValidationError` carries `instance_path`. This is already
// verified by existing unit tests in `src/validate/mod.rs` (`validate_errors_have_instance_path`).

// TODO(WI-31): remove #[ignore] when --set-param is implemented
/// Profile with parameter overrides passes OSCAL v1.2.3 schema validation.
#[test]
#[ignore = "WI-31 (--set-param) not yet implemented"]
fn schema_with_set_param() {
    todo!("Enable when WI-31 (--set-param) is implemented")
}

// ---------------------------------------------------------------------------
// US3: Edge case tests (T007)
// ---------------------------------------------------------------------------

/// Empty include list returns a descriptive error — an empty Profile is not produced.
#[test]
fn edge_empty_include_list() {
    let catalog = make_catalog_file();

    // WI-31 moved the empty-list guard to parse_control_ids (called by execute).
    // build_profile with an empty vec now produces a Profile with no imports (C-2 case).
    // Test via the CLI execute path, which calls parse_control_ids("") and returns Err.
    let result = forge::cli::profile::execute(
        catalog.path(),
        Some(""),
        None,
        &forge::cli::OutputFormat::Json,
        None,
        &[],
        None,
    );

    assert!(result.is_err(), "Empty include list must return an error");
    let err = result.unwrap_err();
    assert!(
        matches!(err, forge::ForgeError::InvalidArgument(_)),
        "Expected ForgeError::InvalidArgument for empty control list, got: {err:?}"
    );
}

/// All-controls include (10 IDs) produces a valid Profile that passes schema validation.
#[test]
fn edge_all_controls_include() {
    let catalog = make_catalog_file();
    let catalog_path = catalog.path().to_string_lossy().to_string();

    let all_ids: Vec<String> = (1..=10).map(|i| format!("AC-{i}")).collect();

    let profile = build_profile(&catalog_path, all_ids, SelectionMode::Include, &[], None)
        .expect("build_profile should succeed with all 10 control IDs");

    let root = ProfileRoot { profile };
    let value = serde_json::to_value(&root).expect("Serialization must succeed");

    let result = validate_artifact(&value, OscalModelType::Profile)
        .expect("validate_artifact should not error");

    assert!(
        result.is_valid,
        "All-controls Profile must be schema-valid. Errors: {:?}",
        result.errors
    );
}

/// Duplicate control IDs are deduplicated — the output contains exactly 2 unique IDs.
#[test]
fn edge_duplicate_control_ids() {
    let catalog = make_catalog_file();
    let catalog_path = catalog.path().to_string_lossy().to_string();

    // AC-1 appears twice; expect deduplication → ["AC-1", "AC-2"]
    // Note: parse_control_ids() deduplicates; build_profile() takes the already-deduped list.
    // Here we pass the pre-deduped list (as the CLI would after parse_control_ids).
    let ids = parse_control_ids("AC-1,AC-1,AC-2").expect("parse_control_ids should succeed");
    assert_eq!(ids.len(), 2, "parse_control_ids must deduplicate AC-1,AC-1,AC-2 → 2 entries");

    let profile = build_profile(&catalog_path, ids, SelectionMode::Include, &[], None)
        .expect("build_profile should succeed");

    let with_ids =
        &profile.imports[0].include_controls.as_ref().expect("include_controls must be Some")[0]
            .with_ids;

    assert_eq!(
        with_ids.len(),
        2,
        "with-ids must contain exactly 2 entries after deduplication, got: {with_ids:?}"
    );
    assert!(with_ids.contains(&"AC-1".to_string()), "AC-1 must be in with-ids");
    assert!(with_ids.contains(&"AC-2".to_string()), "AC-2 must be in with-ids");
}

/// Providing both --include and --exclude returns a mutually-exclusive error.
#[test]
fn edge_both_flags_returns_error() {
    let catalog = make_catalog_file();

    let result = forge::cli::profile::execute(
        catalog.path(),
        Some("AC-1"),
        Some("AC-10"),
        &forge::cli::OutputFormat::Json,
        None,
        &[],
        None,
    );

    assert!(result.is_err(), "Providing both --include and --exclude must return an error");
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("mutually exclusive")
            || msg.contains("--include")
            || msg.contains("--exclude"),
        "Error must reference mutual exclusivity or the conflicting flags, got: {msg}"
    );
}

/// Non-existent catalog path returns a descriptive error.
#[test]
fn edge_invalid_catalog_path() {
    let nonexistent = std::path::Path::new("/tmp/nonexistent-catalog-99999.json");

    let result = forge::cli::profile::execute(
        nonexistent,
        Some("AC-1"),
        None,
        &forge::cli::OutputFormat::Json,
        None,
        &[],
        None,
    );

    assert!(result.is_err(), "Non-existent catalog path must return an error");
    let err = result.unwrap_err();
    assert!(
        matches!(err, forge::ForgeError::FileNotFound { .. }),
        "Expected ForgeError::FileNotFound for non-existent catalog, got: {err:?}"
    );
}

/// A control ID not present in the catalog does not cause an error.
///
/// WI-30's `build_profile()` uses the catalog path as an OSCAL Profile `href`
/// reference only — it does not parse or validate catalog content. Control IDs
/// are stored as-is in the `with-ids` list. Validation against the actual
/// catalog control set is out of scope for WI-30 and WI-32.
#[test]
fn edge_nonexistent_control_id() {
    let catalog = make_catalog_file();
    let catalog_path = catalog.path().to_string_lossy().to_string();

    // "FAKE-999" does not exist in the test catalog, but build_profile does not
    // parse the catalog content — this call succeeds.
    let result =
        build_profile(&catalog_path, vec!["FAKE-999".into()], SelectionMode::Include, &[], None);

    assert!(
        result.is_ok(),
        "build_profile must succeed even with a nonexistent control ID (catalog not parsed): {result:?}"
    );

    let profile = result.unwrap();
    let with_ids =
        &profile.imports[0].include_controls.as_ref().expect("include_controls must be Some")[0]
            .with_ids;

    assert_eq!(with_ids, &["FAKE-999"], "The unknown ID must be stored as-is in with-ids");
}

// TODO(WI-31): remove #[ignore] when --set-param is implemented
/// Conflicting set-param overrides produce a well-defined outcome.
#[test]
#[ignore = "WI-31 (--set-param) not yet implemented"]
fn edge_conflicting_set_param() {
    todo!("Enable when WI-31 (--set-param) is implemented")
}

// ---------------------------------------------------------------------------
// US3: End-to-end acceptance criterion test (T008)
// ---------------------------------------------------------------------------

/// Verifies parent PRD AC-12: given a policy Catalog with multiple controls,
/// forge profile with include flags generates a valid OSCAL Profile.
#[test]
fn e2e_ac12_profile_generation() {
    let catalog = make_catalog_file();
    let catalog_path = catalog.path().to_string_lossy().to_string();

    let profile = build_profile(
        &catalog_path,
        vec!["AC-1".into(), "AC-2".into(), "AC-3".into(), "AC-4".into(), "AC-5".into()],
        SelectionMode::Include,
        &[],
        None,
    )
    .expect("build_profile should succeed for 5-control include list");

    let root = ProfileRoot { profile };
    let value = serde_json::to_value(&root).expect("Serialization must succeed");

    // Schema validation
    let result = validate_artifact(&value, OscalModelType::Profile)
        .expect("validate_artifact should not return a framework error");

    assert!(
        result.is_valid,
        "AC-12 e2e: Profile with 5 included controls must be schema-valid. Errors: {:?}",
        result.errors
    );
    assert!(
        result.errors.is_empty(),
        "AC-12 e2e: Expected zero schema errors, got: {:?}",
        result.errors
    );

    // Structural assertions
    assert!(value.get("profile").is_some(), "AC-12 e2e: JSON must have 'profile' root key");

    let imports = value["profile"]["imports"].as_array().expect("imports must be an array");
    assert!(!imports.is_empty(), "AC-12 e2e: imports must be non-empty");

    let include_controls = imports[0]
        .get("include-controls")
        .and_then(|ic| ic.as_array())
        .expect("include-controls must be present and be an array");

    let with_ids = include_controls[0]
        .get("with-ids")
        .and_then(|ids| ids.as_array())
        .expect("with-ids must be present and be an array");

    assert_eq!(
        with_ids.len(),
        5,
        "AC-12 e2e: with-ids must contain exactly 5 entries, got: {with_ids:?}"
    );
}
