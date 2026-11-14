//! Two-independent-client MCP 2026-07-28 conformance harness over the real stdio worker.
//!
//! D067 (owner decision record 2026-11-13) names MCP Inspector (modern era) +
//! Claude Code (v2 runtime) as the conformance pair. Neither product can be
//! fetched/run fully offline here, so this harness drives `forge mcp serve`
//! with two independent client roles that implement the same 2026-07-28 wire
//! contract with different metadata/ID strategies and assertion paths:
//!
//! * Role A ("inspector-style"): strict schema-validating client, integer IDs,
//!   full namespaced `_meta` with clientInfo and an inert extension.
//! * Role B ("claude-code-style"): minimal client, string IDs, bare mandatory
//!   `_meta`, envelope-level assertions.
//!
//! Both roles run the same fixture and battery. This establishes worker-side
//! conformance to the selected revision; it does NOT claim the real products
//! passed. The real-client/online gate is recorded in
//! docs/plans/2026-11-13-f11-mcp-conformance-verification.md.
//!
//! Fixtures are synthetic declared-owner data built through public APIs; they
//! grant no real approval authority. Deterministic: fixed bytes, no clock, no
//! network, isolated temp roots, EOF-terminated workers.

#![cfg(unix)] // Symlinked-ancestry confinement needs std::os::unix::fs; Windows qualification is separate.

mod common;

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use common::sha256_hex;
use forge::applicability::model::ReportFilters;
use forge::cli::{
    ApplicabilityFailOn, ApplicabilityReportFormat, MappingFailOn, MappingReportFormat,
};
use forge::lifecycle::record::{
    self, APPROVAL_POLICY_VERSION, ApprovalPolicy, ArtifactFingerprint, DeclaredRole,
    FingerprintSet, LifecycleState, NamedHash, Party, PolicyIdentity, ReviewSchedule,
    RoleRequirement, SCHEMA_VERSION, SeparationRules, TimezonePolicy, TransitionEvent,
};
use forge::mapping::{inventory, manifest::ResourceManifest};
use forge::oscal::trace_embedding::FORGE_TRACE_NS;
use serde_json::{Value, json};
use tempfile::TempDir;

const REVISION: &str = "2026-07-28";
const PROPOSAL: &str = "d52da047f0166d9d8cd002221ee9608577f935e7c803969a178b91e797801f49";
const TOOLS: [&str; 7] = [
    "list_policies",
    "search_requirements",
    "get_requirement",
    "trace_control",
    "get_recorded_applicability",
    "get_gap_summary",
    "get_artifact_status",
];
const SOURCE: &[u8] =
    b"Private sentinel caf\xc3\xa9 access audit.\r\nSecond monitoring audit line.\n";

// ─── Fixture: genuine synthetic originals through public APIs ───────────────

struct Fixture {
    project: TempDir,
    decision: TempDir,
    profile_pin: String,
    decision_pin: String,
    mapping_id: String,
    report_counts: Value,
    report_classifications: Value,
}

fn append_event(rec: &mut record::LifecycleRecord, next: LifecycleState, fp: &FingerprintSet) {
    let sequence = u32::try_from(rec.history.len() + 1).expect("bounded history");
    let (actor, role) = if next == LifecycleState::InReview {
        ("private-reviewer", DeclaredRole::Reviewer)
    } else {
        ("private-approver", DeclaredRole::Approver)
    };
    let mut event = TransitionEvent {
        sequence,
        event_id: String::new(),
        legacy_event_id: None,
        previous_state: rec.state,
        next_state: next,
        actor_key: actor.to_owned(),
        declared_role: role,
        timestamp: format!("2026-09-01T{sequence:02}:00:00Z"),
        rationale: "Private rationale remains undisclosed".to_owned(),
        fingerprints: fp.clone(),
        assertions: Vec::new(),
        impact_finding_ids: Vec::new(),
        replacement: None,
    };
    event.event_id = record::event_id(rec, &event).expect("intrinsic event ID");
    rec.history.push(event);
    rec.state = next;
}

fn base_lifecycle(catalog_bytes: &[u8]) -> record::LifecycleRecord {
    record::LifecycleRecord {
        schema_version: SCHEMA_VERSION.to_owned(),
        policy: PolicyIdentity {
            policy_key: "synthetic-policy".to_owned(),
            version_key: "version-1".to_owned(),
            title: "Sensitive local record title".to_owned(),
            owner_keys: vec!["private-owner".to_owned()],
            source: ArtifactFingerprint {
                path: "source.txt".to_owned(),
                sha256: sha256_hex(SOURCE),
                oscal_type: None,
                root_uuid: None,
            },
            generated_artifacts: vec![ArtifactFingerprint {
                path: "catalog.json".to_owned(),
                sha256: sha256_hex(catalog_bytes),
                oscal_type: Some("catalog".to_owned()),
                root_uuid: Some("11111111-1111-4111-8111-111111111111".to_owned()),
            }],
        },
        parties: vec![
            Party { key: "private-owner".to_owned(), roles: vec![DeclaredRole::Owner] },
            Party { key: "private-reviewer".to_owned(), roles: vec![DeclaredRole::Reviewer] },
            Party { key: "private-approver".to_owned(), roles: vec![DeclaredRole::Approver] },
        ],
        approval_policy: ApprovalPolicy {
            schema_version: APPROVAL_POLICY_VERSION.to_owned(),
            required_roles: vec![
                RoleRequirement { role: DeclaredRole::Reviewer, count: 1 },
                RoleRequirement { role: DeclaredRole::Approver, count: 1 },
            ],
            separation: SeparationRules::default(),
        },
        review: ReviewSchedule {
            cadence_days: 30,
            next_review_date: chrono::NaiveDate::from_ymd_opt(2026, 12, 1).expect("fixed date"),
            due_soon_days: 7,
            timezone_policy: TimezonePolicy::DateOnly,
        },
        state: LifecycleState::Draft,
        replaced_by: None,
        history: Vec::new(),
    }
}

fn approve(rec: &mut record::LifecycleRecord, source_sha: &str, generated: Vec<NamedHash>) {
    let fp =
        FingerprintSet { source_sha256: source_sha.to_owned(), generated_artifacts: generated };
    rec.history.clear();
    rec.state = LifecycleState::Draft;
    append_event(rec, LifecycleState::InReview, &fp);
    append_event(rec, LifecycleState::Approved, &fp);
    record::validate(rec).expect("intrinsically admitted fixture record");
}

