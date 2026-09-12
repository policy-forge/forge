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

/// Read a local executable the operator named, following symlinks to the file.
///
/// An executable is not an input document: `/bin/sh`, Homebrew shims, `nvm` and
/// virtualenv entry points are all symlinks, so requiring a non-symlink here
/// would refuse ordinary installs. The path must still resolve to a regular
/// file — never a directory, device, FIFO or socket — and the bytes read are the
/// ones consent is bound to, so replacing the target after consent is caught.
///
/// # Errors
/// Returns [`ForgeError::Io`] for a missing path, a non-regular file, unreadable
/// bytes, or a file larger than `max`.
pub(in crate::suggest) fn read_executable(
    path: &Path,
    max: u64,
) -> Result<Vec<u8>, crate::ForgeError> {
    let metadata = std::fs::metadata(path)?;
    if !metadata.is_file() {
        return Err(crate::ForgeError::Io(std::io::Error::other(format!(
            "an adapter executable must be a regular file: {}",
            crate::io::sanitize_artifact_path(path)
        ))));
    }
    if metadata.len() > max {
        return Err(crate::ForgeError::Io(std::io::Error::other(format!(
            "an adapter executable must be at most {max} bytes: {}",
            crate::io::sanitize_artifact_path(path)
        ))));
    }
    Ok(std::fs::read(path)?)
}

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
    /// Returns an authoring error when the executable is missing, resolves to
    /// something other than a regular file, or is larger than the fingerprint
    /// bound. Symlinks are followed: ordinary installs are shims.
    pub fn new(executable: &Path, argv: &[String]) -> Result<Self, ForgeError> {
        let bytes = read_executable(executable, crate::suggest::prepare::MAX_ADAPTER_BYTES)?;
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
        // Bind execution to the bytes that were fingerprinted, immediately before
        // use: a file or symlink swapped between construction and invocation is
        // refused rather than executed. A residual window remains between this
        // check and the spawn; closing it would need descriptor-bound exec.
        let current = read_executable(&self.executable, super::prepare::MAX_ADAPTER_BYTES)?;
        if crate::hashing::sha256_hex(&current) != self.executable_sha256 {
            return Err(shared::error(
                "the adapter executable changed after consent; re-run prepare and consent again",
            ));
        }
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
        let (stdout_sender, stdout_receiver) = std::sync::mpsc::channel();
        let _reader = std::thread::spawn(move || {
            let _ = stdout_sender.send(read_capped(stdout, RESPONSE_STREAM_BYTES, &out_flag));
        });
        let err_flag = Arc::clone(&oversize);
        let (stderr_sender, stderr_receiver) = std::sync::mpsc::channel();
        let _drainer = std::thread::spawn(move || {
            let _ = stderr_sender.send(read_capped(stderr, MAX_STDERR_BYTES, &err_flag));
        });

        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break Some(status),
                Ok(None) if oversize.load(Ordering::SeqCst) => {
                    terminate(&mut child);
                    break None;
                }
                Ok(None) if start.elapsed() >= timeout => {
                    terminate(&mut child);
                    return Err(shared::error(format!(
                        "the local adapter timed out after {timeout:?}"
                    )));
                }
                Ok(None) => std::thread::sleep(POLL_INTERVAL),
                Err(error) => {
                    terminate(&mut child);
                    return Err(shared::error(format!(
                        "cannot wait for the local adapter: {error}"
                    )));
                }
            }
        };
        if status.is_none() || oversize.load(Ordering::SeqCst) {
            // Terminated, or past a stream bound. Either way the readers are
            // detached: a descendant may still hold the pipes open.
            return Err(bound_error());
        }
        // The child exited, but a descendant it started can still hold the pipes
        // open. Wait for the readers only within the caller's own budget, so a
        // leaked pipe becomes a refusal instead of a hang.
        let deadline = start + timeout;
        let stdout = receive(&stdout_receiver, deadline)?;
        let stderr = receive(&stderr_receiver, deadline)?;
        let _ = writer.join();
        let status = status.expect("checked above");
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
    // A local adapter is free to start its own children; its own process group is
    // what lets a refusal reclaim the whole tree on Unix.
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt as _;
        command.process_group(0);
    }
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

