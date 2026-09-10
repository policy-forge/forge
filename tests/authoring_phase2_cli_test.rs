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
    for policy in graph["policies"].as_array().unwrap() {
        let mut end = 0;
        for span in policy["spans"].as_array().unwrap() {
            assert_eq!(span["output"]["start"].as_u64().unwrap(), end);
            end = span["output"]["end"].as_u64().unwrap();
        }
        assert_eq!(end, policy["byte_length"].as_u64().unwrap());
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
    project["answers"] = json!([]);
    project["human_clauses"] = json!([]);
    write(&root, "project.json", &project);
    // Retain the exact formerly supplied answer pin to represent missing context.
    ext["project_sha256"] = pin(&root, "project.json")["expected_sha256"].clone();
    ext["instances"][0]["parameters"] = json!({"owner-role":{"kind":"answer","question_key":"sample-question","answer_key":"sample-answer","expected_sha256":"390631cd0391e5240b4256d1e886859c2be21e752e18770781d9fc6937414a97"}});
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
#[test]
fn handoff_is_explicit_exact_draft_only_self_contained_and_no_overwrite() {
    let (_tmp, root) = temp();
    fixture(&root);
    let mut pack = read(&root, "pack.json");
    pack["policy_families"]
        .as_array_mut()
        .unwrap()
        .push(json!({"key":"second-family","title":"Second family"}));
    let review = pack["content_rights"]["review"].clone();
    pack["family_assignments"].as_array_mut().unwrap().push(json!({"key":"second-topic-family","topic_key":"independent-topic","policy_family_key":"second-family","review":review}));
    write(&root, "pack.json", &pack);
    let mut project = read(&root, "project.json");
    project["policies"].as_array_mut().unwrap().push(json!({"key":"second-policy","policy_family_key":"second-family","title":"Second synthetic draft"}));
    project["authoring_pack"] = pin(&root, "pack.json");
    project["answers"][0]["authoring_pack_sha256"] =
        pin(&root, "pack.json")["expected_sha256"].clone();
    let answer: forge::authoring::manifest::Answer =
        serde_json::from_value(project["answers"][0].clone()).unwrap();
    project["human_clauses"][0]["answer_refs"][0]["expected_sha256"] =
        json!(forge::authoring::manifest::answer_sha256(&answer).unwrap());
    write(&root, "project.json", &project);
    extension(&root);
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
    exit(
        &run(
            &root,
            &["author", "handoff", "--manifest", "handoff.json", "--output-dir", "drafts-a"],
        ),
        2,
    );
    assert_eq!(original, tree(&root.join("drafts-a")));
    std::fs::remove_dir_all(root.join("build")).unwrap();
    for key in ["sample-policy", "second-policy"] {
        let record = read(&root.join("drafts-a"), &format!("{key}.lifecycle.json"));
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
    let before = tree(&root.join("drafts-a"));
    exit(
        &run(
            &root,
            &["author", "handoff", "--manifest", "handoff.json", "--output-dir", "drafts-a"],
        ),
        2,
    );
    assert_eq!(tree(&root.join("drafts-a")), before);
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn handoff_tampering_any_policy_rejects_whole_set() {
    let (_tmp, root) = temp();
    fixture(&root);
    extension(&root);
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
    handoff_request(&root, &["sample-policy"]);
    std::fs::write(root.join("build/policies/sample-policy.md"), "# Tampered\n").unwrap();
    // Even consciously repinning tampered bytes cannot satisfy exact source replay.
    let mut request = read(&root, "handoff.json");
    request["policies"][0]["draft"] = pin(&root, "build/policies/sample-policy.md");
    write(&root, "handoff.json", &request);
    exit(
        &run(
            &root,
            &["author", "handoff", "--manifest", "handoff.json", "--output-dir", "rejected"],
        ),
        2,
    );
    assert!(!root.join("rejected").exists());
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

#[cfg(unix)]
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
            &["author", "plan", "--manifest", "project.json", "--components", "components.json"],
        ),
        2,
    );
    std::fs::remove_file(root.join("component.md")).unwrap();
    std::fs::hard_link(root.join("real.md"), root.join("component.md")).unwrap();
    exit(
        &run(
            &root,
            &["author", "plan", "--manifest", "project.json", "--components", "components.json"],
        ),
        2,
    );
    std::fs::remove_file(root.join("component.md")).unwrap();
    std::fs::write(root.join("component.md"), body).unwrap();
    let mut unsafe_ext = ext;
    unsafe_ext["instances"][0]["source"]["path"] = json!("../component.md");
    write(&root, "components.json", &unsafe_ext);
    exit(
        &run(
            &root,
            &["author", "plan", "--manifest", "project.json", "--components", "components.json"],
        ),
        2,
    );
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
