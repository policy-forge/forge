//! Session-owned raw-artifact stages, separate from project write authority.
//!
//! ROOT embeds this object inside the existing shared Store mutex. `OtherPoolUsage`
//! must include ALL non-stage retained bytes/records, including unresolved source
//! reservations. This module has no independent20MiB pool, filesystem access,
//! capability, journal, route, public version advertisement or automatic commit.
//! All heavy full-artifact hash/codec work happens through an off-lock lease.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use serde_json::{Value, json};

use super::contract::{Error, Result};
use super::preparation::{ProgressUpdate, Stage, WorkControl, WorkResult};

/// Logical Bundle4 cap; a valid artifact may still fail complete shared admission.
pub(crate) const MAX_ARTIFACT_BYTES: usize = 10 * 1024 * 1024;
/// Exact raw artifact bytes per transport part; wire hex is twice this size.
pub(crate) const PART_BYTES: usize = 32 * 1024;
/// Canonical whole artifact requires at most320 parts, never320 replay records.
const MAX_PARTS: usize = MAX_ARTIFACT_BYTES / PART_BYTES;
/// Finite request/ack cells, identity and metadata reserve inside the same pool.
const METADATA_RESERVE: usize = 64 * 1024;
/// Existing shared pool remains unchanged; this is a combined admission check.
const MAX_RETAINED_BYTES: usize = 20 * 1024 * 1024;
/// Every stage displaces one record from the existing complete256record capacity.
const MAX_RETAINED_RECORDS: usize = 256;
/// One original nonrenewing expiry, measured from successful stage creation.
const STAGE_LIFETIME: Duration = Duration::from_secs(600);

/// ROOT's current retained usage excluding only this `SourceTransfers` field.
#[derive(Clone, Copy, Default)]
pub(crate) struct OtherPoolUsage {
    /// All other private bytes plus complete conservative public/replay charges.
    pub(crate) bytes: usize,
    /// All other records, including unresolved native outcomes/reservations.
    pub(crate) records: usize,
}

/// Closed user declaration, validated before a stage can reserve or allocate.
#[derive(Clone)]
pub(crate) struct StageRequest {
    /// Expected SHA256 of the genuine original raw file, not normalized JSON.
    sha256: String,
    /// Complete original artifact size; allocated/reserved once before any part.
    bytes: usize,
    /// Exact ceil(size/32768); intrinsic parsing checks this relationship.
    parts: usize,
}

impl StageRequest {
    /// Admit the complete closed manifest and explicit source/metadata opt-in.
    pub(crate) fn parse(value: &Value) -> Result<Self> {
        closed(
            value,
            &[
                "schema_version",
                "profile",
                "artifact_sha256",
                "artifact_size_bytes",
                "chunk_size_bytes",
                "chunk_count",
                "acknowledge_sensitive_metadata",
                "acknowledge_source_content",
            ],
        )?;
        let sha256 = value["artifact_sha256"].as_str().ok_or_else(Error::invalid)?;
        let bytes = unsigned(&value["artifact_size_bytes"])?;
        let parts = unsigned(&value["chunk_count"])?;
        if value["schema_version"] != "forge.workspace-index-bundle/4"
            || value["profile"] != "index-and-source-hex-staged"
            || value["acknowledge_sensitive_metadata"] != true
            || value["acknowledge_source_content"] != true
            || unsigned(&value["chunk_size_bytes"])? != PART_BYTES
            || !hash64(sha256)
            || bytes == 0
            || bytes > MAX_ARTIFACT_BYTES
            || parts == 0
            || parts > MAX_PARTS
            || parts != bytes.div_ceil(PART_BYTES)
        {
            return Err(Error::invalid());
        }
        Ok(Self { sha256: sha256.into(), bytes, parts })
    }

    /// Bind an off-lock intrinsic admission to the exact Value retained for create replay.
    /// This is bounded scalar validation, not raw JSON/codec parsing under Store.
    pub(crate) fn matches(&self, value: &Value) -> Result<bool> {
        let admitted = Self::parse(value)?;
        Ok(self.sha256 == admitted.sha256
            && self.bytes == admitted.bytes
            && self.parts == admitted.parts)
    }
}

/// One off-lock decoded, fully hashed part; ROOT strictly parses original JSON first.
pub(crate) struct DecodedPart {
    /// Exact inert original artifact bytes, at most32KiB.
    bytes: Vec<u8>,
    /// Independently recomputed digest; no source text in public failure details.
    sha256: String,
}

