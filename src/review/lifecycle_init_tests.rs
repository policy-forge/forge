// Genuine init and clap operation controls beneath the unchanged receiver fixture.
// Private assertions never construct native owners, currentness or publication proofs.

use std::path::{Path, PathBuf};

use clap::Parser;
use serde_json::{Value, json};

use super::Fixture;
use crate::review::commands::{CommandError, CurrentOptions, InitOptions};
use crate::review::decode::{ContractError, ContractLedger};
use crate::review::{decode_v2, lifecycle_commands};
use crate::review::wire_v2::{ItemState, RecordedCurrentness, SourceKindV2};
use crate::workspace::contract::Error;
use crate::workspace::preparation::{Interruption, NoopControl, ProgressUpdate, Stage, WorkControl, WorkError, WorkResult};

/// Explicit asserted headers do not derive authority or sample an ambient time.
const QUEUE: &str = "10000000-0000-4000-8000-000000000001";
/// One actual canonical declared creation second across repeated operations.
const CREATED: &str = "2026-10-05T00:00:00Z";

/// Only ordinary policy declarations; all native operands remain absent from this file.
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

/// Persist only the ordinary asserted request beside genuine maintained native inputs.
fn policy(fixture: &Fixture) {
    std::fs::write(fixture.root.join("init.json"), serde_json::to_vec(&request()).unwrap()).unwrap();
}

/// Every route and header is explicit; the production operation accepts its sole caller.
fn options<'a>(fixture: &'a Fixture, output: &'a Path) -> InitOptions<'a> {
    InitOptions { project_root: &fixture.root, sources: Path::new("locator.json"),
        policy: Path::new("init.json"), queue_id: QUEUE, created_at: CREATED, output }
}

/// Compare exact original contract causes without changing production error traits.
fn contract<T>(result: Result<T, CommandError>, expected: ContractError) {
    assert!(matches!(result.err(), Some(CommandError::Contract(error)) if error == expected));
}

/// Inspect only ordinary actual output bytes; this decoder issues no native capability.
fn complete(fixture: &Fixture, raw: &[u8], generated: usize) {
    let mut ledger = ContractLedger::default();
    let decoded = decode_v2::decode_queue(raw, &mut ledger, &mut NoopControl).unwrap();
    let document = decoded.document();
    assert_eq!(document.queue_id, QUEUE);
    assert_eq!(document.created_at, CREATED);
    assert_eq!(document.source_pins.len(), generated + 2);
    assert_eq!(document.source_pins.iter().filter(|pin| pin.kind == SourceKindV2::LifecycleRecord).count(), 1);
    assert_eq!(document.source_pins.iter().filter(|pin| pin.kind == SourceKindV2::OpaqueSource).count(), 1);
    assert_eq!(document.source_pins.iter().filter(|pin| pin.kind == SourceKindV2::GeneratedArtifact).count(), generated);
    assert_eq!(document.items.len(), 1);
    assert_eq!(document.policies.len(), 1);
    assert_eq!(document.items[0].source_keys.len(), generated + 2);
    let source = document.source_pins.iter().find(|pin| pin.kind == SourceKindV2::OpaqueSource).unwrap();
    let record_raw = std::fs::read(fixture.root.join("record.json")).unwrap();
    let record = crate::lifecycle::record::parse(&record_raw).unwrap();
    let record_pin = document.source_pins.iter().find(|pin| pin.kind == SourceKindV2::LifecycleRecord).unwrap();
    assert_eq!(record_pin.artifact_key, "lifecycle:record");
    assert_eq!(record_pin.schema_identity.as_deref(), Some(crate::lifecycle::record::SCHEMA_VERSION));
    assert_eq!(record_pin.raw_sha256, crate::hashing::sha256_hex(&record_raw));
    assert_eq!(record_pin.byte_length, u64::try_from(record_raw.len()).unwrap());
    assert_eq!(record_pin.validation_profile, "forge.lifecycle-record-intrinsic/1");
    assert!(record_pin.native_model.is_none() && record_pin.native_root_uuid.is_none());
    let source_raw = std::fs::read(fixture.root.join(&record.policy.source.path)).unwrap();
    assert_eq!(source.artifact_key, "lifecycle:source");
    assert_eq!(source.raw_sha256, crate::hashing::sha256_hex(&source_raw));
    assert_eq!(source.byte_length, u64::try_from(source_raw.len()).unwrap());
    assert_eq!(source.validation_profile, "forge.opaque-source-bytes/1");
    assert!(source.schema_identity.is_none() && source.native_model.is_none() && source.native_root_uuid.is_none());
    let mut declared: Vec<_> = record.policy.generated_artifacts.iter().collect();
    declared.sort_by(|left, right| left.path.cmp(&right.path));
    for pin in &document.source_pins {
        assert_eq!(document.items[0].source_keys.iter().filter(|key| *key == &pin.artifact_key).count(), 1);
        if pin.kind == SourceKindV2::GeneratedArtifact {
            let ordinal: usize = pin.artifact_key.strip_prefix("lifecycle:generated:").unwrap().parse().unwrap();
            let artifact = declared[ordinal];
            let original = std::fs::read(fixture.root.join(&artifact.path)).unwrap();
            assert_eq!(pin.raw_sha256, crate::hashing::sha256_hex(&original));
            assert_eq!(pin.byte_length, u64::try_from(original.len()).unwrap());
            assert_eq!(pin.native_root_uuid.as_deref(), artifact.root_uuid.as_deref());
            assert_eq!(Some(pin.native_model.unwrap().as_str()), artifact.oscal_type.as_deref());
            assert_eq!(pin.validation_profile, "forge.lifecycle-generated-identity/1");
            assert!(pin.schema_identity.is_none());
        }
    }
    assert!(raw.ends_with(b"\n"));
    let text = std::str::from_utf8(raw).unwrap();
    for private in ["private native title", "private native rationale", "source.bin", "generated/", "native\\u0000policy"] {
        assert!(!text.contains(private));
    }
}

