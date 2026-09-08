//! PRD 057 deterministic framework-impact detail filter tests.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

const FRAMEWORK_UUID: &str = "77777777-7777-4777-8777-777777777777";
const POLICY_UUID: &str = "88888888-8888-4888-8888-888888888888";

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_forge")).args(args).output().expect("run forge")
}

fn write_json(path: &Path, value: &Value) -> String {
    let bytes = serde_json::to_vec_pretty(value).expect("serialize fixture");
    std::fs::write(path, &bytes).expect("write fixture");
    format!("{:x}", Sha256::digest(&bytes))
}

fn catalog(uuid: &str, version: &str, groups: &[(&str, &[(&str, &str)])]) -> Value {
    json!({
        "catalog": {
            "uuid": uuid,
            "metadata": {
                "title": "Synthetic redistributable framework",
                "last-modified": "2026-08-25T12:00:00Z",
                "version": version,
                "oscal-version": "1.2.3"
            },
            "groups": groups.iter().map(|(group, controls)| json!({
                "id": group,
                "title": format!("Group {group}"),
                "controls": controls.iter().map(|(id, prose)| json!({
                    "id": id,
                    "title": format!("Control {id}"),
                    "parts": [{
                        "id": format!("{id}_smt"),
                        "name": "statement",
                        "prose": prose
                    }]
                })).collect::<Vec<_>>()
            })).collect::<Vec<_>>()
        }
    })
}

fn impact_manifest(old_hash: &str, new_hash: &str) -> Value {
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
        "mapping_collections": [{
            "artifact": "mapping.json",
            "framework_role": "target"
        }],
        "applicability_manifest": "applicability.json"
    })
}

fn mapping_manifest() -> Value {
    json!({
        "schema_version": "forge.mapping-manifest/1",
        "collection": {
            "key": "impact-filter-collection",
            "title": "Impact filter mapping",
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
            "key": "impact-filter-mapping",
            "scope": "control-only",
            "source": {"type": "catalog", "artifact": "policy.json", "href": "policy.json"},
            "target": {"type": "catalog", "artifact": "old.json", "href": "old.json"},
            "maps": [
                {
                    "key": "changed-edge",
                    "relationship": "intersects-with",
                    "sources": [{"type": "control", "id_ref": "policy-1"}],
                    "targets": [{"type": "control", "id_ref": "changed"}],
                    "reviewer_key": "reviewer",
                    "reviewed_at": "2026-08-25T12:00:00Z",
                    "rationale": "Reviewed changed control relationship."
                },
                {
                    "key": "removed-edge",
                    "relationship": "intersects-with",
                    "sources": [{"type": "control", "id_ref": "policy-1"}],
                    "targets": [{"type": "control", "id_ref": "removed"}],
                    "reviewer_key": "reviewer",
                    "reviewed_at": "2026-08-25T12:00:00Z",
                    "rationale": "Reviewed removed control relationship."
                }
            ]
        }
    })
}

