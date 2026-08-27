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
/// * `ForgeError::InvalidArgument` — input is not a regular file.
/// * `ForgeError::OscalCliNotFound` — oscal-cli not on PATH.
/// * `ForgeError::OscalCliNotFunctional` — oscal-cli found but broken.
/// * `ForgeError::OscalCliExecution` — oscal-cli exited non-zero.
/// * `ForgeError::OscalCliTimeout` — oscal-cli exceeded timeout.
///
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
        execute_check(&detector)?;
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
    let canonical_input = input.canonicalize().map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => ForgeError::FileNotFound { path: input.to_path_buf() },
        std::io::ErrorKind::PermissionDenied => {
            ForgeError::PermissionDenied { path: input.to_path_buf() }
        }
        _ => ForgeError::Io(e),
    })?;
    if !std::fs::metadata(&canonical_input).map_err(ForgeError::Io)?.is_file() {
        return Err(ForgeError::InvalidArgument(format!(
            "input '{}' is not a regular file",
            canonical_input.display()
        )));
    }

    // Derive default output path if not provided
    let output_path = match output {
        Some(p) => p.to_path_buf(),
        None => derive_default_output_path(&canonical_input),
    };

    let canonical_output = canonicalize_output_path(&output_path)?;
    ensure_output_differs_from_input(&canonical_output, &canonical_input)?;

    let cli_info = detector.detect();
    if !cli_info.is_available() {
        return Err(ForgeError::OscalCliNotFound);
    }
    if !cli_info.is_functional() {
        return Err(ForgeError::OscalCliNotFunctional {
            path: cli_info
                .executable_path()
                .map_or_else(|| PathBuf::from("oscal-cli"), Path::to_path_buf),
            detail: cli_info.detail().unwrap_or("oscal-cli version check failed").to_string(),
        });
    }
    let executable_path =
        cli_info.executable_path().map(Path::to_path_buf).ok_or(ForgeError::OscalCliNotFound)?;
    info!(oscal_cli_path = %executable_path.display(), "Resolving with oscal-cli");
    let invoker = ProcessInvoker::new(executable_path);

    let resolve_args = ResolveArgs {
        profile_path: canonical_input,
        output_path: canonical_output,
        timeout: Duration::from_secs(timeout_secs),
    };

    let result = invoker.resolve_profile(&resolve_args)?;

    // Forward warnings
    for warning in &result.warnings {
        tracing::warn!(oscal_cli_warning = %warning, "oscal-cli stderr warning");
    }

    tracing::info!(output_path = %result.output_path.display(), "resolved catalog written");

    Ok(())
}

/// Execute --check: report oscal-cli detection status.
///
/// # Errors
///
/// * `ForgeError::OscalCliNotFound` — oscal-cli not on PATH (exit code 4).
/// * `ForgeError::OscalCliNotFunctional` — oscal-cli found but broken (exit code 4).
fn execute_check(detector: &dyn OscalCliDetect) -> Result<(), ForgeError> {
    let cli_info = detector.detect();
    if !cli_info.is_available() {
        return Err(ForgeError::OscalCliNotFound);
    }
    if !cli_info.is_functional() {
        return Err(ForgeError::OscalCliNotFunctional {
            path: cli_info
                .executable_path()
                .map_or_else(|| PathBuf::from("oscal-cli"), Path::to_path_buf),
            detail: cli_info.detail().unwrap_or("oscal-cli version check failed").to_string(),
        });
    }
    let executable_path = cli_info.executable_path().ok_or(ForgeError::OscalCliNotFound)?;
    let version = cli_info.version().ok_or_else(|| ForgeError::OscalCliNotFunctional {
        path: executable_path.to_path_buf(),
        detail: "oscal-cli version check produced no version".to_string(),
    })?;
    println!("oscal-cli: {version} ({})", executable_path.display());
    Ok(())
}

/// Canonicalize an output path, including a not-yet-created leaf below an
/// existing canonical parent, before it enters an oscal-cli argument struct.
fn canonicalize_output_path(output: &Path) -> Result<PathBuf, ForgeError> {
    match output.canonicalize() {
        Ok(path) => Ok(path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let parent = output
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .unwrap_or_else(|| Path::new("."));
            let parent = parent.canonicalize().map_err(ForgeError::Io)?;
            let filename = output.file_name().ok_or_else(|| {
                ForgeError::InvalidArgument("output path must name a file".to_string())
            })?;
            Ok(parent.join(filename))
        }
        Err(error) => Err(ForgeError::Io(error)),
    }
}

/// Derive the default output path: `<stem>-resolved.json` in the same directory.
fn derive_default_output_path(input: &Path) -> PathBuf {
    let stem = input
        .file_stem()
        .filter(|stem| !stem.is_empty())
        .map_or_else(|| "profile".to_string(), |stem| stem.to_string_lossy().into_owned());
    let filename = format!("{stem}-resolved.json");
    match input.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.join(filename),
        _ => PathBuf::from(filename),
    }
}

