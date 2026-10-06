//! Pure response-chain/policy controls using inert asserted fixtures.
//! These controls do not construct native captures, authentic people or a current
//! disposition bundle. Four fixture helper bodies are copied exactly from the
//! frozen contracts V2 test source; provenance is retained in this packet.

use super::decode::{ContractError, ContractLedger, decode_queue, decode_response};
use super::merge::{PendingPolicyEvaluation, TentativeState, prepare_policy};
use super::validate;
use super::wire::{
    AbstentionRule, Assignment, AuthorSeparation, ContextSnapshot, Disposition, Domain,
    IDENTITY_DISCLAIMER, QueueDocument, RequestedAction, ResponseClassification, ResponseDocument,
    ReviewItem, ReviewPolicy, Reviewer, RoleDefinition, SeatRequirement, Sensitivity, SourceModel,
    SourcePin, Substitution, SupersessionReference,
};
use crate::workspace::preparation::{
    Interruption, NoopControl, ProgressUpdate, Stage, WorkControl, WorkError,
};
use serde::{Serialize, de::DeserializeOwned};

/// Copy an inert typed fixture through ordinary JSON; never construct a proof.
fn copied<T: Serialize + DeserializeOwned>(value: &T) -> T {
    serde_json::from_value(serde_json::to_value(value).unwrap()).unwrap()
}

/// One closed declared queue with no actual native approval/currentness claim.
fn queue_fixture() -> QueueDocument {
    let mut queue = QueueDocument {
        schema_version: "forge.review-queue/1".into(),
        identity_disclaimer: IDENTITY_DISCLAIMER.into(),
        sensitivity: Sensitivity::IdsAndHashes,
        queue_id: "22222222-2222-4222-8222-222222222222".into(),
        created_at: "2026-10-04T00:00:00Z".into(),
        source_pins: vec![SourcePin {
            artifact_key: "mapping".into(),
            model: SourceModel::Mapping,
            native_root_uuid: Some("11111111-1111-4111-8111-111111111111".into()),
            raw_sha256: "0".repeat(64),
            byte_length: 100,
            schema_identity: "oscal.mapping/1".into(),
        }],
        roles: vec![RoleDefinition { key: "review".into() }],
        reviewers: vec![Reviewer { key: "reviewer".into(), role_keys: vec!["review".into()] }],
        policies: vec![ReviewPolicy {
            key: "policy".into(),
            seats: vec![SeatRequirement { role_key: "review".into(), count: 1 }],
            substitutions: vec![],
            abstention_rule: AbstentionRule::Nonapproving,
            empty_abstention_reasons: vec!["no-conflict".into()],
            author_separation: AuthorSeparation::DeclaredKeys,
        }],
        items: vec![],
    };
    let mut item = ReviewItem {
        key: "item".into(),
        item_id: String::new(),
        domain: Domain::MappingAssertion,
        adapter_version: "forge.mapping-review/1".into(),
        subject_id: "map.1".into(),
        requested_action: RequestedAction::ReReview,
        source_keys: vec!["mapping".into()],
        subject_sha256: "1".repeat(64),
        context: ContextSnapshot {
            reason_codes: vec!["rereview".into()],
            related_subject_ids: vec!["control.1".into()],
        },
        context_sha256: String::new(),
        policy_key: "policy".into(),
        policy_sha256: String::new(),
        author_keys: vec!["author".into()],
        assignments: vec![Assignment {
            reviewer_key: "reviewer".into(),
            role_key: "review".into(),
        }],
        due_at: Some("2026-10-05T00:00:00Z".into()),
        allowed_dispositions: vec![
            Disposition::Approve,
            Disposition::Reject,
            Disposition::RequestChanges,
            Disposition::Abstain,
            Disposition::Superseded,
        ],
    };
    let mut ledger = ContractLedger::default();
    item.context_sha256 = validate::context_hash(&queue, &item, &mut ledger).unwrap();
    item.policy_sha256 =
        validate::policy_hash(&queue, &queue.policies[0], &item, &mut ledger).unwrap();
    item.item_id = validate::item_id(&queue, &item, &mut ledger).unwrap();
    queue.items.push(item);
    queue
}

/// Rebind changed inert item declarations; no actual native pin is recalculated.
fn rebind(queue: &mut QueueDocument) {
    let mut item = queue.items.remove(0);
    let mut ledger = ContractLedger::default();
    item.context_sha256 = validate::context_hash(queue, &item, &mut ledger).unwrap();
    item.policy_sha256 =
        validate::policy_hash(queue, &queue.policies[0], &item, &mut ledger).unwrap();
    item.item_id = validate::item_id(queue, &item, &mut ledger).unwrap();
    queue.items.push(item);
}

/// Private response fixture bound to the exact serialized queue original.
fn response_fixture(queue: &QueueDocument, queue_raw: &[u8]) -> ResponseDocument {
    let item = &queue.items[0];
    ResponseDocument {
        schema_version: "forge.review-response/1".into(),
        identity_disclaimer: IDENTITY_DISCLAIMER.into(),
        response_id: "33333333-3333-4333-8333-333333333333".into(),
        queue_id: queue.queue_id.clone(),
        queue_raw_sha256: crate::hashing::sha256_hex(queue_raw),
        item_key: item.key.clone(),
        item_id: item.item_id.clone(),
        domain: item.domain,
        adapter_version: item.adapter_version.clone(),
        requested_action: item.requested_action,
        source_pins: copied(&queue.source_pins),
        subject_sha256: item.subject_sha256.clone(),
        context_sha256: item.context_sha256.clone(),
        policy_sha256: item.policy_sha256.clone(),
        reviewer_key: "reviewer".into(),
        reviewer_role: "review".into(),
        disposition: Disposition::Approve,
        responded_at: "2026-10-04T12:00:00Z".into(),
        rationale: "Declared review rationale.".into(),
        abstention_reason: None,
        proposed_edit: (),
        supersedes: None,
    }
}

