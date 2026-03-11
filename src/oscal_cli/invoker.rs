//! oscal-cli invocation — subprocess execution with timeout and error handling.

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Instant;

use super::{OscalCliInvoke, ResolveArgs, ResolveResult};
use crate::error::ForgeError;

/// Invokes oscal-cli as a subprocess.
pub struct ProcessInvoker {
    executable_path: PathBuf,
}

impl ProcessInvoker {
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

impl OscalCliInvoke for ProcessInvoker {
    fn resolve_profile(&self, args: &ResolveArgs) -> Result<ResolveResult, ForgeError> {
        let mut cmd = Command::new(&self.executable_path);
        cmd.args(["profile", "resolve", "-to=json"]);
        cmd.arg(&args.profile_path);
        cmd.arg(&args.output_path);

        // SEC-7: Clear environment and apply allowlist
        cmd.env_clear();
        for key in ENV_ALLOWLIST {
            if let Ok(val) = std::env::var(key) {
                cmd.env(key, val);
            }
        }

        // Capture stderr via pipe (stdout goes to file via oscal-cli's output arg)
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| ForgeError::OscalCliExecution {
            exit_code: None,
            message: format!("Failed to spawn oscal-cli: {e}"),
            stderr: String::new(),
        })?;

        // Poll-based timeout: check every 100ms
        let start = Instant::now();
        let timeout = args.timeout;
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) => {
                    if start.elapsed() >= timeout {
                        let _ = child.kill();
                        let _ = child.wait(); // Reap the zombie
                        return Err(ForgeError::OscalCliTimeout { timeout });
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                Err(e) => {
                    return Err(ForgeError::OscalCliExecution {
                        exit_code: None,
                        message: format!("Failed to wait for oscal-cli: {e}"),
                        stderr: String::new(),
                    });
                }
            }
        };

        // Read stderr after process completes
        let stderr_str = child
            .stderr
            .take()
            .map(|mut stderr| {
                let mut buf = String::new();
                if let Err(e) = std::io::Read::read_to_string(&mut stderr, &mut buf) {
                    tracing::warn!("Failed to read oscal-cli stderr: {e}");
                }
                buf
            })
            .unwrap_or_default();

        if !status.success() {
            let message = extract_error_message(&stderr_str);
            return Err(ForgeError::OscalCliExecution {
                exit_code: status.code(),
                message,
                stderr: stderr_str,
            });
        }

        // Success — collect any stderr warnings
        let warnings = if stderr_str.trim().is_empty() {
            Vec::new()
        } else {
            stderr_str.lines().filter(|l| !l.trim().is_empty()).map(String::from).collect()
        };

        Ok(ResolveResult { output_path: args.output_path.clone(), warnings })
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
        let warnings: Vec<String> =
            stderr.lines().filter(|l| !l.trim().is_empty()).map(String::from).collect();
        assert_eq!(warnings.len(), 2);
        assert!(warnings[0].contains("deprecated"));
    }

    // --- T044: timeout handling is tested via integration tests ---
    // The timeout mechanism uses polling with try_wait + kill.
    // Full timeout behavior requires a real long-running subprocess.
}
