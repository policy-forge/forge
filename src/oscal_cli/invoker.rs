//! oscal-cli invocation — subprocess execution with timeout and error handling.

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::{ConvertArgs, ConvertResult, OscalCliInvoke, ResolveArgs, ResolveResult};
use crate::error::ForgeError;

/// Invokes oscal-cli as a subprocess.
pub struct ProcessInvoker {
    executable_path: PathBuf,
}

impl ProcessInvoker {
    /// Create a new `ProcessInvoker` that will invoke the oscal-cli binary
    /// at the given `executable_path`.
    #[must_use]
    pub fn new(executable_path: PathBuf) -> Self {
        Self { executable_path }
    }
}

/// Environment variable allowlist for the oscal-cli child process.
const ENV_ALLOWLIST: &[&str] = if cfg!(windows) {
    &["PATH", "HOME", "JAVA_HOME", "TMPDIR", "USERPROFILE", "SYSTEMROOT", "TEMP", "TMP"]
} else {
    &["PATH", "HOME", "JAVA_HOME", "TMPDIR"]
};

/// Result of a successful subprocess execution.
struct SubprocessOutput {
    /// Sanitized stderr warnings (non-empty lines).
    warnings: Vec<String>,
}

/// Prepare, spawn, and wait for an oscal-cli subprocess with timeout and stderr handling.
///
/// Handles: env allowlist, stderr drain thread, poll-based timeout, sanitization,
/// error extraction, and warnings collection.
fn run_oscal_cli(
    cmd: &mut Command,
    timeout: Duration,
    context: &str,
) -> Result<SubprocessOutput, ForgeError> {
    // SEC-7: Clear environment and apply allowlist
    cmd.env_clear();
    for key in ENV_ALLOWLIST {
        if let Ok(val) = std::env::var(key) {
            cmd.env(key, val);
        }
    }

    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| ForgeError::OscalCliExecution {
        exit_code: None,
        message: format!("Failed to spawn oscal-cli {context}: {e}"),
    })?;

    // Drain stderr on a dedicated thread to prevent pipe buffer deadlock.
    // Without this, if oscal-cli writes >4KB (macOS) or >64KB (Linux) to stderr,
    // the child blocks on write and the parent blocks on try_wait — deadlock.
    let mut stderr_handle = child.stderr.take().expect("stderr was piped");
    let stderr_thread = std::thread::spawn(move || {
        let mut buf = String::new();
        let _ = std::io::Read::read_to_string(&mut stderr_handle, &mut buf);
        buf
    });

    // Poll-based timeout: check every 100ms
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait(); // Reap the zombie
                    let _ = stderr_thread.join(); // Prevent thread leak
                    return Err(ForgeError::OscalCliTimeout { timeout });
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                return Err(ForgeError::OscalCliExecution {
                    exit_code: None,
                    message: format!("Failed to wait for oscal-cli {context}: {e}"),
                });
            }
        }
    };

    // Join the stderr drain thread (process has exited, so this won't block long)
    let stderr_str = stderr_thread.join().unwrap_or_default();

    // Sanitize stderr to prevent terminal escape injection (SEC-5).
    let stderr_str = crate::sanitize::strip_control_chars(&stderr_str);

    if !status.success() {
        if !stderr_str.is_empty() {
            tracing::warn!(stderr = %stderr_str, "oscal-cli {context} failed with stderr output");
        }
        let message = extract_error_message(&stderr_str);
        return Err(ForgeError::OscalCliExecution { exit_code: status.code(), message });
    }

    // Success — log stderr at debug level for diagnostics
    if !stderr_str.is_empty() {
        tracing::debug!(stderr = %stderr_str, "oscal-cli {context} stderr output");
    }

    let warnings = collect_warnings(&stderr_str);

    Ok(SubprocessOutput { warnings })
}

/// Collect non-empty stderr lines as warning strings.
fn collect_warnings(stderr: &str) -> Vec<String> {
    if stderr.trim().is_empty() {
        Vec::new()
    } else {
        stderr.lines().filter(|l| !l.trim().is_empty()).map(String::from).collect()
    }
}

