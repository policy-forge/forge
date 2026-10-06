//! Proposed genuine capture controls; no native approval, cohort or proof is fabricated.

use super::*;
use crate::evidence_capture::{CaptureCharge, CaptureSession};
use crate::workspace::preparation::{
    Interruption, NoopControl, ProgressUpdate, Stage, WorkError, WorkResult,
};
use std::path::PathBuf;

/// Create real confined originals for the actual two-Queue/three-Auxiliary input geometry.
fn originals() -> (tempfile::TempDir, PathBuf) {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    for name in ["old.json", "new.json", "sources.json", "impact.json", "links.json", "native.json"]
    {
        std::fs::write(root.join(name), name.as_bytes()).unwrap();
    }
    (directory, root)
}

/// Both capture orders retain actual purposes; ordinary and S3 binders reject the same owner.
#[test]
fn actual_queue_purposes_survive_both_orders_and_refuse_ordinary_binders() {
    for purposes in [
        [QueuePurpose::HistoricalOld, QueuePurpose::CurrentNew],
        [QueuePurpose::CurrentNew, QueuePurpose::HistoricalOld],
    ] {
        let (_directory, root) = originals();
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        let mut capture = ReviewCapture::new_supersession(
            &root,
            Path::new("out.json"),
            &mut ledger,
            &mut control,
        )
        .unwrap();
        for purpose in purposes {
            let path = match purpose {
                QueuePurpose::HistoricalOld => Path::new("old.json"),
                QueuePurpose::CurrentNew => Path::new("new.json"),
            };
            capture.required_supersession_queue(path, purpose, &mut ledger, &mut control).unwrap();
        }
        for (path, purpose) in [
            ("sources.json", AuxiliaryPurpose::NewNativeLocator),
            ("impact.json", AuxiliaryPurpose::ImpactLocator),
            ("links.json", AuxiliaryPurpose::Links),
        ] {
            let actual = capture
                .required_supersession_auxiliary(
                    Path::new(path),
                    purpose,
                    &mut ledger,
                    &mut control,
                )
                .unwrap();
            assert_eq!(capture.supersession_auxiliary_original(purpose).unwrap(), actual);
        }
        let held = capture.finish();
        let state = held.supersession.as_ref().unwrap();
        let old = state.queues[QueuePurpose::HistoricalOld.slot()].unwrap();
        let new = state.queues[QueuePurpose::CurrentNew.slot()].unwrap();
        assert_ne!(old, new);
        assert_eq!(held.bytes(old).unwrap(), b"old.json");
        assert_eq!(held.bytes(new).unwrap(), b"new.json");
        for queue in [old, new] {
            assert_eq!(
                held.bind_review_originals(queue, &[], &mut ledger, &mut control),
                Err(ContractError::Binding)
            );
            assert_eq!(
                held.bind_queue_export_original(queue, &mut ledger, &mut control),
                Err(ContractError::Binding)
            );
        }
        held.verify_inputs(&mut ledger, &mut control).unwrap();
        assert!(!root.join("out.json").exists());
    }
}

/// Ordinary two-Queue and Queue/Recorded requests retain their actual singleton refusal.
#[test]
fn ordinary_queue_and_recorded_singleton_refusals_remain_actual() {
    for (role, pool) in
        [(CaptureRole::ReviewQueue, Pool::Queue), (CaptureRole::ReviewDispositions, Pool::Recorded)]
    {
        let (_directory, root) = originals();
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        let mut capture = ReviewCapture::new(&root, &[], &mut ledger, &mut control).unwrap();
        capture
            .required(
                Path::new("old.json"),
                CaptureRole::ReviewQueue,
                Pool::Queue,
                MAX_QUEUE_BYTES,
                &mut ledger,
                &mut control,
            )
            .unwrap();
        assert!(matches!(
            capture.required(
                Path::new("new.json"),
                role,
                pool,
                MAX_QUEUE_BYTES,
                &mut ledger,
                &mut control
            ),
            Err(ContractError::Capacity)
        ));
        assert_eq!(capture.entries.len(), 1);
    }
}

