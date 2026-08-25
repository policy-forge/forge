use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{Value, json};
use tempfile::TempDir;

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_forge")).args(args).output().expect("run forge")
}

fn run_in(directory: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_forge"))
        .current_dir(directory)
        .args(args)
        .output()
        .expect("run forge")
}

fn catalog(uuid: &str, title: &str) -> Value {
    json!({
        "catalog": {
            "uuid": uuid,
            "metadata": {
                "title": title,
                "last-modified": "2026-08-25T00:00:00Z",
                "version": "1.0.0",
                "oscal-version": "1.2.3"
            },
            "groups": []
        }
    })
}

fn write_json(path: &Path, value: &Value) {
    std::fs::write(path, serde_json::to_vec_pretty(value).expect("serialize fixture"))
        .expect("write fixture");
}

struct Fixture {
    _dir: TempDir,
    source: PathBuf,
    artifact: PathBuf,
    record: PathBuf,
}

impl Fixture {
    fn new(name: &str, shared_actor: bool) -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().canonicalize().expect("canonical tempdir");
        let source = root.join(format!("{name}.md"));
        let artifact = root.join(format!("{name}.json"));
        let record = root.join(format!("{name}-lifecycle.json"));
        std::fs::write(&source, format!("# {name}\n\nPolicy text.\n")).expect("source");
        write_json(&artifact, &catalog("11111111-1111-4111-8111-111111111111", name));
        let mut args = vec![
            "lifecycle",
            "init",
            "--source",
            source.to_str().expect("source path"),
            "--artifact",
            artifact.to_str().expect("artifact path"),
            "--output",
            record.to_str().expect("record path"),
            "--policy-key",
            name,
            "--version-key",
            "v1",
            "--title",
            name,
            "--owner",
            "owner",
            "--party",
            "owner=owner",
            "--next-review",
            "2026-09-24",
        ];
        if shared_actor {
            args.extend(["--party", "same=author,reviewer,approver"]);
        } else {
            args.extend(["--party", "reviewer=reviewer", "--party", "approver=approver"]);
        }
        let result = run(&args);
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
        Self { _dir: dir, source, artifact, record }
    }

    fn transition(&self, to: &str, actor: &str, role: &str, at: &str, extra: &[&str]) -> Output {
        let mut args = vec![
            "lifecycle",
            "transition",
            "--record",
            self.record.to_str().expect("record path"),
            "--to",
            to,
            "--actor",
            actor,
            "--role",
            role,
            "--at",
            at,
            "--rationale",
            "Reviewed lifecycle action",
            "--apply",
        ];
        args.extend_from_slice(extra);
        run(&args)
    }

    fn approve(&self) {
        let review =
            self.transition("in-review", "reviewer", "reviewer", "2026-08-25T10:00:00Z", &[]);
        assert!(review.status.success(), "{}", String::from_utf8_lossy(&review.stderr));
        let approve =
            self.transition("approved", "approver", "approver", "2026-08-25T11:00:00Z", &[]);
        assert!(approve.status.success(), "{}", String::from_utf8_lossy(&approve.stderr));
    }
}

#[test]
fn draft_review_approval_retains_deterministic_evidence() {
    let fixture = Fixture::new("access-policy", false);
    fixture.approve();

    let record: Value =
        serde_json::from_slice(&std::fs::read(&fixture.record).expect("record")).expect("json");
    assert_eq!(record["state"], "approved");
    assert_eq!(record["history"].as_array().expect("history").len(), 2);
    assert_eq!(record["history"][0]["sequence"], 1);
    assert_eq!(record["history"][1]["sequence"], 2);
    assert_eq!(record["history"][0]["fingerprints"], record["history"][1]["fingerprints"]);
    for event in record["history"].as_array().expect("history") {
        uuid::Uuid::parse_str(event["event_id"].as_str().expect("event id")).expect("uuid");
    }
}

