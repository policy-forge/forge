//! Strict, bounded applicability-manifest v1 parsing.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::PathBuf;

use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Number, Value};

use crate::ForgeError;
use crate::mapping::manifest::{ResourceManifest, ReviewerManifest};

/// Applicability manifest schema identifier supported by this release.
pub const MANIFEST_SCHEMA_VERSION: &str = "forge.applicability/1";
/// Maximum manifest size (64 MiB).
pub const MAX_MANIFEST_BYTES: u64 = 64 * 1024 * 1024;
/// Maximum number of explicit control decisions.
pub const MAX_DECISIONS: usize = 100_000;
/// Maximum number of Mapping Collections consumed by one analysis.
pub const MAX_MAPPING_COLLECTIONS: usize = 100;
/// Maximum number of reviewer records.
pub const MAX_REVIEWERS: usize = 100;
/// Maximum UTF-8 byte length of a manifest string.
const MAX_STRING_BYTES: usize = 64 * 1024;

/// Closed, versioned manifest containing human applicability decisions.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicabilityManifest {
    pub schema_version: String,
    pub framework: ResourceManifest,
    pub reviewers: Vec<ReviewerManifest>,
    pub decisions: Vec<ControlDecision>,
    pub mapping_collections: Vec<PathBuf>,
}

/// One explicit human review state for a stable framework control ID.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ControlDecision {
    pub control_id: String,
    pub state: DecisionState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewer_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revisit_date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Human applicability states. Omitted controls have `UnderReview` semantics.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum DecisionState {
    Applicable,
    NotApplicable,
    Deferred,
    UnderReview,
}

/// Parse and validate a closed manifest while rejecting duplicate decoded keys.
///
/// # Errors
///
/// Returns [`ForgeError::ApplicabilityAnalysis`] for invalid JSON, unsupported schema versions,
/// invalid decision evidence, duplicate records, or exceeded bounds.
pub fn parse(bytes: &[u8]) -> Result<ApplicabilityManifest, ForgeError> {
    if bytes.len() as u64 > MAX_MANIFEST_BYTES {
        return Err(error(format!("manifest exceeds the {MAX_MANIFEST_BYTES} byte limit")));
    }
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let strict = StrictValue::deserialize(&mut deserializer)
        .map_err(|cause| error(format!("invalid manifest JSON: {cause}")))?;
    deserializer
        .end()
        .map_err(|cause| error(format!("invalid trailing manifest data: {cause}")))?;
    enforce_value_bounds(&strict.0, "$", 0)?;
    let manifest: ApplicabilityManifest = serde_json::from_value(strict.0)
        .map_err(|cause| error(format!("invalid manifest contract: {cause}")))?;
    validate_contract(&manifest)?;
    Ok(manifest)
}

