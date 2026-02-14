# Implementation Plan: End-to-End Component Definition Pipeline

**Branch**: `018-component-pipeline` | **Date**: 2026-02-13 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/018-component-pipeline/spec.md`
**AR**: [018-ar-component-pipeline.md](../../docs/AR/018-ar-component-pipeline.md)
**SEC**: [018-sec-component-pipeline.md](../../docs/SEC/018-sec-component-pipeline.md)

## Summary

Wire the component-definition pipeline end-to-end so `forge convert policy.md --strategy component` produces a complete OSCAL Component Definition JSON. The pipeline reuses the shared ingest-parse-normalize infrastructure from WI-13 (Catalog pipeline) with a strategy branch point after `PolicyDocument` construction. This is a **completion task**: the codebase is ~85% wired; remaining work focuses on making `--source-profile` optional (S-1), adding file validation (S-2, SEC-3/4), verbose logging (S-3), fixing absolute path leakage (SEC-1), and adding CLI integration tests.

## Technical Context

**Language/Version**: Rust edition 2024, stable 1.93.0
**Primary Dependencies**: clap 4.x (CLI), serde 1.x + serde_json 1.x (serialization), pulldown-cmark 0.13.x (Markdown parsing), uuid 1.20.0 (ID generation), chrono 0.4 (timestamps), thiserror 2.0.18 (errors), tracing 0.1.44 (logging) — all existing, no new dependencies
**Storage**: Filesystem (read input .md and optional source-profile .json; write output .json)
**Testing**: cargo test (unit + integration), TDD mandatory per constitution IV
**Target Platform**: Local CLI tool (macOS, Linux)
**Project Type**: Single Rust binary crate
**Performance Goals**: < 500ms for typical policy documents (startup + pipeline)
**Constraints**: Fully offline operation; JSON output only; no new dependencies
**Scale/Scope**: Single policy document → single Component Definition JSON

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Crate-First Architecture | ✅ Pass | All changes within existing `forge` crate; no new crates needed for pipeline wiring |
| II. Rust-First | ✅ Pass | Pure Rust; no FFI or unsafe code |
| III. Contract-First Development | ✅ Pass | Interfaces defined in AR §Technical Specification; existing types reused |
| IV. Test-First Development | ✅ Pass | TDD workflow mandatory; tests written before implementation |
| V. Complete Implementation | ⏳ Pending | All tasks in tasks.md must be completed before merge |
| VI. Performance-First Design | ✅ Pass | Reuses existing optimized pipeline stages; no new allocations |
| VII. Security-First Design | ✅ Pass | SEC-1 through SEC-7 addressed; file validation added |
| VIII. Error Handling Standards | ✅ Pass | Uses thiserror ForgeError enum; descriptive messages |
| IX. Observability | ✅ Pass | tracing::info! for stage progress; tracing::warn! for missing profile |
| X. Simplicity & Pragmatism | ✅ Pass | Match dispatch (no traits/plugins); reuses existing infrastructure |
| XI. Current Dependency Policy | ✅ Pass | No new dependencies; all existing deps at current versions |

## Project Structure

### Documentation (this feature)

```text
specs/018-component-pipeline/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
├── checklists/
│   └── requirements.md  # Already generated
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
src/
├── cli/
│   ├── mod.rs              # CLI struct with --source-profile, --format default [MODIFY]
│   └── convert.rs          # Strategy dispatch, profile validation [MODIFY]
├── pipeline.rs             # run_component_pipeline — make source_profile optional [MODIFY]
├── oscal/
│   ├── component_definition.rs  # build_component_definition [NO CHANGE — already handles None]
│   ├── implemented_requirements.rs  # [NO CHANGE]
│   └── trace_embedding.rs  # [NO CHANGE]
├── error.rs                # ForgeError variants [NO CHANGE — existing variants sufficient]
└── ...                     # Remaining modules unchanged

tests/
├── component_pipeline_test.rs  # Existing + new tests [MODIFY]
├── cli_integration.rs          # Add component strategy CLI tests [MODIFY]
└── fixtures/
    └── full_policy.md          # Existing fixture [NO CHANGE]
