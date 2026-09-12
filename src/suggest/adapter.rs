//! The local model boundary: a typed invoke seam over a child process.
//!
//! FORGE never resolves a provider, opens a socket, or reads a credential. The
//! operator names a local executable; the payload is written to its standard
//! input, its standard output is the response, and both streams are bounded.
//! The recorded-response adapter implements the same trait, which is what makes
//! the offline workflow and the CI suite possible without any model.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use crate::ForgeError;

use super::request::MAX_ARG_BYTES;
use super::response::MAX_RESPONSE_BYTES;
use super::shared;

/// Maximum adapter standard error bytes drained before the run is refused.
pub const MAX_STDERR_BYTES: usize = 256 * 1024;
/// Response stream bound as a host `usize`; a test ties it to the contract bound.
pub const RESPONSE_STREAM_BYTES: usize = 8 * 1024 * 1024;
/// Poll interval while waiting for the adapter to exit.
const POLL_INTERVAL: Duration = Duration::from_millis(100);
/// Environment variables handed to a local adapter, and nothing else.
#[cfg(unix)]
const ENV_ALLOWLIST: [&str; 4] = ["PATH", "HOME", "TMPDIR", "LANG"];
/// Environment variables handed to a local adapter, and nothing else.
#[cfg(windows)]
const ENV_ALLOWLIST: [&str; 8] =
    ["PATH", "HOME", "TMPDIR", "LANG", "USERPROFILE", "SYSTEMROOT", "TEMP", "TMP"];

/// What one adapter invocation returned.
#[derive(Debug)]
pub struct AdapterOutput {
    /// Exact standard output bytes.
    pub stdout: Vec<u8>,
    /// Drained standard error, sanitized for diagnostics.
    pub stderr: String,
    /// Measured wall time of the invocation.
    pub elapsed: Duration,
    /// Process exit code, or `None` when the adapter was not a process.
    pub exit_code: Option<i32>,
}

/// One local model invocation.
///
/// Implementations are synchronous and must be dispatched to a blocking
/// executor by async callers.
pub trait LocalModelInvoke {
    /// Send the exact payload and return the bounded response.
    ///
    /// # Errors
    /// Returns an authoring error for a missing or unusable adapter, a spawn
    /// failure, a non-zero exit, a timeout, or output past the bound.
    fn invoke(&self, payload: &[u8], timeout: Duration) -> Result<AdapterOutput, ForgeError>;

    /// The adapter executable digest consent was bound to, when it is a process.
    fn executable_sha256(&self) -> Option<&str> {
        None
    }
}

/// A local executable adapter: stdin is the payload, stdout is the response.
#[derive(Debug)]
pub struct ProcessModelAdapter {
    executable: PathBuf,
    executable_sha256: String,
    argv: Vec<String>,
}

impl ProcessModelAdapter {
    /// Bind an adapter to an operator-supplied local executable.
    ///
    /// # Errors
    /// Returns an authoring error when the executable is missing, is not a
    /// regular file, is a symlink, or is larger than the fingerprint bound.
    pub fn new(executable: &Path, argv: &[String]) -> Result<Self, ForgeError> {
        let bytes =
            crate::io::read_bounded(executable, crate::suggest::prepare::MAX_ADAPTER_BYTES)?;
        for argument in argv {
            if argument.len() > MAX_ARG_BYTES || argument.chars().any(char::is_control) {
                return Err(shared::error(format!(
                    "adapter arguments must be at most {MAX_ARG_BYTES} bytes"
                )));
            }
        }
        Ok(Self {
            executable: executable.to_path_buf(),
            executable_sha256: crate::hashing::sha256_hex(&bytes),
            argv: argv.to_vec(),
        })
    }

    /// The executable path this adapter was bound to.
    #[must_use]
    pub fn executable(&self) -> &Path {
        &self.executable
    }
}

