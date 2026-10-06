//! Pure structural controls. Fixtures contain asserted synthetic pins;
//! they are not captured native files, reviewer authority or quorum evidence.

use super::decode::{
    ContractError, ContractLedger, OriginalRelation, decode_dispositions, decode_queue,
    decode_response, original_relation,
};
use super::validate;
use super::wire::{
    AbstentionRule, Assignment, AuthorSeparation, ContextSnapshot, Disposition, DispositionCounts,
    DispositionsDocument, Domain, IDENTITY_DISCLAIMER, ItemDisposition, ItemState, MetSeat,
    QueueDocument, RecordedCurrentness, RecordedResponse, RequestedAction, ResponseClassification,
    ResponseDocument, ReviewItem, ReviewPolicy, Reviewer, RoleDefinition, SeatRequirement,
    Sensitivity, SourceModel, SourcePin, StateCounts, Substitution, SupersessionReference,
    UnmetSeat,
};
use crate::workspace::preparation::{
    Interruption, NoopControl, ProgressUpdate, Stage, WorkControl, WorkError,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

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

/// Assert whole queue refusal; no partial typed receipt is returned.
fn refuse_queue(value: &Value) {
    let raw = serde_json::to_vec(value).unwrap();
    assert!(decode_queue(&raw, &mut ContractLedger::default(), &mut NoopControl).is_err());
}

/// Assert whole private response refusal without inspecting parser prose.
fn refuse_response(value: &Value) {
    let raw = serde_json::to_vec(value).unwrap();
    assert!(decode_response(&raw, &mut ContractLedger::default(), &mut NoopControl).is_err());
}

/// Assert whole bundle refusal without trusting its recorded state label.
fn refuse_bundle(value: &Value) {
    let raw = serde_json::to_vec(value).unwrap();
    assert!(decode_dispositions(&raw, &mut ContractLedger::default(), &mut NoopControl).is_err());
}

/// Caller control that supplies one typed cancellation, never a fake deadline.
struct Cancel;
impl WorkControl for Cancel {
    /// Stop every actual decoder phase with the maintained typed reason.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> Result<(), WorkError> {
        Err(WorkError::Interrupted(Interruption::CancelRequested))
    }
    /// Retain the caller's actual sticky reason.
    fn interruption(&self) -> Option<Interruption> {
        Some(Interruption::CancelRequested)
    }
}

/// Decode all three contracts and exact raw bindings, without a native adapter.
#[test]
fn closed_queue_response_and_bundle_bind_without_native_authority() {
    let queue = queue_fixture();
    let queue_raw = serde_json::to_vec(&queue).unwrap();
    let response_raw = serde_json::to_vec(&response_fixture(&queue, &queue_raw)).unwrap();
    let bundle_raw = serde_json::to_vec(&bundle_fixture(&queue, &queue_raw)).unwrap();
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let queue = decode_queue(&queue_raw, &mut ledger, &mut control).unwrap();
    let response = decode_response(&response_raw, &mut ledger, &mut control).unwrap();
    let bundle = decode_dispositions(&bundle_raw, &mut ledger, &mut control).unwrap();
    validate::bind_response(&queue, &response, &mut ledger, &mut control).unwrap();
    validate::bind_dispositions(&queue, &bundle, &mut ledger, &mut control).unwrap();
    assert_eq!(response.raw(), response_raw);
    assert_eq!(response.raw_sha256(), crate::hashing::sha256_hex(&response_raw));
}

/// Verify exact compact prefixes, complete policy fields and the separate UUID namespace.
#[test]
fn frozen_typed_encoding_vectors_are_consumed() {
    let queue = queue_fixture();
    let item = &queue.items[0];
    assert_eq!(
        item.context_sha256,
        "837e5fe96eb2470c259c2d9380a30846764f811d30325f6f319626b7b5789e0d"
    );
    assert_eq!(
        item.policy_sha256,
        "2c5fb0c20b42efae232100b45dabd57b68f6244ac6c383c60dd0e1fcc6660b46"
    );
    assert_eq!(item.item_id, "db463b91-e63c-5db8-a24b-1faffb4ba0fe");
}

