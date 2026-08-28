//! PRD060 evidence and implementation linkage end-to-end contract tests.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn run_in(directory: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_forge"))
        .current_dir(directory)
        .args(args)
        .output()
        .expect("run forge")
}

fn write_json(path: &Path, value: &Value) {
    std::fs::write(path, serde_json::to_vec_pretty(value).expect("serialize fixture"))
        .expect("write fixture");
}

fn file_hash(path: &Path) -> String {
    format!("{:x}", Sha256::digest(std::fs::read(path).expect("read fixture")))
}

fn catalog() -> Value {
    json!({
        "catalog": {
            "uuid": "11111111-1111-4111-8111-111111111111",
            "metadata": {
                "title": "Synthetic requirement catalog",
                "last-modified": "2026-08-27T00:00:00Z",
                "version": "1.0.0",
                "oscal-version": "1.2.3"
            },
            "controls": [{
                "id": "ac-1",
                "title": "Synthetic access requirement",
                "parts": [{
                    "id": "ac-1_smt",
                    "name": "statement",
                    "prose": "Synthetic statement content."
                }]
            }]
        }
    })
}

fn component_definition() -> Value {
    json!({
        "component-definition": {
            "uuid": "22222222-2222-4222-8222-222222222222",
            "metadata": {
                "title": "Synthetic implementation",
                "last-modified": "2026-08-27T00:00:00Z",
                "version": "1.0.0",
                "oscal-version": "1.2.3"
            },
            "components": [{
                "uuid": "33333333-3333-4333-8333-333333333333",
                "type": "software",
                "title": "Synthetic component",
                "description": "Synthetic component used only for tests.",
                "control-implementations": [{
                    "uuid": "44444444-4444-4444-8444-444444444444",
                    "source": "catalog.json",
                    "description": "Reviewed implementation set.",
                    "implemented-requirements": [{
                        "uuid": "55555555-5555-4555-8555-555555555555",
                        "control-id": "ac-1",
                        "description": "Reviewer-authored implementation statement.",
                        "statements": [{
                            "statement-id": "ac-1_smt",
                            "uuid": "66666666-6666-4666-8666-666666666666",
                            "description": "Statement-level implementation."
                        }]
                    }]
                }]
            }]
        }
    })
}

