//! PRD055 Control Mapping end-to-end contract tests.

use std::path::{Path, PathBuf};
use std::process::Command;

use sha2::{Digest, Sha256};

use serde_json::{Value, json};
use tempfile::TempDir;

fn write_json(path: &Path, value: &Value) {
    std::fs::write(path, serde_json::to_vec_pretty(value).expect("serialize fixture"))
        .expect("write fixture");
}

fn sha256_file(path: &Path) -> String {
    format!("{:x}", Sha256::digest(std::fs::read(path).expect("read fixture")))
}

fn catalog(uuid: &str, ids: &[&str]) -> Value {
    json!({
        "catalog": {
            "uuid": uuid,
            "metadata": {
                "title": "Synthetic redistributable catalog",
                "last-modified": "2026-08-22T17:00:00Z",
                "version": "1.0.0",
                "oscal-version": "1.2.3"
            },
            "groups": [{
                "id": "group-1",
                "title": "Synthetic controls",
                "controls": ids.iter().map(|id| json!({
                    "id": id,
                    "title": format!("Control {id}"),
                    "parts": [{
                        "id": format!("{id}_smt"),
                        "name": "statement",
                        "prose": format!("Synthetic statement for {id}.")
                    }]
                })).collect::<Vec<_>>()
            }]
        }
    })
}

fn manifest(source_id: &str, target_ids: &[&str]) -> Value {
    json!({
        "schema_version": "forge.mapping-manifest/1",
        "collection": {
            "key": "synthetic-collection",
            "title": "Synthetic reviewed mapping",
            "version": "1.0.0",
            "last_modified": "2026-08-22T17:00:00Z"
        },
        "reviewers": [{"key": "reviewer-1", "type": "person", "name": "Test Reviewer"}],
        "provenance": {
            "method": "human",
            "matching_rationale": "semantic",
            "status": "draft",
            "mapping_description": "Human-reviewed synthetic relationship set.",
            "reviewer_keys": ["reviewer-1"],
            "reviewed_at": "2026-08-22T17:00:00Z"
        },
        "mapping": {
            "key": "synthetic-mapping",
            "source": {"type": "catalog", "artifact": "source.json", "href": "source.json"},
            "target": {"type": "catalog", "artifact": "target.json", "href": "target.json"},
            "maps": [{
                "key": "reviewed-edge",
                "matching_rationale": "semantic",
                "relationship": "subset-of",
                "sources": [{"type": "control", "id_ref": source_id}],
                "targets": target_ids.iter().map(|id| json!({"type": "control", "id_ref": id})).collect::<Vec<_>>(),
                "reviewer_key": "reviewer-1",
                "reviewed_at": "2026-08-22T17:00:00Z",
                "rationale": "The source is narrower than the reviewed target set.",
                "confidence_score": {"category": "medium"}
            }]
        }
    })
}

fn setup() -> (TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    write_json(
        &dir.path().join("source.json"),
        &catalog("11111111-1111-4111-8111-111111111111", &["source-1"]),
    );
    write_json(
        &dir.path().join("target.json"),
        &catalog("22222222-2222-4222-8222-222222222222", &["target-1", "target-2", "target-3"]),
    );
    let manifest_path = dir.path().join("manifest.json");
    write_json(&manifest_path, &manifest("source-1", &["target-1"]));
    (dir, manifest_path)
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_forge")).args(args).output().expect("run forge")
}

fn run_in(cwd: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_forge"))
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("run forge")
}

#[test]
fn build_emits_schema_valid_mapping_and_scoped_participation_report() {
    let (dir, manifest_path) = setup();
    let output_path = dir.path().join("mapping.json");
    let report_path = dir.path().join("report.json");
    let result = run(&[
        "mapping",
        "build",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--output",
        output_path.to_str().unwrap(),
        "--report",
        report_path.to_str().unwrap(),
        "--report-format",
        "json",
    ]);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));

    let artifact: Value = serde_json::from_slice(&std::fs::read(output_path).unwrap()).unwrap();
    let validation = forge::validate::validate_artifact(&artifact, forge::OscalModelType::Mapping)
        .expect("validate mapping");
    assert!(validation.is_valid, "{:#?}", validation.errors);
    let map = &artifact["mapping-collection"]["mappings"][0]["maps"][0];
    assert_eq!(map["relationship"], "subset-of");
    assert_eq!(map["sources"].as_array().unwrap().len(), 1);
    assert_eq!(map["targets"].as_array().unwrap().len(), 1);
    assert!(!artifact.to_string().contains("no-relationship"));

    let report: Value = serde_json::from_slice(&std::fs::read(report_path).unwrap()).unwrap();
    assert_eq!(report["target_controls"]["referenced"], 1);
    assert_eq!(report["target_controls"]["eligible"], 3);
    assert_eq!(report["target_controls"]["unmapped_ids"], json!(["target-2", "target-3"]));
    assert_eq!(report["validation"]["mapping_schema_valid"], true);
}

