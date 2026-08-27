//! Divergence log writer — serializes `RoundTripResult` to JSON file.

use std::path::Path;

use crate::error::ForgeError;

use super::divergence::RoundTripResult;

/// Write a `RoundTripResult` as a pretty-printed JSON file.
///
/// Creates or overwrites the file at `output_path`. Parent directory must exist.
/// Serialization completes in memory before the destination is opened, so a
/// serialization failure cannot truncate an existing report.
///
/// # Errors
///
/// Returns `ForgeError::Io` on I/O failure, `ForgeError::Serialization` on serialization failure.
pub fn write_divergence_log(
    result: &RoundTripResult,
    output_path: &Path,
) -> Result<(), ForgeError> {
    let content = serde_json::to_vec_pretty(result)
        .map_err(|error| ForgeError::Serialization(error.to_string()))?;
    std::fs::write(output_path, content).map_err(ForgeError::Io)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::round_trip::divergence::{Divergence, DivergenceClass, ResolutionStatus};
    use std::path::PathBuf;

    // T016(a): Clean pass result → snapshot of written JSON
    #[test]
    fn clean_pass_result_snapshot() {
        let result = RoundTripResult {
            artifact_type: "Catalog".to_string(),
            source_path: PathBuf::from("/tmp/test/catalog.json"),
            declared_oscal_version: Some("1.2.3".to_string()),
            schema_version_used: "1.2.3".to_string(),
            oscal_cli_version: Some("1.0.3".to_string()),
            oscal_cli_model_version: Some("1.1.2".to_string()),
            compatibility_classification:
                crate::round_trip::CompatibilityClassification::AdvisoryOlderModelBaseline,
            divergences: vec![],
        };

        let temp_dir = tempfile::tempdir().unwrap();
        let output = temp_dir.path().join("divergences.json");
        write_divergence_log(&result, &output).unwrap();

        let content: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&output).unwrap()).unwrap();
        insta::assert_json_snapshot!("clean_pass", content);
    }

    // T016(b): Result with ForgeFix (Fixed) and Acceptable (Accepted) divergences → snapshot
    #[test]
    fn divergences_with_resolutions_snapshot() {
        let result = RoundTripResult {
            artifact_type: "Catalog".to_string(),
            source_path: PathBuf::from("/tmp/test/catalog.json"),
            declared_oscal_version: Some("1.2.3".to_string()),
            schema_version_used: "1.2.3".to_string(),
            oscal_cli_version: Some("1.0.3".to_string()),
            oscal_cli_model_version: Some("1.1.2".to_string()),
            compatibility_classification:
                crate::round_trip::CompatibilityClassification::AdvisoryOlderModelBaseline,
            divergences: vec![
                Divergence {
                    json_path: "/catalog/metadata/title".to_string(),
                    expected_index: None,
                    actual_index: None,
                    expected: serde_json::json!("My Security Policy"),
                    actual: serde_json::json!("my-security-policy"),
                    classification: DivergenceClass::ForgeFix,
                    description: "Title casing not preserved through XML serialization".to_string(),
                    resolution: Some(ResolutionStatus::Fixed),
                },
                Divergence {
                    json_path: "/catalog/groups/0/controls".to_string(),
                    expected_index: None,
                    actual_index: None,
                    expected: serde_json::json!([]),
                    actual: serde_json::Value::Null,
                    classification: DivergenceClass::Acceptable,
                    description: "Empty array omitted by oscal-cli".to_string(),
                    resolution: Some(ResolutionStatus::Accepted),
                },
            ],
        };

        let temp_dir = tempfile::tempdir().unwrap();
        let output = temp_dir.path().join("divergences.json");
        write_divergence_log(&result, &output).unwrap();

        let content: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&output).unwrap()).unwrap();
        insta::assert_json_snapshot!("divergences_with_resolutions", content);
    }

    // T016(c): Result with resolution: None → snapshot shows "resolution": null
    #[test]
    fn unresolved_divergence_shows_null_resolution_snapshot() {
        let result = RoundTripResult {
            artifact_type: "ComponentDefinition".to_string(),
            source_path: PathBuf::from("/tmp/test/component.json"),
            declared_oscal_version: Some("1.2.3".to_string()),
            schema_version_used: "1.2.3".to_string(),
            oscal_cli_version: None,
            oscal_cli_model_version: None,
            compatibility_classification:
                crate::round_trip::CompatibilityClassification::Unavailable,
            divergences: vec![Divergence {
                json_path: "/component-definition/metadata/version".to_string(),
                expected_index: None,
                actual_index: None,
                expected: serde_json::json!("1.0"),
                actual: serde_json::json!("1.0.0"),
                classification: DivergenceClass::OscalCliDiff,
                description: "Version string modified by oscal-cli".to_string(),
                resolution: None,
            }],
        };

        let temp_dir = tempfile::tempdir().unwrap();
        let output = temp_dir.path().join("divergences.json");
        write_divergence_log(&result, &output).unwrap();

        let content: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&output).unwrap()).unwrap();
        insta::assert_json_snapshot!("unresolved_divergence_null_resolution", content);
    }
}