struct Fixture {
    dir: tempfile::TempDir,
    manifest: PathBuf,
    evidence: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let catalog_path = dir.path().join("catalog.json");
        let implementation_path = dir.path().join("component.json");
        let evidence_dir = dir.path().join("evidence");
        let evidence = evidence_dir.join("record.bin");
        std::fs::create_dir(&evidence_dir).expect("evidence dir");
        write_json(&catalog_path, &catalog());
        write_json(&implementation_path, &component_definition());
        std::fs::write(&evidence, b"private evidence bytes\n").expect("evidence file");
        let manifest = dir.path().join("linkage.json");
        write_json(
            &manifest,
            &json!({
                "schema_version": "forge.linkage/1",
                "project": {
                    "key": "synthetic-project",
                    "title": "Synthetic linkage project",
                    "expiring_window_days": 30,
                    "max_evidence_bytes": 1_048_576,
                    "approved_uri_schemes": ["vault+corp"]
                },
                "reviewers": [{"key": "reviewer", "name": "Test Reviewer"}],
                "requirement_resources": [{
                    "key": "requirements",
                    "type": "catalog",
                    "artifact": "catalog.json",
                    "href": "catalog.json",
                    "expected_sha256": file_hash(&catalog_path)
                }],
                "implementation_resource": {
                    "key": "implementation",
                    "type": "component-definition",
                    "artifact": "component.json",
                    "href": "component.json",
                    "expected_sha256": file_hash(&implementation_path)
                },
                "evidence_roots": [{"key": "local", "path": "evidence"}],
                "evidence": [{
                    "key": "record",
                    "title": "Synthetic record",
                    "evidence_type": "test-record",
                    "owner": "control-owner",
                    "collected_at": "2026-08-20T00:00:00Z",
                    "valid_through": "2026-12-31",
                    "sensitivity_label": "restricted",
                    "source_label": "reviewed local export",
                    "location": {
                        "kind": "local",
                        "root_key": "local",
                        "path": "record.bin",
                        "expected_sha256": file_hash(&evidence),
                        "expected_size": std::fs::metadata(&evidence).unwrap().len()
                    }
                }],
                "links": [{
                    "key": "access-link",
                    "requirements": [
                        {"resource_key": "requirements", "type": "control", "id_ref": "ac-1"},
                        {"resource_key": "requirements", "type": "statement", "id_ref": "ac-1_smt"}
                    ],
                    "implementations": [
                        {"type": "implemented-requirement", "id_ref": "55555555-5555-4555-8555-555555555555"},
                        {"type": "statement", "id_ref": "66666666-6666-4666-8666-666666666666"}
                    ],
                    "evidence_keys": ["record"],
                    "evidence_required": true,
                    "responsible_role": "control-owner",
                    "implementation_status": "implemented",
                    "review": {
                        "reviewer_key": "reviewer",
                        "reviewed_at": "2026-08-21T00:00:00Z",
                        "rationale": "Reviewer associated these exact subjects and evidence metadata."
                    },
                    "impact_finding_ids": ["framework-impact-1"],
                    "policy_version_keys": ["policy-v1"]
                }]
            }),
        );
        Self { dir, manifest, evidence }
    }

    fn build(&self, output: &Path, extra: &[&str]) -> Output {
        let mut args = vec![
            "linkage",
            "build",
            "--manifest",
            self.manifest.to_str().unwrap(),
            "--as-of",
            "2026-08-27",
            "--output",
            output.to_str().unwrap(),
        ];
        args.extend_from_slice(extra);
        run_in(self.dir.path(), &args)
    }
}

#[test]
fn init_rejects_profile_companion_for_a_catalog() {
    let fixture = Fixture::new();
    let catalog = fixture.dir.path().join("catalog.json");
    let component = fixture.dir.path().join("component.json");
    let output = fixture.dir.path().join("invalid-scaffold.json");
    let result = run_in(
        fixture.dir.path(),
        &[
            "linkage",
            "init",
            "--requirement",
            catalog.to_str().unwrap(),
            "--resolved-catalog",
            catalog.to_str().unwrap(),
            "--implementation",
            component.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
        ],
    );
    assert_eq!(result.status.code(), Some(2));
    assert!(!output.exists());
    assert!(
        String::from_utf8_lossy(&result.stderr)
            .contains("--resolved-catalog is only valid for a Profile")
    );
}

#[test]
fn build_links_every_subject_type_without_copying_evidence() {
    let fixture = Fixture::new();
    let output = fixture.dir.path().join("index.json");
    let result = fixture.build(&output, &[]);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let bytes = std::fs::read(&output).expect("index");
    let index: Value = serde_json::from_slice(&bytes).expect("index JSON");
    assert_eq!(index["schema_version"], "forge.linkage-index/1");
    assert_eq!(index["links"][0]["requirements"].as_array().unwrap().len(), 2);
    assert_eq!(index["links"][0]["implementations"].as_array().unwrap().len(), 2);
    assert_eq!(index["evidence"][0]["freshness"], "current");
    assert_eq!(index["findings"], json!([]));
    let rendered = String::from_utf8(bytes).unwrap();
    assert!(!rendered.contains("private evidence bytes"));
    assert!(!rendered.contains(fixture.dir.path().to_str().unwrap()));
    for prohibited in ["audit-ready", "certified", "compliant", "sufficient", "effective", "passed"]
    {
        assert!(
            !rendered.to_lowercase().contains(prohibited),
            "found prohibited term {prohibited}"
        );
    }
}

