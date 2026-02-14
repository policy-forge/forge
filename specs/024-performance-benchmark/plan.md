# Implementation Plan: Performance Benchmark (WI-24)

**Branch**: `024-performance-benchmark` | **Date**: 2026-02-13 | **Spec**: [PRD](../../docs/PRD/024-prd-performance-benchmark.md)
**Input**: PRD, AR, and SEC from `docs/PRD/`, `docs/AR/`, `docs/SEC/`

## Summary

Establish `criterion`-based benchmark infrastructure to verify the <30s conversion target for a 50-page policy document on commodity hardware. Create a deterministic synthetic fixture (~150KB, ~25,000 words, ~200 requirements), benchmark the full catalog pipeline end-to-end, benchmark each pipeline stage independently for hot-path identification, and integrate benchmarks into CI for regression detection. Optimize hot paths only if the target is not met. This work item is an MS-4 exit criterion blocking the Phase 1 release (WI-25).

## Technical Context

**Language/Version**: Rust edition 2024, stable 1.93.0
**Primary Dependencies**: criterion 0.8.2 (already in dev-deps with `html_reports`), cargo-flamegraph (conditional, only if optimization needed)
**Storage**: Filesystem (read synthetic fixture from `tests/fixtures/`)
**Testing**: `cargo test` (fixture determinism + validity tests), `cargo bench` (criterion benchmarks)
**Target Platform**: Commodity hardware: single-core x86-64, 8 GB RAM, SSD (Linux/macOS)
**Project Type**: Single crate (not workspace)
**Performance Goals**: <30 seconds mean conversion time for full pipeline on 50-page synthetic document
**Constraints**: Deterministic fixture (byte-identical across generations), release-mode benchmarks, no optimization before profiling
**Scale/Scope**: ~150KB fixture, ~25,000 words, ~10 H2 sections, ~200 requirements, ~20 compound statements, ~30 citations, ~10 tables

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Crate-First Architecture | ✅ PASS | Single crate project; benchmark in standard `benches/` directory. No new crate needed — this adds test infrastructure, not production code. |
| II. Rust-First | ✅ PASS | All Rust, no FFI. |
| III. Contract-First | ✅ PASS | Fixture generator interface and benchmark structure defined in AR before implementation. Benchmark function signatures specified in this plan. |
| IV. Test-First (NON-NEGOTIABLE) | ✅ PASS | Tests for fixture determinism (EC-1) and fixture validity (EC-5) written before benchmark code. |
| V. Complete Implementation | ✅ PASS | All Must Have requirements (M-1 through M-5) and Should Have (S-1 through S-3) targeted for completion. |
| VI. Performance-First (NON-NEGOTIABLE) | ✅ PASS | This IS the performance benchmark work item. criterion mandated and already in dev-deps. Benchmarks in `benches/`. |
| VII. Security-First | ✅ PASS | N/A — test infrastructure only. SEC review confirms no security review required. Synthetic fixture contains fabricated content. |
| VIII. Error Handling | ✅ PASS | Benchmark setup errors panic with descriptive messages (appropriate for benchmark harness). No new production error types. |
| IX. Observability | ✅ PASS | criterion provides measurement observability (mean, std dev, throughput, change %). Flamegraph conditional. |
| X. Simplicity & Pragmatism | ✅ PASS | Using established tooling (criterion, cargo-flamegraph). No custom benchmark framework. Measure first, optimize only if needed. |
| XI. Current Dependency Policy | ✅ PASS | criterion 0.8.2 already in Cargo.toml. No new dependencies. |

**Gate Result: PASS** — No violations. Proceed to Phase 0.

## Project Structure

### Documentation (this feature)

