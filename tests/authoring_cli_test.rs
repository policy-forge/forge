//! PRD-061 Phase 1 CLI, containment, baseline, and end-to-end provenance contracts.

use std::collections::BTreeMap;
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

struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
    pack: Value,
    project: Value,
}

impl Fixture {
    fn new() -> Self {
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
        Self { _temp: temp, root, pack, project }
    }

    fn save(&mut self) {
        self.project["authoring_pack"]["expected_sha256"] =
            write_json(&self.root.join("pack.json"), &self.pack).into();
        write_json(&self.root.join("project.json"), &self.project);
    }

    fn answer(&mut self, value: Value) {
        let question: forge::authoring::manifest::Question =
            serde_json::from_value(self.pack["questions"][0].clone()).unwrap();
        self.project["answers"] = json!([{
            "key":"role-answer", "question_key":"responsible-role",
            "question_sha256":forge::authoring::manifest::question_sha256(&question).unwrap(),
            "authoring_pack_sha256":self.project["authoring_pack"]["expected_sha256"],
            "owner":"context-owner","source_label":"Synthetic interview","sensitivity":"internal",
            "review":review(),"state":"provided"
        }]);
        self.project["answers"][0]["value"] = value;
    }

    fn answered_clause(&mut self) {
        self.answer(json!("Synthetic draft custodian"));
        let answer: forge::authoring::manifest::Answer =
            serde_json::from_value(self.project["answers"][0].clone()).unwrap();
        let answer_hash = forge::authoring::manifest::answer_sha256(&answer).unwrap();
        let gap_id = self.gap_id("c-1");
        let bytes = "The fictional café team records access-draft changes.\r\n".as_bytes();
        std::fs::write(self.root.join("access-clause.md"), bytes).unwrap();
        self.project["human_clauses"].as_array_mut().unwrap().push(json!({
            "key":"access-clause", "policy_key":"access-policy", "topic_key":"access-topic",
            "gap_ids":[gap_id],
            "answer_refs":[{"answer_key":"role-answer","expected_sha256":answer_hash}],
            "source":{"path":"access-clause.md","expected_sha256":hash(bytes)},
            "review":review()
        }));
        self.save();
    }

    fn plan(&self) -> Output {
        run(&self.root, &["author", "plan", "--manifest", "project.json", "--format", "json"])
    }

    fn build(&self, dir: &str) -> Output {
        run(
            &self.root,
            &[
                "author",
                "build",
                "--manifest",
                "project.json",
                "--output-dir",
                dir,
                "--format",
                "json",
            ],
        )
    }

    fn gap_id(&self, id: &str) -> String {
        forge::authoring::manifest::gap_id(
            self.project["baseline"]["report_sha256"].as_str().unwrap(),
            id,
        )
    }
}

