//! Closed explicit links selection and complete plain two-queue item accounting.
//!
//! Inputs are actual closed decoded queue declarations. No result in this module
//! proves native/currentness/dependency association, owner membership or publication.
//! Complete companion construction and finding binding remain the genuine caller's work.

use std::cmp::Ordering;
use std::io::{self, Write};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::decode::{ContractError, ContractLedger, Decoded};
use super::supersession_wire::{Differences, Endpoint, GraphCounts};
use super::validate;
use super::wire::{ContextSnapshot, Domain, QueueDocument, ReviewItem, ReviewPolicy, SourcePin};
use crate::workspace::preparation::WorkControl;

/// Fixed separate private selection marker; existing queue/response /1 markers are unchanged.
pub(crate) const LINKS_SCHEMA: &str = "forge.review-queue-links-request/1";
/// One explicit request is one Auxiliary original, within the existing one-MiB ceiling.
const RAW_CAP: usize = 1_048_576;
/// Complete unique explicit lineage edges, without a Cartesian candidate graph.
const EDGE_CAP: usize = 10_000;
/// Aggregate selected finding occurrences, including reuse on different links.
const FINDING_CAP: usize = 100_000;

/// Only explicit selections; no hashes, source paths, queue proof or approval fields.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LinksRequest {
    /// Exact additive private request schema marker.
    pub(crate) schema_version: String,
    /// Complete strictly pair-ordered explicit links.
    pub(crate) links: Vec<LinkRequest>,
}

/// One same-domain pair to resolve against two complete queue declarations.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LinkRequest {
    /// Exact canonical old item UUID, without a subject or key heuristic.
    pub(crate) old_item_id: String,
    /// Exact canonical new item UUID.
    pub(crate) new_item_id: String,
    /// Strictly sorted unique requested current finding UUIDs; existence is still unbound.
    pub(crate) finding_ids: Vec<String>,
}

/// Exact borrowed private original and closed selection data; never a capture proof.
pub(crate) struct DecodedLinks<'a> {
    /// Complete original including whitespace and final LF.
    raw: &'a [u8],
    /// Complete raw digest, not a reserialized selection digest.
    #[cfg(test)]
    raw_sha256: String,
    /// Closed validated selections only.
    document: LinksRequest,
}

impl<'a> DecodedLinks<'a> {
    /// Borrow the complete inert selections.
    pub(crate) fn document(&self) -> &LinksRequest {
        &self.document
    }
    /// Borrow original bytes without reconstructing an owner or purpose registration.
    pub(crate) fn raw(&self) -> &'a [u8] {
        self.raw
    }
    /// Borrow the exact original raw pin.
    #[cfg(test)]
    pub(crate) fn raw_sha256(&self) -> &str {
        &self.raw_sha256
    }
}

/// Decode strict closed selections with original post-phase fences even on syntax refusal.
///
/// This decoder has no native binding authority. Root must feed the genuinely held Links
/// original and retain the same original ledger/control through the rest of the operation.
pub(crate) fn decode_links<'a>(
    raw: &'a [u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<DecodedLinks<'a>, ContractError> {
    ledger.bound(|ledger| {
        ledger.checkpoint(control)?;
        let result = decode_inner(raw, ledger, control);
        ledger.checkpoint(control)?;
        result
    })
}

/// Admit actual raw extent and conservative parse/typed storage before either growth.
fn decode_inner<'a>(
    raw: &'a [u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<DecodedLinks<'a>, ContractError> {
    if raw.len() > RAW_CAP {
        return Err(ledger.capacity());
    }
    if raw.is_empty() || raw.starts_with(&[0xef, 0xbb, 0xbf]) {
        return Err(ContractError::Invalid);
    }
    ledger.bytes(raw.len())?;
    let slots = raw_slots(raw, ledger, control)?;
    let logical = add(mul(slots, 192, ledger)?, add(raw.len(), 512, ledger)?, ledger)?;
    ledger.derived(logical)?;
    ledger.bytes(raw.len())?;
    let parsed = crate::json_strict::parse_value(
        raw,
        "review",
        crate::json_strict::Limits { max_depth: 64, max_string_bytes: 65_536 },
    )
    .map_err(|_| ContractError::Invalid);
    ledger.checkpoint(control)?;
    let value = parsed?;
    let inspected = inspect(&value, ledger, control);
    ledger.checkpoint(control)?;
    inspected?;
    ledger.bytes(raw.len())?;
    let typed = serde_json::from_value(value).map_err(|_| ContractError::Invalid);
    ledger.checkpoint(control)?;
    let document = typed?;
    ledger.bytes(raw.len())?;
    ledger.derived(64)?;
    let raw_sha256 = crate::hashing::sha256_hex(raw);
    ledger.checkpoint(control)?;
    #[cfg(not(test))]
    drop(raw_sha256);
    Ok(DecodedLinks {
        raw,
        #[cfg(test)]
        raw_sha256,
        document,
    })
}

