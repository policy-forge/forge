// Genuine actual-owner controls for the additive plain complete Catalog inventory borrow.
//
// Expected reports/snapshots come from maintained native file commands. Direct plain-helper
// controls borrow only whole genuine inventories and use the same held caller ledger/control;
// their success creates no MCP source/currentness capability.

use super::super::super::super::native_work_v2 as work;
use super::*;
use crate::applicability::captured::{self, BorrowedCatalogInput, NativeInput};
use crate::mapping::inventory;
use crate::mapping::manifest::{ResourceManifest, ResourceType, SubjectType};

/// Add a genuine Statement while preserving all four Controls and six requirement tuples.
fn statement_fixture() -> AppFixture {
    let mut fixture = AppFixture::new();
    fixture.base.catalog["catalog"]["groups"][0]["groups"][1]["controls"][0]["parts"] = json!([
        {"id":"c-parent_smt","name":"statement","prose":"Private statement fingerprint prose"}
    ]);
    fixture.base.write_native(true);
    crate::applicability::execute_init(
        &fixture.base.project.path().join("catalog.json"),
        None,
        Some(&fixture.base.project.path().join("fresh-app.json")),
    )
    .unwrap();
    let scaffold: Value = serde_json::from_slice(
        &fs::read(fixture.base.project.path().join("fresh-app.json")).unwrap(),
    )
    .unwrap();
    fixture.manifest["framework"] = scaffold["framework"].clone();
    fixture.manifest["framework"]["href"] = json!("urn:private-framework:borrowed-evidence");
    fixture.write_manifest(true);
    fixture.bind();
    fixture
}

/// Capture complete real declarations/current native inventories without running App selection.
fn originals<'control, C: WorkControl + ?Sized>(
    fixture: &AppFixture,
    caller: &'control mut C,
) -> NativeSourcesV2<'control, C> {
    let builder = CaptureBuilderV2::new(
        &fixture.base.project_root(),
        &fixture.base.external_root(),
        CapturePurpose::OfflineBuild,
        caller,
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
    prepare_sources(held, selection).unwrap()
}

/// Borrow only the complete actual Catalog and original bytes held by this real scope.
fn catalog_input<'scope, C: WorkControl + ?Sized>(
    sources: &'scope NativeSourcesV2<'_, C>,
) -> BorrowedCatalogInput<'scope> {
    let index = sources.index("catalog").unwrap().unwrap();
    let Observation::Present { bytes, .. } = sources.observation(index).unwrap() else {
        panic!("actual Catalog original")
    };
    BorrowedCatalogInput {
        input: NativeInput {
            key: "catalog",
            resource_type: ResourceType::Catalog,
            bytes,
            resolved_catalog: None,
        },
        loaded: sources.inventory(index).unwrap(),
    }
}

/// Admit every actual operand before calling the non-authorizing library port in isolation.
/// This conservative test harness covers manifest strict/typed/semantic passes, metadata
/// copies, complete raw digest, both snapshot ID sets/framing and complete comparison work.
/// The production adapter supplies its separately source-reviewed exact descriptor.
fn plain_borrow<'scope, C: WorkControl + ?Sized>(
    sources: &'scope NativeSourcesV2<'_, C>,
    row: BorrowedCatalogInput<'scope>,
) -> WorkResult<AdmittedValue<captured::PreparedApplicability<'scope>>> {
    let mut admission = sources.admission();
    sources.verify_inputs()?;
    let index = sources.index("app")?.unwrap();
    let Observation::Present { bytes: raw, .. } = sources.observation(index)? else {
        panic!("actual App original")
    };
    let facts = sources.facts(index).unwrap();
    let (_, raw_logical) = work::raw_bound(raw, 10 * 1024 * 1024, &mut admission)?;
    let mut width = row.input.key.len();
    for text in [
        &row.loaded.evidence.href,
        &row.loaded.evidence.raw_sha256,
        &row.loaded.evidence.root_uuid,
        &row.loaded.evidence.document_version,
        &row.loaded.evidence.oscal_version,
    ] {
        admission.charge(1)?;
        width = work::add(width, text.len(), &mut admission)?;
    }
    let mut subjects = 0;
    for kind in [SubjectType::Control, SubjectType::Statement] {
        for id in row.loaded.inventory.ids_of_type_refs(kind) {
            admission.charge(1)?;
            subjects = work::add(subjects, 1, &mut admission)?;
            width = work::add(width, work::add(id.len(), 128, &mut admission)?, &mut admission)?;
        }
    }
    let mut amount = work::multiply(facts.operand.work, 16, &mut admission)?;
    let pairs = work::multiply(subjects, subjects, &mut admission)?;
    amount = work::add(amount, work::multiply(pairs, 8, &mut admission)?, &mut admission)?;
    let repetitions = work::add(subjects, 8, &mut admission)?;
    let string_work = work::multiply(work::byte_work(width), repetitions, &mut admission)?;
    amount = work::add(amount, string_work, &mut admission)?;
    let raw_work = work::multiply(work::byte_work(raw.len()), 4, &mut admission)?;
    amount = work::add(amount, raw_work, &mut admission)?;
    amount = work::add(amount, work::byte_work(row.input.bytes.len()), &mut admission)?;
    amount = work::add(amount, 128, &mut admission)?;
    let strings = work::multiply(width, 8, &mut admission)?;
    let slots = work::multiply(work::add(subjects, 1, &mut admission)?, 1024, &mut admission)?;
    let logical = work::add(
        work::multiply(raw_logical, 4, &mut admission)?,
        work::add(strings, slots, &mut admission)?,
        &mut admission,
    )?;
    let mut callback = sources.admission();
    let mut native_control = sources.admission();
    let result = admission.retain(logical, |admission| {
        work::run(admission, amount, Stage::PrepareDomain, |_| {
            let result = captured::prepare_borrowed_catalog(
                raw,
                "catalog",
                &[row],
                &mut |n| callback.charge(n),
                &mut native_control,
            );
            native_control.phase(result, Stage::PrepareDomain)
        })
    });
    // Retain ordinary Domain failure until genuine same-owner whole-original/postphase checks.
    let verified = sources.verify_inputs();
    admission.phase(verified, Stage::RetainPrepared)?;
    admission.phase(result, Stage::RetainPrepared)
}

