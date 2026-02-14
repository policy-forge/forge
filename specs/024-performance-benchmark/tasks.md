# Tasks: Performance Benchmark (WI-24)

**Input**: Design documents from `/specs/024-performance-benchmark/`
**Prerequisites**: plan.md (required), prd.md (required), ar.md (required), research.md, data-model.md, contracts/benchmark_api.rs, quickstart.md

**Tests**: Included — plan.md explicitly requires fixture determinism test (EC-1) and fixture validity test (EC-5) per constitution principle IV (Test-First).

**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- Exact file paths included in all descriptions

## Path Conventions

- **Single crate project**: `src/`, `tests/`, `benches/` at repository root

---

## Phase 1: Setup

**Purpose**: Configure build infrastructure for the new benchmark target

- [X] T001 Add `[[bench]] name = "pipeline_benchmark" harness = false` entry to `Cargo.toml`

---

## Phase 2: Foundational — Fixture Generation (Blocking Prerequisites)

**Purpose**: Create the deterministic 50-page synthetic fixture and verify it. All user stories depend on this fixture.

**Traces to**: M-1 (50-page fixture), EC-1 (determinism), EC-5 (validity)

**CRITICAL**: No benchmark work (US1, US2, US3) can begin until this phase is complete.

- [X] T002 Create fixture generator function `generate_synthetic_policy() -> String` in `tests/common/fixture_generator.rs` with 10 H2 sections, ~40 H3 subsections, ~200 requirements, ~20 compound statements, ~30 citations, ~10 tables targeting ~150KB output
- [X] T003 Export `fixture_generator` module from `tests/common/mod.rs`
- [X] T004 Generate static fixture by running the generator and committing output to `tests/fixtures/synthetic-50page-policy.md`
- [X] T005 [P] Write fixture determinism test in `tests/fixture_determinism_test.rs` — call `generate_synthetic_policy()` twice, assert byte-identical output (EC-1)
- [X] T006 [P] Write fixture validity test in `tests/fixture_validity_test.rs` — run committed fixture through full catalog pipeline, assert valid OSCAL JSON with at least 100 controls (EC-5)

**Checkpoint**: `cargo test` passes including determinism and validity tests. Fixture file exists at ~150KB.

---

## Phase 3: User Story 1 — Verify Conversion Performance (Priority: P1) MVP

**Goal**: Measure full pipeline conversion time for 50-page document and verify <30s target on commodity hardware.

**Independent Test**: Run `cargo bench --bench pipeline_benchmark -- full_pipeline` and confirm mean time <30s.

**Traces to**: M-2 (criterion full pipeline), M-3 (<30s target), M-4 (documented results), M-5 (benches/ + cargo bench)

### Implementation for User Story 1

- [X] T007 [US1] Implement `run_full_catalog_pipeline(fixture_path) -> Result<String, ForgeError>` helper function in `benches/pipeline_benchmark.rs` composing all public pipeline stages per contract in `specs/024-performance-benchmark/contracts/benchmark_api.rs`
- [X] T008 [US1] Implement `bench_full_pipeline(c: &mut Criterion)` with criterion group `full_pipeline`, 10s measurement time, reading from `tests/fixtures/synthetic-50page-policy.md`, using `black_box()` on all inputs and outputs in `benches/pipeline_benchmark.rs`
- [X] T009 [US1] Add `criterion_group!` and `criterion_main!` macros wiring `bench_full_pipeline` in `benches/pipeline_benchmark.rs`
- [X] T010 [US1] Run `cargo bench --bench pipeline_benchmark -- full_pipeline` and record initial mean time, std dev, and throughput
- [X] T011 [US1] Create `docs/BENCHMARK_RESULTS.md` with hardware description (CPU, RAM, OS, Rust version), full pipeline mean time, std dev, throughput, and <30s target assessment (M-4)

**Checkpoint**: `cargo bench --bench pipeline_benchmark` completes without errors. Mean time documented. <30s target verified.

---

## Phase 4: User Story 2 — Identify and Optimize Hot Paths (Priority: P2)

**Goal**: Measure each pipeline stage independently to identify time distribution and optimize if <30s target is missed.

**Independent Test**: Run `cargo bench --bench pipeline_benchmark -- pipeline_stages` and confirm all 5 stages report independently.

**Traces to**: S-1 (per-stage benchmarks), S-2 (flamegraph + optimization, conditional)

### Implementation for User Story 2

- [X] T012 [US2] Implement `bench_per_stage(c: &mut Criterion)` with benchmark group `pipeline_stages` containing 5 stage benchmarks (ingest, parse, atomize, catalog_assembly, serialization) in `benches/pipeline_benchmark.rs` — pre-compute input for each stage outside the benchmark loop, use `black_box()` on all inputs/outputs
- [X] T013 [US2] Wire `bench_per_stage` into the existing `criterion_group!` macro alongside `bench_full_pipeline` in `benches/pipeline_benchmark.rs`
- [X] T014 [US2] Run `cargo bench --bench pipeline_benchmark -- pipeline_stages` and update `docs/BENCHMARK_RESULTS.md` with per-stage mean times and percentage of total
- [X] T015 [US2] (Conditional — only if full pipeline mean >30s) Generate flamegraph with `cargo flamegraph --bench pipeline_benchmark -- --bench full_pipeline`, identify dominant hot path, apply targeted optimization, re-run benchmark, and update `docs/BENCHMARK_RESULTS.md` (S-2) — SKIPPED: mean is 2ms, well under 30s target

