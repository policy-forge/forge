//! Minimized applicability re-review facts from a genuine recorded-current closure.
//! The complete captured report-source approval is factory-owned. Hashes, selected
//! IDs, classifier labels and asserted policy quorum create no domain approval.

use std::io::{self, Write};

use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::applicability::model::{ClassificationCounts, GapClassification};
use crate::mapping::inventory::ResourceEvidence;
use crate::mapping::manifest::ResourceType;
use crate::review::applicability_capture::{ApprovedApplicabilityClosure, ControlFacts};
use crate::review::decode::{ContractError, ContractLedger, Decoded};
use crate::review::wire::{ContextSnapshot, Domain, QueueDocument, RequestedAction, SourcePin};
use crate::workspace::preparation::WorkControl;

/// Complete effective framework and explicit selection ceiling, not a partial prefix.
const MAX_CONTROLS: usize = 10_000;
/// Exact existing path-free review subject token ceiling in UTF-8 bytes.
const MAX_SUBJECT_ID: usize = 256;
/// Compact sensitive subject payload ceiling under the same command ledger.
const MAX_SUBJECT_PAYLOAD: usize = 10 * 1024 * 1024;
/// Reviewed applicability subject domain, including its required trailing NUL.
const SUBJECT_PREFIX: &[u8] = b"forge.applicability-review-subject/1\0";
/// First exact applicability re-review adapter version, separate from source versions.
const ADAPTER_VERSION: &str = "forge.applicability-review/1";

/// One selected actual explicit decision; no source approval constructor is exposed.
pub(crate) struct ApplicabilityFact<'a> {
    /// Actual framework control ID, borrowed unchanged from the genuine owner.
    subject_id: &'a str,
    /// Full private decision/native/framework binding, never a raw source replacement.
    subject_sha256: String,
    /// Sorted constant reason codes only; no private native or recorded prose.
    context: ContextSnapshot,
}

impl ApplicabilityFact<'_> {
    /// Borrow the original selected control ID without truncation or normalization.
    pub(crate) fn subject_id(&self) -> &str {
        self.subject_id
    }
    /// Borrow the domain-separated complete sensitive subject digest.
    pub(crate) fn subject_sha256(&self) -> &str {
        &self.subject_sha256
    }
    /// Borrow only bounded fixed classification/scope codes and empty related IDs.
    pub(crate) fn context(&self) -> &ContextSnapshot {
        &self.context
    }
}

/// Complete decision denominators, derived before any explicit selection projection.
struct DecisionCounts {
    /// Every actual Some original decision-array index in the complete native roster.
    explicit: usize,
    /// Every actual omission, including its maintained under-review classification.
    omitted: usize,
}

/// Selected facts borrowing one genuine full current report-source capture owner.
pub(crate) struct PreparedApplicabilityReview<'a> {
    /// Sole native approval/currentness input; never a caller Value or boolean.
    closure: &'a ApprovedApplicabilityClosure,
    /// Every requested explicit decision, sorted by unchanged control identity.
    facts: Vec<ApplicabilityFact<'a>>,
    /// Complete explicit/omitted denominator, independent of selected count.
    decisions: DecisionCounts,
}

