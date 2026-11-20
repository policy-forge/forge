//! OS confinement for the local process adapter (PRD 066 S-1, F16).
//!
//! Two Unix mechanisms, both fail-closed: macOS spawns through
//! `/usr/bin/sandbox-exec` with a deny-default Seatbelt profile; Linux applies
//! a Landlock filesystem ruleset plus a seccomp-BPF filter before `exec`. Both
//! are inherited by every descendant. The child's environment is cleared, it
//! leads a fresh process group (so a timeout or refusal can kill every
//! descendant), and its stdout/stderr are drained under a byte cap.
//!
//! The boundary is filesystem and network only: CPU, memory and process-count
//! limits are not enforced, and the executable runs with the operator's own
//! user identity inside the sandbox. Windows has no approved mechanism and
//! stays refused. See `docs/local-model-confinement.md` for the exact
//! guarantees and how to verify them.

use std::path::Path;
use std::process::Child;
#[cfg(unix)]
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::ForgeError;

use super::adapter::AdapterOutput;
use super::response::MAX_RESPONSE_BYTES;
use super::shared;

/// Drained stderr bound, in bytes.
const MAX_STDERR_BYTES: usize = 64 * 1024;
/// Poll interval while waiting for the child to exit.
const POLL: Duration = Duration::from_millis(5);
/// Grace period to drain the pipes after a kill before detaching readers.
const DRAIN_GRACE: Duration = Duration::from_secs(2);

/// Run the verified executable under OS confinement and return its output.
///
/// `executable` must resolve to the bytes that hash to `expected_sha`; the
/// bytes are re-read here immediately before spawn and a replacement is
/// refused. On platforms without an approved mechanism this always fails.
///
/// # Errors
/// Returns an authoring error when confinement cannot be established, the
/// executable changed, the child exits non-zero, exceeds the timeout or an
/// output bound, or cannot be spawned.
pub(super) fn invoke_confined(
    executable: &Path,
    expected_sha: &str,
    argv: &[String],
    payload: &[u8],
    timeout: Duration,
) -> Result<AdapterOutput, ForgeError> {
    let resolved = std::fs::canonicalize(executable)?;
    let bytes = crate::io::read_bounded(&resolved, crate::suggest::prepare::MAX_ADAPTER_BYTES)?;
    if crate::hashing::sha256_hex(&bytes) != expected_sha {
        return Err(shared::error(
            "the adapter executable changed since it was fingerprinted; re-run prepare and consent again",
        ));
    }
    let scratch = tempfile::Builder::new().prefix("forge-model-").tempdir().map_err(|cause| {
        shared::error(format!("cannot create the adapter scratch directory: {cause}"))
    })?;
    let scratch_path = scratch.path().canonicalize().map_err(|cause| {
        shared::error(format!("cannot resolve the adapter scratch directory: {cause}"))
    })?;
    let started = Instant::now();
    let mut child = spawn(&resolved, argv, &scratch_path)?;
    let (result, drained) = supervise(&mut child, payload, timeout, started);
    if drained {
        // All pipe threads are joined: no descendant can still write, so the
        // scratch directory can be removed safely.
        drop(scratch);
    } else {
        // Readers were detached: an escaped descendant may still write into
        // the scratch directory. Keep it (never recycle the path) instead of
        // deleting it under a live writer.
        let _ = scratch.keep();
    }
    result
}

/// Spawn the confined child in its own process group with piped output.
#[cfg(unix)]
fn spawn(executable: &Path, argv: &[String], scratch: &Path) -> Result<Child, ForgeError> {
    use std::os::unix::process::CommandExt as _;

    let mut command = platform_command(executable, argv, scratch)?;
    command
        .env_clear()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        // A fresh group lets one killpg reach every descendant on refusal.
        .process_group(0);
    command
        .spawn()
        .map_err(|cause| shared::error(format!("cannot spawn the confined adapter: {cause}")))
}

#[cfg(not(unix))]
fn spawn(_executable: &Path, _argv: &[String], _scratch: &Path) -> Result<Child, ForgeError> {
    Err(shared::error(super::adapter::PROCESS_EXECUTION_UNAVAILABLE))
}

