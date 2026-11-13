//! PRD 065 Phase 1 adapter contract fixtures (M-15, G-2/G-3/G-4).
//!
//! Every fixture runs against the in-process loopback fake server; no outbound
//! network access and no new dependencies. The same contract binds the deferred
//! live GitHub HTTPS adapter (see docs/integration-change-sets.md).

use std::path::Path;
use std::process::{Command, Output};

use chrono::{DateTime, Duration, Utc};
use forge::workflow::adapter::{OperationKind, Payload, Precondition, Transport};
use forge::workflow::change_set::{self, ChangeSet, Provenance, WorkItem, WorkItemKind};
use forge::workflow::config::{
    CONFIG_SCHEMA_VERSION, ConnectorConfig, ConnectorKind, NetworkPolicy,
};
use forge::workflow::fake::{FakeCredentials, FakeServer, FakeTransport};
use forge::workflow::plan::{self, ConnectorState, DEFAULT_TTL_HOURS};
use serde_json::{Value, json};
use tempfile::TempDir;

mod common;

use common::sha256_hex;
const TARGET: &str = "policy-forge/forge";

fn now() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339("2026-11-12T00:00:00Z").unwrap().with_timezone(&Utc)
}

fn config() -> ConnectorConfig {
    ConnectorConfig {
        schema_version: CONFIG_SCHEMA_VERSION.to_string(),
        connector: ConnectorKind::GithubIssues,
        targets: vec![TARGET.to_string()],
        network: NetworkPolicy::default(),
    }
}

/// One temp project root holding a real source artifact whose hash the change
/// set records, so plan/apply source re-verification exercises the real path.
struct Fixture {
    _dir: TempDir,
    root: std::path::PathBuf,
    config: ConnectorConfig,
    config_bytes: String,
    set: ChangeSet,
    set_bytes: String,
}

impl Fixture {
    fn new(keys: &[&str]) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        std::fs::create_dir_all(root.join("reports")).unwrap();
        let source_bytes =
            serde_json::to_vec(&json!({"schema_version": "forge.applicability-report/1"})).unwrap();
        std::fs::write(root.join("reports/gaps.json"), &source_bytes).unwrap();
        let hash = sha256_hex(&source_bytes);
        let items = keys
            .iter()
            .map(|key| WorkItem {
                id: change_set::item_id(WorkItemKind::Gap, &hash, key),
                kind: WorkItemKind::Gap,
                key: (*key).to_string(),
                title: format!("gap: {key}"),
                reason_codes: vec!["no-reviewed-mapping".to_string()],
                owners: vec!["owner".to_string()],
                due_date: Some("2026-12-01".to_string()),
                provenance: Provenance {
                    schema_version: "forge.applicability-report/1".to_string(),
                    source: "reports/gaps.json".to_string(),
                    sha256: hash.clone(),
                },
                excerpt: None,
            })
            .collect();
        let set = ChangeSet::new(items).unwrap();
        let set_bytes = set.render().unwrap();
        let config = config();
        let config_bytes = config.render().unwrap();
        Self { _dir: dir, root, config, config_bytes, set, set_bytes }
    }

    fn plan(&self, state: &ConnectorState) -> plan::Plan {
        plan::build_plan(
            &self.root,
            &self.config,
            self.config_bytes.as_bytes(),
            &self.set,
            self.set_bytes.as_bytes(),
            state,
            now(),
            DEFAULT_TTL_HOURS,
        )
        .unwrap()
    }

    fn apply(
        &self,
        document: &plan::Plan,
        state: &ConnectorState,
        transport: &mut FakeTransport,
    ) -> (ConnectorState, plan::AuditReport) {
        plan::apply(
            &self.config,
            self.config_bytes.as_bytes(),
            document,
            document.render().unwrap().as_bytes(),
            self.set_bytes.as_bytes(),
            state,
            transport,
            now(),
        )
        .unwrap()
    }
}

fn transport(server: &FakeServer) -> FakeTransport {
    FakeTransport::new(server.addr(), Box::new(FakeCredentials))
}

