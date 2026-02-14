# Research: Error Handling & Robustness (WI-23)

**Phase 0 output** | **Date**: 2026-02-13

## R1: Current ForgeError Gap Analysis

**Decision**: Extend the existing 12-variant `ForgeError` enum with 4 new variants; modify 1 existing variant.

**Rationale**: The current enum already covers several PRD requirements (UnsupportedFormat, FileTooLarge, InvalidEncoding, NotAFile). The following gaps remain:

| PRD Requirement | Current Handling | Gap | Resolution |
|----------------|-----------------|-----|------------|
| M-6: FileNotFound with path | `ForgeError::Io(io::Error)` — generic, path lost | No dedicated variant | Add `FileNotFound { path }` |
| S-1: PermissionDenied with path | `ForgeError::Io(io::Error)` — generic, path lost | No dedicated variant | Add `PermissionDenied { path }` |
| M-7: Empty file detection | `ForgeError::Validation(String)` in pipeline.rs:64 | String-based, no dedicated variant | Add `EmptyInput { path }` |
| M-8: No structure detected | `tracing::warn!` in pipeline.rs:74 (continues processing) | Warning only, not an error | Add `NoStructureDetected { path }` |
| M-9: Multiple validation errors | `ForgeError::Validation(String)` — single string | Cannot carry multiple errors | Will require Validation variant rework when validate command is implemented (WI-19/WI-20) |
| S-2: Binary file detection | `InvalidEncoding { path }` catches non-UTF-8 | No binary magic byte detection | Add binary detection before UTF-8 check in ingest |

**Existing variants that already satisfy PRD requirements:**
- `UnsupportedFormat { extension }` → partially satisfies S-2 (extension-based detection)
- `FileTooLarge { path, size_bytes, limit_bytes }` → satisfies S-3
- `InvalidEncoding { path }` → catches binary files that fail UTF-8
- `NotAFile { path }` → satisfies EC-7

**Alternatives considered**: Per-module error enums (AR Option 2) — rejected as disproportionate refactoring for a hardening sprint. Consolidating string-based variants (Parse, Validation, CatalogBuild, etc.) into structured variants — deferred per AR decision to minimize refactoring scope.

## R2: anyhow Integration Strategy

**Decision**: Add `anyhow` as a dependency. Use `anyhow::Context` only in `main.rs` for top-level error wrapping. Library code continues using `Result<T, ForgeError>`.

**Rationale**: The constitution mandates `anyhow` only in binary crates. Since FORGE is a single crate, `main.rs` is the binary entry point and the only place where `anyhow` should appear. Pipeline and library functions remain typed as `Result<T, ForgeError>`.

**Pattern**:
```rust
// main.rs only
use anyhow::Context;

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli::execute(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(forge_err) => {
            eprintln!("Error: {forge_err}");
            ExitCode::from(exit_code(&forge_err))
        }
    }
}
```

**Context propagation in library code**: Use `.map_err()` at pipeline stage boundaries to enrich ForgeError messages with operation context. Example: `.map_err(|e| ForgeError::Parse { path: input_path.to_path_buf(), message: format!("while extracting sections: {e}") })?`

**Alternatives considered**:
1. `anyhow` throughout the binary crate (cli/, pipeline.rs) — rejected because it erases type info needed for exit code mapping without downcasting.
2. No `anyhow` at all, only `.map_err()` — viable but `anyhow` provides a cleaner `{:#}` display for error chains in `main.rs`.
3. Custom `.context()` extension trait on `Result<T, ForgeError>` — over-engineering per principle X.

## R3: IoError Disaggregation Strategy

**Decision**: In `ingest_file()`, replace `std::fs::metadata(path)?` (which converts to `ForgeError::Io` via `#[from]`) with explicit `match` on `io::ErrorKind` to produce specific variants.

**Rationale**: The `#[from] std::io::Error` conversion on `ForgeError::Io` loses the file path. For user-actionable messages (PRD M-2, M-6), we need to include the path. The disaggregation happens at the ingestion boundary — the first place file I/O occurs.

