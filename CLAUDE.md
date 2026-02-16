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

Rust edition 2024, stable 1.93.0.

### Production

| Crate | Version | Purpose |
|-------|---------|---------|
| clap | 4.x | CLI argument parsing (derive) |
| pulldown-cmark | 0.13 | Markdown event-stream parsing |
| regex | 1 | Pattern matching (atomization, citations) |
| serde | 1.0.228 | Serialization framework (derive) |
| serde_json | 1.0.149 | JSON serialization |
| serde_yaml_ng | 0.10 | YAML frontmatter parsing |
| sha2 | 0.10.9 | SHA-256 fingerprinting |
| thiserror | 2.0.18 | Error type derivation |
| tracing | 0.1.44 | Structured logging |
| tracing-subscriber | 0.3 | Log output (env-filter) |
| anyhow | 1 | Context-rich error propagation (binary crate) |
| chrono | 0.4 | Timestamps (OSCAL metadata) |
| url | 2.5 | URL parsing (back matter, citations) |
| uuid | 1.20.0 | Deterministic v5 + random v4 identifiers |
| jsonschema | 0.41.0 | OSCAL schema validation |
| quick-xml | 0.37 | XML serialization/deserialization (feature: serde) |

### Development

| Crate | Version | Purpose |
|-------|---------|---------|
| criterion | 0.8.2 | Benchmarking (html_reports) |
| insta | 1.46.3 | Snapshot testing (json feature) |
| tempfile | 3.25.0 | Temporary files in tests |

## Phase 1 Status

24 of 25 work items complete (WI-1 through WI-24). Remaining: WI-25 (Phase 1 release).

See `docs/FORGE_PRODUCT_ROADMAP.md` for full roadmap details.

## Active Technologies
- Rust, Edition 2024, stable 1.93.0 + clap 4.x (derive), serde 1.0.228, serde_json 1.0.149, quick-xml 0.37 (add `serde` feature), serde_yaml_ng 0.10 (aliased as serde_yaml), thiserror 2.0.18 (029-export-subcommand)
- Network dependencies: N/A — reads/writes local files only (029-export-subcommand)
- Rust, Edition 2024, stable 1.93.0 + `serde_json` 1.0.149, `quick-xml` 0.37, `serde_yaml_ng` 0.10 — no runtime dependencies, test-only (in-memory models & fixtures) (028-round-trip-testing)

- Rust, Edition 2024, stable 1.93.0 + quick-xml (latest stable, MIT), existing: clap 4, serde 1.0.228, serde_json 1.0.149, thiserror 2.0.18, tracing 0.1.44, uuid 1.20.0, chrono 0.4 (026-xml-output)
- Rust 2024 edition, stable 1.93.0 + `serde_yaml_ng` 0.10 (aliased as `serde_yaml`, already in `Cargo.toml`), `serde` 1.0.228, `serde_json` 1.0.149 (027-yaml-output)

## Recent Changes

- 026-xml-output: Added Rust, Edition 2024, stable 1.93.0 + quick-xml (latest stable, MIT), existing: clap 4, serde 1.0.228, serde_json 1.0.149, thiserror 2.0.18, tracing 0.1.44, uuid 1.20.0, chrono 0.4
- 027-yaml-output: Added Rust 2024 edition, stable 1.93.0 + `serde_yaml_ng` 0.10 (aliased as `serde_yaml`, already in `Cargo.toml`), `serde` 1.0.228, `serde_json` 1.0.149
