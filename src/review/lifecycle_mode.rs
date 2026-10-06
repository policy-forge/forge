//! Private Lifecycle registration mode on the actual original review capture.
//! Every successful purpose occurrence is retained; repeated originals share only
//! their real native lease. The ordinary /1 role table and all original caps stay fixed.

use std::path::Path;

use crate::evidence_capture::{CaptureLease, CaptureRole};
use crate::workspace::preparation::WorkControl;

use super::super::decode::{ContractError, ContractLedger};
use super::super::lifecycle_capture::{LifecycleMember, PendingLifecycleCohort};
use super::{Entry, HeldReviewInputs, Pool, ReviewCapture, capture_charge, verification_error};

/// Complete native purpose, independent of any raw byte role or proof capability.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum LifecyclePurpose {
    /// The one intrinsic /2 record original, registered first.
    Record,
    /// The one declared source occurrence, including exact record/source reuse.
    Source,
    /// A generated occurrence in full exact native-path order, not public key order.
    Generated(usize),
}

/// One actual successful registration; its private origin is the real session only.
struct LifecycleUse {
    /// Actual capture Entry index; compatible repeats retain that same index.
    index: usize,
    /// Real successful purpose, never supplied as evidence of completeness.
    purpose: LifecyclePurpose,
}

/// Mode-private actual registries survive capture.finish with the original proof.
pub(super) struct LifecycleRegistrations {
    /// One genuine explicit Auxiliary locator registration.
    locator: Option<usize>,
    /// Complete successful record/source/generated occurrence order.
    uses: Vec<LifecycleUse>,
}

