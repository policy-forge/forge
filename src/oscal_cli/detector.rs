//! oscal-cli detection — PATH search and version verification.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const VERSION_CHECK_TIMEOUT: Duration = Duration::from_secs(5);

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
                Err(error) => {
                    let detail = format!(
                        "Configured oscal-cli path '{}' could not be canonicalized: {error}",
                        path.display()
                    );
                    tracing::warn!(%detail, "oscal-cli discovery failed");
                    return OscalCliInfo::unavailable(detail);
                }
            },
            None => search_path_for_oscal_cli(),
        };

        let Some(exe_path) = executable_path else {
            return OscalCliInfo::not_found();
        };

        match run_version_check(&exe_path) {
            Ok(version) => OscalCliInfo::functional(exe_path, version),
            Err(detail) => {
                tracing::warn!(path = %exe_path.display(), %detail, "oscal-cli is not functional");
                OscalCliInfo::not_functional(exe_path, detail)
            }
        }
    }
}

/// Search the system PATH for the oscal-cli binary.
///
/// On Windows, discovers only native `.exe` candidates; wrappers require an
/// explicit shell invocation and are intentionally excluded. On Unix, searches
/// for the bare `oscal-cli` name (no extension).
fn search_path_for_oscal_cli() -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    let pathext = if cfg!(windows) {
        let raw = std::env::var("PATHEXT").unwrap_or_else(|_| ".EXE".to_string());
        parse_pathext(&raw)
    } else {
        vec![]
    };
    search_path_with(&path_var, &pathext)
}

/// Inner search logic, separated for testability (avoids mutating process-global env).
fn search_path_with(path_var: &std::ffi::OsStr, extensions: &[String]) -> Option<PathBuf> {
    for dir_path in std::env::split_paths(path_var) {
        // Empty and relative PATH entries resolve against the cwd, letting a
        // planted ./oscal-cli win discovery (CWE-426); only absolute dirs count.
        if !dir_path.is_absolute() {
            continue;
        }
        if cfg!(windows) {
            for ext in extensions.iter().filter(|extension| extension.eq_ignore_ascii_case(".exe"))
            {
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

/// Select the sole Windows extension that can be directly spawned safely.
///
/// Discovery intentionally ignores user-controlled PATHEXT ordering and wrappers:
/// native oscal-cli installations expose an .exe binary.
fn parse_pathext(_pathext: &str) -> Vec<String> {
    vec![".exe".to_string()]
}

/// Run `oscal-cli --version` within a bounded interval and extract its version.
fn run_version_check(exe_path: &Path) -> Result<String, String> {
    let mut child = Command::new(exe_path)
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Failed to execute oscal-cli: {error}"))?;

    // Drain both pipes while polling: a child writing more than the OS pipe
    // buffer blocks on write and would otherwise be falsely killed as a
    // timeout. Threads send the complete bytes over channels so collection
    // stays bounded even if a descendant keeps a pipe open after kill.
    let (out_tx, out_rx) = std::sync::mpsc::channel();
    let (err_tx, err_rx) = std::sync::mpsc::channel();
    let mut stdout_pipe = child.stdout.take();
    let mut stderr_pipe = child.stderr.take();
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(pipe) = &mut stdout_pipe {
            let _ = std::io::Read::read_to_end(pipe, &mut bytes);
        }
        let _ = out_tx.send(bytes);
    });
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(pipe) = &mut stderr_pipe {
            let _ = std::io::Read::read_to_end(pipe, &mut bytes);
        }
        let _ = err_tx.send(bytes);
    });

    let start = Instant::now();
    let mut timed_out = false;
    let mut status = None;
    loop {
        match child.try_wait() {
            Ok(Some(exit)) => {
                status = Some(exit);
                break;
            }
            Ok(None) if start.elapsed() < VERSION_CHECK_TIMEOUT => {
                std::thread::sleep(Duration::from_millis(25));
            }
            Ok(None) => {
                timed_out = true;
                let _ = child.kill();
                let _ = child.wait();
                break;
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("Failed while waiting for oscal-cli --version: {error}"));
            }
        }
    }

    let grace = Duration::from_secs(1);
    let stdout = out_rx.recv_timeout(grace).unwrap_or_default();
    let stderr = crate::sanitize::strip_control_chars(&String::from_utf8_lossy(
        &err_rx.recv_timeout(grace).unwrap_or_default(),
    ));
    if timed_out {
        return Err(if stderr.is_empty() {
            format!("oscal-cli --version timed out after {VERSION_CHECK_TIMEOUT:?}")
        } else {
            format!("oscal-cli --version timed out after {VERSION_CHECK_TIMEOUT:?}: {stderr}")
        });
    }
    let status = status.ok_or_else(|| "oscal-cli --version exit status lost".to_string())?;
    if !status.success() {
        return Err(if stderr.is_empty() {
            format!("oscal-cli --version exited with {status}")
        } else {
            format!("oscal-cli --version exited with {status}: {stderr}")
        });
    }

    parse_version_from_output(&String::from_utf8_lossy(&stdout))
        .ok_or_else(|| "oscal-cli --version produced no semver-shaped version token".to_string())
}

/// Extract a semver-shaped version token from the oscal-cli version line.
///
/// Restricting the search to tokens following the oscal-cli program name avoids
/// selecting embedded runtime or build-tool versions from startup banners.
fn parse_version_from_output(output: &str) -> Option<String> {
    output.lines().find_map(|line| {
        let mut tokens = line.split_whitespace();
        while let Some(token) = tokens.next() {
            if token.eq_ignore_ascii_case("oscal-cli") {
                return tokens.find(|candidate| is_semver_token(candidate)).map(str::to_string);
            }
        }
        None
    })
}

