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

/// Private synthetic native bundle and its explicitly selected assessment epoch.
struct Fixture {
    /// Owned temporary bundle; the test never grants remediation authority.
    directory: TempDir,
    /// Native result UUID selected alongside its exact stable key.
    result_uuid: String,
}

/// Invoke the actual test binary in the private fixture root and retain its exit and output.
fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_forge")).current_dir(root).args(args).output().unwrap()
}

/// Write deterministic fixture JSON and return the exact persisted bytes for pinning.
fn write_json(path: &Path, value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec_pretty(value).unwrap();
    bytes.push(b'\n');
    std::fs::write(path, &bytes).unwrap();
    bytes
}

/// Construct exact synthetic artifact pins from the persisted fixture bytes and native root.
fn pin(name: &str, bytes: &[u8], value: &Value, root: &str) -> Value {
    json!({"artifact":name,"href":name,"expected_sha256":common::sha256_hex(bytes),
        "root_uuid":value[root]["uuid"],"document_version":value[root]["metadata"]["version"],
        "oscal_version":value[root]["metadata"]["oscal-version"]})
}

/// Construct a native five-artifact assessment bundle with explicit subject and result identities.
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

/// Invoke the actual scaffold command with explicit document, source and selected-result inputs.
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

/// Construct an explicitly unselected foundation fixture without remediation records.
fn scaffold(fixture: &Fixture) -> Value {
    let result = init(fixture, None);
    assert_eq!(result.status.code(), Some(0), "{}", String::from_utf8_lossy(&result.stderr));
    let value = serde_json::from_slice(&result.stdout).unwrap();
    std::fs::write(fixture.directory.path().join("poam.json"), result.stdout).unwrap();
    value
}

/// Invoke the actual source-only command on the private persisted scaffold.
fn check(fixture: &Fixture) -> Output {
    run(
        fixture.directory.path(),
        &["poam", "check", "--manifest", "poam.json", "--source-only", "--format", "json"],
    )
}

/// Mutate a synthetic source and update only its exact file pin for negative qualification controls.
fn ar_mutate(fixture: &Fixture, mutate: impl FnOnce(&mut Value)) {
    let path = fixture.directory.path().join("assessment-results.json");
    let mut value: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    mutate(&mut value);
    write_json(&path, &value);
}

/// Verify that scaffold is explicit empty deterministic and preserves sources.
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

/// Verify that source inventory preserves all denominators and omits sensitive content.
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

