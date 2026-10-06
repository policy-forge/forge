//! Inert /2 policy operands: no native lifecycle owner, record, or currentness.
use super::decode::{ContractError, ContractLedger};
use super::{decode_v2, hash_v2, wire, wire_v2};
use crate::workspace::preparation::{Interruption, ProgressUpdate, Stage, WorkControl, WorkError};
use serde_json::{Value, json};
use wire_v2::*;

/// Actual component controller, with a selectable typed stop at a real callback.
#[derive(Default)]
struct Caller {
    calls: usize,
    stop_at: Option<usize>,
    failed: bool,
    stopped: Option<Interruption>,
}
impl WorkControl for Caller {
    /// Observe production callbacks without introducing a native owner or new timer.
    fn checkpoint(&mut self, _: Stage, _: ProgressUpdate) -> Result<(), WorkError> {
        self.calls += 1;
        if self.stop_at == Some(self.calls) {
            if self.failed {
                return Err(WorkError::Failed(crate::workspace::contract::Error::invalid()));
            }
            self.stopped = Some(Interruption::CancelRequested);
            return Err(WorkError::Interrupted(Interruption::CancelRequested));
        }
        Ok(())
    }
    /// Preserve the original caller's sticky cancellation observation.
    fn interruption(&self) -> Option<Interruption> {
        self.stopped
    }
}
/// Serialize one authored ordinary document as its exact immutable test original.
fn raw(value: &impl serde::Serialize) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(value).unwrap();
    bytes.push(b'\n');
    bytes
}
/// Recompute only inert binary bindings after a declared policy changes.
fn refresh(queue: &mut QueueDocumentV2) {
    let mut ledger = ContractLedger::default();
    let mut caller = Caller::default();
    let value = hash_v2::context(queue, &queue.items[0], &mut ledger, &mut caller).unwrap();
    queue.items[0].context_sha256 = value;
    let value =
        hash_v2::policy(queue, &queue.policies[0], &queue.items[0], &mut ledger, &mut caller)
            .unwrap();
    queue.items[0].policy_sha256 = value;
    let value = hash_v2::item_id(queue, &queue.items[0], &mut ledger, &mut caller).unwrap();
    queue.items[0].item_id = value;
}
/// Complete two-purpose plain declarations with two positive approval seats.
fn queue_fixture() -> QueueDocumentV2 {
    let pins = vec![
        SourcePinV2 {
            artifact_key: "lifecycle:record".into(),
            kind: SourceKindV2::LifecycleRecord,
            raw_sha256: "1".repeat(64),
            byte_length: 512,
            schema_identity: Some("forge.policy-lifecycle/2".into()),
            validation_profile: "forge.lifecycle-record-intrinsic/1".into(),
            native_model: None,
            native_root_uuid: None,
        },
        SourcePinV2 {
            artifact_key: "lifecycle:source".into(),
            kind: SourceKindV2::OpaqueSource,
            raw_sha256: crate::hashing::sha256_hex(b""),
            byte_length: 0,
            schema_identity: None,
            validation_profile: "forge.opaque-source-bytes/1".into(),
            native_model: None,
            native_root_uuid: None,
        },
    ];
    let mut ledger = ContractLedger::default();
    let mut caller = Caller::default();
    let subject_id = hash_v2::subject_id("test policy", "v1", &mut ledger, &mut caller).unwrap();
    let mut queue = QueueDocumentV2 {
        schema_version: "forge.review-queue/2".into(),
        identity_disclaimer: IDENTITY_DISCLAIMER.into(),
        sensitivity: Sensitivity::IdsAndHashes,
        queue_id: "10000000-0000-4000-8000-000000000001".into(),
        created_at: "2026-10-05T00:00:00Z".into(),
        source_pins: pins,
        roles: vec![RoleDefinition { key: "reviewer".into() }],
        reviewers: ["author", "reviewer-a", "reviewer-b", "reviewer-c"]
            .into_iter()
            .map(|key| Reviewer { key: key.into(), role_keys: vec!["reviewer".into()] })
            .collect(),
        policies: vec![ReviewPolicy {
            key: "policy".into(),
            seats: vec![wire::SeatRequirement { role_key: "reviewer".into(), count: 2 }],
            substitutions: vec![],
            abstention_rule: AbstentionRule::Nonapproving,
            empty_abstention_reasons: vec!["absent".into()],
            author_separation: AuthorSeparation::DeclaredKeys,
        }],
        items: vec![ReviewItemV2 {
            key: "item".into(),
            item_id: String::new(),
            domain: DomainV2::LifecyclePolicyVersion,
            adapter_version: ADAPTER.into(),
            subject_id,
            requested_action: RequestedAction::ReReview,
            source_keys: vec!["lifecycle:record".into(), "lifecycle:source".into()],
            subject_sha256: "2".repeat(64),
            context: ContextSnapshot {
                reason_codes: vec!["lifecycle-approved-current".into()],
                related_subject_ids: vec![],
            },
            context_sha256: String::new(),
            policy_key: "policy".into(),
            policy_sha256: String::new(),
            author_keys: vec!["author".into()],
            assignments: ["reviewer-a", "reviewer-b", "reviewer-c"]
                .into_iter()
                .map(|key| Assignment { reviewer_key: key.into(), role_key: "reviewer".into() })
                .collect(),
            due_at: None,
            allowed_dispositions: vec![
                Disposition::Approve,
                Disposition::Reject,
                Disposition::RequestChanges,
                Disposition::Abstain,
                Disposition::Superseded,
            ],
        }],
    };
    refresh(&mut queue);
    queue
}
/// Explicit asserted response bound to observations from the actual strict decoder.
fn response(
    queue: &decode_v2::DecodedV2<'_, QueueDocumentV2>,
    ordinal: u32,
    key: &str,
    disposition: &str,
) -> Value {
    let doc = queue.document();
    let item = &doc.items[0];
    json!({"schema_version":"forge.review-response/2","identity_disclaimer":IDENTITY_DISCLAIMER,
        "response_id":format!("20000000-0000-4000-8000-{ordinal:012}"), "queue_id":doc.queue_id,
        "queue_raw_sha256":queue.raw_sha256(),"item_key":item.key,"item_id":item.item_id,
        "domain":"lifecycle-policy-version","adapter_version":ADAPTER,"requested_action":"re-review",
        "source_pins":doc.source_pins,"subject_sha256":item.subject_sha256,"context_sha256":item.context_sha256,
        "policy_sha256":item.policy_sha256,"reviewer_key":key,"reviewer_role":"reviewer",
        "disposition":disposition,"responded_at":"2026-10-05T00:00:01Z","rationale":"asserted test vote",
        "abstention_reason":if disposition=="abstain" {json!("absent")} else {Value::Null},
        "proposed_edit":null,"supersedes":null})
}
/// Decode every actual original on the same invocation ledger, including duplicates.
fn decode_responses<'a>(
    bytes: &'a [Vec<u8>],
    ledger: &mut ContractLedger,
    caller: &mut Caller,
) -> Vec<decode_v2::DecodedV2<'a, ResponseDocumentV2>> {
    bytes.iter().map(|bytes| decode_v2::decode_response(bytes, ledger, caller).unwrap()).collect()
}

