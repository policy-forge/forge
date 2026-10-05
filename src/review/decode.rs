//! Duplicate-safe closed decoding with exact original-byte identity.
//! Raw slices are borrowed, never reconstructed from typed fields. Parser and
//! schema-validator internals are not covered by the logical derived byte cap;
//! raw caps and decoded-tree limits do not establish total heap confinement.

use serde::de::DeserializeOwned;
use serde_json::Value;

use super::wire::{DispositionsDocument, QueueDocument, ResponseDocument};
use crate::workspace::preparation::{Interruption, ProgressUpdate, Stage, WorkControl, WorkError};

/// Fixed safe failures; no input paths, keys, rationale or parser prose escape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub(crate) enum ContractError {
    /// Syntax, shape or structural correlation is invalid.
    #[error("invalid review contract")]
    Invalid,
    /// A complete command-wide capacity was exceeded.
    #[error("review contract capacity exceeded")]
    Capacity,
    /// Same response UUID has differing complete original bytes.
    #[error("review response identity conflict")]
    IdentityConflict,
    /// A supersession reference is missing, mismatched, cyclic or branched.
    #[error("invalid review response chain")]
    InvalidChain,
    /// An exact queue/item/policy/source assertion differs.
    #[error("review response binding mismatch")]
    Binding,
    /// No domain-specific edit syntax is implemented in this slice.
    #[error("review proposed edit unsupported")]
    UnsupportedEdit,
    /// A trusted embedded schema cannot be compiled.
    #[error("review schema definition invalid")]
    SchemaDefinition,
    /// Caller control stopped work; preserve the exact typed interruption.
    #[error("review preparation interrupted")]
    Interrupted(Interruption),
    /// An ordinary shared-control failure cannot be a successful empty result.
    #[error("review preparation control failed")]
    ControlFailed,
}

/// One monotonic structural ledger, retained across all contract decodes.
/// The actual file-capture, native and matching ledgers must consume this same
/// command owner later; these counters are not a physical file proof or timer.
#[derive(Default)]
pub(crate) struct ContractLedger {
    /// Complete repeated primitive byte work, at most 256 MiB.
    byte_work: u64,
    /// Complete repeated matching visits/probes/backtracking, at most ten million.
    matching_steps: u64,
    /// Complete explicit structural inspection visits, at most one million.
    visits: u64,
    /// Conservative owned typed/registry logical storage, at most 32 MiB.
    derived: u64,
    /// Bounded raw response registrations after preflight, including duplicates.
    response_bytes: u64,
    /// Bounded raw registrations before UTF-8/parse; complete file attempts are IO-owned.
    response_files: u64,
    /// Expanded seat representation across all decoded queues.
    seats: u64,
    /// Complete planned candidate edge occurrences across all items.
    edges: u64,
    /// Conservative candidate/seat graph vertices across all items.
    vertices: u64,
    /// First latched capacity or control stop.
    stopped: Option<ContractError>,
}

impl ContractLedger {
    /// Bound every external structural/hash/binding result by the same first
    /// capacity/control stop. Ordinary invalid/binding/edit refusals do not latch.
    pub(crate) fn bound<T>(
        &mut self,
        work: impl FnOnce(&mut Self) -> Result<T, ContractError>,
    ) -> Result<T, ContractError> {
        if let Some(error) = self.stopped {
            return Err(error);
        }
        let result = work(self);
        if let Some(error) = self.stopped {
            return Err(error);
        }
        match result {
            Err(ContractError::Capacity) => Err(self.capacity()),
            Err(error @ (ContractError::Interrupted(_) | ContractError::ControlFailed)) => {
                self.stopped = Some(error);
                Err(error)
            }
            ordinary => ordinary,
        }
    }

    /// Precharge a bounded sum without overflow; capacity remains sticky.
    fn add(slot: &mut u64, amount: u64, cap: u64) -> Result<(), ContractError> {
        let next = slot.checked_add(amount).ok_or(ContractError::Capacity)?;
        if next > cap {
            return Err(ContractError::Capacity);
        }
        *slot = next;
        Ok(())
    }

    /// Latch capacity/control failure before any later contract can succeed.
    fn remember(&mut self, result: Result<(), ContractError>) -> Result<(), ContractError> {
        if let Some(error) = self.stopped {
            return Err(error);
        }
        if let Err(error) = result {
            self.stopped = Some(error);
        }
        result
    }

    /// Charge complete repeated byte work before the primitive or comparison.
    pub(crate) fn bytes(&mut self, amount: usize) -> Result<(), ContractError> {
        if let Some(error) = self.stopped {
            return Err(error);
        }
        let amount = u64::try_from(amount).map_err(|_| ContractError::Capacity)?;
        let result = Self::add(&mut self.byte_work, amount, 268_435_456);
        self.remember(result)
    }