/// Actual native publication creates a complete new queue, or retains its platform refusal.
#[test]
fn genuine_init_complete_opaque_and_generated_native_pins_preserve_originals() {
    for (generated, shared, empty) in [(0, false, true), (6, false, false), (6, true, false)] {
        let fixture = Fixture::new(generated, shared, empty);
        policy(&fixture);
        let before = std::fs::read(fixture.root.join("record.json")).unwrap();
        let result = lifecycle_commands::init(&options(&fixture, Path::new("queue.json")), &mut NoopControl);
        if cfg!(any(target_os = "linux", target_os = "macos")) {
            result.unwrap();
            complete(&fixture, &std::fs::read(fixture.root.join("queue.json")).unwrap(), generated);
        } else {
            assert!(matches!(result, Err(CommandError::Publication)));
            assert!(!fixture.root.join("queue.json").exists());
        }
        assert_eq!(std::fs::read(fixture.root.join("record.json")).unwrap(), before);
    }
}

/// Separate real invocations preserve exact deterministic bytes at different new destinations.
#[test]
fn genuine_init_exact_deterministic_bytes_ignore_only_new_destination() {
    let fixture = Fixture::new(6, false, false);
    policy(&fixture);
    let first = lifecycle_commands::init(&options(&fixture, Path::new("queue-a.json")), &mut NoopControl);
    let second = lifecycle_commands::init(&options(&fixture, Path::new("queue-b.json")), &mut NoopControl);
    if cfg!(any(target_os = "linux", target_os = "macos")) {
        first.unwrap(); second.unwrap();
        let raw = std::fs::read(fixture.root.join("queue-a.json")).unwrap();
        assert_eq!(raw, std::fs::read(fixture.root.join("queue-b.json")).unwrap());
        complete(&fixture, &raw, 6);
    } else {
        assert!(matches!(first, Err(CommandError::Publication)));
        assert!(matches!(second, Err(CommandError::Publication)));
        assert!(!fixture.root.join("queue-a.json").exists() && !fixture.root.join("queue-b.json").exists());
    }
}

/// Unavailable originals and valid-input publication refusal stay distinct; no early absence proof exists.
#[test]
fn genuine_init_existing_output_preserves_bytes_and_distinct_input_failures() {
    let fixture = Fixture::new(1, false, false);
    policy(&fixture);
    std::fs::write(fixture.root.join("queue.json"), b"existing original destination").unwrap();
    assert!(matches!(lifecycle_commands::init(&options(&fixture, Path::new("queue.json")), &mut NoopControl),
        Err(CommandError::Publication)));
    assert_eq!(std::fs::read(fixture.root.join("queue.json")).unwrap(), b"existing original destination");
    std::fs::remove_file(fixture.root.join("locator.json")).unwrap();
    contract(lifecycle_commands::init(&options(&fixture, Path::new("queue.json")), &mut NoopControl), ContractError::Binding);
    assert_eq!(std::fs::read(fixture.root.join("queue.json")).unwrap(), b"existing original destination");
}