#[allow(clippy::needless_pass_by_value)] // Fixture builder owns each complete JSON identity value.
fn resource(
    key: &str,
    role: &str,
    path: &str,
    bytes: &[u8],
    identity: Option<Value>,
    lifecycle: Option<&str>,
) -> Value {
    json!({"key": key, "role": role, "path": path, "expected_sha256": sha256_hex(bytes),
        "native_identity": identity, "lifecycle_key": lifecycle,
        "citation_label": format!("synthetic-{key}")})
}

impl Fixture {
    #[allow(clippy::too_many_lines)] // Complete genuine fixture remains visible together.
    fn new() -> Self {
        let project = TempDir::new().expect("project root");
        let decision = TempDir::new().expect("decision root");
        let root = project.path().canonicalize().expect("canonical project");
        let catalog = json!({"catalog":{
            "uuid":"11111111-1111-4111-8111-111111111111",
            "metadata":{"title":"Synthetic private instruction title","last-modified":"2026-09-01T00:00:00Z","version":"1","oscal-version":"1.2.3"},
            "controls":[
                {"id":"synthetic-a","title":"Sensitive control title","props":[
                    {"name":"source-file","ns":FORGE_TRACE_NS,"value":"source.txt"},
                    {"name":"source-section","ns":FORGE_TRACE_NS,"value":"section-one"},
                    {"name":"source-line","ns":FORGE_TRACE_NS,"value":"1"}]},
                {"id":"synthetic-b","title":"Sensitive other title","props":[
                    {"name":"source-file","ns":FORGE_TRACE_NS,"value":"source.txt"},
                    {"name":"source-section","ns":FORGE_TRACE_NS,"value":"section-two"},
                    {"name":"source-line","ns":FORGE_TRACE_NS,"value":"2"}]}]}});
        let catalog_bytes = serde_json::to_vec(&catalog).expect("catalog JSON");
        fs::write(root.join("source.txt"), SOURCE).expect("source");
        fs::write(root.join("catalog.json"), &catalog_bytes).expect("catalog");
        let mut lifecycle = base_lifecycle(&catalog_bytes);
        approve(
            &mut lifecycle,
            &sha256_hex(SOURCE),
            vec![NamedHash { path: "catalog.json".to_owned(), sha256: sha256_hex(&catalog_bytes) }],
        );
        let lifecycle_bytes = serde_json::to_vec(&lifecycle).expect("lifecycle JSON");
        fs::write(root.join("lifecycle.json"), &lifecycle_bytes).expect("lifecycle");
        let catalog_identity = json!({"model":"catalog","root_id":"11111111-1111-4111-8111-111111111111","document_version":"1","oscal_version":"1.2.3"});
        let mut resources = vec![
            resource(
                "catalog",
                "oscal-catalog-artifact",
                "catalog.json",
                &catalog_bytes,
                Some(catalog_identity),
                Some("lifecycle"),
            ),
            resource("source", "policy-source", "source.txt", SOURCE, None, Some("lifecycle")),
            resource(
                "lifecycle",
                "lifecycle-record",
                "lifecycle.json",
                &lifecycle_bytes,
                None,
                None,
            ),
        ];
        let mut visible = vec![json!("catalog"), json!("source")];

        // Domain applicability manifest with a real framework inventory.
        let framework_resource: ResourceManifest = serde_json::from_value(json!({
            "type":"catalog","artifact":"catalog.json","href":"catalog.json",
            "expected_sha256": sha256_hex(&catalog_bytes)}))
        .expect("framework resource");
        let loaded = inventory::load(&root, "$.framework", &framework_resource).expect("inventory");
        let mut framework = json!({"type":"catalog","artifact":"catalog.json","href":"catalog.json",
            "expected_sha256": sha256_hex(&catalog_bytes)});
        framework["inventory"] = serde_json::to_value(loaded.snapshot()).expect("snapshot");
        let mut domain = json!({"schema_version":"forge.applicability/1","framework":framework,
            "reviewers":[{"key":"reviewer","type":"person","name":"Private domain reviewer"}],
            "decisions":[{"control_id":"synthetic-a","state":"applicable","reviewer_key":"reviewer",
                "reviewed_at":"2026-09-01T08:00:00Z","rationale":"Private applicability rationale"}],
            "mapping_collections":[]});
        forge::applicability::manifest::parse(&serde_json::to_vec(&domain).unwrap())
            .expect("manifest");

        // Mapping collection built by the maintained public builder.
        let authoring = json!({"schema_version":"forge.mapping-manifest/1",
            "collection":{"key":"synthetic-collection","title":"Private mapping title","version":"1","last_modified":"2026-09-01T08:00:00Z"},
            "reviewers":[{"key":"mapper","type":"person","name":"Private mapping reviewer"}],
            "provenance":{"method":"human","matching_rationale":"semantic","status":"complete",
                "mapping_description":"Private mapping description","reviewer_keys":["mapper"],"reviewed_at":"2026-09-01T08:00:00Z"},
            "mapping":{"key":"synthetic-map","scope":"control-only",
                "source":{"type":"catalog","artifact":"catalog.json","href":"catalog.json"},
                "target":{"type":"catalog","artifact":"catalog.json","href":"catalog.json"},
                "maps":[{"key":"many-to-many","relationship":"intersects-with",
                    "sources":[{"type":"control","id_ref":"synthetic-a"},{"type":"control","id_ref":"synthetic-b"}],
                    "targets":[{"type":"control","id_ref":"synthetic-a"},{"type":"control","id_ref":"synthetic-b"}],
                    "reviewer_key":"mapper","reviewed_at":"2026-09-01T08:00:00Z","rationale":"Private mapping rationale"}]}});
        let authoring_bytes = serde_json::to_vec(&authoring).expect("authoring JSON");
        fs::write(root.join("mapping-authoring.json"), &authoring_bytes).expect("authoring");
        forge::mapping::execute_build(
            &root.join("mapping-authoring.json"),
            Some(&root.join("mapping.json")),
            None,
            &MappingReportFormat::Json,
            None,
            &MappingFailOn::Never,
            false,
        )
        .expect("mapping build");
        let mut native: Value =
            serde_json::from_slice(&fs::read(root.join("mapping.json")).unwrap()).unwrap();
        let map_id = native["mapping-collection"]["mappings"][0]["maps"][0]["uuid"]
            .as_str()
            .expect("map uuid")
            .to_ascii_uppercase();
        native["mapping-collection"]["mappings"][0]["maps"][0]["uuid"] = json!(map_id);
        let native_bytes = serde_json::to_vec(&native).expect("native mapping");
        fs::write(root.join("mapping.json"), &native_bytes).expect("mapping");
        let mapping_identity = json!({"model":"mapping-collection",
            "root_id": native["mapping-collection"]["uuid"],
            "document_version": native["mapping-collection"]["metadata"]["version"],
            "oscal_version": native["mapping-collection"]["metadata"]["oscal-version"]});
        let mut mapping_lifecycle = base_lifecycle(&catalog_bytes);
        "synthetic-mapping-policy".clone_into(&mut mapping_lifecycle.policy.policy_key);
        mapping_lifecycle.policy.source = ArtifactFingerprint {
            path: "mapping-authoring.json".to_owned(),
            sha256: sha256_hex(&authoring_bytes),
            oscal_type: None,
            root_uuid: None,
        };
        mapping_lifecycle.policy.generated_artifacts = vec![ArtifactFingerprint {
            path: "mapping.json".to_owned(),
            sha256: sha256_hex(&native_bytes),
            oscal_type: Some("mapping-collection".to_owned()),
            root_uuid: Some(mapping_identity["root_id"].as_str().unwrap().to_owned()),
        }];
        approve(
            &mut mapping_lifecycle,
            &sha256_hex(&authoring_bytes),
            vec![NamedHash { path: "mapping.json".to_owned(), sha256: sha256_hex(&native_bytes) }],
        );
        let mapping_record_bytes = serde_json::to_vec(&mapping_lifecycle).unwrap();
        fs::write(root.join("mapping-lifecycle.json"), &mapping_record_bytes).unwrap();
        resources.push(resource(
            "mapping",
            "mapping-collection",
            "mapping.json",
            &native_bytes,
            Some(mapping_identity),
            Some("mapping-lifecycle"),
        ));
        resources.push(resource(
            "mapping-source",
            "lifecycle-source",
            "mapping-authoring.json",
            &authoring_bytes,
            None,
            Some("mapping-lifecycle"),
        ));
        resources.push(resource(
            "mapping-lifecycle",
            "lifecycle-record",
            "mapping-lifecycle.json",
            &mapping_record_bytes,
            None,
            None,
        ));
        visible.push(json!("mapping"));
        domain["mapping_collections"] = json!(["mapping.json"]);

        // Install domain + report sources, each with its own approved lifecycle.
        let install = |key: &str,
                       path: &str,
                       value: &Value,
                       role: &str,
                       resources: &mut Vec<Value>,
                       visible: &mut Vec<Value>| {
            let bytes = serde_json::to_vec(value).expect("domain JSON");
            fs::write(root.join(path), &bytes).expect("domain original");
            let mut rec = base_lifecycle(&catalog_bytes);
            rec.policy.policy_key = format!("synthetic-{key}-policy");
            rec.policy.source = ArtifactFingerprint {
                path: path.to_owned(),
                sha256: sha256_hex(&bytes),
                oscal_type: None,
                root_uuid: None,
            };
            approve(
                &mut rec,
                &sha256_hex(&bytes),
                vec![NamedHash {
                    path: "catalog.json".to_owned(),
                    sha256: sha256_hex(&catalog_bytes),
                }],
            );
            let record_bytes = serde_json::to_vec(&rec).unwrap();
            let record_key = format!("{key}-lifecycle");
            let record_path = format!("{key}-lifecycle.json");
            fs::write(root.join(&record_path), &record_bytes).unwrap();
            resources.push(resource(key, role, path, &bytes, None, Some(&record_key)));
            resources.push(resource(
                &record_key,
                "lifecycle-record",
                &record_path,
                &record_bytes,
                None,
                None,
            ));
            visible.push(json!(key));
        };
        install(
            "domain",
            "applicability.json",
            &domain,
            "applicability-manifest",
            &mut resources,
            &mut visible,
        );
        forge::applicability::execute_analyze(
            &root.join("applicability.json"),
            &ApplicabilityReportFormat::Json,
            Some(&root.join("report.json")),
            &ApplicabilityFailOn::Never,
            None,
            ReportFilters::default(),
        )
        .expect("report");
        let report: Value =
            serde_json::from_slice(&fs::read(root.join("report.json")).unwrap()).unwrap();
        let report_counts = report["counts"].clone();
        let report_classifications = json!(
            report["controls"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| (
                    c["control_id"].as_str().unwrap().to_owned(),
                    c["classification"].clone()
                ))
                .collect::<serde_json::Map<String, Value>>()
        );
        install(
            "report",
            "report.json",
            &report,
            "applicability-report",
            &mut resources,
            &mut visible,
        );

        // Config originals with exact raw pins.
        let manifest = json!({"schema_version":"forge.mcp-project/1","project_key":"synthetic-project","resources":resources});
        let manifest_bytes = serde_json::to_vec(&manifest).unwrap();
        fs::write(root.join("forge.mcp.json"), &manifest_bytes).unwrap();
        let discovery_pin = sha256_hex(&manifest_bytes);
        let profile = json!({
            "schema_version":"forge.mcp-visibility/1","project_key":"synthetic-project",
            "discovery_sha256":discovery_pin,"enabled_tools":TOOLS,
            "visible_resource_keys":visible,"noncurrent_metadata_keys":[],
            "source_text_keys":[],"schedule_keys":[],"evidence_metadata_keys":[],
            "static_report_keys":[],"search_index_key":null,
            "identity_disclosure":"bounded-token-only",
            "approved_current_basis":"caller-established-recorded-policy",
            "source_text_mode":"omit","schedule_mode":"omit","as_of":null,
            "report_content_mode":"minimized-metadata","evidence_bytes":false,
            "network":false,"process_execution":false});
        let rows: Vec<Value> = [("D067-P1","product"),("D067-P2","product"),("D067-S1","security"),
            ("D067-E1","engineering"),("D067-S2","product"),("D067-S2","security")]
            .into_iter().map(|(subject, role)| json!({"subject":subject,"role":role,"disposition":"approved",
                "declared_owner_key":"synthetic-declared-owner","recorded_on":"2026-10-04",
                "source_record_key":"synthetic-private-record","source_record_sha256":"a".repeat(64),
                "proposal_index_sha256":PROPOSAL,"profile_sha256":"b".repeat(64),
                "discovery_sha256":discovery_pin})).collect();
        let mut decision_record = json!({"schema_version":"forge.mcp-disclosure-decision/1",
            "decision_key":"synthetic-decision","project_key":"synthetic-project",
            "discovery_sha256":discovery_pin,"profile_sha256":"b".repeat(64),
            "proposal_index_sha256":PROPOSAL,"protocol_revision":REVISION,
            "operator_boundary":"operator-selected-declared-owner-record",
            "visibility_profile":profile,"policy_decisions":rows,
            "evaluation_plan":{"state":"pending","declared_owner_key":null,"recorded_on":null,
                "source_record_key":null,"source_record_sha256":null,
                "corpus_acceptance":"unearned","threshold_acceptance":"unearned","client_acceptance":"unearned"}});
        let profile_bytes = serde_json::to_vec(&profile).unwrap();
        let profile_pin = sha256_hex(&profile_bytes);
        fs::write(root.join("forge.mcp.visibility.json"), &profile_bytes).unwrap();
        decision_record["profile_sha256"] = json!(profile_pin);
        decision_record["visibility_profile"] = profile.clone();
        for row in decision_record["policy_decisions"].as_array_mut().unwrap() {
            row["profile_sha256"] = json!(profile_pin);
        }
        let decision_bytes = serde_json::to_vec(&decision_record).unwrap();
        let decision_pin = sha256_hex(&decision_bytes);
        let decision_root = decision.path().canonicalize().expect("canonical decision");
        fs::write(decision_root.join("forge.mcp.disclosure-decision.json"), &decision_bytes)
            .unwrap();
        Self {
            project,
            decision,
            profile_pin,
            decision_pin,
            mapping_id: map_id,
            report_counts,
            report_classifications,
        }
    }

