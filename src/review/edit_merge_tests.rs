// Genuine edit-capable /4 merge controls over the complete native fixture: AC-5
// wrong-target classification, invalid-edit classification, the never-applied
// guarantee (G-4) and a valid proposed-edit round-trip through the full pipeline.
// Root alone executes these; no constructed lease/currentness/approval token is accepted.
use super::*;
use crate::hashing::sha256_hex;
use crate::review::wire_v3::*;
use crate::review::wire_v4::{DispositionsDocumentV4, PatchFormat, ProposedEdit, ResponseDocumentV4};
use crate::review::{
    decode_v4, edit_binding, edit_finalize, edit_response_binding, encode_v3, encode_v4,
    hash_v3, merge_v4,
};

/// Canonical review UUID; this is never the native project key.
fn review_id(n: u8) -> String {
    format!("{n:08x}-1111-4111-8111-111111111111")
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
/// Derive the ordinary /3 Queue from a genuinely prepared Init owner. The edit-capable
/// family reuses the unchanged closed /3 queue shape; only responses gain an edit slot.
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
        roles: vec![RoleDefinition { key: "approver".into() }],
        reviewers: vec![
            Reviewer { key: "alice".into(), role_keys: vec!["approver".into()] },
            Reviewer { key: "bob".into(), role_keys: vec!["approver".into()] },
        ],
        policies: vec![ReviewPolicy {
            key: "review".into(),
            seats: vec![SeatRequirement { role_key: "approver".into(), count: 1 }],
            substitutions: vec![],
            abstention_rule: AbstentionRule::Nonapproving,
            empty_abstention_reasons: vec![],
            author_separation: AuthorSeparation::DeclaredKeys,
        }],
        items: vec![ReviewItemV3 {
            key: "whole-plan".into(),
            item_id: String::new(),
            domain: DomainV3::AuthoringPlan,
            adapter_version: ADAPTER.into(),
            requested_action: RequestedAction::ReReview,
            subject_id: identity,
            subject_sha256: subject,
            source_keys: vec![],
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
/// Genuine complete fixture, real Init owner, actual Queue bytes and their original digest.
fn fixture() -> (Fixture, QueueDocumentV3, Vec<u8>) {
    let f = Fixture::new(false, false);
    originals(&f);
    let owner =
        init(&f, &mut ContractLedger::default(), &mut NoopControl).expect("actual Init closure");
    let queue = native_queue(&owner);
    let raw = save_queue(&f, &queue);
    (f, queue, raw)
}
/// Persist the genuine complete Queue original and return its exact bytes.
fn save_queue(f: &Fixture, queue: &QueueDocumentV3) -> Vec<u8> {
    let raw =
        encode_v3::queue(queue, &mut ContractLedger::default(), &mut NoopControl).unwrap();
    std::fs::write(f.root.join("queue.json"), &raw).unwrap();
    raw
}
/// One inert proposed edit bound to a target subject, with an exact patch digest.
fn edit(target: &str, patch: &str) -> ProposedEdit {
    ProposedEdit {
        target_subject_sha256: target.into(),
        patch_format: PatchFormat::FullText,
        patch: patch.into(),
        patch_sha256: sha256_hex(patch.as_bytes()),
    }
}
/// Complete edit-capable response bound to the actual Queue original.
fn response(
    queue: &QueueDocumentV3,
    raw: &[u8],
    n: u8,
    key: &str,
    proposed_edit: Option<ProposedEdit>,
) -> ResponseDocumentV4 {
    let item = &queue.items[0];
    ResponseDocumentV4 {
        schema_version: "forge.review-response/4".into(),
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
        disposition: Disposition::Approve,
        responded_at: "2026-09-08T02:01:00Z".into(),
        rationale: "Private rationale must stay out of public dispositions".into(),
        abstention_reason: None,
        proposed_edit,
        supersedes: None,
    }
}
/// Persist genuine finite edit-capable Response bytes; every decode uses its original ledger.
fn save_response(f: &Fixture, path: &str, document: &ResponseDocumentV4) -> Vec<u8> {
    let raw =
        encode_v4::response(document, &mut ContractLedger::default(), &mut NoopControl).unwrap();
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
    let mut capture = begin(f, AuthoringOperation::Merge, ledger, control);
    let pending = receiver::read_pending(&mut capture, Path::new("locator.json"), ledger, control)
        .unwrap();
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
/// Genuine complete merge: bind, evaluate policy, finalize, encode, strict readback and U fence.
/// Returns the recorded bundle and the pending evaluation for classification assertions.
fn merge(
    f: &Fixture,
    paths: &[&str],
) -> (DispositionsDocumentV4, Vec<crate::review::wire::ResponseClassification>) {
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let owner = hold(f, paths, &mut ledger, &mut control);
    let bound = edit_binding::prepare(&owner, &mut ledger, &mut control).unwrap();
    let pending = merge_v4::prepare_policy(
        bound.queue(),
        bound.responses(),
        "2026-09-08T02:10:00Z",
        &mut ledger,
        &mut control,
    )
    .unwrap();
    let classes: Vec<_> = pending.responses().iter().map(|r| r.classification).collect();
    let current = edit_finalize::finalize(&bound, &pending, &mut ledger, &mut control).unwrap();
    let raw = encode_v4::dispositions(current.document(), &mut ledger, &mut control).unwrap();
    let read = decode_v4::decode_dispositions(&raw, &mut ledger, &mut control).unwrap();
    encode_v4::compare_dispositions(current.document(), read.document(), &mut ledger, &mut control)
        .unwrap();
    edit_response_binding::bind_dispositions(bound.queue(), &read, &mut ledger, &mut control)
        .unwrap();
    current.verify_inputs(&mut ledger, &mut control).unwrap();
    (data_copy(read.document()), classes)
}

/// AC-5: a proposed edit bound to a different subject is classified stale on that response,
/// the merge continues, and a clean response from another reviewer still fills the seat.
#[test]
fn wrong_target_hash_is_stale_and_merge_continues() {
    let (f, q, raw) = fixture();
    let wrong = "e".repeat(64);
    assert_ne!(wrong, q.items[0].subject_sha256);
    save_response(
        &f,
        "a.json",
        &response(&q, &raw, 1, "alice", Some(edit(&wrong, "# stale target patch\n"))),
    );
    save_response(&f, "b.json", &response(&q, &raw, 2, "bob", None));
    let (recorded, classes) = merge(&f, &["a.json", "b.json"]);
    assert_eq!(classes, vec![ResponseClassification::Stale, ResponseClassification::Current]);
    assert_eq!(recorded.responses[0].classification, ResponseClassification::Stale);
    assert_eq!(recorded.responses[1].classification, ResponseClassification::Current);
    // The clean response still satisfies the declared seat; the stale edit never blocks it.
    assert_eq!(recorded.items[0].state, ItemState::QuorumMet);
    assert_eq!(recorded.items[0].met_seats.len(), 1);
    assert_eq!(recorded.items[0].met_seats[0].reviewer_key, "bob");
}

/// A malformed edit envelope (wrong patch digest) is classified invalid-edit on that response;
/// the merge continues and a clean response from another reviewer still fills the seat.
#[test]
fn invalid_edit_is_classified_and_merge_continues() {
    let (f, q, raw) = fixture();
    let mut bad = edit(&q.items[0].subject_sha256, "# replacement text\n");
    bad.patch_sha256 = "f".repeat(64); // digest does not match the patch bytes
    save_response(&f, "a.json", &response(&q, &raw, 1, "alice", Some(bad)));
    save_response(&f, "b.json", &response(&q, &raw, 2, "bob", None));
    let (recorded, classes) = merge(&f, &["a.json", "b.json"]);
    assert_eq!(classes, vec![ResponseClassification::InvalidEdit, ResponseClassification::Current]);
    assert_eq!(recorded.responses[0].classification, ResponseClassification::InvalidEdit);
    assert_eq!(recorded.responses[1].classification, ResponseClassification::Current);
    // The clean response still satisfies the declared seat; the invalid edit never supplies one.
    assert_eq!(recorded.items[0].state, ItemState::QuorumMet);
    assert_eq!(recorded.items[0].met_seats.len(), 1);
    assert_eq!(recorded.items[0].met_seats[0].reviewer_key, "bob");
}

/// G-4 never-applied: a valid proposed edit is retained as inert evidence and the complete
/// native source originals are byte-for-byte unchanged after the full merge pipeline.
#[test]
fn proposed_edit_is_never_applied_to_sources() {
    let (f, q, raw) = fixture();
    let before: Vec<(String, Vec<u8>)> = ["project.json", "stored-plan.json", "locator.json"]
        .iter()
        .map(|name| ((*name).to_owned(), std::fs::read(f.root.join(name)).unwrap()))
        .collect();
    let patch = "# Proposed replacement plan text — inert until a human promotes it\n";
    save_response(
        &f,
        "a.json",
        &response(&q, &raw, 1, "alice", Some(edit(&q.items[0].subject_sha256, patch))),
    );
    let (recorded, classes) = merge(&f, &["a.json"]);
    assert_eq!(classes, vec![ResponseClassification::Current]);
    assert_eq!(recorded.items[0].state, ItemState::QuorumMet);
    // Every native source original is byte-for-byte unchanged: the patch was never applied.
    for (name, bytes) in &before {
        assert_eq!(&std::fs::read(f.root.join(name)).unwrap(), bytes, "source {name} mutated");
    }
    // The recorded public bundle carries no patch text: edits stay private inert evidence.
    let public = serde_json::to_string(&recorded).unwrap();
    assert!(!public.contains("Proposed replacement plan text"));
}

/// A valid proposed edit round-trips through the complete pipeline: respond bytes decode,
/// bind to the actual Queue, merge to quorum, and the recorded bundle readback is exact.
#[test]
fn valid_proposed_edit_round_trips_through_merge() {
    let (f, q, raw) = fixture();
    let patch = "# Valid replacement plan text\n";
    let document = response(&q, &raw, 1, "alice", Some(edit(&q.items[0].subject_sha256, patch)));
    let response_raw = save_response(&f, "a.json", &document);
    // The emitted response decodes strictly and retains the exact inert edit.
    let read = decode_v4::decode_response(
        &response_raw,
        &mut ContractLedger::default(),
        &mut NoopControl,
    )
    .unwrap();
    assert!(read.document() == &document);
    assert_eq!(read.document().proposed_edit.as_ref().unwrap().patch, patch);
    let (recorded, classes) = merge(&f, &["a.json"]);
    assert_eq!(classes, vec![ResponseClassification::Current]);
    assert_eq!(recorded.items[0].state, ItemState::QuorumMet);
    assert_eq!(recorded.items[0].met_seats[0].reviewer_key, "alice");
    assert_eq!(recorded.counts.unique_responses, 1);
}

/// The zero-edit contract holds: a /4 response with a null edit merges exactly like /3,
/// and the edit-capable family never mutates the native plan or grants domain approval.
#[test]
fn null_edit_preserves_zero_edit_quorum_contract() {
    let (f, q, raw) = fixture();
    save_response(&f, "a.json", &response(&q, &raw, 1, "alice", None));
    save_response(&f, "b.json", &response(&q, &raw, 2, "bob", None));
    let (recorded, classes) = merge(&f, &["a.json", "b.json"]);
    assert_eq!(classes, vec![ResponseClassification::Current, ResponseClassification::Current]);
    assert_eq!(recorded.items[0].state, ItemState::QuorumMet);
    assert_eq!(recorded.currentness, RecordedCurrentness::RecordedCurrent);
    assert!(recorded.closure_generation.is_some());
}