#[test]
fn configurable_role_counts_accept_distinct_assertions() {
    let fixture = Fixture::new("multi-review-policy", false);
    let mut record: Value =
        serde_json::from_slice(&std::fs::read(&fixture.record).expect("record")).expect("json");
    record["parties"].as_array_mut().expect("parties").push(json!({
        "key": "reviewer-2",
        "roles": ["reviewer"]
    }));
    record["approval_policy"]["required_roles"][0]["count"] = json!(2);
    std::fs::write(&fixture.record, serde_json::to_vec_pretty(&record).expect("serialize record"))
        .expect("write record");

    let review = fixture.transition(
        "in-review",
        "reviewer",
        "reviewer",
        "2026-08-25T10:00:00Z",
        &["--assertion", "reviewer-2=reviewer"],
    );
    assert!(review.status.success(), "{}", String::from_utf8_lossy(&review.stderr));
    let approve =
        fixture.transition("approved", "approver", "approver", "2026-08-25T11:00:00Z", &[]);
    assert!(approve.status.success(), "{}", String::from_utf8_lossy(&approve.stderr));
}

#[test]
fn approved_byte_drift_is_action_required_without_policy_prose() {
    let fixture = Fixture::new("drift-policy", false);
    fixture.approve();
    std::fs::write(&fixture.source, "# drift-policy\n\nChanged secret prose.\n").expect("drift");

    let result = run(&[
        "lifecycle",
        "status",
        "--record",
        fixture.record.to_str().expect("record"),
        "--as-of",
        "2026-08-25",
        "--format",
        "json",
    ]);
    assert_eq!(result.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&result.stdout).expect("status json");
    assert_eq!(report[0]["derived_status"], "approved-drifted");
    assert!(!String::from_utf8_lossy(&result.stdout).contains("Changed secret prose"));
}

#[test]
fn approved_generated_artifact_drift_is_action_required() {
    let fixture = Fixture::new("artifact-drift-policy", false);
    fixture.approve();
    write_json(
        &fixture.artifact,
        &catalog("11111111-1111-4111-8111-111111111111", "Changed artifact title"),
    );

    let result = run(&[
        "lifecycle",
        "status",
        "--record",
        fixture.record.to_str().expect("record"),
        "--as-of",
        "2026-08-25",
        "--format",
        "json",
    ]);
    assert_eq!(result.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&result.stdout).expect("status json");
    assert_eq!(report[0]["derived_status"], "approved-drifted");
}

#[test]
fn separation_failure_and_retired_terminal_leave_record_unchanged() {
    let fixture = Fixture::new("separation-policy", true);
    let review = fixture.transition("in-review", "same", "reviewer", "2026-08-25T10:00:00Z", &[]);
    assert!(review.status.success(), "{}", String::from_utf8_lossy(&review.stderr));

    let mut record: Value =
        serde_json::from_slice(&std::fs::read(&fixture.record).expect("record")).expect("json");
    record["approval_policy"]["separation"]["reviewer_approver"] = json!(true);
    std::fs::write(&fixture.record, serde_json::to_vec_pretty(&record).expect("serialize record"))
        .expect("write record");
    let before = std::fs::read(&fixture.record).expect("before");
    let rejected = fixture.transition("approved", "same", "approver", "2026-08-25T11:00:00Z", &[]);
    assert_eq!(rejected.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("reviewer/approver separation"));
    assert_eq!(std::fs::read(&fixture.record).expect("after"), before);

    record["approval_policy"]["separation"]["reviewer_approver"] = json!(false);
    std::fs::write(&fixture.record, serde_json::to_vec_pretty(&record).expect("serialize record"))
        .expect("write record");
    let retired = fixture.transition("retired", "same", "approver", "2026-08-25T12:00:00Z", &[]);
    assert!(retired.status.success(), "{}", String::from_utf8_lossy(&retired.stderr));
    let retired_before = std::fs::read(&fixture.record).expect("retired");
    let terminal = fixture.transition("draft", "same", "author", "2026-08-25T13:00:00Z", &[]);
    assert_eq!(terminal.status.code(), Some(2));
    assert_eq!(std::fs::read(&fixture.record).expect("terminal"), retired_before);
}