/// Verify that check requires an explicit scope and build requires explicit date and output arguments.
#[test]
fn check_requires_explicit_scope_and_build_requires_complete_arguments() {
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

/// Verify that exact selection tuple is required by library.
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

/// Verify that stale or misbound source pins fail before output.
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

/// Verify that explicit result key and uuid must match same epoch.
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

/// Verify that stale source context assertions are rejected even with fresh file hash.
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

/// Verify that duplicate stable keys and uuids are not accepted as new source.
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

/// Verify that accepted source relocation preserves scaffold and inventory bytes.
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

/// Verify that source revalidation detects changed bytes and same byte replacement.
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

/// Verify that scaffold publication never overwrites existing or source files.
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

/// Verify that symlink hardlink parent alias and special inputs fail closed.
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

/// Verify that native poam validation detects model and enforces system identity choice.
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

/// Verify that source property namespaces are not interchangeable.
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

/// Verify that directly constructed cli cannot bypass source only acknowledgement.
#[test]
fn directly_constructed_cli_cannot_bypass_source_only_acknowledgement() {
    let cli = forge::cli::Cli {
        command: forge::cli::Commands::Poam {
            command: forge::cli::PoamCommand::Check {
                manifest: "missing.json".into(),
                source_only: false,
                workflow: false,
                as_of: None,
                due_soon_days: None,
                baseline: None,
                report: None,
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

/// Create a second synthetic result epoch with independently rewritten native identities.
fn distinct_epoch(original: &Value) -> Value {
    use std::collections::BTreeMap;
    /// Collect fixture object UUIDs before constructing a distinct result epoch.
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
    /// Rewrite the synthetic epoch identity and reference graph consistently.
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

/// Verify that explicit result can be second and cross epoch references fail.
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

/// Verify that every companion identity and type pin is verified.
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

/// Verify that missing duplicate partial and stale native context receipts fail.
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

/// Verify that nested context receipt href must resolve from ar directory.
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

/// Verify that freshly pinned ar rejects invalid structure scope and references before output.
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

/// Verify that wrong companion root with fresh hash is rejected before output.
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

/// Verify that text inventory preserves complete counts and source only scope.
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

/// Verify actual nested AR receipts reach exact companions and reject a byte-identical decoy.
#[test]
#[allow(
    clippy::too_many_lines,
    reason = "one producer-to-consumer control preserves the full nested import chain and wrong-target refusal"
)]
fn produced_nested_context_receipts_pass_source_only_check() {
    let f = fixture();
    let root = f.directory.path();
    let mut plan = scaffold(&f);
    let native = root.join("native");
    let companions_dir = native.join("companions");
    std::fs::create_dir_all(&companions_dir).unwrap();
    let companions = [
        ("assessment_plan", "assessment-plan.json"),
        ("ssp", "ssp.json"),
        ("profile", "profile.json"),
        ("catalog", "catalog.json"),
    ];
    let original: Vec<_> = companions
        .iter()
        .map(|(_, filename)| std::fs::read(root.join(filename)).unwrap())
        .collect();
    let mut ar_manifest: Value =
        serde_json::from_slice(&std::fs::read(root.join("ar-manifest.json")).unwrap()).unwrap();
    for (field, filename) in companions {
        std::fs::rename(root.join(filename), companions_dir.join(filename)).unwrap();
        ar_manifest["context"][field]["artifact"] = json!(format!("companions/{filename}"));
        plan["source"]["context"][field]["artifact"] =
            json!(format!("native/companions/{filename}"));
    }
    ar_manifest["context"]["assessment_plan"]["href"] = json!("companions/assessment-plan.json");
    plan["source"]["context"]["assessment_plan"]["href"] = json!("companions/assessment-plan.json");
    write_json(&native.join("ar-manifest.json"), &ar_manifest);
    let produced = run(
        root,
        &[
            "assessment",
            "results",
            "build",
            "--manifest",
            "native/ar-manifest.json",
            "--output",
            "native/assessment-results.json",
        ],
    );
    assert_eq!(produced.status.code(), Some(0), "{}", String::from_utf8_lossy(&produced.stderr));
    let ar_path = native.join("assessment-results.json");
    let bytes = std::fs::read(&ar_path).unwrap();
    let ar: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(ar["assessment-results"]["results"][0]["uuid"], f.result_uuid);
    plan["source"]["assessment_results"]["artifact"] = json!("native/assessment-results.json");
    plan["source"]["assessment_results"]["expected_sha256"] = json!(common::sha256_hex(&bytes));
    write_json(&root.join("poam.json"), &plan);
    let parsed = manifest::parse(&serde_json::to_vec(&plan).unwrap()).unwrap();
    forge::assessment_results::context::load(&root.join("poam.json"), &parsed.source.context)
        .unwrap();
    let accepted = check(&f);
    assert_eq!(accepted.status.code(), Some(0), "{}", String::from_utf8_lossy(&accepted.stderr));
    let report: Value = serde_json::from_slice(&accepted.stdout).unwrap();
    assert_eq!(report["validation_scope"], "source-integrity-only");
    assert_eq!(report["workflow_validated"], false);
    assert_eq!(report["objects"].as_array().unwrap().len(), 3);
    assert_eq!(std::fs::read(&ar_path).unwrap(), bytes);
    for ((_, filename), original_bytes) in companions.iter().zip(&original) {
        assert_eq!(std::fs::read(companions_dir.join(filename)).unwrap(), *original_bytes);
    }
    std::fs::copy(companions_dir.join("ssp.json"), native.join("ssp.json")).unwrap();
    let mut wrong = ar;
    let receipt = wrong["assessment-results"]["back-matter"]["resources"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|receipt| {
            receipt["props"].as_array().unwrap().iter().any(|prop| {
                prop["name"] == "context-kind" && prop["value"] == "system-security-plan"
            })
        })
        .unwrap();
    receipt["rlinks"][0]["href"] = json!("ssp.json");
    let wrong_bytes = write_json(&ar_path, &wrong);
    plan["source"]["assessment_results"]["expected_sha256"] =
        json!(common::sha256_hex(&wrong_bytes));
    write_json(&root.join("poam.json"), &plan);
    let rejected = check(&f);
    assert_eq!(rejected.status.code(), Some(2));
    assert_eq!(rejected.stdout, [] as [u8; 0]);
    assert!(
        String::from_utf8_lossy(&rejected.stderr)
            .contains("AR context resource link resolves to a different confined companion")
    );
    assert_eq!(std::fs::read(&ar_path).unwrap(), wrong_bytes);
}

/// Author a nonterminal item from an exact actually captured source inventory tuple.
fn authored_workflow(fixture: &Fixture) -> Value {
    let mut value = scaffold(fixture);
    let source_manifest: manifest::PoamManifest = serde_json::from_value(value.clone()).unwrap();
    let prepared =
        source::load(&fixture.directory.path().join("poam.json"), &source_manifest.source).unwrap();
    let selected = prepared
        .inventory()
        .objects
        .iter()
        .find(|object| object.key == "finding-unsatisfied")
        .unwrap();
    value["document"]["last_modified"] = json!("2026-02-20T00:00:00Z");
    value["roles"] = json!([{"id":"owner","title":"Synthetic owner"}]);
    value["parties"] =
        json!([{"key":"remediator","type":"person","name":"SENSITIVE REMEDIATOR NAME"}]);
    let owner = json!({"role_id":"owner","party_key":"remediator","rationale":"SENSITIVE OWNERSHIP RATIONALE"});
    let initial = json!({"key":"plan","actor":{"role_id":"owner","party_key":"remediator"},"at":"2026-01-03T00:00:00Z",
        "from":null,"to":"planned","rationale":"SENSITIVE PLAN RATIONALE","closure":null});
    value["items"] = json!([{"key":"work","title":"Explicit authored work","description":"Explicit authored remediation outcome",
        "source_refs":[{"kind":selected.kind,"key":selected.key,"uuid":selected.uuid,"result_uuid":selected.result_uuid,"expected_sha256":selected.sha256}],
        "owners":[owner],"target_date":"2026-02-10","state":"planned","history":[initial],
        "milestones":[{"key":"step","outcome":"Explicit authored milestone outcome","target_date":"2026-02-05","depends_on":[],
            "owners":[owner],"state":"planned","history":[initial]}]}]);
    value
}

/// Prepare against the actual native five-file fixture without adding CLI publication behavior.
fn prepare_authored(
    fixture: &Fixture,
    value: &Value,
    baseline: Option<&[u8]>,
) -> Result<forge::poam::workflow::PreparedWorkflow, forge::ForgeError> {
    forge::poam::workflow::prepare(
        &fixture.directory.path().join("poam.json"),
        &serde_json::to_vec(value).unwrap(),
        "2026-02-06",
        7,
        baseline,
    )
}

/// Actual native AR/companion capture feeds deterministic nonempty typed artifact and complete minimized schedule.
#[test]
fn authored_item_preparation_is_native_source_bound_deterministic_and_nonmutating() {
    let f = fixture();
    let before = [
        "assessment-results.json",
        "assessment-plan.json",
        "ssp.json",
        "profile.json",
        "catalog.json",
    ]
    .map(|name| (name, std::fs::read(f.directory.path().join(name)).unwrap()));
    let value = authored_workflow(&f);
    let one = prepare_authored(&f, &value, None).unwrap();
    let two = prepare_authored(&f, &value, None).unwrap();
    assert_eq!(one.artifact(), two.artifact());
    assert_eq!(one.schedule(), two.schedule());
    assert!(one.review_required());
    one.verify_inputs().unwrap();
    let artifact: Value = serde_json::from_slice(one.artifact()).unwrap();
    assert_eq!(
        artifact["plan-of-action-and-milestones"]["poam-items"].as_array().unwrap().len(),
        1
    );
    let schema: Value =
        serde_json::from_str(include_str!("../schemas/oscal_poam_schema.json")).unwrap();
    assert!(jsonschema::validator_for(&schema).unwrap().is_valid(&artifact));
    let report: Value = serde_json::from_slice(one.schedule()).unwrap();
    assert_eq!(report["source_objects"], 3);
    assert_eq!(report["items"], 1);
    assert_eq!(report["milestones"], 1);
    assert_eq!(report["rows"].as_array().unwrap().len(), 2);
    assert_eq!(report["overdue"], 1);
    assert_eq!(report["due_soon"], 1);
    let text = String::from_utf8(one.schedule().to_vec()).unwrap();
    for private in ["SENSITIVE", "remediator", "assessment-results.json", "rationale"] {
        assert!(!text.contains(private));
    }
    let native = String::from_utf8(one.artifact().to_vec()).unwrap();
    for source_prose in [
        "SENSITIVE OBSERVATION",
        "SENSITIVE FINDING DESCRIPTION",
        "SENSITIVE SOURCE STATEMENT",
        "SENSITIVE RISK STATEMENT",
    ] {
        assert!(!native.contains(source_prose));
    }
    for (name, bytes) in before {
        assert_eq!(std::fs::read(f.directory.path().join(name)).unwrap(), bytes);
    }
    assert!(!f.directory.path().join("built-poam.json").exists());
}

/// Each source tuple member is checked against actual inventory rather than UUID-only or producer digest shortcuts.
#[test]
fn authored_item_refuses_every_stale_or_cross_domain_source_tuple_before_output() {
    let f = fixture();
    let value = authored_workflow(&f);
    for (field, replacement) in [
        ("key", json!("missing")),
        ("kind", json!("risk")),
        ("uuid", json!("99999999-9999-4999-8999-999999999999")),
        ("result_uuid", json!("88888888-8888-4888-8888-888888888888")),
        ("expected_sha256", json!("a".repeat(64))),
    ] {
        let mut copy = value.clone();
        copy["items"][0]["source_refs"][0][field] = replacement;
        assert!(prepare_authored(&f, &copy, None).is_err(), "{field}");
    }
    assert!(!f.directory.path().join("built-poam.json").exists());
}

/// A captured companion's changed raw bytes cannot receive a plan merely because item UUIDs still exist.
#[test]
fn authored_item_refuses_changed_native_companion_and_retains_other_sources() {
    let f = fixture();
    let value = authored_workflow(&f);
    let ar = std::fs::read(f.directory.path().join("assessment-results.json")).unwrap();
    let path = f.directory.path().join("catalog.json");
    let original = std::fs::read(&path).unwrap();
    let mut changed = original.clone();
    changed.extend_from_slice(b" ");
    std::fs::write(&path, &changed).unwrap();
    assert!(prepare_authored(&f, &value, None).is_err());
    assert_eq!(std::fs::read(f.directory.path().join("assessment-results.json")).unwrap(), ar);
    std::fs::write(&path, &original).unwrap();
    assert!(prepare_authored(&f, &value, None).is_ok());
}

/// Original captured input generations are rechecked immediately before the Root-owned output port.
#[test]
fn prepared_authored_item_detects_source_change_before_publication() {
    let f = fixture();
    let value = authored_workflow(&f);
    let prepared = prepare_authored(&f, &value, None).unwrap();
    let path = f.directory.path().join("assessment-results.json");
    let original = std::fs::read(&path).unwrap();
    std::fs::write(&path, b"{}").unwrap();
    assert!(prepared.verify_inputs().is_err());
    std::fs::write(&path, &original).unwrap();
    assert!(prepared.verify_inputs().is_ok());
    assert!(!f.directory.path().join("built-poam.json").exists());
}

/// Explicit prior authoring bytes guard exact history prefixes without asserting a full baseline impact report.
#[test]
fn actual_source_plan_accepts_nonterminal_append_and_refuses_rewritten_prior_event() {
    let f = fixture();
    let old = authored_workflow(&f);
    let baseline = serde_json::to_vec(&old).unwrap();
    let mut next = old;
    next["items"][0]["history"].as_array_mut().unwrap().push(json!({"key":"start","actor":{"role_id":"owner","party_key":"remediator"},
        "at":"2026-01-04T00:00:00Z","from":"planned","to":"in-progress","rationale":"Explicit underway assertion","closure":null}));
    next["items"][0]["state"] = json!("in-progress");
    assert!(prepare_authored(&f, &next, Some(&baseline)).is_ok());
    next["items"][0]["history"][0]["rationale"] = json!("Overwritten original rationale");
    assert!(prepare_authored(&f, &next, Some(&baseline)).is_err());
}

/// No implicit remediation is selected and original source-only empty scaffolds retain their own contract.
#[test]
fn authored_parser_does_not_turn_foundation_scaffold_into_selected_work() {
    let f = fixture();
    let value = scaffold(&f);
    let bytes = serde_json::to_vec(&value).unwrap();
    assert!(manifest::parse(&bytes).is_ok());
    assert!(forge::poam::workflow::parse(&bytes).is_err());
    assert!(
        forge::poam::workflow::prepare(
            &f.directory.path().join("poam.json"),
            &bytes,
            "2026-02-06",
            7,
            None
        )
        .is_err()
    );
}

/// Persist the exact authored fixture and return original declared input bytes for nonmutation assertions.
fn install_authored_cli(fixture: &Fixture) -> Vec<(String, Vec<u8>)> {
    let value = authored_workflow(fixture);
    write_json(&fixture.directory.path().join("poam.json"), &value);
    [
        "poam.json",
        "assessment-results.json",
        "assessment-plan.json",
        "ssp.json",
        "profile.json",
        "catalog.json",
    ]
    .into_iter()
    .map(|name| (name.to_string(), std::fs::read(fixture.directory.path().join(name)).unwrap()))
    .collect()
}

/// Reconcile the complete original synthetic source/manifests after real command observations.
fn assert_authored_cli_inputs(fixture: &Fixture, inputs: &[(String, Vec<u8>)]) {
    for (name, bytes) in inputs {
        assert_eq!(
            &std::fs::read(fixture.directory.path().join(name)).unwrap(),
            bytes,
            "input {name}"
        );
    }
}

/// Exercise real clap admission without any hidden schedule clock or ambiguous scope fallback.
#[test]
fn authored_cli_requires_date_output_and_exclusive_explicit_scope() {
    use clap::Parser as _;
    for args in [
        vec!["forge", "poam", "build", "--manifest", "poam.json", "--output", "native.json"],
        vec!["forge", "poam", "build", "--manifest", "poam.json", "--as-of", "2026-01-04"],
        vec!["forge", "poam", "check", "--manifest", "poam.json", "--workflow"],
        vec![
            "forge",
            "poam",
            "check",
            "--manifest",
            "poam.json",
            "--workflow",
            "--source-only",
            "--as-of",
            "2026-01-04",
        ],
        vec![
            "forge",
            "poam",
            "check",
            "--manifest",
            "poam.json",
            "--source-only",
            "--as-of",
            "2026-01-04",
        ],
        vec![
            "forge",
            "poam",
            "check",
            "--manifest",
            "poam.json",
            "--source-only",
            "--due-soon-days",
            "0",
        ],
        vec![
            "forge",
            "poam",
            "check",
            "--manifest",
            "poam.json",
            "--source-only",
            "--baseline",
            "prior.json",
        ],
        vec![
            "forge",
            "poam",
            "check",
            "--manifest",
            "poam.json",
            "--source-only",
            "--report",
            "report.json",
        ],
        vec![
            "forge",
            "poam",
            "build",
            "--manifest",
            "poam.json",
            "--as-of",
            "2026-01-04",
            "--output",
            "native.json",
            "--due-soon-days",
            "366",
        ],
    ] {
        assert!(
            forge::cli::Cli::try_parse_from(args.clone()).is_err(),
            "unexpected clap admission: {args:?}"
        );
    }
    assert!(
        forge::cli::Cli::try_parse_from([
            "forge",
            "poam",
            "check",
            "--manifest",
            "poam.json",
            "--source-only"
        ])
        .is_ok()
    );
    assert!(
        forge::cli::Cli::try_parse_from([
            "forge",
            "poam",
            "check",
            "--manifest",
            "poam.json",
            "--workflow",
            "--as-of",
            "2026-01-04"
        ])
        .is_ok()
    );
}

/// Actual workflow check emits complete minimized schedule and exit1 without changing any input or making a native file.
#[test]
fn authored_cli_schedule_action_is_valid_exit_one_with_complete_counts() {
    let fixture = fixture();
    let inputs = install_authored_cli(&fixture);
    let root = fixture.directory.path();
    let output = run(
        root,
        &[
            "poam",
            "check",
            "--workflow",
            "--manifest",
            "poam.json",
            "--as-of",
            "2026-02-06",
            "--due-soon-days",
            "7",
            "--format",
            "json",
        ],
    );
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stderr, [] as [u8; 0]);
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["source_objects"], 3);
    assert_eq!(report["items"], 1);
    assert_eq!(report["milestones"], 1);
    assert_eq!(report["rows"].as_array().unwrap().len(), 2);
    assert_eq!(report["overdue"], 1);
    assert_eq!(report["due_soon"], 1);
    let text = String::from_utf8(output.stdout).unwrap();
    for private in ["SENSITIVE", "remediator", "rationale", "ssp.json", root.to_str().unwrap()] {
        assert!(!text.contains(private));
    }
    assert_authored_cli_inputs(&fixture, &inputs);
    assert!(!root.join("native.json").exists());
    let quiet = run(
        root,
        &[
            "poam",
            "check",
            "--workflow",
            "--manifest",
            "poam.json",
            "--as-of",
            "2026-01-04",
            "--format",
            "text",
        ],
    );
    assert_eq!(quiet.status.code(), Some(0));
    assert!(quiet.stdout.is_ascii());
    assert_eq!(quiet.stderr, [] as [u8; 0]);
}

/// Invalid full dates, stale source hashes and unconfined baselines fail exit2 before any report output.
#[test]
fn authored_cli_invalid_or_stale_inputs_never_earn_schedule_action_credit() {
    let fixture = fixture();
    install_authored_cli(&fixture);
    let root = fixture.directory.path();
    for date in ["2026-02-30", "2026-2-06", "2026-02-06T00:00:00Z"] {
        let output = run(
            root,
            &[
                "poam",
                "check",
                "--workflow",
                "--manifest",
                "poam.json",
                "--as-of",
                date,
                "--format",
                "json",
            ],
        );
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(output.stdout, [] as [u8; 0]);
    }
    let output = run(
        root,
        &[
            "poam",
            "check",
            "--workflow",
            "--manifest",
            "poam.json",
            "--as-of",
            "2026-01-04",
            "--baseline",
            "../prior.json",
        ],
    );
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stdout, [] as [u8; 0]);
    std::fs::write(root.join("assessment-results.json"), b"changed actual native bytes").unwrap();
    let output = run(
        root,
        &[
            "poam",
            "check",
            "--workflow",
            "--manifest",
            "poam.json",
            "--as-of",
            "2026-01-04",
            "--format",
            "json",
        ],
    );
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stdout, [] as [u8; 0]);
}

