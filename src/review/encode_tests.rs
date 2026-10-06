//! Writer controls over actual closed decoders and inert synthetic declarations.
//! These fixtures grant no native currentness, authenticated reviewer or domain approval.
use super::{dispositions, encoded, queue, response};
use crate::review::decode::{
    ContractError, ContractLedger, decode_dispositions, decode_queue, decode_response,
};
use crate::review::validate;
use crate::review::wire::{
    AbstentionRule, Assignment, AuthorSeparation, ContextSnapshot, Disposition, DispositionCounts,
    DispositionsDocument, Domain, IDENTITY_DISCLAIMER, ItemDisposition, ItemState, QueueDocument,
    RecordedCurrentness, RequestedAction, ResponseDocument, ReviewItem, ReviewPolicy, Reviewer,
    RoleDefinition, SeatRequirement, Sensitivity, SourceModel, SourcePin, StateCounts, UnmetSeat,
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

/// Complete inert unassigned bundle, with an explicit unverified closure label.
fn bundle_fixture(queue: &QueueDocument, raw: &[u8]) -> DispositionsDocument {
    DispositionsDocument {
        schema_version: "forge.review-dispositions/1".into(),
        identity_disclaimer: IDENTITY_DISCLAIMER.into(),
        queue_id: queue.queue_id.clone(),
        queue_raw_sha256: crate::hashing::sha256_hex(raw),
        as_of: "2026-10-04T12:00:00Z".into(),
        currentness: RecordedCurrentness::Unverified,
        closure_generation: None,
        source_pins: copied(&queue.source_pins),
        responses: vec![],
        items: vec![ItemDisposition {
            item_key: queue.items[0].key.clone(),
            item_id: queue.items[0].item_id.clone(),
            state: ItemState::Unassigned,
            reason_codes: vec!["unassigned".into()],
            required_seats: 1,
            met_seats: vec![],
            unmet_seats: vec![UnmetSeat { role_key: "review".into(), ordinal: 0 }],
            response_ids: vec![],
            dissent_ids: vec![],
            blocking: false,
        }],
        counts: DispositionCounts {
            items: 1,
            response_files: 0,
            unique_responses: 0,
            exact_duplicates: 0,
            states: StateCounts {
                unassigned: 1,
                assigned: 0,
                in_review: 0,
                conflicted: 0,
                changes_requested: 0,
                quorum_met: 0,
                expired: 0,
                stale: 0,
            },
        },
    }
}

/// A complete closed queue roundtrip keeps all declarations and explicit LF deterministically.
#[test]
fn closed_queue_json_preserves_every_field_and_lf() {
    let original = serde_json::to_vec(&queue_fixture()).unwrap();
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let decoded = decode_queue(&original, &mut ledger, &mut control).unwrap();
    let first = queue(decoded.document(), &mut ledger, &mut control).unwrap();
    let second = queue(decoded.document(), &mut ledger, &mut control).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.last(), Some(&b'\n'));
    let rebound = decode_queue(&first, &mut ledger, &mut control).unwrap();
    assert!(rebound.document() == decoded.document());
    assert_ne!(rebound.raw_sha256(), decoded.raw_sha256());
}

/// Escaped private rationale survives closed response encoding without changing its queue pin.
#[test]
fn closed_response_keeps_original_queue_binding_and_escaped_rationale() {
    let queue_doc = queue_fixture();
    let queue_raw = serde_json::to_vec(&queue_doc).unwrap();
    let mut value = response_fixture(&queue_doc, &queue_raw);
    value.rationale = "Quotes \" slash \\ newline\n Unicode é <script>".into();
    let original = serde_json::to_vec(&value).unwrap();
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let decoded_queue = decode_queue(&queue_raw, &mut ledger, &mut control).unwrap();
    let decoded = decode_response(&original, &mut ledger, &mut control).unwrap();
    let output = response(decoded.document(), &mut ledger, &mut control).unwrap();
    let rebound = decode_response(&output, &mut ledger, &mut control).unwrap();
    validate::bind_response(&decoded_queue, &rebound, &mut ledger, &mut control).unwrap();
    assert!(rebound.document() == decoded.document());
    assert_eq!(rebound.document().queue_raw_sha256, crate::hashing::sha256_hex(&queue_raw));
}

