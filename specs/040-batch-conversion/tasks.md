# Tasks: Batch Conversion

**Input**: Design documents from `/specs/040-batch-conversion/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/batch.rs, PRD, AR, SEC

**Tests**: TDD mandatory per constitution (Principle IV). Test tasks included for each phase.

**Organization**: Tasks grouped by user story. US1+US2 are P1 (MVP), US3+US4 are P2.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story (US1, US2, US3, US4)
- Exact file paths included

---

## Phase 1: Setup

**Purpose**: Add rayon dependency, create batch module skeleton, register in lib.rs

- [x] T001 Add `rayon = "1"` to `[dependencies]` in `Cargo.toml`
- [x] T002 Create `src/batch/mod.rs` with submodule declarations (`pub mod summary; pub mod output_naming; pub mod formatter; pub mod orchestrator;`) and re-exports
- [x] T003 Register `pub mod batch;` in `src/lib.rs` and add public re-exports for `FileResult`, `BatchSummary`
- [x] T004 Add `BatchConversion(String)` variant to `ForgeError` enum in `src/error.rs` with `#[error("Batch conversion error: {0}")]`, map to exit code 1 in `exit_code()`

**Checkpoint**: `cargo build` succeeds with empty batch submodules. `cargo test` passes (no regressions).

---

## Phase 2: Foundational (Data Structures & Pure Functions)

**Purpose**: Implement `FileResult`, `BatchSummary`, output naming, and status formatting — all pure functions with no rayon dependency yet.

**⚠️ CRITICAL**: All user story implementation depends on these structures being complete.

### Tests

- [x] T005 [P] Write unit tests for `FileResult::success()` and `FileResult::failure()` constructor invariants in `src/batch/summary.rs` (success ⇒ output_path Some + error None; failure ⇒ output_path None + error Some)
- [x] T006 [P] Write unit tests for `BatchSummary::from_results()` in `src/batch/summary.rs` (counts correct, sorted by input filename, `has_failures()` logic)
- [x] T007 [P] Write unit tests for `derive_output_paths()` in `src/batch/output_naming.rs`: (a) single file → `{stem}.json`, (b) with output_dir → placed in dir, (c) collision → `_2` suffix, (d) multiple collisions → `_2`, `_3`, (e) different formats (xml, yaml extensions)
- [x] T008 [P] Write unit tests for `format_batch_summary()` in `src/batch/formatter.rs`: (a) all success, (b) all failure, (c) mixed, (d) single file summary

### Implementation

- [x] T009 [P] Implement `FileResult` struct with `success()` and `failure()` constructors in `src/batch/summary.rs` per contracts/batch.rs
- [x] T010 [P] Implement `BatchSummary` struct with `from_results()` and `has_failures()` in `src/batch/summary.rs` per contracts/batch.rs
- [x] T011 [P] Implement `derive_output_paths()` in `src/batch/output_naming.rs` with collision avoidance algorithm (numeric `_n` suffix starting at 2) per research.md R3
- [x] T012 [P] Implement `format_batch_summary()` in `src/batch/formatter.rs` producing human-readable status with ✓/✗ marks, per-file lines sorted by filename, summary header with counts and duration per research.md R5

**Checkpoint**: `cargo test` passes for all batch data structures and pure functions. No rayon usage yet.

---

## Phase 3: User Story 1 — Convert Multiple Policy Documents (Priority: P1) 🎯 MVP

**Goal**: Accept multiple input files in `forge convert` and produce independent OSCAL output for each.

**Independent Test**: `forge convert policy1.md policy2.md policy3.md --strategy catalog --format json --output output/` produces three files.

**Traces to**: PRD M-1, M-2, M-3, FR-001, FR-002, FR-003, FR-009, FR-011, FR-012, FR-013, FR-014, FR-015; SEC-2, SEC-3, SEC-4, SEC-5

### Tests