/// Owned test-only projection; it cannot construct or finalize a production owner.
#[derive(Debug)]
struct Summary {
    /// Complete item tentative state sequence.
    states: Vec<TentativeState>,
    /// Complete unique response classification sequence.
    classes: Vec<ResponseClassification>,
    /// Items/files/unique/additional exact duplicate denominators.
    counts: [usize; 4],
    /// Per-item met role/ordinal/key/original UUID tuples.
    met: Vec<Vec<(String, u32, String, String)>>,
    /// Per-item complete unmet seat counts.
    unmet: Vec<usize>,
    /// Per-item actual active blocker facts.
    blocking: Vec<bool>,
    /// Complete per-item historical dissent original IDs.
    dissent: Vec<Vec<String>>,
    /// Complete finite reason facts.
    reasons: Vec<Vec<String>>,
    /// Complete per-item response reference counts.
    references: Vec<usize>,
}

/// Snapshot only pending policy facts before their actual borrowed holders drop.
fn summary(pending: &PendingPolicyEvaluation<'_>) -> Summary {
    Summary {
        states: pending.items().iter().map(|item| item.tentative_state).collect(),
        classes: pending.responses().iter().map(|response| response.classification).collect(),
        counts: [
            pending.counts().items,
            pending.counts().response_files,
            pending.counts().unique_responses,
            pending.counts().exact_duplicates,
        ],
        met: pending
            .items()
            .iter()
            .map(|item| {
                item.met_seats
                    .iter()
                    .map(|seat| {
                        (
                            seat.role_key.clone(),
                            seat.ordinal,
                            seat.reviewer_key.clone(),
                            seat.response_id.clone(),
                        )
                    })
                    .collect()
            })
            .collect(),
        unmet: pending.items().iter().map(|item| item.unmet_seats.len()).collect(),
        blocking: pending.items().iter().map(|item| item.blocking).collect(),
        dissent: pending.items().iter().map(|item| item.dissent_ids.clone()).collect(),
        reasons: pending.items().iter().map(|item| item.reason_codes.clone()).collect(),
        references: pending.items().iter().map(|item| item.response_ids.len()).collect(),
    }
}

/// Decode every actual fixture original with one ledger, then call the real core.
fn run_originals(
    queue_raw: &[u8],
    raws: &[Vec<u8>],
    as_of: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Summary, ContractError> {
    let queue = decode_queue(queue_raw, ledger, control)?;
    let responses = raws
        .iter()
        .map(|raw| decode_response(raw, ledger, control))
        .collect::<Result<Vec<_>, _>>()?;
    let pending = prepare_policy(&queue, &responses, as_of, ledger, control)?;
    Ok(summary(&pending))
}

/// Serialize closed inert fixtures; no file capture or native approval is implied.
fn run(
    queue: &QueueDocument,
    responses: &[ResponseDocument],
    as_of: &str,
) -> Result<Summary, ContractError> {
    let raw = serde_json::to_vec(queue).unwrap();
    let raws =
        responses.iter().map(|response| serde_json::to_vec(response).unwrap()).collect::<Vec<_>>();
    run_originals(&raw, &raws, as_of, &mut ContractLedger::default(), &mut NoopControl)
}

/// Produce an explicit distinct nonnil synthetic UUID, without ambient randomness.
fn id(number: u32) -> String {
    format!("33333333-3333-4333-8333-{number:012x}")
}

/// Reuse exact closed response fixture bindings and explicitly select asserted key.
fn vote(
    queue: &QueueDocument,
    number: u32,
    key: &str,
    role: &str,
    disposition: Disposition,
) -> ResponseDocument {
    let raw = serde_json::to_vec(queue).unwrap();
    let mut response = response_fixture(queue, &raw);
    response.response_id = id(number);
    response.reviewer_key = key.into();
    response.reviewer_role = role.into();
    response.disposition = disposition;
    if disposition == Disposition::Abstain {
        response.abstention_reason = Some("no-conflict".into());
    }
    response
}

/// Exact prior original pin, not a semantic reconstruction or timestamp winner.
fn withdrawal(queue: &QueueDocument, number: u32, prior: &ResponseDocument) -> ResponseDocument {
    let mut response =
        vote(queue, number, &prior.reviewer_key, &prior.reviewer_role, Disposition::Superseded);
    response.supersedes = Some(SupersessionReference {
        response_id: prior.response_id.clone(),
        raw_sha256: crate::hashing::sha256_hex(&serde_json::to_vec(prior).unwrap()),
    });
    response
}

/// Add a genuine declared inert reviewer/assignment, then rebind exact fingerprints.
fn add_reviewer(queue: &mut QueueDocument, key: &str) {
    queue.reviewers.push(Reviewer { key: key.into(), role_keys: vec!["review".into()] });
    queue.reviewers.sort_by(|left, right| left.key.cmp(&right.key));
    queue.items[0]
        .assignments
        .push(Assignment { reviewer_key: key.into(), role_key: "review".into() });
    queue.items[0].assignments.sort_by(|left, right| {
        (&left.reviewer_key, &left.role_key).cmp(&(&right.reviewer_key, &right.role_key))
    });
    rebind(queue);
}

/// Two mixed seats require a real augment: flexible key sorts before audit-only key.
fn mixed_queue() -> QueueDocument {
    let mut queue = queue_fixture();
    queue.roles =
        vec![RoleDefinition { key: "audit".into() }, RoleDefinition { key: "review".into() }];
    queue.reviewers = vec![
        Reviewer { key: "a-flex".into(), role_keys: vec!["review".into()] },
        Reviewer { key: "z-audit".into(), role_keys: vec!["audit".into()] },
    ];
    queue.policies[0].seats = vec![
        SeatRequirement { role_key: "audit".into(), count: 1 },
        SeatRequirement { role_key: "review".into(), count: 1 },
    ];
    queue.policies[0].substitutions = vec![Substitution {
        seat_role: "audit".into(),
        reviewer_key: "a-flex".into(),
        asserted_role: "review".into(),
        reason_code: "declared-substitute".into(),
    }];
    queue.items[0].assignments = vec![
        Assignment { reviewer_key: "a-flex".into(), role_key: "review".into() },
        Assignment { reviewer_key: "z-audit".into(), role_key: "audit".into() },
    ];
    rebind(&mut queue);
    queue
}

/// Stop later actual phases, preserving the maintained typed deadline reason.
struct Stop {
    /// Number of real checkpoints still allowed.
    remaining: usize,
}
impl WorkControl for Stop {
    /// Supply a sticky typed timeout after the selected actual checkpoint count.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> Result<(), WorkError> {
        if self.remaining == 0 {
            Err(WorkError::Interrupted(Interruption::DeadlineExceeded))
        } else {
            self.remaining -= 1;
            Ok(())
        }
    }
    /// Expose the same actual observed timeout, never a fabricated success.
    fn interruption(&self) -> Option<Interruption> {
        if self.remaining == 0 { Some(Interruption::DeadlineExceeded) } else { None }
    }
}

