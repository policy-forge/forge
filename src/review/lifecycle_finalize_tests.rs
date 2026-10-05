// Prospective genuine native finalizer controls. This module is a private child
// of the existing binding controls, whose native fixture/helpers remain unchanged.

use std::cell::RefCell;
use std::rc::Rc;

use super::{begin, hold, queue, raw, response, Probe, State};
use super::super::Fixture;
use crate::review::capture::ReviewControl;
use crate::review::decode::{ContractError, ContractLedger};
use crate::review::{decode_v2, encode_v2, hash_v2, lifecycle_binding, lifecycle_finalize, merge_v2};
use crate::review::wire_v2::{Disposition, ItemState, RecordedCurrentness, ResponseClassification};
use crate::workspace::preparation::{Interruption, NoopControl};

/// Mutate only ordinary asserted response fixture fields, never a native/current proof.
fn decision(queue: &crate::review::wire_v2::QueueDocumentV2, queue_raw: &[u8], reject: bool) -> Vec<u8> {
    let mut value: serde_json::Value = serde_json::from_slice(&response(queue, queue_raw, false)).unwrap();
    if reject {
        value["response_id"] = serde_json::json!("30000000-0000-4000-8000-000000000003");
        value["disposition"] = serde_json::json!("reject");
        value["rationale"] = serde_json::json!("private independent dissent rationale");
    }
    serde_json::to_vec(&value).unwrap()
}

/// Both clean asserted quorum and blocking dissent come only through real native/policy producers.
#[test]
fn genuine_current_quorum_dissent_full_counts_and_finite_redaction() {
    for case in 0..3 {
        let fixture = Fixture::new(12, true, false);
        let mut ledger = ContractLedger::default();
        let mut caller = NoopControl;
        let mut control = ReviewControl::accept(&mut caller);
        let (capture, cohort) = begin(&fixture, &mut ledger, &mut control);
        let queue = queue(&cohort, &mut ledger, &mut control);
        let bytes = raw(&queue);
        let approve = decision(&queue, &bytes, false);
        let reject = decision(&queue, &bytes, true);
        let inputs = match case { 0 => vec![("approve.json", approve.as_slice())],
            1 => vec![("reject.json", reject.as_slice())],
            _ => vec![("approve.json", approve.as_slice()), ("reject.json", reject.as_slice())] };
        let owner = hold(&fixture, capture, cohort, Some(&bytes), &inputs, &mut ledger, &mut control);
        let binding = lifecycle_binding::prepare(&owner, &mut ledger, &mut control).unwrap();
        let pending = merge_v2::prepare_policy(binding.queue(), binding.responses(), "2026-10-05T00:00:02Z", &mut ledger, &mut control).unwrap();
        let current = lifecycle_finalize::finalize(&binding, &pending, &mut ledger, &mut control).unwrap();
        let document = current.document();
        let expected_state = match case { 0 => ItemState::QuorumMet, 1 => ItemState::InReview, _ => ItemState::Conflicted };
        assert_eq!(document.currentness, RecordedCurrentness::RecordedCurrent);
        assert!(document.closure_generation.is_some());
        assert_eq!(document.source_pins.len(), 14);
        assert!(document.source_pins.as_slice() == owner.source_pins());
        assert_eq!(document.queue_raw_sha256, binding.queue().raw_sha256());
        assert_eq!(document.items.len(), 1);
        let item = &document.items[0];
        assert_eq!(item.state, expected_state);
        assert_eq!(item.required_seats, 1);
        assert_eq!(item.blocking, case != 0);
        assert!(item.reason_codes.iter().any(|code| code == "captured-current-sources"));
        assert!(!item.reason_codes.iter().any(|code| code == "currentness-unverified"));
        assert_eq!(item.response_ids.len(), inputs.len());
        assert_eq!(item.dissent_ids, if case == 0 { vec![] } else { vec!["30000000-0000-4000-8000-000000000003".to_string()] });
        if case == 0 {
            assert_eq!(item.met_seats.len(), 1);
            assert!(item.unmet_seats.is_empty());
            assert_eq!(item.met_seats[0].role_key, "review");
            assert_eq!(item.met_seats[0].reviewer_key, "asserted-reviewer");
            assert_eq!(item.met_seats[0].response_id, "20000000-0000-4000-8000-000000000002");
            assert_eq!(item.met_seats[0].ordinal, 0);
        } else {
            assert!(item.met_seats.is_empty());
            assert_eq!(item.unmet_seats.len(), 1);
            assert_eq!(item.unmet_seats[0].role_key, "review");
            assert_eq!(item.unmet_seats[0].ordinal, 0);
        }
        assert_eq!(document.counts.items, 1);
        assert_eq!(usize::try_from(document.counts.response_files).unwrap(), inputs.len());
        assert_eq!(usize::try_from(document.counts.unique_responses).unwrap(), inputs.len());
        assert_eq!(document.counts.exact_duplicates, 0);
        let states = &document.counts.states;
        assert_eq!([states.unassigned, states.assigned, states.in_review, states.conflicted,
            states.changes_requested, states.quorum_met, states.expired, states.stale],
            match case { 0 => [0, 0, 0, 0, 0, 1, 0, 0], 1 => [0, 0, 1, 0, 0, 0, 0, 0], _ => [0, 0, 0, 1, 0, 0, 0, 0] });
        assert_eq!(document.responses.len(), inputs.len());
        assert_eq!(document.responses.iter().filter(|row| row.disposition == Disposition::Reject).count(), usize::from(case != 0));
        assert_eq!(owner.record().history.len(), 2);
        assert_eq!(owner.record().state, crate::lifecycle::record::LifecycleState::Approved);
        let output = encode_v2::dispositions(document, &mut ledger, &mut control).unwrap();
        assert_eq!(output.last(), Some(&b'\n'));
        let readback = decode_v2::decode_dispositions(&output, &mut ledger, &mut control).unwrap();
        assert!(readback.document() == document);
        let public = std::str::from_utf8(&output).unwrap();
        for private in ["private native title", "private native rationale", "private review evidence",
            "private independent dissent rationale", "source.bin", "generated/00.json", "native\\u0000policy"] {
            assert!(!public.contains(private));
        }
        current.verify_inputs(&mut ledger, &mut control).unwrap();
        assert!(!fixture.root.join("result.json").exists());
    }
}

