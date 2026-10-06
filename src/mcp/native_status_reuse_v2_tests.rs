// Genuine full-status/native-owner controls for immutable parsed lifecycle reuse.
//
// Expected reports/errors come from the maintained complete ordinary status API.
// The parsed wrapper is plain intrinsic data; every actual native scope is issued by
// the real capture factory and retains its original caller/ledger through final fences.

use super::*;
use crate::lifecycle::record::ParsedLifecycleRecord;
use crate::lifecycle::status::{self, CurrentArtifacts, StatusReport};

/// Borrow the exact intrinsically parsed member and full genuine native current tuple.
fn parsed_current<'scope, C: WorkControl + ?Sized>(
    sources: &'scope NativeSourcesV2<'_, C>,
) -> (&'scope ParsedLifecycleRecord, &'scope CurrentArtifacts, usize) {
    let index = sources.index("lifecycle").unwrap().unwrap();
    let parsed = sources.facts(index).unwrap().record.as_ref().unwrap();
    let current =
        &sources.closures.value()[index].as_ref().unwrap().value().as_ref().unwrap().current;
    (parsed, current, index)
}

/// Plain projector intake reserves the same complete status payload before native growth.
/// A deliberately mismatched tuple is ordinary test data and can create no native capability.
fn project_plain<C: WorkControl + ?Sized>(
    sources: &NativeSourcesV2<'_, C>,
    current: &CurrentArtifacts,
    as_of: Option<chrono::NaiveDate>,
) -> WorkResult<AdmittedValue<Option<StatusReport>>> {
    sources.verify_inputs()?;
    let (parsed, _, index) = parsed_current(sources);
    let facts = sources.facts(index).unwrap();
    let mut admission = sources.admission();
    let strings = work::multiply(facts.operand.strings, 8, &mut admission)?;
    let rows = work::add(facts.operand.nodes, facts.operand.fields, &mut admission)?;
    let storage = work::multiply(rows, 256, &mut admission)?;
    let logical = work::add(work::add(strings, storage, &mut admission)?, 4096, &mut admission)?;
    let result = admission
        .retain(logical, |admission| work::project_status(parsed, current, as_of, 0, admission));
    // Keep an ordinary None/error behind complete same-original native verification.
    let verified = sources.verify_inputs();
    admission.phase(verified, Stage::RetainPrepared)?;
    admission.phase(result, Stage::RetainPrepared)
}

/// Neutral and date-only full reports preserve every private field/ordered vector exactly.
#[test]
fn genuine_parsed_status_matches_complete_ordinary_reports_without_mutable_escape() {
    let fixture = statement_fixture();
    let mut caller = NoopControl;
    let sources = originals(&fixture, &mut caller);
    let (parsed, current, _) = parsed_current(&sources);
    let due = parsed.record().review.next_review_date;
    let after = due.checked_add_days(chrono::Days::new(1)).unwrap();
    for date in [None, Some(due), Some(after)] {
        let expected = status::status_from_captured(parsed.record(), current, date).unwrap();
        let actual = project_plain(&sources, current, date).unwrap();
        let actual = actual.value().as_ref().unwrap();
        assert_eq!(serde_json::to_value(actual).unwrap(), serde_json::to_value(expected).unwrap());
    }
    let mut detached = parsed.record().clone();
    detached.state = LifecycleState::Draft;
    assert!(record::validate(&detached).is_err());
    assert_eq!(parsed.record().state, LifecycleState::Approved);
    assert_eq!(
        status::status_from_parsed_captured(parsed, current, None).unwrap().state,
        LifecycleState::Approved
    );
    sources.verify_inputs().unwrap();
}

/// Actual malformed complete history/contract bytes cannot produce immutable parsed input.
#[test]
fn genuine_owned_parser_keeps_full_record_and_event_identity_refusals() {
    let fixture = statement_fixture();
    let bytes = fs::read(fixture.base.project.path().join("lifecycle.json")).unwrap();
    let valid = record::parse_owned(&bytes).unwrap();
    assert_eq!(
        serde_json::to_value(valid.record()).unwrap(),
        serde_json::to_value(record::parse(&bytes).unwrap()).unwrap()
    );
    for case in 0..3 {
        let mut value: Value = serde_json::from_slice(&bytes).unwrap();
        match case {
            0 => value["history"][0]["event_id"] = json!("0".repeat(64)),
            1 => value["state"] = json!("draft"),
            _ => value["policy"]["unexpected"] = json!("private invalid field"),
        }
        let raw = encoded(&value);
        assert!(record::parse(&raw).is_err());
        assert!(record::parse_owned(&raw).is_err());
    }
}