/// No responses preserves declared assignments and the complete positive denominator.
#[test]
fn assigned_without_response_keeps_all_unmet_seats() {
    let result = run(&queue_fixture(), &[], "2026-10-04T12:00:00Z").unwrap();
    assert_eq!(result.states, [TentativeState::Assigned]);
    assert_eq!(result.unmet, [1]);
    assert_eq!(result.met[0], [] as [(String, u32, String, String); 0]);
    assert_eq!(result.counts, [1, 0, 0, 0]);
}

/// A timely eligible approval fills an asserted seat but cannot issue current quorum.
#[test]
fn one_approval_is_only_pending_seat_satisfaction() {
    let queue = queue_fixture();
    let result = run(
        &queue,
        &[vote(&queue, 1, "reviewer", "review", Disposition::Approve)],
        "2026-10-04T12:00:00Z",
    )
    .unwrap();
    assert_eq!(result.states, [TentativeState::SeatsFilled]);
    assert_eq!(result.met[0].len(), 1);
    assert!(result.reasons[0].iter().any(|reason| reason == "currentness-unverified"));
    assert!(!result.blocking[0]);
}

/// Every original holder remains borrowed, including additional exact duplicate bytes.
#[test]
fn all_raw_holders_and_queue_pointer_are_retained() {
    let queue = queue_fixture();
    let queue_raw = serde_json::to_vec(&queue).unwrap();
    let raw =
        serde_json::to_vec(&vote(&queue, 1, "reviewer", "review", Disposition::Approve)).unwrap();
    let mut ledger = ContractLedger::default();
    let decoded_queue = decode_queue(&queue_raw, &mut ledger, &mut NoopControl).unwrap();
    let responses = vec![
        decode_response(&raw, &mut ledger, &mut NoopControl).unwrap(),
        decode_response(&raw, &mut ledger, &mut NoopControl).unwrap(),
    ];
    let pending = prepare_policy(
        &decoded_queue,
        &responses,
        "2026-10-04T12:00:00Z",
        &mut ledger,
        &mut NoopControl,
    )
    .unwrap();
    assert!(std::ptr::eq(std::ptr::from_ref(pending.queue()), std::ptr::from_ref(&decoded_queue),));
    assert!(std::ptr::eq(pending.originals().as_ptr(), responses.as_ptr()));
    assert_eq!(pending.originals().len(), 2);
    assert!(std::ptr::eq(pending.originals()[0].raw().as_ptr(), raw.as_ptr()));
    assert_eq!(pending.as_of(), "2026-10-04T12:00:00Z");
    assert_eq!(pending.items()[0].required_seats, 1);
}

/// Exact UUID/full-byte duplicates count once while every file remains in the denominator.
#[test]
fn exact_duplicates_count_once_without_releasing_originals() {
    let queue = queue_fixture();
    let response = vote(&queue, 1, "reviewer", "review", Disposition::Approve);
    let result =
        run(&queue, &[copied(&response), copied(&response), response], "2026-10-04T12:00:00Z")
            .unwrap();
    assert_eq!(result.counts, [1, 3, 1, 2]);
    assert_eq!(result.met[0].len(), 1);
    assert_eq!(result.references, [1]);
}

/// Same UUID with merely added original whitespace is a whole identity refusal.
#[test]
fn same_id_whitespace_is_not_semantic_duplicate() {
    let queue = queue_fixture();
    let raw_queue = serde_json::to_vec(&queue).unwrap();
    let first =
        serde_json::to_vec(&vote(&queue, 1, "reviewer", "review", Disposition::Approve)).unwrap();
    let mut second = first.clone();
    second.push(b'\n');
    assert!(matches!(
        run_originals(
            &raw_queue,
            &[first, second],
            "2026-10-04T12:00:00Z",
            &mut ContractLedger::default(),
            &mut NoopControl
        ),
        Err(ContractError::IdentityConflict)
    ));
}

/// A differing disposition under the same original UUID refuses before matching.
#[test]
fn same_id_changed_vote_refuses_the_complete_operation() {
    let queue = queue_fixture();
    assert!(matches!(
        run(
            &queue,
            &[
                vote(&queue, 1, "reviewer", "review", Disposition::Approve),
                vote(&queue, 1, "reviewer", "review", Disposition::Reject)
            ],
            "2026-10-04T12:00:00Z"
        ),
        Err(ContractError::IdentityConflict)
    ));
}

