//! Pure encoding, selection and ledger controls; no fixture constructs native approval.
//! Inert Values below test byte binding only, not captured applicability semantics.

use super::*;
use crate::workspace::preparation::{Interruption, NoopControl, ProgressUpdate, Stage, WorkError};
use serde_json::json;

/// Inert evidence tuple with a private href; it is never a sealed native proof.
fn evidence() -> ResourceEvidence {
    ResourceEvidence {
        resource_type: ResourceType::Catalog,
        href: "PRIVATE-PATH/catalog.json".into(),
        raw_sha256: "a".repeat(64),
        root_uuid: "11111111-1111-4111-8111-111111111111".into(),
        document_version: "v1".into(),
        oscal_version: "1.1.2".into(),
        resolved_catalog_sha256: None,
    }
}

/// Private original decision shape for the serializer, without domain-currentness claims.
fn decision() -> Value {
    json!({"control_id":"ac.1","state":"applicable","reviewer_key":"owner",
        "reviewed_at":null,"rationale":"PRIVATE-RATIONALE"})
}

/// Borrow a complete primitive payload without inventing a factory-owned approval.
fn encoding<'a>(framework: &'a ResourceEvidence, original: &'a Value) -> SubjectEncoding<'a> {
    SubjectEncoding {
        adapter_version: ADAPTER_VERSION,
        framework: FrameworkEncoding::from(framework),
        control_fingerprint: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        decision: original,
        classification: GapClassification::ApplicableMapped,
        positive_mapping_count: 2,
        no_relationship_count: 1,
    }
}

/// Hash an inert primitive under the same externally bounded contract ledger.
fn digest(value: &impl Serialize) -> String {
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    ledger.bound(|ledger| subject_hash(value, ledger, &mut control)).unwrap()
}

/// The reviewed compact field order and NUL-separated domain have an independent byte oracle.
#[test]
fn compact_field_order_and_domain_are_exact() {
    let framework = evidence();
    let original = decision();
    let payload = encoding(&framework, &original);
    let expected = concat!(
        "{\"adapter_version\":\"forge.applicability-review/1\",\"framework\":{\"resource_type\":\"catalog\",",
        "\"raw_sha256\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",",
        "\"root_uuid\":\"11111111-1111-4111-8111-111111111111\",\"document_version\":\"v1\",",
        "\"oscal_version\":\"1.1.2\",\"resolved_catalog_sha256\":null},",
        "\"control_fingerprint\":\"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\",",
        "\"decision\":{\"control_id\":\"ac.1\",\"rationale\":\"PRIVATE-RATIONALE\",\"reviewed_at\":null,",
        "\"reviewer_key\":\"owner\",\"state\":\"applicable\"},\"classification\":\"applicable-mapped\",",
        "\"positive_mapping_count\":2,\"no_relationship_count\":1}"
    );
    assert_eq!(serde_json::to_vec(&payload).unwrap(), expected.as_bytes());
    let mut expected_hash = Sha256::new();
    expected_hash.update(b"forge.applicability-review-subject/1\0");
    expected_hash.update(expected.as_bytes());
    assert_eq!(digest(&payload), crate::hashing::lower_hex(&expected_hash.finalize()));
    assert_ne!(digest(&payload), crate::hashing::sha256_hex(expected.as_bytes()));
}

/// A missing private field and an explicit null are different original assertions.
#[test]
fn omission_null_and_private_change_bind_distinctly() {
    let framework = evidence();
    let mut original = decision();
    let with_null = digest(&encoding(&framework, &original));
    original.as_object_mut().unwrap().remove("reviewed_at");
    assert_ne!(with_null, digest(&encoding(&framework, &original)));
    let prior = digest(&encoding(&framework, &original));
    original["rationale"] = json!("OTHER-PRIVATE-RATIONALE");
    assert_ne!(prior, digest(&encoding(&framework, &original)));
}

/// Generic borrowed Value encoding preserves array order; this is not an admitted decision fixture.
#[test]
fn whole_value_array_order_is_not_normalized() {
    assert_ne!(
        digest(&json!({"private-array":["first","second"]})),
        digest(&json!({"private-array":["second","first"]}))
    );
}

/// Every selected framework tuple component affects the sensitive subject hash.
#[test]
fn all_framework_identity_fields_bind() {
    let original = decision();
    let base = evidence();
    let expected = digest(&encoding(&base, &original));
    for field in 0..6 {
        let mut changed = base.clone();
        match field {
            0 => changed.resource_type = ResourceType::Profile,
            1 => changed.raw_sha256 = "c".repeat(64),
            2 => changed.root_uuid = "22222222-2222-4222-8222-222222222222".into(),
            3 => changed.document_version = "v2".into(),
            4 => changed.oscal_version = "1.1.3".into(),
            5 => changed.resolved_catalog_sha256 = Some("d".repeat(64)),
            _ => unreachable!(),
        }
        assert_ne!(expected, digest(&encoding(&changed, &original)));
    }
}