/// Equal copied queue or response bytes can prepare ordinary policy but never match native storage.
#[test]
fn genuine_equal_copied_pending_queue_and_response_raw_are_refused() {
    for copy_queue in [false, true] {
        let fixture = Fixture::new(1, false, false);
        let mut ledger = ContractLedger::default();
        let mut caller = NoopControl;
        let mut control = ReviewControl::accept(&mut caller);
        let (capture, cohort) = begin(&fixture, &mut ledger, &mut control);
        let queue = queue(&cohort, &mut ledger, &mut control);
        let bytes = raw(&queue);
        let original = response(&queue, &bytes, false);
        let owner = hold(&fixture, capture, cohort, Some(&bytes), &[("response.json", &original)], &mut ledger, &mut control);
        let binding = lifecycle_binding::prepare(&owner, &mut ledger, &mut control).unwrap();
        if copy_queue {
            let copy = binding.queue().raw().to_vec();
            assert_eq!(copy.as_slice(), binding.queue().raw());
            assert_ne!(copy.as_ptr(), binding.queue().raw().as_ptr());
            let decoded = decode_v2::decode_queue(&copy, &mut ledger, &mut control).unwrap();
            let pending = merge_v2::prepare_policy(&decoded, binding.responses(), "2026-10-05T00:00:02Z", &mut ledger, &mut control).unwrap();
            assert_eq!(lifecycle_finalize::finalize(&binding, &pending, &mut ledger, &mut control).err(), Some(ContractError::Binding));
        } else {
            let copy = binding.responses()[0].raw().to_vec();
            assert_eq!(copy.as_slice(), binding.responses()[0].raw());
            assert_ne!(copy.as_ptr(), binding.responses()[0].raw().as_ptr());
            let decoded = vec![decode_v2::decode_response(&copy, &mut ledger, &mut control).unwrap()];
            let pending = merge_v2::prepare_policy(binding.queue(), &decoded, "2026-10-05T00:00:02Z", &mut ledger, &mut control).unwrap();
            assert_eq!(lifecycle_finalize::finalize(&binding, &pending, &mut ledger, &mut control).err(), Some(ContractError::Binding));
        }
        assert!(!fixture.root.join("result.json").exists());
    }
}