#[test]
fn build_accepts_schema_valid_groups_without_ids() {
    let (dir, manifest_path) = setup();
    let source_path = dir.path().join("source.json");
    let mut source: Value =
        serde_json::from_slice(&std::fs::read(&source_path).expect("source")).expect("JSON");
    source["catalog"]["groups"][0].as_object_mut().expect("group").remove("id");
    write_json(&source_path, &source);
    let output_path = dir.path().join("mapping.json");

    let result = run(&[
        "mapping",
        "build",
        "--manifest",
        manifest_path.to_str().expect("manifest path"),
        "--output",
        output_path.to_str().expect("output path"),
    ]);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    assert!(output_path.exists());
}

#[test]
fn text_report_includes_resource_validation_and_author_estimate_evidence() {
    let (dir, manifest_path) = setup();
    let report_path = dir.path().join("report.txt");
    let result = run(&[
        "mapping",
        "build",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--report",
        report_path.to_str().unwrap(),
    ]);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let report = std::fs::read_to_string(report_path).expect("read text report");
    assert!(report.contains("source resource: type=catalog"));
    assert!(report.contains("raw-sha256="));
    assert!(
        report.contains(
            "validation: manifest=true resources=true references=true mapping-schema=true"
        )
    );
    assert!(report.contains("author estimate: map-key=reviewed-edge"));
    assert!(report.contains("label=reviewer_estimate_not_compliance_coverage"));
    assert!(report.contains("confidence=medium"));
}

#[test]
fn init_rebases_resource_paths_to_the_output_manifest_directory() {
    let (dir, _manifest_path) = setup();
    let scaffold_dir = dir.path().join("nested").join("manifests");
    std::fs::create_dir_all(&scaffold_dir).expect("create scaffold directory");
    let output_path = scaffold_dir.join("mapping-manifest.json");
    let source_path = dir.path().join("source.json");
    let target_path = dir.path().join("target.json");
    let result = run(&[
        "mapping",
        "init",
        "--source",
        source_path.to_str().unwrap(),
        "--target",
        target_path.to_str().unwrap(),
        "--output",
        output_path.to_str().unwrap(),
    ]);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let scaffold: Value =
        serde_json::from_slice(&std::fs::read(&output_path).expect("read scaffold"))
            .expect("parse scaffold");
    for (side, expected) in [("source", source_path), ("target", target_path)] {
        let relative = scaffold["mapping"][side]["artifact"].as_str().expect("artifact path");
        assert!(!Path::new(relative).is_absolute());
        assert_eq!(
            output_path.parent().unwrap().join(relative).canonicalize().unwrap(),
            expected.canonicalize().unwrap()
        );
    }
}

#[test]
fn missing_reference_fails_without_writing_output() {
    let (dir, manifest_path) = setup();
    write_json(&manifest_path, &manifest("missing-source", &["target-1"]));
    let output_path = dir.path().join("mapping.json");
    let result = run(&[
        "mapping",
        "build",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--output",
        output_path.to_str().unwrap(),
    ]);
    assert_eq!(result.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("$.mapping.maps[0].sources[0]"), "{stderr}");
    assert!(stderr.contains("missing-source"), "{stderr}");
    assert!(!output_path.exists());
}

#[test]
fn identical_inputs_and_manifest_produce_identical_bytes() {
    let (dir, manifest_path) = setup();
    let first = dir.path().join("first.json");
    let second = dir.path().join("second.json");
    for output in [&first, &second] {
        let result = run(&[
            "mapping",
            "build",
            "--manifest",
            manifest_path.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
        ]);
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    }
    assert_eq!(std::fs::read(first).unwrap(), std::fs::read(second).unwrap());
}

