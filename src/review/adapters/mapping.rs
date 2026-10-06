//! Captured Mapping re-review facts over a sealed Approved/current source closure.
//!
//! The capture factory retains the actual complete originals and verifies the
//! maintained native product and recorded lifecycle tuple before sealing.
//! Caller Values, hashes, labels or schema validity alone confer no authority.

use std::io::{self, Write};

use serde::{Serialize, Serializer, ser::SerializeMap};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::review::decode::{ContractError, ContractLedger, Decoded};
use crate::review::mapping_capture::ApprovedMappingClosure;
use crate::review::wire::{ContextSnapshot, Domain, QueueDocument, RequestedAction, SourcePin};
use crate::workspace::preparation::WorkControl;

/// Complete native-map count across all consumed Mapping groups.
const MAX_MAPS: usize = 10_000;
/// Compact subject payload ceiling; repeated bytes also consume the shared ledger.
const MAX_SUBJECT_PAYLOAD: usize = 10 * 1024 * 1024;
/// New domain-separated fingerprint profile; this is not native UUID derivation.
const SUBJECT_PREFIX: &[u8] = b"forge.mapping-review-subject/1\0";

/// One minimized selected subject, constructible only by this captured adapter.
pub(crate) struct MappingFact<'a> {
    /// Exact original native map UUID spelling, never inferred from a locator.
    subject_id: &'a str,
    /// Full sensitive subject/header fingerprint; prose is hashed but not exported.
    subject_sha256: String,
    /// Fixed codes only; no rationale, path, href, reviewer name or native text.
    context: ContextSnapshot,
}

impl MappingFact<'_> {
    /// Borrow the actual selected native UUID.
    pub(crate) fn subject_id(&self) -> &str {
        self.subject_id
    }
    /// Borrow the complete domain-separated sensitive subject binding.
    pub(crate) fn subject_sha256(&self) -> &str {
        &self.subject_sha256
    }
    /// Borrow the bounded default snapshot without source content.
    pub(crate) fn context(&self) -> &ContextSnapshot {
        &self.context
    }
}

/// Minimized facts borrowing the real complete Approved/current capture owner.
pub(crate) struct PreparedMappingReview<'a> {
    /// Sole authority-bearing input is an opaque Root-issued captured closure.
    closure: &'a ApprovedMappingClosure,
    /// All selected facts, sorted by exact original native subject ID.
    facts: Vec<Option<MappingFact<'a>>>,
    /// Complete validated native-map denominator before selection.
    complete_maps: usize,
}

