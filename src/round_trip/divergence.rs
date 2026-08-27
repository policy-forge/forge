//! Divergence types for round-trip validation results.

use std::path::PathBuf;

use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};

/// A single difference between FORGE output and round-tripped output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Divergence {
    /// RFC 6901 JSON Pointer path to the differing element.
    pub json_path: String,
    /// Index of the enclosing expected unordered-array element, when applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_index: Option<usize>,
    /// Index of the enclosing actual unordered-array element, when applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_index: Option<usize>,
    /// Value from the original FORGE output.
    pub expected: serde_json::Value,
    /// Value from the round-tripped output.
    pub actual: serde_json::Value,
    /// Classification of the divergence type.
    pub classification: DivergenceClass,
    /// Human-readable description of the difference.
    pub description: String,
    /// Resolution status of this divergence.
    /// `None` until investigated; serializes as `"resolution": null`.
    pub resolution: Option<ResolutionStatus>,
}

/// Classification of a divergence between FORGE output and oscal-cli output.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DivergenceClass {
    /// FORGE output is non-conformant; fix needed in FORGE.
    ForgeFix,
    /// oscal-cli introduces a non-standard transformation; report upstream.
    OscalCliDiff,
    /// Acceptable variation (empty array vs. omitted, whitespace normalization).
    Acceptable,
}

/// Resolution status of a divergence.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResolutionStatus {
    /// FORGE output has been corrected; divergence no longer occurs.
    Fixed,
    /// Divergence is an acceptable variation; no fix required.
    Accepted,
    /// Divergence is caused by oscal-cli; reported upstream to NIST.
    ReportedUpstream,
}

/// Strength of the interoperability evidence supplied by oscal-cli.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CompatibilityClassification {
    /// The detected tool completed the conversion path and is documented to use
    /// a model baseline compatible with FORGE's pinned OSCAL version.
    VerifiedConversion,
    /// Conversion succeeded, but the tool is documented as using an older
    /// OSCAL model; this is advisory interoperability evidence only.
    AdvisoryOlderModelBaseline,
    /// Conversion was observed, but the detected oscal-cli version has no
    /// documented OSCAL model baseline in FORGE's compatibility matrix.
    UnverifiedBaseline,
    /// oscal-cli was not available, so no external conversion evidence exists.
    Unavailable,
}

impl std::fmt::Display for CompatibilityClassification {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::VerifiedConversion => "verified-conversion",
            Self::AdvisoryOlderModelBaseline => "advisory-older-model-baseline",
            Self::UnverifiedBaseline => "unverified-baseline",
            Self::Unavailable => "unavailable",
        })
    }
}

/// Classify oscal-cli evidence and return its documented model baseline when known.
#[must_use]
pub fn classify_oscal_cli_compatibility(
    oscal_cli_version: Option<&str>,
) -> (CompatibilityClassification, Option<&'static str>) {
    match oscal_cli_version {
        None => (CompatibilityClassification::Unavailable, None),
        // NIST oscal-cli v1.0.3 documents OSCAL v1.1.2 model support.
        Some(version) if version.split(['+', '-']).next() == Some("1.0.3") => {
            (CompatibilityClassification::AdvisoryOlderModelBaseline, Some("1.1.2"))
        }
        // Unknown tool versions are not evidence of a documented model baseline.
        Some(_) => (CompatibilityClassification::UnverifiedBaseline, None),
    }
}

/// OSCAL artifact type represented by a round-trip validation result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactType {
    /// OSCAL catalog artifact.
    Catalog,
    /// OSCAL component definition artifact.
    ComponentDefinition,
    /// Artifact detection did not identify a supported OSCAL model.
    Unknown,
}

impl std::fmt::Display for ArtifactType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Catalog => "Catalog",
            Self::ComponentDefinition => "ComponentDefinition",
            Self::Unknown => "Unknown",
        })
    }
}

/// Aggregate result of a single round-trip validation run.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RoundTripResult {
    /// OSCAL artifact type identified for the round-trip validation.
    pub artifact_type: ArtifactType,
    /// Path to the original FORGE-generated JSON artifact.
    pub source_path: PathBuf,
    /// OSCAL version declared by the source artifact.
    pub declared_oscal_version: Option<String>,
    /// Pinned FORGE schema baseline used for structural validation.
    pub schema_version_used: String,
    /// Detected oscal-cli version, or `None` when it is unavailable.
    pub oscal_cli_version: Option<String>,
    /// OSCAL model baseline documented for the detected oscal-cli, when known.
    pub oscal_cli_model_version: Option<String>,
    /// Classification of the external conversion evidence.
    pub compatibility_classification: CompatibilityClassification,
    /// All divergences found (including Acceptable). Empty on clean pass.
    pub divergences: Vec<Divergence>,
}

