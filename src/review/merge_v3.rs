// A-authored separately typed /3 successor; old /1 and /2 source bodies are untouched.
//! Separate /3 typed-family successor; no /1 document or source-pin cast.
//! Pending declared-policy evaluation from exact borrowed queue/response originals.
//! This module does not serialize a disposition bundle or issue current quorum.
//! Native complete-currentness, physical capture and publication are separate
//! Root-owned ports. Asserted identities, role claims and times are unauthenticated.

use super::chain::{compare, reserved};
use super::chain_v3::{self, ResponseRegistryV3, visit};
use super::decode::{ContractError, ContractLedger};
use super::decode_v3::DecodedV3;
use super::quorum_v3;
use super::validate;
use super::wire::{
    Disposition, MetSeat, RecordedResponse, ResponseClassification, ReviewPolicy, UnmetSeat,
};
use super::wire_v3::{QueueDocumentV3, ResponseDocumentV3, ReviewItemV3, SourcePinV3};
use crate::workspace::preparation::WorkControl;

/// Tentative asserted-policy outcome, deliberately not the public item state.
/// `SeatsFilled` cannot be displayed as current quorum until genuine closure binds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TentativeStateV3 {
    /// No assigned/substitute participation is declared.
    Unassigned,
    /// Eligible assignments/substitutes exist but no counted participation.
    Assigned,
    /// Participation exists, including a blocking lone rejection.
    InReview,
    /// Divergent active asserted dispositions are preserved without a winner.
    Conflicted,
    /// Active request-changes blocks clean seat satisfaction.
    ChangesRequested,
    /// Maximum matching fills every seat, with no active blocker.
    SeatsFilled,
    /// Unsatisfied policy at or after the explicit due second.
    Expired,
    /// Only mismatching declared-binding evidence remains; no native stale claim.
    BindingStale,
}

/// Complete pending item facts, without any native transition or approval token.
pub(crate) struct ItemPolicyFactsV3 {
    /// Selected original queue-local item key.
    pub(crate) item_key: String,
    /// Selected original deterministic item identity.
    pub(crate) item_id: String,
    /// Declared-policy outcome only.
    pub(crate) tentative_state: TentativeStateV3,
    /// All finite reason facts in lexical order, including currentness limitation.
    pub(crate) reason_codes: Vec<String>,
    /// Complete positive declared approval denominator.
    pub(crate) required_seats: u32,
    /// Exact active approval witnesses, one asserted key per seat.
    pub(crate) met_seats: Vec<MetSeat>,
    /// Complete remaining declared seats.
    pub(crate) unmet_seats: Vec<UnmetSeat>,
    /// Every matching item-key response identity, including foreign/stale facts.
    pub(crate) response_ids: Vec<String>,
    /// Every reject/request-changes identity, including withdrawn/history facts.
    pub(crate) dissent_ids: Vec<String>,
    /// Active rejection, changes request or divergent disposition remains blocking.
    pub(crate) blocking: bool,
}

/// Complete counts in the pending original cohort; no deduplicated file fiction.
pub(crate) struct PendingCountsV3 {
    /// Number of selected queue items.
    pub(crate) items: usize,
    /// Every actual bounded response original, including exact duplicate holders.
    pub(crate) response_files: usize,
    /// Distinct explicit UUID/raw identities.
    pub(crate) unique_responses: usize,
    /// Additional byte-identical UUID originals.
    pub(crate) exact_duplicates: usize,
}

/// Opaque pending owner. No Serialize, Clone, Debug, detached constructor,
/// success seal or conversion into a current disposition bundle is provided.
pub(crate) struct PendingPolicyEvaluationV3<'a> {
    /// Actual decoded queue and complete original-byte lifetime.
    queue: &'a DecodedV3<'a, QueueDocumentV3>,
    /// Every actual original response holder and validated chain.
    registry: ResponseRegistryV3<'a>,
    /// Explicit canonical evaluation time; never an ambient machine clock.
    as_of: String,
    /// All unique minimized response facts, with complete raw pins.
    responses: Vec<RecordedResponse>,
    /// All selected pending item facts, with dissent and full seats.
    items: Vec<ItemPolicyFactsV3>,
    /// Full file/identity denominators.
    counts: PendingCountsV3,
}