impl DecodedPart {
    /// Decode canonical closed part JSON and verify size/hash before Store locking.
    pub(crate) fn parse(value: &Value, control: &mut dyn WorkControl) -> WorkResult<Self> {
        control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Clear)?;
        closed(value, &["sha256", "size_bytes", "hex"])?;
        let sha256 = value["sha256"].as_str().ok_or_else(Error::invalid)?;
        let size = unsigned(&value["size_bytes"])?;
        let hex = value["hex"].as_str().ok_or_else(Error::invalid)?;
        if !hash64(sha256)
            || size == 0
            || size > PART_BYTES
            || hex.len() != size.checked_mul(2).ok_or_else(capacity)?
        {
            return Err(Error::invalid().into());
        }
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(size).map_err(|_| capacity())?;
        for pair in hex.as_bytes().chunks_exact(2) {
            bytes.push(nibble(pair[0])? * 16 + nibble(pair[1])?);
        }
        control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
        if crate::hashing::sha256_hex(&bytes) != sha256 {
            return Err(Error::invalid().into());
        }
        Ok(Self { bytes, sha256: sha256.into() })
    }
}

/// Private stage state, unrelated to authoritative Operation/restore enums.
#[derive(Clone, Copy, PartialEq, Eq)]
enum StageState {
    /// Some exact ordinal bytes have not been received.
    Receiving,
    /// Every part exists; the whole raw hash and codec still need preparation.
    Ready,
    /// An off-lock lease owns this generation; mutations cannot replace bytes.
    Preparing,
    /// ROOT retained a complete preview; transport cannot revoke its authority.
    Prepared,
    /// Unconfirmed transport was discarded/expired; a held lease stays charged.
    Discarded,
}

impl StageState {
    /// Expose only the new closed transport vocabulary, never native write claims.
    fn wire(self) -> &'static str {
        match self {
            Self::Receiving => "receiving",
            Self::Ready => "ready",
            Self::Preparing => "preparing",
            Self::Prepared => "prepared",
            Self::Discarded => "discarded",
        }
    }
}

/// Original acknowledgment facts per ordinal; retries do not invent new counts.
#[derive(Clone, Copy)]
struct PartAck {
    /// Total unique completed parts immediately after this part was installed.
    completed: usize,
    /// Total exact bytes immediately after this part was installed.
    received: usize,
}

/// One same-session reservation, whose raw bytes have no destination authority.
struct Transfer {
    /// Complete validated raw-file descriptor; never rewritten by a retry.
    request: StageRequest,
    /// Immutable monotonic lifetime; no receive/read/prepare extends it.
    expires: Instant,
    /// Original bounded public timestamp, calculated by the trusted caller clock.
    expires_at: String,
    /// Exact preallocated raw buffer; a preparation lease shares this one allocation.
    raw: Option<Arc<Vec<u8>>>,
    /// At most320 primitive original-ack cells inside `METADATA_RESERVE`.
    parts: Vec<Option<PartAck>>,
    /// Actual distinct installed parts, not attempts or percentages.
    completed: usize,
    /// Exact distinct installed bytes, not declared size or completed preparation.
    received: usize,
    /// Monotonic retirement fence; late completion cannot revive discarded state.
    generation: u64,
    /// Current private phase; Prepared is not a write or accepted restore.
    state: StageState,
}

impl Transfer {
    /// Charge all reserved raw bytes until actual storage/leases are dropped.
    fn charge(&self) -> Result<usize> {
        METADATA_RESERVE
            .checked_add(if self.raw.is_some() { self.request.bytes } else { 0 })
            .ok_or_else(capacity)
    }

    /// Serialize bounded source-free transport observations from actual counters.
    fn status(&self, stage_id: &str) -> Value {
        json!({"stage_id":stage_id,"schema_version":"forge.workspace-index-bundle/4",
            "profile":"index-and-source-hex-staged","artifact_sha256":self.request.sha256,
            "artifact_size_bytes":self.request.bytes,"chunk_size_bytes":PART_BYTES,
            "chunk_count":self.request.parts,"received_chunk_count":self.completed,
            "received_bytes":self.received,"state":self.state.wire(),"expires_at":self.expires_at})
    }

    /// Release storage only when no preparation lease still holds its exact bytes.
    fn reclaim_raw(&mut self) {
        if matches!(self.state, StageState::Discarded | StageState::Prepared)
            && self.raw.as_ref().is_some_and(|raw| Arc::strong_count(raw) == 1)
        {
            self.raw = None;
        }
    }

    /// Retire a generation before reclaiming bytes; never affect prepared authority.
    fn discard(&mut self) -> Result<()> {
        if self.state == StageState::Prepared {
            return Err(Error::new(
                "operation-not-cancellable",
                "A prepared transfer cannot revoke its restore receipt.",
                false,
            ));
        }
        if self.state != StageState::Discarded {
            self.generation = self.generation.checked_add(1).ok_or_else(capacity)?;
            self.state = StageState::Discarded;
        }
        self.reclaim_raw();
        Ok(())
    }
}

/// Non-authorizing ticket for the already existing stage generation.
pub(crate) struct PreparationTicket {
    /// ROOT-generated stage identity; no project path or write capability.
    stage_id: String,
    /// Exact generation captured before off-lock validation.
    generation: u64,
}

impl PreparationTicket {
    /// Borrow this exact stage identity for method/path/body reservation binding, never authorization.
    pub(crate) fn stage_id(&self) -> &str {
        &self.stage_id
    }
}

