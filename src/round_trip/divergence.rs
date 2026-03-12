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

/// Aggregate result of a single round-trip validation run.
#[derive(Debug, Serialize)]
pub struct RoundTripResult {
    /// OSCAL artifact type: `"Catalog"` or `"ComponentDefinition"`.
    pub artifact_type: String,
    /// Path to the original FORGE-generated JSON artifact.
    pub source_path: PathBuf,
    /// `true` if all divergences are `Acceptable` (zero `ForgeFix` or `OscalCliDiff`).
    pub passed: bool,
    /// All divergences found (including Acceptable). Empty on clean pass.
    pub divergences: Vec<Divergence>,
}
