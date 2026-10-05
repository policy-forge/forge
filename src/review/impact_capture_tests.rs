//! Genuine captured-reader controls over complete maintained native Impact fixtures.
//!
//! Positive fixtures obtain their full expected report through the maintained file producer.
//! Tests retain actual temporary roots and use real capture registrations and leases.

use super::*;
use crate::framework::model::ImpactReport;
use crate::hashing::sha256_hex;
use crate::review::capture::Pool;
use crate::workspace::preparation::{Interruption, NoopControl, Stage, test_support::Recorder};
use serde_json::{Value, json};

/// Redistributable framework root shared by genuine old/new native fixtures.
const FRAMEWORK: &str = "77777777-7777-4777-8777-777777777777";
/// Distinct native policy root used only by the maintained Mapping producer.
const POLICY: &str = "88888888-8888-4888-8888-888888888888";

/// Genuine fixture root remains alive throughout producers, reader and original fences.
struct Fixture {
    /// Actual filesystem allocation owner; never shortened to a detached path.
    directory: tempfile::TempDir,
    /// Complete expected report returned by the maintained full file analysis.
    expected: ImpactReport,
}

/// Serialize ordinary fixture declarations; no native report facts are manufactured.
fn bytes(value: &Value) -> Vec<u8> {
    serde_json::to_vec_pretty(value).expect("fixture JSON")
}

/// Create genuine schema-valid Catalog source data with full statement subjects.
fn catalog(uuid: &str, version: &str, controls: &[(&str, &str)]) -> Value {
    json!({"catalog":{"uuid":uuid,"metadata":{"title":"Synthetic captured framework","last-modified":"2026-08-25T12:00:00Z","version":version,"oscal-version":"1.2.3"},"groups":[{"id":"group-1","title":"Synthetic group","controls":controls.iter().map(|(id,prose)| json!({"id":id,"title":format!("Control {id}"),"parts":[{"id":format!("{id}_smt"),"name":"statement","prose":prose}]})).collect::<Vec<_>>()}]}})
}

/// Bind fixture declarations to exact genuine original hashes and native tuple expectations.
fn resource(artifact: &str, raw: &[u8], version: &str) -> Value {
    json!({"type":"catalog","artifact":artifact,"expected_sha256":sha256_hex(raw),"root_uuid":FRAMEWORK,"document_version":version,"oscal_version":"1.2.3"})
}

/// Produce a complete ordinary report from the declared actual file closure.
fn produce(base: &Path) -> ImpactReport {
    let raw = std::fs::read(base.join("impact.json")).expect("actual Impact manifest");
    let manifest = crate::framework::manifest::parse(&raw).expect("maintained native manifest");
    crate::framework::analysis::analyze(base, &manifest, ImpactFilters::default())
        .expect("complete maintained file producer")
        .0
}

impl Fixture {
    /// Install full native old/new files and obtain the complete current report authentically.
    fn basic() -> Self {
        let directory = tempfile::tempdir().expect("actual fixture lifetime");
        let base = directory.path().join("impact");
        std::fs::create_dir(&base).unwrap();
        let old = bytes(&catalog(
            FRAMEWORK,
            "1.0.0",
            &[("unchanged", "Same"), ("changed", "Old"), ("removed", "Removed")],
        ));
        let new = bytes(&catalog(
            FRAMEWORK,
            "2.0.0 private-name",
            &[("unchanged", "Same"), ("changed", "New"), ("added", "Added")],
        ));
        std::fs::write(base.join("old.json"), &old).unwrap();
        std::fs::write(base.join("new.json"), &new).unwrap();
        std::fs::write(base.join("impact.json"),bytes(&json!({"schema_version":"forge.framework-impact/1","old":resource("old.json",&old,"1.0.0"),"new":resource("new.json",&new,"2.0.0 private-name"),"mapping_collections":[]}))).unwrap();
        let expected = produce(&base);
        std::fs::write(
            base.join("current-report.json"),
            serde_json::to_vec_pretty(&expected).unwrap(),
        )
        .unwrap();
        std::fs::write(directory.path().join("impact-locator.json"),bytes(&json!({"schema_version":LOCATOR_SCHEMA,"manifest_path":"impact/impact.json","report_path":"impact/current-report.json","old_framework_source_key":"old-framework","new_framework_source_key":"new-framework","old_resolved_source_key":null,"new_resolved_source_key":null}))).unwrap();
        Self { directory, expected }
    }

