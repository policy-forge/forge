// Genuine command controls included beneath the existing native fixture owner.
use super::*;
use crate::review::authoring_commands as commands_v3;
use crate::review::commands::{CommandError, CurrentOptions, InitOptions, RespondOptions};
use crate::review::wire::Disposition;
use crate::workspace::preparation::{Interruption, ProgressUpdate, Stage, WorkError, WorkResult};
use std::io::Write;

/// Stable asserted Queue revision, independent from the native project identity.
const QUEUE_ID: &str = "33333333-4444-4555-8666-777777777777";
/// Explicit canonical review creation time, distinct from the native plan date.
const CREATED: &str = "2026-10-01T00:00:00Z";
/// Explicit policy evaluation after ordinary fixture responses.
const EVALUATED: &str = "2026-10-01T00:00:03Z";

/// Persist an ordinary private asserted request, never caller source/hash/currentness fields.
fn policy(f: &Fixture, due: Option<&str>) {
    write(
        &f.root.join("policy.json"),
        &json!({
            "schema_version":"forge.review-authoring-plan-init/1",
            "roles":[{"key":"approver"}],
            "reviewers":[{"key":"alice","role_keys":["approver"]},
                {"key":"bob","role_keys":["approver"]}],
            "policies":[{"key":"review","seats":[{"role_key":"approver","count":1}],
                "substitutions":[],"abstention_rule":"nonapproving",
                "empty_abstention_reasons":["not-qualified"],"author_separation":"declared-keys"}],
            "items":[{"key":"whole-plan","policy_key":"review","author_keys":["author"],
                "assignments":[{"reviewer_key":"alice","role_key":"approver"},
                    {"reviewer_key":"bob","role_key":"approver"}],"due_at":due}]
        }),
    );
}

/// Prepare actual legacy native fixtures and full saved-plan originals before a real Init.
fn setup(profile: bool, mapping: bool, due: Option<&str>) -> Fixture {
    let f = Fixture::new(profile, mapping);
    originals(&f);
    policy(&f, due);
    f
}

/// The actual production Init creates one new Queue using complete native capture/output.
fn create(f: &Fixture, caller: &mut dyn WorkControl) -> Result<(), CommandError> {
    commands_v3::init(
        &InitOptions {
            project_root: &f.root,
            sources: Path::new("locator.json"),
            policy: Path::new("policy.json"),
            queue_id: QUEUE_ID,
            created_at: CREATED,
            output: Path::new("queue.json"),
        },
        caller,
    )
}

/// Publish a real immutable assertion against the actual generated Queue allocation.
fn respond(
    f: &Fixture,
    key: &str,
    disposition: Disposition,
    when: &str,
    output: &str,
    caller: &mut dyn WorkControl,
) -> Result<(), CommandError> {
    std::fs::write(f.root.join("rationale.txt"), "private reviewer evidence").unwrap();
    let response_id = if key == "bob" {
        "22222222-1111-4111-8111-111111111111"
    } else {
        "11111111-1111-4111-8111-111111111111"
    };
    commands_v3::respond(
        &RespondOptions {
            project_root: &f.root,
            queue: Path::new("queue.json"),
            rationale_file: Path::new("rationale.txt"),
            item_key: "whole-plan",
            reviewer_key: key,
            reviewer_role: "approver",
            disposition,
            responded_at: when,
            response_id,
            abstention_reason: if disposition == Disposition::Abstain {
                Some("not-qualified")
            } else {
                None
            },
            supersedes: None,
            output: Path::new(output),
        },
        caller,
    )
}

/// Supply every exact response occurrence explicitly; repeat paths remain occurrences.
fn current_options<'a>(f: &'a Fixture, responses: &'a [PathBuf]) -> CurrentOptions<'a> {
    CurrentOptions {
        project_root: &f.root,
        sources: Path::new("locator.json"),
        queue: Path::new("queue.json"),
        responses,
        as_of: EVALUATED,
    }
}

/// Decode real status output for assertions only, after complete production execution.
fn status(f: &Fixture, responses: &[PathBuf]) -> Value {
    let mut output = Vec::new();
    commands_v3::status(&current_options(f, responses), &mut output, &mut NoopControl).unwrap();
    assert!(output.ends_with(b"}\n"));
    serde_json::from_slice(&output).unwrap()
}