/// Off-lock owner of one exact raw allocation; not Clone and no mutable byte API.
pub(crate) struct PreparationLease {
    /// Same-generation handoff information, independent of source authority.
    ticket: PreparationTicket,
    /// Full expected original-file facts copied from the admitted manifest.
    request: StageRequest,
    /// Shared same allocation; discard/expiry cannot prematurely release its charge.
    raw: Arc<Vec<u8>>,
}

impl PreparationLease {
    /// Verify complete EOF length/full SHA then strict Bundle4 grammar, off Store lock.
    pub(crate) fn decode(
        &self,
        control: &mut dyn WorkControl,
    ) -> WorkResult<super::source_bundles::DecodedSourceBundle> {
        control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Clear)?;
        if self.raw.len() != self.request.bytes
            || crate::hashing::sha256_hex(&self.raw) != self.request.sha256
        {
            return Err(Error::invalid().into());
        }
        control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
        super::staged_source_bundles::decode(&self.raw, control)
    }

    /// Borrow only this attempt's nonauthorizing identity for cooperative off-lock retirement checks.
    /// Final retention still requires consuming the lease and proving its raw handle dropped.
    pub(crate) fn ticket(&self) -> &PreparationTicket {
        &self.ticket
    }

    /// Consume the lease after validation/plan construction; release its raw handle first.
    pub(crate) fn into_ticket(self) -> PreparationTicket {
        self.ticket
    }
}

/// Bounded staging ledger embedded inside ROOT's one existing shared Store lock.
#[derive(Default)]
pub(crate) struct SourceTransfers {
    /// Complete current session stages, including held discarded generations.
    transfers: BTreeMap<String, Transfer>,
}

impl SourceTransfers {
    /// Report this field's exact conservative charge for ROOT's combined ledger.
    pub(crate) fn retained_bytes(&self) -> Result<usize> {
        self.transfers
            .values()
            .try_fold(0_usize, |sum, stage| sum.checked_add(stage.charge()?).ok_or_else(capacity))
    }

    /// Report one record per stage; parts are bounded cells, not separate replay records.
    pub(crate) fn retained_records(&self) -> usize {
        self.transfers.len()
    }

    /// Admit additional bytes/records against all current non-stage and stage usage.
    fn admit(&self, other: OtherPoolUsage, bytes: usize, records: usize) -> Result<()> {
        let total = self
            .retained_bytes()?
            .checked_add(other.bytes)
            .and_then(|sum| sum.checked_add(bytes))
            .ok_or_else(capacity)?;
        let count = self
            .retained_records()
            .checked_add(other.records)
            .and_then(|sum| sum.checked_add(records))
            .ok_or_else(capacity)?;
        if total > MAX_RETAINED_BYTES || count > MAX_RETAINED_RECORDS {
            return Err(capacity());
        }
        Ok(())
    }

    /// Reserve complete raw size and bounded metadata before allocation/insertion.
    /// The HTTP adapter owns exact create-request/key replay and checks the original deadline.
    pub(crate) fn create(
        &mut self,
        stage_id: &str,
        request: StageRequest,
        now: Instant,
        utc: DateTime<Utc>,
        other: OtherPoolUsage,
    ) -> Result<Value> {
        if !stage_id_valid(stage_id) || self.transfers.contains_key(stage_id) {
            return Err(Error::invalid());
        }
        let charge = request.bytes.checked_add(METADATA_RESERVE).ok_or_else(capacity)?;
        self.admit(other, charge, 1)?;
        let expires = now.checked_add(STAGE_LIFETIME).ok_or_else(capacity)?;
        let expires_at = utc
            .checked_add_signed(chrono::TimeDelta::seconds(600))
            .ok_or_else(Error::invalid)?
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let mut raw = Vec::new();
        raw.try_reserve_exact(request.bytes).map_err(|_| capacity())?;
        raw.resize(request.bytes, 0);
        let mut parts = Vec::new();
        parts.try_reserve_exact(request.parts).map_err(|_| capacity())?;
        parts.resize(request.parts, None);
        let stage = Transfer {
            request,
            expires,
            expires_at,
            raw: Some(Arc::new(raw)),
            parts,
            completed: 0,
            received: 0,
            generation: 0,
            state: StageState::Receiving,
        };
        let status = stage.status(stage_id);
        self.transfers.insert(stage_id.into(), stage);
        Ok(status)
    }

    /// Move one completely prepared receiving stage after final shared/replay admission.
    /// This contains no callback or fallible work after the map move.
    pub(crate) fn retain_created(&mut self, other: OtherPoolUsage, mut local: Self) -> Result<()> {
        if local.transfers.len() != 1
            || local.transfers.iter().any(|(id, stage)| {
                self.transfers.contains_key(id)
                    || stage.state != StageState::Receiving
                    || stage.completed != 0
                    || stage.received != 0
                    || stage.generation != 0
                    || stage.raw.as_ref().is_none_or(|raw| Arc::strong_count(raw) != 1)
            })
        {
            return Err(conflict());
        }
        self.admit(other, local.retained_bytes()?, local.retained_records())?;
        self.transfers.append(&mut local.transfers);
        Ok(())
    }

