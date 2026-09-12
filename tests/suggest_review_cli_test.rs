//! `forge suggest review` end-to-end contracts: decisions bound to one bundle,
//! the action-required exit code, and refusals that publish nothing.

use std::path::Path;

use serde_json::{Value, json};

mod common;
use common::sha256_hex as hash;
use common::{
    SUGGEST_BODY as BODY, assert_exit, project, run, suggest_clause as clause,
    suggest_disposition as record, suggest_response_json as response,
};

const TASK_VERSION: &str = "forge.suggest-task-drafting/1";

/// Prepare, run and validate one two-suggestion bundle; returns its JSON.
fn bundled(root: &Path) -> Value {
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
    let validated = common::suggest_validate(root, "prepared/run-1/run.json", "bundle-1", &[]);
    assert_exit(&validated, 0);
    serde_json::from_slice(&std::fs::read(root.join("prepared/bundle-1/suggestions.json")).unwrap())
        .unwrap()
}

/// The decisions an operator would write after reading the bundle.
fn decisions(root: &Path, bundle: &Value, records: &[Value]) -> Vec<u8> {
    let bytes = std::fs::read(root.join("prepared/bundle-1/suggestions.json")).unwrap();
    common::suggest_dispositions_json(bundle, &bytes, records)
}

fn review(root: &Path, extra: &[&str]) -> std::process::Output {
    let mut args = vec![
        "suggest",
        "review",
        "--bundle",
        "prepared/bundle-1/suggestions.json",
        "--decisions",
        "decisions.json",
        "--output-dir",
        "review-1",
    ];
    args.extend_from_slice(extra);
    run(root, &args)
}

