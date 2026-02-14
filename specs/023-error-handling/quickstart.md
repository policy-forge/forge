# Quickstart: Error Handling & Robustness (WI-23)

**Phase 1 output** | **Date**: 2026-02-13

## Prerequisites

- Rust 1.93.0+ (edition 2024)
- All existing tests passing: `cargo test`
- All clippy checks passing: `cargo clippy -- -D warnings`

## Implementation Order

Follow this order strictly. Each phase builds on the previous and all tests must pass before proceeding.

### Phase A: Dependencies & Error Type Expansion

**Files**: `Cargo.toml`, `src/error.rs`

1. Add `anyhow` and `tracing-subscriber` to `Cargo.toml` dependencies
2. Add 5 new ForgeError variants to `src/error.rs` (see `contracts/error.rs`):
   - `FileNotFound { path }`
   - `PermissionDenied { path }`
   - `EmptyInput { path }`
   - `BinaryFile { path }`
   - `NoStructureDetected { path }`
3. Add `exit_code()` function to `src/error.rs` (see `contracts/exit_codes.rs`)
4. Write unit tests for all new variant Display implementations
5. Write unit tests for `exit_code()` covering all variants
6. Run `cargo test` — all tests pass

### Phase B: Input Validation Hardening

**Files**: `src/ingest/mod.rs`

1. Add `is_binary_content()` helper function (see `contracts/input_validation.rs`)
2. Modify `ingest_file()` to disaggregate I/O errors:
   - Replace `std::fs::metadata(path)?` with explicit `map_err` matching `ErrorKind::NotFound` → `FileNotFound`, `ErrorKind::PermissionDenied` → `PermissionDenied`
   - Apply same pattern to `std::fs::read(path)?`
3. Add empty file detection after reading bytes: `if bytes.is_empty() { return Err(EmptyInput) }`
4. Add binary detection after empty check, before UTF-8 conversion
5. Write unit tests for each new check (TDD: test first, then implement)
6. Update existing tests that depend on `ForgeError::Io` for file-not-found — they should now expect `FileNotFound`
7. Run `cargo test` — all tests pass

### Phase C: Pipeline Error Improvements

**Files**: `src/pipeline.rs`

1. Replace empty-file `Validation(String)` (line 63-67) with proper `EmptyInput` check (or remove if handled in ingest)
2. Replace `tracing::warn!` for no sections (line 73-75) with `NoStructureDetected` error when both sections AND clauses are empty
3. Write integration tests for empty file and no-structure-detected pipeline errors
4. Run `cargo test` — all tests pass

### Phase D: Exit Code Mapping & main.rs

**Files**: `src/main.rs`

1. Add `anyhow` import (for potential future context use)
2. Add `tracing-subscriber` initialization wired to `--verbose`/`--quiet`
3. Replace `process::exit(1)` with `exit_code(&err)` mapping via `ExitCode`
4. Write integration tests verifying distinct exit codes for different error categories
5. Run `cargo test` — all tests pass

### Phase E: .unwrap()/.expect() Audit

**Files**: All `src/*.rs` files

1. Add `// SAFETY:` comments to 6 safe `.expect()` calls in regex compilation:
   - `citation.rs:32,37,42,48`
   - `parse/atomize.rs:24,29`
2. Add `// SAFETY:` comment to `parse/clauses.rs:254` documenting the `is_empty()` guard
3. Verify zero unreviewed `.unwrap()` in production code
4. Run `cargo test` — all tests pass

### Phase F: Validate Stub Hardening

**Files**: `src/cli/validate.rs`

1. Replace `println!("not yet implemented"); Ok(())` with `Err(ForgeError::Validation("validate command not yet implemented — coming in a future release".to_string()))`
2. This ensures the stub returns non-zero exit code instead of silently succeeding
3. Update any tests that depend on validate returning Ok
4. Run `cargo test` — all tests pass

### Phase G: Adversarial Input Test Suite

**Files**: `tests/adversarial_input_test.rs`, `tests/fixtures/adversarial/`

1. Create test fixture files:
   - `tests/fixtures/adversarial/empty.md` — zero bytes
   - `tests/fixtures/adversarial/binary.bin` — PNG header bytes (rename to .md for test)
   - `tests/fixtures/adversarial/null_bytes.md` — filled with null bytes
   - `tests/fixtures/adversarial/whitespace_only.md` — only spaces/tabs/newlines
   - `tests/fixtures/adversarial/no_newlines.md` — single long line with no newlines
   - Large file generated at test runtime (>10MB)
2. Write integration tests asserting for each:
   - No panics
   - Non-zero exit code
   - Descriptive error message
3. Add exit code verification tests to `tests/cli_integration.rs`
4. Run `cargo test` — all tests pass

### Phase H: Final Verification

1. `cargo fmt --check` — no formatting issues
2. `cargo clippy -- -D warnings` — no clippy warnings
3. `cargo test` — all tests pass (including new adversarial suite)
4. Grep audit: `grep -rn '\.unwrap()' src/ --include='*.rs'` excluding `#[cfg(test)]` — all documented
5. Grep audit: `grep -rn 'panic!\|todo!\|unimplemented!' src/ --include='*.rs'` excluding `#[cfg(test)]` — zero occurrences
6. Review all ForgeError Display implementations for information leakage (SEC-1 through SEC-5)

## Testing Strategy

| Layer | What | Coverage Target |
|-------|------|-----------------|
| Unit | ForgeError Display for all 17 variants | 100% |
| Unit | `exit_code()` for all 17 variants | 100% |
| Unit | `is_binary_content()` for magic bytes + null ratio | All signatures + edge cases |
| Unit | Ingest validation chain (each check) | All conditions |
| Integration | Adversarial input suite (6+ inputs) | No panics, correct exit codes |
| Integration | CLI exit code verification | 0 for success, 1/2/3 for errors |
| Integration | Pipeline error paths | Context in error messages |

## Key Files Summary

| File | Changes |
|------|---------|
| `Cargo.toml` | Add `anyhow`, `tracing-subscriber` |
| `src/error.rs` | +5 variants, +`exit_code()` fn, +tests |
| `src/ingest/mod.rs` | +binary detection, +IoError disaggregation, +empty check |
| `src/pipeline.rs` | Replace Validation(String) with typed variants |
| `src/main.rs` | +exit code mapping, +tracing init |
| `src/cli/validate.rs` | Stub returns error instead of Ok |
| `src/citation.rs` | +SAFETY comments on .expect() |
| `src/parse/atomize.rs` | +SAFETY comments on .expect() |
| `src/parse/clauses.rs` | +SAFETY comment on .unwrap() |
| `tests/adversarial_input_test.rs` | NEW: adversarial test suite |
| `tests/fixtures/adversarial/` | NEW: test fixture files |