/// Feed the payload, drain both pipes under their caps and enforce the timeout.
fn supervise(
    child: &mut Child,
    payload: &[u8],
    timeout: Duration,
    started: Instant,
) -> (Result<AdapterOutput, ForgeError>, bool) {
    let missing_pipe = || shared::error("the confined adapter lost a standard stream");
    let Some(stdout) = child.stdout.take() else { return (Err(missing_pipe()), true) };
    let Some(stderr) = child.stderr.take() else { return (Err(missing_pipe()), true) };
    let Some(mut stdin) = child.stdin.take() else { return (Err(missing_pipe()), true) };
    let stdout_cap = usize::try_from(MAX_RESPONSE_BYTES).unwrap_or(usize::MAX);
    let stdout_reader = std::thread::spawn(move || read_capped(stdout, stdout_cap));
    let stderr_reader = std::thread::spawn(move || read_capped(stderr, MAX_STDERR_BYTES));
    let payload = payload.to_vec();
    let stdin_writer = std::thread::spawn(move || {
        use std::io::Write as _;
        // A refused child may close stdin early; a broken pipe is expected.
        let _ = stdin.write_all(&payload);
    });

    let deadline = started + timeout;
    let mut failure = None;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    failure = Some(shared::error(format!(
                        "the confined adapter exited with {}",
                        status.code().map_or_else(
                            || "a signal".to_string(),
                            |code| format!("status {code}")
                        )
                    )));
                }
                break;
            }
            Ok(None) => {
                if Instant::now() >= deadline {
                    failure = Some(timeout_error());
                    break;
                }
                std::thread::sleep(POLL);
            }
            Err(cause) => {
                failure =
                    Some(shared::error(format!("cannot wait for the confined adapter: {cause}")));
                break;
            }
        }
    }
    if failure.is_some() {
        kill_group(child);
    } else {
        // Descendants can hold the pipes open after the leader exits; the
        // drain is part of the invocation and shares its deadline.
        while !(stdout_reader.is_finished() && stderr_reader.is_finished()) {
            if Instant::now() >= deadline {
                failure = Some(timeout_error());
                kill_group(child);
                break;
            }
            std::thread::sleep(POLL);
        }
    }
    // An escaped descendant can survive killpg holding a pipe open. Bound the
    // post-kill drain and detach any thread still blocked at the grace deadline
    // so the invocation always returns within a bounded time.
    let drain_deadline = Instant::now() + DRAIN_GRACE;
    let drained = finished_by(&stdout_reader, drain_deadline)
        && finished_by(&stderr_reader, drain_deadline)
        && finished_by(&stdin_writer, drain_deadline);
    if !drained {
        return (Err(failure.unwrap_or_else(timeout_error)), false);
    }
    collect_output(child, stdout_reader, stderr_reader, failure, started)
}

/// Join the drained readers and assemble the output. Reached only once every
/// pipe thread has finished, so the joins cannot block.
fn collect_output(
    child: &mut Child,
    stdout_reader: std::thread::JoinHandle<Result<Vec<u8>, ForgeError>>,
    stderr_reader: std::thread::JoinHandle<Result<Vec<u8>, ForgeError>>,
    failure: Option<ForgeError>,
    started: Instant,
) -> (Result<AdapterOutput, ForgeError>, bool) {
    let Ok(stdout) = stdout_reader.join() else {
        return (Err(shared::error("the stdout reader failed")), true);
    };
    let Ok(stderr) = stderr_reader.join() else {
        return (Err(shared::error("the stderr reader failed")), true);
    };
    if let Some(cause) = failure {
        return (Err(cause), true);
    }
    // An over-bound or failed drain is a refusal too: the child may still be
    // running, so kill its group before returning the error.
    let stdout = match stdout {
        Ok(bytes) => bytes,
        Err(cause) => {
            kill_group(child);
            return (Err(cause), true);
        }
    };
    let stderr = match stderr {
        Ok(bytes) => bytes,
        Err(cause) => {
            kill_group(child);
            return (Err(cause), true);
        }
    };
    let status = match child.wait() {
        Ok(status) => status,
        Err(cause) => {
            return (
                Err(shared::error(format!("cannot reap the confined adapter: {cause}"))),
                true,
            );
        }
    };
    (
        Ok(AdapterOutput {
            stdout,
            stderr: crate::sanitize::strip_control_chars(&String::from_utf8_lossy(&stderr)),
            elapsed: started.elapsed(),
            exit_code: status.code(),
        }),
        true,
    )
}

