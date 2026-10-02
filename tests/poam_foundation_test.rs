//! F07 mechanical foundation fixtures; assertions here are synthetic test data.

use std::path::Path;
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
use tempfile::TempDir;

mod common;

const NS: &str = "https://policy-forge.github.io/ns/assessment-results";
const CONTROL: &str = "AC-1";
const STATEMENT: &str = "AC-1_smt";
const OBJECTIVE: &str = "AC-1_obj";

struct Fixture {
    directory: TempDir,
    result_uuid: String,
}

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_forge")).current_dir(root).args(args).output().unwrap()
}

fn write_json(path: &Path, value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec_pretty(value).unwrap();
    bytes.push(b'\n');
    std::fs::write(path, &bytes).unwrap();
    bytes
}

fn pin(name: &str, bytes: &[u8], value: &Value, root: &str) -> Value {
    json!({"artifact":name,"href":name,"expected_sha256":common::sha256_hex(bytes),
        "root_uuid":value[root]["uuid"],"document_version":value[root]["metadata"]["version"],
        "oscal_version":value[root]["metadata"]["oscal-version"]})
}

#[allow(
    clippy::too_many_lines,
    reason = "one exact generated source bundle covers the five-artifact import contract"
)]
fn fixture() -> Fixture {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let catalog = OscalCatalog {
        uuid: "11111111-1111-4111-8111-111111111111".to_string(),
        metadata: OscalMetadata {
            title: "Synthetic catalog".to_string(),
            last_modified: "2026-01-01T00:00:00Z".to_string(),
            version: "1.0.0".to_string(),
            oscal_version: "1.2.3".to_string(),
        },
        controls: vec![OscalControl {
            id: CONTROL.to_string(),
            uuid: String::new(),
            title: "Synthetic access".to_string(),
            links: Vec::new(),
            params: Vec::new(),
            props: Vec::new(),
            parts: vec![
                OscalPart {
                    id: STATEMENT.to_string(),
                    name: OscalPartName::Statement,
                    prose: "SENSITIVE SOURCE STATEMENT".to_string(),
                    parts: Vec::new(),
                    props: Vec::new(),
                },
                OscalPart {
                    id: OBJECTIVE.to_string(),
                    name: OscalPartName::Objective,
                    prose: "SENSITIVE SOURCE OBJECTIVE".to_string(),
                    parts: Vec::new(),
                    props: Vec::new(),
                },
            ],
        }],
        groups: Vec::new(),
        back_matter: None,
    };
    let catalog_value = serde_json::to_value(CatalogEnvelope { catalog: catalog.clone() }).unwrap();
    let catalog_bytes = write_json(&root.join("catalog.json"), &catalog_value);
    let profile = build_profile(
        "catalog.json",
        vec![CONTROL.to_string()],
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
    let component_uuid =
        ssp_value["system-security-plan"]["system-implementation"]["components"][0]["uuid"].clone();
    let mut ap_value = serde_json::to_value(
        build_assessment_plan(&[CONTROL.to_string()], "ssp.json", "Synthetic assessment").unwrap(),
    )
    .unwrap();
    ap_value["assessment-plan"]["reviewed-controls"]["control-objective-selections"] =
        json!([{"include-objectives":[{"objective-id":OBJECTIVE}]}]);
    ap_value["assessment-plan"]["assessment-subjects"] = json!([{
        "type":"component","include-subjects":[{"subject-uuid":component_uuid,"type":"component"}]}]);
    let ap_bytes = write_json(&root.join("assessment-plan.json"), &ap_value);
    let provenance = json!({"assessor_key":"synthetic-assessor","role_id":"assessor",
        "start":"2026-01-01T01:00:00Z","method":"EXAMINE","rationale":"SENSITIVE HUMAN ASSERTION"});
    let ar_manifest = json!({"schema_version":"forge.assessment-results/1",
        "document":{"key":"synthetic-assessment","title":"Synthetic result source","version":"1.0.0","last_modified":"2026-01-02T00:00:00Z"},
        "context":{"assessment_plan":pin("assessment-plan.json",&ap_bytes,&ap_value,"assessment-plan"),
            "ssp":pin("ssp.json",&ssp_bytes,&ssp_value,"system-security-plan"),
            "profile":pin("profile.json",&profile_bytes,&profile_value,"profile"),
            "catalog":pin("catalog.json",&catalog_bytes,&catalog_value,"catalog")},
        "roles":[{"id":"assessor","title":"Synthetic assessor"}],
        "parties":[{"key":"synthetic-assessor","type":"person","name":"SENSITIVE ACTOR NAME"}],
        "result":{"key":"synthetic-result","title":"Synthetic epoch","description":"SENSITIVE EPOCH DESCRIPTION",
            "start":"2026-01-01T00:00:00Z","end":"2026-01-02T00:00:00Z","control_ids":[CONTROL],"objective_ids":[OBJECTIVE],
            "observations":[{"key":"synthetic-observation","description":"SENSITIVE OBSERVATION",
                "provenance":provenance,"subjects":[{"type":"component","uuid":component_uuid}]}],"findings":[
                {"key":"finding-satisfied","title":"SENSITIVE FINDING TITLE","description":"SENSITIVE FINDING DESCRIPTION",
                "provenance":provenance,"target":{"type":"statement-id","id":STATEMENT,"state":"satisfied","reason":"pass"}},
                {"key":"finding-unsatisfied","title":"Synthetic unmet","description":"Synthetic unmet description",
                "provenance":provenance,"target":{"type":"objective-id","id":OBJECTIVE,"state":"not-satisfied","reason":"fail"}}],
            "risks":[{"key":"risk-closed","title":"SENSITIVE RISK TITLE","description":"SENSITIVE RISK DESCRIPTION",
                "statement":"SENSITIVE RISK STATEMENT","status":"closed","provenance":provenance}],
            "relationships":[
                {"from":{"type":"observation","key":"synthetic-observation"},"to":{"type":"finding","key":"finding-satisfied"}},
                {"from":{"type":"observation","key":"synthetic-observation"},"to":{"type":"finding","key":"finding-unsatisfied"}},
                {"from":{"type":"finding","key":"finding-unsatisfied"},"to":{"type":"risk","key":"risk-closed"}}]}});
    write_json(&root.join("ar-manifest.json"), &ar_manifest);
    let output = run(
        root,
        &[
            "assessment",
            "results",
            "build",
            "--manifest",
            "ar-manifest.json",
            "--output",
            "./assessment-results.json",
        ],
    );
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    let ar: Value =
        serde_json::from_slice(&std::fs::read(root.join("assessment-results.json")).unwrap())
            .unwrap();
    let result_uuid = ar["assessment-results"]["results"][0]["uuid"].as_str().unwrap().to_string();
    Fixture { directory, result_uuid }
}

fn init(fixture: &Fixture, output: Option<&str>) -> Output {
    let mut args = vec![
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
        &fixture.result_uuid,
        "--result-key",
        "synthetic-result",
        "--document-key",
        "synthetic-plan",
        "--title",
        "Synthetic plan",
        "--document-version",
        "0.1.0",
        "--last-modified",
        "2026-01-03T00:00:00Z",
    ];
    if let Some(name) = output {
        args.extend(["--output", name]);
    }
    run(fixture.directory.path(), &args)
}

fn scaffold(fixture: &Fixture) -> Value {
    let result = init(fixture, None);
    assert_eq!(result.status.code(), Some(0), "{}", String::from_utf8_lossy(&result.stderr));
    let value = serde_json::from_slice(&result.stdout).unwrap();
    std::fs::write(fixture.directory.path().join("poam.json"), result.stdout).unwrap();
    value
}

fn check(fixture: &Fixture) -> Output {
    run(
        fixture.directory.path(),
        &["poam", "check", "--manifest", "poam.json", "--source-only", "--format", "json"],
    )
}

fn ar_mutate(fixture: &Fixture, mutate: impl FnOnce(&mut Value)) {
    let path = fixture.directory.path().join("assessment-results.json");
    let mut value: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    mutate(&mut value);
    write_json(&path, &value);
}

#[test]
fn scaffold_is_explicit_empty_deterministic_and_preserves_sources() {
    let f = fixture();
    let before = std::fs::read(f.directory.path().join("assessment-results.json")).unwrap();
    let one = init(&f, None);
    let two = init(&f, None);
    assert_eq!(one.status.code(), Some(0), "{}", String::from_utf8_lossy(&one.stderr));
    assert_eq!(one.stdout, two.stdout);
    let value: Value = serde_json::from_slice(&one.stdout).unwrap();
    assert_eq!(value["schema_version"], "forge.poam/1");
    for field in ["items", "roles", "parties"] {
        assert_eq!(value[field], json!([]));
    }
    assert!(value.get("plan-of-action-and-milestones").is_none());
    assert_eq!(std::fs::read(f.directory.path().join("assessment-results.json")).unwrap(), before);
}

#[test]
fn source_inventory_preserves_all_denominators_and_omits_sensitive_content() {
    let f = fixture();
    scaffold(&f);
    let output = check(&f);
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["validation_scope"], "source-integrity-only");
    assert_eq!(report["workflow_validated"], false);
    assert_eq!(report["objects"].as_array().unwrap().len(), 3);
    assert_eq!(report["objects"][0]["state"], "satisfied");
    assert_eq!(report["objects"][2]["state"], "closed");
    assert_eq!(report["objects"][2]["control_ids"], json!([CONTROL]));
    let text = String::from_utf8(output.stdout).unwrap();
    for sensitive in ["SENSITIVE", "Synthetic epoch", f.directory.path().to_str().unwrap()] {
        assert!(!text.contains(sensitive));
    }
}

