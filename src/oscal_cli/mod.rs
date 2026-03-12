//! oscal-cli integration module — detection, invocation, and data types.
//!
//! Provides trait-based abstractions for detecting and invoking the NIST oscal-cli
//! tool as an external subprocess for OSCAL Profile resolution.

pub mod detector;
pub mod invoker;

use std::path::PathBuf;
use std::time::Duration;

use crate::error::ForgeError;

/// Detection result from oscal-cli PATH lookup and version check.
#[derive(Debug, Clone)]
pub struct OscalCliInfo {
    /// Whether oscal-cli was found on the system PATH (or via --oscal-cli-path).
    pub available: bool,
    /// Whether `oscal-cli --version` succeeded (binary is functional).
    pub functional: bool,
    /// The version string (e.g., "1.0.3"), if detected.
    pub version: Option<String>,
    /// The absolute path to the oscal-cli executable.
    pub executable_path: Option<PathBuf>,
}

impl OscalCliInfo {
    /// Construct the "not found" state.
    #[must_use]
    pub fn not_found() -> Self {
        Self { available: false, functional: false, version: None, executable_path: None }
    }
}

/// Arguments for a `profile resolve` invocation.
#[derive(Debug)]
pub struct ResolveArgs {
    /// Canonicalized absolute path to input Profile JSON.
    pub profile_path: PathBuf,
    /// Path where resolved Catalog will be written.
    pub output_path: PathBuf,
    /// Maximum execution time.
    pub timeout: Duration,
}

/// Successful invocation result.
#[derive(Debug)]
pub struct ResolveResult {
    /// Absolute path where the resolved Catalog was written.
    pub output_path: PathBuf,
    /// Any stderr warnings from oscal-cli (exit code 0 but stderr non-empty).
    pub warnings: Vec<String>,
}

/// Serialization format for an OSCAL document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OscalFormat {
    Json,
    Xml,
    Yaml,
}

impl OscalFormat {
    /// Returns the `--to=<fmt>` CLI flag value for oscal-cli.
    #[must_use]
    pub fn to_cli_flag(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Xml => "xml",
            Self::Yaml => "yaml",
        }
    }
}

/// Arguments for a single `oscal-cli convert` invocation.
#[derive(Debug)]
pub struct ConvertArgs {
    /// Canonicalized absolute path to the input OSCAL file.
    pub input_path: PathBuf,
    /// Path where the converted output will be written.
    pub output_path: PathBuf,
    /// Target serialization format.
    pub output_format: OscalFormat,
    /// Per-invocation timeout (default: 30 seconds).
    pub timeout: Duration,
}

/// Successful result of an `oscal-cli convert` invocation.
#[derive(Debug)]
pub struct ConvertResult {
    /// Absolute path to the written output file.
    pub output_path: PathBuf,
    /// Any stderr lines from oscal-cli when exit code was 0.
    pub warnings: Vec<String>,
}

/// Detect whether oscal-cli is installed and functional.
pub trait OscalCliDetect {
    fn detect(&self) -> OscalCliInfo;
}

/// Invoke oscal-cli operations.
pub trait OscalCliInvoke {
    /// Resolve an OSCAL Profile into a flat Catalog.
    ///
    /// # Errors
    ///
    /// Returns `ForgeError` if the oscal-cli process fails, times out, or produces invalid output.
    fn resolve_profile(&self, args: &ResolveArgs) -> Result<ResolveResult, ForgeError>;

    /// Convert an OSCAL document from one format to another via oscal-cli.
    ///
    /// # Errors
    ///
    /// - `ForgeError::OscalCliTimeout` if the subprocess exceeds `args.timeout`
    /// - `ForgeError::OscalCliExecution` if oscal-cli exits with non-zero status
    fn convert(&self, args: &ConvertArgs) -> Result<ConvertResult, ForgeError>;
}