/// A repeated Queue purpose fails before the nonexistent native read and latches the exact capacity.
#[test]
fn duplicate_queue_purpose_is_charged_before_native_io() {
    let (_directory, root) = originals();
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let mut capture =
        ReviewCapture::new_supersession(&root, Path::new("out.json"), &mut ledger, &mut control)
            .unwrap();
    capture
        .required_supersession_queue(
            Path::new("old.json"),
            QueuePurpose::HistoricalOld,
            &mut ledger,
            &mut control,
        )
        .unwrap();
    assert!(matches!(
        capture.required_supersession_queue(
            Path::new("does-not-exist.json"),
            QueuePurpose::HistoricalOld,
            &mut ledger,
            &mut control
        ),
        Err(ContractError::Capacity)
    ));
    assert_eq!(capture.entries.len(), 1);
    assert_eq!(capture.attempts, 3);
    assert!(matches!(
        capture.required_supersession_queue(
            Path::new("new.json"),
            QueuePurpose::CurrentNew,
            &mut ledger,
            &mut control
        ),
        Err(ContractError::Capacity)
    ));
    assert_eq!(capture.entries.len(), 1);
}

/// Two different purposes cannot reuse the same original spelling or a real hard-link alias.
#[test]
#[cfg(unix)]
fn distinct_queue_purposes_refuse_exact_path_and_physical_alias() {
    for alias in [false, true] {
        let (_directory, root) = originals();
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        let mut capture = ReviewCapture::new_supersession(
            &root,
            Path::new("out.json"),
            &mut ledger,
            &mut control,
        )
        .unwrap();
        capture
            .required_supersession_queue(
                Path::new("old.json"),
                QueuePurpose::HistoricalOld,
                &mut ledger,
                &mut control,
            )
            .unwrap();
        if alias {
            std::fs::hard_link(root.join("old.json"), root.join("alias.json")).unwrap();
        }
        let new_path = if alias { Path::new("alias.json") } else { Path::new("old.json") };
        assert!(matches!(
            capture.required_supersession_queue(
                new_path,
                QueuePurpose::CurrentNew,
                &mut ledger,
                &mut control
            ),
            Err(ContractError::Binding)
        ));
        assert_eq!(capture.entries.len(), 1);
        assert_eq!(capture.supersession.as_ref().unwrap().queues[1], None);
    }
}

/// Three purpose-qualified Auxiliary originals share the one output reservation; forbidden pools do no IO.
#[test]
fn actual_auxiliary_geometry_rejects_generic_queue_response_and_recorded_routes() {
    let (_directory, root) = originals();
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let mut capture =
        ReviewCapture::new_supersession(&root, Path::new("out.json"), &mut ledger, &mut control)
            .unwrap();
    for (name, purpose) in [
        ("sources.json", AuxiliaryPurpose::NewNativeLocator),
        ("impact.json", AuxiliaryPurpose::ImpactLocator),
        ("links.json", AuxiliaryPurpose::Links),
    ] {
        capture
            .required_supersession_auxiliary(Path::new(name), purpose, &mut ledger, &mut control)
            .unwrap();
    }
    for (role, pool) in [
        (CaptureRole::ReviewQueue, Pool::Queue),
        (CaptureRole::ReviewResponse, Pool::Response),
        (CaptureRole::ReviewDispositions, Pool::Recorded),
        (CaptureRole::ReviewPrivateConfig, Pool::Auxiliary),
    ] {
        assert!(matches!(
            capture.required(Path::new("absent.json"), role, pool, 1, &mut ledger, &mut control),
            Err(ContractError::Invalid)
        ));
    }
    assert_eq!(capture.entries.len(), 3);
    assert_eq!(capture.pools[3].registrations, 3);
    assert!(matches!(
        capture.required_supersession_auxiliary(
            Path::new("absent.json"),
            AuxiliaryPurpose::Links,
            &mut ledger,
            &mut control
        ),
        Err(ContractError::Capacity)
    ));
    assert_eq!(capture.entries.len(), 3);
}

/// Genuine native and Impact registrations share one raw allocation and all 100 actual attempts.
#[test]
fn actual_source_overlap_retains_one_allocation_and_charges_all_attempts() {
    let (_directory, root) = originals();
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let mut capture =
        ReviewCapture::new_supersession(&root, Path::new("out.json"), &mut ledger, &mut control)
            .unwrap();
    let native = capture
        .required(
            Path::new("native.json"),
            CaptureRole::Catalog,
            Pool::Source,
            MAX_QUEUE_BYTES,
            &mut ledger,
            &mut control,
        )
        .unwrap();
    for purpose in [ImpactSourcePurpose::OldCatalog, ImpactSourcePurpose::NewCatalog] {
        let actual = capture
            .required_impact_source(Path::new("native.json"), purpose, &mut ledger, &mut control)
            .unwrap();
        assert_eq!(native, actual);
        assert!(capture.lease(native).unwrap().same_original(&capture.lease(actual).unwrap()));
    }
    for _ in 3..MAX_SOURCE_ATTEMPTS {
        assert_eq!(
            capture
                .required(
                    Path::new("native.json"),
                    CaptureRole::Catalog,
                    Pool::Source,
                    MAX_QUEUE_BYTES,
                    &mut ledger,
                    &mut control
                )
                .unwrap(),
            native
        );
    }
    assert_eq!(capture.pools[0].registrations, 100);
    assert_eq!(capture.pools[0].bytes, b"native.json".len());
    assert_eq!(capture.supersession.as_ref().unwrap().impact_uses.len(), 2);
    assert!(matches!(
        capture.required(
            Path::new("absent.json"),
            CaptureRole::Catalog,
            Pool::Source,
            MAX_QUEUE_BYTES,
            &mut ledger,
            &mut control
        ),
        Err(ContractError::Capacity)
    ));
    assert_eq!(capture.entries.len(), 1);
}

