# Tasks: oscal-cli Profile Resolution Integration

**Input**: Design documents from `/specs/036-oscal-cli-profile-resolution/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/oscal_cli.rs
**Tests**: Included (Constitution principle IV: Test-First Development mandatory)

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Module skeleton and project structure

- [X] T001 Create `src/oscal_cli/mod.rs` with module declarations for `detector` and `invoker` submodules, plus re-exports of public types
- [X] T002 Create empty `src/oscal_cli/detector.rs` with module-level doc comment
- [X] T003 Create empty `src/oscal_cli/invoker.rs` with module-level doc comment
- [X] T004 Create empty `src/cli/resolve.rs` with module-level doc comment
- [X] T005 Register `pub mod oscal_cli;` in `src/lib.rs`
- [X] T006 Register `pub mod resolve;` in `src/cli/mod.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Error variants, data structs, and trait definitions that ALL user stories depend on

**CRITICAL**: No user story work can begin until this phase is complete

### Tests for Foundational

- [X] T007 [P] Write tests for new ForgeError display messages (OscalCliNotFound, OscalCliNotFunctional, OscalCliExecution, OscalCliTimeout, ResolveInputNotJson) in `src/error.rs` — verify each variant produces the expected human-readable message using stable substring assertions
- [X] T008 [P] Write tests for new exit code mappings in `src/error.rs` — verify OscalCliNotFound and OscalCliNotFunctional return exit code 4, OscalCliExecution and OscalCliTimeout return exit code 1, ResolveInputNotJson returns exit code 1

### Implementation for Foundational

- [X] T009 Add `OscalCliNotFound` error variant to `ForgeError` enum in `src/error.rs` with message: "oscal-cli not found on system PATH. Install from: https://github.com/usnistgov/oscal-cli"
- [X] T010 Add `OscalCliNotFunctional { path: PathBuf, detail: String }` error variant to `ForgeError` in `src/error.rs` with message: "oscal-cli found at '{path}' but is not functional: {detail}"
- [X] T011 Add `OscalCliExecution { exit_code: Option<i32>, message: String, stderr: String }` error variant to `ForgeError` in `src/error.rs`
- [X] T012 Add `OscalCliTimeout { timeout: std::time::Duration }` error variant to `ForgeError` in `src/error.rs`
- [X] T013 Add `ResolveInputNotJson { path: PathBuf }` error variant to `ForgeError` in `src/error.rs`
- [X] T014 Update `exit_code()` function in `src/error.rs` — map OscalCliNotFound and OscalCliNotFunctional to exit code 4, map OscalCliExecution, OscalCliTimeout, and ResolveInputNotJson to exit code 1
- [X] T015 Define `OscalCliInfo` struct in `src/oscal_cli/mod.rs` with fields: `available: bool`, `functional: bool`, `version: Option<String>`, `executable_path: Option<PathBuf>` — derive Debug, Clone
- [X] T016 Define `ResolveArgs` struct in `src/oscal_cli/mod.rs` with fields: `profile_path: PathBuf`, `output_path: PathBuf`, `timeout: Duration` — derive Debug
- [X] T017 Define `ResolveResult` struct in `src/oscal_cli/mod.rs` with fields: `output_path: PathBuf`, `warnings: Vec<String>` — derive Debug
- [X] T018 Define `OscalCliDetect` trait in `src/oscal_cli/mod.rs` with method `fn detect(&self) -> OscalCliInfo`
- [X] T019 Define `OscalCliInvoke` trait in `src/oscal_cli/mod.rs` with method `fn resolve_profile(&self, args: &ResolveArgs) -> Result<ResolveResult, ForgeError>`
- [X] T020 Verify T007 and T008 tests pass with the new error variants and exit code mappings

**Checkpoint**: Foundation ready — data types, traits, and error handling infrastructure in place. All user story work can now begin.

---

## Phase 3: User Story 1 — Resolve a Profile via oscal-cli (Priority: P1) MVP

**Goal**: A user can run `forge resolve profile.json` and get a resolved Catalog written to a file via oscal-cli delegation.

**Independent Test**: Generate a Profile with `forge profile`, then run `forge resolve profile.json` and verify a resolved Catalog JSON file is produced.

### Tests for User Story 1

