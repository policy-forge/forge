# FORGE — Framework for OSCAL Risk & Governance Execution

## Purpose
A Rust CLI tool that converts security policy documents (Markdown currently, PDF/Word planned) into OSCAL (Open Security Controls Assessment Language), the NIST standard for machine-readable security and compliance policies.

## Tech Stack
- Rust edition 2024, stable 1.93.0
- clap 4 (CLI framework with derive macros)
- pulldown-cmark 0.13 (Markdown parsing)
- serde/serde_json (serialization)
- sha2 (file fingerprinting)
- thiserror 2.0.18 (error handling)
- tempfile 3.25 (dev dependency for tests)

## Project Structure
```
src/
  main.rs          — Entry point, parses CLI and dispatches
  lib.rs           — Module declarations and re-exports ForgeError
  error.rs         — ForgeError enum (thiserror), format_size helper
  cli/
    mod.rs         — Cli struct (clap), Commands enum, Strategy, OutputFormat, execute()
    convert.rs     — Convert subcommand: ingests file, extracts sections, outputs JSON
    validate.rs    — Validate subcommand: stub (not yet implemented)
  ingest/
    mod.rs         — SourceLine, IngestedDocument, ingest_file() — reads/validates/fingerprints Markdown
  parse/
    mod.rs         — SectionNode, extract_sections() — builds heading hierarchy tree from Markdown
  model/mod.rs     — Empty (planned)
  oscal/mod.rs     — Empty (planned)
  export/mod.rs    — Empty (planned)
  validate/mod.rs  — Empty (planned)
tests/
  cli_integration.rs — Integration tests via binary execution
example_data/      — 25 sample policy Markdown documents
```

## Modules Status
- **Implemented**: cli, error, ingest, parse
- **Stub/Empty**: model, oscal, export, validate (library-level)