    /// Install one already decoded exact part, or return its original immutable ack.
    /// This invokes no `WorkControl` callback under ROOT's Store lock; dispatch fences first.
    pub(crate) fn put(
        &mut self,
        stage_id: &str,
        ordinal: usize,
        part: &DecodedPart,
        now: Instant,
    ) -> Result<Value> {
        let stage = self.active(stage_id, now)?;
        if ordinal >= stage.request.parts
            || matches!(stage.state, StageState::Prepared | StageState::Discarded)
        {
            return Err(conflict());
        }
        let start = ordinal.checked_mul(PART_BYTES).ok_or_else(capacity)?;
        let expected = (stage.request.bytes - start).min(PART_BYTES);
        if part.bytes.len() != expected {
            return Err(Error::invalid());
        }
        let end = start.checked_add(expected).ok_or_else(capacity)?;
        if let Some(ack) = stage.parts[ordinal] {
            let previous =
                stage.raw.as_ref().and_then(|raw| raw.get(start..end)).ok_or_else(conflict)?;
            if previous != part.bytes.as_slice() {
                return Err(conflict());
            }
            return Ok(part_ack(stage_id, ordinal, part, ack, &stage.expires_at));
        }
        if stage.state != StageState::Receiving {
            return Err(conflict());
        }
        let next_completed = stage.completed.checked_add(1).ok_or_else(capacity)?;
        let next_received = stage.received.checked_add(expected).ok_or_else(capacity)?;
        let raw = stage.raw.as_mut().and_then(Arc::get_mut).ok_or_else(conflict)?;
        raw.get_mut(start..end).ok_or_else(conflict)?.copy_from_slice(&part.bytes);
        let ack = PartAck { completed: next_completed, received: next_received };
        stage.parts[ordinal] = Some(ack);
        stage.completed = next_completed;
        stage.received = next_received;
        if stage.completed == stage.request.parts {
            stage.state = StageState::Ready;
        }
        Ok(part_ack(stage_id, ordinal, part, ack, &stage.expires_at))
    }

    /// Read actual same-session observations without extending their original expiry.
    pub(crate) fn status(&mut self, stage_id: &str, now: Instant) -> Result<Value> {
        Ok(self.active(stage_id, now)?.status(stage_id))
    }

    /// Retire unconfirmed transport only; any held raw handle remains fully charged.
    pub(crate) fn discard(&mut self, stage_id: &str, now: Instant) -> Result<Value> {
        self.active(stage_id, now)?.discard()?;
        Ok(json!({"stage_id":stage_id,"discarded":true}))
    }

    /// Lease a complete ready generation without hashing/parsing under Store lock.
    pub(crate) fn begin_preparation(
        &mut self,
        stage_id: &str,
        now: Instant,
    ) -> Result<PreparationLease> {
        let stage = self.active(stage_id, now)?;
        if stage.state != StageState::Ready {
            return Err(conflict());
        }
        let generation = stage.generation.checked_add(1).ok_or_else(capacity)?;
        let raw = Arc::clone(stage.raw.as_ref().ok_or_else(conflict)?);
        stage.generation = generation;
        stage.state = StageState::Preparing;
        Ok(PreparationLease {
            ticket: PreparationTicket { stage_id: stage_id.into(), generation: stage.generation },
            request: stage.request.clone(),
            raw,
        })
    }

    /// Retain through ROOT's atomic no-partial-mutation closure only after peak admission.
    /// Ticket consumption proves the caller dropped its raw lease before handoff.
    /// The added charge includes COMPLETE native plan/preview/ready replay, no optimistic credit.
    pub(crate) fn retain_prepared<T>(
        &mut self,
        ticket: &PreparationTicket,
        now: Instant,
        other: OtherPoolUsage,
        plan_bytes: usize,
        plan_records: usize,
        retain: impl FnOnce() -> Result<T>,
    ) -> Result<T> {
        self.check_ticket(ticket, now)?;
        self.admit(other, plan_bytes, plan_records)?;
        let stage = self.transfers.get_mut(&ticket.stage_id).ok_or_else(conflict)?;
        let result = retain()?;
        stage.state = StageState::Prepared;
        stage.reclaim_raw();
        Ok(result)
    }

    /// Release failed preparation back to unchanged ready bytes if not retired/expired.
    pub(crate) fn abandon_preparation(
        &mut self,
        ticket: &PreparationTicket,
        now: Instant,
    ) -> Result<()> {
        self.check_ticket(ticket, now)?;
        let stage = self.transfers.get_mut(&ticket.stage_id).ok_or_else(conflict)?;
        stage.state = StageState::Ready;
        Ok(())
    }

    /// Reclaim expired/unconfirmed transport only after all leases have actually dropped.
    /// Native accepted intents/reservations live outside this ledger and are never swept here.
    pub(crate) fn sweep(&mut self, now: Instant) -> Result<()> {
        for stage in self.transfers.values_mut() {
            if now >= stage.expires && stage.state != StageState::Prepared {
                stage.discard()?;
            }
            stage.reclaim_raw();
        }
        self.transfers.retain(|_, stage| now < stage.expires || stage.raw.is_some());
        Ok(())
    }

