# FORGE

**Framework for OSCAL Risk & Governance Execution**

A Rust CLI tool that converts security policy documents into [OSCAL](https://pages.nist.gov/OSCAL/) (Open Security Controls Assessment Language), the NIST standard for machine-readable security and compliance policies.

## Status

FORGE v0.1.0 — Phase 1 complete. All pipeline stages are implemented and verified:

| Stage | Description | Status |
|-------|-------------|--------|
| Ingest | Read Markdown, validate, fingerprint (SHA-256) | Done |
| Parse sections | Build heading hierarchy tree | Done |
| Extract clauses | Extract list items, tables, paragraphs | Done |
| Assemble | Combine into domain model with YAML frontmatter | Done |
| Atomize | Split compound requirements into atomic statements | Done |
| Assign IDs | Deterministic UUID v5 stable identifiers | Done |
| Citation Extraction | Detect bibliographic refs, URLs, cross-refs | Done |
| Catalog Pipeline | End-to-end `forge convert --strategy catalog` | Done |
| Component Pipeline | End-to-end `forge convert --strategy component` | Done |
| Traceability | Source-to-OSCAL element mapping with props and links | Done |
| Schema Validation | Validate against OSCAL v1.2.0 JSON schemas | Done |
| Error Handling | Structured errors with exit codes and actionable messages | Done |
| Performance | <30s for 50-page documents (benchmarked) | Done |

## Quick Start

```bash
# Install (requires Rust 1.93.0+)
git clone https://github.com/policy-forge/forge.git
cd forge
cargo build --release

# Convert a sample policy to OSCAL Catalog JSON
./target/release/forge convert tests/fixtures/sample_policy.md --strategy catalog --format json
```

## Usage

### Convert a Markdown policy to OSCAL Catalog

```bash
forge convert policy.md --strategy catalog --format json
```

### Convert to OSCAL Component Definition

```bash
forge convert policy.md --strategy component --format json --source-profile baseline.json
```

### Write output to a file

```bash
forge convert policy.md --strategy catalog --format json --output catalog.json
```

### Validate a generated OSCAL artifact

```bash
forge validate catalog.json
```

### Verbose and quiet modes

```bash
# Show pipeline stage information on stderr
forge -v convert policy.md --strategy catalog --format json

# Suppress all non-essential output (only OSCAL artifact on stdout)
forge -q convert policy.md --strategy catalog --format json
```

### Override max file size (in MB)

```bash
forge convert large-policy.md --strategy catalog --format json --max-size 20
```

A sample policy file is available at `tests/fixtures/sample_policy.md`.

## How It Works

FORGE processes a Markdown policy document through a deterministic pipeline:

1. **Ingest** — Reads the file, validates it's UTF-8 Markdown, computes a SHA-256 fingerprint.
2. **Parse** — Builds a heading hierarchy tree from the Markdown structure.
3. **Extract** — Pulls list items, tables, and paragraphs from each section as clause-level content.
4. **Assemble** — Combines parsed sections, extracted clauses, and optional YAML frontmatter into a `PolicyDocument` domain model.
5. **Atomize** — Splits compound requirements (e.g., "Systems must X and must Y") into individual atomic statements.
6. **Assign IDs** — Generates deterministic UUID v5 identifiers for each requirement.
7. **Extract Citations** — Detects bibliographic references (NIST SP, ISO, RFC), URLs, and cross-references.
8. **Map to OSCAL** — Converts the domain model to OSCAL Catalog (groups + controls) or Component Definition (components + implemented-requirements).
9. **Embed Traceability** — Adds source-file, source-section, and source-line props to every control.
10. **Validate** — Auto-validates the generated OSCAL artifact against JSON schemas before output.

## Project Structure

```
src/
  main.rs              Entry point (verbose/quiet filter wiring)
  lib.rs               Module declarations and public API
  error.rs             ForgeError enum (thiserror)
  pipeline.rs          E2E pipeline orchestration
  uuid.rs              Deterministic UUID v5 generation
  citation.rs          Citation extraction (URLs, bibliographic, cross-refs)
  cli/
    mod.rs             CLI definition (clap derive)
    convert.rs         Convert subcommand handler
    validate.rs        Validate subcommand handler
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
    trace.rs           TraceLink model
  oscal/
    mod.rs             OSCAL module declarations
    catalog.rs         Catalog builder (groups, controls)
    parts.rs           Statement parts, prose, props
    metadata.rs        OSCAL metadata assembly
    back_matter.rs     Back matter resources and links
    component_definition.rs  Component Definition builder
    implemented_requirements.rs  Implemented requirements mapping
    trace_embedding.rs  Trace prop/link embedding
  export/
    mod.rs             JSON export
  validate/
    mod.rs             Schema validation orchestration
    error_types.rs     Validation error types
    formatter.rs       Error formatting (SEC-1 truncation)
    report.rs          Validation report
    semantic.rs        Semantic validation
tests/                 Integration tests and fixtures
benches/               Criterion benchmarks
```

## Development

```bash
cargo build                    # Debug build
cargo build --release          # Release build
cargo test                     # Run all tests (680+ tests)
cargo clippy -- -D warnings    # Lint (warnings as errors)
cargo fmt --check              # Check formatting
cargo bench                    # Run benchmarks
cargo mutants                  # Mutation testing (requires cargo-mutants)
```

## License

MIT