/// Count an overestimate of decoded slots without allocating or interpreting selections.
fn raw_slots(
    raw: &[u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<usize, ContractError> {
    let mut count = 1;
    let mut quoted = false;
    let mut escaped = false;
    for (index, byte) in raw.iter().copied().enumerate() {
        if index % 4096 == 0 {
            ledger.checkpoint(control)?;
            ledger.visits(1)?;
        }
        if quoted {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                quoted = false;
            }
        } else {
            if matches!(byte, b'"' | b'{' | b'[' | b',' | b':') {
                count = add(count, 1, ledger)?;
            }
            if byte == b'"' {
                quoted = true;
            }
        }
    }
    Ok(count)
}

/// Require the exact field set before typed container moves; null and omission both refuse.
fn object<'a>(
    value: &'a Value,
    keys: &[&str],
    ledger: &mut ContractLedger,
) -> Result<&'a serde_json::Map<String, Value>, ContractError> {
    let rows = value.as_object().ok_or(ContractError::Invalid)?;
    if rows.len() != keys.len() {
        return Err(ContractError::Invalid);
    }
    for actual in rows.keys() {
        for expected in keys {
            // Complete conservative lookup comparison plan precedes Map key probes.
            let _ = compared(actual, expected, ledger)?;
        }
    }
    if keys.iter().any(|key| !rows.contains_key(*key)) {
        return Err(ContractError::Invalid);
    }
    Ok(rows)
}

/// Inspect canonical UUID syntax under fixed scratch/work admission.
fn id<'a>(value: &'a Value, ledger: &mut ContractLedger) -> Result<&'a str, ContractError> {
    let text = value.as_str().ok_or(ContractError::Invalid)?;
    ledger.visits(1)?;
    ledger.bytes(text.len())?;
    if text.len() != 36 {
        return Err(ContractError::Invalid);
    }
    ledger.derived(64)?;
    validate::uuid(text)?;
    Ok(text)
}

/// Validate all selected rows, aggregate occurrences and strict request ordering.
fn inspect(
    value: &Value,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let root = object(value, &["schema_version", "links"], ledger)?;
    let marker = root["schema_version"].as_str().ok_or(ContractError::Invalid)?;
    if compared(marker, LINKS_SCHEMA, ledger)? != Ordering::Equal {
        return Err(ContractError::Invalid);
    }
    let rows = root["links"].as_array().ok_or(ContractError::Invalid)?;
    if rows.len() > EDGE_CAP {
        return Err(ledger.capacity());
    }
    let mut prior_pair = None;
    let mut occurrences = 0;
    for row in rows {
        ledger.checkpoint(control)?;
        ledger.visits(1)?;
        let row = object(row, &["old_item_id", "new_item_id", "finding_ids"], ledger)?;
        let old = id(&row["old_item_id"], ledger)?;
        let new = id(&row["new_item_id"], ledger)?;
        if let Some((old_prior, new_prior)) = prior_pair {
            let first = compared(old_prior, old, ledger)?;
            let second = compared(new_prior, new, ledger)?;
            if first.then(second) != Ordering::Less {
                return Err(ContractError::Invalid);
            }
        }
        prior_pair = Some((old, new));
        let ids = row["finding_ids"].as_array().ok_or(ContractError::Invalid)?;
        occurrences = add(occurrences, ids.len(), ledger)?;
        if occurrences > FINDING_CAP {
            return Err(ledger.capacity());
        }
        let mut previous = None;
        for value in ids {
            ledger.checkpoint(control)?;
            let current = id(value, ledger)?;
            if let Some(prior) = previous {
                if compared(prior, current, ledger)? != Ordering::Less {
                    return Err(ContractError::Invalid);
                }
            }
            previous = Some(current);
        }
    }
    Ok(())
}

