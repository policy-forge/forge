//! Genuine maintained-command epoch append controls; authored source proposals, not execution evidence.
//!
//! Complete caller continuity is descriptive. These controls exercise actual native build/init and
//! original source qualification, and keep D064, remote authority and platform acceptance separate.

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

/// Exact family key deliberately exercises JSON, text and HTML display escaping.
const RECURRING_FAMILY: &str = "family<&\"a";

/// Owned fixture root keeps every actual native original alive throughout a public command control.
struct Fixture {
    /// Private temp owner; no project or external fixture directories are modified.
    _directory: TempDir,
    /// Canonical owned root avoids the known macOS `/var` symlink alias before confinement.
    root: PathBuf,
    /// Complete closed request copied from actual native source qualification.
    request: Value,
}

/// Execute the actual maintained executable with a fixed cwd and no shell or secret arguments.
fn run(root: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_forge")).current_dir(root).args(arguments).output().unwrap()
}

/// Supply every optional provenance field explicitly within the requested sealed epoch window.
fn provenance(start: &str) -> Value {
    json!({"assessor_key":"synthetic-assessor","role_id":"assessor","start":start,
        "end":null,"method":"EXAMINE","rationale":"SENSITIVE HUMAN ASSERTION"})
}

/// Produce a complete risk row, retaining explicit null optional fields and caller status.
fn risk(key: &str, status: &str, assertion: &Value) -> Value {
    json!({"key":key,"title":"SENSITIVE RISK TITLE","description":"SENSITIVE RISK DESCRIPTION",
        "statement":"SENSITIVE RISK STATEMENT","status":status,"severity":null,"confidence":null,
        "provenance":assertion})
}

/// Construct a complete supported six-field first AR declaration with three actual risks.
fn first_declaration(context: &Value, subject: &Value) -> Value {
    let assertion = provenance("2026-01-01T01:00:00Z");
    let mut relationships = vec![
        json!({"from":{"type":"observation","key":"synthetic-observation"},
            "to":{"type":"finding","key":"finding-satisfied"}}),
        json!({"from":{"type":"observation","key":"synthetic-observation"},
            "to":{"type":"finding","key":"finding-unsatisfied"}}),
    ];
    for key in ["risk-closed", "risk-open", "risk-unselected"] {
        relationships.push(json!({"from":{"type":"finding","key":"finding-unsatisfied"},
            "to":{"type":"risk","key":key}}));
    }
    json!({"schema_version":"forge.assessment-results/1",
        "document":{"key":"synthetic-assessment","title":"Synthetic result source",
            "version":"1.0.0","last_modified":"2026-01-02T00:00:00Z"},
        "context":context,"roles":[{"id":"assessor","title":"Synthetic assessor"}],
        "parties":[{"key":"synthetic-assessor","type":"person","name":"SENSITIVE ACTOR NAME"}],
        "result":{"key":"synthetic-result","title":"Synthetic epoch",
            "description":"SENSITIVE EPOCH DESCRIPTION","start":"2026-01-01T00:00:00Z",
            "end":"2026-01-02T00:00:00Z","control_ids":["AC-1"],"objective_ids":["AC-1_obj"],
            "observations":[{"key":"synthetic-observation","title":null,
                "description":"SENSITIVE OBSERVATION","provenance":assertion,
                "subjects":[{"type":"component","uuid":subject}],"task_uuids":[],"evidence_keys":[]}],
            "findings":[
                {"key":"finding-satisfied","title":"SENSITIVE FINDING TITLE",
                    "description":"SENSITIVE FINDING DESCRIPTION","provenance":assertion,
                    "target":{"type":"statement-id","id":"AC-1_smt","state":"satisfied","reason":"pass"},
                    "implementation_statement_uuid":null},
                {"key":"finding-unsatisfied","title":"Synthetic unmet","description":"Synthetic unmet description",
                    "provenance":assertion,"target":{"type":"objective-id","id":"AC-1_obj",
                        "state":"not-satisfied","reason":"fail"},"implementation_statement_uuid":null}],
            "risks":[risk("risk-closed","closed",&assertion),risk("risk-open","open",&assertion),
                risk("risk-unselected","open",&assertion)],"relationships":relationships}})
}

/// Build a distinct later sealed epoch with one explicit recurrence and one new risk family.
fn next_declaration(first: &Value, subject: &Value, third: bool) -> Value {
    let (result, observation, finding, recurring, new, day, version) = if third {
        (
            "epoch-third",
            "observation-third",
            "finding-third",
            "risk-recurring-again",
            "risk-third",
            "2026-01-05",
            "3.0.0",
        )
    } else {
        (
            "epoch-next",
            "observation-next",
            "finding-next",
            "risk-recurring",
            "risk-new",
            "2026-01-03",
            "2.0.0",
        )
    };
    let end = if third { "2026-01-06T00:00:00Z" } else { "2026-01-04T00:00:00Z" };
    let assertion = provenance(&format!("{day}T01:00:00Z"));
    let mut value = first.clone();
    value["document"]["version"] = json!(version);
    value["document"]["last_modified"] = json!(end);
    value["result"] = json!({"key":result,"title":"Later caller epoch",
        "description":"SENSITIVE LATER EPOCH","start":format!("{day}T00:00:00Z"),"end":end,
        "control_ids":["AC-1"],"objective_ids":["AC-1_obj"],
        "observations":[{"key":observation,"title":null,"description":"SENSITIVE LATER OBSERVATION",
            "provenance":assertion,"subjects":[{"type":"component","uuid":subject}],
            "task_uuids":[],"evidence_keys":[]}],
        "findings":[{"key":finding,"title":"Later finding","description":"SENSITIVE LATER FINDING",
            "provenance":assertion,"target":{"type":"objective-id","id":"AC-1_obj",
                "state":"not-satisfied","reason":"fail"},"implementation_statement_uuid":null}],
        "risks":[risk(recurring,"investigating",&assertion),risk(new,"open",&assertion)],
        "relationships":[{"from":{"type":"observation","key":observation},
            "to":{"type":"finding","key":finding}},
            {"from":{"type":"finding","key":finding},"to":{"type":"risk","key":recurring}},
            {"from":{"type":"finding","key":finding},"to":{"type":"risk","key":new}}]});
    value
}

/// Produce a genuine empty scaffold using the actual persisted native file and selected result identity.
fn initialize(root: &Path, native_name: &str, result: &Value, plan_name: &str) -> Value {
    let output = run(
        root,
        &[
            "poam",
            "init",
            "--assessment-results",
            native_name,
            "--assessment-plan",
            "assessment-plan.json",
            "--ssp",
            "ssp.json",
            "--profile",
            "profile.json",
            "--catalog",
            "catalog.json",
            "--result-uuid",
            result["uuid"].as_str().unwrap(),
            "--result-key",
            native_property(result, "stable-key"),
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
    assert_eq!(output.status.code(), Some(0), "actual init fixture failed");
    assert_eq!(output.stderr, [] as [u8; 0]);
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    std::fs::write(root.join(plan_name), &output.stdout).unwrap();
    value
}

/// Read an exact modeled property; absence is a fixture failure rather than a guessed identity.
fn native_property<'a>(value: &'a Value, name: &str) -> &'a str {
    value["props"].as_array().unwrap().iter()
        .find(|property| property["name"] == name).unwrap()["value"].as_str().unwrap()
}

/// Copy risk tuples only after the maintained loader qualifies all actual five native originals.
fn risk_references(root: &Path, plan_name: &str, scaffold: &Value) -> Vec<Value> {
    let parsed: manifest::PoamManifest = serde_json::from_value(scaffold.clone()).unwrap();
    let actual = source::load(&root.join(plan_name), &parsed.source).unwrap();
    actual.verify_inputs().unwrap();
    actual
        .inventory()
        .objects
        .iter()
        .filter(|object| object.kind.as_str() == "risk")
        .map(|object| {
            json!({"kind":"risk","key":object.key,"uuid":object.uuid,
            "result_uuid":object.result_uuid,"expected_sha256":object.sha256})
        })
        .collect()
}

