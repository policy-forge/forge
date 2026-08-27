//! oscal-cli integration module — detection, invocation, and data types.
//!
//! Provides trait-based abstractions for detecting and invoking the NIST oscal-cli
//! tool as an external subprocess for OSCAL Profile resolution.

pub mod detector;
pub mod invoker;

use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::error::ForgeError;

/// Internal state that makes invalid detector outcomes unrepresentable.
#[derive(Debug, Clone)]
enum DetectionOutcome {
    Unavailable { detail: Option<String> },
    NonFunctional { executable_path: PathBuf, detail: String },
    Functional { executable_path: PathBuf, version: String },
}

/// Detection result from oscal-cli PATH lookup and version check.
///
/// The outcome is private and can only be constructed through the validated
/// constructors below, so callers cannot combine an unavailable state with a
/// functional executable or version.
#[derive(Debug, Clone)]
pub struct OscalCliInfo {
    outcome: DetectionOutcome,
}

impl OscalCliInfo {
    /// Construct the ordinary "not found" state.
    #[must_use]
    pub fn not_found() -> Self {
        Self { outcome: DetectionOutcome::Unavailable { detail: None } }
    }

    /// Construct an unavailable state with an actionable discovery failure.
    #[must_use]
    pub fn unavailable(detail: String) -> Self {
        Self { outcome: DetectionOutcome::Unavailable { detail: Some(detail) } }
    }

    /// Construct the found-but-nonfunctional state.
    #[must_use]
    pub fn not_functional(executable_path: PathBuf, detail: String) -> Self {
        Self { outcome: DetectionOutcome::NonFunctional { executable_path, detail } }
    }

    /// Construct the verified functional state.
    #[must_use]
    pub fn functional(executable_path: PathBuf, version: String) -> Self {
        Self { outcome: DetectionOutcome::Functional { executable_path, version } }
    }

    /// Whether an executable was found.
    #[must_use]
    pub const fn is_available(&self) -> bool {
        !matches!(&self.outcome, DetectionOutcome::Unavailable { .. })
    }

    /// Whether the discovered executable completed its version check.
    #[must_use]
    pub const fn is_functional(&self) -> bool {
        matches!(&self.outcome, DetectionOutcome::Functional { .. })
    }

    /// The executable path when discovery reached a candidate.
    #[must_use]
    pub fn executable_path(&self) -> Option<&Path> {
        match &self.outcome {
            DetectionOutcome::Unavailable { .. } => None,
            DetectionOutcome::NonFunctional { executable_path, .. }
            | DetectionOutcome::Functional { executable_path, .. } => Some(executable_path),
        }
    }

    /// The verified CLI version.
    #[must_use]
    pub fn version(&self) -> Option<&str> {
        match &self.outcome {
            DetectionOutcome::Functional { version, .. } => Some(version),
            DetectionOutcome::Unavailable { .. } | DetectionOutcome::NonFunctional { .. } => None,
        }
    }

    /// Actionable discovery or version-check failure detail.
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        match &self.outcome {
            DetectionOutcome::Unavailable { detail } => detail.as_deref(),
            DetectionOutcome::NonFunctional { detail, .. } => Some(detail),
            DetectionOutcome::Functional { .. } => None,
        }
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
///
/// Used when invoking `oscal-cli convert` with the `--to=` flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OscalFormat {
    /// JSON serialization format.
    Json,
    /// XML serialization format.
    Xml,
    /// YAML serialization format.
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
    /// Per-invocation timeout, selected explicitly by the caller.
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
    /// Run detection: check PATH, verify the binary, and determine version.
    fn detect(&self) -> OscalCliInfo;
}

/// Invoke oscal-cli operations.
///
/// Input and output paths must be canonical absolute paths. Implementations
/// must pass them only as positional arguments after `--`, never as options.
///
/// Implementations are synchronous and may block the calling thread while the
/// subprocess runs and stderr is collected. Async callers must dispatch these
/// operations to a blocking executor.
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
