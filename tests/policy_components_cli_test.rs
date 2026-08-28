//! PRD 059 reusable policy component acceptance and security tests.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_forge")).args(args).output().expect("run forge")
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn write_json(path: &Path, value: &Value) {
    std::fs::write(path, serde_json::to_vec_pretty(value).expect("serialize fixture"))
        .expect("write fixture");
}

struct Fixture {
    _directory: TempDir,
    root: PathBuf,
    manifest: PathBuf,
    source: PathBuf,
    component_manifest: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let directory = tempfile::tempdir().expect("tempdir");
        let root = directory.path().canonicalize().expect("root");
        std::fs::create_dir(root.join("components")).expect("components");
        std::fs::create_dir(root.join("build")).expect("build");
        let source = root.join("components/access-review.md");
        let component_manifest = root.join("components/access-review.json");
        let manifest = root.join("composition.json");
        let markdown = b"## Access Review\n\n1. The {{forge:param:owner-role}} reviews access every {{forge:param:interval-days}} days.\n2. Notifications: {{forge:param:channels}}. Enabled: {{forge:param:enabled}}.\n";
        std::fs::write(&source, markdown).expect("component");
        write_json(
            &component_manifest,
            &json!({
                "schema_version": "forge.policy-component/1",
                "component_key": "access-review",
                "version": "1.2.0",
                "title": "Access review",
                "owner": "security-governance",
                "status": "approved",
                "source": "access-review.md",
                "expected_sha256": hash(markdown),
                "parameters": [
                    {
                        "name": "owner-role",
                        "type": "string",
                        "required": true,
                        "constraints": {"min_length": 2, "max_length": 100, "regex": "^[A-Za-z{].+"}
                    },
                    {
                        "name": "interval-days",
                        "type": "integer",
                        "default": 90,
                        "constraints": {"minimum": 1, "maximum": 365}
                    },
                    {
                        "name": "channels",
                        "type": "string-list",
                        "default": ["email"],
                        "constraints": {"min_items": 1, "max_items": 3}
                    },
                    {
                        "name": "enabled",
                        "type": "boolean",
                        "default": true
                    }
                ]
            }),
        );
        write_json(
            &manifest,
            &json!({
                "schema_version": "forge.policy-composition/1",
                "project_root": ".",
                "policy_key": "access-policy",
                "title": "Access & Identity Policy",
                "version": "2.0.0",
                "outputs": {
                    "markdown": "build/policy.md",
                    "lock": "build/policy.lock.json",
                    "provenance": "build/policy.provenance.json"
                },
                "components": [
                    {
                        "instance_key": "quarterly-review",
                        "component_manifest": "components/access-review.json",
                        "parameters": {
                            "owner-role": "IAM *owners*",
                            "channels": ["email", "ticket"]
                        }
                    },
                    {
                        "instance_key": "placeholder-data-review",
                        "component_manifest": "components/access-review.json",
                        "parameters": {
                            "owner-role": "{{forge:param:interval-days}}",
                            "interval-days": 30
                        }
                    }
                ]
            }),
        );
        Self { _directory: directory, root, manifest, source, component_manifest }
    }

    fn output(&self, name: &str) -> PathBuf {
        self.root.join("build").join(name)
    }

    fn compose(&self, validate: bool) -> Output {
        let mut args =
            vec!["policy", "compose", "--manifest", self.manifest.to_str().expect("manifest path")];
        if validate {
            args.push("--validate");
        }
        run(&args)
    }
}

