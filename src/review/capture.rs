//! One actual confined capture owner for portable review preparation.
//! Source, response, queue/recorded and auxiliary originals have distinct before-read
//! sublimits. All holders share one physical alias registry and final proof.
//! These limits qualify logical work and raw originals, not total heap or
//! parser/syscall preemption. Successful capture alone grants no approval.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::evidence_capture::{
    CaptureCharge, CaptureLease, CaptureProof, CaptureRole, CaptureSession,
};
use crate::linkage::fresh::{self, VerificationError};
use crate::workspace::preparation::{
    Interruption, ProgressUpdate, Stage, WorkControl, WorkError, WorkResult,
};

use super::decode::{ContractError, ContractLedger};

/// Complete actual file attempts across the separate bounded pools.
const MAX_ATTEMPTS: usize = 10_105;

/// The role-specific raw pool; no pool is a source-currentness claim.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Pool {
    /// Whole source/native/lifecycle closure, at most100 originals and50 MiB.
    Source,
    /// Immutable reviewer originals, at most10,000 registrations and32 MiB.
    Response,
    /// One exact queue original, at most10 MiB.
    Queue,
    /// One recorded disposition original, at most32 MiB, sharing the Queue slot.
    Recorded,
    /// Up to four private auxiliary originals, at most1 MiB each.
    Auxiliary,
}

impl Pool {
    /// Select the distinct immutable per-pool complete file and original-byte limits.
    fn limits(self) -> (usize, usize) {
        match self {
            Self::Source => (100, 50 * 1024 * 1024),
            Self::Response => (10_000, 32 * 1024 * 1024),
            Self::Queue => (1, 10 * 1024 * 1024),
            Self::Recorded => (1, 32 * 1024 * 1024),
            Self::Auxiliary => (4, 4 * 1024 * 1024),
        }
    }

    /// Select the individual before-read limit without granting a native model.
    fn per_file(self) -> usize {
        match self {
            Self::Source | Self::Queue => 10 * 1024 * 1024,
            Self::Recorded => 32 * 1024 * 1024,
            Self::Response | Self::Auxiliary => 1024 * 1024,
        }
    }

    /// Address the command's sole typed pool counter, never a newly reset owner.
    fn index(self) -> usize {
        match self {
            Self::Source => 0,
            Self::Response => 1,
            Self::Queue | Self::Recorded => 2,
            Self::Auxiliary => 3,
        }
    }

    /// Refuse use of a private/response/queue role as a source byte pool or vice versa.
    fn compatible(self, role: CaptureRole) -> bool {
        match self {
            Self::Response => role == CaptureRole::ReviewResponse,
            Self::Queue => role == CaptureRole::ReviewQueue,
            Self::Recorded => role == CaptureRole::ReviewDispositions,
            Self::Auxiliary => role == CaptureRole::ReviewPrivateConfig,
            Self::Source => matches!(
                role,
                CaptureRole::ApplicabilityManifest
                    | CaptureRole::ApplicabilityReport
                    | CaptureRole::MappingManifest
                    | CaptureRole::MappingCollection
                    | CaptureRole::LifecycleRecord
                    | CaptureRole::Catalog
                    | CaptureRole::Profile
                    | CaptureRole::ComponentDefinition
            ),
        }
    }
}

/// Actual admitted original extent and monotonic registrations for one pool.
#[derive(Default)]
struct PoolCount {
    /// Every registration, including exact duplicates, counted before native capture.
    registrations: usize,
    /// Bytes of distinct actually retained originals in this typed pool.
    bytes: usize,
}

/// One privately held original and its admitted source role, without copied bytes.
struct Entry {
    /// Complete validated private descendant spelling.
    path: PathBuf,
    /// Exact compatible internal native role.
    role: CaptureRole,
    /// Immutable typed raw pool; a repeated path cannot switch pools.
    pool: Pool,
    /// Actual held original owner; shared Rc retains one charged raw allocation.
    lease: CaptureLease,
}

/// One capture preparation with all source/response/auxiliary observations shared.
pub(crate) struct ReviewCapture {
    /// The genuine maintained no-follow capture, not a caller-authored proof.
    session: CaptureSession,
    /// Actual compatible original registrations, private until sealing.
    entries: Vec<Entry>,
    /// Complete typed raw admission counters, never reset by a consumer.
    pools: [PoolCount; 4],
    /// Complete attempted files, including failures and duplicate registrations.
    attempts: usize,
    /// Planned actual outputs share the five auxiliary/queue/output slots.
    output_slots: usize,
    /// Exact admitted operator root extent for private descendant proof geometry.
    root_extent: usize,
    /// Complete successful response registration order, including same-path repeats.
    response_originals: Vec<usize>,
    /// The one actual successful Queue-pool original, never a wire declaration.
    queue_original: Option<usize>,
}

impl ReviewCapture {
    /// Borrow the actual qualified root for private native manifest-base composition.
    pub(crate) fn root(&self) -> &Path {
        self.session.root()
    }

    /// Borrow the immutable actual entry role; no wire key can create an entry.
    pub(crate) fn role(&self, index: usize) -> Result<CaptureRole, ContractError> {
        self.entries.get(index).map(|entry| entry.role).ok_or(ContractError::Invalid)
    }

    /// Clone only the actual captured Rc lease, retaining its single admitted original allocation.
    pub(crate) fn lease(&self, index: usize) -> Result<CaptureLease, ContractError> {
        self.entries.get(index).map(|entry| entry.lease.clone()).ok_or(ContractError::Invalid)
    }