/// Reject nil/case/ambiguous times and retain a genuine leap-day positive.
#[test]
fn explicit_uuid_and_calendar_seconds_are_required() {
    for invalid in [
        "2026-02-29T12:00:00Z",
        "2026-10-04T12:00:60Z",
        "0000-01-01T00:00:00Z",
        "2026-10-04T12:00:00+00:00",
        "2026-10-04T12:00:00.0Z",
    ] {
        assert!(validate::time(invalid).is_err());
    }
    validate::time("2028-02-29T23:59:59Z").unwrap();
    let mut value = serde_json::to_value(queue_fixture()).unwrap();
    value["queue_id"] = json!("00000000-0000-0000-0000-000000000000");
    refuse_queue(&value);
    value["queue_id"] = json!("22222222-2222-4222-8222-22222222222A");
    refuse_queue(&value);
}

/// Exercise actual strict decoded-key admission, closed shapes and original raw guards.
#[test]
fn duplicate_unknown_bom_utf8_and_trailing_inputs_refuse() {
    let raw = serde_json::to_vec(&queue_fixture()).unwrap();
    let mut bom = vec![0xef, 0xbb, 0xbf];
    bom.extend_from_slice(&raw);
    for invalid in [
        bom,
        vec![0xff],
        [raw.as_slice(), b"{}"].concat(),
        b"{\"queue_id\":1,\"queue_id\":2}".to_vec(),
        br#"{"queue_id":1,"queue_\u0069d":2}"#.to_vec(),
    ] {
        assert!(decode_queue(&invalid, &mut ContractLedger::default(), &mut NoopControl).is_err());
    }
    let mut value = serde_json::to_value(queue_fixture()).unwrap();
    value["authenticated"] = json!(true);
    refuse_queue(&value);
    value.as_object_mut().unwrap().remove("authenticated");
    value["items"][0]["unknown"] = json!(0);
    refuse_queue(&value);
}

/// Require every nullable field through the consumed schema, not serde defaults.
#[test]
fn explicit_null_fields_cannot_be_omitted_or_retyped() {
    let queue = queue_fixture();
    let raw = serde_json::to_vec(&queue).unwrap();
    let value = serde_json::to_value(response_fixture(&queue, &raw)).unwrap();
    for field in ["proposed_edit", "supersedes", "abstention_reason"] {
        let mut changed = value.clone();
        changed.as_object_mut().unwrap().remove(field);
        refuse_response(&changed);
    }
    let mut changed = value;
    changed["supersedes"] = json!(false);
    refuse_response(&changed);
}

/// Use closed integer counts and checked positive seat totals.
#[test]
fn zero_fractional_and_overflow_seats_refuse_before_expansion() {
    let value = serde_json::to_value(queue_fixture()).unwrap();
    for count in [json!(0), json!(1.5), json!(101), json!(18_446_744_073_709_551_615_u64)] {
        let mut changed = value.clone();
        changed["policies"][0]["seats"][0]["count"] = count;
        refuse_queue(&changed);
    }
    let mut queue = queue_fixture();
    queue.policies[0].seats[0].count = 100;
    rebind(&mut queue);
    let raw = serde_json::to_vec(&queue).unwrap();
    decode_queue(&raw, &mut ContractLedger::default(), &mut NoopControl).unwrap();
}

/// Reject repeated asserted identities, roles and assignment pairs.
#[test]
fn duplicate_rosters_and_assignments_cannot_create_seats() {
    let value = serde_json::to_value(queue_fixture()).unwrap();
    for field in ["reviewers", "roles", "policies", "source_pins"] {
        let mut changed = value.clone();
        let first = changed[field][0].clone();
        changed[field].as_array_mut().unwrap().push(first);
        refuse_queue(&changed);
    }
    let mut changed = value;
    let first = changed["items"][0]["assignments"][0].clone();
    changed["items"][0]["assignments"].as_array_mut().unwrap().push(first);
    refuse_queue(&changed);
}

/// Exercise both consumed candidate paths without author-key authentication.
#[test]
fn author_keys_cannot_be_assigned_or_substituted() {
    let mut queue = queue_fixture();
    queue.items[0].author_keys = vec!["reviewer".into()];
    rebind(&mut queue);
    refuse_queue(&serde_json::to_value(queue).unwrap());
    let mut queue = queue_fixture();
    queue.items[0].assignments.clear();
    queue.items[0].author_keys = vec!["reviewer".into()];
    queue.policies[0].substitutions.push(Substitution {
        seat_role: "review".into(),
        reviewer_key: "reviewer".into(),
        asserted_role: "review".into(),
        reason_code: "declared-substitute".into(),
    });
    rebind(&mut queue);
    refuse_queue(&serde_json::to_value(queue).unwrap());
}

