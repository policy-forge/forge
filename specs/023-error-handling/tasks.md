# Tasks: Error Handling & Robustness (WI-23)

**Input**: Design documents from `/specs/023-error-handling/`
**Prerequisites**: plan.md (required), data-model.md, contracts/, research.md, quickstart.md
**PRD**: [023-prd-error-handling](../../docs/PRD/023-prd-error-handling.md)

**Organization**: Tasks grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Add new dependencies required for error handling hardening

- [x] T001 Add `anyhow` (latest stable) and `tracing-subscriber` (latest stable, features=["env-filter"]) to `[dependencies]` in Cargo.toml

---

## Phase 2: Foundational (Error Types & Exit Codes)

**Purpose**: Expand ForgeError enum and add exit code mapping — BLOCKS all user stories

**CRITICAL**: No user story work can begin until this phase is complete

- [x] T002 Add 5 new ForgeError variants (FileNotFound, PermissionDenied, EmptyInput, BinaryFile, NoStructureDetected) to src/error.rs per contracts/error.rs contract
- [x] T003 Add `exit_code(&ForgeError) -> u8` function to src/error.rs per contracts/exit_codes.rs contract
- [x] T004 [P] Write unit tests for all 5 new ForgeError variant Display implementations in src/error.rs (verify user-facing messages match contracts/error.rs)
- [x] T005 [P] Write unit tests for `exit_code()` covering all 17 ForgeError variants (exit codes 1, 2, 3) in src/error.rs

**Checkpoint**: `cargo test` passes. New error types and exit code mapping are available for all stories.

---

## Phase 3: User Story 1 — Missing or Unreadable Input File (Priority: P1) MVP

**Goal**: When input file is missing or unreadable, produce a clear error with the file path and a non-zero exit code.

**Independent Test**: `forge convert nonexistent.md --strategy catalog --format json` prints descriptive error with file path and exits non-zero.

**PRD Requirements**: M-1, M-2, M-6, S-1

### Tests for User Story 1

> Write these tests FIRST, ensure they FAIL before implementation

- [x] T006 [P] [US1] Write unit test in src/ingest/mod.rs: `ingest_file("nonexistent.md")` returns `ForgeError::FileNotFound` with path
- [x] T007 [P] [US1] Write unit test in src/ingest/mod.rs: `ingest_file(unreadable_path)` returns `ForgeError::PermissionDenied` with path

### Implementation for User Story 1

- [x] T008 [US1] Disaggregate IoErrors in `ingest_file()` in src/ingest/mod.rs — replace `std::fs::metadata(path)?` with explicit `map_err` matching `ErrorKind::NotFound` to `FileNotFound` and `ErrorKind::PermissionDenied` to `PermissionDenied` (per research R3)
- [x] T009 [US1] Apply same IoError disaggregation pattern to `std::fs::read(path)?` call in src/ingest/mod.rs
- [x] T010 [US1] Update any existing tests that expect `ForgeError::Io` for file-not-found scenarios to expect `ForgeError::FileNotFound` instead

**Checkpoint**: Tests T006, T007 pass. `ingest_file()` returns typed FileNotFound/PermissionDenied errors with paths.

---

## Phase 4: User Story 2 — Malformed or Unstructured Input (Priority: P1)

**Goal**: When input is empty, binary, or has no detectable structure, produce a clear error describing the problem and expected format.

**Independent Test**: `forge convert empty.md --strategy catalog --format json` with a zero-byte file prints "file is empty" error and exits non-zero.

**PRD Requirements**: M-2, M-7, M-8, S-2

### Tests for User Story 2

> Write these tests FIRST, ensure they FAIL before implementation

- [x] T011 [P] [US2] Write unit tests for `is_binary_content()` in src/ingest/mod.rs: PNG magic bytes, JPEG magic bytes, PDF magic bytes, ZIP magic bytes, ELF magic bytes, null byte ratio >10%, clean text returns false, empty bytes returns false
- [x] T012 [P] [US2] Write unit test in src/ingest/mod.rs: `ingest_file(empty_file)` returns `ForgeError::EmptyInput` with path
- [x] T013 [P] [US2] Write unit test in src/ingest/mod.rs: `ingest_file(binary_file)` returns `ForgeError::BinaryFile` with path

### Implementation for User Story 2

- [x] T014 [US2] Add `is_binary_content(bytes: &[u8]) -> bool` helper function to src/ingest/mod.rs per contracts/input_validation.rs (magic bytes + null byte ratio heuristic)
- [x] T015 [US2] Add empty file check in `ingest_file()` in src/ingest/mod.rs: after reading bytes, if `bytes.is_empty()` return `EmptyInput { path }` (per research R5)
- [x] T016 [US2] Add binary content check in `ingest_file()` in src/ingest/mod.rs: after empty check, before UTF-8 conversion, call `is_binary_content(&bytes)` and return `BinaryFile { path }` (per research R4)
- [x] T017 [US2] Remove or replace `Validation(String)` empty-file check (pipeline.rs ~line 63-67) since empty detection now happens in ingest stage (per research R5)
- [x] T018 [US2] Replace `tracing::warn!` no-sections check (pipeline.rs ~line 73-75) with `NoStructureDetected { path }` error when both sections AND clauses are empty (per research R5)
- [x] T019 [US2] Write integration tests for empty file and no-structure-detected pipeline error paths

