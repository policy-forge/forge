// Genuine prospective binding controls nested under the real receiver fixture's
// private cfg(test) module. No detached capture, pending, pin or native-proof factory.

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use serde_json::json;

use super::Fixture;
use crate::evidence_capture::CaptureRole;
use crate::review::capture::{Pool, ReviewCapture, ReviewControl};
use crate::review::decode::{ContractError, ContractLedger};
use crate::review::{hash_v2, lifecycle_binding, lifecycle_capture, merge_v2};
use crate::review::lifecycle_capture::{ApprovedLifecycleClosure, PendingLifecycleCohort};
use crate::review::wire::SeatRequirement;
use crate::review::wire_v2::*;
use crate::workspace::contract::Error;
use crate::workspace::preparation::{Interruption, NoopControl, ProgressUpdate, Stage, WorkControl, WorkError, WorkResult};

/// Enter one actual mode and complete native preparation on the operation's caller.
fn begin(fixture: &Fixture, ledger: &mut ContractLedger, control: &mut dyn WorkControl)
    -> (ReviewCapture, PendingLifecycleCohort) {
    let mut capture = ReviewCapture::new_lifecycle(&fixture.root, &[Path::new("result.json")], ledger, control).unwrap();
    let pending = lifecycle_capture::read_pending(&mut capture, Path::new("locator.json"), ledger, control).unwrap();
    (capture, pending)
}

/// Ordinary requested review declarations derive expected digests from genuine pending data.
/// Only later actual original registration plus `finish_and_seal` issues the native owner.
fn queue(pending: &PendingLifecycleCohort, ledger: &mut ContractLedger, control: &mut dyn WorkControl)
    -> QueueDocumentV2 {
    let pins = serde_json::from_value(serde_json::to_value(&pending.pins).unwrap()).unwrap();
    let roster: Vec<_> = pending.members.iter().map(|member| hash_v2::RosterEntry {
        purpose: member.pin.kind, route: member.declared_path.as_deref(), pin: &member.pin,
    }).collect();
    let sources = hash_v2::sources(&roster, ledger, control).unwrap();
    let native = &pending.record.policy;
    let subject_id = hash_v2::subject_id(&native.policy_key, &native.version_key, ledger, control).unwrap();
    let subject_sha256 = hash_v2::subject(&native.policy_key, &native.version_key, &sources, ledger, control).unwrap();
    let source_keys = pending.pins.iter().map(|pin| pin.artifact_key.clone()).collect();
    let mut document = QueueDocumentV2 {
        schema_version: "forge.review-queue/2".to_string(), identity_disclaimer: IDENTITY_DISCLAIMER.to_string(),
        sensitivity: Sensitivity::IdsAndHashes, queue_id: "10000000-0000-4000-8000-000000000001".to_string(),
        created_at: "2026-10-05T00:00:00Z".to_string(), source_pins: pins,
        roles: vec![RoleDefinition { key: "review".to_string() }],
        reviewers: vec![Reviewer { key: "asserted-author".to_string(), role_keys: vec!["review".to_string()] },
            Reviewer { key: "asserted-reviewer".to_string(), role_keys: vec!["review".to_string()] }],
        policies: vec![ReviewPolicy { key: "policy-1".to_string(),
            seats: vec![SeatRequirement { role_key: "review".to_string(), count: 1 }], substitutions: vec![],
            abstention_rule: AbstentionRule::Nonapproving, empty_abstention_reasons: vec!["absent".to_string()],
            author_separation: AuthorSeparation::DeclaredKeys }],
        items: vec![ReviewItemV2 { key: "item-1".to_string(), item_id: String::new(),
            domain: DomainV2::LifecyclePolicyVersion, adapter_version: ADAPTER.to_string(), subject_id,
            requested_action: RequestedAction::ReReview, source_keys, subject_sha256,
            context: ContextSnapshot { reason_codes: vec!["lifecycle-approved-current".to_string()], related_subject_ids: vec![] },
            context_sha256: String::new(), policy_key: "policy-1".to_string(), policy_sha256: String::new(),
            author_keys: vec!["asserted-author".to_string()],
            assignments: vec![Assignment { reviewer_key: "asserted-reviewer".to_string(), role_key: "review".to_string() }],
            due_at: None, allowed_dispositions: vec![Disposition::Approve, Disposition::Reject, Disposition::RequestChanges,
                Disposition::Abstain, Disposition::Superseded] }],
    };
    refresh(&mut document, ledger, control);
    document
}