/// Allow empty assignments while retaining a positive approval denominator.
#[test]
fn unassigned_is_valid_and_never_quorum() {
    let mut queue = queue_fixture();
    queue.items[0].assignments.clear();
    rebind(&mut queue);
    let raw = serde_json::to_vec(&queue).unwrap();
    decode_queue(&raw, &mut ContractLedger::default(), &mut NoopControl).unwrap();
    let bundle = bundle_fixture(&queue, &raw);
    assert_eq!(bundle.items[0].required_seats, 1);
    assert_eq!(bundle.items[0].state, ItemState::Unassigned);
}

/// Accept an explicit substitution edge and refuse undeclared/incompatible roles.
#[test]
fn only_explicit_declared_role_substitution_binds() {
    let mut queue = queue_fixture();
    queue.items[0].assignments.clear();
    queue.policies[0].substitutions.push(Substitution {
        seat_role: "review".into(),
        reviewer_key: "reviewer".into(),
        asserted_role: "review".into(),
        reason_code: "declared-substitute".into(),
    });
    rebind(&mut queue);
    let queue_raw = serde_json::to_vec(&queue).unwrap();
    let response_raw = serde_json::to_vec(&response_fixture(&queue, &queue_raw)).unwrap();
    let mut ledger = ContractLedger::default();
    let queue = decode_queue(&queue_raw, &mut ledger, &mut NoopControl).unwrap();
    let response = decode_response(&response_raw, &mut ledger, &mut NoopControl).unwrap();
    validate::bind_response(&queue, &response, &mut ledger, &mut NoopControl).unwrap();
    let mut value = serde_json::to_value(queue.document()).unwrap();
    value["policies"][0]["substitutions"][0]["asserted_role"] = json!("unknown");
    refuse_queue(&value);
}

/// Check complete declared authors/deadlines/assignments rather than a policy key only.
#[test]
fn full_policy_context_and_item_bindings_change_on_revision() {
    let mut queue = queue_fixture();
    let old_policy = queue.items[0].policy_sha256.clone();
    let old_id = queue.items[0].item_id.clone();
    queue.items[0].due_at = Some("2026-10-06T00:00:00Z".into());
    refuse_queue(&serde_json::to_value(&queue).unwrap());
    rebind(&mut queue);
    assert_ne!(queue.items[0].policy_sha256, old_policy);
    assert_ne!(queue.items[0].item_id, old_id);
    queue.items[0].context.related_subject_ids = vec!["control.2".into()];
    refuse_queue(&serde_json::to_value(&queue).unwrap());
}

/// Reject a queue that silently removes rejection or changes-requested responses.
#[test]
fn dissent_vocabulary_cannot_be_removed() {
    let mut value = serde_json::to_value(queue_fixture()).unwrap();
    value["items"][0]["allowed_dispositions"] = json!(["approve", "abstain", "superseded"]);
    refuse_queue(&value);
}

/// Keep two supported domain revisions and no source/rationale prose in queues.
#[test]
fn unknown_domain_versions_and_sensitive_fields_refuse() {
    let value = serde_json::to_value(queue_fixture()).unwrap();
    for (field, replacement) in [
        ("domain", json!("poam")),
        ("adapter_version", json!("forge.mapping-review/2")),
        ("subject_sha256", json!("A".repeat(64))),
    ] {
        let mut changed = value.clone();
        changed["items"][0][field] = replacement;
        refuse_queue(&changed);
    }
    let mut changed = value;
    changed["items"][0]["context"]["source_excerpt"] = json!("private source");
    refuse_queue(&changed);
}

/// Native pins need UUIDs, companions need null, and declared complete bytes stay bounded.
#[test]
fn source_pin_size_uuid_and_complete_set_correlate() {
    let value = serde_json::to_value(queue_fixture()).unwrap();
    for (field, replacement) in [
        ("byte_length", json!(10_485_761)),
        ("native_root_uuid", Value::Null),
        ("raw_sha256", json!("0".repeat(63))),
    ] {
        let mut changed = value.clone();
        changed["source_pins"][0][field] = replacement;
        refuse_queue(&changed);
    }
    let mut changed = value;
    changed["source_pins"][0]["model"] = json!("mapping-manifest");
    refuse_queue(&changed);
}

