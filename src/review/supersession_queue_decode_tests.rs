//! Actual failed Queue decoder phases preserve post-checkpoint and first-stop chronology.
//! Plain invalid originals cannot issue native/capture authority or reach publication.

use super::*;
use crate::workspace::preparation::{Interruption, ProgressUpdate, Stage, WorkError, WorkResult};
use std::cell::RefCell;
use std::rc::Rc;

/// Observe real checkpoints while the original accepted caller stays borrowed.
#[derive(Default)]
struct State {
    /// Monotonic actual caller invocation count, never reset within a case.
    calls: usize,
    /// Actual global checkpoint selected after an observed ordinary failed phase.
    selected: Option<usize>,
    /// Select an actual Failed control result instead of Interrupted.
    failed: bool,
    /// Whether that authentic selected checkpoint was reached.
    fired: bool,
    /// The real first interruption exposed by the actual caller.
    interrupted: Option<Interruption>,
}

/// Actual `WorkControl` with shared observation only; owns no clock or native data.
struct Stop(Rc<RefCell<State>>);

impl WorkControl for Stop {
    /// Fail only at the selected actual boundary and preserve a real caller interruption.
    fn checkpoint(&mut self, stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
        assert_eq!(stage, Stage::PrepareDomain);
        let mut observation = self.0.borrow_mut();
        observation.calls += 1;
        if observation.selected == Some(observation.calls) {
            observation.fired = true;
            if observation.failed {
                return Err(WorkError::Failed(crate::workspace::contract::Error::invalid()));
            }
            observation.interrupted = Some(Interruption::CancelRequested);
            return Err(WorkError::Interrupted(Interruption::CancelRequested));
        }
        Ok(())
    }
    /// Expose the same real interruption to the maintained accepted owner.
    fn interruption(&self) -> Option<Interruption> {
        self.0.borrow().interrupted
    }
}

/// Invalid UTF-8, syntax and closed-schema phases cannot bypass a later caller stop.
#[test]
fn real_queue_decode_failure_fences_preserve_actual_interrupted_and_failed_controls() {
    for raw in [b"\xff".as_slice(), b"{".as_slice(), b"{}".as_slice()] {
        for failed in [false, true] {
            let state = Rc::new(RefCell::new(State::default()));
            let mut caller = Stop(Rc::clone(&state));
            let mut control = ReviewControl::accept(&mut caller);
            let mut ledger = ContractLedger::default();
            let before = state.borrow().calls;
            assert!(matches!(
                decoded_queue(raw, &mut ledger, &mut control),
                Err(ContractError::Invalid)
            ));
            assert_eq!(ledger.bound(|_| Ok(())), Ok(()));
            let observed = state.borrow().calls;
            assert!(observed - before >= 2, "actual entry and post-failed phase reached");
            let selected = observed + (observed - before);
            {
                let mut state = state.borrow_mut();
                state.selected = Some(selected);
                state.failed = failed;
            }
            let expected = if failed {
                ContractError::ControlFailed
            } else {
                ContractError::Interrupted(Interruption::CancelRequested)
            };
            assert!(
                matches!(decoded_queue(raw,&mut ledger,&mut control),Err(error) if error==expected)
            );
            assert!(state.borrow().fired);
            assert_eq!(state.borrow().calls, selected);
            assert_eq!(ledger.bound::<()>(|_| Err(ContractError::Invalid)), Err(expected));
            assert!(
                matches!(decoded_queue(raw,&mut ledger,&mut control),Err(error) if error==expected)
            );
            assert_eq!(ledger.checkpoint(&mut control), Err(expected));
            assert_eq!(state.borrow().calls, selected);
        }
    }
}

/// A genuine first derived-capacity refusal bypasses later decoder and caller phases.
#[test]
fn real_queue_decode_preserves_original_capacity_over_later_actual_control_failure() {
    for failed in [false, true] {
        let state = Rc::new(RefCell::new(State::default()));
        let mut caller = Stop(Rc::clone(&state));
        let mut control = ReviewControl::accept(&mut caller);
        let mut ledger = ContractLedger::default();
        assert!(matches!(
            decoded_queue(b"{", &mut ledger, &mut control),
            Err(ContractError::Invalid)
        ));
        assert_eq!(ledger.derived(33_554_433), Err(ContractError::Capacity));
        let selected = state.borrow().calls + 1;
        {
            let mut state = state.borrow_mut();
            state.selected = Some(selected);
            state.failed = failed;
        }
        let later = control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged);
        if failed {
            assert!(matches!(later, Err(WorkError::Failed(_))));
        } else {
            assert!(matches!(later, Err(WorkError::Interrupted(Interruption::CancelRequested))));
        }
        assert!(state.borrow().fired);
        assert!(matches!(
            decoded_queue(b"{", &mut ledger, &mut control),
            Err(ContractError::Capacity)
        ));
        assert_eq!(ledger.checkpoint(&mut control), Err(ContractError::Capacity));
        assert_eq!(state.borrow().calls, selected);
    }
}
