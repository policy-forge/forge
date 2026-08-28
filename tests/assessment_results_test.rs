//! End-to-end coverage for the PRD 063 Assessment Results workflow.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use chrono::{TimeZone as _, Utc};
use forge::oscal::catalog::OscalMetadata as CatalogMetadata;
use forge::oscal::parts::OscalPartName;
use forge::oscal::{
    CatalogEnvelope, OscalCatalog, OscalControl, OscalPart, ProfileRoot, SelectionMode,
    SspComponentInput, build_assessment_plan, build_profile, build_ssp_skeleton,
};
use serde_json::{Value, json};
use sha2::{Digest as _, Sha256};
use tempfile::TempDir;
use uuid::Uuid;

const CONTROL_ID: &str = "AC-1";
const STATEMENT_ID: &str = "AC-1_smt";
const OBJECTIVE_ID: &str = "AC-1_obj";
const TASK_UUID: &str = "77777777-7777-4777-8777-777777777777";
const INVENTORY_UUID: &str = "33333333-3333-4333-8333-333333333333";
const LOCATION_UUID: &str = "44444444-4444-4444-8444-444444444444";
const PARTY_UUID: &str = "55555555-5555-4555-8555-555555555555";
const EVIDENCE_HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

struct Fixture {
    directory: TempDir,
    manifest: PathBuf,
    output: PathBuf,
    report: PathBuf,
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_forge")).args(args).output().expect("run forge")
}

fn write_json(path: &Path, value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec_pretty(value).expect("serialize fixture");
    bytes.push(b'\n');
    std::fs::write(path, &bytes).expect("write fixture");
    bytes
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn artifact(path: &str, href: &str, bytes: &[u8], value: &Value, root: &str) -> Value {
    let document = &value[root];
    json!({
        "artifact": path,
        "href": href,
        "expected_sha256": sha256(bytes),
        "root_uuid": document["uuid"],
        "document_version": document["metadata"]["version"],
        "oscal_version": document["metadata"]["oscal-version"]
    })
}

fn mutate_assessment_plan(fixture: &Fixture, mutate: impl FnOnce(&mut Value)) {
    let root = fixture.manifest.parent().unwrap();
    let assessment_plan_path = root.join("assessment-plan.json");
    let mut assessment_plan: Value =
        serde_json::from_slice(&std::fs::read(&assessment_plan_path).unwrap()).unwrap();
    mutate(&mut assessment_plan);
    let assessment_plan_bytes = write_json(&assessment_plan_path, &assessment_plan);
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(&fixture.manifest).unwrap()).unwrap();
    manifest["context"]["assessment_plan"]["expected_sha256"] =
        json!(sha256(&assessment_plan_bytes));
    write_json(&fixture.manifest, &manifest);
}

