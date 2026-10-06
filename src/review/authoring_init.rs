//! Native-derived whole-plan Init using the actual captured private policy.
use super::{command_phase, publish};
use crate::review::capture::authoring::AuthoringOperation;
use crate::review::capture::{ReviewCapture, ReviewControl};
use crate::review::commands::{CommandError, InitOptions};
use crate::review::decode::ContractLedger;
use crate::review::decode_v2::phase;
use crate::review::validate;
use crate::review::{authoring_capture, authoring_queue_export};
use crate::workspace::preparation::WorkControl;

/// Capture full native sources and the Init policy, then publish a new Queue.
/// The generated Queue is never registered as an input original.
pub(crate) fn init(
    options: &InitOptions<'_>,
    caller: &mut dyn WorkControl,
) -> Result<(), CommandError> {
    let mut control = ReviewControl::accept(caller);
    let mut ledger = ContractLedger::default();
    command_phase(&mut ledger, &mut control, |ledger, control| {
        phase(ledger, control, |ledger, _| {
            for value in [options.queue_id, options.created_at] {
                ledger.visits(1)?;
                let header_work = value.len().checked_mul(3).ok_or_else(|| ledger.capacity())?;
                ledger.bytes(header_work)?;
                ledger.derived(128)?;
            }
            validate::uuid(options.queue_id)?;
            validate::time(options.created_at)
        })?;
        let closure = phase(ledger, control, |ledger, control| {
            let mut capture = ReviewCapture::new_authoring(
                options.project_root,
                &[options.output],
                AuthoringOperation::Init,
                ledger,
                control,
            )?;
            let native =
                authoring_capture::read_pending(&mut capture, options.sources, ledger, control)?;
            capture.required_authoring_policy(options.policy, ledger, control)?;
            native.finish_and_seal(capture, ledger, control)
        })?;
        let prepared = authoring_queue_export::prepare(
            &closure,
            options.queue_id,
            options.created_at,
            ledger,
            control,
        )?;
        let encoded = prepared.encode(ledger, control)?;
        encoded.verify_inputs(ledger, control)?;
        publish(
            options.project_root,
            options.output,
            encoded.bytes(),
            ledger,
            control,
            |ledger, control| encoded.verify_inputs(ledger, control),
        )
    })
}
