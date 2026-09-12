#![cfg(any(target_os = "linux", target_os = "macos"))]

mod common;
use common::*;
use forge::suggest::adapter::{LocalModelInvoke, ProcessModelAdapter};
use serde_json::{Value, json};
use std::path::Path;
use std::time::{Duration, Instant};

const VERSION: &str = "forge.suggest-task-drafting/1";

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

fn validate_clause(root: &Path, clause: Value) -> std::process::Output {
    suggest_run(root, "run-1", &suggest_response_json(&[clause], "policy-drafting", VERSION));
    suggest_validate(root, "prepared/run-1/run.json", "bundle-1", &[])
}

fn bundle(root: &Path) -> Value {
    read_json(&root.join("prepared/bundle-1/suggestions.json"))
}

fn promote(root: &Path, generation: &str) -> std::process::Output {
    run(
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
            generation,
            "--format",
            "json",
        ],
    )
}

#[test]
fn response_secret_must_be_refused_before_quarantine() {
    let (_temp, root) = project();
    suggest_prepare(&root);
    let mut clause = suggest_clause("unit-0002", None);
    let secret = "password: synthetic-review-secret";
    assert!(forge::suggest::redact::refuse_secrets(secret).is_err());
    clause["draft_text"] = json!(secret);
    let output = validate_clause(&root, clause);
    let published = root.join("prepared/bundle-1/suggestions.json");
    eprintln!(
        "validation exit={:?}, secret retained={}",
        output.status.code(),
        published.exists() && std::fs::read_to_string(&published).unwrap().contains(secret)
    );
    assert_exit(&output, 2);
    assert!(!published.exists());
}

#[test]
fn clause_target_must_belong_to_the_requested_sections() {
    let (_temp, root) = project();
    suggest_prepare(&root);
    let mut clause = suggest_clause("unit-0002", None);
    clause["policy_key"] = json!("invented-policy");
    clause["topic_key"] = json!("invented-topic");
    let output = validate_clause(&root, clause);
    if output.status.success() {
        eprintln!(
            "invented target admitted with evidence={}",
            bundle(&root)["suggestions"][0]["evidence_support"]
        );
    }
    assert_exit(&output, 2);
}

#[test]
fn edited_acceptance_must_preserve_the_edited_destination() {
    let (_temp, root) = project();
    let mut project_doc = read_json(&root.join("project.json"));
    project_doc["human_clauses"] = json!([]);
    write_json(&root.join("project.json"), &project_doc);
    suggest_prepare(&root);
    assert_exit(&validate_clause(&root, suggest_clause("unit-0003", None)), 0);
    let bundle = bundle(&root);
    let mut decision = suggest_disposition(&bundle, 0, "accept-edited");
    let mut edited = bundle["suggestions"][0]["body"].clone();
    edited["drafting"]["policy_key"] = json!("operations-policy");
    edited["drafting"]["topic_key"] = json!("operations-topic");
    edited["drafting"]["draft_text"] = json!("Operations changes require review.");
    let typed: forge::suggest::SuggestionBody = serde_json::from_value(edited.clone()).unwrap();
    decision["edited_sha256"] = json!(sha256_hex(&serde_json::to_vec(&typed).unwrap()));
    decision["edited"] = edited;
    assert_exit(&suggest_review(&root, &[decision], "review-1"), 0);
    assert_exit(&promote(&root, "promotion-1"), 0);
    let patch = read_json(&root.join("promotion-1/promotion/proposed-project.json"));
    let actual = &patch["human_clauses"][0];
    eprintln!("reviewed operations-policy/operations-topic, promoted={actual}");
    assert_eq!(actual["policy_key"], "operations-policy");
    assert_eq!(actual["topic_key"], "operations-topic");
}

#[test]
fn promotion_must_run_destination_cross_reference_validation() {
    let (_temp, root) = project();
    suggest_prepare(&root);
    assert_exit(&validate_clause(&root, suggest_clause("unit-0002", None)), 0);
    let bundle = bundle(&root);
    assert_exit(
        &suggest_review(&root, &[suggest_disposition(&bundle, 0, "accept-as-is")], "review-1"),
        0,
    );
    let mut pack = read_json(&root.join("pack.json"));
    pack["control_assignments"][0]["control_id"] = json!("c-4");
    let pack_hash = write_json(&root.join("pack.json"), &pack);
    let mut project_doc = read_json(&root.join("project.json"));
    project_doc["authoring_pack"]["expected_sha256"] = json!(pack_hash);
    write_json(&root.join("project.json"), &project_doc);
    let output = promote(&root, "promotion-1");
    if output.status.success() {
        std::fs::create_dir(root.join("promotion")).unwrap();
        let proposal = read_json(&root.join("promotion-1/promotion.json"));
        let artifact = proposal["entries"][0]["artifact"].as_str().unwrap();
        std::fs::copy(root.join("promotion-1").join(artifact), root.join(artifact)).unwrap();
        std::fs::copy(
            root.join("promotion-1/promotion/proposed-project.json"),
            root.join("project.json"),
        )
        .unwrap();
        let plan = run(&root, &["author", "plan", "--manifest", "project.json"]);
        eprintln!(
            "promotion exit=0; downstream plan exit={:?}; stderr={}",
            plan.status.code(),
            String::from_utf8_lossy(&plan.stderr)
        );
        assert_exit(&plan, 2);
    }
    assert_exit(&output, 2);
}

