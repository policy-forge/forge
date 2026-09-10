//! Pure, explicit draft-only PRD-058 handoff. Filesystem capture and publication
//! belong to the coordinator; this module never reads, writes, or transitions records.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::error;
use super::manifest::PinnedFile;
use super::output::{MAX_OUTPUT_BYTES, OutputArtifact};
use super::render::RenderedAuthorProject;
use crate::ForgeError;
use crate::hashing::sha256_hex;
use crate::lifecycle::record::{
    self, ApprovalPolicy, ArtifactFingerprint, LifecycleRecord, LifecycleState, Party,
    PolicyIdentity, ReviewSchedule,
};

pub const SCHEMA_VERSION: &str = "forge.author-handoff/1";
pub const MAX_POLICIES: usize = 128;
pub const MAX_MANIFEST_BYTES: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HandoffManifest {
    pub schema_version: String,
    pub project: PinnedFile,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub components: Option<PinnedFile>,
    pub provenance: PinnedFile,
    pub policies: Vec<HandoffPolicy>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HandoffPolicy {
    pub author_policy_key: String,
    pub draft: PinnedFile,
    pub policy_key: String,
    pub version_key: String,
    pub title: String,
    pub owner_keys: Vec<String>,
    pub parties: Vec<Party>,
    pub approval_policy: ApprovalPolicy,
    pub review: ReviewSchedule,
}

/// Parse a closed, bounded handoff request with explicit governance configuration.
///
/// # Errors
/// Rejects unknown/duplicate/null fields, unsupported versions, unsafe pins,
/// duplicate identities, and invalid lifecycle metadata before rendering.
pub fn parse_manifest(bytes: &[u8]) -> Result<HandoffManifest, ForgeError> {
    if bytes.len() > MAX_MANIFEST_BYTES {
        return Err(error("handoff manifest exceeds the 2 MiB limit"));
    }
    let value = crate::json_strict::parse_value(
        bytes,
        "author handoff",
        crate::json_strict::Limits { max_depth: 16, max_string_bytes: 4096 },
    )
    .map_err(|cause| error(cause.to_string()))?;
    reject_null(&value)?;
    // PRD-058 permits omitted false separation rules for legacy records. This
    // new request requires a conscious value for every governance option.
    if let Some(policies) = value.get("policies").and_then(serde_json::Value::as_array) {
        for policy in policies {
            let separation = policy.get("approval_policy").and_then(|v| v.get("separation"));
            for field in ["author_reviewer", "author_approver", "reviewer_approver"] {
                if separation.and_then(|v| v.get(field)).is_none() {
                    return Err(error(
                        "handoff requires every approval separation option explicitly",
                    ));
                }
            }
        }
    }
    let manifest: HandoffManifest = serde_json::from_value(value)
        .map_err(|cause| error(format!("invalid author handoff contract: {cause}")))?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

fn reject_null(value: &serde_json::Value) -> Result<(), ForgeError> {
    match value {
        serde_json::Value::Null => Err(error("null is not a handoff value; omit optional fields")),
        serde_json::Value::Array(values) => values.iter().try_for_each(reject_null),
        serde_json::Value::Object(values) => values.values().try_for_each(reject_null),
        _ => Ok(()),
    }
}

fn validate_manifest(manifest: &HandoffManifest) -> Result<(), ForgeError> {
    validate_manifest_bounded(manifest, record_byte_limit()?)
}

fn record_byte_limit() -> Result<usize, ForgeError> {
    usize::try_from(record::MAX_RECORD_BYTES)
        .map_err(|_| error("lifecycle record byte limit is unsupported on this platform"))
}

fn validate_manifest_bounded(manifest: &HandoffManifest, limit: usize) -> Result<(), ForgeError> {
    if manifest.schema_version != SCHEMA_VERSION {
        return Err(error("unsupported author handoff schema version"));
    }
    if manifest.policies.is_empty() || manifest.policies.len() > MAX_POLICIES {
        return Err(error("handoff requires 1 through 128 explicitly selected policies"));
    }
    for pin in std::iter::once(&manifest.project)
        .chain(manifest.components.iter())
        .chain(std::iter::once(&manifest.provenance))
        .chain(manifest.policies.iter().map(|policy| &policy.draft))
    {
        validate_pin(pin)?;
    }
    let mut authors = BTreeSet::new();
    let mut identities = BTreeSet::new();
    for policy in &manifest.policies {
        if !valid_key(&policy.author_policy_key) || !authors.insert(&policy.author_policy_key) {
            return Err(error("handoff author policy keys must be unique portable keys"));
        }
        if !identities.insert((&policy.policy_key, &policy.version_key)) {
            return Err(error("handoff lifecycle policy/version identities must be unique"));
        }
        for required in [record::DeclaredRole::Reviewer, record::DeclaredRole::Approver] {
            if !policy
                .approval_policy
                .required_roles
                .iter()
                .any(|role| role.role == required && role.count > 0)
            {
                return Err(error("handoff requires positive reviewer and approver counts"));
            }
        }
        let record = draft_record(
            policy,
            &format!("policies/{}.md", policy.author_policy_key),
            &policy.draft.expected_sha256,
        );
        record::validate(&record)
            .map_err(|cause| error(format!("invalid handoff governance: {cause}")))?;
        // Also enforce the existing record parser's byte/string/collection limits.
        let bytes = encode_bounded(&record, limit.min(record_byte_limit()?))?;
        record::parse(&bytes).map_err(|cause| error(format!("invalid handoff record: {cause}")))?;
    }
    Ok(())
}

fn validate_pin(pin: &PinnedFile) -> Result<(), ForgeError> {
    crate::json_strict::validate_lowercase_sha256("handoff pin", &pin.expected_sha256)
        .map_err(error)?;
    let path = pin.path.to_str().ok_or_else(|| error("handoff paths must be UTF-8"))?;
    if path.is_empty() || path.len() > 1024 || path.split('/').count() > 32 {
        return Err(error("handoff input path is outside its bounds"));
    }
    for part in path.split('/') {
        let stem = part.split('.').next().unwrap_or_default().to_ascii_lowercase();
        let device = matches!(stem.as_str(), "con" | "prn" | "aux" | "nul")
            || ["com", "lpt"].iter().any(|prefix| {
                stem.strip_prefix(prefix)
                    .is_some_and(|v| v.len() == 1 && matches!(v.as_bytes()[0], b'1'..=b'9'))
            });
        if part.is_empty()
            || part == "."
            || part == ".."
            || part.len() > 255
            || part.trim() != part
            || part.ends_with('.')
            || device
            || part.chars().any(|c| {
                c.is_control() || matches!(c, '\\' | ':' | '<' | '>' | '"' | '|' | '?' | '*')
            })
        {
            return Err(error("handoff input paths require portable descendant components"));
        }
    }
    Ok(())
}

fn valid_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= 64
        && key.as_bytes()[0].is_ascii_lowercase()
        && key.split('-').all(|part| {
            !part.is_empty() && part.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        })
}

fn draft_record(policy: &HandoffPolicy, source_path: &str, source_hash: &str) -> LifecycleRecord {
    let mut owners = policy.owner_keys.clone();
    owners.sort();
    let mut parties = policy.parties.clone();
    for party in &mut parties {
        party.roles.sort();
    }
    parties.sort_by(|left, right| left.key.cmp(&right.key));
    let mut approval_policy = policy.approval_policy.clone();
    approval_policy.required_roles.sort_by_key(|role| role.role);
    LifecycleRecord {
        schema_version: record::SCHEMA_VERSION.to_owned(),
        policy: PolicyIdentity {
            policy_key: policy.policy_key.clone(),
            version_key: policy.version_key.clone(),
            title: policy.title.clone(),
            owner_keys: owners,
            source: ArtifactFingerprint {
                path: source_path.to_owned(),
                sha256: source_hash.to_owned(),
                oscal_type: None,
                root_uuid: None,
            },
            // PRD-058 generated artifacts are OSCAL. The separate receipt binds
            // authoring provenance without laundering it into that contract.
            generated_artifacts: Vec::new(),
        },
        parties,
        approval_policy,
        review: policy.review.clone(),
        state: LifecycleState::Draft,
        replaced_by: None,
        history: Vec::new(),
    }
}

#[derive(Serialize)]
struct HandoffReceipt<'a> {
    schema_version: &'static str,
    manifest_contract_sha256: String,
    project: &'a PinnedFile,
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<&'a PinnedFile>,
    provenance_sha256: String,
    records: Vec<ReceiptRecord<'a>>,
    trust_boundary: &'static str,
}

