//! Actual complete captured Impact/current-native union controls over real originals.
//! Queue bytes here test capture purposes only; full queue binding and publication
//! remain separate command controls. No pending object or capture proof is fabricated.

use super::*;
use crate::framework::{analysis, manifest, model::ImpactFilters};
use crate::review::applicability_capture::{self, tests::Fixture};
use crate::review::impact_capture;
use crate::workspace::preparation::NoopControl;
use serde_json::{Value, json};
use std::path::PathBuf;

/// Persist fixture declarations before any production capture begins.
fn write(root: &Path, name: &str, value: &Value) {
    std::fs::write(root.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

/// Generate a full native Impact report using the maintained file producer.
fn impact_fixture(profile: bool, nested_applicability: bool) -> Fixture {
    let fixture = Fixture::new(profile);
    let root = fixture.root();
    let raw = std::fs::read(root.join("framework.json")).unwrap();
    let value: Value = serde_json::from_slice(&raw).unwrap();
    let model = if profile { "profile" } else { "catalog" };
    let mut resource = json!({
        "type": model, "artifact": "framework.json", "expected_sha256": sha256_hex(&raw),
        "root_uuid": value[model]["uuid"], "document_version": value[model]["metadata"]["version"],
        "oscal_version": value[model]["metadata"]["oscal-version"]
    });
    if profile {
        resource["resolved_catalog"] = json!("resolved-catalog.json");
        resource["resolved_catalog_attestation"] = json!(true);
        resource["expected_resolved_catalog_sha256"] =
            json!(sha256_hex(&std::fs::read(root.join("resolved-catalog.json")).unwrap()));
    }
    let mut declaration = json!({
        "schema_version": manifest::MANIFEST_SCHEMA_VERSION,
        "old": resource, "new": resource,
        "mapping_collections": [{"artifact":"mapping.json", "framework_role":"target"}]
    });
    if nested_applicability {
        declaration["applicability_manifest"] = json!("applicability.json");
    }
    write(root, "impact-manifest.json", &declaration);
    let parsed =
        manifest::parse(&std::fs::read(root.join("impact-manifest.json")).unwrap()).unwrap();
    let (report, _) = analysis::analyze(root, &parsed, ImpactFilters::default()).unwrap();
    write(root, "impact-report.json", &serde_json::to_value(&report).unwrap());
    write(
        root,
        "impact-locator.json",
        &json!({
            "schema_version":"forge.review-queue-impact-locator/1",
            "manifest_path":"impact-manifest.json", "report_path":"impact-report.json",
            "old_framework_source_key":"framework", "new_framework_source_key":"framework",
            "old_resolved_source_key": if profile {json!("resolved")} else {Value::Null},
            "new_resolved_source_key": if profile {json!("resolved")} else {Value::Null}
        }),
    );
    // This component never decodes these Queue originals or treats them as review approval.
    write(root, "old-queue.json", &json!({"component_capture_purpose":"old"}));
    write(root, "new-queue.json", &json!({"component_capture_purpose":"new"}));
    write(
        root,
        "links.json",
        &json!({"schema_version":"forge.review-queue-links-request/1","links":[]}),
    );
    fixture
}

/// Register the actual two Queue and three Auxiliary purposes on one genuine session.
fn capture(fixture: &Fixture, ledger: &mut ContractLedger) -> ReviewCapture {
    let mut capture = ReviewCapture::new_supersession(
        fixture.root(),
        Path::new("supersession.json"),
        ledger,
        &mut NoopControl,
    )
    .unwrap();
    for (path, purpose) in [
        ("old-queue.json", QueuePurpose::HistoricalOld),
        ("new-queue.json", QueuePurpose::CurrentNew),
    ] {
        capture
            .required_supersession_queue(Path::new(path), purpose, ledger, &mut NoopControl)
            .unwrap();
    }
    for (path, purpose) in [
        ("locator.json", AuxiliaryPurpose::NewNativeLocator),
        ("impact-locator.json", AuxiliaryPurpose::ImpactLocator),
        ("links.json", AuxiliaryPurpose::Links),
    ] {
        capture
            .required_supersession_auxiliary(Path::new(path), purpose, ledger, &mut NoopControl)
            .unwrap();
    }
    capture
}

/// Obtain both opaque genuine pending cohorts without hand-building their fields.
fn prepare(
    capture: &mut ReviewCapture,
    ledger: &mut ContractLedger,
) -> (PendingNewNativeCohort, impact_capture::PendingImpactCohort) {
    let locator =
        capture.supersession_auxiliary_original(AuxiliaryPurpose::NewNativeLocator).unwrap();
    let native =
        applicability_capture::prepare(capture, locator, ledger, &mut NoopControl).unwrap();
    let impact = impact_capture::prepare(capture, ledger, &mut NoopControl).unwrap();
    (native.into_supersession_pending(), impact)
}

/// Complete Catalog/Profile plus configured applicability read sets share one owner and real leases.
#[test]
fn actual_complete_native_and_impact_union_seals_both_framework_families() {
    for profile in [false, true] {
        for nested in [false, true] {
            let fixture = impact_fixture(profile, nested);
            let mut ledger = ContractLedger::default();
            let mut captured = capture(&fixture, &mut ledger);
            let (native, impact) = prepare(&mut captured, &mut ledger);
            assert_eq!(impact.facts().report.summary.old_controls, 3);
            assert_eq!(impact.facts().report.summary.new_controls, 3);
            assert_eq!(impact.facts().report.summary.unchanged, 3);
            assert!(impact.facts().report.findings.is_empty());
            let held = Rc::new(captured.finish());
            let union = seal_complete_union(
                Rc::clone(&held),
                native,
                impact,
                &mut ledger,
                &mut NoopControl,
            )
            .unwrap();
            assert!(Rc::ptr_eq(union.held_inputs(), &held));
            assert!(matches!(union.new_native(), NewNativeCohort::Applicability(_)));
            assert_ne!(
                union.queue_original(QueuePurpose::HistoricalOld).unwrap(),
                union.queue_original(QueuePurpose::CurrentNew).unwrap()
            );
            union.verify_inputs(&mut ledger, &mut NoopControl).unwrap();
            assert!(!fixture.root().join("supersession.json").exists());
        }
    }
}

/// Genuine complete membership refuses an unrelated real captured Source original.
#[test]
fn actual_union_refuses_extra_source_outside_both_genuine_cohorts() {
    let fixture = impact_fixture(false, true);
    std::fs::write(fixture.root().join("extra.json"), b"{}").unwrap();
    let mut ledger = ContractLedger::default();
    let mut captured = capture(&fixture, &mut ledger);
    let (native, impact) = prepare(&mut captured, &mut ledger);
    captured
        .required(
            Path::new("extra.json"),
            CaptureRole::Catalog,
            Pool::Source,
            MAX_QUEUE_BYTES,
            &mut ledger,
            &mut NoopControl,
        )
        .unwrap();
    assert!(matches!(
        seal_complete_union(
            Rc::new(captured.finish()),
            native,
            impact,
            &mut ledger,
            &mut NoopControl
        ),
        Err(ContractError::Binding)
    ));
    assert!(!fixture.root().join("supersession.json").exists());
}

/// Identical raw bytes in independent actual sessions cannot substitute for real cohort leases.
#[test]
fn actual_union_refuses_byte_identical_cross_owner_cohorts() {
    let fixture = impact_fixture(true, true);
    let mut first_ledger = ContractLedger::default();
    let mut first = capture(&fixture, &mut first_ledger);
    let (first_native, first_impact) = prepare(&mut first, &mut first_ledger);
    let first_owner = Rc::new(first.finish());
    let mut second_ledger = ContractLedger::default();
    let mut second = capture(&fixture, &mut second_ledger);
    let (second_native, second_impact) = prepare(&mut second, &mut second_ledger);
    let second_owner = Rc::new(second.finish());
    assert!(!Rc::ptr_eq(&first_owner, &second_owner));
    assert!(matches!(
        seal_complete_union(
            Rc::clone(&first_owner),
            second_native,
            first_impact,
            &mut first_ledger,
            &mut NoopControl
        ),
        Err(ContractError::Binding)
    ));
    assert!(matches!(
        seal_complete_union(
            second_owner,
            first_native,
            second_impact,
            &mut second_ledger,
            &mut NoopControl
        ),
        Err(ContractError::Binding)
    ));
}

/// Full stored-report equality refuses a forged complete summary before issuing a cohort.
#[test]
fn actual_captured_reader_refuses_modified_complete_stored_report() {
    let fixture = impact_fixture(false, true);
    let root = fixture.root();
    let mut report: Value =
        serde_json::from_slice(&std::fs::read(root.join("impact-report.json")).unwrap()).unwrap();
    report["summary"]["unchanged"] = json!(2);
    write(root, "impact-report.json", &report);
    let mut ledger = ContractLedger::default();
    let mut captured = capture(&fixture, &mut ledger);
    assert!(matches!(
        impact_capture::prepare(&mut captured, &mut ledger, &mut NoopControl),
        Err(ContractError::Binding)
    ));
    assert!(!root.join("supersession.json").exists());
}

/// Every declared nested reader original is required; missing files cannot produce partial facts.
#[test]
fn actual_captured_reader_refuses_missing_nested_applicability_mapping() {
    let fixture = impact_fixture(false, true);
    let root = fixture.root();
    // A separate original is declared only by the configured applicability reader.
    // Keep the top-level Impact Mapping file present to reach that nested read.
    std::fs::copy(root.join("mapping.json"), root.join("nested-mapping.json")).unwrap();
    let mut applicability: Value =
        serde_json::from_slice(&std::fs::read(root.join("applicability.json")).unwrap()).unwrap();
    applicability["mapping_collections"] = json!(["nested-mapping.json"]);
    write(root, "applicability.json", &applicability);
    let parsed =
        manifest::parse(&std::fs::read(root.join("impact-manifest.json")).unwrap()).unwrap();
    let (report, _) = analysis::analyze(root, &parsed, ImpactFilters::default()).unwrap();
    write(root, "impact-report.json", &serde_json::to_value(&report).unwrap());
    std::fs::remove_file(root.join("nested-mapping.json")).unwrap();
    assert!(root.join("mapping.json").is_file());
    let mut ledger = ContractLedger::default();
    let mut captured = capture(&fixture, &mut ledger);
    assert!(impact_capture::prepare(&mut captured, &mut ledger, &mut NoopControl).is_err());
    assert!(!fixture.root().join("supersession.json").exists());
}

/// A later actual input mutation refuses the full final original fence after genuine preparation.
#[test]
fn actual_complete_union_fence_refuses_impact_report_drift() {
    let fixture = impact_fixture(true, true);
    let mut ledger = ContractLedger::default();
    let mut captured = capture(&fixture, &mut ledger);
    let (native, impact) = prepare(&mut captured, &mut ledger);
    let union = seal_complete_union(
        Rc::new(captured.finish()),
        native,
        impact,
        &mut ledger,
        &mut NoopControl,
    )
    .unwrap();
    let path: PathBuf = fixture.root().join("impact-report.json");
    let mut raw = std::fs::read(&path).unwrap();
    raw.push(b' ');
    std::fs::write(path, raw).unwrap();
    assert!(matches!(
        union.verify_inputs(&mut ledger, &mut NoopControl),
        Err(ContractError::Binding)
    ));
    assert!(!fixture.root().join("supersession.json").exists());
}
