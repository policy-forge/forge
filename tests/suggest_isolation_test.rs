//! Quarantine isolation and provenance: the pipeline may add generations, but
//! it must never touch the operator's own inputs, and the record it leaves must
//! carry what the run actually measured.

#![cfg(any(target_os = "linux", target_os = "macos"))]

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::{Value, json};

mod common;
use common::sha256_hex as hash;
use common::{
    SUGGEST_BODY as BODY, assert_exit, project, run, suggest_clause as clause,
    suggest_disposition as record, suggest_response_json as response,
};

const TASK_VERSION: &str = "forge.suggest-task-drafting/1";

/// The files the operator supplied; none of them may change.
const SUPPLIED: [&str; 8] = [
    "framework.json",
    "applicability.json",
    "gap-report.json",
    "pack.json",
    "project.json",
    "clause.md",
    "corpus.json",
    "prior/access.md",
];

fn digest(path: &Path) -> String {
    hash(&std::fs::read(path).unwrap())
}

fn supplied_digests(root: &Path) -> BTreeMap<&'static str, String> {
    SUPPLIED.iter().map(|name| (*name, digest(&root.join(name)))).collect()
}

fn top_level(root: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// Run the whole pipeline once and return the reviewed bundle.
fn pipeline(root: &Path) -> (Value, Value) {
    common::suggest_prepare(root);
    common::suggest_run(
        root,
        "run-1",
        &response(
            &[clause("unit-0002", Some(BODY)), clause("unit-0001", None)],
            "policy-drafting",
            TASK_VERSION,
        ),
    );
    assert_exit(
        &common::suggest_validate(root, "prepared/run-1/run.json", "bundle-1", &["--retain-raw"]),
        0,
    );
    let bundle: Value = serde_json::from_slice(
        &std::fs::read(root.join("prepared/bundle-1/suggestions.json")).unwrap(),
    )
    .unwrap();
    let reviewed = common::suggest_review(root, &[record(&bundle, 0, "accept-as-is")], "review-1");
    assert_exit(&reviewed, 1);
    let promoted = run(
        root,
        &[
            "suggest",
            "promote",
            "--bundle",
            "prepared/bundle-1/suggestions.json",
            "--request",
            "prepared/request.json",
            "--dispositions",
            "prepared/bundle-1/review-1/dispositions.json",
            "--destination",
            "project.json",
            "--output-dir",
            "promotion-1",
        ],
    );
    assert_exit(&promoted, 0);
    let proposal: Value =
        serde_json::from_slice(&std::fs::read(root.join("promotion-1/promotion.json")).unwrap())
            .unwrap();
    (bundle, proposal)
}

#[test]
fn the_pipeline_never_touches_the_supplied_inputs() {
    let (_temp, root) = project();
    // The corpus and adapter are operator inputs too; write them before the snapshot.
    common::corpus(&root, BODY);
    common::suggest_adapter(&root);
    let before = supplied_digests(&root);
    let (_, proposal) = pipeline(&root);
    let after = supplied_digests(&root);
    assert_eq!(before, after, "a supplied input changed during the pipeline");

    // Everything new is inside a generation the command published.
    let mut expected = vec![
        "applicability.json",
        "clause.md",
        "corpus.json",
        "decisions.json",
        "framework.json",
        "gap-report.json",
        "local-adapter",
        "pack.json",
        "prepared",
        "prior",
        "project.json",
        "promotion-1",
        "recorded.json",
    ];
    expected.sort_unstable();
    assert_eq!(top_level(&root), expected);
    assert_eq!(proposal["status"], json!("proposed-unapproved"));
}

#[test]
fn the_bundle_carries_what_the_run_measured_and_the_run_carries_the_digests() {
    let (_temp, root) = project();
    let (bundle, proposal) = pipeline(&root);

    let record: Value =
        serde_json::from_slice(&std::fs::read(root.join("prepared/run-1/run.json")).unwrap())
            .unwrap();
    let request_bytes = std::fs::read(root.join("prepared/request.json")).unwrap();
    let response_bytes = std::fs::read(root.join("prepared/run-1/response.raw")).unwrap();
    let request: Value = serde_json::from_slice(&request_bytes).unwrap();

    // The run record binds the request, payload and response it measured.
    assert_eq!(record["request_sha256"], json!(hash(&request_bytes)));
    assert_eq!(record["payload_sha256"], request["payload"]["sha256"]);
    assert_eq!(record["response_sha256"], json!(hash(&response_bytes)));
    assert_eq!(record["response_bytes"], json!(response_bytes.len()));
    assert_eq!(record["mode"], json!("recorded-response"));
    assert_eq!(record["exit_code"], json!(0));

    // The bundle carries every field the provenance requirement names, and the
    // values are the run record's own.
    let provenance = &bundle["provenance"];
    for key in [
        "run_record_sha256",
        "mode",
        "adapter_sha256",
        "model_id",
        "elapsed_ms",
        "exit_code",
        "redactions",
    ] {
        assert!(!provenance[key].is_null(), "provenance.{key} is missing");
    }
    // A recorded response is not an adapter run, and the bundle says so.
    assert_eq!(provenance["mode"], json!("recorded-response"));
    assert_eq!(
        provenance["run_record_sha256"],
        json!(hash(&std::fs::read(root.join("prepared/run-1/run.json")).unwrap()))
    );
    assert_eq!(provenance["adapter_sha256"], record["adapter_executable_sha256"]);
    assert_eq!(provenance["model_id"], record["model_id"]);
    // An empty argument list is an absent key, in the bundle and the record alike.
    assert_eq!(
        provenance.get("argv").cloned().unwrap_or_else(|| json!([])),
        record.get("argv").cloned().unwrap_or_else(|| json!([]))
    );
    assert_eq!(provenance["elapsed_ms"], record["elapsed_ms"]);
    assert_eq!(provenance["exit_code"], record["exit_code"]);
    assert_eq!(bundle["task"]["schema_version"], json!(TASK_VERSION));
    assert_eq!(bundle["request"]["payload_sha256"], request["payload"]["sha256"]);
    assert_eq!(bundle["response"]["retained"], json!(true));
    assert_eq!(bundle["response"]["artifact"], json!("response.raw"));
    assert_eq!(std::fs::read(root.join("prepared/bundle-1/response.raw")).unwrap(), response_bytes);

    // The proposal binds the bundle, the decisions and the patch it proposed.
    let dispositions =
        std::fs::read(root.join("prepared/bundle-1/review-1/dispositions.json")).unwrap();
    let patch = std::fs::read(root.join("promotion-1/promotion/proposed-project.json")).unwrap();
    assert_eq!(proposal["bundle_id"], bundle["bundle_id"]);
    assert_eq!(
        proposal["bundle_sha256"],
        json!(hash(&std::fs::read(root.join("prepared/bundle-1/suggestions.json")).unwrap()))
    );
    assert_eq!(proposal["dispositions_sha256"], json!(hash(&dispositions)));
    assert_eq!(proposal["patch"]["sha256"], json!(hash(&patch)));
    assert_eq!(
        proposal["destination"]["expected_sha256"],
        json!(hash(&std::fs::read(root.join("project.json")).unwrap()))
    );

    // No clock: the only timestamps are the supplied project ones.
    assert_eq!(bundle["as_of"], json!("2026-09-08T00:00:00Z"));
    let today = "2026-09-12";
    assert!(!bundle.to_string().contains(today), "the bundle must not read a clock");
}

#[test]
fn prepare_and_validate_refuse_an_existing_generation() {
    let (_temp, root) = project();
    common::suggest_prepare(&root);
    let payload = std::fs::read(root.join("prepared/payload.txt")).unwrap();
    let again = run(
        &root,
        &[
            "suggest",
            "prepare",
            "--manifest",
            "project.json",
            "--output-dir",
            "prepared",
            "--adapter",
            root.join("local-adapter").to_string_lossy().as_ref(),
            "--model-id",
            "synthetic-model",
            "--corpus",
            "corpus.json",
            "--include-document",
            "prior-access",
        ],
    );
    assert_exit(&again, 2);
    assert!(String::from_utf8_lossy(&again.stderr).contains("already exists"));
    assert_eq!(std::fs::read(root.join("prepared/payload.txt")).unwrap(), payload);

    common::suggest_run(
        &root,
        "run-1",
        &response(&[clause("unit-0001", None)], "policy-drafting", TASK_VERSION),
    );
    assert_exit(&common::suggest_validate(&root, "prepared/run-1/run.json", "bundle-1", &[]), 0);
    let twice = common::suggest_validate(&root, "prepared/run-1/run.json", "bundle-1", &[]);
    assert_exit(&twice, 2);
    assert!(String::from_utf8_lossy(&twice.stderr).contains("already exists"));
}
