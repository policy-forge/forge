//! Seven exact admitted Authoring /3 hash profiles, returning plain non-authorizing data.
//!
//! Native membership, physical aliases, complete currentness and actual registration
//! ownership are absent here. No digest, plain roster or DTO can issue any such capability.
//! Every profile uses one caller ContractLedger/WorkControl and the unchanged shared fences.

use super::decode::{ContractError, ContractLedger};
use super::decode_v2::{checkpoint, phase};
use super::wire_v3::{
    ADAPTER, AbstentionRule, AuthorSeparation, ContextSnapshot, DOMAIN, Disposition, DomainV3,
    NativeModelV3, RequestedAction, ReviewItemV3, ReviewPolicy, SourceKindV3, SourcePinV3,
};
use crate::workspace::preparation::WorkControl;
use sha2::{Digest as _, Sha256};
use uuid::Uuid;

/// Exact fixed logical streaming state reserved before construction.
const STATE_BYTES: usize = 256;
/// Actual digest updates borrow at most 32 KiB under the same original controller.
const CHUNK: usize = 32 * 1024;
/// Finite ordinary profile ceilings are not renewed invocation allowances.
const SMALL_CAP: usize = 16_384;
/// Complete public/native ordinary profile stream ceiling.
const STREAM_CAP: usize = 1_048_576;
/// Complete actual-occurrence generation stream ceiling, independent of raw storage quotas.
const GENERATION_CAP: usize = 33_554_432;

/// One plain full native provenance row, not a native/capture/complete-roster proof.
pub(crate) struct NativeProvenance<'a> {
    /// Exact maintained role spelling, including native clause keys/Mapping ordinals.
    pub(crate) role: &'a str,
    /// Exact portable native project-root-relative label, never normalized for hashing.
    pub(crate) path: &'a str,
    /// Exact full raw hash; equality does not confer physical source ownership.
    pub(crate) raw_sha256: &'a str,
    /// Full original extent, not a projected serialization length.
    pub(crate) byte_length: u64,
}
/// Closed ordinary Auxiliary purpose labels; the real registry remains absent.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuxiliaryPurposeV3 {
    /// Whole current-operation locator occurrence, framed as 01.
    Locator,
    /// Negative test operand; complete current-operation framing refuses Init policy.
    #[cfg(test)]
    InitPolicy,
}
/// Plain full Auxiliary raw occurrence; no actual lease or registration can be constructed.
pub(crate) struct AuxiliaryOriginal<'a> {
    /// Exact ordinary purpose observation, without physical authority.
    pub(crate) purpose: AuxiliaryPurposeV3,
    /// Full actual raw digest operand.
    pub(crate) raw_sha256: &'a str,
    /// Full actual raw extent operand.
    pub(crate) byte_length: u64,
}
/// Plain response occurrence; exact duplicates stay separate and ordered in the hash.
pub(crate) struct ResponseOriginal<'a> {
    /// Canonical nonnil review response identity.
    pub(crate) response_id: &'a str,
    /// Full raw digest, never semantic reserialization.
    pub(crate) raw_sha256: &'a str,
    /// Full extent, including every original formatting byte.
    pub(crate) byte_length: u64,
}

/// Plain borrowed generation operands; copying references confers no native or physical authority.
#[derive(Clone, Copy)]
pub(crate) struct ClosureGenerationInputs<'a> {
    /// Exact complete native provenance digest operand.
    pub(crate) native: &'a str,
    /// Every complete declared eight-field source pin in its actual order.
    pub(crate) pins: &'a [SourcePinV3],
    /// Complete actual Queue raw digest operand.
    pub(crate) queue: &'a str,
    /// Complete actual Queue raw extent operand.
    pub(crate) queue_length: u64,
    /// Whole current-operation Auxiliary occurrence list, never a lease or registration.
    pub(crate) aux: &'a [AuxiliaryOriginal<'a>],
    /// Every actual response occurrence, including repeats, in its original order.
    pub(crate) responses: &'a [ResponseOriginal<'a>],
}