#[test]
fn changed_evidence_is_a_stable_action_finding_and_preserves_output() {
    let fixture = Fixture::new();
    std::fs::write(&fixture.evidence, b"private evidence changed by one byte!\n").unwrap();
    let first = fixture.dir.path().join("first.json");
    let second = fixture.dir.path().join("second.json");
    let first_result = fixture.build(&first, &[]);
    let second_result = fixture.build(&second, &[]);
    assert_eq!(first_result.status.code(), Some(1));
    assert_eq!(second_result.status.code(), Some(1));
    assert_eq!(std::fs::read(&first).unwrap(), std::fs::read(&second).unwrap());
    let index: Value = serde_json::from_slice(&std::fs::read(first).unwrap()).unwrap();
    assert_eq!(index["findings"][0]["reason_code"], "evidence-changed");
    assert_eq!(index["evidence"][0]["freshness"], "changed");
    assert_ne!(
        index["evidence"][0]["reference"]["approved_sha256"],
        index["evidence"][0]["reference"]["observed_sha256"]
    );
}

#[test]
fn changed_evidence_check_reports_provenance_and_both_hashes() {
    let fixture = Fixture::new();
    std::fs::write(&fixture.evidence, b"private evidence changed by one byte!\n").unwrap();
    let report = fixture.dir.path().join("report.json");
    let result = run_in(
        fixture.dir.path(),
        &[
            "linkage",
            "check",
            "--manifest",
            fixture.manifest.to_str().unwrap(),
            "--as-of",
            "2026-08-27",
            "--format",
            "json",
            "--output",
            report.to_str().unwrap(),
        ],
    );
    assert_eq!(result.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&std::fs::read(report).unwrap()).unwrap();
    assert_eq!(report["schema_version"], "forge.linkage-report/1");
    assert!(report["provenance"]["manifest_sha256"].is_string());
    assert_ne!(
        report["evidence"][0]["reference"]["approved_sha256"],
        report["evidence"][0]["reference"]["observed_sha256"]
    );
    assert_eq!(report["findings"][0]["reason_code"], "evidence-changed");
}

#[test]
fn unlinked_implementation_subject_is_an_action_finding() {
    let fixture = Fixture::new();
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(&fixture.manifest).unwrap()).unwrap();
    manifest["links"][0]["implementations"].as_array_mut().unwrap().pop();
    write_json(&fixture.manifest, &manifest);
    let output = fixture.dir.path().join("index.json");
    let result = fixture.build(&output, &[]);
    assert_eq!(result.status.code(), Some(1));
    let index: Value = serde_json::from_slice(&std::fs::read(output).unwrap()).unwrap();
    assert!(index["findings"].as_array().unwrap().iter().any(|finding| {
        finding["reason_code"] == "implementation-subject-unlinked"
            && finding["action_required"] == true
    }));
}

#[test]
fn identical_projects_in_different_directories_are_byte_identical() {
    let first = Fixture::new();
    let second = Fixture::new();
    let first_output = first.dir.path().join("index.json");
    let second_output = second.dir.path().join("index.json");
    assert!(first.build(&first_output, &[]).status.success());
    assert!(second.build(&second_output, &[]).status.success());
    let first_bytes = std::fs::read(first_output).unwrap();
    let second_bytes = std::fs::read(second_output).unwrap();
    assert_eq!(first_bytes, second_bytes);
    assert!(!String::from_utf8(first_bytes).unwrap().contains(first.dir.path().to_str().unwrap()));
}

