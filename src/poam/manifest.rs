//! Closed, bounded `forge.poam/1` foundation manifest.
//!
//! The foundation accepts an explicitly unselected scaffold. Authored remediation
//! work remains unsupported until the full workflow contract is implemented and
//! reviewed; successfully parsing a scaffold asserts no remediation eligibility.

use std::collections::BTreeSet;
use std::path::Path;

use chrono::DateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ForgeError;
pub use crate::assessment_results::manifest::{
    ArtifactManifest, ContextManifest, DocumentManifest, PartyManifest, RoleManifest,
};
use crate::assessment_results::manifest::{validate_artifact, validate_context};
use crate::json_strict::{self, Limits};

/// Proposed POA&M manifest schema understood by this foundation.
pub const MANIFEST_SCHEMA_VERSION: &str = "forge.poam/1";
/// Maximum raw manifest size, before JSON allocations.
pub const MAX_MANIFEST_BYTES: u64 = 4 * 1024 * 1024;
/// Maximum decoded string or object-key size.
pub const MAX_STRING_BYTES: usize = 64 * 1024;
/// Maximum remediation items allowed by the proposed shape.
pub const MAX_ITEMS: usize = 10_000;
/// Maximum references or milestone identities on an item.
pub const MAX_REFERENCES: usize = 1_000;
/// Maximum role or party records allowed by the proposed shape.
pub const MAX_PARTIES: usize = 1_000;

/// Explicit document metadata and exact, unselected assessment source.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PoamManifest {
    /// Exact manifest format identifier; the foundation accepts `forge.poam/1`.
    pub schema_version: String,
    /// Explicit plan key, prose, version and RFC 3339 modification time; no defaults are inferred.
    pub document: DocumentManifest,
    /// Exact AR result and four companions whose bytes and import relationships must agree.
    pub source: SourceManifest,
    /// Reserved roles; this foundation requires an empty array and assigns no actors.
    pub roles: Vec<RoleManifest>,
    /// Reserved parties; this foundation requires an empty array and infers no authority.
    pub parties: Vec<PartyManifest>,
    /// Reserved remediation work; only an empty, unselected scaffold is supported here.
    pub items: Vec<ItemManifest>,
}

/// Five exact local identities and one explicitly selected result.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceManifest {
    /// Confined local Forge AR 1.2.3 input, pinned by exact file hash, UUID and versions.
    pub assessment_results: ArtifactManifest,
    /// Explicit native UUID and stable key; both must identify the same result.
    pub result: ResultIdentity,
    /// Exact AP/SSP/Profile/Catalog pins; companion versions 1.2.0–1.2.3 are supported.
    /// An evidence index is unsupported by this foundation.
    pub context: ContextManifest,
}

/// Both native result UUID and Forge stable key must identify the same result.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResultIdentity {
    /// Canonical lowercase, hyphenated native UUID of the selected result.
    pub uuid: String,
    /// Exact Forge stable key from that same result; array position is never a selector.
    pub key: String,
}

/// Reserved authored shape; nonempty items require the complete workflow.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ItemManifest {
    /// Immutable authored key used for deterministic item identity within the plan.
    pub key: String,
    /// Exact source tuples; the reserved shape requires 1–1,000 unique kind/key pairs.
    pub source_refs: Vec<SourceReference>,
    /// Up to 1,000 unique immutable milestone keys, without inferred dates or completion.
    pub milestones: Vec<MilestoneIdentity>,
}

/// Exact source tuple; the digest binds the computed canonical source object.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct SourceReference {
    /// Source domain; equal keys in finding and risk domains remain distinct.
    pub kind: SourceKind,
    /// Exact stable key from the source object; no normalization or prose matching occurs.
    pub key: String,
    /// Canonical native UUID of that source object, preserved rather than regenerated.
    pub uuid: String,
    /// Canonical UUID of the explicitly selected result containing this object.
    pub result_uuid: String,
    /// Lowercase SHA-256 of the computed canonical object, including its current prose.
    /// This must match the entire result/kind/key/UUID tuple; declared digest properties
    /// cannot substitute for this value or establish review or remediation eligibility.
    pub expected_sha256: String,
}

