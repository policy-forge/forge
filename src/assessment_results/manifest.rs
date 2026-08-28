//! Closed, bounded `forge.assessment-results/1` manifest contract.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::ForgeError;
use crate::json_strict::{self, Limits};

/// Assessment Results manifest schema supported by this release.
pub const MANIFEST_SCHEMA_VERSION: &str = "forge.assessment-results/1";
/// Maximum manifest byte size.
pub const MAX_MANIFEST_BYTES: u64 = 4 * 1024 * 1024;
/// Maximum decoded string size.
pub const MAX_STRING_BYTES: usize = 64 * 1024;
/// Maximum number of conclusion objects of one type.
pub const MAX_CONCLUSIONS: usize = 10_000;
/// Maximum number of references attached to one conclusion.
pub const MAX_REFERENCES: usize = 1_000;
/// Maximum assessor parties or roles.
pub const MAX_PARTIES: usize = 1_000;

/// Reviewer-authored Assessment Results input.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AssessmentResultsManifest {
    pub schema_version: String,
    pub document: DocumentManifest,
    pub context: ContextManifest,
    pub roles: Vec<RoleManifest>,
    pub parties: Vec<PartyManifest>,
    pub result: ResultManifest,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentManifest {
    pub key: String,
    pub title: String,
    pub version: String,
    pub last_modified: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContextManifest {
    pub assessment_plan: ArtifactManifest,
    pub ssp: ArtifactManifest,
    pub profile: ArtifactManifest,
    pub catalog: ArtifactManifest,
    #[serde(default)]
    pub evidence_index: Option<EvidenceIndexManifest>,
}

/// Exact local OSCAL input identity.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactManifest {
    pub artifact: PathBuf,
    pub href: String,
    pub expected_sha256: String,
    pub root_uuid: String,
    pub document_version: String,
    pub oscal_version: String,
}