    fn root(&self) -> PathBuf {
        self.project.path().canonicalize().expect("canonical project")
    }

    fn decision_root(&self) -> PathBuf {
        self.decision.path().canonicalize().expect("canonical decision")
    }

    fn args(&self) -> Vec<String> {
        vec![
            "mcp".into(),
            "serve".into(),
            "--project".into(),
            self.root().display().to_string(),
            "--decision-root".into(),
            self.decision_root().display().to_string(),
            "--decision-sha256".into(),
            self.decision_pin.clone(),
            "--profile-sha256".into(),
            self.profile_pin.clone(),
        ]
    }

    fn static_args(&self) -> Vec<String> {
        vec!["mcp".into(), "serve".into(), "--project".into(), self.root().display().to_string()]
    }
}

// ─── Two independent client roles ───────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum Role {
    /// Inspector-style: integer IDs, full namespaced metadata, schema validation.
    Inspector,
    /// Claude-Code-style: string IDs, bare mandatory metadata, envelope checks.
    ClaudeCode,
}

impl Role {
    fn meta(self) -> Value {
        match self {
            Self::Inspector => json!({
                "io.modelcontextprotocol/protocolVersion": REVISION,
                "io.modelcontextprotocol/clientCapabilities": {},
                "io.modelcontextprotocol/clientInfo": {"name":"forge-conformance-inspector-role","version":"1"},
                "org.example/trace": {"inert": true}}),
            Self::ClaudeCode => json!({
                "io.modelcontextprotocol/protocolVersion": REVISION,
                "io.modelcontextprotocol/clientCapabilities": {}}),
        }
    }