```text
specs/024-performance-benchmark/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output (fixture structure)
├── quickstart.md        # Phase 1 output (how to run benchmarks)
├── contracts/           # Phase 1 output (benchmark interfaces)
│   └── benchmark_api.rs # Benchmark function signatures
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
benches/
├── atomize.rs               # Existing (WI-6 benchmarks)
├── uuid_benchmark.rs        # Existing (WI-7 benchmarks)
└── pipeline_benchmark.rs    # NEW: Full pipeline + per-stage benchmarks (M-2, M-5, S-1)

tests/
├── common/
│   ├── mod.rs               # Existing test utilities
│   └── fixture_generator.rs # NEW: Deterministic 50-page fixture generator
├── fixtures/
│   ├── sample_policy.md             # Existing (~631B)
│   ├── full_policy.md               # Existing (~1.4K)
│   └── synthetic-50page-policy.md   # NEW: Committed 50-page fixture (~150KB) (M-1)
├── fixture_determinism_test.rs      # NEW: Verify generator determinism (EC-1)
├── fixture_validity_test.rs         # NEW: Verify fixture produces valid OSCAL (EC-5)
└── ...                              # Existing integration tests

docs/
└── BENCHMARK_RESULTS.md     # NEW: Documented benchmark results (M-4)

Cargo.toml                   # MODIFIED: Add [[bench]] entry for pipeline_benchmark
```

**Structure Decision**: Single-crate project. New benchmark file added to existing `benches/` directory. Fixture generator is a test utility in `tests/common/` (consistent with existing test helpers). Static fixture committed to `tests/fixtures/` (consistent with existing fixtures).

## Complexity Tracking

No constitution violations to justify. All additions follow established patterns.

---

## Phase 0: Research

### R-1: Pipeline Stage Boundaries for Benchmarking

**Decision**: Map the PRD's 6 benchmark stages to actual codebase functions.

The PRD defines these per-stage benchmarks: ingest, parse, atomize, catalog assembly, validation, serialization. However, the codebase has no validation stage (`src/validate/mod.rs` is empty). The actual pipeline from `src/pipeline.rs` (`prepare_document` + `run_catalog_pipeline`) decomposes as:

| PRD Stage | Actual Code | Functions |
|-----------|-------------|-----------|
| **Ingest** | File read + fingerprint + reconstruct | `ingest::ingest_file()` → `IngestedDocument::reconstruct_content()` |
| **Parse** | Section extraction + clause extraction | `parse::extract_sections()` + `parse::extract_clauses()` |
| **Atomize** | Document assembly + atomization + UUID + citations | `model::assemble_document()` → `parse::atomize_document()` → `uuid::assign_stable_ids()` → `citation::extract_citations()` |
| **Catalog Assembly** | OSCAL catalog build + traces + metadata + back matter | `oscal::build_catalog()` → `trace_embedding::embed_trace_in_catalog()` → `oscal::assemble_metadata()` → `oscal::generate_back_matter()` → envelope construction |
| **Validation** | **Empty stub** — no OSCAL schema validation exists | `validate/mod.rs` is empty. Omit from per-stage benchmarks. Document as N/A. |
| **Serialization** | JSON output | `serde_json::to_string_pretty(&envelope)` |

**Rationale**: Benchmarking an empty validation stub provides no value. When validation is implemented (future WI), it should be added to the benchmark suite. The 5 active stages provide complete pipeline coverage.

**Alternatives considered**: (1) Benchmark each of the ~13 individual function calls — rejected as too granular; criterion overhead per benchmark would dominate. (2) Only benchmark full pipeline — rejected; PRD S-1 requires per-stage visibility for hot path identification.

### R-2: Full Pipeline Benchmark — Composing Public Functions

**Decision**: The full pipeline benchmark composes the same public functions as `pipeline::run_catalog_pipeline`, but captures the JSON string result instead of writing to file/stdout.

The `run_catalog_pipeline` function writes output (file or stdout), which is undesirable in a tight benchmark loop. The internal `prepare_document` function is private. However, all individual stage functions are public and accessible from the benchmark crate.

The benchmark will replicate the pipeline steps from `src/pipeline.rs:55-94` (prepare_document) and `src/pipeline.rs:105-158` (run_catalog_pipeline), calling the same public functions in the same order. This avoids modifying production code per the AR guardrail: "Must Not Touch: Pipeline implementation in `src/` (unless optimization is required to meet target)."

**Rationale**: Composing public functions in the benchmark is simpler than adding a new public function to `src/`. It also matches how criterion isolates measurement — no file I/O in the serialization measurement path.

**Alternatives considered**: (1) Add `pub fn convert_catalog_to_string()` to pipeline.rs — rejected per AR guardrail against touching src/. (2) Write output to `/dev/null` or tempfile in benchmark — rejected; adds I/O noise to measurement.

### R-3: Synthetic Fixture Design

