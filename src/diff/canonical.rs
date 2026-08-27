//! Versioned, content-safe comparison for generated OSCAL drift checks.
//!
//! Unlike the human-oriented [`super::diff_artifacts`] report, this comparator
//! checks the complete JSON value and never returns policy content. Contract v1
//! ignores only the fields FORGE currently generates nondeterministically:
//! the artifact root `uuid` and `metadata.last-modified`.

use std::io::Read;
use std::path::Path;

use serde_json::Value;

use super::ArtifactType;
use crate::error::ForgeError;
use crate::types::OscalModelType;

/// Version of the canonical comparison contract.
///
/// Increment this value whenever the set of ignored fields or comparison
/// semantics changes.
pub const DRIFT_COMPARISON_CONTRACT_VERSION: u8 = 1;

/// JSON field paths excluded by the v1 comparison contract.
const EXCLUDED_FIELDS: &[&[&str]] = &[&["uuid"], &["metadata", "last-modified"]];

/// Outcome of a canonical artifact comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriftStatus {
    /// The artifacts are equivalent under the active comparison contract.
    Clean,
    /// At least one non-volatile JSON value differs.
    Drift,
}

impl DriftStatus {
    /// Stable lowercase representation for machine-readable output.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Clean => "clean",
            Self::Drift => "drift",
        }
    }
}

/// Content-free result of a canonical comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriftComparison {
    /// Supported OSCAL model detected in both files.
    pub artifact_type: ArtifactType,
    /// Whether substantive drift was detected.
    pub status: DriftStatus,
}

impl DriftComparison {
    /// Return `true` when the artifacts differ under the active contract.
    #[must_use]
    pub const fn has_drift(&self) -> bool {
        matches!(self.status, DriftStatus::Drift)
    }
}

/// Compare complete Catalog or Component Definition JSON artifacts while
/// excluding only the two documented FORGE-generated volatile fields.
///
/// Object key order and insignificant JSON whitespace do not affect the
/// result. Array order and every field outside the v1 exclusion list remain
/// significant. The returned value contains no artifact content.
///
/// # Errors
///
/// Returns [`ForgeError::DiffError`] when either file cannot be read or parsed,
/// when its OSCAL model is unsupported, when the model types differ, or when
/// required root/metadata objects are missing.
pub fn compare_artifacts_for_drift(
    committed_path: &Path,
    generated_path: &Path,
) -> Result<DriftComparison, ForgeError> {
    let mut committed = parse_artifact(committed_path, ArtifactRole::Committed)?;
    let mut generated = parse_artifact(generated_path, ArtifactRole::Generated)?;

    let committed_type = detect_artifact_type(&committed, ArtifactRole::Committed)?;
    let generated_type = detect_artifact_type(&generated, ArtifactRole::Generated)?;
    if committed_type != generated_type {
        return Err(ForgeError::DiffError(format!(
            "Artifact type mismatch: committed is {committed_type}, generated is {generated_type}"
        )));
    }

    canonicalize(&mut committed, &committed_type, ArtifactRole::Committed)?;
    canonicalize(&mut generated, &generated_type, ArtifactRole::Generated)?;

    let status = if committed == generated { DriftStatus::Clean } else { DriftStatus::Drift };
    Ok(DriftComparison { artifact_type: committed_type, status })
}

#[derive(Clone, Copy)]
enum ArtifactRole {
    Committed,
    Generated,
}

impl ArtifactRole {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Committed => "committed",
            Self::Generated => "generated",
        }
    }
}

