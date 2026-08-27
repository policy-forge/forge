//! oscal-cli invocation — subprocess execution with timeout and error handling.

use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde::de::{self, Deserialize, IgnoredAny, MapAccess, Visitor};

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
    cmd.env_clear();
    for key in ENV_ALLOWLIST {
        match std::env::var(key) {
            Ok(value) => {
                tracing::debug!(
                    environment_variable = *key,
                    "Passing allowlisted environment variable to oscal-cli"
                );
                cmd.env(key, value);
            }
            Err(std::env::VarError::NotPresent) => {
                tracing::debug!(
                    environment_variable = *key,
                    "Allowlisted environment variable is absent"
                );
            }
            Err(std::env::VarError::NotUnicode(_)) => {
                tracing::warn!(
                    environment_variable = *key,
                    "Allowlisted environment variable is not valid Unicode and was omitted"
                );
            }
        }
    }

    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|error| ForgeError::OscalCliExecution {
        exit_code: None,
        message: format!("Failed to spawn oscal-cli {context}: {error}"),
    })?;

    let Some(mut stderr_handle) = child.stderr.take() else {
        let _ = child.kill();
        let _ = child.wait();
        return Err(ForgeError::OscalCliExecution {
            exit_code: None,
            message: format!("oscal-cli {context} did not provide a stderr pipe"),
        });
    };
    let stderr_thread = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = std::io::Read::read_to_end(&mut stderr_handle, &mut bytes);
        String::from_utf8_lossy(&bytes).into_owned()
    });

    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if start.elapsed() >= timeout => {
                let partial_stderr = cleanup_failed_child(&mut child, stderr_thread);
                let partial_stderr = crate::sanitize::strip_control_chars(&partial_stderr);
                if !partial_stderr.is_empty() {
                    tracing::warn!(
                        stderr = %partial_stderr,
                        "oscal-cli {context} timed out with partial stderr output"
                    );
                }
                return Err(ForgeError::OscalCliTimeout { timeout });
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(100)),
            Err(error) => {
                let _ = cleanup_failed_child(&mut child, stderr_thread);
                return Err(ForgeError::OscalCliExecution {
                    exit_code: None,
                    message: format!("Failed to wait for oscal-cli {context}: {error}"),
                });
            }
        }
    };

    let stderr_str = if let Ok(stderr) = stderr_thread.join() {
        stderr
    } else {
        tracing::error!("oscal-cli stderr collection thread panicked");
        String::new()
    };
    let stderr_str = crate::sanitize::strip_control_chars(&stderr_str);

    if !status.success() {
        if !stderr_str.is_empty() {
            tracing::warn!(stderr = %stderr_str, "oscal-cli {context} failed with stderr output");
        }
        return Err(ForgeError::OscalCliExecution {
            exit_code: status.code(),
            message: extract_error_message(&stderr_str),
        });
    }

    if !stderr_str.is_empty() {
        tracing::debug!(stderr = %stderr_str, "oscal-cli {context} stderr output");
    }

    Ok(SubprocessOutput { warnings: collect_warnings(&stderr_str) })
}

/// Cleans up a child process before an early error return and returns drained stderr.
///
/// Invariant: every early return after the stderr drain starts calls this helper, ensuring
/// neither a child process nor its stderr reader thread outlives a failed invocation.
fn cleanup_failed_child(
    child: &mut std::process::Child,
    stderr_thread: std::thread::JoinHandle<String>,
) -> String {
    let _ = child.kill();
    let _ = child.wait();
    if let Ok(stderr) = stderr_thread.join() {
        stderr
    } else {
        tracing::error!("oscal-cli stderr collection thread panicked during cleanup");
        String::new()
    }
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
        cmd.args(["profile", "resolve", "-to=json", "--"]);
        cmd.arg(&args.profile_path);
        cmd.arg(&args.output_path);

        let output = run_oscal_cli(&mut cmd, args.timeout, "resolve")?;

        Ok(ResolveResult { output_path: args.output_path.clone(), warnings: output.warnings })
    }

    fn convert(&self, args: &ConvertArgs) -> Result<ConvertResult, ForgeError> {
        let to_flag = format!("-to={}", args.output_format.to_cli_flag());
        let model_command = detect_model_command(&args.input_path)?;

        let mut cmd = Command::new(&self.executable_path);
        cmd.args([model_command, "convert", &to_flag, "--"]);
        cmd.arg(&args.input_path);
        cmd.arg(&args.output_path);

        let output = run_oscal_cli(&mut cmd, args.timeout, "convert")?;

        Ok(ConvertResult { output_path: args.output_path.clone(), warnings: output.warnings })
    }
}

/// A streaming deserializer that retains only the first string key of the root mapping.
struct RootKey(String);

impl<'de> Deserialize<'de> for RootKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct RootKeyVisitor;

        impl<'de> Visitor<'de> for RootKeyVisitor {
            type Value = RootKey;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("an OSCAL root mapping with a string key")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut root = None;
                while let Some(key) = map.next_key::<serde_yaml::Value>()? {
                    let key = key.as_str().map(str::to_owned);
                    map.next_value::<IgnoredAny>()?;
                    if root.is_none() {
                        root = key;
                    }
                }
                root.map(RootKey)
                    .ok_or_else(|| de::Error::custom("root mapping contains no string key"))
            }
        }

        deserializer.deserialize_map(RootKeyVisitor)
    }
}