/// Find one actual selected tuple by stable key without title, position or synthetic UUID inference.
fn reference<'a>(references: &'a [Value], key: &str) -> &'a Value {
    references.iter().find(|reference| reference["key"] == key).unwrap()
}

/// Author every seed and next classification explicitly from the actual original risk inventory.
fn append_request(scaffold: &Value, references: &[Value], next: &Value) -> Value {
    assert_eq!(references.len(), 3);
    let seeds: Vec<Value> = references
        .iter()
        .map(|risk| {
            let family = if risk["key"] == "risk-open" {
                RECURRING_FAMILY.to_string()
            } else {
                format!("family-{}", risk["key"].as_str().unwrap())
            };
            json!({"family_key":family,"risk":risk,"caller_asserted_continuity":true,
            "provenance":provenance("2026-01-01T01:00:00Z")})
        })
        .collect();
    json!({"schema_version":"forge.assessment-epoch-append/1","prior_source":scaffold["source"],
        "next_epoch":next,"prior_report":null,"seed_families":seeds,
        "new_risks":[{"next_key":"risk-recurring","family_key":RECURRING_FAMILY,
            "prior":reference(references,"risk-open"),"caller_asserted_continuity":true,
            "provenance":provenance("2026-01-03T01:00:00Z")},
            {"next_key":"risk-new","family_key":"family-new","prior":null,
                "caller_asserted_continuity":true,"provenance":provenance("2026-01-03T01:00:00Z")}]})
}

/// Check the complete actual native Value with the pinned official AR schema; public `forge validate` does not admit AR.
fn assert_assessment_schema(native: &Value) {
    let schema: Value =
        serde_json::from_str(include_str!("../schemas/oscal_assessment-results_schema.json"))
            .unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    assert!(validator.is_valid(native));
}

/// Build and officially validate native originals before any public epoch append assertion.
fn fixture() -> Fixture {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let (mut context, subject) = native_companions(&root);
    context["evidence_index"] = Value::Null;
    let declaration = first_declaration(&context, &subject);
    write_json(&root.join("ar-manifest.json"), &declaration);
    let built = run(&root, &["assessment", "results", "build", "--manifest", "ar-manifest.json"]);
    assert_eq!(built.status.code(), Some(0), "actual AR builder fixture failed");
    assert_eq!(built.stderr, [] as [u8; 0]);
    let native: Value = serde_json::from_slice(&built.stdout).unwrap();
    assert_assessment_schema(&native);
    std::fs::write(root.join("assessment-results.json"), &built.stdout).unwrap();
    let scaffold = initialize(
        &root,
        "assessment-results.json",
        &native["assessment-results"]["results"][0],
        "empty-plan.json",
    );
    let references = risk_references(&root, "empty-plan.json", &scaffold);
    let request =
        append_request(&scaffold, &references, &next_declaration(&declaration, &subject, false));
    write_json(&root.join("append.json"), &request);
    Fixture { _directory: directory, root, request }
}

/// Invoke the actual public two-file command; no producer, proof or response stand-in is used.
fn append(root: &Path, request: &str, output: &str, report: &str, extra: &[&str]) -> Output {
    let mut arguments = vec![
        "assessment",
        "results",
        "append-epoch",
        "--request",
        request,
        "--output",
        output,
        "--report",
        report,
    ];
    arguments.extend_from_slice(extra);
    run(root, &arguments)
}

/// Preserve complete input bytes so every refusal control also checks nonmutation.
fn originals(fixture: &Fixture) -> BTreeMap<String, Vec<u8>> {
    [
        "catalog.json",
        "profile.json",
        "ssp.json",
        "assessment-plan.json",
        "assessment-results.json",
        "ar-manifest.json",
        "empty-plan.json",
        "append.json",
    ]
    .into_iter()
    .map(|name| (name.to_string(), std::fs::read(fixture.root.join(name)).unwrap()))
    .collect()
}

/// Check all retained original bytes after the command, without claiming captured identity timing.
fn assert_originals(fixture: &Fixture, before: &BTreeMap<String, Vec<u8>>) {
    for (name, bytes) in before {
        assert_eq!(&std::fs::read(fixture.root.join(name)).unwrap(), bytes);
    }
}

/// On unsupported publishers, actual qualified inputs still yield failure without either destination or stdout.
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
#[test]
fn unsupported_publication_never_emits_an_epoch_pair() {
    let fixture = fixture();
    let before = originals(&fixture);
    let output = append(
        &fixture.root,
        "append.json",
        "next.json",
        "next-report.json",
        &["--fail-on", "never"],
    );
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stdout, [] as [u8; 0]);
    assert!(!fixture.root.join("next.json").exists());
    assert!(!fixture.root.join("next-report.json").exists());
    assert_originals(&fixture, &before);
}

/// Real no-replace publication controls are bounded to the currently qualified Linux/macOS publisher.
#[cfg(any(target_os = "linux", target_os = "macos"))]
mod publication {
    use super::*;

    /// Persisted pair read from the actual command, never a detached constructed proof.
    struct Pair {
        /// Complete original native bytes produced by one successful explicit append.
        bytes: Vec<u8>,
        /// Parsed complete native representation for preservation and downstream consumer assertions.
        native: Value,
        /// Complete closed report decoded from its separately published original bytes.
        report: Value,
    }

    /// Decode both durable files after valid delivery and verify official native/schema admission.
    fn pair(root: &Path, native_name: &str, report_name: &str) -> Pair {
        let bytes = std::fs::read(root.join(native_name)).unwrap();
        let native: Value = serde_json::from_slice(&bytes).unwrap();
        assert_assessment_schema(&native);
        let report: Value =
            serde_json::from_slice(&std::fs::read(root.join(report_name)).unwrap()).unwrap();
        let schema: Value = serde_json::from_str(include_str!(
            "../schemas/forge.assessment-epoch-report-1.schema.json"
        ))
        .unwrap();
        let validator = jsonschema::validator_for(&schema).unwrap();
        assert!(validator.is_valid(&report));
        assert_eq!(report["native"]["artifact"], native_name);
        assert_eq!(report["native"]["raw_sha256"], sha256_hex(&bytes));
        Pair { bytes, native, report }
    }

    /// Exercise one actual valid append before the following negative request controls can earn credit.
    fn canary() -> Fixture {
        let fixture = fixture();
        let before = originals(&fixture);
        let output =
            append(&fixture.root, "append.json", "canary-native.json", "canary-report.json", &[]);
        assert_eq!(
            output.status.code(),
            Some(1),
            "valid append must reach review-required delivery"
        );
        assert_eq!(output.stdout, [] as [u8; 0]);
        assert_eq!(output.stderr, [] as [u8; 0]);
        let actual = pair(&fixture.root, "canary-native.json", "canary-report.json");
        assert_first_report(&fixture, &actual);
        assert_originals(&fixture, &before);
        fixture
    }

    /// Count every recursive array element in complete result graphs, preserving repeated occurrences.
    fn graph_occurrences(value: &Value) -> u64 {
        match value {
            Value::Array(values) => {
                values.iter().map(graph_occurrences).sum::<u64>()
                    + u64::try_from(values.len()).unwrap()
            }
            Value::Object(values) => values.values().map(graph_occurrences).sum(),
            _ => 0,
        }
    }