    /// Return a live same-session stage; expiry retires bytes but cannot revive authority.
    fn active(&mut self, stage_id: &str, now: Instant) -> Result<&mut Transfer> {
        let stage = self.transfers.get_mut(stage_id).ok_or_else(not_found)?;
        if now >= stage.expires {
            if stage.state != StageState::Prepared {
                stage.discard()?;
            }
            return Err(Error::new("receipt-expired", "The source transfer expired.", false));
        }
        Ok(stage)
    }

    /// Observe a generation during off-lock work without asserting that its raw lease has dropped.
    /// The controller maps explicit retirement to sticky typed cancellation, never invalid domain data.
    pub(crate) fn preparation_active(
        &self,
        ticket: &PreparationTicket,
        now: Instant,
    ) -> Result<bool> {
        let Some(stage) = self.transfers.get(&ticket.stage_id) else {
            return Ok(false);
        };
        if now >= stage.expires {
            return Err(Error::new("receipt-expired", "The source transfer expired.", false));
        }
        Ok(stage.state == StageState::Preparing && stage.generation == ticket.generation)
    }

    /// Check generation/lifetime/raw lease drop before any public/native retention.
    fn check_ticket(&self, ticket: &PreparationTicket, now: Instant) -> Result<()> {
        let stage = self.transfers.get(&ticket.stage_id).ok_or_else(not_found)?;
        if now >= stage.expires {
            return Err(Error::new("receipt-expired", "The source transfer expired.", false));
        }
        if stage.state != StageState::Preparing
            || stage.generation != ticket.generation
            || stage.raw.as_ref().is_none_or(|raw| Arc::strong_count(raw) != 1)
        {
            return Err(conflict());
        }
        Ok(())
    }
}

/// Serialize one original part acknowledgment without content or producer diagnostics.
fn part_ack(id: &str, ordinal: usize, part: &DecodedPart, ack: PartAck, expiry: &str) -> Value {
    json!({"stage_id":id,"chunk_ordinal":ordinal,"sha256":part.sha256,
        "size_bytes":part.bytes.len(),"received_chunk_count":ack.completed,
        "received_bytes":ack.received,"expires_at":expiry})
}

/// Require exactly all named fields, preserving upstream strict raw duplicate rejection.
fn closed(value: &Value, names: &[&str]) -> Result<()> {
    let object = value.as_object().ok_or_else(Error::invalid)?;
    if object.len() != names.len() || names.iter().any(|key| !object.contains_key(*key)) {
        return Err(Error::invalid());
    }
    Ok(())
}

/// Accept exact unsigned JSON integers fitting this platform, never floats/negatives.
fn unsigned(value: &Value) -> Result<usize> {
    value.as_u64().and_then(|n| usize::try_from(n).ok()).ok_or_else(Error::invalid)
}

/// Admit only canonical lowercase SHA256, with no normalization of supplied facts.
fn hash64(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Decode one lowercase ASCII hex nibble; uppercase/non-ASCII inputs reject whole.
fn nibble(byte: u8) -> Result<u8> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(Error::invalid()),
    }
}

/// Check a ROOT-generated bounded opaque stage identity; it never grants authority.
fn stage_id_valid(id: &str) -> bool {
    id.strip_prefix("bst_").is_some_and(|suffix| {
        (12..=80).contains(&suffix.len())
            && suffix.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    })
}

/// Return configured whole-bound failure without leaking allocation or source details.
fn capacity() -> Error {
    Error::new(
        "payload-too-large",
        "The complete source transfer exceeds available capacity.",
        false,
    )
}

/// Return a stable safe generation/idempotent-part conflict, never private byte details.
fn conflict() -> Error {
    Error::new(
        "version-conflict",
        "The source transfer changed or is not ready. Read its status.",
        true,
    )
}

/// Distinguish missing stage state from any accepted native restore outcome.
fn not_found() -> Error {
    Error::new("not-found", "The source transfer is not present in this session.", false)
}

#[cfg(test)]
/// Pure staging controls exercise actual foundation methods, never native authority.
mod tests {
    use super::*;
    use crate::workspace::preparation::{NoopControl, WorkError};
    use chrono::TimeZone as _;
    use std::cell::Cell;

    /// Build a closed exact original-file manifest from inert synthetic byte facts.
    fn manifest(raw: &[u8]) -> Value {
        json!({"schema_version":"forge.workspace-index-bundle/4",
            "profile":"index-and-source-hex-staged",
            "artifact_sha256":crate::hashing::sha256_hex(raw),"artifact_size_bytes":raw.len(),
            "chunk_size_bytes":PART_BYTES,"chunk_count":raw.len().div_ceil(PART_BYTES),
            "acknowledge_sensitive_metadata":true,"acknowledge_source_content":true})
    }