fn timeout_error() -> ForgeError {
    shared::error("the confined adapter exceeded its timeout")
}

/// Poll a thread until it finishes or the deadline passes; false means it is
/// still blocked and must be detached rather than joined.
fn finished_by<T>(handle: &std::thread::JoinHandle<T>, deadline: Instant) -> bool {
    while !handle.is_finished() {
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(POLL);
    }
    true
}

/// Kill the child's whole process group (Unix) or the child itself.
fn kill_group(child: &mut Child) {
    #[cfg(unix)]
    {
        // SAFETY: killpg only signals the group the child leads; the pid is a
        // live child we own.
        #[allow(unsafe_code)]
        unsafe {
            libc::killpg(child.id() as libc::pid_t, libc::SIGKILL);
        }
    }
    let _ = child.kill();
    let _ = child.wait();
}

/// Read at most `cap` bytes; an over-cap stream is an error, not a truncation.
fn read_capped(mut reader: impl std::io::Read, cap: usize) -> Result<Vec<u8>, ForgeError> {
    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 8192];
    loop {
        let read = reader
            .read(&mut chunk)
            .map_err(|cause| shared::error(format!("cannot drain the adapter output: {cause}")))?;
        if read == 0 {
            return Ok(bytes);
        }
        if bytes.len() + read > cap {
            return Err(shared::error("the adapter output exceeded its byte bound"));
        }
        bytes.extend_from_slice(&chunk[..read]);
    }
}

// ---------------------------------------------------------------------------
// macOS: deny-default Seatbelt via /usr/bin/sandbox-exec
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
fn platform_command(
    executable: &Path,
    argv: &[String],
    scratch: &Path,
) -> Result<Command, ForgeError> {
    const SANDBOX_EXEC: &str = "/usr/bin/sandbox-exec";
    let launcher = Path::new(SANDBOX_EXEC);
    if !launcher.is_file() {
        return Err(shared::error(format!(
            "{SANDBOX_EXEC} is unavailable; refusing to run the adapter unconfined"
        )));
    }
    let profile = seatbelt_profile(executable, scratch)?;
    let mut command = Command::new(launcher);
    command.arg("-p").arg(profile).arg(executable).args(argv);
    Ok(command)
}

/// The deny-default profile: read-only runtime and executable, one private
/// writable scratch directory, no network. Children inherit it.
#[cfg(target_os = "macos")]
fn seatbelt_profile(executable: &Path, scratch: &Path) -> Result<String, ForgeError> {
    use std::fmt::Write as _;
    let mut allow_read = Vec::new();
    // Ancestor directories need read access for path traversal.
    for ancestor in executable.ancestors().skip(1) {
        allow_read.push(format!("(literal \"{}\")", escape(ancestor)?));
    }
    for ancestor in scratch.ancestors().skip(1) {
        allow_read.push(format!("(literal \"{}\")", escape(ancestor)?));
    }
    let mut profile = String::from(
        "(version 1)\n(deny default)\n\
         (allow process*)\n(allow signal)\n(allow sysctl-read)\n(allow mach*)\n\
         (allow file-read* file-write* (literal \"/dev/null\") (literal \"/dev/zero\") \
         (literal \"/dev/random\") (literal \"/dev/urandom\") (literal \"/dev/tty\") \
         (literal \"/dev/dtracehelper\"))\n\
         (allow file-read* (subpath \"/usr/lib\") (subpath \"/bin\") (subpath \"/sbin\") \
         (subpath \"/usr/bin\") (subpath \"/usr/sbin\") (subpath \"/System\") \
         (subpath \"/private/var/db\") (subpath \"/private/var/select\") \
         (subpath \"/Library/Preferences\") (subpath \"/dev/fd\"))\n",
    );
    let _ = writeln!(
        profile,
        "(allow file-read* {} (literal \"{}\"))",
        allow_read.join(" "),
        escape(executable)?
    );
    let _ = writeln!(profile, "(allow file-read* file-write* (subpath \"{}\"))", escape(scratch)?);
    Ok(profile)
}

