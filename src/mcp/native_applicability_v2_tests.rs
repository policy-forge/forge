//! Genuine file/native controls for the bounded /2 captured Catalog applicability bridge.
//!
//! Fixture scaffolds and full reports are produced by maintained native commands. Recorded
//! reviewer/owner identities stay synthetic, and no test manufactures capture/current proof.

use super::super::super::native_applicability_v2;
use super::*;
use crate::applicability::model::{GapClassification, ReportFilters};

/// Complete ordinary App originals layered onto the valid six-tuple native fixture.
struct AppFixture {
    /// Actual root lifetimes, native files and complete fixed config declarations.
    base: Fixture,
    /// Actual maintained source scaffold, then explicit human decision fixture edits.
    manifest: Value,
    /// Include the maintained complete saved report and its own independent lifecycle.
    report: bool,
    /// A second complete same-byte manifest retains a distinct actual native original.
    duplicate: bool,
}

impl AppFixture {
    /// Produce a real complete inventory snapshot, explicit decisions and current source chain.
    fn new() -> Self {
        let base = Fixture::new();
        let root = base.project.path();
        crate::applicability::execute_init(
            &root.join("catalog.json"),
            None,
            Some(&root.join("app.json")),
        )
        .expect("maintained native inventory scaffold");
        let mut manifest: Value =
            serde_json::from_slice(&fs::read(root.join("app.json")).unwrap()).unwrap();
        manifest["reviewers"] =
            json!([{"key":"app-reviewer","type":"person","name":"Private App reviewer"}]);
        manifest["decisions"] = json!([
            {"control_id":"c-parent","state":"applicable","reviewer_key":"app-reviewer",
                "reviewed_at":"2026-09-01T08:00:00Z","rationale":"Private applicable rationale"},
            {"control_id":"c-child","state":"not-applicable","reviewer_key":"app-reviewer",
                "reviewed_at":"2026-09-01T08:00:00Z","rationale":"Private excluded rationale"},
            {"control_id":"c-nested","state":"deferred","reviewer_key":"app-reviewer",
                "reviewed_at":"2026-09-01T08:00:00Z","rationale":"Private deferral",
                "revisit_date":"2026-12-01"}
        ]);
        let mut fixture = Self { base, manifest, report: false, duplicate: false };
        fixture.write_manifest(true);
        fixture.bind();
        fixture
    }

    /// Write actual source bytes and a full intrinsically validated independent approval chain.
    fn write_manifest(&self, approved: bool) {
        let root = self.base.project.path();
        fs::write(root.join("app.json"), encoded(&self.manifest)).unwrap();
        self.write_source_record("app.json", "app-life.json", approved);
    }

    /// Recompute the real recorded source and complete generated fingerprints before event IDs.
    fn write_source_record(&self, source: &str, path: &str, approved: bool) {
        let root = self.base.project.path();
        let mut actual = lifecycle(root, false);
        actual.policy.policy_key = format!("domain-{source}");
        actual.policy.source.path = source.into();
        actual.policy.source.sha256 = sha256_hex(&fs::read(root.join(source)).unwrap());
        if approved {
            let fingerprints = FingerprintSet {
                source_sha256: actual.policy.source.sha256.clone(),
                generated_artifacts: actual
                    .policy
                    .generated_artifacts
                    .iter()
                    .map(|row| NamedHash { path: row.path.clone(), sha256: row.sha256.clone() })
                    .collect(),
            };
            event(&mut actual, LifecycleState::InReview, &fingerprints);
            event(&mut actual, LifecycleState::Approved, &fingerprints);
        }
        record::validate(&actual).expect("full maintained domain source/lifecycle validation");
        fs::write(root.join(path), serde_json::to_vec(&actual).unwrap()).unwrap();
    }