impl PreparedMappingReview<'_> {
    /// Borrow exact whole captured source/native/lifecycle pins; no filtered set.
    pub(crate) fn source_pins(&self) -> &[SourcePin] {
        self.closure.source_pins()
    }
    /// Borrow every selected fact; callers cannot construct replacement proofs.
    pub(crate) fn facts(&self) -> impl Iterator<Item = &MappingFact<'_>> {
        self.facts.iter().filter_map(Option::as_ref)
    }
    /// Return the actual selected cardinality before caller fact-vector admission.
    pub(crate) fn selected_count(&self) -> usize {
        self.facts.len()
    }
    /// Return complete native maps, distinct from explicit selected item count.
    pub(crate) fn complete_maps(&self) -> usize {
        self.complete_maps
    }
    /// Borrow the real sealed current-source owner for the Root merge orchestrator.
    pub(crate) fn closure(&self) -> &ApprovedMappingClosure {
        self.closure
    }
    /// Recheck actual originals/identity/root under the same budget and control.
    pub(crate) fn verify_inputs(
        &self,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        ledger.bound(|ledger| self.closure.verify_inputs(ledger, control))
    }

    /// Compare complete decoded queue subjects/pins; policy validation is separate.
    ///
    /// This does not publish, merge votes or replace the final actual original fence.
    pub(crate) fn bind_queue(
        &self,
        queue: &Decoded<'_, QueueDocument>,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        ledger.bound(|ledger| {
            ledger.checkpoint(control)?;
            let queue = queue.document();
            charge_pins(&queue.source_pins, ledger)?;
            charge_pins(self.source_pins(), ledger)?;
            if queue.source_pins != self.source_pins() || queue.items.len() != self.facts.len() {
                return Err(ContractError::Binding);
            }
            ledger.derived(self.facts.len())?;
            let mut seen = vec![false; self.facts.len()];
            for item in &queue.items {
                ledger.checkpoint(control)?;
                let (index, fact) = self.find(&item.subject_id, ledger)?;
                if seen[index] {
                    return Err(ContractError::Binding);
                }
                seen[index] = true;
                ledger.visits(1)?;
                let extent = item
                    .adapter_version
                    .len()
                    .checked_add("forge.mapping-review/1".len())
                    .and_then(|value| value.checked_add(item.subject_sha256.len()))
                    .and_then(|value| value.checked_add(fact.subject_sha256.len()))
                    .ok_or(ContractError::Capacity)?;
                ledger.bytes(extent)?;
                charge_contexts(&item.context, &fact.context, ledger)?;
                if item.domain != Domain::MappingAssertion
                    || item.adapter_version != "forge.mapping-review/1"
                    || item.requested_action != RequestedAction::ReReview
                    || item.subject_sha256 != fact.subject_sha256
                    || item.context != fact.context
                {
                    return Err(ContractError::Binding);
                }
                ledger.visits(self.source_pins().len())?;
                ledger.bytes(
                    self.source_pins().len().checked_mul(256).ok_or(ContractError::Capacity)?,
                )?;
                if item.source_keys.len() != self.source_pins().len()
                    || item
                        .source_keys
                        .iter()
                        .zip(self.source_pins())
                        .any(|(key, pin)| key != &pin.artifact_key)
                {
                    return Err(ContractError::Binding);
                }
            }
            ledger.checkpoint(control)
        })
    }

    /// Search the pre-admitted sorted native-ID registry without a repeated full scan.
    fn find(
        &self,
        id: &str,
        ledger: &mut ContractLedger,
    ) -> Result<(usize, &MappingFact<'_>), ContractError> {
        let mut low = 0;
        let mut high = self.facts.len();
        while low < high {
            ledger.visits(1)?;
            ledger.bytes(id.len().checked_add(36).ok_or(ContractError::Capacity)?)?;
            let middle = low + (high - low) / 2;
            let fact = self.facts[middle].as_ref().ok_or(ContractError::Binding)?;
            match fact.subject_id.cmp(id) {
                std::cmp::Ordering::Less => low = middle + 1,
                std::cmp::Ordering::Greater => high = middle,
                std::cmp::Ordering::Equal => return Ok((middle, fact)),
            }
        }
        Err(ContractError::Binding)
    }
}

