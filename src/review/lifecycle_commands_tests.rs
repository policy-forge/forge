// Genuine current merge/status sink controls beneath the real captured binding fixture.
// Fixture input preparation grants no detached currentness/native owner or publication right.

use std::cell::RefCell;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use super::{begin, hold, queue, raw, response};
use super::super::Fixture;
use crate::review::commands::{CommandError, CurrentOptions};
use crate::review::decode::{ContractError, ContractLedger};
use crate::review::{decode_v2, lifecycle_commands};
use crate::review::wire_v2::{ItemState, RecordedCurrentness, ResponseClassification};
use crate::workspace::contract::Error;
use crate::workspace::preparation::{Interruption, NoopControl, ProgressUpdate, Stage, WorkControl, WorkError, WorkResult};

/// Create only real input files using the actual native pending/registration fixture core.
/// All preparation owners drop before the command accepts its own single invocation control.
fn inputs(count: usize, empty: bool) -> (Fixture, Vec<PathBuf>) {
    let fixture = Fixture::new(count, false, empty);
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let (capture, pending) = begin(&fixture, &mut ledger, &mut caller);
    let declaration = queue(&pending, &mut ledger, &mut caller);
    let queue_raw = raw(&declaration);
    let current = response(&declaration, &queue_raw, false);
    let foreign = response(&declaration, &queue_raw, true);
    let owner = hold(&fixture, capture, pending, Some(&queue_raw),
        &[("response.json", &current), ("foreign.json", &foreign), ("response.json", &current)],
        &mut ledger, &mut caller);
    drop(owner);
    (fixture, vec!["response.json".into(), "foreign.json".into(), "response.json".into()])
}

/// Complete explicit operation inputs use the actual closed /2 queue and every occurrence.
fn options<'a>(fixture: &'a Fixture, responses: &'a [PathBuf]) -> CurrentOptions<'a> {
    CurrentOptions { project_root: &fixture.root, sources: Path::new("locator.json"),
        queue: Path::new("queue.json"), responses, as_of: "2026-10-05T00:00:02Z" }
}

/// Safe exact contract error comparison without widening production `CommandError` derives.
fn contract<T>(result: Result<T, CommandError>, expected: ContractError) {
    let actual = result.err();
    assert!(matches!(actual, Some(CommandError::Contract(error)) if error == expected), "expected {expected:?}, actual {actual:?}");
}

/// Compare the fixed native publication category without exposing private publisher errors.
fn publication<T>(result: Result<T, CommandError>) {
    let actual = result.err();
    assert!(matches!(actual, Some(CommandError::Publication)), "expected Publication, actual {actual:?}");
}

/// Complete actual minimized current bytes retain the full native/source/response denominators.
fn complete(raw: &[u8], pins: usize) {
    let mut ledger = ContractLedger::default();
    let decoded = decode_v2::decode_dispositions(raw, &mut ledger, &mut NoopControl).unwrap();
    let document = decoded.document();
    assert_eq!(document.currentness, RecordedCurrentness::RecordedCurrent);
    assert!(document.closure_generation.as_ref().is_some_and(|hash| hash.len() == 64));
    assert_eq!(document.source_pins.len(), pins);
    assert_eq!(document.counts.items, 1);
    assert_eq!(document.counts.response_files, 3);
    assert_eq!(document.counts.unique_responses, 2);
    assert_eq!(document.counts.exact_duplicates, 1);
    assert_eq!(document.responses.len(), 2);
    assert_eq!(document.responses.iter().filter(|row| row.classification == ResponseClassification::Current).count(), 1);
    assert_eq!(document.responses.iter().filter(|row| row.classification == ResponseClassification::Foreign).count(), 1);
    assert_eq!(document.items[0].state, ItemState::QuorumMet);
    assert!(document.items[0].reason_codes.iter().any(|reason| reason == "captured-current-sources"));
    assert!(!document.items[0].reason_codes.iter().any(|reason| reason == "currentness-unverified"));
    assert!(raw.ends_with(b"\n"));
    assert!(!raw.windows(b"private review evidence".len()).any(|part| part == b"private review evidence"));
    let value: serde_json::Value = serde_json::from_slice(raw).unwrap();
    for row in value["responses"].as_array().unwrap() { assert!(row.get("rationale").is_none()); }
}

/// Both empty opaque and all six generated identities produce complete real status sink bytes.
#[test]
fn genuine_status_sink_complete_current_duplicate_foreign_and_native_inputs_unchanged() {
    for (count, empty) in [(0, true), (6, false)] {
        let (fixture, responses) = inputs(count, empty);
        let before_record = std::fs::read(fixture.root.join("record.json")).unwrap();
        let mut output = Vec::new();
        lifecycle_commands::status(&options(&fixture, &responses), &mut output, &mut NoopControl).unwrap();
        complete(&output, count + 2);
        assert_eq!(std::fs::read(fixture.root.join("record.json")).unwrap(), before_record);
        assert!(!fixture.root.join("result.json").exists());
    }
}