**Checkpoint**: Tests T011-T013 pass. Binary, empty, and no-structure inputs produce typed errors.

---

## Phase 5: User Story 3 — Validation Errors Are Comprehensive (Priority: P1)

**Goal**: The `forge validate` stub returns a proper error instead of silently succeeding.

**Independent Test**: `forge validate` exits with non-zero status and descriptive message.

**PRD Requirements**: M-9 (partial — full implementation deferred to WI-19/WI-20 per research R8)

- [x] T020 [US3] Write test verifying `forge validate` returns an error (not Ok) in src/cli/validate.rs
- [x] T021 [US3] Replace `println!("not yet implemented"); Ok(())` with `Err(ForgeError::Validation("validate command not yet implemented — coming in a future release".into()))` in src/cli/validate.rs
- [x] T022 [US3] Update any tests that depend on validate returning Ok

**Checkpoint**: `forge validate` exits non-zero with descriptive message.

---

## Phase 6: User Story 5 — Non-Zero Exit Codes for All Errors (Priority: P1)

**Goal**: Wire exit code mapping in `main.rs` so all errors produce distinct non-zero exit codes (1=input/IO, 2=parse/structure, 3=validation).

**Independent Test**: `forge convert nonexistent.md` exits with code 1; a parse error exits with code 2; a validation error exits with code 3.

**PRD Requirements**: M-1, S-4

### Tests for User Story 5

- [x] T023 [P] [US5] Write integration tests in tests/cli_integration.rs verifying exit code 1 for file-not-found, exit code 2 for no-structure, exit code 3 for validation errors

### Implementation for User Story 5

- [x] T024 [US5] Add `tracing_subscriber::fmt()` initialization in src/main.rs wired to `--verbose` (debug level) and `--quiet` (error level), default warn level (per research R10)
- [x] T025 [US5] Replace `process::exit(1)` (or current error handling) in src/main.rs with `ExitCode::from(exit_code(&err))` pattern, printing `"Error: {err}"` to stderr (per research R2, R6)

**Checkpoint**: All error categories produce correct distinct exit codes. Tracing responds to --verbose/--quiet.

---

## Phase 7: User Story 4 — No Panics on Any Input (Priority: P1)

**Goal**: Audit all `.unwrap()`/`.expect()` calls and create an adversarial input test suite proving no panics on any input.

**Independent Test**: Run `cargo test` with adversarial test suite — zero panics on empty, binary, null byte, whitespace, and no-newline inputs.

**PRD Requirements**: M-3, M-4, M-10

### .unwrap()/.expect() Audit