fn setup_filter_fixture() -> (TempDir, PathBuf) {
    let directory = tempfile::tempdir().expect("tempdir");
    let old_hash = write_json(
        &directory.path().join("old.json"),
        &catalog(
            FRAMEWORK_UUID,
            "1.0.0",
            &[
                ("group-alpha", &[("changed", "Old requirement.")]),
                ("group-beta", &[("removed", "Removed requirement.")]),
            ],
        ),
    );
    let new_hash = write_json(
        &directory.path().join("new.json"),
        &catalog(
            FRAMEWORK_UUID,
            "2.0.0",
            &[
                ("group-alpha", &[("changed", "Revised requirement.")]),
                ("group-gamma", &[("added", "New requirement.")]),
            ],
        ),
    );
    write_json(
        &directory.path().join("policy.json"),
        &catalog(POLICY_UUID, "1.0.0", &[("policy", &[("policy-1", "Policy requirement.")])]),
    );
    write_json(&directory.path().join("mapping-manifest.json"), &mapping_manifest());
    let mapping = run(&[
        "mapping",
        "build",
        "--manifest",
        directory.path().join("mapping-manifest.json").to_str().unwrap(),
        "--output",
        directory.path().join("mapping.json").to_str().unwrap(),
    ]);
    assert!(mapping.status.success(), "{}", String::from_utf8_lossy(&mapping.stderr));

    let applicability_path = directory.path().join("applicability.json");
    let initialized = run(&[
        "applicability",
        "init",
        "--framework",
        directory.path().join("old.json").to_str().unwrap(),
        "--output",
        applicability_path.to_str().unwrap(),
    ]);
    assert!(initialized.status.success(), "{}", String::from_utf8_lossy(&initialized.stderr));
    let mut applicability: Value =
        serde_json::from_slice(&std::fs::read(&applicability_path).unwrap()).unwrap();
    applicability["reviewers"] = json!([
        {"key": "scope-alpha", "type": "person", "name": "Alpha Owner"},
        {"key": "scope-beta", "type": "person", "name": "Beta Owner"}
    ]);
    applicability["decisions"] = json!([
        {
            "control_id": "changed",
            "state": "applicable",
            "reviewer_key": "scope-alpha",
            "reviewed_at": "2026-08-25T13:00:00Z"
        },
        {
            "control_id": "removed",
            "state": "not-applicable",
            "reviewer_key": "scope-beta",
            "reviewed_at": "2026-08-25T13:00:00Z",
            "rationale": "Explicitly outside the prior scope."
        }
    ]);
    applicability["mapping_collections"] = json!(["mapping.json"]);
    write_json(&applicability_path, &applicability);

    let manifest_path = directory.path().join("impact.json");
    write_json(&manifest_path, &impact_manifest(&old_hash, &new_hash));
    (directory, manifest_path)
}

fn filtered_report(manifest: &Path, flag: &str, value: &str) -> (Output, Value) {
    let output = run(&[
        "framework",
        "impact",
        "--manifest",
        manifest.to_str().unwrap(),
        "--format",
        "json",
        flag,
        value,
    ]);
    let report = serde_json::from_slice(&output.stdout).expect("valid filtered JSON report");
    (output, report)
}

#[test]
fn filters_exact_finding_details_without_changing_totals_or_gate() {
    let (_directory, manifest) = setup_filter_fixture();

    let (group_status, group) = filtered_report(&manifest, "--group", "group-alpha");
    assert_eq!(group_status.status.code(), Some(1));
    assert!(group["findings"].as_array().unwrap().iter().all(|finding| {
        finding["framework_groups"].as_array().unwrap().iter().any(|value| value == "group-alpha")
    }));

    let (decision_status, decision) = filtered_report(&manifest, "--decision-state", "applicable");
    assert_eq!(decision_status.status.code(), Some(1));
    assert_eq!(decision["matched_findings"], 1);
    assert_eq!(decision["findings"][0]["prior_decision_state"], "applicable");

    let (source_status, source) = filtered_report(&manifest, "--policy-source", "policy.json");
    assert_eq!(source_status.status.code(), Some(1));
    assert!(source["findings"].as_array().unwrap().iter().all(|finding| {
        finding["policy_sources"].as_array().unwrap().iter().any(|value| value == "policy.json")
    }));

    let (owner_status, owner) = filtered_report(&manifest, "--owner", "scope-beta");
    assert_eq!(owner_status.status.code(), Some(1));
    assert_eq!(owner["matched_findings"], 1);
    assert_eq!(owner["findings"][0]["owner"], "scope-beta");

    let (priority_status, priority) = filtered_report(&manifest, "--priority", "informational");
    assert_eq!(
        priority_status.status.code(),
        Some(1),
        "a hidden review-required finding must still fire the default gate"
    );
    assert!(
        priority["findings"]
            .as_array()
            .unwrap()
            .iter()
            .all(|finding| { finding["priority"] == "informational" })
    );
    assert_eq!(priority["summary"]["findings"], group["summary"]["findings"]);
    assert!(
        priority["summary"]["findings"].as_u64().unwrap()
            > priority["matched_findings"].as_u64().unwrap()
    );
}

