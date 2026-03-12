//! Profile golden-file regression tests (WI-32, US2).
//!
//! Uses `insta` snapshot testing to lock down expected Profile JSON for
//! include-only and exclude-only scenarios. Dynamic fields (UUIDs,
//! `last-modified`) are normalized before comparison so snapshots are
//! stable across repeated runs.

mod common;

use std::io::Write as _;

use tempfile::NamedTempFile;

use forge::oscal::profile::{ProfileRoot, SelectionMode, build_profile};

/// Minimal OSCAL catalog JSON with 10 controls (AC-1 through AC-10).
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
// US2: Golden-file snapshot tests (T005)
// ---------------------------------------------------------------------------

/// Include-only golden file: AC-1, AC-2, AC-3 selected.
///
/// Snapshot is normalized before comparison to eliminate dynamic fields
/// (UUIDs and `last-modified` timestamps).
///
/// Note: `build_profile` does NOT read the catalog file (see guardrails),
/// so we pass a fixed path. `sanitize_artifact_path` extracts the filename,
/// producing a deterministic `href` in the output.
#[test]
fn golden_include_only() {
    let profile = build_profile(
        "/fixed/path/catalog.json",
        vec!["AC-1".into(), "AC-2".into(), "AC-3".into()],
        SelectionMode::Include,
        &[],
    )
    .expect("build_profile should succeed");

    let root = ProfileRoot { profile };
    let value = serde_json::to_value(&root).expect("Serialization must succeed");
    let normalized = common::normalize_for_snapshot(&value);

    insta::assert_json_snapshot!("golden_include_only", &normalized);
}

/// Exclude-only golden file: AC-9 and AC-10 excluded.
///
/// Snapshot is normalized before comparison to eliminate dynamic fields
/// (UUIDs and `last-modified` timestamps).
///
/// Note: `build_profile` does NOT read the catalog file (see guardrails),
/// so we pass a fixed path. `sanitize_artifact_path` extracts the filename,
/// producing a deterministic `href` in the output.
#[test]
fn golden_exclude_only() {
    let profile = build_profile(
        "/fixed/path/catalog.json",
        vec!["AC-9".into(), "AC-10".into()],
        SelectionMode::Exclude,
        &[],
    )
    .expect("build_profile should succeed");

    let root = ProfileRoot { profile };
    let value = serde_json::to_value(&root).expect("Serialization must succeed");
    let normalized = common::normalize_for_snapshot(&value);

    insta::assert_json_snapshot!("golden_exclude_only", &normalized);
}

// TODO(WI-31): remove #[ignore] when --set-param is implemented
/// Include-only with parameter overrides golden file.
#[test]
#[ignore = "WI-31 (--set-param) not yet implemented"]
fn golden_include_with_params() {
    todo!("Enable when WI-31 (--set-param) is implemented")
}
