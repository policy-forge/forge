# Quickstart: Deterministic UUID Generation

**Feature**: 007-uuid-generation | **Date**: 2026-02-11

## Prerequisites

- Rust stable 1.93.0+ (edition 2024)
- Existing FORGE repository cloned and building (`cargo build` passes)
- Feature branch: `git checkout 007-uuid-generation`

## Setup

### 1. Add uuid dependency

```bash
cargo add uuid --features v5
```

This adds to `Cargo.toml`:

```toml
uuid = { version = "1", features = ["v5"] }
```

### 2. Register the module

In `src/lib.rs`, add:

```rust
pub mod uuid;
```

### 3. Create the module file

Create `src/uuid.rs` with the three public functions and the namespace constant.

## Key Patterns

### Generating a UUID for any text

```rust
use forge::uuid::generate_stable_id;

let uuid = generate_stable_id("All users must use multi-factor authentication");
println!("{uuid}"); // e.g., "a1b2c3d4-e5f6-5789-abcd-ef0123456789"

// Same text always produces same UUID
let uuid2 = generate_stable_id("All users must use multi-factor authentication");
assert_eq!(uuid, uuid2);
```

### Whitespace normalization

```rust
use forge::uuid::normalize_for_hashing;

assert_eq!(normalize_for_hashing("  foo   bar  "), "foo bar");
assert_eq!(normalize_for_hashing("foo\t\nbar"), "foo bar");
assert_eq!(normalize_for_hashing(""), "");
```

### Assigning IDs to a document

```rust
use forge::uuid::assign_stable_ids;

let mut document: PolicyDocument = /* from WI-6 atomization */;

// Before: all stable_ids are None
assign_stable_ids(&mut document);
// After: all stable_ids are Some(uuid_string)
```

## Testing

```bash
# Run all tests
cargo test

# Run only UUID generation tests
cargo test uuid

# Run with tracing debug output to see normalization output
RUST_LOG=debug cargo test uuid

# Lint check
cargo clippy -- -D warnings

# Format check
cargo fmt --check
```

## TDD Workflow

This feature uses Test-Driven Development:

1. **Write test first** — e.g., `test_same_text_produces_same_uuid()`
2. **Run test, verify it fails** — `cargo test test_same_text`
3. **Implement minimal code** to make test pass
4. **Run test, verify it passes**
5. **Refactor** if needed (keeping tests green)
6. **Repeat** for next test

### Test categories (in order):

1. **Normalization tests**: verify `normalize_for_hashing` behavior
2. **Determinism tests**: same text → same UUID (AC-1)
3. **Whitespace resilience tests**: formatting changes → same UUID (AC-2)
4. **Sensitivity tests**: substantive changes → different UUID (AC-3)
5. **Coverage tests**: all requirements populated (AC-4)
6. **Format tests**: valid RFC 4122 UUID v5 format (AC-5)
7. **Edge case tests**: empty text, Unicode whitespace, nested sections (EC-1 through EC-5)

## Common Issues

### "unresolved import `uuid`"

Ensure `uuid` is in `[dependencies]` in `Cargo.toml` with `features = ["v5"]`.

### "cannot find type `PolicyDocument`"

The domain model types depend on WI-5/WI-6. If not yet implemented, use stub types or test only the core `generate_stable_id` function with `&str` input.

### "UUID version is not 5"

Verify you're using `Uuid::new_v5`, not `Uuid::new_v4`. Check the version nibble: byte 6, high nibble should be `5`.
