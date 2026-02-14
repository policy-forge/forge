# Data Model: Performance Benchmark (WI-24)

**Feature Branch**: `024-performance-benchmark`
**Date**: 2026-02-13

## Overview

No new domain model types are introduced. This work item creates test infrastructure (fixture + benchmarks) that operates on existing types from the FORGE pipeline.

## Existing Types Used

### Input Types

| Type | Module | Role in Benchmark |
|------|--------|-------------------|
| `IngestedDocument` | `ingest/mod.rs` | Output of ingest stage; input to parse/assemble stages |
| `SourceLine` | `ingest/mod.rs` | Lines within `IngestedDocument` |
| `SectionNode` | `parse/mod.rs` | Output of section extraction; input to assembly |
| `ExtractedContent` | `parse/clauses.rs` | Output of clause extraction; input to assembly |
| `PolicyDocument` | `model/mod.rs` | Central domain model; flows through atomize → UUID → citations → catalog |
| `PolicySection` | `model/mod.rs` | Section within `PolicyDocument` |
| `PolicyRequirement` | `model/mod.rs` | Requirement within `PolicySection` |
| `DocumentMetadata` | `model/mod.rs` | Metadata within `PolicyDocument` |
| `Citation` | `model/mod.rs` | Citation attached to `PolicyRequirement` |

### Output Types

| Type | Module | Role in Benchmark |
|------|--------|-------------------|
| `OscalCatalog` | `oscal/catalog.rs` | Catalog output from `build_catalog()` |
| `CatalogEnvelope` | `oscal/catalog.rs` | Top-level JSON envelope wrapping `OscalCatalog` |
| `OscalGroup` | `oscal/catalog.rs` | Group within catalog (maps from `PolicySection`) |
| `OscalControl` | `oscal/catalog.rs` | Control within group (maps from `PolicyRequirement`) |
| `OscalMetadata` | `oscal/metadata.rs` | Metadata for catalog envelope |
| `BackMatter` | `oscal/back_matter.rs` | Back matter section (currently empty for this pipeline) |
| `TraceLinkCollection` | `model/trace.rs` | Trace links captured during catalog generation |

## Synthetic Fixture Structure

The synthetic 50-page fixture is a Markdown document, not a Rust type. Its structure must exercise all pipeline stages:

```text
synthetic-50page-policy.md (~150KB)
├── YAML Frontmatter
│   ├── title: "Comprehensive Information Security Policy"
│   ├── version: "1.0.0"
│   ├── author: "Policy Division"
│   └── date: "2026-01-01"
├── H1: Title
├── H2: Section 1 (Access Control)
│   ├── H3: Subsection 1.1 (User Account Management)
│   │   ├── Requirement 1.1.1 (atomic, with citation)
│   │   ├── Requirement 1.1.2 (compound: "must X and must Y")
│   │   └── ...
│   ├── H3: Subsection 1.2 (Authentication)
│   │   └── ...
│   └── Table: Role-Responsibility Matrix
├── H2: Section 2 (Data Protection)
│   └── ...
├── ... (10 H2 sections total)
└── H2: Section 10 (Compliance Monitoring)
```

### Content Distribution Targets

| Content Type | Target Count | Pipeline Stage Exercised |
|--------------|-------------|--------------------------|
| H2 sections | 10 | `parse::extract_sections` |
| H3 subsections | ~40 | `parse::extract_sections` |
| H4 sub-subsections | ~10 | `parse::extract_sections` |
| Numbered requirements | ~200 | `parse::extract_clauses`, `model::assemble_document` |
| Compound statements | ~20 | `parse::atomize_document` |
| Citations/references | ~30 | `citation::extract_citations` |
| Tables | ~10 | `parse::extract_clauses` |
| Total words | ~25,000 | All stages |
| Total characters | ~150,000 | All stages |

## Relationships

```text
Fixture File (on disk)
    │
    ▼
IngestedDocument ──── reconstruct_content() ────► content: String
    │                                               │
    │                                               ▼
    │                                    extract_sections() ──► Vec<SectionNode>
    │                                               │
    │                                    extract_clauses() ──► Vec<ExtractedContent>
    │                                               │
    ▼                                               ▼
         assemble_document() ──────────────────► PolicyDocument
                                                    │
                                         atomize_document()
                                                    │
                                         assign_stable_ids()
                                                    │
                                         extract_citations()
                                                    │
                                                    ▼
                                         build_catalog() ──► OscalCatalog
                                                    │
                                         embed_trace_in_catalog()
                                                    │
                                         assemble_metadata() ──► OscalMetadata
                                                    │
                                         generate_back_matter() ──► BackMatter
                                                    │
                                         CatalogEnvelope assembly
                                                    │
                                         serde_json::to_string_pretty()
                                                    │
                                                    ▼
                                              JSON String
```
