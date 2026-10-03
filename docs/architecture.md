# FORGE Architecture

FORGE (Framework for OSCAL Risk & Governance Execution) is a Rust CLI that converts Markdown security policy documents into OSCAL (Open Security Controls Assessment Language) artifacts. This document describes the pipeline stages, crate/module structure, and data flow so a new developer can navigate the codebase.

## Overview

```
Markdown Policy (.md)
        │
        ▼
┌─────────────────────────────────────────────────────────────┐
│                     CORE PIPELINE                           │
│                                                             │
│  Ingest → Parse (sections + clauses) → Assemble → Atomize   │
│    → Assign UUIDs → Extract Citations → Detect Modality     │
│    → Extract Parameters                                     │
└──────────────────────┬──────────────────────────────────────┘
                       │
              ┌────────┴────────┐
              ▼                 ▼
     Catalog Pipeline    Component Pipeline
              │                 │
              ▼                 ▼
      OSCAL Catalog     Component Definition
              │                 │
              └────────┬────────┘
                       ▼
              Validate (schema + semantic)
                       │
                       ▼
              Serialize (JSON / XML / YAML)
```

## Crate Structure

FORGE is a single-crate project (`forge`). All code lives under `src/`:

```
src/
├── main.rs              — CLI entry point, exit code mapping
├── lib.rs               — Public API, re-exports for downstream consumers
├── cli/                 — CLI subcommand implementations
│   ├── mod.rs           — Cli struct, Commands enum, execute() dispatcher
│   ├── convert.rs       — `forge convert` — single + batch conversion
│   ├── diff.rs          — `forge diff` — compare two OSCAL artifacts
│   ├── export.rs        — `forge export` — format conversion (JSON↔XML↔YAML)
│   ├── output.rs        — Output writing helpers (stdout vs file)
│   ├── profile.rs       — `forge profile` — generate OSCAL Profile from catalog
│   ├── resolve.rs       — `forge resolve` — resolve profile via oscal-cli
│   ├── trace.rs         — `forge trace` — source-to-OSCAL traceability report
│   └── validate.rs      — `forge validate` — schema + round-trip validation
├── pipeline.rs          — Pipeline orchestrator: run_catalog_pipeline, run_component_pipeline
├── ingest/              — Stage 1: file ingestion
│   └── mod.rs           — ingest_file(), IngestedDocument, format/size/encoding validation
├── parse/               — Stage 2-3: structural extraction + atomization
│   ├── mod.rs           — extract_sections(), SectionNode (heading tree)
│   ├── clauses.rs       — extract_clauses(), list/table/paragraph extraction
│   ├── atomize.rs       — atomize_document(), compound statement splitting
│   └── modality.rs      — annotate_modalities(), RFC 2119 verb detection
├── model/               — Stage 4: internal domain model
│   ├── mod.rs           — PolicyDocument, PolicySection, PolicyRequirement, etc.
│   ├── assemble.rs      — assemble_document() — builds PolicyDocument from parsed data
│   ├── frontmatter.rs   — YAML frontmatter extraction
│   └── trace.rs         — TraceLink, SourceLocation, traceability model
├── uuid.rs              — Stage 5: UUID v5 generation for stable identifiers
├── citation.rs          — Stage 6: URL and reference extraction from requirements
├── parameter/           — Stage 7: parameterizable value extraction
│   ├── mod.rs           — extract_parameters(), PolicyParameter
│   └── matchers.rs      — Regex-based parameter pattern matchers
├── oscal/               — OSCAL output generation
│   ├── mod.rs           — Public exports: build_catalog, build_component_definition, etc.
│   ├── catalog.rs       — OscalCatalog, OscalGroup, OscalControl, build_catalog()
│   ├── component_definition.rs — ComponentDefinition, DocumentaryComponent
│   ├── implemented_requirements.rs — ImplementedRequirement generation
│   ├── metadata.rs      — OscalMetadata, assemble_metadata()
│   ├── parts.rs         — OscalPart, OscalProp — control statement parts
│   ├── back_matter.rs   — BackMatter, BackMatterResource, citation→resource mapping
│   ├── profile.rs       — OscalProfile, build_profile(), control selection
│   ├── assessment_plan.rs — AssessmentPlan generation
│   ├── trace_embedding.rs — Embed trace props into OSCAL controls
│   └── test_utils.rs    — Test fixtures and helpers
├── oscal_cli/           — External oscal-cli integration
│   ├── mod.rs           — OscalCliInfo, trait-based abstractions
│   ├── detector.rs      — PATH lookup and version detection
│   └── invoker.rs       — Subprocess invocation for profile resolve
├── export/              — Serialization layer
│   ├── mod.rs           — Re-exports for XML and YAML
│   ├── xml_serializer.rs — OSCAL→XML via quick-xml
│   ├── xml_deserializer.rs — XML→OSCAL (round-trip support)
│   └── yaml.rs          — OSCAL↔YAML via serde_yaml_ng
├── validate/            — Schema and semantic validation
│   ├── mod.rs           — run_full_validation(), model auto-detection
│   ├── error_types.rs   — ValidationReport, ValidationErrorCategory
│   ├── formatter.rs     — Human-readable and JSON error rendering
│   ├── report.rs        — ValidationReport construction
│   └── semantic.rs      — Semantic checks (orphaned links, missing refs)
├── round_trip/          — Round-trip validation (JSON→XML→YAML→JSON)
│   ├── mod.rs           — Public API: run_round_trip_chain, compare_oscal_json
│   ├── chain.rs         — Multi-format conversion chain
│   ├── comparator.rs    — OSCAL-aware semantic equality comparison
│   ├── divergence.rs    — Divergence classification and tracking
│   ├── log.rs           — Divergence log writing
│   └── rules.rs         — OscalComparisonRules
├── diff/                — Artifact comparison
│   ├── mod.rs           — Diff engine public API
│   ├── engine.rs        — Semantic diff between OSCAL artifacts
│   ├── extractor.rs     — Extract comparable elements from OSCAL
│   ├── formatter.rs     — Diff output formatting
│   └── types.rs         — DiffResult, Change types
├── trace/               — Traceability reporting
│   ├── mod.rs           — Trace report public API
│   ├── extractor.rs     — Extract trace links from OSCAL artifacts
│   ├── formatter.rs     — Human-readable trace report
│   ├── report.rs        — TraceReport data structure
│   ├── resolver.rs      — Resolve source locations from trace data
│   └── walker.rs        — Walk OSCAL element tree for trace extraction
├── batch/               — Batch conversion (parallel processing)
│   ├── mod.rs           — Public exports
│   ├── orchestrator.rs  — run_batch_conversion() with rayon parallelism
│   ├── formatter.rs     — Batch summary formatting
│   ├── output_naming.rs — Auto-generated output filenames
│   └── summary.rs       — BatchSummary, FileOutcome, FileResult
├── summary/             — Conversion statistics dashboard
│   ├── mod.rs           — ConversionStatistics, count_catalog_controls
│   └── format.rs        — Dashboard text rendering
├── sanitize.rs          — Input sanitization helpers
├── io.rs                — Shared I/O constants and utilities
├── error.rs             — ForgeError enum (all error variants + exit codes)
├── types.rs             — Shared enums: Strategy, OutputFormat, OscalModelType
└── testing/             — Test utilities (not public API)
    ├── mod.rs
    └── semantic_eq.rs   — Semantic equality for testing
```