    /// Return the exact actual root-relative native manifest base.
    fn base(&self) -> PathBuf {
        self.directory.path().join("impact")
    }

    /// Read an ordinary manifest declaration before an explicit fixture edit.
    fn manifest(&self) -> Value {
        serde_json::from_slice(&std::fs::read(self.base().join("impact.json")).unwrap()).unwrap()
    }

    /// Write only an explicitly declared native manifest fixture edit.
    fn set_manifest(&self, value: &Value) {
        std::fs::write(self.base().join("impact.json"), bytes(value)).unwrap();
    }

    /// Read the actual private locator declaration before an explicit negative fixture edit.
    fn locator(&self) -> Value {
        serde_json::from_slice(
            &std::fs::read(self.directory.path().join("impact-locator.json")).unwrap(),
        )
        .unwrap()
    }

    /// Write an ordinary private locator assertion; it never issues a native capability.
    fn set_locator(&self, value: &Value) {
        std::fs::write(self.directory.path().join("impact-locator.json"), bytes(value)).unwrap();
    }

    /// Regenerate the entire stored current report using the maintained native file producer.
    fn refresh(&mut self) {
        self.expected = produce(&self.base());
        std::fs::write(
            self.base().join("current-report.json"),
            serde_json::to_vec_pretty(&self.expected).unwrap(),
        )
        .unwrap();
    }

    /// Register the actual locator through the purpose-qualified capture API on one ledger/control.
    fn capture(&self, ledger: &mut ContractLedger, control: &mut dyn WorkControl) -> ReviewCapture {
        let root = self.directory.path().canonicalize().expect("qualified actual temporary root");
        let mut capture = ReviewCapture::new_supersession(
            &root,
            Path::new("review-result.json"),
            ledger,
            control,
        )
        .expect("actual capture session");
        capture
            .required_supersession_auxiliary(
                Path::new("impact-locator.json"),
                AuxiliaryPurpose::ImpactLocator,
                ledger,
                control,
            )
            .expect("actual locator registration");
        capture
    }

    /// Convert both genuine native frameworks to Profiles with explicit Catalog companions.
    fn profiles(&mut self) {
        let mut manifest = self.manifest();
        for (side, version) in [("old", "1.0.0"), ("new", "2.0.0 private-name")] {
            let artifact = format!("{side}.json");
            let companion = format!("{side}-resolved.json");
            let catalog = std::fs::read(self.base().join(&artifact)).unwrap();
            std::fs::write(self.base().join(&companion), &catalog).unwrap();
            let profile = bytes(
                &json!({"profile":{"uuid":FRAMEWORK,"metadata":{"title":"Synthetic Profile","last-modified":"2026-08-25T12:00:00Z","version":version,"oscal-version":"1.2.3"},"imports":[{"href":"unopened-import.json","include-all":{}}]}}),
            );
            std::fs::write(self.base().join(&artifact), &profile).unwrap();
            manifest[side] = json!({"type":"profile","artifact":artifact,"resolved_catalog":companion,"resolved_catalog_attestation":true,"expected_sha256":sha256_hex(&profile),"expected_resolved_catalog_sha256":sha256_hex(&catalog),"root_uuid":FRAMEWORK,"document_version":version,"oscal_version":"1.2.3"});
        }
        self.set_manifest(&manifest);
        let mut locator = self.locator();
        locator["old_resolved_source_key"] = json!("old-resolved");
        locator["new_resolved_source_key"] = json!("new-resolved");
        self.set_locator(&locator);
        self.refresh();
    }

