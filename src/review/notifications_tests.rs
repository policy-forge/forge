//! Proposed file-backed notification controls. Synthetic native pins are inert
//! recorded declarations. No native approval, identity authentication, delivery,
//! signature, connector plan or human acceptance is established by these tests.
//! The actual queue is encoded by maintained contracts, captured once and held.

use std::path::{Path, PathBuf};
use std::rc::Rc;

use serde_json::{Value, json};

use crate::evidence_capture::CaptureRole;
use crate::review::capture::{HeldReviewInputs, Pool, ReviewCapture};
use crate::review::decode::{ContractError, ContractLedger};
use crate::review::wire::{
    AbstentionRule, Assignment, AuthorSeparation, ContextSnapshot, Disposition, Domain,
    IDENTITY_DISCLAIMER, QueueDocument, RequestedAction, ReviewItem, ReviewPolicy, Reviewer,
    RoleDefinition, SeatRequirement, Sensitivity, SourceModel, SourcePin, Substitution,
};
use crate::review::{encode, validate};
use crate::workspace::preparation::{
    Interruption, NoopControl, ProgressUpdate, Stage, WorkControl, WorkError,
};

/// Actual original owner and its unchanged command-wide accounting state.
struct FileFixture {
    /// All real queue/optional extra originals; no detached owner is constructed.
    held: Rc<HeldReviewInputs>,
    /// Actual index returned by the confined queue capture.
    queue_index: usize,
    /// The same monotonic ledger used for capture, prepare and final fencing.
    ledger: ContractLedger,
    /// Canonical private owned root used only by these local file controls.
    root: PathBuf,
    /// Last field drops after held handles, permitting Windows cleanup.
    _directory: tempfile::TempDir,
}

/// Make a complete mixed-domain recorded queue with valid maintained bindings.
fn queue_fixture() -> QueueDocument {
    let mut queue = QueueDocument {
        schema_version: "forge.review-queue/1".into(),
        identity_disclaimer: IDENTITY_DISCLAIMER.into(),
        sensitivity: Sensitivity::IdsAndHashes,
        queue_id: "22222222-2222-4222-8222-222222222222".into(),
        created_at: "2026-10-04T00:00:00Z".into(),
        source_pins: vec![SourcePin {
            artifact_key: "declared".into(),
            model: SourceModel::Catalog,
            native_root_uuid: Some("11111111-1111-4111-8111-111111111111".into()),
            raw_sha256: "0".repeat(64),
            byte_length: 100,
            schema_identity: "oscal.catalog/1".into(),
        }],
        roles: vec![
            RoleDefinition { key: "review".into() },
            RoleDefinition { key: "security".into() },
        ],
        reviewers: vec![
            Reviewer { key: "alice".into(), role_keys: vec!["review".into(), "security".into()] },
            Reviewer { key: "author".into(), role_keys: vec!["review".into()] },
            Reviewer { key: "bob".into(), role_keys: vec!["review".into()] },
            Reviewer { key: "spare".into(), role_keys: vec!["security".into()] },
        ],
        policies: vec![ReviewPolicy {
            key: "policy".into(),
            seats: vec![
                SeatRequirement { role_key: "review".into(), count: 1 },
                SeatRequirement { role_key: "security".into(), count: 1 },
            ],
            substitutions: vec![Substitution {
                seat_role: "security".into(),
                reviewer_key: "spare".into(),
                asserted_role: "security".into(),
                reason_code: "declared-substitute".into(),
            }],
            abstention_rule: AbstentionRule::Nonapproving,
            empty_abstention_reasons: vec!["recused".into()],
            author_separation: AuthorSeparation::DeclaredKeys,
        }],
        items: vec![
            item_fixture(
                "assigned",
                Domain::MappingAssertion,
                "map.1",
                vec![Assignment { reviewer_key: "bob".into(), role_key: "review".into() }],
            ),
            item_fixture(
                "duo",
                Domain::ApplicabilityDecision,
                "ac.1",
                vec![
                    Assignment { reviewer_key: "alice".into(), role_key: "review".into() },
                    Assignment { reviewer_key: "alice".into(), role_key: "security".into() },
                ],
            ),
            item_fixture("unassigned", Domain::ApplicabilityDecision, "ac.2", vec![]),
        ],
    };
    queue.items[2].due_at = None;
    rebind(&mut queue);
    queue
}

/// Construct one inert item; maintained hash functions bind it before encoding.
fn item_fixture(
    key: &str,
    domain: Domain,
    subject_id: &str,
    assignments: Vec<Assignment>,
) -> ReviewItem {
    ReviewItem {
        key: key.into(),
        item_id: String::new(),
        domain,
        adapter_version: match domain {
            Domain::MappingAssertion => "forge.mapping-review/1",
            Domain::ApplicabilityDecision => "forge.applicability-review/1",
        }
        .into(),
        subject_id: subject_id.into(),
        requested_action: RequestedAction::ReReview,
        source_keys: vec!["declared".into()],
        subject_sha256: "1".repeat(64),
        context: ContextSnapshot {
            reason_codes: vec!["rereview".into()],
            related_subject_ids: vec!["PRIVATE-RELATED-ID".into()],
        },
        context_sha256: String::new(),
        policy_key: "policy".into(),
        policy_sha256: String::new(),
        author_keys: vec!["author".into()],
        assignments,
        due_at: Some("2026-10-05T00:00:00Z".into()),
        allowed_dispositions: vec![
            Disposition::Approve,
            Disposition::Reject,
            Disposition::RequestChanges,
            Disposition::Abstain,
            Disposition::Superseded,
        ],
    }
}

/// Recompute complete declared item identities, without inventing native proof.
fn rebind(queue: &mut QueueDocument) {
    let mut items = std::mem::take(&mut queue.items);
    let mut ledger = ContractLedger::default();
    for item in &mut items {
        item.context_sha256 = validate::context_hash(queue, item, &mut ledger).unwrap();
        item.policy_sha256 =
            validate::policy_hash(queue, &queue.policies[0], item, &mut ledger).unwrap();
        item.item_id = validate::item_id(queue, item, &mut ledger).unwrap();
    }
    queue.items = items;
}

