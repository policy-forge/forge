//! Divergence types for round-trip validation results.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// A single difference between FORGE output and round-tripped output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Divergence {
    /// RFC 6901 JSON Pointer path to the differing element.
    pub json_path: String,
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
    /// oscal-cli was not available, so no external conversion evidence exists.
    Unavailable,
}

impl std::fmt::Display for CompatibilityClassification {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::VerifiedConversion => "verified-conversion",
            Self::AdvisoryOlderModelBaseline => "advisory-older-model-baseline",
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
        Some("1.0.3") => (CompatibilityClassification::AdvisoryOlderModelBaseline, Some("1.1.2")),
        // Unknown tool versions are not evidence of a compatible model baseline.
        // Promote a version to VerifiedConversion only after its OSCAL baseline
        // has been documented and exercised against the compatibility matrix.
        Some(_) => (CompatibilityClassification::AdvisoryOlderModelBaseline, None),
    }
}

/// Aggregate result of a single round-trip validation run.
#[derive(Debug, Serialize)]
pub struct RoundTripResult {
    /// OSCAL artifact type: `"Catalog"` or `"ComponentDefinition"`.
    pub artifact_type: String,
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
    /// `true` if all divergences are `Acceptable` (zero `ForgeFix` or `OscalCliDiff`).
    pub passed: bool,
    /// All divergences found (including Acceptable). Empty on clean pass.
    pub divergences: Vec<Divergence>,
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
    fn unknown_cli_baseline_is_advisory() {
        assert_eq!(
            classify_oscal_cli_compatibility(Some("9.9.9")),
            (CompatibilityClassification::AdvisoryOlderModelBaseline, None)
        );
    }
}