#[test]
fn uri_reports_are_redacted_and_never_fetched() {
    let fixture = Fixture::new();
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(&fixture.manifest).unwrap()).unwrap();
    manifest["evidence"] = json!([{
        "key": "record",
        "title": "Remote record",
        "evidence_type": "ticket",
        "owner": "control-owner",
        "collected_at": "2026-08-20T00:00:00Z",
        "sensitivity_label": "restricted",
        "source_label": "reviewed ticket reference",
        "location": {
            "kind": "uri",
            "uri": "https://user:password@example.invalid/ticket/1?token=secret#private",
            "unverified": true
        }
    }]);
    write_json(&fixture.manifest, &manifest);
    let report = fixture.dir.path().join("report.txt");
    let output = fixture.dir.path().join("index.json");
    let result =
        fixture.build(&output, &["--report", report.to_str().unwrap(), "--format", "text"]);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let combined = format!(
        "{}{}",
        std::fs::read_to_string(&output).unwrap(),
        std::fs::read_to_string(report).unwrap()
    );
    assert!(combined.contains("https://example.invalid/ticket/1"));
    let index: Value = serde_json::from_slice(&std::fs::read(output).unwrap()).unwrap();
    assert_eq!(index["findings"][0]["reason_code"], "evidence-uri-unverified");
    assert_eq!(index["findings"][0]["action_required"], false);
    for secret in ["user", "password", "token", "secret", "#private"] {
        assert!(!combined.contains(secret), "leaked URI field {secret}");
    }
}

#[test]
fn html_trace_and_owner_queue_remain_metadata_only() {
    let fixture = Fixture::new();
    std::fs::write(&fixture.evidence, b"changed private evidence\n").unwrap();
    let index = fixture.dir.path().join("index.json");
    let html = fixture.dir.path().join("trace.html");
    let build = fixture.build(&index, &["--report", html.to_str().unwrap(), "--format", "html"]);
    assert_eq!(build.status.code(), Some(1));
    let html_text = std::fs::read_to_string(&html).unwrap();
    assert!(html_text.contains("access-link"));
    assert!(html_text.contains("requirements:control:ac-1"));
    assert!(!html_text.contains("changed private evidence"));

    let queue = fixture.dir.path().join("queue.json");
    let queued = run_in(
        fixture.dir.path(),
        &[
            "linkage",
            "queue",
            "--manifest",
            fixture.manifest.to_str().unwrap(),
            "--as-of",
            "2026-08-27",
            "--output",
            queue.to_str().unwrap(),
        ],
    );
    assert_eq!(queued.status.code(), Some(1));
    let queue_json: Value = serde_json::from_slice(&std::fs::read(queue).unwrap()).unwrap();
    assert_eq!(queue_json["schema_version"], "forge.linkage-queue/1");
    assert_eq!(queue_json["groups"][0]["owner"], "control-owner");
}

#[test]
fn owner_queue_cannot_overwrite_a_referenced_evidence_file() {
    let fixture = Fixture::new();
    let original = std::fs::read(&fixture.evidence).unwrap();
    let queued = run_in(
        fixture.dir.path(),
        &[
            "linkage",
            "queue",
            "--manifest",
            fixture.manifest.to_str().unwrap(),
            "--as-of",
            "2026-08-27",
            "--output",
            fixture.evidence.to_str().unwrap(),
        ],
    );
    assert_eq!(queued.status.code(), Some(2));
    assert_eq!(std::fs::read(&fixture.evidence).unwrap(), original);
    assert!(String::from_utf8_lossy(&queued.stderr).contains("destination aliases an input"));
}

#[test]
fn wrong_side_duplicate_and_missing_subjects_are_invalid_analysis() {
    let fixture = Fixture::new();
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(&fixture.manifest).unwrap()).unwrap();
    manifest["links"][0]["requirements"][0]["type"] = json!("statement");
    manifest["links"][0]["requirements"][0]["id_ref"] = json!("ac-1");
    write_json(&fixture.manifest, &manifest);
    let output = fixture.dir.path().join("must-not-exist.json");
    let result = fixture.build(&output, &[]);
    assert_eq!(result.status.code(), Some(2));
    assert!(!output.exists());
    assert!(String::from_utf8_lossy(&result.stderr).contains("has type 'control'"));
}

