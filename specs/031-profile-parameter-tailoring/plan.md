# Implementation Plan: Profile Parameter Tailoring

**Branch**: `031-profile-parameter-tailoring` | **Date**: 2026-02-18 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/031-profile-parameter-tailoring/spec.md`

## Summary

WI-31 extends the WI-30 OSCAL Profile builder with parameter tailoring support. A repeatable `--set-param <id> <value>` CLI flag is added to `forge profile`. When provided, the generated Profile includes a `modify.set-parameters` array with parameter override entries — aggregated by param-id and alphabetically ordered for deterministic output. When absent, the Profile is identical to WI-30 output (no `modify` section). Two pre-existing compile errors from WI-33/WI-34 model additions must be fixed first. No new dependencies are required.

## Technical Context

**Language/Version**: Rust stable 1.93.0, Edition 2024
**Primary Dependencies**: clap 4.x (derive), serde 1.0.228, serde_json 1.0.149, tracing 0.1.44 — all already in `Cargo.toml`
**Storage**: N/A — in-memory; writes OSCAL JSON to local filesystem (unchanged from WI-30)
**Testing**: `cargo test`, `insta` for snapshot tests (already in dev-dependencies)
**Target Platform**: macOS/Linux CLI binary
**Project Type**: Single project (existing Cargo workspace — not multi-crate)
**Performance Goals**: CLI tool; no explicit latency SLA; BTreeMap aggregation is O(n log n) on param count
**Constraints**: No new dependencies; backward compatible with WI-30 output; `cargo clippy -- -D warnings` must pass
**Scale/Scope**: Single command invocation; no concurrent use case

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Gate | Principle | Status | Notes |
|------|-----------|--------|-------|
| Crate-First Architecture | I | ✅ PASS | Extending `oscal/profile.rs`; scope too small for a new crate; existing module has a clear single responsibility |
| Rust-First | II | ✅ PASS | Pure safe Rust; no FFI; no `unsafe` |
| Contract-First | III | ✅ PASS | AR-031 defines `Modify`, `SetParameter`, `build_modify_section` signature, and CLI flag before implementation |
| Test-First (TDD) | IV | ✅ REQUIRED | Tests must be written before implementation for each task; failing tests confirmed before proceeding |
| Complete Implementation | V | ✅ REQUIRED | All FR-001–009 and acceptance criteria must pass before merge |
| Performance-First | VI | ✅ PASS | BTreeMap operation is non-hot-path; CLI startup time unaffected |
| Security-First | VII | ✅ PASS | Param values treated as opaque strings; no catalog I/O; clap handles input parsing |
| Error Handling | VIII | ✅ PASS | Uses `ForgeError::InvalidArgument`, `ForgeError::Serialization`; `thiserror` pattern maintained |
| Observability | IX | ✅ PASS | `#[tracing::instrument]` on `build_modify_section`; INFO log on param count in `execute` |
| Simplicity | X | ✅ PASS | Two new structs + one new function + field extensions; no premature abstraction |
| Current Dependency Policy | XI | ✅ PASS | No new dependencies; all crates already in Cargo.toml |

**Pre-merge quality gates**:
```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test --workspace
cargo test --doc
```

## Project Structure

### Documentation (this feature)

```text
specs/031-profile-parameter-tailoring/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   └── rust-api.md      # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (modified files only)

```text
src/
├── cli/
│   ├── mod.rs           # [MODIFY] Add set_params Vec<String> to Commands::Profile
│   └── profile.rs       # [MODIFY] Add set_params param; parse_set_param_pairs; C-2 warn
├── oscal/
│   └── profile.rs       # [MODIFY] Add Modify, SetParameter; extend OscalProfile; build_modify_section; extend build_profile
└── parse/
    ├── atomize.rs        # [FIX] Doctest struct literal missing modality + parameters
    └── modality.rs       # [FIX] Test helper req() missing parameters field