/// Current captured prior declaration admits append-only nonterminal history but refuses a rewritten event.
#[test]
fn authored_cli_consumes_explicit_baseline_and_refuses_prior_history_rewrite() {
    let fixture = fixture();
    install_authored_cli(&fixture);
    let root = fixture.directory.path();
    let original = std::fs::read(root.join("poam.json")).unwrap();
    std::fs::write(root.join("prior.json"), &original).unwrap();
    let mut next: Value = serde_json::from_slice(&original).unwrap();
    next["items"][0]["state"] = json!("in-progress");
    next["items"][0]["history"].as_array_mut().unwrap().push(json!({"key":"begin", "actor":{"role_id":"owner", "party_key":"remediator"},
        "at":"2026-01-04T00:00:00Z", "from":"planned", "to":"in-progress", "rationale":"Explicit new assertion", "closure":null}));
    write_json(&root.join("poam.json"), &next);
    let accepted = run(
        root,
        &[
            "poam",
            "check",
            "--workflow",
            "--manifest",
            "poam.json",
            "--baseline",
            "prior.json",
            "--as-of",
            "2026-01-04",
            "--format",
            "json",
        ],
    );
    assert_eq!(accepted.status.code(), Some(0));
    assert_ne!(accepted.stdout, [] as [u8; 0]);
    next["items"][0]["history"][0]["rationale"] = json!("Rewritten old event");
    write_json(&root.join("poam.json"), &next);
    let refused = run(
        root,
        &[
            "poam",
            "check",
            "--workflow",
            "--manifest",
            "poam.json",
            "--baseline",
            "prior.json",
            "--as-of",
            "2026-01-04",
            "--format",
            "json",
        ],
    );
    assert_eq!(refused.status.code(), Some(2));
    assert_eq!(refused.stdout, [] as [u8; 0]);
    assert_eq!(std::fs::read(root.join("prior.json")).unwrap(), original);
}

