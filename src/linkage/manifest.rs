//! Closed, duplicate-key-safe `forge.linkage/1` manifest contract.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use chrono::{DateTime, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::ForgeError;
use crate::json_strict::{self, Limits};
use crate::mapping::manifest::ResourceType;

pub const SCHEMA_VERSION: &str = "forge.linkage/1";
pub const MAX_MANIFEST_BYTES: u64 = 2 * 1024 * 1024;
pub const MAX_RESOURCES: usize = 64;
pub const MAX_REVIEWERS: usize = 256;
pub const MAX_EVIDENCE_ROOTS: usize = 64;
pub const MAX_EVIDENCE: usize = 10_000;
pub const MAX_LINKS: usize = 10_000;
pub const MAX_SUBJECTS_PER_SIDE: usize = 128;
pub const MAX_EVIDENCE_PER_LINK: usize = 256;
pub const MAX_STRING_BYTES: usize = 16 * 1024;
pub const MAX_EVIDENCE_BYTES: u64 = 50 * 1024 * 1024;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LinkageManifest {
    pub schema_version: String,
    pub project: ProjectManifest,
    #[serde(default)]
    pub reviewers: Vec<ReviewerManifest>,
    pub requirement_resources: Vec<RequirementResourceManifest>,
    pub implementation_resource: ImplementationResourceManifest,
    #[serde(default)]
    pub evidence_roots: Vec<EvidenceRootManifest>,
    #[serde(default)]
    pub evidence: Vec<EvidenceManifest>,
    #[serde(default)]
    pub links: Vec<LinkManifest>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectManifest {
    pub key: String,
    pub title: String,
    #[serde(default = "default_expiring_window_days")]
    pub expiring_window_days: u16,
    #[serde(default = "default_max_evidence_bytes")]
    pub max_evidence_bytes: u64,
    #[serde(default)]
    pub approved_uri_schemes: Vec<String>,
}

const fn default_expiring_window_days() -> u16 {
    30
}

const fn default_max_evidence_bytes() -> u64 {
    10 * 1024 * 1024
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewerManifest {
    pub key: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RequirementResourceManifest {
    pub key: String,
    #[serde(rename = "type")]
    pub resource_type: ResourceType,
    pub artifact: PathBuf,
    pub href: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_catalog: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_catalog_attestation: Option<bool>,
    pub expected_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_resolved_catalog_sha256: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ImplementationResourceType {
    ComponentDefinition,
    SystemSecurityPlan,
}

impl ImplementationResourceType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ComponentDefinition => "component-definition",
            Self::SystemSecurityPlan => "system-security-plan",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ImplementationResourceManifest {
    pub key: String,
    #[serde(rename = "type")]
    pub resource_type: ImplementationResourceType,
    pub artifact: PathBuf,
    pub href: String,
    pub expected_sha256: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRootManifest {
    pub key: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceManifest {
    pub key: String,
    pub title: String,
    pub evidence_type: String,
    pub owner: String,
    pub collected_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_through: Option<NaiveDate>,
    pub sensitivity_label: String,
    pub source_label: String,
    pub location: EvidenceLocation,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum EvidenceLocation {
    Local {
        root_key: String,
        path: PathBuf,
        expected_sha256: String,
        expected_size: u64,
    },
    Uri {
        uri: String,
        unverified: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expected_sha256: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum RequirementSubjectType {
    Control,
    Statement,
}

impl RequirementSubjectType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Control => "control",
            Self::Statement => "statement",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum ImplementationSubjectType {
    ImplementedRequirement,
    Statement,
}

impl ImplementationSubjectType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ImplementedRequirement => "implemented-requirement",
            Self::Statement => "statement",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct RequirementSubjectManifest {
    pub resource_key: String,
    #[serde(rename = "type")]
    pub subject_type: RequirementSubjectType,
    pub id_ref: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct ImplementationSubjectManifest {
    #[serde(rename = "type")]
    pub subject_type: ImplementationSubjectType,
    pub id_ref: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ImplementationStatus {
    Planned,
    Partial,
    Implemented,
    NotApplicable,
    Unknown,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ReviewEvidence {
    pub reviewer_key: String,
    pub reviewed_at: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LinkManifest {
    pub key: String,
    pub requirements: Vec<RequirementSubjectManifest>,
    pub implementations: Vec<ImplementationSubjectManifest>,
    #[serde(default)]
    pub evidence_keys: Vec<String>,
    #[serde(default)]
    pub evidence_required: bool,
    pub responsible_role: String,
    pub implementation_status: ImplementationStatus,
    pub review: ReviewEvidence,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub not_applicable_review: Option<ReviewEvidence>,
    #[serde(default)]
    pub impact_finding_ids: Vec<String>,
    #[serde(default)]
    pub policy_version_keys: Vec<String>,
}

/// Parse and validate one closed linkage manifest.
///
/// # Errors
///
/// Returns [`ForgeError::Linkage`] for an unsupported or malformed contract, a duplicate key,
/// invalid Unicode/JSON, a bound violation, or an intrinsic/cross-reference validation failure.
pub fn parse(bytes: &[u8]) -> Result<LinkageManifest, ForgeError> {
    if bytes.len() as u64 > MAX_MANIFEST_BYTES {
        return Err(error(format!("manifest exceeds the {MAX_MANIFEST_BYTES} byte limit")));
    }
    let value = json_strict::parse_value(
        bytes,
        "linkage manifest",
        Limits { max_depth: 64, max_string_bytes: MAX_STRING_BYTES },
    )
    .map_err(|cause| error(cause.to_string()))?;
    let manifest: LinkageManifest = serde_json::from_value(value)
        .map_err(|cause| error(format!("invalid linkage contract: {cause}")))?;
    validate(&manifest)?;
    Ok(manifest)
}

/// Validate manifest bounds and cross-references that do not require filesystem access.
///
/// # Errors
///
/// Returns [`ForgeError::Linkage`] when a field, bound, path, hash, timestamp, uniqueness rule,
/// status assertion, or local manifest cross-reference is invalid.
pub fn validate(manifest: &LinkageManifest) -> Result<(), ForgeError> {
    if manifest.schema_version != SCHEMA_VERSION {
        return Err(error(format!(
            "unsupported schema_version '{}'; expected {SCHEMA_VERSION}",
            json_strict::bounded(&manifest.schema_version)
        )));
    }
    non_empty("$.project.key", &manifest.project.key)?;
    non_empty("$.project.title", &manifest.project.title)?;
    if manifest.project.max_evidence_bytes == 0
        || manifest.project.max_evidence_bytes > MAX_EVIDENCE_BYTES
    {
        return Err(error(format!(
            "$.project.max_evidence_bytes must be between 1 and {MAX_EVIDENCE_BYTES}"
        )));
    }
    validate_uri_schemes(&manifest.project.approved_uri_schemes)?;
    let reviewers = validate_reviewers(&manifest.reviewers)?;
    let resources = validate_requirement_resources(&manifest.requirement_resources)?;
    validate_implementation_resource(&manifest.implementation_resource)?;
    let roots = validate_evidence_roots(&manifest.evidence_roots)?;
    let evidence = validate_evidence(&manifest.evidence, &roots)?;
    validate_links(&manifest.links, &reviewers, &resources, &evidence)?;
    Ok(())
}

fn validate_uri_schemes(values: &[String]) -> Result<(), ForgeError> {
    let mut unique = BTreeSet::new();
    for (index, value) in values.iter().enumerate() {
        let valid = value.len() <= 64
            && value.bytes().next().is_some_and(|byte| byte.is_ascii_lowercase())
            && value.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"+-.".contains(&byte)
            });
        if !valid || value == "https" || !unique.insert(value.as_str()) {
            return Err(error(format!(
                "$.project.approved_uri_schemes[{index}] must be a unique lowercase custom URI scheme"
            )));
        }
    }
    Ok(())
}

fn validate_reviewers(values: &[ReviewerManifest]) -> Result<BTreeSet<&str>, ForgeError> {
    if values.len() > MAX_REVIEWERS {
        return Err(error(format!("$.reviewers exceeds the {MAX_REVIEWERS} entry limit")));
    }
    let mut keys = BTreeSet::new();
    for (index, value) in values.iter().enumerate() {
        non_empty(&format!("$.reviewers[{index}].key"), &value.key)?;
        non_empty(&format!("$.reviewers[{index}].name"), &value.name)?;
        if !keys.insert(value.key.as_str()) {
            return Err(error(format!("$.reviewers[{index}].key duplicates a reviewer key")));
        }
    }
    Ok(keys)
}

fn validate_requirement_resources(
    values: &[RequirementResourceManifest],
) -> Result<BTreeSet<&str>, ForgeError> {
    if values.is_empty() || values.len() > MAX_RESOURCES {
        return Err(error(format!(
            "$.requirement_resources must contain 1..={MAX_RESOURCES} entries"
        )));
    }
    let mut keys = BTreeSet::new();
    for (index, value) in values.iter().enumerate() {
        let path = format!("$.requirement_resources[{index}]");
        non_empty(&format!("{path}.key"), &value.key)?;
        non_empty(&format!("{path}.href"), &value.href)?;
        local_json_path(&format!("{path}.artifact"), &value.artifact)?;
        sha256(&format!("{path}.expected_sha256"), &value.expected_sha256)?;
        match value.resource_type {
            ResourceType::Profile => {
                let companion = value.resolved_catalog.as_ref().ok_or_else(|| {
                    error(format!("{path}.resolved_catalog is required for a Profile"))
                })?;
                local_json_path(&format!("{path}.resolved_catalog"), companion)?;
                if value.resolved_catalog_attestation != Some(true) {
                    return Err(error(format!(
                        "{path}.resolved_catalog_attestation must be true after review"
                    )));
                }
                sha256(
                    &format!("{path}.expected_resolved_catalog_sha256"),
                    value.expected_resolved_catalog_sha256.as_deref().ok_or_else(|| {
                        error(format!(
                            "{path}.expected_resolved_catalog_sha256 is required for a Profile"
                        ))
                    })?,
                )?;
            }
            ResourceType::Catalog
                if value.resolved_catalog.is_some()
                    || value.resolved_catalog_attestation.is_some()
                    || value.expected_resolved_catalog_sha256.is_some() =>
            {
                return Err(error(format!(
                    "{path} resolved Catalog fields are only valid for Profile resources"
                )));
            }
            ResourceType::Catalog => {}
        }
        if !keys.insert(value.key.as_str()) {
            return Err(error(format!("{path}.key duplicates a requirement resource key")));
        }
    }
    Ok(keys)
}

fn validate_implementation_resource(
    value: &ImplementationResourceManifest,
) -> Result<(), ForgeError> {
    non_empty("$.implementation_resource.key", &value.key)?;
    non_empty("$.implementation_resource.href", &value.href)?;
    local_json_path("$.implementation_resource.artifact", &value.artifact)?;
    sha256("$.implementation_resource.expected_sha256", &value.expected_sha256)
}

fn validate_evidence_roots(values: &[EvidenceRootManifest]) -> Result<BTreeSet<&str>, ForgeError> {
    if values.len() > MAX_EVIDENCE_ROOTS {
        return Err(error(format!(
            "$.evidence_roots exceeds the {MAX_EVIDENCE_ROOTS} entry limit"
        )));
    }
    let mut keys = BTreeSet::new();
    for (index, value) in values.iter().enumerate() {
        non_empty(&format!("$.evidence_roots[{index}].key"), &value.key)?;
        local_path(&format!("$.evidence_roots[{index}].path"), &value.path)?;
        if !keys.insert(value.key.as_str()) {
            return Err(error(format!("$.evidence_roots[{index}].key duplicates a root key")));
        }
    }
    Ok(keys)
}

fn validate_evidence<'a>(
    values: &'a [EvidenceManifest],
    roots: &BTreeSet<&str>,
) -> Result<BTreeMap<&'a str, &'a EvidenceManifest>, ForgeError> {
    if values.len() > MAX_EVIDENCE {
        return Err(error(format!("$.evidence exceeds the {MAX_EVIDENCE} entry limit")));
    }
    let mut evidence = BTreeMap::new();
    for (index, value) in values.iter().enumerate() {
        let path = format!("$.evidence[{index}]");
        for (field, text) in [
            ("key", &value.key),
            ("title", &value.title),
            ("evidence_type", &value.evidence_type),
            ("owner", &value.owner),
            ("sensitivity_label", &value.sensitivity_label),
            ("source_label", &value.source_label),
        ] {
            non_empty(&format!("{path}.{field}"), text)?;
        }
        timestamp(&format!("{path}.collected_at"), &value.collected_at)?;
        match &value.location {
            EvidenceLocation::Local { root_key, path: relative, expected_sha256, .. } => {
                if !roots.contains(root_key.as_str()) {
                    return Err(error(format!("{path}.root_key references an unknown root")));
                }
                local_path(&format!("{path}.path"), relative)?;
                sha256(&format!("{path}.expected_sha256"), expected_sha256)?;
            }
            EvidenceLocation::Uri { uri, unverified, expected_sha256 } => {
                non_empty(&format!("{path}.uri"), uri)?;
                if !unverified {
                    return Err(error(format!(
                        "{path}.unverified must be true because URI evidence is never fetched"
                    )));
                }
                if let Some(hash) = expected_sha256 {
                    sha256(&format!("{path}.expected_sha256"), hash)?;
                }
            }
        }
        if evidence.insert(value.key.as_str(), value).is_some() {
            return Err(error(format!("{path}.key duplicates an evidence key")));
        }
    }
    Ok(evidence)
}

fn validate_links(
    values: &[LinkManifest],
    reviewers: &BTreeSet<&str>,
    resources: &BTreeSet<&str>,
    evidence: &BTreeMap<&str, &EvidenceManifest>,
) -> Result<(), ForgeError> {
    if values.len() > MAX_LINKS {
        return Err(error(format!("$.links exceeds the {MAX_LINKS} entry limit")));
    }
    let mut keys = BTreeSet::new();
    for (index, value) in values.iter().enumerate() {
        let path = format!("$.links[{index}]");
        non_empty(&format!("{path}.key"), &value.key)?;
        non_empty(&format!("{path}.responsible_role"), &value.responsible_role)?;
        review(&path, &value.review, reviewers)?;
        if value.implementation_status == ImplementationStatus::NotApplicable {
            review(
                &format!("{path}.not_applicable_review"),
                value.not_applicable_review.as_ref().ok_or_else(|| {
                    error(format!(
                        "{path}.not_applicable_review is required for a not-applicable assertion"
                    ))
                })?,
                reviewers,
            )?;
        } else if value.not_applicable_review.is_some() {
            return Err(error(format!(
                "{path}.not_applicable_review is only valid for not-applicable status"
            )));
        }
        if value.requirements.is_empty() || value.requirements.len() > MAX_SUBJECTS_PER_SIDE {
            return Err(error(format!(
                "{path}.requirements must contain 1..={MAX_SUBJECTS_PER_SIDE} entries"
            )));
        }
        if value.implementations.is_empty() || value.implementations.len() > MAX_SUBJECTS_PER_SIDE {
            return Err(error(format!(
                "{path}.implementations must contain 1..={MAX_SUBJECTS_PER_SIDE} entries"
            )));
        }
        let mut requirement_subjects = BTreeSet::new();
        for (subject_index, subject) in value.requirements.iter().enumerate() {
            if !resources.contains(subject.resource_key.as_str()) {
                return Err(error(format!(
                    "{path}.requirements[{subject_index}].resource_key is unknown"
                )));
            }
            non_empty(&format!("{path}.requirements[{subject_index}].id_ref"), &subject.id_ref)?;
            if !requirement_subjects.insert(subject) {
                return Err(error(format!(
                    "{path}.requirements[{subject_index}] duplicates a requirement subject"
                )));
            }
        }
        let mut implementation_subjects = BTreeSet::new();
        for (subject_index, subject) in value.implementations.iter().enumerate() {
            non_empty(&format!("{path}.implementations[{subject_index}].id_ref"), &subject.id_ref)?;
            if !implementation_subjects.insert(subject) {
                return Err(error(format!(
                    "{path}.implementations[{subject_index}] duplicates an implementation subject"
                )));
            }
        }
        if value.evidence_keys.len() > MAX_EVIDENCE_PER_LINK {
            return Err(error(format!(
                "{path}.evidence_keys exceeds the {MAX_EVIDENCE_PER_LINK} entry limit"
            )));
        }
        let mut evidence_keys = BTreeSet::new();
        for (evidence_index, key) in value.evidence_keys.iter().enumerate() {
            if !evidence.contains_key(key.as_str()) {
                return Err(error(format!("{path}.evidence_keys[{evidence_index}] is unknown")));
            }
            if !evidence_keys.insert(key.as_str()) {
                return Err(error(format!(
                    "{path}.evidence_keys[{evidence_index}] duplicates an evidence reference"
                )));
            }
        }
        unique_non_empty(&format!("{path}.impact_finding_ids"), &value.impact_finding_ids)?;
        unique_non_empty(&format!("{path}.policy_version_keys"), &value.policy_version_keys)?;
        if !keys.insert(value.key.as_str()) {
            return Err(error(format!("{path}.key duplicates a link key")));
        }
    }
    Ok(())
}

fn review(
    path: &str,
    value: &ReviewEvidence,
    reviewers: &BTreeSet<&str>,
) -> Result<(), ForgeError> {
    if !reviewers.contains(value.reviewer_key.as_str()) {
        return Err(error(format!("{path}.reviewer_key references an unknown reviewer")));
    }
    timestamp(&format!("{path}.reviewed_at"), &value.reviewed_at)?;
    non_empty(&format!("{path}.rationale"), &value.rationale)
}

fn unique_non_empty(path: &str, values: &[String]) -> Result<(), ForgeError> {
    let mut unique = BTreeSet::new();
    for (index, value) in values.iter().enumerate() {
        non_empty(&format!("{path}[{index}]"), value)?;
        if !unique.insert(value.as_str()) {
            return Err(error(format!("{path}[{index}] duplicates an entry")));
        }
    }
    Ok(())
}

fn local_json_path(path: &str, value: &Path) -> Result<(), ForgeError> {
    local_path(path, value)?;
    if value.extension().and_then(|extension| extension.to_str()) != Some("json") {
        return Err(error(format!("{path} must be a local .json file")));
    }
    Ok(())
}

fn local_path(path: &str, value: &Path) -> Result<(), ForgeError> {
    if value.as_os_str().is_empty()
        || value.is_absolute()
        || value.components().any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(error(format!(
            "{path} must be a relative descendant path without '.', '..', or leading separators"
        )));
    }
    Ok(())
}

fn timestamp(path: &str, value: &str) -> Result<(), ForgeError> {
    DateTime::parse_from_rfc3339(value)
        .map(|_| ())
        .map_err(|_| error(format!("{path} must be an RFC 3339 timestamp")))
}

fn sha256(path: &str, value: &str) -> Result<(), ForgeError> {
    json_strict::validate_lowercase_sha256(path, value).map_err(error)
}

fn non_empty(path: &str, value: &str) -> Result<(), ForgeError> {
    if value.trim().is_empty() { Err(error(format!("{path} must not be empty"))) } else { Ok(()) }
}

fn error(detail: impl Into<String>) -> ForgeError {
    ForgeError::Linkage(detail.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_duplicate_unknown_and_unbounded_manifest_data() {
        let duplicate =
            br#"{"schema_version":"forge.linkage/1","schema_version":"forge.linkage/1"}"#;
        assert!(parse(duplicate).unwrap_err().to_string().contains("duplicate object key"));

        let unknown = br#"{
            "schema_version":"forge.linkage/1",
            "project":{"key":"p","title":"P","surprise":true},
            "requirement_resources":[],
            "implementation_resource":{"key":"i","type":"component-definition","artifact":"i.json","href":"i.json","expected_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}
        }"#;
        assert!(parse(unknown).unwrap_err().to_string().contains("unknown field"));

        let oversized = vec![b' '; usize::try_from(MAX_MANIFEST_BYTES).unwrap() + 1];
        assert!(parse(&oversized).unwrap_err().to_string().contains("byte limit"));
    }

    #[test]
    fn rejects_traversal_and_unreviewed_uri_or_not_applicable_assertions() {
        assert!(local_path("$.path", Path::new("../secret")).is_err());
        assert!(local_path("$.path", Path::new("/secret")).is_err());
        assert!(local_path("$.path", Path::new("evidence/file.txt")).is_ok());
        assert!(validate_uri_schemes(&["HTTPS".to_string()]).is_err());
        assert!(validate_uri_schemes(&["vault+corp".to_string()]).is_ok());
    }
}