use super::merge_v2::{self, TentativeState};

/// Exact duplicate originals retain the complete file denominator without a second vote.
#[test]
fn distinct_keys_fill_seats_but_duplicate_originals_never_add_votes() {
    let mut ledger = ContractLedger::default();
    let mut caller = Caller::default();
    let queue_bytes = raw(&queue_fixture());
    let queue = decode_v2::decode_queue(&queue_bytes, &mut ledger, &mut caller).unwrap();
    let a = raw(&response(&queue, 1, "reviewer-a", "approve"));
    let b = raw(&response(&queue, 2, "reviewer-b", "approve"));
    let originals = vec![a.clone(), a, b];
    let decoded = decode_responses(&originals, &mut ledger, &mut caller);
    let pending = merge_v2::prepare_policy(
        &queue,
        &decoded,
        "2026-10-05T00:00:04Z",
        &mut ledger,
        &mut caller,
    )
    .unwrap();
    assert_eq!(pending.counts().response_files, 3);
    assert_eq!(pending.counts().unique_responses, 2);
    assert_eq!(pending.counts().exact_duplicates, 1);
    assert_eq!(pending.originals().len(), 3);
    assert_eq!(pending.items()[0].tentative_state, TentativeState::SeatsFilled);
    assert_eq!(pending.items()[0].met_seats.len(), 2);
    assert!(pending.items()[0].unmet_seats.is_empty());
    assert_ne!(
        pending.items()[0].met_seats[0].reviewer_key,
        pending.items()[0].met_seats[1].reviewer_key
    );
    assert!(
        pending.items()[0].reason_codes.iter().any(|reason| reason == "currentness-unverified")
    );
    assert_eq!(pending.queue().raw(), queue_bytes.as_slice());
    assert_eq!(pending.as_of(), "2026-10-05T00:00:04Z");
}

