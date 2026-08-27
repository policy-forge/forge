//! OSCAL artifact diffing: compare two OSCAL JSON artifacts and produce
//! structured change reports with added/removed/modified control tracking.
//!
//! Supports Catalog and Component Definition model types.

/// CI-safe, full-artifact comparison with a versioned volatility contract.
pub mod canonical;
/// Diff engine: compute a `DiffReport` between two artifact snapshots.
pub mod engine;
/// Snapshot extraction: build `ControlSnapshot` lists from OSCAL JSON.
pub mod extractor;
/// Human-readable diff report formatting.
pub mod formatter;
/// Diff domain types: `DiffReport`, `ControlSnapshot`, `DiffEntry`, `FieldChange`.
pub mod types;

use std::io::Read;
use std::path::Path;

use crate::error::ForgeError;
use crate::types::OscalModelType;
use crate::validate::detect_model_type;

pub use canonical::{
    DRIFT_COMPARISON_CONTRACT_VERSION, DriftComparison, DriftStatus, compare_artifacts_for_drift,
};
pub use formatter::format_diff_report;
pub use types::{ArtifactType, DiffReport};

/// # Errors
///
/// Returns `ForgeError::DiffError` for file, JSON, or artifact-type errors.
pub fn diff_artifacts(old_path: &Path, new_path: &Path) -> Result<DiffReport, ForgeError> {
    let (old_type, old_controls) = load_snapshot(old_path)?;
    let (new_type, new_controls) = load_snapshot(new_path)?;

    if old_type != new_type {
        return Err(ForgeError::DiffError(format!(
            "Artifact type mismatch: old is {old_type}, new is {new_type}"
        )));
    }

    let total_old = old_controls.len();
    let total_new = new_controls.len();
    tracing::debug!(total_old, total_new, "Control counts extracted");

    let entries = engine::compare_controls(&old_controls, &new_controls);
    let summary = engine::build_summary(&entries, total_old, total_new);

    Ok(DiffReport {
        old_file: old_path.display().to_string(),
        new_file: new_path.display().to_string(),
        artifact_type: old_type,
        entries,
        summary,
    })
}

fn read_diff_file(path: &Path) -> Result<String, ForgeError> {
    let file = std::fs::File::open(path).map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => {
            ForgeError::DiffError(format!("File not found: '{}'", path.display()))
        }
        _ => ForgeError::DiffError(format!("Failed to read '{}': {error}", path.display())),
    })?;
    let metadata = file.metadata().map_err(|error| {
        ForgeError::DiffError(format!("Failed to inspect '{}': {error}", path.display()))
    })?;
    if !metadata.is_file() {
        return Err(ForgeError::DiffError(format!("'{}' is not a regular file", path.display())));
    }
    if metadata.len() > crate::io::MAX_FILE_SIZE {
        return Err(ForgeError::DiffError(format!(
            "'{}' exceeds the {} byte size limit",
            path.display(),
            crate::io::MAX_FILE_SIZE
        )));
    }

    let capacity = usize::try_from(metadata.len()).map_err(|_| {
        ForgeError::DiffError(format!("'{}' size cannot be represented in memory", path.display()))
    })?;
    let mut content = String::with_capacity(capacity);
    file.take(crate::io::MAX_FILE_SIZE + 1).read_to_string(&mut content).map_err(|error| {
        ForgeError::DiffError(format!("Failed to read '{}': {error}", path.display()))
    })?;
    if content.len() as u64 > crate::io::MAX_FILE_SIZE {
        return Err(ForgeError::DiffError(format!(
            "'{}' exceeds the {} byte size limit",
            path.display(),
            crate::io::MAX_FILE_SIZE
        )));
    }
    Ok(content)
}

fn load_snapshot(
    path: &Path,
) -> Result<(ArtifactType, std::collections::HashMap<String, types::ControlSnapshot>), ForgeError> {
    let text = read_diff_file(path)?;
    let json: serde_json::Value = serde_json::from_str(&text).map_err(|error| {
        ForgeError::DiffError(format!("Invalid JSON in '{}': {error}", path.display()))
    })?;
    let artifact_type = to_artifact_type(&json, path)?;
    let controls = extractor::extract_controls(&json, &artifact_type);
    drop(json);
    drop(text);
    Ok((artifact_type, controls))
}