## Pipeline Stages

The core conversion pipeline is defined in `src/pipeline.rs`. Two pipelines share a common preparation stage and then diverge:

### Shared Preparation (prepare_document)

This function encapsulates steps 1-9, used by both catalog and component pipelines:

| Step | Module | Function | Description |
|------|--------|----------|-------------|
| 1 | `ingest` | `ingest_file()` | Read file, validate encoding/format/size, compute SHA-256 fingerprint |
| 2 | `ingest` | `reconstruct_content()` | Join source lines back into a content string |
| 3 | `parse` | `extract_sections()` | Parse Markdown headings into a hierarchical tree of `SectionNode` using pulldown-cmark event-based parser with stack-based O(n) tree construction |
| 4 | `parse` | `extract_clauses()` | Extract list items, tables, and paragraphs from Markdown for clause-level structure detection |
| 4b | (inline) | — | Error if no sections AND no clauses found (`NoStructureDetected`) |
| 5 | `model` | `assemble_document()` | Build `PolicyDocument` from ingested data, sections, and clauses; extract YAML frontmatter metadata |
| 6 | `parse` | `atomize_document()` | Split compound requirements ("must X and must Y") into atomic units using regex-based conjunction+verb detection |
| 7 | `uuid` | `assign_stable_ids()` | Assign deterministic UUID v5 identifiers to every requirement (content-addressed, stable across re-conversions) |
| 7b | `citation` | `extract_citations()` | Scan requirement text for URLs and bibliographic references |
| 7c | `parse` | `annotate_modalities()` | Classify requirements as Normative (must/shall) or Advisory (should/may) via RFC 2119 verb heuristics |
| 7d | `parameter` | `extract_parameters()` | Extract configurable parameters (time windows, thresholds, frequencies, quantities) from requirement prose |

