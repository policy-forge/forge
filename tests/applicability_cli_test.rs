//! PRD056 framework applicability and policy-gap end-to-end contract tests.

use std::path::Path;
use std::process::{Command, Output};

use serde_json::{Value, json};

fn write_json(path: &Path, value: &Value) {
    std::fs::write(path, serde_json::to_vec_pretty(value).expect("serialize fixture"))
        .expect("write fixture");
}

fn catalog(uuid: &str, ids: &[&str]) -> Value {
    json!({
        "catalog": {
            "uuid": uuid,
            "metadata": {
                "title": "Synthetic redistributable catalog",
                "last-modified": "2026-08-25T08:00:00Z",
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

fn profile() -> Value {
    json!({
        "profile": {
            "uuid": "33333333-3333-4333-8333-333333333333",
            "metadata": {
                "title": "Synthetic framework profile",
                "last-modified": "2026-08-25T08:00:00Z",
                "version": "1.0.0",
                "oscal-version": "1.2.3"
            },
            "imports": [{
                "href": "resolved-catalog.json",
                "include-all": {}
            }]
        }
    })
}

fn mapping_manifest() -> Value {
    json!({
        "schema_version": "forge.mapping-manifest/1",
        "collection": {
            "key": "applicability-test-collection",
            "title": "Synthetic reviewed mapping",
            "version": "1.0.0",
            "last_modified": "2026-08-25T08:00:00Z"
        },
        "reviewers": [{"key": "mapper", "type": "person", "name": "Mapping Reviewer"}],
        "provenance": {
            "method": "human",
            "matching_rationale": "semantic",
            "status": "complete",
            "mapping_description": "Synthetic mapping participation fixture.",
            "reviewer_keys": ["mapper"],
            "reviewed_at": "2026-08-25T08:00:00Z"
        },
        "mapping": {
            "key": "applicability-test-mapping",
            "scope": "control-only",
            "source": {"type": "catalog", "artifact": "policy.json", "href": "policy.json"},
            "target": {"type": "catalog", "artifact": "framework.json", "href": "framework.json"},
            "maps": [
                {
                    "key": "positive-c1",
                    "relationship": "intersects-with",
                    "sources": [{"type": "control", "id_ref": "policy-1"}],
                    "targets": [{"type": "control", "id_ref": "c1"}],
                    "reviewer_key": "mapper",
                    "reviewed_at": "2026-08-25T08:00:00Z",
                    "rationale": "Human-reviewed positive relationship."
                },
                {
                    "key": "no-relationship-c2",
                    "relationship": "no-relationship",
                    "sources": [{"type": "control", "id_ref": "policy-2"}],
                    "targets": [
                        {"type": "control", "id_ref": "c1"},
                        {"type": "control", "id_ref": "c2"}
                    ],
                    "reviewer_key": "mapper",
                    "reviewed_at": "2026-08-25T08:00:00Z",
                    "rationale": "Human review found no relationship."
                }
            ]
        }
    })
}

fn run_in(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_forge"))
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("run forge")
}

fn build_mapping(dir: &Path, manifest_name: &str, output_name: &str, value: &Value) {
    let manifest_path = dir.join(manifest_name);
    let output_path = dir.join(output_name);
    write_json(&manifest_path, value);
    let result = run_in(
        dir,
        &[
            "mapping",
            "build",
            "--manifest",
            manifest_path.to_str().expect("manifest path"),
            "--output",
            output_path.to_str().expect("mapping path"),
        ],
    );
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
}

fn scaffold(dir: &Path) -> Value {
    let output = run_in(dir, &["applicability", "init", "--framework", "framework.json"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    serde_json::from_slice(&output.stdout).expect("parse scaffold")
}

fn setup() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    write_json(
        &dir.path().join("policy.json"),
        &catalog("11111111-1111-4111-8111-111111111111", &["policy-1", "policy-2"]),
    );
    write_json(
        &dir.path().join("framework.json"),
        &catalog("22222222-2222-4222-8222-222222222222", &["c1", "c2", "c3", "c4", "c5", "c6"]),
    );
    build_mapping(dir.path(), "mapping-manifest.json", "mapping.json", &mapping_manifest());
    dir
}

fn reviewed_decision(control_id: &str, state: &str) -> Value {
    json!({
        "control_id": control_id,
        "state": state,
        "reviewer_key": "scope-reviewer",
        "reviewed_at": "2026-08-25T09:00:00Z"
    })
}

fn analyzed_manifest(dir: &Path) -> Value {
    let mut manifest = scaffold(dir);
    manifest["reviewers"] = json!([{
        "key": "scope-reviewer",
        "type": "person",
        "name": "Scope Reviewer"
    }]);
    manifest["decisions"] = json!([
        reviewed_decision("c1", "applicable"),
        reviewed_decision("c2", "applicable"),
        reviewed_decision("c3", "applicable"),
        {
            "control_id": "c4",
            "state": "not-applicable",
            "reviewer_key": "scope-reviewer",
            "reviewed_at": "2026-08-25T09:00:00Z",
            "rationale": "Outside the reviewed organizational scope."
        },
        {
            "control_id": "c5",
            "state": "deferred",
            "reviewer_key": "scope-reviewer",
            "reviewed_at": "2026-08-25T09:00:00Z",
            "rationale": "Pending the named architecture decision.",
            "revisit_date": "2026-10-01"
        }
    ]);
    manifest["mapping_collections"] = json!(["mapping.json"]);
    manifest
}

#[test]
fn init_scaffolds_omitted_controls_as_under_review_with_exact_framework_evidence() {
    let dir = setup();
    let manifest = scaffold(dir.path());
    assert_eq!(manifest["schema_version"], "forge.applicability/1");
    assert_eq!(
        manifest["framework"]["inventory"]["control_ids"],
        json!(["c1", "c2", "c3", "c4", "c5", "c6"])
    );
    assert!(manifest["decisions"].as_array().expect("decisions").is_empty());
    assert_eq!(manifest["framework"]["expected_sha256"].as_str().expect("fingerprint").len(), 64);
}

#[test]
fn profile_init_requires_and_fingerprints_a_resolved_catalog_companion() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_json(&dir.path().join("profile.json"), &profile());
    write_json(
        &dir.path().join("resolved-catalog.json"),
        &catalog("44444444-4444-4444-8444-444444444444", &["profile-control"]),
    );
    let missing = run_in(dir.path(), &["applicability", "init", "--framework", "profile.json"]);
    assert_eq!(missing.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&missing.stderr).contains("--resolved-catalog is required"));

    let output = run_in(
        dir.path(),
        &[
            "applicability",
            "init",
            "--framework",
            "profile.json",
            "--resolved-catalog",
            "resolved-catalog.json",
        ],
    );
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let manifest: Value = serde_json::from_slice(&output.stdout).expect("profile scaffold");
    assert_eq!(manifest["framework"]["type"], "profile");
    assert_eq!(manifest["framework"]["resolved_catalog_attestation"], false);
    assert_eq!(manifest["framework"]["inventory"]["control_ids"], json!(["profile-control"]));
    assert_eq!(
        manifest["framework"]["inventory"]["root_uuid"],
        "33333333-3333-4333-8333-333333333333"
    );
}

#[test]
fn analyze_classifies_all_six_states_and_reconciles_deterministically() {
    let dir = setup();
    write_json(&dir.path().join("applicability.json"), &analyzed_manifest(dir.path()));
    let first = run_in(
        dir.path(),
        &["applicability", "analyze", "--manifest", "applicability.json", "--format", "json"],
    );
    let second = run_in(
        dir.path(),
        &["applicability", "analyze", "--manifest", "applicability.json", "--format", "json"],
    );
    assert!(first.status.success(), "{}", String::from_utf8_lossy(&first.stderr));
    assert_eq!(first.stdout, second.stdout, "identical inputs must produce identical bytes");
    insta::assert_snapshot!(
        "applicability_report_cross_platform",
        String::from_utf8(first.stdout.clone()).expect("UTF-8 report")
    );
    let report: Value = serde_json::from_slice(&first.stdout).expect("report JSON");
    assert_eq!(report["schema_version"], "forge.applicability-report/1");
    assert_eq!(report["counts"]["total"], 6);
    for label in [
        "applicable-mapped",
        "applicable-reviewed-no-relationship",
        "applicable-unmapped",
        "not-applicable",
        "deferred",
        "under-review",
    ] {
        assert_eq!(report["counts"][label], 1, "unexpected {label} count");
    }
    let controls = report["controls"].as_array().expect("controls");
    assert_eq!(controls[0]["control_id"], "c1");
    assert_eq!(controls[0]["classification"], "applicable-mapped");
    assert_eq!(controls[0]["no_relationship_count"], 1);
    assert_eq!(controls[1]["classification"], "applicable-reviewed-no-relationship");
    assert_eq!(controls[2]["classification"], "applicable-unmapped");
    assert_eq!(controls[5]["classification"], "under-review");
    assert_eq!(report["mapping_collections"].as_array().expect("mappings").len(), 1);
    let rendered = String::from_utf8(first.stdout).expect("UTF-8 report");
    assert!(!rendered.contains("non-compliant"));
    assert!(!rendered.contains("compliant"));
}

#[test]
fn selected_gate_returns_one_after_emitting_a_valid_report() {
    let dir = setup();
    write_json(&dir.path().join("applicability.json"), &analyzed_manifest(dir.path()));
    let output = run_in(
        dir.path(),
        &[
            "applicability",
            "analyze",
            "--manifest",
            "applicability.json",
            "--format",
            "json",
            "--fail-on",
            "applicable-unmapped",
        ],
    );
    assert_eq!(output.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&output.stdout).expect("valid report before gate");
    assert_eq!(report["counts"]["applicable-unmapped"], 1);
}

#[test]
fn stale_decisions_and_mismatched_mapping_resources_fail_before_output() {
    let dir = setup();
    let mut manifest = analyzed_manifest(dir.path());
    manifest["decisions"][0]["control_id"] = json!("removed-control");
    write_json(&dir.path().join("stale.json"), &manifest);
    let stale = run_in(
        dir.path(),
        &[
            "applicability",
            "analyze",
            "--manifest",
            "stale.json",
            "--output",
            "must-not-exist.json",
        ],
    );
    assert_eq!(stale.status.code(), Some(2));
    assert!(!dir.path().join("must-not-exist.json").exists());
    assert!(String::from_utf8_lossy(&stale.stderr).contains("does not resolve"));

    let mut mapping: Value =
        serde_json::from_slice(&std::fs::read(dir.path().join("mapping.json")).expect("mapping"))
            .expect("mapping JSON");
    let props = mapping["mapping-collection"]["mappings"][0]["target-resource"]["props"]
        .as_array_mut()
        .expect("props");
    props.iter_mut().find(|prop| prop["name"] == "raw-sha256").expect("hash prop")["value"] =
        json!("0".repeat(64));
    write_json(&dir.path().join("mapping.json"), &mapping);
    write_json(&dir.path().join("mismatch.json"), &analyzed_manifest(dir.path()));
    let mismatch = run_in(dir.path(), &["applicability", "analyze", "--manifest", "mismatch.json"]);
    assert_eq!(mismatch.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&mismatch.stderr).contains("does not match"));
}

#[test]
fn missing_exclusion_rationale_and_unknown_manifest_keys_are_rejected() {
    let dir = setup();
    let mut manifest = analyzed_manifest(dir.path());
    manifest["decisions"][3].as_object_mut().expect("decision").remove("rationale");
    write_json(&dir.path().join("missing-rationale.json"), &manifest);
    let missing =
        run_in(dir.path(), &["applicability", "analyze", "--manifest", "missing-rationale.json"]);
    assert_eq!(missing.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&missing.stderr).contains("rationale is required"));

    let mut unknown = analyzed_manifest(dir.path());
    unknown["automatic_scope"] = json!(true);
    write_json(&dir.path().join("unknown.json"), &unknown);
    let unknown_result =
        run_in(dir.path(), &["applicability", "analyze", "--manifest", "unknown.json"]);
    assert_eq!(unknown_result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&unknown_result.stderr).contains("unknown field"));
}

#[test]
fn duplicate_mapping_collection_uuids_are_rejected() {
    let dir = setup();
    std::fs::copy(dir.path().join("mapping.json"), dir.path().join("mapping-copy.json"))
        .expect("copy mapping");
    let mut manifest = analyzed_manifest(dir.path());
    manifest["mapping_collections"] = json!(["mapping.json", "mapping-copy.json"]);
    write_json(&dir.path().join("duplicates.json"), &manifest);
    let output = run_in(
        dir.path(),
        &[
            "applicability",
            "analyze",
            "--manifest",
            "duplicates.json",
            "--output",
            "must-not-exist.json",
        ],
    );
    assert_eq!(output.status.code(), Some(2));
    assert!(!dir.path().join("must-not-exist.json").exists());
    assert!(String::from_utf8_lossy(&output.stderr).contains("duplicates Mapping Collection UUID"));
}

#[test]
fn filters_preserve_complete_totals_and_filter_control_details_and_queue() {
    let dir = setup();
    write_json(&dir.path().join("applicability.json"), &analyzed_manifest(dir.path()));
    let cases = [
        (vec!["--group", "group-1"], 6),
        (vec!["--control-prefix", "c1"], 1),
        (vec!["--state", "deferred"], 1),
        (vec!["--reviewer", "scope-reviewer"], 5),
        (vec!["--policy-source", "policy.json"], 2),
    ];
    for (filter, expected) in cases {
        let mut args = vec![
            "applicability",
            "analyze",
            "--manifest",
            "applicability.json",
            "--format",
            "json",
        ];
        args.extend(filter);
        let output = run_in(dir.path(), &args);
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let report: Value = serde_json::from_slice(&output.stdout).expect("filtered report");
        assert_eq!(report["counts"]["total"], 6, "denominator changed for {args:?}");
        assert_eq!(report["matched_controls"], expected, "filter mismatch for {args:?}");
        assert_eq!(
            report["controls"].as_array().expect("controls").len(),
            expected,
            "detail mismatch for {args:?}"
        );
    }
}

#[test]
fn review_queue_has_stable_reason_owner_and_revisit_metadata() {
    let dir = setup();
    write_json(&dir.path().join("applicability.json"), &analyzed_manifest(dir.path()));
    let output = run_in(
        dir.path(),
        &["applicability", "analyze", "--manifest", "applicability.json", "--format", "json"],
    );
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let report: Value = serde_json::from_slice(&output.stdout).expect("report");
    let queue = report["review_queue"].as_array().expect("review queue");
    assert_eq!(queue.len(), 4);
    assert_eq!(queue[0]["control_id"], "c2");
    assert_eq!(queue[0]["reason_code"], "reviewed-no-positive-relationship");
    assert_eq!(queue[1]["reason_code"], "no-reviewed-mapping");
    assert_eq!(queue[2]["reason_code"], "deferred-scope-decision");
    assert_eq!(queue[2]["owner"], "scope-reviewer");
    assert_eq!(queue[2]["revisit_date"], "2026-10-01");
    assert_eq!(queue[3]["reason_code"], "scope-decision-required");
}

#[test]
fn overdue_deferred_gate_requires_an_explicit_deterministic_date() {
    let dir = setup();
    write_json(&dir.path().join("applicability.json"), &analyzed_manifest(dir.path()));
    let missing = run_in(
        dir.path(),
        &[
            "applicability",
            "analyze",
            "--manifest",
            "applicability.json",
            "--fail-on",
            "overdue-deferred",
        ],
    );
    assert_eq!(missing.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&missing.stderr).contains("--as-of is required"));

    let current = run_in(
        dir.path(),
        &[
            "applicability",
            "analyze",
            "--manifest",
            "applicability.json",
            "--fail-on",
            "overdue-deferred",
            "--as-of",
            "2026-10-01",
        ],
    );
    assert!(current.status.success(), "{}", String::from_utf8_lossy(&current.stderr));
    let overdue = run_in(
        dir.path(),
        &[
            "applicability",
            "analyze",
            "--manifest",
            "applicability.json",
            "--fail-on",
            "overdue-deferred",
            "--as-of",
            "2026-10-02",
        ],
    );
    assert_eq!(overdue.status.code(), Some(1));
    assert!(serde_json::from_slice::<Value>(&overdue.stdout).is_err());
    assert!(String::from_utf8_lossy(&overdue.stdout).contains("FORGE framework applicability"));
}

