//! Genuine synthetic applicability/lifecycle original controls, proposed and unexecuted.
//! Fixture generation alone uses maintained file-oriented producers; actual factory
//! calls consume the once-captured originals and real proof. No authenticated owner,
//! caller-constructed success token or measured platform acceptance is fabricated.

use super::*;
use crate::applicability::model::ReportFilters;
use crate::mapping::{
    inventory,
    manifest::{ResourceManifest, ResourceType},
};
use crate::workspace::preparation::{Interruption, NoopControl, Stage, test_support::Recorder};
use serde_json::json;

/// Own every actual original and its canonical test root through proof lifetime.
pub(crate) struct Fixture {
    /// Sole actual temporary directory owner.
    directory: tempfile::TempDir,
    /// Qualified canonical test root; canonicalization is fixture orchestration only.
    root: PathBuf,
    /// Actual fixture native framework family.
    profile: bool,
}

impl Fixture {
    /// Generate genuine saved native/report originals before creating any capture.
    pub(crate) fn new(profile_framework: bool) -> Self {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let framework_catalog =
            catalog("22222222-2222-4222-8222-222222222222", &["c1", "c2", "c3"]);
        write_json(&root.join("framework.json"), &framework_catalog);
        write_json(
            &root.join("policy.json"),
            &catalog("11111111-1111-4111-8111-111111111111", &["policy-1", "policy-2"]),
        );
        if profile_framework {
            write_json(&root.join("resolved-catalog.json"), &framework_catalog);
            write_json(&root.join("framework.json"), &profile());
        }
        let mut framework = resource(
            if profile_framework { ResourceType::Profile } else { ResourceType::Catalog },
            "framework.json",
        );
        if profile_framework {
            framework.resolved_catalog = Some("resolved-catalog.json".into());
            framework.resolved_catalog_attestation = Some(true);
            framework.expected_resolved_catalog_sha256 =
                Some(sha256_hex(&std::fs::read(root.join("resolved-catalog.json")).unwrap()));
        }
        let loaded = inventory::load(&root, "synthetic framework", &framework).unwrap();
        framework.expected_sha256 = Some(loaded.evidence.raw_sha256.clone());
        framework.inventory = Some(loaded.snapshot());
        write_json(&root.join("mapping-manifest.json"), &mapping_source(&framework, false));
        let built =
            crate::mapping::prepare(&root.join("mapping-manifest.json"), None, false).unwrap();
        std::fs::write(root.join("mapping.json"), built.artifact_json.as_bytes()).unwrap();
        write_json(&root.join("applicability.json"), &applicability_source(&framework));
        let report = crate::applicability::prepare_analysis(
            &root.join("applicability.json"),
            ReportFilters::default(),
        )
        .unwrap();
        write_json(&root.join("report.json"), &serde_json::to_value(&report.report).unwrap());
        let fixture = Self { directory, root, profile: profile_framework };
        fixture.approve("report.json");
        fixture.write_locator();
        fixture
    }