#[test]
fn check_requires_explicit_source_only_scope_and_no_build_is_advertised() {
    let f = fixture();
    scaffold(&f);
    assert_eq!(
        run(f.directory.path(), &["poam", "check", "--manifest", "poam.json"]).status.code(),
        Some(2)
    );
    assert_eq!(
        run(f.directory.path(), &["poam", "build", "--manifest", "poam.json"]).status.code(),
        Some(2)
    );
}

#[test]
fn exact_selection_tuple_is_required_by_library() {
    let f = fixture();
    let value = scaffold(&f);
    let parsed = manifest::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
    let prepared = source::load(&f.directory.path().join("poam.json"), &parsed.source).unwrap();
    let row = &prepared.inventory().objects[0];
    let reference = manifest::SourceReference {
        kind: row.kind,
        key: row.key.clone(),
        uuid: row.uuid.clone(),
        result_uuid: row.result_uuid.clone(),
        expected_sha256: row.sha256.clone(),
    };
    source::validate_selection(&prepared, std::slice::from_ref(&reference)).unwrap();
    let mut mutations = Vec::new();
    let mut wrong = reference.clone();
    wrong.expected_sha256 = "a".repeat(64);
    mutations.push(wrong);
    let mut wrong = reference.clone();
    wrong.kind = manifest::SourceKind::Risk;
    mutations.push(wrong);
    let mut wrong = reference.clone();
    wrong.result_uuid = "88888888-8888-4888-8888-888888888888".to_string();
    mutations.push(wrong);
    let mut wrong = reference.clone();
    wrong.uuid = "99999999-9999-4999-8999-999999999999".to_string();
    mutations.push(wrong);
    let mut wrong = reference.clone();
    wrong.key = "other-key".to_string();
    mutations.push(wrong);
    for wrong in mutations {
        assert!(source::validate_selection(&prepared, &[wrong]).is_err());
    }
    assert!(source::validate_selection(&prepared, &[reference.clone(), reference]).is_err());
}