/// Observe the exact ordinary command error without requiring Debug on private owners.
fn command_error<T>(result: Result<T, CommandError>) -> CommandError {
    match result {
        Ok(value) => {
            drop(value);
            panic!("expected real command refusal")
        }
        Err(error) => error,
    }
}

/// Catalog Init retains exact full native pins and distinct canonical review time.
#[test]
fn catalog_init_produces_complete_minimized_queue_from_actual_native_plan() {
    let f = setup(false, false, None);
    create(&f, &mut NoopControl).unwrap();
    let raw = std::fs::read(f.root.join("queue.json")).unwrap();
    let q: Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(q["schema_version"], "forge.review-queue/3");
    assert_eq!(q["created_at"], CREATED);
    assert_eq!(q["items"][0]["domain"], "authoring-plan");
    assert_eq!(q["source_pins"].as_array().unwrap().len(), 7);
    assert_eq!(q["items"][0]["source_keys"].as_array().unwrap().len(), 7);
    assert!(
        q["source_pins"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["artifact_key"] == "authoring:clause:0")
    );
    let text = std::str::from_utf8(&raw).unwrap();
    for private in
        ["project.json", "native-project", "Private access title", "Private native interview"]
    {
        assert!(!text.contains(private));
    }
}

/// Profile Init includes its explicit resolved companion and every actual Mapping original.
#[test]
fn profile_mapping_init_retains_complete_source_roster() {
    let f = setup(true, true, None);
    create(&f, &mut NoopControl).unwrap();
    let q: Value =
        serde_json::from_slice(&std::fs::read(f.root.join("queue.json")).unwrap()).unwrap();
    let keys: Vec<&str> = q["source_pins"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["artifact_key"].as_str().unwrap())
        .collect();
    assert_eq!(keys.len(), 9);
    assert!(keys.contains(&"authoring:resolved"));
    assert!(keys.contains(&"authoring:mapping:0"));
    assert!(keys.windows(2).all(|pair| pair[0] < pair[1]));
}

/// Actual Init, Respond, Merge and Status agree on whole current recorded output.
#[test]
fn all_four_native_commands_preserve_current_quorum_and_private_rationale() {
    let f = setup(false, false, None);
    create(&f, &mut NoopControl).unwrap();
    respond(
        &f,
        "alice",
        Disposition::Approve,
        "2026-10-01T00:00:01Z",
        "alice.json",
        &mut NoopControl,
    )
    .unwrap();
    let paths = vec![PathBuf::from("alice.json")];
    let recorded = status(&f, &paths);
    assert_eq!(recorded["currentness"], "recorded-current");
    assert_eq!(recorded["items"][0]["state"], "quorum-met");
    assert_eq!(recorded["counts"]["response_files"], 1);
    assert!(!recorded.to_string().contains("private reviewer evidence"));
    commands_v3::merge(&current_options(&f, &paths), Path::new("merged.json"), &mut NoopControl)
        .unwrap();
    let merged: Value =
        serde_json::from_slice(&std::fs::read(f.root.join("merged.json")).unwrap()).unwrap();
    assert_eq!(merged, recorded);
}

/// An empty explicit Response list retains the real assigned item without inventing approval.
#[test]
fn no_response_current_status_is_assigned_without_approval() {
    let f = setup(false, false, None);
    create(&f, &mut NoopControl).unwrap();
    let result = status(&f, &[]);
    assert_eq!(result["items"][0]["state"], "assigned");
    assert_eq!(result["items"][0]["blocking"], false);
    assert_eq!(result["counts"]["unique_responses"], 0);
    assert_eq!(result["items"][0]["met_seats"].as_array().unwrap().as_slice(), &[] as &[Value]);
}

/// Every repeated actual occurrence is counted without supplying another approval key.
#[test]
fn duplicate_response_paths_preserve_occurrence_denominators() {
    let f = setup(false, false, None);
    create(&f, &mut NoopControl).unwrap();
    respond(
        &f,
        "alice",
        Disposition::Approve,
        "2026-10-01T00:00:01Z",
        "alice.json",
        &mut NoopControl,
    )
    .unwrap();
    let result = status(&f, &[PathBuf::from("alice.json"), PathBuf::from("alice.json")]);
    assert_eq!(result["counts"]["response_files"], 2);
    assert_eq!(result["counts"]["unique_responses"], 1);
    assert_eq!(result["counts"]["exact_duplicates"], 1);
    assert_eq!(result["items"][0]["met_seats"].as_array().unwrap().len(), 1);
}

