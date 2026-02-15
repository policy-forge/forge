# FORGE Architecture Codemap

Quick architectural reference for navigating the Forge codebase.

## Module Overview

```
src/
  main.rs                  CLI entry point (anyhow error handling)
  lib.rs                   Public API re-exports
  pipeline.rs              Pipeline orchestration (catalog + component)
  error.rs                 ForgeError enum (16 variants)
  uuid.rs                  Deterministic UUID v5 + stable ID assignment
  citation.rs              URL/reference extraction + deduplication

  cli/                     Command-line interface (clap derive)
    mod.rs                 Top-level CLI struct, verbosity flags
    convert.rs             `forge convert` (--strategy catalog|component)
    validate.rs            `forge validate` (schema + semantic checks)

  ingest/                  File ingestion layer
    mod.rs                 Read file, validate UTF-8, SHA-256 fingerprint

  parse/                   Markdown parsing layer
    mod.rs                 Heading hierarchy extraction (pulldown-cmark)
    clauses.rs             List items, tables, paragraphs per section
    atomize.rs             Compound requirement splitting ("must X and must Y")

  model/                   Domain model
    mod.rs                 PolicyDocument, PolicySection, PolicyRequirement
    frontmatter.rs         YAML frontmatter parsing (serde_yaml_ng)
    assemble.rs            Combine sections + clauses + frontmatter
    trace.rs               TraceLink, TraceIndex (source-to-OSCAL mapping)

  oscal/                   OSCAL output generation
    mod.rs                 Module declarations + OscalModelType enum
    catalog.rs             Catalog builder (groups, controls, abbreviations)
    component_definition.rs  Component Definition builder
    implemented_requirements.rs  Control implementations
    parts.rs               Statement parts (prose, guidance, props)
    metadata.rs            OSCAL metadata (UUID, title, version, timestamps)
    back_matter.rs         Back matter resources from citations
    trace_embedding.rs     Post-processing traceability injection
    test_utils.rs          Shared test helpers for OSCAL tests

  export/                  Output serialization
    mod.rs                 JSON export (XML/YAML planned)

  validate/                Validation layer
    mod.rs                 Orchestration (schema + semantic)
    error_types.rs         ValidationErrorCategory enum
    formatter.rs           Human-friendly error formatting
    report.rs              Validation report generation
    semantic.rs            Semantic checks (orphan links, missing fields)
```

## Data Flow

The conversion pipeline is a 16-stage functional transformation:

```
Input (.md file)
  |
  v
1. ingest::read_file()          -- Read + validate UTF-8 + SHA-256 fingerprint
  |
  v
2. parse::extract_sections()    -- Markdown -> heading hierarchy tree
  |
  v
3. parse::clauses::extract()    -- List items, tables, paragraphs per section
  |
  v
4. model::assemble::assemble()  -- Sections + clauses + YAML frontmatter -> PolicyDocument
  |
  v
5. parse::atomize::atomize()    -- Split compound requirements into atomic statements
  |
  v
6. uuid::assign_stable_ids()    -- Deterministic UUID v5 for each requirement
  |
  v
7. citation::extract_citations()  -- URL/reference extraction + deduplication
  |
  v  (branch based on --strategy)
  |
  +--[catalog]-------------------------------------------+
  |                                                      |
  | 8.  oscal::catalog::build_catalog()                  |
  | 9.  oscal::parts::build_control_parts()              |
  | 10. oscal::metadata::build_metadata()                |
  | 11. oscal::back_matter::build_back_matter()          |
  | 12. oscal::trace_embedding::embed_trace_links()      |
  | 13. validate (if enabled)                            |
  | 14. export::to_json()                                |
  |                                                      |
  +--[component]-----------------------------------------+
  |                                                      |
  | 8.  oscal::component_definition::build()             |
  | 9.  oscal::implemented_requirements::build()         |
  | 10. oscal::metadata::build_metadata()                |
  | 11. oscal::back_matter::build_back_matter()          |
  | 12. oscal::trace_embedding::embed_trace_links()      |
  | 13. validate (if enabled)                            |
  | 14. export::to_json()                                |
  +------------------------------------------------------+
  |
  v
Output (.json file or stdout)
```

## Key Types

### Domain Model (`src/model/`)

| Type | Purpose |
|------|---------|
| `PolicyDocument` | Root: sections[], metadata, source fingerprint |
| `PolicySection` | Heading node: title, depth, requirements[], children[] |
| `PolicyRequirement` | Atomic requirement: text, stable_id, citations[], source_line |
| `Citation` | Extracted reference: id, url, context |
| `TraceLink` | Source line -> OSCAL element mapping |
| `TraceIndex` | Collection of TraceLinks for a document |

### Error Types (`src/error.rs`)

`ForgeError` has 16 variants covering: IO, Parse, Serialization, Validation, CatalogBuild, ComponentDefinitionBuild, BackMatter, Schema, Semantic, and more. Uses `thiserror` for `Display`/`Error` derivation.

### OSCAL Output

All OSCAL types use `serde::Serialize` for JSON output. Key envelopes:
- `CatalogEnvelope` wraps catalog + metadata + back-matter
- `ComponentDefinitionEnvelope` wraps component-definition + metadata + back-matter

## Testing

- **660 total tests** (529 unit + 131 integration), all passing
- **Inline unit tests**: `#[cfg(test)]` modules in every source file
- **Integration tests**: `tests/` directory (pipeline, CLI, adversarial, golden files, traceability)
- **Benchmarks**: `benches/` (atomize, uuid, pipeline) using Criterion
- **Snapshot tests**: `insta` for golden file regression testing
- **Adversarial tests**: Binary files, null bytes, oversized input, empty files

## Performance

Benchmark results (from `cargo bench`):
- Full catalog pipeline (50-page synthetic): target < 100ms
- Atomization (1000 requirements): sub-millisecond per requirement
- UUID generation (1000 IDs): sub-microsecond per ID

## Architecture Decisions

1. **Functional pipeline**: Each stage takes ownership and returns enriched data (no shared mutable state)
2. **Deterministic output**: UUID v5 with content-based namespace ensures identical input always produces identical output
3. **Schema-first validation**: OSCAL JSON schemas embedded at compile time via `include_str!`
4. **Post-processing traceability**: Trace links injected after OSCAL generation to keep generation logic clean
5. **Lazy regex compilation**: `LazyLock<Regex>` for patterns used in atomization and citation extraction