/// Href is excluded from the fingerprint while exact raw source pins remain independently required.
#[test]
fn private_href_is_not_exposed_or_hashed_as_a_path() {
    let original = decision();
    let base = evidence();
    let mut changed = base.clone();
    changed.href = "OTHER-PRIVATE-PATH/catalog.json".into();
    assert_eq!(digest(&encoding(&base, &original)), digest(&encoding(&changed, &original)));
    let encoded = serde_json::to_vec(&encoding(&base, &original)).unwrap();
    assert!(!encoded.windows(b"PRIVATE-PATH".len()).any(|window| window == b"PRIVATE-PATH"));
}

/// Actual native fingerprint, classification and both per-target counters bind independently.
#[test]
fn native_fingerprint_classification_and_counts_bind() {
    let framework = evidence();
    let original = decision();
    let base = digest(&encoding(&framework, &original));
    let mut changed = encoding(&framework, &original);
    changed.control_fingerprint = "different-native-fingerprint";
    assert_ne!(base, digest(&changed));
    changed = encoding(&framework, &original);
    changed.classification = GapClassification::ApplicableUnmapped;
    assert_ne!(base, digest(&changed));
    changed = encoding(&framework, &original);
    changed.positive_mapping_count = 3;
    assert_ne!(base, digest(&changed));
    changed = encoding(&framework, &original);
    changed.no_relationship_count = 2;
    assert_ne!(base, digest(&changed));
}

/// Public context contains exactly fixed sorted codes and no private IDs or prose.
#[test]
fn context_is_minimized_and_sorted() {
    let context = context_codes(
        GapClassification::ApplicableMapped,
        "applicable",
        &mut ContractLedger::default(),
        &mut NoopControl,
    )
    .unwrap();
    assert_eq!(
        context.reason_codes,
        ["applicability-re-review", "applicable-mapped", "scope-applicable"]
    );
    assert_eq!(context.related_subject_ids, Vec::<String>::new());
    let encoded = serde_json::to_string(&context).unwrap();
    for marker in ["PRIVATE", "owner", "rationale", "reviewed_at"] {
        assert!(!encoded.contains(marker));
    }
}

/// Unsupported raw scope spelling is refused without guessing a classification or trimming.
#[test]
fn unknown_scope_is_not_normalized() {
    for state in [" Applicable", "unknown", "applicable "] {
        assert_eq!(
            context_codes(
                GapClassification::ApplicableMapped,
                state,
                &mut ContractLedger::default(),
                &mut NoopControl
            )
            .err(),
            Some(ContractError::Binding)
        );
    }
}

/// Explicit selection requires the complete existing token grammar and strict sorted uniqueness.
#[test]
fn selected_ids_reject_invalid_width_spelling_and_order() {
    for ids in [
        &[][..],
        &["ac.2", "ac.1"][..],
        &["ac.1", "ac.1"][..],
        &["ac/1"][..],
        &["éclair"][..],
        &[".ac"][..],
    ] {
        assert_eq!(
            validate_selected(ids, &mut ContractLedger::default(), &mut NoopControl),
            Err(ContractError::Invalid)
        );
    }
    let wide = "a".repeat(257);
    assert_eq!(
        validate_selected(&[&wide], &mut ContractLedger::default(), &mut NoopControl),
        Err(ContractError::Invalid)
    );
    let edge = "a".repeat(256);
    assert_eq!(
        validate_selected(&[&edge], &mut ContractLedger::default(), &mut NoopControl),
        Ok(())
    );
    assert_eq!(
        validate_selected(&["ac.1", "ac:2"], &mut ContractLedger::default(), &mut NoopControl),
        Ok(())
    );
}

/// A missing original or a foreign control ID cannot be promoted into an explicit selected decision.
#[test]
fn actual_omission_and_foreign_original_refuse() {
    assert_eq!(
        require_decision(None, "ac.1", &mut ContractLedger::default()).err(),
        Some(ContractError::Binding)
    );
    let original = decision();
    assert_eq!(
        require_decision(Some(&original), "ac.2", &mut ContractLedger::default()).err(),
        Some(ContractError::Binding)
    );
    assert!(std::ptr::eq(
        std::ptr::from_ref(
            require_decision(Some(&original), "ac.1", &mut ContractLedger::default()).unwrap()
        ),
        std::ptr::from_ref(&original)
    ));
}