/// Encode a structurally valid declaration via the maintained real queue encoder.
fn queue_raw(queue: &QueueDocument) -> Vec<u8> {
    encode::queue(queue, &mut ContractLedger::default(), &mut NoopControl).unwrap()
}

/// Capture the actual nonempty queue once, optionally with one real forbidden original.
fn file_fixture(raw: &[u8], extra: Option<(CaptureRole, Pool)>) -> FileFixture {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    std::fs::write(root.join("queue.json"), raw).unwrap();
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let mut capture = ReviewCapture::new(&root, &[], &mut ledger, &mut control).unwrap();
    let queue_index = capture
        .required(
            Path::new("queue.json"),
            CaptureRole::ReviewQueue,
            Pool::Queue,
            10_485_760,
            &mut ledger,
            &mut control,
        )
        .unwrap();
    if let Some((role, pool)) = extra {
        std::fs::write(root.join("extra.json"), b"{}").unwrap();
        capture
            .required(Path::new("extra.json"), role, pool, 1_048_576, &mut ledger, &mut control)
            .unwrap();
    }
    FileFixture {
        held: Rc::new(capture.finish()),
        queue_index,
        ledger,
        root,
        _directory: directory,
    }
}

/// Prepare from actual held bytes and recheck all real inputs under the same ledger.
fn exported(fixture: &mut FileFixture) -> Value {
    assert!(fixture.root.join("queue.json").is_file());
    let prepared =
        super::prepare(&fixture.held, fixture.queue_index, &mut fixture.ledger, &mut NoopControl)
            .unwrap_or_else(|_| panic!("valid captured synthetic queue must export"));
    prepared.verify_inputs(&mut fixture.ledger, &mut NoopControl).unwrap();
    assert_eq!(prepared.bytes().last(), Some(&b'\n'));
    assert!(prepared.bytes().len() <= 10_485_760);
    // Ordinary assertion decoding gives no duplicate-key admission credit.
    serde_json::from_slice(prepared.bytes()).unwrap()
}

/// Compute the independently specified fixed prefix and ordered notice tuple.
fn expected_notice_key(
    queue: &QueueDocument,
    item: &ReviewItem,
    intention: &str,
    recipient: Option<&str>,
    role: Option<&str>,
) -> String {
    let mut bytes = b"forge.review-notification-key/1\0".to_vec();
    bytes.extend(
        serde_json::to_vec(&(&queue.queue_id, &item.item_id, intention, recipient, role)).unwrap(),
    );
    format!("notice_{}", crate::hashing::sha256_hex(&bytes))
}

/// Describe the complete minimized row independently of the producer's DTO.
fn expected_row(
    queue: &QueueDocument,
    item: &ReviewItem,
    recipient: Option<&str>,
    role: Option<&str>,
) -> Value {
    let intention = if recipient.is_some() { "request-review" } else { "assign-reviewer" };
    json!({
        "notification_key": expected_notice_key(queue, item, intention, recipient, role),
        "item_key": item.key,
        "item_id": item.item_id,
        "domain": item.domain,
        "adapter_version": item.adapter_version,
        "subject_id": item.subject_id,
        "subject_sha256": item.subject_sha256,
        "context_sha256": item.context_sha256,
        "policy_key": item.policy_key,
        "policy_sha256": item.policy_sha256,
        "source_keys": item.source_keys,
        "reason_codes": item.context.reason_codes,
        "due_at": item.due_at,
        "intention": intention,
        "recipient_key": recipient,
        "recipient_role": role,
    })
}

/// Borrow only the expected complete array of actual proposed output rows.
fn notices(output: &Value) -> &[Value] {
    output["notifications"].as_array().unwrap()
}

/// Retain every notice key in original order for semantic-generation controls.
fn notice_keys(output: &Value) -> Vec<&str> {
    notices(output).iter().map(|row| row["notification_key"].as_str().unwrap()).collect()
}

/// Deterministic caller cancellation, with no clock or native authority.
struct Stop;

impl WorkControl for Stop {
    /// Refuse the next reached cooperative phase with a fixed typed first stop.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> Result<(), WorkError> {
        Err(WorkError::Interrupted(Interruption::CancelRequested))
    }

    /// Expose the same fixed cause to later fences.
    fn interruption(&self) -> Option<Interruption> {
        Some(Interruption::CancelRequested)
    }
}

/// The complete mixed-domain queue projects every explicit assignment and zero-assignment item.
#[test]
fn complete_rows_counts_order_and_raw_queue_pin_are_exact() {
    let queue = queue_fixture();
    let raw = queue_raw(&queue);
    let output = exported(&mut file_fixture(&raw, None));
    let expected = json!({
        "schema_version": "forge.review-notifications/1",
        "identity_disclaimer": IDENTITY_DISCLAIMER,
        "source_scope": "captured-queue-recorded-snapshot-only",
        "delivery_scope": "local-export-only",
        "sensitivity": "ids-and-hashes",
        "queue_original": {
            "schema_version": queue.schema_version,
            "queue_id": queue.queue_id,
            "raw_sha256": crate::hashing::sha256_hex(&raw),
            "byte_length": raw.len(),
            "created_at": queue.created_at,
        },
        "recorded_source_pins": queue.source_pins,
        "counts": {
            "queue_items": 3, "assigned_items": 2, "unassigned_items": 1,
            "assignment_occurrences": 3, "notification_rows": 4,
            "declared_reviewers": 4, "declared_roles": 2,
            "declared_policies": 1, "recorded_source_pins": 1,
        },
        "notifications": [
            expected_row(&queue, &queue.items[0], Some("bob"), Some("review")),
            expected_row(&queue, &queue.items[1], Some("alice"), Some("review")),
            expected_row(&queue, &queue.items[1], Some("alice"), Some("security")),
            expected_row(&queue, &queue.items[2], None, None),
        ],
    });
    assert_eq!(output, expected);
}

