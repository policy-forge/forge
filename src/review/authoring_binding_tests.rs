// Genuine captured Authoring Queue/3 binding, typed policy and current-owner controls.
// Root alone executes this leaf; the parent 21 receiver controls remain exact.
use super::*;
use crate::review::wire_v3::*;
use crate::review::{
    authoring_binding as native_binding, authoring_finalize,
    authoring_response_binding as ordinary_binding, decode_v3, encode_v3, hash_v3, merge_v3,
};
use std::rc::Rc;

/// Canonical review UUID; this is never the native project key.
fn review_id(n: u8) -> String {
    format!("{n:08x}-1111-4111-8111-111111111111")
}
/// Inspect an ordinary error without Debug on private owners.
fn refusal<T>(value: Result<T, ContractError>) -> ContractError {
    value.err().expect("expected refusal")
}
/// Test-only full ordinary typed copy, never a native owner constructor.
fn data_copy<T: serde::de::DeserializeOwned, S: serde::Serialize + ?Sized>(value: &S) -> T {
    serde_json::from_value(serde_json::to_value(value).unwrap()).unwrap()
}
/// Reframe ordinary policy/context/item fields after an explicit fixture declaration change.
fn reframe(queue: &mut QueueDocumentV3) {
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let item = &mut queue.items[0];
    item.source_keys = queue.source_pins.iter().map(|p| p.artifact_key.clone()).collect();
    item.context_sha256 =
        hash_v3::context(&queue.source_pins, &item.context, &mut ledger, &mut control).unwrap();
    item.policy_sha256 =
        hash_v3::policy(&queue.source_pins, &queue.policies[0], item, &mut ledger, &mut control)
            .unwrap();
    item.item_id =
        hash_v3::item_id(&queue.queue_id, item, &queue.source_pins, &mut ledger, &mut control)
            .unwrap();
}
/// Derive the ordinary Queue from a genuinely prepared Init owner before the tested operation.
/// Fixture setup has its own real operation; its owner is dropped and cannot authorize Merge.
fn native_queue(owner: &CurrentAuthoringPlanClosure) -> QueueDocumentV3 {
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let pins = data_copy(owner.source_pins());
    let provenance: Vec<_> = owner.native_provenance().collect();
    let digest = hash_v3::native_provenance(&provenance, &mut ledger, &mut control).unwrap();
    let project = &owner.facts().loaded.project;
    let stored = owner.source_pins().iter().find(|p| p.kind == SourceKindV3::StoredPlan).unwrap();
    let subject = hash_v3::subject(
        &project.project_key,
        &project.as_of,
        &digest,
        &stored.raw_sha256,
        owner.source_pins(),
        &mut ledger,
        &mut control,
    )
    .unwrap();
    let identity = hash_v3::subject_id(&project.project_key, &mut ledger, &mut control).unwrap();
    let mut queue = QueueDocumentV3 {
        schema_version: "forge.review-queue/3".into(),
        identity_disclaimer: IDENTITY_DISCLAIMER.into(),
        sensitivity: Sensitivity::IdsAndHashes,
        queue_id: review_id(50),
        created_at: "2026-09-08T02:00:00Z".into(),
        source_pins: pins,
        roles: vec![
            RoleDefinition { key: "approver".into() },
            RoleDefinition { key: "owner".into() },
        ],
        reviewers: vec![
            Reviewer { key: "alice".into(), role_keys: vec!["approver".into(), "owner".into()] },
            Reviewer { key: "author".into(), role_keys: vec!["approver".into()] },
            Reviewer { key: "bob".into(), role_keys: vec!["approver".into(), "owner".into()] },
            Reviewer { key: "carol".into(), role_keys: vec!["approver".into()] },
        ],
        policies: vec![ReviewPolicy {
            key: "review".into(),
            seats: vec![SeatRequirement { role_key: "approver".into(), count: 1 }],
            substitutions: vec![],
            abstention_rule: AbstentionRule::Nonapproving,
            empty_abstention_reasons: vec!["not-qualified".into()],
            author_separation: AuthorSeparation::DeclaredKeys,
        }],
        items: vec![ReviewItemV3 {
            key: "whole-plan".into(),
            item_id: String::new(),
            domain: DomainV3::AuthoringPlan,
            adapter_version: ADAPTER.into(),
            subject_id: identity,
            requested_action: RequestedAction::ReReview,
            source_keys: vec![],
            subject_sha256: subject,
            context: ContextSnapshot {
                reason_codes: vec!["authoring-plan-current".into()],
                related_subject_ids: vec![],
            },
            context_sha256: String::new(),
            policy_key: "review".into(),
            policy_sha256: String::new(),
            author_keys: vec!["author".into()],
            assignments: vec![
                Assignment { reviewer_key: "alice".into(), role_key: "approver".into() },
                Assignment { reviewer_key: "bob".into(), role_key: "approver".into() },
                Assignment { reviewer_key: "carol".into(), role_key: "approver".into() },
            ],
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
    reframe(&mut queue);
    queue
}
/// Save a full finite ordinary Queue before starting the tested complete capture operation.
fn save_queue(f: &Fixture, queue: &QueueDocumentV3) -> Vec<u8> {
    let raw = encode_v3::queue(queue, &mut ContractLedger::default(), &mut NoopControl).unwrap();
    std::fs::write(f.root.join("queue.json"), &raw).unwrap();
    raw
}
/// Prepare genuine native fixture and persist the ordinary Queue independently of test authority.
fn fixture(profile: bool, mapping: bool) -> (Fixture, QueueDocumentV3, Vec<u8>) {
    let f = Fixture::new(profile, mapping);
    originals(&f);
    let queue = {
        let mut ledger = ContractLedger::default();
        let mut caller = NoopControl;
        let mut control = ReviewControl::accept(&mut caller);
        let owner = init(&f, &mut ledger, &mut control).unwrap();
        native_queue(&owner)
    };
    let raw = save_queue(&f, &queue);
    (f, queue, raw)
}
/// Complete ordinary response fields copy the actual original Queue snapshot, not native proof.
fn response(
    queue: &QueueDocumentV3,
    raw: &[u8],
    n: u8,
    key: &str,
    disposition: Disposition,
) -> ResponseDocumentV3 {
    let item = &queue.items[0];
    ResponseDocumentV3 {
        schema_version: "forge.review-response/3".into(),
        identity_disclaimer: IDENTITY_DISCLAIMER.into(),
        response_id: review_id(n),
        queue_id: queue.queue_id.clone(),
        queue_raw_sha256: sha256_hex(raw),
        item_key: item.key.clone(),
        item_id: item.item_id.clone(),
        domain: item.domain,
        adapter_version: item.adapter_version.clone(),
        requested_action: item.requested_action,
        source_pins: data_copy(&queue.source_pins),
        subject_sha256: item.subject_sha256.clone(),
        context_sha256: item.context_sha256.clone(),
        policy_sha256: item.policy_sha256.clone(),
        reviewer_key: key.into(),
        reviewer_role: "approver".into(),
        disposition,
        responded_at: "2026-09-08T02:01:00Z".into(),
        rationale: "Private rationale must stay out of public dispositions".into(),
        abstention_reason: None,
        proposed_edit: (),
        supersedes: None,
    }
}
/// Persist genuine finite Response bytes; every tested decode uses its original operation ledger.
fn save_response(f: &Fixture, path: &str, document: &ResponseDocumentV3) -> Vec<u8> {
    let raw =
        encode_v3::response(document, &mut ContractLedger::default(), &mut NoopControl).unwrap();
    std::fs::write(f.root.join(path), &raw).unwrap();
    raw
}
/// Capture exactly the maintained complete native closure and every declared Response occurrence.
fn hold(
    f: &Fixture,
    paths: &[&str],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> CurrentAuthoringPlanClosure {
    let mut capture = begin(f, AuthoringOperation::Status, ledger, control);
    let pending =
        receiver::read_pending(&mut capture, Path::new("locator.json"), ledger, control).unwrap();
    capture
        .required(
            Path::new("queue.json"),
            CaptureRole::ReviewQueue,
            Pool::Queue,
            10 * 1024 * 1024,
            ledger,
            control,
        )
        .unwrap();
    for path in paths {
        capture
            .required(
                Path::new(path),
                CaptureRole::ReviewResponse,
                Pool::Response,
                1024 * 1024,
                ledger,
                control,
            )
            .unwrap();
    }
    pending.finish_and_seal(capture, ledger, control).unwrap()
}
/// Genuine complete operation followed by finite output, strict readback, full equality and U fence.
fn recorded(f: &Fixture, paths: &[&str]) -> DispositionsDocumentV3 {
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let owner = hold(f, paths, &mut ledger, &mut control);
    let bound = native_binding::prepare(&owner, &mut ledger, &mut control).unwrap();
    let pending = merge_v3::prepare_policy(
        bound.queue(),
        bound.responses(),
        "2026-09-08T02:10:00Z",
        &mut ledger,
        &mut control,
    )
    .unwrap();
    let current =
        authoring_finalize::finalize(&bound, &pending, &mut ledger, &mut control).unwrap();
    let raw = encode_v3::dispositions(current.document(), &mut ledger, &mut control).unwrap();
    let read = decode_v3::decode_dispositions(&raw, &mut ledger, &mut control).unwrap();
    encode_v3::compare_dispositions(current.document(), read.document(), &mut ledger, &mut control)
        .unwrap();
    ordinary_binding::bind_dispositions(bound.queue(), &read, &mut ledger, &mut control).unwrap();
    current.verify_inputs(&mut ledger, &mut control).unwrap();
    data_copy(read.document())
}
/// Genuine Catalog and Profile/Mapping closures support current declared quorum and complete pins.
#[test]
fn native_catalog_profile_mapping_full_output_and_redaction() {
    for (profile, mapping) in [(false, false), (true, true)] {
        let (f, q, raw) = fixture(profile, mapping);
        save_response(&f, "a.json", &response(&q, &raw, 1, "alice", Disposition::Approve));
        let d = recorded(&f, &["a.json"]);
        assert_eq!(d.currentness, RecordedCurrentness::RecordedCurrent);
        assert_eq!(d.items[0].state, ItemState::QuorumMet);
        assert_eq!(d.counts.response_files, 1);
        assert!(d.source_pins == q.source_pins);
        assert_eq!(d.items[0].met_seats[0].reviewer_key, "alice");
        let public = serde_json::to_string(&d).unwrap();
        for private in [
            "Private rationale",
            "project.json",
            "clause.md",
            "native-project",
            "Private access title",
        ] {
            assert!(!public.contains(private), "private native operand escaped");
        }
        assert!(!f.root.join("new-output.json").exists());
    }
}
/// Plain reframed identity and subject mutations cannot replace exact native project/as-of operands.
#[test]
fn changed_native_identity_and_subject_fail_after_plain_queue_validation() {
    for identity in [true, false] {
        let (f, mut q, _) = fixture(false, false);
        if identity {
            q.items[0].subject_id = format!("authoring-plan:{}", "f".repeat(64));
        } else {
            q.items[0].subject_sha256 = "f".repeat(64);
        }
        reframe(&mut q);
        save_queue(&f, &q);
        let mut l = ContractLedger::default();
        let mut c = NoopControl;
        let mut c = ReviewControl::accept(&mut c);
        let owner = hold(&f, &[], &mut l, &mut c);
        assert_eq!(
            refusal(native_binding::prepare(&owner, &mut l, &mut c)),
            ContractError::Binding
        );
    }
}
/// A coherent plain raw-pin change reaches the native full eight-field roster equality.
#[test]
fn coherent_changed_native_raw_pin_is_not_current() {
    let (f, mut q, _) = fixture(false, false);
    q.source_pins[0].raw_sha256 = "f".repeat(64);
    reframe(&mut q);
    save_queue(&f, &q);
    let mut l = ContractLedger::default();
    let mut caller = NoopControl;
    let mut c = ReviewControl::accept(&mut caller);
    let owner = hold(&f, &[], &mut l, &mut c);
    assert_eq!(refusal(native_binding::prepare(&owner, &mut l, &mut c)), ContractError::Binding);
}
/// A genuine Init native owner has no actual Queue and cannot impersonate a current input operation.
#[test]
fn genuine_init_owner_is_not_a_merge_binding() {
    let f = Fixture::new(false, false);
    originals(&f);
    let mut l = ContractLedger::default();
    let mut caller = NoopControl;
    let mut c = ReviewControl::accept(&mut caller);
    let owner = init(&f, &mut l, &mut c).unwrap();
    assert_eq!(refusal(native_binding::prepare(&owner, &mut l, &mut c)), ContractError::Binding);
}
/// Byte-equal copied Queue declarations fail the actual pointer/full-extent finalizer predicate.
#[test]
fn copied_equal_queue_pending_is_refused() {
    let (f, _, _) = fixture(false, false);
    let mut l = ContractLedger::default();
    let mut caller = NoopControl;
    let mut c = ReviewControl::accept(&mut caller);
    let owner = hold(&f, &[], &mut l, &mut c);
    let bound = native_binding::prepare(&owner, &mut l, &mut c).unwrap();
    let copy = bound.queue().raw().to_vec();
    let q = decode_v3::decode_queue(&copy, &mut l, &mut c).unwrap();
    let pending =
        merge_v3::prepare_policy(&q, bound.responses(), "2026-09-08T02:10:00Z", &mut l, &mut c)
            .unwrap();
    assert_eq!(
        refusal(authoring_finalize::finalize(&bound, &pending, &mut l, &mut c)),
        ContractError::Binding
    );
}
/// Byte-equal copied Response cannot replace its genuine actual registration allocation.
#[test]
fn copied_equal_response_pending_is_refused() {
    let (f, q, raw) = fixture(false, false);
    save_response(&f, "a.json", &response(&q, &raw, 1, "alice", Disposition::Approve));
    let mut l = ContractLedger::default();
    let mut caller = NoopControl;
    let mut c = ReviewControl::accept(&mut caller);
    let owner = hold(&f, &["a.json"], &mut l, &mut c);
    let bound = native_binding::prepare(&owner, &mut l, &mut c).unwrap();
    let copy = bound.responses()[0].raw().to_vec();
    let copied = [decode_v3::decode_response(&copy, &mut l, &mut c).unwrap()];
    let pending =
        merge_v3::prepare_policy(bound.queue(), &copied, "2026-09-08T02:10:00Z", &mut l, &mut c)
            .unwrap();
    assert_eq!(
        refusal(authoring_finalize::finalize(&bound, &pending, &mut l, &mut c)),
        ContractError::Binding
    );
}
/// Omitted or reordered real registrations cannot satisfy complete positional occurrence binding.
#[test]
fn omitted_or_reordered_actual_duplicate_cohort_is_refused() {
    let (f, q, raw) = fixture(false, false);
    save_response(&f, "a.json", &response(&q, &raw, 1, "alice", Disposition::Approve));
    save_response(&f, "b.json", &response(&q, &raw, 2, "bob", Disposition::Approve));
    let mut l = ContractLedger::default();
    let mut caller = NoopControl;
    let mut c = ReviewControl::accept(&mut caller);
    let owner = hold(&f, &["a.json", "b.json", "a.json"], &mut l, &mut c);
    let bound = native_binding::prepare(&owner, &mut l, &mut c).unwrap();
    let short = merge_v3::prepare_policy(
        bound.queue(),
        &bound.responses()[..2],
        "2026-09-08T02:10:00Z",
        &mut l,
        &mut c,
    )
    .unwrap();
    assert_eq!(
        refusal(authoring_finalize::finalize(&bound, &short, &mut l, &mut c)),
        ContractError::Binding
    );
    let reordered = vec![
        decode_v3::decode_response(bound.responses()[1].raw(), &mut l, &mut c).unwrap(),
        decode_v3::decode_response(bound.responses()[0].raw(), &mut l, &mut c).unwrap(),
        decode_v3::decode_response(bound.responses()[2].raw(), &mut l, &mut c).unwrap(),
    ];
    let pending =
        merge_v3::prepare_policy(bound.queue(), &reordered, "2026-09-08T02:10:00Z", &mut l, &mut c)
            .unwrap();
    assert_eq!(
        refusal(authoring_finalize::finalize(&bound, &pending, &mut l, &mut c)),
        ContractError::Binding
    );
}
/// Foreign/stale/unassigned/future/late files remain complete visible evidence without approvals.
#[test]
fn complete_noncurrent_response_classes_are_preserved() {
    let (f, mut q, _) = fixture(false, false);
    q.items[0].due_at = Some("2026-09-08T02:05:00Z".into());
    reframe(&mut q);
    let raw = save_queue(&f, &q);
    let mut rows = Vec::new();
    let mut row = response(&q, &raw, 1, "alice", Disposition::Reject);
    row.queue_id = review_id(99);
    rows.push(row);
    let mut row = response(&q, &raw, 2, "bob", Disposition::Approve);
    row.subject_sha256 = "f".repeat(64);
    rows.push(row);
    rows.push(response(&q, &raw, 3, "outsider", Disposition::Approve));
    let mut row = response(&q, &raw, 4, "alice", Disposition::Approve);
    row.responded_at = "2026-09-08T02:11:00Z".into();
    rows.push(row);
    let mut row = response(&q, &raw, 5, "bob", Disposition::Approve);
    row.responded_at = "2026-09-08T02:05:00Z".into();
    rows.push(row);
    for (n, row) in rows.iter().enumerate() {
        save_response(&f, &format!("r{n}.json"), row);
    }
    let d = recorded(&f, &["r0.json", "r1.json", "r2.json", "r3.json", "r4.json"]);
    let classes: Vec<_> = d.responses.iter().map(|r| r.classification).collect();
    assert_eq!(
        classes,
        vec![
            ResponseClassification::Foreign,
            ResponseClassification::Stale,
            ResponseClassification::Unassigned,
            ResponseClassification::Future,
            ResponseClassification::Late
        ]
    );
    assert_eq!(d.counts.response_files, 5);
    assert_eq!(d.items[0].dissent_ids, vec![review_id(1)]);
    assert!(d.items[0].met_seats.is_empty());
    assert_eq!(d.items[0].state, ItemState::Stale);
}
/// One explicit identity with distinct full raw bytes is fatal, regardless of equal parsed vote.
#[test]
fn same_response_uuid_different_raw_is_fatal() {
    let (f, q, raw) = fixture(false, false);
    let original =
        save_response(&f, "a.json", &response(&q, &raw, 1, "alice", Disposition::Approve));
    let mut changed = original.clone();
    changed.push(b' ');
    std::fs::write(f.root.join("b.json"), changed).unwrap();
    let mut l = ContractLedger::default();
    let mut caller = NoopControl;
    let mut c = ReviewControl::accept(&mut caller);
    let owner = hold(&f, &["a.json", "b.json"], &mut l, &mut c);
    let bound = native_binding::prepare(&owner, &mut l, &mut c).unwrap();
    assert_eq!(
        refusal(merge_v3::prepare_policy(
            bound.queue(),
            bound.responses(),
            "2026-09-08T02:10:00Z",
            &mut l,
            &mut c
        )),
        ContractError::IdentityConflict
    );
}
/// A genuine exact withdrawal chain retains history and withdrawal-of-withdrawal never reopens.
#[test]
fn withdrawal_of_withdrawal_preserves_dissent_without_reopening() {
    let (f, q, raw) = fixture(false, false);
    let first = response(&q, &raw, 1, "alice", Disposition::Reject);
    let first_raw = save_response(&f, "a.json", &first);
    let mut second = response(&q, &raw, 2, "alice", Disposition::Superseded);
    second.supersedes = Some(SupersessionReference {
        response_id: review_id(1),
        raw_sha256: sha256_hex(&first_raw),
    });
    let second_raw = save_response(&f, "b.json", &second);
    let mut third = response(&q, &raw, 3, "alice", Disposition::Superseded);
    third.supersedes = Some(SupersessionReference {
        response_id: review_id(2),
        raw_sha256: sha256_hex(&second_raw),
    });
    save_response(&f, "c.json", &third);
    let d = recorded(&f, &["a.json", "b.json", "c.json"]);
    assert!(d.responses.iter().all(|r| r.classification == ResponseClassification::Superseded));
    assert_eq!(d.items[0].dissent_ids, vec![review_id(1)]);
    assert!(d.items[0].met_seats.is_empty());
}
/// Two actual same-subject children of one exact prior constitute an invalid branch.
#[test]
fn exact_withdrawal_branch_is_refused_as_a_whole() {
    let (f, q, raw) = fixture(false, false);
    let first = save_response(&f, "a.json", &response(&q, &raw, 1, "alice", Disposition::Approve));
    for (id, path) in [(2, "b.json"), (3, "c.json")] {
        let mut row = response(&q, &raw, id, "alice", Disposition::Superseded);
        row.supersedes = Some(SupersessionReference {
            response_id: review_id(1),
            raw_sha256: sha256_hex(&first),
        });
        save_response(&f, path, &row);
    }
    let mut l = ContractLedger::default();
    let mut caller = NoopControl;
    let mut c = ReviewControl::accept(&mut caller);
    let owner = hold(&f, &["a.json", "b.json", "c.json"], &mut l, &mut c);
    let bound = native_binding::prepare(&owner, &mut l, &mut c).unwrap();
    assert_eq!(
        refusal(merge_v3::prepare_policy(
            bound.queue(),
            bound.responses(),
            "2026-09-08T02:10:00Z",
            &mut l,
            &mut c
        )),
        ContractError::InvalidChain
    );
}
/// Only actual eligible timely withdrawal can retire a current approval.
#[test]
fn future_withdrawal_does_not_erase_timely_approval() {
    let (f, q, raw) = fixture(false, false);
    let first = save_response(&f, "a.json", &response(&q, &raw, 1, "alice", Disposition::Approve));
    let mut row = response(&q, &raw, 2, "alice", Disposition::Superseded);
    row.responded_at = "2026-09-08T02:11:00Z".into();
    row.supersedes =
        Some(SupersessionReference { response_id: review_id(1), raw_sha256: sha256_hex(&first) });
    save_response(&f, "b.json", &row);
    let d = recorded(&f, &["a.json", "b.json"]);
    assert_eq!(d.items[0].state, ItemState::QuorumMet);
    assert_eq!(d.responses[0].classification, ResponseClassification::Current);
    assert_eq!(d.responses[1].classification, ResponseClassification::Future);
}
/// Complete distinct-key matching succeeds where greedy first-fit would strand a required seat.
#[test]
fn maximum_matching_requires_distinct_declared_keys() {
    let (f, mut q, _) = fixture(false, false);
    q.policies[0].seats = vec![
        SeatRequirement { role_key: "approver".into(), count: 1 },
        SeatRequirement { role_key: "owner".into(), count: 1 },
    ];
    q.policies[0].substitutions = vec![Substitution {
        seat_role: "owner".into(),
        reviewer_key: "alice".into(),
        asserted_role: "approver".into(),
        reason_code: "backup".into(),
    }];
    reframe(&mut q);
    let raw = save_queue(&f, &q);
    save_response(&f, "a.json", &response(&q, &raw, 1, "alice", Disposition::Approve));
    save_response(&f, "b.json", &response(&q, &raw, 2, "bob", Disposition::Approve));
    let d = recorded(&f, &["a.json", "b.json"]);
    assert_eq!(d.items[0].state, ItemState::QuorumMet);
    assert_eq!(d.items[0].met_seats.len(), 2);
    assert_eq!(d.items[0].met_seats[0].reviewer_key, "bob");
    assert_eq!(d.items[0].met_seats[1].reviewer_key, "alice");
    let one = recorded(&f, &["a.json", "a.json"]);
    assert_eq!(one.items[0].met_seats.len(), 1);
    assert_eq!(one.counts.response_files, 2);
    assert_eq!(one.counts.unique_responses, 1);
    assert_eq!(one.counts.exact_duplicates, 1);
}
/// Asserted author keys and declared empty abstentions never supply approvals.
#[test]
fn author_and_empty_abstention_remain_nonapproving() {
    let (f, q, raw) = fixture(false, false);
    save_response(&f, "a.json", &response(&q, &raw, 1, "author", Disposition::Approve));
    let mut abstain = response(&q, &raw, 2, "alice", Disposition::Abstain);
    abstain.rationale.clear();
    abstain.abstention_reason = Some("not-qualified".into());
    save_response(&f, "b.json", &abstain);
    let d = recorded(&f, &["a.json", "b.json"]);
    assert_eq!(d.responses[0].classification, ResponseClassification::Unassigned);
    assert!(d.items[0].met_seats.is_empty());
    assert_eq!(d.items[0].state, ItemState::InReview);
    assert!(d.items[0].reason_codes.iter().any(|r| r == "nonapproving-abstention"));
}
/// Independent rejection/request-change and same-key divergence survive complete output projection.
#[test]
fn conflicts_and_requests_preserve_all_dissent() {
    let (f, q, raw) = fixture(false, false);
    save_response(&f, "a.json", &response(&q, &raw, 1, "alice", Disposition::Approve));
    save_response(&f, "b.json", &response(&q, &raw, 2, "alice", Disposition::Reject));
    save_response(&f, "c.json", &response(&q, &raw, 3, "bob", Disposition::RequestChanges));
    let d = recorded(&f, &["a.json", "b.json", "c.json"]);
    assert_eq!(d.items[0].state, ItemState::Conflicted);
    assert!(d.items[0].blocking);
    assert_eq!(d.items[0].dissent_ids, vec![review_id(2), review_id(3)]);
    assert_eq!(d.responses[0].classification, ResponseClassification::Conflicted);
    assert_eq!(d.responses[1].classification, ResponseClassification::Conflicted);
}
/// Raw generation retains actual order and duplicates even when unique policy output is equal.
#[test]
fn generation_binds_full_order_and_duplicate_occurrences() {
    let (f, q, raw) = fixture(false, false);
    save_response(&f, "a.json", &response(&q, &raw, 1, "alice", Disposition::Approve));
    save_response(&f, "b.json", &response(&q, &raw, 2, "bob", Disposition::Approve));
    let first = recorded(&f, &["a.json", "b.json"]);
    let order = recorded(&f, &["b.json", "a.json"]);
    let duplicate = recorded(&f, &["a.json", "b.json", "a.json"]);
    assert_ne!(first.closure_generation, order.closure_generation);
    assert_ne!(first.closure_generation, duplicate.closure_generation);
    assert_eq!(duplicate.counts.response_files, 3);
    assert_eq!(duplicate.counts.unique_responses, 2);
    assert_eq!(duplicate.counts.exact_duplicates, 1);
    assert_eq!(
        serde_json::to_value(&first.items).unwrap(),
        serde_json::to_value(&order.items).unwrap()
    );
}
/// A later native/locator/Queue/Response drift invalidates the whole retained current output owner.
#[test]
fn whole_owner_fence_after_finite_output_rejects_real_drift() {
    for changed in ["clause.md", "locator.json", "queue.json", "a.json"] {
        let (f, q, raw) = fixture(false, false);
        save_response(&f, "a.json", &response(&q, &raw, 1, "alice", Disposition::Approve));
        let mut l = ContractLedger::default();
        let mut caller = NoopControl;
        let mut c = ReviewControl::accept(&mut caller);
        let owner = hold(&f, &["a.json"], &mut l, &mut c);
        let bound = native_binding::prepare(&owner, &mut l, &mut c).unwrap();
        let pending = merge_v3::prepare_policy(
            bound.queue(),
            bound.responses(),
            "2026-09-08T02:10:00Z",
            &mut l,
            &mut c,
        )
        .unwrap();
        let current = authoring_finalize::finalize(&bound, &pending, &mut l, &mut c).unwrap();
        let output = encode_v3::dispositions(current.document(), &mut l, &mut c).unwrap();
        assert_ne!(output.as_slice(), &[] as &[u8]);
        std::fs::write(f.root.join(changed), b"changed actual held original").unwrap();
        assert_eq!(current.verify_inputs(&mut l, &mut c).unwrap_err(), ContractError::Binding);
        assert!(!f.root.join("new-output.json").exists());
    }
}
/// Recorded Current and its digest remain ordinary data; witness role must bind to its exact seat.
#[test]
fn recorded_witness_wrong_seat_is_not_an_authorizing_claim() {
    let (fixture, declarations, raw) = fixture(false, false);
    save_response(
        &fixture,
        "a.json",
        &response(&declarations, &raw, 1, "alice", Disposition::Approve),
    );
    let mut dispositions = recorded(&fixture, &["a.json"]);
    dispositions.items[0].met_seats[0].role_key = "owner".into();
    let bytes =
        encode_v3::dispositions(&dispositions, &mut ContractLedger::default(), &mut NoopControl)
            .unwrap();
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let queue = decode_v3::decode_queue(&raw, &mut ledger, &mut control).unwrap();
    let read = decode_v3::decode_dispositions(&bytes, &mut ledger, &mut control).unwrap();
    assert_eq!(
        ordinary_binding::bind_dispositions(&queue, &read, &mut ledger, &mut control).unwrap_err(),
        ContractError::Binding
    );
}
/// Original authentic checkpoint observations support calibration without timer or ledger renewal.
#[derive(Default)]
struct ProbeState {
    calls: usize,
    stop: Option<usize>,
    fail: bool,
}
/// Shared observation of one actual caller; no native success fact is supplied by the probe.
struct Caller {
    state: Rc<RefCell<ProbeState>>,
}
impl WorkControl for Caller {
    /// Return only the genuine transient callback error at its calibrated actual ordinal.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
        let mut state = self.state.borrow_mut();
        state.calls += 1;
        if state.stop == Some(state.calls) {
            return if state.fail {
                Err(WorkError::Failed(Error::invalid()))
            } else {
                Err(WorkError::Interrupted(Interruption::CancelRequested))
            };
        }
        Ok(())
    }
    /// The caller has no synthetic latched status; the original ledger preserves first stop.
    fn interruption(&self) -> Option<Interruption> {
        None
    }
}
/// Success and ordinary copied-input failure both have unconditional terminal phase checkpoints.
#[test]
fn finalizer_terminal_success_and_ordinary_failure_stop_is_sticky() {
    for copied in [false, true] {
        for failed in [false, true] {
            let (fixture, _, _) = fixture(false, false);
            let state = Rc::new(RefCell::new(ProbeState::default()));
            let mut caller = Caller { state: Rc::clone(&state) };
            let mut control = ReviewControl::accept(&mut caller);
            let mut ledger = ContractLedger::default();
            let owner = hold(&fixture, &[], &mut ledger, &mut control);
            let bound = native_binding::prepare(&owner, &mut ledger, &mut control).unwrap();
            let copy = bound.queue().raw().to_vec();
            let decoded_queue = decode_v3::decode_queue(&copy, &mut ledger, &mut control).unwrap();
            let selected = if copied { &decoded_queue } else { bound.queue() };
            let pending = merge_v3::prepare_policy(
                selected,
                bound.responses(),
                "2026-09-08T02:10:00Z",
                &mut ledger,
                &mut control,
            )
            .unwrap();
            let before = state.borrow().calls;
            let baseline =
                authoring_finalize::finalize(&bound, &pending, &mut ledger, &mut control);
            if copied {
                assert_eq!(refusal(baseline), ContractError::Binding);
            } else {
                assert!(baseline.is_ok());
            }
            let delta = state.borrow().calls - before;
            assert!(delta >= 2);
            {
                let mut observation = state.borrow_mut();
                observation.stop = Some(observation.calls + delta);
                observation.fail = failed;
            }
            let expected = if failed {
                ContractError::ControlFailed
            } else {
                ContractError::Interrupted(Interruption::CancelRequested)
            };
            assert_eq!(
                refusal(authoring_finalize::finalize(&bound, &pending, &mut ledger, &mut control)),
                expected
            );
            let callbacks = state.borrow().calls;
            assert_eq!(ledger.bytes(0).unwrap_err(), expected);
            assert_eq!(bound.verify_inputs(&mut ledger, &mut control).unwrap_err(), expected);
            assert_eq!(state.borrow().calls, callbacks);
        }
    }
}
/// First real ledger Capacity wins over later ordinary binding and transient callback stop.
#[test]
fn first_capacity_precedes_later_control_and_binding_failure() {
    let (f, _, _) = fixture(false, false);
    let state = Rc::new(RefCell::new(ProbeState::default()));
    let mut caller = Caller { state: Rc::clone(&state) };
    let mut c = ReviewControl::accept(&mut caller);
    let mut l = ContractLedger::default();
    let owner = hold(&f, &[], &mut l, &mut c);
    let bound = native_binding::prepare(&owner, &mut l, &mut c).unwrap();
    let pending = merge_v3::prepare_policy(
        bound.queue(),
        bound.responses(),
        "2026-09-08T02:10:00Z",
        &mut l,
        &mut c,
    )
    .unwrap();
    assert_eq!(l.derived(33_554_433).unwrap_err(), ContractError::Capacity);
    let callbacks = state.borrow().calls;
    {
        let mut s = state.borrow_mut();
        s.stop = Some(s.calls + 1);
        s.fail = true;
    }
    assert_eq!(
        refusal(authoring_finalize::finalize(&bound, &pending, &mut l, &mut c)),
        ContractError::Capacity
    );
    assert_eq!(state.borrow().calls, callbacks);
    assert!(!f.root.join("new-output.json").exists());
}

/// Actual malformed held Queue returns ordinary Invalid through an unconditional post-phase fence.
#[test]
fn held_malformed_queue_ordinary_failure_has_terminal_stop_fence() {
    for failed in [false, true] {
        let (f, _, _) = fixture(false, false);
        std::fs::write(f.root.join("queue.json"), b"{}").unwrap();
        let state = Rc::new(RefCell::new(ProbeState::default()));
        let mut caller = Caller { state: Rc::clone(&state) };
        let mut c = ReviewControl::accept(&mut caller);
        let mut l = ContractLedger::default();
        let owner = hold(&f, &[], &mut l, &mut c);
        let before = state.borrow().calls;
        assert_eq!(
            refusal(native_binding::prepare(&owner, &mut l, &mut c)),
            ContractError::Invalid
        );
        let delta = state.borrow().calls - before;
        assert!(delta >= 2);
        l.bytes(0).unwrap();
        {
            let mut s = state.borrow_mut();
            s.stop = Some(s.calls + delta);
            s.fail = failed;
        }
        let expected = if failed {
            ContractError::ControlFailed
        } else {
            ContractError::Interrupted(Interruption::CancelRequested)
        };
        assert_eq!(refusal(native_binding::prepare(&owner, &mut l, &mut c)), expected);
        assert_eq!(l.bytes(0).unwrap_err(), expected);
    }
}
/// Original raw identity includes syntax-only whitespace and cannot be replaced by typed equality.
#[test]
fn separately_decoded_original_relation_retains_full_raw_extent() {
    let (fixture, declarations, raw) = fixture(false, false);
    let first = save_response(
        &fixture,
        "a.json",
        &response(&declarations, &raw, 1, "alice", Disposition::Approve),
    );
    let mut changed = first.clone();
    changed.push(b' ');
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let first_decoded = decode_v3::decode_response(&first, &mut ledger, &mut control).unwrap();
    let changed_decoded = decode_v3::decode_response(&changed, &mut ledger, &mut control).unwrap();
    assert!(first_decoded.document() == changed_decoded.document());
    assert_eq!(
        ordinary_binding::original_relation(
            &first_decoded,
            &changed_decoded,
            &mut ledger,
            &mut control
        )
        .unwrap(),
        crate::review::decode::OriginalRelation::IdentityConflict
    );
    let same = decode_v3::decode_response(&first, &mut ledger, &mut control).unwrap();
    assert_eq!(
        ordinary_binding::original_relation(&first_decoded, &same, &mut ledger, &mut control)
            .unwrap(),
        crate::review::decode::OriginalRelation::ExactDuplicate
    );
}
