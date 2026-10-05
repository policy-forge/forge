//! Ordinary Lifecycle response declarations from the actual Queue and rationale.
//! This command binds recorded pins and asserted eligibility, and grants no native
//! currentness, reviewer authority, quorum or Lifecycle transition.

use super::{command_phase, publish};
use crate::evidence_capture::CaptureRole;
use crate::review::capture::{HeldReviewInputs, Pool, ReviewCapture, ReviewControl};
use crate::review::chain::{compare, reserved};
use crate::review::chain_v2::visit;
use crate::review::commands::{CommandError, RespondOptions};
use crate::review::decode::{ContractError, ContractLedger};
use crate::review::decode_v2::{self, DecodedV2, phase};
use crate::review::encode_v2;
use crate::review::wire_v2::{
    IDENTITY_DISCLAIMER, QueueDocumentV2, ResponseDocumentV2, SourcePinV2, SupersessionReference,
};
use crate::workspace::preparation::WorkControl;

/// Publish one new private assertion from complete captured ordinary Queue/rationale inputs.
/// One original operation ledger/control remains accepted through the final output fence.
pub(crate) fn respond(
    options: &RespondOptions<'_>,
    caller: &mut dyn WorkControl,
) -> Result<(), CommandError> {
    let mut control = ReviewControl::accept(caller);
    let mut ledger = ContractLedger::default();
    command_phase(&mut ledger, &mut control, |ledger, control| {
        let mut capture =
            ReviewCapture::new(options.project_root, &[options.output], ledger, control)?;
        let queue_index = capture.required(
            options.queue,
            CaptureRole::ReviewQueue,
            Pool::Queue,
            10_485_760,
            ledger,
            control,
        )?;
        let rationale_index = capture.required(
            options.rationale_file,
            CaptureRole::ReviewPrivateConfig,
            Pool::Auxiliary,
            8_192,
            ledger,
            control,
        )?;
        let held = phase(ledger, control, |ledger, _control| {
            ledger.derived(std::mem::size_of::<HeldReviewInputs>())?;
            Ok(capture.finish())
        })?;
        let queue = decode_v2::decode_queue(held.bytes(queue_index)?, ledger, control)?;
        let response =
            build_response(options, &queue, held.bytes(rationale_index)?, ledger, control)?;
        let bytes = encode_v2::exchange::response(&response, ledger, control)?;
        let encoded = decode_v2::decode_response(&bytes, ledger, control)?;
        decode_v2::bind_response(&queue, &encoded, ledger, control)?;
        held.verify_inputs(ledger, control)?;
        publish(options.project_root, options.output, &bytes, ledger, control, |ledger, control| {
            held.verify_inputs(ledger, control)
        })
    })
}