/// A real augmenting path moves a multi-role key so a narrower key fills the other seat.
#[test]
fn maximum_matching_preserves_one_key_per_seat_across_roles() {
    let mut declared = queue_fixture();
    declared.roles =
        vec![RoleDefinition { key: "a-role".into() }, RoleDefinition { key: "b-role".into() }];
    for row in &mut declared.reviewers {
        row.role_keys = vec!["a-role".into(), "b-role".into()];
    }
    declared.policies[0].seats = vec![
        wire::SeatRequirement { role_key: "a-role".into(), count: 1 },
        wire::SeatRequirement { role_key: "b-role".into(), count: 1 },
    ];
    declared.items[0].assignments = vec![
        Assignment { reviewer_key: "reviewer-a".into(), role_key: "a-role".into() },
        Assignment { reviewer_key: "reviewer-a".into(), role_key: "b-role".into() },
        Assignment { reviewer_key: "reviewer-b".into(), role_key: "a-role".into() },
    ];
    refresh(&mut declared);
    let mut ledger = ContractLedger::default();
    let mut caller = Caller::default();
    let bytes = raw(&declared);
    let queue = decode_v2::decode_queue(&bytes, &mut ledger, &mut caller).unwrap();
    let mut a_left = response(&queue, 1, "reviewer-a", "approve");
    a_left["reviewer_role"] = json!("a-role");
    let mut a_right = response(&queue, 2, "reviewer-a", "approve");
    a_right["reviewer_role"] = json!("b-role");
    let mut b_left = response(&queue, 3, "reviewer-b", "approve");
    b_left["reviewer_role"] = json!("a-role");
    let originals = vec![raw(&a_left), raw(&a_right), raw(&b_left)];
    let decoded = decode_responses(&originals, &mut ledger, &mut caller);
    let pending = merge_v2::prepare_policy(
        &queue,
        &decoded,
        "2026-10-05T00:00:04Z",
        &mut ledger,
        &mut caller,
    )
    .unwrap();
    assert_eq!(pending.items()[0].tentative_state, TentativeState::SeatsFilled);
    let seats = &pending.items()[0].met_seats;
    assert_eq!(seats.len(), 2);
    assert_eq!((&*seats[0].role_key, &*seats[0].reviewer_key), ("a-role", "reviewer-b"));
    assert_eq!((&*seats[1].role_key, &*seats[1].reviewer_key), ("b-role", "reviewer-a"));
}

/// Divergent same-key assertions remain visible, including dissent, and cannot approve.
#[test]
fn conflict_keeps_all_dissent_and_removes_the_conflicted_key_from_matching() {
    let mut ledger = ContractLedger::default();
    let mut caller = Caller::default();
    let bytes = raw(&queue_fixture());
    let queue = decode_v2::decode_queue(&bytes, &mut ledger, &mut caller).unwrap();
    let originals = vec![
        raw(&response(&queue, 1, "reviewer-a", "approve")),
        raw(&response(&queue, 2, "reviewer-a", "reject")),
        raw(&response(&queue, 3, "reviewer-b", "approve")),
    ];
    let decoded = decode_responses(&originals, &mut ledger, &mut caller);
    let pending = merge_v2::prepare_policy(
        &queue,
        &decoded,
        "2026-10-05T00:00:04Z",
        &mut ledger,
        &mut caller,
    )
    .unwrap();
    let item = &pending.items()[0];
    assert_eq!(item.tentative_state, TentativeState::Conflicted);
    assert!(item.blocking);
    assert_eq!(item.response_ids.len(), 3);
    assert_eq!(item.dissent_ids, vec!["20000000-0000-4000-8000-000000000002"]);
    assert_eq!(item.met_seats.len(), 1);
    assert_eq!(item.unmet_seats.len(), 1);
    assert!(
        pending
            .responses()
            .iter()
            .filter(|row| row.reviewer_key == "reviewer-a")
            .all(|row| row.classification == ResponseClassification::Conflicted)
    );
}