### Catalog Pipeline (run_catalog_pipeline)

After preparation, the catalog pipeline:

| Step | Module | Description |
|------|--------|-------------|
| 8 | `oscal::catalog` | `build_catalog()` — Map `PolicyDocument` → `OscalCatalog` with groups, controls, and statement parts |
| 8b | `oscal::trace_embedding` | `embed_trace_in_catalog()` — Embed trace props/links into catalog controls for provenance |
| 9 | `oscal::metadata` | `assemble_metadata()` — Build OSCAL metadata block from document metadata |
| 10 | `oscal::back_matter` | `generate_back_matter()` — Convert extracted citations into OSCAL back-matter resources |
| 11 | `oscal` | Assemble `OscalCatalog` into `CatalogEnvelope` (the top-level OSCAL JSON object) |
| 12 | `validate` | Schema validation against NIST OSCAL JSON schema + semantic checks |
| 12b | `oscal::assessment_plan` | Optional: generate Assessment Plan skeleton if `--import-ssp` was provided |
| 13 | `export` | Serialize to requested format (JSON / XML / YAML) |

### Component Pipeline (run_component_pipeline)

After preparation, the component pipeline:

| Step | Module | Description |
|------|--------|-------------|
| 10 | `oscal::component_definition` | `build_component_definition()` — Map `PolicyDocument` → `ComponentDefinition` with documentary components and implemented requirements |
| 11 | `validate` | Schema validation against component-definition JSON schema |
| 11b | `oscal::assessment_plan` | Optional: generate Assessment Plan skeleton if `--import-ssp` was provided |
| 12 | `export` | Serialize to requested format (JSON / XML / YAML) |

## Key Data Types

### Core Domain Model (src/model/mod.rs)

- **`PolicyDocument`** — Top-level internal representation. Contains `id`, `metadata`, and a tree of `sections`. Passed through pipeline stages via functional transformation (each stage takes ownership and returns an enriched version).
- **`PolicySection`** — Hierarchical section mapped from a Markdown heading. Contains `title`, `heading_level`, `source_line`, `body_text`, nested `children`, and `requirements`.
- **`PolicyRequirement`** — Atomic policy requirement. Fields: `stable_id` (UUID, populated at step 7), `text`, `source_line`, `nesting_depth`, `atom_index`, `parent_text` (original compound text if split), `citations`, `modality`, `parameters`.
- **`PolicyParameter`** — Configurable value extracted from requirement prose (time windows, thresholds, frequencies, quantities).
- **`Citation`** — URL or bibliographic reference with `id`, `text`, `url`, `source_requirement_id`.
- **`DocumentMetadata`** — Title, version, author, date, source path, content hash.

### Ingestion (src/ingest/mod.rs)

- **`IngestedDocument`** — File read with SHA-256 fingerprint, line-by-line with 1-based line numbers.
- **`SourceLine`** — Single line with `number` and `text`.

