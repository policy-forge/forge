//! Prospective genuine native file/captured Lifecycle component controls.
//! Real maintained init/transitions create synthetic asserted local approvals.
//! These tests confer no human authority, production publication or platform acceptance.

use super::*;
use crate::cli::LifecycleOutputFormat;
use crate::lifecycle::record::{self, DeclaredRole};
use crate::lifecycle::{self, InitOptions, TransitionOptions};
use crate::workspace::contract::Error;
use crate::workspace::preparation::{
    Interruption, NoopControl, ProgressUpdate, Stage, WorkError, WorkResult,
};
use serde_json::{Value, json};

/// Real confined fixture bytes and ordinary file-oriented native lifecycle producers.
struct Fixture {
    /// Actual private root survives the complete native held-owner lifetime.
    _directory: tempfile::TempDir,
    /// Canonical actual native root, not a caller proof assertion.
    root: PathBuf,
    /// Complete original generated file routes in native declared order.
    generated: Vec<PathBuf>,
}
impl Fixture {
    /// Build complete genuine approved inputs; bare roots intentionally validate identity only.
    fn new(count: usize, source_is_generated: bool, empty_source: bool) -> Self {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        std::fs::create_dir(root.join("generated")).unwrap();
        std::fs::write(
            root.join("source.bin"),
            if empty_source { &b""[..] } else { &b"\xff\0private opaque source"[..] },
        )
        .unwrap();
        let models = [
            "catalog",
            "profile",
            "component-definition",
            "mapping-collection",
            "system-security-plan",
            "plan-of-action-and-milestones",
        ];
        let uuids = [
            "11111111-1111-4111-8111-111111111111",
            "ABCDEFAB1234556789ABCDEF01234567",
            "{ABCDEFAB-1234-5567-89AB-CDEF01234567}",
            "urn:uuid:ABCDEFAB-1234-5567-89AB-CDEF01234567",
            "00000000-0000-0000-0000-000000000000",
            "abcdefab-1234-1567-09ab-cdef01234567",
        ];
        let mut generated = Vec::new();
        for index in 0..count {
            let route = PathBuf::from(format!("generated/{index:02}.json"));
            let bytes =
                serde_json::to_vec(&json!({(models[index % 6]): {"uuid": uuids[index % 6]}}))
                    .unwrap();
            std::fs::write(root.join(&route), bytes).unwrap();
            generated.push(route);
        }
        let source =
            if source_is_generated { root.join(&generated[0]) } else { root.join("source.bin") };
        let absolute: Vec<_> = generated.iter().map(|route| root.join(route)).collect();
        lifecycle::execute_init(&InitOptions {
            source: &source,
            artifacts: &absolute,
            output: &root.join("record.json"),
            policy_key: " native\0policy λ ",
            version_key: " version:1 ",
            title: "private native title",
            owners: &["owner".to_string()],
            parties: &["reviewer=reviewer".to_string(), "approver=approver".to_string()],
            next_review: chrono::NaiveDate::from_ymd_opt(2026, 9, 1).unwrap(),
            cadence_days: 30,
            due_soon_days: 7,
            required_reviewers: 1,
            required_approvers: 1,
            separate_author_reviewer: false,
            separate_author_approver: false,
            separate_reviewer_approver: false,
        })
        .unwrap();
        for (state, actor, role, timestamp, assertions) in [
            (
                LifecycleState::InReview,
                "reviewer",
                DeclaredRole::Reviewer,
                "2026-08-01T00:00:00Z",
                Vec::<String>::new(),
            ),
            (
                LifecycleState::Approved,
                "approver",
                DeclaredRole::Approver,
                "2026-08-01T00:00:01Z",
                vec!["reviewer=reviewer".to_string()],
            ),
        ] {
            lifecycle::execute_transition(&TransitionOptions {
                record_path: &root.join("record.json"),
                next_state: state,
                actor_key: actor,
                role,
                timestamp,
                rationale: "private native rationale",
                assertions: &assertions,
                impact_finding_ids: &[],
                replacement_policy_key: None,
                replacement_version_key: None,
                apply: true,
                output: None,
            })
            .unwrap();
        }
        let fixture = Self { _directory: directory, root, generated };
        fixture.locator();
        fixture
    }
    /// Write only inert complete routes from the actual maintained intrinsically validated record.
    fn locator(&self) {
        let raw = std::fs::read(self.root.join("record.json")).unwrap();
        let record = record::parse(&raw).unwrap();
        let mut declared: Vec<_> =
            record.policy.generated_artifacts.iter().map(|row| row.path.as_str()).collect();
        declared.sort_unstable();
        let locator = json!({"schema_version":"forge.review-lifecycle-inputs/1", "record":{"path":"record.json"},
            "source":{"declared_path":record.policy.source.path,"path":record.policy.source.path},
            "generated_artifacts":declared.iter().map(|path|json!({"declared_path":path,"path":path})).collect::<Vec<_>>()});
        std::fs::write(self.root.join("locator.json"), serde_json::to_vec(&locator).unwrap())
            .unwrap();
    }
    /// Enter the actual mode and read the genuine complete original cohort under one caller ledger.
    fn pending(&self) -> (ReviewCapture, PendingLifecycleCohort, ContractLedger) {
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        let mut capture = ReviewCapture::new_lifecycle(
            &self.root,
            &[Path::new("result.json")],
            &mut ledger,
            &mut control,
        )
        .unwrap();
        let pending =
            read_pending(&mut capture, Path::new("locator.json"), &mut ledger, &mut control)
                .unwrap();
        (capture, pending, ledger)
    }
    /// Consume the genuine capture only after the exact complete operation's registrations.
    fn closure(&self) -> (ApprovedLifecycleClosure, ContractLedger) {
        let (capture, pending, mut ledger) = self.pending();
        let closure = pending.finish_and_seal(capture, &mut ledger, &mut NoopControl).unwrap();
        (closure, ledger)
    }
}