#[test]
fn every_decision_is_bound_to_the_bundle_and_published_unchanged() {
    let (_temp, root) = project();
    let bundle = bundled(&root);
    let encoded = decisions(
        &root,
        &bundle,
        &[record(&bundle, 0, "accept-as-is"), record(&bundle, 1, "reject")],
    );
    std::fs::write(root.join("decisions.json"), &encoded).unwrap();

    let output = review(&root, &["--format", "json"]);
    assert_exit(&output, 0);
    let published: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(published["schema_version"], json!("forge.suggest-dispositions/1"));
    assert_eq!(published["bundle_id"], bundle["bundle_id"]);
    assert_eq!(published["records"][0]["status"], json!("accept-as-is"));
    assert_eq!(published["records"][1]["status"], json!("reject"));
    assert_eq!(
        published["records"][0]["original_sha256"],
        bundle["suggestions"][0]["content_sha256"]
    );

    let on_disk: Value = serde_json::from_slice(
        &std::fs::read(root.join("prepared/bundle-1/review-1/dispositions.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(on_disk, published);

    // The bundle itself is untouched, and the generation holds only the record.
    assert_eq!(
        hash(&std::fs::read(root.join("prepared/bundle-1/suggestions.json")).unwrap()),
        published["bundle_sha256"]
    );
    let mut names: Vec<String> = std::fs::read_dir(root.join("prepared/bundle-1/review-1"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(names, vec!["dispositions.json"]);
}

#[test]
fn an_undecided_suggestion_is_action_required() {
    let (_temp, root) = project();
    let bundle = bundled(&root);
    std::fs::write(
        root.join("decisions.json"),
        decisions(&root, &bundle, &[record(&bundle, 0, "accept-as-is")]),
    )
    .unwrap();
    let output = review(&root, &["--format", "json"]);
    assert_exit(&output, 1);
    let published: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(published["records"].as_array().unwrap().len(), 1);
    assert!(root.join("prepared/bundle-1/review-1/dispositions.json").exists());
}

#[test]
fn a_manifest_for_another_bundle_identifier_digest_or_task_is_refused() {
    let (_temp, root) = project();
    let bundle = bundled(&root);

    for (path, value) in [
        ("bundle_id", json!("1b2c3d4e-5f60-4718-9a2b-3c4d5e6f7081")),
        ("bundle_sha256", json!("b".repeat(64))),
        (
            "task",
            json!({"kind": "mapping-candidates", "schema_version": "forge.suggest-task-mapping/1"}),
        ),
    ] {
        let mut manifest: Value =
            serde_json::from_slice(&decisions(&root, &bundle, &[record(&bundle, 0, "reject")]))
                .unwrap();
        manifest[path] = value;
        std::fs::write(root.join("decisions.json"), serde_json::to_vec_pretty(&manifest).unwrap())
            .unwrap();
        let output = review(&root, &[]);
        assert_exit(&output, 2);
        assert!(!root.join("prepared/bundle-1/review-1").exists(), "{path}");
    }
}

#[test]
fn a_record_outside_the_bundle_or_with_another_content_digest_is_refused() {
    let (_temp, root) = project();
    let bundle = bundled(&root);

    let mut unknown = record(&bundle, 0, "reject");
    unknown["suggestion_id"] = json!("3f2b1a4c-5d6e-4f70-8a91-b2c3d4e5f607");
    std::fs::write(root.join("decisions.json"), decisions(&root, &bundle, &[unknown])).unwrap();
    let output = review(&root, &[]);
    assert_exit(&output, 2);
    assert!(String::from_utf8_lossy(&output.stderr).contains("not in this bundle"));

    let mut altered = record(&bundle, 0, "reject");
    altered["original_sha256"] = json!("c".repeat(64));
    std::fs::write(root.join("decisions.json"), decisions(&root, &bundle, &[altered])).unwrap();
    let output = review(&root, &[]);
    assert_exit(&output, 2);
    assert!(String::from_utf8_lossy(&output.stderr).contains("quarantined content digest"));
    assert!(!root.join("prepared/bundle-1/review-1").exists());
}

#[test]
fn an_edited_acceptance_keeps_both_the_original_and_the_edit() {
    let (_temp, root) = project();
    let bundle = bundled(&root);
    let mut edited = record(&bundle, 0, "accept-edited");
    let body = json!({
        "drafting": {
            "policy_key": "access-policy",
            "topic_key": "access-topic",
            "draft_text": "Access requests are approved quarterly by the account owner.",
            "citations": [{"unit_id": "unit-0002", "quote": BODY}],
            "assumptions": [],
            "unresolved_questions": []
        }
    });
    edited["edited_sha256"] = json!(hash(serde_json::to_vec(&body).unwrap().as_slice()));
    edited["edited"] = body;
    std::fs::write(
        root.join("decisions.json"),
        decisions(&root, &bundle, &[edited, record(&bundle, 1, "expired")]),
    )
    .unwrap();

    let output = review(&root, &["--format", "json"]);
    assert_exit(&output, 0);
    let published: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(published["records"][0]["status"], json!("accept-edited"));
    assert_eq!(
        published["records"][0]["original_sha256"],
        bundle["suggestions"][0]["content_sha256"]
    );
    assert_eq!(
        published["records"][0]["edited"]["drafting"]["draft_text"],
        json!("Access requests are approved quarterly by the account owner.")
    );
    assert_eq!(published["records"][1]["status"], json!("expired"));
}

#[test]
fn a_decision_for_the_other_task_shape_is_refused_by_the_contract() {
    let (_temp, root) = project();
    let bundle = bundled(&root);
    let mut foreign = record(&bundle, 0, "accept-edited");
    foreign["edited"] = json!({"mapping": {"policy_key": "access-policy"}});
    foreign["edited_sha256"] = json!("d".repeat(64));
    std::fs::write(root.join("decisions.json"), decisions(&root, &bundle, &[foreign])).unwrap();
    let output = review(&root, &[]);
    assert_exit(&output, 2);
    assert!(!root.join("prepared/bundle-1/review-1").exists());
}

#[test]
fn a_published_review_generation_is_never_replaced() {
    let (_temp, root) = project();
    let bundle = bundled(&root);
    std::fs::write(
        root.join("decisions.json"),
        decisions(&root, &bundle, &[record(&bundle, 0, "reject"), record(&bundle, 1, "reject")]),
    )
    .unwrap();
    assert_exit(&review(&root, &[]), 0);
    let again = review(&root, &[]);
    assert_exit(&again, 2);
    assert!(String::from_utf8_lossy(&again.stderr).contains("already exists"));
}