/// Two explicit roles for one asserted key remain two notices; substitutes and authors are not inferred.
#[test]
fn recipient_roles_are_occurrences_and_substitution_is_not_inferred() {
    let output = exported(&mut file_fixture(&queue_raw(&queue_fixture()), None));
    let recipients: Vec<_> = notices(&output)
        .iter()
        .map(|row| (row["recipient_key"].as_str(), row["recipient_role"].as_str()))
        .collect();
    assert_eq!(
        recipients,
        vec![
            (Some("bob"), Some("review")),
            (Some("alice"), Some("review")),
            (Some("alice"), Some("security")),
            (None, None),
        ]
    );
    assert!(
        notices(&output)
            .iter()
            .all(|row| row["recipient_key"] != "spare" && row["recipient_key"] != "author")
    );
    assert_ne!(notices(&output)[1]["notification_key"], notices(&output)[2]["notification_key"]);
}

/// A differently spaced actual queue changes its original hash without changing typed notice identity.
#[test]
fn whitespace_only_original_generation_keeps_notice_keys() {
    let queue = queue_fixture();
    let compact = queue_raw(&queue);
    let mut spaced = serde_json::to_vec_pretty(&queue).unwrap();
    spaced.push(b'\n');
    assert_ne!(compact, spaced);
    let first = exported(&mut file_fixture(&compact, None));
    let second = exported(&mut file_fixture(&spaced, None));
    assert_eq!(notice_keys(&first), notice_keys(&second));
    assert_eq!(first["notifications"], second["notifications"]);
    assert_ne!(first["queue_original"]["raw_sha256"], second["queue_original"]["raw_sha256"]);
    assert_eq!(second["queue_original"]["raw_sha256"], crate::hashing::sha256_hex(&spaced));
    assert_eq!(second["queue_original"]["byte_length"], spaced.len());
}

/// A valid changed recorded source pin is rebound through maintained identities and changes every notice.
#[test]
fn regenerated_source_pin_changes_item_and_notice_identities() {
    let initial = queue_fixture();
    let mut changed = queue_fixture();
    changed.source_pins[0].raw_sha256 = "2".repeat(64);
    rebind(&mut changed);
    let first = exported(&mut file_fixture(&queue_raw(&initial), None));
    let second = exported(&mut file_fixture(&queue_raw(&changed), None));
    assert_eq!(first["counts"], second["counts"]);
    for (before, after) in notices(&first).iter().zip(notices(&second)) {
        assert_ne!(before["item_id"], after["item_id"]);
        assert_ne!(before["notification_key"], after["notification_key"]);
    }
    assert_eq!(second["recorded_source_pins"], serde_json::to_value(&changed.source_pins).unwrap());
}

/// Deadline declarations bind identity but never cause a hidden current-time or expiry decision.
#[test]
fn regenerated_deadline_changes_only_its_items_notices() {
    let initial = queue_fixture();
    let mut changed = queue_fixture();
    changed.items[0].due_at = Some("2026-10-06T00:00:00Z".into());
    rebind(&mut changed);
    let first = exported(&mut file_fixture(&queue_raw(&initial), None));
    let second = exported(&mut file_fixture(&queue_raw(&changed), None));
    assert_ne!(notices(&first)[0]["notification_key"], notices(&second)[0]["notification_key"]);
    assert_eq!(notices(&second)[0]["due_at"], "2026-10-06T00:00:00Z");
    assert_eq!(&notices(&first)[1..], &notices(&second)[1..]);
    assert_eq!(first["counts"], second["counts"]);
}

/// Full context hashes are regenerated while only permitted reason tokens are exported.
#[test]
fn regenerated_context_changes_identity_without_related_id_disclosure() {
    let initial = queue_fixture();
    let mut changed = queue_fixture();
    changed.items[1].context.reason_codes = vec!["source-changed".into()];
    changed.items[1].context.related_subject_ids = vec!["SECOND-PRIVATE-RELATED-ID".into()];
    rebind(&mut changed);
    let first = exported(&mut file_fixture(&queue_raw(&initial), None));
    let second = exported(&mut file_fixture(&queue_raw(&changed), None));
    for index in [1, 2] {
        assert_ne!(
            notices(&first)[index]["notification_key"],
            notices(&second)[index]["notification_key"]
        );
        assert_eq!(notices(&second)[index]["reason_codes"], json!(["source-changed"]));
    }
    assert_eq!(notices(&first)[0], notices(&second)[0]);
    assert_eq!(notices(&first)[3], notices(&second)[3]);
    assert!(!second.to_string().contains("PRIVATE-RELATED-ID"));
}

/// A reviewer appears only after a valid explicit assignment is added and rebound.
#[test]
fn explicit_assignment_is_required_to_notify_a_declared_substitute() {
    let mut queue = queue_fixture();
    queue.items[2].assignments =
        vec![Assignment { reviewer_key: "spare".into(), role_key: "security".into() }];
    rebind(&mut queue);
    let output = exported(&mut file_fixture(&queue_raw(&queue), None));
    assert_eq!(output["counts"]["assigned_items"], 3);
    assert_eq!(output["counts"]["unassigned_items"], 0);
    assert_eq!(output["counts"]["assignment_occurrences"], 4);
    assert_eq!(output["counts"]["notification_rows"], 4);
    assert_eq!(
        notices(&output)[3],
        expected_row(&queue, &queue.items[2], Some("spare"), Some("security"))
    );
}

/// Stored valid-looking hashes cannot bypass maintained complete queue binding validation.
#[test]
fn stale_item_binding_refuses_the_complete_export() {
    let raw = queue_raw(&queue_fixture());
    let mut value: Value = serde_json::from_slice(&raw).unwrap();
    value["items"][0]["subject_sha256"] = json!("3".repeat(64));
    let changed = serde_json::to_vec(&value).unwrap();
    let mut fixture = file_fixture(&changed, None);
    assert!(matches!(
        super::prepare(&fixture.held, fixture.queue_index, &mut fixture.ledger, &mut NoopControl,),
        Err(ContractError::Binding)
    ));
}

