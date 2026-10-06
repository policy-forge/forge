//! Explicit direct lineage command retaining genuine native owners through publication.
//! The old queue stays historical; no predecessor response is read or transferred.

use super::{CommandError, PendingNative, owned, prepare_native, publish};
use crate::review::capture::supersession::{AuxiliaryPurpose, QueuePurpose, seal_complete_union};
use crate::review::capture::{ReviewCapture, ReviewControl};
use crate::review::decode::{self, ContractError, ContractLedger, Decoded};
use crate::review::{encode, impact_capture, links, validate};
use crate::workspace::preparation::WorkControl;
use std::path::Path;

/// Complete explicit inputs; paths are confined and all identities/times remain assertions.
pub(crate) struct SupersedeOptions<'a> {
    /// Actual normalized project root containing every retained original and destination.
    pub(crate) project_root: &'a Path,
    /// Complete historical queue original; neither native approval nor old responses are revived.
    pub(crate) old_queue: &'a Path,
    /// Complete new queue original bound to the actual current native adapter.
    pub(crate) new_queue: &'a Path,
    /// Exact current domain source locator, using the existing native factories.
    pub(crate) sources: &'a Path,
    /// Closed seven-field private Impact locator and explicit complete current report.
    pub(crate) impact: &'a Path,
    /// Complete closed explicit old/new link pairs and requested current native finding IDs.
    pub(crate) links: &'a Path,
    /// Explicit canonical nonnil companion UUID; no random or inferred lineage ID.
    pub(crate) supersession_id: &'a str,
    /// Explicit asserted canonical UTC seconds, no earlier than either queue declaration.
    pub(crate) created_at: &'a str,
    /// Confined new immutable companion destination; existing destinations are refused.
    pub(crate) output: &'a Path,
}

/// Accept one caller/control/ledger before IO, then retain all originals through the native fence.
pub(crate) fn supersede(
    options: &SupersedeOptions<'_>,
    caller: &mut dyn WorkControl,
) -> Result<(), CommandError> {
    let mut control = ReviewControl::accept(caller);
    let mut ledger = ContractLedger::default();
    run(options, &mut ledger, &mut control)
}

/// Fence both successful and failed real Queue decoding before propagating its result.
/// This inert decoder grants no held/native authority and preserves the first ledger stop.
fn decoded_queue<'raw>(
    raw: &'raw [u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Decoded<'raw, crate::review::wire::QueueDocument>, ContractError> {
    let result = decode::decode_queue(raw, ledger, control);
    ledger.checkpoint(control)?;
    result
}

/// Preserve the accepted original ledger across every success or refusal stage.
fn run(
    options: &SupersedeOptions<'_>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), CommandError> {
    ledger.checkpoint(control)?;
    let extent = options
        .supersession_id
        .len()
        .checked_add(options.created_at.len())
        .ok_or_else(|| ledger.capacity())?;
    ledger.bytes(extent)?;
    if options.supersession_id.len() != 36 || options.created_at.len() != 20 {
        return Err(ContractError::Invalid.into());
    }
    validate::uuid(options.supersession_id)?;
    validate::time(options.created_at)?;
    let mut capture =
        ReviewCapture::new_supersession(options.project_root, options.output, ledger, control)?;
    for (path, purpose) in [
        (options.old_queue, QueuePurpose::HistoricalOld),
        (options.new_queue, QueuePurpose::CurrentNew),
    ] {
        capture.required_supersession_queue(path, purpose, ledger, control)?;
    }
    for (path, purpose) in [
        (options.sources, AuxiliaryPurpose::NewNativeLocator),
        (options.impact, AuxiliaryPurpose::ImpactLocator),
        (options.links, AuxiliaryPurpose::Links),
    ] {
        capture.required_supersession_auxiliary(path, purpose, ledger, control)?;
    }
    let locator = capture.supersession_auxiliary_original(AuxiliaryPurpose::NewNativeLocator)?;
    let pending = match prepare_native(&mut capture, locator, ledger, control)? {
        PendingNative::Mapping(value) => value.into_supersession_pending(),
        PendingNative::Applicability(value) => (*value).into_supersession_pending(),
    };
    let impact = impact_capture::prepare(&mut capture, ledger, control)?;
    let held =
        seal_complete_union(owned(capture, ledger, control)?, pending, impact, ledger, control)?;
    let old = decoded_queue(
        held.held_inputs().bytes(held.queue_original(QueuePurpose::HistoricalOld)?)?,
        ledger,
        control,
    )?;
    let new = decoded_queue(
        held.held_inputs().bytes(held.queue_original(QueuePurpose::CurrentNew)?)?,
        ledger,
        control,
    )?;
    let request = links::decode_links(
        held.held_inputs().bytes(held.auxiliary_original(AuxiliaryPurpose::Links)?)?,
        ledger,
        control,
    )?;
    let bound = super::supersession_binding::prepare(&held, &old, &new, &request, ledger, control)?;
    let document = super::supersession_projection::build(&bound, options, ledger, control)?;
    let output = encode::supersession(&document, ledger, control)?;
    let recorded = crate::review::supersession_decode::decode(&output, ledger, control)?;
    crate::review::supersession_decode::bind_queues(&recorded, &old, &new, ledger, control)?;
    bound.verify_inputs(ledger, control)?;
    publish(options.project_root, options.output, &output, ledger, control, |ledger, control| {
        bound.verify_inputs(ledger, control)
    })
}

#[cfg(test)]
#[path = "supersession_queue_decode_tests.rs"]
/// Actual Queue decoder failure chronology on one original accepted control and ledger.
mod queue_decode_tests;