impl<'a> PendingPolicyEvaluationV3<'a> {
    /// Borrow the exact queue owner required by a future genuine currentness gate.
    pub(crate) fn queue(&self) -> &'a DecodedV3<'a, QueueDocumentV3> {
        self.queue
    }
    /// Borrow every actual original, including additional exact duplicate files.
    pub(crate) fn originals(&self) -> &'a [DecodedV3<'a, ResponseDocumentV3>] {
        self.registry.originals()
    }
    /// Borrow the explicit declared evaluation second.
    pub(crate) fn as_of(&self) -> &str {
        &self.as_of
    }
    /// Borrow pending minimized evidence; it grants no currentness or authority.
    pub(crate) fn responses(&self) -> &[RecordedResponse] {
        &self.responses
    }
    /// Borrow tentative policy outcomes, never public current quorum states.
    pub(crate) fn items(&self) -> &[ItemPolicyFactsV3] {
        &self.items
    }
    /// Borrow complete original denominators.
    pub(crate) fn counts(&self) -> &PendingCountsV3 {
        &self.counts
    }
}

/// Copy only an admitted minimized extent, with work/storage charged before growth.
fn text(value: &str, ledger: &mut ContractLedger) -> Result<String, ContractError> {
    ledger.bytes(value.len())?;
    ledger.derived(value.len())?;
    let mut result = String::new();
    result.try_reserve_exact(value.len()).map_err(|_| ContractError::Capacity)?;
    result.push_str(value);
    Ok(result)
}

/// Precharge both complete typed source-pin extents before equality comparisons.
fn pin_work(
    pins: &[SourcePinV3],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    for pin in pins {
        visit(ledger, control)?;
        let mut extent = 64_usize;
        for size in [
            pin.artifact_key.len(),
            pin.schema_identity.as_ref().map_or(0, String::len),
            pin.validation_profile.len(),
            pin.raw_sha256.len(),
            pin.native_root_uuid.as_ref().map_or(0, String::len),
        ] {
            extent = extent.checked_add(size).ok_or(ContractError::Capacity)?;
        }
        ledger.bytes(extent)?;
    }
    Ok(())
}

/// Inspect all declared-binding fields before deciding stale versus eligibility.
fn snapshot_matches(
    queue: &DecodedV3<'_, QueueDocumentV3>,
    item: &ReviewItemV3,
    response: &ResponseDocumentV3,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<bool, ContractError> {
    pin_work(&queue.document().source_pins, ledger, control)?;
    pin_work(&response.source_pins, ledger, control)?;
    let pairs = [
        (&response.queue_raw_sha256[..], queue.raw_sha256()),
        (&response.item_id, &item.item_id),
        (&response.adapter_version, &item.adapter_version),
        (&response.subject_sha256, &item.subject_sha256),
        (&response.context_sha256, &item.context_sha256),
        (&response.policy_sha256, &item.policy_sha256),
    ];
    let mut equal = true;
    for (left, right) in pairs {
        visit(ledger, control)?;
        equal &= compare(left, right, ledger)?.is_eq();
    }
    ledger.bytes(16)?;
    Ok(equal
        && response.domain == item.domain
        && response.requested_action == item.requested_action
        && response.source_pins == queue.document().source_pins)
}

/// Exact item lookup in the structurally admitted sorted roster, without cloning.
fn selected<'a>(
    queue: &'a QueueDocumentV3,
    key: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Option<&'a ReviewItemV3>, ContractError> {
    let mut low = 0;
    let mut high = queue.items.len();
    while low < high {
        visit(ledger, control)?;
        let middle = low + (high - low) / 2;
        match compare(&queue.items[middle].key, key, ledger)? {
            std::cmp::Ordering::Less => low = middle + 1,
            std::cmp::Ordering::Greater => high = middle,
            std::cmp::Ordering::Equal => return Ok(Some(&queue.items[middle])),
        }
    }
    Ok(None)
}