#[test]
fn filters_reject_ambiguous_groups_invalid_values_and_unsafe_policy_hrefs() {
    let (directory, manifest) = setup_filter_fixture();
    let invalid = run(&[
        "framework",
        "impact",
        "--manifest",
        manifest.to_str().unwrap(),
        "--group",
        " group-alpha",
    ]);
    assert_eq!(invalid.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("leading or trailing whitespace"));

    let unsafe_filter = run(&[
        "framework",
        "impact",
        "--manifest",
        manifest.to_str().unwrap(),
        "--policy-source",
        "file:///private/secret-policy.json",
    ]);
    assert_eq!(unsafe_filter.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&unsafe_filter.stderr).contains("absolute local path"));
    assert!(unsafe_filter.stdout.is_empty());

    let mut old: Value =
        serde_json::from_slice(&std::fs::read(directory.path().join("old.json")).unwrap()).unwrap();
    old["catalog"]["groups"].as_array_mut().unwrap().push(json!({
        "id": "group-alpha",
        "title": "Ambiguous duplicate",
        "controls": [{
            "id": "duplicate-group-control",
            "title": "Duplicate group control",
            "parts": [{
                "id": "duplicate_group_control_smt",
                "name": "statement",
                "prose": "A valid control under the duplicate group identifier."
            }]
        }]
    }));
    let old_hash = write_json(&directory.path().join("old.json"), &old);
    let new_hash =
        format!("{:x}", Sha256::digest(std::fs::read(directory.path().join("new.json")).unwrap()));
    write_json(&manifest, &impact_manifest(&old_hash, &new_hash));
    let ambiguous = run(&[
        "framework",
        "impact",
        "--manifest",
        manifest.to_str().unwrap(),
        "--group",
        "group-alpha",
    ]);
    assert_eq!(ambiguous.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&ambiguous.stderr).contains("duplicate group id"),
        "{}",
        String::from_utf8_lossy(&ambiguous.stderr)
    );

    let (directory, manifest) = setup_filter_fixture();
    let mapping_path = directory.path().join("mapping.json");
    let mut mapping: Value =
        serde_json::from_slice(&std::fs::read(&mapping_path).unwrap()).unwrap();
    mapping["mapping-collection"]["mappings"][0]["source-resource"]["href"] =
        json!("file:///private/secret-policy.json");
    write_json(&mapping_path, &mapping);
    let mut impact: Value = serde_json::from_slice(&std::fs::read(&manifest).unwrap()).unwrap();
    impact.as_object_mut().unwrap().remove("applicability_manifest");
    write_json(&manifest, &impact);
    let unsafe_href =
        run(&["framework", "impact", "--manifest", manifest.to_str().unwrap(), "--format", "json"]);
    assert_eq!(unsafe_href.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&unsafe_href.stderr).contains("absolute local path"));
    assert!(unsafe_href.stdout.is_empty());
}

#[test]
fn migration_group_filter_uses_sorted_old_and_new_group_union() {
    let directory = tempfile::tempdir().unwrap();
    let old_hash = write_json(
        &directory.path().join("old.json"),
        &catalog(FRAMEWORK_UUID, "1.0.0", &[("legacy", &[("old-id", "Old.")])]),
    );
    let new_hash = write_json(
        &directory.path().join("new.json"),
        &catalog(FRAMEWORK_UUID, "2.0.0", &[("current", &[("new-id", "New.")])]),
    );
    write_json(
        &directory.path().join("successors.json"),
        &json!({
            "schema_version": "forge.successor-map/1",
            "relationships": [{
                "relationship": "successor",
                "old_ids": ["old-id"],
                "new_ids": ["new-id"],
                "approved_by": "reviewer",
                "approved_at": "2026-08-25T12:00:00Z",
                "rationale": "Reviewed successor."
            }]
        }),
    );
    let mut impact = impact_manifest(&old_hash, &new_hash);
    impact["mapping_collections"] = json!([]);
    impact.as_object_mut().unwrap().remove("applicability_manifest");
    impact["successor_map"] = json!("successors.json");
    let manifest = directory.path().join("impact.json");
    write_json(&manifest, &impact);

    for group in ["legacy", "current"] {
        let (status, report) = filtered_report(&manifest, "--group", group);
        assert_eq!(status.status.code(), Some(1));
        assert_eq!(report["matched_findings"], 1);
        assert_eq!(report["findings"][0]["framework_groups"], json!(["current", "legacy"]));
    }
}
