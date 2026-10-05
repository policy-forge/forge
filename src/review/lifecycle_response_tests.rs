// Genuine ordinary /2 response controls beneath the real captured binding fixture.
// Native-origin Queue input preparation supplies no currentness to the response command.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use super::{begin, hold, queue, raw, response, Probe, State};
use super::super::Fixture;
use crate::review::commands::{CommandError, RespondOptions};
use crate::review::decode::{ContractError, ContractLedger};
use crate::review::{decode_v2, lifecycle_commands};
use crate::review::wire_v2::{Disposition, QueueDocumentV2, SourceKindV2};
use crate::workspace::contract::Error;
use crate::workspace::preparation::{Interruption, NoopControl, ProgressUpdate, Stage, WorkControl, WorkError, WorkResult};

/// Prepare a real complete native-origin Queue original; preparation owners drop before respond.
/// The command captures only its actual ordinary Queue and private rationale originals.
fn inputs(count: usize, rationale: &[u8]) -> (Fixture, QueueDocumentV2, Vec<u8>) {
    let fixture = Fixture::new(count, false, false);
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let (capture, pending) = begin(&fixture, &mut ledger, &mut caller);
    let declaration = queue(&pending, &mut ledger, &mut caller);
    let queue_raw = raw(&declaration);
    let owner = hold(&fixture, capture, pending, Some(&queue_raw), &[], &mut ledger, &mut caller);
    drop(owner);
    std::fs::write(fixture.root.join("rationale.txt"), rationale).unwrap();
    (fixture, declaration, queue_raw)
}

/// Explicit asserted inputs select one actual Queue item and a new confined response destination.
fn options(fixture: &Fixture) -> RespondOptions<'_> {
    RespondOptions { project_root: &fixture.root, queue: Path::new("queue.json"),
        rationale_file: Path::new("rationale.txt"), item_key: "item-1", reviewer_key: "asserted-reviewer",
        reviewer_role: "review", disposition: Disposition::Approve, responded_at: "2026-10-05T00:00:01Z",
        response_id: "20000000-0000-4000-8000-000000000002", abstention_reason: None,
        supersedes: None, output: Path::new("result.json") }
}

/// Compare a precise safe contract cause without changing production error derives.
fn contract<T>(result: Result<T, CommandError>, expected: ContractError) {
    let actual = result.err();
    assert!(matches!(actual, Some(CommandError::Contract(error)) if error == expected), "expected {expected:?}, actual {actual:?}");
}

/// Compare the fixed native publication category without exposing private publisher errors.
fn publication<T>(result: Result<T, CommandError>) {
    let actual = result.err();
    assert!(matches!(actual, Some(CommandError::Publication)), "expected Publication, actual {actual:?}");
}

/// Check actual native publisher success or its genuine unsupported-platform refusal.
/// An unsupported publication cannot be treated as an observed response output.
fn published(fixture: &Fixture, result: Result<(), CommandError>) -> Option<Vec<u8>> {
    if cfg!(any(target_os = "linux", target_os = "macos")) {
        result.unwrap();
        Some(std::fs::read(fixture.root.join("result.json")).unwrap())
    } else {
        assert!(matches!(result, Err(CommandError::Publication)));
        assert!(!fixture.root.join("result.json").exists());
        None
    }
}

/// Validate full actual output against a specification fixture and the exact whole Queue raw.
/// This ordinary readback grants no native currentness or owner from loaded labels.
fn complete(actual: &[u8], queue: &QueueDocumentV2, queue_raw: &[u8], rationale: &str) {
    let mut expected: serde_json::Value = serde_json::from_slice(&response(queue, queue_raw, false)).unwrap();
    expected["rationale"] = serde_json::json!(rationale);
    let observed: serde_json::Value = serde_json::from_slice(actual).unwrap();
    assert_eq!(observed.as_object().unwrap().len(), 22);
    assert_eq!(observed, expected);
    assert!(actual.ends_with(b"\n"));
    let mut ledger = ContractLedger::default();
    let decoded_queue = decode_v2::decode_queue(queue_raw, &mut ledger, &mut NoopControl).unwrap();
    let decoded = decode_v2::decode_response(actual, &mut ledger, &mut NoopControl).unwrap();
    decode_v2::bind_response(&decoded_queue, &decoded, &mut ledger, &mut NoopControl).unwrap();
    assert!(decoded.document().source_pins == queue.source_pins);
    assert_eq!(decoded.document().rationale, rationale);
    assert_eq!(decoded.document().queue_raw_sha256, crate::hashing::sha256_hex(queue_raw));
}