/// Wait for one reader thread's result within the caller's remaining budget.
///
/// # Errors
/// Returns an authoring error when the budget runs out, which means the adapter
/// exited while something it started kept the stream open.
fn receive(
    receiver: &std::sync::mpsc::Receiver<Result<Vec<u8>, ForgeError>>,
    deadline: Instant,
) -> Result<Vec<u8>, ForgeError> {
    let remaining = deadline.saturating_duration_since(Instant::now());
    receiver.recv_timeout(remaining).map_err(|_| {
        shared::error(
            "the local adapter exited while a descendant still held its output; refusing the response",
        )
    })?
}

/// One refusal for either stream past its bound.
fn bound_error() -> ForgeError {
    shared::error(format!(
        "the local adapter exceeded the {MAX_RESPONSE_BYTES} byte response bound or the {MAX_STDERR_BYTES} byte error bound"
    ))
}

/// Kill and reap the adapter, and on Unix every process in its group.
///
/// The direct child is always killed and waited for. On Unix the adapter runs in
/// its own process group, so the group is signalled too: otherwise a descendant
/// could keep running, and keep the pipes open, after the refusal.
#[allow(unsafe_code)] // Reviewed single-signal process-group termination.
fn terminate(child: &mut Child) {
    #[cfg(unix)]
    {
        // SAFETY: `id()` is this live child's pid and the child was started with
        // `process_group(0)`, so its group id equals its pid. `killpg` sends one
        // signal to that group and cannot outlive it.
        let Ok(group) = i32::try_from(child.id()) else {
            let _ = child.kill();
            let _ = child.wait();
            return;
        };
        unsafe {
            libc::killpg(group, libc::SIGKILL);
        }
    }
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
    fn an_executable_may_be_a_symlink_but_never_a_special_file() {
        // `/bin/sh` is a symlink on Linux and a regular file on macOS; both are fine.
        let resolved = read_executable(Path::new("/bin/sh"), 64 * 1024 * 1024).unwrap();
        assert!(!resolved.is_empty());
        assert!(read_executable(Path::new("/"), 64 * 1024 * 1024).is_err());
        let temp = tempfile::tempdir().unwrap();
        assert!(read_executable(temp.path(), 1024).is_err());
        let missing = temp.path().join("absent");
        assert!(read_executable(&missing, 1024).is_err());
        let small = temp.path().join("executable");
        std::fs::write(&small, b"x").unwrap();
        assert!(read_executable(&small, 1).is_ok());
        assert!(read_executable(&small, 0).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn a_process_adapter_may_be_reached_through_a_symlink() {
        // The wiring, not just the helper: a shim is how most installs look.
        let temp = tempfile::tempdir().unwrap();
        let link = temp.path().join("shim");
        std::os::unix::fs::symlink("/bin/sh", &link).unwrap();
        let adapter = ProcessModelAdapter::new(&link, &["-c".into(), "cat".into()]).unwrap();
        let output = adapter.invoke(b"through a shim", Duration::from_secs(10)).unwrap();
        assert_eq!(output.stdout, b"through a shim");
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

    #[cfg(unix)]
    #[test]
    fn an_executable_replaced_after_fingerprinting_is_refused() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("adapter");
        std::fs::write(&path, b"#!/bin/sh\ncat\n").unwrap();
        let adapter = ProcessModelAdapter::new(&path, &[]).unwrap();
        std::fs::write(&path, b"#!/bin/sh\nprintf 'replaced'\n").unwrap();
        let error = adapter.invoke(b"payload", Duration::from_secs(10)).unwrap_err().to_string();
        assert!(error.contains("changed after consent"), "{error}");
    }

    #[cfg(unix)]
    #[test]
    fn a_descendant_holding_the_output_is_refused_rather_than_awaited() {
        // The child exits at once, but the background sleep keeps stdout open.
        let adapter = ProcessModelAdapter::new(
            Path::new("/bin/sh"),
            &["-c".into(), "sleep 20 & exit 0".into()],
        )
        .unwrap();
        let start = Instant::now();
        let error = adapter.invoke(b"", Duration::from_millis(600)).unwrap_err().to_string();
        assert!(error.contains("descendant"), "{error}");
        assert!(start.elapsed() < Duration::from_secs(10), "must not wait for the descendant");
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

        // An existing file, so the refusal is about the argument, not the path.
        let executable = temp.path().join("adapter");
        std::fs::write(&executable, b"#!/bin/sh\n").unwrap();
        let argument = vec!["x".repeat(MAX_ARG_BYTES + 1)];
        assert!(ProcessModelAdapter::new(&executable, &argument).is_err());
        assert!(ProcessModelAdapter::new(&executable, &["ok".to_string()]).is_ok());
    }
}