    /// Qualify the actual normalized root and reserve complete output namespaces before reads.
    pub(crate) fn new(
        root: &Path,
        outputs: &[&Path],
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<Self, ContractError> {
        ledger.bound(|ledger| {
            ledger.checkpoint(control)?;
            let root_extent = root.as_os_str().as_encoded_bytes().len();
            ledger.bytes(root_extent)?;
            if root_extent > 4096 || outputs.len() > 2 {
                return Err(ContractError::Invalid);
            }
            let components = root.components().count();
            if components > 64 {
                return Err(ContractError::Invalid);
            }
            ledger.visits(components)?;
            ledger.derived(
                components
                    .checked_mul(
                        root_extent
                            .checked_mul(2)
                            .and_then(|n| n.checked_add(256))
                            .ok_or(ContractError::Capacity)?,
                    )
                    .ok_or(ContractError::Capacity)?,
            )?;
            for path in outputs {
                let length = path.as_os_str().as_encoded_bytes().len();
                ledger.bytes(length)?;
                if length > 4096 || path.components().count() > 64 {
                    return Err(ContractError::Invalid);
                }
                fresh::validate_relative(path).map_err(|_| ContractError::Invalid)?;
                ledger.derived(length.checked_add(256).ok_or(ContractError::Capacity)?)?;
            }
            let mut session = CaptureSession::new_admitted(root, &mut |bytes| {
                ledger.checkpoint(control)?;
                ledger.visits(1)?;
                ledger.bytes(bytes)
            })
            .map_err(|error| match error {
                VerificationError::Domain(_) => ContractError::Invalid,
                VerificationError::Admission(error) => error,
                VerificationError::Capacity => ContractError::Capacity,
            })?;
            if !outputs.is_empty() {
                for left in outputs {
                    for right in outputs {
                        let extent = left
                            .as_os_str()
                            .as_encoded_bytes()
                            .len()
                            .checked_add(right.as_os_str().as_encoded_bytes().len())
                            .ok_or(ContractError::Capacity)?;
                        ledger.bytes(extent)?;
                        for _ in left.components().chain(right.components()) {
                            ledger.checkpoint(control)?;
                            ledger.visits(1)?;
                        }
                    }
                }
                session.reserve_outputs(outputs).map_err(|_| ContractError::Invalid)?;
            }
            ledger.checkpoint(control)?;
            Ok(Self {
                session,
                entries: Vec::new(),
                pools: std::array::from_fn(|_| PoolCount::default()),
                attempts: outputs.len(),
                output_slots: outputs.len(),
                root_extent,
                response_originals: Vec::new(),
                queue_original: None,
            })
        })
    }

    /// Capture one bounded actual regular-file original under its immutable pool and role.
    /// Counters and path/collision work are admitted before native IO and retained growth.
    pub(crate) fn required(
        &mut self,
        path: &Path,
        role: CaptureRole,
        pool: Pool,
        per_file: usize,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<usize, ContractError> {
        ledger.bound(|ledger| {
            let (maximum, components) =
                self.admit_registration(path, role, pool, per_file, ledger, control)?;
            let spelling = path.as_os_str().as_encoded_bytes();
            if let Some(index) = self.entries.iter().position(|entry| entry.path == path) {
                let entry = &self.entries[index];
                if entry.role != role || entry.pool != pool {
                    return Err(ContractError::Invalid);
                }
                if entry.lease.bytes().len() > maximum {
                    return Err(ContractError::Capacity);
                }
                self.session
                    .required_admitted(
                        path,
                        role,
                        u64::try_from(maximum).map_err(|_| ContractError::Capacity)?,
                        &mut |charge| capture_charge(charge, ledger, control),
                    )
                    .map_err(verification_error)?;
                self.register_original(index, pool, ledger, control)?;
                return Ok(index);
            }
            let (_, byte_cap) = pool.limits();
            let count = &mut self.pools[pool.index()];
            let remaining = byte_cap.checked_sub(count.bytes).ok_or(ContractError::Capacity)?;
            let limit = maximum.min(remaining);
            // Reserve all three retained descendant spellings and a conservative
            // held/transient ancestor geometry before the native constructor grows it.
            let ancestor = self
                .root_extent
                .checked_add(spelling.len())
                .and_then(|n| n.checked_add(256))
                .and_then(|n| n.checked_mul(2))
                .and_then(|n| n.checked_mul(components))
                .ok_or(ContractError::Capacity)?;
            ledger.derived(
                spelling
                    .len()
                    .checked_mul(3)
                    .and_then(|n| n.checked_add(256))
                    .and_then(|n| n.checked_add(ancestor))
                    .ok_or(ContractError::Capacity)?,
            )?;
            let lease = self
                .session
                .required_admitted(
                    path,
                    role,
                    u64::try_from(limit).map_err(|_| ContractError::Capacity)?,
                    &mut |charge| capture_charge(charge, ledger, control),
                )
                .map_err(verification_error)?;
            count.bytes = count
                .bytes
                .checked_add(lease.bytes().len())
                .filter(|n| *n <= byte_cap)
                .ok_or(ContractError::Capacity)?;
            ledger.checkpoint(control)?;
            let index = self.entries.len();
            self.entries.push(Entry { path: path.to_path_buf(), role, pool, lease });
            self.register_original(index, pool, ledger, control)?;
            Ok(index)
        })
    }

    /// Admit complete attempts, typed slots and registry work before native IO or retention.
    /// The returned individual limit is not a captured original or a source proof.
    fn admit_registration(
        &mut self,
        path: &Path,
        role: CaptureRole,
        pool: Pool,
        per_file: usize,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(usize, usize), ContractError> {
        ledger.checkpoint(control)?;
        let spelling = path.as_os_str().as_encoded_bytes();
        ledger.bytes(spelling.len())?;
        if spelling.len() > 4096 || !pool.compatible(role) {
            return Err(ContractError::Invalid);
        }
        let components = path.components().count();
        if components > 64 {
            return Err(ContractError::Invalid);
        }
        fresh::validate_relative(path).map_err(|_| ContractError::Invalid)?;
        self.attempts = self
            .attempts
            .checked_add(1)
            .filter(|n| *n <= MAX_ATTEMPTS)
            .ok_or(ContractError::Capacity)?;
        let (mut file_cap, _) = pool.limits();
        if pool == Pool::Auxiliary {
            file_cap = file_cap.checked_sub(self.output_slots).ok_or(ContractError::Capacity)?;
        }
        let count = &mut self.pools[pool.index()];
        count.registrations = count
            .registrations
            .checked_add(1)
            .filter(|n| *n <= file_cap)
            .ok_or(ContractError::Capacity)?;
        let maximum = per_file.min(pool.per_file());
        // The maintained native registry also scans complete physical identities.
        // Explicitly precharge that complete scan, including duplicate/failed paths.
        ledger.visits(
            self.entries
                .len()
                .checked_mul(5)
                .and_then(|n| n.checked_add(components))
                .and_then(|n| n.checked_add(1))
                .ok_or(ContractError::Capacity)?,
        )?;
        ledger.bytes(self.entries.iter().try_fold(0_usize, |n, entry| {
            let pair = entry
                .path
                .as_os_str()
                .as_encoded_bytes()
                .len()
                .checked_add(spelling.len())
                .and_then(|n| n.checked_mul(4))
                .and_then(|n| n.checked_add(32))
                .ok_or(ContractError::Capacity)?;
            n.checked_add(pair).ok_or(ContractError::Capacity)
        })?)?;
        if self.entries.iter().any(|entry| {
            entry.path != path
                && entry.path.as_os_str().as_encoded_bytes().eq_ignore_ascii_case(spelling)
        }) {
            return Err(ContractError::Invalid);
        }
        Ok((maximum, components))
    }

    /// Retain successful cohort multiplicity under the same before-growth ledger.
    fn register_original(
        &mut self,
        index: usize,
        pool: Pool,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        ledger.checkpoint(control)?;
        match pool {
            Pool::Queue => {
                if self.queue_original.replace(index).is_some() {
                    return Err(ContractError::Binding);
                }
            }
            Pool::Response => {
                ledger.visits(1)?;
                ledger.derived(std::mem::size_of::<usize>())?;
                if self.response_originals.len() == self.response_originals.capacity() {
                    ledger.bytes(
                        self.response_originals
                            .len()
                            .checked_mul(std::mem::size_of::<usize>())
                            .ok_or(ContractError::Capacity)?,
                    )?;
                }
                self.response_originals.try_reserve(1).map_err(|_| ledger.capacity())?;
                self.response_originals.push(index);
            }
            Pool::Source | Pool::Recorded | Pool::Auxiliary => {}
        }
        Ok(())
    }

    /// Borrow only an actually captured original; no IO occurs through public source pins.
    pub(crate) fn bytes(&self, index: usize) -> Result<&[u8], ContractError> {
        self.entries.get(index).map(|entry| entry.lease.bytes()).ok_or(ContractError::Invalid)
    }

    /// Borrow an admitted private descendant path for a native resource base join.
    pub(crate) fn path(&self, index: usize) -> Result<&Path, ContractError> {
        self.entries.get(index).map(|entry| entry.path.as_path()).ok_or(ContractError::Invalid)
    }

    /// Transfer all actual originals and all maintained generations into the final fence owner.
    pub(crate) fn finish(self) -> HeldReviewInputs {
        HeldReviewInputs {
            proof: self.session.finish(),
            entries: self.entries,
            response_originals: self.response_originals,
            queue_original: self.queue_original,
        }
    }
}

/// Genuine complete captured input owner; it remains opaque to public wire declarations.
pub(crate) struct HeldReviewInputs {
    /// Maintained root/file/directory/absence and reserved-output generations.
    proof: CaptureProof,
    /// Exact originals stay alive until both the proof and every consumer drop.
    entries: Vec<Entry>,
    /// Complete successful Response-pool registrations, including duplicate paths.
    response_originals: Vec<usize>,
    /// The one actual successful Queue-pool registration.
    queue_original: Option<usize>,
}

impl HeldReviewInputs {
    /// Bind exactly the actual queue and complete ordered successful response cohort.
    /// This checks real capture registrations, not hashes or caller-authored proof flags.
    pub(crate) fn bind_review_originals(
        &self,
        queue_index: usize,
        response_indices: &[usize],
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        ledger.bound(|ledger| {
            ledger.checkpoint(control)?;
            if response_indices.len() > 10_000 {
                return Err(ContractError::Invalid);
            }
            ledger.visits(
                response_indices
                    .len()
                    .checked_add(self.response_originals.len())
                    .and_then(|n| n.checked_add(1))
                    .ok_or(ContractError::Capacity)?,
            )?;
            ledger.bytes(
                response_indices
                    .len()
                    .checked_add(self.response_originals.len())
                    .and_then(|n| n.checked_mul(std::mem::size_of::<usize>()))
                    .ok_or(ContractError::Capacity)?,
            )?;
            if self.queue_original != Some(queue_index)
                || self.entries.get(queue_index).is_none_or(|entry| entry.pool != Pool::Queue)
                || response_indices != self.response_originals
            {
                return Err(ContractError::Binding);
            }
            ledger.checkpoint(control)
        })
    }

    /// Bind the complete one-Queue retained cohort for recorded notification export.
    /// Every extra retained Source, Auxiliary, Response or Recorded original refuses.
    /// This verifies successful registrations only, not failed attempts or directory absence.
    pub(crate) fn bind_queue_export_original(
        &self,
        queue_index: usize,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        ledger.bound(|ledger| {
            ledger.checkpoint(control)?;
            let extent = self
                .entries
                .len()
                .checked_add(self.response_originals.len())
                .and_then(|count| count.checked_add(1))
                .ok_or(ContractError::Capacity)?;
            ledger.visits(extent)?;
            ledger.bytes(
                extent.checked_mul(std::mem::size_of::<usize>()).ok_or(ContractError::Capacity)?,
            )?;
            if self.entries.len() != 1
                || !self.response_originals.is_empty()
                || self.queue_original != Some(queue_index)
                || self.entries.get(queue_index).is_none_or(|entry| {
                    entry.pool != Pool::Queue || entry.role != CaptureRole::ReviewQueue
                })
            {
                return Err(ContractError::Binding);
            }
            ledger.checkpoint(control)
        })
    }

    /// Count the complete distinct actual Source-pool originals without reading or cloning bytes.
    pub(crate) fn source_original_count(&self) -> usize {
        self.entries.iter().filter(|entry| entry.pool == Pool::Source).count()
    }

    /// Require the same actual original owner when sealing a native closure into this complete proof.
    pub(crate) fn same_original(
        &self,
        index: usize,
        original: &CaptureLease,
    ) -> Result<bool, ContractError> {
        self.entries
            .get(index)
            .map(|entry| entry.lease.same_original(original))
            .ok_or(ContractError::Invalid)
    }

    /// Borrow one actual original for raw-preserving response or queue decoding.
    pub(crate) fn bytes(&self, index: usize) -> Result<&[u8], ContractError> {
        self.entries.get(index).map(|entry| entry.lease.bytes()).ok_or(ContractError::Invalid)
    }

    /// Recheck the complete actual closure under the same monotonic ledger and control.
    pub(crate) fn verify_inputs(
        &self,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        ledger.bound(|ledger| {
            self.proof
                .verify_inputs_admitted(&mut |bytes| {
                    ledger.checkpoint(control)?;
                    ledger.visits(1)?;
                    ledger.bytes(bytes)
                })
                .map_err(verification_error)?;
            ledger.checkpoint(control)
        })
    }
}

/// Consume actual native observations and bounded chunk work before reading or retaining bytes.
fn capture_charge(
    charge: CaptureCharge,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    ledger.checkpoint(control)?;
    ledger.visits(1)?;
    match charge {
        CaptureCharge::Checkpoint => Ok(()),
        CaptureCharge::Observed { bytes, limit } => {
            if bytes > limit {
                return Err(ledger.capacity());
            }
            Ok(())
        }
        CaptureCharge::ByteWork(bytes) => ledger.bytes(bytes),
    }
}

/// Keep a caller's precise capacity/control failure; redact only native generation errors.
fn verification_error(error: VerificationError<ContractError>) -> ContractError {
    match error {
        VerificationError::Domain(error) => {
            drop(error);
            ContractError::Binding
        }
        VerificationError::Admission(error) => error,
        VerificationError::Capacity => ContractError::Capacity,
    }
}

/// Fixed accepted-operation deadline shared with caller cancellation and sticky contract work.
pub(crate) struct ReviewControl<'a> {
    /// The unchanged caller control; no per-file cancellation state is created.
    caller: &'a mut dyn WorkControl,
    /// One accepted timestamp, never refreshed during parsing/capture/matching/rendering.
    accepted: Instant,
    /// First actual control stop, preserved at every later checkpoint.
    stopped: Option<Interruption>,
}

impl<'a> ReviewControl<'a> {
    /// Start the one30-second cooperative budget before any root/input native IO.
    pub(crate) fn accept(caller: &'a mut dyn WorkControl) -> Self {
        Self { caller, accepted: Instant::now(), stopped: None }
    }
}

impl WorkControl for ReviewControl<'_> {
    /// Preserve the first caller stop or deadline before the next bounded phase.
    fn checkpoint(&mut self, stage: Stage, progress: ProgressUpdate) -> WorkResult<()> {
        if let Some(reason) = self.stopped {
            return Err(WorkError::Interrupted(reason));
        }
        if let Err(error) = self.caller.checkpoint(stage, progress) {
            if let WorkError::Interrupted(reason) = &error {
                self.stopped = Some(*reason);
            }
            return Err(error);
        }
        if self.accepted.elapsed() >= Duration::from_secs(30) {
            self.stopped = Some(Interruption::DeadlineExceeded);
            return Err(WorkError::Interrupted(Interruption::DeadlineExceeded));
        }
        Ok(())
    }

    /// Expose the same first stop to the final output fence.
    fn interruption(&self) -> Option<Interruption> {
        self.stopped.or_else(|| self.caller.interruption())
    }
}

#[cfg(test)]
/// Real confined-file and control regressions; synthetic fixtures supply no human approval.
mod tests {
    use super::*;
    use crate::workspace::preparation::{NoopControl, test_support::Recorder};

