//! Genuine filesystem/control tests for the non-authorizing raw capture foundation.
//! Every positive traverses the real native readers; no proof/facts factory is used.

use super::*;
use std::cell::Cell;
use std::path::PathBuf;

/// Real bounded disposable project plus physically separate external control root.
struct Fixture {
    _dir: tempfile::TempDir,
    project: PathBuf,
    external: PathBuf,
}
impl Fixture {
    /// Canonicalize only the cfg fixture root before product no-follow traversal.
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let project = base.join("project");
        let external = base.join("external");
        std::fs::create_dir(&project).unwrap();
        std::fs::create_dir(&external).unwrap();
        std::fs::write(project.join("forge.mcp.json"), b"actual discovery original").unwrap();
        std::fs::write(project.join("forge.mcp.visibility.json"), b"actual visibility original")
            .unwrap();
        std::fs::write(
            external.join("forge.mcp.disclosure-decision.json"),
            b"actual final decision original",
        )
        .unwrap();
        std::fs::write(
            external.join("forge.mcp.index-build-intent.json"),
            b"actual intent original",
        )
        .unwrap();
        Self { _dir: dir, project, external }
    }
    /// Persist owned cfg fixture bytes, never a staged analyzed project.
    fn write(&self, path: &str, bytes: &[u8]) {
        std::fs::write(self.project.join(path), bytes).unwrap();
    }
}
/// Actual stop selected at an observed callback, without clock reset/deadline proof.
#[derive(Clone, Copy)]
enum Failure {
    Interrupted,
    Failed,
}
/// Independent externally armed observation of the same actual caller callbacks.
#[derive(Default)]
struct Events {
    calls: Cell<usize>,
    retain: Cell<usize>,
    stop_call: Cell<Option<usize>>,
    stop_retain: Cell<Option<usize>>,
}
/// Genuine `WorkControl` retains its own first stop and exposes actual phase counts.
struct Control {
    events: Rc<Events>,
    failure: Failure,
    stop: Option<Stop>,
}
impl Control {
    /// Keep one actual shared caller through capture, codec and all final fences.
    fn new(events: Rc<Events>, failure: Failure) -> Self {
        Self { events, failure, stop: None }
    }
}
impl WorkControl for Control {
    /// Observe real producer calls; the first selected failure remains sticky.
    fn checkpoint(&mut self, stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
        if let Some(stop) = &self.stop {
            return Err(stop.error());
        }
        let calls = self.events.calls.get() + 1;
        self.events.calls.set(calls);
        if stage == Stage::RetainPrepared {
            self.events.retain.set(self.events.retain.get() + 1);
        }
        if self.events.stop_call.get() == Some(calls)
            || stage == Stage::RetainPrepared
                && self.events.stop_retain.get() == Some(self.events.retain.get())
        {
            self.stop = Some(match self.failure {
                Failure::Interrupted => Stop::Interrupted(Interruption::CancelRequested),
                Failure::Failed => Stop::Failed(Error::new(
                    "cfg-control-stop",
                    "Synthetic control stopped.",
                    false,
                )),
            });
            return Err(self.stop.as_ref().unwrap().error());
        }
        Ok(())
    }
    /// Never infer a new interruption from ordinary native invalidity.
    fn interruption(&self) -> Option<Interruption> {
        match &self.stop {
            Some(Stop::Interrupted(reason)) => Some(*reason),
            _ => None,
        }
    }
}
/// Compare fixed safe failure identity without requiring capture/native Debug.
fn capacity(error: WorkError) {
    assert!(matches!(error, WorkError::Failed(error) if error.code == "mcp-capture-capacity"));
}

/// Both purposes read their actual fixed originals and retain an observed binary member.
#[test]
fn genuine_both_purposes_and_complete_binary_originals() {
    let fixture = Fixture::new();
    fixture.write("binary.bin", &[0, 255, 10, 13]);
    for (purpose, expected) in [
        (CapturePurpose::ServerRead, b"actual final decision original".as_slice()),
        (CapturePurpose::OfflineBuild, b"actual intent original".as_slice()),
    ] {
        let events = Rc::new(Events::default());
        let mut control = Control::new(events, Failure::Interrupted);
        let mut builder =
            CaptureBuilderV2::new(&fixture.project, &fixture.external, purpose, &mut control)
                .unwrap();
        assert_eq!(builder.configuration(Configuration::Purpose).unwrap(), Some(expected));
        builder
            .open_original("binary", Path::new("binary.bin"), 10, MissingPolicy::Required)
            .unwrap();
        builder.read_remaining().unwrap();
        let held = builder.finish().unwrap();
        assert!(held.purpose() == purpose);
        assert!(
            matches!(held.original("binary").unwrap(), Some(Observation::Present { bytes, raw_sha256 })
            if bytes == [0,255,10,13] && raw_sha256 == crate::hashing::sha256_hex(bytes))
        );
        held.verify_complete().unwrap();
    }
}

/// Native typed absence survives full raw close, with no zero-byte substitute.
#[test]
fn genuine_missing_and_empty_are_distinct() {
    let fixture = Fixture::new();
    fixture.write("empty.bin", b"");
    let mut control = Control::new(Rc::new(Events::default()), Failure::Interrupted);
    let mut builder = CaptureBuilderV2::new(
        &fixture.project,
        &fixture.external,
        CapturePurpose::OfflineBuild,
        &mut control,
    )
    .unwrap();
    builder
        .open_original("missing", Path::new("not-present/file.bin"), 10, MissingPolicy::Observe)
        .unwrap();
    builder.open_original("empty", Path::new("empty.bin"), 10, MissingPolicy::Required).unwrap();
    builder.read_remaining().unwrap();
    let held = builder.finish().unwrap();
    assert!(matches!(held.original("missing").unwrap(), Some(Observation::Absent)));
    assert!(
        matches!(held.original("empty").unwrap(), Some(Observation::Present { bytes, raw_sha256 }) if bytes.is_empty() && raw_sha256 == crate::hashing::sha256_hex(b""))
    );
}

