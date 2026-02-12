# Research: OSCAL Metadata Assembly

**Phase**: 0 | **Date**: 2026-02-12

## R-1: `chrono` Crate for ISO 8601 Timestamps

**Decision**: Use `chrono` crate (latest stable) for `DateTime<Utc>` and ISO 8601 / RFC 3339 formatting.

**Rationale**:
- `chrono` provides `Utc::now()` returning `DateTime<Utc>` which serializes to RFC 3339 format via serde
- RFC 3339 is a profile of ISO 8601 — fully compatible with OSCAL timestamp requirements
- Default serde serialization of `DateTime<Utc>` produces format like `"2026-05-14T10:30:00Z"` (whole seconds) or `"2026-05-14T10:30:00.123456789Z"` (with nanoseconds if non-zero)
- NIST OSCAL examples include subsecond precision, so this is acceptable
- `chrono` is the industry-standard Rust datetime crate (MIT/Apache-2.0 license)

**Alternatives considered**:
- `time` crate: Lighter weight but less ergonomic serde integration; `chrono` already chosen in AR
- `std::time::SystemTime`: Manual formatting is error-prone (noted as anti-pattern in AR)

**Serde behavior**: `chrono` provides `serde` feature which serializes `DateTime<Utc>` to RFC 3339 string automatically. No custom serializer needed.

## R-2: `uuid` Crate v4 Feature

**Decision**: Add `v4` feature to existing `uuid` dependency (already at v1.20.0 with `v5` feature).

**Rationale**:
- `uuid` crate already in dependency tree from WI-7 (stable IDs use v5)
- Adding `v4` feature enables `Uuid::new_v4()` for random artifact instance UUIDs
- `v4` feature pulls in `getrandom` for OS-provided CSPRNG — well-audited randomness source
- UUID v4 serializes as standard hyphenated format (e.g., `"550e8400-e29b-41d4-a716-446655440000"`)

**Alternatives considered**:
- UUID v5 for artifacts: Rejected — v5 produces same UUID for same input, conflating distinct generation instances
- `rand` crate directly: Unnecessary — `uuid` crate handles randomness internally

**Cargo.toml change**: `uuid = { version = "1.20.0", features = ["serde", "v4", "v5"] }`

## R-3: Serde Rename for OSCAL Field Names

**Decision**: Use `#[serde(rename = "...")]` for hyphenated OSCAL field names.

**Rationale**:
- OSCAL JSON uses hyphenated field names: `last-modified`, `oscal-version`
- Rust identifiers cannot contain hyphens
- `#[serde(rename = "last-modified")]` on `last_modified` field handles serialization
- Same pattern for `oscal_version` → `oscal-version`

**Alternatives considered**:
- `#[serde(rename_all = "kebab-case")]` on struct: Would rename ALL fields including `uuid`, `title`, `version` which don't need renaming. Selective `rename` is more precise.

## R-4: Error Handling Strategy

**Decision**: `assemble_metadata` returns `Result<OscalMetadata, ForgeError>` but the current implementation path is infallible (always returns `Ok`).

**Rationale**:
- The AR shows the function body is always `Ok(...)` with no error paths
- Returning `Result` maintains API consistency with other forge functions and allows future extension (e.g., if validation of input fields is added)
- No new `ForgeError` variant needed at this time
- Empty title: accepted as-is with `tracing::warn!` (per AR error handling strategy and EC-1)

**Alternatives considered**:
- Return `OscalMetadata` directly (no `Result`): Simpler but breaks pattern if validation is added later
- Add `ForgeError::MetadataAssembly` variant: Over-engineering — no error paths exist currently

## R-5: Security Constraints (from SEC review)

**Decision**: Implement all SEC requirements as code-level constraints.

| SEC ID | Constraint | Implementation |
|--------|-----------|----------------|
| SEC-1 | No system-identifying information in metadata | Function only reads `DocumentMetadata` fields, `Uuid::new_v4()`, `Utc::now()`, `OSCAL_VERSION` |
| SEC-2 | UTC timestamps only (Z suffix) | Use `chrono::Utc::now()` exclusively |
| SEC-3 | Empty title must not panic or leak system info | Accept empty string, emit `tracing::warn!` |
| SEC-4 | Default version "0.0.0" used as-is | Direct clone from `DocumentMetadata.version` |
| SEC-5 | Pure function — no file I/O, no network, no env vars | Function signature constrains inputs |
| SEC-6 | MetadataOptions not exposed as CLI flags | Struct is `pub` in library but not wired to CLI |

## Summary

All technical decisions resolved. No NEEDS CLARIFICATION items remain.

| Topic | Decision | Confidence |
|-------|----------|------------|
| Timestamp crate | `chrono` (latest stable) | High |
| UUID generation | `uuid` v4 feature (existing crate) | High |
| Field naming | `#[serde(rename)]` per field | High |
| Error handling | `Result` return type, infallible body | High |
| Security | All SEC-1 through SEC-6 constraints implementable | High |