/// Reject response binding after semantically equal queue whitespace changes.
#[test]
fn queue_raw_hash_cannot_be_a_semantic_reserialization() {
    let queue = queue_fixture();
    let raw = serde_json::to_vec(&queue).unwrap();
    let response_raw = serde_json::to_vec(&response_fixture(&queue, &raw)).unwrap();
    let mut changed = raw;
    changed.push(b'\n');
    let mut ledger = ContractLedger::default();
    let queue = decode_queue(&changed, &mut ledger, &mut NoopControl).unwrap();
    let response = decode_response(&response_raw, &mut ledger, &mut NoopControl).unwrap();
    assert!(matches!(
        validate::bind_response(&queue, &response, &mut ledger, &mut NoopControl),
        Err(ContractError::Binding)
    ));
}

/// Preserve whitespace-distinct exact originals instead of typed Value equality.
#[test]
fn same_identity_different_original_bytes_is_conflict() {
    let queue = queue_fixture();
    let queue_raw = serde_json::to_vec(&queue).unwrap();
    let raw = serde_json::to_vec(&response_fixture(&queue, &queue_raw)).unwrap();
    let mut changed = raw.clone();
    changed.push(b'\n');
    let mut ledger = ContractLedger::default();
    let first = decode_response(&raw, &mut ledger, &mut NoopControl).unwrap();
    let second = decode_response(&changed, &mut ledger, &mut NoopControl).unwrap();
    assert_eq!(
        original_relation(&first, &second, &mut ledger, &mut NoopControl).unwrap(),
        OriginalRelation::IdentityConflict
    );
    let duplicate = decode_response(&raw, &mut ledger, &mut NoopControl).unwrap();
    assert_eq!(
        original_relation(&first, &duplicate, &mut ledger, &mut NoopControl).unwrap(),
        OriginalRelation::ExactDuplicate
    );
    assert!(first.document() == second.document());
    assert_ne!(first.raw_sha256(), second.raw_sha256());
}

/// Different explicit UUIDs never deduplicate by content or reviewer alone.
#[test]
fn different_response_uuids_remain_independent() {
    let queue = queue_fixture();
    let queue_raw = serde_json::to_vec(&queue).unwrap();
    let response = response_fixture(&queue, &queue_raw);
    let raw = serde_json::to_vec(&response).unwrap();
    let mut changed = copied(&response);
    changed.response_id = "44444444-4444-4444-8444-444444444444".into();
    let changed = serde_json::to_vec(&changed).unwrap();
    let mut ledger = ContractLedger::default();
    let first = decode_response(&raw, &mut ledger, &mut NoopControl).unwrap();
    let second = decode_response(&changed, &mut ledger, &mut NoopControl).unwrap();
    assert_eq!(
        original_relation(&first, &second, &mut ledger, &mut NoopControl).unwrap(),
        OriginalRelation::DifferentIdentity
    );
}

/// Exercise genuine eight-KiB boundaries and nonempty non-abstaining rationale.
#[test]
fn rationale_is_private_required_and_utf8_byte_bounded() {
    let queue = queue_fixture();
    let raw = serde_json::to_vec(&queue).unwrap();
    let mut response = response_fixture(&queue, &raw);
    for accepted in ["x".repeat(8192), "é".repeat(4096)] {
        response.rationale = accepted;
        let raw = serde_json::to_vec(&response).unwrap();
        decode_response(&raw, &mut ContractLedger::default(), &mut NoopControl).unwrap();
    }
    for refused in [String::new(), "  ".into(), "x".repeat(8193), "é".repeat(4097)] {
        response.rationale = refused;
        refuse_response(&serde_json::to_value(&response).unwrap());
    }
}