/// Admit explicit selection and derive facts only from the sealed original native tree.
///
/// The sealed input has consumed the complete maintained native comparison,
/// source/target/companion inventories and latest recorded approval/current tuple.
/// This adapter derives selected facts without opening files or promoting approval.
pub(crate) fn prepare<'a>(
    closure: &'a ApprovedMappingClosure,
    selected: &[&str],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<PreparedMappingReview<'a>, ContractError> {
    ledger.bound(|ledger| {
        ledger.checkpoint(control)?;
        if selected.is_empty() || selected.len() > MAX_MAPS {
            return Err(ContractError::Invalid);
        }
        ledger.visits(selected.len())?;
        // Reject original width before parser/string scans; this lends no authority.
        if selected.iter().any(|id| id.len() != 36) {
            return Err(ContractError::Invalid);
        }
        ledger.bytes(selected.len().checked_mul(72).ok_or(ContractError::Capacity)?)?;
        for id in selected {
            crate::review::validate::uuid(id)?;
        }
        if selected.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(ContractError::Invalid);
        }
        closure.verify_inputs(ledger, control)?;
        let native =
            closure.native_value().get("mapping-collection").ok_or(ContractError::Binding)?;
        let root_id = native.get("uuid").and_then(Value::as_str).ok_or(ContractError::Binding)?;
        let metadata = native.get("metadata").ok_or(ContractError::Binding)?;
        let provenance = native.get("provenance").ok_or(ContractError::Binding)?;
        let groups =
            native.get("mappings").and_then(Value::as_array).ok_or(ContractError::Binding)?;
        ledger.visits(groups.len())?;
        let total = groups.iter().try_fold(0_usize, |count, group| {
            count
                .checked_add(
                    group
                        .get("maps")
                        .and_then(Value::as_array)
                        .ok_or(ContractError::Binding)?
                        .len(),
                )
                .ok_or(ContractError::Capacity)
        })?;
        if total == 0 {
            return Err(ContractError::Binding);
        }
        if total > MAX_MAPS {
            return Err(ContractError::Capacity);
        }
        ledger.derived(selected.len().checked_mul(128).ok_or(ContractError::Capacity)?)?;
        let mut facts = Vec::new();
        facts.resize_with(selected.len(), || None);
        for group in groups {
            for map in group.get("maps").and_then(Value::as_array).ok_or(ContractError::Binding)? {
                ledger.checkpoint(control)?;
                ledger.visits(1)?;
                ledger.bytes(72)?;
                let id = map.get("uuid").and_then(Value::as_str).ok_or(ContractError::Binding)?;
                crate::review::validate::uuid(id)?;
                // The sealed constructor proves ALL original native UUIDs unique before issuance.
                // Account the whole binary-search worst case before comparisons.
                ledger.visits(15)?;
                ledger.bytes(15 * 72)?;
                let Ok(index) = selected.binary_search(&id) else {
                    continue;
                };
                if facts[index].is_some() {
                    return Err(ContractError::Binding);
                }
                let relationship = map
                    .get("relationship")
                    .and_then(Value::as_str)
                    .ok_or(ContractError::Binding)?;
                let reason = relationship_reason(relationship)?;
                let encoding = SubjectEncoding {
                    adapter_version: "forge.mapping-review/1",
                    collection_uuid: root_id,
                    metadata,
                    provenance,
                    mapping: MappingHeader(group),
                    assertion: map,
                };
                let subject_sha256 = subject_hash(&encoding, ledger, control)?;
                ledger.derived(128)?;
                let context = ContextSnapshot {
                    reason_codes: vec!["mapping-re-review".into(), reason.into()],
                    related_subject_ids: Vec::new(),
                };
                facts[index] = Some(MappingFact { subject_id: id, subject_sha256, context });
            }
        }
        ledger.visits(facts.len())?;
        if facts.iter().any(Option::is_none) {
            return Err(ContractError::Binding);
        }
        ledger.checkpoint(control)?;
        Ok(PreparedMappingReview { closure, facts, complete_maps: total })
    })
}

/// Complete sensitive subject fields in this exact versioned serializer order.
#[derive(Serialize)]
struct SubjectEncoding<'a> {
    /// Exact reviewed native adapter profile.
    adapter_version: &'static str,
    /// Original collection UUID, not a reserialized typed identity.
    collection_uuid: &'a str,
    /// All original metadata, including sensitive fields hashed but not shown.
    metadata: &'a Value,
    /// All original provenance and native reviewer declarations.
    provenance: &'a Value,
    /// Complete original mapping descriptor except its full maps array.
    mapping: MappingHeader<'a>,
    /// Complete actual selected native assertion, including notes and qualifiers.
    assertion: &'a Value,
}

/// Borrow all original mapping fields except the separately selected maps array.
struct MappingHeader<'a>(&'a Value);

impl Serialize for MappingHeader<'_> {
    /// Preserve complete keys/values without building a cloned descriptor object.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let object = self
            .0
            .as_object()
            .ok_or_else(|| serde::ser::Error::custom("invalid mapping descriptor"))?;
        if !object.contains_key("maps") {
            return Err(serde::ser::Error::custom("missing mapping array"));
        }
        let mut map = serializer.serialize_map(Some(object.len() - 1))?;
        for (key, value) in object {
            if key != "maps" {
                map.serialize_entry(key, value)?;
            }
        }
        map.end()
    }
}

/// Nonretaining digest sink with explicit payload/work/control admission.
struct HashSink<'a> {
    /// Existing SHA256 state, containing no retained source output.
    hash: Sha256,
    /// Exact compact payload length; domain prefix is charged separately.
    length: usize,
    /// Exact payload ceiling checked before each digest update.
    limit: usize,
    /// Shared monotonic caller-owned complete work/storage ledger.
    ledger: &'a mut ContractLedger,
    /// Same actual accepted-deadline and sticky stop source.
    control: &'a mut dyn WorkControl,
    /// Fixed actual writer/control refusal; never source-bearing IO prose.
    failure: Option<ContractError>,
}

