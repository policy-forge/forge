//! S4 capture geometry and private complete-union preparation.
//!
//! The complete captured Impact reader supplies the private pending comparison.
//! Its native facts and original leases remain held through complete union sealing.
//! Plain borrowed native facts and stored-report equality cannot issue that port.
//! Existing ordinary seals and recorded/Queue binders remain unchanged.

use std::path::Path;
use std::rc::Rc;

use crate::evidence_capture::{CaptureLease, CaptureRole};
use crate::hashing::sha256_hex;
use crate::workspace::preparation::WorkControl;

use super::{Entry, HeldReviewInputs, Pool, ReviewCapture, capture_charge, verification_error};
use crate::review::applicability_capture::{
    ApprovedApplicabilityClosure, PendingApplicabilityClosure,
};
use crate::review::decode::{ContractError, ContractLedger};
use crate::review::impact_capture::PendingImpactCohort;
use crate::review::mapping_capture::{ApprovedMappingClosure, PendingMappingClosure};
use crate::review::wire::{SourceModel, SourcePin};

/// One output reservation plus at most 105 attempted input registrations.
const MAX_SUPERSESSION_SLOTS: usize = 106;
/// Existing shared complete Source-original bound, with no second Source pool.
const MAX_SOURCE_ATTEMPTS: usize = 100;
/// Existing shared distinct Source-original raw extent.
const MAX_SOURCE_BYTES: usize = 50 * 1024 * 1024;
/// Each of two exact Queue originals retains the existing per-file bound.
const MAX_QUEUE_BYTES: usize = 10 * 1024 * 1024;
/// Each of three exact Auxiliary originals retains the existing per-file bound.
const MAX_AUXILIARY_BYTES: usize = 1024 * 1024;

/// Actual capture purpose; neither variant asserts native currentness.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum QueuePurpose {
    /// Historical old queue original, decoded later as a recorded declaration.
    HistoricalOld,
    /// New queue original, requiring its separate genuine native binding later.
    CurrentNew,
}

impl QueuePurpose {
    /// Address one immutable purpose slot without assigning it from insertion order.
    fn slot(self) -> usize {
        match self {
            Self::HistoricalOld => 0,
            Self::CurrentNew => 1,
        }
    }
}

/// Exact private input purpose; all three use the existing native config role.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuxiliaryPurpose {
    /// Existing complete new-domain locator consumed by the actual native factory.
    NewNativeLocator,
    /// Explicit complete native Impact route declaration.
    ImpactLocator,
    /// Explicit lineage request, supplying no source hashes or proof.
    Links,
}

impl AuxiliaryPurpose {
    /// Address the three input slots left by the one frozen output namespace.
    fn slot(self) -> usize {
        match self {
            Self::NewNativeLocator => 0,
            Self::ImpactLocator => 1,
            Self::Links => 2,
        }
    }
}

/// Closed actual Impact-read purposes, separate from public `SourceModel` values.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ImpactSourcePurpose {
    /// Maintained native Impact manifest.
    Manifest,
    /// Supplied complete current stored report, required as the S4 private oracle input.
    CurrentReport,
    /// Old Catalog framework original.
    OldCatalog,
    /// Old Profile framework original.
    OldProfile,
    /// Explicit old Profile resolved Catalog companion.
    OldResolvedCatalog,
    /// New Catalog framework original.
    NewCatalog,
    /// New Profile framework original.
    NewProfile,
    /// Explicit new Profile resolved Catalog companion.
    NewResolvedCatalog,
    /// Each declared Mapping Collection in native manifest order.
    MappingCollection,
    /// Configured applicability manifest, using its own relative base.
    ApplicabilityManifest,
    /// Configured applicability Catalog framework original.
    ApplicabilityCatalog,
    /// Configured applicability Profile framework original.
    ApplicabilityProfile,
    /// Configured applicability explicit resolved Catalog companion.
    ApplicabilityResolvedCatalog,
    /// Each configured applicability Mapping Collection in its own manifest order.
    ApplicabilityMappingCollection,
    /// Explicitly declared control successor relation.
    SuccessorMap,
    /// Paired prior report; semantic history admission remains the native engine's job.
    PriorReport,
    /// Paired native Impact dispositions, never review-response dispositions.
    Dispositions,
}

