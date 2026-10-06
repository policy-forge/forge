// Genuine new-output Lifecycle queue projection controls.
// Maintained lifecycle init/transitions, actual Auxiliary leases and full native
// owners are used; no private proof factory or fabricated captured Queue exists.

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use serde_json::{Value, json};

use super::Fixture;
use crate::evidence_capture::CaptureRole;
use crate::review::capture::{Pool, ReviewCapture, ReviewControl};
use crate::review::decode::{ContractError, ContractLedger};
use crate::review::{decode_v2, encode_v2, lifecycle_capture, lifecycle_queue_export, merge_v2};
use crate::review::lifecycle_capture::ApprovedLifecycleClosure;
use crate::workspace::contract::Error;
use crate::workspace::preparation::{Interruption, NoopControl, ProgressUpdate, Stage, WorkControl, WorkError, WorkResult};

/// One explicit caller UUID, never native subject identity or a detached approval token.
const QUEUE: &str = "10000000-0000-4000-8000-000000000001";
/// One explicit canonical caller creation second, with no ambient clock.
const CREATED: &str = "2026-10-05T00:00:00Z";

/// A complete closed asserted-only request, retaining all five required item fields.
fn request() -> Value {
    json!({"schema_version":"forge.review-lifecycle-init/1",
        "roles":[{"key":"review"}],
        "reviewers":[{"key":"asserted-author","role_keys":["review"]},
            {"key":"asserted-reviewer","role_keys":["review"]}],
        "policies":[{"key":"policy-1","seats":[{"role_key":"review","count":1}],
            "substitutions":[],"abstention_rule":"nonapproving",
            "empty_abstention_reasons":["absent"],"author_separation":"declared-keys"}],
        "items":[{"key":"item-1","policy_key":"policy-1","author_keys":["asserted-author"],
            "assignments":[{"reviewer_key":"asserted-reviewer","role_key":"review"}],"due_at":null}]})
}

/// Actual capture/native read plus real private-request registration before complete sealing.
fn owner(
    fixture: &Fixture, raw: &[u8], ledger: &mut ContractLedger, control: &mut dyn WorkControl,
) -> (ApprovedLifecycleClosure, usize) {
    let route = fixture.root.join("init.json");
    if route.exists() { assert_eq!(std::fs::read(&route).unwrap(), raw); }
    else { std::fs::write(&route, raw).unwrap(); }
    let mut capture = ReviewCapture::new_lifecycle(&fixture.root, &[Path::new("result.json")], ledger, control).unwrap();
    let pending = lifecycle_capture::read_pending(&mut capture, Path::new("locator.json"), ledger, control).unwrap();
    let index = capture.required(Path::new("init.json"), CaptureRole::ReviewPrivateConfig, Pool::Auxiliary,
        1024 * 1024, ledger, control).unwrap();
    (pending.finish_and_seal(capture, ledger, control).unwrap(), index)
}

/// The actual original lease bytes, not a reconstruction from request declarations.
fn actual(owner: &ApprovedLifecycleClosure, index: usize) -> &[u8] {
    owner.held_inputs().bytes(index).unwrap()
}

/// Complete genuine Source/purpose denominator and private native facts survive finite new output.
#[test]
fn genuine_complete_new_queue_export_empty_and_repeated_native_sources() {
    for (count, shared, empty) in [(0, false, true), (12, true, false)] {
        let fixture = Fixture::new(count, shared, empty);
        let input = serde_json::to_vec(&request()).unwrap();
        let mut ledger = ContractLedger::default();
        let mut caller = NoopControl;
        let mut control = ReviewControl::accept(&mut caller);
        let (native, index) = owner(&fixture, &input, &mut ledger, &mut control);
        let prepared = lifecycle_queue_export::prepare(&native, actual(&native, index), QUEUE, CREATED, &mut ledger, &mut control).unwrap();
        assert!(std::ptr::eq(prepared.closure(), &raw const native));
        assert!(prepared.document().source_pins.as_slice() == native.source_pins());
        assert_eq!(prepared.document().items[0].source_keys.len(), count + 2);
        assert_eq!(native.roster().len(), count + 2);
        assert_eq!(native.held_inputs().source_original_count(), count + if shared { 1 } else { 2 });
        assert_eq!(native.held_inputs().lifecycle_review_originals(&mut ledger, &mut control).err(), Some(ContractError::Binding));
        let output = encode_v2::exchange::queue(prepared.document(), &mut ledger, &mut control).unwrap();
        let decoded = decode_v2::decode_queue(&output, &mut ledger, &mut control).unwrap();
        prepared.bind_output(&decoded, &mut ledger, &mut control).unwrap();
        assert_eq!(output.last(), Some(&b'\n'));
        assert_eq!(decoded.raw().as_ptr(), output.as_ptr());
        assert!(decoded.document() == prepared.document());
        let displayed = std::str::from_utf8(&output).unwrap();
        for private in ["private native title", "private native rationale", "source.bin", "generated/", r"native\u0000policy"] {
            assert!(!displayed.contains(private));
        }
        assert!(!fixture.root.join("result.json").exists());
    }
}