    /// Obtain the real mandatory applicability inventory scaffold under its declared own base.
    fn add_applicability(&mut self, nested: bool, with_mapping: bool) {
        let own_base = if nested { self.base().join("nested") } else { self.base() };
        if nested {
            std::fs::create_dir(&own_base).unwrap();
        }
        let framework = if nested {
            let path = own_base.join("baseline.json");
            std::fs::copy(self.base().join("old.json"), &path).unwrap();
            path
        } else {
            self.base().join("old.json")
        };
        let output = own_base.join("applicability.json");
        crate::applicability::execute_init(&framework, None, Some(&output))
            .expect("genuine native inventory scaffold");
        let mut app: Value = serde_json::from_slice(&std::fs::read(&output).unwrap()).unwrap();
        app["framework"]["href"] = json!("private-framework-label.json");
        if with_mapping {
            app["reviewers"] = json!([{"key":"owner","type":"person","name":"Synthetic Owner"}]);
            app["decisions"] = json!([{"control_id":"changed","state":"applicable","reviewer_key":"owner","reviewed_at":"2026-08-25T12:00:00Z"},{"control_id":"removed","state":"not-applicable","reviewer_key":"owner","reviewed_at":"2026-08-25T12:00:00Z","rationale":"Synthetic scope declaration."}]);
            app["mapping_collections"] = json!(["mapping-0.json"]);
        }
        std::fs::write(output, bytes(&app)).unwrap();
        let mut manifest = self.manifest();
        manifest["applicability_manifest"] =
            json!(if nested { "nested/applicability.json" } else { "applicability.json" });
        self.set_manifest(&manifest);
        self.refresh();
    }

    /// Produce both framework Mapping roles through the maintained genuine Mapping producer.
    fn add_mappings(&mut self) {
        std::fs::write(
            self.base().join("policy.json"),
            bytes(&catalog(POLICY, "1.0.0", &[("policy-1", "Policy")])),
        )
        .unwrap();
        for (index, target_framework) in [(0, true), (1, false)] {
            let framework = json!({"type":"catalog","artifact":"old.json","href":"old.json"});
            let policy = json!({"type":"catalog","artifact":"policy.json","href":"private-policy-endpoint.json"});
            let (source, target) =
                if target_framework { (policy, framework) } else { (framework, policy) };
            let maps:Vec<_>=["changed","removed"].into_iter().enumerate().map(|(slot,id)| {
                let framework=json!({"type":"control","id_ref":id});
                let policy=json!({"type":"control","id_ref":"policy-1"});
                let (sources,targets)=if target_framework { (vec![policy],vec![framework]) } else { (vec![framework],vec![policy]) };
                json!({"key":format!("native-{index}-{slot}"),"relationship":"intersects-with","sources":sources,"targets":targets,"reviewer_key":"reviewer","reviewed_at":"2026-08-25T12:00:00Z","rationale":"Synthetic native relationship."})
            }).collect();
            let declaration = bytes(
                &json!({"schema_version":"forge.mapping-manifest/1","collection":{"key":format!("native-{index}"),"title":"Synthetic Mapping","version":"1.0.0","last_modified":"2026-08-25T12:00:00Z"},"reviewers":[{"key":"reviewer","type":"person","name":"Synthetic Reviewer"}],"provenance":{"method":"human","matching_rationale":"semantic","status":"complete","mapping_description":"Synthetic data; no project approval.","reviewer_keys":["reviewer"],"reviewed_at":"2026-08-25T12:00:00Z"},"mapping":{"key":format!("native-{index}"),"scope":"control-only","source":source,"target":target,"maps":maps}}),
            );
            let producer = self.base().join("mapping-build.json");
            std::fs::write(&producer, declaration).unwrap();
            let prepared = crate::mapping::prepare(&producer, None, false)
                .expect("actual maintained Mapping producer");
            std::fs::write(
                self.base().join(format!("mapping-{index}.json")),
                prepared.artifact_json,
            )
            .unwrap();
        }
        let mut manifest = self.manifest();
        manifest["mapping_collections"] = json!([{"artifact":"mapping-0.json","framework_role":"target"},{"artifact":"mapping-1.json","framework_role":"source"}]);
        self.set_manifest(&manifest);
        std::fs::remove_file(self.base().join("policy.json")).unwrap();
        std::fs::remove_file(self.base().join("mapping-build.json")).unwrap();
        self.refresh();
    }
}

/// Compare complete private native output and all fifteen summary cells, then real held leases.
fn assert_full(
    fixture: &Fixture,
    pending: &PendingImpactCohort,
    capture: ReviewCapture,
    purposes: &[ImpactSourcePurpose],
    distinct_sources: usize,
) {
    assert_eq!(
        serde_json::to_value(&pending.facts().report).unwrap(),
        serde_json::to_value(&fixture.expected).unwrap()
    );
    assert_eq!(pending.facts().report.summary.rows(), fixture.expected.summary.rows());
    assert_eq!(pending.facts().report.filtered_out_findings.len(), 0);
    assert!(pending.members().iter().map(ImpactMember::purpose).eq(purposes.iter().copied()));
    for member in pending.members() {
        assert_eq!(capture.path(member.index()).unwrap(), member.lease().path());
    }
    let held = capture.finish();
    assert_eq!(held.source_original_count(), distinct_sources);
    let (index, lease) = pending.locator();
    assert!(held.same_original(index, lease).unwrap());
    for member in pending.members() {
        assert!(held.same_original(member.index(), member.lease()).unwrap());
        assert_eq!(member.bytes(), held.bytes(member.index()).unwrap());
    }
}