#[allow(
    clippy::too_many_lines,
    reason = "one self-contained fixture exposes the complete supported OSCAL reference surface"
)]
fn fixture() -> Fixture {
    let directory = TempDir::new().expect("tempdir");
    let root = directory.path().to_path_buf();

    let catalog = OscalCatalog {
        uuid: Uuid::parse_str("11111111-1111-4111-8111-111111111111").unwrap().to_string(),
        metadata: CatalogMetadata {
            title: "Assessment fixture catalog".to_string(),
            last_modified: "2026-01-01T00:00:00Z".to_string(),
            version: "1.0.0".to_string(),
            oscal_version: "1.2.3".to_string(),
        },
        controls: vec![OscalControl {
            id: CONTROL_ID.to_string(),
            uuid: String::new(),
            title: "Access control".to_string(),
            links: Vec::new(),
            params: Vec::new(),
            parts: vec![
                OscalPart {
                    id: STATEMENT_ID.to_string(),
                    name: OscalPartName::Statement,
                    prose: "Access is explicitly authorized.".to_string(),
                    parts: Vec::new(),
                    props: Vec::new(),
                },
                OscalPart {
                    id: OBJECTIVE_ID.to_string(),
                    name: OscalPartName::Objective,
                    prose: "Verify that access is explicitly authorized.".to_string(),
                    parts: Vec::new(),
                    props: Vec::new(),
                },
            ],
            props: Vec::new(),
        }],
        groups: Vec::new(),
        back_matter: None,
    };
    let catalog_value = serde_json::to_value(CatalogEnvelope { catalog: catalog.clone() }).unwrap();
    let catalog_bytes = write_json(&root.join("catalog.json"), &catalog_value);

    let profile = build_profile(
        "catalog.json",
        vec![CONTROL_ID.to_string()],
        SelectionMode::Include,
        &[],
        Some(Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap()),
    )
    .expect("build profile");
    let profile_value = serde_json::to_value(ProfileRoot { profile }).unwrap();
    let profile_bytes = write_json(&root.join("profile.json"), &profile_value);

    let ssp = build_ssp_skeleton(
        "Assessment fixture",
        "1.0.0",
        &catalog,
        &[SspComponentInput {
            title: "Fixture service".to_string(),
            description: "Fixture component boundary".to_string(),
            component_type: forge::oscal::ssp::ComponentType::Software,
        }],
        "profile.json",
    )
    .expect("build SSP");
    let mut ssp_value = serde_json::to_value(ssp).unwrap();
    ssp_value["system-security-plan"]["metadata"]["locations"] =
        json!([{"uuid": LOCATION_UUID, "title": "Fixture location"}]);
    ssp_value["system-security-plan"]["metadata"]["parties"] =
        json!([{"uuid": PARTY_UUID, "type": "organization", "name": "Fixture organization"}]);
    ssp_value["system-security-plan"]["system-implementation"]["inventory-items"] =
        json!([{"uuid": INVENTORY_UUID, "description": "Fixture inventory item"}]);
    let ssp_bytes = write_json(&root.join("ssp.json"), &ssp_value);
    let component_uuid = ssp_value
        .pointer("/system-security-plan/system-implementation/components/0/uuid")
        .and_then(Value::as_str)
        .unwrap()
        .to_string();
    let implementation_uuid = ssp_value
        .pointer("/system-security-plan/control-implementation/implemented-requirements/0/uuid")
        .and_then(Value::as_str)
        .unwrap()
        .to_string();
    let user_uuid = ssp_value
        .pointer("/system-security-plan/system-implementation/users/0/uuid")
        .and_then(Value::as_str)
        .unwrap()
        .to_string();
    let resource_uuid = ssp_value
        .pointer("/system-security-plan/back-matter/resources/0/uuid")
        .and_then(Value::as_str)
        .unwrap()
        .to_string();

    let mut assessment_plan = serde_json::to_value(
        build_assessment_plan(&[CONTROL_ID.to_string()], "ssp.json", "Assessment fixture")
            .expect("build assessment plan"),
    )
    .unwrap();
    assessment_plan["assessment-plan"]["assessment-subjects"] = json!([{
        "type": "component",
        "include-subjects": [{"subject-uuid": component_uuid, "type": "component"}]
    }, {
        "type": "inventory-item",
        "include-subjects": [{"subject-uuid": INVENTORY_UUID, "type": "inventory-item"}]
    }, {
        "type": "location",
        "include-subjects": [{"subject-uuid": LOCATION_UUID, "type": "location"}]
    }, {
        "type": "party",
        "include-subjects": [{"subject-uuid": PARTY_UUID, "type": "party"}]
    }, {
        "type": "user",
        "include-subjects": [{"subject-uuid": user_uuid, "type": "user"}]
    }, {
        "type": "resource",
        "include-subjects": [{"subject-uuid": resource_uuid, "type": "resource"}]
    }]);
    assessment_plan["assessment-plan"]["reviewed-controls"]["control-objective-selections"] =
        json!([{"include-objectives": [{"objective-id": OBJECTIVE_ID}]}]);
    assessment_plan["assessment-plan"]["tasks"] = json!([{
        "uuid": TASK_UUID,
        "type": "action",
        "title": "Review fixture evidence",
        "description": "Human review activity."
    }]);
    let assessment_plan_bytes = write_json(&root.join("assessment-plan.json"), &assessment_plan);

    let evidence_value = json!({
        "schema_version": "forge.linkage-index/1",
        "records": [{
            "key": "evidence-1",
            "sha256": EVIDENCE_HASH,
            "content": "HIGHLY SENSITIVE EVIDENCE EXCERPT"
        }]
    });
    let evidence_bytes = write_json(&root.join("evidence-index.json"), &evidence_value);

    let manifest_value = json!({
        "schema_version": "forge.assessment-results/1",
        "document": {
            "key": "assessment-2026-q1",
            "title": "Q1 reviewed assessment results",
            "version": "1.0.0",
            "last_modified": "2026-01-02T00:00:00Z"
        },
        "context": {
            "assessment_plan": artifact(
                "assessment-plan.json", "assessment-plan.json", &assessment_plan_bytes,
                &assessment_plan, "assessment-plan"
            ),
            "ssp": artifact("ssp.json", "ssp.json", &ssp_bytes, &ssp_value, "system-security-plan"),
            "profile": artifact(
                "profile.json", "profile.json", &profile_bytes, &profile_value, "profile"
            ),
            "catalog": artifact(
                "catalog.json", "catalog.json", &catalog_bytes, &catalog_value, "catalog"
            ),
            "evidence_index": {
                "artifact": "evidence-index.json",
                "expected_sha256": sha256(&evidence_bytes)
            }
        },
        "roles": [{"id": "assessor", "title": "Assessor"}],
        "parties": [{"key": "assessor-alice", "type": "person", "name": "Alice Assessor"}],
        "result": {
            "key": "epoch-2026-q1",
            "title": "Q1 review epoch",
            "description": "Explicit reviewer assertions for the selected assessment scope.",
            "start": "2026-01-01T00:00:00Z",
            "end": "2026-01-02T00:00:00Z",
            "control_ids": [CONTROL_ID],
            "objective_ids": [OBJECTIVE_ID],
            "observations": [{
                "key": "observation-access-review",
                "title": "Reviewed access evidence",
                "description": "The assessor recorded the reviewed condition.",
                "provenance": {
                    "assessor_key": "assessor-alice", "role_id": "assessor",
                    "start": "2026-01-01T01:00:00Z", "end": "2026-01-01T02:00:00Z",
                    "method": "EXAMINE", "rationale": "Recorded after direct human review."
                },
                "subjects": [
                    {"type": "component", "uuid": component_uuid},
                    {"type": "inventory-item", "uuid": INVENTORY_UUID},
                    {"type": "location", "uuid": LOCATION_UUID},
                    {"type": "party", "uuid": PARTY_UUID},
                    {"type": "user", "uuid": user_uuid},
                    {"type": "resource", "uuid": resource_uuid}
                ],
                "task_uuids": [TASK_UUID],
                "evidence_keys": ["evidence-1"]
            }],
            "findings": [{
                "key": "finding-access-state",
                "title": "Assessor-declared access state",
                "description": "The assessor explicitly declared the target state.",
                "provenance": {
                    "assessor_key": "assessor-alice", "role_id": "assessor",
                    "start": "2026-01-01T02:00:00Z",
                    "method": "EXAMINE", "rationale": "Human judgment based on the observation."
                },
                "target": {"type": "statement-id", "id": STATEMENT_ID,
                    "state": "satisfied", "reason": "pass"},
                "implementation_statement_uuid": implementation_uuid
            }, {
                "key": "finding-access-objective",
                "title": "Assessor-declared objective state",
                "description": "The assessor explicitly declared the objective state.",
                "provenance": {
                    "assessor_key": "assessor-alice", "role_id": "assessor",
                    "start": "2026-01-01T02:30:00Z",
                    "method": "TEST", "rationale": "Human judgment based on the reviewed test."
                },
                "target": {"type": "objective-id", "id": OBJECTIVE_ID,
                    "state": "not-satisfied", "reason": "fail"}
            }],
            "risks": [{
                "key": "risk-access-review",
                "title": "Reviewer-recorded access risk",
                "description": "Risk language supplied by the assessor.",
                "statement": "The assessor chose to record this residual risk.",
                "status": "open", "severity": "moderate", "confidence": 0.8,
                "provenance": {
                    "assessor_key": "assessor-alice", "role_id": "assessor",
                    "start": "2026-01-01T03:00:00Z",
                    "method": "INTERVIEW", "rationale": "Human risk judgment."
                }
            }],
            "relationships": [
                {"from": {"type": "observation", "key": "observation-access-review"},
                 "to": {"type": "finding", "key": "finding-access-state"}},
                {"from": {"type": "observation", "key": "observation-access-review"},
                 "to": {"type": "finding", "key": "finding-access-objective"}},
                {"from": {"type": "finding", "key": "finding-access-state"},
                 "to": {"type": "risk", "key": "risk-access-review"}}
            ]
        }
    });
    let manifest = root.join("manifest.json");
    write_json(&manifest, &manifest_value);
    Fixture {
        directory,
        manifest,
        output: root.join("assessment-results.json"),
        report: root.join("assessment-results-report.json"),
    }
}