fn entry<'a>(report: &'a plan::AuditReport, work_item_id: &str) -> &'a plan::AuditEntry {
    report.entries.iter().find(|entry| entry.work_item_id == work_item_id).unwrap()
}

/// G-3: repeating an accepted change set creates zero duplicate remote objects,
/// both through server-side idempotency keys and local reconcile-before-create.
#[test]
fn repeated_apply_creates_zero_duplicates() {
    let fixture = Fixture::new(&["ac-1", "ac-2"]);
    let server = FakeServer::start().unwrap();
    let mut client = transport(&server);
    let document = fixture.plan(&ConnectorState::empty());
    let (state, report) = fixture.apply(&document, &ConnectorState::empty(), &mut client);
    assert!(report.entries.iter().all(|entry| entry.status == "applied"));
    assert_eq!(server.issue_count().unwrap(), 2);
    assert_eq!(state.bindings.len(), 2);

    // Same plan applied again against the recorded state: every create is
    // skipped locally, and the server keeps exactly two objects.
    let (_, again) = fixture.apply(&document, &state, &mut client);
    assert!(again.entries.iter().all(|entry| entry.status == "skipped-duplicate"));
    assert_eq!(server.issue_count().unwrap(), 2);

    // A freshly rebuilt plan against the same state plans updates, not creates.
    let update_plan = fixture.plan(&state);
    assert!(update_plan.operations.iter().all(|op| op.kind == OperationKind::Update));
    let (next, applied) = fixture.apply(&update_plan, &state, &mut client);
    assert!(applied.entries.iter().all(|entry| entry.status == "applied"));
    assert_eq!(server.issue_count().unwrap(), 2);
    for binding in next.bindings.values() {
        assert_eq!(binding.remote_version, 2);
    }
}

/// G-4: remote version drift is a recorded conflict, never silent
/// last-write-wins; the local binding keeps the last successful version.
#[test]
fn remote_drift_is_a_conflict_not_last_write_wins() {
    let fixture = Fixture::new(&["ac-1"]);
    let server = FakeServer::start().unwrap();
    let mut client = transport(&server);
    let document = fixture.plan(&ConnectorState::empty());
    let (state, _) = fixture.apply(&document, &ConnectorState::empty(), &mut client);
    let binding = state.bindings.values().next().unwrap();
    let remote_id = binding.remote_id.clone();

    // An external actor bumps the remote version between plan and apply.
    server.inject(|injections| injections.drift = Some((remote_id.clone(), 7))).unwrap();
    let update_plan = fixture.plan(&state);
    let (next, report) = fixture.apply(&update_plan, &state, &mut client);
    let item_id = &update_plan.operations[0].work_item_id;
    let recorded = entry(&report, item_id);
    assert_eq!(recorded.status, "conflict");
    assert_eq!(recorded.error_class.as_deref(), Some("conflict"));
    assert_eq!(recorded.remote_version, Some(7));
    // The binding still records the last successful apply, not the drifted one.
    assert_eq!(next.bindings[item_id].remote_version, 1);
    assert_eq!(server.issue_count().unwrap(), 1);
}

/// M-13: provider rate limits are retried with the same idempotency key, so a
/// retried create never duplicates the remote object.
#[test]
fn rate_limited_create_retries_without_duplicates() {
    let fixture = Fixture::new(&["ac-1"]);
    let server = FakeServer::start().unwrap();
    let mut client = transport(&server);
    server.inject(|injections| injections.rate_limit_next = 2).unwrap();
    let document = fixture.plan(&ConnectorState::empty());
    let (_, report) = fixture.apply(&document, &ConnectorState::empty(), &mut client);
    assert!(report.entries.iter().all(|entry| entry.status == "applied"));
    assert_eq!(server.issue_count().unwrap(), 1);
    // Two 429s plus the accepted create.
    assert_eq!(server.requests().unwrap().len(), 3);
}