/// Recorded counts/currentness are conserved without fabricating a current native owner.
#[test]
fn closed_dispositions_keep_unverified_record_and_complete_denominators() {
    let queue = queue_fixture();
    let queue_raw = serde_json::to_vec(&queue).unwrap();
    let original = serde_json::to_vec(&bundle_fixture(&queue, &queue_raw)).unwrap();
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let decoded = decode_dispositions(&original, &mut ledger, &mut control).unwrap();
    let output = dispositions(decoded.document(), &mut ledger, &mut control).unwrap();
    let rebound = decode_dispositions(&output, &mut ledger, &mut control).unwrap();
    assert!(rebound.document() == decoded.document());
    assert_eq!(rebound.document().currentness, RecordedCurrentness::Unverified);
    assert_eq!(rebound.document().counts.items, 1);
    assert_eq!(rebound.document().counts.response_files, 0);
}

/// The true encoded ceiling includes escaping, UTF-8 and the trailing LF.
#[test]
fn encoded_capacity_is_measured_after_escaping_and_includes_lf() {
    let value = "é\"\\\n";
    let expected = "\"é\\\"\\\\\\n\"\n".as_bytes();
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let exact = encoded(&value, expected.len(), &mut ledger, &mut control).unwrap();
    assert_eq!(exact, expected);
    let refused =
        encoded(&value, expected.len() - 1, &mut ContractLedger::default(), &mut NoopControl);
    assert_eq!(refused.err(), Some(ContractError::Capacity));
}

/// Encoding shares prior retained storage and preserves the first sticky capacity failure.
#[test]
fn prior_command_storage_cannot_be_reset_by_encoder() {
    let mut ledger = ContractLedger::default();
    ledger.derived(33_554_431).unwrap();
    let first = encoded(&"x", 100, &mut ledger, &mut NoopControl);
    assert_eq!(first.err(), Some(ContractError::Capacity));
    let second = encoded(&"", 100, &mut ledger, &mut NoopControl);
    assert_eq!(second.err(), Some(ContractError::Capacity));
}

/// Actual writer checkpoint control records reached work before its sticky typed cancellation.
struct StopAfter {
    /// Actual observed checkpoint count, not fabricated native execution.
    calls: usize,
}
impl WorkControl for StopAfter {
    /// Stop only after the serializer has reached multiple real writer fences.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> Result<(), WorkError> {
        self.calls += 1;
        if self.calls >= 4 {
            Err(WorkError::Interrupted(Interruption::CancelRequested))
        } else {
            Ok(())
        }
    }
    /// Preserve the exact latched caller reason after the first actual stop.
    fn interruption(&self) -> Option<Interruption> {
        (self.calls >= 4).then_some(Interruption::CancelRequested)
    }
}

/// A reached writer stop returns no partial Vec and later encoding retains that exact reason.
#[test]
fn reached_writer_control_stop_is_exact_and_sticky() {
    let mut control = StopAfter { calls: 0 };
    let mut ledger = ContractLedger::default();
    let result = encoded(&["one", "two", "three"], 100, &mut ledger, &mut control);
    assert_eq!(result.err(), Some(ContractError::Interrupted(Interruption::CancelRequested)));
    assert!(control.calls >= 4);
    let later = encoded(&"x", 100, &mut ledger, &mut NoopControl);
    assert_eq!(later.err(), Some(ContractError::Interrupted(Interruption::CancelRequested)));
}

/// An invalid typed declaration cannot bypass the closed semantic validator on output.
#[test]
fn output_rejects_corrupted_item_binding_before_returning_bytes() {
    let mut value = queue_fixture();
    value.items[0].subject_sha256 = "f".repeat(64);
    let result = queue(&value, &mut ContractLedger::default(), &mut NoopControl);
    assert_eq!(result.err(), Some(ContractError::Binding));
}