- [X] T021 [P] [US1] Write unit tests for `PathDetector::detect()` in `src/oscal_cli/detector.rs` — test case: oscal-cli found and `--version` succeeds → OscalCliInfo { available: true, functional: true, version: Some("..."), executable_path: Some(...) }
- [X] T022 [P] [US1] Write unit tests for `ProcessInvoker::resolve_profile()` success path in `src/oscal_cli/invoker.rs` — test case: mock subprocess exits 0, output file exists → ResolveResult with correct output_path
- [X] T023 [P] [US1] Write unit test for CLI arg parsing of `forge resolve profile.json` in `src/cli/mod.rs` — verify Resolve variant parsed with correct input path and default options
- [X] T024 [P] [US1] Write unit test for CLI arg parsing of `forge resolve profile.json --output resolved.json --timeout 30 --oscal-cli-path /usr/local/bin/oscal-cli` in `src/cli/mod.rs` — verify all options parsed correctly
- [X] T025 [P] [US1] Write unit test for default output path derivation in `src/cli/resolve.rs` — verify `my-profile.json` → `my-profile-resolved.json` in same directory
- [X] T026 [US1] Write unit test for input file validation in `src/cli/resolve.rs` — test case: input file does not exist → ForgeError::FileNotFound; input file is not .json → ForgeError::ResolveInputNotJson

### Implementation for User Story 1

- [X] T027 [US1] Implement cross-platform PATH search helper in `src/oscal_cli/detector.rs` — split PATH env var by platform separator, check each directory for oscal-cli binary (append `EXE_SUFFIX` on Windows), return first match as absolute path
- [X] T028 [US1] Implement `PathDetector` struct and `OscalCliDetect` for `PathDetector` in `src/oscal_cli/detector.rs` — detect(): search PATH (or use explicit override path), run `oscal-cli --version` via `Command`, parse version from stdout, return `OscalCliInfo`
- [X] T029 [US1] Implement `ProcessInvoker` struct in `src/oscal_cli/invoker.rs` with field `executable_path: PathBuf`
- [X] T030 [US1] Implement `OscalCliInvoke` for `ProcessInvoker` in `src/oscal_cli/invoker.rs` — build `Command::new(executable_path).args(["profile", "resolve", "-to=json", input_path, output_path])`, apply `env_clear()` + allowlist (PATH, HOME, JAVA_HOME, TMPDIR; + USERPROFILE, SYSTEMROOT, TEMP, TMP on Windows), spawn and wait for completion, return ResolveResult on success
- [X] T031 [US1] Implement thread-based timeout watchdog in `src/oscal_cli/invoker.rs` — spawn watchdog thread that calls `child.kill()` after timeout duration, join watchdog after child exits or is killed
- [X] T032 [US1] Add `Resolve` variant to `Commands` enum in `src/cli/mod.rs` with fields: `input: PathBuf`, `output: Option<PathBuf>`, `check: bool` (default false), `timeout: u64` (default 60), `oscal_cli_path: Option<PathBuf>`
- [X] T033 [US1] Wire `Resolve` variant dispatch in `cli::execute()` in `src/cli/mod.rs` — call `resolve::execute()` with parsed args
- [X] T034 [US1] Implement `resolve::execute()` in `src/cli/resolve.rs` — validate input file exists and has .json extension (FR-007), canonicalize input path (FR-014), derive default output path if --output not provided (`<stem>-resolved.json`), detect oscal-cli (using --oscal-cli-path if provided), log detected binary path at INFO (SEC-6), invoke resolve_profile, print success message with output path. Ensure all tracing calls log file paths only, never file contents (SEC-1).
- [X] T035 [US1] Verify all US1 tests pass — run `cargo test` and confirm T021–T026 are green. Verify INFO log output contains the detected binary path (SEC-6).

**Checkpoint**: Core happy path works — `forge resolve profile.json` produces a resolved Catalog via oscal-cli.

---

## Phase 4: User Story 2 — Graceful Degradation Without oscal-cli (Priority: P1)

**Goal**: When oscal-cli is not installed, `forge resolve` warns the user with installation guidance and exits gracefully (exit code 4). Other FORGE commands are unaffected.

**Independent Test**: Remove oscal-cli from PATH, run `forge resolve profile.json`, verify descriptive warning with installation link.

### Tests for User Story 2

- [X] T036 [P] [US2] Write unit test for `PathDetector::detect()` not-found path in `src/oscal_cli/detector.rs` — test case: oscal-cli not on PATH → OscalCliInfo { available: false, functional: false, version: None, executable_path: None }
- [X] T037 [P] [US2] Write unit test for resolve handler graceful degradation in `src/cli/resolve.rs` — test case: detector returns unavailable → ForgeError::OscalCliNotFound with installation guidance message
- [X] T038 [P] [US2] Write unit test verifying other FORGE commands don't check oscal-cli in `src/cli/mod.rs` — parse `forge convert test.md --strategy catalog` and verify no oscal-cli-related behavior

### Implementation for User Story 2

- [X] T039 [US2] Ensure `resolve::execute()` in `src/cli/resolve.rs` checks `OscalCliInfo.available` before invocation — if not available, return `ForgeError::OscalCliNotFound` (which includes installation URL in its Display impl)
- [X] T040 [US2] Ensure `resolve::execute()` checks `OscalCliInfo.functional` — if available but not functional, return `ForgeError::OscalCliNotFunctional` with detail from version check failure (e.g., "Java may be missing")
- [X] T041 [US2] Verify all US2 tests pass — run `cargo test` and confirm T036–T038 are green