impl LocalModelInvoke for ProcessModelAdapter {
    fn invoke(&self, payload: &[u8], timeout: Duration) -> Result<AdapterOutput, ForgeError> {
        let mut command = Command::new(&self.executable);
        command.args(&self.argv);
        let start = Instant::now();
        let mut child = spawn(&mut command)?;
        let oversize = Arc::new(AtomicBool::new(false));
        let stdin = child.stdin.take();
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let payload = payload.to_vec();
        let writer = std::thread::spawn(move || {
            use std::io::Write as _;
            if let Some(mut stdin) = stdin {
                let _ = stdin.write_all(&payload);
            }
        });
        let out_flag = Arc::clone(&oversize);
        let reader =
            std::thread::spawn(move || read_capped(stdout, RESPONSE_STREAM_BYTES, &out_flag));
        let err_flag = Arc::clone(&oversize);
        let drainer = std::thread::spawn(move || read_capped(stderr, MAX_STDERR_BYTES, &err_flag));

        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break Some(status),
                Ok(None) if oversize.load(Ordering::SeqCst) => {
                    reap(&mut child);
                    break None;
                }
                Ok(None) if start.elapsed() >= timeout => {
                    reap(&mut child);
                    return Err(shared::error(format!(
                        "the local adapter timed out after {timeout:?}"
                    )));
                }
                Ok(None) => std::thread::sleep(POLL_INTERVAL),
                Err(error) => {
                    reap(&mut child);
                    return Err(shared::error(format!(
                        "cannot wait for the local adapter: {error}"
                    )));
                }
            }
        };
        let _ = writer.join();
        let stdout =
            reader.join().map_err(|_| shared::error("the adapter output reader failed"))??;
        let stderr =
            drainer.join().map_err(|_| shared::error("the adapter error reader failed"))??;
        let Some(status) = status else {
            return Err(bound_error());
        };
        if oversize.load(Ordering::SeqCst) {
            return Err(bound_error());
        }
        let stderr = crate::sanitize::strip_control_chars(&String::from_utf8_lossy(&stderr));
        if !status.success() {
            return Err(shared::error(format!(
                "the local adapter exited with status {}: {}",
                status.code().map_or_else(|| "signal".to_string(), |code| code.to_string()),
                crate::json_strict::bounded(&stderr)
            )));
        }
        Ok(AdapterOutput { stdout, stderr, elapsed: start.elapsed(), exit_code: status.code() })
    }

    fn executable_sha256(&self) -> Option<&str> {
        Some(&self.executable_sha256)
    }
}

/// An offline adapter that replays one recorded response file.
#[derive(Debug)]
pub struct RecordedResponseAdapter {
    bytes: Vec<u8>,
}

impl RecordedResponseAdapter {
    /// Read a recorded response through the bounded, no-follow reader.
    ///
    /// # Errors
    /// Returns an authoring error when the file is missing, unsafe or oversized.
    pub fn new(path: &Path) -> Result<Self, ForgeError> {
        let bytes = crate::io::read_bounded(path, MAX_RESPONSE_BYTES)?;
        Ok(Self { bytes })
    }
}

impl LocalModelInvoke for RecordedResponseAdapter {
    fn invoke(&self, _payload: &[u8], _timeout: Duration) -> Result<AdapterOutput, ForgeError> {
        Ok(AdapterOutput {
            stdout: self.bytes.clone(),
            stderr: String::new(),
            elapsed: Duration::ZERO,
            exit_code: Some(0),
        })
    }
}

fn spawn(command: &mut Command) -> Result<Child, ForgeError> {
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    command.env_clear();
    for name in ENV_ALLOWLIST {
        if let Some(value) = std::env::var_os(name) {
            command.env(name, value);
        }
    }
    command.spawn().map_err(|error| {
        shared::error(format!(
            "cannot start the local adapter '{}': {error}",
            crate::io::sanitize_artifact_path(Path::new(command.get_program()))
        ))
    })
}

/// One refusal for either stream past its bound.
fn bound_error() -> ForgeError {
    shared::error(format!(
        "the local adapter exceeded the {MAX_RESPONSE_BYTES} byte response bound or the {MAX_STDERR_BYTES} byte error bound"
    ))
}

