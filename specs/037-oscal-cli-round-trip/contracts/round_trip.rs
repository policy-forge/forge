//! Interface contract for the `round_trip` module (WI-37).
//!
//! This file defines the complete public surface area that will be implemented
//! in `src/round_trip/`. It is a contract document — not compiled code.
//! All signatures must match the final implementation exactly.

use std::path::{Path, PathBuf};
use std::time::Duration;

// ─── Re-exported from src/oscal_cli/mod.rs (extensions) ─────────────────────

/// Serialization format for an OSCAL document.
pub enum OscalFormat {
    Json,
    Xml,
    Yaml,
}

impl OscalFormat {
    /// Returns the `--to=<fmt>` CLI flag value for oscal-cli.
    pub fn to_cli_flag(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Xml => "xml",
            Self::Yaml => "yaml",
        }
    }
}

/// Arguments for a single `oscal-cli convert` invocation.
pub struct ConvertArgs {
    /// Canonicalized absolute path to the input OSCAL file.
    pub input_path: PathBuf,
    /// Path where the converted output will be written.
    pub output_path: PathBuf,
    /// Target serialization format.
    pub output_format: OscalFormat,
    /// Per-invocation timeout. Default: 30 seconds (per clarification Q4).
    pub timeout: Duration,
}

/// Successful result of an `oscal-cli convert` invocation.
pub struct ConvertResult {
    /// Absolute path to the written output file.
    pub output_path: PathBuf,
    /// Any stderr lines from oscal-cli when exit code was 0.
    pub warnings: Vec<String>,
}

/// Extended `OscalCliInvoke` trait — adds `convert` alongside existing `resolve_profile`.
///
/// Implementors: `ProcessInvoker` in `src/oscal_cli/invoker.rs`.
/// For unit tests: implement with a `MockInvoker` that returns `ConvertResult` or
/// `Err(ForgeError::OscalCliExecution{..})` as needed.
pub trait OscalCliInvoke {
    fn resolve_profile(&self, args: &ResolveArgs) -> Result<ResolveResult, ForgeError>;

    /// Convert an OSCAL document from one format to another via oscal-cli.
    ///
    /// # Errors
    ///
    /// - `ForgeError::OscalCliTimeout` if the subprocess exceeds `args.timeout`
    /// - `ForgeError::OscalCliExecution` if oscal-cli exits with non-zero status
    fn convert(&self, args: &ConvertArgs) -> Result<ConvertResult, ForgeError>;
}

// ─── round_trip::divergence ──────────────────────────────────────────────────

/// A single difference between FORGE output and round-tripped output.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Divergence {
    /// RFC 6901 JSON Pointer path to the differing element.
    /// Example: `/catalog/metadata/title`
    pub json_path: String,
    /// Value from the original FORGE output.
    pub expected: serde_json::Value,
    /// Value from the round-tripped output.
    pub actual: serde_json::Value,
    /// Classification of the divergence type.
    pub classification: DivergenceClass,
    /// Human-readable description of the difference.
    pub description: String,
    /// Resolution status of this divergence (PRD M-6 / AC-6).
    /// `None` until the divergence has been investigated and actioned.
    ///
    /// Serde behavior: serializes as `"resolution": null` when `None` (explicit presence,
    /// not omitted). Do NOT add `#[serde(skip_serializing_if = "Option::is_none")]` —
    /// the null value in the log makes unresolved divergences visible to reviewers.
    pub resolution: Option<ResolutionStatus>,
}

/// Classification of a divergence between FORGE output and oscal-cli output.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum DivergenceClass {
    /// FORGE output is non-conformant; fix needed in FORGE.
    ForgeFix,
    /// oscal-cli introduces a non-standard transformation; report upstream.
    OscalCliDiff,
    /// Acceptable variation (empty array vs. omitted, whitespace normalization).
    Acceptable,
}

/// Resolution status of a divergence (PRD M-6, AC-6, spec US2 AS-2).
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum ResolutionStatus {
    /// FORGE output has been corrected; divergence no longer occurs.
    Fixed,
    /// Divergence is an acceptable variation; no fix required.
    Accepted,
    /// Divergence is caused by oscal-cli; reported upstream to NIST.
    ReportedUpstream,
}