/// Source object kinds supported by the foundation inventory.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum SourceKind {
    /// An original assessment finding, including one whose asserted state is satisfied.
    Finding,
    /// An original assessment risk, including one whose asserted status is closed.
    Risk,
}

impl SourceKind {
    /// Native, stable source-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Finding => "finding",
            Self::Risk => "risk",
        }
    }
}

/// Stable identity reserved for a future authored milestone.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MilestoneIdentity {
    /// Immutable authored key scoped by its parent item and plan, independent of dates or prose.
    pub key: String,
}

/// Parse a duplicate-safe foundation scaffold with explicit source identities.
///
/// Raw JSON is capped at [`MAX_MANIFEST_BYTES`], nesting at 64 levels, and decoded
/// strings and object keys at [`MAX_STRING_BYTES`]. Every authored record is closed
/// to unknown fields. Nonempty work, roles or parties return an unsupported-workflow
/// error even when their reserved shape is structurally valid. Parsing establishes
/// declaration syntax only; [`super::source::load`] separately verifies source bytes.
///
/// # Errors
///
/// Returns an error for malformed or unbounded JSON, an unknown contract field,
/// invalid identity or metadata syntax, aliases, or unsupported authored work.
pub fn parse(bytes: &[u8]) -> Result<PoamManifest, ForgeError> {
    if bytes.len() as u64 > MAX_MANIFEST_BYTES {
        return Err(error(format!("manifest exceeds the {MAX_MANIFEST_BYTES} byte limit")));
    }
    let value = json_strict::parse_value(
        bytes,
        "POA&M manifest",
        Limits { max_depth: 64, max_string_bytes: MAX_STRING_BYTES },
    )
    .map_err(|cause| error(cause.to_string()))?;
    let manifest: PoamManifest = serde_json::from_value(value)
        .map_err(|cause| error(format!("invalid manifest contract: {cause}")))?;
    validate(&manifest)?;
    Ok(manifest)
}

/// Validate declaration syntax and retain the empty-workflow boundary.
fn validate(manifest: &PoamManifest) -> Result<(), ForgeError> {
    if manifest.schema_version != MANIFEST_SCHEMA_VERSION {
        return Err(error(format!(
            "unsupported schema_version '{}'; expected {MANIFEST_SCHEMA_VERSION}",
            json_strict::bounded(&manifest.schema_version)
        )));
    }
    non_empty("$.document.key", &manifest.document.key)?;
    non_empty("$.document.title", &manifest.document.title)?;
    non_empty("$.document.version", &manifest.document.version)?;
    DateTime::parse_from_rfc3339(&manifest.document.last_modified)
        .map_err(|_| error("$.document.last_modified must be an RFC 3339 timestamp"))?;
    validate_source(&manifest.source)?;
    validate_items(&manifest.items, &manifest.source.result.uuid)?;
    for (path, count) in [("$.roles", manifest.roles.len()), ("$.parties", manifest.parties.len())]
    {
        if count > MAX_PARTIES {
            return Err(error(format!("{path} exceeds the {MAX_PARTIES} entry limit")));
        }
    }
    if !manifest.items.is_empty() || !manifest.roles.is_empty() || !manifest.parties.is_empty() {
        return Err(error(
            "unsupported full POA&M workflow: foundation scaffolds require empty items, roles, and parties; source-only validation does not validate remediation work",
        ));
    }
    Ok(())
}

