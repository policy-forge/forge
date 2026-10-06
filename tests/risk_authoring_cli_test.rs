//! Proposed F10 blackbox controls using genuine five-native inputs and maintained CLI consumers.
//!
//! These declarations are authored source only. Fixture writes create private synthetic
//! inputs; they do not fabricate a captured inventory, reviewed authority or live evidence.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use chrono::{TimeZone as _, Utc};
use forge::oscal::catalog::OscalMetadata;
use forge::oscal::parts::OscalPartName;
use forge::oscal::{
    CatalogEnvelope, OscalCatalog, OscalControl, OscalPart, ProfileRoot, SelectionMode,
    SspComponentInput, build_assessment_plan, build_profile, build_ssp_skeleton,
};
use forge::poam::{manifest, source};
use serde_json::{Value, json};
use sha2::{Digest as _, Sha256};
use tempfile::TempDir;

/// Encode SHA-256 with the maintained helper's exact digest body, avoiding registration of unrelated common tests.
fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for &byte in &digest {
        use std::fmt::Write as _;
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

/// All actual native, declaration and caller-request bytes compared after each command.
const ORIGINALS: [&str; 8] = [
    "catalog.json",
    "profile.json",
    "ssp.json",
    "assessment-plan.json",
    "assessment-results.json",
    "ar-manifest.json",
    "empty-plan.json",
    "requests/reviewed.json",
];
/// Explicit scheduling date; the exporter must never choose a clock value.
const AS_OF: &str = "2026-10-04";
/// Stable native property namespace consumed by the existing workflow renderer.
#[cfg(any(target_os = "linux", target_os = "macos"))]
const WORKFLOW_NS: &str = "https://policy-forge.github.io/forge/ns/poam-workflow";

/// Private real-file source bundle and complete caller authoring request.
struct Fixture {
    /// Keep the owned directory alive without serializing its absolute pathname.
    _directory: TempDir,
    /// Canonical owned temporary root, avoiding macOS's `/var` alias in fixture setup.
    root: PathBuf,
    /// Exact caller-authored declaration with tuples observed from actual qualified native files.
    request: Value,
}

/// Execute the maintained binary in an explicitly chosen private working directory.
fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_forge")).current_dir(root).args(args).output().unwrap()
}

/// Persist deterministic fixture JSON and return the original bytes used by native pin declarations.
fn write_json(path: &Path, value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec_pretty(value).unwrap();
    bytes.push(b'\n');
    std::fs::write(path, &bytes).unwrap();
    bytes
}

/// Pin persisted native bytes and actual generated root metadata, never detached inventory labels.
fn pin(name: &str, bytes: &[u8], value: &Value, root: &str) -> Value {
    json!({"artifact":name,"href":name,"expected_sha256":sha256_hex(bytes),
        "root_uuid":value[root]["uuid"],"document_version":value[root]["metadata"]["version"],
        "oscal_version":value[root]["metadata"]["oscal-version"]})
}

/// Build the existing supported Catalog shape with deliberately sensitive unselected source prose.
fn native_catalog() -> OscalCatalog {
    OscalCatalog {
        uuid: "11111111-1111-4111-8111-111111111111".to_string(),
        metadata: OscalMetadata {
            title: "Synthetic catalog".to_string(),
            last_modified: "2026-01-01T00:00:00Z".to_string(),
            version: "1.0.0".to_string(),
            oscal_version: "1.2.3".to_string(),
        },
        controls: vec![OscalControl {
            id: "AC-1".to_string(),
            uuid: String::new(),
            title: "Synthetic access".to_string(),
            links: Vec::new(),
            params: Vec::new(),
            props: Vec::new(),
            parts: vec![
                OscalPart {
                    id: "AC-1_smt".to_string(),
                    name: OscalPartName::Statement,
                    prose: "SENSITIVE SOURCE STATEMENT".to_string(),
                    parts: Vec::new(),
                    props: Vec::new(),
                },
                OscalPart {
                    id: "AC-1_obj".to_string(),
                    name: OscalPartName::Objective,
                    prose: "SENSITIVE SOURCE OBJECTIVE".to_string(),
                    parts: Vec::new(),
                    props: Vec::new(),
                },
            ],
        }],
        groups: Vec::new(),
        back_matter: None,
    }
}

/// Generate four genuine companion models, bind the AP subject to the actual SSP component and pin bytes.
fn native_companions(root: &Path) -> (Value, Value) {
    let catalog = native_catalog();
    let catalog_value = serde_json::to_value(CatalogEnvelope { catalog: catalog.clone() }).unwrap();
    let catalog_bytes = write_json(&root.join("catalog.json"), &catalog_value);
    let profile = build_profile(
        "catalog.json",
        vec!["AC-1".to_string()],
        SelectionMode::Include,
        &[],
        Some(Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap()),
    )
    .unwrap();
    let profile_value = serde_json::to_value(ProfileRoot { profile }).unwrap();
    let profile_bytes = write_json(&root.join("profile.json"), &profile_value);
    let ssp = build_ssp_skeleton(
        "Synthetic system",
        "1.0.0",
        &catalog,
        &[SspComponentInput {
            title: "Synthetic component".to_string(),
            description: "Synthetic boundary".to_string(),
            component_type: forge::oscal::ssp::ComponentType::Software,
        }],
        "profile.json",
    )
    .unwrap();
    let ssp_value = serde_json::to_value(ssp).unwrap();
    let ssp_bytes = write_json(&root.join("ssp.json"), &ssp_value);
    let subject =
        ssp_value["system-security-plan"]["system-implementation"]["components"][0]["uuid"].clone();
    let mut ap_value = serde_json::to_value(
        build_assessment_plan(&["AC-1".to_string()], "ssp.json", "Synthetic assessment").unwrap(),
    )
    .unwrap();
    ap_value["assessment-plan"]["reviewed-controls"]["control-objective-selections"] =
        json!([{"include-objectives":[{"objective-id":"AC-1_obj"}]}]);
    ap_value["assessment-plan"]["assessment-subjects"] = json!([{
        "type":"component","include-subjects":[{"subject-uuid":subject,"type":"component"}]}]);
    let ap_bytes = write_json(&root.join("assessment-plan.json"), &ap_value);
    (
        json!({
            "assessment_plan":pin("assessment-plan.json",&ap_bytes,&ap_value,"assessment-plan"),
            "ssp":pin("ssp.json",&ssp_bytes,&ssp_value,"system-security-plan"),
            "profile":pin("profile.json",&profile_bytes,&profile_value,"profile"),
            "catalog":pin("catalog.json",&catalog_bytes,&catalog_value,"catalog"),
        }),
        subject,
    )
}

