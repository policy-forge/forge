//! PRD 057 framework change impact contract tests.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

const FRAMEWORK_UUID: &str = "77777777-7777-4777-8777-777777777777";

fn write_json(path: &Path, value: &Value) -> String {
    let bytes = serde_json::to_vec_pretty(value).expect("serialize fixture");
    std::fs::write(path, &bytes).expect("write fixture");
    format!("{:x}", Sha256::digest(&bytes))
}

fn catalog(uuid: &str, version: &str, controls: &[(&str, &str)]) -> Value {
    json!({
        "catalog": {
            "uuid": uuid,
            "metadata": {
                "title": "Synthetic redistributable framework",
                "last-modified": "2026-08-25T12:00:00Z",
                "version": version,
                "oscal-version": "1.2.3"
            },
            "groups": [{
                "id": "group-1",
                "title": "Synthetic controls",
                "controls": controls.iter().map(|(id, prose)| json!({
                    "id": id,
                    "title": format!("Control {id}"),
                    "parts": [{
                        "id": format!("{id}_smt"),
                        "name": "statement",
                        "prose": prose
                    }]
                })).collect::<Vec<_>>()
            }]
        }
    })
}

fn impact_manifest(old_hash: &str, new_hash: &str, mappings: &[Value]) -> Value {
    json!({
        "schema_version": "forge.framework-impact/1",
        "old": {
            "type": "catalog",
            "artifact": "old.json",
            "expected_sha256": old_hash,
            "root_uuid": FRAMEWORK_UUID,
            "document_version": "1.0.0",
            "oscal_version": "1.2.3"
        },
        "new": {
            "type": "catalog",
            "artifact": "new.json",
            "expected_sha256": new_hash,
            "root_uuid": FRAMEWORK_UUID,
            "document_version": "2.0.0",
            "oscal_version": "1.2.3"
        },
        "mapping_collections": mappings
    })
}

fn setup_revision() -> (TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let old_hash = write_json(
        &dir.path().join("old.json"),
        &catalog(
            FRAMEWORK_UUID,
            "1.0.0",
            &[
                ("unchanged", "Same requirement."),
                ("changed", "Old requirement."),
                ("removed", "Removed requirement."),
            ],
        ),
    );
    let new_hash = write_json(
        &dir.path().join("new.json"),
        &catalog(
            FRAMEWORK_UUID,
            "2.0.0",
            &[
                ("unchanged", "Same requirement."),
                ("changed", "Revised requirement."),
                ("added", "New requirement."),
            ],
        ),
    );
    let manifest_path = dir.path().join("impact.json");
    write_json(&manifest_path, &impact_manifest(&old_hash, &new_hash, &[]));
    (dir, manifest_path)
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_forge")).args(args).output().expect("run forge")
}

fn assert_success(output: &std::process::Output) {
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
}

fn write_disposition_fixture(path: &Path, prior_bytes: &[u8], prior_report: &Value) {
    let finding = |reason: &str| {
        prior_report["findings"]
            .as_array()
            .unwrap()
            .iter()
            .find(|finding| finding["reason_code"] == reason)
            .unwrap()["finding_id"]
            .as_str()
            .unwrap()
    };
    write_json(
        path,
        &json!({
            "schema_version": "forge.framework-impact-dispositions/1",
            "prior_report_sha256": format!("{:x}", Sha256::digest(prior_bytes)),
            "dispositions": [
                {
                    "finding_id": finding("control_added"),
                    "status": "resolved",
                    "decided_by": "reviewer",
                    "decided_at": "2026-08-25T15:00:00Z",
                    "rationale": "The new control was reviewed."
                },
                {
                    "finding_id": finding("control_content_changed"),
                    "status": "accepted-risk",
                    "decided_by": "risk-owner",
                    "decided_at": "2026-08-25T15:01:00Z",
                    "rationale": "The exact revision risk was accepted."
                },
                {
                    "finding_id": finding("control_removed"),
                    "status": "still-open",
                    "decided_by": "reviewer",
                    "decided_at": "2026-08-25T15:02:00Z",
                    "rationale": "Removal review remains open."
                }
            ]
        }),
    );
}

fn write_framework_successor_map(path: &Path) {
    write_json(
        path,
        &json!({
            "schema_version": "forge.successor-map/1",
            "relationships": [
                {
                    "relationship": "successor",
                    "old_ids": ["successor-old"],
                    "new_ids": ["successor-new"],
                    "approved_by": "reviewer",
                    "approved_at": "2026-08-25T12:00:00Z",
                    "rationale": "Reviewed successor."
                },
                {
                    "relationship": "split",
                    "old_ids": ["split-old"],
                    "new_ids": ["split-new-b", "split-new-a"],
                    "approved_by": "reviewer",
                    "approved_at": "2026-08-25T12:00:00Z",
                    "rationale": "Reviewed split."
                },
                {
                    "relationship": "merge",
                    "old_ids": ["merge-old-b", "merge-old-a"],
                    "new_ids": ["merge-new"],
                    "approved_by": "reviewer",
                    "approved_at": "2026-08-25T12:00:00Z",
                    "rationale": "Reviewed merge."
                }
            ]
        }),
    );
}

fn build_framework_migration_mapping(directory: &Path) {
    write_json(
        &directory.join("policy.json"),
        &catalog(
            "88888888-8888-4888-8888-888888888888",
            "1.0.0",
            &[("policy-1", "Policy requirement.")],
        ),
    );
    let mut mapping = mapping_manifest();
    mapping["mapping"]["maps"] = json!([{
        "key": "successor-edge",
        "relationship": "intersects-with",
        "sources": [{"type": "control", "id_ref": "successor-old"}],
        "targets": [{"type": "control", "id_ref": "policy-1"}],
        "reviewer_key": "reviewer",
        "reviewed_at": "2026-08-25T12:00:00Z",
        "rationale": "Reviewed relationship for successor control."
    }]);
    write_json(&directory.join("mapping-manifest.json"), &mapping);
    let mapping_build = run(&[
        "mapping",
        "build",
        "--manifest",
        directory.join("mapping-manifest.json").to_str().unwrap(),
        "--output",
        directory.join("mapping.json").to_str().unwrap(),
    ]);
    assert_success(&mapping_build);
}