#[test]
fn build_is_schema_valid_deterministic_and_content_minimizing() {
    let fixture = fixture();
    let first = run(&[
        "assessment",
        "results",
        "build",
        "--manifest",
        fixture.manifest.to_str().unwrap(),
        "--output",
        fixture.output.to_str().unwrap(),
        "--report",
        fixture.report.to_str().unwrap(),
        "--report-format",
        "json",
    ]);
    assert_eq!(first.status.code(), Some(0), "{}", String::from_utf8_lossy(&first.stderr));
    let first_bytes = std::fs::read(&fixture.output).unwrap();
    let report_bytes = std::fs::read(&fixture.report).unwrap();
    let artifact: Value = serde_json::from_slice(&first_bytes).unwrap();
    let report: Value = serde_json::from_slice(&report_bytes).unwrap();
    let schema: Value =
        serde_json::from_str(include_str!("../schemas/oscal_assessment-results_schema.json"))
            .unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    let schema_errors: Vec<_> =
        validator.iter_errors(&artifact).map(|error| error.to_string()).collect();
    assert!(schema_errors.is_empty(), "{}", schema_errors.join("\n"));
    assert_eq!(
        artifact["assessment-results"]["results"][0]["observations"][0]["methods"][0],
        "EXAMINE"
    );
    let result = &artifact["assessment-results"]["results"][0];
    let findings = result["findings"].as_array().unwrap();
    assert!(findings.iter().any(|finding| {
        finding["target"]["target-id"] == STATEMENT_ID
            && finding["target"]["status"]["state"] == "satisfied"
    }));
    assert!(findings.iter().any(|finding| {
        finding["target"]["target-id"] == OBJECTIVE_ID
            && finding["target"]["status"]["state"] == "not-satisfied"
    }));
    assert_eq!(result["observations"][0]["subjects"].as_array().unwrap().len(), 6);
    assert_eq!(result["observations"][0]["origins"][0]["related-tasks"][0]["task-uuid"], TASK_UUID);
    assert_eq!(report["validation"]["assessment_results_schema_valid"], true);
    let rendered = String::from_utf8(first_bytes.clone()).unwrap();
    assert!(!rendered.contains("HIGHLY SENSITIVE EVIDENCE EXCERPT"));
    assert!(!rendered.contains(fixture.directory.path().to_str().unwrap()));

    let second = run(&[
        "assessment",
        "results",
        "build",
        "--manifest",
        fixture.manifest.to_str().unwrap(),
        "--output",
        fixture.output.to_str().unwrap(),
        "--report",
        fixture.report.to_str().unwrap(),
        "--report-format",
        "json",
    ]);
    assert_eq!(second.status.code(), Some(0), "{}", String::from_utf8_lossy(&second.stderr));
    assert_eq!(first_bytes, std::fs::read(&fixture.output).unwrap());
    assert_eq!(report_bytes, std::fs::read(&fixture.report).unwrap());
}

