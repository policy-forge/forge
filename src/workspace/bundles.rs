//! Metadata-only workspace bundle previews and registered-resource comparisons.
//!
//! No supplied path is opened here. Fingerprint agreement is an observation of
//! the captured registered inventory, not import readiness or domain approval.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::contract::{self, ApiMajor, Error, Result};
use super::index::Index;
use super::services::{Item, Snapshot};

/// Maximum encoded bundle and complete verification request size.
const MAX_BUNDLE_BYTES: usize = 1024 * 1024;
/// Existing index ceiling; queries never apply the smaller effect-input ceiling.
const MAX_RESOURCES: usize = 1000;
/// Existing per-resource capture limit, including declared bundle lengths.
const MAX_RESOURCE_BYTES: usize = 10 * 1024 * 1024;
/// Declared unique resource bytes plus normalized index may not exceed this bound.
const MAX_DECLARED_BYTES: usize = 50 * 1024 * 1024;

/// Closed metadata wire model; source content and approval state are absent.
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Bundle {
    /// Exact identifier of this narrow resource-index bundle contract.
    pub(crate) schema_version: String,
    /// Fixed metadata-only profile, never a source-content opt-in.
    pub(crate) content_profile: String,
    /// Exact same-version closed index; original /1 role interpretation remains unchanged.
    pub(crate) index: Index,
    /// Hash of normalized `Index::bytes`, not original index formatting.
    pub(crate) index_sha256: String,
    /// One exact-byte fingerprint for every index key, in index order.
    pub(crate) pins: Vec<Pin>,
}

/// Closed exact-byte pin for one explicitly indexed resource.
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Pin {
    /// Exact index key; the index supplies role and portable path.
    pub(crate) key: String,
    /// Lowercase SHA256 of original resource bytes.
    pub(crate) sha256: String,
    /// Exact declared resource length, independently compared to captured bytes.
    pub(crate) size_bytes: usize,
}

/// Return every registered resource's metadata without minting an effect receipt.
///
/// # Errors
/// Returns the safe setup error for an absent index, or a contract/limit error
/// for inconsistent captured metadata or an oversized complete bundle.
pub(crate) fn preview(snapshot: &Snapshot) -> Result<Value> {
    preview_for_api(snapshot, ApiMajor::V1)
}