    /// Restore only the two authorized metadata leaves and remove exactly the new result for whole-Value equality.
    fn assert_preserved(old: &Value, new: &Value) {
        let old_root = &old["assessment-results"];
        let new_root = &new["assessment-results"];
        let old_rows = old_root["results"].as_array().unwrap();
        let new_rows = new_root["results"].as_array().unwrap();
        assert_eq!(new_rows.len(), old_rows.len() + 1);
        assert_eq!(&new_rows[..old_rows.len()], old_rows.as_slice());
        let mut restored = new.clone();
        restored["assessment-results"]["results"].as_array_mut().unwrap().pop().unwrap();
        for field in ["version", "last-modified"] {
            restored["assessment-results"]["metadata"][field] = old_root["metadata"][field].clone();
        }
        assert_eq!(&restored, old, "no omitted or unknown original Value may disappear");
    }

    /// Reconcile every epoch and object row with complete actual native results, hashes and declared properties.
    fn assert_rows(native: &Value, report: &Value) {
        let results = native["assessment-results"]["results"].as_array().unwrap();
        let epochs = report["epochs"].as_array().unwrap();
        assert_eq!(epochs.len(), results.len());
        let objects = report["objects"].as_array().unwrap();
        let mut actual_count = 0;
        for (position, (result, epoch)) in results.iter().zip(epochs).enumerate() {
            assert_eq!(epoch["position"], position);
            assert_eq!(epoch["key"], native_property(result, "stable-key"));
            assert_eq!(epoch["uuid"], result["uuid"]);
            assert_eq!(epoch["canonical_sha256"], canonical_digest(result));
            assert_eq!(epoch["start"], result["start"]);
            assert_eq!(epoch["end"], result["end"]);
            assert_eq!(
                epoch["classification"],
                if position + 1 == results.len() { "appended" } else { "preserved" }
            );
            for (kind, field) in
                [("observation", "observations"), ("finding", "findings"), ("risk", "risks")]
            {
                let entries = result[field].as_array().unwrap();
                assert_eq!(epoch[field], entries.len());
                for value in entries {
                    actual_count += 1;
                    let matches: Vec<_> = objects
                        .iter()
                        .filter(|row| {
                            row["kind"] == kind
                                && row["result_uuid"] == result["uuid"]
                                && row["uuid"] == value["uuid"]
                        })
                        .collect();
                    assert_eq!(matches.len(), 1);
                    let row = matches[0];
                    assert_eq!(row["result_key"], native_property(result, "stable-key"));
                    assert_eq!(row["key"], native_property(value, "stable-key"));
                    assert_eq!(row["computed_sha256"], canonical_digest(value));
                    assert_eq!(
                        row["declared_content_sha256"],
                        native_property(value, "content-sha256")
                    );
                    assert_eq!(
                        row["declared_rationale_sha256"],
                        native_property(value, "rationale-sha256")
                    );
                    assert_eq!(
                        row["classification"],
                        if position + 1 == results.len() { "added" } else { "preserved" }
                    );
                    let status = if kind == "risk" {
                        value.get("status")
                    } else if kind == "finding" {
                        value.pointer("/target/status/state")
                    } else {
                        None
                    };
                    assert_eq!(row["status"], status.cloned().unwrap_or(Value::Null));
                }
            }
        }
        assert_eq!(objects.len(), actual_count);
    }

    /// Resolve every minimized family/edge endpoint to exactly one already reconciled actual native risk row.
    fn assert_report_reference(report: &Value, reference: &Value) {
        assert_eq!(reference["kind"], "risk");
        let matches: Vec<_> = report["objects"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| {
                row["kind"] == "risk"
                    && row["result_uuid"] == reference["result_uuid"]
                    && row["uuid"] == reference["uuid"]
                    && row["key"] == reference["key"]
                    && row["computed_sha256"] == reference["expected_sha256"]
            })
            .collect();
        assert_eq!(matches.len(), 1);
    }

    /// Check two complete reciprocal rows per edge and exact current family endpoints without inferred continuity.
    fn assert_reciprocals(report: &Value) {
        let rows = report["reciprocal_rows"].as_array().unwrap();
        let edges = report["continuity_edges"].as_array().unwrap();
        assert_eq!(rows.len(), 2 * edges.len());
        for family in report["families"].as_array().unwrap() {
            assert_report_reference(report, &family["first"]);
            assert_report_reference(report, &family["current"]);
        }
        for edge in edges {
            assert_report_reference(report, &edge["predecessor"]);
            assert_report_reference(report, &edge["successor"]);
            let selected: Vec<_> =
                rows.iter().filter(|row| row["edge_sha256"] == edge["edge_sha256"]).collect();
            assert_eq!(selected.len(), 2);
            for (direction, from, to) in [
                ("predecessor", "predecessor", "successor"),
                ("successor", "successor", "predecessor"),
            ] {
                let row = selected.iter().find(|row| row["direction"] == direction).unwrap();
                assert_eq!(row["family_key"], edge["family_key"]);
                assert_eq!(row["from"], edge[from]);
                assert_eq!(row["to"], edge[to]);
            }
            assert_eq!(edge["caller_asserted_continuity"], true);
            let family = report["families"]
                .as_array()
                .unwrap()
                .iter()
                .find(|family| family["family_key"] == edge["family_key"])
                .unwrap();
            let current = family["current"]["result_uuid"].as_str().unwrap();
            if current == edge["successor"]["result_uuid"].as_str().unwrap() {
                assert_eq!(family["current"], edge["successor"]);
            }
        }
    }

    /// Check every first-report context identity against its genuine declared companion pin.
    fn assert_report_context(fixture: &Fixture, report: &Value) {
        let context = report["context"].as_array().unwrap();
        assert_eq!(context.len(), 4);
        for (kind, key) in [
            ("assessment-plan", "assessment_plan"),
            ("system-security-plan", "ssp"),
            ("profile", "profile"),
            ("catalog", "catalog"),
        ] {
            let row = context.iter().find(|row| row["kind"] == kind).unwrap();
            let original = &fixture.request["prior_source"]["context"][key];
            assert_eq!(row["sha256"], original["expected_sha256"]);
            for field in ["root_uuid", "document_version", "oscal_version"] {
                assert_eq!(row[field], original[field]);
            }
        }
    }