#[test]
fn stale_or_misbound_source_pins_fail_before_output() {
    let f = fixture();
    let original = scaffold(&f);
    for pointer in
        ["/source/assessment_results/expected_sha256", "/source/context/catalog/expected_sha256"]
    {
        let mut bad = original.clone();
        bad.pointer_mut(pointer).map(|value| *value = json!("a".repeat(64))).unwrap();
        write_json(&f.directory.path().join("poam.json"), &bad);
        let out = check(&f);
        assert_eq!(out.status.code(), Some(2));
        assert_eq!(out.stdout, [] as [u8; 0]);
    }
}

#[test]
fn explicit_result_key_and_uuid_must_match_same_epoch() {
    let f = fixture();
    let original = scaffold(&f);
    for (pointer, replacement) in [
        ("/source/result/key", "other-result"),
        ("/source/result/uuid", "99999999-9999-4999-8999-999999999999"),
    ] {
        let mut bad = original.clone();
        *bad.pointer_mut(pointer).unwrap() = json!(replacement);
        write_json(&f.directory.path().join("poam.json"), &bad);
        let out = check(&f);
        assert_eq!(out.status.code(), Some(2));
        assert_eq!(out.stdout, [] as [u8; 0]);
    }
}

#[test]
fn stale_source_context_assertions_are_rejected_even_with_fresh_file_hash() {
    let f = fixture();
    ar_mutate(&f, |ar| {
        let props = ar["assessment-results"]["metadata"]["props"].as_array_mut().unwrap();
        let prop = props
            .iter_mut()
            .find(|p| {
                p["name"] == "context-sha256"
                    && p["value"].as_str().unwrap().starts_with("catalog:")
            })
            .unwrap();
        prop["value"] = json!(format!("catalog:{}", "a".repeat(64)));
    });
    let out = init(&f, Some("failed.json"));
    assert_eq!(out.status.code(), Some(2));
    assert!(!f.directory.path().join("failed.json").exists());
}