    /// Create actual regular originals under a genuinely qualified temporary root.
    fn fixture() -> (tempfile::TempDir, PathBuf) {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        std::fs::write(root.join("one.json"), b"original").unwrap();
        (directory, root)
    }

    /// Repeated compatible declarations share one charged raw generation and remain bounded registrations.
    #[test]
    fn same_original_shares_one_source_allocation() {
        let (_dir, root) = fixture();
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        let mut capture = ReviewCapture::new(&root, &[], &mut ledger, &mut control).unwrap();
        let a = capture
            .required(
                Path::new("one.json"),
                CaptureRole::Catalog,
                Pool::Source,
                8,
                &mut ledger,
                &mut control,
            )
            .unwrap();
        let b = capture
            .required(
                Path::new("one.json"),
                CaptureRole::Catalog,
                Pool::Source,
                8,
                &mut ledger,
                &mut control,
            )
            .unwrap();
        assert_eq!(a, b);
        assert_eq!(capture.pools[0].bytes, 8);
        assert_eq!(capture.pools[0].registrations, 2);
        capture.finish().verify_inputs(&mut ledger, &mut control).unwrap();
    }

    /// No native read succeeds after the complete source budget has no remaining raw extent.
    #[test]
    fn exhausted_source_extent_refuses_before_retention() {
        let (_dir, root) = fixture();
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        let mut capture = ReviewCapture::new(&root, &[], &mut ledger, &mut control).unwrap();
        capture.pools[0].bytes = 50 * 1024 * 1024;
        assert!(matches!(
            capture.required(
                Path::new("one.json"),
                CaptureRole::Catalog,
                Pool::Source,
                8,
                &mut ledger,
                &mut control
            ),
            Err(ContractError::Capacity)
        ));
        assert!(capture.entries.is_empty());
    }

