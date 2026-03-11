# Quickstart: Batch Conversion

## Prerequisites

- Rust 1.93.0+ (stable)
- Existing `forge` crate builds successfully (`cargo build`)
- Test fixtures exist at `tests/fixtures/sample_policy.md`

## Implementation Order

### Phase 1: Data Structures & Pure Functions (no rayon yet)

1. Add `rayon = "1"` to `Cargo.toml` `[dependencies]`
2. Create `src/batch/mod.rs` — module root with re-exports
3. Create `src/batch/summary.rs` — `FileResult` and `BatchSummary` structs
4. Create `src/batch/output_naming.rs` — `derive_output_paths()` pure function
5. Create `src/batch/formatter.rs` — `format_batch_summary()` pure function
6. Register `pub mod batch` in `src/lib.rs`
7. Write unit tests for all pure functions

### Phase 2: CLI Extension

8. Modify `src/cli/mod.rs` — Change `Convert.input: PathBuf` to `Vec<PathBuf>` with `num_args = 1..`; add `--jobs` flag
9. Update existing CLI parse tests
10. Write new CLI parse tests for multi-file and --jobs

### Phase 3: Batch Orchestrator

11. Create `src/batch/orchestrator.rs` — `validate_inputs()`, `run_batch_conversion()` with rayon
12. Modify `src/cli/convert.rs` — Add batch dispatch logic (single vs batch mode)
13. Add `BatchConversion` variant to `ForgeError` in `src/error.rs`
14. Write integration tests for batch mode

### Phase 4: Edge Cases & Polish

15. Implement filename collision avoidance
16. Implement `--output` directory validation and auto-creation
17. Implement >100 file warning
18. Write edge case integration tests
19. Run `cargo clippy -- -D warnings` and `cargo fmt --check`

## Quick Verification

```bash
# After each phase, verify:
cargo test                        # All tests pass
cargo clippy -- -D warnings       # No warnings
cargo fmt --check                 # No formatting issues

# Final verification — batch mode:
cargo run -- convert tests/fixtures/sample_policy.md tests/fixtures/full_policy.md \
  --strategy catalog --format json --output /tmp/batch-test/
ls /tmp/batch-test/               # Should show sample_policy.json, full_policy.json
```

## Key Files to Read Before Starting

1. `src/cli/mod.rs` — Current CLI structure (Convert variant)
2. `src/cli/convert.rs` — Current `execute()` function (single-file)
3. `src/pipeline.rs` — Pipeline functions to wrap (DO NOT MODIFY)
4. `src/error.rs` — Error types and exit codes
5. `src/main.rs` — Entry point and error handling