/// Copy one whole admitted bounded string; retained payload/work precede allocation.
fn copied(
    value: &str,
    maximum: usize,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<String, ContractError> {
    visit(ledger, control)?;
    ledger.bytes(value.len())?;
    if value.len() > maximum {
        return Err(ContractError::Invalid);
    }
    ledger.derived(
        value.len().checked_add(std::mem::size_of::<String>()).ok_or(ContractError::Capacity)?,
    )?;
    let mut result = String::new();
    result.try_reserve_exact(value.len()).map_err(|_| ContractError::Capacity)?;
    result.push_str(value);
    Ok(result)
}

/// Preserve every exact Queue pin field and nullable native identity in complete order.
/// Recorded pin equality remains ordinary data rather than a native original proof.
fn pins(
    input: &[SourcePinV2],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<SourcePinV2>, ContractError> {
    let mut result = reserved(input.len(), ledger)?;
    for pin in input {
        visit(ledger, control)?;
        ledger.bytes(std::mem::size_of::<SourcePinV2>())?;
        result.push(SourcePinV2 {
            artifact_key: copied(&pin.artifact_key, 128, ledger, control)?,
            kind: pin.kind,
            raw_sha256: copied(&pin.raw_sha256, 64, ledger, control)?,
            byte_length: pin.byte_length,
            schema_identity: pin
                .schema_identity
                .as_deref()
                .map(|value| copied(value, 128, ledger, control))
                .transpose()?,
            validation_profile: copied(&pin.validation_profile, 128, ledger, control)?,
            native_model: pin.native_model,
            native_root_uuid: pin
                .native_root_uuid
                .as_deref()
                .map(|value| copied(value, 128, ledger, control))
                .transpose()?,
        });
    }
    Ok(result)
}

/// Derive all queue-correlated fields from its actual strict decoder and whole UTF-8 rationale.
/// Response identity, asserted reviewer/disposition/time and supersession are explicit inputs.
fn build_response(
    options: &RespondOptions<'_>,
    queue: &DecodedV2<'_, QueueDocumentV2>,
    rationale: &[u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<ResponseDocumentV2, ContractError> {
    phase(ledger, control, |ledger, control| {
        if rationale.len() > 8_192 {
            return Err(ledger.capacity());
        }
        ledger.bytes(rationale.len())?;
        let rationale = std::str::from_utf8(rationale).map_err(|_| ContractError::Invalid)?;
        ledger.bytes(options.item_key.len())?;
        if options.item_key.len() > 128 {
            return Err(ContractError::Invalid);
        }
        let document = queue.document();
        let mut selected = None;
        for item in &document.items {
            visit(ledger, control)?;
            if compare(&item.key, options.item_key, ledger)?.is_eq() {
                selected = Some(item);
            }
        }
        let item = selected.ok_or(ContractError::Binding)?;
        ledger.bytes(std::mem::size_of::<ResponseDocumentV2>())?;
        ledger.derived(std::mem::size_of::<ResponseDocumentV2>())?;
        let supersedes = options
            .supersedes
            .as_ref()
            .map(|prior| -> Result<SupersessionReference, ContractError> {
                ledger.bytes(std::mem::size_of::<SupersessionReference>())?;
                ledger.derived(std::mem::size_of::<SupersessionReference>())?;
                Ok(SupersessionReference {
                    response_id: copied(prior.response_id, 36, ledger, control)?,
                    raw_sha256: copied(prior.raw_sha256, 64, ledger, control)?,
                })
            })
            .transpose()?;
        let response = ResponseDocumentV2 {
            schema_version: copied("forge.review-response/2", 128, ledger, control)?,
            identity_disclaimer: copied(IDENTITY_DISCLAIMER, 128, ledger, control)?,
            response_id: copied(options.response_id, 36, ledger, control)?,
            queue_id: copied(&document.queue_id, 36, ledger, control)?,
            queue_raw_sha256: copied(queue.raw_sha256(), 64, ledger, control)?,
            item_key: copied(&item.key, 128, ledger, control)?,
            item_id: copied(&item.item_id, 36, ledger, control)?,
            domain: item.domain,
            adapter_version: copied(&item.adapter_version, 128, ledger, control)?,
            requested_action: item.requested_action,
            source_pins: pins(&document.source_pins, ledger, control)?,
            subject_sha256: copied(&item.subject_sha256, 64, ledger, control)?,
            context_sha256: copied(&item.context_sha256, 64, ledger, control)?,
            policy_sha256: copied(&item.policy_sha256, 64, ledger, control)?,
            reviewer_key: copied(options.reviewer_key, 128, ledger, control)?,
            reviewer_role: copied(options.reviewer_role, 128, ledger, control)?,
            disposition: options.disposition,
            responded_at: copied(options.responded_at, 20, ledger, control)?,
            rationale: copied(rationale, 8_192, ledger, control)?,
            abstention_reason: options
                .abstention_reason
                .map(|value| copied(value, 128, ledger, control))
                .transpose()?,
            proposed_edit: (),
            supersedes,
        };
        decode_v2::response(&response, ledger, control)?;
        Ok(response)
    })
}
