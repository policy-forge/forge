// A-authored actual private factory; Root alone integrates and executes it.
//! Genuine Authoring currentness finalization from the sealed captured binding.
//! A pending policy result, copied bytes or recorded digest cannot issue this owner.
//! The complete input closure stays borrowed through encoding and publication;
//! asserted review quorum never updates a native Authoring plan or reviewer authority.

use super::authoring_binding::PendingAuthoringBinding;
use super::chain::{compare, reserved};
use super::chain_v3::visit;
use super::decode::{ContractError, ContractLedger};
use super::decode_v2::phase;
use super::decode_v3;
use super::hash_v3::{self, AuxiliaryOriginal, AuxiliaryPurposeV3, ResponseOriginal};
use super::merge_v3::{ItemPolicyFactsV3, PendingPolicyEvaluationV3, TentativeStateV3};
use super::wire::{StateCounts, UnmetSeat};
use super::wire_v3::{
    DispositionCounts, DispositionsDocumentV3, IDENTITY_DISCLAIMER, ItemDisposition, ItemState,
    MetSeat, RecordedCurrentness, RecordedResponse, SourcePinV3,
};
use crate::workspace::preparation::WorkControl;

/// Actual currentness-qualified record retaining the real sealed source/input borrow.
/// There is no Serialize, Deserialize, Clone, Debug or detached success constructor.
pub(crate) struct CurrentAuthoringDispositions<'a, 'native> {
    /// Internally issued native binding and its actual Queue/Response registrations.
    binding: &'a PendingAuthoringBinding<'native>,
    /// Fully admitted minimized wire record; its labels are inert after serialization.
    document: DispositionsDocumentV3,
}

impl CurrentAuthoringDispositions<'_, '_> {
    /// Borrow the finite typed record without issuing another native proof owner.
    pub(crate) fn document(&self) -> &DispositionsDocumentV3 {
        &self.document
    }

    /// Repeat the full original input and reserved-output fence on the same control.
    /// Root retains this owner through its final output callback; this is not an atomic snapshot.
    pub(crate) fn verify_inputs(
        &self,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        phase(ledger, control, |ledger, control| self.binding.verify_inputs(ledger, control))
    }
}

/// Finalize only a genuinely source-bound queue and the complete actual pending cohort.
/// No caller Rc, index list, recorded current label or native-success flag is accepted.
pub(crate) fn finalize<'a, 'native>(
    binding: &'a PendingAuthoringBinding<'native>,
    pending: &PendingPolicyEvaluationV3<'_>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<CurrentAuthoringDispositions<'a, 'native>, ContractError> {
    phase(ledger, control, |ledger, control| {
        phase(ledger, control, |ledger, control| {
            bind_originals(binding, pending, ledger, control)
        })?;
        let generation = generation(binding, pending, ledger, control)?;
        let document =
            document(pending, binding.closure().source_pins(), generation, ledger, control)?;
        decode_v3::dispositions(&document, ledger, control)?;
        phase(ledger, control, |ledger, control| binding.verify_inputs(ledger, control))?;
        ledger.derived(std::mem::size_of::<CurrentAuthoringDispositions<'_, '_>>())?;
        Ok(CurrentAuthoringDispositions { binding, document })
    })
}

/// Require the pending decoders to borrow every actual admitted input allocation in order.
/// The binding's registry is internally issued from this same opaque native closure.
fn bind_originals(
    binding: &PendingAuthoringBinding<'_>,
    pending: &PendingPolicyEvaluationV3<'_>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let actual = binding.originals();
    binding.verify_inputs(ledger, control)?;
    actual.verify_review_originals(ledger, control)?;
    visit(ledger, control)?;
    ledger.bytes(12 * std::mem::size_of::<usize>())?;
    let counts = pending.counts();
    let actual_responses = actual.responses();
    let files = actual_responses.len();
    if pending.originals().len() != files
        || binding.responses().len() != files
        || counts.response_files != files
        || counts.items != 1
        || pending.items().len() != 1
        || binding.queue().document().items.len() != 1
        || pending.responses().len() != counts.unique_responses
        || counts.unique_responses.checked_add(counts.exact_duplicates) != Some(files)
    {
        return Err(ContractError::Binding);
    }
    let queue = actual.queue_raw().ok_or(ContractError::Binding)?;
    bind_storage(pending.queue().raw(), queue, ledger)?;
    bind_storage(binding.queue().raw(), queue, ledger)?;
    for ((raw, original), bound) in
        actual_responses.zip(pending.originals()).zip(binding.responses())
    {
        visit(ledger, control)?;
        bind_storage(original.raw(), raw, ledger)?;
        bind_storage(bound.raw(), raw, ledger)?;
    }
    Ok(())
}

