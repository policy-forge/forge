# FORGE

**Framework for OSCAL Risk & Governance Execution**

A Rust CLI tool that converts security policy documents into [OSCAL](https://pages.nist.gov/OSCAL/) (Open Security Controls Assessment Language), the NIST standard for machine-readable security and compliance policies.

## Status

FORGE is nearing Phase 1 completion (24 of 25 work items done). Both the Catalog and Component Definition pipelines are fully operational with schema validation, traceability, and error reporting.

| Stage | Description | Status |
|-------|-------------|--------|
| Ingest | Read Markdown, validate, fingerprint (SHA-256) | Done |
| Parse sections | Build heading hierarchy tree | Done |
| Extract clauses | Extract list items, tables, paragraphs | Done |
| Assemble | Combine into domain model with YAML frontmatter | Done |
| Atomize | Split compound requirements into atomic statements | Done |
| Assign IDs | Deterministic UUID v5 stable identifiers | Done |
| Citation extraction | URL/reference detection with deduplication | Done |
| Catalog groups & controls | Map sections/requirements to OSCAL groups and controls | Done |
| Catalog statement parts | Control parts with prose, props, and structure | Done |
| OSCAL metadata | UUID, title, last-modified, version, oscal-version | Done |
| Back matter | Resources from citations, link patterns | Done |
| Catalog pipeline | End-to-end `forge convert --strategy catalog` | Done |
| Component Definition | OSCAL Component Definition generation | Done |
| Implemented requirements | Control implementations with source profiles | Done |
| Traceability | Source-to-OSCAL element mapping with embedding | Done |
| Validate | Schema + semantic validation against OSCAL v1.2.0 | Done |
| Error reporting | Structured validation errors with human-friendly output | Done |
| Golden file tests | Snapshot-based regression testing with insta | Done |
| Performance benchmarks | Criterion benchmarks for pipeline stages | Done |
| Error handling | Anyhow-based context propagation in CLI | Done |
| Phase 1 release | Final packaging and release prep | Planned |

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
# Convert a Markdown policy to an OSCAL Catalog (JSON)
forge convert policy.md --strategy catalog --format json

# Convert to an OSCAL Component Definition
forge convert policy.md --strategy component --format json

# Convert with a source profile reference
forge convert policy.md --strategy component --source-profile ./profile.json --format json

# Write output to a file
forge convert policy.md --strategy catalog --format json --output catalog.json

# Validate an existing OSCAL artifact against the schema
forge validate artifact.json

# Override max file size (default: 10 MB)
forge convert policy.md --strategy catalog --format json --max-size 20

# Verbose or quiet mode
forge -v convert policy.md --strategy catalog --format json
forge -q convert policy.md --strategy catalog --format json
```

## How It Works

FORGE processes a Markdown policy document through a deterministic pipeline:

1. **Ingest** -- Reads the file, validates it's UTF-8 Markdown, computes a SHA-256 fingerprint.
2. **Parse** -- Builds a heading hierarchy tree from the Markdown structure.
3. **Extract** -- Pulls list items, tables, and paragraphs from each section as clause-level content.
4. **Assemble** -- Combines parsed sections, extracted clauses, and optional YAML frontmatter into a `PolicyDocument` domain model.
5. **Atomize** -- Splits compound requirements (e.g., "Systems must X and must Y") into individual atomic statements, each with a preliminary content-hash ID.
6. **Assign IDs** -- Generates deterministic UUID v5 identifiers for each requirement using a project namespace and normalized content, ensuring stable IDs across re-conversions.

Both the OSCAL Catalog and Component Definition pipelines are complete with full schema validation, traceability embedding, and structured error reporting. The remaining work item (WI-25) is Phase 1 release packaging.

## Project Structure

```
src/
  main.rs              Entry point (anyhow error handling)
  lib.rs               Module declarations and public API
  error.rs             ForgeError enum (thiserror)
  pipeline.rs          Catalog and Component pipeline orchestration
  uuid.rs              Deterministic UUID v5 generation
  citation.rs          URL/reference extraction and deduplication
  cli/
    mod.rs             CLI definition (clap derive)
    convert.rs         Convert subcommand (catalog + component strategies)
    validate.rs        Validate subcommand (schema + semantic checks)
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
    trace.rs           Traceability model (TraceLink, TraceIndex)
  oscal/
    mod.rs             OSCAL module declarations
    catalog.rs         Catalog builder (groups, controls)
    component_definition.rs  Component Definition builder
    implemented_requirements.rs  Control implementations
    parts.rs           Statement parts, prose, props
    metadata.rs        OSCAL metadata assembly
    back_matter.rs     Back matter resources and links
    trace_embedding.rs Post-processing traceability embedding
    test_utils.rs      Shared OSCAL test helpers
  export/mod.rs        Multi-format export (JSON implemented; XML/YAML planned)
  validate/
    mod.rs             Validation orchestration
    error_types.rs     Structured validation error categories
    formatter.rs       Human-friendly error formatting
    report.rs          Validation report generation
    semantic.rs        Semantic validation (orphan links, missing fields)
tests/                 Integration tests (131 tests across 13 files)
benches/               Criterion benchmarks (atomization, UUID, pipeline)
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