#[test]
fn html_report_is_static_deterministic_and_escapes_human_text() {
    let dir = setup();
    let mut manifest = analyzed_manifest(dir.path());
    manifest["reviewers"][0]["name"] = json!("Scope <Reviewer> & Team");
    manifest["decisions"].as_array_mut().expect("decisions").push(json!({
        "control_id": "c6",
        "state": "under-review",
        "reviewer_key": "scope-reviewer",
        "note": "Needs <legal> & security review."
    }));
    write_json(&dir.path().join("applicability.json"), &manifest);
    let args = ["applicability", "analyze", "--manifest", "applicability.json", "--format", "html"];
    let first = run_in(dir.path(), &args);
    let second = run_in(dir.path(), &args);
    assert!(first.status.success(), "{}", String::from_utf8_lossy(&first.stderr));
    assert_eq!(first.stdout, second.stdout);
    let html = String::from_utf8(first.stdout).expect("HTML UTF-8");
    assert!(html.starts_with("<!doctype html>\n"));
    assert!(html.contains("Scope &lt;Reviewer&gt; &amp; Team"));
    assert!(html.contains("2026-08-25T09:00:00Z"));
    assert!(html.contains("Outside the reviewed organizational scope."));
    assert!(html.contains("Needs &lt;legal&gt; &amp; security review."));
    assert!(!html.contains("<script"));
    assert!(!html.contains(dir.path().to_str().expect("temp path")));
}

