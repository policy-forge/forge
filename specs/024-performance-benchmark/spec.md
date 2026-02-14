# Feature Specification: Performance Benchmark (WI-24)

**Feature Branch**: `024-performance-benchmark`
**Created**: 2026-02-13
**Status**: Draft
**Input**: End-to-end pipeline performance benchmarking with per-stage breakdown

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Verify Conversion Target (Priority: P1)

As a developer, I run `cargo bench --bench pipeline_benchmark -- full_pipeline` against the 50-page synthetic fixture to confirm the full catalog pipeline completes within the 30-second target on commodity hardware.

**Why this priority**: The <30s conversion target is an MS-4 exit criterion blocking the Phase 1 release (WI-25). This is the primary deliverable.

**Independent Test**: Run `cargo bench --bench pipeline_benchmark -- full_pipeline` and verify mean time is reported below 30 seconds.

**Acceptance Scenarios**:

1. **Given** the synthetic 50-page fixture exists at `tests/fixtures/synthetic-50page-policy.md`, **When** I run the full pipeline benchmark, **Then** Criterion reports a mean conversion time under 30 seconds.
2. **Given** the benchmark has been run before, **When** I run it again, **Then** Criterion reports the change percentage relative to the previous run.

---

### User Story 2 - Identify Pipeline Hotspots (Priority: P2)

As a developer, I run per-stage benchmarks to identify which pipeline stages consume the most time, so I know where to focus optimization if the target is not met.

**Why this priority**: Without per-stage breakdown, optimization would be guesswork. This enables data-driven performance work.

**Independent Test**: Run `cargo bench --bench pipeline_benchmark -- pipeline_stages` and verify each stage reports individual timing.

**Acceptance Scenarios**:

1. **Given** the synthetic fixture exists, **When** I run per-stage benchmarks, **Then** I see individual timings for ingest, parse, atomize, catalog assembly, and serialization stages.

---

### User Story 3 - Detect Performance Regressions in CI (Priority: P3)

As a maintainer, I see benchmark results in CI so that performance regressions are caught before merging.

**Why this priority**: Without CI integration, regressions can silently ship. Lower priority because local benchmarking (P1) already validates the target.

**Independent Test**: Push a commit and verify the CI workflow runs the benchmark step successfully.

**Acceptance Scenarios**:

1. **Given** a push to a branch with the benchmark workflow, **When** CI runs, **Then** the benchmark step executes and reports results.

### Edge Cases

- What happens when the synthetic fixture file is missing? Benchmark panics with a descriptive assertion message.
- What happens when a pipeline stage returns an error? The benchmark propagates the error via `Result` instead of panicking silently.
- How does the benchmark handle noisy CI environments? Criterion uses statistical analysis (confidence intervals) to distinguish real regressions from noise.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST measure full pipeline throughput (iterations per second) for a 50-page synthetic policy document
- **FR-002**: System MUST record per-stage latency breakdown (ingest, parse, atomize, catalog assembly, serialization)
- **FR-003**: System MUST complete the full pipeline in under 30 seconds mean time on commodity hardware (single-core x86-64, 8 GB RAM)
- **FR-004**: System MUST export benchmark results via Criterion HTML reports for historical comparison
- **FR-005**: System MUST detect performance regressions by comparing against saved baselines

### Key Entities

- **Synthetic Fixture**: A deterministic ~158KB Markdown policy document with ~200 requirements, ~30 citations, and ~10 tables, committed at `tests/fixtures/synthetic-50page-policy.md`
- **Pipeline Stage**: One of five measured phases (ingest, parse, atomize, catalog assembly, serialization) that compose the full catalog pipeline
- **CatalogEnvelope**: The final OSCAL JSON output produced by the full pipeline, used as the serialization benchmark target

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Full pipeline mean conversion time is under 30 seconds on commodity hardware (single-core x86-64, 8 GB RAM)
- **SC-002**: Per-stage breakdown accounts for >95% of total pipeline time (sum of stages closely matches full pipeline measurement)
- **SC-003**: Synthetic fixture is deterministic — regenerating produces byte-identical output
- **SC-004**: All existing tests (498+) continue to pass with the benchmark infrastructure added
