//! F16 confined local model execution: refusal paths, verified bytes,
//! descendant cleanup, timeout, and the filesystem/network boundary.
//!
//! Platform tests are gated: macOS exercises Seatbelt through sandbox-exec,
//! Linux exercises Landlock + seccomp-BPF. On hosts where the mechanism is
//! unavailable (old kernel, missing sandbox-exec, a blocking container seccomp
//! profile) the adapter must fail closed, and the boundary tests assert that
//! refusal instead of an unconfined run.

#![cfg(any(target_os = "linux", target_os = "macos"))]

use std::path::Path;
use std::time::{Duration, Instant};

use forge::suggest::adapter::{LocalModelInvoke, ProcessModelAdapter};

fn sh_adapter(script: &str) -> ProcessModelAdapter {
    ProcessModelAdapter::new(Path::new("/bin/sh"), &["-c".to_string(), script.to_string()]).unwrap()
}

/// True when the host can establish confinement; false when the adapter must
/// (and does) fail closed. Probes with a trivial confined command.
fn confinement_available() -> bool {
    sh_adapter("printf ok").invoke(b"", Duration::from_secs(10)).is_ok()
}

#[test]
fn a_replaced_executable_is_refused_before_any_spawn() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("model");
    std::fs::write(&path, b"#!/bin/sh\ncat\n").unwrap();
    let adapter = ProcessModelAdapter::new(&path, &[]).unwrap();
    std::fs::write(&path, b"#!/bin/sh\nprintf replaced\n").unwrap();
    let error = adapter.invoke(b"payload", Duration::from_secs(5)).unwrap_err();
    assert!(error.to_string().contains("changed since it was fingerprinted"));
}

#[test]
fn a_deleted_executable_is_refused() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("model");
    std::fs::write(&path, b"#!/bin/sh\ncat\n").unwrap();
    let adapter = ProcessModelAdapter::new(&path, &[]).unwrap();
    std::fs::remove_file(&path).unwrap();
    assert!(adapter.invoke(b"payload", Duration::from_secs(5)).is_err());
}

#[test]
fn a_confined_child_cannot_read_outside_the_allowlist() {
    let temp = tempfile::tempdir().unwrap();
    let secret = temp.path().join("secret.txt");
    std::fs::write(&secret, b"unselected secret").unwrap();
    let script = format!(
        "cat '{}' >/dev/null 2>&1 && echo READ-ESCAPED || echo read-denied",
        secret.display()
    );
    let adapter = sh_adapter(&script);
    match adapter.invoke(b"payload", Duration::from_secs(10)) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("read-denied"), "child read outside the allowlist: {stdout}");
            assert!(!stdout.contains("READ-ESCAPED"));
        }
        Err(cause) => {
            // No mechanism on this host: the refusal is the guarantee.
            assert!(!confinement_available(), "unexpected refusal: {cause}");
        }
    }
}

#[test]
fn a_confined_child_cannot_write_outside_the_scratch_directory() {
    let temp = tempfile::tempdir().unwrap();
    let sink = temp.path().join("escaped.txt");
    let script = format!(
        "printf leaked > '{}' 2>/dev/null && echo WRITE-ESCAPED || echo write-denied",
        sink.display()
    );
    let adapter = sh_adapter(&script);
    match adapter.invoke(b"payload", Duration::from_secs(10)) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(
                stdout.contains("write-denied"),
                "child wrote outside the scratch dir: {stdout}"
            );
        }
        Err(cause) => {
            assert!(!confinement_available(), "unexpected refusal: {cause}");
        }
    }
    assert!(!sink.exists(), "the adapter escaped the write boundary");
}

#[test]
fn a_confined_child_receives_the_exact_payload() {
    if !confinement_available() {
        return; // fail-closed hosts are covered by the refusal tests
    }
    let adapter = sh_adapter("cat");
    let output = adapter.invoke(b"the exact payload", Duration::from_secs(10)).unwrap();
    assert_eq!(output.stdout, b"the exact payload");
    assert_eq!(output.exit_code, Some(0));
}

#[test]
fn a_timeout_kills_every_descendant() {
    let adapter = sh_adapter("sleep 30 & sleep 30 & echo started; sleep 30");
    let started = Instant::now();
    let result = adapter.invoke(b"", Duration::from_millis(300));
    assert!(result.is_err(), "a hanging adapter must time out");
    assert!(started.elapsed() < Duration::from_secs(5), "timeout was not honored");
    // The whole process group must be dead, not just the leader.
    std::thread::sleep(Duration::from_millis(200));
    let survivors = std::process::Command::new("pgrep").args(["-f", "sleep 30"]).output().unwrap();
    assert!(
        survivors.stdout.is_empty(),
        "descendants survived the timeout: {}",
        String::from_utf8_lossy(&survivors.stdout)
    );
}