**Checkpoint**: All 5 per-stage benchmarks run and report results. Hot path identified from time distribution.

---

## Phase 5: User Story 3 — Regression Detection in CI (Priority: P2)

**Goal**: Integrate criterion benchmarks into CI pipeline for automated regression detection.

**Independent Test**: Push a commit and verify CI runs `cargo bench` with criterion output in logs.

**Traces to**: S-3 (CI integration)

### Implementation for User Story 3

- [X] T016 [US3] Add benchmark step to `.github/workflows/ci.yml` — run `cargo bench --bench pipeline_benchmark` after the `cargo test` step, capture criterion terminal output in CI logs (informational, no threshold enforcement)

**Checkpoint**: CI pipeline runs benchmarks. Criterion output visible in CI logs.

---

## Phase 6: Polish & Quality Gates

**Purpose**: Final verification that all quality gates pass and deliverables are complete.

**Traces to**: Plan Phase G quality gates, AR guardrails compliance

- [X] T017 [P] Run `cargo fmt --check` and fix any formatting violations
- [X] T018 [P] Run `cargo clippy --workspace --all-targets -- -D warnings` and fix any warnings
- [X] T019 Run `cargo test --workspace` and verify all tests pass (including fixture determinism + validity)
- [X] T020 Run `cargo bench --bench pipeline_benchmark` and verify all benchmarks complete successfully
- [X] T021 Validate `specs/024-performance-benchmark/quickstart.md` commands match actual implementation

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Setup (T001) — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Foundational (Phase 2) completion
- **US2 (Phase 4)**: Depends on US1 (Phase 3) — needs `bench_full_pipeline` and `run_full_catalog_pipeline` helper already in place
- **US3 (Phase 5)**: Depends on US1 (Phase 3) — needs working benchmarks to integrate into CI
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **US1 (P1)**: Depends on foundational fixture only — no dependencies on other stories
- **US2 (P2)**: Depends on US1 — extends `benches/pipeline_benchmark.rs` with per-stage group, needs the full pipeline helper
- **US3 (P2)**: Depends on US1 — needs working benchmarks to add CI step

### Within Phases

- **Phase 2**: T002 → T003 → T004 (generator before export before fixture commit). T005 and T006 can be parallel after T004.
- **Phase 3**: T007 → T008 → T009 sequential (helper → benchmark function → wiring). T010 → T011 sequential (run → document).
- **Phase 4**: T012 → T013 → T014 sequential. T015 is conditional.
- **Phase 6**: T017 and T018 can be parallel. T019 → T020 sequential.

### Parallel Opportunities

```text
Phase 2 (after T004):
  T005 (determinism test) ║ T006 (validity test)

Phase 6:
  T017 (fmt check) ║ T018 (clippy check)
```

---

## Parallel Example: Foundational Phase

```bash
# After T004 (fixture committed), launch tests in parallel:
Task: "Write fixture determinism test in tests/fixture_determinism_test.rs"
Task: "Write fixture validity test in tests/fixture_validity_test.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001)
2. Complete Phase 2: Foundational fixture generation (T002–T006)
3. Complete Phase 3: User Story 1 — full pipeline benchmark (T007–T011)
4. **STOP and VALIDATE**: Run `cargo bench --bench pipeline_benchmark` — verify <30s target met
5. This alone satisfies all Must Have requirements (M-1 through M-5)

### Incremental Delivery

1. Setup + Foundational → Fixture ready, tests passing
2. Add US1 → Full pipeline benchmark with results documentation (MVP!)
3. Add US2 → Per-stage benchmarks for hot path identification
4. Add US3 → CI regression detection
5. Quality gates → Final verification

### Traceability Summary

| PRD Req | Task(s) | Phase |
|---------|---------|-------|
| M-1 (50-page fixture) | T002, T003, T004 | Phase 2 |
| M-2 (criterion full pipeline) | T007, T008 | Phase 3 |
| M-3 (<30s target) | T010 | Phase 3 |
| M-4 (documented results) | T011 | Phase 3 |
| M-5 (benches/ + cargo bench) | T001, T008, T009 | Phase 1, 3 |
| S-1 (per-stage benchmarks) | T012, T013, T014 | Phase 4 |
| S-2 (flamegraph + optimize) | T015 (conditional) | Phase 4 |
| S-3 (CI integration) | T016 | Phase 5 |
| EC-1 (deterministic fixture) | T005 | Phase 2 |
| EC-5 (fixture validity) | T006 | Phase 2 |

---

## Notes

- [P] tasks = different files, no dependencies on incomplete tasks
- [Story] label maps task to specific user story for traceability
- AR guardrail: DO NOT modify `src/` unless optimization is required (T015 only)
- AR guardrail: All benchmarks MUST use `criterion::black_box()` on inputs and outputs
- AR guardrail: DO NOT benchmark with debug profile — `cargo bench` uses release mode by default
- Validation stage omitted from per-stage benchmarks — `src/validate/mod.rs` is empty (research R-1)
- T015 (optimization) is conditional — skip entirely if <30s target is met after T010
- Commit after each task or logical group
- Stop at any checkpoint to validate independently
