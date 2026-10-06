//! Private Authoring-plan registration mode on the genuine original capture.
//! Native occurrences and stored-plan are distinct Source entries; ordinary /1
//! compatible roles and all fixed pools remain unchanged.
use super::super::authoring_capture::{AuthoringMember, PendingAuthoringCohort};
use super::super::decode::{ContractError, ContractLedger};
use super::super::decode_v2::{checkpoint, phase};
use super::{Entry, HeldReviewInputs, Pool, ReviewCapture, capture_charge, verification_error};
use crate::evidence_capture::{CaptureLease, CaptureRole};
use crate::workspace::preparation::WorkControl;
use std::path::Path;

/// Closed operation purpose governs complete actually registered review inputs.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuthoringOperation {
    /// New output Queue: locator and init-policy, no input Queue or Responses.
    Init,
    /// Current merge: locator, one input Queue and complete Response occurrences.
    Merge,
    /// Current status: locator, one input Queue and complete Response occurrences.
    Status,
}
/// Complete native read purpose; public pin order is a separate minimized projection.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuthoringPurpose {
    /// Actual project filename original, native first input.
    Project,
    /// Exact declared native pack.
    Pack,
    /// Exact full saved baseline report.
    GapReport,
    /// Exact native App manifest.
    Applicability,
    /// Full native framework original.
    Framework,
    /// Explicit complete Profile companion only.
    Resolved,
    /// Complete Mapping declaration ordinal.
    Mapping(usize),
    /// Complete native human-clause declaration ordinal.
    Clause(usize),
    /// Separate stored-plan Source; excluded from every native provenance roster.
    StoredPlan,
}
/// Actual successful distinct Source registration from this mode's original session.
struct AuthoringUse {
    /// Exact actual Entry index, never a caller authority selector.
    index: usize,
    /// Actual successful native purpose.
    purpose: AuthoringPurpose,
}
/// Complete mode-private successful registries survive only the genuine finish transfer.
pub(super) struct AuthoringRegistrations {
    /// Closed selected operation, not a caller completeness Boolean.
    operation: AuthoringOperation,
    /// Actual one Auxiliary locator.
    locator: Option<usize>,
    /// Actual one init-policy only for Init.
    policy: Option<usize>,
    /// Every actual Source registration in complete receiver read order.
    uses: Vec<AuthoringUse>,
    /// Every successful actual Auxiliary occurrence, including repeated ordinary attempts.
    auxiliaries: Vec<usize>,
}
impl ReviewCapture {
    /// Start authoring mode on the unchanged physical capture with the original ledger/control.
    pub(crate) fn new_authoring(
        root: &Path,
        outputs: &[&Path],
        operation: AuthoringOperation,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<Self, ContractError> {
        phase(ledger, control, |ledger, control| {
            let prepared = Self::new(root, outputs, ledger, control);
            checkpoint(ledger, control)?;
            let mut capture = prepared?;
            ledger.derived(std::mem::size_of::<AuthoringRegistrations>())?;
            capture.authoring = Some(AuthoringRegistrations {
                operation,
                locator: None,
                policy: None,
                uses: Vec::new(),
                auxiliaries: Vec::new(),
            });
            Ok(capture)
        })
    }
    /// Register exactly one actual locator before any native Source read.
    pub(crate) fn required_authoring_locator(
        &mut self,
        path: &Path,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<usize, ContractError> {
        self.authoring_auxiliary(path, false, ledger, control)
    }
    /// Register exactly one init-policy original; current operations refuse this purpose.
    pub(crate) fn required_authoring_policy(
        &mut self,
        path: &Path,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<usize, ContractError> {
        self.authoring_auxiliary(path, true, ledger, control)
    }
    /// Count/admit the actual Auxiliary attempt before private purpose registration.
    fn authoring_auxiliary(
        &mut self,
        path: &Path,
        policy: bool,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<usize, ContractError> {
        phase(ledger, control, |ledger, control| {
            let raw = self.required(
                path,
                CaptureRole::ReviewPrivateConfig,
                Pool::Auxiliary,
                1024 * 1024,
                ledger,
                control,
            );
            checkpoint(ledger, control)?;
            let index = raw?;
            let state = self.authoring.as_mut().ok_or(ContractError::Binding)?;
            if policy {
                if state.operation != AuthoringOperation::Init
                    || state.policy.is_some()
                    || state.locator == Some(index)
                {
                    return Err(ContractError::Binding);
                }
                state.policy = Some(index);
            } else {
                if state.locator.is_some() || state.policy == Some(index) {
                    return Err(ContractError::Binding);
                }
                state.locator = Some(index);
            }
            Ok(index)
        })
    }
    /// Record every ordinary Auxiliary registration before mode-purpose attribution.
    /// A repeated or unrelated Auxiliary cannot hide behind a reused Entry index.
    pub(super) fn register_authoring_auxiliary(
        &mut self,
        index: usize,
        ledger: &mut ContractLedger,
    ) -> Result<(), ContractError> {
        let state = self.authoring.as_mut().ok_or(ContractError::Binding)?;
        ledger.visits(1)?;
        ledger.derived(std::mem::size_of::<usize>())?;
        if state.auxiliaries.len() == state.auxiliaries.capacity() {
            ledger.bytes(
                state
                    .auxiliaries
                    .len()
                    .checked_mul(std::mem::size_of::<usize>())
                    .ok_or(ContractError::Capacity)?,
            )?;
        }
        state.auxiliaries.try_reserve(1).map_err(|_| ledger.capacity())?;
        state.auxiliaries.push(index);
        Ok(())
    }
    /// Capture one distinct source occurrence, never reusing a route or physical alias.
    pub(crate) fn required_authoring_source(
        &mut self,
        path: &Path,
        purpose: AuthoringPurpose,
        maximum: usize,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<usize, ContractError> {
        phase(ledger, control, |ledger, control| {
            let admitted = self.authoring_admission(path, maximum, ledger, control);
            checkpoint(ledger, control)?;
            let components = admitted?;
            let state = self.authoring.as_ref().ok_or(ContractError::Binding)?;
            let inspections = state
                .uses
                .len()
                .checked_mul(2)
                .and_then(|n| n.checked_add(1))
                .ok_or_else(|| ledger.capacity())?;
            ledger.visits(inspections)?;
            ledger.bytes(
                state
                    .uses
                    .len()
                    .checked_mul(2 * std::mem::size_of::<AuthoringUse>())
                    .ok_or(ContractError::Capacity)?,
            )?;
            if self.lifecycle.is_some()
                || self.supersession.is_some()
                || state.locator.is_none()
                || state.uses.iter().any(|u| u.purpose == purpose)
                || state.uses.iter().any(|u| u.purpose == AuthoringPurpose::StoredPlan)
                || (state.uses.is_empty() && purpose != AuthoringPurpose::Project)
            {
                return Err(ContractError::Binding);
            }
            let actual = self.authoring_original(
                path,
                CaptureRole::AuthoringArtifactOriginal,
                maximum,
                components,
                ledger,
                control,
            );
            checkpoint(ledger, control)?;
            let index = actual?;
            let state = self.authoring.as_mut().ok_or(ContractError::Binding)?;
            ledger.derived(std::mem::size_of::<AuthoringUse>())?;
            if state.uses.len() == state.uses.capacity() {
                let extent = state
                    .uses
                    .len()
                    .checked_mul(std::mem::size_of::<AuthoringUse>())
                    .ok_or_else(|| ledger.capacity())?;
                ledger.bytes(extent)?;
            }
            state.uses.try_reserve(1).map_err(|_| ledger.capacity())?;
            state.uses.push(AuthoringUse { index, purpose });
            Ok(index)
        })
    }
    /// Count an ordinary foreign Source attempt but preserve the unchanged /1 role whitelist.
    pub(super) fn reject_authoring_source(
        &mut self,
        path: &Path,
        maximum: usize,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<usize, ContractError> {
        phase(ledger, control, |ledger, control| {
            let admission = self.authoring_admission(path, maximum, ledger, control);
            checkpoint(ledger, control)?;
            admission?;
            Err(ContractError::Invalid)
        })
    }
    /// Count all failed/repeated attempts and precharge complete actual path comparisons.
    fn authoring_admission(
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
    fn authoring_lease(
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
    fn authoring_original(
        &mut self,
        path: &Path,
        role: CaptureRole,
        maximum: usize,
        components: usize,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<usize, ContractError> {
        ledger.visits(self.entries.len())?;
        let comparison = self.entries.iter().try_fold(0usize, |n, e| {
            n.checked_add(e.path.as_os_str().len())
                .and_then(|n| n.checked_add(path.as_os_str().len()))
                .ok_or(ContractError::Capacity)
        })?;
        ledger.bytes(comparison)?;
        if self.entries.iter().any(|entry| entry.path == path) {
            return Err(ContractError::Binding);
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
        let lease = self.authoring_lease(path, role, maximum.min(remaining), ledger, control)?;
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
    /// Verify whole actual N/S/U membership, operation registries and native original identity.
    pub(in crate::review) fn verify_authoring_cohort(
        &self,
        pending: &PendingAuthoringCohort,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        phase(ledger, control, |ledger, control| {
            let state = self.authoring.as_ref().ok_or(ContractError::Binding)?;
            ledger.visits(self.entries.len() + state.uses.len() + 1)?;
            if self.lifecycle.is_some()
                || self.supersession.is_some()
                || state.locator != Some(pending.locator().0)
                || self.proof.captured_original_generations() != self.entries.len()
                || state.uses.len() != pending.members().len()
                || !(6..=100).contains(&state.uses.len())
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
                self.verify_authoring_member(actual, expected, ledger, control)?;
            }
            let expected_aux = if state.operation == AuthoringOperation::Init { 2 } else { 1 };
            ledger
                .visits(state.auxiliaries.len().checked_mul(2).ok_or(ContractError::Capacity)?)?;
            ledger.bytes(
                state
                    .auxiliaries
                    .len()
                    .checked_mul(2 * std::mem::size_of::<usize>())
                    .ok_or(ContractError::Capacity)?,
            )?;
            if state.auxiliaries.len() != expected_aux
                || state.auxiliaries.iter().filter(|i| Some(**i) == state.locator).count() != 1
                || (state.operation == AuthoringOperation::Init
                    && state.auxiliaries.iter().filter(|i| Some(**i) == state.policy).count() != 1)
            {
                return Err(ContractError::Binding);
            }
            let slots = self.entries.len();
            ledger.derived(slots)?;
            ledger.bytes(slots)?;
            let mut marked = Vec::new();
            marked.try_reserve_exact(slots).map_err(|_| ledger.capacity())?;
            marked.resize(slots, false);
            marked[pending.locator().0] = true;
            for row in &state.uses {
                checkpoint(ledger, control)?;
                ledger.visits(1)?;
                marked[row.index] = true;
            }
            match state.operation {
                AuthoringOperation::Init => {
                    if self.queue_original.is_some() || !self.response_originals.is_empty() {
                        return Err(ContractError::Binding);
                    }
                    let policy = state.policy.ok_or(ContractError::Binding)?;
                    let entry = self.entries.get(policy).ok_or(ContractError::Binding)?;
                    if entry.pool != Pool::Auxiliary
                        || entry.role != CaptureRole::ReviewPrivateConfig
                        || marked[policy]
                    {
                        return Err(ContractError::Binding);
                    }
                    marked[policy] = true;
                }
                AuthoringOperation::Merge | AuthoringOperation::Status => {
                    if state.policy.is_some() {
                        return Err(ContractError::Binding);
                    }
                    let queue = self.queue_original.ok_or(ContractError::Binding)?;
                    let entry = self.entries.get(queue).ok_or(ContractError::Binding)?;
                    if entry.pool != Pool::Queue
                        || entry.role != CaptureRole::ReviewQueue
                        || marked[queue]
                    {
                        return Err(ContractError::Binding);
                    }
                    marked[queue] = true;
                    for &index in &self.response_originals {
                        checkpoint(ledger, control)?;
                        ledger.visits(1)?;
                        ledger.bytes(std::mem::size_of::<usize>() + 1)?;
                        let entry = self.entries.get(index).ok_or(ContractError::Binding)?;
                        if entry.pool != Pool::Response || entry.role != CaptureRole::ReviewResponse
                        {
                            return Err(ContractError::Binding);
                        }
                        marked[index] = true;
                    }
                    self.bind_review_originals(queue, &self.response_originals, ledger, control)?;
                }
            }
            self.verify_marked_entries(&marked, ledger, control)?;
            Ok(())
        })
    }
    /// Require every genuine actual Entry to be marked, retaining complete final scan charges.
    fn verify_marked_entries(
        &self,
        marked: &[bool],
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        for (index, entry) in self.entries.iter().enumerate() {
            checkpoint(ledger, control)?;
            ledger.visits(1)?;
            ledger.bytes(1)?;
            if !marked[index] || entry.pool == Pool::Recorded {
                return Err(ContractError::Binding);
            }
        }
        Ok(())
    }
    /// Compare actual purpose, full captured path and exact real lease allocation.
    fn verify_authoring_member(
        &self,
        actual: &AuthoringUse,
        expected: &AuthoringMember,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        checkpoint(ledger, control)?;
        ledger.visits(1)?;
        let entry = self.entries.get(actual.index).ok_or(ContractError::Binding)?;
        if actual.index != expected.index()
            || actual.purpose != expected.purpose()
            || entry.pool != Pool::Source
            || entry.role != CaptureRole::AuthoringArtifactOriginal
            || !self.same_original(actual.index, expected.lease())?
        {
            return Err(ContractError::Binding);
        }
        let extent = entry
            .path
            .as_os_str()
            .len()
            .checked_add(expected.path().as_os_str().len())
            .ok_or_else(|| ledger.capacity())?;
        ledger.bytes(extent)?;
        if entry.path != expected.path() {
            return Err(ContractError::Binding);
        }
        checkpoint(ledger, control)
    }
}
/// Borrowed whole actual operation registrations from a receiver-created owner only.
/// This view cannot be manufactured from caller indices, lists or copied bytes.
pub(crate) struct RegisteredAuthoringOriginals<'a> {
    /// Sole held physical owner.
    held: &'a HeldReviewInputs,
    /// Closed operation selected before any input read.
    operation: AuthoringOperation,
    /// Actual optional init-policy Entry, internally selected.
    policy: Option<usize>,
}
impl<'a> RegisteredAuthoringOriginals<'a> {
    /// Borrow the closed genuine operation purpose.
    pub(crate) fn operation(&self) -> AuthoringOperation {
        self.operation
    }
    /// Borrow actual init-policy raw bytes only when this operation owns it.
    pub(crate) fn policy_raw(&self) -> Option<&'a [u8]> {
        self.policy.map(|i| self.held.entries[i].lease.bytes())
    }
    /// Borrow the sole actual input Queue; init has none.
    pub(crate) fn queue_raw(&self) -> Option<&'a [u8]> {
        self.held.queue_original.map(|i| self.held.entries[i].lease.bytes())
    }
    /// Borrow every registered response occurrence in actual order, including repeats.
    pub(crate) fn responses(&self) -> impl ExactSizeIterator<Item = &'a [u8]> + 'a {
        let held = self.held;
        held.response_originals.iter().map(move |i| held.entries[*i].lease.bytes())
    }
}
impl HeldReviewInputs {
    /// Borrow inert actual registries; this view alone confers no sealed/native capability.
    /// The genuine current owner invokes this only after complete private whole-U verification.
    pub(in crate::review) fn authoring_originals(
        &self,
    ) -> Result<RegisteredAuthoringOriginals<'_>, ContractError> {
        let state = self.authoring.as_ref().ok_or(ContractError::Binding)?;
        Ok(RegisteredAuthoringOriginals {
            held: self,
            operation: state.operation,
            policy: state.policy,
        })
    }
}

/// Repeat only this internally issued actual Merge/Status Queue/Response cohort.
impl RegisteredAuthoringOriginals<'_> {
    /// No supplied indices/list can create or narrow actual review registrations.
    /// The full native receiver/physical closure remains separately required.
    pub(crate) fn verify_review_originals(
        &self,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        phase(ledger, control, |ledger, control| {
            ledger.visits(1)?;
            if !matches!(self.operation, AuthoringOperation::Merge | AuthoringOperation::Status)
                || self.policy.is_some()
            {
                return Err(ContractError::Binding);
            }
            let queue = self.held.queue_original.ok_or(ContractError::Binding)?;
            self.held.bind_review_originals(queue, &self.held.response_originals, ledger, control)
        })
    }
}