#[test]
fn classifies_exact_changes_and_emits_byte_identical_reports() {
    let (dir, manifest_path) = setup_revision();
    let first = dir.path().join("first.json");
    let second = dir.path().join("second.json");
    for output in [&first, &second] {
        let result = run(&[
            "framework",
            "impact",
            "--manifest",
            manifest_path.to_str().unwrap(),
            "--format",
            "json",
            "--output",
            output.to_str().unwrap(),
        ]);
        assert_eq!(result.status.code(), Some(1), "{}", String::from_utf8_lossy(&result.stderr));
    }
    let first_bytes = std::fs::read(&first).unwrap();
    assert_eq!(first_bytes, std::fs::read(second).unwrap());
    let report: Value = serde_json::from_slice(&first_bytes).unwrap();
    assert_eq!(report["schema_version"], "forge.framework-impact-report/1");
    assert_eq!(report["summary"]["added"], 1);
    assert_eq!(report["summary"]["removed"], 1);
    assert_eq!(report["summary"]["content_changed"], 1);
    assert_eq!(report["summary"]["unchanged"], 1);
    let classes: Vec<_> = report["changes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|change| {
            (change["subject_id"].as_str().unwrap(), change["change_class"].as_str().unwrap())
        })
        .collect();
    assert_eq!(
        classes,
        vec![
            ("added", "added"),
            ("changed", "content-changed"),
            ("removed", "removed"),
            ("unchanged", "unchanged"),
        ]
    );
    assert!(!String::from_utf8_lossy(&first_bytes).contains("Revised requirement"));
    assert!(!String::from_utf8_lossy(&first_bytes).contains(dir.path().to_str().unwrap()));
    assert!(report["findings"].as_array().unwrap().iter().all(|finding| {
        finding["finding_id"].as_str().is_some_and(|value| uuid::Uuid::parse_str(value).is_ok())
    }));
}

#[test]
fn identical_prose_under_a_new_id_is_not_an_inferred_successor() {
    let dir = tempfile::tempdir().unwrap();
    let prose = "The organization shall review privileged access quarterly.";
    let old_hash = write_json(
        &dir.path().join("old.json"),
        &catalog(FRAMEWORK_UUID, "1.0.0", &[("old-control-id", prose)]),
    );
    let new_hash = write_json(
        &dir.path().join("new.json"),
        &catalog(FRAMEWORK_UUID, "2.0.0", &[("new-control-id", prose)]),
    );
    let manifest_path = dir.path().join("impact.json");
    write_json(&manifest_path, &impact_manifest(&old_hash, &new_hash, &[]));

    let result = run(&[
        "framework",
        "impact",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(result.status.code(), Some(1), "{}", String::from_utf8_lossy(&result.stderr));
    let report: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["summary"]["added"], 1);
    assert_eq!(report["summary"]["removed"], 1);
    assert_eq!(report["summary"]["identity_migrated"], 0);
    assert_eq!(
        report["changes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|change| (
                change["subject_id"].as_str().unwrap(),
                change["change_class"].as_str().unwrap()
            ))
            .collect::<Vec<_>>(),
        vec![("new-control-id", "added"), ("old-control-id", "removed")]
    );
    assert!(
        report["changes"]
            .as_array()
            .unwrap()
            .iter()
            .all(|change| { change.get("migration").is_none() || change["migration"].is_null() })
    );
    assert!(report["findings"].as_array().unwrap().iter().all(|finding| {
        finding["reason_code"] != "identity_migration_declared"
            && finding["change_class"] != "identity-migrated"
    }));
}

fn mapping_manifest() -> Value {
    json!({
        "schema_version": "forge.mapping-manifest/1",
        "collection": {
            "key": "impact-test-collection",
            "title": "Impact test mapping",
            "version": "1.0.0",
            "last_modified": "2026-08-25T12:00:00Z"
        },
        "reviewers": [{"key": "reviewer", "type": "person", "name": "Test Reviewer"}],
        "provenance": {
            "method": "human",
            "matching_rationale": "semantic",
            "status": "complete",
            "mapping_description": "Synthetic reviewed mapping.",
            "reviewer_keys": ["reviewer"],
            "reviewed_at": "2026-08-25T12:00:00Z"
        },
        "mapping": {
            "key": "impact-test-mapping",
            "scope": "control-only",
            "source": {"type": "catalog", "artifact": "old.json", "href": "old.json"},
            "target": {"type": "catalog", "artifact": "policy.json", "href": "policy.json"},
            "maps": [
                {
                    "key": "changed-edge",
                    "relationship": "intersects-with",
                    "sources": [{"type": "control", "id_ref": "changed"}],
                    "targets": [{"type": "control", "id_ref": "policy-1"}],
                    "reviewer_key": "reviewer",
                    "reviewed_at": "2026-08-25T12:00:00Z",
                    "rationale": "Reviewed relationship for changed control."
                },
                {
                    "key": "removed-edge",
                    "relationship": "intersects-with",
                    "sources": [{"type": "control", "id_ref": "removed"}],
                    "targets": [{"type": "control", "id_ref": "policy-1"}],
                    "reviewer_key": "reviewer",
                    "reviewed_at": "2026-08-25T12:00:00Z",
                    "rationale": "Reviewed relationship for removed control."
                }
            ]
        }
    })
}

