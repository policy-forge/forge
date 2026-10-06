//! Route plain borrowed legacy preparation through the existing review work owner.
//!
//! These wrappers borrow the caller's existing ledger and cooperative control. They
//! construct no capture, lease, currentness, cohort, Source or publication authority.
//! Only the genuine capture/factory caller can establish the complete original roster.
//! A private stack cell permits sequential native admission and control callbacks to
//! address the same ledger; it does not create a second owner or reset any counter.
//!
//! Native parser and engine calls remain synchronous. Checkpoints observe the original
//! accepted deadline before and after phases; they cannot preempt an in-progress call.

use std::cell::RefCell;

use crate::ForgeError;
use crate::applicability::legacy_borrowed::{
    self as applicability, ApplicabilityCharge, LegacyBorrowedError,
};
#[cfg(test)]
use crate::applicability::legacy_borrowed::{LegacyApplicabilityFacts, LegacyApplicabilityInputs};
#[cfg(test)]
use crate::applicability::model::ReportFilters;
use crate::framework::analysis::legacy_borrowed::{
    self as impact, ImpactCharge, ImpactInputs, LegacyImpactError, LegacyImpactFacts,
};
use crate::framework::model::ImpactFilters;
use crate::workspace::preparation::{
    Interruption, ProgressUpdate, Stage, WorkControl, WorkError, WorkResult,
};

use super::decode::{ContractError, ContractLedger};

/// Private preparation failure; full native prose must be minimized at the review boundary.
#[derive(Debug)]
pub(crate) enum BorrowedPreparationError {
    /// Maintained complete native diagnostic, without source or owner authority.
    Native(ForgeError),
    /// Exact original ledger admission failure or actual cooperative control stop.
    Contract(ContractError),
}

impl BorrowedPreparationError {
    /// Consume private native detail at the existing fixed review error boundary.
    /// Actual admission/control failures keep their precise typed classification.
    pub(crate) fn into_contract(self) -> ContractError {
        match self {
            Self::Native(error) => {
                drop(error);
                ContractError::Binding
            }
            Self::Contract(error) => error,
        }
    }
}

/// Complete private descriptor vocabulary understood by this caller-owned interpreter.
enum Charge {
    /// Existing borrowed applicability request, including nested native inventories.
    Applicability(ApplicabilityCharge),
    /// Borrowed complete legacy Impact request, with repeated Mapping projection growth.
    Impact(ImpactCharge),
}