    /// Borrow the actual qualified fixture root while this cfg-only owner stays alive.
    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    /// Record valid explicitly synthetic neutral approval of actual complete originals.
    fn approve(&self, source: &str) {
        let mut files = vec![
            ("framework.json", if self.profile { "profile" } else { "catalog" }),
            ("mapping.json", "mapping-collection"),
            ("policy.json", "catalog"),
        ];
        if self.profile {
            files.push(("resolved-catalog.json", "catalog"));
        }
        files.sort_by_key(|row| row.0);
        let generated: Vec<Value> = files.into_iter().map(|(path, kind)| {
            let raw = std::fs::read(self.root.join(path)).unwrap();
            let value: Value = serde_json::from_slice(&raw).unwrap();
            let model = if kind == "profile" { OscalModelType::Profile }
                else if kind == "mapping-collection" { OscalModelType::Mapping }
                else { OscalModelType::Catalog };
            assert!(crate::validate::validate_artifact(&value, model).unwrap().is_valid);
            json!({"path":path,"sha256":sha256_hex(&raw),"oscal_type":kind,"root_uuid":value[kind]["uuid"]})
        }).collect();
        let source_raw = std::fs::read(self.root.join(source)).unwrap();
        let fingerprint = json!({"path":source,"sha256":sha256_hex(&source_raw)});
        let fingerprints = json!({"source_sha256":sha256_hex(&source_raw),
            "generated_artifacts":generated.iter().map(|row| json!({"path":row["path"],"sha256":row["sha256"]})).collect::<Vec<_>>()});
        let value = json!({"schema_version":record::LEGACY_SCHEMA_VERSION,"policy":{
            "policy_key":"synthetic-applicability","version_key":"v1","title":"Synthetic report-source lifecycle",
            "owner_keys":["owner"],"source":fingerprint,"generated_artifacts":generated},
            "parties":[{"key":"owner","roles":["owner"]},{"key":"reviewer","roles":["reviewer"]},{"key":"approver","roles":["approver"]}],
            "approval_policy":{"schema_version":record::APPROVAL_POLICY_VERSION,"required_roles":[{"role":"approver","count":1}],"separation":{}},
            "review":{"cadence_days":30,"next_review_date":"2026-12-01","due_soon_days":7,"timezone_policy":"date-only"},
            "state":"approved","history":[
                {"sequence":1,"event_id":"pending","previous_state":"draft","next_state":"in-review","actor_key":"reviewer","declared_role":"reviewer","timestamp":"2026-08-25T10:00:00Z","rationale":"Synthetic fixture only.","fingerprints":fingerprints},
                {"sequence":2,"event_id":"pending","previous_state":"in-review","next_state":"approved","actor_key":"approver","declared_role":"approver","timestamp":"2026-08-25T10:01:00Z","rationale":"Synthetic declared approval only.","fingerprints":fingerprints}]});
        let mut record: LifecycleRecord = serde_json::from_value(value).unwrap();
        rewrite_event_ids(&mut record);
        record::validate(&record).unwrap();
        std::fs::write(self.root.join("lifecycle.json"), serde_json::to_vec(&record).unwrap())
            .unwrap();
    }

    /// Write the complete sorted explicit source roster, not a proof constructor.
    fn write_locator(&self) {
        let mut sources = vec![
            json!({"key":"framework","path":"framework.json","model":if self.profile {"profile"} else {"catalog"},"resolved_catalog_key":if self.profile {json!("resolved")} else {Value::Null}}),
            json!({"key":"lifecycle","path":"lifecycle.json","model":"lifecycle-record","resolved_catalog_key":null}),
            json!({"key":"manifest","path":"applicability.json","model":"applicability-manifest","resolved_catalog_key":null}),
            json!({"key":"mapping","path":"mapping.json","model":"mapping","resolved_catalog_key":null}),
            json!({"key":"policy","path":"policy.json","model":"catalog","resolved_catalog_key":null}),
            json!({"key":"report","path":"report.json","model":"applicability-report","resolved_catalog_key":null}),
        ];
        if self.profile {
            sources.push(json!({"key":"resolved","path":"resolved-catalog.json","model":"resolved-catalog","resolved_catalog_key":null}));
        }
        write_json(
            &self.root.join("locator.json"),
            &json!({"schema_version":LOCATOR_SCHEMA,
            "applicability_manifest_key":"manifest","applicability_report_key":"report","lifecycle_record_key":"lifecycle",
            "framework_key":"framework","mapping_keys":["mapping"],"sources":sources}),
        );
    }

    /// Create one genuine capture and prepare against its actual locator original.
    fn pending(&self, ledger: &mut ContractLedger) -> (ReviewCapture, PendingApplicabilityClosure) {
        let mut capture = ReviewCapture::new(&self.root, &[], ledger, &mut NoopControl).unwrap();
        let locator = capture
            .required(
                Path::new("locator.json"),
                CaptureRole::ReviewPrivateConfig,
                Pool::Auxiliary,
                MAX_LOCATOR_BYTES,
                ledger,
                &mut NoopControl,
            )
            .unwrap();
        let pending = prepare(&mut capture, locator, ledger, &mut NoopControl).unwrap();
        (capture, pending)
    }

