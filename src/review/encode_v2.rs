//! Pure finite /2 recorded JSON encoding and complete exact typed readback.
//! Inputs and output bytes are ordinary data, never a native/currentness owner.
//! Root keeps actual queue/native binding and guarded publication separately.
//! Logical payload, primitive work and possible reallocation copies are admitted;
//! allocator spare capacity and serde internals are not total-heap confinement.

use super::decode::{ContractError, ContractLedger};
use super::decode_v2;
use super::wire::{StateCounts, UnmetSeat};
use super::wire_v2::{
    DispositionCounts, DispositionsDocumentV2, ItemDisposition, MetSeat, RecordedResponse,
    SourcePinV2,
};
use crate::workspace::preparation::WorkControl;
use serde::Serialize;
use std::io::{self, Write};

/// Unchanged full recorded-document raw cap, including its one trailing LF.
const MAX_RECORDED_BYTES: usize = 33_554_432;

/// Admit all full original string bytes before this typed operand is inspected again.
fn text(value: &str, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    ledger.visits(1)?;
    ledger.bytes(value.len())
}
/// A fixed scalar/tag unit conservatively covers a bounded field and its comparison.
fn fixed(fields: usize, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    ledger.visits(fields)?;
    let extent = fields.checked_mul(64).ok_or_else(|| ledger.capacity())?;
    ledger.bytes(extent)
}
/// Preserve null versus present-empty and admit the complete present string operand.
fn optional(value: Option<&str>, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    fixed(1, ledger)?;
    if let Some(value) = value {
        text(value, ledger)?;
    }
    Ok(())
}
/// Charge the complete supplied order; no sorting, omission or deduplication occurs.
fn strings(values: &[String], ledger: &mut ContractLedger) -> Result<(), ContractError> {
    fixed(1, ledger)?;
    ledger.visits(values.len())?;
    for value in values {
        text(value, ledger)?;
    }
    Ok(())
}
/// Exhaustive eight-field pin admission; a new field forces this pattern to change.
fn pin(value: &SourcePinV2, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    let SourcePinV2 {
        artifact_key,
        kind: _,
        raw_sha256,
        byte_length: _,
        schema_identity,
        validation_profile,
        native_model: _,
        native_root_uuid,
    } = value;
    fixed(3, ledger)?;
    for value in [artifact_key, raw_sha256, validation_profile] {
        text(value, ledger)?;
    }
    optional(schema_identity.as_deref(), ledger)?;
    optional(native_root_uuid.as_deref(), ledger)
}
/// Exhaustive complete original-response metadata, including every recorded label.
fn response(value: &RecordedResponse, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    let RecordedResponse {
        response_id,
        raw_sha256,
        byte_length: _,
        item_key,
        reviewer_key,
        reviewer_role,
        disposition: _,
        responded_at,
        classification: _,
    } = value;
    fixed(3, ledger)?;
    for value in [response_id, raw_sha256, item_key, reviewer_key, reviewer_role, responded_at] {
        text(value, ledger)?;
    }
    Ok(())
}
/// Exhaustive matching witness data; no matching or reviewer authority is inferred.
fn met(value: &MetSeat, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    let MetSeat { role_key, ordinal: _, reviewer_key, response_id } = value;
    fixed(1, ledger)?;
    for value in [role_key, reviewer_key, response_id] {
        text(value, ledger)?;
    }
    Ok(())
}
/// Exhaustive unmet-seat data, without a default or a manufactured approval.
fn unmet(value: &UnmetSeat, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    let UnmetSeat { role_key, ordinal: _ } = value;
    fixed(1, ledger)?;
    text(role_key, ledger)
}
/// Admit complete ordered status, witness, response and historical dissent operands.
fn item(value: &ItemDisposition, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    let ItemDisposition {
        item_key,
        item_id,
        state: _,
        reason_codes,
        required_seats: _,
        met_seats,
        unmet_seats,
        response_ids,
        dissent_ids,
        blocking: _,
    } = value;
    fixed(5, ledger)?;
    text(item_key, ledger)?;
    text(item_id, ledger)?;
    strings(reason_codes, ledger)?;
    strings(response_ids, ledger)?;
    strings(dissent_ids, ledger)?;
    ledger.visits(met_seats.len())?;
    for value in met_seats {
        met(value, ledger)?;
    }
    ledger.visits(unmet_seats.len())?;
    for value in unmet_seats {
        unmet(value, ledger)?;
    }
    Ok(())
}
/// Admit all twelve denominators; the exhaustive patterns preserve future field coverage.
fn counts(value: &DispositionCounts, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    let DispositionCounts {
        items: _,
        response_files: _,
        unique_responses: _,
        exact_duplicates: _,
        states,
    } = value;
    let StateCounts {
        unassigned: _,
        assigned: _,
        in_review: _,
        conflicted: _,
        changes_requested: _,
        quorum_met: _,
        expired: _,
        stale: _,
    } = states;
    fixed(12, ledger)
}
/// Full typed serialization/equality operands; borrowed geometry creates no new owner.
fn operands(
    value: &DispositionsDocumentV2,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let DispositionsDocumentV2 {
        schema_version,
        identity_disclaimer,
        queue_id,
        queue_raw_sha256,
        as_of,
        currentness: _,
        closure_generation,
        source_pins,
        responses,
        items,
        counts: totals,
    } = value;
    decode_v2::checkpoint(ledger, control)?;
    fixed(4, ledger)?;
    for value in [schema_version, identity_disclaimer, queue_id, queue_raw_sha256, as_of] {
        text(value, ledger)?;
    }
    optional(closure_generation.as_deref(), ledger)?;
    ledger.visits(source_pins.len())?;
    for value in source_pins {
        decode_v2::checkpoint(ledger, control)?;
        pin(value, ledger)?;
    }
    ledger.visits(responses.len())?;
    for value in responses {
        decode_v2::checkpoint(ledger, control)?;
        response(value, ledger)?;
    }
    ledger.visits(items.len())?;
    for value in items {
        decode_v2::checkpoint(ledger, control)?;
        item(value, ledger)?;
    }
    counts(totals, ledger)
}

