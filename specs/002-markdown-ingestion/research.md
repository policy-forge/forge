# Research: Markdown Ingestion

**Feature**: 002-markdown-ingestion
**Date**: 2026-02-11

## Research Tasks

### RT-001: JSON Serialization Crate

**Decision**: `serde` 1.x + `serde_json` 1.x

**Rationale**: De facto standard for Rust serialization. Listed in the project constitution's Technology Stack table. Zero-cost when possible, derive macros for ergonomic type definitions. Needed to serialize `IngestedDocument` to JSON for stdout output (FR-001).

**Alternatives considered**:
- `simd-json`: Higher performance but adds complexity; overkill for serializing a single document.
- Manual JSON construction: Error-prone, no type safety.

### RT-002: SHA-256 Hashing Crate

**Decision**: `sha2` 0.10.x (from RustCrypto project)

**Rationale**: Well-maintained, pure-Rust implementation from the RustCrypto ecosystem. No `unsafe` code. Hardware acceleration via `cpufeatures` crate when available. Needed for FR-008 (content fingerprint).

**Alternatives considered**:
- `ring`: More comprehensive crypto library, but heavier dependency for just SHA-256.
- `sha256` (wrapper crate): Thin wrapper around `sha2`, adds unnecessary indirection.
- `blake3`: Faster but not SHA-256; spec explicitly requires SHA-256.

### RT-003: Project Structure — Module vs Crate

**Decision**: Keep single-crate structure with module boundaries matching future crate extraction.

**Rationale**: Constitution Principle I (Crate-First) recommends standalone crates, but Principle X (Simplicity/YAGNI) takes precedence at current project scale. The `src/ingest/` module is already scaffolded with a clear boundary. The ingestion API will be designed with a clean public interface (`pub fn ingest_file(...)`) so extraction to `crates/ingest/` is mechanical when the project grows. Current scope (2 structs, 1 public function, ~150 lines) does not justify crate overhead.

**Alternatives considered**:
- Convert to workspace with `crates/ingest/`: Premature for project size; adds build complexity.
- Put types in `src/model/`: Splits ingestion concern across modules unnecessarily.

### RT-004: Error Handling Strategy

**Decision**: Extend existing `ForgeError` enum with ingestion-specific variants.

**Rationale**: Constitution Principle VIII requires `thiserror` for library errors with meaningful variants. The existing `ForgeError` already handles `Io` errors. New variants needed for format validation, encoding, and file size — these are distinct failure modes with different user-facing messages.

**Alternatives considered**:
- Separate `IngestError` enum: Would require conversion boilerplate. Premature until the ingest module is extracted to its own crate.
- Reuse `Validation(String)`: Loses type safety; can't pattern-match on specific ingestion failures.

### RT-005: File Extension Detection

**Decision**: Manual extension check using `Path::extension()` with case-insensitive comparison.

**Rationale**: Simple, zero-dependency approach. Only two extensions to match (`.md`, `.markdown`). `Path::extension()` handles edge cases (no extension, multiple dots) correctly.

**Alternatives considered**:
- `mime_guess` crate: Overkill for two extensions; adds dependency.
- Content-based detection (magic bytes): Markdown has no magic bytes; extension-based is the correct approach.
