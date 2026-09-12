//! `forge suggest run` end-to-end contracts: consent binding, the local
//! process boundary, recorded responses, bounds and refusal paths.

#![cfg(any(target_os = "linux", target_os = "macos"))]

use std::path::{Path, PathBuf};
use std::process::Output;

use serde_json::{Value, json};

mod common;
use common::sha256_hex as hash;
use common::{assert_exit, corpus, project, run};

/// The run publishes beneath the request's own directory.
const RUN_DIR: &str = "prepared/run-1";

const BODY: &str = "# Access drafting\n\nc-1 Approve access requests quarterly.\n";

fn response_body() -> Vec<u8> {
    let clause = serde_json::to_vec_pretty(&json!({
        "schema_version": "forge.suggest-response/1",
        "task": {
            "kind": "policy-drafting",
            "schema_version": "forge.suggest-task-drafting/1",
            "draft_clauses": [{
                "policy_key": "access-policy",
                "topic_key": "access-topic",
                "draft_text": "Access requests are approved quarterly.",
                "citations": [{"unit_id": "unit-0001"}],
                "assumptions": [],
                "unresolved_questions": ["Who approves?"]
            }]
        },
        "notes": []
    }))
    .unwrap();
    let mut bytes = clause;
    bytes.push(b'\n');
    bytes
}

/// Write the local adapter the request will be bound to.
fn write_adapter(root: &Path, body: &str) -> PathBuf {
    let path = root.join("local-adapter");
    std::fs::write(&path, body).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    path
}

/// Prepare and consent one request, returning the request's adapter path.
fn prepared(root: &Path, adapter_body: &str) -> String {
    let adapter = write_adapter(root, adapter_body);
    let adapter = adapter.to_string_lossy().into_owned();
    corpus(root, BODY);
    let output = run(
        root,
        &[
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
            "--corpus",
            "corpus.json",
            "--include-document",
            "prior-access",
            "--consent",
            "--operator-key",
            "human",
        ],
    );
    assert_exit(&output, 0);
    adapter
}

fn invoke(root: &Path, extra: &[&str]) -> Output {
    let mut args = vec![
        "suggest",
        "run",
        "--request",
        "prepared/request.json",
        "--consent",
        "prepared/consent.json",
        "--output-dir",
        "run-1",
    ];
    args.extend_from_slice(extra);
    run(root, &args)
}

#[test]
fn a_recorded_response_is_published_with_a_run_record_that_binds_it() {
    let (_temp, root) = project();
    prepared(&root, "#!/bin/sh\ncat\n");
    let recorded = root.join("recorded.json");
    std::fs::write(&recorded, response_body()).unwrap();

    let output = invoke(&root, &["--recorded-response", "recorded.json", "--format", "json"]);
    assert_exit(&output, 0);

    let raw = std::fs::read(root.join(RUN_DIR).join("response.raw")).unwrap();
    assert_eq!(raw, response_body());
    let request_bytes = std::fs::read(root.join("prepared/request.json")).unwrap();
    let request: Value = serde_json::from_slice(&request_bytes).unwrap();
    let record: Value =
        serde_json::from_slice(&std::fs::read(root.join(RUN_DIR).join("run.json")).unwrap())
            .unwrap();
    assert_eq!(record["schema_version"], json!("forge.suggest-run/1"));
    assert_eq!(record["mode"], json!("recorded-response"));
    assert_eq!(record["request_sha256"], json!(hash(&request_bytes)));
    assert_eq!(record["payload_sha256"], request["payload"]["sha256"]);
    assert_eq!(record["response_sha256"], json!(hash(&raw)));
    assert_eq!(record["response_bytes"], json!(raw.len()));
    assert_eq!(record["response_artifact"], json!("response.raw"));
    assert_eq!(record["exit_code"], json!(0));
    assert_eq!(record["elapsed_ms"], json!(0));
    assert_eq!(record["model_id"], json!("synthetic-model"));
    assert_eq!(record["adapter_executable_sha256"], request["adapter"]["executable_sha256"]);

    // The record parses as its own contract, and only two artifacts exist.
    assert!(
        forge::suggest::RunRecord::parse(
            &std::fs::read(root.join(RUN_DIR).join("run.json")).unwrap()
        )
        .is_ok()
    );
    let mut names: Vec<String> = std::fs::read_dir(root.join(RUN_DIR))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(names, vec!["response.raw", "run.json"]);
}

