// Genuine complete file/native receiver controls under the fixture-owning authoring tests.
// No constructed lease/currentness flag/owner or simulated native report is accepted.
use super::*;
use crate::evidence_capture::CaptureRole;
use crate::review::authoring_capture::{self as receiver, CurrentAuthoringPlanClosure};
use crate::review::capture::AuthoringOperation;
use crate::review::capture::{Pool, ReviewCapture, ReviewControl};

/// Persist the genuine legacy full native plan and closed private locator before capture.
fn originals(f: &Fixture) -> Value {
    let legacy = crate::authoring::input::prepare(&f.root.join("project.json"))
        .expect("genuine native file capture");
    let plan =
        crate::authoring::plan::build_plan(&legacy.loaded).expect("genuine full native plan");
    let value = serde_json::to_value(plan).expect("full native plan serde");
    write(&f.root.join("stored-plan.json"), &value);
    write(
        &f.root.join("locator.json"),
        &json!({"schema_version":"forge.review-authoring-plan-inputs/1",
        "project":{"path":"project.json"},"plan":{"path":"stored-plan.json"}}),
    );
    std::fs::write(f.root.join("policy.json"), b"private init-policy bytes").unwrap();
    value
}
/// Create one genuine accepted-operation capture with an actual reserved output namespace.
fn begin(
    f: &Fixture,
    operation: AuthoringOperation,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> ReviewCapture {
    let outputs: Vec<&Path> = if operation == AuthoringOperation::Status {
        Vec::new()
    } else {
        vec![Path::new("new-output.json")]
    };
    ReviewCapture::new_authoring(&f.root, &outputs, operation, ledger, control)
        .expect("genuine capture")
}
/// Complete the actual Init union; the saved locator/policy remain outside native provenance.
fn init(
    f: &Fixture,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<CurrentAuthoringPlanClosure, ContractError> {
    let mut capture = begin(f, AuthoringOperation::Init, ledger, control);
    let pending = receiver::read_pending(&mut capture, Path::new("locator.json"), ledger, control)?;
    capture.required_authoring_policy(Path::new("policy.json"), ledger, control)?;
    pending.finish_and_seal(capture, ledger, control)
}
/// Compare the full private result and native denominator with the unchanged genuine file wrapper.
fn parity(f: &Fixture, owner: &CurrentAuthoringPlanClosure) {
    let legacy = crate::authoring::input::prepare(&f.root.join("project.json")).unwrap();
    let plan = crate::authoring::plan::build_plan(&legacy.loaded).unwrap();
    assert_eq!(
        serde_json::to_value(&owner.facts().plan).unwrap(),
        serde_json::to_value(plan).unwrap()
    );
    assert_eq!(owner.facts().loaded.inputs, legacy.loaded.inputs);
    assert_eq!(owner.native_provenance().count(), legacy.loaded.inputs.len());
    assert_eq!(owner.source_pins().len(), legacy.loaded.inputs.len() + 1);
    assert!(
        owner
            .native_provenance()
            .all(|p| !matches!(p.role, "stored-plan" | "review-locator" | "init-policy"))
    );
    let public = serde_json::to_string(owner.source_pins()).unwrap();
    for private in [
        "Private access title",
        "Private native interview",
        "clause.md",
        "project.json",
        "native-project",
    ] {
        assert!(!public.contains(private), "private operand escaped: {private}");
    }
}
/// Require a precise ordinary binding refusal with a safe fixed actual diagnostic.
fn binding<T>(result: Result<T, ContractError>) {
    let error = result.err().expect("genuine receiver must refuse");
    assert_eq!(error, ContractError::Binding, "actual safe error: {error:?}");
}
/// Native Catalog, private unresolved context and full stored-plan equality survive the real receiver.
#[test]
fn native_catalog_full_plan_and_private_denominator_match() {
    let f = Fixture::new(false, false);
    originals(&f);
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let owner = init(&f, &mut ledger, &mut control).unwrap();
    parity(&f, &owner);
    assert!(owner.originals().unwrap().operation() == AuthoringOperation::Init);
    assert!(owner.originals().unwrap().queue_raw().is_none());
    assert_eq!(owner.originals().unwrap().policy_raw().unwrap(), b"private init-policy bytes");
    assert!(!f.root.join("new-output.json").exists());
}
/// Complete Profile, explicit Catalog and Mapping originals derive exact actual native UUIDs.
#[test]
fn native_profile_mapping_complete_roster_and_original_root_text() {
    let f = Fixture::new(true, true);
    originals(&f);
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let owner = init(&f, &mut ledger, &mut control).unwrap();
    parity(&f, &owner);
    let pins = owner.source_pins();
    assert_eq!(
        pins.iter()
            .find(|p| p.artifact_key == "authoring:framework")
            .unwrap()
            .native_root_uuid
            .as_deref(),
        Some("22222222-2222-4222-8222-222222222222")
    );
    assert_eq!(
        pins.iter()
            .find(|p| p.artifact_key == "authoring:resolved")
            .unwrap()
            .native_root_uuid
            .as_deref(),
        Some("11111111-1111-4111-8111-111111111111")
    );
    assert!(pins.iter().any(|p| p.artifact_key == "authoring:mapping:0"));
    assert!(pins.windows(2).all(|p| p[0].artifact_key < p[1].artifact_key));
}
/// Native project identity and offset/fraction time remain exact rather than UUID/UTC substitutions.
#[test]
fn exact_native_key_and_as_of_are_retained() {
    let f = Fixture::new(false, false);
    originals(&f);
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let owner = init(&f, &mut ledger, &mut control).unwrap();
    assert_eq!(owner.facts().loaded.project.project_key, "native-project");
    assert_eq!(owner.facts().plan.as_of, "2026-09-08T01:30:00.250+01:30");
    assert!(uuid::Uuid::parse_str(&owner.facts().plan.project_key).is_err());
}
/// Syntax-only saved-plan whitespace passes full equality while its actual public raw pin changes.
#[test]
fn stored_plan_whitespace_changes_raw_identity_without_changing_native_plan() {
    let f = Fixture::new(false, false);
    let plan = originals(&f);
    let before = std::fs::read(f.root.join("stored-plan.json")).unwrap();
    let mut changed = serde_json::to_vec(&plan).unwrap();
    changed.push(b'\n');
    std::fs::write(f.root.join("stored-plan.json"), &changed).unwrap();
    assert_ne!(sha256_hex(&before), sha256_hex(&changed));
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let owner = init(&f, &mut ledger, &mut control).unwrap();
    parity(&f, &owner);
    assert_eq!(
        owner.source_pins().iter().find(|p| p.artifact_key == "authoring:plan").unwrap().raw_sha256,
        sha256_hex(&changed)
    );
}
/// A coherent change to any full private native field reaches the genuine complete plan oracle.
#[test]
fn complete_stored_plan_private_mutations_are_refused() {
    let f = Fixture::new(false, false);
    let expected = originals(&f);
    let mut candidates = Vec::new();
    let mut c = expected.clone();
    c["project_key"] = json!("different-project");
    candidates.push(c);
    let mut c = expected.clone();
    c["as_of"] = json!("2026-09-08T00:00:00Z");
    candidates.push(c);
    let mut c = expected.clone();
    c["provenance"]["baseline_review"]["rationale"] = json!("Different private rationale");
    candidates.push(c);
    let mut c = expected.clone();
    c["counts"]["total"] = json!(5);
    candidates.push(c);
    let mut c = expected.clone();
    c["policies"][0]["title"] = json!("Different private title");
    candidates.push(c);
    let mut c = expected.clone();
    c["provenance"]["inputs"].as_array_mut().unwrap().reverse();
    candidates.push(c);
    let mut c = expected.clone();
    c["unresolved_questions"] = json!([]);
    candidates.push(c);
    let mut c = expected.clone();
    c["unexpected"] = json!(null);
    candidates.push(c);
    for value in candidates {
        write(&f.root.join("stored-plan.json"), &value);
        let mut ledger = ContractLedger::default();
        let mut caller = NoopControl;
        let mut control = ReviewControl::accept(&mut caller);
        binding(init(&f, &mut ledger, &mut control));
        assert!(!f.root.join("new-output.json").exists());
    }
}
/// Duplicate decoded native-plan property names cannot be erased by a typed decoder.
#[test]
fn stored_plan_duplicate_decoded_property_is_refused() {
    let f = Fixture::new(false, false);
    let expected = originals(&f);
    let raw = serde_json::to_string(&expected).unwrap();
    let duplicate = format!("{{\"schema_version\":\"forge.authoring-plan/1\",{}", &raw[1..]);
    std::fs::write(f.root.join("stored-plan.json"), duplicate).unwrap();
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    binding(init(&f, &mut ledger, &mut control));
}
/// An under-cap malformed native original is ordinary Binding and cannot latch a later valid operation.
#[test]
fn native_parse_failure_is_unlatched_and_same_original_control_continues() {
    let f = Fixture::new(false, false);
    originals(&f);
    std::fs::write(f.root.join("project.json"), b"").unwrap();
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    binding(init(&f, &mut ledger, &mut control));
    std::fs::write(f.root.join("project.json"), &f.project_raw).unwrap();
    let owner = init(&f, &mut ledger, &mut control).unwrap();
    parity(&f, &owner);
}
/// The native saved App report must remain the complete unfiltered regenerated private result.
#[test]
fn stale_private_gap_report_is_not_accepted_as_a_saved_plan_basis() {
    let mut f = Fixture::new(false, false);
    originals(&f);
    let mut report: Value = serde_json::from_slice(&f.report).unwrap();
    report["private-extra"] = json!("invented");
    f.report = write(&f.root.join("gap-report.json"), &report);
    f.project["gap_report"]["expected_sha256"] = json!(sha256_hex(&f.report));
    f.project["baseline"]["report_sha256"] = json!(sha256_hex(&f.report));
    f.pack["baseline"] = f.project["baseline"].clone();
    f.save();
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    binding(init(&f, &mut ledger, &mut control));
}
/// A full N route occurrence cannot be reused as the review-only saved-plan S original.
#[test]
fn stored_plan_exact_native_route_alias_is_refused() {
    let f = Fixture::new(false, false);
    originals(&f);
    write(
        &f.root.join("locator.json"),
        &json!({"schema_version":"forge.review-authoring-plan-inputs/1",
        "project":{"path":"project.json"},"plan":{"path":"project.json"}}),
    );
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    binding(init(&f, &mut ledger, &mut control));
}
/// The actual held capture refuses physical hard aliases before any saved-plan parser can repair them.
#[test]
fn stored_plan_physical_native_alias_is_refused() {
    let f = Fixture::new(false, false);
    originals(&f);
    std::fs::remove_file(f.root.join("stored-plan.json")).unwrap();
    std::fs::hard_link(f.root.join("project.json"), f.root.join("stored-plan.json")).unwrap();
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    binding(init(&f, &mut ledger, &mut control));
}
/// Extra and repeated Auxiliary occurrences are observed separately from distinct retained Entry count.
#[test]
fn unrelated_and_repeated_auxiliary_occurrences_cannot_seal() {
    let f = Fixture::new(false, false);
    originals(&f);
    std::fs::write(f.root.join("extra.bin"), b"extra").unwrap();
    for path in ["extra.bin", "locator.json"] {
        let mut ledger = ContractLedger::default();
        let mut caller = NoopControl;
        let mut control = ReviewControl::accept(&mut caller);
        let mut capture = begin(&f, AuthoringOperation::Init, &mut ledger, &mut control);
        let pending = receiver::read_pending(
            &mut capture,
            Path::new("locator.json"),
            &mut ledger,
            &mut control,
        )
        .unwrap();
        capture
            .required_authoring_policy(Path::new("policy.json"), &mut ledger, &mut control)
            .unwrap();
        capture
            .required(
                Path::new(path),
                CaptureRole::ReviewPrivateConfig,
                Pool::Auxiliary,
                1024 * 1024,
                &mut ledger,
                &mut control,
            )
            .unwrap();
        binding(pending.finish_and_seal(capture, &mut ledger, &mut control));
    }
}
/// Init/current operation requirements are real registry correlations, never caller completeness flags.
#[test]
fn closed_operation_requires_exact_policy_or_queue_registry() {
    let f = Fixture::new(false, false);
    originals(&f);
    for operation in
        [AuthoringOperation::Init, AuthoringOperation::Merge, AuthoringOperation::Status]
    {
        let mut ledger = ContractLedger::default();
        let mut caller = NoopControl;
        let mut control = ReviewControl::accept(&mut caller);
        let mut capture = begin(&f, operation, &mut ledger, &mut control);
        let pending = receiver::read_pending(
            &mut capture,
            Path::new("locator.json"),
            &mut ledger,
            &mut control,
        )
        .unwrap();
        binding(pending.finish_and_seal(capture, &mut ledger, &mut control));
    }
}
/// Every captured response occurrence survives, even two registrations of one exact same original.
/// Raw Queue/Response bytes here exercise registry ownership only; native envelope binding is absent.
#[test]
fn current_registry_retains_duplicate_response_occurrences_without_native_queue_claim() {
    let f = Fixture::new(false, false);
    originals(&f);
    std::fs::write(f.root.join("queue.json"), b"inert registry fixture Queue original").unwrap();
    std::fs::write(f.root.join("response.json"), b"inert registry fixture Response original")
        .unwrap();
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let mut capture = begin(&f, AuthoringOperation::Status, &mut ledger, &mut control);
    let pending =
        receiver::read_pending(&mut capture, Path::new("locator.json"), &mut ledger, &mut control)
            .unwrap();
    capture
        .required(
            Path::new("queue.json"),
            CaptureRole::ReviewQueue,
            Pool::Queue,
            10 * 1024 * 1024,
            &mut ledger,
            &mut control,
        )
        .unwrap();
    for _ in 0..2 {
        capture
            .required(
                Path::new("response.json"),
                CaptureRole::ReviewResponse,
                Pool::Response,
                1024 * 1024,
                &mut ledger,
                &mut control,
            )
            .unwrap();
    }
    let owner = pending.finish_and_seal(capture, &mut ledger, &mut control).unwrap();
    assert_eq!(owner.originals().unwrap().responses().count(), 2);
    assert_eq!(
        owner.originals().unwrap().queue_raw().unwrap(),
        b"inert registry fixture Queue original"
    );
    assert!(owner.originals().unwrap().policy_raw().is_none());
    parity(&f, &owner);
}
/// Same bytes/routes in a foreign capture cannot repair the actual pending allocation owner.
#[test]
fn foreign_capture_with_same_bytes_cannot_seal_pending_native_cohort() {
    let f = Fixture::new(false, false);
    originals(&f);
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let mut first = begin(&f, AuthoringOperation::Init, &mut ledger, &mut control);
    let pending =
        receiver::read_pending(&mut first, Path::new("locator.json"), &mut ledger, &mut control)
            .unwrap();
    let mut other = begin(&f, AuthoringOperation::Init, &mut ledger, &mut control);
    let _other_pending =
        receiver::read_pending(&mut other, Path::new("locator.json"), &mut ledger, &mut control)
            .unwrap();
    other.required_authoring_policy(Path::new("policy.json"), &mut ledger, &mut control).unwrap();
    binding(pending.finish_and_seal(other, &mut ledger, &mut control));
}
/// Drift of a real N source after native preparation is rejected by the actual final physical fence.
#[test]
fn native_source_drift_after_preparation_cannot_issue_current_owner() {
    let f = Fixture::new(false, false);
    originals(&f);
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let mut capture = begin(&f, AuthoringOperation::Init, &mut ledger, &mut control);
    let pending =
        receiver::read_pending(&mut capture, Path::new("locator.json"), &mut ledger, &mut control)
            .unwrap();
    capture.required_authoring_policy(Path::new("policy.json"), &mut ledger, &mut control).unwrap();
    std::fs::write(f.root.join("clause.md"), b"changed complete human clause").unwrap();
    binding(pending.finish_and_seal(capture, &mut ledger, &mut control));
    assert!(!f.root.join("new-output.json").exists());
}
/// A later real original change invalidates the retained whole physical owner too.
#[test]
fn full_held_fence_retains_locator_policy_and_native_source_generations() {
    let f = Fixture::new(false, false);
    originals(&f);
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let owner = init(&f, &mut ledger, &mut control).unwrap();
    std::fs::write(f.root.join("policy.json"), b"changed private policy original").unwrap();
    binding(owner.verify_inputs(&mut ledger, &mut control));
}
/// Actual bounded work consumed after genuine preparation cannot be renewed by sealing another owner.
#[test]
fn first_capacity_stop_survives_real_receiver_and_seal_phases() {
    let f = Fixture::new(false, false);
    originals(&f);
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let mut capture = begin(&f, AuthoringOperation::Init, &mut ledger, &mut control);
    let pending =
        receiver::read_pending(&mut capture, Path::new("locator.json"), &mut ledger, &mut control)
            .unwrap();
    capture.required_authoring_policy(Path::new("policy.json"), &mut ledger, &mut control).unwrap();
    assert_eq!(ledger.visits(1_000_001).unwrap_err(), ContractError::Capacity);
    let error = pending.finish_and_seal(capture, &mut ledger, &mut control).err().unwrap();
    assert_eq!(error, ContractError::Capacity);
    assert_eq!(ledger.bytes(0).unwrap_err(), ContractError::Capacity);
}
/// One original callback observer for genuine calibrated native/physical final-stop controls.
struct FenceProbe {
    /// Complete actual callback count on this one original caller.
    calls: usize,
    /// Calibrated actual callback at which a transient error is returned.
    stop: Option<usize>,
    /// Select actual ordinary `WorkError` rather than typed interruption.
    fail: bool,
}
impl WorkControl for FenceProbe {
    /// Preserve actual original callback errors; no native success flag is supplied.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
        self.calls += 1;
        if self.stop == Some(self.calls) {
            if self.fail {
                return Err(WorkError::Failed(Error::invalid()));
            }
            return Err(WorkError::Interrupted(Interruption::CancelRequested));
        }
        Ok(())
    }
    /// This test exercises transient actual checkpoint errors, not an invented latched caller state.
    fn interruption(&self) -> Option<Interruption> {
        None
    }
}
/// Actual final owner fences preserve both transient interruption and ordinary original-control failure.
#[test]
fn original_final_seal_checkpoint_stop_prevents_current_owner() {
    let f = Fixture::new(false, false);
    originals(&f);
    let mut baseline = FenceProbe { calls: 0, stop: None, fail: false };
    {
        let mut ledger = ContractLedger::default();
        let mut control = ReviewControl::accept(&mut baseline);
        let _owner = init(&f, &mut ledger, &mut control).unwrap();
    }
    let terminal = baseline.calls;
    assert!(terminal > 10);
    for fail in [false, true] {
        let mut probe = FenceProbe { calls: 0, stop: Some(terminal), fail };
        let mut ledger = ContractLedger::default();
        let result = {
            let mut control = ReviewControl::accept(&mut probe);
            init(&f, &mut ledger, &mut control)
        };
        let expected = if fail {
            ContractError::ControlFailed
        } else {
            ContractError::Interrupted(Interruption::CancelRequested)
        };
        assert_eq!(result.err().unwrap(), expected);
        let before = probe.calls;
        assert_eq!(ledger.bytes(0).unwrap_err(), expected);
        assert_eq!(probe.calls, before, "latched original stop must bypass later callbacks");
    }
}

/// The actual project containing directory defines N labels separately from review capture routes.
#[test]
fn nested_actual_project_parent_preserves_native_filename_and_roster() {
    let f = Fixture::new(false, false);
    originals(&f);
    let nested = f.root.join("native");
    std::fs::create_dir(&nested).unwrap();
    for name in [
        "project.json",
        "pack.json",
        "gap-report.json",
        "applicability.json",
        "framework.json",
        "clause.md",
    ] {
        std::fs::rename(f.root.join(name), nested.join(name)).unwrap();
    }
    write(
        &f.root.join("locator.json"),
        &json!({"schema_version":"forge.review-authoring-plan-inputs/1",
        "project":{"path":"native/project.json"},"plan":{"path":"stored-plan.json"}}),
    );
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let owner = init(&f, &mut ledger, &mut control).unwrap();
    let legacy = crate::authoring::input::prepare(&nested.join("project.json")).unwrap();
    assert_eq!(owner.facts().loaded.inputs, legacy.loaded.inputs);
    assert!(owner.native_provenance().all(|row| !row.path.starts_with("native/")));
    assert_eq!(
        owner.native_provenance().find(|row| row.role == "author-project").unwrap().path,
        "project.json"
    );
    owner.verify_inputs(&mut ledger, &mut control).unwrap();
}
/// Strict private locator admission is reached through genuine capture; failure leaves native files intact.
#[test]
fn genuine_locator_duplicates_unknown_fields_and_escape_are_refused() {
    let f = Fixture::new(false, false);
    originals(&f);
    let candidates=[br#"{"schema_version":"forge.review-authoring-plan-inputs/1","project":{"path":"project.json"},"plan":{"path":"stored-plan.json"},"extra":null}"#.as_slice(),
        br#"{"schema_version":"forge.review-authoring-plan-inputs/1","project":{"path":"../project.json"},"plan":{"path":"stored-plan.json"}}"#.as_slice(),
        br#"{"schema_version":"forge.review-authoring-plan-inputs/1","schema_version":"forge.review-authoring-plan-inputs/1","project":{"path":"project.json"},"plan":{"path":"stored-plan.json"}}"#.as_slice()];
    for raw in candidates {
        std::fs::write(f.root.join("locator.json"), raw).unwrap();
        let mut ledger = ContractLedger::default();
        let mut caller = NoopControl;
        let mut control = ReviewControl::accept(&mut caller);
        assert_eq!(init(&f, &mut ledger, &mut control).err().unwrap(), ContractError::Invalid);
        assert_eq!(std::fs::read(f.root.join("project.json")).unwrap(), f.project_raw);
        assert!(!f.root.join("new-output.json").exists());
    }
}
/// A new private role cannot widen the unchanged ordinary /1 Source admission table.
#[test]
fn ordinary_capture_refuses_authoring_private_source_role() {
    let f = Fixture::new(false, false);
    originals(&f);
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let mut capture = ReviewCapture::new(&f.root, &[], &mut ledger, &mut control).unwrap();
    assert_eq!(
        capture
            .required(
                Path::new("project.json"),
                CaptureRole::AuthoringArtifactOriginal,
                Pool::Source,
                2 * 1024 * 1024,
                &mut ledger,
                &mut control
            )
            .unwrap_err(),
        ContractError::Invalid
    );
}

/// Genuine native Queue/Response binding and complete-owner policy/finalizer controls.
mod binding_tests {
    include!("authoring_binding_tests.rs");
}

/// Genuine Init export controls borrowing only the actual maintained native fixture.
mod init_export_tests {
    include!("authoring_init_export_tests.rs");
}

/// Genuine native command and publisher controls on the Unix staging observer.
#[cfg(unix)]
mod command_tests {
    include!("authoring_commands_tests.rs");
}