fn to_artifact_type(json: &serde_json::Value, path: &Path) -> Result<ArtifactType, ForgeError> {
    match detect_model_type(json) {
        Ok(OscalModelType::Catalog) => Ok(ArtifactType::Catalog),
        Ok(OscalModelType::ComponentDefinition) => Ok(ArtifactType::ComponentDefinition),
        Ok(unsupported @ (OscalModelType::Profile | OscalModelType::Mapping)) => {
            Err(ForgeError::DiffError(format!(
                "'{}': {} artifacts are not supported by diff; expected Catalog or ComponentDefinition",
                path.display(),
                unsupported.as_str()
            )))
        }
        Err(error) => Err(ForgeError::DiffError(format!(
            "'{}': expected a single supported OSCAL root key ('catalog' or 'component-definition'): {error}",
            path.display()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_json_file(json: &serde_json::Value) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(serde_json::to_string(json).unwrap().as_bytes()).unwrap();
        f
    }

    fn make_catalog_json(controls: &[(&str, &str, &str)]) -> serde_json::Value {
        fixtures::catalog(controls)
    }

    fn make_component_def_json(reqs: &[(&str, &str, &str)]) -> serde_json::Value {
        fixtures::component_def(reqs)
    }

    pub(crate) mod fixtures {
        pub fn catalog(controls: &[(&str, &str, &str)]) -> serde_json::Value {
            let controls_json: Vec<_> = controls
                .iter()
                .map(|(id, title, prose)| {
                    serde_json::json!({
                        "id": id,
                        "title": title,
                        "parts": [{"name": "statement", "id": format!("{id}_smt"), "prose": prose}]
                    })
                })
                .collect();
            serde_json::json!({
                "catalog": {
                    "uuid": "test-uuid",
                    "metadata": {"title": "Test", "last-modified": "2026-01-01T00:00:00Z",
                                 "version": "1.0", "oscal-version": "1.2.0"},
                    "groups": [{"id": "test", "title": "Test", "controls": controls_json}]
                }
            })
        }

        pub fn component_def(reqs: &[(&str, &str, &str)]) -> serde_json::Value {
            let reqs_json: Vec<_> = reqs
                .iter()
                .map(|(cid, uuid, desc)| {
                    serde_json::json!({"uuid": uuid, "control-id": cid, "description": desc})
                })
                .collect();
            serde_json::json!({
                "component-definition": {
                    "uuid": "cd-uuid",
                    "metadata": {"title": "Test", "last-modified": "2026-01-01T00:00:00Z",
                                 "version": "1.0", "oscal-version": "1.2.0"},
                    "components": [{
                        "uuid": "comp-uuid", "type": "policy", "title": "Test",
                        "description": "Test",
                        "control-implementations": [{
                            "uuid": "ci-uuid", "source": "./baseline.json",
                            "description": "Test",
                            "implemented-requirements": reqs_json
                        }]
                    }]
                }
            })
        }
    }

    #[test]
    fn test_type_mismatch_error() {
        let catalog = make_catalog_json(&[("POL-AC-001", "T1", "P1")]);
        let comp_def = serde_json::json!({
            "component-definition": {
                "uuid": "cd-uuid",
                "metadata": {"title": "Test", "last-modified": "2026-01-01T00:00:00Z",
                             "version": "1.0", "oscal-version": "1.2.0"},
                "components": []
            }
        });
        let old_file = write_json_file(&catalog);
        let new_file = write_json_file(&comp_def);
        let result = diff_artifacts(old_file.path(), new_file.path());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ForgeError::DiffError(_)));
        let msg = err.to_string();
        assert!(msg.contains("mismatch") || msg.contains("Mismatch") || msg.contains("different"));
    }

    #[test]
    fn test_missing_file_error() {
        let result =
            diff_artifacts(Path::new("/nonexistent/old.json"), Path::new("/nonexistent/new.json"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ForgeError::DiffError(_)));
        let msg = err.to_string();
        assert!(msg.contains("not found") || msg.contains("No such file") || msg.contains("exist"));
    }

    #[test]
    fn test_invalid_json_returns_error() {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(b"not valid json{{{").unwrap();
        let valid = write_json_file(&make_catalog_json(&[("A", "B", "C")]));
        let result = diff_artifacts(f.path(), valid.path());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ForgeError::DiffError(_)));
    }

    #[test]
    fn test_non_oscal_json_returns_error() {
        let non_oscal = serde_json::json!({"foo": "bar"});
        let f1 = write_json_file(&non_oscal);
        let f2 = write_json_file(&non_oscal);
        let result = diff_artifacts(f1.path(), f2.path());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ForgeError::DiffError(_)));
        let msg = err.to_string();
        assert!(msg.contains("OSCAL") || msg.contains("recognized"));
    }

    #[test]
    fn test_diff_component_definition_end_to_end() {
        let old = make_component_def_json(&[
            ("POL-AC-001", "uuid-1", "Old implementation"),
            ("POL-AC-002", "uuid-2", "Stays same"),
        ]);
        let new = make_component_def_json(&[
            ("POL-AC-001", "uuid-1", "New implementation"),
            ("POL-AC-002", "uuid-2", "Stays same"),
            ("POL-AC-003", "uuid-3", "Brand new"),
        ]);
        let old_file = write_json_file(&old);
        let new_file = write_json_file(&new);
        let report = diff_artifacts(old_file.path(), new_file.path()).unwrap();
        assert_eq!(report.summary.added, 1);
        assert_eq!(report.summary.changed, 1);
        assert_eq!(report.summary.unchanged, 1);
        assert_eq!(report.summary.total_old, 2);
        assert_eq!(report.summary.total_new, 3);
        assert_eq!(report.artifact_type, ArtifactType::ComponentDefinition);
    }

    #[test]
    fn test_diff_produces_report() {
        let old =
            make_catalog_json(&[("POL-AC-001", "Access Control", "Users shall authenticate.")]);
        let new = make_catalog_json(&[
            ("POL-AC-001", "Access Control", "Users shall authenticate."),
            ("POL-AC-002", "Password Policy", "Passwords shall be complex."),
        ]);
        let old_file = write_json_file(&old);
        let new_file = write_json_file(&new);
        let report = diff_artifacts(old_file.path(), new_file.path()).unwrap();
        assert_eq!(report.summary.added, 1);
        assert_eq!(report.summary.total_old, 1);
        assert_eq!(report.summary.total_new, 2);
    }

    #[test]
    fn test_ambiguous_oscal_artifact_reports_detected_roots() {
        let ambiguous = serde_json::json!({
            "catalog": {},
            "component-definition": {}
        });
        let old_file = write_json_file(&ambiguous);
        let new_file = write_json_file(&ambiguous);

        let error = diff_artifacts(old_file.path(), new_file.path()).unwrap_err();
        let message = error.to_string();
        assert!(message.contains("catalog"));
        assert!(message.contains("component-definition"));
    }

    #[test]
    fn test_profile_rejected() {
        let error =
            to_artifact_type(&serde_json::json!({"profile": {}}), Path::new("profile.json"))
                .unwrap_err();
        assert!(error.to_string().contains("profile artifacts are not supported"));
    }

    #[test]
    fn test_mapping_rejected() {
        let error = to_artifact_type(
            &serde_json::json!({"mapping-collection": {}}),
            Path::new("mapping.json"),
        )
        .unwrap_err();
        assert!(error.to_string().contains("mapping-collection artifacts are not supported"));
    }

    #[test]
    fn test_oversize_diff_file_rejected() {
        let file = NamedTempFile::new().unwrap();
        file.as_file().set_len(crate::io::MAX_FILE_SIZE + 1).unwrap();

        let error = read_diff_file(file.path()).unwrap_err();
        assert!(error.to_string().contains("size limit"));
    }
}
