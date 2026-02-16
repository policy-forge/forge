# Implementation Plan: YAML Output (WI-27)

**Branch**: `027-yaml-output` | **Date**: 2026-02-15 | **Spec**: [PRD](../../docs/PRD/027-prd-yaml-output.md)
**Input**: PRD `027-prd-yaml-output.md`, AR `027-ar-yaml-output.md`, SEC `027-sec-yaml-output.md`

## Summary

Add YAML output format to `forge convert` using `serde_yaml` (already available as `serde_yaml_ng` v0.10 in `Cargo.toml`). The implementation leverages existing `#[derive(Serialize)]` on OSCAL model structs for zero-additional-model-code serialization to YAML. The `OutputFormat::Yaml` CLI variant already exists. The core work is: (1) removing the non-JSON format guard in `cli/convert.rs`, (2) adding a YAML serializer module in `src/export/`, (3) refactoring the pipeline to accept output format and dispatch serialization, and (4) writing comprehensive tests including semantic equivalence and security verification.

## Technical Context

**Language/Version**: Rust 2024 edition, stable 1.93.0
**Primary Dependencies**: `serde_yaml_ng` 0.10 (aliased as `serde_yaml`, already in `Cargo.toml`), `serde` 1.0.228, `serde_json` 1.0.149
**Storage**: N/A — file output, same pattern as JSON (`write_output` in `pipeline.rs`)
**Testing**: `cargo test`, `insta` (snapshot), `proptest` (property), `tempfile`
**Target Platform**: CLI (macOS, Linux)
**Project Type**: Single Cargo package (not a workspace)
**Performance Goals**: YAML serialization < 100ms for typical policy documents (same order as JSON)
**Constraints**: Semantic equivalence with JSON (PRD M-3), TDD mandatory (constitution IV), `serde_yaml::to_string()` only — no custom YAML formatting (AR guardrail)
**Scale/Scope**: 2 OSCAL model types (Catalog, Component Definition)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Crate-First Architecture | ✅ PASS | YAML serialization lives in existing `export` module within the `forge` crate. ~30 lines of new serialization code does not warrant a separate crate. |
| II. Rust-First | ✅ PASS | Pure Rust, no FFI, no unsafe code. |
| III. Contract-First | ✅ PASS | Interface contracts defined in AR: `serialize_to_yaml<T: Serialize>`, `deserialize_from_yaml<T: DeserializeOwned>`. See `contracts/yaml_serializer.rs`. |
| IV. Test-First | ✅ PASS | TDD mandatory — tests written before implementation. |
| V. Complete Implementation | ✅ PASS | All tasks must complete before merge. |
| VI. Performance-First | ✅ PASS | `serde_yaml::to_string()` is O(n) on model size, same complexity as JSON. Benchmark added for regression detection. |
| VII. Security-First | ✅ PASS | SEC-1 through SEC-5 from security review incorporated as test tasks. No unsafe code. |
| VIII. Error Handling | ✅ PASS | `ForgeError::Serialization` already exists; YAML errors wrapped with descriptive messages. |
| IX. Observability | ✅ PASS | `tracing::debug!` for YAML serialization events, `tracing::info!` for format selection in verbose mode (PRD S-2). |
| X. Simplicity/YAGNI | ✅ PASS | Using `serde_yaml::to_string()` exclusively. No custom YAML formatting. No new abstractions. |
| XI. Current Dependency Policy | ✅ PASS | `serde_yaml_ng` 0.10 already in `Cargo.toml` (used for frontmatter parsing). No new dependency needed. |

**No constitution violations. Gate passed.**

## Project Structure

### Documentation (this feature)

