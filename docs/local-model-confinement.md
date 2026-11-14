# Local model confinement boundary

`forge suggest run` can execute a consented local model executable
(`ProcessModelAdapter`) or replay a separately reviewed recorded response
(`--recorded-response`). Process execution is confined by the operating
system on macOS and Linux and **fails closed everywhere else**: if the
confinement mechanism cannot be established, FORGE refuses to run the adapter
rather than running it unconfined. This note records exactly what is confined,
the platform limitations, and how to verify the boundary.

The design was approved Unix-first in the owner decision record of 2026-11-13
(ticket #72 / F16). Windows remains refused pending a separate `windows-sys`
PRD 069 approval; no Windows confinement is implemented or claimed.

## What is confined

On both supported platforms the adapter child:

- runs with a **cleared environment** (`env_clear`);
- is bound to **verified bytes**: the executable is re-hashed immediately
  before spawn and refused if it differs from the consented
  `executable_sha256`;
- leads a **fresh process group**, so a timeout or refusal kills every
  descendant with one `killpg`, not just the leader;
- has **bounded stdout/stderr** (8 MiB / 64 KiB); an over-bound stream is a
  refusal, never a silent truncation;
- must finish within the requested `--timeout`; descendants holding the pipes
  open cannot extend it.

### macOS — Seatbelt via `sandbox-exec`

The child is spawned through `/usr/bin/sandbox-exec` with a deny-default
profile. The profile allows:

- read of the executable, its ancestor directories (path traversal), and the
  system runtime (`/usr/lib`, `/bin`, `/usr/bin`, `/System`, the dyld cache
  locations, `/dev/fd`);
- read/write of `/dev/null`, `/dev/zero`, `/dev/random`, `/dev/urandom`,
  `/dev/tty`, `/dev/dtracehelper`;
- read/write of one private per-invocation scratch directory;
- `process*`, `signal`, `sysctl-read`, `mach*` (needed to launch and run).

Everything else — including **all network access** and any file outside the
allowlist — is denied. The profile is inherited by every descendant. If
`sandbox-exec` is absent the adapter fails closed.

### Linux — Landlock + seccomp-BPF

Before `exec`, the child installs (via `libc`, no new dependency):

- a **Landlock** filesystem ruleset (kernel ≥ 5.13) allowing read/execute of
  the executable, read/execute of the runtime directories and linker cache,
  read/write of the null/zero/random devices, and read/write/create within the
  private scratch directory. All other filesystem access is denied. On a
  kernel without Landlock the adapter refuses with a clear error.
- a **seccomp-BPF** filter that returns `EPERM` for every socket-creating
  syscall (`socket`, the `io_uring_*` family, and `socketcall` on i386),
  kills the process on a foreign ABI or the x32 syscall range, and allows the
  rest. This blocks network egress including the `io_uring` bypass. The filter
  is inherited across `fork`/`exec`.

`no_new_privs` is set first, as both mechanisms require it. The seccomp policy
is defined for x86-64 and aarch64; other Linux architectures fail closed.

## What is NOT confined

The boundary is **filesystem and network only**. CPU time, memory, and process
count are not limited, and the executable runs with the operator's own user
identity inside the sandbox. A malicious model can still consume resources
until the timeout kills its group. Treat the executable as trusted code that
is being prevented from reaching data and networks it was not consented to,
not as a defense against arbitrary hostile computation.

## Platform limitations

- **Windows**: refused (`PROCESS_EXECUTION_UNAVAILABLE`). No mechanism is
  implemented; a separate `windows-sys` approval is deferred.
- **Linux kernels < 5.13** (or Landlock disabled): refused with a clear
  "Landlock is unavailable" error.
- **Linux containers with a blocking seccomp profile** (e.g. the Docker
  default profile, which denies the `landlock_*` syscalls): the adapter fails
  closed. To exercise confinement in such a container, run with
  `--security-opt seccomp=unconfined` or a profile that permits the Landlock
  syscalls; FORGE does not weaken confinement to accommodate the container.
- **macOS without `/usr/bin/sandbox-exec`**: refused.
- **Linux architectures other than x86-64/aarch64**: refused (no approved
  seccomp policy).

## How to verify

The integration suite `tests/suggest_confinement_test.rs` exercises the
boundary on each platform (gated by `cfg`):

```bash
cargo test --test suggest_confinement_test
```

It asserts: a replaced or deleted executable is refused before spawn; a
confined child cannot read or write outside the allowlist; a confined child
cannot create a socket or reach a listening socket (with an unconfined control
proving the listener is reachable); the exact payload is delivered; a timeout
kills every descendant; a pipe-holding descendant cannot extend the timeout;
and over-bound output and non-zero exits are refused. On hosts where the
mechanism is unavailable the tests assert the fail-closed refusal instead.

To verify manually on Linux in Docker:

```bash
docker run --rm --security-opt seccomp=unconfined \
  -v "$PWD":/w -w /w rust:1.90-bookworm \
  cargo test --test suggest_confinement_test
```

Recorded-response mode (`--recorded-response`) never spawns a process and is
unaffected by any of the above.