#[test]
fn large_init_scaffold_can_be_analyzed_without_manual_compaction() {
    let dir = tempfile::tempdir().expect("tempdir");
    let ids = (0..30_000)
        .map(|index| format!("control-{index:05}-with-a-deliberately-long-stable-identifier"))
        .collect::<Vec<_>>();
    let id_refs = ids.iter().map(String::as_str).collect::<Vec<_>>();
    write_json(
        &dir.path().join("large-framework.json"),
        &catalog("77777777-7777-4777-8777-777777777777", &id_refs),
    );

    let init =
        run_in(dir.path(), &["applicability", "init", "--framework", "large-framework.json"]);
    assert!(init.status.success(), "{}", String::from_utf8_lossy(&init.stderr));
    assert!(init.stdout.len() > 2 * 1024 * 1024, "fixture must exceed the former limit");
    let scaffold: Value = serde_json::from_slice(&init.stdout).expect("large scaffold");
    assert!(scaffold["decisions"].as_array().expect("decisions").is_empty());
    std::fs::write(dir.path().join("large-applicability.json"), &init.stdout)
        .expect("write large scaffold");

    let analyze = run_in(
        dir.path(),
        &[
            "applicability",
            "analyze",
            "--manifest",
            "large-applicability.json",
            "--format",
            "json",
            "--control-prefix",
            "no-such-control",
        ],
    );
    assert!(analyze.status.success(), "{}", String::from_utf8_lossy(&analyze.stderr));
    let report: Value = serde_json::from_slice(&analyze.stdout).expect("large report");
    assert_eq!(report["counts"]["total"], 30_000);
    assert_eq!(report["counts"]["under-review"], 30_000);
    assert_eq!(report["matched_controls"], 0);
}