/// A newly appeared original invalidates the actual same-owner absence at final fence.
#[test]
fn genuine_absence_appearance_refuses_final_owner() {
    let fixture = Fixture::new();
    let mut control = Control::new(Rc::new(Events::default()), Failure::Interrupted);
    let mut builder = CaptureBuilderV2::new(
        &fixture.project,
        &fixture.external,
        CapturePurpose::OfflineBuild,
        &mut control,
    )
    .unwrap();
    builder.open_original("future", Path::new("future.bin"), 10, MissingPolicy::Observe).unwrap();
    builder.read_remaining().unwrap();
    fixture.write("future.bin", b"new");
    assert!(builder.finish().is_err());
}

/// Complete present and configuration bytes are rechecked, including hidden registrations.
#[test]
fn genuine_hidden_and_configuration_drift_refuses() {
    for configuration in [false, true] {
        let fixture = Fixture::new();
        fixture.write("hidden.bin", b"original");
        let mut control = Control::new(Rc::new(Events::default()), Failure::Interrupted);
        let mut builder = CaptureBuilderV2::new(
            &fixture.project,
            &fixture.external,
            CapturePurpose::ServerRead,
            &mut control,
        )
        .unwrap();
        builder
            .open_original("hidden", Path::new("hidden.bin"), 20, MissingPolicy::Required)
            .unwrap();
        builder.read_remaining().unwrap();
        fixture.write(if configuration { "forge.mcp.json" } else { "hidden.bin" }, b"mutated!");
        assert!(builder.finish().is_err());
    }
}

/// Two real directory descendants are retained once; duplicate path reuse never reopens.
#[test]
fn genuine_unused_directory_has_complete_geometry_and_reuse() {
    let fixture = Fixture::new();
    std::fs::create_dir_all(fixture.project.join("unused/nested")).unwrap();
    let mut control = Control::new(Rc::new(Events::default()), Failure::Interrupted);
    let mut builder = CaptureBuilderV2::new(
        &fixture.project,
        &fixture.external,
        CapturePurpose::ServerRead,
        &mut control,
    )
    .unwrap();
    let before = builder.shared.ledger.borrow().handles;
    builder.capture_directory(Path::new("unused/nested")).unwrap();
    assert_eq!(builder.shared.ledger.borrow().handles, before + 2);
    assert_eq!(builder.directories[0].native.as_ref().unwrap().geometry().unwrap().handles(), 2);
    let bytes = builder.shared.ledger.borrow().bytes;
    builder.capture_directory(Path::new("unused/nested")).unwrap();
    assert_eq!(builder.shared.ledger.borrow().handles, before + 2);
    assert_eq!(builder.shared.ledger.borrow().bytes, bytes);
    builder.read_remaining().unwrap();
    let held = builder.finish().unwrap();
    let mut routes = Vec::new();
    held.visit_directories(&mut |route| {
        routes.push(route.to_path_buf());
        Ok(())
    })
    .unwrap();
    assert_eq!(routes, vec![PathBuf::from("unused/nested")]);
    held.verify_complete().unwrap();
}

/// Real no-delete Windows holders refuse replacement; Unix replacement fails final identity.
#[test]
fn genuine_unused_directory_replacement_or_native_denial() {
    let fixture = Fixture::new();
    std::fs::create_dir(fixture.project.join("unused")).unwrap();
    let mut control = Control::new(Rc::new(Events::default()), Failure::Interrupted);
    let mut builder = CaptureBuilderV2::new(
        &fixture.project,
        &fixture.external,
        CapturePurpose::ServerRead,
        &mut control,
    )
    .unwrap();
    builder.capture_directory(Path::new("unused")).unwrap();
    builder.read_remaining().unwrap();
    let result = std::fs::rename(fixture.project.join("unused"), fixture.project.join("previous"));
    #[cfg(windows)]
    {
        assert!(result.is_err());
        builder.finish().unwrap().verify_complete().unwrap();
    }
    #[cfg(unix)]
    {
        result.unwrap();
        std::fs::create_dir(fixture.project.join("unused")).unwrap();
        assert!(builder.finish().is_err());
    }
}

/// The global unchanged directory bound is tested with 65 real independently declared directories.
#[test]
fn genuine_global_directory_capacity() {
    let fixture = Fixture::new();
    for index in 0..65 {
        std::fs::create_dir(fixture.project.join(format!("d{index}"))).unwrap();
    }
    let mut control = Control::new(Rc::new(Events::default()), Failure::Interrupted);
    let mut builder = CaptureBuilderV2::new(
        &fixture.project,
        &fixture.external,
        CapturePurpose::ServerRead,
        &mut control,
    )
    .unwrap();
    for index in 0..64 {
        builder.capture_directory(Path::new(&format!("d{index}"))).unwrap();
    }
    capacity(builder.capture_directory(Path::new("d64")).err().unwrap());
    capacity(
        DeclarationAdmission::checkpoint(&mut builder.admission(), Stage::PrepareDomain)
            .err()
            .unwrap(),
    );
}

