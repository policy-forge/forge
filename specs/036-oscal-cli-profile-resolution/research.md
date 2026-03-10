# Research: oscal-cli Profile Resolution Integration

**Date**: 2026-03-10
**Feature**: 036-oscal-cli-profile-resolution

## R-1: oscal-cli resolve-profile CLI Interface

**Decision**: The actual subcommand is `oscal-cli profile resolve` (not `resolve-profile`).

**Interface Contract**:
```
oscal-cli profile resolve [-to=<format>] <input-file> <output-file>
```

- **Input**: First positional argument — file path to OSCAL Profile (JSON, XML, or YAML; auto-detected from extension)
- **Output**: Second positional argument — file path where resolved Catalog is written (required)
- **`-to` flag**: Output serialization format (e.g., `-to=json`, `-to=xml`). If omitted, auto-detected from output file extension.
- **Version check**: `oscal-cli --version`
- **Exit codes**: 0 for success, non-zero for errors (standard conventions; no documented specific codes)
- **Stderr**: Error messages to stderr; may include Java stack traces on failures

**Rationale**: Confirmed via NIST oscal-cli GitHub repository (usnistgov/oscal-cli) and issue #269.

**Alternatives considered**: None — NIST oscal-cli is the authoritative tool per Parent PRD W-3.

**Impact on spec**: FR-002 updated to reference `oscal-cli profile resolve`. The output file is a required positional arg for oscal-cli (not stdout). FORGE must always generate a temp output path or the user-specified `--output` path and pass it as the second positional argument.

## R-2: Cross-Platform PATH Detection

**Decision**: Use manual PATH search via `std::env::var("PATH")` + `std::path::Path::exists()` checks, avoiding the `which` crate.

**Rationale**: The detection logic is simple (split PATH by separator, check for `oscal-cli` or `oscal-cli.exe`). Adding a crate for this violates Constitution principle XI (Dependency Policy). The `--oscal-cli-path` flag provides an explicit override for edge cases.

**Alternatives considered**:
- `which` crate (MIT) — rejected: unnecessary dependency for ~20 lines of code
- `Command::new("oscal-cli").arg("--version")` without PATH search — rejected: doesn't provide the executable path for logging (SEC-6)

**Platform considerations**:
- Unix: Split PATH on `:`, check for `oscal-cli` (no extension)
- Windows: Split PATH on `;`, check for `oscal-cli.exe`, `oscal-cli.bat`, `oscal-cli.cmd`
- Use `std::env::consts::EXE_SUFFIX` for platform-appropriate suffix

## R-3: Timeout Implementation

**Decision**: Use polling with `Child::try_wait()` in a loop that checks elapsed time and calls `Child::kill()` on timeout.

**Rationale**: `std::process::Command` has no built-in timeout. A polling approach avoids the complexity of spawning a watchdog thread and is straightforward for a synchronous CLI tool. The 100ms poll interval adds negligible overhead for the expected subprocess durations (seconds).

**Pattern**:
```rust
// Spawn child
// Loop: try_wait() → if exited, break; if elapsed > timeout, kill + return error; else sleep 100ms
// Read stderr after exit
```

**Alternatives considered**:
- `tokio::time::timeout` — rejected: adds async runtime dependency for one operation
- Thread-based watchdog — considered but polling is simpler for this use case; a watchdog thread would be warranted if concurrent stderr draining is needed (see future improvement notes)

## R-4: Environment Variable Filtering

**Decision**: Use `Command::env_clear()` + explicit allowlist: `PATH`, `HOME`, `JAVA_HOME`, `TMPDIR`.

**Rationale**: Clarification session confirmed this approach. Minimizes leakage of sensitive env vars (API keys, tokens) to the child process. The allowlist covers oscal-cli's known requirements (Java runtime via PATH/JAVA_HOME, temp files via TMPDIR, home directory via HOME).

**Platform additions**:
- Windows: Also include `USERPROFILE`, `SYSTEMROOT`, `TEMP`, `TMP`
- All: `JAVA_OPTS` may be needed if oscal-cli requires JVM flags — add only if testing reveals need

## R-5: Error Message Parsing Strategy

**Decision**: Capture full stderr from oscal-cli. Extract the last meaningful non-empty line (skipping stack trace lines) as the primary error message. Include full stderr in debug log.

**Rationale**: oscal-cli is Java-based and may produce verbose stack traces. Users need the root cause, not `at org.nist.oscal.cli.Main.run(Main.java:42)`. The last meaningful line typically contains the actual error description.

**Alternatives considered**:
- Regex-based extraction of specific error patterns — rejected: fragile, couples to oscal-cli output format
- Passing full stderr to user — rejected: overwhelms users with Java internals
- Only showing exit code — rejected: not actionable enough (PRD M-5)