/// Resolved explicit pair and selected IDs only; it cannot emit `FindingReferences`.
pub(crate) struct ItemLink<'a> {
    /// Complete actual old inert item, never replaced by its UUID alone.
    old: &'a ReviewItem,
    /// Complete actual new inert item.
    new: &'a ReviewItem,
    /// All exact declared requested IDs; native existence/association is still absent.
    requested_findings: &'a [String],
    /// Exact complete snapshot differences.
    differences: Differences,
}

impl ItemLink<'_> {
    /// Borrow the full old declaration for later genuine native dependency binding.
    pub(crate) fn old_item(&self) -> &ReviewItem {
        self.old
    }
    /// Borrow the full new declaration; this grants no native currentness.
    #[cfg(test)]
    pub(crate) fn new_item(&self) -> &ReviewItem {
        self.new
    }
    /// Project the fixed old endpoint whitelist, without owner or title/prose fields.
    pub(crate) fn old_endpoint(&self) -> Endpoint<'_> {
        endpoint(self.old)
    }
    /// Project the fixed new endpoint whitelist.
    pub(crate) fn new_endpoint(&self) -> Endpoint<'_> {
        endpoint(self.new)
    }
    /// Borrow all explicit finding selections, without claiming a verified association.
    pub(crate) fn requested_findings(&self) -> &[String] {
        self.requested_findings
    }
    /// Return complete typed differences, not hash-only equivalence.
    pub(crate) fn differences(&self) -> Differences {
        self.differences
    }
}

/// Complete plain graph borrowing both full closed queue declarations and the request.
pub(crate) struct ItemGraph<'a> {
    /// Complete old declaration used for all references, not a selected subset.
    old: &'a QueueDocument,
    /// Complete new declaration used for all references.
    new: &'a QueueDocument,
    /// Complete strictly pair-ordered resolved links.
    links: Vec<ItemLink<'a>>,
    /// Every unmatched old row, in original queue item order.
    unmatched_old: Vec<&'a ReviewItem>,
    /// Every unmatched new row, in original queue item order.
    unmatched_new: Vec<&'a ReviewItem>,
    /// Complete strictly ordered distinct requested IDs, not a native finding union.
    requested: Vec<&'a str>,
    /// Checked complete item/selection conservation.
    counts: GraphCounts,
}

impl ItemGraph<'_> {
    /// Borrow the actual complete old inert queue declaration.
    pub(crate) fn old_queue(&self) -> &QueueDocument {
        self.old
    }
    /// Borrow the actual complete new inert queue declaration.
    pub(crate) fn new_queue(&self) -> &QueueDocument {
        self.new
    }
    /// Borrow every declared resolved pair in exact request order.
    pub(crate) fn links(&self) -> &[ItemLink<'_>] {
        &self.links
    }
    /// Borrow the complete unmatched old roster in queue order.
    pub(crate) fn unmatched_old(&self) -> &[&ReviewItem] {
        &self.unmatched_old
    }
    /// Borrow the complete unmatched new roster in queue order.
    pub(crate) fn unmatched_new(&self) -> &[&ReviewItem] {
        &self.unmatched_new
    }
    /// Borrow strictly sorted distinct requested IDs before actual native binding.
    pub(crate) fn requested_findings(&self) -> &[&str] {
        &self.requested
    }
    /// Return counters derived from the complete actual arrays, without native claims.
    pub(crate) fn counts(&self) -> GraphCounts {
        self.counts
    }
}

/// Prepare complete explicit item accounting from actual Decoded queues under one owner ledger.
///
/// No completeness/native/currentness boolean, lease index or factory capability is accepted.
/// Root must separately bind the exact held raw queue/request originals and native finding
/// associations before constructing/encoding a full companion. No response is consumed.
pub(crate) fn prepare_graph<'a>(
    old: &'a Decoded<'_, QueueDocument>,
    new: &'a Decoded<'_, QueueDocument>,
    request: &'a DecodedLinks<'_>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<ItemGraph<'a>, ContractError> {
    ledger.bound(|ledger| {
        ledger.checkpoint(control)?;
        let extent = add(old.document().queue_id.len(), new.document().queue_id.len(), ledger)?;
        ledger.bytes(extent)?;
        let result = if old.document().queue_id == new.document().queue_id {
            Err(ContractError::Binding)
        } else {
            graph_inner(old.document(), new.document(), request.document(), ledger, control)
        };
        ledger.checkpoint(control)?;
        result
    })
}