#[test]
fn duplicate_framework_group_ids_are_rejected_as_filter_ambiguous() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut framework = catalog("88888888-8888-4888-8888-888888888888", &["first-control"]);
    framework["catalog"]["groups"].as_array_mut().expect("groups").push(json!({
        "id": "group-1",
        "title": "Duplicate group identifier",
        "controls": [{"id": "second-control", "title": "Second control"}]
    }));
    write_json(&dir.path().join("duplicate-groups.json"), &framework);

    let output =
        run_in(dir.path(), &["applicability", "init", "--framework", "duplicate-groups.json"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("duplicate group id 'group-1'"));
}

#[test]
fn statement_only_maps_do_not_roll_up_to_parent_control_classifications() {
    let dir = setup();
    let mut statement_mapping = mapping_manifest();
    statement_mapping["mapping"]["scope"] = json!("control-plus-statement");
    statement_mapping["mapping"]["maps"] = json!([{
        "key": "statement-only",
        "relationship": "intersects-with",
        "sources": [{"type": "statement", "id_ref": "policy-1_smt"}],
        "targets": [{"type": "statement", "id_ref": "c1_smt"}],
        "reviewer_key": "mapper",
        "reviewed_at": "2026-08-25T08:00:00Z",
        "rationale": "The statements overlap without asserting parent-control coverage."
    }]);
    build_mapping(
        dir.path(),
        "statement-mapping-manifest.json",
        "statement-mapping.json",
        &statement_mapping,
    );
    let mut manifest = analyzed_manifest(dir.path());
    manifest["mapping_collections"] = json!(["statement-mapping.json"]);
    write_json(&dir.path().join("statement-only.json"), &manifest);

    let output = run_in(
        dir.path(),
        &["applicability", "analyze", "--manifest", "statement-only.json", "--format", "json"],
    );
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let report: Value = serde_json::from_slice(&output.stdout).expect("report");
    assert_eq!(report["counts"]["applicable-mapped"], 0);
    assert_eq!(report["counts"]["applicable-unmapped"], 3);
    assert_eq!(report["controls"][0]["positive_mapping_count"], 0);
}

#[test]
fn whitespace_padded_filters_and_missing_output_directories_are_rejected_clearly() {
    let dir = setup();
    write_json(&dir.path().join("applicability.json"), &analyzed_manifest(dir.path()));
    let filter = run_in(
        dir.path(),
        &["applicability", "analyze", "--manifest", "applicability.json", "--group", " group-1"],
    );
    assert_eq!(filter.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&filter.stderr).contains("leading or trailing whitespace"));

    let output = run_in(
        dir.path(),
        &[
            "applicability",
            "init",
            "--framework",
            "framework.json",
            "--output",
            "missing/applicability.json",
        ],
    );
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("output directory 'missing' does not exist")
    );
}