fn parse_artifact(path: &Path, role: ArtifactRole) -> Result<Value, ForgeError> {
    let role_name = role.as_str();
    let file = std::fs::File::open(path).map_err(|error| {
        ForgeError::DiffError(format!(
            "unable to inspect {role_name} artifact ({:?})",
            error.kind()
        ))
    })?;
    let metadata = file.metadata().map_err(|error| {
        ForgeError::DiffError(format!(
            "unable to inspect {role_name} artifact ({:?})",
            error.kind()
        ))
    })?;
    if !metadata.is_file() {
        return Err(ForgeError::DiffError(format!("{role_name} artifact is not a regular file")));
    }
    if metadata.len() > crate::io::MAX_FILE_SIZE {
        return Err(ForgeError::DiffError(format!(
            "{role_name} artifact exceeds the {} byte comparison limit",
            crate::io::MAX_FILE_SIZE
        )));
    }

    let capacity = usize::try_from(metadata.len()).map_err(|_| {
        ForgeError::DiffError(format!("{role_name} artifact size cannot be represented in memory"))
    })?;
    let mut bytes = Vec::with_capacity(capacity);
    file.take(crate::io::MAX_FILE_SIZE + 1).read_to_end(&mut bytes).map_err(|error| {
        ForgeError::DiffError(format!("unable to read {role_name} artifact ({:?})", error.kind()))
    })?;
    if bytes.len() as u64 > crate::io::MAX_FILE_SIZE {
        return Err(ForgeError::DiffError(format!(
            "{role_name} artifact exceeds the {} byte comparison limit",
            crate::io::MAX_FILE_SIZE
        )));
    }
    let text = String::from_utf8(bytes).map_err(|error| {
        ForgeError::DiffError(format!(
            "unable to read {role_name} artifact ({:?})",
            error.utf8_error()
        ))
    })?;
    serde_json::from_str(&text).map_err(|error| {
        ForgeError::DiffError(format!(
            "invalid JSON in {role_name} artifact at line {}, column {}",
            error.line(),
            error.column()
        ))
    })
}

fn detect_artifact_type(value: &Value, role: ArtifactRole) -> Result<ArtifactType, ForgeError> {
    match crate::validate::detect_model_type(value) {
        Ok(OscalModelType::Catalog) => Ok(ArtifactType::Catalog),
        Ok(OscalModelType::ComponentDefinition) => Ok(ArtifactType::ComponentDefinition),
        Ok(OscalModelType::Profile) => Err(ForgeError::DiffError(format!(
            "{} artifact uses unsupported Profile model; expected Catalog or Component Definition",
            role.as_str()
        ))),
        Ok(OscalModelType::Mapping) => Err(ForgeError::DiffError(format!(
            "{} artifact uses unsupported Control Mapping model; expected Catalog or Component Definition",
            role.as_str()
        ))),
        Err(_) => Err(ForgeError::DiffError(format!(
            "{} artifact is not a recognized Catalog or Component Definition",
            role.as_str()
        ))),
    }
}