/// Existing init/source-only requests remain independent of authored selection and workflow schedule options.
#[test]
fn authored_cli_does_not_silently_reinterpret_foundation_source_only_scope() {
    let fixture = fixture();
    scaffold(&fixture);
    let root = fixture.directory.path();
    assert_eq!(check(&fixture).status.code(), Some(0));
    let value = authored_workflow(&fixture);
    write_json(&root.join("poam.json"), &value);
    let source_only = check(&fixture);
    assert_eq!(source_only.status.code(), Some(2));
    assert_eq!(source_only.stdout, [] as [u8; 0]);
    let workflow = run(
        root,
        &[
            "poam",
            "check",
            "--manifest",
            "poam.json",
            "--workflow",
            "--as-of",
            "2026-01-04",
            "--format",
            "json",
        ],
    );
    assert_eq!(workflow.status.code(), Some(0));
}

/// Adequate proposed risk-review fields remain refused by the public shipping boundary and never change native source assertions.
#[test]
fn authored_cli_has_no_terminal_closure_override() {
    let fixture = fixture();
    let mut value = authored_workflow(&fixture);
    let root = fixture.directory.path();
    let foundation: manifest::PoamManifest = serde_json::from_value(scaffold(&fixture)).unwrap();
    let prepared = source::load(&root.join("poam.json"), &foundation.source).unwrap();
    let risk = prepared
        .inventory()
        .objects
        .iter()
        .find(|object| object.kind == manifest::SourceKind::Risk)
        .unwrap();
    value["items"][0]["source_refs"] = json!([{"kind":risk.kind, "key":risk.key, "uuid":risk.uuid, "result_uuid":risk.result_uuid, "expected_sha256":risk.sha256}]);
    value["roles"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"reviewer", "title":"Synthetic reviewer"}));
    value["parties"]
        .as_array_mut()
        .unwrap()
        .push(json!({"key":"reviewer", "type":"person", "name":"Synthetic reviewer"}));
    value["items"][0]["state"] = json!("accepted-risk-asserted");
    value["items"][0]["history"].as_array_mut().unwrap().push(json!({"key":"accept", "actor":{"role_id":"owner", "party_key":"remediator"},
        "at":"2026-01-06T00:00:00Z", "from":"planned", "to":"accepted-risk-asserted", "rationale":"Proposed explicit risk assertion",
        "closure":{"reviewer":{"role_id":"reviewer", "party_key":"reviewer"}, "reviewed_at":"2026-01-07T00:00:00Z",
        "rationale":"Proposed review only", "evidence":[{"key":"assertion", "href":"evidence.json", "expected_sha256":"a".repeat(64)}]}}));
    write_json(&root.join("poam.json"), &value);
    let inputs = ["poam.json", "assessment-results.json"]
        .map(|name| (name.to_string(), std::fs::read(root.join(name)).unwrap()));
    let output = run(
        root,
        &[
            "poam",
            "check",
            "--workflow",
            "--manifest",
            "poam.json",
            "--as-of",
            "2026-02-06",
            "--format",
            "json",
        ],
    );
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stdout, [] as [u8; 0]);
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("pending recorded closure disposition")
    );
    assert_authored_cli_inputs(&fixture, &inputs);
}