/// Closed ordinary assertions, full declared native closure and actual portable routes are enforced.
#[test]
fn genuine_init_private_native_and_escape_refusals_never_emit_output() {
    for case in 0..4 {
        let fixture = Fixture::new(6, false, false);
        policy(&fixture);
        let expected = if case == 2 { ContractError::Binding } else { ContractError::Invalid };
        let mut selected = options(&fixture, Path::new("queue.json"));
        match case {
            0 => { let mut value = request(); value["source_pins"] = json!([]);
                std::fs::write(fixture.root.join("init.json"), serde_json::to_vec(&value).unwrap()).unwrap(); },
            1 => { let mut value = request(); value["items"][0].as_object_mut().unwrap().remove("due_at");
                std::fs::write(fixture.root.join("init.json"), serde_json::to_vec(&value).unwrap()).unwrap(); },
            2 => { let path = fixture.root.join("locator.json");
                let mut value: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
                value["generated_artifacts"].as_array_mut().unwrap().pop();
                std::fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap(); },
            _ => selected.policy = Path::new("../escaped-init.json"),
        }
        contract(lifecycle_commands::init(&selected, &mut NoopControl), expected);
        assert!(!fixture.root.join("queue.json").exists());
    }
}

/// A genuine observed staged Queue selects either a physical edit or original caller stop.
enum StagedAction {
    /// Actual registered original file path, never a currentness assertion.
    Drift(PathBuf),
    /// Exact real `WorkError` cause delivered by the original caller callback.
    Stop(bool),
}

/// Private staging observation establishes actual reach only, not a race/atomic-snapshot proof.
struct StagedCaller {
    /// Actual canonical private fixture root survives the complete command invocation.
    root: PathBuf,
    /// The sole action occurs after observing the publisher's complete actual staged bytes.
    action: Option<StagedAction>,
    /// Number of complete staged files genuinely observed by this original controller.
    observed: usize,
}
impl WorkControl for StagedCaller {
    /// The maintained publisher enters its true pre-rename guarded hook after complete staging.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
        if self.action.is_none() { return Ok(()); }
        for entry in std::fs::read_dir(&self.root).unwrap() {
            let entry = entry.unwrap();
            if !entry.file_name().to_string_lossy().starts_with(".forge-authoring-file-") { continue; }
            let Ok(bytes) = std::fs::read(entry.path().join("file")) else { continue; };
            if !bytes.ends_with(b"\n") { continue; }
            let value: Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(value["schema_version"], "forge.review-queue/2");
            assert_eq!(value["source_pins"].as_array().unwrap().len(), 8);
            assert_eq!(value["items"][0]["domain"], "lifecycle-policy-version");
            self.observed += 1;
            match self.action.take().unwrap() {
                StagedAction::Drift(path) => std::fs::write(self.root.join(path), b"actual changed captured original").unwrap(),
                StagedAction::Stop(failed) => return Err(if failed { WorkError::Failed(Error::invalid()) }
                    else { WorkError::Interrupted(Interruption::CancelRequested) }),
            }
            break;
        }
        Ok(())
    }
    /// All stops originate in checkpoint; ordinary physical drift remains a native observation.
    fn interruption(&self) -> Option<Interruption> { None }
}

/// Neither native Source nor captured private-policy drift can publish after complete staging.
#[test]
fn genuine_init_guard_rechecks_native_and_auxiliary_originals_after_staging() {
    for target in ["source.bin", "init.json"] {
        let fixture = Fixture::new(6, false, false);
        policy(&fixture);
        let mut caller = StagedCaller { root: fixture.root.clone(),
            action: Some(StagedAction::Drift(PathBuf::from(target))), observed: 0 };
        let result = lifecycle_commands::init(&options(&fixture, Path::new("queue.json")), &mut caller);
        if cfg!(any(target_os = "linux", target_os = "macos")) {
            contract(result, ContractError::Binding); assert_eq!(caller.observed, 1);
            assert_eq!(std::fs::read(fixture.root.join(target)).unwrap(), b"actual changed captured original");
        } else { assert!(matches!(result, Err(CommandError::Publication))); assert_eq!(caller.observed, 0); }
        assert!(!fixture.root.join("queue.json").exists());
        assert!(std::fs::read_dir(&fixture.root).unwrap().all(|entry|
            !entry.unwrap().file_name().to_string_lossy().starts_with(".forge-authoring-file-")));
    }
}