/// Scan every declared reviewer with checkpoints and both key extents charged.
/// Unknown asserted keys remain nonapproving evidence; structural binding errors
/// still propagate through the unchanged response validator.
fn reviewer_is_declared(
    queue: &QueueDocumentV3,
    key: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<bool, ContractError> {
    let mut declared = false;
    for reviewer in &queue.reviewers {
        visit(ledger, control)?;
        declared |= compare(&reviewer.key, key, ledger)?.is_eq();
    }
    Ok(declared)
}

/// Classify declared binding/eligibility/time; no native freshness is inferred.
fn classify(
    queue: &DecodedV3<'_, QueueDocumentV3>,
    response: &DecodedV3<'_, ResponseDocumentV3>,
    as_of: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<ResponseClassification, ContractError> {
    visit(ledger, control)?;
    let document = response.document();
    if !compare(&document.queue_id, &queue.document().queue_id, ledger)?.is_eq() {
        return Ok(ResponseClassification::Foreign);
    }
    let Some(item) = selected(queue.document(), &document.item_key, ledger, control)? else {
        return Ok(ResponseClassification::Foreign);
    };
    if !snapshot_matches(queue, item, document, ledger, control)?
        || compare(&document.responded_at, &queue.document().created_at, ledger)?.is_lt()
    {
        return Ok(ResponseClassification::Stale);
    }
    if !reviewer_is_declared(queue.document(), &document.reviewer_key, ledger, control)? {
        return Ok(ResponseClassification::Unassigned);
    }
    match super::authoring_response_binding::bind_response(queue, response, ledger, control) {
        Ok(()) => {}
        Err(ContractError::Binding) => return Ok(ResponseClassification::Unassigned),
        Err(error) => return Err(error),
    }
    if compare(&document.responded_at, as_of, ledger)?.is_gt() {
        return Ok(ResponseClassification::Future);
    }
    if let Some(due) = &item.due_at {
        if !compare(&document.responded_at, due, ledger)?.is_lt() {
            return Ok(ResponseClassification::Late);
        }
    }
    Ok(ResponseClassification::Current)
}

/// Retire only exact eligible timely withdrawals. Withdrawal-of-withdrawal never
/// reopens a prior vote, and future/late markers cannot erase a timely approval.
fn retire(
    registry: &ResponseRegistryV3<'_>,
    classes: &mut [ResponseClassification],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let mut targets = reserved(registry.len(), ledger)?;
    for (index, &class) in classes.iter().enumerate().take(registry.len()) {
        visit(ledger, control)?;
        if class == ResponseClassification::Current {
            if let Some(target) = registry.target(index) {
                targets.push(target);
            }
        }
    }
    for target in targets {
        visit(ledger, control)?;
        if classes[target] == ResponseClassification::Current {
            classes[target] = ResponseClassification::Superseded;
        }
    }
    for (index, class) in classes.iter_mut().enumerate() {
        visit(ledger, control)?;
        if *class == ResponseClassification::Current
            && registry.get(index).document().disposition == Disposition::Superseded
        {
            *class = ResponseClassification::Superseded;
        }
    }
    Ok(())
}

/// Fixed disposition bit for detecting divergence without choosing a winner.
fn vote_bit(disposition: Disposition) -> u8 {
    match disposition {
        Disposition::Approve => 1,
        Disposition::Reject => 2,
        Disposition::RequestChanges => 4,
        Disposition::Abstain => 8,
        Disposition::Superseded => 0,
    }
}

/// Preserve every divergent active assertion and exclude conflicted keys from seats.
fn mark_conflicts(
    queue: &QueueDocumentV3,
    registry: &ResponseRegistryV3<'_>,
    classes: &mut [ResponseClassification],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    for item in &queue.items {
        for reviewer in &queue.reviewers {
            visit(ledger, control)?;
            let mut mask = 0_u8;
            for (index, &class) in classes.iter().enumerate() {
                visit(ledger, control)?;
                let response = registry.get(index).document();
                if class == ResponseClassification::Current
                    && compare(&response.item_key, &item.key, ledger)?.is_eq()
                    && compare(&response.reviewer_key, &reviewer.key, ledger)?.is_eq()
                {
                    mask |= vote_bit(response.disposition);
                }
            }
            if mask.count_ones() > 1 {
                for (index, class) in classes.iter_mut().enumerate() {
                    visit(ledger, control)?;
                    let response = registry.get(index).document();
                    if *class == ResponseClassification::Current
                        && compare(&response.item_key, &item.key, ledger)?.is_eq()
                        && compare(&response.reviewer_key, &reviewer.key, ledger)?.is_eq()
                    {
                        *class = ResponseClassification::Conflicted;
                    }
                }
            }
        }
    }
    Ok(())
}

/// Presence of one independent response fact; other facts may coexist.
#[derive(Clone, Copy, Default)]
enum Presence {
    /// No inspected response has contributed this fact.
    #[default]
    Absent,
    /// At least one inspected response has contributed this fact.
    Present,
}

impl Presence {
    /// Preserve an earlier occurrence while including one new independent fact.
    fn include(&mut self, present: bool) {
        if present {
            *self = Self::Present;
        }
    }

    /// Read this fact without excluding any other recorded fact.
    fn is_present(self) -> bool {
        matches!(self, Self::Present)
    }
}

/// Complete finite response facts used for pending state priority and reasons.
/// Each presence field is independent; no mutually exclusive state replaces it.
#[derive(Default)]
struct Evidence {
    /// At least one active eligible disposition, including abstention/conflict.
    active: Presence,
    /// Complete decisive active disposition bit union across asserted keys.
    decisive: u8,
    /// At least one same-key divergent assertion.
    key_conflict: Presence,
    /// Stale declared-binding evidence exists, not a native stale assertion.
    stale: Presence,
    /// At least one asserted response after as-of.
    future: Presence,
    /// At least one ineligible due-boundary/after-due response.
    late: Presence,
    /// At least one nonapproving active abstention.
    abstain: Presence,
    /// Unassigned/asserted-author/incompatible role evidence exists.
    unassigned: Presence,
    /// Withdrawn history or a timely withdrawal marker remains visible.
    superseded: Presence,
}

/// Inspect every item-key response and preserve complete dissent references.
fn evidence(
    item: &ReviewItemV3,
    registry: &ResponseRegistryV3<'_>,
    classes: &[ResponseClassification],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(Evidence, Vec<String>, Vec<String>), ContractError> {
    let mut facts = Evidence::default();
    let mut ids = reserved(registry.len(), ledger)?;
    let mut dissent = reserved(registry.len(), ledger)?;
    for (index, &class) in classes.iter().enumerate() {
        visit(ledger, control)?;
        let response = registry.get(index).document();
        if !compare(&response.item_key, &item.key, ledger)?.is_eq() {
            continue;
        }
        ids.push(text(&response.response_id, ledger)?);
        if matches!(response.disposition, Disposition::Reject | Disposition::RequestChanges) {
            dissent.push(text(&response.response_id, ledger)?);
        }
        match class {
            ResponseClassification::Current | ResponseClassification::Conflicted => {
                facts.active = Presence::Present;
                facts.decisive |= vote_bit(response.disposition) & 7;
                facts.key_conflict.include(class == ResponseClassification::Conflicted);
                facts.abstain.include(response.disposition == Disposition::Abstain);
            }
            ResponseClassification::Stale => facts.stale = Presence::Present,
            ResponseClassification::Future => facts.future = Presence::Present,
            ResponseClassification::Late => facts.late = Presence::Present,
            ResponseClassification::Unassigned => facts.unassigned = Presence::Present,
            ResponseClassification::Superseded => facts.superseded = Presence::Present,
            ResponseClassification::Foreign => {}
        }
    }
    Ok((facts, ids, dissent))
}

/// Preserve explicit state priority while keeping unverified seats distinctly named.
fn state(
    item: &ReviewItemV3,
    policy: &ReviewPolicy,
    facts: &Evidence,
    complete: bool,
    as_of: &str,
    ledger: &mut ContractLedger,
) -> Result<TentativeStateV3, ContractError> {
    if facts.stale.is_present() && !facts.active.is_present() {
        return Ok(TentativeStateV3::BindingStale);
    }
    if facts.key_conflict.is_present() || facts.decisive.count_ones() > 1 {
        return Ok(TentativeStateV3::Conflicted);
    }
    if facts.decisive & 4 != 0 {
        return Ok(TentativeStateV3::ChangesRequested);
    }
    if complete && facts.decisive & 2 == 0 {
        return Ok(TentativeStateV3::SeatsFilled);
    }
    if let Some(due) = &item.due_at {
        if !compare(as_of, due, ledger)?.is_lt() {
            return Ok(TentativeStateV3::Expired);
        }
    }
    if facts.active.is_present() {
        return Ok(TentativeStateV3::InReview);
    }
    if !item.assignments.is_empty() || !policy.substitutions.is_empty() {
        return Ok(TentativeStateV3::Assigned);
    }
    Ok(TentativeStateV3::Unassigned)
}

/// All applicable reason tokens in fixed lexical order; no input prose is exposed.
fn reasons(
    facts: &Evidence,
    state: TentativeStateV3,
    ledger: &mut ContractLedger,
) -> Result<Vec<String>, ContractError> {
    let mut result = reserved(13, ledger)?;
    let flags = [
        (state == TentativeStateV3::SeatsFilled, "approval-seats-filled"),
        (true, "asserted-identity-only"),
        (facts.stale.is_present(), "binding-stale"),
        (
            facts.key_conflict.is_present() || facts.decisive.count_ones() > 1,
            "conflicting-dispositions",
        ),
        (true, "currentness-unverified"),
        (state == TentativeStateV3::Expired, "deadline-unsatisfied"),
        (facts.future.is_present(), "future-responses"),
        (facts.late.is_present(), "late-responses"),
        (facts.abstain.is_present(), "nonapproving-abstention"),
        (facts.decisive & 2 != 0, "rejection-blocks"),
        (facts.decisive & 4 != 0, "request-changes-blocks"),
        (facts.unassigned.is_present(), "unassigned-responses"),
        (facts.superseded.is_present(), "withdrawn-history"),
    ];
    for (present, token) in flags {
        ledger.visits(1)?;
        if present {
            result.push(text(token, ledger)?);
        }
    }
    Ok(result)
}

/// Resolve the actual declared policy through a charged finite scan.
fn policy<'a>(
    queue: &'a QueueDocumentV3,
    item: &ReviewItemV3,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<&'a ReviewPolicy, ContractError> {
    for policy in &queue.policies {
        visit(ledger, control)?;
        if compare(&policy.key, &item.policy_key, ledger)?.is_eq() {
            return Ok(policy);
        }
    }
    Err(ContractError::Invalid)
}

/// One complete pending item, with precharged owned projection and full witnesses.
fn item_facts(
    queue: &QueueDocumentV3,
    item: &ReviewItemV3,
    registry: &ResponseRegistryV3<'_>,
    classes: &[ResponseClassification],
    as_of: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<ItemPolicyFactsV3, ContractError> {
    let policy = policy(queue, item, ledger, control)?;
    let (facts, response_ids, dissent_ids) = evidence(item, registry, classes, ledger, control)?;
    let matched = quorum_v3::evaluate(queue, item, policy, registry, classes, ledger, control)?;
    let mut met = reserved(matched.seats.len(), ledger)?;
    let mut unmet = reserved(matched.seats.len(), ledger)?;
    for (seat, witness) in matched.seats.iter().zip(&matched.witnesses) {
        visit(ledger, control)?;
        match witness {
            Some(index) => {
                let response = registry.get(*index).document();
                met.push(MetSeat {
                    role_key: text(seat.role, ledger)?,
                    ordinal: seat.ordinal,
                    reviewer_key: text(&response.reviewer_key, ledger)?,
                    response_id: text(&response.response_id, ledger)?,
                });
            }
            None => {
                unmet.push(UnmetSeat { role_key: text(seat.role, ledger)?, ordinal: seat.ordinal });
            }
        }
    }
    let tentative = state(item, policy, &facts, unmet.is_empty(), as_of, ledger)?;
    Ok(ItemPolicyFactsV3 {
        item_key: text(&item.key, ledger)?,
        item_id: text(&item.item_id, ledger)?,
        tentative_state: tentative,
        reason_codes: reasons(&facts, tentative, ledger)?,
        required_seats: u32::try_from(matched.seats.len()).map_err(|_| ContractError::Capacity)?,
        met_seats: met,
        unmet_seats: unmet,
        response_ids,
        dissent_ids,
        blocking: facts.key_conflict.is_present()
            || facts.decisive.count_ones() > 1
            || facts.decisive & 6 != 0,
    })
}

/// Minimize an actual original without replacing its raw holder or rationale.
fn recorded(
    response: &DecodedV3<'_, ResponseDocumentV3>,
    classification: ResponseClassification,
    ledger: &mut ContractLedger,
) -> Result<RecordedResponse, ContractError> {
    let document = response.document();
    Ok(RecordedResponse {
        response_id: text(&document.response_id, ledger)?,
        raw_sha256: text(response.raw_sha256(), ledger)?,
        byte_length: u64::try_from(response.raw().len()).map_err(|_| ContractError::Capacity)?,
        item_key: text(&document.item_key, ledger)?,
        reviewer_key: text(&document.reviewer_key, ledger)?,
        reviewer_role: text(&document.reviewer_role, ledger)?,
        disposition: document.disposition,
        responded_at: text(&document.responded_at, ledger)?,
        classification,
    })
}

/// Prepare complete in-memory chain and asserted-policy facts atomically.
/// All raw inputs must have been decoded with this same command ledger; caller
/// IO owns complete attempt/physical-alias counts. Native currentness is absent.
pub(crate) fn prepare_policy<'a>(
    queue: &'a DecodedV3<'a, QueueDocumentV3>,
    responses: &'a [DecodedV3<'a, ResponseDocumentV3>],
    as_of: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<PendingPolicyEvaluationV3<'a>, ContractError> {
    super::decode_v2::phase(ledger, control, |ledger, control| {
        ledger.checkpoint(control)?;
        ledger.bytes(as_of.len())?;
        ledger.derived(84)?;
        validate::time(as_of)?;
        if compare(as_of, &queue.document().created_at, ledger)?.is_lt() {
            return Err(ContractError::Invalid);
        }
        let registry = chain_v3::prepare(responses, ledger, control)?;
        let mut classes = reserved(registry.len(), ledger)?;
        for index in 0..registry.len() {
            classes.push(classify(queue, registry.get(index), as_of, ledger, control)?);
        }
        retire(&registry, &mut classes, ledger, control)?;
        mark_conflicts(queue.document(), &registry, &mut classes, ledger, control)?;
        let mut items = reserved(queue.document().items.len(), ledger)?;
        for item in &queue.document().items {
            visit(ledger, control)?;
            items.push(item_facts(
                queue.document(),
                item,
                &registry,
                &classes,
                as_of,
                ledger,
                control,
            )?);
        }
        let mut projected = reserved(registry.len(), ledger)?;
        for (index, class) in classes.into_iter().enumerate() {
            visit(ledger, control)?;
            projected.push(recorded(registry.get(index), class, ledger)?);
        }
        let counts = PendingCountsV3 {
            items: items.len(),
            response_files: responses.len(),
            unique_responses: registry.len(),
            exact_duplicates: registry.duplicates(),
        };
        let as_of = text(as_of, ledger)?;
        ledger.checkpoint(control)?;
        Ok(PendingPolicyEvaluationV3 {
            queue,
            registry,
            as_of,
            responses: projected,
            items,
            counts,
        })
    })
}