    fn id(self, n: u64) -> Value {
        match self {
            Self::Inspector => json!(n),
            Self::ClaudeCode => json!(format!("req-{n}")),
        }
    }

    fn request(self, n: u64, method: &str, mut params: Value) -> Vec<u8> {
        params.as_object_mut().expect("object params").insert("_meta".to_owned(), self.meta());
        let mut bytes = serde_json::to_vec(&json!({
            "jsonrpc":"2.0","id":self.id(n),"method":method,"params":params}))
        .unwrap();
        bytes.push(b'\n');
        bytes
    }

    #[allow(clippy::needless_pass_by_value)] // The finite fixture builder owns each complete argument value.
    fn call(self, n: u64, tool: &str, arguments: Value) -> Vec<u8> {
        self.request(n, "tools/call", json!({"name":tool,"arguments":arguments}))
    }
}

struct Exchange {
    status: std::process::ExitStatus,
    rows: Vec<Value>,
    stderr: String,
}

/// Drive one real worker process: write the complete request stream, EOF, collect.
fn drive(args: &[String], requests: &[Vec<u8>], expected: usize) -> Exchange {
    drive_inner(args, requests, expected, false)
}

/// Blind burst: write every request before reading, to exercise the bounded queue.
fn drive_burst(args: &[String], requests: &[Vec<u8>], expected: usize) -> Exchange {
    drive_inner(args, requests, expected, true)
}

fn drive_inner(args: &[String], requests: &[Vec<u8>], expected: usize, burst: bool) -> Exchange {
    use std::io::{BufRead, BufReader, Write};
    let mut child = Command::new(env!("CARGO_BIN_EXE_forge"))
        .args(args)
        .env_remove("RUST_MIN_STACK")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the actual Forge binary must start");
    let stdout = child.stdout.take().expect("stdout");
    let (sender, receiver) = std::sync::mpsc::channel();
    let reader = std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            let line = line.expect("stdout read");
            if !line.is_empty() && sender.send(line).is_err() {
                break;
            }
        }
    });
    // Real-client pacing: one request at a time, awaiting its response before
    // the next, so the worker's bounded queue (1 active + 2 queued) is never
    // overflowed by a blind burst. Notifications expect no response. The worker
    // contract keeps stdin open while awaiting responses; EOF only at the end.
    let mut rows = Vec::new();
    let mut stdin = child.stdin.take().expect("stdin");
    if burst {
        for request in requests {
            stdin.write_all(request).expect("request write");
        }
        stdin.flush().expect("stdin flush");
        while rows.len() < expected {
            match receiver.recv_timeout(std::time::Duration::from_secs(30)) {
                Ok(line) => {
                    rows.push(serde_json::from_str(&line).expect("complete JSON output record"));
                }
                Err(_) => break,
            }
        }
    } else {
        for request in requests {
            stdin.write_all(request).expect("request write");
            stdin.flush().expect("stdin flush");
            let notification = serde_json::from_slice::<Value>(request)
                .is_ok_and(|value| value.get("id").is_none());
            if notification {
                continue;
            }
            match receiver.recv_timeout(std::time::Duration::from_secs(30)) {
                Ok(line) => {
                    rows.push(serde_json::from_str(&line).expect("complete JSON output record"));
                }
                Err(_) => break,
            }
            if rows.len() >= expected {
                break;
            }
        }
    }
    drop(stdin);
    let output = child.wait_with_output().expect("worker exit");
    reader.join().expect("stdout reader");
    Exchange {
        status: output.status,
        rows,
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

fn row_for<'a>(exchange: &'a Exchange, id: &Value) -> &'a Value {
    exchange
        .rows
        .iter()
        .find(|row| row.get("id") == Some(id))
        .unwrap_or_else(|| panic!("missing response for {id}: {:?}", exchange.rows))
}