#[test]
fn invalid_graph_stale_context_and_missing_provenance_exit_two_without_output() {
    let invalid_graph_fixture = fixture();
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(&invalid_graph_fixture.manifest).unwrap()).unwrap();
    manifest["result"]["relationships"][0]["from"]["type"] = json!("risk");
    manifest["result"]["relationships"][0]["from"]["key"] = json!("risk-access-review");
    write_json(&invalid_graph_fixture.manifest, &manifest);
    let result = run(&[
        "assessment",
        "results",
        "build",
        "--manifest",
        invalid_graph_fixture.manifest.to_str().unwrap(),
        "--output",
        invalid_graph_fixture.output.to_str().unwrap(),
    ]);
    assert_eq!(result.status.code(), Some(2));
    assert!(!invalid_graph_fixture.output.exists());
    assert!(String::from_utf8_lossy(&result.stderr).contains("wrong-side"));

    let stale_context_fixture = fixture();
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(&stale_context_fixture.manifest).unwrap()).unwrap();
    manifest["context"]["catalog"]["expected_sha256"] = json!(EVIDENCE_HASH);
    write_json(&stale_context_fixture.manifest, &manifest);
    let result = run(&[
        "assessment",
        "results",
        "build",
        "--manifest",
        stale_context_fixture.manifest.to_str().unwrap(),
        "--output",
        stale_context_fixture.output.to_str().unwrap(),
    ]);
    assert_eq!(result.status.code(), Some(2));
    assert!(!stale_context_fixture.output.exists());
    assert!(String::from_utf8_lossy(&result.stderr).contains("SHA-256 mismatch"));

    let provenance_fixture = fixture();
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(&provenance_fixture.manifest).unwrap()).unwrap();
    manifest["result"]["findings"][0]["provenance"]["rationale"] = json!("");
    write_json(&provenance_fixture.manifest, &manifest);
    let result = run(&[
        "assessment",
        "results",
        "build",
        "--manifest",
        provenance_fixture.manifest.to_str().unwrap(),
        "--output",
        provenance_fixture.output.to_str().unwrap(),
    ]);
    assert_eq!(result.status.code(), Some(2));
    assert!(!provenance_fixture.output.exists());
    assert!(String::from_utf8_lossy(&result.stderr).contains("rationale"));

    let absent_target_fixture = fixture();
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(&absent_target_fixture.manifest).unwrap()).unwrap();
    manifest["result"]["findings"][0]["target"]["id"] = json!("ABSENT_smt");
    write_json(&absent_target_fixture.manifest, &manifest);
    let result = run(&[
        "assessment",
        "results",
        "build",
        "--manifest",
        absent_target_fixture.manifest.to_str().unwrap(),
        "--output",
        absent_target_fixture.output.to_str().unwrap(),
    ]);
    assert_eq!(result.status.code(), Some(2));
    assert!(!absent_target_fixture.output.exists());
    assert!(String::from_utf8_lossy(&result.stderr).contains("out-of-scope"));
}

