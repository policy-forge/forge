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

## Dependencies

Rust edition 2024 with MSRV 1.85. The version ranges below mirror
`Cargo.toml`; `Cargo.lock` is authoritative for exact resolved versions.

### Production

| Crate | Version | Purpose |
|-------|---------|---------|
| clap | 4 | CLI argument parsing (derive) |
| pulldown-cmark | 0.13 | Markdown event-stream parsing |
| regex | 1 | Pattern matching (atomization, citations) |
| serde | 1 | Serialization framework (derive) |
| serde_json | 1 | JSON serialization |
| serde_yaml_ng | 0.10 | YAML frontmatter parsing |
| sha2 | 0.11 | SHA-256 fingerprinting |
| thiserror | 2 | Error type derivation |
| tracing | 0.1 | Structured logging |
| tracing-subscriber | 0.3 | Log output (env-filter) |
| anyhow | 1 | Context-rich error propagation (binary crate) |
| chrono | 0.4 | Timestamps (OSCAL metadata) |
| url | 2 | URL parsing (back matter, citations) |
| uuid | 1 | Deterministic v5 + random v4 identifiers |
| jsonschema | 0.45 | Offline OSCAL schema validation (resolver defaults disabled) |
| quick-xml | 0.41 | XML serialization/deserialization (serialize feature) |
| rayon | 1 | Parallel batch processing |
| tempfile | 3 | Atomic temporary output files |
| pdf-extract | 0.12 | Local PDF text extraction |
| zip | 8 | DOCX/OOXML archive reading (deflate only) |
| toml | 1.1 | Project configuration parsing (parse and serde features) |
| libc | 0.2 | Unix no-follow filesystem operations |

### Development

| Crate | Version | Purpose |
|-------|---------|---------|
| criterion | 0.8 | Benchmarking (html_reports) |
| insta | 1 | Snapshot testing (json feature) |
| proptest | 1 | Property-based testing |

## Phase 1 Status

24 of 25 work items complete (WI-1 through WI-24). Remaining: WI-25 (Phase 1 release).

See `docs/FORGE_PRODUCT_ROADMAP.md` for full roadmap details.

