# Implementation Plan: Markdown Ingestion

**Branch**: `002-markdown-ingestion` | **Date**: 2026-02-11 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/002-markdown-ingestion/spec.md`

## Summary

Implement Markdown file ingestion for FORGE: read a Markdown file from the filesystem, validate format/encoding/size, compute a SHA-256 content fingerprint, track line numbers, and output structured JSON to stdout. This is the entry point to the entire OSCAL conversion pipeline — every downstream feature (structural extraction, clause parsing, OSCAL generation) depends on this capability.

**Technical approach**: Extend the existing `forge convert` CLI command to perform ingestion. Add `IngestedDocument` and `SourceLine` types in the `src/ingest/` module. Add ingestion-specific error variants to `ForgeError`. Use `serde_json` for JSON output and `sha2` for fingerprinting. Follow TDD with contract-first type definitions.

## Technical Context

**Language/Version**: Rust (edition 2024, stable 1.93.0)
**Primary Dependencies**: clap 4, thiserror 2.0.18 + NEW: serde 1.x, serde_json 1.x, sha2 0.10.x
**Storage**: Filesystem (read-only)
**Testing**: `cargo test` (built-in unit + integration tests)
**Target Platform**: Cross-platform CLI (macOS, Linux, Windows)
**Project Type**: Single binary crate with modular structure
**Performance Goals**: < 1 second for typical policy documents (< 1MB); SHA-256 and line splitting are fast for this scale
**Constraints**: Files must be valid UTF-8; default 10MB size limit (overridable via `--max-size`)
**Scale/Scope**: Single file at a time; policy documents typically < 1MB

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Crate-First Architecture | JUSTIFIED DEVIATION | Project is a single crate (from 001-scaffolding). Ingestion logic in `src/ingest/` with clean public API boundary. Extraction to `crates/ingest/` is mechanical when project scale warrants it. Principle X (Simplicity/YAGNI) takes precedence at current scope (~150 LOC). See Complexity Tracking. |
| II. Rust-First | PASS | No FFI, no unsafe code needed. |
| III. Contract-First | PASS | Types (`IngestedDocument`, `SourceLine`), error variants, CLI args, and JSON schema defined before implementation. See `data-model.md` and `contracts/`. |
| IV. Test-First | PASS | TDD cycle: write tests → verify failure → implement → verify pass → refactor. |
| V. Complete Implementation | PASS | All spec requirements (FR-001 through FR-011) covered in task plan. |
| VI. Performance-First | PASS | Simple file I/O + SHA-256. No hot paths requiring benchmarks at this scale. |
| VII. Security-First | PASS | Input validation at boundary: extension check, size limit, UTF-8 validation, regular file check. No unsafe code. No secrets. |
| VIII. Error Handling | PASS | `thiserror` with typed variants: `UnsupportedFormat`, `FileTooLarge`, `InvalidEncoding`, `NotAFile`. Descriptive user messages with actionable suggestions. |
| IX. Observability | PASS (SHOULD deferred) | CLI stdout for output, stderr for errors. Verbose mode already scaffolded. `#[instrument]` tracing deferred to when pipeline has multiple stages — will add in next feature when tracing infrastructure is established. |
| X. Simplicity | PASS | Minimal implementation: 2 structs, 1 public function, 4 error variants. No unnecessary abstractions. |
| XI. Current Dependency Policy | PASS | All new dependencies (`serde`, `serde_json`, `sha2`) at latest stable versions. Pre-addition checks: no RustSec advisories, MIT/Apache-2.0 licensed, well-maintained. |

**Post-Phase 1 re-check**: All gates still pass. No design changes introduced violations.

## Project Structure

### Documentation (this feature)

```text
specs/002-markdown-ingestion/
├── plan.md                              # This file
├── research.md                          # Phase 0: dependency decisions
├── data-model.md                        # Phase 1: entity definitions
├── quickstart.md                        # Phase 1: usage guide
├── contracts/
│   └── ingested-document.schema.json    # Phase 1: JSON output schema
└── tasks.md                             # Phase 2: task breakdown (via /speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── cli/
│   ├── mod.rs          # MODIFY: add --max-size arg to Convert variant
│   ├── convert.rs      # MODIFY: implement ingestion call + JSON output
│   └── validate.rs     # UNCHANGED
├── ingest/
│   └── mod.rs          # MODIFY: IngestedDocument, SourceLine, ingest_file()
├── error.rs            # MODIFY: add UnsupportedFormat, FileTooLarge, InvalidEncoding, NotAFile
├── lib.rs              # UNCHANGED
└── main.rs             # UNCHANGED

tests/
└── cli_integration.rs  # MODIFY: add ingestion integration tests
```