#[test]
fn promotion_must_not_reuse_an_existing_clause_source_path() {
    let (_temp, root) = project();
    std::fs::create_dir(root.join("promotion")).unwrap();
    std::fs::copy(root.join("clause.md"), root.join("promotion/entry-0001.md")).unwrap();
    let mut project_doc = read_json(&root.join("project.json"));
    project_doc["human_clauses"][0]["source"]["path"] = json!("promotion/entry-0001.md");
    write_json(&root.join("project.json"), &project_doc);
    suggest_prepare(&root);
    assert_exit(&validate_clause(&root, suggest_clause("unit-0002", None)), 0);
    let bundle = bundle(&root);
    assert_exit(
        &suggest_review(&root, &[suggest_disposition(&bundle, 0, "accept-as-is")], "review-1"),
        0,
    );
    let output = promote(&root, "promotion-1");
    eprintln!(
        "promotion into existing promotion/entry-0001.md: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_exit(&output, 0);
    let patch = read_json(&root.join("promotion-1/promotion/proposed-project.json"));
    let old = &patch["human_clauses"][0]["source"];
    let new = &patch["human_clauses"][1]["source"];
    eprintln!("old source={old}; new source={new}");
    assert_ne!(old["path"], new["path"], "one file cannot satisfy two different pinned digests");
}

#[test]
fn process_refusal_cannot_wait_on_descendant_pipes() {
    let adapter =
        ProcessModelAdapter::new(Path::new("/bin/sh"), &["-c".into(), "sleep 2 & exit 0".into()])
            .unwrap();
    let start = Instant::now();
    let result = adapter.invoke(b"", Duration::from_millis(100));
    eprintln!("timeout=100ms; elapsed={:?}; result={result:?}", start.elapsed());
    assert!(start.elapsed() < Duration::from_millis(800));
    assert!(result.is_err());
}

#[test]
fn bare_adapter_name_cannot_bypass_the_process_execution_refusal() {
    use std::os::unix::fs::PermissionsExt;
    let (_temp, root) = project();
    let local = root.join("review-adapter");
    std::fs::write(&local, b"#!/bin/sh\nprintf verified-adapter\n").unwrap();
    std::fs::set_permissions(&local, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::fs::create_dir(root.join("bin")).unwrap();
    let different = root.join("bin/review-adapter");
    std::fs::write(&different, b"#!/bin/sh\nprintf unverified-path-adapter\n").unwrap();
    std::fs::set_permissions(&different, std::fs::Permissions::from_mode(0o755)).unwrap();
    let output = run(
        &root,
        &[
            "suggest",
            "prepare",
            "--manifest",
            "project.json",
            "--output-dir",
            "prepared",
            "--adapter",
            "review-adapter",
            "--model-id",
            "synthetic",
            "--consent",
            "--operator-key",
            "human",
        ],
    );
    assert_exit(&output, 0);
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_forge"))
        .current_dir(&root)
        .env("PATH", root.join("bin"))
        .args([
            "suggest",
            "run",
            "--request",
            "prepared/request.json",
            "--consent",
            "prepared/consent.json",
            "--output-dir",
            "run-1",
        ])
        .output()
        .unwrap();
    assert_exit(&output, 1);
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("process adapter execution is disabled")
    );
    assert!(!root.join("prepared/run-1").exists());
}

#[test]
fn adapter_must_not_read_or_write_unallowlisted_project_files() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("unselected.txt");
    let sink = temp.path().join("modified.txt");
    std::fs::write(&source, b"synthetic unselected content").unwrap();
    let script = format!("cat '{}' > '{}'", source.display(), sink.display());
    let adapter = ProcessModelAdapter::new(Path::new("/bin/sh"), &["-c".into(), script]).unwrap();
    let output = adapter.invoke(b"only consented payload", Duration::from_secs(5));
    eprintln!("adapter result={output:?}; unallowlisted copy={}", sink.exists());
    assert!(!sink.exists(), "adapter escaped the declared context and publication boundary");
}