/// Resolve all complete actual rows, then derive unmatched and distinct membership conservation.
fn graph_inner<'a>(
    old: &'a QueueDocument,
    new: &'a QueueDocument,
    request: &'a LinksRequest,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<ItemGraph<'a>, ContractError> {
    if request.links.len() > EDGE_CAP {
        return Err(ledger.capacity());
    }
    let mut occurrences = 0;
    for link in &request.links {
        ledger.checkpoint(control)?;
        ledger.visits(1)?;
        occurrences = add(occurrences, link.finding_ids.len(), ledger)?;
        if occurrences > FINDING_CAP {
            return Err(ledger.capacity());
        }
    }
    let old_order = item_order(&old.items, ledger, control)?;
    let new_order = item_order(&new.items, ledger, control)?;
    let mut old_used = vector::<bool>(old.items.len(), ledger)?;
    ledger.bytes(old.items.len())?;
    old_used.resize(old.items.len(), false);
    let mut new_used = vector::<bool>(new.items.len(), ledger)?;
    ledger.bytes(new.items.len())?;
    new_used.resize(new.items.len(), false);
    let mut links = vector::<ItemLink<'a>>(request.links.len(), ledger)?;
    let mut requested = vector::<&str>(occurrences, ledger)?;
    for link in &request.links {
        ledger.checkpoint(control)?;
        let left = locate_item(&old.items, &old_order, &link.old_item_id, ledger, control)?;
        let right = locate_item(&new.items, &new_order, &link.new_item_id, ledger, control)?;
        if old.items[left].domain != new.items[right].domain {
            return Err(ContractError::Binding);
        }
        let differences =
            differences(old, &old.items[left], new, &new.items[right], ledger, control)?;
        ledger.visits(2)?;
        ledger.bytes(2)?;
        old_used[left] = true;
        new_used[right] = true;
        ledger.bytes(std::mem::size_of::<ItemLink<'a>>())?;
        links.push(ItemLink {
            old: &old.items[left],
            new: &new.items[right],
            requested_findings: &link.finding_ids,
            differences,
        });
        for id in &link.finding_ids {
            ledger.visits(1)?;
            ledger.bytes(std::mem::size_of::<&str>())?;
            requested.push(id.as_str());
        }
    }
    ordered(&mut requested, &mut |a, b, ledger| compared(a, b, ledger), ledger, control)?;
    unique(&mut requested, ledger, control)?;
    let unmatched_old = unmatched(&old.items, &old_used, ledger, control)?;
    let unmatched_new = unmatched(&new.items, &new_used, ledger, control)?;
    let linked_old_items =
        old.items.len().checked_sub(unmatched_old.len()).ok_or_else(|| ledger.capacity())?;
    let linked_new_items =
        new.items.len().checked_sub(unmatched_new.len()).ok_or_else(|| ledger.capacity())?;
    let counts = GraphCounts {
        old_items: old.items.len(),
        new_items: new.items.len(),
        link_edges: links.len(),
        linked_old_items,
        linked_new_items,
        unmatched_old_items: unmatched_old.len(),
        unmatched_new_items: unmatched_new.len(),
        link_finding_occurrences: occurrences,
        distinct_requested_findings: requested.len(),
    };
    if add(counts.linked_old_items, counts.unmatched_old_items, ledger)? != counts.old_items
        || add(counts.linked_new_items, counts.unmatched_new_items, ledger)? != counts.new_items
    {
        return Err(ContractError::Binding);
    }
    Ok(ItemGraph { old, new, links, unmatched_old, unmatched_new, requested, counts })
}

/// Project only exact contract endpoint fields from the complete item.
fn endpoint(item: &ReviewItem) -> Endpoint<'_> {
    Endpoint {
        key: &item.key,
        item_id: &item.item_id,
        domain: item.domain,
        adapter_version: &item.adapter_version,
        subject_id: &item.subject_id,
        subject_sha256: &item.subject_sha256,
        context_sha256: &item.context_sha256,
        policy_key: &item.policy_key,
        policy_sha256: &item.policy_sha256,
        requested_action: item.requested_action,
        source_keys: &item.source_keys,
    }
}