#[test]
fn a_descendant_holding_the_pipes_cannot_extend_the_timeout() {
    let adapter = sh_adapter("sleep 30 & exit 0");
    let started = Instant::now();
    let result = adapter.invoke(b"", Duration::from_millis(300));
    assert!(result.is_err());
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn output_past_the_bound_is_refused() {
    if !confinement_available() {
        return;
    }
    // One byte past MAX_RESPONSE_BYTES (8 MiB).
    let adapter = sh_adapter("head -c 8388609 /dev/zero");
    let error = adapter.invoke(b"", Duration::from_secs(10)).unwrap_err();
    assert!(error.to_string().contains("byte bound"));
}

#[test]
fn a_nonzero_exit_is_refused() {
    if !confinement_available() {
        return;
    }
    let adapter = sh_adapter("exit 3");
    let error = adapter.invoke(b"", Duration::from_secs(10)).unwrap_err();
    assert!(error.to_string().contains("status 3"));
}

// ---------------------------------------------------------------------------
// macOS: sandbox-exec availability
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
#[test]
fn a_missing_sandbox_exec_fails_closed() {
    // The launcher path is fixed; when it is absent the adapter must refuse
    // rather than fall back to an unconfined spawn. On a normal macOS host the
    // binary exists, so this asserts the available path runs confined.
    let adapter = sh_adapter("printf ok");
    let result = adapter.invoke(b"", Duration::from_secs(10));
    if Path::new("/usr/bin/sandbox-exec").is_file() {
        assert!(result.is_ok(), "sandbox-exec present but invocation failed: {result:?}");
    } else {
        let error = result.unwrap_err();
        assert!(error.to_string().contains("sandbox-exec"));
        assert!(error.to_string().contains("unconfined"));
    }
}

// ---------------------------------------------------------------------------
// Linux: Landlock availability and the seccomp network deny
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
#[allow(unsafe_code)] // raw Landlock ABI query mirrors the adapter's probe
#[test]
fn an_unavailable_landlock_fails_closed_with_a_clear_error() {
    // Query the ABI the same way the adapter does. When Landlock is missing
    // (old kernel) or blocked (container seccomp profile), invoke must refuse
    // with a clear message and never run unconfined.
    let abi = unsafe {
        libc::syscall(libc::SYS_landlock_create_ruleset, std::ptr::null::<u8>(), 0_usize, 1_u32)
    };
    let adapter = sh_adapter("printf ok");
    let result = adapter.invoke(b"", Duration::from_secs(10));
    if abi < 1 {
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Landlock"), "clear kernel error required: {error}");
    } else if result.is_err() {
        // Landlock exists but a blocking seccomp profile (e.g. Docker default)
        // prevents restrict_self: still a fail-closed refusal, never a run.
        assert!(!confinement_available());
    }
}

#[test]
fn a_confined_child_cannot_reach_a_listening_socket() {
    use std::net::TcpListener;

    if !confinement_available() {
        return; // fail-closed hosts are covered by the platform refusal tests
    }
    let Some(python) = which_python() else {
        return; // no python3 on this host; the socket-creation deny is covered below
    };
    // Bind a real listener so a successful connect is observable, then prove
    // an unconfined child reaches it and a confined child cannot.
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let connect = format!(
        "{python} -c 'import socket,sys; s=socket.socket(); s.settimeout(3); \
         s.connect((\"127.0.0.1\", {port})); sys.stdout.write(\"CONNECTED\")' 2>/dev/null || true"
    );

    // Control: unconfined, the same script connects.
    let control = std::process::Command::new("/bin/sh").args(["-c", &connect]).output().unwrap();
    assert!(
        String::from_utf8_lossy(&control.stdout).contains("CONNECTED"),
        "the listener is not reachable even unconfined; test is invalid"
    );
    let _ = listener.accept().unwrap();

    // Confined: the connect must fail and the listener must see no connection.
    let adapter = sh_adapter(&connect);
    let output = adapter.invoke(b"", Duration::from_secs(10)).unwrap();
    assert!(
        !String::from_utf8_lossy(&output.stdout).contains("CONNECTED"),
        "the confined child reached the network"
    );
    listener.set_nonblocking(true).unwrap();
    assert!(listener.accept().is_err(), "the confined child opened a connection");
}

#[cfg(target_os = "linux")]
#[test]
fn a_confined_child_cannot_create_a_socket() {
    if !confinement_available() {
        return;
    }
    let Some(python) = which_python() else {
        return;
    };
    // Minimal child: the probe arrives on stdin (argv rejects control
    // characters) and every socket-creation attempt is guarded, so no
    // uncaught exception can print a traceback echoing this source — the
    // old false escape marker — or trigger Ubuntu's apport/apt hook. `-S`
    // skips site-packages. The verdict is an exact stdout match, so stderr
    // startup noise can never fake it.
    let probe = r#"import errno, socket, sys
for attempt in (socket.socket, lambda: socket.socket(socket.AF_INET6), lambda: socket.socket(socket.AF_UNIX), socket.socketpair):
    try:
        attempt()
    except OSError as e:
        if e.errno != errno.EPERM:
            sys.stdout.write("WRONG-ERRNO-%d" % e.errno)
            sys.exit(0)
    else:
        sys.stdout.write("NET-ESCAPED")
        sys.exit(0)
sys.stdout.write("SOCKET-DENIED")"#;
    let adapter = sh_adapter(&format!("{python} -S -"));
    let output = adapter.invoke(probe.as_bytes(), Duration::from_secs(10)).unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout, "SOCKET-DENIED",
        "socket creation was not denied with EPERM; stderr: {}",
        output.stderr
    );
}

/// Locate a python3 interpreter, or None when the host has none.
fn which_python() -> Option<String> {
    for candidate in ["python3", "/usr/bin/python3"] {
        if std::process::Command::new(candidate)
            .arg("--version")
            .output()
            .is_ok_and(|o| o.status.success())
        {
            return Some(candidate.to_string());
        }
    }
    None
}