    /// Generate the complete maintained saved report; no private `PreparedAnalysis` is constructed.
    fn write_report(&mut self) {
        let root = self.base.project.path();
        let actual = crate::applicability::prepare_analysis(
            &root.join("app.json"),
            ReportFilters::default(),
        )
        .expect("genuine maintained complete App analysis");
        fs::write(root.join("report.json"), serde_json::to_vec(&actual.report).unwrap()).unwrap();
        self.write_source_record("report.json", "report-life.json", true);
        self.report = true;
    }

    /// Repin the full actual declaration/config/intent roster without renewing any runtime owner.
    fn bind(&mut self) {
        let keys = ["app", "app-life", "report", "report-life", "app-copy", "copy-life"];
        self.base.discovery["resources"]
            .as_array_mut()
            .unwrap()
            .retain(|row| !keys.contains(&row["key"].as_str().unwrap()));
        self.base.bind();
        let mut rows = vec![
            self.row("app", "applicability-manifest", "app.json", Some("app-life")),
            self.row("app-life", "lifecycle-record", "app-life.json", None),
        ];
        let mut visible = vec![json!("catalog"), json!("component"), json!("source"), json!("app")];
        if self.report {
            rows.push(self.row(
                "report",
                "applicability-report",
                "report.json",
                Some("report-life"),
            ));
            rows.push(self.row("report-life", "lifecycle-record", "report-life.json", None));
            visible.push(json!("report"));
        }
        if self.duplicate {
            rows.push(self.row(
                "app-copy",
                "applicability-manifest",
                "app-copy.json",
                Some("copy-life"),
            ));
            rows.push(self.row("copy-life", "lifecycle-record", "copy-life.json", None));
            visible.push(json!("app-copy"));
        }
        self.base.discovery["resources"].as_array_mut().unwrap().extend(rows);
        self.base.profile["visible_resource_keys"] = Value::Array(visible);
        let discovery = encoded(&self.base.discovery);
        self.base.profile["discovery_sha256"] = json!(sha256_hex(&discovery));
        let profile = encoded(&self.base.profile);
        self.base.profile_pin = sha256_hex(&profile);
        self.base.intent["discovery_sha256"] = json!(sha256_hex(&discovery));
        self.base.intent["profile_sha256"] = json!(self.base.profile_pin);
        self.base.intent["visibility_profile"] = self.base.profile.clone();
        self.base.intent["capture_roster"]["resources"] = self.base.discovery["resources"].clone();
        self.base.intent["policy_decisions"] =
            decision_rows(&sha256_hex(&discovery), &self.base.profile_pin);
        fs::write(self.base.project.path().join("forge.mcp.json"), discovery).unwrap();
        fs::write(self.base.project.path().join("forge.mcp.visibility.json"), profile).unwrap();
        self.base.write_intent();
    }

    /// Declare only ordinary actual byte hashes and exact independent lifecycle keys.
    fn row(&self, key: &str, role: &str, path: &str, lifecycle: Option<&str>) -> Value {
        let mut row = resource(key, role, path, None, self.base.project.path());
        row["lifecycle_key"] = json!(lifecycle);
        row
    }
}

/// Independent App source lifecycle keys qualify full native classifications without losing tuples.
#[test]
fn genuine_app_source_and_independent_native_approvals_conserve_complete_classifications() {
    let fixture = AppFixture::new();
    let mut caller = NoopControl;
    let OfflineBuildGate::Validated(capture) = fixture.base.load(&mut caller).unwrap() else {
        panic!("genuine current App baseline unavailable")
    };
    assert_eq!(capture.search_sources().len(), 6);
    let plan = native_applicability_v2::prepare_selected(&capture.sources, "app").unwrap().unwrap();
    let counts = plan.facts().counts();
    assert_eq!(
        (
            counts.total,
            counts.applicable_unmapped,
            counts.not_applicable,
            counts.deferred,
            counts.under_review,
            counts.applicable_mapped,
            counts.applicable_reviewed_no_relationship
        ),
        (4, 1, 1, 1, 1, 0, 0)
    );
    let rows = plan
        .facts()
        .control_rows()
        .map(|row| (row.control_id().to_owned(), row.classification()))
        .collect::<Vec<_>>();
    assert_eq!(
        rows,
        vec![
            ("c-child".into(), GapClassification::NotApplicable),
            ("c-nested".into(), GapClassification::Deferred),
            ("c-parent".into(), GapClassification::ApplicableUnmapped),
            ("z-last".into(), GapClassification::UnderReview)
        ]
    );
    assert_eq!(
        capture.sources.policies()[capture.sources.index("app").unwrap().unwrap()]
            .lifecycle_key
            .as_deref(),
        Some("app-life")
    );
    assert_eq!(
        capture.sources.policies()[capture.sources.index("catalog").unwrap().unwrap()]
            .lifecycle_key
            .as_deref(),
        Some("lifecycle")
    );
    capture.verify_inputs().unwrap();
}