#[test]
fn duplicate_manifest_key_and_output_alias_are_rejected() {
    let (_dir, manifest_path) = setup();
    std::fs::write(
        &manifest_path,
        r#"{"schema_version":"forge.mapping-manifest/1","schema_version":"forge.mapping-manifest/1"}"#,
    )
    .unwrap();
    let duplicate = run(&["mapping", "build", "--manifest", manifest_path.to_str().unwrap()]);
    assert_eq!(duplicate.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&duplicate.stderr).contains("duplicate object key"));

    write_json(&manifest_path, &manifest("source-1", &["target-1"]));
    let alias = run(&[
        "mapping",
        "build",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--output",
        manifest_path.to_str().unwrap(),
    ]);
    assert_eq!(alias.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&alias.stderr).contains("aliases a mapping input"));
}

#[test]
fn hard_link_output_alias_is_rejected_without_modifying_the_input() {
    let (dir, manifest_path) = setup();
    let source_path = dir.path().join("source.json");
    let source_before = std::fs::read(&source_path).expect("read source fixture");
    let output_alias = dir.path().join("output.json");
    std::fs::hard_link(&source_path, &output_alias).expect("create hard-link alias");
    let result = run(&[
        "mapping",
        "build",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--output",
        output_alias.to_str().unwrap(),
    ]);
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("aliases a mapping input"));
    assert_eq!(std::fs::read(source_path).unwrap(), source_before);
}

#[test]
fn baseline_check_reports_subject_change_and_new_gap_with_exit_1() {
    let (dir, manifest_path) = setup();
    let baseline_path = dir.path().join("baseline.json");
    let build = run(&[
        "mapping",
        "build",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--output",
        baseline_path.to_str().unwrap(),
    ]);
    assert!(build.status.success(), "{}", String::from_utf8_lossy(&build.stderr));

    let mut changed = catalog(
        "22222222-2222-4222-8222-222222222222",
        &["target-1", "target-2", "target-3", "target-4"],
    );
    changed["catalog"]["groups"][0]["controls"][0]["parts"][0]["prose"] =
        json!("Substantively revised synthetic statement.");
    write_json(&dir.path().join("target.json"), &changed);

    let check = run(&[
        "mapping",
        "check",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--baseline",
        baseline_path.to_str().unwrap(),
        "--report-format",
        "json",
        "--fail-on",
        "any",
    ]);
    assert_eq!(check.status.code(), Some(1), "{}", String::from_utf8_lossy(&check.stderr));
    let report: Value = serde_json::from_slice(&check.stdout).expect("JSON impact report");
    let codes: Vec<_> = report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|finding| finding["code"].as_str())
        .collect();
    assert!(codes.contains(&"subject_changed"), "{codes:?}");
    assert!(codes.contains(&"new_gap"), "{codes:?}");
    let subject_change = report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|finding| finding["code"] == "subject_changed")
        .expect("subject change finding");
    let old_fingerprint = subject_change["old_fingerprint"].as_str().expect("old fingerprint");
    let new_fingerprint = subject_change["new_fingerprint"].as_str().expect("new fingerprint");
    assert_eq!(old_fingerprint.len(), 64);
    assert_eq!(new_fingerprint.len(), 64);
    assert_ne!(old_fingerprint, new_fingerprint);
}

#[test]
fn baseline_check_reports_changed_map_review_evidence() {
    let (dir, manifest_path) = setup();
    let baseline_path = dir.path().join("baseline.json");
    let build = run(&[
        "mapping",
        "build",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--output",
        baseline_path.to_str().unwrap(),
    ]);
    assert!(build.status.success(), "{}", String::from_utf8_lossy(&build.stderr));

    let mut revised = manifest("source-1", &["target-1"]);
    revised["mapping"]["maps"][0]["reviewed_at"] = json!("2026-08-23T17:00:00Z");
    write_json(&manifest_path, &revised);
    let check = run(&[
        "mapping",
        "check",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--baseline",
        baseline_path.to_str().unwrap(),
        "--report-format",
        "json",
    ]);
    assert_eq!(check.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&check.stdout).expect("JSON impact report");
    assert!(
        report["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| { finding["code"] == "map_review_evidence_changed" })
    );
}

#[test]
fn invalid_baseline_is_incomplete_analysis_exit_2() {
    let (dir, manifest_path) = setup();
    let baseline_path = dir.path().join("baseline.json");
    write_json(
        &baseline_path,
        &json!({
            "catalog": {
                "uuid": "44444444-4444-4444-8444-444444444444",
                "metadata": {
                    "title": "Not a mapping",
                    "last-modified": "2026-08-22T17:00:00Z",
                    "version": "1.0.0",
                    "oscal-version": "1.2.3"
                }
            }
        }),
    );
    let check = run(&[
        "mapping",
        "check",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--baseline",
        baseline_path.to_str().unwrap(),
    ]);
    assert_eq!(check.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&check.stderr).contains("must be an OSCAL Control Mapping"));
}