impl RoundTripResult {
    /// Returns `true` only when every divergence is acceptable.
    #[must_use]
    pub fn passed(&self) -> bool {
        self.divergences
            .iter()
            .all(|divergence| divergence.classification == DivergenceClass::Acceptable)
    }
}

impl Serialize for RoundTripResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("RoundTripResult", 9)?;
        state.serialize_field("artifact_type", &self.artifact_type)?;
        state.serialize_field("source_path", &self.source_path)?;
        state.serialize_field("declared_oscal_version", &self.declared_oscal_version)?;
        state.serialize_field("schema_version_used", &self.schema_version_used)?;
        state.serialize_field("oscal_cli_version", &self.oscal_cli_version)?;
        state.serialize_field("oscal_cli_model_version", &self.oscal_cli_model_version)?;
        state
            .serialize_field("compatibility_classification", &self.compatibility_classification)?;
        state.serialize_field("passed", &self.passed())?;
        state.serialize_field("divergences", &self.divergences)?;
        state.end()
    }
}

#[cfg(test)]
mod compatibility_tests {
    use super::*;

    #[test]
    fn known_older_cli_baseline_is_advisory() {
        assert_eq!(
            classify_oscal_cli_compatibility(Some("1.0.3")),
            (CompatibilityClassification::AdvisoryOlderModelBaseline, Some("1.1.2"))
        );
    }

    #[test]
    fn unavailable_cli_has_explicit_classification() {
        assert_eq!(
            classify_oscal_cli_compatibility(None),
            (CompatibilityClassification::Unavailable, None)
        );
    }

    #[test]
    fn unknown_cli_baseline_is_unverified() {
        assert_eq!(
            classify_oscal_cli_compatibility(Some("9.9.9")),
            (CompatibilityClassification::UnverifiedBaseline, None)
        );
    }

    #[test]
    fn pass_status_is_derived_from_divergence_classifications() {
        let mut result = RoundTripResult {
            artifact_type: ArtifactType::Catalog,
            source_path: PathBuf::from("catalog.json"),
            declared_oscal_version: Some("1.2.3".to_string()),
            schema_version_used: "1.2.3".to_string(),
            oscal_cli_version: None,
            oscal_cli_model_version: None,
            compatibility_classification: CompatibilityClassification::Unavailable,
            divergences: Vec::new(),
        };
        assert!(result.passed());
        assert_eq!(serde_json::to_value(&result).unwrap()["passed"], true);

        result.divergences.push(Divergence {
            json_path: "/catalog/metadata/title".to_string(),
            expected_index: None,
            actual_index: None,
            expected: serde_json::json!("expected"),
            actual: serde_json::json!("actual"),
            classification: DivergenceClass::ForgeFix,
            description: "title changed".to_string(),
            resolution: None,
        });
        assert!(!result.passed());
        assert_eq!(serde_json::to_value(&result).unwrap()["passed"], false);
    }

    #[test]
    fn version_suffixes_keep_the_documented_older_model_baseline() {
        for version in ["1.0.3+build.7", "1.0.3-rc.1"] {
            assert_eq!(
                classify_oscal_cli_compatibility(Some(version)),
                (CompatibilityClassification::AdvisoryOlderModelBaseline, Some("1.1.2"))
            );
        }
    }

    #[test]
    fn compatibility_display_matches_its_serialized_value() {
        for classification in [
            CompatibilityClassification::VerifiedConversion,
            CompatibilityClassification::AdvisoryOlderModelBaseline,
            CompatibilityClassification::UnverifiedBaseline,
            CompatibilityClassification::Unavailable,
        ] {
            assert_eq!(
                serde_json::to_value(classification).unwrap(),
                serde_json::Value::String(classification.to_string())
            );
        }
    }

    #[test]
    fn artifact_type_display_matches_its_serialized_value() {
        for artifact_type in
            [ArtifactType::Catalog, ArtifactType::ComponentDefinition, ArtifactType::Unknown]
        {
            assert_eq!(
                serde_json::to_value(artifact_type).unwrap(),
                serde_json::Value::String(artifact_type.to_string())
            );
        }
    }

    #[test]
    fn round_trip_result_can_be_reloaded_from_its_divergence_log_json() {
        let result = RoundTripResult {
            artifact_type: ArtifactType::Catalog,
            source_path: PathBuf::from("catalog.json"),
            declared_oscal_version: Some("1.2.3".to_string()),
            schema_version_used: "1.2.3".to_string(),
            oscal_cli_version: None,
            oscal_cli_model_version: None,
            compatibility_classification: CompatibilityClassification::Unavailable,
            divergences: Vec::new(),
        };

        let serialized = serde_json::to_string(&result).unwrap();
        let reloaded: RoundTripResult = serde_json::from_str(&serialized).unwrap();
        assert_eq!(reloaded, result);
    }
}
