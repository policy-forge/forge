//! `forge author reuse` end-to-end contracts: closed report, exit codes, spans.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{Value, json};
mod common;
use common::sha256_hex as hash;

fn write_json(path: &Path, value: &Value) -> String {
    let bytes = serde_json::to_vec_pretty(value).unwrap();
    std::fs::write(path, &bytes).unwrap();
    hash(&bytes)
}

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_forge")).current_dir(root).args(args).output().unwrap()
}

fn assert_exit(output: &Output, expected: i32) {
    assert_eq!(
        output.status.code(),
        Some(expected),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn review() -> Value {
    json!({"reviewer_key":"human", "reviewed_at":"2026-09-01T00:00:00Z", "rationale":"Explicit synthetic drafting decision."})
}

/// A minimal authoring project with one blocked section and one drafted section.
fn project() -> (tempfile::TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let catalog = json!({"catalog": {
        "uuid":"11111111-1111-4111-8111-111111111111",
        "metadata": {"title":"Synthetic framework", "last-modified":"2026-09-01T00:00:00Z", "version":"1.0.0", "oscal-version":"1.2.3"},
        "controls": (1..=4).map(|n| json!({"id":format!("c-{n}"),"title":format!("Synthetic control {n}")})).collect::<Vec<_>>()
    }});
    let framework_hash = write_json(&root.join("framework.json"), &catalog);
    let initial = run(&root, &["applicability", "init", "--framework", "framework.json"]);
    assert_exit(&initial, 0);
    let mut applicability: Value = serde_json::from_slice(&initial.stdout).unwrap();
    applicability["reviewers"] =
        json!([{"key":"human","type":"person","name":"Synthetic Reviewer"}]);
    applicability["decisions"] = json!(
        (1..=4)
            .map(|n| json!({
                "control_id":format!("c-{n}"),"state":"applicable","reviewer_key":"human",
                "reviewed_at":"2026-09-01T00:00:00Z"
            }))
            .collect::<Vec<_>>()
    );
    let applicability_hash = write_json(&root.join("applicability.json"), &applicability);
    let result = run(
        &root,
        &["applicability", "analyze", "--manifest", "applicability.json", "--format", "json"],
    );
    assert_exit(&result, 0);
    std::fs::write(root.join("gap-report.json"), &result.stdout).unwrap();
    let report_hash = hash(&result.stdout);
    let baseline = json!({"framework_sha256":framework_hash,"report_sha256":report_hash});
    let question = json!({
        "key":"responsible-role", "prompt":"Which role writes this draft?", "type":"string",
        "required":true,"owner":"context-owner","sensitivity":"internal","source_label":"Synthetic interview",
        "max_age_days":30,"constraints":{"min_length":1,"max_length":100}
    });
    let pack = json!({
        "schema_version":"forge.authoring-pack/1","pack_key":"synthetic-pack","version":"1.0.0",
        "baseline":baseline,"reviewers":[{"key":"human","name":"Synthetic Reviewer"}],
        "content_rights":{"source_label":"Repository synthetic fixture","statement":"All fixture content is synthetic.","review":review()},
        "topics":[
            {"key":"access-topic","title":"Access drafting","order":20,"question_keys":["responsible-role"]},
            {"key":"operations-topic","title":"Operations drafting","order":10,"question_keys":[]}
        ],
        "policy_families":[{"key":"access-family","title":"Access family"},{"key":"operations-family","title":"Operations family"}],
        "questions":[question],
        "control_assignments":[
            {"key":"control-access","control_id":"c-1","topic_key":"access-topic","review":review()},
            {"key":"control-operations","control_id":"c-2","topic_key":"operations-topic","review":review()}
        ],
        "family_assignments":[
            {"key":"family-access","topic_key":"access-topic","policy_family_key":"access-family","review":review()},
            {"key":"family-operations","topic_key":"operations-topic","policy_family_key":"operations-family","review":review()}
        ]
    });
    let pack_hash = write_json(&root.join("pack.json"), &pack);
    let gap = |id: &str| forge::authoring::manifest::gap_id(&report_hash, id);
    let clause = "The fictional team records draft changes in the sample register.\n";
    std::fs::write(root.join("clause.md"), clause).unwrap();
    let project = json!({
        "schema_version":"forge.author-project/1","project_key":"synthetic-project","project_root":".",
        "baseline":baseline,"as_of":"2026-09-08T00:00:00Z",
        "applicability_manifest":{"path":"applicability.json","expected_sha256":applicability_hash},
        "gap_report":{"path":"gap-report.json","expected_sha256":report_hash},
        "authoring_pack":{"path":"pack.json","expected_sha256":pack_hash},
        "reviewers":[{"key":"human","name":"Synthetic Reviewer"}],"baseline_review":review(),
        "policies":[{"key":"access-policy","policy_family_key":"access-family","title":"Access draft"},{"key":"operations-policy","policy_family_key":"operations-family","title":"Operations draft"}],
        "answers":[],
        "deferrals":[{"key":"later","gap_id":gap("c-3"),"review":review(),"revisit_date":"2026-10-01"}],
        "human_clauses":[{"key":"operations-clause","policy_key":"operations-policy","topic_key":"operations-topic","gap_ids":[gap("c-2")],"answer_refs":[],"source":{"path":"clause.md","expected_sha256":hash(clause.as_bytes())},"review":review()}]
    });
    write_json(&root.join("project.json"), &project);
    (temp, root)
}

/// Write a corpus manifest and its one document, returning the document bytes.
fn corpus(root: &Path, body: &str) -> Vec<u8> {
    std::fs::create_dir_all(root.join("prior")).unwrap();
    std::fs::write(root.join("prior/access.md"), body).unwrap();
    let manifest = json!({
        "schema_version":"forge.reuse-corpus/1","corpus_key":"synthetic-corpus",
        "title":"Synthetic prior policies",
        "documents":[{
            "key":"prior-access","path":"prior/access.md","title":"Prior access policy",
            "status":"approved","rights_label":"Repository synthetic fixture",
            "source_label":"Synthetic interview","expected_sha256":hash(body.as_bytes()),
            "topic_keys":["access-topic"],"control_ids":["c-1"]
        }]
    });
    write_json(&root.join("corpus.json"), &manifest);
    body.as_bytes().to_vec()
}

fn reuse(root: &Path, extra: &[&str]) -> Output {
    let mut args = vec!["author", "reuse", "--manifest", "project.json", "--corpus", "corpus.json"];
    args.extend_from_slice(extra);
    run(root, &args)
}

#[test]
fn report_is_closed_and_span_exact_and_exits_zero() {
    let (_temp, root) = project();
    let body = "# Access drafting\n\nc-1 Approve access requests.\n\nUnrelated tail line.\n";
    let bytes = corpus(&root, body);
    let output = reuse(&root, &["--format", "json"]);
    assert_exit(&output, 0);
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["schema_version"], json!("forge.authoring-reuse/1"));
    assert_eq!(report["project_key"], json!("synthetic-project"));
    assert_eq!(report["as_of"], json!("2026-09-08T00:00:00Z"));
    assert_eq!(report["counts"]["sections"], json!(1));
    assert_eq!(report["counts"]["with_candidates"], json!(1));
    assert_eq!(report["counts"]["low_confidence"], json!(0));
    let section = &report["sections"][0];
    assert_eq!(section["policy_key"], json!("access-policy"));
    assert_eq!(section["topic_key"], json!("access-topic"));
    assert_eq!(section["state"], json!("blocked-context"));
    assert_eq!(section["control_ids"], json!(["c-1"]));
    let candidate = &section["candidates"][0];
    assert_eq!(candidate["source_key"], json!("prior-access"));
    assert_eq!(candidate["source_path"], json!("prior/access.md"));
    assert_eq!(candidate["source_sha256"], json!(hash(&bytes)));
    assert_eq!(candidate["low_confidence"], json!(false));
    assert!(candidate["reasons"].as_array().unwrap().contains(&json!("control-match")));
    let start = usize::try_from(candidate["span"]["start"].as_u64().unwrap()).unwrap();
    let end = usize::try_from(candidate["span"]["end"].as_u64().unwrap()).unwrap();
    assert_eq!(&bytes[start..end], b"c-1 Approve access requests.");
}

#[test]
fn weak_candidates_and_empty_sections_exit_one_with_a_valid_report() {
    let (_temp, root) = project();
    corpus(&root, "# Access drafting\n\nunrelated words\n");
    let weak = reuse(&root, &["--format", "json"]);
    assert_exit(&weak, 1);
    let report: Value = serde_json::from_slice(&weak.stdout).unwrap();
    assert_eq!(report["counts"]["low_confidence"], json!(1));
    assert_eq!(report["sections"][0]["candidates"][0]["low_confidence"], json!(true));

    corpus(&root, "# Other\n\nzzz\n");
    let empty = reuse(&root, &["--format", "json"]);
    assert_exit(&empty, 1);
    let report: Value = serde_json::from_slice(&empty.stdout).unwrap();
    assert_eq!(report["counts"]["sections"], json!(1));
    assert_eq!(report["counts"]["with_candidates"], json!(0));
}

#[test]
fn text_reports_match_the_json_facts() {
    let (_temp, root) = project();
    corpus(&root, "# Access drafting\n\nc-1 Approve access requests.\n");
    let output = reuse(&root, &[]);
    assert_exit(&output, 0);
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("schema: forge.authoring-reuse/1"));
    assert!(text.contains("project: synthetic-project"));
    assert!(text.contains("path=prior/access.md"));
    assert!(text.contains("reasons=control-match"));
    assert!(text.contains("sections: 1 with-candidates=1 low-confidence=0 candidates=1"));
    for forbidden in ["approved", "compliant", "certified", "validated"] {
        assert!(!text.to_lowercase().contains(forbidden), "{forbidden}");
    }
}