/// Full file-engine report parity includes private href, Statement inventory and all classes.
#[test]
fn genuine_borrow_preserves_full_native_file_report_and_private_evidence() {
    let mut fixture = statement_fixture();
    fixture.write_report();
    fixture.bind();
    let report: Value =
        serde_json::from_slice(&fs::read(fixture.base.project.path().join("report.json")).unwrap())
            .unwrap();
    let mut caller = NoopControl;
    let OfflineBuildGate::Validated(capture) = fixture.base.load(&mut caller).unwrap() else {
        panic!("complete genuine borrowed Report baseline")
    };
    assert_eq!(capture.search_sources().len(), 6);
    let row = catalog_input(&capture.sources);
    assert_eq!(row.loaded.inventory.count(SubjectType::Statement), 1);
    assert_eq!(row.loaded.snapshot().statement_ids, vec!["c-parent_smt"]);
    assert_eq!(row.loaded.evidence.href, "catalog");
    assert_eq!(report["framework"]["href"], "urn:private-framework:borrowed-evidence");
    let plan =
        native_applicability_v2::prepare_selected(&capture.sources, "report").unwrap().unwrap();
    let counts = plan.facts().counts();
    assert_eq!(
        (
            counts.total,
            counts.applicable_unmapped,
            counts.not_applicable,
            counts.deferred,
            counts.under_review
        ),
        (4, 1, 1, 1, 1)
    );
    let actual = plan
        .facts()
        .control_rows()
        .map(|row| (row.control_id().to_owned(), row.classification()))
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        vec![
            ("c-child".into(), GapClassification::NotApplicable),
            ("c-nested".into(), GapClassification::Deferred),
            ("c-parent".into(), GapClassification::ApplicableUnmapped),
            ("z-last".into(), GapClassification::UnderReview)
        ]
    );
    assert_eq!(row.loaded.evidence.href, "catalog");
    capture.verify_inputs().unwrap();
}

/// Coherently current sources cannot omit Statement IDs or alter the full fingerprint digest.
#[test]
fn genuine_borrow_checks_complete_statement_and_fingerprint_snapshot() {
    let baseline = statement_fixture();
    let mut caller = NoopControl;
    {
        let OfflineBuildGate::Validated(capture) = baseline.base.load(&mut caller).unwrap() else {
            panic!("complete Statement snapshot baseline")
        };
        assert_eq!(
            catalog_input(&capture.sources).loaded.snapshot().statement_ids,
            vec!["c-parent_smt"]
        );
    }
    for digest in [false, true] {
        let mut fixture = statement_fixture();
        if digest {
            fixture.manifest["framework"]["inventory"]["fingerprint_digest"] =
                json!("0".repeat(64));
        } else {
            fixture.manifest["framework"]["inventory"]["statement_ids"] = json!([]);
        }
        fixture.write_manifest(true);
        fixture.bind();
        refused(fixture.base.load(&mut NoopControl));
    }
}