#[derive(Serialize)]
struct ReceiptRecord<'a> {
    author_policy_key: &'a str,
    policy_key: &'a str,
    version_key: &'a str,
    source_path: String,
    source_sha256: String,
    record_path: String,
    record_sha256: String,
}

/// Prepare all draft records and their receipt without publishing any file.
///
/// The coordinator must first capture every manifest pin through confined reads,
/// rebuild the complete project, compare captured emitted bytes to this generation,
/// and revalidate captures before atomically publishing these artifacts together
/// with the complete generation. Every rendered policy must be explicitly selected.
///
/// # Errors
/// Rejects incomplete selection, draft/provenance tampering, unsafe output labels,
/// invalid governance and aggregate/record output bounds.
pub fn prepare_records(
    manifest: &HandoffManifest,
    rendered: &RenderedAuthorProject,
) -> Result<Vec<OutputArtifact>, ForgeError> {
    prepare_records_bounded(manifest, rendered, MAX_OUTPUT_BYTES)
}

/// Prepare draft records within the remaining complete-generation output budget.
///
/// Requires the same confined capture and exact replay checks as [`prepare_records`].
/// `limit` counts only the new records and receipt; the coordinator must subtract
/// every report, policy, lock, provenance and HTML artifact before calling.
///
/// # Errors
/// Returns the validation errors from [`prepare_records`] or a budget error
/// before record/receipt serialization can exceed the supplied remaining limit.
pub fn prepare_records_bounded(
    manifest: &HandoffManifest,
    rendered: &RenderedAuthorProject,
    limit: usize,
) -> Result<Vec<OutputArtifact>, ForgeError> {
    validate_manifest_bounded(manifest, limit)?;
    if rendered.policies.len() != manifest.policies.len() {
        return Err(error("handoff must explicitly select every rendered policy exactly once"));
    }
    if sha256_hex(&rendered.provenance) != manifest.provenance.expected_sha256 {
        return Err(error("handoff provenance pin does not match the rebuilt generation"));
    }
    let mut policies = manifest.policies.iter().collect::<Vec<_>>();
    policies.sort_by(|a, b| a.author_policy_key.cmp(&b.author_policy_key));
    let mut artifacts = Vec::new();
    let mut records = Vec::new();
    let mut bytes_remaining = MAX_OUTPUT_BYTES
        .checked_sub(rendered.provenance.len())
        .ok_or_else(|| error("handoff exceeds the generation byte budget"))?;
    let mut rendered_keys = BTreeSet::new();
    for policy in &rendered.policies {
        if !rendered_keys.insert(&policy.policy_key) {
            return Err(error("handoff generation contains duplicate policy keys"));
        }
        bytes_remaining = bytes_remaining
            .checked_sub(policy.markdown.len())
            .ok_or_else(|| error("handoff exceeds the generation byte budget"))?;
    }
    bytes_remaining = bytes_remaining.min(limit);
    for policy in policies {
        let draft = rendered
            .policies
            .iter()
            .find(|draft| draft.policy_key == policy.author_policy_key)
            .ok_or_else(|| error("handoff references a policy outside the rebuilt generation"))?;
        let source_path = format!("policies/{}.md", policy.author_policy_key);
        if draft.relative_path != source_path {
            return Err(error("handoff requires the canonical policy output path"));
        }
        let source_sha256 = sha256_hex(&draft.markdown);
        if source_sha256 != policy.draft.expected_sha256 {
            return Err(error("handoff draft pin does not match the rebuilt exact policy bytes"));
        }
        let record = draft_record(policy, &source_path, &source_sha256);
        record::validate(&record)
            .map_err(|cause| error(format!("invalid handoff draft: {cause}")))?;
        let bytes = encode_bounded(&record, bytes_remaining.min(record_byte_limit()?))?;
        record::parse(&bytes).map_err(|cause| error(format!("invalid handoff record: {cause}")))?;
        bytes_remaining -= bytes.len();
        let record_path = format!("{}.lifecycle.json", policy.author_policy_key);
        records.push(ReceiptRecord {
            author_policy_key: &policy.author_policy_key,
            policy_key: &policy.policy_key,
            version_key: &policy.version_key,
            source_path,
            source_sha256,
            record_path: record_path.clone(),
            record_sha256: sha256_hex(&bytes),
        });
        artifacts.push(OutputArtifact { relative_path: record_path, bytes });
    }
    let receipt = HandoffReceipt {
        schema_version: "forge.author-handoff-receipt/1",
        manifest_contract_sha256: sha256_hex(&encode_bounded(manifest, MAX_MANIFEST_BYTES)?),
        project: &manifest.project,
        components: manifest.components.as_ref(),
        provenance_sha256: sha256_hex(&rendered.provenance),
        records,
        trust_boundary: "Draft records only. Parties and review metadata are supplied assertions; human review remains pending.",
    };
    artifacts.push(OutputArtifact {
        relative_path: "handoff.json".into(),
        bytes: encode_bounded(&receipt, bytes_remaining)?,
    });
    Ok(artifacts)
}