impl PreparedApplicabilityReview<'_> {
    /// Borrow the complete actual source/native/report/lifecycle pin roster.
    pub(crate) fn source_pins(&self) -> &[SourcePin] {
        self.closure.source_pins()
    }
    /// Borrow all explicitly selected minimized subjects without exposing source prose.
    pub(crate) fn facts(&self) -> impl Iterator<Item = &ApplicabilityFact<'_>> {
        self.facts.iter()
    }
    /// Return the actual selected cardinality before caller fact-vector growth or iteration.
    pub(crate) fn selected_count(&self) -> usize {
        self.facts.len()
    }
    /// Return every effective framework control, including omitted decisions.
    pub(crate) fn complete_controls(&self) -> usize {
        self.closure.control_facts().len()
    }
    /// Return all explicit original decisions before selection, including under-review.
    pub(crate) fn explicit_decisions(&self) -> usize {
        self.decisions.explicit
    }
    /// Return all actual omissions; no omitted control becomes a selected item.
    pub(crate) fn omitted_decisions(&self) -> usize {
        self.decisions.omitted
    }
    /// Borrow the reconciled six categories plus total from the full native report.
    pub(crate) fn complete_counts(&self) -> &ClassificationCounts {
        self.closure.complete_counts()
    }
    /// Return every native map, distinct from per-target participation and selection.
    pub(crate) fn complete_maps(&self) -> usize {
        self.closure.complete_maps()
    }
    /// Return the complete Cartesian locator count, not its repeated work charge.
    pub(crate) fn complete_pair_inspections(&self) -> usize {
        self.closure.complete_pair_inspections()
    }
    /// Borrow the actual sealed owner for same-Rc finalizer binding.
    pub(crate) fn closure(&self) -> &ApprovedApplicabilityClosure {
        self.closure
    }
    /// Recheck complete original/root/generation proofs with the same sticky budget.
    pub(crate) fn verify_inputs(
        &self,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        ledger.bound(|ledger| self.closure.verify_inputs(ledger, control))
    }

    /// Bind every selected queue item and the whole source roster without policy promotion.
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
            ledger.derived(self.facts.len().checked_add(32).ok_or(ContractError::Capacity)?)?;
            let mut seen = vec![false; self.facts.len()];
            for item in &queue.items {
                ledger.checkpoint(control)?;
                let index = self.find(&item.subject_id, ledger, control)?;
                if seen[index] {
                    return Err(ContractError::Binding);
                }
                seen[index] = true;
                self.bind_item(item, &self.facts[index], ledger, control)?;
            }
            self.closure.verify_inputs(ledger, control)?;
            ledger.checkpoint(control)
        })
    }

    /// Compare full caller/native metadata extents before exact applicability-only binding.
    fn bind_item(
        &self,
        item: &crate::review::wire::ReviewItem,
        fact: &ApplicabilityFact<'_>,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        ledger.checkpoint(control)?;
        ledger.visits(1)?;
        ledger.bytes(
            item.adapter_version
                .len()
                .checked_add(ADAPTER_VERSION.len())
                .and_then(|n| n.checked_add(item.subject_sha256.len()))
                .and_then(|n| n.checked_add(fact.subject_sha256.len()))
                .ok_or(ContractError::Capacity)?,
        )?;
        charge_contexts(&item.context, &fact.context, ledger)?;
        if item.domain != Domain::ApplicabilityDecision
            || item.adapter_version != ADAPTER_VERSION
            || item.requested_action != RequestedAction::ReReview
            || item.subject_sha256 != fact.subject_sha256
            || item.context != fact.context
        {
            return Err(ContractError::Binding);
        }
        charge_source_keys(&item.source_keys, self.source_pins(), ledger)?;
        if item.source_keys.len() != self.source_pins().len()
            || item
                .source_keys
                .iter()
                .zip(self.source_pins())
                .any(|(key, pin)| key != &pin.artifact_key)
        {
            return Err(ContractError::Binding);
        }
        Ok(())
    }

    /// Search only the sorted selected registry, charging both actual string operands.
    fn find(
        &self,
        id: &str,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<usize, ContractError> {
        let mut low = 0;
        let mut high = self.facts.len();
        while low < high {
            ledger.checkpoint(control)?;
            let middle = low + (high - low) / 2;
            ledger.visits(1)?;
            ledger.bytes(
                id.len()
                    .checked_add(self.facts[middle].subject_id.len())
                    .ok_or(ContractError::Capacity)?,
            )?;
            match self.facts[middle].subject_id.cmp(id) {
                std::cmp::Ordering::Less => low = middle + 1,
                std::cmp::Ordering::Greater => high = middle,
                std::cmp::Ordering::Equal => return Ok(middle),
            }
        }
        Err(ContractError::Binding)
    }
}

/// Derive selected explicit facts only from a fully sealed native report-source closure.
/// Whole counts, omissions and private source-array positions are retained independently.
pub(crate) fn prepare<'a>(
    closure: &'a ApprovedApplicabilityClosure,
    selected: &[&str],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<PreparedApplicabilityReview<'a>, ContractError> {
    ledger.bound(|ledger| {
        validate_selected(selected, ledger, control)?;
        closure.verify_inputs(ledger, control)?;
        let decisions = complete_denominators(closure, ledger, control)?;
        ledger.derived(
            selected
                .len()
                .checked_mul(std::mem::size_of::<ApplicabilityFact<'_>>())
                .and_then(|n| n.checked_add(32))
                .ok_or(ContractError::Capacity)?,
        )?;
        let mut facts = Vec::with_capacity(selected.len());
        for id in selected {
            let row = find_control(closure.control_facts(), id, ledger, control)?;
            facts.push(build_fact(closure, row, ledger, control)?);
        }
        closure.verify_inputs(ledger, control)?;
        ledger.checkpoint(control)?;
        Ok(PreparedApplicabilityReview { closure, facts, decisions })
    })
}