#[test]
fn explicit_date_status_is_byte_deterministic_and_due_soon_boundary_is_inclusive() {
    let fixture = Fixture::new("date-policy", false);
    let args = [
        "lifecycle",
        "status",
        "--record",
        fixture.record.to_str().expect("record"),
        "--as-of",
        "2026-08-25",
        "--format",
        "json",
        "--gate",
        "none",
    ];
    let first = run(&args);
    let second = run(&args);
    assert!(first.status.success());
    assert_eq!(first.stdout, second.stdout);
    let report: Value = serde_json::from_slice(&first.stdout).expect("json");
    assert_eq!(report[0]["derived_status"], "due-soon");

    let overdue = run(&[
        "lifecycle",
        "status",
        "--record",
        fixture.record.to_str().expect("record"),
        "--as-of",
        "2026-09-25",
        "--format",
        "json",
        "--gate",
        "none",
    ]);
    assert!(overdue.status.success());
    let overdue_report: Value = serde_json::from_slice(&overdue.stdout).expect("overdue json");
    assert_eq!(overdue_report[0]["derived_status"], "overdue");
}

#[test]
fn check_is_schedule_neutral_and_accepts_a_relative_record_path() {
    let fixture = Fixture::new("relative-check-policy", false);
    let directory = fixture.record.parent().expect("record parent");
    let result = run_in(
        directory,
        &[
            "lifecycle",
            "check",
            "--record",
            fixture.record.file_name().and_then(|name| name.to_str()).expect("record name"),
            "--format",
            "json",
        ],
    );
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let reports: Value = serde_json::from_slice(&result.stdout).expect("check json");
    assert_eq!(reports.as_array().expect("stable report array").len(), 1);
    assert_eq!(reports[0]["derived_status"], "draft");
    assert_eq!(reports[0]["as_of"], Value::Null);
    assert_eq!(reports[0]["blockers"], json!([]));
}

#[test]
fn queue_groups_deterministically_by_owner_and_due_date() {
    let first = Fixture::new("queue-policy-a", false);
    let second = Fixture::new("queue-policy-b", false);
    let args = [
        "lifecycle",
        "queue",
        "--record",
        second.record.to_str().expect("second"),
        "--record",
        first.record.to_str().expect("first"),
        "--as-of",
        "2026-08-25",
        "--format",
        "json",
        "--gate",
        "none",
    ];
    let first_run = run(&args);
    let second_run = run(&args);
    assert!(first_run.status.success(), "{}", String::from_utf8_lossy(&first_run.stderr));
    assert_eq!(first_run.stdout, second_run.stdout);
    let queue: Value = serde_json::from_slice(&first_run.stdout).expect("queue json");
    assert_eq!(queue["schema_version"], "forge.policy-lifecycle-queue/1");
    assert_eq!(queue["groups"].as_array().expect("groups").len(), 1);
    assert_eq!(queue["groups"][0]["owner_key"], "owner");
    assert_eq!(queue["groups"][0]["next_review_date"], "2026-09-24");
    assert_eq!(queue["groups"][0]["items"][0]["policy_key"], "queue-policy-a");
    assert_eq!(queue["groups"][0]["items"][1]["policy_key"], "queue-policy-b");
}

#[test]
fn impact_finding_ids_are_bounded_review_reasons() {
    let fixture = Fixture::new("impact-policy", false);
    fixture.approve();
    let review = fixture.transition(
        "in-review",
        "reviewer",
        "reviewer",
        "2026-08-25T12:00:00Z",
        &[
            "--impact-finding-id",
            "framework-impact-002",
            "--impact-finding-id",
            "framework-impact-001",
            "--impact-finding-id",
            "framework-impact-002",
        ],
    );
    assert!(review.status.success(), "{}", String::from_utf8_lossy(&review.stderr));
    let record: Value =
        serde_json::from_slice(&std::fs::read(&fixture.record).expect("record")).expect("json");
    assert_eq!(
        record["history"][2]["impact_finding_ids"],
        json!(["framework-impact-001", "framework-impact-002"])
    );
    let status = run(&[
        "lifecycle",
        "status",
        "--record",
        fixture.record.to_str().expect("record"),
        "--as-of",
        "2026-08-25",
        "--format",
        "json",
        "--gate",
        "none",
    ]);
    assert!(status.status.success());
    let report: Value = serde_json::from_slice(&status.stdout).expect("status json");
    assert_eq!(
        report[0]["impact_finding_ids"],
        json!(["framework-impact-001", "framework-impact-002"])
    );

    let before = std::fs::read(&fixture.record).expect("before invalid transition");
    let rejected = fixture.transition(
        "approved",
        "approver",
        "approver",
        "2026-08-25T13:00:00Z",
        &["--impact-finding-id", "framework-impact-003"],
    );
    assert_eq!(rejected.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&rejected.stderr).contains("only valid when entering in-review")
    );
    assert_eq!(std::fs::read(&fixture.record).expect("after rejection"), before);
}