/// Duplicate keys, missing explicit nulls, BOM and unknown transport/private fields refuse before export.
#[test]
fn strict_raw_closed_shape_refuses_all_unsupported_fields() {
    let raw = queue_raw(&queue_fixture());
    let value: Value = serde_json::from_slice(&raw).unwrap();
    let mut unknown = value.clone();
    unknown["delivery_url"] = json!("https://example.invalid/private-token");
    let mut rationale = value.clone();
    rationale["items"][0]["rationale"] = json!("PRIVATE-RATIONALE-BODY");
    let mut missing = value;
    missing["items"][2].as_object_mut().unwrap().remove("due_at");
    let mut duplicate = b"{\"schema_version\":\"forge.review-queue/1\",".to_vec();
    duplicate.extend_from_slice(&raw[1..]);
    let mut bom = vec![0xef, 0xbb, 0xbf];
    bom.extend_from_slice(&raw);
    for invalid in [
        serde_json::to_vec(&unknown).unwrap(),
        serde_json::to_vec(&rationale).unwrap(),
        serde_json::to_vec(&missing).unwrap(),
        duplicate,
        bom,
    ] {
        let mut fixture = file_fixture(&invalid, None);
        assert!(
            super::prepare(
                &fixture.held,
                fixture.queue_index,
                &mut fixture.ledger,
                &mut NoopControl,
            )
            .is_err()
        );
    }
}

/// A real captured Source original refuses even when it resembles an inert recorded pin declaration.
#[test]
fn complete_queue_only_cohort_refuses_actual_source_original() {
    let mut fixture =
        file_fixture(&queue_raw(&queue_fixture()), Some((CaptureRole::Catalog, Pool::Source)));
    assert_eq!(fixture.held.source_original_count(), 1);
    assert!(matches!(
        super::prepare(&fixture.held, fixture.queue_index, &mut fixture.ledger, &mut NoopControl,),
        Err(ContractError::Binding)
    ));
}

/// A real captured auxiliary configuration is not silently omitted from the final original cohort.
#[test]
fn complete_queue_only_cohort_refuses_actual_auxiliary_original() {
    let mut fixture = file_fixture(
        &queue_raw(&queue_fixture()),
        Some((CaptureRole::ReviewPrivateConfig, Pool::Auxiliary)),
    );
    assert!(matches!(
        super::prepare(&fixture.held, fixture.queue_index, &mut fixture.ledger, &mut NoopControl,),
        Err(ContractError::Binding)
    ));
}

/// A real captured private response is not an input to notification export or a delivery authority.
#[test]
fn complete_queue_only_cohort_refuses_actual_response_original() {
    let mut fixture = file_fixture(
        &queue_raw(&queue_fixture()),
        Some((CaptureRole::ReviewResponse, Pool::Response)),
    );
    assert!(matches!(
        super::prepare(&fixture.held, fixture.queue_index, &mut fixture.ledger, &mut NoopControl,),
        Err(ContractError::Binding)
    ));
}

/// Appending to an actually held queue refuses its final fence; this control is Unix-only.
#[cfg(unix)]
#[test]
fn actual_queue_tail_drift_refuses_prepared_final_fence() {
    use std::io::Write as _;
    let mut fixture = file_fixture(&queue_raw(&queue_fixture()), None);
    let prepared =
        super::prepare(&fixture.held, fixture.queue_index, &mut fixture.ledger, &mut NoopControl)
            .unwrap_or_else(|_| panic!("valid captured synthetic queue must export"));
    let mut file =
        std::fs::OpenOptions::new().append(true).open(fixture.root.join("queue.json")).unwrap();
    file.write_all(b" ").unwrap();
    drop(file);
    assert!(prepared.verify_inputs(&mut fixture.ledger, &mut NoopControl).is_err());
}

/// An actual caller stop stays latched if the same captured preparation is retried with a no-op control.
#[test]
fn cancellation_stays_sticky_for_the_same_capture_and_ledger() {
    let mut fixture = file_fixture(&queue_raw(&queue_fixture()), None);
    assert!(matches!(
        super::prepare(&fixture.held, fixture.queue_index, &mut fixture.ledger, &mut Stop,),
        Err(ContractError::Interrupted(Interruption::CancelRequested))
    ));
    assert!(matches!(
        super::prepare(&fixture.held, fixture.queue_index, &mut fixture.ledger, &mut NoopControl,),
        Err(ContractError::Interrupted(Interruption::CancelRequested))
    ));
}

/// Exhausting the same actual capture's byte-work ledger refuses all later preparation without reset.
#[test]
fn capacity_stays_sticky_after_actual_capture_work() {
    let mut fixture = file_fixture(&queue_raw(&queue_fixture()), None);
    // Capture already charged positive bytes; adding the entire ceiling must exceed it.
    assert_eq!(fixture.ledger.bytes(268_435_456), Err(ContractError::Capacity));
    assert!(matches!(
        super::prepare(&fixture.held, fixture.queue_index, &mut fixture.ledger, &mut NoopControl,),
        Err(ContractError::Capacity)
    ));
}

/// A caller cancellation at the genuine prepared holder's final fence cannot be erased by retry.
#[test]
fn prepared_final_fence_preserves_first_typed_stop() {
    let mut fixture = file_fixture(&queue_raw(&queue_fixture()), None);
    let prepared =
        super::prepare(&fixture.held, fixture.queue_index, &mut fixture.ledger, &mut NoopControl)
            .unwrap_or_else(|_| panic!("valid captured synthetic queue must export"));
    assert!(matches!(
        prepared.verify_inputs(&mut fixture.ledger, &mut Stop),
        Err(ContractError::Interrupted(Interruption::CancelRequested))
    ));
    assert!(matches!(
        prepared.verify_inputs(&mut fixture.ledger, &mut NoopControl),
        Err(ContractError::Interrupted(Interruption::CancelRequested))
    ));
}

