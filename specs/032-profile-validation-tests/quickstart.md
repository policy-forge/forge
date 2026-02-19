# Quickstart: Profile Validation Tests (WI-32)

**Branch**: `032-profile-validation-tests` | **Date**: 2026-02-18

## Prerequisites

- Rust stable 1.93.0+ (`rustup update stable`)
- `cargo` (ships with rustc)
- `cargo-insta` for snapshot review: `cargo install cargo-insta`

## Running Profile Tests

```bash
# Run all profile-related tests
cargo test profile

# Run only schema validation + edge case tests
cargo test --test profile_validation_tests

# Run only golden-file snapshot tests
cargo test --test profile_golden_file_tests

# Run all tests (must pass with zero failures)
cargo test
```

## Working with Insta Snapshots

```bash
# After first run: review and accept new snapshots
cargo insta review

# Accept all pending snapshots without interactive review
cargo insta accept

# After intentional output change: update snapshots
# 1. Delete old .snap files or let cargo insta detect mismatches
# 2. Run tests (they will fail with "snapshot mismatch")
cargo test --test profile_golden_file_tests
# 3. Review and approve changes
cargo insta review
```

## Code Quality Checks

```bash
# Must pass before committing
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

## Schema Validation Details

Profile output is validated against the embedded OSCAL v1.2.0 Profile JSON schema:
- Schema file: `schemas/oscal_profile_schema.json`
- Source: NIST OSCAL v1.2.0 release (`oscal_profile_schema.json`)
- Validation function: `validate_artifact(json, OscalModelType::Profile)`

## Snapshot File Locations

Insta snapshot files are stored in `tests/snapshots/`:
- `profile_golden_file_tests__golden_include_only.snap`
- `profile_golden_file_tests__golden_exclude_only.snap`
- *(parameter tailoring snapshot added when WI-31 is implemented)*

## Scenario Coverage

| Test Name | Type | Status |
|-----------|------|--------|
| `schema_include_only` | Schema validation | Active |
| `schema_exclude_only` | Schema validation | Active |
| `schema_with_set_param` | Schema validation | `#[ignore]` (needs WI-31) |
| `golden_include_only` | Insta snapshot | Active |
| `golden_exclude_only` | Insta snapshot | Active |
| `golden_include_with_params` | Insta snapshot | `#[ignore]` (needs WI-31) |
| `edge_empty_include_list` | Edge case | Active |
| `edge_all_controls_include` | Edge case | Active |
| `edge_conflicting_set_param` | Edge case | `#[ignore]` (needs WI-31) |
| `edge_duplicate_control_ids` | Edge case | Active |
| `edge_both_flags_returns_error` | Edge case | Active |
| `edge_invalid_catalog_path` | Edge case | Active |
| `edge_nonexistent_control_id` | Edge case | Active |
| `e2e_ac12_profile_generation` | End-to-end | Active |
| `load_schema_profile` | Unit (validate mod) | Active |
| `detect_model_type_profile` | Unit (validate mod) | Active |

## Enabling WI-31 Tests

When WI-31 (`--set-param`, `modify` section) is implemented:
1. Remove `#[ignore]` from parameter tailoring tests
2. Run `cargo test --test profile_golden_file_tests`
3. Accept new snapshots with `cargo insta accept`
4. Verify all tests pass

## Troubleshooting

**Snapshot mismatch after code change:**
An intentional change to Profile output requires explicit snapshot approval:
```bash
cargo insta review   # approve or reject the change
```

**Schema validation fails unexpectedly:**
Check that `schemas/oscal_profile_schema.json` exists and is valid JSON:
```bash
python3 -c "import json; json.load(open('schemas/oscal_profile_schema.json'))" && echo "Valid JSON"
```

**`cargo test profile` shows ignored tests:**
This is expected for WI-31-dependent tests. Run `cargo test -- --ignored` to confirm
they fail with a helpful message.