/// New native and report files are deterministic, schema-valid and based at the actual manifest directory.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn authored_cli_build_publishes_explicit_new_files_without_source_mutation() {
    let fixture = fixture();
    let inputs = install_authored_cli(&fixture);
    let root = fixture.directory.path();
    let args = [
        "poam",
        "build",
        "--manifest",
        "poam.json",
        "--as-of",
        "2026-01-04",
        "--output",
        "native.json",
        "--report",
        "schedule.json",
    ];
    let output = run(root, &args);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, [] as [u8; 0]);
    assert_eq!(output.stderr, [] as [u8; 0]);
    let artifact_bytes = std::fs::read(root.join("native.json")).unwrap();
    let native: Value = serde_json::from_slice(&artifact_bytes).unwrap();
    let schema: Value =
        serde_json::from_str(include_str!("../schemas/oscal_poam_schema.json")).unwrap();
    assert!(jsonschema::validator_for(&schema).unwrap().is_valid(&native));
    assert_eq!(native["plan-of-action-and-milestones"]["import-ssp"]["href"], "ssp.json");
    assert_eq!(
        native["plan-of-action-and-milestones"]["poam-items"][0]["links"][0]["href"]
            .as_str()
            .unwrap()
            .split('#')
            .next()
            .unwrap(),
        "assessment-results.json"
    );
    let report: Value =
        serde_json::from_slice(&std::fs::read(root.join("schedule.json")).unwrap()).unwrap();
    assert_eq!(report["as_of"], "2026-01-04");
    assert_authored_cli_inputs(&fixture, &inputs);
    let output = run(root, &args);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stdout, [] as [u8; 0]);
    assert_eq!(std::fs::read(root.join("native.json")).unwrap(), artifact_bytes);
}

