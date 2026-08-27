//! Profile golden-file regression tests (WI-32, US2).
//!
//! Uses `insta` snapshot testing to lock down expected Profile JSON for
//! include-only and exclude-only scenarios. Dynamic fields (UUIDs,
//! `last-modified`) are normalized before comparison so snapshots are
//! stable across repeated runs.

mod common;

use chrono::{DateTime, Utc};
use forge::oscal::profile::{ProfileRoot, SelectionMode, build_profile};

fn fixed_timestamp() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
        .expect("fixed RFC 3339 timestamp")
        .with_timezone(&Utc)
}

fn normalized_profile_value(
    control_ids: Vec<String>,
    selection_mode: SelectionMode,
) -> serde_json::Value {
    let profile = build_profile(
        "/fixed/path/catalog.json",
        control_ids,
        selection_mode,
        &[],
        Some(fixed_timestamp()),
    )
    .expect("build_profile should succeed");

    let root = ProfileRoot { profile };
    let value = serde_json::to_value(&root).expect("serialization must succeed");
    common::normalize_for_snapshot(&value)
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
/// so the helper passes a fixed path. `sanitize_artifact_path` extracts the filename,
/// producing a deterministic `href` in the output.
#[test]
fn golden_include_only() {
    let normalized = normalized_profile_value(
        vec!["AC-1".into(), "AC-2".into(), "AC-3".into()],
        SelectionMode::Include,
    );

    insta::assert_json_snapshot!("golden_include_only", &normalized);
}

/// Exclude-only golden file: AC-9 and AC-10 excluded.
///
/// Snapshot is normalized before comparison to eliminate dynamic fields
/// (UUIDs and `last-modified` timestamps).
///
/// Note: `build_profile` does NOT read the catalog file (see guardrails),
/// so the helper passes a fixed path. `sanitize_artifact_path` extracts the filename,
/// producing a deterministic `href` in the output.
#[test]
fn golden_exclude_only() {
    let normalized =
        normalized_profile_value(vec!["AC-9".into(), "AC-10".into()], SelectionMode::Exclude);

    insta::assert_json_snapshot!("golden_exclude_only", &normalized);
}

/// C-2: an empty selection preserves the documented empty-imports shape.
///
/// This is intentionally not schema validated: the current OSCAL profile schema requires at
/// least one import, while `build_profile` documents and emits this empty selection.
#[test]
fn empty_selection_serializes_empty_imports() {
    let normalized = normalized_profile_value(vec![], SelectionMode::Include);
    assert_eq!(normalized["profile"]["imports"], serde_json::json!([]));
}

// TODO(WI-31): remove #[ignore] when --set-param is implemented
/// Include-only with parameter overrides golden file.
#[test]
#[ignore = "WI-31 (--set-param) not yet implemented"]
fn golden_include_with_params() {
    todo!("Enable when WI-31 (--set-param) is implemented")
}