/// Different consistent originals from one key never create another approval seat.
#[test]
fn multiple_consistent_approvals_one_key_fill_at_most_one_seat() {
    let mut queue = queue_fixture();
    queue.policies[0].seats[0].count = 2;
    rebind(&mut queue);
    let result = run(
        &queue,
        &[
            vote(&queue, 2, "reviewer", "review", Disposition::Approve),
            vote(&queue, 1, "reviewer", "review", Disposition::Approve),
        ],
        "2026-10-04T12:00:00Z",
    )
    .unwrap();
    assert_eq!(result.counts[2], 2);
    assert_eq!(result.met[0].len(), 1);
    assert_eq!(result.unmet, [1]);
    assert_eq!(result.met[0][0].3, id(1));
}

/// Input-file permutation does not change the deterministic original witness.
#[test]
fn response_order_does_not_choose_a_vote_winner() {
    let queue = queue_fixture();
    let first = vote(&queue, 1, "reviewer", "review", Disposition::Approve);
    let second = vote(&queue, 2, "reviewer", "review", Disposition::Approve);
    let left = run(&queue, &[copied(&first), copied(&second)], "2026-10-04T12:00:00Z").unwrap();
    let right = run(&queue, &[second, first], "2026-10-04T12:00:00Z").unwrap();
    assert_eq!(left.met, right.met);
    assert_eq!(left.classes, right.classes);
    assert_eq!(left.counts, right.counts);
}

/// Divergent live originals from one key remain conflict and fill no seat.
#[test]
fn same_key_approval_and_rejection_are_conflicted() {
    let queue = queue_fixture();
    let result = run(
        &queue,
        &[
            vote(&queue, 1, "reviewer", "review", Disposition::Approve),
            vote(&queue, 2, "reviewer", "review", Disposition::Reject),
        ],
        "2026-10-04T12:00:00Z",
    )
    .unwrap();
    assert_eq!(result.states, [TentativeState::Conflicted]);
    assert_eq!(result.classes, [ResponseClassification::Conflicted; 2]);
    assert_eq!(result.met[0], [] as [(String, u32, String, String); 0]);
    assert_eq!(result.dissent[0], [id(2)]);
    assert!(result.blocking[0]);
}

/// Approval/rejection across distinct declared keys preserves AC-2 conflict facts.
#[test]
fn distinct_key_approval_and_rejection_preserve_dissent() {
    let mut queue = queue_fixture();
    add_reviewer(&mut queue, "z-reviewer");
    let result = run(
        &queue,
        &[
            vote(&queue, 1, "reviewer", "review", Disposition::Approve),
            vote(&queue, 2, "z-reviewer", "review", Disposition::Reject),
        ],
        "2026-10-04T12:00:00Z",
    )
    .unwrap();
    assert_eq!(result.states, [TentativeState::Conflicted]);
    assert_eq!(result.met[0].len(), 1);
    assert!(result.blocking[0]);
    assert_eq!(result.dissent[0], [id(2)]);
}

/// A lone rejection blocks satisfaction without inventing a public rejected state.
#[test]
fn lone_rejection_is_blocking_in_review() {
    let queue = queue_fixture();
    let result = run(
        &queue,
        &[vote(&queue, 1, "reviewer", "review", Disposition::Reject)],
        "2026-10-04T12:00:00Z",
    )
    .unwrap();
    assert_eq!(result.states, [TentativeState::InReview]);
    assert!(result.blocking[0]);
    assert_eq!(result.dissent[0], [id(1)]);
}

/// A current request-changes preserves the full original and blocks clean agreement.
#[test]
fn request_changes_is_nonapproving_and_preserved() {
    let queue = queue_fixture();
    let result = run(
        &queue,
        &[vote(&queue, 1, "reviewer", "review", Disposition::RequestChanges)],
        "2026-10-04T12:00:00Z",
    )
    .unwrap();
    assert_eq!(result.states, [TentativeState::ChangesRequested]);
    assert!(result.blocking[0]);
    assert_eq!(result.dissent[0], [id(1)]);
}

/// Abstention participates without shrinking the positive approval denominator.
#[test]
fn abstention_never_fills_or_removes_a_seat() {
    let queue = queue_fixture();
    let result = run(
        &queue,
        &[vote(&queue, 1, "reviewer", "review", Disposition::Abstain)],
        "2026-10-04T12:00:00Z",
    )
    .unwrap();
    assert_eq!(result.states, [TentativeState::InReview]);
    assert_eq!(result.unmet, [1]);
    assert_eq!(result.met[0], [] as [(String, u32, String, String); 0]);
    assert!(!result.blocking[0]);
}

/// A distinct key's abstention does not turn another key's approval into conflict.
#[test]
fn independent_abstention_preserves_one_valid_approval() {
    let mut queue = queue_fixture();
    add_reviewer(&mut queue, "z-reviewer");
    let result = run(
        &queue,
        &[
            vote(&queue, 1, "reviewer", "review", Disposition::Approve),
            vote(&queue, 2, "z-reviewer", "review", Disposition::Abstain),
        ],
        "2026-10-04T12:00:00Z",
    )
    .unwrap();
    assert_eq!(result.states, [TentativeState::SeatsFilled]);
}

/// A single key's simultaneous approving/abstaining assertions are not timestamp-resolved.
#[test]
fn same_key_approval_and_abstention_exclude_conflicted_key() {
    let queue = queue_fixture();
    let result = run(
        &queue,
        &[
            vote(&queue, 1, "reviewer", "review", Disposition::Approve),
            vote(&queue, 2, "reviewer", "review", Disposition::Abstain),
        ],
        "2026-10-04T12:00:00Z",
    )
    .unwrap();
    assert_eq!(result.states, [TentativeState::Conflicted]);
    assert_eq!(result.met[0], [] as [(String, u32, String, String); 0]);
}