#[test]
fn duplicate_stable_keys_and_uuids_are_not_accepted_as_new_source() {
    for duplicate_uuid in [false, true] {
        let f = fixture();
        ar_mutate(&f, |ar| {
            let findings =
                ar["assessment-results"]["results"][0]["findings"].as_array_mut().unwrap();
            if duplicate_uuid {
                findings[1]["uuid"] = findings[0]["uuid"].clone();
            } else {
                let key = findings[0]["props"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|p| p["name"] == "stable-key")
                    .unwrap()["value"]
                    .clone();
                let prop = findings[1]["props"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|p| p["name"] == "stable-key")
                    .unwrap();
                prop["value"] = key;
            }
        });
        let out = init(&f, None);
        assert_eq!(out.status.code(), Some(2));
        assert_eq!(out.stdout, [] as [u8; 0]);
    }
}

#[test]
fn accepted_source_relocation_preserves_scaffold_and_inventory_bytes() {
    let f = fixture();
    let first = init(&f, None);
    scaffold(&f);
    let report = check(&f);
    let other = tempfile::tempdir().unwrap();
    for entry in std::fs::read_dir(f.directory.path()).unwrap() {
        let entry = entry.unwrap();
        std::fs::copy(entry.path(), other.path().join(entry.file_name())).unwrap();
    }
    let relocated = Fixture { directory: other, result_uuid: f.result_uuid.clone() };
    assert_eq!(init(&relocated, None).stdout, first.stdout);
    assert_eq!(check(&relocated).stdout, report.stdout);
}

