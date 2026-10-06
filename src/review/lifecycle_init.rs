//! New Lifecycle review queues derived from the complete actual native closure.
//! Private policy contains asserted seats and assignments; source/currentness facts
//! come solely from the held receiver and remain borrowed through publication.

use super::{command_phase, publish};
use crate::evidence_capture::CaptureRole;
use crate::review::capture::{Pool, ReviewCapture, ReviewControl};
use crate::review::commands::{CommandError, InitOptions};
use crate::review::decode::ContractLedger;
use crate::review::decode_v2::{self, phase};
use crate::review::{encode_v2, lifecycle_capture, lifecycle_queue_export};
use crate::workspace::preparation::WorkControl;

/// Capture complete actual native and private policy inputs, then publish one new queue.
/// Generated output bytes are never registered as a captured Queue input original.
pub(crate) fn init(
    options: &InitOptions<'_>,
    caller: &mut dyn WorkControl,
) -> Result<(), CommandError> {
    let mut control = ReviewControl::accept(caller);
    let mut ledger = ContractLedger::default();
    command_phase(&mut ledger, &mut control, |ledger, control| {
        let (closure, policy_index) = phase(ledger, control, |ledger, control| {
            let mut capture = ReviewCapture::new_lifecycle(
                options.project_root,
                &[options.output],
                ledger,
                control,
            )?;
            let policy_index = capture.required(
                options.policy,
                CaptureRole::ReviewPrivateConfig,
                Pool::Auxiliary,
                1_048_576,
                ledger,
                control,
            )?;
            let native =
                lifecycle_capture::read_pending(&mut capture, options.sources, ledger, control)?;
            let closure = native.finish_and_seal(capture, ledger, control)?;
            ledger.bytes(std::mem::size_of::<usize>())?;
            Ok((closure, policy_index))
        })?;
        let prepared = lifecycle_queue_export::prepare(
            &closure,
            closure.held_inputs().bytes(policy_index)?,
            options.queue_id,
            options.created_at,
            ledger,
            control,
        )?;
        let bytes = encode_v2::exchange::queue(prepared.document(), ledger, control)?;
        let encoded = decode_v2::decode_queue(&bytes, ledger, control)?;
        prepared.bind_output(&encoded, ledger, control)?;
        publish(options.project_root, options.output, &bytes, ledger, control, |ledger, control| {
            prepared.closure().verify_inputs(ledger, control)
        })
    })
}