    /// Pool capacity/control failures remain sticky before later valid actual captures.
    #[test]
    fn file_registration_limit_latches_capacity() {
        let (_dir, root) = fixture();
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        let mut capture = ReviewCapture::new(&root, &[], &mut ledger, &mut control).unwrap();
        capture.pools[0].registrations = 100;
        assert!(matches!(
            capture.required(
                Path::new("one.json"),
                CaptureRole::Catalog,
                Pool::Source,
                8,
                &mut ledger,
                &mut control
            ),
            Err(ContractError::Capacity)
        ));
        capture.pools[0].registrations = 0;
        assert!(matches!(
            capture.required(
                Path::new("one.json"),
                CaptureRole::Catalog,
                Pool::Source,
                8,
                &mut ledger,
                &mut control
            ),
            Err(ContractError::Capacity)
        ));
    }

    /// A response role cannot spend the source pool or convert a native original into a response.
    #[test]
    fn incompatible_pool_role_refuses_without_read() {
        let (_dir, root) = fixture();
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        let mut capture = ReviewCapture::new(&root, &[], &mut ledger, &mut control).unwrap();
        assert!(matches!(
            capture.required(
                Path::new("one.json"),
                CaptureRole::ReviewResponse,
                Pool::Source,
                8,
                &mut ledger,
                &mut control
            ),
            Err(ContractError::Invalid)
        ));
        assert!(capture.entries.is_empty());
        assert_eq!(capture.attempts, 0);
    }