#[test]
fn source_revalidation_detects_changed_bytes_and_same_byte_replacement() {
    for replacement in [false, true] {
        let f = fixture();
        let value = scaffold(&f);
        let parsed = manifest::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
        let prepared = source::load(&f.directory.path().join("poam.json"), &parsed.source).unwrap();
        let path = f.directory.path().join("assessment-results.json");
        let bytes = std::fs::read(&path).unwrap();
        if replacement {
            std::fs::rename(&path, f.directory.path().join("old-source.json")).unwrap();
            std::fs::write(&path, bytes).unwrap();
        } else {
            std::fs::write(&path, b"changed").unwrap();
        }
        assert!(prepared.verify_inputs().is_err());
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn scaffold_publication_never_overwrites_existing_or_source_files() {
    let f = fixture();
    assert_eq!(init(&f, Some("new-plan.json")).status.code(), Some(0));
    let before = std::fs::read(f.directory.path().join("new-plan.json")).unwrap();
    assert_eq!(init(&f, Some("new-plan.json")).status.code(), Some(2));
    assert_eq!(std::fs::read(f.directory.path().join("new-plan.json")).unwrap(), before);
    let source = std::fs::read(f.directory.path().join("assessment-results.json")).unwrap();
    assert_eq!(init(&f, Some("assessment-results.json")).status.code(), Some(2));
    assert_eq!(std::fs::read(f.directory.path().join("assessment-results.json")).unwrap(), source);
}

#[cfg(unix)]
#[test]
fn symlink_hardlink_parent_alias_and_special_inputs_fail_closed() {
    for kind in 0..4 {
        let f = fixture();
        let source = f.directory.path().join("assessment-results.json");
        match kind {
            0 => {
                std::fs::rename(&source, f.directory.path().join("original.json")).unwrap();
                std::os::unix::fs::symlink("original.json", &source).unwrap();
            }
            1 => std::fs::hard_link(&source, f.directory.path().join("hardlink.json")).unwrap(),
            2 => {
                std::fs::remove_file(&source).unwrap();
                std::fs::create_dir(&source).unwrap();
            }
            _ => {
                std::fs::remove_file(&source).unwrap();
                let status = Command::new("mkfifo").arg(&source).status().unwrap();
                assert!(status.success());
            }
        }
        let out = init(&f, None);
        assert_eq!(out.status.code(), Some(2));
        assert_eq!(out.stdout, [] as [u8; 0]);
    }
}

#[test]
fn native_poam_validation_detects_model_and_enforces_system_identity_choice() {
    let mut value = json!({"plan-of-action-and-milestones":{"uuid":"11111111-1111-4111-8111-111111111111",
        "metadata":{"title":"Synthetic native plan","last-modified":"2026-01-01T00:00:00Z","version":"1.0.0","oscal-version":"1.2.3"},
        "import-ssp":{"href":"ssp.json"},"poam-items":[{"title":"Synthetic item","description":"Explicit synthetic work"}]}});
    assert_eq!(forge::validate::detect_model_type(&value).unwrap(), forge::OscalModelType::Poam);
    assert!(
        forge::validate::validate_artifact(&value, forge::OscalModelType::Poam).unwrap().is_valid
    );
    value["plan-of-action-and-milestones"].as_object_mut().unwrap().remove("import-ssp");
    let invalid = forge::validate::validate_artifact(&value, forge::OscalModelType::Poam).unwrap();
    assert!(!invalid.is_valid);
    assert!(invalid.errors.iter().any(|e| e.message.contains("import-ssp or system-id")));
    value["plan-of-action-and-milestones"]["system-id"] = json!({"id":"synthetic-system"});
    assert!(
        forge::validate::validate_artifact(&value, forge::OscalModelType::Poam).unwrap().is_valid
    );
    value["plan-of-action-and-milestones"]["poam-items"] = json!([]);
    assert!(
        !forge::validate::validate_artifact(&value, forge::OscalModelType::Poam).unwrap().is_valid
    );
}

#[test]
fn source_property_namespaces_are_not_interchangeable() {
    let f = fixture();
    ar_mutate(&f, |ar| {
        let props = ar["assessment-results"]["results"][0]["props"].as_array_mut().unwrap();
        assert!(props.iter().any(|p| p["name"] == "stable-key"));
        for prop in props {
            if prop["name"] == "stable-key" {
                assert_eq!(prop["ns"], NS);
                prop["ns"] = json!("https://example.invalid/other");
            }
        }
    });
    let out = init(&f, None);
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(out.stdout, [] as [u8; 0]);
}

#[test]
fn directly_constructed_cli_cannot_bypass_source_only_acknowledgement() {
    let cli = forge::cli::Cli {
        command: forge::cli::Commands::Poam {
            command: forge::cli::PoamCommand::Check {
                manifest: "missing.json".into(),
                source_only: false,
                format: forge::cli::AuthorReportFormat::Json,
            },
        },
        verbose: false,
        quiet: false,
        config: None,
    };
    let error = forge::cli::execute(&cli).unwrap_err();
    assert!(error.to_string().contains("source-only acknowledgement"));
    assert_eq!(forge::error::exit_code(&error), 2);
}

fn distinct_epoch(original: &Value) -> Value {
    use std::collections::BTreeMap;
    fn collect(value: &Value, map: &mut BTreeMap<String, String>) {
        match value {
            Value::Object(object) => {
                if let Some(uuid) = object.get("uuid").and_then(Value::as_str) {
                    map.insert(
                        uuid.to_string(),
                        uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, uuid.as_bytes()).to_string(),
                    );
                }
                for child in object.values() {
                    collect(child, map);
                }
            }
            Value::Array(values) => {
                for child in values {
                    collect(child, map);
                }
            }
            _ => {}
        }
    }
    fn rewrite(value: &mut Value, map: &BTreeMap<String, String>) {
        match value {
            Value::Object(object) => {
                if object.get("name").and_then(Value::as_str) == Some("stable-key") {
                    let key = object["value"].as_str().unwrap().to_string();
                    object.insert("value".to_string(), json!(format!("other-{key}")));
                }
                for child in object.values_mut() {
                    rewrite(child, map);
                }
            }
            Value::Array(values) => {
                for child in values {
                    rewrite(child, map);
                }
            }
            Value::String(string) => {
                if let Some(replacement) = map.get(string) {
                    *string = replacement.clone();
                }
            }
            _ => {}
        }
    }
    let mut map = BTreeMap::new();
    collect(original, &mut map);
    let mut other = original.clone();
    rewrite(&mut other, &map);
    other
}

#[test]
fn explicit_result_can_be_second_and_cross_epoch_references_fail() {
    let f = fixture();
    let mut original_risk = String::new();
    ar_mutate(&f, |ar| {
        let original = ar["assessment-results"]["results"][0].clone();
        original_risk = original["risks"][0]["uuid"].as_str().unwrap().to_string();
        ar["assessment-results"]["results"] = json!([distinct_epoch(&original), original]);
    });
    scaffold(&f);
    let output = check(&f);
    assert_eq!(output.status.code(), Some(0));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["result_uuid"], f.result_uuid);
    assert_eq!(report["result_key"], "synthetic-result");
    assert_eq!(report["objects"].as_array().unwrap().len(), 3);
    ar_mutate(&f, |ar| {
        ar["assessment-results"]["results"][0]["findings"][1]["related-risks"][0]["risk-uuid"] =
            json!(original_risk);
    });
    let output = init(&f, None);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stdout, [] as [u8; 0]);
}