/// M-10: one failing operation never blocks the others, is never recorded as
/// applied, and a later apply resumes exactly the failed work.
#[test]
fn partial_failure_records_outcomes_and_resumes() {
    let fixture = Fixture::new(&["ac-1", "ac-2"]);
    let server = FakeServer::start().unwrap();
    let mut client = transport(&server);
    // Fail every retry of the first create so it is a true partial failure.
    server.inject(|injections| injections.fail_next_creates = plan::MAX_ATTEMPTS).unwrap();
    let document = fixture.plan(&ConnectorState::empty());
    let (state, report) = fixture.apply(&document, &ConnectorState::empty(), &mut client);
    let failed: Vec<_> = report.entries.iter().filter(|entry| entry.status == "failed").collect();
    let applied: Vec<_> = report.entries.iter().filter(|entry| entry.status == "applied").collect();
    assert_eq!(failed.len(), 1);
    assert_eq!(applied.len(), 1);
    assert_eq!(failed[0].error_class.as_deref(), Some("server-failure"));
    assert_eq!(state.bindings.len(), 1);
    assert!(!state.bindings.contains_key(&failed[0].work_item_id));

    // Resume: only the unbound item is planned again; the applied one updates.
    let resume = fixture.plan(&state);
    let (next, report) = fixture.apply(&resume, &state, &mut client);
    assert!(report.entries.iter().all(|entry| entry.status == "applied"));
    assert_eq!(next.bindings.len(), 2);
    assert_eq!(server.issue_count().unwrap(), 2);
}

/// M-4/M-13: operations outside the explicit allowlist are refused before any
/// transport activity, and the refusal is recorded per operation.
#[test]
fn non_allowlisted_targets_are_refused() {
    let fixture = Fixture::new(&["ac-1"]);
    let server = FakeServer::start().unwrap();
    let mut client = transport(&server);
    let mut document = fixture.plan(&ConnectorState::empty());
    document.operations[0].target = "other/repo".to_string();
    let (state, report) = fixture.apply(&document, &ConnectorState::empty(), &mut client);
    let recorded = entry(&report, &document.operations[0].work_item_id);
    assert_eq!(recorded.status, "target-refused");
    assert_eq!(recorded.error_class.as_deref(), Some("target-refused"));
    assert!(state.bindings.is_empty());
    assert_eq!(server.issue_count().unwrap(), 0);
    assert_eq!(server.requests().unwrap().len(), 0);
}

/// M-14: audit reports, state and the fake request log never carry credentials
/// or provider prose, and reports are deterministic for identical outcomes.
#[test]
fn reports_and_state_are_redacted_and_deterministic() {
    let fixture = Fixture::new(&["ac-1"]);
    let server = FakeServer::start().unwrap();
    let mut client = transport(&server);
    let document = fixture.plan(&ConnectorState::empty());
    let (state, report) = fixture.apply(&document, &ConnectorState::empty(), &mut client);
    let rendered = report.render().unwrap();
    assert_eq!(rendered, report.render().unwrap());
    for (case, secret) in ["fake-phase1-token", "Bearer", "Authorization"].into_iter().enumerate() {
        assert!(!rendered.contains(secret), "audit report leaked credential marker {case}");
        assert!(!state.render().unwrap().contains(secret));
    }
    let log = format!("{:?}", server.requests().unwrap());
    assert!(!log.contains("fake-phase1-token"));
    assert!(!log.to_ascii_lowercase().contains("bearer"));
    // The audit report binds the exact plan bytes.
    assert_eq!(
        report.plan_sha256.as_deref(),
        Some(sha256_hex(document.render().unwrap().as_bytes()).as_str())
    );
}

