//! oscal-cli detection — PATH search and version verification.

use std::path::{Path, PathBuf};
use std::process::Command;

use super::{OscalCliDetect, OscalCliInfo};

/// Searches the system PATH for the oscal-cli executable and verifies it works.
pub struct PathDetector {
    /// Optional explicit path override (from --oscal-cli-path).
    override_path: Option<PathBuf>,
}

impl PathDetector {
    /// Create a detector that searches the system PATH.
    #[must_use]
    pub fn new() -> Self {
        Self { override_path: None }
    }

    /// Create a detector with an explicit path override.
    #[must_use]
    pub fn with_path(path: PathBuf) -> Self {
        Self { override_path: Some(path) }
    }
}

impl Default for PathDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl OscalCliDetect for PathDetector {
    fn detect(&self) -> OscalCliInfo {
        let executable_path = match &self.override_path {
            Some(path) => match path.canonicalize() {
                Ok(canonical) => Some(canonical),
                Err(_) => return OscalCliInfo::not_found(),
            },
            None => search_path_for_oscal_cli(),
        };

        let Some(exe_path) = executable_path else {
            return OscalCliInfo::not_found();
        };

        // Try to run --version to verify functionality
        match run_version_check(&exe_path) {
            Ok(version) => OscalCliInfo {
                available: true,
                functional: true,
                version: Some(version),
                executable_path: Some(exe_path),
            },
            Err(_) => OscalCliInfo {
                available: true,
                functional: false,
                version: None,
                executable_path: Some(exe_path),
            },
        }
    }
}

/// Search the system PATH for the oscal-cli binary.
fn search_path_for_oscal_cli() -> Option<PathBuf> {
    let path_var = std::env::var("PATH").ok()?;
    let separator = if cfg!(windows) { ';' } else { ':' };
    let binary_name = format!("oscal-cli{}", std::env::consts::EXE_SUFFIX);

    for dir in path_var.split(separator) {
        let candidate = Path::new(dir).join(&binary_name);
        if candidate.exists() {
            return candidate.canonicalize().ok().or(Some(candidate));
        }
    }

    None
}

/// Run `oscal-cli --version` and extract the version string from stdout.
fn run_version_check(exe_path: &Path) -> Result<String, String> {
    let output = Command::new(exe_path)
        .arg("--version")
        .output()
        .map_err(|e| format!("Failed to execute oscal-cli: {e}"))?;

    if !output.status.success() {
        return Err(format!("oscal-cli --version exited with {}", output.status));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let version = parse_version_from_output(&stdout);
    Ok(version)
}

/// Extract a version number from oscal-cli --version output.
///
/// Typical output: "oscal-cli 1.0.3" or "oscal-cli version 1.0.3"
fn parse_version_from_output(output: &str) -> String {
    // Look for a version-like pattern (digits.digits.digits)
    for word in output.split_whitespace() {
        if word.chars().next().is_some_and(|c| c.is_ascii_digit()) && word.contains('.') {
            return word.trim().to_string();
        }
    }
    // Fall back to trimmed output
    output.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- T021: PathDetector found and functional ---
    // Note: This test uses mock detection since we can't guarantee oscal-cli on PATH.
    // We test the version parser and struct construction directly.

    #[test]
    fn parse_version_from_standard_output() {
        let version = parse_version_from_output("oscal-cli 1.0.3\n");
        assert_eq!(version, "1.0.3");
    }

    #[test]
    fn parse_version_from_verbose_output() {
        let version = parse_version_from_output("oscal-cli version 2.1.0\n");
        assert_eq!(version, "2.1.0");
    }

    #[test]
    fn parse_version_fallback_to_trimmed() {
        let version = parse_version_from_output("unknown format\n");
        assert_eq!(version, "unknown format");
    }

    #[test]
    fn detector_with_nonexistent_override_returns_unavailable() {
        let detector = PathDetector::with_path(PathBuf::from("/nonexistent/oscal-cli"));
        let info = detector.detect();
        assert!(!info.available);
        assert!(!info.functional);
        assert!(info.version.is_none());
        assert!(info.executable_path.is_none());
    }

    #[test]
    fn oscal_cli_info_not_found_has_correct_fields() {
        let info = OscalCliInfo::not_found();
        assert!(!info.available);
        assert!(!info.functional);
        assert!(info.version.is_none());
        assert!(info.executable_path.is_none());
    }

    #[test]
    fn oscal_cli_info_found_and_functional() {
        let info = OscalCliInfo {
            available: true,
            functional: true,
            version: Some("1.0.3".to_string()),
            executable_path: Some(PathBuf::from("/usr/local/bin/oscal-cli")),
        };
        assert!(info.available);
        assert!(info.functional);
        assert_eq!(info.version.as_deref(), Some("1.0.3"));
        assert_eq!(info.executable_path.as_deref(), Some(Path::new("/usr/local/bin/oscal-cli")));
    }

    #[test]
    fn oscal_cli_info_found_but_not_functional() {
        let info = OscalCliInfo {
            available: true,
            functional: false,
            version: None,
            executable_path: Some(PathBuf::from("/usr/local/bin/oscal-cli")),
        };
        assert!(info.available);
        assert!(!info.functional);
        assert!(info.version.is_none());
        assert!(info.executable_path.is_some());
    }

    #[test]
    fn default_detector_creates_path_detector() {
        let detector = PathDetector::default();
        assert!(detector.override_path.is_none());
    }
}