fn canonicalize(
    artifact: &mut Value,
    artifact_type: &ArtifactType,
    role: ArtifactRole,
) -> Result<(), ForgeError> {
    let root_key = match artifact_type {
        ArtifactType::Catalog => "catalog",
        ArtifactType::ComponentDefinition => "component-definition",
    };

    let root = artifact.get_mut(root_key).and_then(Value::as_object_mut).ok_or_else(|| {
        ForgeError::DiffError(format!(
            "{} artifact must contain a '{root_key}' JSON object",
            role.as_str()
        ))
    })?;

    for excluded_path in EXCLUDED_FIELDS {
        let (field, parent_path) = excluded_path.split_last().ok_or_else(|| {
            ForgeError::DiffError("drift comparison exclusion path must not be empty".to_string())
        })?;
        let mut object = &mut *root;
        for parent in parent_path {
            object = object.get_mut(*parent).and_then(Value::as_object_mut).ok_or_else(|| {
                ForgeError::DiffError(format!(
                    "{} artifact must contain a '{root_key}.{}' JSON object",
                    role.as_str(),
                    parent_path.join(".")
                ))
            })?;
        }
        object.remove(*field);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::NamedTempFile;

    use super::*;

    fn write_json(value: &Value) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(serde_json::to_string(value).unwrap().as_bytes()).unwrap();
        file
    }

    fn catalog(uuid: &str, last_modified: &str) -> Value {
        serde_json::json!({
            "catalog": {
                "uuid": uuid,
                "metadata": {
                    "title": "Policy",
                    "last-modified": last_modified,
                    "version": "1.0",
                    "oscal-version": "1.2.0"
                },
                "groups": [{
                    "id": "access-control",
                    "title": "Access Control",
                    "controls": [{
                        "id": "POL-AC-001",
                        "uuid": "stable-control-uuid",
                        "title": "Authentication",
                        "parts": [{"name": "statement", "prose": "Users must authenticate."}]
                    }]
                }]
            }
        })
    }

    fn component_definition(uuid: &str, last_modified: &str) -> Value {
        serde_json::json!({
            "component-definition": {
                "uuid": uuid,
                "metadata": {
                    "title": "Policy component",
                    "last-modified": last_modified,
                    "version": "1.0",
                    "oscal-version": "1.2.0"
                },
                "components": [{
                    "uuid": "stable-component-uuid",
                    "type": "policy",
                    "title": "Policy",
                    "description": "Documentary component"
                }]
            }
        })
    }

    fn compare_values(committed: &Value, generated: &Value) -> DriftComparison {
        let committed_file = write_json(committed);
        let generated_file = write_json(generated);
        compare_artifacts_for_drift(committed_file.path(), generated_file.path()).unwrap()
    }

    #[test]
    fn ignores_only_generated_catalog_root_uuid_and_timestamp() {
        let committed = catalog("uuid-a", "2026-01-01T00:00:00Z");
        let generated = catalog("uuid-b", "2026-08-23T12:34:56Z");

        let result = compare_values(&committed, &generated);
        assert_eq!(result.status, DriftStatus::Clean);
        assert_eq!(result.artifact_type, ArtifactType::Catalog);
    }

    #[test]
    fn ignores_only_generated_component_root_uuid_and_timestamp() {
        let committed = component_definition("uuid-a", "2026-01-01T00:00:00Z");
        let generated = component_definition("uuid-b", "2026-08-23T12:34:56Z");

        let result = compare_values(&committed, &generated);
        assert_eq!(result.status, DriftStatus::Clean);
        assert_eq!(result.artifact_type, ArtifactType::ComponentDefinition);
    }

    #[test]
    fn detects_policy_content_change_without_returning_content() {
        let committed = catalog("uuid-a", "2026-01-01T00:00:00Z");
        let mut generated = catalog("uuid-b", "2026-08-23T12:34:56Z");
        generated["catalog"]["groups"][0]["controls"][0]["parts"][0]["prose"] =
            Value::String("Sensitive changed statement".to_string());

        let result = compare_values(&committed, &generated);
        assert_eq!(result.status, DriftStatus::Drift);
    }

    #[test]
    fn detects_nested_uuid_change() {
        let committed = component_definition("uuid-a", "2026-01-01T00:00:00Z");
        let mut generated = component_definition("uuid-b", "2026-08-23T12:34:56Z");
        generated["component-definition"]["components"][0]["uuid"] =
            Value::String("different-component-uuid".to_string());

        assert!(compare_values(&committed, &generated).has_drift());
    }

    #[test]
    fn detects_nonvolatile_metadata_change() {
        let committed = catalog("uuid-a", "2026-01-01T00:00:00Z");
        let mut generated = catalog("uuid-b", "2026-08-23T12:34:56Z");
        generated["catalog"]["metadata"]["version"] = Value::String("2.0".to_string());

        assert!(compare_values(&committed, &generated).has_drift());
    }

    #[test]
    fn repeated_volatile_changes_do_not_create_false_drift() {
        let committed = catalog("committed-uuid", "2026-01-01T00:00:00Z");
        for run in 0..100 {
            let generated = catalog(
                &format!("generated-uuid-{run}"),
                &format!("2026-08-23T12:{:02}:{:02}Z", run / 60, run % 60),
            );
            assert_eq!(compare_values(&committed, &generated).status, DriftStatus::Clean);
        }
    }

    #[test]
    fn rejects_mismatched_artifact_types() {
        let committed = write_json(&catalog("uuid-a", "2026-01-01T00:00:00Z"));
        let generated = write_json(&component_definition("uuid-b", "2026-01-01T00:00:00Z"));

        let error = compare_artifacts_for_drift(committed.path(), generated.path()).unwrap_err();
        assert!(error.to_string().contains("type mismatch"));
    }

    #[test]
    fn exclusion_table_matches_v1_contract() {
        assert_eq!(DRIFT_COMPARISON_CONTRACT_VERSION, 1);
        assert_eq!(EXCLUDED_FIELDS, &[&["uuid"][..], &["metadata", "last-modified"][..]]);
    }
}