#[test]
fn not_applicable_assertions_require_independent_review_evidence() {
    let fixture = Fixture::new();
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(&fixture.manifest).unwrap()).unwrap();
    manifest["links"][0]["implementation_status"] = json!("not-applicable");
    write_json(&fixture.manifest, &manifest);
    let output = fixture.dir.path().join("index.json");
    let rejected = fixture.build(&output, &[]);
    assert_eq!(rejected.status.code(), Some(2));
    assert!(!output.exists());

    manifest["links"][0]["not_applicable_review"] = json!({
        "reviewer_key": "reviewer",
        "reviewed_at": "2026-08-22T00:00:00Z",
        "rationale": "Reviewer separately reviewed the not-applicable assertion."
    });
    write_json(&fixture.manifest, &manifest);
    let accepted = fixture.build(&output, &[]);
    assert!(accepted.status.success(), "{}", String::from_utf8_lossy(&accepted.stderr));
}

#[test]
fn profile_requirement_requires_and_fingerprints_reviewed_companion() {
    let fixture = Fixture::new();
    let profile_path = fixture.dir.path().join("profile.json");
    write_json(
        &profile_path,
        &json!({
            "profile": {
                "uuid": "77777777-7777-4777-8777-777777777777",
                "metadata": {
                    "title": "Synthetic profile",
                    "last-modified": "2026-08-27T00:00:00Z",
                    "version": "1.0.0",
                    "oscal-version": "1.2.3"
                },
                "imports": [{"href": "catalog.json", "include-all": {}}]
            }
        }),
    );
    let catalog_path = fixture.dir.path().join("catalog.json");
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(&fixture.manifest).unwrap()).unwrap();
    manifest["requirement_resources"][0] = json!({
        "key": "requirements",
        "type": "profile",
        "artifact": "profile.json",
        "href": "profile.json",
        "resolved_catalog": "catalog.json",
        "resolved_catalog_attestation": true,
        "expected_sha256": file_hash(&profile_path),
        "expected_resolved_catalog_sha256": file_hash(&catalog_path)
    });
    write_json(&fixture.manifest, &manifest);
    let output = fixture.dir.path().join("index.json");
    let result = fixture.build(&output, &[]);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let index: Value = serde_json::from_slice(&std::fs::read(output).unwrap()).unwrap();
    assert_eq!(
        index["provenance"]["requirement_resources"][0]["resolved_catalog_sha256"],
        file_hash(&catalog_path)
    );
}

#[cfg(unix)]
#[test]
fn symlink_evidence_is_rejected_before_output_changes() {
    let fixture = Fixture::new();
    let link = fixture.dir.path().join("evidence").join("link.bin");
    std::os::unix::fs::symlink(&fixture.evidence, &link).unwrap();
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(&fixture.manifest).unwrap()).unwrap();
    manifest["evidence"][0]["location"]["path"] = json!("link.bin");
    write_json(&fixture.manifest, &manifest);
    let output = fixture.dir.path().join("existing.json");
    std::fs::write(&output, b"preserve-me").unwrap();
    let result = fixture.build(&output, &[]);
    assert_eq!(result.status.code(), Some(2));
    assert_eq!(std::fs::read(&output).unwrap(), b"preserve-me");
}

#[cfg(unix)]
#[test]
fn singly_declared_hard_link_and_non_file_evidence_are_rejected() {
    let fixture = Fixture::new();
    let alias = fixture.dir.path().join("evidence").join("alias.bin");
    std::fs::hard_link(&fixture.evidence, &alias).unwrap();
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(&fixture.manifest).unwrap()).unwrap();
    manifest["evidence"][0]["location"]["path"] = json!("alias.bin");
    write_json(&fixture.manifest, &manifest);
    let output = fixture.dir.path().join("alias-index.json");
    let alias_result = fixture.build(&output, &[]);
    assert_eq!(alias_result.status.code(), Some(2));
    assert!(!output.exists());

    let directory_path = fixture.dir.path().join("evidence").join("not-a-file");
    std::fs::create_dir(&directory_path).unwrap();
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(&fixture.manifest).unwrap()).unwrap();
    manifest["evidence"][0]["location"]["path"] = json!("not-a-file");
    write_json(&fixture.manifest, &manifest);
    let non_file_result = fixture.build(&output, &[]);
    assert_eq!(non_file_result.status.code(), Some(2));
    assert!(!output.exists());
}