tests/                   # [MODIFY] Integration tests for --set-param CLI behavior
```

**Structure Decision**: Single project (Option 1). No new files or modules — all changes are additive modifications to existing files. Consistent with WI-30's extension pattern and the AR-031 decision.

---

## Phase 0: Research

> Research complete. See [`research.md`](research.md) for full findings.

### Key Decisions

| Decision | Outcome |
|----------|---------|
| clap arg pattern | `num_args = 2, action = ArgAction::Append, value_names = ["PARAM_ID", "VALUE"]` on `Vec<String>` |
| Aggregation strategy | `BTreeMap<String, Vec<String>>` — alphabetical ordering for free |
| Return type | `Option<Modify>` (strongly-typed, not `serde_json::Value`) — consistent with codebase |
| Pre-existing compile errors | Fix first: `atomize.rs` doctest + `modality.rs` test helper |
| C-2 warning behavior | `eprintln!` + `tracing::warn!` to stderr; continue; exit 0 |
| Serde annotation | `#[serde(skip_serializing_if = "Option::is_none")]` on `OscalProfile::modify` |

---

## Phase 1: Design & Contracts

> Design complete. See [`data-model.md`](data-model.md) and [`contracts/rust-api.md`](contracts/rust-api.md).

### Implementation Sequence

The following order satisfies compile-time dependencies and TDD discipline:

**Task 0 — Fix pre-existing compile errors** *(prerequisite)*
- `src/parse/atomize.rs`: Add `modality: None, parameters: vec![]` to doctest struct literal
- `src/parse/modality.rs`: Add `parameters: vec![]` to `req()` test helper
- Verify: `cargo build` succeeds

**Task 1 — New serde types: `Modify` and `SetParameter`** *(TDD: doc-test first)*
- Write `build_modify_section` doctests before implementation
- Add `Modify` and `SetParameter` structs with serde renames to `src/oscal/profile.rs`
- Add `build_modify_section` function skeleton (return `todo!()`)
- Confirm doctests fail (red)

**Task 2 — Implement `build_modify_section`**
- Write unit tests for: empty input, single pair, multiple pairs, duplicate param-ids, space in value, empty-string value, ten params, alphabetical ordering
- Implement function body using `BTreeMap`
- Confirm all unit tests pass (green)

**Task 3 — Extend `OscalProfile` with optional `modify`**
- Add `modify: Option<Modify>` field with `skip_serializing_if`
- Update `build_profile` signature to accept `param_overrides: &[(String, String)]`
- Update `build_profile` body to call `build_modify_section` and assign `modify`
- Update all existing `build_profile` callers (tests + `cli/profile.rs`) to pass `&[]`
- Confirm existing tests still pass (green — backward compat)

**Task 4 — CLI: add `--set-param` to `Commands::Profile`**
- Write CLI parse tests: `--set-param prm1 val1` parses correctly; two `--set-param` flags; space-in-value
- Add `set_params: Vec<String>` with `#[arg(long = "set-param", num_args = 2, action = clap::ArgAction::Append, value_names = ["PARAM_ID", "VALUE"])]` to `Commands::Profile` in `src/cli/mod.rs`
- Update `Commands::Profile` dispatch in `execute` to pass `&set_params` to `profile::execute`
- Confirm CLI parse tests pass (green)

**Task 5 — Update `cli/profile.rs execute`**
- Write integration tests for `execute` with `--set-param` (JSON output contains `modify`)
- Add `set_params: &[String]` parameter to `execute`
- Add `parse_set_param_pairs` helper
- Implement C-2 warning logic (warn when params non-empty + no include/exclude)
- Pass pairs to `build_profile`
- Confirm integration tests pass (green)

**Task 6 — Integration and regression tests**
- Verify JSON shape matches OSCAL contract for all edge cases (EC-1 through EC-6)
- Verify backward compat: no `--set-param` produces no `"modify"` key
- Verify determinism: same inputs twice → identical output string
- Snapshot test for `modify` section shape (insta)

---

## Gates (Post-Design Re-evaluation)

All constitution gates remain ✅ PASS. The design adds the minimum needed: two structs, one function, and field extensions. No new crates, no new abstractions, no new dependencies.

**Complexity Tracking**: No violations. Not applicable.