**Pattern**:
```rust
let metadata = std::fs::metadata(path).map_err(|e| match e.kind() {
    std::io::ErrorKind::NotFound => ForgeError::FileNotFound { path: path.to_path_buf() },
    std::io::ErrorKind::PermissionDenied => ForgeError::PermissionDenied { path: path.to_path_buf() },
    _ => ForgeError::Io(e),
})?;
```

**Apply same pattern to**: `std::fs::read(path)` call on ingest/mod.rs:76.

**Alternatives considered**: Removing `#[from] std::io::Error` entirely — rejected because some I/O errors (e.g., in `write_output()`) don't need path-specific variants.

## R4: Binary Content Detection

**Decision**: Add binary detection in `ingest_file()` between the size check and UTF-8 check. Use a two-tier approach: (1) magic byte signatures, (2) null byte ratio heuristic.

**Rationale**: The current `InvalidEncoding` variant catches non-UTF-8 binary files, but the error message says "not valid UTF-8" which doesn't clearly convey "this is a binary file." Adding explicit binary detection provides a better user experience (PRD S-2) and covers the SEC-8 requirement for magic byte checking.

**Algorithm** (from AR):
1. Read raw bytes (already done for SHA-256 fingerprint)
2. Check first 4-8 bytes against known magic byte signatures:
   - PNG: `\x89PNG\r\n\x1a\n`
   - JPEG: `\xFF\xD8\xFF`
   - PDF: `%PDF`
   - ZIP/DOCX: `PK\x03\x04`
   - ELF: `\x7fELF`
   - Mach-O: `\xFE\xED\xFA` or `\xCF\xFA\xED\xFE`
3. If no signature match, check first 512 bytes for null byte ratio >10%
4. If binary detected, return `ForgeError::UnsupportedFormat` (reuse existing variant with enhanced message) or a new variant

**Decision on variant**: Reuse the existing `InvalidEncoding` concept but add a separate `BinaryFile { path }` variant to distinguish "this is clearly a binary file" from "this has encoding issues." The PRD interface contract shows `UnsupportedFormat { path }` for binary detection, but the current codebase uses `UnsupportedFormat { extension }`. We'll add a new variant to avoid changing the existing extension-based variant.

**Implementation**: Insert binary detection after `std::fs::read(path)` (line 76) and before `String::from_utf8()` (line 79) in `ingest/mod.rs`.

**Alternatives considered**: Relying solely on `InvalidEncoding` — rejected because "not valid UTF-8" is less actionable than "appears to be a binary file."

## R5: Empty File and No-Structure Detection

**Decision**: Add empty file detection in `ingest_file()` (before any parsing). Move no-structure detection from warning to error in `prepare_document()`.

**Rationale**:
- **Empty file** (PRD M-7): Currently detected in `pipeline.rs:63-67` as `ForgeError::Validation(String)`. Move detection earlier to `ingest_file()` with a dedicated `EmptyInput { path }` variant, so it fails fast before any pipeline processing.
- **No structure** (PRD M-8): Currently a `tracing::warn!` in `pipeline.rs:74` that continues processing. Per PRD, this should be an error. Add a `NoStructureDetected { path }` variant and return it when `sections.is_empty()` AND `clauses` contains no list items or table items.

**Empty detection logic**: After `String::from_utf8()` succeeds, check if `content.trim().is_empty()`. If so, return `EmptyInput { path }`.

**No-structure detection logic**: After `extract_sections()` and `extract_clauses()`, check if both are empty. If so, return `NoStructureDetected { path }`.

**Alternatives considered**: Keeping empty file as a Validation(String) — rejected because it loses the path and doesn't distinguish from other validation errors for exit code mapping.

## R6: Exit Code Mapping

**Decision**: Map ForgeError variants to 3 exit code categories: 1=input/IO, 2=parse/structure, 3=validation.

**Rationale**: PRD S-4 requires distinct exit codes for programmatic classification. The AR selects this mapping:

| Exit Code | Error Category | ForgeError Variants |
|-----------|---------------|---------------------|
| 0 | Success | N/A |
| 1 | Input/IO | FileNotFound, PermissionDenied, EmptyInput, BinaryFile, UnsupportedFormat, FileTooLarge, InvalidEncoding, NotAFile, Io |
| 2 | Parse/Structure | NoStructureDetected, Parse, CatalogBuild, BackMatter, ComponentDefinitionBuild |
| 3 | Validation | Validation, Config |
| 1 | Fallback | Serialization (unlikely in practice) |