    /// Construct canonical original-byte transport without parsing or rewriting JSON.
    fn part(raw: &[u8]) -> DecodedPart {
        let mut hex = String::new();
        for byte in raw {
            use std::fmt::Write as _;
            write!(hex, "{byte:02x}").unwrap();
        }
        let value =
            json!({"sha256":crate::hashing::sha256_hex(raw),"size_bytes":raw.len(),"hex":hex});
        DecodedPart::parse(&value, &mut NoopControl).unwrap()
    }

    /// Create one stage with deterministic monotonic/wall clock facts and complete charge.
    fn stage(raw: &[u8]) -> (SourceTransfers, String, Instant) {
        let id = "bst_0123456789ab".to_string();
        let now = Instant::now();
        let mut store = SourceTransfers::default();
        store
            .create(
                &id,
                StageRequest::parse(&manifest(raw)).unwrap(),
                now,
                Utc.timestamp_opt(1_700_000_000, 0).single().unwrap(),
                OtherPoolUsage::default(),
            )
            .unwrap();
        (store, id, now)
    }

    /// Author a schema-shaped empty raw bundle without inventing physical captures.
    fn empty_bundle() -> Vec<u8> {
        let index = crate::workspace::index::Index {
            schema_version: "forge.workspace/2".into(),
            label: "empty fixture".into(),
            resources: Vec::new(),
        };
        let value = json!({"schema_version":"forge.workspace-index-bundle/4",
            "profile":"index-and-source-hex-staged","source_content_included":true,
            "index":index,"index_sha256":crate::hashing::sha256_hex(&index.bytes().unwrap()),
            "pins":[],"contents":[]});
        crate::workspace::contract::encode(&value, MAX_ARTIFACT_BYTES, false).unwrap()
    }

    /// Install every part in authorial order through the actual bounded PUT method.
    fn ready(raw: &[u8]) -> (SourceTransfers, String, Instant) {
        let (mut store, id, now) = stage(raw);
        for (ordinal, bytes) in raw.chunks(PART_BYTES).enumerate() {
            store.put(&id, ordinal, &part(bytes), now).unwrap();
        }
        (store, id, now)
    }

    /// Require safe typed complete refusals without diagnostics that echo byte payloads.
    fn error_code<T>(result: Result<T>, expected: &str) {
        match result {
            Err(error) => assert_eq!(error.code, expected),
            Ok(_) => panic!("expected whole typed refusal"),
        }
    }

    /// Exact integers/format/ack/count/unknown fields admit no partial manifest.
    #[test]
    fn declaration_is_closed_exact_and_canonical() {
        let original = manifest(b"{}");
        let mutations = [
            ("artifact_size_bytes", json!(0)),
            ("artifact_size_bytes", json!(1.5)),
            ("artifact_size_bytes", json!(MAX_ARTIFACT_BYTES + 1)),
            ("chunk_count", json!(0)),
            ("chunk_count", json!(321)),
            ("chunk_count", json!(2)),
            ("chunk_size_bytes", json!(1)),
            ("acknowledge_source_content", json!(false)),
            ("acknowledge_sensitive_metadata", json!(1)),
            ("schema_version", json!("forge.workspace-index-bundle/3")),
            ("profile", json!("index-and-source-hex")),
            ("artifact_sha256", json!("F".repeat(64))),
        ];
        for (key, value) in mutations {
            let mut candidate = original.clone();
            candidate[key] = value;
            error_code(StageRequest::parse(&candidate), "invalid-request");
        }
        let mut extra = original;
        extra["private-path"] = json!("synthetic");
        error_code(StageRequest::parse(&extra), "invalid-request");
    }

    /// The whole shared pool, including other records, fences allocation/insertion.
    #[test]
    fn shared_capacity_is_not_a_second_stage_pool() {
        let raw = b"{}";
        let request = StageRequest::parse(&manifest(raw)).unwrap();
        let now = Instant::now();
        let utc = Utc.timestamp_opt(1_700_000_000, 0).single().unwrap();
        for other in [
            OtherPoolUsage { bytes: MAX_RETAINED_BYTES, records: 0 },
            OtherPoolUsage { bytes: 0, records: MAX_RETAINED_RECORDS },
        ] {
            let mut store = SourceTransfers::default();
            error_code(
                store.create("bst_0123456789ab", request.clone(), now, utc, other),
                "payload-too-large",
            );
            assert_eq!(store.retained_records(), 0);
            assert_eq!(store.retained_bytes().unwrap(), 0);
        }
    }

    /// Reverse arrival and late identical retries preserve the original per-part ack facts.
    #[test]
    fn complete_out_of_order_and_identical_part_replay() {
        let mut raw = vec![b'a'; PART_BYTES];
        raw.extend_from_slice(b"tail");
        let (mut store, id, now) = stage(&raw);
        let last = store.put(&id, 1, &part(b"tail"), now).unwrap();
        assert_eq!(last["received_chunk_count"], 1);
        assert_eq!(last["received_bytes"], 4);
        store.put(&id, 0, &part(&raw[..PART_BYTES]), now).unwrap();
        assert_eq!(store.put(&id, 1, &part(b"tail"), now).unwrap(), last);
        let status = store.status(&id, now).unwrap();
        assert_eq!(status["state"], "ready");
        assert_eq!(status["received_bytes"], raw.len());
        assert_eq!(store.retained_records(), 1);
    }