/// Author an actual AR-builder input containing two selected risks, one unselected risk and two findings.
fn assessment_declaration(context: &Value, subject: &Value) -> Value {
    let provenance = json!({"assessor_key":"synthetic-assessor","role_id":"assessor",
        "start":"2026-01-01T01:00:00Z","method":"EXAMINE","rationale":"SENSITIVE HUMAN ASSERTION"});
    let risks: Vec<Value> =
        [("risk-closed", "closed"), ("risk-open", "open"), ("risk-unselected", "open")]
            .into_iter()
            .map(|(key, status)| {
                json!({"key":key,"title":"SENSITIVE RISK TITLE",
            "description":"SENSITIVE RISK DESCRIPTION","statement":"SENSITIVE RISK STATEMENT",
            "status":status,"provenance":provenance})
            })
            .collect();
    let mut relationships = vec![
        json!({"from":{"type":"observation","key":"synthetic-observation"},"to":{"type":"finding","key":"finding-satisfied"}}),
        json!({"from":{"type":"observation","key":"synthetic-observation"},"to":{"type":"finding","key":"finding-unsatisfied"}}),
    ];
    for key in ["risk-closed", "risk-open", "risk-unselected"] {
        relationships.push(json!({"from":{"type":"finding","key":"finding-unsatisfied"},"to":{"type":"risk","key":key}}));
    }
    json!({"schema_version":"forge.assessment-results/1",
        "document":{"key":"synthetic-assessment","title":"Synthetic result source","version":"1.0.0","last_modified":"2026-01-02T00:00:00Z"},
        "context":context,"roles":[{"id":"assessor","title":"Synthetic assessor"}],
        "parties":[{"key":"synthetic-assessor","type":"person","name":"SENSITIVE ACTOR NAME"}],
        "result":{"key":"synthetic-result","title":"Synthetic epoch","description":"SENSITIVE EPOCH DESCRIPTION",
            "start":"2026-01-01T00:00:00Z","end":"2026-01-02T00:00:00Z","control_ids":["AC-1"],"objective_ids":["AC-1_obj"],
            "observations":[{"key":"synthetic-observation","description":"SENSITIVE OBSERVATION",
                "provenance":provenance,"subjects":[{"type":"component","uuid":subject}]}],
            "findings":[
                {"key":"finding-satisfied","title":"SENSITIVE FINDING TITLE","description":"SENSITIVE FINDING DESCRIPTION",
                    "provenance":provenance,"target":{"type":"statement-id","id":"AC-1_smt","state":"satisfied","reason":"pass"}},
                {"key":"finding-unsatisfied","title":"Synthetic unmet","description":"Synthetic unmet description",
                    "provenance":provenance,"target":{"type":"objective-id","id":"AC-1_obj","state":"not-satisfied","reason":"fail"}},
            ],"risks":risks,"relationships":relationships}})
}

/// Run real AR build to stdout and persist its complete native output; fixture setup never needs a file publisher.
fn assessment_native(root: &Path, context: &Value, subject: &Value) -> String {
    write_json(&root.join("ar-manifest.json"), &assessment_declaration(context, subject));
    let output = run(root, &["assessment", "results", "build", "--manifest", "ar-manifest.json"]);
    assert_eq!(output.status.code(), Some(0), "AR fixture build failed");
    assert_eq!(output.stderr, [] as [u8; 0]);
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    std::fs::write(root.join("assessment-results.json"), &output.stdout).unwrap();
    value["assessment-results"]["results"][0]["uuid"].as_str().unwrap().to_string()
}

/// Obtain the empty scaffold through maintained `poam init`, retaining its full current five-source declarations.
fn initialize_scaffold(root: &Path, result_uuid: &str) -> Value {
    let output = run(
        root,
        &[
            "poam",
            "init",
            "--assessment-results",
            "assessment-results.json",
            "--assessment-plan",
            "assessment-plan.json",
            "--ssp",
            "ssp.json",
            "--profile",
            "profile.json",
            "--catalog",
            "catalog.json",
            "--result-uuid",
            result_uuid,
            "--result-key",
            "synthetic-result",
            "--document-key",
            "synthetic-plan",
            "--title",
            "Synthetic caller plan",
            "--document-version",
            "1.0.0",
            "--last-modified",
            "2026-10-04T12:00:00Z",
        ],
    );
    assert_eq!(output.status.code(), Some(0), "maintained init fixture failed");
    assert_eq!(output.stderr, [] as [u8; 0]);
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    for field in ["roles", "parties", "items"] {
        assert_eq!(value[field].as_array().unwrap().as_slice(), &[] as &[Value]);
    }
    std::fs::write(root.join("empty-plan.json"), &output.stdout).unwrap();
    value
}

/// Copy exact public observations only after the existing source loader qualifies the actual five native originals.
fn current_risk_references(root: &Path, scaffold: &Value) -> Vec<Value> {
    let parsed: manifest::PoamManifest = serde_json::from_value(scaffold.clone()).unwrap();
    let actual = source::load(&root.join("empty-plan.json"), &parsed.source).unwrap();
    actual.verify_inputs().unwrap();
    assert_eq!(actual.inventory().objects.len(), 5);
    ["risk-closed", "risk-open"]
        .into_iter()
        .map(|key| {
            let object =
                actual.inventory().objects.iter().find(|object| object.key == key).unwrap();
            json!({"kind":object.kind,"key":object.key,"uuid":object.uuid,
            "result_uuid":object.result_uuid,"expected_sha256":object.sha256})
        })
        .collect()
}