/// Quote a path for the Seatbelt profile language.
#[cfg(target_os = "macos")]
fn escape(path: &Path) -> Result<String, ForgeError> {
    let text = path.to_str().ok_or_else(|| shared::error("the adapter path must be UTF-8"))?;
    if text.contains(['"', '\\', ')', '(', '\n', '\r']) {
        return Err(shared::error(
            "the adapter path contains characters the Seatbelt profile cannot quote",
        ));
    }
    Ok(text.to_string())
}

// ---------------------------------------------------------------------------
// Linux: Landlock filesystem ruleset + seccomp-BPF network deny
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
fn platform_command(
    executable: &Path,
    argv: &[String],
    scratch: &Path,
) -> Result<Command, ForgeError> {
    use std::os::unix::process::CommandExt as _;

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    return Err(shared::error(
        "no approved seccomp policy exists for this architecture; refusing to run the adapter unconfined",
    ));
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    {
        // Probe Landlock before spawning so an unsupported kernel fails closed
        // with a clear error instead of a killed child.
        let abi = landlock_abi_version()?;
        // Precompute every allocation before fork: after fork in a
        // multi-threaded parent only async-signal-safe syscalls may run, so
        // the closure below installs this plan without allocating.
        let mut plan = linux_confinement(executable, scratch, abi)?;
        let mut command = Command::new(executable);
        command.args(argv);
        // SAFETY: the closure installs the precomputed Landlock ruleset and
        // seccomp filter between fork and exec using only async-signal-safe
        // syscalls; on any failure it returns the raw errno and the child
        // exits without executing the adapter.
        #[allow(unsafe_code)]
        unsafe {
            command.pre_exec(move || plan.install());
        }
        Ok(command)
    }
}

/// Landlock ABI version, or a clear refusal on kernels without Landlock.
#[cfg(target_os = "linux")]
fn landlock_abi_version() -> Result<i32, ForgeError> {
    // SAFETY: a null attribute with the query flag only asks for the ABI.
    #[allow(unsafe_code)]
    let version = unsafe {
        libc::syscall(
            libc::SYS_landlock_create_ruleset,
            std::ptr::null::<u8>(),
            0_usize,
            LANDLOCK_CREATE_RULESET_VERSION,
        )
    };
    if version < 1 {
        return Err(shared::error(format!(
            "Landlock is unavailable (kernel 5.13 or newer with Landlock enabled is required); \
             refusing to run the adapter unconfined (errno {})",
            std::io::Error::last_os_error()
        )));
    }
    i32::try_from(version).map_err(|_| shared::error("unexpected Landlock ABI version"))
}

#[cfg(target_os = "linux")]
const LANDLOCK_CREATE_RULESET_VERSION: u32 = 1;
#[cfg(target_os = "linux")]
const LANDLOCK_RULE_PATH_BENEATH: u32 = 1;

// Landlock access bits (uapi/linux/landlock.h); libc does not expose them yet.
#[cfg(target_os = "linux")]
mod landlock_access {
    pub const EXECUTE: u64 = 1 << 0;
    pub const WRITE_FILE: u64 = 1 << 1;
    pub const READ_FILE: u64 = 1 << 2;
    pub const READ_DIR: u64 = 1 << 3;
    pub const REMOVE_DIR: u64 = 1 << 4;
    pub const REMOVE_FILE: u64 = 1 << 5;
    pub const MAKE_CHAR: u64 = 1 << 6;
    pub const MAKE_DIR: u64 = 1 << 7;
    pub const MAKE_REG: u64 = 1 << 8;
    pub const MAKE_SOCK: u64 = 1 << 9;
    pub const MAKE_FIFO: u64 = 1 << 10;
    pub const MAKE_BLOCK: u64 = 1 << 11;
    pub const MAKE_SYM: u64 = 1 << 12;
    pub const REFER: u64 = 1 << 13;
    pub const TRUNCATE: u64 = 1 << 14;

    /// Every access right of ABI v1.
    pub const V1: u64 = EXECUTE
        | WRITE_FILE
        | READ_FILE
        | READ_DIR
        | REMOVE_DIR
        | REMOVE_FILE
        | MAKE_CHAR
        | MAKE_DIR
        | MAKE_REG
        | MAKE_SOCK
        | MAKE_FIFO
        | MAKE_BLOCK
        | MAKE_SYM;
}