/// A full actual file-oriented neutral report equals the captured native full projection.
#[test]
fn genuine_full_native_status_parity_six_models_and_original_uuid_text() {
    let fixture = Fixture::new(6, false, false);
    let native_output = fixture.root.join("native-status.json");
    assert!(
        !lifecycle::execute_check(
            &[fixture.root.join("record.json")],
            &LifecycleOutputFormat::Json,
            Some(&native_output)
        )
        .unwrap()
    );
    let full: Value = serde_json::from_slice(&std::fs::read(native_output).unwrap()).unwrap();
    let (owner, mut ledger) = fixture.closure();
    let (parsed, plan) =
        native::record(owner.members[0].lease.bytes(), &mut ledger, &mut NoopControl).unwrap();
    let mut ordered: Vec<_> = parsed.policy.generated_artifacts.iter().collect();
    ordered.sort_by(|a, b| a.path.cmp(&b.path));
    let current =
        observed_artifacts(&ordered, &owner.members, &mut ledger, &mut NoopControl).unwrap();
    let status = native::status(&parsed, &current, &plan, &mut ledger, &mut NoopControl).unwrap();
    assert_eq!(full, json!([status]));
    assert_eq!(owner.source_pins().len(), 8);
    assert_eq!(owner.record().policy.policy_key, " native\0policy λ ");
    for (native, member) in ordered.iter().zip(&owner.members[2..]) {
        assert_eq!(member.pin.native_root_uuid.as_deref(), native.root_uuid.as_deref());
        assert_eq!(
            member.pin.native_model.unwrap().as_str(),
            native.oscal_type.as_deref().unwrap()
        );
        assert!(member.pin.schema_identity.is_none());
        assert_eq!(member.pin.validation_profile, GENERATED_PROFILE);
    }
    assert_eq!(owner.roster().len(), 8);
    owner.verify_inputs(&mut ledger, &mut NoopControl).unwrap();
    assert!(!fixture.root.join("result.json").exists());
}