- [x] T013 [US1] Write unit test in `src/cli/mod.rs` for parsing multiple positional input files (`forge convert a.md b.md --strategy catalog`) — verify `Vec<PathBuf>` contains both
- [x] T014 [P] [US1] Write unit test in `src/batch/orchestrator.rs` for `validate_inputs()`: (a) all valid files → Ok, (b) one missing → Err listing invalid path, (c) zero files → Ok (caller owns emptiness check)
- [x] T015 [P] [US1] Write integration test in `tests/batch_conversion_test.rs` for batch of 3 valid Markdown files → 3 output files in `--output` directory with correct names (AC-1, AC-2)
- [x] T016 [P] [US1] Write integration test in `tests/batch_conversion_test.rs` for single-file backward compatibility — single input behaves identically to existing (EC-1, FR-011)
- [x] T017 [P] [US1] Write integration test in `tests/batch_conversion_test.rs` for filename collision — two files with same stem from different dirs → `policy.json` and `policy_2.json` (EC-3, FR-012)
- [x] T018 [P] [US1] Write integration test in `tests/batch_conversion_test.rs` for `--output` is a file (not dir) with multiple inputs → error (EC-4, FR-014)
- [x] T019 [P] [US1] Write integration test in `tests/batch_conversion_test.rs` for zero input files (empty glob) → descriptive error (EC-2, SEC-5)
- [x] T020 [P] [US1] Write integration test in `tests/batch_conversion_test.rs` for `--output` dir does not exist → auto-created (EC-6, FR-015)
- [x] T021 [P] [US1] Write integration test in `tests/batch_conversion_test.rs` for batch without `--output` → files written to current directory with auto-generated names (FR-009)

### Implementation

- [x] T022 [US1] Modify `Commands::Convert` in `src/cli/mod.rs`: change `input: PathBuf` to `input: Vec<PathBuf>` with `#[arg(num_args = 1..)]`
- [x] T023 [US1] Update all existing CLI parse tests in `src/cli/mod.rs` to work with `Vec<PathBuf>` (unpack first element where single file was expected)
- [x] T024 [US1] Implement `validate_inputs()` in `src/batch/orchestrator.rs` — check each path exists and is readable; return Err with all invalid paths listed (fail-fast, SEC-2)
- [x] T025 [US1] Implement batch dispatch in `src/cli/convert.rs`: when `input.len() == 1` delegate to existing `execute()` unchanged; when `input.len() > 1` enter batch mode
- [x] T026 [US1] Implement batch mode in `src/cli/convert.rs`: validate `--output` is dir or absent (FR-014), create dir if needed (FR-015), call `validate_inputs()`, call `derive_output_paths()`, call `run_batch_conversion()` (sequential for now — rayon in US3), print summary to stderr
- [x] T027 [US1] Implement sequential `run_batch_conversion()` in `src/batch/orchestrator.rs` — iterate files, call existing pipeline per file via `convert::execute()` or direct pipeline call, collect `FileResult`s, build `BatchSummary` (parallel version deferred to US3)
- [x] T028 [US1] Update `cli::execute()` match arm in `src/cli/mod.rs` to pass `Vec<PathBuf>` to `convert::execute()` or the new batch dispatch entry point

**Checkpoint**: `cargo test` passes. Batch conversion of multiple files works sequentially. Single-file backward compatibility verified. Edge cases (collision, zero files, --output validation) pass.

---

## Phase 4: User Story 2 — Aggregated Status Output (Priority: P1)

**Goal**: Print aggregated per-file success/failure summary to stderr after batch completes. Non-zero exit on any failure.

**Independent Test**: Batch with 1 invalid file → stderr shows 2 successes, 1 failure with error message; exit code non-zero.

**Traces to**: PRD M-4, M-5, M-6, FR-004, FR-005, FR-006, FR-010; SEC-6, SEC-7

### Tests

- [x] T029 [P] [US2] Write integration test in `tests/batch_conversion_test.rs` for mixed batch (2 valid + 1 invalid Markdown file) → aggregated status shows 2 successes, 1 failure with error message on stderr (AC-3, AC-4, FR-004, FR-005)
- [x] T030 [P] [US2] Write integration test in `tests/batch_conversion_test.rs` for all-success batch → aggregated status on stderr shows all succeeded with total count and elapsed time (FR-010)
- [x] T031 [P] [US2] Write integration test in `tests/batch_conversion_test.rs` for batch with any failure → non-zero exit code (AC-5, FR-006)
- [x] T032 [P] [US2] Write integration test in `tests/batch_conversion_test.rs` for all-failure batch → aggregated status shows all failures, exit code non-zero (EC-5)
- [x] T033 [P] [US2] Write integration test in `tests/batch_conversion_test.rs` verifying aggregated status is on stderr (not stdout) and OSCAL output is only in files (SEC-7)

### Implementation

- [x] T034 [US2] Implement per-file error isolation in `src/batch/orchestrator.rs` — wrap each pipeline invocation with `catch_unwind(AssertUnwindSafe(...))` per research.md R2; capture panics as `FileResult::failure` with "Internal error (panic during conversion)" message (SEC-6, FR-005)
- [x] T035 [US2] Wire aggregated status output: after `run_batch_conversion()` returns, call `format_batch_summary()` and `eprintln!()` to stderr in `src/cli/convert.rs` (FR-004, SEC-7)
- [x] T036 [US2] Implement exit code logic in `src/cli/convert.rs`: if `BatchSummary::has_failures()` return non-zero exit (FR-006). Determine appropriate exit code (1 for batch-level, using `ForgeError::BatchConversion`)