/// Abstention has no seat effect and only the selected explicit exception permits empty rationale.
#[test]
fn empty_abstention_requires_documented_policy_reason() {
    let queue = queue_fixture();
    let raw = serde_json::to_vec(&queue).unwrap();
    let mut response = response_fixture(&queue, &raw);
    response.disposition = Disposition::Abstain;
    response.rationale.clear();
    response.abstention_reason = Some("no-conflict".into());
    let response_raw = serde_json::to_vec(&response).unwrap();
    let mut ledger = ContractLedger::default();
    let decoded_queue = decode_queue(&raw, &mut ledger, &mut NoopControl).unwrap();
    let decoded_response = decode_response(&response_raw, &mut ledger, &mut NoopControl).unwrap();
    validate::bind_response(&decoded_queue, &decoded_response, &mut ledger, &mut NoopControl)
        .unwrap();
    response.abstention_reason = Some("undeclared".into());
    let response_raw = serde_json::to_vec(&response).unwrap();
    let decoded_response = decode_response(&response_raw, &mut ledger, &mut NoopControl).unwrap();
    assert!(
        validate::bind_response(&decoded_queue, &decoded_response, &mut ledger, &mut NoopControl)
            .is_err()
    );
}

/// Require explicit nonself references; chain/cross-reviewer proof remains merge work.
#[test]
fn supersession_binds_exact_prior_id_and_hash_without_retiring_it() {
    let queue = queue_fixture();
    let raw = serde_json::to_vec(&queue).unwrap();
    let mut response = response_fixture(&queue, &raw);
    response.disposition = Disposition::Superseded;
    response.supersedes = Some(SupersessionReference {
        response_id: "44444444-4444-4444-8444-444444444444".into(),
        raw_sha256: "2".repeat(64),
    });
    let raw = serde_json::to_vec(&response).unwrap();
    decode_response(&raw, &mut ContractLedger::default(), &mut NoopControl).unwrap();
    response.supersedes.as_mut().unwrap().response_id = response.response_id.clone();
    refuse_response(&serde_json::to_value(&response).unwrap());
    response.supersedes = None;
    refuse_response(&serde_json::to_value(&response).unwrap());
}

/// Non-null edit syntax refuses explicitly, rather than accepting a generic patch.
#[test]
fn unsupported_edit_never_enters_a_clean_typed_response() {
    let queue = queue_fixture();
    let raw = serde_json::to_vec(&queue).unwrap();
    let mut value = serde_json::to_value(response_fixture(&queue, &raw)).unwrap();
    value["proposed_edit"] =
        json!({"adapter_version":"forge.mapping-edit/1","target_sha256":"0".repeat(64)});
    let raw = serde_json::to_vec(&value).unwrap();
    assert!(matches!(
        decode_response(&raw, &mut ContractLedger::default(), &mut NoopControl),
        Err(ContractError::UnsupportedEdit)
    ));
}

/// Equality at one MiB and aggregate 32 MiB is admitted; plus one refuses without reset.
#[test]
fn complete_raw_response_cap_and_shared_original_pool_are_consumed() {
    let queue = queue_fixture();
    let queue_raw = serde_json::to_vec(&queue).unwrap();
    let mut raw = serde_json::to_vec(&response_fixture(&queue, &queue_raw)).unwrap();
    raw.resize(1_048_576, b' ');
    let mut ledger = ContractLedger::default();
    for _ in 0..32 {
        decode_response(&raw, &mut ledger, &mut NoopControl).unwrap();
    }
    assert!(matches!(
        decode_response(&raw, &mut ledger, &mut NoopControl),
        Err(ContractError::Capacity)
    ));
    assert!(matches!(ledger.bytes(0), Err(ContractError::Capacity)));
    raw.push(b' ');
    assert!(decode_response(&raw, &mut ContractLedger::default(), &mut NoopControl).is_err());
}

/// Exhaust logical capacities before new work, including checked overflow.
#[test]
fn shared_checked_work_and_derived_limits_are_sticky() {
    let mut ledger = ContractLedger::default();
    ledger.bytes(268_435_456).unwrap();
    assert!(ledger.bytes(1).is_err());
    assert!(ledger.visits(0).is_err());
    let mut ledger = ContractLedger::default();
    ledger.derived(33_554_432).unwrap();
    assert!(ledger.derived(1).is_err());
    let mut ledger = ContractLedger::default();
    ledger.visits(1_000_000).unwrap();
    assert!(ledger.visits(1).is_err());
    let mut ledger = ContractLedger::default();
    assert!(ledger.bytes(usize::MAX).is_err());
    let mut ledger = ContractLedger::default();
    ledger.graph(100_000, 200_000, 400_000).unwrap();
    assert!(ledger.graph(1, 0, 0).is_err());
}