**Checkpoint**: Graceful degradation works — oscal-cli absence produces clear warning, other commands unaffected.

---

## Phase 5: User Story 3 — Handle oscal-cli Execution Errors (Priority: P1)

**Goal**: When oscal-cli fails (invalid input, non-zero exit), FORGE displays a clear, actionable error message with relevant oscal-cli error detail (not raw stack traces).

**Independent Test**: Provide an invalid Profile JSON to `forge resolve`, verify FORGE shows clear error with relevant oscal-cli detail.

### Tests for User Story 3

- [X] T042 [P] [US3] Write unit test for `ProcessInvoker` non-zero exit handling in `src/oscal_cli/invoker.rs` — test case: child exits with code 1 and stderr content → ForgeError::OscalCliExecution with exit_code, parsed message, and full stderr
- [X] T043 [P] [US3] Write unit test for stderr parsing helper in `src/oscal_cli/invoker.rs` — test case: multi-line stderr with Java stack trace → extracted last meaningful non-stack-trace line as message
- [X] T044 [P] [US3] Write unit test for timeout handling in `src/oscal_cli/invoker.rs` — test case: process exceeds timeout → ForgeError::OscalCliTimeout with timeout duration
- [X] T045 [P] [US3] Write unit test for stderr warnings with exit 0 in `src/oscal_cli/invoker.rs` — test case: exit 0 but stderr non-empty → ResolveResult with warnings populated

### Implementation for User Story 3

- [X] T046 [US3] Implement `extract_error_message()` helper in `src/oscal_cli/invoker.rs` — parse stderr to extract last meaningful non-empty line (skip lines starting with "at " or "\t at " which indicate Java stack traces), return as primary error message
- [X] T047 [US3] Wire error extraction into `ProcessInvoker::resolve_profile()` in `src/oscal_cli/invoker.rs` — on non-zero exit: extract message from stderr, construct ForgeError::OscalCliExecution with exit_code, message, and full stderr
- [X] T048 [US3] Wire timeout error into `ProcessInvoker::resolve_profile()` in `src/oscal_cli/invoker.rs` — on timeout: return ForgeError::OscalCliTimeout with timeout duration
- [X] T049 [US3] Wire stderr warnings into success path in `src/oscal_cli/invoker.rs` — on exit 0 with non-empty stderr: populate ResolveResult.warnings and forward them to user via `tracing::warn!` in resolve handler
- [X] T050 [US3] Verify all US3 tests pass — run `cargo test` and confirm T042–T045 are green

**Checkpoint**: Error handling complete — all oscal-cli failure modes produce actionable FORGE error messages.

---

## Phase 6: User Story 4 — Diagnostic oscal-cli Check (Priority: P2)

**Goal**: `forge resolve --check` reports oscal-cli detection status, version, and path without performing resolution.

**Independent Test**: Run `forge resolve --check`, verify it prints version and path (or "not found" with installation guidance).

### Tests for User Story 4

- [X] T051 [P] [US4] Write unit test for `--check` with oscal-cli available in `src/cli/resolve.rs` — test case: detector returns available + version + path → output contains version string and path
- [X] T052 [P] [US4] Write unit test for `--check` with oscal-cli missing in `src/cli/resolve.rs` — test case: detector returns unavailable → output contains "not found" and installation URL
- [X] T053 [P] [US4] Write unit test for CLI arg parsing of `forge resolve --check` in `src/cli/mod.rs` — verify check flag is true, input is not required when --check is set

### Implementation for User Story 4

- [X] T054 [US4] Implement `--check` branch in `resolve::execute()` in `src/cli/resolve.rs` — if `--check` flag: detect oscal-cli, print status (available/not, version, path), include installation guidance if not found, return Ok(()) without requiring input file argument
- [X] T055 [US4] Update `Resolve` variant in `src/cli/mod.rs` to make `input` optional when `--check` is provided — use conditional required logic or separate subcommand group
- [X] T056 [US4] Verify all US4 tests pass — run `cargo test` and confirm T051–T053 are green

**Checkpoint**: Diagnostic --check flag works — users can verify oscal-cli integration status.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Quality gates, integration tests, and cross-platform validation