/// Sole finite output Vec; no intermediate String/Value or external writer exists.
struct JsonWriter<'a> {
    /// Complete owned output payload, with one LF included in the same limit.
    output: Vec<u8>,
    /// Exact selected cap, never larger than the unchanged recorded cap.
    limit: usize,
    /// Same original invocation's monotonic ledger.
    ledger: &'a mut ContractLedger,
    /// Same original accepted controller and deadline.
    control: &'a mut dyn WorkControl,
    /// First actual typed stop through serde's non-sensitive IO error wrapper.
    error: Option<ContractError>,
}
impl JsonWriter<'_> {
    /// Admit a complete emitted token before Vec growth and possible prior-payload copy.
    fn append(&mut self, bytes: &[u8]) -> Result<(), ContractError> {
        decode_v2::checkpoint(self.ledger, self.control)?;
        self.ledger.visits(1)?;
        self.ledger.bytes(bytes.len())?;
        let next =
            self.output.len().checked_add(bytes.len()).ok_or_else(|| self.ledger.capacity())?;
        if next > self.limit {
            return Err(self.ledger.capacity());
        }
        self.ledger.derived(bytes.len())?;
        if next > self.output.capacity() {
            self.ledger.bytes(self.output.len())?;
            let capacity = self.output.capacity().saturating_mul(2).clamp(1, self.limit).max(next);
            self.output
                .try_reserve_exact(capacity - self.output.len())
                .map_err(|_| self.ledger.capacity())?;
        }
        self.output.extend_from_slice(bytes);
        Ok(())
    }
    /// Preserve first exact cause while hiding serde/IO diagnostics from consumers.
    fn refused(&mut self, error: ContractError) -> io::Error {
        if self.error.is_none() {
            self.error = Some(error);
        }
        io::Error::other("bounded review-v2 JSON encoding refused")
    }
}
impl Write for JsonWriter<'_> {
    /// Every complete token uses actual original pre-growth admission.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if let Some(error) = self.error {
            return Err(self.refused(error));
        }
        self.append(bytes).map_err(|error| self.refused(error))?;
        Ok(bytes.len())
    }
    /// No external stream; an explicit flush still observes the original sticky stop.
    fn flush(&mut self) -> io::Result<()> {
        if let Some(error) = self.error {
            return Err(self.refused(error));
        }
        decode_v2::checkpoint(self.ledger, self.control).map_err(|error| self.refused(error))
    }
}
/// Private generic writer for component controls; ordinary failures always reach its postfence.
fn encoded<T: Serialize>(
    document: &T,
    limit: usize,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<u8>, ContractError> {
    decode_v2::phase(ledger, control, |ledger, control| {
        if limit == 0 || limit > MAX_RECORDED_BYTES {
            return Err(ContractError::Invalid);
        }
        ledger.derived(std::mem::size_of::<Vec<u8>>())?;
        let mut writer = JsonWriter { output: Vec::new(), limit, ledger, control, error: None };
        if serde_json::to_writer(&mut writer, document).is_err() {
            return Err(writer.error.unwrap_or(ContractError::Invalid));
        }
        writer.append(b"\n")?;
        Ok(writer.output)
    })
}
/// Exact finite output readback plus full typed equality; success remains ordinary data.
fn readback(
    expected: &DispositionsDocumentV2,
    raw: &[u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    decode_v2::phase(ledger, control, |ledger, control| {
        let actual = decode_v2::decode_dispositions(raw, ledger, control)?;
        operands(expected, ledger, control)?;
        operands(actual.document(), ledger, control)?;
        if actual.document() != expected {
            return Err(ContractError::Binding);
        }
        Ok(())
    })
}
/// Encode a full plain /2 DTO, strictly read the actual bytes and check every typed field.
/// Root separately binds actual queue/native owners and keeps them through guarded publication.
pub(crate) fn dispositions(
    document: &DispositionsDocumentV2,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<u8>, ContractError> {
    decode_v2::phase(ledger, control, |ledger, control| {
        decode_v2::dispositions(document, ledger, control)?;
        operands(document, ledger, control)?;
        let output = encoded(document, MAX_RECORDED_BYTES, ledger, control)?;
        readback(document, &output, ledger, control)?;
        Ok(output)
    })
}

#[cfg(test)]
#[path = "encode_v2_tests.rs"]
/// Real finite writer/strict-readback controls with ordinary inert DTO fixtures.
mod tests;

/// Pure additive typed queue/response codecs, retaining every old disposition byte.
#[path = "encode_v2_exchange.rs"]
pub(crate) mod exchange;