fn root_key_from_json(reader: BufReader<File>) -> Result<String, String> {
    serde_json::from_reader::<_, RootKey>(reader)
        .map(|root| root.0)
        .map_err(|error| format!("invalid JSON: {error}"))
}

fn root_key_from_yaml(reader: BufReader<File>) -> Result<String, String> {
    serde_yaml::from_reader::<_, RootKey>(reader)
        .map(|root| root.0)
        .map_err(|error| format!("invalid YAML: {error}"))
}

/// Detect the oscal-cli model subcommand from the document root.
fn detect_model_command(path: &std::path::Path) -> Result<&'static str, ForgeError> {
    let extension = path.extension().and_then(std::ffi::OsStr::to_str).unwrap_or_default();
    let reader = || File::open(path).map(BufReader::new).map_err(ForgeError::Io);

    let root = match extension {
        "json" => root_key_from_json(reader()?).map_err(|error| ForgeError::OscalCliExecution {
            exit_code: None,
            message: format!("Unable to detect OSCAL model from '{}': {error}", path.display()),
        })?,
        "yaml" | "yml" => {
            root_key_from_yaml(reader()?).map_err(|error| ForgeError::OscalCliExecution {
                exit_code: None,
                message: format!("Unable to detect OSCAL model from '{}': {error}", path.display()),
            })?
        }
        "xml" => xml_root_name(reader()?).map_err(|error| ForgeError::OscalCliExecution {
            exit_code: None,
            message: format!("Unable to detect OSCAL model from '{}': {error}", path.display()),
        })?,
        _ => {
            return Err(ForgeError::OscalCliExecution {
                exit_code: None,
                message: format!("Unable to detect OSCAL model from '{}'", path.display()),
            });
        }
    };

    model_command_for_root(&root).ok_or_else(|| ForgeError::OscalCliExecution {
        exit_code: None,
        message: format!("Unsupported OSCAL document root '{root}' in '{}'", path.display()),
    })
}

fn xml_root_name(reader: BufReader<File>) -> Result<String, String> {
    let mut reader = quick_xml::Reader::from_reader(reader);
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(
                quick_xml::events::Event::Start(element) | quick_xml::events::Event::Empty(element),
            ) => {
                return std::str::from_utf8(element.local_name().as_ref())
                    .map(str::to_owned)
                    .map_err(|error| format!("invalid XML root name: {error}"));
            }
            Ok(quick_xml::events::Event::Eof) => {
                return Err("invalid XML: document contains no root element".to_string());
            }
            Ok(_) => {}
            Err(error) => return Err(format!("invalid XML: {error}")),
        }
        buffer.clear();
    }
}

fn model_command_for_root(root: &str) -> Option<&'static str> {
    match root {
        "catalog" => Some("catalog"),
        "profile" => Some("profile"),
        "component-definition" => Some("component-definition"),
        "system-security-plan" => Some("ssp"),
        "assessment-plan" => Some("ap"),
        "assessment-results" => Some("ar"),
        "plan-of-action-and-milestones" => Some("poam"),
        _ => None,
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

    #[test]
    fn detects_model_command_across_round_trip_formats() {
        let temp = tempfile::tempdir().unwrap();
        let json = temp.path().join("catalog.json");
        let xml = temp.path().join("catalog.xml");
        let yaml = temp.path().join("catalog.yaml");
        std::fs::write(&json, r#"{"catalog":{}}"#).unwrap();
        std::fs::write(&xml, r#"<?xml version="1.0"?><catalog/>"#).unwrap();
        std::fs::write(&yaml, "catalog: {}\n").unwrap();

        assert_eq!(detect_model_command(&json).unwrap(), "catalog");
        assert_eq!(detect_model_command(&xml).unwrap(), "catalog");
        assert_eq!(detect_model_command(&yaml).unwrap(), "catalog");
    }

    #[test]
    fn rejects_unknown_model_root() {
        let temp = tempfile::tempdir().unwrap();
        let input = temp.path().join("unknown.json");
        std::fs::write(&input, r#"{"unknown":{}}"#).unwrap();
        let error = detect_model_command(&input).unwrap_err();
        assert!(error.to_string().contains("Unsupported OSCAL document root"));
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

    #[test]
    fn model_detection_surfaces_json_parse_errors() {
        let temp = tempfile::tempdir().unwrap();
        let input = temp.path().join("truncated.json");
        std::fs::write(&input, r#"{"catalog": "#).unwrap();

        let error = detect_model_command(&input).unwrap_err();
        assert!(error.to_string().contains("invalid JSON"));
    }

    #[test]
    fn model_detection_skips_non_string_yaml_root_keys() {
        let temp = tempfile::tempdir().unwrap();
        let input = temp.path().join("catalog.yaml");
        std::fs::write(&input, "1: ignored\ncatalog: {}\n").unwrap();

        assert_eq!(detect_model_command(&input).unwrap(), "catalog");
    }
}