#[test]
fn compose_emits_deterministic_lock_and_complete_distinct_provenance() {
    let fixture = Fixture::new();
    let result = fixture.compose(true);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));

    let markdown_path = fixture.output("policy.md");
    let lock_path = fixture.output("policy.lock.json");
    let provenance_path = fixture.output("policy.provenance.json");
    let markdown = std::fs::read_to_string(&markdown_path).expect("markdown");
    assert!(markdown.starts_with("# Access & Identity Policy\n\n## Access Review"));
    assert!(markdown.contains(r"IAM \*owners\* reviews access every 90 days"));
    assert!(markdown.contains(r"\{\{forge:param:interval\-days\}\} reviews access every 30 days"));
    assert_eq!(markdown.matches("## Access Review").count(), 2);

    let lock: Value = serde_json::from_slice(&std::fs::read(lock_path).unwrap()).unwrap();
    assert_eq!(lock["schema_version"], "forge.policy-composition-lock/1");
    assert_eq!(lock["components"][0]["instance_key"], "quarterly-review");
    assert_eq!(lock["components"][1]["instance_key"], "placeholder-data-review");
    assert!(lock.to_string().contains("parameter_value_sha256"));
    assert!(!lock.to_string().contains("IAM *owners*"));

    let provenance: Value =
        serde_json::from_slice(&std::fs::read(provenance_path).unwrap()).unwrap();
    assert_eq!(provenance["schema_version"], "forge.policy-composition-provenance/1");
    let spans = provenance["spans"].as_array().expect("spans");
    assert!(spans.iter().any(|span| {
        span["origin"]["kind"] == "parameter"
            && span["origin"]["instance_key"] == "quarterly-review"
            && span["origin"]["parameter_name"] == "owner-role"
    }));
    assert!(spans.iter().any(|span| span["origin"]["instance_key"] == "placeholder-data-review"));
    assert!(!provenance.to_string().contains("IAM *owners*"));

    for (line_number, line) in markdown.lines().enumerate().filter(|(_, line)| !line.is_empty()) {
        let mut ranges = spans
            .iter()
            .filter(|span| span["output"]["line"] == line_number + 1)
            .map(|span| {
                (
                    usize::try_from(span["output"]["start_column"].as_u64().unwrap()).unwrap(),
                    usize::try_from(span["output"]["end_column"].as_u64().unwrap()).unwrap(),
                )
            })
            .collect::<Vec<_>>();
        ranges.sort_unstable();
        assert!(!ranges.is_empty(), "line {} has no provenance: {line}", line_number + 1);
        assert_eq!(ranges[0].0, 1, "line {} starts unmapped", line_number + 1);
        for pair in ranges.windows(2) {
            assert_eq!(pair[0].1, pair[1].0, "line {} has a provenance gap", line_number + 1);
        }
        assert_eq!(ranges.last().unwrap().1, line.chars().count() + 1);
    }

    let first = [
        std::fs::read(&markdown_path).unwrap(),
        std::fs::read(fixture.output("policy.lock.json")).unwrap(),
        std::fs::read(fixture.output("policy.provenance.json")).unwrap(),
    ];
    assert!(fixture.compose(false).status.success());
    let second = [
        std::fs::read(markdown_path).unwrap(),
        std::fs::read(fixture.output("policy.lock.json")).unwrap(),
        std::fs::read(fixture.output("policy.provenance.json")).unwrap(),
    ];
    assert_eq!(first, second);
}

#[test]
fn pin_drift_exits_two_and_never_creates_or_replaces_outputs() {
    let fixture = Fixture::new();
    for name in ["policy.md", "policy.lock.json", "policy.provenance.json"] {
        std::fs::write(fixture.output(name), format!("old-{name}")).unwrap();
    }
    std::fs::write(&fixture.source, "## Access Review\n\nChanged.\n").unwrap();
    let result = fixture.compose(false);
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("SHA-256 mismatch"));
    for name in ["policy.md", "policy.lock.json", "policy.provenance.json"] {
        assert_eq!(std::fs::read_to_string(fixture.output(name)).unwrap(), format!("old-{name}"));
    }
}

#[test]
fn compose_check_and_component_check_are_read_only() {
    let fixture = Fixture::new();
    let component =
        run(&["policy", "component", "check", fixture.component_manifest.to_str().unwrap()]);
    assert!(component.status.success(), "{}", String::from_utf8_lossy(&component.stderr));
    let composition =
        run(&["policy", "compose", "check", "--manifest", fixture.manifest.to_str().unwrap()]);
    assert!(composition.status.success(), "{}", String::from_utf8_lossy(&composition.stderr));
    assert!(!fixture.output("policy.md").exists());
    assert!(!fixture.output("policy.lock.json").exists());
    assert!(!fixture.output("policy.provenance.json").exists());
}