/// Omitted duplicate occurrences and reordered live originals cannot substitute a complete cohort.
#[test]
fn genuine_omitted_and_reordered_duplicate_cohorts_refuse() {
    for reordered in [false, true] {
        let fixture = Fixture::new(1, false, false);
        let mut ledger = ContractLedger::default();
        let mut caller = NoopControl;
        let mut control = ReviewControl::accept(&mut caller);
        let (capture, cohort) = begin(&fixture, &mut ledger, &mut control);
        let queue = queue(&cohort, &mut ledger, &mut control);
        let bytes = raw(&queue);
        let current = response(&queue, &bytes, false);
        let foreign = response(&queue, &bytes, true);
        let owner = hold(&fixture, capture, cohort, Some(&bytes),
            &[("current.json", &current), ("foreign.json", &foreign), ("current.json", &current)], &mut ledger, &mut control);
        let binding = lifecycle_binding::prepare(&owner, &mut ledger, &mut control).unwrap();
        if reordered {
            let mut reversed = Vec::new();
            for index in [1, 0, 2] {
                reversed.push(decode_v2::decode_response(binding.responses()[index].raw(), &mut ledger, &mut control).unwrap());
            }
            let pending = merge_v2::prepare_policy(binding.queue(), &reversed, "2026-10-05T00:00:02Z", &mut ledger, &mut control).unwrap();
            assert_eq!(pending.counts().response_files, 3);
            assert_eq!(pending.counts().exact_duplicates, 1);
            assert_eq!(lifecycle_finalize::finalize(&binding, &pending, &mut ledger, &mut control).err(), Some(ContractError::Binding));
        } else {
            let pending = merge_v2::prepare_policy(binding.queue(), &binding.responses()[..2], "2026-10-05T00:00:02Z", &mut ledger, &mut control).unwrap();
            assert_eq!(pending.counts().response_files, 2);
            assert_eq!(pending.counts().exact_duplicates, 0);
            assert_eq!(lifecycle_finalize::finalize(&binding, &pending, &mut ledger, &mut control).err(), Some(ContractError::Binding));
        }
        assert!(!fixture.root.join("result.json").exists());
    }
}

/// The real current generation uses every occurrence in native registration order, not unique UUIDs.
#[test]
fn genuine_generation_complete_order_duplicates_and_foreign_denominator() {
    let fixture = Fixture::new(12, true, false);
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let (capture, cohort) = begin(&fixture, &mut ledger, &mut control);
    let queue = queue(&cohort, &mut ledger, &mut control);
    let bytes = raw(&queue);
    let current_response = response(&queue, &bytes, false);
    let foreign = response(&queue, &bytes, true);
    let owner = hold(&fixture, capture, cohort, Some(&bytes),
        &[("current.json", &current_response), ("foreign.json", &foreign), ("current.json", &current_response)], &mut ledger, &mut control);
    let binding = lifecycle_binding::prepare(&owner, &mut ledger, &mut control).unwrap();
    let pending = merge_v2::prepare_policy(binding.queue(), binding.responses(), "2026-10-05T00:00:02Z", &mut ledger, &mut control).unwrap();
    let current = lifecycle_finalize::finalize(&binding, &pending, &mut ledger, &mut control).unwrap();
    let rows: Vec<_> = binding.responses().iter().map(|original| hash_v2::ResponseOriginal {
        response_id: &original.document().response_id, raw_sha256: original.raw_sha256(),
        byte_length: u64::try_from(original.raw().len()).unwrap() }).collect();
    let expected = hash_v2::closure_generation(binding.queue().raw_sha256(), u64::try_from(binding.queue().raw().len()).unwrap(),
        owner.source_pins(), &rows, &mut ledger, &mut control).unwrap();
    assert_eq!(current.document().closure_generation.as_deref(), Some(expected.as_str()));
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].raw_sha256, rows[2].raw_sha256);
    assert_eq!(rows[0].response_id, rows[2].response_id);
    let reordered = [hash_v2::ResponseOriginal { response_id: rows[1].response_id, raw_sha256: rows[1].raw_sha256, byte_length: rows[1].byte_length },
        hash_v2::ResponseOriginal { response_id: rows[0].response_id, raw_sha256: rows[0].raw_sha256, byte_length: rows[0].byte_length },
        hash_v2::ResponseOriginal { response_id: rows[2].response_id, raw_sha256: rows[2].raw_sha256, byte_length: rows[2].byte_length }];
    let changed = hash_v2::closure_generation(binding.queue().raw_sha256(), u64::try_from(binding.queue().raw().len()).unwrap(),
        owner.source_pins(), &reordered, &mut ledger, &mut control).unwrap();
    assert_ne!(changed, expected);
    let omitted = hash_v2::closure_generation(binding.queue().raw_sha256(), u64::try_from(binding.queue().raw().len()).unwrap(),
        owner.source_pins(), &rows[..2], &mut ledger, &mut control).unwrap();
    assert_ne!(omitted, expected);
    assert_eq!(current.document().counts.response_files, 3);
    assert_eq!(current.document().counts.unique_responses, 2);
    assert_eq!(current.document().counts.exact_duplicates, 1);
    assert_eq!(current.document().responses.iter().filter(|row| row.classification == ResponseClassification::Foreign).count(), 1);
    current.verify_inputs(&mut ledger, &mut control).unwrap();
    assert!(!fixture.root.join("result.json").exists());
}

