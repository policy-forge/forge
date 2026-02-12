# FORGE

**Framework for OSCAL Risk & Governance Execution**

A Rust CLI tool that converts security policy documents into [OSCAL](https://pages.nist.gov/OSCAL/) (Open Security Controls Assessment Language), the NIST standard for machine-readable security and compliance policies.

## Status

FORGE is in active development. The ingestion and parsing pipeline is implemented:

| Stage | Description | Status |
|-------|-------------|--------|
| Ingest | Read Markdown, validate, fingerprint (SHA-256) | Done |
| Parse sections | Build heading hierarchy tree | Done |
| Extract clauses | Extract list items, tables, paragraphs | Done |
| Assemble | Combine into domain model with YAML frontmatter | Done |
| Atomize | Split compound requirements into atomic statements | Done |
| Assign IDs | Deterministic UUID v5 stable identifiers | Done |
| OSCAL mapping | Map domain model to OSCAL Catalog / Component Definition | Planned |
| Validate | Schema validation against OSCAL v1.2.0 | Planned |
| Export | JSON / XML / YAML output | Planned |

## Installation

Requires Rust 1.93.0+ (edition 2024).

```bash
git clone https://github.com/policy-forge/forge.git
cd forge
cargo build --release
```

The binary is at `target/release/forge`.

## Usage

```bash
# Convert a Markdown policy to JSON (current output is the internal domain model)
forge convert policy.md

# Specify output format (JSON is the default; XML and YAML are planned)
forge convert policy.md --format json

# Write output to a file
forge convert policy.md --output output.json

# Override max file size (default: 10 MB)
forge convert policy.md --max-size 20

# Verbose or quiet mode
forge -v convert policy.md
forge -q convert policy.md
```

## How It Works

FORGE processes a Markdown policy document through a deterministic pipeline:

1. **Ingest** -- Reads the file, validates it's UTF-8 Markdown, computes a SHA-256 fingerprint.
2. **Parse** -- Builds a heading hierarchy tree from the Markdown structure.
3. **Extract** -- Pulls list items, tables, and paragraphs from each section as clause-level content.
4. **Assemble** -- Combines parsed sections, extracted clauses, and optional YAML frontmatter into a `PolicyDocument` domain model.
5. **Atomize** -- Splits compound requirements (e.g., "Systems must X and must Y") into individual atomic statements, each with a preliminary content-hash ID.
6. **Assign IDs** -- Generates deterministic UUID v5 identifiers for each requirement using a project namespace and normalized content, ensuring stable IDs across re-conversions.

The output is currently the internal domain model as JSON. OSCAL Catalog and Component Definition mapping is the next milestone.

## Project Structure

```
src/
  main.rs              Entry point
  lib.rs               Module declarations and public API
  error.rs             ForgeError enum (thiserror)
  cli/
    mod.rs             CLI definition (clap derive)
    convert.rs         Convert subcommand
    validate.rs        Validate subcommand (stub)
  ingest/
    mod.rs             File ingestion and fingerprinting
  parse/
    mod.rs             Heading hierarchy extraction
    clauses.rs         List item, table, and paragraph extraction
    atomize.rs         Compound requirement splitting
  model/
    mod.rs             PolicyDocument, PolicySection, PolicyRequirement
    frontmatter.rs     YAML frontmatter parsing
    assemble.rs        Pipeline assembly (sections + clauses + frontmatter)
  uuid.rs              Deterministic UUID v5 generation
  oscal/mod.rs         OSCAL types (planned)
  export/mod.rs        Multi-format export (planned)
  validate/mod.rs      Schema validation (planned)
tests/                 Integration tests and fixtures
benches/               Criterion benchmarks (atomization, UUID generation)
example_data/          Sample policy Markdown documents
```

## Development

```bash
cargo build                    # Debug build
cargo test                     # Run all tests
cargo clippy -- -D warnings    # Lint (warnings as errors)
cargo fmt --check              # Check formatting
cargo bench                    # Run benchmarks
cargo mutants                  # Mutation testing (requires cargo-mutants)
```

## License

MIT