/// Recompute all ordinary self-correlations so a native mismatch reaches the binding leaf.
fn refresh(queue: &mut QueueDocumentV2, ledger: &mut ContractLedger, control: &mut dyn WorkControl) {
    let context = hash_v2::context(queue, &queue.items[0], ledger, control).unwrap();
    queue.items[0].context_sha256 = context;
    let policy = hash_v2::policy(queue, &queue.policies[0], &queue.items[0], ledger, control).unwrap();
    queue.items[0].policy_sha256 = policy;
    let item = hash_v2::item_id(queue, &queue.items[0], ledger, control).unwrap();
    queue.items[0].item_id = item;
}

/// Only test input bytes are serialized here; actual production strict decode consumes held raw.
fn raw(queue: &QueueDocumentV2) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(queue).unwrap();
    bytes.push(b'\n');
    bytes
}

/// Declare ordinary response evidence with complete pins, including explicit required nulls.
fn response(queue: &QueueDocumentV2, queue_raw: &[u8], foreign: bool) -> Vec<u8> {
    let item = &queue.items[0];
    serde_json::to_vec(&json!({"schema_version":"forge.review-response/2", "identity_disclaimer":IDENTITY_DISCLAIMER,
        "response_id": if foreign {"30000000-0000-4000-8000-000000000003"} else {"20000000-0000-4000-8000-000000000002"},
        "queue_id": if foreign {"40000000-0000-4000-8000-000000000004"} else {&queue.queue_id},
        "queue_raw_sha256":crate::hashing::sha256_hex(queue_raw), "item_key":item.key, "item_id":item.item_id,
        "domain":"lifecycle-policy-version", "adapter_version":ADAPTER, "requested_action":"re-review",
        "source_pins":queue.source_pins, "subject_sha256":item.subject_sha256, "context_sha256":item.context_sha256,
        "policy_sha256":item.policy_sha256, "reviewer_key":"asserted-reviewer", "reviewer_role":"review", "disposition":"approve",
        "responded_at":"2026-10-05T00:00:01Z", "rationale":"private review evidence", "abstention_reason":null,
        "proposed_edit":null, "supersedes":null})).unwrap()
}

/// Register real operation originals in their actual complete order before native sealing.
fn hold(fixture: &Fixture, mut capture: ReviewCapture, pending: PendingLifecycleCohort,
    queue_raw: Option<&[u8]>, responses: &[(&str, &[u8])], ledger: &mut ContractLedger,
    control: &mut dyn WorkControl) -> ApprovedLifecycleClosure {
    if let Some(bytes) = queue_raw {
        std::fs::write(fixture.root.join("queue.json"), bytes).unwrap();
        capture.required(Path::new("queue.json"), CaptureRole::ReviewQueue, Pool::Queue, 10 * 1024 * 1024, ledger, control).unwrap();
    }
    for &(path, bytes) in responses {
        // Repeated registration borrows the existing same-path original without rewriting it.
        if !fixture.root.join(path).exists() { std::fs::write(fixture.root.join(path), bytes).unwrap(); }
        capture.required(Path::new(path), CaptureRole::ReviewResponse, Pool::Response, 1024 * 1024, ledger, control).unwrap();
    }
    pending.finish_and_seal(capture, ledger, control).unwrap()
}