#[test]
fn unsupported_placeholder_context_and_output_alias_fail_closed() {
    let fixture = Fixture::new();
    let fenced = b"## Access Review\n\n```text\n{{forge:param:owner-role}}\n```\n";
    std::fs::write(&fixture.source, fenced).unwrap();
    let mut sidecar: Value =
        serde_json::from_slice(&std::fs::read(&fixture.component_manifest).unwrap()).unwrap();
    sidecar["expected_sha256"] = json!(hash(fenced));
    write_json(&fixture.component_manifest, &sidecar);
    let result = fixture.compose(false);
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("fenced code block"));

    let mut composition: Value =
        serde_json::from_slice(&std::fs::read(&fixture.manifest).unwrap()).unwrap();
    composition["outputs"]["markdown"] = json!("composition.json");
    write_json(&fixture.manifest, &composition);
    let alias = fixture.compose(false);
    assert_eq!(alias.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&alias.stderr).contains("aliases"));
}

#[test]
fn byte_identical_projects_in_different_absolute_directories_match() {
    let first = Fixture::new();
    let second = Fixture::new();
    assert!(first.compose(false).status.success());
    assert!(second.compose(false).status.success());
    for name in ["policy.md", "policy.lock.json", "policy.provenance.json"] {
        let first_bytes = std::fs::read(first.output(name)).unwrap();
        let second_bytes = std::fs::read(second.output(name)).unwrap();
        assert_eq!(first_bytes, second_bytes, "{name} differs by absolute directory");
        let rendered = String::from_utf8(first_bytes).unwrap();
        assert!(!rendered.contains(first.root.to_str().unwrap()));
        assert!(!rendered.contains(second.root.to_str().unwrap()));
    }
}

#[test]
fn impact_report_is_explicit_deterministic_and_does_not_refresh_locks() {
    let fixture = Fixture::new();
    assert!(fixture.compose(false).status.success());
    let lock_before = std::fs::read(fixture.output("policy.lock.json")).unwrap();
    std::fs::write(&fixture.source, "## Access Review\n\nUpdated shared clause.\n").unwrap();
    let report = fixture.root.join("impact.json");
    let result = run(&[
        "policy",
        "component",
        "impact",
        "--component-key",
        "access-review",
        "--manifest",
        fixture.manifest.to_str().unwrap(),
        "--output",
        report.to_str().unwrap(),
    ]);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let impact: Value = serde_json::from_slice(&std::fs::read(report).unwrap()).unwrap();
    assert_eq!(impact["schema_version"], "forge.policy-component-impact/1");
    assert_eq!(impact["affected_policy_count"], 1);
    assert_eq!(impact["affected_instance_count"], 2);
    assert_eq!(impact["affected_instances"][0]["pin_matches"], false);
    assert_eq!(impact["affected_instances"][1]["pin_matches"], false);
    assert_eq!(std::fs::read(fixture.output("policy.lock.json")).unwrap(), lock_before);
}

#[test]
fn scaffold_emits_a_draft_pinned_sidecar_without_approval() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("incident-reporting.md");
    let output = directory.path().join("incident-reporting.json");
    std::fs::write(&source, "## Incident Reporting\n\nReport incidents promptly.\n").unwrap();
    let result = run(&[
        "policy",
        "component",
        "scaffold",
        source.to_str().unwrap(),
        "--component-key",
        "incident-reporting",
        "--version",
        "0.1.0",
        "--title",
        "Incident reporting",
        "--owner",
        "security",
        "--output",
        output.to_str().unwrap(),
    ]);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let sidecar: Value = serde_json::from_slice(&std::fs::read(&output).unwrap()).unwrap();
    assert_eq!(sidecar["status"], "draft");
    assert_eq!(sidecar["source"], "incident-reporting.md");
    assert_eq!(sidecar["expected_sha256"], hash(&std::fs::read(source).unwrap()));
    let check = run(&["policy", "component", "check", output.to_str().unwrap()]);
    assert!(check.status.success(), "{}", String::from_utf8_lossy(&check.stderr));
}