/// Optional identity-only adapter for a PRD 060 `forge.linkage-index/1` artifact.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceIndexManifest {
    pub artifact: PathBuf,
    pub expected_sha256: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RoleManifest {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PartyManifest {
    pub key: String,
    #[serde(rename = "type")]
    pub party_type: PartyType,
    pub name: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PartyType {
    Person,
    Organization,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResultManifest {
    pub key: String,
    pub title: String,
    pub description: String,
    pub start: String,
    #[serde(default)]
    pub end: Option<String>,
    pub control_ids: Vec<String>,
    #[serde(default)]
    pub objective_ids: Vec<String>,
    #[serde(default)]
    pub observations: Vec<ObservationManifest>,
    #[serde(default)]
    pub findings: Vec<FindingManifest>,
    #[serde(default)]
    pub risks: Vec<RiskManifest>,
    #[serde(default)]
    pub relationships: Vec<RelationshipManifest>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservationManifest {
    pub key: String,
    #[serde(default)]
    pub title: Option<String>,
    pub description: String,
    pub provenance: ProvenanceManifest,
    pub subjects: Vec<SubjectReferenceManifest>,
    #[serde(default)]
    pub task_uuids: Vec<String>,
    #[serde(default)]
    pub evidence_keys: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FindingManifest {
    pub key: String,
    pub title: String,
    pub description: String,
    pub provenance: ProvenanceManifest,
    pub target: FindingTargetManifest,
    #[serde(default)]
    pub implementation_statement_uuid: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FindingTargetManifest {
    #[serde(rename = "type")]
    pub target_type: FindingTargetType,
    pub id: String,
    pub state: FindingState,
    #[serde(default)]
    pub reason: Option<FindingReason>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum FindingTargetType {
    StatementId,
    ObjectiveId,
}

impl FindingTargetType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StatementId => "statement-id",
            Self::ObjectiveId => "objective-id",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum FindingState {
    Satisfied,
    NotSatisfied,
}

impl FindingState {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Satisfied => "satisfied",
            Self::NotSatisfied => "not-satisfied",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum FindingReason {
    Pass,
    Fail,
    Other,
}

impl FindingReason {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::Other => "other",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RiskManifest {
    pub key: String,
    pub title: String,
    pub description: String,
    pub statement: String,
    pub status: RiskStatus,
    #[serde(default)]
    pub severity: Option<String>,
    #[serde(default)]
    pub confidence: Option<f64>,
    pub provenance: ProvenanceManifest,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RiskStatus {
    Open,
    Investigating,
    Remediating,
    DeviationRequested,
    DeviationApproved,
    Closed,
}

impl RiskStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Investigating => "investigating",
            Self::Remediating => "remediating",
            Self::DeviationRequested => "deviation-requested",
            Self::DeviationApproved => "deviation-approved",
            Self::Closed => "closed",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProvenanceManifest {
    pub assessor_key: String,
    pub role_id: String,
    pub start: String,
    #[serde(default)]
    pub end: Option<String>,
    pub method: AssessmentMethod,
    pub rationale: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum AssessmentMethod {
    #[serde(rename = "EXAMINE")]
    Examine,
    #[serde(rename = "INTERVIEW")]
    Interview,
    #[serde(rename = "TEST")]
    Test,
    #[serde(rename = "UNKNOWN")]
    Unknown,
}

impl AssessmentMethod {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Examine => "EXAMINE",
            Self::Interview => "INTERVIEW",
            Self::Test => "TEST",
            Self::Unknown => "UNKNOWN",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct SubjectReferenceManifest {
    #[serde(rename = "type")]
    pub subject_type: SubjectType,
    pub uuid: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum SubjectType {
    Component,
    InventoryItem,
    Location,
    Party,
    User,
    Resource,
}

impl SubjectType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Component => "component",
            Self::InventoryItem => "inventory-item",
            Self::Location => "location",
            Self::Party => "party",
            Self::User => "user",
            Self::Resource => "resource",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct RelationshipManifest {
    pub from: RelationshipEndpoint,
    pub to: RelationshipEndpoint,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct RelationshipEndpoint {
    #[serde(rename = "type")]
    pub object_type: ConclusionType,
    pub key: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum ConclusionType {
    Observation,
    Finding,
    Risk,
}

impl ConclusionType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Observation => "observation",
            Self::Finding => "finding",
            Self::Risk => "risk",
        }
    }
}

/// Strictly parse and validate a complete manifest.
///
/// # Errors
///
/// Returns an error when JSON is invalid or duplicated, a resource bound is
/// exceeded, the contract is unsupported, or any semantic invariant fails.
pub fn parse(bytes: &[u8]) -> Result<AssessmentResultsManifest, ForgeError> {
    if bytes.len() as u64 > MAX_MANIFEST_BYTES {
        return Err(error(format!("manifest exceeds the {MAX_MANIFEST_BYTES} byte limit")));
    }
    let value = json_strict::parse_value(
        bytes,
        "Assessment Results manifest",
        Limits { max_depth: 64, max_string_bytes: MAX_STRING_BYTES },
    )
    .map_err(|cause| error(cause.to_string()))?;
    let manifest: AssessmentResultsManifest = serde_json::from_value(value)
        .map_err(|cause| error(format!("invalid manifest contract: {cause}")))?;
    validate(&manifest)?;
    Ok(manifest)
}

fn validate(manifest: &AssessmentResultsManifest) -> Result<(), ForgeError> {
    if manifest.schema_version != MANIFEST_SCHEMA_VERSION {
        return Err(error(format!(
            "unsupported schema_version '{}'; expected {MANIFEST_SCHEMA_VERSION}",
            bounded(&manifest.schema_version)
        )));
    }
    non_empty("$.document.key", &manifest.document.key)?;
    non_empty("$.document.title", &manifest.document.title)?;
    non_empty("$.document.version", &manifest.document.version)?;
    timestamp("$.document.last_modified", &manifest.document.last_modified)?;
    validate_context(&manifest.context)?;

    if manifest.roles.is_empty() || manifest.roles.len() > MAX_PARTIES {
        return Err(error(format!("$.roles must contain 1..={MAX_PARTIES} entries")));
    }
    let mut roles = BTreeSet::new();
    for (index, role) in manifest.roles.iter().enumerate() {
        non_empty(&format!("$.roles[{index}].id"), &role.id)?;
        non_empty(&format!("$.roles[{index}].title"), &role.title)?;
        if !roles.insert(role.id.as_str()) {
            return Err(error(format!(
                "$.roles[{index}].id duplicates role '{}'",
                bounded(&role.id)
            )));
        }
    }

    if manifest.parties.is_empty() || manifest.parties.len() > MAX_PARTIES {
        return Err(error(format!("$.parties must contain 1..={MAX_PARTIES} entries")));
    }
    let mut parties = BTreeSet::new();
    for (index, party) in manifest.parties.iter().enumerate() {
        non_empty(&format!("$.parties[{index}].key"), &party.key)?;
        non_empty(&format!("$.parties[{index}].name"), &party.name)?;
        if !parties.insert(party.key.as_str()) {
            return Err(error(format!(
                "$.parties[{index}].key duplicates party key '{}'",
                bounded(&party.key)
            )));
        }
    }

    validate_result(&manifest.result, &parties, &roles)
}

fn validate_context(context: &ContextManifest) -> Result<(), ForgeError> {
    for (path, artifact) in [
        ("$.context.assessment_plan", &context.assessment_plan),
        ("$.context.ssp", &context.ssp),
        ("$.context.profile", &context.profile),
        ("$.context.catalog", &context.catalog),
    ] {
        validate_artifact(path, artifact)?;
    }
    if let Some(index) = &context.evidence_index {
        relative_json_path("$.context.evidence_index.artifact", &index.artifact)?;
        json_strict::validate_lowercase_sha256(
            "$.context.evidence_index.expected_sha256",
            &index.expected_sha256,
        )
        .map_err(error)?;
    }
    Ok(())
}

fn validate_artifact(path: &str, artifact: &ArtifactManifest) -> Result<(), ForgeError> {
    relative_json_path(&format!("{path}.artifact"), &artifact.artifact)?;
    relative_href(&format!("{path}.href"), &artifact.href)?;
    json_strict::validate_lowercase_sha256(
        &format!("{path}.expected_sha256"),
        &artifact.expected_sha256,
    )
    .map_err(error)?;
    uuid(&format!("{path}.root_uuid"), &artifact.root_uuid)?;
    non_empty(&format!("{path}.document_version"), &artifact.document_version)?;
    non_empty(&format!("{path}.oscal_version"), &artifact.oscal_version)
}

fn validate_result(
    result: &ResultManifest,
    parties: &BTreeSet<&str>,
    roles: &BTreeSet<&str>,
) -> Result<(), ForgeError> {
    non_empty("$.result.key", &result.key)?;
    non_empty("$.result.title", &result.title)?;
    non_empty("$.result.description", &result.description)?;
    time_range("$.result", &result.start, result.end.as_deref())?;
    unique_non_empty("$.result.control_ids", &result.control_ids, true, MAX_CONCLUSIONS)?;
    unique_non_empty("$.result.objective_ids", &result.objective_ids, false, MAX_CONCLUSIONS)?;
    for (label, count) in [
        ("observations", result.observations.len()),
        ("findings", result.findings.len()),
        ("risks", result.risks.len()),
    ] {
        if count > MAX_CONCLUSIONS {
            return Err(error(format!(
                "$.result.{label} exceeds the {MAX_CONCLUSIONS} entry limit"
            )));
        }
    }

    let mut conclusion_keys = BTreeMap::new();
    for (index, observation) in result.observations.iter().enumerate() {
        let path = format!("$.result.observations[{index}]");
        conclusion_key(&path, ConclusionType::Observation, &observation.key, &mut conclusion_keys)?;
        non_empty(&format!("{path}.description"), &observation.description)?;
        if let Some(title) = &observation.title {
            non_empty(&format!("{path}.title"), title)?;
        }
        provenance(&format!("{path}.provenance"), &observation.provenance, parties, roles)?;
        if observation.subjects.is_empty() || observation.subjects.len() > MAX_REFERENCES {
            return Err(error(format!(
                "{path}.subjects must contain 1..={MAX_REFERENCES} entries"
            )));
        }
        let mut subjects = BTreeSet::new();
        for (subject_index, subject) in observation.subjects.iter().enumerate() {
            uuid(&format!("{path}.subjects[{subject_index}].uuid"), &subject.uuid)?;
            if !subjects.insert((subject.subject_type, subject.uuid.as_str())) {
                return Err(error(format!(
                    "{path}.subjects[{subject_index}] duplicates a subject reference"
                )));
            }
        }
        unique_non_empty(
            &format!("{path}.task_uuids"),
            &observation.task_uuids,
            false,
            MAX_REFERENCES,
        )?;
        for (task_index, task_uuid) in observation.task_uuids.iter().enumerate() {
            uuid(&format!("{path}.task_uuids[{task_index}]"), task_uuid)?;
        }
        unique_non_empty(
            &format!("{path}.evidence_keys"),
            &observation.evidence_keys,
            false,
            MAX_REFERENCES,
        )?;
    }

    for (index, finding) in result.findings.iter().enumerate() {
        let path = format!("$.result.findings[{index}]");
        conclusion_key(&path, ConclusionType::Finding, &finding.key, &mut conclusion_keys)?;
        non_empty(&format!("{path}.title"), &finding.title)?;
        non_empty(&format!("{path}.description"), &finding.description)?;
        provenance(&format!("{path}.provenance"), &finding.provenance, parties, roles)?;
        non_empty(&format!("{path}.target.id"), &finding.target.id)?;
        if let Some(value) = &finding.implementation_statement_uuid {
            uuid(&format!("{path}.implementation_statement_uuid"), value)?;
        }
    }

    for (index, risk) in result.risks.iter().enumerate() {
        let path = format!("$.result.risks[{index}]");
        conclusion_key(&path, ConclusionType::Risk, &risk.key, &mut conclusion_keys)?;
        non_empty(&format!("{path}.title"), &risk.title)?;
        non_empty(&format!("{path}.description"), &risk.description)?;
        non_empty(&format!("{path}.statement"), &risk.statement)?;
        if let Some(severity) = &risk.severity {
            non_empty(&format!("{path}.severity"), severity)?;
        }
        if let Some(confidence) = risk.confidence
            && (!confidence.is_finite() || !(0.0..=1.0).contains(&confidence))
        {
            return Err(error(format!("{path}.confidence must be between 0 and 1")));
        }
        provenance(&format!("{path}.provenance"), &risk.provenance, parties, roles)?;
    }

    validate_relationships(result, &conclusion_keys)
}

fn conclusion_key<'a>(
    path: &str,
    object_type: ConclusionType,
    key: &'a str,
    keys: &mut BTreeMap<&'a str, ConclusionType>,
) -> Result<(), ForgeError> {
    non_empty(&format!("{path}.key"), key)?;
    if let Some(existing) = keys.insert(key, object_type) {
        return Err(error(format!(
            "{path}.key '{}' duplicates a {} key",
            bounded(key),
            existing.as_str()
        )));
    }
    Ok(())
}

fn validate_relationships(
    result: &ResultManifest,
    keys: &BTreeMap<&str, ConclusionType>,
) -> Result<(), ForgeError> {
    if result.relationships.len() > MAX_CONCLUSIONS.saturating_mul(2) {
        return Err(error("$.result.relationships exceeds the configured graph edge limit"));
    }
    let mut unique = BTreeSet::new();
    let mut findings_with_observation = BTreeSet::new();
    let mut risks_with_finding = BTreeSet::new();
    for (index, relationship) in result.relationships.iter().enumerate() {
        let path = format!("$.result.relationships[{index}]");
        non_empty(&format!("{path}.from.key"), &relationship.from.key)?;
        non_empty(&format!("{path}.to.key"), &relationship.to.key)?;
        for (side, endpoint) in [("from", &relationship.from), ("to", &relationship.to)] {
            match keys.get(endpoint.key.as_str()) {
                Some(actual) if *actual == endpoint.object_type => {}
                Some(actual) => {
                    return Err(error(format!(
                        "{path}.{side} declares {} key '{}' but that key belongs to {}",
                        endpoint.object_type.as_str(),
                        bounded(&endpoint.key),
                        actual.as_str()
                    )));
                }
                None => {
                    return Err(error(format!(
                        "{path}.{side} references missing {} key '{}'",
                        endpoint.object_type.as_str(),
                        bounded(&endpoint.key)
                    )));
                }
            }
        }
        let allowed = matches!(
            (relationship.from.object_type, relationship.to.object_type),
            (ConclusionType::Observation, ConclusionType::Finding)
                | (ConclusionType::Finding, ConclusionType::Risk)
        );
        if !allowed {
            return Err(error(format!(
                "{path} has wrong-side or circular relationship {} -> {}; only observation -> finding and finding -> risk are supported",
                relationship.from.object_type.as_str(),
                relationship.to.object_type.as_str()
            )));
        }
        if !unique.insert(relationship) {
            return Err(error(format!("{path} duplicates an existing relationship")));
        }
        match (relationship.from.object_type, relationship.to.object_type) {
            (ConclusionType::Observation, ConclusionType::Finding) => {
                findings_with_observation.insert(relationship.to.key.as_str());
            }
            (ConclusionType::Finding, ConclusionType::Risk) => {
                risks_with_finding.insert(relationship.to.key.as_str());
            }
            _ => {}
        }
    }
    for finding in &result.findings {
        if !findings_with_observation.contains(finding.key.as_str()) {
            return Err(error(format!(
                "finding '{}' requires an explicit observation -> finding relationship",
                bounded(&finding.key)
            )));
        }
    }
    for risk in &result.risks {
        if !risks_with_finding.contains(risk.key.as_str()) {
            return Err(error(format!(
                "risk '{}' requires an explicit finding -> risk relationship",
                bounded(&risk.key)
            )));
        }
    }
    Ok(())
}

fn provenance(
    path: &str,
    provenance: &ProvenanceManifest,
    parties: &BTreeSet<&str>,
    roles: &BTreeSet<&str>,
) -> Result<(), ForgeError> {
    if !parties.contains(provenance.assessor_key.as_str()) {
        return Err(error(format!(
            "{path}.assessor_key references unknown party '{}'",
            bounded(&provenance.assessor_key)
        )));
    }
    if !roles.contains(provenance.role_id.as_str()) {
        return Err(error(format!(
            "{path}.role_id references unknown role '{}'",
            bounded(&provenance.role_id)
        )));
    }
    time_range(path, &provenance.start, provenance.end.as_deref())?;
    non_empty(&format!("{path}.rationale"), &provenance.rationale)
}

fn time_range(path: &str, start: &str, end: Option<&str>) -> Result<(), ForgeError> {
    let start = timestamp(&format!("{path}.start"), start)?;
    if let Some(end) = end {
        let end = timestamp(&format!("{path}.end"), end)?;
        if end < start {
            return Err(error(format!("{path}.end must not precede {path}.start")));
        }
    }
    Ok(())
}

fn timestamp(path: &str, value: &str) -> Result<DateTime<FixedOffset>, ForgeError> {
    DateTime::parse_from_rfc3339(value)
        .map_err(|_| error(format!("{path} must be an RFC 3339 timestamp")))
}

fn relative_json_path(path: &str, value: &Path) -> Result<(), ForgeError> {
    if value.is_absolute()
        || value.components().any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(error(format!(
            "{path} must be a relative path without '.', '..', or leading separators"
        )));
    }
    if value.extension().and_then(|extension| extension.to_str()) != Some("json") {
        return Err(error(format!("{path} must name a local .json file")));
    }
    Ok(())
}

fn relative_href(path: &str, value: &str) -> Result<(), ForgeError> {
    non_empty(path, value)?;
    let bytes = value.as_bytes();
    let windows_absolute = bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'/' | b'\\');
    let uri_scheme = value.split('/').next().is_some_and(|segment| segment.contains(':'));
    if value.contains('\n')
        || value.contains('\r')
        || value.starts_with("file:")
        || value.contains('\\')
        || value.contains('?')
        || value.contains('#')
        || uri_scheme
        || windows_absolute
        || Path::new(value).is_absolute()
        || Path::new(value)
            .components()
            .any(|component| !matches!(component, Component::Normal(_) | Component::CurDir))
    {
        return Err(error(format!(
            "{path} must be a confined slash-separated local reference without '..', URI scheme, query, fragment, backslash, or leading separator"
        )));
    }
    Ok(())
}

fn unique_non_empty(
    path: &str,
    values: &[String],
    require_non_empty: bool,
    limit: usize,
) -> Result<(), ForgeError> {
    if require_non_empty && values.is_empty() {
        return Err(error(format!("{path} must not be empty")));
    }
    if values.len() > limit {
        return Err(error(format!("{path} exceeds the {limit} entry limit")));
    }
    let mut unique = BTreeSet::new();
    for (index, value) in values.iter().enumerate() {
        non_empty(&format!("{path}[{index}]"), value)?;
        if !unique.insert(value.as_str()) {
            return Err(error(format!("{path}[{index}] duplicates '{}'", bounded(value))));
        }
    }
    Ok(())
}

fn uuid(path: &str, value: &str) -> Result<(), ForgeError> {
    uuid::Uuid::parse_str(value).map(|_| ()).map_err(|_| error(format!("{path} must be a UUID")))
}

fn non_empty(path: &str, value: &str) -> Result<(), ForgeError> {
    if value.trim().is_empty() { Err(error(format!("{path} must not be empty"))) } else { Ok(()) }
}

fn bounded(value: &str) -> String {
    json_strict::bounded(value)
}

fn error(message: impl Into<String>) -> ForgeError {
    ForgeError::AssessmentResultsBuild(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn provenance() -> ProvenanceManifest {
        ProvenanceManifest {
            assessor_key: "assessor".to_string(),
            role_id: "assessor".to_string(),
            start: "2026-01-01T00:00:00Z".to_string(),
            end: None,
            method: AssessmentMethod::Examine,
            rationale: "Reviewer rationale".to_string(),
        }
    }

    fn graph_result() -> ResultManifest {
        ResultManifest {
            key: "result".to_string(),
            title: "Result".to_string(),
            description: "Result".to_string(),
            start: "2026-01-01T00:00:00Z".to_string(),
            end: None,
            control_ids: vec!["AC-1".to_string()],
            objective_ids: Vec::new(),
            observations: Vec::new(),
            findings: vec![FindingManifest {
                key: "finding".to_string(),
                title: "Finding".to_string(),
                description: "Finding".to_string(),
                provenance: provenance(),
                target: FindingTargetManifest {
                    target_type: FindingTargetType::StatementId,
                    id: "AC-1_smt".to_string(),
                    state: FindingState::Satisfied,
                    reason: Some(FindingReason::Pass),
                },
                implementation_statement_uuid: None,
            }],
            risks: vec![RiskManifest {
                key: "risk".to_string(),
                title: "Risk".to_string(),
                description: "Risk".to_string(),
                statement: "Risk".to_string(),
                status: RiskStatus::Open,
                severity: None,
                confidence: None,
                provenance: provenance(),
            }],
            relationships: vec![
                RelationshipManifest {
                    from: RelationshipEndpoint {
                        object_type: ConclusionType::Observation,
                        key: "observation".to_string(),
                    },
                    to: RelationshipEndpoint {
                        object_type: ConclusionType::Finding,
                        key: "finding".to_string(),
                    },
                },
                RelationshipManifest {
                    from: RelationshipEndpoint {
                        object_type: ConclusionType::Finding,
                        key: "finding".to_string(),
                    },
                    to: RelationshipEndpoint {
                        object_type: ConclusionType::Risk,
                        key: "risk".to_string(),
                    },
                },
            ],
        }
    }

    fn graph_keys() -> BTreeMap<&'static str, ConclusionType> {
        BTreeMap::from([
            ("observation", ConclusionType::Observation),
            ("finding", ConclusionType::Finding),
            ("risk", ConclusionType::Risk),
        ])
    }

    #[test]
    fn duplicate_keys_are_rejected_before_typed_deserialization() {
        let error = parse(
            br#"{"schema_version":"forge.assessment-results/1","schema_version":"forge.assessment-results/1"}"#,
        )
        .expect_err("duplicate key must fail");
        assert!(error.to_string().contains("duplicate object key"));
    }

    #[test]
    fn paths_must_be_confined_relative_json_names() {
        for invalid in ["../plan.json", "./plan.json", "/tmp/plan.json", "plan.yaml"] {
            assert!(relative_json_path("$.artifact", Path::new(invalid)).is_err(), "{invalid}");
        }
        assert!(relative_json_path("$.artifact", Path::new("context/plan.json")).is_ok());
    }

    #[test]
    fn graph_rejects_duplicate_missing_wrong_type_and_circular_edges() {
        let keys = graph_keys();
        assert!(validate_relationships(&graph_result(), &keys).is_ok());

        let mut duplicate = graph_result();
        duplicate.relationships.push(duplicate.relationships[0].clone());
        assert!(
            validate_relationships(&duplicate, &keys)
                .unwrap_err()
                .to_string()
                .contains("duplicates")
        );

        let mut missing = graph_result();
        missing.relationships[0].from.key = "absent".to_string();
        assert!(
            validate_relationships(&missing, &keys).unwrap_err().to_string().contains("missing")
        );

        let mut wrong_type = graph_result();
        wrong_type.relationships[0].from.object_type = ConclusionType::Risk;
        assert!(
            validate_relationships(&wrong_type, &keys)
                .unwrap_err()
                .to_string()
                .contains("belongs to")
        );

        let mut circular = graph_result();
        circular.relationships[0] = RelationshipManifest {
            from: RelationshipEndpoint {
                object_type: ConclusionType::Finding,
                key: "finding".to_string(),
            },
            to: RelationshipEndpoint {
                object_type: ConclusionType::Observation,
                key: "observation".to_string(),
            },
        };
        assert!(
            validate_relationships(&circular, &keys).unwrap_err().to_string().contains("circular")
        );

        let mut unlinked = graph_result();
        unlinked.relationships.remove(0);
        assert!(
            validate_relationships(&unlinked, &keys).unwrap_err().to_string().contains("requires")
        );
    }
}
