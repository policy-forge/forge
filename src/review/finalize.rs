//! Genuine-current Mapping review finalization, not a native domain promotion.
//! The sole successful port requires the sealed captured adapter and exact held
//! queue/response cohort. Wire claims/hashes/byte-equal detached decoders cannot
//! construct its opaque owner. Root still owns bounded serialization/publication.

use std::io::{self, Write};
use std::rc::Rc;

use serde::{Serialize, Serializer, ser::SerializeSeq};
use sha2::{Digest, Sha256};

use super::adapters::mapping::PreparedMappingReview;
use super::capture::HeldReviewInputs;
use super::chain::{compare, reserved, visit};
use super::decode::{ContractError, ContractLedger, Decoded};
use super::merge::{ItemPolicyFacts, PendingPolicyEvaluation, TentativeState};
use super::validate;
use super::wire::{
    DispositionCounts, DispositionsDocument, IDENTITY_DISCLAIMER, ItemDisposition, ItemState,
    MetSeat, RecordedCurrentness, RecordedResponse, ResponseDocument, SourcePin, StateCounts,
    UnmetSeat,
};
use crate::workspace::preparation::WorkControl;

/// Versioned recorded complete-original binding, never an input approval token.
const GENERATION_PREFIX: &[u8] = b"forge.review-current-closure/1\0";
/// Complete nonretaining generation encoding cap, sharing existing work limits.
const GENERATION_LIMIT: usize = 33_554_432;

/// Currentness-qualified redacted record retaining the real native/shared proof borrows.
/// No Deserialize, Serialize, Clone, Debug or detached success constructor is offered.
pub(crate) struct CurrentMappingDispositions<'a, 'native> {
    /// Complete real root/source/queue/response/auxiliary/output proof owner.
    held: &'a Rc<HeldReviewInputs>,
    /// Selected actual native facts, with the opaque Approved/current closure alive.
    mapping: &'a PreparedMappingReview<'native>,
    /// Owned redacted record; its wire currentness is recorded, not a reusable proof.
    document: DispositionsDocument,
}

impl CurrentMappingDispositions<'_, '_> {
    /// Borrow the complete record for Root's separately bounded serializer.
    /// Its labels cannot be reused as the input to this successful owner constructor.
    pub(crate) fn document(&self) -> &DispositionsDocument {
        &self.document
    }

    /// Repeat the actual native/whole-original fence after any serialization/rendering.
    /// Caller must retain this owner and the same ledger/control through publication.
    pub(crate) fn verify_inputs(
        &self,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        ledger.bound(|ledger| native_fence(self.held, self.mapping, ledger, control))
    }
}