/// Complete occurrences share a real lease while distinct physical raw accounting is unchanged.
#[test]
fn genuine_source_generated_lease_reuse_and_empty_opaque_source() {
    for (shared, empty, count) in [(true, false, 6), (false, true, 0)] {
        let fixture = Fixture::new(count, shared, empty);
        let (owner, mut ledger) = fixture.closure();
        assert_eq!(owner.members.len(), count + 2);
        assert_eq!(owner.held_inputs().source_original_count(), count + if shared { 1 } else { 2 });
        if shared {
            assert!(owner.members[1].lease.same_original(&owner.members[2].lease));
            assert_eq!(owner.members[1].index, owner.members[2].index);
            assert_eq!(owner.members[1].pin.raw_sha256, owner.members[2].pin.raw_sha256);
            assert!(owner.members[1].pin.native_root_uuid.is_none());
            assert!(owner.members[2].pin.native_root_uuid.is_some());
        } else {
            assert_eq!(owner.members[1].pin.byte_length, 0);
            assert_eq!(owner.members[1].pin.raw_sha256, sha256_hex(b""));
        }
        owner.verify_inputs(&mut ledger, &mut NoopControl).unwrap();
    }
}

/// All98 declared generated occurrences consume the unchanged hundred Source registration profile.
#[test]
fn genuine_complete_98_generated_roster_and_public_key_order() {
    let fixture = Fixture::new(98, false, false);
    let (owner, mut ledger) = fixture.closure();
    assert_eq!(owner.members.len(), 100);
    assert_eq!(owner.held_inputs().source_original_count(), 100);
    assert!(owner.pins.windows(2).all(|pair| pair[0].artifact_key < pair[1].artifact_key));
    assert_eq!(owner.members[12].pin.artifact_key, "lifecycle:generated:10");
    assert_eq!(owner.members[4].pin.artifact_key, "lifecycle:generated:2");
    assert!(
        owner.pins.iter().position(|p| p.artifact_key == "lifecycle:generated:10").unwrap()
            < owner.pins.iter().position(|p| p.artifact_key == "lifecycle:generated:2").unwrap()
    );
    owner.verify_inputs(&mut ledger, &mut NoopControl).unwrap();
}

/// Omission or a same-sized wrong explicit route never selects a native prefix/foreign artifact.
#[test]
fn genuine_complete_locator_omission_and_route_mismatch_refuse() {
    for omitted in [true, false] {
        let fixture = Fixture::new(6, false, false);
        let path = fixture.root.join("locator.json");
        let mut locator: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        if omitted {
            locator["generated_artifacts"].as_array_mut().unwrap().pop();
        } else {
            locator["generated_artifacts"][0]["path"] = json!("source.bin");
        }
        std::fs::write(path, serde_json::to_vec(&locator).unwrap()).unwrap();
        let mut ledger = ContractLedger::default();
        let mut capture = ReviewCapture::new_lifecycle(
            &fixture.root,
            &[Path::new("result.json")],
            &mut ledger,
            &mut NoopControl,
        )
        .unwrap();
        assert!(matches!(
            read_pending(&mut capture, Path::new("locator.json"), &mut ledger, &mut NoopControl),
            Err(ContractError::Binding)
        ));
        assert!(!fixture.root.join("result.json").exists());
    }
}

/// Source/raw or native generated UUID drift defeats the latest actual recorded approval.
#[test]
fn genuine_currentness_drift_and_exact_uuid_spelling_refuse() {
    for identity in [false, true] {
        let fixture = Fixture::new(6, false, false);
        if identity {
            let path = fixture.root.join(&fixture.generated[1]);
            let mut value: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            value["profile"]["uuid"] = json!("abcdefab-1234-5567-89ab-cdef01234567");
            std::fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
        } else {
            std::fs::write(fixture.root.join("source.bin"), b"changed actual raw source").unwrap();
        }
        let mut ledger = ContractLedger::default();
        let mut capture = ReviewCapture::new_lifecycle(
            &fixture.root,
            &[Path::new("result.json")],
            &mut ledger,
            &mut NoopControl,
        )
        .unwrap();
        assert!(matches!(
            read_pending(&mut capture, Path::new("locator.json"), &mut ledger, &mut NoopControl),
            Err(ContractError::Binding)
        ));
        assert!(!fixture.root.join("result.json").exists());
    }
}