    /// Changed bytes at an already admitted ordinal never replace prior exact bytes.
    #[test]
    fn changed_part_conflicts_without_counter_or_buffer_mutation() {
        let (mut store, id, now) = ready(b"first");
        let before = store.status(&id, now).unwrap();
        error_code(store.put(&id, 0, &part(b"other"), now), "version-conflict");
        assert_eq!(store.status(&id, now).unwrap(), before);
        let lease = store.begin_preparation(&id, now).unwrap();
        assert_eq!(lease.raw.as_slice(), b"first");
    }

    /// Canonical final sizes and ordinal membership refuse whole before mutation.
    #[test]
    fn missing_or_wrong_final_part_cannot_become_ready() {
        let raw = vec![b'x'; PART_BYTES + 1];
        let (mut store, id, now) = stage(&raw);
        error_code(store.put(&id, 2, &part(b"x"), now), "version-conflict");
        error_code(store.put(&id, 1, &part(b"xx"), now), "invalid-request");
        let status = store.status(&id, now).unwrap();
        assert_eq!(status["received_chunk_count"], 0);
        error_code(store.begin_preparation(&id, now), "version-conflict");
    }

    /// Actual closed part parsing rejects upperhex/size/hash/extra/float facts.
    #[test]
    fn part_decode_preserves_hash_and_canonical_exact_bytes() {
        let good = json!({"sha256":crate::hashing::sha256_hex(b"\0\xff\r\n"),"size_bytes":4,"hex":"00ff0d0a"});
        assert_eq!(DecodedPart::parse(&good, &mut NoopControl).unwrap().bytes, b"\0\xff\r\n");
        for (key, value) in [
            ("hex", json!("00FF0d0a")),
            ("size_bytes", json!(4.0)),
            ("hex", json!("00ff0d")),
            ("sha256", json!("0".repeat(64))),
            ("private-detail", json!("synthetic")),
        ] {
            let mut candidate = good.clone();
            candidate[key] = value;
            assert!(
                matches!(DecodedPart::parse(&candidate,&mut NoopControl),Err(WorkError::Failed(error)) if error.code=="invalid-request")
            );
        }
    }

    /// The declared whole SHA is verified off-lock and cannot be inferred from part hashes.
    #[test]
    fn whole_raw_hash_mismatch_is_not_ready_preview_success() {
        let raw = b"{}";
        let (mut store, id, now) = stage(raw);
        store.put(&id, 0, &part(b"[]"), now).unwrap();
        let lease = store.begin_preparation(&id, now).unwrap();
        assert!(
            matches!(lease.decode(&mut NoopControl),Err(WorkError::Failed(error)) if error.code=="invalid-request")
        );
    }

    /// A discarded lease remains charged until dropped; late native retention is fenced.
    #[test]
    fn cancel_during_off_lock_preparation_never_releases_held_bytes_early() {
        let (mut store, id, now) = ready(b"{}");
        let original = store.retained_bytes().unwrap();
        let lease = store.begin_preparation(&id, now).unwrap();
        store.discard(&id, now).unwrap();
        assert_eq!(store.retained_bytes().unwrap(), original);
        let ticket = lease.into_ticket();
        let called = Cell::new(false);
        error_code(
            store.retain_prepared(&ticket, now, OtherPoolUsage::default(), 0, 1, || {
                called.set(true);
                Ok(())
            }),
            "version-conflict",
        );
        assert!(!called.get());
        store.sweep(now).unwrap();
        assert_eq!(store.retained_bytes().unwrap(), METADATA_RESERVE);
        assert_eq!(store.status(&id, now).unwrap()["state"], "discarded");
    }

    /// Exact expiry equality stops new work and retains only actually held lease charges.
    #[test]
    fn original_expiry_boundary_is_not_extended_by_progress_or_reads() {
        let (mut store, id, now) = ready(b"{}");
        let expiry = now + STAGE_LIFETIME;
        let before =
            store.status(&id, expiry.checked_sub(Duration::from_nanos(1)).unwrap()).unwrap();
        assert_eq!(before["state"], "ready");
        let lease = store
            .begin_preparation(&id, expiry.checked_sub(Duration::from_nanos(1)).unwrap())
            .unwrap();
        error_code(store.status(&id, expiry), "receipt-expired");
        store.sweep(expiry).unwrap();
        assert_eq!(store.retained_bytes().unwrap(), METADATA_RESERVE + 2);
        let ticket = lease.into_ticket();
        error_code(store.abandon_preparation(&ticket, expiry), "receipt-expired");
        store.sweep(expiry).unwrap();
        assert_eq!(store.retained_records(), 0);
    }