fn validate_contract(manifest: &ApplicabilityManifest) -> Result<(), ForgeError> {
    if manifest.schema_version != MANIFEST_SCHEMA_VERSION {
        return Err(error(format!(
            "unsupported schema_version '{}'; expected {MANIFEST_SCHEMA_VERSION}",
            bounded(&manifest.schema_version)
        )));
    }
    if manifest.reviewers.len() > MAX_REVIEWERS {
        return Err(error(format!("$.reviewers exceeds the {MAX_REVIEWERS} entry limit")));
    }
    if manifest.decisions.len() > MAX_DECISIONS {
        return Err(error(format!("$.decisions exceeds the {MAX_DECISIONS} entry limit")));
    }
    if manifest.mapping_collections.len() > MAX_MAPPING_COLLECTIONS {
        return Err(error(format!(
            "$.mapping_collections exceeds the {MAX_MAPPING_COLLECTIONS} entry limit"
        )));
    }
    validate_framework(&manifest.framework)?;

    let mut reviewers = BTreeSet::new();
    for (index, reviewer) in manifest.reviewers.iter().enumerate() {
        non_empty(&format!("$.reviewers[{index}].key"), &reviewer.key)?;
        non_empty(&format!("$.reviewers[{index}].name"), &reviewer.name)?;
        if !reviewers.insert(reviewer.key.as_str()) {
            return Err(error(format!(
                "$.reviewers[{index}].key duplicates reviewer key '{}'",
                bounded(&reviewer.key)
            )));
        }
    }

    let mut decisions = BTreeMap::new();
    for (index, decision) in manifest.decisions.iter().enumerate() {
        let path = format!("$.decisions[{index}]");
        non_empty(&format!("{path}.control_id"), &decision.control_id)?;
        if decisions.insert(decision.control_id.as_str(), index).is_some() {
            return Err(error(format!(
                "{path}.control_id duplicates control decision '{}'",
                bounded(&decision.control_id)
            )));
        }
        validate_decision(&path, decision, &reviewers)?;
    }

    let mut mappings = BTreeSet::new();
    for (index, mapping) in manifest.mapping_collections.iter().enumerate() {
        if mapping.extension().and_then(|value| value.to_str()) != Some("json") {
            return Err(error(format!(
                "$.mapping_collections[{index}] must be a local .json file"
            )));
        }
        if !mappings.insert(mapping) {
            return Err(error(format!(
                "$.mapping_collections[{index}] duplicates Mapping Collection path '{}'",
                mapping.display()
            )));
        }
    }
    Ok(())
}

fn validate_framework(framework: &ResourceManifest) -> Result<(), ForgeError> {
    if framework.artifact.extension().and_then(|value| value.to_str()) != Some("json") {
        return Err(error("$.framework.artifact must be a local .json file"));
    }
    non_empty("$.framework.href", &framework.href)?;
    validate_report_href("$.framework.href", &framework.href)?;
    match framework.resource_type {
        crate::mapping::manifest::ResourceType::Profile => {
            let Some(companion) = &framework.resolved_catalog else {
                return Err(error(
                    "$.framework.resolved_catalog is required for a Profile; run 'forge resolve' explicitly",
                ));
            };
            if companion.extension().and_then(|value| value.to_str()) != Some("json") {
                return Err(error("$.framework.resolved_catalog must be a .json file"));
            }
            if framework.resolved_catalog_attestation != Some(true) {
                return Err(error(
                    "$.framework.resolved_catalog_attestation must be true to attest that the companion represents this Profile",
                ));
            }
        }
        crate::mapping::manifest::ResourceType::Catalog => {
            if framework.resolved_catalog.is_some()
                || framework.resolved_catalog_attestation.is_some()
            {
                return Err(error(
                    "$.framework resolved Catalog fields are only valid for a Profile",
                ));
            }
        }
    }
    if let Some(hash) = &framework.expected_sha256
        && (hash.len() != 64
            || !hash.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()))
    {
        return Err(error(
            "$.framework.expected_sha256 must be 64 lowercase hexadecimal characters",
        ));
    }
    if framework.expected_sha256.is_none() {
        return Err(error("$.framework.expected_sha256 is required"));
    }
    if framework.inventory.is_none() {
        return Err(error("$.framework.inventory is required"));
    }
    Ok(())
}

/// Reject local absolute path spellings from fields preserved in deterministic reports.
pub(crate) fn validate_report_href(path: &str, value: &str) -> Result<(), ForgeError> {
    let bytes = value.as_bytes();
    let windows_drive = bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':';
    let rooted_local_path = value.starts_with('/') || value.starts_with('\\');
    let parsed_url = url::Url::parse(value);
    let file_url = parsed_url.as_ref().is_ok_and(|url| url.scheme() == "file");
    let ntfs_alternate_stream =
        parsed_url.is_err() && value.split(['/', '\\']).any(|component| component.contains(':'));
    if PathBuf::from(value).is_absolute()
        || rooted_local_path
        || windows_drive
        || file_url
        || ntfs_alternate_stream
    {
        Err(error(format!(
            "{path} must not contain an absolute local path or NTFS alternate data stream; use a relative label or non-file URI"
        )))
    } else {
        Ok(())
    }
}

