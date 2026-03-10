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

/// Arguments for a resolve-profile invocation.
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
}