/// Inspect complete same-version metadata for the explicitly admitted API major.
/// V1 preserves only index1/bundle1; V2 reads index1/2 without granting import authority.
pub(crate) fn preview_for_api(snapshot: &Snapshot, api_major: ApiMajor) -> Result<Value> {
    admit_snapshot_major(snapshot, api_major)?;
    if !snapshot.index_present {
        return Err(Error::new("not-found", "The workspace resource index is not present.", false));
    }
    let current = current_items(snapshot)?;
    let pins = snapshot
        .index
        .resources
        .iter()
        .map(|resource| {
            let item = current.get(resource.key.as_str()).ok_or_else(Error::invalid)?;
            Ok(Pin {
                key: resource.key.clone(),
                sha256: item.captured.sha256.clone(),
                size_bytes: item.captured.bytes.len(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let bundle = Bundle {
        schema_version: bundle_version(&snapshot.index)?.into(),
        content_profile: "index-and-hashes".into(),
        index: snapshot.index.clone(),
        index_sha256: normalized_index_hash(&snapshot.index)?,
        pins,
    };
    let bytes = contract::encode(&bundle, MAX_BUNDLE_BYTES, false)?;
    let value = contract::parse(&bytes, MAX_BUNDLE_BYTES, 64 * 1024)?;
    decode_bundle_for_api(&value, api_major)?;
    let reply = json!({
        "bundle": value,
        "snapshot_version": snapshot.version,
        "source_index_present": true,
        "included_metadata": ["project-label", "resource-keys", "typed-roles",
            "project-relative-paths", "sha256-fingerprints", "byte-lengths"],
        "source_content_included": false,
    });
    contract::validate_for(api_major, "ProjectBundlePreview", &reply)?;
    Ok(reply)
}

/// Encode the complete validated metadata profile for a selected-major export.
///
/// This consumes the same preview/decoder checks without source content or any
/// supplied-path reads. Artifact bytes are compact JSON; the nested index hash
/// still identifies deterministic `Index::bytes` including its newline.
pub(crate) fn encode_metadata_for_api(snapshot: &Snapshot, api_major: ApiMajor) -> Result<Vec<u8>> {
    let preview = preview_for_api(snapshot, api_major)?;
    contract::encode(&preview["bundle"], MAX_BUNDLE_BYTES, false)
}

/// Compare all supplied expected pins with current registered captures only.
///
/// Unregistered paths are reported without opening them. A matching stale or
/// invalid registered resource retains that observed validation state. Extras
/// in the current index remain present and are counted separately.
///
/// # Errors
/// Returns safe contract/limit errors for malformed direct Value requests or
/// inconsistent snapshots. Fingerprint mismatches are complete result rows.
pub(crate) fn verify_registered(snapshot: &Snapshot, request: &Value) -> Result<Value> {
    verify_registered_for_api(snapshot, request, ApiMajor::V1)
}

/// Compare a selected-major bundle only with captured registrations, preserving version-pair closure.
/// Cross-version V2 comparisons report extras and index inequality without reading supplied paths.
pub(crate) fn verify_registered_for_api(
    snapshot: &Snapshot,
    request: &Value,
    api_major: ApiMajor,
) -> Result<Value> {
    admit_snapshot_major(snapshot, api_major)?;
    let bytes = contract::encode(request, MAX_BUNDLE_BYTES, false)?;
    let request = contract::parse(&bytes, MAX_BUNDLE_BYTES, 64 * 1024)?;
    contract::validate_for(api_major, "WorkspaceBundleVerificationRequest", &request)?;
    let bundle = decode_bundle_for_api(&request["bundle"], api_major)?;
    let current = current_items(snapshot)?;
    let expected_keys =
        bundle.index.resources.iter().map(|r| r.key.as_str()).collect::<BTreeSet<_>>();
    let mut items = Vec::with_capacity(bundle.pins.len());
    for (registration, pin) in bundle.index.resources.iter().zip(&bundle.pins) {
        let observed = current.get(registration.key.as_str()).copied();
        let mut reasons = Vec::new();
        let (status, validation_state) = if let Some(item) = observed {
            if item.registration.role != registration.role
                || item.registration.path != registration.path
            {
                reasons.push("registration-conflict");
            } else {
                if item.captured.sha256 != pin.sha256 {
                    reasons.push("sha256-mismatch");
                }
                if item.captured.bytes.len() != pin.size_bytes {
                    reasons.push("size-mismatch");
                }
            }
            (
                if reasons.is_empty() { "matched" } else { "mismatched" },
                item.metadata["validation_state"].as_str().ok_or_else(Error::invalid)?,
            )
        } else {
            reasons.push("registration-not-found");
            ("not-registered", "not-registered")
        };
        items.push(json!({"key":pin.key,"status":status,"reason_codes":reasons,
            "observed_resource_validation_state":validation_state}));
    }
    let matched = items.iter().filter(|item| item["status"] == "matched").count();
    let unregistered = items.iter().filter(|item| item["status"] == "not-registered").count();
    let mismatched = items.iter().filter(|item| item["status"] == "mismatched").count();
    let total = matched
        .checked_add(unregistered)
        .and_then(|n| n.checked_add(mismatched))
        .ok_or_else(Error::invalid)?;
    if total != bundle.index.resources.len() {
        return Err(Error::invalid());
    }
    let index_matches =
        snapshot.index_present && normalized_index_hash(&snapshot.index)? == bundle.index_sha256;
    let reply = json!({
        "scope":"registered-fingerprints-only", "snapshot_version":snapshot.version,
        "source_index_present":snapshot.index_present,
        "state":if !snapshot.index_present {"missing-index"} else if matched == total {"matched"} else {"mismatched"},
        "current_resources":snapshot.items.len(),
        "current_only_resources":current.keys().filter(|key| !expected_keys.contains(**key)).count(),
        "expected_index_matches_current":index_matches,
        "expected_resources":total, "matched_resources":matched,
        "unregistered_resources":unregistered, "mismatched_resources":mismatched,
        "items":items, "source_content_included":false,
    });
    contract::validate_for(api_major, "ProjectBundleVerification", &reply)?;
    Ok(reply)
}

/// Validate intrinsic bundle shape, normalized index identity and complete pins.
///
/// Uses the normative API validator and existing index parser, with no separate
/// JSON Schema validator or supplied-path filesystem access.
fn decode_bundle(value: &Value) -> Result<Bundle> {
    decode_bundle_with_contract(value, ApiMajor::V1)
}

/// Consume the original closed decoder for V1 and the explicit version-paired decoder for V2.
pub(crate) fn decode_bundle_for_api(value: &Value, api_major: ApiMajor) -> Result<Bundle> {
    match api_major {
        ApiMajor::V1 => decode_bundle(value),
        ApiMajor::V2 => decode_bundle_with_contract(value, ApiMajor::V2),
    }
}

/// Revalidate the entire supplied JSON tree and intrinsic bounds under its selected API contract.
fn decode_bundle_with_contract(value: &Value, api_major: ApiMajor) -> Result<Bundle> {
    let bytes = contract::encode(value, MAX_BUNDLE_BYTES, false)?;
    let value = contract::parse(&bytes, MAX_BUNDLE_BYTES, 64 * 1024)?;
    contract::validate_for(api_major, "WorkspaceIndexBundle", &value)?;
    let bundle: Bundle = serde_json::from_value(value).map_err(|_| Error::invalid())?;
    let index_bytes = bundle.index.bytes()?;
    if bundle.index.resources.len() > MAX_RESOURCES
        || bundle.pins.len() != bundle.index.resources.len()
        || bundle.schema_version != bundle_version(&bundle.index)?
        || bundle.content_profile != "index-and-hashes"
        || bundle.index_sha256 != crate::hashing::sha256_hex(&index_bytes)
    {
        return Err(Error::invalid());
    }
    let mut declared = index_bytes.len();
    for (resource, pin) in bundle.index.resources.iter().zip(&bundle.pins) {
        if pin.key != resource.key || pin.size_bytes > MAX_RESOURCE_BYTES {
            return Err(Error::invalid());
        }
        declared = declared.checked_add(pin.size_bytes).ok_or_else(Error::invalid)?;
        if declared > MAX_DECLARED_BYTES {
            return Err(Error::new(
                "payload-too-large",
                "The bundle declarations exceed the input limit.",
                false,
            ));
        }
    }
    Ok(bundle)
}

/// Reject unsupported current index versions even for direct internal query consumers.
/// This supplements, rather than replaces, the HTTP pre-resource-read capture fence.
fn admit_snapshot_major(snapshot: &Snapshot, api_major: ApiMajor) -> Result<()> {
    match (api_major, snapshot.index.schema_version.as_str()) {
        (ApiMajor::V1, "forge.workspace/1")
        | (ApiMajor::V2, "forge.workspace/1" | "forge.workspace/2") => Ok(()),
        _ => Err(Error::invalid()),
    }
}

/// Choose the only compatible metadata bundle version for a validated index.
/// This is no content opt-in, import readiness or interpretation of new roles under /1.
fn bundle_version(index: &Index) -> Result<&'static str> {
    match index.schema_version.as_str() {
        "forge.workspace/1" => Ok("forge.workspace-index-bundle/1"),
        "forge.workspace/2" => Ok("forge.workspace-index-bundle/2"),
        _ => Err(Error::invalid()),
    }
}

/// Hash the validated, deterministic version-specific index encoding including newline.
fn normalized_index_hash(index: &Index) -> Result<String> {
    Ok(crate::hashing::sha256_hex(&index.bytes()?))
}

/// Check captured registration consistency before allocating the key lookup.
///
/// This guards direct service callers while retaining Snapshot's observed state;
/// it neither recomputes domain approval nor opens any resource.
fn current_items(snapshot: &Snapshot) -> Result<BTreeMap<&str, &Item>> {
    if snapshot.index.resources.len() > MAX_RESOURCES
        || snapshot.items.len() != snapshot.index.resources.len()
        || (!snapshot.index_present && !snapshot.items.is_empty())
    {
        return Err(Error::invalid());
    }
    snapshot.index.bytes()?;
    let mut current = BTreeMap::new();
    for (resource, item) in snapshot.index.resources.iter().zip(&snapshot.items) {
        if resource.key != item.registration.key
            || resource.role != item.registration.role
            || resource.path != item.registration.path
            || !matches!(
                item.metadata["validation_state"].as_str(),
                Some("valid" | "invalid" | "stale")
            )
        {
            return Err(Error::invalid());
        }
        current.insert(resource.key.as_str(), item);
    }
    Ok(current)
}

#[cfg(test)]
mod tests {
    use super::super::index::{Resource, Role};
    use super::super::root::Root;
    use super::*;

    /// Actual confined snapshot and its synthetic-development fixture directory.
    struct Fixture {
        /// Retains the temporary source/index files for preservation assertions.
        project: tempfile::TempDir,
        /// Actual production Snapshot captured from the explicit fixture index.
        snapshot: Snapshot,
    }

    /// Build an actual explicitly registered Markdown snapshot without inference.
    fn fixture(count: usize, index_present: bool) -> Fixture {
        let project = tempfile::tempdir().expect("synthetic project directory");
        let mut index = Index::empty();
        index.label = "Synthetic metadata fixture".into();
        for n in 0..count {
            let path = format!("policy-{n:04}.md");
            std::fs::write(
                project.path().join(&path),
                format!("# Policy {n}\n\n## Rules\nPRIVATE SOURCE SENTINEL {n}\n"),
            )
            .expect("write synthetic source");
            index.resources.push(Resource {
                key: format!("policy-{n:04}"),
                role: Role::PolicySource,
                path,
            });
        }
        if index_present {
            std::fs::write(
                project.path().join(super::super::index::INDEX_PATH),
                index.bytes().expect("valid fixture index"),
            )
            .expect("write explicit index");
        }
        let root = Root::open(project.path()).expect("confined fixture root");
        let snapshot = Snapshot::capture(&root).expect("actual production snapshot");
        Fixture { project, snapshot }
    }

    /// Obtain the intrinsic bundle from the actual preview service.
    fn bundle(snapshot: &Snapshot) -> Value {
        preview(snapshot).expect("metadata preview")["bundle"].clone()
    }

    /// Refresh only the normalized index pin after an intentional fixture edit.
    fn refresh_index_hash(value: &mut Value) {
        let index: Index =
            serde_json::from_value(value["index"].clone()).expect("fixture index shape");
        value["index_sha256"] = json!(normalized_index_hash(&index).expect("valid fixture index"));
    }

    /// Assert the stable workspace error contract rather than any nonzero failure.
    fn expect_error(result: Result<Value>, code: &str) {
        let error = result.expect_err("must reject before a result");
        assert_eq!(error.code, code);
        assert!(!error.retryable);
        assert!(!error.message.contains("PRIVATE SOURCE SENTINEL"));
    }

    /// Preserve complete ordered metadata for zero,101 and1,000 registrations.
    #[test]
    fn complete_query_denominators_do_not_use_the_effect_input_cap() {
        for count in [0, 101, 1000] {
            let f = fixture(count, true);
            let value = bundle(&f.snapshot);
            let pins = value["pins"].as_array().expect("complete pins");
            assert_eq!(pins.len(), count);
            let comparison =
                verify_registered(&f.snapshot, &json!({"bundle":value})).expect("comparison");
            assert_eq!(comparison["state"], "matched");
            assert_eq!(comparison["expected_resources"], json!(count));
            assert_eq!(comparison["matched_resources"], json!(count));
            assert_eq!(comparison["items"].as_array().expect("all results").len(), count);
            assert_eq!(comparison["expected_index_matches_current"], true);
            for (n, pin) in pins.iter().enumerate() {
                assert_eq!(pin["key"], json!(format!("policy-{n:04}")));
                assert_eq!(comparison["items"][n]["key"], pin["key"]);
            }
        }
    }

    /// Missing index is distinct from an explicitly indexed empty inventory.
    #[test]
    fn absent_index_never_matches_even_an_empty_expected_bundle() {
        let absent = fixture(0, false);
        expect_error(preview(&absent.snapshot), "not-found");
        let empty = fixture(0, true);
        let value = bundle(&empty.snapshot);
        let result = verify_registered(&absent.snapshot, &json!({"bundle":value}))
            .expect("missing comparison");
        assert_eq!(result["state"], "missing-index");
        assert_eq!(result["source_index_present"], false);
        assert_eq!(result["expected_resources"], 0);
        assert_eq!(result["expected_index_matches_current"], false);
        assert!(!absent.project.path().join(super::super::index::INDEX_PATH).exists());
        let nonempty = fixture(1, true);
        let result =
            verify_registered(&absent.snapshot, &json!({"bundle":bundle(&nonempty.snapshot)}))
                .expect("missing comparison");
        assert_eq!(result["unregistered_resources"], 1);
        assert_eq!(result["items"][0]["observed_resource_validation_state"], "not-registered");
    }

    /// Metadata preview preserves source/index bytes and contains no source prose.
    #[test]
    fn preview_is_deterministic_metadata_with_original_resource_hashes() {
        let f = fixture(2, true);
        let index_path = f.project.path().join(super::super::index::INDEX_PATH);
        let original_index = std::fs::read(&index_path).expect("original index");
        let source =
            std::fs::read(f.project.path().join("policy-0000.md")).expect("original source");
        let first = preview(&f.snapshot).expect("preview");
        assert_eq!(first, preview(&f.snapshot).expect("same preview"));
        assert_eq!(first["bundle"]["pins"][0]["sha256"], crate::hashing::sha256_hex(&source));
        assert_eq!(first["bundle"]["pins"][0]["size_bytes"], json!(source.len()));
        assert_eq!(first["source_content_included"], false);
        assert_eq!(first["source_index_present"], true);
        assert!(
            !serde_json::to_string(&first)
                .expect("preview JSON")
                .contains("PRIVATE SOURCE SENTINEL")
        );
        assert_eq!(std::fs::read(index_path).expect("unchanged index"), original_index);
        assert_eq!(
            std::fs::read(f.project.path().join("policy-0000.md")).expect("unchanged source"),
            source
        );
    }

    /// Expected subset agreement reports current extras and whole-index inequality.
    #[test]
    fn expected_subset_does_not_remove_or_hide_current_extra_registrations() {
        let f = fixture(3, true);
        let mut value = bundle(&f.snapshot);
        value["index"]["resources"].as_array_mut().expect("resources").truncate(1);
        value["pins"].as_array_mut().expect("pins").truncate(1);
        refresh_index_hash(&mut value);
        let result =
            verify_registered(&f.snapshot, &json!({"bundle":value})).expect("subset comparison");
        assert_eq!(result["state"], "matched");
        assert_eq!(result["expected_resources"], 1);
        assert_eq!(result["current_resources"], 3);
        assert_eq!(result["current_only_resources"], 2);
        assert_eq!(result["expected_index_matches_current"], false);
        assert_eq!(f.snapshot.items.len(), 3);
    }

    /// Whole-index label/order differences do not change exact fingerprint agreement.
    #[test]
    fn authorial_order_and_display_label_are_distinct_from_resource_matches() {
        let f = fixture(3, true);
        let expected = bundle(&f.snapshot);
        for reorder in [false, true] {
            let mut index = f.snapshot.index.clone();
            if reorder {
                index.resources.reverse();
            } else {
                index.label = "Another asserted display label".into();
            }
            std::fs::write(
                f.project.path().join(super::super::index::INDEX_PATH),
                index.bytes().expect("changed metadata index"),
            )
            .expect("fixture metadata change");
            let root = Root::open(f.project.path()).expect("fixture root");
            let current = Snapshot::capture(&root).expect("new actual capture");
            let observed = bundle(&current);
            assert_eq!(
                observed["pins"][0]["key"],
                if reorder { "policy-0002" } else { "policy-0000" }
            );
            let result = verify_registered(&current, &json!({"bundle":expected}))
                .expect("fingerprint comparison");
            assert_eq!(result["state"], "matched");
            assert_eq!(result["current_only_resources"], 0);
            assert_eq!(result["expected_index_matches_current"], false);
        }
    }

    /// Every mixed expected entry is classified without opening supplied paths.
    #[test]
    fn mismatches_and_unregistered_paths_keep_the_complete_expected_order() {
        let f = fixture(3, true);
        let mut value = bundle(&f.snapshot);
        value["pins"][1]["sha256"] = json!("f".repeat(64));
        value["pins"][1]["size_bytes"] = json!(0_usize);
        value["index"]["resources"][2]["key"] = json!("unregistered");
        value["index"]["resources"][2]["path"] = json!("private-directory");
        value["pins"][2]["key"] = json!("unregistered");
        std::fs::create_dir(f.project.path().join("private-directory"))
            .expect("unsafe unregistered path sentinel");
        refresh_index_hash(&mut value);
        let result = verify_registered(&f.snapshot, &json!({"bundle":value}))
            .expect("complete comparison without path read");
        assert_eq!(result["state"], "mismatched");
        assert_eq!(result["matched_resources"], 1);
        assert_eq!(result["mismatched_resources"], 1);
        assert_eq!(result["unregistered_resources"], 1);
        assert_eq!(result["current_only_resources"], 1);
        assert_eq!(result["items"][0]["status"], "matched");
        assert_eq!(result["items"][1]["reason_codes"], json!(["sha256-mismatch", "size-mismatch"]));
        assert_eq!(result["items"][2]["key"], "unregistered");
        assert_eq!(result["items"][2]["reason_codes"], json!(["registration-not-found"]));
    }

    /// Same-key role/path conflicts are registration mismatches, not new reads.
    #[test]
    fn registration_conflicts_are_distinct_from_content_fingerprint_mismatches() {
        let f = fixture(1, true);
        for (field, changed) in [("path", "private.json"), ("role", "applicability-report")] {
            let mut value = bundle(&f.snapshot);
            value["index"]["resources"][0][field] = json!(changed);
            refresh_index_hash(&mut value);
            let result = verify_registered(&f.snapshot, &json!({"bundle":value}))
                .expect("registration conflict");
            assert_eq!(result["mismatched_resources"], 1);
            assert_eq!(result["items"][0]["reason_codes"], json!(["registration-conflict"]));
        }
    }

    /// Hash-match status never promotes captured stale/invalid validation state.
    #[test]
    fn fingerprint_agreement_retains_stale_and_invalid_observed_states() {
        let mut f = fixture(2, true);
        f.snapshot.items[0].metadata["validation_state"] = json!("stale");
        f.snapshot.items[1].metadata["validation_state"] = json!("invalid");
        let value = bundle(&f.snapshot);
        let result =
            verify_registered(&f.snapshot, &json!({"bundle":value})).expect("metadata comparison");
        assert_eq!(result["state"], "matched");
        assert_eq!(result["items"][0]["observed_resource_validation_state"], "stale");
        assert_eq!(result["items"][1]["observed_resource_validation_state"], "invalid");
        assert!(result.get("import_ready").is_none());
        assert!(result.get("approval_state").is_none());
    }

    /// Unknown fields at every nested authority boundary and forward profiles fail.
    #[test]
    fn closed_bundle_fields_versions_profiles_and_request_envelopes_are_enforced() {
        let f = fixture(1, true);
        let original = bundle(&f.snapshot);
        let mut invalid = Vec::new();
        for (field, changed) in [
            ("schema_version", "forge.workspace-index-bundle/2"),
            ("content_profile", "source-inclusive"),
        ] {
            let mut value = original.clone();
            value[field] = json!(changed);
            invalid.push(value);
        }
        let mut value = original.clone();
        value["source_content"] = json!("PRIVATE SOURCE SENTINEL");
        invalid.push(value);
        let mut value = original.clone();
        value["index"]["approval_state"] = json!("approved");
        invalid.push(value);
        let mut value = original.clone();
        value["pins"][0]["reviewer"] = json!("invented");
        invalid.push(value);
        for value in invalid {
            expect_error(
                verify_registered(&f.snapshot, &json!({"bundle":value})),
                "invalid-request",
            );
        }
        expect_error(
            verify_registered(&f.snapshot, &json!({"bundle":original,"confirmed":true})),
            "invalid-request",
        );
        expect_error(verify_registered(&f.snapshot, &json!({})), "invalid-request");
    }

    /// Pin omissions, extras, duplication and reordering cannot shrink the denominator.
    #[test]
    fn pin_bijection_and_normalized_index_hash_are_intrinsic_requirements() {
        let f = fixture(2, true);
        let original = bundle(&f.snapshot);
        let mut invalid = Vec::new();
        let mut value = original.clone();
        value["pins"].as_array_mut().expect("pins").pop();
        invalid.push(value);
        let mut value = original.clone();
        let extra = value["pins"][0].clone();
        value["pins"].as_array_mut().expect("pins").push(extra);
        invalid.push(value);
        let mut value = original.clone();
        value["pins"][1] = value["pins"][0].clone();
        invalid.push(value);
        let mut value = original.clone();
        value["pins"].as_array_mut().expect("pins").swap(0, 1);
        invalid.push(value);
        let mut value = original.clone();
        value["index_sha256"] = json!("0".repeat(64));
        invalid.push(value);
        for value in invalid {
            expect_error(
                verify_registered(&f.snapshot, &json!({"bundle":value})),
                "invalid-request",
            );
        }
    }

    /// Pin hashes and lengths reject malformed or unrepresentable declarations.
    #[test]
    fn pin_hash_format_and_integer_length_bounds_are_closed() {
        let f = fixture(1, true);
        let original = bundle(&f.snapshot);
        for hash in ["not-a-hash".to_owned(), "F".repeat(64), "a".repeat(63)] {
            let mut value = original.clone();
            value["pins"][0]["sha256"] = json!(hash);
            expect_error(
                verify_registered(&f.snapshot, &json!({"bundle":value})),
                "invalid-request",
            );
        }
        for length in
            [json!(-1_i64), json!(1.5_f64), json!(u64::MAX), json!(MAX_RESOURCE_BYTES + 1)]
        {
            let mut value = original.clone();
            value["pins"][0]["size_bytes"] = length;
            expect_error(
                verify_registered(&f.snapshot, &json!({"bundle":value})),
                "invalid-request",
            );
        }
    }

    /// Existing portable path/key alias rules still govern nested index data.
    #[test]
    fn nested_index_rejects_duplicate_keys_casefold_aliases_and_unsafe_paths() {
        let f = fixture(2, true);
        let original = bundle(&f.snapshot);
        for field in ["key", "path"] {
            let mut value = original.clone();
            value["index"]["resources"][1][field] = value["index"]["resources"][0][field].clone();
            expect_error(
                verify_registered(&f.snapshot, &json!({"bundle":value})),
                "invalid-request",
            );
        }
        let mut alias = original.clone();
        alias["index"]["resources"][1]["path"] = json!("POLICY-0000.md");
        expect_error(verify_registered(&f.snapshot, &json!({"bundle":alias})), "invalid-request");
        for path in ["../private.md", "C:/private.md", "private\\file.md", ".hidden.md"] {
            let mut value = original.clone();
            value["index"]["resources"][0]["path"] = json!(path);
            expect_error(
                verify_registered(&f.snapshot, &json!({"bundle":value})),
                "invalid-request",
            );
        }
        let mut device = original;
        device["index"]["resources"][0]["path"] = json!("CON");
        expect_error(
            verify_registered(&f.snapshot, &json!({"bundle":device})),
            "resource-containment",
        );
    }

    /// Resource ceiling rejects1,001 entries without blessing a1,000-entry prefix.
    #[test]
    fn one_over_resource_limit_rejects_the_whole_bundle() {
        let f = fixture(0, true);
        let mut value = bundle(&f.snapshot);
        let resources = (0..1001).map(|n| json!({"key":format!("key-{n}"),"role":"policy-source","path":format!("file-{n}.md")})).collect::<Vec<_>>();
        let pins = (0..1001)
            .map(|n| json!({"key":format!("key-{n}"),"sha256":"b".repeat(64),"size_bytes":0}))
            .collect::<Vec<_>>();
        value["index"]["resources"] = json!(resources);
        value["pins"] = json!(pins);
        expect_error(verify_registered(&f.snapshot, &json!({"bundle":value})), "invalid-request");
    }

    /// Declared bytes include the normalized index at exactly/one-over50MiB.
    #[test]
    fn declared_aggregate_bound_includes_the_normalized_index() {
        let f = fixture(5, true);
        let mut value = bundle(&f.snapshot);
        let parsed: Index = serde_json::from_value(value["index"].clone()).expect("known index");
        let index_bytes = parsed.bytes().expect("normalized index").len();
        for n in 0..4 {
            value["pins"][n]["size_bytes"] = json!(MAX_RESOURCE_BYTES);
        }
        let remaining = MAX_DECLARED_BYTES - index_bytes - 4 * MAX_RESOURCE_BYTES;
        value["pins"][4]["size_bytes"] = json!(remaining);
        assert!(decode_bundle(&value).is_ok(), "exact aggregate declaration is representable");
        value["pins"][4]["size_bytes"] = json!(remaining + 1);
        let error = decode_bundle(&value).err().expect("one-over aggregate");
        assert_eq!(error.code, "payload-too-large");
        value["pins"][0]["size_bytes"] = json!(MAX_RESOURCE_BYTES + 1);
        expect_error(verify_registered(&f.snapshot, &json!({"bundle":value})), "invalid-request");
    }

    /// Direct Value callers retain the whole-wrapper encoding limit and safe errors.
    #[test]
    fn direct_request_size_bound_includes_extra_wrapper_bytes() {
        let f = fixture(0, true);
        let request = json!({"bundle":bundle(&f.snapshot),"unknown":"x".repeat(MAX_BUNDLE_BYTES)});
        expect_error(verify_registered(&f.snapshot, &request), "payload-too-large");
    }

    /// A captured preview is not a promise that files stay unchanged afterward.
    #[test]
    fn comparison_uses_observed_capture_not_a_supplied_path_reopen() {
        let f = fixture(1, true);
        let value = bundle(&f.snapshot);
        let path = f.project.path().join("policy-0000.md");
        std::fs::write(&path, b"later external source bytes").expect("external change");
        let result = verify_registered(&f.snapshot, &json!({"bundle":value}))
            .expect("observed capture comparison");
        assert_eq!(result["state"], "matched");
        assert_eq!(
            std::fs::read(path).expect("external bytes preserved"),
            b"later external source bytes"
        );
    }

    /// Malformed observed validation states cannot be serialized as a qualified result.
    #[test]
    fn inconsistent_direct_snapshot_state_rejects_without_a_partial_result() {
        let mut f = fixture(1, true);
        let value = bundle(&f.snapshot);
        f.snapshot.items[0].metadata["validation_state"] = json!("approved");
        expect_error(preview(&f.snapshot), "invalid-request");
        expect_error(verify_registered(&f.snapshot, &json!({"bundle":value})), "invalid-request");
        f.snapshot.items.clear();
        expect_error(preview(&f.snapshot), "invalid-request");
    }
    /// Capture an explicit /2 project with actual opaque source bytes, including zero entries.
    fn version_two_fixture(count: usize) -> Fixture {
        let project = tempfile::tempdir().expect("private /2 fixture");
        let mut index = Index::empty();
        index.schema_version = "forge.workspace/2".into();
        for n in 0..count {
            let path = format!("source-{n:04}.bin");
            let bytes = [0, 255, u8::try_from(n % 251).unwrap()];
            std::fs::write(project.path().join(&path), bytes).unwrap();
            index.resources.push(Resource {
                key: format!("source-{n:04}"),
                role: Role::LifecycleSource,
                path,
            });
        }
        std::fs::write(
            project.path().join(super::super::index::INDEX_PATH),
            index.bytes().unwrap(),
        )
        .unwrap();
        let root = Root::open(project.path()).unwrap();
        let snapshot = Snapshot::capture(&root).expect("actual /2 capture");
        Fixture { project, snapshot }
    }

    /// Selected-major queries preserve all zero,101 and1,000 /2 resources while v1 remains closed.
    #[test]
    fn explicit_v2_metadata_consumers_preserve_complete_denominators() {
        for count in [0, 101, 1000] {
            let f = version_two_fixture(count);
            let response = preview_for_api(&f.snapshot, ApiMajor::V2).unwrap();
            let value = &response["bundle"];
            assert_eq!(value["schema_version"], "forge.workspace-index-bundle/2");
            assert_eq!(value["index"]["schema_version"], "forge.workspace/2");
            assert_eq!(value["pins"].as_array().unwrap().len(), count);
            let request = json!({"bundle":value});
            let result = verify_registered_for_api(&f.snapshot, &request, ApiMajor::V2).unwrap();
            assert_eq!(result["matched_resources"], json!(count));
            assert_eq!(result["current_resources"], json!(count));
            assert_eq!(result["expected_index_matches_current"], true);
            assert_eq!(result["source_content_included"], false);
            expect_error(preview(&f.snapshot), "invalid-request");
            expect_error(verify_registered(&f.snapshot, &request), "invalid-request");
            for item in &f.snapshot.items {
                assert_eq!(item.metadata["validation_profile"], "opaque-fingerprint-bytes");
                assert_eq!(item.metadata["validation_state"], "valid");
            }
        }
    }

    /// Both explicit pairs reject cross-version index interpretations and unknown bundle fields.
    #[test]
    fn v2_bundle_pairs_do_not_reinterpret_original_index_or_roles() {
        let f = version_two_fixture(1);
        let valid = preview_for_api(&f.snapshot, ApiMajor::V2).unwrap()["bundle"].clone();
        for (key, value) in [
            ("schema_version", json!("forge.workspace-index-bundle/1")),
            ("schema_version", json!("forge.workspace-index-bundle/3")),
            ("content_profile", json!("source-and-hashes")),
            ("unknown", json!("PRIVATE SOURCE SENTINEL")),
        ] {
            let mut invalid = valid.clone();
            invalid[key] = value;
            expect_error(
                verify_registered_for_api(&f.snapshot, &json!({"bundle":invalid}), ApiMajor::V2),
                "invalid-request",
            );
        }
        let mut wrong_index = valid.clone();
        wrong_index["index"]["schema_version"] = json!("forge.workspace/1");
        expect_error(
            verify_registered_for_api(&f.snapshot, &json!({"bundle":wrong_index}), ApiMajor::V2),
            "invalid-request",
        );
        assert!(
            decode_bundle(&valid).is_err(),
            "the original supplied-bundle decoder stays /1-only"
        );
    }

    /// A prior /1 bundle compares registered captures in /2 without hiding extras or claiming index equality.
    #[test]
    fn supplied_version_one_comparison_in_v2_retains_registered_only_scope() {
        let mut f = fixture(1, true);
        let original = bundle(&f.snapshot);
        let original_snapshot = f.snapshot;
        let mut index = original_snapshot.index.clone();
        index.schema_version = "forge.workspace/2".into();
        index.resources.push(Resource {
            key: "opaque-source".into(),
            role: Role::LifecycleSource,
            path: "opaque.bin".into(),
        });
        std::fs::write(f.project.path().join("opaque.bin"), b"").unwrap();
        std::fs::write(
            f.project.path().join(super::super::index::INDEX_PATH),
            index.bytes().unwrap(),
        )
        .unwrap();
        let root = Root::open(f.project.path()).unwrap();
        f.snapshot = Snapshot::capture(&root).unwrap();
        let request = json!({"bundle":original});
        let compared = verify_registered_for_api(&f.snapshot, &request, ApiMajor::V2).unwrap();
        assert_eq!(compared["state"], "matched");
        assert_eq!(compared["matched_resources"], 1);
        assert_eq!(compared["current_only_resources"], 1);
        assert_eq!(compared["expected_index_matches_current"], false);
        let v2_bundle = preview_for_api(&f.snapshot, ApiMajor::V2).unwrap()["bundle"].clone();
        let reverse = verify_registered_for_api(
            &original_snapshot,
            &json!({"bundle":v2_bundle}),
            ApiMajor::V2,
        )
        .unwrap();
        assert_eq!(reverse["matched_resources"], 1);
        assert_eq!(reverse["unregistered_resources"], 1);
        assert_eq!(reverse["current_only_resources"], 0);
        assert_eq!(reverse["expected_index_matches_current"], false);
    }
}
