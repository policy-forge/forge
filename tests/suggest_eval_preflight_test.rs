//! Synthetic development fixtures for artifact preflight; no human or model evidence.

mod common;
use common::sha256_hex;
use forge::suggest::eval::{
    manifest, preflight, preflight_to,
    report::{CaseState, EvidenceStatus, Status},
};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

struct Fixture {
    directory: tempfile::TempDir,
    manifest: Value,
}
impl Fixture {
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let mut fixture = Self { directory, manifest: json!({}) };
        let source = b"Supplied policy statement.";
        let source_ref = fixture.write("inputs/policy.md", source);
        fixture.write("nested/request/payload.txt", source);
        let request = json!({
            "schema_version":"forge.suggest-request/1", "project_key":"fixture-project", "as_of":"2026-10-02T00:00:00Z",
            "task":{"kind":"policy-drafting","schema_version":"forge.suggest-task-drafting/1","drafting_sections":[{
                "policy_key":"policy","topic_key":"topic","order":0,"title":"Fixture","prompt":"Propose policy text.","gap_ids":[],"control_ids":[]}]},
            "adapter":{"executable":"must-never-be-invoked","executable_sha256":"b".repeat(64),"model_id":"synthetic-fixture","argv":[]},
            "context":{"units":[{"unit_id":"unit-source","kind":"source-span","label":"Synthetic source","sensitivity":"internal",
                "source":{"key":"source","path":"policy.md","sha256":source_ref["sha256"],"start":0,"end":source.len()},
                "payload":{"start":0,"end":source.len()}}]},
            "redactions":[], "payload":{"artifact":"payload.txt","sha256":sha256_hex(source),"bytes":source.len(),"units":1},
            "retention_notice":"Synthetic development fixture; no execution consent."
        });
        let request_ref = fixture.write_json("nested/request/request.json", &request);
        let response = Self::response();
        let response_ref = fixture.write_json("nested/run/response.raw", &response);
        let run = json!({
            "schema_version":"forge.suggest-run/1", "request_sha256":request_ref["sha256"], "payload_sha256":sha256_hex(source),
            "response_sha256":response_ref["sha256"], "response_bytes":response_ref["bytes"],"response_artifact":"response.raw",
            "adapter_executable_sha256":"b".repeat(64),"model_id":"synthetic-fixture","argv":[],"redactions":[],"elapsed_ms":0,"exit_code":0,"mode":"recorded-response"
        });
        let run_ref = fixture.write_json("nested/run/run.json", &run);
        fixture.manifest = json!({
            "schema_version":"forge.suggest-eval-preflight-corpus/1","corpus_key":"fixture-corpus","version":"1",
            "cases":[{"case_key":"case-one","task":{"kind":"policy-drafting","schema_version":"forge.suggest-task-drafting/1"},
                "origin":"synthetic-development","stratum":"development","source_family":"fixture-family","workflow_group":"fixture-workflow",
                "source_base":"inputs","request":request_ref,"sources":[{"key":"source","artifact":source_ref}],
                "outcome":{"state":"response","run":run_ref,"response":response_ref,"adjudication":"missing"}}]
        });
        fixture
    }
    fn response() -> Value {
        json!({"schema_version":"forge.suggest-response/1","task":{"kind":"policy-drafting","schema_version":"forge.suggest-task-drafting/1",
            "draft_clauses":[{"policy_key":"policy","topic_key":"topic","draft_text":"Proposed policy clause.",
                "citations":[{"unit_id":"unit-source","quote":"Supplied policy statement."}],"assumptions":[],"unresolved_questions":[]}]}})
    }
    fn root(&self) -> &Path {
        self.directory.path()
    }
    fn write(&self, path: &str, bytes: &[u8]) -> Value {
        let target = self.root().join(path);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(target, bytes).unwrap();
        json!({"path":path,"sha256":sha256_hex(bytes),"bytes":bytes.len()})
    }
    fn write_json(&self, path: &str, value: &Value) -> Value {
        self.write(path, &serde_json::to_vec(value).unwrap())
    }
    fn read_json(&self, path: &str) -> Value {
        serde_json::from_slice(&std::fs::read(self.root().join(path)).unwrap()).unwrap()
    }
    fn store(&self) -> PathBuf {
        self.write_json("corpus.json", &self.manifest);
        PathBuf::from("corpus.json")
    }
    fn replace_response(&mut self, bytes: &[u8]) {
        let response = self.write("nested/run/response.raw", bytes);
        let mut run = self.read_json("nested/run/run.json");
        run["response_sha256"] = response["sha256"].clone();
        run["response_bytes"] = response["bytes"].clone();
        let run_ref = self.write_json("nested/run/run.json", &run);
        self.manifest["cases"][0]["outcome"]["response"] = response;
        self.manifest["cases"][0]["outcome"]["run"] = run_ref;
    }
    fn replace_request(&mut self, request: &Value) {
        let request_ref = self.write_json("nested/request/request.json", request);
        let mut run = self.read_json("nested/run/run.json");
        run["request_sha256"] = request_ref["sha256"].clone();
        let run_ref = self.write_json("nested/run/run.json", &run);
        self.manifest["cases"][0]["request"] = request_ref;
        self.manifest["cases"][0]["outcome"]["run"] = run_ref;
    }
}

