//! CLI handler for `forge resolve` — OSCAL Profile resolution via oscal-cli delegation.

use std::path::{Path, PathBuf};
use std::time::Duration;

use tracing::info;

use crate::error::ForgeError;
use crate::oscal_cli::detector::PathDetector;
use crate::oscal_cli::invoker::ProcessInvoker;
use crate::oscal_cli::{OscalCliDetect, OscalCliInvoke, ResolveArgs};

/// Execute the `forge resolve` subcommand.
///
/// # Errors
///
/// * `ForgeError::FileNotFound` — input file does not exist.
/// * `ForgeError::ResolveInputNotJson` — input file is not `.json`.
/// * `ForgeError::OscalCliNotFound` — oscal-cli not on PATH.
/// * `ForgeError::OscalCliNotFunctional` — oscal-cli found but broken.
/// * `ForgeError::OscalCliExecution` — oscal-cli exited non-zero.
/// * `ForgeError::OscalCliTimeout` — oscal-cli exceeded timeout.
///
/// # Panics
///
/// Panics if oscal-cli is detected as functional but has no executable path (invariant violation).
pub fn execute(
    input: Option<&Path>,
    output: Option<&Path>,
    check: bool,
    timeout_secs: u64,
    oscal_cli_path: Option<&Path>,
) -> Result<(), ForgeError> {
    // Build detector
    let detector = match oscal_cli_path {
        Some(path) => PathDetector::with_path(path.to_path_buf()),
        None => PathDetector::new(),
    };

    // --check mode: report oscal-cli status and exit
    if check {
        execute_check(&detector);
        return Ok(());
    }

    // Input is required when not in --check mode
    let input = input.ok_or_else(|| {
        ForgeError::InvalidArgument(
            "Input file is required (use --check to verify oscal-cli status without resolving)"
                .to_string(),
        )
    })?;

    // FR-007: Validate input file has .json extension
    match input.extension().and_then(|e| e.to_str()) {
        Some("json") => {}
        _ => {
            return Err(ForgeError::ResolveInputNotJson { path: input.to_path_buf() });
        }
    }

    // FR-007 + FR-014: Canonicalize input path (validates existence implicitly)
    let canonical_input =
        input.canonicalize().map_err(|_| ForgeError::FileNotFound { path: input.to_path_buf() })?;

    // Derive default output path if not provided
    let output_path = match output {
        Some(p) => p.to_path_buf(),
        None => derive_default_output_path(&canonical_input),
    };

    // Detect oscal-cli
    let cli_info = detector.detect();

    if !cli_info.available {
        return Err(ForgeError::OscalCliNotFound);
    }

    if !cli_info.functional {
        return Err(ForgeError::OscalCliNotFunctional {
            path: cli_info.executable_path.unwrap_or_default(),
            detail: "oscal-cli --version check failed (Java may be missing)".to_string(),
        });
    }

    // SEC-6: Log detected binary path
    info!(
        oscal_cli_path = %cli_info.executable_path.as_deref().unwrap_or(Path::new("unknown")).display(),
        oscal_cli_version = cli_info.version.as_deref().unwrap_or("unknown"),
        "Detected oscal-cli"
    );

    // Build invoker and resolve
    let invoker = ProcessInvoker::new(
        cli_info.executable_path.expect("functional oscal-cli must have a path"),
    );

    let resolve_args = ResolveArgs {
        profile_path: canonical_input,
        output_path,
        timeout: Duration::from_secs(timeout_secs),
    };

    let result = invoker.resolve_profile(&resolve_args)?;

    // Forward warnings
    for warning in &result.warnings {
        tracing::warn!(oscal_cli_warning = %warning, "oscal-cli stderr warning");
    }

    println!("Resolved catalog written to: {}", result.output_path.display());

    Ok(())
}

/// Execute --check: report oscal-cli detection status.
fn execute_check(detector: &dyn OscalCliDetect) {
    let info = detector.detect();

    if !info.available {
        println!("oscal-cli: not found");
        println!("Install from: https://github.com/usnistgov/oscal-cli");
        return;
    }

    if !info.functional {
        println!(
            "oscal-cli: found at {} but not functional",
            info.executable_path.as_deref().unwrap_or(Path::new("unknown")).display()
        );
        println!("Check that Java runtime is installed and on PATH");
        return;
    }

    println!(
        "oscal-cli: {} ({})",
        info.version.as_deref().unwrap_or("unknown version"),
        info.executable_path.as_deref().unwrap_or(Path::new("unknown")).display()
    );
}