/// Only the closed minimized projection is returned, without private context, native currentness or send claims.
#[test]
fn privacy_minimization_retains_disclaimers_and_sensitive_pin_labels() {
    let output = exported(&mut file_fixture(&queue_raw(&queue_fixture()), None));
    let encoded = output.to_string();
    assert!(!encoded.contains("PRIVATE-RELATED-ID"));
    assert!(!encoded.contains("declared-substitute"));
    assert!(!encoded.contains("recused"));
    for field in [
        "rationale",
        "proposed_edit",
        "source_excerpt",
        "currentness",
        "closure_generation",
        "sent",
        "delivered",
        "recipient_url",
        "project_root",
        "absolute_path",
    ] {
        assert!(output.get(field).is_none());
        assert!(notices(&output).iter().all(|row| row.get(field).is_none()));
    }
    assert_eq!(output["identity_disclaimer"], IDENTITY_DISCLAIMER);
    assert_eq!(output["source_scope"], "captured-queue-recorded-snapshot-only");
    assert_eq!(output["delivery_scope"], "local-export-only");
    assert_eq!(output["sensitivity"], "ids-and-hashes");
    assert_eq!(output["recorded_source_pins"][0]["artifact_key"], "declared");
}

/// Exact same held queue bytes produce the same full output under two different private roots.
#[test]
fn output_is_deterministic_across_distinct_actual_capture_roots() {
    let raw = queue_raw(&queue_fixture());
    let mut first = file_fixture(&raw, None);
    let mut second = file_fixture(&raw, None);
    assert_ne!(first.root, second.root);
    assert_eq!(exported(&mut first), exported(&mut second));
}

/// Borrow the real captured queue into a complete private output model for oracle controls.
/// This does not construct a successful prepared owner or grant native authority.
fn with_document(
    work: impl FnOnce(
        &super::Document<'_>,
        &crate::review::decode::Decoded<'_, QueueDocument>,
        &mut ContractLedger,
        &mut dyn WorkControl,
    ),
) {
    let mut fixture = file_fixture(&queue_raw(&queue_fixture()), None);
    let mut control = NoopControl;
    fixture
        .held
        .bind_queue_export_original(fixture.queue_index, &mut fixture.ledger, &mut control)
        .unwrap();
    let decoded = crate::review::decode::decode_queue(
        fixture.held.bytes(fixture.queue_index).unwrap(),
        &mut fixture.ledger,
        &mut control,
    )
    .unwrap();
    let counts = super::counts(decoded.document(), &mut fixture.ledger, &mut control).unwrap();
    let identities = super::identities(
        decoded.document(),
        counts.notification_rows,
        &mut fixture.ledger,
        &mut control,
    )
    .unwrap();
    let document = super::Document {
        schema_version: "forge.review-notifications/1",
        identity_disclaimer: IDENTITY_DISCLAIMER,
        source_scope: super::SOURCE_SCOPE,
        delivery_scope: super::DELIVERY_SCOPE,
        sensitivity: "ids-and-hashes",
        queue_original: super::QueueOriginal {
            schema_version: &decoded.document().schema_version,
            queue_id: &decoded.document().queue_id,
            raw_sha256: decoded.raw_sha256(),
            byte_length: decoded.raw().len(),
            created_at: &decoded.document().created_at,
        },
        recorded_source_pins: &decoded.document().source_pins,
        counts,
        notifications: super::Rows { queue: decoded.document(), identities: &identities },
    };
    work(&document, &decoded, &mut fixture.ledger, &mut control);
}

/// The dry/retained writers consume the complete admitted byte ceiling, including LF.
#[test]
fn full_document_dry_count_and_retained_ceiling_match_exactly() {
    with_document(|document, decoded, ledger, control| {
        let (admitted, none) =
            super::encode(document, super::MAX_OUTPUT, false, ledger, control).unwrap();
        assert!(none.is_none());
        super::reserve_projection(document, ledger, control).unwrap();
        let (actual, bytes) = super::encode(document, admitted, true, ledger, control).unwrap();
        let bytes = bytes.unwrap();
        assert_eq!(actual, admitted);
        assert_eq!(bytes.len(), admitted);
        assert_eq!(bytes.last(), Some(&b'\n'));
        super::oracle(&bytes, decoded, document, ledger, control).unwrap();
    });
}

/// One byte less than the actual complete document refuses, never returns a partial export.
#[test]
fn complete_output_limit_minus_one_latches_whole_capacity() {
    with_document(|document, _decoded, ledger, control| {
        let (admitted, _) =
            super::encode(document, super::MAX_OUTPUT, false, ledger, control).unwrap();
        assert!(matches!(
            ledger.bound(|ledger| super::encode(document, admitted - 1, true, ledger, control)),
            Err(ContractError::Capacity)
        ));
        assert!(matches!(ledger.checkpoint(control), Err(ContractError::Capacity)));
    });
}

/// Actual token work reaches the primitive global ceiling and refuses its first extra byte.
/// These tokens are not a closed notification document or a successful capture owner.
#[test]
fn primitive_whole_writer_bound_counts_lf_and_extra_token() {
    let token = vec![b' '; super::MAX_OUTPUT - 1];
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let mut writer = super::JsonWriter {
        output: None,
        count: 0,
        limit: super::MAX_OUTPUT,
        ledger: &mut ledger,
        control: &mut control,
        error: None,
    };
    writer.append(&token).unwrap();
    writer.append(b"\n").unwrap();
    assert_eq!(writer.count, super::MAX_OUTPUT);
    assert!(matches!(writer.append(b"x"), Err(ContractError::Capacity)));
    assert!(matches!(writer.ledger.checkpoint(writer.control), Err(ContractError::Capacity)));
}