    /// Simultaneous raw stage+plan admission refuses before the atomic retention closure.
    #[test]
    fn peak_charge_cannot_credit_raw_drop_before_complete_plan_retention() {
        let (mut store, id, now) = ready(b"{}");
        let ticket = store.begin_preparation(&id, now).unwrap().into_ticket();
        let called = Cell::new(false);
        error_code(
            store.retain_prepared(
                &ticket,
                now,
                OtherPoolUsage::default(),
                MAX_RETAINED_BYTES,
                1,
                || {
                    called.set(true);
                    Ok(())
                },
            ),
            "payload-too-large",
        );
        assert!(!called.get());
        assert_eq!(store.retained_bytes().unwrap(), METADATA_RESERVE + 2);
        store.retain_prepared(&ticket, now, OtherPoolUsage::default(), 1024, 1, || Ok(())).unwrap();
        assert_eq!(store.retained_bytes().unwrap(), METADATA_RESERVE);
        assert_eq!(store.status(&id, now).unwrap()["state"], "prepared");
        error_code(store.discard(&id, now), "operation-not-cancellable");
    }

    /// A ROOT atomic retain refusal cannot relabel or retire the raw stage.
    #[test]
    fn failed_atomic_retain_keeps_original_stage_for_same_generation_retry() {
        let (mut store, id, now) = ready(b"{}");
        let ticket = store.begin_preparation(&id, now).unwrap().into_ticket();
        error_code(
            store.retain_prepared(&ticket, now, OtherPoolUsage::default(), 1024, 1, || {
                Err::<(), _>(Error::invalid())
            }),
            "invalid-request",
        );
        assert_eq!(store.retained_bytes().unwrap(), METADATA_RESERVE + 2);
        store.abandon_preparation(&ticket, now).unwrap();
        assert_eq!(store.status(&id, now).unwrap()["state"], "ready");
    }

    /// Split escaped duplicate raw JSON survives transport and fails the strict whole codec.
    #[test]
    fn raw_json_duplicates_are_not_collapsed_by_transport() {
        let genuine = empty_bundle();
        assert!(super::super::staged_source_bundles::decode(&genuine, &mut NoopControl).is_ok());
        let body = String::from_utf8(genuine).unwrap();
        let body =
            body.replacen('{', r#"{"\u0073chema_version":"forge.workspace-index-bundle/4","#, 1);
        let mut raw = vec![b' '; PART_BYTES - 3];
        raw.extend_from_slice(body.as_bytes());
        let (mut store, id, now) = ready(&raw);
        let lease = store.begin_preparation(&id, now).unwrap();
        assert!(
            matches!(lease.decode(&mut NoopControl),Err(WorkError::Failed(error)) if error.code=="invalid-request")
        );
    }

    /// Complete raw transport decodes all genuine index facts before any ROOT native planning.
    #[test]
    fn genuine_complete_stage_has_no_parser_rewrite_or_project_authority() {
        let raw = empty_bundle();
        let (mut store, id, now) = ready(&raw);
        let lease = store.begin_preparation(&id, now).unwrap();
        let decoded = lease.decode(&mut NoopControl).unwrap();
        assert_eq!(decoded.index.schema_version, "forge.workspace/2");
        assert_eq!(decoded.files.len(), 0);
        assert_eq!(lease.raw.as_slice(), raw.as_slice());
    }

    /// Off-lock original checkpoint interruptions remain typed, never valid/invalid success.
    #[test]
    fn off_lock_parse_and_whole_hash_interruption_cannot_be_retained() {
        let raw = empty_bundle();
        let (mut store, id, now) = ready(&raw);
        let lease = store.begin_preparation(&id, now).unwrap();
        for visit in [1, 2, 3] {
            let mut control = crate::workspace::preparation::test_support::Recorder::at(
                Stage::PrepareDomain,
                visit,
            );
            assert!(matches!(lease.decode(&mut control), Err(WorkError::Interrupted(_))));
        }
        assert_eq!(store.status(&id, now).unwrap()["state"], "preparing");
    }

    /// Old tickets cannot retain or abandon a newer attempt after its raw lease drops.
    #[test]
    fn every_preparation_attempt_has_a_fresh_generation() {
        let (mut store, id, now) = ready(b"{}");
        let old = store.begin_preparation(&id, now).unwrap().into_ticket();
        store.abandon_preparation(&old, now).unwrap();
        let current = store.begin_preparation(&id, now).unwrap().into_ticket();
        assert_ne!(current.generation, old.generation);
        let called = Cell::new(false);
        error_code(
            store.retain_prepared(&old, now, OtherPoolUsage::default(), 0, 1, || {
                called.set(true);
                Ok(())
            }),
            "version-conflict",
        );
        error_code(store.abandon_preparation(&old, now), "version-conflict");
        assert!(!called.get());
        assert_eq!(store.status(&id, now).unwrap()["state"], "preparing");
        store
            .retain_prepared(&current, now, OtherPoolUsage::default(), 1024, 1, || Ok(()))
            .unwrap();
        assert_eq!(store.status(&id, now).unwrap()["state"], "prepared");
    }
}
