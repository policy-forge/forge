//! Strict, bounded framework-impact manifest parsing.

use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::ForgeError;
use crate::mapping::manifest::ResourceType;

pub const MANIFEST_SCHEMA_VERSION: &str = "forge.framework-impact/1";
pub const MAX_MANIFEST_BYTES: u64 = 2 * 1024 * 1024;
pub const MAX_MAPPING_COLLECTIONS: usize = 1_000;
const MAX_STRING_BYTES: usize = 64 * 1024;
const MAX_JSON_DEPTH: usize = 64;
const STRICT_JSON_LIMITS: crate::json_strict::Limits =
    crate::json_strict::Limits { max_depth: MAX_JSON_DEPTH, max_string_bytes: MAX_STRING_BYTES };

/// A closed portfolio manifest for one exact framework revision comparison.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ImpactManifest {
    pub schema_version: String,
    pub old: FrameworkResource,
    pub new: FrameworkResource,
    #[serde(default)]
    pub mapping_collections: Vec<MappingDependency>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applicability_manifest: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub successor_map: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prior_report: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disposition_file: Option<PathBuf>,
}

/// Exact evidence expected for one local Catalog or Profile companion pair.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FrameworkResource {
    #[serde(rename = "type")]
    pub resource_type: ResourceType,
    pub artifact: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_catalog: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_catalog_attestation: Option<bool>,
    pub expected_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_resolved_catalog_sha256: Option<String>,
    pub root_uuid: String,
    pub document_version: String,
    pub oscal_version: String,
}

/// One PRD 055 Mapping Collection and the side occupied by the old framework.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MappingDependency {
    pub artifact: PathBuf,
    pub framework_role: FrameworkRole,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum FrameworkRole {
    Source,
    Target,
}

/// Parse a duplicate-key-safe, bounded framework-impact manifest.
///
/// # Errors
///
/// Returns [`ForgeError::FrameworkImpact`] for invalid JSON, contract fields, or bounds.
pub fn parse(bytes: &[u8]) -> Result<ImpactManifest, ForgeError> {
    if bytes.len() as u64 > MAX_MANIFEST_BYTES {
        return Err(impact_error(format!("manifest exceeds the {MAX_MANIFEST_BYTES} byte limit")));
    }
    let value = crate::json_strict::parse_value(bytes, "manifest", STRICT_JSON_LIMITS)
        .map_err(impact_error)?;
    let manifest: ImpactManifest = serde_json::from_value(value)
        .map_err(|error| impact_error(format!("invalid manifest contract: {error}")))?;
    validate(&manifest)?;
    Ok(manifest)
}

fn validate(manifest: &ImpactManifest) -> Result<(), ForgeError> {
    if manifest.schema_version != MANIFEST_SCHEMA_VERSION {
        return Err(impact_error(format!(
            "unsupported schema_version '{}'; expected {MANIFEST_SCHEMA_VERSION}",
            crate::json_strict::bounded(&manifest.schema_version)
        )));
    }
    if manifest.old.resource_type != manifest.new.resource_type {
        return Err(impact_error("$.old.type and $.new.type must describe the same OSCAL model"));
    }
    validate_resource("$.old", &manifest.old)?;
    validate_resource("$.new", &manifest.new)?;
    if manifest.mapping_collections.len() > MAX_MAPPING_COLLECTIONS {
        return Err(impact_error(format!(
            "$.mapping_collections exceeds the {MAX_MAPPING_COLLECTIONS} entry limit"
        )));
    }
    let mut paths = BTreeSet::new();
    for (index, dependency) in manifest.mapping_collections.iter().enumerate() {
        validate_json_path(
            &format!("$.mapping_collections[{index}].artifact"),
            &dependency.artifact,
        )?;
        if !paths.insert(dependency.artifact.as_path()) {
            return Err(impact_error(format!(
                "$.mapping_collections[{index}].artifact duplicates another Mapping Collection"
            )));
        }
    }
    if let Some(path) = &manifest.applicability_manifest {
        validate_json_path("$.applicability_manifest", path)?;
    }
    if let Some(path) = &manifest.successor_map {
        validate_json_path("$.successor_map", path)?;
    }
    match (&manifest.prior_report, &manifest.disposition_file) {
        (Some(prior), Some(dispositions)) => {
            validate_json_path("$.prior_report", prior)?;
            validate_json_path("$.disposition_file", dispositions)?;
        }
        (None, None) => {}
        _ => {
            return Err(impact_error(
                "$.prior_report and $.disposition_file must either both be present or both be absent",
            ));
        }
    }
    Ok(())
}