/// Full digest inspection rejects a collision after an otherwise distinct prefix.
/// Injected digest facts exercise the real registry helper, not SHA collision evidence.
#[test]
fn full_digest_collision_refuses_and_distinct_registry_restores_order() {
    let mut collision = vec![
        super::Identity { digest: [9; 32], ordinal: 0 },
        super::Identity { digest: [2; 32], ordinal: 1 },
        super::Identity { digest: [2; 32], ordinal: 2 },
    ];
    assert!(matches!(
        super::admit_identities(&mut collision, &mut ContractLedger::default(), &mut NoopControl),
        Err(ContractError::Binding)
    ));
    let mut distinct = vec![
        super::Identity { digest: [9; 32], ordinal: 0 },
        super::Identity { digest: [2; 32], ordinal: 1 },
        super::Identity { digest: [4; 32], ordinal: 2 },
    ];
    super::admit_identities(&mut distinct, &mut ContractLedger::default(), &mut NoopControl)
        .unwrap();
    assert_eq!(distinct.iter().map(|row| row.ordinal).collect::<Vec<_>>(), vec![0, 1, 2]);
    assert_eq!(distinct.iter().map(|row| row.digest[0]).collect::<Vec<_>>(), vec![9, 2, 4]);
}

/// Schema-valid field tampering still fails complete original projection equality.
#[test]
fn generated_projection_oracle_rejects_counts_pins_and_row_drift() {
    with_document(|document, decoded, ledger, control| {
        let (_, bytes) = super::encode(document, super::MAX_OUTPUT, true, ledger, control).unwrap();
        let original: Value = serde_json::from_slice(&bytes.unwrap()).unwrap();
        let definition: Value = serde_json::from_str(super::SCHEMA).unwrap();
        let validator = jsonschema::validator_for(&definition).unwrap();
        let mut altered = Vec::new();
        let mut counts = original.clone();
        counts["counts"]["assigned_items"] = json!(3);
        altered.push(counts);
        let mut pin = original.clone();
        pin["recorded_source_pins"][0]["byte_length"] = json!(99);
        altered.push(pin);
        let mut raw = original.clone();
        raw["queue_original"]["raw_sha256"] = json!("f".repeat(64));
        altered.push(raw);
        let mut row = original.clone();
        row["notifications"][0]["due_at"] = json!("2026-10-06T00:00:00Z");
        altered.push(row);
        for changed in altered {
            assert!(validator.is_valid(&changed));
            let encoded = serde_json::to_vec(&changed).unwrap();
            assert!(matches!(
                super::oracle(&encoded, decoded, document, ledger, control),
                Err(ContractError::Binding)
            ));
        }
    });
}

/// Complete borrowed row arithmetic refuses its first extra row before registry allocation.
/// The direct counter input is inert and intentionally not a valid assigned queue.
#[test]
fn primitive_row_ceiling_refuses_full_bound_plus_one() {
    let mut queue = queue_fixture();
    queue.items.truncate(1);
    queue.items[0].assignments = (0..super::MAX_ROWS)
        .map(|_| Assignment { reviewer_key: "alice".into(), role_key: "review".into() })
        .collect();
    let mut ledger = ContractLedger::default();
    let count = super::counts(&queue, &mut ledger, &mut NoopControl).unwrap();
    assert_eq!(count.notification_rows, super::MAX_ROWS);
    queue.items[0]
        .assignments
        .push(Assignment { reviewer_key: "bob".into(), role_key: "review".into() });
    assert!(matches!(
        ledger.bound(|ledger| super::counts(&queue, ledger, &mut NoopControl)),
        Err(ContractError::Capacity)
    ));
    assert!(matches!(ledger.checkpoint(&mut NoopControl), Err(ContractError::Capacity)));
}

/// Caller stop reached at the mandatory postparse fence, without a production clock override.
struct StopAfterParse {
    /// Actual checkpoint calls, not a guessed deadline or proof flag.
    calls: usize,
}

impl WorkControl for StopAfterParse {
    /// Admit the preparse fence, then stop at its unconditional postparse fence.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> Result<(), WorkError> {
        self.calls += 1;
        if self.calls >= 2 {
            Err(WorkError::Interrupted(Interruption::CancelRequested))
        } else {
            Ok(())
        }
    }
    /// Preserve the actual reached caller reason after the second checkpoint.
    fn interruption(&self) -> Option<Interruption> {
        (self.calls >= 2).then_some(Interruption::CancelRequested)
    }
}

/// A failed strict parse still reaches the original postphase control and preserves its stop.
/// Invalid bytes are a private oracle fault fixture, never an admitted prepared document.
#[test]
fn invalid_oracle_parse_reaches_postphase_and_keeps_first_stop() {
    with_document(|document, decoded, ledger, _control| {
        let mut stop = StopAfterParse { calls: 0 };
        assert!(matches!(
            ledger.bound(|ledger| super::oracle(b"{", decoded, document, ledger, &mut stop)),
            Err(ContractError::Interrupted(Interruption::CancelRequested))
        ));
        assert_eq!(stop.calls, 2);
        assert!(matches!(
            ledger.checkpoint(&mut NoopControl),
            Err(ContractError::Interrupted(Interruption::CancelRequested))
        ));
    });
}
/// Count genuine decoder checkpoints and stop only at the selected extra boundary.
/// This is a deterministic observer, not an acceptance clock or native proof flag.
struct StopAtPostDecode {
    /// Complete reached checkpoints in this observer instance.
    calls: usize,
    /// None calibrates; Some selects the mandatory new post-decode occurrence.
    stop_at: Option<usize>,
}

impl WorkControl for StopAtPostDecode {
    /// Record the actual caller boundary and preserve cancellation once reached.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> Result<(), WorkError> {
        self.calls += 1;
        if self.stop_at.is_some_and(|stop| self.calls >= stop) {
            Err(WorkError::Interrupted(Interruption::CancelRequested))
        } else {
            Ok(())
        }
    }

    /// Return only the actual reached observer cancellation.
    fn interruption(&self) -> Option<Interruption> {
        self.stop_at.filter(|stop| self.calls >= *stop).map(|_| Interruption::CancelRequested)
    }
}

