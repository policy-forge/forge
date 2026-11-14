//! Edit-capable /4 decode admission controls: the closed schema shape gates intake while
//! semantic edit validity remains a per-response merge classification. Never native proof.
use super::{decode_response, response};
use crate::hashing::sha256_hex;
use crate::review::decode::{ContractError, ContractLedger};
use crate::review::wire_v3::NativeModelV3;
use crate::review::wire_v4::{
    ADAPTER, Disposition, DomainV3, IDENTITY_DISCLAIMER, PatchFormat, ProposedEdit,
    RequestedAction, ResponseDocumentV4, SourceKindV3, SourcePinV3,
};
use crate::workspace::preparation::NoopControl;
use serde_json::Value;

/// Canonical plain review identity.
fn id(n: u8) -> String {
    format!("{n:08x}-1111-4111-8111-111111111111")
}
/// One ordinary source pin; a digest is asserted data only.
fn pin(kind: SourceKindV3, key: &str, schema: Option<&str>, profile: &str) -> SourcePinV3 {
    SourcePinV3 {
        artifact_key: key.into(),
        kind,
        raw_sha256: "a".repeat(64),
        byte_length: 17,
        schema_identity: schema.map(str::to_owned),
        validation_profile: profile.into(),
        native_model: matches!(kind, SourceKindV3::Framework).then_some(NativeModelV3::Catalog),
        native_root_uuid: matches!(kind, SourceKindV3::Framework)
            .then(|| "AAAAAAAA-BBBB-4CCC-8DDD-EEEEEEEEEEEE".to_owned()),
    }
}
/// The full ordinary six-purpose roster, sorted by artifact key.
fn pins() -> Vec<SourcePinV3> {
    let mut values = vec![
        pin(
            SourceKindV3::AuthorProject,
            "authoring:project",
            Some("forge.author-project/1"),
            "forge.author-project-intrinsic/1",
        ),
        pin(
            SourceKindV3::AuthoringPack,
            "authoring:pack",
            Some("forge.authoring-pack/1"),
            "forge.authoring-pack-intrinsic/1",
        ),
        pin(
            SourceKindV3::GapReport,
            "authoring:gap-report",
            Some("forge.applicability-report/1"),
            "forge.authoring-gap-report-complete/1",
        ),
        pin(
            SourceKindV3::ApplicabilityManifest,
            "authoring:applicability",
            Some("forge.applicability/1"),
            "forge.authoring-applicability-intrinsic/1",
        ),
        pin(
            SourceKindV3::Framework,
            "authoring:framework",
            None,
            "forge.authoring-framework-native/1",
        ),
        pin(
            SourceKindV3::StoredPlan,
            "authoring:plan",
            Some("forge.authoring-plan/1"),
            "forge.authoring-plan-complete-equality/1",
        ),
    ];
    values.sort_by(|a, b| a.artifact_key.cmp(&b.artifact_key));
    values
}
/// Complete private edit-capable response with a valid inert proposed edit.
fn document() -> ResponseDocumentV4 {
    let patch = "# Replacement text\n";
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
        proposed_edit: Some(ProposedEdit {
            target_subject_sha256: "b".repeat(64),
            patch_format: PatchFormat::FullText,
            patch: patch.into(),
            patch_sha256: sha256_hex(patch.as_bytes()),
        }),
        supersedes: None,
    }
}
/// Complete test-only serialization.
fn raw(value: &ResponseDocumentV4) -> Vec<u8> {
    let mut bytes = serde_json::to_vec_pretty(value).unwrap();
    bytes.push(b'\n');
    bytes
}
/// Inspect an ordinary error without Debug on private owners.
fn refusal<T>(value: Result<T, ContractError>) -> ContractError {
    value.err().expect("expected refusal")
}

/// The typed validator admits a complete valid edit-capable response.
#[test]
fn valid_response_passes_typed_validator() {
    response(&document(), &mut ContractLedger::default(), &mut NoopControl)
        .expect("valid edit-capable response");
}

/// The closed schema refuses an unknown edit field before Serde admits the document.
#[test]
fn unknown_edit_field_is_refused_by_schema() {
    let mut value: Value = serde_json::to_value(document()).unwrap();
    value["proposed_edit"]["apply_now"] = Value::Bool(true);
    let mut bytes = serde_json::to_vec_pretty(&value).unwrap();
    bytes.push(b'\n');
    assert!(matches!(
        refusal(decode_response(&bytes, &mut ContractLedger::default(), &mut NoopControl)),
        ContractError::Invalid
    ));
}

/// The closed schema refuses an empty patch (shape), independent of merge classification.
#[test]
fn empty_patch_is_refused_by_schema() {
    let mut document = document();
    document.proposed_edit.as_mut().unwrap().patch = String::new();
    assert!(matches!(
        refusal(decode_response(&raw(&document), &mut ContractLedger::default(), &mut NoopControl)),
        ContractError::Invalid
    ));
}

/// A wrong patch digest passes the closed shape and is left for per-response merge classification.
#[test]
fn wrong_patch_digest_passes_shape_for_merge() {
    let mut document = document();
    document.proposed_edit.as_mut().unwrap().patch_sha256 = "f".repeat(64);
    let bytes = raw(&document);
    let read = decode_response(&bytes, &mut ContractLedger::default(), &mut NoopControl)
        .expect("shape admits; merge classifies invalid-edit");
    assert!(read.document() == &document);
}