#[test]
fn unsigned_attestation_is_deterministic_and_refuses_drift() {
    let fixture = Fixture::new("attestation-policy", false);
    fixture.approve();
    let args = ["lifecycle", "attest", "--record", fixture.record.to_str().expect("record")];
    let first = run(&args);
    let second = run(&args);
    assert!(first.status.success(), "{}", String::from_utf8_lossy(&first.stderr));
    assert_eq!(first.stdout, second.stdout);
    let attestation: Value = serde_json::from_slice(&first.stdout).expect("attestation json");
    assert_eq!(attestation["schema_version"], "forge.policy-approval-attestation/1");
    assert_eq!(attestation["unsigned"], true);
    assert_eq!(attestation["assertions"].as_array().expect("assertions").len(), 2);
    assert!(attestation.get("rationale").is_none());
    assert!(!String::from_utf8_lossy(&first.stdout).contains("Policy text"));

    std::fs::write(&fixture.source, "# attestation-policy\n\nChanged bytes.\n").expect("drift");
    let drifted = run(&args);
    assert_eq!(drifted.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&drifted.stderr).contains("approved-drifted"));
}

#[test]
fn closed_schema_rejects_unknown_and_duplicate_keys() {
    let fixture = Fixture::new("schema-policy", false);
    let mut value: Value =
        serde_json::from_slice(&std::fs::read(&fixture.record).expect("record")).expect("json");
    value["unknown"] = json!(true);
    std::fs::write(&fixture.record, serde_json::to_vec(&value).expect("serialize")).expect("write");
    let unknown =
        run(&["lifecycle", "check", "--record", fixture.record.to_str().expect("record")]);
    assert_eq!(unknown.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&unknown.stderr).contains("unknown field"));

    std::fs::write(
        &fixture.record,
        r#"{"schema_version":"forge.policy-lifecycle/1","schema_version":"forge.policy-lifecycle/1"}"#,
    )
    .expect("write duplicate");
    let duplicate =
        run(&["lifecycle", "check", "--record", fixture.record.to_str().expect("record")]);
    assert_eq!(duplicate.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&duplicate.stderr).contains("duplicate object key"));
}

#[test]
fn portfolio_check_rejects_supersession_cycle() {
    let first = Fixture::new("policy-a", false);
    let second = Fixture::new("policy-b", false);
    first.approve();
    second.approve();
    let first_supersede = first.transition(
        "superseded",
        "approver",
        "approver",
        "2026-08-25T12:00:00Z",
        &["--replacement-policy-key", "policy-b", "--replacement-version-key", "v1"],
    );
    assert!(
        first_supersede.status.success(),
        "{}",
        String::from_utf8_lossy(&first_supersede.stderr)
    );
    let second_supersede = second.transition(
        "superseded",
        "approver",
        "approver",
        "2026-08-25T13:00:00Z",
        &["--replacement-policy-key", "policy-a", "--replacement-version-key", "v1"],
    );
    assert!(
        second_supersede.status.success(),
        "{}",
        String::from_utf8_lossy(&second_supersede.stderr)
    );

    let result = run(&[
        "lifecycle",
        "check",
        "--record",
        first.record.to_str().expect("first"),
        "--record",
        second.record.to_str().expect("second"),
    ]);
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("supersession cycle"));
}

#[test]
fn superseded_record_can_retire_without_losing_replacement_evidence() {
    let original = Fixture::new("retired-original", false);
    let replacement = Fixture::new("retired-replacement", false);
    original.approve();
    replacement.approve();
    let superseded = original.transition(
        "superseded",
        "approver",
        "approver",
        "2026-08-25T12:00:00Z",
        &["--replacement-policy-key", "retired-replacement", "--replacement-version-key", "v1"],
    );
    assert!(superseded.status.success(), "{}", String::from_utf8_lossy(&superseded.stderr));
    let retired =
        original.transition("retired", "approver", "approver", "2026-08-25T13:00:00Z", &[]);
    assert!(retired.status.success(), "{}", String::from_utf8_lossy(&retired.stderr));

    let record: Value =
        serde_json::from_slice(&std::fs::read(&original.record).expect("record")).expect("json");
    assert_eq!(record["state"], "retired");
    assert_eq!(record["replaced_by"]["policy_key"], "retired-replacement");
    let check = run(&[
        "lifecycle",
        "check",
        "--record",
        original.record.to_str().expect("original"),
        "--record",
        replacement.record.to_str().expect("replacement"),
    ]);
    assert!(check.status.success(), "{}", String::from_utf8_lossy(&check.stderr));
}