    /// The current fence detects actual tail-byte changes and never returns an empty successful closure.
    #[test]
    fn changed_actual_original_refuses_final_fence() {
        let (_dir, root) = fixture();
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        let mut capture = ReviewCapture::new(&root, &[], &mut ledger, &mut control).unwrap();
        capture
            .required(
                Path::new("one.json"),
                CaptureRole::Catalog,
                Pool::Source,
                8,
                &mut ledger,
                &mut control,
            )
            .unwrap();
        let held = capture.finish();
        std::fs::write(root.join("one.json"), b"originaL").unwrap();
        assert!(matches!(
            held.verify_inputs(&mut ledger, &mut control),
            Err(ContractError::Binding)
        ));
    }

    /// Actual namespace aliases are refused before any original source buffer is retained.
    #[test]
    fn output_alias_precedes_source_capture() {
        let (_dir, root) = fixture();
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        let mut capture =
            ReviewCapture::new(&root, &[Path::new("ONE.json")], &mut ledger, &mut control).unwrap();
        assert!(
            capture
                .required(
                    Path::new("one.json"),
                    CaptureRole::Catalog,
                    Pool::Source,
                    8,
                    &mut ledger,
                    &mut control
                )
                .is_err()
        );
        assert!(capture.entries.is_empty());
    }

    /// Output reservations share the complete five auxiliary/queue/output slots.
    #[test]
    fn reserved_outputs_reduce_auxiliary_input_slots() {
        let (_dir, root) = fixture();
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        let mut capture = ReviewCapture::new(
            &root,
            &[Path::new("queue.json"), Path::new("status.html")],
            &mut ledger,
            &mut control,
        )
        .unwrap();
        capture.pools[3].registrations = 2;
        assert!(matches!(
            capture.required(
                Path::new("one.json"),
                CaptureRole::ReviewPrivateConfig,
                Pool::Auxiliary,
                8,
                &mut ledger,
                &mut control
            ),
            Err(ContractError::Capacity)
        ));
        assert!(capture.entries.is_empty());
    }