## Active Technologies
- Rust, Edition 2024, stable 1.93.0 + clap 4.x (derive), serde 1.0.228, serde_json 1.0.149, quick-xml 0.37 (add `serde` feature), serde_yaml_ng 0.10 (aliased as serde_yaml), thiserror 2.0.18 (029-export-subcommand)
- Network dependencies: N/A — reads/writes local files only (029-export-subcommand)
- Rust, Edition 2024, stable 1.93.0 + `serde_json` 1.0.149, `quick-xml` 0.37, `serde_yaml_ng` 0.10 — no runtime dependencies, test-only (in-memory models & fixtures) (028-round-trip-testing)
- Rust, edition 2024, stable 1.93.0 + clap 4.x (derive), serde 1.0.228, serde_json 1.0.149, uuid 1.20.0, chrono 0.4, thiserror 2.0.18 — all already in `Cargo.toml`; **no new dependencies required** (030-prd-profile-generation)
- Rust, Edition 2024, stable 1.93.0 + `regex = "1"` (already in `Cargo.toml`), `std::sync::LazyLock` (stdlib, matching `citation.rs` pattern) (033-prd-normative-advisory-detection)
- N/A — in-memory pipeline enrichment pass (033-prd-normative-advisory-detection)
- Rust, Edition 2024, stable 1.93.0 + `regex = "1"` (already in Cargo.toml), `std::sync::LazyLock` (std, no new dep), `thiserror = "2.0.18"`, `serde = "1.0.228"`, `tracing = "0.1.44"` (034-prd-parameter-extraction)
- N/A — transforms in-memory `PolicyDocument`; writes OSCAL JSON/XML/YAML to local filesystem (unchanged) (034-prd-parameter-extraction)
- Rust, Edition 2024, stable 1.93.0 + `jsonschema` 0.41.0 (already in Cargo.toml), `insta` 1.46.3 with `json` feature (already in Cargo.toml), `tempfile` 3.25.0 (already in Cargo.toml) — **NO NEW DEPENDENCIES REQUIRED** (032-profile-validation-tests)
- `schemas/oscal_profile_schema.json` (compile-time embedded via `include_str!`); `tests/snapshots/*.snap` (insta golden files checked into git) (032-profile-validation-tests)
- Rust 1.93.0 (Edition 2024) + `serde_json`, `quick-xml` 0.37, `serde_yaml_ng` 0.10, `clap` 4.x, `insta` 1.46.3, `tempfile` 3.25.0 (035-prd-phase2-release)
- Local filesystem (OSCAL JSON/XML/YAML files, test fixtures) (035-prd-phase2-release)
- Rust, Edition 2024, stable 1.93.0 + clap 4.x (CLI), serde 1.0.228, serde_json 1.0.149, thiserror 2.0.18, tracing 0.1.44 — all existing (044-summary-dashboard)
- N/A — dashboard writes to stderr; artifact output behavior is unchanged (044-summary-dashboard)
- Rust, Edition 2024, stable 1.93.0 + clap 4.x (derive), serde 1.0.228, thiserror 2.0.18, rayon (new — MIT/Apache-2.0) (040-batch-conversion)
- Local filesystem (read input .md files, write OSCAL JSON/XML/YAML output files) (040-batch-conversion)
- Rust, Edition 2024, stable 1.93.0 + clap 4.x (derive), thiserror 2.0.18, tracing 0.1.44 — all existing in Cargo.toml (036-oscal-cli-profile-resolution)
- Local filesystem (reads Profile JSON, writes resolved Catalog JSON) (036-oscal-cli-profile-resolution)
- Rust, Edition 2024, stable 1.93.0 + clap 4.x (CLI), serde 1.0.228, serde_json 1.0.149, chrono 0.4, thiserror 2.0.18, tracing 0.1.44 — all already in Cargo.toml. No new production dependencies. (038-traceability-report)
- N/A — reads local files, outputs to stdout or file (038-traceability-report)
- Rust 1.93.0 (Edition 2024) + `serde_json` (existing), `std::collections::HashMap` (stdlib), `clap 4.x` (existing), `thiserror 2.0.18` (existing), `tracing 0.1.44` (existing) (043-diff-report)
- N/A — reads two local JSON files into memory; no writes (043-diff-report)
- Rust, Edition 2024, stable 1.93.0 + clap 4.x, serde 1.0.228, serde_json 1.0.149, uuid 1.20.0, chrono 0.4, thiserror 2.0.18, tracing 0.1.44 — all existing in `Cargo.toml`; **no new dependencies** (041-assessment-plan-controls)
- Local filesystem — reads Markdown, writes JSON (AP always JSON regardless of `--format`) (041-assessment-plan-controls)

- Rust, Edition 2024, stable 1.93.0 + quick-xml (latest stable, MIT), existing: clap 4, serde 1.0.228, serde_json 1.0.149, thiserror 2.0.18, tracing 0.1.44, uuid 1.20.0, chrono 0.4 (026-xml-output)
- Rust 2024 edition, stable 1.93.0 + `serde_yaml_ng` 0.10 (aliased as `serde_yaml`, already in `Cargo.toml`), `serde` 1.0.228, `serde_json` 1.0.149 (027-yaml-output)

## Recent Changes

- 026-xml-output: Added Rust, Edition 2024, stable 1.93.0 + quick-xml (latest stable, MIT), existing: clap 4, serde 1.0.228, serde_json 1.0.149, thiserror 2.0.18, tracing 0.1.44, uuid 1.20.0, chrono 0.4
- 027-yaml-output: Added Rust 2024 edition, stable 1.93.0 + `serde_yaml_ng` 0.10 (aliased as `serde_yaml`, already in `Cargo.toml`), `serde` 1.0.228, `serde_json` 1.0.149
