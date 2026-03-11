# Implementation Plan: oscal-cli Profile Resolution Integration

**Branch**: `036-oscal-cli-profile-resolution` | **Date**: 2026-03-10 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/036-oscal-cli-profile-resolution/spec.md`

## Summary

Integrate NIST oscal-cli as an external subprocess to resolve OSCAL Profiles into flat Catalog baselines. Adds a `forge resolve` subcommand that detects oscal-cli on PATH, invokes `oscal-cli profile resolve` with proper argument arrays and environment filtering, handles errors and timeouts, and degrades gracefully when oscal-cli is unavailable. Uses `std::process::Command` with trait-based abstractions for testability. No new crate dependencies required.

## Technical Context

**Language/Version**: Rust, Edition 2024, stable 1.93.0
**Primary Dependencies**: clap 4.x (derive), thiserror 2.0.18, tracing 0.1.44 — all existing in Cargo.toml
**New Dependencies**: None — stdlib `std::process::Command` for subprocess management
**Storage**: Local filesystem (reads Profile JSON, writes resolved Catalog JSON)
**Testing**: cargo test (unit with mocks, integration conditional on oscal-cli availability)
**Target Platform**: Linux, macOS, Windows (cross-platform PATH detection and process invocation)
**Project Type**: Single Rust crate (existing structure)
**Performance Goals**: <60s for typical profile resolution (dominated by oscal-cli execution time)
**Constraints**: Configurable timeout (default 60s); no network; no async runtime
**Scale/Scope**: Single subprocess invocation per `forge resolve` call

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Crate-First Architecture | PASS | New `oscal_cli` module within existing crate; no new crate |
| II. Rust-First Implementation | PASS | Pure stable Rust; no `unsafe`; no FFI |
| III. Contract-First Development | PASS | Trait interfaces defined in contracts/oscal_cli.rs before implementation |
| IV. Test-First Development | PASS | Mock traits enable TDD; unit tests before integration tests |
| V. Complete Requirement Delivery | PASS | All Must-Have (M-1 through M-7) mapped to components; acceptance criteria testable |
| VI. Performance and Scope Discipline | PASS | 60s timeout is measurable; no speculative benchmarks |
| VII. Security-First Design | PASS | SEC requirements (SEC-2 through SEC-10) mapped to implementation; env filtering, arg arrays, path canonicalization |
| VIII. Error Handling Standards | PASS | All error variants produce actionable messages; tested via stable substrings |
| IX. Observability and Debuggability | PASS | Binary path logged at INFO; invocation args at DEBUG; stderr at WARN |
| X. Simplicity and Pragmatism | PASS | stdlib only; no new dependencies; extends existing CLI and error patterns |
| XI. Dependency Policy | PASS | No new dependencies; `which` crate rejected in favor of manual PATH search |

**Post-Phase 1 Re-check**: All gates still pass. Trait abstraction is the minimum complexity for testability (justified by Constitution IV + external dependency risk D-9).

## Project Structure

### Documentation (this feature)

```text
specs/036-oscal-cli-profile-resolution/
├── plan.md              # This file
├── spec.md              # Feature specification (with clarifications)
├── research.md          # Phase 0: oscal-cli interface research
├── data-model.md        # Phase 1: entities, traits, error variants
├── quickstart.md        # Phase 1: build sequence and verification
├── contracts/           # Phase 1: trait and struct contracts
│   └── oscal_cli.rs     # OscalCliDetect, OscalCliInvoke, data structs
└── tasks.md             # Phase 2 output (created by /speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── oscal_cli/           # NEW: oscal-cli integration module
│   ├── mod.rs           # Module declarations, re-exports, data structs
│   ├── detector.rs      # OscalCliDetect trait + PathDetector
│   └── invoker.rs       # OscalCliInvoke trait + ProcessInvoker
├── cli/
│   ├── mod.rs           # MODIFY: add Resolve variant + resolve dispatch
│   └── resolve.rs       # NEW: forge resolve subcommand handler
├── error.rs             # MODIFY: add OscalCli* error variants + exit codes
└── lib.rs               # MODIFY: add `pub mod oscal_cli;`
```

**Structure Decision**: Single project structure (existing). New `oscal_cli` module follows the same pattern as existing top-level modules (`validate/`, `export/`, `parse/`). CLI handler follows `cli/profile.rs` pattern.

## Complexity Tracking

No constitution violations. No complexity justifications needed.

## Implementation Phases

### Phase 1: Error Variants + Data Structs

**Files**: `src/error.rs`, `src/oscal_cli/mod.rs`

1. Add ForgeError variants: `OscalCliNotFound`, `OscalCliNotFunctional`, `OscalCliExecution`, `OscalCliTimeout`, `ResolveInputNotJson`
2. Add exit code mappings (exit 4 for dependency-unavailable, exit 1 for execution errors)
3. Define `OscalCliInfo`, `ResolveArgs`, `ResolveResult` structs
4. Define `OscalCliDetect` and `OscalCliInvoke` traits
5. Register module in `src/lib.rs`

**Tests**: Error display messages, exit code mappings

### Phase 2: Detector (PathDetector)

**Files**: `src/oscal_cli/detector.rs`

1. Implement `PathDetector` with cross-platform PATH search
2. Support `--oscal-cli-path` override (skip PATH search, use provided path)
3. Run `oscal-cli --version` to verify functionality and capture version
4. Parse version string from stdout
5. Return `OscalCliInfo` with available/functional/version/path

**Tests** (unit, with mock filesystem):
- oscal-cli found and functional → OscalCliInfo { available: true, functional: true, version: Some(...) }
- oscal-cli not on PATH → OscalCliInfo { available: false, functional: false }
- oscal-cli found but --version fails → OscalCliInfo { available: true, functional: false }
- Explicit --oscal-cli-path override
- Platform-specific executable suffix handling

### Phase 3: Invoker (ProcessInvoker)

**Files**: `src/oscal_cli/invoker.rs`

1. Build `Command` with argument array: `[oscal-cli-path, "profile", "resolve", "-to=json", input-path, output-path]`
2. Apply `env_clear()` + allowlist (PATH, HOME, JAVA_HOME, TMPDIR; + USERPROFILE, SYSTEMROOT, TEMP, TMP on Windows)
3. Spawn child process
4. Implement thread-based timeout watchdog
5. On success (exit 0): return `ResolveResult` with output path + any stderr warnings
6. On failure: parse stderr, extract meaningful error, return `ForgeError::OscalCliExecution`
7. On timeout: kill child, return `ForgeError::OscalCliTimeout`

**Tests** (unit, with MockInvoker):
- Successful resolution → ResolveResult
- Non-zero exit code → OscalCliExecution error with parsed message
- Timeout → OscalCliTimeout error
- stderr warnings with exit 0 → ResolveResult with warnings populated
- Environment filtering (verify env_clear + allowlist applied)

### Phase 4: CLI Subcommand (forge resolve)

**Files**: `src/cli/resolve.rs`, `src/cli/mod.rs`

1. Add `Resolve` variant to `Commands` enum with: `input: PathBuf`, `--output: Option<PathBuf>`, `--check: bool`, `--timeout: u64`, `--oscal-cli-path: Option<PathBuf>`
2. Implement `resolve::execute()` handler:
   a. Validate input file exists and has `.json` extension (FR-007)
   b. Canonicalize input path (FR-014)
   c. Derive default output path if `--output` not provided (`<stem>-resolved.json`)
   d. If `--check`: detect and report status, then return
   e. Detect oscal-cli (using `--oscal-cli-path` override if provided)
   f. If not available/functional: return appropriate error with installation guidance
   g. Log detected binary path at INFO (SEC-6)
   h. Invoke `resolve_profile` with args
   i. On success: print output path; forward warnings to stderr
   j. On failure: return ForgeError
3. Wire into `cli::execute()` dispatch

**Tests**:
- Parse `forge resolve profile.json` args
- Parse `forge resolve --check` args
- Parse `forge resolve profile.json --output out.json --timeout 30 --oscal-cli-path /usr/local/bin/oscal-cli`
- Input file not found → ForgeError::FileNotFound
- Input file not JSON → ForgeError::ResolveInputNotJson
- Default output path derivation
- `--check` with oscal-cli available
- `--check` with oscal-cli missing
- Other FORGE commands unaffected (US-2, scenario 2)

### Phase 5: Integration Tests

**Files**: `tests/resolve_integration.rs` (or inline in modules)

Conditional on oscal-cli availability:
1. Happy path: resolve a valid Profile → verify output file exists and contains resolved Catalog
2. Invalid Profile input → verify descriptive error
3. `forge resolve --check` → verify version and path output
4. Default output path → verify `<stem>-resolved.json` created

Always-run tests (mock-based):
1. Full flow with MockDetector + MockInvoker
2. Graceful degradation path (oscal-cli absent)
3. Error translation from oscal-cli failure
4. Timeout handling

## Requirement Traceability

| Requirement | Phase | Component | Test |
|-------------|-------|-----------|------|
| M-1 (detect oscal-cli) | Phase 2 | PathDetector | Unit: found/not-found/not-functional |
| M-2 (invoke profile resolve) | Phase 3 | ProcessInvoker | Unit: mock; Integration: real oscal-cli |
| M-3 (capture output) | Phase 3+4 | ProcessInvoker + resolve handler | Integration: verify output file |
| M-4 (graceful degradation) | Phase 4 | resolve handler | Unit: mock detector returns unavailable |
| M-5 (error messages) | Phase 3+4 | ProcessInvoker + error variants | Unit: stderr parsing |
| M-6 (forge resolve subcommand) | Phase 4 | cli/resolve.rs + mod.rs | Unit: arg parsing |
| M-7 (validate input) | Phase 4 | resolve handler | Unit: missing file, non-JSON |
| S-1 (version detection) | Phase 2 | PathDetector | Unit: version parsing |
| S-2 (--check flag) | Phase 4 | resolve handler | Unit + Integration |
| S-3 (timeout) | Phase 3 | ProcessInvoker | Unit: timeout scenario |
| S-4 (installation guidance) | Phase 1+4 | error message + handler | Unit: error display |
| FR-012 (arg arrays) | Phase 3 | ProcessInvoker | Code review |
| FR-013 (env filtering) | Phase 3 | ProcessInvoker | Unit: verify env_clear |
| FR-014 (path canonicalization) | Phase 4 | resolve handler | Unit: relative path input |
| SEC-4 (no shell interpolation) | Phase 3 | ProcessInvoker | Code review |
| SEC-5 (timeout enforcement) | Phase 3 | ProcessInvoker | Unit: timeout kills child |
| SEC-6 (log binary path) | Phase 4 | resolve handler | Unit: verify INFO log |
| SEC-7 (env filtering) | Phase 3 | ProcessInvoker | Unit: verify allowlist |
| SEC-8 (absolute path for invocation) | Phase 2+3 | PathDetector + ProcessInvoker | Unit: absolute path used |

## Quality Gates

- `cargo fmt --check` — no formatting violations
- `cargo clippy -- -D warnings` — no clippy warnings
- `cargo test` — all tests pass
- Test coverage >80% for `oscal_cli` module
- All Must-Have requirements have at least one test