fn output_validator(tool: &str) -> jsonschema::Validator {
    let schema: Value = serde_json::from_str(match tool {
        "list_policies" => include_str!("../src/mcp/tool-schemas/list_policies-output.schema.json"),
        "search_requirements" => {
            include_str!("../src/mcp/tool-schemas/search_requirements-output.schema.json")
        }
        "get_requirement" => {
            include_str!("../src/mcp/tool-schemas/get_requirement-output.schema.json")
        }
        "trace_control" => include_str!("../src/mcp/tool-schemas/trace_control-output.schema.json"),
        "get_recorded_applicability" => {
            include_str!("../src/mcp/tool-schemas/get_recorded_applicability-output.schema.json")
        }
        "get_gap_summary" => {
            include_str!("../src/mcp/tool-schemas/get_gap_summary-output.schema.json")
        }
        "get_artifact_status" => {
            include_str!("../src/mcp/tool-schemas/get_artifact_status-output.schema.json")
        }
        _ => panic!("unknown tool"),
    })
    .unwrap();
    jsonschema::validator_for(&schema).expect("shipped output schema compiles")
}

/// Complete result typing: modern discriminator, inert text, schema-checked structuredContent.
fn assert_tool_result<'a>(row: &'a Value, id: &Value, tool: &str, available: bool) -> &'a Value {
    assert_eq!(row["jsonrpc"], "2.0");
    assert_eq!(row["id"], *id);
    let result = &row["result"];
    assert_eq!(result["resultType"], "complete");
    assert_eq!(result["isError"], !available);
    assert_eq!(result["content"][0]["type"], "text");
    assert!(result["content"][0]["text"].as_str().is_some_and(|text| !text.is_empty()));
    let structured = &result["structuredContent"];
    assert_eq!(structured["schema_version"], "forge.mcp-query/1");
    assert_eq!(structured["tool"], tool);
    assert_eq!(structured["availability"], if available { "available" } else { "unavailable" });
    let validator = output_validator(tool);
    assert!(
        validator.is_valid(structured),
        "structuredContent violates {tool} output schema: {structured}"
    );
    if available {
        assert_eq!(structured["reason"], Value::Null);
        assert!(structured["data"].is_object());
    } else {
        assert!(structured["reason"].is_string());
        assert_eq!(structured["data"], Value::Null, "unavailable data must be null");
    }
    structured
}

// ─── Conformance battery ────────────────────────────────────────────────────

fn discover_and_typing(role: Role) {
    let fixture = Fixture::new();
    let mut requests = vec![
        role.request(1, "server/discover", json!({})),
        role.request(2, "tools/list", json!({})),
        // Historical initialize-first handshake is not part of 2026-07-28.
        role.request(3, "initialize", json!({})),
        // tools/list is one complete page; cursors are refused.
        role.request(4, "tools/list", json!({"cursor":"x"})),
    ];
    requests.push(role.call(5, "list_policies", json!({})));
    let exchange = drive(&fixture.args(), &requests, 5);
    assert!(exchange.status.success(), "worker must exit cleanly: {}", exchange.stderr);
    assert_eq!(exchange.rows.len(), 5);

    let discover = row_for(&exchange, &role.id(1));
    assert_eq!(discover["result"]["resultType"], "complete");
    assert_eq!(discover["result"]["supportedVersions"], json!([REVISION]));
    assert_eq!(discover["result"]["ttlMs"], 0);
    assert_eq!(discover["result"]["cacheScope"], "private");
    assert!(discover["result"]["capabilities"]["tools"].is_object());
    assert!(discover["result"]["instructions"].as_str().is_some_and(|text| text.contains("inert")));

    let list = row_for(&exchange, &role.id(2));
    assert_eq!(list["result"]["resultType"], "complete");
    assert_eq!(list["result"]["ttlMs"], 0);
    assert_eq!(list["result"]["cacheScope"], "private");
    let tools = list["result"]["tools"].as_array().expect("tools array");
    assert_eq!(tools.len(), 7);
    for (tool, expected) in tools.iter().zip(TOOLS) {
        assert_eq!(tool["name"], expected);
        assert_eq!(tool["annotations"]["readOnlyHint"], true);
        assert_eq!(tool["annotations"]["openWorldHint"], false);
        assert_eq!(tool["inputSchema"]["type"], "object");
        assert_eq!(tool["inputSchema"]["additionalProperties"], false);
        assert_eq!(tool["outputSchema"]["type"], "object");
    }

    assert_eq!(row_for(&exchange, &role.id(3))["error"]["code"], -32601);
    assert_eq!(row_for(&exchange, &role.id(4))["error"]["code"], -32602);
    assert_tool_result(row_for(&exchange, &role.id(5)), &role.id(5), "list_policies", true);
}

#[test]
fn role_a_inspector_style_discover_and_complete_result_typing() {
    discover_and_typing(Role::Inspector);
}

#[test]
fn role_b_claude_code_style_discover_and_complete_result_typing() {
    discover_and_typing(Role::ClaudeCode);
}