/// Current decisive dissent survives actual command publication and blocks quorum.
#[test]
fn complete_current_dissent_blocks_native_command_quorum() {
    let f = setup(false, false, None);
    create(&f, &mut NoopControl).unwrap();
    respond(
        &f,
        "alice",
        Disposition::Approve,
        "2026-10-01T00:00:01Z",
        "alice.json",
        &mut NoopControl,
    )
    .unwrap();
    respond(&f, "bob", Disposition::Reject, "2026-10-01T00:00:02Z", "bob.json", &mut NoopControl)
        .unwrap();
    let result = status(&f, &[PathBuf::from("alice.json"), PathBuf::from("bob.json")]);
    assert_eq!(result["items"][0]["state"], "conflicted");
    assert_eq!(result["items"][0]["blocking"], true);
    assert_eq!(result["items"][0]["dissent_ids"].as_array().unwrap().len(), 1);
}

/// Abstention remains recorded evidence and never fills a declared approval seat.
#[test]
fn actual_abstention_response_never_fills_a_seat() {
    let f = setup(false, false, None);
    create(&f, &mut NoopControl).unwrap();
    respond(
        &f,
        "alice",
        Disposition::Abstain,
        "2026-10-01T00:00:01Z",
        "alice.json",
        &mut NoopControl,
    )
    .unwrap();
    let result = status(&f, &[PathBuf::from("alice.json")]);
    assert_eq!(result["items"][0]["met_seats"].as_array().unwrap().as_slice(), &[] as &[Value]);
    assert_ne!(result["items"][0]["state"], "quorum-met");
}

/// A response at the exact due instant is late and cannot create current approval.
#[test]
fn exact_due_response_is_late_after_real_command_flow() {
    let f = setup(false, false, Some("2026-10-01T00:00:02Z"));
    create(&f, &mut NoopControl).unwrap();
    respond(
        &f,
        "alice",
        Disposition::Approve,
        "2026-10-01T00:00:02Z",
        "alice.json",
        &mut NoopControl,
    )
    .unwrap();
    let result = status(&f, &[PathBuf::from("alice.json")]);
    assert_eq!(result["responses"][0]["classification"], "late");
    assert_ne!(result["items"][0]["state"], "quorum-met");
}

/// Saved-plan formatting preserves native equality while invalidating recorded Queue pins.
#[test]
fn saved_plan_raw_drift_refuses_current_recorded_queue() {
    let f = setup(false, false, None);
    create(&f, &mut NoopControl).unwrap();
    let mut raw = std::fs::read(f.root.join("stored-plan.json")).unwrap();
    raw.push(b'\n');
    std::fs::write(f.root.join("stored-plan.json"), raw).unwrap();
    let mut output = Vec::new();
    let error = command_error(commands_v3::status(
        &current_options(&f, &[]),
        &mut output,
        &mut NoopControl,
    ));
    assert!(matches!(error, CommandError::Contract(ContractError::Binding)));
    assert_eq!(output.as_slice(), &[] as &[u8]);
}

/// Real output destinations survive Init, Respond and Merge no-replace refusals.
#[test]
fn every_file_command_preserves_existing_destination_bytes() {
    let f = setup(false, false, None);
    std::fs::write(f.root.join("queue.json"), b"sentinel").unwrap();
    assert!(matches!(command_error(create(&f, &mut NoopControl)), CommandError::Publication));
    assert_eq!(std::fs::read(f.root.join("queue.json")).unwrap(), b"sentinel");
    std::fs::remove_file(f.root.join("queue.json")).unwrap();
    create(&f, &mut NoopControl).unwrap();
    std::fs::write(f.root.join("alice.json"), b"sentinel").unwrap();
    assert!(matches!(
        command_error(respond(
            &f,
            "alice",
            Disposition::Approve,
            "2026-10-01T00:00:01Z",
            "alice.json",
            &mut NoopControl
        )),
        CommandError::Publication
    ));
    assert_eq!(std::fs::read(f.root.join("alice.json")).unwrap(), b"sentinel");
    std::fs::write(f.root.join("merged.json"), b"sentinel").unwrap();
    assert!(matches!(
        command_error(commands_v3::merge(
            &current_options(&f, &[]),
            Path::new("merged.json"),
            &mut NoopControl
        )),
        CommandError::Publication
    ));
    assert_eq!(std::fs::read(f.root.join("merged.json")).unwrap(), b"sentinel");
}

