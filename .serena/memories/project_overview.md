# FORGE — Framework for OSCAL Risk & Governance Execution

## Purpose
A Rust CLI tool that converts security policy documents (Markdown currently, PDF/Word planned) into OSCAL (Open Security Controls Assessment Language), the NIST standard for machine-readable security and compliance policies.

## Tech Stack
- Rust edition 2024, stable 1.93.0
- clap 4 (CLI framework with derive macros)
- pulldown-cmark 0.13 (Markdown parsing)
- serde/serde_json/serde_yaml_ng (serialization, YAML frontmatter)
- sha2 (file fingerprinting)
- regex (requirement atomization splitting)
- uuid 1.x with v5 (deterministic stable IDs)
- tracing 0.1 (structured logging)
- thiserror 2.0.18 (error handling)
- criterion 0.8 (benchmarks, dev)
- tempfile 3.25 (tests, dev)

## Project Structure
```
src/
  main.rs              — Entry point, parses CLI and dispatches
  lib.rs               — Module declarations (cli, error, export, ingest, model, oscal, parse, uuid, validate)
  error.rs             — ForgeError enum (thiserror), format_size helper
  cli/
    mod.rs             — Cli struct (clap), Commands enum, Strategy, OutputFormat, execute()
    convert.rs         — Convert subcommand: ingests file, extracts sections/clauses, assembles domain model, assigns UUIDs, outputs JSON
    validate.rs        — Validate subcommand (stub)
  ingest/
    mod.rs             — SourceLine, IngestedDocument, ingest_file() — reads/validates/fingerprints Markdown
  parse/
    mod.rs             — SectionNode, extract_sections() — builds heading hierarchy tree from Markdown
    clauses.rs         — ExtractedListItem, ExtractedTable, ExtractedParagraph, ExtractedContent, extract_clauses() — extracts lists/tables/paragraphs
    atomize.rs         — AtomizationResult, atomize_requirement(), atomize_document() — splits compound requirements into atomic statements
  model/
    mod.rs             — PolicyDocument, DocumentMetadata, PolicySection, PolicyRequirement — domain model structs
    frontmatter.rs     — FrontmatterData, parse_frontmatter() — YAML frontmatter extraction
    assemble.rs        — assemble_document() — combines parsed sections + clauses + frontmatter into PolicyDocument
  uuid.rs              — FORGE_NAMESPACE_UUID, generate_stable_id(), assign_stable_ids() — deterministic UUID v5 generation
  oscal/mod.rs         — Empty (planned)
  export/mod.rs        — Empty (planned)
  validate/mod.rs      — Empty (planned)
tests/
  cli_integration.rs   — Integration tests via binary execution
  atomize_integration.rs — Atomization integration tests
  pipeline_test.rs     — End-to-end pipeline tests
  common/              — Shared test utilities
  fixtures/            — Test fixture files
benches/
  atomize.rs           — Atomization benchmarks (criterion)
  uuid_benchmark.rs    — UUID generation benchmarks (criterion)
example_data/          — 25 sample policy Markdown documents
```

## Pipeline Flow
1. **Ingest** — Read Markdown file, validate, fingerprint (SHA-256)
2. **Parse sections** — Build heading hierarchy tree from Markdown
3. **Extract clauses** — Pull out list items, tables, paragraphs from each section
4. **Assemble** — Combine into PolicyDocument domain model (with YAML frontmatter metadata)
5. **Atomize** — Split compound requirements into atomic statements with preliminary IDs
6. **Assign UUIDs** — Generate deterministic UUID v5 stable IDs for each requirement
7. **Export** — Output as JSON (OSCAL mapping planned)

## Modules Status
- **Implemented**: cli, error, ingest, parse (sections + clauses + atomize), model (domain + frontmatter + assemble), uuid
- **Stub/Empty**: oscal, export, validate (library-level)
