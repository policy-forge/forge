# Research: Performance Benchmark (WI-24)

**Feature Branch**: `024-performance-benchmark`
**Date**: 2026-02-13

## R-1: Pipeline Stage Boundaries for Benchmarking

**Decision**: Map PRD's 6 benchmark stages to 5 active pipeline stages (validation is an empty stub).

**Rationale**: `src/validate/mod.rs` is empty — no OSCAL schema validation exists. Benchmarking a no-op provides no value. The 5 active stages (ingest, parse, atomize, catalog assembly, serialization) cover 100% of actual pipeline work.

**Alternatives considered**:
- Benchmark all ~13 individual functions — too granular; criterion overhead would dominate measurements for fast functions
- Only benchmark full pipeline — insufficient; PRD S-1 requires per-stage hot path identification

## R-2: Full Pipeline Benchmark Composition

**Decision**: Compose public pipeline functions in the benchmark rather than modifying `src/pipeline.rs`.

**Rationale**: AR guardrail states "Must Not Touch: Pipeline implementation in src/ (unless optimization is required to meet target)." All stage functions (`ingest_file`, `extract_sections`, `extract_clauses`, `assemble_document`, `atomize_document`, `assign_stable_ids`, `extract_citations`, `build_catalog`, `embed_trace_in_catalog`, `assemble_metadata`, `generate_back_matter`) are public. The private `prepare_document` and I/O-performing `run_catalog_pipeline` can be replicated from public functions.

**Alternatives considered**:
- Add `pub fn convert_catalog_to_string()` — rejected per AR guardrail
- Use `run_catalog_pipeline` with tempfile output — rejected; adds I/O noise

## R-3: Synthetic Fixture Design

**Decision**: Hardcoded deterministic Markdown using 10 policy domains with realistic structure.

**Rationale**: Fixed templates (no RNG, no system time) guarantee byte-identical output (EC-1). Realistic content (normative language, citations, tables, compound statements) exercises all pipeline stages. Target ~150KB matches ~50 printed pages at ~500 words/page.

**Alternatives considered**:
- Use a real policy document — rejected (licensing, sensitivity concerns per AR Decision Log)
- Runtime-generated fixture — rejected (adds benchmark overhead, harder to reproduce per AR)
- Parameterized page count — rejected (PRD only requires 50-page; YAGNI per constitution X)

## R-4: Criterion Configuration

**Decision**: Use defaults (100 samples, 5s warm-up) with 10s measurement time for full pipeline.

**Rationale**: Extended measurement time for the full pipeline ensures sufficient iterations even if each takes 1-5 seconds. Per-stage benchmarks use defaults since individual stages are faster.

**Alternatives considered**:
- Reduce sample count for faster CI — rejected; 100 samples is reasonable for stable CI
- Custom statistical configuration — rejected (YAGNI; defaults provide sufficient rigor)

## R-5: CI Integration Strategy

**Decision**: Log-based CI integration. Run `cargo bench` in CI, capture terminal output in logs. No hard threshold enforcement initially.

**Rationale**: PRD S-3 requires CI execution. Threshold enforcement (C-3) is "Could Have." Criterion's built-in `+X.XX%` markers provide regression visibility without custom tooling. CI runners have variable performance, making hard thresholds unreliable (PRD R-3 risk).

**Alternatives considered**:
- Store HTML reports as CI artifacts — adds complexity, not required by PRD
- Enforce 30s threshold in CI — unreliable on shared runners; deferred to C-3
- Use `criterion-compare` GitHub Action — adds dependency; criterion output is sufficient
