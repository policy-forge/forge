//! Genuine plain original-byte and recorded-history codec controls, never native proof.
use super::{MAX_RECORDED_BYTES, MAX_RESPONSE_BYTES, decode_dispositions, decode_response};
use crate::review::decode::{ContractError, ContractLedger};
use crate::review::wire_v3::{
    ADAPTER, Disposition, DispositionCounts, DispositionsDocumentV3, DomainV3, IDENTITY_DISCLAIMER,
    ItemDisposition, ItemState, MetSeat, NativeModelV3, RecordedCurrentness, RecordedResponse,
    RequestedAction, ResponseClassification, ResponseDocumentV3, SourceKindV3, SourcePinV3,
    StateCounts, SupersessionReference, UnmetSeat,
};
use crate::review::{decode_v3, encode_v3};
use crate::workspace::preparation::{
    Interruption, NoopControl, ProgressUpdate, Stage, WorkControl, WorkError, WorkResult,
};
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

/// Canonical plain review identities, independent of native identifiers.
fn id(n: u8) -> String {
    format!("{n:08x}-1111-4111-8111-111111111111")
}
/// The full ordinary six-purpose roster; a digest is asserted data only.
fn pins() -> Vec<SourcePinV3> {
    let mut values = vec![
        pin(SourceKindV3::AuthorProject, 0),
        pin(SourceKindV3::AuthoringPack, 0),
        pin(SourceKindV3::GapReport, 0),
        pin(SourceKindV3::ApplicabilityManifest, 0),
        pin(SourceKindV3::Framework, 0),
        pin(SourceKindV3::StoredPlan, 0),
    ];
    values.sort_by(|a, b| a.artifact_key.cmp(&b.artifact_key));
    values
}
/// Complete private response assertions, with all three explicit nulls initially present.
fn response() -> ResponseDocumentV3 {
    ResponseDocumentV3 {
        schema_version: "forge.review-response/3".into(),
        identity_disclaimer: IDENTITY_DISCLAIMER.into(),
        response_id: id(1),
        queue_id: id(9),
        queue_raw_sha256: "a".repeat(64),
        item_key: "whole-plan".into(),
        item_id: id(8),
        domain: DomainV3::AuthoringPlan,
        adapter_version: ADAPTER.into(),
        requested_action: RequestedAction::ReReview,
        source_pins: pins(),
        subject_sha256: "b".repeat(64),
        context_sha256: "c".repeat(64),
        policy_sha256: "d".repeat(64),
        reviewer_key: "alice".into(),
        reviewer_role: "approver".into(),
        disposition: Disposition::Approve,
        responded_at: "2026-10-01T00:00:00Z".into(),
        rationale: "private reviewer evidence".into(),
        abstention_reason: None,
        proposed_edit: (),
        supersedes: None,
    }
}
/// Full recorded history includes withdrawn dissent, a foreign target, and an exact duplicate count.
fn recorded() -> DispositionsDocumentV3 {
    let rows = vec![
        RecordedResponse {
            response_id: id(1),
            raw_sha256: "a".repeat(64),
            byte_length: 200,
            item_key: "whole-plan".into(),
            reviewer_key: "alice".into(),
            reviewer_role: "approver".into(),
            disposition: Disposition::Approve,
            responded_at: "2026-10-01T00:00:00Z".into(),
            classification: ResponseClassification::Current,
        },
        RecordedResponse {
            response_id: id(2),
            raw_sha256: "b".repeat(64),
            byte_length: 201,
            item_key: "whole-plan".into(),
            reviewer_key: "bob".into(),
            reviewer_role: "approver".into(),
            disposition: Disposition::Reject,
            responded_at: "2026-10-01T00:00:01Z".into(),
            classification: ResponseClassification::Superseded,
        },
        RecordedResponse {
            response_id: id(3),
            raw_sha256: "c".repeat(64),
            byte_length: 202,
            item_key: "foreign-plan".into(),
            reviewer_key: "carol".into(),
            reviewer_role: "approver".into(),
            disposition: Disposition::RequestChanges,
            responded_at: "2026-10-01T00:00:02Z".into(),
            classification: ResponseClassification::Foreign,
        },
    ];
    DispositionsDocumentV3 {
        schema_version: "forge.review-dispositions/3".into(),
        identity_disclaimer: IDENTITY_DISCLAIMER.into(),
        queue_id: id(9),
        queue_raw_sha256: "a".repeat(64),
        as_of: "2026-10-01T00:00:03Z".into(),
        currentness: RecordedCurrentness::Unverified,
        closure_generation: None,
        source_pins: pins(),
        responses: rows,
        items: vec![ItemDisposition {
            item_key: "whole-plan".into(),
            item_id: id(8),
            state: ItemState::InReview,
            reason_codes: vec!["asserted-identity-only".into()],
            required_seats: 2,
            met_seats: vec![MetSeat {
                role_key: "approver".into(),
                ordinal: 0,
                reviewer_key: "alice".into(),
                response_id: id(1),
            }],
            unmet_seats: vec![UnmetSeat { role_key: "approver".into(), ordinal: 1 }],
            response_ids: vec![id(1), id(2)],
            dissent_ids: vec![id(2)],
            blocking: true,
        }],
        counts: DispositionCounts {
            items: 1,
            response_files: 4,
            unique_responses: 3,
            exact_duplicates: 1,
            states: StateCounts {
                unassigned: 0,
                assigned: 0,
                in_review: 1,
                conflicted: 0,
                changes_requested: 0,
                quorum_met: 0,
                expired: 0,
                stale: 0,
            },
        },
    }
}
/// Complete test-only serialization; actual production emit/readback has separate controls.
fn raw<T: serde::Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).unwrap()
}
/// Inspect a fixed error without exposing holder-private diagnostics or requiring Debug.
fn error<T>(value: Result<T, ContractError>) -> ContractError {
    match value {
        Ok(unexpected) => {
            drop(unexpected);
            panic!("expected complete refusal");
        }
        Err(e) => e,
    }
}
/// Plain response intake on one caller-owned ledger and control.
fn decode_r(bytes: &[u8]) -> Result<decode_v3::DecodedV3<'_, ResponseDocumentV3>, ContractError> {
    decode_response(bytes, &mut ContractLedger::default(), &mut NoopControl)
}
/// Plain whole recorded intake on one caller-owned ledger and control.
fn decode_d(
    bytes: &[u8],
) -> Result<decode_v3::DecodedV3<'_, DispositionsDocumentV3>, ContractError> {
    decode_dispositions(bytes, &mut ContractLedger::default(), &mut NoopControl)
}
/// Mutate complete raw data without creating a native owner or approval.
fn rv() -> Value {
    serde_json::to_value(response()).unwrap()
}
/// Complete recorded raw tree, including all foreign and withdrawn rows.
fn dv() -> Value {
    serde_json::to_value(recorded()).unwrap()
}