    /// The admitted streamed fence preserves caller cancellation through native error translation.
    #[test]
    fn final_fence_preserves_exact_cancellation() {
        let (_dir, root) = fixture();
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        let mut capture = ReviewCapture::new(&root, &[], &mut ledger, &mut control).unwrap();
        capture
            .required(
                Path::new("one.json"),
                CaptureRole::Catalog,
                Pool::Source,
                8,
                &mut ledger,
                &mut control,
            )
            .unwrap();
        let held = capture.finish();
        let mut stop = Recorder::at(Stage::PrepareDomain, 4);
        assert!(matches!(
            held.verify_inputs(&mut ledger, &mut stop),
            Err(ContractError::Interrupted(Interruption::CancelRequested))
        ));
        assert!(matches!(
            held.verify_inputs(&mut ledger, &mut control),
            Err(ContractError::Interrupted(Interruption::CancelRequested))
        ));
    }

    /// Portable folded aliases refuse even when the host stores two different regular files.
    #[test]
    fn distinct_folded_paths_refuse_before_second_capture() {
        let (_dir, root) = fixture();
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        let mut capture = ReviewCapture::new(&root, &[], &mut ledger, &mut control).unwrap();
        capture
            .required(
                Path::new("one.json"),
                CaptureRole::Catalog,
                Pool::Source,
                8,
                &mut ledger,
                &mut control,
            )
            .unwrap();
        assert!(matches!(
            capture.required(
                Path::new("ONE.json"),
                CaptureRole::Catalog,
                Pool::Source,
                8,
                &mut ledger,
                &mut control
            ),
            Err(ContractError::Invalid)
        ));
        assert_eq!(capture.entries.len(), 1);
    }

    /// A small individual raw limit reaches the actual held reader and latches Capacity.
    #[test]
    fn individual_raw_capacity_is_typed_and_sticky() {
        let (_dir, root) = fixture();
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        let mut capture = ReviewCapture::new(&root, &[], &mut ledger, &mut control).unwrap();
        assert!(matches!(
            capture.required(
                Path::new("one.json"),
                CaptureRole::Catalog,
                Pool::Source,
                7,
                &mut ledger,
                &mut control
            ),
            Err(ContractError::Capacity)
        ));
        assert!(capture.entries.is_empty());
        assert!(matches!(
            capture.required(
                Path::new("one.json"),
                CaptureRole::Catalog,
                Pool::Source,
                8,
                &mut ledger,
                &mut control
            ),
            Err(ContractError::Capacity)
        ));
    }

    /// Arm the actual caller stop after size observation and before the first content scratch read.
    /// Geometry callbacks cannot move this control into an earlier native phase.
    #[test]
    fn actual_read_chunk_cancellation_preserves_first_stop() {
        let (_dir, root) = fixture();
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        let mut capture = CaptureSession::new(&root).unwrap();
        let mut stop = Recorder::at(Stage::PrepareDomain, 1);
        let mut observed = false;
        let result = ledger.bound(|ledger| {
            capture
                .required_admitted(Path::new("one.json"), CaptureRole::Catalog, 8, &mut |charge| {
                    if matches!(&charge, CaptureCharge::Observed { .. }) {
                        observed = true;
                    }
                    if observed && matches!(&charge, CaptureCharge::ByteWork(_)) {
                        capture_charge(charge, ledger, &mut stop)
                    } else {
                        capture_charge(charge, ledger, &mut control)
                    }
                })
                .map_err(verification_error)
        });
        assert!(observed);
        assert!(matches!(result, Err(ContractError::Interrupted(Interruption::CancelRequested))));
        assert!(capture.captured(Path::new("one.json"), CaptureRole::Catalog).is_err());
        assert!(matches!(
            ledger.bound(|ledger| {
                capture
                    .required_admitted(
                        Path::new("one.json"),
                        CaptureRole::Catalog,
                        8,
                        &mut |charge| capture_charge(charge, ledger, &mut control),
                    )
                    .map_err(verification_error)
            }),
            Err(ContractError::Interrupted(Interruption::CancelRequested))
        ));
    }

    /// One accepted deadline stays latched; later timestamps/caller success do not reopen work.
    #[test]
    fn accepted_deadline_does_not_reset() {
        let mut noop = NoopControl;
        let mut control = ReviewControl::accept(&mut noop);
        control.accepted = Instant::now()
            .checked_sub(Duration::from_secs(31))
            .expect("test monotonic instant must permit a 31-second elapsed deadline");
        assert!(matches!(
            control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged),
            Err(WorkError::Interrupted(Interruption::DeadlineExceeded))
        ));
        control.accepted = Instant::now();
        assert_eq!(control.interruption(), Some(Interruption::DeadlineExceeded));
        assert!(control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged).is_err());
    }
}

#[cfg(all(test, any(unix, windows)))]
/// Same-ledger wrapper control for the complete after-native final boundary.
mod admitted_final_tests {
    use super::*;
    use crate::workspace::preparation::{NoopControl, test_support::Recorder};

