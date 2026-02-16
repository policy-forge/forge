# FORGE

**Framework for OSCAL Risk & Governance Execution**

A Rust CLI tool that converts security policy documents into [OSCAL](https://pages.nist.gov/OSCAL/) (Open Security Controls Assessment Language), the NIST standard for machine-readable security and compliance policies.

FORGE reads Markdown policy documents — with optional YAML frontmatter — and produces schema-validated OSCAL Catalogs and Component Definitions in JSON, XML, or YAML. It also validates existing OSCAL artifacts and converts between output formats.

## Features

- **Markdown to OSCAL** — Convert policy documents into OSCAL Catalogs or Component Definitions
- **Multi-format output** — JSON, XML, and YAML with round-trip fidelity between all three
- **Schema validation** — Validate artifacts against OSCAL v1.2.0 JSON schemas with semantic checks
- **Format conversion** — Export existing OSCAL artifacts between JSON, XML, and YAML
- **Requirement atomization** — Automatically split compound policy statements into individual controls
- **Deterministic IDs** — UUID v5 generation ensures stable identifiers across re-conversions
- **Citation extraction** — URLs and references extracted into OSCAL back-matter resources
- **Traceability** — Source-to-OSCAL element mapping embedded as provenance metadata
- **Zero network dependencies** — Reads and writes local files only

## Quick Start

```bash
# Install (requires Rust 1.93.0+)
git clone https://github.com/policy-forge/forge.git
cd forge
cargo build --release

# Convert a policy to an OSCAL Catalog (JSON)
./target/release/forge convert tests/fixtures/sample_policy.md --strategy catalog --format json

# Convert to an OSCAL Component Definition
./target/release/forge convert tests/fixtures/sample_policy.md --strategy component --format json

# Validate a generated OSCAL artifact
./target/release/forge validate catalog.json
```

## Usage

### Convert

Convert a Markdown policy document into an OSCAL artifact.

```bash
# OSCAL Catalog (groups, controls, statements)
forge convert policy.md --strategy catalog --format json

# OSCAL Component Definition (implemented requirements)
forge convert policy.md --strategy component --format json

# With a source profile reference for component strategy
forge convert policy.md --strategy component --source-profile baseline.json --format json

# Output as XML or YAML
forge convert policy.md --strategy catalog --format xml
forge convert policy.md --strategy catalog --format yaml

# Write to a file instead of stdout
forge convert policy.md --strategy catalog --format json --output catalog.json

# Override max input file size (default: 10 MB)
forge convert large-policy.md --strategy catalog --format json --max-size 20
```

### Export

Convert an existing OSCAL artifact between formats. Auto-detects the input format from the file extension.

```bash
# JSON to XML
forge export catalog.json --format xml

# XML to YAML
forge export catalog.xml --format yaml

# YAML to JSON, written to a file
forge export catalog.yaml --format json --output catalog.json
```

### Validate

Validate an OSCAL artifact against the OSCAL v1.2.0 JSON schema. Auto-detects the model type (Catalog or Component Definition) from the document structure.

```bash
# Validate with human-readable output
forge validate catalog.json

# Machine-parseable JSON output
forge validate catalog.json --format json

# Override auto-detected model type
forge validate artifact.json --schema-type catalog
```

### Global Options

```bash
# Verbose: show pipeline stage information on stderr
forge -v convert policy.md --strategy catalog --format json

# Quiet: suppress all non-essential output
forge -q convert policy.md --strategy catalog --format json
```

## Input Format

FORGE accepts Markdown files (`.md` / `.markdown`) with optional YAML frontmatter:

```markdown
---
title: "Access Control Policy"
version: "2.0"
author: "Security Team"
date: "2026-01-15"
---

# Access Control

All users must authenticate before accessing systems.

## Authentication Requirements

- Users must use multi-factor authentication
- Passwords must be at least 12 characters
- Sessions must timeout after 30 minutes of inactivity

## Authorization

- Access must follow principle of least privilege
- Role-based access control must be enforced
```

Headings become OSCAL groups and controls. List items, tables, and paragraphs become control statements. Compound requirements like "Systems must X and must Y" are automatically split into atomic statements.

For other document formats (PDF, DOCX), convert to Markdown first using tools like [pandoc](https://pandoc.org/) or [markitdown](https://github.com/microsoft/markitdown).

25 sample policies are included in `example_data/` covering topics from acceptable use to incident response.

## How It Works

FORGE processes a Markdown policy document through a deterministic pipeline:

```
Ingest → Parse → Extract → Assemble → Atomize → Assign IDs → Map to OSCAL → Serialize → Validate
```

1. **Ingest** — Reads the file, validates UTF-8, computes a SHA-256 fingerprint
2. **Parse** — Builds a heading hierarchy tree from Markdown structure
3. **Extract** — Pulls list items, tables, and paragraphs from each section
4. **Assemble** — Combines sections, clauses, and YAML frontmatter into a `PolicyDocument`
5. **Atomize** — Splits compound requirements into individual atomic statements
6. **Assign IDs** — Generates deterministic UUID v5 identifiers from content, ensuring stability across re-conversions
7. **Map to OSCAL** — Transforms the domain model into OSCAL Catalog or Component Definition structures, with metadata, back matter, citation resources, and traceability links
8. **Serialize** — Outputs JSON, XML, or YAML
9. **Validate** — Checks the output against the OSCAL v1.2.0 JSON schema and runs semantic validation

## Project Structure

```
src/
  main.rs                    Entry point (anyhow error handling)
  lib.rs                     Module declarations and public API
  error.rs                   ForgeError enum (thiserror, 20+ variants)
  pipeline.rs                Catalog and Component pipeline orchestration
  uuid.rs                    Deterministic UUID v5 generation
  citation.rs                URL/reference extraction and deduplication
  cli/
    mod.rs                   CLI definition (clap derive)
    convert.rs               Convert subcommand (catalog + component strategies)
    export.rs                Export subcommand (format conversion)
    validate.rs              Validate subcommand (schema + semantic checks)
  ingest/
    mod.rs                   File ingestion, format detection, fingerprinting
  parse/
    mod.rs                   Heading hierarchy extraction (pulldown-cmark)
    clauses.rs               List item, table, and paragraph extraction
    atomize.rs               Compound requirement splitting
  model/
    mod.rs                   PolicyDocument, PolicySection, PolicyRequirement
    frontmatter.rs           YAML frontmatter parsing
    assemble.rs              Pipeline assembly (sections + clauses + frontmatter)
    trace.rs                 Traceability model (TraceLink, TraceIndex)
  oscal/
    mod.rs                   OSCAL module declarations
    catalog.rs               Catalog builder (groups, controls)
    component_definition.rs  Component Definition builder
    implemented_requirements.rs  Control implementations
    parts.rs                 Statement parts, prose, props
    metadata.rs              OSCAL metadata assembly
    back_matter.rs           Back matter resources and links
    trace_embedding.rs       Provenance metadata embedding
  export/
    mod.rs                   Format serialization orchestration
    xml_serializer.rs        OSCAL XML serialization (quick-xml)
    xml_deserializer.rs      OSCAL XML deserialization
    yaml.rs                  OSCAL YAML serialization (serde_yaml_ng)
  validate/
    mod.rs                   Validation orchestration
    error_types.rs           Structured validation error categories
    formatter.rs             Human-friendly error formatting
    report.rs                Validation report generation
    semantic.rs              Semantic validation (orphan links, missing fields)
  testing/
    mod.rs                   Round-trip testing utilities
    semantic_eq.rs           Format-agnostic equivalence comparison
tests/                       Integration tests (905 tests across 27 suites)
benches/                     Criterion benchmarks (pipeline, atomize, UUID, XML, export)
example_data/                25 sample Markdown policy documents
```

## Development

```bash
cargo build                    # Debug build
cargo build --release          # Release build
cargo test                     # Run all tests (905 tests)
cargo clippy -- -D warnings    # Lint (warnings as errors)
cargo fmt --check              # Check formatting
cargo bench                    # Run benchmarks
cargo mutants                  # Mutation testing (requires cargo-mutants)
```

### CI

Every push and PR runs:

- `cargo fmt --check` + `cargo clippy -- -D warnings`
- `cargo test` (full suite)
- `cargo bench` (pipeline benchmarks)
- `cargo audit` (security advisory check)
- `cargo deny check` (license and advisory compliance)

### Releases

Tagged releases (`v*`) build cross-platform binaries for:

- Linux (`x86_64-unknown-linux-gnu`)
- macOS (`x86_64-apple-darwin`, `aarch64-apple-darwin`)
- Windows (`x86_64-pc-windows-msvc`)

Each release includes SHA-256 checksums and [SLSA Level 3](https://slsa.dev/) provenance attestation.

## Roadmap

### Current: Phase 1 — Foundation (v0.1.0)

Core Markdown-to-OSCAL pipeline. **24 of 25 work items complete.** Remaining: Phase 1 release packaging.

| Capability | Status |
|------------|--------|
| Markdown ingestion with SHA-256 fingerprinting | Done |
| Heading hierarchy and clause extraction | Done |
| YAML frontmatter support | Done |
| Requirement atomization | Done |
| Deterministic UUID v5 identifiers | Done |
| Citation and reference extraction | Done |
| OSCAL Catalog generation (groups, controls, statements) | Done |
| OSCAL Component Definition generation | Done |
| Implemented requirements with source profiles | Done |
| OSCAL metadata, back matter, and traceability | Done |
| Schema validation against OSCAL v1.2.0 | Done |
| Structured error reporting | Done |
| Golden-file regression testing | Done |
| Performance benchmarks (<30s for 50-page documents) | Done |
| JSON, XML, and YAML output | Done |
| Format round-trip equivalence (JSON/XML/YAML) | Done |
| `forge export` format conversion subcommand | Done |
| Phase 1 release (v0.1.0 tag) | Pending |

### Next: Phase 2 — Profile & Tailoring (v0.2.0)

OSCAL Profile generation with baseline selection, parameter extraction, and normative/advisory classification.

- `forge profile` subcommand with `--include`/`--exclude` control selection
- Parameter tailoring (`--set-param`) for profile `modify` sections
- Normative vs advisory language detection (`must`/`shall` vs `should`/`may`)
- Policy parameter extraction (time windows, thresholds) into OSCAL `param` elements

### Future: Phase 3 — Ecosystem (v0.3.0+)

Integration with the broader OSCAL toolchain and community adoption.

- **oscal-cli integration** — Delegate profile resolution and cross-format validation to NIST's tooling
- **Traceability reports** — `forge trace` mapping every OSCAL element back to source text with line numbers
- **Batch conversion** — Process multiple policy documents in a single invocation
- **Assessment Plan scaffolding** — Generate OSCAL Assessment Plan skeletons from policies
- **Diff reports** — Compare two conversions of the same policy to show changes
- **SSP templates** — System Security Plan scaffolding with placeholders and trace links
- **Community examples and documentation** — Sample policies, usage guides, CONTRIBUTING.md
- **Cross-platform binary releases** — Pre-built binaries via GitHub Releases (CI already configured)

See `docs/FORGE_PRODUCT_ROADMAP.md` for the full 50-item sprint plan.

## License

MIT
