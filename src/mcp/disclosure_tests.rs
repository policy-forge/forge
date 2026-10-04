//! File-backed synthetic gate controls; no real owner provenance or client acceptance.
//! The native/lifecycle fixture helpers are literal copies from the frozen neutral core,
//! except unused core-only methods are omitted. Synthetic declarations grant no human approval.

use super::super::{MANIFEST_PATH, MANIFEST_PROFILE};
use super::*;
use crate::hashing::sha256_hex;
use crate::lifecycle::record::{
    APPROVAL_POLICY_VERSION, ApprovalPolicy, ArtifactFingerprint, DeclaredRole, FingerprintSet,
    LifecycleState, NamedHash, Party, PolicyIdentity, ReviewSchedule, RoleRequirement,
    SCHEMA_VERSION, SeparationRules, TimezonePolicy, TransitionEvent,
};
use crate::workspace::preparation::{Interruption, NoopControl};
use serde_json::json;
use std::fs;
use tempfile::TempDir;

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
        let source = b"Synthetic private policy prose; not an instruction channel.\n";
        let catalog = json!({"catalog":{
            "uuid":"11111111-1111-4111-8111-111111111111",
            "metadata":{"title":"Synthetic private instruction title", "last-modified":"2026-09-01T00:00:00Z", "version":"1", "oscal-version":"1.2.3"},
            "controls":[{"id":"synthetic-a", "title":"Synthetic control"}]
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
            "discovery_sha256":discovery_pin, "enabled_tools":["get_artifact_status"],
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

/// Missing/half decision pairs never touch nonexistent or malformed original roots/files.
#[test]
fn no_pair_is_non_disclosing_without_root_io() {
    for (root, pin) in
        [(None, None), (Some(Path::new("missing/../unsafe")), None), (None, Some("not-a-pin"))]
    {
        let gate = load_disclosure_decision(
            Path::new("missing/../unsafe"),
            root,
            pin,
            "not-a-pin",
            &mut NoopControl,
        )
        .unwrap();
        assert!(matches!(gate, DisclosureGate::Unavailable));
    }
}

/// Complete synthetic handoff still requires genuine native and lifecycle proof for content.
#[test]
fn complete_gate_borrows_actual_native_and_current_lifecycle() {
    let fixture = GateFixture::new();
    let scope = fixture.scope();
    let native = scope.approved("catalog", &mut NoopControl).unwrap().unwrap();
    assert_eq!(
        native.resource().native_value().unwrap()["catalog"]["controls"][0]["id"],
        "synthetic-a"
    );
    assert_eq!(native.neutral_status().derived_status, "approved");
    assert!(native.evaluated_status().is_none());
    assert_eq!(scope.visible_keys().collect::<Vec<_>>(), vec!["catalog", "source"]);
    assert!(!scope.visible_keys().any(|key| key == "lifecycle"));
    assert!(scope.resource("lifecycle").is_some());
    assert!(scope.tool_enabled("get_artifact_status"));
    assert!(!scope.tool_enabled("search_requirements"));
    assert!(!scope.excerpt_allowed("source"));
    scope.verify_inputs(&mut NoopControl).unwrap();
}

/// Only the exact declared source base/current tuple yields sealed bytes, never unknown href IO.
#[test]
fn declared_native_source_uses_actual_original_tuple() {
    let fixture = GateFixture::new();
    let scope = fixture.scope();
    let source = scope
        .declared_source_for_native("catalog", "source.txt", &mut NoopControl)
        .unwrap()
        .unwrap();
    assert_eq!(source.key(), "source");
    assert!(source.pin_matches());
    assert_eq!(
        source.bytes(),
        fs::read(fixture.project.directory.path().join("source.txt")).unwrap()
    );
    for bad in
        ["../source.txt", "./source.txt", "source.txt/", "https://private.invalid", "unknown.txt"]
    {
        assert!(
            scope.declared_source_for_native("catalog", bad, &mut NoopControl).unwrap().is_none()
        );
    }
}

/// Trusted date/excerpt modes are profile-bound and schedule uses the same neutral current tuple.
#[test]
fn schedule_and_excerpt_require_paired_original_profile() {
    let mut fixture = GateFixture::new();
    fixture.profile["source_text_mode"] = json!("exact-span-opt-in");
    fixture.profile["source_text_keys"] = json!(["source"]);
    fixture.profile["schedule_mode"] = json!("explicit-as-of");
    fixture.profile["schedule_keys"] = json!(["catalog"]);
    fixture.profile["as_of"] = json!("2026-11-27");
    fixture.write_selected_configs();
    let scope = fixture.scope();
    let native = scope.approved("catalog", &mut NoopControl).unwrap().unwrap();
    assert!(scope.excerpt_allowed("source"));
    assert!(!scope.excerpt_allowed("catalog"));
    assert_eq!(native.neutral_status().derived_status, "approved");
    assert_eq!(native.evaluated_status().unwrap().derived_status, "due-soon");
    assert_eq!(
        native.evaluated_status().unwrap().current_fingerprints,
        native.current().fingerprints
    );
    assert_eq!(native.record().review.due_soon_days, 7);
    fixture.profile["as_of"] = json!("2026-02-30");
    fixture.write_selected_configs();
    assert!(matches!(fixture.load(), DisclosureGate::Unavailable));
}

/// Refused, missing, duplicate or misbound declared subject/role provenance never grants scope.
#[test]
fn all_six_scoped_subject_role_rows_are_required() {
    for case in 0..5 {
        let mut fixture = GateFixture::new();
        match case {
            0 => fixture.decision["policy_decisions"][0]["disposition"] = json!("refused"),
            1 => fixture.decision["policy_decisions"][0]["declared_owner_key"] = Value::Null,
            2 => {
                fixture.decision["policy_decisions"][0] =
                    fixture.decision["policy_decisions"][1].clone();
            }
            3 => fixture.decision["policy_decisions"][0]["profile_sha256"] = json!("c".repeat(64)),
            _ => fixture.decision["policy_decisions"][0]["recorded_on"] = json!("2026-02-30"),
        }
        fixture.write_decision_only();
        assert!(matches!(fixture.load(), DisclosureGate::Unavailable));
    }
}

/// Complete record/profile equality includes array order, nulls and every closed field.
#[test]
fn full_profile_equality_and_unknown_record_fields_are_not_normalized() {
    let mut fixture = GateFixture::new();
    fixture.decision["visibility_profile"]["visible_resource_keys"] = json!(["source", "catalog"]);
    fixture.write_decision_only();
    assert!(matches!(fixture.load(), DisclosureGate::Unavailable));
    fixture.write_selected_configs();
    fixture.decision["owner_override"] = json!(true);
    fixture.write_decision_only();
    assert!(matches!(fixture.load(), DisclosureGate::Unavailable));
}

/// Original raw pins, duplicate keys and byte ceilings precede closed typed config growth.
#[test]
fn raw_decision_pin_and_duplicate_keys_refuse() {
    let fixture = GateFixture::new();
    let path = fixture.decision_directory.path().join(CONFIG_PATHS[2]);
    let bytes = fs::read(&path).unwrap();
    let mut changed = bytes.clone();
    changed.push(b' ');
    fs::write(&path, &changed).unwrap();
    assert!(matches!(fixture.load(), DisclosureGate::Unavailable));
    let duplicate =
        b"{\"schema_version\":\"forge.mcp-disclosure-decision/1\",\"schema_version\":\"x\"}";
    fs::write(&path, duplicate).unwrap();
    assert!(matches!(
        load_disclosure_decision(
            &fixture.project.root(),
            Some(&fixture.decision_root()),
            Some(&sha256_hex(duplicate)),
            &fixture.profile_pin,
            &mut NoopControl
        )
        .unwrap(),
        DisclosureGate::Unavailable
    ));
    fs::write(&path, vec![b' '; CONFIG_LIMIT + 1]).unwrap();
    assert!(matches!(fixture.load(), DisclosureGate::Unavailable));
}

/// Equal/nested actual roots and unsafe original spellings refuse before selected data parsing.
#[test]
fn root_disjointness_and_raw_spelling_are_required() {
    let fixture = GateFixture::new();
    assert!(matches!(
        load_disclosure_decision(
            &fixture.project.root(),
            Some(&fixture.project.root()),
            Some(&fixture.decision_pin),
            &fixture.profile_pin,
            &mut NoopControl
        )
        .unwrap(),
        DisclosureGate::Unavailable
    ));
    let nested = fixture.project.root().join("control");
    fs::create_dir(&nested).unwrap();
    assert!(matches!(
        load_disclosure_decision(
            &fixture.project.root(),
            Some(&nested),
            Some(&fixture.decision_pin),
            &fixture.profile_pin,
            &mut NoopControl
        )
        .unwrap(),
        DisclosureGate::Unavailable
    ));
    let unsafe_root = fixture.project.root().join(".");
    assert!(matches!(
        load_disclosure_decision(
            &unsafe_root,
            Some(&fixture.decision_root()),
            Some(&fixture.decision_pin),
            &fixture.profile_pin,
            &mut NoopControl
        )
        .unwrap(),
        DisclosureGate::Unavailable
    ));
}

/// Three configs leave 998 declared original slots; typed missing entries count before reads.
#[test]
fn complete_resource_union_has_no_second_original_pool() {
    let mut fixture = GateFixture::new();
    fixture.profile["visible_resource_keys"] = json!([]);
    fixture.project.declaration.resources = (0..998)
        .map(|index| ResourcePolicy {
            key: format!("key-{index}"),
            role: Role::PolicySource,
            path: format!("absent-{index}.txt"),
            expected_sha256: "a".repeat(64),
            native_identity: None,
            lifecycle_key: None,
            citation_label: format!("citation-{index}"),
        })
        .collect();
    fixture.project.write_manifest();
    fixture.write_selected_configs();
    let scope = fixture.scope();
    assert_eq!(scope.project.objects.len() + 3, ORIGINAL_LIMIT);
    assert!(scope.resources().next().is_none());
    scope.verify_inputs(&mut NoopControl).unwrap();
    fixture.project.declaration.resources.push(ResourcePolicy {
        key: "overflow".into(),
        role: Role::PolicySource,
        path: "overflow.txt".into(),
        expected_sha256: "a".repeat(64),
        native_identity: None,
        lifecycle_key: None,
        citation_label: "overflow-citation".into(),
    });
    fixture.project.write_manifest();
    fixture.write_selected_configs();
    assert!(matches!(fixture.load(), DisclosureGate::Unavailable));
}

/// Complete alias admission compares config/resource originals before any resource read.
#[test]
#[cfg(unix)]
fn actual_hardlink_alias_is_not_permission() {
    let mut fixture = GateFixture::new();
    fs::remove_file(fixture.project.directory.path().join("source.txt")).unwrap();
    fs::hard_link(
        fixture.project.directory.path().join(CONFIG_PATHS[1]),
        fixture.project.directory.path().join("source.txt"),
    )
    .unwrap();
    fixture.project.declaration.resources[1].expected_sha256 = fixture.profile_pin.clone();
    fixture.project.write_manifest();
    fixture.write_selected_configs();
    assert!(matches!(fixture.load(), DisclosureGate::Unavailable));
}

/// A typed captured absence retains its actual ancestor fence and cannot become a zero-byte source.
#[test]
fn original_absence_and_decision_generations_are_rechecked() {
    let mut fixture = GateFixture::new();
    fixture.project.declaration.resources.push(ResourcePolicy {
        key: "missing".into(),
        role: Role::PolicySource,
        path: "missing.txt".into(),
        expected_sha256: "a".repeat(64),
        native_identity: None,
        lifecycle_key: None,
        citation_label: "missing-citation".into(),
    });
    fixture.project.write_manifest();
    fixture.write_selected_configs();
    let scope = fixture.scope();
    fs::write(fixture.project.directory.path().join("missing.txt"), b"appeared").unwrap();
    assert!(scope.verify_inputs(&mut NoopControl).is_err());
    fs::remove_file(fixture.project.directory.path().join("missing.txt")).unwrap();
    fs::write(fixture.decision_directory.path().join(CONFIG_PATHS[2]), b"changed").unwrap();
    assert!(scope.verify_inputs(&mut NoopControl).is_err());
}

/// The monotonic relationship limit checks overflow before counter mutation or derived insertion.
#[test]
fn relation_admission_is_complete_and_never_reset() {
    let fixture = GateFixture::new();
    let scope = fixture.scope();
    let remaining = RELATION_LIMIT - scope.relationships.get();
    scope.charge_relationships(remaining).unwrap();
    assert_eq!(scope.relationships.get(), RELATION_LIMIT);
    assert!(scope.charge_relationships(1).is_err());
    assert!(scope.charge_relationships(usize::MAX).is_err());
    assert_eq!(scope.relationships.get(), RELATION_LIMIT);
}

/// Cursor generation changes after an explicitly selected config generation; stale originals fail.
#[test]
fn config_generation_is_part_of_scope_generation() {
    let mut fixture = GateFixture::new();
    let scope = fixture.scope();
    let generation = scope.scope_generation().to_owned();
    fixture.profile["enabled_tools"] = json!(["get_artifact_status", "list_policies"]);
    fixture.write_selected_configs();
    assert!(scope.verify_inputs(&mut NoopControl).is_err());
    let next = fixture.scope();
    assert_ne!(next.scope_generation(), generation);
}

/// Sticky interruption bypasses unavailable classification even with no decision pair.
#[test]
fn interruption_is_not_owner_unavailability() {
    /// Actual synthetic control returns one fixed typed cancellation without IO or error text.
    struct Cancel;
    impl WorkControl for Cancel {
        /// Preserve a typed interruption at every cooperative producer fence.
        fn checkpoint(
            &mut self,
            _stage: Stage,
            _progress: crate::workspace::preparation::ProgressUpdate,
        ) -> WorkResult<()> {
            Err(WorkError::Interrupted(Interruption::CancelRequested))
        }
        /// Return the same sticky reason when no original IO is required.
        fn interruption(&self) -> Option<Interruption> {
            Some(Interruption::CancelRequested)
        }
    }
    let result = load_disclosure_decision(Path::new("invalid"), None, None, "invalid", &mut Cancel);
    assert!(matches!(result, Err(WorkError::Interrupted(Interruption::CancelRequested))));
}

/// Complete observed raw sizes are refused before any staged original content read.
#[test]
fn pre_read_aggregate_size_has_one_pool() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().canonicalize().unwrap();
    let root = Rc::new(fresh::qualify_root(&path).unwrap());
    let mut ledger = InputLedger::new();
    let mut originals = Vec::new();
    for index in 0..6 {
        let name = format!("part-{index}.bin");
        let file = fs::File::create(path.join(&name)).unwrap();
        file.set_len(8 * 1024 * 1024).unwrap();
        drop(file);
        let held = fresh::prepare_local(Rc::clone(&root), Path::new(&name), false).unwrap();
        ledger.admit(&held, super::super::MAX_RESOURCE_BYTES).unwrap();
        originals.push(held);
    }
    let file = fs::File::create(path.join("overflow.bin")).unwrap();
    file.set_len(8 * 1024 * 1024).unwrap();
    drop(file);
    let overflow = fresh::prepare_local(root, Path::new("overflow.bin"), false).unwrap();
    assert!(ledger.admit(&overflow, super::super::MAX_RESOURCE_BYTES).is_err());
    assert_eq!(ledger.count, 6);
    assert_eq!(ledger.bytes, 48 * 1024 * 1024);
    assert_eq!(originals.len(), 6);
}

/// Different valid native/record bases join to one exact actual captured source without href IO.
#[test]
fn native_and_record_relative_bases_preserve_one_source_identity() {
    let mut fixture = GateFixture::new();
    let root = fixture.project.root();
    fs::create_dir(root.join("assets")).unwrap();
    fs::rename(root.join("source.txt"), root.join("assets/source.txt")).unwrap();
    fs::rename(root.join("catalog.json"), root.join("assets/catalog.json")).unwrap();
    let mut lifecycle = record::parse(&fs::read(root.join("lifecycle.json")).unwrap()).unwrap();
    lifecycle.policy.source.path = "assets/source.txt".into();
    lifecycle.policy.generated_artifacts[0].path = "assets/catalog.json".into();
    lifecycle.history.clear();
    lifecycle.state = LifecycleState::Draft;
    let fingerprints = FingerprintSet {
        source_sha256: lifecycle.policy.source.sha256.clone(),
        generated_artifacts: vec![NamedHash {
            path: "assets/catalog.json".into(),
            sha256: lifecycle.policy.generated_artifacts[0].sha256.clone(),
        }],
    };
    append_event(&mut lifecycle, LifecycleState::InReview, &fingerprints);
    append_event(&mut lifecycle, LifecycleState::Approved, &fingerprints);
    record::validate(&lifecycle).unwrap();
    let bytes = serde_json::to_vec(&lifecycle).unwrap();
    fs::write(root.join("lifecycle.json"), &bytes).unwrap();
    fixture.project.declaration.resources[0].path = "assets/catalog.json".into();
    fixture.project.declaration.resources[1].path = "assets/source.txt".into();
    fixture.project.declaration.resources[2].expected_sha256 = sha256_hex(&bytes);
    fixture.project.write_manifest();
    fixture.write_selected_configs();
    let scope = fixture.scope();
    let source = scope
        .declared_source_for_native("catalog", "source.txt", &mut NoopControl)
        .unwrap()
        .unwrap();
    assert_eq!(source.path(), "assets/source.txt");
    assert!(
        scope
            .declared_source_for_native("catalog", "assets/source.txt", &mut NoopControl)
            .unwrap()
            .is_none()
    );
}

/// Pinned schema-valid bytes and approved owner rows cannot erase drift or confer report authority.
#[test]
fn native_drift_and_report_roles_never_infer_approval() {
    let mut fixture = GateFixture::new();
    let root = fixture.project.root();
    let mut catalog: Value =
        serde_json::from_slice(&fs::read(root.join("catalog.json")).unwrap()).unwrap();
    catalog["catalog"]["metadata"]["version"] = json!("2");
    let bytes = serde_json::to_vec(&catalog).unwrap();
    fs::write(root.join("catalog.json"), &bytes).unwrap();
    fixture.project.declaration.resources[0].expected_sha256 = sha256_hex(&bytes);
    fixture.project.declaration.resources[0].native_identity.as_mut().unwrap().document_version =
        "2".into();
    let report = b"{}";
    fs::write(root.join("report.json"), report).unwrap();
    fixture.project.declaration.resources.push(ResourcePolicy {
        key: "report".into(),
        role: Role::ApplicabilityReport,
        path: "report.json".into(),
        expected_sha256: sha256_hex(report),
        native_identity: None,
        lifecycle_key: None,
        citation_label: "report-citation".into(),
    });
    fixture.profile["visible_resource_keys"] = json!(["catalog", "source", "report"]);
    fixture.project.write_manifest();
    fixture.write_selected_configs();
    let scope = fixture.scope();
    assert!(scope.resource("catalog").unwrap().native_value().is_some());
    assert!(scope.approved("catalog", &mut NoopControl).unwrap().is_none());
    assert!(scope.resource("report").is_some());
    assert!(scope.approved("report", &mut NoopControl).unwrap().is_none());
}

/// Add a real closed applicability manifest and separately approved source lifecycle.
/// Existing Catalog approval remains bound to its original independent record. The
/// report-role variant is deliberately only strict JSON: candidacy is not report validity.
fn domain_fixture(role: Role) -> GateFixture {
    let mut fixture = GateFixture::new();
    let root = fixture.project.root();
    let native_bytes = fs::read(root.join("catalog.json")).unwrap();
    let value = json!({"schema_version":"forge.applicability/1", "framework":{
        "type":"catalog", "artifact":"catalog.json", "href":"catalog.json",
        "expected_sha256":sha256_hex(&native_bytes), "inventory":{
            "root_uuid":"11111111-1111-4111-8111-111111111111",
            "document_version":"1", "oscal_version":"1.2.3",
            "control_ids":["synthetic-a"], "statement_ids":[], "fingerprint_digest":null}},
        "reviewers":[], "decisions":[], "mapping_collections":[]});
    let bytes = serde_json::to_vec(&value).unwrap();
    crate::applicability::manifest::parse(&bytes).expect("real intrinsic manifest admission");
    fs::write(root.join("applicability.json"), &bytes).unwrap();
    let mut record = record::parse(&fs::read(root.join("lifecycle.json")).unwrap()).unwrap();
    record.policy.policy_key = "synthetic-domain-policy".into();
    record.policy.source = ArtifactFingerprint {
        path: "applicability.json".into(),
        sha256: sha256_hex(&bytes),
        oscal_type: None,
        root_uuid: None,
    };
    let fingerprints = record_fingerprints(&record);
    reset_domain_approval(&mut record, &fingerprints);
    let record_bytes = serde_json::to_vec(&record).unwrap();
    fs::write(root.join("domain-lifecycle.json"), &record_bytes).unwrap();
    fixture.project.declaration.resources.extend([
        ResourcePolicy {
            key: "domain".into(),
            role,
            path: "applicability.json".into(),
            expected_sha256: sha256_hex(&bytes),
            native_identity: None,
            lifecycle_key: Some("domain-lifecycle".into()),
            citation_label: "synthetic-domain".into(),
        },
        ResourcePolicy {
            key: "domain-lifecycle".into(),
            role: Role::LifecycleRecord,
            path: "domain-lifecycle.json".into(),
            expected_sha256: sha256_hex(&record_bytes),
            native_identity: None,
            lifecycle_key: None,
            citation_label: "synthetic-domain-record".into(),
        },
    ]);
    fixture.profile["visible_resource_keys"] = json!(["catalog", "source", "domain"]);
    fixture.profile["enabled_tools"] =
        json!(["get_artifact_status", "get_recorded_applicability", "get_gap_summary"]);
    fixture.project.write_manifest();
    fixture.write_selected_configs();
    fixture
}

/// Construct a complete sorted fingerprint tuple from a real intrinsic fixture record.
fn record_fingerprints(record: &LifecycleRecord) -> FingerprintSet {
    let mut generated_artifacts: Vec<_> = record
        .policy
        .generated_artifacts
        .iter()
        .map(|artifact| NamedHash { path: artifact.path.clone(), sha256: artifact.sha256.clone() })
        .collect();
    generated_artifacts.sort();
    FingerprintSet { source_sha256: record.policy.source.sha256.clone(), generated_artifacts }
}

/// Regenerate actual deterministic event IDs after explicitly changing a fixture record.
fn reset_domain_approval(record: &mut LifecycleRecord, fingerprints: &FingerprintSet) {
    record.history.clear();
    record.state = LifecycleState::Draft;
    append_event(record, LifecycleState::InReview, fingerprints);
    append_event(record, LifecycleState::Approved, fingerprints);
    record::validate(record).expect("real intrinsic domain fixture record");
}

/// Write an explicit new intrinsic record generation and repin complete selected configs.
fn write_domain_record(fixture: &mut GateFixture, record: &LifecycleRecord) {
    record::validate(record).unwrap();
    let bytes = serde_json::to_vec(record).unwrap();
    fs::write(fixture.project.root().join("domain-lifecycle.json"), &bytes).unwrap();
    fixture
        .project
        .declaration
        .resources
        .iter_mut()
        .find(|row| row.key == "domain-lifecycle")
        .unwrap()
        .expected_sha256 = sha256_hex(&bytes);
    fixture.project.write_manifest();
    fixture.write_selected_configs();
}

/// Read only an actual owned fixture record for a new generation before production capture.
fn read_domain_record(fixture: &GateFixture) -> LifecycleRecord {
    record::parse(&fs::read(fixture.project.root().join("domain-lifecycle.json")).unwrap()).unwrap()
}

/// A real manifest source is neutral-current while its native dependency stays independently approved.
#[test]
fn domain_source_is_exact_and_neutral_core_remains_closed() {
    let fixture = domain_fixture(Role::ApplicabilityManifest);
    let scope = fixture.scope();
    assert!(scope.approved("domain", &mut NoopControl).unwrap().is_none());
    let candidate = scope.domain_source_candidate("domain", &mut NoopControl).unwrap().unwrap();
    let resource = candidate.resource();
    assert_eq!(resource.role(), Role::ApplicabilityManifest);
    assert_eq!(resource.path(), "applicability.json");
    assert!(resource.native_identity().is_none());
    assert_eq!(resource.raw_sha256(), sha256_hex(resource.bytes()));
    crate::applicability::manifest::parse(resource.bytes()).unwrap();
    let native = scope.approved("catalog", &mut NoopControl).unwrap().unwrap();
    assert_eq!(native.resource().lifecycle_key(), Some("lifecycle"));
    assert_eq!(resource.lifecycle_key(), Some("domain-lifecycle"));
    assert!(scope.domain_source_candidate("catalog", &mut NoopControl).unwrap().is_none());
    assert!(scope.domain_source_candidate("source", &mut NoopControl).unwrap().is_none());
    assert!(!Role::ApplicabilityManifest.status_supported());
    assert!(!Role::ApplicabilityReport.status_supported());
    scope.verify_inputs(&mut NoopControl).unwrap();
}

/// Strict captured report-role bytes may qualify as a source but never establish report semantics.
#[test]
fn report_source_candidate_is_not_intrinsic_report_authority() {
    let fixture = domain_fixture(Role::ApplicabilityReport);
    let scope = fixture.scope();
    let candidate = scope.domain_source_candidate("domain", &mut NoopControl).unwrap().unwrap();
    assert_eq!(candidate.resource().role(), Role::ApplicabilityReport);
    assert_eq!(candidate.resource().json().unwrap()["schema_version"], "forge.applicability/1");
    assert!(scope.approved("domain", &mut NoopControl).unwrap().is_none());
    // No domain query or stored-report parser is called: this test intentionally
    // claims only actual lifecycle-source provenance over strict original bytes.
    scope.verify_inputs(&mut NoopControl).unwrap();
}

/// An approved domain source does not independently approve a generated native dependency.
#[test]
fn generated_listing_does_not_grant_native_approval() {
    let mut fixture = domain_fixture(Role::ApplicabilityManifest);
    let root = fixture.project.root();
    let mut native_record = record::parse(&fs::read(root.join("lifecycle.json")).unwrap()).unwrap();
    native_record.history.clear();
    native_record.state = LifecycleState::Draft;
    record::validate(&native_record).unwrap();
    let bytes = serde_json::to_vec(&native_record).unwrap();
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
    assert!(scope.approved("catalog", &mut NoopControl).unwrap().is_none());
    scope.verify_inputs(&mut NoopControl).unwrap();
}

/// A different captured object with equal bytes is not the selected actual lifecycle source.
#[test]
fn domain_source_selection_is_not_hash_only() {
    let mut fixture = domain_fixture(Role::ApplicabilityManifest);
    let root = fixture.project.root();
    let bytes = fs::read(root.join("applicability.json")).unwrap();
    fs::write(root.join("other-domain.json"), &bytes).unwrap();
    fixture.project.declaration.resources.push(ResourcePolicy {
        key: "other-domain".into(),
        role: Role::ApplicabilityManifest,
        path: "other-domain.json".into(),
        expected_sha256: sha256_hex(&bytes),
        native_identity: None,
        lifecycle_key: Some("domain-lifecycle".into()),
        citation_label: "other-domain".into(),
    });
    fixture.profile["visible_resource_keys"] =
        json!(["catalog", "source", "domain", "other-domain"]);
    let mut record = read_domain_record(&fixture);
    record.policy.source.path = "other-domain.json".into();
    let fingerprints = record_fingerprints(&record);
    reset_domain_approval(&mut record, &fingerprints);
    write_domain_record(&mut fixture, &record);
    let scope = fixture.scope();
    assert!(scope.domain_source_candidate("domain", &mut NoopControl).unwrap().is_none());
    assert!(scope.domain_source_candidate("other-domain", &mut NoopControl).unwrap().is_some());
}

/// Visibility and source lifecycle binding are required even for matching held content.
#[test]
fn hidden_or_unbound_domain_source_is_unavailable() {
    let mut fixture = domain_fixture(Role::ApplicabilityManifest);
    fixture.profile["visible_resource_keys"] = json!(["catalog", "source"]);
    fixture.write_selected_configs();
    let scope = fixture.scope();
    assert!(scope.resource("domain").is_some());
    assert!(scope.domain_source_candidate("domain", &mut NoopControl).unwrap().is_none());
    drop(scope);
    fixture.profile["visible_resource_keys"] = json!(["catalog", "source", "domain"]);
    fixture
        .project
        .declaration
        .resources
        .iter_mut()
        .find(|row| row.key == "domain")
        .unwrap()
        .lifecycle_key = None;
    fixture.project.write_manifest();
    fixture.write_selected_configs();
    let scope = fixture.scope();
    assert!(scope.domain_source_candidate("domain", &mut NoopControl).unwrap().is_none());
}

/// Latest approved fingerprints must match the complete actual source/generated tuple.
#[test]
fn stale_approved_domain_fingerprints_refuse_candidacy() {
    let mut fixture = domain_fixture(Role::ApplicabilityManifest);
    let mut record = read_domain_record(&fixture);
    let mut fingerprints = record_fingerprints(&record);
    fingerprints.source_sha256 = "a".repeat(64);
    reset_domain_approval(&mut record, &fingerprints);
    write_domain_record(&mut fixture, &record);
    let scope = fixture.scope();
    assert!(scope.domain_source_candidate("domain", &mut NoopControl).unwrap().is_none());
}

/// Missing, changed and model/root-mismatched generated members cannot form a source proof.
#[test]
fn complete_generated_tuple_requires_original_pin_and_identity() {
    for case in 0..5 {
        let mut fixture = domain_fixture(Role::ApplicabilityManifest);
        let root = fixture.project.root();
        match case {
            0 => {
                fs::remove_file(root.join("catalog.json")).unwrap();
            }
            1 => {
                fs::write(root.join("catalog.json"), b"{}").unwrap();
            }
            _ => {
                let mut record = read_domain_record(&fixture);
                if case == 2 {
                    record.policy.generated_artifacts[0].root_uuid =
                        Some("22222222-2222-4222-8222-222222222222".into());
                } else if case == 3 {
                    record.policy.generated_artifacts[0].oscal_type = Some("profile".into());
                } else {
                    record.policy.generated_artifacts[0].sha256 = "a".repeat(64);
                }
                let fingerprints = record_fingerprints(&record);
                reset_domain_approval(&mut record, &fingerprints);
                write_domain_record(&mut fixture, &record);
            }
        }
        let scope = fixture.scope();
        assert!(scope.domain_source_candidate("domain", &mut NoopControl).unwrap().is_none());
    }
}

/// Full record occurrences are admitted before derived record/tuple/status retention.
#[test]
fn domain_source_uses_shared_monotonic_relationship_budget() {
    let fixture = domain_fixture(Role::ApplicabilityManifest);
    let scope = fixture.scope();
    let remaining = RELATION_LIMIT - scope.relationships.get();
    scope.charge_relationships(remaining).unwrap();
    assert!(scope.domain_source_candidate("domain", &mut NoopControl).is_err());
    assert_eq!(scope.relationships.get(), RELATION_LIMIT);
    assert!(scope.approved("catalog", &mut NoopControl).is_err());
}

/// Source, generated, private record and config originals remain in the final full proof.
#[test]
fn retained_domain_candidate_does_not_replace_original_generation_fences() {
    for name in ["applicability.json", "catalog.json", "domain-lifecycle.json", CONFIG_PATHS[1]] {
        let fixture = domain_fixture(Role::ApplicabilityManifest);
        let scope = fixture.scope();
        let candidate = scope.domain_source_candidate("domain", &mut NoopControl).unwrap().unwrap();
        fs::write(fixture.project.root().join(name), b"changed after candidacy").unwrap();
        assert!(scope.verify_inputs(&mut NoopControl).is_err());
        assert!(scope.domain_source_candidate("domain", &mut NoopControl).is_err());
        assert_eq!(candidate.resource().path(), "applicability.json");
    }
}

/// A write during actual source-proof preparation fails the final original fence before return.
#[test]
fn domain_source_late_drift_cannot_return_candidate() {
    /// Synthetic control changes an owned fixture original at the actual derivation checkpoint.
    struct Drift {
        /// Exact owned file, never an external caller-controlled capture constructor.
        target: std::path::PathBuf,
        /// One source mutation after capture; repeated fences retain the original failure.
        changed: bool,
    }
    impl WorkControl for Drift {
        /// Apply one actual fixture write while the producer still holds its sealed original.
        fn checkpoint(
            &mut self,
            stage: Stage,
            _progress: crate::workspace::preparation::ProgressUpdate,
        ) -> WorkResult<()> {
            if stage == Stage::PrepareDomain && !self.changed {
                fs::write(&self.target, b"actual late drift").unwrap();
                self.changed = true;
            }
            Ok(())
        }
        /// This control exercises actual drift, not synthetic cancellation classification.
        fn interruption(&self) -> Option<Interruption> {
            None
        }
    }
    let fixture = domain_fixture(Role::ApplicabilityManifest);
    let scope = fixture.scope();
    let mut control =
        Drift { target: fixture.project.root().join("applicability.json"), changed: false };
    assert!(scope.domain_source_candidate("domain", &mut control).is_err());
    assert!(control.changed);
}

/// Typed interruption remains interruption rather than approval unavailability.
#[test]
fn domain_source_interruption_remains_typed() {
    /// Fixed typed cancellation control has no detached scope or file read path.
    struct Cancel;
    impl WorkControl for Cancel {
        /// Reject every actual cooperative fence using the original typed cancellation reason.
        fn checkpoint(
            &mut self,
            _stage: Stage,
            _progress: crate::workspace::preparation::ProgressUpdate,
        ) -> WorkResult<()> {
            Err(WorkError::Interrupted(Interruption::CancelRequested))
        }
        /// Preserve the same sticky cancellation reason through all gate projections.
        fn interruption(&self) -> Option<Interruption> {
            Some(Interruption::CancelRequested)
        }
    }
    let fixture = domain_fixture(Role::ApplicabilityManifest);
    let scope = fixture.scope();
    assert!(matches!(
        scope.domain_source_candidate("domain", &mut Cancel),
        Err(WorkError::Interrupted(Interruption::CancelRequested))
    ));
}