    /// Calibrate actual complete fence callbacks, then stop its final wrapper boundary.
    #[test]
    fn final_wrapper_stop_is_sticky_after_complete_native_fence() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        std::fs::write(root.join("actual.bin"), b"actual").unwrap();
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        let mut capture = ReviewCapture::new(&root, &[], &mut ledger, &mut control).unwrap();
        capture
            .required(
                Path::new("actual.bin"),
                CaptureRole::Catalog,
                Pool::Source,
                6,
                &mut ledger,
                &mut control,
            )
            .unwrap();
        let held = capture.finish();
        let mut recorded = Recorder::default();
        held.verify_inputs(&mut ledger, &mut recorded).unwrap();
        let count =
            recorded.events.iter().filter(|(stage, _)| *stage == Stage::PrepareDomain).count();
        assert!(count > 0);
        let mut stop = Recorder::at(Stage::PrepareDomain, count);
        assert!(matches!(
            held.verify_inputs(&mut ledger, &mut stop),
            Err(ContractError::Interrupted(Interruption::CancelRequested))
        ));
        assert!(matches!(
            held.verify_inputs(&mut ledger, &mut control),
            Err(ContractError::Interrupted(Interruption::CancelRequested))
        ));
    }
}

#[cfg(test)]
/// Actual captured cohort controls; no native lifecycle or review approval is fabricated.
mod original_cohort_tests {
    use super::*;
    use crate::workspace::preparation::{NoopControl, test_support::Recorder};

    /// Capture real distinct paths and a repeated response under one actual owner.
    fn cohort() -> (tempfile::TempDir, HeldReviewInputs, usize, Vec<usize>) {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        for (path, raw) in [
            ("queue.json", b"queue".as_slice()),
            ("response-a.json", b"response".as_slice()),
            ("response-b.json", b"response".as_slice()),
        ] {
            std::fs::write(root.join(path), raw).unwrap();
        }
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        let mut capture = ReviewCapture::new(&root, &[], &mut ledger, &mut control).unwrap();
        let queue = capture
            .required(
                Path::new("queue.json"),
                CaptureRole::ReviewQueue,
                Pool::Queue,
                100,
                &mut ledger,
                &mut control,
            )
            .unwrap();
        let mut responses = Vec::new();
        for name in ["response-a.json", "response-a.json", "response-b.json"] {
            responses.push(
                capture
                    .required(
                        Path::new(name),
                        CaptureRole::ReviewResponse,
                        Pool::Response,
                        100,
                        &mut ledger,
                        &mut control,
                    )
                    .unwrap(),
            );
        }
        (directory, capture.finish(), queue, responses)
    }

    /// Equal bytes across two real paths are distinct originals; repeated path registration remains counted.
    #[test]
    fn exact_actual_cohort_retains_same_path_duplicates_and_distinct_owners() {
        let (_directory, held, queue, responses) = cohort();
        assert_eq!(responses[0], responses[1]);
        assert_ne!(responses[0], responses[2]);
        assert_eq!(held.bytes(responses[0]).unwrap(), held.bytes(responses[2]).unwrap());
        assert!(!std::ptr::eq(
            held.bytes(responses[0]).unwrap(),
            held.bytes(responses[2]).unwrap()
        ));
        let mut ledger = ContractLedger::default();
        held.bind_review_originals(queue, &responses, &mut ledger, &mut NoopControl).unwrap();
        held.verify_inputs(&mut ledger, &mut NoopControl).unwrap();
    }

    /// Omitted duplicates, reordered declarations and a response used as queue cannot bind the genuine cohort.
    #[test]
    fn complete_actual_cohort_refuses_omissions_reordering_and_wrong_queue() {
        let (_directory, held, queue, responses) = cohort();
        for altered in
            [vec![responses[0], responses[2]], vec![responses[0], responses[2], responses[1]]]
        {
            assert_eq!(
                held.bind_review_originals(
                    queue,
                    &altered,
                    &mut ContractLedger::default(),
                    &mut NoopControl
                ),
                Err(ContractError::Binding)
            );
        }
        assert_eq!(
            held.bind_review_originals(
                responses[0],
                &responses,
                &mut ContractLedger::default(),
                &mut NoopControl
            ),
            Err(ContractError::Binding)
        );
    }

    /// The complete cohort comparison consumes actual shared work and preserves the first precise stop.
    #[test]
    fn actual_cohort_binding_capacity_and_cancellation_remain_sticky() {
        let (_directory, held, queue, responses) = cohort();
        let mut ledger = ContractLedger::default();
        ledger.bytes(268_435_455).unwrap();
        assert_eq!(
            held.bind_review_originals(queue, &responses, &mut ledger, &mut NoopControl),
            Err(ContractError::Capacity)
        );
        assert_eq!(
            held.bind_review_originals(queue, &responses, &mut ledger, &mut NoopControl),
            Err(ContractError::Capacity)
        );
        let mut ledger = ContractLedger::default();
        let mut stop = Recorder::at(Stage::PrepareDomain, 1);
        let expected = Err(ContractError::Interrupted(Interruption::CancelRequested));
        assert_eq!(held.bind_review_originals(queue, &responses, &mut ledger, &mut stop), expected);
        assert_eq!(
            held.bind_review_originals(queue, &responses, &mut ledger, &mut NoopControl),
            expected
        );
    }
}

#[cfg(test)]
/// Actual recorded-only capture controls; this role never grants queue or native authority.
mod recorded_pool_tests {
    use super::*;
    use crate::workspace::preparation::NoopControl;

