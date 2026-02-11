# Tasks: Markdown Ingestion

**Input**: Design documents from `/specs/002-markdown-ingestion/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/

**Tests**: Included — constitution mandates TDD (Principle IV) and plan specifies test-first approach.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add new dependencies required for ingestion feature

- [x] T001 Add `serde` (with `derive` feature), `serde_json`, and `sha2` dependencies at their latest stable versions to `Cargo.toml` — run `cargo build` to verify resolution and update `Cargo.lock`, then run `cargo audit` and `cargo deny check` to verify no advisories or license violations per constitution Principle XI

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Define types and error variants that ALL user stories depend on

**CRITICAL**: No user story work can begin until this phase is complete

- [x] T002 [P] Add four ingestion-specific error variants to `ForgeError` in `src/error.rs`: `UnsupportedFormat { extension: String }` with message suggesting pandoc/markitdown, `FileTooLarge { path: PathBuf, size_bytes: u64, limit_bytes: u64 }` with MB-formatted message and --max-size hint, `InvalidEncoding { path: PathBuf }` explaining UTF-8 requirement, and `NotAFile { path: PathBuf }` — add unit tests for each variant's Display output verifying the error messages match data-model.md
- [x] T003 [P] Define `SourceLine` struct (fields: `number: usize`, `text: String`; derives: `Debug`, `Serialize`, `PartialEq`) and `IngestedDocument` struct (fields: `source_path: PathBuf`, `fingerprint: String`, `lines: Vec<SourceLine>`; derives: `Debug`, `Serialize`) in `src/ingest/mod.rs` — add `pub fn ingest_file(path: &Path, max_size_bytes: u64) -> Result<IngestedDocument, ForgeError>` as a stub returning `todo!()` — add `use serde::Serialize` and necessary imports

**Checkpoint**: Types compile, error variants display correctly, function stub exists — user story implementation can now begin

---

## Phase 3: User Story 1 — Read a Markdown Policy File (Priority: P1) MVP

**Goal**: A compliance engineer provides a Markdown file and receives a JSON object on stdout containing `source_path`, `fingerprint` (SHA-256), and `lines` (1-based numbered)

**Independent Test**: Run `forge convert policy.md` with a valid Markdown file and verify JSON output contains all three fields with correct values

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL (panic from todo!()) before implementation**

- [x] T004 [US1] Write unit tests for `ingest_file()` happy path in `src/ingest/mod.rs` `#[cfg(test)] mod tests` block: (1) valid `.md` file returns `IngestedDocument` with correct `source_path` matching the input path, (2) `fingerprint` is a 64-char lowercase hex SHA-256 hash matching independently computed expected value, (3) `lines` vec has correct count matching file line count, (4) each `SourceLine` has 1-based `number` and `text` matching original line content without trailing newline, (5) empty (0-byte) `.md` file returns `IngestedDocument` with empty `lines` vec and the SHA-256 hash of empty string (`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`), (6) same file content always produces same fingerprint, (7) symlink to a valid `.md` file is followed and returns correct IngestedDocument (use `std::os::unix::fs::symlink` with `#[cfg(unix)]`), (8) file path containing spaces (e.g., `"my policy.md"`) is handled correctly — use `tempfile` or create test fixtures in a temp directory

### Implementation for User Story 1

- [x] T005 [US1] Implement `ingest_file()` core logic in `src/ingest/mod.rs`: read file as raw bytes via `std::fs::read(path)?`, compute SHA-256 hex digest using `sha2::Sha256` + `sha2::Digest` trait (format as lowercase hex), parse bytes as UTF-8 via `String::from_utf8(bytes)` (map error to `ForgeError::Parse` for now), split content into `SourceLine` vec using `.lines().enumerate()` with 1-based numbering, construct `IngestedDocument` with canonicalized `source_path` — verify all T004 unit tests pass
- [x] T006 [US1] Implement `convert::execute()` in `src/cli/convert.rs`: call `ingest_file(input, 10 * 1024 * 1024)` with default 10MB limit, serialize result to JSON via `serde_json::to_string_pretty()`, print JSON to stdout — update function to use the `input` parameter (remove underscore prefix)
- [x] T007 [US1] Write integration test in `tests/cli_integration.rs`: run `forge convert <valid.md>` with a temp file containing known Markdown content, assert exit code 0, assert stdout is valid JSON parseable by `serde_json::Value`, assert JSON contains `source_path` (string), `fingerprint` (64-char hex string), and `lines` (array with correct count)

**Checkpoint**: `forge convert policy.md` outputs valid JSON with source_path, fingerprint, and lines — User Story 1 is independently functional and testable

---

## Phase 4: User Story 2 — Reject Unsupported File Formats (Priority: P1)