struct BoundedJson {
    bytes: Vec<u8>,
    limit: usize,
}
impl std::io::Write for BoundedJson {
    fn write(&mut self, value: &[u8]) -> std::io::Result<usize> {
        if value.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(std::io::Error::other("handoff output exceeds its byte limit"));
        }
        self.bytes.extend_from_slice(value);
        Ok(value.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn encode_bounded(value: &impl Serialize, limit: usize) -> Result<Vec<u8>, ForgeError> {
    let mut output = BoundedJson { bytes: Vec::new(), limit: limit.saturating_sub(1) };
    serde_json::to_writer_pretty(&mut output, value)
        .map_err(|cause| error(format!("cannot render handoff: {cause}")))?;
    if output.bytes.len() >= limit {
        return Err(error("handoff output exceeds its byte limit"));
    }
    output.bytes.push(b'\n');
    Ok(output.bytes)
}

#[cfg(test)]
mod tests {
    use super::super::render::RenderedPolicy;
    use super::*;
    use serde_json::{Value, json};

    fn fixture() -> (HandoffManifest, RenderedAuthorProject) {
        let provenance = b"{\"schema_version\":\"forge.authoring-provenance/1\"}\n".to_vec();
        let rendered = RenderedAuthorProject {
            policies: ["access", "operations"]
                .into_iter()
                .map(|key| RenderedPolicy {
                    policy_key: key.into(),
                    relative_path: format!("policies/{key}.md"),
                    markdown: format!("# Synthetic {key}\n\nDraft state: skeleton-ready.\n")
                        .into_bytes(),
                })
                .collect(),
            provenance,
        };
        let policies = rendered.policies.iter().map(|draft| json!({
            "author_policy_key": draft.policy_key,
            "draft": {"path":format!("prior/{}", draft.relative_path), "expected_sha256":sha256_hex(&draft.markdown)},
            "policy_key":format!("human-{}", draft.policy_key), "version_key":"0.1.0", "title":"Explicit synthetic policy",
            "owner_keys":["owner"],
            "parties":[{"key":"owner", "roles":["owner"]}, {"key":"reviewer", "roles":["reviewer"]}, {"key":"approver", "roles":["approver"]}],
            "approval_policy":{"schema_version":"forge.approval-policy/1", "required_roles":[{"role":"reviewer", "count":1},{"role":"approver", "count":1}], "separation":{"author_reviewer":true,"author_approver":true,"reviewer_approver":true}},
            "review":{"cadence_days":90,"next_review_date":"2027-01-01","due_soon_days":7,"timezone_policy":"date-only"}
        })).collect::<Vec<_>>();
        let value = json!({"schema_version":SCHEMA_VERSION,
            "project":{"path":"project.json","expected_sha256":"a".repeat(64)},
            "provenance":{"path":"prior/provenance.json","expected_sha256":sha256_hex(&rendered.provenance)},
            "policies":policies
        });
        (parse_manifest(&serde_json::to_vec(&value).unwrap()).unwrap(), rendered)
    }

    #[test]
    fn closed_manifest_rejects_forward_duplicate_unknown_null_and_implicit_governance() {
        let (manifest, _) = fixture();
        let base = serde_json::to_value(&manifest).unwrap();
        for (path, value) in [
            ("/schema_version", json!("forge.author-handoff/2")),
            ("/project", Value::Null),
            ("/policies/0/approval_policy/required_roles", json!([])),
            ("/policies/0/owner_keys", json!([])),
            ("/policies/0/review/next_review_date", json!("2027-02-31")),
            ("/policies/0/parties", json!([])),
            ("/policies/0/draft/path", json!("../policy.md")),
        ] {
            let mut changed = base.clone();
            *changed.pointer_mut(path).unwrap() = value;
            assert!(parse_manifest(&serde_json::to_vec(&changed).unwrap()).is_err(), "{path}");
        }
        let mut changed = base.clone();
        changed["policies"][0]["state"] = json!("approved");
        assert!(parse_manifest(&serde_json::to_vec(&changed).unwrap()).is_err());
        let mut changed = base;
        changed["policies"][0]["approval_policy"]["separation"]
            .as_object_mut()
            .unwrap()
            .remove("author_reviewer");
        assert!(parse_manifest(&serde_json::to_vec(&changed).unwrap()).is_err());
        assert!(parse_manifest(br#"{"schema_version":"forge.author-handoff/1","schema_version":"forge.author-handoff/1"}"#).is_err());
    }

    #[test]
    fn exact_bytes_and_provenance_and_complete_unique_selection_are_required() {
        let (mut manifest, mut rendered) = fixture();
        let original = rendered.policies[0].markdown.clone();
        rendered.policies[0].markdown.push(b' ');
        assert!(prepare_records(&manifest, &rendered).is_err());
        rendered.policies[0].markdown = original;
        rendered.provenance.push(b' ');
        assert!(prepare_records(&manifest, &rendered).is_err());
        rendered.provenance.pop();
        manifest.policies[1].author_policy_key = manifest.policies[0].author_policy_key.clone();
        assert!(prepare_records(&manifest, &rendered).is_err());
        manifest.policies.pop();
        assert!(prepare_records(&manifest, &rendered).is_err());
    }

    #[test]
    fn records_are_deterministic_drafts_with_exact_retained_sources_and_receipt() {
        let (mut manifest, rendered) = fixture();
        let first = prepare_records(&manifest, &rendered).unwrap();
        manifest.policies.reverse();
        let second = prepare_records(&manifest, &rendered).unwrap();
        // Record content ordering is canonical. The receipt additionally binds
        // exact manifest contract ordering and therefore intentionally changes.
        for index in 0..2 {
            assert_eq!(first[index].relative_path, second[index].relative_path);
            assert_eq!(first[index].bytes, second[index].bytes);
            let record = record::parse(&first[index].bytes).unwrap();
            assert_eq!(record.state, LifecycleState::Draft);
            assert!(record.history.is_empty());
            assert!(record.replaced_by.is_none());
            assert!(record.policy.generated_artifacts.is_empty());
            let source = rendered
                .policies
                .iter()
                .find(|p| p.relative_path == record.policy.source.path)
                .unwrap();
            assert_eq!(record.policy.source.sha256, sha256_hex(&source.markdown));
        }
        let receipt: Value = serde_json::from_slice(&first[2].bytes).unwrap();
        assert_eq!(receipt["provenance_sha256"], sha256_hex(&rendered.provenance));
        assert_eq!(receipt["records"][0]["record_sha256"], sha256_hex(&first[0].bytes));
        assert_eq!(
            second.iter().map(|a| &a.bytes).collect::<Vec<_>>(),
            prepare_records(&manifest, &rendered)
                .unwrap()
                .iter()
                .map(|a| &a.bytes)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn aliases_and_duplicate_lifecycle_identity_are_rejected() {
        let (manifest, rendered) = fixture();
        for path in
            ["/root/a", "a\\b", "C:a", "a/../b", "a//b", "a/./b", "a.", "CON.txt", "lpt1", " a"]
        {
            let mut changed = manifest.clone();
            changed.policies[0].draft.path = path.into();
            assert!(prepare_records(&changed, &rendered).is_err(), "{path}");
        }
        let mut changed = manifest;
        changed.policies[1].policy_key = changed.policies[0].policy_key.clone();
        assert!(prepare_records(&changed, &rendered).is_err());
    }

    #[test]
    fn output_budget_is_enforced_during_serialization() {
        assert!(encode_bounded(&json!({"text":"long"}), 3).is_err());
        let (mut manifest, rendered) = fixture();
        let artifacts = prepare_records(&manifest, &rendered).unwrap();
        let size = artifacts.iter().map(|item| item.bytes.len()).sum::<usize>();
        assert!(prepare_records_bounded(&manifest, &rendered, size - 1).is_err());
        assert!(prepare_records_bounded(&manifest, &rendered, size).is_ok());
        manifest.policies[0].title = "x".repeat(4097);
        assert!(prepare_records(&manifest, &rendered).is_err());
    }

    #[test]
    fn schema_and_runtime_agree_on_closed_governance_and_identity_contracts() {
        let schema: Value =
            serde_json::from_slice(include_bytes!("../../schemas/author-handoff.schema.json"))
                .unwrap();
        let validator = jsonschema::options().should_validate_formats(true).build(&schema).unwrap();
        let (manifest, _) = fixture();
        let value = serde_json::to_value(&manifest).unwrap();
        assert!(validator.is_valid(&value));
        for path in [
            "/schema_version",
            "/policies/0/review/next_review_date",
            "/policies/0/approval_policy/separation/author_reviewer",
        ] {
            let mut changed = value.clone();
            *changed.pointer_mut(path).unwrap() = Value::Null;
            assert!(!validator.is_valid(&changed), "{path}");
            assert!(parse_manifest(&serde_json::to_vec(&changed).unwrap()).is_err());
        }
        let mut changed = value.clone();
        changed["policies"][0]["history"] = json!([]);
        assert!(!validator.is_valid(&changed));
        assert!(parse_manifest(&serde_json::to_vec(&changed).unwrap()).is_err());
        let mut changed = value;
        changed["policies"][0]["approval_policy"]["required_roles"] =
            json!([{"role":"reviewer","count":1},{"role":"owner","count":1}]);
        assert!(!validator.is_valid(&changed));
        assert!(parse_manifest(&serde_json::to_vec(&changed).unwrap()).is_err());
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn published_records_pass_normal_lifecycle_check_and_existing_history_is_preserved() {
        let (manifest, rendered) = fixture();
        let mut artifacts = prepare_records(&manifest, &rendered).unwrap();
        artifacts.push(OutputArtifact {
            relative_path: "provenance.json".into(),
            bytes: rendered.provenance.clone(),
        });
        artifacts.extend(rendered.policies.iter().map(|p| OutputArtifact {
            relative_path: p.relative_path.clone(),
            bytes: p.markdown.clone(),
        }));
        for _ in 0..2 {
            let directory = tempfile::tempdir().unwrap();
            let root = directory.path().canonicalize().unwrap();
            super::super::output::publish(&root, std::path::Path::new("handoff"), &artifacts)
                .unwrap();
            let paths = manifest
                .policies
                .iter()
                .map(|p| {
                    root.join("handoff").join(format!("{}.lifecycle.json", p.author_policy_key))
                })
                .collect::<Vec<_>>();
            let output = root.join("status.json");
            assert!(
                !crate::lifecycle::execute_check(
                    &paths,
                    &crate::cli::LifecycleOutputFormat::Json,
                    Some(&output)
                )
                .unwrap()
            );
            let before = std::fs::read(&paths[0]).unwrap();
            assert!(
                super::super::output::publish(&root, std::path::Path::new("handoff"), &artifacts)
                    .is_err()
            );
            assert_eq!(before, std::fs::read(&paths[0]).unwrap());
        }
    }
}
