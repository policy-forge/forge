# Data Model: Deterministic UUID Generation

**Feature**: 007-uuid-generation | **Date**: 2026-02-11

## Entity Overview

This feature introduces one new constant and three functions in `src/uuid.rs`. It also depends on domain model types from `src/model/mod.rs` (WI-5/WI-6).

## Constants

### FORGE_NAMESPACE_UUID

| Property | Value |
|----------|-------|
| Type | `uuid::Uuid` |
| Visibility | `pub const` |
| Mutability | Compile-time constant (immutable) |
| Purpose | Fixed namespace parameter for all UUID v5 generation |
| Breaking change | Changing this value invalidates ALL previously generated stable_ids |

**Representation**: `Uuid::from_bytes([...16 bytes...])` — a project-specific UUID v4 generated once during development and hardcoded.

## Entities (from WI-5/WI-6 domain model)

### PolicyRequirement

| Field | Type | Source | Description |
|-------|------|--------|-------------|
| text | `String` | WI-6 (atomization) | Raw requirement text extracted from policy document |
| source_line | `usize` | WI-6 (atomization) | 1-based line number in source document |
| nesting_depth | `u8` | WI-6 (atomization) | 0-based nesting depth |
| **stable_id** | **`Option<String>`** | **WI-5 (defined), WI-7 (populated)** | **UUID v5 string, populated by this feature** |

**State transition**:
- Before WI-7: `stable_id = None`
- After WI-7: `stable_id = Some("xxxxxxxx-xxxx-5xxx-yxxx-xxxxxxxxxxxx")`

### PolicySection

| Field | Type | Description |
|-------|------|-------------|
| title | `String` | Section heading text |
| requirements | `Vec<PolicyRequirement>` | Requirements within this section |
| children | `Vec<PolicySection>` | Nested subsections |

### PolicyDocument

| Field | Type | Description |
|-------|------|-------------|
| sections | `Vec<PolicySection>` | Top-level sections with nested hierarchy |

## Data Flow

```text
PolicyRequirement.text (String)
    │
    ▼
normalize_for_hashing(&str) → String
    │  split_whitespace().collect().join(" ")
    │  "  foo   bar  " → "foo bar"
    ▼
generate_stable_id(&str) → Uuid
    │  Uuid::new_v5(&FORGE_NAMESPACE_UUID, normalized.as_bytes())
    ▼
PolicyRequirement.stable_id = Some(uuid.to_string())
```

## Normalization Rules

| Input | Normalized | UUID Same? |
|-------|-----------|------------|
| `"foo bar"` | `"foo bar"` | Baseline |
| `"  foo   bar  "` | `"foo bar"` | Yes ✓ |
| `"foo\tbar"` | `"foo bar"` | Yes ✓ |
| `"foo\n\nbar"` | `"foo bar"` | Yes ✓ |
| `"foo\u{00A0}bar"` (NBSP) | `"foo bar"` | Yes ✓ |
| `""` | `""` | Yes (empty→empty) ✓ |
| `"   "` | `""` | Yes (whitespace-only→empty) ✓ |
| `"Foo bar"` vs `"foo bar"` | Different | No ✗ (case-sensitive) |
| `"foo bar."` vs `"foo bar"` | Different | No ✗ (punctuation-sensitive) |

## Validation Rules

1. `stable_id` must be a valid RFC 4122 UUID v5 string after assignment (SC-005)
2. UUID version nibble must be `5` (byte 6, high nibble)
3. UUID variant bits must indicate RFC 4122 (byte 8, high bits `10`)
4. All `PolicyRequirement`s in a `PolicyDocument` must have `stable_id = Some(...)` after `assign_stable_ids` (SC-004)
5. No `PolicyRequirement` may have `stable_id = None` after processing