/// Real original cancellation/failure at the complete staged hook prevents native rename.
#[test]
fn genuine_init_staged_original_caller_stop_refuses_complete_output() {
    for failed in [false, true] {
        let fixture = Fixture::new(6, false, false);
        policy(&fixture);
        let mut caller = StagedCaller { root: fixture.root.clone(), action: Some(StagedAction::Stop(failed)), observed: 0 };
        let result = lifecycle_commands::init(&options(&fixture, Path::new("queue.json")), &mut caller);
        if cfg!(any(target_os = "linux", target_os = "macos")) {
            contract(result, stop(failed)); assert_eq!(caller.observed, 1);
        } else { assert!(matches!(result, Err(CommandError::Publication))); assert_eq!(caller.observed, 0); }
        assert!(!fixture.root.join("queue.json").exists());
        assert!(std::fs::read_dir(&fixture.root).unwrap().all(|entry|
            !entry.unwrap().file_name().to_string_lossy().starts_with(".forge-authoring-file-")));
    }
}

/// Observe actual callback counts from the complete real operation, without altering budgets.
struct CountedCaller {
    /// Actual original callback count for one operation invocation.
    calls: usize,
    /// One calibrated real callback, or no requested stop during baseline observation.
    at: Option<usize>,
    /// Choose only the actual original `WorkError` cause.
    failed: bool,
    /// Number of real requested stops reached by this original controller.
    stops: usize,
}
impl WorkControl for CountedCaller {
    /// The callback receives every real command stage and preserves the exact requested first stop.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
        self.calls += 1;
        if self.at == Some(self.calls) {
            self.stops += 1;
            return Err(if self.failed { WorkError::Failed(Error::invalid()) }
                else { WorkError::Interrupted(Interruption::CancelRequested) });
        }
        Ok(())
    }
    /// No deadline, interruption or controller replacement is fabricated here.
    fn interruption(&self) -> Option<Interruption> { None }
}

/// Exact original first-stop expectations use the actual common contract variants.
fn stop(failed: bool) -> ContractError {
    if failed { ContractError::ControlFailed }
    else { ContractError::Interrupted(Interruption::CancelRequested) }
}

/// A failed real private decode still reaches the final unconditional original command postfence.
#[test]
fn genuine_init_ordinary_private_failure_observes_calibrated_final_stop() {
    let fixture = Fixture::new(0, false, true);
    std::fs::write(fixture.root.join("init.json"), b"{").unwrap();
    let mut baseline = CountedCaller { calls: 0, at: None, failed: false, stops: 0 };
    contract(lifecycle_commands::init(&options(&fixture, Path::new("queue.json")), &mut baseline), ContractError::Invalid);
    assert!(baseline.calls > 1);
    for failed in [false, true] {
        let mut caller = CountedCaller { calls: 0, at: Some(baseline.calls), failed, stops: 0 };
        contract(lifecycle_commands::init(&options(&fixture, Path::new("queue.json")), &mut caller), stop(failed));
        assert_eq!(caller.calls, baseline.calls); assert_eq!(caller.stops, 1);
        assert!(!fixture.root.join("queue.json").exists());
    }
}

/// A genuine overbound private original wins before an armed later caller stop, with fixed caps.
#[test]
fn genuine_init_original_capacity_prevents_later_control_probe() {
    let fixture = Fixture::new(0, false, true);
    std::fs::write(fixture.root.join("init.json"), vec![b' '; 1_048_577]).unwrap();
    let mut baseline = CountedCaller { calls: 0, at: None, failed: false, stops: 0 };
    contract(lifecycle_commands::init(&options(&fixture, Path::new("queue.json")), &mut baseline), ContractError::Capacity);
    assert!(baseline.calls > 0);
    let mut caller = CountedCaller { calls: 0, at: Some(baseline.calls + 1), failed: true, stops: 0 };
    contract(lifecycle_commands::init(&options(&fixture, Path::new("queue.json")), &mut caller), ContractError::Capacity);
    assert_eq!(caller.calls, baseline.calls); assert_eq!(caller.stops, 0);
    assert!(!fixture.root.join("queue.json").exists());
}