impl Write for HashSink<'_> {
    /// Admit the whole supplied compact chunk before hashing, without source retention.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.failure.is_some() {
            return Err(io::Error::other("bounded subject encoding refused"));
        }
        let result = (|| {
            self.ledger.checkpoint(self.control)?;
            let next = self
                .length
                .checked_add(bytes.len())
                .filter(|value| *value <= self.limit)
                .ok_or(ContractError::Capacity)?;
            self.ledger.bytes(bytes.len())?;
            self.hash.update(bytes);
            self.length = next;
            Ok(())
        })();
        if let Err(error) = result {
            self.failure = Some(error);
            return Err(io::Error::other("bounded subject encoding refused"));
        }
        Ok(bytes.len())
    }
    /// Complete the same cooperative stop check; there is no external writer to flush.
    fn flush(&mut self) -> io::Result<()> {
        if self.failure.is_some() {
            return Err(io::Error::other("bounded subject encoding refused"));
        }
        self.ledger.checkpoint(self.control).map_err(|error| {
            self.failure = Some(error);
            io::Error::other("bounded subject encoding refused")
        })
    }
}

/// Hash complete borrowed subject fields; never serialize to an unbounded temporary Vec.
fn subject_hash(
    value: &impl Serialize,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<String, ContractError> {
    ledger.bytes(SUBJECT_PREFIX.len())?;
    let mut hash = Sha256::new();
    hash.update(SUBJECT_PREFIX);
    let mut sink =
        HashSink { hash, length: 0, limit: MAX_SUBJECT_PAYLOAD, ledger, control, failure: None };
    if serde_json::to_writer(&mut sink, value).is_err() {
        return Err(sink.failure.unwrap_or(ContractError::Invalid));
    }
    sink.ledger.checkpoint(sink.control)?;
    sink.ledger.derived(64)?;
    Ok(crate::hashing::lower_hex(&sink.hash.finalize()))
}

/// Precharge both complete caller/native context extents before structural equality.
fn charge_contexts(
    left: &ContextSnapshot,
    right: &ContextSnapshot,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    let mut bytes = 0_usize;
    for context in [left, right] {
        let rows = context
            .reason_codes
            .len()
            .checked_add(context.related_subject_ids.len())
            .ok_or(ContractError::Capacity)?;
        ledger.visits(rows)?;
        for text in context.reason_codes.iter().chain(&context.related_subject_ids) {
            bytes = bytes.checked_add(text.len()).ok_or(ContractError::Capacity)?;
        }
    }
    ledger.bytes(bytes)
}

/// Meter complete pin equality extents before comparison, including both callers separately.
fn charge_pins(pins: &[SourcePin], ledger: &mut ContractLedger) -> Result<(), ContractError> {
    ledger.visits(pins.len())?;
    let bytes = pins.iter().try_fold(0_usize, |count, pin| {
        [
            pin.artifact_key.len(),
            pin.raw_sha256.len(),
            pin.schema_identity.len(),
            pin.native_root_uuid.as_ref().map_or(0, String::len),
            64,
        ]
        .into_iter()
        .try_fold(count, |total, amount| total.checked_add(amount).ok_or(ContractError::Capacity))
    })?;
    ledger.bytes(bytes)
}

/// Project only the native relationship vocabulary into a bounded fixed reason.
fn relationship_reason(relationship: &str) -> Result<&'static str, ContractError> {
    match relationship {
        "equivalent-to" => Ok("relationship-equivalent-to"),
        "equal-to" => Ok("relationship-equal-to"),
        "subset-of" => Ok("relationship-subset-of"),
        "superset-of" => Ok("relationship-superset-of"),
        "intersects-with" => Ok("relationship-intersects-with"),
        "no-relationship" => Ok("relationship-no-relationship"),
        _ => Err(ContractError::Binding),
    }
}

#[cfg(test)]
/// Structural encoding controls; these Values supply no approval/capture proof.
mod tests {
    use super::*;
    use crate::workspace::preparation::{Interruption, NoopControl, Stage, test_support::Recorder};
    use serde_json::json;

    /// Bounded inert native-shaped fixture for hashing, not a native/schema approval.
    fn fixture() -> Value {
        json!({"uuid":"11111111-1111-4111-8111-111111111111",
            "metadata":{"title":"private title","version":"1"},
            "provenance":{"mapping-description":"private rationale"},
            "mappings":[{"uuid":"22222222-2222-4222-8222-222222222222",
                "mapping-description":"group rationale","props":[{"name":"mapping-key","value":"group"}],
                "maps":[{"uuid":"33333333-3333-4333-8333-333333333333",
                    "relationship":"equivalent-to","sources":[{"type":"control","id-ref":"a"}],
                    "targets":[{"type":"control","id-ref":"b"}],"remarks":"sensitive private note"},
                    {"uuid":"44444444-4444-4444-8444-444444444444","relationship":"no-relationship"}]}]})
    }

