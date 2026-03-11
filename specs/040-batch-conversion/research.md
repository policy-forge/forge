# Research: Batch Conversion (Phase 0)

## R1: rayon Integration Pattern

**Decision**: Use `rayon::par_iter()` with a custom `ThreadPoolBuilder` to control parallelism via `--jobs`.

**Rationale**: rayon provides work-stealing thread pool with `par_iter()` as a drop-in replacement for `iter()`. Custom `ThreadPoolBuilder::new().num_threads(jobs).build()` allows per-invocation thread pool control without affecting the global pool.

**Alternatives considered**:
- `std::thread` manual pool: More boilerplate, no work-stealing. Rejected per AR-040.
- `tokio` async runtime: Over-engineered for CPU-bound work. Rejected per AR-040.
- Global rayon pool with `rayon::ThreadPoolBuilder::new().num_threads(n).build_global()`: Mutates global state; not testable in parallel tests. Use scoped pool instead.

**Key pattern**:
```rust
let pool = rayon::ThreadPoolBuilder::new()
    .num_threads(jobs)
    .build()
    .map_err(|e| ForgeError::BatchConversion(format!("Failed to create thread pool: {e}")))?;

pool.install(|| {
    input_paths.par_iter()
        .map(|input| convert_single_file(input, ...))
        .collect::<Vec<FileResult>>()
})
```

## R2: catch_unwind for Panic Isolation

**Decision**: Wrap each per-file pipeline invocation in `std::panic::catch_unwind`.

**Rationale**: The existing pipeline uses `unwrap()` in some internal paths. A panic in one file's conversion would terminate the rayon thread and potentially abort the entire batch. `catch_unwind` converts panics into `Err` values.

**Constraint**: The closure passed to `catch_unwind` must be `UnwindSafe`. Since we only pass owned/cloned data (`PathBuf`, `String`), this is satisfied.

**Pattern**:
```rust
let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
    pipeline_convert(input, output, ...)
}));
match result {
    Ok(Ok(())) => FileResult::success(input, output, duration),
    Ok(Err(e)) => FileResult::failure(input, e.to_string(), duration),
    Err(_) => FileResult::failure(input, "Internal error (panic during conversion)".into(), duration),
}
```

## R3: Output Filename Derivation with Collision Avoidance

**Decision**: `{stem}.{format_ext}` with `_{n}` suffix on collision (n starts at 2).

**Rationale**: Deterministic, predictable naming. Collision avoidance handles same-name files from different directories (e.g., `dir1/policy.md` and `dir2/policy.md` → `policy.json` and `policy_2.json`).

**Algorithm**: Pre-compute all output paths before processing. Iterate inputs in order; for each, check if the candidate path is already claimed. If so, increment suffix until unique. This is O(n²) in worst case but n is bounded practically.

## R4: clap Multi-Value Positional Argument

**Decision**: Change `input: PathBuf` to `input: Vec<PathBuf>` with `#[arg(num_args = 1..)]`.

**Rationale**: clap 4.x supports `num_args = 1..` for positional arguments, requiring at least one value. This preserves backward compatibility (single file still works) while enabling multiple files.

**Impact on existing tests**: Existing CLI parse tests reference single `input: PathBuf`. These must be updated to unpack `Vec<PathBuf>`.

## R5: Aggregated Status Format

**Decision**: Human-readable table format on stderr.

**Format**:
```
Batch conversion complete: 3 files (2 succeeded, 1 failed) in 1.23s

  ✓ policy1.md → output/policy1.json (0.45s)
  ✓ policy2.md → output/policy2.json (0.38s)
  ✗ policy3.md — Parse error: no policy structure detected (0.40s)
```

**Rationale**: Clear, scannable. Per-file lines sorted by input filename for deterministic order. Summary line first for quick overview. Check/cross marks for visual scanning. Duration per file aids debugging slow conversions.

## R6: Backward Compatibility Strategy

**Decision**: When `input.len() == 1`, delegate directly to existing `convert::execute()` with no behavioral changes.

**Rationale**: The simplest way to guarantee zero regression is to not change the single-file code path at all. Batch mode (2+ files) is a separate branch.

**Key invariant**: Single-file mode still writes to stdout when `--output` is not specified. Batch mode always writes to files (auto-generated names or `--output` directory).

## R7: --jobs Flag Validation

**Decision**: clap `value_parser` with `1..=256` range. Default: 0 (meaning "use all available cores").

**Rationale**: 0 as default maps to `num_cpus::get()` (rayon default). Upper bound of 256 prevents absurd values per SEC R6 clarification. clap enforces the range at parse time.