    /// Consume genuine capture, pending facts and full final proof in one ledger.
    fn seal(&self) -> ApprovedApplicabilityClosure {
        let mut ledger = ContractLedger::default();
        let (capture, pending) = self.pending(&mut ledger);
        pending.seal(Rc::new(capture.finish()), &mut ledger, &mut NoopControl).unwrap()
    }

    /// Refuse only through the actual complete factory, never a transport self-check.
    fn refuses(&self) {
        let mut ledger = ContractLedger::default();
        let mut capture =
            ReviewCapture::new(&self.root, &[], &mut ledger, &mut NoopControl).unwrap();
        let locator = capture
            .required(
                Path::new("locator.json"),
                CaptureRole::ReviewPrivateConfig,
                Pool::Auxiliary,
                MAX_LOCATOR_BYTES,
                &mut ledger,
                &mut NoopControl,
            )
            .unwrap();
        assert!(prepare(&mut capture, locator, &mut ledger, &mut NoopControl).is_err());
    }
}

/// Persist one fixture Value; this writer is never production authority.
fn write_json(path: &Path, value: &Value) {
    std::fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

/// Borrow a saved fixture form only for targeted mutations.
fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

/// Recompute maintained event identities after a targeted valid synthetic record change.
fn rewrite_event_ids(record: &mut LifecycleRecord) {
    for index in 0..record.history.len() {
        record.history[index].event_id = record::event_id(record, &record.history[index]).unwrap();
    }
}

/// Construct genuine official-schema Catalog data used by both maintained engines.
fn catalog(uuid: &str, ids: &[&str]) -> Value {
    json!({"catalog": {"uuid": uuid, "metadata": {
        "title": "Synthetic native controls", "last-modified": "2026-08-25T08:00:00Z",
        "version": "1.0.0", "oscal-version": "1.2.3"}, "groups": [{"id": "group-1",
        "title": "Synthetic group", "controls": ids.iter().map(|id| json!({"id": id,
            "title": format!("Control {id}"), "parts": [{"id": format!("{id}_smt"),
                "name": "statement", "prose": format!("Actual synthetic statement {id}.")}]}))
            .collect::<Vec<_>>()}]}})
}

/// Construct a genuine Profile with an explicitly supplied actual resolved Catalog.
fn profile() -> Value {
    json!({"profile": {"uuid": "33333333-3333-4333-8333-333333333333", "metadata": {
        "title": "Synthetic profile", "last-modified": "2026-08-25T08:00:00Z",
        "version": "1.0.0", "oscal-version": "1.2.3"}, "imports": [{"href": "resolved-catalog.json", "include-all": {}}]}})
}

/// Build a real resource descriptor; fixture schema/inventory admission adds pins.
fn resource(resource_type: ResourceType, name: &str) -> ResourceManifest {
    ResourceManifest {
        resource_type,
        artifact: name.into(),
        href: name.to_owned(),
        resolved_catalog: None,
        resolved_catalog_attestation: None,
        expected_sha256: None,
        expected_resolved_catalog_sha256: None,
        inventory: None,
    }
}

/// Author reviewed many-to-many fixture input for the actual maintained Mapping builder.
fn mapping_source(framework: &ResourceManifest, self_mapping: bool) -> Value {
    let source = if self_mapping {
        framework.clone()
    } else {
        resource(ResourceType::Catalog, "policy.json")
    };
    let ids = if self_mapping { ["c1", "c2"] } else { ["policy-1", "policy-2"] };
    json!({"schema_version": "forge.mapping-manifest/1", "collection": {
        "key": "native-collection-key", "title": "Synthetic reviewed collection", "version": "1.0.0",
        "last_modified": "2026-08-25T08:00:00Z"}, "reviewers": [{"key": "mapper", "type": "person", "name": "Mapping Reviewer"}],
        "provenance": {"method": "human", "matching_rationale": "semantic", "status": "complete",
            "mapping_description": "Synthetic current participation.", "reviewer_keys": ["mapper"], "reviewed_at": "2026-08-25T08:00:00Z"},
        "mapping": {"key": "native-mapping-key", "scope": "control-only", "source": source, "target": framework,
            "maps": [{"key": "pair", "relationship": "intersects-with", "sources": ids.iter().map(|id|
                json!({"type": "control", "id_ref": id})).collect::<Vec<_>>(),
                "targets": [{"type": "control", "id_ref": "c1"}, {"type": "control", "id_ref": "c2"}],
                "reviewer_key": "mapper", "reviewed_at": "2026-08-25T08:00:00Z", "rationale": "Reviewed synthetic native pair."}]}})
}

/// Author actual closed scope decisions with every default-filter classification preserved.
fn applicability_source(framework: &ResourceManifest) -> Value {
    json!({"schema_version": "forge.applicability/1", "framework": framework,
        "reviewers": [{"key": "scope-reviewer", "type": "person", "name": "Scope Reviewer"}],
        "decisions": [{"control_id": "c1", "state": "applicable", "reviewer_key": "scope-reviewer", "reviewed_at": "2026-08-25T09:00:00Z"},
            {"control_id": "c2", "state": "deferred", "reviewer_key": "scope-reviewer", "reviewed_at": "2026-08-25T09:00:00Z",
                "rationale": "Pending an actual synthetic scope decision.", "revisit_date": "2026-10-01"}],
        "mapping_collections": ["mapping.json"]})
}

/// Genuine complete Catalog relation preserves all explicit and omitted controls.
#[test]
fn genuine_catalog_full_report_source_seals_complete_control_roster() {
    let fixture = Fixture::new(false);
    let closure = fixture.seal();
    assert!(fixture.directory.path().exists());
    assert!(fixture.root().join("report.json").is_file());
    assert_eq!(closure.source_pins().len(), 6);
    assert_eq!(closure.complete_counts().total, 3);
    assert_eq!(closure.complete_counts().applicable_mapped, 1);
    assert_eq!(closure.complete_counts().deferred, 1);
    assert_eq!(closure.complete_counts().under_review, 1);
    assert_eq!(closure.complete_maps(), 1);
    assert_eq!(closure.complete_pair_inspections(), 4);
    let facts = closure.control_facts();
    assert_eq!(facts.iter().map(ControlFacts::control_id).collect::<Vec<_>>(), ["c1", "c2", "c3"]);
    assert_eq!(facts[0].positive_count(), 1);
    assert_eq!(facts[1].positive_count(), 1);
    assert_eq!(facts[0].decision_index(), Some(0));
    assert_eq!(facts[1].decision_index(), Some(1));
    assert_eq!(facts[2].decision_index(), None);
    assert_eq!(closure.decision_original(1).unwrap()["control_id"], "c2");
    assert!(closure.decision_original(2).is_none());
    closure.verify_inputs(&mut ContractLedger::default(), &mut NoopControl).unwrap();
}

/// Actual Profile and explicit companion share genuine full report/current proof.
#[test]
fn genuine_profile_companion_report_source_seals_complete_union() {
    let fixture = Fixture::new(true);
    let closure = fixture.seal();
    assert_eq!(closure.source_pins().len(), 7);
    assert_eq!(closure.framework_evidence().resource_type, ResourceType::Profile);
    assert!(closure.framework_evidence().resolved_catalog_sha256.is_some());
    assert_eq!(closure.complete_counts().total, 3);
    assert_eq!(closure.complete_pair_inspections(), 4);
}

/// A current synthetic approval cannot authorize a different full report reviewer field.
#[test]
fn complete_report_private_reviewer_difference_refuses_after_refreshed_approval() {
    let fixture = Fixture::new(false);
    drop(fixture.seal());
    let path = fixture.root.join("report.json");
    let mut value = read_json(&path);
    value["reviewers"][0]["name"] = json!("Different private asserted reviewer");
    write_json(&path, &value);
    fixture.approve("report.json");
    fixture.refuses();
}

/// Actual filtered report generation cannot replace the required full default-filter report.
#[test]
fn genuine_filtered_report_refuses_even_with_current_recorded_approval() {
    let fixture = Fixture::new(false);
    drop(fixture.seal());
    let report = crate::applicability::prepare_analysis(
        &fixture.root.join("applicability.json"),
        ReportFilters {
            state: Some(model::GapClassification::ApplicableMapped),
            ..ReportFilters::default()
        },
    )
    .unwrap();
    write_json(&fixture.root.join("report.json"), &serde_json::to_value(&report.report).unwrap());
    fixture.approve("report.json");
    fixture.refuses();
}

/// Manifest-source lifecycle is unselected; no fallback silently substitutes its approval.
#[test]
fn manifest_as_lifecycle_source_refuses_without_report_source_fallback() {
    let fixture = Fixture::new(false);
    drop(fixture.seal());
    fixture.approve("applicability.json");
    fixture.refuses();
}

/// Changed complete report bytes cannot reuse old current/approved source fingerprints.
#[test]
fn actual_report_raw_drift_refuses_old_recorded_source_hash() {
    let fixture = Fixture::new(false);
    drop(fixture.seal());
    let mut raw = std::fs::read(fixture.root.join("report.json")).unwrap();
    raw.push(b'\n');
    std::fs::write(fixture.root.join("report.json"), raw).unwrap();
    fixture.refuses();
}

/// Schema-valid native endpoint fingerprints must match actual current native subjects.
#[test]
fn native_mapping_subject_fingerprint_refuses_after_valid_schema_and_approval() {
    let fixture = Fixture::new(false);
    drop(fixture.seal());
    let mut value = read_json(&fixture.root.join("mapping.json"));
    let props = value["mapping-collection"]["mappings"][0]["maps"][0]["sources"][1]["props"]
        .as_array_mut()
        .unwrap();
    let property = props.iter_mut().find(|prop| prop["name"] == "subject-sha256").unwrap();
    property["value"] = json!("a".repeat(64));
    assert!(crate::validate::validate_artifact(&value, OscalModelType::Mapping).unwrap().is_valid);
    write_json(&fixture.root.join("mapping.json"), &value);
    fixture.approve("report.json");
    fixture.refuses();
}

/// Complete intrinsic lifecycle with no latest Approved history cannot seal a report-source proof.
#[test]
fn genuine_unapproved_record_refuses_without_schema_or_hash_authority() {
    let fixture = Fixture::new(false);
    drop(fixture.seal());
    let mut record: LifecycleRecord =
        serde_json::from_slice(&std::fs::read(fixture.root.join("lifecycle.json")).unwrap())
            .unwrap();
    record.history.truncate(1);
    record.state = LifecycleState::InReview;
    rewrite_event_ids(&mut record);
    record::validate(&record).unwrap();
    std::fs::write(fixture.root.join("lifecycle.json"), serde_json::to_vec(&record).unwrap())
        .unwrap();
    fixture.refuses();
}

/// Missing original generated dependency is rejected from the declared union before capture.
#[test]
fn missing_current_original_declaration_refuses_complete_union() {
    let fixture = Fixture::new(false);
    drop(fixture.seal());
    let mut value = read_json(&fixture.root.join("locator.json"));
    value["sources"].as_array_mut().unwrap().retain(|row| row["key"] != "policy");
    write_json(&fixture.root.join("locator.json"), &value);
    fixture.refuses();
}

/// Extra unrelated source declarations cannot inflate or replace the required original union.
#[test]
fn unrelated_declared_original_refuses_whole_union() {
    let fixture = Fixture::new(false);
    drop(fixture.seal());
    write_json(
        &fixture.root.join("extra.json"),
        &catalog("44444444-4444-4444-8444-444444444444", &["extra"]),
    );
    let mut value = read_json(&fixture.root.join("locator.json"));
    value["sources"].as_array_mut().unwrap().push(
        json!({"key":"zz-extra","path":"extra.json","model":"catalog","resolved_catalog_key":null}),
    );
    write_json(&fixture.root.join("locator.json"), &value);
    fixture.refuses();
}

/// An explicit Profile companion cannot be omitted or replaced by hash-only facts.
#[test]
fn missing_profile_companion_original_refuses_complete_relation() {
    let fixture = Fixture::new(true);
    drop(fixture.seal());
    let mut value = read_json(&fixture.root.join("locator.json"));
    value["sources"].as_array_mut().unwrap()[0]["resolved_catalog_key"] = Value::Null;
    write_json(&fixture.root.join("locator.json"), &value);
    fixture.refuses();
}

/// Equal complete bytes in a second genuine capture cannot substitute original owner identity.
#[test]
fn same_bytes_different_actual_capture_cannot_seal_pending_owner() {
    let fixture = Fixture::new(false);
    let mut ledger = ContractLedger::default();
    let (first, pending) = fixture.pending(&mut ledger);
    // This is a second independent invocation, not a reset of the first ledger.
    let mut foreign_ledger = ContractLedger::default();
    let (second, other) = fixture.pending(&mut foreign_ledger);
    drop(other);
    assert!(matches!(
        pending.seal(Rc::new(second.finish()), &mut ledger, &mut NoopControl),
        Err(ContractError::Binding)
    ));
    drop(first);
}

/// Extra actual Source original after preparation cannot satisfy complete source-count sealing.
#[test]
fn added_actual_source_original_refuses_pending_complete_count() {
    let fixture = Fixture::new(false);
    let mut ledger = ContractLedger::default();
    let (mut capture, pending) = fixture.pending(&mut ledger);
    write_json(
        &fixture.root.join("extra.json"),
        &catalog("44444444-4444-4444-8444-444444444444", &["extra"]),
    );
    capture
        .required(
            Path::new("extra.json"),
            CaptureRole::Catalog,
            Pool::Source,
            MAX_SOURCE_FILE,
            &mut ledger,
            &mut NoopControl,
        )
        .unwrap();
    assert!(matches!(
        pending.seal(Rc::new(capture.finish()), &mut ledger, &mut NoopControl),
        Err(ContractError::Binding)
    ));
}

/// Late original tail drift is detected by the actual full original-generation fence.
#[test]
fn late_report_original_tail_drift_refuses_final_seal() {
    let fixture = Fixture::new(false);
    let mut ledger = ContractLedger::default();
    let (capture, pending) = fixture.pending(&mut ledger);
    let mut raw = std::fs::read(fixture.root.join("report.json")).unwrap();
    raw.push(b'\n');
    std::fs::write(fixture.root.join("report.json"), raw).unwrap();
    assert!(matches!(
        pending.seal(Rc::new(capture.finish()), &mut ledger, &mut NoopControl),
        Err(ContractError::Binding)
    ));
}

/// Caller cancellation at sealing remains exact and sticky in the same command ledger.
#[test]
fn same_caller_control_stop_cannot_seal_then_reopen_with_success() {
    let fixture = Fixture::new(false);
    let mut ledger = ContractLedger::default();
    let (capture, pending) = fixture.pending(&mut ledger);
    let held = Rc::new(capture.finish());
    let mut stop = Recorder::at(Stage::PrepareDomain, 1);
    assert!(matches!(
        pending.seal(held.clone(), &mut ledger, &mut stop),
        Err(ContractError::Interrupted(Interruption::CancelRequested))
    ));
    assert!(matches!(
        held.verify_inputs(&mut ledger, &mut NoopControl),
        Err(ContractError::Interrupted(Interruption::CancelRequested))
    ));
}

/// A genuine captured invocation latches byte-work exhaustion before subsequent factory work.
#[test]
fn shared_byte_work_exhaustion_is_typed_and_sticky() {
    let fixture = Fixture::new(false);
    let mut ledger = ContractLedger::default();
    let mut capture =
        ReviewCapture::new(&fixture.root, &[], &mut ledger, &mut NoopControl).unwrap();
    let locator = capture
        .required(
            Path::new("locator.json"),
            CaptureRole::ReviewPrivateConfig,
            Pool::Auxiliary,
            MAX_LOCATOR_BYTES,
            &mut ledger,
            &mut NoopControl,
        )
        .unwrap();
    // Capture has already charged positive work, so a further complete ceiling
    // deliberately exceeds it; this tests prior exhaustion, not a guessed phase.
    assert!(matches!(ledger.bytes(256 * 1024 * 1024), Err(ContractError::Capacity)));
    assert!(matches!(
        prepare(&mut capture, locator, &mut ledger, &mut NoopControl),
        Err(ContractError::Capacity)
    ));
    assert!(matches!(ledger.checkpoint(&mut NoopControl), Err(ContractError::Capacity)));
}

/// Required null companion fields and closed locator metadata cannot be silently defaulted.
#[test]
fn strict_locator_rejects_unknown_and_omitted_required_null_fields() {
    for unknown in [false, true] {
        let fixture = Fixture::new(false);
        drop(fixture.seal());
        let mut value = read_json(&fixture.root.join("locator.json"));
        if unknown {
            value["unknown"] = json!(true);
        } else {
            value["sources"][0].as_object_mut().unwrap().remove("resolved_catalog_key");
        }
        write_json(&fixture.root.join("locator.json"), &value);
        fixture.refuses();
    }
}

/// Reserved report output cannot alias any actual Source original at the real capture boundary.
#[test]
fn report_output_namespace_alias_refuses_actual_source_registration() {
    let fixture = Fixture::new(false);
    let mut ledger = ContractLedger::default();
    let mut capture = ReviewCapture::new(
        &fixture.root,
        &[Path::new("report.json")],
        &mut ledger,
        &mut NoopControl,
    )
    .unwrap();
    let locator = capture
        .required(
            Path::new("locator.json"),
            CaptureRole::ReviewPrivateConfig,
            Pool::Auxiliary,
            MAX_LOCATOR_BYTES,
            &mut ledger,
            &mut NoopControl,
        )
        .unwrap();
    assert!(prepare(&mut capture, locator, &mut ledger, &mut NoopControl).is_err());
}

// Test-only successor helper inside the existing fixture-owning cfg module.
impl Fixture {
    /// Regenerate genuine native Mapping/applicability/lifecycle after one real framework change.
    pub(crate) fn refresh_framework_statement_for_supersession(&self) {
        let catalog_path =
            self.root.join(if self.profile { "resolved-catalog.json" } else { "framework.json" });
        let mut catalog = read_json(&catalog_path);
        catalog["catalog"]["metadata"]["version"] = json!("2.0.0");
        catalog["catalog"]["groups"][0]["controls"][0]["parts"][0]["prose"] =
            json!("Changed synthetic control c1 statement.");
        write_json(&catalog_path, &catalog);
        if self.profile {
            let mut profile = read_json(&self.root.join("framework.json"));
            profile["profile"]["metadata"]["version"] = json!("2.0.0");
            write_json(&self.root.join("framework.json"), &profile);
        }
        let mut framework = resource(
            if self.profile { ResourceType::Profile } else { ResourceType::Catalog },
            "framework.json",
        );
        if self.profile {
            framework.resolved_catalog = Some("resolved-catalog.json".into());
            framework.resolved_catalog_attestation = Some(true);
            framework.expected_resolved_catalog_sha256 =
                Some(sha256_hex(&std::fs::read(self.root.join("resolved-catalog.json")).unwrap()));
        }
        let loaded =
            inventory::load(&self.root, "synthetic changed framework", &framework).unwrap();
        framework.expected_sha256 = Some(loaded.evidence.raw_sha256.clone());
        framework.inventory = Some(loaded.snapshot());
        write_json(&self.root.join("mapping-manifest.json"), &mapping_source(&framework, false));
        let built =
            crate::mapping::prepare(&self.root.join("mapping-manifest.json"), None, false).unwrap();
        std::fs::write(self.root.join("mapping.json"), built.artifact_json.as_bytes()).unwrap();
        write_json(&self.root.join("applicability.json"), &applicability_source(&framework));
        let report = crate::applicability::prepare_analysis(
            &self.root.join("applicability.json"),
            ReportFilters::default(),
        )
        .unwrap();
        write_json(&self.root.join("report.json"), &serde_json::to_value(&report.report).unwrap());
        self.approve("report.json");
        self.write_locator();
    }
}