#[test]
fn baseline_with_duplicate_map_uuid_is_rejected() {
    let (dir, manifest_path) = setup();
    let baseline_path = dir.path().join("baseline.json");
    let build = run(&[
        "mapping",
        "build",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--output",
        baseline_path.to_str().unwrap(),
    ]);
    assert!(build.status.success(), "{}", String::from_utf8_lossy(&build.stderr));

    let mut baseline: Value =
        serde_json::from_slice(&std::fs::read(&baseline_path).unwrap()).unwrap();
    let maps =
        baseline["mapping-collection"]["mappings"][0]["maps"].as_array_mut().expect("maps array");
    let mut duplicate = maps[0].clone();
    duplicate["relationship"] = json!("equivalent-to");
    maps.push(duplicate);
    write_json(&baseline_path, &baseline);

    let check = run(&[
        "mapping",
        "check",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--baseline",
        baseline_path.to_str().unwrap(),
    ]);
    assert_eq!(check.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&check.stderr).contains("duplicate map UUID"));
}

#[test]
fn public_build_rejects_unknown_provenance_reviewer() {
    let (dir, _manifest_path) = setup();
    let mut manifest: forge::mapping::manifest::MappingManifest =
        serde_json::from_value(manifest("source-1", &["target-1"])).unwrap();
    manifest.provenance.reviewer_keys = vec!["unknown-reviewer".to_string()];
    let source =
        forge::mapping::inventory::load(dir.path(), "$.mapping.source", &manifest.mapping.source)
            .unwrap();
    let target =
        forge::mapping::inventory::load(dir.path(), "$.mapping.target", &manifest.mapping.target)
            .unwrap();

    let Err(error) = forge::mapping::model::build(&manifest, &source, &target, false) else {
        panic!("unknown reviewer must return an error");
    };
    assert!(error.to_string().contains("references unknown reviewer"));
}

#[test]
fn profile_requires_and_records_resolved_catalog_companion() {
    let (dir, manifest_path) = setup();
    write_json(
        &dir.path().join("target-profile.json"),
        &json!({
            "profile": {
                "uuid": "55555555-5555-4555-8555-555555555555",
                "metadata": {
                    "title": "Synthetic profile",
                    "last-modified": "2026-08-22T17:00:00Z",
                    "version": "1.0.0",
                    "oscal-version": "1.2.3"
                },
                "imports": [{"href": "target.json", "include-all": {}}]
            }
        }),
    );
    let mut profile_manifest = manifest("source-1", &["target-1"]);
    profile_manifest["mapping"]["target"] = json!({
        "type": "profile",
        "artifact": "target-profile.json",
        "resolved_catalog": "target.json",
        "expected_resolved_catalog_sha256": sha256_file(&dir.path().join("target.json")),
        "resolved_catalog_attestation": true,
        "href": "target-profile.json"
    });
    write_json(&manifest_path, &profile_manifest);
    let output_path = dir.path().join("mapping.json");
    let result = run(&[
        "mapping",
        "build",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--output",
        output_path.to_str().unwrap(),
    ]);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let artifact: Value = serde_json::from_slice(&std::fs::read(output_path).unwrap()).unwrap();
    let target = &artifact["mapping-collection"]["mappings"][0]["target-resource"];
    assert_eq!(target["type"], "profile");
    assert!(target["props"].as_array().unwrap().iter().any(|prop| {
        prop["name"] == "resolved-catalog-sha256"
            && prop["value"].as_str().is_some_and(|value| value.len() == 64)
    }));
}