/// A failed initial decode reaches its new postphase fence, not an old decoder stop.
#[test]
fn failed_initial_queue_decode_observes_only_new_postphase_and_latches_stop() {
    let mut fixture = file_fixture(b"{", None);
    let mut observed = StopAtPostDecode { calls: 0, stop_at: None };
    // Calibrate the actual existing pre-decode path against genuine held bytes.
    // No ledger is reset and this ordinary Invalid result creates no prepared owner.
    fixture.ledger.checkpoint(&mut observed).unwrap();
    fixture
        .held
        .bind_queue_export_original(fixture.queue_index, &mut fixture.ledger, &mut observed)
        .unwrap();
    assert!(matches!(
        crate::review::decode::decode_queue(
            fixture.held.bytes(fixture.queue_index).unwrap(),
            &mut fixture.ledger,
            &mut observed,
        ),
        Err(ContractError::Invalid)
    ));
    let postdecode = observed.calls.checked_add(1).unwrap();
    let mut stop = StopAtPostDecode { calls: 0, stop_at: Some(postdecode) };
    assert!(matches!(
        super::prepare(&fixture.held, fixture.queue_index, &mut fixture.ledger, &mut stop,),
        Err(ContractError::Interrupted(Interruption::CancelRequested))
    ));
    assert_eq!(stop.calls, postdecode);
    assert!(matches!(
        super::prepare(&fixture.held, fixture.queue_index, &mut fixture.ledger, &mut NoopControl,),
        Err(ContractError::Interrupted(Interruption::CancelRequested))
    ));
}

/// Enter maintained clap parsing and public execution for the explicit local export.
/// This is library CLI dispatch, not a subprocess or authenticated sender campaign.
#[cfg(any(target_os = "linux", target_os = "macos", windows))]
fn invoke_notification_cli(
    root: &Path,
    queue: &Path,
    output: &Path,
) -> Result<(), crate::ForgeError> {
    use clap::Parser as _;
    let cli = crate::cli::Cli::try_parse_from([
        std::ffi::OsString::from("forge"),
        std::ffi::OsString::from("review"),
        std::ffi::OsString::from("export-notifications"),
        std::ffi::OsString::from("--project-root"),
        root.as_os_str().to_owned(),
        std::ffi::OsString::from("--queue"),
        queue.as_os_str().to_owned(),
        std::ffi::OsString::from("--output"),
        output.as_os_str().to_owned(),
    ])
    .unwrap();
    crate::cli::execute(&cli)
}

/// Check the complete maintained fixture projection against an actual published file.
/// Existing independent row expectations are reused; no producer output is the oracle.
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn assert_cli_notification_artifact(path: &Path, queue: &QueueDocument, raw: &[u8]) -> Vec<u8> {
    let bytes = std::fs::read(path).unwrap();
    assert_eq!(bytes.last(), Some(&b'\n'));
    assert!(bytes.len() <= 10_485_760);
    let actual: Value = serde_json::from_slice(&bytes).unwrap();
    let expected = json!({
        "schema_version": "forge.review-notifications/1",
        "identity_disclaimer": IDENTITY_DISCLAIMER,
        "source_scope": "captured-queue-recorded-snapshot-only",
        "delivery_scope": "local-export-only",
        "sensitivity": "ids-and-hashes",
        "queue_original": {
            "schema_version": queue.schema_version,
            "queue_id": queue.queue_id,
            "raw_sha256": crate::hashing::sha256_hex(raw),
            "byte_length": raw.len(),
            "created_at": queue.created_at,
        },
        "recorded_source_pins": queue.source_pins,
        "counts": {
            "queue_items": 3, "assigned_items": 2, "unassigned_items": 1,
            "assignment_occurrences": 3, "notification_rows": 4,
            "declared_reviewers": 4, "declared_roles": 2,
            "declared_policies": 1, "recorded_source_pins": 1,
        },
        "notifications": [
            expected_row(queue, &queue.items[0], Some("bob"), Some("review")),
            expected_row(queue, &queue.items[1], Some("alice"), Some("review")),
            expected_row(queue, &queue.items[1], Some("alice"), Some("security")),
            expected_row(queue, &queue.items[2], None, None),
        ],
    });
    assert_eq!(actual, expected);
    bytes
}

/// Actual clap dispatch publishes the complete minimized mixed-domain queue projection.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn real_cli_notification_export_publishes_complete_counts_rows_and_original_pin() {
    let queue = queue_fixture();
    let raw = queue_raw(&queue);
    let fixture = file_fixture(&raw, None);
    invoke_notification_cli(&fixture.root, Path::new("queue.json"), Path::new("notices.json"))
        .unwrap();
    let _published =
        assert_cli_notification_artifact(&fixture.root.join("notices.json"), &queue, &raw);
    assert_eq!(std::fs::read(fixture.root.join("queue.json")).unwrap(), raw);
}

/// The same actual original produces byte-identical files through two real CLI publications.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn real_cli_notification_export_is_byte_identical_at_distinct_new_destinations() {
    let queue = queue_fixture();
    let raw = queue_raw(&queue);
    let fixture = file_fixture(&raw, None);
    for output in ["first.json", "second.json"] {
        invoke_notification_cli(&fixture.root, Path::new("queue.json"), Path::new(output)).unwrap();
    }
    let first = assert_cli_notification_artifact(&fixture.root.join("first.json"), &queue, &raw);
    let second = assert_cli_notification_artifact(&fixture.root.join("second.json"), &queue, &raw);
    assert_eq!(first, second);
    assert_eq!(std::fs::read(fixture.root.join("queue.json")).unwrap(), raw);
}