/// Both empty opaque and shared generated originals keep the complete purpose and response rosters.
#[test]
fn genuine_owner_complete_pins_order_duplicates_and_foreign_evidence() {
    for (count, shared, empty) in [(0, false, true), (12, true, false)] {
        let fixture = Fixture::new(count, shared, empty);
        let mut ledger = ContractLedger::default();
        let mut caller = NoopControl;
        let mut control = ReviewControl::accept(&mut caller);
        let (capture, pending) = begin(&fixture, &mut ledger, &mut control);
        let queue = queue(&pending, &mut ledger, &mut control);
        let bytes = raw(&queue);
        let current = response(&queue, &bytes, false);
        let foreign = response(&queue, &bytes, true);
        let owner = hold(&fixture, capture, pending, Some(&bytes),
            &[("response.json", &current), ("foreign.json", &foreign), ("response.json", &current)], &mut ledger, &mut control);
        let bound = lifecycle_binding::prepare(&owner, &mut ledger, &mut control).unwrap();
        assert!(std::ptr::eq(bound.closure(), &raw const owner));
        assert!(bound.queue().document().source_pins.as_slice() == owner.source_pins());
        assert_eq!(bound.queue().raw(), bytes.as_slice());
        assert_eq!(bound.queue().raw().as_ptr(), bound.originals().queue_raw().as_ptr());
        assert_eq!(bound.responses().len(), 3);
        assert_eq!(bound.originals().responses().len(), 3);
        assert_eq!(bound.originals().response_indices()[0], bound.originals().response_indices()[2]);
        assert_ne!(bound.originals().response_indices()[0], bound.originals().response_indices()[1]);
        for (decoded, original) in bound.responses().iter().zip(bound.originals().responses()) {
            assert_eq!(decoded.raw(), original);
            assert_eq!(decoded.raw().as_ptr(), original.as_ptr());
            assert_eq!(decoded.raw_sha256(), crate::hashing::sha256_hex(original));
        }
        assert_eq!(owner.roster().len(), count + 2);
        assert_eq!(owner.held_inputs().source_original_count(), count + if shared { 1 } else { 2 });
        if count == 12 {
            let pins = owner.source_pins();
            assert!(pins.iter().position(|pin| pin.artifact_key == "lifecycle:generated:10").unwrap()
                < pins.iter().position(|pin| pin.artifact_key == "lifecycle:generated:2").unwrap());
        }
        let policy = merge_v2::prepare_policy(bound.queue(), bound.responses(), "2026-10-05T00:00:02Z", &mut ledger, &mut control).unwrap();
        assert_eq!(policy.counts().response_files, 3);
        assert_eq!(policy.counts().unique_responses, 2);
        assert_eq!(policy.counts().exact_duplicates, 1);
        assert_eq!(policy.responses().iter().filter(|row| row.classification == ResponseClassification::Foreign).count(), 1);
        assert_eq!(policy.responses().iter().filter(|row| row.classification == ResponseClassification::Current).count(), 1);
        assert!(!fixture.root.join("result.json").exists());
    }
}

/// Public self-consistent pin differences still fail against the actual complete native owner.
#[test]
fn genuine_full_pin_mismatches_reach_native_binding() {
    for field in 0..4 {
        let fixture = Fixture::new(1, false, false);
        let mut ledger = ContractLedger::default();
        let mut caller = NoopControl;
        let mut control = ReviewControl::accept(&mut caller);
        let (capture, pending) = begin(&fixture, &mut ledger, &mut control);
        let mut queue = queue(&pending, &mut ledger, &mut control);
        let generated = queue.source_pins.iter_mut().find(|pin| pin.kind == SourceKindV2::GeneratedArtifact).unwrap();
        match field {
            0 => generated.raw_sha256 = "0".repeat(64),
            1 => generated.byte_length += 1,
            2 => generated.native_model = Some(NativeModelV2::Profile),
            _ => generated.native_root_uuid = Some("22222222-2222-4222-8222-222222222222".to_string()),
        }
        refresh(&mut queue, &mut ledger, &mut control);
        let bytes = raw(&queue);
        let owner = hold(&fixture, capture, pending, Some(&bytes), &[], &mut ledger, &mut control);
        assert_eq!(lifecycle_binding::prepare(&owner, &mut ledger, &mut control).err(), Some(ContractError::Binding));
        assert!(!fixture.root.join("result.json").exists());
    }
}