/// Actual finite response preserves declaration order, private evidence, required nulls and one LF.
#[test]
fn finite_response_preserves_order_nulls_private_bytes_and_actual_digest() {
    let expected = response();
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let output = encode_v3::response(&expected, &mut ledger, &mut control).unwrap();
    assert!(
        output
            .starts_with(br#"{"schema_version":"forge.review-response/3","identity_disclaimer":"#)
    );
    assert!(output.ends_with(b"}\n"));
    assert!(!output.ends_with(b"}\n\n"));
    let text = std::str::from_utf8(&output).unwrap();
    assert!(text.contains(r#""abstention_reason":null,"proposed_edit":null,"supersedes":null"#));
    assert!(text.contains("private reviewer evidence"));
    let actual = decode_response(&output, &mut ledger, &mut control).unwrap();
    assert!(actual.document() == &expected);
    assert_eq!(actual.raw(), output.as_slice());
    assert_eq!(actual.raw_sha256(), crate::hashing::sha256_hex(&output));
}
/// Whole recorded output retains history/foreign rows/duplicate counts and contains no rationale.
#[test]
fn finite_dispositions_preserves_whole_history_counts_witnesses_and_no_rationale() {
    let expected = recorded();
    let output =
        encode_v3::dispositions(&expected, &mut ContractLedger::default(), &mut NoopControl)
            .unwrap();
    assert!(output.ends_with(b"}\n"));
    let actual = decode_d(&output).unwrap();
    assert!(actual.document() == &expected);
    assert_eq!(actual.document().responses.len(), 3);
    assert_eq!(actual.document().counts.response_files, 4);
    let text = std::str::from_utf8(&output).unwrap();
    assert!(!text.contains("rationale"));
    assert!(!text.contains("private reviewer evidence"));
    assert!(text.contains(r#""currentness":"unverified","closure_generation":null"#));
}
/// All five asserted dispositions survive plain decode, without deriving votes or approval.
#[test]
fn all_five_response_choices_roundtrip_without_disposition_omission() {
    for choice in [
        Disposition::Approve,
        Disposition::Reject,
        Disposition::RequestChanges,
        Disposition::Abstain,
        Disposition::Superseded,
    ] {
        let mut expected = response();
        expected.disposition = choice;
        if choice == Disposition::Abstain {
            expected.abstention_reason = Some("not-qualified".into());
            expected.rationale.clear();
        }
        if choice == Disposition::Superseded {
            expected.supersedes =
                Some(SupersessionReference { response_id: id(7), raw_sha256: "e".repeat(64) });
        }
        let output =
            encode_v3::response(&expected, &mut ContractLedger::default(), &mut NoopControl)
                .unwrap();
        assert!(decode_r(&output).unwrap().document() == &expected);
    }
}
/// Required nulls cannot silently become Serde defaults in any original response pin.
#[test]
fn response_required_nulls_and_every_pin_option_are_not_defaulted() {
    for field in ["abstention_reason", "proposed_edit", "supersedes"] {
        let mut v = rv();
        v.as_object_mut().unwrap().remove(field);
        assert_eq!(error(decode_r(&raw(&v))), ContractError::Invalid);
    }
    for field in ["schema_identity", "native_model", "native_root_uuid"] {
        let mut v = rv();
        v["source_pins"][0].as_object_mut().unwrap().remove(field);
        assert_eq!(error(decode_r(&raw(&v))), ContractError::Invalid);
    }
}
/// Whole recorded schemas retain required generation/nulls and closed nested row/seat/counter shapes.
#[test]
fn recorded_required_null_and_nested_unknown_fields_refuse() {
    let mut v = dv();
    v.as_object_mut().unwrap().remove("closure_generation");
    assert_eq!(error(decode_d(&raw(&v))), ContractError::Invalid);
    for pointer in [
        "",
        "/source_pins/0",
        "/responses/0",
        "/items/0",
        "/items/0/met_seats/0",
        "/items/0/unmet_seats/0",
        "/counts",
        "/counts/states",
    ] {
        let mut v = dv();
        v.pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("unexpected".into(), json!(true));
        assert_eq!(error(decode_d(&raw(&v))), ContractError::Invalid);
    }
}
/// Duplicate original keys, including escaped aliases, refuse in both new families.
#[test]
fn escaped_root_and_nested_duplicate_keys_are_not_last_wins() {
    for (v, response) in [(rv(), true), (dv(), false)] {
        let s = String::from_utf8(raw(&v)).unwrap();
        let duplicate = format!(r#"{{"queue\u005fid":"{}",{}"#, id(9), &s[1..]);
        let nested = s.replacen(
            r#""native_model":"catalog""#,
            r#""native_model":"catalog","native_model":"catalog""#,
            1,
        );
        for input in [duplicate, nested] {
            let result = if response {
                error(decode_r(input.as_bytes()))
            } else {
                error(decode_d(input.as_bytes()))
            };
            assert_eq!(result, ContractError::Invalid);
        }
    }
}
/// Both families refuse BOM, invalid UTF8, trailing JSON, malformed or empty input.
#[test]
fn malformed_original_syntax_never_becomes_empty_success() {
    for good in [raw(&rv()), raw(&dv())] {
        let mut bom = vec![0xef, 0xbb, 0xbf];
        bom.extend_from_slice(&good);
        let mut trailing = good;
        trailing.extend_from_slice(b" {}");
        for input in [bom, trailing, vec![0xff], b"{".to_vec(), vec![]] {
            assert_eq!(error(decode_r(&input)), ContractError::Invalid);
            assert_eq!(error(decode_d(&input)), ContractError::Invalid);
        }
    }
}
/// Old families, Queue and private locators cannot be cast through either operational decoder.
#[test]
fn separately_closed_markers_never_fall_back_to_another_envelope() {
    for marker in [
        "forge.review-response/1",
        "forge.review-response/2",
        "forge.review-dispositions/1",
        "forge.review-dispositions/2",
        "forge.review-queue/3",
        "forge.review-authoring-plan-inputs/1",
    ] {
        let mut v = rv();
        v["schema_version"] = json!(marker);
        assert_eq!(error(decode_r(&raw(&v))), ContractError::Invalid);
        let mut v = dv();
        v["schema_version"] = json!(marker);
        assert_eq!(error(decode_d(&raw(&v))), ContractError::Invalid);
    }
}
/// Unsupported edits win over ordinary schema errors only after complete strict duplicate parsing.
#[test]
fn unsupported_edit_precedes_schema_and_never_interprets_edit_payload() {
    let malformed = json!({"proposed_edit":{"anything":"private arbitrary edit"}});
    assert_eq!(error(decode_r(&raw(&malformed))), ContractError::UnsupportedEdit);
    assert_eq!(
        error(decode_r(br#"{"proposed_edit":{},"proposed_edit":null}"#)),
        ContractError::Invalid
    );
    let mut v = rv();
    v["proposed_edit"] = json!("arbitrary");
    assert_eq!(error(decode_r(&raw(&v))), ContractError::UnsupportedEdit);
}
/// Real calendar and nonnil canonical review UUID checks exceed lexical schema patterns.
#[test]
fn canonical_uuid_and_real_calendar_semantics_are_required() {
    for (pointer, value) in [
        ("/responded_at", "2026-02-30T00:00:00Z"),
        ("/response_id", "00000000-0000-0000-0000-000000000000"),
        ("/item_id", "AAAAAAAA-BBBB-4CCC-8DDD-EEEEEEEEEEEE"),
    ] {
        let mut v = rv();
        *v.pointer_mut(pointer).unwrap() = json!(value);
        assert_eq!(error(decode_r(&raw(&v))), ContractError::Invalid);
    }
    let mut v = dv();
    v["as_of"] = json!("2026-02-30T00:00:00Z");
    assert_eq!(error(decode_d(&raw(&v))), ContractError::Invalid);
    let mut v = dv();
    v["responses"][0]["response_id"] = json!("00000000-0000-0000-0000-000000000000");
    assert_eq!(error(decode_d(&raw(&v))), ContractError::Invalid);
}
/// Byte limits preserve full UTF8 evidence, while nonabstention whitespace is ordinary invalidity.
#[test]
fn rationale_uses_complete_utf8_bytes_and_nonblank_semantics() {
    let mut v = rv();
    v["rationale"] = json!("é".repeat(4096));
    assert!(decode_r(&raw(&v)).is_ok());
    v["rationale"] = json!("é".repeat(4097));
    assert_eq!(error(decode_r(&raw(&v))), ContractError::Capacity);
    let mut v = rv();
    v["rationale"] = json!(" \n\t");
    assert_eq!(error(decode_r(&raw(&v))), ContractError::Invalid);
}
/// Abstention and withdrawal require their full exact nullable correlations and no self-reference.
#[test]
fn abstention_reason_and_supersession_reference_correlations_are_closed() {
    for changes in [
        json!({"disposition":"abstain"}),
        json!({"abstention_reason":"not-qualified"}),
        json!({"disposition":"superseded"}),
        json!({"supersedes":{"response_id":id(7),"raw_sha256":"e".repeat(64)}}),
        json!({"disposition":"superseded","supersedes":{"response_id":id(1),"raw_sha256":"e".repeat(64)}}),
    ] {
        let mut v = rv();
        for (key, value) in changes.as_object().unwrap() {
            v[key] = value.clone();
        }
        assert_eq!(error(decode_r(&raw(&v))), ContractError::Invalid);
    }
}
/// Complete purpose tuples, mandatory singletons, Profile companions and gap-free ordinals remain live.
#[test]
fn complete_source_purpose_and_roster_rules_are_shared_without_authority() {
    let mut v = rv();
    v["source_pins"].as_array_mut().unwrap().remove(0);
    assert_eq!(error(decode_r(&raw(&v))), ContractError::Invalid);
    let mut v = rv();
    v["source_pins"].as_array_mut().unwrap().reverse();
    assert_eq!(error(decode_r(&raw(&v))), ContractError::Invalid);
    let mut r = response();
    r.source_pins.push(pin(SourceKindV3::MappingCollection, 2));
    r.source_pins.sort_by(|a, b| a.artifact_key.cmp(&b.artifact_key));
    assert_eq!(error(decode_r(&raw(&r))), ContractError::Invalid);
    let mut r = response();
    r.source_pins.iter_mut().find(|p| p.kind == SourceKindV3::Framework).unwrap().native_model =
        Some(NativeModelV3::Profile);
    assert_eq!(error(decode_r(&raw(&r))), ContractError::Invalid);
    r.source_pins.push(pin(SourceKindV3::ResolvedCatalog, 0));
    r.source_pins.sort_by(|a, b| a.artifact_key.cmp(&b.artifact_key));
    assert!(decode_r(&raw(&r)).is_ok());
    let mut v = dv();
    v["source_pins"][0]["native_model"] = json!("catalog");
    assert_eq!(error(decode_d(&raw(&v))), ContractError::Invalid);
}
/// Recorded UUID ordering/extent and occurrence conservation cannot be selected or deduplicated away.
#[test]
fn recorded_roster_order_extent_and_duplicate_counters_are_complete() {
    for changes in
        [json!({"response_files":3}), json!({"unique_responses":2}), json!({"exact_duplicates":2})]
    {
        let mut v = dv();
        for (key, value) in changes.as_object().unwrap() {
            v["counts"][key] = value.clone();
        }
        assert_eq!(error(decode_d(&raw(&v))), ContractError::Invalid);
    }
    let mut v = dv();
    v["responses"].as_array_mut().unwrap().reverse();
    assert_eq!(error(decode_d(&raw(&v))), ContractError::Invalid);
    let mut v = dv();
    v["responses"][1]["response_id"] = v["responses"][0]["response_id"].clone();
    assert_eq!(error(decode_d(&raw(&v))), ContractError::Invalid);
    for extent in [0, 1_048_577] {
        let mut v = dv();
        v["responses"][0]["byte_length"] = json!(extent);
        assert_eq!(error(decode_d(&raw(&v))), ContractError::Invalid);
    }
    let mut d = recorded();
    d.responses = (1..=33)
        .map(|n| RecordedResponse {
            response_id: id(n),
            raw_sha256: "a".repeat(64),
            byte_length: 1_048_576,
            item_key: "foreign-plan".into(),
            reviewer_key: "foreign".into(),
            reviewer_role: "approver".into(),
            disposition: Disposition::Approve,
            responded_at: "2026-10-01T00:00:00Z".into(),
            classification: ResponseClassification::Foreign,
        })
        .collect();
    assert_eq!(error(decode_d(&raw(&d))), ContractError::Capacity);
}
/// Every matching row and all withdrawn dissent remain referenced; foreign rows are retained whole.
#[test]
fn matching_history_and_all_dissent_are_preserved_with_foreign_rows() {
    assert_eq!(decode_d(&raw(&recorded())).unwrap().document().responses.len(), 3);
    for pointer in ["/items/0/response_ids", "/items/0/dissent_ids"] {
        let mut v = dv();
        v.pointer_mut(pointer).unwrap().as_array_mut().unwrap().pop();
        assert_eq!(error(decode_d(&raw(&v))), ContractError::Invalid);
    }
    let mut v = dv();
    v["items"][0]["response_ids"].as_array_mut().unwrap().push(json!(id(3)));
    assert_eq!(error(decode_d(&raw(&v))), ContractError::Invalid);
}
/// Recorded seat witnesses must reference Current Approve, distinct keys and every required seat.
#[test]
fn seat_witness_duplicates_nonapproval_and_missing_denominators_refuse() {
    for change in [json!({"classification":"late"}), json!({"disposition":"abstain"})] {
        let mut v = dv();
        for (key, value) in change.as_object().unwrap() {
            v["responses"][0][key] = value.clone();
        }
        assert_eq!(error(decode_d(&raw(&v))), ContractError::Invalid);
    }
    let mut v = dv();
    v["items"][0]["unmet_seats"][0]["ordinal"] = json!(0);
    assert_eq!(error(decode_d(&raw(&v))), ContractError::Invalid);
    let mut v = dv();
    let duplicate = v["items"][0]["met_seats"][0].clone();
    v["items"][0]["met_seats"].as_array_mut().unwrap().push(duplicate);
    assert_eq!(error(decode_d(&raw(&v))), ContractError::Invalid);
    let mut v = dv();
    v["items"][0]["required_seats"] = json!(3);
    assert_eq!(error(decode_d(&raw(&v))), ContractError::Invalid);
}
/// Recorded quorum admits withdrawn historical dissent but refuses current decisive/conflicted evidence.
#[test]
fn recorded_quorum_cannot_hide_blockers_or_decisive_current_dissent() {
    let mut v = dv();
    v["items"][0]["state"] = json!("quorum-met");
    v["items"][0]["required_seats"] = json!(1);
    v["items"][0]["unmet_seats"] = json!([]);
    v["items"][0]["blocking"] = json!(false);
    v["counts"]["states"]["in_review"] = json!(0);
    v["counts"]["states"]["quorum_met"] = json!(1);
    assert!(decode_d(&raw(&v)).is_ok());
    for class in ["current", "conflicted"] {
        let mut changed = v.clone();
        changed["responses"][1]["classification"] = json!(class);
        assert_eq!(error(decode_d(&raw(&changed))), ContractError::Invalid);
    }
    v["items"][0]["blocking"] = json!(true);
    assert_eq!(error(decode_d(&raw(&v))), ContractError::Invalid);
}
/// Every one of eight state counters is conserved against the exact one-item enum.
#[test]
fn all_eight_recorded_states_have_exact_one_item_counter_conservation() {
    for (state, counter) in [
        ("unassigned", "unassigned"),
        ("assigned", "assigned"),
        ("in-review", "in_review"),
        ("conflicted", "conflicted"),
        ("changes-requested", "changes_requested"),
        ("quorum-met", "quorum_met"),
        ("expired", "expired"),
        ("stale", "stale"),
    ] {
        let mut v = dv();
        v["items"][0]["state"] = json!(state);
        for x in v["counts"]["states"].as_object_mut().unwrap().values_mut() {
            *x = json!(0);
        }
        v["counts"]["states"][counter] = json!(1);
        if state == "quorum-met" {
            v["items"][0]["required_seats"] = json!(1);
            v["items"][0]["unmet_seats"] = json!([]);
            v["items"][0]["blocking"] = json!(false);
        }
        assert!(decode_d(&raw(&v)).is_ok());
        v["counts"]["states"][counter] = json!(0);
        assert_eq!(error(decode_d(&raw(&v))), ContractError::Invalid);
    }
}
/// Loaded currentness and digest labels are exact inert correlations, never a current owner.
#[test]
fn recorded_current_generation_null_correlation_is_plain_data_only() {
    let mut v = dv();
    v["currentness"] = json!("recorded-current");
    assert_eq!(error(decode_d(&raw(&v))), ContractError::Invalid);
    v["closure_generation"] = json!("f".repeat(64));
    assert!(decode_d(&raw(&v)).is_ok());
    v["currentness"] = json!("unverified");
    assert_eq!(error(decode_d(&raw(&v))), ContractError::Invalid);
}
/// Coherent private evidence, raw pin and recorded extent changes reach readback but never equal originals.
#[test]
fn full_typed_readback_observes_coherent_unused_and_private_operands() {
    let expected = response();
    let mut changed = rv();
    changed["rationale"] = json!("another full private rationale");
    assert!(decode_r(&raw(&changed)).is_ok());
    assert_eq!(
        error(encode_v3::readback_response(
            &expected,
            &raw(&changed),
            &mut ContractLedger::default(),
            &mut NoopControl
        )),
        ContractError::Binding
    );
    for field in ["raw_sha256", "byte_length"] {
        let mut changed = rv();
        changed["source_pins"][0][field] =
            if field == "raw_sha256" { json!("e".repeat(64)) } else { json!(18) };
        assert!(decode_r(&raw(&changed)).is_ok());
        assert_eq!(
            error(encode_v3::readback_response(
                &expected,
                &raw(&changed),
                &mut ContractLedger::default(),
                &mut NoopControl
            )),
            ContractError::Binding
        );
    }
    let expected = recorded();
    let mut changed = dv();
    changed["responses"][2]["byte_length"] = json!(203);
    assert!(decode_d(&raw(&changed)).is_ok());
    assert_eq!(
        error(encode_v3::readback_dispositions(
            &expected,
            &raw(&changed),
            &mut ContractLedger::default(),
            &mut NoopControl
        )),
        ContractError::Binding
    );
}
/// One actual accepted test controller exposes first interruption/failure and after-Ok stops.
struct Probe {
    /// Actual callback count on this same controller.
    calls: usize,
    /// One calibrated actual callback index.
    stop_at: usize,
    /// Interrupt, failure or after-Ok interruption mode.
    mode: u8,
    /// Actual first retained interruption.
    stopped: Option<Interruption>,
}
impl Probe {
    /// Construct one original test controller, never replace it during an operation.
    fn new(stop_at: usize, mode: u8) -> Self {
        Self { calls: 0, stop_at, mode, stopped: None }
    }
}
impl WorkControl for Probe {
    /// Return the selected actual first cause at a genuine producer callback.
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
    /// The original phase observes a stop even after an Ok checkpoint.
    fn interruption(&self) -> Option<Interruption> {
        self.stopped
    }
}
/// Ordinary parse/schema/edit refusals always cross the actual last same-control checkpoint.
#[test]
fn ordinary_failures_have_original_postfences_and_first_sticky_priority() {
    for (input, response_family) in [
        (b"{".to_vec(), true),
        (raw(&json!({})), true),
        (raw(&json!({"proposed_edit":{}})), true),
        (raw(&json!({})), false),
    ] {
        let mut record = Probe::new(usize::MAX, 0);
        let mut ledger = ContractLedger::default();
        let initial = if response_family {
            error(decode_response(&input, &mut ledger, &mut record))
        } else {
            error(decode_dispositions(&input, &mut ledger, &mut record))
        };
        assert!(matches!(initial, ContractError::Invalid | ContractError::UnsupportedEdit));
        for mode in 0..3 {
            let mut control = Probe::new(record.calls, mode);
            let mut ledger = ContractLedger::default();
            let expected = if mode == 1 {
                ContractError::ControlFailed
            } else {
                ContractError::Interrupted(Interruption::CancelRequested)
            };
            let actual = if response_family {
                error(decode_response(&input, &mut ledger, &mut control))
            } else {
                error(decode_dispositions(&input, &mut ledger, &mut control))
            };
            assert_eq!(actual, expected);
            let calls = control.calls;
            assert_eq!(
                error(decode_response(&raw(&response()), &mut ledger, &mut control)),
                expected
            );
            assert_eq!(control.calls, calls);
        }
    }
}
/// Complete successful decodes and real finite emission cannot skip their final original fence.
#[test]
fn successful_decode_and_finite_emission_have_exact_original_final_fences() {
    for family in 0..4 {
        let r = response();
        let d = recorded();
        let mut record = Probe::new(usize::MAX, 0);
        let mut ledger = ContractLedger::default();
        match family {
            0 => {
                decode_response(&raw(&r), &mut ledger, &mut record).unwrap();
            }
            1 => {
                decode_dispositions(&raw(&d), &mut ledger, &mut record).unwrap();
            }
            2 => {
                encode_v3::response(&r, &mut ledger, &mut record).unwrap();
            }
            _ => {
                encode_v3::dispositions(&d, &mut ledger, &mut record).unwrap();
            }
        }
        let mut control = Probe::new(record.calls, 2);
        let mut ledger = ContractLedger::default();
        let expected = ContractError::Interrupted(Interruption::CancelRequested);
        let actual = match family {
            0 => error(decode_response(&raw(&r), &mut ledger, &mut control)),
            1 => error(decode_dispositions(&raw(&d), &mut ledger, &mut control)),
            2 => error(encode_v3::response(&r, &mut ledger, &mut control)),
            _ => error(encode_v3::dispositions(&d, &mut ledger, &mut control)),
        };
        assert_eq!(actual, expected);
        let calls = control.calls;
        assert_eq!(error(encode_v3::response(&r, &mut ledger, &mut control)), expected);
        assert_eq!(control.calls, calls);
    }
}
/// Every failed strict response attempt remains in the actual unchanged aggregate registration count.
#[test]
fn failed_actual_response_attempts_are_not_refunded_or_renewed() {
    let mut ledger = ContractLedger::default();
    let mut control = Probe::new(usize::MAX, 0);
    for _ in 0..10_000 {
        assert_eq!(error(decode_response(b"{", &mut ledger, &mut control)), ContractError::Invalid);
    }
    assert_eq!(error(decode_response(b"{", &mut ledger, &mut control)), ContractError::Capacity);
    let calls = control.calls;
    control.stop_at = calls + 1;
    control.mode = 1;
    assert_eq!(
        error(decode_dispositions(&raw(&recorded()), &mut ledger, &mut control)),
        ContractError::Capacity
    );
    assert_eq!(control.calls, calls);
}
/// Complete raw ceilings and failed work preserve Capacity ahead of any later control/ordinary error.
#[test]
fn raw_caps_and_failed_work_preserve_capacity_without_limit_changes() {
    for (cap, response) in [(MAX_RESPONSE_BYTES, true), (MAX_RECORDED_BYTES, false)] {
        let input = vec![b' '; cap + 1];
        let mut ledger = ContractLedger::default();
        let mut control = Probe::new(usize::MAX, 0);
        let actual = if response {
            error(decode_response(&input, &mut ledger, &mut control))
        } else {
            error(decode_dispositions(&input, &mut ledger, &mut control))
        };
        assert_eq!(actual, ContractError::Capacity);
        let calls = control.calls;
        control.stop_at = calls + 1;
        control.mode = 1;
        assert_eq!(
            error(decode_response(b"{}", &mut ledger, &mut control)),
            ContractError::Capacity
        );
        assert_eq!(control.calls, calls);
    }
    let mut ledger = ContractLedger::default();
    let mut control = Probe::new(usize::MAX, 0);
    assert_eq!(error(decode_response(b"x", &mut ledger, &mut control)), ContractError::Invalid);
    assert_eq!(ledger.bytes(268_435_456), Err(ContractError::Capacity));
    let calls = control.calls;
    assert_eq!(
        error(encode_v3::dispositions(&recorded(), &mut ledger, &mut control)),
        ContractError::Capacity
    );
    assert_eq!(control.calls, calls);
}