/// Complete genuine output preserves every nullable pin and all six original native UUID forms.
/// Source changes outside the ordinary Queue/rationale cohort confer no response currentness.
#[test]
fn genuine_response_complete_eight_field_pins_nulls_native_uuid_spelling_and_private_rationale() {
    let rationale = " private λ\0reason \n ";
    let (fixture, declaration, queue_raw) = inputs(12, rationale.as_bytes());
    let before_record = std::fs::read(fixture.root.join("record.json")).unwrap();
    // This operation makes no native-currentness promise and never opens source.bin.
    std::fs::write(fixture.root.join("source.bin"), b"actual native source now differs from recorded Queue").unwrap();
    let result = lifecycle_commands::respond(&options(&fixture), &mut NoopControl);
    if let Some(actual) = published(&fixture, result) {
        complete(&actual, &declaration, &queue_raw, rationale);
        let generated: Vec<_> = declaration.source_pins.iter().filter(|pin|
            pin.kind == SourceKindV2::GeneratedArtifact).collect();
        assert_eq!(generated.len(), 12);
        for pin in declaration.source_pins.iter().filter(|pin| pin.kind != SourceKindV2::GeneratedArtifact) {
            assert!(pin.native_model.is_none() && pin.native_root_uuid.is_none());
        }
        for spelling in ["11111111-1111-4111-8111-111111111111", "ABCDEFAB1234556789ABCDEF01234567",
            "{ABCDEFAB-1234-5567-89AB-CDEF01234567}", "urn:uuid:ABCDEFAB-1234-5567-89AB-CDEF01234567",
            "00000000-0000-0000-0000-000000000000", "abcdefab-1234-1567-09ab-cdef01234567"] {
            assert_eq!(generated.iter().filter(|pin| pin.native_root_uuid.as_deref() == Some(spelling)).count(), 2);
        }
        publication(lifecycle_commands::respond(&options(&fixture), &mut NoopControl));
        assert_eq!(std::fs::read(fixture.root.join("result.json")).unwrap(), actual);
    }
    assert_eq!(std::fs::read(fixture.root.join("queue.json")).unwrap(), queue_raw);
    assert_eq!(std::fs::read(fixture.root.join("rationale.txt")).unwrap(), rationale.as_bytes());
    assert_eq!(std::fs::read(fixture.root.join("record.json")).unwrap(), before_record);
}

/// Actual empty or whitespace rationale follows only the selected explicit abstention policy.
#[test]
fn genuine_response_empty_abstention_policy_allowed_and_disallowed() {
    for (rationale, reason, allowed) in [("", "absent", true), (" \n\t", "absent", true),
        ("", "other", false), ("nonempty private abstention", "other", true)] {
        let (fixture, declaration, queue_raw) = inputs(1, rationale.as_bytes());
        let mut request = options(&fixture);
        request.disposition = Disposition::Abstain;
        request.abstention_reason = Some(reason);
        let result = lifecycle_commands::respond(&request, &mut NoopControl);
        if !allowed {
            contract(result, ContractError::Binding);
            assert!(!fixture.root.join("result.json").exists());
        } else if let Some(actual) = published(&fixture, result) {
            let mut expected: serde_json::Value = serde_json::from_slice(&response(&declaration, &queue_raw, false)).unwrap();
            expected["disposition"] = serde_json::json!("abstain");
            expected["abstention_reason"] = serde_json::json!(reason);
            expected["rationale"] = serde_json::json!(rationale);
            assert_eq!(serde_json::from_slice::<serde_json::Value>(&actual).unwrap(), expected);
            let mut ledger = ContractLedger::default();
            let q = decode_v2::decode_queue(&queue_raw, &mut ledger, &mut NoopControl).unwrap();
            let r = decode_v2::decode_response(&actual, &mut ledger, &mut NoopControl).unwrap();
            decode_v2::bind_response(&q, &r, &mut ledger, &mut NoopControl).unwrap();
            assert!(r.document().source_pins == declaration.source_pins);
        }
    }
    let (fixture, _, _) = inputs(0, b"  \n\t");
    contract(lifecycle_commands::respond(&options(&fixture), &mut NoopControl), ContractError::Invalid);
    assert!(!fixture.root.join("result.json").exists());
}

