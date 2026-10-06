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

/// Actual native source epochs classify prior not-current separately from invalid prior history or authority.
#[test]
fn baseline_comparison_uses_actual_current_native_generation_without_prior_freshness_claim() {
    let f = fixture();
    let previous = authored_workflow(&f);
    let prior_bytes = serde_json::to_vec(&previous).unwrap();
    ar_mutate(&f, |ar| {
        ar["assessment-results"]["results"][0]["findings"][1]["description"] =
            json!("SENSITIVE changed native finding prose");
    });
    let current = authored_workflow(&f);
    let current_bytes = serde_json::to_vec(&current).unwrap();
    let prepared = forge::poam::workflow_baseline::prepare(
        &f.directory.path().join("poam.json"),
        &current_bytes,
        &prior_bytes,
        None,
        "2026-02-06",
    )
    .unwrap();
    prepared.verify_inputs().unwrap();
    let report: Value = serde_json::from_slice(prepared.report()).unwrap();
    assert_eq!(report["current_refs_not_current"], 0);
    assert_eq!(report["previous_refs_not_current"], 1);
    assert_eq!(report["previous_validation"], "bounded-structural-history-only");
    assert_eq!(report["source_declaration_changed"], true);
    assert_eq!(report["artifact_validated"], false);
    assert_eq!(report["previous_manifest_sha256"], common::sha256_hex(&prior_bytes));
    assert_eq!(report["current_manifest_sha256"], common::sha256_hex(&current_bytes));
    assert!(!String::from_utf8(prepared.report().to_vec()).unwrap().contains("SENSITIVE"));
    assert!(!f.directory.path().join("built-poam.json").exists());
}

/// A readonly comparison retains/rechecks all five actual inputs and does not fabricate artifact publication.
#[test]
fn baseline_comparison_retains_real_source_generations_and_refuses_drift() {
    let f = fixture();
    let value = authored_workflow(&f);
    let bytes = serde_json::to_vec(&value).unwrap();
    let prepared = forge::poam::workflow_baseline::prepare(
        &f.directory.path().join("poam.json"),
        &bytes,
        &bytes,
        None,
        "2026-02-06",
    )
    .unwrap();
    prepared.verify_inputs().unwrap();
    assert!(prepared.revision_rules_compatible());
    let path = f.directory.path().join("catalog.json");
    let original = std::fs::read(&path).unwrap();
    let mut changed = original.clone();
    changed.extend_from_slice(b" ");
    std::fs::write(&path, &changed).unwrap();
    assert!(prepared.verify_inputs().is_err());
    assert!(!f.directory.path().join("built-poam.json").exists());
    std::fs::write(&path, &original).unwrap();
    assert!(prepared.verify_inputs().is_ok());
}

/// Install explicit current/prior authored declarations and retain their actual source bytes.
fn install_baseline_cli(fixture: &Fixture) -> Vec<(String, Vec<u8>)> {
    let mut inputs = install_authored_cli(fixture);
    let prior = std::fs::read(fixture.directory.path().join("poam.json")).unwrap();
    std::fs::write(fixture.directory.path().join("prior.json"), &prior).unwrap();
    inputs.push(("prior.json".to_string(), prior));
    inputs
}

/// Invoke the proposed readonly command with explicit date and two declaration filenames.
fn baseline_cli(fixture: &Fixture, extra: &[&str]) -> Output {
    let mut args = vec![
        "poam",
        "baseline",
        "--manifest",
        "poam.json",
        "--baseline",
        "prior.json",
        "--as-of",
        "2026-02-06",
    ];
    args.extend_from_slice(extra);
    run(fixture.directory.path(), &args)
}

/// Actual clap requires every comparison input/date and exposes no artifact or override options.
#[test]
fn baseline_cli_requires_explicit_inputs_date_and_readonly_scope() {
    let fixture = fixture();
    install_baseline_cli(&fixture);
    for args in [
        vec!["poam", "baseline"],
        vec!["poam", "baseline", "--manifest", "poam.json", "--baseline", "prior.json"],
        vec!["poam", "baseline", "--manifest", "poam.json", "--as-of", "2026-02-06"],
        vec!["poam", "baseline", "--baseline", "prior.json", "--as-of", "2026-02-06"],
    ] {
        let result = run(fixture.directory.path(), &args);
        assert_eq!(result.status.code(), Some(2), "{args:?}");
        assert!(result.stdout.is_empty(), "{args:?}");
    }
    for extra in [
        vec!["--output", "artifact.json"],
        vec!["--workflow"],
        vec!["--source-only"],
        vec!["--due-soon-days", "7"],
        vec!["--allow-terminal"],
    ] {
        let result = baseline_cli(&fixture, &extra);
        assert_eq!(result.status.code(), Some(2), "{extra:?}");
        assert!(result.stdout.is_empty(), "{extra:?}");
    }
    assert!(!fixture.directory.path().join("artifact.json").exists());
}

/// An unchanged actual five-source comparison is deterministic exit0 and publishes only a new report.
#[test]
fn baseline_cli_unchanged_report_is_complete_deterministic_and_nonmutating() {
    let fixture = fixture();
    let inputs = install_baseline_cli(&fixture);
    let first = baseline_cli(&fixture, &[]);
    let second = baseline_cli(&fixture, &[]);
    assert_eq!(first.status.code(), Some(0), "{}", String::from_utf8_lossy(&first.stderr));
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(first.stdout, second.stdout);
    let report: Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(report["schema_version"], "forge.poam-baseline/1");
    assert_eq!(report["artifact_validated"], false);
    assert_eq!(report["revision_rules_compatible"], true);
    assert_eq!(
        report["items"],
        json!({"previous":1,"current":1,"common":1,"added":0,"removed":0,"changed":0})
    );
    assert_eq!(report["milestones"], report["items"]);
    assert_eq!(report["rows"].as_array().unwrap().len(), 2);
    assert_eq!(report["references"].as_array().unwrap().len(), 2);
    assert_eq!(report["previous_manifest_sha256"], report["current_manifest_sha256"]);
    assert_eq!(report["previous_validation"], "bounded-structural-history-only");
    assert!(!String::from_utf8(first.stdout.clone()).unwrap().contains("SENSITIVE"));
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let published = baseline_cli(&fixture, &["--report", "baseline-report.json"]);
        assert_eq!(published.status.code(), Some(0));
        assert_eq!(published.stdout, [] as [u8; 0]);
        assert_eq!(
            std::fs::read(fixture.directory.path().join("baseline-report.json")).unwrap(),
            first.stdout
        );
        let refused = baseline_cli(&fixture, &["--report", "baseline-report.json"]);
        assert_eq!(refused.status.code(), Some(2));
        assert_eq!(refused.stdout, [] as [u8; 0]);
        assert_eq!(
            std::fs::read(fixture.directory.path().join("baseline-report.json")).unwrap(),
            first.stdout
        );
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let root = fixture.directory.path();
        let entries_before = std::fs::read_dir(root).unwrap().count();
        let refused = baseline_cli(&fixture, &["--report", "baseline-report.json"]);
        assert_eq!(refused.status.code(), Some(2), "{}", String::from_utf8_lossy(&refused.stderr));
        assert_eq!(refused.stdout, [] as [u8; 0]);
        assert!(
            String::from_utf8_lossy(&refused.stderr).contains("cannot publish new workflow output")
        );
        assert!(!root.join("baseline-report.json").exists());
        assert_eq!(std::fs::read_dir(root).unwrap().count(), entries_before);
    }
    assert_authored_cli_inputs(&fixture, &inputs);
}

/// Compatible owner/date/outcome changes still request review rather than earning unchanged exit0.
#[test]
fn baseline_cli_compatible_declared_changes_are_valid_exit_one() {
    let fixture = fixture();
    install_baseline_cli(&fixture);
    let root = fixture.directory.path();
    let mut current: Value =
        serde_json::from_slice(&std::fs::read(root.join("poam.json")).unwrap()).unwrap();
    current["items"][0]["owners"][0]["rationale"] =
        json!("Changed sensitive responsibility rationale");
    current["items"][0]["target_date"] = json!("2026-02-11");
    current["items"][0]["description"] = json!("Changed sensitive outcome");
    current["items"][0]["milestones"][0]["outcome"] = json!("Changed sensitive milestone outcome");
    let bytes = write_json(&root.join("poam.json"), &current);
    let result = baseline_cli(&fixture, &[]);
    assert_eq!(result.status.code(), Some(1), "{}", String::from_utf8_lossy(&result.stderr));
    assert_eq!(result.stderr, [] as [u8; 0]);
    let report: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["revision_rules_compatible"], true);
    assert_eq!(report["items"]["changed"], 1);
    assert_eq!(report["milestones"]["changed"], 1);
    let item = report["rows"].as_array().unwrap().iter().find(|row| row["kind"] == "item").unwrap();
    for field in ["owners", "target-date", "outcome"] {
        assert!(item["changes"].as_array().unwrap().contains(&json!(field)), "{field}");
    }
    let text = String::from_utf8(result.stdout).unwrap();
    assert!(!text.contains("Changed sensitive"));
    assert_eq!(std::fs::read(root.join("poam.json")).unwrap(), bytes);
}

/// Complete removals remain observable while ordinary check/build continue to refuse them.
#[test]
fn baseline_cli_empty_current_preserves_removed_rows_without_build_admission() {
    let fixture = fixture();
    install_baseline_cli(&fixture);
    let root = fixture.directory.path();
    let mut current: Value =
        serde_json::from_slice(&std::fs::read(root.join("poam.json")).unwrap()).unwrap();
    current["items"] = json!([]);
    current["roles"] = json!([]);
    current["parties"] = json!([]);
    write_json(&root.join("poam.json"), &current);
    let result = baseline_cli(&fixture, &[]);
    assert_eq!(result.status.code(), Some(1), "{}", String::from_utf8_lossy(&result.stderr));
    let report: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["items"]["removed"], 1);
    assert_eq!(report["items"]["current"], 0);
    assert_eq!(report["milestones"]["removed"], 1);
    assert_eq!(report["rows"].as_array().unwrap().len(), 2);
    assert_eq!(report["revision_rules_compatible"], false);
    assert_eq!(report["artifact_validated"], false);
    let refusals = report["refusal_observations"].as_array().unwrap();
    assert!(refusals.contains(&json!("empty-current")));
    assert!(refusals.contains(&json!("removed-identity")));
    let check = run(
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
            "2026-02-06",
        ],
    );
    assert_eq!(check.status.code(), Some(2));
    assert_eq!(check.stdout, [] as [u8; 0]);
    let build = run(
        root,
        &[
            "poam",
            "build",
            "--manifest",
            "poam.json",
            "--baseline",
            "prior.json",
            "--as-of",
            "2026-02-06",
            "--output",
            "refused.json",
        ],
    );
    assert_eq!(build.status.code(), Some(2));
    assert!(!root.join("refused.json").exists());
}

/// Rewritten history is a valid comparison finding and remains invalid workflow admission.
#[test]
fn baseline_cli_reports_rewritten_history_without_relaxing_workflow_check() {
    let fixture = fixture();
    install_baseline_cli(&fixture);
    let root = fixture.directory.path();
    let mut current: Value =
        serde_json::from_slice(&std::fs::read(root.join("poam.json")).unwrap()).unwrap();
    current["items"][0]["history"][0]["rationale"] = json!("Rewritten sensitive event");
    write_json(&root.join("poam.json"), &current);
    let result = baseline_cli(&fixture, &[]);
    assert_eq!(result.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["revision_rules_compatible"], false);
    assert!(
        report["refusal_observations"].as_array().unwrap().contains(&json!("rewritten-history"))
    );
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
            "2026-02-06",
        ],
    );
    assert_eq!(refused.status.code(), Some(2));
    assert_eq!(refused.stdout, [] as [u8; 0]);
}

/// Stale selected tuples are complete comparison rows; invalid actual source capture is exit2.
#[test]
fn baseline_cli_distinguishes_stale_selection_from_invalid_source_capture() {
    let fixture = fixture();
    install_baseline_cli(&fixture);
    let root = fixture.directory.path();
    let mut current: Value =
        serde_json::from_slice(&std::fs::read(root.join("poam.json")).unwrap()).unwrap();
    current["items"][0]["source_refs"][0]["expected_sha256"] = json!("f".repeat(64));
    write_json(&root.join("poam.json"), &current);
    let result = baseline_cli(&fixture, &[]);
    assert_eq!(result.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["previous_refs_not_current"], 0);
    assert_eq!(report["current_refs_not_current"], 1);
    assert_eq!(report["references"].as_array().unwrap().len(), 2);
    assert_eq!(report["revision_rules_compatible"], false);
    assert!(
        report["refusal_observations"]
            .as_array()
            .unwrap()
            .contains(&json!("current-reference-stale"))
    );
    std::fs::write(root.join("assessment-results.json"), b"{}").unwrap();
    let invalid = baseline_cli(&fixture, &["--report", "invalid-source.json"]);
    assert_eq!(invalid.status.code(), Some(2));
    assert_eq!(invalid.stdout, [] as [u8; 0]);
    assert!(!root.join("invalid-source.json").exists());
}

/// Unsafe destinations, malformed auxiliary data and declaration aliases never publish reports.
#[test]
fn baseline_cli_confines_all_inputs_and_preflights_report_outputs() {
    let fixture = fixture();
    let inputs = install_baseline_cli(&fixture);
    let root = fixture.directory.path();
    for extra in [
        vec!["--report", "nested/report.json"],
        vec!["--report", "../report.json"],
        vec!["--report", "poam.json"],
        vec!["--report", "PRIOR.json"],
        vec!["--report", "SSP.json"],
        vec!["--report", "NUL.json"],
        vec!["--report", "report.txt"],
        vec!["--reopens", "../outside.json", "--report", "outside-report.json"],
    ] {
        let result = baseline_cli(&fixture, &extra);
        assert_eq!(result.status.code(), Some(2), "{extra:?}");
        assert!(result.stdout.is_empty(), "{extra:?}");
    }
    std::fs::write(root.join("reopens.json"), b"[]").unwrap();
    let refused =
        baseline_cli(&fixture, &["--reopens", "reopens.json", "--report", "REOPENS.json"]);
    assert_eq!(refused.status.code(), Some(2));
    assert_eq!(refused.stdout, [] as [u8; 0]);
    assert_eq!(std::fs::read(root.join("reopens.json")).unwrap(), b"[]");
    std::fs::write(root.join("reopens.json"), b"[{\"unknown\":true}]").unwrap();
    let invalid =
        baseline_cli(&fixture, &["--reopens", "reopens.json", "--report", "invalid-reopen.json"]);
    assert_eq!(invalid.status.code(), Some(2));
    assert_eq!(invalid.stdout, [] as [u8; 0]);
    assert!(!root.join("invalid-reopen.json").exists());
    assert_authored_cli_inputs(&fixture, &inputs);
}