/// M-11: reconcile reports observations only — drift and missing objects are
/// recorded, but neither FORGE source artifacts nor connector state change.
#[test]
fn reconcile_observes_without_mutating_state_or_sources() {
    let fixture = Fixture::new(&["ac-1", "ac-2"]);
    let server = FakeServer::start().unwrap();
    let mut client = transport(&server);
    let document = fixture.plan(&ConnectorState::empty());
    let (state, _) = fixture.apply(&document, &ConnectorState::empty(), &mut client);
    let state_bytes = state.render().unwrap();
    let source_bytes = std::fs::read(fixture.root.join("reports/gaps.json")).unwrap();

    let ids: Vec<String> = state.bindings.values().map(|b| b.remote_id.clone()).collect();
    server.inject(|injections| injections.drift = Some((ids[0].clone(), 9))).unwrap();
    // The drift injection applies on the next PATCH; force it through an update
    // so reconcile then observes the drifted version.
    let update_plan = fixture.plan(&state);
    let mut drift_op = update_plan.operations[0].clone();
    drift_op.precondition = Some(Precondition {
        remote_id: ids[0].clone(),
        remote_version: state.bindings[&drift_op.work_item_id].remote_version,
    });
    let request = forge::workflow::adapter::OperationRequest {
        target: TARGET.to_string(),
        idempotency_key: drift_op.idempotency_key.clone(),
        kind: OperationKind::Update,
        payload: Payload { title: "drift".to_string(), body: "drift".to_string() },
        precondition: drift_op.precondition.clone(),
    };
    let _ = client.execute(&NetworkPolicy::default(), &request);

    let report = plan::reconcile(&fixture.config, &state, &mut client).unwrap();
    assert_eq!(report.phase, "reconcile");
    assert!(report.plan_sha256.is_none());
    let statuses: Vec<&str> = report.entries.iter().map(|entry| entry.status.as_str()).collect();
    assert!(statuses.contains(&"drift"), "expected a drift observation: {statuses:?}");
    assert!(statuses.contains(&"observed"));
    // Observations never rewrite local state or FORGE sources.
    assert_eq!(state.render().unwrap(), state_bytes);
    assert_eq!(std::fs::read(fixture.root.join("reports/gaps.json")).unwrap(), source_bytes);

    // A deleted remote object is reported missing, still without mutation.
    let mut missing_state = state.clone();
    missing_state.bindings.values_mut().next().unwrap().remote_id = "issue-999".to_string();
    let missing = plan::reconcile(&fixture.config, &missing_state, &mut client).unwrap();
    assert!(missing.entries.iter().any(|entry| entry.status == "missing"));
}

/// M-6/G-2: apply refuses expired plans and plans whose change-set or config
/// bytes drifted; M-3: source drift is refused before planning.
#[test]
fn plan_gates_refuse_expiry_drift_and_tampering() {
    let fixture = Fixture::new(&["ac-1"]);
    let server = FakeServer::start().unwrap();
    let mut client = transport(&server);
    let document = fixture.plan(&ConnectorState::empty());
    let plan_bytes = document.render().unwrap();

    let expired = plan::apply(
        &fixture.config,
        fixture.config_bytes.as_bytes(),
        &document,
        plan_bytes.as_bytes(),
        fixture.set_bytes.as_bytes(),
        &ConnectorState::empty(),
        &mut client,
        now() + Duration::hours(DEFAULT_TTL_HOURS + 1),
    );
    assert!(expired.is_err());
    let drifted_set = plan::apply(
        &fixture.config,
        fixture.config_bytes.as_bytes(),
        &document,
        plan_bytes.as_bytes(),
        b"different change set",
        &ConnectorState::empty(),
        &mut client,
        now(),
    );
    assert!(drifted_set.is_err());
    assert_eq!(server.issue_count().unwrap(), 0);

    // Tampered payload hashes are refused at parse time.
    let mut value = serde_json::to_value(&document).unwrap();
    value["operations"][0]["payload"]["title"] = json!("tampered");
    assert!(plan::Plan::parse(&serde_json::to_vec(&value).unwrap()).is_err());

    // Source drift is refused before any operation is planned.
    std::fs::write(fixture.root.join("reports/gaps.json"), b"drifted source").unwrap();
    assert!(
        plan::build_plan(
            &fixture.root,
            &fixture.config,
            fixture.config_bytes.as_bytes(),
            &fixture.set,
            fixture.set_bytes.as_bytes(),
            &ConnectorState::empty(),
            now(),
            DEFAULT_TTL_HOURS,
        )
        .is_err()
    );
}