fn seven_tools(role: Role) {
    let fixture = Fixture::new();
    let cases: [(&str, Value); 7] = [
        ("list_policies", json!({"limit":50})),
        ("search_requirements", json!({"query":"audit","artifact_key":"catalog","limit":50})),
        ("get_requirement", json!({"artifact_key":"catalog","requirement_id":"synthetic-a"})),
        ("trace_control", json!({"artifact_key":"catalog","control_id":"synthetic-a","limit":50})),
        ("get_recorded_applicability", json!({"artifact_key":"domain","limit":50})),
        ("get_gap_summary", json!({"artifact_key":"report","limit":50})),
        ("get_artifact_status", json!({"artifact_key":"catalog"})),
    ];
    let requests: Vec<Vec<u8>> = cases
        .iter()
        .enumerate()
        .map(|(index, (tool, arguments))| {
            role.call(u64::try_from(index + 1).unwrap(), tool, arguments.clone())
        })
        .collect();
    let exchange = drive(&fixture.args(), &requests, 7);
    assert!(exchange.status.success(), "worker must exit cleanly: {}", exchange.stderr);
    assert_eq!(exchange.rows.len(), 7);
    let data: Vec<&Value> = cases
        .iter()
        .enumerate()
        .map(|(index, (tool, _))| {
            let id = role.id(u64::try_from(index + 1).unwrap());
            let structured = assert_tool_result(row_for(&exchange, &id), &id, tool, true);
            &structured["data"]
        })
        .collect();

    // list_policies: exactly the one visible native policy.
    assert_eq!(data[0]["matched"], 1);
    assert_eq!(data[0]["rows"][0]["artifact_key"], "catalog");
    assert_eq!(data[0]["rows"][0]["lifecycle_state"], "approved");

    // Deterministic search: both controls' captured source lines contain "audit".
    assert_eq!(data[1]["matched"], 2);
    assert_eq!(data[1]["rows"][0]["requirement"]["requirement_id"], "synthetic-a");
    assert_eq!(data[1]["rows"][1]["requirement"]["requirement_id"], "synthetic-b");
    assert_eq!(data[1]["rows"][0]["match_kind"], "all-tokens");

    // Exact requirement with exact captured source span (line 1, no CRLF).
    assert_eq!(data[2]["requirement_id"], "synthetic-a");
    assert_eq!(data[2]["excerpt"], Value::Null);
    let citation = &data[2]["citations"][0];
    assert_eq!(citation["native_pointer"], "/catalog/controls/0");
    assert_eq!(citation["source_state"], "exact-captured-source");
    let line1 = "Private sentinel café access audit.";
    assert_eq!(citation["source_span"], json!({"start_byte":0,"end_byte":line1.len()}));

    // Exact-ID trace: one direct recorded control + three recorded mapping pairs,
    // preserving the original uppercase native mapping UUID spelling.
    assert_eq!(data[3]["matched"], 4);
    assert_eq!(data[3]["rows"][0]["relation"], "exact-recorded-control");
    assert_eq!(data[3]["rows"][0]["mapping_id"], Value::Null);
    for relation in &data[3]["rows"].as_array().unwrap()[1..] {
        assert_eq!(relation["relation"], "recorded-mapping");
        assert_eq!(relation["mapping_id"], json!(fixture.mapping_id));
        assert_eq!(relation["citations"].as_array().unwrap().len(), 3);
    }

    // Recorded applicability: explicit decision vs implicit under-review omission.
    assert_eq!(data[4]["matched"], 2);
    assert_eq!(data[4]["rows"][0]["control_id"], "synthetic-a");
    assert_eq!(data[4]["rows"][0]["decision_state"], "applicable");
    assert_eq!(data[4]["rows"][0]["decision_source"], "explicit-record");
    assert_eq!(data[4]["rows"][1]["control_id"], "synthetic-b");
    assert_eq!(data[4]["rows"][1]["decision_state"], Value::Null);
    assert_eq!(data[4]["rows"][1]["decision_source"], "implicit-under-review");
    assert_eq!(data[4]["rows"][1]["classification"], "under-review");
    // Cross-tool consistency with the maintained report classification.
    assert_eq!(data[4]["rows"][0]["classification"], fixture.report_classifications["synthetic-a"]);

    // Gap summary: full maintained counts independent of paging.
    assert_eq!(data[5]["summary"], fixture.report_counts);
    assert_eq!(data[5]["matched"], 2);

    // Artifact status: recorded approval, raw pin, no schedule without opt-in.
    assert_eq!(data[6]["artifact_key"], "catalog");
    assert_eq!(data[6]["lifecycle_state"], "approved");
    assert_eq!(data[6]["freshness_scope"], "complete-recorded-fingerprint-current");
    assert_eq!(data[6]["schedule_scope"], "not-evaluated");
    assert_eq!(data[6]["schedule"], Value::Null);
    let catalog_bytes = fs::read(fixture.root().join("catalog.json")).unwrap();
    assert_eq!(data[6]["raw_sha256"], json!(sha256_hex(&catalog_bytes)));
    assert_eq!(data[6]["native_identity"]["model"], "catalog");

    // No private prose, paths or owner keys leak into any response.
    let raw = serde_json::to_string(&exchange.rows).unwrap();
    for private in [
        "Private sentinel",
        "Sensitive control title",
        "private-owner",
        "Private applicability rationale",
        "source.txt",
        "Sensitive local record title",
    ] {
        assert!(!raw.contains(private), "private fixture fact disclosed: {private}");
    }
}

#[test]
fn role_a_inspector_style_seven_tools_exact_recorded_results() {
    seven_tools(Role::Inspector);
}

#[test]
fn role_b_claude_code_style_seven_tools_exact_recorded_results() {
    seven_tools(Role::ClaudeCode);
}

#[test]
fn deterministic_search_pagination_and_generation_cursors() {
    let role = Role::Inspector;
    let fixture = Fixture::new();
    let search = json!({"query":"audit","artifact_key":"catalog","limit":50});
    let requests = vec![
        role.call(1, "search_requirements", search.clone()),
        role.call(2, "search_requirements", search.clone()),
        role.call(
            3,
            "search_requirements",
            json!({"query":"audit","artifact_key":"catalog","limit":1}),
        ),
        // Exact-ID search ranks the exact native ID first and alone.
        role.call(
            4,
            "search_requirements",
            json!({"query":"synthetic-a","artifact_key":"catalog"}),
        ),
    ];
    let exchange = drive(&fixture.args(), &requests, 4);
    assert!(exchange.status.success(), "{}", exchange.stderr);
    let first = row_for(&exchange, &role.id(1));
    let second = row_for(&exchange, &role.id(2));
    assert_eq!(
        first["result"]["structuredContent"], second["result"]["structuredContent"],
        "identical queries must produce identical results"
    );
    let generation =
        first["result"]["structuredContent"]["data"]["generation"].as_str().unwrap().to_owned();
    assert_eq!(generation.len(), 64);

    let page1 = &row_for(&exchange, &role.id(3))["result"]["structuredContent"]["data"];
    assert_eq!(page1["matched"], 2);
    assert_eq!(page1["emitted"], 1);
    let cursor = page1["next_cursor"].as_str().expect("continuation cursor").to_owned();
    assert_eq!(cursor, format!("{generation}:1"));

    // Follow the exact cursor, then refuse a foreign-generation cursor.
    let requests = vec![
        role.call(1, "search_requirements", json!({"query":"audit","artifact_key":"catalog","limit":1,"cursor":cursor})),
        role.call(2, "search_requirements", json!({"query":"audit","artifact_key":"catalog","limit":1,"cursor":format!("{}:1","f".repeat(64))})),
    ];
    let exchange = drive(&fixture.args(), &requests, 2);
    assert!(exchange.status.success(), "{}", exchange.stderr);
    let page2 = &row_for(&exchange, &role.id(1))["result"]["structuredContent"]["data"];
    assert_eq!(page2["rows"][0]["requirement"]["requirement_id"], "synthetic-b");
    assert_eq!(page2["next_cursor"], Value::Null);
    let refused = assert_tool_result(
        row_for(&exchange, &role.id(2)),
        &role.id(2),
        "search_requirements",
        false,
    );
    assert_eq!(refused["reason"], "generation-changed");
}

