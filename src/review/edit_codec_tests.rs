//! Genuine plain edit-capable /4 codec controls: proposed-edit round-trip, structural
//! refusal and the new invalid-edit recorded classification. Never native proof.
use super::{decode_dispositions, decode_response};
use crate::hashing::sha256_hex;
use crate::review::decode::{ContractError, ContractLedger};
use crate::review::encode_v4;
use crate::review::wire_v3::NativeModelV3;
use crate::review::wire_v4::{
    ADAPTER, Disposition, DispositionCounts, DispositionsDocumentV4, DomainV3, IDENTITY_DISCLAIMER,
    ItemDisposition, ItemState, PatchFormat, ProposedEdit, RecordedCurrentness, RecordedResponse,
    RequestedAction, ResponseClassification, ResponseDocumentV4, SourceKindV3, SourcePinV3,
    StateCounts, UnmetSeat,
};
use crate::workspace::preparation::NoopControl;
use serde_json::Value;

/// Canonical plain review identities, independent of native identifiers.
fn id(n: u8) -> String {
    format!("{n:08x}-1111-4111-8111-111111111111")
}
/// One ordinary source pin; a digest is asserted data only, never native truth.
fn pin(kind: SourceKindV3, _ordinal: usize) -> SourcePinV3 {
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
        SourceKindV3::StoredPlan => (
            "authoring:plan".into(),
            Some("forge.authoring-plan/1"),
            "forge.authoring-plan-complete-equality/1",
            None,
            None,
        ),
        _ => unreachable!("fixture uses only the six ordinary purposes"),
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
/// The full ordinary six-purpose roster, sorted by artifact key.
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
/// One inert proposed edit bound to the fixture subject, with an exact patch digest.
fn edit(patch: &str) -> ProposedEdit {
    ProposedEdit {
        target_subject_sha256: "b".repeat(64),
        patch_format: PatchFormat::FullText,
        patch: patch.into(),
        patch_sha256: sha256_hex(patch.as_bytes()),
    }
}
/// Complete private edit-capable response; the proposed edit is optional inert data.
fn response(proposed_edit: Option<ProposedEdit>) -> ResponseDocumentV4 {
    ResponseDocumentV4 {
        schema_version: "forge.review-response/4".into(),
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
        proposed_edit,
        supersedes: None,
    }
}
/// Complete test-only serialization; production emit/readback has separate controls.
fn raw<T: serde::Serialize>(value: &T) -> Vec<u8> {
    let mut bytes = serde_json::to_vec_pretty(value).unwrap();
    bytes.push(b'\n');
    bytes
}
/// Inspect an ordinary error without Debug on private owners.
fn refusal<T>(value: Result<T, ContractError>) -> ContractError {
    value.err().expect("expected refusal")
}

/// A valid proposed edit round-trips byte-exactly through emit, strict decode and equality.
#[test]
fn valid_proposed_edit_round_trips() {
    let document = response(Some(edit("# Replacement plan text\n")));
    let bytes = encode_v4::response(&document, &mut ContractLedger::default(), &mut NoopControl)
        .expect("finite edit-capable response output");
    let read = decode_response(&bytes, &mut ContractLedger::default(), &mut NoopControl)
        .expect("strict edit-capable readback");
    assert!(read.document() == &document);
    assert_eq!(read.raw(), bytes.as_slice());
    assert_eq!(read.raw_sha256(), sha256_hex(&bytes));
    let edit = read.document().proposed_edit.as_ref().expect("edit retained");
    assert_eq!(edit.patch, "# Replacement plan text\n");
    assert_eq!(edit.patch_sha256, sha256_hex(edit.patch.as_bytes()));
}

/// The null edit slot preserves the zero-edit behavior and still round-trips.
#[test]
fn null_proposed_edit_round_trips() {
    let document = response(None);
    let bytes =
        encode_v4::response(&document, &mut ContractLedger::default(), &mut NoopControl).unwrap();
    let read = decode_response(&bytes, &mut ContractLedger::default(), &mut NoopControl).unwrap();
    assert!(read.document() == &document);
    assert!(read.document().proposed_edit.is_none());
}

/// A patch digest that does not match the exact patch bytes still decodes: semantic edit
/// validity is a per-response merge classification (invalid-edit), not a decode refusal.
#[test]
fn wrong_patch_digest_decodes_for_merge_classification() {
    let mut document = response(Some(edit("# Replacement plan text\n")));
    document.proposed_edit.as_mut().unwrap().patch_sha256 = "f".repeat(64);
    let bytes = raw(&document);
    let read = decode_response(&bytes, &mut ContractLedger::default(), &mut NoopControl)
        .expect("malformed edit decodes; merge classifies it invalid-edit");
    assert!(read.document() == &document);
}

/// An empty patch is structurally refused; the envelope requires nonempty bounded text.
#[test]
fn empty_patch_is_refused() {
    let document = response(Some(edit("")));
    let bytes = raw(&document);
    assert!(matches!(
        refusal(decode_response(&bytes, &mut ContractLedger::default(), &mut NoopControl)),
        ContractError::Invalid
    ));
}

/// A patch beyond the 64 KiB bound is refused before any allocation grows.
#[test]
fn oversized_patch_is_refused() {
    let document = response(Some(edit(&"x".repeat(64 * 1024 + 1))));
    let bytes = raw(&document);
    assert!(matches!(
        refusal(decode_response(&bytes, &mut ContractLedger::default(), &mut NoopControl)),
        ContractError::Invalid | ContractError::Capacity
    ));
}

/// A wrong target hash still decodes: the binding is a per-response merge classification
/// (AC-5), not a structural decode refusal. The patch is retained as inert evidence.
#[test]
fn wrong_target_hash_decodes_for_merge_classification() {
    let mut document = response(Some(edit("# Replacement plan text\n")));
    document.proposed_edit.as_mut().unwrap().target_subject_sha256 = "e".repeat(64);
    let bytes = raw(&document);
    let read = decode_response(&bytes, &mut ContractLedger::default(), &mut NoopControl)
        .expect("wrong-target edit decodes; merge classifies it stale");
    assert_eq!(
        read.document().proposed_edit.as_ref().unwrap().target_subject_sha256,
        "e".repeat(64)
    );
}

/// One recorded bundle carrying the new invalid-edit classification round-trips exactly.
fn recorded_invalid_edit() -> DispositionsDocumentV4 {
    DispositionsDocumentV4 {
        schema_version: "forge.review-dispositions/4".into(),
        identity_disclaimer: IDENTITY_DISCLAIMER.into(),
        queue_id: id(9),
        queue_raw_sha256: "a".repeat(64),
        as_of: "2026-10-01T00:00:03Z".into(),
        currentness: RecordedCurrentness::Unverified,
        closure_generation: None,
        source_pins: pins(),
        responses: vec![RecordedResponse {
            response_id: id(1),
            raw_sha256: "a".repeat(64),
            byte_length: 200,
            item_key: "whole-plan".into(),
            reviewer_key: "alice".into(),
            reviewer_role: "approver".into(),
            disposition: Disposition::Approve,
            responded_at: "2026-10-01T00:00:00Z".into(),
            classification: ResponseClassification::InvalidEdit,
        }],
        items: vec![ItemDisposition {
            item_key: "whole-plan".into(),
            item_id: id(8),
            state: ItemState::Assigned,
            reason_codes: vec!["asserted-identity-only".into()],
            required_seats: 1,
            met_seats: vec![],
            unmet_seats: vec![UnmetSeat { role_key: "approver".into(), ordinal: 0 }],
            response_ids: vec![id(1)],
            dissent_ids: vec![],
            blocking: false,
        }],
        counts: DispositionCounts {
            items: 1,
            response_files: 1,
            unique_responses: 1,
            exact_duplicates: 0,
            states: StateCounts {
                unassigned: 0,
                assigned: 1,
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

/// The invalid-edit classification is a closed recorded value that survives emit/readback.
#[test]
fn invalid_edit_classification_round_trips() {
    let document = recorded_invalid_edit();
    let bytes =
        encode_v4::dispositions(&document, &mut ContractLedger::default(), &mut NoopControl)
            .expect("finite recorded output");
    let read = decode_dispositions(&bytes, &mut ContractLedger::default(), &mut NoopControl)
        .expect("strict recorded readback");
    assert!(read.document() == &document);
    assert_eq!(read.document().responses[0].classification, ResponseClassification::InvalidEdit);
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["responses"][0]["classification"], "invalid-edit");
}

/// The /4 response decoder refuses a /3 version token; the families stay separately closed.
#[test]
fn response_version_is_separately_closed() {
    let mut document = response(None);
    document.schema_version = "forge.review-response/3".into();
    let bytes = raw(&document);
    assert!(matches!(
        refusal(decode_response(&bytes, &mut ContractLedger::default(), &mut NoopControl)),
        ContractError::Invalid
    ));
}