/// Preserve the maintained typed cancellation and refuse all later phases.
#[test]
fn actual_caller_interruption_never_becomes_invalid_or_empty() {
    let raw = serde_json::to_vec(&queue_fixture()).unwrap();
    let mut ledger = ContractLedger::default();
    assert!(matches!(
        decode_queue(&raw, &mut ledger, &mut Cancel),
        Err(ContractError::Interrupted(Interruption::CancelRequested))
    ));
    assert!(matches!(
        decode_queue(&raw, &mut ledger, &mut NoopControl),
        Err(ContractError::Interrupted(Interruption::CancelRequested))
    ));
}

/// A declared closure digest is not a fresh proof; inconsistent pairings refuse.
#[test]
fn recorded_currentness_is_inert_and_has_closed_pairing() {
    let queue = queue_fixture();
    let raw = serde_json::to_vec(&queue).unwrap();
    let mut bundle = bundle_fixture(&queue, &raw);
    bundle.currentness = RecordedCurrentness::RecordedCurrent;
    bundle.closure_generation = Some("3".repeat(64));
    let raw = serde_json::to_vec(&bundle).unwrap();
    let decoded =
        decode_dispositions(&raw, &mut ContractLedger::default(), &mut NoopControl).unwrap();
    assert_eq!(decoded.document().currentness, RecordedCurrentness::RecordedCurrent);
    bundle.closure_generation = None;
    refuse_bundle(&serde_json::to_value(bundle).unwrap());
}

/// Reject forged total counts and erased approval denominators.
#[test]
fn all_recorded_counters_and_unmet_seats_must_be_complete() {
    let queue = queue_fixture();
    let raw = serde_json::to_vec(&queue).unwrap();
    let value = serde_json::to_value(bundle_fixture(&queue, &raw)).unwrap();
    for (field, replacement) in
        [("items", json!(2)), ("unique_responses", json!(1)), ("response_files", json!(1))]
    {
        let mut changed = value.clone();
        changed["counts"][field] = replacement;
        refuse_bundle(&changed);
    }
    let mut changed = value.clone();
    changed["items"][0]["required_seats"] = json!(0);
    refuse_bundle(&changed);
    let mut changed = value;
    changed["items"][0]["unmet_seats"] = json!([]);
    refuse_bundle(&changed);
}

/// A historical rejection remains in public minimized evidence, with no rationale.
#[test]
fn every_dissent_record_must_remain_referenced() {
    let queue = queue_fixture();
    let raw = serde_json::to_vec(&queue).unwrap();
    let mut bundle = bundle_fixture(&queue, &raw);
    let id = "33333333-3333-4333-8333-333333333333".to_string();
    bundle.responses.push(RecordedResponse {
        response_id: id.clone(),
        raw_sha256: "4".repeat(64),
        byte_length: 200,
        item_key: "item".into(),
        reviewer_key: "reviewer".into(),
        reviewer_role: "review".into(),
        disposition: Disposition::Reject,
        responded_at: "2026-10-04T12:00:00Z".into(),
        classification: ResponseClassification::Superseded,
    });
    bundle.items[0].response_ids.push(id.clone());
    bundle.items[0].dissent_ids.push(id);
    bundle.counts.response_files = 1;
    bundle.counts.unique_responses = 1;
    let raw = serde_json::to_vec(&bundle).unwrap();
    decode_dispositions(&raw, &mut ContractLedger::default(), &mut NoopControl).unwrap();
    bundle.items[0].dissent_ids.clear();
    refuse_bundle(&serde_json::to_value(bundle).unwrap());
}

/// Reject nonapproval witnesses and the same asserted key filling two roles.
#[test]
fn one_key_abstention_and_rejection_never_fabricate_seat_witnesses() {
    let queue = queue_fixture();
    let raw = serde_json::to_vec(&queue).unwrap();
    let mut bundle = bundle_fixture(&queue, &raw);
    let id = "33333333-3333-4333-8333-333333333333".to_string();
    bundle.responses.push(RecordedResponse {
        response_id: id.clone(),
        raw_sha256: "4".repeat(64),
        byte_length: 200,
        item_key: "item".into(),
        reviewer_key: "reviewer".into(),
        reviewer_role: "review".into(),
        disposition: Disposition::Abstain,
        responded_at: "2026-10-04T12:00:00Z".into(),
        classification: ResponseClassification::Current,
    });
    bundle.items[0].response_ids.push(id.clone());
    bundle.items[0].unmet_seats.clear();
    bundle.items[0].met_seats.push(MetSeat {
        role_key: "review".into(),
        ordinal: 0,
        reviewer_key: "reviewer".into(),
        response_id: id.clone(),
    });
    bundle.counts.response_files = 1;
    bundle.counts.unique_responses = 1;
    refuse_bundle(&serde_json::to_value(&bundle).unwrap());
    bundle.responses[0].disposition = Disposition::Approve;
    bundle.items[0].required_seats = 2;
    bundle.items[0].met_seats.push(MetSeat {
        role_key: "second".into(),
        ordinal: 0,
        reviewer_key: "reviewer".into(),
        response_id: id,
    });
    refuse_bundle(&serde_json::to_value(bundle).unwrap());
}

