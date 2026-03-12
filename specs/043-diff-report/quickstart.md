# Quickstart: WI-43 Diff Report Implementation

**Date**: 2026-03-12 | **Branch**: `043-diff-report`

## Pre-requisites

```bash
# Verify build environment
cargo build                          # Should compile clean
cargo test                           # All existing tests should pass
cargo clippy -- -D warnings          # No warnings
cargo fmt --check                    # Formatted
```

## File Creation Order

Follow this order strictly (each file depends on the previous):

### Step 1: Data Types
```
src/diff/types.rs
```
Pure data structs/enums. No external dependencies beyond `std`. Write unit tests for `DiffSummary` derivation logic.

### Step 2: Extractor
```
src/diff/extractor.rs
```
Uses `serde_json::Value`. Implements recursive Catalog group traversal and Component Definition path extraction.

Key test: Load a FORGE-generated Catalog JSON and verify controls are extracted at all nesting levels.

### Step 3: Engine
```
src/diff/engine.rs
```
Pure logic: `compare_controls()` and `build_summary()`. No I/O.

Key tests: all AC-* and EC-* scenarios using in-memory `HashMap<String, ControlSnapshot>` fixtures.

### Step 4: Formatter
```
src/diff/formatter.rs
```
Pure string building: `format_diff_report()`. No I/O.

Key test: verify summary line and section headers appear correctly for all change category combinations.

### Step 5: Module wiring
```
src/diff/mod.rs
```
Orchestration: `diff_artifacts()` calls extractor + engine + formatter. Handles file I/O and error wrapping.

Key tests: error cases (missing file, invalid JSON, non-OSCAL, type mismatch).

### Step 6: Error additions
```
src/error.rs  (modify existing)
```
Add `ForgeError::DiffHasChanges` and `ForgeError::DiffError(String)`.
Add `DiffError → 2` and `DiffHasChanges → 1` to `exit_code()`.

### Step 7: Lib re-export
```
src/lib.rs  (modify existing)
```
Add `pub mod diff;`

### Step 8: CLI handler
```
src/cli/diff.rs
```
Thin handler: call `diff_artifacts`, print formatted report, return `Ok(has_changes)`.

### Step 9: CLI dispatch
```
src/cli/mod.rs  (modify existing)
```
Add `Commands::Diff { old_artifact: PathBuf, new_artifact: PathBuf }` variant.
Dispatch: `diff::execute(&old_artifact, &new_artifact).and_then(|has_changes| if has_changes { Err(ForgeError::DiffHasChanges) } else { Ok(()) })`

### Step 10: Main exit code
```
src/main.rs  (modify existing)
```
Add `Err(ForgeError::DiffHasChanges) => ExitCode::from(1u8)` before the generic error arm.

## Test Fixtures

Create minimal in-memory JSON fixtures in test helper functions (no files on disk for unit tests).

Example fixture pattern (from `src/oscal/catalog.rs` test pattern):

```rust
fn make_catalog_json(controls: &[(&str, &str, &str)]) -> serde_json::Value {
    // controls: [(id, title, statement_prose)]
    let controls_json: Vec<_> = controls.iter().map(|(id, title, prose)| {
        serde_json::json!({
            "id": id,
            "title": title,
            "parts": [{"name": "statement", "id": format!("{id}_smt"), "prose": prose}]
        })
    }).collect();
    serde_json::json!({
        "catalog": {
            "uuid": "test-uuid",
            "metadata": {"title": "Test", "last-modified": "2026-01-01T00:00:00Z",
                         "version": "1.0", "oscal-version": "1.2.0"},
            "groups": [{"id": "test", "title": "Test", "controls": controls_json}]
        }
    })
}

fn make_component_def_json(reqs: &[(&str, &str, &str)]) -> serde_json::Value {
    // reqs: [(control_id, uuid, description)]
    let reqs_json: Vec<_> = reqs.iter().map(|(cid, uuid, desc)| {
        serde_json::json!({"uuid": uuid, "control-id": cid, "description": desc})
    }).collect();
    serde_json::json!({
        "component-definition": {
            "uuid": "cd-uuid",
            "metadata": {"title": "Test", "last-modified": "2026-01-01T00:00:00Z",
                         "version": "1.0", "oscal-version": "1.2.0"},
            "components": [{
                "uuid": "comp-uuid", "type": "policy", "title": "Test",
                "description": "Test",
                "control-implementations": [{
                    "uuid": "ci-uuid", "source": "./baseline.json",
                    "description": "Test",
                    "implemented-requirements": reqs_json
                }]
            }]
        }
    })
}
```

For file-based tests (error cases), use `tempfile::NamedTempFile` (already in Cargo.toml).

## Running Tests

```bash
# Run only diff module tests
cargo test diff

# Run with output for debugging
cargo test diff -- --nocapture

# Run all tests (verify no regressions)
cargo test

# Quality gates (must pass before PR)
cargo clippy -- -D warnings
cargo fmt --check
```

## Verifying Exit Codes Manually

```bash
# Build release binary
cargo build --release

# Test exit code 0 (identical files)
./target/release/forge diff tests/fixtures/catalog-v1.json tests/fixtures/catalog-v1.json
echo $?   # Should be 0

# Test exit code 1 (differences found)
./target/release/forge diff tests/fixtures/catalog-v1.json tests/fixtures/catalog-v2.json
echo $?   # Should be 1

# Test exit code 2 (error)
./target/release/forge diff missing.json also-missing.json
echo $?   # Should be 2
```

## Key Anti-patterns to Avoid

- ❌ Do NOT use `json-patch` or any JSON diff library as primary diff mechanism
- ❌ Do NOT match controls by UUID — always use `control_id` as the matching key
- ❌ Do NOT panic on invalid input — always return `DiffError`
- ❌ Do NOT sort inconsistently — always sort by control_id ascending
- ❌ Do NOT use `todo!()` or `unimplemented!()` in delivered code
- ✅ DO handle the empty artifact cases (EC-2, EC-3) — they are valid, not errors
- ✅ DO test the co-occurrence case: UUID changed AND field changed → `Changed { uuid_changed: true }`
