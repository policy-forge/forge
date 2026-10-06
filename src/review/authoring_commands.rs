//! Explicit whole-plan Authoring review operations over genuine captured originals.
//! One accepted control and ledger survive native preparation, policy evaluation,
//! complete output readback and the actual sink. Review never promotes the native plan.

use std::io::Write;
use std::path::Path;

use super::capture::authoring::AuthoringOperation;
use super::capture::{Pool, ReviewCapture, ReviewControl};
use super::chain_v2::visit;
use super::commands::{CommandError, CurrentOptions};
use super::decode::{ContractError, ContractLedger};
use super::decode_v2::phase;
use super::{
    authoring_binding, authoring_capture, authoring_finalize, authoring_response_binding,
    decode_v3, encode_v3, merge_v3, validate,
};
use crate::evidence_capture::CaptureRole;
use crate::workspace::preparation::WorkControl;

/// Actual sink remains inside the invocation retaining its complete native owner.
enum Destination<'a> {
    /// Reserved new local file, never overwritten.
    File(&'a Path),
    /// Caller-owned stream which may retain partial bytes on failure.
    Writer(&'a mut dyn Write),
}

/// Merge every actual response occurrence and publish one new recorded bundle.
pub(crate) fn merge(
    options: &CurrentOptions<'_>,
    output: &Path,
    caller: &mut dyn WorkControl,
) -> Result<(), CommandError> {
    current(options, Destination::File(output), caller)
}

/// Write the complete current record after full native and original verification.
pub(crate) fn status(
    options: &CurrentOptions<'_>,
    writer: &mut dyn Write,
    caller: &mut dyn WorkControl,
) -> Result<(), CommandError> {
    current(options, Destination::Writer(writer), caller)
}

/// Retain ordinary sink errors through the same original unconditional postfence.
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

/// Preserve one accepted thirty-second controller and monotonic ledger through the sink.
fn current(
    options: &CurrentOptions<'_>,
    destination: Destination<'_>,
    caller: &mut dyn WorkControl,
) -> Result<(), CommandError> {
    let mut control = ReviewControl::accept(caller);
    let mut ledger = ContractLedger::default();
    command_phase(&mut ledger, &mut control, |ledger, control| {
        phase(ledger, control, |ledger, _| {
            ledger.visits(1)?;
            let time_work = options.as_of.len().checked_mul(3).ok_or_else(|| ledger.capacity())?;
            ledger.bytes(time_work)?;
            ledger.derived(128)?;
            validate::time(options.as_of)?;
            if options.responses.len() > 10_000 {
                return Err(ledger.capacity());
            }
            Ok(())
        })?;
        let (output, operation) = match &destination {
            Destination::File(path) => (Some(*path), AuthoringOperation::Merge),
            Destination::Writer(_) => (None, AuthoringOperation::Status),
        };
        let closure = capture_current(options, output, operation, ledger, control)?;
        let binding = authoring_binding::prepare(&closure, ledger, control)?;
        let policy = merge_v3::prepare_policy(
            binding.queue(),
            binding.responses(),
            options.as_of,
            ledger,
            control,
        )?;
        let current = authoring_finalize::finalize(&binding, &policy, ledger, control)?;
        let bytes = encode_v3::dispositions(current.document(), ledger, control)?;
        let readback = decode_v3::decode_dispositions(&bytes, ledger, control)?;
        encode_v3::compare_dispositions(current.document(), readback.document(), ledger, control)?;
        authoring_response_binding::bind_dispositions(binding.queue(), &readback, ledger, control)?;
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

/// Register the exact Queue and every positional Response before sealing full N/S/U.
fn capture_current(
    options: &CurrentOptions<'_>,
    output: Option<&Path>,
    operation: AuthoringOperation,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<authoring_capture::CurrentAuthoringPlanClosure, ContractError> {
    phase(ledger, control, |ledger, control| {
        let mut capture = ReviewCapture::new_authoring(
            options.project_root,
            output.as_slice(),
            operation,
            ledger,
            control,
        )?;
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
            authoring_capture::read_pending(&mut capture, options.sources, ledger, control)?;
        native.finish_and_seal(capture, ledger, control)
    })
}

/// Repeat true whole-input verification at actual native no-replace publication.
/// A late durability error or stop can accompany a complete new destination.
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
        let ordinary = crate::authoring::output::publish_new_file_guarded(
            root,
            output,
            bytes,
            || match phase(ledger, control, |ledger, control| fence(ledger, control)) {
                Ok(()) => Ok(()),
                Err(error) => {
                    cause = Some(error);
                    Err(crate::error::ForgeError::Authoring("review output fence refused".into()))
                }
            },
        );
        if let Some(error) = cause {
            return Err(error.into());
        }
        ordinary.map_err(|_| CommandError::Publication)
    })
}

/// Ordinary immutable Response/3 output from complete captured Queue/rationale bytes.
#[path = "authoring_response.rs"]
mod response_command;
/// Publish an asserted response without granting native currentness.
pub(crate) use response_command::respond;

/// Native-derived Init retains its private consuming encoded output owner.
#[path = "authoring_init.rs"]
mod init_command;
/// Publish a new whole-plan Queue from a genuine current native closure.
pub(crate) use init_command::init;
