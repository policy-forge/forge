//! PRD-061 Phase 1 CLI, containment, baseline, and end-to-end provenance contracts.

#[cfg(any(target_os = "linux", target_os = "macos"))]
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

    fn refresh_baseline(&mut self, manifest_path: &str) {
        assert!(self.project["answers"].as_array().unwrap().is_empty());
        let old_report_hash =
            self.project["baseline"]["report_sha256"].as_str().unwrap().to_owned();
        let old_report: Value =
            serde_json::from_slice(&std::fs::read(self.root.join("gap-report.json")).unwrap())
                .unwrap();
        let output = run(
            &self.root,
            &["applicability", "analyze", "--manifest", manifest_path, "--format", "json"],
        );
        assert_exit(&output, 0);
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        let report_hash = hash(&output.stdout);
        let remap_gap = |old_gap: &Value| {
            let control = old_report["controls"]
                .as_array()
                .unwrap()
                .iter()
                .find(|control| {
                    forge::authoring::manifest::gap_id(
                        &old_report_hash,
                        control["control_id"].as_str().unwrap(),
                    ) == old_gap.as_str().unwrap()
                })
                .unwrap();
            Value::String(forge::authoring::manifest::gap_id(
                &report_hash,
                control["control_id"].as_str().unwrap(),
            ))
        };
        for deferral in self.project["deferrals"].as_array_mut().unwrap() {
            deferral["gap_id"] = remap_gap(&deferral["gap_id"]);
        }
        for clause in self.project["human_clauses"].as_array_mut().unwrap() {
            for gap in clause["gap_ids"].as_array_mut().unwrap() {
                *gap = remap_gap(gap);
            }
        }
        let baseline = json!({
            "framework_sha256":report["framework"]["raw_sha256"],
            "report_sha256":report_hash
        });
        self.project["baseline"] = baseline.clone();
        self.pack["baseline"] = baseline;
        self.project["applicability_manifest"] = json!({
            "path":manifest_path,
            "expected_sha256":hash(&std::fs::read(self.root.join(manifest_path)).unwrap())
        });
        self.project["gap_report"]["expected_sha256"] = report_hash.into();
        std::fs::write(self.root.join("gap-report.json"), &output.stdout).unwrap();
        self.save();
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

#[cfg(any(target_os = "linux", target_os = "macos"))]
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
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn optional_context_pinned_by_a_clause_uses_a_truthful_blocking_marker() {
    let mut fixture = Fixture::new();
    fixture.pack["questions"][0]["required"] = json!(false);
    fixture.save();
    fixture.answered_clause();
    fixture.project["answers"][0]["state"] = json!("no-answer");
    fixture.project["answers"][0].as_object_mut().unwrap().remove("value");
    let answer: forge::authoring::manifest::Answer =
        serde_json::from_value(fixture.project["answers"][0].clone()).unwrap();
    fixture.project["human_clauses"][1]["answer_refs"][0]["expected_sha256"] =
        forge::authoring::manifest::answer_sha256(&answer).unwrap().into();
    fixture.save();

    assert_exit(&fixture.build("drafts"), 1);
    let markdown =
        std::fs::read_to_string(fixture.root.join("drafts/policies/access-policy.md")).unwrap();
    assert!(markdown.contains("UNRESOLVED OPTIONAL CONTEXT"));
    assert!(markdown.contains("Unresolved context prevents inclusion of human clauses"));
    assert!(!markdown.contains("Required context prevents"));
    assert!(!markdown.contains("fictional café team"));
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
fn imported_baseline_accepts_upstream_sized_rationale_and_framework_prose() {
    let mut fixture = Fixture::new();
    let framework_path = fixture.root.join("framework.json");
    let mut framework: Value =
        serde_json::from_slice(&std::fs::read(&framework_path).unwrap()).unwrap();
    let prose = "Synthetic ".repeat(7 * 1024);
    assert_eq!(prose.len(), 70 * 1024);
    framework["catalog"]["controls"][0]["parts"] = json!([{
        "id":"c-1-statement", "name":"statement", "prose":prose
    }]);
    write_json(&framework_path, &framework);
    let initial = run(&fixture.root, &["applicability", "init", "--framework", "framework.json"]);
    assert_exit(&initial, 0);
    let scaffold: Value = serde_json::from_slice(&initial.stdout).unwrap();
    let manifest_path = fixture.root.join("applicability.json");
    let mut applicability: Value =
        serde_json::from_slice(&std::fs::read(&manifest_path).unwrap()).unwrap();
    applicability["framework"] = scaffold["framework"].clone();
    let rationale = "Synthetic ".repeat(2048);
    assert_eq!(rationale.len(), 20 * 1024);
    applicability["decisions"][0]["rationale"] = rationale.clone().into();
    applicability["decisions"][1]["note"] = rationale.clone().into();
    write_json(&manifest_path, &applicability);
    fixture.refresh_baseline("applicability.json");
    let report: Value =
        serde_json::from_slice(&std::fs::read(fixture.root.join("gap-report.json")).unwrap())
            .unwrap();
    assert_eq!(report["controls"][0]["rationale"], rationale);
    assert_eq!(report["controls"][1]["note"], rationale);
    let output = fixture.plan();
    assert_exit(&output, 1);
    let plan: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(plan["counts"], json!({"total":4,"assigned":2,"deferred":1,"unresolved":1}));
    assert_eq!(
        plan["provenance"]["framework"]["raw_sha256"],
        hash(&std::fs::read(framework_path).unwrap())
    );
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    assert_exit(&fixture.build("drafts"), 1);
}

#[test]
fn imported_applicability_manifest_above_two_mib_keeps_its_exact_byte_pin() {
    let mut fixture = Fixture::new();
    let path = fixture.root.join("applicability.json");
    let mut bytes = std::fs::read(&path).unwrap();
    bytes.resize(2 * 1024 * 1024 + 1, b' ');
    std::fs::write(&path, &bytes).unwrap();
    fixture.refresh_baseline("applicability.json");
    let output = fixture.plan();
    assert_exit(&output, 1);
    let plan: Value = serde_json::from_slice(&output.stdout).unwrap();
    let fingerprint = plan["provenance"]["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|input| input["role"] == "applicability-manifest")
        .unwrap();
    assert_eq!(fingerprint["sha256"], hash(&bytes));
    assert_eq!(fingerprint["byte_length"].as_u64().unwrap(), u64::try_from(bytes.len()).unwrap());
    assert_eq!(fixture.project["applicability_manifest"]["expected_sha256"], hash(&bytes));
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    assert_exit(&fixture.build("drafts"), 1);
}

#[test]
fn nested_applicability_manifest_can_reference_its_contained_parent_framework() {
    let mut fixture = Fixture::new();
    let mut applicability: Value =
        serde_json::from_slice(&std::fs::read(fixture.root.join("applicability.json")).unwrap())
            .unwrap();
    applicability["framework"]["artifact"] = "../framework.json".into();
    std::fs::create_dir(fixture.root.join("baselines")).unwrap();
    write_json(&fixture.root.join("baselines/applicability.json"), &applicability);
    fixture.refresh_baseline("baselines/applicability.json");
    let output = fixture.plan();
    assert_exit(&output, 1);
    let plan: Value = serde_json::from_slice(&output.stdout).unwrap();
    let inputs = plan["provenance"]["inputs"].as_array().unwrap();
    assert!(inputs.iter().any(|input| input["role"] == "applicability-manifest"
        && input["path"] == "baselines/applicability.json"));
    assert!(
        inputs
            .iter()
            .any(|input| input["role"] == "framework" && input["path"] == "framework.json")
    );
    assert_eq!(plan["counts"], json!({"total":4,"assigned":2,"deferred":1,"unresolved":1}));
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    assert_exit(&fixture.build("drafts"), 1);
}

#[test]
fn nested_baseline_dependencies_keep_portable_labels_on_every_platform() {
    for (manifest_path, artifact, framework_path) in [
        ("applicability.json", "resources/framework.json", "resources/framework.json"),
        (
            "baselines/applicability.json",
            "resources/framework.json",
            "baselines/resources/framework.json",
        ),
        (
            "baselines/nested/applicability.json",
            "../resources/framework.json",
            "baselines/resources/framework.json",
        ),
    ] {
        let mut fixture = Fixture::new();
        let mut applicability: Value = serde_json::from_slice(
            &std::fs::read(fixture.root.join("applicability.json")).unwrap(),
        )
        .unwrap();
        applicability["framework"]["artifact"] = artifact.into();
        let framework = fixture.root.join(framework_path);
        std::fs::create_dir_all(framework.parent().unwrap()).unwrap();
        std::fs::copy(fixture.root.join("framework.json"), &framework).unwrap();
        let manifest = fixture.root.join(manifest_path);
        std::fs::create_dir_all(manifest.parent().unwrap()).unwrap();
        write_json(&manifest, &applicability);
        fixture.refresh_baseline(manifest_path);
        let output = fixture.plan();
        assert_exit(&output, 1);
        let plan: Value = serde_json::from_slice(&output.stdout).unwrap();
        let inputs = plan["provenance"]["inputs"].as_array().unwrap();
        assert!(
            inputs
                .iter()
                .any(|input| input["role"] == "framework" && input["path"] == framework_path)
        );
        assert!(inputs.iter().all(|input| !input["path"].as_str().unwrap().contains('\\')));
        assert_eq!(plan["counts"], json!({"total":4,"assigned":2,"deferred":1,"unresolved":1}));
    }
}

#[cfg(windows)]
#[test]
fn windows_manifest_root_aliases_fail_before_input_reads() {
    let fixture = Fixture::new();
    for suffix in [".", " "] {
        let mut alias = fixture.root.as_os_str().to_owned();
        alias.push(suffix);
        let manifest = PathBuf::from(alias).join("project.json");
        let output = run(
            &fixture.root,
            &["author", "plan", "--manifest", manifest.to_str().unwrap(), "--format", "json"],
        );
        assert_exit(&output, 2);
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("confined input root must be an absolute normalized directory")
        );
    }
}

#[test]
fn noncanonical_baseline_dependencies_fail_even_with_a_real_matching_report() {
    for artifact in
        ["sub/../framework.json", "./framework.json", "sub//framework.json", "sub/./framework.json"]
    {
        let mut fixture = Fixture::new();
        std::fs::create_dir(fixture.root.join("sub")).unwrap();
        std::fs::copy(fixture.root.join("framework.json"), fixture.root.join("sub/framework.json"))
            .unwrap();
        let path = fixture.root.join("applicability.json");
        let mut applicability: Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        applicability["framework"]["artifact"] = artifact.into();
        write_json(&path, &applicability);
        fixture.refresh_baseline("applicability.json");
        for output in [fixture.plan(), fixture.build("drafts")] {
            assert_exit(&output, 2);
            assert!(output.stdout.is_empty());
            assert!(
                String::from_utf8_lossy(&output.stderr).contains("non-canonical path spelling"),
                "{artifact}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        assert!(!fixture.root.join("drafts").exists());
    }
}

#[cfg(unix)]
#[test]
fn canceled_symlink_dependency_is_rejected_even_when_the_external_bytes_match() {
    use std::os::unix::fs::symlink;

    let mut fixture = Fixture::new();
    let outside = tempfile::tempdir().unwrap();
    let outside_root = outside.path().canonicalize().unwrap();
    std::fs::create_dir(outside_root.join("leaf")).unwrap();
    std::fs::copy(fixture.root.join("framework.json"), outside_root.join("framework.json"))
        .unwrap();
    symlink(outside_root.join("leaf"), fixture.root.join("sub")).unwrap();
    let path = fixture.root.join("applicability.json");
    let mut applicability: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    applicability["framework"]["artifact"] = "sub/../framework.json".into();
    write_json(&path, &applicability);
    fixture.refresh_baseline("applicability.json");
    for output in [fixture.plan(), fixture.build("drafts")] {
        assert_exit(&output, 2);
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("non-canonical path spelling"));
    }
    assert!(!fixture.root.join("drafts").exists());
    assert!(!outside_root.join("drafts").exists());
}

#[test]
fn checked_in_synthetic_example_keeps_its_exact_hash_chain_and_published_contracts() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let inputs: [(&str, &[u8]); 6] = [
        ("framework.json", include_bytes!("../examples/authoring/framework.json")),
        ("applicability.json", include_bytes!("../examples/authoring/applicability.json")),
        ("gap-report.json", include_bytes!("../examples/authoring/gap-report.json")),
        ("pack.json", include_bytes!("../examples/authoring/pack.json")),
        ("project.json", include_bytes!("../examples/authoring/project.json")),
        ("clause.md", include_bytes!("../examples/authoring/clause.md")),
    ];
    for (name, bytes) in inputs {
        std::fs::write(root.join(name), bytes).unwrap();
    }
    for (name, schema_bytes) in [
        ("pack.json", include_bytes!("../schemas/authoring-pack.schema.json").as_slice()),
        ("project.json", include_bytes!("../schemas/author-project.schema.json").as_slice()),
    ] {
        let schema: Value = serde_json::from_slice(schema_bytes).unwrap();
        let instance: Value =
            serde_json::from_slice(&std::fs::read(root.join(name)).unwrap()).unwrap();
        let validator = jsonschema::options().should_validate_formats(true).build(&schema).unwrap();
        assert!(validator.is_valid(&instance), "example violates published schema: {name}");
    }
    let output = run(&root, &["author", "plan", "--manifest", "project.json", "--format", "json"]);
    assert_exit(&output, 0);
    let plan: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(plan["counts"], json!({"total":2,"assigned":2,"deferred":0,"unresolved":0}));
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        for destination in ["first", "second"] {
            let build = run(
                &root,
                &[
                    "author",
                    "build",
                    "--manifest",
                    "project.json",
                    "--format",
                    "json",
                    "--output-dir",
                    destination,
                ],
            );
            assert_exit(&build, 0);
            assert_eq!(build.stdout, output.stdout);
        }
        assert_eq!(output_tree(&root.join("first")), output_tree(&root.join("second")));
    }
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
    // S-4: one gap is shared by both policy families through distinct topics.
    let mut policy_keys: Vec<&str> = first["assignments"]
        .as_array()
        .unwrap()
        .iter()
        .map(|assignment| assignment["policy_key"].as_str().unwrap())
        .collect();
    policy_keys.sort_unstable();
    assert_eq!(policy_keys, ["access-policy", "operations-policy"]);
    for policy_key in policy_keys {
        assert!(plan["policies"].as_array().unwrap().iter().any(|policy| {
            policy["policy_key"] == policy_key
                && policy["sections"].as_array().unwrap().iter().any(|section| {
                    section["gap_ids"].as_array().unwrap().iter().any(|gap| gap == &first["gap_id"])
                })
        }));
    }
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

/// S-1: scaffold an empty pack from a valid framework inventory.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn scaffold_creates_an_empty_pack_without_assignments_or_reviewer_provenance() {
    let fixture = Fixture::new();
    std::fs::remove_file(fixture.root.join("pack.json")).unwrap();
    assert_exit(&run(&fixture.root, &["author", "scaffold", "--manifest", "project.json"]), 0);
    let bytes = std::fs::read(fixture.root.join("pack.json")).unwrap();
    let pack: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(pack["schema_version"], "forge.authoring-pack/1");
    assert_eq!(pack["baseline"], fixture.project["baseline"]);
    assert_eq!(pack["reviewers"], json!([]));
    for field in
        ["topics", "policy_families", "questions", "control_assignments", "family_assignments"]
    {
        assert_eq!(pack[field], json!([]), "{field}");
    }
    assert_eq!(
        pack["content_rights"]["review"],
        json!({"reviewer_key":"","reviewed_at":"","rationale":""})
    );
    // Required reviewer provenance is emitted empty, so the scaffold is intentionally
    // not yet a usable pack.
    assert!(forge::authoring::manifest::parse_pack(&bytes).is_err());

    // Identical inputs reproduce identical bytes and never replace an existing pack.
    std::fs::remove_file(fixture.root.join("pack.json")).unwrap();
    assert_exit(&run(&fixture.root, &["author", "scaffold", "--manifest", "project.json"]), 0);
    assert_eq!(std::fs::read(fixture.root.join("pack.json")).unwrap(), bytes);
    assert_exit(&run(&fixture.root, &["author", "scaffold", "--manifest", "project.json"]), 2);
    assert_eq!(std::fs::read(fixture.root.join("pack.json")).unwrap(), bytes);
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn scaffold_requires_an_exactly_valid_framework_inventory() {
    let mut fixture = Fixture::new();
    std::fs::remove_file(fixture.root.join("pack.json")).unwrap();
    fixture.project["baseline"]["report_sha256"] = json!("0".repeat(64));
    fixture.project["gap_report"]["expected_sha256"] = json!("0".repeat(64));
    write_json(&fixture.root.join("project.json"), &fixture.project);
    assert_exit(&run(&fixture.root, &["author", "scaffold", "--manifest", "project.json"]), 2);
    assert!(!fixture.root.join("pack.json").exists());
}

#[cfg(unix)]
#[test]
fn scaffold_never_follows_a_symlinked_destination() {
    let fixture = Fixture::new();
    let outside = fixture.root.join("elsewhere.json");
    std::fs::write(&outside, b"{}\n").unwrap();
    std::fs::remove_file(fixture.root.join("pack.json")).unwrap();
    std::os::unix::fs::symlink("elsewhere.json", fixture.root.join("pack.json")).unwrap();
    assert_exit(&run(&fixture.root, &["author", "scaffold", "--manifest", "project.json"]), 2);
    assert_eq!(std::fs::read(&outside).unwrap(), b"{}\n");
}