**Checkpoint**: `cargo test` passes. Mixed valid/invalid batches show correct status. Exit codes are correct. Stderr output verified.

---

## Phase 5: User Story 3 — Parallel Processing for Performance (Priority: P2)

**Goal**: Process files in parallel via rayon; `--jobs` flag controls parallelism.

**Independent Test**: Batch of 10 files completes in ≤50% of sequential time (2x speedup).

**Traces to**: PRD S-1, S-2, FR-007, FR-008; SEC-8

### Tests

- [x] T037 [P] [US3] Write unit test in `src/cli/mod.rs` for `--jobs` flag parsing: (a) `--jobs 4` → 4, (b) `--jobs 1` → 1, (c) `--jobs 0` → rejected by clap, (d) `--jobs 257` → rejected by clap, (e) default (no flag) → 0 meaning auto (FR-008, SEC-8)
- [x] T038 [P] [US3] Write integration test in `tests/batch_conversion_test.rs` for `--jobs 1` → sequential processing (no parallelism) (US3-AC3)
- [x] T039 [US3] Write benchmark or integration test in `tests/batch_conversion_test.rs` that converts 10 files with default parallelism and asserts wall-clock time < 50% of sequential baseline (SC-004, 2x speedup target)
- [x] T040 [P] [US3] Write integration test in `tests/batch_conversion_test.rs` for error isolation under parallelism — one file panics/fails while others succeed (US3-AC2)

### Implementation

- [x] T041 [US3] Add `--jobs` flag to `Commands::Convert` in `src/cli/mod.rs` with `#[arg(long, default_value = "0", value_parser = clap::value_parser!(u16).range(0..=256))]` (FR-008, SEC-8, clarification: max 256)
- [x] T042 [US3] Replace sequential iteration with `rayon::ThreadPoolBuilder::new().num_threads(jobs).build()` + `pool.install(|| input_paths.par_iter()...)` in `src/batch/orchestrator.rs` per research.md R1
- [x] T043 [US3] Pass `jobs` parameter from CLI through batch dispatch in `src/cli/convert.rs` to `run_batch_conversion()` in `src/batch/orchestrator.rs`
- [x] T044 [US3] Add `tracing::info!` logging when entering batch mode: file count, parallelism level (jobs value) in `src/batch/orchestrator.rs` per AR observability requirements

**Checkpoint**: `cargo test` passes. Parallel speedup verified. `--jobs 1` forces sequential. `--jobs` validation works.

---

## Phase 6: User Story 4 — Glob Pattern Input (Priority: P2)

**Goal**: Shell glob expansion transparently provides multiple files. FORGE needs no special glob handling — just robust behavior with shell-expanded paths.

**Independent Test**: `forge convert policies/*.md --strategy catalog --format json --output output/` where `policies/` has 5 files → 5 outputs.

**Traces to**: PRD M-1 (via shell expansion); FR-016

### Tests

- [x] T045 [P] [US4] Write integration test in `tests/batch_conversion_test.rs` simulating glob expansion by passing 5 files from a temp directory → 5 output files (AC-7)
- [x] T046 [P] [US4] Write integration test in `tests/batch_conversion_test.rs` for >100 file warning: create 101 temp files, run batch, verify warning on stderr (FR-016)

### Implementation

- [x] T047 [US4] Implement >100 file warning in `src/batch/orchestrator.rs`: if `input_paths.len() > 100`, emit `tracing::warn!("Large batch: {} files. Processing may take a while.", count)` (FR-016)

**Note**: Symlink handling requires no implementation task — per spec clarification, FORGE follows symlinks silently (standard CLI behavior). SEC R3 is accepted risk.

**Checkpoint**: `cargo test` passes. Glob-expanded paths work correctly. Large batch warning fires at >100 files.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Quality gates, edge case hardening, documentation