/// Genuine guarded file publication equals status data on supported native platforms.
/// Other platforms must keep their real Publication refusal and absent destination.
#[test]
fn genuine_merge_native_no_replace_and_platform_refusal_preserve_inputs() {
    let (fixture, responses) = inputs(6, false);
    let before_record = std::fs::read(fixture.root.join("record.json")).unwrap();
    let mut status_bytes = Vec::new();
    lifecycle_commands::status(&options(&fixture, &responses), &mut status_bytes, &mut NoopControl).unwrap();
    let published = lifecycle_commands::merge(&options(&fixture, &responses), Path::new("result.json"), &mut NoopControl);
    if cfg!(any(target_os = "linux", target_os = "macos")) {
        published.unwrap();
        let actual = std::fs::read(fixture.root.join("result.json")).unwrap();
        complete(&actual, 8);
        assert_eq!(actual, status_bytes);
        publication(lifecycle_commands::merge(&options(&fixture, &responses), Path::new("result.json"), &mut NoopControl));
        assert_eq!(std::fs::read(fixture.root.join("result.json")).unwrap(), actual);
    } else {
        assert!(matches!(published, Err(CommandError::Publication)));
        assert!(!fixture.root.join("result.json").exists());
    }
    assert_eq!(std::fs::read(fixture.root.join("record.json")).unwrap(), before_record);
}

/// Missing inputs return Binding after lexical output namespace reservation.
/// Reservation performs no destination IO; the existing bytes and absence of staging are retained.
#[test]
fn genuine_reserved_output_namespace_missing_inputs_preserve_existing_bytes() {
    let (fixture, responses) = inputs(1, false);
    std::fs::write(fixture.root.join("result.json"), b"existing destination bytes").unwrap();
    std::fs::remove_file(fixture.root.join("queue.json")).unwrap();
    std::fs::remove_file(fixture.root.join("locator.json")).unwrap();
    contract(lifecycle_commands::merge(&options(&fixture, &responses), Path::new("result.json"), &mut NoopControl), ContractError::Binding);
    assert_eq!(std::fs::read(fixture.root.join("result.json")).unwrap(), b"existing destination bytes");
    assert!(std::fs::read_dir(&fixture.root).unwrap().all(|entry|
        !entry.unwrap().file_name().to_string_lossy().starts_with(".forge-authoring-file-")));
}

/// Actual original callback observations, shared solely with the real caller-owned writer.
#[derive(Default)]
struct SinkState {
    /// Actual monotonically observed calls throughout one command invocation.
    calls: usize,
    /// Arm one exact post-write callback without changing the ledger or accepted deadline.
    next_stop: Option<bool>,
    /// Number of actual requested stops observed by the original caller.
    stops: usize,
}

/// The sole original caller is forwarded through the command's real `ReviewControl`.
struct SinkCaller(Rc<RefCell<SinkState>>);
impl WorkControl for SinkCaller {
    /// The next actual callback after sink completion returns its precise original cause.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
        let mut state = self.0.borrow_mut();
        state.calls += 1;
        if let Some(failed) = state.next_stop.take() {
            state.stops += 1;
            if failed { return Err(WorkError::Failed(Error::invalid())); }
            return Err(WorkError::Interrupted(Interruption::CancelRequested));
        }
        Ok(())
    }
    /// The genuine ledger retains any first observed stop; no loaded proof is returned.
    fn interruption(&self) -> Option<Interruption> { None }
}

/// Real writer retains complete or partial bytes and can arm its next original callback.
struct Sink {
    /// Actual bytes accepted by Write, inspected after the real command result.
    bytes: Vec<u8>,
    /// Force a genuine second-write IO error after a positive partial first write.
    partial_error: bool,
    /// Select a real final cancellation/control failure or ordinary unarmed continuation.
    arm: Option<bool>,
    /// Shared actual original caller observations, never a replacement controller.
    state: Rc<RefCell<SinkState>>,
}
impl Write for Sink {
    /// Actual `write_all` must retain the accepted prefix before the next IO failure.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.partial_error && self.bytes.is_empty() {
            let count = bytes.len().min(13);
            self.bytes.extend_from_slice(&bytes[..count]);
            return Ok(count);
        }
        if let Some(failed) = self.arm.take() { self.state.borrow_mut().next_stop = Some(failed); }
        if self.partial_error { return Err(io::Error::other("private sink failure detail")); }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    /// The retained command's `write_all` does not invent a flush/publication operation.
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}

/// Exact safe original contract stop selected by a genuine caller checkpoint.
fn selected_stop(failed: bool) -> ContractError {
    if failed { ContractError::ControlFailed }
    else { ContractError::Interrupted(Interruption::CancelRequested) }
}