fn applicability_mapping_manifest() -> Value {
    let mut value = mapping_manifest();
    value["collection"]["key"] = json!("applicability-impact-collection");
    value["mapping"]["key"] = json!("applicability-impact-mapping");
    value["mapping"]["source"] =
        json!({"type": "catalog", "artifact": "policy.json", "href": "policy.json"});
    value["mapping"]["target"] =
        json!({"type": "catalog", "artifact": "old.json", "href": "old.json"});
    value["mapping"]["maps"][0]["sources"] = json!([{"type": "control", "id_ref": "policy-1"}]);
    value["mapping"]["maps"][0]["targets"] = json!([{"type": "control", "id_ref": "changed"}]);
    value["mapping"]["maps"][1]["sources"] = json!([{"type": "control", "id_ref": "policy-1"}]);
    value["mapping"]["maps"][1]["targets"] = json!([{"type": "control", "id_ref": "removed"}]);
    value
}

#[test]
fn traverses_exact_mapping_dependencies_with_stable_paths_and_priorities() {
    let (dir, manifest_path) = setup_revision();
    write_json(
        &dir.path().join("policy.json"),
        &catalog(
            "88888888-8888-4888-8888-888888888888",
            "1.0.0",
            &[("policy-1", "Policy requirement.")],
        ),
    );
    let mapping_manifest_path = dir.path().join("mapping-manifest.json");
    write_json(&mapping_manifest_path, &mapping_manifest());
    let mapping_path = dir.path().join("mapping.json");
    let build = run(&[
        "mapping",
        "build",
        "--manifest",
        mapping_manifest_path.to_str().unwrap(),
        "--output",
        mapping_path.to_str().unwrap(),
    ]);
    assert!(build.status.success(), "{}", String::from_utf8_lossy(&build.stderr));

    let old_hash =
        format!("{:x}", Sha256::digest(std::fs::read(dir.path().join("old.json")).unwrap()));
    let new_hash =
        format!("{:x}", Sha256::digest(std::fs::read(dir.path().join("new.json")).unwrap()));
    write_json(
        &manifest_path,
        &impact_manifest(
            &old_hash,
            &new_hash,
            &[json!({"artifact": "mapping.json", "framework_role": "source"})],
        ),
    );
    let result = run(&[
        "framework",
        "impact",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(result.status.code(), Some(1), "{}", String::from_utf8_lossy(&result.stderr));
    let report: Value = serde_json::from_slice(&result.stdout).unwrap();
    let findings = report["findings"].as_array().unwrap();
    let changed = findings
        .iter()
        .find(|finding| finding["reason_code"] == "mapping_subject_changed")
        .expect("changed mapping dependency");
    assert_eq!(changed["priority"], "review-required");
    assert_eq!(changed["required_action"], "reapprove-mapping-rationale");
    assert_eq!(changed["dependency_path"].as_array().unwrap().len(), 5);
    assert!(changed["policy_resource_identity"].as_str().is_some());
    let removed = findings
        .iter()
        .find(|finding| finding["reason_code"] == "mapping_reference_removed")
        .expect("removed mapping dependency");
    assert_eq!(removed["priority"], "blocking");
    assert_eq!(report["summary"]["blocking"], 1);
    assert_eq!(report["summary"]["review_required"], 2);

    let mapping_alias = dir.path().join("mapping-alias.json");
    std::fs::hard_link(&mapping_path, &mapping_alias).unwrap();
    write_json(
        &manifest_path,
        &impact_manifest(
            &old_hash,
            &new_hash,
            &[
                json!({"artifact": "mapping.json", "framework_role": "source"}),
                json!({"artifact": "mapping-alias.json", "framework_role": "source"}),
            ],
        ),
    );
    let duplicate = run(&["framework", "impact", "--manifest", manifest_path.to_str().unwrap()]);
    assert_eq!(duplicate.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&duplicate.stderr)
            .contains("aliases another Mapping Collection input")
    );
}

#[test]
fn removed_control_reports_three_mapping_and_one_applicability_dependency() {
    let dir = tempfile::tempdir().unwrap();
    let old_hash = write_json(
        &dir.path().join("old.json"),
        &catalog(
            FRAMEWORK_UUID,
            "1.0.0",
            &[("stable", "Same requirement."), ("removed", "Removed requirement.")],
        ),
    );
    let new_hash = write_json(
        &dir.path().join("new.json"),
        &catalog(FRAMEWORK_UUID, "2.0.0", &[("stable", "Same requirement.")]),
    );
    build_three_removed_control_mappings(dir.path());
    write_removed_control_applicability(dir.path());

    let manifest_path = dir.path().join("impact.json");
    let mut manifest = impact_manifest(
        &old_hash,
        &new_hash,
        &[json!({"artifact": "mapping.json", "framework_role": "source"})],
    );
    manifest["applicability_manifest"] = json!("applicability.json");
    write_json(&manifest_path, &manifest);
    let result = run(&[
        "framework",
        "impact",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(result.status.code(), Some(1), "{}", String::from_utf8_lossy(&result.stderr));
    let report: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_removed_control_blast_radius(&report);
}

fn build_three_removed_control_mappings(directory: &Path) {
    write_json(
        &directory.join("policy.json"),
        &catalog(
            "88888888-8888-4888-8888-888888888888",
            "1.0.0",
            &[
                ("policy-1", "Policy requirement one."),
                ("policy-2", "Policy requirement two."),
                ("policy-3", "Policy requirement three."),
            ],
        ),
    );
    let mut mapping = mapping_manifest();
    mapping["mapping"]["maps"] = json!([
        removed_mapping("removed-edge-1", "policy-1", "Reviewed first dependency."),
        removed_mapping("removed-edge-2", "policy-2", "Reviewed second dependency."),
        removed_mapping("removed-edge-3", "policy-3", "Reviewed third dependency.")
    ]);
    write_json(&directory.join("mapping-manifest.json"), &mapping);
    let mapping_build = run(&[
        "mapping",
        "build",
        "--manifest",
        directory.join("mapping-manifest.json").to_str().unwrap(),
        "--output",
        directory.join("mapping.json").to_str().unwrap(),
    ]);
    assert_success(&mapping_build);
}

fn removed_mapping(key: &str, target: &str, rationale: &str) -> Value {
    json!({
        "key": key,
        "relationship": "intersects-with",
        "sources": [{"type": "control", "id_ref": "removed"}],
        "targets": [{"type": "control", "id_ref": target}],
        "reviewer_key": "reviewer",
        "reviewed_at": "2026-08-25T12:00:00Z",
        "rationale": rationale
    })
}

fn write_removed_control_applicability(directory: &Path) {
    let path = directory.join("applicability.json");
    let init = run(&[
        "applicability",
        "init",
        "--framework",
        directory.join("old.json").to_str().unwrap(),
        "--output",
        path.to_str().unwrap(),
    ]);
    assert_success(&init);
    let mut applicability: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    applicability["reviewers"] =
        json!([{"key": "scope-reviewer", "type": "person", "name": "Scope Reviewer"}]);
    applicability["decisions"] = json!([{
        "control_id": "removed",
        "state": "applicable",
        "reviewer_key": "scope-reviewer",
        "reviewed_at": "2026-08-25T12:00:00Z"
    }]);
    write_json(&path, &applicability);
}

fn assert_removed_control_blast_radius(report: &Value) {
    let removed_changes = report["changes"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|change| change["subject_id"] == "removed")
        .collect::<Vec<_>>();
    assert_eq!(removed_changes.len(), 1);
    assert_eq!(removed_changes[0]["change_class"], "removed");

    let removed_findings = report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| finding["subject_id"] == "removed")
        .collect::<Vec<_>>();
    assert_eq!(removed_findings.len(), 5, "one subject plus four dependency findings");
    let mapping_findings = removed_findings
        .iter()
        .filter(|finding| finding["reason_code"] == "mapping_reference_removed")
        .collect::<Vec<_>>();
    assert_eq!(mapping_findings.len(), 3);
    assert!(mapping_findings.iter().all(|finding| {
        finding["priority"] == "blocking"
            && finding["dependency_path"].as_array().is_some_and(|path| path.len() == 5)
    }));
    let applicability_findings = removed_findings
        .iter()
        .filter(|finding| finding["reason_code"] == "applicability_decision_removed")
        .collect::<Vec<_>>();
    assert_eq!(applicability_findings.len(), 1);
    assert_eq!(applicability_findings[0]["prior_decision_state"], "applicable");
    assert_eq!(applicability_findings[0]["owner"], "scope-reviewer");
}

#[test]
fn rejects_mapping_from_a_different_baseline_without_partial_output() {
    let (dir, manifest_path) = setup_revision();
    write_json(
        &dir.path().join("policy.json"),
        &catalog(
            "88888888-8888-4888-8888-888888888888",
            "1.0.0",
            &[("policy-1", "Policy requirement.")],
        ),
    );
    let mapping_manifest_path = dir.path().join("mapping-manifest.json");
    write_json(&mapping_manifest_path, &mapping_manifest());
    let mapping_path = dir.path().join("mapping.json");
    let build = run(&[
        "mapping",
        "build",
        "--manifest",
        mapping_manifest_path.to_str().unwrap(),
        "--output",
        mapping_path.to_str().unwrap(),
    ]);
    assert!(build.status.success(), "{}", String::from_utf8_lossy(&build.stderr));
    let mut mapping: Value =
        serde_json::from_slice(&std::fs::read(&mapping_path).unwrap()).unwrap();
    let props = mapping["mapping-collection"]["mappings"][0]["source-resource"]["props"]
        .as_array_mut()
        .unwrap();
    props.iter_mut().find(|prop| prop["name"] == "raw-sha256").unwrap()["value"] =
        json!("0".repeat(64));
    write_json(&mapping_path, &mapping);
    let old_hash =
        format!("{:x}", Sha256::digest(std::fs::read(dir.path().join("old.json")).unwrap()));
    let new_hash =
        format!("{:x}", Sha256::digest(std::fs::read(dir.path().join("new.json")).unwrap()));
    write_json(
        &manifest_path,
        &impact_manifest(
            &old_hash,
            &new_hash,
            &[json!({"artifact": "mapping.json", "framework_role": "source"})],
        ),
    );
    let output_path = dir.path().join("report.json");
    let result = run(&[
        "framework",
        "impact",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--format",
        "json",
        "--output",
        output_path.to_str().unwrap(),
    ]);
    assert_eq!(result.status.code(), Some(2));
    assert!(!output_path.exists());
    assert!(String::from_utf8_lossy(&result.stderr).contains("different old baseline"));
}

#[test]
fn gate_thresholds_and_destination_aliases_are_enforced() {
    let dir = tempfile::tempdir().unwrap();
    let old_path = dir.path().join("old.json");
    let new_path = dir.path().join("new.json");
    let old_hash = write_json(
        &old_path,
        &catalog(
            FRAMEWORK_UUID,
            "1.0.0",
            &[("stable", "Same requirement."), ("removed", "Old requirement.")],
        ),
    );
    let new_hash = write_json(
        &new_path,
        &catalog(FRAMEWORK_UUID, "2.0.0", &[("stable", "Same requirement.")]),
    );
    let manifest_path = dir.path().join("impact.json");
    write_json(&manifest_path, &impact_manifest(&old_hash, &new_hash, &[]));

    let default_gate = run(&[
        "framework",
        "impact",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert!(default_gate.status.success(), "{}", String::from_utf8_lossy(&default_gate.stderr));
    let strict_gate = run(&[
        "framework",
        "impact",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--format",
        "json",
        "--fail-on",
        "any",
    ]);
    assert_eq!(strict_gate.status.code(), Some(1));

    let original = std::fs::read(&old_path).unwrap();
    let alias = run(&[
        "framework",
        "impact",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--output",
        old_path.to_str().unwrap(),
    ]);
    assert_eq!(alias.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&alias.stderr).contains("aliases a framework impact input"));
    assert_eq!(std::fs::read(&old_path).unwrap(), original);

    let hard_link = dir.path().join("report-alias.json");
    std::fs::hard_link(&old_path, &hard_link).unwrap();
    let hard_link_alias = run(&[
        "framework",
        "impact",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--output",
        hard_link.to_str().unwrap(),
    ]);
    assert_eq!(hard_link_alias.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&hard_link_alias.stderr)
            .contains("aliases a framework impact input")
    );
    assert_eq!(std::fs::read(old_path).unwrap(), original);
}

#[test]
fn dispositions_preserve_raw_findings_control_gates_and_retain_prior_only_history() {
    let (dir, manifest_path) = setup_revision();
    let prior_path = dir.path().join("prior-report.json");
    let prior = run(&[
        "framework",
        "impact",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--format",
        "json",
        "--output",
        prior_path.to_str().unwrap(),
    ]);
    assert_eq!(prior.status.code(), Some(1));
    let prior_bytes = std::fs::read(&prior_path).unwrap();
    let prior_report: Value = serde_json::from_slice(&prior_bytes).unwrap();
    let disposition_path = dir.path().join("dispositions.json");
    write_disposition_fixture(&disposition_path, &prior_bytes, &prior_report);
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(&manifest_path).unwrap()).unwrap();
    manifest["prior_report"] = json!("prior-report.json");
    manifest["disposition_file"] = json!("dispositions.json");
    write_json(&manifest_path, &manifest);

    let default_gate = run(&[
        "framework",
        "impact",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert!(default_gate.status.success(), "{}", String::from_utf8_lossy(&default_gate.stderr));
    let report: Value = serde_json::from_slice(&default_gate.stdout).unwrap();
    assert_eq!(report["summary"]["findings"], prior_report["summary"]["findings"]);
    assert_eq!(report["summary"]["dispositioned_resolved"], 1);
    assert_eq!(report["summary"]["dispositioned_accepted_risk"], 1);
    assert_eq!(report["summary"]["dispositioned_still_open"], 1);
    let strict_gate = run(&[
        "framework",
        "impact",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--fail-on",
        "any",
    ]);
    assert_eq!(strict_gate.status.code(), Some(1));

    write_json(
        &dir.path().join("successors.json"),
        &json!({
            "schema_version": "forge.successor-map/1",
            "relationships": [{
                "relationship": "successor",
                "old_ids": ["removed"],
                "new_ids": ["added"],
                "approved_by": "reviewer",
                "approved_at": "2026-08-25T16:00:00Z",
                "rationale": "Reviewed framework successor."
            }]
        }),
    );
    manifest["successor_map"] = json!("successors.json");
    write_json(&manifest_path, &manifest);
    let migrated = run(&[
        "framework",
        "impact",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(migrated.status.code(), Some(1));
    let migrated_report: Value = serde_json::from_slice(&migrated.stdout).unwrap();
    assert_eq!(migrated_report["prior_only_dispositions"].as_array().unwrap().len(), 2);
    assert_eq!(migrated_report["summary"]["identity_migrated"], 1);

    let alias = run(&[
        "framework",
        "impact",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--output",
        prior_path.to_str().unwrap(),
    ]);
    assert_eq!(alias.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&alias.stderr).contains("aliases a framework impact input"));
}

#[test]
fn github_annotations_are_deterministic_content_safe_workflow_commands() {
    let (dir, manifest_path) = setup_revision();
    let first = run(&[
        "framework",
        "impact",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--format",
        "github",
    ]);
    let second = run(&[
        "framework",
        "impact",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--format",
        "github",
    ]);
    assert_eq!(first.status.code(), Some(1));
    assert_eq!(first.stdout, second.stdout);
    let rendered = String::from_utf8(first.stdout).unwrap();
    assert!(rendered.lines().all(|line| {
        line.starts_with("::warning title=") || line.starts_with("::notice title=")
    }));
    assert!(rendered.contains("finding="));
    assert!(rendered.contains("action=review-applicability"));
    assert!(!rendered.contains("Revised requirement"));
    assert!(!rendered.contains(dir.path().to_str().unwrap()));
}

#[test]
fn markdown_and_html_cli_formats_render_complete_static_reports() {
    let (_directory, manifest_path) = setup_revision();

    for (format, prefix, suffix) in [
        ("markdown", "# FORGE framework change impact report\n", "\n"),
        ("html", "<!doctype html>\n<html lang=\"en\">", "</html>\n"),
    ] {
        let first = run(&[
            "framework",
            "impact",
            "--manifest",
            manifest_path.to_str().unwrap(),
            "--format",
            format,
        ]);
        let second = run(&[
            "framework",
            "impact",
            "--manifest",
            manifest_path.to_str().unwrap(),
            "--format",
            format,
        ]);
        assert_eq!(first.status.code(), Some(1));
        assert_eq!(first.stdout, second.stdout);
        let rendered = String::from_utf8(first.stdout).unwrap();
        assert!(rendered.starts_with(prefix));
        assert!(rendered.ends_with(suffix));
        assert!(rendered.contains("Review findings"));
        assert!(!rendered.contains("Revised requirement"));
    }
}

#[test]
fn duplicate_manifest_keys_are_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let manifest = dir.path().join("impact.json");
    std::fs::write(
        &manifest,
        r#"{"schema_version":"forge.framework-impact/1","schema_version":"forge.framework-impact/1"}"#,
    )
    .unwrap();
    let result = run(&["framework", "impact", "--manifest", manifest.to_str().unwrap()]);
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("duplicate object key"));
}

#[test]
fn closed_manifest_enforces_unknown_field_depth_and_size_bounds_without_output() {
    let (dir, manifest_path) = setup_revision();
    let output_path = dir.path().join("report.json");
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(&manifest_path).unwrap()).unwrap();
    manifest["unexpected"] = json!(true);
    write_json(&manifest_path, &manifest);
    let unknown = run(&[
        "framework",
        "impact",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--output",
        output_path.to_str().unwrap(),
    ]);
    assert_eq!(unknown.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&unknown.stderr).contains("unknown field"));
    assert!(!output_path.exists());

    let nested = format!("{}null{}", "[".repeat(66), "]".repeat(66));
    std::fs::write(
        &manifest_path,
        format!(r#"{{"schema_version":"forge.framework-impact/1","nested":{nested}}}"#),
    )
    .unwrap();
    let too_deep = run(&[
        "framework",
        "impact",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--output",
        output_path.to_str().unwrap(),
    ]);
    assert_eq!(too_deep.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&too_deep.stderr).contains("maximum JSON depth"));
    assert!(!output_path.exists());

    std::fs::write(&manifest_path, vec![b' '; 2 * 1024 * 1024 + 1]).unwrap();
    let too_large = run(&[
        "framework",
        "impact",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--output",
        output_path.to_str().unwrap(),
    ]);
    assert_eq!(too_large.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&too_large.stderr).contains("exceeding the 2.0MB limit"));
    assert!(!output_path.exists());
}

