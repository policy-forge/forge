# Implementation Plan: OSCAL XML Output

**Branch**: `026-xml-output` | **Date**: 2026-02-15 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/026-xml-output/spec.md`

## Summary

Add OSCAL XML serialization to FORGE using `quick-xml` with manual element construction. This implements PRD requirements M-1 through M-7 by creating an `xml_serializer` module in `src/export/` that serializes existing OSCAL model structs (Catalog, Component Definition) to valid OSCAL v1.2.0 XML. The serializer writes elements in XSD-prescribed order, places UUIDs as XML attributes, includes the OSCAL namespace on the root element, and validates output against OSCAL XSD schemas using `xmllint`. The `forge convert --format xml` command path is wired through the existing format dispatcher in the pipeline.

## Technical Context

**Language/Version**: Rust, Edition 2024, stable 1.93.0
**Primary Dependencies**: quick-xml (latest stable, MIT), existing: clap 4, serde 1.0.228, serde_json 1.0.149, thiserror 2.0.18, tracing 0.1.44, uuid 1.20.0, chrono 0.4
**Storage**: N/A — file output only (stdout or file path)
**Testing**: cargo test (unit + integration), insta (snapshot), xmllint (XSD validation in integration tests)
**Target Platform**: macOS, Linux (CLI tool)
**Project Type**: Single crate (existing workspace)
**Performance Goals**: XML serialization < 50ms for typical policy documents (< 1000 controls); comparable to JSON serialization
**Constraints**: XML elements must follow XSD-prescribed ordering; UUIDs as XML attributes; OSCAL namespace on root element only; no string concatenation for XML construction
**Scale/Scope**: 2 OSCAL model types (Catalog, Component Definition); ~400-600 LOC for serializer module; ~200-300 LOC for tests

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Crate-First Architecture | PASS | Adding to existing single-crate project; `src/export/xml_serializer.rs` module within existing crate boundary |
| II. Rust-First | PASS | Pure Rust implementation using `quick-xml` (MIT); no FFI or unsafe code |
| III. Contract-First Development | PASS | XML serialization function signatures defined in contracts/; error types use existing `ForgeError::Serialization` |
| IV. Test-First Development | PASS | TDD mandatory; unit tests for each serializer function, integration tests with XSD validation |
| V. Complete Implementation | PASS | All tasks must complete before merge; binary completion |
| VI. Performance-First Design | PASS | `quick-xml::Writer` with `new_with_indent` provides efficient streaming serialization; benchmark included |
| VII. Security-First Design | PASS | SEC-1 through SEC-7 from security review; `quick-xml` handles XML escaping automatically; no DTD, no string concatenation |
| VIII. Error Handling Standards | PASS | Uses existing `ForgeError::Serialization(String)` for quick-xml write errors and UTF-8 conversion failures |
| IX. Observability | PASS | DEBUG-level tracing for serialization start/end with artifact type |
| X. Simplicity & Pragmatism | PASS | Manual element construction is the minimum approach that guarantees XSD compliance; justified by OSCAL XML's strict element ordering |
| XI. Current Dependency Policy | PASS | `quick-xml` added at latest stable version; MIT license; no known CVEs |

## Project Structure

### Documentation (this feature)

```text
specs/026-xml-output/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── xml-serializer.md
└── tasks.md             # Phase 2 output (NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── export/
│   ├── mod.rs               # Module declaration (wire xml_serializer)
│   └── xml_serializer.rs    # XML serialization functions (NEW)
├── oscal/
│   ├── catalog.rs           # OscalCatalog, OscalGroup, OscalControl (MODIFY: add Deserialize derive to CatalogEnvelope for US4)
│   ├── component_definition.rs  # ComponentDefinition, DocumentaryComponent (MODIFY: add Deserialize derive to ComponentDefinitionEnvelope for US4)
│   ├── metadata.rs          # OscalMetadata (READ-ONLY)
│   ├── parts.rs             # OscalPart, OscalProp (READ-ONLY)
│   ├── back_matter.rs       # BackMatter, BackMatterResource, OscalLink, Rlink, Prop (READ-ONLY)
│   └── mod.rs               # Module declarations (READ-ONLY)
├── cli/
│   ├── mod.rs               # OutputFormat enum (already has Xml variant)
│   └── convert.rs           # Convert command (MODIFY: remove XML rejection)
├── error.rs                 # ForgeError (READ-ONLY — Serialization variant exists)
└── pipeline.rs              # Pipeline orchestration (MODIFY: add XML serialization path)

tests/
├── xml_catalog_test.rs          # Catalog XML integration tests (NEW)
├── xml_component_test.rs        # Component Definition XML integration tests (NEW)
└── xml_validation_test.rs       # XSD schema validation integration tests (NEW)
```

**Structure Decision**: Single-project layout (Option 1). The XML serializer is a new module within `src/export/`, following the existing crate structure. OSCAL model structs in `src/oscal/` are read-only — no XML-specific serde attributes are added. The CLI already has the `Xml` variant in `OutputFormat`; `convert.rs` currently rejects it and needs modification to accept it.

## Complexity Tracking

No constitution violations. The manual element construction approach is the simplest that satisfies OSCAL XSD compliance requirements (PRD M-5, M-7). A derive-based approach would be simpler in LOC but cannot meet the element ordering and attribute placement requirements.