/// Derive the default output path: `<stem>-resolved.json` in the same directory.
fn derive_default_output_path(input: &Path) -> PathBuf {
    let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("profile");
    let filename = format!("{stem}-resolved.json");
    match input.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.join(filename),
        _ => PathBuf::from(filename),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- T025: Default output path derivation ---

    #[test]
    fn derive_default_output_path_standard() {
        let input = PathBuf::from("/tmp/my-profile.json");
        let output = derive_default_output_path(&input);
        assert_eq!(output, PathBuf::from("/tmp/my-profile-resolved.json"));
    }

    #[test]
    fn derive_default_output_path_no_parent() {
        let input = PathBuf::from("profile.json");
        let output = derive_default_output_path(&input);
        assert_eq!(output, PathBuf::from("profile-resolved.json"));
    }

    #[test]
    fn derive_default_output_path_nested() {
        let input = PathBuf::from("/data/oscal/nist-800-53.json");
        let output = derive_default_output_path(&input);
        assert_eq!(output, PathBuf::from("/data/oscal/nist-800-53-resolved.json"));
    }

    // --- T026: Input file validation ---

    #[test]
    fn input_not_found_returns_error() {
        let result = execute(Some(Path::new("/nonexistent/profile.json")), None, false, 60, None);
        assert!(matches!(result, Err(ForgeError::FileNotFound { .. })));
    }

    #[test]
    fn input_not_json_returns_error() {
        // Create a temp file with .xml extension
        let tmp = tempfile::NamedTempFile::with_suffix(".xml").unwrap();
        let result = execute(Some(tmp.path()), None, false, 60, None);
        assert!(matches!(result, Err(ForgeError::ResolveInputNotJson { .. })));
    }

    // --- T051: --check with oscal-cli available (mock via PATH search) ---
    // These are tested by the execute_check function with constructed OscalCliInfo.

    #[test]
    fn check_mode_with_unavailable_oscal_cli() {
        struct MockDetector;
        impl OscalCliDetect for MockDetector {
            fn detect(&self) -> crate::oscal_cli::OscalCliInfo {
                crate::oscal_cli::OscalCliInfo::not_found()
            }
        }
        execute_check(&MockDetector);
    }

    #[test]
    fn check_mode_with_functional_oscal_cli() {
        struct MockDetector;
        impl OscalCliDetect for MockDetector {
            fn detect(&self) -> crate::oscal_cli::OscalCliInfo {
                crate::oscal_cli::OscalCliInfo {
                    available: true,
                    functional: true,
                    version: Some("1.0.3".to_string()),
                    executable_path: Some(PathBuf::from("/usr/local/bin/oscal-cli")),
                }
            }
        }
        execute_check(&MockDetector);
    }

    #[test]
    fn check_mode_with_nonfunctional_oscal_cli() {
        struct MockDetector;
        impl OscalCliDetect for MockDetector {
            fn detect(&self) -> crate::oscal_cli::OscalCliInfo {
                crate::oscal_cli::OscalCliInfo {
                    available: true,
                    functional: false,
                    version: None,
                    executable_path: Some(PathBuf::from("/usr/bin/oscal-cli")),
                }
            }
        }
        execute_check(&MockDetector);
    }

    // --- T037: Graceful degradation ---

    #[test]
    fn resolve_with_missing_oscal_cli_returns_not_found() {
        // Use a tempfile as input so file validation passes
        let tmp = tempfile::NamedTempFile::with_suffix(".json").unwrap();
        // Use an oscal-cli-path that doesn't exist to force "not found"
        let result =
            execute(Some(tmp.path()), None, false, 60, Some(Path::new("/nonexistent/oscal-cli")));
        assert!(matches!(result, Err(ForgeError::OscalCliNotFound)));
    }

    // --- T038: Other commands don't check oscal-cli ---
    // This is verified by the fact that `forge convert` and other commands
    // don't reference the oscal_cli module — tested via existing CLI tests.

    // --- T053: --check flag parsing ---
    // Tested in cli/mod.rs tests below (T053 in CLI arg parsing tests).

    // --- T052: --check with missing oscal-cli ---
    // Covered by check_mode_with_unavailable_oscal_cli test above.
}