#[test]
fn every_companion_identity_and_type_pin_is_verified() {
    let f = fixture();
    let original = scaffold(&f);
    for companion in ["assessment_plan", "ssp", "profile", "catalog"] {
        for field in ["root_uuid", "document_version", "oscal_version"] {
            let mut bad = original.clone();
            bad["source"]["context"][companion][field] = json!(match field {
                "root_uuid" => "88888888-8888-4888-8888-888888888888",
                "document_version" => "different-version",
                _ => "1.2.0",
            });
            write_json(&f.directory.path().join("poam.json"), &bad);
            let output = check(&f);
            assert_eq!(output.status.code(), Some(2), "{companion}.{field}");
            assert_eq!(output.stdout, [] as [u8; 0]);
        }
    }
    let mut parsed = manifest::parse(&serde_json::to_vec(&original).unwrap()).unwrap();
    parsed.source.context.catalog.artifact = "../catalog.json".into();
    assert!(source::load(&f.directory.path().join("poam.json"), &parsed.source).is_err());
    parsed.source.context.catalog.artifact = "./catalog.json".into();
    assert!(source::load(&f.directory.path().join("poam.json"), &parsed.source).is_err());
}

#[test]
fn missing_duplicate_partial_and_stale_native_context_receipts_fail() {
    for case in 0..7 {
        let f = fixture();
        ar_mutate(&f, |ar| {
            let document = &mut ar["assessment-results"];
            match case {
                0 => {
                    document.as_object_mut().unwrap().remove("back-matter");
                }
                1 => {
                    document["back-matter"]["resources"].as_array_mut().unwrap().pop();
                }
                2 => {
                    let first = document["back-matter"]["resources"][0]["props"].clone();
                    document["back-matter"]["resources"][1]["props"] = first;
                }
                3 => {
                    document["back-matter"]["resources"][0]["rlinks"][0]["hashes"][0]["value"] =
                        json!("a".repeat(64));
                }
                4 => {
                    let props = document["metadata"]["props"].as_array_mut().unwrap();
                    let duplicate =
                        props.iter().find(|p| p["name"] == "context-sha256").unwrap().clone();
                    props.push(duplicate);
                }
                5 => {
                    let props = document["metadata"]["props"].as_array_mut().unwrap();
                    let index = props.iter().position(|p| p["name"] == "context-sha256").unwrap();
                    props.remove(index);
                }
                _ => {
                    let props =
                        document["back-matter"]["resources"][0]["props"].as_array_mut().unwrap();
                    let prop = props.iter_mut().find(|p| p["name"] == "document-version").unwrap();
                    prop["value"] = json!("different-version");
                }
            }
        });
        let output = init(&f, None);
        assert_eq!(output.status.code(), Some(2), "case {case}");
        assert_eq!(output.stdout, [] as [u8; 0]);
    }
}