/// Complete actual roots and portable file namespaces cannot fold/overlap into a second owner.
#[test]
fn genuine_root_and_registration_aliases_refuse() {
    let fixture = Fixture::new();
    let mut control = Control::new(Rc::new(Events::default()), Failure::Interrupted);
    assert!(
        CaptureBuilderV2::new(
            &fixture.project,
            &fixture.project,
            CapturePurpose::ServerRead,
            &mut control
        )
        .is_err()
    );
    let mut control = Control::new(Rc::new(Events::default()), Failure::Interrupted);
    let mut builder = CaptureBuilderV2::new(
        &fixture.project,
        &fixture.external,
        CapturePurpose::ServerRead,
        &mut control,
    )
    .unwrap();
    fixture.write("data.bin", b"actual");
    builder.open_original("data", Path::new("data.bin"), 10, MissingPolicy::Required).unwrap();
    assert!(
        builder.open_original("other", Path::new("DATA.bin"), 10, MissingPolicy::Required).is_err()
    );
    assert!(
        builder
            .open_original("escape", Path::new("../escape.bin"), 10, MissingPolicy::Observe)
            .is_err()
    );
    assert!(
        builder
            .open_original("directory", Path::new("data.bin/child"), 10, MissingPolicy::Observe)
            .is_err()
    );
}

/// Every original is opened before declared reads; later registration and unread close refuse.
#[test]
fn genuine_registration_read_chronology() {
    let fixture = Fixture::new();
    fixture.write("a.bin", b"a");
    fixture.write("b.bin", b"b");
    let mut control = Control::new(Rc::new(Events::default()), Failure::Interrupted);
    let mut builder = CaptureBuilderV2::new(
        &fixture.project,
        &fixture.external,
        CapturePurpose::ServerRead,
        &mut control,
    )
    .unwrap();
    builder.open_original("a", Path::new("a.bin"), 10, MissingPolicy::Required).unwrap();
    builder.read_original("a").unwrap();
    assert!(builder.open_original("b", Path::new("b.bin"), 10, MissingPolicy::Required).is_err());
    builder.finish().unwrap();
    let mut control = Control::new(Rc::new(Events::default()), Failure::Interrupted);
    let mut builder = CaptureBuilderV2::new(
        &fixture.project,
        &fixture.external,
        CapturePurpose::ServerRead,
        &mut control,
    )
    .unwrap();
    builder.open_original("a", Path::new("a.bin"), 10, MissingPolicy::Required).unwrap();
    assert!(builder.finish().is_err());
}

/// A actual joint-byte failure wins later caller interruption/failed control on the same ledger.
#[test]
fn genuine_joint_capacity_is_sticky_before_later_control_stop() {
    for failure in [Failure::Interrupted, Failure::Failed] {
        let fixture = Fixture::new();
        let bytes = vec![b'x'; 9 * 1024 * 1024];
        for index in 0..6 {
            fixture.write(&format!("large{index}.bin"), &bytes);
        }
        drop(bytes);
        let events = Rc::new(Events::default());
        let mut control = Control::new(Rc::clone(&events), failure);
        let mut builder = CaptureBuilderV2::new(
            &fixture.project,
            &fixture.external,
            CapturePurpose::ServerRead,
            &mut control,
        )
        .unwrap();
        for index in 0..6 {
            builder
                .open_original(
                    &format!("large{index}"),
                    Path::new(&format!("large{index}.bin")),
                    MAX_FILE,
                    MissingPolicy::Required,
                )
                .unwrap();
        }
        capacity(builder.read_remaining().err().unwrap());
        let calls = events.calls.get();
        events.stop_call.set(Some(calls + 1));
        capacity(
            DeclarationAdmission::checkpoint(&mut builder.admission(), Stage::PrepareDomain)
                .err()
                .unwrap(),
        );
        assert_eq!(events.calls.get(), calls);
        assert!(builder.finish().is_err());
    }
}

/// Independently calibrate actual missing-original failure and stop only at its post-phase fence.
#[test]
fn genuine_ordinary_native_failure_keeps_postphase_stop() {
    let fixture = Fixture::new();
    std::fs::remove_file(fixture.external.join("forge.mcp.disclosure-decision.json")).unwrap();
    let baseline = Rc::new(Events::default());
    let mut control = Control::new(Rc::clone(&baseline), Failure::Interrupted);
    assert!(
        CaptureBuilderV2::new(
            &fixture.project,
            &fixture.external,
            CapturePurpose::ServerRead,
            &mut control
        )
        .is_err()
    );
    let post = baseline.calls.get();
    assert!(post > 0);
    for failure in [Failure::Interrupted, Failure::Failed] {
        let events = Rc::new(Events::default());
        events.stop_call.set(Some(post));
        let mut control = Control::new(events, failure);
        let error = CaptureBuilderV2::new(
            &fixture.project,
            &fixture.external,
            CapturePurpose::ServerRead,
            &mut control,
        )
        .err()
        .unwrap();
        match failure {
            Failure::Interrupted => {
                assert!(matches!(error, WorkError::Interrupted(Interruption::CancelRequested)));
            }
            Failure::Failed => assert!(
                matches!(error, WorkError::Failed(error) if error.code == "cfg-control-stop")
            ),
        }
    }
}

/// Calibrate complete final fences on the SAME real held owner, ledger and caller.
#[test]
fn genuine_same_owner_final_stop_and_actual_drop_release() {
    for failure in [Failure::Interrupted, Failure::Failed] {
        let fixture = Fixture::new();
        fixture.write("member.bin", b"actual complete raw original");
        let events = Rc::new(Events::default());
        let mut control = Control::new(Rc::clone(&events), failure);
        let mut builder = CaptureBuilderV2::new(
            &fixture.project,
            &fixture.external,
            CapturePurpose::ServerRead,
            &mut control,
        )
        .unwrap();
        builder
            .open_original("member", Path::new("member.bin"), 100, MissingPolicy::Required)
            .unwrap();
        builder.read_remaining().unwrap();
        let held = builder.finish().unwrap();
        let start = events.retain.get();
        held.verify_complete().unwrap();
        let complete = events.retain.get() - start;
        assert!(complete > 1);
        events.stop_retain.set(Some(events.retain.get() + complete));
        let error = held.verify_complete().err().unwrap();
        match failure {
            Failure::Interrupted => {
                assert!(matches!(error, WorkError::Interrupted(Interruption::CancelRequested)));
            }
            Failure::Failed => assert!(
                matches!(error, WorkError::Failed(error) if error.code == "cfg-control-stop")
            ),
        }
        let admission = held.admission();
        assert!(admission.shared.ledger.borrow().handles > 0);
        drop(held);
        assert_eq!(admission.shared.ledger.borrow().handles, 0);
        assert_eq!(admission.shared.ledger.borrow().bytes, 0);
        assert!(admission.shared.ledger.borrow().work > 0);
        assert_eq!(admission.shared.ledger.borrow().originals, 4);
    }
}

