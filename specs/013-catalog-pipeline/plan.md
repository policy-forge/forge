# Implementation Plan: End-to-End Catalog Pipeline

**Branch**: `013-catalog-pipeline` | **Date**: 2026-02-12 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/013-catalog-pipeline/spec.md`
**PRD**: [013-prd-catalog-pipeline.md](../../docs/PRD/013-prd-catalog-pipeline.md)
**AR**: [013-ar-catalog-pipeline.md](../../docs/AR/013-ar-catalog-pipeline.md)
**SEC**: [013-sec-catalog-pipeline.md](../../docs/SEC/013-sec-catalog-pipeline.md)

## Summary

Wire all 12 existing pipeline stages (WI-1 through WI-12) into a single `forge convert` command that transforms Markdown policy documents into OSCAL Catalog JSON. Implement a `run_catalog_pipeline` orchestrator function that calls each stage in sequence, add CLI flag handling for `--strategy catalog`, `--format json`, and `--output <path>`, and verify with an end-to-end smoke test. This is the MS-2 milestone exit criteria.

## Technical Context

**Language/Version**: Rust edition 2024, stable 1.93.0
**Primary Dependencies**: clap 4 (CLI), serde 1.x + serde_json 1.x (serialization), pulldown-cmark 0.13 (Markdown parsing), uuid 1.20.0 (ID generation), chrono 0.4 (timestamps), thiserror 2.0.18 (errors), tracing 0.1.44 (logging) -- all existing, no new dependencies
**Storage**: Filesystem (read input .md, optional write output .json)
**Testing**: `cargo test` (unit tests inline, integration tests in `tests/`)
**Target Platform**: macOS/Linux CLI
**Project Type**: Single Rust crate (binary + library)
**Performance Goals**: Deferred to WI-24 (performance benchmarking sprint)
**Constraints**: Policy documents typically <1MB; no new dependencies; OSCAL v1.2.0 output format
**Scale/Scope**: Single-user CLI tool; single document at a time

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Crate-First Architecture | PASS | Single crate (existing). WI-13 adds `src/pipeline.rs` module within existing boundary. No new crates. |
| II. Rust-First | PASS | Pure Rust. No FFI, no unsafe. |
| III. Contract-First | PASS | Pipeline orchestrator interface defined in AR. Contracts in `contracts/pipeline.rs`. |
| IV. Test-First (TDD) | PASS | Smoke test + edge case tests written before implementation. |
| V. Complete Implementation | PASS | All tasks must be done before merge. |
| VI. Performance-First | PASS (deferred) | No specific targets for WI-13. Deferred to WI-24 per PRD/AR. |
| VII. Security-First | PASS | SEC review completed. No unsafe code. Input validated by clap + ingest stage. |
| VIII. Error Handling | PASS | `thiserror` for `ForgeError`. `?` propagation. Add `Serialization` variant. |
| IX. Observability | PASS (minimal) | Existing `tracing` spans on stages. AR says "not yet needed" for additional. |
| X. Simplicity (YAGNI) | PASS | Sequential function composition per AR Option 1. No traits, no dynamic dispatch. |
| XI. Current Dependencies | PASS | No new dependencies. All crates already in `Cargo.toml`. |

No violations. No complexity tracking needed.

## Project Structure

### Documentation (this feature)

```text
specs/013-catalog-pipeline/
├── plan.md              # This file
├── research.md          # Phase 0 output (codebase research)
├── data-model.md        # Phase 1 output (no new entities)
├── quickstart.md        # Phase 1 output (developer guide)
├── contracts/           # Phase 1 output (pipeline interface)
│   └── pipeline.rs      # run_catalog_pipeline + write_output signatures
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── cli/
│   ├── mod.rs           # MODIFY: make --strategy required, --format required
│   └── convert.rs       # MODIFY: dispatch to pipeline orchestrator
├── error.rs             # MODIFY: add Serialization variant
├── pipeline.rs          # NEW: pipeline orchestrator module
├── lib.rs               # MODIFY: add pub mod pipeline
├── ingest/mod.rs        # EXISTING: WI-2 (no changes)
├── parse/
│   ├── mod.rs           # EXISTING: WI-3 (no changes)
│   ├── clauses.rs       # EXISTING: WI-4 (no changes)
│   └── atomize.rs       # EXISTING: WI-6 (no changes)
├── model/
│   ├── mod.rs           # EXISTING: WI-5 (no changes)
│   ├── assemble.rs      # EXISTING: WI-5 (no changes)
│   └── frontmatter.rs   # EXISTING: WI-5 (no changes)
├── oscal/
│   ├── mod.rs           # EXISTING: (no changes)
│   ├── catalog.rs       # EXISTING: WI-9 (no changes)
│   ├── parts.rs         # EXISTING: WI-10 (no changes)
│   ├── metadata.rs      # EXISTING: WI-11 (no changes)
│   └── back_matter.rs   # EXISTING: WI-12 (no changes)
├── uuid.rs              # EXISTING: WI-7 (no changes)
└── main.rs              # EXISTING: (no changes)

tests/
├── common/mod.rs                # EXISTING: test helpers
├── pipeline_test.rs             # EXISTING: WI-5 integration test (no changes)
├── catalog_pipeline_test.rs     # NEW: end-to-end smoke test (WI-13)
├── cli_integration.rs           # MODIFY: add convert command integration tests
└── fixtures/
    ├── sample_policy.md         # EXISTING: basic policy fixture
    └── full_policy.md           # NEW: enhanced fixture (3+ sections, 10+ reqs, compounds)
