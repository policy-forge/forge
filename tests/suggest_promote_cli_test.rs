//! `forge suggest promote` end-to-end contracts: a proposal that passes the
//! destination's own validator, stays unapproved, and never writes into it.

use std::path::Path;

use serde_json::{Value, json};

mod common;
use common::sha256_hex as hash;
use common::{
    SUGGEST_BODY as BODY, assert_exit, project, run, suggest_clause as clause,
    suggest_disposition as record, suggest_response_json as response,
};

const TASK_VERSION: &str = "forge.suggest-task-drafting/1";

/// Prepare, run, validate and review one bundle; returns its reviewed JSON.
fn reviewed(root: &Path, decisions: &[(usize, &str)]) -> Value {
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
    assert_exit(&common::suggest_validate(root, "prepared/run-1/run.json", "bundle-1", &[]), 0);
    let bundle: Value = serde_json::from_slice(
        &std::fs::read(root.join("prepared/bundle-1/suggestions.json")).unwrap(),
    )
    .unwrap();
    let records: Vec<Value> =
        decisions.iter().map(|(index, status)| record(&bundle, *index, status)).collect();
    let reviewed = common::suggest_review(root, &records, "review-1");
    assert_exit(&reviewed, 0);
    bundle
}

fn promote(root: &Path, destination: &str, extra: &[&str]) -> std::process::Output {
    let mut args = vec![
        "suggest",
        "promote",
        "--bundle",
        "prepared/bundle-1/suggestions.json",
        "--dispositions",
        "prepared/bundle-1/review-1/dispositions.json",
        "--request",
        "prepared/request.json",
        "--destination",
        destination,
        "--output-dir",
        "promotion-1",
    ];
    args.extend_from_slice(extra);
    run(root, &args)
}