### Parsing (src/parse/mod.rs)

- **`SectionNode`** — Raw section tree from Markdown heading extraction (before domain model assembly).
- **`ExtractedContent`** — Clause-level extraction: list items, tables, paragraphs.
- **`AtomizationResult`** — Result of splitting one compound requirement into atomic parts.

### OSCAL Model (src/oscal/)

- **`OscalCatalog`** — OSCAL Catalog representation: `uuid`, `metadata`, `controls`, `groups`, `back_matter`.
- **`CatalogEnvelope`** — Top-level wrapper: `{ "catalog": OscalCatalog }`.
- **`OscalGroup`** — Control group with `id`, `title`, `controls`, nested `groups`, `props`.
- **`OscalControl`** — Single control with `id`, `title`, `parts`, `props`.
- **`ComponentDefinition`** / `ComponentDefinitionEnvelope` — Component definition OSCAL model.
- **`OscalProfile`** — OSCAL Profile for control selection.
- **`BackMatter`** — Collection of `BackMatterResource` (citations, URLs).
- **`AssessmentPlanEnvelope`** — Assessment Plan OSCAL model.

### Types (src/types.rs)

- **`Strategy`** — Conversion target: `Catalog` or `Component`.
- **`OutputFormat`** — Serialization format: `Json`, `Xml`, `Yaml`.
- **`OscalModelType`** — General detected model: `Catalog`, `ComponentDefinition`, `Profile`, `SystemSecurityPlan` or `Mapping`. Assessment Results uses a dedicated builder validator. General validation support does not imply support by the two-model `export` command.

### Pipeline Output (src/pipeline.rs)

- **`PipelineOutput`** — Return value from both pipelines: `content` (serialized string), `format`, `secondary_outputs` (e.g., assessment plans), `statistics` (sections parsed, controls generated, etc.).
- **`SecondaryOutput`** — Auxiliary artifact with `filename` and `content`.

## Local workspace runtime