/// Full physical rationale bytes are bounded before publication; invalid UTF-8 never substitutes text.
#[test]
fn genuine_response_rationale_utf8_full_limit_and_over_limit_refusal() {
    for (rationale, expected) in [(vec![0xff, 0, b'x'], Some(ContractError::Invalid)),
        (vec![b'x'; 8_193], Some(ContractError::Capacity)), (vec![b'x'; 8_192], None)] {
        let (fixture, declaration, queue_raw) = inputs(1, &rationale);
        let result = lifecycle_commands::respond(&options(&fixture), &mut NoopControl);
        if let Some(error) = expected {
            contract(result, error);
            assert!(!fixture.root.join("result.json").exists());
        } else if let Some(actual) = published(&fixture, result) {
            complete(&actual, &declaration, &queue_raw, std::str::from_utf8(&rationale).unwrap());
        }
        assert_eq!(std::fs::read(fixture.root.join("rationale.txt")).unwrap(), rationale);
    }
}

/// Asserted author exclusion, membership, selected item, chronology and UUID grammar stay enforced.
#[test]
fn genuine_response_asserted_eligibility_and_scalar_refusals_leave_no_output() {
    for case in 0..6 {
        let (fixture, _, _) = inputs(1, b"private assertion rationale");
        let mut request = options(&fixture);
        let expected = match case {
            0 => { request.reviewer_key = "asserted-author"; ContractError::Binding },
            1 => { request.reviewer_key = "undeclared"; ContractError::Invalid },
            2 => { request.reviewer_role = "other"; ContractError::Binding },
            3 => { request.item_key = "absent-item"; ContractError::Binding },
            4 => { request.responded_at = "2026-10-04T23:59:59Z"; ContractError::Binding },
            _ => { request.response_id = "20000000000040008000000000000002"; ContractError::Invalid },
        };
        contract(lifecycle_commands::respond(&request, &mut NoopControl), expected);
        assert!(!fixture.root.join("result.json").exists());
    }
}

/// Missing ordinary inputs return Binding after lexical output namespace reservation.
/// Reservation performs no destination IO; the existing bytes and absence of staging are retained.
#[test]
fn genuine_response_reserved_output_namespace_missing_inputs_preserve_existing_bytes() {
    let (fixture, _, _) = inputs(1, b"private rationale");
    std::fs::write(fixture.root.join("result.json"), b"existing private response bytes").unwrap();
    std::fs::remove_file(fixture.root.join("queue.json")).unwrap();
    std::fs::remove_file(fixture.root.join("rationale.txt")).unwrap();
    contract(lifecycle_commands::respond(&options(&fixture), &mut NoopControl), ContractError::Binding);
    assert_eq!(std::fs::read(fixture.root.join("result.json")).unwrap(), b"existing private response bytes");
    assert!(std::fs::read_dir(&fixture.root).unwrap().all(|entry|
        !entry.unwrap().file_name().to_string_lossy().starts_with(".forge-authoring-file-")));
}

/// Calibrate the final actual caller callback on an ordinary malformed-rationale result.
/// Every invocation has one original accepted control; no in-operation deadline/ledger reset occurs.
#[test]
fn genuine_response_ordinary_failure_final_original_stop_supersedes_invalid() {
    let (fixture, _, _) = inputs(1, &[0xff]);
    for failed in [false, true] {
        let state = Rc::new(RefCell::new(State::default()));
        let mut caller = Probe(Rc::clone(&state));
        contract(lifecycle_commands::respond(&options(&fixture), &mut caller), ContractError::Invalid);
        let delta = state.borrow().calls;
        assert!(delta > 2);
        { let mut observed = state.borrow_mut(); observed.at = Some(delta * 2); observed.failed = failed; }
        let expected = if failed { ContractError::ControlFailed }
            else { ContractError::Interrupted(Interruption::CancelRequested) };
        contract(lifecycle_commands::respond(&options(&fixture), &mut caller), expected);
        assert_eq!(state.borrow().calls, delta * 2);
        assert!(!fixture.root.join("result.json").exists());
    }
}