/// A real existing destination and the actual input original cannot be overwritten through CLI dispatch.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn real_cli_notification_export_refuses_overwrite_and_input_output_alias() {
    let raw = queue_raw(&queue_fixture());
    let fixture = file_fixture(&raw, None);
    let retained = b"unrelated complete retained output\n";
    std::fs::write(fixture.root.join("existing.json"), retained).unwrap();
    assert!(
        invoke_notification_cli(&fixture.root, Path::new("queue.json"), Path::new("existing.json"))
            .is_err()
    );
    assert_eq!(std::fs::read(fixture.root.join("existing.json")).unwrap(), retained);
    assert!(
        invoke_notification_cli(&fixture.root, Path::new("queue.json"), Path::new("queue.json"))
            .is_err()
    );
    assert_eq!(std::fs::read(fixture.root.join("queue.json")).unwrap(), raw);
}

/// Malformed originals and real parent/absolute escape attempts produce no notification destination.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn real_cli_notification_export_refuses_malformed_and_actual_escape_inputs() {
    let raw = queue_raw(&queue_fixture());
    let fixture = file_fixture(&raw, None);
    std::fs::write(fixture.root.join("malformed.json"), b"{").unwrap();
    assert!(
        invoke_notification_cli(
            &fixture.root,
            Path::new("malformed.json"),
            Path::new("malformed-notices.json")
        )
        .is_err()
    );
    assert!(!fixture.root.join("malformed-notices.json").exists());

    let outside_directory = tempfile::tempdir().unwrap();
    let outside_root = outside_directory.path().canonicalize().unwrap();
    assert_eq!(fixture.root.parent(), outside_root.parent());
    let outside_queue = outside_root.join("outside-queue.json");
    std::fs::write(&outside_queue, &raw).unwrap();
    let escaped_queue =
        Path::new("..").join(outside_root.file_name().unwrap()).join("outside-queue.json");
    for (input, output) in [
        (escaped_queue.as_path(), "parent-escape-notices.json"),
        (outside_queue.as_path(), "absolute-escape-notices.json"),
    ] {
        assert!(invoke_notification_cli(&fixture.root, input, Path::new(output)).is_err());
        assert!(!fixture.root.join(output).exists());
    }
    let outside_output = outside_root.join("outside-notices.json");
    let escaped_output =
        Path::new("..").join(outside_root.file_name().unwrap()).join("outside-notices.json");
    assert!(
        invoke_notification_cli(&fixture.root, Path::new("queue.json"), &escaped_output).is_err()
    );
    assert!(!outside_output.exists());
    assert_eq!(std::fs::read(outside_queue).unwrap(), raw);
    assert_eq!(std::fs::read(fixture.root.join("queue.json")).unwrap(), raw);
}

/// Inject real queue drift only when the maintained publisher has a complete staged file.
/// Its exact staging-name coupling qualifies this as one implementation-specific final-fence control.
#[cfg(any(target_os = "linux", target_os = "macos"))]
struct DriftAtNotificationPublication {
    /// Actual owned fixture root, containing only the original and publisher staging.
    root: PathBuf,
    /// Complete bytes genuinely observed after the publisher wrote and synced its stage.
    staged: Option<Vec<u8>>,
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
impl WorkControl for DriftAtNotificationPublication {
    /// Observe a real publisher stage, then alter the held queue before its final proof check.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> Result<(), WorkError> {
        use std::io::Write as _;
        if self.staged.is_none() {
            for entry in std::fs::read_dir(&self.root).unwrap() {
                let entry = entry.unwrap();
                if entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| name.starts_with(".forge-authoring-file-"))
                {
                    let bytes = std::fs::read(entry.path().join("file")).unwrap();
                    assert_eq!(bytes.last(), Some(&b'\n'));
                    let value: Value = serde_json::from_slice(&bytes).unwrap();
                    assert_eq!(value["schema_version"], "forge.review-notifications/1");
                    let mut queue = std::fs::OpenOptions::new()
                        .append(true)
                        .open(self.root.join("queue.json"))
                        .unwrap();
                    queue.write_all(b" ").unwrap();
                    drop(queue);
                    self.staged = Some(bytes);
                    break;
                }
            }
        }
        Ok(())
    }

    /// This observer injects an actual file-generation fault, never a control-stop proof.
    fn interruption(&self) -> Option<Interruption> {
        None
    }
}

/// Real command publication refuses queue drift after complete staging and before the guarded rename.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn real_notification_command_final_owner_fence_refuses_queue_drift_after_staging() {
    use crate::review::commands::{CommandError, ExportNotificationsOptions, export_notifications};
    let raw = queue_raw(&queue_fixture());
    let fixture = file_fixture(&raw, None);
    let mut drift = DriftAtNotificationPublication { root: fixture.root.clone(), staged: None };
    let result = export_notifications(
        &ExportNotificationsOptions {
            project_root: &fixture.root,
            queue: Path::new("queue.json"),
            output: Path::new("late-drift-notices.json"),
        },
        &mut drift,
    );
    assert!(drift.staged.is_some());
    assert!(matches!(result, Err(CommandError::Contract(ContractError::Binding))));
    assert!(!fixture.root.join("late-drift-notices.json").exists());
    let mut changed = raw;
    changed.push(b' ');
    assert_eq!(std::fs::read(fixture.root.join("queue.json")).unwrap(), changed);
    assert!(std::fs::read_dir(&fixture.root).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_str()
            .is_some_and(|name| name.starts_with(".forge-authoring-file-"))
    }));
}

/// Real Windows CLI dispatch refuses the unavailable no-replace publisher rather than claiming an export.
/// Only a future actual Windows run can qualify this proposed platform control.
#[cfg(windows)]
#[test]
fn real_windows_notification_cli_refuses_unsupported_publication_without_destination() {
    let raw = queue_raw(&queue_fixture());
    let fixture = file_fixture(&raw, None);
    let result = invoke_notification_cli(
        &fixture.root,
        Path::new("queue.json"),
        Path::new("windows-notices.json"),
    );
    assert!(matches!(result, Err(crate::ForgeError::Io(_))));
    assert!(!fixture.root.join("windows-notices.json").exists());
    assert_eq!(std::fs::read(fixture.root.join("queue.json")).unwrap(), raw);
}