**Implementation**: `pub fn exit_code(err: &ForgeError) -> u8` function in `error.rs`.

**Alternatives considered**: Single non-zero exit code for all errors — rejected as less useful for CI/CD automation (PRD S-4).

## R7: .unwrap()/.expect() Audit Results

**Decision**: 6 `.expect()` calls in production code are safe (static regex compilation). 1 `.unwrap()` is safe but needs documentation. 0 risky production `.unwrap()` calls found.

**Detailed audit**:

| Location | Call | Assessment | Action |
|----------|------|------------|--------|
| `citation.rs:32` | `Regex::new(...).expect(...)` | Safe: static regex, panics on startup if invalid | Add `// SAFETY:` comment |
| `citation.rs:37` | `Regex::new(...).expect(...)` | Safe: static regex | Add `// SAFETY:` comment |
| `citation.rs:42` | `Regex::new(...).expect(...)` | Safe: static regex | Add `// SAFETY:` comment |
| `citation.rs:48` | `Regex::new(...).expect(...)` | Safe: static regex | Add `// SAFETY:` comment |
| `parse/atomize.rs:24` | `Regex::new(...).expect(...)` | Safe: static regex | Add `// SAFETY:` comment |
| `parse/atomize.rs:29` | `Regex::new(...).expect(...)` | Safe: static regex | Add `// SAFETY:` comment |
| `parse/clauses.rs:254` | `list_type_stack.last().unwrap()` | Safe: guarded by `!list_type_stack.is_empty()` check on line 253 | Add `// SAFETY:` comment documenting the guard |

**`.unwrap_or()` and `.unwrap_or_else()` calls**: ~13 occurrences, all safe (provide default values). No action needed.

**`panic!()`, `todo!()`, `unimplemented!()`**: Zero occurrences found in production code. PRD M-3 requirement is already met.

## R8: Validate Subcommand Status

**Decision**: The `forge validate` command is currently a stub (`cli/validate.rs:10-13`). PRD M-9 requires multiple validation errors to be reported by this command. However, the actual validation logic depends on WI-19/WI-20 (schema validation).

**Resolution**: For WI-23, update the validate stub to return a proper error (not silently succeed). The full validation error collection (M-9 with `Vec<ValidationDetail>`) will be implemented when the validate command gets real logic. The error infrastructure (variant with `Vec`) should be prepared but the command implementation is out of scope.

**Alternatives considered**: Implementing full validation in WI-23 — rejected as scope creep beyond the hardening mandate.

## R9: source_path Canonicalization (SEC F3)

**Decision**: The `IngestedDocument.source_path` field uses `path.canonicalize()` (ingest/mod.rs:88), which resolves to an absolute system path. Per SEC review F3, error messages must not expose resolved system paths.

**Assessment**: `source_path` is used in the pipeline for metadata and tracing, not in error messages. ForgeError variants receive paths from the `path` parameter (user-provided), not from `source_path`. **No change needed** — the current design already separates user-provided paths (in errors) from canonical paths (in processing).

**Action**: Verify during implementation that no error construction path passes `source_path` into ForgeError variants.

## R10: tracing Subscriber Initialization

**Decision**: Wire the existing `--verbose` and `--quiet` CLI flags to `tracing_subscriber` initialization in `main.rs`.

**Rationale**: Constitution principle IX requires observability. The `--verbose`/`--quiet` flags are already defined in `Cli` but unused. Adding tracing subscriber initialization is minimal effort and enables error context visibility for debugging.

**New dependency**: `tracing-subscriber` (latest stable) for `fmt::Subscriber` with `EnvFilter`.

**Pattern**:
```rust
let filter = if cli.verbose { "debug" } else if cli.quiet { "error" } else { "warn" };
tracing_subscriber::fmt().with_env_filter(filter).init();
```

**Alternatives considered**: Full observability infrastructure — deferred to dedicated observability WI per constitution.