/// New Impact roles require their exact explicit mode purpose; current/prior retain compatible leases.
#[test]
fn impact_roles_do_not_expand_ordinary_or_generic_source_admission() {
    let (_directory, root) = originals();
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let mut ordinary = ReviewCapture::new(&root, &[], &mut ledger, &mut control).unwrap();
    assert!(matches!(
        ordinary.required(
            Path::new("impact.json"),
            CaptureRole::ImpactManifest,
            Pool::Source,
            MAX_QUEUE_BYTES,
            &mut ledger,
            &mut control
        ),
        Err(ContractError::Invalid)
    ));
    assert!(ordinary.entries.is_empty());
    let mut capture =
        ReviewCapture::new_supersession(&root, Path::new("out.json"), &mut ledger, &mut control)
            .unwrap();
    assert!(matches!(
        capture.required(
            Path::new("impact.json"),
            CaptureRole::ImpactManifest,
            Pool::Source,
            MAX_QUEUE_BYTES,
            &mut ledger,
            &mut control
        ),
        Err(ContractError::Invalid)
    ));
    let manifest = capture
        .required_impact_source(
            Path::new("impact.json"),
            ImpactSourcePurpose::Manifest,
            &mut ledger,
            &mut control,
        )
        .unwrap();
    assert!(capture.role(manifest).unwrap() == CaptureRole::ImpactManifest);
    let current = capture
        .required_impact_source(
            Path::new("new.json"),
            ImpactSourcePurpose::CurrentReport,
            &mut ledger,
            &mut control,
        )
        .unwrap();
    let prior = capture
        .required_impact_source(
            Path::new("new.json"),
            ImpactSourcePurpose::PriorReport,
            &mut ledger,
            &mut control,
        )
        .unwrap();
    assert_eq!(current, prior);
    assert!(capture.lease(current).unwrap().same_original(&capture.lease(prior).unwrap()));
    assert_eq!(capture.entries.len(), 2);
}

/// Actual 10 MiB Queue files both fit; an individual extra byte refuses before original retention.
#[test]
fn actual_two_queue_raw_limits_are_ten_mib_each_and_twenty_mib_combined() {
    let (_directory, root) = originals();
    for name in ["old.json", "new.json"] {
        std::fs::File::create(root.join(name)).unwrap().set_len(MAX_QUEUE_BYTES as u64).unwrap();
    }
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let mut capture =
        ReviewCapture::new_supersession(&root, Path::new("out.json"), &mut ledger, &mut control)
            .unwrap();
    for (name, purpose) in
        [("old.json", QueuePurpose::HistoricalOld), ("new.json", QueuePurpose::CurrentNew)]
    {
        let index = capture
            .required_supersession_queue(Path::new(name), purpose, &mut ledger, &mut control)
            .unwrap();
        assert_eq!(capture.bytes(index).unwrap().len(), MAX_QUEUE_BYTES);
    }
    assert_eq!(capture.pools[2].bytes, 2 * MAX_QUEUE_BYTES);
    let held = capture.finish();
    held.verify_inputs(&mut ledger, &mut control).unwrap();
    drop(held);
    std::fs::File::create(root.join("old.json"))
        .unwrap()
        .set_len(MAX_QUEUE_BYTES as u64 + 1)
        .unwrap();
    let mut ledger = ContractLedger::default();
    let mut capture = ReviewCapture::new_supersession(
        &root,
        Path::new("other-out.json"),
        &mut ledger,
        &mut control,
    )
    .unwrap();
    assert!(matches!(
        capture.required_supersession_queue(
            Path::new("old.json"),
            QueuePurpose::HistoricalOld,
            &mut ledger,
            &mut control
        ),
        Err(ContractError::Capacity)
    ));
    assert!(capture.entries.is_empty());
}