`workspace::launch` selects one API major before capture, credentials or listening.
The current source publishes API v1 **1.2.0 / 39 operations** and API v2
**2.3.0 / 57 operations**. The source workflow adds six v2 routes to the retained
metadata and inspection surfaces; it does not move v1 callers to v2. These local
contract versions are separate from the product release version. See the
[workspace codemap](CODEMAPS.md#workspace-source-bundle-paths) and
[API compatibility policy](api/compatibility.md).

| Method and source route | Handoff |
| --- | --- |
| POST `/api/v2/project/source-bundle-exports` | 202 ordinary export preparation; later single-file confirmation saves the source JSON |
| GET `/api/v2/project/source-bundle-exports/{operation_id}/download` | 200 exact committed bytes from the private source-JSON family |
| POST `/api/v2/project/source-bundle-imports` | 200 complete restore preview, replacement, input bindings and directory intentions |
| POST `/api/v2/project/bundle-restores/{preview_id}/commit` | 202 only after exact confirmed intent and the original reply are durable |
| GET `/api/v2/project/bundle-restores/{operation_id}` | 200 safe persisted outcome under current same-root session authority |
| POST `/api/v2/project/bundle-restores/{operation_id}/cancel` | 200 cancellation acknowledgment/outcome, not proof of rollback |

### Capture and lock ownership

`services::Snapshot` admits the complete source planning union before registered
resource reads. `source_bundles` preserves exact bytes, `source_validation` admits
the proposed registered closure, and `source_bundle_effects` builds a complete
off-Store plan. Session receipt retention is owned by `effects::Store`, with
`effects::source_receipts` handling source previews and volatile job facts.

Participating workspace capture/download paths acquire the State's project
`RwLock` read lease before Store or project I/O; ordinary single-file commits and
confirmed restore workers use its write lease. Source workers use short Store
sections for admission, replay, cancellation and measured staging progress;
native journal/staging/publication I/O runs outside that Store lock. Pure restore
status and cancellation remain reachable without a project lease. A retained
reply can be read under Store alone; the lock is released before acquiring new
project authority. The separate journal owner serializes durable replacements.

This fence covers participating workspace paths. External CLI commands and
editors can observe mixed whole-file generations during index-last publication;
the runtime does not claim a shared CLI barrier or globally atomic multi-file
visibility.

### Confirmed restore and restart

The preview supplies a nonauthorizing operation ID before confirmation. Store
checks the one-time receipt, original observed batch version and complete capacity;
`Root::accept_restore` durably records accepted intent before worker dispatch and
HTTP 202. The sealed plan then stages all targets, verifies original generations,
publishes resources in authorial order with the index last, verifies the complete
proposed result and records the committed decision.

Before that decision, interruption attempts conditional rollback of verified owned
objects; foreign replacements are not overwritten. After the decision, settlement
cannot roll back or relabel a committed write as none. `write_outcome` and
`cleanup_state` remain separate facts. Uncertain journal, ownership or cleanup
blocks participating project access; a cancellation acknowledgment or a missing
outcome does not prove no write. Staging counts are measured files, not publication
percentages, and cooperative deadlines do not preempt parsers or syscalls.

On Unix, `TransactionState::open_qualified` checks OS-selected private per-user
state, held no-link ancestry, owner/mode constraints and an exclusive `flock` on
the root-bound journal lock before index capture, credentials or listening. A
writable API2 launch may settle accepted state; a read-only launch performs no
recovery writes and refuses unresolved state. Fresh same-root authenticated reads
can use the known outcome ID after restart; old tokens and preview authority are
not revived. The Windows native restore port returns typed unavailable rather
than using a weaker publication fallback.

The [source workflow guide](workspace-source-bundles.md) defines the finite inline
profile and whole planning/capture/retention limits. Larger staged transfers,
cross-platform crash/rollback qualification, human acceptance and the final
integrated documentation gate remain open; this architectural description supplies
no execution or acceptance result.

## CLI Commands

The table below highlights conversion utilities. The current command tree also
includes drift, migration, mapping, assessment, linkage, lifecycle, applicability,
framework, policy, authoring, suggestions, workspace and configuration commands.
See `src/cli/mod.rs`, the [usage guide](usage-guide.md) and the
[domain/workspace codemap](CODEMAPS.md#current-domain-and-workspace-entrypoints).

| Command | Module | Description |
|---------|--------|-------------|
| `forge convert` | `cli/convert.rs` | Convert Markdown policy → OSCAL. Supports batch mode with `--jobs` for parallel processing via rayon. |
| `forge export` | `cli/export.rs` | Convert Catalog or Component Definition typed projections between JSON, XML and YAML. |
| `forge validate` | `cli/validate.rs` | Validate OSCAL JSON against schemas. Supports `--round-trip` for JSON→XML→YAML→JSON fidelity check via oscal-cli. |
| `forge resolve` | `cli/resolve.rs` | Resolve an OSCAL Profile into a flat Catalog via oscal-cli. |
| `forge profile` | `cli/profile.rs` | Generate an OSCAL Profile by selecting controls from a source Catalog. |
| `forge diff` | `cli/diff.rs` | Compare two OSCAL artifacts, show semantic differences. |
| `forge trace` | `cli/trace.rs` | Show traceability between OSCAL elements and source policy locations. |

## Data Flow

```
┌────────────────────────────────────────────────────────────────────┐
│  main.rs                                                           │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  cli/execute()                                                │  │
│  │  ┌────────────────────────────────────────────────────────┐  │  │
│  │  │  convert::execute_dispatch()                           │  │  │
│  │  │  ┌──────────────────────┐  ┌────────────────────────┐  │  │  │
│  │  │  │ Single file:         │  │ Batch (jobs > 1):      │  │  │  │
│  │  │  │ pipeline::run_*      │  │ batch::run_batch_*     │  │  │  │
│  │  │  └──────────┬───────────┘  └───────────┬────────────┘  │  │  │
│  │  │             │                           │               │  │  │
│  │  │  ┌──────────▼───────────────────────────▼────────────┐  │  │  │
│  │  │  │  pipeline::prepare_document()                     │  │  │  │
│  │  │  │  ingest → parse → assemble → atomize → uuid →     │  │  │  │
│  │  │  │  citations → modality → parameters                │  │  │  │
│  │  │  └──────────────────────┬────────────────────────────┘  │  │  │
│  │  │                       │                                 │  │  │
│  │  │     ┌─────────────────┼─────────────────┐              │  │  │
│  │  │     ▼                 ▼                 ▼              │  │  │
│  │  │  oscal::           oscal::          validate::        │  │  │
│  │  │  build_catalog()   build_comp_def() run_validation()  │  │  │
│  │  │     │                 │                 │              │  │  │
│  │  │     └─────────────────┼─────────────────┘              │  │  │
│  │  │                       ▼                                │  │  │
│  │  │              export::{json,xml,yaml}                   │  │  │
│  │  └───────────────────────┬────────────────────────────────┘  │  │
│  │                          ▼                                   │  │
│  │                  cli/output::write()                         │  │
│  └──────────────────────────┬───────────────────────────────────┘  │
│                             ▼                                      │
│                      stdout or file                                │
└────────────────────────────────────────────────────────────────────┘
```

### Functional Transformation Pattern

The `PolicyDocument` flows through the pipeline via ownership-based functional transformation:

```
IngestedDocument ──reconstruct──> String ──extract_sections──> Vec<SectionNode>
       │
       └──extract_clauses──> ExtractedContent
                                    │
                                    ▼
                    assemble_document() ──> PolicyDocument
                                    │
                        ┌───────────┼───────────┐
                        ▼           ▼           ▼
                   atomize_      assign_     extract_
                   document()    stable_ids  citations()
                        │           │           │
                        ▼           ▼           ▼
                   PolicyDoc ──> PolicyDoc ──> PolicyDoc
                        │
                   annotate_
                   modalities()
                        │
                        ▼
                   PolicyDoc
                        │
                   extract_
                   parameters()
                        │
                        ▼
                   PolicyDoc ──> OSCAL generation
```

Each enrichment stage takes ownership of the document, mutates it (or returns a new version), and passes it forward. This makes the pipeline deterministic and easy to test in isolation.

## External Dependencies

### NIST oscal-cli

FORGE integrates with the NIST `oscal-cli` tool for:
- **Profile resolution** (`forge resolve`) — resolves OSCAL Profiles into flat Catalogs
- **Round-trip validation** (`forge validate --round-trip`) — converts JSON→XML→YAML→JSON and compares for semantic equality

Detection is automatic via PATH lookup, or explicit via `--oscal-cli-path`. The integration uses trait-based abstractions (`src/oscal_cli/`) for testability.

### Embedded OSCAL Schemas

FORGE embeds pinned NIST OSCAL v1.2.3 JSON schemas for offline validation. Supported older declarations use this same baseline; metadata never selects or downloads another schema. See the [OSCAL compatibility guide](OSCAL_COMPATIBILITY.md) for model support and asset provenance.

## Testing Strategy

- **Unit tests** — Inline `#[cfg(test)]` modules in each source file
- **Integration tests** — `tests/` directory with fixture-based tests
- **Golden-file tests** — `insta` snapshots for OSCAL output stability
- **Property-based tests** — `proptest` for edge case coverage
- **Benchmarks** — `criterion` benchmarks for atomization, UUID generation, pipeline, XML, export, and parameter extraction (`benches/`)

## Error Handling

All errors are represented as `ForgeError` (src/error.rs), a `thiserror` enum with categorized exit codes:

| Exit Code | Category | Examples |
|-----------|----------|----------|
| 0 | Success | — |
| 1 | Input/IO, observed changes or review required | FileNotFound, Serialization, DiffHasChanges, LifecycleActionRequired |
| 2 | Parse, domain or usage | Parse, CatalogBuild, Lifecycle, MissingRequiredArgument |
| 3 | Validation/Config | Validation, SchemaValidation, Config |
| 4 | External tool unavailable | OscalCliNotFound, OscalCliNotFunctional |