    /// Run the actual nonretaining encoding under its real shared structural ledger.
    fn digest(value: &Value) -> String {
        let group = &value["mappings"][0];
        let encoding = SubjectEncoding {
            adapter_version: "forge.mapping-review/1",
            collection_uuid: value["uuid"].as_str().unwrap(),
            metadata: &value["metadata"],
            provenance: &value["provenance"],
            mapping: MappingHeader(group),
            assertion: &group["maps"][0],
        };
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        ledger.bound(|ledger| subject_hash(&encoding, ledger, &mut control)).unwrap()
    }

    /// Equivalent formatting/key order produces the same admitted typed subject digest.
    #[test]
    fn complete_subject_hash_is_typed_and_deterministic() {
        let value = fixture();
        let original = digest(&value);
        let raw = serde_json::to_string_pretty(&value).unwrap();
        let reordered: Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(original, digest(&reordered));
        assert_eq!(original.len(), 64);
        assert!(
            original.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        );
    }

    /// Native metadata, provenance and complete descriptor edits bind the subject anew.
    #[test]
    fn header_metadata_and_provenance_are_not_omitted() {
        let base = fixture();
        let original = digest(&base);
        for pointer in [
            "/metadata/title",
            "/provenance/mapping-description",
            "/mappings/0/mapping-description",
            "/mappings/0/props/0/value",
        ] {
            let mut changed = base.clone();
            *changed.pointer_mut(pointer).unwrap() =
                Value::String("changed private original".into());
            assert_ne!(original, digest(&changed), "omitted complete subject field {pointer}");
        }
    }

    /// Complete selected assertion rationale and native endpoint arrays affect the digest.
    #[test]
    fn selected_rationale_and_each_endpoint_are_bound() {
        let base = fixture();
        let original = digest(&base);
        for pointer in [
            "/mappings/0/maps/0/remarks",
            "/mappings/0/maps/0/sources/0/id-ref",
            "/mappings/0/maps/0/targets/0/id-ref",
        ] {
            let mut changed = base.clone();
            *changed.pointer_mut(pointer).unwrap() =
                Value::String("different exact native fact".into());
            assert_ne!(original, digest(&changed));
        }
    }

    /// Unselected maps are excluded from item fingerprint but still belong to whole raw pins.
    #[test]
    fn unselected_map_does_not_replace_whole_closure_binding() {
        let base = fixture();
        let original = digest(&base);
        let mut changed = base.clone();
        changed["mappings"][0]["maps"][1]["relationship"] = Value::String("equal-to".into());
        assert_eq!(original, digest(&changed));
        assert_ne!(serde_json::to_vec(&base).unwrap(), serde_json::to_vec(&changed).unwrap());
    }

    /// Collection and selected native UUID identities cannot be replaced by an href/hash.
    #[test]
    fn actual_native_identity_is_in_sensitive_fingerprint() {
        let base = fixture();
        let original = digest(&base);
        for pointer in ["/uuid", "/mappings/0/maps/0/uuid"] {
            let mut changed = base.clone();
            *changed.pointer_mut(pointer).unwrap() =
                Value::String("55555555-5555-4555-8555-555555555555".into());
            assert_ne!(original, digest(&changed));
        }
    }

    /// Descriptor serialization borrows complete fields and removes only the maps member.
    #[test]
    fn descriptor_removal_oracle_keeps_all_other_fields() {
        let value = fixture();
        let group = &value["mappings"][0];
        let encoded = serde_json::to_value(MappingHeader(group)).unwrap();
        let mut expected = group.clone();
        expected.as_object_mut().unwrap().remove("maps");
        assert_eq!(encoded, expected);
        assert!(serde_json::to_vec(&MappingHeader(&json!({"uuid":"x"}))).is_err());
    }

