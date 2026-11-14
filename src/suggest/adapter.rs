//! Recorded-response adapters and a confined process boundary.
//!
//! On macOS and Linux a process adapter runs under OS filesystem/network
//! confinement (Seatbelt or Landlock+seccomp), bound to verified bytes, with
//! every descendant owned by FORGE. Where no approved mechanism exists the
//! adapter fails closed; a local executable is not itself a sandbox. See
//! `docs/local-model-confinement.md` for the exact boundary.

use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::ForgeError;

use super::request::MAX_ARG_BYTES;
use super::response::MAX_RESPONSE_BYTES;
use super::shared;

/// Why the process adapter cannot execute on platforms without an approved
/// confinement mechanism, shared by CLI and library callers.
pub const PROCESS_EXECUTION_UNAVAILABLE: &str = "process adapter execution is disabled on this platform: no approved OS filesystem/network confinement mechanism exists; use --recorded-response with a separately reviewed response file";

/// Read a local executable the operator named, following symlinks to the file.
///
/// An executable is not an input document: `/bin/sh`, Homebrew shims, `nvm` and
/// virtualenv entry points are all symlinks, so requiring a non-symlink here
/// would refuse ordinary installs. The path must still resolve to a regular
/// file — never a directory, device, FIFO or socket — and the bytes read are the
/// ones consent is bound to. This helper never executes the file.
///
/// # Errors
/// Returns [`ForgeError::Io`] for a missing path, a non-regular file, unreadable
/// bytes, or a file larger than `max`.
pub(in crate::suggest) fn read_executable(
    path: &Path,
    max: u64,
) -> Result<Vec<u8>, crate::ForgeError> {
    // Resolve shims for fingerprinting. invoke re-reads the resolved bytes
    // immediately before spawn and refuses any replacement.
    let resolved = std::fs::canonicalize(path)?;
    crate::io::read_bounded(&resolved, max)
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

/// A fingerprinted local executable executed under OS confinement.
///
/// Invocation re-verifies the executable bytes, clears the environment, runs
/// the child under a deny-default filesystem/network sandbox in its own
/// process group, and kills the whole group on timeout or refusal. See
/// the `confinement` module and `docs/local-model-confinement.md`.
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
        super::confinement::invoke_confined(
            &self.executable,
            &self.executable_sha256,
            &self.argv,
            payload,
            timeout,
        )
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

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_ne!(resolved, [] as [u8; 0]);
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

    #[test]
    fn a_replaced_executable_is_refused_before_spawn() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("adapter");
        std::fs::write(&path, b"#!/bin/sh\ncat\n").unwrap();
        let adapter = ProcessModelAdapter::new(&path, &[]).unwrap();
        assert_eq!(adapter.executable(), path);
        assert!(adapter.executable_sha256().is_some());
        std::fs::write(&path, b"#!/bin/sh\nprintf replaced\n").unwrap();
        let error = adapter.invoke(b"private payload", Duration::from_secs(5)).unwrap_err();
        assert!(error.to_string().contains("changed since it was fingerprinted"));
    }

    #[cfg(not(unix))]
    #[test]
    fn process_invocation_is_unavailable_without_an_approved_mechanism() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("adapter");
        std::fs::write(&path, b"placeholder executable").unwrap();
        let adapter = ProcessModelAdapter::new(&path, &[]).unwrap();
        let error = adapter.invoke(b"private payload", Duration::ZERO).unwrap_err();
        assert!(error.to_string().contains(PROCESS_EXECUTION_UNAVAILABLE));
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
