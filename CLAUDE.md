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