/// Empty rationale counts as abstention only through the exact declared exception.
#[test]
fn empty_abstention_requires_declared_reason() {
    let queue = queue_fixture();
    let mut response = vote(&queue, 1, "reviewer", "review", Disposition::Abstain);
    response.rationale.clear();
    assert_eq!(
        run(&queue, &[copied(&response)], "2026-10-04T12:00:00Z").unwrap().classes,
        [ResponseClassification::Current]
    );
    response.abstention_reason = Some("not-declared".into());
    assert_eq!(
        run(&queue, &[response], "2026-10-04T12:00:00Z").unwrap().classes,
        [ResponseClassification::Unassigned]
    );
}

/// An asserted author response cannot become eligible through key spelling alone.
#[test]
fn author_is_unassigned_and_cannot_fill_a_seat() {
    let queue = queue_fixture();
    let result = run(
        &queue,
        &[vote(&queue, 1, "author", "review", Disposition::Approve)],
        "2026-10-04T12:00:00Z",
    )
    .unwrap();
    assert_eq!(result.classes, [ResponseClassification::Unassigned]);
    assert_eq!(result.met[0], [] as [(String, u32, String, String); 0]);
}

/// An unknown asserted key remains a recorded unassigned fact, never a seat.
#[test]
fn undeclared_reviewer_is_preserved_nonapproving() {
    let queue = queue_fixture();
    let result = run(
        &queue,
        &[vote(&queue, 1, "unknown", "review", Disposition::Approve)],
        "2026-10-04T12:00:00Z",
    )
    .unwrap();
    assert_eq!(result.classes, [ResponseClassification::Unassigned]);
    assert_eq!(result.references, [1]);
}

/// An incompatible asserted role does not borrow another role's permission.
#[test]
fn unassigned_role_cannot_count_as_an_approval() {
    let queue = queue_fixture();
    let result = run(
        &queue,
        &[vote(&queue, 1, "reviewer", "other", Disposition::Approve)],
        "2026-10-04T12:00:00Z",
    )
    .unwrap();
    assert_eq!(result.classes, [ResponseClassification::Unassigned]);
    assert_eq!(result.met[0], [] as [(String, u32, String, String); 0]);
}

/// The original whole queue raw pin remains decision-bearing despite similar fields.
#[test]
fn changed_queue_raw_pin_is_stale_binding_evidence() {
    let queue = queue_fixture();
    let mut response = vote(&queue, 1, "reviewer", "review", Disposition::Approve);
    response.queue_raw_sha256 = "9".repeat(64);
    let result = run(&queue, &[response], "2026-10-04T12:00:00Z").unwrap();
    assert_eq!(result.classes, [ResponseClassification::Stale]);
    assert_eq!(result.states, [TentativeState::BindingStale]);
}

/// Foreign queue identity is retained separately without currentness or seat credit.
#[test]
fn foreign_queue_keeps_raw_evidence_and_item_reference() {
    let queue = queue_fixture();
    let mut response = vote(&queue, 1, "reviewer", "review", Disposition::Reject);
    response.queue_id = "44444444-4444-4444-8444-444444444444".into();
    let result = run(&queue, &[response], "2026-10-04T12:00:00Z").unwrap();
    assert_eq!(result.classes, [ResponseClassification::Foreign]);
    assert_eq!(result.references, [1]);
    assert_eq!(result.dissent[0], [id(1)]);
    assert!(!result.blocking[0]);
}

/// A foreign item key does not enter any selected item denominator.
#[test]
fn foreign_item_has_no_selected_item_reference() {
    let queue = queue_fixture();
    let mut response = vote(&queue, 1, "reviewer", "review", Disposition::Approve);
    response.item_key = "foreign".into();
    let result = run(&queue, &[response], "2026-10-04T12:00:00Z").unwrap();
    assert_eq!(result.classes, [ResponseClassification::Foreign]);
    assert_eq!(result.references, [0]);
    assert_eq!(result.counts[2], 1);
}

/// Stale old evidence cannot supply a seat or erase a valid current assertion.
#[test]
fn stale_history_is_preserved_alongside_current_seat_facts() {
    let queue = queue_fixture();
    let mut stale = vote(&queue, 2, "reviewer", "review", Disposition::Reject);
    stale.context_sha256 = "9".repeat(64);
    let result = run(
        &queue,
        &[vote(&queue, 1, "reviewer", "review", Disposition::Approve), stale],
        "2026-10-04T12:00:00Z",
    )
    .unwrap();
    assert_eq!(result.states, [TentativeState::SeatsFilled]);
    assert_eq!(result.dissent[0], [id(2)]);
    assert!(result.reasons[0].iter().any(|reason| reason == "binding-stale"));
}

/// Future declared responses are visible but cannot vote at the explicit as-of.
#[test]
fn future_approval_is_visible_but_nonapproving() {
    let queue = queue_fixture();
    let mut response = vote(&queue, 1, "reviewer", "review", Disposition::Approve);
    response.responded_at = "2026-10-04T12:00:01Z".into();
    let result = run(&queue, &[response], "2026-10-04T12:00:00Z").unwrap();
    assert_eq!(result.classes, [ResponseClassification::Future]);
    assert_eq!(result.met[0], [] as [(String, u32, String, String); 0]);
}

/// Equality to as-of is eligible; equality to due is explicitly too late.
#[test]
fn due_equality_refuses_counting_even_when_as_of_matches() {
    let queue = queue_fixture();
    let mut response = vote(&queue, 1, "reviewer", "review", Disposition::Approve);
    response.responded_at = queue.items[0].due_at.clone().unwrap();
    let result = run(&queue, &[response], "2026-10-05T00:00:00Z").unwrap();
    assert_eq!(result.classes, [ResponseClassification::Late]);
    assert_eq!(result.states, [TentativeState::Expired]);
}

