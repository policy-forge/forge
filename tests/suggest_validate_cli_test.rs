//! `forge suggest validate` end-to-end contracts: closed decode, citation
//! resolution, exact quoting, quarantine bundle and refusal paths.

use std::path::{Path, PathBuf};
use std::process::Output;

use serde_json::{Value, json};

mod common;
use common::sha256_hex as hash;
use common::{assert_exit, corpus, project, run};

const BODY: &str = "# Access drafting\n\nc-1 Approve access requests quarterly.\n";

fn expect(output: &Output, code: i32, label: &str) {
    assert_eq!(
        output.status.code(),
        Some(code),
        "{label}\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn clause(unit_id: &str, quote: Option<&str>) -> Value {
    let mut citations = vec![json!({"unit_id": unit_id})];
    if let Some(quote) = quote {
        citations = vec![json!({"unit_id": unit_id, "quote": quote})];
    }
    json!({
        "policy_key": "access-policy",
        "topic_key": "access-topic",
        "draft_text": "Access requests are approved quarterly.",
        "citations": citations,
        "assumptions": [],
        "unresolved_questions": ["Who approves?"]
    })
}

fn candidate() -> Value {
    json!({
        "policy_key": "access-policy",
        "topic_key": "access-topic",
        "control_id": "c-1",
        "relationship": "intersects-with",
        "rationale": "The supplied statement covers account review.",
        "citations": [{"unit_id": "unit-0001"}],
        "assumptions": [],
        "unresolved_questions": []
    })
}

fn response(clauses: &[Value], kind: &str, version: &str) -> Vec<u8> {
    let task = if kind == "policy-drafting" {
        json!({"kind": kind, "schema_version": version, "draft_clauses": clauses})
    } else {
        json!({"kind": kind, "schema_version": version, "mapping_candidates": clauses})
    };
    let mut bytes = serde_json::to_vec_pretty(
        &json!({"schema_version": "forge.suggest-response/1", "task": task}),
    )
    .unwrap();
    bytes.push(b'\n');
    bytes
}

/// Prepare, consent and run one recorded response, returning the request path.
fn pipeline(root: &Path) -> PathBuf {
    let adapter = root.join("local-adapter");
    std::fs::write(&adapter, b"#!/bin/sh\ncat\n").unwrap();
    corpus(root, BODY);
    let prepared = run(
        root,
        &[
            "suggest",
            "prepare",
            "--manifest",
            "project.json",
            "--output-dir",
            "prepared",
            "--adapter",
            adapter.to_string_lossy().as_ref(),
            "--model-id",
            "synthetic-model",
            "--corpus",
            "corpus.json",
            "--include-document",
            "prior-access",
            "--consent",
            "--operator-key",
            "human",
        ],
    );
    expect(&prepared, 0, "prepare");
    root.join("prepared/request.json")
}

fn record(root: &Path, run_dir: &str, body: &[u8]) -> Output {
    std::fs::write(root.join("recorded.json"), body).unwrap();
    let output = run(
        root,
        &[
            "suggest",
            "run",
            "--request",
            "prepared/request.json",
            "--consent",
            "prepared/consent.json",
            "--output-dir",
            run_dir,
            "--recorded-response",
            "recorded.json",
        ],
    );
    expect(&output, 0, "run");
    output
}

fn validate(root: &Path, run_record: &str, bundle_dir: &str, extra: &[&str]) -> Output {
    let mut args = vec![
        "suggest",
        "validate",
        "--request",
        "prepared/request.json",
        "--run",
        run_record,
        "--output-dir",
        bundle_dir,
    ];
    args.extend_from_slice(extra);
    run(root, &args)
}

#[test]
fn a_grounded_response_becomes_a_quarantine_bundle_with_rated_evidence() {
    let (_temp, root) = project();
    pipeline(&root);
    record(
        &root,
        "run-1",
        &response(
            &[clause("unit-0002", Some(BODY)), clause("unit-0001", None)],
            "policy-drafting",
            "forge.suggest-task-drafting/1",
        ),
    );

    let output = validate(&root, "prepared/run-1/run.json", "bundle-1", &["--format", "json"]);
    assert_exit(&output, 0);
    let bundle: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(bundle["schema_version"], json!("forge.suggestions/1"));
    assert_eq!(bundle["project_key"], json!("synthetic-project"));
    assert_eq!(bundle["as_of"], json!("2026-09-08T00:00:00Z"));
    assert_eq!(bundle["task"]["kind"], json!("policy-drafting"));
    assert_eq!(bundle["counts"]["suggestions"], json!(2));
    assert_eq!(bundle["counts"]["drafting"], json!(2));
    assert_eq!(bundle["counts"]["high"], json!(1));
    assert_eq!(bundle["counts"]["medium"], json!(0));
    assert_eq!(bundle["counts"]["low"], json!(1));
    assert_eq!(bundle["suggestions"][0]["evidence_support"], json!("high"));
    assert_eq!(bundle["suggestions"][1]["evidence_support"], json!("low"));
    assert_eq!(bundle["response"]["retained"], json!(false));
    assert_eq!(
        bundle["response"]["bytes"],
        json!(
            response(
                &[clause("unit-0002", Some(BODY)), clause("unit-0001", None)],
                "policy-drafting",
                "forge.suggest-task-drafting/1",
            )
            .len()
        )
    );
    assert_eq!(bundle["provenance"]["model_id"], json!("synthetic-model"));
    assert_eq!(bundle["provenance"]["elapsed_ms"], json!(0));

    // Identifier stability: the same content always receives the same id.
    let second = validate(&root, "prepared/run-1/run.json", "bundle-2", &["--format", "json"]);
    assert_exit(&second, 0);
    let again: Value = serde_json::from_slice(&second.stdout).unwrap();
    assert_eq!(again["suggestions"][0]["suggestion_id"], bundle["suggestions"][0]["suggestion_id"]);
    assert_eq!(again["bundle_id"], bundle["bundle_id"]);

    // Only the bundle is published unless raw retention is requested.
    let mut names: Vec<String> = std::fs::read_dir(root.join("prepared/bundle-1"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(names, vec!["suggestions.json"]);
}

#[test]
fn raw_retention_is_opt_in_and_reproduces_the_exact_response() {
    let (_temp, root) = project();
    pipeline(&root);
    let body =
        response(&[clause("unit-0002", None)], "policy-drafting", "forge.suggest-task-drafting/1");
    record(&root, "run-1", &body);
    let output = validate(
        &root,
        "prepared/run-1/run.json",
        "bundle-1",
        &["--retain-raw", "--format", "json"],
    );
    assert_exit(&output, 0);
    let bundle: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(bundle["response"]["retained"], json!(true));
    assert_eq!(bundle["response"]["artifact"], json!("response.raw"));
    assert_eq!(std::fs::read(root.join("prepared/bundle-1/response.raw")).unwrap(), body);
    assert!(
        forge::suggest::SuggestionsBundle::parse(
            &std::fs::read(root.join("prepared/bundle-1/suggestions.json")).unwrap()
        )
        .is_ok()
    );
}

#[test]
fn ungrounded_altered_and_unallowlisted_citations_are_refused_without_output() {
    let (_temp, root) = project();
    pipeline(&root);

    record(
        &root,
        "run-1",
        &response(
            &[clause("unit-0002", Some("not the supplied bytes"))],
            "policy-drafting",
            "forge.suggest-task-drafting/1",
        ),
    );
    let altered = validate(&root, "prepared/run-1/run.json", "bundle-1", &[]);
    assert_exit(&altered, 2);
    let stderr = String::from_utf8_lossy(&altered.stderr).into_owned();
    assert!(stderr.contains("not the supplied text"), "{stderr}");
    assert!(!root.join("prepared/bundle-1").exists());

    record(
        &root,
        "run-2",
        &response(&[clause("unit-9999", None)], "policy-drafting", "forge.suggest-task-drafting/1"),
    );
    let unknown = validate(&root, "prepared/run-2/run.json", "bundle-1", &[]);
    assert_exit(&unknown, 2);
    let stderr = String::from_utf8_lossy(&unknown.stderr).into_owned();
    assert!(stderr.contains("not in the request allowlist"), "{stderr}");
    assert!(!root.join("prepared/bundle-1").exists());
}

#[test]
fn a_response_for_another_task_or_an_unknown_shape_is_refused() {
    let (_temp, root) = project();
    pipeline(&root);

    // A contract-valid mapping response answers the wrong task.
    record(
        &root,
        "run-1",
        &response(&[candidate()], "mapping-candidates", "forge.suggest-task-mapping/1"),
    );
    let foreign = validate(&root, "prepared/run-1/run.json", "bundle-1", &[]);
    assert_exit(&foreign, 2);
    let stderr = String::from_utf8_lossy(&foreign.stderr).into_owned();
    assert!(stderr.contains("different task"), "{stderr}");
    assert!(!root.join("prepared/bundle-1").exists());

    // Shapes that do not belong to the declared task are refused at decode.
    record(
        &root,
        "run-2",
        &response(
            &[clause("unit-0001", None)],
            "mapping-candidates",
            "forge.suggest-task-mapping/1",
        ),
    );
    let mis_shaped = validate(&root, "prepared/run-2/run.json", "bundle-1", &[]);
    assert_exit(&mis_shaped, 2);
    let stderr = String::from_utf8_lossy(&mis_shaped.stderr).into_owned();
    assert!(stderr.contains("invalid suggest response contract"), "{stderr}");

    // Trailing bytes after the document are refused rather than ignored.
    let mut trailing_bytes =
        response(&[clause("unit-0001", None)], "policy-drafting", "forge.suggest-task-drafting/1");
    trailing_bytes.extend_from_slice(b"{}");
    record(&root, "run-3", &trailing_bytes);
    let trailing = validate(&root, "prepared/run-3/run.json", "bundle-1", &[]);
    assert_exit(&trailing, 2);
    assert!(!root.join("prepared/bundle-1").exists());
}

#[test]
fn a_response_that_carries_a_tool_call_or_an_omitted_field_is_refused() {
    let (_temp, root) = project();
    pipeline(&root);

    let mut with_tool = serde_json::from_slice::<Value>(&response(
        &[clause("unit-0001", None)],
        "policy-drafting",
        "forge.suggest-task-drafting/1",
    ))
    .unwrap();
    with_tool["tool_calls"] = json!([{"name": "read_file", "args": {"path": "/etc/passwd"}}]);
    record(&root, "run-1", &serde_json::to_vec_pretty(&with_tool).unwrap());
    let tool = validate(&root, "prepared/run-1/run.json", "bundle-1", &[]);
    assert_exit(&tool, 2);

    let mut omitted = serde_json::from_slice::<Value>(&response(
        &[clause("unit-0001", None)],
        "policy-drafting",
        "forge.suggest-task-drafting/1",
    ))
    .unwrap();
    omitted["task"]["draft_clauses"][0].as_object_mut().unwrap().remove("assumptions");
    record(&root, "run-2", &serde_json::to_vec_pretty(&omitted).unwrap());
    let missing = validate(&root, "prepared/run-2/run.json", "bundle-1", &[]);
    assert_exit(&missing, 2);
    assert!(!root.join("prepared/bundle-1").exists());
}

#[test]
fn a_tampered_run_record_is_refused() {
    let (_temp, root) = project();
    pipeline(&root);
    record(
        &root,
        "run-1",
        &response(&[clause("unit-0001", None)], "policy-drafting", "forge.suggest-task-drafting/1"),
    );
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(root.join("prepared/run-1/run.json")).unwrap())
            .unwrap();
    manifest["response_sha256"] = json!("b".repeat(64));
    std::fs::write(
        root.join("prepared/run-1/run.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
    let output = validate(&root, "prepared/run-1/run.json", "bundle-1", &[]);
    assert_exit(&output, 2);
    assert!(String::from_utf8_lossy(&output.stderr).contains("does not match"));
    assert!(!root.join("prepared/bundle-1").exists());
}

#[test]
fn an_empty_response_is_a_bundle_with_nothing_to_review() {
    let (_temp, root) = project();
    pipeline(&root);
    record(&root, "run-1", &response(&[], "policy-drafting", "forge.suggest-task-drafting/1"));
    let output = validate(&root, "prepared/run-1/run.json", "bundle-1", &["--format", "json"]);
    assert_exit(&output, 1);
    let bundle: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(bundle["counts"]["suggestions"], json!(0));
    assert!(root.join("prepared/bundle-1/suggestions.json").exists());
}

#[test]
fn repeated_validation_of_copied_inputs_is_byte_identical() {
    let (_temp, root) = project();
    pipeline(&root);
    record(
        &root,
        "run-1",
        &response(
            &[clause("unit-0002", Some(BODY)), clause("unit-0001", None)],
            "policy-drafting",
            "forge.suggest-task-drafting/1",
        ),
    );
    assert_exit(&validate(&root, "prepared/run-1/run.json", "bundle-1", &[]), 0);

    let (_second_temp, copy) = project();
    std::fs::create_dir_all(copy.join("prepared")).unwrap();
    std::fs::create_dir_all(copy.join("prepared/run-1")).unwrap();
    for (from, to) in [
        ("prepared/request.json", "prepared/request.json"),
        ("prepared/payload.txt", "prepared/payload.txt"),
        ("prepared/run-1/run.json", "prepared/run-1/run.json"),
        ("prepared/run-1/response.raw", "prepared/run-1/response.raw"),
    ] {
        std::fs::copy(root.join(from), copy.join(to)).unwrap();
    }
    assert_exit(&validate(&copy, "prepared/run-1/run.json", "bundle-1", &[]), 0);

    let first = std::fs::read(root.join("prepared/bundle-1/suggestions.json")).unwrap();
    let second = std::fs::read(copy.join("prepared/bundle-1/suggestions.json")).unwrap();
    assert_eq!(first, second);
    assert_eq!(hash(&first), hash(&second));
}