/// Distinct native/Impact originals share the actual complete 50 MiB Source budget.
#[test]
fn actual_distinct_source_raw_union_has_one_fifty_mib_pool() {
    let (_directory, root) = originals();
    for index in 0..5 {
        std::fs::File::create(root.join(format!("source-{index}.json")))
            .unwrap()
            .set_len(MAX_QUEUE_BYTES as u64)
            .unwrap();
    }
    std::fs::write(root.join("extra.json"), b"x").unwrap();
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let mut capture =
        ReviewCapture::new_supersession(&root, Path::new("out.json"), &mut ledger, &mut control)
            .unwrap();
    for index in 0..5 {
        let name = format!("source-{index}.json");
        if index % 2 == 0 {
            capture
                .required(
                    Path::new(&name),
                    CaptureRole::Catalog,
                    Pool::Source,
                    MAX_QUEUE_BYTES,
                    &mut ledger,
                    &mut control,
                )
                .unwrap();
        } else {
            capture
                .required_impact_source(
                    Path::new(&name),
                    ImpactSourcePurpose::OldCatalog,
                    &mut ledger,
                    &mut control,
                )
                .unwrap();
        }
    }
    assert_eq!(capture.pools[0].bytes, MAX_SOURCE_BYTES);
    assert!(matches!(
        capture.required_impact_source(
            Path::new("extra.json"),
            ImpactSourcePurpose::NewCatalog,
            &mut ledger,
            &mut control
        ),
        Err(ContractError::Capacity)
    ));
    assert_eq!(capture.entries.len(), 5);
}

/// Real caller control records every reached checkpoint and can stop one observed absolute ordinal.
#[derive(Default)]
struct CountingControl {
    /// Actual checkpoints reached; construction work is retained rather than reset.
    seen: usize,
    /// Exact absolute checkpoint selected after independent actual native calibration.
    stop_at: Option<usize>,
    /// Actual first interruption, kept visible to later callers.
    stopped: bool,
}

impl WorkControl for CountingControl {
    /// Stop the actual selected checkpoint without changing a clock or creating capture evidence.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
        self.seen += 1;
        if self.stopped || self.stop_at == Some(self.seen) {
            self.stopped = true;
            return Err(WorkError::Interrupted(Interruption::CancelRequested));
        }
        Ok(())
    }

    /// Expose only an actual reached interruption.
    fn interruption(&self) -> Option<Interruption> {
        self.stopped.then_some(Interruption::CancelRequested)
    }
}

/// Stop only the new wrapper post-phase checkpoint after a real native missing-file Domain failure.
#[test]
fn native_domain_failure_reaches_unconditional_post_phase_checkpoint() {
    let (_directory, root) = originals();
    let missing = Path::new("missing.json");
    let mut native = CaptureSession::new(&root).unwrap();
    native.reserve_output(Path::new("out.json")).unwrap();
    let mut native_callbacks = 0_usize;
    let native_result = native.required_admitted(
        missing,
        CaptureRole::ReviewQueue,
        MAX_QUEUE_BYTES as u64,
        &mut |_charge: CaptureCharge| {
            native_callbacks += 1;
            Ok::<(), ContractError>(())
        },
    );
    assert!(matches!(native_result, Err(crate::linkage::fresh::VerificationError::Domain(_))));
    assert!(native_callbacks > 0);
    let mut ledger = ContractLedger::default();
    let mut control = CountingControl::default();
    let mut capture =
        ReviewCapture::new_supersession(&root, Path::new("out.json"), &mut ledger, &mut control)
            .unwrap();
    // One actual admission checkpoint, the independently counted native callbacks,
    // then exactly the new wrapper's post-native checkpoint. No counter is reset.
    control.stop_at = Some(control.seen + 1 + native_callbacks + 1);
    assert!(matches!(
        capture.required_supersession_queue(
            missing,
            QueuePurpose::HistoricalOld,
            &mut ledger,
            &mut control
        ),
        Err(ContractError::Interrupted(Interruption::CancelRequested))
    ));
    assert!(control.stopped);
    assert!(capture.entries.is_empty());
    assert!(matches!(
        capture.required_supersession_queue(
            Path::new("new.json"),
            QueuePurpose::CurrentNew,
            &mut ledger,
            &mut NoopControl
        ),
        Err(ContractError::Interrupted(Interruption::CancelRequested))
    ));
}