/// The public command reaches the core's canonical date check rather than accepting signed/expanded years.
#[test]
fn baseline_cli_rejects_noncanonical_years_and_accepts_real_leap_day() {
    let fixture = fixture();
    let inputs = install_baseline_cli(&fixture);
    let root = fixture.directory.path();
    for date in
        ["--as-of=-0001-01-01", "--as-of=+10000-12-31", "--as-of=2026-02-29", "--as-of=2026-2-06"]
    {
        let result = run(
            root,
            &["poam", "baseline", "--manifest", "poam.json", "--baseline", "prior.json", date],
        );
        assert_eq!(result.status.code(), Some(2), "{date}");
        assert!(result.stdout.is_empty(), "{date}");
    }
    let valid = run(
        root,
        &[
            "poam",
            "baseline",
            "--manifest",
            "poam.json",
            "--baseline",
            "prior.json",
            "--as-of=2028-02-29",
        ],
    );
    assert_eq!(valid.status.code(), Some(0), "{}", String::from_utf8_lossy(&valid.stderr));
    let report: Value = serde_json::from_slice(&valid.stdout).unwrap();
    assert_eq!(report["as_of"], "2028-02-29");
    assert_authored_cli_inputs(&fixture, &inputs);
}

/// Empty prior inspection reports complete additions and a refusal reason while artifact guards remain exact.
#[test]
fn baseline_cli_empty_previous_explains_complete_additions_without_build_admission() {
    let fixture = fixture();
    install_baseline_cli(&fixture);
    let root = fixture.directory.path();
    let mut prior: Value =
        serde_json::from_slice(&std::fs::read(root.join("prior.json")).unwrap()).unwrap();
    prior["items"] = json!([]);
    prior["roles"] = json!([]);
    prior["parties"] = json!([]);
    let previous = write_json(&root.join("prior.json"), &prior);
    let comparison = baseline_cli(&fixture, &[]);
    assert_eq!(
        comparison.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&comparison.stderr)
    );
    let report: Value = serde_json::from_slice(&comparison.stdout).unwrap();
    assert_eq!(report["items"]["previous"], 0);
    assert_eq!(report["items"]["added"], 1);
    assert_eq!(report["milestones"]["previous"], 0);
    assert_eq!(report["milestones"]["added"], 1);
    assert_eq!(report["rows"].as_array().unwrap().len(), 2);
    assert_eq!(report["revision_rules_compatible"], false);
    assert_eq!(report["artifact_validated"], false);
    assert!(report["refusal_observations"].as_array().unwrap().contains(&json!("empty-previous")));
    let check = run(
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
            "2026-02-06",
        ],
    );
    assert_eq!(check.status.code(), Some(2));
    assert_eq!(check.stdout, [] as [u8; 0]);
    let build = run(
        root,
        &[
            "poam",
            "build",
            "--manifest",
            "poam.json",
            "--baseline",
            "prior.json",
            "--as-of",
            "2026-02-06",
            "--output",
            "refused-empty-prior.json",
        ],
    );
    assert_eq!(build.status.code(), Some(2));
    assert!(!root.join("refused-empty-prior.json").exists());
    assert_eq!(std::fs::read(root.join("prior.json")).unwrap(), previous);
}
// S1/S2 synthetic native controls appended to the exact frozen foundation helpers.
// Authored here as source only. No Rust execution, representative-plan or consumer acceptance.

/// Explicit synthetic event; actor/rationale are declarations rather than verified authority.
fn portfolio_native_event(key: &str, at: &str, from: Option<&str>, to: &str, party: &str) -> Value {
    json!({"key":key,"actor":{"role_id":"owner","party_key":party},"at":at,
        "from":from,"to":to,"rationale":"SENSITIVE authored\nline\twith tab","closure":null})
}

/// Persist a real current authoring/native pair prepared from the fixture's actual five sources.
fn portfolio_native_pair(
    fixture: &Fixture,
    value: &Value,
    authoring: &str,
    native: &str,
) -> forge::poam::portfolio::Input {
    let root = fixture.directory.path();
    write_json(&root.join(authoring), value);
    let prepared = prepare_authored(fixture, value, None).unwrap();
    prepared.verify_inputs().unwrap();
    std::fs::write(root.join(native), prepared.artifact()).unwrap();
    forge::poam::portfolio::Input {
        manifest: root.join(authoring),
        native_artifact: Some(std::path::PathBuf::from(native)),
    }
}

/// Select the exact actual native risk tuple, without interpreting its status as eligibility.
fn portfolio_native_risk(fixture: &Fixture, value: &Value) -> Value {
    let declaration = forge::poam::workflow::parse(&serde_json::to_vec(value).unwrap()).unwrap();
    let captured =
        source::load(&fixture.directory.path().join("poam.json"), &declaration.source).unwrap();
    let selected = captured
        .inventory()
        .objects
        .iter()
        .find(|object| object.kind == manifest::SourceKind::Risk)
        .unwrap();
    json!({"kind":selected.kind,"key":selected.key,"uuid":selected.uuid,
        "result_uuid":selected.result_uuid,"expected_sha256":selected.sha256})
}

/// Snapshot only caller-named files; no directory scanning or implicit source discovery.
fn portfolio_native_snapshot(root: &Path, names: &[&str]) -> Vec<(std::path::PathBuf, Vec<u8>)> {
    names
        .iter()
        .map(|name| {
            let path = root.join(name);
            let bytes = std::fs::read(&path).unwrap();
            (path, bytes)
        })
        .collect()
}

/// Require exact literal source/author/native preservation after read-only projection.
fn portfolio_native_unchanged(snapshot: &[(std::path::PathBuf, Vec<u8>)]) {
    for (path, bytes) in snapshot {
        assert_eq!(std::fs::read(path).unwrap(), *bytes, "{}", path.display());
    }
}

/// Two actual supplied plans charge shared sources twice and retain null/cancelled/future rows.
#[test]
#[allow(
    clippy::too_many_lines,
    reason = "one complete paired-native portfolio checks all repeated denominators and inert traces"
)]
fn portfolio_native_complete_counts_temporal_trace_and_escaped_html() {
    use forge::poam::{html, portfolio};
    let fixture = fixture();
    let root = fixture.directory.path();
    let base = authored_workflow(&fixture);
    let declaration = forge::poam::workflow::parse(&serde_json::to_vec(&base).unwrap()).unwrap();
    let captured = source::load(&root.join("poam.json"), &declaration.source).unwrap();
    let inventory_count = captured.inventory().objects.len();
    let source_names = [
        "assessment-results.json",
        "assessment-plan.json",
        "ssp.json",
        "profile.json",
        "catalog.json",
    ];
    let one_plan_source_bytes: usize =
        source_names.iter().map(|name| std::fs::read(root.join(name)).unwrap().len()).sum();
    drop(captured);
    let hostile_key = "<img src=\"https://example.invalid/x\" onerror='run()'>&é";
    let hostile_party = "remediator<&\"'é>";
    let mut first = base.clone();
    first["document"]["key"] = json!("portfolio-a");
    first["parties"][0]["key"] = json!(hostile_party);
    first["items"][0]["key"] = json!(hostile_key);
    first["items"][0]["owners"][0]["party_key"] = json!(hostile_party);
    first["items"][0]["state"] = json!("cancelled");
    first["items"][0]["history"] = json!([
        portfolio_native_event("plan", "2026-01-03T00:00:00Z", None, "planned", hostile_party),
        portfolio_native_event(
            "blocked",
            "2026-01-06T01:00:00+01:00",
            Some("planned"),
            "blocked",
            hostile_party
        ),
        portfolio_native_event(
            "cancel",
            "2026-01-09T00:00:00Z",
            Some("blocked"),
            "cancelled",
            hostile_party
        ),
    ]);
    first["items"][0]["milestones"][0]["key"] = json!(hostile_key);
    first["items"][0]["milestones"][0]["owners"][0]["party_key"] = json!(hostile_party);
    first["items"][0]["milestones"][0]["history"] = json!([portfolio_native_event(
        "future-plan",
        "2026-01-08T00:00:00Z",
        None,
        "planned",
        hostile_party
    ),]);
    let mut second = base.clone();
    second["document"]["key"] = json!("portfolio-b");
    second["items"][0]["source_refs"] = json!([portfolio_native_risk(&fixture, &base)]);
    second["items"][0]["state"] = json!("cancelled");
    second["items"][0]["history"] = json!([
        portfolio_native_event("plan", "2026-01-03T00:00:00Z", None, "planned", "remediator"),
        portfolio_native_event(
            "cancel",
            "2026-01-06T00:00:00Z",
            Some("planned"),
            "cancelled",
            "remediator"
        ),
    ]);
    second["items"][0]["milestones"][0]["state"] = json!("cancelled");
    second["items"][0]["milestones"][0]["history"] = second["items"][0]["history"].clone();
    let first_input = portfolio_native_pair(&fixture, &first, "portfolio-a.json", "native-a.json");
    let second_input =
        portfolio_native_pair(&fixture, &second, "portfolio-b.json", "native-b.json");
    let original = portfolio_native_snapshot(
        root,
        &[
            "assessment-results.json",
            "assessment-plan.json",
            "ssp.json",
            "profile.json",
            "catalog.json",
            "portfolio-a.json",
            "native-a.json",
            "portfolio-b.json",
            "native-b.json",
        ],
    );
    let inputs = [second_input, first_input];
    let prepared = portfolio::prepare_native_portfolio(&inputs, "2026-01-07", 7).unwrap();
    prepared.verify_inputs().unwrap();
    assert_eq!(prepared.validation_scope(), "supplied-native-artifact-portfolio");
    assert_eq!(prepared.supplied_native_artifacts(), 2);
    assert!(prepared.review_required());
    let report_bytes = prepared.json().to_vec();
    let report: Value = serde_json::from_slice(&report_bytes).unwrap();
    assert_eq!(report["schema_version"], json!("forge.poam-portfolio/1"));
    assert_eq!(report["as_of"], json!("2026-01-07"));
    assert_eq!(report["due_soon_days"], json!(7));
    assert_eq!(
        report["counts"],
        json!({
            "plans":2,"supplied_native_artifacts":2,"authoring_only_plans":0,
            "source_files":10,"source_bytes":2 * one_plan_source_bytes,"source_objects":2 * inventory_count,
            "items":2,"milestones":2,"records":4,"history_events":8,"source_selections":2,
            "owner_declarations":4,"dependency_edges":0,"overdue":0,"due_soon":0,"blocked":1,
            "before_first_assertion":1,"cancelled":2,
        })
    );
    let plans = report["plans"].as_array().unwrap();
    assert_eq!(plans.len(), 2);
    assert_eq!(plans[0]["key"], json!("portfolio-a"));
    assert_eq!(plans[1]["key"], json!("portfolio-b"));
    let records = plans[0]["records"].as_array().unwrap();
    let item = records.iter().find(|row| row["milestone_key"].is_null()).unwrap();
    let milestone = records.iter().find(|row| !row["milestone_key"].is_null()).unwrap();
    assert_eq!(item["state_at_as_of"], json!("blocked"));
    assert_eq!(item["declared_state"], json!("cancelled"));
    assert_eq!(milestone["state_at_as_of"], Value::Null);
    assert_eq!(milestone["declared_state"], json!("planned"));
    let timeline = plans[0]["timeline"].as_array().unwrap();
    assert_eq!(
        timeline.iter().map(|row| row["at"].as_str().unwrap()).collect::<Vec<_>>(),
        vec![
            "2026-01-03T00:00:00Z",
            "2026-01-06T01:00:00+01:00",
            "2026-01-08T00:00:00Z",
            "2026-01-09T00:00:00Z",
        ]
    );
    assert_eq!(timeline.iter().filter(|row| row["after_as_of"] == json!(true)).count(), 2);
    let page = String::from_utf8(html::render(prepared.report()).unwrap()).unwrap();
    let escaped_key =
        "&lt;img src=&quot;https://example.invalid/x&quot; onerror=&#39;run()&#39;&gt;&amp;é";
    assert!(page.contains(escaped_key));
    for caption in [
        "Complete item and milestone schedule",
        "Complete author assertion timeline, ordered by actual instant",
        "Complete selected source-to-remediation trace",
    ] {
        let marker = format!("{caption}</caption>");
        let table = page.split(marker.as_str()).nth(1).unwrap().split("</table>").next().unwrap();
        assert!(table.contains(escaped_key), "missing escaped work identity in {caption}");
        assert!(!table.contains(hostile_key));
    }
    assert!(page.contains("remediator&lt;&amp;&quot;&#39;é&gt;"));
    assert!(page.contains("before first assertion"));
    assert!(page.contains("Explicit as-of: <code>2026-01-07</code>"));
    for active in
        ["<img", "<a ", "<script", "<iframe", "<svg", "<form", "href=\"", "src=\"", "onerror='"]
    {
        assert!(!page.contains(active), "active markup: {active}");
    }
    assert!(!page.contains("SENSITIVE"));
    assert!(!String::from_utf8(report_bytes.clone()).unwrap().contains("SENSITIVE"));
    for (index, (author, native)) in
        [(first, "native-a.json"), (second, "native-b.json")].iter().enumerate()
    {
        let native_bytes = std::fs::read(root.join(native)).unwrap();
        let actual_native: Value = serde_json::from_slice(&native_bytes).unwrap();
        assert_eq!(
            plans[index]["supplied_native_sha256"],
            json!(common::sha256_hex(&native_bytes))
        );
        assert_eq!(
            plans[index]["generated_native_sha256"],
            json!(common::sha256_hex(&native_bytes))
        );
        assert_eq!(plans[index]["supplied_native_matched"], json!(true));
        let rows = plans[index]["sources"].as_array().unwrap();
        assert_eq!(rows.len(), 5);
        for receipt in actual_native["plan-of-action-and-milestones"]["back-matter"]["resources"]
            .as_array()
            .unwrap()
        {
            let row = rows.iter().find(|row| row["kind"] == receipt["title"]).unwrap();
            assert_eq!(row["native_href"], receipt["rlinks"][0]["href"]);
            assert_eq!(row["sha256"], receipt["rlinks"][0]["hashes"][0]["value"]);
            assert!(page.contains(row["native_href"].as_str().unwrap()));
        }
        let trace = &plans[index]["trace"][0];
        let selection = &author["items"][0]["source_refs"][0];
        assert_eq!(trace["kind"], selection["kind"]);
        assert_eq!(trace["source_key"], selection["key"]);
        assert_eq!(trace["source_uuid"], selection["uuid"]);
        assert_eq!(trace["result_uuid"], selection["result_uuid"]);
        assert_eq!(trace["canonical_object_sha256"], selection["expected_sha256"]);
        assert_eq!(
            trace["assessment_results_file_sha256"],
            author["source"]["assessment_results"]["expected_sha256"]
        );
        assert_eq!(
            trace["native_href"],
            actual_native["plan-of-action-and-milestones"]["poam-items"][0]["links"][0]["href"]
        );
        assert!(page.contains(trace["native_href"].as_str().unwrap()));
    }
    assert_eq!(prepared.json(), report_bytes.as_slice());
    for (as_of, overdue, due_soon) in [("2026-02-05", 0, 1), ("2026-02-06", 1, 0)] {
        let dated = portfolio::prepare_native_portfolio(&inputs, as_of, 0).unwrap();
        let dated_report: Value = serde_json::from_slice(dated.json()).unwrap();
        assert_eq!(dated_report["as_of"], json!(as_of));
        assert_eq!(dated_report["counts"]["overdue"], json!(overdue));
        assert_eq!(dated_report["counts"]["due_soon"], json!(due_soon));
        assert_eq!(dated_report["counts"]["cancelled"], json!(3));
        assert_eq!(dated_report["counts"]["blocked"], json!(0));
        assert_eq!(dated_report["counts"]["before_first_assertion"], json!(0));
        for denominator in [
            "plans",
            "source_files",
            "source_bytes",
            "source_objects",
            "items",
            "milestones",
            "records",
            "history_events",
            "source_selections",
            "owner_declarations",
        ] {
            assert_eq!(dated_report["counts"][denominator], report["counts"][denominator]);
        }
        dated.verify_inputs().unwrap();
    }
    prepared.verify_inputs().unwrap();
    portfolio_native_unchanged(&original);
}