#[cfg(unix)]
#[test]
fn symlinked_manifest_is_rejected_without_output() {
    let (dir, manifest_path) = setup_revision();
    let symlink_path = dir.path().join("impact-link.json");
    std::os::unix::fs::symlink(&manifest_path, &symlink_path).unwrap();
    let output_path = dir.path().join("report.json");
    let result = run(&[
        "framework",
        "impact",
        "--manifest",
        symlink_path.to_str().unwrap(),
        "--output",
        output_path.to_str().unwrap(),
    ]);
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("must not be a symbolic link"));
    assert!(!output_path.exists());
}

#[test]
fn text_report_escapes_terminal_control_characters() {
    let dir = tempfile::tempdir().unwrap();
    let version = "1.0.0\u{1b}[31mforged";
    let framework = catalog(FRAMEWORK_UUID, version, &[("stable", "Same requirement.")]);
    let old_hash = write_json(&dir.path().join("old.json"), &framework);
    let new_hash = write_json(&dir.path().join("new.json"), &framework);
    let mut manifest = impact_manifest(&old_hash, &new_hash, &[]);
    manifest["old"]["document_version"] = json!(version);
    manifest["new"]["document_version"] = json!(version);
    let manifest_path = dir.path().join("impact.json");
    write_json(&manifest_path, &manifest);

    let result = run(&["framework", "impact", "--manifest", manifest_path.to_str().unwrap()]);
    assert_success(&result);
    let rendered = String::from_utf8(result.stdout).unwrap();
    assert!(!rendered.contains('\u{1b}'));
    assert!(rendered.contains(r"version=1.0.0\u{1b}[31mforged"));
}