    /// Domain separation and complete compact fields are independently reproduced in a fixture.
    #[test]
    fn digest_matches_explicit_versioned_compact_payload() {
        let value = fixture();
        let group = &value["mappings"][0];
        let mut header = group.clone();
        header.as_object_mut().unwrap().remove("maps");
        let fields = format!(
            "{{\"adapter_version\":\"forge.mapping-review/1\",\"collection_uuid\":{},\"metadata\":{},\"provenance\":{},\"mapping\":{},\"assertion\":{}}}",
            serde_json::to_string(&value["uuid"]).unwrap(),
            serde_json::to_string(&value["metadata"]).unwrap(),
            serde_json::to_string(&value["provenance"]).unwrap(),
            serde_json::to_string(&header).unwrap(),
            serde_json::to_string(&group["maps"][0]).unwrap()
        );
        let mut hash = Sha256::new();
        hash.update(SUBJECT_PREFIX);
        hash.update(fields.as_bytes());
        assert_eq!(digest(&value), crate::hashing::lower_hex(&hash.finalize()));
    }

    /// String escapes and non-ASCII data remain exact sensitive bytes without disclosure.
    #[test]
    fn private_scalar_spelling_is_hashed_not_projected() {
        let mut value = fixture();
        value["mappings"][0]["maps"][0]["remarks"] =
            Value::String("private\n\t\"\\λ\u{202e}".into());
        let hash = digest(&value);
        assert!(!hash.contains("private"));
        assert_ne!(hash, digest(&fixture()));
        assert_eq!(relationship_reason("equivalent-to").unwrap(), "relationship-equivalent-to");
    }

    /// Only closed native relationship codes become minimized reasons; arbitrary prose refuses.
    #[test]
    fn unknown_native_relationship_cannot_become_reason_text() {
        assert_eq!(relationship_reason("no-relationship").unwrap(), "relationship-no-relationship");
        assert_eq!(relationship_reason("private author phrase"), Err(ContractError::Binding));
    }

    /// Exact payload equality is admitted, while plus-one refuses before digest mutation.
    #[test]
    fn hash_sink_bound_precedes_growth_and_is_sticky() {
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        let result = ledger.bound(|ledger| {
            let mut sink = HashSink {
                hash: Sha256::new(),
                length: 0,
                limit: 3,
                ledger,
                control: &mut control,
                failure: None,
            };
            sink.write_all(b"abc").unwrap();
            let at_bound = sink.hash.clone().finalize();
            assert!(sink.write_all(b"d").is_err());
            assert_eq!(sink.length, 3);
            assert_eq!(at_bound, sink.hash.clone().finalize());
            assert!(sink.write(b"").is_err());
            Err::<(), _>(sink.failure.unwrap())
        });
        assert_eq!(result, Err(ContractError::Capacity));
        assert_eq!(ledger.bound(|_| Ok(())), Err(ContractError::Capacity));
    }

    /// Full caller contexts can consume shared byte work even if equality would short-circuit.
    #[test]
    fn caller_context_extent_is_not_a_fixed_width_credit() {
        let actual = ContextSnapshot {
            reason_codes: vec!["mapping-re-review".into()],
            related_subject_ids: Vec::new(),
        };
        let caller = ContextSnapshot {
            reason_codes: vec!["r".repeat(128); 32],
            related_subject_ids: vec!["s".repeat(256); 32],
        };
        let mut ledger = ContractLedger::default();
        ledger.bytes(268_435_456 - 512).unwrap();
        assert_eq!(
            ledger.bound(|ledger| charge_contexts(&caller, &actual, ledger)),
            Err(ContractError::Capacity)
        );
        assert_eq!(ledger.bound(|_| Ok(())), Err(ContractError::Capacity));
    }

    /// Cancellation at a real writer checkpoint stops later bytes and shared commands.
    #[test]
    fn hash_sink_preserves_exact_cancelled_stop() {
        let mut ledger = ContractLedger::default();
        let mut control = Recorder::at(Stage::PrepareDomain, 2);
        let result = ledger.bound(|ledger| {
            let mut sink = HashSink {
                hash: Sha256::new(),
                length: 0,
                limit: 8,
                ledger,
                control: &mut control,
                failure: None,
            };
            sink.write_all(b"one").unwrap();
            assert!(sink.write_all(b"two").is_err());
            assert_eq!(sink.length, 3);
            assert!(sink.flush().is_err());
            Err::<(), _>(sink.failure.unwrap())
        });
        assert_eq!(result, Err(ContractError::Interrupted(Interruption::CancelRequested)));
        assert_eq!(ledger.bound(|_| Ok(())), result);
    }
}