/// Unknown fields, prose, receipt hashes and source hrefs are refused against a valid whole native pair.
#[test]
fn portfolio_native_refuses_full_value_extensions_and_known_field_changes() {
    use forge::poam::portfolio;
    let fixture = fixture();
    let root = fixture.directory.path();
    let authored = authored_workflow(&fixture);
    let input = portfolio_native_pair(&fixture, &authored, "paired.json", "native.json");
    portfolio::prepare_native_portfolio(std::slice::from_ref(&input), "2026-01-07", 7)
        .unwrap()
        .verify_inputs()
        .unwrap();
    let native_path = root.join("native.json");
    let original = std::fs::read(&native_path).unwrap();
    let expected: Value = serde_json::from_slice(&original).unwrap();
    let mut compact = serde_json::to_vec(&expected).unwrap();
    compact.push(b'\n');
    assert_ne!(compact, original);
    std::fs::write(&native_path, &compact).unwrap();
    let reformatted =
        portfolio::prepare_native_portfolio(std::slice::from_ref(&input), "2026-01-07", 7).unwrap();
    let reformatted_report: Value = serde_json::from_slice(reformatted.json()).unwrap();
    assert_eq!(
        reformatted_report["plans"][0]["supplied_native_sha256"],
        json!(common::sha256_hex(&compact))
    );
    assert_eq!(
        reformatted_report["plans"][0]["generated_native_sha256"],
        json!(common::sha256_hex(&original))
    );
    reformatted.verify_inputs().unwrap();
    drop(reformatted);
    std::fs::write(&native_path, &original).unwrap();
    let unchanged = portfolio_native_snapshot(
        root,
        &[
            "assessment-results.json",
            "assessment-plan.json",
            "ssp.json",
            "profile.json",
            "catalog.json",
            "paired.json",
        ],
    );
    for mutation in 0..4 {
        let mut changed = expected.clone();
        let native = &mut changed["plan-of-action-and-milestones"];
        match mutation {
            0 => native["unrecognized-extension"] = json!({"private":"unexpected"}),
            1 => native["poam-items"][0]["description"] = json!("Changed authored native prose"),
            2 => {
                let property = native["metadata"]["props"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|property| property["name"] == json!("source-file-sha256"))
                    .unwrap();
                property["value"] = json!("0".repeat(64));
            }
            3 => {
                native["poam-items"][0]["links"][0]["href"] =
                    json!("different.json#11111111-1111-4111-8111-111111111111");
            }
            _ => unreachable!(),
        }
        assert_ne!(changed, expected);
        write_json(&native_path, &changed);
        let reason =
            portfolio::prepare_native_portfolio(std::slice::from_ref(&input), "2026-01-07", 7)
                .unwrap_err()
                .to_string();
        assert!(
            reason.contains("differs from its full supported authoring projection"),
            "{reason}"
        );
        portfolio_native_unchanged(&unchanged);
        std::fs::write(&native_path, &original).unwrap();
        portfolio::prepare_native_portfolio(std::slice::from_ref(&input), "2026-01-07", 7)
            .unwrap()
            .verify_inputs()
            .unwrap();
    }
    assert_eq!(std::fs::read(native_path).unwrap(), original);
}

/// Rechecks bind literal JSON bytes and live identity of every input, including same-byte replacement.
#[test]
fn portfolio_native_rechecks_all_originals_and_same_byte_source_replacement() {
    use forge::poam::portfolio;
    let fixture = fixture();
    let root = fixture.directory.path();
    let authored = authored_workflow(&fixture);
    let input = portfolio_native_pair(&fixture, &authored, "paired.json", "native.json");
    let original = portfolio_native_snapshot(
        root,
        &[
            "assessment-results.json",
            "assessment-plan.json",
            "ssp.json",
            "profile.json",
            "catalog.json",
            "paired.json",
            "native.json",
        ],
    );
    let prepared = portfolio::prepare_native_portfolio(&[input], "2026-01-07", 7).unwrap();
    prepared.verify_inputs().unwrap();
    for (path, bytes) in &original {
        let mut changed = bytes.clone();
        let suffix = changed.last_mut().unwrap();
        assert!(
            suffix.is_ascii_whitespace(),
            "fixture JSON must end in whitespace: {}",
            path.display()
        );
        *suffix = if *suffix == b' ' { b'\n' } else { b' ' };
        assert_ne!(changed, *bytes);
        assert_eq!(
            serde_json::from_slice::<Value>(&changed).unwrap(),
            serde_json::from_slice::<Value>(bytes).unwrap()
        );
        std::fs::write(path, &changed).unwrap();
        let reason = prepared.verify_inputs().unwrap_err().to_string();
        assert!(reason.contains("original portfolio input identity or bytes changed"), "{reason}");
        std::fs::write(path, bytes).unwrap();
        prepared.verify_inputs().unwrap();
    }
    let catalog = root.join("catalog.json");
    let replacement = root.join("catalog-replacement.json");
    let same_bytes = std::fs::read(&catalog).unwrap();
    // Allocate the replacement while the captured original exists, preventing accidental ID reuse here.
    std::fs::write(&replacement, &same_bytes).unwrap();
    std::fs::remove_file(&catalog).unwrap();
    std::fs::rename(&replacement, &catalog).unwrap();
    assert_eq!(std::fs::read(&catalog).unwrap(), same_bytes);
    let reason = prepared.verify_inputs().unwrap_err().to_string();
    assert!(reason.contains("original portfolio input identity or bytes changed"), "{reason}");
    portfolio_native_unchanged(&original);
}

/// Every admitted plan requires its supplied native, and real input/source aliases cannot supply it.
#[test]
fn portfolio_native_requires_every_plan_and_rejects_stable_or_file_aliases() {
    use forge::poam::portfolio;
    let fixture = fixture();
    let root = fixture.directory.path();
    let first = authored_workflow(&fixture);
    let mut second = first.clone();
    second["document"]["key"] = json!("other-explicit-plan");
    let first_input = portfolio_native_pair(&fixture, &first, "first.json", "first-native.json");
    let second_input =
        portfolio_native_pair(&fixture, &second, "second.json", "second-native.json");
    portfolio::prepare_native_portfolio(
        &[first_input.clone(), second_input.clone()],
        "2026-01-07",
        7,
    )
    .unwrap()
    .verify_inputs()
    .unwrap();
    let mut missing_native = second_input;
    missing_native.native_artifact = None;
    let reason = portfolio::prepare_native_portfolio(
        &[first_input.clone(), missing_native],
        "2026-01-07",
        7,
    )
    .unwrap_err()
    .to_string();
    assert!(
        reason.contains("requires an explicit native file for every authoring companion"),
        "{reason}"
    );
    let duplicate_plan =
        portfolio_native_pair(&fixture, &first, "duplicate-plan.json", "duplicate-native.json");
    let reason = portfolio::prepare_native_portfolio(
        &[first_input.clone(), duplicate_plan],
        "2026-01-07",
        7,
    )
    .unwrap_err()
    .to_string();
    assert!(reason.contains("duplicate stable plan identity"), "{reason}");
    let reason = portfolio::prepare_native_portfolio(
        &[first_input.clone(), first_input.clone()],
        "2026-01-07",
        7,
    )
    .unwrap_err()
    .to_string();
    assert!(reason.contains("duplicate or aliased explicit authoring/native input"), "{reason}");
    for (native, expected_reason) in [
        ("first.json", "duplicate or aliased explicit authoring/native input"),
        ("catalog.json", "authoring or native input aliases a source input"),
    ] {
        let mut alias = first_input.clone();
        alias.native_artifact = Some(std::path::PathBuf::from(native));
        let reason =
            portfolio::prepare_native_portfolio(&[alias], "2026-01-07", 7).unwrap_err().to_string();
        assert!(reason.contains(expected_reason), "{reason}");
    }
    portfolio::prepare_native_portfolio(&[first_input], "2026-01-07", 7)
        .unwrap()
        .verify_inputs()
        .unwrap();
    assert!(!root.join("portfolio-report.json").exists());
    assert!(!root.join("portfolio-report.html").exists());
}

/// Past and future closure assertions hit the public pending guard, after valid positive source/native pairs.
#[test]
fn portfolio_native_terminal_refusal_is_not_a_missing_milestone_or_risk_shape_error() {
    use forge::poam::portfolio;
    let fixture = fixture();
    let root = fixture.directory.path();
    let mut positive = authored_workflow(&fixture);
    positive["roles"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"reviewer","title":"Declared reviewer"}));
    positive["parties"]
        .as_array_mut()
        .unwrap()
        .push(json!({"key":"reviewer-party","type":"person","name":"SENSITIVE REVIEWER NAME"}));
    let actual_risk = portfolio_native_risk(&fixture, &positive);
    let source_originals = portfolio_native_snapshot(
        root,
        &[
            "assessment-results.json",
            "assessment-plan.json",
            "ssp.json",
            "profile.json",
            "catalog.json",
        ],
    );
    for state in ["completed-asserted", "accepted-risk-asserted"] {
        for at in ["2026-01-06T00:00:00Z", "2026-02-15T00:00:00Z"] {
            let mut baseline = positive.clone();
            if state == "accepted-risk-asserted" {
                baseline["items"][0]["source_refs"]
                    .as_array_mut()
                    .unwrap()
                    .push(actual_risk.clone());
            }
            let input = portfolio_native_pair(
                &fixture,
                &baseline,
                "closure-authoring.json",
                "closure-native.json",
            );
            portfolio::prepare_native_portfolio(std::slice::from_ref(&input), "2026-01-07", 7)
                .unwrap()
                .verify_inputs()
                .unwrap();
            let original_native = std::fs::read(root.join("closure-native.json")).unwrap();
            let original_author = std::fs::read(&input.manifest).unwrap();
            let mut refused = baseline;
            let record = if state == "completed-asserted" {
                &mut refused["items"][0]["milestones"][0]
            } else {
                &mut refused["items"][0]
            };
            record["state"] = json!(state);
            record["history"].as_array_mut().unwrap().push(portfolio_native_event(
                "start",
                "2026-01-04T00:00:00Z",
                Some("planned"),
                "in-progress",
                "remediator",
            ));
            let mut terminal =
                portfolio_native_event("terminal", at, Some("in-progress"), state, "remediator");
            terminal["closure"] = json!({"reviewer":{"role_id":"reviewer","party_key":"reviewer-party"},
                "reviewed_at":at,"rationale":"Declared review only",
                "evidence":[{"key":"proof","href":"proof.json","expected_sha256":"c".repeat(64)}]});
            record["history"].as_array_mut().unwrap().push(terminal);
            let raw = write_json(&input.manifest, &refused);
            // parse validates the full current shape before this refusal; its reason is a canary.
            let shape_reason = forge::poam::workflow::parse(&raw).unwrap_err().to_string();
            assert!(
                shape_reason.contains("pending recorded closure disposition"),
                "{shape_reason}"
            );
            let reason =
                portfolio::prepare_native_portfolio(std::slice::from_ref(&input), "2026-01-07", 7)
                    .unwrap_err()
                    .to_string();
            assert!(reason.contains("pending recorded closure disposition"), "{reason}");
            assert_eq!(std::fs::read(root.join("closure-native.json")).unwrap(), original_native);
            portfolio_native_unchanged(&source_originals);
            std::fs::write(&input.manifest, &original_author).unwrap();
            portfolio::prepare_native_portfolio(std::slice::from_ref(&input), "2026-01-07", 7)
                .unwrap()
                .verify_inputs()
                .unwrap();
        }
    }
}

/// Invoke only explicitly paired native inputs and an explicit date/output directory.
fn portfolio_cli_run(root: &Path, pairs: &[(&Path, &str)], date: &str, extra: &[&str]) -> Output {
    let mut args = vec![
        "poam".to_string(),
        "portfolio".to_string(),
        "--as-of".to_string(),
        date.to_string(),
        "--output-root".to_string(),
        root.to_str().unwrap().to_string(),
    ];
    for (manifest, native) in pairs {
        args.extend([
            "--manifest".to_string(),
            manifest.to_str().unwrap().to_string(),
            "--native".to_string(),
            (*native).to_string(),
        ]);
    }
    args.extend(extra.iter().map(|value| (*value).to_string()));
    let borrowed: Vec<&str> = args.iter().map(String::as_str).collect();
    run(root, &borrowed)
}

/// Real supplied-native stdout is deterministic, complete, minimized and selects valid action exits.
#[test]
fn portfolio_cli_stdout_matches_complete_native_projection_and_schedule_exits() {
    let fixture = fixture();
    let value = authored_workflow(&fixture);
    let input = portfolio_native_pair(&fixture, &value, "authoring.json", "native.json");
    let root = fixture.directory.path();
    let snapshot = portfolio_native_snapshot(
        root,
        &[
            "authoring.json",
            "native.json",
            "assessment-results.json",
            "assessment-plan.json",
            "ssp.json",
            "profile.json",
            "catalog.json",
        ],
    );
    let pairs = [(input.manifest.as_path(), "native.json")];
    let first = portfolio_cli_run(root, &pairs, "2026-01-04", &[]);
    let second = portfolio_cli_run(root, &pairs, "2026-01-04", &[]);
    assert_eq!(first.status.code(), Some(0), "{}", String::from_utf8_lossy(&first.stderr));
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(first.stderr, [] as [u8; 0]);
    assert_eq!(first.stdout, second.stdout);
    let prepared = forge::poam::portfolio::prepare_native_portfolio(
        std::slice::from_ref(&input),
        "2026-01-04",
        0,
    )
    .unwrap();
    assert_eq!(first.stdout, prepared.json());
    let report: Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(report["schema_version"], "forge.poam-portfolio/1");
    assert_eq!(report["validation_scope"], "supplied-native-artifact-portfolio");
    for (name, count) in [
        ("plans", 1),
        ("supplied_native_artifacts", 1),
        ("authoring_only_plans", 0),
        ("source_files", 5),
        ("source_objects", 3),
        ("items", 1),
        ("milestones", 1),
        ("records", 2),
        ("history_events", 2),
    ] {
        assert_eq!(report["counts"][name], count, "{name}");
    }
    assert!(!String::from_utf8(first.stdout).unwrap().contains("SENSITIVE"));
    let overdue = portfolio_cli_run(root, &pairs, "2026-02-06", &[]);
    assert_eq!(overdue.status.code(), Some(1));
    assert_eq!(overdue.stderr, [] as [u8; 0]);
    let later: Value = serde_json::from_slice(&overdue.stdout).unwrap();
    assert_eq!(later["counts"]["overdue"], 1);
    portfolio_native_unchanged(&snapshot);
}