/// Every private saved report field is compared after a genuine complete maintained baseline.
#[test]
fn genuine_saved_report_matches_full_native_then_coherent_private_mutations_refuse() {
    let mut baseline = AppFixture::new();
    baseline.write_report();
    baseline.bind();
    {
        let mut caller = NoopControl;
        let OfflineBuildGate::Validated(capture) = baseline.base.load(&mut caller).unwrap() else {
            panic!("maintained complete report baseline unavailable")
        };
        assert_eq!(capture.search_sources().len(), 6);
        assert_eq!(
            native_applicability_v2::prepare_selected(&capture.sources, "report")
                .unwrap()
                .unwrap()
                .facts()
                .counts()
                .total,
            4
        );
    }
    for case in 0..4 {
        let mut fixture = AppFixture::new();
        fixture.write_report();
        let path = fixture.base.project.path().join("report.json");
        let mut report: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        match case {
            0 => report["controls"][0]["rationale"] = json!("Changed private rationale"),
            1 => report["reviewers"][0]["name"] = json!("Changed private reviewer"),
            2 => report["filters"]["group"] = json!("g-a"),
            _ => {
                let extra = report["controls"][0].clone();
                report["controls"].as_array_mut().unwrap().push(extra);
            }
        }
        fs::write(&path, encoded(&report)).unwrap();
        fixture.write_source_record("report.json", "report-life.json", true);
        fixture.bind();
        refused(fixture.base.load(&mut NoopControl));
    }
}

/// Actual Draft, latest source drift and generated-native drift refuse the complete App family.
#[test]
fn actual_domain_source_and_generated_drift_never_use_labels_or_old_approval() {
    for case in 0..3 {
        let mut fixture = AppFixture::new();
        match case {
            0 => fixture.write_manifest(false),
            1 => {
                fixture.manifest["decisions"][0]["rationale"] = json!("Changed after approval");
                fs::write(fixture.base.project.path().join("app.json"), encoded(&fixture.manifest))
                    .unwrap();
            }
            _ => {
                fixture.base.catalog["catalog"]["metadata"]["title"] =
                    json!("Changed generated Catalog");
                fs::write(
                    fixture.base.project.path().join("catalog.json"),
                    encoded(&fixture.base.catalog),
                )
                .unwrap();
            }
        }
        fixture.bind();
        refused(fixture.base.load(&mut NoopControl));
    }
}

/// Authentic manifest inventory and decision references cannot be replaced by coherent source pins.
#[test]
fn actual_complete_inventory_snapshot_and_unknown_decision_refuse() {
    for case in 0..2 {
        let mut fixture = AppFixture::new();
        if case == 0 {
            fixture.manifest["framework"]["inventory"]["control_ids"].as_array_mut().unwrap().pop();
        } else {
            fixture.manifest["decisions"][0]["control_id"] = json!("unknown-control");
        }
        fixture.write_manifest(true);
        fixture.bind();
        refused(fixture.base.load(&mut NoopControl));
    }
}