/// A copied equal raw or shorter same-pointer slice fails, then the real original works unchanged.
#[test]
fn genuine_auxiliary_copy_and_full_extent_refusals_allow_original_continuation() {
    let fixture = Fixture::new(1, false, false);
    let input = serde_json::to_vec(&request()).unwrap();
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let (native, index) = owner(&fixture, &input, &mut ledger, &mut control);
    let raw = actual(&native, index);
    let equal_copy = raw.to_vec();
    assert_eq!(equal_copy, raw);
    assert_ne!(equal_copy.as_ptr(), raw.as_ptr());
    for refused in [equal_copy.as_slice(), &raw[..raw.len() - 1]] {
        assert_eq!(lifecycle_queue_export::prepare(&native, refused, QUEUE, CREATED, &mut ledger, &mut control).err(), Some(ContractError::Binding));
    }
    let prepared = lifecycle_queue_export::prepare(&native, raw, QUEUE, CREATED, &mut ledger, &mut control).unwrap();
    let output = encode_v2::exchange::queue(prepared.document(), &mut ledger, &mut control).unwrap();
    let decoded = decode_v2::decode_queue(&output, &mut ledger, &mut control).unwrap();
    prepared.bind_output(&decoded, &mut ledger, &mut control).unwrap();
    assert!(!fixture.root.join("result.json").exists());
}

/// A real same-byte Response original has the wrong actual role/pool; Auxiliary remains genuine.
#[test]
fn genuine_wrong_role_original_refuses_before_private_decode() {
    let fixture = Fixture::new(1, false, false);
    let input = serde_json::to_vec(&request()).unwrap();
    std::fs::write(fixture.root.join("init.json"), &input).unwrap();
    std::fs::write(fixture.root.join("foreign.json"), &input).unwrap();
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let mut capture = ReviewCapture::new_lifecycle(&fixture.root, &[Path::new("result.json")], &mut ledger, &mut control).unwrap();
    let pending = lifecycle_capture::read_pending(&mut capture, Path::new("locator.json"), &mut ledger, &mut control).unwrap();
    let wrong = capture.required(Path::new("foreign.json"), CaptureRole::ReviewResponse, Pool::Response, 1024 * 1024, &mut ledger, &mut control).unwrap();
    let right = capture.required(Path::new("init.json"), CaptureRole::ReviewPrivateConfig, Pool::Auxiliary, 1024 * 1024, &mut ledger, &mut control).unwrap();
    let native = pending.finish_and_seal(capture, &mut ledger, &mut control).unwrap();
    assert_eq!(actual(&native, wrong), actual(&native, right));
    assert_eq!(lifecycle_queue_export::prepare(&native, actual(&native, wrong), QUEUE, CREATED, &mut ledger, &mut control).err(), Some(ContractError::Binding));
    lifecycle_queue_export::prepare(&native, actual(&native, right), QUEUE, CREATED, &mut ledger, &mut control).unwrap();
    assert!(!fixture.root.join("result.json").exists());
}

/// Equal hashes/routes from another real owner cannot repair the actual allocation membership.
#[test]
fn genuine_other_owner_request_original_is_not_an_auxiliary_proof() {
    let fixture = Fixture::new(1, false, false);
    let input = serde_json::to_vec(&request()).unwrap();
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let (first, first_index) = owner(&fixture, &input, &mut ledger, &mut control);
    let (second, second_index) = owner(&fixture, &input, &mut ledger, &mut control);
    assert_eq!(actual(&first, first_index), actual(&second, second_index));
    assert_ne!(actual(&first, first_index).as_ptr(), actual(&second, second_index).as_ptr());
    assert_eq!(lifecycle_queue_export::prepare(&first, actual(&second, second_index), QUEUE, CREATED, &mut ledger, &mut control).err(), Some(ContractError::Binding));
    lifecycle_queue_export::prepare(&first, actual(&first, first_index), QUEUE, CREATED, &mut ledger, &mut control).unwrap();
    assert!(!fixture.root.join("result.json").exists());
}

