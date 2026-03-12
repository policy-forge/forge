# Implementation Plan: oscal-cli Round-Trip Validation

**Branch**: `037-oscal-cli-round-trip` | **Date**: 2026-03-12 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/037-oscal-cli-round-trip/spec.md`, PRD [037-prd-oscal-cli-round-trip](../../docs/PRD/037-prd-oscal-cli-round-trip.md), AR [037-ar-oscal-cli-round-trip](../../docs/AR/037-ar-oscal-cli-round-trip.md), SEC [037-sec-oscal-cli-round-trip](../../docs/SEC/037-sec-oscal-cli-round-trip.md)

---

## Summary

Extend the FORGE codebase with a `round_trip` library module and integration test suite that converts FORGE-generated OSCAL artifacts through an oscal-cli chain (JSON → XML → YAML → JSON), compares the result semantically against the original, and writes a structured JSON divergence log. Builds on the oscal-cli subprocess infrastructure from WI-36 (AR-036) — no new production dependencies required.

---

## Technical Context

**Language/Version**: Rust 1.93.0, Edition 2024
**Primary Dependencies**: `serde_json` 1.0.149, `serde` 1.0.228 (derive), `thiserror` 2.0.18, `tracing` 0.1.44 — all already in `Cargo.toml`. `tempfile` 3.25.0 (dev-dep, already present).
**Storage**: Local filesystem — temporary intermediate files (XML, YAML) in a `tempfile::TempDir`; divergence log written to a configurable path (default: `divergences.json`).
**Testing**: `cargo test` — unit tests (no oscal-cli required) + integration tests (conditional on oscal-cli availability via runtime `PathDetector`). `insta` snapshots for divergence log output.
**Target Platform**: Linux/macOS (developer workstation + CI runner).
**Project Type**: Single Rust crate.
**Performance Goals**: 30-second per-invocation timeout for each oscal-cli subprocess call; three calls per full round-trip cycle.
**Constraints**: No new production dependencies. Temporary files cleaned up via RAII (`TempDir` drop). Integration tests skip gracefully when oscal-cli is unavailable.
**Scale/Scope**: 2 OSCAL artifact types (Catalog, Component Definition); 3-step conversion chain; divergence log written per test run.

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Crate-First Architecture | ✅ Pass | New `src/round_trip/` module within existing crate; no new crates |
| II. Rust-First | ✅ Pass | Stable Rust throughout; no `unsafe` |
| III. Contract-First | ✅ Pass | Rust interface contracts defined in `contracts/round_trip.rs` before tasks are written |
| IV. Test-First (TDD) | ✅ Pass | Unit tests for comparator/divergence types written before implementation; integration tests before chain code |
| V. Complete Requirement Delivery | ✅ Pass | All M-* requirements have tasks with acceptance coverage |
| VI. Performance & Scope Discipline | ✅ Pass | 30s timeout is measurable; W-3 (benchmarking) explicitly out of scope |
| VII. Security-First | ✅ Pass | SEC-037 requirements SEC-2 through SEC-6 mapped to explicit tasks |
| VIII. Error Handling | ✅ Pass | Each conversion step returns `ForgeError`; actionable, testable messages |
| IX. Observability | ✅ Pass | DEBUG logging per conversion step; INFO summary; structured divergence JSON |
| X. Simplicity & Pragmatism | ✅ Pass | Custom 50-line tree walker; extends existing `OscalCliInvoke` trait; no new frameworks |
| XI. Dependency Policy | ✅ Pass | No new production deps; `tempfile` already in dev deps |

**Post-constitution check**: No violations. No complexity justification table required.

---

## Project Structure

### Documentation (this feature)

```text
specs/037-oscal-cli-round-trip/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── round_trip.rs    # Rust interface contract
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
src/
├── round_trip/
│   ├── mod.rs           # Public re-exports; module documentation
│   ├── divergence.rs    # Divergence, DivergenceClass, RoundTripResult structs/enums
│   ├── rules.rs         # OscalComparisonRules (unordered array paths: props, links, parts)
│   ├── comparator.rs    # compare_oscal_json — OSCAL-aware recursive Value comparison
│   ├── chain.rs         # run_round_trip_chain — oscal-cli 3-step conversion chain
│   └── log.rs           # write_divergence_log — JSON file writer
└── oscal_cli/
    └── mod.rs           # Extend: add ConvertArgs, ConvertResult, convert() to OscalCliInvoke
    └── invoker.rs       # Extend ProcessInvoker: implement convert()

