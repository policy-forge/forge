//! Plain authoring utilities routed through the same original review ledger/control.
//! A stack borrow shares admission and actual forwarding callbacks without a second owner.

use super::decode::{ContractError, ContractLedger};
use super::legacy_work::BorrowedPreparationError;
use crate::authoring::borrowed::{
    self as native, AuthoringCharge, BorrowedAuthoringError, BorrowedAuthoringInputs,
    PreparedAuthoringFacts,
};
use crate::authoring::manifest::AuthorProject;
use crate::authoring::model::AuthoringPlan;
use crate::workspace::preparation::{
    Interruption, ProgressUpdate, Stage, WorkControl, WorkError, WorkResult,
};
use std::cell::RefCell;
use std::path::{Path, PathBuf};

/// Decode complete project declarations; ordinary native failures retain the actual postfence.
pub(super) fn discover(
    raw: &[u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<AuthorProject, ContractError> {
    run(ledger, control, |forwarded, admit| {
        native::discover_project(raw, forwarded, &mut |c| admit(c)).map_err(native_error)
    })
    .map_err(BorrowedPreparationError::into_contract)
}
/// Prepare the full native App report and complete authoring plan once from genuine originals.
pub(super) fn prepare(
    inputs: BorrowedAuthoringInputs<'_>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<PreparedAuthoringFacts, ContractError> {
    run(ledger, control, |forwarded, admit| {
        native::prepare(inputs, forwarded, &mut |c| admit(c)).map_err(native_error)
    })
    .map_err(BorrowedPreparationError::into_contract)
}
/// Resolve a native declared dependency with the maintained leading-parent rule and original fences.
pub(super) fn route(
    base: &Path,
    relative: &Path,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<PathBuf, ContractError> {
    run(ledger, control, |forwarded, admit| {
        native::resolve_review_dependency(base, relative, forwarded, &mut |c| admit(c))
            .map_err(native_error)
    })
    .map_err(BorrowedPreparationError::into_contract)
}
/// Admit and compare the complete stored plan with the actual regenerated private native result.
pub(super) fn stored(
    raw: &[u8],
    plan: &AuthoringPlan,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    run(ledger, control, |forwarded, admit| {
        native::compare_review_stored_plan(raw, plan, forwarded, &mut |c| admit(c))
            .map_err(native_error)
    })
    .map_err(BorrowedPreparationError::into_contract)
}
/// Observe exact original UUID spelling after genuine complete native preparation, without a proof cast.
pub(super) fn root(
    raw: &[u8],
    model: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<String, ContractError> {
    run(ledger, control, |forwarded, admit| {
        native::review_root_text(raw, model, forwarded, &mut |c| admit(c)).map_err(native_error)
    })
    .map_err(BorrowedPreparationError::into_contract)
}

/// Share one borrowed actual ledger across strictly sequential admission/control callbacks.
/// All cell guards end inside those callbacks before any native phase or original checkpoint.
/// The original exclusive ledger borrow is restored before interpreting the terminal result.
fn run<T>(
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
    operation: impl FnOnce(
        &mut dyn WorkControl,
        &mut dyn FnMut(AuthoringCharge) -> Result<(), ContractError>,
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

/// Preserve maintained applicability charge order and all command-wide logical caps.
fn interpret(ledger: &mut ContractLedger, charge: AuthoringCharge) -> Result<(), ContractError> {
    ledger.bound(|ledger| match charge {
        AuthoringCharge::Checkpoint => Ok(()),
        AuthoringCharge::Work { visits, byte_work, matching_steps } => {
            ledger.visits(visits)?;
            ledger.bytes(byte_work)?;
            ledger.matching(matching_steps)
        }
        AuthoringCharge::Reserve { logical_bytes } => ledger.derived(logical_bytes),
        AuthoringCharge::Capacity => Err(ledger.capacity()),
    })
}

/// Classify an actual caller work result without replacing its typed interruption reason.
fn work_error(error: &WorkError) -> ContractError {
    match error {
        WorkError::Interrupted(reason) => ContractError::Interrupted(*reason),
        WorkError::Failed(_) => ContractError::ControlFailed,
    }
}

/// Preserve complete private applicability domain detail and actual first caller failures.
fn native_error(error: BorrowedAuthoringError<ContractError>) -> BorrowedPreparationError {
    match error {
        BorrowedAuthoringError::Domain(error) => BorrowedPreparationError::Native(error),
        BorrowedAuthoringError::Admission(error) => BorrowedPreparationError::Contract(error),
        BorrowedAuthoringError::Work(error) => {
            BorrowedPreparationError::Contract(work_error(&error))
        }
        BorrowedAuthoringError::Capacity => {
            BorrowedPreparationError::Contract(ContractError::Capacity)
        }
    }
}
