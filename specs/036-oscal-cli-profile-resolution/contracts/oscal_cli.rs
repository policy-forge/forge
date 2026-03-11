// Contract: oscal_cli module interfaces
// Feature: 036-oscal-cli-profile-resolution
// This file defines the trait and struct contracts for implementation.

use std::path::{Path, PathBuf};
use std::time::Duration;

// ── Data Structs ──────────────────────────────────────────────

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

// ── Traits ────────────────────────────────────────────────────

/// Detect whether oscal-cli is installed and functional.
///
/// Implementations:
/// - `PathDetector`: Production — searches PATH, runs `--version`.
/// - `MockDetector`: Test — returns preconfigured `OscalCliInfo`.
pub trait OscalCliDetect {
    fn detect(&self) -> OscalCliInfo;
}

/// Invoke oscal-cli operations.
///
/// Implementations:
/// - `ProcessInvoker`: Production — spawns subprocess via `std::process::Command`.
/// - `MockInvoker`: Test — returns preconfigured success/failure.
pub trait OscalCliInvoke {
    fn resolve_profile(&self, args: &ResolveArgs) -> Result<ResolveResult, ForgeError>;
}

// ── Error Variants (additions to existing ForgeError) ─────────

// These variants will be added to the existing ForgeError enum in src/error.rs:
//
// OscalCliNotFound
//   → "oscal-cli not found on system PATH. Install from: https://github.com/usnistgov/oscal-cli"
//   → Exit code: 4
//
// OscalCliNotFunctional { path: PathBuf, detail: String }
//   → "oscal-cli found at '{path}' but is not functional: {detail}"
//   → Exit code: 4
//
// OscalCliExecution { exit_code: Option<i32>, message: String, stderr: String }
//   → "oscal-cli execution failed (exit code {exit_code}): {message}"
//   → Exit code: 1
//
// OscalCliTimeout { timeout: Duration }
//   → "oscal-cli execution timed out after {timeout:?}"
//   → Exit code: 1
//
// ResolveInputNotJson { path: PathBuf }
//   → "Expected JSON input file, got: '{path}'. Only .json files are supported for profile resolution."
//   → Exit code: 1

// ── CLI Contract ──────────────────────────────────────────────

// forge resolve <profile-path> [OPTIONS]
//
// Arguments:
//   <profile-path>       Path to the OSCAL Profile JSON file
//
// Options:
//   --output <path>      Output file path (default: <input-stem>-resolved.json)
//   --check              Report oscal-cli detection status and exit
//   --timeout <seconds>  Timeout for oscal-cli execution (default: 60)
//   --oscal-cli-path <path>  Explicit path to oscal-cli binary (overrides PATH)

// ── oscal-cli Invocation Contract ─────────────────────────────

// Detection:
//   1. If --oscal-cli-path provided: use that path directly
//   2. Else: search PATH for "oscal-cli" (+ platform suffix)
//   3. Run: <oscal-cli-path> --version
//   4. Parse version from stdout
//   5. Return OscalCliInfo

// Resolution:
//   Command: <oscal-cli-path> profile resolve -to=json <input-path> <output-path>
//   Environment: env_clear() + PATH, HOME, JAVA_HOME, TMPDIR
//   Timeout: thread-based watchdog, default 60s
//   On success (exit 0): return ResolveResult with output_path
//   On failure (exit != 0): parse stderr, return ForgeError::OscalCliExecution
//   On timeout: kill child, return ForgeError::OscalCliTimeout