/// Supply every owner, date, milestone and initial assertion explicitly; the second item repeats one exact risk.
fn author_request(mut scaffold: Value, references: &[Value]) -> Value {
    scaffold["roles"] = json!([{"id":"owner","title":"Caller owner role"}]);
    scaffold["parties"] = json!([{"key":"caller-owner","type":"person","name":"Caller owner"}]);
    let owner = json!({"role_id":"owner","party_key":"caller-owner","rationale":"Caller declared responsibility"});
    let initial = json!({"key":"initial","actor":{"role_id":"owner","party_key":"caller-owner"},
        "at":"2026-10-01T12:00:00Z","from":null,"to":"planned","rationale":"Caller reviewed planned work","closure":null});
    scaffold["items"] = json!([
        {"key":"work-a","title":"Caller work A","description":"Caller description A",
            "source_refs":references,"owners":[owner],"target_date":"2026-10-05","state":"planned","history":[initial],
            "milestones":[{"key":"step-a","outcome":"Caller measurable outcome A","target_date":"2026-10-04","depends_on":[],
                "owners":[owner],"state":"planned","history":[initial]}]},
        {"key":"work-b","title":"Caller work B","description":"Caller description B",
            "source_refs":[references[1]],"owners":[owner],"target_date":"2026-10-03","state":"planned","history":[initial],
            "milestones":[{"key":"step-b","outcome":"Caller measurable outcome B","target_date":"2026-10-02","depends_on":[],
                "owners":[owner],"state":"planned","history":[initial]}]},
    ]);
    json!({"schema_version":"forge.poam-risk-authoring/1","workflow":scaffold,
        "reviewed_risks":references.iter().map(|reference| json!({
            "source_ref":reference,"caller_asserted_reviewed":true})).collect::<Vec<_>>()})
}

/// Construct the actual source closure and verify a public stdout canary before every subsequent negative matrix.
fn fixture() -> Fixture {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let (context, subject) = native_companions(&root);
    let result_uuid = assessment_native(&root, &context, &subject);
    let scaffold = initialize_scaffold(&root, &result_uuid);
    let references = current_risk_references(&root, &scaffold);
    let request = author_request(scaffold, &references);
    std::fs::create_dir(root.join("requests")).unwrap();
    write_json(&root.join("requests/reviewed.json"), &request);
    let fixture = Fixture { _directory: directory, root, request };
    let canary = export(&fixture, None);
    assert_eq!(
        canary.status.code(),
        Some(0),
        "actual source/export canary must precede refusal assertions"
    );
    assert_eq!(canary.stderr, [] as [u8; 0]);
    assert_eq!(
        serde_json::from_slice::<Value>(&canary.stdout).unwrap(),
        fixture.request["workflow"]
    );
    fixture
}

/// Preserve every actual fixture original, including the declaration used by the native AR builder.
fn originals(fixture: &Fixture) -> BTreeMap<String, Vec<u8>> {
    ORIGINALS
        .into_iter()
        .map(|name| (name.to_string(), std::fs::read(fixture.root.join(name)).unwrap()))
        .collect()
}

/// Compare complete original bytes rather than transport digests or selected object shortcuts.
fn assert_originals(fixture: &Fixture, before: &BTreeMap<String, Vec<u8>>) {
    for (name, bytes) in before {
        assert_eq!(&std::fs::read(fixture.root.join(name)).unwrap(), bytes);
    }
}

/// Dispatch exact public export flags; omitted output means a complete JSON declaration on stdout.
fn export(fixture: &Fixture, output: Option<&str>) -> Output {
    let mut args = vec![
        "assessment",
        "results",
        "export-poam",
        "--scaffold",
        "empty-plan.json",
        "--authoring",
        "requests/reviewed.json",
        "--as-of",
        AS_OF,
    ];
    if let Some(name) = output {
        args.extend(["--output", name]);
    }
    run(&fixture.root, &args)
}

/// Require refusal before successful stdout/file publication and preserve every actual original byte.
fn assert_refused(fixture: &Fixture, output: &Output, before: &BTreeMap<String, Vec<u8>>) {
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stdout, [] as [u8; 0]);
    assert_ne!(output.stderr, [] as [u8; 0]);
    assert!(!fixture.root.join("refused.json").exists());
    assert_originals(fixture, before);
}

/// Persist one intentionally invalid caller request and exercise the actual public command with stdout mode.
fn refuse_request(fixture: &Fixture, request: &Value) {
    write_json(&fixture.root.join("requests/reviewed.json"), request);
    let before = originals(fixture);
    assert_refused(fixture, &export(fixture, None), &before);
}

/// Decode one existing namespaced compact native declaration without substituting a transport self-checker.
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn native_property(value: &Value, name: &str) -> Value {
    let property = value["props"]
        .as_array()
        .unwrap()
        .iter()
        .find(|property| property["ns"] == WORKFLOW_NS && property["name"] == name)
        .unwrap();
    serde_json::from_str(property["value"].as_str().unwrap()).unwrap()
}

/// Verify complete native links, authored declarations and all five real-file receipt hashes after maintained build.
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn assert_native_sources(fixture: &Fixture, native: &Value) {
    let plan = &native["plan-of-action-and-milestones"];
    assert_eq!(plan["import-ssp"]["href"], "ssp.json");
    let items = plan["poam-items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    for native_item in items {
        let stable = native_item["props"]
            .as_array()
            .unwrap()
            .iter()
            .find(|property| property["ns"] == WORKFLOW_NS && property["name"] == "stable-key")
            .unwrap();
        let caller = fixture.request["workflow"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["key"] == stable["value"])
            .unwrap();
        assert_eq!(native_item["title"], caller["title"]);
        assert_eq!(native_item["description"], caller["description"]);
        assert_eq!(native_property(native_item, "source-references"), caller["source_refs"]);
        assert_eq!(native_property(native_item, "history"), caller["history"]);
        let steps = native_property(native_item, "milestones");
        let milestones = caller["milestones"].as_array().unwrap();
        assert_eq!(steps.as_array().unwrap().len(), milestones.len());
        for (step, declaration) in steps.as_array().unwrap().iter().zip(milestones) {
            let mut supplied = step.clone();
            let native_uuid = supplied.as_object_mut().unwrap().remove("uuid").unwrap();
            assert!(uuid::Uuid::parse_str(native_uuid.as_str().unwrap()).is_ok());
            assert_eq!(&supplied, declaration);
        }
        let owners = native_property(native_item, "owners");
        let caller_owners = caller["owners"].as_array().unwrap();
        assert_eq!(owners.as_array().unwrap().len(), caller_owners.len());
        for (owner, declaration) in owners.as_array().unwrap().iter().zip(caller_owners) {
            let mut supplied = owner.clone();
            let party_uuid = supplied.as_object_mut().unwrap().remove("party_uuid").unwrap();
            assert!(uuid::Uuid::parse_str(party_uuid.as_str().unwrap()).is_ok());
            assert_eq!(&supplied, declaration);
        }
        let links = native_item["links"].as_array().unwrap();
        let references = caller["source_refs"].as_array().unwrap();
        assert_eq!(links.len(), references.len());
        for (link, reference) in links.iter().zip(references) {
            assert_eq!(link["rel"], "assessment-source");
            assert_eq!(
                link["href"],
                format!("assessment-results.json#{}", reference["uuid"].as_str().unwrap())
            );
        }
        assert!(native_item.get("related-risks").is_none());
    }
    let receipts = plan["back-matter"]["resources"].as_array().unwrap();
    assert_eq!(receipts.len(), 5);
    for receipt in receipts {
        let link = &receipt["rlinks"][0];
        let name = link["href"].as_str().unwrap();
        assert!(ORIGINALS[..5].contains(&name));
        assert_eq!(link["hashes"][0]["algorithm"], "SHA-256");
        assert_eq!(
            link["hashes"][0]["value"],
            sha256_hex(&std::fs::read(fixture.root.join(name)).unwrap())
        );
    }
}