/// Admit complete selected token/ordering work before registry allocation or native use.
fn validate_selected(
    selected: &[&str],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    ledger.checkpoint(control)?;
    if selected.is_empty() || selected.len() > MAX_CONTROLS {
        return Err(ContractError::Invalid);
    }
    ledger.visits(selected.len())?;
    for id in selected {
        ledger.checkpoint(control)?;
        if id.is_empty() || id.len() > MAX_SUBJECT_ID {
            return Err(ContractError::Invalid);
        }
        ledger.bytes(id.len())?;
        crate::review::validate::token(id, MAX_SUBJECT_ID)?;
    }
    for pair in selected.windows(2) {
        ledger.checkpoint(control)?;
        ledger.visits(1)?;
        ledger.bytes(pair[0].len().checked_add(pair[1].len()).ok_or(ContractError::Capacity)?)?;
        if pair[0] >= pair[1] {
            return Err(ContractError::Invalid);
        }
    }
    Ok(())
}

/// Reconcile the complete sorted native roster; selected items never shrink denominators.
fn complete_denominators(
    closure: &ApprovedApplicabilityClosure,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<DecisionCounts, ContractError> {
    let rows = closure.control_facts();
    if rows.len() > MAX_CONTROLS
        || closure.complete_maps() > 10_000
        || closure.complete_pair_inspections() > 100_000
    {
        return Err(ContractError::Capacity);
    }
    let mut categories = [0_usize; 6];
    let mut explicit = 0_usize;
    let mut prior: Option<&str> = None;
    for row in rows {
        ledger.checkpoint(control)?;
        ledger.visits(1)?;
        ledger.bytes(
            row.control_id()
                .len()
                .checked_add(prior.map_or(0, str::len))
                .ok_or(ContractError::Capacity)?,
        )?;
        if prior.is_some_and(|id| id >= row.control_id()) {
            return Err(ContractError::Binding);
        }
        prior = Some(row.control_id());
        let slot = category_slot(row.classification());
        categories[slot] = categories[slot].checked_add(1).ok_or(ContractError::Capacity)?;
        if row.decision_index().is_some() {
            explicit = explicit.checked_add(1).ok_or(ContractError::Capacity)?;
        }
    }
    reconcile_counts(rows.len(), categories, closure.complete_counts(), ledger)?;
    let omitted = rows.len().checked_sub(explicit).ok_or(ContractError::Binding)?;
    Ok(DecisionCounts { explicit, omitted })
}

/// Compare all six category cells and their checked whole sum, not a selected subtotal.
fn reconcile_counts(
    total: usize,
    categories: [usize; 6],
    counts: &ClassificationCounts,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    ledger.visits(7)?;
    ledger.bytes(14 * std::mem::size_of::<usize>())?;
    let expected = [
        counts.applicable_mapped,
        counts.applicable_reviewed_no_relationship,
        counts.applicable_unmapped,
        counts.not_applicable,
        counts.deferred,
        counts.under_review,
    ];
    let sum = expected
        .iter()
        .try_fold(0_usize, |n, value| n.checked_add(*value).ok_or(ContractError::Capacity))?;
    if counts.total != total || sum != total || expected != categories {
        return Err(ContractError::Binding);
    }
    Ok(())
}

/// Index the existing native classification vocabulary without recomputing its domain meaning.
fn category_slot(classification: GapClassification) -> usize {
    match classification {
        GapClassification::ApplicableMapped => 0,
        GapClassification::ApplicableReviewedNoRelationship => 1,
        GapClassification::ApplicableUnmapped => 2,
        GapClassification::NotApplicable => 3,
        GapClassification::Deferred => 4,
        GapClassification::UnderReview => 5,
    }
}

/// Binary-search the complete factory-validated sorted roster with actual two-sided charges.
fn find_control<'a>(
    rows: &'a [ControlFacts],
    id: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<&'a ControlFacts, ContractError> {
    let mut low = 0;
    let mut high = rows.len();
    while low < high {
        ledger.checkpoint(control)?;
        let middle = low + (high - low) / 2;
        ledger.visits(1)?;
        ledger.bytes(
            id.len().checked_add(rows[middle].control_id().len()).ok_or(ContractError::Capacity)?,
        )?;
        match rows[middle].control_id().cmp(id) {
            std::cmp::Ordering::Less => low = middle + 1,
            std::cmp::Ordering::Greater => high = middle,
            std::cmp::Ordering::Equal => return Ok(&rows[middle]),
        }
    }
    Err(ContractError::Binding)
}