/// Observe genuine complete staging and change an actual captured ordinary input in place.
/// Private staging-name coupling is explicit; this is not an atomic snapshot or race proof.
struct StagedDrift {
    /// Real fixture root retained throughout publication.
    root: PathBuf,
    /// Actual Queue or rationale route whose existing native file remains open.
    path: &'static str,
    /// Complete staged response observation precedes the physical input edit.
    edited: bool,
}
impl WorkControl for StagedDrift {
    /// The actual guarded pre-rename phase exposes a completed finite response file.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
        if !self.edited {
            for entry in std::fs::read_dir(&self.root).unwrap() {
                let entry = entry.unwrap();
                if !entry.file_name().to_string_lossy().starts_with(".forge-authoring-file-") { continue; }
                let Ok(bytes) = std::fs::read(entry.path().join("file")) else { continue; };
                if !bytes.ends_with(b"\n") { continue; }
                let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                assert_eq!(value["schema_version"], "forge.review-response/2");
                assert_eq!(value["rationale"], "private physical rationale");
                std::fs::write(self.root.join(self.path), b"changed actual captured ordinary input bytes").unwrap();
                self.edited = true;
                break;
            }
        }
        Ok(())
    }
    /// Genuine physical drift is diagnosed by full capture verification, not a fabricated caller stop.
    fn interruption(&self) -> Option<Interruption> { None }
}

/// Complete response staging is refused after either real Queue or rationale byte drift.
#[test]
fn genuine_response_guarded_queue_and_rationale_drift_refuse_and_clean_staging() {
    for path in ["queue.json", "rationale.txt"] {
        let (fixture, _, _) = inputs(1, b"private physical rationale");
        let mut caller = StagedDrift { root: fixture.root.clone(), path, edited: false };
        let result = lifecycle_commands::respond(&options(&fixture), &mut caller);
        if cfg!(any(target_os = "linux", target_os = "macos")) {
            contract(result, ContractError::Binding);
            assert!(caller.edited);
            assert_eq!(std::fs::read(fixture.root.join(path)).unwrap(), b"changed actual captured ordinary input bytes");
        } else {
            assert!(matches!(result, Err(CommandError::Publication)));
            assert!(!caller.edited);
        }
        assert!(!fixture.root.join("result.json").exists());
        assert!(std::fs::read_dir(&fixture.root).unwrap().all(|entry|
            !entry.unwrap().file_name().to_string_lossy().starts_with(".forge-authoring-file-")));
    }
}

/// Return one real original stop only after actual output publication becomes observable.
struct PublishedStop {
    /// Actual new destination route, never a supplied successful-output capability.
    path: PathBuf,
    /// Select genuine caller failure versus cancellation.
    failed: bool,
    /// Number of actual post-publication callbacks that returned the selected first stop.
    stops: usize,
}
impl WorkControl for PublishedStop {
    /// Actual command post-output fencing can occur after complete destination bytes exist.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
        if self.path.exists() {
            self.stops += 1;
            if self.failed { return Err(WorkError::Failed(Error::invalid())); }
            return Err(WorkError::Interrupted(Interruption::CancelRequested));
        }
        Ok(())
    }
    /// The real command ledger/control preserves its first observed stop.
    fn interruption(&self) -> Option<Interruption> { None }
}

/// A post-publication original stop can accompany a complete immutable response file.
#[test]
fn genuine_response_successful_publication_final_stop_keeps_complete_output() {
    for failed in [false, true] {
        let (fixture, declaration, queue_raw) = inputs(1, b"private final-stop rationale");
        let mut caller = PublishedStop { path: fixture.root.join("result.json"), failed, stops: 0 };
        let result = lifecycle_commands::respond(&options(&fixture), &mut caller);
        if cfg!(any(target_os = "linux", target_os = "macos")) {
            let expected = if failed { ContractError::ControlFailed }
                else { ContractError::Interrupted(Interruption::CancelRequested) };
            contract(result, expected);
            assert_eq!(caller.stops, 1);
            let actual = std::fs::read(&caller.path).unwrap();
            complete(&actual, &declaration, &queue_raw, "private final-stop rationale");
            publication(lifecycle_commands::respond(&options(&fixture), &mut NoopControl));
            assert_eq!(std::fs::read(&caller.path).unwrap(), actual);
        } else {
            assert!(matches!(result, Err(CommandError::Publication)));
            assert_eq!(caller.stops, 0);
            assert!(!caller.path.exists());
        }
    }
}