/// Consume the exported declaration through BOTH maintained commands, official validation and the current schedule.
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn consume_workflow(fixture: &Fixture, manifest_name: &str, native_name: &str) -> (Vec<u8>, Value) {
    let checked = run(
        &fixture.root,
        &[
            "poam",
            "check",
            "--manifest",
            manifest_name,
            "--workflow",
            "--as-of",
            AS_OF,
            "--due-soon-days",
            "7",
            "--format",
            "json",
        ],
    );
    assert_eq!(checked.status.code(), Some(1), "valid declared work has current schedule actions");
    assert_eq!(checked.stderr, [] as [u8; 0]);
    let checked_schedule: Value = serde_json::from_slice(&checked.stdout).unwrap();
    let built = run(
        &fixture.root,
        &[
            "poam",
            "build",
            "--manifest",
            manifest_name,
            "--as-of",
            AS_OF,
            "--due-soon-days",
            "7",
            "--output",
            native_name,
            "--format",
            "json",
        ],
    );
    assert_eq!(built.status.code(), Some(1), "build retains valid-action exit semantics");
    assert_eq!(built.stderr, [] as [u8; 0]);
    let built_schedule: Value = serde_json::from_slice(&built.stdout).unwrap();
    assert_eq!(built_schedule, checked_schedule);
    assert_eq!(built_schedule["schema_version"], "forge.poam-schedule/1");
    assert_eq!(built_schedule["as_of"], AS_OF);
    assert_eq!(built_schedule["source_objects"], 5);
    assert_eq!(built_schedule["items"], 2);
    assert_eq!(built_schedule["milestones"], 2);
    assert_eq!(built_schedule["rows"].as_array().unwrap().len(), 4);
    assert_eq!(built_schedule["overdue"], 2);
    assert_eq!(built_schedule["due_soon"], 2);
    for row in built_schedule["rows"].as_array().unwrap() {
        assert_eq!(row["state"], "planned");
    }
    let bytes = std::fs::read(fixture.root.join(native_name)).unwrap();
    let native: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(
        forge::validate::validate_artifact(&native, forge::OscalModelType::Poam).unwrap().is_valid
    );
    let validated = run(&fixture.root, &["validate", native_name, "--format", "json"]);
    assert_eq!(validated.status.code(), Some(0));
    assert_native_sources(fixture, &native);
    for sensitive in
        ["SENSITIVE SOURCE", "SENSITIVE RISK", "SENSITIVE OBSERVATION", "SENSITIVE FINDING"]
    {
        assert!(!String::from_utf8_lossy(&bytes).contains(sensitive));
    }
    (bytes, built_schedule)
}

/// Genuine two-risk export preserves the whole caller declaration and serves both maintained native consumers.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn reviewed_risks_export_is_consumed_by_check_build_validate_and_complete_schedule() {
    let fixture = fixture();
    let before = originals(&fixture);
    let exported = export(&fixture, None);
    assert_eq!(exported.status.code(), Some(0));
    assert_eq!(exported.stderr, [] as [u8; 0]);
    assert_eq!(exported.stdout.last(), Some(&b'\n'));
    let workflow: Value = serde_json::from_slice(&exported.stdout).unwrap();
    assert_eq!(workflow, fixture.request["workflow"]);
    assert!(workflow.get("reviewed_risks").is_none());
    assert_eq!(workflow["items"][0]["source_refs"].as_array().unwrap().len(), 2);
    assert_eq!(workflow["items"][1]["source_refs"][0], workflow["items"][0]["source_refs"][1]);
    assert!(!String::from_utf8_lossy(&exported.stdout).contains("risk-unselected"));
    std::fs::write(fixture.root.join("workflow.json"), &exported.stdout).unwrap();
    consume_workflow(&fixture, "workflow.json", "native.json");
    assert_originals(&fixture, &before);
}

/// Separate output filenames do not create new source or item identities; repeated commands remain byte deterministic.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn deterministic_file_exports_and_native_builds_do_not_replace_existing_outputs() {
    let fixture = fixture();
    let before = originals(&fixture);
    for name in ["workflow-a.json", "workflow-b.json"] {
        let output = export(&fixture, Some(name));
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(output.stdout, [] as [u8; 0]);
        assert_eq!(output.stderr, [] as [u8; 0]);
    }
    let bytes = std::fs::read(fixture.root.join("workflow-a.json")).unwrap();
    assert_eq!(bytes, std::fs::read(fixture.root.join("workflow-b.json")).unwrap());
    let first = consume_workflow(&fixture, "workflow-a.json", "native-a.json");
    let second = consume_workflow(&fixture, "workflow-b.json", "native-b.json");
    assert_eq!(first, second);
    assert_eq!(export(&fixture, Some("workflow-a.json")).status.code(), Some(2));
    assert_eq!(std::fs::read(fixture.root.join("workflow-a.json")).unwrap(), bytes);
    assert_originals(&fixture, &before);
}

/// Supported multiline author descriptions and rationale retain exact values and still reach both native consumers.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn multiline_supported_caller_values_survive_real_native_consumers() {
    let mut fixture = fixture();
    fixture.request["workflow"]["items"][0]["description"] =
        json!("Caller description\nwith a\ttab");
    fixture.request["workflow"]["items"][0]["owners"][0]["rationale"] =
        json!("Declared\nowner\trationale");
    fixture.request["workflow"]["items"][0]["history"][0]["rationale"] =
        json!("Declared\nreview\trationale");
    write_json(&fixture.root.join("requests/reviewed.json"), &fixture.request);
    let before = originals(&fixture);
    let output = export(&fixture, None);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        fixture.request["workflow"]
    );
    std::fs::write(fixture.root.join("workflow.json"), &output.stdout).unwrap();
    consume_workflow(&fixture, "workflow.json", "native.json");
    assert_originals(&fixture, &before);
}