impl OscalCliInvoke for ProcessInvoker {
    fn resolve_profile(&self, args: &ResolveArgs) -> Result<ResolveResult, ForgeError> {
        let mut cmd = Command::new(&self.executable_path);
        cmd.args(["profile", "resolve", "-to=json"]);
        cmd.arg(&args.profile_path);
        cmd.arg(&args.output_path);

        let output = run_oscal_cli(&mut cmd, args.timeout, "resolve")?;

        Ok(ResolveResult { output_path: args.output_path.clone(), warnings: output.warnings })
    }

    fn convert(&self, args: &ConvertArgs) -> Result<ConvertResult, ForgeError> {
        let to_flag = format!("-to={}", args.output_format.to_cli_flag());

        let mut cmd = Command::new(&self.executable_path);
        cmd.args(["convert", &to_flag]);
        cmd.arg(&args.input_path);
        cmd.arg(&args.output_path);

        let output = run_oscal_cli(&mut cmd, args.timeout, "convert")?;

        Ok(ConvertResult { output_path: args.output_path.clone(), warnings: output.warnings })
    }
}

/// Extract the most meaningful error message from oscal-cli stderr.
///
/// Skips Java stack trace lines (starting with "at " or "\tat ") and returns
/// the last meaningful non-empty line.
#[must_use]
pub(crate) fn extract_error_message(stderr: &str) -> String {
    stderr
        .lines()
        .rfind(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty()
                && !trimmed.starts_with("at ")
                && !trimmed.starts_with("\tat ")
                && !trimmed.starts_with("Caused by:")
        })
        .unwrap_or("Unknown error")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- T022: ProcessInvoker success path is tested via integration tests ---
    // Mock-based unit tests for the invoker logic.

    // --- T042: stderr parsing ---

    #[test]
    fn extract_error_message_from_simple_error() {
        let stderr = "Error: Invalid profile format\n";
        assert_eq!(extract_error_message(stderr), "Error: Invalid profile format");
    }

    #[test]
    fn extract_error_message_skips_java_stack_trace() {
        let stderr = "java.lang.RuntimeException: Profile resolution failed\n\
                       at org.nist.oscal.Main.run(Main.java:42)\n\
                       at org.nist.oscal.Main.main(Main.java:10)\n";
        assert_eq!(
            extract_error_message(stderr),
            "java.lang.RuntimeException: Profile resolution failed"
        );
    }

    #[test]
    fn extract_error_message_skips_tabbed_stack_trace() {
        let stderr = "Exception in thread \"main\"\n\
                       \tat org.example.Foo.bar(Foo.java:1)\n\
                       \tat org.example.Main.main(Main.java:5)\n";
        assert_eq!(extract_error_message(stderr), "Exception in thread \"main\"");
    }

    #[test]
    fn extract_error_message_empty_stderr() {
        assert_eq!(extract_error_message(""), "Unknown error");
        assert_eq!(extract_error_message("   \n  \n"), "Unknown error");
    }

    // --- T043: multi-line stderr with meaningful last line ---

    #[test]
    fn extract_error_message_multi_line_picks_last_meaningful() {
        let stderr = "WARNING: some warning\n\
                       ERROR: The profile file is malformed\n";
        assert_eq!(extract_error_message(stderr), "ERROR: The profile file is malformed");
    }

    // --- T045: stderr warnings with exit 0 ---

    #[test]
    fn warnings_collected_from_nonempty_stderr() {
        let stderr = "WARNING: deprecated feature used\nWARNING: old format\n";
        let warnings = collect_warnings(stderr);
        assert_eq!(warnings.len(), 2);
        assert!(warnings[0].contains("deprecated"));
    }

    // --- T044: timeout handling is tested via integration tests ---
    // The timeout mechanism uses polling with try_wait + kill.
    // Full timeout behavior requires a real long-running subprocess.

    // --- T060: large stderr — documents intent of concurrent drain fix ---

    #[test]
    fn extract_error_message_handles_large_stderr() {
        // Simulate stderr larger than macOS pipe buffer (4KB)
        let mut stderr = String::new();
        for i in 0..1000 {
            use std::fmt::Write;
            let _ = writeln!(stderr, "WARNING: line {i}");
        }
        stderr.push_str("ERROR: final meaningful error\n");
        assert_eq!(extract_error_message(&stderr), "ERROR: final meaningful error");
    }
}