/// Abstention and asserted authors stay nonapproving; a lone active rejection blocks seats.
#[test]
fn abstention_author_exclusion_and_rejection_are_preserved_nonapproval() {
    for (key, vote, class) in [
        ("reviewer-a", "abstain", ResponseClassification::Current),
        ("author", "approve", ResponseClassification::Unassigned),
        ("reviewer-a", "reject", ResponseClassification::Current),
    ] {
        let mut ledger = ContractLedger::default();
        let mut caller = Caller::default();
        let bytes = raw(&queue_fixture());
        let queue = decode_v2::decode_queue(&bytes, &mut ledger, &mut caller).unwrap();
        let originals = vec![raw(&response(&queue, 1, key, vote))];
        let decoded = decode_responses(&originals, &mut ledger, &mut caller);
        let pending = merge_v2::prepare_policy(
            &queue,
            &decoded,
            "2026-10-05T00:00:04Z",
            &mut ledger,
            &mut caller,
        )
        .unwrap();
        assert_eq!(pending.responses()[0].classification, class);
        assert!(pending.items()[0].met_seats.is_empty());
        assert_eq!(pending.items()[0].unmet_seats.len(), 2);
        assert_ne!(pending.items()[0].tentative_state, TentativeState::SeatsFilled);
        assert_eq!(pending.items()[0].blocking, vote == "reject");
        if vote == "reject" {
            assert_eq!(pending.items()[0].dissent_ids.len(), 1);
        }
    }
}

/// A withdrawal of a withdrawal cannot resurrect the original assertion.
#[test]
fn complete_withdrawal_history_never_reopens_an_earlier_vote() {
    let mut ledger = ContractLedger::default();
    let mut caller = Caller::default();
    let bytes = raw(&queue_fixture());
    let queue = decode_v2::decode_queue(&bytes, &mut ledger, &mut caller).unwrap();
    let prior = response(&queue, 1, "reviewer-a", "approve");
    let prior_bytes = raw(&prior);
    let mut marker = response(&queue, 2, "reviewer-a", "superseded");
    marker["responded_at"] = json!("2026-10-05T00:00:02Z");
    marker["supersedes"] = json!({"response_id":prior["response_id"],"raw_sha256":crate::hashing::sha256_hex(&prior_bytes)});
    let marker_bytes = raw(&marker);
    let mut later = response(&queue, 3, "reviewer-a", "superseded");
    later["responded_at"] = json!("2026-10-05T00:00:03Z");
    later["supersedes"] = json!({"response_id":marker["response_id"],"raw_sha256":crate::hashing::sha256_hex(&marker_bytes)});
    let originals = vec![raw(&later), prior_bytes, marker_bytes];
    let decoded = decode_responses(&originals, &mut ledger, &mut caller);
    let pending = merge_v2::prepare_policy(
        &queue,
        &decoded,
        "2026-10-05T00:00:04Z",
        &mut ledger,
        &mut caller,
    )
    .unwrap();
    assert!(
        pending
            .responses()
            .iter()
            .all(|row| row.classification == ResponseClassification::Superseded)
    );
    assert!(pending.items()[0].met_seats.is_empty());
    assert_eq!(pending.items()[0].response_ids.len(), 3);
    assert!(pending.items()[0].reason_codes.iter().any(|reason| reason == "withdrawn-history"));
}

/// At-due and future markers preserve the earlier timely approval as active evidence.
#[test]
fn late_and_future_withdrawals_cannot_erase_timely_approval() {
    for (at, as_of, class) in [
        ("2026-10-05T00:00:04Z", "2026-10-05T00:00:05Z", ResponseClassification::Late),
        ("2026-10-05T00:00:03Z", "2026-10-05T00:00:02Z", ResponseClassification::Future),
    ] {
        let mut declared = queue_fixture();
        declared.items[0].due_at = Some("2026-10-05T00:00:04Z".into());
        refresh(&mut declared);
        let mut ledger = ContractLedger::default();
        let mut caller = Caller::default();
        let bytes = raw(&declared);
        let queue = decode_v2::decode_queue(&bytes, &mut ledger, &mut caller).unwrap();
        let prior = response(&queue, 1, "reviewer-a", "approve");
        let prior_bytes = raw(&prior);
        let mut marker = response(&queue, 2, "reviewer-a", "superseded");
        marker["responded_at"] = json!(at);
        marker["supersedes"] = json!({"response_id":prior["response_id"],"raw_sha256":crate::hashing::sha256_hex(&prior_bytes)});
        let originals = vec![raw(&marker), prior_bytes];
        let decoded = decode_responses(&originals, &mut ledger, &mut caller);
        let pending =
            merge_v2::prepare_policy(&queue, &decoded, as_of, &mut ledger, &mut caller).unwrap();
        assert_eq!(pending.responses()[0].classification, ResponseClassification::Current);
        assert_eq!(pending.responses()[1].classification, class);
        assert_eq!(pending.items()[0].met_seats.len(), 1);
        assert_eq!(pending.items()[0].unmet_seats.len(), 1);
    }
}