fn validate_decision(
    path: &str,
    decision: &ControlDecision,
    reviewers: &BTreeSet<&str>,
) -> Result<(), ForgeError> {
    if let Some(key) = decision.reviewer_key.as_deref()
        && !reviewers.contains(key)
    {
        return Err(error(format!(
            "{path}.reviewer_key references unknown reviewer '{}'",
            bounded(key)
        )));
    }
    if let Some(timestamp) = decision.reviewed_at.as_deref() {
        chrono::DateTime::parse_from_rfc3339(timestamp)
            .map_err(|_| error(format!("{path}.reviewed_at must be an RFC 3339 timestamp")))?;
    }
    for (field, value) in [("rationale", &decision.rationale), ("note", &decision.note)] {
        if let Some(value) = value {
            non_empty(&format!("{path}.{field}"), value)?;
        }
    }
    if let Some(date) = decision.revisit_date.as_deref() {
        chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
            .map_err(|_| error(format!("{path}.revisit_date must use YYYY-MM-DD")))?;
    }

    match decision.state {
        DecisionState::Applicable => {
            require_reviewer_and_time(path, decision)?;
            if decision.revisit_date.is_some() {
                return Err(error(format!(
                    "{path}.revisit_date is only valid for deferred decisions"
                )));
            }
        }
        DecisionState::NotApplicable => {
            require_reviewer_and_time(path, decision)?;
            require_rationale(path, decision)?;
            if decision.revisit_date.is_some() {
                return Err(error(format!(
                    "{path}.revisit_date is only valid for deferred decisions"
                )));
            }
        }
        DecisionState::Deferred => {
            require_reviewer_and_time(path, decision)?;
            require_rationale(path, decision)?;
            if decision.revisit_date.is_none() {
                return Err(error(format!(
                    "{path}.revisit_date is required for deferred decisions"
                )));
            }
        }
        DecisionState::UnderReview => {
            if decision.reviewed_at.is_some()
                || decision.rationale.is_some()
                || decision.revisit_date.is_some()
            {
                return Err(error(format!(
                    "{path} under-review may contain only optional reviewer_key and note evidence"
                )));
            }
        }
    }
    Ok(())
}

fn require_reviewer_and_time(path: &str, decision: &ControlDecision) -> Result<(), ForgeError> {
    if decision.reviewer_key.is_none() {
        return Err(error(format!("{path}.reviewer_key is required for this decision state")));
    }
    if decision.reviewed_at.is_none() {
        return Err(error(format!("{path}.reviewed_at is required for this decision state")));
    }
    Ok(())
}

fn require_rationale(path: &str, decision: &ControlDecision) -> Result<(), ForgeError> {
    if decision.rationale.is_none() {
        Err(error(format!("{path}.rationale is required for this decision state")))
    } else {
        Ok(())
    }
}

fn non_empty(path: &str, value: &str) -> Result<(), ForgeError> {
    if value.trim().is_empty() { Err(error(format!("{path} must not be empty"))) } else { Ok(()) }
}

fn enforce_value_bounds(value: &Value, path: &str, depth: usize) -> Result<(), ForgeError> {
    let mut path = path.to_string();
    enforce_value_bounds_inner(value, &mut path, depth)
}

