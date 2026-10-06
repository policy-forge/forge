//! Plain Queue/3 controls use complete inert declarations, never native approval fixtures.
use super::{ContractError, ContractLedger, DecodedV3, decode_queue};
use crate::review::wire_v3::{
    ADAPTER, AbstentionRule, Assignment, AuthorSeparation, ContextSnapshot, Disposition, DomainV3,
    IDENTITY_DISCLAIMER, NativeModelV3, QueueDocumentV3, RequestedAction, ReviewItemV3,
    ReviewPolicy, Reviewer, RoleDefinition, SeatRequirement, Sensitivity, SourceKindV3,
    SourcePinV3, Substitution,
};
use crate::review::{encode_v3, hash_v3};
use crate::workspace::preparation::{
    Interruption, NoopControl, ProgressUpdate, Stage, WorkControl, WorkError, WorkResult,
};
use serde::Serialize;
use serde_json::{Value, json};

/// Complete ordinary source-purpose operands; labels/digests do not prove native facts.
fn pin(kind: SourceKindV3, ordinal: usize) -> SourcePinV3 {
    let (key, schema, profile, model, root) = match kind {
        SourceKindV3::AuthorProject => (
            "authoring:project".into(),
            Some("forge.author-project/1"),
            "forge.author-project-intrinsic/1",
            None,
            None,
        ),
        SourceKindV3::AuthoringPack => (
            "authoring:pack".into(),
            Some("forge.authoring-pack/1"),
            "forge.authoring-pack-intrinsic/1",
            None,
            None,
        ),
        SourceKindV3::GapReport => (
            "authoring:gap-report".into(),
            Some("forge.applicability-report/1"),
            "forge.authoring-gap-report-complete/1",
            None,
            None,
        ),
        SourceKindV3::ApplicabilityManifest => (
            "authoring:applicability".into(),
            Some("forge.applicability/1"),
            "forge.authoring-applicability-intrinsic/1",
            None,
            None,
        ),
        SourceKindV3::Framework => (
            "authoring:framework".into(),
            None,
            "forge.authoring-framework-native/1",
            Some(NativeModelV3::Catalog),
            Some("AAAAAAAA-BBBB-4CCC-8DDD-EEEEEEEEEEEE"),
        ),
        SourceKindV3::ResolvedCatalog => (
            "authoring:resolved".into(),
            None,
            "forge.authoring-resolved-catalog-native/1",
            Some(NativeModelV3::Catalog),
            Some("11111111-2222-4333-8444-555555555555"),
        ),
        SourceKindV3::MappingCollection => (
            format!("authoring:mapping:{ordinal}"),
            None,
            "forge.authoring-mapping-native/1",
            Some(NativeModelV3::MappingCollection),
            Some("22222222-3333-4444-8555-666666666666"),
        ),
        SourceKindV3::HumanClause => (
            format!("authoring:clause:{ordinal}"),
            None,
            "forge.authoring-clause-validated/1",
            None,
            None,
        ),
        SourceKindV3::StoredPlan => (
            "authoring:plan".into(),
            Some("forge.authoring-plan/1"),
            "forge.authoring-plan-complete-equality/1",
            None,
            None,
        ),
    };
    SourcePinV3 {
        artifact_key: key,
        kind,
        raw_sha256: "a".repeat(64),
        byte_length: 17,
        schema_identity: schema.map(str::to_owned),
        validation_profile: profile.into(),
        native_model: model,
        native_root_uuid: root.map(str::to_owned),
    }
}
/// Recompute only plain framed identities; this is not a current native binder.
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
/// Complete plain six-purpose Queue fixture with explicit asserted policy and roles.
fn fixture() -> QueueDocumentV3 {
    let mut pins = vec![
        pin(SourceKindV3::AuthorProject, 0),
        pin(SourceKindV3::AuthoringPack, 0),
        pin(SourceKindV3::GapReport, 0),
        pin(SourceKindV3::ApplicabilityManifest, 0),
        pin(SourceKindV3::Framework, 0),
        pin(SourceKindV3::StoredPlan, 0),
    ];
    pins.sort_by(|a, b| a.artifact_key.cmp(&b.artifact_key));
    let mut queue = QueueDocumentV3 {
        schema_version: "forge.review-queue/3".into(),
        identity_disclaimer: IDENTITY_DISCLAIMER.into(),
        sensitivity: Sensitivity::IdsAndHashes,
        queue_id: "33333333-4444-4555-8666-777777777777".into(),
        created_at: "2026-10-01T00:00:00Z".into(),
        source_pins: pins,
        roles: vec![
            RoleDefinition { key: "approver".into() },
            RoleDefinition { key: "owner".into() },
        ],
        reviewers: vec![
            Reviewer { key: "alice".into(), role_keys: vec!["approver".into()] },
            Reviewer { key: "bob".into(), role_keys: vec!["owner".into()] },
        ],
        policies: vec![ReviewPolicy {
            key: "review".into(),
            seats: vec![SeatRequirement { role_key: "approver".into(), count: 1 }],
            substitutions: vec![Substitution {
                seat_role: "approver".into(),
                reviewer_key: "bob".into(),
                asserted_role: "owner".into(),
                reason_code: "backup".into(),
            }],
            abstention_rule: AbstentionRule::Nonapproving,
            empty_abstention_reasons: vec!["not-qualified".into()],
            author_separation: AuthorSeparation::DeclaredKeys,
        }],
        items: vec![ReviewItemV3 {
            key: "whole-plan".into(),
            item_id: String::new(),
            domain: DomainV3::AuthoringPlan,
            adapter_version: ADAPTER.into(),
            subject_id: format!("authoring-plan:{}", "0".repeat(64)),
            requested_action: RequestedAction::ReReview,
            source_keys: vec![],
            subject_sha256: "b".repeat(64),
            context: ContextSnapshot {
                reason_codes: vec!["authoring-plan-current".into()],
                related_subject_ids: vec![],
            },
            context_sha256: String::new(),
            policy_key: "review".into(),
            policy_sha256: String::new(),
            author_keys: vec!["author".into()],
            assignments: vec![Assignment {
                reviewer_key: "alice".into(),
                role_key: "approver".into(),
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
    };
    reframe(&mut queue);
    queue
}
/// Serialize only test data for adversarial input; producer emission is tested separately.
fn raw(value: &Value) -> Vec<u8> {
    serde_json::to_vec(value).unwrap()
}
/// Observe a precise safe error without requiring Debug on private ordinary holders.
fn error<T>(value: Result<T, ContractError>) -> ContractError {
    match value {
        Ok(unexpected) => {
            drop(unexpected);
            panic!("expected complete refusal");
        }
        Err(error) => error,
    }
}
/// Real complete Queue-byte intake on an ordinary caller ledger/control.
fn decode(raw: &[u8]) -> Result<DecodedV3<'_, QueueDocumentV3>, ContractError> {
    decode_queue(raw, &mut ContractLedger::default(), &mut NoopControl)
}
/// Complete fixture raw Value remains inert and fully writable only inside this test.
fn value() -> Value {
    serde_json::to_value(fixture()).unwrap()
}

/// Actual finite emission retains exact field order, explicit nulls, original UUID text and LF.
#[test]
fn finite_queue_roundtrip_has_literal_order_nulls_and_one_lf() {
    let expected = fixture();
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let output = encode_v3::queue(&expected, &mut ledger, &mut control).unwrap();
    assert!(
        output.starts_with(br#"{"schema_version":"forge.review-queue/3","identity_disclaimer":"#)
    );
    assert!(output.ends_with(b"}\n"));
    assert!(!output.ends_with(b"}\n\n"));
    let text = std::str::from_utf8(&output).unwrap();
    assert!(text.contains(r#""due_at":null,"allowed_dispositions":["approve","reject","request-changes","abstain","superseded"]"#));
    assert!(text.contains(r#""native_root_uuid":"AAAAAAAA-BBBB-4CCC-8DDD-EEEEEEEEEEEE""#));
    let actual = decode_queue(&output, &mut ledger, &mut control).unwrap();
    assert!(actual.document() == &expected);
    assert_eq!(actual.raw(), output.as_slice());
    assert_eq!(actual.raw_sha256(), crate::hashing::sha256_hex(&output));
}
/// Harmless JSON object order/whitespace is semantic-equal but changes raw identity.
#[test]
fn original_formatting_changes_raw_digest_without_typed_omission() {
    let expected = fixture();
    let compact = serde_json::to_vec(&expected).unwrap();
    let mut pretty = serde_json::to_vec_pretty(&expected).unwrap();
    pretty.push(b'\n');
    let left = decode(&compact).unwrap();
    let right = decode(&pretty).unwrap();
    assert!(left.document() == right.document());
    assert_ne!(left.raw_sha256(), right.raw_sha256());
    encode_v3::readback_queue(&expected, &pretty, &mut ContractLedger::default(), &mut NoopControl)
        .unwrap();
}
/// Every required null must be present, including the three eight-field pin option fields.
#[test]
fn omitted_required_nulls_refuse_before_serde_defaults() {
    for field in ["schema_identity", "native_model", "native_root_uuid"] {
        let mut v = value();
        v["source_pins"][0].as_object_mut().unwrap().remove(field);
        assert_eq!(error(decode(&raw(&v))), ContractError::Invalid);
    }
    let mut v = value();
    v["items"][0].as_object_mut().unwrap().remove("due_at");
    assert_eq!(error(decode(&raw(&v))), ContractError::Invalid);
    assert!(decode(&raw(&value())).is_ok());
}
/// Duplicate spellings, including escaped identical keys, are actual strict-parser refusals.
#[test]
fn duplicate_root_and_nested_escaped_keys_are_not_last_wins() {
    let encoded = String::from_utf8(raw(&value())).unwrap();
    let duplicate =
        format!(r#"{{"queue\u005fid":"33333333-4444-4555-8666-777777777777",{}"#, &encoded[1..]);
    assert_eq!(error(decode(duplicate.as_bytes())), ContractError::Invalid);
    let pattern = r#""native_model":"catalog""#;
    assert_eq!(encoded.matches(pattern).count(), 1);
    let duplicate =
        encoded.replacen(pattern, r#""native_model":"catalog","native_model":"catalog""#, 1);
    assert_eq!(error(decode(duplicate.as_bytes())), ContractError::Invalid);
}
/// Full nested closed schemas reject unknown properties without selected projection fallback.
#[test]
fn unknown_fields_refuse_at_every_queue_shape() {
    for pointer in [
        "",
        "/source_pins/0",
        "/roles/0",
        "/reviewers/0",
        "/policies/0",
        "/items/0",
        "/items/0/context",
    ] {
        let mut v = value();
        v.pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("private-extra".into(), json!("never-export"));
        assert_eq!(error(decode(&raw(&v))), ContractError::Invalid);
    }
}
/// Complete raw grammar cannot accept BOM, invalid UTF8, trailing JSON or malformed nesting.
#[test]
fn raw_syntax_utf8_bom_and_trailing_data_refuse() {
    let good = raw(&value());
    let mut bom = vec![0xef, 0xbb, 0xbf];
    bom.extend_from_slice(&good);
    let mut trailing = good;
    trailing.extend_from_slice(b" {}");
    for v in [bom, trailing, vec![0xff], b"{".to_vec(), b"".to_vec()] {
        assert_eq!(error(decode(&v)), ContractError::Invalid);
    }
}
/// Lexical schema patterns cannot substitute for real UTC calendar/nonnil UUID predicates.
#[test]
fn canonical_real_times_and_review_uuids_are_required() {
    for (pointer, replacement) in [
        ("/created_at", "2026-02-30T00:00:00Z"),
        ("/created_at", "0000-10-01T00:00:00Z"),
        ("/queue_id", "00000000-0000-0000-0000-000000000000"),
        ("/items/0/item_id", "AAAAAAAA-BBBB-4CCC-8DDD-EEEEEEEEEEEE"),
    ] {
        let mut v = value();
        *v.pointer_mut(pointer).unwrap() = json!(replacement);
        assert_eq!(error(decode(&raw(&v))), ContractError::Invalid);
    }
    let mut q = fixture();
    q.items[0].due_at = Some(q.created_at.clone());
    reframe(&mut q);
    assert_eq!(error(decode(&serde_json::to_vec(&q).unwrap())), ContractError::Invalid);
}
/// Every public pin field is observed by shape and/or complete framed correlation.
#[test]
fn all_eight_pin_fields_are_live_complete_operands() {
    for (field, replacement) in [
        ("artifact_key", json!("authoring:other")),
        ("kind", json!("human-clause")),
        ("raw_sha256", json!("c".repeat(64))),
        ("byte_length", json!(18)),
        ("schema_identity", Value::Null),
        ("validation_profile", json!("forge.authoring-clause-validated/1")),
        ("native_model", json!("catalog")),
        ("native_root_uuid", json!("11111111-2222-4333-8444-555555555555")),
    ] {
        let mut v = value();
        v["source_pins"][0][field] = replacement;
        assert!(matches!(error(decode(&raw(&v))), ContractError::Invalid | ContractError::Binding));
    }
}
/// Mandatory source counts, complete item source keys and lexical order remain exact.
#[test]
fn source_subsets_reorders_and_ordinal_gaps_refuse() {
    let mut v = value();
    v["source_pins"].as_array_mut().unwrap().remove(0);
    assert_eq!(error(decode(&raw(&v))), ContractError::Invalid);
    let mut v = value();
    v["source_pins"].as_array_mut().unwrap().reverse();
    assert_eq!(error(decode(&raw(&v))), ContractError::Invalid);
    let mut v = value();
    v["items"][0]["source_keys"].as_array_mut().unwrap().remove(0);
    assert_eq!(error(decode(&raw(&v))), ContractError::Invalid);
    let mut q = fixture();
    q.source_pins.push(pin(SourceKindV3::MappingCollection, 2));
    q.source_pins.sort_by(|a, b| a.artifact_key.cmp(&b.artifact_key));
    q.items[0].source_keys = q.source_pins.iter().map(|p| p.artifact_key.clone()).collect();
    assert_eq!(error(decode(&serde_json::to_vec(&q).unwrap())), ContractError::Invalid);
}
/// All nine kinds and ordinal10 retain lexical public order with explicit Profile companion.
#[test]
fn all_nine_kinds_profile_and_lexical_ordinals_roundtrip() {
    let mut q = fixture();
    q.source_pins.iter_mut().find(|p| p.kind == SourceKindV3::Framework).unwrap().native_model =
        Some(NativeModelV3::Profile);
    q.source_pins.push(pin(SourceKindV3::ResolvedCatalog, 0));
    for i in 0..11 {
        q.source_pins.push(pin(SourceKindV3::MappingCollection, i));
        q.source_pins.push(pin(SourceKindV3::HumanClause, i));
    }
    q.source_pins.sort_by(|a, b| a.artifact_key.cmp(&b.artifact_key));
    reframe(&mut q);
    let output = encode_v3::queue(&q, &mut ContractLedger::default(), &mut NoopControl).unwrap();
    let actual = decode(&output).unwrap();
    assert!(actual.document() == &q);
    let keys: Vec<_> = actual
        .document()
        .source_pins
        .iter()
        .filter(|p| p.kind == SourceKindV3::HumanClause)
        .map(|p| p.artifact_key.as_str())
        .collect();
    assert_eq!(
        &keys[..4],
        &["authoring:clause:0", "authoring:clause:1", "authoring:clause:10", "authoring:clause:2"]
    );
    q.source_pins.retain(|p| p.kind != SourceKindV3::ResolvedCatalog);
    q.items[0].source_keys = q.source_pins.iter().map(|p| p.artifact_key.clone()).collect();
    assert_eq!(error(decode(&serde_json::to_vec(&q).unwrap())), ContractError::Invalid);
}
/// Asserted full registry order/membership and author exclusion are not inferred defaults.
#[test]
fn role_membership_author_exclusion_and_substitution_identity_refuse() {
    for case in 0..6 {
        let mut v = value();
        match case {
            0 => v["roles"].as_array_mut().unwrap().reverse(),
            1 => v["reviewers"][0]["role_keys"] = json!(["missing"]),
            2 => v["items"][0]["author_keys"] = json!(["alice"]),
            3 => v["policies"][0]["substitutions"][0]["reviewer_key"] = json!("missing"),
            4 => {
                let mut duplicate = v["policies"][0]["substitutions"][0].clone();
                duplicate["reason_code"] = json!("different-reason");
                v["policies"][0]["substitutions"].as_array_mut().unwrap().push(duplicate);
            }
            _ => v["items"][0]["assignments"][0]["role_key"] = json!("owner"),
        }
        assert_eq!(error(decode(&raw(&v))), ContractError::Invalid);
    }
}
/// Coherent unused declared rows reach strict readback but still differ in complete equality.
#[test]
fn coherent_unused_registry_change_is_not_equal_output() {
    let expected = fixture();
    let mut v = serde_json::to_value(&expected).unwrap();
    v["reviewers"].as_array_mut().unwrap().push(json!({"key":"zed","role_keys":["owner"]}));
    let changed = raw(&v);
    assert!(decode(&changed).is_ok());
    assert_eq!(
        error(encode_v3::readback_queue(
            &expected,
            &changed,
            &mut ContractLedger::default(),
            &mut NoopControl
        )),
        ContractError::Binding
    );
}
/// Independent public hashes remain fully correlated, while opaque native subject truth is absent.
#[test]
fn context_policy_and_item_hash_mutations_refuse_binding() {
    for pointer in ["/items/0/context_sha256", "/items/0/policy_sha256", "/items/0/item_id"] {
        let mut v = value();
        *v.pointer_mut(pointer).unwrap() = if pointer.ends_with("item_id") {
            json!("11111111-2222-4333-8444-555555555555")
        } else {
            json!("c".repeat(64))
        };
        assert_eq!(error(decode(&raw(&v))), ContractError::Binding);
    }
}
/// Old family markers and private locator/init envelopes cannot enter Queue/3 intake.
#[test]
fn cross_family_envelopes_never_cast_to_queue_three() {
    for marker in [
        "forge.review-queue/1",
        "forge.review-queue/2",
        "forge.review-response/3",
        "forge.review-dispositions/3",
    ] {
        let mut v = value();
        v["schema_version"] = json!(marker);
        assert_eq!(error(decode(&raw(&v))), ContractError::Invalid);
    }
    for v in [
        json!({"schema_version":"forge.review-authoring-plan-inputs/1","project":{"path":"project.json"},"plan":{"path":"plan.json"}}),
        json!({"schema_version":"forge.review-authoring-plan-init/1","roles":[],"reviewers":[],"policies":[],"items":[]}),
    ] {
        assert_eq!(error(decode(&raw(&v))), ContractError::Invalid);
    }
}
/// One accepted controller records actual callbacks; the original ledger latches all stops.
struct Probe {
    /// Complete actual callbacks observed on this same accepted controller.
    calls: usize,
    /// One-based actual callback chosen after an authentic baseline traversal.
    stop_at: usize,
    /// Interrupt, ordinary failure, or after-Ok interruption behavior.
    mode: u8,
    /// Original first interruption retained by this actual controller.
    stopped: Option<Interruption>,
}
impl Probe {
    /// Initialize one test controller without a replacement mid-operation.
    fn new(stop_at: usize, mode: u8) -> Self {
        Self { calls: 0, stop_at, mode, stopped: None }
    }
}
impl WorkControl for Probe {
    /// Observe one real callback and return the selected exact first stop.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
        self.calls += 1;
        if let Some(reason) = self.stopped {
            return Err(WorkError::Interrupted(reason));
        }
        if self.calls == self.stop_at {
            if self.mode == 1 {
                return Err(WorkError::Failed(crate::workspace::contract::Error::invalid()));
            }
            self.stopped = Some(Interruption::CancelRequested);
            if self.mode == 2 {
                return Ok(());
            }
            return Err(WorkError::Interrupted(Interruption::CancelRequested));
        }
        Ok(())
    }
    /// Expose the retained after-Ok interruption to the genuine decoder fence.
    fn interruption(&self) -> Option<Interruption> {
        self.stopped
    }
}
/// Genuine malformed parse/schema errors are retained through the original final postfence.
#[test]
fn ordinary_parse_and_schema_failures_observe_final_control() {
    for bad in [b"{".to_vec(), raw(&json!({"schema_version":"forge.review-queue/3"}))] {
        let mut recorder = Probe::new(usize::MAX, 0);
        assert_eq!(
            error(decode_queue(&bad, &mut ContractLedger::default(), &mut recorder)),
            ContractError::Invalid
        );
        let last = recorder.calls;
        for mode in 0..3 {
            let mut control = Probe::new(last, mode);
            let mut ledger = ContractLedger::default();
            let expected = if mode == 1 {
                ContractError::ControlFailed
            } else {
                ContractError::Interrupted(Interruption::CancelRequested)
            };
            assert_eq!(error(decode_queue(&bad, &mut ledger, &mut control)), expected);
            let calls = control.calls;
            assert_eq!(error(decode_queue(&raw(&value()), &mut ledger, &mut control)), expected);
            assert_eq!(control.calls, calls);
        }
    }
}
/// Successful complete intake/encoding cannot bypass their last original checkpoint.
#[test]
fn successful_decode_and_emission_final_fences_preserve_first_stop() {
    let q = fixture();
    let input = serde_json::to_vec(&q).unwrap();
    for encode in [false, true] {
        let mut record = Probe::new(usize::MAX, 0);
        let mut ledger = ContractLedger::default();
        if encode {
            encode_v3::queue(&q, &mut ledger, &mut record).unwrap();
        } else {
            decode_queue(&input, &mut ledger, &mut record).unwrap();
        }
        let mut stopped = Probe::new(record.calls, 2);
        let mut ledger = ContractLedger::default();
        let refusal = if encode {
            error(encode_v3::queue(&q, &mut ledger, &mut stopped))
        } else {
            error(decode_queue(&input, &mut ledger, &mut stopped))
        };
        assert_eq!(refusal, ContractError::Interrupted(Interruption::CancelRequested));
        let calls = stopped.calls;
        assert_eq!(error(encode_v3::queue(&q, &mut ledger, &mut stopped)), refusal);
        assert_eq!(stopped.calls, calls);
    }
}
/// Failed ordinary work stays charged; a sticky Capacity blocks even later failure callbacks.
#[test]
fn ordinary_failed_work_and_capacity_are_not_reset_or_refunded() {
    let mut control = Probe::new(usize::MAX, 0);
    let mut ledger = ContractLedger::default();
    assert_eq!(error(decode_queue(b"x", &mut ledger, &mut control)), ContractError::Invalid);
    assert_eq!(ledger.bytes(268_435_456), Err(ContractError::Capacity));
    let calls = control.calls;
    control.stop_at = calls + 1;
    control.mode = 1;
    assert_eq!(
        error(decode_queue(&raw(&value()), &mut ledger, &mut control)),
        ContractError::Capacity
    );
    assert_eq!(control.calls, calls);
}
/// The real finite writer reaches its exact LF-inclusive cap before Vec growth.
#[test]
fn finite_writer_cap_and_ordinary_serializer_failure_are_postfenced() {
    /// An ordinary serializer rejection with no native or control authority.
    struct Refuses;
    impl Serialize for Refuses {
        /// Return an ordinary serializer failure through the real writer phase.
        fn serialize<S: serde::Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("plain test serialization refusal"))
        }
    }
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    assert_eq!(
        crate::review::encode_v2::admitted_json(&"é", 5, &mut ledger, &mut control).unwrap(),
        b"\"\xc3\xa9\"\n"
    );
    assert_eq!(
        error(crate::review::encode_v2::admitted_json(&"é", 4, &mut ledger, &mut control)),
        ContractError::Capacity
    );
    let mut record = Probe::new(usize::MAX, 0);
    let mut ledger = ContractLedger::default();
    assert_eq!(
        error(crate::review::encode_v2::admitted_json(&Refuses, 16, &mut ledger, &mut record)),
        ContractError::Invalid
    );
    let mut stopped = Probe::new(record.calls, 1);
    let mut ledger = ContractLedger::default();
    assert_eq!(
        error(crate::review::encode_v2::admitted_json(&Refuses, 16, &mut ledger, &mut stopped)),
        ContractError::ControlFailed
    );
}
/// The public raw ceiling is checked before parsing, under unchanged original Capacity semantics.
#[test]
fn complete_raw_cap_is_not_renewed_by_queue_intake() {
    let mut input = serde_json::to_vec(&fixture()).unwrap();
    input.resize(encode_v3::MAX_QUEUE_BYTES + 1, b' ');
    let mut ledger = ContractLedger::default();
    let mut control = Probe::new(usize::MAX, 0);
    assert_eq!(error(decode_queue(&input, &mut ledger, &mut control)), ContractError::Capacity);
    let calls = control.calls;
    assert_eq!(error(decode_queue(b"{}", &mut ledger, &mut control)), ContractError::Capacity);
    assert_eq!(control.calls, calls);
}

/// Repeated eligibility for one key is one graph edge; distinct keys exhaust the real bound.
#[test]
fn repeated_eligibility_and_distinct_keys_use_exact_graph_cardinality() {
    let mut repeated = fixture();
    repeated.reviewers[0].role_keys.push("owner".into());
    repeated.policies[0].substitutions[0].reviewer_key = "alice".into();
    reframe(&mut repeated);
    let mut ledger = ContractLedger::default();
    ledger.graph(0, 199_999, 0).unwrap();
    super::queue(&repeated, &mut ledger, &mut NoopControl).unwrap();
    let distinct = fixture();
    let mut ledger = ContractLedger::default();
    ledger.graph(0, 199_999, 0).unwrap();
    assert_eq!(
        super::queue(&distinct, &mut ledger, &mut NoopControl),
        Err(ContractError::Capacity)
    );
}