/// Bind the exact original decision index and derive private hash plus minimal fixed context.
fn build_fact<'a>(
    closure: &'a ApprovedApplicabilityClosure,
    row: &'a ControlFacts,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<ApplicabilityFact<'a>, ContractError> {
    ledger.checkpoint(control)?;
    let index = row.decision_index().ok_or(ContractError::Binding)?;
    let original = require_decision(closure.decision_original(index), row.control_id(), ledger)?;
    let encoding = SubjectEncoding {
        adapter_version: ADAPTER_VERSION,
        framework: FrameworkEncoding::from(closure.framework_evidence()),
        control_fingerprint: row.native_fingerprint(),
        decision: original,
        classification: row.classification(),
        positive_mapping_count: row.positive_count(),
        no_relationship_count: row.no_relationship_count(),
    };
    let subject_sha256 = subject_hash(&encoding, ledger, control)?;
    ledger.visits(1)?;
    ledger.bytes(128)?;
    let state = original.get("state").and_then(Value::as_str).ok_or(ContractError::Binding)?;
    let context = context_codes(row.classification(), state, ledger, control)?;
    Ok(ApplicabilityFact { subject_id: row.control_id(), subject_sha256, context })
}

/// Refuse actual omissions or mismatched raw control identity without manufacturing a decision.
fn require_decision<'a>(
    original: Option<&'a Value>,
    id: &str,
    ledger: &mut ContractLedger,
) -> Result<&'a Value, ContractError> {
    ledger.visits(1)?;
    ledger.bytes(128)?;
    let original = original.ok_or(ContractError::Binding)?;
    let actual =
        original.get("control_id").and_then(Value::as_str).ok_or(ContractError::Binding)?;
    ledger.bytes(actual.len().checked_add(id.len()).ok_or(ContractError::Capacity)?)?;
    if actual != id {
        return Err(ContractError::Binding);
    }
    Ok(original)
}

/// Full ordered framework tuple without private href; raw roster pins independently bind paths/prose.
#[derive(Serialize)]
struct FrameworkEncoding<'a> {
    /// Actual schema-admitted native family, not a locator-inferred type.
    resource_type: ResourceType,
    /// Exact selected framework raw original digest.
    raw_sha256: &'a str,
    /// Actual native root UUID spelling in the maintained evidence tuple.
    root_uuid: &'a str,
    /// Actual native document version, retained without normalization.
    document_version: &'a str,
    /// Actual offline-schema version.
    oscal_version: &'a str,
    /// Actual resolved companion digest, including explicit null for a Catalog.
    resolved_catalog_sha256: Option<&'a str>,
}

impl<'a> From<&'a ResourceEvidence> for FrameworkEncoding<'a> {
    /// Borrow exact native evidence while deliberately excluding its href field.
    fn from(evidence: &'a ResourceEvidence) -> Self {
        Self {
            resource_type: evidence.resource_type,
            raw_sha256: &evidence.raw_sha256,
            root_uuid: &evidence.root_uuid,
            document_version: &evidence.document_version,
            oscal_version: &evidence.oscal_version,
            resolved_catalog_sha256: evidence.resolved_catalog_sha256.as_deref(),
        }
    }
}

/// Complete sensitive decision fields in the exact reviewed seven-field serializer order.
#[derive(Serialize)]
struct SubjectEncoding<'a> {
    /// Exact reviewed adapter version preceding the actual framework tuple.
    adapter_version: &'static str,
    /// Actual current framework identity/effective-source tuple without href.
    framework: FrameworkEncoding<'a>,
    /// Actual maintained effective control fingerprint.
    control_fingerprint: &'a str,
    /// Complete strict original decision Value, preserving omission/null/array distinctions.
    decision: &'a Value,
    /// Actual native classification, not copied or inferred from review policy.
    classification: GapClassification,
    /// Native positive edge-target count, never Cartesian product count.
    positive_mapping_count: usize,
    /// Native reviewed negative edge-target count, never selected subtotal.
    no_relationship_count: usize,
}

/// Nonretaining hash sink with whole chunk admission before digest state changes.
struct HashSink<'a> {
    /// Sensitive digest state without a retained JSON output copy.
    hash: Sha256,
    /// Actual compact payload bytes already hashed; prefix is separately charged.
    length: usize,
    /// Whole sensitive payload ceiling, not a per-chunk allowance.
    limit: usize,
    /// Same sticky command ledger for repeated encoding work and output digest storage.
    ledger: &'a mut ContractLedger,
    /// Same original deadline and caller cancellation source.
    control: &'a mut dyn WorkControl,
    /// First safe typed refusal, with no source-bearing IO diagnostics.
    failure: Option<ContractError>,
}