/// Validate source declaration syntax for both parsed and directly constructed inputs.
///
/// Enforces normalized, distinct local paths, exact identity syntax, supported
/// versions and the no-evidence-index boundary. This does not read artifacts;
/// [`super::source::load`] checks their bytes, schemas and relationships.
///
/// # Errors
/// Returns an error for invalid identity/path syntax or an unsupported source profile.
pub(crate) fn validate_source(source: &SourceManifest) -> Result<(), ForgeError> {
    validate_artifact("$.source.assessment_results", &source.assessment_results)
        .map_err(|cause| error(cause.to_string()))?;
    validate_context(&source.context).map_err(|cause| error(cause.to_string()))?;
    if source.context.evidence_index.is_some() {
        return Err(error(
            "$.source.context.evidence_index is unsupported by the POA&M foundation",
        ));
    }
    if source.assessment_results.oscal_version != "1.2.3" {
        return Err(error(
            "$.source.assessment_results.oscal_version must be 1.2.3 for the supported Forge AR source profile",
        ));
    }
    canonical_uuid("$.source.result.uuid", &source.result.uuid)?;
    non_empty("$.source.result.key", &source.result.key)?;
    let mut paths = BTreeSet::new();
    for (path, artifact) in [
        ("$.source.assessment_results", &source.assessment_results),
        ("$.source.context.assessment_plan", &source.context.assessment_plan),
        ("$.source.context.ssp", &source.context.ssp),
        ("$.source.context.profile", &source.context.profile),
        ("$.source.context.catalog", &source.context.catalog),
    ] {
        normalized_json_path(&format!("{path}.artifact"), &artifact.artifact)?;
        normalized_local_reference(&format!("{path}.href"), &artifact.href)?;
        canonical_uuid(&format!("{path}.root_uuid"), &artifact.root_uuid)?;
        if !matches!(artifact.oscal_version.as_str(), "1.2.0" | "1.2.1" | "1.2.2" | "1.2.3") {
            return Err(error(format!(
                "{path}.oscal_version must be one of 1.2.0, 1.2.1, 1.2.2, or 1.2.3"
            )));
        }
        if !paths.insert(artifact.artifact.as_path()) {
            return Err(error(format!("{path}.artifact duplicates another source artifact path")));
        }
    }
    Ok(())
}

/// Check reserved item tuples, duplicate keys and collection bounds before refusing authored work.
fn validate_items(items: &[ItemManifest], result_uuid: &str) -> Result<(), ForgeError> {
    if items.len() > MAX_ITEMS {
        return Err(error(format!("$.items exceeds the {MAX_ITEMS} entry limit")));
    }
    let mut keys = BTreeSet::new();
    for (index, item) in items.iter().enumerate() {
        let path = format!("$.items[{index}]");
        non_empty(&format!("{path}.key"), &item.key)?;
        if !keys.insert(item.key.as_str()) {
            return Err(error(format!("{path}.key duplicates an item key")));
        }
        if item.source_refs.is_empty() || item.source_refs.len() > MAX_REFERENCES {
            return Err(error(format!(
                "{path}.source_refs must contain 1..={MAX_REFERENCES} entries"
            )));
        }
        let mut references = BTreeSet::new();
        for (reference_index, reference) in item.source_refs.iter().enumerate() {
            let reference_path = format!("{path}.source_refs[{reference_index}]");
            validate_source_reference(&reference_path, reference)?;
            if reference.result_uuid != result_uuid {
                return Err(error(format!(
                    "{reference_path}.result_uuid does not match the selected result"
                )));
            }
            if !references.insert((reference.kind, reference.key.as_str())) {
                return Err(error(format!("{reference_path} duplicates a source kind and key")));
            }
        }
        if item.milestones.len() > MAX_REFERENCES {
            return Err(error(format!(
                "{path}.milestones exceeds the {MAX_REFERENCES} entry limit"
            )));
        }
        let mut milestone_keys = BTreeSet::new();
        for (milestone_index, milestone) in item.milestones.iter().enumerate() {
            let milestone_path = format!("{path}.milestones[{milestone_index}].key");
            non_empty(&milestone_path, &milestone.key)?;
            if !milestone_keys.insert(milestone.key.as_str()) {
                return Err(error(format!("{milestone_path} duplicates a milestone key")));
            }
        }
    }
    Ok(())
}

/// Validate tuple syntax without asserting that its source exists or is eligible.
pub(crate) fn validate_source_reference(
    path: &str,
    reference: &SourceReference,
) -> Result<(), ForgeError> {
    non_empty(&format!("{path}.key"), &reference.key)?;
    canonical_uuid(&format!("{path}.uuid"), &reference.uuid)?;
    canonical_uuid(&format!("{path}.result_uuid"), &reference.result_uuid)?;
    json_strict::validate_lowercase_sha256(
        &format!("{path}.expected_sha256"),
        &reference.expected_sha256,
    )
    .map_err(error)
}