#[test]
fn baseline_check_rejects_resolved_catalog_companion_hash_changes() {
    let (dir, manifest_path) = setup();
    let profile_path = dir.path().join("target-profile.json");
    write_json(
        &profile_path,
        &json!({
            "profile": {
                "uuid": "55555555-5555-4555-8555-555555555555",
                "metadata": {
                    "title": "Synthetic profile",
                    "last-modified": "2026-08-22T17:00:00Z",
                    "version": "1.0.0",
                    "oscal-version": "1.2.3"
                },
                "imports": [{"href": "target.json", "include-all": {}}]
            }
        }),
    );
    let mut profile_manifest = manifest("source-1", &["target-1"]);
    profile_manifest["mapping"]["target"] = json!({
        "type": "profile",
        "artifact": "target-profile.json",
        "resolved_catalog": "target.json",
        "resolved_catalog_attestation": true,
        "expected_resolved_catalog_sha256": sha256_file(&dir.path().join("target.json")),
        "href": "target-profile.json"
    });
    write_json(&manifest_path, &profile_manifest);
    let baseline_path = dir.path().join("baseline.json");
    let build = run(&[
        "mapping",
        "build",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--output",
        baseline_path.to_str().unwrap(),
    ]);
    assert!(build.status.success(), "{}", String::from_utf8_lossy(&build.stderr));

    let mut changed_companion =
        catalog("22222222-2222-4222-8222-222222222222", &["target-1", "target-2", "target-3"]);
    changed_companion["catalog"]["metadata"]["title"] = json!("Revised resolved Catalog metadata");
    write_json(&dir.path().join("target.json"), &changed_companion);
    let check = run(&[
        "mapping",
        "check",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--baseline",
        baseline_path.to_str().unwrap(),
        "--report-format",
        "json",
    ]);
    assert_eq!(check.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&check.stderr).contains("expected_resolved_catalog_sha256 mismatch"),
        "{}",
        String::from_utf8_lossy(&check.stderr)
    );
}

#[test]
fn profile_without_companion_and_guidance_as_statement_are_rejected() {
    let (dir, manifest_path) = setup();
    let mut missing_companion = manifest("source-1", &["target-1"]);
    missing_companion["mapping"]["target"] = json!({
        "type": "profile",
        "artifact": "target.json",
        "href": "target.json"
    });
    write_json(&manifest_path, &missing_companion);
    let result = run(&["mapping", "build", "--manifest", manifest_path.to_str().unwrap()]);
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("run 'forge resolve' explicitly"));

    let mut target = catalog("22222222-2222-4222-8222-222222222222", &["target-1"]);
    target["catalog"]["groups"][0]["controls"][0]["parts"][0] = json!({
        "id": "guidance-1",
        "name": "guidance",
        "prose": "Synthetic guidance."
    });
    write_json(&dir.path().join("target.json"), &target);
    let mut wrong_type = manifest("source-1", &["target-1"]);
    wrong_type["mapping"]["maps"][0]["targets"] =
        json!([{"type": "statement", "id_ref": "guidance-1"}]);
    write_json(&manifest_path, &wrong_type);
    let result = run(&["mapping", "build", "--manifest", manifest_path.to_str().unwrap()]);
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("ineligible part type 'guidance'"));
}

#[test]
fn different_working_directories_do_not_change_output_or_leak_paths() {
    let (first_dir, first_manifest) = setup();
    let (second_dir, second_manifest) = setup();
    let first_output = first_dir.path().join("mapping.json");
    let second_output = second_dir.path().join("mapping.json");
    for (manifest, output) in [(&first_manifest, &first_output), (&second_manifest, &second_output)]
    {
        let result = run_in(
            manifest.parent().unwrap(),
            &[
                "mapping",
                "build",
                "--manifest",
                manifest.to_str().unwrap(),
                "--output",
                output.to_str().unwrap(),
            ],
        );
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    }
    let first = std::fs::read_to_string(first_output).unwrap();
    let second = std::fs::read_to_string(second_output).unwrap();
    assert_eq!(first, second);
    assert!(!first.contains(first_dir.path().to_str().unwrap()));
    assert!(!second.contains(second_dir.path().to_str().unwrap()));
}