/// Complete actual subject comparison, matching the declared four-field rule.
#[derive(Serialize, PartialEq, Eq)]
struct Subject<'a> {
    /// Native review family.
    domain: Domain,
    /// Complete actual adapter marker.
    adapter: &'a str,
    /// Exact unnormalized subject ID.
    id: &'a str,
    /// Exact recorded review fingerprint.
    hash: &'a str,
}

/// Complete input vocabulary of the unchanged `ContextEncoding` profile.
#[derive(Serialize, PartialEq, Eq)]
struct ContextInput<'a> {
    /// Actual declared family.
    domain: Domain,
    /// Exact adapter marker.
    adapter: &'a str,
    /// Entire queue pin roster bound by existing context hashing, not just item keys.
    pins: &'a [SourcePin],
    /// Complete minimized typed context, not a display hash alone.
    context: &'a ContextSnapshot,
}

/// Complete input vocabulary of the unchanged `PolicyEncoding` profile.
#[derive(Serialize, PartialEq, Eq)]
struct PolicyInput<'a> {
    /// Actual declared family.
    domain: Domain,
    /// Exact adapter marker.
    adapter: &'a str,
    /// Entire queue pin roster bound by existing policy hashing.
    pins: &'a [SourcePin],
    /// Complete selected policy including its key and substitutions.
    policy: &'a ReviewPolicy,
    /// Complete exact sorted declared authors.
    authors: &'a [String],
    /// Complete exact ordered assignments.
    assignments: &'a [super::wire::Assignment],
    /// Exact asserted deadline or null.
    due: &'a Option<String>,
    /// Entire dissent-preserving disposition vocabulary.
    allowed: &'a [super::wire::Disposition],
}

/// Derive complete typed differences after all comparison extents are admitted.
fn differences(
    old: &QueueDocument,
    left: &ReviewItem,
    new: &QueueDocument,
    right: &ReviewItem,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Differences, ContractError> {
    let subject = differs(
        &Subject {
            domain: left.domain,
            adapter: &left.adapter_version,
            id: &left.subject_id,
            hash: &left.subject_sha256,
        },
        &Subject {
            domain: right.domain,
            adapter: &right.adapter_version,
            id: &right.subject_id,
            hash: &right.subject_sha256,
        },
        ledger,
        control,
    )?;
    let context = differs(
        &ContextInput {
            domain: left.domain,
            adapter: &left.adapter_version,
            pins: &old.source_pins,
            context: &left.context,
        },
        &ContextInput {
            domain: right.domain,
            adapter: &right.adapter_version,
            pins: &new.source_pins,
            context: &right.context,
        },
        ledger,
        control,
    )?;
    let old_policy =
        locate(&old.policies, &left.policy_key, |row| row.key.as_str(), ledger, control)?;
    let new_policy =
        locate(&new.policies, &right.policy_key, |row| row.key.as_str(), ledger, control)?;
    let policy = differs(
        &PolicyInput {
            domain: left.domain,
            adapter: &left.adapter_version,
            pins: &old.source_pins,
            policy: old_policy,
            authors: &left.author_keys,
            assignments: &left.assignments,
            due: &left.due_at,
            allowed: &left.allowed_dispositions,
        },
        &PolicyInput {
            domain: right.domain,
            adapter: &right.adapter_version,
            pins: &new.source_pins,
            policy: new_policy,
            authors: &right.author_keys,
            assignments: &right.assignments,
            due: &right.due_at,
            allowed: &right.allowed_dispositions,
        },
        ledger,
        control,
    )?;
    let source_pins = source_difference(old, left, new, right, ledger, control)?;
    Ok(Differences { subject, context, policy, source_pins })
}

/// Compare every corresponding selected `SourcePin` value, preserving separate roster identity.
fn source_difference(
    old: &QueueDocument,
    left: &ReviewItem,
    new: &QueueDocument,
    right: &ReviewItem,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<bool, ContractError> {
    let mut different = left.source_keys.len() != right.source_keys.len();
    for index in 0..left.source_keys.len().max(right.source_keys.len()) {
        ledger.checkpoint(control)?;
        let a = left
            .source_keys
            .get(index)
            .map(|key| {
                locate(&old.source_pins, key, |pin| pin.artifact_key.as_str(), ledger, control)
            })
            .transpose()?;
        let b = right
            .source_keys
            .get(index)
            .map(|key| {
                locate(&new.source_pins, key, |pin| pin.artifact_key.as_str(), ledger, control)
            })
            .transpose()?;
        match (a, b) {
            (Some(a), Some(b)) => {
                different |= differs(a, b, ledger, control)?;
            }
            (Some(value), None) | (None, Some(value)) => {
                cost(value, ledger, control)?;
                different = true;
            }
            (None, None) => return Err(ContractError::Binding),
        }
    }
    Ok(different)
}

/// Stream actual closed typed comparison extents without retaining serialized text.
struct CostWriter<'a> {
    /// The same caller ledger, with no reset or comparison allowance.
    ledger: &'a mut ContractLedger,
    /// The same actual accepted cooperative control.
    control: &'a mut dyn WorkControl,
    /// Precise first admission/control failure through serde's IO wrapper.
    error: Option<ContractError>,
}

impl Write for CostWriter<'_> {
    /// Admit actual serializer work plus a conservative complete subsequent typed comparison.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let result = self.ledger.bound(|ledger| {
            ledger.checkpoint(self.control)?;
            ledger.visits(1)?;
            let extent = add(mul(bytes.len(), 2, ledger)?, 64, ledger)?;
            ledger.bytes(extent)
        });
        if let Err(error) = result {
            self.error.get_or_insert(error);
            return Err(io::Error::other("review comparison refused"));
        }
        Ok(bytes.len())
    }
    /// No retained output exists; final fence is owned by the enclosing phase.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Charge complete typed output and full comparison work, then fence success and errors.
