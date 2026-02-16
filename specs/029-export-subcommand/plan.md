# Implementation Plan: Export Subcommand (WI-29)

**Branch**: `029-export-subcommand` | **Date**: 2026-02-15 | **Spec**: [spec.md](spec.md)
**Input**: PRD [029-prd-export-subcommand](../../docs/PRD/029-prd-export-subcommand.md), AR [029-ar-export-subcommand](../../docs/AR/029-ar-export-subcommand.md), SEC [029-sec-export-subcommand](../../docs/SEC/029-sec-export-subcommand.md)

## Summary

Implement `forge export <input> --format <json|xml|yaml> [--output <path>]` as a new CLI subcommand that converts existing OSCAL artifacts between JSON, XML, and YAML formats. The subcommand uses a single generic deserialize-reserialize pipeline through the internal OSCAL model, with output validation against OSCAL v1.2.0 schemas. This is distinct from `forge convert` which operates on source policy documents.

The implementation reuses existing serialization infrastructure from WI-26 (XML) and WI-27 (YAML), adds XML deserialization via `quick-xml` serde integration, and wires it all through a new `export` CLI module. No new dependencies are required — `quick-xml`'s `serde` feature is enabled on the existing dependency.

## Technical Context

**Language/Version**: Rust, Edition 2024, stable 1.93.0
**Primary Dependencies**: clap 4.x (derive), serde 1.0.228, serde_json 1.0.149, quick-xml 0.37 (add `serde` feature), serde_yaml_ng 0.10 (aliased as serde_yaml), thiserror 2.0.18
**Storage**: N/A — reads/writes local files only
**Testing**: `cargo test` (unit + integration), insta (snapshots), tempfile (temp dirs)
**Target Platform**: Cross-platform CLI (Linux, macOS, Windows)
**Project Type**: Single Rust binary crate
**Performance Goals**: Format conversion of 100KB–1MB artifact in under 1 second
**Constraints**: No network dependency; all validation offline with bundled schemas; no new crate dependencies (only enable feature on existing quick-xml)
**Scale/Scope**: 9 format pair combinations (3×3), 2 OSCAL model types (Catalog, ComponentDefinition)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Crate-First Architecture | PASS | Export module lives within existing crate structure (`src/cli/export.rs`, `src/export/`); no new crate needed for this thin orchestration layer |
| II. Rust-First | PASS | Pure Rust implementation; no FFI or unsafe code |
| III. Contract-First Development | PASS | Contracts defined in AR (interface definitions, `ExportArgs`, `detect_format`, `export_artifact`); implemented here |
| IV. Test-First Development | PASS | TDD mandatory; tests written before implementation per task ordering |
| V. Complete Implementation | PASS | All 9 format pairs, validation, error handling, CLI integration covered in tasks |
| VI. Performance-First Design | PASS | Sub-1-second target for typical artifacts; no streaming needed for MVP |
| VII. Security-First Design | PASS | SEC review completed; XXE prevention verified (SEC-1), input validation (SEC-3–5), output validation (SEC-7) |
| VIII. Error Handling Standards | PASS | New `ForgeError` variants with descriptive messages; `thiserror` enum; no `.unwrap()` in production |
| IX. Observability | PASS | `tracing` logging at INFO/DEBUG levels for format detection and pipeline stages |
| X. Simplicity & Pragmatism | PASS | Single generic pipeline (no 9 separate functions); simple match on format enum; YAGNI |
| XI. Current Dependency Policy | PASS | No new dependencies; enable `serde` feature on existing quick-xml 0.37 |

**Gate Result: PASS** — No violations. Proceed to Phase 0.

## Project Structure

### Documentation (this feature)

```text
specs/029-export-subcommand/
├── plan.md              # This file
├── spec.md              # Feature specification (populated from PRD)
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── export.rs        # Interface contract (Rust trait/type definitions)
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── cli/
│   ├── mod.rs               # MODIFY: Add Export variant to Commands enum
│   ├── convert.rs            # UNCHANGED
│   ├── validate.rs           # UNCHANGED
│   └── export.rs             # NEW: Export subcommand handler
├── export/
│   ├── mod.rs                # MODIFY: Add xml_deserializer module, re-export new functions
│   ├── xml_serializer.rs     # UNCHANGED (reuse for XML output)
│   ├── xml_deserializer.rs   # NEW: XML deserialization via quick-xml serde
│   └── yaml.rs               # UNCHANGED (reuse serialize_to_yaml + deserialize_from_yaml)
├── pipeline.rs               # UNCHANGED (reuse write_output directly — already public)
├── error.rs                  # MODIFY: Add export-specific error variants
├── lib.rs                    # MODIFY: Re-export new public items
└── validate/
    └── mod.rs                # UNCHANGED (reuse validate_artifact, detect_model_type)

tests/
├── fixtures/
│   └── export/               # NEW: Test fixture files for export
│       ├── catalog.json      # Valid OSCAL Catalog JSON fixture
│       ├── catalog.xml       # Valid OSCAL Catalog XML fixture
│       ├── catalog.yaml      # Valid OSCAL Catalog YAML fixture
│       ├── component.json    # Valid Component Definition JSON fixture
│       ├── component.xml     # Valid Component Definition XML fixture
│       └── component.yaml    # Valid Component Definition YAML fixture
└── export_integration.rs     # NEW: CLI integration tests for forge export

benches/
└── export_bench.rs            # NEW: Performance benchmark for export pipeline (criterion)
```

**Structure Decision**: Single project structure. Export is a thin CLI layer added to the existing binary crate. New files are `src/cli/export.rs` (handler), `src/export/xml_deserializer.rs` (XML deserialization), test fixtures, and integration tests. All other changes are modifications to existing files.

## Complexity Tracking

No constitution violations to justify. The implementation follows the simplest possible approach: a single generic deserialize-reserialize pipeline through the internal OSCAL model, reusing all existing infrastructure.