    /// Create genuine confined originals, including a sparse raw extent when required.
    fn fixture(length: u64) -> (tempfile::TempDir, PathBuf) {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        std::fs::File::create(root.join("recorded.json")).unwrap().set_len(length).unwrap();
        std::fs::write(root.join("queue.json"), b"queue").unwrap();
        (directory, root)
    }

    /// An actual original above the Queue limit retains its own role and complete final proof.
    #[test]
    fn recorded_extent_above_queue_limit_is_held_without_queue_authority() {
        use std::io::{Seek as _, Write as _};
        let extent = 10 * 1024 * 1024 + 1;
        let (_directory, root) = fixture(extent as u64);
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        let mut capture = ReviewCapture::new(&root, &[], &mut ledger, &mut control).unwrap();
        let index = capture
            .required(
                Path::new("recorded.json"),
                CaptureRole::ReviewDispositions,
                Pool::Recorded,
                32 * 1024 * 1024,
                &mut ledger,
                &mut control,
            )
            .unwrap();
        assert_eq!(capture.bytes(index).unwrap().len(), extent);
        assert_eq!(capture.pools[2].bytes, extent);
        assert_eq!(capture.pools[2].registrations, 1);
        let held = capture.finish();
        held.verify_inputs(&mut ledger, &mut control).unwrap();
        assert_eq!(held.source_original_count(), 0);
        assert_eq!(
            held.bind_review_originals(index, &[], &mut ledger, &mut control),
            Err(ContractError::Binding)
        );
        // A separate ledger here tests native mutation refusal after the deliberate binding failure.
        let mut changed =
            std::fs::OpenOptions::new().write(true).open(root.join("recorded.json")).unwrap();
        changed.seek(std::io::SeekFrom::End(-1)).unwrap();
        changed.write_all(b"x").unwrap();
        changed.sync_all().unwrap();
        assert_eq!(
            held.verify_inputs(&mut ContractLedger::default(), &mut control),
            Err(ContractError::Binding)
        );
    }

    /// The Queue and Recorded alternatives share one successful-or-attempted input slot.
    #[test]
    fn queue_and_recorded_cannot_expand_the_shared_single_slot() {
        let (_directory, root) = fixture(8);
        for recorded_first in [false, true] {
            let mut ledger = ContractLedger::default();
            let mut control = NoopControl;
            let mut capture = ReviewCapture::new(&root, &[], &mut ledger, &mut control).unwrap();
            let queue = (Path::new("queue.json"), CaptureRole::ReviewQueue, Pool::Queue);
            let recorded =
                (Path::new("recorded.json"), CaptureRole::ReviewDispositions, Pool::Recorded);
            let (first, second) =
                if recorded_first { (recorded, queue) } else { (queue, recorded) };
            capture.required(first.0, first.1, first.2, 100, &mut ledger, &mut control).unwrap();
            assert_eq!(
                capture.required(second.0, second.1, second.2, 100, &mut ledger, &mut control),
                Err(ContractError::Capacity)
            );
            assert_eq!(capture.entries.len(), 1);
            assert_eq!(capture.pools[2].registrations, 1);
            assert_eq!(
                capture.required(first.0, first.1, first.2, 100, &mut ledger, &mut control),
                Err(ContractError::Capacity)
            );
        }
    }

    /// Recorded bytes cannot enter Queue/Source/Auxiliary or borrow another role's limit.
    #[test]
    fn recorded_pool_role_mismatches_refuse_before_native_read() {
        let (_directory, root) = fixture(8);
        for (pool, role) in [
            (Pool::Recorded, CaptureRole::ReviewQueue),
            (Pool::Queue, CaptureRole::ReviewDispositions),
            (Pool::Source, CaptureRole::ReviewDispositions),
            (Pool::Auxiliary, CaptureRole::ReviewDispositions),
        ] {
            let mut ledger = ContractLedger::default();
            let mut control = NoopControl;
            let mut capture = ReviewCapture::new(&root, &[], &mut ledger, &mut control).unwrap();
            assert_eq!(
                capture.required(
                    Path::new("recorded.json"),
                    role,
                    pool,
                    100,
                    &mut ledger,
                    &mut control
                ),
                Err(ContractError::Invalid)
            );
            assert!(capture.entries.is_empty());
            assert_eq!(capture.attempts, 0);
        }
    }

    /// The actual held-handle extent refuses above32 MiB before a raw allocation is retained.
    #[test]
    fn recorded_raw_cap_is_before_read_and_sticky() {
        let (_directory, root) = fixture(32 * 1024 * 1024 + 1);
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        let mut capture = ReviewCapture::new(&root, &[], &mut ledger, &mut control).unwrap();
        assert_eq!(
            capture.required(
                Path::new("recorded.json"),
                CaptureRole::ReviewDispositions,
                Pool::Recorded,
                32 * 1024 * 1024 + 1,
                &mut ledger,
                &mut control
            ),
            Err(ContractError::Capacity)
        );
        assert!(capture.entries.is_empty());
        assert_eq!(capture.pools[2].bytes, 0);
        assert_eq!(
            capture.required(
                Path::new("queue.json"),
                CaptureRole::ReviewQueue,
                Pool::Queue,
                100,
                &mut ledger,
                &mut control
            ),
            Err(ContractError::Capacity)
        );
    }
}