/// A native report hash hint matching two genuinely current complete manifest originals is ambiguous.
#[test]
fn actual_distinct_complete_manifest_originals_cannot_satisfy_one_report_by_hash_hint() {
    let mut fixture = AppFixture::new();
    fixture.write_report();
    let root = fixture.base.project.path();
    fs::write(root.join("app-copy.json"), fs::read(root.join("app.json")).unwrap()).unwrap();
    fixture.write_source_record("app-copy.json", "copy-life.json", true);
    fixture.duplicate = true;
    fixture.bind();
    refused(fixture.base.load(&mut NoopControl));
}

/// Nonempty Mapping and explicit Profile domains retain genuine whole-family refusal boundaries.
#[test]
fn missing_mapping_and_profile_producers_refuse_without_discarding_declared_scope() {
    let mut mapping = AppFixture::new();
    mapping.manifest["mapping_collections"] = json!(["not-captured.json"]);
    mapping.write_manifest(true);
    mapping.bind();
    refused(mapping.base.load(&mut NoopControl));
    let mut profile = AppFixture::new();
    profile.manifest["framework"]["type"] = json!("profile");
    profile.manifest["framework"]["resolved_catalog"] = json!("catalog.json");
    profile.manifest["framework"]["resolved_catalog_attestation"] = json!(true);
    profile.manifest["framework"]["expected_resolved_catalog_sha256"] =
        profile.manifest["framework"]["expected_sha256"].clone();
    // The closed manifest is coherent, but no genuine Profile native/companion producer exists.
    crate::applicability::manifest::parse(&encoded(&profile.manifest)).unwrap();
    profile.write_manifest(true);
    profile.bind();
    refused(profile.base.load(&mut NoopControl));
}

/// All observed actual App work shares the same original final stop and capacity chronology.
#[test]
fn genuine_app_full_factory_retains_first_original_control_and_capacity_stop() {
    let fixture = AppFixture::new();
    let mut baseline = Caller::new(None, false);
    {
        let OfflineBuildGate::Validated(capture) = fixture.base.load(&mut baseline).unwrap() else {
            panic!("real App final-fence baseline")
        };
        drop(capture);
    }
    let at = baseline.stages.len();
    assert_eq!(baseline.stages.last(), Some(&Stage::RetainPrepared));
    for failed in [false, true] {
        let mut caller = Caller::new(Some(at), failed);
        let result = fixture.base.load(&mut caller);
        if failed {
            assert!(
                matches!(result, Err(WorkError::Failed(error)) if error.code == "actual-test-control")
            );
        } else {
            assert!(matches!(result, Err(WorkError::Interrupted(Interruption::CancelRequested))));
        }
    }
    let mut caller = Caller::new(None, true);
    let OfflineBuildGate::Validated(capture) = fixture.base.load(&mut caller).unwrap() else {
        panic!("real App capacity baseline")
    };
    let before = capture.with_control(|control| {
        let before = control.stages.len();
        control.at = Some(before + 2);
        before
    });
    let mut admission = capture.admission();
    assert!(
        matches!(admission.charge(100_000), Err(WorkError::Failed(error)) if error.code == "mcp-capture-capacity")
    );
    assert!(matches!(native_applicability_v2::prepare_selected(&capture.sources, "app"),
        Err(WorkError::Failed(error)) if error.code == "mcp-capture-capacity"));
    assert_eq!(capture.with_control(|control| control.stages.len()), before + 1);
}

/// Postqualification App byte mutation is rejected by the original owner without recapture.
#[test]
fn actual_app_original_remains_in_final_verification_denominator() {
    let fixture = AppFixture::new();
    let mut caller = NoopControl;
    let OfflineBuildGate::Validated(capture) = fixture.base.load(&mut caller).unwrap() else {
        panic!("genuine App held baseline")
    };
    fs::write(fixture.base.project.path().join("app.json"), b"changed actual App original bytes")
        .unwrap();
    assert!(capture.verify_inputs().is_err());
    assert_eq!(capture.search_sources().len(), 6);
}