/// Every actual private request rejects native operands, omitted required nulls and duplicate keys.
#[test]
fn genuine_private_request_is_closed_and_cannot_supply_native_facts() {
    for case in 0..7 {
        let fixture = Fixture::new(1, false, false);
        let mut value = request();
        match case {
            0 => value["source_pins"] = json!([]),
            1 => value["items"][0]["subject_sha256"] = json!("0".repeat(64)),
            2 => { value["items"][0].as_object_mut().unwrap().remove("due_at"); },
            3 => value["policies"][0]["unknown"] = json!(true),
            4 => { value["items"].as_array_mut().unwrap().clear(); },
            5 => value["schema_version"] = json!("forge.review-lifecycle-inputs/1"),
            _ => {}
        }
        let mut raw = serde_json::to_vec(&value).unwrap();
        if case == 6 {
            raw.splice(1..1, b"\"schema_version\":\"forge.review-lifecycle-init/1\", ".iter().copied());
        }
        let mut ledger = ContractLedger::default();
        let mut caller = NoopControl;
        let mut control = ReviewControl::accept(&mut caller);
        let (native, index) = owner(&fixture, &raw, &mut ledger, &mut control);
        assert_eq!(lifecycle_queue_export::prepare(&native, actual(&native, index), QUEUE, CREATED, &mut ledger, &mut control).err(), Some(ContractError::Invalid));
        assert!(!fixture.root.join("result.json").exists());
    }
}

/// Actual structural success cannot bypass role membership, author exclusion or UTC equality.
#[test]
fn genuine_asserted_policy_and_explicit_headers_are_fully_validated() {
    for case in 0..6 {
        let fixture = Fixture::new(1, false, false);
        let mut value = request();
        let mut queue_id = QUEUE;
        let mut created_at = CREATED;
        match case {
            0 => value["items"][0]["assignments"][0]["reviewer_key"] = json!("asserted-author"),
            1 => value["items"][0]["assignments"][0]["role_key"] = json!("undeclared"),
            2 => value["items"][0]["due_at"] = json!(CREATED),
            3 => value["items"][0]["policy_key"] = json!("other-policy"),
            4 => queue_id = "10000000-0000-4000-8000-00000000000A",
            _ => created_at = "2026-10-05T00:00:00+00:00"
        }
        let raw = serde_json::to_vec(&value).unwrap();
        let mut ledger = ContractLedger::default();
        let mut caller = NoopControl;
        let mut control = ReviewControl::accept(&mut caller);
        let (native, index) = owner(&fixture, &raw, &mut ledger, &mut control);
        assert_eq!(lifecycle_queue_export::prepare(&native, actual(&native, index), queue_id, created_at, &mut ledger, &mut control).err(), Some(ContractError::Invalid));
        assert!(!fixture.root.join("result.json").exists());
    }
}

/// A legitimate unassigned item and explicit later due second do not infer native party seats.
#[test]
fn genuine_unassigned_and_nullable_due_policy_preserves_asserted_identity() {
    for due in [None, Some("2026-10-06T00:00:00Z")] {
        let fixture = Fixture::new(6, false, false);
        let mut value = request();
        value["items"][0]["assignments"] = json!([]);
        value["items"][0]["due_at"] = json!(due);
        let raw = serde_json::to_vec(&value).unwrap();
        let mut ledger = ContractLedger::default();
        let mut caller = NoopControl;
        let mut control = ReviewControl::accept(&mut caller);
        let (native, index) = owner(&fixture, &raw, &mut ledger, &mut control);
        let prepared = lifecycle_queue_export::prepare(&native, actual(&native, index), QUEUE, CREATED, &mut ledger, &mut control).unwrap();
        assert_eq!(prepared.document().items[0].due_at.as_deref(), due);
        assert!(prepared.document().items[0].assignments.is_empty());
        for reviewer in &prepared.document().reviewers { assert!(reviewer.key.starts_with("asserted-")); }
        let output = encode_v2::exchange::queue(prepared.document(), &mut ledger, &mut control).unwrap();
        let decoded = decode_v2::decode_queue(&output, &mut ledger, &mut control).unwrap();
        prepared.bind_output(&decoded, &mut ledger, &mut control).unwrap();
        let policy = merge_v2::prepare_policy(&decoded, &[], CREATED, &mut ledger, &mut control).unwrap();
        assert_eq!(policy.items()[0].tentative_state, merge_v2::TentativeState::Unassigned);
        assert_eq!(policy.counts().response_files, 0);
        assert!(!fixture.root.join("result.json").exists());
    }
}

