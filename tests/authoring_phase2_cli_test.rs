//! Synthetic end-to-end Phase 2 contracts and publication boundaries.
use serde_json::{Value, json};
mod common;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn hash(bytes: &[u8]) -> String {
    common::sha256_hex(bytes)
}
fn read(root: &Path, path: &str) -> Value {
    serde_json::from_slice(&std::fs::read(root.join(path)).unwrap()).unwrap()
}
fn write(root: &Path, path: &str, value: &Value) {
    std::fs::write(root.join(path), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}
fn pin(root: &Path, path: &str) -> Value {
    json!({"path":path,"expected_sha256":hash(&std::fs::read(root.join(path)).unwrap())})
}
fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_forge")).current_dir(root).args(args).output().unwrap()
}
fn exit(output: &Output, code: i32) {
    assert_eq!(
        output.status.code(),
        Some(code),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if code == 2 {
        assert!(
            String::from_utf8_lossy(&output.stderr).starts_with("Error: "),
            "expected an application diagnostic, not a clap usage failure: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
fn fixture(root: &Path) {
    std::fs::create_dir_all(root).unwrap();
    for file in [
        "framework.json",
        "applicability.json",
        "gap-report.json",
        "pack.json",
        "project.json",
        "clause.md",
    ] {
        std::fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/authoring").join(file),
            root.join(file),
        )
        .unwrap();
    }
}
fn temp() -> (tempfile::TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    (temp, root)
}
fn extension(root: &Path) -> Value {
    let project = read(root, "project.json");
    let body = "## Sample drafting section\n\nThe fictional {{forge:param:owner-role}} records sample decisions.\n";
    std::fs::write(root.join("component.md"), body).unwrap();
    write(
        root,
        "component.json",
        &json!({"schema_version":"forge.policy-component/1","component_key":"sample-component","version":"1.0.0","title":"Synthetic component","owner":"sample-owner","status":"approved","source":"component.md","expected_sha256":hash(body.as_bytes()),"parameters":[{"name":"owner-role","type":"string","required":false,"default":"Never implicitly selected","constraints":{"min_length":1,"max_length":100}}]}),
    );
    let answer: forge::authoring::manifest::Answer =
        serde_json::from_value(project["answers"][0].clone()).unwrap();
    let extension = json!({"schema_version":"forge.author-components/1","project_sha256":pin(root,"project.json")["expected_sha256"],"instances":[{"instance_key":"sample-instance","policy_key":"sample-policy","topic_key":"sample-topic","gap_ids":project["human_clauses"][0]["gap_ids"],"component_manifest":pin(root,"component.json"),"source":pin(root,"component.md"),"parameters":{"owner-role":{"kind":"answer","question_key":"sample-question","answer_key":"sample-answer","expected_sha256":forge::authoring::manifest::answer_sha256(&answer).unwrap()}},"review":project["baseline_review"]}]});
    write(root, "components.json", &extension);
    extension
}
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn tree(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn visit(root: &Path, at: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for item in std::fs::read_dir(at).unwrap() {
            let path = item.unwrap().path();
            if path.is_dir() {
                visit(root, &path, out);
            } else {
                out.insert(
                    path.strip_prefix(root).unwrap().to_str().unwrap().replace('\\', "/"),
                    std::fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut out = BTreeMap::new();
    visit(root, root, &mut out);
    out
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn opt_in_components_html_exact_provenance_and_cross_directory_bytes() {
    let (_tmp, root) = temp();
    for directory in ["left", "right"] {
        let at = root.join(directory);
        fixture(&at);
        extension(&at);
        exit(
            &run(
                &at,
                &[
                    "author",
                    "build",
                    "--manifest",
                    "project.json",
                    "--components",
                    "components.json",
                    "--html",
                    "--output-dir",
                    "out",
                ],
            ),
            0,
        );
    }
    assert_eq!(tree(&root.join("left/out")), tree(&root.join("right/out")));
    let at = root.join("left");
    let outputs = tree(&at.join("out"));
    assert!(outputs.contains_key("components.lock.json"));
    assert!(outputs.contains_key("plan.html"));
    let markdown = String::from_utf8(outputs["policies/sample-policy.md"].clone()).unwrap();
    assert_eq!(markdown.lines().filter(|line| line.starts_with("# ")).count(), 1);
    assert_eq!(markdown.matches("## Sample drafting section").count(), 1);
    assert!(markdown.contains("Fictional draft custodian"));
    for path in [
        "plan.json",
        "plan.txt",
        "provenance.json",
        "components.lock.json",
        "plan.html",
        "provenance.html",
    ] {
        let text = String::from_utf8(outputs[path].clone()).unwrap();
        assert!(!text.contains("Fictional draft custodian"), "{path}");
    }
    let graph: Value = serde_json::from_slice(&outputs["provenance.json"]).unwrap();
    assert_eq!(graph["schema_version"], "forge.authoring-provenance/2");
    let policies = graph["policies"].as_array().unwrap();
    assert_eq!(policies.len(), 1);
    assert_eq!(policies[0]["policy_key"], "sample-policy");
    for policy in policies {
        let mut end = 0;
        let spans = policy["spans"].as_array().unwrap();
        assert!(!spans.is_empty());
        for span in spans {
            assert_eq!(span["output"]["start"].as_u64().unwrap(), end);
            end = span["output"]["end"].as_u64().unwrap();
        }
        assert_eq!(end, policy["byte_length"].as_u64().unwrap());
        assert_eq!(usize::try_from(end).unwrap(), outputs["policies/sample-policy.md"].len());
    }
    let before = outputs;
    std::fs::write(at.join("component.md"), "## Sample drafting section\n\nDrift.\n").unwrap();
    exit(
        &run(
            &at,
            &[
                "author",
                "build",
                "--manifest",
                "project.json",
                "--components",
                "components.json",
                "--output-dir",
                "out",
            ],
        ),
        2,
    );
    assert_eq!(before, tree(&at.join("out")));
    exit(
        &run(
            &at,
            &[
                "author",
                "build",
                "--manifest",
                "project.json",
                "--components",
                "components.json",
                "--output-dir",
                "drift",
            ],
        ),
        2,
    );
    assert!(!at.join("drift").exists());
}

#[test]
fn component_defaults_never_fill_missing_explicit_bindings_and_blocked_sources_still_validate() {
    let (_tmp, root) = temp();
    fixture(&root);
    let mut ext = extension(&root);
    ext["instances"][0]["parameters"] = json!({});
    write(&root, "components.json", &ext);
    exit(
        &run(
            &root,
            &["author", "plan", "--manifest", "project.json", "--components", "components.json"],
        ),
        2,
    );
    let mut project = read(&root, "project.json");
    let original_answer: forge::authoring::manifest::Answer =
        serde_json::from_value(project["answers"][0].clone()).unwrap();
    let original_answer_sha256 =
        forge::authoring::manifest::answer_sha256(&original_answer).unwrap();
    project["answers"] = json!([]);
    project["human_clauses"] = json!([]);
    write(&root, "project.json", &project);
    // Retain the exact formerly supplied answer pin to represent missing context.
    ext["project_sha256"] = pin(&root, "project.json")["expected_sha256"].clone();
    ext["instances"][0]["parameters"] = json!({"owner-role":{"kind":"answer","question_key":"sample-question","answer_key":"sample-answer","expected_sha256":original_answer_sha256}});
    write(&root, "components.json", &ext);
    let result = run(
        &root,
        &[
            "author",
            "plan",
            "--manifest",
            "project.json",
            "--components",
            "components.json",
            "--format",
            "json",
        ],
    );
    exit(&result, 1);
    let value: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(value["plan"]["policies"][0]["sections"][0]["state"], "blocked-context");
    assert_ne!(value["plan"]["policies"][0]["sections"][1]["state"], "blocked-context");
    std::fs::write(root.join("component.md"), "changed").unwrap();
    exit(
        &run(
            &root,
            &["author", "plan", "--manifest", "project.json", "--components", "components.json"],
        ),
        2,
    );
}

fn impact_request(root: &Path, old: &str, new: &str) {
    write(
        root,
        "impact.json",
        &json!({"schema_version":"forge.authoring-impact/1","old":{"project":pin(root,old)},"new":{"project":pin(root,new)}}),
    );
}
#[test]
fn impact_noop_independent_directories_report_churn_and_unverified_inputs() {
    let (_tmp, root) = temp();
    fixture(&root.join("old"));
    fixture(&root.join("new"));
    impact_request(&root, "old/project.json", "new/project.json");
    let result = run(&root, &["author", "impact", "--manifest", "impact.json", "--format", "json"]);
    exit(&result, 0);
    let value: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(value["status"], "complete");
    assert!(value["findings"].as_array().unwrap().is_empty());
    impact_request(&root, "old/project.json", "old/project.json");
    exit(&run(&root, &["author", "impact", "--manifest", "impact.json"]), 0);
    // Reformat only the report, manually refreshing every affected binding.
    let new = root.join("new");
    let report = read(&new, "gap-report.json");
    std::fs::write(new.join("gap-report.json"), serde_json::to_vec(&report).unwrap()).unwrap();
    let report_hash = pin(&new, "gap-report.json")["expected_sha256"].as_str().unwrap().to_owned();
    let mut pack = read(&new, "pack.json");
    pack["baseline"]["report_sha256"] = json!(report_hash);
    write(&new, "pack.json", &pack);
    let pack_hash = pin(&new, "pack.json")["expected_sha256"].clone();
    let mut project = read(&new, "project.json");
    project["baseline"]["report_sha256"] = json!(report_hash);
    project["gap_report"] = pin(&new, "gap-report.json");
    project["authoring_pack"] = pin(&new, "pack.json");
    project["answers"][0]["authoring_pack_sha256"] = pack_hash;
    let answer: forge::authoring::manifest::Answer =
        serde_json::from_value(project["answers"][0].clone()).unwrap();
    project["human_clauses"][0]["answer_refs"][0]["expected_sha256"] =
        json!(forge::authoring::manifest::answer_sha256(&answer).unwrap());
    project["human_clauses"][0]["gap_ids"] =
        json!([forge::authoring::manifest::gap_id(&report_hash, "sample-1")]);
    write(&new, "project.json", &project);
    impact_request(&root, "old/project.json", "new/project.json");
    let result = run(&root, &["author", "impact", "--manifest", "impact.json", "--format", "json"]);
    exit(&result, 1);
    let value: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(value["status"], "complete");
    for finding in value["findings"].as_array().unwrap() {
        assert_ne!(finding["category"], "gap-added");
        assert_ne!(finding["category"], "gap-removed");
    }
    assert!(!String::from_utf8_lossy(&result.stdout).contains("Fictional draft custodian"));
    std::fs::write(new.join("clause.md"), "drift").unwrap();
    let result = run(&root, &["author", "impact", "--manifest", "impact.json", "--format", "json"]);
    exit(&result, 2);
    let value: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(value["status"], "incomplete");
    assert!(value["sections"].as_array().unwrap().is_empty());
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn handoff_request(root: &Path, keys: &[&str]) {
    let policies:Vec<_>=keys.iter().map(|key|json!({"author_policy_key":key,"draft":pin(root,&format!("build/policies/{key}.md")),"policy_key":format!("governed-{key}"),"version_key":"draft-v1","title":"Explicit synthetic draft","owner_keys":["owner"],"parties":[{"key":"owner","roles":["owner"]},{"key":"reviewer","roles":["reviewer"]},{"key":"approver","roles":["approver"]}],"approval_policy":{"schema_version":"forge.approval-policy/1","required_roles":[{"role":"reviewer","count":1},{"role":"approver","count":1}],"separation":{"author_reviewer":true,"author_approver":true,"reviewer_approver":true}},"review":{"cadence_days":365,"next_review_date":"2027-09-01","due_soon_days":30,"timezone_policy":"date-only"}})).collect();
    write(
        root,
        "handoff.json",
        &json!({"schema_version":"forge.author-handoff/1","project":pin(root,"project.json"),"components":pin(root,"components.json"),"provenance":pin(root,"build/provenance.json"),"policies":policies}),
    );
}
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn two_policy_fixture(root: &Path) -> String {
    fixture(root);
    let mut pack = read(root, "pack.json");
    pack["policy_families"]
        .as_array_mut()
        .unwrap()
        .push(json!({"key":"second-family","title":"Second family"}));
    let review = pack["content_rights"]["review"].clone();
    pack["family_assignments"].as_array_mut().unwrap().push(json!({"key":"second-topic-family","topic_key":"independent-topic","policy_family_key":"second-family","review":review}));
    write(root, "pack.json", &pack);
    let mut project = read(root, "project.json");
    let answer_value = project["answers"][0]["value"].as_str().unwrap().to_owned();
    project["policies"].as_array_mut().unwrap().push(json!({"key":"second-policy","policy_family_key":"second-family","title":"Second synthetic draft"}));
    project["authoring_pack"] = pin(root, "pack.json");
    project["answers"][0]["authoring_pack_sha256"] =
        pin(root, "pack.json")["expected_sha256"].clone();
    let answer: forge::authoring::manifest::Answer =
        serde_json::from_value(project["answers"][0].clone()).unwrap();
    project["human_clauses"][0]["answer_refs"][0]["expected_sha256"] =
        json!(forge::authoring::manifest::answer_sha256(&answer).unwrap());
    write(root, "project.json", &project);
    extension(root);
    answer_value
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn handoff_is_explicit_exact_draft_only_self_contained_and_no_overwrite() {
    let (_tmp, root) = temp();
    let answer_value = two_policy_fixture(&root);
    exit(
        &run(
            &root,
            &[
                "author",
                "build",
                "--manifest",
                "project.json",
                "--components",
                "components.json",
                "--output-dir",
                "build",
            ],
        ),
        0,
    );
    assert!(!root.join("build/sample-policy.lifecycle.json").exists());
    handoff_request(&root, &["sample-policy", "second-policy"]);
    for dir in ["drafts-a", "drafts-b"] {
        exit(
            &run(
                &root,
                &["author", "handoff", "--manifest", "handoff.json", "--output-dir", dir, "--html"],
            ),
            0,
        );
    }
    assert_eq!(tree(&root.join("drafts-a")), tree(&root.join("drafts-b")));
    let original = tree(&root.join("drafts-a"));
    assert!(
        String::from_utf8_lossy(&original["policies/sample-policy.md"]).contains(&answer_value)
    );
    for (path, bytes) in &original {
        if !path.starts_with("policies/") {
            assert!(!String::from_utf8_lossy(bytes).contains(&answer_value), "{path}");
        }
    }
    let receipt: Value = serde_json::from_slice(&original["handoff.json"]).unwrap();
    assert_eq!(receipt["schema_version"], "forge.author-handoff-receipt/1");
    assert_eq!(receipt["records"].as_array().unwrap().len(), 2);
    // All exact source pins remain available, so rejection must be the existing
    // destination rather than a missing prior build input.
    let retry = run(
        &root,
        &["author", "handoff", "--manifest", "handoff.json", "--output-dir", "drafts-a"],
    );
    exit(&retry, 2);
    assert!(String::from_utf8_lossy(&retry.stderr).contains("output directory already exists"));
    assert_eq!(original, tree(&root.join("drafts-a")));
    std::fs::remove_dir_all(root.join("build")).unwrap();
    for key in ["sample-policy", "second-policy"] {
        let record = read(&root.join("drafts-a"), &format!("{key}.lifecycle.json"));
        assert_eq!(record["schema_version"], "forge.policy-lifecycle/2");
        assert_eq!(record["state"], "draft");
        assert!(record["history"].as_array().unwrap().is_empty());
        exit(
            &run(
                &root.join("drafts-a"),
                &["lifecycle", "check", "--record", &format!("{key}.lifecycle.json")],
            ),
            0,
        );
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn handoff_tampering_any_policy_rejects_whole_set() {
    let (_tmp, root) = temp();
    two_policy_fixture(&root);
    exit(
        &run(
            &root,
            &[
                "author",
                "build",
                "--manifest",
                "project.json",
                "--components",
                "components.json",
                "--output-dir",
                "build",
            ],
        ),
        0,
    );
    let original = tree(&root.join("build"));
    for (index, key) in ["sample-policy", "second-policy"].iter().enumerate() {
        handoff_request(&root, &["sample-policy", "second-policy"]);
        let draft_path = format!("build/policies/{key}.md");
        std::fs::write(root.join(&draft_path), "# Tampered\n").unwrap();
        // Even consciously repinning either tampered policy cannot satisfy replay.
        let mut request = read(&root, "handoff.json");
        request["policies"][index]["draft"] = pin(&root, &draft_path);
        write(&root, "handoff.json", &request);
        let destination = format!("rejected-{key}");
        exit(
            &run(
                &root,
                &["author", "handoff", "--manifest", "handoff.json", "--output-dir", &destination],
            ),
            2,
        );
        assert!(!root.join(&destination).exists());
        for policy in ["sample-policy", "second-policy"] {
            assert!(!root.join(&destination).join(format!("{policy}.lifecycle.json")).exists());
            assert!(!root.join(&destination).join(format!("policies/{policy}.md")).exists());
        }
        std::fs::write(root.join(&draft_path), &original[&format!("policies/{key}.md")]).unwrap();
        assert_eq!(tree(&root.join("build")), original);
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn optional_unresolved_marker_stays_after_its_component_heading() {
    let (_tmp, root) = temp();
    fixture(&root);
    let mut ext = extension(&root);
    let mut pack = read(&root, "pack.json");
    pack["questions"][0]["required"] = json!(false);
    write(&root, "pack.json", &pack);
    let mut project = read(&root, "project.json");
    project["authoring_pack"] = pin(&root, "pack.json");
    project["answers"] = json!([]);
    project["human_clauses"] = json!([]);
    write(&root, "project.json", &project);
    ext["project_sha256"] = pin(&root, "project.json")["expected_sha256"].clone();
    ext["instances"][0]["parameters"]["owner-role"] =
        json!({"kind":"literal","value":"explicit role","sensitivity":"internal"});
    write(&root, "components.json", &ext);
    exit(
        &run(
            &root,
            &[
                "author",
                "build",
                "--manifest",
                "project.json",
                "--components",
                "components.json",
                "--output-dir",
                "out",
            ],
        ),
        0,
    );
    let text = std::fs::read_to_string(root.join("out/policies/sample-policy.md")).unwrap();
    let heading = text.find("## Sample drafting section").unwrap();
    let marker = text.find("UNRESOLVED OPTIONAL CONTEXT").unwrap();
    let next = text.find("## Independent drafting section").unwrap();
    assert!(heading < marker && marker < next);
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn component_symlink_hardlink_and_traversal_fail_before_publication() {
    use std::os::unix::fs::symlink;
    let (_tmp, root) = temp();
    fixture(&root);
    let ext = extension(&root);
    let body = std::fs::read(root.join("component.md")).unwrap();
    std::fs::rename(root.join("component.md"), root.join("real.md")).unwrap();
    symlink("real.md", root.join("component.md")).unwrap();
    exit(
        &run(
            &root,
            &[
                "author",
                "build",
                "--manifest",
                "project.json",
                "--components",
                "components.json",
                "--output-dir",
                "symlink-rejected",
            ],
        ),
        2,
    );
    assert!(!root.join("symlink-rejected").exists());
    std::fs::remove_file(root.join("component.md")).unwrap();
    std::fs::hard_link(root.join("real.md"), root.join("component.md")).unwrap();
    exit(
        &run(
            &root,
            &[
                "author",
                "build",
                "--manifest",
                "project.json",
                "--components",
                "components.json",
                "--output-dir",
                "hardlink-rejected",
            ],
        ),
        2,
    );
    assert!(!root.join("hardlink-rejected").exists());
    std::fs::remove_file(root.join("component.md")).unwrap();
    std::fs::write(root.join("component.md"), body).unwrap();
    let mut unsafe_ext = ext;
    unsafe_ext["instances"][0]["source"]["path"] = json!("../component.md");
    write(&root, "components.json", &unsafe_ext);
    exit(
        &run(
            &root,
            &[
                "author",
                "build",
                "--manifest",
                "project.json",
                "--components",
                "components.json",
                "--output-dir",
                "traversal-rejected",
            ],
        ),
        2,
    );
    assert!(!root.join("traversal-rejected").exists());
}

#[test]
fn cli_components_paths_and_html_without_destination_are_rejected() {
    let (_tmp, root) = temp();
    fixture(&root);
    extension(&root);
    let absolute = root.join("components.json");
    for path in [
        absolute.to_str().unwrap(),
        "../components.json",
        "components\\input.json",
        "C:components.json",
    ] {
        let output = run(
            &root,
            &[
                "author",
                "plan",
                "--manifest",
                "project.json",
                "--components",
                path,
                "--output-dir",
                "rejected",
            ],
        );
        exit(&output, 2);
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("canonical project-relative path")
        );
        assert!(!root.join("rejected").exists());
    }
    impact_request(&root, "project.json", "project.json");
    for (command, manifest) in
        [("plan", "project.json"), ("build", "project.json"), ("impact", "impact.json")]
    {
        let output = run(&root, &["author", command, "--manifest", manifest, "--html"]);
        assert_eq!(output.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&output.stderr).starts_with("error:"));
        assert!(!String::from_utf8_lossy(&output.stderr).contains("Policy authoring error:"));
        assert!(String::from_utf8_lossy(&output.stderr).contains("--output-dir"));
        for artifact in ["plan.html", "provenance.html", "impact.html"] {
            assert!(!root.join(artifact).exists(), "{command}: {artifact}");
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn plan_only_html_uses_validated_legacy_and_component_reports_without_build_artifacts() {
    let (_tmp, root) = temp();
    fixture(&root);
    extension(&root);
    for (destination, schema, components) in [
        ("legacy-plan", "forge.authoring-plan/1", false),
        ("component-plan", "forge.authoring-plan/2", true),
    ] {
        let mut args = vec![
            "author",
            "plan",
            "--manifest",
            "project.json",
            "--output-dir",
            destination,
            "--html",
        ];
        if components {
            args.extend(["--components", "components.json"]);
        }
        exit(&run(&root, &args), 0);
        let files = tree(&root.join(destination));
        assert_eq!(
            files.keys().map(String::as_str).collect::<Vec<_>>(),
            vec!["plan.html", "plan.json", "plan.txt"]
        );
        let plan: Value = serde_json::from_slice(&files["plan.json"]).unwrap();
        assert_eq!(plan["schema_version"], schema);
        if components {
            assert_eq!(plan["components"].as_array().unwrap().len(), 1);
        }
        let html = String::from_utf8_lossy(&files["plan.html"]);
        assert!(html.contains(schema));
        assert!(html.contains("Human review remains pending"));
        assert!(!html.contains("Fictional draft custodian"));
        assert!(!html.contains("<script"));
    }
}

#[test]
fn component_extension_reformat_is_visible_and_drift_is_unverified() {
    let (_tmp, root) = temp();
    fixture(&root);
    let ext = extension(&root);
    std::fs::write(root.join("reformatted.json"), serde_json::to_vec(&ext).unwrap()).unwrap();
    write(
        &root,
        "impact.json",
        &json!({"schema_version":"forge.authoring-impact/1",
        "old":{"project":pin(&root,"project.json"),"components":pin(&root,"components.json")},
        "new":{"project":pin(&root,"project.json"),"components":pin(&root,"reformatted.json")}}),
    );
    let output = run(&root, &["author", "impact", "--manifest", "impact.json", "--format", "json"]);
    exit(&output, 1);
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        report["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["category"] == "component-extension-binding-changed")
    );
    assert!(
        report["findings"]
            .as_array()
            .unwrap()
            .iter()
            .all(|finding| finding["category"] != "gap-added"
                && finding["category"] != "gap-removed")
    );
    std::fs::write(root.join("component.md"), "drift").unwrap();
    let output = run(&root, &["author", "impact", "--manifest", "impact.json", "--format", "json"]);
    exit(&output, 2);
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["status"], "incomplete");
    assert!(report["sections"].as_array().unwrap().is_empty());
    assert!(String::from_utf8_lossy(&output.stdout).contains("component-input"));
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn impact_html_is_deterministic_across_directories_and_keeps_answer_values_private() {
    let (_tmp, root) = temp();
    for directory in ["left", "right"] {
        let at = root.join(directory);
        fixture(&at.join("old"));
        fixture(&at.join("new"));
        let mut project = read(&at.join("new"), "project.json");
        project["answers"][0]["value"] = json!("Different draft custodian");
        let answer: forge::authoring::manifest::Answer =
            serde_json::from_value(project["answers"][0].clone()).unwrap();
        project["human_clauses"][0]["answer_refs"][0]["expected_sha256"] =
            json!(forge::authoring::manifest::answer_sha256(&answer).unwrap());
        write(&at.join("new"), "project.json", &project);
        impact_request(&at, "old/project.json", "new/project.json");
        let output = run(
            &at,
            &[
                "author",
                "impact",
                "--manifest",
                "impact.json",
                "--format",
                "json",
                "--html",
                "--output-dir",
                "impact-view",
            ],
        );
        exit(&output, 1);
        let artifacts = tree(&at.join("impact-view"));
        assert_eq!(
            artifacts.keys().map(String::as_str).collect::<Vec<_>>(),
            vec!["impact.html", "impact.json", "impact.txt"]
        );
        assert_eq!(artifacts["impact.json"], output.stdout);
        let report: Value = serde_json::from_slice(&artifacts["impact.json"]).unwrap();
        assert_eq!(report["schema_version"], "forge.authoring-impact-report/1");
        assert_eq!(report["status"], "complete");
        assert!(!report["findings"].as_array().unwrap().is_empty());
        for (path, bytes) in &artifacts {
            let text = String::from_utf8_lossy(bytes);
            for value in ["Fictional draft custodian", "Different draft custodian"] {
                assert!(!text.contains(value), "{path}");
            }
        }
        let html = String::from_utf8_lossy(&artifacts["impact.html"]);
        assert!(html.contains("forge.authoring-impact-report/1"));
        assert!(html.contains("Human review remains pending"));
        assert!(!html.contains("<script"));
    }
    assert_eq!(tree(&root.join("left/impact-view")), tree(&root.join("right/impact-view")));
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn handoff_publishes_complete_drafts_with_exit_one_when_drafting_work_remains() {
    let (_tmp, root) = temp();
    fixture(&root);
    let mut components = extension(&root);
    let mut project = read(&root, "project.json");
    project["answers"] = json!([]);
    project["human_clauses"] = json!([]);
    write(&root, "project.json", &project);
    components["project_sha256"] = pin(&root, "project.json")["expected_sha256"].clone();
    write(&root, "components.json", &components);
    exit(
        &run(
            &root,
            &[
                "author",
                "build",
                "--manifest",
                "project.json",
                "--components",
                "components.json",
                "--output-dir",
                "build",
            ],
        ),
        1,
    );
    handoff_request(&root, &["sample-policy"]);
    let output = run(
        &root,
        &[
            "author",
            "handoff",
            "--manifest",
            "handoff.json",
            "--output-dir",
            "pending-drafts",
            "--html",
        ],
    );
    exit(&output, 1);
    assert!(String::from_utf8_lossy(&output.stdout).contains("Created draft lifecycle records"));
    let generation = tree(&root.join("pending-drafts"));
    for artifact in [
        "sample-policy.lifecycle.json",
        "handoff.json",
        "policies/sample-policy.md",
        "plan.json",
        "provenance.json",
        "components.lock.json",
        "plan.html",
        "provenance.html",
    ] {
        assert!(generation.contains_key(artifact), "{artifact}");
    }
    let plan = read(&root.join("pending-drafts"), "plan.json");
    assert_eq!(plan["plan"]["policies"][0]["state"], "blocked-context");
    let record = read(&root.join("pending-drafts"), "sample-policy.lifecycle.json");
    assert_eq!(record["state"], "draft");
    assert!(record["history"].as_array().unwrap().is_empty());
    exit(
        &run(
            &root.join("pending-drafts"),
            &["lifecycle", "check", "--record", "sample-policy.lifecycle.json"],
        ),
        0,
    );
    let retry = run(
        &root,
        &["author", "handoff", "--manifest", "handoff.json", "--output-dir", "pending-drafts"],
    );
    exit(&retry, 2);
    assert!(String::from_utf8_lossy(&retry.stderr).contains("already exists"));
    assert_eq!(tree(&root.join("pending-drafts")), generation);
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn outside_component_pins_are_classified_consistently_across_snapshot_locations() {
    for new_project in ["nested/project.json", "other/project.json"] {
        let (_tmp, root) = temp();
        fixture(&root.join("nested"));
        fixture(&root.join("other"));
        extension(&root.join("nested"));
        std::fs::copy(root.join("nested/components.json"), root.join("outside-components.json"))
            .unwrap();
        write(
            &root,
            "impact.json",
            &json!({
                "schema_version":"forge.authoring-impact/1",
                "old":{"project":pin(&root,"nested/project.json")},
                "new":{"project":pin(&root,new_project),"components":pin(&root,"outside-components.json")}
            }),
        );
        let output = run(
            &root,
            &[
                "author",
                "impact",
                "--manifest",
                "impact.json",
                "--format",
                "json",
                "--html",
                "--output-dir",
                "incomplete",
            ],
        );
        exit(&output, 2);
        let artifacts = tree(&root.join("incomplete"));
        assert_eq!(
            artifacts.keys().map(String::as_str).collect::<Vec<_>>(),
            vec!["impact.html", "impact.json", "impact.txt"]
        );
        assert_eq!(artifacts["impact.json"], output.stdout);
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["status"], "incomplete");
        assert_eq!(report["findings"][0]["category"], "component-input-unverified");
        assert_eq!(report["findings"][0]["unverified_reason"], "component-input");
        assert!(report["sections"].as_array().unwrap().is_empty());
        assert!(report["policies"].as_array().unwrap().is_empty());
        for (path, bytes) in &artifacts {
            let text = String::from_utf8_lossy(bytes);
            for private in [
                "outside-components.json",
                "nested/project.json",
                "Fictional draft custodian",
                root.to_str().unwrap(),
            ] {
                assert!(!text.contains(private), "{path}");
            }
        }
    }
}

#[test]
fn failed_old_capture_does_not_refund_unknown_bytes_to_the_new_snapshot() {
    for new_project in ["old/project.json", "new/project.json"] {
        let (_tmp, root) = temp();
        fixture(&root.join("old"));
        fixture(&root.join("new"));
        extension(&root.join("old"));
        // Both project-only snapshots are independently usable before comparison.
        for project in ["old/project.json", new_project] {
            exit(&run(&root, &["author", "plan", "--manifest", project]), 0);
        }
        // Failure occurs after loading the old project and entering its private
        // component capture, whose partial byte count is unavailable on error.
        std::fs::write(root.join("old/component.md"), "## Changed pinned source\n").unwrap();
        write(
            &root,
            "impact.json",
            &json!({
                "schema_version":"forge.authoring-impact/1",
                "old":{"project":pin(&root,"old/project.json"),"components":pin(&root,"old/components.json")},
                "new":{"project":pin(&root,new_project)}
            }),
        );
        let output =
            run(&root, &["author", "impact", "--manifest", "impact.json", "--format", "json"]);
        exit(&output, 2);
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["status"], "incomplete");
        assert_eq!(report["findings"][0]["subject_key"], "both");
        assert!(report["old_report_sha256"].is_null());
        assert!(report["new_report_sha256"].is_null());
        assert!(report["sections"].as_array().unwrap().is_empty());
        assert!(report["policies"].as_array().unwrap().is_empty());
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn html_views_accept_validated_upstream_control_ids_beyond_authored_input_string_limits() {
    let (_tmp, root) = temp();
    fixture(&root);
    let control_id = "x".repeat(32 * 1024);
    let mut framework = read(&root, "framework.json");
    framework["catalog"]["controls"].as_array_mut().unwrap().push(json!({
        "id":control_id,"title":"Synthetic long unassigned control identifier"
    }));
    write(&root, "framework.json", &framework);
    let original_applicability = read(&root, "applicability.json");
    let init = run(&root, &["applicability", "init", "--framework", "framework.json"]);
    exit(&init, 0);
    let mut applicability: Value = serde_json::from_slice(&init.stdout).unwrap();
    applicability["reviewers"] = original_applicability["reviewers"].clone();
    applicability["decisions"] = original_applicability["decisions"].clone();
    let mut decision = original_applicability["decisions"][0].clone();
    decision["control_id"] = json!(control_id);
    applicability["decisions"].as_array_mut().unwrap().push(decision);
    write(&root, "applicability.json", &applicability);
    let analysis = run(
        &root,
        &["applicability", "analyze", "--manifest", "applicability.json", "--format", "json"],
    );
    exit(&analysis, 0);
    std::fs::write(root.join("gap-report.json"), &analysis.stdout).unwrap();
    let mut pack = read(&root, "pack.json");
    pack["baseline"]["framework_sha256"] = pin(&root, "framework.json")["expected_sha256"].clone();
    pack["baseline"]["report_sha256"] = pin(&root, "gap-report.json")["expected_sha256"].clone();
    write(&root, "pack.json", &pack);
    let mut project = read(&root, "project.json");
    project["baseline"] = pack["baseline"].clone();
    project["applicability_manifest"] = pin(&root, "applicability.json");
    project["gap_report"] = pin(&root, "gap-report.json");
    project["authoring_pack"] = pin(&root, "pack.json");
    project["answers"] = json!([]);
    project["human_clauses"] = json!([]);
    write(&root, "project.json", &project);
    let reference =
        run(&root, &["author", "plan", "--manifest", "project.json", "--format", "json"]);
    exit(&reference, 1);
    let plan: Value = serde_json::from_slice(&reference.stdout).unwrap();
    assert!(plan["gaps"].as_array().unwrap().iter().any(|gap| gap["control_id"] == control_id));
    for command in ["plan", "build"] {
        let destination = format!("{command}-html");
        exit(
            &run(
                &root,
                &[
                    "author",
                    command,
                    "--manifest",
                    "project.json",
                    "--html",
                    "--output-dir",
                    &destination,
                ],
            ),
            1,
        );
        let outputs = tree(&root.join(&destination));
        assert_eq!(outputs["plan.json"], reference.stdout);
        assert!(String::from_utf8_lossy(&outputs["plan.html"]).contains(&control_id));
        if command == "build" {
            assert!(String::from_utf8_lossy(&outputs["provenance.html"]).contains(&control_id));
        }
    }
}
