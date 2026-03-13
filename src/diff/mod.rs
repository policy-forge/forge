pub mod engine;
pub mod extractor;
pub mod formatter;
pub mod types;

use std::path::Path;

use crate::error::ForgeError;
use crate::types::OscalModelType;
use crate::validate::detect_model_type;

pub use formatter::format_diff_report;
pub use types::{ArtifactType, DiffReport};

/// # Errors
///
/// Returns `ForgeError::DiffError` for file, JSON, or artifact-type errors.
pub fn diff_artifacts(old_path: &Path, new_path: &Path) -> Result<DiffReport, ForgeError> {
    // Read files directly (no TOCTOU exists() pre-check)
    let old_text = read_diff_file(old_path)?;
    let new_text = read_diff_file(new_path)?;

    // Parse JSON
    let old_json: serde_json::Value = serde_json::from_str(&old_text).map_err(|e| {
        ForgeError::DiffError(format!("Invalid JSON in '{}': {e}", old_path.display()))
    })?;
    let new_json: serde_json::Value = serde_json::from_str(&new_text).map_err(|e| {
        ForgeError::DiffError(format!("Invalid JSON in '{}': {e}", new_path.display()))
    })?;

    // Detect artifact types (reuses validate::detect_model_type)
    let old_type = to_artifact_type(&old_json, old_path)?;
    let new_type = to_artifact_type(&new_json, new_path)?;

    // Validate same type
    if old_type != new_type {
        return Err(ForgeError::DiffError(format!(
            "Artifact type mismatch: old is {old_type}, new is {new_type}"
        )));
    }

    // Extract controls
    let old_controls = extractor::extract_controls(&old_json, &old_type);
    let new_controls = extractor::extract_controls(&new_json, &new_type);

    let total_old = old_controls.len();
    let total_new = new_controls.len();

    tracing::debug!(total_old, total_new, "Control counts extracted");

    // Compare
    let entries = engine::compare_controls(&old_controls, &new_controls);

    // Build summary
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
    // Guard against oversized files; ignore I/O errors (e.g. NotFound) so
    // read_to_string produces the user-facing DiffError below.
    match crate::io::check_file_size(path, crate::io::MAX_FILE_SIZE) {
        Ok(_) | Err(ForgeError::Io(_)) => {}
        Err(e) => return Err(e),
    }
    std::fs::read_to_string(path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => {
            ForgeError::DiffError(format!("File not found: '{}'", path.display()))
        }
        _ => ForgeError::DiffError(format!("Failed to read '{}': {e}", path.display())),
    })
}

fn to_artifact_type(json: &serde_json::Value, path: &Path) -> Result<ArtifactType, ForgeError> {
    match detect_model_type(json) {
        Ok(OscalModelType::Catalog) => Ok(ArtifactType::Catalog),
        Ok(OscalModelType::ComponentDefinition) => Ok(ArtifactType::ComponentDefinition),
        Ok(OscalModelType::Profile) => Err(ForgeError::DiffError(format!(
            "'{}': Profile artifacts are not supported by diff; expected Catalog or ComponentDefinition",
            path.display()
        ))),
        Err(_) => Err(ForgeError::DiffError(format!(
            "'{}': not a recognized OSCAL artifact; expected 'catalog' or 'component-definition' root key",
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
}