fn cost<T: Serialize + ?Sized>(
    value: &T,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let result = {
        let mut writer = CostWriter { ledger, control, error: None };
        let serialized = serde_json::to_writer(&mut writer, value);
        if serialized.is_err() {
            Err(writer.error.unwrap_or(ContractError::Invalid))
        } else {
            Ok(())
        }
    };
    ledger.checkpoint(control)?;
    result
}

/// Compare exact complete values only after both complete sides have been precharged.
fn differs<T: Serialize + PartialEq + ?Sized>(
    left: &T,
    right: &T,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<bool, ContractError> {
    cost(left, ledger, control)?;
    cost(right, ledger, control)?;
    ledger.visits(1)?;
    ledger.matching(1)?;
    Ok(left != right)
}

/// Build a complete item UUID index with admitted fallible in-place deterministic sorting.
fn item_order(
    items: &[ReviewItem],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<usize>, ContractError> {
    let mut order = vector::<usize>(items.len(), ledger)?;
    for index in 0..items.len() {
        ledger.visits(1)?;
        ledger.bytes(std::mem::size_of::<usize>())?;
        order.push(index);
    }
    ordered(
        &mut order,
        &mut |a, b, ledger| compared(&items[a].item_id, &items[b].item_id, ledger),
        ledger,
        control,
    )?;
    Ok(order)
}

/// Resolve the exact requested UUID by charged binary lookup in the full item index.
fn locate_item(
    items: &[ReviewItem],
    order: &[usize],
    key: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<usize, ContractError> {
    let mut lower = 0;
    let mut upper = order.len();
    while lower < upper {
        ledger.checkpoint(control)?;
        let mid = lower + (upper - lower) / 2;
        match compared(&items[order[mid]].item_id, key, ledger)? {
            Ordering::Less => lower = mid + 1,
            Ordering::Greater => upper = mid,
            Ordering::Equal => return Ok(order[mid]),
        }
    }
    Err(ContractError::Binding)
}

/// Resolve sorted actual policy/source keys without an inferred fallback or full Cartesian scan.
fn locate<'a, T>(
    rows: &'a [T],
    key: &str,
    key_of: impl Fn(&T) -> &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<&'a T, ContractError> {
    let mut lower = 0;
    let mut upper = rows.len();
    while lower < upper {
        ledger.checkpoint(control)?;
        let mid = lower + (upper - lower) / 2;
        match compared(key_of(&rows[mid]), key, ledger)? {
            Ordering::Less => lower = mid + 1,
            Ordering::Greater => upper = mid,
            Ordering::Equal => return Ok(&rows[mid]),
        }
    }
    Err(ContractError::Binding)
}

/// Precharge both complete original string extents for each actual comparison.
fn compared(
    left: &str,
    right: &str,
    ledger: &mut ContractLedger,
) -> Result<Ordering, ContractError> {
    ledger.visits(1)?;
    ledger.matching(1)?;
    let extent = add(left.len(), right.len(), ledger)?;
    ledger.bytes(extent)?;
    Ok(left.cmp(right))
}

/// Fallible heapsort; every real comparison and index/reference swap is admitted first.
fn ordered<T: Copy>(
    rows: &mut [T],
    compare: &mut impl FnMut(T, T, &mut ContractLedger) -> Result<Ordering, ContractError>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    for start in (0..rows.len() / 2).rev() {
        sift(rows, start, rows.len(), compare, ledger, control)?;
    }
    for end in (1..rows.len()).rev() {
        swapped(rows, 0, end, ledger)?;
        sift(rows, 0, end, compare, ledger, control)?;
    }
    Ok(())
}

/// Restore a bounded max heap with fallible real comparisons and no allocation.
fn sift<T: Copy>(
    rows: &mut [T],
    mut parent: usize,
    end: usize,
    compare: &mut impl FnMut(T, T, &mut ContractLedger) -> Result<Ordering, ContractError>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    loop {
        ledger.checkpoint(control)?;
        let mut child = add(mul(parent, 2, ledger)?, 1, ledger)?;
        if child >= end {
            return Ok(());
        }
        let sibling = add(child, 1, ledger)?;
        if sibling < end && compare(rows[child], rows[sibling], ledger)? == Ordering::Less {
            child = sibling;
        }
        if compare(rows[parent], rows[child], ledger)? != Ordering::Less {
            return Ok(());
        }
        swapped(rows, parent, child, ledger)?;
        parent = child;
    }
}

/// Charge complete fixed-width copy work before every actual swap.
fn swapped<T>(
    rows: &mut [T],
    a: usize,
    b: usize,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    ledger.visits(1)?;
    let extent = mul(std::mem::size_of::<T>(), 3, ledger)?;
    ledger.bytes(extent)?;
    rows.swap(a, b);
    Ok(())
}

/// Compact a sorted borrowed ID array with charged actual equality/move work.
fn unique(
    rows: &mut Vec<&str>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let mut retained = 0;
    for index in 0..rows.len() {
        ledger.checkpoint(control)?;
        if retained == 0 || compared(rows[retained - 1], rows[index], ledger)? != Ordering::Equal {
            ledger.bytes(std::mem::size_of::<&str>())?;
            rows[retained] = rows[index];
            retained = add(retained, 1, ledger)?;
        }
    }
    rows.truncate(retained);
    Ok(())
}

/// Keep all unmatched complete rows in exact native queue item order.
fn unmatched<'a>(
    items: &'a [ReviewItem],
    used: &[bool],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<&'a ReviewItem>, ContractError> {
    let mut rows = vector::<&ReviewItem>(items.len(), ledger)?;
    for (item, selected) in items.iter().zip(used) {
        ledger.checkpoint(control)?;
        ledger.visits(1)?;
        if !selected {
            ledger.bytes(std::mem::size_of::<&ReviewItem>())?;
            rows.push(item);
        }
    }
    Ok(rows)
}

/// Reserve complete owned vector payload before growth; borrowed strings are not copied.
fn vector<T>(count: usize, ledger: &mut ContractLedger) -> Result<Vec<T>, ContractError> {
    let extent =
        add(std::mem::size_of::<Vec<T>>(), mul(count, std::mem::size_of::<T>(), ledger)?, ledger)?;
    ledger.derived(extent)?;
    let mut rows = Vec::new();
    rows.try_reserve_exact(count).map_err(|_| ledger.capacity())?;
    Ok(rows)
}

/// Checked sum whose actual first capacity refusal latches before any later fence.
fn add(left: usize, right: usize, ledger: &mut ContractLedger) -> Result<usize, ContractError> {
    left.checked_add(right).ok_or_else(|| ledger.capacity())
}
/// Checked product with the same actual first-stop semantics.
fn mul(left: usize, right: usize, ledger: &mut ContractLedger) -> Result<usize, ContractError> {
    left.checked_mul(right).ok_or_else(|| ledger.capacity())
}

#[cfg(test)]
#[path = "links_tests.rs"]
/// Prospective real decoder/accounting controls; inert declarations grant no native proof.
mod tests;