#[test]
fn parameter_completeness_and_constraint_failures_are_actionable() {
    let fixture = Fixture::new();
    let original: Value =
        serde_json::from_slice(&std::fs::read(&fixture.manifest).unwrap()).unwrap();

    let mut missing = original.clone();
    missing["components"][0]["parameters"].as_object_mut().unwrap().remove("owner-role");
    write_json(&fixture.manifest, &missing);
    let result = fixture.compose(false);
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("missing required parameter"));

    let mut unknown = original.clone();
    unknown["components"][0]["parameters"]["unknown-value"] = json!("x");
    write_json(&fixture.manifest, &unknown);
    let result = fixture.compose(false);
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("unknown parameter"));

    let mut invalid_range = original.clone();
    invalid_range["components"][0]["parameters"]["interval-days"] = json!(366);
    write_json(&fixture.manifest, &invalid_range);
    let result = fixture.compose(false);
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("integer range"));

    let mut wrong_type = original.clone();
    wrong_type["components"][0]["parameters"]["enabled"] = json!("true");
    write_json(&fixture.manifest, &wrong_type);
    let result = fixture.compose(false);
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("wrong parameter type"));

    let mut duplicate_instance = original;
    duplicate_instance["components"][1]["instance_key"] = json!("quarterly-review");
    write_json(&fixture.manifest, &duplicate_instance);
    let result = fixture.compose(false);
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("duplicates instance key"));
}

#[test]
fn supplied_but_unused_declared_values_are_rejected() {
    let fixture = Fixture::new();
    let mut sidecar: Value =
        serde_json::from_slice(&std::fs::read(&fixture.component_manifest).unwrap()).unwrap();
    sidecar["parameters"].as_array_mut().unwrap().push(json!({
        "name": "review-note",
        "type": "string"
    }));
    write_json(&fixture.component_manifest, &sidecar);
    let mut composition: Value =
        serde_json::from_slice(&std::fs::read(&fixture.manifest).unwrap()).unwrap();
    composition["components"][0]["parameters"]["review-note"] = json!("not referenced");
    write_json(&fixture.manifest, &composition);
    let result = fixture.compose(false);
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("unused parameter"));
}

#[test]
fn oscal_trace_can_resolve_assembled_lines_to_component_origins() {
    let fixture = Fixture::new();
    assert!(fixture.compose(false).status.success());
    let artifact = fixture.output("catalog.json");
    let convert = run(&[
        "convert",
        fixture.output("policy.md").to_str().unwrap(),
        "--strategy",
        "catalog",
        "--output",
        artifact.to_str().unwrap(),
    ]);
    assert!(convert.status.success(), "{}", String::from_utf8_lossy(&convert.stderr));
    let trace_output = fixture.output("trace.txt");
    let trace = run(&[
        "trace",
        artifact.to_str().unwrap(),
        "--source",
        fixture.output("policy.md").to_str().unwrap(),
        "--composition-provenance",
        fixture.output("policy.provenance.json").to_str().unwrap(),
        "--output",
        trace_output.to_str().unwrap(),
    ]);
    assert!(trace.status.success(), "{}", String::from_utf8_lossy(&trace.stderr));
    let report = std::fs::read_to_string(trace_output).unwrap();
    assert!(report.contains("Composition provenance:"));
    assert!(report.contains("quarterly-review"));
    assert!(report.contains("components/access-review.md"));
}

#[cfg(unix)]
#[test]
fn symlinked_component_source_is_rejected() {
    use std::os::unix::fs::symlink;

    let fixture = Fixture::new();
    let outside = fixture.root.parent().unwrap().join("outside-policy-component.md");
    std::fs::write(&outside, "## Outside\n").unwrap();
    std::fs::remove_file(&fixture.source).unwrap();
    symlink(&outside, &fixture.source).unwrap();
    let result = fixture.compose(false);
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("symbolic link"));
}