impl ImpactSourcePurpose {
    /// Match exact native role compatibility; hrefs and embedded policy IDs grant no route.
    fn role(self) -> CaptureRole {
        match self {
            Self::Manifest => CaptureRole::ImpactManifest,
            Self::CurrentReport | Self::PriorReport => CaptureRole::ImpactReport,
            Self::OldCatalog
            | Self::OldResolvedCatalog
            | Self::NewCatalog
            | Self::NewResolvedCatalog
            | Self::ApplicabilityCatalog
            | Self::ApplicabilityResolvedCatalog => CaptureRole::Catalog,
            Self::OldProfile | Self::NewProfile | Self::ApplicabilityProfile => {
                CaptureRole::Profile
            }
            Self::MappingCollection | Self::ApplicabilityMappingCollection => {
                CaptureRole::MappingCollection
            }
            Self::ApplicabilityManifest => CaptureRole::ApplicabilityManifest,
            Self::SuccessorMap => CaptureRole::ControlSuccessorMap,
            Self::Dispositions => CaptureRole::ImpactDispositions,
        }
    }
}

/// One successful actual Source-purpose registration; it is not a complete read set.
struct ImpactUse {
    /// Actual successful entry index on this same capture owner.
    index: usize,
    /// Exact purpose supplied by the declared native reader.
    purpose: ImpactSourcePurpose,
}

/// Mode-private geometry and successful registrations, moved into the actual held owner.
pub(super) struct SupersessionRegistrations {
    /// Attempted registrations including the original output reservation and refusals.
    attempts: usize,
    /// Queue-purpose attempts, charged before any native IO.
    queue_attempts: [usize; 2],
    /// Auxiliary-purpose attempts, charged before any native IO.
    auxiliary_attempts: [usize; 3],
    /// Exact successful Queue-purpose registrations; no ordinary Queue slot is assigned.
    queues: [Option<usize>; 2],
    /// Exact successful Auxiliary-purpose registrations.
    auxiliaries: [Option<usize>; 3],
    /// Complete successful ordered Impact-purpose registrations, including repeats.
    impact_uses: Vec<ImpactUse>,
}

/// Only mode-private callers select purpose-qualified registration paths.
#[derive(Clone, Copy)]
enum Request {
    /// Existing genuine native factory Source registration under its unchanged role table.
    NativeSource,
    /// Exact old/new Queue purpose registration.
    Queue(QueuePurpose),
    /// Exact one of three Auxiliary purposes.
    Auxiliary(AuxiliaryPurpose),
    /// Exact declared Impact-read Source purpose.
    Impact(ImpactSourcePurpose),
}

impl Request {
    /// Return the one existing raw pool; Impact and native never get separate Source pools.
    fn pool(self) -> Pool {
        match self {
            Self::NativeSource | Self::Impact(_) => Pool::Source,
            Self::Queue(_) => Pool::Queue,
            Self::Auxiliary(_) => Pool::Auxiliary,
        }
    }

    /// Select the mode-specific registration and distinct raw extent bounds.
    fn limits(self) -> (usize, usize, usize) {
        match self {
            Self::NativeSource | Self::Impact(_) => {
                (MAX_SOURCE_ATTEMPTS, MAX_SOURCE_BYTES, MAX_QUEUE_BYTES)
            }
            Self::Queue(_) => (2, 2 * MAX_QUEUE_BYTES, MAX_QUEUE_BYTES),
            Self::Auxiliary(_) => (3, 3 * MAX_AUXILIARY_BYTES, MAX_AUXILIARY_BYTES),
        }
    }

    /// Require the exact internal role before capture; ordinary role admission is unchanged.
    fn accepts(self, role: CaptureRole) -> bool {
        match self {
            Self::NativeSource => Pool::Source.compatible(role),
            Self::Queue(_) => role == CaptureRole::ReviewQueue,
            Self::Auxiliary(_) => role == CaptureRole::ReviewPrivateConfig,
            Self::Impact(purpose) => role == purpose.role(),
        }
    }
}