/// Captured hash/path/identity and date overflow refusals retain full ordinary error behavior.
#[test]
fn genuine_parsed_status_keeps_complete_captured_correspondence_error_order() {
    let fixture = statement_fixture();
    // Persist this plain Draft fixture before acquiring the real owner. It creates no
    // approval/history or declaration, and changing its review date mints no native scope.
    let path = fixture.base.project.path().join("overflow-status-life.json");
    let original = fs::read(fixture.base.project.path().join("lifecycle.json")).unwrap();
    let mut overflow = record::parse(&original).unwrap();
    overflow.review.next_review_date = chrono::NaiveDate::MAX;
    overflow.state = LifecycleState::Draft;
    overflow.history.clear();
    fs::write(&path, serde_json::to_vec(&overflow).unwrap()).unwrap();
    let overflow = record::parse_owned(&fs::read(&path).unwrap()).unwrap();
    let mut caller = NoopControl;
    let sources = originals(&fixture, &mut caller);
    let (parsed, current, _) = parsed_current(&sources);
    for case in 0..4 {
        let mut supplied = current.clone();
        match case {
            0 => supplied.fingerprints.source_sha256 = "invalid".into(),
            1 => supplied.fingerprints.generated_artifacts[0].path = "foreign/path.json".into(),
            2 => supplied.identity_changes = vec!["foreign/path.json".into()],
            _ => supplied
                .fingerprints
                .generated_artifacts
                .push(supplied.fingerprints.generated_artifacts[0].clone()),
        }
        let old = status::status_from_captured(parsed.record(), &supplied, None);
        let new = status::status_from_parsed_captured(parsed, &supplied, None);
        match (old, new) {
            (Err(old), Err(new)) => assert_eq!(old.to_string(), new.to_string()),
            _ => panic!("full native captured correspondence must refuse both paths"),
        }
        assert!(project_plain(&sources, &supplied, None).unwrap().value().is_none());
    }
    // Date-only projection of the parse-valid Draft remains plain non-authorizing data.
    let old =
        status::status_from_captured(overflow.record(), current, Some(chrono::NaiveDate::MAX));
    let new = status::status_from_parsed_captured(&overflow, current, Some(chrono::NaiveDate::MAX));
    match (old, new) {
        (Err(old), Err(new)) => {
            assert!(old.to_string().contains("due-soon date calculation overflowed"));
            assert_eq!(old.to_string(), new.to_string());
        }
        _ => panic!("complete due-soon overflow must retain native error behavior"),
    }
    sources.verify_inputs().unwrap();
}

/// Immutable parser data never substitutes for the actual source generation on reuse.
#[test]
fn genuine_parsed_record_drift_still_refuses_same_owner_domain_use() {
    let fixture = statement_fixture();
    let mut caller = NoopControl;
    let sources = originals(&fixture, &mut caller);
    let plan = native_applicability_v2::prepare_selected(&sources, "app").unwrap().unwrap();
    let path = fixture.base.project.path().join("app-life.json");
    let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["policy"]["title"] = json!("changed actual native lifecycle bytes");
    fs::write(&path, encoded(&value)).unwrap();
    assert!(sources.verify_inputs().is_err());
    assert!(native_applicability_v2::prepare_selected(&sources, "app").is_err());
    assert_eq!(plan.facts().counts().total, 4);
}

/// Ordinary native tuple refusal reaches the real final fence; first Capacity stays first.
#[test]
fn genuine_parsed_status_ordinary_failure_preserves_original_stop_and_capacity() {
    let fixture = statement_fixture();
    let mut caller = Caller::new(None, false);
    let target = {
        let sources = originals(&fixture, &mut caller);
        let (_, current, _) = parsed_current(&sources);
        let mut wrong = current.clone();
        wrong.fingerprints.source_sha256 = "invalid".into();
        assert!(project_plain(&sources, &wrong, None).unwrap().value().is_none());
        sources.with_control(|control| control.stages.len())
    };
    for failed in [false, true] {
        let mut caller = Caller::new(Some(target), failed);
        let sources = originals(&fixture, &mut caller);
        let (_, current, _) = parsed_current(&sources);
        let mut wrong = current.clone();
        wrong.fingerprints.source_sha256 = "invalid".into();
        let result = project_plain(&sources, &wrong, None);
        if failed {
            assert!(
                matches!(result, Err(WorkError::Failed(error)) if error.code == "actual-test-control")
            );
        } else {
            assert!(matches!(result, Err(WorkError::Interrupted(Interruption::CancelRequested))));
        }
    }
    let mut caller = Caller::new(None, false);
    let sources = originals(&fixture, &mut caller);
    let (_, current, _) = parsed_current(&sources);
    let mut wrong = current.clone();
    wrong.fingerprints.source_sha256 = "invalid".into();
    assert!(matches!(sources.admission().charge(100_001),
        Err(WorkError::Failed(error)) if error.code == "mcp-capture-capacity"));
    sources.with_control(|control| {
        control.at = Some(control.stages.len() + 1);
        control.failed = true;
    });
    assert!(matches!(project_plain(&sources, &wrong, None),
        Err(WorkError::Failed(error)) if error.code == "mcp-capture-capacity"));
}
