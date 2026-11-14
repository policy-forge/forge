//! Closed edit-capable /4 declaration shape controls: the optional inert edit slot and
//! the single closed patch format. These are asserted data shapes, never native proof.
use super::{PatchFormat, ProposedEdit, RESPONSE_VERSION};
use crate::hashing::sha256_hex;
use serde_json::{Value, json};

/// One inert proposed edit bound to a subject, with an exact patch digest.
fn edit() -> ProposedEdit {
    let patch = "# Replacement text\n";
    ProposedEdit {
        target_subject_sha256: "b".repeat(64),
        patch_format: PatchFormat::FullText,
        patch: patch.into(),
        patch_sha256: sha256_hex(patch.as_bytes()),
    }
}

/// The proposed edit serializes to the closed kebab-case envelope with all four fields.
#[test]
fn proposed_edit_serializes_closed_envelope() {
    let value = serde_json::to_value(edit()).unwrap();
    assert_eq!(value["patch_format"], "full-text");
    assert_eq!(value["target_subject_sha256"], "b".repeat(64));
    assert_eq!(value["patch"], "# Replacement text\n");
    assert_eq!(value["patch_sha256"], sha256_hex(b"# Replacement text\n"));
    assert_eq!(value.as_object().unwrap().len(), 4);
}

/// The edit slot is optional: null preserves the zero-edit behavior, present retains the patch.
#[test]
fn edit_slot_is_optional() {
    let null: Value = serde_json::from_str("null").unwrap();
    let absent: Option<ProposedEdit> = serde_json::from_value(null).unwrap();
    assert!(absent.is_none());
    let present: Option<ProposedEdit> =
        serde_json::from_value(serde_json::to_value(edit()).unwrap()).unwrap();
    assert_eq!(present.unwrap().patch, "# Replacement text\n");
}

/// The closed envelope refuses unknown fields; no detached edit channel is admitted.
#[test]
fn proposed_edit_denies_unknown_fields() {
    let value = json!({
        "target_subject_sha256": "b".repeat(64),
        "patch_format": "full-text",
        "patch": "# text\n",
        "patch_sha256": sha256_hex(b"# text\n"),
        "apply_now": true
    });
    assert!(serde_json::from_value::<ProposedEdit>(value).is_err());
}

/// The patch format is a single closed enum; no multi-format registry is admitted.
#[test]
fn patch_format_is_closed() {
    assert!(serde_json::from_value::<PatchFormat>(json!("full-text")).is_ok());
    assert!(serde_json::from_value::<PatchFormat>(json!("unified-diff")).is_err());
}

/// The response version token is the separately closed /4 family, never /3.
#[test]
fn response_version_is_closed_v4() {
    assert_eq!(RESPONSE_VERSION, "forge.review-response/4");
}