#[cfg(target_os = "linux")]
#[repr(C)]
struct LandlockRulesetAttr {
    handled_access_fs: u64,
}

#[cfg(target_os = "linux")]
#[repr(C)]
struct LandlockPathBeneathAttr {
    allowed_access: u64,
    parent_fd: i32,
}

/// Everything the child needs to install confinement, precomputed before
/// fork so the post-fork closure performs no allocation.
#[cfg(target_os = "linux")]
struct LinuxConfinement {
    attr: LandlockRulesetAttr,
    exe_c: std::ffi::CString,
    scratch_c: std::ffi::CString,
    scratch_access: u64,
    /// Runtime paths (loader directories, linker cache, devices) and the
    /// access each is granted; a missing path is skipped at install time.
    runtime_rules: Vec<(std::ffi::CString, u64)>,
    filter: Vec<libc::sock_filter>,
    filter_len: u16,
}

/// Precompute the Landlock ruleset and seccomp filter before fork.
#[cfg(target_os = "linux")]
fn linux_confinement(
    executable: &Path,
    scratch: &Path,
    abi: i32,
) -> Result<LinuxConfinement, ForgeError> {
    use std::os::unix::ffi::OsStrExt as _;

    let mut handled = landlock_access::V1;
    if abi >= 2 {
        handled |= landlock_access::REFER;
    }
    if abi >= 3 {
        handled |= landlock_access::TRUNCATE;
    }
    let exe_c = std::ffi::CString::new(executable.as_os_str().as_bytes())
        .map_err(|_| shared::error("the executable path contains a NUL"))?;
    let scratch_c = std::ffi::CString::new(scratch.as_os_str().as_bytes())
        .map_err(|_| shared::error("the scratch path contains a NUL"))?;
    let read = landlock_access::READ_FILE | landlock_access::READ_DIR;
    let mut runtime_rules = Vec::new();
    // The dynamic loader needs the runtime directories, the linker cache and
    // null/zero/random devices. Missing paths (merged-usr layouts) are skipped
    // at install; a present path that cannot be ruled fails closed.
    for path in ["/usr/lib", "/lib", "/usr/bin", "/bin", "/usr/sbin", "/sbin"] {
        runtime_rules.push((runtime_c(path)?, read | landlock_access::EXECUTE));
    }
    runtime_rules.push((runtime_c("/etc/ld.so.cache")?, landlock_access::READ_FILE));
    for path in ["/dev/null", "/dev/zero", "/dev/random", "/dev/urandom"] {
        runtime_rules
            .push((runtime_c(path)?, landlock_access::READ_FILE | landlock_access::WRITE_FILE));
    }
    let filter = seccomp_filter();
    let filter_len = u16::try_from(filter.len())
        .map_err(|_| shared::error("the seccomp filter is too large"))?;
    Ok(LinuxConfinement {
        attr: LandlockRulesetAttr { handled_access_fs: handled },
        exe_c,
        scratch_c,
        scratch_access: handled
            & !(landlock_access::EXECUTE
                | landlock_access::MAKE_SOCK
                | landlock_access::MAKE_CHAR
                | landlock_access::MAKE_BLOCK),
        runtime_rules,
        filter,
        filter_len,
    })
}

/// A constant runtime path as a `CString` (validated before fork).
#[cfg(target_os = "linux")]
fn runtime_c(path: &str) -> Result<std::ffi::CString, ForgeError> {
    std::ffi::CString::new(path).map_err(|_| shared::error("a runtime path contains a NUL"))
}