#[test]
fn report_preserves_framework_mapping_and_reviewer_provenance() {
    let dir = setup();
    write_json(&dir.path().join("applicability.json"), &analyzed_manifest(dir.path()));
    let output = run_in(
        dir.path(),
        &["applicability", "analyze", "--manifest", "applicability.json", "--format", "json"],
    );
    assert!(output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).expect("report");
    assert_eq!(report["framework"]["root_uuid"], "22222222-2222-4222-8222-222222222222");
    assert_eq!(report["framework"]["document_version"], "1.0.0");
    assert_eq!(report["framework"]["oscal_version"], "1.2.3");
    assert_eq!(report["manifest_sha256"].as_str().expect("manifest hash").len(), 64);
    let mapping = &report["mapping_collections"][0];
    assert_eq!(mapping["version"], "1.0.0");
    assert_eq!(mapping["oscal_version"], "1.2.3");
    assert_eq!(mapping["reviewed_at"], "2026-08-25T08:00:00Z");
    assert_eq!(mapping["reviewers"][0]["name"], "Mapping Reviewer");
    assert_eq!(mapping["reviewers"][0]["type"], "person");
    assert_eq!(mapping["source_resources"][0]["href"], "policy.json");
    assert_eq!(mapping["source_resources"][0]["root_uuid"], "11111111-1111-4111-8111-111111111111");
    assert_eq!(mapping["source_resources"][0]["document_version"], "1.0.0");
    assert_eq!(mapping["source_resources"][0]["oscal_version"], "1.2.3");
    assert_eq!(
        mapping["source_resources"][0]["raw_sha256"].as_str().expect("source hash").len(),
        64
    );
}

#[test]
fn multiple_mapping_collections_aggregate_without_changing_the_denominator() {
    let dir = setup();
    let mut second = mapping_manifest();
    second["collection"]["key"] = json!("second-collection");
    second["mapping"]["key"] = json!("second-mapping");
    second["mapping"]["maps"] = json!([{
        "key": "positive-c3",
        "relationship": "intersects-with",
        "sources": [{"type": "control", "id_ref": "policy-1"}],
        "targets": [{"type": "control", "id_ref": "c3"}],
        "reviewer_key": "mapper",
        "reviewed_at": "2026-08-25T08:00:00Z",
        "rationale": "Second reviewed relationship."
    }]);
    build_mapping(dir.path(), "mapping-2-manifest.json", "mapping-2.json", &second);
    let mut manifest = analyzed_manifest(dir.path());
    manifest["mapping_collections"] = json!(["mapping.json", "mapping-2.json"]);
    write_json(&dir.path().join("applicability.json"), &manifest);
    let output = run_in(
        dir.path(),
        &["applicability", "analyze", "--manifest", "applicability.json", "--format", "json"],
    );
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let report: Value = serde_json::from_slice(&output.stdout).expect("report");
    assert_eq!(report["counts"]["total"], 6);
    assert_eq!(report["counts"]["applicable-mapped"], 2);
    assert_eq!(report["counts"]["applicable-unmapped"], 0);
    assert_eq!(report["mapping_collections"].as_array().expect("mappings").len(), 2);
}

#[test]
fn contradictory_policy_source_fingerprints_are_rejected() {
    let dir = setup();
    let mut second = mapping_manifest();
    second["collection"]["key"] = json!("conflicting-source-collection");
    second["mapping"]["key"] = json!("conflicting-source-mapping");
    second["mapping"]["maps"] = json!([{
        "key": "conflicting-source-c3",
        "relationship": "intersects-with",
        "sources": [{"type": "control", "id_ref": "policy-1"}],
        "targets": [{"type": "control", "id_ref": "c3"}],
        "reviewer_key": "mapper",
        "reviewed_at": "2026-08-25T08:00:00Z",
        "rationale": "Second reviewed relationship."
    }]);
    build_mapping(dir.path(), "mapping-2-manifest.json", "mapping-2.json", &second);

    let mapping_path = dir.path().join("mapping-2.json");
    let mut mapping: Value =
        serde_json::from_slice(&std::fs::read(&mapping_path).expect("mapping")).expect("JSON");
    let source_props = mapping["mapping-collection"]["mappings"][0]["source-resource"]["props"]
        .as_array_mut()
        .expect("source props");
    source_props.iter_mut().find(|prop| prop["name"] == "raw-sha256").expect("source hash")["value"] =
        json!("0".repeat(64));
    write_json(&mapping_path, &mapping);

    let mut manifest = analyzed_manifest(dir.path());
    manifest["mapping_collections"] = json!(["mapping.json", "mapping-2.json"]);
    write_json(&dir.path().join("source-conflict.json"), &manifest);
    let output =
        run_in(dir.path(), &["applicability", "analyze", "--manifest", "source-conflict.json"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("contradicts another resource fingerprint")
    );
}

#[test]
fn contradictory_relationships_for_the_same_policy_subject_are_rejected() {
    let dir = setup();
    let mapping_path = dir.path().join("mapping.json");
    let mut mapping: Value =
        serde_json::from_slice(&std::fs::read(&mapping_path).expect("mapping")).expect("JSON");
    let positive_source =
        mapping["mapping-collection"]["mappings"][0]["maps"][0]["sources"][0].clone();
    mapping["mapping-collection"]["mappings"][0]["maps"][1]["sources"][0] = positive_source;
    write_json(&mapping_path, &mapping);
    write_json(&dir.path().join("relationship-conflict.json"), &analyzed_manifest(dir.path()));

    let output = run_in(
        dir.path(),
        &["applicability", "analyze", "--manifest", "relationship-conflict.json"],
    );
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("contradicts another reviewed relationship")
    );
}

