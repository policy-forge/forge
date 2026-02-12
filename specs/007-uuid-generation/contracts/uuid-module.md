# Contract: UUID Generation Module (`src/uuid.rs`)

**Feature**: 007-uuid-generation | **Date**: 2026-02-11

## Module Purpose

Deterministic UUID v5 generation for content-addressed identifiers. All functions are deterministic for a given input and perform no external network or filesystem I/O, but may mutate in-memory data structures and emit tracing/logging instrumentation.

## Public API

### Constant: `FORGE_NAMESPACE_UUID`

```rust
/// Fixed namespace UUID for all FORGE content-addressed identifier generation.
/// This is a project-specific UUID v4 generated once and hardcoded to ensure global uniqueness.
///
/// # Breaking Change Warning
///
/// Changing this value will change ALL generated stable_ids across the entire
/// FORGE ecosystem. Any change requires a documented migration path.
pub const FORGE_NAMESPACE_UUID: Uuid = Uuid::from_bytes([
    // 16 bytes — project-specific v4 UUID generated during implementation
    // Example: 0xAA, 0xBB, 0xCC, ...
]);
```

**Guarantees**:
- Compile-time constant — cannot be changed at runtime (SEC-3)
- Unique to FORGE project — no collision with other UUID v5 namespaces
- Immutable — changing is a breaking change

---

### Function: `normalize_for_hashing`

```rust
/// Normalize text for stable ID generation.
///
/// Trims leading/trailing whitespace and collapses internal whitespace runs
/// to a single space. Uses Rust's `split_whitespace()` which handles Unicode
/// whitespace characters (NBSP, em space, etc.), tabs, and newlines.
///
/// # Arguments
///
/// * `text` - Raw text to normalize
///
/// # Returns
///
/// Normalized string with single-space separation. Empty string if input
/// is empty or contains only whitespace.
///
/// # Examples
///
/// ```
/// assert_eq!(normalize_for_hashing("  foo   bar  "), "foo bar");
/// assert_eq!(normalize_for_hashing("foo\t\nbar"), "foo bar");
/// assert_eq!(normalize_for_hashing("   "), "");
/// assert_eq!(normalize_for_hashing(""), "");
/// ```
pub fn normalize_for_hashing(text: &str) -> String
```

**Contract**:
- Input: Any `&str` (including empty, whitespace-only, Unicode)
- Output: Normalized `String` with single-space word separation
- Pure function: no side effects
- Idempotent: `normalize(normalize(x)) == normalize(x)`

---

### Function: `generate_stable_id`

```rust
/// Generate a deterministic UUID v5 from text content.
///
/// The text is normalized before hashing to ensure whitespace-insensitivity.
/// Uses `FORGE_NAMESPACE_UUID` as the namespace parameter.
///
/// This function accepts any string input, not just PolicyRequirement text,
/// enabling reuse for other content-addressed identifiers (PRD S-2).
///
/// # Arguments
///
/// * `text` - Raw text to generate UUID from (will be normalized internally)
///
/// # Returns
///
/// A deterministic `Uuid` (v5) derived from the normalized text.
///
/// # Determinism Guarantee
///
/// Same text → same UUID, always. Different substantive text → different UUID.
/// Whitespace-only differences are normalized away.
pub fn generate_stable_id(text: &str) -> Uuid
```

**Contract**:
- Input: Any `&str`
- Output: `uuid::Uuid` (RFC 4122 v5)
- Deterministic: same input always produces same output (PRD M-4)
- Whitespace-insensitive: whitespace-only changes produce same UUID (PRD M-2)
- Content-sensitive: substantive text changes produce different UUID (PRD M-5)
- Pure function: no side effects, no I/O (SEC-4)

---

### Function: `assign_stable_ids`

```rust
/// Populate stable_id on all PolicyRequirements in a PolicyDocument.
///
/// Walks the full section tree recursively, generating a UUID v5 for each
/// requirement's text and setting `stable_id = Some(uuid.to_string())`.
///
/// After this function returns, no PolicyRequirement in the document will
/// have `stable_id = None`.
///
/// # Arguments
///
/// * `document` - Mutable reference to the PolicyDocument to process
pub fn assign_stable_ids(document: &mut PolicyDocument)
```

**Contract**:
- Input: `&mut PolicyDocument` with `stable_id = None` on requirements
- Output: All requirements have `stable_id = Some(uuid_string)` (PRD M-3)
- Recursive: processes all nesting depths (EC-5)
- Idempotent: calling twice produces same result (deterministic UUIDs)

---

## Internal Function

### `assign_stable_ids_to_section`

```rust
/// Recursively assign stable IDs to all requirements in a section and its children.
fn assign_stable_ids_to_section(section: &mut PolicySection)
```

Not part of public API. Implementation detail.

## Error Handling

None of these functions can fail:
- `normalize_for_hashing`: Always produces a valid String (empty input → empty output)
- `generate_stable_id`: `Uuid::new_v5` is infallible
- `assign_stable_ids`: Walks a tree, calls infallible functions

No `Result` return types needed. No error variants added to `ForgeError`.

## Dependencies

| Dependency | Version | Usage |
|-----------|---------|-------|
| `uuid` | 1.x | `Uuid`, `Uuid::new_v5`, `Uuid::from_bytes` |
| `crate::model` | internal | `PolicyDocument`, `PolicySection`, `PolicyRequirement` |
