use std::process::Command;

use serde_json::Value;
use serde_json::json;

fn forge() -> Command {
    Command::new(env!("CARGO_BIN_EXE_forge"))
}

#[test]
fn identical_policy_emits_complete_json_and_exits_zero() {
    let fixture = "tests/fixtures/sample_policy.md";
    let output = forge().args(["migrate", fixture, fixture, "--format", "json"]).output().unwrap();

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(output.stderr.is_empty());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["schema_version"], "forge.migration-report/1");
    assert_eq!(report["forge_version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(report["analysis_complete"], true);
    assert_eq!(report["summary"]["total_old"], report["summary"]["total_new"]);
    assert_eq!(report["summary"]["old_requirements"]["unchanged"], report["summary"]["total_old"]);
    assert_eq!(report["summary"]["new_requirements"]["unchanged"], report["summary"]["total_new"]);

    let repeated =
        forge().args(["migrate", fixture, fixture, "--format", "json"]).output().unwrap();
    assert_eq!(output.stdout, repeated.stdout, "identical inputs must be byte-deterministic");
}

#[test]
fn substantive_change_report_is_written_before_exit_one() {
    let old = "tests/fixtures/edge-cases/ec06-substantive-change/input-original.md";
    let new = "tests/fixtures/edge-cases/ec06-substantive-change/input-changed.md";
    let output = forge().args(["migrate", old, new, "--format", "json"]).output().unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["summary"]["substantive_change_candidates"], 1);
}

#[test]
fn analysis_failure_exits_two_without_partial_report() {
    let output = forge()
        .args(["migrate", "missing-old.md", "missing-new.md", "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Migration error"));
}

#[test]
fn reviewer_successor_declaration_overrides_a_candidate_and_preserves_approval() {
    let old = "tests/fixtures/edge-cases/ec06-substantive-change/input-original.md";
    let new = "tests/fixtures/edge-cases/ec06-substantive-change/input-changed.md";
    let initial = forge().args(["migrate", old, new, "--format", "json"]).output().unwrap();
    let initial_report: Value = serde_json::from_slice(&initial.stdout).unwrap();
    let candidate = initial_report["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["classification"] == "substantive_change_candidate")
        .unwrap();
    let old_id = candidate["old"][0]["stable_id"].as_str().unwrap();
    let new_id = candidate["new"][0]["stable_id"].as_str().unwrap();

    let directory = tempfile::tempdir().unwrap();
    let successor_path = directory.path().join("successors.json");
    std::fs::write(
        &successor_path,
        serde_json::to_vec_pretty(&json!({
            "schema_version": "forge.successor-map/1",
            "relationships": [{
                "relationship": "successor",
                "old_ids": [old_id],
                "new_ids": [new_id],
                "approved_by": "reviewer",
                "approved_at": "2026-08-25T12:00:00Z",
                "rationale": "Reviewed requirement continuity."
            }]
        }))
        .unwrap(),
    )
    .unwrap();
    let declared = forge()
        .args([
            "migrate",
            old,
            new,
            "--format",
            "json",
            "--successor-map",
            successor_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(declared.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&declared.stdout).unwrap();
    assert_eq!(report["summary"]["declared_successors"], 1);
    let entry = report["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["classification"] == "declared_successor")
        .unwrap();
    assert_eq!(entry["approval_status"], "declared");
    assert_eq!(entry["evidence"], json!(["reviewer_declaration"]));
    assert_eq!(entry["declaration"]["approved_by"], "reviewer");
    assert_eq!(entry["declaration"]["rationale"], "Reviewed requirement continuity.");

    let aliased = forge()
        .args([
            "migrate",
            old,
            new,
            "--successor-map",
            successor_path.to_str().unwrap(),
            "--output",
            successor_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(aliased.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&aliased.stderr).contains("successor map path"));
}