    /// Assert complete first-append counts, original preservation, descriptive privacy and row joins.
    fn assert_first_report(fixture: &Fixture, actual: &Pair) {
        let old: Value = serde_json::from_slice(
            &std::fs::read(fixture.root.join("assessment-results.json")).unwrap(),
        )
        .unwrap();
        assert_preserved(&old, &actual.native);
        let report = &actual.report;
        assert_eq!(report["assessment_authority"], false);
        assert_eq!(report["continuity_authority"], "caller-asserted");
        assert_eq!(report["review_required"], true);
        assert_eq!(report["status"], "review-required");
        assert_eq!(report["prior_native"]["uuid"], old["assessment-results"]["uuid"]);
        for (wire, native) in
            [("uuid", "uuid"), ("document_version", "version"), ("oscal_version", "oscal-version")]
        {
            let expected = if wire == "uuid" {
                &actual.native["assessment-results"][native]
            } else {
                &actual.native["assessment-results"]["metadata"][native]
            };
            assert_eq!(&report["native"][wire], expected);
        }
        assert_report_context(fixture, report);
        assert_eq!(
            report["prior_native"]["raw_sha256"],
            fixture.request["prior_source"]["assessment_results"]["expected_sha256"]
        );
        for (name, value) in [
            ("before_epochs", 1),
            ("after_epochs", 2),
            ("preserved_epochs", 1),
            ("appended_epochs", 1),
            ("before_observations", 1),
            ("after_observations", 2),
            ("before_findings", 2),
            ("after_findings", 3),
            ("before_risks", 3),
            ("after_risks", 5),
            ("before_objects", 6),
            ("after_objects", 10),
            ("preserved_objects", 6),
            ("added_objects", 4),
            ("families", 4),
            ("continuity_edges", 1),
            ("reciprocal_rows", 2),
            ("classified_next_risks", 2),
            ("uncontinued_families", 2),
            ("captured_original_generations", 6),
        ] {
            assert_eq!(report["counts"][name], value, "complete count {name}");
        }
        assert_eq!(
            report["counts"]["native_graph_occurrences_before"],
            graph_occurrences(&old["assessment-results"]["results"])
        );
        assert_eq!(
            report["counts"]["native_graph_occurrences_after"],
            graph_occurrences(&actual.native["assessment-results"]["results"])
        );
        let changes = report["changes"].as_array().unwrap();
        assert_eq!(report["counts"]["review_actions"], changes.len());
        assert_eq!(changes.iter().filter(|row| row["code"] == "risk-family-created").count(), 1);
        let new_family = report["families"]
            .as_array()
            .unwrap()
            .iter()
            .find(|family| family["family_key"] == "family-new")
            .unwrap();
        assert_eq!(new_family["continued_in_append"], false);
        assert_eq!(new_family["first"], new_family["current"]);
        assert_eq!(new_family["current"]["key"], "risk-new");
        for seed in fixture.request["seed_families"].as_array().unwrap() {
            let family = report["families"]
                .as_array()
                .unwrap()
                .iter()
                .find(|family| family["family_key"] == seed["family_key"])
                .unwrap();
            assert_eq!(family["first"], seed["risk"]);
            if seed["family_key"] == RECURRING_FAMILY {
                assert_eq!(family["current"]["key"], "risk-recurring");
                assert_eq!(family["continued_in_append"], true);
            } else {
                assert_eq!(family["current"], seed["risk"]);
                assert_eq!(family["continued_in_append"], false);
            }
        }
        assert_rows(&actual.native, report);
        assert_reciprocals(report);
        let encoded = serde_json::to_string(report).unwrap();
        for sensitive in ["SENSITIVE", "Synthetic result source", "Synthetic caller plan"] {
            assert!(!encoded.contains(sensitive));
        }
        assert!(!encoded.contains(fixture.root.to_str().unwrap()));
    }

    /// Assert a deterministic request refusal before either output while preserving every input byte.
    fn refuse_request(fixture: &Fixture, request: &Value) {
        write_json(&fixture.root.join("append.json"), request);
        let before = originals(fixture);
        let output = append(
            &fixture.root,
            "append.json",
            "refused-native.json",
            "refused-report.json",
            &["--fail-on", "never"],
        );
        assert_eq!(output.status.code(), Some(2), "invalid input is never policy exit zero or one");
        assert_eq!(output.stdout, [] as [u8; 0]);
        assert!(!fixture.root.join("refused-native.json").exists());
        assert!(!fixture.root.join("refused-report.json").exists());
        assert_originals(fixture, &before);
    }

    /// A genuine two-epoch append preserves whole old values and emits complete descriptive continuity.
    #[test]
    fn actual_append_preserves_original_and_all_report_rows() {
        let fixture = canary();
        let old = originals(&fixture);
        let output = append(
            &fixture.root,
            "append.json",
            "epoch-two.json",
            "epoch-two-report.json",
            &["--fail-on", "never"],
        );
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(output.stdout, [] as [u8; 0]);
        assert_eq!(output.stderr, [] as [u8; 0]);
        let actual = pair(&fixture.root, "epoch-two.json", "epoch-two-report.json");
        assert_first_report(&fixture, &actual);
        assert_originals(&fixture, &old);
        let canary = pair(&fixture.root, "canary-native.json", "canary-report.json");
        assert_eq!(
            actual.bytes, canary.bytes,
            "no hidden clock or output-name-derived native generation"
        );
        let mut canary_report = canary.report;
        canary_report["native"]["artifact"] = json!("epoch-two.json");
        assert_eq!(actual.report, canary_report);
    }

    /// Independently encode sorted JSON object keys and original array order for complete record hash assertions.
    fn canonical_bytes(value: &Value, bytes: &mut Vec<u8>) {
        match value {
            Value::Object(map) => {
                bytes.push(b'{');
                let ordered: BTreeMap<_, _> = map.iter().collect();
                for (position, (key, child)) in ordered.into_iter().enumerate() {
                    if position != 0 {
                        bytes.push(b',');
                    }
                    serde_json::to_writer(&mut *bytes, key).unwrap();
                    bytes.push(b':');
                    canonical_bytes(child, bytes);
                }
                bytes.push(b'}');
            }
            Value::Array(values) => {
                bytes.push(b'[');
                for (position, child) in values.iter().enumerate() {
                    if position != 0 {
                        bytes.push(b',');
                    }
                    canonical_bytes(child, bytes);
                }
                bytes.push(b']');
            }
            value => serde_json::to_writer(bytes, value).unwrap(),
        }
    }

    /// Hash complete canonical records, keeping raw file pins distinct from decoded-object digests.
    fn canonical_digest(value: &Value) -> String {
        let mut bytes = Vec::new();
        canonical_bytes(value, &mut bytes);
        sha256_hex(&bytes)
    }

    /// Capture a third request from the actual durable second native file and its exact full prior report bytes.
    fn third_request(fixture: &Fixture, second: &Pair) -> Value {
        let rows = second.native["assessment-results"]["results"].as_array().unwrap();
        let scaffold =
            initialize(&fixture.root, "epoch-two.json", rows.last().unwrap(), "second-plan.json");
        let references = risk_references(&fixture.root, "second-plan.json", &scaffold);
        assert_eq!(references.len(), 2);
        let first: Value =
            serde_json::from_slice(&std::fs::read(fixture.root.join("ar-manifest.json")).unwrap())
                .unwrap();
        let subject = &first["result"]["observations"][0]["subjects"][0]["uuid"];
        json!({"schema_version":"forge.assessment-epoch-append/1","prior_source":scaffold["source"],
            "next_epoch":next_declaration(&first,subject,true),
            "prior_report":{"artifact":"epoch-two-report.json",
                "expected_sha256":sha256_hex(&std::fs::read(fixture.root.join("epoch-two-report.json")).unwrap())},
            "seed_families":[],"new_risks":[
                {"next_key":"risk-recurring-again","family_key":RECURRING_FAMILY,
                    "prior":reference(&references,"risk-recurring"),"caller_asserted_continuity":true,
                    "provenance":provenance("2026-01-05T01:00:00Z")},
                {"next_key":"risk-third","family_key":"family-third","prior":null,
                    "caller_asserted_continuity":true,"provenance":provenance("2026-01-05T01:00:00Z")}]})
    }

    /// Persist the actual second append needed by genuine later-epoch and prior-report controls.
    fn second_pair(fixture: &Fixture) -> Pair {
        let output =
            append(&fixture.root, "append.json", "epoch-two.json", "epoch-two-report.json", &[]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, [] as [u8; 0]);
        assert_eq!(output.stderr, [] as [u8; 0]);
        pair(&fixture.root, "epoch-two.json", "epoch-two-report.json")
    }