#[test]
fn mixed_catalog_and_profile_revisions_are_rejected() {
    let (dir, manifest_path) = setup_revision();
    let output_path = dir.path().join("framework-impact-report.json");
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(&manifest_path).unwrap()).unwrap();
    manifest["new"]["type"] = json!("profile");
    write_json(&manifest_path, &manifest);
    let result = run(&[
        "framework",
        "impact",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--output",
        output_path.to_str().unwrap(),
    ]);
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("must describe the same OSCAL model"));
    assert!(!output_path.exists());
}

#[test]
fn declared_successor_split_and_merge_preserve_cardinality_and_review_evidence() {
    let dir = tempfile::tempdir().unwrap();
    let old_hash = write_json(
        &dir.path().join("old.json"),
        &catalog(
            FRAMEWORK_UUID,
            "1.0.0",
            &[
                ("successor-old", "Old successor requirement."),
                ("split-old", "Old combined requirement."),
                ("merge-old-a", "Old merge requirement A."),
                ("merge-old-b", "Old merge requirement B."),
            ],
        ),
    );
    let new_hash = write_json(
        &dir.path().join("new.json"),
        &catalog(
            FRAMEWORK_UUID,
            "2.0.0",
            &[
                ("successor-new", "New successor requirement."),
                ("split-new-a", "New split requirement A."),
                ("split-new-b", "New split requirement B."),
                ("merge-new", "New merged requirement."),
            ],
        ),
    );
    let successor_path = dir.path().join("successors.json");
    write_framework_successor_map(&successor_path);
    build_framework_migration_mapping(dir.path());
    let manifest_path = dir.path().join("impact.json");
    let mut manifest = impact_manifest(
        &old_hash,
        &new_hash,
        &[json!({"artifact": "mapping.json", "framework_role": "source"})],
    );
    manifest["successor_map"] = json!("successors.json");
    write_json(&manifest_path, &manifest);
    let result = run(&[
        "framework",
        "impact",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(result.status.code(), Some(1), "{}", String::from_utf8_lossy(&result.stderr));
    let report: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["summary"]["identity_migrated"], 3);
    assert_eq!(report["summary"]["added"], 0);
    assert_eq!(report["summary"]["removed"], 0);
    assert_split_review_contract(&report);
    let migrated_mapping = report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|finding| finding["reason_code"] == "mapping_subject_migrated")
        .unwrap();
    assert_eq!(migrated_mapping["subject_id"], "successor-old");
    assert_eq!(migrated_mapping["change_class"], "identity-migrated");
    assert_eq!(migrated_mapping["dependency_path"].as_array().unwrap().len(), 5);
}