/// M-5/M-14: credential-shaped content is refused at the change-set boundary
/// and never reaches the transport.
/// The idempotency key is the duplicate-prevention anchor, so a plan whose key is not
/// the canonical value for its (target, work item) is refused at parse time.
#[test]
fn tampered_idempotency_key_is_refused_at_parse() {
    let fixture = Fixture::new(&["ac-1"]);
    let document = fixture.plan(&ConnectorState::empty());
    let mut value = serde_json::to_value(&document).unwrap();
    // Untouched plan still parses.
    assert!(plan::Plan::parse(&serde_json::to_vec(&value).unwrap()).is_ok());
    value["operations"][0]["idempotency_key"] = json!("not-the-canonical-key");
    assert!(plan::Plan::parse(&serde_json::to_vec(&value).unwrap()).is_err());
}

#[test]
fn credential_shaped_payloads_are_refused() {
    for (case, secret) in
        ["ghp_0123456789abcdef", "-----BEGIN PRIVATE KEY-----", "Authorization: Bearer x"]
            .into_iter()
            .enumerate()
    {
        let mut item = WorkItem {
            id: change_set::item_id(WorkItemKind::Gap, &"a".repeat(64), "ac-1"),
            kind: WorkItemKind::Gap,
            key: "ac-1".to_string(),
            title: secret.to_string(),
            reason_codes: vec!["no-reviewed-mapping".to_string()],
            owners: Vec::new(),
            due_date: None,
            provenance: Provenance {
                schema_version: "forge.applicability-report/1".to_string(),
                source: "reports/gaps.json".to_string(),
                sha256: "a".repeat(64),
            },
            excerpt: None,
        };
        assert!(ChangeSet::new(vec![item.clone()]).is_err(), "title case {case} was accepted");
        item.title = "gap: ac-1".to_string();
        item.excerpt = Some(secret.to_string());
        assert!(ChangeSet::new(vec![item]).is_err(), "excerpt case {case} was accepted");
    }
}

// ---- CLI end-to-end fixtures -------------------------------------------------

fn forge_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_forge"))
}

fn run_in(root: &Path, args: &[&str]) -> Output {
    forge_bin().current_dir(root).args(args).output().expect("run forge")
}