fn enforce_value_bounds_inner(
    value: &Value,
    path: &mut String,
    depth: usize,
) -> Result<(), ForgeError> {
    const MAX_DEPTH: usize = 64;
    if depth > MAX_DEPTH {
        return Err(error(format!("{path} exceeds maximum JSON depth {MAX_DEPTH}")));
    }
    match value {
        Value::String(text) if text.len() > MAX_STRING_BYTES => {
            Err(error(format!("{path} exceeds maximum string length {MAX_STRING_BYTES} bytes")))
        }
        Value::Array(values) => {
            for (index, child) in values.iter().enumerate() {
                let parent_len = path.len();
                let _ = write!(path, "[{index}]");
                let result = enforce_value_bounds_inner(child, path, depth + 1);
                path.truncate(parent_len);
                result?;
            }
            Ok(())
        }
        Value::Object(values) => {
            for (key, child) in values {
                let parent_len = path.len();
                path.push('.');
                path.push_str(key);
                let result = enforce_value_bounds_inner(child, path, depth + 1);
                path.truncate(parent_len);
                result?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn bounded(value: &str) -> String {
    value.chars().take(120).flat_map(char::escape_default).collect()
}

fn error(message: impl Into<String>) -> ForgeError {
    ForgeError::ApplicabilityAnalysis(message.into())
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
    use serde_json::json;

    fn valid_manifest() -> Value {
        json!({
            "schema_version": MANIFEST_SCHEMA_VERSION,
            "framework": {
                "type": "catalog",
                "artifact": "framework.json",
                "href": "framework.json",
                "expected_sha256": "0".repeat(64),
                "inventory": {
                    "root_uuid": "11111111-1111-4111-8111-111111111111",
                    "document_version": "1.0.0",
                    "oscal_version": "1.2.3",
                    "control_ids": [],
                    "statement_ids": []
                }
            },
            "reviewers": [],
            "decisions": [],
            "mapping_collections": []
        })
    }

    #[test]
    fn duplicate_decoded_key_is_rejected() {
        let cause = parse(br#"{"schema_version":"a","schema_version":"b"}"#)
            .expect_err("duplicate key must fail");
        assert!(cause.to_string().contains("duplicate object key 'schema_version'"));
    }

    #[test]
    fn size_depth_utf8_and_collection_count_limits_are_enforced() {
        let oversized = vec![b' '; usize::try_from(MAX_MANIFEST_BYTES).expect("limit") + 1];
        assert!(parse(&oversized).expect_err("oversized").to_string().contains("byte limit"));
        assert!(
            parse(b"{\"schema_version\":\"\xff\"}")
                .expect_err("invalid UTF-8")
                .to_string()
                .contains("invalid manifest JSON")
        );

        let mut too_many = valid_manifest();
        too_many["mapping_collections"] = Value::Array(
            (0..=MAX_MAPPING_COLLECTIONS)
                .map(|index| Value::String(format!("mapping-{index}.json")))
                .collect(),
        );
        let bytes = serde_json::to_vec(&too_many).expect("serialize");
        assert!(parse(&bytes).expect_err("count limit").to_string().contains("entry limit"));

        let mut nested = Value::Null;
        for _ in 0..=65 {
            nested = json!([nested]);
        }
        let mut deep = valid_manifest();
        deep["unexpected"] = nested;
        let bytes = serde_json::to_vec(&deep).expect("serialize");
        assert!(parse(&bytes).expect_err("depth limit").to_string().contains("maximum JSON depth"));
    }

    #[test]
    fn unsupported_versions_and_absolute_path_spellings_are_rejected() {
        let mut unsupported = valid_manifest();
        unsupported["schema_version"] = json!("forge.applicability/2");
        let bytes = serde_json::to_vec(&unsupported).expect("serialize");
        assert!(parse(&bytes).expect_err("unsupported").to_string().contains("unsupported"));

        for href in [
            "/private/framework.json",
            r"\Windows\framework.json",
            r"C:\Users\example\framework.json",
            "C:framework.json",
            r"\\server\share\framework.json",
            "file:///private/framework.json",
            "reports/framework.json:private",
        ] {
            assert!(
                validate_report_href("$.href", href)
                    .expect_err("absolute path")
                    .to_string()
                    .contains("absolute local path"),
                "accepted {href}"
            );
        }
        for href in [
            "framework.json",
            "../framework.json",
            "https://example.invalid/framework",
            "urn:example:framework",
        ] {
            validate_report_href("$.href", href).expect("safe report href");
        }
    }
}