```

**Structure Decision**: Single Rust crate. WI-13 adds one new module (`src/pipeline.rs`), one new test file, one new fixture, and modifies four existing files (`src/cli/mod.rs`, `src/cli/convert.rs`, `src/error.rs`, `src/lib.rs`). All existing pipeline stage modules are called but NOT modified per AR guardrail.

## Key Design Decisions

### D1: OscalMetadata Type Bridging

**Problem**: Two `OscalMetadata` types exist:
- `oscal::catalog::OscalMetadata` -- placeholder with String fields (used by `OscalCatalog` struct)
- `oscal::metadata::OscalMetadata` -- real type with `Uuid`, `DateTime<Utc>` (from `assemble_metadata`)

**Decision**: Do NOT modify existing types (AR guardrail). In the orchestrator, map from real metadata fields to placeholder string fields when constructing the final `OscalCatalog`.

### D2: WI-8 Citation Extraction (Not Yet Available)

**Problem**: Citation extraction (WI-8) has no extraction function in the codebase. `Citation` type exists but no code populates it.

**Decision**: Wire pipeline with empty citations (`&[]`). Produces `back_matter: None` in output. When WI-8 extraction lands, the pipeline will naturally integrate it. OSCAL Catalog is valid without back matter.

### D3: CLI Flag Requirements

**Current**: `--strategy` is `Option<Strategy>`, `--format` has `default_value = "json"`.
**Decision per spec EC-4/EC-5**: Make both required. Remove `Option` from strategy, remove `default_value` from format. Clap produces descriptive errors when omitted.
**Per S-3**: Validate `--strategy component` → descriptive rejection error in handler.

### D4: Pipeline Module Placement

**Decision**: New `src/pipeline.rs` module. Per AR: "run_catalog_pipeline must be a standalone function (testable without CLI)."

### D5: Serialization Error Variant

**Decision**: Add `ForgeError::Serialization(String)` variant for `serde_json` failures. Replaces misuse of `ForgeError::Parse` for serialization.

## Pipeline Stage Composition

The orchestrator calls these functions in sequence (all signatures verified against codebase):

| Step | Module | Function | Input | Output |
|------|--------|----------|-------|--------|
| 1 | `ingest` | `ingest_file(path, max_bytes)` | `&Path, u64` | `IngestedDocument` |
| 2 | -- | `ingested.reconstruct_content()` | `&IngestedDocument` | `String` |
| 3 | `parse` | `extract_sections(&content)` | `&str` | `Vec<SectionNode>` |
| 4 | `parse` | `extract_clauses(&content)` | `&str` | `ExtractedContent` |
| 5 | `model` | `assemble_document(&ingested, &sections, &clauses)` | refs | `PolicyDocument` |
| 6 | `parse` | `atomize_document(&document)` | `&PolicyDocument` | `PolicyDocument` (new) |
| 7 | `uuid` | `assign_stable_ids(&mut document)` | `&mut PolicyDocument` | `()` (mutates) |
| 8 | -- | *WI-8 stub: empty citations* | -- | `&[]` |
| 9 | `oscal` | `build_catalog(&document)` | `&PolicyDocument` | `OscalCatalog` |
| 10 | `oscal` | `assemble_metadata(&doc.metadata, None)` | `&DocumentMetadata` | `metadata::OscalMetadata` |
| 11 | `oscal` | `generate_back_matter(&[])` | `&[Citation]` | `BackMatter` |
| 12 | -- | Assemble final `CatalogEnvelope` | components | `CatalogEnvelope` |
| 13 | `serde_json` | `to_string_pretty(&envelope)` | `&CatalogEnvelope` | `String` |
| 14 | `pipeline` | `write_output(&json, output_path)` | `&str, Option<&Path>` | `()` |

## Test Strategy

| Test Type | File | What It Validates | Req |
|-----------|------|-------------------|-----|
| Smoke: full pipeline | `tests/catalog_pipeline_test.rs` | Fixture to valid OSCAL JSON | M-7, AC-6 |
| Smoke: group count | `tests/catalog_pipeline_test.rs` | Groups match source sections | AC-1 |
| Smoke: metadata | `tests/catalog_pipeline_test.rs` | title, version, oscal-version, last-modified | AC-2 |
| Smoke: atomization | `tests/catalog_pipeline_test.rs` | Compound reqs split into controls | AC-7 |
| Unit: write_output file | `src/pipeline.rs` | File created with correct content | M-5 |
| Unit: write_output stdout | `src/pipeline.rs` | JSON printed to stdout | M-4 |
| CLI: stdout output | `tests/cli_integration.rs` | forge convert → JSON on stdout | AC-3 |
| CLI: file output | `tests/cli_integration.rs` | --output creates file | AC-4 |
| Edge: missing file | `tests/cli_integration.rs` | Non-zero exit, error msg | EC-1 |
| Edge: empty file | `tests/cli_integration.rs` | Non-zero exit, error msg | EC-2 |
| Edge: bad output dir | `tests/cli_integration.rs` | Non-zero exit, error msg | EC-3 |
| Edge: no --strategy | `tests/cli_integration.rs` | Descriptive error | EC-4 |
| Edge: no --format | `tests/cli_integration.rs` | Descriptive error | EC-5 |
| Edge: no sections | `tests/catalog_pipeline_test.rs` | Empty groups, warning | EC-6 |
| Edge: overwrite file | `tests/cli_integration.rs` | File overwritten | EC-7 |
| Edge: strategy=component | `tests/cli_integration.rs` | Rejection error | S-3 |