/// Unix links are rejected by actual no-follow native reads, including directory declarations.
#[cfg(unix)]
#[test]
fn genuine_native_symlink_and_hardlink_refusal() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new();
    fixture.write("actual.bin", b"actual");
    std::fs::create_dir(fixture.project.join("actual-directory")).unwrap();
    symlink("actual.bin", fixture.project.join("linked.bin")).unwrap();
    symlink("actual-directory", fixture.project.join("linked-directory")).unwrap();
    let mut control = Control::new(Rc::new(Events::default()), Failure::Interrupted);
    let mut builder = CaptureBuilderV2::new(
        &fixture.project,
        &fixture.external,
        CapturePurpose::ServerRead,
        &mut control,
    )
    .unwrap();
    assert!(
        builder
            .open_original("linked", Path::new("linked.bin"), 10, MissingPolicy::Observe)
            .is_err()
    );
    assert!(builder.capture_directory(Path::new("linked-directory")).is_err());
    builder.open_original("actual", Path::new("actual.bin"), 10, MissingPolicy::Required).unwrap();
    std::fs::hard_link(fixture.project.join("actual.bin"), fixture.project.join("alias.bin"))
        .unwrap();
    assert!(
        builder
            .open_original("alias", Path::new("alias.bin"), 10, MissingPolicy::Required)
            .is_err()
    );
    assert!(builder.read_remaining().is_err());
}

/// Maintained linkage roots require a nonempty Normal descendant; bare/dot roots refuse.
#[test]
fn actual_zero_component_directory_routes_refuse_without_native_omission() {
    let fixture = Fixture::new();
    let mut control = Control::new(Rc::new(Events::default()), Failure::Interrupted);
    let mut builder = CaptureBuilderV2::new(
        &fixture.project,
        &fixture.external,
        CapturePurpose::ServerRead,
        &mut control,
    )
    .unwrap();
    let handles = builder.shared.ledger.borrow().handles;
    for route in ["", ".", "..", "/"] {
        assert!(builder.capture_directory(Path::new(route)).is_err());
    }
    assert_eq!(builder.directories.len(), 0);
    assert_eq!(builder.shared.ledger.borrow().handles, handles);
    assert!(builder.capture_directory(Path::new("required-missing-directory")).is_err());
}

/// Actual owner iteration includes all declarations in opening order and typed absence.
#[test]
fn complete_actual_member_iteration_preserves_order_and_absence() {
    let fixture = Fixture::new();
    fixture.write("first.bin", b"first");
    fixture.write("hidden.bin", b"hidden");
    let mut control = Control::new(Rc::new(Events::default()), Failure::Interrupted);
    let mut builder = CaptureBuilderV2::new(
        &fixture.project,
        &fixture.external,
        CapturePurpose::ServerRead,
        &mut control,
    )
    .unwrap();
    builder.open_original("z", Path::new("first.bin"), 100, MissingPolicy::Required).unwrap();
    builder.open_original("missing", Path::new("absent.bin"), 100, MissingPolicy::Observe).unwrap();
    builder
        .open_original("a-hidden", Path::new("hidden.bin"), 100, MissingPolicy::Required)
        .unwrap();
    builder.read_remaining().unwrap();
    let held = builder.finish().unwrap();
    let mut rows = Vec::new();
    held.visit_declared(&mut |member| {
        rows.push((
            member.key().to_owned(),
            member.route().to_path_buf(),
            match member.observation() {
                Observation::Present { bytes, .. } => Some(bytes.to_vec()),
                Observation::Absent => None,
            },
        ));
        Ok(())
    })
    .unwrap();
    assert_eq!(
        rows,
        vec![
            ("z".to_owned(), PathBuf::from("first.bin"), Some(b"first".to_vec())),
            ("missing".to_owned(), PathBuf::from("absent.bin"), None),
            ("a-hidden".to_owned(), PathBuf::from("hidden.bin"), Some(b"hidden".to_vec()))
        ]
    );
    held.verify_complete().unwrap();
}

/// A whole ordinary visitor failure is post-fenced on the SAME held originals/control.
#[test]
fn actual_member_callback_failure_keeps_postphase_stop() {
    for failure in [Failure::Interrupted, Failure::Failed] {
        let fixture = Fixture::new();
        fixture.write("member.bin", b"actual");
        let events = Rc::new(Events::default());
        let mut control = Control::new(Rc::clone(&events), failure);
        let mut builder = CaptureBuilderV2::new(
            &fixture.project,
            &fixture.external,
            CapturePurpose::ServerRead,
            &mut control,
        )
        .unwrap();
        builder
            .open_original("member", Path::new("member.bin"), 100, MissingPolicy::Required)
            .unwrap();
        builder.read_remaining().unwrap();
        let held = builder.finish().unwrap();
        let start = events.calls.get();
        assert!(matches!(held.visit_declared(&mut |_| Err(invalid())).err().unwrap(),
            WorkError::Failed(error) if error.code == "invalid-request"));
        let calls = events.calls.get() - start;
        assert!(calls > 1);
        events.stop_call.set(Some(events.calls.get() + calls));
        let error = held.visit_declared(&mut |_| Err(invalid())).err().unwrap();
        match failure {
            Failure::Interrupted => {
                assert!(matches!(error, WorkError::Interrupted(Interruption::CancelRequested)));
            }
            Failure::Failed => assert!(
                matches!(error, WorkError::Failed(error) if error.code == "cfg-control-stop")
            ),
        }
    }
}