/// Finalize only from the actual complete captured original cohort and native adapter.
/// The held registration bridge and closure's read-only Rc getter are genuine Root
/// ports, not supplied here as stubs; pending policy arithmetic alone cannot succeed.
pub(crate) fn finalize<'a, 'native>(
    held: &'a Rc<HeldReviewInputs>,
    queue_index: usize,
    response_indices: &[usize],
    pending: &PendingPolicyEvaluation<'_>,
    mapping: &'a PreparedMappingReview<'native>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<CurrentMappingDispositions<'a, 'native>, ContractError> {
    ledger.bound(|ledger| {
        ledger.checkpoint(control)?;
        same_owner(held, mapping, ledger)?;
        held.bind_review_originals(queue_index, response_indices, ledger, control)?;
        bind_originals(held, queue_index, response_indices, pending, ledger, control)?;
        mapping.bind_queue(pending.queue(), ledger, control)?;
        let generation = generation(pending, mapping.source_pins(), ledger, control)?;
        let document = document(pending, mapping.source_pins(), generation, ledger, control)?;
        validate::dispositions(&document, ledger, control)?;
        // The adapter's real closure owns THIS same complete held proof. Its fence
        // rechecks every source AND queue/response/auxiliary/output generation.
        native_fence(held, mapping, ledger, control)?;
        Ok(CurrentMappingDispositions { held, mapping, document })
    })
}

/// Require the actual shared owner identity; a caller hash/bool cannot substitute it.
fn same_owner(
    held: &Rc<HeldReviewInputs>,
    mapping: &PreparedMappingReview<'_>,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    ledger.visits(1)?;
    ledger.bytes(2 * std::mem::size_of::<Rc<HeldReviewInputs>>())?;
    if !Rc::ptr_eq(held, mapping.closure().held_inputs()) {
        return Err(ContractError::Binding);
    }
    Ok(())
}

/// Require the entire decoded-original sequence to borrow the actual live allocations.
/// Same bytes in a copied Vec fail; live actual leases prevent allocation-address reuse.
fn bind_originals(
    held: &HeldReviewInputs,
    queue_index: usize,
    response_indices: &[usize],
    pending: &PendingPolicyEvaluation<'_>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    visit(ledger, control)?;
    if response_indices.len() != pending.originals().len()
        || response_indices.len() != pending.counts().response_files
    {
        return Err(ContractError::Binding);
    }
    bind_storage(pending.queue().raw(), held.bytes(queue_index)?, ledger)?;
    for (index, original) in response_indices.iter().zip(pending.originals()) {
        visit(ledger, control)?;
        bind_storage(original.raw(), held.bytes(*index)?, ledger)?;
    }
    ledger.checkpoint(control)
}

/// Compare pointer and complete extent only AFTER real role/cohort/Rc owner admission.
/// This private predicate alone is not a file proof, source gate or decoder constructor.
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

/// Native adapter fence with exact shared owner recheck and explicit final sticky stop.
/// One adapter verification consumes the complete `HeldReviewInputs` proof, not a
/// source-only prefix. It does not claim atomic external state or syscall preemption.
fn native_fence(
    held: &Rc<HeldReviewInputs>,
    mapping: &PreparedMappingReview<'_>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    ledger.checkpoint(control)?;
    same_owner(held, mapping, ledger)?;
    mapping.verify_inputs(ledger, control)?;
    ledger.checkpoint(control)
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
    pins: &[SourcePin],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<SourcePin>, ContractError> {
    let mut result = reserved(pins.len(), ledger)?;
    for pin in pins {
        visit(ledger, control)?;
        ledger.bytes(std::mem::size_of::<SourcePin>())?;
        result.push(SourcePin {
            artifact_key: text(&pin.artifact_key, ledger, control)?,
            model: pin.model,
            native_root_uuid: optional(pin.native_root_uuid.as_deref(), ledger, control)?,
            raw_sha256: text(&pin.raw_sha256, ledger, control)?,
            byte_length: pin.byte_length,
            schema_identity: text(&pin.schema_identity, ledger, control)?,
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
fn item_state(value: TentativeState) -> ItemState {
    match value {
        TentativeState::Unassigned => ItemState::Unassigned,
        TentativeState::Assigned => ItemState::Assigned,
        TentativeState::InReview => ItemState::InReview,
        TentativeState::Conflicted => ItemState::Conflicted,
        TentativeState::ChangesRequested => ItemState::ChangesRequested,
        TentativeState::SeatsFilled => ItemState::QuorumMet,
        TentativeState::Expired => ItemState::Expired,
        TentativeState::BindingStale => ItemState::Stale,
    }
}

/// Copy every item/witness/reference/dissent fact; only the genuine gate supplies public state.
fn items(
    rows: &[ItemPolicyFacts],
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
    pending: &PendingPolicyEvaluation<'_>,
    pins: &[SourcePin],
    generation: String,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<DispositionsDocument, ContractError> {
    ledger.bytes(std::mem::size_of::<DispositionsDocument>())?;
    ledger.derived(std::mem::size_of::<DispositionsDocument>())?;
    let items = items(pending.items(), ledger, control)?;
    let states = state_counts(&items, ledger, control)?;
    let counts = pending.counts();
    Ok(DispositionsDocument {
        schema_version: text("forge.review-dispositions/1", ledger, control)?,
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

/// Exact borrowed original identity, not a serialized raw input or permission to read a path.
#[derive(Serialize)]
struct RawOriginal<'a> {
    /// Actual closed-decoder raw pin after real held-allocation binding.
    raw_sha256: &'a str,
    /// Complete actual borrowed original extent.
    byte_length: usize,
}

/// Serialize every original occurrence in actual admitted order, including exact duplicates.
struct ResponseOriginals<'a>(
    /// Complete borrowed decoded registration cohort; each row already shares an actual live lease.
    &'a [Decoded<'a, ResponseDocument>],
);

impl Serialize for ResponseOriginals<'_> {
    /// Stream complete raw pins/lengths without cloning responses or exposing private rationale.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut rows = serializer.serialize_seq(Some(self.0.len()))?;
        for original in self.0 {
            rows.serialize_element(&RawOriginal {
                raw_sha256: original.raw_sha256(),
                byte_length: original.raw().len(),
            })?;
        }
        rows.end()
    }
}

/// Exact typed complete-current record binding; a digest alone is never authority.
#[derive(Serialize)]
struct ClosureEncoding<'a> {
    /// Fixed first native Mapping re-review profile.
    adapter_version: &'static str,
    /// Actual bound immutable queue original, not reconstructed field equality.
    queue_original: RawOriginal<'a>,
    /// All actual original response registrations; no duplicate-file count loss.
    response_originals: ResponseOriginals<'a>,
    /// Complete native sealed current source union in original source-key order.
    source_pins: &'a [SourcePin],
    /// Explicit asserted UTC policy-evaluation second, not an ambient machine time.
    as_of: &'a str,
}

/// Nonretaining hash sink with complete encoded extent and sticky ledger/control admission.
struct GenerationSink<'a> {
    /// Fixed-size cryptographic accumulator, containing no copied source payload.
    hash: Sha256,
    /// Measured complete compact payload bytes already consumed.
    length: usize,
    /// Original command's monotonic work/logical-storage owner.
    ledger: &'a mut ContractLedger,
    /// Original caller's deadline/cancellation source.
    control: &'a mut dyn WorkControl,
    /// First actual typed writer/control failure, without source-bearing error prose.
    failure: Option<ContractError>,
}

impl Write for GenerationSink<'_> {
    /// Check complete extent/work before every digest update; no encoded Vec is allocated.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.failure.is_some() {
            return Err(io::Error::other("review generation encoding refused"));
        }
        let result = (|| {
            self.ledger.checkpoint(self.control)?;
            self.ledger.visits(1)?;
            let next = self
                .length
                .checked_add(bytes.len())
                .filter(|next| *next <= GENERATION_LIMIT)
                .ok_or(ContractError::Capacity)?;
            self.ledger.bytes(bytes.len())?;
            self.hash.update(bytes);
            self.length = next;
            Ok(())
        })();
        if let Err(error) = result {
            self.failure = Some(error);
            return Err(io::Error::other("review generation encoding refused"));
        }
        Ok(bytes.len())
    }
    /// Consume the same caller checkpoint; there is no external IO stream to flush.
    fn flush(&mut self) -> io::Result<()> {
        if self.failure.is_some() {
            return Err(io::Error::other("review generation encoding refused"));
        }
        self.ledger.checkpoint(self.control).map_err(|error| {
            self.failure = Some(error);
            io::Error::other("review generation encoding refused")
        })
    }
}