#[test]
fn an_accepted_clause_becomes_a_proposal_validated_by_the_destination() {
    let (_temp, root) = project();
    let destination_before = std::fs::read(root.join("project.json")).unwrap();
    reviewed(&root, &[(0, "accept-as-is"), (1, "reject")]);

    let output = promote(&root, "project.json", &["--format", "json"]);
    assert_exit(&output, 0);
    let proposal: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(proposal["schema_version"], json!("forge.suggest-promotion/1"));
    assert_eq!(proposal["status"], json!("proposed-unapproved"));
    assert_eq!(proposal["destination"]["kind"], json!("authoring-project"));
    assert_eq!(proposal["destination"]["path"], json!("project.json"));
    assert_eq!(proposal["destination"]["expected_sha256"], json!(hash(&destination_before)));
    assert_eq!(proposal["entries"].as_array().unwrap().len(), 1);
    assert_eq!(proposal["entries"][0]["contract"], json!("forge.author-project/1"));
    assert_eq!(proposal["entries"][0]["artifact"], json!("promotion/entry-0001.md"));
    assert_eq!(proposal["patch"]["artifact"], json!("promotion/proposed-project.json"));

    // The proposed clause file is the quarantined text plus one newline.
    let clause_file = std::fs::read(root.join("promotion-1/promotion/entry-0001.md")).unwrap();
    assert_eq!(clause_file, b"Access requests are approved quarterly.\n");
    assert_eq!(proposal["entries"][0]["sha256"], json!(hash(&clause_file)));

    // The proposed document passes the destination's own validator, and it
    // carries one more clause than the destination does.
    let patch = std::fs::read(root.join("promotion-1/promotion/proposed-project.json")).unwrap();
    let parsed = forge::authoring::manifest::parse_project(&patch)
        .expect("the proposed project passes its own contract");
    let destination = forge::authoring::manifest::parse_project(&destination_before).unwrap();
    assert_eq!(parsed.human_clauses.len(), destination.human_clauses.len() + 1);
    assert_eq!(parsed.human_clauses.last().unwrap().source.expected_sha256, hash(&clause_file));
    assert_eq!(parsed.human_clauses.last().unwrap().review.reviewer_key, "human");
    assert_eq!(proposal["patch"]["sha256"], json!(hash(&patch)));

    // The proposal is published beside the destination, and the destination is untouched.
    assert_eq!(std::fs::read(root.join("project.json")).unwrap(), destination_before);
    let mut names: Vec<String> = std::fs::read_dir(root.join("promotion-1"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(names, vec!["promotion", "promotion.json"]);
    assert!(
        forge::suggest::PromotionProposal::parse(
            &std::fs::read(root.join("promotion-1/promotion.json")).unwrap()
        )
        .is_ok()
    );
}

#[test]
fn an_edited_acceptance_proposes_the_reviewers_text() {
    let (_temp, root) = project();
    let bundle = reviewed(&root, &[(0, "accept-as-is"), (1, "reject")]);

    // Re-review with an edit, then promote again into a fresh generation.
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
    let reviewed = common::suggest_review(&root, &[edited], "review-2");
    assert_exit(&reviewed, 1);

    let output = run(
        &root,
        &[
            "suggest",
            "promote",
            "--bundle",
            "prepared/bundle-1/suggestions.json",
            "--dispositions",
            "prepared/bundle-1/review-2/dispositions.json",
            "--request",
            "prepared/request.json",
            "--destination",
            "project.json",
            "--output-dir",
            "promotion-2",
            "--format",
            "json",
        ],
    );
    assert_exit(&output, 0);
    assert_eq!(
        std::fs::read(root.join("promotion-2/promotion/entry-0001.md")).unwrap(),
        b"Access requests are approved quarterly by the account owner.\n"
    );
}

#[test]
fn nothing_accepted_is_action_required_and_publishes_nothing() {
    let (_temp, root) = project();
    reviewed(&root, &[(0, "reject"), (1, "expired")]);
    let output = promote(&root, "project.json", &[]);
    assert_exit(&output, 1);
    assert!(String::from_utf8_lossy(&output.stdout).contains("nothing to propose"));
    assert!(!root.join("promotion-1").exists());
}

#[test]
fn a_destination_that_is_not_the_accepted_one_or_not_a_project_is_refused() {
    let (_temp, root) = project();
    reviewed(&root, &[(0, "accept-as-is"), (1, "reject")]);

    // An absolute destination is not a portable relative path.
    let absolute = root.join("project.json");
    let escaped = promote(&root, absolute.to_string_lossy().as_ref(), &[]);
    assert_exit(&escaped, 2);

    // A document that is not a valid forge.author-project/1 is refused by its own validator.
    std::fs::write(root.join("broken.json"), b"{\"schema_version\":\"forge.author-project/1\"}")
        .unwrap();
    let broken = promote(&root, "broken.json", &[]);
    assert_exit(&broken, 2);
    let stderr = String::from_utf8_lossy(&broken.stderr).into_owned();
    assert!(stderr.contains("forge.author-project/1"), "{stderr}");
    assert!(!root.join("promotion-1").exists());
}

#[test]
fn decisions_for_another_bundle_are_refused() {
    let (_temp, root) = project();
    reviewed(&root, &[(0, "accept-as-is"), (1, "reject")]);
    let mut decisions: Value = serde_json::from_slice(
        &std::fs::read(root.join("prepared/bundle-1/review-1/dispositions.json")).unwrap(),
    )
    .unwrap();
    decisions["bundle_sha256"] = json!("b".repeat(64));
    std::fs::write(
        root.join("prepared/bundle-1/review-1/dispositions.json"),
        serde_json::to_vec_pretty(&decisions).unwrap(),
    )
    .unwrap();
    let output = promote(&root, "project.json", &[]);
    assert_exit(&output, 2);
    assert!(String::from_utf8_lossy(&output.stderr).contains("digest of exactly this bundle"));
    assert!(!root.join("promotion-1").exists());
}

#[test]
fn a_published_promotion_generation_is_never_replaced() {
    let (_temp, root) = project();
    reviewed(&root, &[(0, "accept-as-is"), (1, "reject")]);
    assert_exit(&promote(&root, "project.json", &[]), 0);
    let again = promote(&root, "project.json", &[]);
    assert_exit(&again, 2);
    assert!(String::from_utf8_lossy(&again.stderr).contains("already exists"));
}