/// A schema-valid new Queue with changed nonnative registry/header data fails full output equality.
#[test]
fn genuine_full_typed_output_equality_rejects_coherent_replacements() {
    for header in [false, true] {
        let fixture = Fixture::new(1, false, false);
        let input = serde_json::to_vec(&request()).unwrap();
        let mut ledger = ContractLedger::default();
        let mut caller = NoopControl;
        let mut control = ReviewControl::accept(&mut caller);
        let (native, index) = owner(&fixture, &input, &mut ledger, &mut control);
        let prepared = lifecycle_queue_export::prepare(&native, actual(&native, index), QUEUE, CREATED, &mut ledger, &mut control).unwrap();
        let mut value = serde_json::to_value(prepared.document()).unwrap();
        if header { value["created_at"] = json!("2026-10-05T00:00:01Z"); }
        else { value["reviewers"].as_array_mut().unwrap().push(json!({"key":"z-extra","role_keys":["review"]})); }
        let raw = serde_json::to_vec(&value).unwrap();
        let decoded = decode_v2::decode_queue(&raw, &mut ledger, &mut control).unwrap();
        assert_eq!(prepared.bind_output(&decoded, &mut ledger, &mut control), Err(ContractError::Binding));
        let output = encode_v2::exchange::queue(prepared.document(), &mut ledger, &mut control).unwrap();
        let decoded = decode_v2::decode_queue(&output, &mut ledger, &mut control).unwrap();
        prepared.bind_output(&decoded, &mut ledger, &mut control).unwrap();
        assert!(!fixture.root.join("result.json").exists());
    }
}

/// The real native Source or private init file can drift after projection, before actual output fence.
#[test]
fn genuine_new_output_binding_rechecks_native_and_private_request_originals() {
    for policy in [false, true] {
        let fixture = Fixture::new(1, false, false);
        let input = serde_json::to_vec(&request()).unwrap();
        let mut ledger = ContractLedger::default();
        let mut caller = NoopControl;
        let mut control = ReviewControl::accept(&mut caller);
        let (native, index) = owner(&fixture, &input, &mut ledger, &mut control);
        let prepared = lifecycle_queue_export::prepare(&native, actual(&native, index), QUEUE, CREATED, &mut ledger, &mut control).unwrap();
        let output = encode_v2::exchange::queue(prepared.document(), &mut ledger, &mut control).unwrap();
        let decoded = decode_v2::decode_queue(&output, &mut ledger, &mut control).unwrap();
        let route = if policy { "init.json" } else { "source.bin" };
        std::fs::write(fixture.root.join(route), b"actual changed in-place original after export preparation").unwrap();
        assert_eq!(prepared.bind_output(&decoded, &mut ledger, &mut control), Err(ContractError::Binding));
        assert!(!fixture.root.join("result.json").exists());
    }
}

/// Shared observations belong to one actual accepted controller through the entire operation.
#[derive(Default)]
struct State {
    /// Actual original callback count, never a fresh deadline or reconstructed progress.
    calls: usize,
    /// Fixed actual one-based callback target, calibrated on that same failed phase.
    at: Option<usize>,
    /// Actual Failed versus Interrupted cause at the chosen original callback.
    failed: bool,
}
/// The original caller is retained by `ReviewControl`; no alternate production controller exists.
struct Probe(Rc<RefCell<State>>);
impl WorkControl for Probe {
    /// Produce the actual chosen original failure only at its observed checkpoint.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
        let mut state = self.0.borrow_mut();
        state.calls += 1;
        if state.at == Some(state.calls) {
            if state.failed { Err(WorkError::Failed(Error::invalid())) }
            else { Err(WorkError::Interrupted(Interruption::CancelRequested)) }
        } else { Ok(()) }
    }
    /// Actual first stop is retained by the original accepted controller/ledger.
    fn interruption(&self) -> Option<Interruption> { None }
}