/// Distinct directories retain explicit positional companions, stable sorting and full per-plan totals.
#[test]
fn portfolio_cli_accepts_two_explicit_roots_without_rebasing_native_hrefs() {
    let first = fixture();
    let second = fixture();
    let mut a = authored_workflow(&first);
    let mut b = authored_workflow(&second);
    a["document"]["key"] = json!("z-last");
    b["document"]["key"] = json!("a-first");
    let pa = portfolio_native_pair(&first, &a, "authoring.json", "native-a.json");
    let pb = portfolio_native_pair(&second, &b, "authoring.json", "native-b.json");
    let output_root = tempfile::tempdir().unwrap();
    let output = portfolio_cli_run(
        output_root.path(),
        &[(pa.manifest.as_path(), "native-a.json"), (pb.manifest.as_path(), "native-b.json")],
        "2026-01-04",
        &[],
    );
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    let prepared = forge::poam::portfolio::prepare_native_portfolio(
        &[pa.clone(), pb.clone()],
        "2026-01-04",
        0,
    )
    .unwrap();
    assert_eq!(output.stdout, prepared.json());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["counts"]["plans"], 2);
    assert_eq!(report["counts"]["supplied_native_artifacts"], 2);
    assert_eq!(report["counts"]["source_files"], 10);
    assert_eq!(report["counts"]["source_objects"], 6);
    assert_eq!(report["plans"][0]["key"], "a-first");
    assert_eq!(report["plans"][1]["key"], "z-last");
    assert_eq!(report["plans"][0]["sources"][0]["native_href"], "assessment-results.json");
    // Both deliberately mispaired files actually exist and are independently valid native values.
    let native_a = std::fs::read(first.directory.path().join("native-a.json")).unwrap();
    let native_b = std::fs::read(second.directory.path().join("native-b.json")).unwrap();
    std::fs::write(first.directory.path().join("native-b.json"), native_b).unwrap();
    std::fs::write(second.directory.path().join("native-a.json"), native_a).unwrap();
    let mispaired = portfolio_cli_run(
        output_root.path(),
        &[(pa.manifest.as_path(), "native-b.json"), (pb.manifest.as_path(), "native-a.json")],
        "2026-01-04",
        &["--report", "mispaired.json", "--html", "mispaired.html"],
    );
    assert_eq!(mispaired.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&mispaired.stderr).contains("full supported authoring projection")
    );
    assert!(!output_root.path().join("mispaired.json").exists());
    assert!(!output_root.path().join("mispaired.html").exists());
    assert_eq!(std::fs::read_dir(output_root.path()).unwrap().count(), 0);
}

/// Actual new-file publication emits the exact complete JSON/HTML and preflights every known conflict.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn portfolio_cli_publishes_complete_json_html_and_preserves_existing_outputs() {
    let fixture = fixture();
    let mut value = authored_workflow(&fixture);
    value["document"]["key"] = json!("synthetic-demo-plan");
    let input = portfolio_native_pair(&fixture, &value, "authoring.json", "native.json");
    let root = fixture.directory.path();
    let snapshot = portfolio_native_snapshot(
        root,
        &[
            "authoring.json",
            "native.json",
            "assessment-results.json",
            "assessment-plan.json",
            "ssp.json",
            "profile.json",
            "catalog.json",
        ],
    );
    let output_root = tempfile::tempdir().unwrap();
    let pairs = [(input.manifest.as_path(), "native.json")];
    let output = portfolio_cli_run(
        output_root.path(),
        &pairs,
        "2026-02-06",
        &["--report", "portfolio.json", "--html", "portfolio.html"],
    );
    assert_eq!(output.status.code(), Some(1), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.stdout, [] as [u8; 0]);
    assert_eq!(output.stderr, [] as [u8; 0]);
    let prepared = forge::poam::portfolio::prepare_native_portfolio(
        std::slice::from_ref(&input),
        "2026-02-06",
        0,
    )
    .unwrap();
    let json_bytes = std::fs::read(output_root.path().join("portfolio.json")).unwrap();
    let html_bytes = std::fs::read(output_root.path().join("portfolio.html")).unwrap();
    assert_eq!(json_bytes, prepared.json());
    assert_eq!(html_bytes, forge::poam::html::render(prepared.report()).unwrap());
    let text = String::from_utf8(html_bytes.clone()).unwrap();
    assert!(text.contains("Complete author assertion timeline"));
    assert!(text.contains("Complete selected source-to-remediation trace"));
    assert!(!text.contains("SENSITIVE"));
    for extras in [
        vec!["--report", "portfolio.json", "--html", "other.html"],
        vec!["--report", "other.json", "--html", "portfolio.html"],
    ] {
        let refused = portfolio_cli_run(output_root.path(), &pairs, "2026-02-06", &extras);
        assert_eq!(refused.status.code(), Some(2));
        assert_eq!(refused.stdout, [] as [u8; 0]);
        assert!(!output_root.path().join("other.json").exists());
        assert!(!output_root.path().join("other.html").exists());
    }
    assert_eq!(std::fs::read(output_root.path().join("portfolio.json")).unwrap(), json_bytes);
    assert_eq!(std::fs::read(output_root.path().join("portfolio.html")).unwrap(), html_bytes);
    portfolio_native_unchanged(&snapshot);
}

/// Every actual declaration/native/source full path is protected even with a case-variant output name.
#[test]
fn portfolio_cli_preflights_full_input_paths_and_portable_destinations() {
    let fixture = fixture();
    let input = portfolio_native_pair(
        &fixture,
        &authored_workflow(&fixture),
        "authoring.json",
        "native.json",
    );
    let root = fixture.directory.path();
    let pairs = [(input.manifest.as_path(), "native.json")];
    let positive = portfolio_cli_run(root, &pairs, "2026-01-04", &[]);
    assert_eq!(positive.status.code(), Some(0), "{}", String::from_utf8_lossy(&positive.stderr));
    let snapshot = portfolio_native_snapshot(
        root,
        &[
            "authoring.json",
            "native.json",
            "assessment-results.json",
            "assessment-plan.json",
            "ssp.json",
            "profile.json",
            "catalog.json",
        ],
    );
    for name in [
        "AUTHORING.json",
        "NATIVE.json",
        "ASSESSMENT-RESULTS.json",
        "ASSESSMENT-PLAN.json",
        "SSP.json",
        "PROFILE.json",
        "CATALOG.json",
        "../outside.json",
        "nested/report.json",
        "NUL.json",
        "bad:name.json",
        "unicode-é.json",
        "wrong.txt",
    ] {
        let output = portfolio_cli_run(
            root,
            &pairs,
            "2026-01-04",
            &["--report", name, "--html", "should-not-exist.html"],
        );
        assert_eq!(output.status.code(), Some(2), "{name}");
        assert_eq!(output.stdout, [] as [u8; 0]);
        assert!(!root.join("should-not-exist.html").exists(), "{name}");
    }
    for name in ["../outside.html", "NUL.html", "wrong.json", "nested/report.html"] {
        let output = portfolio_cli_run(
            root,
            &pairs,
            "2026-01-04",
            &["--report", "should-not-exist.json", "--html", name],
        );
        assert_eq!(output.status.code(), Some(2), "{name}");
        assert_eq!(output.stdout, [] as [u8; 0]);
        assert!(!root.join("should-not-exist.json").exists(), "{name}");
    }
    portfolio_native_unchanged(&snapshot);
}

/// A positive real pair precedes precise tampering, source-drift and duplicate-plan refusals.
#[test]
fn portfolio_cli_native_tampering_stale_source_and_duplicate_plans_fail_without_outputs() {
    let fixture = fixture();
    let value = authored_workflow(&fixture);
    let input = portfolio_native_pair(&fixture, &value, "authoring.json", "native.json");
    let root = fixture.directory.path();
    let pairs = [(input.manifest.as_path(), "native.json")];
    assert_eq!(portfolio_cli_run(root, &pairs, "2026-01-04", &[]).status.code(), Some(0));
    let native_bytes = std::fs::read(root.join("native.json")).unwrap();
    let mut native: Value = serde_json::from_slice(&native_bytes).unwrap();
    native["plan-of-action-and-milestones"]["metadata"]["title"] = json!("changed");
    write_json(&root.join("native.json"), &native);
    let changed = portfolio_cli_run(
        root,
        &pairs,
        "2026-01-04",
        &["--report", "refused.json", "--html", "refused.html"],
    );
    assert_eq!(changed.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&changed.stderr).contains("native"));
    assert!(!root.join("refused.json").exists());
    assert!(!root.join("refused.html").exists());
    std::fs::write(root.join("native.json"), &native_bytes).unwrap();
    let duplicate = portfolio_cli_run(root, &[pairs[0], pairs[0]], "2026-01-04", &[]);
    assert_eq!(duplicate.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&duplicate.stderr)
            .contains("duplicate or aliased explicit authoring/native input")
    );
    let sources = std::fs::read(root.join("catalog.json")).unwrap();
    std::fs::write(root.join("catalog.json"), b"{}").unwrap();
    let stale = portfolio_cli_run(root, &pairs, "2026-01-04", &["--report", "refused.json"]);
    assert_eq!(stale.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&stale.stderr).contains("whole-file pin"));
    assert!(!root.join("refused.json").exists());
    std::fs::write(root.join("catalog.json"), sources).unwrap();
    assert_eq!(portfolio_cli_run(root, &pairs, "2026-01-04", &[]).status.code(), Some(0));
}

/// Clap and runtime require every pair/date/root; no directory discovery or authoring-preview override exists.
#[test]
fn portfolio_cli_requires_complete_explicit_pairs_date_and_output_root() {
    let fixture = fixture();
    let input = portfolio_native_pair(
        &fixture,
        &authored_workflow(&fixture),
        "authoring.json",
        "native.json",
    );
    let root = fixture.directory.path();
    for args in [
        vec!["poam", "portfolio"],
        vec![
            "poam",
            "portfolio",
            "--manifest",
            "authoring.json",
            "--native",
            "native.json",
            "--output-root",
            ".",
        ],
        vec![
            "poam",
            "portfolio",
            "--manifest",
            "authoring.json",
            "--native",
            "native.json",
            "--as-of",
            "2026-01-04",
        ],
        vec![
            "poam",
            "portfolio",
            "--manifest",
            "authoring.json",
            "--as-of",
            "2026-01-04",
            "--output-root",
            ".",
        ],
    ] {
        let output = run(root, &args);
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(output.stdout, [] as [u8; 0]);
    }
    let pairs = [(input.manifest.as_path(), "native.json")];
    assert_eq!(portfolio_cli_run(root, &pairs, "2026-01-04", &[]).status.code(), Some(0));
    let mismatch = portfolio_cli_run(root, &pairs, "2026-01-04", &["--manifest", "authoring.json"]);
    assert_eq!(mismatch.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&mismatch.stderr).contains("one native filename for each"));
    for extra in [
        vec!["--authoring-preview"],
        vec!["--allow-terminal"],
        vec!["--discover"],
        vec!["--format", "text"],
        vec!["--due-soon-days", "366"],
        vec!["--native", "../outside.json"],
    ] {
        let output = portfolio_cli_run(root, &pairs, "2026-01-04", &extra);
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(output.stdout, [] as [u8; 0]);
    }
}

/// Canonical date refusal reaches the admitted pair and all dates are explicit without hidden clock use.
#[test]
fn portfolio_cli_rejects_noncanonical_dates_without_partial_reports() {
    let fixture = fixture();
    let input = portfolio_native_pair(
        &fixture,
        &authored_workflow(&fixture),
        "authoring.json",
        "native.json",
    );
    let root = fixture.directory.path();
    let pairs = [(input.manifest.as_path(), "native.json")];
    assert_eq!(portfolio_cli_run(root, &pairs, "2026-01-04", &[]).status.code(), Some(0));
    for date in ["2026-02-29", "2026-2-06", "2026-02-06T00:00:00Z", "+10000-01-01"] {
        let output = portfolio_cli_run(
            root,
            &pairs,
            date,
            &["--report", "invalid-date.json", "--html", "invalid-date.html"],
        );
        assert_eq!(output.status.code(), Some(2), "{date}");
        assert_eq!(output.stdout, [] as [u8; 0]);
        assert!(!root.join("invalid-date.json").exists());
        assert!(!root.join("invalid-date.html").exists());
    }
    let leap = portfolio_cli_run(root, &pairs, "2028-02-29", &[]);
    assert_eq!(leap.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&leap.stdout).unwrap();
    assert_eq!(report["as_of"], "2028-02-29");
}

// Proposed synthetic S3 controls: append only after Root composes the actual module/CLI ports.
// Source only; no Rust run, representative plan, consumer approval or remote authority is claimed.

/// Encode the complete closed caller selection without inventing a remote target or intent.
fn outbound_control_selection(operations: Value) -> Vec<u8> {
    let value = Value::Object(serde_json::Map::from_iter([
        ("schema_version".into(), json!("forge.poam-outbound-selection/1")),
        ("operations".into(), operations),
    ]));
    serde_json::to_vec(&value).unwrap()
}

/// Persist actual authoring and native originals prepared from this fixture's five captured sources.
fn outbound_control_pair(fixture: &Fixture, value: &Value) -> std::path::PathBuf {
    let root = fixture.directory.path();
    let path = root.join("outbound-authoring-current.json");
    let raw = serde_json::to_vec(value).unwrap();
    std::fs::write(&path, &raw).unwrap();
    let prepared = forge::poam::workflow::prepare(&path, &raw, "2026-02-06", 0, None).unwrap();
    prepared.verify_inputs().unwrap();
    std::fs::write(root.join("outbound-native-current.json"), prepared.artifact()).unwrap();
    path
}

/// Retain caller-named literal originals; this helper never discovers additional inputs.
fn outbound_control_originals(fixture: &Fixture) -> Vec<(std::path::PathBuf, Vec<u8>)> {
    [
        "outbound-authoring-current.json",
        "outbound-native-current.json",
        "outbound-selection-current.json",
        "assessment-results.json",
        "assessment-plan.json",
        "ssp.json",
        "profile.json",
        "catalog.json",
    ]
    .iter()
    .map(|name| {
        let path = fixture.directory.path().join(name);
        let raw = std::fs::read(&path).unwrap();
        (path, raw)
    })
    .collect()
}