/// Require an alias-free local JSON path for each pinned source artifact.
fn normalized_json_path(path: &str, value: &Path) -> Result<(), ForgeError> {
    let text = value.to_str().ok_or_else(|| error(format!("{path} must be UTF-8")))?;
    normalized_local_reference(path, text)?;
    if value.extension().and_then(|extension| extension.to_str()) != Some("json") {
        return Err(error(format!("{path} must name a local .json file")));
    }
    Ok(())
}

/// Reject absolute, device, traversal and ambiguous local reference spellings.
fn normalized_local_reference(path: &str, value: &str) -> Result<(), ForgeError> {
    if value.is_empty()
        || value.chars().any(char::is_control)
        || value.contains(['\\', ':', '?', '#'])
    {
        return Err(error(format!("{path} must be a normalized slash-separated local reference")));
    }
    for segment in value.split('/') {
        if segment.is_empty()
            || matches!(segment, "." | "..")
            || segment.ends_with(['.', ' '])
            || windows_device(segment)
        {
            return Err(error(format!(
                "{path} must be a normalized local reference without aliases or parent traversal"
            )));
        }
    }
    Ok(())
}

/// Recognize reserved Windows device stems before local path admission.
fn windows_device(segment: &str) -> bool {
    let stem = segment.split('.').next().unwrap_or_default().to_ascii_uppercase();
    matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || stem.strip_prefix("COM").is_some_and(device_number)
        || stem.strip_prefix("LPT").is_some_and(device_number)
}