/// Bind complete bundle rows to the original queue, without promoting recorded quorum.
#[test]
fn foreign_queue_hash_and_recorded_policy_seats_never_bind() {
    let queue = queue_fixture();
    let raw = serde_json::to_vec(&queue).unwrap();
    let mut bundle = bundle_fixture(&queue, &raw);
    bundle.items[0].unmet_seats[0].ordinal = 1;
    let bundle_raw = serde_json::to_vec(&bundle).unwrap();
    let mut ledger = ContractLedger::default();
    let queue = decode_queue(&raw, &mut ledger, &mut NoopControl).unwrap();
    let bundle = decode_dispositions(&bundle_raw, &mut ledger, &mut NoopControl).unwrap();
    assert!(matches!(
        validate::bind_dispositions(&queue, &bundle, &mut ledger, &mut NoopControl),
        Err(ContractError::Binding)
    ));
}

/// Admit complete maximum asserted reviewer-role membership, not a deduplicated prefix.
#[test]
fn complete_roster_membership_boundary_is_consumed() {
    let mut queue = queue_fixture();
    queue.roles = (0..32).map(|n| RoleDefinition { key: format!("role{n:02}") }).collect();
    queue.reviewers = (0..100)
        .map(|n| Reviewer {
            key: format!("reviewer{n:03}"),
            role_keys: (0..32).map(|role| format!("role{role:02}")).collect(),
        })
        .collect();
    queue.policies[0].seats[0].role_key = "role00".into();
    queue.items[0].assignments =
        vec![Assignment { reviewer_key: "reviewer000".into(), role_key: "role00".into() }];
    rebind(&mut queue);
    let raw = serde_json::to_vec(&queue).unwrap();
    decode_queue(&raw, &mut ContractLedger::default(), &mut NoopControl).unwrap();
    let mut value = serde_json::to_value(&queue).unwrap();
    value["reviewers"]
        .as_array_mut()
        .unwrap()
        .push(json!({"key":"reviewer100","role_keys":["role00"]}));
    refuse_queue(&value);
}

/// Sum finite role counts before constructing any expanded seats.
#[test]
fn summed_item_seat_boundary_refuses_the_complete_queue() {
    let mut queue = queue_fixture();
    queue.roles.push(RoleDefinition { key: "second".into() });
    queue.reviewers[0].role_keys.push("second".into());
    queue.policies[0].seats[0].count = 50;
    queue.policies[0].seats.push(SeatRequirement { role_key: "second".into(), count: 51 });
    rebind(&mut queue);
    let raw = serde_json::to_vec(&queue).unwrap();
    assert!(matches!(
        decode_queue(&raw, &mut ContractLedger::default(), &mut NoopControl),
        Err(ContractError::Capacity)
    ));
}

/// Complete declared source bytes stay bounded independently of the selected subset.
#[test]
fn complete_declared_source_byte_total_is_consumed() {
    let mut value = serde_json::to_value(queue_fixture()).unwrap();
    let original = value["source_pins"][0].clone();
    let mut rows = Vec::new();
    for index in 0..6 {
        let mut row = original.clone();
        row["artifact_key"] = json!(format!("mapping{index}"));
        row["byte_length"] = json!(10_485_760);
        rows.push(row);
    }
    value["source_pins"] = Value::Array(rows);
    value["items"][0]["source_keys"] = json!(["mapping0"]);
    let raw = serde_json::to_vec(&value).unwrap();
    assert!(matches!(
        decode_queue(&raw, &mut ContractLedger::default(), &mut NoopControl),
        Err(ContractError::Capacity)
    ));
}