/// Check both pointer and full extent after the genuine role/cohort owner admission.
/// Equal copied bytes are deliberately refused; this predicate alone is not a file proof.
fn bind_storage(
    raw: &[u8],
    actual: &[u8],
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    ledger.visits(1)?;
    ledger.bytes(4 * std::mem::size_of::<usize>())?;
    if raw.len() != actual.len() || !std::ptr::eq(raw.as_ptr(), actual.as_ptr()) {
        return Err(ContractError::Binding);
    }
    Ok(())
}

/// Hash all real registered response occurrences, preserving order and exact duplicates.
/// The /3 domain-separated digest records the bound cohort; it is never a proof constructor.
fn generation(
    binding: &PendingAuthoringBinding<'_>,
    pending: &PendingPolicyEvaluationV3<'_>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<String, ContractError> {
    phase(ledger, control, |ledger, control| {
        let mut rows = reserved(pending.originals().len(), ledger)?;
        for original in pending.originals() {
            visit(ledger, control)?;
            ledger.bytes(std::mem::size_of::<ResponseOriginal<'_>>())?;
            rows.push(ResponseOriginal {
                response_id: &original.document().response_id,
                raw_sha256: original.raw_sha256(),
                byte_length: u64::try_from(original.raw().len()).map_err(|_| ledger.capacity())?,
            });
        }
        let locator = binding.closure().locator_raw();
        ledger.bytes(locator.len())?;
        ledger.derived(128)?;
        let locator_hash = phase(ledger, control, |_, _| Ok(crate::hashing::sha256_hex(locator)))?;
        ledger.derived(std::mem::size_of::<AuxiliaryOriginal<'_>>())?;
        let auxiliary = [AuxiliaryOriginal {
            purpose: AuxiliaryPurposeV3::Locator,
            raw_sha256: &locator_hash,
            byte_length: u64::try_from(locator.len()).map_err(|_| ledger.capacity())?,
        }];
        let native = super::authoring_binding::native_digest(binding.closure(), ledger, control)?;
        hash_v3::closure_generation(
            hash_v3::ClosureGenerationInputs {
                native: &native,
                pins: binding.closure().source_pins(),
                queue: pending.queue().raw_sha256(),
                queue_length: u64::try_from(pending.queue().raw().len())
                    .map_err(|_| ledger.capacity())?,
                aux: &auxiliary,
                responses: &rows,
            },
            ledger,
            control,
        )
    })
}

/// Copy one admitted redacted string; whole extent/work and retained payload precede growth.
fn text(
    value: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<String, ContractError> {
    visit(ledger, control)?;
    ledger.bytes(value.len())?;
    ledger.derived(value.len())?;
    let mut result = String::new();
    result.try_reserve_exact(value.len()).map_err(|_| ContractError::Capacity)?;
    result.push_str(value);
    Ok(result)
}

/// Copy all ordered redacted references without filtering history or dissent.
fn strings(
    values: &[String],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<String>, ContractError> {
    let mut result = reserved(values.len(), ledger)?;
    for value in values {
        ledger.bytes(std::mem::size_of::<String>())?;
        result.push(text(value, ledger, control)?);
    }
    Ok(result)
}

/// Copy a nullable admitted original identity without filling an absent native UUID.
fn optional(
    value: Option<&str>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Option<String>, ContractError> {
    value.map(|value| text(value, ledger, control)).transpose()
}

/// Copy the complete actual sealed source set, never a queue-supplied selected prefix.
fn source_pins(
    pins: &[SourcePinV3],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<SourcePinV3>, ContractError> {
    let mut result = reserved(pins.len(), ledger)?;
    for pin in pins {
        visit(ledger, control)?;
        ledger.bytes(std::mem::size_of::<SourcePinV3>())?;
        result.push(SourcePinV3 {
            artifact_key: text(&pin.artifact_key, ledger, control)?,
            kind: pin.kind,
            native_model: pin.native_model,
            validation_profile: text(&pin.validation_profile, ledger, control)?,
            native_root_uuid: optional(pin.native_root_uuid.as_deref(), ledger, control)?,
            raw_sha256: text(&pin.raw_sha256, ledger, control)?,
            byte_length: pin.byte_length,
            schema_identity: optional(pin.schema_identity.as_deref(), ledger, control)?,
        });
    }
    Ok(result)
}

/// Preserve every distinct original classification, disposition, asserted key/role/time and pin.
fn responses(
    rows: &[RecordedResponse],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<RecordedResponse>, ContractError> {
    let mut result = reserved(rows.len(), ledger)?;
    for row in rows {
        visit(ledger, control)?;
        ledger.bytes(std::mem::size_of::<RecordedResponse>())?;
        result.push(RecordedResponse {
            response_id: text(&row.response_id, ledger, control)?,
            raw_sha256: text(&row.raw_sha256, ledger, control)?,
            byte_length: row.byte_length,
            item_key: text(&row.item_key, ledger, control)?,
            reviewer_key: text(&row.reviewer_key, ledger, control)?,
            reviewer_role: text(&row.reviewer_role, ledger, control)?,
            disposition: row.disposition,
            responded_at: text(&row.responded_at, ledger, control)?,
            classification: row.classification,
        });
    }
    Ok(result)
}

/// Copy every recorded matching witness; no substitute key or approval is invented.
fn met_seats(
    rows: &[MetSeat],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<MetSeat>, ContractError> {
    let mut result = reserved(rows.len(), ledger)?;
    for row in rows {
        visit(ledger, control)?;
        ledger.bytes(std::mem::size_of::<MetSeat>())?;
        result.push(MetSeat {
            role_key: text(&row.role_key, ledger, control)?,
            ordinal: row.ordinal,
            reviewer_key: text(&row.reviewer_key, ledger, control)?,
            response_id: text(&row.response_id, ledger, control)?,
        });
    }
    Ok(result)
}

/// Copy every unsatisfied seat without shrinking its positive required denominator.
fn unmet_seats(
    rows: &[UnmetSeat],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<UnmetSeat>, ContractError> {
    let mut result = reserved(rows.len(), ledger)?;
    for row in rows {
        visit(ledger, control)?;
        ledger.bytes(std::mem::size_of::<UnmetSeat>())?;
        result.push(UnmetSeat {
            role_key: text(&row.role_key, ledger, control)?,
            ordinal: row.ordinal,
        });
    }
    Ok(result)
}

/// Replace only the pending currentness limitation after real native/source admission;
/// preserve every other reason and lexical ordering, with complete compared extents.
fn current_reasons(
    values: &[String],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<String>, ContractError> {
    let mut result = reserved(values.len().checked_add(1).ok_or(ContractError::Capacity)?, ledger)?;
    let mut inserted = false;
    let mut removed = false;
    for value in values {
        visit(ledger, control)?;
        if compare(value, "currentness-unverified", ledger)?.is_eq() {
            if removed {
                return Err(ContractError::Binding);
            }
            removed = true;
            continue;
        }
        let ordering = compare(value, "captured-current-sources", ledger)?;
        if ordering.is_eq() {
            return Err(ContractError::Binding);
        }
        if !inserted && ordering.is_gt() {
            ledger.bytes(std::mem::size_of::<String>())?;
            result.push(text("captured-current-sources", ledger, control)?);
            inserted = true;
        }
        ledger.bytes(std::mem::size_of::<String>())?;
        result.push(text(value, ledger, control)?);
    }
    if !removed {
        return Err(ContractError::Binding);
    }
    if !inserted {
        ledger.bytes(std::mem::size_of::<String>())?;
        result.push(text("captured-current-sources", ledger, control)?);
    }
    Ok(result)
}

/// Convert pending states only inside the genuine currentness-gated constructor.
/// Quorum remains asserted review policy, never a native domain approval transition.
fn item_state(value: TentativeStateV3) -> ItemState {
    match value {
        TentativeStateV3::Unassigned => ItemState::Unassigned,
        TentativeStateV3::Assigned => ItemState::Assigned,
        TentativeStateV3::InReview => ItemState::InReview,
        TentativeStateV3::Conflicted => ItemState::Conflicted,
        TentativeStateV3::ChangesRequested => ItemState::ChangesRequested,
        TentativeStateV3::SeatsFilled => ItemState::QuorumMet,
        TentativeStateV3::Expired => ItemState::Expired,
        TentativeStateV3::BindingStale => ItemState::Stale,
    }
}

/// Copy every item/witness/reference/dissent fact; only the genuine gate supplies public state.
fn items(
    rows: &[ItemPolicyFactsV3],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<ItemDisposition>, ContractError> {
    let mut result = reserved(rows.len(), ledger)?;
    for row in rows {
        visit(ledger, control)?;
        ledger.bytes(std::mem::size_of::<ItemDisposition>())?;
        result.push(ItemDisposition {
            item_key: text(&row.item_key, ledger, control)?,
            item_id: text(&row.item_id, ledger, control)?,
            state: item_state(row.tentative_state),
            reason_codes: current_reasons(&row.reason_codes, ledger, control)?,
            required_seats: row.required_seats,
            met_seats: met_seats(&row.met_seats, ledger, control)?,
            unmet_seats: unmet_seats(&row.unmet_seats, ledger, control)?,
            response_ids: strings(&row.response_ids, ledger, control)?,
            dissent_ids: strings(&row.dissent_ids, ledger, control)?,
            blocking: row.blocking,
        });
    }
    Ok(result)
}

/// Checked complete state counters, preserving all zeros and never deriving a winning vote.
fn state_counts(
    rows: &[ItemDisposition],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<StateCounts, ContractError> {
    ledger.bytes(std::mem::size_of::<StateCounts>())?;
    let mut result = StateCounts {
        unassigned: 0,
        assigned: 0,
        in_review: 0,
        conflicted: 0,
        changes_requested: 0,
        quorum_met: 0,
        expired: 0,
        stale: 0,
    };
    for row in rows {
        visit(ledger, control)?;
        ledger.bytes(std::mem::size_of::<ItemState>() + std::mem::size_of::<u32>())?;
        let slot = match row.state {
            ItemState::Unassigned => &mut result.unassigned,
            ItemState::Assigned => &mut result.assigned,
            ItemState::InReview => &mut result.in_review,
            ItemState::Conflicted => &mut result.conflicted,
            ItemState::ChangesRequested => &mut result.changes_requested,
            ItemState::QuorumMet => &mut result.quorum_met,
            ItemState::Expired => &mut result.expired,
            ItemState::Stale => &mut result.stale,
        };
        *slot = slot.checked_add(1).ok_or(ContractError::Capacity)?;
    }
    Ok(result)
}

/// Build the complete redacted wire record under the same monotonic logical payload cap.
fn document(
    pending: &PendingPolicyEvaluationV3<'_>,
    pins: &[SourcePinV3],
    generation: String,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<DispositionsDocumentV3, ContractError> {
    ledger.bytes(std::mem::size_of::<DispositionsDocumentV3>())?;
    ledger.derived(std::mem::size_of::<DispositionsDocumentV3>())?;
    let items = items(pending.items(), ledger, control)?;
    let states = state_counts(&items, ledger, control)?;
    let counts = pending.counts();
    Ok(DispositionsDocumentV3 {
        schema_version: text("forge.review-dispositions/3", ledger, control)?,
        identity_disclaimer: text(IDENTITY_DISCLAIMER, ledger, control)?,
        queue_id: text(&pending.queue().document().queue_id, ledger, control)?,
        queue_raw_sha256: text(pending.queue().raw_sha256(), ledger, control)?,
        as_of: text(pending.as_of(), ledger, control)?,
        currentness: RecordedCurrentness::RecordedCurrent,
        closure_generation: Some(generation),
        source_pins: source_pins(pins, ledger, control)?,
        responses: responses(pending.responses(), ledger, control)?,
        items,
        counts: DispositionCounts {
            items: u32::try_from(counts.items).map_err(|_| ContractError::Capacity)?,
            response_files: u32::try_from(counts.response_files)
                .map_err(|_| ContractError::Capacity)?,
            unique_responses: u32::try_from(counts.unique_responses)
                .map_err(|_| ContractError::Capacity)?,
            exact_duplicates: u32::try_from(counts.exact_duplicates)
                .map_err(|_| ContractError::Capacity)?,
            states,
        },
    })
}