/// A real foreign complete inventory cannot replace the held member; refusal keeps its work.
#[test]
fn genuine_foreign_inventory_refuses_then_actual_inventory_succeeds_on_same_owner() {
    let fixture = statement_fixture();
    let mut foreign = AppFixture::new();
    foreign.base.catalog["catalog"]["uuid"] = json!("99999999-9999-4999-8999-999999999999");
    foreign.base.write_native(true);
    let resource = ResourceManifest {
        resource_type: ResourceType::Catalog,
        artifact: "catalog.json".into(),
        href: "foreign-catalog".into(),
        resolved_catalog: None,
        resolved_catalog_attestation: None,
        expected_sha256: None,
        expected_resolved_catalog_sha256: None,
        inventory: None,
    };
    let loaded =
        inventory::load(foreign.base.project.path(), "actual foreign inventory", &resource)
            .unwrap();
    let mut caller = Caller::new(None, false);
    let sources = originals(&fixture, &mut caller);
    let actual = catalog_input(&sources);
    assert_ne!(loaded.evidence.raw_sha256, actual.loaded.evidence.raw_sha256);
    let before = sources.with_control(|control| control.stages.len());
    let wrong = BorrowedCatalogInput { input: actual.input, loaded: &loaded };
    assert!(matches!(plain_borrow(&sources, wrong), Err(WorkError::Failed(error))
        if error.code == "invalid-request"));
    let after = sources.with_control(|control| control.stages.len());
    assert!(after > before);
    let valid = plain_borrow(&sources, actual).unwrap();
    assert_eq!(valid.value().counts().total, 4);
    assert_eq!(actual.loaded.snapshot().statement_ids, vec!["c-parent_smt"]);
    sources.verify_inputs().unwrap();
}

/// Borrowed plan/data lifetimes retain the actual original; postprepare mutation is not reopened.
#[test]
fn genuine_borrowed_inventory_retains_whole_native_original_final_fence() {
    let fixture = statement_fixture();
    let mut caller = NoopControl;
    let sources = originals(&fixture, &mut caller);
    let plan = native_applicability_v2::prepare_selected(&sources, "app").unwrap().unwrap();
    assert_eq!(plan.facts().counts().total, 4);
    fs::write(
        fixture.base.project.path().join("catalog.json"),
        b"changed actual native Catalog bytes",
    )
    .unwrap();
    assert!(sources.verify_inputs().is_err());
    assert!(native_applicability_v2::prepare_selected(&sources, "app").is_err());
    assert_eq!(plan.facts().counts().total, 4);
}

/// A genuine ordinary snapshot failure reaches the same final control fence; first Capacity wins.
#[test]
fn genuine_borrowed_snapshot_failure_and_first_stop_use_original_ledger() {
    let mut fixture = statement_fixture();
    fixture.manifest["framework"]["inventory"]["statement_ids"] = json!([]);
    fixture.write_manifest(true);
    fixture.bind();
    let mut baseline = Caller::new(None, false);
    let at = {
        let sources = originals(&fixture, &mut baseline);
        let result = plain_borrow(&sources, catalog_input(&sources));
        assert!(matches!(result, Err(WorkError::Failed(error)) if error.code == "invalid-request"));
        sources.with_control(|control| {
            assert_eq!(control.stages.last(), Some(&Stage::RetainPrepared));
            control.stages.len()
        })
    };
    for failed in [false, true] {
        let mut caller = Caller::new(Some(at), failed);
        let sources = originals(&fixture, &mut caller);
        let result = plain_borrow(&sources, catalog_input(&sources));
        if failed {
            assert!(
                matches!(result, Err(WorkError::Failed(error)) if error.code == "actual-test-control")
            );
        } else {
            assert!(matches!(result, Err(WorkError::Interrupted(Interruption::CancelRequested))));
        }
    }
    let mut caller = Caller::new(None, true);
    let sources = originals(&fixture, &mut caller);
    let row = catalog_input(&sources);
    let before = sources.with_control(|control| {
        let before = control.stages.len();
        control.at = Some(before + 2);
        before
    });
    let mut admission = sources.admission();
    assert!(matches!(admission.charge(100_000), Err(WorkError::Failed(error))
        if error.code == "mcp-capture-capacity"));
    assert!(matches!(plain_borrow(&sources, row), Err(WorkError::Failed(error))
        if error.code == "mcp-capture-capacity"));
    assert_eq!(sources.with_control(|control| control.stages.len()), before + 1);
}

/// Genuine status/immutable-parser controls consume this same private actual-owner fixture.
mod status_reuse {
    include!("native_status_reuse_v2_tests.rs");
}