/// Equal UUIDs with changed raw whitespace refuse the whole registry, despite equal JSON.
#[test]
fn response_uuid_raw_identity_conflict_refuses_the_whole_pending_result() {
    let mut ledger = ContractLedger::default();
    let mut caller = Caller::default();
    let bytes = raw(&queue_fixture());
    let queue = decode_v2::decode_queue(&bytes, &mut ledger, &mut caller).unwrap();
    let original = raw(&response(&queue, 1, "reviewer-a", "approve"));
    let mut changed = vec![b' '];
    changed.extend_from_slice(&original);
    let originals = vec![original, changed];
    let decoded = decode_responses(&originals, &mut ledger, &mut caller);
    assert_eq!(
        merge_v2::prepare_policy(
            &queue,
            &decoded,
            "2026-10-05T00:00:04Z",
            &mut ledger,
            &mut caller
        )
        .err(),
        Some(ContractError::IdentityConflict)
    );
    assert_ne!(decoded[0].raw(), decoded[1].raw());
    assert_eq!(decoded.len(), 2);
}

/// A same-length declared source-pin change is binding-stale, never a counted approval.
#[test]
fn complete_pin_snapshot_mismatch_remains_nonapproving_binding_evidence() {
    let mut ledger = ContractLedger::default();
    let mut caller = Caller::default();
    let bytes = raw(&queue_fixture());
    let queue = decode_v2::decode_queue(&bytes, &mut ledger, &mut caller).unwrap();
    let mut vote = response(&queue, 1, "reviewer-a", "approve");
    vote["source_pins"][0]["raw_sha256"] = json!("9".repeat(64));
    let originals = vec![raw(&vote)];
    let decoded = decode_responses(&originals, &mut ledger, &mut caller);
    let pending = merge_v2::prepare_policy(
        &queue,
        &decoded,
        "2026-10-05T00:00:04Z",
        &mut ledger,
        &mut caller,
    )
    .unwrap();
    assert_eq!(pending.responses()[0].classification, ResponseClassification::Stale);
    assert_eq!(pending.items()[0].tentative_state, TentativeState::BindingStale);
    assert!(pending.items()[0].met_seats.is_empty());
    assert_eq!(pending.items()[0].response_ids.len(), 1);
    assert_eq!(pending.items()[0].unmet_seats.len(), 2);
}

/// Both success and ordinary rejection remain subject to the final original callback.
#[test]
fn pending_policy_final_fence_keeps_first_control_failure_and_stops_future_work() {
    for conflict in [false, true] {
        for failed in [false, true] {
            let mut ledger = ContractLedger::default();
            let mut caller = Caller::default();
            let bytes = raw(&queue_fixture());
            let queue = decode_v2::decode_queue(&bytes, &mut ledger, &mut caller).unwrap();
            let original = raw(&response(&queue, 1, "reviewer-a", "approve"));
            let mut changed = vec![b' '];
            changed.extend_from_slice(&original);
            let originals = if conflict { vec![original, changed] } else { vec![original] };
            let decoded = decode_responses(&originals, &mut ledger, &mut caller);
            let begin = caller.calls;
            let first = merge_v2::prepare_policy(
                &queue,
                &decoded,
                "2026-10-05T00:00:04Z",
                &mut ledger,
                &mut caller,
            );
            if conflict {
                assert_eq!(first.err(), Some(ContractError::IdentityConflict));
            } else {
                assert!(first.is_ok());
            }
            let cost = caller.calls - begin;
            caller.stop_at = Some(caller.calls + cost);
            caller.failed = failed;
            let result = merge_v2::prepare_policy(
                &queue,
                &decoded,
                "2026-10-05T00:00:04Z",
                &mut ledger,
                &mut caller,
            )
            .err();
            assert_eq!(
                result,
                Some(if failed {
                    ContractError::ControlFailed
                } else {
                    ContractError::Interrupted(Interruption::CancelRequested)
                })
            );
            let calls = caller.calls;
            assert_eq!(
                merge_v2::prepare_policy(
                    &queue,
                    &decoded,
                    "2026-10-05T00:00:04Z",
                    &mut ledger,
                    &mut caller
                )
                .err(),
                result
            );
            assert_eq!(caller.calls, calls);
            assert_eq!(decoded.len(), if conflict { 2 } else { 1 });
        }
    }
}