#[test]
fn duplicate_inner_mapping_uuids_and_unstable_collection_uuids_are_rejected() {
    let dir = setup();
    let mut second = mapping_manifest();
    second["collection"]["key"] = json!("different-collection-same-mapping");
    build_mapping(dir.path(), "mapping-2-manifest.json", "mapping-2.json", &second);
    let mut manifest = analyzed_manifest(dir.path());
    manifest["mapping_collections"] = json!(["mapping.json", "mapping-2.json"]);
    write_json(&dir.path().join("duplicate-inner.json"), &manifest);
    let duplicate =
        run_in(dir.path(), &["applicability", "analyze", "--manifest", "duplicate-inner.json"]);
    assert_eq!(duplicate.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&duplicate.stderr).contains("duplicates mapping UUID"));

    let mapping_path = dir.path().join("mapping.json");
    let mut mapping: Value =
        serde_json::from_slice(&std::fs::read(&mapping_path).expect("mapping")).expect("JSON");
    mapping["mapping-collection"]["uuid"] = json!("bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb");
    write_json(&mapping_path, &mapping);
    let mut single = analyzed_manifest(dir.path());
    single["mapping_collections"] = json!(["mapping.json"]);
    write_json(&dir.path().join("unstable-uuid.json"), &single);
    let unstable =
        run_in(dir.path(), &["applicability", "analyze", "--manifest", "unstable-uuid.json"]);
    assert_eq!(unstable.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&unstable.stderr)
            .contains("does not match the deterministic FORGE collection identity")
    );
}

#[test]
fn map_reviewer_keys_must_resolve_to_declared_parties() {
    let dir = setup();
    let mapping_path = dir.path().join("mapping.json");
    let mut mapping: Value =
        serde_json::from_slice(&std::fs::read(&mapping_path).expect("mapping")).expect("JSON");
    let props = mapping["mapping-collection"]["mappings"][0]["maps"][0]["props"]
        .as_array_mut()
        .expect("map props");
    props.iter_mut().find(|prop| prop["name"] == "reviewer-key").expect("reviewer key")["value"] =
        json!("ghost-reviewer");
    write_json(&mapping_path, &mapping);
    write_json(&dir.path().join("unknown-map-reviewer.json"), &analyzed_manifest(dir.path()));

    let output = run_in(
        dir.path(),
        &["applicability", "analyze", "--manifest", "unknown-map-reviewer.json"],
    );
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("references undeclared reviewer"));
}

#[test]
fn every_invalid_decision_evidence_shape_fails_before_output() {
    let dir = setup();
    let mut cases = Vec::new();

    let mut missing_reviewer = analyzed_manifest(dir.path());
    missing_reviewer["decisions"][0].as_object_mut().expect("decision").remove("reviewer_key");
    cases.push(("missing-reviewer", missing_reviewer, "reviewer_key is required"));

    let mut unknown_reviewer = analyzed_manifest(dir.path());
    unknown_reviewer["decisions"][0]["reviewer_key"] = json!("unknown-reviewer");
    cases.push(("unknown-reviewer", unknown_reviewer, "references unknown reviewer"));

    let mut missing_time = analyzed_manifest(dir.path());
    missing_time["decisions"][0].as_object_mut().expect("decision").remove("reviewed_at");
    cases.push(("missing-time", missing_time, "reviewed_at is required"));

    let mut invalid_time = analyzed_manifest(dir.path());
    invalid_time["decisions"][0]["reviewed_at"] = json!("yesterday");
    cases.push(("invalid-time", invalid_time, "RFC 3339"));

    let mut missing_revisit = analyzed_manifest(dir.path());
    missing_revisit["decisions"][4].as_object_mut().expect("decision").remove("revisit_date");
    cases.push(("missing-revisit", missing_revisit, "revisit_date is required"));

    let mut invalid_revisit = analyzed_manifest(dir.path());
    invalid_revisit["decisions"][4]["revisit_date"] = json!("October 1");
    cases.push(("invalid-revisit", invalid_revisit, "YYYY-MM-DD"));

    let mut unknown_state = analyzed_manifest(dir.path());
    unknown_state["decisions"][0]["state"] = json!("automatically-applicable");
    cases.push(("unknown-state", unknown_state, "unknown variant"));

    for (name, manifest, expected_error) in cases {
        let manifest_name = format!("{name}.json");
        let output_name = format!("{name}-output.json");
        write_json(&dir.path().join(&manifest_name), &manifest);
        let output = run_in(
            dir.path(),
            &["applicability", "analyze", "--manifest", &manifest_name, "--output", &output_name],
        );
        assert_eq!(output.status.code(), Some(2), "case {name}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(expected_error),
            "case {name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!dir.path().join(output_name).exists(), "case {name} wrote output");
    }
}

#[test]
fn duplicate_and_conflicting_control_decisions_are_rejected() {
    let dir = setup();
    let mut manifest = analyzed_manifest(dir.path());
    let conflicting = json!({
        "control_id": "c1",
        "state": "not-applicable",
        "reviewer_key": "scope-reviewer",
        "reviewed_at": "2026-08-25T10:00:00Z",
        "rationale": "Conflicting duplicate state."
    });
    manifest["decisions"].as_array_mut().expect("decisions").push(conflicting);
    write_json(&dir.path().join("conflict.json"), &manifest);
    let output = run_in(dir.path(), &["applicability", "analyze", "--manifest", "conflict.json"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("duplicates control decision 'c1'"));
}

#[test]
fn duplicate_and_type_ambiguous_framework_ids_are_rejected() {
    let dir = tempfile::tempdir().expect("tempdir");
    let duplicate = catalog("55555555-5555-4555-8555-555555555555", &["duplicate", "duplicate"]);
    write_json(&dir.path().join("duplicate.json"), &duplicate);
    let duplicate_result =
        run_in(dir.path(), &["applicability", "init", "--framework", "duplicate.json"]);
    assert_eq!(duplicate_result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&duplicate_result.stderr).contains("duplicate eligible id"));

    let mut ambiguous = catalog("66666666-6666-4666-8666-666666666666", &["ambiguous"]);
    ambiguous["catalog"]["groups"][0]["controls"][0]["parts"][0]["id"] = json!("ambiguous");
    write_json(&dir.path().join("ambiguous.json"), &ambiguous);
    let ambiguous_result =
        run_in(dir.path(), &["applicability", "init", "--framework", "ambiguous.json"]);
    assert_eq!(ambiguous_result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&ambiguous_result.stderr).contains("type-ambiguous"));
}

