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
fn review_corpus_serialization_stays_in_its_own_contract() {
    let (_temp, root) = project();
    corpus(&root, "# Access\n\nc-1 text\n");
    let parsed =
        forge::reuse::corpus::Corpus::parse(&std::fs::read(root.join("corpus.json")).unwrap())
            .unwrap();
    let serialized = serde_json::to_vec(&parsed).unwrap();
    let roundtrip = forge::reuse::corpus::Corpus::parse(&serialized);
    assert!(roundtrip.is_ok(), "a parsed Corpus serializes to rejected JSON: {roundtrip:?}");
}

#[test]
fn review_candidate_pointers_are_bound_to_captured_inputs() {
    let (_temp, root) = project();
    corpus(&root, "# Access\n\nc-1 text\n");
    let output = reuse(&root, &["--format", "json"]);
    assert_exit(&output, 0);
    let good: Value = serde_json::from_slice(&output.stdout).unwrap();
    let mut accepted = Vec::new();
    for (name, path, replacement) in [
        ("outside path", "/sections/0/candidates/0/source_path", json!("../../outside.md")),
        ("wrong source hash", "/sections/0/candidates/0/source_sha256", json!("b".repeat(64))),
        ("span beyond EOF", "/sections/0/candidates/0/span/end", json!(1_048_577)),
        (
            "empty span",
            "/sections/0/candidates/0/span/end",
            good["sections"][0]["candidates"][0]["span"]["start"].clone(),
        ),
        ("missing source", "/sections/0/candidates/0/source_key", json!("undeclared")),
    ] {
        let mut value = good.clone();
        *value.pointer_mut(path).unwrap() = replacement;
        if forge::reuse::report::ReuseReport::parse(&serde_json::to_vec(&value).unwrap()).is_ok() {
            accepted.push(name);
        }
    }
    assert!(accepted.is_empty(), "accepted invalid candidate pointers: {accepted:?}");
}

#[cfg(unix)]
#[test]
fn review_symlinked_corpus_manifest_is_rejected() {
    let (_temp, root) = project();
    corpus(&root, "# Access\n\nc-1 text\n");
    std::fs::rename(root.join("corpus.json"), root.join("real-corpus.json")).unwrap();
    std::os::unix::fs::symlink("real-corpus.json", root.join("corpus.json")).unwrap();
    assert_exit(&reuse(&root, &["--format", "json"]), 2);
}

#[cfg(unix)]
#[test]
fn review_fifo_corpus_is_rejected_without_blocking() {
    use std::time::{Duration, Instant};
    let (_temp, root) = project();
    assert!(Command::new("mkfifo").arg(root.join("corpus.json")).status().unwrap().success());
    let mut child = Command::new(env!("CARGO_BIN_EXE_forge"))
        .current_dir(&root)
        .args(["author", "reuse", "--manifest", "project.json", "--corpus", "corpus.json"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert_eq!(status.code(), Some(2));
            return;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("corpus FIFO blocked the command instead of failing closed");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(unix)]
#[test]
fn review_project_changed_after_seed_is_rejected_before_publication() {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    use std::time::{Duration, Instant};
    let (_temp, root) = project();
    corpus(&root, "# Access\n\nc-1 text\n");
    let bytes = std::fs::read(root.join("corpus.json")).unwrap();
    std::fs::remove_file(root.join("corpus.json")).unwrap();
    assert!(Command::new("mkfifo").arg(root.join("corpus.json")).status().unwrap().success());
    let mut child = Command::new(env!("CARGO_BIN_EXE_forge"))
        .current_dir(&root)
        .args([
            "author",
            "reuse",
            "--manifest",
            "project.json",
            "--corpus",
            "corpus.json",
            "--format",
            "json",
            "--output-dir",
            "out",
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut pipe = loop {
        if let Ok(pipe) = std::fs::OpenOptions::new()
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(root.join("corpus.json"))
        {
            break pipe;
        }
        if let Some(status) = child.try_wait().unwrap() {
            assert_eq!(status.code(), Some(2));
            return;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("child never opened the corpus");
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    // Opening the writer establishes that reuse_seed has completed. The reader
    // cannot finish until the corpus bytes arrive, so mutate a pin in that gap.
    std::fs::write(root.join("pack.json"), b"{}\n").unwrap();
    pipe.write_all(&bytes).unwrap();
    drop(pipe);
    let output = child.wait_with_output().unwrap();
    let published = root.join("out/reuse.json").exists();
    assert_eq!(
        output.status.code(),
        Some(2),
        "changed project accepted; complete stale generation published={published}; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn review_heading_expansion_stays_within_source_budget() {
    let heading = "a".repeat(65536);
    let body = format!("# {heading}\n\n{}", "body\n\n".repeat(1024));
    assert!(body.len() < usize::try_from(forge::reuse::corpus::MAX_DOCUMENT_BYTES).unwrap());
    let blocks = forge::reuse::blocks::blocks(body.as_bytes()).unwrap();
    let allocated: usize =
        blocks.iter().filter_map(|b| b.scope_title.as_ref()).map(String::capacity).sum();
    assert!(
        allocated <= 50 * 1024 * 1024,
        "{} bytes of input retained {} bytes of duplicated headings in {} blocks",
        body.len(),
        allocated,
        blocks.len()
    );
}
