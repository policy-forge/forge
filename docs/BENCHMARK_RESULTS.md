# Benchmark Results (WI-24)

## Hardware Description

| Parameter | Value |
|-----------|-------|
| CPU | Apple M4 |
| RAM | 16 GB |
| OS | macOS 26.2 (Darwin) |
| Rust | rustc 1.93.0 (254b59607 2026-01-19) |
| Profile | release (default for `cargo bench`) |
| Criterion | 0.8.2 |

## Full Pipeline Benchmark

**Fixture**: `tests/fixtures/synthetic-50page-policy.md` (~158 KB, ~200 requirements)

**Pipeline**: ingest -> parse -> assemble -> atomize -> IDs -> citations -> catalog -> traces -> metadata -> back_matter -> serialize

| Metric | Value |
|--------|-------|
| Mean | 1.9957 ms |
| Std Dev | ~0.037 ms (range: 1.9781 ms - 2.0155 ms) |
| Throughput | ~500 iterations/sec |
| Outliers | 4/100 (3 high mild, 1 high severe) |

### <30s Target Assessment

**PASS** -- The full pipeline completes in ~2 ms, which is **~15,000x faster** than the 30-second target for commodity hardware. The target is met with substantial margin.

## Per-Stage Benchmarks

| Stage | Mean | % of Total |
|-------|------|-----------|
| ingest | 352.95 us | 18.2% |
| parse | 339.73 us | 17.5% |
| atomize | 481.43 us | 24.9% |
| catalog_assembly | 339.87 us | 17.6% |
| serialization | 430.65 us | 22.2% |
| **Sum of stages** | **1,944.63 us** | **~100%** |

### Stage Analysis

- **Hottest stage**: atomize (24.9%) -- includes `assemble_document()`, `atomize_document()`, `assign_stable_ids()`, and `extract_citations()`
- **Second**: serialization (22.2%) -- `serde_json::to_string_pretty()` on the full OSCAL envelope
- **Third**: ingest (18.2%) -- file I/O + `reconstruct_content()`
- **Near-equal**: parse and catalog_assembly (~17.5% each)

The pipeline time is evenly distributed across stages with no single dominant bottleneck. The sum of per-stage means (~1.94 ms) closely matches the full pipeline mean (~2.00 ms), confirming minimal overhead between stages.

### Optimization Assessment

No optimization required. The full pipeline mean (2 ms) is well under the 30s target. Flamegraph profiling (S-2) is **skipped** per the conditional rule in the task plan.

## Running Benchmarks

```bash
# Full pipeline benchmark
cargo bench --bench pipeline_benchmark -- full_pipeline

# Per-stage benchmarks
cargo bench --bench pipeline_benchmark -- pipeline_stages

# All benchmarks
cargo bench --bench pipeline_benchmark
```

Criterion HTML reports are generated in `target/criterion/` after each run.
