# Quickstart: Performance Benchmark (WI-24)

**Feature Branch**: `024-performance-benchmark`
**Date**: 2026-02-13

## Running Benchmarks

### Full Pipeline Benchmark

```bash
# Run the full pipeline benchmark (50-page synthetic doc → OSCAL JSON)
cargo bench --bench pipeline_benchmark -- full_pipeline

# Expected output:
# full_pipeline/catalog_50page
#                         time:   [X.XXX ms Y.YYY ms Z.ZZZ ms]
```

### Per-Stage Benchmarks

```bash
# Run all per-stage benchmarks
cargo bench --bench pipeline_benchmark -- pipeline_stages

# Run a specific stage
cargo bench --bench pipeline_benchmark -- pipeline_stages/ingest
cargo bench --bench pipeline_benchmark -- pipeline_stages/parse
cargo bench --bench pipeline_benchmark -- pipeline_stages/atomize
cargo bench --bench pipeline_benchmark -- pipeline_stages/catalog_assembly
cargo bench --bench pipeline_benchmark -- pipeline_stages/serialization
```

### All Benchmarks

```bash
# Run all benchmarks (full pipeline + per-stage + existing atomize + uuid)
cargo bench
```

### Existing Benchmarks

```bash
# Atomization benchmarks (WI-6)
cargo bench --bench atomize

# UUID generation benchmarks (WI-7)
cargo bench --bench uuid_benchmark
```

## Viewing Reports

Criterion generates HTML reports at:
```
target/criterion/report/index.html
```

Open in a browser to see:
- Historical performance trends
- Statistical distribution of measurements
- Comparison against previous baselines

## Interpreting Results

### Target
- Full pipeline: **<30 seconds** mean time on commodity hardware (single-core x86-64, 8 GB RAM)

### Regression Detection
Criterion reports changes relative to the last saved baseline:
- `+X.XX%` — performance regression
- `-X.XX%` — performance improvement
- `No change` — within noise threshold

### Saving a Baseline
```bash
# Run benchmarks and save as baseline
cargo bench --bench pipeline_benchmark -- --save-baseline main

# Compare against saved baseline
cargo bench --bench pipeline_benchmark -- --baseline main
```

## Fixture Details

The synthetic 50-page policy fixture is at:
```
tests/fixtures/synthetic-50page-policy.md
```

This is a committed static file (~150KB) containing:
- 10 H2 policy domain sections
- ~40 H3 subsections
- ~200 numbered policy requirements
- ~20 compound statements (for atomization testing)
- ~30 citations/references
- ~10 tables

The fixture is **deterministic** — regenerating it produces byte-identical output.

## Conditional Profiling

If the <30s target is not met, generate a flamegraph:

```bash
# Install cargo-flamegraph (if not already installed)
cargo install flamegraph

# Generate flamegraph for the full pipeline benchmark
cargo flamegraph --bench pipeline_benchmark -- full_pipeline
# Output: flamegraph.svg (open in browser)
```