    /// A third append consumes the second durable originals, complete prior report and latest family endpoint.
    #[test]
    fn third_append_preserves_all_epochs_and_prior_edges() {
        let fixture = canary();
        let second = second_pair(&fixture);
        let request = third_request(&fixture, &second);
        write_json(&fixture.root.join("append-third.json"), &request);
        let before_second = std::fs::read(fixture.root.join("epoch-two.json")).unwrap();
        let before_report = std::fs::read(fixture.root.join("epoch-two-report.json")).unwrap();
        let output = append(
            &fixture.root,
            "append-third.json",
            "epoch-three.json",
            "epoch-three-report.json",
            &["--view-format", "html", "--fail-on", "never"],
        );
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(output.stderr, [] as [u8; 0]);
        let third = pair(&fixture.root, "epoch-three.json", "epoch-three-report.json");
        assert_preserved(&second.native, &third.native);
        for (name, value) in [
            ("before_epochs", 2),
            ("after_epochs", 3),
            ("preserved_epochs", 2),
            ("before_objects", 10),
            ("after_objects", 14),
            ("added_objects", 4),
            ("families", 5),
            ("continuity_edges", 2),
            ("reciprocal_rows", 4),
            ("classified_next_risks", 2),
            ("uncontinued_families", 3),
            ("captured_original_generations", 7),
        ] {
            assert_eq!(third.report["counts"][name], value);
        }
        assert_eq!(
            third.report["counts"]["review_actions"],
            third.report["changes"].as_array().unwrap().len()
        );
        assert_eq!(
            &third.report["continuity_edges"].as_array().unwrap()[..1],
            second.report["continuity_edges"].as_array().unwrap().as_slice()
        );
        assert_eq!(
            &third.report["reciprocal_rows"].as_array().unwrap()[..2],
            second.report["reciprocal_rows"].as_array().unwrap().as_slice()
        );
        assert_rows(&third.native, &third.report);
        assert_reciprocals(&third.report);
        assert_eq!(std::fs::read(fixture.root.join("epoch-two.json")).unwrap(), before_second);
        assert_eq!(
            std::fs::read(fixture.root.join("epoch-two-report.json")).unwrap(),
            before_report
        );
        let html = String::from_utf8(output.stdout).unwrap();
        assert!(html.contains("epoch-third"));
        assert!(html.contains("family-third"));
        assert!(!html.contains("family<&\"a"));
        assert!(!html.contains("SENSITIVE"));
    }

    /// Decode only the finite inert HTML entities once, without recursively interpreting literal entity-shaped data.
    fn html_body_json(text: &str) -> Value {
        let body =
            text.split_once("<pre>").unwrap().1.strip_suffix("</pre></body></html>\n").unwrap();
        let mut decoded = String::new();
        let mut remaining = body;
        while !remaining.is_empty() {
            if remaining.starts_with('&') {
                let (entity, scalar) = [
                    ("&amp;", '&'),
                    ("&lt;", '<'),
                    ("&gt;", '>'),
                    ("&quot;", '"'),
                    ("&#39;", '\''),
                ]
                .into_iter()
                .find(|(entity, _)| remaining.starts_with(entity))
                .unwrap();
                decoded.push(scalar);
                remaining = &remaining[entity.len()..];
            } else {
                let scalar = remaining.chars().next().unwrap();
                decoded.push(scalar);
                remaining = &remaining[scalar.len_utf8()..];
            }
        }
        serde_json::from_str(&decoded).unwrap()
    }

    /// All selected views are complete: JSON matches durable bytes and text/HTML retain every family and epoch safely.
    #[test]
    fn every_complete_view_is_opt_in_and_preserves_safe_display() {
        let fixture = canary();
        for (index, format) in ["json", "text", "html"].into_iter().enumerate() {
            let native_name = format!("view-{index}.json");
            let report_name = format!("view-{index}-report.json");
            let output = append(
                &fixture.root,
                "append.json",
                &native_name,
                &report_name,
                &["--view-format", format, "--fail-on", "never"],
            );
            assert_eq!(output.status.code(), Some(0));
            assert_eq!(output.stderr, [] as [u8; 0]);
            let actual = pair(&fixture.root, &native_name, &report_name);
            assert_first_report(&fixture, &actual);
            if format == "json" {
                assert_eq!(output.stdout, std::fs::read(fixture.root.join(&report_name)).unwrap());
                assert_eq!(serde_json::from_slice::<Value>(&output.stdout).unwrap(), actual.report);
            } else {
                let text = String::from_utf8(output.stdout).unwrap();
                let decoded: Value = if format == "html" {
                    html_body_json(&text)
                } else {
                    serde_json::from_str(
                        text.strip_prefix("Assessment epoch report\n\n")
                            .unwrap()
                            .strip_suffix('\n')
                            .unwrap(),
                    )
                    .unwrap()
                };
                assert_eq!(
                    decoded, actual.report,
                    "every count, row, null and field survives the complete view"
                );
                assert_eq!(decoded["counts"].as_object().unwrap().len(), 23);
                for value in [
                    "synthetic-result",
                    "epoch-next",
                    "family-new",
                    "family-risk-closed",
                    "family-risk-unselected",
                ] {
                    assert!(text.contains(value), "complete {format} must retain {value}");
                }
                assert!(!text.contains("SENSITIVE"));
                assert!(!text.contains(fixture.root.to_str().unwrap()));
                if format == "html" {
                    assert!(!text.contains(RECURRING_FAMILY));
                    assert!(!text.contains("<script"));
                    assert!(!text.contains("href="));
                    assert!(text.contains("&lt;"));
                    assert!(text.contains("&amp;"));
                } else {
                    assert!(text.contains("family<"));
                }
            }
        }
    }

    /// Current decimal confidence extremes remain actual model values and are accounted before output growth.
    #[test]
    fn decimal_confidence_extremes_reach_valid_native_output() {
        let fixture = canary();
        for (position, confidence) in
            [1e-100_f64, 5e-324_f64, -0.0_f64, 0.0_f64, 1.0_f64].into_iter().enumerate()
        {
            let mut request = fixture.request.clone();
            request["next_epoch"]["result"]["risks"][0]["confidence"] = json!(confidence);
            write_json(&fixture.root.join("confidence.json"), &request);
            let native_name = format!("confidence-{position}.json");
            let report_name = format!("confidence-{position}-report.json");
            let output = append(
                &fixture.root,
                "confidence.json",
                &native_name,
                &report_name,
                &["--fail-on", "never"],
            );
            assert_eq!(output.status.code(), Some(0));
            let actual = pair(&fixture.root, &native_name, &report_name);
            let risks =
                actual.native["assessment-results"]["results"][1]["risks"].as_array().unwrap();
            let row = risks
                .iter()
                .find(|row| native_property(row, "stable-key") == "risk-recurring")
                .unwrap();
            assert_eq!(
                native_property(row, "reviewer-declared-confidence"),
                confidence.to_string()
            );
            if position < 2 {
                assert!(native_property(row, "reviewer-declared-confidence").len() > 32);
            }
        }
    }

    /// Closed admission refuses unknown, omitted, duplicate and BOM request shapes before publication.
    #[test]
    fn strict_request_shapes_are_not_lossily_normalized() {
        let fixture = canary();
        let mut unknown = fixture.request.clone();
        unknown["unexpected"] = json!(true);
        refuse_request(&fixture, &unknown);
        for pointer in [
            "/next_epoch/context/evidence_index",
            "/next_epoch/result/observations/0/title",
            "/next_epoch/result/observations/0/task_uuids",
            "/next_epoch/result/risks/0/severity",
            "/next_epoch/result/risks/0/confidence",
            "/next_epoch/result/findings/0/implementation_statement_uuid",
            "/new_risks/0/provenance/end",
        ] {
            let mut request = fixture.request.clone();
            let (parent, leaf) = pointer.rsplit_once('/').unwrap();
            request.pointer_mut(parent).unwrap().as_object_mut().unwrap().remove(leaf);
            refuse_request(&fixture, &request);
        }
        let raw = serde_json::to_vec(&fixture.request).unwrap();
        let mut duplicate = b"{\"schema_version\":\"forge.assessment-epoch-append/1\",".to_vec();
        duplicate.extend_from_slice(&raw[1..]);
        let mut bom = vec![0xef, 0xbb, 0xbf];
        bom.extend_from_slice(&raw);
        for bytes in [duplicate, bom] {
            std::fs::write(fixture.root.join("append.json"), bytes).unwrap();
            let output = append(
                &fixture.root,
                "append.json",
                "raw-invalid.json",
                "raw-invalid-report.json",
                &[],
            );
            assert_eq!(output.status.code(), Some(2));
            assert_eq!(output.stdout, [] as [u8; 0]);
            assert!(!fixture.root.join("raw-invalid.json").exists());
            assert!(!fixture.root.join("raw-invalid-report.json").exists());
        }
    }