/// Owned data returned by a genuine raw-byte consumer drops before its ticket is released.
#[test]
fn actual_data_retention_and_concrete_controller_use_one_owner() {
    /// Test-owned native-byte copy with observable real Vec destruction chronology.
    struct OwnedCopy {
        bytes: Option<Vec<u8>>,
        ledger: Rc<RefCell<Ledger>>,
        baseline: usize,
        dropped: Rc<Cell<bool>>,
    }
    impl Drop for OwnedCopy {
        /// Observe retained capacity while the actual copy drops, then mark completion.
        fn drop(&mut self) {
            assert_eq!(
                self.ledger.borrow().bytes,
                self.baseline + self.bytes.as_ref().unwrap().len()
            );
            drop(self.bytes.take());
            self.dropped.set(true);
        }
    }
    let fixture = Fixture::new();
    fixture.write("member.bin", b"complete actual native raw");
    let events = Rc::new(Events::default());
    let mut control = Control::new(Rc::clone(&events), Failure::Interrupted);
    let mut builder = CaptureBuilderV2::new(
        &fixture.project,
        &fixture.external,
        CapturePurpose::ServerRead,
        &mut control,
    )
    .unwrap();
    builder.open_original("member", Path::new("member.bin"), 100, MissingPolicy::Required).unwrap();
    builder.read_remaining().unwrap();
    let held = builder.finish().unwrap();
    held.with_control(|actual| assert!(Rc::ptr_eq(&actual.events, &events)));
    let Some(Observation::Present { bytes, .. }) = held.original("member").unwrap() else {
        panic!("real member absent")
    };
    let mut admission = held.admission();
    let baseline = admission.shared.ledger.borrow().bytes;
    let dropped = Rc::new(Cell::new(false));
    let owned = admission
        .retain(bytes.len(), |same| {
            DeclarationAdmission::charge(same, bytes.len())?;
            Ok(OwnedCopy {
                bytes: Some(bytes.to_vec()),
                ledger: Rc::clone(&same.shared.ledger),
                baseline,
                dropped: Rc::clone(&dropped),
            })
        })
        .unwrap();
    assert_eq!(owned.value().bytes.as_deref(), Some(bytes));
    assert_eq!(admission.shared.ledger.borrow().bytes, baseline + bytes.len());
    drop(owned);
    assert!(dropped.get());
    assert_eq!(admission.shared.ledger.borrow().bytes, baseline);
    held.verify_complete().unwrap();
}

/// The original joint cap refuses before a genuine data callback allocates anything.
#[test]
fn actual_data_capacity_precedes_production_and_stays_sticky() {
    let fixture = Fixture::new();
    let events = Rc::new(Events::default());
    let mut control = Control::new(Rc::clone(&events), Failure::Failed);
    let builder = CaptureBuilderV2::new(
        &fixture.project,
        &fixture.external,
        CapturePurpose::OfflineBuild,
        &mut control,
    )
    .unwrap();
    let held = builder.finish().unwrap();
    let mut admission = held.admission();
    let invoked = Cell::new(false);
    let result = admission.retain(MAX_BYTES, |_| {
        invoked.set(true);
        Ok(vec![b'x'; MAX_BYTES])
    });
    capacity(result.err().unwrap());
    assert!(!invoked.get());
    let calls = events.calls.get();
    events.stop_call.set(Some(calls + 1));
    capacity(DeclarationAdmission::checkpoint(&mut admission, Stage::PrepareDomain).err().unwrap());
    assert_eq!(events.calls.get(), calls);
}

/// Actual invalid UTF-8 member data still reaches the retained data phase's final stop.
#[test]
fn actual_retained_data_failure_keeps_same_owner_postphase_stop() {
    for failure in [Failure::Interrupted, Failure::Failed] {
        let fixture = Fixture::new();
        fixture.write("invalid.bin", &[0xff]);
        let events = Rc::new(Events::default());
        let mut control = Control::new(Rc::clone(&events), failure);
        let mut builder = CaptureBuilderV2::new(
            &fixture.project,
            &fixture.external,
            CapturePurpose::ServerRead,
            &mut control,
        )
        .unwrap();
        builder
            .open_original("invalid", Path::new("invalid.bin"), 100, MissingPolicy::Required)
            .unwrap();
        builder.read_remaining().unwrap();
        let held = builder.finish().unwrap();
        let Some(Observation::Present { bytes, .. }) = held.original("invalid").unwrap() else {
            panic!("actual member missing")
        };
        let mut admission = held.admission();
        let baseline_bytes = admission.shared.ledger.borrow().bytes;
        let start = events.calls.get();
        let first = admission.retain(100, |same| {
            DeclarationAdmission::charge(same, bytes.len())?;
            std::str::from_utf8(bytes).map(str::to_owned).map_err(|_| invalid())
        });
        assert!(
            matches!(first.err().unwrap(), WorkError::Failed(error) if error.code == "invalid-request")
        );
        assert_eq!(admission.shared.ledger.borrow().bytes, baseline_bytes);
        let calls = events.calls.get() - start;
        assert!(calls > 1);
        events.stop_call.set(Some(events.calls.get() + calls));
        let second = admission.retain(100, |same| {
            DeclarationAdmission::charge(same, bytes.len())?;
            std::str::from_utf8(bytes).map(str::to_owned).map_err(|_| invalid())
        });
        match failure {
            Failure::Interrupted => assert!(matches!(
                second.err().unwrap(),
                WorkError::Interrupted(Interruption::CancelRequested)
            )),
            Failure::Failed => assert!(
                matches!(second.err().unwrap(), WorkError::Failed(error) if error.code == "cfg-control-stop")
            ),
        }
        assert_eq!(admission.shared.ledger.borrow().bytes, baseline_bytes);
    }
}

