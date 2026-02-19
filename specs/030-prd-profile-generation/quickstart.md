# Quickstart: 030-prd-profile-generation

## Environment

- Rust stable 1.93.0 (Rust edition 2024)
- `cargo` in `PATH`
- Branch: `030-prd-profile-generation`

## Build & Test

```bash
cargo build              # Verify compilation
cargo test               # All tests must pass
cargo clippy -- -D warnings
cargo fmt --check
```

## What Gets Added

| File | Action | Description |
|------|--------|-------------|
| `src/oscal/profile.rs` | CREATE | Profile types + `build_profile` + `parse_control_ids` |
| `src/oscal/mod.rs` | MODIFY | Add `pub mod profile;` + re-exports |
| `src/cli/profile.rs` | CREATE | `forge profile` subcommand handler |
| `src/cli/mod.rs` | MODIFY | Add `Profile { ... }` variant + dispatch |
| `tests/profile_generation_test.rs` | CREATE | Integration tests |

## Example Usage

```bash
# Include specific controls
forge profile --catalog catalog.json --include POL-AC-001,POL-AC-002

# Exclude specific controls
forge profile --catalog catalog.json --exclude POL-AC-003

# Write to file
forge profile --catalog catalog.json --include POL-AC-001 --output baseline.json

# Help
forge profile --help
```

## Expected Output

```json
{
  "profile": {
    "uuid": "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
    "metadata": {
      "title": "Policy Baseline Profile",
      "last-modified": "2026-09-22T10:00:00Z",
      "version": "1.0.0",
      "oscal-version": "1.2.0"
    },
    "imports": [
      {
        "href": "catalog.json",
        "include-controls": [
          {
            "with-ids": ["POL-AC-001", "POL-AC-002"]
          }
        ]
      }
    ]
  }
}
```

## Key Constraints

- `--include` and `--exclude` are **mutually exclusive** — providing both is an error
- The catalog path is stored **as-is** in `href` — not read, not normalized
- Control IDs are **trimmed** and **deduplicated** (order preserved)
- Profile generation does **not** read or validate the source catalog file