**Decision**: Generate a deterministic Markdown document with the following structure (per PRD M-1 and AR):

- YAML frontmatter: title, version, author, date (all hardcoded)
- 10 H2 sections with policy domain names (Access Control, Data Protection, Incident Response, etc.)
- 3-5 H3 subsections per H2 (total ~40 subsections)
- 3-8 numbered requirements per H3 using normative language (total ~200 requirements)
- ~20 compound statements ("must X and must Y") at fixed positions
- ~30 inline citations/references ("[NIST SP 800-53 AC-2]", "See Section 3.2")
- ~10 tables (role-responsibility matrices, retention schedules)
- Target: ~25,000 words / ~150,000 characters (~50 printed pages at ~500 words/page)

The generator function signature: `fn generate_synthetic_policy() -> String`

No page count parameter needed — the PRD only requires a single 50-page fixture. The function uses hardcoded templates with no randomness, no RNG, no `SystemTime`, producing byte-identical output across invocations.

**Rationale**: Fixed structure ensures reproducibility. Realistic content (normative language, citations, tables) exercises all pipeline stages. The generator is simple enough to verify by inspection.

### R-4: Criterion Configuration

**Decision**: Use criterion defaults with one modification.

- **Samples**: 100 (criterion default) — sufficient for stable confidence intervals
- **Warm-up**: 5 seconds (criterion default)
- **Measurement time**: 5 seconds (criterion default)
- **Custom config**: Set `measurement_time` to 10 seconds for the full pipeline benchmark to ensure enough iterations if the pipeline takes >1 second

Baseline storage: `target/criterion/` (gitignored, regenerated per machine). criterion HTML reports generated automatically via the `html_reports` feature already enabled.

**Rationale**: Defaults provide good statistical rigor. Extended measurement time for the full pipeline ensures at least 5+ iterations even if each takes several seconds.

### R-5: CI Integration Approach

**Decision**: Add a GitHub Actions step that runs `cargo bench` after tests pass. Store criterion terminal output in CI logs. Do not store HTML reports as artifacts (keeps CI simple). Do not enforce a hard threshold initially — criterion's regression detection (`+X.XX%` markers) provides visibility.

**Rationale**: PRD S-3 requires benchmarks to run in CI. Keeping it simple (log output only) avoids CI complexity. Threshold enforcement (PRD C-3) is a "Could Have" and can be added later.

---

## Phase 1: Design & Contracts

### Data Model

See `specs/024-performance-benchmark/data-model.md` (generated separately).

**Key entities for this feature:**

| Entity | Description | Notes |
|--------|-------------|-------|
| Synthetic Fixture | ~150KB Markdown file with realistic policy structure | Committed static file in `tests/fixtures/` |
| Fixture Generator | Pure function producing the Markdown string | Test utility in `tests/common/fixture_generator.rs` |
| Full Pipeline Benchmark | Criterion benchmark measuring ingest→serialize | `benches/pipeline_benchmark.rs` |
| Per-Stage Benchmarks | Criterion benchmark group with 5 stage measurements | Same file, separate group |
| Benchmark Results | Hardware + timing + memory documentation | `docs/BENCHMARK_RESULTS.md` |

No new domain model types are introduced. The benchmark operates on existing types: `IngestedDocument`, `PolicyDocument`, `OscalCatalog`, `CatalogEnvelope`.

### Contracts

#### Fixture Generator Interface

```rust
// tests/common/fixture_generator.rs

/// Generate a deterministic 50-page synthetic Markdown policy document.
///
/// Produces ~25,000 words / ~150,000 characters of Markdown with:
/// - YAML frontmatter (title, version, author, date)
/// - 10 H2 sections (Access Control, Data Protection, etc.)
/// - ~40 H3 subsections (3-5 per H2)
/// - ~200 numbered policy requirements (normative language)
/// - ~20 compound statements ("must X and must Y")
/// - ~30 citations/references ("[NIST SP 800-53 AC-2]")
/// - ~10 tables (role-responsibility matrices)
///
/// # Determinism
/// This function uses NO randomness, NO system time, NO RNG.
/// Two calls produce byte-identical output.
pub fn generate_synthetic_policy() -> String {
    // Implementation uses hardcoded templates
    todo!()
}
```

#### Benchmark Interface