/// Invoke the adopted single-pair black-box contract with explicit date, root and nondefault basenames.
fn outbound_control_cli(
    fixture: &Fixture,
    native: &str,
    selection: &str,
    date: &str,
    extra: &[&str],
) -> Output {
    let root = fixture.directory.path();
    let path = root.join("outbound-authoring-current.json");
    let mut args = vec![
        "poam",
        "outbound",
        "--manifest",
        path.to_str().unwrap(),
        "--native",
        native,
        "--selection",
        selection,
        "--as-of",
        date,
        "--output-root",
        root.to_str().unwrap(),
    ];
    args.extend_from_slice(extra);
    run(root, &args)
}

/// Every explicit intent admits a real 64-owner pair, separates generations from stable keys and rejects 65 owners.
#[test]
#[allow(
    clippy::too_many_lines,
    reason = "one actual 64-owner pair checks bounded intent and generation semantics"
)]
fn outbound_contract_actual_pair_minimizes_all_intents_and_enforces_owner_boundary() {
    let f = fixture();
    let root = f.directory.path();
    let mut value = authored_workflow(&f);
    let parties: Vec<_> = (0..64)
        .map(|index| {
            json!({"key":format!("owner-{index:02}"),
        "type":"person","name":"SENSITIVE synthetic owner name"})
        })
        .collect();
    let owners: Vec<_> = (0..64)
        .map(|index| {
            json!({"role_id":"owner",
        "party_key":format!("owner-{index:02}"),"rationale":"SENSITIVE synthetic owner rationale"})
        })
        .collect();
    value["parties"] = json!(parties);
    value["items"][0]["owners"] = json!(owners);
    value["items"][0]["milestones"][0]["owners"] = json!([owners[0].clone()]);
    value["items"][0]["history"][0]["actor"]["party_key"] = json!("owner-00");
    value["items"][0]["milestones"][0]["history"][0]["actor"]["party_key"] = json!("owner-00");
    let path = outbound_control_pair(&f, &value);
    let native = std::fs::read(root.join("outbound-native-current.json")).unwrap();
    let mut keys = std::collections::BTreeSet::new();
    let mut create_report = Value::Null;
    for intent in ["create", "update", "close-request"] {
        let selection = outbound_control_selection(json!([{"item_key":"work","intent":intent}]));
        let prepared = forge::poam::workflow_outbound::prepare(
            &path,
            Path::new("outbound-native-current.json"),
            &selection,
            "2026-02-06",
            0,
        )
        .unwrap();
        prepared.verify_inputs().unwrap();
        let report: Value = serde_json::from_slice(prepared.report()).unwrap();
        assert_eq!(report["schema_version"], "forge.integration-change-set/1");
        assert_eq!(report["operation_count"], 1);
        assert_eq!(report["source"]["raw_sha256"], common::sha256_hex(&native));
        assert_eq!(report["source"]["raw_bytes"], native.len());
        assert_eq!(report["selection_sha256"], common::sha256_hex(&selection));
        assert_eq!(report["preparation_due_soon_days"], 0);
        assert_eq!(report["operations"][0]["fields"]["owners"].as_array().unwrap().len(), 64);
        assert_eq!(report["operations"][0]["desired_operation"], intent);
        assert_eq!(report["operations"][0]["fields"]["declared_state"], "planned");
        assert!(report["operations"][0]["remote_preconditions"].is_null());
        assert_eq!(report["remote_apply_authorized"], false);
        assert!(!std::str::from_utf8(prepared.report()).unwrap().contains("SENSITIVE"));
        assert!(
            keys.insert(report["operations"][0]["operation_key"].as_str().unwrap().to_string())
        );
        if intent == "create" {
            create_report = report;
        }
    }
    assert_eq!(keys.len(), 3);
    let selection = outbound_control_selection(json!([{"item_key":"work","intent":"create"}]));
    let native_value: Value = serde_json::from_slice(&native).unwrap();
    let compact = serde_json::to_vec(&native_value).unwrap();
    assert_ne!(compact, native);
    std::fs::write(root.join("outbound-native-current.json"), &compact).unwrap();
    let reformatted = forge::poam::workflow_outbound::prepare(
        &path,
        Path::new("outbound-native-current.json"),
        &selection,
        "2026-02-06",
        0,
    )
    .unwrap();
    let reformatted: Value = serde_json::from_slice(reformatted.report()).unwrap();
    assert_eq!(reformatted["operations"], create_report["operations"]);
    assert_ne!(reformatted["source"]["raw_sha256"], create_report["source"]["raw_sha256"]);
    assert_eq!(
        reformatted["authoring_manifest_sha256"],
        create_report["authoring_manifest_sha256"]
    );
    let mut changed = value.clone();
    changed["items"][0]["target_date"] = json!("2026-03-01");
    changed["items"][0]["description"] = json!("SENSITIVE changed private prose");
    outbound_control_pair(&f, &changed);
    let changed_prepared = forge::poam::workflow_outbound::prepare(
        &path,
        Path::new("outbound-native-current.json"),
        &selection,
        "2026-02-06",
        0,
    )
    .unwrap();
    let changed_report: Value = serde_json::from_slice(changed_prepared.report()).unwrap();
    assert_eq!(
        changed_report["operations"][0]["operation_key"],
        create_report["operations"][0]["operation_key"]
    );
    assert_ne!(
        changed_report["operations"][0]["payload_sha256"],
        create_report["operations"][0]["payload_sha256"]
    );
    assert_ne!(
        changed_report["authoring_manifest_sha256"],
        create_report["authoring_manifest_sha256"]
    );
    changed_prepared.verify_inputs().unwrap();
    let mut too_many = changed.clone();
    too_many["parties"]
        .as_array_mut()
        .unwrap()
        .push(json!({"key":"owner-64","type":"person","name":"SENSITIVE"}));
    too_many["items"][0]["owners"]
        .as_array_mut()
        .unwrap()
        .push(json!({"role_id":"owner","party_key":"owner-64","rationale":"SENSITIVE"}));
    write_json(&path, &too_many);
    let refused = forge::poam::workflow_outbound::prepare(
        &path,
        Path::new("outbound-native-current.json"),
        &selection,
        "2026-02-06",
        0,
    )
    .unwrap_err();
    assert!(refused.to_string().contains("authoring workflow is unsupported or invalid"));
    outbound_control_pair(&f, &value);
    forge::poam::workflow_outbound::prepare(
        &path,
        Path::new("outbound-native-current.json"),
        &selection,
        "2026-02-06",
        0,
    )
    .unwrap()
    .verify_inputs()
    .unwrap();
    assert!(!root.join("change-set.json").exists());
}

/// Exact selected stable keys consume actual distinct finding/risk items; malformed selections fail before capture.
#[test]
#[allow(
    clippy::too_many_lines,
    reason = "one actual two-item source pair distinguishes selection errors from capture errors"
)]
fn outbound_contract_actual_selection_is_complete_explicit_and_closed() {
    let f = fixture();
    let root = f.directory.path();
    let mut value = authored_workflow(&f);
    let declaration = forge::poam::workflow::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
    let inventory = source::load(&root.join("poam.json"), &declaration.source).unwrap();
    let risk = inventory
        .inventory()
        .objects
        .iter()
        .find(|r| r.kind == manifest::SourceKind::Risk)
        .unwrap();
    let mut second = value["items"][0].clone();
    second["key"] = json!("risk-work");
    second["source_refs"] = json!([{"kind":risk.kind,"key":risk.key,"uuid":risk.uuid,
        "result_uuid":risk.result_uuid,"expected_sha256":risk.sha256}]);
    value["items"].as_array_mut().unwrap().push(second);
    let path = outbound_control_pair(&f, &value);
    let selection = outbound_control_selection(json!([{"item_key":"work","intent":"create"},
        {"item_key":"risk-work","intent":"update"}]));
    let complete = forge::poam::workflow_outbound::prepare(
        &path,
        Path::new("outbound-native-current.json"),
        &selection,
        "2026-02-06",
        0,
    )
    .unwrap();
    let report: Value = serde_json::from_slice(complete.report()).unwrap();
    assert_eq!(report["operation_count"], 2);
    assert_eq!(report["operations"][0]["item_key"], "risk-work");
    assert_eq!(report["operations"][1]["item_key"], "work");
    let subset = outbound_control_selection(json!([{"item_key":"work","intent":"close-request"}]));
    let subset = forge::poam::workflow_outbound::prepare(
        &path,
        Path::new("outbound-native-current.json"),
        &subset,
        "2026-02-06",
        0,
    )
    .unwrap();
    let subset: Value = serde_json::from_slice(subset.report()).unwrap();
    assert_eq!(subset["operation_count"], 1);
    assert_eq!(subset["operations"][0]["item_key"], "work");
    assert_eq!(subset["operations"][0]["fields"]["declared_state"], "planned");
    assert_eq!(subset["remote_apply_authorized"], false);
    let unknown = outbound_control_selection(json!([{"item_key":"absent","intent":"create"}]));
    let refused = forge::poam::workflow_outbound::prepare(
        &path,
        Path::new("outbound-native-current.json"),
        &unknown,
        "2026-02-06",
        0,
    )
    .unwrap_err();
    assert!(refused.to_string().contains("selected item identity is absent"));
    let raw = std::fs::read(root.join("outbound-native-current.json")).unwrap();
    std::fs::write(root.join("outbound-native-current.json"), b"not-native-json").unwrap();
    let duplicate = br#"{"schema_version":"forge.poam-outbound-selection/1","schema_version":"forge.poam-outbound-selection/1","operations":[{"item_key":"work","intent":"create"}]}"#;
    let refused = forge::poam::workflow_outbound::prepare(
        &path,
        Path::new("outbound-native-current.json"),
        duplicate,
        "2026-02-06",
        0,
    )
    .unwrap_err();
    assert!(refused.to_string().contains("selection JSON is malformed, duplicate or unbounded"));
    std::fs::write(root.join("outbound-native-current.json"), raw).unwrap();
    complete.verify_inputs().unwrap();
}

/// All seven held original generations reject byte drift; supported-platform replacement retains the old inode.
#[test]
#[allow(
    clippy::too_many_lines,
    reason = "all seven originals and native mutation vectors share one bounded genuine source pair"
)]
fn outbound_contract_all_actual_originals_and_complete_native_values_are_rechecked() {
    let f = fixture();
    let root = f.directory.path();
    let path = outbound_control_pair(&f, &authored_workflow(&f));
    let selection = outbound_control_selection(json!([{"item_key":"work","intent":"update"}]));
    for name in [
        "outbound-authoring-current.json",
        "outbound-native-current.json",
        "assessment-results.json",
        "assessment-plan.json",
        "ssp.json",
        "profile.json",
        "catalog.json",
    ] {
        let held = forge::poam::workflow_outbound::prepare(
            &path,
            Path::new("outbound-native-current.json"),
            &selection,
            "2026-02-06",
            0,
        )
        .unwrap();
        held.verify_inputs().unwrap();
        let current = root.join(name);
        let original = std::fs::read(&current).unwrap();
        let mut changed = original.clone();
        changed.extend_from_slice(b" ");
        std::fs::write(&current, changed).unwrap();
        let refused = held.verify_inputs().unwrap_err().to_string();
        assert!(
            refused.contains(if name.starts_with("outbound-") {
                "actual outbound original generation changed"
            } else {
                "current captured source generation changed"
            }),
            "{name}: {refused}"
        );
        std::fs::write(&current, original).unwrap();
        forge::poam::workflow_outbound::prepare(
            &path,
            Path::new("outbound-native-current.json"),
            &selection,
            "2026-02-06",
            0,
        )
        .unwrap()
        .verify_inputs()
        .unwrap();
    }
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let held = forge::poam::workflow_outbound::prepare(
            &path,
            Path::new("outbound-native-current.json"),
            &selection,
            "2026-02-06",
            0,
        )
        .unwrap();
        let native_path = root.join("outbound-native-current.json");
        let original = std::fs::read(&native_path).unwrap();
        std::fs::rename(&native_path, root.join("retained-old-native.json")).unwrap();
        std::fs::write(&native_path, original).unwrap();
        assert!(
            held.verify_inputs()
                .unwrap_err()
                .to_string()
                .contains("actual outbound original generation changed")
        );
        forge::poam::workflow_outbound::prepare(
            &path,
            Path::new("outbound-native-current.json"),
            &selection,
            "2026-02-06",
            0,
        )
        .unwrap()
        .verify_inputs()
        .unwrap();
    }
    let original = std::fs::read(root.join("outbound-native-current.json")).unwrap();
    let native: Value = serde_json::from_slice(&original).unwrap();
    for mutation in ["unknown", "prose", "hash"] {
        let mut changed = native.clone();
        match mutation {
            "unknown" => {
                changed["plan-of-action-and-milestones"]["private-extra"] = json!("SENSITIVE");
            }
            "prose" => {
                changed["plan-of-action-and-milestones"]["metadata"]["title"] =
                    json!("SENSITIVE altered title");
            }
            _ => {
                let property = changed["plan-of-action-and-milestones"]["metadata"]["props"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|p| p["name"] == "source-file-sha256")
                    .unwrap();
                property["value"] = json!("0".repeat(64));
            }
        }
        assert_ne!(changed, native, "{mutation}");
        write_json(&root.join("outbound-native-current.json"), &changed);
        let refused = forge::poam::workflow_outbound::prepare(
            &path,
            Path::new("outbound-native-current.json"),
            &selection,
            "2026-02-06",
            0,
        )
        .unwrap_err();
        assert!(
            refused
                .to_string()
                .contains("native original differs from the complete supported current projection"),
            "{mutation}"
        );
        std::fs::write(root.join("outbound-native-current.json"), &original).unwrap();
        forge::poam::workflow_outbound::prepare(
            &path,
            Path::new("outbound-native-current.json"),
            &selection,
            "2026-02-06",
            0,
        )
        .unwrap()
        .verify_inputs()
        .unwrap();
    }
    assert!(!root.join("change-set.json").exists());
}

