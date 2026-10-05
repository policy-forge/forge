//! Genuine component controls execute real strict decoders/framing over inert fixtures.
//! Synthetic vector operands are explicitly not native records, leases or approvals.
use super::{decode, decode_v2, hash_v2, wire, wire_v2};
use crate::workspace::preparation::{Interruption, ProgressUpdate, Stage, WorkControl, WorkError};
use decode::{ContractError, ContractLedger, OriginalRelation};
use serde_json::{Value, json};
use wire_v2::*;

/// One original component caller, with independently selected actual callback failures.
#[derive(Default)]
struct Caller {
    calls: usize,
    stop_at: Option<usize>,
    failed: bool,
    stopped: Option<Interruption>,
}
impl WorkControl for Caller {
    /// Count genuine consumer checkpoints; no fake native owner or clock is supplied.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> Result<(), WorkError> {
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
    /// Preserve the actual interruption reported by this same caller.
    fn interruption(&self) -> Option<Interruption> {
        self.stopped
    }
}
/// Load frozen literal vector data, never a real native lifecycle corpus.
fn vectors() -> Value {
    serde_json::from_str(include_str!("../../tests/fixtures/review/lifecycle-hash-vectors.json"))
        .unwrap()
}
/// Exact three ordinary pin declarations from the frozen data contract.
fn pins() -> Vec<SourcePinV2> {
    let mut pins: Vec<SourcePinV2> =
        serde_json::from_value(vectors()["synthetic_roster_pins_native_order"].clone()).unwrap();
    pins.sort_by(|a, b| a.artifact_key.cmp(&b.artifact_key));
    pins
}
/// Build complete inert queue data; ordinary framing is real, native authority is absent.
fn queue_fixture() -> QueueDocumentV2 {
    let source_pins = pins();
    let mut queue = QueueDocumentV2 {
        schema_version: "forge.review-queue/2".into(),
        identity_disclaimer: IDENTITY_DISCLAIMER.into(),
        sensitivity: Sensitivity::IdsAndHashes,
        queue_id: "10000000-0000-4000-8000-000000000001".into(),
        created_at: "2026-10-05T00:00:00Z".into(),
        roles: vec![RoleDefinition { key: "reviewer".into() }],
        reviewers: vec![
            Reviewer { key: "author-1".into(), role_keys: vec!["reviewer".into()] },
            Reviewer { key: "reviewer-1".into(), role_keys: vec!["reviewer".into()] },
        ],
        policies: vec![ReviewPolicy {
            key: "policy-1".into(),
            seats: vec![wire::SeatRequirement { role_key: "reviewer".into(), count: 1 }],
            substitutions: vec![],
            abstention_rule: AbstentionRule::Nonapproving,
            empty_abstention_reasons: vec!["absent".into()],
            author_separation: AuthorSeparation::DeclaredKeys,
        }],
        items: vec![ReviewItemV2 {
            key: "item-1".into(),
            item_id: String::new(),
            domain: DomainV2::LifecyclePolicyVersion,
            adapter_version: ADAPTER.into(),
            subject_id:
                "lifecycle:f9b711b5fc7336a01fd72284be01982df765d49077c61dbb2eeeea012fd54937".into(),
            requested_action: RequestedAction::ReReview,
            source_keys: source_pins.iter().map(|p| p.artifact_key.clone()).collect(),
            subject_sha256: "104e3188fd2c8168ddb4b3f555689c722364f0ac69de675e3f60fb49756c796f"
                .into(),
            context: ContextSnapshot {
                reason_codes: vec!["lifecycle-approved-current".into()],
                related_subject_ids: vec![],
            },
            context_sha256: String::new(),
            policy_key: "policy-1".into(),
            policy_sha256: String::new(),
            author_keys: vec!["author-1".into()],
            assignments: vec![Assignment {
                reviewer_key: "reviewer-1".into(),
                role_key: "reviewer".into(),
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
    refresh(&mut queue);
    queue
}
/// Use actual framing for a changed ordinary fixture before serializing it as raw input.
fn refresh(queue: &mut QueueDocumentV2) {
    let mut ledger = ContractLedger::default();
    let mut caller = Caller::default();
    let context = hash_v2::context(queue, &queue.items[0], &mut ledger, &mut caller).unwrap();
    queue.items[0].context_sha256 = context;
    let policy =
        hash_v2::policy(queue, &queue.policies[0], &queue.items[0], &mut ledger, &mut caller)
            .unwrap();
    queue.items[0].policy_sha256 = policy;
    let id = hash_v2::item_id(queue, &queue.items[0], &mut ledger, &mut caller).unwrap();
    queue.items[0].item_id = id;
}
/// Serialize an ordinary cfg fixture once; actual decoder retains these exact bytes.
fn raw(value: &impl serde::Serialize) -> Vec<u8> {
    let mut raw = serde_json::to_vec(value).unwrap();
    raw.push(b'\n');
    raw
}
/// Exact asserted response fixture bound to real decoder observations.
fn response_fixture(queue: &decode_v2::DecodedV2<'_, QueueDocumentV2>) -> Value {
    let doc = queue.document();
    let item = &doc.items[0];
    json!({"schema_version":"forge.review-response/2","identity_disclaimer":IDENTITY_DISCLAIMER,
        "response_id":"20000000-0000-4000-8000-000000000002","queue_id":doc.queue_id,"queue_raw_sha256":queue.raw_sha256(),
        "item_key":item.key,"item_id":item.item_id,"domain":"lifecycle-policy-version","adapter_version":ADAPTER,
        "requested_action":"re-review","source_pins":doc.source_pins,"subject_sha256":item.subject_sha256,
        "context_sha256":item.context_sha256,"policy_sha256":item.policy_sha256,"reviewer_key":"reviewer-1",
        "reviewer_role":"reviewer","disposition":"approve","responded_at":"2026-10-05T00:00:01Z",
        "rationale":"private immutable explanation","abstention_reason":null,"proposed_edit":null,"supersedes":null})
}

/// All stable private-key vector digests use the actual binary profile, including UTF8/NUL.
#[test]
fn complete_native_key_pair_vectors_and_private_byte_bound() {
    for vector in vectors()["fingerprint_vectors"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["profile"] == "subject-id")
    {
        let mut ledger = ContractLedger::default();
        let mut caller = Caller::default();
        let id = hash_v2::subject_id(
            vector["private_policy_key"].as_str().unwrap(),
            vector["private_version_key"].as_str().unwrap(),
            &mut ledger,
            &mut caller,
        )
        .unwrap();
        assert_eq!(id, vector["subject_id"].as_str().unwrap());
    }
    let mut ledger = ContractLedger::default();
    let mut caller = Caller::default();
    assert!(hash_v2::subject_id(&"x".repeat(4096), "v1", &mut ledger, &mut caller).is_ok());
    assert_eq!(
        hash_v2::subject_id(&"x".repeat(4097), "v1", &mut ledger, &mut caller).err(),
        Some(ContractError::Invalid)
    );
}
/// Full sources/subject/context/policy/item/generation vectors preserve all order and options.
#[test]
fn complete_all_remaining_framing_vectors() {
    let queue = queue_fixture();
    let data = vectors();
    let values = data["fingerprint_vectors"].as_array().unwrap();
    let expected = |name: &str| {
        values.iter().find(|v| v["profile"] == name).unwrap()["sha256"].as_str().unwrap()
    };
    let native: Vec<SourcePinV2> =
        serde_json::from_value(data["synthetic_roster_pins_native_order"].clone()).unwrap();
    let rows = [
        hash_v2::RosterEntry {
            purpose: SourceKindV2::LifecycleRecord,
            route: None,
            pin: &native[0],
        },
        hash_v2::RosterEntry {
            purpose: SourceKindV2::OpaqueSource,
            route: Some("source.md"),
            pin: &native[1],
        },
        hash_v2::RosterEntry {
            purpose: SourceKindV2::GeneratedArtifact,
            route: Some("artifact.json"),
            pin: &native[2],
        },
    ];
    let mut ledger = ContractLedger::default();
    let mut caller = Caller::default();
    let sources = hash_v2::sources(&rows, &mut ledger, &mut caller).unwrap();
    assert_eq!(sources, expected("sources"));
    assert_eq!(
        hash_v2::subject(" Policy α ", "v1", &sources, &mut ledger, &mut caller).unwrap(),
        expected("subject")
    );
    assert_eq!(queue.items[0].context_sha256, expected("context"));
    assert_eq!(queue.items[0].policy_sha256, expected("policy"));
    let item = values.iter().find(|v| v["profile"] == "item").unwrap();
    assert_eq!(queue.items[0].item_id, item["uuid_v5_result"].as_str().unwrap());
    let response = [hash_v2::ResponseOriginal {
        response_id: "20000000-0000-4000-8000-000000000002",
        raw_sha256: "5eafda03665e59c3b01c69b93c826b2314475518a5405a1d0302f7c2396dc3fb",
        byte_length: 25,
    }];
    let generation = hash_v2::closure_generation(
        "9b356dc951f1c85c42f25cfe7bb24438d119f30049400d1131e75736735b1405",
        22,
        &queue.source_pins,
        &response,
        &mut ledger,
        &mut caller,
    )
    .unwrap();
    assert_eq!(generation, expected("closure-generation"));
}
/// Native UUID original text accepts all maintained spellings without imposing review IDs.
#[test]
fn native_uuid_spellings_preserve_original_and_review_uuid_remains_canonical() {
    for original in [
        "00000000-0000-0000-0000-000000000000",
        "ABCDEFAB-1234-5678-9ABC-ABCDEFABCDEF",
        "abcdefab123456789abcabcdefabcdef",
        "{ABCDEFAB-1234-5678-9ABC-ABCDEFABCDEF}",
        "urn:uuid:abcdefab-1234-5678-9abc-abcdefabcdef",
    ] {
        let mut queue = queue_fixture();
        queue.source_pins[0].native_root_uuid = Some(original.into());
        refresh(&mut queue);
        let bytes = raw(&queue);
        let mut ledger = ContractLedger::default();
        let mut caller = Caller::default();
        let decoded = decode_v2::decode_queue(&bytes, &mut ledger, &mut caller).unwrap();
        assert_eq!(decoded.document().source_pins[0].native_root_uuid.as_deref(), Some(original));
    }
    let mut queue = serde_json::to_value(queue_fixture()).unwrap();
    queue["queue_id"] = json!("00000000-0000-0000-0000-000000000000");
    assert_eq!(
        decode_v2::decode_queue(
            &raw(&queue),
            &mut ContractLedger::default(),
            &mut Caller::default()
        )
        .err(),
        Some(ContractError::Invalid)
    );
}
/// All explicit nullable pin/response/item/bundle fields must remain present.
#[test]
fn omitted_required_nulls_and_unknown_fields_refuse() {
    for field in ["schema_identity", "native_model", "native_root_uuid"] {
        let mut doc = serde_json::to_value(queue_fixture()).unwrap();
        doc["source_pins"][1].as_object_mut().unwrap().remove(field);
        assert_eq!(
            decode_v2::decode_queue(
                &raw(&doc),
                &mut ContractLedger::default(),
                &mut Caller::default()
            )
            .err(),
            Some(ContractError::Invalid)
        );
    }
    let mut doc = serde_json::to_value(queue_fixture()).unwrap();
    doc["items"][0].as_object_mut().unwrap().remove("due_at");
    assert_eq!(
        decode_v2::decode_queue(&raw(&doc), &mut ContractLedger::default(), &mut Caller::default())
            .err(),
        Some(ContractError::Invalid)
    );
    let mut doc = serde_json::to_value(queue_fixture()).unwrap();
    doc["source_pins"][0]["path"] = json!("private.json");
    assert_eq!(
        decode_v2::decode_queue(&raw(&doc), &mut ContractLedger::default(), &mut Caller::default())
            .err(),
        Some(ContractError::Invalid)
    );
}
/// /1 and /2 actual decoders reject cross-family and marker-only conversions.
#[test]
fn actual_closed_family_dispatch_without_v1_widening() {
    let bytes = raw(&queue_fixture());
    assert_eq!(
        decode::decode_queue(&bytes, &mut ContractLedger::default(), &mut Caller::default()).err(),
        Some(ContractError::Invalid)
    );
    let mut doc = serde_json::to_value(queue_fixture()).unwrap();
    doc["schema_version"] = json!("forge.review-queue/1");
    let changed = raw(&doc);
    assert_eq!(
        decode_v2::decode_queue(&changed, &mut ContractLedger::default(), &mut Caller::default())
            .err(),
        Some(ContractError::Invalid)
    );
    assert_eq!(
        decode::decode_queue(&changed, &mut ContractLedger::default(), &mut Caller::default())
            .err(),
        Some(ContractError::Invalid)
    );
}
/// Complete generated ordinal gaps and source-key subsets refuse even if array lengths fit.
#[test]
fn complete_pin_role_ordinal_and_item_source_roster() {
    for variant in 0..4 {
        let mut doc = serde_json::to_value(queue_fixture()).unwrap();
        match variant {
            0 => doc["source_pins"][0]["artifact_key"] = json!("lifecycle:generated:1"),
            1 => doc["source_pins"][1]["validation_profile"] = json!("forge.opaque-source-bytes/1"),
            2 => doc["items"][0]["source_keys"] = json!(["lifecycle:record", "lifecycle:source"]),
            _ => doc["source_pins"][0]["schema_identity"] = json!("forge.policy-lifecycle/2"),
        }
        assert_eq!(
            decode_v2::decode_queue(
                &raw(&doc),
                &mut ContractLedger::default(),
                &mut Caller::default()
            )
            .err(),
            Some(ContractError::Invalid)
        );
    }
}
/// Public key order differs from native generated ordinal order and is checked completely.
#[test]
fn complete_lexical_pin_order_cannot_be_replaced_by_ordinal_order() {
    let mut queue = queue_fixture();
    let mut pins = Vec::new();
    for n in 0..12 {
        let mut pin: SourcePinV2 =
            serde_json::from_value(serde_json::to_value(&queue.source_pins[0]).unwrap()).unwrap();
        pin.artifact_key = format!("lifecycle:generated:{n}");
        pins.push(pin);
    }
    pins.push(
        serde_json::from_value(serde_json::to_value(&queue.source_pins[1]).unwrap()).unwrap(),
    );
    pins.push(
        serde_json::from_value(serde_json::to_value(&queue.source_pins[2]).unwrap()).unwrap(),
    );
    queue.source_pins = pins;
    assert_eq!(
        decode_v2::validate_pins(&queue.source_pins, &mut ContractLedger::default()).err(),
        Some(ContractError::Invalid)
    );
    queue.source_pins.sort_by(|a, b| a.artifact_key.cmp(&b.artifact_key));
    assert!(decode_v2::validate_pins(&queue.source_pins, &mut ContractLedger::default()).is_ok());
}
/// Valid empty opaque bytes remain distinct from forbidden empty record/generated claims.
#[test]
fn empty_source_and_complete_occurrence_extent_bound() {
    let queue = queue_fixture();
    let bytes = raw(&queue);
    assert!(
        decode_v2::decode_queue(&bytes, &mut ContractLedger::default(), &mut Caller::default())
            .is_ok()
    );
    for index in [0, 1] {
        let mut doc = serde_json::to_value(&queue).unwrap();
        doc["source_pins"][index]["byte_length"] = json!(0);
        assert_eq!(
            decode_v2::decode_queue(
                &raw(&doc),
                &mut ContractLedger::default(),
                &mut Caller::default()
            )
            .err(),
            Some(ContractError::Invalid)
        );
    }
    let mut many = pins();
    for ordinal in 1..6 {
        let mut pin: SourcePinV2 =
            serde_json::from_value(serde_json::to_value(&many[0]).unwrap()).unwrap();
        pin.artifact_key = format!("lifecycle:generated:{ordinal}");
        pin.byte_length = 10_485_760;
        many.push(pin);
    }
    many.sort_by(|a, b| a.artifact_key.cmp(&b.artifact_key));
    assert_eq!(
        decode_v2::validate_pins(&many, &mut ContractLedger::default()).err(),
        Some(ContractError::Capacity)
    );
}
/// Actual raw-byte changes refuse old response transfer despite stable ordinary subject/item data.
#[test]
fn complete_raw_queue_and_response_bindings() {
    let mut ledger = ContractLedger::default();
    let mut caller = Caller::default();
    let bytes = raw(&queue_fixture());
    let queue = decode_v2::decode_queue(&bytes, &mut ledger, &mut caller).unwrap();
    let response_raw = raw(&response_fixture(&queue));
    let response = decode_v2::decode_response(&response_raw, &mut ledger, &mut caller).unwrap();
    decode_v2::bind_response(&queue, &response, &mut ledger, &mut caller).unwrap();
    let mut changed = b" ".to_vec();
    changed.extend_from_slice(&bytes);
    let other = decode_v2::decode_queue(&changed, &mut ledger, &mut caller).unwrap();
    assert_eq!(
        decode_v2::bind_response(&other, &response, &mut ledger, &mut caller).err(),
        Some(ContractError::Binding)
    );
    let mut altered = response_fixture(&queue);
    altered["source_pins"][1]["raw_sha256"] = json!("0".repeat(64));
    let altered_raw = raw(&altered);
    let altered = decode_v2::decode_response(&altered_raw, &mut ledger, &mut caller).unwrap();
    assert_eq!(
        decode_v2::bind_response(&queue, &altered, &mut ledger, &mut caller).err(),
        Some(ContractError::Binding)
    );
}
/// Complete original-byte identity distinguishes exact duplicates from semantic whitespace equality.
#[test]
fn actual_response_identity_relation_keeps_full_bytes() {
    let mut ledger = ContractLedger::default();
    let mut caller = Caller::default();
    let queue_raw = raw(&queue_fixture());
    let queue = decode_v2::decode_queue(&queue_raw, &mut ledger, &mut caller).unwrap();
    let bytes = raw(&response_fixture(&queue));
    let left = decode_v2::decode_response(&bytes, &mut ledger, &mut caller).unwrap();
    let same = decode_v2::decode_response(&bytes, &mut ledger, &mut caller).unwrap();
    assert_eq!(
        decode_v2::original_relation(&left, &same, &mut ledger, &mut caller).unwrap(),
        OriginalRelation::ExactDuplicate
    );
    let mut other = b" ".to_vec();
    other.extend_from_slice(&bytes);
    let other = decode_v2::decode_response(&other, &mut ledger, &mut caller).unwrap();
    assert_eq!(
        decode_v2::original_relation(&left, &other, &mut ledger, &mut caller).unwrap(),
        OriginalRelation::IdentityConflict
    );
}
/// Genuine mixed /1-/2 malformed registrations reach the shared 10,000 original counter.
#[test]
fn mixed_family_original_response_counter_is_not_renewed() {
    let mut ledger = ContractLedger::default();
    let mut caller = Caller::default();
    for n in 0..10_000 {
        let result = if n % 2 == 0 {
            decode::decode_response(b"{", &mut ledger, &mut caller).err()
        } else {
            decode_v2::decode_response(b"{", &mut ledger, &mut caller).err()
        };
        assert_eq!(result, Some(ContractError::Invalid));
    }
    assert_eq!(
        decode_v2::decode_response(b"{", &mut ledger, &mut caller).err(),
        Some(ContractError::Capacity)
    );
    let calls = caller.calls;
    caller.stop_at = Some(calls + 1);
    assert_eq!(
        decode::decode_response(b"{", &mut ledger, &mut caller).err(),
        Some(ContractError::Capacity)
    );
    assert_eq!(caller.calls, calls);
}
/// Genuine mixed malformed UTF8 registrations share the exact32-MiB aggregate byte ceiling.
#[test]
fn mixed_family_response_raw_byte_counter_and_capacity_first() {
    let raw = vec![0xff; 1_048_576];
    let mut ledger = ContractLedger::default();
    let mut caller = Caller::default();
    for n in 0..32 {
        let result = if n % 2 == 0 {
            decode::decode_response(&raw, &mut ledger, &mut caller).err()
        } else {
            decode_v2::decode_response(&raw, &mut ledger, &mut caller).err()
        };
        assert_eq!(result, Some(ContractError::Invalid));
    }
    assert_eq!(
        decode_v2::decode_response(b"x", &mut ledger, &mut caller).err(),
        Some(ContractError::Capacity)
    );
    let calls = caller.calls;
    caller.stop_at = Some(calls + 1);
    caller.failed = true;
    assert_eq!(
        decode_v2::decode_queue(b"{", &mut ledger, &mut caller).err(),
        Some(ContractError::Capacity)
    );
    assert_eq!(caller.calls, calls);
}
/// Last genuine post-ordinary callback preserves interruption/control failure on the same ledger.
#[test]
fn ordinary_strict_parser_and_schema_failure_keep_original_postfence() {
    for malformed in [br#"{"x":1,"x":2}"#.as_slice(), b"{}".as_slice()] {
        for failed in [false, true] {
            let mut ledger = ContractLedger::default();
            let mut caller = Caller::default();
            assert_eq!(
                decode_v2::decode_queue(malformed, &mut ledger, &mut caller).err(),
                Some(ContractError::Invalid)
            );
            let count = caller.calls;
            caller.stop_at = Some(caller.calls + count);
            caller.failed = failed;
            let result = decode_v2::decode_queue(malformed, &mut ledger, &mut caller).err();
            assert_eq!(
                result,
                Some(if failed {
                    ContractError::ControlFailed
                } else {
                    ContractError::Interrupted(Interruption::CancelRequested)
                })
            );
            let calls = caller.calls;
            assert_eq!(hash_v2::subject_id("a", "b", &mut ledger, &mut caller).err(), result);
            assert_eq!(caller.calls, calls);
        }
    }
}
/// Strict duplicate/depth/string/raw boundaries use actual decoder phases, never a parser stand-in.
#[test]
fn strict_duplicate_depth_string_and_raw_capacity() {
    for bytes in [
        b"{\"schema_version\":\"a\",\"schema_version\":\"b\"}".to_vec(),
        format!("{}0{}", "[".repeat(66), "]".repeat(66)).into_bytes(),
        format!("{{\"x\":\"{}\"}}", "x".repeat(65_537)).into_bytes(),
    ] {
        assert_eq!(
            decode_v2::decode_queue(&bytes, &mut ContractLedger::default(), &mut Caller::default())
                .err(),
            Some(ContractError::Invalid)
        );
    }
    assert_eq!(
        decode_v2::decode_response(
            &vec![b' '; 1_048_577],
            &mut ContractLedger::default(),
            &mut Caller::default()
        )
        .err(),
        Some(ContractError::Capacity)
    );
}
/// Full private locator field/route ordering is inert; missing/duplicate declarations refuse.
#[test]
fn strict_private_locator_complete_order_and_duplicate_refusal() {
    let locator = json!({"schema_version":"forge.review-lifecycle-inputs/1","record":{"path":"records/r.json"},
        "source":{"declared_path":"source.md","path":"records/source.md"},
        "generated_artifacts":[{"declared_path":"a.json","path":"records/a.json"},{"declared_path":"b.json","path":"records/b.json"}]});
    assert!(
        decode_v2::decode_lifecycle_inputs(
            &raw(&locator),
            &mut ContractLedger::default(),
            &mut Caller::default()
        )
        .is_ok()
    );
    let mut duplicate = locator.clone();
    duplicate["generated_artifacts"][1]["declared_path"] = json!("a.json");
    assert_eq!(
        decode_v2::decode_lifecycle_inputs(
            &raw(&duplicate),
            &mut ContractLedger::default(),
            &mut Caller::default()
        )
        .err(),
        Some(ContractError::Invalid)
    );
}

/// Complete recorded data fixture has no native owner despite its asserted presentation state.
fn recorded_fixture(
    queue: &decode_v2::DecodedV2<'_, QueueDocumentV2>,
    response_raw: &[u8],
) -> Value {
    let doc = queue.document();
    let item = &doc.items[0];
    let response: Value = serde_json::from_slice(response_raw).unwrap();
    json!({"schema_version":"forge.review-dispositions/2","identity_disclaimer":IDENTITY_DISCLAIMER,
        "queue_id":doc.queue_id,"queue_raw_sha256":queue.raw_sha256(),"as_of":"2026-10-05T00:00:02Z",
        "currentness":"unverified","closure_generation":null,"source_pins":doc.source_pins,
        "responses":[{"response_id":response["response_id"],"raw_sha256":crate::hashing::sha256_hex(response_raw),"byte_length":response_raw.len(),
            "item_key":item.key,"reviewer_key":"reviewer-1","reviewer_role":"reviewer","disposition":"approve",
            "responded_at":"2026-10-05T00:00:01Z","classification":"current"}],
        "items":[{"item_key":item.key,"item_id":item.item_id,"state":"quorum-met","reason_codes":["quorum-met"],
            "required_seats":1,"met_seats":[{"role_key":"reviewer","ordinal":0,"reviewer_key":"reviewer-1","response_id":response["response_id"]}],
            "unmet_seats":[],"response_ids":[response["response_id"]],"dissent_ids":[],"blocking":false}],
        "counts":{"items":1,"response_files":1,"unique_responses":1,"exact_duplicates":0,
            "states":{"unassigned":0,"assigned":0,"in_review":0,"conflicted":0,"changes_requested":0,"quorum_met":1,"expired":0,"stale":0}}})
}
/// Actual recorded decoder checks all item/response/state counters and source binding.
#[test]
fn complete_recorded_denominators_and_binding() {
    let mut ledger = ContractLedger::default();
    let mut caller = Caller::default();
    let queue_raw = raw(&queue_fixture());
    let queue = decode_v2::decode_queue(&queue_raw, &mut ledger, &mut caller).unwrap();
    let response_raw = raw(&response_fixture(&queue));
    let recorded = recorded_fixture(&queue, &response_raw);
    let bytes = raw(&recorded);
    let decoded = decode_v2::decode_dispositions(&bytes, &mut ledger, &mut caller).unwrap();
    decode_v2::bind_dispositions(&queue, &decoded, &mut ledger, &mut caller).unwrap();
    for variant in 0..4 {
        let mut bad = recorded.clone();
        match variant {
            0 => bad["counts"]["response_files"] = json!(0),
            1 => bad["counts"]["states"]["assigned"] = json!(1),
            2 => bad["items"] = json!([]),
            _ => bad["items"][0]["required_seats"] = json!(2),
        }
        assert_eq!(
            decode_v2::decode_dispositions(
                &raw(&bad),
                &mut ContractLedger::default(),
                &mut Caller::default()
            )
            .err(),
            Some(ContractError::Invalid)
        );
    }
    let mut altered = recorded;
    altered["queue_raw_sha256"] = json!("0".repeat(64));
    let changed_raw = raw(&altered);
    let changed = decode_v2::decode_dispositions(&changed_raw, &mut ledger, &mut caller).unwrap();
    assert_eq!(
        decode_v2::bind_dispositions(&queue, &changed, &mut ledger, &mut caller).err(),
        Some(ContractError::Binding)
    );
}
/// Complete historical dissent remains visible and cannot fill an approval seat.
#[test]
fn complete_recorded_dissent_and_currentness_labels_are_inert() {
    let mut ledger = ContractLedger::default();
    let mut caller = Caller::default();
    let queue_raw = raw(&queue_fixture());
    let queue = decode_v2::decode_queue(&queue_raw, &mut ledger, &mut caller).unwrap();
    let response_raw = raw(&response_fixture(&queue));
    let mut recorded = recorded_fixture(&queue, &response_raw);
    recorded["responses"][0]["disposition"] = json!("reject");
    recorded["items"][0]["state"] = json!("changes-requested");
    recorded["items"][0]["met_seats"] = json!([]);
    recorded["items"][0]["unmet_seats"] = json!([{"role_key":"reviewer","ordinal":0}]);
    recorded["counts"]["states"]["quorum_met"] = json!(0);
    recorded["counts"]["states"]["changes_requested"] = json!(1);
    assert_eq!(
        decode_v2::decode_dispositions(&raw(&recorded), &mut ledger, &mut caller).err(),
        Some(ContractError::Invalid)
    );
    recorded["items"][0]["dissent_ids"] = json!(["20000000-0000-4000-8000-000000000002"]);
    assert!(decode_v2::decode_dispositions(&raw(&recorded), &mut ledger, &mut caller).is_ok());
    recorded["currentness"] = json!("recorded-current");
    assert_eq!(
        decode_v2::decode_dispositions(&raw(&recorded), &mut ledger, &mut caller).err(),
        Some(ContractError::Invalid)
    );
    recorded["closure_generation"] = json!("0".repeat(64));
    let bytes = raw(&recorded);
    let decoded = decode_v2::decode_dispositions(&bytes, &mut ledger, &mut caller).unwrap();
    assert_eq!(decoded.raw(), bytes.as_slice());
    assert_eq!(decoded.document().currentness, RecordedCurrentness::RecordedCurrent);
}
/// Private response nulls/edit refusal and author exclusion use actual decode/bind paths.
#[test]
fn private_response_nulls_edit_and_asserted_author_exclusion() {
    let mut ledger = ContractLedger::default();
    let mut caller = Caller::default();
    let queue_raw = raw(&queue_fixture());
    let queue = decode_v2::decode_queue(&queue_raw, &mut ledger, &mut caller).unwrap();
    let response = response_fixture(&queue);
    for field in ["abstention_reason", "proposed_edit", "supersedes"] {
        let mut missing = response.clone();
        missing.as_object_mut().unwrap().remove(field);
        assert_eq!(
            decode_v2::decode_response(&raw(&missing), &mut ledger, &mut caller).err(),
            Some(ContractError::Invalid)
        );
    }
    let mut edit = response.clone();
    edit["proposed_edit"] = json!({"replace":"native"});
    assert_eq!(
        decode_v2::decode_response(&raw(&edit), &mut ledger, &mut caller).err(),
        Some(ContractError::UnsupportedEdit)
    );
    let mut author = response;
    author["reviewer_key"] = json!("author-1");
    let bytes = raw(&author);
    let decoded = decode_v2::decode_response(&bytes, &mut ledger, &mut caller).unwrap();
    assert_eq!(
        decode_v2::bind_response(&queue, &decoded, &mut ledger, &mut caller).err(),
        Some(ContractError::Binding)
    );
}

/// A complete successful decode/hash cannot escape the final actual post-phase stop.
#[test]
fn successful_decode_and_hash_keep_the_same_final_control() {
    let bytes = raw(&queue_fixture());
    for failed in [false, true] {
        let mut ledger = ContractLedger::default();
        let mut caller = Caller::default();
        assert!(decode_v2::decode_queue(&bytes, &mut ledger, &mut caller).is_ok());
        let cost = caller.calls;
        caller.stop_at = Some(caller.calls + cost);
        caller.failed = failed;
        let result = decode_v2::decode_queue(&bytes, &mut ledger, &mut caller).err();
        assert_eq!(
            result,
            Some(if failed {
                ContractError::ControlFailed
            } else {
                ContractError::Interrupted(Interruption::CancelRequested)
            })
        );
        let calls = caller.calls;
        assert_eq!(hash_v2::subject_id("a", "bc", &mut ledger, &mut caller).err(), result);
        assert_eq!(caller.calls, calls);
    }
    let mut ledger = ContractLedger::default();
    let mut caller = Caller::default();
    assert!(hash_v2::subject_id("a", "bc", &mut ledger, &mut caller).is_ok());
    let cost = caller.calls;
    caller.stop_at = Some(caller.calls + cost);
    assert_eq!(
        hash_v2::subject_id("a", "bc", &mut ledger, &mut caller).err(),
        Some(ContractError::Interrupted(Interruption::CancelRequested))
    );
}
/// Private roster order/route/purpose is distinct from public key order and keeps repeats.
#[test]
fn full_private_roster_binds_native_paths_and_compatible_occurrences() {
    let native: Vec<SourcePinV2> =
        serde_json::from_value(vectors()["synthetic_roster_pins_native_order"].clone()).unwrap();
    let mut second: SourcePinV2 =
        serde_json::from_value(serde_json::to_value(&native[2]).unwrap()).unwrap();
    second.artifact_key = "lifecycle:generated:1".into();
    // Equal raw hashes/extents/model identities represent plain repeated operands,
    // not manufactured capture leases; every purpose occurrence remains encoded.
    let rows = [
        hash_v2::RosterEntry {
            purpose: SourceKindV2::LifecycleRecord,
            route: None,
            pin: &native[0],
        },
        hash_v2::RosterEntry {
            purpose: SourceKindV2::OpaqueSource,
            route: Some("source.md"),
            pin: &native[1],
        },
        hash_v2::RosterEntry {
            purpose: SourceKindV2::GeneratedArtifact,
            route: Some("a.json"),
            pin: &native[2],
        },
        hash_v2::RosterEntry {
            purpose: SourceKindV2::GeneratedArtifact,
            route: Some("b.json"),
            pin: &second,
        },
    ];
    let mut ledger = ContractLedger::default();
    let mut caller = Caller::default();
    let complete = hash_v2::sources(&rows, &mut ledger, &mut caller).unwrap();
    let shorter = hash_v2::sources(&rows[..3], &mut ledger, &mut caller).unwrap();
    assert_ne!(complete, shorter);
    let changed = [
        hash_v2::RosterEntry {
            purpose: SourceKindV2::LifecycleRecord,
            route: None,
            pin: &native[0],
        },
        hash_v2::RosterEntry {
            purpose: SourceKindV2::OpaqueSource,
            route: Some("different.md"),
            pin: &native[1],
        },
        hash_v2::RosterEntry {
            purpose: SourceKindV2::GeneratedArtifact,
            route: Some("a.json"),
            pin: &native[2],
        },
        hash_v2::RosterEntry {
            purpose: SourceKindV2::GeneratedArtifact,
            route: Some("b.json"),
            pin: &second,
        },
    ];
    assert_ne!(complete, hash_v2::sources(&changed, &mut ledger, &mut caller).unwrap());
    for variant in 0..3 {
        let mut invalid = [
            hash_v2::RosterEntry {
                purpose: SourceKindV2::LifecycleRecord,
                route: None,
                pin: &native[0],
            },
            hash_v2::RosterEntry {
                purpose: SourceKindV2::OpaqueSource,
                route: Some("source.md"),
                pin: &native[1],
            },
            hash_v2::RosterEntry {
                purpose: SourceKindV2::GeneratedArtifact,
                route: Some("a.json"),
                pin: &native[2],
            },
            hash_v2::RosterEntry {
                purpose: SourceKindV2::GeneratedArtifact,
                route: Some("b.json"),
                pin: &second,
            },
        ];
        match variant {
            0 => invalid[0].route = Some("record.json"),
            1 => invalid[3].route = Some("a.json"),
            _ => invalid.swap(2, 3),
        }
        assert_eq!(
            hash_v2::sources(&invalid, &mut ledger, &mut caller).err(),
            Some(ContractError::Invalid)
        );
    }
}