**Goal**: Non-Markdown files (.pdf, .docx, no extension) are rejected with a descriptive error suggesting conversion tools; uppercase extensions (.MD, .MARKDOWN) are accepted

**Independent Test**: Run `forge convert policy.pdf` and verify non-zero exit code with error message suggesting pandoc or markitdown

### Tests for User Story 2

- [x] T008 [US2] Write unit tests for extension validation in `src/ingest/mod.rs`: (1) `.pdf` file returns `ForgeError::UnsupportedFormat` with extension "pdf", (2) `.docx` file returns `UnsupportedFormat`, (3) file with no extension returns `UnsupportedFormat` with empty extension, (4) `.MD` (uppercase) file is accepted (not rejected), (5) `.Markdown` (mixed case) file is accepted, (6) `.MARKDOWN` (uppercase) file is accepted, (7) `.markdown` (lowercase) file is accepted — use temp files with appropriate extensions

### Implementation for User Story 2

- [x] T009 [US2] Implement case-insensitive extension validation at the start of `ingest_file()` in `src/ingest/mod.rs`: extract extension via `path.extension().and_then(|e| e.to_str())`, compare case-insensitively against `["md", "markdown"]`, return `ForgeError::UnsupportedFormat { extension }` for non-matches or missing extension — this guard must execute before any file I/O
- [x] T010 [US2] Write integration test in `tests/cli_integration.rs`: run `forge convert policy.pdf` (create temp file with .pdf extension), assert non-zero exit code, assert stderr contains "Unsupported file format" and mentions "pandoc" or "markitdown" as conversion suggestions

**Checkpoint**: Unsupported formats are rejected with actionable suggestions; valid Markdown extensions (case-insensitive) still work

---

## Phase 5: User Story 3 — Handle File Access Errors Gracefully (Priority: P1)

**Goal**: File-not-found, permission-denied, non-UTF-8 encoding, and directory paths all produce descriptive, actionable error messages with non-zero exit codes

**Independent Test**: Run `forge convert nonexistent.md` and verify descriptive "file not found" error with non-zero exit code

### Tests for User Story 3

- [x] T011 [US3] Write unit tests for file access error handling in `src/ingest/mod.rs`: (1) non-existent `.md` path returns `ForgeError::Io` with `NotFound` kind, (2) directory path returns `ForgeError::NotAFile`, (3) `.md` file containing non-UTF-8 bytes (e.g., `[0xFF, 0xFE]`) returns `ForgeError::InvalidEncoding`, (4) `.md` file with read permissions removed returns `ForgeError::Io` with `PermissionDenied` kind (skip on Windows via `#[cfg(unix)]`) — use temp dir for directory test and write raw bytes for encoding test

### Implementation for User Story 3

- [x] T012 [US3] Add file access validation to `ingest_file()` in `src/ingest/mod.rs`: after extension check, call `std::fs::metadata(path)?` (propagates NotFound/PermissionDenied as `ForgeError::Io`), check `metadata.is_file()` and return `ForgeError::NotAFile` if false, change UTF-8 conversion from `ForgeError::Parse` to `ForgeError::InvalidEncoding { path }` for the `String::from_utf8()` error case
- [x] T013 [US3] Write integration tests in `tests/cli_integration.rs`: (1) `forge convert nonexistent.md` returns non-zero exit code with stderr containing "not found" or "No such file", (2) `forge convert <directory>` returns non-zero exit code with stderr containing "not a regular file"

**Checkpoint**: All file access error conditions produce descriptive, actionable error messages — combined with US1 and US2, the ingestion pipeline handles all P1 requirements

---

## Phase 6: User Story 4 — Handle Oversized Files Safely (Priority: P2)

**Goal**: Files exceeding 10MB are rejected by default; `--max-size` CLI flag allows users to override the limit

**Independent Test**: Create a file >10MB, verify rejection with descriptive error. Retry with `--max-size 20` and verify acceptance

### Tests for User Story 4

- [x] T014 [US4] Write unit tests for file size validation in `src/ingest/mod.rs`: (1) file exceeding `max_size_bytes` returns `ForgeError::FileTooLarge` with correct `size_bytes` and `limit_bytes`, (2) file exactly at limit is accepted (boundary test), (3) file within custom higher limit is accepted when `max_size_bytes` is increased — use temp files with controlled sizes (write known byte counts)

### Implementation for User Story 4

