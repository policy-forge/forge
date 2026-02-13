# Research: Traceability Model (WI-16)

**Phase**: 0 — Outline & Research
**Date**: 2026-02-13

## Status

No NEEDS CLARIFICATION items exist. The AR (016-ar-traceability-model.md) and SEC review (016-sec-traceability-model.md) provide comprehensive technical decisions. All design questions resolved.

## Decisions

### D-1: Data Structure — Adjacency List with Dual HashMap Indexes

- **Decision**: `Vec<TraceLink>` insertion-order store + `HashMap<String, Vec<TraceLink>>` grouped forward index + `HashMap<String, usize>` reverse index
- **Rationale**: O(1) amortized bidirectional lookup. Standard library only. Append-only `links` Vec preserves insertion order for `iter()`. Forward index stores cloned TraceLinks grouped by requirement to enable `&[TraceLink]` slice return from `by_requirement()`. Reverse index maps `oscal_element_id` to position in `links` Vec. Memory overhead from cloned TraceLinks is negligible at expected scale (hundreds to low thousands).
- **Alternatives considered**: Matrix model (rejected — quadratic memory, sparse), petgraph (rejected — external dependency, YAGNI)
- **Source**: AR Option 1

### D-2: Module Placement — src/model/trace.rs

- **Decision**: New `trace` submodule within existing `model` module
- **Rationale**: TraceLink, SourceLocation are domain model types. Cohesive with PolicyDocument, PolicySection, PolicyRequirement. AR specifies `src/model/trace.rs`.
- **Alternatives considered**: Separate `traceability` crate (rejected — not warranted for a focused data structure)

### D-3: Error Strategy — Dedicated TraceError Enum

- **Decision**: `TraceError` enum with `thiserror` in `src/model/trace.rs`. Single variant: `DuplicateElement { element_id: String }`.
- **Rationale**: Follows constitution principle VIII. Separate from `ForgeError` to keep domain errors focused. `ForgeError` can add a `Trace(TraceError)` variant for pipeline-level propagation if needed.
- **Alternatives considered**: Reuse ForgeError directly (rejected — mixes concerns), anyhow (rejected — library crate, not binary)

### D-4: oscal_json_path Format — Dot-Notation

- **Decision**: Dot-notation format (e.g., `catalog.groups[0].controls[2]`)
- **Rationale**: More human-readable than JSON Pointer (RFC 6901). Matches all existing examples in codebase. Clarification resolved in spec session 2026-02-13.
- **Source**: Spec clarification OQ-1

### D-5: Integration Approach — Pass &mut TraceLinkCollection to Builders

- **Decision**: Modify `build_catalog` and `build_component_definition` to accept an optional `&mut TraceLinkCollection` parameter. Record trace links inside the builder loops where controls/implemented-requirements are created.
- **Rationale**: Minimal coupling — builders already iterate over requirements and create OSCAL elements. Adding a `collection.record()` call at the point of element creation is the natural integration point. Optional parameter preserves backward compatibility.
- **Alternatives considered**: Return trace links as a separate output (rejected — complicates caller), post-processing pass (rejected — loses source context)

### D-6: No New Dependencies

- **Decision**: Zero new crate additions
- **Rationale**: All needed functionality available from std library (HashMap, Vec, PathBuf) and existing dependencies (serde, thiserror, tracing). AR and PRD both confirm this constraint.

## Codebase Integration Points

### Catalog Builder (src/oscal/catalog.rs)

- **Function**: `build_catalog(document: &PolicyDocument) -> Result<OscalCatalog, ForgeError>`
- **Modification**: Add `trace_links: Option<&mut TraceLinkCollection>` parameter
- **Integration point**: Inside the `for (req_idx, req)` loop, after `OscalControl` is constructed, call `trace_links.record(...)` with the requirement's stable_id, the generated control's uuid, the computed JSON path, and a SourceLocation from the requirement + section + document metadata.
- **JSON path format**: `catalog.groups[{group_idx}].controls[{ctrl_idx}]`

### Component Definition Builder (src/oscal/component_definition.rs)

- **Function**: `build_component_definition(document: &PolicyDocument) -> Result<ComponentDefinitionEnvelope, ForgeError>`
- **Current state**: WI-14 structure only. `control_implementations` is an empty Vec placeholder. WI-15 (Implemented Requirements) will populate it.
- **Modification**: Add `trace_links: Option<&mut TraceLinkCollection>` parameter. When WI-15 adds implemented-requirements (or if already present), record trace links.
- **Note**: If WI-15 is not yet merged, this integration is stubbed — the builder currently produces no implemented-requirements to trace.

### Pipeline Orchestrator (src/pipeline.rs)

- **Function**: `run_catalog_pipeline(...)`
- **Modification**: Create `TraceLinkCollection::new()` before builder calls. Pass `Some(&mut trace_links)` to builders. Log trace link count after generation.

### Domain Model Fields Used for SourceLocation

- `PolicyRequirement.source_line` -> `SourceLocation.line_number`
- `PolicySection.title` -> `SourceLocation.section_title`
- `DocumentMetadata.source_path` -> `SourceLocation.file_path`