fn validate_resource(path: &str, resource: &FrameworkResource) -> Result<(), ForgeError> {
    validate_json_path(&format!("{path}.artifact"), &resource.artifact)?;
    validate_sha256(&format!("{path}.expected_sha256"), &resource.expected_sha256)?;
    match resource.resource_type {
        ResourceType::Catalog => {
            if resource.resolved_catalog.is_some()
                || resource.resolved_catalog_attestation.is_some()
                || resource.expected_resolved_catalog_sha256.is_some()
            {
                return Err(impact_error(format!(
                    "{path} resolved Catalog fields are only valid for a Profile"
                )));
            }
        }
        ResourceType::Profile => {
            let companion = resource.resolved_catalog.as_ref().ok_or_else(|| {
                impact_error(format!("{path}.resolved_catalog is required for a Profile"))
            })?;
            validate_json_path(&format!("{path}.resolved_catalog"), companion)?;
            if resource.resolved_catalog_attestation != Some(true) {
                return Err(impact_error(format!(
                    "{path}.resolved_catalog_attestation must be true for a Profile"
                )));
            }
            let fingerprint =
                resource.expected_resolved_catalog_sha256.as_ref().ok_or_else(|| {
                    impact_error(format!(
                        "{path}.expected_resolved_catalog_sha256 is required for a Profile"
                    ))
                })?;
            validate_sha256(&format!("{path}.expected_resolved_catalog_sha256"), fingerprint)?;
        }
    }
    uuid::Uuid::parse_str(&resource.root_uuid)
        .map_err(|_| impact_error(format!("{path}.root_uuid must be a UUID")))?;
    for (name, value) in [
        ("document_version", resource.document_version.as_str()),
        ("oscal_version", resource.oscal_version.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(impact_error(format!("{path}.{name} must not be empty")));
        }
    }
    if resource.oscal_version != crate::oscal::OSCAL_VERSION {
        return Err(impact_error(format!(
            "{path}.oscal_version must be {} for this release",
            crate::oscal::OSCAL_VERSION
        )));
    }
    Ok(())
}

fn validate_json_path(path: &str, value: &Path) -> Result<(), ForgeError> {
    if value.as_os_str().is_empty()
        || value.extension().and_then(|extension| extension.to_str()) != Some("json")
    {
        return Err(impact_error(format!("{path} must be a local .json file")));
    }
    let spelling = value.to_string_lossy();
    let bytes = spelling.as_bytes();
    let windows_drive = bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':';
    let rooted_spelling = spelling.starts_with('/') || spelling.starts_with('\\');
    let parent_component = value.components().any(|component| component == Component::ParentDir)
        || spelling.split(['/', '\\']).any(|component| component == "..");
    if value.is_absolute() || windows_drive || rooted_spelling || parent_component {
        return Err(impact_error(format!(
            "{path} must be a relative local .json path without parent-directory components"
        )));
    }
    Ok(())
}

fn validate_sha256(path: &str, value: &str) -> Result<(), ForgeError> {
    crate::json_strict::validate_lowercase_sha256(path, value).map_err(impact_error)
}

fn impact_error(message: impl Into<String>) -> ForgeError {
    ForgeError::FrameworkImpact(message.into())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::validate_json_path;

    #[test]
    fn manifest_paths_are_relative_and_cannot_traverse_parent_directories() {
        for rejected in [
            "/tmp/catalog.json",
            "../catalog.json",
            "nested/../catalog.json",
            r"C:\catalog.json",
            r"\\server\share\catalog.json",
            r"nested\..\catalog.json",
        ] {
            let error = validate_json_path("$.artifact", Path::new(rejected)).unwrap_err();
            assert!(error.to_string().contains("relative local .json path"), "{rejected}: {error}");
        }
        assert!(validate_json_path("$.artifact", Path::new("catalog.json")).is_ok());
        assert!(validate_json_path("$.artifact", Path::new("nested/catalog.json")).is_ok());
    }
}