fn add_answer(root: &Path, stale: bool) {
    let pack_bytes = std::fs::read(root.join("pack.json")).unwrap();
    let pack = forge::authoring::manifest::parse_pack(&pack_bytes).unwrap();
    let question = &pack.questions[0];
    let mut doc = read_json(&root.join("project.json"));
    doc["answers"] = json!([{
        "key": "provided-answer", "question_key": question.key,
        "question_sha256": if stale { "f".repeat(64) } else { forge::authoring::manifest::question_sha256(question).unwrap() },
        "authoring_pack_sha256": sha256_hex(&pack_bytes), "owner": question.owner,
        "source_label": "Synthetic interview", "sensitivity": "internal", "review": review(),
        "state": "provided", "value": "Fictional Security Officer"
    }]);
    write_json(&root.join("project.json"), &doc);
}

fn prepare_with_answer(root: &Path) -> std::process::Output {
    let adapter = suggest_adapter(root);
    run(
        root,
        &[
            "suggest",
            "prepare",
            "--manifest",
            "project.json",
            "--output-dir",
            "prepared",
            "--adapter",
            &adapter,
            "--model-id",
            "synthetic",
            "--answer",
            "provided-answer",
            "--consent",
            "--operator-key",
            "human",
        ],
    )
}

#[test]
fn stale_answers_must_not_be_offered_as_approved_context() {
    let (_temp, root) = project();
    add_answer(&root, true);
    let plan = run(&root, &["author", "plan", "--manifest", "project.json", "--format", "json"]);
    assert!(String::from_utf8_lossy(&plan.stdout).contains("stale"));
    let output = prepare_with_answer(&root);
    eprintln!(
        "author plan reports stale; prepare exit={:?}; retained={}",
        output.status.code(),
        root.join("prepared/payload.txt").exists()
            && std::fs::read_to_string(root.join("prepared/payload.txt"))
                .unwrap()
                .contains("Fictional Security Officer")
    );
    assert_exit(&output, 2);
}

#[test]
fn promotion_must_pin_selected_required_answers() {
    let (_temp, root) = project();
    add_answer(&root, false);
    assert_exit(&prepare_with_answer(&root), 0);
    assert_exit(&validate_clause(&root, suggest_clause("unit-0002", None)), 0);
    let bundle = bundle(&root);
    assert_exit(
        &suggest_review(&root, &[suggest_disposition(&bundle, 0, "accept-as-is")], "review-1"),
        0,
    );
    assert_exit(&promote(&root, "promotion-1"), 0);
    std::fs::create_dir(root.join("promotion")).unwrap();
    let proposal = read_json(&root.join("promotion-1/promotion.json"));
    let artifact = proposal["entries"][0]["artifact"].as_str().unwrap();
    std::fs::copy(root.join("promotion-1").join(artifact), root.join(artifact)).unwrap();
    std::fs::copy(
        root.join("promotion-1/promotion/proposed-project.json"),
        root.join("project.json"),
    )
    .unwrap();
    let plan = run(&root, &["author", "plan", "--manifest", "project.json"]);
    eprintln!(
        "promoted selected-answer clause; author plan: {}",
        String::from_utf8_lossy(&plan.stderr)
    );
    assert_ne!(plan.status.code(), Some(2));
}

#[test]
fn bundle_provenance_must_distinguish_recording_from_model_execution() {
    let (_temp, root) = project();
    suggest_prepare(&root);
    assert_exit(&validate_clause(&root, suggest_clause("unit-0002", None)), 0);
    let bundle = bundle(&root);
    eprintln!(
        "run mode={}, bundle provenance={}",
        read_json(&root.join("prepared/run-1/run.json"))["mode"],
        bundle["provenance"]
    );
    assert_eq!(bundle["provenance"]["mode"], "recorded-response");
}

#[test]
fn redaction_must_not_recursively_expand_replacement_markers() {
    use std::fmt::Write as _;
    let mut rules = String::new();
    for index in 0..20 {
        writeln!(rules, "rule-{index}\te").unwrap();
    }
    let rules = forge::suggest::redact::parse_rules(rules.as_bytes()).unwrap();
    let (output, applied) = forge::suggest::redact::apply("e", &rules).unwrap();
    eprintln!(
        "one input byte, {} accepted rules, {} applied, {} output bytes",
        rules.len(),
        applied.len(),
        output.len()
    );
    assert!(output.len() <= 1024, "replacement markers must not become input to later redactions");
}