fn write_json(path: &Path, value: &Value) {
    std::fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn resource(href: &str) -> Value {
    json!({
        "resource_type": "catalog",
        "href": href,
        "raw_sha256": "a".repeat(64),
        "root_uuid": "11111111-1111-4111-8111-111111111111",
        "document_version": "1.0.0",
        "oscal_version": "1.2.3"
    })
}

fn framework_report() -> Value {
    json!({
        "schema_version": "forge.framework-impact-report/1",
        "status": "complete",
        "old": resource("old.json"),
        "new": resource("new.json"),
        "summary": {
            "old_controls": 1, "new_controls": 2, "added": 1, "removed": 0,
            "content_changed": 0, "identity_migrated": 0, "unchanged": 1,
            "findings": 1, "blocking": 0, "review_required": 1, "informational": 0,
            "dispositioned_resolved": 0, "dispositioned_accepted_risk": 0,
            "dispositioned_still_open": 0, "undispositioned": 1
        },
        "filters": {},
        "matched_findings": 0,
        "changes": [{
            "subject_id": "ac-2", "change_class": "added",
            "old_subjects": [], "new_subjects": [{"id": "ac-2", "sha256": "b".repeat(64)}]
        }],
        "findings": [{
            "finding_id": "finding-1", "priority": "review-required",
            "reason_code": "control_added", "required_action": "review-applicability",
            "subject_id": "ac-2", "change_class": "added",
            "old_subjects": [], "new_subjects": [{"id": "ac-2", "sha256": "b".repeat(64)}],
            "dependency_path": [], "framework_groups": ["ac"],
            "owner": "owner", "policy_sources": ["policy.md"]
        }]
    })
}

fn applicability_report() -> Value {
    json!({
        "schema_version": "forge.applicability-report/1",
        "manifest_sha256": "a".repeat(64),
        "framework": resource("framework.json"),
        "mapping_collections": [],
        "reviewers": [],
        "counts": {
            "total": 1, "applicable_mapped": 0, "applicable_reviewed_no_relationship": 0,
            "applicable_unmapped": 1, "not_applicable": 0, "deferred": 0, "under_review": 0
        },
        "filters": {},
        "matched_controls": 1,
        "controls": [{
            "control_id": "ac-1", "groups": ["ac"], "classification": "applicable-unmapped",
            "positive_mapping_count": 0, "no_relationship_count": 0, "policy_sources": []
        }],
        "review_queue": [{
            "control_id": "ac-1", "reason_code": "no-reviewed-mapping",
            "owner": "owner", "revisit_date": "2026-12-01", "policy_sources": []
        }]
    })
}

fn poam_manifest() -> Value {
    let pin = |name: &str| {
        json!({
            "artifact": format!("{name}.json"), "href": format!("{name}.json"),
            "expected_sha256": "a".repeat(64),
            "root_uuid": "11111111-1111-4111-8111-111111111111",
            "document_version": "1.0.0", "oscal_version": "1.2.3"
        })
    };
    let initial = json!({
        "key": "initial", "actor": {"role_id": "owner", "party_key": "alice"},
        "at": "2026-01-01T00:00:00Z", "from": null, "to": "planned",
        "rationale": "initial plan", "closure": null
    });
    let owner = json!({"role_id": "owner", "party_key": "alice", "rationale": "assigned owner"});
    json!({
        "schema_version": "forge.poam/1",
        "document": {"key": "plan", "title": "Authored plan", "version": "1.0.0",
                     "last_modified": "2026-02-20T00:00:00Z"},
        "source": {
            "assessment_results": pin("ar"),
            "result": {"uuid": "11111111-1111-4111-8111-111111111111", "key": "result"},
            "context": {"assessment_plan": pin("ap"), "ssp": pin("ssp"),
                        "profile": pin("profile"), "catalog": pin("catalog")}
        },
        "roles": [{"id": "owner", "title": "Owner"}],
        "parties": [{"key": "alice", "type": "person", "name": "Alice"}],
        "items": [{
            "key": "work", "title": "Authored work", "description": "Remediation description",
            "source_refs": [{"kind": "finding", "key": "finding",
                             "uuid": "22222222-2222-4222-8222-222222222222",
                             "result_uuid": "11111111-1111-4111-8111-111111111111",
                             "expected_sha256": "b".repeat(64)}],
            "owners": [owner], "target_date": "2026-02-01", "state": "planned",
            "history": [initial],
            "milestones": [{
                "key": "step", "outcome": "Authored outcome", "target_date": "2026-01-20",
                "depends_on": [], "owners": [owner], "state": "planned", "history": [initial]
            }]
        }]
    })
}

/// M-1/M-2/M-12: the CLI exports deterministic change sets from real framework,
/// applicability, lifecycle and POA&M artifacts; default payloads are minimized
/// and excerpts require explicit opt-in.
#[test]
fn cli_export_is_deterministic_and_minimized() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    write_json(&root.join("impact.json"), &framework_report());
    write_json(&root.join("gaps.json"), &applicability_report());
    write_json(&root.join("poam.json"), &poam_manifest());
    std::fs::write(root.join("policy.md"), "# Policy\n\nPolicy text.\n").unwrap();
    write_json(
        &root.join("catalog.json"),
        &json!({"catalog": {"uuid": "11111111-1111-4111-8111-111111111111",
            "metadata": {"title": "Policy", "last-modified": "2026-08-25T00:00:00Z",
                         "version": "1.0.0", "oscal-version": "1.2.3"}, "groups": []}}),
    );
    let init = run_in(
        &root,
        &[
            "lifecycle",
            "init",
            "--source",
            "policy.md",
            "--artifact",
            "catalog.json",
            "--output",
            "lifecycle.json",
            "--policy-key",
            "policy",
            "--version-key",
            "v1",
            "--title",
            "Policy",
            "--owner",
            "owner",
            "--party",
            "owner=owner",
            "--party",
            "reviewer=reviewer",
            "--party",
            "approver=approver",
            "--next-review",
            "2026-12-01",
        ],
    );
    assert!(init.status.success(), "{}", String::from_utf8_lossy(&init.stderr));

    let export_args = [
        "integration",
        "export",
        "--root",
        ".",
        "--framework-report",
        "impact.json",
        "--applicability-report",
        "gaps.json",
        "--lifecycle-record",
        "lifecycle.json",
        "--poam-manifest",
        "poam.json",
    ];
    let first = run_in(&root, &export_args);
    assert!(first.status.success(), "{}", String::from_utf8_lossy(&first.stderr));
    let second = run_in(&root, &export_args);
    assert_eq!(first.stdout, second.stdout, "export must be byte-deterministic");
    let set: Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(set["schema_version"], "forge.integration-change-set/1");
    let items = set["items"].as_array().unwrap();
    let kinds: Vec<&str> = items.iter().map(|item| item["kind"].as_str().unwrap()).collect();
    for kind in ["finding", "framework-impact", "gap", "lifecycle-review", "poam-milestone"] {
        assert!(kinds.contains(&kind), "missing {kind} in {kinds:?}");
    }
    for item in items {
        assert!(item.get("excerpt").is_none(), "default payloads must omit excerpts");
        let source = item["provenance"]["source"].as_str().unwrap();
        assert!(!source.starts_with('/'), "links must be repository-relative: {source}");
        assert!(!item["title"].as_str().unwrap().contains("Remediation description"));
    }
    // Items are sorted by stable ID.
    let ids: Vec<&str> = items.iter().map(|item| item["id"].as_str().unwrap()).collect();
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    assert_eq!(ids, sorted);

    // Opt-in excerpts appear only for the POA&M milestone description.
    let mut with_excerpts = export_args.to_vec();
    with_excerpts.push("--include-excerpts");
    let opted = run_in(&root, &with_excerpts);
    assert!(opted.status.success(), "{}", String::from_utf8_lossy(&opted.stderr));
    let opted_set: Value = serde_json::from_slice(&opted.stdout).unwrap();
    let milestone = opted_set["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["kind"] == "poam-milestone")
        .unwrap();
    assert_eq!(milestone["excerpt"], "Remediation description");
}

/// M-1/M-6/M-7: the CLI plan/apply flow requires confirmation, binds the exact
/// plan, and records redacted audit output; reconcile stays observation-only.
#[allow(clippy::too_many_lines)] // One end-to-end CLI flow keeps the gate sequence readable.
#[test]
fn cli_plan_apply_reconcile_flow() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    write_json(&root.join("gaps.json"), &applicability_report());
    let config_value = json!({
        "schema_version": CONFIG_SCHEMA_VERSION,
        "connector": "github-issues",
        "targets": [TARGET],
        "network": NetworkPolicy::default()
    });
    write_json(&root.join("connector.json"), &config_value);
    let export = run_in(
        &root,
        &[
            "integration",
            "export",
            "--root",
            ".",
            "--applicability-report",
            "gaps.json",
            "--output",
            "change-set.json",
        ],
    );
    assert!(export.status.success(), "{}", String::from_utf8_lossy(&export.stderr));

    let plan_args = [
        "integration",
        "plan",
        "--root",
        ".",
        "--change-set",
        "change-set.json",
        "--connector-config",
        "connector.json",
        "--state",
        "state.json",
        "--as-of",
        "2026-11-12T00:00:00Z",
        "--output",
        "plan.json",
    ];
    let planned = run_in(&root, &plan_args);
    assert!(planned.status.success(), "{}", String::from_utf8_lossy(&planned.stderr));

    // Without --confirm, apply prints the summary and mutates nothing.
    let preview = run_in(
        &root,
        &[
            "integration",
            "apply",
            "--root",
            ".",
            "--change-set",
            "change-set.json",
            "--connector-config",
            "connector.json",
            "--plan",
            "plan.json",
            "--state",
            "state.json",
            "--as-of",
            "2026-11-12T00:00:00Z",
        ],
    );
    assert!(preview.status.success());
    let summary = String::from_utf8_lossy(&preview.stdout);
    assert!(summary.contains("Targets: policy-forge/forge"));
    assert!(summary.contains("--confirm"));
    assert!(!root.join("state.json").exists(), "preview must not write state");

    let applied = run_in(
        &root,
        &[
            "integration",
            "apply",
            "--root",
            ".",
            "--change-set",
            "change-set.json",
            "--connector-config",
            "connector.json",
            "--plan",
            "plan.json",
            "--state",
            "state.json",
            "--as-of",
            "2026-11-12T00:00:00Z",
            "--confirm",
            "--report",
            "audit.json",
        ],
    );
    assert!(applied.status.success(), "{}", String::from_utf8_lossy(&applied.stderr));
    let audit: Value =
        serde_json::from_slice(&std::fs::read(root.join("audit.json")).unwrap()).unwrap();
    assert_eq!(audit["schema_version"], "forge.integration-audit/1");
    assert_eq!(audit["phase"], "apply");
    assert_eq!(audit["entries"][0]["status"], "applied");
    let state: Value =
        serde_json::from_slice(&std::fs::read(root.join("state.json")).unwrap()).unwrap();
    assert_eq!(state["schema_version"], "forge.integration-state/1");
    assert_eq!(state["bindings"].as_object().unwrap().len(), 1);

    // An expired plan is refused at apply time.
    let expired = run_in(
        &root,
        &[
            "integration",
            "apply",
            "--root",
            ".",
            "--change-set",
            "change-set.json",
            "--connector-config",
            "connector.json",
            "--plan",
            "plan.json",
            "--state",
            "state.json",
            "--as-of",
            "2026-11-14T00:00:00Z",
            "--confirm",
        ],
    );
    assert!(!expired.status.success());
    assert!(String::from_utf8_lossy(&expired.stderr).contains("expired"));

    // Reconcile against the Phase 1 fake reports observations only.
    let reconciled = run_in(
        &root,
        &[
            "integration",
            "reconcile",
            "--connector-config",
            "connector.json",
            "--state",
            "state.json",
            "--report",
            "reconcile.json",
        ],
    );
    assert!(reconciled.status.success(), "{}", String::from_utf8_lossy(&reconciled.stderr));
    let report: Value =
        serde_json::from_slice(&std::fs::read(root.join("reconcile.json")).unwrap()).unwrap();
    assert_eq!(report["phase"], "reconcile");
    assert_eq!(report["entries"][0]["operation"], "observe");
    // The fake starts empty each run, so the recorded binding reads as missing;
    // the local state is never rewritten by reconcile.
    assert_eq!(report["entries"][0]["status"], "missing");
    let after: Value =
        serde_json::from_slice(&std::fs::read(root.join("state.json")).unwrap()).unwrap();
    assert_eq!(after, state);
}