/// Reject an output that aliases the canonical input profile (F0348).
fn ensure_output_differs_from_input(
    output_path: &Path,
    canonical_input: &Path,
) -> Result<(), ForgeError> {
    if output_path == canonical_input
        || output_path.canonicalize().is_ok_and(|path| path == canonical_input)
    {
        return Err(ForgeError::InvalidArgument(format!(
            "output path '{}' must differ from the input profile '{}'",
            output_path.display(),
            canonical_input.display()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- T025: Default output path derivation ---

    #[test]
    fn canonicalizes_resolve_output_before_invocation() {
        let directory = tempfile::tempdir().unwrap();
        let output = directory.path().join("resolved.json");

        let canonical = canonicalize_output_path(&output).unwrap();

        assert_eq!(canonical, directory.path().canonicalize().unwrap().join("resolved.json"));
    }

    #[test]
    fn derive_default_output_path_standard() {
        let input = PathBuf::from("/tmp/my-profile.json");
        let output = derive_default_output_path(&input);
        assert_eq!(output, PathBuf::from("/tmp/my-profile-resolved.json"));
    }

    #[cfg(unix)]
    #[test]
    fn derive_default_output_path_preserves_non_utf8_stem_lossily() {
        use std::os::unix::ffi::OsStrExt;

        let input = PathBuf::from(std::ffi::OsStr::from_bytes(b"profile-\xff.json"));
        let output = derive_default_output_path(&input);

        assert_ne!(output, PathBuf::from("profile-resolved.json"));
        assert!(output.to_string_lossy().ends_with("-resolved.json"));
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

    #[test]
    fn input_directory_returns_invalid_argument() {
        let temp_dir = tempfile::tempdir().unwrap();
        let directory = temp_dir.path().join("profile.json");
        std::fs::create_dir(&directory).unwrap();

        let error = execute(Some(&directory), None, false, 60, None)
            .expect_err("directories must not be sent to oscal-cli");

        assert!(matches!(error, ForgeError::InvalidArgument(_)));
        assert!(error.to_string().contains(&directory.display().to_string()));
    }

    #[test]
    fn output_aliasing_input_is_rejected() {
        let input = tempfile::NamedTempFile::with_suffix(".json").unwrap();
        let canonical_input = input.path().canonicalize().unwrap();

        let error = ensure_output_differs_from_input(input.path(), &canonical_input)
            .expect_err("output must not overwrite the source profile");

        assert!(error.to_string().contains("must differ from the input profile"), "{error}");
    }

    // --- T051: --check with oscal-cli available (mock via PATH search) ---
    // These are tested by the execute_check function with constructed OscalCliInfo.

    #[test]
    fn check_mode_with_unavailable_oscal_cli_returns_not_found() {
        struct MockDetector;
        impl OscalCliDetect for MockDetector {
            fn detect(&self) -> crate::oscal_cli::OscalCliInfo {
                crate::oscal_cli::OscalCliInfo::not_found()
            }
        }
        let result = execute_check(&MockDetector);
        assert!(matches!(result, Err(ForgeError::OscalCliNotFound)));
    }

    #[test]
    fn check_mode_with_functional_oscal_cli_returns_ok() {
        struct MockDetector;
        impl OscalCliDetect for MockDetector {
            fn detect(&self) -> crate::oscal_cli::OscalCliInfo {
                crate::oscal_cli::OscalCliInfo::functional(
                    PathBuf::from("/usr/local/bin/oscal-cli"),
                    "1.0.3".to_string(),
                )
            }
        }
        let result = execute_check(&MockDetector);
        assert!(result.is_ok());
    }

    #[test]
    fn check_mode_with_nonfunctional_oscal_cli_returns_not_functional() {
        struct MockDetector;
        impl OscalCliDetect for MockDetector {
            fn detect(&self) -> crate::oscal_cli::OscalCliInfo {
                crate::oscal_cli::OscalCliInfo::not_functional(
                    PathBuf::from("/usr/bin/oscal-cli"),
                    "version check failed".to_string(),
                )
            }
        }
        let result = execute_check(&MockDetector);
        assert!(matches!(result, Err(ForgeError::OscalCliNotFunctional { .. })));
    }

    // --- T037: Graceful degradation ---

    #[test]
    fn resolve_with_missing_oscal_cli_returns_not_found() {
        // Use a tempfile as input so file validation passes
        let tmp = tempfile::NamedTempFile::with_suffix(".json").unwrap();
        // Use an oscal-cli-path that doesn't exist to force "not found"
        let result =
            execute(Some(tmp.path()), None, false, 60, Some(Path::new("/nonexistent/oscal-cli")));
        assert!(matches!(result, Err(ForgeError::OscalCliNotFound)), "{result:?}");
    }

    // --- T038: Other commands don't check oscal-cli ---
    // This is verified by the fact that `forge convert` and other commands
    // don't reference the oscal_cli module — tested via existing CLI tests.

    // --- T053: --check flag parsing ---
    // Tested in cli/mod.rs tests below (T053 in CLI arg parsing tests).

    // --- T052: --check with missing oscal-cli ---
    // Covered by check_mode_with_unavailable_oscal_cli test above.
}
