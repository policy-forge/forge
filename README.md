# FORGE

**Framework for OSCAL Risk & Governance Execution**

A Rust CLI tool that converts security policy documents (Markdown) into [OSCAL](https://pages.nist.gov/OSCAL/) (Open Security Controls Assessment Language), the NIST standard for machine-readable security and compliance artifacts.

## Quick Start

```bash
# Build
cargo build --release

# Convert a Markdown policy document
forge convert path/to/policy.md

# Convert with custom max file size (default: 10 MB)
forge convert large-policy.md --max-size 50

# Validate an OSCAL artifact
forge validate artifact.json
```

## Features

- **Markdown Ingestion** — Reads `.md` / `.markdown` files with UTF-8 validation, SHA-256 content hashing, and line-level tracking
- **Structural Extraction** — Parses headings (H1-H6) into a hierarchical section tree using pulldown-cmark, capturing titles, levels, source line numbers, and body text
- **Convert Pipeline** — `forge convert` ingests a document, extracts section structure, and outputs JSON

## Architecture

```
src/
├── cli/           # CLI dispatch (clap 4) — convert and validate subcommands
├── error.rs       # ForgeError enum (thiserror)
├── ingest/        # Markdown file ingestion — UTF-8 validation, line tracking, hashing
├── parse/         # Structural extraction — heading hierarchy via pulldown-cmark
├── model/         # Domain model (future: PolicySection assembly)
├── export/        # OSCAL export (future)
├── oscal/         # OSCAL types (future)
└── validate/      # OSCAL validation (future)
```

## Development

```bash
cargo test                       # Run all tests (73 tests)
cargo fmt --check                # Check formatting
cargo clippy -- -D warnings      # Lint with strict warnings
cargo doc --no-deps              # Build documentation
```

### Example Data

The `example_data/` directory contains 25 sample policy documents (e.g., Acceptable Use, Access Control, Incident Response) used for integration testing.

## Tech Stack

- **Rust** edition 2024 (stable 1.93.0)
- **clap 4** — CLI argument parsing
- **pulldown-cmark 0.13** — Markdown event-based parsing
- **serde / serde_json** — JSON serialization
- **sha2** — Content hashing
- **thiserror** — Error type derivation

## License

MIT