impl ReviewCapture {
    /// Start this mode on the ordinary actual capture; accept no replacement control.
    pub(crate) fn new_lifecycle(
        root: &Path,
        outputs: &[&Path],
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<Self, ContractError> {
        ledger.bound(|ledger| {
            let prepared = Self::new(root, outputs, ledger, control);
            ledger.checkpoint(control)?;
            let mut capture = prepared?;
            ledger.derived(std::mem::size_of::<LifecycleRegistrations>())?;
            capture.lifecycle = Some(LifecycleRegistrations { locator: None, uses: Vec::new() });
            ledger.checkpoint(control)?;
            Ok(capture)
        })
    }

    /// Register the exact locator using the unchanged bounded ordinary Auxiliary path.
    pub(crate) fn required_lifecycle_locator(
        &mut self,
        path: &Path,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<usize, ContractError> {
        ledger.bound(|ledger| {
            ledger.checkpoint(control)?;
            let actual = self.required(
                path,
                CaptureRole::ReviewPrivateConfig,
                Pool::Auxiliary,
                1024 * 1024,
                ledger,
                control,
            );
            ledger.checkpoint(control)?;
            let index = actual?;
            let state = self.lifecycle.as_mut().ok_or(ContractError::Invalid)?;
            if state.locator.is_some() {
                return Err(ContractError::Invalid);
            }
            state.locator = Some(index);
            ledger.checkpoint(control)?;
            Ok(index)
        })
    }

    /// Retain each native occurrence on the same Source pool before exposing its index.
    pub(crate) fn required_lifecycle_source(
        &mut self,
        path: &Path,
        purpose: LifecyclePurpose,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<usize, ContractError> {
        ledger.bound(|ledger| {
            ledger.checkpoint(control)?;
            let admitted = self.lifecycle_admission(path, 10 * 1024 * 1024, ledger, control);
            ledger.checkpoint(control)?;
            let components = admitted?;
            let state = self.lifecycle.as_ref().ok_or(ContractError::Invalid)?;
            let expected = match state.uses.len() {
                0 => LifecyclePurpose::Record,
                1 => LifecyclePurpose::Source,
                n => LifecyclePurpose::Generated(n - 2),
            };
            ledger.visits(1)?;
            if purpose != expected || state.locator.is_none() || self.supersession.is_some() {
                return Err(ContractError::Binding);
            }
            let role = self.lifecycle_role(path, purpose, ledger, control)?;
            let maximum = if role == CaptureRole::LifecycleRecord {
                usize::try_from(crate::lifecycle::record::MAX_RECORD_BYTES)
                    .map_err(|_| ContractError::Capacity)?
            } else {
                10 * 1024 * 1024
            };
            let captured =
                self.lifecycle_original(path, role, maximum, components, ledger, control);
            ledger.checkpoint(control)?;
            let index = captured?;
            let state = self.lifecycle.as_mut().ok_or(ContractError::Binding)?;
            ledger.derived(std::mem::size_of::<LifecycleUse>())?;
            if state.uses.len() == state.uses.capacity() {
                ledger.bytes(
                    state
                        .uses
                        .len()
                        .checked_mul(std::mem::size_of::<LifecycleUse>())
                        .ok_or(ContractError::Capacity)?,
                )?;
            }
            state.uses.try_reserve(1).map_err(|_| ledger.capacity())?;
            state.uses.push(LifecycleUse { index, purpose });
            ledger.checkpoint(control)?;
            Ok(index)
        })
    }

    /// Reuse the actual intrinsic record role only for the complete Source occurrence.
    fn lifecycle_role(
        &self,
        path: &Path,
        purpose: LifecyclePurpose,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<CaptureRole, ContractError> {
        ledger.checkpoint(control)?;
        if purpose == LifecyclePurpose::Record {
            return Ok(CaptureRole::LifecycleRecord);
        }
        if purpose == LifecyclePurpose::Source {
            let record = self
                .lifecycle
                .as_ref()
                .and_then(|s| s.uses.first())
                .ok_or(ContractError::Binding)?;
            let entry = self.entries.get(record.index).ok_or(ContractError::Binding)?;
            ledger.visits(1)?;
            ledger.bytes(
                entry
                    .path
                    .as_os_str()
                    .as_encoded_bytes()
                    .len()
                    .checked_add(path.as_os_str().as_encoded_bytes().len())
                    .ok_or(ContractError::Capacity)?,
            )?;
            if entry.path == path {
                return Ok(CaptureRole::LifecycleRecord);
            }
        }
        Ok(CaptureRole::LifecycleArtifactOriginal)
    }

    /// Charge a foreign ordinary Source attempt but never widen /1 role compatibility.
    pub(super) fn reject_lifecycle_source(
        &mut self,
        path: &Path,
        maximum: usize,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<usize, ContractError> {
        ledger.bound(|ledger| {
            let result = self.lifecycle_admission(path, maximum, ledger, control);
            ledger.checkpoint(control)?;
            result?;
            Err(ContractError::Invalid)
        })
    }

    /// Count all failed/repeated attempts and precharge complete actual path comparisons.
    fn lifecycle_admission(
        &mut self,
        path: &Path,
        maximum: usize,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<usize, ContractError> {
        ledger.checkpoint(control)?;
        self.attempts = self
            .attempts
            .checked_add(1)
            .filter(|n| *n <= super::MAX_ATTEMPTS)
            .ok_or(ContractError::Capacity)?;
        let count = &mut self.pools[Pool::Source.index()];
        count.registrations = count
            .registrations
            .checked_add(1)
            .filter(|n| *n <= 100)
            .ok_or(ContractError::Capacity)?;
        let spelling = path.as_os_str().as_encoded_bytes();
        ledger.bytes(spelling.len())?;
        let components = path.components().count();
        ledger.visits(components.checked_add(1).ok_or(ContractError::Capacity)?)?;
        if spelling.len() > 4096 || components > 64 || maximum > 10 * 1024 * 1024 {
            return Err(ContractError::Invalid);
        }
        crate::linkage::fresh::validate_relative(path).map_err(|_| ContractError::Invalid)?;
        ledger.visits(self.entries.len())?;
        let operand_work = self.entries.iter().try_fold(0_usize, |total, entry| {
            total
                .checked_add(entry.path.as_os_str().as_encoded_bytes().len())
                .and_then(|n| n.checked_add(spelling.len()))
                .ok_or(ContractError::Capacity)
        })?;
        ledger.bytes(operand_work)?;
        let aliases = self.reject_folded_aliases(path, components, ledger, control);
        ledger.checkpoint(control)?;
        aliases?;
        Ok(components)
    }

    /// Use only the native original session; preserve failures through the actual post fence.
    fn lifecycle_lease(
        &mut self,
        path: &Path,
        role: CaptureRole,
        maximum: usize,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<CaptureLease, ContractError> {
        let actual = ledger.bound(|ledger| {
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
        actual
    }

    /// Reserve the actual new Entry and full root/path geometry before native capture grows.
    fn lifecycle_original(
        &mut self,
        path: &Path,
        role: CaptureRole,
        maximum: usize,
        components: usize,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<usize, ContractError> {
        if let Some(index) = self.entries.iter().position(|entry| entry.path == path) {
            let entry = &self.entries[index];
            if entry.pool != Pool::Source || entry.role != role {
                return Err(ContractError::Binding);
            }
            if entry.lease.bytes().len() > maximum {
                return Err(ledger.capacity());
            }
            self.lifecycle_lease(path, role, maximum, ledger, control)?;
            return Ok(index);
        }
        let remaining = (50 * 1024 * 1024_usize)
            .checked_sub(self.pools[Pool::Source.index()].bytes)
            .ok_or(ContractError::Capacity)?;
        let spelling = path.as_os_str().as_encoded_bytes().len();
        let ancestor = self
            .root_extent
            .checked_add(spelling)
            .and_then(|n| n.checked_add(256))
            .and_then(|n| n.checked_mul(2))
            .and_then(|n| n.checked_mul(components))
            .ok_or(ContractError::Capacity)?;
        ledger.derived(
            spelling
                .checked_mul(3)
                .and_then(|n| n.checked_add(256))
                .and_then(|n| n.checked_add(ancestor))
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
        let lease = self.lifecycle_lease(path, role, maximum.min(remaining), ledger, control)?;
        let count = &mut self.pools[Pool::Source.index()];
        count.bytes = count
            .bytes
            .checked_add(lease.bytes().len())
            .filter(|n| *n <= 50 * 1024 * 1024)
            .ok_or(ContractError::Capacity)?;
        ledger.checkpoint(control)?;
        let index = self.entries.len();
        self.entries.push(Entry {
            path: path.to_path_buf(),
            components,
            role,
            pool: Pool::Source,
            lease,
        });
        Ok(index)
    }
}

impl HeldReviewInputs {
    /// Compare all genuine mode registrations and every distinct actual Source Entry.
    pub(in crate::review) fn verify_lifecycle_cohort(
        &self,
        pending: &PendingLifecycleCohort,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        ledger.bound(|ledger| {
            ledger.checkpoint(control)?;
            let state = self.lifecycle.as_ref().ok_or(ContractError::Binding)?;
            ledger.visits(
                self.entries
                    .len()
                    .checked_add(state.uses.len())
                    .and_then(|n| n.checked_add(1))
                    .ok_or(ContractError::Capacity)?,
            )?;
            if self.supersession.is_some()
                || state.locator != Some(pending.locator().0)
                || self.proof.captured_original_generations() != self.entries.len()
                || state.uses.len() != pending.members().len()
                || !(2..=100).contains(&state.uses.len())
            {
                return Err(ContractError::Binding);
            }
            let locator = self.entries.get(pending.locator().0).ok_or(ContractError::Binding)?;
            if locator.pool != Pool::Auxiliary
                || locator.role != CaptureRole::ReviewPrivateConfig
                || !self.same_original(pending.locator().0, pending.locator().1)?
            {
                return Err(ContractError::Binding);
            }
            for (actual, expected) in state.uses.iter().zip(pending.members()) {
                self.verify_lifecycle_member(actual, expected, ledger, control)?;
            }
            let mut total = 0_usize;
            for (index, entry) in self.entries.iter().enumerate() {
                ledger.checkpoint(control)?;
                ledger.visits(1)?;
                if entry.pool == Pool::Source {
                    ledger.visits(state.uses.len())?;
                    if !state.uses.iter().any(|row| row.index == index) {
                        return Err(ContractError::Binding);
                    }
                    total = total
                        .checked_add(entry.lease.bytes().len())
                        .ok_or(ContractError::Capacity)?;
                    if entry.lease.bytes().len() > 10 * 1024 * 1024 {
                        return Err(ContractError::Binding);
                    }
                }
            }
            if total > 50 * 1024 * 1024 {
                return Err(ContractError::Binding);
            }
            ledger.checkpoint(control)
        })
    }

    /// Bind actual per-purpose role/route/index/lease identities without accepting detached pins.
    fn verify_lifecycle_member(
        &self,
        actual: &LifecycleUse,
        expected: &LifecycleMember,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        ledger.checkpoint(control)?;
        ledger.visits(1)?;
        if actual.index != expected.index() || actual.purpose != expected.purpose() {
            return Err(ContractError::Binding);
        }
        let entry = self.entries.get(actual.index).ok_or(ContractError::Binding)?;
        if entry.pool != Pool::Source
            || entry.role != expected.role()
            || !self.same_original(actual.index, expected.lease())?
        {
            return Err(ContractError::Binding);
        }
        ledger.bytes(
            entry
                .path
                .as_os_str()
                .as_encoded_bytes()
                .len()
                .checked_add(expected.path().as_os_str().as_encoded_bytes().len())
                .ok_or(ContractError::Capacity)?,
        )?;
        if entry.path != expected.path() {
            return Err(ContractError::Binding);
        }
        ledger.checkpoint(control)
    }
}

/// Borrowed observations of this genuine Lifecycle owner's actual review registrations.
/// Private fields prevent caller indices, copied bytes or an arbitrary response list
/// from creating the view. This view issues no native/currentness capability.
pub(crate) struct RegisteredLifecycleReviewOriginals<'a> {
    /// Sole actual original owner, borrowed without cloning its Rc.
    held: &'a HeldReviewInputs,
    /// Internally obtained successful Queue registration, never a caller selector.
    queue_index: usize,
    /// All successful Response occurrences in original registration order, including repeats.
    response_indices: &'a [usize],
}
impl<'a> RegisteredLifecycleReviewOriginals<'a> {
    /// Borrow the complete actually registered Queue allocation.
    pub(crate) fn queue_raw(&self) -> &'a [u8] {
        self.held.entries[self.queue_index].lease.bytes()
    }
    /// Borrow every actual Response allocation in complete original occurrence order.
    pub(crate) fn responses(&self) -> impl ExactSizeIterator<Item = &'a [u8]> + 'a {
        let held = self.held;
        let indices = self.response_indices;
        indices.iter().map(move |index| held.entries[*index].lease.bytes())
    }
    /// Borrow the actual Queue registration for Root's whole-original finalizer.
    pub(crate) fn queue_index(&self) -> usize {
        self.queue_index
    }
    /// Borrow the complete original registration indices; no caller-list constructor exists.
    pub(crate) fn response_indices(&self) -> &'a [usize] {
        self.response_indices
    }
}
impl HeldReviewInputs {
    /// Inspect this Lifecycle mode's exact complete successful Queue/Response registrations.
    /// Native Sources and the private locator remain in the same actual held owner.
    /// Ordinary failures pass the original caller postfence; earlier admission stops win.
    pub(crate) fn lifecycle_review_originals(
        &self,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<RegisteredLifecycleReviewOriginals<'_>, ContractError> {
        super::super::decode_v2::phase(ledger, control, |ledger, control| {
            ledger.visits(1)?;
            if self.lifecycle.is_none()
                || self.supersession.is_some()
                || self.proof.captured_original_generations() != self.entries.len()
                || self.response_originals.len() > 10_000
            {
                return Err(ContractError::Binding);
            }
            let queue_index = self.queue_original.ok_or(ContractError::Binding)?;
            let queue = self.entries.get(queue_index).ok_or(ContractError::Binding)?;
            if queue.pool != Pool::Queue || queue.role != CaptureRole::ReviewQueue {
                return Err(ContractError::Binding);
            }
            // One scratch mark per distinct actual Entry avoids an unbounded nested
            // occurrence/Entry membership scan. Repeats mark the same real Entry.
            let slots = self.entries.len();
            let extent =
                slots.checked_mul(std::mem::size_of::<bool>()).ok_or_else(|| ledger.capacity())?;
            ledger.derived(extent)?;
            ledger.bytes(extent)?;
            let mut registered = Vec::new();
            registered.try_reserve_exact(slots).map_err(|_| ledger.capacity())?;
            registered.resize(slots, false);
            for &index in &self.response_originals {
                super::super::decode_v2::checkpoint(ledger, control)?;
                ledger.visits(1)?;
                ledger.bytes(std::mem::size_of::<usize>() + std::mem::size_of::<bool>())?;
                let entry = self.entries.get(index).ok_or(ContractError::Binding)?;
                if entry.pool != Pool::Response || entry.role != CaptureRole::ReviewResponse {
                    return Err(ContractError::Binding);
                }
                registered[index] = true;
            }
            for (index, entry) in self.entries.iter().enumerate() {
                super::super::decode_v2::checkpoint(ledger, control)?;
                ledger.visits(1)?;
                ledger.bytes(std::mem::size_of::<usize>() + std::mem::size_of::<bool>())?;
                if (entry.pool == Pool::Queue && index != queue_index)
                    || (entry.pool == Pool::Response && !registered[index])
                {
                    return Err(ContractError::Binding);
                }
            }
            // Delegate the maintained exact registration predicate with its own
            // internally obtained actual indices; no supplied subset is admitted.
            self.bind_review_originals(queue_index, &self.response_originals, ledger, control)?;
            ledger.derived(std::mem::size_of::<RegisteredLifecycleReviewOriginals<'_>>())?;
            Ok(RegisteredLifecycleReviewOriginals {
                held: self,
                queue_index,
                response_indices: &self.response_originals,
            })
        })
    }
}

/// Bind one whole actually held private init-request allocation on this Lifecycle owner.
/// No caller index/list/hash or detached byte copy can select an Auxiliary original.
impl HeldReviewInputs {
    /// Return the real lease bytes only after complete actual Entry/role/pool observation.
    pub(crate) fn lifecycle_init_original(
        &self,
        supplied: &[u8],
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<&[u8], ContractError> {
        super::super::decode_v2::phase(ledger, control, |ledger, control| {
            ledger.visits(1)?;
            if self.lifecycle.is_none()
                || self.supersession.is_some()
                || self.proof.captured_original_generations() != self.entries.len()
            {
                return Err(ContractError::Binding);
            }
            // Pointer and full extent are constant-size observations, not byte equality
            // or a digest repair. Admit the complete bounded scan before membership.
            let work = self
                .entries
                .len()
                .checked_mul(4 * std::mem::size_of::<usize>())
                .ok_or_else(|| ledger.capacity())?;
            ledger.bytes(work)?;
            ledger.visits(self.entries.len())?;
            let mut selected = None;
            for entry in &self.entries {
                super::super::decode_v2::checkpoint(ledger, control)?;
                let raw = entry.lease.bytes();
                if raw.as_ptr() == supplied.as_ptr() && raw.len() == supplied.len() {
                    if selected.is_some()
                        || entry.pool != Pool::Auxiliary
                        || entry.role != CaptureRole::ReviewPrivateConfig
                    {
                        return Err(ContractError::Binding);
                    }
                    selected = Some(raw);
                }
            }
            selected.ok_or(ContractError::Binding)
        })
    }
}