/// All-platform CLI stdout is complete deterministic local intent, including overdue work and close requests.
#[test]
#[allow(
    clippy::too_many_lines,
    reason = "one complete source pair checks deterministic minimized CLI output across dates"
)]
fn outbound_contract_cli_stdout_uses_closed_nondefault_companions_and_never_exit1() {
    let f = fixture();
    let root = f.directory.path();
    let path = outbound_control_pair(&f, &authored_workflow(&f));
    let selection =
        outbound_control_selection(json!([{"item_key":"work","intent":"close-request"}]));
    std::fs::write(root.join("outbound-selection-current.json"), &selection).unwrap();
    let originals = outbound_control_originals(&f);
    let first = outbound_control_cli(
        &f,
        "outbound-native-current.json",
        "outbound-selection-current.json",
        "2026-02-06",
        &[],
    );
    let second = outbound_control_cli(
        &f,
        "outbound-native-current.json",
        "outbound-selection-current.json",
        "2026-02-06",
        &["--due-soon-days", "0"],
    );
    assert_eq!(first.status.code(), Some(0), "{}", String::from_utf8_lossy(&first.stderr));
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(first.stderr, [] as [u8; 0]);
    assert_eq!(first.stdout, second.stdout);
    assert!(first.stdout.ends_with(b"\n"));
    let prepared = forge::poam::workflow_outbound::prepare(
        &path,
        Path::new("outbound-native-current.json"),
        &selection,
        "2026-02-06",
        0,
    )
    .unwrap();
    assert_eq!(first.stdout, prepared.report());
    let report: Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(report["operation_count"], 1);
    assert_eq!(report["selection_sha256"], common::sha256_hex(&selection));
    assert_eq!(report["preparation_due_soon_days"], 0);
    assert_eq!(report["operations"][0]["desired_operation"], "close-request");
    assert_eq!(report["remote_apply_authorized"], false);
    assert!(report["operations"][0]["remote_preconditions"].is_null());
    assert!(!String::from_utf8(first.stdout).unwrap().contains("SENSITIVE"));
    let overdue = outbound_control_cli(
        &f,
        "outbound-native-current.json",
        "outbound-selection-current.json",
        "2028-02-29",
        &[],
    );
    assert_eq!(overdue.status.code(), Some(0));
    let overdue: Value = serde_json::from_slice(&overdue.stdout).unwrap();
    assert_eq!(overdue["preparation_as_of"], "2028-02-29");
    assert_eq!(overdue["operations"][0]["fields"]["declared_state"], "planned");
    for (path, raw) in originals {
        assert_eq!(std::fs::read(path).unwrap(), raw);
    }
    assert!(!root.join("change-set.json").exists());
}

/// Full-path input conflicts are refused everywhere; file success and unsupported refusal have separate branches.
#[test]
#[allow(
    clippy::too_many_lines,
    reason = "eight input collisions and the platform publication split are one bounded CLI contract"
)]
fn outbound_contract_cli_preflights_all_eight_originals_and_qualifies_file_publication() {
    let f = fixture();
    let root = f.directory.path();
    outbound_control_pair(&f, &authored_workflow(&f));
    let selection = outbound_control_selection(json!([{"item_key":"work","intent":"create"}]));
    std::fs::write(root.join("outbound-selection-current.json"), selection).unwrap();
    let originals = outbound_control_originals(&f);
    let positive = outbound_control_cli(
        &f,
        "outbound-native-current.json",
        "outbound-selection-current.json",
        "2026-02-06",
        &[],
    );
    assert_eq!(positive.status.code(), Some(0), "{}", String::from_utf8_lossy(&positive.stderr));
    let entries_before = std::fs::read_dir(root).unwrap().count();
    for (path, _) in &originals {
        // Keep the portable suffix valid so it cannot preempt the full input-path check.
        // An existing destination can still preempt that check on case-insensitive volumes.
        let stem = path.file_stem().unwrap().to_str().unwrap().to_ascii_uppercase();
        let name = format!("{stem}.json");
        let refused = outbound_control_cli(
            &f,
            "outbound-native-current.json",
            "outbound-selection-current.json",
            "2026-02-06",
            &["--report", &name],
        );
        assert_eq!(refused.status.code(), Some(2), "{name}");
        assert_eq!(refused.stdout, [] as [u8; 0]);
        assert_eq!(std::fs::read_dir(root).unwrap().count(), entries_before);
    }
    for name in [
        "../outside.json",
        "nested/report.json",
        "NUL.json",
        "bad:name.json",
        "unicode-é.json",
        "wrong.html",
    ] {
        let refused = outbound_control_cli(
            &f,
            "outbound-native-current.json",
            "outbound-selection-current.json",
            "2026-02-06",
            &["--report", name],
        );
        assert_eq!(refused.status.code(), Some(2), "{name}");
        assert_eq!(refused.stdout, [] as [u8; 0]);
        assert_eq!(std::fs::read_dir(root).unwrap().count(), entries_before);
    }
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let published = outbound_control_cli(
            &f,
            "outbound-native-current.json",
            "outbound-selection-current.json",
            "2026-02-06",
            &["--report", "change-set.json"],
        );
        assert_eq!(
            published.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&published.stderr)
        );
        assert_eq!(published.stdout, [] as [u8; 0]);
        assert_eq!(std::fs::read(root.join("change-set.json")).unwrap(), positive.stdout);
        let refused = outbound_control_cli(
            &f,
            "outbound-native-current.json",
            "outbound-selection-current.json",
            "2026-02-06",
            &["--report", "change-set.json"],
        );
        assert_eq!(refused.status.code(), Some(2));
        assert_eq!(refused.stdout, [] as [u8; 0]);
        assert_eq!(std::fs::read(root.join("change-set.json")).unwrap(), positive.stdout);
        // Equal basenames in genuinely different explicit roots are not input aliases.
        let other_root = tempfile::tempdir().unwrap();
        let manifest_path = root.join("outbound-authoring-current.json");
        let other = run(
            other_root.path(),
            &[
                "poam",
                "outbound",
                "--manifest",
                manifest_path.to_str().unwrap(),
                "--native",
                "outbound-native-current.json",
                "--selection",
                "outbound-selection-current.json",
                "--as-of",
                "2026-02-06",
                "--output-root",
                other_root.path().to_str().unwrap(),
                "--report",
                "outbound-native-current.json",
            ],
        );
        assert_eq!(other.status.code(), Some(0), "{}", String::from_utf8_lossy(&other.stderr));
        assert_eq!(other.stdout, [] as [u8; 0]);
        assert_eq!(
            std::fs::read(other_root.path().join("outbound-native-current.json")).unwrap(),
            positive.stdout
        );
        assert_eq!(std::fs::read_dir(other_root.path()).unwrap().count(), 1);
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let refused = outbound_control_cli(
            &f,
            "outbound-native-current.json",
            "outbound-selection-current.json",
            "2026-02-06",
            &["--report", "change-set.json"],
        );
        assert_eq!(refused.status.code(), Some(2));
        assert_eq!(refused.stdout, [] as [u8; 0]);
        assert!(!root.join("change-set.json").exists());
        assert_eq!(std::fs::read_dir(root).unwrap().count(), entries_before);
    }
    for (path, raw) in originals {
        assert_eq!(std::fs::read(path).unwrap(), raw);
    }
}

/// Positive real CLI preparation precedes exact native/source/selection failures and closed argument refusal.
#[test]
#[allow(
    clippy::too_many_lines,
    reason = "one real paired source control checks refusal precedence without fabricated authority"
)]
fn outbound_contract_cli_rejects_tampering_stale_sources_and_malformed_requests_without_output() {
    let f = fixture();
    let root = f.directory.path();
    outbound_control_pair(&f, &authored_workflow(&f));
    let selection = outbound_control_selection(json!([{"item_key":"work","intent":"update"}]));
    std::fs::write(root.join("outbound-selection-current.json"), &selection).unwrap();
    let original_native = std::fs::read(root.join("outbound-native-current.json")).unwrap();
    let positive = outbound_control_cli(
        &f,
        "outbound-native-current.json",
        "outbound-selection-current.json",
        "2026-02-06",
        &[],
    );
    assert_eq!(positive.status.code(), Some(0), "{}", String::from_utf8_lossy(&positive.stderr));
    let mut changed: Value = serde_json::from_slice(&original_native).unwrap();
    changed["plan-of-action-and-milestones"]["private-extra"] = json!("SENSITIVE unknown field");
    write_json(&root.join("outbound-native-current.json"), &changed);
    let refused = outbound_control_cli(
        &f,
        "outbound-native-current.json",
        "outbound-selection-current.json",
        "2026-02-06",
        &["--report", "refused.json"],
    );
    assert_eq!(refused.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&refused.stderr).contains("complete supported current projection")
    );
    assert_eq!(refused.stdout, [] as [u8; 0]);
    assert!(!root.join("refused.json").exists());
    std::fs::write(root.join("outbound-native-current.json"), &original_native).unwrap();
    for name in [
        "assessment-results.json",
        "assessment-plan.json",
        "ssp.json",
        "profile.json",
        "catalog.json",
    ] {
        assert_eq!(
            outbound_control_cli(
                &f,
                "outbound-native-current.json",
                "outbound-selection-current.json",
                "2026-02-06",
                &[]
            )
            .status
            .code(),
            Some(0)
        );
        let path = root.join(name);
        let raw = std::fs::read(&path).unwrap();
        let mut changed = raw.clone();
        changed.extend_from_slice(b" ");
        std::fs::write(&path, changed).unwrap();
        let refused = outbound_control_cli(
            &f,
            "outbound-native-current.json",
            "outbound-selection-current.json",
            "2026-02-06",
            &["--report", "refused.json"],
        );
        assert_eq!(refused.status.code(), Some(2), "{name}");
        assert!(
            String::from_utf8_lossy(&refused.stderr)
                .contains("actual current source or workflow preparation is invalid"),
            "{name}"
        );
        assert_eq!(refused.stdout, [] as [u8; 0]);
        assert!(!root.join("refused.json").exists());
        std::fs::write(&path, raw).unwrap();
    }
    std::fs::write(root.join("outbound-native-current.json"), b"not-native-json").unwrap();
    std::fs::write(root.join("outbound-selection-current.json"), br#"{"schema_version":"forge.poam-outbound-selection/1","schema_version":"forge.poam-outbound-selection/1","operations":[{"item_key":"work","intent":"update"}]}"#).unwrap();
    let refused = outbound_control_cli(
        &f,
        "outbound-native-current.json",
        "outbound-selection-current.json",
        "2026-02-06",
        &["--report", "refused.json"],
    );
    assert_eq!(refused.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&refused.stderr)
            .contains("selection JSON is malformed, duplicate or unbounded")
    );
    assert_eq!(refused.stdout, [] as [u8; 0]);
    assert!(!root.join("refused.json").exists());
    std::fs::write(root.join("outbound-native-current.json"), &original_native).unwrap();
    let mut extended_selection: Value = serde_json::from_slice(&selection).unwrap();
    extended_selection["remote_target"] = json!("https://example.invalid/private");
    write_json(&root.join("outbound-selection-current.json"), &extended_selection);
    let refused = outbound_control_cli(
        &f,
        "outbound-native-current.json",
        "outbound-selection-current.json",
        "2026-02-06",
        &["--report", "refused.json"],
    );
    assert_eq!(refused.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&refused.stderr)
            .contains("selection is not the closed item-intent contract")
    );
    assert_eq!(refused.stdout, [] as [u8; 0]);
    assert!(!root.join("refused.json").exists());
    std::fs::write(root.join("outbound-selection-current.json"), &selection).unwrap();
    let unknown = outbound_control_selection(json!([{"item_key":"absent","intent":"create"}]));
    std::fs::write(root.join("outbound-selection-current.json"), unknown).unwrap();
    let refused = outbound_control_cli(
        &f,
        "outbound-native-current.json",
        "outbound-selection-current.json",
        "2026-02-06",
        &["--report", "refused.json"],
    );
    assert_eq!(refused.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&refused.stderr).contains("selected item identity is absent"));
    assert!(!root.join("refused.json").exists());
    std::fs::write(root.join("outbound-selection-current.json"), &selection).unwrap();
    let manifest_path = root.join("outbound-authoring-current.json");
    for missing in ["--manifest", "--native", "--selection", "--as-of", "--output-root"] {
        let mut args = vec!["poam", "outbound"];
        for (flag, value) in [
            ("--manifest", manifest_path.to_str().unwrap()),
            ("--native", "outbound-native-current.json"),
            ("--selection", "outbound-selection-current.json"),
            ("--as-of", "2026-02-06"),
            ("--output-root", root.to_str().unwrap()),
        ] {
            if flag != missing {
                args.extend([flag, value]);
            }
        }
        let refused = run(root, &args);
        assert_eq!(refused.status.code(), Some(2), "{missing}");
        assert_eq!(refused.stdout, [] as [u8; 0]);
    }
    for date in ["2026-02-29", "2026-2-06", "2026-02-06T00:00:00Z", "+10000-01-01"] {
        let refused = outbound_control_cli(
            &f,
            "outbound-native-current.json",
            "outbound-selection-current.json",
            date,
            &["--report", "refused.json"],
        );
        assert_eq!(refused.status.code(), Some(2), "{date}");
        assert_eq!(refused.stdout, [] as [u8; 0]);
        assert!(!root.join("refused.json").exists());
    }
    for extra in [
        vec!["--due-soon-days", "366"],
        vec!["--apply"],
        vec!["--allow-terminal"],
        vec!["--target-url", "https://example.invalid"],
        vec!["--connector", "tracker"],
        vec!["--excerpt", "secret"],
    ] {
        let refused = outbound_control_cli(
            &f,
            "outbound-native-current.json",
            "outbound-selection-current.json",
            "2026-02-06",
            &extra,
        );
        assert_eq!(refused.status.code(), Some(2));
        assert_eq!(refused.stdout, [] as [u8; 0]);
    }
    for native in ["../outside.json", "nested/native.json", "outbound-native-current.JSON"] {
        let refused =
            outbound_control_cli(&f, native, "outbound-selection-current.json", "2026-02-06", &[]);
        assert_eq!(refused.status.code(), Some(2));
        assert_eq!(refused.stdout, [] as [u8; 0]);
    }
    for selection_name in
        ["../outside.json", "nested/selection.json", "outbound-selection-current.JSON"]
    {
        let refused = outbound_control_cli(
            &f,
            "outbound-native-current.json",
            selection_name,
            "2026-02-06",
            &[],
        );
        assert_eq!(refused.status.code(), Some(2));
        assert_eq!(refused.stdout, [] as [u8; 0]);
    }
    assert_eq!(
        outbound_control_cli(
            &f,
            "outbound-native-current.json",
            "outbound-selection-current.json",
            "2026-02-06",
            &[]
        )
        .status
        .code(),
        Some(0)
    );
    assert_eq!(std::fs::read(root.join("outbound-native-current.json")).unwrap(), original_native);
    assert_eq!(std::fs::read(root.join("outbound-selection-current.json")).unwrap(), selection);
}

/// Debug formatting of an admitted real capture reveals only the complete report byte count.
#[test]
fn outbound_contract_debug_keeps_actual_originals_and_private_paths_opaque() {
    let f = fixture();
    let value = authored_workflow(&f);
    let path = outbound_control_pair(&f, &value);
    let selection = outbound_control_selection(json!([{"item_key":"work","intent":"create"}]));
    let prepared = forge::poam::workflow_outbound::prepare(
        &path,
        Path::new("outbound-native-current.json"),
        &selection,
        "2026-02-06",
        0,
    )
    .unwrap();
    prepared.verify_inputs().unwrap();
    let actual = format!("{prepared:?}");
    let expected = format!("PreparedOutbound {{ report_bytes: {}, .. }}", prepared.report().len());
    assert!(actual == expected, "admitted capture Debug must remain opaque");
}