#[test]
fn traversal_and_duplicate_cardinality_are_rejected() {
    let fixture = Fixture::new();
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(&fixture.manifest).unwrap()).unwrap();
    manifest["evidence"][0]["location"]["path"] = json!("../record.bin");
    write_json(&fixture.manifest, &manifest);
    let output = fixture.dir.path().join("index.json");
    let traversal = fixture.build(&output, &[]);
    assert_eq!(traversal.status.code(), Some(2));
    assert!(!output.exists());

    manifest["evidence"][0]["location"]["path"] = json!("record.bin");
    let duplicate = manifest["links"][0]["requirements"][0].clone();
    manifest["links"][0]["requirements"].as_array_mut().unwrap().push(duplicate);
    write_json(&fixture.manifest, &manifest);
    let duplicate_result = fixture.build(&output, &[]);
    assert_eq!(duplicate_result.status.code(), Some(2));
    assert!(!output.exists());
}

#[test]
fn baseline_distinguishes_subject_content_and_relationship_edits() {
    let fixture = Fixture::new();
    let baseline = fixture.dir.path().join("baseline.json");
    assert!(fixture.build(&baseline, &[]).status.success());

    let implementation_path = fixture.dir.path().join("component.json");
    let mut implementation: Value =
        serde_json::from_slice(&std::fs::read(&implementation_path).unwrap()).unwrap();
    implementation["component-definition"]["components"][0]["control-implementations"][0]["implemented-requirements"]
        [0]["description"] = json!("The implementation statement changed after review.");
    write_json(&implementation_path, &implementation);
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(&fixture.manifest).unwrap()).unwrap();
    manifest["implementation_resource"]["expected_sha256"] = json!(file_hash(&implementation_path));
    manifest["links"][0]["evidence_keys"] = json!([]);
    manifest["links"][0]["evidence_required"] = json!(false);
    write_json(&fixture.manifest, &manifest);

    let current = fixture.dir.path().join("current.json");
    let result =
        fixture.build(&current, &["--baseline", baseline.to_str().unwrap(), "--fail-on", "never"]);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let index: Value = serde_json::from_slice(&std::fs::read(current).unwrap()).unwrap();
    let reasons: Vec<_> = index["findings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| finding["reason_code"].as_str().unwrap())
        .collect();
    assert!(reasons.contains(&"implementation-subject-content-changed"));
    assert!(reasons.contains(&"relationship-edited"));
    assert!(reasons.contains(&"evidence-missing"));
}