- [x] T048 Run `cargo clippy -- -D warnings` and fix any warnings across all new and modified files
- [x] T049 Run `cargo fmt --check` and fix any formatting violations across all new and modified files
- [x] T050 [P] Add `BatchConversion` variant to `exit_code()` tests in `src/error.rs`
- [x] T051 [P] Run full test suite (`cargo test`) and verify zero regressions in existing tests
- [x] T052 Verify quickstart.md validation: run the manual verification commands from `specs/040-batch-conversion/quickstart.md`
- [x] T053 [P] Verify `src/lib.rs` public exports include `batch::FileResult`, `batch::BatchSummary`, `batch::format_batch_summary` for library consumers (verification of T003; add any exports missed during Phase 1)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 — BLOCKS all user stories
- **Phase 3 (US1)**: Depends on Phase 2
- **Phase 4 (US2)**: Depends on Phase 3 (needs batch mode working to add status output)
- **Phase 5 (US3)**: Depends on Phase 3 (needs sequential batch mode to upgrade to parallel)
- **Phase 6 (US4)**: Depends on Phase 3 (needs batch mode working; can run in parallel with US2/US3)
- **Phase 7 (Polish)**: Depends on all user story phases

### User Story Dependencies

- **US1 (P1)**: Foundational → Core batch functionality (MVP)
- **US2 (P1)**: US1 → Adds error isolation and status reporting on top of batch mode
- **US3 (P2)**: US1 → Replaces sequential loop with rayon parallel (can run in parallel with US2)
- **US4 (P2)**: US1 → Adds >100 file warning (can run in parallel with US2, US3)

### Within Each User Story

- Tests MUST be written and FAIL before implementation (TDD)
- Data structures before logic
- Pure functions before orchestration
- Core implementation before edge cases

### Parallel Opportunities

**Phase 2**: T005, T006, T007, T008 (test tasks) all in parallel; T009, T010, T011, T012 (impl tasks) all in parallel

**Phase 3**: T014–T021 (test tasks) all in parallel; T022 must be first (CLI change), then T023–T028

**Phase 4**: T029–T033 (test tasks) all in parallel; T034–T036 sequential

**Phase 5**: T037, T038, T040 in parallel; T039 after T042 (needs parallel to benchmark)

**Phase 6**: T045, T046 in parallel

**Cross-story parallelism**: After US1 is done, US2, US3, and US4 can all proceed simultaneously with different agents.

---

## Implementation Strategy

### MVP First (US1 + US2 = Phase 3 + Phase 4)

1. Complete Phase 1: Setup (T001–T004)
2. Complete Phase 2: Foundational (T005–T012)
3. Complete Phase 3: US1 — multi-file batch conversion (T013–T028)
4. Complete Phase 4: US2 — aggregated status + error isolation (T029–T036)
5. **STOP and VALIDATE**: Test batch conversion with mixed valid/invalid files
6. MVP is complete — batch conversion works sequentially with full status reporting

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. US1 → Multi-file conversion works → Functional MVP
3. US2 → Status reporting + error isolation → Production-ready MVP
4. US3 → Parallel processing → Performance enhancement
5. US4 → Large batch warning → Polish
6. Phase 7 → Quality gates → Release-ready

---

## Requirement Traceability

| Requirement | Task(s) | Status |
|-------------|---------|--------|
| FR-001 (M-1) | T022, T028 | Phase 3 |
| FR-002 (M-2) | T027 | Phase 3 |
| FR-003 (M-3) | T011, T026 | Phase 2+3 |
| FR-004 (M-4) | T035 | Phase 4 |
| FR-005 (M-5) | T034 | Phase 4 |
| FR-006 (M-6) | T036 | Phase 4 |
| FR-007 (S-1) | T042 | Phase 5 |
| FR-008 (S-2) | T041 | Phase 5 |
| FR-009 (S-3) | T026 | Phase 3 |
| FR-010 (S-4) | T012, T035 | Phase 2+4 |
| FR-011 | T025 | Phase 3 |
| FR-012 | T011 | Phase 2 |
| FR-013 | T024 | Phase 3 |
| FR-014 | T026 | Phase 3 |
| FR-015 | T026 | Phase 3 |
| FR-016 | T047 | Phase 6 |
| SEC-1 | T034 | Phase 4 |
| SEC-2 | T024 | Phase 3 |
| SEC-3 | T026 | Phase 3 |
| SEC-4 | T011 | Phase 2 |
| SEC-5 | T019 | Phase 3 |
| SEC-6 | T034 | Phase 4 |
| SEC-7 | T035 | Phase 4 |
| SEC-8 | T041 | Phase 5 |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story
- Pipeline modules (`src/pipeline.rs`, `src/parse/`, `src/oscal/`, etc.) must not be modified
- All new code in `src/batch/` module and modifications to `src/cli/`, `src/error.rs`, `src/lib.rs`
- `catch_unwind` with `AssertUnwindSafe` is required for panic isolation (SEC-6)
- Aggregated status to stderr only (SEC-7) — never stdout