```rust
// benches/pipeline_benchmark.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;

/// Full catalog pipeline: ingest → parse → assemble → atomize → IDs →
/// citations → catalog → traces → metadata → back_matter → serialize.
///
/// Reads the committed 50-page fixture from disk.
/// Uses extended measurement time (10s) for stable results.
fn bench_full_pipeline(c: &mut Criterion) {
    let fixture_path = std::path::Path::new("tests/fixtures/synthetic-50page-policy.md");
    assert!(fixture_path.exists(), "Synthetic fixture must exist — run fixture generator first");

    let mut group = c.benchmark_group("full_pipeline");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("catalog_50page", |b| {
        b.iter(|| {
            // Compose all pipeline stages (same as pipeline::run_catalog_pipeline
            // but capturing JSON string instead of writing to file)
            let result = run_full_catalog_pipeline(black_box(fixture_path));
            black_box(result)
        });
    });

    group.finish();
}

/// Per-stage benchmarks: each pipeline stage measured independently.
///
/// Pre-computes input for each stage outside the benchmark loop.
/// Uses black_box() to prevent compiler optimization.
fn bench_per_stage(c: &mut Criterion) {
    let fixture_path = std::path::Path::new("tests/fixtures/synthetic-50page-policy.md");
    let mut group = c.benchmark_group("pipeline_stages");

    // Stage 1: Ingest (file read + SHA-256 + reconstruct)
    group.bench_function("ingest", |b| { /* ... */ });

    // Stage 2: Parse (sections + clauses extraction)
    // Pre-computed input: content string from ingest
    group.bench_function("parse", |b| { /* ... */ });

    // Stage 3: Assemble + Atomize + IDs + Citations
    // Pre-computed input: sections + clauses from parse
    group.bench_function("atomize", |b| { /* ... */ });

    // Stage 4: Catalog Assembly (build + traces + metadata + back matter + envelope)
    // Pre-computed input: PolicyDocument from atomize stage
    group.bench_function("catalog_assembly", |b| { /* ... */ });

    // Stage 5: Serialization (serde_json pretty-print)
    // Pre-computed input: CatalogEnvelope from assembly stage
    group.bench_function("serialization", |b| { /* ... */ });

    group.finish();
}

/// Helper: runs the full catalog pipeline and returns JSON string.
/// Mirrors src/pipeline.rs but without file I/O output.
fn run_full_catalog_pipeline(fixture_path: &std::path::Path) -> Result<String, forge::ForgeError> {
    // Step 1: Ingest
    let ingested = forge::ingest::ingest_file(fixture_path, 10 * 1024 * 1024)?;
    let content = ingested.reconstruct_content();

    // Step 2: Parse
    let sections = forge::parse::extract_sections(&content)?;
    let clauses = forge::parse::extract_clauses(&content)?;

    // Step 3: Assemble + Atomize
    let document = forge::model::assemble_document(&ingested, &sections, &clauses)?;
    let atomized = forge::parse::atomize_document(&document)?;
    let mut doc = atomized;
    forge::uuid::assign_stable_ids(&mut doc);
    forge::citation::extract_citations(&mut doc)?;

    // Step 4: Catalog Assembly
    let mut trace_links = forge::TraceLinkCollection::new();
    let mut catalog = forge::oscal::build_catalog(&doc, Some(&mut trace_links))?;
    forge::oscal::trace_embedding::embed_trace_in_catalog(&mut catalog, &trace_links);
    let metadata = forge::oscal::assemble_metadata(&doc.metadata, None)?;
    let (back_matter_resources, _) = forge::oscal::generate_back_matter(&[])?;
    let back_matter = if back_matter_resources.is_empty() {
        None
    } else {
        Some(forge::BackMatter { resources: back_matter_resources })
    };
    let envelope = forge::oscal::CatalogEnvelope {
        catalog: forge::oscal::OscalCatalog {
            uuid: metadata.uuid.to_string(),
            metadata: forge::oscal::catalog::OscalMetadata {
                title: metadata.title,
                last_modified: metadata.last_modified.to_rfc3339(),
                version: metadata.version,
                oscal_version: metadata.oscal_version,
            },
            groups: catalog.groups,
            back_matter,
        },
    };

    // Step 5: Serialize
    serde_json::to_string_pretty(&envelope)
        .map_err(|e| forge::ForgeError::Serialization(e.to_string()))
}

criterion_group!(benches, bench_full_pipeline, bench_per_stage);
criterion_main!(benches);
```