/// The unchanged 1001 original attempts count genuine failed Required native opens too.
#[test]
fn genuine_original_attempt_bound_precedes_later_control_failure() {
    for failure in [Failure::Interrupted, Failure::Failed] {
        let fixture = Fixture::new();
        let events = Rc::new(Events::default());
        let mut control = Control::new(Rc::clone(&events), failure);
        let mut builder = CaptureBuilderV2::new(
            &fixture.project,
            &fixture.external,
            CapturePurpose::ServerRead,
            &mut control,
        )
        .unwrap();
        let original_handles = builder.shared.ledger.borrow().handles;
        let original_bytes = builder.shared.ledger.borrow().bytes;
        let before = builder.shared.ledger.borrow().originals;
        assert_eq!(before, 3);
        let initial_work = builder.shared.ledger.borrow().work;
        let first = builder.open_original(
            "always-missing",
            Path::new("required-missing.bin"),
            100,
            MissingPolicy::Required,
        );
        assert!(
            matches!(first.err().unwrap(), WorkError::Failed(error) if error.code == "resource-containment")
        );
        let per_attempt = builder.shared.ledger.borrow().work - initial_work;
        assert!(per_attempt > 0);
        let remaining = MAX_ORIGINALS - builder.shared.ledger.borrow().originals;
        // Source-calibrate genuine constant three-config scans, not a lowered work bound.
        assert!(
            builder.shared.ledger.borrow().work + remaining * per_attempt + per_attempt < MAX_WORK
        );
        for _ in 0..remaining {
            let result = builder.open_original(
                "always-missing",
                Path::new("required-missing.bin"),
                100,
                MissingPolicy::Required,
            );
            assert!(
                matches!(result.err().unwrap(), WorkError::Failed(error) if error.code == "resource-containment")
            );
        }
        assert_eq!(builder.shared.ledger.borrow().originals, MAX_ORIGINALS);
        assert_eq!(builder.files.len(), 3);
        assert_eq!(builder.shared.ledger.borrow().handles, original_handles);
        assert_eq!(builder.shared.ledger.borrow().bytes, original_bytes);
        assert!(!fixture.project.join("required-missing.bin").exists());
        capacity(
            builder
                .open_original(
                    "always-missing",
                    Path::new("required-missing.bin"),
                    100,
                    MissingPolicy::Required,
                )
                .err()
                .unwrap(),
        );
        assert_eq!(builder.shared.ledger.borrow().originals, MAX_ORIGINALS);
        let calls = events.calls.get();
        events.stop_call.set(Some(calls + 1));
        capacity(
            DeclarationAdmission::checkpoint(&mut builder.admission(), Stage::PrepareDomain)
                .err()
                .unwrap(),
        );
        assert_eq!(events.calls.get(), calls);
    }
}

/// Real complete directories reach the unchanged 1001 reservation peak; the next native open refuses.
/// Retained geometry is separate: no assertion invents 1001 simultaneously retained handles.
#[test]
fn genuine_actual_directory_geometry_and_unchanged_handle_peak() {
    /// Create a genuine unique complete native directory path in the existing private fixture.
    fn real_directory(fixture: &Fixture, index: usize, depth: usize) -> PathBuf {
        let mut spelling = format!("peak{index}");
        for _ in 1..depth {
            spelling.push_str("/n");
        }
        let route = PathBuf::from(spelling);
        std::fs::create_dir_all(fixture.project.join(&route)).unwrap();
        route
    }
    let fixture = Fixture::new();
    let events = Rc::new(Events::default());
    let mut control = Control::new(Rc::clone(&events), Failure::Interrupted);
    let mut builder = CaptureBuilderV2::new(
        &fixture.project,
        &fixture.external,
        CapturePurpose::OfflineBuild,
        &mut control,
    )
    .unwrap();
    let baseline = builder.shared.ledger.borrow().handles;
    let root = builder.root(RootSlot::Project).native().geometry().unwrap();
    let root_handles = root.handles();
    let probe = real_directory(&fixture, 0, 16);
    let envelope =
        fresh::descendant_envelope(builder.root(RootSlot::Project).native(), &probe).unwrap();
    builder.capture_directory(&probe).unwrap();
    let probe_handles =
        builder.directories.last().unwrap().native.as_ref().unwrap().geometry().unwrap().handles();
    assert_eq!(probe_handles, 16);
    assert_eq!(envelope.handles(), probe_handles + root_handles + 2);
    assert_eq!(builder.shared.ledger.borrow().handles, baseline + probe_handles);
    let target_increment = MAX_HANDLES - baseline - root_handles - 2;
    let mut remaining = target_increment - probe_handles;
    let mut index = 1;
    while remaining > 0 {
        let planned = remaining.min(probe_handles);
        let route = real_directory(&fixture, index, planned);
        let previous = builder.shared.ledger.borrow().handles;
        builder.capture_directory(&route).unwrap();
        let actual = builder
            .directories
            .last()
            .unwrap()
            .native
            .as_ref()
            .unwrap()
            .geometry()
            .unwrap()
            .handles();
        assert_eq!(actual, planned);
        assert_eq!(builder.shared.ledger.borrow().handles, previous + actual);
        remaining -= actual;
        index += 1;
    }
    let retained = builder.shared.ledger.borrow().handles;
    assert_eq!(retained, MAX_HANDLES - root_handles - 2);
    assert_eq!(builder.shared.ledger.borrow().peak_handles, MAX_HANDLES);
    assert!(builder.directories.len() < MAX_DIRECTORIES);
    assert!(builder.shared.ledger.borrow().work < MAX_WORK);
    assert!(builder.shared.ledger.borrow().bytes < MAX_BYTES);
    let overflow = real_directory(&fixture, index, 1);
    let next =
        fresh::descendant_envelope(builder.root(RootSlot::Project).native(), &overflow).unwrap();
    assert_eq!(retained + next.handles(), MAX_HANDLES + 1);
    let count = builder.directories.len();
    capacity(builder.capture_directory(&overflow).err().unwrap());
    assert_eq!(builder.directories.len(), count);
    assert_eq!(builder.shared.ledger.borrow().handles, retained);
    let calls = events.calls.get();
    events.stop_call.set(Some(calls + 1));
    capacity(
        DeclarationAdmission::checkpoint(&mut builder.admission(), Stage::RetainPrepared)
            .err()
            .unwrap(),
    );
    assert_eq!(events.calls.get(), calls);
}