impl Write for HashSink<'_> {
    /// Charge and hash the whole supplied chunk or refuse before any digest/length mutation.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.failure.is_some() {
            return Err(io::Error::other("applicability subject encoding refused"));
        }
        let result = (|| {
            self.ledger.checkpoint(self.control)?;
            self.ledger.visits(1)?;
            let next = self
                .length
                .checked_add(bytes.len())
                .filter(|n| *n <= self.limit)
                .ok_or(ContractError::Capacity)?;
            self.ledger.bytes(bytes.len())?;
            self.hash.update(bytes);
            self.length = next;
            Ok(())
        })();
        if let Err(error) = result {
            self.failure = Some(error);
            return Err(io::Error::other("applicability subject encoding refused"));
        }
        Ok(bytes.len())
    }
    /// Observe the same stop source without IO or permitting reuse after a writer refusal.
    fn flush(&mut self) -> io::Result<()> {
        if self.failure.is_some() {
            return Err(io::Error::other("applicability subject encoding refused"));
        }
        self.ledger.checkpoint(self.control).map_err(|error| {
            self.failure = Some(error);
            io::Error::other("applicability subject encoding refused")
        })
    }
}

/// Serialize borrowed originals directly into a capped digest sink under the existing ledger.
fn subject_hash(
    value: &impl Serialize,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<String, ContractError> {
    ledger.checkpoint(control)?;
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

/// Return only fixed native classification and explicit scope labels, sorted before String growth.
fn context_codes(
    classification: GapClassification,
    state: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<ContextSnapshot, ContractError> {
    ledger.checkpoint(control)?;
    ledger.bytes(state.len())?;
    let scope = match state {
        "applicable" => "scope-applicable",
        "not-applicable" => "scope-not-applicable",
        "deferred" => "scope-deferred",
        "under-review" => "scope-under-review",
        _ => return Err(ContractError::Binding),
    };
    let mut codes = ["applicability-re-review", classification.as_str(), scope];
    let bytes = codes
        .iter()
        .try_fold(0_usize, |n, code| n.checked_add(code.len()).ok_or(ContractError::Capacity))?;
    ledger.visits(6)?;
    ledger.bytes(bytes.checked_mul(6).ok_or(ContractError::Capacity)?)?;
    codes.sort_unstable();
    ledger.derived(
        bytes
            .checked_add(3 * std::mem::size_of::<String>())
            .and_then(|n| n.checked_add(64))
            .ok_or(ContractError::Capacity)?,
    )?;
    Ok(ContextSnapshot {
        reason_codes: codes.into_iter().map(str::to_string).collect(),
        related_subject_ids: Vec::new(),
    })
}

/// Meter both complete contexts before structural equality, even when fields mismatch early.
fn charge_contexts(
    left: &ContextSnapshot,
    right: &ContextSnapshot,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    let mut bytes = 0_usize;
    for context in [left, right] {
        ledger.visits(
            context
                .reason_codes
                .len()
                .checked_add(context.related_subject_ids.len())
                .ok_or(ContractError::Capacity)?,
        )?;
        for text in context.reason_codes.iter().chain(&context.related_subject_ids) {
            bytes = bytes.checked_add(text.len()).ok_or(ContractError::Capacity)?;
        }
    }
    ledger.bytes(bytes)
}

/// Meter every complete pin field on each side before equality; these pins alone confer no proof.
fn charge_pins(pins: &[SourcePin], ledger: &mut ContractLedger) -> Result<(), ContractError> {
    ledger.visits(pins.len())?;
    let bytes = pins.iter().try_fold(0_usize, |n, pin| {
        [
            pin.artifact_key.len(),
            pin.raw_sha256.len(),
            pin.schema_identity.len(),
            pin.native_root_uuid.as_ref().map_or(0, String::len),
            64,
        ]
        .into_iter()
        .try_fold(n, |n, extent| n.checked_add(extent).ok_or(ContractError::Capacity))
    })?;
    ledger.bytes(bytes)
}

/// Admit complete two-sided source-key equality, never a truncated or selected roster prefix.
fn charge_source_keys(
    keys: &[String],
    pins: &[SourcePin],
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    ledger.visits(keys.len().checked_add(pins.len()).ok_or(ContractError::Capacity)?)?;
    let key_bytes = keys
        .iter()
        .try_fold(0_usize, |n, key| n.checked_add(key.len()).ok_or(ContractError::Capacity))?;
    let all_bytes = pins.iter().try_fold(key_bytes, |n, pin| {
        n.checked_add(pin.artifact_key.len()).ok_or(ContractError::Capacity)
    })?;
    ledger.bytes(all_bytes)
}

#[cfg(test)]
/// Pure primitive controls; no fixture constructs a sealed applicability approval.
#[path = "applicability_tests.rs"]
mod tests;