/// Actual synthetic source/native/linkage files; no detached prepared proof is fabricated.
struct S4ControlFixture {
    /// Retain the owning existing native five-file fixture directory.
    _owner: Fixture,
    /// Canonical actual plan directory, optionally different from the fixture/cwd root.
    root: std::path::PathBuf,
    /// Actual complete authored workflow value, with source tuples from the old helper.
    plan: Value,
    /// Actual closed companion value, persisted and raw-pinned after every edit.
    links: Value,
    /// Actual native linkage declaration, consumed by the real fresh loader.
    linkage: Value,
    /// Actual original authoring declaration path.
    plan_path: std::path::PathBuf,
    /// Actual nested companion path, resolved from plan root rather than cwd.
    links_path: std::path::PathBuf,
    /// Actual flat or nested native linkage manifest.
    linkage_path: std::path::PathBuf,
    /// Actual native requirement Catalog path; flat profile shares the five-source Catalog.
    native_catalog: std::path::PathBuf,
    /// Actual schema-validated native Component Definition path.
    component: std::path::PathBuf,
    /// Actual local evidence payload; its bytes must never appear in reports or Debug.
    evidence: std::path::PathBuf,
}

/// Build a real native Component Definition over the actual fixture's exact control and statement IDs.
fn s4_control_component() -> Value {
    json!({"component-definition":{"uuid":"22222222-2222-4222-8222-222222222222",
        "metadata":{"title":"SENSITIVE IMPLEMENTATION TITLE","last-modified":"2026-01-01T00:00:00Z","version":"1.0.0","oscal-version":"1.2.3"},
        "components":[{"uuid":"33333333-3333-4333-8333-333333333333","type":"software","title":"SENSITIVE COMPONENT","description":"SENSITIVE IMPLEMENTATION PROSE",
            "control-implementations":[{"uuid":"44444444-4444-4444-8444-444444444444","source":"catalog.json","description":"SENSITIVE IMPLEMENTATION SET",
                "implemented-requirements":[{"uuid":"55555555-5555-4555-8555-555555555555","control-id":CONTROL,"description":"SENSITIVE REQUIREMENT PROSE",
                    "statements":[{"statement-id":STATEMENT,"uuid":"66666666-6666-4666-8666-666666666666","description":"SENSITIVE STATEMENT PROSE"}]}]}]}]}})
}

/// Add coherent item/milestone closure assertions; actual files and distinct reviewer remain explicit.
fn s4_control_close(plan: &mut Value, href: &str, hash: &str) {
    plan["roles"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"reviewer","title":"SENSITIVE REVIEWER ROLE"}));
    plan["parties"].as_array_mut().unwrap().push(
        json!({"key":"independent-reviewer","type":"person","name":"SENSITIVE REVIEWER NAME"}),
    );
    let close = |record: &mut Value| {
        record["history"].as_array_mut().unwrap().push(json!({"key":"start","actor":{"role_id":"owner","party_key":"remediator"},
            "at":"2026-01-04T00:00:00Z","from":"planned","to":"in-progress","rationale":"SENSITIVE START RATIONALE","closure":null}));
        record["history"].as_array_mut().unwrap().push(json!({"key":"complete","actor":{"role_id":"owner","party_key":"remediator"},
            "at":"2026-01-20T11:00:00Z","from":"in-progress","to":"completed-asserted","rationale":"SENSITIVE COMPLETION RATIONALE",
            "closure":{"reviewer":{"role_id":"reviewer","party_key":"independent-reviewer"},"reviewed_at":"2026-01-20T12:00:00Z",
                "rationale":"SENSITIVE CLOSURE REVIEW","evidence":[{"key":"proof","href":href,"expected_sha256":hash}]}}));
        record["state"] = json!("completed-asserted");
    };
    close(&mut plan["items"][0]);
    close(&mut plan["items"][0]["milestones"][0]);
    plan["items"][0]["milestones"][0]["history"][2]["at"] = json!("2026-01-20T10:00:00Z");
    plan["items"][0]["milestones"][0]["history"][2]["closure"]["reviewed_at"] =
        json!("2026-01-20T10:30:00Z");
}

/// Persist exact declarations and update only the explicit companion raw linkage-manifest pin.
fn s4_control_persist(fixture: &mut S4ControlFixture) {
    let raw = write_json(&fixture.linkage_path, &fixture.linkage);
    fixture.links["linkage"]["expected_sha256"] = json!(common::sha256_hex(&raw));
    write_json(&fixture.plan_path, &fixture.plan);
    write_json(&fixture.links_path, &fixture.links);
}

/// Create real five-source/native/current-local inputs; nested mode also uses a nondefault plan directory.
fn s4_control_fixture(terminal: bool, nested: bool) -> S4ControlFixture {
    let owner = fixture();
    let mut plan = authored_workflow(&owner);
    let original_root = owner.directory.path().canonicalize().unwrap();
    let root = if nested {
        let path = original_root.join("plan-root");
        std::fs::create_dir(&path).unwrap();
        for name in [
            "assessment-results.json",
            "assessment-plan.json",
            "ssp.json",
            "profile.json",
            "catalog.json",
        ] {
            std::fs::copy(original_root.join(name), path.join(name)).unwrap();
        }
        path
    } else {
        original_root
    };
    let base = if nested { root.join("linked") } else { root.clone() };
    if nested {
        std::fs::create_dir(&base).unwrap();
    }
    let native_catalog = base.join("catalog.json");
    if nested {
        std::fs::copy(root.join("catalog.json"), &native_catalog).unwrap();
    }
    let component = base.join("component.json");
    write_json(&component, &s4_control_component());
    std::fs::create_dir(base.join("evidence")).unwrap();
    let evidence = base.join("evidence/proof.bin");
    std::fs::write(&evidence, b"PRIVATE S4 EVIDENCE CONTENT\n").unwrap();
    let hash = common::sha256_file(&evidence);
    let href = if nested { "linked/evidence/proof.bin" } else { "evidence/proof.bin" };
    if terminal {
        s4_control_close(&mut plan, href, &hash);
    }
    let linkage = json!({"schema_version":"forge.linkage/1",
        "project":{"key":"s4-project","title":"SENSITIVE PROJECT TITLE","expiring_window_days":30,"max_evidence_bytes":1_048_576,"approved_uri_schemes":["vault+corp"]},
        "reviewers":[{"key":"reviewer","name":"SENSITIVE LINKAGE REVIEWER"}],
        "requirement_resources":[{"key":"requirements","type":"catalog","artifact":"catalog.json","href":"catalog.json","expected_sha256":common::sha256_file(&native_catalog)}],
        "implementation_resource":{"key":"implementation","type":"component-definition","artifact":"component.json","href":"component.json","expected_sha256":common::sha256_file(&component)},
        "evidence_roots":[{"key":"local","path":"evidence"}],
        "evidence":[{"key":"record","title":"SENSITIVE EVIDENCE TITLE","evidence_type":"test-record","owner":"SENSITIVE EVIDENCE OWNER",
            "collected_at":"2026-01-10T00:00:00Z","valid_through":"2026-12-31","sensitivity_label":"restricted","source_label":"SENSITIVE SOURCE LABEL",
            "location":{"kind":"local","root_key":"local","path":"proof.bin","expected_sha256":hash,"expected_size":std::fs::metadata(&evidence).unwrap().len()}}],
        "links":[{"key":"access-link","requirements":[{"resource_key":"requirements","type":"control","id_ref":CONTROL},{"resource_key":"requirements","type":"statement","id_ref":STATEMENT}],
            "implementations":[{"type":"implemented-requirement","id_ref":"55555555-5555-4555-8555-555555555555"},{"type":"statement","id_ref":"66666666-6666-4666-8666-666666666666"}],
            "evidence_keys":["record"],"evidence_required":true,"responsible_role":"SENSITIVE RESPONSIBLE ROLE","implementation_status":"implemented",
            "review":{"reviewer_key":"reviewer","reviewed_at":"2026-01-11T00:00:00Z","rationale":"SENSITIVE LINKAGE RATIONALE"},"impact_finding_ids":[],"policy_version_keys":[]}]});
    let bindings: Vec<_> = if terminal {
        [None, Some("step")].into_iter().map(|step| {
        json!({"locator":{"item_key":"work","milestone_key":step,"event_key":"complete","assertion_evidence_key":"proof"},"link_key":"access-link","evidence_key":"record"})
    }).collect()
    } else {
        Vec::new()
    };
    let links = json!({"schema_version":"forge.poam-evidence-links/1","plan_key":plan["document"]["key"],
        "linkage":{"artifact":if nested { "linked/linkage.json" } else { "linkage.json" },"expected_sha256":"0".repeat(64),"project_key":"s4-project"},"bindings":bindings});
    std::fs::create_dir(root.join("companions")).unwrap();
    let mut fixture = S4ControlFixture {
        _owner: owner,
        plan_path: root.join("poam-s4.json"),
        links_path: root.join("companions/links.json"),
        linkage_path: base.join("linkage.json"),
        root,
        plan,
        links,
        linkage,
        native_catalog,
        component,
        evidence,
    };
    s4_control_persist(&mut fixture);
    fixture
}

/// Use the public current-source inspector, never a forged proof or supplied native index.
fn s4_control_prepare(
    fixture: &S4ControlFixture,
) -> forge::poam::workflow_evidence::PreparedEvidence {
    let prepared = forge::poam::workflow_evidence::prepare(
        &fixture.plan_path,
        &fixture.links_path,
        "2026-01-15",
    )
    .unwrap();
    prepared.verify_inputs().unwrap();
    prepared
}

/// Read complete JSON, apply the consumed closed schema and reconcile both complete partitions.
fn s4_control_report(prepared: &forge::poam::workflow_evidence::PreparedEvidence) -> Value {
    let report: Value = serde_json::from_slice(prepared.report()).unwrap();
    let schema: Value = serde_json::from_str(include_str!(
        "../schemas/forge.poam-evidence-inspection-1.schema.json"
    ))
    .unwrap();
    assert!(jsonschema::validator_for(&schema).unwrap().is_valid(&report));
    let summary = &report["summary"];
    let rows = report["rows"].as_array().unwrap().len() as u64;
    let sum =
        |names: &[&str]| names.iter().map(|name| summary[name].as_u64().unwrap()).sum::<u64>();
    assert_eq!(sum(&["matched", "unbound", "mismatched", "binding_unavailable"]), rows);
    assert_eq!(
        sum(&[
            "current",
            "expiring",
            "expired",
            "changed",
            "unavailable",
            "unverified_uri",
            "freshness_unmeasured"
        ]),
        rows
    );
    assert_eq!(summary["closure_assertions"], rows);
    assert_eq!(summary["assertion_rows"], rows);
    assert_eq!(report["terminal_admitted"], false);
    assert_eq!(report["artifact_validated"], false);
    report
}

/// Snapshot every held original generation as test evidence; native Catalog reuse is deduplicated by full path.
fn s4_control_originals(fixture: &S4ControlFixture) -> Vec<(std::path::PathBuf, Vec<u8>)> {
    let mut paths: std::collections::BTreeSet<_> = [
        fixture.plan_path.clone(),
        fixture.links_path.clone(),
        fixture.linkage_path.clone(),
        fixture.native_catalog.clone(),
        fixture.component.clone(),
        fixture.evidence.clone(),
    ]
    .into_iter()
    .collect();
    for name in [
        "assessment-results.json",
        "assessment-plan.json",
        "ssp.json",
        "profile.json",
        "catalog.json",
    ] {
        paths.insert(fixture.root.join(name));
    }
    paths
        .into_iter()
        .map(|path| {
            let bytes = std::fs::read(&path).unwrap();
            (path, bytes)
        })
        .collect()
}

/// Invoke the actual public command from an explicit cwd, using the plan-relative nested companion contract.
fn s4_control_cli(fixture: &S4ControlFixture, cwd: &Path, report: Option<&str>) -> Output {
    let mut args = vec![
        "poam",
        "evidence",
        "--manifest",
        fixture.plan_path.to_str().unwrap(),
        "--links",
        "companions/links.json",
        "--as-of",
        "2026-01-15",
    ];
    if let Some(name) = report {
        args.extend(["--report", name]);
    }
    run(cwd, &args)
}

/// Require an invalid no-output CLI result with redacted path/content categories, preserving exact stdout emptiness.
fn s4_control_invalid(output: &Output, fixture: &S4ControlFixture) {
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stdout, [] as [u8; 0]);
    let text = String::from_utf8_lossy(&output.stderr);
    for private in ["SENSITIVE", "PRIVATE S4", fixture.root.to_str().unwrap()] {
        assert!(!text.contains(private));
    }
}

/// Actual matched terminal references remain review-only; complete future/milestone rows are deterministic and schema-valid.
#[test]
fn s4_actual_matched_terminal_rows_keep_complete_counts_and_no_native_authority() {
    let f = s4_control_fixture(true, false);
    let originals = s4_control_originals(&f);
    let one = s4_control_prepare(&f);
    let two = s4_control_prepare(&f);
    assert_eq!(one.report(), two.report());
    assert!(one.review_required());
    let report = s4_control_report(&one);
    let summary = &report["summary"];
    for (field, expected) in [
        ("items", 1),
        ("milestones", 1),
        ("history_events", 6),
        ("closure_assertions", 2),
        ("bindings_supplied", 2),
        ("assertion_rows", 2),
        ("distinct_referenced_evidence", 1),
        ("distinct_captured_local_evidence", 1),
        ("linkage_evidence", 1),
        ("linkage_links", 1),
        ("matched", 2),
        ("current", 2),
        ("future_assertions", 2),
        ("captured_original_generations", 10),
    ] {
        assert_eq!(summary[field], expected, "{field}");
    }
    let rows = report["rows"].as_array().unwrap();
    assert!(rows[0]["locator"]["milestone_key"].is_null());
    assert_eq!(rows[1]["locator"]["milestone_key"], "step");
    assert_ne!(rows[0]["item_uuid"], rows[1]["milestone_uuid"]);
    for row in rows {
        assert_eq!(row["match_status"], "matched");
        assert_eq!(row["local_bytes_revalidated"], true);
        assert_eq!(row["asserted_after_as_of"], true);
    }
    let rendered = String::from_utf8(one.report().to_vec()).unwrap();
    let debug = format!("{one:?}");
    for private in [
        "PRIVATE S4 EVIDENCE CONTENT",
        "SENSITIVE",
        "remediator",
        "independent-reviewer",
        "evidence/proof.bin",
        f.root.to_str().unwrap(),
    ] {
        assert!(!rendered.contains(private));
        assert!(!debug.contains(private));
    }
    assert!(!debug.contains("finding-unsatisfied"));
    let refused = forge::poam::workflow::prepare(
        &f.plan_path,
        &serde_json::to_vec(&f.plan).unwrap(),
        "2026-01-15",
        0,
        None,
    )
    .unwrap_err();
    assert!(refused.to_string().contains("pending recorded closure disposition"));
    let cli = s4_control_cli(&f, &f.root, None);
    assert_eq!(cli.status.code(), Some(1));
    assert_eq!(cli.stdout, one.report());
    for (path, bytes) in originals {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
}

/// A real nonterminal no-assertion inspection has zero action, but still captures unused linkage/source originals completely.
#[test]
fn s4_actual_nonterminal_no_assertions_is_complete_zero_action_stdout() {
    let f = s4_control_fixture(false, false);
    let prepared = s4_control_prepare(&f);
    assert!(!prepared.review_required());
    let report = s4_control_report(&prepared);
    assert_eq!(report["rows"], json!([]));
    assert_eq!(report["review_required"], false);
    assert_eq!(report["summary"]["history_events"], 2);
    assert_eq!(report["summary"]["bindings_supplied"], 0);
    assert_eq!(report["summary"]["captured_original_generations"], 10);
    let output = s4_control_cli(&f, &f.root, None);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, prepared.report());
    let native = forge::poam::workflow::prepare(
        &f.plan_path,
        &serde_json::to_vec(&f.plan).unwrap(),
        "2026-01-15",
        0,
        None,
    )
    .unwrap();
    native.verify_inputs().unwrap();
}