- [X] T057 [P] Run `cargo fmt --check` and fix any formatting violations
- [X] T058 [P] Run `cargo clippy -- -D warnings` and fix any linter warnings
- [X] T059 Write integration test (conditional on oscal-cli availability) in `src/oscal_cli/invoker.rs` or `tests/` — happy path: resolve a valid Profile fixture → verify output file contains resolved Catalog JSON
- [X] T060 Write integration test for `forge resolve --check` (conditional on oscal-cli availability) — verify version and path output
- [X] T061 Verify all tests pass with `cargo test` — confirm >80% coverage for `oscal_cli` module (SC-005)
- [X] T062 Run quickstart.md validation — verify build sequence and all verification commands succeed

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 — BLOCKS all user stories
- **Phase 3 (US1)**: Depends on Phase 2 — core happy path
- **Phase 4 (US2)**: Depends on Phase 3 (PathDetector implementation T027-T028, resolve handler T034) — can run in parallel with US4
- **Phase 5 (US3)**: Depends on Phase 3 (ProcessInvoker must exist) — error handling on top of invoker
- **Phase 6 (US4)**: Depends on Phase 3 (PathDetector implementation T027-T028) — can run in parallel with US2
- **Phase 7 (Polish)**: Depends on all user stories complete

### User Story Dependencies

```
Phase 1 (Setup)
    ↓
Phase 2 (Foundational)
    ↓
    ├── Phase 3 (US1: Resolve Profile) ── MVP
    │       ↓
    │   ├── Phase 5 (US3: Error Handling)
    │   │
    │   ├── Phase 4 (US2: Graceful Degradation)
    │   │
    │   └── Phase 6 (US4: Diagnostic Check)
            ↓
        Phase 7 (Polish)
```

### Within Each User Story

- Tests MUST be written and FAIL before implementation (TDD)
- Data structs before logic
- Detection before invocation
- Core implementation before integration
- Story complete before moving to next priority

### Parallel Opportunities

- T002, T003, T004 can run in parallel (empty files, different paths)
- T007, T008 can run in parallel (different test functions)
- T009–T013 can run in parallel (different error variants, same file — but sequential edits safer)
- T021–T026 can run in parallel (different test functions)
- T036–T038 can run in parallel (different test functions)
- T042–T045 can run in parallel (different test functions)
- T051–T053 can run in parallel (different test functions)
- US2 (Phase 4) and US4 (Phase 6) can run in parallel after Phase 3
- T057, T058 can run in parallel (different tools)

---

## Parallel Example: User Story 1

```bash
# Launch all US1 tests in parallel:
Task: "T021 - Unit test PathDetector success in src/oscal_cli/detector.rs"
Task: "T022 - Unit test ProcessInvoker success in src/oscal_cli/invoker.rs"
Task: "T023 - Unit test CLI arg parsing in src/cli/mod.rs"
Task: "T024 - Unit test CLI arg parsing with all options in src/cli/mod.rs"
Task: "T025 - Unit test default output path in src/cli/resolve.rs"
Task: "T026 - Unit test input validation in src/cli/resolve.rs"

# Then implement sequentially:
# T027 → T028 (detector) → T029 → T030 → T031 (invoker) → T032 → T033 → T034 (CLI)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T006)
2. Complete Phase 2: Foundational (T007–T020)
3. Complete Phase 3: User Story 1 (T021–T035)
4. **STOP and VALIDATE**: Run `forge resolve profile.json` with oscal-cli installed
5. Core resolution works — ship or continue

### Incremental Delivery

1. Setup + Foundational → types and traits ready
2. US1 (Resolve Profile) → core happy path works → **MVP**
3. US2 (Graceful Degradation) → absence handled cleanly
4. US3 (Error Handling) → all failure modes covered
5. US4 (Diagnostic Check) → troubleshooting capability
6. Polish → quality gates, integration tests

### Requirement Coverage

| Requirement | Task(s) | User Story |
|-------------|---------|------------|
| M-1 | T027, T028, T036 | US1, US2 |
| M-2 | T029, T030 | US1 |
| M-3 | T030, T034 | US1 |
| M-4 | T039, T040 | US2 |
| M-5 | T046, T047 | US3 |
| M-6 | T032, T033 | US1 |
| M-7 | T034 (validation) | US1 |
| S-1 | T028 (version) | US1 |
| S-2 | T054, T055 | US4 |
| S-3 | T031, T048 | US1, US3 |
| S-4 | T009 (error msg) | US2 |
| FR-012 | T030 (arg arrays) | US1 |
| FR-013 | T030 (env_clear) | US1 |
| FR-014 | T034 (canonicalize) | US1 |
| SEC-1 | T034 (log paths only) | US1 |
| SEC-4 | T030 (no shell) | US1 |
| SEC-5 | T031, T048 | US1, US3 |
| SEC-6 | T034 (log path) | US1 |
| SEC-7 | T030 (env filter) | US1 |
| SEC-8 | T028 (abs path) | US1 |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Constitution principle IV mandates TDD — all test tasks must run RED before implementation
- oscal-cli subcommand is `profile resolve` (not `resolve-profile`) per research.md
- No new crate dependencies required — stdlib `std::process::Command` only
- Exit code 4 is new for "external dependency unavailable" — distinct from existing 1/2/3