#[test]
fn nested_layout_validates_exact_bytes_without_acceptance_or_execution() {
    let fixture = Fixture::new();
    let before: Vec<_> = [
        "inputs/policy.md",
        "nested/request/request.json",
        "nested/request/payload.txt",
        "nested/run/run.json",
        "nested/run/response.raw",
    ]
    .iter()
    .map(|path| std::fs::read(fixture.root().join(path)).unwrap())
    .collect();
    let report = preflight(fixture.root(), &fixture.store()).unwrap();
    assert_eq!(report.status, Status::Incomplete);
    assert!(report.preflight_complete);
    assert!(
        !report.acceptance_eligible && !report.generation_enabled && !report.task_selection_ready
    );
    assert_eq!(report.counts.expected, 1);
    assert_eq!(report.counts.artifacts_valid, 1);
    assert_eq!(report.cases[0].state, CaseState::AwaitingAdjudication);
    assert_eq!(report.cases[0].mechanical_counts.unwrap().suggestions, 1);
    assert!(report.pending_gates.iter().any(|gate| gate == "final-roadmap-documentation-review"));
    for (index, path) in [
        "inputs/policy.md",
        "nested/request/request.json",
        "nested/request/payload.txt",
        "nested/run/run.json",
        "nested/run/response.raw",
    ]
    .iter()
    .enumerate()
    {
        assert_eq!(std::fs::read(fixture.root().join(path)).unwrap(), before[index]);
    }
    assert!(!fixture.root().join("suggestions.json").exists());
    let raw = report.to_json_bytes().unwrap();
    assert!(!String::from_utf8(raw).unwrap().contains("Supplied policy statement"));
}

#[test]
fn malformed_response_does_not_drop_failed_blocked_or_unfinished_cases() {
    let mut fixture = Fixture::new();
    fixture.replace_response(b"{malformed SECRET RESPONSE PROSE");
    let base = fixture.manifest["cases"][0].clone();
    for (key, state) in
        [("case-two", "not-run"), ("case-three", "execution-blocked"), ("case-four", "tool-error")]
    {
        let mut case = base.clone();
        case["case_key"] = json!(key);
        case["outcome"] = json!({"state":state});
        fixture.manifest["cases"].as_array_mut().unwrap().push(case);
    }
    let report = preflight(fixture.root(), &fixture.store()).unwrap();
    assert_eq!(report.status, Status::Failed);
    assert_eq!(report.counts.expected, 4);
    assert_eq!(report.counts.response_invalid, 1);
    assert_eq!(report.counts.not_run, 1);
    assert_eq!(report.counts.execution_blocked, 1);
    assert_eq!(report.counts.tool_error, 1);
    assert_eq!(report.cases.len(), 4);
    assert!(!report.preflight_complete && !report.acceptance_eligible);
    assert!(!String::from_utf8(report.to_json_bytes().unwrap()).unwrap().contains("SECRET"));
}

#[test]
fn wrong_task_target_citation_and_quote_are_refused_as_candidate_outcomes() {
    for change in [0, 1, 2, 3] {
        let mut fixture = Fixture::new();
        let mut response = Fixture::response();
        match change {
            0 => {
                response["task"]["kind"] = json!("mapping-candidates");
                response["task"]["schema_version"] = json!("forge.suggest-task-mapping/1");
            }
            1 => response["task"]["draft_clauses"][0]["topic_key"] = json!("invented"),
            2 => {
                response["task"]["draft_clauses"][0]["citations"][0]["unit_id"] = json!("invented");
            }
            _ => {
                response["task"]["draft_clauses"][0]["citations"][0]["quote"] =
                    json!("Altered quote.");
            }
        }
        fixture.replace_response(&serde_json::to_vec(&response).unwrap());
        let report = preflight(fixture.root(), &fixture.store()).unwrap();
        assert_eq!(report.counts.response_invalid, 1, "case {change}");
        assert_eq!(report.counts.artifacts_valid, 0);
    }
}