/// Caller array ordering is preserved in authoring output while native consumers keep their own stable-key ordering.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn caller_item_reference_and_review_order_are_not_rewritten() {
    let mut fixture = fixture();
    fixture.request["workflow"]["items"][0]["source_refs"].as_array_mut().unwrap().reverse();
    fixture.request["workflow"]["items"].as_array_mut().unwrap().reverse();
    fixture.request["reviewed_risks"].as_array_mut().unwrap().reverse();
    write_json(&fixture.root.join("requests/reviewed.json"), &fixture.request);
    let before = originals(&fixture);
    let output = export(&fixture, None);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        fixture.request["workflow"]
    );
    std::fs::write(fixture.root.join("workflow.json"), &output.stdout).unwrap();
    consume_workflow(&fixture, "workflow.json", "native.json");
    assert_originals(&fixture, &before);
}

/// A qualified stdout canary precedes all negative matrices, including platforms whose publisher is unavailable.
#[test]
fn stdout_export_requires_explicit_review_without_inventing_work_or_source_filtering() {
    let fixture = fixture();
    let before = originals(&fixture);
    let output = export(&fixture, None);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stderr, [] as [u8; 0]);
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        fixture.request["workflow"]
    );
    let again = export(&fixture, None);
    assert_eq!(again.status.code(), Some(0));
    assert_eq!(again.stdout, output.stdout);
    assert_originals(&fixture, &before);
}

/// Root-relative authoring is resolved from the scaffold parent even when the invoker works elsewhere.
#[test]
fn absolute_scaffold_keeps_authoring_root_and_does_not_follow_current_directory() {
    let fixture = fixture();
    let before = originals(&fixture);
    let outside = tempfile::tempdir().unwrap();
    let scaffold = fixture.root.join("empty-plan.json");
    let output = run(
        outside.path(),
        &[
            "assessment",
            "results",
            "export-poam",
            "--scaffold",
            scaffold.to_str().unwrap(),
            "--authoring",
            "requests/reviewed.json",
            "--as-of",
            AS_OF,
        ],
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        fixture.request["workflow"]
    );
    assert_eq!(std::fs::read_dir(outside.path()).unwrap().count(), 0);
    assert_originals(&fixture, &before);
}

/// Omission or misspelling of any mandatory public flag fails before an output can be created.
#[test]
fn missing_or_unknown_command_options_do_not_produce_a_declaration() {
    let fixture = fixture();
    let before = originals(&fixture);
    for args in [
        vec!["assessment", "results", "export-poam"],
        vec![
            "assessment",
            "results",
            "export-poam",
            "--scaffold",
            "empty-plan.json",
            "--as-of",
            AS_OF,
        ],
        vec![
            "assessment",
            "results",
            "export-poam",
            "--scaffold",
            "empty-plan.json",
            "--authoring",
            "requests/reviewed.json",
        ],
        vec![
            "assessment",
            "results",
            "export-poam",
            "--scaffold",
            "empty-plan.json",
            "--authoring",
            "requests/reviewed.json",
            "--as-of",
            AS_OF,
            "--approve-terminal",
        ],
    ] {
        assert_refused(&fixture, &run(&fixture.root, &args), &before);
    }
}

/// Closed request admission rejects unknown fields and review assertions that are absent, false or nonboolean.
#[test]
fn unknown_fields_and_nonexplicit_review_flags_are_whole_input_refusals() {
    let fixture = fixture();
    let mut cases = Vec::new();
    let mut unknown = fixture.request.clone();
    unknown["surprise"] = json!(true);
    cases.push(unknown);
    let mut nested = fixture.request.clone();
    nested["reviewed_risks"][0]["surprise"] = json!(true);
    cases.push(nested);
    for flag in [json!(false), json!("true"), Value::Null] {
        let mut request = fixture.request.clone();
        request["reviewed_risks"][0]["caller_asserted_reviewed"] = flag;
        cases.push(request);
    }
    let mut missing = fixture.request.clone();
    missing["reviewed_risks"][0].as_object_mut().unwrap().remove("caller_asserted_reviewed");
    cases.push(missing);
    for request in cases {
        refuse_request(&fixture, &request);
    }
}

/// Raw duplicate members, BOM, invalid UTF-8 and trailing values cannot be rescued by typed deserialization.
#[test]
fn raw_json_admission_is_strict_before_any_output() {
    let fixture = fixture();
    let bytes = serde_json::to_vec(&fixture.request).unwrap();
    let mut bom = vec![0xef, 0xbb, 0xbf];
    bom.extend_from_slice(&bytes);
    let mut trailing = bytes.clone();
    trailing.extend_from_slice(b" {}");
    let text = String::from_utf8(bytes.clone()).unwrap();
    let duplicate = text.replacen(
        "\"caller_asserted_reviewed\":true",
        "\"caller_asserted_reviewed\":true,\"caller_asserted_reviewed\":true",
        1,
    );
    assert_ne!(duplicate, text);
    let nested =
        text.replacen("\"state\":\"planned\"", "\"state\":\"planned\",\"state\":\"planned\"", 1);
    assert_ne!(nested, text);
    for invalid in [bom, trailing, vec![0xff, 0xfe], duplicate.into_bytes(), nested.into_bytes()] {
        std::fs::write(fixture.root.join("requests/reviewed.json"), invalid).unwrap();
        let before = originals(&fixture);
        assert_refused(&fixture, &export(&fixture, None), &before);
    }
}

/// Reviewed selection and the complete unique item union must agree; repetition is permitted only across items.
#[test]
fn selected_union_and_duplicate_scope_are_enforced_without_filtering() {
    let fixture = fixture();
    let mut missing = fixture.request.clone();
    missing["reviewed_risks"].as_array_mut().unwrap().remove(0);
    refuse_request(&fixture, &missing);
    let mut extra = fixture.request.clone();
    extra["workflow"]["items"][0]["source_refs"].as_array_mut().unwrap().remove(0);
    refuse_request(&fixture, &extra);
    let mut repeated = fixture.request.clone();
    let first = repeated["reviewed_risks"][0].clone();
    repeated["reviewed_risks"].as_array_mut().unwrap().push(first);
    refuse_request(&fixture, &repeated);
    let mut conflicting = fixture.request.clone();
    let mut first = conflicting["reviewed_risks"][0].clone();
    first["source_ref"]["expected_sha256"] = json!("a".repeat(64));
    conflicting["reviewed_risks"].as_array_mut().unwrap().push(first);
    refuse_request(&fixture, &conflicting);
    let mut item_duplicate = fixture.request.clone();
    let first = item_duplicate["workflow"]["items"][0]["source_refs"][0].clone();
    item_duplicate["workflow"]["items"][0]["source_refs"].as_array_mut().unwrap().push(first);
    refuse_request(&fixture, &item_duplicate);
}