    /// Every first-profile risk and every next risk is classified exactly once with exact qualified source tuples.
    #[test]
    fn missing_duplicate_and_mismatched_classifiers_are_refused() {
        let fixture = canary();
        for field in ["seed_families", "new_risks"] {
            let mut omitted = fixture.request.clone();
            omitted[field].as_array_mut().unwrap().pop().unwrap();
            refuse_request(&fixture, &omitted);
            let mut duplicate = fixture.request.clone();
            let first = duplicate[field][0].clone();
            duplicate[field].as_array_mut().unwrap().push(first);
            refuse_request(&fixture, &duplicate);
        }
        for pointer in
            ["/seed_families/0/risk/expected_sha256", "/new_risks/0/prior/expected_sha256"]
        {
            let mut request = fixture.request.clone();
            *request.pointer_mut(pointer).unwrap() = json!("0".repeat(64));
            refuse_request(&fixture, &request);
        }
        let mut unknown = fixture.request.clone();
        unknown["new_risks"][0]["next_key"] = json!("not-an-actual-next-risk");
        refuse_request(&fixture, &unknown);
        let mut unasserted = fixture.request.clone();
        unasserted["new_risks"][0]["caller_asserted_continuity"] = json!(false);
        refuse_request(&fixture, &unasserted);
        let mut wrong_family = fixture.request.clone();
        wrong_family["new_risks"][0]["family_key"] = json!("family-risk-closed");
        refuse_request(&fixture, &wrong_family);
    }

    /// Context, evidence-free scope, chronology and real actor/provenance admission cannot be bypassed by policy.
    #[test]
    fn source_context_evidence_chronology_and_actor_guards_hold() {
        let fixture = canary();
        for (pointer, value) in [
            ("/prior_source/assessment_results/expected_sha256", json!("0".repeat(64))),
            (
                "/next_epoch/context/catalog/root_uuid",
                json!("99999999-9999-4999-8999-999999999999"),
            ),
            (
                "/next_epoch/context/evidence_index",
                json!({"artifact":"evidence.json","expected_sha256":"0".repeat(64)}),
            ),
            ("/next_epoch/result/observations/0/evidence_keys", json!(["hidden-evidence"])),
            ("/next_epoch/result/end", Value::Null),
            ("/next_epoch/result/start", json!("2026-01-01T00:00:00Z")),
            ("/next_epoch/result/key", json!("synthetic-result")),
            ("/next_epoch/result/risks/0/key", json!("risk-open")),
            ("/next_epoch/roles/0/id", json!("different-assessor")),
            ("/next_epoch/parties/0/name", json!("different-declared-party")),
            ("/new_risks/0/provenance/assessor_key", json!("undeclared")),
            ("/new_risks/0/provenance/start", json!("2027-01-01T01:00:00Z")),
        ] {
            let mut request = fixture.request.clone();
            *request.pointer_mut(pointer).unwrap() = value;
            refuse_request(&fixture, &request);
        }
    }

    /// Complete prior-report validation refuses rehashed semantic tampering rather than only stale byte pins.
    #[test]
    fn durable_prior_report_tamper_is_not_reaccepted_by_rehashing() {
        let fixture = canary();
        let second = second_pair(&fixture);
        let request = third_request(&fixture, &second);
        write_json(&fixture.root.join("third-canary.json"), &request);
        let canary = append(
            &fixture.root,
            "third-canary.json",
            "third-canary-native.json",
            "third-canary-report.json",
            &[],
        );
        assert_eq!(canary.status.code(), Some(1));
        pair(&fixture.root, "third-canary-native.json", "third-canary-report.json");
        for kind in ["epoch-digest", "reciprocal", "native-identity", "family-tuple"] {
            let mut previous = second.report.clone();
            match kind {
                "epoch-digest" => previous["epochs"][0]["canonical_sha256"] = json!("0".repeat(64)),
                "reciprocal" => {
                    previous["reciprocal_rows"].as_array_mut().unwrap().pop().unwrap();
                    previous["counts"]["reciprocal_rows"] = json!(1);
                }
                "native-identity" => {
                    previous["native"]["uuid"] = json!("99999999-9999-4999-8999-999999999999");
                }
                "family-tuple" => {
                    previous["families"][0]["current"]["expected_sha256"] = json!("0".repeat(64));
                }
                _ => unreachable!(),
            }
            let bytes = write_json(&fixture.root.join("epoch-two-report.json"), &previous);
            let mut tampered = request.clone();
            tampered["prior_report"]["expected_sha256"] = json!(sha256_hex(&bytes));
            refuse_request(&fixture, &tampered);
        }
    }

    /// A recurrence must use the latest qualified family endpoint, never an old nonlatest risk or implicit match.
    #[test]
    fn later_append_refuses_nonlatest_endpoints_and_initial_reseeding() {
        let fixture = canary();
        let second = second_pair(&fixture);
        let request = third_request(&fixture, &second);
        let mut stale = request.clone();
        stale["new_risks"][0]["prior"] = fixture.request["new_risks"][0]["prior"].clone();
        refuse_request(&fixture, &stale);
        let mut reseed = request;
        reseed["seed_families"] = fixture.request["seed_families"].clone();
        refuse_request(&fixture, &reseed);
    }

    /// Original spelling and declared source paths are admitted before path extraction or normalization.
    #[test]
    fn request_and_source_path_alias_spellings_are_refused() {
        let fixture = canary();
        let absolute = fixture.root.join("append.json").to_str().unwrap().to_string();
        for request_name in [
            "./append.json".to_string(),
            "child/../append.json".to_string(),
            format!("{absolute}/"),
            format!("{}/./append.json", fixture.root.display()),
        ] {
            let output =
                append(&fixture.root, &request_name, "alias-native.json", "alias-report.json", &[]);
            assert_eq!(output.status.code(), Some(2));
            assert_eq!(output.stdout, [] as [u8; 0]);
            assert!(!fixture.root.join("alias-native.json").exists());
            assert!(!fixture.root.join("alias-report.json").exists());
        }
        for name in
            ["./assessment-results.json", "child/../assessment-results.json", "/outside.json"]
        {
            let mut request = fixture.request.clone();
            request["prior_source"]["assessment_results"]["artifact"] = json!(name);
            request["prior_source"]["assessment_results"]["href"] = json!(name);
            refuse_request(&fixture, &request);
        }
    }

    /// A hardlinked Catalog cannot substitute for a Profile; this public negative does not isolate the internal identity fence.
    #[test]
    fn hardlinked_wrong_model_refuses_without_any_output() {
        let fixture = canary();
        std::fs::hard_link(
            fixture.root.join("catalog.json"),
            fixture.root.join("catalog-copy.json"),
        )
        .unwrap();
        let mut request = fixture.request.clone();
        request["prior_source"]["context"]["profile"] =
            request["prior_source"]["context"]["catalog"].clone();
        request["prior_source"]["context"]["profile"]["artifact"] = json!("catalog-copy.json");
        request["prior_source"]["context"]["profile"]["href"] = json!("catalog-copy.json");
        request["next_epoch"]["context"] = request["prior_source"]["context"].clone();
        refuse_request(&fixture, &request);
    }