#[test]
fn absolute_local_hrefs_are_rejected_from_framework_and_mapping_reports() {
    let dir = setup();
    let mut framework_href = analyzed_manifest(dir.path());
    framework_href["framework"]["href"] = json!("/private/sensitive/framework.json");
    write_json(&dir.path().join("absolute-framework.json"), &framework_href);
    let framework_result =
        run_in(dir.path(), &["applicability", "analyze", "--manifest", "absolute-framework.json"]);
    assert_eq!(framework_result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&framework_result.stderr).contains("absolute local path"));

    let mut mapping: Value =
        serde_json::from_slice(&std::fs::read(dir.path().join("mapping.json")).expect("mapping"))
            .expect("mapping JSON");
    mapping["mapping-collection"]["mappings"][0]["source-resource"]["href"] =
        json!("/private/sensitive/policy.json");
    write_json(&dir.path().join("mapping.json"), &mapping);
    write_json(&dir.path().join("absolute-mapping.json"), &analyzed_manifest(dir.path()));
    let mapping_result =
        run_in(dir.path(), &["applicability", "analyze", "--manifest", "absolute-mapping.json"]);
    assert_eq!(mapping_result.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&mapping_result.stderr).contains("absolute local path"),
        "{}",
        String::from_utf8_lossy(&mapping_result.stderr)
    );
}

#[test]
fn stale_subject_fingerprints_and_invalid_mapping_schema_are_rejected() {
    let dir = setup();
    let mut mapping: Value =
        serde_json::from_slice(&std::fs::read(dir.path().join("mapping.json")).expect("mapping"))
            .expect("mapping JSON");
    let target_props =
        mapping["mapping-collection"]["mappings"][0]["maps"][0]["targets"][0]["props"]
            .as_array_mut()
            .expect("target props");
    target_props
        .iter_mut()
        .find(|prop| prop["name"] == "subject-sha256")
        .expect("subject fingerprint")["value"] = json!("0".repeat(64));
    write_json(&dir.path().join("mapping.json"), &mapping);
    write_json(&dir.path().join("stale-fingerprint.json"), &analyzed_manifest(dir.path()));
    let stale =
        run_in(dir.path(), &["applicability", "analyze", "--manifest", "stale-fingerprint.json"]);
    assert_eq!(stale.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&stale.stderr).contains("subject-sha256' is stale"));

    let mut invalid = mapping;
    invalid["mapping-collection"]["metadata"].as_object_mut().expect("metadata").remove("version");
    write_json(&dir.path().join("mapping.json"), &invalid);
    let invalid_result =
        run_in(dir.path(), &["applicability", "analyze", "--manifest", "stale-fingerprint.json"]);
    assert_eq!(invalid_result.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&invalid_result.stderr).contains("not a valid mapping-collection")
    );
}

#[test]
fn output_alias_is_rejected_without_modifying_the_manifest() {
    let dir = setup();
    let manifest_path = dir.path().join("applicability.json");
    write_json(&manifest_path, &analyzed_manifest(dir.path()));
    let original = std::fs::read(&manifest_path).expect("manifest bytes");
    let result = run_in(
        dir.path(),
        &[
            "applicability",
            "analyze",
            "--manifest",
            "applicability.json",
            "--output",
            "applicability.json",
        ],
    );
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("aliases an applicability input"));
    assert_eq!(std::fs::read(manifest_path).expect("preserved manifest"), original);
}

#[cfg(unix)]
#[test]
fn hard_link_output_alias_is_rejected() {
    let dir = setup();
    let manifest_path = dir.path().join("applicability.json");
    let alias_path = dir.path().join("hard-link-output.json");
    write_json(&manifest_path, &analyzed_manifest(dir.path()));
    std::fs::hard_link(&manifest_path, &alias_path).expect("hard link");
    let result = run_in(
        dir.path(),
        &[
            "applicability",
            "analyze",
            "--manifest",
            "applicability.json",
            "--output",
            "hard-link-output.json",
        ],
    );
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("aliases an applicability input"));
}