/// Every tuple member is matched to actual current native objects, including divergent reviewed/item copies.
#[test]
fn stale_cross_result_or_nonrisk_tuple_members_cannot_author_work() {
    let fixture = fixture();
    let scaffold: Value =
        serde_json::from_slice(&std::fs::read(fixture.root.join("empty-plan.json")).unwrap())
            .unwrap();
    let parsed: manifest::PoamManifest = serde_json::from_value(scaffold).unwrap();
    let actual = source::load(&fixture.root.join("empty-plan.json"), &parsed.source).unwrap();
    let finding = actual
        .inventory()
        .objects
        .iter()
        .find(|object| object.key == "finding-unsatisfied")
        .unwrap();
    let reference = json!({"kind":finding.kind,"key":finding.key,"uuid":finding.uuid,
        "result_uuid":finding.result_uuid,"expected_sha256":finding.sha256});
    drop(actual);
    let mut current_finding = fixture.request.clone();
    current_finding["reviewed_risks"][0]["source_ref"] = reference.clone();
    current_finding["workflow"]["items"][0]["source_refs"][0] = reference;
    refuse_request(&fixture, &current_finding);
    for (field, value) in [
        ("kind", json!("finding")),
        ("kind", json!("observation")),
        ("key", json!("missing-risk")),
        ("uuid", json!("99999999-9999-4999-8999-999999999999")),
        ("result_uuid", json!("88888888-8888-4888-8888-888888888888")),
        ("expected_sha256", json!("a".repeat(64))),
    ] {
        let mut request = fixture.request.clone();
        request["reviewed_risks"][0]["source_ref"][field] = value.clone();
        request["workflow"]["items"][0]["source_refs"][0][field] = value;
        refuse_request(&fixture, &request);
    }
    let mut divergent = fixture.request.clone();
    divergent["workflow"]["items"][1]["source_refs"][0]["expected_sha256"] = json!("b".repeat(64));
    refuse_request(&fixture, &divergent);
}

/// The caller cannot replace any scaffold source pin, selected epoch or immutable document key.
#[test]
fn scaffold_source_and_document_identity_are_not_overwritten_with_defaults() {
    let fixture = fixture();
    for pointer in [
        "/workflow/document/key",
        "/workflow/source/assessment_results/artifact",
        "/workflow/source/assessment_results/href",
        "/workflow/source/assessment_results/expected_sha256",
        "/workflow/source/assessment_results/root_uuid",
        "/workflow/source/assessment_results/document_version",
        "/workflow/source/assessment_results/oscal_version",
        "/workflow/source/result/key",
        "/workflow/source/result/uuid",
        "/workflow/source/context/catalog/expected_sha256",
    ] {
        let mut request = fixture.request.clone();
        *request.pointer_mut(pointer).unwrap() = json!("caller-mismatched-pin");
        refuse_request(&fixture, &request);
    }
    let mut nonempty: Value =
        serde_json::from_slice(&std::fs::read(fixture.root.join("empty-plan.json")).unwrap())
            .unwrap();
    nonempty["roles"] = fixture.request["workflow"]["roles"].clone();
    write_json(&fixture.root.join("empty-plan.json"), &nonempty);
    write_json(&fixture.root.join("requests/reviewed.json"), &fixture.request);
    let before = originals(&fixture);
    assert_refused(&fixture, &export(&fixture, None), &before);
}

/// Required caller work and ownership cannot be omitted, emptied or pointed at undeclared actors.
#[test]
fn incomplete_work_owner_actor_and_milestone_values_are_refused() {
    let fixture = fixture();
    for pointer in [
        "/workflow/items/0/title",
        "/workflow/items/0/description",
        "/workflow/items/0/target_date",
        "/workflow/items/0/owners/0/rationale",
        "/workflow/items/0/history/0/rationale",
        "/workflow/items/0/milestones/0/outcome",
        "/workflow/items/0/milestones/0/target_date",
    ] {
        let mut request = fixture.request.clone();
        *request.pointer_mut(pointer).unwrap() = json!("");
        refuse_request(&fixture, &request);
    }
    for pointer in
        ["/workflow/items/0/owners", "/workflow/items/0/milestones", "/workflow/items/0/history"]
    {
        let mut request = fixture.request.clone();
        *request.pointer_mut(pointer).unwrap() = json!([]);
        refuse_request(&fixture, &request);
    }
    for pointer in
        ["/workflow/items/0/owners/0/party_key", "/workflow/items/0/history/0/actor/role_id"]
    {
        let mut request = fixture.request.clone();
        *request.pointer_mut(pointer).unwrap() = json!("undeclared");
        refuse_request(&fixture, &request);
    }
    let mut absent = fixture.request.clone();
    absent["workflow"]["items"][0].as_object_mut().unwrap().remove("description");
    refuse_request(&fixture, &absent);
}

/// Every item and milestone starts planned with exactly one explicit initial event; no terminal bypass is exposed.
#[test]
fn initial_profile_and_d064_history_boundary_are_whole_input_guards() {
    let fixture = fixture();
    for state in ["in-progress", "cancelled", "completed", "risk-accepted"] {
        for location in ["/workflow/items/0", "/workflow/items/0/milestones/0"] {
            let mut request = fixture.request.clone();
            let record = request.pointer_mut(location).unwrap();
            record["state"] = json!(state);
            record["history"][0]["to"] = json!(state);
            refuse_request(&fixture, &request);
        }
    }
    let mut additional = fixture.request.clone();
    let initial = additional["workflow"]["items"][0]["history"][0].clone();
    additional["workflow"]["items"][0]["history"].as_array_mut().unwrap().push(initial);
    refuse_request(&fixture, &additional);
    for field in ["from", "closure"] {
        let mut absent = fixture.request.clone();
        absent["workflow"]["items"][0]["history"][0].as_object_mut().unwrap().remove(field);
        refuse_request(&fixture, &absent);
    }
    let mut prior = fixture.request.clone();
    prior["workflow"]["items"][0]["history"][0]["from"] = json!("completed");
    refuse_request(&fixture, &prior);
    let mut closure = fixture.request.clone();
    closure["workflow"]["items"][0]["history"][0]["closure"] = json!({});
    refuse_request(&fixture, &closure);
}