#[test]
fn evidence_link_without_findings_does_not_invent_a_conclusion() {
    let fixture = fixture();
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(&fixture.manifest).unwrap()).unwrap();
    manifest["result"]["findings"] = json!([]);
    manifest["result"]["risks"] = json!([]);
    manifest["result"]["relationships"] = json!([]);
    write_json(&fixture.manifest, &manifest);
    let result = run(&[
        "assessment",
        "results",
        "build",
        "--manifest",
        fixture.manifest.to_str().unwrap(),
        "--output",
        fixture.output.to_str().unwrap(),
    ]);
    assert_eq!(result.status.code(), Some(0), "{}", String::from_utf8_lossy(&result.stderr));
    let artifact: Value = serde_json::from_slice(&std::fs::read(&fixture.output).unwrap()).unwrap();
    let result = &artifact["assessment-results"]["results"][0];
    assert_eq!(result["observations"].as_array().unwrap().len(), 1);
    assert!(result.get("findings").is_none());
    assert!(result.get("risks").is_none());
}

#[test]
fn assessment_plan_scope_excluded_by_profile_is_rejected() {
    let fixture = fixture();
    let root = fixture.manifest.parent().unwrap();
    let profile_path = root.join("profile.json");
    let mut profile: Value =
        serde_json::from_slice(&std::fs::read(&profile_path).unwrap()).unwrap();
    profile["profile"]["imports"] = json!([{
        "href": "catalog.json",
        "include-all": {},
        "exclude-controls": [{"with-ids": [CONTROL_ID]}]
    }]);
    let profile_bytes = write_json(&profile_path, &profile);
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(&fixture.manifest).unwrap()).unwrap();
    manifest["context"]["profile"]["expected_sha256"] = json!(sha256(&profile_bytes));
    write_json(&fixture.manifest, &manifest);

    let result = run(&[
        "assessment",
        "results",
        "build",
        "--manifest",
        fixture.manifest.to_str().unwrap(),
        "--output",
        fixture.output.to_str().unwrap(),
    ]);
    assert_eq!(result.status.code(), Some(2));
    assert!(!fixture.output.exists());
    assert!(String::from_utf8_lossy(&result.stderr).contains("resolved Profile/Catalog scope"));
}