/// Complete generation is stable and its actual owned digest retains exactly64 charged bytes.
#[test]
fn genuine_generation_complete_stability_and_owned_digest_charge() {
    let fixture = Fixture::new();
    fixture.write("hidden.bin", b"actual complete hidden original");
    std::fs::create_dir(fixture.project.join("unused")).unwrap();
    let mut control = Control::new(Rc::new(Events::default()), Failure::Interrupted);
    let mut builder = CaptureBuilderV2::new(
        &fixture.project,
        &fixture.external,
        CapturePurpose::ServerRead,
        &mut control,
    )
    .unwrap();
    builder.open_original("hidden", Path::new("hidden.bin"), 100, MissingPolicy::Required).unwrap();
    builder.open_original("absent", Path::new("absent.bin"), 100, MissingPolicy::Observe).unwrap();
    builder.capture_directory(Path::new("unused")).unwrap();
    builder.read_remaining().unwrap();
    let held = builder.finish().unwrap();
    let before_bytes = held.builder.shared.ledger.borrow().bytes;
    let before_work = held.builder.shared.ledger.borrow().work;
    let first = held.generation_digest().unwrap();
    assert_eq!(first.value().len(), 64);
    assert!(
        first.value().bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    );
    assert_eq!(held.builder.shared.ledger.borrow().bytes, before_bytes + 64);
    let second = held.generation_digest().unwrap();
    assert_eq!(first.value(), second.value());
    assert_eq!(held.builder.shared.ledger.borrow().bytes, before_bytes + 128);
    assert!(held.builder.shared.ledger.borrow().work > before_work);
    drop(first);
    drop(second);
    assert_eq!(held.builder.shared.ledger.borrow().bytes, before_bytes);
    assert!(held.builder.shared.ledger.borrow().work > before_work);
    held.verify_complete().unwrap();
}

/// A genuine same-byte replacement changes generation after the old native owner drops.
#[test]
fn genuine_generation_byte_equal_instance_replacement_changes() {
    let fixture = Fixture::new();
    fixture.write("hidden.bin", b"same exact bytes");
    let before = {
        let mut control = Control::new(Rc::new(Events::default()), Failure::Interrupted);
        let mut builder = CaptureBuilderV2::new(
            &fixture.project,
            &fixture.external,
            CapturePurpose::ServerRead,
            &mut control,
        )
        .unwrap();
        builder
            .open_original("hidden", Path::new("hidden.bin"), 100, MissingPolicy::Required)
            .unwrap();
        builder.read_remaining().unwrap();
        let held = builder.finish().unwrap();
        held.generation_digest().unwrap().value().clone()
    };
    // Both regular instances exist simultaneously before replacing the old spelling.
    fixture.write("replacement.bin", b"same exact bytes");
    std::fs::remove_file(fixture.project.join("hidden.bin")).unwrap();
    std::fs::rename(fixture.project.join("replacement.bin"), fixture.project.join("hidden.bin"))
        .unwrap();
    let mut control = Control::new(Rc::new(Events::default()), Failure::Interrupted);
    let mut builder = CaptureBuilderV2::new(
        &fixture.project,
        &fixture.external,
        CapturePurpose::ServerRead,
        &mut control,
    )
    .unwrap();
    builder.open_original("hidden", Path::new("hidden.bin"), 100, MissingPolicy::Required).unwrap();
    builder.read_remaining().unwrap();
    let held = builder.finish().unwrap();
    let after = held.generation_digest().unwrap();
    assert_ne!(&before, after.value());
    held.verify_complete().unwrap();
}