/// A correct total cannot conceal a wrong native category histogram.
#[test]
fn all_six_counts_and_total_are_reconciled() {
    let mut counts = ClassificationCounts {
        total: 6,
        applicable_mapped: 1,
        applicable_reviewed_no_relationship: 1,
        applicable_unmapped: 1,
        not_applicable: 1,
        deferred: 1,
        under_review: 1,
    };
    assert_eq!(reconcile_counts(6, [1; 6], &counts, &mut ContractLedger::default()), Ok(()));
    counts.applicable_mapped = 2;
    counts.under_review = 0;
    assert_eq!(
        reconcile_counts(6, [1; 6], &counts, &mut ContractLedger::default()),
        Err(ContractError::Binding)
    );
    counts.total = 5;
    assert_eq!(
        reconcile_counts(6, [2, 1, 1, 1, 1, 0], &counts, &mut ContractLedger::default()),
        Err(ContractError::Binding)
    );
}

/// Histogram overflow is capacity, and the same ledger refuses later unrelated work.
#[test]
fn count_overflow_and_shared_capacity_remain_sticky() {
    let counts = ClassificationCounts {
        total: 0,
        applicable_mapped: usize::MAX,
        applicable_reviewed_no_relationship: 1,
        ..ClassificationCounts::default()
    };
    let mut ledger = ContractLedger::default();
    assert_eq!(
        ledger.bound(|ledger| reconcile_counts(0, [0; 6], &counts, ledger)),
        Err(ContractError::Capacity)
    );
    assert_eq!(ledger.visits(0), Err(ContractError::Capacity));
    let mut ledger = ContractLedger::default();
    ledger.derived(33_554_432).unwrap();
    assert_eq!(
        context_codes(
            GapClassification::UnderReview,
            "under-review",
            &mut ledger,
            &mut NoopControl
        )
        .err(),
        Some(ContractError::Capacity)
    );
    assert_eq!(ledger.bytes(0), Err(ContractError::Capacity));
}

/// The actual writer permits its complete limit then refuses before changing digest state or length.
#[test]
fn hash_writer_boundary_refuses_before_mutation() {
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let mut sink = HashSink {
        hash: Sha256::new(),
        length: 0,
        limit: 3,
        ledger: &mut ledger,
        control: &mut control,
        failure: None,
    };
    sink.write_all(b"abc").unwrap();
    let prior = sink.hash.clone().finalize();
    assert!(sink.write_all(b"d").is_err());
    assert_eq!(sink.length, 3);
    assert_eq!(sink.hash.clone().finalize(), prior);
    assert_eq!(sink.failure, Some(ContractError::Capacity));
    assert!(sink.write(b"").is_err());
    assert!(sink.flush().is_err());
}

/// Deterministic caller control interrupts an actual serializer writer fence, not setup alone.
struct WriterStop {
    /// Remaining successful checkpoints before a sticky cancellation.
    remaining: usize,
    /// Actual delivered cancellation, exposed through the shared control contract.
    interrupted: Option<Interruption>,
}

impl WorkControl for WriterStop {
    /// Stop at the configured actual encoding checkpoint and retain its typed reason.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> Result<(), WorkError> {
        if self.remaining == 0 {
            self.interrupted = Some(Interruption::CancelRequested);
            return Err(WorkError::Interrupted(Interruption::CancelRequested));
        }
        self.remaining -= 1;
        Ok(())
    }
    /// Return only the cancellation that was actually reached by the writer.
    fn interruption(&self) -> Option<Interruption> {
        self.interrupted
    }
}

/// A reached writer cancellation remains typed and cannot become a partial successful hash.
#[test]
fn serializer_cancellation_is_typed_and_sticky() {
    let mut ledger = ContractLedger::default();
    let mut control = WriterStop { remaining: 1, interrupted: None };
    assert_eq!(
        ledger
            .bound(|ledger| subject_hash(&json!({"private":"value"}), ledger, &mut control))
            .err(),
        Some(ContractError::Interrupted(Interruption::CancelRequested))
    );
    assert_eq!(control.interruption(), Some(Interruption::CancelRequested));
    assert_eq!(ledger.bytes(0), Err(ContractError::Interrupted(Interruption::CancelRequested)));
}

/// Repeated subject encodings consume the existing command byte-work cap; no new ledger is installed.
#[test]
fn hash_encoding_consumes_existing_byte_work_budget() {
    let mut ledger = ContractLedger::default();
    ledger.bytes(268_435_456 - SUBJECT_PREFIX.len()).unwrap();
    assert_eq!(
        ledger.bound(|ledger| subject_hash(&json!("x"), ledger, &mut NoopControl)).err(),
        Some(ContractError::Capacity)
    );
    assert_eq!(ledger.bytes(0), Err(ContractError::Capacity));
}
