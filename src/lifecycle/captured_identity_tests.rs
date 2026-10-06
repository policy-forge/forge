//! Genuine file-oriented versus borrowed-byte native identity controls.
//! No captured lifecycle closure or approval/currentness factory is constructed.

use super::*;

/// Keep real file bytes and exact canonical path under one disposable fixture.
struct Original {
    /// Own the entire disposable file directory throughout both observations.
    _dir: tempfile::TempDir,
    /// Actual regular original used by maintained file-oriented fingerprinting.
    path: PathBuf,
    /// Actual normalized record directory consumed by the maintained wrapper.
    base: PathBuf,
}

impl Original {
    /// Persist a literal control original before either native identity observation.
    fn new(bytes: &[u8]) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().canonicalize().unwrap();
        let path = base.join("original.bin");
        std::fs::write(&path, bytes).unwrap();
        Self { _dir: dir, path, base }
    }

    /// Read genuine original bytes for the ordinary borrowed identity predicate.
    fn bytes(&self) -> Vec<u8> {
        std::fs::read(&self.path).unwrap()
    }
}

/// Empty, invalid UTF-8 and non-JSON opaque originals retain only their true raw hash.
#[test]
fn opaque_originals_preserve_empty_binary_and_non_json_bytes() {
    for bytes in [b"".as_slice(), &[0, 255, 128, 13, 10], b"{broken JSON"] {
        let original = Original::new(bytes);
        let observed = original.bytes();
        let identity = artifact_identity_from_bytes(&original.path, &observed, false).unwrap();
        let legacy = fingerprint(&original.path, &original.base, false).unwrap();
        assert_eq!(identity, (None, None));
        assert_eq!(legacy.path, "original.bin");
        assert_eq!(legacy.sha256, sha256_hex(bytes));
        assert_eq!((legacy.oscal_type, legacy.root_uuid), identity);
    }
}

/// All six genuine model roots keep identity-only behavior and original native UUID forms.
#[test]
fn generated_models_and_native_uuid_spellings_preserve_file_behavior() {
    let roots = [
        "catalog",
        "component-definition",
        "profile",
        "system-security-plan",
        "mapping-collection",
        "plan-of-action-and-milestones",
    ];
    let uuids = [
        "ABCDEFAB-1234-5678-9ABC-ABCDEFABCDEF",
        "abcdefab123456789abcabcdefabcdef",
        "{ABCDEFAB-1234-5678-9ABC-ABCDEFABCDEF}",
        "urn:uuid:ABCDEFAB-1234-5678-9ABC-ABCDEFABCDEF",
        "00000000-0000-0000-0000-000000000000",
        "ffffffff-ffff-ffff-ffff-ffffffffffff",
    ];
    for root in roots {
        for uuid in uuids {
            // Deliberately incomplete OSCAL body: maintained native identity checks
            // model/root/UUID alone, so a stronger schema claim would be incorrect.
            let bytes = serde_json::to_vec(&serde_json::json!({(root): {"uuid": uuid}})).unwrap();
            let original = Original::new(&bytes);
            let observed = original.bytes();
            let identity = artifact_identity_from_bytes(&original.path, &observed, true).unwrap();
            let legacy = fingerprint(&original.path, &original.base, true).unwrap();
            assert_eq!(identity, (Some(root.to_owned()), Some(uuid.to_owned())));
            assert_eq!(legacy.sha256, sha256_hex(&bytes));
            assert_eq!((legacy.oscal_type, legacy.root_uuid), identity);
        }
    }
}

/// Native generated parsing keeps its existing last-value duplicate behavior.
#[test]
fn generated_duplicate_uuid_preserves_maintained_serde_semantics() {
    let bytes = br#"{"catalog":{"uuid":"invalid","uuid":"ABCDEFAB-1234-5678-9ABC-ABCDEFABCDEF"}}"#;
    let original = Original::new(bytes);
    let identity = artifact_identity_from_bytes(&original.path, &original.bytes(), true).unwrap();
    let legacy = fingerprint(&original.path, &original.base, true).unwrap();
    assert_eq!(
        identity,
        (Some("catalog".to_owned()), Some("ABCDEFAB-1234-5678-9ABC-ABCDEFABCDEF".to_owned()))
    );
    assert_eq!((legacy.oscal_type, legacy.root_uuid), identity);
    assert_eq!(legacy.sha256, sha256_hex(bytes));
}

/// Ordinary parse/model/root/UUID failures retain exact private wrapper diagnostics.
#[test]
fn malformed_generated_originals_preserve_native_failure_details() {
    let cases: &[&[u8]] = &[
        b"",
        b"not JSON",
        b"null",
        b"{}",
        br#"{"catalog":null}"#,
        br#"{"catalog":{}}"#,
        br#"{"catalog":{"uuid":1}}"#,
        br#"{"catalog":{"uuid":"bad"}}"#,
        br#"{"catalog":{"uuid":"00000000-0000-0000-0000-000000000000"},"profile":{}}"#,
    ];
    for bytes in cases {
        let original = Original::new(bytes);
        let borrowed =
            artifact_identity_from_bytes(&original.path, &original.bytes(), true).unwrap_err();
        let legacy = fingerprint(&original.path, &original.base, true).unwrap_err();
        assert_eq!(borrowed.to_string(), legacy.to_string());
        assert!(borrowed.to_string().contains("generated artifact"));
    }
}

/// Borrowed identity accepts diagnostic labels as inert data and never opens their route.
#[test]
fn borrowed_predicate_never_opens_the_diagnostic_path() {
    let dir = tempfile::tempdir().unwrap();
    let unopened = dir.path().join("does-not-exist.json");
    let original = Original::new(br#"{"catalog":{"uuid":"00000000-0000-0000-0000-000000000000"}}"#);
    assert!(!unopened.exists());
    let identity = artifact_identity_from_bytes(&unopened, &original.bytes(), true).unwrap();
    assert_eq!(
        identity,
        (Some("catalog".to_owned()), Some("00000000-0000-0000-0000-000000000000".to_owned()))
    );
    assert!(!unopened.exists());
    assert!(fingerprint(&unopened, dir.path(), true).is_err());
}