#[test]
fn profile_companion_analysis_accepts_exact_profile_target_mapping() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_json(
        &dir.path().join("policy.json"),
        &catalog("77777777-7777-4777-8777-777777777777", &["policy-1"]),
    );
    write_json(&dir.path().join("profile.json"), &profile());
    write_json(
        &dir.path().join("resolved-catalog.json"),
        &catalog("88888888-8888-4888-8888-888888888888", &["profile-control"]),
    );
    let mut mapping = mapping_manifest();
    mapping["mapping"]["target"] = json!({
        "type": "profile",
        "artifact": "profile.json",
        "href": "profile.json",
        "resolved_catalog": "resolved-catalog.json",
        "resolved_catalog_attestation": true
    });
    mapping["mapping"]["maps"][0]["targets"] =
        json!([{"type": "control", "id_ref": "profile-control"}]);
    mapping["mapping"]["maps"].as_array_mut().expect("maps").truncate(1);
    build_mapping(dir.path(), "profile-mapping-manifest.json", "profile-mapping.json", &mapping);

    let scaffold_output = run_in(
        dir.path(),
        &[
            "applicability",
            "init",
            "--framework",
            "profile.json",
            "--resolved-catalog",
            "resolved-catalog.json",
        ],
    );
    assert!(scaffold_output.status.success());
    let mut manifest: Value = serde_json::from_slice(&scaffold_output.stdout).expect("scaffold");
    manifest["framework"]["resolved_catalog_attestation"] = json!(true);
    manifest["reviewers"] = json!([{
        "key": "scope-reviewer",
        "type": "person",
        "name": "Scope Reviewer"
    }]);
    manifest["decisions"] = json!([reviewed_decision("profile-control", "applicable")]);
    manifest["mapping_collections"] = json!(["profile-mapping.json"]);
    write_json(&dir.path().join("profile-applicability.json"), &manifest);
    let analyzed = run_in(
        dir.path(),
        &[
            "applicability",
            "analyze",
            "--manifest",
            "profile-applicability.json",
            "--format",
            "json",
        ],
    );
    assert!(analyzed.status.success(), "{}", String::from_utf8_lossy(&analyzed.stderr));
    let report: Value = serde_json::from_slice(&analyzed.stdout).expect("report");
    assert_eq!(report["framework"]["resource_type"], "profile");
    assert_eq!(report["counts"]["applicable-mapped"], 1);
    assert_eq!(
        report["framework"]["resolved_catalog_sha256"].as_str().expect("companion hash").len(),
        64
    );
}

#[test]
fn generated_report_vocabulary_avoids_assurance_claims_in_all_formats() {
    let dir = setup();
    write_json(&dir.path().join("applicability.json"), &analyzed_manifest(dir.path()));
    for format in ["text", "json", "html"] {
        let output = run_in(
            dir.path(),
            &["applicability", "analyze", "--manifest", "applicability.json", "--format", format],
        );
        assert!(output.status.success(), "{format}");
        let rendered = String::from_utf8(output.stdout).expect("UTF-8").to_lowercase();
        for prohibited in
            ["compliant", "non-compliant", "effectiveness", "implemented", "certification"]
        {
            assert!(!rendered.contains(prohibited), "{format} contains {prohibited}");
        }
    }
}

#[test]
fn hundred_control_acceptance_fixture_reconciles_sixty_applicable_and_forty_mapped() {
    let dir = tempfile::tempdir().expect("tempdir");
    let framework_ids: Vec<_> = (1..=100).map(|index| format!("c{index:03}")).collect();
    let framework_refs: Vec<_> = framework_ids.iter().map(String::as_str).collect();
    write_json(
        &dir.path().join("policy.json"),
        &catalog("99999999-9999-4999-8999-999999999999", &["policy-1"]),
    );
    write_json(
        &dir.path().join("framework.json"),
        &catalog("aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa", &framework_refs),
    );
    let mut mapping = mapping_manifest();
    mapping["mapping"]["maps"] = json!([{
        "key": "forty-reviewed-positive-relationships",
        "relationship": "intersects-with",
        "sources": [{"type": "control", "id_ref": "policy-1"}],
        "targets": framework_ids[..40]
            .iter()
            .map(|id| json!({"type": "control", "id_ref": id}))
            .collect::<Vec<_>>(),
        "reviewer_key": "mapper",
        "reviewed_at": "2026-08-25T08:00:00Z",
        "rationale": "Forty explicit reviewed positive relationships."
    }]);
    build_mapping(dir.path(), "mapping-manifest.json", "mapping.json", &mapping);

    let mut manifest = scaffold(dir.path());
    manifest["reviewers"] = json!([{
        "key": "scope-reviewer",
        "type": "person",
        "name": "Scope Reviewer"
    }]);
    let mut decisions = Vec::new();
    decisions.extend(framework_ids[..60].iter().map(|id| reviewed_decision(id, "applicable")));
    decisions.extend(framework_ids[60..70].iter().map(|id| {
        json!({
            "control_id": id,
            "state": "not-applicable",
            "reviewer_key": "scope-reviewer",
            "reviewed_at": "2026-08-25T09:00:00Z",
            "rationale": "Explicit reviewed exclusion."
        })
    }));
    decisions.extend(framework_ids[70..75].iter().map(|id| {
        json!({
            "control_id": id,
            "state": "deferred",
            "reviewer_key": "scope-reviewer",
            "reviewed_at": "2026-08-25T09:00:00Z",
            "rationale": "Explicit reviewed deferral.",
            "revisit_date": "2026-12-01"
        })
    }));
    manifest["decisions"] = Value::Array(decisions);
    manifest["mapping_collections"] = json!(["mapping.json"]);
    write_json(&dir.path().join("applicability.json"), &manifest);
    let output = run_in(
        dir.path(),
        &["applicability", "analyze", "--manifest", "applicability.json", "--format", "json"],
    );
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let report: Value = serde_json::from_slice(&output.stdout).expect("report");
    assert_eq!(report["counts"]["total"], 100);
    assert_eq!(report["counts"]["applicable-mapped"], 40);
    assert_eq!(report["counts"]["applicable-unmapped"], 20);
    assert_eq!(report["counts"]["not-applicable"], 10);
    assert_eq!(report["counts"]["deferred"], 5);
    assert_eq!(report["counts"]["under-review"], 25);
    assert!(!output.stdout.windows(2).any(|window| window == b"\r\n"));
}