/// Prepare complete plain legacy applicability data using the same invocation work owner.
///
/// The caller supplies and retains genuine complete originals separately. The result
/// is detached domain data, so this function cannot seal a source, approval or cohort.
///
/// # Errors
///
/// Preserve the first actual capacity/control stop already saved by the caller ledger.
/// Native success and domain refusal both pass the original final cooperative fence.
/// First callback admission/work failures return without a replacing later control probe.
#[cfg(test)]
pub(crate) fn prepare_applicability(
    inputs: LegacyApplicabilityInputs<'_>,
    filters: ReportFilters,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<LegacyApplicabilityFacts, BorrowedPreparationError> {
    run(ledger, control, |forwarded, admit| {
        applicability::prepare(inputs, filters, forwarded, &mut |charge| {
            admit(Charge::Applicability(charge))
        })
        .map_err(applicability_error)
    })
}

/// Prepare complete plain legacy Impact data, including its configured applicability roster.
///
/// Input declarations and their bytes confer no correspondence, alias, owner, currentness,
/// cohort or union proof. The genuine capture and extraction bridge is separate.
///
/// # Errors
///
/// Preserve the same first caller failure across all nested native preparations. A final
/// actual original-control fence precedes returning either complete facts or a native error.
pub(crate) fn prepare_impact(
    inputs: ImpactInputs<'_>,
    filters: ImpactFilters,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<LegacyImpactFacts, BorrowedPreparationError> {
    run(ledger, control, |forwarded, admit| {
        impact::prepare(inputs, filters, forwarded, &mut |charge| admit(Charge::Impact(charge)))
            .map_err(impact_error)
    })
}

/// Decode plain Impact declarations with the same actual caller ledger/control.
///
/// The returned manifest grants no IO or capture authority. Only the genuine reader
/// may resolve its declared routes on the retained actual capture session.
///
/// # Errors
///
/// Preserve actual first admission/work failures and native success/domain final fences.
pub(crate) fn discover_impact_manifest(
    bytes: &[u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<crate::framework::manifest::ImpactManifest, BorrowedPreparationError> {
    run(ledger, control, |forwarded, admit| {
        impact::discover_manifest(bytes, forwarded, &mut |charge| admit(Charge::Impact(charge)))
            .map_err(impact_error)
    })
}

/// Decode plain applicability declarations under the same unchanged work owner.
///
/// The native declaration's own base remains the genuine reader's responsibility;
/// historical policy hrefs and imports never become routes in this wrapper.
///
/// # Errors
///
/// Preserve actual first admission/work failures and native success/domain final fences.
pub(crate) fn discover_applicability_manifest(
    bytes: &[u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<crate::applicability::manifest::ApplicabilityManifest, BorrowedPreparationError> {
    run(ledger, control, |forwarded, admit| {
        applicability::discover_manifest(bytes, forwarded, &mut |charge| {
            admit(Charge::Applicability(charge))
        })
        .map_err(applicability_error)
    })
}

/// Share one borrowed actual ledger across strictly sequential admission/control callbacks.
/// All cell guards end inside those callbacks before any native phase or original checkpoint.
/// The original exclusive ledger borrow is restored before interpreting the terminal result.
fn run<T>(
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
    operation: impl FnOnce(
        &mut dyn WorkControl,
        &mut dyn FnMut(Charge) -> Result<(), ContractError>,
    ) -> Result<T, BorrowedPreparationError>,
) -> Result<T, BorrowedPreparationError> {
    ledger.checkpoint(control).map_err(BorrowedPreparationError::Contract)?;
    let result = {
        let shared = RefCell::new(&mut *ledger);
        let mut admit = |charge| {
            let mut borrowed = shared.borrow_mut();
            interpret(&mut borrowed, charge)
        };
        let mut forwarded = ForwardControl { caller: control, ledger: &shared };
        operation(&mut forwarded, &mut admit)
    };
    match result {
        Err(BorrowedPreparationError::Contract(error)) => {
            // The helper has already observed a concrete first callback/control error.
            // Consult the same sticky ledger without issuing a later replacement probe.
            ledger.bound(|_| Err(error)).map_err(BorrowedPreparationError::Contract)
        }
        native_result => {
            // Complete native facts and native Domain failures both require the actual
            // post-phase fence, even when the native helper already made its own probe.
            ledger.checkpoint(control).map_err(BorrowedPreparationError::Contract)?;
            native_result
        }
    }
}

/// Forward exact original cooperative results while latching real failures in the same ledger.
struct ForwardControl<'a, 'ledger> {
    /// Existing accepted caller control; never substituted, accepted again or reset.
    caller: &'a mut dyn WorkControl,
    /// Stack-only borrow of the existing command ledger; no independent counters exist.
    ledger: &'a RefCell<&'ledger mut ContractLedger>,
}

impl WorkControl for ForwardControl<'_, '_> {
    /// Record the actual observed `WorkError` before native code can classify its result.
    /// A saved ledger stop does not manufacture a `WorkError` or alter the caller's result.
    fn checkpoint(&mut self, stage: Stage, progress: ProgressUpdate) -> WorkResult<()> {
        let result = self.caller.checkpoint(stage, progress);
        if let Err(error) = &result {
            let failure = work_error(error);
            let mut ledger = self.ledger.borrow_mut();
            let _ = ledger.bound::<()>(|_| Err(failure));
        }
        result
    }

    /// Preserve the exact original caller's interruption state for its final owner fence.
    fn interruption(&self) -> Option<Interruption> {
        self.caller.interruption()
    }
}

/// Interpret a complete helper descriptor under the existing first-stop bound.
fn interpret(ledger: &mut ContractLedger, charge: Charge) -> Result<(), ContractError> {
    ledger.bound(|ledger| match charge {
        Charge::Applicability(charge) => applicability_charge(ledger, charge),
        Charge::Impact(charge) => match charge {
            ImpactCharge::Bytes(amount) => ledger.derived(amount),
            ImpactCharge::Rows(amount) => ledger.visits(amount),
            ImpactCharge::Work { nodes, string_bytes, comparisons } => {
                ledger.visits(nodes)?;
                ledger.bytes(string_bytes)?;
                ledger.matching(comparisons)
            }
            ImpactCharge::MappingProjection { logical_bytes, controls } => {
                ledger.derived(logical_bytes)?;
                ledger.visits(controls)
            }
            ImpactCharge::Applicability(charge) => applicability_charge(ledger, charge),
            ImpactCharge::Capacity => Err(ledger.capacity()),
        },
    })
}

/// Preserve maintained applicability charge order and all command-wide logical caps.
fn applicability_charge(
    ledger: &mut ContractLedger,
    charge: ApplicabilityCharge,
) -> Result<(), ContractError> {
    match charge {
        ApplicabilityCharge::Checkpoint => Ok(()),
        ApplicabilityCharge::Work { visits, byte_work, matching_steps } => {
            ledger.visits(visits)?;
            ledger.bytes(byte_work)?;
            ledger.matching(matching_steps)
        }
        ApplicabilityCharge::Reserve { logical_bytes } => ledger.derived(logical_bytes),
        ApplicabilityCharge::Capacity => Err(ledger.capacity()),
    }
}

/// Classify an actual caller work result without replacing its typed interruption reason.
fn work_error(error: &WorkError) -> ContractError {
    match error {
        WorkError::Interrupted(reason) => ContractError::Interrupted(*reason),
        WorkError::Failed(_) => ContractError::ControlFailed,
    }
}

/// Preserve complete private applicability domain detail and actual first caller failures.
fn applicability_error(error: LegacyBorrowedError<ContractError>) -> BorrowedPreparationError {
    match error {
        LegacyBorrowedError::Domain(error) => BorrowedPreparationError::Native(error),
        LegacyBorrowedError::Admission(error) => BorrowedPreparationError::Contract(error),
        LegacyBorrowedError::Work(error) => BorrowedPreparationError::Contract(work_error(&error)),
        LegacyBorrowedError::Capacity => {
            BorrowedPreparationError::Contract(ContractError::Capacity)
        }
    }
}

/// Preserve complete private Impact domain detail and actual first caller failures.
fn impact_error(error: LegacyImpactError<ContractError>) -> BorrowedPreparationError {
    match error {
        LegacyImpactError::Domain(error) => BorrowedPreparationError::Native(error),
        LegacyImpactError::Admission(error) => BorrowedPreparationError::Contract(error),
        LegacyImpactError::Work(error) => BorrowedPreparationError::Contract(work_error(&error)),
        LegacyImpactError::Capacity => BorrowedPreparationError::Contract(ContractError::Capacity),
    }
}

#[cfg(test)]
#[path = "legacy_work_tests.rs"]
/// Actual complete native parity and first-stop controls; synthetic fixtures grant no approval.
mod tests;
