# Quickstart: OSCAL Metadata Assembly

**Phase**: 1 | **Date**: 2026-02-12

## Prerequisites

- Rust stable 1.93.0+ (edition 2024)
- Existing `forge` crate with `src/oscal/mod.rs` (currently empty)
- `DocumentMetadata` struct available in `src/model/mod.rs`
- `ForgeError` enum available in `src/error.rs`

## Step-by-Step Implementation Order

### Step 1: Update Dependencies

Add `v4` feature to `uuid` and add `chrono` crate:

```bash
# In Cargo.toml, update uuid line:
uuid = { version = "1.20.0", features = ["v4", "v5"] }

# Add chrono:
cargo add chrono --features serde

# Verify no security advisories (Constitution XI):
cargo audit
```

### Step 2: Create `src/oscal/metadata.rs`

New file containing:
- `OSCAL_VERSION` constant
- `OscalMetadata` struct with serde derives
- `MetadataOptions` struct with Default derive
- `assemble_metadata` function
- `#[cfg(test)] mod tests` with unit tests

### Step 3: Update `src/oscal/mod.rs`

```rust
pub mod metadata;
pub use metadata::{assemble_metadata, MetadataOptions, OscalMetadata, OSCAL_VERSION};
```

### Step 4: Update `src/lib.rs`

Add re-export for ergonomic access:
```rust
pub use oscal::{OscalMetadata, assemble_metadata};
```

### Step 5: Write Tests (TDD — before implementation)

Tests to write:
1. `assemble_with_overrides_uses_injected_values` — fixed UUID and timestamp
2. `assemble_populates_title_from_document_metadata` — title passthrough
3. `assemble_populates_version_from_document_metadata` — version passthrough
4. `assemble_sets_oscal_version_to_1_2_0` — constant check
5. `assemble_generates_valid_uuid_v4` — version nibble and variant check
6. `assemble_generates_utc_timestamp` — timezone check
7. `assemble_two_calls_produce_different_uuids` — uniqueness
8. `assemble_empty_title_passes_through` — edge case EC-1
9. `assemble_default_version_passthrough` — edge case EC-2
10. `assemble_special_characters_in_title` — edge case EC-5
11. `serialization_produces_correct_json_field_names` — serde rename check

### Step 6: Implement

Fill in the function body per the contract in `contracts/oscal-metadata.rs`.

### Step 7: Verify

```bash
cargo test --lib -- oscal::metadata
cargo test --doc
cargo clippy -- -D warnings
cargo fmt --check
```

## Key Files

| File | Action | Purpose |
|------|--------|---------|
| `Cargo.toml` | MODIFY | Add `chrono`, add `v4` to `uuid` features |
| `src/oscal/metadata.rs` | CREATE | All new code lives here |
| `src/oscal/mod.rs` | MODIFY | Declare submodule + re-exports |
| `src/lib.rs` | MODIFY | Re-export key types |

## Common Pitfalls

- **Do NOT** use `Uuid::new_v5()` for artifact UUIDs — that's for stable content IDs (WI-7)
- **Do NOT** use `Local::now()` — must be `Utc::now()` for OSCAL compliance
- **Do NOT** add `#[serde(rename_all = "kebab-case")]` — it would rename `uuid`, `title`, `version` incorrectly
- **Do NOT** call `std::env::var()`, `hostname()`, or read filesystem paths in `assemble_metadata`
- **Do NOT** duplicate metadata logic — there is exactly one `assemble_metadata` function