/// Ordinary malformed actual request retains the original final postphase stop and work.
#[test]
fn genuine_private_decode_failure_postfence_keeps_same_original_first_stop() {
    for failed in [false, true] {
        let fixture = Fixture::new(1, false, false);
        let state = Rc::new(RefCell::new(State::default()));
        let mut caller = Probe(Rc::clone(&state));
        let mut control = ReviewControl::accept(&mut caller);
        let mut ledger = ContractLedger::default();
        let (native, index) = owner(&fixture, b"{}", &mut ledger, &mut control);
        let before = state.borrow().calls;
        assert_eq!(lifecycle_queue_export::prepare(&native, actual(&native, index), QUEUE, CREATED, &mut ledger, &mut control).err(), Some(ContractError::Invalid));
        let complete = state.borrow().calls;
        let delta = complete - before;
        assert!(delta > 1);
        { let mut shared = state.borrow_mut(); shared.at = Some(complete + delta); shared.failed = failed; }
        let expected = if failed { ContractError::ControlFailed } else { ContractError::Interrupted(Interruption::CancelRequested) };
        assert_eq!(lifecycle_queue_export::prepare(&native, actual(&native, index), QUEUE, CREATED, &mut ledger, &mut control).err(), Some(expected));
        let stopped = state.borrow().calls;
        assert_eq!(stopped, complete + delta);
        assert_eq!(lifecycle_queue_export::prepare(&native, actual(&native, index), QUEUE, CREATED, &mut ledger, &mut control).err(), Some(expected));
        assert_eq!(state.borrow().calls, stopped);
        assert!(!fixture.root.join("result.json").exists());
    }
}

/// Ordinary coherent output mismatch reaches the original postfailure fence, with no quota reset.
#[test]
fn genuine_output_equality_failure_postfence_keeps_same_original_first_stop() {
    for failed in [false, true] {
        let fixture = Fixture::new(1, false, false);
        let input = serde_json::to_vec(&request()).unwrap();
        let state = Rc::new(RefCell::new(State::default()));
        let mut caller = Probe(Rc::clone(&state));
        let mut control = ReviewControl::accept(&mut caller);
        let mut ledger = ContractLedger::default();
        let (native, index) = owner(&fixture, &input, &mut ledger, &mut control);
        let prepared = lifecycle_queue_export::prepare(&native, actual(&native, index), QUEUE, CREATED, &mut ledger, &mut control).unwrap();
        let mut value = serde_json::to_value(prepared.document()).unwrap();
        value["created_at"] = json!("2026-10-05T00:00:01Z");
        let raw = serde_json::to_vec(&value).unwrap();
        let decoded = decode_v2::decode_queue(&raw, &mut ledger, &mut control).unwrap();
        let before = state.borrow().calls;
        assert_eq!(prepared.bind_output(&decoded, &mut ledger, &mut control), Err(ContractError::Binding));
        let complete = state.borrow().calls;
        let delta = complete - before;
        assert!(delta > 1);
        { let mut shared = state.borrow_mut(); shared.at = Some(complete + delta); shared.failed = failed; }
        let expected = if failed { ContractError::ControlFailed } else { ContractError::Interrupted(Interruption::CancelRequested) };
        assert_eq!(prepared.bind_output(&decoded, &mut ledger, &mut control), Err(expected));
        let stopped = state.borrow().calls;
        assert_eq!(stopped, complete + delta);
        assert_eq!(prepared.bind_output(&decoded, &mut ledger, &mut control), Err(expected));
        assert_eq!(state.borrow().calls, stopped);
        assert!(!fixture.root.join("result.json").exists());
    }
}

/// A genuine unchanged-cap first admission refusal prevents later parsing or callback replacement.
#[test]
fn genuine_export_capacity_on_original_owner_precedes_later_control_failure() {
    let fixture = Fixture::new(1, false, false);
    let input = serde_json::to_vec(&request()).unwrap();
    let state = Rc::new(RefCell::new(State::default()));
    let mut caller = Probe(Rc::clone(&state));
    let mut control = ReviewControl::accept(&mut caller);
    let mut ledger = ContractLedger::default();
    let (native, index) = owner(&fixture, &input, &mut ledger, &mut control);
    assert_eq!(ledger.derived(33_554_433), Err(ContractError::Capacity));
    let before = state.borrow().calls;
    { let mut shared = state.borrow_mut(); shared.at = Some(before + 1); shared.failed = true; }
    assert_eq!(lifecycle_queue_export::prepare(&native, actual(&native, index), QUEUE, CREATED, &mut ledger, &mut control).err(), Some(ContractError::Capacity));
    assert_eq!(state.borrow().calls, before);
    assert!(!fixture.root.join("result.json").exists());
}
