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

## Recent Changes
- 002-markdown-ingestion: Added Rust (edition 2024, stable 1.93.0) + clap 4, thiserror 2.0.18 + NEW: serde 1.x, serde_json 1.x, sha2 0.10.x