#[test]
fn source_base_and_importer_relative_response_must_bind_the_right_file() {
    let mut fixture = Fixture::new();
    fixture.manifest["cases"][0]["source_base"] = json!("other");
    assert!(
        preflight(fixture.root(), &fixture.store())
            .unwrap_err()
            .to_string()
            .contains("source base/path/SHA-256")
    );
    let mut fixture = Fixture::new();
    let bytes = std::fs::read(fixture.root().join("nested/run/response.raw")).unwrap();
    fixture.manifest["cases"][0]["outcome"]["response"] = fixture.write("response.raw", &bytes);
    assert!(
        preflight(fixture.root(), &fixture.store())
            .unwrap_err()
            .to_string()
            .contains("href resolves to a different")
    );
}

#[test]
fn changed_source_hash_or_request_run_binding_is_invalid_trusted_input() {
    let fixture = Fixture::new();
    std::fs::write(fixture.root().join("inputs/policy.md"), b"Changed source bytes.").unwrap();
    assert!(
        preflight(fixture.root(), &fixture.store())
            .unwrap_err()
            .to_string()
            .contains("exact SHA-256 and byte pin")
    );
    let mut fixture = Fixture::new();
    let mut run = fixture.read_json("nested/run/run.json");
    run["request_sha256"] = json!("c".repeat(64));
    fixture.manifest["cases"][0]["outcome"]["run"] =
        fixture.write_json("nested/run/run.json", &run);
    assert!(
        preflight(fixture.root(), &fixture.store())
            .unwrap_err()
            .to_string()
            .contains("does not bind the exact")
    );
}

#[test]
fn opaque_approval_strings_and_process_mode_never_qualify_authentic_evidence() {
    let mut fixture = Fixture::new();
    let opaque = fixture.write(
        "private/attestation.json",
        b"{\"approved\":true,\"reviewer\":\"PRIVATE REVIEWER\"}",
    );
    fixture.manifest["evidence"] = json!({"rights":[opaque.clone()],"judgments":[opaque.clone()],"thresholds":[opaque.clone()],"execution_profile":[opaque.clone()],"freeze":[opaque.clone()],"candidate":[opaque]});
    let mut run = fixture.read_json("nested/run/run.json");
    run["mode"] = json!("process");
    fixture.manifest["cases"][0]["outcome"]["run"] =
        fixture.write_json("nested/run/run.json", &run);
    let report = preflight(fixture.root(), &fixture.store()).unwrap();
    assert!(
        report.evidence_gates.iter().all(|gate| gate.status == EvidenceStatus::OpaqueUnsupported)
    );
    assert_eq!(report.cases[0].declared_run_mode, Some(forge::suggest::RunMode::Process));
    assert!(
        !report.acceptance_eligible && !report.task_selection_ready && !report.generation_enabled
    );
    assert_eq!(report.status, Status::Incomplete);
    let raw = String::from_utf8(report.to_json_bytes().unwrap()).unwrap();
    assert!(!raw.contains("PRIVATE REVIEWER") && !raw.contains("private/attestation"));
}

#[test]
fn disputes_and_empty_valid_responses_stay_unadjudicated() {
    let mut fixture = Fixture::new();
    fixture.manifest["cases"][0]["outcome"]["adjudication"] = json!("disputed");
    let mut response = Fixture::response();
    response["task"]["draft_clauses"] = json!([]);
    fixture.replace_response(&serde_json::to_vec(&response).unwrap());
    let report = preflight(fixture.root(), &fixture.store()).unwrap();
    assert_eq!(report.counts.adjudication_disputed, 1);
    assert_eq!(report.cases[0].mechanical_counts.unwrap().suggestions, 0);
    assert!(!report.acceptance_eligible);
}