    /// Both output filenames are fully admitted together, including portable names, input aliases and pair collisions.
    #[test]
    fn both_destinations_are_preflighted_before_any_output() {
        let fixture = canary();
        let long = format!("{}.json", "x".repeat(124));
        for report_name in [
            "../bad.json",
            "nested/bad.json",
            "unicode-é.json",
            "colon:name.json",
            "CON.json",
            "wrong.txt",
            long.as_str(),
            "APPEND.json",
            "catalog.json",
            "GOOD.json",
        ] {
            let output = append(
                &fixture.root,
                "append.json",
                "good.json",
                report_name,
                &["--fail-on", "never"],
            );
            assert_eq!(output.status.code(), Some(2));
            assert_eq!(output.stdout, [] as [u8; 0]);
            assert!(!fixture.root.join("good.json").exists());
        }
        let before = originals(&fixture);
        let output = append(&fixture.root, "append.json", "catalog.json", "new-report.json", &[]);
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(output.stdout, [] as [u8; 0]);
        assert!(!fixture.root.join("new-report.json").exists());
        assert_originals(&fixture, &before);
    }

    /// Existing destinations are never replaced and a deterministic invalid second destination leaves no first file.
    #[test]
    fn existing_output_sentinels_remain_exact() {
        let fixture = canary();
        std::fs::write(fixture.root.join("existing-native.json"), b"native sentinel").unwrap();
        let output =
            append(&fixture.root, "append.json", "existing-native.json", "absent-report.json", &[]);
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(output.stdout, [] as [u8; 0]);
        assert_eq!(
            std::fs::read(fixture.root.join("existing-native.json")).unwrap(),
            b"native sentinel"
        );
        assert!(!fixture.root.join("absent-report.json").exists());
        std::fs::write(fixture.root.join("existing-report.json"), b"report sentinel").unwrap();
        let output =
            append(&fixture.root, "append.json", "absent-native.json", "existing-report.json", &[]);
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(output.stdout, [] as [u8; 0]);
        assert_eq!(
            std::fs::read(fixture.root.join("existing-report.json")).unwrap(),
            b"report sentinel"
        );
        assert!(!fixture.root.join("absent-native.json").exists());
    }

    /// Finite views and the mandatory durable report are CLI admission gates even with a missing request file.
    #[test]
    fn invalid_view_and_missing_report_fail_at_argument_admission() {
        let fixture = canary();
        let invalid = run(
            &fixture.root,
            &[
                "assessment",
                "results",
                "append-epoch",
                "--request",
                "absent.json",
                "--output",
                "unused.json",
                "--report",
                "unused-report.json",
                "--view-format",
                "yaml",
            ],
        );
        assert_eq!(invalid.status.code(), Some(2));
        assert_eq!(invalid.stdout, [] as [u8; 0]);
        assert!(String::from_utf8(invalid.stderr).unwrap().contains("--view-format"));
        let missing = run(
            &fixture.root,
            &[
                "assessment",
                "results",
                "append-epoch",
                "--request",
                "absent.json",
                "--output",
                "unused.json",
            ],
        );
        assert_eq!(missing.status.code(), Some(2));
        assert_eq!(missing.stdout, [] as [u8; 0]);
        assert!(String::from_utf8(missing.stderr).unwrap().contains("--report"));
        assert!(!fixture.root.join("unused.json").exists());
        assert!(!fixture.root.join("unused-report.json").exists());
    }