```

**Structure Decision**: Single Rust binary crate. All changes are within the existing module structure. No new files needed — only modifications to 4 source files and 2 test files.

## Complexity Tracking

No constitution violations to justify. The implementation uses match dispatch (not traits), reuses existing infrastructure, and introduces no new dependencies.

---

## Codebase Gap Analysis

### What's Already Done (~85%)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| M-1: `--strategy component` CLI flag | ✅ Done | `Strategy::Component` in `src/cli/mod.rs:67` |
| M-2: `--source-profile <path>` flag | ✅ Done | `source_profile: Option<String>` in `src/cli/mod.rs:55` |
| M-3: Wire full pipeline chain | ✅ Done | `run_component_pipeline()` in `src/pipeline.rs:170` calls `prepare_document()` then `build_component_definition()` |
| M-4: Documentary component with control-implementations | ✅ Done | `build_component_definition()` in `src/oscal/component_definition.rs:112` |
| M-5: OSCAL metadata fields | ✅ Done | `assemble_metadata()` reused from WI-11 in component_definition.rs:128 |
| M-6: Traceability props/links | ✅ Done | `build_trace_props()` + `build_trace_link()` in implemented_requirements.rs:115-116 |
| M-7: Back matter resources | ✅ Done | `generate_back_matter()` in component_definition.rs:178 |
| M-8: JSON to stdout/file | ✅ Done | `write_output()` in `src/pipeline.rs:21` |

### What Needs to Change (~15%)

| Gap | Spec Req | Files | Change |
|-----|----------|-------|--------|
| **G-1**: `--source-profile` is required; spec says optional | S-1, AC-7, SEC-6 | `cli/convert.rs`, `pipeline.rs` | Make `source_profile` optional; emit warning when omitted |
| **G-2**: No file validation for `--source-profile` | S-2, SEC-3, SEC-4, AC-8 | `cli/convert.rs` | Validate path exists, is regular file, is readable |
| **G-3**: No verbose pipeline stage logging | S-3 | `pipeline.rs` | Add `tracing::info!()` for each stage |
| **G-4**: Absolute path may leak into OSCAL output | SEC-1, AR guardrail | `pipeline.rs` | Use filename-only for source_file prop |
| **G-5**: `--format` is required; spec EC-1 says default JSON | EC-1 | `cli/mod.rs` | Add `default_value = "json"` to clap |
| **G-6**: No CLI integration tests for component strategy | Testing | `cli_integration.rs` | Add end-to-end CLI tests via forge binary |
| **G-7**: Existing CLI test asserts `--source-profile is required` | S-1 change | `cli_integration.rs` | Update test to assert warning, not error |

---

## Phase 0: Research

### R-1: Optional `--source-profile` threading

**Decision**: Change `run_component_pipeline` signature from `source_profile: &str` to `source_profile: Option<&str>`. The downstream `build_component_definition` already accepts `Option<&str>` — when None, it produces an empty `control-implementations` array. The CLI layer emits a `tracing::warn!()` to stderr when omitted.

**Rationale**: The lower-level API already handles the None case. Only the pipeline function signature and CLI handler need updating.

**Alternatives considered**: Adding a sentinel value (e.g., empty string) — rejected because it bypasses type safety.

### R-2: Source profile file validation

**Decision**: Add file validation in `cli/convert.rs` before calling the pipeline. Validate: (1) path exists, (2) is a regular file (not directory/symlink), (3) is readable. Do NOT parse JSON at CLI level — profile parsing belongs in the pipeline (per AR anti-pattern guidance). However, since the current implementation treats `--source-profile` as a string reference (not parsed), and W-3 defers profile resolution, JSON parsing validation is deferred to a future WI that actually parses the profile content.

**Rationale**: AR §Anti-patterns says "Don't parse the source profile eagerly at CLI argument parsing time". SEC-3 requires existence validation; SEC-4 requires JSON parsability. Since the current pipeline doesn't parse the profile (it uses the path as a reference string), SEC-4 compliance is partial — the path is validated, but content parsing is deferred to a future WI.

**Alternatives considered**: Full JSON deserialization at CLI level — rejected per AR anti-pattern guidance.

### R-3: Absolute path mitigation

**Decision**: Use `input_path.file_name()` (filename only) for the `source_file` parameter in `build_component_definition`. This prevents absolute filesystem paths from appearing in the generated OSCAL `props`. Users who need path context can inspect the metadata title (derived from frontmatter).

**Rationale**: SEC-1 says "shall not leak absolute filesystem paths beyond what the user explicitly provides". Using filename-only is the most conservative approach.

**Alternatives considered**: Relative path calculation — more complex, requires reference directory, deferred.

### R-4: `--format` default value

**Decision**: Add `default_value = "json"` to the `--format` clap argument. This makes `--format` optional while maintaining the current behavior when explicitly specified. EC-1 requires that omitting `--format` defaults to JSON.

**Rationale**: Simple one-line clap attribute change. All existing tests that specify `--format json` continue to work unchanged.

**Alternatives considered**: Making `format` an `Option<OutputFormat>` and defaulting in code — rejected as more complex; clap `default_value` is idiomatic.

---

## Phase 1: Design & Contracts

### Data Model

No new types needed. All changes operate on existing types:

- `PolicyDocument` — unchanged (shared pipeline output)
- `ComponentDefinitionEnvelope` — unchanged (build_component_definition output)
- `ForgeError` — unchanged (existing variants sufficient)
- `Strategy::Component` — unchanged
- `OutputFormat` — unchanged, but default value added

### Interface Changes

#### `src/pipeline.rs` — `run_component_pipeline`

```rust
// BEFORE:
pub fn run_component_pipeline(
    input_path: &Path,
    output_path: Option<&Path>,
    max_size_bytes: u64,
    source_profile: &str,          // required
) -> Result<(), ForgeError>

