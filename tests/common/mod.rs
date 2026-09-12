#![allow(dead_code)]
pub mod fixture_generator;

use std::path::{Path, PathBuf};

use forge::model::{DocumentMetadata, PolicyDocument, PolicyRequirement, PolicySection};

use sha2::{Digest, Sha256};

/// SHA-256 fingerprint as 64 lowercase hexadecimal characters.
///
/// Mirrors the production representation in `src/hashing.rs` (digest outputs
/// lost their `LowerHex` impl in the sha2 0.11 stack, so bytes are encoded
/// explicitly).
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for &byte in &digest {
        use std::fmt::Write as _;
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

/// SHA-256 fingerprint of one file's contents.
pub fn sha256_file(path: &std::path::Path) -> String {
    sha256_hex(&std::fs::read(path).expect("read fixture"))
}

/// Normalize a JSON value for stable snapshot comparison.
///
/// Replaces dynamic fields with stable placeholder values so that
/// identical inputs always produce the same snapshot, regardless of
/// when or where the test is run:
///
/// - Whole-string UUID values → `"00000000-0000-0000-0000-000000000000"`
///
/// Embedded UUIDs are intentionally preserved: current dynamic UUIDs are whole-string values,
/// while UUIDs in composite OSCAL content are deterministic identifiers with semantic meaning.
/// - ISO 8601 timestamp strings → `"2026-01-01T00:00:00Z"`
/// - Repo-local and Windows absolute path strings → `"NORMALIZED_PATH"`
///
/// Normalization is applied recursively to all JSON values.
pub fn normalize_for_snapshot(value: &serde_json::Value) -> serde_json::Value {
    use serde_json::Value;

    // UUID pattern: 8-4-4-4-12 hex digits
    static UUID_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(
            r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$",
        )
        .expect("UUID regex is valid")
    });
    static TIMESTAMP_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r"^\d{4}-\d{2}-\d{2}T[0-9:.]+(?:Z|[+-]\d{2}:\d{2})$")
            .expect("timestamp regex is valid")
    });

    match value {
        Value::Object(map) => Value::Object(
            map.iter().map(|(key, value)| (key.clone(), normalize_for_snapshot(value))).collect(),
        ),
        Value::Array(values) => Value::Array(values.iter().map(normalize_for_snapshot).collect()),
        Value::String(value) if UUID_RE.is_match(value) => {
            Value::String("00000000-0000-0000-0000-000000000000".to_string())
        }
        Value::String(value) if TIMESTAMP_RE.is_match(value) => {
            Value::String("2026-01-01T00:00:00Z".to_string())
        }
        Value::String(value) if is_repo_local_path(value) || is_windows_path(value) => {
            Value::String("NORMALIZED_PATH".to_string())
        }
        _ => value.clone(),
    }
}

/// Returns true when a path belongs to this checkout rather than OSCAL content.
fn is_repo_local_path(s: &str) -> bool {
    Path::new(s).starts_with(Path::new(env!("CARGO_MANIFEST_DIR")))
}

/// Returns true if the string looks like a Windows absolute path.
fn is_windows_path(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(
        (chars.next(), chars.next(), chars.next()),
        (Some(c), Some(':'), Some('\\' | '/')) if c.is_ascii_alphabetic()
    ) || s.starts_with(r"\\")
}

/// Shared production ingest limit used by integration tests.
pub const DEFAULT_MAX_SIZE_BYTES: u64 = forge::DEFAULT_MAX_SIZE_BYTES;
/// Assert that a required fixture exists, failing the test otherwise.
///
/// The synthetic fixture generator is deterministic (no randomness or time
/// dependence), so a missing fixture is always a genuine defect — never a
/// reason to skip quietly (F0832).
#[track_caller]
pub fn require_fixture(path: &Path) {
    assert!(
        path.exists(),
        "required fixture missing (run the fixture generator?): {}",
        path.display()
    );
}

pub fn make_req(text: &str, source_line: usize) -> PolicyRequirement {
    PolicyRequirement {
        stable_id: None,
        text: text.to_string(),
        source_line,
        nesting_depth: 0,
        atom_index: 0,
        parent_text: None,
        citations: vec![],
        modality: None,
        parameters: vec![],
        parameters_extracted: false,
    }
}