#[test]
fn draft_documents_are_excluded_unless_requested() {
    let (_temp, root) = project();
    std::fs::create_dir_all(root.join("prior")).unwrap();
    std::fs::write(root.join("prior/access.md"), "# Access drafting\n\nc-1 Approved text.\n")
        .unwrap();
    std::fs::write(root.join("prior/draft.md"), "# Access drafting\n\nc-1 Draft text.\n").unwrap();
    let manifest = json!({
        "schema_version":"forge.reuse-corpus/1","corpus_key":"synthetic-corpus",
        "documents":[{
            "key":"prior-access","path":"prior/access.md","title":"Prior access policy",
            "status":"approved","rights_label":"Repository synthetic fixture",
            "source_label":"Synthetic interview",
            "expected_sha256":hash(b"# Access drafting\n\nc-1 Approved text.\n"),"control_ids":["c-1"]
        },{
            "key":"draft-access","path":"prior/draft.md","title":"Draft access policy",
            "status":"draft","rights_label":"Repository synthetic fixture",
            "source_label":"Synthetic interview",
            "expected_sha256":hash(b"# Access drafting\n\nc-1 Draft text.\n"),"control_ids":["c-1"]
        }]
    });
    write_json(&root.join("corpus.json"), &manifest);

    let default: Value =
        serde_json::from_slice(&reuse(&root, &["--format", "json"]).stdout).unwrap();
    let keys: Vec<&str> = default["sections"][0]["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .map(|candidate| candidate["source_key"].as_str().unwrap())
        .collect();
    assert_eq!(keys, ["prior-access"]);

    let included: Value =
        serde_json::from_slice(&reuse(&root, &["--format", "json", "--include-draft"]).stdout)
            .unwrap();
    assert_eq!(included["sections"][0]["candidates"].as_array().unwrap().len(), 2);
}

#[test]
fn invalid_arguments_and_inputs_exit_two() {
    let (_temp, root) = project();
    corpus(&root, "# Access drafting\n\nc-1 Approve access requests.\n");

    let capped = reuse(&root, &["--max-candidates", "0"]);
    assert_exit(&capped, 2);
    assert!(capped.stdout.is_empty());

    let negative = reuse(&root, &["--min-score", "-1"]);
    assert_exit(&negative, 2);

    let missing =
        run(&root, &["author", "reuse", "--manifest", "project.json", "--corpus", "absent.json"]);
    assert_exit(&missing, 2);

    let mut unknown: Value =
        serde_json::from_slice(&std::fs::read(root.join("corpus.json")).unwrap()).unwrap();
    unknown["pointer"] = json!("nope");
    write_json(&root.join("corpus.json"), &unknown);
    assert_exit(&reuse(&root, &[]), 2);
}

#[test]
fn repeated_runs_produce_byte_identical_reports() {
    let (_temp, root) = project();
    corpus(&root, "# Access drafting\n\nc-1 Approve access requests.\n");
    let first = reuse(&root, &["--format", "json"]);
    let second = reuse(&root, &["--format", "json"]);
    assert_exit(&first, 0);
    assert_eq!(first.stdout, second.stdout);
}