#[test]
/// Capture all Catalog rows/private tuple data and real originals against the full native producer.
fn complete_catalog_roster_and_private_native_report_are_genuine() {
    let fixture = Fixture::basic();
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let mut capture = fixture.capture(&mut ledger, &mut control);
    let pending =
        prepare(&mut capture, &mut ledger, &mut control).expect("complete genuine reader");
    let locator = pending.locator_fields();
    assert_eq!(pending.locator.schema_version.as_str(), LOCATOR_SCHEMA);
    assert_eq!(pending.locator.manifest_path.as_path(), Path::new("impact/impact.json"));
    assert_eq!(pending.locator.report_path.as_path(), Path::new("impact/current-report.json"));
    assert_eq!(locator.old_framework, "old-framework");
    assert_eq!(locator.new_framework, "new-framework");
    assert_eq!((locator.old_resolved, locator.new_resolved), (None, None));
    assert!(matches!(capture.role(pending.members()[2].index()).unwrap(), CaptureRole::Catalog));
    assert_eq!(pending.facts().report.new.document_version, "2.0.0 private-name");
    assert_full(
        &fixture,
        &pending,
        capture,
        &[
            ImpactSourcePurpose::Manifest,
            ImpactSourcePurpose::CurrentReport,
            ImpactSourcePurpose::OldCatalog,
            ImpactSourcePurpose::NewCatalog,
        ],
        4,
    );
}

#[test]
/// Capture both explicit Profile companions while leaving imported hrefs physically absent.
fn profile_companions_are_complete_and_imports_remain_unopened() {
    let mut fixture = Fixture::basic();
    fixture.profiles();
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let mut capture = fixture.capture(&mut ledger, &mut control);
    let pending =
        prepare(&mut capture, &mut ledger, &mut control).expect("genuine Profile closure");
    let locator = pending.locator_fields();
    assert_eq!(
        (locator.old_resolved, locator.new_resolved),
        (Some("old-resolved"), Some("new-resolved"))
    );
    assert!(pending.facts().report.old.resolved_catalog_sha256.is_some());
    assert!(!fixture.base().join("unopened-import.json").exists());
    assert_full(
        &fixture,
        &pending,
        capture,
        &[
            ImpactSourcePurpose::Manifest,
            ImpactSourcePurpose::CurrentReport,
            ImpactSourcePurpose::OldProfile,
            ImpactSourcePurpose::OldResolvedCatalog,
            ImpactSourcePurpose::NewProfile,
            ImpactSourcePurpose::NewResolvedCatalog,
        ],
        6,
    );
}

#[test]
/// Require both explicit nullable fields and native-kind agreement, never inferred companion keys.
fn locator_nullable_companion_keys_cannot_be_omitted_or_inferred() {
    for field in ["old_resolved_source_key", "new_resolved_source_key"] {
        let fixture = Fixture::basic();
        let mut locator = fixture.locator();
        locator.as_object_mut().unwrap().remove(field);
        fixture.set_locator(&locator);
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        let mut capture = fixture.capture(&mut ledger, &mut control);
        assert!(matches!(
            prepare(&mut capture, &mut ledger, &mut control),
            Err(ContractError::Invalid)
        ));
        assert_eq!(capture.finish().source_original_count(), 0);
    }
    for profile in [false, true] {
        let mut fixture = Fixture::basic();
        if profile {
            fixture.profiles();
        }
        let mut locator = fixture.locator();
        locator["old_resolved_source_key"] =
            if profile { Value::Null } else { json!("unclaimed-companion") };
        fixture.set_locator(&locator);
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        let mut capture = fixture.capture(&mut ledger, &mut control);
        assert!(matches!(
            prepare(&mut capture, &mut ledger, &mut control),
            Err(ContractError::Binding)
        ));
        assert_eq!(capture.finish().source_original_count(), 1);
    }
}