/// Canonical dates, actor chronology and milestone ordering are not repaired from a hidden clock.
#[test]
fn calendar_chronology_and_dependency_errors_do_not_produce_partial_work() {
    let fixture = fixture();
    let before = originals(&fixture);
    for date in ["2027-02-29", "2026-1-04", "+2026-10-04", "02026-10-04", "2026-10-04T00:00:00Z"] {
        let output = run(
            &fixture.root,
            &[
                "assessment",
                "results",
                "export-poam",
                "--scaffold",
                "empty-plan.json",
                "--authoring",
                "requests/reviewed.json",
                "--as-of",
                date,
            ],
        );
        assert_refused(&fixture, &output, &before);
    }
    let valid = run(
        &fixture.root,
        &[
            "assessment",
            "results",
            "export-poam",
            "--scaffold",
            "empty-plan.json",
            "--authoring",
            "requests/reviewed.json",
            "--as-of",
            "2028-02-29",
        ],
    );
    assert_eq!(valid.status.code(), Some(0));
    assert_eq!(
        serde_json::from_slice::<Value>(&valid.stdout).unwrap(),
        fixture.request["workflow"]
    );
    for (pointer, value) in [
        ("/workflow/items/0/target_date", json!("2027-02-29")),
        ("/workflow/items/0/history/0/at", json!("2026-10-05T00:00:00Z")),
        ("/workflow/items/0/milestones/0/target_date", json!("2026-12-31")),
        ("/workflow/items/0/milestones/0/depends_on", json!(["step-a"])),
    ] {
        let mut request = fixture.request.clone();
        *request.pointer_mut(pointer).unwrap() = value;
        refuse_request(&fixture, &request);
    }
}

/// Native single-line titles remain official-schema constrained even when generic authored text admits them.
#[test]
fn native_invalid_titles_are_refused_before_export() {
    let fixture = fixture();
    for pointer in [
        "/workflow/document/title",
        "/workflow/items/0/title",
        "/workflow/roles/0/title",
        "/workflow/parties/0/name",
    ] {
        let mut request = fixture.request.clone();
        *request.pointer_mut(pointer).unwrap() = json!("Caller\nline break");
        refuse_request(&fixture, &request);
    }
}

/// Raw and scalar byte caps refuse full input rather than truncating an otherwise valid declaration.
#[test]
fn raw_request_and_string_byte_limits_are_checked_before_output() {
    let fixture = fixture();
    let mut oversized = serde_json::to_vec(&fixture.request).unwrap();
    oversized.resize(4 * 1024 * 1024 + 1, b' ');
    std::fs::write(fixture.root.join("requests/reviewed.json"), oversized).unwrap();
    let before = originals(&fixture);
    assert_refused(&fixture, &export(&fixture, None), &before);
    let mut request = fixture.request.clone();
    request["workflow"]["items"][0]["description"] = json!("é".repeat(32_769));
    refuse_request(&fixture, &request);
    let mut key = fixture.request.clone();
    key["workflow"]["items"][0]["key"] = json!("k".repeat(257));
    refuse_request(&fixture, &key);
    let mut owners = fixture.request.clone();
    let mut declared = vec![owners["workflow"]["items"][0]["owners"][0].clone()];
    for index in 0..64 {
        let key = format!("owner-{index}");
        owners["workflow"]["parties"].as_array_mut().unwrap().push(json!({
            "key":key,"type":"person","name":format!("Caller owner {index}")}));
        declared.push(
            json!({"role_id":"owner","party_key":key,"rationale":"Explicit unique responsibility"}),
        );
    }
    owners["workflow"]["items"][0]["owners"] = json!(declared);
    let mut supported = owners.clone();
    supported["workflow"]["items"][0]["owners"].as_array_mut().unwrap().pop();
    write_json(&fixture.root.join("requests/reviewed.json"), &supported);
    let output = export(&fixture, None);
    assert_eq!(
        output.status.code(),
        Some(0),
        "64 unique declared owners form the positive boundary"
    );
    assert_eq!(serde_json::from_slice::<Value>(&output.stdout).unwrap(), supported["workflow"]);
    refuse_request(&fixture, &owners);
}

/// Conservative whole-projection refusal uses unique items and repeated valid cross-item tuples, not duplicate shortcuts.
#[test]
fn whole_projection_refuses_large_valid_repetition_without_prefix_success() {
    let fixture = fixture();
    let mut request = fixture.request.clone();
    let template = request["workflow"]["items"][1].clone();
    let mut items = Vec::new();
    for index in 0..100 {
        let mut item = template.clone();
        item["key"] = json!(format!("bounded-{index}"));
        item["description"] = json!("Caller ".repeat(1_000));
        items.push(item);
    }
    request["workflow"]["items"] = json!(items);
    let first = fixture.request["workflow"]["items"][0].clone();
    request["workflow"]["items"].as_array_mut().unwrap().push(first);
    assert!(serde_json::to_vec(&request).unwrap().len() < 4 * 1024 * 1024);
    refuse_request(&fixture, &request);
}

/// Changed full companion bytes invalidate fixed scaffold pins even when semantic JSON content is unchanged.
#[test]
fn every_native_original_pin_is_checked_without_selected_only_shortcuts() {
    let fixture = fixture();
    for name in &ORIGINALS[..5] {
        let path = fixture.root.join(name);
        let original = std::fs::read(&path).unwrap();
        let mut changed = original.clone();
        changed.extend_from_slice(b" \n");
        std::fs::write(&path, changed).unwrap();
        let before = originals(&fixture);
        assert_refused(&fixture, &export(&fixture, None), &before);
        std::fs::write(&path, original).unwrap();
    }
    assert_eq!(export(&fixture, None).status.code(), Some(0));
}