fn assert_split_review_contract(report: &Value) {
    let migrations = report["changes"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|change| change["change_class"] == "identity-migrated")
        .collect::<Vec<_>>();
    assert_eq!(migrations.len(), 3);
    let split =
        migrations.iter().find(|change| change["migration"]["relationship"] == "split").unwrap();
    assert_eq!(split["old_subjects"].as_array().unwrap().len(), 1);
    assert_eq!(split["new_subjects"].as_array().unwrap().len(), 2);
    assert_eq!(split["migration"]["approved_by"], "reviewer");
    assert!(split["new_subjects"].as_array().unwrap().iter().all(|subject| {
        subject["id"].as_str().is_some() && subject["sha256"].as_str().is_some()
    }));
    assert_eq!(
        split["new_subjects"]
            .as_array()
            .unwrap()
            .iter()
            .map(|subject| subject["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["split-new-a", "split-new-b"]
    );
    let split_review = report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|finding| {
            finding["reason_code"] == "identity_migration_declared"
                && finding["migration"]["relationship"] == "split"
        })
        .expect("split remains in the review queue");
    assert_eq!(split_review["subject_id"], "split-old=>split-new-a,split-new-b");
    assert_eq!(split_review["old_subjects"].as_array().unwrap().len(), 1);
    assert_eq!(split_review["new_subjects"].as_array().unwrap().len(), 2);
    assert_eq!(split_review["priority"], "review-required");
    assert_eq!(split_review["required_action"], "review-identity-migration");
    assert!(split_review.get("disposition").is_none() || split_review["disposition"].is_null());
    assert!(report["findings"].as_array().unwrap().iter().all(|finding| {
        !(finding["reason_code"] == "control_added"
            && matches!(finding["subject_id"].as_str(), Some("split-new-a" | "split-new-b")))
    }));
}

#[test]
fn applicability_impacts_preserve_prior_gap_state_and_feed_lifecycle_review() {
    let (dir, impact_manifest_path) = setup_revision();
    write_json(
        &dir.path().join("policy.json"),
        &catalog(
            "88888888-8888-4888-8888-888888888888",
            "1.0.0",
            &[("policy-1", "Policy requirement.")],
        ),
    );
    let mapping_manifest_path = dir.path().join("mapping-manifest.json");
    write_json(&mapping_manifest_path, &applicability_mapping_manifest());
    let mapping_path = dir.path().join("mapping.json");
    let mapping_build = run(&[
        "mapping",
        "build",
        "--manifest",
        mapping_manifest_path.to_str().unwrap(),
        "--output",
        mapping_path.to_str().unwrap(),
    ]);
    assert_success(&mapping_build);

    let applicability_path = dir.path().join("applicability.json");
    let applicability_init = run(&[
        "applicability",
        "init",
        "--framework",
        dir.path().join("old.json").to_str().unwrap(),
        "--output",
        applicability_path.to_str().unwrap(),
    ]);
    assert_success(&applicability_init);
    let mut applicability: Value =
        serde_json::from_slice(&std::fs::read(&applicability_path).unwrap()).unwrap();
    applicability["reviewers"] =
        json!([{"key": "scope-reviewer", "type": "person", "name": "Scope Reviewer"}]);
    applicability["decisions"] = json!([
        {
            "control_id": "changed",
            "state": "not-applicable",
            "reviewer_key": "scope-reviewer",
            "reviewed_at": "2026-08-25T12:00:00Z",
            "rationale": "The prior revision was explicitly excluded."
        },
        {
            "control_id": "removed",
            "state": "applicable",
            "reviewer_key": "scope-reviewer",
            "reviewed_at": "2026-08-25T12:00:00Z"
        }
    ]);
    applicability["mapping_collections"] = json!(["mapping.json"]);
    write_json(&applicability_path, &applicability);

    let old_hash =
        format!("{:x}", Sha256::digest(std::fs::read(dir.path().join("old.json")).unwrap()));
    let new_hash =
        format!("{:x}", Sha256::digest(std::fs::read(dir.path().join("new.json")).unwrap()));
    let mut impact = impact_manifest(
        &old_hash,
        &new_hash,
        &[json!({"artifact": "mapping.json", "framework_role": "target"})],
    );
    impact["applicability_manifest"] = json!("applicability.json");
    write_json(&impact_manifest_path, &impact);
    let result = run(&[
        "framework",
        "impact",
        "--manifest",
        impact_manifest_path.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(result.status.code(), Some(1), "{}", String::from_utf8_lossy(&result.stderr));
    let report: Value = serde_json::from_slice(&result.stdout).unwrap();
    let changed = report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|finding| finding["reason_code"] == "applicability_decision_changed")
        .expect("changed applicability finding");
    assert_eq!(changed["prior_gap_classification"], "not-applicable");
    assert_eq!(changed["owner"], "scope-reviewer");
    assert_eq!(changed["policy_sources"], json!(["policy.json"]));
    let removed = report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|finding| finding["reason_code"] == "applicability_decision_removed")
        .expect("removed applicability finding");
    assert_eq!(removed["prior_gap_classification"], "applicable-mapped");

    let finding_id = changed["finding_id"].as_str().unwrap();
    assert_lifecycle_accepts_impact_finding(dir.path(), finding_id);
    assert_applicability_finding_stability_and_closed_portfolio(
        &applicability_path,
        &mut applicability,
        &impact_manifest_path,
        &mut impact,
        finding_id,
    );
}

fn assert_lifecycle_accepts_impact_finding(directory: &Path, finding_id: &str) {
    let canonical_dir = directory.canonicalize().unwrap();
    let policy_source = canonical_dir.join("policy.md");
    std::fs::write(&policy_source, "# Policy\n\n1. The organization shall review changes.\n")
        .unwrap();
    let lifecycle_path = canonical_dir.join("lifecycle.json");
    let lifecycle_init = run(&[
        "lifecycle",
        "init",
        "--source",
        policy_source.to_str().unwrap(),
        "--artifact",
        canonical_dir.join("policy.json").to_str().unwrap(),
        "--output",
        lifecycle_path.to_str().unwrap(),
        "--policy-key",
        "synthetic-policy",
        "--version-key",
        "v1",
        "--title",
        "Synthetic Policy",
        "--owner",
        "owner",
        "--party",
        "owner=owner,author",
        "--party",
        "reviewer=reviewer",
        "--party",
        "approver=approver",
        "--next-review",
        "2027-08-25",
    ]);
    assert!(lifecycle_init.status.success(), "{}", String::from_utf8_lossy(&lifecycle_init.stderr));
    let transition = run(&[
        "lifecycle",
        "transition",
        "--record",
        lifecycle_path.to_str().unwrap(),
        "--to",
        "in-review",
        "--actor",
        "reviewer",
        "--role",
        "reviewer",
        "--at",
        "2026-08-25T14:00:00Z",
        "--rationale",
        "Framework impact requires review.",
        "--impact-finding-id",
        finding_id,
        "--apply",
    ]);
    assert!(transition.status.success(), "{}", String::from_utf8_lossy(&transition.stderr));
    let lifecycle: Value =
        serde_json::from_slice(&std::fs::read(&lifecycle_path).unwrap()).unwrap();
    assert_eq!(lifecycle["history"][0]["impact_finding_ids"], json!([finding_id]));
}

fn assert_applicability_finding_stability_and_closed_portfolio(
    applicability_path: &Path,
    applicability: &mut Value,
    impact_manifest_path: &Path,
    impact: &mut Value,
    finding_id: &str,
) {
    applicability["reviewers"].as_array_mut().unwrap().push(json!({
        "key": "unrelated-reviewer",
        "type": "person",
        "name": "Unrelated Reviewer"
    }));
    write_json(applicability_path, applicability);
    let unrelated_manifest_edit = run(&[
        "framework",
        "impact",
        "--manifest",
        impact_manifest_path.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(unrelated_manifest_edit.status.code(), Some(1));
    let updated_report: Value = serde_json::from_slice(&unrelated_manifest_edit.stdout).unwrap();
    let updated_finding_id = updated_report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|finding| finding["reason_code"] == "applicability_decision_changed")
        .unwrap()["finding_id"]
        .as_str()
        .unwrap();
    assert_eq!(updated_finding_id, finding_id);

    impact["mapping_collections"] = json!([]);
    write_json(impact_manifest_path, impact);
    let mismatched_portfolio =
        run(&["framework", "impact", "--manifest", impact_manifest_path.to_str().unwrap()]);
    assert_eq!(mismatched_portfolio.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&mismatched_portfolio.stderr)
            .contains("does not exactly match framework-role target inputs")
    );
}

fn profile(uuid: &str, version: &str, import_href: &str) -> Value {
    json!({
        "profile": {
            "uuid": uuid,
            "metadata": {
                "title": "Synthetic framework profile",
                "last-modified": "2026-08-25T12:00:00Z",
                "version": version,
                "oscal-version": "1.2.3"
            },
            "imports": [{"href": import_href, "include-all": {}}]
        }
    })
}

#[test]
fn profile_companion_pairs_use_resolved_catalogs_for_change_classification() {
    let dir = tempfile::tempdir().unwrap();
    let old_catalog_hash = write_json(
        &dir.path().join("old-resolved.json"),
        &catalog(
            FRAMEWORK_UUID,
            "1.0.0",
            &[("stable", "Same requirement."), ("changed", "Old requirement.")],
        ),
    );
    let new_catalog_hash = write_json(
        &dir.path().join("new-resolved.json"),
        &catalog(
            FRAMEWORK_UUID,
            "2.0.0",
            &[("stable", "Same requirement."), ("changed", "New requirement.")],
        ),
    );
    let old_profile_hash = write_json(
        &dir.path().join("old-profile.json"),
        &profile("aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa", "1.0.0", "old-resolved.json"),
    );
    let new_profile_hash = write_json(
        &dir.path().join("new-profile.json"),
        &profile("bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb", "2.0.0", "new-resolved.json"),
    );
    let manifest_path = dir.path().join("impact.json");
    write_json(
        &manifest_path,
        &json!({
            "schema_version": "forge.framework-impact/1",
            "old": {
                "type": "profile",
                "artifact": "old-profile.json",
                "resolved_catalog": "old-resolved.json",
                "resolved_catalog_attestation": true,
                "expected_sha256": old_profile_hash,
                "expected_resolved_catalog_sha256": old_catalog_hash,
                "root_uuid": "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
                "document_version": "1.0.0",
                "oscal_version": "1.2.3"
            },
            "new": {
                "type": "profile",
                "artifact": "new-profile.json",
                "resolved_catalog": "new-resolved.json",
                "resolved_catalog_attestation": true,
                "expected_sha256": new_profile_hash,
                "expected_resolved_catalog_sha256": new_catalog_hash,
                "root_uuid": "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
                "document_version": "2.0.0",
                "oscal_version": "1.2.3"
            },
            "mapping_collections": []
        }),
    );
    let result = run(&[
        "framework",
        "impact",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let report: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["old"]["resource_type"], "profile");
    assert_eq!(report["summary"]["content_changed"], 1);
    assert_eq!(report["summary"]["unchanged"], 1);
    assert_eq!(report["summary"]["review_required"], 0);
}