/// Positive actual joins precede assertion path/hash, membership, unresolved-key and approved/date/observed mismatch vectors.
#[test]
fn s4_actual_match_and_freshness_partitions_preserve_independent_observations() {
    let mut f = s4_control_fixture(true, false);
    let positive = s4_control_report(&s4_control_prepare(&f));
    assert_eq!(positive["summary"]["matched"], 2);
    let original_plan = f.plan.clone();
    let original_links = f.links.clone();
    let original_linkage = f.linkage.clone();
    for (field, value, expected) in [
        ("expected_sha256", "0".repeat(64), "hash-mismatch"),
        ("href", "unrelated/proof.bin".to_string(), "href-mismatch"),
    ] {
        f.plan = original_plan.clone();
        f.plan["items"][0]["history"][2]["closure"]["evidence"][0][field] = json!(value);
        s4_control_persist(&mut f);
        let report = s4_control_report(&s4_control_prepare(&f));
        assert_eq!(report["rows"][0]["match_status"], expected);
        assert_eq!(report["rows"][0]["freshness"], "current");
        assert_eq!(report["rows"][0]["observed_sha256"], common::sha256_file(&f.evidence));
        assert_eq!(report["rows"][0]["local_bytes_revalidated"], true);
        assert_eq!(report["summary"]["mismatched"], 1);
        assert_eq!(report["summary"]["matched"], 1);
        assert_eq!(report["summary"]["current"], 2);
    }
    f.plan = original_plan.clone();
    f.linkage["links"][0]["evidence_keys"] = json!([]);
    f.linkage["links"][0]["evidence_required"] = json!(false);
    s4_control_persist(&mut f);
    let report = s4_control_report(&s4_control_prepare(&f));
    assert_eq!(report["summary"]["mismatched"], 2);
    assert_eq!(report["rows"][0]["match_status"], "wrong-link-membership");
    assert_eq!(report["rows"][0]["observed_sha256"], common::sha256_file(&f.evidence));
    assert_eq!(report["rows"][0]["local_bytes_revalidated"], false);
    f.linkage = original_linkage.clone();
    f.links["bindings"][0]["evidence_key"] = json!("unresolved");
    s4_control_persist(&mut f);
    let report = s4_control_report(&s4_control_prepare(&f));
    assert_eq!(report["summary"]["binding_unavailable"], 1);
    assert_eq!(report["summary"]["freshness_unmeasured"], 1);
    assert!(report["rows"][0]["freshness"].is_null());
    f.links = original_links.clone();
    for (date, expected) in
        [("2026-01-15", "expired"), ("2026-02-14", "expiring"), ("+12345-01-01", "current")]
    {
        f.linkage["evidence"][0]["valid_through"] = json!(date);
        s4_control_persist(&mut f);
        let report = s4_control_report(&s4_control_prepare(&f));
        assert_eq!(report["summary"]["matched"], 2);
        assert_eq!(report["rows"][0]["freshness"], expected);
        assert_eq!(report["rows"][0]["recorded_valid_through"], date);
    }
    f.linkage = original_linkage.clone();
    f.linkage["evidence"][0]["location"]["expected_size"] = json!(u64::MAX);
    s4_control_persist(&mut f);
    let report = s4_control_report(&s4_control_prepare(&f));
    assert_eq!(report["rows"][0]["approved_size"], u64::MAX);
    assert_eq!(report["summary"]["changed"], 2);
    assert_eq!(report["summary"]["mismatched"], 2);
    f.linkage = original_linkage;
    s4_control_persist(&mut f);
    s4_control_prepare(&f);
    std::fs::write(&f.evidence, b"CHANGED LOCAL OBSERVATION\n").unwrap();
    let report = s4_control_report(&s4_control_prepare(&f));
    assert_eq!(report["summary"]["changed"], 2);
    assert_eq!(report["summary"]["mismatched"], 2);
    assert_ne!(report["rows"][0]["approved_sha256"], report["rows"][0]["observed_sha256"]);
}

/// Every original actual byte generation is rechecked; unavailable paths cannot become present behind the sealed proof.
#[test]
fn s4_actual_original_bytes_identities_and_absence_are_rechecked() {
    let mut f = s4_control_fixture(true, false);
    let originals = s4_control_originals(&f);
    for (path, bytes) in &originals {
        let prepared = s4_control_prepare(&f);
        let mut changed = bytes.clone();
        changed.push(b' ');
        std::fs::write(path, changed).unwrap();
        let refused = prepared.verify_inputs().unwrap_err();
        assert!(refused.to_string().contains("original inputs changed or became unsafe"));
        assert!(!refused.to_string().contains(f.root.to_str().unwrap()));
        std::fs::write(path, bytes).unwrap();
    }
    #[cfg(unix)]
    for (path, bytes) in &originals {
        let prepared = s4_control_prepare(&f);
        let held = path.with_extension("s4-held");
        std::fs::rename(path, &held).unwrap();
        std::fs::write(path, bytes).unwrap();
        assert!(prepared.verify_inputs().is_err());
        std::fs::remove_file(path).unwrap();
        std::fs::rename(held, path).unwrap();
    }
    let prepared = s4_control_prepare(&f);
    #[cfg(unix)]
    {
        let original_dir = f.evidence.parent().unwrap();
        let held = original_dir.with_file_name("s4-held-directory");
        std::fs::rename(original_dir, &held).unwrap();
        std::fs::create_dir(original_dir).unwrap();
        std::fs::write(&f.evidence, b"PRIVATE S4 EVIDENCE CONTENT\n").unwrap();
        assert!(prepared.verify_inputs().is_err());
        std::fs::remove_file(&f.evidence).unwrap();
        std::fs::remove_dir(original_dir).unwrap();
        std::fs::rename(held, original_dir).unwrap();
    }
    prepared.verify_inputs().unwrap();
    f.linkage["evidence"][0]["location"]["path"] = json!("not-present/proof.bin");
    f.plan["items"][0]["history"][2]["closure"]["evidence"][0]["href"] =
        json!("evidence/not-present/proof.bin");
    f.plan["items"][0]["milestones"][0]["history"][2]["closure"]["evidence"][0]["href"] =
        json!("evidence/not-present/proof.bin");
    s4_control_persist(&mut f);
    let absent = s4_control_prepare(&f);
    let report = s4_control_report(&absent);
    assert_eq!(report["summary"]["unavailable"], 2);
    assert_eq!(report["summary"]["binding_unavailable"], 2);
    assert_eq!(report["summary"]["captured_original_generations"], 9);
    assert_eq!(report["summary"]["distinct_captured_local_evidence"], 0);
    std::fs::create_dir(f.root.join("evidence/not-present")).unwrap();
    std::fs::write(f.root.join("evidence/not-present/proof.bin"), b"PRIVATE S4 EVIDENCE CONTENT\n")
        .unwrap();
    assert!(absent.verify_inputs().is_err());
}

/// Real URI references stay unfetched and mismatched; closed/duplicate/extraneous companions and stale source tuples fail wholly.
#[test]
fn s4_actual_uri_and_closed_binding_source_refusals_are_not_closure_authority() {
    let mut f = s4_control_fixture(true, false);
    s4_control_prepare(&f);
    let original_links = f.links.clone();
    for field in ["link_key", "evidence_key"] {
        f.links = original_links.clone();
        f.links["bindings"][0][field] = json!("unresolved");
        s4_control_persist(&mut f);
        let report = s4_control_report(&s4_control_prepare(&f));
        assert_eq!(report["summary"]["binding_unavailable"], 1);
    }
    f.links = original_links.clone();
    f.links["bindings"].as_array_mut().unwrap().remove(0);
    s4_control_persist(&mut f);
    let unbound = s4_control_report(&s4_control_prepare(&f));
    assert_eq!(unbound["summary"]["unbound"], 1);
    assert_eq!(unbound["summary"]["matched"], 1);
    assert_eq!(unbound["summary"]["freshness_unmeasured"], 1);
    f.links = original_links.clone();
    s4_control_persist(&mut f);
    s4_control_prepare(&f);
    let raw = std::fs::read(&f.links_path).unwrap();
    let mut duplicate_json = br#"{"schema_version":"forge.poam-evidence-links/1","#.to_vec();
    duplicate_json.extend_from_slice(&raw[1..]);
    std::fs::write(&f.links_path, duplicate_json).unwrap();
    assert!(
        forge::poam::workflow_evidence::prepare(&f.plan_path, &f.links_path, "2026-01-15").is_err()
    );
    s4_control_invalid(&s4_control_cli(&f, &f.root, None), &f);
    for mutation in 0..4 {
        f.links = original_links.clone();
        match mutation {
            0 => {
                let duplicate = f.links["bindings"][0].clone();
                f.links["bindings"].as_array_mut().unwrap().push(duplicate);
            }
            1 => f.links["bindings"][0]["locator"]["event_key"] = json!("extraneous"),
            2 => {
                f.links["bindings"][0]["locator"]["private_unknown"] =
                    json!("SENSITIVE MALFORMED FIELD");
            }
            _ => {
                f.links["bindings"][0]["locator"].as_object_mut().unwrap().remove("milestone_key");
            }
        }
        s4_control_persist(&mut f);
        let refused =
            forge::poam::workflow_evidence::prepare(&f.plan_path, &f.links_path, "2026-01-15")
                .unwrap_err();
        assert!(!refused.to_string().contains("SENSITIVE"));
        s4_control_invalid(&s4_control_cli(&f, &f.root, None), &f);
    }
    f.links = original_links;
    s4_control_persist(&mut f);
    s4_control_prepare(&f);
    let original_plan = f.plan.clone();
    f.plan["items"][0]["source_refs"][0]["expected_sha256"] = json!("0".repeat(64));
    s4_control_persist(&mut f);
    let refused =
        forge::poam::workflow_evidence::prepare(&f.plan_path, &f.links_path, "2026-01-15")
            .unwrap_err();
    assert!(refused.to_string().contains("current selected source object tuple differs"));
    f.plan = original_plan;
    f.linkage["evidence"][0]["location"] = json!({"kind":"uri","uri":"https://127.0.0.1:1/PRIVATE-S4-NO-FETCH","unverified":true,"expected_sha256":common::sha256_file(&f.evidence)});
    s4_control_persist(&mut f);
    let prepared = s4_control_prepare(&f);
    let report = s4_control_report(&prepared);
    assert_eq!(report["summary"]["unverified_uri"], 2);
    assert_eq!(report["summary"]["mismatched"], 2);
    assert_eq!(report["rows"][0]["match_status"], "reference-kind-mismatch");
    assert!(
        report["rows"][0]["approved_sha256"].is_null()
            && report["rows"][0]["observed_sha256"].is_null()
    );
    assert_eq!(report["rows"][0]["local_bytes_revalidated"], false);
    assert!(!String::from_utf8(prepared.report().to_vec()).unwrap().contains("127.0.0.1"));
}

/// Nested plan/companion/linkage bases win over cwd/companion-local decoys and keep every captured generation explicit.
#[test]
fn s4_actual_nested_plan_companion_linkage_and_cwd_decoys_resolve_exactly() {
    let f = s4_control_fixture(true, true);
    let prepared = s4_control_prepare(&f);
    let report = s4_control_report(&prepared);
    assert_eq!(report["summary"]["captured_original_generations"], 11);
    let decoy = tempfile::tempdir().unwrap();
    std::fs::create_dir(decoy.path().join("companions")).unwrap();
    std::fs::write(decoy.path().join("companions/links.json"), b"PRIVATE INVALID CWD DECOY")
        .unwrap();
    std::fs::write(
        f.root.join("companions/linkage.json"),
        b"PRIVATE INVALID COMPANION-LOCAL DECOY",
    )
    .unwrap();
    std::fs::write(f.root.join("linkage.json"), b"PRIVATE INVALID PLAN-LOCAL DECOY").unwrap();
    let output = s4_control_cli(&f, decoy.path(), None);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, prepared.report());
    prepared.verify_inputs().unwrap();
}

/// All original target attempts preserve bytes; portable-name/existence preemption and platform publication are qualified explicitly.
#[test]
fn s4_actual_cli_outputs_preflight_originals_and_preserve_portable_publication() {
    let f = s4_control_fixture(true, false);
    let prepared = s4_control_prepare(&f);
    prepared.verify_inputs().unwrap();
    let originals = s4_control_originals(&f);
    for (path, bytes) in &originals {
        let relative = path.strip_prefix(&f.root).unwrap().to_str().unwrap();
        s4_control_invalid(&s4_control_cli(&f, &f.root, Some(relative)), &f);
        assert_eq!(std::fs::read(path).unwrap(), *bytes);
        // Nested paths/suffixes and already-owned destinations may preempt the
        // held-input alias guard. This proves no overwrite, not all guard arms.
    }
    for name in [
        "CATALOG.json",
        "ASSESSMENT-RESULTS.json",
        "POAM-S4.json",
        "../outside.json",
        "evidence/not-present.json",
    ] {
        s4_control_invalid(&s4_control_cli(&f, &f.root, Some(name)), &f);
    }
    let owned = f.root.join("owned-inspection.json");
    std::fs::write(&owned, b"OWNED INSPECTION SENTINEL").unwrap();
    s4_control_invalid(&s4_control_cli(&f, &f.root, Some("owned-inspection.json")), &f);
    assert_eq!(std::fs::read(owned).unwrap(), b"OWNED INSPECTION SENTINEL");
    // Same basename as the nested companion, a different actual full path.
    let result = s4_control_cli(&f, &f.root, Some("links.json"));
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        assert_eq!(result.status.code(), Some(1));
        assert_eq!(result.stdout, [] as [u8; 0]);
        assert_eq!(std::fs::read(f.root.join("links.json")).unwrap(), prepared.report());
        s4_control_invalid(&s4_control_cli(&f, &f.root, Some("links.json")), &f);
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        s4_control_invalid(&result, &f);
        assert!(!f.root.join("links.json").exists());
    }
    for (path, bytes) in originals {
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
    // Missing native/source or malformed declarations are errors, never action reports.
    std::fs::write(&f.plan_path, b"PRIVATE INVALID PLAN").unwrap();
    s4_control_invalid(&s4_control_cli(&f, &f.root, None), &f);
}
