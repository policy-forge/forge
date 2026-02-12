# Research: Deterministic UUID Generation

**Feature**: 007-uuid-generation | **Date**: 2026-02-11

## R-1: UUID Generation Approach

**Decision**: Use `uuid` crate with UUID v5 (namespace + SHA-1 hash of content)

**Rationale**: UUID v5 is deterministic by definition — same namespace + same name always produces the same UUID. This satisfies PRD M-1, M-4, and parent PRD M-8 without any persistence layer. The `uuid` crate is the de facto Rust UUID library (MIT/Apache-2.0), supports v5 via the `v5` feature flag, and produces RFC 4122 compliant UUIDs that OSCAL tooling expects.

**Alternatives considered**:
- **UUID v4 (random)**: Rejected — non-deterministic, violates P-3 and M-8. Explicitly rejected in parent PRD Decision Log.
- **Custom SHA-256 hash-based IDs**: Rejected — non-RFC 4122 format; OSCAL expects standard UUIDs; reinvents what `uuid` crate provides; violates YAGNI.
- **Do nothing**: Rejected — leaves `stable_id` as `None`, no deterministic identifiers.

## R-2: Namespace UUID Value

**Decision**: Generate a new project-specific UUID v4 once and hardcode it as a compile-time constant

**Rationale**: A project-specific UUID v4 ensures FORGE's identifier namespace is globally unique and won't collide with other tools that may use UUID v5. The namespace UUID is hardcoded as `pub const FORGE_NAMESPACE_UUID: Uuid = Uuid::from_bytes([...]);` — a compile-time constant that cannot be changed at runtime (SEC-3).

**Alternatives considered**:
- **Use example UUID from RFC/AR document (6ba7b810-9dad...)**: Rejected — this is a well-known DNS namespace UUID; would tie FORGE identifiers to a public namespace, risking collisions.
- **Derive from hash of "FORGE"**: Rejected — unconventional; adds unnecessary complexity.
- **Runtime-configurable**: Rejected — accidental changes would break all generated IDs (PRD S-1, Implementation Guardrail).

**Implementation**: Generate a UUID v4 during development using `uuid::Uuid::new_v4()` (or `uuidgen` CLI), then hardcode the bytes as a constant. The specific value is an implementation detail; what matters is that it's unique and never changes.

## R-3: Content Normalization Strategy

**Decision**: Trim leading/trailing whitespace and collapse all internal whitespace runs to single spaces using `split_whitespace().collect::<Vec<&str>>().join(" ")`

**Rationale**: Rust's `split_whitespace()` handles Unicode whitespace, tabs, newlines, and multiple spaces in a single idiomatic expression. This satisfies PRD M-2 (whitespace resilience) while preserving substantive text differences (PRD M-5). It also leverages Rust's standard library Unicode handling (Assumption A-5).

**Alternatives considered**:
- **No normalization**: Rejected — trivial formatting changes would alter UUIDs, violating EC-5.
- **Aggressive normalization (lowercasing, punctuation removal)**: Rejected — risks false collisions between distinct requirements (PRD W-3).
- **Regex-based whitespace replacement**: Rejected — more complex, less idiomatic Rust; `split_whitespace` already does this correctly.

## R-4: Module Organization

**Decision**: Dedicated `src/uuid.rs` module for clear separation of concerns

**Rationale**: The UUID generation functions are a distinct capability, separate from the domain model (WI-5) and pipeline orchestration. A dedicated module enables reuse for other content-addressed identifiers (PRD S-2) and makes the logic independently testable.

**Alternatives considered**:
- **Part of domain model (src/model/)**: Rejected — UUID generation is a capability, not data definition.
- **Part of pipeline code**: Rejected — ties UUID logic to orchestration; harder to reuse.
- **Utils module**: Rejected — too generic; this has a clear, specific purpose.

## R-5: Pipeline Integration Point

**Decision**: Automatically invoke `assign_stable_ids` immediately after requirement atomization (WI-6) completes

**Rationale**: Ensures `stable_id` is always populated before downstream OSCAL generation (WI-9) receives the `PolicyDocument`. Prevents incomplete data from flowing through the pipeline.

**Alternatives considered**:
- **Separate CLI command**: Rejected — users must remember an extra step; risk of None stable_ids reaching OSCAL generation.
- **Lazy on-demand generation**: Rejected — adds complexity; violates the explicit assignment contract (PRD M-3).
- **During serialization**: Rejected — too late; downstream processing needs stable_ids.

## R-6: Debug Logging Scope

**Decision**: Log normalized text + generated UUID at debug level (PRD C-1)

**Rationale**: Sufficient to verify that normalization and UUID generation work correctly. Including only normalized text (not original) avoids log bloat while still enabling debugging of the core transformation. Source location context is unnecessary since the debug log is per-requirement.

**Implementation**: `debug!("UUID generated: text='{}' uuid='{}'", normalized_text, uuid)`

## R-7: uuid Crate Version and Features

**Decision**: Use latest stable `uuid` crate with `v5` feature enabled

**Rationale**: The `v5` feature provides `Uuid::new_v5()` which implements RFC 4122 UUID v5 (SHA-1 hash of namespace + name). No other features needed. The crate is MIT/Apache-2.0 licensed, actively maintained, and widely used in the Rust ecosystem.

**Cargo.toml addition**:

```toml
uuid = { version = "1", features = ["v5"] }
```

Note: The `v4` feature is NOT needed for this WI. It may be needed later for generating the namespace UUID during development, but the namespace UUID is hardcoded as bytes, not generated at runtime.