/// Aggregate result of a single round-trip validation run.
#[derive(Debug, serde::Serialize)]
pub struct RoundTripResult {
    /// OSCAL artifact type: "Catalog" or "ComponentDefinition".
    pub artifact_type: String,
    /// Path to the original FORGE-generated JSON artifact.
    pub source_path: PathBuf,
    /// `true` if all divergences are `Acceptable` (zero ForgeFix or OscalCliDiff).
    pub passed: bool,
    /// All divergences found (including Acceptable). Empty on clean pass.
    pub divergences: Vec<Divergence>,
}

// ─── round_trip::rules ───────────────────────────────────────────────────────

/// OSCAL-specific rules for the semantic comparison algorithm.
pub struct OscalComparisonRules {
    /// JSON key names whose array values are compared without regard to element order.
    /// Default: `["props", "links", "parts"]` (clarification Q2).
    /// Uses `HashSet` for O(1) membership tests during recursive tree walk.
    pub unordered_array_paths: HashSet<String>,
    /// JSON Pointer prefixes to skip entirely (reserved for future use; empty by default).
    pub ignored_paths: Vec<String>,
}

impl Default for OscalComparisonRules {
    fn default() -> Self {
        Self {
            unordered_array_paths: ["props", "links", "parts"]
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            ignored_paths: vec![],
        }
    }
}

// ─── round_trip::comparator ──────────────────────────────────────────────────

/// Compare two OSCAL JSON trees semantically.
///
/// Applies OSCAL-aware comparison rules:
/// - JSON objects: keys compared as unordered sets; values compared recursively
/// - Arrays at `props`, `links`, `parts` paths: elements matched by identity key
///   (`uuid` field, then `name`+`ns` composite, then positional fallback)
/// - All other arrays: elements compared positionally
/// - Primitives: compared by type and value
///
/// Returns a `Vec<Divergence>` — empty if the documents are semantically equivalent.
///
/// # Arguments
///
/// * `expected` - Original FORGE-generated JSON value
/// * `actual` - Round-tripped JSON value
/// * `path` - Current JSON Pointer path prefix (pass `""` at the root)
/// * `rules` - OSCAL-specific comparison rules
pub fn compare_oscal_json(
    expected: &serde_json::Value,
    actual: &serde_json::Value,
    path: &str,
    rules: &OscalComparisonRules,
) -> Vec<Divergence>;

// ─── round_trip::chain ───────────────────────────────────────────────────────

/// Execute the full oscal-cli conversion chain: JSON → XML → YAML → JSON.
///
/// Uses the caller-provided `temp_dir` for intermediate files (XML, YAML).
/// The caller owns cleanup responsibility (typically via `tempfile::TempDir` RAII).
/// This design allows the integration test to inspect intermediate files if needed.
///
/// # Arguments
///
/// * `input_json_path` - Path to the original FORGE-generated Catalog or Component JSON
/// * `invoker` - `OscalCliInvoke` implementor (real `ProcessInvoker` or test mock)
/// * `temp_dir` - Directory for intermediate XML and YAML files
/// * `timeout` - Per-invocation timeout (applied independently to each of the 3 steps)
///
/// # Returns
///
/// Path to the final round-tripped JSON file (written to `temp_dir`).
///
/// # Errors
///
/// Returns `ForgeError` if any conversion step fails or times out.
pub fn run_round_trip_chain(
    input_json_path: &Path,
    invoker: &dyn OscalCliInvoke,
    temp_dir: &Path,
    timeout: Duration,
) -> Result<PathBuf, ForgeError>;

// ─── round_trip::log ─────────────────────────────────────────────────────────

/// Write a `RoundTripResult` as a pretty-printed JSON file.
///
/// Creates or overwrites the file at `output_path`. Parent directory must exist.
///
/// # Errors
///
/// Returns `ForgeError::Io` if the file cannot be created or written.
pub fn write_divergence_log(
    result: &RoundTripResult,
    output_path: &Path,
) -> Result<(), ForgeError>;

// ─── round_trip::mod (public re-exports) ─────────────────────────────────────

// The following are re-exported from `src/round_trip/mod.rs`:
// pub use divergence::{Divergence, DivergenceClass, RoundTripResult};
// pub use rules::OscalComparisonRules;
// pub use comparator::compare_oscal_json;
// pub use chain::run_round_trip_chain;
// pub use log::write_divergence_log;