/// A completed strict-predeadline policy remains satisfied after due, without native authority.
#[test]
fn completed_before_due_remains_pending_satisfied_after_due() {
    let queue = queue_fixture();
    let mut response = vote(&queue, 1, "reviewer", "review", Disposition::Approve);
    response.responded_at = "2026-10-04T23:59:59Z".into();
    let result = run(&queue, &[response], "2026-10-06T00:00:00Z").unwrap();
    assert_eq!(result.states, [TentativeState::SeatsFilled]);
    assert_eq!(result.classes, [ResponseClassification::Current]);
}

/// No approval at the exact due instant yields unsatisfied expiry.
#[test]
fn incomplete_policy_expires_at_due_equality() {
    assert_eq!(
        run(&queue_fixture(), &[], "2026-10-05T00:00:00Z").unwrap().states,
        [TentativeState::Expired]
    );
}

/// No declared due time is never replaced by an ambient expiration.
#[test]
fn no_deadline_remains_assigned_at_later_explicit_time() {
    let mut queue = queue_fixture();
    queue.items[0].due_at = None;
    rebind(&mut queue);
    assert_eq!(
        run(&queue, &[], "2027-01-01T00:00:00Z").unwrap().states,
        [TentativeState::Assigned]
    );
}

/// Evaluation cannot precede the actual explicit queue creation second.
#[test]
fn as_of_before_creation_refuses_the_whole_operation() {
    assert!(matches!(
        run(&queue_fixture(), &[], "2026-10-03T23:59:59Z"),
        Err(ContractError::Invalid)
    ));
}

/// Canonical calendar seconds are required without offsets, fractions or signed years.
#[test]
fn noncanonical_and_impossible_as_of_refuse() {
    for date in [
        "2026-02-29T00:00:00Z",
        "+2026-10-04T12:00:00Z",
        "2026-10-04T12:00:00+00:00",
        "2026-10-04T12:00:00.0Z",
    ] {
        assert!(matches!(run(&queue_fixture(), &[], date), Err(ContractError::Invalid)));
    }
}

/// A valid exact eligible withdrawal retires the prior approval without deleting bytes.
#[test]
fn timely_exact_withdrawal_retires_approval_and_keeps_both_pins() {
    let queue = queue_fixture();
    let prior = vote(&queue, 1, "reviewer", "review", Disposition::Approve);
    let marker = withdrawal(&queue, 2, &prior);
    let result = run(&queue, &[prior, marker], "2026-10-04T12:00:00Z").unwrap();
    assert_eq!(result.classes, [ResponseClassification::Superseded; 2]);
    assert_eq!(result.met[0], [] as [(String, u32, String, String); 0]);
    assert_eq!(result.counts, [1, 2, 2, 0]);
}

/// A future marker cannot retroactively erase an eligible timely assertion.
#[test]
fn future_withdrawal_does_not_retire_active_approval() {
    let queue = queue_fixture();
    let prior = vote(&queue, 1, "reviewer", "review", Disposition::Approve);
    let mut marker = withdrawal(&queue, 2, &prior);
    marker.responded_at = "2026-10-04T12:00:01Z".into();
    let result = run(&queue, &[prior, marker], "2026-10-04T12:00:00Z").unwrap();
    assert_eq!(result.classes, [ResponseClassification::Current, ResponseClassification::Future]);
    assert_eq!(result.states, [TentativeState::SeatsFilled]);
}

/// A marker at the exact due second is late and cannot erase completed-before-due seats.
#[test]
fn due_boundary_withdrawal_does_not_retire_predeadline_approval() {
    let queue = queue_fixture();
    let prior = vote(&queue, 1, "reviewer", "review", Disposition::Approve);
    let mut marker = withdrawal(&queue, 2, &prior);
    marker.responded_at = "2026-10-05T00:00:00Z".into();
    let result = run(&queue, &[prior, marker], "2026-10-06T00:00:00Z").unwrap();
    assert_eq!(result.classes, [ResponseClassification::Current, ResponseClassification::Late]);
    assert_eq!(result.states, [TentativeState::SeatsFilled]);
}

/// Withdrawing a withdrawal never resurrects its already retired prior vote.
#[test]
fn withdrawal_of_withdrawal_never_reopens_an_approval() {
    let queue = queue_fixture();
    let prior = vote(&queue, 1, "reviewer", "review", Disposition::Approve);
    let middle = withdrawal(&queue, 2, &prior);
    let last = withdrawal(&queue, 3, &middle);
    let result = run(&queue, &[last, prior, middle], "2026-10-04T12:00:00Z").unwrap();
    assert_eq!(result.classes, [ResponseClassification::Superseded; 3]);
    assert_eq!(result.met[0], [] as [(String, u32, String, String); 0]);
}

/// Historical withdrawn rejection stays in complete dissent references without blocking a new consistent approval.
#[test]
fn withdrawn_rejection_is_preserved_as_dissent_history() {
    let queue = queue_fixture();
    let rejection = vote(&queue, 1, "reviewer", "review", Disposition::Reject);
    let marker = withdrawal(&queue, 2, &rejection);
    let approval = vote(&queue, 3, "reviewer", "review", Disposition::Approve);
    let result = run(&queue, &[rejection, marker, approval], "2026-10-04T12:00:00Z").unwrap();
    assert_eq!(result.dissent[0], [id(1)]);
    assert_eq!(result.states, [TentativeState::SeatsFilled]);
}

/// Missing exact prior originals refuse every derived matching result.
#[test]
fn missing_prior_original_is_a_whole_chain_refusal() {
    let queue = queue_fixture();
    let prior = vote(&queue, 1, "reviewer", "review", Disposition::Approve);
    assert!(matches!(
        run(&queue, &[withdrawal(&queue, 2, &prior)], "2026-10-04T12:00:00Z"),
        Err(ContractError::InvalidChain)
    ));
}