    /// Charge explicit visits before a registry scan or inspection.
    pub(crate) fn visits(&mut self, amount: usize) -> Result<(), ContractError> {
        if let Some(error) = self.stopped {
            return Err(error);
        }
        let amount = u64::try_from(amount).map_err(|_| ContractError::Capacity)?;
        let result = Self::add(&mut self.visits, amount, 1_000_000);
        self.remember(result)
    }

    /// Charge every actual matching node/edge/reset/backtracking visit, including repeats.
    pub(crate) fn matching(&mut self, amount: usize) -> Result<(), ContractError> {
        if let Some(error) = self.stopped {
            return Err(error);
        }
        let amount = u64::try_from(amount).map_err(|_| ContractError::Capacity)?;
        let result = Self::add(&mut self.matching_steps, amount, 10_000_000);
        self.remember(result)
    }

    /// Reserve conservative typed payload/registry storage before its growth.
    pub(crate) fn derived(&mut self, amount: usize) -> Result<(), ContractError> {
        if let Some(error) = self.stopped {
            return Err(error);
        }
        let amount = u64::try_from(amount).map_err(|_| ContractError::Capacity)?;
        let result = Self::add(&mut self.derived, amount, 33_554_432);
        self.remember(result)
    }

    /// Precharge each raw-preflight-admitted response; actual file attempts/holders are IO-owned.
    fn response(&mut self, bytes: usize) -> Result<(), ContractError> {
        if let Some(error) = self.stopped {
            return Err(error);
        }
        let bytes = u64::try_from(bytes).map_err(|_| ContractError::Capacity)?;
        let result = Self::add(&mut self.response_files, 1, 10_000)
            .and_then(|()| Self::add(&mut self.response_bytes, bytes, 33_554_432));
        self.remember(result)
    }

    /// Reserve complete proposed graph sizes; this never performs matching.
    pub(crate) fn graph(
        &mut self,
        seats: usize,
        edges: usize,
        vertices: usize,
    ) -> Result<(), ContractError> {
        if let Some(error) = self.stopped {
            return Err(error);
        }
        let seats = u64::try_from(seats).map_err(|_| ContractError::Capacity)?;
        let edges = u64::try_from(edges).map_err(|_| ContractError::Capacity)?;
        let vertices = u64::try_from(vertices).map_err(|_| ContractError::Capacity)?;
        let result = Self::add(&mut self.seats, seats, 100_000)
            .and_then(|()| Self::add(&mut self.edges, edges, 200_000))
            .and_then(|()| Self::add(&mut self.vertices, vertices, 400_000));
        self.remember(result)
    }

    /// Preserve caller cancellation and original deadline before bounded phases.
    pub(crate) fn checkpoint(
        &mut self,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        if let Some(error) = self.stopped {
            return Err(error);
        }
        let result =
            control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged).map_err(|error| {
                match error {
                    WorkError::Interrupted(reason) => ContractError::Interrupted(reason),
                    WorkError::Failed(_) => ContractError::ControlFailed,
                }
            });
        self.remember(result)
    }

    /// Latch an explicit complete encoded-capacity refusal before further work.
    pub(crate) fn capacity(&mut self) -> ContractError {
        let error = self.stopped.unwrap_or(ContractError::Capacity);
        self.stopped = Some(error);
        error
    }

    /// Recover a precise sticky writer/control failure without raw IO prose.
    pub(crate) fn failure(&self) -> ContractError {
        self.stopped.unwrap_or(ContractError::Invalid)
    }
}

/// Borrowed exact original plus an inert typed declaration. No public constructor,
/// clone, debug disclosure or file/currentness proof is exposed.
pub(crate) struct Decoded<'a, T> {
    /// Actual bytes passed to this decoder, retained unchanged by borrowing.
    raw: &'a [u8],
    /// SHA-256 of those original bytes, including whitespace and final LF.
    raw_sha256: String,
    /// Closed typed declarations; all domain facts remain unverified here.
    document: T,
}

impl<'a, T> Decoded<'a, T> {
    /// Borrow inert declarations without manufacturing native approval.
    pub(crate) fn document(&self) -> &T {
        &self.document
    }
    /// Borrow the exact original extent, not reserialized JSON.
    pub(crate) fn raw(&self) -> &'a [u8] {
        self.raw
    }
    /// Borrow its complete original-byte pin.
    pub(crate) fn raw_sha256(&self) -> &str {
        &self.raw_sha256
    }
}

/// Three complete original-identity relations; no semantic equality is used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OriginalRelation {
    /// Different explicit response UUIDs remain independent evidence.
    DifferentIdentity,
    /// Same UUID and exactly equal full original bytes may count once.
    ExactDuplicate,
    /// Same UUID with any differing original bytes refuses a validated bundle.
    IdentityConflict,
}