/// Parse the actual maintained clap grammar and invoke its actual execute dispatch.
fn cli(fixture: &Fixture, command: &str, tail: &[&str]) -> crate::cli::Cli {
    let mut argv = vec!["forge".to_string(), "review".to_string(), "lifecycle".to_string(),
        command.to_string(), "--project-root".to_string(), fixture.root.to_string_lossy().into_owned()];
    argv.extend(tail.iter().map(|value| (*value).to_string()));
    crate::cli::Cli::try_parse_from(argv).unwrap()
}

/// The four real routes consume native-created Queue/Response bytes and preserve exact duplicate counts.
/// Unsupported publication is executed and refused; no positive full workflow is claimed there.
#[test]
fn genuine_clap_execute_init_respond_merge_status_complete_workflow() {
    let fixture = Fixture::new(6, false, false);
    policy(&fixture);
    let before = std::fs::read(fixture.root.join("record.json")).unwrap();
    let init = cli(&fixture, "init", &["--sources", "locator.json", "--policy", "init.json",
        "--queue-id", QUEUE, "--created-at", CREATED, "--output", "queue.json"]);
    let respond = cli(&fixture, "respond", &["--queue", "queue.json", "--item-key", "item-1",
        "--reviewer-key", "asserted-reviewer", "--reviewer-role", "review", "--disposition", "approve",
        "--responded-at", "2026-10-05T00:00:01Z", "--response-id", "20000000-0000-4000-8000-000000000001",
        "--rationale-file", "rationale.txt", "--output", "response.json"]);
    let merge = cli(&fixture, "merge", &["--sources", "locator.json", "--queue", "queue.json",
        "--response", "response.json", "--response", "response.json", "--as-of", "2026-10-05T00:00:02Z",
        "--output", "dispositions.json"]);
    let status = cli(&fixture, "status", &["--sources", "locator.json", "--queue", "queue.json",
        "--response", "response.json", "--response", "response.json", "--as-of", "2026-10-05T00:00:02Z"]);
    let initialized = crate::cli::execute(&init);
    if !cfg!(any(target_os = "linux", target_os = "macos")) {
        match initialized {
            Err(crate::error::ForgeError::Io(error)) => assert_eq!(error.to_string(), "review output publication failed"),
            _ => panic!("unsupported publisher must retain its exact ordinary CLI diagnostic"),
        }
        assert!(!fixture.root.join("queue.json").exists());
        assert_eq!(std::fs::read(fixture.root.join("record.json")).unwrap(), before);
        return;
    }
    initialized.unwrap();
    complete(&fixture, &std::fs::read(fixture.root.join("queue.json")).unwrap(), 6);
    std::fs::write(fixture.root.join("rationale.txt"), b"private asserted review rationale").unwrap();
    crate::cli::execute(&respond).unwrap(); crate::cli::execute(&merge).unwrap();
    let raw = std::fs::read(fixture.root.join("dispositions.json")).unwrap();
    let mut ledger = ContractLedger::default();
    let decoded = decode_v2::decode_dispositions(&raw, &mut ledger, &mut NoopControl).unwrap();
    assert_eq!(decoded.document().currentness, RecordedCurrentness::RecordedCurrent);
    assert_eq!(decoded.document().items[0].state, ItemState::QuorumMet);
    assert_eq!(decoded.document().source_pins.len(), 8);
    assert_eq!(decoded.document().counts.response_files, 2);
    assert_eq!(decoded.document().counts.unique_responses, 1);
    assert_eq!(decoded.document().counts.exact_duplicates, 1);
    let responses = vec![PathBuf::from("response.json"), PathBuf::from("response.json")];
    let mut expected = Vec::new();
    lifecycle_commands::status(&CurrentOptions { project_root: &fixture.root, sources: Path::new("locator.json"),
        queue: Path::new("queue.json"), responses: &responses, as_of: "2026-10-05T00:00:02Z" }, &mut expected, &mut NoopControl).unwrap();
    assert_eq!(expected, raw);
    crate::cli::execute(&status).unwrap();
    assert!(!raw.windows(b"private asserted review rationale".len()).any(|row| row == b"private asserted review rationale"));
    assert_eq!(std::fs::read(fixture.root.join("record.json")).unwrap(), before);
}
