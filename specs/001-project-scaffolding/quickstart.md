# Quickstart: Project Scaffolding

**Feature**: 001-project-scaffolding
**Date**: 2026-02-11

## Prerequisites

- Rust toolchain installed via [rustup](https://rustup.rs/) (latest stable)
- Git installed
- GitHub Actions enabled on the repository (for CI)

## Setup

### 1. Clone and Build

```bash
git clone https://github.com/policy-forge/forge.git
cd forge
cargo build
```

### 2. Verify CLI

```bash
# Show help text with available subcommands
cargo run -- --help

# Show version
cargo run -- --version

# Try convert subcommand (stub — prints "not yet implemented")
cargo run -- convert policy.md

# Try validate subcommand (stub — prints "not yet implemented")
cargo run -- validate artifact.json
```

### 3. Run Quality Gates

```bash
# Format check
cargo fmt --check

# Lint check (all warnings are errors)
cargo clippy -- -D warnings

# Run all tests
cargo test
```

All three must pass with zero violations.

## Project Layout

```text
src/
├── main.rs          # Entry point
├── lib.rs           # Library root
├── error.rs         # ForgeError enum
├── cli/             # CLI definitions and subcommand handlers
│   ├── mod.rs       # Cli struct, Commands enum
│   ├── convert.rs   # Convert handler (stub)
│   └── validate.rs  # Validate handler (stub)
├── ingest/mod.rs    # File ingestion (stub)
├── parse/mod.rs     # Markdown parsing (stub)
├── model/mod.rs     # Domain model (stub)
├── oscal/mod.rs     # OSCAL generation (stub)
├── validate/mod.rs  # Schema validation (stub)
└── export/mod.rs    # Output serialization (stub)
```

## Development Workflow

1. Write a failing test (`cargo test` shows red)
2. Implement minimal code to pass the test
3. Run quality gates: `cargo fmt && cargo clippy -- -D warnings && cargo test`
4. Refactor if needed (tests must stay green)
5. Commit with conventional commit message

## What's Next

After scaffolding is complete, subsequent work items add real functionality:
- **WI-2** (002): Markdown ingestion — reads policy files into the `ingest` module
- **WI-5** (005): Domain model — defines OSCAL-aligned structs in the `model` module
- **WI-9+**: OSCAL generation — produces OSCAL output in the `oscal` module