#[test]
fn invalid_confidence_and_unknown_manifest_key_are_rejected() {
    let (_dir, manifest_path) = setup();
    let mut invalid = manifest("source-1", &["target-1"]);
    invalid["mapping"]["maps"][0]["confidence_score"] = json!({"percentage": 1.1});
    write_json(&manifest_path, &invalid);
    let confidence = run(&["mapping", "build", "--manifest", manifest_path.to_str().unwrap()]);
    assert_eq!(confidence.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&confidence.stderr).contains("between 0 and 1"));

    invalid["mapping"]["maps"][0]["confidence_score"] = json!({"category": "medium"});
    invalid["unexpected"] = json!(true);
    write_json(&manifest_path, &invalid);
    let unknown = run(&["mapping", "build", "--manifest", manifest_path.to_str().unwrap()]);
    assert_eq!(unknown.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&unknown.stderr).contains("unknown field `unexpected`"));
}

#[test]
fn init_emits_deterministic_unapproved_inventory_scaffold() {
    let (dir, _manifest_path) = setup();
    let first = dir.path().join("scaffold-1.json");
    let second = dir.path().join("scaffold-2.json");
    for output in [&first, &second] {
        let result = run(&[
            "mapping",
            "init",
            "--source",
            dir.path().join("source.json").to_str().unwrap(),
            "--target",
            dir.path().join("target.json").to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
        ]);
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    }
    let first_bytes = std::fs::read(&first).unwrap();
    assert_eq!(first_bytes, std::fs::read(second).unwrap());
    let scaffold: Value = serde_json::from_slice(&first_bytes).unwrap();
    assert_eq!(scaffold["mapping"]["maps"], json!([]));
    assert_eq!(scaffold["reviewers"], json!([]));
    assert_eq!(
        scaffold["mapping"]["target"]["inventory"]["control_ids"],
        json!(["target-1", "target-2", "target-3"])
    );
    assert!(
        scaffold["mapping"]["source"]["expected_sha256"]
            .as_str()
            .is_some_and(|value| value.len() == 64)
    );
    assert!(!String::from_utf8_lossy(&first_bytes).contains(dir.path().to_str().unwrap()));
}

#[test]
fn explicit_scope_qualifiers_estimates_and_opt_in_excerpts_are_preserved() {
    let (dir, manifest_path) = setup();
    let mut value = manifest("source-1", &["target-1"]);
    value["mapping"]["scope"] = json!("control-only");
    value["mapping"]["maps"][0]["coverage"] =
        json!({"generation_method": "arbitrary", "target_coverage": 0.5});
    value["mapping"]["maps"][0]["qualifiers"] = json!([{
        "subject": "target",
        "predicate": "has-requirement",
        "category": "addressable",
        "description": "A reviewer must assess the remaining target requirement."
    }]);
    write_json(&manifest_path, &value);
    let output_path = dir.path().join("mapping.json");
    let report_path = dir.path().join("report.txt");
    let result = run(&[
        "mapping",
        "build",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--output",
        output_path.to_str().unwrap(),
        "--report",
        report_path.to_str().unwrap(),
        "--include-excerpts",
    ]);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let artifact: Value = serde_json::from_slice(&std::fs::read(output_path).unwrap()).unwrap();
    let map = &artifact["mapping-collection"]["mappings"][0]["maps"][0];
    assert_eq!(map["coverage"]["generation-method"], "arbitrary");
    assert_eq!(map["coverage"]["target-coverage"], 0.5);
    assert_eq!(map["qualifiers"][0]["category"], "addressable");
    let report = std::fs::read_to_string(report_path).unwrap();
    assert!(report.contains("source statements review participation: 0/0"));
    assert!(report.contains("subject excerpts (sensitive; explicitly requested)"));
    assert!(report.contains("Control source-1"));
}

#[test]
fn control_only_scope_rejects_statement_relationships() {
    let (_dir, manifest_path) = setup();
    let mut value = manifest("source-1", &["target-1"]);
    value["mapping"]["scope"] = json!("control-only");
    value["mapping"]["maps"][0]["sources"] =
        json!([{"type": "statement", "id_ref": "source-1_smt"}]);
    write_json(&manifest_path, &value);
    let result = run(&["mapping", "build", "--manifest", manifest_path.to_str().unwrap()]);
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("outside control-only scope"));
}