/// A partial real writer keeps its prefix and ordinary safe Output diagnostic.
#[test]
fn genuine_partial_status_writer_error_retains_prefix_and_safe_output_diagnostic() {
    let (fixture, responses) = inputs(1, false);
    let state = Rc::new(RefCell::new(SinkState::default()));
    let mut caller = SinkCaller(Rc::clone(&state));
    let mut sink = Sink { bytes: Vec::new(), partial_error: true, arm: None, state: Rc::clone(&state) };
    let error = lifecycle_commands::status(&options(&fixture, &responses), &mut sink, &mut caller).unwrap_err();
    assert!(matches!(error, CommandError::Output));
    assert_eq!(error.to_string(), "review status output failed");
    assert_eq!(sink.bytes.len(), 13);
    assert!(sink.bytes.starts_with(b"{\"schema_ver"));
    assert!(state.borrow().calls > 0);
    assert_eq!(state.borrow().stops, 0);
    assert!(!fixture.root.join("result.json").exists());
}

/// Ordinary partial Output failure is unconditionally postfenced by the actual caller.
#[test]
fn genuine_partial_status_output_failure_observes_final_original_stop() {
    for failed in [false, true] {
        let (fixture, responses) = inputs(1, false);
        let state = Rc::new(RefCell::new(SinkState::default()));
        let mut caller = SinkCaller(Rc::clone(&state));
        let mut sink = Sink { bytes: Vec::new(), partial_error: true, arm: Some(failed), state: Rc::clone(&state) };
        contract(lifecycle_commands::status(&options(&fixture, &responses), &mut sink, &mut caller), selected_stop(failed));
        assert_eq!(sink.bytes.len(), 13);
        assert_eq!(state.borrow().stops, 1);
        assert!(state.borrow().next_stop.is_none());
        assert!(!fixture.root.join("result.json").exists());
    }
}

/// Successful write can emit a full record before a later original post-output stop.
#[test]
fn genuine_successful_status_output_observes_final_original_stop_with_complete_bytes() {
    for failed in [false, true] {
        let (fixture, responses) = inputs(6, false);
        let state = Rc::new(RefCell::new(SinkState::default()));
        let mut caller = SinkCaller(Rc::clone(&state));
        let mut sink = Sink { bytes: Vec::new(), partial_error: false, arm: Some(failed), state: Rc::clone(&state) };
        contract(lifecycle_commands::status(&options(&fixture, &responses), &mut sink, &mut caller), selected_stop(failed));
        complete(&sink.bytes, 8);
        assert_eq!(state.borrow().stops, 1);
        assert!(state.borrow().next_stop.is_none());
        assert!(!fixture.root.join("result.json").exists());
    }
}

/// Observe complete actual native staging and change a captured Source before its final fence.
/// This explicit private staging-name observation is a component control, not a race proof.
struct StagedSourceDrift {
    /// Actual private fixture root; no caller currentness or capture claim is supplied.
    root: PathBuf,
    /// Complete actual staged record observed before mutating the real native Source.
    staged_complete: bool,
    /// Perform the physical byte edit once while the genuine capture stays live.
    edited: bool,
}
impl WorkControl for StagedSourceDrift {
    /// The maintained publisher calls its real guarded hook after staging complete bytes.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
        if !self.edited {
            for entry in std::fs::read_dir(&self.root).unwrap() {
                let entry = entry.unwrap();
                if !entry.file_name().to_string_lossy().starts_with(".forge-authoring-file-") { continue; }
                let staged = entry.path().join("file");
                let Ok(bytes) = std::fs::read(staged) else { continue; };
                if !bytes.ends_with(b"\n") { continue; }
                let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                assert_eq!(value["schema_version"], "forge.review-dispositions/2");
                assert_eq!(value["counts"]["response_files"], 3);
                self.staged_complete = true;
                std::fs::write(self.root.join("source.bin"), b"actual changed source after complete staging").unwrap();
                self.edited = true;
                break;
            }
        }
        Ok(())
    }
    /// Real physical drift is detected by capture verification, not by an invented stop.
    fn interruption(&self) -> Option<Interruption> { None }
}

/// Genuine staged complete native output cannot publish after actual physical Source drift.
/// Unsupported publishers remain truly refused without skipping this control declaration.
#[test]
fn genuine_merge_guarded_source_drift_refuses_publication_and_cleans_actual_staging() {
    let (fixture, responses) = inputs(1, false);
    let mut caller = StagedSourceDrift { root: fixture.root.clone(), staged_complete: false, edited: false };
    let result = lifecycle_commands::merge(&options(&fixture, &responses), Path::new("result.json"), &mut caller);
    if cfg!(any(target_os = "linux", target_os = "macos")) {
        contract(result, ContractError::Binding);
        assert!(caller.staged_complete && caller.edited);
        assert_eq!(std::fs::read(fixture.root.join("source.bin")).unwrap(), b"actual changed source after complete staging");
    } else {
        assert!(matches!(result, Err(CommandError::Publication)));
        assert!(!caller.staged_complete && !caller.edited);
    }
    assert!(!fixture.root.join("result.json").exists());
    assert!(std::fs::read_dir(&fixture.root).unwrap().all(|entry|
        !entry.unwrap().file_name().to_string_lossy().starts_with(".forge-authoring-file-")));
}