#[test]
fn baseline_detects_changed_uri_reference_and_expected_hash() {
    let fixture = Fixture::new();
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(&fixture.manifest).unwrap()).unwrap();
    manifest["evidence"] = json!([{
        "key": "record",
        "title": "Remote record",
        "evidence_type": "ticket",
        "owner": "control-owner",
        "collected_at": "2026-08-20T00:00:00Z",
        "sensitivity_label": "restricted",
        "source_label": "reviewed ticket reference",
        "location": {
            "kind": "uri",
            "uri": "https://records.example.invalid/ticket/1",
            "unverified": true,
            "expected_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        }
    }]);
    write_json(&fixture.manifest, &manifest);
    let baseline = fixture.dir.path().join("baseline.json");
    assert!(fixture.build(&baseline, &[]).status.success());

    manifest["evidence"][0]["location"]["uri"] = json!("https://records.example.invalid/ticket/2");
    manifest["evidence"][0]["location"]["expected_sha256"] =
        json!("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
    write_json(&fixture.manifest, &manifest);
    let current = fixture.dir.path().join("current.json");
    let result =
        fixture.build(&current, &["--baseline", baseline.to_str().unwrap(), "--fail-on", "never"]);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let current: Value = serde_json::from_slice(&std::fs::read(current).unwrap()).unwrap();
    assert!(current["findings"].as_array().unwrap().iter().any(|finding| {
        finding["reason_code"] == "evidence-reference-changed" && finding["action_required"] == true
    }));
}

#[test]
fn system_security_plan_implemented_requirements_are_schema_valid_subjects() {
    let dir = tempfile::tempdir().expect("tempdir");
    let catalog_envelope: forge::oscal::catalog::CatalogEnvelope =
        serde_json::from_value(catalog()).expect("typed catalog");
    let ssp = forge::oscal::build_ssp_skeleton(
        "Synthetic policy",
        "1.0.0",
        &catalog_envelope.catalog,
        &[forge::oscal::ssp::SspComponentInput {
            title: "Synthetic service".to_string(),
            description: "Synthetic component for SSP linkage testing.".to_string(),
            component_type: forge::oscal::ssp::ComponentType::Service,
        }],
        "profile.json",
    )
    .expect("build SSP");
    let ssp_value = serde_json::to_value(&ssp).expect("serialize SSP");
    let validation =
        forge::validate::validate_artifact(&ssp_value, forge::OscalModelType::SystemSecurityPlan)
            .expect("validate SSP schema");
    assert!(validation.is_valid, "{:#?}", validation.errors);

    let catalog_path = dir.path().join("catalog.json");
    let ssp_path = dir.path().join("ssp.json");
    write_json(&catalog_path, &catalog());
    write_json(&ssp_path, &ssp_value);
    let implementation_uuid =
        ssp_value["system-security-plan"]["control-implementation"]["implemented-requirements"][0]
            ["uuid"]
            .as_str()
            .unwrap();
    let manifest_path = dir.path().join("linkage.json");
    write_json(
        &manifest_path,
        &json!({
            "schema_version": "forge.linkage/1",
            "project": {"key": "ssp-project", "title": "SSP project"},
            "reviewers": [{"key": "reviewer", "name": "Reviewer"}],
            "requirement_resources": [{
                "key": "requirements",
                "type": "catalog",
                "artifact": "catalog.json",
                "href": "catalog.json",
                "expected_sha256": file_hash(&catalog_path)
            }],
            "implementation_resource": {
                "key": "ssp",
                "type": "system-security-plan",
                "artifact": "ssp.json",
                "href": "ssp.json",
                "expected_sha256": file_hash(&ssp_path)
            },
            "links": [{
                "key": "ssp-link",
                "requirements": [{
                    "resource_key": "requirements",
                    "type": "control",
                    "id_ref": "ac-1"
                }],
                "implementations": [{
                    "type": "implemented-requirement",
                    "id_ref": implementation_uuid
                }],
                "evidence_required": false,
                "responsible_role": "system-owner",
                "implementation_status": "planned",
                "review": {
                    "reviewer_key": "reviewer",
                    "reviewed_at": "2026-08-27T00:00:00Z",
                    "rationale": "Reviewer associated the exact SSP implementation subject."
                }
            }]
        }),
    );
    let output_path = dir.path().join("index.json");
    let result = run_in(
        dir.path(),
        &[
            "linkage",
            "build",
            "--manifest",
            manifest_path.to_str().unwrap(),
            "--as-of",
            "2026-08-27",
            "--output",
            output_path.to_str().unwrap(),
        ],
    );
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let index: Value = serde_json::from_slice(&std::fs::read(output_path).unwrap()).unwrap();
    assert_eq!(index["provenance"]["implementation_resource"]["type"], "system-security-plan");
}