#[test]
/// Discover a real scaffold's own nested framework base and complete native report without label IO.
fn nested_applicability_uses_actual_manifest_parent_and_inventory() {
    let mut fixture = Fixture::basic();
    fixture.add_applicability(true, false);
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let mut capture = fixture.capture(&mut ledger, &mut control);
    let pending = prepare(&mut capture, &mut ledger, &mut control)
        .expect("native nested applicability closure");
    assert_eq!(
        capture.path(pending.members()[5].index()).unwrap(),
        Path::new("impact/nested/baseline.json")
    );
    assert!(!fixture.base().join("nested/private-framework-label.json").exists());
    assert_eq!(
        pending.facts().applicability_manifest.as_ref().unwrap().framework.artifact,
        Path::new("baseline.json")
    );
    assert_full(
        &fixture,
        &pending,
        capture,
        &[
            ImpactSourcePurpose::Manifest,
            ImpactSourcePurpose::CurrentReport,
            ImpactSourcePurpose::OldCatalog,
            ImpactSourcePurpose::NewCatalog,
            ImpactSourcePurpose::ApplicabilityManifest,
            ImpactSourcePurpose::ApplicabilityCatalog,
        ],
        6,
    );
}

#[test]
/// Preserve both genuine Mapping roles and repeated compatible Source occurrences, without href reads.
fn native_mapping_and_applicability_repeats_share_real_source_originals() {
    let mut fixture = Fixture::basic();
    fixture.add_mappings();
    fixture.add_applicability(false, true);
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let mut capture = fixture.capture(&mut ledger, &mut control);
    let pending = prepare(&mut capture, &mut ledger, &mut control)
        .expect("complete native roles and applicability");
    assert_eq!(pending.members()[2].index(), pending.members()[7].index());
    assert_eq!(pending.members()[4].index(), pending.members()[8].index());
    assert!(!fixture.base().join("private-policy-endpoint.json").exists());
    assert!(!fixture.base().join("policy.json").exists());
    assert!(!fixture.base().join("mapping-build.json").exists());
    assert_full(
        &fixture,
        &pending,
        capture,
        &[
            ImpactSourcePurpose::Manifest,
            ImpactSourcePurpose::CurrentReport,
            ImpactSourcePurpose::OldCatalog,
            ImpactSourcePurpose::NewCatalog,
            ImpactSourcePurpose::MappingCollection,
            ImpactSourcePurpose::MappingCollection,
            ImpactSourcePurpose::ApplicabilityManifest,
            ImpactSourcePurpose::ApplicabilityCatalog,
            ImpactSourcePurpose::ApplicabilityMappingCollection,
        ],
        7,
    );
}

#[test]
/// Capture actual successor/split/merge declarations and preserve the complete native migration result.
fn genuine_successor_split_merge_original_and_report_are_preserved() {
    let mut fixture = Fixture::basic();
    let old = bytes(&catalog(
        FRAMEWORK,
        "1.0.0",
        &[("successor-old", "A"), ("split-old", "B"), ("merge-old-a", "C"), ("merge-old-b", "D")],
    ));
    let new = bytes(&catalog(
        FRAMEWORK,
        "2.0.0",
        &[("successor-new", "A"), ("split-new-a", "B"), ("split-new-b", "B"), ("merge-new", "CD")],
    ));
    std::fs::write(fixture.base().join("old.json"), &old).unwrap();
    std::fs::write(fixture.base().join("new.json"), &new).unwrap();
    std::fs::write(fixture.base().join("successor.json"),bytes(&json!({"schema_version":"forge.successor-map/1","relationships":[{"relationship":"successor","old_ids":["successor-old"],"new_ids":["successor-new"],"approved_by":"Synthetic Reviewer","approved_at":"2026-08-25T12:00:00Z","rationale":"Synthetic relation."},{"relationship":"split","old_ids":["split-old"],"new_ids":["split-new-b","split-new-a"],"approved_by":"Synthetic Reviewer","approved_at":"2026-08-25T12:00:00Z","rationale":"Synthetic relation."},{"relationship":"merge","old_ids":["merge-old-b","merge-old-a"],"new_ids":["merge-new"],"approved_by":"Synthetic Reviewer","approved_at":"2026-08-25T12:00:00Z","rationale":"Synthetic relation."}]}))).unwrap();
    fixture.set_manifest(&json!({"schema_version":"forge.framework-impact/1","old":resource("old.json",&old,"1.0.0"),"new":resource("new.json",&new,"2.0.0"),"mapping_collections":[],"successor_map":"successor.json"}));
    fixture.refresh();
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let mut capture = fixture.capture(&mut ledger, &mut control);
    let pending =
        prepare(&mut capture, &mut ledger, &mut control).expect("actual native migration");
    assert_eq!(pending.facts().report.summary.identity_migrated, 3);
    assert!(pending.facts().report.changes.iter().all(|row| row.migration.is_some()));
    assert_full(
        &fixture,
        &pending,
        capture,
        &[
            ImpactSourcePurpose::Manifest,
            ImpactSourcePurpose::CurrentReport,
            ImpactSourcePurpose::OldCatalog,
            ImpactSourcePurpose::NewCatalog,
            ImpactSourcePurpose::SuccessorMap,
        ],
        5,
    );
}