/// Actual publisher reach, rather than a predicted callback count, triggers native drift.
struct StagedDrift<'a> {
    /// Actual isolated fixture root, used to observe the real private staging directory.
    root: &'a Path,
    /// Complete actual input path changed once during the pre-rename callback.
    input: &'a str,
    /// True only after observing a real stage and changing the actual file.
    witnessed: bool,
}
impl WorkControl for StagedDrift<'_> {
    /// Observe the actual staged publisher before mutating one genuine held original.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
        if !self.witnessed
            && std::fs::read_dir(self.root).unwrap().any(|row| {
                row.unwrap().file_name().to_string_lossy().starts_with(".forge-authoring-file-")
            })
        {
            let path = self.root.join(self.input);
            let mut bytes = std::fs::read(&path).unwrap();
            bytes.push(b'\n');
            std::fs::write(path, bytes).unwrap();
            self.witnessed = true;
        }
        Ok(())
    }
    /// This controller never invents a stop; the actual original drift causes refusal.
    fn interruption(&self) -> Option<Interruption> {
        None
    }
}

/// Init retains the complete native owner through the real staged publication fence.
#[test]
fn staged_init_native_drift_refuses_and_cleans_private_stage() {
    let f = setup(false, false, None);
    let mut drift = StagedDrift { root: &f.root, input: "stored-plan.json", witnessed: false };
    assert!(matches!(
        command_error(create(&f, &mut drift)),
        CommandError::Contract(ContractError::Binding)
    ));
    assert!(drift.witnessed);
    assert!(!f.root.join("queue.json").exists());
    assert!(!std::fs::read_dir(&f.root).unwrap().any(|row| {
        row.unwrap().file_name().to_string_lossy().starts_with(".forge-authoring-file-")
    }));
}

/// Respond retains ordinary Queue/rationale originals through actual no-replace output.
#[test]
fn staged_response_rationale_drift_refuses_publication() {
    let f = setup(false, false, None);
    create(&f, &mut NoopControl).unwrap();
    let mut drift = StagedDrift { root: &f.root, input: "rationale.txt", witnessed: false };
    assert!(matches!(
        command_error(respond(
            &f,
            "alice",
            Disposition::Approve,
            "2026-10-01T00:00:01Z",
            "alice.json",
            &mut drift
        )),
        CommandError::Contract(ContractError::Binding)
    ));
    assert!(drift.witnessed);
    assert!(!f.root.join("alice.json").exists());
}

/// Actual staged Merge repeats complete native/original currentness before rename.
#[test]
fn staged_merge_locator_drift_refuses_publication() {
    let f = setup(false, false, None);
    create(&f, &mut NoopControl).unwrap();
    let mut drift = StagedDrift { root: &f.root, input: "locator.json", witnessed: false };
    assert!(matches!(
        command_error(commands_v3::merge(
            &current_options(&f, &[]),
            Path::new("merged.json"),
            &mut drift
        )),
        CommandError::Contract(ContractError::Binding)
    ));
    assert!(drift.witnessed);
    assert!(!f.root.join("merged.json").exists());
}