tests/
└── oscal_cli_round_trip.rs   # Integration tests (conditional on oscal-cli availability)
```

**Structure Decision**: Single project (Option 1). All new code lives within the existing Rust crate. The `round_trip` module is a new library module alongside existing modules (`oscal_cli`, `validate`, `batch`, etc.). Integration tests follow the established `tests/*.rs` pattern with `tempfile::TempDir` for temp file management.

---

## Phase 0: Research

*See [research.md](research.md) for full findings. Summary below.*

| Question | Decision | Rationale |
|----------|----------|-----------|
| JSON comparison algorithm | Custom recursive `serde_json::Value` tree walker | OSCAL-specific unordered-array rules (`props`, `links`, `parts`) require custom logic; no `assert_json_diff` dependency (resolved in clarification Q3) |
| Unordered OSCAL arrays | `props`, `links`, `parts` | Confirmed in clarification Q2; these are the fields most commonly reordered by oscal-cli conversions |
| Divergence log format | JSON file at configurable path (default `divergences.json`) | Machine-readable; enables automated tracking (C-2); `serde_json` already present (resolved in clarification Q1) |
| Subprocess timeout | 30 seconds per invocation | JVM cold-start ~2–5s; 30s gives headroom on slow CI (resolved in clarification Q4) |
| oscal-cli `convert` command | `oscal-cli convert --to=<format> <input> <output>` | Confirmed by oscal-cli documentation and WI-36 integration pattern |
| Unordered array element identity key | `uuid` field; fall back to `name` field; fall back to position | OSCAL elements with `uuid` are uniquely identifiable; props use `name`+`ns` as composite key |

---

## Phase 1: Design & Contracts

### Data Model

*See [data-model.md](data-model.md) for full entity descriptions.*

**New types (in `src/round_trip/`)**:

| Type | Location | Description |
|------|----------|-------------|
| `Divergence` | `divergence.rs` | Single difference: json_path, expected, actual, classification, description |
| `DivergenceClass` | `divergence.rs` | Enum: ForgeFix / OscalCliDiff / Acceptable |
| `RoundTripResult` | `divergence.rs` | Aggregate: artifact_type, source_path, passed, divergences Vec |
| `OscalComparisonRules` | `rules.rs` | Config: unordered_array_paths, ignored_paths |

**Extended types (in `src/oscal_cli/`)**:

| Type | Location | Description |
|------|----------|-------------|
| `ConvertArgs` | `mod.rs` | input_path, output_path, output_format, timeout |
| `ConvertResult` | `mod.rs` | output_path (written by oscal-cli) |
| `OscalFormat` | `mod.rs` | Enum: Json / Xml / Yaml (for `--to=` flag) |

**Extended trait** (`OscalCliInvoke` in `mod.rs`):
```rust
fn convert(&self, args: &ConvertArgs) -> Result<ConvertResult, ForgeError>;
```

### API Contracts

*See [contracts/round_trip.rs](contracts/round_trip.rs) for full Rust interface.*

Key public surface:
```rust
// Core comparison — unit-testable, no oscal-cli required
pub fn compare_oscal_json(
    expected: &serde_json::Value,
    actual: &serde_json::Value,
    path: &str,
    rules: &OscalComparisonRules,
) -> Vec<Divergence>

// Conversion chain — requires oscal-cli
pub fn run_round_trip_chain(
    input_json_path: &Path,
    invoker: &dyn OscalCliInvoke,
    temp_dir: &Path,
    timeout: Duration,
) -> Result<PathBuf, ForgeError>

// Divergence log writer
pub fn write_divergence_log(
    result: &RoundTripResult,
    output_path: &Path,
) -> Result<(), ForgeError>
```

### Agent Context Update

Agent context updated via `.specify/scripts/bash/update-agent-context.sh claude`.

---

## Implementation Sequence

The AR-037 suggested implementation order drives task sequencing:

1. **Data structures** — `Divergence`, `DivergenceClass`, `RoundTripResult`, `OscalComparisonRules`
2. **OSCAL comparison rules** — populate `unordered_array_paths` (`props`, `links`, `parts`); `ignored_paths` (none initially)
3. **Comparator** — `compare_oscal_json` with recursive tree walk, OSCAL-aware unordered array matching; full unit test suite
4. **oscal-cli convert extension** — `ConvertArgs`/`ConvertResult`/`OscalFormat` types; extend `OscalCliInvoke` trait; implement in `ProcessInvoker`
5. **Conversion chain** — `run_round_trip_chain` orchestrating JSON→XML→YAML→JSON via `OscalCliInvoke`
6. **Divergence log writer** — `write_divergence_log` serializing `RoundTripResult` to JSON file
7. **Integration tests** — Catalog round-trip (conditional on oscal-cli); Component Definition round-trip; classify and document any discovered divergences
8. **FORGE fix pass** — resolve any discovered FORGE-caused divergences (FR-005 / M-5 / SC-001 / SC-002)
9. **Three-format validation** — confirm JSON→XML→YAML→JSON round-trip passes (S-1 / FR-007 / SC-003)

---

## Security Task Mapping (SEC-037)

| SEC Req | Implementation Task |
|---------|---------------------|
| SEC-2 | `chain.rs`: Create temp dir via `tempfile::tempdir()` (unique name, not fixed path) |
| SEC-3 | `chain.rs`: `TempDir` used as RAII handle — drops and cleans up even on panic |
| SEC-4 | `tempfile::tempdir()` creates directories with 0o700 permissions on Unix by default |
| SEC-5 | Integration tests: verify graceful skip when oscal-cli unavailable (test in `oscal_cli_round_trip.rs`) |
| SEC-6 | `chain.rs`: Pass 30-second `timeout` to each `ConvertArgs` independently (per-invocation, not shared) |

All subprocess security (SEC-1 / argument arrays / env filtering / sanitize) inherited from AR-036 `ProcessInvoker` with no changes required.

---

## Key Algorithmic Decisions

### Unordered Array Matching

For paths in `OscalComparisonRules::unordered_array_paths`:
1. Try to match elements by `uuid` field (OSCAL primary identity key)
2. Fall back to `name` field for props (where `name`+`ns` form the composite key)
3. Fall back to positional comparison if no identity key found

This avoids false positives when oscal-cli reorders props/links/parts during conversion.

### Divergence Classification

Initial classification: all divergences are reported as `ForgeFix`. After investigation, divergences caused by oscal-cli behavior (e.g., oscal-cli omits empty arrays, adds default fields) are reclassified as `OscalCliDiff` or `Acceptable`. The comparator emits raw divergences; classification is applied by the integration test investigator.

### Integration Test Conditional Execution

```rust
fn skip_if_no_oscal_cli() -> Option<ProcessInvoker> {
    let detector = PathDetector;
    let info = detector.detect();
    if !info.available || !info.functional {
        eprintln!("SKIP: oscal-cli not available ({:?})", info);
        return None;
    }
    let path = info.executable_path.unwrap();
    Some(ProcessInvoker::new(path))
}
```

Tests call this helper and return early (not fail) if `None`.

---

## Validation Criteria (post-implementation)

All of the following must pass before the feature is complete:

- [ ] `cargo test` — all unit tests green (comparator, divergence types, log writer)
- [ ] `cargo test --test oscal_cli_round_trip` — integration tests pass (or skip gracefully) when oscal-cli available
- [ ] `cargo clippy -- -D warnings` — zero warnings
- [ ] `cargo fmt --check` — zero violations
- [ ] SC-001: Catalog JSON → XML → JSON round-trip: zero unresolved FORGE-caused divergences
- [ ] SC-002: Component Definition JSON → XML → JSON round-trip: zero unresolved FORGE-caused divergences
- [ ] SC-003: Catalog + Component Definition JSON → XML → YAML → JSON round-trip: zero unresolved FORGE-caused divergences
- [ ] SC-004: All divergences documented in `divergences.json` with classification + resolution status
- [ ] SC-005: Tests skip cleanly when oscal-cli unavailable (verified via unit test with mock invoker)