#[test]
fn assessment_plan_exclusions_remove_controls_objectives_and_subjects_from_scope() {
    let cases: [fn(&mut Value); 3] = [
        |assessment_plan| {
            assessment_plan["assessment-plan"]["reviewed-controls"]["control-selections"][0] = json!({
                "include-all": {},
                "exclude-controls": [{"control-id": CONTROL_ID}]
            });
        },
        |assessment_plan| {
            assessment_plan["assessment-plan"]["reviewed-controls"]["control-objective-selections"]
                [0]["exclude-objectives"] = json!([{"objective-id": OBJECTIVE_ID}]);
        },
        |assessment_plan| {
            let component_uuid =
                assessment_plan["assessment-plan"]["assessment-subjects"][0]["include-subjects"][0]
                    ["subject-uuid"]
                    .clone();
            assessment_plan["assessment-plan"]["assessment-subjects"][0] = json!({
                "type": "component",
                "include-all": {},
                "exclude-subjects": [
                    {"subject-uuid": component_uuid, "type": "component"}
                ]
            });
        },
    ];

    for edit in cases {
        let fixture = fixture();
        mutate_assessment_plan(&fixture, edit);
        let result = run(&[
            "assessment",
            "results",
            "build",
            "--manifest",
            fixture.manifest.to_str().unwrap(),
            "--output",
            fixture.output.to_str().unwrap(),
        ]);
        assert_eq!(result.status.code(), Some(2), "{}", String::from_utf8_lossy(&result.stderr));
        assert!(!fixture.output.exists());
    }
}

#[test]
fn nested_assessment_plan_task_is_available_to_observation_provenance() {
    let fixture = fixture();
    mutate_assessment_plan(&fixture, |assessment_plan| {
        let nested_task = assessment_plan["assessment-plan"]["tasks"][0].take();
        assessment_plan["assessment-plan"]["tasks"] = json!([{
            "uuid": "88888888-8888-4888-8888-888888888888",
            "type": "action",
            "title": "Parent review task",
            "tasks": [nested_task]
        }]);
    });

    let result = run(&[
        "assessment",
        "results",
        "build",
        "--manifest",
        fixture.manifest.to_str().unwrap(),
        "--output",
        fixture.output.to_str().unwrap(),
    ]);
    assert_eq!(result.status.code(), Some(0), "{}", String::from_utf8_lossy(&result.stderr));
    let artifact: Value = serde_json::from_slice(&std::fs::read(&fixture.output).unwrap()).unwrap();
    assert_eq!(
        artifact["assessment-results"]["results"][0]["observations"][0]["origins"][0]["related-tasks"]
            [0]["task-uuid"],
        TASK_UUID
    );
}