#[test]
fn nested_context_receipt_href_must_resolve_from_ar_directory() {
    let f = fixture();
    let root = f.directory.path();
    let mut plan = scaffold(&f);
    std::fs::create_dir(root.join("nested")).unwrap();
    for (field, filename) in [
        ("assessment_plan", "assessment-plan.json"),
        ("ssp", "ssp.json"),
        ("profile", "profile.json"),
        ("catalog", "catalog.json"),
    ] {
        let bytes = std::fs::read(root.join(filename)).unwrap();
        std::fs::rename(root.join(filename), root.join("nested").join(filename)).unwrap();
        assert_eq!(std::fs::read(root.join("nested").join(filename)).unwrap(), bytes);
        plan["source"]["context"][field]["artifact"] = json!(format!("nested/{filename}"));
    }
    plan["source"]["context"]["assessment_plan"]["href"] = json!("nested/assessment-plan.json");
    ar_mutate(&f, |ar| {
        ar["assessment-results"]["import-ap"]["href"] = json!("nested/assessment-plan.json");
        let receipts = ar["assessment-results"]["back-matter"]["resources"].as_array_mut().unwrap();
        let ap = receipts
            .iter_mut()
            .find(|receipt| {
                receipt["props"].as_array().unwrap().iter().any(|prop| {
                    prop["name"] == "context-kind" && prop["value"] == "assessment-plan"
                })
            })
            .unwrap();
        ap["rlinks"][0]["href"] = json!("nested/assessment-plan.json");
    });
    let bytes = std::fs::read(root.join("assessment-results.json")).unwrap();
    plan["source"]["assessment_results"]["expected_sha256"] = json!(common::sha256_hex(&bytes));
    write_json(&root.join("poam.json"), &plan);
    let parsed = manifest::parse(&serde_json::to_vec(&plan).unwrap()).unwrap();
    // All companion bytes and importer-relative AP/SSP/Profile/Catalog links remain valid.
    forge::assessment_results::context::load(&root.join("poam.json"), &parsed.source.context)
        .unwrap();
    let ar: Value = serde_json::from_slice(&bytes).unwrap();
    let ssp = ar["assessment-results"]["back-matter"]["resources"]
        .as_array()
        .unwrap()
        .iter()
        .find(|receipt| {
            receipt["props"].as_array().unwrap().iter().any(|prop| {
                prop["name"] == "context-kind" && prop["value"] == "system-security-plan"
            })
        })
        .unwrap();
    assert_eq!(ssp["rlinks"][0]["href"], plan["source"]["context"]["ssp"]["href"]);
    assert_ne!(ssp["rlinks"][0]["href"], plan["source"]["context"]["ssp"]["artifact"]);
    let output = check(&f);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stdout, [] as [u8; 0]);
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("AR context resource link resolves to a different confined companion")
    );
}

