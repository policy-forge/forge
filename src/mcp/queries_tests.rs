//! Query controls over actual file-backed captured originals.
//!
//! Fixture writers originate from the frozen gate's synthetic helpers; selected
//! native nodes add genuine maintained trace properties. All owner assertions are
//! explicitly synthetic; these fixtures provide no real owner or client acceptance.

use super::super::disclosure::{DisclosureGate, load_disclosure_decision};
use super::super::{MANIFEST_PATH, MANIFEST_PROFILE, ProjectDeclaration, ResourcePolicy};
use super::*;
use crate::hashing::sha256_hex;
use crate::lifecycle::record::{
    self, APPROVAL_POLICY_VERSION, ApprovalPolicy, ArtifactFingerprint, DeclaredRole,
    FingerprintSet, LifecycleState, NamedHash, Party, PolicyIdentity, ReviewSchedule,
    RoleRequirement, SCHEMA_VERSION, SeparationRules, TimezonePolicy, TransitionEvent,
};
use crate::workspace::preparation::{Interruption, NoopControl, test_support::Recorder};
use chrono::NaiveDate;
use serde_json::{Value, json};
use std::fs;
use tempfile::TempDir;

/// Fixed already-approved draft binding; fixture text is never genuine human provenance.
const PROPOSAL: &str = "d52da047f0166d9d8cd002221ee9608577f935e7c803969a178b91e797801f49";
/// Exact selected protocol binding only; these controls implement no protocol.
const PROTOCOL: &str = "2026-07-28";
/// Actual fixed filenames used by the complete gate's two-root loader.
const CONFIG_PATHS: [&str; 3] =
    ["forge.mcp.json", "forge.mcp.visibility.json", "forge.mcp.disclosure-decision.json"];

/// Genuine synthetic local originals and independently declared caller policy for tests.
struct TestProject {
    /// Owned ordinary temporary project, never a production or authoritative source.
    directory: TempDir,
    /// Complete caller declaration copied into the independently pinned local manifest.
    declaration: ProjectDeclaration,
}