// AFTER:
pub fn run_component_pipeline(
    input_path: &Path,
    output_path: Option<&Path>,
    max_size_bytes: u64,
    source_profile: Option<&str>,  // optional
) -> Result<(), ForgeError>
```

Internal changes:
- Remove empty-string validation (None handles the case)
- Use `source_file` as filename-only (SEC-1)
- Add `tracing::info!()` for pipeline stages (S-3)
- Pass `source_profile` directly to `build_component_definition` (already accepts Option)

#### `src/cli/convert.rs` — `execute`

```rust
// BEFORE:
Strategy::Component => {
    let profile = match source_profile {
        None => return Err(ForgeError::Validation("--source-profile is required...")),
        Some(p) if p.trim().is_empty() => return Err(ForgeError::Validation("...")),
        Some(p) => p,
    };
    crate::pipeline::run_component_pipeline(input, output, max_size_bytes, profile)
}

// AFTER:
Strategy::Component => {
    let profile_ref = match source_profile {
        None => {
            tracing::warn!("--source-profile not provided; control-id mapping will be skipped. ...");
            None
        }
        Some(p) if p.trim().is_empty() => {
            return Err(ForgeError::Validation("--source-profile must not be empty".to_string()));
        }
        Some(p) => {
            // Validate file exists and is a regular file (SEC-3)
            let path = std::path::Path::new(p);
            if !path.exists() { return Err(ForgeError::Validation("...")); }
            if !path.is_file() { return Err(ForgeError::Validation("...")); }
            Some(p)
        }
    };
    crate::pipeline::run_component_pipeline(input, output, max_size_bytes, profile_ref)
}
```

#### `src/cli/mod.rs` — `OutputFormat` default

```rust
// BEFORE:
#[arg(long)]
format: OutputFormat,

