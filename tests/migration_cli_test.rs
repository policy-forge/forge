use std::process::Command;

use serde_json::Value;

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
    assert!(String::from_utf8_lossy(&output.stderr).contains("Migration analysis error"));
}
