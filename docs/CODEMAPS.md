# FORGE Architecture Codemap

Quick architectural reference for navigating the Forge codebase.

## Module Overview

```
src/
  main.rs                  CLI entry point (anyhow error handling)
  lib.rs                   Public API re-exports
  pipeline.rs              Pipeline orchestration (catalog + component)
  error.rs                 Categorized ForgeError enum
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
    mod.rs                 XML/YAML module declarations and re-exports
    xml_serializer.rs      Typed OSCAL model to XML
    xml_deserializer.rs    XML to typed OSCAL model
    yaml.rs                Generic serde-based YAML serialization/deserialization

  validate/                Validation layer
    mod.rs                 Orchestration (schema + semantic)
    error_types.rs         ValidationErrorCategory enum
    formatter.rs           Human-friendly error formatting
    report.rs              Validation report generation
    semantic.rs            Semantic checks (orphan links, missing fields)
```

## Current domain and workspace entrypoints

The diagram below describes conversion. The CLI also exposes domain workflows;
see `src/cli/mod.rs` for the current command tree. These modules are implemented
technical surfaces, not claims that their full PRD acceptance gates are complete.

| Module | Purpose |
|---|---|
| `applicability/` | Human-reviewed framework applicability and policy-gap analysis |
| `assessment_results/` | Human-authored Assessment Results and revision review |
| `authoring/` | Human assignments, drafting plans and traceable skeletons |
| `framework/` | Read-only framework revision impact |
| `lifecycle/` | Deterministic local lifecycle records and review queues |
| `linkage/` | Evidence/implementation indexes and maintenance reports |
| `mapping/` | Human-reviewed Control Mapping workflows |
| `migration/` | Read-only source-policy revision analysis |
| `policy/` | Composition of local, hash-pinned Markdown components |
| `reuse/` | Read-only retrieval of operator-supplied local corpus excerpts |
| `suggest/` | Bounded, quarantined offline suggestion artifacts |
| `workspace/` | Confined selected-major queries, single-file receipts and confirmed source restore |

Within `workspace/`, `services.rs` captures registered inputs; `domain.rs` stages
private copies for domain engines; and `actions.rs` prepares effects without
project publication. `inspection.rs`, `lifecycle.rs`, `impact.rs` and
`provenance.rs` project read-only registered views. `effects.rs` retains session
receipts and confirms exact-byte single-file writes. Metadata bundle comparison
does not authorize import; source restore has a separate complete preview and
batch-confirmation path. See the [workspace guide](local-workspace.md) and
[workspace runtime architecture](architecture.md#local-workspace-runtime).

POA&M and suggestion-evaluation modules are absent at the reviewed `532c9e8`
head. Separate draft implementations must not be inferred from this map.

### Workspace source-bundle paths

These are consumed modules in the current source branch, including nested modules
declared with `#[path]`; the map does not establish platform or PRD acceptance.

| Entry point | Responsibility |
| --- | --- |
| [`http.rs`](../src/workspace/http.rs), [`http::source`](../src/workspace/http_source.rs) | Selected-major admission, original deadlines, project leases, six source-route adapters and status/cancellation reads |
| [`services.rs`](../src/workspace/services.rs), [`root.rs`](../src/workspace/root.rs) | Complete planned-path preflight, captured registered generations and held-object confinement |
| [`source_bundles.rs`](../src/workspace/source_bundles.rs) | Strict bounded Bundle3 codec with ordered pin/content/index bijection and exact source bytes |
| [`source_validation.rs`](../src/workspace/source_validation.rs) | Intrinsic role admission and complete proposed registered dependency closure; not universal freshness or human approval |
| [`source_bundle_effects.rs`](../src/workspace/source_bundle_effects.rs) | Off-Store source export and restore planning, full replacement projection and preallocated nonauthorizing outcome ID |
| [`effects.rs`](../src/workspace/effects.rs), [`effects::source_receipts`](../src/workspace/source_receipts.rs) | Shared bounded retention, one-time session receipts, exact-request replay and separate private download families |
| [`root::root_transaction`](../src/workspace/root_transaction.rs), [`root::transaction_state`](../src/workspace/transaction_state.rs) | Qualified Unix durable intent, index-last publication, owned conditional rollback, recovery and safe persisted outcomes |

The [source workflow guide](workspace-source-bundles.md) covers explicit opt-in,
all six routes, confirmation, known-ID recovery and finite capacity. The native
Windows restore port is unavailable; larger staged transfers and full
cross-platform qualification remain open.

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

`ForgeError` categorizes input, parsing, serialization, validation and domain workflow failures. See [`src/error.rs`](../src/error.rs) for the current declared variants and exit-code mapping; this reference does not freeze a variant count. It uses `thiserror` for `Display`/`Error` derivation.

### OSCAL Output

All OSCAL types use `serde::Serialize` for JSON output. Key envelopes:
- `CatalogEnvelope` wraps catalog + metadata + back-matter
- `ComponentDefinitionEnvelope` wraps component-definition + metadata + back-matter

## Testing

- **Historical codemap inventory:** 660 tests (529 unit + 131 integration). This retained count is not a current executed suite result; use the dated [source verification checkpoint](workspace-source-bundle-verification.md) and its exact receipts for separately measured scopes.
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
