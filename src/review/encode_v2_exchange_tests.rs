//! Prospective finite typed /2 queue/response controls; all fixtures are inert declarations.
//! The actual consumer shares one supplied ledger/controller; fixture framing grants no owner.
use super::{
    MAX_QUEUE_BYTES, MAX_RESPONSE_BYTES, encoded, queue, readback_queue, readback_response,
    response,
};
use crate::review::decode::{ContractError, ContractLedger};
use crate::review::wire::{SeatRequirement, Substitution};
use crate::review::wire_v2::*;
use crate::review::{decode_v2, hash_v2};
use crate::workspace::contract::Error;
use crate::workspace::preparation::{
    Interruption, ProgressUpdate, Stage, WorkControl, WorkError, WorkResult,
};

/// Same component caller and absolute callback count across repeated real codec calls.
#[derive(Default)]
struct Caller {
    /// Actual callback count, never reset between consumers within a control.
    calls: usize,
    /// Optional absolute actual checkpoint to stop on.
    at: Option<usize>,
    /// Return safe control failure instead of typed cancellation at that same point.
    failed: bool,
}
impl WorkControl for Caller {
    /// Observe the real codec boundary without an invented native or currentness flag.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
        self.calls += 1;
        if self.at == Some(self.calls) {
            if self.failed {
                return Err(WorkError::Failed(Error::invalid()));
            }
            return Err(WorkError::Interrupted(Interruption::CancelRequested));
        }
        Ok(())
    }
    /// Stops are supplied through the genuine callback result, not loaded labels.
    fn interruption(&self) -> Option<Interruption> {
        None
    }
}
/// Preserve the exact original error category for a selected real callback.
fn stop(failed: bool) -> ContractError {
    if failed {
        ContractError::ControlFailed
    } else {
        ContractError::Interrupted(Interruption::CancelRequested)
    }
}
/// Complete source declarations with empty opaque bytes and original native UUID spelling.
fn pins() -> Vec<SourcePinV2> {
    vec![
        SourcePinV2 {
            artifact_key: "lifecycle:generated:0".into(),
            kind: SourceKindV2::GeneratedArtifact,
            raw_sha256: "2".repeat(64),
            byte_length: 128,
            schema_identity: None,
            validation_profile: "forge.lifecycle-generated-identity/1".into(),
            native_model: Some(NativeModelV2::Catalog),
            native_root_uuid: Some("{ABCDEFAB-1234-5567-89AB-CDEF01234567}".into()),
        },
        SourcePinV2 {
            artifact_key: "lifecycle:record".into(),
            kind: SourceKindV2::LifecycleRecord,
            raw_sha256: "3".repeat(64),
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
    ]
}
/// Complete ordinary one-item policy/registry with no captured or native approval owner.
fn queue_fixture() -> QueueDocumentV2 {
    let source_pins = pins();
    let mut document = QueueDocumentV2 {
        schema_version: "forge.review-queue/2".into(),
        identity_disclaimer: IDENTITY_DISCLAIMER.into(),
        sensitivity: Sensitivity::IdsAndHashes,
        queue_id: "11111111-1111-4111-8111-111111111111".into(),
        created_at: "2026-10-05T00:00:00Z".into(),
        roles: vec![RoleDefinition { key: "review".into() }],
        reviewers: vec![
            Reviewer { key: "author".into(), role_keys: vec!["review".into()] },
            Reviewer { key: "reviewer".into(), role_keys: vec!["review".into()] },
        ],
        policies: vec![ReviewPolicy {
            key: "policy".into(),
            seats: vec![SeatRequirement { role_key: "review".into(), count: 1 }],
            substitutions: vec![],
            abstention_rule: AbstentionRule::Nonapproving,
            empty_abstention_reasons: vec!["no-conflict".into()],
            author_separation: AuthorSeparation::DeclaredKeys,
        }],
        items: vec![ReviewItemV2 {
            key: "item".into(),
            item_id: String::new(),
            domain: DomainV2::LifecyclePolicyVersion,
            adapter_version: ADAPTER.into(),
            subject_id: format!("lifecycle:{}", "1".repeat(64)),
            requested_action: RequestedAction::ReReview,
            source_keys: source_pins.iter().map(|pin| pin.artifact_key.clone()).collect(),
            subject_sha256: "4".repeat(64),
            context: ContextSnapshot {
                reason_codes: vec!["lifecycle-approved-current".into()],
                related_subject_ids: vec![],
            },
            context_sha256: String::new(),
            policy_key: "policy".into(),
            policy_sha256: String::new(),
            author_keys: vec!["author".into()],
            assignments: vec![Assignment {
                reviewer_key: "reviewer".into(),
                role_key: "review".into(),
            }],
            due_at: None,
            allowed_dispositions: vec![
                Disposition::Approve,
                Disposition::Reject,
                Disposition::RequestChanges,
                Disposition::Abstain,
                Disposition::Superseded,
            ],
        }],
        source_pins,
    };
    refresh(&mut document);
    document
}
/// Genuine ordinary framing refresh before changed fixture input; no production allowance renews.
fn refresh(document: &mut QueueDocumentV2) {
    let mut fixture_ledger = ContractLedger::default();
    let mut fixture_control = Caller::default();
    document.items[0].context_sha256 =
        hash_v2::context(document, &document.items[0], &mut fixture_ledger, &mut fixture_control)
            .unwrap();
    document.items[0].policy_sha256 = hash_v2::policy(
        document,
        &document.policies[0],
        &document.items[0],
        &mut fixture_ledger,
        &mut fixture_control,
    )
    .unwrap();
    document.items[0].item_id =
        hash_v2::item_id(document, &document.items[0], &mut fixture_ledger, &mut fixture_control)
            .unwrap();
}
/// Exact private immutable response fixture to an ordinary typed queue's actual raw bytes.
fn response_fixture(document: &QueueDocumentV2, queue_raw: &[u8]) -> ResponseDocumentV2 {
    let item = &document.items[0];
    ResponseDocumentV2 {
        schema_version: "forge.review-response/2".into(),
        identity_disclaimer: IDENTITY_DISCLAIMER.into(),
        response_id: "55555555-5555-4555-8555-555555555555".into(),
        queue_id: document.queue_id.clone(),
        queue_raw_sha256: crate::hashing::sha256_hex(queue_raw),
        item_key: item.key.clone(),
        item_id: item.item_id.clone(),
        domain: item.domain,
        adapter_version: item.adapter_version.clone(),
        requested_action: item.requested_action,
        source_pins: serde_json::from_value(serde_json::to_value(&document.source_pins).unwrap())
            .unwrap(),
        subject_sha256: item.subject_sha256.clone(),
        context_sha256: item.context_sha256.clone(),
        policy_sha256: item.policy_sha256.clone(),
        reviewer_key: "reviewer".into(),
        reviewer_role: "review".into(),
        disposition: Disposition::Approve,
        responded_at: "2026-10-05T00:00:01Z".into(),
        rationale: "  Private α\n\"quoted\"\\path text  ".into(),
        abstention_reason: None,
        proposed_edit: (),
        supersedes: None,
    }
}
/// Ordinary fixture serialization, not the bounded production path under qualification.
fn raw(value: &impl serde::Serialize) -> Vec<u8> {
    let mut raw = serde_json::to_vec(value).unwrap();
    raw.push(b'\n');
    raw
}

/// Both real typed finite codecs preserve exact nulls, original spelling and private rationale.
#[test]
fn queue_response_full_finite_roundtrip_and_exact_raw_queue_binding() {
    let queue_doc = queue_fixture();
    let mut ledger = ContractLedger::default();
    let mut caller = Caller::default();
    let queue_raw = queue(&queue_doc, &mut ledger, &mut caller).unwrap();
    assert_eq!(queue_raw, raw(&queue_doc));
    assert!(queue_raw.len() <= MAX_QUEUE_BYTES);
    let decoded_queue = decode_v2::decode_queue(&queue_raw, &mut ledger, &mut caller).unwrap();
    let response_doc = response_fixture(&queue_doc, &queue_raw);
    let response_raw = response(&response_doc, &mut ledger, &mut caller).unwrap();
    assert_eq!(response_raw, raw(&response_doc));
    assert!(response_raw.len() <= MAX_RESPONSE_BYTES);
    let decoded_response =
        decode_v2::decode_response(&response_raw, &mut ledger, &mut caller).unwrap();
    assert!(decoded_queue.document() == &queue_doc);
    assert!(decoded_response.document() == &response_doc);
    decode_v2::bind_response(&decoded_queue, &decoded_response, &mut ledger, &mut caller).unwrap();
    let queue_value: serde_json::Value = serde_json::from_slice(&queue_raw).unwrap();
    assert!(queue_value["items"][0]["due_at"].is_null());
    assert_eq!(
        queue_value["source_pins"][0]["native_root_uuid"],
        "{ABCDEFAB-1234-5567-89AB-CDEF01234567}"
    );
    let response_value: serde_json::Value = serde_json::from_slice(&response_raw).unwrap();
    for field in ["abstention_reason", "supersedes", "proposed_edit"] {
        assert!(response_value[field].is_null());
    }
    assert_eq!(decoded_response.document().rationale, response_doc.rationale);
    assert_eq!(queue(&queue_doc, &mut ledger, &mut caller).unwrap(), queue_raw);
    assert_eq!(response(&response_doc, &mut ledger, &mut caller).unwrap(), response_raw);
}
/// Change coherent ordinary queue declarations before genuine full hash refresh.
fn change_queue(document: &mut QueueDocumentV2, case: usize) {
    match case {
        0 => document.queue_id = "77777777-7777-4777-8777-777777777777".into(),
        1 => document.created_at = "2026-10-05T00:00:01Z".into(),
        2 => document.source_pins[0].raw_sha256 = "a".repeat(64),
        3 => document.source_pins[0].byte_length += 1,
        4 => document.source_pins[0].native_model = Some(NativeModelV2::Profile),
        5 => {
            document.source_pins[0].native_root_uuid =
                Some("abcdefab-1234-5567-89ab-cdef01234567".into());
        }
        6 => document.items[0].subject_id = format!("lifecycle:{}", "a".repeat(64)),
        7 => document.items[0].subject_sha256 = "a".repeat(64),
        8 => document.items[0].key = "another-item".into(),
        9 => document.items[0].due_at = Some("2026-10-06T00:00:00Z".into()),
        10 => {
            document.policies[0].key = "another-policy".into();
            document.items[0].policy_key = "another-policy".into();
        }
        11 => document.policies[0].seats[0].count = 2,
        12 => document.policies[0].empty_abstention_reasons = vec!["another-reason".into()],
        13 => document.items[0].assignments.clear(),
        14 => document.policies[0].substitutions.push(Substitution {
            seat_role: "review".into(),
            reviewer_key: "reviewer".into(),
            asserted_role: "review".into(),
            reason_code: "explicit".into(),
        }),
        15 => {
            document.reviewers.insert(
                1,
                Reviewer { key: "other-author".into(), role_keys: vec!["review".into()] },
            );
            document.items[0].author_keys = vec!["other-author".into()];
        }
        _ => panic!("unknown queue fixture mutation"),
    }
    refresh(document);
}
/// Fully strict-admitted coherent queue differences cannot escape the complete equality oracle.
#[test]
fn queue_full_readback_refuses_all_coherent_registry_policy_item_and_pin_changes() {
    let expected = queue_fixture();
    for case in 0..16 {
        let mut changed = queue_fixture();
        change_queue(&mut changed, case);
        let bytes = raw(&changed);
        let mut ledger = ContractLedger::default();
        let mut caller = Caller::default();
        let loaded = decode_v2::decode_queue(&bytes, &mut ledger, &mut caller).unwrap();
        assert!(loaded.document() == &changed && loaded.document() != &expected, "case {case}");
        assert_eq!(
            readback_queue(&expected, &bytes, &mut ledger, &mut caller),
            Err(ContractError::Binding),
            "case {case}"
        );
        assert!(
            queue(&expected, &mut ledger, &mut caller).is_ok(),
            "ordinary refusal latched in case {case}"
        );
    }
}
/// Change coherent standalone closed response evidence; exact queue binding is a separate port.
fn change_response(document: &mut ResponseDocumentV2, case: usize) {
    match case {
        0 => document.response_id = "77777777-7777-4777-8777-777777777777".into(),
        1 => document.queue_id = "77777777-7777-4777-8777-777777777777".into(),
        2 => document.queue_raw_sha256 = "a".repeat(64),
        3 => document.item_key = "another-item".into(),
        4 => document.item_id = "77777777-7777-4777-8777-777777777777".into(),
        5 => document.source_pins[0].raw_sha256 = "a".repeat(64),
        6 => {
            document.source_pins[0].native_root_uuid =
                Some("abcdefab-1234-5567-89ab-cdef01234567".into());
        }
        7 => document.subject_sha256 = "a".repeat(64),
        8 => document.context_sha256 = "a".repeat(64),
        9 => document.policy_sha256 = "a".repeat(64),
        10 => document.reviewer_key = "another-reviewer".into(),
        11 => document.reviewer_role = "another-role".into(),
        12 => document.responded_at = "2026-10-05T00:00:02Z".into(),
        13 => document.rationale.push_str(" exact added text"),
        14 => document.disposition = Disposition::RequestChanges,
        15 => {
            document.disposition = Disposition::Abstain;
            document.abstention_reason = Some("no-conflict".into());
        }
        16 => {
            document.disposition = Disposition::Superseded;
            document.supersedes = Some(SupersessionReference {
                response_id: "77777777-7777-4777-8777-777777777777".into(),
                raw_sha256: "a".repeat(64),
            });
        }
        _ => panic!("unknown response fixture mutation"),
    }
}
/// Complete real response decoder equality retains private prose, options and exact declared pins.
#[test]
fn response_full_readback_refuses_coherent_private_nullable_and_binding_changes() {
    let queue_doc = queue_fixture();
    let queue_raw = raw(&queue_doc);
    let expected = response_fixture(&queue_doc, &queue_raw);
    for case in 0..17 {
        let mut changed = response_fixture(&queue_doc, &queue_raw);
        change_response(&mut changed, case);
        let bytes = raw(&changed);
        let mut ledger = ContractLedger::default();
        let mut caller = Caller::default();
        let loaded = decode_v2::decode_response(&bytes, &mut ledger, &mut caller).unwrap();
        assert!(loaded.document() == &changed && loaded.document() != &expected, "case {case}");
        assert_eq!(
            readback_response(&expected, &bytes, &mut ledger, &mut caller),
            Err(ContractError::Binding),
            "case {case}"
        );
        assert!(
            response(&expected, &mut ledger, &mut caller).is_ok(),
            "ordinary refusal latched in case {case}"
        );
    }
}
/// Required nulls/closed shape/supplied array order and old envelope markers are never repaired.
#[test]
fn queue_response_strict_shape_null_and_order_refusal_is_ordinary() {
    let queue_doc = queue_fixture();
    let queue_raw = raw(&queue_doc);
    let response_doc = response_fixture(&queue_doc, &queue_raw);
    let mut ledger = ContractLedger::default();
    let mut caller = Caller::default();
    let mut missing = serde_json::to_value(&queue_doc).unwrap();
    missing["items"][0].as_object_mut().unwrap().remove("due_at");
    assert_eq!(
        readback_queue(&queue_doc, &raw(&missing), &mut ledger, &mut caller),
        Err(ContractError::Invalid)
    );
    let mut unsorted = queue_fixture();
    unsorted.reviewers.swap(0, 1);
    assert_eq!(queue(&unsorted, &mut ledger, &mut caller), Err(ContractError::Invalid));
    for field in ["abstention_reason", "supersedes", "proposed_edit"] {
        let mut missing = serde_json::to_value(&response_doc).unwrap();
        missing.as_object_mut().unwrap().remove(field);
        assert_eq!(
            readback_response(&response_doc, &raw(&missing), &mut ledger, &mut caller),
            Err(ContractError::Invalid)
        );
    }
    let mut foreign = serde_json::to_value(&response_doc).unwrap();
    foreign["schema_version"] = serde_json::json!("forge.review-response/1");
    assert_eq!(
        readback_response(&response_doc, &raw(&foreign), &mut ledger, &mut caller),
        Err(ContractError::Invalid)
    );
    let mut extra = serde_json::to_value(&queue_doc).unwrap();
    extra["path"] = serde_json::json!("private.json");
    assert_eq!(
        readback_queue(&queue_doc, &raw(&extra), &mut ledger, &mut caller),
        Err(ContractError::Invalid)
    );
    assert!(queue(&queue_doc, &mut ledger, &mut caller).is_ok());
    assert!(response(&response_doc, &mut ledger, &mut caller).is_ok());
}
/// Real typed queue tokens reach the private finite writer boundary; public cap never changes.
#[test]
fn queue_response_writer_exact_limit_includes_lf_and_keeps_first_capacity() {
    let queue_doc = queue_fixture();
    let queue_raw = raw(&queue_doc);
    let response_doc = response_fixture(&queue_doc, &queue_raw);
    for is_response in [false, true] {
        let mut ledger = ContractLedger::default();
        let mut caller = Caller::default();
        let (first, refused) = if is_response {
            let expected = raw(&response_doc);
            let first = encoded(&response_doc, expected.len(), &mut ledger, &mut caller).unwrap();
            let refused = encoded(&response_doc, expected.len() - 1, &mut ledger, &mut caller);
            (first, refused)
        } else {
            let first = encoded(&queue_doc, queue_raw.len(), &mut ledger, &mut caller).unwrap();
            let refused = encoded(&queue_doc, queue_raw.len() - 1, &mut ledger, &mut caller);
            (first, refused)
        };
        assert!(first.ends_with(b"\n"));
        assert_eq!(refused, Err(ContractError::Capacity));
        let before = caller.calls;
        caller.at = Some(before + 1);
        caller.failed = true;
        assert_eq!(queue(&queue_doc, &mut ledger, &mut caller), Err(ContractError::Capacity));
        assert_eq!(response(&response_doc, &mut ledger, &mut caller), Err(ContractError::Capacity));
        assert_eq!(caller.calls, before);
    }
}
/// The completed actual queue encode/readback's final same-controller stop keeps no output.
#[test]
fn queue_final_complete_readback_stop_is_sticky_without_renewal() {
    let document = queue_fixture();
    for failed in [false, true] {
        let mut ledger = ContractLedger::default();
        let mut caller = Caller::default();
        queue(&document, &mut ledger, &mut caller).unwrap();
        let cost = caller.calls;
        caller.at = Some(caller.calls + cost);
        caller.failed = failed;
        assert_eq!(queue(&document, &mut ledger, &mut caller), Err(stop(failed)));
        let before = caller.calls;
        assert_eq!(
            readback_queue(&document, &raw(&document), &mut ledger, &mut caller),
            Err(stop(failed))
        );
        assert_eq!(caller.calls, before);
    }
}
/// The completed private response's final same-controller stop cannot become emitted success.
#[test]
fn response_final_complete_readback_stop_is_sticky_without_renewal() {
    let queue_doc = queue_fixture();
    let document = response_fixture(&queue_doc, &raw(&queue_doc));
    for failed in [false, true] {
        let mut ledger = ContractLedger::default();
        let mut caller = Caller::default();
        response(&document, &mut ledger, &mut caller).unwrap();
        let cost = caller.calls;
        caller.at = Some(caller.calls + cost);
        caller.failed = failed;
        assert_eq!(response(&document, &mut ledger, &mut caller), Err(stop(failed)));
        let before = caller.calls;
        assert_eq!(
            readback_response(&document, &raw(&document), &mut ledger, &mut caller),
            Err(stop(failed))
        );
        assert_eq!(caller.calls, before);
    }
}
/// Ordinary typed queue failure is postfenced; a later valid codec shares the same ledger/controller.
#[test]
fn queue_ordinary_failure_has_original_postfence_and_continuation() {
    let document = queue_fixture();
    let mut invalid = queue_fixture();
    invalid.schema_version = "forge.review-queue/1".into();
    for failed in [false, true] {
        let mut ledger = ContractLedger::default();
        let mut caller = Caller::default();
        assert_eq!(queue(&invalid, &mut ledger, &mut caller), Err(ContractError::Invalid));
        let cost = caller.calls;
        caller.at = Some(caller.calls + cost);
        caller.failed = failed;
        assert_eq!(queue(&invalid, &mut ledger, &mut caller), Err(stop(failed)));
        let before = caller.calls;
        assert_eq!(queue(&document, &mut ledger, &mut caller), Err(stop(failed)));
        assert_eq!(caller.calls, before);
    }
    let mut ledger = ContractLedger::default();
    let mut caller = Caller::default();
    assert_eq!(queue(&invalid, &mut ledger, &mut caller), Err(ContractError::Invalid));
    assert!(queue(&document, &mut ledger, &mut caller).is_ok());
}
/// Ordinary response schema/readback failure is postfenced on the original control even on error.
#[test]
fn response_ordinary_readback_failure_has_original_postfence_and_continuation() {
    let queue_doc = queue_fixture();
    let document = response_fixture(&queue_doc, &raw(&queue_doc));
    for failed in [false, true] {
        let mut ledger = ContractLedger::default();
        let mut caller = Caller::default();
        assert_eq!(
            readback_response(&document, b"{}", &mut ledger, &mut caller),
            Err(ContractError::Invalid)
        );
        let cost = caller.calls;
        caller.at = Some(caller.calls + cost);
        caller.failed = failed;
        assert_eq!(
            readback_response(&document, b"{}", &mut ledger, &mut caller),
            Err(stop(failed))
        );
        let before = caller.calls;
        assert_eq!(response(&document, &mut ledger, &mut caller), Err(stop(failed)));
        assert_eq!(caller.calls, before);
    }
    let mut ledger = ContractLedger::default();
    let mut caller = Caller::default();
    assert_eq!(
        readback_response(&document, b"{}", &mut ledger, &mut caller),
        Err(ContractError::Invalid)
    );
    assert!(response(&document, &mut ledger, &mut caller).is_ok());
}
/// Previous shared logical reservations remain spent across both new typed ports.
#[test]
fn queue_response_both_observe_existing_shared_capacity_stop() {
    let queue_doc = queue_fixture();
    let response_doc = response_fixture(&queue_doc, &raw(&queue_doc));
    let mut ledger = ContractLedger::default();
    ledger.derived(33_554_432 - 7).unwrap();
    let mut caller = Caller::default();
    assert_eq!(queue(&queue_doc, &mut ledger, &mut caller), Err(ContractError::Capacity));
    let before = caller.calls;
    caller.at = Some(before + 1);
    caller.failed = true;
    assert_eq!(response(&response_doc, &mut ledger, &mut caller), Err(ContractError::Capacity));
    assert_eq!(queue(&queue_doc, &mut ledger, &mut caller), Err(ContractError::Capacity));
    assert_eq!(caller.calls, before);
}