/// Existing consumers reject a generation changed after export; this is an inter-command check, not a paused-worker proof.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn changed_original_after_export_refuses_both_consumers_without_native_publication() {
    let fixture = fixture();
    let exported = export(&fixture, None);
    assert_eq!(exported.status.code(), Some(0));
    std::fs::write(fixture.root.join("workflow.json"), exported.stdout).unwrap();
    let path = fixture.root.join("catalog.json");
    let mut bytes = std::fs::read(&path).unwrap();
    bytes.push(b' ');
    std::fs::write(&path, bytes).unwrap();
    let before = originals(&fixture);
    for args in [
        vec![
            "poam",
            "check",
            "--manifest",
            "workflow.json",
            "--workflow",
            "--as-of",
            AS_OF,
            "--format",
            "json",
        ],
        vec![
            "poam",
            "build",
            "--manifest",
            "workflow.json",
            "--as-of",
            AS_OF,
            "--output",
            "refused.json",
        ],
    ] {
        assert_eq!(run(&fixture.root, &args).status.code(), Some(2));
    }
    assert!(!fixture.root.join("refused.json").exists());
    assert_originals(&fixture, &before);
}

/// Original scaffold and private authoring spellings are validated before path normalization can erase aliases.
#[test]
fn unsafe_raw_input_paths_cannot_escape_or_alias_the_captured_root() {
    let fixture = fixture();
    let before = originals(&fixture);
    for scaffold in [
        "./empty-plan.json",
        "requests/../empty-plan.json",
        "empty-plan.json/",
        "requests//../empty-plan.json",
    ] {
        let output = run(
            &fixture.root,
            &[
                "assessment",
                "results",
                "export-poam",
                "--scaffold",
                scaffold,
                "--authoring",
                "requests/reviewed.json",
                "--as-of",
                AS_OF,
            ],
        );
        assert_refused(&fixture, &output, &before);
    }
    let absolute = fixture.root.join("requests/reviewed.json");
    for authoring in [
        "./requests/reviewed.json",
        "requests//reviewed.json",
        "requests/../requests/reviewed.json",
        "../reviewed.json",
        absolute.to_str().unwrap(),
    ] {
        let output = run(
            &fixture.root,
            &[
                "assessment",
                "results",
                "export-poam",
                "--scaffold",
                "empty-plan.json",
                "--authoring",
                authoring,
                "--as-of",
                AS_OF,
            ],
        );
        assert_refused(&fixture, &output, &before);
    }
}

/// Hardlinked or symlinked request originals cannot impersonate a separate qualified capture role.
#[cfg(unix)]
#[test]
fn request_link_aliases_are_refused_without_opening_outside_source_inputs() {
    let fixture = fixture();
    let before = originals(&fixture);
    std::fs::hard_link(
        fixture.root.join("requests/reviewed.json"),
        fixture.root.join("requests/hard.json"),
    )
    .unwrap();
    let hard = run(
        &fixture.root,
        &[
            "assessment",
            "results",
            "export-poam",
            "--scaffold",
            "empty-plan.json",
            "--authoring",
            "requests/hard.json",
            "--as-of",
            AS_OF,
        ],
    );
    assert_refused(&fixture, &hard, &before);
    std::fs::remove_file(fixture.root.join("requests/hard.json")).unwrap();
    std::os::unix::fs::symlink("reviewed.json", fixture.root.join("requests/link.json")).unwrap();
    let linked = run(
        &fixture.root,
        &[
            "assessment",
            "results",
            "export-poam",
            "--scaffold",
            "empty-plan.json",
            "--authoring",
            "requests/link.json",
            "--as-of",
            AS_OF,
        ],
    );
    assert_refused(&fixture, &linked, &before);
}

/// Export destinations stay absent portable single filenames beside the scaffold and never overwrite inputs.
#[test]
fn existing_relocated_casefolded_or_input_output_names_are_not_replaced() {
    let fixture = fixture();
    let before = originals(&fixture);
    std::fs::write(fixture.root.join("existing.json"), b"preserve foreign destination").unwrap();
    let absolute = fixture.root.join("refused.json");
    for name in [
        "existing.json",
        "empty-plan.json",
        "EMPTY-PLAN.JSON",
        "catalog.json",
        "requests/reviewed.json",
        "subdir/new.json",
        "../refused.json",
        "./refused.json",
        "CON.json",
        "private:stream.json",
        absolute.to_str().unwrap(),
    ] {
        assert_refused(&fixture, &export(&fixture, Some(name)), &before);
    }
    assert_eq!(
        std::fs::read(fixture.root.join("existing.json")).unwrap(),
        b"preserve foreign destination"
    );
}

/// Two genuine competing command invocations grant exactly one no-replace destination without mutating originals.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn competing_export_attempts_have_one_complete_no_replace_winner() {
    let fixture = fixture();
    let before = originals(&fixture);
    let outcomes = std::thread::scope(|scope| {
        let one = scope.spawn(|| export(&fixture, Some("shared.json")));
        let two = scope.spawn(|| export(&fixture, Some("shared.json")));
        [one.join().unwrap(), two.join().unwrap()]
    });
    let mut exits: Vec<_> = outcomes.iter().map(|output| output.status.code().unwrap()).collect();
    exits.sort_unstable();
    assert_eq!(exits, [0, 2]);
    for output in &outcomes {
        assert_eq!(output.stdout, [] as [u8; 0]);
    }
    let winner: Value =
        serde_json::from_slice(&std::fs::read(fixture.root.join("shared.json")).unwrap()).unwrap();
    assert_eq!(winner, fixture.request["workflow"]);
    assert_originals(&fixture, &before);
}

/// Fixed refusal diagnostics must not disclose absolute private roots or assessor prose from a stale source.
#[test]
fn refusal_diagnostics_do_not_copy_source_prose_or_private_root() {
    let fixture = fixture();
    let path = fixture.root.join("assessment-results.json");
    let mut bytes = std::fs::read(&path).unwrap();
    bytes.push(b' ');
    std::fs::write(path, bytes).unwrap();
    let before = originals(&fixture);
    let output = export(&fixture, None);
    assert_refused(&fixture, &output, &before);
    let error = String::from_utf8(output.stderr).unwrap();
    assert!(!error.contains(fixture.root.to_str().unwrap()));
    for source in ["SENSITIVE", "synthetic-assessor", "Caller description"] {
        assert!(!error.contains(source));
    }
}

/// Windows file publication remains typed unavailable after a valid stdout canary; no fallback output is written.
#[cfg(windows)]
#[test]
fn windows_keeps_supported_stdout_and_refuses_file_publication() {
    let fixture = fixture();
    let before = originals(&fixture);
    let stdout = export(&fixture, None);
    assert_eq!(stdout.status.code(), Some(0));
    assert_eq!(
        serde_json::from_slice::<Value>(&stdout.stdout).unwrap(),
        fixture.request["workflow"]
    );
    assert_refused(&fixture, &export(&fixture, Some("refused.json")), &before);
}