#[test]
/// Capture a native-produced prior report and actual hash-bound declared disposition as a paired roster.
fn genuine_prior_report_and_native_dispositions_are_complete() {
    let mut fixture = Fixture::basic();
    let prior = serde_json::to_vec_pretty(&fixture.expected).unwrap();
    let finding = fixture.expected.findings.first().expect("genuine finding").finding_id.clone();
    std::fs::write(fixture.base().join("prior.json"), &prior).unwrap();
    std::fs::write(fixture.base().join("dispositions.json"),bytes(&json!({"schema_version":"forge.framework-impact-dispositions/1","prior_report_sha256":sha256_hex(&prior),"dispositions":[{"finding_id":finding,"status":"resolved","decided_by":"Synthetic Reviewer","decided_at":"2026-08-25T12:00:00Z","rationale":"Synthetic historical assertion."}]}))).unwrap();
    let mut manifest = fixture.manifest();
    manifest["prior_report"] = json!("prior.json");
    manifest["disposition_file"] = json!("dispositions.json");
    fixture.set_manifest(&manifest);
    fixture.refresh();
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let mut capture = fixture.capture(&mut ledger, &mut control);
    let pending = prepare(&mut capture, &mut ledger, &mut control).expect("genuine history pair");
    assert_eq!(
        pending.facts().report.summary.dispositioned_resolved,
        fixture.expected.summary.dispositioned_resolved
    );
    assert_eq!(pending.facts().report.summary.dispositioned_resolved, 1);
    assert_full(
        &fixture,
        &pending,
        capture,
        &[
            ImpactSourcePurpose::Manifest,
            ImpactSourcePurpose::CurrentReport,
            ImpactSourcePurpose::OldCatalog,
            ImpactSourcePurpose::NewCatalog,
            ImpactSourcePurpose::PriorReport,
            ImpactSourcePurpose::Dispositions,
        ],
        6,
    );
}

#[test]
/// Refuse missing/unknown/count/private-tuple/ordered native content differences in the stored oracle.
fn full_stored_report_differences_issue_no_pending_cohort() {
    for mutation in 0..5 {
        let fixture = Fixture::basic();
        let mut stored = serde_json::to_value(&fixture.expected).unwrap();
        match mutation {
            0 => {
                stored.as_object_mut().unwrap().remove("status");
            }
            1 => {
                stored["unknown_private_field"] = json!("must-refuse");
            }
            2 => {
                stored["summary"]["content_changed"] = json!(999);
            }
            3 => {
                stored["new"]["document_version"] = json!("private-text-changed");
            }
            _ => {
                stored["changes"].as_array_mut().unwrap().reverse();
            }
        }
        std::fs::write(fixture.base().join("current-report.json"), bytes(&stored)).unwrap();
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        let mut capture = fixture.capture(&mut ledger, &mut control);
        assert!(matches!(
            prepare(&mut capture, &mut ledger, &mut control),
            Err(ContractError::Binding)
        ));
        assert_eq!(capture.finish().source_original_count(), 4);
    }
}