#[test]
fn exact_id_search_ranks_alone() {
    let role = Role::Inspector;
    let fixture = Fixture::new();
    let exchange = drive(
        &fixture.args(),
        &[role.call(
            1,
            "search_requirements",
            json!({"query":"synthetic-a","artifact_key":"catalog"}),
        )],
        1,
    );
    assert!(exchange.status.success(), "{}", exchange.stderr);
    let data = &row_for(&exchange, &role.id(1))["result"]["structuredContent"]["data"];
    // Exact ID ranks first and alone in its rank; weaker some-token matches follow.
    assert_eq!(data["matched"], 2);
    assert_eq!(data["rows"][0]["match_kind"], "exact-id");
    assert_eq!(data["rows"][0]["rank"], 1);
    assert_eq!(data["rows"][0]["requirement"]["requirement_id"], "synthetic-a");
    assert_eq!(data["rows"][1]["match_kind"], "some-tokens");
    assert_eq!(data["rows"][1]["requirement"]["requirement_id"], "synthetic-b");
}

fn null_content_rules(role: Role) {
    let fixture = Fixture::new();
    // Static-only worker: no decision pair -> every tool is closed unavailable.
    let arguments = [
        json!({}),
        json!({"query":"audit"}),
        json!({"artifact_key":"catalog","requirement_id":"synthetic-a"}),
        json!({"artifact_key":"catalog","control_id":"synthetic-a"}),
        json!({"artifact_key":"domain"}),
        json!({"artifact_key":"report"}),
        json!({"artifact_key":"catalog"}),
    ];
    let requests: Vec<Vec<u8>> = TOOLS
        .iter()
        .zip(arguments)
        .enumerate()
        .map(|(index, (tool, arguments))| {
            role.call(u64::try_from(index + 1).unwrap(), tool, arguments)
        })
        .collect();
    let exchange = drive(&fixture.static_args(), &requests, 7);
    assert!(exchange.status.success(), "{}", exchange.stderr);
    assert_eq!(exchange.rows.len(), 7);
    for (index, tool) in TOOLS.iter().enumerate() {
        let id = role.id(u64::try_from(index + 1).unwrap());
        let structured = assert_tool_result(row_for(&exchange, &id), &id, tool, false);
        assert_eq!(structured["reason"], "approval-unavailable");
    }

    // Valid scope: absent selection and refused excerpt remain null-content errors.
    let requests = vec![
        role.call(
            1,
            "get_requirement",
            json!({"artifact_key":"catalog","requirement_id":"absent-control"}),
        ),
        role.call(
            2,
            "get_requirement",
            json!({"artifact_key":"catalog","requirement_id":"synthetic-a","include_excerpt":true}),
        ),
        role.call(3, "get_artifact_status", json!({"artifact_key":"not-visible"})),
        role.call(4, "trace_control", json!({"artifact_key":"source","control_id":"synthetic-a"})),
    ];
    let exchange = drive(&fixture.args(), &requests, 4);
    assert!(exchange.status.success(), "{}", exchange.stderr);
    let reasons = ["not-found", "visibility-refused", "visibility-refused", "unsupported-role"];
    let tools = ["get_requirement", "get_requirement", "get_artifact_status", "trace_control"];
    for (index, (reason, tool)) in reasons.iter().zip(tools).enumerate() {
        let id = role.id(u64::try_from(index + 1).unwrap());
        let structured = assert_tool_result(row_for(&exchange, &id), &id, tool, false);
        assert_eq!(structured["reason"], *reason);
    }
}

#[test]
fn role_a_inspector_style_null_content_rules() {
    null_content_rules(Role::Inspector);
}

#[test]
fn role_b_claude_code_style_null_content_rules() {
    null_content_rules(Role::ClaudeCode);
}

#[test]
fn request_specific_namespaced_metadata_is_not_cached() {
    let role = Role::Inspector;
    let fixture = Fixture::new();
    // Second request omits _meta entirely; the first must not authorize it.
    let mut bare = serde_json::to_vec(&json!({
        "jsonrpc":"2.0","id":role.id(2),"method":"server/discover","params":{}}))
    .unwrap();
    bare.push(b'\n');
    let mut wrong_version = role.request(3, "server/discover", json!({}));
    // Rewrite the revision inside the third request's metadata.
    let text = String::from_utf8(wrong_version.clone()).unwrap().replace(REVISION, "2025-06-18");
    wrong_version = text.into_bytes();
    let exchange = drive(
        &fixture.static_args(),
        &[role.request(1, "server/discover", json!({})), bare, wrong_version],
        3,
    );
    assert!(exchange.status.success(), "{}", exchange.stderr);
    assert_eq!(exchange.rows.len(), 3);
    assert_eq!(row_for(&exchange, &role.id(1))["result"]["resultType"], "complete");
    assert_eq!(row_for(&exchange, &role.id(2))["error"]["code"], -32602);
    let version_error = row_for(&exchange, &role.id(3));
    assert_eq!(version_error["error"]["code"], -32022);
    assert_eq!(version_error["error"]["data"]["supported"], json!([REVISION]));
    assert_eq!(version_error["error"]["data"]["requested"], "2025-06-18");
}

#[test]
fn bounded_input_and_admission_limits() {
    let role = Role::Inspector;
    let fixture = Fixture::new();
    // Oversized frame (>64 KiB) is drained and refused without killing the worker.
    let mut oversized = vec![b'a'; 70_000];
    oversized.push(b'\n');
    let exchange = drive(
        &fixture.static_args(),
        &[
            oversized,
            role.request(1, "server/discover", json!({})),
            role.call(2, "list_policies", json!({"limit":51})),
            role.call(3, "list_policies", json!({"limit":1,"unknown_argument":true})),
            role.request(4, "resources/list", json!({})),
        ],
        5,
    );
    assert!(exchange.status.success(), "{}", exchange.stderr);
    assert_eq!(exchange.rows.len(), 5);
    assert_eq!(exchange.rows[0]["error"]["code"], -32700);
    assert!(exchange.rows[0].get("id").is_none(), "unparseable frame must not manufacture an ID");
    assert_eq!(row_for(&exchange, &role.id(1))["result"]["resultType"], "complete");
    assert_eq!(row_for(&exchange, &role.id(2))["error"]["code"], -32602);
    assert_eq!(row_for(&exchange, &role.id(3))["error"]["code"], -32602);
    assert_eq!(row_for(&exchange, &role.id(4))["error"]["code"], -32601);
}