// AFTER:
#[arg(long, default_value = "json")]
format: OutputFormat,
```

### Test Contracts

New tests to write (TDD — tests first):

| Test ID | Location | Verifies | Spec Req |
|---------|----------|----------|----------|
| T-S1-01 | `cli/convert.rs` | Component strategy with `source_profile: None` succeeds | S-1, AC-7 |
| T-S1-02 | `cli_integration.rs` | CLI `--strategy component` without `--source-profile` produces output + warning on stderr | S-1, SEC-6 |
| T-S1-03 | `component_pipeline_test.rs` | Pipeline with `source_profile: None` produces Component Definition with empty control-implementations | S-1, AC-7 |
| T-S2-01 | `cli/convert.rs` | Component strategy with non-existent `--source-profile` path errors | S-2, SEC-3, AC-8 |
| T-S2-02 | `cli/convert.rs` | Component strategy with directory as `--source-profile` errors | SEC-3 |
| T-S2-03 | `cli_integration.rs` | CLI with non-existent `--source-profile` exits non-zero | AC-8 |
| T-S3-01 | `pipeline.rs` | Component pipeline emits tracing events for stage progress | S-3 |
| T-SEC1 | `component_pipeline_test.rs` | Source-file prop does not contain absolute path | SEC-1 |
| T-EC1 | `cli_integration.rs` | `--format` omitted defaults to JSON | EC-1 |
| T-EC1-02 | `cli/mod.rs` | Clap parses without `--format` and defaults to Json | EC-1 |
| T-UPD-01 | `cli_integration.rs` | Update existing test that expects `--source-profile is required` to expect warning instead | S-1 change |

### Quickstart

After implementation:

```bash
# Full pipeline with source profile
forge convert policy.md --strategy component --source-profile baseline.json --format json

# Without source profile (produces empty control-implementations + warning)
forge convert policy.md --strategy component --format json

# Default format (JSON inferred)
forge convert policy.md --strategy component --source-profile baseline.json

# Output to file
forge convert policy.md --strategy component --source-profile baseline.json --output out.json
```

---

## Implementation Order

Based on AR §Suggested Implementation Order, adapted to the gap analysis:

1. **Make `--format` optional** (G-5, EC-1) — smallest change, unblocks test updates
2. **Make `--source-profile` optional in pipeline** (G-1, S-1) — signature change + warning
3. **Add source-profile file validation** (G-2, S-2, SEC-3) — input validation in CLI
4. **Fix absolute path leakage** (G-4, SEC-1) — use filename-only for source_file
5. **Add verbose pipeline stage logging** (G-3, S-3) — tracing::info! calls
6. **Update existing tests** (G-7) — fix test that expects required source-profile
7. **Add new CLI integration tests** (G-6) — component strategy via forge binary
8. **Add new unit/integration tests** — TDD for all changes

---

## AR Implementation Guardrails Compliance

- [x] **DO NOT** duplicate ingest → parse → normalize stages — reuses `prepare_document()` ✅
- [x] **DO NOT** create a Strategy trait or plugin registry — uses match expression ✅
- [x] **DO NOT** ignore missing `--source-profile` — emits warning to stderr ✅ (G-1 fixes this)
- [x] **DO NOT** embed absolute file paths in OSCAL output — uses filename only ✅ (G-4 fixes this)
- [x] **MUST** validate `--source-profile` path exists and is readable — ✅ (G-2 adds this)
- [x] **MUST** produce complete Component Definition with metadata, back matter, trace props — ✅ (already done)
- [x] **MUST** support `--output <path>` and default to stdout — ✅ (already done)

## SEC Security Requirements Compliance

| SEC Req | Status | Implementation |
|---------|--------|----------------|
| SEC-1 | ✅ Fixed by G-4 | Use `input_path.file_name()` for source_file prop |
| SEC-2 | ✅ Already done | Error messages use ForgeError with user-provided paths only |
| SEC-3 | ✅ Fixed by G-2 | Validate `--source-profile` path exists and is a regular file |
| SEC-4 | ⚠️ Partial | Path validated; content parsing deferred (profile not parsed by current design) |
| SEC-5 | ✅ Already done | `write_output()` validates parent directory existence |
| SEC-6 | ✅ Fixed by G-1 | Warning emitted to stderr when `--source-profile` omitted |
| SEC-7 | ✅ Already done | `main.rs` exits with code 1 on any error |