#### Cargo.toml Addition

```toml
[[bench]]
name = "pipeline_benchmark"
harness = false
```

### Implementation Sequence

The following phases must be executed in order. Each phase has entry criteria (what must be complete before starting) and exit criteria (what must pass before moving to the next phase).

#### Phase A: Fixture Generation (M-1, EC-1, EC-5)

**Entry criteria**: Branch checked out, existing tests pass.

1. **Create fixture generator** (`tests/common/fixture_generator.rs`)
   - Pure function `generate_synthetic_policy() -> String`
   - Hardcoded templates for 10 policy domains
   - ~200 requirements, ~20 compound statements, ~30 citations, ~10 tables
   - No randomness, no system time

2. **Generate and commit the static fixture**
   - Write output to `tests/fixtures/synthetic-50page-policy.md`
   - Verify size is ~150KB (±30KB acceptable range)
   - Commit the file

3. **Write determinism test** (`tests/fixture_determinism_test.rs`)
   - Call generator twice, assert byte-identical output (EC-1)
   - RED: test must fail if generator doesn't exist yet (write test before generator)

4. **Write validity test** (`tests/fixture_validity_test.rs`)
   - Run the committed fixture through the full catalog pipeline
   - Assert it produces valid OSCAL JSON output (EC-5)
   - Assert it produces at least 100 controls (sanity check for scale)

**Exit criteria**: `cargo test` passes including determinism and validity tests.

#### Phase B: Full Pipeline Benchmark (M-2, M-3, M-5)

**Entry criteria**: Phase A complete (fixture committed, tests passing).

1. **Add `[[bench]]` entry** to `Cargo.toml` for `pipeline_benchmark`

2. **Create benchmark file** (`benches/pipeline_benchmark.rs`)
   - Implement `run_full_catalog_pipeline()` helper (composes public pipeline functions)
   - Implement `bench_full_pipeline()` with criterion group (100 samples, 10s measurement)
   - Use `criterion::black_box()` for all inputs and outputs

3. **Run benchmark**: `cargo bench --bench pipeline_benchmark`
   - Record initial mean time, std dev, throughput
   - Verify <30s target on development machine (M-3)

**Exit criteria**: `cargo bench --bench pipeline_benchmark` completes without errors and reports results.

#### Phase C: Per-Stage Benchmarks (S-1)

**Entry criteria**: Phase B complete (full pipeline benchmark working).

1. **Add per-stage benchmark group** to `benches/pipeline_benchmark.rs`
   - Pre-compute input for each stage outside the benchmark loop
   - 5 stage benchmarks: ingest, parse, atomize, catalog_assembly, serialization
   - Each stage uses `black_box()` for measurement accuracy

2. **Run per-stage benchmarks**: `cargo bench --bench pipeline_benchmark`
   - Verify all 5 stages report independently
   - Identify which stage dominates total time

**Exit criteria**: All 5 per-stage benchmarks run and report results.

#### Phase D: Results Documentation (M-4)

**Entry criteria**: Phase C complete (all benchmarks running).

1. **Create benchmark results document** (`docs/BENCHMARK_RESULTS.md`)
   - Hardware description (CPU, RAM, OS, Rust version)
   - Full pipeline: mean time, std dev, throughput
   - Per-stage breakdown: mean time per stage, percentage of total
   - Assessment: <30s target met or not met

**Exit criteria**: Documentation committed with complete benchmark results.

#### Phase E: CI Integration (S-3)

**Entry criteria**: Phase D complete.

1. **Add CI step** to GitHub Actions workflow
   - Run `cargo bench --bench pipeline_benchmark` after tests
   - Capture criterion output in CI logs
   - Do not enforce threshold (informational only)

**Exit criteria**: CI pipeline runs benchmarks and results appear in logs.

#### Phase F: Optimization (S-2, conditional)

**Entry criteria**: Phase B complete AND full pipeline mean time >30 seconds.

**If target is met**: Skip this phase entirely.