/// Source==record reuses its actual record lease but cannot manufacture a current hash fixed point.
#[test]
fn genuine_source_record_lease_relation_remains_drift_refusal() {
    let fixture = Fixture::new(6, false, false);
    let path = fixture.root.join("record.json");
    let mut record = record::parse(&std::fs::read(&path).unwrap()).unwrap();
    record.policy.source.path = "record.json".to_string();
    for index in 0..record.history.len() {
        record.history[index].event_id = record::event_id(&record, &record.history[index]).unwrap();
    }
    record::validate(&record).unwrap();
    std::fs::write(path, serde_json::to_vec(&record).unwrap()).unwrap();
    fixture.locator();
    let mut ledger = ContractLedger::default();
    let mut capture = ReviewCapture::new_lifecycle(
        &fixture.root,
        &[Path::new("result.json")],
        &mut ledger,
        &mut NoopControl,
    )
    .unwrap();
    assert!(matches!(
        read_pending(&mut capture, Path::new("locator.json"), &mut ledger, &mut NoopControl),
        Err(ContractError::Binding)
    ));
    assert_eq!(capture.finish().source_original_count(), 7);
    assert!(!fixture.root.join("result.json").exists());
}

/// Genuine extra successful Source occurrences and byte-equal other capture owners fail sealing.
#[test]
fn genuine_whole_source_and_same_original_owner_refusals() {
    let fixture = Fixture::new(6, false, false);
    let (mut capture, pending, mut ledger) = fixture.pending();
    std::fs::write(
        fixture.root.join("extra.json"),
        br#"{"catalog":{"uuid":"11111111-1111-4111-8111-111111111111"}}"#,
    )
    .unwrap();
    capture
        .required_lifecycle_source(
            Path::new("extra.json"),
            LifecyclePurpose::Generated(6),
            &mut ledger,
            &mut NoopControl,
        )
        .unwrap();
    assert!(matches!(
        pending.finish_and_seal(capture, &mut ledger, &mut NoopControl),
        Err(ContractError::Binding)
    ));
    let (_capture_a, pending_a, _ledger_a) = fixture.pending();
    let (capture_b, pending_b, mut ledger_b) = fixture.pending();
    drop(pending_b);
    assert!(matches!(
        pending_a.finish_and_seal(capture_b, &mut ledger_b, &mut NoopControl),
        Err(ContractError::Binding)
    ));
    assert!(!fixture.root.join("result.json").exists());
}

/// Held owners detect an actual in-place source edit at the final native fence.
#[test]
fn genuine_final_stale_source_and_ordinary_role_table_refusal() {
    let fixture = Fixture::new(6, false, false);
    let (owner, mut ledger) = fixture.closure();
    std::fs::write(fixture.root.join("source.bin"), b"different actual final source bytes")
        .unwrap();
    assert_eq!(owner.verify_inputs(&mut ledger, &mut NoopControl), Err(ContractError::Binding));
    let mut ordinary_ledger = ContractLedger::default();
    let mut ordinary =
        ReviewCapture::new(&fixture.root, &[], &mut ordinary_ledger, &mut NoopControl).unwrap();
    assert_eq!(
        ordinary.required(
            Path::new("source.bin"),
            CaptureRole::LifecycleArtifactOriginal,
            super::super::capture::Pool::Source,
            10 * 1024 * 1024,
            &mut ordinary_ledger,
            &mut NoopControl
        ),
        Err(ContractError::Invalid)
    );
    assert!(!fixture.root.join("result.json").exists());
}