#[test]
fn grouped_direction_and_low_confidence_are_preserved_without_policy_effects() {
    let (dir, manifest_path) = setup();
    let mut value = manifest("source-1", &["target-3", "target-1", "target-2"]);
    value["mapping"]["maps"][0]["confidence_score"] = json!({"category": "low"});
    write_json(&manifest_path, &value);
    let output_path = dir.path().join("mapping.json");
    let report_path = dir.path().join("report.json");
    let result = run(&[
        "mapping",
        "build",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--output",
        output_path.to_str().unwrap(),
        "--report",
        report_path.to_str().unwrap(),
        "--report-format",
        "json",
    ]);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let artifact: Value = serde_json::from_slice(&std::fs::read(output_path).unwrap()).unwrap();
    let map = &artifact["mapping-collection"]["mappings"][0]["maps"][0];
    assert_eq!(map["relationship"], "subset-of");
    assert_eq!(map["sources"][0]["id-ref"], "source-1");
    assert_eq!(
        Value::Array(
            map["targets"].as_array().unwrap().iter().map(|item| item["id-ref"].clone()).collect()
        ),
        json!(["target-1", "target-2", "target-3"])
    );
    assert_eq!(map["confidence-score"]["category"], "low");
    let report: Value = serde_json::from_slice(&std::fs::read(report_path).unwrap()).unwrap();
    assert_eq!(report["target_controls"]["unmapped_ids"], json!([]));
    assert_eq!(report["author_estimates"][0]["label"], "reviewer_estimate_not_compliance_coverage");
}

#[test]
fn extension_hash_and_duplicate_inventory_fail_before_output() {
    let (dir, manifest_path) = setup();
    let output_path = dir.path().join("mapping.json");
    let mut value = manifest("source-1", &["target-1"]);
    value["mapping"]["target"]["artifact"] = json!("target.xml");
    write_json(&manifest_path, &value);
    let extension = run(&[
        "mapping",
        "build",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--output",
        output_path.to_str().unwrap(),
    ]);
    assert_eq!(extension.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&extension.stderr).contains("local .json file"));
    assert!(!output_path.exists());

    value = manifest("source-1", &["target-1"]);
    value["mapping"]["target"]["expected_sha256"] = json!("0".repeat(64));
    write_json(&manifest_path, &value);
    let hash = run(&["mapping", "build", "--manifest", manifest_path.to_str().unwrap()]);
    assert_eq!(hash.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&hash.stderr).contains("expected_sha256 mismatch"));

    let mut duplicated = catalog("22222222-2222-4222-8222-222222222222", &["target-1"]);
    let duplicate_control = duplicated["catalog"]["groups"][0]["controls"][0].clone();
    duplicated["catalog"]["groups"][0]["controls"].as_array_mut().unwrap().push(duplicate_control);
    write_json(&dir.path().join("target.json"), &duplicated);
    write_json(&manifest_path, &manifest("source-1", &["target-1"]));
    let duplicate = run(&["mapping", "build", "--manifest", manifest_path.to_str().unwrap()]);
    assert_eq!(duplicate.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&duplicate.stderr).contains("duplicate eligible id 'target-1'")
    );
}

#[test]
fn baseline_removed_subject_is_reported_as_stale_without_guessing_successor() {
    let (dir, manifest_path) = setup();
    let baseline_path = dir.path().join("baseline.json");
    let build = run(&[
        "mapping",
        "build",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--output",
        baseline_path.to_str().unwrap(),
    ]);
    assert!(build.status.success(), "{}", String::from_utf8_lossy(&build.stderr));

    write_json(
        &dir.path().join("target.json"),
        &catalog("22222222-2222-4222-8222-222222222222", &["target-2", "target-3"]),
    );
    let mut revised = manifest("source-1", &["target-2"]);
    revised["mapping"]["maps"][0]["key"] = json!("replacement-reviewed-edge");
    write_json(&manifest_path, &revised);
    let check = run(&[
        "mapping",
        "check",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--baseline",
        baseline_path.to_str().unwrap(),
        "--report-format",
        "json",
    ]);
    assert_eq!(check.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&check.stdout).unwrap();
    let stale = report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|finding| finding["code"] == "stale_reference")
        .expect("stale reference finding");
    assert!(stale["message"].as_str().unwrap().contains("target-1"));
    assert!(!report.to_string().contains("successor"));
}