/// Artifact location follows the captured manifest root, never the invoker's unrelated current directory.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn authored_cli_absolute_manifest_does_not_invent_output_href_relocation() {
    let fixture = fixture();
    install_authored_cli(&fixture);
    let root = fixture.directory.path();
    let outside = tempfile::tempdir().unwrap();
    let manifest_path = root.join("poam.json");
    let output = run(
        outside.path(),
        &[
            "poam",
            "build",
            "--manifest",
            manifest_path.to_str().unwrap(),
            "--as-of",
            "2026-01-04",
            "--output",
            "native.json",
        ],
    );
    assert_eq!(output.status.code(), Some(0));
    assert!(!outside.path().join("native.json").exists());
    assert!(root.join("native.json").is_file());
    let relocated = run(
        root,
        &[
            "poam",
            "build",
            "--manifest",
            "poam.json",
            "--as-of",
            "2026-01-04",
            "--output",
            "nested/native.json",
        ],
    );
    assert_eq!(relocated.status.code(), Some(2));
    assert_eq!(relocated.stdout, [] as [u8; 0]);
    assert!(!root.join("nested").exists());
}

/// All output names are checked before preparation, so a known report conflict cannot leave a newly-created artifact.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn authored_cli_known_report_conflict_and_output_alias_fail_before_artifact_write() {
    let fixture = fixture();
    install_authored_cli(&fixture);
    let root = fixture.directory.path();
    std::fs::write(root.join("existing.json"), b"sentinel").unwrap();
    for report in [
        "existing.json",
        "native.json",
        "NATIVE.json",
        "../report.json",
        "NUL.json",
        "bad:name.json",
        "unicode-é.json",
        "POAM.json",
        "ASSESSMENT-RESULTS.json",
    ] {
        let output = run(
            root,
            &[
                "poam",
                "build",
                "--manifest",
                "poam.json",
                "--as-of",
                "2026-01-04",
                "--output",
                "native.json",
                "--report",
                report,
            ],
        );
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(output.stdout, [] as [u8; 0]);
        assert!(!root.join("native.json").exists());
    }
    let overlong = format!("{}.json", "x".repeat(124));
    let output = run(
        root,
        &[
            "poam",
            "build",
            "--manifest",
            "poam.json",
            "--as-of",
            "2026-01-04",
            "--output",
            "native.json",
            "--report",
            &overlong,
        ],
    );
    assert_eq!(output.status.code(), Some(2));
    assert!(!root.join("native.json").exists());
    std::fs::copy(root.join("poam.json"), root.join("prior.json")).unwrap();
    let output = run(
        root,
        &[
            "poam",
            "build",
            "--manifest",
            "poam.json",
            "--as-of",
            "2026-01-04",
            "--baseline",
            "prior.json",
            "--output",
            "native.json",
            "--report",
            "PRIOR.json",
        ],
    );
    assert_eq!(output.status.code(), Some(2));
    assert!(!root.join("native.json").exists());
    assert_eq!(std::fs::read(root.join("existing.json")).unwrap(), b"sentinel");
}