fn output_tree(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn visit(base: &Path, path: &Path, files: &mut BTreeMap<String, Vec<u8>>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                visit(base, &entry.path(), files);
            } else {
                files.insert(
                    entry.path().strip_prefix(base).unwrap().to_string_lossy().replace('\\', "/"),
                    std::fs::read(entry.path()).unwrap(),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}

#[test]
fn complete_gap_accounting_and_dependent_only_blocking() {
    let fixture = Fixture::new();
    let output = fixture.plan();
    assert_exit(&output, 1);
    let plan: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(plan["counts"], json!({"total":4,"assigned":2,"deferred":1,"unresolved":1}));
    assert_eq!(plan["policies"][0]["state"], "blocked-context");
    assert_eq!(plan["policies"][1]["state"], "human-draft-present");
    assert_eq!(plan["unresolved_gaps"][0]["control_id"], "c-4");
    assert_eq!(plan["unresolved_questions"][0]["state"], "missing");
    assert_eq!(plan["gaps"][0]["assignments"][0]["control_assignment"]["review"], review());
}

#[test]
fn supplied_answer_is_hash_only_in_reports_and_never_interpolated() {
    let mut fixture = Fixture::new();
    fixture.answer(json!("PRIVATE_CONTEXT_VALUE"));
    fixture.save();
    let plan = fixture.plan();
    assert_exit(&plan, 1);
    assert!(!String::from_utf8_lossy(&plan.stdout).contains("PRIVATE_CONTEXT_VALUE"));
    let value: Value = serde_json::from_slice(&plan.stdout).unwrap();
    assert_eq!(value["policies"][0]["state"], "skeleton-ready");
    assert_eq!(value["questions"][0]["state"], "available");
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        assert_exit(&fixture.build("drafts"), 1);
        for bytes in output_tree(&fixture.root.join("drafts")).values() {
            assert!(!String::from_utf8_lossy(bytes).contains("PRIVATE_CONTEXT_VALUE"));
        }
    }
}

#[test]
fn invalid_stale_expired_and_explicit_no_answer_remain_local_blockers() {
    for (field, value, expected) in [
        ("value", json!(false), "invalid"),
        ("question_sha256", json!("0".repeat(64)), "stale"),
        ("authoring_pack_sha256", json!("0".repeat(64)), "stale"),
        ("expires_at", json!("2026-09-08T00:00:00Z"), "expired"),
    ] {
        let mut fixture = Fixture::new();
        fixture.answer(json!("Draft custodian"));
        fixture.project["answers"][0][field] = value;
        fixture.save();
        let output = fixture.plan();
        assert_exit(&output, 1);
        let plan: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(plan["questions"][0]["state"], expected, "{field}");
        assert_eq!(plan["policies"][0]["state"], "blocked-context");
        assert_eq!(plan["policies"][1]["state"], "human-draft-present");
    }
    let mut fixture = Fixture::new();
    fixture.answer(json!("Unused"));
    fixture.project["answers"][0]["state"] = json!("no-answer");
    fixture.project["answers"][0].as_object_mut().unwrap().remove("value");
    fixture.save();
    let output = fixture.plan();
    assert_exit(&output, 1);
    let plan: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(plan["questions"][0]["state"], "no-answer");
}

#[test]
fn age_expiry_and_optional_unanswered_context_are_explicit() {
    let mut fixture = Fixture::new();
    fixture.pack["questions"][0]["max_age_days"] = json!(1);
    fixture.save();
    fixture.answer(json!("Draft custodian"));
    fixture.save();
    let output = fixture.plan();
    assert_exit(&output, 1);
    let plan: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(plan["questions"][0]["state"], "expired");
    fixture.pack["questions"][0]["required"] = json!(false);
    fixture.project["answers"] = json!([]);
    fixture.save();
    let output = fixture.plan();
    assert_exit(&output, 1);
    let plan: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(plan["policies"][0]["state"], "skeleton-ready");
    assert_eq!(plan["unresolved_questions"][0]["state"], "missing");
}

#[test]
fn exact_source_fingerprint_mismatches_fail_without_output() {
    for file in
        ["framework.json", "applicability.json", "gap-report.json", "pack.json", "clause.md"]
    {
        let fixture = Fixture::new();
        let path = fixture.root.join(file);
        let mut bytes = std::fs::read(&path).unwrap();
        bytes.push(b' ');
        std::fs::write(path, bytes).unwrap();
        let output = fixture.build("drafts");
        assert_exit(&output, 2);
        assert!(output.stdout.is_empty());
        assert!(!fixture.root.join("drafts").exists());
    }
}

#[test]
fn pinned_fabricated_and_filtered_reports_are_rejected() {
    for edit in ["counts", "controls", "filters", "unknown"] {
        let mut fixture = Fixture::new();
        let path = fixture.root.join("gap-report.json");
        let mut report: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        match edit {
            "counts" => report["counts"]["total"] = json!(3),
            "controls" => {
                report["controls"].as_array_mut().unwrap().pop();
            }
            "filters" => report["filters"]["control_prefix"] = json!("c-1"),
            _ => report["unknown"] = json!(true),
        }
        let report_hash = write_json(&path, &report);
        fixture.project["gap_report"]["expected_sha256"] = report_hash.clone().into();
        fixture.project["baseline"]["report_sha256"] = report_hash.clone().into();
        fixture.pack["baseline"]["report_sha256"] = report_hash.into();
        fixture.save();
        let output = fixture.plan();
        assert_exit(&output, 2);
        assert!(String::from_utf8_lossy(&output.stderr).contains("complete current unfiltered"));
    }
}

#[test]
fn assignment_and_deferral_conflicts_are_rejected() {
    let mut fixture = Fixture::new();
    fixture.project["deferrals"][0]["gap_id"] = fixture.gap_id("c-1").into();
    fixture.save();
    assert_exit(&fixture.plan(), 2);
}

#[test]
fn assignments_without_a_selected_policy_remain_unresolved() {
    let mut fixture = Fixture::new();
    fixture.project["policies"].as_array_mut().unwrap().remove(0);
    fixture.save();
    let output = fixture.plan();
    assert_exit(&output, 1);
    let plan: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(plan["counts"], json!({"total":4,"assigned":1,"deferred":1,"unresolved":2}));
}

#[test]
fn multiple_topics_and_families_do_not_double_count_gaps() {
    let mut fixture = Fixture::new();
    fixture.pack["control_assignments"].as_array_mut().unwrap().push(json!({
        "key":"control-shared", "control_id":"c-1", "topic_key":"operations-topic", "review":review()
    }));
    fixture.save();
    let output = fixture.plan();
    assert_exit(&output, 1);
    let plan: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(plan["counts"]["assigned"], 2);
    let first =
        plan["gaps"].as_array().unwrap().iter().find(|gap| gap["control_id"] == "c-1").unwrap();
    assert_eq!(first["assignments"].as_array().unwrap().len(), 2);
}

#[test]
fn all_accounted_gaps_and_available_context_exit_zero() {
    let mut fixture = Fixture::new();
    fixture.answer(json!("Draft custodian"));
    let gap_id = fixture.gap_id("c-4");
    fixture.project["deferrals"].as_array_mut().unwrap().push(json!({
        "key":"later-four", "gap_id":gap_id, "review":review()
    }));
    fixture.save();
    assert_exit(&fixture.plan(), 0);
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn repeated_and_cross_directory_builds_have_identical_complete_bytes() {
    let fixture = Fixture::new();
    assert_exit(&fixture.build("first"), 1);
    assert_exit(&fixture.build("second"), 1);
    let first = output_tree(&fixture.root.join("first"));
    assert_eq!(first, output_tree(&fixture.root.join("second")));
    let relocated = tempfile::tempdir().unwrap();
    let other_root = relocated.path().canonicalize().unwrap();
    for name in [
        "project.json",
        "pack.json",
        "applicability.json",
        "framework.json",
        "gap-report.json",
        "clause.md",
    ] {
        std::fs::copy(fixture.root.join(name), other_root.join(name)).unwrap();
    }
    assert_exit(
        &run(
            &other_root,
            &[
                "author",
                "build",
                "--manifest",
                "project.json",
                "--output-dir",
                "third",
                "--format",
                "json",
            ],
        ),
        1,
    );
    assert_eq!(first, output_tree(&other_root.join("third")));
    for bytes in first.values() {
        let text = String::from_utf8_lossy(bytes);
        assert!(!text.contains(fixture.root.to_str().unwrap()));
        assert!(!text.contains(other_root.to_str().unwrap()));
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn overwrite_and_stale_source_preserve_existing_complete_generation() {
    let fixture = Fixture::new();
    assert_exit(&fixture.build("drafts"), 1);
    let before = output_tree(&fixture.root.join("drafts"));
    assert_exit(&fixture.build("drafts"), 2);
    assert_eq!(before, output_tree(&fixture.root.join("drafts")));
    std::fs::write(fixture.root.join("clause.md"), "Changed human words.\n").unwrap();
    assert_exit(&fixture.build("drafts"), 2);
    assert_eq!(before, output_tree(&fixture.root.join("drafts")));
}

#[test]
fn invalid_output_paths_and_input_aliases_fail_closed() {
    let fixture = Fixture::new();
    for destination in ["../outside", "pack.json", "new-parent/drafts", ".", "C:\\drafts"] {
        assert_exit(&fixture.build(destination), 2);
    }
    let before = std::fs::read(fixture.root.join("pack.json")).unwrap();
    assert_exit(&fixture.build("pack.json"), 2);
    assert_eq!(before, std::fs::read(fixture.root.join("pack.json")).unwrap());
}

#[cfg(unix)]
#[test]
fn symlink_and_hard_link_inputs_and_output_parents_are_rejected() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new();
    std::fs::rename(fixture.root.join("clause.md"), fixture.root.join("real.md")).unwrap();
    symlink("real.md", fixture.root.join("clause.md")).unwrap();
    assert_exit(&fixture.plan(), 2);
    std::fs::remove_file(fixture.root.join("clause.md")).unwrap();
    std::fs::hard_link(fixture.root.join("real.md"), fixture.root.join("clause.md")).unwrap();
    assert_exit(&fixture.plan(), 2);
    let fixture = Fixture::new();
    std::fs::create_dir(fixture.root.join("real")).unwrap();
    symlink("real", fixture.root.join("linked")).unwrap();
    assert_exit(&fixture.build("linked/drafts"), 2);
    assert!(!fixture.root.join("real/drafts").exists());
}

#[test]
fn generated_reports_use_only_authoring_outcome_terminology() {
    let fixture = Fixture::new();
    let output = fixture.plan();
    assert_exit(&output, 1);
    let text = String::from_utf8(output.stdout).unwrap().to_lowercase();
    for word in ["compliant", "certified", "implemented", "effective", "approved"] {
        assert!(!text.split(|c: char| !c.is_alphanumeric()).any(|token| token == word), "{word}");
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
#[test]
fn unsupported_directory_publication_fails_before_any_output() {
    let fixture = Fixture::new();
    assert_exit(&fixture.build("drafts"), 2);
    assert!(!fixture.root.join("drafts").exists());
    assert_exit(&fixture.plan(), 1);
}

#[test]
fn changed_clause_answer_pin_fails_plan_and_build_without_output() {
    let mut fixture = Fixture::new();
    fixture.answered_clause();
    assert_exit(&fixture.plan(), 1);
    fixture.project["answers"][0]["value"] = json!("A different explicit custodian");
    fixture.save();
    for output in [fixture.plan(), fixture.build("drafts")] {
        assert_exit(&output, 2);
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("answer pin"));
    }
    assert!(!fixture.root.join("drafts").exists());
}

#[test]
fn invalid_pinned_clause_grammar_fails_plan_and_build_before_output() {
    for bytes in [
        b"# A second policy title\n".as_slice(),
        b"<script>example</script>\n".as_slice(),
        b"```text\nUnclosed code block\n".as_slice(),
        b"[shared]: https://example.invalid/\n".as_slice(),
        b"Invalid UTF-8: \xff\n".as_slice(),
    ] {
        let mut fixture = Fixture::new();
        std::fs::write(fixture.root.join("clause.md"), bytes).unwrap();
        fixture.project["human_clauses"][0]["source"]["expected_sha256"] = hash(bytes).into();
        fixture.save();
        for output in [fixture.plan(), fixture.build("drafts")] {
            assert_exit(&output, 2);
            assert!(output.stdout.is_empty());
            assert!(String::from_utf8_lossy(&output.stderr).contains("human clause"));
        }
        assert!(!fixture.root.join("drafts").exists());
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod provenance_contract {
    use std::collections::BTreeSet;

    use super::*;

    fn record<'a>(records: &'a Value, field: &str, key: &Value) -> &'a Value {
        records.as_array().unwrap().iter().find(|record| &record[field] == key).unwrap()
    }

    fn labels(value: &Value) -> BTreeSet<&str> {
        value.as_array().unwrap().iter().map(|label| label.as_str().unwrap()).collect()
    }

    fn byte_range(value: &Value) -> std::ops::Range<usize> {
        usize::try_from(value["start"].as_u64().unwrap()).unwrap()
            ..usize::try_from(value["end"].as_u64().unwrap()).unwrap()
    }

    fn assert_input_hashes(fixture: &Fixture, plan: &Value, graph: &Value) {
        assert_eq!(graph["input_provenance"], plan["provenance"]);
        let inputs = graph["input_provenance"]["inputs"].as_array().unwrap();
        let actual_paths: BTreeSet<_> =
            inputs.iter().map(|input| input["path"].as_str().unwrap()).collect();
        assert_eq!(
            actual_paths,
            BTreeSet::from([
                "project.json",
                "pack.json",
                "applicability.json",
                "framework.json",
                "gap-report.json",
                "clause.md",
                "access-clause.md",
            ])
        );
        assert_eq!(inputs.len(), actual_paths.len());
        for input in inputs {
            let path = input["path"].as_str().unwrap();
            assert!(!Path::new(path).is_absolute());
            let bytes = std::fs::read(fixture.root.join(path)).unwrap();
            assert_eq!(input["sha256"], hash(&bytes));
            assert_eq!(input["byte_length"].as_u64().unwrap(), u64::try_from(bytes.len()).unwrap());
            assert!(!input["role"].as_str().unwrap().is_empty());
        }
        for (field, path) in [
            ("project_sha256", "project.json"),
            ("pack_sha256", "pack.json"),
            ("report_sha256", "gap-report.json"),
        ] {
            assert_eq!(
                graph["input_provenance"][field],
                hash(&std::fs::read(fixture.root.join(path)).unwrap())
            );
        }
        assert_eq!(
            graph["input_provenance"]["framework"]["raw_sha256"],
            hash(&std::fs::read(fixture.root.join("framework.json")).unwrap())
        );
        assert_eq!(
            graph["input_provenance"]["baseline_review"],
            fixture.project["baseline_review"]
        );
        assert_eq!(graph["input_provenance"]["pack_reviewers"], fixture.pack["reviewers"]);
        assert_eq!(graph["input_provenance"]["project_reviewers"], fixture.project["reviewers"]);
    }

    fn assert_section_edges(fixture: &Fixture, graph: &Value, policy: &Value, section: &Value) {
        let topic = record(&fixture.pack["topics"], "key", &section["topic_key"]);
        assert_eq!(section["title"], topic["title"]);
        let controls: BTreeSet<_> = section["gap_ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|id| record(&graph["gaps"], "gap_id", id)["control_id"].as_str().unwrap())
            .collect();
        assert_eq!(labels(&section["control_ids"]), controls);
        for assignment in section["assignments"].as_array().unwrap() {
            assert_eq!(assignment["policy_key"], policy["policy_key"]);
            assert_eq!(assignment["topic_key"], section["topic_key"]);
            let control = &assignment["control_assignment"];
            let family = &assignment["family_assignment"];
            assert_eq!(
                control,
                record(&fixture.pack["control_assignments"], "key", &control["key"])
            );
            assert_eq!(family, record(&fixture.pack["family_assignments"], "key", &family["key"]));
            assert_eq!(control["topic_key"], section["topic_key"]);
            assert_eq!(family["topic_key"], section["topic_key"]);
            assert_eq!(family["policy_family_key"], policy["policy_family_key"]);
            let gap = record(&graph["gaps"], "control_id", &control["control_id"]);
            assert!(gap["assignments"].as_array().unwrap().contains(assignment));
        }
        for question in section["questions"].as_array().unwrap() {
            assert_eq!(
                question,
                record(&graph["questions"], "question_key", &question["question_key"])
            );
        }
    }

    fn assert_clause_origin(fixture: &Fixture, graph: &Value, origin: &Value, output: &[u8]) {
        let clause = record(&graph["clauses"], "key", &origin["clause_key"]);
        assert_eq!(clause, record(&fixture.project["human_clauses"], "key", &origin["clause_key"]));
        assert_eq!(origin["policy_key"], clause["policy_key"]);
        assert_eq!(origin["topic_key"], clause["topic_key"]);
        assert_eq!(labels(&origin["gap_ids"]), labels(&clause["gap_ids"]));
        let controls: BTreeSet<_> = origin["gap_ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|id| record(&graph["gaps"], "gap_id", id)["control_id"].as_str().unwrap())
            .collect();
        assert_eq!(labels(&origin["control_ids"]), controls);
        let policy = record(&graph["policy_plans"], "policy_key", &origin["policy_key"]);
        let section = record(&policy["sections"], "topic_key", &origin["topic_key"]);
        for (origin_field, edge_field) in [
            ("control_assignment_keys", "control_assignment"),
            ("family_assignment_keys", "family_assignment"),
        ] {
            let keys: BTreeSet<_> = section["assignments"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|assignment| {
                    controls
                        .contains(assignment["control_assignment"]["control_id"].as_str().unwrap())
                })
                .map(|assignment| assignment[edge_field]["key"].as_str().unwrap())
                .collect();
            assert_eq!(labels(&origin[origin_field]), keys);
        }
        assert_eq!(origin["answer_refs"], clause["answer_refs"]);
        for pin in origin["answer_refs"].as_array().unwrap() {
            let answer: forge::authoring::manifest::Answer = serde_json::from_value(
                record(&fixture.project["answers"], "key", &pin["answer_key"]).clone(),
            )
            .unwrap();
            let digest = forge::authoring::manifest::answer_sha256(&answer).unwrap();
            assert_eq!(pin["expected_sha256"], digest);
            let question = record(&graph["questions"], "answer_key", &pin["answer_key"]);
            assert_eq!(question["answer_sha256"], digest);
            assert_eq!(question["question_key"], answer.question_key);
            assert_eq!(question["review"], serde_json::to_value(&answer.review).unwrap());
        }
        let source = &origin["source"];
        assert_eq!(source["path"], clause["source"]["path"]);
        let bytes = std::fs::read(fixture.root.join(source["path"].as_str().unwrap())).unwrap();
        assert_eq!(source["sha256"], hash(&bytes));
        assert_eq!(source["sha256"], clause["source"]["expected_sha256"]);
        let range = byte_range(&source["bytes"]);
        assert_eq!(range, 0..bytes.len());
        assert_eq!(output, &bytes[range]);
    }

    fn assert_policy_spans(fixture: &Fixture, graph: &Value, tree: &BTreeMap<String, Vec<u8>>) {
        let mut included_clauses = BTreeSet::new();
        let policies = graph["policies"].as_array().unwrap();
        assert_eq!(policies.len(), fixture.project["policies"].as_array().unwrap().len());
        for policy in policies {
            let bytes = &tree[policy["path"].as_str().unwrap()];
            let text = std::str::from_utf8(bytes).unwrap();
            assert_eq!(policy["sha256"], hash(bytes));
            assert_eq!(
                policy["byte_length"].as_u64().unwrap(),
                u64::try_from(bytes.len()).unwrap()
            );
            let policy_plan = record(&graph["policy_plans"], "policy_key", &policy["policy_key"]);
            let mut cursor = 0;
            for span in policy["spans"].as_array().unwrap() {
                let range = byte_range(&span["output"]);
                assert_eq!(range.start, cursor);
                assert!(range.end > range.start && range.end <= bytes.len());
                assert!(text.is_char_boundary(range.start) && text.is_char_boundary(range.end));
                cursor = range.end;
                let origin = &span["origin"];
                assert_eq!(origin["policy_key"], policy["policy_key"]);
                if !origin["topic_key"].is_null() {
                    let section =
                        record(&policy_plan["sections"], "topic_key", &origin["topic_key"]);
                    assert_section_edges(fixture, graph, policy_plan, section);
                }
                match origin["kind"].as_str().unwrap() {
                    "human-clause" => {
                        assert_clause_origin(fixture, graph, origin, &bytes[range]);
                        assert!(included_clauses.insert(origin["clause_key"].as_str().unwrap()));
                    }
                    "generated-metadata" => {
                        assert!(!origin["field"].as_str().unwrap().is_empty());
                        assert!(origin["source"].is_null());
                    }
                    unknown => panic!("unknown provenance origin: {unknown}"),
                }
            }
            assert_eq!(cursor, bytes.len());
        }
        assert_eq!(included_clauses, BTreeSet::from(["access-clause", "operations-clause"]));
    }

    #[test]
    fn published_provenance_binds_every_output_byte_to_complete_input_and_human_evidence() {
        let mut fixture = Fixture::new();
        for control_id in ["c-3", "c-4"] {
            fixture.pack["control_assignments"].as_array_mut().unwrap().push(json!({
                "key":format!("control-access-{control_id}"), "control_id":control_id,
                "topic_key":"access-topic", "review":review()
            }));
        }
        fixture.project["deferrals"] = json!([]);
        fixture.save();
        fixture.answered_clause();
        fixture.project["human_clauses"][1]["gap_ids"] =
            json!([fixture.gap_id("c-4"), fixture.gap_id("c-1"), fixture.gap_id("c-3")]);
        fixture.save();
        let output = fixture.build("drafts");
        assert_exit(&output, 0);
        let tree = output_tree(&fixture.root.join("drafts"));
        let plan: Value = serde_json::from_slice(&tree["plan.json"]).unwrap();
        let graph: Value = serde_json::from_slice(&tree["provenance.json"]).unwrap();
        assert_eq!(output.stdout, tree["plan.json"]);
        assert_eq!(graph["schema_version"], "forge.authoring-provenance/1");
        assert_eq!(graph["project_key"], fixture.project["project_key"]);
        assert_eq!(graph["plan_sha256"], hash(&tree["plan.json"]));
        assert_eq!(graph["gaps"], plan["gaps"]);
        assert_eq!(graph["questions"], plan["questions"]);
        assert_eq!(graph["policy_plans"], plan["policies"]);
        assert_eq!(graph["clauses"].as_array().unwrap().len(), 2);
        assert_input_hashes(&fixture, &plan, &graph);
        assert_policy_spans(&fixture, &graph, &tree);
    }
}