/// The real finalizer's after-DTO fence catches drift after the prior binding succeeded.
#[test]
fn genuine_actual_queue_drift_before_current_finalization_refuses() {
    let fixture = Fixture::new(1, false, false);
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let (capture, cohort) = begin(&fixture, &mut ledger, &mut control);
    let queue = queue(&cohort, &mut ledger, &mut control);
    let bytes = raw(&queue);
    let original = response(&queue, &bytes, false);
    let owner = hold(&fixture, capture, cohort, Some(&bytes), &[("response.json", &original)], &mut ledger, &mut control);
    let binding = lifecycle_binding::prepare(&owner, &mut ledger, &mut control).unwrap();
    let pending = merge_v2::prepare_policy(binding.queue(), binding.responses(), "2026-10-05T00:00:02Z", &mut ledger, &mut control).unwrap();
    std::fs::write(fixture.root.join("queue.json"), b"changed actual original after binding").unwrap();
    assert_eq!(lifecycle_finalize::finalize(&binding, &pending, &mut ledger, &mut control).err(), Some(ContractError::Binding));
    assert!(!fixture.root.join("result.json").exists());
}

/// A successful owner remains held across actual finite encoding and refuses subsequent physical drift.
#[test]
fn genuine_current_owner_repeated_after_encoding_whole_input_fence() {
    let fixture = Fixture::new(1, false, false);
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let (capture, cohort) = begin(&fixture, &mut ledger, &mut control);
    let queue = queue(&cohort, &mut ledger, &mut control);
    let bytes = raw(&queue);
    let original = response(&queue, &bytes, false);
    let owner = hold(&fixture, capture, cohort, Some(&bytes), &[("response.json", &original)], &mut ledger, &mut control);
    let binding = lifecycle_binding::prepare(&owner, &mut ledger, &mut control).unwrap();
    let pending = merge_v2::prepare_policy(binding.queue(), binding.responses(), "2026-10-05T00:00:02Z", &mut ledger, &mut control).unwrap();
    let current = lifecycle_finalize::finalize(&binding, &pending, &mut ledger, &mut control).unwrap();
    let output = encode_v2::dispositions(current.document(), &mut ledger, &mut control).unwrap();
    assert_eq!(output.last(), Some(&b'\n'));
    current.verify_inputs(&mut ledger, &mut control).unwrap();
    std::fs::write(fixture.root.join("source.bin"), b"changed physical source after real output encoding").unwrap();
    assert_eq!(current.verify_inputs(&mut ledger, &mut control), Err(ContractError::Binding));
    assert!(!fixture.root.join("result.json").exists());
}

/// The final outer ordinary-failure fence remains on the same real holder/ledger/accepted controller.
#[test]
fn genuine_finalizer_ordinary_binding_postfence_stops_are_sticky() {
    for failed in [false, true] {
        let fixture = Fixture::new(1, false, false);
        let state = Rc::new(RefCell::new(State::default()));
        let mut caller = Probe(Rc::clone(&state));
        let mut control = ReviewControl::accept(&mut caller);
        let mut ledger = ContractLedger::default();
        let (capture, cohort) = begin(&fixture, &mut ledger, &mut control);
        let queue = queue(&cohort, &mut ledger, &mut control);
        let bytes = raw(&queue);
        let original = response(&queue, &bytes, false);
        let owner = hold(&fixture, capture, cohort, Some(&bytes), &[("response.json", &original)], &mut ledger, &mut control);
        let binding = lifecycle_binding::prepare(&owner, &mut ledger, &mut control).unwrap();
        let copy = binding.queue().raw().to_vec();
        let decoded = decode_v2::decode_queue(&copy, &mut ledger, &mut control).unwrap();
        let pending = merge_v2::prepare_policy(&decoded, binding.responses(), "2026-10-05T00:00:02Z", &mut ledger, &mut control).unwrap();
        let before = state.borrow().calls;
        assert_eq!(lifecycle_finalize::finalize(&binding, &pending, &mut ledger, &mut control).err(), Some(ContractError::Binding));
        let completed = state.borrow().calls;
        let delta = completed - before;
        assert!(delta > 2);
        { let mut observation = state.borrow_mut(); observation.at = Some(completed + delta); observation.failed = failed; }
        let expected = if failed { ContractError::ControlFailed } else { ContractError::Interrupted(Interruption::CancelRequested) };
        assert_eq!(lifecycle_finalize::finalize(&binding, &pending, &mut ledger, &mut control).err(), Some(expected));
        let stopped = state.borrow().calls;
        assert_eq!(stopped, completed + delta);
        assert_eq!(lifecycle_finalize::finalize(&binding, &pending, &mut ledger, &mut control).err(), Some(expected));
        assert_eq!(state.borrow().calls, stopped);
        assert!(!fixture.root.join("result.json").exists());
    }
}

