# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**FORGE** — Framework for OSCAL Risk & Governance Execution

A Rust CLI tool that converts security policies from documents (PDFs, Word docs, Markdown, etc.) into OSCAL (Open Security Controls Assessment Language), the NIST standard for machine-readable security and compliance policies.

- **License**: MIT
- **Organization**: [policy-forge](https://github.com/policy-forge)

## Build Commands

```bash
cargo build                  # Debug build
cargo build --release        # Release build
cargo run -- [args]          # Run with arguments
```

## Testing

```bash
cargo test                   # Run all tests
cargo test <test_name>       # Run a single test by name
cargo test --lib             # Run only library unit tests
cargo test --doc             # Run documentation tests
```

## Linting & Formatting

```bash
cargo fmt                    # Format code
cargo fmt --check            # Check formatting without modifying
cargo clippy                 # Run linter
cargo clippy -- -D warnings  # Treat all warnings as errors
```

## Mutation Testing

```bash
cargo mutants                # Run mutation testing (cargo-mutants must be installed)
```

## Active Technologies
- Rust (edition 2024, stable 1.93.0) + clap 4, thiserror 2.0.18 + NEW: serde 1.x, serde_json 1.x, sha2 0.10.x (002-markdown-ingestion)
- Filesystem (read-only) (002-markdown-ingestion)
- Rust edition 2024, stable 1.93.0 + pulldown-cmark 0.13.x (new), clap 4, thiserror 2.0.18, serde 1.x, serde_json 1.x, sha2 0.10.x (existing) (003-structural-extraction-headings)
- N/A — in-memory processing only (003-structural-extraction-headings)
- Rust (edition 2024, stable 1.93.0) + pulldown-cmark 0.13.x (existing), serde 1.x (existing), thiserror 2.0.18 (existing) (004-structural-extraction-clauses)
- N/A — in-memory processing only; operates on domain model structs (004-structural-extraction-clauses)
- Rust (edition 2024, stable 1.93.0) + serde 1.x (serialization), serde_yaml (YAML frontmatter parsing), thiserror 2.0.18 (error handling), pulldown-cmark 0.13.x (existing from WI-3/4) (005-domain-model)
- N/A (in-memory processing only; no persistent storage) (005-domain-model)
- Rust (edition 2024, stable 1.93.0) + regex (latest stable, already in use), sha2 0.10.x (already a dependency from WI-2), thiserror 2.0.18 (existing error handling) (006-requirement-atomization)
- N/A — in-memory processing only; operates on domain model structs (006-requirement-atomization)
- Rust (edition 2024, stable 1.93.0) + uuid (v5 feature, NEW), tracing (NEW, Constitution IX), pulldown-cmark 0.13.x, serde 1.x, serde_json 1.x, sha2 0.10.x, clap 4, thiserror 2.0.18 (existing) (007-uuid-generation)
- Rust (edition 2024, stable 1.93.0) + serde 1.x, serde_json 1.x, thiserror 2.0.18, tracing 0.1.44, uuid 1.20.0 (all existing) (009-catalog-groups-controls)
- N/A — in-memory processing only; no persistent storage (009-catalog-groups-controls)
- Rust (edition 2024, stable 1.93.0) + `uuid` 1.20.0 (add `v4` feature), `chrono` latest stable (NEW), `serde` 1.x (existing), `thiserror` 2.0.18 (existing), `tracing` 0.1.44 (existing) (011-oscal-metadata)
- Rust (edition 2024, stable 1.93.0) + serde 1.x, serde_json 1.x, thiserror 2.0.18, tracing 0.1.44 (all existing — no new dependencies) (010-catalog-statement-parts)
- N/A (in-memory processing only) (010-catalog-statement-parts)
- Rust edition 2024, stable 1.93.0 + serde 1.x, serde_json 1.x, uuid 1.20.0 (v5 feature, existing), url 2.5.x (NEW), thiserror 2.0.18 (existing), tracing 0.1.44 (existing) (012-back-matter)
- Rust (edition 2024, stable 1.93.0) + regex 1.x (existing), url 2.5.x (existing), uuid 1.20.0 (existing, v5 feature), tracing 0.1.44 (existing), thiserror 2.0.18 (existing), serde 1.x (existing) (008-citation-extraction)
- Rust edition 2024, stable 1.93.0 + clap 4 (CLI), serde 1.x + serde_json 1.x (serialization), pulldown-cmark 0.13 (Markdown parsing), uuid 1.20.0 (ID generation), chrono 0.4 (timestamps), thiserror 2.0.18 (errors), tracing 0.1.44 (logging) -- all existing, no new dependencies (013-catalog-pipeline)
- Filesystem (read input .md, optional write output .json) (013-catalog-pipeline)
- Rust edition 2024, stable 1.93.0 + serde 1.x, serde_json 1.x, uuid 1.20.0 (v4+v5 features), chrono 0.4, thiserror 2.0.18, tracing 0.1.44 -- all existing, no new dependencies (014-component-definition-structure)
- Rust (edition 2024, stable 1.93.0) + uuid 1.20.0 (v5 feature), serde 1.x, serde_json 1.x, clap 4.x, thiserror 2.0.18, tracing 0.1.44 — all existing, no new dependencies (015-component-implemented-requirements)
- Rust edition 2024, stable 1.93.0 + serde 1.x (Serialize/Deserialize), thiserror 2.0.18 (error types), tracing 0.1.44 (logging) -- all existing (016-traceability-model)
- N/A -- in-memory processing only (process lifetime) (016-traceability-model)
- Rust edition 2024, stable 1.93.0 + serde 1.0.228, serde_json 1.0.149, thiserror 2.0.18, tracing 0.1.44, uuid 1.20.0, url 2.5 (all existing -- no new dependencies) (017-traceability-embedding)
- Rust edition 2024, stable 1.93.0 + clap 4.x (CLI), serde 1.x + serde_json 1.x (serialization), pulldown-cmark 0.13.x (Markdown parsing), uuid 1.20.0 (ID generation), chrono 0.4 (timestamps), thiserror 2.0.18 (errors), tracing 0.1.44 (logging) — all existing, no new dependencies (018-component-pipeline)
- Filesystem (read input .md and optional source-profile .json; write output .json) (018-component-pipeline)
- Rust edition 2024, stable 1.93.0 + jsonschema (NEW), serde_json 1.x, clap 4.x, thiserror 2.0.18 (all existing except jsonschema) (019-schema-validation)
- N/A — in-memory processing only; schemas embedded at compile time via `include_str!` (019-schema-validation)
- Rust (edition 2024, stable 1.93.0) + thiserror 2.0.18, clap 4.x, pulldown-cmark 0.13.x, serde 1.x, serde_json 1.x, uuid 1.20.0, chrono 0.4, tracing 0.1.44, sha2 0.10.9, regex 1, url 2.5 (all existing) + NEW: `anyhow` (latest stable, for `.context()` in binary crate) (023-error-handling)
- Rust edition 2024, stable 1.93.0 + criterion 0.8.2 (already in dev-deps with `html_reports`), cargo-flamegraph (conditional, only if optimization needed) (024-performance-benchmark)
- Filesystem (read synthetic fixture from `tests/fixtures/`) (024-performance-benchmark)

## Recent Changes
- 002-markdown-ingestion: Added Rust (edition 2024, stable 1.93.0) + clap 4, thiserror 2.0.18 + NEW: serde 1.x, serde_json 1.x, sha2 0.10.x