/// Compare exact immutable response identity after precharging full extents.
pub(crate) fn original_relation(
    left: &Decoded<'_, ResponseDocument>,
    right: &Decoded<'_, ResponseDocument>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<OriginalRelation, ContractError> {
    ledger.bound(|ledger| {
        ledger.checkpoint(control)?;
        ledger.bytes(
            left.document
                .response_id
                .len()
                .checked_add(right.document.response_id.len())
                .ok_or(ContractError::Capacity)?,
        )?;
        if left.document.response_id != right.document.response_id {
            return Ok(OriginalRelation::DifferentIdentity);
        }
        ledger
            .bytes(left.raw.len().checked_add(right.raw.len()).ok_or(ContractError::Capacity)?)?;
        let relation = if left.raw == right.raw {
            OriginalRelation::ExactDuplicate
        } else {
            OriginalRelation::IdentityConflict
        };
        ledger.checkpoint(control)?;
        Ok(relation)
    })
}

/// Walk an already bounded decoded tree before moving it into typed containers.
/// Sixty-four bytes per node plus all string/key bytes is conservative logical
/// representation accounting, not an allocator or parser-internal heap bound.
fn reserve_typed(value: &Value, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    ledger.visits(1)?;
    ledger.derived(64)?;
    match value {
        Value::String(text) => ledger.derived(text.len())?,
        Value::Array(rows) => {
            for row in rows {
                reserve_typed(row, ledger)?;
            }
        }
        Value::Object(rows) => {
            for (key, row) in rows {
                ledger.derived(key.len())?;
                reserve_typed(row, ledger)?;
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
    Ok(())
}

/// Decode one closed raw document; primitive allocation limits are qualified above.
fn decode<'a, T: DeserializeOwned>(
    raw: &'a [u8],
    cap: usize,
    schema: &str,
    response: bool,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Decoded<'a, T>, ContractError> {
    ledger.checkpoint(control)?;
    if raw.len() > cap {
        return Err(ledger.capacity());
    }
    if raw.is_empty() || raw.starts_with(&[0xef, 0xbb, 0xbf]) {
        return Err(ContractError::Invalid);
    }
    if response {
        ledger.response(raw.len())?;
    }
    ledger.bytes(raw.len())?;
    std::str::from_utf8(raw).map_err(|_| ContractError::Invalid)?;
    ledger.bytes(raw.len())?;
    let value = crate::json_strict::parse_value(
        raw,
        "review",
        crate::json_strict::Limits { max_depth: 64, max_string_bytes: 65_536 },
    )
    .map_err(|_| ContractError::Invalid)?;
    ledger.checkpoint(control)?;
    if response && value.get("proposed_edit").is_some_and(|edit| !edit.is_null()) {
        return Err(ContractError::UnsupportedEdit);
    }
    ledger.bytes(schema.len())?;
    let definition: Value =
        serde_json::from_str(schema).map_err(|_| ContractError::SchemaDefinition)?;
    let validator =
        jsonschema::validator_for(&definition).map_err(|_| ContractError::SchemaDefinition)?;
    ledger.bytes(raw.len())?;
    if !validator.is_valid(&value) {
        return Err(ContractError::Invalid);
    }
    reserve_typed(&value, ledger)?;
    ledger.bytes(raw.len())?;
    let document = serde_json::from_value(value).map_err(|_| ContractError::Invalid)?;
    ledger.bytes(raw.len())?;
    ledger.derived(64)?;
    let raw_sha256 = crate::hashing::sha256_hex(raw);
    ledger.checkpoint(control)?;
    Ok(Decoded { raw, raw_sha256, document })
}

/// Decode a closed queue, without native source/capture or quorum authority.
pub(crate) fn decode_queue<'a>(
    raw: &'a [u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Decoded<'a, QueueDocument>, ContractError> {
    ledger.bound(|ledger| {
        let decoded = decode(
            raw,
            10_485_760,
            include_str!("../../schemas/forge.review-queue-1.schema.json"),
            false,
            ledger,
            control,
        )?;
        super::validate::queue(decoded.document(), ledger, control)?;
        Ok(decoded)
    })
}

/// Decode private immutable response assertions; exact queue binding is separate.
pub(crate) fn decode_response<'a>(
    raw: &'a [u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Decoded<'a, ResponseDocument>, ContractError> {
    ledger.bound(|ledger| {
        let decoded = decode(
            raw,
            1_048_576,
            include_str!("../../schemas/forge.review-response-1.schema.json"),
            true,
            ledger,
            control,
        )?;
        super::validate::response(decoded.document(), ledger, control)?;
        Ok(decoded)
    })
}

/// Decode a recorded bundle; its currentness/quorum labels remain inert claims.
pub(crate) fn decode_dispositions<'a>(
    raw: &'a [u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Decoded<'a, DispositionsDocument>, ContractError> {
    ledger.bound(|ledger| {
        let decoded = decode(
            raw,
            33_554_432,
            include_str!("../../schemas/forge.review-dispositions-1.schema.json"),
            false,
            ledger,
            control,
        )?;
        super::validate::dispositions(decoded.document(), ledger, control)?;
        Ok(decoded)
    })
}