/// Recognize the single digit suffix used by reserved COM and LPT names.
fn device_number(suffix: &str) -> bool {
    matches!(suffix, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³")
}

/// Require the exact lowercase hyphenated UUID spelling used by the source profile.
fn canonical_uuid(path: &str, value: &str) -> Result<(), ForgeError> {
    let parsed = Uuid::parse_str(value).map_err(|_| error(format!("{path} must be a UUID")))?;
    if parsed.to_string() != value {
        return Err(error(format!("{path} must be a canonical lowercase hyphenated UUID")));
    }
    Ok(())
}

/// Reject a declaration containing only whitespace.
fn non_empty(path: &str, value: &str) -> Result<(), ForgeError> {
    if value.trim().is_empty() { Err(error(format!("{path} must not be empty"))) } else { Ok(()) }
}

/// Construct the typed foundation failure without granting source or workflow approval.
fn error(message: impl Into<String>) -> ForgeError {
    ForgeError::PoamBuild(message.into())
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    const UUID: &str = "11111111-1111-4111-8111-111111111111";

    /// Construct a syntactically pinned companion for manifest boundary controls.
    fn artifact(name: &str) -> Value {
        json!({
            "artifact": format!("context/{name}.json"),
            "href": format!("{name}.json"),
            "expected_sha256": "a".repeat(64),
            "root_uuid": UUID,
            "document_version": "1.0.0",
            "oscal_version": "1.2.3"
        })
    }

    /// Construct an explicitly unselected foundation fixture without remediation records.
    fn scaffold() -> Value {
        json!({
            "schema_version": MANIFEST_SCHEMA_VERSION,
            "document": { "key": "plan-1", "title": "Explicit plan title", "version": "1.0.0", "last_modified": "2026-10-02T12:34:56Z" },
            "source": {
                "assessment_results": artifact("ar"),
                "result": { "uuid": UUID, "key": "result-1" },
                "context": {
                    "assessment_plan": artifact("ap"), "ssp": artifact("ssp"),
                    "profile": artifact("profile"), "catalog": artifact("catalog")
                }
            },
            "roles": [], "parties": [], "items": []
        })
    }

    /// Construct reserved item syntax for refusal and duplicate-identity controls.
    fn item() -> Value {
        json!({ "key": "work-1", "source_refs": [{ "kind": "finding", "key": "finding-1", "uuid": UUID, "result_uuid": UUID, "expected_sha256": "b".repeat(64) }], "milestones": [{ "key": "milestone-1" }] })
    }

    /// Serialize a synthetic declaration through the actual strict manifest parser.
    fn parse_value(value: &Value) -> Result<PoamManifest, ForgeError> {
        parse(&serde_json::to_vec(value).expect("test value serializes"))
    }

    /// Verify that accepts explicit empty scaffold without inventing work or metadata.
    #[test]
    fn accepts_explicit_empty_scaffold_without_inventing_work_or_metadata() {
        let manifest = parse_value(&scaffold()).expect("empty scaffold is supported");
        assert!(manifest.items.is_empty());
        assert!(manifest.roles.is_empty());
        assert!(manifest.parties.is_empty());
        assert_eq!(manifest.document.last_modified, "2026-10-02T12:34:56Z");
        assert_eq!(manifest.source.result.key, "result-1");
        assert_eq!(manifest.source.assessment_results.expected_sha256, "a".repeat(64));
    }

    /// Verify that rejects duplicate nested json and trailing values.
    #[test]
    fn rejects_duplicate_nested_json_and_trailing_values() {
        let bytes = serde_json::to_vec(&scaffold()).unwrap();
        let text = String::from_utf8(bytes)
            .unwrap()
            .replace("\"key\":\"result-1\"", "\"key\":\"result-1\",\"key\":\"result-2\"");
        assert!(parse(text.as_bytes()).unwrap_err().to_string().contains("duplicate object key"));
        let mut trailing = serde_json::to_vec(&scaffold()).unwrap();
        trailing.extend_from_slice(b" {}");
        assert!(parse(&trailing).unwrap_err().to_string().contains("trailing"));
    }

    /// Verify that rejects unknown fields at every authored level.
    #[test]
    fn rejects_unknown_fields_at_every_authored_level() {
        for pointer in [
            "",
            "/document",
            "/source",
            "/source/result",
            "/source/context",
            "/source/assessment_results",
            "/source/context/assessment_plan",
        ] {
            let mut value = scaffold();
            value
                .pointer_mut(pointer)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("unknown".to_string(), json!(true));
            assert!(
                parse_value(&value).unwrap_err().to_string().contains("unknown field"),
                "{pointer}"
            );
        }
        for pointer in ["/items/0", "/items/0/source_refs/0", "/items/0/milestones/0"] {
            let mut value = scaffold();
            value["items"] = json!([item()]);
            value
                .pointer_mut(pointer)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("unknown".to_string(), json!(true));
            assert!(
                parse_value(&value).unwrap_err().to_string().contains("unknown field"),
                "{pointer}"
            );
        }
    }

    /// Verify that rejects unsupported authored items roles and parties.
    #[test]
    fn rejects_unsupported_authored_items_roles_and_parties() {
        for (field, record) in [
            ("items", item()),
            ("roles", json!({ "id": "reviewer", "title": "Reviewer" })),
            ("parties", json!({ "key": "person-1", "type": "person", "name": "Explicit person" })),
        ] {
            let mut value = scaffold();
            value[field] = json!([record]);
            assert!(
                parse_value(&value)
                    .unwrap_err()
                    .to_string()
                    .contains("unsupported full POA&M workflow"),
                "{field}"
            );
        }
    }

    /// Verify that validates source reference tuples before unsupported workflow error.
    #[test]
    fn validates_source_reference_tuples_before_unsupported_workflow_error() {
        for (field, replacement, expected) in [
            ("key", json!(" "), "must not be empty"),
            ("uuid", json!("not-a-uuid"), "must be a UUID"),
            ("result_uuid", json!("22222222-2222-4222-8222-222222222222"), "does not match"),
            ("expected_sha256", json!("B".repeat(64)), "lowercase hexadecimal"),
        ] {
            let mut value = scaffold();
            value["items"] = json!([item()]);
            value["items"][0]["source_refs"][0][field] = replacement;
            assert!(parse_value(&value).unwrap_err().to_string().contains(expected), "{field}");
        }
        let mut value = scaffold();
        value["items"] = json!([item()]);
        value["items"][0]["source_refs"] = json!([]);
        assert!(parse_value(&value).unwrap_err().to_string().contains("must contain 1..="));
    }

    /// Verify that rejects duplicate item source and milestone keys.
    #[test]
    fn rejects_duplicate_item_source_and_milestone_keys() {
        let mut value = scaffold();
        value["items"] = json!([item(), item()]);
        assert!(parse_value(&value).unwrap_err().to_string().contains("duplicates an item key"));
        value["items"] = json!([item()]);
        let source = value["items"][0]["source_refs"][0].clone();
        value["items"][0]["source_refs"] = json!([source.clone(), source]);
        assert!(
            parse_value(&value)
                .unwrap_err()
                .to_string()
                .contains("duplicates a source kind and key")
        );
        value["items"] = json!([item()]);
        value["items"][0]["milestones"] = json!([{ "key": "same" }, { "key": "same" }]);
        assert!(
            parse_value(&value).unwrap_err().to_string().contains("duplicates a milestone key")
        );
    }

    /// Verify that rejects manifest string depth and collection overflow.
    #[test]
    fn rejects_manifest_string_depth_and_collection_overflow() {
        let oversize = vec![b' '; usize::try_from(MAX_MANIFEST_BYTES).unwrap() + 1];
        assert!(parse(&oversize).unwrap_err().to_string().contains("byte limit"));
        let mut value = scaffold();
        value["document"]["title"] = json!("x".repeat(MAX_STRING_BYTES + 1));
        assert!(parse_value(&value).unwrap_err().to_string().contains("maximum string length"));
        let nested = format!("{}0{}", "[".repeat(66), "]".repeat(66));
        assert!(parse(nested.as_bytes()).unwrap_err().to_string().contains("maximum JSON depth"));
        let manifest = parse_value(&scaffold()).unwrap();
        let mut too_many = manifest;
        too_many.items = (0..=MAX_ITEMS)
            .map(|index| ItemManifest {
                key: index.to_string(),
                source_refs: Vec::new(),
                milestones: Vec::new(),
            })
            .collect();
        assert!(validate(&too_many).unwrap_err().to_string().contains("entry limit"));
        too_many.items.clear();
        too_many.roles =
            vec![RoleManifest { id: "r".to_string(), title: "R".to_string() }; MAX_PARTIES + 1];
        assert!(validate(&too_many).unwrap_err().to_string().contains("entry limit"));
        too_many.roles.clear();
        too_many.parties = vec![
            PartyManifest {
                key: "p".to_string(),
                party_type: crate::assessment_results::manifest::PartyType::Person,
                name: "P".to_string()
            };
            MAX_PARTIES + 1
        ];
        assert!(validate(&too_many).unwrap_err().to_string().contains("entry limit"));
        too_many.parties.clear();
        let reference: SourceReference =
            serde_json::from_value(item()["source_refs"][0].clone()).unwrap();
        too_many.items = vec![ItemManifest {
            key: "work".to_string(),
            source_refs: vec![reference.clone(); MAX_REFERENCES + 1],
            milestones: Vec::new(),
        }];
        assert!(validate(&too_many).unwrap_err().to_string().contains("must contain 1..="));
        too_many.items[0].source_refs = vec![reference];
        too_many.items[0].milestones =
            vec![MilestoneIdentity { key: "step".to_string() }; MAX_REFERENCES + 1];
        assert!(validate(&too_many).unwrap_err().to_string().contains("entry limit"));
    }

    /// Verify that rejects missing empty metadata invalid timestamp uuid hash and versions.
    #[test]
    fn rejects_missing_empty_metadata_invalid_timestamp_uuid_hash_and_versions() {
        for pointer in [
            "/document/key",
            "/document/title",
            "/document/version",
            "/source/result/key",
            "/source/assessment_results/document_version",
        ] {
            let mut value = scaffold();
            *value.pointer_mut(pointer).unwrap() = json!(" ");
            assert!(parse_value(&value).is_err(), "{pointer}");
        }
        for (pointer, replacement) in [
            ("/schema_version", "forge.poam/2"),
            ("/document/last_modified", "2026-10-02"),
            ("/source/result/uuid", "not-a-uuid"),
            ("/source/assessment_results/root_uuid", "11111111111141118111111111111111"),
            ("/source/assessment_results/expected_sha256", "abcd"),
            ("/source/context/catalog/oscal_version", "1.3.0"),
        ] {
            let mut value = scaffold();
            *value.pointer_mut(pointer).unwrap() = json!(replacement);
            assert!(parse_value(&value).is_err(), "{pointer}");
        }
        let mut missing = scaffold();
        missing["document"].as_object_mut().unwrap().remove("last_modified");
        assert!(parse_value(&missing).unwrap_err().to_string().contains("missing field"));
    }

    /// Verify that requires current forge ar version and preserves companion version pins.
    #[test]
    fn requires_current_forge_ar_version_and_preserves_companion_version_pins() {
        for version in ["1.2.0", "1.2.1", "1.2.2"] {
            let mut value = scaffold();
            value["source"]["assessment_results"]["oscal_version"] = json!(version);
            assert!(parse_value(&value).unwrap_err().to_string().contains("must be 1.2.3"));
        }
        for version in ["1.2.0", "1.2.1", "1.2.2", "1.2.3"] {
            let mut value = scaffold();
            for field in ["assessment_plan", "ssp", "profile", "catalog"] {
                value["source"]["context"][field]["oscal_version"] = json!(version);
            }
            let parsed = parse_value(&value).unwrap();
            assert_eq!(parsed.source.assessment_results.oscal_version, "1.2.3");
            for artifact in [
                &parsed.source.context.assessment_plan,
                &parsed.source.context.ssp,
                &parsed.source.context.profile,
                &parsed.source.context.catalog,
            ] {
                assert_eq!(artifact.oscal_version, version);
            }
        }
    }

    /// Verify that rejects non normalized paths hrefs and duplicate source artifacts.
    #[test]
    fn rejects_non_normalized_paths_hrefs_and_duplicate_source_artifacts() {
        for invalid in [
            "../ar.json",
            "./ar.json",
            "/ar.json",
            "context/./ar.json",
            "context//ar.json",
            "context/ar.json/",
            "C:/ar.json",
            "context\\ar.json",
            "context/ar.json:stream",
            "context./ar.json",
            "CON/ar.json",
            "context/LPT¹.json",
            "context/ar.yaml",
            "context/\nar.json",
        ] {
            let mut value = scaffold();
            value["source"]["assessment_results"]["artifact"] = json!(invalid);
            assert!(parse_value(&value).is_err(), "{invalid}");
        }
        for invalid in [
            "./ar.json",
            "context//ar.json",
            "../ar.json",
            "https://host/ar.json",
            "ar.json?query",
            "ar.json#fragment",
            "ar.json ",
        ] {
            let mut value = scaffold();
            value["source"]["assessment_results"]["href"] = json!(invalid);
            assert!(parse_value(&value).is_err(), "{invalid}");
        }
        let mut value = scaffold();
        value["source"]["assessment_results"]["artifact"] =
            value["source"]["context"]["assessment_plan"]["artifact"].clone();
        assert!(
            parse_value(&value)
                .unwrap_err()
                .to_string()
                .contains("duplicates another source artifact path")
        );
    }

    /// Verify that source artifacts require json extension even with valid identity pins.
    #[test]
    fn source_artifacts_require_json_extension_even_with_valid_identity_pins() {
        let mut value = scaffold();
        value["source"]["assessment_results"]["artifact"] = json!("context/ar.yaml");
        assert!(
            parse_value(&value).unwrap_err().to_string().contains("must name a local .json file")
        );
    }

    /// Verify that rejects evidence index even when identity syntax is valid.
    #[test]
    fn rejects_evidence_index_even_when_identity_syntax_is_valid() {
        let mut value = scaffold();
        value["source"]["context"]["evidence_index"] =
            json!({ "artifact": "evidence.json", "expected_sha256": "c".repeat(64) });
        assert!(
            parse_value(&value).unwrap_err().to_string().contains("evidence_index is unsupported")
        );
    }

    /// Verify that source kind tokens are closed and sort findings before risks.
    #[test]
    fn source_kind_tokens_are_closed_and_sort_findings_before_risks() {
        assert_eq!(serde_json::to_string(&SourceKind::Finding).unwrap(), "\"finding\"");
        assert_eq!(SourceKind::Risk.as_str(), "risk");
        assert!(SourceKind::Finding < SourceKind::Risk);
        assert!(serde_json::from_str::<SourceKind>("\"observation\"").is_err());
    }
}
