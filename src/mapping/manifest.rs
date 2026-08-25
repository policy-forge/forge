//! Strict, bounded mapping-manifest v1 parsing.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Number, Value};

use crate::ForgeError;

/// Mapping manifest schema identifier supported by this release.
pub const MANIFEST_SCHEMA_VERSION: &str = "forge.mapping-manifest/1";
/// Maximum manifest size (2 MiB).
pub const MAX_MANIFEST_BYTES: u64 = 2 * 1024 * 1024;
/// Maximum number of reviewer records.
pub const MAX_REVIEWERS: usize = 100;
/// Maximum number of maps in the single MVP mapping.
pub const MAX_MAPS: usize = 10_000;
/// Maximum number of subjects on either side of one map.
pub const MAX_SUBJECTS_PER_SIDE: usize = 100;
/// Maximum number of standard qualifiers on one map.
pub const MAX_QUALIFIERS_PER_MAP: usize = 100;
/// Maximum UTF-8 byte length of any manifest string.
pub const MAX_STRING_BYTES: usize = 64 * 1024;

/// Closed, versioned reviewer-authored manifest.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MappingManifest {
    pub schema_version: String,
    pub collection: CollectionManifest,
    pub reviewers: Vec<ReviewerManifest>,
    pub provenance: ProvenanceManifest,
    pub mapping: MappingManifestBody,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionManifest {
    pub key: String,
    pub title: String,
    pub version: String,
    pub last_modified: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewerManifest {
    pub key: String,
    #[serde(rename = "type")]
    pub party_type: ReviewerType,
    pub name: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ReviewerType {
    Person,
    Organization,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProvenanceManifest {
    pub method: MappingMethod,
    pub matching_rationale: MatchingRationale,
    pub status: MappingStatus,
    pub mapping_description: String,
    pub reviewer_keys: Vec<String>,
    pub reviewed_at: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MappingManifestBody {
    pub key: String,
    #[serde(default)]
    pub scope: ReviewScope,
    #[serde(default)]
    pub method: Option<MappingMethod>,
    #[serde(default)]
    pub matching_rationale: Option<MatchingRationale>,
    #[serde(default)]
    pub status: Option<MappingStatus>,
    #[serde(default)]
    pub mapping_description: Option<String>,
    #[serde(default)]
    pub confidence_score: Option<ConfidenceScoreManifest>,
    #[serde(default)]
    pub coverage: Option<CoverageManifest>,
    pub source: ResourceManifest,
    pub target: ResourceManifest,
    pub maps: Vec<MapManifest>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ReviewScope {
    ControlOnly,
    #[default]
    ControlPlusStatement,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceManifest {
    #[serde(rename = "type")]
    pub resource_type: ResourceType,
    pub artifact: PathBuf,
    pub href: String,
    #[serde(default)]
    pub resolved_catalog: Option<PathBuf>,
    #[serde(default)]
    pub resolved_catalog_attestation: Option<bool>,
    #[serde(default)]
    pub expected_sha256: Option<String>,
    #[serde(default)]
    pub inventory: Option<ResourceInventorySnapshot>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ResourceInventorySnapshot {
    pub root_uuid: String,
    pub document_version: String,
    pub oscal_version: String,
    pub control_ids: Vec<String>,
    pub statement_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ResourceType {
    Catalog,
    Profile,
}

impl ResourceType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Catalog => "catalog",
            Self::Profile => "profile",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MapManifest {
    pub key: String,
    #[serde(default)]
    pub matching_rationale: Option<MatchingRationale>,
    pub relationship: Relationship,
    pub sources: Vec<SubjectManifest>,
    pub targets: Vec<SubjectManifest>,
    pub reviewer_key: String,
    pub reviewed_at: String,
    pub rationale: String,
    #[serde(default)]
    pub confidence_score: Option<ConfidenceScoreManifest>,
    #[serde(default)]
    pub coverage: Option<CoverageManifest>,
    #[serde(default)]
    pub qualifiers: Vec<QualifierManifest>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct SubjectManifest {
    #[serde(rename = "type")]
    pub subject_type: SubjectType,
    pub id_ref: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum SubjectType {
    Control,
    Statement,
}

impl SubjectType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Control => "control",
            Self::Statement => "statement",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MappingMethod {
    Human,
    Automation,
    Hybrid,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MatchingRationale {
    Syntactic,
    Semantic,
    Functional,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MappingStatus {
    Complete,
    NotComplete,
    Draft,
    Deprecated,
    Superseded,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Relationship {
    EquivalentTo,
    EqualTo,
    SubsetOf,
    SupersetOf,
    IntersectsWith,
    NoRelationship,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConfidenceScoreManifest {
    #[serde(default)]
    pub category: Option<ConfidenceCategory>,
    #[serde(default)]
    pub percentage: Option<f64>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ConfidenceCategory {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageManifest {
    pub generation_method: CoverageGenerationMethod,
    pub target_coverage: f64,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CoverageGenerationMethod {
    Arbitrary,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct QualifierManifest {
    pub subject: QualifierSubject,
    pub predicate: QualifierPredicate,
    pub category: QualifierCategory,
    pub description: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum QualifierSubject {
    Source,
    Target,
    Both,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum QualifierPredicate {
    HasRequirement,
    HasIncompatibility,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum QualifierCategory {
    Restricted,
    Addressable,
    Blocked,
}

/// Parse a manifest while rejecting duplicate decoded object keys.
///
/// # Errors
///
/// Returns [`ForgeError::MappingBuild`] when JSON, the closed manifest contract, references,
/// vocabulary, timestamps, confidence, or documented bounds are invalid.
pub fn parse(bytes: &[u8]) -> Result<MappingManifest, ForgeError> {
    if bytes.len() as u64 > MAX_MANIFEST_BYTES {
        return Err(mapping_error(format!("manifest exceeds the {MAX_MANIFEST_BYTES} byte limit")));
    }
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let strict = StrictValue::deserialize(&mut deserializer)
        .map_err(|error| mapping_error(format!("invalid manifest JSON: {error}")))?;
    deserializer
        .end()
        .map_err(|error| mapping_error(format!("invalid trailing manifest data: {error}")))?;
    enforce_value_bounds(&strict.0, "$", 0)?;
    let manifest: MappingManifest = serde_json::from_value(strict.0)
        .map_err(|error| mapping_error(format!("invalid manifest contract: {error}")))?;
    validate_contract(&manifest)?;
    Ok(manifest)
}

fn validate_contract(manifest: &MappingManifest) -> Result<(), ForgeError> {
    if manifest.schema_version != MANIFEST_SCHEMA_VERSION {
        return Err(mapping_error(format!(
            "unsupported schema_version '{}'; expected {MANIFEST_SCHEMA_VERSION}",
            bounded(&manifest.schema_version)
        )));
    }
    non_empty("$.collection.key", &manifest.collection.key)?;
    non_empty("$.collection.title", &manifest.collection.title)?;
    non_empty("$.collection.version", &manifest.collection.version)?;
    validate_timestamp("$.collection.last_modified", &manifest.collection.last_modified)?;
    if manifest.reviewers.is_empty() || manifest.reviewers.len() > MAX_REVIEWERS {
        return Err(mapping_error(format!("$.reviewers must contain 1..={MAX_REVIEWERS} entries")));
    }
    let mut reviewers = BTreeMap::new();
    for (index, reviewer) in manifest.reviewers.iter().enumerate() {
        non_empty(&format!("$.reviewers[{index}].key"), &reviewer.key)?;
        non_empty(&format!("$.reviewers[{index}].name"), &reviewer.name)?;
        if reviewers.insert(reviewer.key.as_str(), index).is_some() {
            return Err(mapping_error(format!(
                "$.reviewers[{index}].key duplicates reviewer key '{}'",
                bounded(&reviewer.key)
            )));
        }
    }
    non_empty("$.provenance.mapping_description", &manifest.provenance.mapping_description)?;
    validate_timestamp("$.provenance.reviewed_at", &manifest.provenance.reviewed_at)?;
    if manifest.provenance.reviewer_keys.is_empty() {
        return Err(mapping_error("$.provenance.reviewer_keys must not be empty"));
    }
    for key in &manifest.provenance.reviewer_keys {
        if !reviewers.contains_key(key.as_str()) {
            return Err(mapping_error(format!(
                "$.provenance.reviewer_keys references unknown reviewer '{}'",
                bounded(key)
            )));
        }
    }
    non_empty("$.mapping.key", &manifest.mapping.key)?;
    if let Some(description) = &manifest.mapping.mapping_description {
        non_empty("$.mapping.mapping_description", description)?;
    }
    validate_confidence("$.mapping.confidence_score", manifest.mapping.confidence_score.as_ref())?;
    validate_coverage("$.mapping.coverage", manifest.mapping.coverage.as_ref())?;
    validate_resource("$.mapping.source", &manifest.mapping.source)?;
    validate_resource("$.mapping.target", &manifest.mapping.target)?;
    if manifest.mapping.maps.is_empty() || manifest.mapping.maps.len() > MAX_MAPS {
        return Err(mapping_error(format!("$.mapping.maps must contain 1..={MAX_MAPS} entries")));
    }
    let mut map_keys = BTreeMap::new();
    for (index, map) in manifest.mapping.maps.iter().enumerate() {
        let path = format!("$.mapping.maps[{index}]");
        non_empty(&format!("{path}.key"), &map.key)?;
        if map_keys.insert(map.key.as_str(), index).is_some() {
            return Err(mapping_error(format!(
                "{path}.key duplicates map key '{}'",
                bounded(&map.key)
            )));
        }
        validate_subjects(&format!("{path}.sources"), &map.sources)?;
        validate_subjects(&format!("{path}.targets"), &map.targets)?;
        if !reviewers.contains_key(map.reviewer_key.as_str()) {
            return Err(mapping_error(format!(
                "{path}.reviewer_key references unknown reviewer '{}'",
                bounded(&map.reviewer_key)
            )));
        }
        validate_timestamp(&format!("{path}.reviewed_at"), &map.reviewed_at)?;
        non_empty(&format!("{path}.rationale"), &map.rationale)?;
        validate_confidence(&format!("{path}.confidence_score"), map.confidence_score.as_ref())?;
        validate_coverage(&format!("{path}.coverage"), map.coverage.as_ref())?;
        if map.qualifiers.len() > MAX_QUALIFIERS_PER_MAP {
            return Err(mapping_error(format!(
                "{path}.qualifiers exceeds the {MAX_QUALIFIERS_PER_MAP} entry limit"
            )));
        }
        for (qualifier_index, qualifier) in map.qualifiers.iter().enumerate() {
            non_empty(
                &format!("{path}.qualifiers[{qualifier_index}].description"),
                &qualifier.description,
            )?;
        }
    }
    Ok(())
}

fn validate_confidence(
    path: &str,
    score: Option<&ConfidenceScoreManifest>,
) -> Result<(), ForgeError> {
    let Some(score) = score else { return Ok(()) };
    match (score.category, score.percentage) {
        (Some(_), None) => Ok(()),
        (None, Some(value)) if value.is_finite() && (0.0..=1.0).contains(&value) => Ok(()),
        (None, Some(_)) => Err(mapping_error(format!("{path}.percentage must be between 0 and 1"))),
        _ => {
            Err(mapping_error(format!("{path} must contain exactly one of category or percentage")))
        }
    }
}

fn validate_coverage(path: &str, coverage: Option<&CoverageManifest>) -> Result<(), ForgeError> {
    if let Some(coverage) = coverage
        && (!coverage.target_coverage.is_finite()
            || !(0.0..=1.0).contains(&coverage.target_coverage))
    {
        Err(mapping_error(format!("{path}.target_coverage must be between 0 and 1")))
    } else {
        Ok(())
    }
}

fn validate_resource(path: &str, resource: &ResourceManifest) -> Result<(), ForgeError> {
    if resource.artifact.extension().and_then(|value| value.to_str()) != Some("json") {
        return Err(mapping_error(format!("{path}.artifact must be a local .json file")));
    }
    non_empty(&format!("{path}.href"), &resource.href)?;
    if resource.resource_type == ResourceType::Profile {
        let Some(companion) = &resource.resolved_catalog else {
            return Err(mapping_error(format!(
                "{path}.resolved_catalog is required for a Profile; run 'forge resolve' explicitly"
            )));
        };
        if companion.extension().and_then(|value| value.to_str()) != Some("json") {
            return Err(mapping_error(format!("{path}.resolved_catalog must be a .json file")));
        }
        if resource.resolved_catalog_attestation != Some(true) {
            return Err(mapping_error(format!(
                "{path}.resolved_catalog_attestation must be true to record reviewer attestation that the companion represents this Profile"
            )));
        }
    } else if resource.resolved_catalog.is_some() {
        return Err(mapping_error(format!(
            "{path}.resolved_catalog is only valid for Profile resources"
        )));
    } else if resource.resolved_catalog_attestation.is_some() {
        return Err(mapping_error(format!(
            "{path}.resolved_catalog_attestation is only valid for Profile resources"
        )));
    }
    if let Some(hash) = &resource.expected_sha256
        && (hash.len() != 64
            || !hash.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()))
    {
        return Err(mapping_error(format!(
            "{path}.expected_sha256 must be 64 lowercase hexadecimal characters"
        )));
    }
    Ok(())
}

fn validate_subjects(path: &str, subjects: &[SubjectManifest]) -> Result<(), ForgeError> {
    if subjects.is_empty() || subjects.len() > MAX_SUBJECTS_PER_SIDE {
        return Err(mapping_error(format!(
            "{path} must contain 1..={MAX_SUBJECTS_PER_SIDE} subjects"
        )));
    }
    let mut unique = BTreeMap::new();
    for (index, subject) in subjects.iter().enumerate() {
        non_empty(&format!("{path}[{index}].id_ref"), &subject.id_ref)?;
        if unique.insert((subject.subject_type, subject.id_ref.as_str()), index).is_some() {
            return Err(mapping_error(format!(
                "{path}[{index}] duplicates {} '{}'",
                subject.subject_type.as_str(),
                bounded(&subject.id_ref)
            )));
        }
    }
    Ok(())
}

fn validate_timestamp(path: &str, value: &str) -> Result<(), ForgeError> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|_| ())
        .map_err(|_| mapping_error(format!("{path} must be an RFC 3339 timestamp")))
}

fn non_empty(path: &str, value: &str) -> Result<(), ForgeError> {
    if value.trim().is_empty() {
        Err(mapping_error(format!("{path} must not be empty")))
    } else {
        Ok(())
    }
}

fn enforce_value_bounds(value: &Value, path: &str, depth: usize) -> Result<(), ForgeError> {
    const MAX_DEPTH: usize = 64;
    if depth > MAX_DEPTH {
        return Err(mapping_error(format!("{path} exceeds maximum JSON depth {MAX_DEPTH}")));
    }
    match value {
        Value::String(text) if text.len() > MAX_STRING_BYTES => Err(mapping_error(format!(
            "{path} exceeds maximum string length {MAX_STRING_BYTES} bytes"
        ))),
        Value::Array(values) => {
            for (index, child) in values.iter().enumerate() {
                enforce_value_bounds(child, &format!("{path}[{index}]"), depth + 1)?;
            }
            Ok(())
        }
        Value::Object(values) => {
            for (key, child) in values {
                enforce_value_bounds(child, &format!("{path}.{key}"), depth + 1)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn bounded(value: &str) -> String {
    value.chars().take(120).flat_map(char::escape_default).collect()
}

fn mapping_error(message: impl Into<String>) -> ForgeError {
    ForgeError::MappingBuild(message.into())
}

struct StrictValue(Value);

impl<'de> Deserialize<'de> for StrictValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(StrictValueVisitor)
    }
}

struct StrictValueVisitor;

impl<'de> Visitor<'de> for StrictValueVisitor {
    type Value = StrictValue;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a JSON value without duplicate object keys")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Bool(value)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Number(Number::from(value))))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Number(Number::from(value))))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Number::from_f64(value)
            .map(Value::Number)
            .map(StrictValue)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(StrictValue(Value::String(value.to_string())))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::String(value)))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Null))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Null))
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element::<StrictValue>()? {
            values.push(value.0);
        }
        Ok(StrictValue(Value::Array(values)))
    }

    fn visit_map<A>(self, mut object: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Map::new();
        while let Some((key, value)) = object.next_entry::<String, StrictValue>()? {
            if values.insert(key.clone(), value.0).is_some() {
                return Err(de::Error::custom(format!("duplicate object key '{key}'")));
            }
        }
        Ok(StrictValue(Value::Object(values)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_decoded_key_is_rejected() {
        let error = parse(br#"{"schema_version":"a","schema_version":"b"}"#)
            .expect_err("duplicate key must fail");
        assert!(error.to_string().contains("duplicate object key 'schema_version'"));
    }

    #[test]
    fn oversized_and_invalid_utf8_manifests_are_rejected_without_panic() {
        let oversized = vec![b' '; usize::try_from(MAX_MANIFEST_BYTES).unwrap() + 1];
        assert!(
            parse(&oversized).expect_err("oversized input must fail").to_string().contains("limit")
        );
        assert!(
            parse(b"{\"schema_version\":\"\xff\"}")
                .expect_err("invalid UTF-8 must fail")
                .to_string()
                .contains("invalid manifest JSON")
        );
    }
}