/// Private streaming encoder precharges every actual prefix, tag, frame and operand.
struct Stream<'a> {
    /// One original invocation ledger.
    ledger: &'a mut ContractLedger,
    /// Same accepted control; no deadline renewal or replacement.
    control: &'a mut dyn WorkControl,
    /// Actual running hash state; never a complete payload allocation.
    digest: Sha256,
    /// Complete encoded count, including prefix and every frame/tag.
    encoded: usize,
    /// Fixed profile ceiling, not an alternate invocation allowance.
    cap: usize,
}
impl<'a> Stream<'a> {
    /// Reserve streaming state before construction, then emit the exact NUL prefix.
    fn new(
        prefix: &[u8],
        cap: usize,
        ledger: &'a mut ContractLedger,
        control: &'a mut dyn WorkControl,
    ) -> Result<Self, ContractError> {
        ledger.derived(STATE_BYTES)?;
        let mut stream = Self { ledger, control, digest: Sha256::new(), encoded: 0, cap };
        stream.put(prefix)?;
        Ok(stream)
    }
    /// Precharge the whole encoded extent and each actual borrowed digest update.
    fn put(&mut self, bytes: &[u8]) -> Result<(), ContractError> {
        let next = self.encoded.checked_add(bytes.len()).ok_or_else(|| self.ledger.capacity())?;
        if next > self.cap {
            return Err(self.ledger.capacity());
        }
        self.ledger.visits(1)?;
        self.ledger.bytes(bytes.len())?;
        for chunk in bytes.chunks(CHUNK) {
            self.ledger.visits(1)?;
            checkpoint(self.ledger, self.control)?;
            self.digest.update(chunk);
        }
        self.encoded = next;
        Ok(())
    }
    /// Exact eight-byte unsigned little-endian frame.
    fn u64(&mut self, value: u64) -> Result<(), ContractError> {
        self.put(&value.to_le_bytes())
    }
    /// Convert real Rust cardinalities with checked representation before emission.
    fn count(&mut self, value: usize) -> Result<(), ContractError> {
        let value = u64::try_from(value).map_err(|_| self.ledger.capacity())?;
        self.u64(value)
    }
    /// Exact one-byte tag; no unknown-enum fallback is permitted.
    fn byte(&mut self, value: u8) -> Result<(), ContractError> {
        self.put(&[value])
    }
    /// Length-framed original UTF-8 bytes, with no normalization or JSON escapes.
    fn string(&mut self, value: &str) -> Result<(), ContractError> {
        self.count(value.len())?;
        self.put(value.as_bytes())
    }
    /// Null is 00; present empty is 01 followed by the empty string frame.
    fn optional_string(&mut self, value: Option<&str>) -> Result<(), ContractError> {
        match value {
            None => self.byte(0),
            Some(value) => {
                self.byte(1)?;
                self.string(value)
            }
        }
    }
    /// Decode exactly lowercase 64 hex to fixed stack bytes before the hash update.
    fn hash(&mut self, value: &str) -> Result<(), ContractError> {
        self.ledger.bytes(value.len())?;
        if value.len() != 64 {
            return Err(ContractError::Invalid);
        }
        self.ledger.visits(32)?;
        let mut decoded = [0_u8; 32];
        for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
            /// Accept exact lowercase hexadecimal digits without coercion.
            fn nibble(byte: u8) -> Option<u8> {
                match byte {
                    b'0'..=b'9' => Some(byte - b'0'),
                    b'a'..=b'f' => Some(byte - b'a' + 10),
                    _ => None,
                }
            }
            let high = nibble(pair[0]).ok_or(ContractError::Invalid)?;
            let low = nibble(pair[1]).ok_or(ContractError::Invalid)?;
            decoded[index] = high * 16 + low;
        }
        self.put(&decoded)
    }
    /// Exact complete declared string list in its already validated order.
    fn strings(&mut self, values: &[String]) -> Result<(), ContractError> {
        self.count(values.len())?;
        for value in values {
            self.ledger.visits(1)?;
            self.string(value)?;
        }
        Ok(())
    }
    /// Full eight-field pin in normative order; original generated UUID text survives.
    fn pin(&mut self, pin: &SourcePinV3) -> Result<(), ContractError> {
        self.ledger.visits(1)?;
        self.string(&pin.artifact_key)?;
        self.byte(kind_tag(pin.kind))?;
        self.hash(&pin.raw_sha256)?;
        self.u64(pin.byte_length)?;
        self.optional_string(pin.schema_identity.as_deref())?;
        self.string(&pin.validation_profile)?;
        self.optional_string(pin.native_model.map(NativeModelV3::as_str))?;
        self.optional_string(pin.native_root_uuid.as_deref())
    }
    /// Complete public pin order; source validation occurs before the stream opens.
    fn pins(&mut self, pins: &[SourcePinV3]) -> Result<(), ContractError> {
        self.count(pins.len())?;
        for pin in pins {
            self.pin(pin)?;
        }
        Ok(())
    }
    /// Native and review enums use fixed explicit contract tags only.
    fn dispositions(&mut self, values: &[Disposition]) -> Result<(), ContractError> {
        self.count(values.len())?;
        for value in values {
            self.byte(disposition_tag(*value))?;
        }
        Ok(())
    }
    /// Selected complete asserted policy, without evaluating quorum or native authority.
    fn policy(&mut self, policy: &ReviewPolicy) -> Result<(), ContractError> {
        self.string(&policy.key)?;
        self.count(policy.seats.len())?;
        for seat in &policy.seats {
            self.ledger.visits(1)?;
            self.string(&seat.role_key)?;
            self.u64(u64::from(seat.count))?;
        }
        self.count(policy.substitutions.len())?;
        for edge in &policy.substitutions {
            self.ledger.visits(1)?;
            self.string(&edge.seat_role)?;
            self.string(&edge.reviewer_key)?;
            self.string(&edge.asserted_role)?;
            self.string(&edge.reason_code)?;
        }
        match policy.abstention_rule {
            AbstentionRule::Nonapproving => self.byte(1)?,
        }
        self.strings(&policy.empty_abstention_reasons)?;
        match policy.author_separation {
            AuthorSeparation::DeclaredKeys => self.byte(1),
        }
    }
    /// Final digest state is ordinary data; the same caller is fenced before return.
    fn finish(self) -> Result<[u8; 32], ContractError> {
        checkpoint(self.ledger, self.control)?;
        Ok(self.digest.finalize().into())
    }
    /// Reserve all 64 output bytes and its fixed string representation before formatting.
    fn finish_hex(self) -> Result<String, ContractError> {
        self.ledger.derived(128)?;
        let bytes = self.finish()?;
        Ok(crate::hashing::lower_hex(&bytes))
    }
}
/// Fixed closed purpose tags; unknown source enums cannot fall back to a value.
fn kind_tag(kind: SourceKindV3) -> u8 {
    match kind {
        SourceKindV3::AuthorProject => 1,
        SourceKindV3::AuthoringPack => 2,
        SourceKindV3::GapReport => 3,
        SourceKindV3::ApplicabilityManifest => 4,
        SourceKindV3::Framework => 5,
        SourceKindV3::ResolvedCatalog => 6,
        SourceKindV3::MappingCollection => 7,
        SourceKindV3::HumanClause => 8,
        SourceKindV3::StoredPlan => 9,
    }
}
/// Exact complete disposition tags preserve dissent and withdrawal choices.
fn disposition_tag(value: Disposition) -> u8 {
    match value {
        Disposition::Approve => 1,
        Disposition::Reject => 2,
        Disposition::RequestChanges => 3,
        Disposition::Abstain => 4,
        Disposition::Superseded => 5,
    }
}
/// Charge complete scalar/string operands before each ordinary validation pass.
fn operands(values: &[&str], ledger: &mut ContractLedger) -> Result<(), ContractError> {
    ledger.visits(values.len())?;
    for value in values {
        ledger.bytes(value.len())?;
    }
    Ok(())
}
/// Exact lowercase raw SHA-256 spelling; decoding to raw32 remains a separate charged pass.
fn hash_operand(value: &str, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    operands(&[value], ledger)?;
    if value.len() != 64 || !value.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(ContractError::Invalid);
    }
    Ok(())
}
/// Exact common asserted ASCII token, admitted before its grammar scan.
fn key(value: &str, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    operands(&[value], ledger)?;
    super::validate::token(value, 128)
}
/// Preserve the maintained native private project-key grammar without normalization.
fn project_key(value: &str, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    let extent = value.len().checked_mul(3).ok_or_else(|| ledger.capacity())?;
    ledger.bytes(extent)?;
    if value.is_empty()
        || value.len() > 64
        || !value.as_bytes()[0].is_ascii_lowercase()
        || !value.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        || value.ends_with('-')
        || value.contains("--")
    {
        return Err(ContractError::Invalid);
    }
    Ok(())
}
/// Exact raw native RFC3339 spelling, including its original offset/fractional representation.
fn native_time(value: &str, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    let work = value.len().checked_mul(3).ok_or_else(|| ledger.capacity())?;
    ledger.bytes(work)?;
    ledger.derived(256)?;
    if value.len() > 64
        || value.trim() != value
        || chrono::DateTime::parse_from_rfc3339(value).is_err()
    {
        return Err(ContractError::Invalid);
    }
    Ok(())
}
/// Admit canonical review identity parsing/formatting; native UUID text uses a separate rule.
fn review_uuid(value: &str, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    let work = value.len().checked_mul(3).ok_or_else(|| ledger.capacity())?;
    ledger.bytes(work)?;
    ledger.visits(1)?;
    ledger.derived(128)?;
    super::validate::uuid(value)
}
/// Exact review UTC seconds, with complete parser/format scratch admitted before construction.
fn review_time(value: &str, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    let work = value.len().checked_mul(3).ok_or_else(|| ledger.capacity())?;
    ledger.bytes(work)?;
    ledger.derived(128)?;
    super::validate::time(value)
}
/// Parse the complete gap-free ordinary ordinal spelling; no key/path is constructed.
fn ordinal(value: &str, prefix: &str) -> Option<usize> {
    let rest = value.strip_prefix(prefix)?;
    if rest.is_empty()
        || rest.len() > 2
        || (rest.len() > 1 && rest.starts_with('0'))
        || !rest.bytes().all(|b| b.is_ascii_digit())
    {
        return None;
    }
    rest.parse().ok()
}
/// Admit prefix, decimal grammar and actual numeric parse before an ordinal read.
fn ordinal_work(value: &str, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    let work = value.len().checked_mul(3).ok_or_else(|| ledger.capacity())?;
    ledger.bytes(work)?;
    ledger.visits(1)
}
/// Exact closed static source tuple; selecting it does not inspect or approve native input.
fn pin_expectations(
    kind: SourceKindV3,
) -> (Option<&'static str>, Option<&'static str>, &'static str, u64) {
    match kind {
        SourceKindV3::AuthorProject => (
            Some("authoring:project"),
            Some("forge.author-project/1"),
            "forge.author-project-intrinsic/1",
            2_097_152,
        ),
        SourceKindV3::AuthoringPack => (
            Some("authoring:pack"),
            Some("forge.authoring-pack/1"),
            "forge.authoring-pack-intrinsic/1",
            2_097_152,
        ),
        SourceKindV3::GapReport => (
            Some("authoring:gap-report"),
            Some("forge.applicability-report/1"),
            "forge.authoring-gap-report-complete/1",
            10_485_760,
        ),
        SourceKindV3::ApplicabilityManifest => (
            Some("authoring:applicability"),
            Some("forge.applicability/1"),
            "forge.authoring-applicability-intrinsic/1",
            10_485_760,
        ),
        SourceKindV3::Framework => {
            (Some("authoring:framework"), None, "forge.authoring-framework-native/1", 10_485_760)
        }
        SourceKindV3::ResolvedCatalog => (
            Some("authoring:resolved"),
            None,
            "forge.authoring-resolved-catalog-native/1",
            10_485_760,
        ),
        SourceKindV3::MappingCollection => {
            (None, None, "forge.authoring-mapping-native/1", 10_485_760)
        }
        SourceKindV3::HumanClause => (None, None, "forge.authoring-clause-validated/1", 1_048_576),
        SourceKindV3::StoredPlan => (
            Some("authoring:plan"),
            Some("forge.authoring-plan/1"),
            "forge.authoring-plan-complete-equality/1",
            10_485_760,
        ),
    }
}
/// Fixed inert tuple expectations, independent of native parser/schema approval.
fn pin_shape(pin: &SourcePinV3, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    operands(
        &[
            &pin.artifact_key,
            &pin.validation_profile,
            pin.schema_identity.as_deref().unwrap_or(""),
            pin.native_root_uuid.as_deref().unwrap_or(""),
        ],
        ledger,
    )?;
    hash_operand(&pin.raw_sha256, ledger)?;
    let (expected_key, schema, profile, cap) = pin_expectations(pin.kind);
    if pin.byte_length == 0
        || pin.byte_length > cap
        || pin.schema_identity.as_deref() != schema
        || pin.validation_profile != profile
        || expected_key.is_some_and(|k| k != pin.artifact_key)
    {
        return Err(ContractError::Invalid);
    }
    let native = match pin.kind {
        SourceKindV3::Framework => {
            matches!(pin.native_model, Some(NativeModelV3::Catalog | NativeModelV3::Profile))
        }
        SourceKindV3::ResolvedCatalog => pin.native_model == Some(NativeModelV3::Catalog),
        SourceKindV3::MappingCollection => {
            pin.native_model == Some(NativeModelV3::MappingCollection)
        }
        _ => {
            if pin.native_model.is_some() || pin.native_root_uuid.is_some() {
                return Err(ContractError::Invalid);
            }
            false
        }
    };
    if matches!(
        pin.kind,
        SourceKindV3::Framework | SourceKindV3::ResolvedCatalog | SourceKindV3::MappingCollection
    ) {
        if !native {
            return Err(ContractError::Invalid);
        }
        let text = pin.native_root_uuid.as_deref().ok_or(ContractError::Invalid)?;
        if !(32..=45).contains(&text.len()) {
            return Err(ContractError::Invalid);
        }
        Uuid::parse_str(text).map_err(|_| ContractError::Invalid)?;
    }
    match pin.kind {
        SourceKindV3::MappingCollection => {
            ordinal_work(&pin.artifact_key, ledger)?;
            if ordinal(&pin.artifact_key, "authoring:mapping:").is_none() {
                Err(ContractError::Invalid)
            } else {
                Ok(())
            }
        }
        SourceKindV3::HumanClause => {
            ordinal_work(&pin.artifact_key, ledger)?;
            if ordinal(&pin.artifact_key, "authoring:clause:").is_none() {
                Err(ContractError::Invalid)
            } else {
                Ok(())
            }
        }
        _ => Ok(()),
    }
}
/// Full public array syntax, including lexicographic order and gap-free ordinal families.
/// This ordinary roster check cannot establish physical/native complete membership.
fn public_pins(pins: &[SourcePinV3], ledger: &mut ContractLedger) -> Result<(), ContractError> {
    if !(6..=100).contains(&pins.len()) {
        return Err(ContractError::Invalid);
    }
    ledger.derived(128)?;
    let mut kinds = [0_usize; 9];
    let mut total = 0_u64;
    let mut profile = false;
    for pair in pins.windows(2) {
        operands(&[&pair[0].artifact_key, &pair[1].artifact_key], ledger)?;
        if pair[0].artifact_key >= pair[1].artifact_key {
            return Err(ContractError::Invalid);
        }
    }
    for pin in pins {
        ledger.visits(1)?;
        pin_shape(pin, ledger)?;
        kinds[usize::from(kind_tag(pin.kind) - 1)] += 1;
        total = total.checked_add(pin.byte_length).ok_or_else(|| ledger.capacity())?;
        if pin.kind == SourceKindV3::Framework {
            profile = pin.native_model == Some(NativeModelV3::Profile);
        }
    }
    if total > 52_428_800 {
        return Err(ledger.capacity());
    }
    if [0, 1, 2, 3, 4, 8].into_iter().any(|i| kinds[i] != 1) || kinds[5] != usize::from(profile) {
        return Err(ContractError::Invalid);
    }
    for pin in pins {
        ledger.visits(1)?;
        operands(&[&pin.artifact_key], ledger)?;
        let family = match pin.kind {
            SourceKindV3::MappingCollection => Some(("authoring:mapping:", kinds[6])),
            SourceKindV3::HumanClause => Some(("authoring:clause:", kinds[7])),
            _ => None,
        };
        if let Some((prefix, count)) = family {
            ordinal_work(&pin.artifact_key, ledger)?;
            if ordinal(&pin.artifact_key, prefix).is_none_or(|i| i >= count) {
                return Err(ContractError::Invalid);
            }
        }
    }
    Ok(())
}
/// Exact public subject spelling; hashes remain data, not genuine source truth.
fn subject_operand(value: &str, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    operands(&[value], ledger)?;
    let hash = value.strip_prefix("authoring-plan:").ok_or(ContractError::Invalid)?;
    hash_operand(hash, ledger)
}
/// Plain item scalar/source-key operands; Queue registry/native binding remain separate.
fn item_operands(
    item: &ReviewItemV3,
    pins: &[SourcePinV3],
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    public_pins(pins, ledger)?;
    key(&item.key, ledger)?;
    operands(&[&item.adapter_version], ledger)?;
    subject_operand(&item.subject_id, ledger)?;
    if item.domain != DomainV3::AuthoringPlan
        || item.adapter_version != ADAPTER
        || item.source_keys.len() != pins.len()
    {
        return Err(ContractError::Invalid);
    }
    for (key, pin) in item.source_keys.iter().zip(pins) {
        ledger.visits(1)?;
        operands(&[key, &pin.artifact_key], ledger)?;
        if key != &pin.artifact_key {
            return Err(ContractError::Invalid);
        }
    }
    Ok(())
}
/// Exact sorted ordinary key lists; membership/native authority is not inferred.
fn ordered_keys(
    values: &[String],
    minimum: usize,
    maximum: usize,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    if !(minimum..=maximum).contains(&values.len()) {
        return Err(ContractError::Invalid);
    }
    for value in values {
        ledger.visits(1)?;
        key(value, ledger)?;
    }
    for pair in values.windows(2) {
        operands(&[&pair[0], &pair[1]], ledger)?;
        if pair[0] >= pair[1] {
            return Err(ContractError::Invalid);
        }
    }
    Ok(())
}
/// Complete selected asserted policy operands; this is not registry eligibility or quorum.
fn policy_operands(
    policy: &ReviewPolicy,
    item: &ReviewItemV3,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    key(&policy.key, ledger)?;
    key(&item.policy_key, ledger)?;
    operands(&[&policy.key, &item.policy_key], ledger)?;
    if policy.key != item.policy_key
        || !(1..=32).contains(&policy.seats.len())
        || policy.substitutions.len() > 3200
        || item.assignments.len() > 3200
    {
        return Err(ContractError::Invalid);
    }
    let mut seats = 0_usize;
    for row in &policy.seats {
        ledger.visits(1)?;
        key(&row.role_key, ledger)?;
        if !(1..=100).contains(&row.count) {
            return Err(ContractError::Invalid);
        }
        let count = usize::try_from(row.count).map_err(|_| ledger.capacity())?;
        seats = seats.checked_add(count).ok_or_else(|| ledger.capacity())?;
    }
    if seats > 100 {
        return Err(ContractError::Invalid);
    }
    for pair in policy.seats.windows(2) {
        operands(&[&pair[0].role_key, &pair[1].role_key], ledger)?;
        if pair[0].role_key >= pair[1].role_key {
            return Err(ContractError::Invalid);
        }
    }
    ordered_keys(&policy.empty_abstention_reasons, 0, 32, ledger)?;
    ordered_keys(&item.author_keys, 1, 100, ledger)?;
    for row in &policy.substitutions {
        ledger.visits(1)?;
        for text in [&row.seat_role, &row.reviewer_key, &row.asserted_role, &row.reason_code] {
            key(text, ledger)?;
        }
    }
    for pair in policy.substitutions.windows(2) {
        for row in pair {
            operands(&[&row.seat_role, &row.reviewer_key, &row.asserted_role], ledger)?;
        }
        if (&pair[0].seat_role, &pair[0].reviewer_key, &pair[0].asserted_role)
            >= (&pair[1].seat_role, &pair[1].reviewer_key, &pair[1].asserted_role)
        {
            return Err(ContractError::Invalid);
        }
    }
    for row in &item.assignments {
        ledger.visits(1)?;
        key(&row.reviewer_key, ledger)?;
        key(&row.role_key, ledger)?;
    }
    for pair in item.assignments.windows(2) {
        operands(
            &[&pair[0].reviewer_key, &pair[0].role_key, &pair[1].reviewer_key, &pair[1].role_key],
            ledger,
        )?;
        if (&pair[0].reviewer_key, &pair[0].role_key) >= (&pair[1].reviewer_key, &pair[1].role_key)
        {
            return Err(ContractError::Invalid);
        }
    }
    if let Some(due) = &item.due_at {
        review_time(due, ledger)?;
    }
    let allowed = [
        Disposition::Approve,
        Disposition::Reject,
        Disposition::RequestChanges,
        Disposition::Abstain,
        Disposition::Superseded,
    ];
    ledger.visits(item.allowed_dispositions.len())?;
    if item.allowed_dispositions != allowed {
        return Err(ContractError::Invalid);
    }
    Ok(())
}
/// Profile1: stable native project identity; reproducing it grants no authority.
pub(crate) fn subject_id(
    project: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<String, ContractError> {
    phase(ledger, control, |ledger, control| {
        project_key(project, ledger)?;
        let mut stream =
            Stream::new(b"forge.authoring-plan-review-subject-id/1\0", SMALL_CAP, ledger, control)?;
        stream.string("forge.author-project/1")?;
        stream.string("forge.authoring-plan/1")?;
        stream.string(project)?;
        stream.ledger.derived(79 + std::mem::size_of::<String>())?;
        let hash = stream.finish_hex()?;
        let mut id = String::with_capacity(79);
        id.push_str("authoring-plan:");
        id.push_str(&hash);
        Ok(id)
    })
}
/// Exact native provenance role grammar, without renaming native clause keys or ordinals.
fn native_role(value: &str, ledger: &mut ContractLedger) -> Result<SourceKindV3, ContractError> {
    // Six exact labels, two prefix checks and the decimal grammar/parse are bounded
    // by ten complete role byte passes; the clause suffix has its own key pass.
    let work = value.len().checked_mul(10).ok_or_else(|| ledger.capacity())?;
    ledger.bytes(work)?;
    ledger.visits(1)?;
    match value {
        "author-project" => Ok(SourceKindV3::AuthorProject),
        "authoring-pack" => Ok(SourceKindV3::AuthoringPack),
        "gap-report" => Ok(SourceKindV3::GapReport),
        "applicability-manifest" => Ok(SourceKindV3::ApplicabilityManifest),
        "framework" => Ok(SourceKindV3::Framework),
        "resolved-catalog" => Ok(SourceKindV3::ResolvedCatalog),
        _ if ordinal(value, "mapping-collection-").is_some() => Ok(SourceKindV3::MappingCollection),
        _ if value.starts_with("human-clause-") => {
            project_key(&value[13..], ledger)?;
            Ok(SourceKindV3::HumanClause)
        }
        _ => Err(ContractError::Invalid),
    }
}
/// Admit maintained plain portable-path validation; filesystem/native alias truth is absent.
fn native_path(value: &str, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    let work = value.len().checked_mul(20).ok_or_else(|| ledger.capacity())?;
    ledger.bytes(work)?;
    let scratch = value
        .len()
        .checked_mul(20)
        .and_then(|n| n.checked_add(4096))
        .ok_or_else(|| ledger.capacity())?;
    ledger.derived(scratch)?;
    crate::authoring::manifest::validate_local_path(
        "framed native route",
        std::path::Path::new(value),
    )
    .map_err(|_| ContractError::Invalid)
}
/// Ordinary complete native-row syntax/order/ASCII-label uniqueness, never physical proof.
fn native_rows(
    rows: &[NativeProvenance<'_>],
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    if !(5..=99).contains(&rows.len()) {
        return Err(ContractError::Invalid);
    }
    ledger.derived(128)?;
    let mut kinds = [0_usize; 9];
    let mut total = 0_u64;
    for row in rows {
        ledger.visits(1)?;
        let kind = native_role(row.role, ledger)?;
        kinds[usize::from(kind_tag(kind) - 1)] += 1;
        native_path(row.path, ledger)?;
        hash_operand(row.raw_sha256, ledger)?;
        let cap = match kind {
            SourceKindV3::AuthorProject | SourceKindV3::AuthoringPack => 2_097_152,
            SourceKindV3::HumanClause => 1_048_576,
            _ => 10_485_760,
        };
        if row.byte_length == 0 || row.byte_length > cap {
            return Err(ContractError::Invalid);
        }
        total = total.checked_add(row.byte_length).ok_or_else(|| ledger.capacity())?;
    }
    if total > 52_428_800 {
        return Err(ledger.capacity());
    }
    if kinds[..5].iter().any(|n| *n != 1) || kinds[5] > 1 {
        return Err(ContractError::Invalid);
    }
    for pair in rows.windows(2) {
        operands(
            &[
                pair[0].role,
                pair[0].path,
                pair[0].raw_sha256,
                pair[1].role,
                pair[1].path,
                pair[1].raw_sha256,
            ],
            ledger,
        )?;
        if (pair[0].role, pair[0].path, pair[0].raw_sha256, pair[0].byte_length)
            >= (pair[1].role, pair[1].path, pair[1].raw_sha256, pair[1].byte_length)
            || {
                operands(&[pair[0].role, pair[1].role], ledger)?;
                pair[0].role == pair[1].role
            }
        {
            return Err(ContractError::Invalid);
        }
    }
    for (i, row) in rows.iter().enumerate() {
        ordinal_work(row.role, ledger)?;
        if ordinal(row.role, "mapping-collection-").is_some_and(|n| n >= kinds[6]) {
            return Err(ContractError::Invalid);
        }
        for prior in &rows[..i] {
            ledger.visits(1)?;
            operands(&[prior.path, row.path], ledger)?;
            if prior.path.eq_ignore_ascii_case(row.path) {
                return Err(ContractError::Invalid);
            }
        }
    }
    Ok(())
}
/// Profile2: every native provenance row in exact maintained order, excluding stored plan.
pub(crate) fn native_provenance(
    rows: &[NativeProvenance<'_>],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<String, ContractError> {
    phase(ledger, control, |ledger, control| {
        native_rows(rows, ledger)?;
        let mut stream = Stream::new(
            b"forge.authoring-plan-review-native-inputs/1\0",
            STREAM_CAP,
            ledger,
            control,
        )?;
        stream.count(rows.len())?;
        for row in rows {
            stream.ledger.visits(1)?;
            stream.string(row.role)?;
            stream.string(row.path)?;
            stream.hash(row.raw_sha256)?;
            stream.u64(row.byte_length)?;
        }
        stream.finish_hex()
    })
}
/// Profile3: complete ordinary native/private stored-plan inputs; no full-plan oracle issued.
pub(crate) fn subject(
    project: &str,
    as_of: &str,
    native: &str,
    plan: &str,
    pins: &[SourcePinV3],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<String, ContractError> {
    phase(ledger, control, |ledger, control| {
        project_key(project, ledger)?;
        native_time(as_of, ledger)?;
        public_pins(pins, ledger)?;
        let mut stream =
            Stream::new(b"forge.authoring-plan-review-subject/1\0", STREAM_CAP, ledger, control)?;
        for value in
            [DOMAIN, ADAPTER, "forge.author-project/1", "forge.authoring-plan/1", project, as_of]
        {
            stream.string(value)?;
        }
        stream.hash(native)?;
        stream.hash(plan)?;
        stream.pins(pins)?;
        stream.finish_hex()
    })
}
/// Profile4: exact minimized complete context/pins; native prose never enters this stream.
pub(crate) fn context(
    pins: &[SourcePinV3],
    context: &ContextSnapshot,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<String, ContractError> {
    phase(ledger, control, |ledger, control| {
        public_pins(pins, ledger)?;
        ledger.visits(context.reason_codes.len())?;
        for reason in &context.reason_codes {
            operands(&[reason], ledger)?;
        }
        ledger.visits(context.related_subject_ids.len())?;
        for related in &context.related_subject_ids {
            operands(&[related], ledger)?;
        }
        if context.reason_codes.as_slice() != ["authoring-plan-current"]
            || !context.related_subject_ids.is_empty()
        {
            return Err(ContractError::Invalid);
        }
        let mut stream = Stream::new(b"forge.review-context/3\0", STREAM_CAP, ledger, control)?;
        stream.string(DOMAIN)?;
        stream.string(ADAPTER)?;
        stream.pins(pins)?;
        stream.strings(&context.reason_codes)?;
        stream.strings(&context.related_subject_ids)?;
        stream.finish_hex()
    })
}
/// Profile5: all selected asserted policy/author/assignment/deadline operands in actual order.
pub(crate) fn policy(
    pins: &[SourcePinV3],
    selected: &ReviewPolicy,
    item: &ReviewItemV3,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<String, ContractError> {
    phase(ledger, control, |ledger, control| {
        item_operands(item, pins, ledger)?;
        policy_operands(selected, item, ledger)?;
        let mut stream =
            Stream::new(b"forge.review-policy-binding/3\0", STREAM_CAP, ledger, control)?;
        stream.string(DOMAIN)?;
        stream.string(ADAPTER)?;
        stream.pins(pins)?;
        stream.policy(selected)?;
        stream.strings(&item.author_keys)?;
        stream.count(item.assignments.len())?;
        for row in &item.assignments {
            stream.ledger.visits(1)?;
            stream.string(&row.reviewer_key)?;
            stream.string(&row.role_key)?;
        }
        stream.optional_string(item.due_at.as_deref())?;
        stream.dispositions(&item.allowed_dispositions)?;
        stream.finish_hex()
    })
}
/// Profile6: SHA256's exact raw32 bytes are the existing review namespace `UUIDv5` name.
pub(crate) fn item_id(
    queue: &str,
    item: &ReviewItemV3,
    pins: &[SourcePinV3],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<String, ContractError> {
    phase(ledger, control, |ledger, control| {
        review_uuid(queue, ledger)?;
        item_operands(item, pins, ledger)?;
        let mut stream = Stream::new(b"forge.review-item/3\0", STREAM_CAP, ledger, control)?;
        for value in [queue, item.key.as_str(), DOMAIN, ADAPTER, item.subject_id.as_str()] {
            stream.string(value)?;
        }
        match item.requested_action {
            RequestedAction::ReReview => stream.byte(1)?,
        }
        stream.strings(&item.source_keys)?;
        stream.pins(pins)?;
        stream.hash(&item.subject_sha256)?;
        stream.hash(&item.context_sha256)?;
        stream.hash(&item.policy_sha256)?;
        stream.ledger.bytes(48)?;
        stream.ledger.derived(100)?;
        let digest = stream.finish()?;
        let namespace = Uuid::from_bytes([
            0x9f, 0x7c, 0x24, 0xf8, 0x56, 0x1d, 0x55, 0xa1, 0xa4, 0x3d, 0xfc, 0x4d, 0x81, 0x59,
            0xb4, 0x60,
        ]);
        Ok(Uuid::new_v5(&namespace, &digest).to_string())
    })
}
/// Profile7: complete ordinary raw occurrences in actual order, including duplicate responses.
/// This function cannot establish the actual owner, distinct-byte quotas or registry truth.
pub(crate) fn closure_generation(
    input: ClosureGenerationInputs<'_>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<String, ContractError> {
    let ClosureGenerationInputs { native, pins, queue, queue_length, aux, responses } = input;
    phase(ledger, control, |ledger, control| {
        public_pins(pins, ledger)?;
        if queue_length == 0
            || queue_length > 10_485_760
            || aux.len() != 1
            || responses.len() > 10_000
        {
            return Err(ContractError::Invalid);
        }
        for row in aux {
            ledger.visits(1)?;
            if row.purpose != AuxiliaryPurposeV3::Locator
                || row.byte_length == 0
                || row.byte_length > 1_048_576
            {
                return Err(ContractError::Invalid);
            }
        }
        for row in responses {
            ledger.visits(1)?;
            review_uuid(row.response_id, ledger)?;
            if row.byte_length == 0 || row.byte_length > 1_048_576 {
                return Err(ContractError::Invalid);
            }
        }
        let mut stream =
            Stream::new(b"forge.review-current-closure/3\0", GENERATION_CAP, ledger, control)?;
        stream.string(DOMAIN)?;
        stream.string(ADAPTER)?;
        stream.hash(native)?;
        stream.pins(pins)?;
        stream.hash(queue)?;
        stream.u64(queue_length)?;
        stream.count(aux.len())?;
        for row in aux {
            stream.ledger.visits(1)?;
            match row.purpose {
                AuxiliaryPurposeV3::Locator => stream.byte(1)?,
                #[cfg(test)]
                AuxiliaryPurposeV3::InitPolicy => return Err(ContractError::Invalid),
            }
            stream.hash(row.raw_sha256)?;
            stream.u64(row.byte_length)?;
        }
        stream.count(responses.len())?;
        for row in responses {
            stream.ledger.visits(1)?;
            stream.string(row.response_id)?;
            stream.hash(row.raw_sha256)?;
            stream.u64(row.byte_length)?;
        }
        stream.finish_hex()
    })
}

#[cfg(test)]
#[path = "hash_v3_tests.rs"]
mod tests;

/// Validate complete ordinary SourcePin/3 purpose and roster syntax without hashing.
/// Successful plain syntax never establishes native membership or currentness.
pub(crate) fn validate_public_pins(
    pins: &[SourcePinV3],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    phase(ledger, control, |ledger, _| public_pins(pins, ledger))
}