**If target is NOT met**:
1. Generate flamegraph: `cargo flamegraph --bench pipeline_benchmark -- --bench`
2. Identify dominant hot path function(s)
3. Apply targeted optimization to the hot path
4. Re-run benchmark to verify improvement
5. Iterate until <30s target is met
6. Update benchmark results documentation

**Exit criteria**: Full pipeline mean time <30 seconds.

#### Phase G: Quality Gates

**Entry criteria**: All prior phases complete.

1. `cargo fmt --check` — no formatting violations
2. `cargo clippy --workspace --all-targets -- -D warnings` — no warnings
3. `cargo test --workspace` — all tests pass
4. `cargo bench --bench pipeline_benchmark` — benchmarks run successfully
5. All Must Have requirements verified (M-1 through M-5)
6. All Should Have requirements verified (S-1 through S-3)

---

## Constitution Check — Post-Design Re-evaluation

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Crate-First | ✅ PASS | No new crate. Benchmark in `benches/`, fixture in `tests/`. |
| II. Rust-First | ✅ PASS | All Rust. |
| III. Contract-First | ✅ PASS | Contracts defined above before implementation. |
| IV. Test-First | ✅ PASS | Determinism test (Phase A.3) and validity test (Phase A.4) written before benchmark code (Phase B). |
| V. Complete Implementation | ✅ PASS | All M and S requirements have implementation phases. |
| VI. Performance-First | ✅ PASS | This is the performance work item. criterion benchmarks established. |
| VII. Security-First | ✅ PASS | N/A confirmed by SEC review. |
| VIII. Error Handling | ✅ PASS | Benchmark panics on setup failures (standard for benchmark harness). |
| IX. Observability | ✅ PASS | criterion reports + flamegraph (conditional). |
| X. Simplicity | ✅ PASS | No custom framework. Standard criterion patterns. Fixture generator is straightforward string concatenation. |
| XI. Dependency Policy | ✅ PASS | criterion 0.8.2 already present. No new deps. |

**Post-Design Gate Result: PASS** — No violations.

---

## Traceability Matrix

| PRD Req | Plan Phase | Deliverable | Verification |
|---------|------------|-------------|--------------|
| M-1 (50-page fixture) | Phase A | `tests/fixtures/synthetic-50page-policy.md` | Fixture determinism test + validity test |
| M-2 (criterion full pipeline) | Phase B | `benches/pipeline_benchmark.rs` `bench_full_pipeline()` | `cargo bench` reports mean/stddev |
| M-3 (<30s target) | Phase B + F | Benchmark results | Mean time <30s in results doc |
| M-4 (documented results) | Phase D | `docs/BENCHMARK_RESULTS.md` | File exists with hardware + timing |
| M-5 (benches/ + cargo bench) | Phase B | `benches/pipeline_benchmark.rs` + `Cargo.toml` `[[bench]]` | `cargo bench` executes |
| S-1 (per-stage benchmarks) | Phase C | `bench_per_stage()` group in pipeline_benchmark.rs | 5 stages report independently |
| S-2 (flamegraph + optimize) | Phase F | Conditional — only if >30s | Flamegraph generated, hot path identified |
| S-3 (CI integration) | Phase E | GitHub Actions benchmark step | CI logs contain criterion output |
| EC-1 (deterministic fixture) | Phase A.3 | `tests/fixture_determinism_test.rs` | Generate twice → byte-identical |
| EC-5 (fixture validity) | Phase A.4 | `tests/fixture_validity_test.rs` | Fixture → valid OSCAL JSON |

## AR Guardrails Compliance

| Guardrail | Plan Compliance |
|-----------|----------------|
| DO NOT use `#[bench]` | Using criterion with `harness = false` |
| DO NOT generate fixture at benchmark time | Static committed file, read from disk |
| DO NOT benchmark with debug profile | `cargo bench` uses release mode by default |
| DO NOT optimize before profiling | Phase F (optimization) is conditional on >30s result |
| DO NOT use random content | Generator uses hardcoded templates, no RNG |
| DO NOT hardcode absolute paths | Using relative path `tests/fixtures/synthetic-50page-policy.md` |
| MUST use `black_box()` | All benchmark inputs and outputs wrapped |
| MUST verify fixture validity | Phase A.4 validity test |
| MUST document results | Phase D `docs/BENCHMARK_RESULTS.md` |