/// A complete publicly coherent snapshot cannot substitute other private native keys or sources.
#[test]
fn genuine_private_subject_id_and_fingerprint_mismatches_refuse() {
    for stable_id in [false, true] {
        let fixture = Fixture::new(1, false, false);
        let mut ledger = ContractLedger::default();
        let mut caller = NoopControl;
        let mut control = ReviewControl::accept(&mut caller);
        let (capture, pending) = begin(&fixture, &mut ledger, &mut control);
        let mut queue = queue(&pending, &mut ledger, &mut control);
        let native = &pending.record.policy;
        if stable_id {
            queue.items[0].subject_id = hash_v2::subject_id(&native.policy_key, "different native version", &mut ledger, &mut control).unwrap();
        } else {
            queue.items[0].subject_sha256 = hash_v2::subject(&native.policy_key, &native.version_key, &"0".repeat(64), &mut ledger, &mut control).unwrap();
        }
        refresh(&mut queue, &mut ledger, &mut control);
        let bytes = raw(&queue);
        let owner = hold(&fixture, capture, pending, Some(&bytes), &[], &mut ledger, &mut control);
        assert_eq!(lifecycle_binding::prepare(&owner, &mut ledger, &mut control).err(), Some(ContractError::Binding));
        assert!(!fixture.root.join("result.json").exists());
    }
}

/// Missing real Queue and malformed actually registered Response originals cannot be skipped.
#[test]
fn genuine_registration_completeness_and_strict_raw_failure() {
    for missing_queue in [false, true] {
        let fixture = Fixture::new(1, false, false);
        let mut ledger = ContractLedger::default();
        let mut caller = NoopControl;
        let mut control = ReviewControl::accept(&mut caller);
        let (capture, pending) = begin(&fixture, &mut ledger, &mut control);
        let queue = queue(&pending, &mut ledger, &mut control);
        let bytes = raw(&queue);
        let queue_raw = if missing_queue { None } else { Some(bytes.as_slice()) };
        let malformed = &b""[..];
        let owner = hold(&fixture, capture, pending, queue_raw, &[("malformed.json", malformed)], &mut ledger, &mut control);
        let expected = if missing_queue { ContractError::Binding } else { ContractError::Invalid };
        assert_eq!(lifecycle_binding::prepare(&owner, &mut ledger, &mut control).err(), Some(expected));
        assert!(!fixture.root.join("result.json").exists());
    }
}

/// Only actual observations of the original accepted controller are shared externally.
#[derive(Default)]
struct State {
    /// Complete one-based callback count in the genuine operation.
    calls: usize,
    /// Calibrated actual selected call; no clock or ledger is reset.
    at: Option<usize>,
    /// Actual typed caller failure selection, rather than a synthesized capture proof.
    failed: bool,
}
/// The one original caller remains inside the same `ReviewControl` for native capture and binding.
struct Probe(Rc<RefCell<State>>);
impl WorkControl for Probe {
    /// Observe the real callback and return an actual selected original cause.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
        let mut state = self.0.borrow_mut();
        state.calls += 1;
        if state.at == Some(state.calls) {
            if state.failed { Err(WorkError::Failed(Error::invalid())) }
            else { Err(WorkError::Interrupted(Interruption::CancelRequested)) }
        } else { Ok(()) }
    }
    /// The original ledger/accepted controller retain first observed stops.
    fn interruption(&self) -> Option<Interruption> { None }
}