/// A matching prior UUID with an incorrect complete raw SHA is not a withdrawal.
#[test]
fn wrong_prior_raw_pin_refuses_the_chain() {
    let queue = queue_fixture();
    let prior = vote(&queue, 1, "reviewer", "review", Disposition::Approve);
    let mut marker = withdrawal(&queue, 2, &prior);
    marker.supersedes.as_mut().unwrap().raw_sha256 = "f".repeat(64);
    assert!(matches!(
        run(&queue, &[prior, marker], "2026-10-04T12:00:00Z"),
        Err(ContractError::InvalidChain)
    ));
}

/// A different asserted reviewer cannot withdraw another key's original response.
#[test]
fn cross_reviewer_withdrawal_refuses_even_with_exact_prior_hash() {
    let mut queue = queue_fixture();
    add_reviewer(&mut queue, "z-reviewer");
    let prior = vote(&queue, 1, "reviewer", "review", Disposition::Approve);
    let mut marker = withdrawal(&queue, 2, &prior);
    marker.reviewer_key = "z-reviewer".into();
    assert!(matches!(
        run(&queue, &[prior, marker], "2026-10-04T12:00:00Z"),
        Err(ContractError::InvalidChain)
    ));
}

/// An exact prior UUID/hash cannot cross the complete queue/item/policy snapshot.
#[test]
fn changed_binding_withdrawal_refuses_before_retirement() {
    let queue = queue_fixture();
    let prior = vote(&queue, 1, "reviewer", "review", Disposition::Approve);
    let mut marker = withdrawal(&queue, 2, &prior);
    marker.policy_sha256 = "f".repeat(64);
    assert!(matches!(
        run(&queue, &[prior, marker], "2026-10-04T12:00:00Z"),
        Err(ContractError::InvalidChain)
    ));
}

/// Two different successors of one original are ambiguity, including late/future branches.
#[test]
fn competing_successor_branches_refuse_without_timestamp_winner() {
    let queue = queue_fixture();
    let prior = vote(&queue, 1, "reviewer", "review", Disposition::Approve);
    let first = withdrawal(&queue, 2, &prior);
    let mut second = withdrawal(&queue, 3, &prior);
    second.responded_at = "2026-10-06T00:00:00Z".into();
    assert!(matches!(
        run(&queue, &[prior, first, second], "2026-10-04T12:00:00Z"),
        Err(ContractError::InvalidChain)
    ));
}

/// Actual mixed-role edges need an augmenting path rather than greedy first-seat assignment.
#[test]
fn mixed_roles_find_maximum_matching_not_greedy_subset() {
    let queue = mixed_queue();
    let result = run(
        &queue,
        &[
            vote(&queue, 1, "a-flex", "review", Disposition::Approve),
            vote(&queue, 2, "z-audit", "audit", Disposition::Approve),
        ],
        "2026-10-04T12:00:00Z",
    )
    .unwrap();
    assert_eq!(result.states, [TentativeState::SeatsFilled]);
    assert_eq!(result.unmet[0], 0);
    assert_eq!(result.met[0][0].2, "z-audit");
    assert_eq!(result.met[0][1].2, "a-flex");
}

/// One flexible key cannot satisfy two declared role seats through substitution.
#[test]
fn flexible_key_still_fills_only_one_mixed_role_seat() {
    let queue = mixed_queue();
    let result = run(
        &queue,
        &[vote(&queue, 1, "a-flex", "review", Disposition::Approve)],
        "2026-10-04T12:00:00Z",
    )
    .unwrap();
    assert_eq!(result.met[0].len(), 1);
    assert_eq!(result.unmet, [1]);
}

/// Removing the explicit cross-role edge removes its seat eligibility, not the denominator.
#[test]
fn absent_substitution_cannot_create_cross_role_approval() {
    let mut queue = mixed_queue();
    queue.policies[0].substitutions.clear();
    rebind(&mut queue);
    let result = run(
        &queue,
        &[vote(&queue, 1, "a-flex", "review", Disposition::Approve)],
        "2026-10-04T12:00:00Z",
    )
    .unwrap();
    assert_eq!(result.met[0][0].0, "review");
    assert_eq!(result.unmet, [1]);
}

/// A valid declared empty assignment policy remains explicitly unassigned.
#[test]
fn no_assignments_or_substitutes_is_unassigned() {
    let mut queue = queue_fixture();
    queue.items[0].assignments.clear();
    rebind(&mut queue);
    assert_eq!(
        run(&queue, &[], "2026-10-04T12:00:00Z").unwrap().states,
        [TentativeState::Unassigned]
    );
}

/// Actual matching step exhaustion is sticky before any partial seat result returns.
#[test]
fn exhausted_matching_budget_refuses_and_stays_latched() {
    let queue = queue_fixture();
    let raw = serde_json::to_vec(&queue).unwrap();
    let response =
        serde_json::to_vec(&vote(&queue, 1, "reviewer", "review", Disposition::Approve)).unwrap();
    let mut ledger = ContractLedger::default();
    let queue = decode_queue(&raw, &mut ledger, &mut NoopControl).unwrap();
    let responses = [decode_response(&response, &mut ledger, &mut NoopControl).unwrap()];
    ledger.matching(10_000_000).unwrap();
    assert!(matches!(
        prepare_policy(&queue, &responses, "2026-10-04T12:00:00Z", &mut ledger, &mut NoopControl),
        Err(ContractError::Capacity)
    ));
    assert!(matches!(
        decode_queue(&raw, &mut ledger, &mut NoopControl),
        Err(ContractError::Capacity)
    ));
}