impl TestProject {
    /// Write a fully native-valid Catalog, policy source and real intrinsic lifecycle history.
    #[allow(clippy::too_many_lines)] // Complete genuine source/native/lifecycle fixture remains visible together.
    fn new(approved: bool) -> Self {
        let directory = tempfile::tempdir().expect("synthetic project directory");
        let source =
            "Private sentinel café access audit.\r\nSecond monitoring audit line.\n".as_bytes();
        let catalog = json!({"catalog":{
            "uuid":"11111111-1111-4111-8111-111111111111",
            "metadata":{"title":"Synthetic private instruction title", "last-modified":"2026-09-01T00:00:00Z", "version":"1", "oscal-version":"1.2.3"},
            "controls":[
                {"id":"synthetic-a", "title":"Sensitive control title", "props":[
                    {"name":"source-file","ns":crate::oscal::trace_embedding::FORGE_TRACE_NS,"value":"source.txt"},
                    {"name":"source-section","ns":crate::oscal::trace_embedding::FORGE_TRACE_NS,"value":"section-one"},
                    {"name":"source-line","ns":crate::oscal::trace_embedding::FORGE_TRACE_NS,"value":"1"}]},
                {"id":"synthetic-b", "title":"Sensitive other title", "props":[
                    {"name":"source-file","ns":crate::oscal::trace_embedding::FORGE_TRACE_NS,"value":"source.txt"},
                    {"name":"source-section","ns":crate::oscal::trace_embedding::FORGE_TRACE_NS,"value":"section-two"},
                    {"name":"source-line","ns":crate::oscal::trace_embedding::FORGE_TRACE_NS,"value":"2"}]}
            ]
        }});
        let catalog_bytes = serde_json::to_vec(&catalog).expect("native fixture JSON");
        fs::write(directory.path().join("source.txt"), source).expect("actual source fixture");
        fs::write(directory.path().join("catalog.json"), &catalog_bytes)
            .expect("actual native fixture");
        let mut lifecycle = record::LifecycleRecord {
            schema_version: SCHEMA_VERSION.to_owned(),
            policy: PolicyIdentity {
                policy_key: "synthetic-policy".to_owned(),
                version_key: "version-1".to_owned(),
                title: "Sensitive local record title".to_owned(),
                owner_keys: vec!["private-owner".to_owned()],
                source: ArtifactFingerprint {
                    path: "source.txt".to_owned(),
                    sha256: sha256_hex(source),
                    oscal_type: None,
                    root_uuid: None,
                },
                generated_artifacts: vec![ArtifactFingerprint {
                    path: "catalog.json".to_owned(),
                    sha256: sha256_hex(&catalog_bytes),
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
                next_review_date: NaiveDate::from_ymd_opt(2026, 12, 1).expect("synthetic date"),
                due_soon_days: 7,
                timezone_policy: TimezonePolicy::DateOnly,
            },
            state: LifecycleState::Draft,
            replaced_by: None,
            history: Vec::new(),
        };
        if approved {
            let fingerprints = FingerprintSet {
                source_sha256: sha256_hex(source),
                generated_artifacts: vec![NamedHash {
                    path: "catalog.json".to_owned(),
                    sha256: sha256_hex(&catalog_bytes),
                }],
            };
            append_event(&mut lifecycle, LifecycleState::InReview, &fingerprints);
            append_event(&mut lifecycle, LifecycleState::Approved, &fingerprints);
        }
        record::validate(&lifecycle).expect("intrinsically admitted fixture record");
        let lifecycle_bytes = serde_json::to_vec(&lifecycle).expect("real lifecycle fixture JSON");
        fs::write(directory.path().join("lifecycle.json"), &lifecycle_bytes)
            .expect("actual record fixture");
        let native_identity = NativeIdentity {
            model: "catalog".to_owned(),
            root_id: "11111111-1111-4111-8111-111111111111".to_owned(),
            document_version: "1".to_owned(),
            oscal_version: "1.2.3".to_owned(),
        };
        let declaration = ProjectDeclaration {
            schema_version: MANIFEST_PROFILE.to_owned(),
            project_key: "synthetic-project".to_owned(),
            resources: vec![
                ResourcePolicy {
                    key: "catalog".to_owned(),
                    role: Role::OscalCatalogArtifact,
                    path: "catalog.json".to_owned(),
                    expected_sha256: sha256_hex(&catalog_bytes),
                    native_identity: Some(native_identity),
                    lifecycle_key: Some("lifecycle".to_owned()),
                    citation_label: "synthetic-catalog".to_owned(),
                },
                ResourcePolicy {
                    key: "source".to_owned(),
                    role: Role::PolicySource,
                    path: "source.txt".to_owned(),
                    expected_sha256: sha256_hex(source),
                    native_identity: None,
                    lifecycle_key: Some("lifecycle".to_owned()),
                    citation_label: "synthetic-source".to_owned(),
                },
                ResourcePolicy {
                    key: "lifecycle".to_owned(),
                    role: Role::LifecycleRecord,
                    path: "lifecycle.json".to_owned(),
                    expected_sha256: sha256_hex(&lifecycle_bytes),
                    native_identity: None,
                    lifecycle_key: None,
                    citation_label: "synthetic-lifecycle".to_owned(),
                },
            ],
        };
        let project = Self { directory, declaration };
        project.write_manifest();
        project
    }

    /// Return the actual canonical known fixture root, not an erased unsafe user spelling.
    fn root(&self) -> std::path::PathBuf {
        self.directory.path().canonicalize().expect("known ordinary fixture root")
    }

    /// Publish exact synthetic expected declaration before actual production capture.
    fn write_manifest(&self) {
        fs::write(
            self.directory.path().join(MANIFEST_PATH),
            serde_json::to_vec(&self.declaration).expect("closed caller declaration"),
        )
        .expect("ordinary fixture manifest");
    }
}

/// Use the real deterministic lifecycle event-ID function; no forged approval schema fixture.
fn append_event(
    record: &mut record::LifecycleRecord,
    next: LifecycleState,
    fingerprints: &FingerprintSet,
) {
    let sequence = u32::try_from(record.history.len() + 1).expect("bounded synthetic history");
    let (actor, role) = if next == LifecycleState::InReview {
        ("private-reviewer", DeclaredRole::Reviewer)
    } else {
        ("private-approver", DeclaredRole::Approver)
    };
    let mut event = TransitionEvent {
        sequence,
        event_id: String::new(),
        legacy_event_id: None,
        previous_state: record.state,
        next_state: next,
        actor_key: actor.to_owned(),
        declared_role: role,
        timestamp: format!("2026-09-01T{sequence:02}:00:00Z"),
        rationale: "Private rationale remains undisclosed".to_owned(),
        fingerprints: fingerprints.clone(),
        assertions: Vec::new(),
        impact_finding_ids: Vec::new(),
        replacement: None,
    };
    event.event_id = record::event_id(record, &event).expect("actual intrinsic event ID");
    record.history.push(event);
    record.state = next;
}

/// Genuine local project/control roots with clearly synthetic declared-owner assertions.
struct GateFixture {
    /// Actual native/source/lifecycle test originals from the retained core fixture.
    project: TestProject,
    /// Separate actual temporary decision directory, never an owner-authenticated store.
    decision_directory: TempDir,
    /// Complete selected private test profile, not a request-time scope override.
    profile: Value,
    /// Complete private synthetic record; only the gate's file-backed loader consumes it.
    decision: Value,
    /// Actual raw selected profile pin after writing this explicit fixture generation.
    profile_pin: String,
    /// Actual raw selected record pin after writing this explicit fixture generation.
    decision_pin: String,
}

impl GateFixture {
    /// Write complete fixed configs over genuine native/lifecycle test originals.
    fn new() -> Self {
        let project = TestProject::new(true);
        let discovery_pin =
            sha256_hex(&fs::read(project.directory.path().join(MANIFEST_PATH)).unwrap());
        let profile = json!({
            "schema_version":"forge.mcp-visibility/1", "project_key":"synthetic-project",
            "discovery_sha256":discovery_pin, "enabled_tools":["list_policies","search_requirements","get_requirement","trace_control","get_recorded_applicability","get_gap_summary","get_artifact_status"],
            "visible_resource_keys":["catalog","source"], "noncurrent_metadata_keys":[],
            "source_text_keys":[], "schedule_keys":[], "evidence_metadata_keys":[],
            "static_report_keys":[], "search_index_key":null,
            "identity_disclosure":"bounded-token-only",
            "approved_current_basis":"caller-established-recorded-policy",
            "source_text_mode":"omit", "schedule_mode":"omit", "as_of":null,
            "report_content_mode":"minimized-metadata", "evidence_bytes":false,
            "network":false, "process_execution":false
        });
        let rows: Vec<_> = [("D067-P1","product"),("D067-P2","product"),
            ("D067-S1","security"),("D067-E1","engineering"),
            ("D067-S2","product"),("D067-S2","security")].into_iter().map(|(subject,role)| {
            json!({"subject":subject,"role":role,"disposition":"approved",
                "declared_owner_key":"synthetic-declared-owner", "recorded_on":"2026-10-04",
                "source_record_key":"synthetic-private-record", "source_record_sha256":"a".repeat(64),
                "proposal_index_sha256":PROPOSAL, "profile_sha256":"b".repeat(64),
                "discovery_sha256":discovery_pin})
        }).collect();
        let decision = json!({"schema_version":"forge.mcp-disclosure-decision/1",
            "decision_key":"synthetic-decision", "project_key":"synthetic-project",
            "discovery_sha256":discovery_pin, "profile_sha256":"b".repeat(64),
            "proposal_index_sha256":PROPOSAL,"protocol_revision":PROTOCOL,
            "operator_boundary":"operator-selected-declared-owner-record",
            "visibility_profile":profile, "policy_decisions":rows,
            "evaluation_plan":{"state":"pending","declared_owner_key":null,"recorded_on":null,
                "source_record_key":null,"source_record_sha256":null,
                "corpus_acceptance":"unearned","threshold_acceptance":"unearned",
                "client_acceptance":"unearned"}});
        let mut fixture = Self {
            project,
            decision_directory: tempfile::tempdir().unwrap(),
            profile,
            decision,
            profile_pin: String::new(),
            decision_pin: String::new(),
        };
        fixture.write_selected_configs();
        fixture
    }
    /// Canonicalize only a known ordinary test directory before passing the raw production API.
    fn decision_root(&self) -> std::path::PathBuf {
        self.decision_directory.path().canonicalize().unwrap()
    }
    /// Explicitly publish a new selected test generation; this is not automatic production retry.
    fn write_selected_configs(&mut self) {
        let discovery_pin =
            sha256_hex(&fs::read(self.project.directory.path().join(MANIFEST_PATH)).unwrap());
        self.profile["discovery_sha256"] = json!(discovery_pin);
        let profile_bytes = serde_json::to_vec(&self.profile).unwrap();
        self.profile_pin = sha256_hex(&profile_bytes);
        fs::write(self.project.directory.path().join(CONFIG_PATHS[1]), profile_bytes).unwrap();
        self.decision["profile_sha256"] = json!(self.profile_pin);
        self.decision["discovery_sha256"] = json!(discovery_pin);
        self.decision["visibility_profile"] = self.profile.clone();
        for row in self.decision["policy_decisions"].as_array_mut().unwrap() {
            row["profile_sha256"] = json!(self.profile_pin);
            row["discovery_sha256"] = json!(discovery_pin);
        }
        self.write_decision_only();
    }
    /// Write only the chosen private test record, retaining deliberate profile/scope conflicts.
    fn write_decision_only(&mut self) {
        let bytes = serde_json::to_vec(&self.decision).unwrap();
        self.decision_pin = sha256_hex(&bytes);
        fs::write(self.decision_directory.path().join(CONFIG_PATHS[2]), bytes).unwrap();
    }
    /// Invoke only the actual complete gate with this fixture's selected original pins.
    fn load(&self) -> DisclosureGate {
        load_disclosure_decision(
            &self.project.root(),
            Some(&self.decision_root()),
            Some(&self.decision_pin),
            &self.profile_pin,
            &mut NoopControl,
        )
        .unwrap()
    }
    /// Retain the actual opaque scope without manufacturing a detached proof constructor.
    fn scope(&self) -> CapturedQueryScope {
        match self.load() {
            DisclosureGate::Validated(scope) => *scope,
            DisclosureGate::Unavailable => panic!("fixture unavailable"),
        }
    }
}

/// Deliberately publish a new synthetic source/native/lifecycle generation before capture.
/// This uses actual intrinsic event IDs and current bytes, rather than detached proof fields.
impl GateFixture {
    /// Refresh the full recorded source/generated tuple after explicit authored fixture changes.
    fn refresh(&mut self) {
        let root = self.project.directory.path();
        let source = fs::read(root.join("source.txt")).unwrap();
        let native_bytes = fs::read(root.join("catalog.json")).unwrap();
        let native: Value = serde_json::from_slice(&native_bytes).unwrap();
        let kind = if native.get("catalog").is_some() { "catalog" } else { "component-definition" };
        let mut lifecycle = record::parse(&fs::read(root.join("lifecycle.json")).unwrap()).unwrap();
        lifecycle.policy.source.sha256 = sha256_hex(&source);
        lifecycle.policy.generated_artifacts[0].sha256 = sha256_hex(&native_bytes);
        lifecycle.policy.generated_artifacts[0].oscal_type = Some(kind.to_owned());
        lifecycle.policy.generated_artifacts[0].root_uuid =
            Some(native[kind]["uuid"].as_str().unwrap().to_owned());
        lifecycle.history.clear();
        lifecycle.state = LifecycleState::Draft;
        let fingerprints = FingerprintSet {
            source_sha256: sha256_hex(&source),
            generated_artifacts: vec![NamedHash {
                path: "catalog.json".to_owned(),
                sha256: sha256_hex(&native_bytes),
            }],
        };
        append_event(&mut lifecycle, LifecycleState::InReview, &fingerprints);
        append_event(&mut lifecycle, LifecycleState::Approved, &fingerprints);
        record::validate(&lifecycle).unwrap();
        let record_bytes = serde_json::to_vec(&lifecycle).unwrap();
        fs::write(root.join("lifecycle.json"), &record_bytes).unwrap();
        for resource in &mut self.project.declaration.resources {
            resource.expected_sha256 = sha256_hex(&fs::read(root.join(&resource.path)).unwrap());
            if resource.key == "catalog" {
                resource.role = if kind == "catalog" {
                    Role::OscalCatalogArtifact
                } else {
                    Role::OscalComponentArtifact
                };
                resource.native_identity = Some(NativeIdentity {
                    model: kind.to_owned(),
                    root_id: native[kind]["uuid"].as_str().unwrap().to_owned(),
                    document_version: native[kind]["metadata"]["version"]
                        .as_str()
                        .unwrap()
                        .to_owned(),
                    oscal_version: native[kind]["metadata"]["oscal-version"]
                        .as_str()
                        .unwrap()
                        .to_owned(),
                });
            }
        }
        self.project.write_manifest();
        self.write_selected_configs();
    }
    /// Mutate only authored native fixture bytes, then deliberately refresh the full selected inputs.
    fn change_native(&mut self, change: impl FnOnce(&mut Value)) {
        let path = self.project.directory.path().join("catalog.json");
        let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        change(&mut value);
        fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
        self.refresh();
    }
}

/// Select one exact first-page control under the public page bounds.
fn page(limit: usize) -> Page {
    Page { cursor: None, limit }
}

/// Obtain complete encoded DTOs only through actual preparation, never a data constructor.
fn available(scope: &CapturedQueryScope, query: Query) -> Value {
    let prepared = prepare(scope, query, &mut NoopControl)
        .unwrap_or_else(|_| panic!("expected actual captured query"));
    serde_json::to_value(prepared.response()).unwrap()
}

/// Assert a fixed typed refusal without exposing private underlying error messages.
#[allow(clippy::needless_pass_by_value)] // Consume and drop the actual prepared proof before the fixture is retired.
fn refused(result: QueryResult<PreparedQuery<'_>>, reason: Reason) {
    match result {
        Err(QueryError::Unavailable(found)) => {
            assert_eq!(serde_json::to_value(found).unwrap(), serde_json::to_value(reason).unwrap());
        }
        _ => panic!("expected fixed unavailable query"),
    }
}

/// Genuine native/source proof supplies exact native pointer, UTF-8 offsets and default privacy.
#[test]
fn captured_requirement_has_exact_pointer_span_and_no_private_prose() {
    let fixture = GateFixture::new();
    let scope = fixture.scope();
    let actual = available(
        &scope,
        Query::GetRequirement {
            artifact_key: "catalog".into(),
            requirement_id: "synthetic-a".into(),
            include_excerpt: false,
        },
    );
    assert_eq!(actual["availability"], "available");
    assert_eq!(actual["data"]["requirement_id"], "synthetic-a");
    let citation = &actual["data"]["citations"][0];
    assert_eq!(citation["native_pointer"], "/catalog/controls/0");
    assert_eq!(citation["source_key"], "source");
    let source = fs::read(fixture.project.directory.path().join("source.txt")).unwrap();
    let first_end = source.windows(2).position(|pair| pair == b"\r\n").unwrap();
    assert_eq!(citation["source_raw_sha256"], sha256_hex(&source));
    assert_eq!(citation["source_span"], json!({"start_byte":0,"end_byte":first_end}));
    assert_eq!(actual["data"]["excerpt"], Value::Null);
    let encoded = serde_json::to_string(&actual).unwrap();
    for private in [
        "Private sentinel",
        "Sensitive control",
        "private-owner",
        "Private rationale",
        "source.txt",
    ] {
        assert!(!encoded.contains(private));
    }
}

/// Search evaluates actual native IDs and captured lexical text, with complete matching counts.
#[test]
fn lexical_search_is_deterministic_and_does_not_echo_matching_text() {
    let fixture = GateFixture::new();
    let scope = fixture.scope();
    let actual = available(
        &scope,
        Query::SearchRequirements {
            query: "MONITORING audit".into(),
            artifact_key: None,
            include_excerpt: false,
            page: page(50),
        },
    );
    assert_eq!(actual["data"]["matched"], 2);
    assert_eq!(actual["data"]["emitted"], 2);
    assert_eq!(actual["data"]["rows"][0]["requirement"]["requirement_id"], "synthetic-b");
    assert_eq!(actual["data"]["rows"][0]["match_kind"], "all-tokens");
    assert_eq!(actual["data"]["rows"][1]["match_kind"], "some-tokens");
    assert!(!serde_json::to_string(&actual).unwrap().contains("monitoring audit line"));
    let exact = available(
        &scope,
        Query::SearchRequirements {
            query: "synthetic-a".into(),
            artifact_key: None,
            include_excerpt: false,
            page: page(50),
        },
    );
    assert_eq!(exact["data"]["rows"][0]["match_kind"], "exact-id");
    assert_eq!(exact["data"]["rows"][0]["requirement"]["requirement_id"], "synthetic-a");
    let empty = available(
        &scope,
        Query::SearchRequirements {
            query: "unmatched".into(),
            artifact_key: None,
            include_excerpt: false,
            page: page(50),
        },
    );
    assert_eq!(empty["data"]["matched"], 0);
    assert_eq!(empty["data"]["rows"], json!([]));
}

/// A genuine direct control trace retains its full source citation without inventing mappings.
#[test]
fn direct_trace_and_policy_list_use_actual_capture() {
    let fixture = GateFixture::new();
    let scope = fixture.scope();
    let trace = available(
        &scope,
        Query::TraceControl {
            artifact_key: "catalog".into(),
            control_id: "synthetic-b".into(),
            page: page(50),
        },
    );
    assert_eq!(trace["data"]["matched"], 1);
    assert_eq!(trace["data"]["rows"][0]["mapping_id"], Value::Null);
    assert_eq!(trace["data"]["rows"][0]["relation"], "exact-recorded-control");
    assert_eq!(trace["data"]["rows"][0]["citations"][0]["native_pointer"], "/catalog/controls/1");
    let listed = available(&scope, Query::ListPolicies(page(50)));
    assert_eq!(listed["data"]["matched"], 1);
    assert_eq!(listed["data"]["rows"][0]["artifact_key"], "catalog");
    assert_eq!(listed["data"]["rows"][0]["schedule"], Value::Null);
}

/// Explicit profile opt-in returns an exact bounded UTF-8 substring and an untrusted marker.
#[test]
fn opted_excerpt_is_exact_utf8_and_requires_both_scopes() {
    let mut fixture = GateFixture::new();
    let first = "é".repeat(300);
    fs::write(
        fixture.project.directory.path().join("source.txt"),
        format!("{first}\nSecond audit.\n"),
    )
    .unwrap();
    fixture.refresh();
    let denied = fixture.scope();
    refused(
        prepare(
            &denied,
            Query::GetRequirement {
                artifact_key: "catalog".into(),
                requirement_id: "synthetic-a".into(),
                include_excerpt: true,
            },
            &mut NoopControl,
        ),
        Reason::VisibilityRefused,
    );
    fixture.profile["source_text_mode"] = json!("exact-span-opt-in");
    fixture.profile["source_text_keys"] = json!(["source"]);
    fixture.write_selected_configs();
    let scope = fixture.scope();
    let actual = available(
        &scope,
        Query::GetRequirement {
            artifact_key: "catalog".into(),
            requirement_id: "synthetic-a".into(),
            include_excerpt: true,
        },
    );
    assert_eq!(
        actual["data"]["excerpt"]["text"].as_str().unwrap().as_bytes(),
        &first.as_bytes()[..512]
    );
    assert_eq!(actual["data"]["excerpt"]["trust"], "untrusted-content");
    assert_eq!(
        actual["data"]["excerpt"]["citation"]["source_span"],
        json!({"start_byte":0,"end_byte":512})
    );
    assert_eq!(
        available(
            &scope,
            Query::GetRequirement {
                artifact_key: "catalog".into(),
                requirement_id: "synthetic-a".into(),
                include_excerpt: false
            }
        )["data"]["excerpt"],
        Value::Null
    );
}

/// Component implemented-requirement UUIDs are actual IDs and retain their separate control relation.
#[test]
fn component_requirement_uuid_is_not_a_synthesized_composite_key() {
    let mut fixture = GateFixture::new();
    let props = json!([
        {"name":"source-file","ns":crate::oscal::trace_embedding::FORGE_TRACE_NS,"value":"source.txt"},
        {"name":"source-section","ns":crate::oscal::trace_embedding::FORGE_TRACE_NS,"value":"component"},
        {"name":"source-line","ns":crate::oscal::trace_embedding::FORGE_TRACE_NS,"value":"1"}]);
    let native = json!({"component-definition":{"uuid":"11111111-1111-4111-8111-111111111111",
        "metadata":{"title":"Sensitive component","last-modified":"2026-09-01T00:00:00Z","version":"1","oscal-version":"1.2.3"},
        "components":[{"uuid":"22222222-2222-4222-8222-222222222222","type":"software","title":"Sensitive title","description":"Sensitive description",
            "control-implementations":[{"uuid":"33333333-3333-4333-8333-333333333333","source":"urn:synthetic-framework","description":"Private rationale",
                "implemented-requirements":[{"uuid":"44444444-4444-4444-8444-444444444444","control-id":"synthetic-a","description":"Private implemented prose","props":props}]}]}]}});
    fs::write(
        fixture.project.directory.path().join("catalog.json"),
        serde_json::to_vec(&native).unwrap(),
    )
    .unwrap();
    fixture.refresh();
    let scope = fixture.scope();
    let actual = available(
        &scope,
        Query::GetRequirement {
            artifact_key: "catalog".into(),
            requirement_id: "44444444-4444-4444-8444-444444444444".into(),
            include_excerpt: false,
        },
    );
    assert_eq!(actual["data"]["requirement_id"], "44444444-4444-4444-8444-444444444444");
    assert_eq!(
        actual["data"]["citations"][0]["native_pointer"],
        "/component-definition/components/0/control-implementations/0/implemented-requirements/0"
    );
    let direct = available(
        &scope,
        Query::TraceControl {
            artifact_key: "catalog".into(),
            control_id: "synthetic-a".into(),
            page: page(50),
        },
    );
    assert_eq!(direct["data"]["rows"][0]["requirement_id"], "44444444-4444-4444-8444-444444444444");
}

/// Unsupported or ambiguous trace facts never fall back to title search or an undeclared file.
#[test]
fn missing_duplicate_and_outside_source_relations_refuse_whole_query() {
    for case in 0..3 {
        let mut fixture = GateFixture::new();
        fixture.change_native(|native| {
            let props = native["catalog"]["controls"][0]["props"].as_array_mut().unwrap();
            match case {
                0 => {
                    props.remove(2);
                }
                1 => {
                    props.push(props[0].clone());
                }
                _ => {
                    props[0]["value"] = json!("../tempting-source.txt");
                }
            }
        });
        let scope = fixture.scope();
        assert!(matches!(
            prepare(
                &scope,
                Query::SearchRequirements {
                    query: "audit".into(),
                    artifact_key: None,
                    include_excerpt: false,
                    page: page(1)
                },
                &mut NoopControl
            ),
            Err(QueryError::Unavailable(_))
        ));
    }
    let mut fixture = GateFixture::new();
    fixture.change_native(|native| {
        native["catalog"]["controls"][0]["props"][2]["value"] = json!("900");
    });
    let scope = fixture.scope();
    refused(
        prepare(
            &scope,
            Query::GetRequirement {
                artifact_key: "catalog".into(),
                requirement_id: "synthetic-a".into(),
                include_excerpt: false,
            },
            &mut NoopControl,
        ),
        Reason::SourceSpanUnavailable,
    );
}

/// Status cannot expose a hidden but otherwise genuinely approved captured resource.
#[test]
fn status_visible_selection_precedes_approval_lookup() {
    let mut fixture = GateFixture::new();
    fixture.profile["visible_resource_keys"] = json!(["source"]);
    fixture.write_selected_configs();
    let scope = fixture.scope();
    assert!(scope.approved("catalog", &mut NoopControl).unwrap().is_some());
    refused(
        prepare(
            &scope,
            Query::GetArtifactStatus { artifact_key: "catalog".into() },
            &mut NoopControl,
        ),
        Reason::VisibilityRefused,
    );
    let actual = available(&scope, Query::GetArtifactStatus { artifact_key: "source".into() });
    assert_eq!(actual["data"]["native_identity"], Value::Null);
    assert_eq!(actual["data"]["lifecycle_state"], "approved");
}

/// An advisory date uses the same complete neutral approval without replacing its recorded state.
#[test]
fn explicit_schedule_keeps_neutral_approval_and_actual_record_window() {
    let mut fixture = GateFixture::new();
    fixture.profile["schedule_mode"] = json!("explicit-as-of");
    fixture.profile["as_of"] = json!("2026-12-02");
    fixture.profile["schedule_keys"] = json!(["catalog"]);
    fixture.write_selected_configs();
    let scope = fixture.scope();
    let actual = available(&scope, Query::GetArtifactStatus { artifact_key: "catalog".into() });
    assert_eq!(actual["data"]["lifecycle_state"], "approved");
    assert_eq!(actual["data"]["schedule_scope"], "explicit-as-of");
    assert_eq!(
        actual["data"]["schedule"],
        json!({"as_of":"2026-12-02","next_review_date":"2026-12-01","due_soon_days":7,"derived_status":"overdue"})
    );
}

/// Paging keeps whole counts, allows a different page size, and refuses a different query generation.
#[test]
fn cursor_binds_complete_scope_selection_and_not_transport_page_size() {
    let fixture = GateFixture::new();
    let scope = fixture.scope();
    let first = available(
        &scope,
        Query::SearchRequirements {
            query: "audit".into(),
            artifact_key: None,
            include_excerpt: false,
            page: page(1),
        },
    );
    assert_eq!(first["data"]["matched"], 2);
    assert_eq!(first["data"]["emitted"], 1);
    let cursor = first["data"]["next_cursor"].as_str().unwrap().to_owned();
    let second = available(
        &scope,
        Query::SearchRequirements {
            query: "audit".into(),
            artifact_key: None,
            include_excerpt: false,
            page: Page { cursor: Some(cursor.clone()), limit: 50 },
        },
    );
    assert_eq!(second["data"]["matched"], 2);
    assert_eq!(second["data"]["emitted"], 1);
    assert_eq!(second["data"]["next_cursor"], Value::Null);
    refused(
        prepare(
            &scope,
            Query::SearchRequirements {
                query: "monitoring".into(),
                artifact_key: None,
                include_excerpt: false,
                page: Page { cursor: Some(cursor), limit: 1 },
            },
            &mut NoopControl,
        ),
        Reason::GenerationChanged,
    );
}

/// The prepared result retains actual file generations and refuses same-length late source drift.
#[test]
fn actual_late_original_mutation_refuses_publication_fence() {
    let fixture = GateFixture::new();
    let scope = fixture.scope();
    let prepared = prepare(
        &scope,
        Query::GetRequirement {
            artifact_key: "catalog".into(),
            requirement_id: "synthetic-a".into(),
            include_excerpt: false,
        },
        &mut NoopControl,
    )
    .unwrap_or_else(|_| panic!("actual captured preparation"));
    let path = fixture.project.directory.path().join("source.txt");
    let mut bytes = fs::read(&path).unwrap();
    bytes[0] = b'Q';
    fs::write(path, bytes).unwrap();
    assert!(prepared.verify_inputs(&mut NoopControl).is_err());
}

/// Existing structural report/manifest fields and caller pins cannot manufacture domain approval.
#[test]
fn domain_jobs_are_honestly_unavailable_before_separate_bridge() {
    let mut fixture = GateFixture::new();
    let bytes =
        br#"{"schema_version":"forge.applicability-report/1","private":"sensitive rationale"}"#;
    fs::write(fixture.project.directory.path().join("report.json"), bytes).unwrap();
    fixture.project.declaration.resources.push(ResourcePolicy {
        key: "report".into(),
        role: Role::ApplicabilityReport,
        path: "report.json".into(),
        expected_sha256: sha256_hex(bytes),
        native_identity: None,
        lifecycle_key: Some("lifecycle".into()),
        citation_label: "synthetic-report".into(),
    });
    fixture.project.write_manifest();
    fixture.profile["visible_resource_keys"] = json!(["catalog", "source", "report"]);
    fixture.write_selected_configs();
    let scope = fixture.scope();
    refused(
        prepare(
            &scope,
            Query::GetRecordedApplicability {
                artifact_key: "report".into(),
                subject_id: None,
                page: page(50),
            },
            &mut NoopControl,
        ),
        Reason::ApprovalUnavailable,
    );
    refused(
        prepare(
            &scope,
            Query::GetGapSummary { artifact_key: "report".into(), page: page(50) },
            &mut NoopControl,
        ),
        Reason::ApprovalUnavailable,
    );
    let failure = serde_json::to_value(unavailable(
        &Query::GetGapSummary { artifact_key: "report".into(), page: page(50) },
        Reason::ApprovalUnavailable,
    ))
    .unwrap();
    assert_eq!(failure["data"], Value::Null);
    assert!(!serde_json::to_string(&failure).unwrap().contains("sensitive rationale"));
}

/// The shared full relationship ledger refuses before query registry growth or prefix response.
#[test]
fn complete_shared_budget_and_sticky_interruption_never_yield_partial_data() {
    let fixture = GateFixture::new();
    let scope = fixture.scope();
    let mut refused_charge = false;
    for _ in 0..=100_000 {
        if scope.charge_relationships(1).is_err() {
            refused_charge = true;
            break;
        }
    }
    assert!(refused_charge);
    assert!(matches!(
        prepare(&scope, Query::ListPolicies(page(1)), &mut NoopControl),
        Err(QueryError::Work(_))
    ));
    let scope = fixture.scope();
    let mut control = Recorder::at(Stage::PrepareDomain, 1);
    assert!(matches!(
        prepare(
            &scope,
            Query::GetRequirement {
                artifact_key: "catalog".into(),
                requirement_id: "synthetic-a".into(),
                include_excerpt: false
            },
            &mut control
        ),
        Err(QueryError::Work(WorkError::Interrupted(Interruption::CancelRequested)))
    ));
    assert_eq!(control.interruption(), Some(Interruption::CancelRequested));
}

/// Typed arguments reject unsafe metadata, invalid page sizes and unbounded token counts before capture.
#[test]
fn query_argument_admission_matches_selected_metadata_and_search_bounds() {
    for id in ["", "../private", "has space", "https://private.invalid"] {
        assert!(
            validate_query(&Query::GetRequirement {
                artifact_key: "catalog".into(),
                requirement_id: id.into(),
                include_excerpt: false
            })
            .is_err()
        );
    }
    for limit in [0, 51, usize::MAX] {
        assert!(validate_query(&Query::ListPolicies(page(limit))).is_err());
    }
    let many = (0..33).map(|index| format!("t{index}")).collect::<Vec<_>>().join(" ");
    for query in ["\nprivate".to_owned(), "!".into(), "x".repeat(4097), many] {
        assert!(tokens(&query).is_err());
    }
    assert_eq!(
        tokens("CAFÉ Café audit").unwrap_or_else(|_| panic!("bounded Unicode tokens")),
        vec!["café", "audit"]
    );
}

/// Complete row byte accounting includes escaping and refuses before retaining an overbound member.
#[test]
fn row_and_counter_bounds_preserve_exact_bytes_and_existing_rows() {
    let raw = "\"\n";
    assert_eq!(count(&raw, 6).unwrap_or_else(|_| panic!("exact escaped count")), 6);
    assert!(count(&raw, 5).is_err());
    let mut rows = vec!["old".to_owned()];
    let mut charge = MAX_RESPONSE - 1;
    assert!(retain(&mut rows, "new".to_owned(), &mut charge).is_err());
    assert_eq!(rows, vec!["old"]);
    assert_eq!(charge, MAX_RESPONSE - 1);
    let mut counter = Counter { bytes: 0, limit: 3 };
    assert_eq!(counter.write(b"abc").unwrap(), 3);
    assert!(counter.write(b"d").is_err());
    assert_eq!(counter.bytes, 3);
    let mut hash = HashWriter { hash: Sha256::new(), bytes: 0, limit: 3 };
    hash.write_all(b"abc").unwrap();
    let before = hash.hash.clone().finalize();
    assert!(hash.write(b"d").is_err());
    assert_eq!(hash.hash.clone().finalize(), before);
    assert_eq!(hash.bytes, 3);
}

/// Exact logical line and pointer rules retain UTF-8/CRLF and refuse phantom trailing lines.
#[test]
fn source_line_and_pointer_helpers_never_guess_or_clip() {
    assert_eq!(requirements::line_span("é\r\nlast\n", 1), Some((0, 2)));
    assert_eq!(requirements::line_span("é\r\nlast\n", 2), Some((4, 8)));
    assert_eq!(requirements::line_span("é\r\nlast\n", 3), None);
    assert_eq!(requirements::line_span("x", 0), None);
    let generation = "a".repeat(64);
    for ordinal in ["01", "-1", "1:2", ""] {
        assert!(
            page_start(
                &generation,
                &Page { cursor: Some(format!("{generation}:{ordinal}")), limit: 1 },
                4
            )
            .is_err()
        );
    }
    assert!(
        page_start(&generation, &Page { cursor: Some(format!("{generation}:4")), limit: 1 }, 4)
            .is_err()
    );
}

/// Install an explicitly authored synthetic source and its separate real lifecycle.
/// The existing Catalog retains its own independent approval; fixture generation
/// precedes production capture and never supplies a detached query proof.
fn install_domain_source(
    fixture: &mut GateFixture,
    key: &str,
    path: &str,
    value: &Value,
    role: Role,
) {
    let root = fixture.project.root();
    let bytes = serde_json::to_vec(value).unwrap();
    fs::write(root.join(path), &bytes).unwrap();
    let mut lifecycle = record::parse(&fs::read(root.join("lifecycle.json")).unwrap()).unwrap();
    lifecycle.policy.policy_key = format!("synthetic-{key}-policy");
    lifecycle.policy.source = ArtifactFingerprint {
        path: path.into(),
        sha256: sha256_hex(&bytes),
        oscal_type: None,
        root_uuid: None,
    };
    let fingerprints = FingerprintSet {
        source_sha256: sha256_hex(&bytes),
        generated_artifacts: vec![NamedHash {
            path: "catalog.json".into(),
            sha256: lifecycle.policy.generated_artifacts[0].sha256.clone(),
        }],
    };
    lifecycle.history.clear();
    lifecycle.state = LifecycleState::Draft;
    append_event(&mut lifecycle, LifecycleState::InReview, &fingerprints);
    append_event(&mut lifecycle, LifecycleState::Approved, &fingerprints);
    record::validate(&lifecycle).unwrap();
    let record_bytes = serde_json::to_vec(&lifecycle).unwrap();
    let record_key = format!("{key}-lifecycle");
    let record_path = format!("{key}-lifecycle.json");
    fs::write(root.join(&record_path), &record_bytes).unwrap();
    let resource = ResourcePolicy {
        key: key.into(),
        role,
        path: path.into(),
        expected_sha256: sha256_hex(&bytes),
        native_identity: None,
        lifecycle_key: Some(record_key.clone()),
        citation_label: format!("synthetic-{key}"),
    };
    let record = ResourcePolicy {
        key: record_key,
        role: Role::LifecycleRecord,
        path: record_path,
        expected_sha256: sha256_hex(&record_bytes),
        native_identity: None,
        lifecycle_key: None,
        citation_label: format!("synthetic-{key}-record"),
    };
    for resource in [resource, record] {
        if let Some(old) =
            fixture.project.declaration.resources.iter_mut().find(|row| row.key == resource.key)
        {
            *old = resource;
        } else {
            fixture.project.declaration.resources.push(resource);
        }
    }
    let visible = fixture.profile["visible_resource_keys"].as_array_mut().unwrap();
    if !visible.iter().any(|value| value.as_str() == Some(key)) {
        visible.push(json!(key));
    }
    fixture.project.write_manifest();
    fixture.write_selected_configs();
}

/// Author a real intrinsic manifest with an explicit applicable decision and omission.
fn domain_fixture() -> GateFixture {
    let mut fixture = GateFixture::new();
    let root = fixture.project.root();
    let native = fs::read(root.join("catalog.json")).unwrap();
    let mut framework = json!({"type":"catalog", "artifact":"catalog.json", "href":"catalog.json",
                               "expected_sha256":sha256_hex(&native)});
    let resource: crate::mapping::manifest::ResourceManifest =
        serde_json::from_value(framework.clone()).unwrap();
    let loaded = crate::mapping::inventory::load(&root, "$.framework", &resource).unwrap();
    framework["inventory"] = serde_json::to_value(loaded.snapshot()).unwrap();
    let manifest = json!({"schema_version":"forge.applicability/1", "framework":framework,
        "reviewers":[{"key":"reviewer","type":"person","name":"Private domain reviewer"}],
        "decisions":[{"control_id":"synthetic-a","state":"applicable","reviewer_key":"reviewer",
                      "reviewed_at":"2026-09-01T08:00:00Z","rationale":"Private applicability rationale"}],
        "mapping_collections":[]});
    crate::applicability::manifest::parse(&serde_json::to_vec(&manifest).unwrap()).unwrap();
    install_domain_source(
        &mut fixture,
        "domain",
        "applicability.json",
        &manifest,
        Role::ApplicabilityManifest,
    );
    fixture
}

/// Generate an actual maintained report before the query capture, with no supplied proof.
/// File-oriented preparation here is only fixture generation; production queries
/// consume the separate captured-byte helper and never call this legacy reader.
fn report_fixture(
    fixture: &mut GateFixture,
    filters: crate::applicability::model::ReportFilters,
) -> Value {
    let prepared = crate::applicability::prepare_analysis(
        &fixture.project.root().join("applicability.json"),
        filters,
    )
    .unwrap();
    let report = serde_json::to_value(&prepared.report).unwrap();
    install_domain_source(fixture, "report", "report.json", &report, Role::ApplicabilityReport);
    report
}

/// Actual recorded decisions and omissions use maintained classifications and exact spans.
#[test]
fn actual_domain_manifest_projects_current_full_rows_without_private_prose() {
    let fixture = domain_fixture();
    let scope = fixture.scope();
    let actual = available(
        &scope,
        Query::GetRecordedApplicability {
            artifact_key: "domain".into(),
            subject_id: None,
            page: page(50),
        },
    );
    assert_eq!(actual["data"]["matched"], 2);
    assert_eq!(actual["data"]["emitted"], 2);
    assert_eq!(actual["data"]["rows"][0]["control_id"], "synthetic-a");
    assert_eq!(actual["data"]["rows"][0]["decision_state"], "applicable");
    assert_eq!(actual["data"]["rows"][0]["classification"], "applicable-unmapped");
    assert_eq!(actual["data"]["rows"][1]["decision_state"], Value::Null);
    assert_eq!(actual["data"]["rows"][1]["decision_source"], "implicit-under-review");
    assert_eq!(actual["data"]["rows"][1]["classification"], "under-review");
    assert_eq!(actual["data"]["rows"][0]["citations"][0]["native_pointer"], "/catalog/controls/0");
    assert_eq!(
        actual["data"]["rows"][0]["citations"][0]["source_span"],
        json!({"start_byte":0,"end_byte":"Private sentinel café access audit.".len()})
    );
    let text = serde_json::to_string(&actual).unwrap();
    for private in [
        "Private applicability rationale",
        "Private domain reviewer",
        "Sensitive control title",
        "source.txt",
    ] {
        assert!(!text.contains(private));
    }
    let gaps =
        available(&scope, Query::GetGapSummary { artifact_key: "domain".into(), page: page(1) });
    assert_eq!(gaps["data"]["summary"]["total"], 2);
    assert_eq!(gaps["data"]["summary"]["applicable_unmapped"], 1);
    assert_eq!(gaps["data"]["summary"]["under_review"], 1);
    assert_eq!(gaps["data"]["matched"], 2);
    assert_eq!(gaps["data"]["emitted"], 1);
}

/// Subject selection and page traversal cannot turn the full summary into a filtered denominator.
#[test]
fn domain_pages_bind_selection_and_preserve_full_classification_counts() {
    let fixture = domain_fixture();
    let scope = fixture.scope();
    let filtered = available(
        &scope,
        Query::GetRecordedApplicability {
            artifact_key: "domain".into(),
            subject_id: Some("synthetic-b".into()),
            page: page(1),
        },
    );
    assert_eq!(filtered["data"]["matched"], 1);
    assert_eq!(filtered["data"]["rows"][0]["control_id"], "synthetic-b");
    let first =
        available(&scope, Query::GetGapSummary { artifact_key: "domain".into(), page: page(1) });
    let cursor = first["data"]["next_cursor"].as_str().unwrap().to_owned();
    let second = available(
        &scope,
        Query::GetGapSummary {
            artifact_key: "domain".into(),
            page: Page { cursor: Some(cursor.clone()), limit: 50 },
        },
    );
    assert_eq!(first["data"]["summary"], second["data"]["summary"]);
    assert_eq!(second["data"]["matched"], 2);
    assert_eq!(second["data"]["emitted"], 1);
    assert_eq!(second["data"]["next_cursor"], Value::Null);
    refused(
        prepare(
            &scope,
            Query::GetRecordedApplicability {
                artifact_key: "domain".into(),
                subject_id: None,
                page: Page { cursor: Some(cursor), limit: 1 },
            },
            &mut NoopControl,
        ),
        Reason::GenerationChanged,
    );
}

/// A source-approved report is current only after actual complete empty-filter equality.
#[test]
fn actual_report_requires_full_current_equality_and_rejects_filtered_projection() {
    let mut fixture = domain_fixture();
    let report =
        report_fixture(&mut fixture, crate::applicability::model::ReportFilters::default());
    let scope = fixture.scope();
    let actual =
        available(&scope, Query::GetGapSummary { artifact_key: "report".into(), page: page(50) });
    assert_eq!(actual["data"]["summary"], report["counts"]);
    drop(scope);
    let filtered = report_fixture(
        &mut fixture,
        crate::applicability::model::ReportFilters {
            state: Some(crate::applicability::model::GapClassification::UnderReview),
            ..Default::default()
        },
    );
    assert_eq!(filtered["matched_controls"], 1);
    let scope = fixture.scope();
    refused(
        prepare(
            &scope,
            Query::GetGapSummary { artifact_key: "report".into(), page: page(50) },
            &mut NoopControl,
        ),
        Reason::IncompleteClosure,
    );
}

/// Equal hash candidates all undergo genuine approval/current closure; two full matches refuse.
#[test]
fn report_hash_is_only_a_candidate_filter_and_complete_ambiguity_refuses() {
    let mut fixture = domain_fixture();
    report_fixture(&mut fixture, crate::applicability::model::ReportFilters::default());
    let manifest: Value = serde_json::from_slice(
        &fs::read(fixture.project.root().join("applicability.json")).unwrap(),
    )
    .unwrap();
    install_domain_source(
        &mut fixture,
        "domain-two",
        "other-applicability.json",
        &manifest,
        Role::ApplicabilityManifest,
    );
    let scope = fixture.scope();
    assert_eq!(
        scope.resource("domain").unwrap().raw_sha256(),
        scope.resource("domain-two").unwrap().raw_sha256()
    );
    assert!(scope.domain_source_candidate("domain", &mut NoopControl).unwrap().is_some());
    assert!(scope.domain_source_candidate("domain-two", &mut NoopControl).unwrap().is_some());
    refused(
        prepare(
            &scope,
            Query::GetGapSummary { artifact_key: "report".into(), page: page(50) },
            &mut NoopControl,
        ),
        Reason::IncompleteClosure,
    );
}

/// A report approval never approves its manifest or independently visible native dependency.
#[test]
fn domain_source_approval_does_not_replace_independent_native_visibility_or_approval() {
    let mut fixture = domain_fixture();
    let root = fixture.project.root();
    let mut lifecycle = record::parse(&fs::read(root.join("lifecycle.json")).unwrap()).unwrap();
    lifecycle.history.clear();
    lifecycle.state = LifecycleState::Draft;
    let bytes = serde_json::to_vec(&lifecycle).unwrap();
    fs::write(root.join("lifecycle.json"), &bytes).unwrap();
    fixture
        .project
        .declaration
        .resources
        .iter_mut()
        .find(|row| row.key == "lifecycle")
        .unwrap()
        .expected_sha256 = sha256_hex(&bytes);
    fixture.project.write_manifest();
    fixture.write_selected_configs();
    let scope = fixture.scope();
    assert!(scope.domain_source_candidate("domain", &mut NoopControl).unwrap().is_some());
    refused(
        prepare(
            &scope,
            Query::GetRecordedApplicability {
                artifact_key: "domain".into(),
                subject_id: None,
                page: page(50),
            },
            &mut NoopControl,
        ),
        Reason::Unapproved,
    );
    drop(scope);
    fixture.profile["visible_resource_keys"] = json!(["domain"]);
    fixture.write_selected_configs();
    let scope = fixture.scope();
    refused(
        prepare(
            &scope,
            Query::GetGapSummary { artifact_key: "domain".into(), page: page(50) },
            &mut NoopControl,
        ),
        Reason::IncompleteClosure,
    );
}

/// Correct pins/record approval do not hide a missing explicitly declared native closure path.
#[test]
fn admitted_domain_raw_path_cannot_read_or_substitute_an_undeclared_dependency() {
    let mut fixture = domain_fixture();
    let mut manifest: Value = serde_json::from_slice(
        &fs::read(fixture.project.root().join("applicability.json")).unwrap(),
    )
    .unwrap();
    manifest["framework"]["artifact"] = json!("outside.json");
    install_domain_source(
        &mut fixture,
        "domain",
        "applicability.json",
        &manifest,
        Role::ApplicabilityManifest,
    );
    let scope = fixture.scope();
    refused(
        prepare(
            &scope,
            Query::GetGapSummary { artifact_key: "domain".into(), page: page(50) },
            &mut NoopControl,
        ),
        Reason::IncompleteClosure,
    );
    assert!(!fixture.project.root().join("outside.json").exists());
}

/// Real domain preparation retains original bytes and rejects same-size drift before publication.
#[test]
fn actual_domain_original_mutation_blocks_completed_publication() {
    let fixture = domain_fixture();
    let scope = fixture.scope();
    let prepared = prepare(
        &scope,
        Query::GetGapSummary { artifact_key: "domain".into(), page: page(50) },
        &mut NoopControl,
    )
    .unwrap_or_else(|_| panic!("actual domain source closure"));
    let path = fixture.project.root().join("applicability.json");
    let mut bytes = fs::read(&path).unwrap();
    let index = bytes.iter().position(|byte| *byte == b'P').unwrap();
    bytes[index] = b'Q';
    fs::write(path, bytes).unwrap();
    assert!(prepared.verify_inputs(&mut NoopControl).is_err());
}

/// Shared ledger refusal and sticky interruption cannot be downgraded by candidate scanning.
#[test]
fn domain_callback_failures_and_typed_stop_never_become_successful_candidate_data() {
    let fixture = domain_fixture();
    let scope = fixture.scope();
    for _ in 0..=100_000 {
        if scope.charge_relationships(1).is_err() {
            break;
        }
    }
    assert!(matches!(
        recorded::run_domain(&scope, |charge| {
            let _ = charge(1);
            Ok(())
        }),
        Err(QueryError::Work(_))
    ));
    let scope = fixture.scope();
    let mut stop = Recorder::at(Stage::PrepareDomain, 1);
    assert!(matches!(
        prepare(
            &scope,
            Query::GetGapSummary { artifact_key: "domain".into(), page: page(1) },
            &mut stop
        ),
        Err(QueryError::Work(WorkError::Interrupted(Interruption::CancelRequested)))
    ));
}

/// Build a genuine self-mapping through the maintained native builder before capture.
/// The many-to-many source/target pairs and fingerprints are actual native facts;
/// all authoring and lifecycle records here are explicitly synthetic fixture data.
#[allow(clippy::too_many_lines)] // Build the complete real native Mapping and its independent lifecycle tuple together.
fn add_mapping_fixture(fixture: &mut GateFixture) -> String {
    let root = fixture.project.root();
    let manifest = json!({"schema_version":"forge.mapping-manifest/1",
        "collection":{"key":"synthetic-collection","title":"Private mapping title","version":"1",
                      "last_modified":"2026-09-01T08:00:00Z"},
        "reviewers":[{"key":"mapper","type":"person","name":"Private mapping reviewer"}],
        "provenance":{"method":"human","matching_rationale":"semantic","status":"complete",
            "mapping_description":"Private mapping description","reviewer_keys":["mapper"],
            "reviewed_at":"2026-09-01T08:00:00Z"},
        "mapping":{"key":"synthetic-map","scope":"control-only",
            "source":{"type":"catalog","artifact":"catalog.json","href":"catalog.json"},
            "target":{"type":"catalog","artifact":"catalog.json","href":"catalog.json"},
            "maps":[{"key":"many-to-many","relationship":"intersects-with",
                "sources":[{"type":"control","id_ref":"synthetic-a"},{"type":"control","id_ref":"synthetic-b"}],
                "targets":[{"type":"control","id_ref":"synthetic-a"},{"type":"control","id_ref":"synthetic-b"}],
                "reviewer_key":"mapper","reviewed_at":"2026-09-01T08:00:00Z","rationale":"Private mapping rationale"}]}});
    let bytes = serde_json::to_vec(&manifest).unwrap();
    fs::write(root.join("mapping-authoring.json"), &bytes).unwrap();
    let built = crate::mapping::prepare(&root.join("mapping-authoring.json"), None, false).unwrap();
    let mut native: Value = serde_json::from_str(&built.artifact_json).unwrap();
    let map_id = native["mapping-collection"]["mappings"][0]["maps"][0]["uuid"]
        .as_str()
        .unwrap()
        .to_ascii_uppercase();
    native["mapping-collection"]["mappings"][0]["maps"][0]["uuid"] = json!(map_id);
    let native_bytes = serde_json::to_vec(&native).unwrap();
    fs::write(root.join("mapping.json"), &native_bytes).unwrap();
    let identity = NativeIdentity {
        model: "mapping-collection".into(),
        root_id: native["mapping-collection"]["uuid"].as_str().unwrap().into(),
        document_version: native["mapping-collection"]["metadata"]["version"]
            .as_str()
            .unwrap()
            .into(),
        oscal_version: native["mapping-collection"]["metadata"]["oscal-version"]
            .as_str()
            .unwrap()
            .into(),
    };
    let mut lifecycle = record::parse(&fs::read(root.join("lifecycle.json")).unwrap()).unwrap();
    lifecycle.policy.policy_key = "synthetic-mapping-policy".into();
    lifecycle.policy.source = ArtifactFingerprint {
        path: "mapping-authoring.json".into(),
        sha256: sha256_hex(&bytes),
        oscal_type: None,
        root_uuid: None,
    };
    lifecycle.policy.generated_artifacts = vec![ArtifactFingerprint {
        path: "mapping.json".into(),
        sha256: sha256_hex(&native_bytes),
        oscal_type: Some(identity.model.clone()),
        root_uuid: Some(identity.root_id.clone()),
    }];
    let fingerprints = FingerprintSet {
        source_sha256: sha256_hex(&bytes),
        generated_artifacts: vec![NamedHash {
            path: "mapping.json".into(),
            sha256: sha256_hex(&native_bytes),
        }],
    };
    lifecycle.history.clear();
    lifecycle.state = LifecycleState::Draft;
    append_event(&mut lifecycle, LifecycleState::InReview, &fingerprints);
    append_event(&mut lifecycle, LifecycleState::Approved, &fingerprints);
    record::validate(&lifecycle).unwrap();
    let record_bytes = serde_json::to_vec(&lifecycle).unwrap();
    fs::write(root.join("mapping-lifecycle.json"), &record_bytes).unwrap();
    fixture.project.declaration.resources.extend([
        ResourcePolicy {
            key: "mapping".into(),
            role: Role::MappingCollection,
            path: "mapping.json".into(),
            expected_sha256: sha256_hex(&native_bytes),
            native_identity: Some(identity),
            lifecycle_key: Some("mapping-lifecycle".into()),
            citation_label: "synthetic-mapping".into(),
        },
        ResourcePolicy {
            key: "mapping-source".into(),
            role: Role::LifecycleSource,
            path: "mapping-authoring.json".into(),
            expected_sha256: sha256_hex(&bytes),
            native_identity: None,
            lifecycle_key: Some("mapping-lifecycle".into()),
            citation_label: "synthetic-mapping-source".into(),
        },
        ResourcePolicy {
            key: "mapping-lifecycle".into(),
            role: Role::LifecycleRecord,
            path: "mapping-lifecycle.json".into(),
            expected_sha256: sha256_hex(&record_bytes),
            native_identity: None,
            lifecycle_key: None,
            citation_label: "synthetic-mapping-record".into(),
        },
    ]);
    let mut domain: Value =
        serde_json::from_slice(&fs::read(root.join("applicability.json")).unwrap()).unwrap();
    domain["mapping_collections"] = json!(["mapping.json"]);
    install_domain_source(
        fixture,
        "domain",
        "applicability.json",
        &domain,
        Role::ApplicabilityManifest,
    );
    fixture.profile["visible_resource_keys"].as_array_mut().unwrap().push(json!("mapping"));
    fixture.project.write_manifest();
    fixture.write_selected_configs();
    map_id
}

/// Actual many-to-many native pairs are complete, source-grounded and retain original UUID spelling.
#[test]
fn actual_mapping_trace_keeps_complete_pairs_and_exact_native_uuid_spelling() {
    let mut fixture = domain_fixture();
    let map_id = add_mapping_fixture(&mut fixture);
    let scope = fixture.scope();
    let actual = available(
        &scope,
        Query::TraceControl {
            artifact_key: "catalog".into(),
            control_id: "synthetic-a".into(),
            page: page(50),
        },
    );
    assert_eq!(actual["data"]["matched"], 4);
    assert_eq!(actual["data"]["emitted"], 4);
    let rows = actual["data"]["rows"].as_array().unwrap();
    assert_eq!(rows[0]["mapping_id"], Value::Null);
    for row in &rows[1..] {
        assert_eq!(row["mapping_id"], map_id);
        assert_eq!(row["relation"], "recorded-mapping");
        let citations = row["citations"].as_array().unwrap();
        assert_eq!(citations.len(), 3);
        assert_eq!(citations[0]["source_state"], "exact-captured-source");
        assert_eq!(citations[1]["source_state"], "exact-captured-source");
        assert_eq!(citations[2]["source_state"], "native-only");
        assert_eq!(citations[2]["native_pointer"], "/mapping-collection/mappings/0/maps/0");
        assert_eq!(citations[2]["source_span"], Value::Null);
    }
    let text = serde_json::to_string(&actual).unwrap();
    for private in [
        "Private mapping title",
        "Private mapping reviewer",
        "Private mapping rationale",
        "mapping.json",
    ] {
        assert!(!text.contains(private));
    }
    let current =
        available(&scope, Query::GetGapSummary { artifact_key: "domain".into(), page: page(50) });
    assert_eq!(current["data"]["summary"]["applicable_mapped"], 1);
}

/// A native mapping omitted from all approved manifest closures cannot be silently hidden by a page.
#[test]
fn actual_visible_mapping_missing_declared_closure_refuses_whole_trace() {
    let mut fixture = domain_fixture();
    add_mapping_fixture(&mut fixture);
    let mut domain: Value = serde_json::from_slice(
        &fs::read(fixture.project.root().join("applicability.json")).unwrap(),
    )
    .unwrap();
    domain["mapping_collections"] = json!([]);
    install_domain_source(
        &mut fixture,
        "domain",
        "applicability.json",
        &domain,
        Role::ApplicabilityManifest,
    );
    let scope = fixture.scope();
    assert!(scope.approved("mapping", &mut NoopControl).unwrap().is_some());
    refused(
        prepare(
            &scope,
            Query::TraceControl {
                artifact_key: "catalog".into(),
                control_id: "synthetic-a".into(),
                page: page(1),
            },
            &mut NoopControl,
        ),
        Reason::IncompleteClosure,
    );
}

/// Test-only owned originals reuse the maintained gate/domain/native fixture constructors.
/// Every declared owner and lifecycle assertion remains synthetic fixture data.
pub(in crate::mcp::artifact_status) struct StdioFixture {
    /// Genuine existing file-backed captured-query fixture, never a detached proof.
    inner: GateFixture,
    /// Original uppercase UUID spelling of the generated actual Mapping map.
    mapping_id: String,
    /// Complete maintained full empty-filter report created before any capture.
    report: Value,
}

impl StdioFixture {
    /// Author native Catalog/Mapping, applicability and current report originals before capture.
    pub(in crate::mcp::artifact_status) fn new() -> Self {
        let mut inner = domain_fixture();
        let mapping_id = add_mapping_fixture(&mut inner);
        let report =
            report_fixture(&mut inner, crate::applicability::model::ReportFilters::default());
        Self { inner, mapping_id, report }
    }

    /// Return ordinary actual root/pin selections for the real two-root production loader.
    pub(in crate::mcp::artifact_status) fn startup(&self) -> super::super::stdio::Startup {
        super::super::stdio::Startup {
            project_root: self.inner.project.root(),
            decision_root: Some(self.inner.decision_root()),
            decision_sha256: Some(self.inner.decision_pin.clone()),
            profile_sha256: self.inner.profile_pin.clone(),
        }
    }

    /// Model a complete response through the actual loader, query and final original fence.
    /// This expected value is not inserted into the worker or supplied as authority.
    pub(in crate::mcp::artifact_status) fn expected(&self, query: Query) -> Value {
        let scope = self.inner.scope();
        let prepared = prepare(&scope, query, &mut NoopControl)
            .unwrap_or_else(|_| panic!("genuine fixture query preparation"));
        let value = serde_json::to_value(prepared.response()).unwrap();
        assert_eq!(value["availability"], "available");
        prepared.verify_inputs(&mut NoopControl).unwrap();
        value
    }

    /// Return only the actual fixed source fixture path for ordinary late-drift controls.
    pub(in crate::mcp::artifact_status) fn source_path(&self) -> std::path::PathBuf {
        self.inner.project.root().join("source.txt")
    }

    /// Return only the actual fixed approved applicability path for late-closure drift.
    pub(in crate::mcp::artifact_status) fn domain_path(&self) -> std::path::PathBuf {
        self.inner.project.root().join("applicability.json")
    }

    /// Borrow the actual generated native UUID spelling for an independent trace assertion.
    pub(in crate::mcp::artifact_status) fn mapping_id(&self) -> &str {
        &self.mapping_id
    }

    /// Borrow the complete maintained report counts, independent of paging in wire output.
    pub(in crate::mcp::artifact_status) fn report_counts(&self) -> &Value {
        &self.report["counts"]
    }
}
