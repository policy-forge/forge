# Quickstart: Summary Dashboard (044)

## What This Feature Does

Adds a `--summary` flag to `forge convert` that prints a formatted statistics dashboard to stdout after conversion completes.

## Files to Create

1. **`src/summary/mod.rs`** — `ConversionStatistics` struct, `ValidationStatus` enum, counting helpers
2. **`src/summary/format.rs`** — `format_summary_dashboard()`, ANSI color helpers, box-drawing output, `format_elapsed()`
3. **`tests/summary_dashboard_test.rs`** — Unit and integration tests

## Files to Modify

1. **`src/lib.rs`** — Add `pub mod summary;`
2. **`src/cli/mod.rs`** — Add `--summary` flag to `Convert` variant
3. **`src/cli/convert.rs`** — Wire stats collection, elapsed timing, dashboard printing
4. **`src/pipeline.rs`** — Change `run_catalog_pipeline` and `run_component_pipeline` return types from `Result<(), ForgeError>` to `Result<ConversionStatistics, ForgeError>`

## Implementation Order

1. Create `src/summary/mod.rs` with `ConversionStatistics`, `ValidationStatus`, `Default` impls, `mapping_coverage()`, counting helpers
2. Write unit tests for `mapping_coverage()` (zero, partial, full, >100%)
3. Create `src/summary/format.rs` with `format_summary_dashboard()` and `format_elapsed()`
4. Write unit tests for formatting (snapshot tests with insta)
5. Add `pub mod summary;` to `src/lib.rs`
6. Add `--summary` flag to CLI in `src/cli/mod.rs`
7. Modify `src/pipeline.rs` to return `ConversionStatistics` from both pipeline functions
8. Update `src/cli/convert.rs` to pass `--summary` flag, measure elapsed time, and print dashboard
9. Update existing tests that match on `Ok(())` from pipeline functions
10. Write integration test: `forge convert --summary` produces dashboard output
11. Write integration test: `forge convert` (without `--summary`) produces no dashboard

## Build & Verify

```bash
cargo test                           # All tests pass
cargo clippy -- -D warnings          # No warnings
cargo fmt --check                    # Formatting clean
```

## Expected Dashboard Output

```
┌─────────────────────────────────────────┐
│        FORGE Conversion Summary         │
├─────────────────────────────────────────┤
│ Strategy:           catalog             │
│ Output:             output/catalog.json │
│ Elapsed:            0.42s               │
├─────────────────────────────────────────┤
│ Sections parsed:    12                  │
│ Requirements:       47                  │
│ Controls generated: 47                  │
│ Mapping coverage:   100.0% (47/47)     │
├─────────────────────────────────────────┤
│ Validation:         PASSED              │
└─────────────────────────────────────────┘
```