#[test]
fn baseline_status_change_is_a_stable_review_action() {
    let fixture = fixture();
    let first = run(&[
        "assessment",
        "results",
        "build",
        "--manifest",
        fixture.manifest.to_str().unwrap(),
        "--output",
        fixture.output.to_str().unwrap(),
    ]);
    assert_eq!(first.status.code(), Some(0), "{}", String::from_utf8_lossy(&first.stderr));
    let baseline = fixture.directory.path().join("baseline.json");
    std::fs::copy(&fixture.output, &baseline).unwrap();

    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(&fixture.manifest).unwrap()).unwrap();
    manifest["result"]["risks"][0]["status"] = json!("investigating");
    manifest["result"]["risks"][0]["severity"] = json!("high");
    write_json(&fixture.manifest, &manifest);
    let result = run(&[
        "assessment",
        "results",
        "build",
        "--manifest",
        fixture.manifest.to_str().unwrap(),
        "--output",
        fixture.output.to_str().unwrap(),
        "--baseline",
        baseline.to_str().unwrap(),
        "--report",
        fixture.report.to_str().unwrap(),
        "--report-format",
        "json",
    ]);
    assert_eq!(result.status.code(), Some(1), "{}", String::from_utf8_lossy(&result.stderr));
    let report: Value = serde_json::from_slice(&std::fs::read(&fixture.report).unwrap()).unwrap();
    assert!(report["findings"].as_array().unwrap().iter().any(|finding| {
        finding["code"] == "status-changed" && finding["key"] == "risk-access-review"
    }));
    assert!(report["findings"].as_array().unwrap().iter().any(|finding| {
        finding["code"] == "content-changed" && finding["key"] == "risk-access-review"
    }));

    let html_path = fixture.directory.path().join("assessment-results-review.html");
    let html_result = run(&[
        "assessment",
        "results",
        "build",
        "--manifest",
        fixture.manifest.to_str().unwrap(),
        "--output",
        fixture.output.to_str().unwrap(),
        "--baseline",
        baseline.to_str().unwrap(),
        "--report",
        html_path.to_str().unwrap(),
        "--report-format",
        "html",
        "--fail-on",
        "never",
    ]);
    assert_eq!(html_result.status.code(), Some(0));
    let html = std::fs::read_to_string(html_path).unwrap();
    assert!(html.starts_with("<!doctype html>"));
    assert!(!html.contains("<script"));
}