#[test]
fn closed_metadata_and_case_cardinality_fail_before_artifact_processing() {
    let fixture = Fixture::new();
    let valid = serde_json::to_vec(&fixture.manifest).unwrap();
    assert!(manifest::Corpus::parse(&valid).is_ok());
    for alteration in ["unknown", "null", "version", "duplicate", "empty", "excess"] {
        let mut value = fixture.manifest.clone();
        match alteration {
            "unknown" => value["unexpected"] = json!(true),
            "null" => value["evidence"] = Value::Null,
            "version" => value["schema_version"] = json!("forge.suggest-eval-preflight-corpus/2"),
            "duplicate" => {
                let case = value["cases"][0].clone();
                value["cases"].as_array_mut().unwrap().push(case);
            }
            "empty" => value["cases"] = json!([]),
            _ => {
                let case = value["cases"][0].clone();
                value["cases"] = Value::Array(
                    (0..=manifest::MAX_CASES)
                        .map(|i| {
                            let mut case = case.clone();
                            case["case_key"] = json!(format!("case-{i}"));
                            case
                        })
                        .collect(),
                );
            }
        }
        assert!(
            manifest::Corpus::parse(&serde_json::to_vec(&value).unwrap()).is_err(),
            "{alteration}"
        );
    }
    let duplicate = br#"{"schema_version":"forge.suggest-eval-preflight-corpus/1","schema_version":"forge.suggest-eval-preflight-corpus/1"}"#;
    assert!(manifest::Corpus::parse(duplicate).is_err());
    assert!(
        manifest::Corpus::parse(&vec![
            b' ';
            usize::try_from(manifest::MAX_MANIFEST_BYTES).unwrap() + 1
        ])
        .is_err()
    );
}

#[test]
fn trusted_inventory_limits_and_missing_request_fail_before_missing_file_reads() {
    for alteration in ["source-excess", "source-key", "request-missing", "evidence-excess"] {
        let mut fixture = Fixture::new();
        let expected = match alteration {
            "source-excess" => {
                fixture.manifest["cases"][0]["sources"] = Value::Array(
                    (0..=manifest::MAX_SOURCES).map(|index| json!({
                        "key":format!("source-{index}"),
                        "artifact":{"path":format!("inputs/missing-{index}.md"),"sha256":"a".repeat(64),"bytes":1}
                    })).collect());
                "source inventory exceeds"
            }
            "source-key" => {
                let mut duplicate = fixture.manifest["cases"][0]["sources"][0].clone();
                duplicate["artifact"]["path"] = json!("inputs/missing-duplicate.md");
                fixture.manifest["cases"][0]["sources"].as_array_mut().unwrap().push(duplicate);
                "duplicate evaluation source key"
            }
            "request-missing" => {
                fixture.manifest["cases"][0].as_object_mut().unwrap().remove("request");
                "response outcome requires a prepared request"
            }
            _ => {
                fixture.manifest["evidence"] = json!({"rights": (0..=manifest::MAX_EVIDENCE_REFS).map(|index| json!({
                    "path":format!("evidence/missing-{index}.json"),"sha256":"a".repeat(64),"bytes":1
                })).collect::<Vec<_>>()});
                "attestation inventory exceeds"
            }
        };
        let error = preflight(fixture.root(), &fixture.store()).unwrap_err().to_string();
        assert!(error.contains(expected), "alteration {alteration}: {error}");
    }
}

#[test]
fn unsafe_paths_aliases_and_source_byte_bound_are_rejected() {
    for path in ["../outside.md", "/absolute.md", "inputs/../policy.md"] {
        let mut fixture = Fixture::new();
        fixture.manifest["cases"][0]["sources"][0]["artifact"]["path"] = json!(path);
        assert!(preflight(fixture.root(), &fixture.store()).is_err());
    }
    let mut fixture = Fixture::new();
    let mut duplicate = fixture.manifest["cases"][0]["sources"][0].clone();
    duplicate["key"] = json!("other-source");
    fixture.manifest["cases"][0]["sources"].as_array_mut().unwrap().push(duplicate);
    assert!(preflight(fixture.root(), &fixture.store()).is_err());
    let mut fixture = Fixture::new();
    fixture.manifest["cases"][0]["sources"][0]["artifact"]["bytes"] = json!(1024 * 1024 + 1);
    assert!(
        preflight(fixture.root(), &fixture.store())
            .unwrap_err()
            .to_string()
            .contains("byte declaration exceeds")
    );
}