impl ReviewCapture {
    /// Reserve one actual output before reads on the existing native session and ledger.
    pub(crate) fn new_supersession(
        root: &Path,
        output: &Path,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<Self, ContractError> {
        ledger.bound(|ledger| {
            let capture = Self::new(root, &[output], ledger, control);
            ledger.checkpoint(control)?;
            let mut capture = capture?;
            ledger.derived(std::mem::size_of::<SupersessionRegistrations>())?;
            capture.supersession = Some(SupersessionRegistrations {
                attempts: 1,
                queue_attempts: [0; 2],
                auxiliary_attempts: [0; 3],
                queues: [None; 2],
                auxiliaries: [None; 3],
                impact_uses: Vec::new(),
            });
            ledger.checkpoint(control)?;
            Ok(capture)
        })
    }

    /// Capture an actual distinct old/new Queue original under its persisted purpose.
    pub(crate) fn required_supersession_queue(
        &mut self,
        path: &Path,
        purpose: QueuePurpose,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<usize, ContractError> {
        self.required_supersession(
            path,
            CaptureRole::ReviewQueue,
            Request::Queue(purpose),
            MAX_QUEUE_BYTES,
            ledger,
            control,
        )
    }

    /// Capture one exact Auxiliary input without consuming a second namespace/session.
    pub(crate) fn required_supersession_auxiliary(
        &mut self,
        path: &Path,
        purpose: AuxiliaryPurpose,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<usize, ContractError> {
        self.required_supersession(
            path,
            CaptureRole::ReviewPrivateConfig,
            Request::Auxiliary(purpose),
            MAX_AUXILIARY_BYTES,
            ledger,
            control,
        )
    }

    /// Capture an explicit Impact Source purpose; completeness still needs the genuine reader.
    pub(crate) fn required_impact_source(
        &mut self,
        path: &Path,
        purpose: ImpactSourcePurpose,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<usize, ContractError> {
        self.required_supersession(
            path,
            purpose.role(),
            Request::Impact(purpose),
            MAX_QUEUE_BYTES,
            ledger,
            control,
        )
    }

    /// Borrow only an actual successful Auxiliary-purpose registration, never a caller subset.
    pub(crate) fn supersession_auxiliary_original(
        &self,
        purpose: AuxiliaryPurpose,
    ) -> Result<usize, ContractError> {
        self.supersession
            .as_ref()
            .and_then(|state| state.auxiliaries[purpose.slot()])
            .ok_or(ContractError::Binding)
    }

    /// Route genuine existing Source factories through the same mode-specific admission.
    pub(super) fn required_supersession_native(
        &mut self,
        path: &Path,
        role: CaptureRole,
        pool: Pool,
        per_file: usize,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<usize, ContractError> {
        if pool != Pool::Source {
            return ledger.bound(|ledger| {
                self.supersession_attempt(ledger, control)?;
                Err(ContractError::Invalid)
            });
        }
        self.required_supersession(path, role, Request::NativeSource, per_file, ledger, control)
    }

    /// Count every mode input attempt before role/path/native validation or IO.
    fn supersession_attempt(
        &mut self,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        ledger.checkpoint(control)?;
        let state = self.supersession.as_mut().ok_or(ContractError::Invalid)?;
        self.attempts = self
            .attempts
            .checked_add(1)
            .filter(|count| *count <= MAX_SUPERSESSION_SLOTS)
            .ok_or(ContractError::Capacity)?;
        state.attempts = self.attempts;
        Ok(())
    }

    /// Admit purpose, shared pool, complete path comparisons and geometry before native work.
    fn supersession_admission(
        &mut self,
        path: &Path,
        role: CaptureRole,
        request: Request,
        per_file: usize,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(usize, usize), ContractError> {
        self.supersession_attempt(ledger, control)?;
        let (file_cap, _, per_file_cap) = request.limits();
        let count = &mut self.pools[request.pool().index()];
        count.registrations = count
            .registrations
            .checked_add(1)
            .filter(|count| *count <= file_cap)
            .ok_or(ContractError::Capacity)?;
        let state = self.supersession.as_mut().ok_or(ContractError::Invalid)?;
        let purpose_attempt = match request {
            Request::Queue(purpose) => Some(&mut state.queue_attempts[purpose.slot()]),
            Request::Auxiliary(purpose) => Some(&mut state.auxiliary_attempts[purpose.slot()]),
            Request::NativeSource | Request::Impact(_) => None,
        };
        if let Some(attempt) = purpose_attempt {
            *attempt = attempt
                .checked_add(1)
                .filter(|count| *count <= 1)
                .ok_or(ContractError::Capacity)?;
        }
        let spelling = path.as_os_str().as_encoded_bytes();
        ledger.bytes(spelling.len())?;
        let components = path.components().count();
        if !request.accepts(role) || spelling.len() > 4096 || components > 64 {
            return Err(ContractError::Invalid);
        }
        crate::linkage::fresh::validate_relative(path).map_err(|_| ContractError::Invalid)?;
        ledger.visits(
            self.entries
                .len()
                .checked_mul(5)
                .and_then(|count| count.checked_add(components))
                .and_then(|count| count.checked_add(1))
                .ok_or(ContractError::Capacity)?,
        )?;
        ledger.bytes(self.entries.iter().try_fold(0_usize, |total, entry| {
            let pair = entry
                .path
                .as_os_str()
                .as_encoded_bytes()
                .len()
                .checked_add(spelling.len())
                .and_then(|size| size.checked_mul(4))
                .and_then(|size| size.checked_add(32))
                .ok_or(ContractError::Capacity)?;
            total.checked_add(pair).ok_or(ContractError::Capacity)
        })?)?;
        self.reject_folded_aliases(path, components, ledger, control)?;
        Ok((per_file.min(per_file_cap), components))
    }

    /// Retain the actual original with native before-growth admission and unconditional post fence.
    fn supersession_lease(
        &mut self,
        path: &Path,
        role: CaptureRole,
        maximum: usize,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<CaptureLease, ContractError> {
        let captured = ledger.bound(|ledger| {
            self.session
                .required_admitted(
                    path,
                    role,
                    u64::try_from(maximum).map_err(|_| ContractError::Capacity)?,
                    &mut |charge| capture_charge(charge, ledger, control),
                )
                .map_err(verification_error)
        });
        ledger.checkpoint(control)?;
        captured
    }

    /// Persist genuine successful registrations; repeated Source-purpose work remains visible.
    fn supersession_register(
        &mut self,
        index: usize,
        request: Request,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        ledger.checkpoint(control)?;
        let state = self.supersession.as_mut().ok_or(ContractError::Invalid)?;
        match request {
            Request::Queue(purpose) => {
                if state.queues[purpose.slot()].replace(index).is_some() {
                    return Err(ContractError::Binding);
                }
            }
            Request::Auxiliary(purpose) => {
                if state.auxiliaries[purpose.slot()].replace(index).is_some() {
                    return Err(ContractError::Binding);
                }
            }
            Request::Impact(purpose) => {
                ledger.visits(1)?;
                ledger.derived(std::mem::size_of::<ImpactUse>())?;
                if state.impact_uses.len() == state.impact_uses.capacity() {
                    ledger.bytes(
                        state
                            .impact_uses
                            .len()
                            .checked_mul(std::mem::size_of::<ImpactUse>())
                            .ok_or(ContractError::Capacity)?,
                    )?;
                }
                state.impact_uses.try_reserve(1).map_err(|_| ledger.capacity())?;
                state.impact_uses.push(ImpactUse { index, purpose });
            }
            Request::NativeSource => {}
        }
        ledger.checkpoint(control)
    }

    /// Capture only on this actual native session; no detached bytes, path staging or proof flags.
    fn required_supersession(
        &mut self,
        path: &Path,
        role: CaptureRole,
        request: Request,
        per_file: usize,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<usize, ContractError> {
        ledger.bound(|ledger| {
            let (maximum, components) =
                self.supersession_admission(path, role, request, per_file, ledger, control)?;
            let pool = request.pool();
            if let Some(index) = self.entries.iter().position(|entry| entry.path == path) {
                let entry = &self.entries[index];
                if pool != Pool::Source || entry.pool != pool || entry.role != role {
                    return Err(ContractError::Binding);
                }
                if entry.lease.bytes().len() > maximum {
                    return Err(ledger.capacity());
                }
                self.supersession_lease(path, role, maximum, ledger, control)?;
                self.supersession_register(index, request, ledger, control)?;
                return Ok(index);
            }
            let (_, byte_cap, _) = request.limits();
            let remaining = byte_cap
                .checked_sub(self.pools[pool.index()].bytes)
                .ok_or(ContractError::Capacity)?;
            let spelling = path.as_os_str().as_encoded_bytes().len();
            let ancestor = self
                .root_extent
                .checked_add(spelling)
                .and_then(|size| size.checked_add(256))
                .and_then(|size| size.checked_mul(2))
                .and_then(|size| size.checked_mul(components))
                .ok_or(ContractError::Capacity)?;
            ledger.derived(
                spelling
                    .checked_mul(3)
                    .and_then(|size| size.checked_add(256))
                    .and_then(|size| size.checked_add(ancestor))
                    .ok_or(ContractError::Capacity)?,
            )?;
            ledger.derived(std::mem::size_of::<Entry>())?;
            if self.entries.len() == self.entries.capacity() {
                ledger.bytes(
                    self.entries
                        .len()
                        .checked_mul(std::mem::size_of::<Entry>())
                        .ok_or(ContractError::Capacity)?,
                )?;
            }
            self.entries.try_reserve(1).map_err(|_| ledger.capacity())?;
            let lease =
                self.supersession_lease(path, role, maximum.min(remaining), ledger, control)?;
            self.pools[pool.index()].bytes = self.pools[pool.index()]
                .bytes
                .checked_add(lease.bytes().len())
                .filter(|size| *size <= byte_cap)
                .ok_or(ContractError::Capacity)?;
            ledger.checkpoint(control)?;
            let index = self.entries.len();
            self.entries.push(Entry { path: path.to_path_buf(), components, role, pool, lease });
            self.supersession_register(index, request, ledger, control)?;
            Ok(index)
        })
    }
}

/// A consumed genuine factory pending result; no caller indices or completeness flag.
pub(crate) struct PendingNewNativeCohort {
    /// Full private maintained native facts and original leases remain owned together.
    native: NativePending,
}

/// The only new-native constructors accept the existing genuine pending types.
#[expect(
    clippy::large_enum_variant,
    reason = "Inline native closure storage avoids a new unadmitted heap allocation"
)]
enum NativePending {
    /// Full Mapping manifest/product/lifecycle predicates have run in its actual factory.
    Mapping(PendingMappingClosure),
    /// Full applicability report/current/lifecycle predicates have run in its actual factory.
    Applicability(PendingApplicabilityClosure),
}

impl PendingNewNativeCohort {
    /// Consume the factory's genuine Mapping pending object; this is not held/current sealing.
    pub(in crate::review) fn mapping(pending: PendingMappingClosure) -> Self {
        Self { native: NativePending::Mapping(pending) }
    }

    /// Consume the factory's genuine applicability pending object; plain facts cannot substitute.
    pub(in crate::review) fn applicability(pending: PendingApplicabilityClosure) -> Self {
        Self { native: NativePending::Applicability(pending) }
    }

    /// Borrow the complete already validated Source pin roster without dropping any member.
    fn pins(&self) -> &[SourcePin] {
        match &self.native {
            NativePending::Mapping(pending) => pending.supersession_pins(),
            NativePending::Applicability(pending) => pending.supersession_pins(),
        }
    }

    /// Count every factory original including the exact original locator.
    fn original_count(&self) -> usize {
        match &self.native {
            NativePending::Mapping(pending) => pending.supersession_original_count(),
            NativePending::Applicability(pending) => pending.supersession_original_count(),
        }
    }

    /// Borrow the factory's genuine locator ownership binding, never a caller index selector.
    fn locator(&self) -> Option<(usize, &CaptureLease)> {
        match &self.native {
            NativePending::Mapping(pending) => pending.supersession_locator(),
            NativePending::Applicability(pending) => pending.supersession_locator(),
        }
    }

    /// Borrow a member of the complete consumed factory roster in its original order.
    fn member(&self, slot: usize) -> Option<(usize, &CaptureLease)> {
        match &self.native {
            NativePending::Mapping(pending) => pending.supersession_member(slot),
            NativePending::Applicability(pending) => pending.supersession_member(slot),
        }
    }

    /// Consume the exact pending value checked by the private complete-union issuer.
    fn seal(self, union: &NativeUnionSeal) -> NewNativeCohort {
        match self.native {
            NativePending::Mapping(pending) => {
                NewNativeCohort::Mapping(pending.seal_supersession_union(union))
            }
            NativePending::Applicability(pending) => {
                NewNativeCohort::Applicability(pending.seal_supersession_union(union))
            }
        }
    }
}

/// Private consumed-union token; only this complete issuer can construct or expose a value.
/// It is never a caller vector, completeness boolean, serialized record or alternate owner.
pub(in crate::review) struct NativeUnionSeal {
    /// Same genuine held owner after every complete cohort and original fence succeeded.
    held: Rc<HeldReviewInputs>,
}

impl NativeUnionSeal {
    /// Borrow the sole actual owner for the factory's pure consuming construction.
    pub(in crate::review) fn held_inputs(&self) -> &Rc<HeldReviewInputs> {
        &self.held
    }
}

/// Existing genuine domain closure issued only from the consumed complete factory result.
#[expect(
    clippy::large_enum_variant,
    reason = "Inline native closure storage avoids a new unadmitted heap allocation"
)]
pub(crate) enum NewNativeCohort {
    /// Current Mapping approval/native/source predicates plus complete same-owner union.
    Mapping(ApprovedMappingClosure),
    /// Current applicability report/approval/source predicates plus complete same-owner union.
    Applicability(ApprovedApplicabilityClosure),
}

/// Final same-owner binding that retains the genuine captured Impact reader result.
/// It supplies no public queue/currentness/deserialization constructor or historical approval.
pub(crate) struct HeldSupersessionInputs {
    /// The one genuine held registry, retaining every original and reserved namespace.
    held: Rc<HeldReviewInputs>,
    /// The exact consumed genuine domain closure shares this same complete held owner.
    new_native: NewNativeCohort,
    /// Complete native comparison facts and captured reader originals remain owned.
    impact: PendingImpactCohort,
}

impl HeldSupersessionInputs {
    /// Borrow only the genuine successful old/new purpose registration on this held owner.
    /// Native queue admission/correlation is a separate coupled consumer obligation.
    pub(crate) fn queue_original(&self, purpose: QueuePurpose) -> Result<usize, ContractError> {
        self.held
            .supersession
            .as_ref()
            .and_then(|state| state.queues[purpose.slot()])
            .ok_or(ContractError::Binding)
    }

    /// Borrow only a genuine successful Auxiliary purpose for the coupled declared consumer.
    pub(crate) fn auxiliary_original(
        &self,
        purpose: AuxiliaryPurpose,
    ) -> Result<usize, ContractError> {
        self.held
            .supersession
            .as_ref()
            .and_then(|state| state.auxiliaries[purpose.slot()])
            .ok_or(ContractError::Binding)
    }

    /// Borrow the same actual held owner for coupled adapter and publication work.
    pub(crate) fn held_inputs(&self) -> &Rc<HeldReviewInputs> {
        &self.held
    }

    /// Borrow only the genuine domain closure for exact current-new queue binding.
    pub(crate) fn new_native(&self) -> &NewNativeCohort {
        &self.new_native
    }

    /// Borrow the complete captured reader comparison/cohort, never a selected report row.
    pub(crate) fn impact(&self) -> &PendingImpactCohort {
        &self.impact
    }

    /// Fence every actual original and namespace under the unchanged accepted ledger/control.
    pub(crate) fn verify_inputs(
        &self,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        ledger.bound(|ledger| {
            let result = self.held.verify_inputs(ledger, control);
            ledger.checkpoint(control)?;
            result
        })
    }
}

/// Consume both complete private pending cohorts and require exact complete Source-union equality.
/// Only the complete captured reader can supply its private Impact pending cohort.
pub(crate) fn seal_complete_union(
    held: Rc<HeldReviewInputs>,
    new_native: PendingNewNativeCohort,
    impact: PendingImpactCohort,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<HeldSupersessionInputs, ContractError> {
    ledger.bound(|ledger| {
        let state = held.supersession.as_ref().ok_or(ContractError::Binding)?;
        verify_mode(&held, state, ledger, control)?;
        verify_new_native(&held, state, &new_native, ledger, control)?;
        verify_impact_members(&held, state, &impact, ledger, control)?;
        for (index, entry) in held.entries.iter().enumerate() {
            ledger.checkpoint(control)?;
            ledger.visits(1)?;
            if entry.pool == Pool::Source
                && !union_contains(index, &new_native, &impact, ledger, control)?
            {
                return Err(ContractError::Binding);
            }
        }
        let originals = held.verify_inputs(ledger, control);
        ledger.checkpoint(control)?;
        originals?;
        ledger.derived(
            std::mem::size_of::<HeldSupersessionInputs>()
                .checked_add(std::mem::size_of::<NativeUnionSeal>())
                .and_then(|size| size.checked_add(std::mem::size_of::<NewNativeCohort>()))
                .ok_or(ContractError::Capacity)?,
        )?;
        let union = NativeUnionSeal { held: Rc::clone(&held) };
        let new_native = new_native.seal(&union);
        ledger.checkpoint(control)?;
        Ok(HeldSupersessionInputs { held, new_native, impact })
    })
}

/// Check actual purpose geometry and the native registry/entry bijection, not a caller assertion.
fn verify_mode(
    held: &HeldReviewInputs,
    state: &SupersessionRegistrations,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    ledger.checkpoint(control)?;
    ledger.visits(
        MAX_SUPERSESSION_SLOTS
            .checked_add(held.entries.len())
            .and_then(|count| count.checked_add(6))
            .ok_or(ContractError::Capacity)?,
    )?;
    ledger.bytes(
        held.entries
            .len()
            .checked_mul(std::mem::size_of::<Pool>())
            .ok_or(ContractError::Capacity)?,
    )?;
    if state.attempts > MAX_SUPERSESSION_SLOTS
        || state.queue_attempts != [1; 2]
        || state.auxiliary_attempts != [1; 3]
        || held.queue_original.is_some()
        || !held.response_originals.is_empty()
        || held.proof.captured_original_generations() != held.entries.len()
    {
        return Err(ContractError::Binding);
    }
    let [Some(old), Some(new)] = state.queues else { return Err(ContractError::Binding) };
    if old == new {
        return Err(ContractError::Binding);
    }
    for index in [old, new] {
        if held
            .entries
            .get(index)
            .is_none_or(|entry| entry.pool != Pool::Queue || entry.role != CaptureRole::ReviewQueue)
        {
            return Err(ContractError::Binding);
        }
    }
    let mut source_count = 0_usize;
    let mut source_bytes = 0_usize;
    let mut queue_bytes = 0_usize;
    let mut auxiliary_count = 0_usize;
    for (index, entry) in held.entries.iter().enumerate() {
        ledger.checkpoint(control)?;
        ledger.visits(1)?;
        match entry.pool {
            Pool::Source => {
                source_count = source_count.checked_add(1).ok_or(ContractError::Capacity)?;
                source_bytes = source_bytes
                    .checked_add(entry.lease.bytes().len())
                    .ok_or(ContractError::Capacity)?;
                if entry.lease.bytes().len() > MAX_QUEUE_BYTES {
                    return Err(ContractError::Binding);
                }
            }
            Pool::Queue => {
                if entry.role != CaptureRole::ReviewQueue
                    || (index != old && index != new)
                    || entry.lease.bytes().len() > MAX_QUEUE_BYTES
                {
                    return Err(ContractError::Binding);
                }
                queue_bytes = queue_bytes
                    .checked_add(entry.lease.bytes().len())
                    .ok_or(ContractError::Capacity)?;
            }
            Pool::Auxiliary => {
                if entry.role != CaptureRole::ReviewPrivateConfig
                    || !state.auxiliaries.contains(&Some(index))
                    || entry.lease.bytes().len() > MAX_AUXILIARY_BYTES
                {
                    return Err(ContractError::Binding);
                }
                auxiliary_count += 1;
            }
            Pool::Response | Pool::Recorded => return Err(ContractError::Binding),
        }
    }
    if source_count > MAX_SOURCE_ATTEMPTS
        || source_bytes > MAX_SOURCE_BYTES
        || queue_bytes > 2 * MAX_QUEUE_BYTES
        || auxiliary_count != 3
    {
        return Err(ContractError::Binding);
    }
    for index in state.queues.into_iter().chain(state.auxiliaries).flatten() {
        ledger.visits(1)?;
        if held.entries.get(index).is_none() {
            return Err(ContractError::Binding);
        }
    }
    ledger.checkpoint(control)
}

/// Check every consumed genuine native member, its actual role/lease and complete raw pin.
fn verify_new_native(
    held: &HeldReviewInputs,
    state: &SupersessionRegistrations,
    native: &PendingNewNativeCohort,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    ledger.checkpoint(control)?;
    let pins = native.pins();
    if pins.is_empty()
        || pins.len() > MAX_SOURCE_ATTEMPTS
        || native.original_count() != pins.len().checked_add(1).ok_or(ContractError::Capacity)?
    {
        return Err(ContractError::Binding);
    }
    let (locator_index, locator_lease) = native.locator().ok_or(ContractError::Binding)?;
    if state.auxiliaries[AuxiliaryPurpose::NewNativeLocator.slot()] != Some(locator_index) {
        return Err(ContractError::Binding);
    }
    verify_member(
        held,
        locator_index,
        locator_lease,
        Pool::Auxiliary,
        CaptureRole::ReviewPrivateConfig,
        ledger,
        control,
    )?;
    for (slot, pin) in pins.iter().enumerate() {
        let (index, lease) = native.member(slot).ok_or(ContractError::Binding)?;
        verify_member(held, index, lease, Pool::Source, native_role(pin.model), ledger, control)?;
        for earlier in 0..slot {
            ledger.checkpoint(control)?;
            ledger.visits(1)?;
            if native.member(earlier).ok_or(ContractError::Binding)?.0 == index {
                return Err(ContractError::Binding);
            }
        }
        ledger.bytes(lease.bytes().len())?;
        ledger.derived(64)?;
        let actual_hash = sha256_hex(lease.bytes());
        ledger.bytes(
            actual_hash.len().checked_add(pin.raw_sha256.len()).ok_or(ContractError::Capacity)?,
        )?;
        if u64::try_from(lease.bytes().len()).map_err(|_| ContractError::Capacity)?
            != pin.byte_length
            || actual_hash != pin.raw_sha256
        {
            return Err(ContractError::Binding);
        }
    }
    ledger.checkpoint(control)
}

/// Preserve the closed existing public `SourceModel` vocabulary and its exact native roles.
fn native_role(model: SourceModel) -> CaptureRole {
    match model {
        SourceModel::Mapping => CaptureRole::MappingCollection,
        SourceModel::Catalog | SourceModel::ResolvedCatalog => CaptureRole::Catalog,
        SourceModel::Profile => CaptureRole::Profile,
        SourceModel::ComponentDefinition => CaptureRole::ComponentDefinition,
        SourceModel::MappingManifest => CaptureRole::MappingManifest,
        SourceModel::ApplicabilityManifest => CaptureRole::ApplicabilityManifest,
        SourceModel::ApplicabilityReport => CaptureRole::ApplicabilityReport,
        SourceModel::LifecycleRecord => CaptureRole::LifecycleRecord,
    }
}

/// Bind every captured reader member to the full actual ordered purpose registration cohort.
fn verify_impact_members(
    held: &HeldReviewInputs,
    state: &SupersessionRegistrations,
    impact: &PendingImpactCohort,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    ledger.checkpoint(control)?;
    let (locator_index, locator_lease) = impact.locator();
    let members = impact.members();
    if state.auxiliaries[AuxiliaryPurpose::ImpactLocator.slot()] != Some(locator_index)
        || members.is_empty()
        || members.len() > MAX_SOURCE_ATTEMPTS
        || members.len() != state.impact_uses.len()
    {
        return Err(ContractError::Binding);
    }
    verify_member(
        held,
        locator_index,
        locator_lease,
        Pool::Auxiliary,
        CaptureRole::ReviewPrivateConfig,
        ledger,
        control,
    )?;
    let mut manifests = 0_usize;
    let mut current_reports = 0_usize;
    for (member, registered) in members.iter().zip(&state.impact_uses) {
        ledger.checkpoint(control)?;
        ledger.visits(1)?;
        ledger.bytes(2 * std::mem::size_of::<ImpactUse>())?;
        if member.index() != registered.index || member.purpose() != registered.purpose {
            return Err(ContractError::Binding);
        }
        verify_member(
            held,
            member.index(),
            member.lease(),
            Pool::Source,
            member.purpose().role(),
            ledger,
            control,
        )?;
        if member.purpose() == ImpactSourcePurpose::Manifest {
            manifests += 1;
        }
        if member.purpose() == ImpactSourcePurpose::CurrentReport {
            current_reports += 1;
        }
    }
    if manifests != 1 || current_reports != 1 {
        return Err(ContractError::Binding);
    }
    // These geometry checks cannot establish native read completeness. The pending cohort
    // must come from the complete captured reader's native computation and stored-report oracle.
    ledger.checkpoint(control)
}

/// Check actual allocation identity on the one held owner before consuming any member.
fn verify_member(
    held: &HeldReviewInputs,
    index: usize,
    lease: &CaptureLease,
    pool: Pool,
    role: CaptureRole,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    ledger.checkpoint(control)?;
    ledger.visits(1)?;
    ledger.bytes(2 * std::mem::size_of::<CaptureLease>() + std::mem::size_of::<usize>())?;
    let entry = held.entries.get(index).ok_or(ContractError::Binding)?;
    if entry.pool != pool || entry.role != role || !entry.lease.same_original(lease) {
        return Err(ContractError::Binding);
    }
    Ok(())
}

/// Charge both complete membership scans; compatible overlap is one retained Source allocation.
fn union_contains(
    index: usize,
    native: &PendingNewNativeCohort,
    impact: &PendingImpactCohort,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<bool, ContractError> {
    let mut found = false;
    for slot in 0..native.pins().len() {
        ledger.checkpoint(control)?;
        ledger.visits(1)?;
        ledger.bytes(2 * std::mem::size_of::<usize>())?;
        found |= native.member(slot).ok_or(ContractError::Binding)?.0 == index;
    }
    for member in impact.members() {
        ledger.checkpoint(control)?;
        ledger.visits(1)?;
        ledger.bytes(2 * std::mem::size_of::<usize>())?;
        found |= member.index() == index;
    }
    Ok(found)
}

/// Real-file purpose-qualified capture geometry and original-owner controls.
#[cfg(test)]
#[path = "supersession_capture_tests.rs"]
mod tests;

/// Actual complete captured native/Impact union controls; no full command claim.
#[cfg(test)]
#[path = "supersession_union_tests.rs"]
mod union_tests;