#[test]
#[allow(
    clippy::too_many_lines,
    reason = "one revision scenario exercises every baseline finding category together"
)]
fn baseline_reports_object_content_rationale_stale_and_upstream_changes() {
    let fixture = fixture();
    let root = fixture.manifest.parent().unwrap();
    let first = run(&[
        "assessment",
        "results",
        "build",
        "--manifest",
        fixture.manifest.to_str().unwrap(),
        "--output",
        fixture.output.to_str().unwrap(),
    ]);
    assert_eq!(first.status.code(), Some(0), "{}", String::from_utf8_lossy(&first.stderr));
    let baseline = root.join("baseline-full.json");
    std::fs::copy(&fixture.output, &baseline).unwrap();

    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(&fixture.manifest).unwrap()).unwrap();
    manifest["result"]["observations"][0]["description"] =
        json!("Reviewer changed the recorded observation content.");
    manifest["result"]["observations"][0]["provenance"]["rationale"] =
        json!("Reviewer supplied a revised rationale.");
    manifest["result"]["observations"][0]["subjects"]
        .as_array_mut()
        .unwrap()
        .retain(|subject| subject["type"] != "location");
    manifest["result"]["observations"][0]["evidence_keys"] = json!([]);
    manifest["result"]["findings"]
        .as_array_mut()
        .unwrap()
        .retain(|finding| finding["key"] != "finding-access-objective");
    manifest["result"]["relationships"].as_array_mut().unwrap().retain(|relationship| {
        relationship["from"]["key"] != "finding-access-objective"
            && relationship["to"]["key"] != "finding-access-objective"
    });
    let mut added_risk = manifest["result"]["risks"][0].clone();
    added_risk["key"] = json!("risk-new-reviewer-record");
    added_risk["title"] = json!("New reviewer-recorded risk");
    manifest["result"]["risks"].as_array_mut().unwrap().push(added_risk);
    manifest["result"]["relationships"].as_array_mut().unwrap().push(json!({
        "from": {"type": "finding", "key": "finding-access-state"},
        "to": {"type": "risk", "key": "risk-new-reviewer-record"}
    }));

    let assessment_plan_path = root.join("assessment-plan.json");
    let mut assessment_plan: Value =
        serde_json::from_slice(&std::fs::read(&assessment_plan_path).unwrap()).unwrap();
    assessment_plan["assessment-plan"]["assessment-subjects"]
        .as_array_mut()
        .unwrap()
        .retain(|subject| subject["type"] != "location");
    let assessment_plan_bytes = write_json(&assessment_plan_path, &assessment_plan);
    manifest["context"]["assessment_plan"]["expected_sha256"] =
        json!(sha256(&assessment_plan_bytes));

    let ssp_path = root.join("ssp.json");
    let mut ssp: Value = serde_json::from_slice(&std::fs::read(&ssp_path).unwrap()).unwrap();
    ssp["system-security-plan"]["metadata"].as_object_mut().unwrap().remove("locations");
    let ssp_bytes = write_json(&ssp_path, &ssp);
    manifest["context"]["ssp"]["expected_sha256"] = json!(sha256(&ssp_bytes));

    let catalog_path = root.join("catalog.json");
    let mut catalog: Value =
        serde_json::from_slice(&std::fs::read(&catalog_path).unwrap()).unwrap();
    catalog["catalog"]["metadata"]["last-modified"] = json!("2026-01-03T00:00:00Z");
    let catalog_bytes = write_json(&catalog_path, &catalog);
    manifest["context"]["catalog"]["expected_sha256"] = json!(sha256(&catalog_bytes));

    let evidence_path = root.join("evidence-index.json");
    let evidence = json!({"schema_version": "forge.linkage-index/1", "records": []});
    let evidence_bytes = write_json(&evidence_path, &evidence);
    manifest["context"]["evidence_index"]["expected_sha256"] = json!(sha256(&evidence_bytes));
    write_json(&fixture.manifest, &manifest);

    let result = run(&[
        "assessment",
        "results",
        "build",
        "--manifest",
        fixture.manifest.to_str().unwrap(),
        "--output",
        fixture.output.to_str().unwrap(),
        "--baseline",
        baseline.to_str().unwrap(),
        "--report",
        fixture.report.to_str().unwrap(),
        "--report-format",
        "json",
        "--fail-on",
        "never",
    ]);
    assert_eq!(result.status.code(), Some(0), "{}", String::from_utf8_lossy(&result.stderr));
    let report: Value = serde_json::from_slice(&std::fs::read(&fixture.report).unwrap()).unwrap();
    let codes: std::collections::BTreeSet<_> = report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|finding| finding["code"].as_str())
        .collect();
    for expected in [
        "object-added",
        "object-removed",
        "content-changed",
        "rationale-changed",
        "stale-subject",
        "upstream-fingerprint-changed",
    ] {
        assert!(codes.contains(expected), "missing {expected}: {report:#}");
    }
}

#[test]
fn init_scaffolds_validated_scope_without_conclusions() {
    let fixture = fixture();
    let root = fixture.manifest.parent().unwrap();
    let scaffold = root.join("scaffold.json");
    let result = run(&[
        "assessment",
        "results",
        "init",
        "--assessment-plan",
        root.join("assessment-plan.json").to_str().unwrap(),
        "--ssp",
        root.join("ssp.json").to_str().unwrap(),
        "--profile",
        root.join("profile.json").to_str().unwrap(),
        "--catalog",
        root.join("catalog.json").to_str().unwrap(),
        "--evidence-index",
        root.join("evidence-index.json").to_str().unwrap(),
        "--output",
        scaffold.to_str().unwrap(),
    ]);
    assert_eq!(result.status.code(), Some(0), "{}", String::from_utf8_lossy(&result.stderr));
    let value: Value = serde_json::from_slice(&std::fs::read(scaffold).unwrap()).unwrap();
    assert_eq!(value["result"]["control_ids"], json!([CONTROL_ID]));
    assert_eq!(value["result"]["observations"], json!([]));
    assert_eq!(value["result"]["findings"], json!([]));
    assert_eq!(value["result"]["risks"], json!([]));
}