    /// Field bounds and complete pre-growth N/R accounting refuse otherwise coherent selected next-risk inputs.
    #[test]
    fn complete_projection_and_scalar_bounds_refuse_without_outputs() {
        let fixture = canary();
        let mut long_family = fixture.request.clone();
        long_family["new_risks"][1]["family_key"] = json!("x".repeat(257));
        refuse_request(&fixture, &long_family);
        let mut long_scalar = fixture.request.clone();
        long_scalar["next_epoch"]["result"]["description"] = json!("x".repeat(65_537));
        refuse_request(&fixture, &long_scalar);
        let mut heavy = fixture.request.clone();
        let assertion = provenance("2026-01-03T01:00:00Z");
        for position in 0..12 {
            let key = format!("bounded-extra-{position}");
            let mut added = risk(&key, "open", &assertion);
            added["description"] = json!("x".repeat(65_536));
            heavy["next_epoch"]["result"]["risks"].as_array_mut().unwrap().push(added);
            heavy["next_epoch"]["result"]["relationships"].as_array_mut().unwrap().push(json!({
                "from":{"type":"finding","key":"finding-next"},"to":{"type":"risk","key":key}}));
            heavy["new_risks"].as_array_mut().unwrap().push(json!({"next_key":key,
                "family_key":format!("bounded-family-{position}"),"prior":null,
                "caller_asserted_continuity":true,"provenance":assertion}));
        }
        let raw = serde_json::to_vec(&heavy).unwrap();
        assert!(
            raw.len() < 4 * 1024 * 1024,
            "projection control must pass the raw request ceiling"
        );
        assert_eq!(
            heavy["new_risks"].as_array().unwrap().len(),
            heavy["next_epoch"]["result"]["risks"].as_array().unwrap().len()
        );
        write_json(&fixture.root.join("append.json"), &heavy);
        let before = originals(&fixture);
        let output = append(
            &fixture.root,
            "append.json",
            "heavy-native.json",
            "heavy-report.json",
            &["--fail-on", "never"],
        );
        assert_eq!(output.status.code(), Some(2));
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("epoch shared conservative projection bound exceeded"),
            "this coherent control must reach the intended N/R refusal, not an earlier input gate"
        );
        assert_eq!(output.stdout, [] as [u8; 0]);
        assert!(!fixture.root.join("heavy-native.json").exists());
        assert!(!fixture.root.join("heavy-report.json").exists());
        assert_originals(&fixture, &before);
    }

    /// Supply complete planned caller work for two real reviewed risks, repeating one risk in another item.
    fn planned_request(mut scaffold: Value, references: &[Value]) -> Value {
        assert!(references.len() >= 2);
        let references = &references[..2];
        scaffold["roles"] = json!([{"id":"owner","title":"Caller owner role"}]);
        scaffold["parties"] = json!([{"key":"caller-owner","type":"person","name":"Caller owner"}]);
        let owner = json!({"role_id":"owner","party_key":"caller-owner","rationale":"Caller declared responsibility"});
        let initial = json!({"key":"initial","actor":{"role_id":"owner","party_key":"caller-owner"},
            "at":"2026-10-01T12:00:00Z","from":null,"to":"planned",
            "rationale":"Caller reviewed planned work","closure":null});
        scaffold["items"] = json!([
            {"key":"work-a","title":"Caller work A","description":"Caller description A",
                "source_refs":references,"owners":[owner],"target_date":"2026-10-05",
                "state":"planned","history":[initial],"milestones":[{"key":"step-a",
                    "outcome":"Caller measurable outcome A","target_date":"2026-10-04","depends_on":[],
                    "owners":[owner],"state":"planned","history":[initial]}]},
            {"key":"work-b","title":"Caller work B","description":"Caller description B",
                "source_refs":[references[1]],"owners":[owner],"target_date":"2026-10-03",
                "state":"planned","history":[initial],"milestones":[{"key":"step-b",
                    "outcome":"Caller measurable outcome B","target_date":"2026-10-02","depends_on":[],
                    "owners":[owner],"state":"planned","history":[initial]}]}]);
        json!({"schema_version":"forge.poam-risk-authoring/1","workflow":scaffold,
            "reviewed_risks":references.iter().map(|reference| json!({
                "source_ref":reference,"caller_asserted_reviewed":true})).collect::<Vec<_>>()})
    }

    /// Export complete caller planned work through the maintained consumed-risk command, never a transport-only validator.
    fn export_workflow(
        root: &Path,
        plan_name: &str,
        author_name: &str,
        request: &Value,
        workflow_name: &str,
    ) -> Value {
        write_json(&root.join(author_name), request);
        let output = run(
            root,
            &[
                "assessment",
                "results",
                "export-poam",
                "--scaffold",
                plan_name,
                "--authoring",
                author_name,
                "--as-of",
                "2026-10-04",
            ],
        );
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(output.stderr, [] as [u8; 0]);
        let workflow: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(workflow, request["workflow"]);
        std::fs::write(root.join(workflow_name), &output.stdout).unwrap();
        workflow
    }

    /// Consume complete workflow values through BOTH real check/build paths, current schedule and official native validation.
    fn consume_workflow(
        root: &Path,
        workflow_name: &str,
        native_name: &str,
        expected_source_objects: u64,
    ) -> Value {
        let checked = run(
            root,
            &[
                "poam",
                "check",
                "--manifest",
                workflow_name,
                "--workflow",
                "--as-of",
                "2026-10-04",
                "--due-soon-days",
                "7",
                "--format",
                "json",
            ],
        );
        assert_eq!(checked.status.code(), Some(1));
        assert_eq!(checked.stderr, [] as [u8; 0]);
        let checked: Value = serde_json::from_slice(&checked.stdout).unwrap();
        let built = run(
            root,
            &[
                "poam",
                "build",
                "--manifest",
                workflow_name,
                "--as-of",
                "2026-10-04",
                "--due-soon-days",
                "7",
                "--output",
                native_name,
                "--format",
                "json",
            ],
        );
        assert_eq!(built.status.code(), Some(1));
        assert_eq!(built.stderr, [] as [u8; 0]);
        let schedule: Value = serde_json::from_slice(&built.stdout).unwrap();
        assert_eq!(schedule, checked);
        assert_eq!(schedule["source_objects"], expected_source_objects);
        assert_eq!(schedule["items"], 2);
        assert_eq!(schedule["milestones"], 2);
        assert_eq!(schedule["rows"].as_array().unwrap().len(), 4);
        assert_eq!(schedule["overdue"], 2);
        assert_eq!(schedule["due_soon"], 2);
        let native: Value =
            serde_json::from_slice(&std::fs::read(root.join(native_name)).unwrap()).unwrap();
        assert!(
            forge::validate::validate_artifact(&native, forge::OscalModelType::Poam)
                .unwrap()
                .is_valid
        );
        let validated = run(root, &["validate", native_name, "--format", "json"]);
        assert_eq!(validated.status.code(), Some(0));
        native
    }

    /// Inspect genuine native POA&M source links against each exact selected risk and the original multi-epoch filename.
    fn assert_downstream_links(native: &Value, workflow: &Value) {
        let items = native["plan-of-action-and-milestones"]["poam-items"].as_array().unwrap();
        assert_eq!(items.len(), workflow["items"].as_array().unwrap().len());
        let source_name = workflow["source"]["assessment_results"]["artifact"].as_str().unwrap();
        for (item, authored) in items.iter().zip(workflow["items"].as_array().unwrap()) {
            let links = item["links"].as_array().unwrap();
            let references = authored["source_refs"].as_array().unwrap();
            assert_eq!(links.len(), references.len());
            for (link, reference) in links.iter().zip(references) {
                assert_eq!(link["rel"], "assessment-source");
                assert_eq!(
                    link["href"],
                    format!("{source_name}#{}", reference["uuid"].as_str().unwrap())
                );
            }
            assert!(item.get("related-risks").is_none());
        }
    }

    /// A local S3 handoff consumes the actual persisted native/workflow pair and explicit operation selection without remote mutation.
    fn consume_outbound(root: &Path, workflow_name: &str, native_name: &str, selection_name: &str) {
        write_json(
            &root.join(selection_name),
            &json!({"schema_version":"forge.poam-outbound-selection/1",
            "operations":[{"item_key":"work-a","intent":"create"},{"item_key":"work-b","intent":"update"}]}),
        );
        let root_text = root.to_str().unwrap();
        let output = run(
            root,
            &[
                "poam",
                "outbound",
                "--manifest",
                workflow_name,
                "--native",
                native_name,
                "--selection",
                selection_name,
                "--as-of",
                "2026-10-04",
                "--due-soon-days",
                "7",
                "--output-root",
                root_text,
            ],
        );
        assert_eq!(
            output.status.code(),
            Some(0),
            "valid local outbound differs from schedule-action exits"
        );
        assert_eq!(output.stderr, [] as [u8; 0]);
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["schema_version"], "forge.integration-change-set/1");
        assert_eq!(value["remote_apply_authorized"], false);
        assert_eq!(value["operation_count"], 2);
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(!text.contains("SENSITIVE"));
        assert!(!text.contains(root_text));
    }

    /// Old nonlatest and newly appended result selections remain genuine source inputs for init/check/build/S3 consumers.
    #[test]
    fn old_and_new_result_selections_are_consumed_by_existing_commands() {
        let fixture = canary();
        let second = second_pair(&fixture);
        let results = second.native["assessment-results"]["results"].as_array().unwrap();
        for (index, expected_objects) in [(0, 5_u64), (1, 3_u64)] {
            let plan_name = format!("selected-{index}-scaffold.json");
            let author_name = format!("selected-{index}-authoring.json");
            let workflow_name = format!("selected-{index}-workflow.json");
            let native_name = format!("selected-{index}-poam.json");
            let selection_name = format!("selected-{index}-selection.json");
            let scaffold = initialize(&fixture.root, "epoch-two.json", &results[index], &plan_name);
            let references = risk_references(&fixture.root, &plan_name, &scaffold);
            let source_only = run(
                &fixture.root,
                &["poam", "check", "--manifest", &plan_name, "--source-only", "--format", "json"],
            );
            assert_eq!(source_only.status.code(), Some(0));
            assert_eq!(source_only.stderr, [] as [u8; 0]);
            let request = planned_request(scaffold, &references);
            let workflow =
                export_workflow(&fixture.root, &plan_name, &author_name, &request, &workflow_name);
            let native =
                consume_workflow(&fixture.root, &workflow_name, &native_name, expected_objects);
            assert_downstream_links(&native, &workflow);
            consume_outbound(&fixture.root, &workflow_name, &native_name, &selection_name);
        }
    }

    /// Appended epochs do not confer terminal workflow authority or reopen the D064 owner/evidence gate.
    #[test]
    fn append_does_not_authorize_terminal_caller_work() {
        let fixture = canary();
        let second = second_pair(&fixture);
        let scaffold = initialize(
            &fixture.root,
            "epoch-two.json",
            &second.native["assessment-results"]["results"][1],
            "terminal-scaffold.json",
        );
        let references = risk_references(&fixture.root, "terminal-scaffold.json", &scaffold);
        let base = planned_request(scaffold, &references);
        for state in ["completed", "risk-accepted", "cancelled"] {
            let mut request = base.clone();
            request["workflow"]["items"][0]["state"] = json!(state);
            request["workflow"]["items"][0]["history"][0]["to"] = json!(state);
            write_json(&fixture.root.join("terminal-authoring.json"), &request);
            let output = run(
                &fixture.root,
                &[
                    "assessment",
                    "results",
                    "export-poam",
                    "--scaffold",
                    "terminal-scaffold.json",
                    "--authoring",
                    "terminal-authoring.json",
                    "--as-of",
                    "2026-10-04",
                ],
            );
            assert_eq!(output.status.code(), Some(2));
            assert_eq!(output.stdout, [] as [u8; 0]);
        }
    }
}