#[test]
fn duplicate_accepted_id_retires_the_transport() {
    let role = Role::Inspector;
    let fixture = Fixture::new();
    let exchange = drive_burst(
        &fixture.static_args(),
        &[
            role.request(1, "server/discover", json!({})),
            role.request(1, "server/discover", json!({})),
            role.request(2, "server/discover", json!({})),
        ],
        1,
    );
    assert!(!exchange.status.success(), "duplicate accepted ID must retire the transport");
    // Retirement shuts down before any queued work executes: nothing may publish.
    assert_eq!(exchange.rows.len(), 0, "no work after retirement may publish");
    assert!(exchange.stderr.contains("repeated accepted request ID"), "{}", exchange.stderr);
}

#[test]
fn bounded_queue_capacity_refuses_the_fourth_concurrent_request() {
    let role = Role::Inspector;
    let fixture = Fixture::new();
    // One burst: one active slot + two queued; the fourth is refused with the
    // declared local capacity code before any retained growth.
    let requests: Vec<Vec<u8>> =
        (1..=4).map(|n| role.call(n, "list_policies", json!({}))).collect();
    let exchange = drive_burst(&fixture.args(), &requests, 4);
    assert!(exchange.status.success(), "{}", exchange.stderr);
    assert_eq!(exchange.rows.len(), 4);
    let capacity = row_for(&exchange, &role.id(4));
    assert_eq!(capacity["error"]["code"], -32000);
    for n in 1..=3 {
        let id = role.id(n);
        assert_tool_result(row_for(&exchange, &id), &id, "list_policies", true);
    }
}

#[test]
fn cancellation_suppresses_the_response_and_keeps_the_worker_alive() {
    let role = Role::ClaudeCode;
    let fixture = Fixture::new();
    let mut cancel = serde_json::to_vec(&json!({
        "jsonrpc":"2.0","method":"notifications/cancelled",
        "params":{"requestId":role.id(7),"reason":"client withdrew interest"}}))
    .unwrap();
    cancel.push(b'\n');
    // Burst the request + cancellation together so the notification is admitted
    // before the worker publishes, then confirm the worker stays alive.
    let exchange = drive_burst(
        &fixture.args(),
        &[
            role.call(7, "get_gap_summary", json!({"artifact_key":"report"})),
            cancel,
            role.request(8, "server/discover", json!({})),
        ],
        1,
    );
    assert!(exchange.status.success(), "{}", exchange.stderr);
    assert_eq!(exchange.rows.len(), 1, "cancelled work must publish no response");
    assert_eq!(exchange.rows[0]["id"], role.id(8));
    assert_eq!(exchange.rows[0]["result"]["resultType"], "complete");
}

#[test]
fn confined_roots_refuse_overlap_and_symlinked_ancestry() {
    let role = Role::Inspector;
    let fixture = Fixture::new();
    let call = role.call(1, "get_artifact_status", json!({"artifact_key":"catalog"}));

    // Decision root nested inside the project root: the gate fails closed.
    let nested = fixture.root().join("nested-decisions");
    fs::create_dir(&nested).unwrap();
    fs::copy(
        fixture.decision_root().join("forge.mcp.disclosure-decision.json"),
        nested.join("forge.mcp.disclosure-decision.json"),
    )
    .unwrap();
    let args = vec![
        "mcp".into(),
        "serve".into(),
        "--project".into(),
        fixture.root().display().to_string(),
        "--decision-root".into(),
        nested.display().to_string(),
        "--decision-sha256".into(),
        fixture.decision_pin.clone(),
        "--profile-sha256".into(),
        fixture.profile_pin.clone(),
    ];
    let exchange = drive(&args, &[role.request(2, "server/discover", json!({})), call.clone()], 2);
    assert!(exchange.status.success(), "{}", exchange.stderr);
    assert_eq!(
        row_for(&exchange, &role.id(2))["result"]["resultType"],
        "complete",
        "static discovery never opens either root"
    );
    let structured = assert_tool_result(
        row_for(&exchange, &role.id(1)),
        &role.id(1),
        "get_artifact_status",
        false,
    );
    assert_eq!(structured["reason"], "approval-unavailable");

    // Symlinked project root spelling: O_NOFOLLOW ancestry refuses the gate.
    let link = fixture.project.path().join("alias-root");
    std::os::unix::fs::symlink(fixture.root(), &link).unwrap();
    let args = vec![
        "mcp".into(),
        "serve".into(),
        "--project".into(),
        link.display().to_string(),
        "--decision-root".into(),
        fixture.decision_root().display().to_string(),
        "--decision-sha256".into(),
        fixture.decision_pin.clone(),
        "--profile-sha256".into(),
        fixture.profile_pin.clone(),
    ];
    let exchange = drive(&args, &[call], 1);
    assert!(exchange.status.success(), "{}", exchange.stderr);
    let structured = assert_tool_result(
        row_for(&exchange, &role.id(1)),
        &role.id(1),
        "get_artifact_status",
        false,
    );
    assert_eq!(structured["reason"], "approval-unavailable");
}

#[test]
fn wrong_raw_pin_fails_closed_without_private_diagnostics() {
    let role = Role::Inspector;
    let fixture = Fixture::new();
    let args = vec![
        "mcp".into(),
        "serve".into(),
        "--project".into(),
        fixture.root().display().to_string(),
        "--decision-root".into(),
        fixture.decision_root().display().to_string(),
        "--decision-sha256".into(),
        "0".repeat(64),
        "--profile-sha256".into(),
        fixture.profile_pin.clone(),
    ];
    let exchange =
        drive(&args, &[role.call(1, "get_artifact_status", json!({"artifact_key":"catalog"}))], 1);
    assert!(exchange.status.success(), "{}", exchange.stderr);
    let structured = assert_tool_result(
        row_for(&exchange, &role.id(1)),
        &role.id(1),
        "get_artifact_status",
        false,
    );
    assert_eq!(structured["reason"], "approval-unavailable");
    assert!(!exchange.stderr.contains("synthetic"), "terminal errors must not echo fixture facts");
}