/// Kill and reap a child so no adapter outlives a refusal.
fn reap(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

/// Read a stream up to `max` bytes, flagging any read past the bound.
fn read_capped(
    stream: Option<impl Read>,
    max: usize,
    oversize: &AtomicBool,
) -> Result<Vec<u8>, ForgeError> {
    let Some(stream) = stream else {
        return Ok(Vec::new());
    };
    let mut collected = Vec::new();
    let mut buffer = [0_u8; 8192];
    let mut stream = stream;
    loop {
        let read = stream
            .read(&mut buffer)
            .map_err(|error| shared::error(format!("cannot read adapter output: {error}")))?;
        if read == 0 {
            return Ok(collected);
        }
        if collected.len() + read > max {
            oversize.store(true, Ordering::SeqCst);
            return Ok(collected);
        }
        collected.extend_from_slice(&buffer[..read]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_stream_bound_matches_the_response_contract() {
        assert_eq!(RESPONSE_STREAM_BYTES as u64, MAX_RESPONSE_BYTES);
    }

    #[test]
    fn recorded_adapter_replays_the_exact_bytes_without_a_process() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("response.json");
        std::fs::write(&path, b"{\"recorded\":true}\n").unwrap();
        let adapter = RecordedResponseAdapter::new(&path).unwrap();
        let output = adapter.invoke(b"payload", Duration::from_secs(1)).unwrap();
        assert_eq!(output.stdout, b"{\"recorded\":true}\n");
        assert_eq!(output.exit_code, Some(0));
        assert!(adapter.executable_sha256().is_none());
    }

    #[cfg(unix)]
    #[test]
    fn a_process_adapter_round_trips_the_payload_through_stdin_and_stdout() {
        let adapter =
            ProcessModelAdapter::new(Path::new("/bin/sh"), &["-c".into(), "cat".into()]).unwrap();
        assert_eq!(adapter.executable(), Path::new("/bin/sh"));
        assert!(adapter.executable_sha256().is_some_and(|value| value.len() == 64));
        let output = adapter.invoke(b"exact payload bytes", Duration::from_secs(10)).unwrap();
        assert_eq!(output.stdout, b"exact payload bytes");
        assert_eq!(output.exit_code, Some(0));
    }

    #[cfg(unix)]
    #[test]
    fn a_failing_or_hanging_adapter_is_refused_and_reaped() {
        let failing =
            ProcessModelAdapter::new(Path::new("/bin/sh"), &["-c".into(), "exit 3".into()])
                .unwrap();
        let error = failing.invoke(b"", Duration::from_secs(10)).unwrap_err().to_string();
        assert!(error.contains("exited with status 3"), "{error}");

        let hanging = ProcessModelAdapter::new(
            Path::new("/bin/sh"),
            &["-c".into(), "printf 'partial'; sleep 30".into()],
        )
        .unwrap();
        let start = Instant::now();
        let error = hanging.invoke(b"", Duration::from_millis(300)).unwrap_err().to_string();
        assert!(error.contains("timed out"), "{error}");
        assert!(
            start.elapsed() < Duration::from_secs(10),
            "the adapter must be killed, not awaited"
        );
    }

    #[cfg(unix)]
    #[test]
    fn oversized_output_or_error_streams_are_refused() {
        let stderr_flood = ProcessModelAdapter::new(
            Path::new("/bin/sh"),
            &["-c".into(), format!("head -c {} /dev/zero 1>&2", MAX_STDERR_BYTES + 4096)],
        )
        .unwrap();
        let error = stderr_flood.invoke(b"", Duration::from_secs(30)).unwrap_err().to_string();
        assert!(error.contains("bound"), "{error}");

        let stdout_flood = ProcessModelAdapter::new(
            Path::new("/bin/sh"),
            &["-c".into(), format!("head -c {} /dev/zero", RESPONSE_STREAM_BYTES + 1)],
        )
        .unwrap();
        let error = stdout_flood.invoke(b"", Duration::from_secs(30)).unwrap_err().to_string();
        assert!(error.contains("bound"), "{error}");
    }

    #[test]
    fn a_missing_or_unsafe_adapter_is_refused() {
        let temp = tempfile::tempdir().unwrap();
        let missing = temp.path().join("absent-adapter");
        assert!(ProcessModelAdapter::new(&missing, &[]).is_err());
        assert!(RecordedResponseAdapter::new(&missing).is_err());

        let directory = temp.path().join("dir");
        std::fs::create_dir(&directory).unwrap();
        assert!(ProcessModelAdapter::new(&directory, &[]).is_err());

        let argument = vec!["x".repeat(MAX_ARG_BYTES + 1)];
        assert!(ProcessModelAdapter::new(Path::new("/bin/sh"), &argument).is_err());
    }
}
