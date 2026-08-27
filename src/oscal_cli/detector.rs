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
///
/// On Windows, respects the `PATHEXT` environment variable so that `.bat`, `.cmd`,
/// and other wrapper scripts are discovered in addition to `.exe`.
/// On Unix, searches for the bare `oscal-cli` name (no extension).
fn search_path_for_oscal_cli() -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    let pathext = if cfg!(windows) {
        let raw = std::env::var("PATHEXT").unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_string());
        parse_pathext(&raw)
    } else {
        vec![]
    };
    search_path_with(&path_var, &pathext)
}

/// Inner search logic, separated for testability (avoids mutating process-global env).
fn search_path_with(path_var: &std::ffi::OsStr, extensions: &[String]) -> Option<PathBuf> {
    for dir_path in std::env::split_paths(path_var) {
        if cfg!(windows) {
            for ext in extensions {
                let candidate = dir_path.join(format!("oscal-cli{ext}"));
                if let Ok(metadata) = std::fs::metadata(&candidate)
                    && metadata.is_file()
                    && is_executable(&metadata)
                {
                    return candidate.canonicalize().ok().or(Some(candidate));
                }
            }
        } else {
            let candidate = dir_path.join("oscal-cli");
            if let Ok(metadata) = std::fs::metadata(&candidate)
                && metadata.is_file()
                && is_executable(&metadata)
            {
                return candidate.canonicalize().ok().or(Some(candidate));
            }
        }
    }

    None
}

#[cfg(unix)]
fn is_executable(metadata: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;

    metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn is_executable(_metadata: &std::fs::Metadata) -> bool {
    // Windows candidates have an executable PATHEXT extension; other targets preserve prior behavior.
    true
}

/// Parse PATHEXT into a list of lowercased extensions.
///
/// Returns the default Windows extensions when the input is empty.
fn parse_pathext(pathext: &str) -> Vec<String> {
    let extensions: Vec<String> =
        pathext.split(';').filter(|e| !e.is_empty()).map(str::to_lowercase).collect();

    if extensions.is_empty() {
        vec![".com".to_string(), ".exe".to_string(), ".bat".to_string(), ".cmd".to_string()]
    } else {
        extensions
    }
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

    // --- PATHEXT parsing tests ---

    #[test]
    fn parse_pathext_standard_windows_value() {
        let exts = parse_pathext(".COM;.EXE;.BAT;.CMD");
        assert_eq!(exts, vec![".com", ".exe", ".bat", ".cmd"]);
    }

    #[test]
    fn parse_pathext_lowercases_extensions() {
        let exts = parse_pathext(".EXE;.Bat;.CMD");
        assert_eq!(exts, vec![".exe", ".bat", ".cmd"]);
    }

    #[test]
    fn parse_pathext_filters_empty_segments() {
        let exts = parse_pathext(".EXE;;.BAT;");
        assert_eq!(exts, vec![".exe", ".bat"]);
    }

    #[test]
    fn parse_pathext_empty_string_returns_defaults() {
        let exts = parse_pathext("");
        assert_eq!(exts, vec![".com", ".exe", ".bat", ".cmd"]);
    }

    #[test]
    fn parse_pathext_custom_extensions() {
        let exts = parse_pathext(".EXE;.PS1;.VBS");
        assert_eq!(exts, vec![".exe", ".ps1", ".vbs"]);
    }

    #[cfg(unix)]
    #[test]
    fn search_path_finds_bare_binary_on_unix() {
        use std::ffi::OsStr;
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::tempdir().unwrap();
        let bin_path = tmp.path().join("oscal-cli");
        std::fs::write(&bin_path, "#!/bin/sh\necho test").unwrap();
        std::fs::set_permissions(&bin_path, std::fs::Permissions::from_mode(0o755)).unwrap();

        let path_var = OsStr::new(tmp.path().to_str().unwrap());
        let result = search_path_with(path_var, &[]);

        assert!(result.is_some());
        let found = result.unwrap();
        assert!(found.ends_with("oscal-cli"));
    }

    #[cfg(unix)]
    #[test]
    fn search_path_skips_directories_and_non_executable_files_on_unix() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::tempdir().unwrap();
        let directory_entry = tmp.path().join("directory-entry");
        let non_executable_entry = tmp.path().join("non-executable-entry");
        let executable_entry = tmp.path().join("executable-entry");
        std::fs::create_dir_all(directory_entry.join("oscal-cli")).unwrap();
        std::fs::create_dir_all(&non_executable_entry).unwrap();
        std::fs::create_dir_all(&executable_entry).unwrap();

        let non_executable = non_executable_entry.join("oscal-cli");
        std::fs::write(&non_executable, "#!/bin/sh\necho blocked").unwrap();
        std::fs::set_permissions(&non_executable, std::fs::Permissions::from_mode(0o644)).unwrap();

        let executable = executable_entry.join("oscal-cli");
        std::fs::write(&executable, "#!/bin/sh\necho found").unwrap();
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755)).unwrap();

        let path_var =
            std::env::join_paths([&directory_entry, &non_executable_entry, &executable_entry])
                .unwrap();
        let expected = executable.canonicalize().unwrap();
        let result = search_path_with(path_var.as_os_str(), &[]);

        assert_eq!(result.as_deref(), Some(expected.as_path()));
    }

    #[cfg(windows)]
    #[test]
    fn search_path_finds_exe_on_windows() {
        use std::ffi::OsStr;

        let tmp = tempfile::tempdir().unwrap();
        let bin_path = tmp.path().join("oscal-cli.exe");
        std::fs::write(&bin_path, "placeholder").unwrap();

        let path_var = OsStr::new(tmp.path().to_str().unwrap());
        let extensions = vec![".exe".to_string(), ".bat".to_string(), ".cmd".to_string()];
        let result = search_path_with(path_var, &extensions);

        assert!(result.is_some());
        let found = result.unwrap();
        assert!(found.to_string_lossy().ends_with("oscal-cli.exe"));
    }

    #[cfg(windows)]
    #[test]
    fn search_path_skips_directory_before_bat_wrapper_on_windows() {
        use std::ffi::OsStr;

        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("oscal-cli.exe")).unwrap();
        let bin_path = tmp.path().join("oscal-cli.bat");
        std::fs::write(&bin_path, "@echo off\necho oscal-cli").unwrap();

        let path_var = OsStr::new(tmp.path().to_str().unwrap());
        // .exe first, then .bat — should still find .bat since .exe doesn't exist
        let extensions = vec![".exe".to_string(), ".bat".to_string()];
        let result = search_path_with(path_var, &extensions);

        assert!(result.is_some());
        let found = result.unwrap();
        assert!(found.to_string_lossy().ends_with("oscal-cli.bat"));
    }
}
