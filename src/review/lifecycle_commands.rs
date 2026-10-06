//! Explicit Lifecycle current-review operations over one genuine input owner.
//! One accepted control and ledger survive capture, finalization, finite encoding
//! and the actual output sink. Native Lifecycle approvals are never changed here.

use std::io::Write;
use std::path::Path;

use super::capture::{Pool, ReviewCapture, ReviewControl};
use super::chain_v2::visit;
use super::commands::{CommandError, CurrentOptions};
use super::decode::{ContractError, ContractLedger};
use super::decode_v2::{self, phase};
use super::{
    encode_v2, lifecycle_binding, lifecycle_capture, lifecycle_finalize, merge_v2, validate,
};
use crate::evidence_capture::CaptureRole;
use crate::workspace::preparation::WorkControl;

/// Explicit current-review sink; native input borrows stay live through either output.
enum Destination<'a> {
    /// Actual reserved new local file, never an overwrite.
    File(&'a Path),
    /// Caller-owned writer, which may retain a partial result on failure.
    Writer(&'a mut dyn Write),
}

/// Merge every actual response occurrence and publish one new current recorded bundle.
pub(crate) fn merge(
    options: &CurrentOptions<'_>,
    output: &Path,
    caller: &mut dyn WorkControl,
) -> Result<(), CommandError> {
    current(options, Destination::File(output), caller)
}

/// Write a complete current record to the caller after the final full input fence.
/// Writer failure or a later stop can accompany bytes already emitted; no retry occurs.
pub(crate) fn status(
    options: &CurrentOptions<'_>,
    writer: &mut dyn Write,
    caller: &mut dyn WorkControl,
) -> Result<(), CommandError> {
    current(options, Destination::Writer(writer), caller)
}

/// Retain ordinary sink failure until the same original control's unconditional postfence.
/// Contract admission stops are bound immediately; Publication/Output remain ordinary data.
fn command_phase<T>(
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
    work: impl FnOnce(&mut ContractLedger, &mut dyn WorkControl) -> Result<T, CommandError>,
) -> Result<T, CommandError> {
    match phase(ledger, control, |ledger, control| match work(ledger, control) {
        Err(CommandError::Contract(error)) => Err(error),
        ordinary => Ok(ordinary),
    }) {
        Ok(ordinary) => ordinary,
        Err(error) => Err(error.into()),
    }
}

/// Preserve the sole accepted operation work owner and all native/output borrows.
fn current(
    options: &CurrentOptions<'_>,
    destination: Destination<'_>,
    caller: &mut dyn WorkControl,
) -> Result<(), CommandError> {
    let mut control = ReviewControl::accept(caller);
    let mut ledger = ContractLedger::default();
    command_phase(&mut ledger, &mut control, |ledger, control| {
        phase(ledger, control, |ledger, _control| {
            ledger.visits(1)?;
            ledger.bytes(options.as_of.len())?;
            // The maintained time validator formats at most one complete UTC timestamp.
            ledger.derived(84)?;
            validate::time(options.as_of)?;
            if options.responses.len() > 10_000 {
                return Err(ledger.capacity());
            }
            Ok(())
        })?;
        let output = match &destination {
            Destination::File(path) => Some(*path),
            Destination::Writer(_) => None,
        };
        let closure = capture_current(options, output, ledger, control)?;
        let binding = lifecycle_binding::prepare(&closure, ledger, control)?;
        let policy = merge_v2::prepare_policy(
            binding.queue(),
            binding.responses(),
            options.as_of,
            ledger,
            control,
        )?;
        let current = lifecycle_finalize::finalize(&binding, &policy, ledger, control)?;
        let bytes = encode_v2::dispositions(current.document(), ledger, control)?;
        let encoded = decode_v2::decode_dispositions(&bytes, ledger, control)?;
        decode_v2::bind_dispositions(binding.queue(), &encoded, ledger, control)?;
        current.verify_inputs(ledger, control)?;
        match destination {
            Destination::File(path) => {
                publish(options.project_root, path, &bytes, ledger, control, |ledger, control| {
                    current.verify_inputs(ledger, control)
                })
            }
            Destination::Writer(writer) => command_phase(ledger, control, |ledger, control| {
                ledger.bytes(bytes.len())?;
                current.verify_inputs(ledger, control)?;
                writer.write_all(&bytes).map_err(|_| CommandError::Output)
            }),
        }
    })
}

/// Register every queue/response/native occurrence before consuming the genuine capture.
/// Successful actual registration indices stay internal to the actual owner and binding.
fn capture_current(
    options: &CurrentOptions<'_>,
    output: Option<&Path>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<lifecycle_capture::ApprovedLifecycleClosure, ContractError> {
    phase(ledger, control, |ledger, control| {
        let mut capture =
            ReviewCapture::new_lifecycle(options.project_root, output.as_slice(), ledger, control)?;
        capture.required(
            options.queue,
            CaptureRole::ReviewQueue,
            Pool::Queue,
            10_485_760,
            ledger,
            control,
        )?;
        for path in options.responses {
            visit(ledger, control)?;
            capture.required(
                path.as_path(),
                CaptureRole::ReviewResponse,
                Pool::Response,
                1_048_576,
                ledger,
                control,
            )?;
        }
        let native =
            lifecycle_capture::read_pending(&mut capture, options.sources, ledger, control)?;
        native.finish_and_seal(capture, ledger, control)
    })
}

/// Repeat the true whole input fence at native no-replace publication's pre-rename hook.
/// Ordinary native IO failure and successful writes receive the same original postfence;
/// a late durability error or stop can leave a complete output at its new destination.
fn publish(
    root: &Path,
    output: &Path,
    bytes: &[u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
    mut fence: impl FnMut(&mut ContractLedger, &mut dyn WorkControl) -> Result<(), ContractError>,
) -> Result<(), CommandError> {
    command_phase(ledger, control, |ledger, control| {
        ledger.bytes(bytes.len())?;
        let mut cause = None;
        let ordinary =
            crate::authoring::output::publish_new_file_guarded(root, output, bytes, || {
                let checked = phase(ledger, control, |ledger, control| fence(ledger, control));
                match checked {
                    Ok(()) => Ok(()),
                    Err(error) => {
                        cause = Some(error);
                        Err(crate::error::ForgeError::Authoring(
                            "review output fence refused".into(),
                        ))
                    }
                }
            });
        if let Some(error) = cause {
            return Err(error.into());
        }
        ordinary.map_err(|_| CommandError::Publication)
    })
}

/// Ordinary /2 response capture and complete finite output using the same private sink fence.
#[path = "lifecycle_response.rs"]
mod response_command;
/// Publish an asserted Lifecycle response without issuing native currentness.
pub(crate) use response_command::respond;

/// Native-derived /2 init uses the actual held private request and guarded output sink.
#[path = "lifecycle_init.rs"]
mod init_command;
/// Publish a new Lifecycle queue from a complete actual Approved/current source closure.
pub(crate) use init_command::init;
