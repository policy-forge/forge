//! Versioned, content-safe comparison for generated OSCAL drift checks.
//!
//! Unlike the human-oriented [`super::diff_artifacts`] report, this comparator
//! checks the complete JSON value and never returns policy content. Contract v1
//! ignores only the fields FORGE currently generates nondeterministically:
//! the artifact root `uuid` and `metadata.last-modified`.

use std::path::Path;

use serde_json::Value;

use super::ArtifactType;
use crate::error::ForgeError;

/// Version of the canonical comparison contract.
///
/// Increment this value whenever the set of ignored fields or comparison
/// semantics changes.
pub const DRIFT_COMPARISON_CONTRACT_VERSION: u8 = 1;

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
    let mut committed = parse_artifact(committed_path)?;
    let mut generated = parse_artifact(generated_path)?;

    let committed_type = super::to_artifact_type(&committed, committed_path)?;
    let generated_type = super::to_artifact_type(&generated, generated_path)?;
    if committed_type != generated_type {
        return Err(ForgeError::DiffError(format!(
            "Artifact type mismatch: committed is {committed_type}, generated is {generated_type}"
        )));
    }

    canonicalize(&mut committed, &committed_type, committed_path)?;
    canonicalize(&mut generated, &generated_type, generated_path)?;

    let status = if committed == generated { DriftStatus::Clean } else { DriftStatus::Drift };
    Ok(DriftComparison { artifact_type: committed_type, status })
}

fn parse_artifact(path: &Path) -> Result<Value, ForgeError> {
    let text = super::read_diff_file(path)?;
    serde_json::from_str(&text)
        .map_err(|e| ForgeError::DiffError(format!("Invalid JSON in '{}': {e}", path.display())))
}

fn canonicalize(
    artifact: &mut Value,
    artifact_type: &ArtifactType,
    path: &Path,
) -> Result<(), ForgeError> {
    let root_key = match artifact_type {
        ArtifactType::Catalog => "catalog",
        ArtifactType::ComponentDefinition => "component-definition",
    };

    let root = artifact.get_mut(root_key).and_then(Value::as_object_mut).ok_or_else(|| {
        ForgeError::DiffError(format!(
            "'{}': expected '{root_key}' to be a JSON object",
            path.display()
        ))
    })?;

    // Contract v1 exclusions. Do not add fields here without incrementing
    // DRIFT_COMPARISON_CONTRACT_VERSION and documenting the security impact.
    root.remove("uuid");
    let metadata = root.get_mut("metadata").and_then(Value::as_object_mut).ok_or_else(|| {
        ForgeError::DiffError(format!(
            "'{}': expected '{root_key}.metadata' to be a JSON object",
            path.display()
        ))
    })?;
    metadata.remove("last-modified");

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
}