#[cfg(target_os = "linux")]
#[allow(unsafe_code)]
impl LinuxConfinement {
    /// Install Landlock then seccomp between fork and exec. Only
    /// async-signal-safe syscalls run here; errors carry the raw errno, which
    /// std reports to the parent without allocating. Any failure leaves the
    /// child un-exec'd (fail closed).
    fn install(&mut self) -> std::io::Result<()> {
        // SAFETY: all arguments are precomputed and live in `self`; the calls
        // below are plain syscalls on the child between fork and exec.
        unsafe {
            // no_new_privs must precede both Landlock and seccomp.
            if libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            let ruleset = libc::syscall(
                libc::SYS_landlock_create_ruleset,
                &raw const self.attr,
                std::mem::size_of::<LandlockRulesetAttr>(),
                0_u32,
            );
            if ruleset < 0 {
                return Err(std::io::Error::last_os_error());
            }
            // A successful `landlock_create_ruleset` returns a file descriptor, which always fits
            // in an `i32`; a fallible conversion would add an allocation after fork.
            #[allow(clippy::cast_possible_truncation)]
            let ruleset = ruleset as i32;
            let result = self.install_rules(ruleset);
            libc::close(ruleset);
            result?;
            let program =
                libc::sock_fprog { len: self.filter_len, filter: self.filter.as_mut_ptr() };
            if libc::prctl(
                libc::PR_SET_SECCOMP,
                libc::SECCOMP_MODE_FILTER,
                &raw const program,
                0,
                0,
            ) != 0
            {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        }
    }

    unsafe fn install_rules(&self, ruleset: i32) -> std::io::Result<()> {
        // SAFETY: ruleset is live and all paths are pre-built CStrings; only
        // async-signal-safe syscalls run here.
        unsafe {
            // O_PATH opens without reading the file; the rules bind the inode.
            let exe_fd = libc::open(self.exe_c.as_ptr(), libc::O_PATH | libc::O_CLOEXEC);
            if exe_fd < 0 {
                return Err(std::io::Error::last_os_error());
            }
            let scratch_fd = libc::open(self.scratch_c.as_ptr(), libc::O_PATH | libc::O_CLOEXEC);
            if scratch_fd < 0 {
                let err = std::io::Error::last_os_error();
                libc::close(exe_fd);
                return Err(err);
            }
            let result = Self::add_rule(
                ruleset,
                exe_fd,
                landlock_access::READ_FILE | landlock_access::EXECUTE,
            )
            .and_then(|()| Self::add_rule(ruleset, scratch_fd, self.scratch_access))
            .and_then(|()| {
                for (path, access) in &self.runtime_rules {
                    let fd = libc::open(path.as_ptr(), libc::O_PATH | libc::O_CLOEXEC);
                    if fd < 0 {
                        let err = std::io::Error::last_os_error();
                        if err.raw_os_error() == Some(libc::ENOENT) {
                            continue;
                        }
                        return Err(err);
                    }
                    let added = Self::add_rule(ruleset, fd, *access);
                    libc::close(fd);
                    added?;
                }
                Ok(())
            })
            .and_then(|()| {
                if libc::syscall(libc::SYS_landlock_restrict_self, ruleset, 0_u32) < 0 {
                    Err(std::io::Error::last_os_error())
                } else {
                    Ok(())
                }
            });
            libc::close(exe_fd);
            libc::close(scratch_fd);
            result
        }
    }

    /// SAFETY: `fd` is an open `O_PATH` descriptor and `ruleset` a live ruleset fd.
    unsafe fn add_rule(ruleset: i32, fd: i32, access: u64) -> std::io::Result<()> {
        let rule = LandlockPathBeneathAttr { allowed_access: access, parent_fd: fd };
        // SAFETY: rule is a valid path-beneath attribute for the live ruleset.
        let rc = unsafe {
            libc::syscall(
                libc::SYS_landlock_add_rule,
                ruleset,
                LANDLOCK_RULE_PATH_BENEATH,
                &raw const rule,
                0_u32,
            )
        };
        if rc < 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(())
    }
}

/// Build the deny-list filter for the running architecture.
// The BPF opcodes combined here (`BPF_JMP | BPF_JEQ | BPF_K` and friends) are small
// compile-time constants that fit the 16-bit `code` field of `sock_filter`.
#[allow(clippy::cast_possible_truncation)]
#[cfg(target_os = "linux")]
fn seccomp_filter() -> Vec<libc::sock_filter> {
    let insn = |code: u16, k: u32| libc::sock_filter { code, jt: 0, jf: 0, k };
    let ret = |k: u32| insn((libc::BPF_RET | libc::BPF_K) as u16, k);
    // seccomp_data: nr at offset 0, arch at offset 4. Check the architecture
    // first, then reload the syscall number for the deny list.
    let mut filter = vec![insn((libc::BPF_LD | libc::BPF_W | libc::BPF_ABS) as u16, 4)];
    #[cfg(target_arch = "x86_64")]
    let arch: u32 = 0xC000_003E; // AUDIT_ARCH_X86_64
    #[cfg(target_arch = "aarch64")]
    let arch: u32 = 0xC000_00B7; // AUDIT_ARCH_AARCH64
    // Unknown architecture: arch 0 never matches, so everything is killed.
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    let arch: u32 = 0;
    // A foreign ABI (e.g. i386 on x86_64) must not slip past the deny list.
    filter.push(libc::sock_filter {
        code: (libc::BPF_JMP | libc::BPF_JEQ | libc::BPF_K) as u16,
        jt: 1,
        jf: 0,
        k: arch,
    });
    filter.push(ret(libc::SECCOMP_RET_KILL_PROCESS));
    filter.push(insn((libc::BPF_LD | libc::BPF_W | libc::BPF_ABS) as u16, 0));
    #[cfg(target_arch = "x86_64")]
    {
        // x32 syscalls carry bit 30; they are not in the deny list.
        filter.push(libc::sock_filter {
            code: (libc::BPF_JMP | libc::BPF_JGE | libc::BPF_K) as u16,
            jt: 0,
            jf: 1,
            k: 0x4000_0000,
        });
        filter.push(ret(libc::SECCOMP_RET_KILL_PROCESS));
    }
    for syscall in denied_syscalls() {
        filter.push(libc::sock_filter {
            code: (libc::BPF_JMP | libc::BPF_JEQ | libc::BPF_K) as u16,
            jt: 0,
            jf: 1,
            k: syscall,
        });
        filter.push(ret(libc::SECCOMP_RET_ERRNO | libc::EPERM as u32));
    }
    filter.push(ret(libc::SECCOMP_RET_ALLOW));
    filter
}

/// Socket-creating syscalls plus the cross-process access syscalls present on this architecture.
///
/// Landlock confines filesystem access but not access to other processes of the same user, so
/// `ptrace`, `process_vm_readv/writev` and `pidfd_getfd` are denied as well: depending on
/// `kernel.yama.ptrace_scope` they could otherwise reach processes outside the sandbox.
/// Signalling (`kill`) is not denied because a confined adapter legitimately manages its own
/// children; see `docs/local-model-confinement.md`.
// Syscall numbers are small non-negative constants that fit the 32-bit `k` field of a BPF
// instruction.
#[allow(clippy::cast_possible_truncation)]
#[cfg(target_os = "linux")]
fn denied_syscalls() -> Vec<u32> {
    #[cfg_attr(not(target_arch = "x86"), allow(unused_mut))]
    let mut numbers = vec![
        libc::SYS_socket as u32,
        libc::SYS_socketpair as u32,
        libc::SYS_io_uring_setup as u32,
        libc::SYS_io_uring_enter as u32,
        libc::SYS_io_uring_register as u32,
        libc::SYS_ptrace as u32,
        libc::SYS_process_vm_readv as u32,
        libc::SYS_process_vm_writev as u32,
        libc::SYS_pidfd_getfd as u32,
    ];
    // i386 routes socket calls through one multiplexer; 64-bit architectures
    // have no socketcall.
    #[cfg(target_arch = "x86")]
    numbers.push(libc::SYS_socketcall as u32);
    numbers
}

// ---------------------------------------------------------------------------
// Other Unix platforms: no approved mechanism, fail closed.
// ---------------------------------------------------------------------------

#[cfg(all(unix, not(any(target_os = "macos", target_os = "linux"))))]
fn platform_command(
    _executable: &Path,
    _argv: &[String],
    _scratch: &Path,
) -> Result<Command, ForgeError> {
    Err(shared::error(
        "no OS confinement mechanism is approved for this platform; refusing to run the adapter unconfined",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finished_by_detaches_a_thread_that_outlives_the_deadline() {
        // A reader blocked on a pipe an escaped descendant holds open never
        // finishes; finished_by must report false at the deadline so supervise
        // detaches it instead of joining forever.
        let blocked = std::thread::spawn(|| std::thread::sleep(Duration::from_secs(30)));
        let start = Instant::now();
        assert!(!finished_by(&blocked, start + Duration::from_millis(50)));
        assert!(start.elapsed() < Duration::from_secs(1), "the wait was not bounded");
        // A finished thread is reported immediately.
        let done = std::thread::spawn(|| {});
        assert!(finished_by(&done, Instant::now() + Duration::from_secs(5)));
    }
}