/// Typed absence preserves actual parent identity and all unused declared directories contribute.
#[test]
fn genuine_generation_absence_ancestors_and_unused_directory_changes() {
    for replace_absence_parent in [true, false] {
        let fixture = Fixture::new();
        std::fs::create_dir(fixture.project.join("parent")).unwrap();
        std::fs::create_dir(fixture.project.join("unused")).unwrap();
        let before = {
            let mut control = Control::new(Rc::new(Events::default()), Failure::Interrupted);
            let mut builder = CaptureBuilderV2::new(
                &fixture.project,
                &fixture.external,
                CapturePurpose::OfflineBuild,
                &mut control,
            )
            .unwrap();
            builder
                .open_original(
                    "absent",
                    Path::new("parent/missing.bin"),
                    100,
                    MissingPolicy::Observe,
                )
                .unwrap();
            builder.capture_directory(Path::new("unused")).unwrap();
            builder.read_remaining().unwrap();
            let held = builder.finish().unwrap();
            held.generation_digest().unwrap().value().clone()
        };
        let path = if replace_absence_parent { "parent" } else { "unused" };
        std::fs::rename(fixture.project.join(path), fixture.project.join("old-directory")).unwrap();
        std::fs::create_dir(fixture.project.join(path)).unwrap();
        let mut control = Control::new(Rc::new(Events::default()), Failure::Interrupted);
        let mut builder = CaptureBuilderV2::new(
            &fixture.project,
            &fixture.external,
            CapturePurpose::OfflineBuild,
            &mut control,
        )
        .unwrap();
        builder
            .open_original("absent", Path::new("parent/missing.bin"), 100, MissingPolicy::Observe)
            .unwrap();
        builder.capture_directory(Path::new("unused")).unwrap();
        builder.read_remaining().unwrap();
        let held = builder.finish().unwrap();
        assert_ne!(&before, held.generation_digest().unwrap().value());
        held.verify_complete().unwrap();
    }
}

/// Fixed configuration bytes and hidden byte edits change the complete original generation.
#[test]
fn genuine_generation_all_configuration_and_hidden_raw_bytes_change() {
    for (purpose, path, external) in [
        (CapturePurpose::ServerRead, "forge.mcp.json", false),
        (CapturePurpose::ServerRead, "forge.mcp.visibility.json", false),
        (CapturePurpose::ServerRead, "hidden.bin", false),
        (CapturePurpose::ServerRead, "forge.mcp.disclosure-decision.json", true),
        (CapturePurpose::OfflineBuild, "forge.mcp.index-build-intent.json", true),
    ] {
        let fixture = Fixture::new();
        fixture.write("hidden.bin", b"original");
        let before = {
            let mut control = Control::new(Rc::new(Events::default()), Failure::Interrupted);
            let mut builder =
                CaptureBuilderV2::new(&fixture.project, &fixture.external, purpose, &mut control)
                    .unwrap();
            builder
                .open_original("hidden", Path::new("hidden.bin"), 100, MissingPolicy::Required)
                .unwrap();
            builder.read_remaining().unwrap();
            let held = builder.finish().unwrap();
            held.generation_digest().unwrap().value().clone()
        };
        if external {
            std::fs::write(fixture.external.join(path), b"different actual complete bytes")
                .unwrap();
        } else {
            fixture.write(path, b"different actual complete bytes");
        }
        let mut control = Control::new(Rc::new(Events::default()), Failure::Interrupted);
        let mut builder =
            CaptureBuilderV2::new(&fixture.project, &fixture.external, purpose, &mut control)
                .unwrap();
        builder
            .open_original("hidden", Path::new("hidden.bin"), 100, MissingPolicy::Required)
            .unwrap();
        builder.read_remaining().unwrap();
        let held = builder.finish().unwrap();
        assert_ne!(&before, held.generation_digest().unwrap().value());
        held.verify_complete().unwrap();
    }
}

/// Entry purpose, registration key and typed absence remain distinct complete framed fields.
#[test]
fn genuine_generation_purpose_registration_and_absence_are_framed() {
    let fixture = Fixture::new();
    fixture.write("empty.bin", b"");
    let mut generations = Vec::new();
    for (purpose, key, path, missing) in [
        (CapturePurpose::ServerRead, "first", "empty.bin", MissingPolicy::Required),
        (CapturePurpose::OfflineBuild, "first", "empty.bin", MissingPolicy::Required),
        (CapturePurpose::ServerRead, "second", "empty.bin", MissingPolicy::Required),
        (CapturePurpose::ServerRead, "first", "absent.bin", MissingPolicy::Observe),
    ] {
        let mut control = Control::new(Rc::new(Events::default()), Failure::Interrupted);
        let mut builder =
            CaptureBuilderV2::new(&fixture.project, &fixture.external, purpose, &mut control)
                .unwrap();
        builder.open_original(key, Path::new(path), 100, missing).unwrap();
        builder.read_remaining().unwrap();
        let held = builder.finish().unwrap();
        generations.push(held.generation_digest().unwrap().value().clone());
    }
    for left in 0..generations.len() {
        for right in left + 1..generations.len() {
            assert_ne!(generations[left], generations[right]);
        }
    }
}

/// Late same-original control stops reject a partial digest and release its actual64-byte ticket.
#[test]
fn genuine_generation_control_stops_preserve_first_and_release_owned_ticket() {
    for failure in [Failure::Interrupted, Failure::Failed] {
        let fixture = Fixture::new();
        let events = Rc::new(Events::default());
        let mut control = Control::new(Rc::clone(&events), failure);
        let mut builder = CaptureBuilderV2::new(
            &fixture.project,
            &fixture.external,
            CapturePurpose::ServerRead,
            &mut control,
        )
        .unwrap();
        builder.read_remaining().unwrap();
        let held = builder.finish().unwrap();
        let before_bytes = held.builder.shared.ledger.borrow().bytes;
        events.stop_call.set(Some(events.calls.get() + 15));
        let error = held.generation_digest().err().unwrap();
        match failure {
            Failure::Interrupted => {
                assert!(matches!(error, WorkError::Interrupted(Interruption::CancelRequested)));
            }
            Failure::Failed => assert!(
                matches!(error, WorkError::Failed(error) if error.code == "cfg-control-stop")
            ),
        }
        assert_eq!(held.builder.shared.ledger.borrow().bytes, before_bytes);
        let calls = events.calls.get();
        assert!(held.generation_digest().is_err());
        assert_eq!(events.calls.get(), calls);
        assert_eq!(held.builder.shared.ledger.borrow().bytes, before_bytes);
    }
}