/// Calibrate on the same actual held owner and stop only the final post-ordinary Binding fence.
#[test]
fn genuine_failed_binding_postfence_and_sticky_original_stops() {
    for failed in [false, true] {
        let fixture = Fixture::new(1, false, false);
        let state = Rc::new(RefCell::new(State::default()));
        let mut caller = Probe(Rc::clone(&state));
        let mut control = ReviewControl::accept(&mut caller);
        let mut ledger = ContractLedger::default();
        let (capture, pending) = begin(&fixture, &mut ledger, &mut control);
        let mut queue = queue(&pending, &mut ledger, &mut control);
        queue.items[0].subject_id = hash_v2::subject_id("other native policy", "other native version", &mut ledger, &mut control).unwrap();
        refresh(&mut queue, &mut ledger, &mut control);
        let bytes = raw(&queue);
        let owner = hold(&fixture, capture, pending, Some(&bytes), &[], &mut ledger, &mut control);
        let before = state.borrow().calls;
        assert_eq!(lifecycle_binding::prepare(&owner, &mut ledger, &mut control).err(), Some(ContractError::Binding));
        let completed = state.borrow().calls;
        let delta = completed - before;
        assert!(delta > 2);
        { let mut observation = state.borrow_mut(); observation.at = Some(completed + delta); observation.failed = failed; }
        let expected = if failed { ContractError::ControlFailed } else { ContractError::Interrupted(Interruption::CancelRequested) };
        assert_eq!(lifecycle_binding::prepare(&owner, &mut ledger, &mut control).err(), Some(expected));
        let stopped = state.borrow().calls;
        assert_eq!(stopped, completed + delta);
        assert_eq!(lifecycle_binding::prepare(&owner, &mut ledger, &mut control).err(), Some(expected));
        assert_eq!(state.borrow().calls, stopped);
        assert!(!fixture.root.join("result.json").exists());
    }
}

/// Actual native physical source edits after sealing are caught by the leaf's whole-owner fence.
#[test]
fn genuine_actual_source_drift_at_complete_binding_fence() {
    let fixture = Fixture::new(1, false, false);
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let (capture, pending) = begin(&fixture, &mut ledger, &mut control);
    let queue = queue(&pending, &mut ledger, &mut control);
    let bytes = raw(&queue);
    let owner = hold(&fixture, capture, pending, Some(&bytes), &[], &mut ledger, &mut control);
    std::fs::write(fixture.root.join("source.bin"), b"changed physical source after real seal").unwrap();
    assert_eq!(lifecycle_binding::prepare(&owner, &mut ledger, &mut control).err(), Some(ContractError::Binding));
    assert!(!fixture.root.join("result.json").exists());
}

/// A genuine unchanged-cap derived failure on the original ledger wins over every later caller stop.
#[test]
fn genuine_original_capacity_precedes_later_binding_and_control_failures() {
    let fixture = Fixture::new(1, false, false);
    let state = Rc::new(RefCell::new(State::default()));
    let mut caller = Probe(Rc::clone(&state));
    let mut control = ReviewControl::accept(&mut caller);
    let mut ledger = ContractLedger::default();
    let (capture, pending) = begin(&fixture, &mut ledger, &mut control);
    let queue = queue(&pending, &mut ledger, &mut control);
    let bytes = raw(&queue);
    let owner = hold(&fixture, capture, pending, Some(&bytes), &[], &mut ledger, &mut control);
    assert_eq!(ledger.derived(33_554_433), Err(ContractError::Capacity));
    let before = state.borrow().calls;
    { let mut observation = state.borrow_mut(); observation.at = Some(before + 1); observation.failed = true; }
    assert_eq!(lifecycle_binding::prepare(&owner, &mut ledger, &mut control).err(), Some(ContractError::Capacity));
    assert_eq!(state.borrow().calls, before);
    assert!(!fixture.root.join("result.json").exists());
}


/// Actual finalizer controls share the same private captured native fixture and helpers.
mod finalizer_tests {
    include!("lifecycle_finalize_tests.rs");
}

/// Genuine current merge/status sink controls remain beneath the real binding fixture.
mod command_tests { include!("lifecycle_commands_tests.rs"); }

/// Genuine ordinary response sink controls remain beneath the real binding fixture.
mod response_tests { include!("lifecycle_response_tests.rs"); }