pub fn make_section(title: &str, requirements: Vec<PolicyRequirement>) -> PolicySection {
    PolicySection {
        title: title.to_string(),
        heading_level: 1,
        source_line: 1,
        body_text: None,
        children: vec![],
        requirements,
    }
}

pub fn make_doc(title: &str, sections: Vec<PolicySection>) -> PolicyDocument {
    PolicyDocument {
        id: "test".to_string(),
        metadata: DocumentMetadata {
            title: title.to_string(),
            version: "0.0.0".to_string(),
            author: None,
            date: None,
            source_path: PathBuf::from("test.md"),
            content_hash: None,
        },
        sections,
    }
}

// Shared authoring/suggestion fixtures
use serde_json::{Value, json};
use std::process::{Command, Output};
pub fn write_json(path: &Path, value: &Value) -> String {
    let bytes = serde_json::to_vec_pretty(value).unwrap();
    std::fs::write(path, &bytes).unwrap();
    sha256_hex(&bytes)
}

pub fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_forge")).current_dir(root).args(args).output().unwrap()
}

pub fn assert_exit(output: &Output, expected: i32) {
    assert_eq!(
        output.status.code(),
        Some(expected),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

pub fn review() -> Value {
    json!({"reviewer_key":"human", "reviewed_at":"2026-09-01T00:00:00Z", "rationale":"Explicit synthetic drafting decision."})
}

/// A minimal authoring project with one blocked section and one drafted section.
pub fn project() -> (tempfile::TempDir, PathBuf) {
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
    let report_hash = sha256_hex(&result.stdout);
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
        "human_clauses":[{"key":"operations-clause","policy_key":"operations-policy","topic_key":"operations-topic","gap_ids":[gap("c-2")],"answer_refs":[],"source":{"path":"clause.md","expected_sha256":sha256_hex(clause.as_bytes())},"review":review()}]
    });
    write_json(&root.join("project.json"), &project);
    (temp, root)
}

/// Write a corpus manifest and its one document, returning the document bytes.
pub fn corpus(root: &Path, body: &str) -> Vec<u8> {
    std::fs::create_dir_all(root.join("prior")).unwrap();
    std::fs::write(root.join("prior/access.md"), body).unwrap();
    let manifest = json!({
        "schema_version":"forge.reuse-corpus/1","corpus_key":"synthetic-corpus",
        "title":"Synthetic prior policies",
        "documents":[{
            "key":"prior-access","path":"prior/access.md","title":"Prior access policy",
            "status":"approved","rights_label":"Repository synthetic fixture",
            "source_label":"Synthetic interview","expected_sha256":sha256_hex(body.as_bytes()),
            "topic_keys":["access-topic"],"control_ids":["c-1"]
        }]
    });
    write_json(&root.join("corpus.json"), &manifest);
    body.as_bytes().to_vec()
}

pub fn reuse(root: &Path, extra: &[&str]) -> Output {
    let mut args = vec!["author", "reuse", "--manifest", "project.json", "--corpus", "corpus.json"];
    args.extend_from_slice(extra);
    run(root, &args)
}

#[cfg(test)]
mod normalization_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn preserves_slash_prefixed_oscal_hrefs() {
        let value = normalize_for_snapshot(&json!({"href": "/oscal/cat/1.1.3"}));
        assert_eq!(value["href"], "/oscal/cat/1.1.3");
    }

    #[test]
    fn normalizes_checkout_and_windows_absolute_paths() {
        let checkout = format!("{}/tests/fixture.md", env!("CARGO_MANIFEST_DIR"));
        let value = normalize_for_snapshot(&json!({
            "checkout": checkout,
            "unc": r"\\server\share\fixture.md",
            "verbatim": r"\\?\C:\fixture.md",
        }));
        assert_eq!(value["checkout"], "NORMALIZED_PATH");
        assert_eq!(value["unc"], "NORMALIZED_PATH");
        assert_eq!(value["verbatim"], "NORMALIZED_PATH");
    }

    #[test]
    fn normalizes_timestamp_values_regardless_of_key() {
        let value = normalize_for_snapshot(&json!({
            "last-modified": "2026-08-26T12:34:56Z",
            "published": "2026-08-26T12:34:56.789+02:00",
            "date": "2026-08-26",
        }));

        assert_eq!(value["last-modified"], "2026-01-01T00:00:00Z");
        assert_eq!(value["published"], "2026-01-01T00:00:00Z");
        assert_eq!(value["date"], "2026-08-26");
    }
}