/// Maintained generated duplicate-last semantics and intrinsic duplicate-safe records stay distinct.
#[test]
fn genuine_generated_duplicates_preserved_and_intrinsic_duplicates_refused() {
    let fixture = Fixture::new(6, false, false);
    std::fs::write(
        fixture.root.join(&fixture.generated[0]),
        br#"{"catalog":{"uuid":"private-invalid","uuid":"11111111-1111-4111-8111-111111111111"}}"#,
    )
    .unwrap();
    // Renew the actual recorded native approval through ordinary file producers, not hashes.
    for (state, actor, role, timestamp, assertions) in [
        (
            LifecycleState::InReview,
            "reviewer",
            DeclaredRole::Reviewer,
            "2026-08-02T00:00:00Z",
            Vec::<String>::new(),
        ),
        (
            LifecycleState::Approved,
            "approver",
            DeclaredRole::Approver,
            "2026-08-02T00:00:01Z",
            vec!["reviewer=reviewer".to_string()],
        ),
    ] {
        lifecycle::execute_transition(&TransitionOptions {
            record_path: &fixture.root.join("record.json"),
            next_state: state,
            actor_key: actor,
            role,
            timestamp,
            rationale: "private native duplicate fixture",
            assertions: &assertions,
            impact_finding_ids: &[],
            replacement_policy_key: None,
            replacement_version_key: None,
            apply: true,
            output: None,
        })
        .unwrap();
    }
    let (owner, mut ledger) = fixture.closure();
    assert_eq!(
        owner.members[2].pin.native_root_uuid.as_deref(),
        Some("11111111-1111-4111-8111-111111111111")
    );
    owner.verify_inputs(&mut ledger, &mut NoopControl).unwrap();
    drop(owner);
    let path = fixture.root.join("record.json");
    let raw = std::fs::read_to_string(&path).unwrap();
    let duplicated = format!("{{\"schema_version\":\"{}\",{}", record::SCHEMA_VERSION, &raw[1..]);
    std::fs::write(path, duplicated.as_bytes()).unwrap();
    let mut ledger = ContractLedger::default();
    let mut capture = ReviewCapture::new_lifecycle(
        &fixture.root,
        &[Path::new("result.json")],
        &mut ledger,
        &mut NoopControl,
    )
    .unwrap();
    assert!(matches!(
        read_pending(&mut capture, Path::new("locator.json"), &mut ledger, &mut NoopControl),
        Err(ContractError::Invalid)
    ));
    assert!(!fixture.root.join("result.json").exists());
}

/// Actual original control observations, without a production control/deadline replacement.
#[derive(Default)]
struct Control {
    /// Actual observed checkpoint calls in this one component invocation.
    calls: usize,
    /// A fixed one-based observed call, calibrated against the same real failed phase.
    at: Option<usize>,
    /// Return an ordinary Failed control cause instead of cancellation.
    failed: bool,
}
impl WorkControl for Control {
    /// Return the actual selected typed failure at its genuine observation.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
        self.calls += 1;
        if self.at == Some(self.calls) {
            if self.failed {
                Err(WorkError::Failed(Error::invalid()))
            } else {
                Err(WorkError::Interrupted(Interruption::CancelRequested))
            }
        } else {
            Ok(())
        }
    }
    /// The original ledger, rather than this probe, preserves every first observed stop.
    fn interruption(&self) -> Option<Interruption> {
        None
    }
}