- [x] T026 [P] [US4] Add `// SAFETY: static regex — panics only if regex literal is invalid` comments to `.expect()` calls at src/citation.rs:32, 37, 42, 48
- [x] T027 [P] [US4] Add `// SAFETY: static regex — panics only if regex literal is invalid` comments to `.expect()` calls at src/parse/atomize.rs:24, 29
- [x] T028 [P] [US4] Add `// SAFETY: guarded by !list_type_stack.is_empty() check on previous line` comment to `.unwrap()` at src/parse/clauses.rs:254
- [x] T029 [US4] Verify zero unreviewed `.unwrap()` or `.expect()` in production code (src/ excluding #[cfg(test)]) — run `grep -rn '\.unwrap()' src/ --include='*.rs'` and `grep -rn '\.expect(' src/ --include='*.rs'`

### Adversarial Test Suite

- [x] T030 [US4] Create adversarial test fixture files in tests/fixtures/adversarial/: empty.md (zero bytes), binary.bin (PNG header bytes), null_bytes.md (filled with \x00), whitespace_only.md (spaces/tabs/newlines only), no_newlines.md (single long line with no newline characters)
- [x] T031 [US4] Write adversarial input integration test suite in tests/adversarial_input_test.rs — for each fixture: assert no panics, non-zero exit code, descriptive error message in stderr
- [x] T032 [US4] Add large file test (>10MB, generated at test runtime) to adversarial suite verifying `ForgeError::FileTooLarge` or graceful handling

**Checkpoint**: `cargo test` passes. Zero panics on all adversarial inputs. All `.unwrap()`/`.expect()` in production code are documented.

---

## Phase 8: Polish & Final Verification

**Purpose**: Cross-cutting quality gates and final validation

- [x] T033 Run `cargo fmt --check` and fix any formatting issues
- [x] T034 Run `cargo clippy -- -D warnings` and fix any warnings
- [x] T035 Run full `cargo test` suite (all existing + new adversarial + exit code tests)
- [x] T036 Audit grep: `grep -rn 'panic!\|todo!\|unimplemented!' src/ --include='*.rs'` excluding #[cfg(test)] — verify zero occurrences
- [x] T037 Review all ForgeError Display implementations for information leakage (SEC-1 through SEC-5): no internal module names, no system paths beyond user-provided input, no Rust type names

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Phase 2 — IoError disaggregation in ingest
- **US2 (Phase 4)**: Depends on Phase 2 — binary/empty/no-structure detection (same file as US1, run sequentially)
- **US3 (Phase 5)**: Depends on Phase 2 — validate stub fix
- **US5 (Phase 6)**: Depends on Phase 2 + US1 + US2 — needs all error variants wired before exit code integration tests
- **US4 (Phase 7)**: Depends on US1 + US2 + US5 — adversarial tests verify full pipeline
- **Polish (Phase 8)**: Depends on all user stories complete

### User Story Dependencies

- **US1 (Phase 3)**: After Foundational. No dependency on other stories.
- **US2 (Phase 4)**: After Foundational. Touches same file as US1 (src/ingest/mod.rs) — run sequentially after US1.
- **US3 (Phase 5)**: After Foundational. Independent of US1/US2 — can run in parallel with US1/US2 if different agent.
- **US5 (Phase 6)**: After US1 + US2 — exit code integration tests need error variants active in pipeline.
- **US4 (Phase 7)**: After US1 + US2 + US5 — adversarial tests validate full pipeline end-to-end.

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Implementation follows contract specifications exactly
- Run `cargo test` after each task to verify incremental progress

### Parallel Opportunities

- T004 + T005 (foundational tests) can run in parallel
- T006 + T007 (US1 tests) can run in parallel
- T011 + T012 + T013 (US2 tests) can run in parallel
- T026 + T027 + T028 (SAFETY comments) can run in parallel
- US3 (Phase 5) can run in parallel with US1 + US2 if on different agent (different files)

---

## Parallel Example: Phase 2 (Foundational)

```text
# Sequential: variants then function
Task T002: Add ForgeError variants to src/error.rs
Task T003: Add exit_code() function to src/error.rs (depends on T002)

# Parallel: tests (after T002+T003)
Task T004: Unit tests for variant Display impls
Task T005: Unit tests for exit_code() mapping
```

## Parallel Example: Phase 7 (SAFETY Comments)

```text
# All touch different files — run in parallel
Task T026: SAFETY comments in src/citation.rs
Task T027: SAFETY comments in src/parse/atomize.rs
Task T028: SAFETY comment in src/parse/clauses.rs
```

---

## Implementation Strategy

### MVP First (US1 + US2)

1. Complete Phase 1: Setup (Cargo.toml)
2. Complete Phase 2: Foundational (error types + exit_code)
3. Complete Phase 3: US1 (file access errors)
4. Complete Phase 4: US2 (malformed input errors)
5. **STOP and VALIDATE**: `cargo test` — core error handling is functional

### Full Delivery

6. Complete Phase 5: US3 (validate stub)
7. Complete Phase 6: US5 (exit code wiring in main.rs)
8. Complete Phase 7: US4 (audit + adversarial tests)
9. Complete Phase 8: Polish (clippy, fmt, final verification)

### Parallel Team Strategy

With multiple agents:
1. All agents complete Setup + Foundational together
2. Once Foundational is done:
   - Agent A: US1 (Phase 3) then US2 (Phase 4) — same file (src/ingest/mod.rs)
   - Agent B: US3 (Phase 5) — independent file (src/cli/validate.rs)
3. After US1+US2+US3 complete:
   - Agent A: US5 (Phase 6) — src/main.rs + tests
   - Agent B: US4 SAFETY comments (T026-T029) — different src/ files
4. After US5 complete:
   - Agent A or B: US4 adversarial tests (T030-T032)
5. Final: Phase 8 Polish (any agent)

---

## Task Summary

| Phase | Story | Task Count | Parallel Tasks |
|-------|-------|-----------|----------------|
| Phase 1: Setup | — | 1 | 0 |
| Phase 2: Foundational | — | 4 | 2 (T004, T005) |
| Phase 3: US1 | US1 | 5 | 2 (T006, T007) |
| Phase 4: US2 | US2 | 9 | 3 (T011, T012, T013) |
| Phase 5: US3 | US3 | 3 | 0 |
| Phase 6: US5 | US5 | 3 | 1 (T023) |
| Phase 7: US4 | US4 | 7 | 3 (T026, T027, T028) |
| Phase 8: Polish | — | 5 | 0 |
| **Total** | | **37** | **11** |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific PRD user story
- All 5 PRD user stories are Priority P1 — ordering is by dependency, not priority
- Verify tests fail (RED) before implementing (GREEN)
- Commit after each phase or logical group
- `cargo test` after every task to catch regressions early