- [x] T015 [US4] Implement file size check in `ingest_file()` in `src/ingest/mod.rs`: after `metadata()` call and `is_file()` check, compare `metadata.len()` against `max_size_bytes`, return `ForgeError::FileTooLarge { path, size_bytes: metadata.len(), limit_bytes: max_size_bytes }` if exceeded — this must execute before reading file content to prevent memory exhaustion
- [x] T016 [US4] Add `--max-size` CLI argument to `Commands::Convert` in `src/cli/mod.rs`: add `max_size: Option<u64>` field with `#[arg(long, help = "Maximum file size in MB (default: 10)")]`, wire through `cli::execute()` dispatch to `convert::execute()`, update `convert::execute()` in `src/cli/convert.rs` to compute `max_size_bytes = max_size.unwrap_or(10) * 1024 * 1024` and pass to `ingest_file()`
- [x] T017 [US4] Write integration tests in `tests/cli_integration.rs`: (1) `forge convert <oversized.md>` (file >10MB) returns non-zero exit code with stderr containing "exceeding" and "max-size", (2) `forge convert <oversized.md> --max-size 20` succeeds for a file between 10-20MB, (3) verify `--max-size` flag is recognized by clap (parse test)

**Checkpoint**: Oversized files are rejected safely; --max-size flag provides user override — all user stories are complete

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Code quality verification and documentation

- [x] T018 [P] Run `cargo fmt --check` and fix any formatting issues across all modified files
- [x] T019 [P] Run `cargo clippy -- -D warnings` and fix any warnings across all modified files
- [x] T020 Add rustdoc documentation for all public items in `src/ingest/mod.rs`: module-level `//!` doc explaining ingestion purpose, `/// ...` docs for `IngestedDocument`, `SourceLine` (with field descriptions), and `ingest_file()` (with `# Arguments`, `# Errors`, `# Examples` sections per constitution Principle III)
- [x] T021 Run `cargo doc --no-deps` and verify documentation builds without warnings; run full `cargo test` to confirm all unit and integration tests pass

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Foundational — MVP milestone
- **US2 (Phase 4)**: Depends on US1 (adds validation guard to existing `ingest_file()`)
- **US3 (Phase 5)**: Depends on US1 (refines error handling in existing `ingest_file()`)
- **US4 (Phase 6)**: Depends on US1 (adds size guard + CLI arg)
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

- **US1 (P1)**: Can start after Foundational — no dependencies on other stories — **MVP**
- **US2 (P1)**: Depends on US1 — adds extension validation guard before US1's read logic
- **US3 (P1)**: Depends on US1 — refines error handling in US1's read logic; can run in parallel with US2 (different code paths)
- **US4 (P2)**: Depends on US1 — adds size guard + CLI arg; can run in parallel with US2/US3

### Within Each User Story

1. Tests MUST be written and FAIL before implementation
2. Implementation tasks follow logical order (core logic → CLI wiring → integration test)
3. Story is complete when all its tests pass

### Parallel Opportunities

- **Phase 2**: T002 and T003 can run in parallel (different files: `error.rs` vs `ingest/mod.rs`)
- **Phase 4-6**: US2, US3, and US4 can potentially run in parallel after US1 completes (US2 modifies top of function, US3 modifies middle, US4 adds new check + CLI arg — minimal conflict)
- **Phase 7**: T018 and T019 can run in parallel (formatting vs linting)

---

## Parallel Example: User Story 1

```bash
# Phase 2 foundational tasks in parallel (different files):
Task: "T002 — Add error variants in src/error.rs"
Task: "T003 — Define types and function stub in src/ingest/mod.rs"

# After US1 completes, US2/US3/US4 can proceed in parallel:
Task: "T008-T010 — US2: Extension validation"
Task: "T011-T013 — US3: File access errors"
Task: "T014-T017 — US4: Size limit + CLI flag"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (add dependencies)
2. Complete Phase 2: Foundational (define types + error variants)
3. Complete Phase 3: User Story 1 (core ingestion + JSON output)
4. **STOP and VALIDATE**: Run `forge convert policy.md` and verify JSON output
5. This gives a working `forge convert` command that reads Markdown files

### Incremental Delivery

1. Setup + Foundational → Types and errors ready
2. US1 → Core ingestion works → **MVP!** (`forge convert policy.md` → JSON)
3. US2 → Format validation added → Non-Markdown files rejected with suggestions
4. US3 → Error handling refined → All failure modes produce actionable messages
5. US4 → Size safety added → Large files rejected, `--max-size` override available
6. Polish → Code quality verified → Ready for merge

### Single Developer Strategy

Execute phases sequentially in priority order:
1. Phase 1 → Phase 2 → Phase 3 (US1 MVP) → validate
2. Phase 4 (US2) → Phase 5 (US3) → Phase 6 (US4) → validate all
3. Phase 7 (Polish) → final validation → ready for merge

---

## Notes

- [P] tasks = different files, no dependencies — safe to parallelize
- [Story] label maps task to specific user story for traceability
- Each user story should be independently testable after completion
- TDD cycle: Write tests → Verify FAIL → Implement → Verify PASS → Refactor
- Commit after each completed user story phase
- Stop at any checkpoint to validate story independently
- All file paths are relative to repository root