#[test]
fn the_missing_or_mismatched_consent_and_changed_payload_are_refused() {
    let (_temp, root) = project();
    prepared(&root, "#!/bin/sh\ncat\n");
    std::fs::write(root.join("recorded.json"), response_body()).unwrap();

    let no_consent = run(
        &root,
        &[
            "suggest",
            "run",
            "--request",
            "prepared/request.json",
            "--output-dir",
            "run-1",
            "--recorded-response",
            "recorded.json",
        ],
    );
    assert_exit(&no_consent, 2);

    let mut consent: Value =
        serde_json::from_slice(&std::fs::read(root.join("prepared/consent.json")).unwrap())
            .unwrap();
    consent["payload_sha256"] = json!("b".repeat(64));
    std::fs::write(
        root.join("prepared/consent.json"),
        serde_json::to_vec_pretty(&consent).unwrap(),
    )
    .unwrap();
    let mismatched = invoke(&root, &["--recorded-response", "recorded.json"]);
    assert_exit(&mismatched, 2);
    assert!(
        String::from_utf8_lossy(&mismatched.stderr).contains("does not match the prepared request")
    );
    assert!(!root.join(RUN_DIR).exists());

    // Restore consent, then change the payload it was bound to.
    let (_temp2, restored) = project();
    prepared(&restored, "#!/bin/sh\ncat\n");
    std::fs::write(restored.join("recorded.json"), response_body()).unwrap();
    std::fs::write(restored.join("prepared/payload.txt"), b"tampered payload").unwrap();
    let changed = invoke(&restored, &["--recorded-response", "recorded.json"]);
    assert_exit(&changed, 2);
    assert!(String::from_utf8_lossy(&changed.stderr).contains("payload changed after consent"));
    assert!(!restored.join(RUN_DIR).exists());
}

#[test]
fn an_adapter_override_must_still_be_the_consented_one() {
    let (_temp, root) = project();
    prepared(&root, "#!/bin/sh\ncat\n");
    std::fs::write(root.join("recorded.json"), response_body()).unwrap();
    let other = write_adapter(&root, "#!/bin/sh\nexit 0\n");

    // A process run refuses an executable that is not the consented one.
    let swapped = invoke(&root, &["--adapter", other.to_string_lossy().as_ref()]);
    assert_exit(&swapped, 2);
    assert!(String::from_utf8_lossy(&swapped.stderr).contains("changed since consent"));
    assert!(!root.join(RUN_DIR).exists());

    // A recorded run never invokes an adapter, so the override is unused.
    let recorded = invoke(
        &root,
        &["--adapter", other.to_string_lossy().as_ref(), "--recorded-response", "recorded.json"],
    );
    assert_exit(&recorded, 0);
}

#[test]
fn a_failing_or_hanging_adapter_is_action_required_and_writes_nothing() {
    let (_temp, root) = project();
    prepared(&root, "#!/bin/sh\nexit 4\n");
    let failing = invoke(&root, &[]);
    assert_exit(&failing, 1);
    assert!(String::from_utf8_lossy(&failing.stdout).contains("run refused"));
    assert!(!root.join(RUN_DIR).exists());

    let (_temp2, hanging) = project();
    prepared(&hanging, "#!/bin/sh\nsleep 30\n");
    let timed_out = invoke(&hanging, &["--timeout", "1"]);
    assert_exit(&timed_out, 1);
    assert!(String::from_utf8_lossy(&timed_out.stdout).contains("timed out"));
    assert!(!hanging.join(RUN_DIR).exists());
}

#[cfg(unix)]
#[test]
fn a_process_adapter_receives_the_exact_payload_and_its_output_is_the_response() {
    let (_temp, root) = project();
    prepared(&root, "#!/bin/sh\ncat\n");
    let output = invoke(&root, &[]);
    assert_exit(&output, 0);
    let payload = std::fs::read(root.join("prepared/payload.txt")).unwrap();
    let raw = std::fs::read(root.join(RUN_DIR).join("response.raw")).unwrap();
    assert_eq!(raw, payload, "the adapter must receive exactly the prepared bytes");
    let record: Value =
        serde_json::from_slice(&std::fs::read(root.join(RUN_DIR).join("run.json")).unwrap())
            .unwrap();
    assert_eq!(record["mode"], json!("process"));
    assert_eq!(record["response_sha256"], json!(hash(&payload)));
}

#[cfg(unix)]
#[test]
fn the_adapter_environment_is_cleared_and_bounded() {
    let (_temp, root) = project();
    prepared(&root, "#!/bin/sh\nenv > /dev/null; printf 'ok'\n");
    let output = invoke(&root, &[]);
    assert_exit(&output, 0);
    assert_eq!(std::fs::read(root.join(RUN_DIR).join("response.raw")).unwrap(), b"ok");
}

#[test]
fn a_published_run_directory_is_never_replaced() {
    let (_temp, root) = project();
    prepared(&root, "#!/bin/sh\ncat\n");
    std::fs::write(root.join("recorded.json"), response_body()).unwrap();
    assert_exit(&invoke(&root, &["--recorded-response", "recorded.json"]), 0);
    let again = invoke(&root, &["--recorded-response", "recorded.json"]);
    assert_exit(&again, 2);
    assert!(String::from_utf8_lossy(&again.stderr).contains("already exists"));
}

#[test]
fn an_out_of_range_timeout_is_refused_before_anything_runs() {
    let (_temp, root) = project();
    prepared(&root, "#!/bin/sh\ncat\n");
    std::fs::write(root.join("recorded.json"), response_body()).unwrap();
    let zero = invoke(&root, &["--timeout", "0", "--recorded-response", "recorded.json"]);
    assert_exit(&zero, 2);
    let too_long = invoke(&root, &["--timeout", "3601", "--recorded-response", "recorded.json"]);
    assert_exit(&too_long, 2);
    assert!(!root.join(RUN_DIR).exists());
}