**Structure Decision**: Single binary crate with modular `src/` layout (matches existing 001-scaffolding output). The `ingest` module is the primary implementation target. Changes touch 4 existing files and add no new files.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Single crate instead of workspace (Principle I) | Feature scope is ~150 LOC, 2 structs, 1 function. Crate overhead (Cargo.toml, lib.rs, workspace config) exceeds the code it would contain. | Module boundaries provide the same API isolation. `pub fn ingest_file()` is the only public entry point. Extraction to a crate later is a mechanical refactor (move files, add Cargo.toml, update workspace). |

## Implementation Sequence

> **Note**: tasks.md reorganizes these phases by user story (story-centric) rather than by activity type. Task IDs in tasks.md are the authoritative execution order.

### Phase 1: Contracts & Types (Contract-First)

1. **Add dependencies** to `Cargo.toml`: `serde`, `serde_json`, `sha2`
2. **Define error variants** in `src/error.rs`: `UnsupportedFormat`, `FileTooLarge`, `InvalidEncoding`, `NotAFile`
3. **Define domain types** in `src/ingest/mod.rs`: `IngestedDocument`, `SourceLine` with `Serialize` derives
4. **Define public API** in `src/ingest/mod.rs`: `pub fn ingest_file(path: &Path, max_size_bytes: u64) -> Result<IngestedDocument, ForgeError>`
5. **Add `--max-size` CLI arg** to `Commands::Convert` in `src/cli/mod.rs`

### Phase 2: Tests (Test-First)

6. **Unit tests** for `ingest_file()`:
   - Valid .md file → returns IngestedDocument with correct fields
   - Valid .markdown file → accepted
   - Case-insensitive extension (.MD, .Markdown) → accepted
   - Unsupported extension (.pdf, .docx, no extension) → UnsupportedFormat error
   - Non-existent file → Io error (NotFound)
   - Non-UTF-8 file → InvalidEncoding error
   - Oversized file → FileTooLarge error
   - Empty file → valid IngestedDocument with empty lines
   - Directory path → NotAFile error
   - Line numbering accuracy (1-based, correct count)
   - SHA-256 fingerprint determinism
7. **Unit tests** for error display messages (verify actionable suggestions)
8. **Integration tests** in `tests/cli_integration.rs`:
   - `forge convert valid.md` → exits 0, stdout is valid JSON matching schema
   - `forge convert policy.pdf` → exits non-zero, stderr contains suggestion
   - `forge convert missing.md` → exits non-zero, stderr contains "not found"
   - `forge convert valid.md --max-size 0` → exits non-zero (size rejection)

### Phase 3: Implementation

9. **Implement `ingest_file()`** in `src/ingest/mod.rs`:
   - Extension validation
   - Metadata checks (is_file, size)
   - Read file content (UTF-8)
   - Compute SHA-256
   - Split into SourceLines
   - Return IngestedDocument
10. **Implement `convert::execute()`** in `src/cli/convert.rs`:
    - Extract `max_size` from CLI args (default 10MB)
    - Call `ingest_file()`
    - Serialize result to JSON
    - Print to stdout

### Phase 4: Verification

11. All tests pass (`cargo test`)
12. `cargo clippy -- -D warnings` passes
13. `cargo fmt --check` passes
14. `cargo doc --no-deps` builds without warnings

## Traceability

| Spec Requirement | Implementation Location | Test Coverage |
|-----------------|------------------------|---------------|
| FR-001 (read + JSON output) | `ingest::ingest_file()` + `convert::execute()` | Unit: valid file → JSON; Integration: stdout JSON |
| FR-002 (extension detection) | `ingest::ingest_file()` | Unit: .md, .markdown, .MD, .MARKDOWN accepted |
| FR-003 (reject unsupported) | `ingest::ingest_file()` | Unit: .pdf, .docx, no ext → error; Integration: stderr message |
| FR-004 (line tracking) | `ingest::ingest_file()` | Unit: line count + numbering accuracy |
| FR-005 (file not found) | `ingest::ingest_file()` via `ForgeError::Io` | Unit + Integration: non-existent path |
| FR-006 (permission denied) | `ingest::ingest_file()` via `ForgeError::Io` | Unit: restricted file |
| FR-007 (UTF-8 validation) | `ingest::ingest_file()` | Unit: non-UTF-8 → InvalidEncoding |
| FR-008 (SHA-256 fingerprint) | `ingest::ingest_file()` | Unit: deterministic hash |
| FR-009 (source path) | `IngestedDocument.source_path` | Unit: path preserved in output |
| FR-010 (size limit + override) | `ingest::ingest_file()` + CLI `--max-size` | Unit: oversized → error; Integration: --max-size flag |
| FR-011 (regular file check) | `ingest::ingest_file()` | Unit: directory → NotAFile |