#[test]
fn schema_valid_non_forge_baseline_is_rejected_for_missing_stable_evidence() {
    let (dir, manifest_path) = setup();
    let baseline_path = dir.path().join("baseline.json");
    write_json(
        &baseline_path,
        &json!({
            "mapping-collection": {
                "uuid": "11111111-1111-4111-8111-111111111111",
                "metadata": {
                    "title": "External mapping",
                    "last-modified": "2026-08-22T17:00:00Z",
                    "version": "1.0.0",
                    "oscal-version": "1.2.3"
                },
                "provenance": {
                    "method": "human",
                    "matching-rationale": "semantic",
                    "status": "draft",
                    "mapping-description": "No FORGE stable evidence."
                },
                "mappings": [{
                    "uuid": "22222222-2222-4222-8222-222222222222",
                    "source-resource": {"type": "catalog", "href": "source.json"},
                    "target-resource": {"type": "catalog", "href": "target.json"},
                    "maps": [{
                        "uuid": "33333333-3333-4333-8333-333333333333",
                        "relationship": "equal-to",
                        "sources": [{"type": "control", "id-ref": "source-1"}],
                        "targets": [{"type": "control", "id-ref": "target-1"}]
                    }]
                }]
            }
        }),
    );
    let check = run(&[
        "mapping",
        "check",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--baseline",
        baseline_path.to_str().unwrap(),
    ]);
    assert_eq!(check.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&check.stderr);
    assert!(stderr.contains("lacks required FORGE property"), "{stderr}");
}

#[test]
fn identical_local_resources_are_allowed_and_https_href_is_only_preserved() {
    let (dir, manifest_path) = setup();
    let mut value = manifest("source-1", &["source-1"]);
    value["mapping"]["target"] = json!({
        "type": "catalog",
        "artifact": "source.json",
        "href": "https://example.invalid/framework/catalog.json"
    });
    write_json(&manifest_path, &value);
    let output_path = dir.path().join("mapping.json");
    let result = run(&[
        "mapping",
        "build",
        "--manifest",
        manifest_path.to_str().unwrap(),
        "--output",
        output_path.to_str().unwrap(),
    ]);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let artifact: Value = serde_json::from_slice(&std::fs::read(output_path).unwrap()).unwrap();
    assert_eq!(
        artifact["mapping-collection"]["mappings"][0]["target-resource"]["href"],
        "https://example.invalid/framework/catalog.json"
    );
}

#[test]
fn all_standard_relationships_and_grouped_cardinalities_validate() {
    let (dir, manifest_path) = setup();
    write_json(
        &dir.path().join("source.json"),
        &catalog("11111111-1111-4111-8111-111111111111", &["source-1", "source-2", "source-3"]),
    );
    let cases = [
        ("equal-to", &["source-1"][..], &["target-1"][..]),
        ("equivalent-to", &["source-1"][..], &["target-1", "target-2"][..]),
        ("subset-of", &["source-1", "source-2"][..], &["target-1"][..]),
        ("superset-of", &["source-1", "source-2"][..], &["target-1", "target-2"][..]),
        ("intersects-with", &["source-2"][..], &["target-2"][..]),
        ("no-relationship", &["source-3"][..], &["target-3"][..]),
    ];
    for (index, (relationship, sources, targets)) in cases.iter().enumerate() {
        let mut value = manifest(sources[0], targets);
        value["mapping"]["maps"][0]["relationship"] = json!(relationship);
        value["mapping"]["maps"][0]["sources"] = Value::Array(
            sources.iter().map(|id| json!({"type": "control", "id_ref": id})).collect(),
        );
        write_json(&manifest_path, &value);
        let output_path = dir.path().join(format!("case-{index}.json"));
        let result = run(&[
            "mapping",
            "build",
            "--manifest",
            manifest_path.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
        ]);
        assert!(
            result.status.success(),
            "{relationship}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        let artifact: Value = serde_json::from_slice(&std::fs::read(output_path).unwrap()).unwrap();
        let validation =
            forge::validate::validate_artifact(&artifact, forge::OscalModelType::Mapping).unwrap();
        assert!(validation.is_valid, "{relationship}: {:#?}", validation.errors);
    }

    let mut invalid = manifest("source-1", &["target-1"]);
    invalid["mapping"]["maps"][0]["relationship"] = json!("subset_of");
    write_json(&manifest_path, &invalid);
    let result = run(&["mapping", "build", "--manifest", manifest_path.to_str().unwrap()]);
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("unknown variant `subset_of`"));
}