#[test]
fn escaped_secret_strings_are_refused_in_every_output_field() {
    for (field, array) in [
        ("draft_text", false),
        ("heading", false),
        ("assumptions", true),
        ("unresolved_questions", true),
        ("notes", true),
    ] {
        for retain_raw in [false, true] {
            let (_temp, root) = project();
            suggest_prepare(&root);
            let mut response: Value = serde_json::from_slice(&suggest_response_json(
                &[suggest_clause("unit-0002", None)],
                "policy-drafting",
                VERSION,
            ))
            .unwrap();
            let secret = "password: synthetic-review-secret";
            if field == "notes" {
                response[field] = json!([secret]);
            } else {
                response["task"]["draft_clauses"][0][field] =
                    if array { json!([secret]) } else { json!(secret) };
            }
            // The raw JSON contains no recognized credential key; decoding must
            // precede scanning to reject it.
            let encoded =
                serde_json::to_string(&response).unwrap().replace("password", r"pass\u0077ord");
            suggest_run(&root, "run-1", encoded.as_bytes());
            let extra: &[&str] = if retain_raw { &["--retain-raw"] } else { &[] };
            let output = suggest_validate(&root, "prepared/run-1/run.json", "bundle-1", extra);
            assert_exit(&output, 2);
            assert!(String::from_utf8_lossy(&output.stderr).contains("secret pattern"));
            assert!(!String::from_utf8_lossy(&output.stderr).contains("synthetic-review-secret"));
            assert!(!root.join("prepared/bundle-1").exists());
        }
    }
}

#[test]
fn stale_expired_and_invalid_answer_selections_include_the_evaluated_reason() {
    for reason in ["stale", "expired", "invalid"] {
        let (_temp, root) = project();
        add_answer(&root, reason == "stale");
        let mut project_doc = read_json(&root.join("project.json"));
        match reason {
            "expired" => project_doc["answers"][0]["expires_at"] = json!("2026-09-07T00:00:00Z"),
            "invalid" => project_doc["answers"][0]["value"] = json!(42),
            _ => {}
        }
        write_json(&root.join("project.json"), &project_doc);
        let output = prepare_with_answer(&root);
        assert_exit(&output, 2);
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(reason),
            "{reason}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!root.join("prepared").exists());
    }
}

#[test]
fn promotion_validation_never_overwrites_or_removes_existing_clause_paths() {
    for invalid_destination in [false, true] {
        let (_temp, root) = project();
        suggest_prepare(&root);
        assert_exit(&validate_clause(&root, suggest_clause("unit-0002", None)), 0);
        let bundle = bundle(&root);
        assert_exit(
            &suggest_review(&root, &[suggest_disposition(&bundle, 0, "accept-as-is")], "review-1"),
            0,
        );
        let id = bundle["suggestions"][0]["suggestion_id"].as_str().unwrap();
        std::fs::create_dir(root.join("promotion")).unwrap();
        let existing = root.join(format!("promotion/entry-{id}.md"));
        std::fs::write(&existing, b"unrelated operator work").unwrap();
        if invalid_destination {
            let mut pack = read_json(&root.join("pack.json"));
            pack["control_assignments"][0]["control_id"] = json!("c-4");
            let digest = write_json(&root.join("pack.json"), &pack);
            let mut doc = read_json(&root.join("project.json"));
            doc["authoring_pack"]["expected_sha256"] = json!(digest);
            write_json(&root.join("project.json"), &doc);
        }
        let before = std::fs::read(root.join("project.json")).unwrap();
        let output = promote(&root, "promotion-1");
        assert_exit(&output, if invalid_destination { 2 } else { 0 });
        assert_eq!(std::fs::read(&existing).unwrap(), b"unrelated operator work");
        assert_eq!(std::fs::read(root.join("project.json")).unwrap(), before);
    }
}

#[cfg(unix)]
#[test]
fn promotion_validation_does_not_follow_destination_staging_symlinks() {
    let (_temp, root) = project();
    suggest_prepare(&root);
    assert_exit(&validate_clause(&root, suggest_clause("unit-0002", None)), 0);
    let bundle = bundle(&root);
    assert_exit(
        &suggest_review(&root, &[suggest_disposition(&bundle, 0, "accept-as-is")], "review-1"),
        0,
    );
    let outside = tempfile::tempdir().unwrap();
    let id = bundle["suggestions"][0]["suggestion_id"].as_str().unwrap();
    let sentinel = outside.path().join(format!("entry-{id}.md"));
    std::fs::write(&sentinel, b"outside source").unwrap();
    std::os::unix::fs::symlink(outside.path(), root.join("promotion")).unwrap();
    assert_exit(&promote(&root, "promotion-1"), 0);
    assert_eq!(std::fs::read(&sentinel).unwrap(), b"outside source");
    assert!(root.join("promotion").is_symlink());
}
