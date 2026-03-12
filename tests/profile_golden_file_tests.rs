//! Profile golden-file regression tests (WI-32, US2).
//!
//! Uses `insta` snapshot testing to lock down expected Profile JSON for
//! include-only and exclude-only scenarios. Dynamic fields (UUIDs,
//! `last-modified`) are normalized before comparison so snapshots are
//! stable across repeated runs.

mod common;

use forge::oscal::profile::{ProfileRoot, SelectionMode, build_profile};

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