/// Caller writer failure preserves partial bytes and the fixed ordinary output error.
struct FailingWriter {
    /// Actual retained output prefix before an ordinary write failure.
    prefix: Vec<u8>,
}
impl Write for FailingWriter {
    /// Retain one real prefix, then fail the writer's next actual attempt.
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.prefix.is_empty() {
            let n = bytes.len().min(17);
            self.prefix.extend_from_slice(&bytes[..n]);
            Ok(n)
        } else {
            Err(std::io::Error::other("actual test writer failure"))
        }
    }
    /// Explicit writer flush completes without further side effects.
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Original caller callbacks expose precise first failures or after-Ok interruption.
struct TerminalProbe {
    /// Actual original callback count.
    calls: usize,
    /// Exact calibrated terminal callback, never a substituted producer visit count.
    stop_at: usize,
    /// Failure or after-Ok interruption mode.
    failed: bool,
    /// Retained actual interruption, observed by the accepted original wrapper.
    stopped: Option<Interruption>,
}
impl TerminalProbe {
    /// Construct one original controller before each actual operation.
    fn new(stop_at: usize, failed: bool) -> Self {
        Self { calls: 0, stop_at, failed, stopped: None }
    }
}
impl WorkControl for TerminalProbe {
    /// Count actual callbacks and return the selected first terminal cause.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
        self.calls += 1;
        if self.calls == self.stop_at {
            if self.failed {
                return Err(WorkError::Failed(crate::workspace::contract::Error::invalid()));
            }
            self.stopped = Some(Interruption::CancelRequested);
        }
        Ok(())
    }
    /// The same original operation detects an after-Ok stop.
    fn interruption(&self) -> Option<Interruption> {
        self.stopped
    }
}

/// Real partial writer failure crosses the final original command postfence.
#[test]
fn partial_status_output_failure_has_terminal_original_stop_priority() {
    let f = setup(false, false, None);
    create(&f, &mut NoopControl).unwrap();
    let mut probe = TerminalProbe::new(usize::MAX, false);
    let mut writer = FailingWriter { prefix: Vec::new() };
    assert!(matches!(
        command_error(commands_v3::status(&current_options(&f, &[]), &mut writer, &mut probe)),
        CommandError::Output
    ));
    assert_eq!(writer.prefix.len(), 17);
    for failed in [false, true] {
        let mut control = TerminalProbe::new(probe.calls, failed);
        let mut writer = FailingWriter { prefix: Vec::new() };
        let actual = command_error(commands_v3::status(
            &current_options(&f, &[]),
            &mut writer,
            &mut control,
        ));
        let expected = if failed {
            ContractError::ControlFailed
        } else {
            ContractError::Interrupted(Interruption::CancelRequested)
        };
        assert!(matches!(actual,CommandError::Contract(error) if error==expected));
        assert_eq!(writer.prefix.len(), 17);
    }
}

/// Ordinary malformed Queue failure also receives the actual final original postfence.
#[test]
fn ordinary_queue_parse_refusal_has_terminal_original_stop_priority() {
    let f = setup(false, false, None);
    create(&f, &mut NoopControl).unwrap();
    std::fs::write(f.root.join("queue.json"), b"{").unwrap();
    let mut probe = TerminalProbe::new(usize::MAX, false);
    let mut bytes = Vec::new();
    assert!(matches!(
        command_error(commands_v3::status(&current_options(&f, &[]), &mut bytes, &mut probe)),
        CommandError::Contract(ContractError::Invalid)
    ));
    let mut control = TerminalProbe::new(probe.calls, false);
    assert!(matches!(
        command_error(commands_v3::status(&current_options(&f, &[]), &mut bytes, &mut control)),
        CommandError::Contract(ContractError::Interrupted(Interruption::CancelRequested))
    ));
    assert_eq!(bytes.as_slice(), &[] as &[u8]);
}

/// Complete real status output can precede a final original-control stop.
#[test]
fn successful_status_terminal_stop_preserves_actual_emitted_bytes() {
    let f = setup(false, false, None);
    create(&f, &mut NoopControl).unwrap();
    let mut probe = TerminalProbe::new(usize::MAX, false);
    let mut expected = Vec::new();
    commands_v3::status(&current_options(&f, &[]), &mut expected, &mut probe).unwrap();
    let mut control = TerminalProbe::new(probe.calls, false);
    let mut actual = Vec::new();
    assert!(matches!(
        command_error(commands_v3::status(&current_options(&f, &[]), &mut actual, &mut control)),
        CommandError::Contract(ContractError::Interrupted(Interruption::CancelRequested))
    ));
    assert_eq!(actual, expected);
    assert_ne!(actual.as_slice(), &[] as &[u8]);
}