#[test]
fn portfolio_check_uses_the_replacements_latest_approval() {
    let original = Fixture::new("chronology-original", false);
    let replacement = Fixture::new("chronology-replacement", false);
    original.approve();
    replacement.approve();
    let superseded = original.transition(
        "superseded",
        "approver",
        "approver",
        "2026-08-25T11:30:00Z",
        &["--replacement-policy-key", "chronology-replacement", "--replacement-version-key", "v1"],
    );
    assert!(superseded.status.success(), "{}", String::from_utf8_lossy(&superseded.stderr));
    let review =
        replacement.transition("in-review", "reviewer", "reviewer", "2026-08-25T12:00:00Z", &[]);
    assert!(review.status.success(), "{}", String::from_utf8_lossy(&review.stderr));
    let approved =
        replacement.transition("approved", "approver", "approver", "2026-08-25T13:00:00Z", &[]);
    assert!(approved.status.success(), "{}", String::from_utf8_lossy(&approved.stderr));

    let check = run(&[
        "lifecycle",
        "check",
        "--record",
        original.record.to_str().expect("original"),
        "--record",
        replacement.record.to_str().expect("replacement"),
    ]);
    assert_eq!(check.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&check.stderr)
            .contains("replacement approval must not be later than supersession")
    );
}

#[cfg(unix)]
#[test]
fn init_rejects_symlink_output_without_changing_target() {
    use std::os::unix::fs::symlink;

    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().canonicalize().expect("canonical tempdir");
    let source = root.join("policy.md");
    let target = root.join("target.json");
    let output = root.join("record.json");
    std::fs::write(&source, "# Policy\n").expect("source");
    std::fs::write(&target, "keep-me").expect("target");
    symlink(&target, &output).expect("symlink");
    let result = run(&[
        "lifecycle",
        "init",
        "--source",
        source.to_str().expect("source"),
        "--output",
        output.to_str().expect("output"),
        "--policy-key",
        "safe-policy",
        "--version-key",
        "v1",
        "--title",
        "Safe Policy",
        "--owner",
        "owner",
        "--next-review",
        "2027-08-25",
    ]);
    assert_eq!(result.status.code(), Some(2));
    assert_eq!(std::fs::read_to_string(target).expect("target"), "keep-me");
}

#[cfg(unix)]
#[test]
fn init_rejects_a_symlink_in_an_input_path_component() {
    use std::os::unix::fs::symlink;

    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().canonicalize().expect("canonical tempdir");
    let real = root.join("real");
    let alias = root.join("alias");
    let output = root.join("record.json");
    std::fs::create_dir(&real).expect("real directory");
    std::fs::write(real.join("policy.md"), "# Policy\n").expect("source");
    symlink(&real, &alias).expect("directory symlink");
    let source = alias.join("policy.md");
    let result = run(&[
        "lifecycle",
        "init",
        "--source",
        source.to_str().expect("source"),
        "--output",
        output.to_str().expect("output"),
        "--policy-key",
        "component-safe-policy",
        "--version-key",
        "v1",
        "--title",
        "Component Safe Policy",
        "--owner",
        "owner",
        "--next-review",
        "2027-08-25",
    ]);
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("is a symlink"));
    assert!(!output.exists());
}

#[cfg(unix)]
#[test]
fn transition_rejects_hard_link_alias_without_changing_record() {
    let fixture = Fixture::new("hard-link-policy", false);
    let alias = fixture.record.with_file_name("lifecycle-alias.json");
    std::fs::hard_link(&fixture.record, &alias).expect("hard link");
    let before = std::fs::read(&fixture.record).expect("before");
    let result =
        fixture.transition("in-review", "reviewer", "reviewer", "2026-08-25T10:00:00Z", &[]);
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("hard-link aliases"));
    assert_eq!(std::fs::read(&fixture.record).expect("record"), before);
    assert_eq!(std::fs::read(alias).expect("alias"), before);
}