fn is_semver_token(token: &str) -> bool {
    let (core, suffix) =
        token.split_once(['-', '+']).map_or((token, None), |(core, suffix)| (core, Some(suffix)));
    if suffix.is_some_and(str::is_empty) {
        return false;
    }

    let mut parts = core.split('.');
    let valid_core = parts.clone().count() == 3
        && parts.all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()));
    let valid_suffix = suffix.is_none_or(|value| {
        !value.is_empty()
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'+'))
    });
    valid_core && valid_suffix
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- T021: PathDetector found and functional ---
    // Note: This test uses mock detection since we can't guarantee oscal-cli on PATH.
    // We test the version parser and struct construction directly.

    #[cfg(unix)]
    #[test]
    fn version_check_drains_pipes_so_chatty_children_do_not_false_timeout() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        let exe = tmp.path().join("oscal-cli");
        // >64 KiB of stderr fills the OS pipe buffer; without concurrent
        // draining the child blocks on write and is falsely killed.
        std::fs::write(
            &exe,
            "#!/bin/sh\ni=0; while [ $i -lt 2000 ]; do echo 'banner warning line padding padding padding padding' >&2; i=$((i+1)); done\necho 'oscal-cli 1.2.3'\n",
        )
        .unwrap();
        std::fs::set_permissions(&exe, std::fs::Permissions::from_mode(0o755)).unwrap();
        let version = run_version_check(&exe).expect("chatty child must still be detected");
        assert_eq!(version, "1.2.3");
    }

    #[test]
    fn parse_version_from_standard_output() {
        let version = parse_version_from_output("oscal-cli 1.0.3\n");
        assert_eq!(version.as_deref(), Some("1.0.3"));
    }

    #[test]
    fn parse_version_from_verbose_output() {
        let version = parse_version_from_output("oscal-cli version 2.1.0-rc.1+build.7\n");
        assert_eq!(version.as_deref(), Some("2.1.0-rc.1+build.7"));
    }

    #[test]
    fn parse_version_fallback_to_trimmed() {
        assert_eq!(parse_version_from_output("unknown format\n"), None);
        assert_eq!(
            parse_version_from_output("openjdk 17.0.10\noscal-cli 1.2.3\n").as_deref(),
            Some("1.2.3")
        );
        assert_eq!(parse_version_from_output("\n"), None);
    }

    #[test]
    fn detector_with_nonexistent_override_returns_unavailable() {
        let detector = PathDetector::with_path(PathBuf::from("/nonexistent/oscal-cli"));
        let info = detector.detect();
        assert!(!info.is_available());
        assert!(!info.is_functional());
        assert!(info.version().is_none());
        assert!(info.executable_path().is_none());
        assert!(info.detail().is_some());
    }

    #[test]
    fn oscal_cli_info_not_found_has_correct_fields() {
        let info = OscalCliInfo::not_found();
        assert!(!info.is_available());
        assert!(!info.is_functional());
        assert!(info.version().is_none());
        assert!(info.executable_path().is_none());
        assert!(info.detail().is_none());
    }

    #[test]
    fn oscal_cli_info_found_and_functional() {
        let info = OscalCliInfo::functional(
            PathBuf::from("/usr/local/bin/oscal-cli"),
            "1.0.3".to_string(),
        );
        assert!(info.is_available());
        assert!(info.is_functional());
        assert_eq!(info.version(), Some("1.0.3"));
        assert_eq!(info.executable_path(), Some(Path::new("/usr/local/bin/oscal-cli")));
    }

    #[test]
    fn oscal_cli_info_found_but_not_functional() {
        let info = OscalCliInfo::not_functional(
            PathBuf::from("/usr/local/bin/oscal-cli"),
            "version check failed".to_string(),
        );
        assert!(info.is_available());
        assert!(!info.is_functional());
        assert!(info.version().is_none());
        assert!(info.executable_path().is_some());
        assert_eq!(info.detail(), Some("version check failed"));
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
        assert_eq!(exts, vec![".exe"]);
    }

    #[test]
    fn parse_pathext_lowercases_extensions() {
        let exts = parse_pathext(".EXE;.Bat;.CMD");
        assert_eq!(exts, vec![".exe"]);
    }

    #[test]
    fn parse_pathext_filters_empty_segments() {
        let exts = parse_pathext(".BAT;;.CMD;");
        assert_eq!(exts, vec![".exe"]);
    }

    #[test]
    fn parse_pathext_empty_string_returns_defaults() {
        assert_eq!(parse_pathext(""), vec![".exe"]);
    }

    #[test]
    fn parse_pathext_custom_extensions() {
        assert_eq!(parse_pathext(".PS1;.VBS"), vec![".exe"]);
    }

    #[cfg(unix)]
    #[test]
    fn search_path_skips_empty_and_relative_entries() {
        use std::ffi::OsStr;
        use std::os::unix::fs::PermissionsExt;

        // A relative PATH entry resolves against the attacker-chosen cwd;
        // a planted ./oscal-cli must never be discovered.
        let tmp = tempfile::Builder::new().prefix("path-rel-").tempdir_in(".").unwrap();
        let relative = tmp.path().file_name().unwrap().to_owned();
        let bin_path = tmp.path().join("oscal-cli");
        std::fs::write(&bin_path, "#!/bin/sh\necho test").unwrap();
        std::fs::set_permissions(&bin_path, std::fs::Permissions::from_mode(0o755)).unwrap();

        let joined = std::env::join_paths([std::path::PathBuf::new(), relative.into()]).unwrap();
        assert!(
            search_path_with(OsStr::new(&joined), &[]).is_none(),
            "empty and relative PATH entries must be skipped"
        );
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
        let extensions = vec![".exe".to_string(), ".bat".to_string()];
        assert!(search_path_with(path_var, &extensions).is_none());
    }
}
