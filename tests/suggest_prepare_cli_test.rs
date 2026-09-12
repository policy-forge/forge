//! `forge suggest prepare` end-to-end contracts: allowlist, exact payload,
//! preview, consent and refusal paths. Nothing here calls a model.

use std::path::Path;
use std::process::Output;

use serde_json::{Value, json};

mod common;
use common::sha256_hex as hash;
use common::{assert_exit, corpus, project, run};

const BODY: &str = "# Access drafting\n\nc-1 Approve access requests quarterly.\n";
const PROMPT: &str = "Which role writes this draft?";
const NOTICE: &str = "The payload below is the exact byte sequence this machine will hand to the local adapter. \
                      FORGE opens no network connection and transmits nothing.";

/// Write a local adapter stand-in and return its path as an argument string.
fn adapter(root: &Path) -> String {
    let path = root.join("local-adapter");
    std::fs::write(&path, b"#!/bin/sh\ncat\n").unwrap();
    path.to_string_lossy().into_owned()
}

fn prepare(root: &Path, extra: &[&str]) -> Output {
    let adapter = adapter(root);
    let mut args = vec![
        "suggest",
        "prepare",
        "--manifest",
        "project.json",
        "--output-dir",
        "prepared",
        "--adapter",
        adapter.as_str(),
        "--model-id",
        "synthetic-model",
    ];
    args.extend_from_slice(extra);
    run(root, &args)
}

fn prepare_json(root: &Path, extra: &[&str]) -> (Output, Value, String) {
    let output = prepare(root, extra);
    assert_exit(&output, 0);
    let request: Value =
        serde_json::from_slice(&std::fs::read(root.join("prepared/request.json")).unwrap())
            .unwrap();
    let payload = std::fs::read_to_string(root.join("prepared/payload.txt")).unwrap();
    (output, request, payload)
}