```text
specs/027-yaml-output/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   └── yaml_serializer.rs  # Phase 1 output — interface contract
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
src/
├── cli/
│   ├── mod.rs              # NO CHANGE — OutputFormat::Yaml already exists
│   └── convert.rs          # MODIFY: Remove non-JSON guard, pass format to pipeline
├── export/
│   ├── mod.rs              # MODIFY: Add yaml submodule, re-export serialize functions
│   └── yaml.rs             # NEW: serialize_to_yaml, deserialize_from_yaml
├── pipeline.rs             # MODIFY: Accept OutputFormat param, dispatch serialization
└── oscal/                  # NO CHANGES — model structs remain format-agnostic

tests/
├── yaml_serializer_test.rs    # NEW: Unit tests for YAML serialization
├── yaml_equivalence_test.rs   # NEW: JSON ↔ YAML semantic equivalence
├── yaml_security_test.rs      # NEW: SEC-1 through SEC-4 verification
└── cli_integration.rs         # MODIFY: Add --format yaml CLI integration tests

benches/
└── pipeline_benchmark.rs      # MODIFY: Add YAML serialization benchmarks for both model types
```

**Structure Decision**: Single project (Option 1). YAML serialization adds a small module to the existing `src/export/` directory. No new crates, no new top-level directories.

## Complexity Tracking

No constitution violations to justify.

## Key Findings from Codebase Analysis

### What Already Exists

| Item | Location | Status |
|------|----------|--------|
| `OutputFormat::Yaml` CLI variant | `src/cli/mod.rs:107` | ✅ Already defined |
| `serde_yaml` dependency | `Cargo.toml:13` (`serde_yaml_ng` 0.10) | ✅ Already in Cargo.toml |
| `ForgeError::Serialization` | `src/error.rs:92` | ✅ Already defined |
| `write_output` (format-agnostic) | `src/pipeline.rs:21` | ✅ Writes any string to file/stdout |
| `#[derive(Serialize)]` on OSCAL structs | `src/oscal/catalog.rs`, `src/oscal/component_definition.rs` | ✅ All model types derive Serialize |
| Non-JSON format guard | `src/cli/convert.rs:24` | ❌ Must be removed/updated |
| Empty `src/export/mod.rs` | `src/export/mod.rs` | ❌ Must be populated |
| JSON-only serialization in pipeline | `src/pipeline.rs:152, 221` | ❌ Must accept format parameter |

### Critical Design Decision: `serde_json::Value` in Component Definition

`DocumentaryComponent.control_implementations` is typed as `Vec<serde_json::Value>` (`src/oscal/component_definition.rs:93`). When serializing to YAML via `serde_yaml::to_string()`, `serde_json::Value` implements `Serialize` so it will produce valid YAML output. This has been verified as a non-issue — serde_yaml handles serde_json::Value correctly because both operate on the serde data model.

### Validation Strategy for YAML Output

OSCAL schemas are JSON Schema. The current pipeline:
1. Serializes model → JSON string (`serde_json::to_string_pretty`)
2. Parses JSON string → `serde_json::Value`
3. Validates Value against JSON Schema

For YAML output, the refactored pipeline will:
1. Serialize model → `serde_json::Value` directly (via `serde_json::to_value`)
2. Validate Value against JSON Schema (same validation regardless of output format)
3. If valid, serialize model → requested format string (JSON or YAML)
4. Write output string

This ensures validation is format-independent and always uses JSON Schema.

### Deferred: `forge export` (W-5)

PRD W-5 (formerly M-4) defers `--format yaml` on `forge export` to WI-29, as `forge export` does not exist yet (no `Export` command in CLI). The YAML serializer module built here will be directly reusable when `forge export` is implemented.

## Implementation Phases

### Phase 0: Research

See [research.md](research.md) — all items resolved.

### Phase 1: Design

See [data-model.md](data-model.md) and [contracts/yaml_serializer.rs](contracts/yaml_serializer.rs).

### Phase 2: Tasks

Generated by `/speckit.tasks` command (not part of `/speckit.plan`).

## Artifacts Generated

| Artifact | Path | Description |
|----------|------|-------------|
| Implementation Plan | `specs/027-yaml-output/plan.md` | This file |
| Research | `specs/027-yaml-output/research.md` | Phase 0 findings (all resolved) |
| Data Model | `specs/027-yaml-output/data-model.md` | Entity analysis (no new entities) |
| Interface Contracts | `specs/027-yaml-output/contracts/yaml_serializer.rs` | YAML serializer API contract |
| Quickstart | `specs/027-yaml-output/quickstart.md` | Getting started guide |