/// An ordinary maintained unknown-decision failure still reaches the original final domain fence.
#[test]
fn actual_native_domain_failure_cannot_bypass_same_original_postphase_stop() {
    let mut fixture = AppFixture::new();
    fixture.manifest["decisions"][0]["control_id"] = json!("unknown-control");
    fixture.write_manifest(true);
    fixture.bind();
    let mut baseline = Caller::new(None, false);
    let at = {
        let builder = CaptureBuilderV2::new(
            &fixture.base.project_root(),
            &fixture.base.external_root(),
            CapturePurpose::OfflineBuild,
            &mut baseline,
        )
        .unwrap();
        let selection = decode_builder(
            &builder,
            CapturePurpose::OfflineBuild,
            &fixture.base.intent_pin,
            &fixture.base.profile_pin,
        )
        .unwrap();
        let held = capture_declared(builder, &selection).unwrap();
        let sources = prepare_sources(held, selection).unwrap();
        assert!(native_applicability_v2::prepare_selected(&sources, "app").unwrap().is_none());
        sources.with_control(|caller| {
            assert_eq!(caller.stages.last(), Some(&Stage::RetainPrepared));
            caller.stages.len()
        })
    };
    for failed in [false, true] {
        let mut caller = Caller::new(Some(at), failed);
        let builder = CaptureBuilderV2::new(
            &fixture.base.project_root(),
            &fixture.base.external_root(),
            CapturePurpose::OfflineBuild,
            &mut caller,
        )
        .unwrap();
        let selection = decode_builder(
            &builder,
            CapturePurpose::OfflineBuild,
            &fixture.base.intent_pin,
            &fixture.base.profile_pin,
        )
        .unwrap();
        let held = capture_declared(builder, &selection).unwrap();
        let sources = prepare_sources(held, selection).unwrap();
        let result = native_applicability_v2::prepare_selected(&sources, "app");
        if failed {
            assert!(
                matches!(result, Err(WorkError::Failed(error)) if error.code == "actual-test-control")
            );
        } else {
            assert!(matches!(result, Err(WorkError::Interrupted(Interruption::CancelRequested))));
        }
    }
}

/// A coherent independently Approved framework cannot replace a missing actual App generated binding.
#[test]
fn genuine_current_source_with_omitted_framework_generation_refuses_complete_domain() {
    for report in [false, true] {
        let mut fixture = AppFixture::new();
        if report {
            fixture.write_report();
        }
        let source = if report { "report.json" } else { "app.json" };
        let life = if report { "report-life.json" } else { "app-life.json" };
        let root = fixture.base.project.path();
        let mut actual = lifecycle(root, false);
        actual.policy.policy_key = format!("domain-{source}");
        actual.policy.source.path = source.into();
        actual.policy.source.sha256 = sha256_hex(&fs::read(root.join(source)).unwrap());
        actual.policy.generated_artifacts.retain(|row| row.path == "component.json");
        let fingerprints = FingerprintSet {
            source_sha256: actual.policy.source.sha256.clone(),
            generated_artifacts: actual
                .policy
                .generated_artifacts
                .iter()
                .map(|row| NamedHash { path: row.path.clone(), sha256: row.sha256.clone() })
                .collect(),
        };
        event(&mut actual, LifecycleState::InReview, &fingerprints);
        event(&mut actual, LifecycleState::Approved, &fingerprints);
        record::validate(&actual)
            .expect("complete genuine source currentness without framework generation");
        fs::write(root.join(life), serde_json::to_vec(&actual).unwrap()).unwrap();
        fixture.bind();
        refused(fixture.base.load(&mut NoopControl));
    }
}

/// Additive genuine whole-inventory borrow controls retain the complete original test prefix.
mod borrowed_inventory {
    include!("native_applicability_borrowed_v2_tests.rs");
}

/// Additive genuine ServerRead/query/worker controls preserve every prior App body.
mod server_read {
    include!("server_app_v2_tests.rs");
}