#[test]
/// Recheck each actual locator/report/native generation after complete preparation, with no publication.
fn actual_original_generation_changes_refuse_the_final_input_fence() {
    for path in ["impact-locator.json", "impact/current-report.json", "impact/old.json"] {
        let fixture = Fixture::basic();
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        let mut capture = fixture.capture(&mut ledger, &mut control);
        let pending =
            prepare(&mut capture, &mut ledger, &mut control).expect("genuine retained reader");
        let held = capture.finish();
        let actual = fixture.directory.path().join(path);
        let mut raw = std::fs::read(&actual).unwrap();
        raw.push(b'\n');
        std::fs::write(actual, raw).unwrap();
        assert!(matches!(
            held.verify_inputs(&mut ledger, &mut control),
            Err(ContractError::Binding)
        ));
        assert_eq!(pending.members().len(), 4);
        assert!(!fixture.directory.path().join("review-result.json").exists());
    }
}

#[test]
/// Observe real native success/Domain boundaries and replay the actual final caller interruption.
fn original_control_final_fences_preserve_success_and_native_failure_stops() {
    for malformed in [false, true] {
        let fixture = Fixture::basic();
        if malformed {
            std::fs::write(fixture.base().join("impact.json"), b"{").unwrap();
        }
        let mut observed = Recorder::default();
        let mut ledger = ContractLedger::default();
        let mut capture = fixture.capture(&mut ledger, &mut observed);
        let result = prepare(&mut capture, &mut ledger, &mut observed);
        if malformed {
            assert!(matches!(result, Err(ContractError::Binding)));
        } else {
            assert!(result.is_ok());
        }
        let visit =
            observed.events.iter().filter(|(stage, _)| *stage == Stage::PrepareDomain).count();
        assert!(visit > 0);
        let mut stopped = Recorder::at(Stage::PrepareDomain, visit);
        let mut ledger = ContractLedger::default();
        let mut capture = fixture.capture(&mut ledger, &mut stopped);
        assert!(matches!(
            prepare(&mut capture, &mut ledger, &mut stopped),
            Err(ContractError::Interrupted(Interruption::CancelRequested))
        ));
        assert!(matches!(
            ledger.bound::<()>(|_| Err(ContractError::Binding)),
            Err(ContractError::Interrupted(Interruption::CancelRequested))
        ));
        assert!(!fixture.directory.path().join("review-result.json").exists());
    }
}

#[test]
/// Complete native and Impact reads consume one actual 100-attempt Source pool before IO.
fn complete_source_pool_capacity_refuses_before_a_missing_report_read() {
    let fixture = Fixture::basic();
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let mut capture = fixture.capture(&mut ledger, &mut control);
    for _ in 0..99 {
        capture
            .required(
                Path::new("impact/old.json"),
                CaptureRole::Catalog,
                Pool::Source,
                10 * 1024 * 1024,
                &mut ledger,
                &mut control,
            )
            .expect("actual compatible native registration");
    }
    std::fs::remove_file(fixture.base().join("current-report.json")).unwrap();
    assert!(matches!(
        prepare(&mut capture, &mut ledger, &mut control),
        Err(ContractError::Capacity)
    ));
    let mut stopped = Recorder::at(Stage::PrepareDomain, 1);
    assert!(matches!(ledger.checkpoint(&mut stopped), Err(ContractError::Capacity)));
    let no_events: [(Stage, crate::workspace::preparation::ProgressUpdate); 0] = [];
    assert_eq!(stopped.events, no_events);
    assert_eq!(capture.finish().source_original_count(), 2);
    assert!(!fixture.directory.path().join("review-result.json").exists());
}

#[test]
/// Preserve closed role refusal and portable folded-alias refusal on actual registrations.
fn actual_incompatible_roles_and_folded_paths_are_refused() {
    for folded in [false, true] {
        let fixture = Fixture::basic();
        let mut manifest = fixture.manifest();
        manifest["old"]["artifact"] = json!(if folded { "OLD.json" } else { "impact.json" });
        fixture.set_manifest(&manifest);
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        let mut capture = fixture.capture(&mut ledger, &mut control);
        if folded {
            capture
                .required(
                    Path::new("impact/old.json"),
                    CaptureRole::Catalog,
                    Pool::Source,
                    10 * 1024 * 1024,
                    &mut ledger,
                    &mut control,
                )
                .unwrap();
        }
        let result = prepare(&mut capture, &mut ledger, &mut control);
        if folded {
            assert!(matches!(result, Err(ContractError::Invalid)));
        } else {
            assert!(matches!(result, Err(ContractError::Binding)));
        }
        assert!(!fixture.directory.path().join("review-result.json").exists());
    }
}
