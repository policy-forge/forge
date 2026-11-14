//! Edit-capable /4 finite output and exact readback controls: a proposed edit survives
//! emit/readback/equality byte-exactly and never leaks into the public recorded bundle.
use super::{compare_responses, readback_response, response};
use crate::hashing::sha256_hex;
use crate::review::decode::{ContractError, ContractLedger};
use crate::review::wire_v3::NativeModelV3;
use crate::review::wire_v4::{
    ADAPTER, Disposition, DomainV3, IDENTITY_DISCLAIMER, PatchFormat, ProposedEdit,
    RequestedAction, ResponseDocumentV4, SourceKindV3, SourcePinV3,
};
use crate::workspace::preparation::NoopControl;

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

/// Emit, strict readback and exhaustive typed equality retain the exact inert edit.
#[test]
fn emit_readback_equality_retains_edit() {
    let document = document();
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let bytes = response(&document, &mut ledger, &mut control).expect("finite output");
    readback_response(&document, &bytes, &mut ledger, &mut control).expect("strict readback");
    let read = crate::review::decode_v4::decode_response(&bytes, &mut ledger, &mut control)
        .expect("decode for field inspection");
    assert_eq!(read.document().proposed_edit.as_ref().unwrap().patch, "# Replacement text\n");
}

/// A changed patch byte breaks exhaustive equality; no silent reserialization is admitted.
#[test]
fn changed_patch_breaks_equality() {
    let document = document();
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let mut changed: ResponseDocumentV4 =
        serde_json::from_value(serde_json::to_value(&document).unwrap()).unwrap();
    changed.proposed_edit.as_mut().unwrap().patch = "# Different text\n".into();
    let result = compare_responses(&document, &changed, &mut ledger, &mut control);
    assert!(matches!(result, Err(ContractError::Binding)));
}