#[test]
fn freshly_pinned_ar_rejects_invalid_structure_scope_and_references_before_output() {
    let failures = [
        "source AR fails the pinned OSCAL 1.2.3 schema",
        "AR import-ap href differs from the exact Assessment Plan companion href",
        "AR duplicates a result stable key",
        "AR result scope differs from the captured AP/Profile/Catalog scope",
        "AR actor does not reference an exact local metadata party and role",
        "AR observation references an absent captured AP task",
    ];
    for (case, expected_error) in failures.into_iter().enumerate() {
        let f = fixture();
        let mut plan = scaffold(&f);
        ar_mutate(&f, |ar| {
            let document = &mut ar["assessment-results"];
            match case {
                0 => document["metadata"]["title"] = Value::Null,
                1 => document["import-ap"]["href"] = json!("different-assessment-plan.json"),
                2 => {
                    let original = document["results"][0].clone();
                    let mut other = distinct_epoch(&original);
                    let properties = other["props"].as_array_mut().unwrap();
                    let key =
                        properties.iter_mut().find(|prop| prop["name"] == "stable-key").unwrap();
                    key["value"] = json!("synthetic-result");
                    document["results"] = json!([original, other]);
                }
                3 => {
                    document["results"][0]["reviewed-controls"]["control-selections"][0]["include-controls"]
                        [0]["control-id"] = json!("absent-control");
                }
                4 => {
                    document["results"][0]["observations"][0]["origins"][0]["actors"][0]["role-id"] =
                        json!("absent-role");
                }
                _ => {
                    document["results"][0]["observations"][0]["origins"][0]["related-tasks"] =
                        json!([{ "task-uuid": "88888888-8888-4888-8888-888888888888" }]);
                }
            }
        });
        let bytes = std::fs::read(f.directory.path().join("assessment-results.json")).unwrap();
        plan["source"]["assessment_results"]["expected_sha256"] = json!(common::sha256_hex(&bytes));
        write_json(&f.directory.path().join("poam.json"), &plan);
        let output = check(&f);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(2), "case {case}: {stderr}");
        assert_eq!(output.stdout, [] as [u8; 0], "case {case}");
        assert!(
            stderr.contains(expected_error),
            "case {case}: expected {expected_error}; got {stderr}"
        );
    }
}

#[test]
fn wrong_companion_root_with_fresh_hash_is_rejected_before_output() {
    for (name, kind) in [
        ("assessment-plan.json", "assessment-plan"),
        ("ssp.json", "system-security-plan"),
        ("profile.json", "profile"),
        ("catalog.json", "catalog"),
    ] {
        let f = fixture();
        let mut scaffold = scaffold(&f);
        let path = f.directory.path().join(name);
        let mut value: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let document = value.as_object_mut().unwrap().remove(kind).unwrap();
        value.as_object_mut().unwrap().insert("unsupported-native-root".to_string(), document);
        let bytes = write_json(&path, &value);
        let field = match kind {
            "assessment-plan" => "assessment_plan",
            "system-security-plan" => "ssp",
            other => other,
        };
        scaffold["source"]["context"][field]["expected_sha256"] = json!(common::sha256_hex(&bytes));
        write_json(&f.directory.path().join("poam.json"), &scaffold);
        let result = check(&f);
        assert_eq!(result.status.code(), Some(2));
        assert_eq!(result.stdout, [] as [u8; 0]);
    }
}

#[test]
fn text_inventory_preserves_complete_counts_and_source_only_scope() {
    let f = fixture();
    scaffold(&f);
    let first = run(
        f.directory.path(),
        &["poam", "check", "--manifest", "poam.json", "--source-only", "--format", "text"],
    );
    let second = run(
        f.directory.path(),
        &["poam", "check", "--manifest", "poam.json", "--source-only", "--format", "text"],
    );
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(first.stdout, second.stdout);
    let text = String::from_utf8(first.stdout).unwrap();
    assert!(text.contains("objects: 3"));
    assert!(text.contains("finding finding-satisfied:"));
    assert!(!text.contains('\u{1b}'));
    assert!(text.contains("Source integrity only"));
    assert!(!text.contains("SENSITIVE"));
}
