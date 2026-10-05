//! Complete bounded compact JSON plus LF under the invocation's existing ledger.
//! The writer retains one output Vec and admits every encoded growth before append.
//! Logical payload and possible reallocation-copy work are counted; allocator spare
//! capacity and serde internals are not a total-heap or preemption guarantee.

use super::decode::{ContractError, ContractLedger};
use super::validate;
use super::wire::{DispositionsDocument, QueueDocument, ResponseDocument};
use crate::workspace::preparation::WorkControl;
use serde::Serialize;
use std::io::{self, Write};

/// One compact output owner, including its required trailing LF in the ceiling.
struct JsonWriter<'a> {
    /// Sole retained encoded payload; no unbounded intermediate String or Value.
    output: Vec<u8>,
    /// Exact selected closed-document raw ceiling, including LF.
    limit: usize,
    /// Original monotonic command ledger, not an independent encoding allowance.
    ledger: &'a mut ContractLedger,
    /// Original caller's accepted cooperative control and deadline.
    control: &'a mut dyn WorkControl,
    /// First exact writer failure preserved through serde's IO error wrapper.
    error: Option<ContractError>,
}

impl JsonWriter<'_> {
    /// Precharge a complete token before output growth and any possible prior-payload copy.
    fn append(&mut self, bytes: &[u8]) -> Result<(), ContractError> {
        self.ledger.checkpoint(self.control)?;
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

    /// Retain the exact first typed cause while returning a fixed non-sensitive IO error.
    fn refused(&mut self, error: ContractError) -> io::Error {
        if self.error.is_none() {
            self.error = Some(error);
        }
        io::Error::other("bounded review JSON encoding refused")
    }
}

impl Write for JsonWriter<'_> {
    /// Serialize each full token through the same pre-growth control/storage admission.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if let Some(error) = self.error {
            return Err(self.refused(error));
        }
        self.append(bytes).map_err(|error| self.refused(error))?;
        Ok(bytes.len())
    }

    /// No external stream exists; flushing still checks the original sticky control.
    fn flush(&mut self) -> io::Result<()> {
        if let Some(error) = self.error {
            return Err(self.refused(error));
        }
        self.ledger.checkpoint(self.control).map_err(|error| self.refused(error))
    }
}

/// Encode one complete typed payload, preserving typed stops instead of serde diagnostics.
fn encoded<T: Serialize>(
    document: &T,
    limit: usize,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<u8>, ContractError> {
    ledger.bound(|ledger| {
        ledger.checkpoint(control)?;
        if limit == 0 || limit > 33_554_432 {
            return Err(ContractError::Invalid);
        }
        ledger.derived(std::mem::size_of::<Vec<u8>>())?;
        let mut writer = JsonWriter { output: Vec::new(), limit, ledger, control, error: None };
        if serde_json::to_writer(&mut writer, document).is_err() {
            return Err(writer.error.unwrap_or(ContractError::Invalid));
        }
        writer.append(b"\n")?;
        writer.ledger.checkpoint(writer.control)?;
        Ok(writer.output)
    })
}

/// Validate and encode a closed asserted queue within its ten-MiB raw ceiling.
/// Encoding alone grants no native currentness, authenticated identity or publication.
pub(crate) fn queue(
    document: &QueueDocument,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<u8>, ContractError> {
    ledger.bound(|ledger| {
        validate::queue(document, ledger, control)?;
        encoded(document, 10_485_760, ledger, control)
    })
}

/// Validate and encode a private response within one MiB, with explicit null edit syntax.
pub(crate) fn response(
    document: &ResponseDocument,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<u8>, ContractError> {
    ledger.bound(|ledger| {
        validate::response(document, ledger, control)?;
        encoded(document, 1_048_576, ledger, control)
    })
}

/// Validate and encode all disposition records within 32 MiB of the same logical ledger.
/// Recorded labels remain inert; a genuine finalizer owner must survive the output fence.
pub(crate) fn dispositions(
    document: &DispositionsDocument,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<u8>, ContractError> {
    ledger.bound(|ledger| {
        validate::dispositions(document, ledger, control)?;
        encoded(document, 33_554_432, ledger, control)
    })
}

/// Encode a complete minimized companion under the same finite 32-MiB writer.
/// The genuine command checks the actual encoded recorded form before publication.
/// Encoding grants no capture, native currentness, approval or publication capability.
pub(crate) fn supersession(
    document: &super::supersession_wire::SupersessionDocument,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<u8>, ContractError> {
    encoded(document, 33_554_432, ledger, control)
}

#[cfg(test)]
#[path = "encode_tests.rs"]
/// Literal writer/closed-decoder controls; no positive native proof is manufactured.
mod tests;