/// Inputs are read through the bounded regular-file guard, so a symlinked
/// connector config is refused instead of silently followed.
#[cfg(unix)]
#[test]
fn cli_plan_refuses_symlinked_inputs() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    write_json(&root.join("gaps.json"), &applicability_report());
    write_json(
        &root.join("connector.json"),
        &json!({
            "schema_version": CONFIG_SCHEMA_VERSION,
            "connector": "github-issues",
            "targets": [TARGET],
            "network": NetworkPolicy::default()
        }),
    );
    let export = run_in(
        &root,
        &[
            "integration",
            "export",
            "--root",
            ".",
            "--applicability-report",
            "gaps.json",
            "--output",
            "change-set.json",
        ],
    );
    assert!(export.status.success(), "{}", String::from_utf8_lossy(&export.stderr));
    std::os::unix::fs::symlink(root.join("connector.json"), root.join("link.json")).unwrap();
    for (flag, link_target) in [("--connector-config", "link.json"), ("--change-set", "link.json")]
    {
        let (config, set) = if flag == "--connector-config" {
            (link_target, "change-set.json")
        } else {
            ("connector.json", link_target)
        };
        let planned = run_in(
            &root,
            &[
                "integration",
                "plan",
                "--root",
                ".",
                "--change-set",
                set,
                "--connector-config",
                config,
                "--state",
                "state.json",
                "--as-of",
                "2026-11-12T00:00:00Z",
                "--output",
                "plan.json",
            ],
        );
        assert!(!planned.status.success(), "symlinked input via {flag} must be refused");
        assert!(
            String::from_utf8_lossy(&planned.stderr).contains("regular file"),
            "{}",
            String::from_utf8_lossy(&planned.stderr)
        );
    }
}