/// Actual context equality at two KiB is admitted; plus one refuses before binding.
#[test]
fn actual_minimized_context_encoded_boundary_is_consumed() {
    let mut queue = queue_fixture();
    queue.items[0].context = ContextSnapshot {
        reason_codes: (0..32).map(|n| format!("r{n:02}{}", "a".repeat(55))).collect(),
        related_subject_ids: vec![],
    };
    queue.items[0].context.reason_codes[31].push_str(&"a".repeat(53));
    assert_eq!(serde_json::to_vec(&queue.items[0].context).unwrap().len(), 2_048);
    rebind(&mut queue);
    let raw = serde_json::to_vec(&queue).unwrap();
    decode_queue(&raw, &mut ContractLedger::default(), &mut NoopControl).unwrap();
    queue.items[0].context.reason_codes[31].push('a');
    assert!(matches!(
        validate::context_hash(&queue, &queue.items[0], &mut ContractLedger::default()),
        Err(ContractError::Capacity)
    ));
}

/// Live dissent cannot be hidden behind a recorded clean quorum and cleared blocker flag.
#[test]
fn live_dissent_cannot_hide_behind_recorded_quorum() {
    let queue = queue_fixture();
    let raw = serde_json::to_vec(&queue).unwrap();
    let mut bundle = bundle_fixture(&queue, &raw);
    let approve_id = "33333333-3333-4333-8333-333333333333".to_string();
    let reject_id = "44444444-4444-4444-8444-444444444444".to_string();
    for (id, disposition) in
        [(approve_id.clone(), Disposition::Approve), (reject_id.clone(), Disposition::Reject)]
    {
        bundle.responses.push(RecordedResponse {
            response_id: id.clone(),
            raw_sha256: "4".repeat(64),
            byte_length: 200,
            item_key: "item".into(),
            reviewer_key: "reviewer".into(),
            reviewer_role: "review".into(),
            disposition,
            responded_at: "2026-10-04T12:00:00Z".into(),
            classification: ResponseClassification::Current,
        });
        bundle.items[0].response_ids.push(id);
    }
    bundle.items[0].dissent_ids.push(reject_id);
    bundle.items[0].state = ItemState::QuorumMet;
    bundle.items[0].unmet_seats.clear();
    bundle.items[0].met_seats.push(MetSeat {
        role_key: "review".into(),
        ordinal: 0,
        reviewer_key: "reviewer".into(),
        response_id: approve_id,
    });
    bundle.counts.response_files = 2;
    bundle.counts.unique_responses = 2;
    bundle.counts.states.unassigned = 0;
    bundle.counts.states.quorum_met = 1;
    refuse_bundle(&serde_json::to_value(bundle).unwrap());
}

/// An external structural capacity refusal latches across a later valid raw decode.
#[test]
fn complete_source_capacity_remains_sticky_for_the_command() {
    let valid = serde_json::to_vec(&queue_fixture()).unwrap();
    let mut oversized = serde_json::to_value(queue_fixture()).unwrap();
    let original = oversized["source_pins"][0].clone();
    let mut rows = Vec::new();
    for n in 0..6 {
        let mut row = original.clone();
        row["artifact_key"] = json!(format!("mapping{n}"));
        row["byte_length"] = json!(10_485_760);
        rows.push(row);
    }
    oversized["source_pins"] = Value::Array(rows);
    oversized["items"][0]["source_keys"] = json!(["mapping0"]);
    let raw = serde_json::to_vec(&oversized).unwrap();
    let mut ledger = ContractLedger::default();
    assert!(matches!(
        decode_queue(&raw, &mut ledger, &mut NoopControl),
        Err(ContractError::Capacity)
    ));
    assert!(matches!(
        decode_queue(&valid, &mut ledger, &mut NoopControl),
        Err(ContractError::Capacity)
    ));
}

/// Ordinary malformed input is a refusal but does not invent a sticky control stop.
#[test]
fn ordinary_invalid_input_does_not_latch_as_capacity_or_control() {
    let valid = serde_json::to_vec(&queue_fixture()).unwrap();
    let mut ledger = ContractLedger::default();
    assert!(matches!(
        decode_queue(b"{}", &mut ledger, &mut NoopControl),
        Err(ContractError::Invalid)
    ));
    decode_queue(&valid, &mut ledger, &mut NoopControl).unwrap();
}