/// Shared derived storage exhaustion is not reset by a nested pure core.
#[test]
fn exhausted_derived_budget_refuses_before_registry_growth() {
    let queue = queue_fixture();
    let raw = serde_json::to_vec(&queue).unwrap();
    let mut ledger = ContractLedger::default();
    let queue = decode_queue(&raw, &mut ledger, &mut NoopControl).unwrap();
    let initial = ledger.derived(33_554_432);
    assert!(matches!(initial, Err(ContractError::Capacity)));
    assert!(matches!(
        prepare_policy(&queue, &[], "2026-10-04T12:00:00Z", &mut ledger, &mut NoopControl),
        Err(ContractError::Capacity)
    ));
}

/// Shared byte-work exhaustion cannot be converted into an empty policy success.
#[test]
fn exhausted_byte_work_refuses_without_empty_success() {
    let queue = queue_fixture();
    let raw = serde_json::to_vec(&queue).unwrap();
    let mut ledger = ContractLedger::default();
    let queue = decode_queue(&raw, &mut ledger, &mut NoopControl).unwrap();
    assert!(matches!(ledger.bytes(268_435_456), Err(ContractError::Capacity)));
    assert!(matches!(
        prepare_policy(&queue, &[], "2026-10-04T12:00:00Z", &mut ledger, &mut NoopControl),
        Err(ContractError::Capacity)
    ));
}

/// A caller timeout after actual decode stops the core with the same typed reason.
#[test]
fn control_timeout_is_preserved_and_sticky() {
    let raw = serde_json::to_vec(&queue_fixture()).unwrap();
    let mut ledger = ContractLedger::default();
    let queue = decode_queue(&raw, &mut ledger, &mut NoopControl).unwrap();
    assert!(matches!(
        prepare_policy(
            &queue,
            &[],
            "2026-10-04T12:00:00Z",
            &mut ledger,
            &mut Stop { remaining: 0 }
        ),
        Err(ContractError::Interrupted(Interruption::DeadlineExceeded))
    ));
    assert!(matches!(
        decode_queue(&raw, &mut ledger, &mut NoopControl),
        Err(ContractError::Interrupted(Interruption::DeadlineExceeded))
    ));
}

/// An ordinary shared-control failure is fixed, sticky and never stale evidence.
struct FailedControl;
impl WorkControl for FailedControl {
    /// Supply the maintained fixed invalid error through the actual failure branch.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> Result<(), WorkError> {
        Err(WorkError::Failed(crate::workspace::contract::Error::invalid()))
    }
    /// Ordinary failure does not invent a cancellation reason.
    fn interruption(&self) -> Option<Interruption> {
        None
    }
}

/// Shared ordinary control failure cannot produce successful empty policy facts.
#[test]
fn ordinary_control_failure_is_sticky_without_stale_fallback() {
    let raw = serde_json::to_vec(&queue_fixture()).unwrap();
    let mut ledger = ContractLedger::default();
    let queue = decode_queue(&raw, &mut ledger, &mut NoopControl).unwrap();
    assert!(matches!(
        prepare_policy(&queue, &[], "2026-10-04T12:00:00Z", &mut ledger, &mut FailedControl),
        Err(ContractError::ControlFailed)
    ));
    assert!(matches!(
        decode_queue(&raw, &mut ledger, &mut NoopControl),
        Err(ContractError::ControlFailed)
    ));
}

/// A real sequential raw-pin withdrawal chain retains every complete original.
#[test]
fn complete_thirty_two_link_chain_keeps_every_original_and_no_vote() {
    let queue = queue_fixture();
    let mut responses = vec![vote(&queue, 1, "reviewer", "review", Disposition::Approve)];
    for number in 2..=33 {
        responses.push(withdrawal(&queue, number, responses.last().unwrap()));
    }
    let result = run(&queue, &responses, "2026-10-04T12:00:00Z").unwrap();
    assert_eq!(result.counts, [1, 33, 33, 0]);
    assert_eq!(result.references, [33]);
    assert_eq!(result.met[0], [] as [(String, u32, String, String); 0]);
    assert!(result.classes.iter().all(|class| *class == ResponseClassification::Superseded));
}

/// Multiple items remain distinct despite one key and shared complete source pins.
#[test]
fn all_selected_items_keep_their_own_response_and_seat_denominators() {
    let mut queue = queue_fixture();
    let mut second: ReviewItem = copied(&queue.items[0]);
    second.key = "item.2".into();
    second.subject_id = "map.2".into();
    second.item_id = validate::item_id(&queue, &second, &mut ContractLedger::default()).unwrap();
    queue.items.push(second);
    let mut response = vote(&queue, 2, "reviewer", "review", Disposition::Approve);
    response.item_key = queue.items[1].key.clone();
    response.item_id = queue.items[1].item_id.clone();
    response.subject_sha256 = queue.items[1].subject_sha256.clone();
    let result = run(
        &queue,
        &[vote(&queue, 1, "reviewer", "review", Disposition::Approve), response],
        "2026-10-04T12:00:00Z",
    )
    .unwrap();
    assert_eq!(result.counts, [2, 2, 2, 0]);
    assert_eq!(result.states, [TentativeState::SeatsFilled; 2]);
    assert_eq!(result.references, [1, 1]);
}

/// A late rejection is preserved as dissent but cannot erase completed-before-due facts.
#[test]
fn late_dissent_is_complete_without_reopening_predeadline_policy() {
    let mut queue = queue_fixture();
    add_reviewer(&mut queue, "z-reviewer");
    let mut late = vote(&queue, 2, "z-reviewer", "review", Disposition::Reject);
    late.responded_at = "2026-10-05T00:00:00Z".into();
    let result = run(
        &queue,
        &[vote(&queue, 1, "reviewer", "review", Disposition::Approve), late],
        "2026-10-06T00:00:00Z",
    )
    .unwrap();
    assert_eq!(result.states, [TentativeState::SeatsFilled]);
    assert_eq!(result.dissent[0], [id(2)]);
    assert!(!result.blocking[0]);
}