/// Both native intrinsic/UUID ordinary failures take the actual original post-failure fence.
#[test]
fn genuine_native_failure_postfences_and_original_first_stop() {
    for generated in [false, true] {
        let raw =
            if generated { &br#"{"catalog":{"uuid":"private-invalid"}}"#[..] } else { &b"{}"[..] };
        let mut baseline = Control::default();
        let mut ledger = ContractLedger::default();
        let result = if generated {
            native::identity(Path::new("private.json"), raw, &mut ledger, &mut baseline).map(|_| ())
        } else {
            native::record(raw, &mut ledger, &mut baseline).map(|_| ())
        };
        assert!(matches!(result, Err(ContractError::Invalid | ContractError::Binding)));
        for failed in [false, true] {
            let mut control = Control { at: Some(baseline.calls), failed, ..Control::default() };
            let mut ledger = ContractLedger::default();
            let result = if generated {
                native::identity(Path::new("private.json"), raw, &mut ledger, &mut control)
                    .map(|_| ())
            } else {
                native::record(raw, &mut ledger, &mut control).map(|_| ())
            };
            let expected = if failed {
                ContractError::ControlFailed
            } else {
                ContractError::Interrupted(Interruption::CancelRequested)
            };
            assert_eq!(result, Err(expected));
            let calls = control.calls;
            assert_eq!(native::record(b"{}", &mut ledger, &mut control).err(), Some(expected));
            assert_eq!(control.calls, calls);
        }
    }
    let mut ledger = ContractLedger::default();
    assert_eq!(ledger.derived(33_554_433), Err(ContractError::Capacity));
    let mut control = Control { at: Some(1), failed: true, ..Control::default() };
    assert_eq!(
        native::record(b"{}", &mut ledger, &mut control).err(),
        Some(ContractError::Capacity)
    );
    assert_eq!(control.calls, 0);
}

/// Empty native JSON is an ordinary failure; real subsequent work and final stops share its caller.
#[test]
fn genuine_empty_native_failure_allows_real_continuation_and_keeps_postfence() {
    let fixture = Fixture::new(1, false, false);
    let record_bytes = std::fs::read(fixture.root.join("record.json")).unwrap();
    let generated_path = fixture.root.join(&fixture.generated[0]);
    let generated_bytes = std::fs::read(&generated_path).unwrap();
    for generated in [false, true] {
        for failed in [false, true] {
            let mut ledger = ContractLedger::default();
            let mut control = Control::default();
            let empty = if generated {
                native::identity(&generated_path, b"", &mut ledger, &mut control).map(|_| ())
            } else {
                native::record(b"", &mut ledger, &mut control).map(|_| ())
            };
            assert_eq!(empty, Err(ContractError::Invalid));
            let empty_calls = control.calls;
            assert!(empty_calls > 0);
            let continued = if generated {
                native::identity(&generated_path, &generated_bytes, &mut ledger, &mut control)
                    .map(|_| ())
            } else {
                native::record(&record_bytes, &mut ledger, &mut control).map(|_| ())
            };
            assert_eq!(continued, Ok(()));
            assert!(control.calls > empty_calls);
            control.at = Some(control.calls + empty_calls);
            control.failed = failed;
            let stopped = if generated {
                native::identity(&generated_path, b"", &mut ledger, &mut control).map(|_| ())
            } else {
                native::record(b"", &mut ledger, &mut control).map(|_| ())
            };
            let expected = if failed {
                ContractError::ControlFailed
            } else {
                ContractError::Interrupted(Interruption::CancelRequested)
            };
            assert_eq!(stopped, Err(expected));
            let calls = control.calls;
            let later = if generated {
                native::identity(&generated_path, &generated_bytes, &mut ledger, &mut control)
                    .map(|_| ())
            } else {
                native::record(&record_bytes, &mut ledger, &mut control).map(|_| ())
            };
            assert_eq!(later, Err(expected));
            assert_eq!(control.calls, calls);
        }
    }
    assert!(!fixture.root.join("result.json").exists());
}

/// Genuine pending-binding controls borrow this private maintained native fixture.
mod binding_tests {
    include!("lifecycle_binding_tests.rs");
}

/// Genuine new-output queue export controls borrow the private native receiver fixture.
mod queue_export_tests {
    include!("lifecycle_queue_export_tests.rs");
}

mod init_tests {
    include!("lifecycle_init_tests.rs");
}
