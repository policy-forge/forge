# FORGE Architecture Plan

## Context

FORGE (Framework for OSCAL Risk & Governance Execution) needs its initial architecture. The repo is a blank Rust project. We need to set up the module structure, dependencies, core types, and pipeline for converting security policy documents (Markdown, PDF, DOCX) into OSCAL JSON.

OSCAL v1.2.0 is the latest NIST standard. No mature Rust OSCAL crate exists — we'll hand-write types for the four core models.

## Architecture Overview

```
Document (MD/PDF/DOCX)
    → [Document Reader]       format-specific parsing
    → DocumentContent (IR)    format-agnostic intermediate representation
    → [Rule Engine]           pattern matching, regex, heuristics
    → ExtractionResult        with confidence scores per item
    → [LLM Provider]          only for items below confidence threshold
    → [OSCAL Builder]         constructs typed OSCAL structs
    → [Validator]             semantic checks
    → JSON output             serde_json serialization
```

## Project Structure

```
src/
  main.rs                     # Entry point, CLI dispatch
  lib.rs                      # Library root, re-exports
  error.rs                    # ForgeError enum (thiserror)
  cli/
    mod.rs
    args.rs                   # Clap derive structs (convert, validate, init)
    output.rs                 # Tracing init, output formatting
  config/
    mod.rs
    settings.rs               # TOML config loading (~/.config/forge/config.toml)
    provider_config.rs         # LLM provider config types
  document/
    mod.rs
    format.rs                 # DocumentFormat enum, extension-based detection
    reader.rs                 # DocumentReader trait
    content.rs                # DocumentContent IR (Section, Table, ListBlock)
    markdown.rs               # pulldown-cmark reader
    pdf.rs                    # pdf-extract reader
    docx.rs                   # docx-rust reader
  oscal/
    mod.rs                    # OscalDocument enum (untagged)
    common.rs                 # Metadata, Property, Link, Part, Parameter, Role, Party
    catalog.rs                # Catalog, Control, Group
    profile.rs                # Profile, Import, Merge, Modify
    component_definition.rs   # ComponentDefinition, DefinedComponent, ControlImplementation
    ssp.rs                    # SystemSecurityPlan, SystemCharacteristics, etc.
    validation.rs             # Validatable trait, semantic checks
  parser/
    mod.rs
    pipeline.rs               # Orchestrator: reader → rules → LLM → builder → validate
    rule_engine.rs            # ExtractionRule trait, RuleEngine
    confidence.rs             # ExtractedItem, ExtractedItemKind, confidence scoring
    rules/
      mod.rs
      heading_rules.rs        # Control ID patterns (e.g., "AC-1 Access Control")
      control_rules.rs        # Statement/guidance extraction
      table_rules.rs          # Tabular data extraction
  llm/
    mod.rs                    # create_provider() factory
    provider.rs               # LlmProvider async trait
    prompt.rs                 # Prompt templates per OSCAL model
    response.rs               # LLM response parsing
    providers/
      mod.rs
      claude.rs               # Anthropic API
      openai.rs               # OpenAI-compatible API
      ollama.rs               # Local Ollama
```

## Dependencies (Cargo.toml)

| Crate | Purpose |
|-------|---------|
| `clap` (derive) | CLI with subcommands |
| `pulldown-cmark` | Markdown parsing |
| `pdf-extract` | PDF text extraction |
| `docx-rust` | DOCX parsing |
| `serde` + `serde_json` | Serialization |
| `reqwest` (json, stream) | HTTP client for LLM APIs |
| `tokio` (rt-multi-thread, macros) | Async runtime |
| `thiserror` | Domain error types |
| `anyhow` | Application-level errors |
| `uuid` (v4, serde) | OSCAL UUID generation |
| `chrono` (serde) | OSCAL timestamps |
| `toml` | Config file parsing |
| `directories` | Platform config paths |
| `tracing` + `tracing-subscriber` | Logging |

## Key Design Decisions

1. **Single crate** — no workspace overhead for MVP; extract crates later if needed
2. **Hand-written OSCAL types** — only 4 models needed; gives us idiomatic Rust with proper serde rename attributes for OSCAL's kebab-case fields
3. **DocumentContent IR** — decouples format parsing from OSCAL extraction; all readers produce the same structure
4. **Confidence-based LLM fallback** — rule engine scores each extraction 0.0-1.0; items below threshold (default 0.7) get sent to the LLM
5. **Trait-based LLM abstraction** — `async_trait` for runtime polymorphism across providers
6. **TOML config** — idiomatic for Rust; stores provider settings at `~/.config/forge/config.toml`

## CLI Subcommands

- `forge convert <FILE> --model <catalog|profile|component-definition|ssp>` — core conversion
  - `--output <FILE>` — output path (default: stdout)
  - `--use-llm` — enable LLM fallback
  - `--provider <claude|openai|ollama>` — override default provider
  - `--confidence-threshold <0.0-1.0>` — tune LLM trigger sensitivity
- `forge validate <FILE>` — validate existing OSCAL JSON
- `forge init` — generate default config file

## Phased Implementation

### Phase 1: Foundation
Set up project structure, define OSCAL common types + Catalog model, implement Markdown reader, wire up basic CLI. **Goal:** `forge convert policy.md --model catalog` produces valid OSCAL Catalog JSON.

Files: `error.rs`, `cli/args.rs`, `document/content.rs`, `document/reader.rs`, `document/markdown.rs`, `document/format.rs`, `oscal/common.rs`, `oscal/catalog.rs`, `oscal/mod.rs`, `main.rs`, `lib.rs`

### Phase 2: Rule Engine + Pipeline
Implement extraction rules, confidence scoring, OSCAL builder, and validation. **Goal:** Robust Catalog extraction from Markdown with confidence scores.

Files: `parser/pipeline.rs`, `parser/rule_engine.rs`, `parser/confidence.rs`, `parser/rules/*`, `oscal/validation.rs`

### Phase 3: Additional Readers
Add PDF and DOCX readers. **Goal:** `forge convert` works with all three input formats.

Files: `document/pdf.rs`, `document/docx.rs`

### Phase 4: LLM Integration
Implement provider trait, Claude/OpenAI/Ollama providers, prompt templates, config system. **Goal:** `--use-llm` flag works for ambiguous content.

Files: `llm/*`, `config/*`

### Phase 5: Remaining OSCAL Models
Add Profile, Component Definition, and SSP types/builders/rules. **Goal:** All four OSCAL models supported.

Files: `oscal/profile.rs`, `oscal/component_definition.rs`, `oscal/ssp.rs`, model-specific rules

### Phase 6: Polish
Progress reporting, streaming LLM output, comprehensive tests, CI/CD.

## Verification

After Phase 1:
```bash
cargo build
cargo test
echo "# AC-1 Access Control\nThe organization shall..." > /tmp/test.md
cargo run -- convert /tmp/test.md --model catalog
# Should output valid OSCAL Catalog JSON to stdout
```

After each phase: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`