#[cfg(unix)]
#[test]
fn symbolic_and_hard_link_inputs_are_not_accepted() {
    let fixture = Fixture::new();
    let path = fixture.root().join("inputs/policy.md");
    let saved = fixture.root().join("saved.md");
    std::fs::rename(&path, &saved).unwrap();
    std::os::unix::fs::symlink(&saved, &path).unwrap();
    assert!(preflight(fixture.root(), &fixture.store()).is_err());
    std::fs::remove_file(&path).unwrap();
    std::fs::hard_link(&saved, &path).unwrap();
    assert!(preflight(fixture.root(), &fixture.store()).is_err());
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn no_replace_publication_is_deterministic_and_contains_only_the_preflight_receipt() {
    let fixture = Fixture::new();
    let manifest = fixture.store();
    let output = fixture.root().join("preflight-output");
    let report = preflight_to(fixture.root(), &manifest, Path::new("preflight-output")).unwrap();
    let bytes = std::fs::read(output.join("preflight.json")).unwrap();
    assert_eq!(bytes, report.to_json_bytes().unwrap());
    assert_eq!(std::fs::read_dir(&output).unwrap().count(), 1);
    assert_eq!(preflight(fixture.root(), &manifest).unwrap(), report);
    assert!(preflight_to(fixture.root(), &manifest, Path::new("preflight-output")).is_err());
    assert_eq!(std::fs::read(output.join("preflight.json")).unwrap(), bytes);
    assert!(!fixture.root().join("mapping.json").exists());
    assert!(!fixture.root().join("forge.authoring.json").exists());
}

#[test]
fn payload_utf8_span_boundaries_are_checked_even_without_quoted_citations() {
    let mut fixture = Fixture::new();
    let payload = "é".as_bytes();
    fixture.write("nested/request/payload.txt", payload);
    let mut request = fixture.read_json("nested/request/request.json");
    request["payload"]["sha256"] = json!(sha256_hex(payload));
    request["payload"]["bytes"] = json!(payload.len());
    request["context"]["units"][0]["payload"] = json!({"start":0,"end":1});
    fixture.replace_request(&request);
    assert!(
        preflight(fixture.root(), &fixture.store())
            .unwrap_err()
            .to_string()
            .contains("UTF-8 boundaries")
    );
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn supplied_quarantine_bundle_and_retained_copy_are_checked_against_reconstruction() {
    let mut fixture = Fixture::new();
    let request = fixture.root().join("nested/request/request.json");
    let run = fixture.root().join("nested/run/run.json");
    let output = fixture.root().join("nested/request/bundle");
    // The ordinary producer creates this synthetic quarantine fixture, without
    // invoking a model. Only the subsequent preflight operation is under test.
    forge::suggest::validate::execute(&forge::suggest::validate::ValidateArgs {
        request: &request,
        run: &run,
        output_dir: Path::new("bundle"),
        retain_raw: true,
        format: forge::cli::AuthorReportFormat::Text,
    })
    .unwrap();
    let bytes = std::fs::read(output.join("suggestions.json")).unwrap();
    let bundle_ref = json!({"path":"nested/request/bundle/suggestions.json","sha256":sha256_hex(&bytes),"bytes":bytes.len()});
    fixture.manifest["cases"][0]["outcome"]["bundle"] = bundle_ref;
    let report = preflight(fixture.root(), &fixture.store()).unwrap();
    assert_eq!(report.counts.artifacts_valid, 1);
    assert!(report.cases[0].artifacts.iter().any(|pin| pin.role == "retained-response"));
    let mut bundle: Value = serde_json::from_slice(&bytes).unwrap();
    let retained_path =
        format!("nested/request/bundle/{}", bundle["response"]["artifact"].as_str().unwrap());
    let retained = std::fs::read(fixture.root().join(&retained_path)).unwrap();
    let mut altered = retained.clone();
    altered[0] ^= 1;
    std::fs::write(fixture.root().join(&retained_path), &altered).unwrap();
    assert!(
        preflight(fixture.root(), &fixture.store())
            .unwrap_err()
            .to_string()
            .contains("exact SHA-256 and byte pin")
    );
    std::fs::write(fixture.root().join(&retained_path), &retained).unwrap();
    bundle["suggestions"][0]["evidence_support"] = json!("low");
    bundle["counts"]["high"] = json!(0);
    bundle["counts"]["low"] = json!(1);
    fixture.manifest["cases"][0]["outcome"]["bundle"] =
        fixture.write_json("nested/request/bundle/suggestions.json", &bundle);
    assert!(
        preflight(fixture.root(), &fixture.store())
            .unwrap_err()
            .to_string()
            .contains("differs from exact captured response validation")
    );
}

#[test]
fn contradictory_known_pins_are_rejected_before_case_files_are_opened() {
    let mut fixture = Fixture::new();
    let mut first = fixture.manifest["cases"][0].clone();
    first.as_object_mut().unwrap().remove("request");
    first["outcome"] = json!({"state":"not-run"});
    first["sources"] = json!([{"key":"source","artifact":{"path":"missing/source.md","sha256":"a".repeat(64),"bytes":1}}]);
    let mut second = first.clone();
    second["case_key"] = json!("case-two");
    second["sources"][0]["artifact"]["sha256"] = json!("b".repeat(64));
    fixture.manifest["cases"] = json!([first, second]);
    assert!(
        preflight(fixture.root(), &fixture.store())
            .unwrap_err()
            .to_string()
            .contains("references disagree about the same artifact")
    );
    // Even a case alias is refused before the missing target can be opened.
    fixture.manifest["cases"][1]["sources"][0]["artifact"]["sha256"] = json!("a".repeat(64));
    fixture.manifest["cases"][1]["sources"][0]["artifact"]["path"] = json!("missing/Source.md");
    assert!(
        preflight(fixture.root(), &fixture.store())
            .unwrap_err()
            .to_string()
            .contains("case-distinct paths")
    );
}

#[test]
fn declared_inventory_and_total_byte_bounds_precede_missing_artifact_reads() {
    let mut fixture = Fixture::new();
    let mut base = fixture.manifest["cases"][0].clone();
    base.as_object_mut().unwrap().remove("request");
    base["outcome"] = json!({"state":"not-run"});
    let cases: Vec<_> = (0..64).map(|case_index| {
        let mut case = base.clone();
        case["case_key"] = json!(format!("case-{case_index}"));
        case["sources"] = Value::Array((0..64).map(|source_index| json!({
            "key":format!("source-{source_index}"),
            "artifact":{"path":format!("missing/case-{case_index}/source-{source_index}.md"),"sha256":"a".repeat(64),"bytes":1}
        })).collect());
        case
    }).collect();
    fixture.manifest["cases"] = Value::Array(cases);
    // The manifest itself plus 4,096 declared sources is one artifact too many.
    assert!(
        preflight(fixture.root(), &fixture.store())
            .unwrap_err()
            .to_string()
            .contains("declared artifact inventory exceeds")
    );
    fixture.manifest["cases"][63]["sources"].as_array_mut().unwrap().pop();
    assert!(
        preflight(fixture.root(), &fixture.store())
            .unwrap_err()
            .to_string()
            .contains("unsafe, unreadable")
    );
    let cases: Vec<_> = (0..17).map(|case_index| {
        let mut case = base.clone();
        case["case_key"] = json!(format!("case-{case_index}"));
        case["sources"] = Value::Array((0..32).map(|source_index| json!({
            "key":format!("source-{source_index}"),
            "artifact":{"path":format!("missing/case-{case_index}/source-{source_index}.md"),"sha256":"a".repeat(64),"bytes":1024*1024}
        })).collect());
        case
    }).collect();
    fixture.manifest["cases"] = Value::Array(cases);
    // Each case stays below 50 MiB, but declared unique bytes exceed 512 MiB.
    assert!(
        preflight(fixture.root(), &fixture.store())
            .unwrap_err()
            .to_string()
            .contains("declared unique artifact bytes exceed")
    );
    fixture.manifest["cases"] = json!([base]);
    fixture.manifest["cases"][0]["sources"] = Value::Array((0..51).map(|index| json!({
        "key":format!("source-{index}"),"artifact":{"path":format!("missing/source-{index}.md"),"sha256":"a".repeat(64),"bytes":1024*1024}
    })).collect());
    assert!(
        preflight(fixture.root(), &fixture.store())
            .unwrap_err()
            .to_string()
            .contains("declared case or evidence bytes exceed")
    );
}

#[test]
fn report_encoder_refuses_mutated_qualification_denominators_and_state_claims() {
    let fixture = Fixture::new();
    let report = preflight(fixture.root(), &fixture.store()).unwrap();
    assert!(report.to_json_bytes().is_ok());
    for alteration in 0..14 {
        let mut changed = report.clone();
        match alteration {
            0 => changed.acceptance_eligible = true,
            1 => changed.generation_enabled = true,
            2 => changed.task_selection_ready = true,
            3 => changed.counts.expected = 0,
            4 => changed.counts.expected = 257,
            5 => changed.counts.awaiting_adjudication = 0,
            6 => changed.preflight_complete = false,
            7 => changed.status = Status::Failed,
            8 => changed.scope = "full-evaluation".to_owned(),
            9 => changed.schema_version = "forge.suggest-eval-preflight/2".to_owned(),
            10 => changed.cases[0].state = CaseState::NotRun,
            11 => changed.cases[0].artifacts_valid = false,
            12 => changed.cases.clear(),
            _ => {
                changed.cases.push(changed.cases[0].clone());
                changed.counts.expected = 2;
                changed.counts.artifacts_valid = 2;
                changed.counts.awaiting_adjudication = 2;
            }
        }
        assert!(changed.to_json_bytes().is_err(), "alteration {alteration}");
    }
    let mut fixture = Fixture::new();
    fixture.replace_response(b"{malformed");
    let mut report = preflight(fixture.root(), &fixture.store()).unwrap();
    report.status = Status::Incomplete;
    assert!(report.to_json_bytes().unwrap_err().to_string().contains("status contradicts"));
}

#[cfg(unix)]
#[test]
fn symbolic_root_alias_is_rejected_before_manifest_capture() {
    let fixture = Fixture::new();
    let manifest = fixture.store();
    let parent = tempfile::tempdir().unwrap();
    let alias = parent.path().join("corpus-alias");
    std::os::unix::fs::symlink(fixture.root(), &alias).unwrap();
    assert!(
        preflight(&alias, &manifest)
            .unwrap_err()
            .to_string()
            .contains("root must be a directory, not a symbolic link")
    );
}

#[test]
fn private_unknown_null_keys_are_not_echoed_in_metadata_diagnostics() {
    let mut fixture = Fixture::new();
    fixture.manifest["PRIVATE SOURCE PROSE"] = Value::Null;
    let message = preflight(fixture.root(), &fixture.store()).unwrap_err().to_string();
    assert!(message.contains("must omit null values"));
    assert!(!message.contains("PRIVATE SOURCE PROSE"));
}

#[test]
fn mapping_task_preflight_reconstructs_mechanical_counts_without_acceptance() {
    let mut fixture = Fixture::new();
    let mut request = fixture.read_json("nested/request/request.json");
    request["task"] = json!({"kind":"mapping-candidates","schema_version":"forge.suggest-task-mapping/1",
        "mapping_subjects":[{"policy_key":"policy","topic_key":"topic","title":"Fixture mapping",
            "text":"Supplied policy statement.","control_ids":["ac-1"],"gap_ids":[]}]});
    fixture.replace_request(&request);
    fixture.manifest["cases"][0]["task"] =
        json!({"kind":"mapping-candidates","schema_version":"forge.suggest-task-mapping/1"});
    let response = json!({"schema_version":"forge.suggest-response/1",
        "task":{"kind":"mapping-candidates","schema_version":"forge.suggest-task-mapping/1",
            "mapping_candidates":[{"policy_key":"policy","topic_key":"topic","control_id":"ac-1",
                "relationship":"intersects-with","rationale":"Synthetic candidate rationale.",
                "citations":[{"unit_id":"unit-source","quote":"Supplied policy statement."}],
                "assumptions":[],"unresolved_questions":[]}]}});
    fixture.replace_response(&serde_json::to_vec(&response).unwrap());
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let request_path = fixture.root().join("nested/request/request.json");
        let run = fixture.root().join("nested/run/run.json");
        let output = fixture.root().join("nested/request/mapping-bundle");
        forge::suggest::validate::execute(&forge::suggest::validate::ValidateArgs {
            request: &request_path,
            run: &run,
            output_dir: Path::new("mapping-bundle"),
            retain_raw: false,
            format: forge::cli::AuthorReportFormat::Text,
        })
        .unwrap();
        let bundle = std::fs::read(output.join("suggestions.json")).unwrap();
        fixture.manifest["cases"][0]["outcome"]["bundle"] = json!({
            "path":"nested/request/mapping-bundle/suggestions.json","sha256":sha256_hex(&bundle),"bytes":bundle.len()});
    }
    let report = preflight(fixture.root(), &fixture.store()).unwrap();
    assert_eq!(report.cases[0].task, forge::suggest::task::TaskKind::MappingCandidates);
    let counts = report.cases[0].mechanical_counts.unwrap();
    assert_eq!(counts.suggestions, 1);
    assert_eq!(counts.mapping, 1);
    assert_eq!(counts.drafting, 0);
    assert_eq!(counts.high, 1);
    assert_eq!(report.counts.artifacts_valid, 1);
    assert_eq!(report.status, Status::Incomplete);
    assert!(
        report.preflight_complete && !report.acceptance_eligible && !report.task_selection_ready
    );
}

/// Invoke only Forge's artifact preflight command; no adapter is executed.
fn cli_preflight(fixture: &Fixture, extra: &[&str]) -> std::process::Output {
    fixture.store();
    std::process::Command::new(env!("CARGO_BIN_EXE_forge"))
        .args(["suggest", "eval", "preflight", "--root"])
        .arg(fixture.root())
        .args(["--manifest", "corpus.json"])
        .args(extra)
        .output()
        .unwrap()
}

#[test]
fn cli_reports_valid_and_invalid_candidates_as_action_required_without_execution() {
    let mut fixture = Fixture::new();
    let output = cli_preflight(&fixture, &["--format", "json"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stderr.len(), 0);
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["schema_version"], json!("forge.suggest-eval-preflight/1"));
    assert_eq!(report["status"], json!("incomplete"));
    assert_eq!(report["acceptance_eligible"], json!(false));
    assert_eq!(report["counts"]["expected"], json!(1));
    assert!(!fixture.root().join("preflight.json").exists());
    let text = cli_preflight(&fixture, &[]);
    assert_eq!(text.status.code(), Some(1));
    assert!(String::from_utf8(text.stdout).unwrap().contains("gates remain open"));

    fixture.replace_response(b"{invalid PRIVATE CANDIDATE PROSE");
    let failed = cli_preflight(&fixture, &["--format", "json"]);
    assert_eq!(failed.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&failed.stdout).unwrap();
    assert_eq!(report["status"], json!("failed"));
    assert_eq!(report["counts"]["response_invalid"], json!(1));
    assert!(!String::from_utf8(failed.stdout).unwrap().contains("PRIVATE"));
}

#[test]
fn cli_rejects_invalid_trusted_metadata_without_output_or_publication() {
    let mut fixture = Fixture::new();
    fixture.manifest["PRIVATE UNKNOWN FIELD"] = Value::Null;
    let output = cli_preflight(&fixture, &["--format", "json", "--output-dir", "receipt"]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stdout.len(), 0);
    let diagnostic = String::from_utf8(output.stderr).unwrap();
    assert!(diagnostic.contains("must omit null values"));
    assert!(!diagnostic.contains("PRIVATE"));
    assert!(!fixture.root().join("receipt").exists());
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn cli_publication_matches_stdout_and_refuses_existing_generations() {
    let fixture = Fixture::new();
    let output = cli_preflight(&fixture, &["--format", "json", "--output-dir", "receipt"]);
    assert_eq!(output.status.code(), Some(1));
    let receipt = fixture.root().join("receipt/preflight.json");
    assert_eq!(std::fs::read(&receipt).unwrap(), output.stdout);
    assert_eq!(std::fs::read_dir(fixture.root().join("receipt")).unwrap().count(), 1);
    let again = cli_preflight(&fixture, &["--format", "json", "--output-dir", "receipt"]);
    assert_eq!(again.status.code(), Some(2));
    assert_eq!(again.stdout.len(), 0);
    assert_eq!(std::fs::read(receipt).unwrap(), output.stdout);
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
#[test]
fn unsupported_platform_publication_fails_before_staging() {
    let fixture = Fixture::new();
    let manifest = fixture.store();
    assert!(preflight(fixture.root(), &manifest).is_ok());
    assert!(preflight_to(fixture.root(), &manifest, Path::new("receipt")).is_err());
    let output = cli_preflight(&fixture, &["--format", "json", "--output-dir", "receipt"]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stdout.len(), 0);
    assert!(String::from_utf8(output.stderr).unwrap().contains("unsupported on this platform"));
    assert!(!fixture.root().join("receipt").exists());
}