#[test]
fn prepare_writes_the_request_the_exact_payload_and_the_preview_without_a_token() {
    let (_temp, root) = project();
    let bytes = corpus(&root, BODY);
    let (output, request, payload) = prepare_json(
        &root,
        &["--corpus", "corpus.json", "--include-document", "prior-access", "--format", "json"],
    );

    assert_eq!(request["schema_version"], json!("forge.suggest-request/1"));
    assert_eq!(request["project_key"], json!("synthetic-project"));
    assert_eq!(request["as_of"], json!("2026-09-08T00:00:00Z"));
    assert_eq!(request["task"]["kind"], json!("policy-drafting"));
    assert_eq!(request["task"]["schema_version"], json!("forge.suggest-task-drafting/1"));
    assert_eq!(request["adapter"]["model_id"], json!("synthetic-model"));
    assert_eq!(
        request["retention_notice"],
        json!("No copy of this payload or its response leaves this machine.")
    );

    // The request the command published passes the crate's own contract.
    let parsed = forge::suggest::SuggestRequest::parse(
        &std::fs::read(root.join("prepared/request.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(parsed.payload.bytes, payload.len() as u64);
    assert_eq!(parsed.payload.sha256, hash(payload.as_bytes()));
    assert_eq!(request["payload"]["artifact"], json!("payload.txt"));

    // Units are ordered, non-overlapping and bounded by the payload.
    let units = request["context"]["units"].as_array().unwrap();
    assert_eq!(units.len(), 2, "{units:?}");
    let mut previous = 0_u64;
    for unit in units {
        let start = unit["payload"]["start"].as_u64().unwrap();
        let end = unit["payload"]["end"].as_u64().unwrap();
        assert!(start >= previous && end > start, "{unit}");
        assert!(end <= payload.len() as u64, "{unit}");
        previous = end;
    }

    // A source-span unit's bytes are the captured document, verbatim.
    let span_unit = &units[1];
    assert_eq!(span_unit["kind"], json!("source-span"));
    assert_eq!(span_unit["source"]["key"], json!("prior-access"));
    assert_eq!(span_unit["source"]["path"], json!("prior/access.md"));
    assert_eq!(span_unit["source"]["sha256"], json!(hash(&bytes)));
    let start = usize::try_from(span_unit["payload"]["start"].as_u64().unwrap()).unwrap();
    let end = usize::try_from(span_unit["payload"]["end"].as_u64().unwrap()).unwrap();
    assert_eq!(&payload[start..end], BODY);

    // The plan-section unit carries the supplied prompt text.
    let section_unit = &units[0];
    assert_eq!(section_unit["kind"], json!("plan-section"));
    let start = usize::try_from(section_unit["payload"]["start"].as_u64().unwrap()).unwrap();
    let end = usize::try_from(section_unit["payload"]["end"].as_u64().unwrap()).unwrap();
    assert!(payload[start..end].contains(PROMPT));

    // The preview shows the exact bytes and the handling notice; no token yet.
    let preview = std::fs::read_to_string(root.join("prepared/preview.txt")).unwrap();
    assert!(preview.contains(payload.as_str()), "preview must embed the payload verbatim");
    assert!(preview.contains(NOTICE));
    assert!(preview.contains("unit-0001 kind=plan-section sensitivity=internal"));
    assert!(!root.join("prepared/consent.json").exists());
    let stdout: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(stdout["schema_version"], json!("forge.suggest-request/1"));
    assert_eq!(stdout["payload"]["sha256"], json!(hash(payload.as_bytes())));

    // Nothing else was created beside the four known artifacts.
    let mut names: Vec<String> = std::fs::read_dir(root.join("prepared"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(names, vec!["payload.txt", "preview.txt", "request.json"]);
}

#[test]
fn consent_is_written_only_when_requested_and_binds_the_exact_payload() {
    let (_temp, root) = project();
    corpus(&root, BODY);
    let adapter_bytes = b"#!/bin/sh\ncat\n";
    let (_, request, payload) = prepare_json(
        &root,
        &[
            "--corpus",
            "corpus.json",
            "--include-document",
            "prior-access",
            "--consent",
            "--operator-key",
            "human",
        ],
    );
    let token: Value =
        serde_json::from_slice(&std::fs::read(root.join("prepared/consent.json")).unwrap())
            .unwrap();
    assert_eq!(token["schema_version"], json!("forge.suggest-consent/1"));
    assert_eq!(token["payload_sha256"], json!(hash(payload.as_bytes())));
    assert_eq!(token["adapter_sha256"], json!(hash(adapter_bytes)));
    assert_eq!(token["model_id"], json!("synthetic-model"));
    assert_eq!(token["operator_key"], json!("human"));
    assert_eq!(token["retention_notice"], request["retention_notice"]);
    assert_eq!(token["as_of"], json!("2026-09-08T00:00:00Z"));

    // The token authorises exactly this request and nothing else.
    let parsed = forge::suggest::ConsentToken::parse(
        &std::fs::read(root.join("prepared/consent.json")).unwrap(),
    )
    .unwrap();
    assert!(
        parsed
            .authorises(
                &forge::suggest::SuggestRequest::parse(
                    &std::fs::read(root.join("prepared/request.json")).unwrap()
                )
                .unwrap()
            )
            .is_ok()
    );
}

#[test]
fn consent_and_operator_key_are_mutually_required() {
    let (_temp, root) = project();
    corpus(&root, BODY);
    let missing_key = prepare(
        &root,
        &["--corpus", "corpus.json", "--include-document", "prior-access", "--consent"],
    );
    assert_exit(&missing_key, 2);
    assert!(String::from_utf8_lossy(&missing_key.stderr).contains("operator-key"));

    let key_without_consent = prepare(
        &root,
        &[
            "--corpus",
            "corpus.json",
            "--include-document",
            "prior-access",
            "--operator-key",
            "human",
        ],
    );
    assert_exit(&key_without_consent, 2);
}

#[test]
fn redaction_rewrites_the_payload_and_records_only_the_rule_identity() {
    let (_temp, root) = project();
    corpus(&root, "# Access drafting\n\nc-1 Approve requests for 123456789012.\n");
    std::fs::write(root.join("rules.txt"), b"account-id\t123456789012\n").unwrap();
    let (_, request, payload) = prepare_json(
        &root,
        &[
            "--corpus",
            "corpus.json",
            "--include-document",
            "prior-access",
            "--redact-rules",
            "rules.txt",
        ],
    );
    assert!(payload.contains("[redacted]"));
    assert!(!payload.contains("123456789012"));
    assert_eq!(request["redactions"].as_array().unwrap().len(), 1);
    assert_eq!(request["redactions"][0]["rule_id"], json!("account-id"));
    assert_eq!(request["redactions"][0]["unit_id"], json!("unit-0002"));
    assert_eq!(request["redactions"][0]["rule_sha256"], json!(hash(b"account-id\t123456789012")));
}

#[test]
fn unmatched_rules_secret_shaped_documents_and_bad_selection_all_fail_before_output() {
    let (_temp, root) = project();
    corpus(&root, BODY);
    std::fs::write(root.join("rules.txt"), b"absent\tnever-present-literal\n").unwrap();
    let unmatched = prepare(
        &root,
        &[
            "--corpus",
            "corpus.json",
            "--include-document",
            "prior-access",
            "--redact-rules",
            "rules.txt",
        ],
    );
    assert_exit(&unmatched, 2);
    assert!(String::from_utf8_lossy(&unmatched.stderr).contains("matched nothing"));
    assert!(!root.join("prepared").exists(), "a refused prepare must publish nothing");

    corpus(&root, "# Access drafting\n\ntoken sk-abcdefghijklmnopqrstuvwxyz\n");
    let secret = prepare(&root, &["--corpus", "corpus.json", "--include-document", "prior-access"]);
    assert_exit(&secret, 2);
    let stderr = String::from_utf8_lossy(&secret.stderr).into_owned();
    assert!(stderr.contains("secret pattern"), "{stderr}");
    assert!(!stderr.contains("sk-abcdefghijklmnopqrstuvwxyz"), "must not echo the match");
    assert!(!root.join("prepared").exists());

    let no_corpus = prepare(&root, &["--include-document", "prior-access"]);
    assert_exit(&no_corpus, 2);
    let empty_selection = prepare(&root, &["--corpus", "corpus.json"]);
    assert_exit(&empty_selection, 2);
    let unknown_document =
        prepare(&root, &["--corpus", "corpus.json", "--include-document", "absent-document"]);
    assert_exit(&unknown_document, 2);
    assert!(String::from_utf8_lossy(&unknown_document.stderr).contains("names no corpus document"));
    assert!(!root.join("prepared").exists());
}

#[test]
fn the_mapping_task_is_refused_until_the_owner_selects_it() {
    let (_temp, root) = project();
    corpus(&root, BODY);
    let output = prepare(
        &root,
        &["--task", "mapping", "--corpus", "corpus.json", "--include-document", "prior-access"],
    );
    assert_exit(&output, 2);
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(stderr.contains("mapping-candidate task"), "{stderr}");
    assert!(!root.join("prepared").exists());
}

#[test]
fn repeated_prepares_produce_byte_identical_payloads_across_directories() {
    let (_first_temp, first) = project();
    corpus(&first, BODY);
    let (_, _, first_payload) =
        prepare_json(&first, &["--corpus", "corpus.json", "--include-document", "prior-access"]);

    let (_second_temp, second) = project();
    corpus(&second, BODY);
    let (_, _, second_payload) =
        prepare_json(&second, &["--corpus", "corpus.json", "--include-document", "prior-access"]);

    assert_eq!(first_payload, second_payload);
    assert_eq!(hash(first_payload.as_bytes()), hash(second_payload.as_bytes()));
}

#[test]
fn the_payload_carries_no_absolute_path_and_the_request_carries_no_system_clock() {
    let (_temp, root) = project();
    corpus(&root, BODY);
    let (_, request, payload) =
        prepare_json(&root, &["--corpus", "corpus.json", "--include-document", "prior-access"]);
    let root_text = root.to_string_lossy().into_owned();
    assert!(!payload.contains(&root_text), "the payload must not name a local path");
    assert_eq!(request["as_of"], json!("2026-09-08T00:00:00Z"));
    let today = "2026-09-12";
    assert!(!request.to_string().contains(today), "no field may read the system clock: {request}");
}