/// Successful native current finalization also observes the final original callback before returning.
#[test]
fn genuine_successful_finalizer_last_checkpoint_stops_are_sticky() {
    for failed in [false, true] {
        let fixture = Fixture::new(1, false, false);
        let state = Rc::new(RefCell::new(State::default()));
        let mut caller = Probe(Rc::clone(&state));
        let mut control = ReviewControl::accept(&mut caller);
        let mut ledger = ContractLedger::default();
        let (capture, cohort) = begin(&fixture, &mut ledger, &mut control);
        let queue = queue(&cohort, &mut ledger, &mut control);
        let bytes = raw(&queue);
        let original = response(&queue, &bytes, false);
        let owner = hold(&fixture, capture, cohort, Some(&bytes), &[("response.json", &original)], &mut ledger, &mut control);
        let binding = lifecycle_binding::prepare(&owner, &mut ledger, &mut control).unwrap();
        let pending = merge_v2::prepare_policy(binding.queue(), binding.responses(), "2026-10-05T00:00:02Z", &mut ledger, &mut control).unwrap();
        let before = state.borrow().calls;
        assert_eq!(lifecycle_finalize::finalize(&binding, &pending, &mut ledger, &mut control).map(|_| ()), Ok(()));
        let completed = state.borrow().calls;
        let delta = completed - before;
        assert!(delta > 2);
        { let mut observation = state.borrow_mut(); observation.at = Some(completed + delta); observation.failed = failed; }
        let expected = if failed { ContractError::ControlFailed } else { ContractError::Interrupted(Interruption::CancelRequested) };
        assert_eq!(lifecycle_finalize::finalize(&binding, &pending, &mut ledger, &mut control).err(), Some(expected));
        let stopped = state.borrow().calls;
        assert_eq!(stopped, completed + delta);
        assert_eq!(lifecycle_finalize::finalize(&binding, &pending, &mut ledger, &mut control).err(), Some(expected));
        assert_eq!(state.borrow().calls, stopped);
        assert!(!fixture.root.join("result.json").exists());
    }
}

/// Original unchanged-cap admission failure cannot be replaced by a later finalizer caller error.
#[test]
fn genuine_finalizer_first_capacity_precedes_later_stop_and_ordinary_checks() {
    let fixture = Fixture::new(1, false, false);
    let state = Rc::new(RefCell::new(State::default()));
    let mut caller = Probe(Rc::clone(&state));
    let mut control = ReviewControl::accept(&mut caller);
    let mut ledger = ContractLedger::default();
    let (capture, cohort) = begin(&fixture, &mut ledger, &mut control);
    let queue = queue(&cohort, &mut ledger, &mut control);
    let bytes = raw(&queue);
    let original = response(&queue, &bytes, false);
    let owner = hold(&fixture, capture, cohort, Some(&bytes), &[("response.json", &original)], &mut ledger, &mut control);
    let binding = lifecycle_binding::prepare(&owner, &mut ledger, &mut control).unwrap();
    let pending = merge_v2::prepare_policy(binding.queue(), binding.responses(), "2026-10-05T00:00:02Z", &mut ledger, &mut control).unwrap();
    assert_eq!(ledger.derived(33_554_433), Err(ContractError::Capacity));
    let before = state.borrow().calls;
    { let mut observation = state.borrow_mut(); observation.at = Some(before + 1); observation.failed = true; }
    assert_eq!(lifecycle_finalize::finalize(&binding, &pending, &mut ledger, &mut control).err(), Some(ContractError::Capacity));
    assert_eq!(state.borrow().calls, before);
    assert!(!fixture.root.join("result.json").exists());
}