/// Compute a domain-separated record binding only from the already owner/native-bound cohort.
/// Prefix + compact fields/order are versioned; this is not JCS or a currentness factory.
fn generation(
    pending: &PendingPolicyEvaluation<'_>,
    pins: &[SourcePin],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<String, ContractError> {
    ledger.checkpoint(control)?;
    ledger.bytes(GENERATION_PREFIX.len())?;
    let mut hash = Sha256::new();
    hash.update(GENERATION_PREFIX);
    let encoding = ClosureEncoding {
        adapter_version: "forge.mapping-review/1",
        queue_original: RawOriginal {
            raw_sha256: pending.queue().raw_sha256(),
            byte_length: pending.queue().raw().len(),
        },
        response_originals: ResponseOriginals(pending.originals()),
        source_pins: pins,
        as_of: pending.as_of(),
    };
    let mut sink = GenerationSink { hash, length: 0, ledger, control, failure: None };
    if serde_json::to_writer(&mut sink, &encoding).is_err() {
        return Err(sink.failure.unwrap_or(ContractError::Invalid));
    }
    sink.ledger.checkpoint(sink.control)?;
    sink.ledger.derived(64)?;
    Ok(crate::hashing::lower_hex(&sink.hash.finalize()))
}

#[cfg(test)]
/// Narrow predicate/storage controls; no synthetic closure issues current status or quorum.
mod tests {
    use super::*;
    use crate::workspace::preparation::NoopControl;

    /// Byte equality from a different live allocation cannot satisfy the private storage predicate.
    #[test]
    fn copied_equal_original_is_not_the_actual_allocation() {
        let actual = Rc::new(b"immutable actual original".to_vec());
        let lease = Rc::clone(&actual);
        let copied = actual.as_ref().clone();
        let mut ledger = ContractLedger::default();
        assert_eq!(bind_storage(lease.as_slice(), actual.as_slice(), &mut ledger), Ok(()));
        assert_eq!(
            bind_storage(copied.as_slice(), actual.as_slice(), &mut ledger),
            Err(ContractError::Binding)
        );
        assert_eq!(
            bind_storage(&actual[..actual.len() - 1], actual.as_slice(), &mut ledger),
            Err(ContractError::Binding)
        );
    }

    /// Redacted-copy growth uses the same prior logical reservation and exact sticky refusal.
    #[test]
    fn redacted_copy_cannot_reset_prior_logical_capacity() {
        let mut ledger = ContractLedger::default();
        ledger.derived(33_554_431).unwrap();
        assert!(matches!(
            ledger.bound(|ledger| text("xx", ledger, &mut NoopControl)),
            Err(ContractError::Capacity)
        ));
        assert!(matches!(
            ledger.bound(|ledger| text("", ledger, &mut NoopControl)),
            Err(ContractError::Capacity)
        ));
    }
}
