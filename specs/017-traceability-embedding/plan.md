# Implementation Plan: Traceability Embedding

**Branch**: `017-traceability-embedding` | **Date**: 2026-02-13 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/017-traceability-embedding/spec.md`
**Depends On**: WI-16 (016-traceability-model), WI-15 (015-component-implemented-requirements)
**AR**: [docs/AR/017-ar-traceability-embedding.md](../../docs/AR/017-ar-traceability-embedding.md)
**SEC**: [docs/SEC/017-sec-traceability-embedding.md](../../docs/SEC/017-sec-traceability-embedding.md)

## Summary

Embed traceability metadata from WI-16's `TraceLinkCollection` into generated OSCAL JSON artifacts as namespaced `prop` and `link` elements. Every generated control (Catalog) and implemented-requirement (Component Definition) receives three props (`source-file`, `source-section`, `source-line`) with the FORGE trace namespace, plus one link with `rel: "source"` and `href: "<file>#line=<n>"`. Groups receive a `source-section` prop. The documentary component receives a `source-file` prop. No trace data appears in `remarks` fields.

**Selected Architecture**: Option 3 (Link + Prop Hybrid) from AR-017.

## Technical Context

**Language/Version**: Rust edition 2024, stable 1.93.0
**Primary Dependencies**: serde 1.0.228, serde_json 1.0.149, thiserror 2.0.18, tracing 0.1.44, uuid 1.20.0, url 2.5 (all existing -- no new dependencies)
**Storage**: N/A -- in-memory processing only
**Testing**: `cargo test` (TDD mandatory per constitution IV)
**Target Platform**: CLI tool (local filesystem, all platforms)
**Project Type**: Single Rust binary crate with library
**Performance Goals**: Negligible overhead -- ~200 bytes per annotated element (3 props + 1 link)
**Constraints**: No new crate dependencies; JSON output only; all trace props must use FORGE namespace
**Scale/Scope**: Handles documents with up to 500+ controls; existing TraceLinkCollection handles all lookup operations

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Crate-First | PASS | Changes are within the existing `forge` crate; trace embedding is tightly coupled with OSCAL generation types. No new crate needed. |
| II. Rust-First | PASS | Pure Rust, no FFI, no unsafe. |
| III. Contract-First | PASS | Types, constants, and function signatures defined in Phase 1 before implementation. |
| IV. TDD | PASS | Tests written before implementation per TDD cycle. |
| V. Complete Implementation | PASS | All M-1 through M-8 and S-1/S-2 requirements covered. |
| VI. Performance-First | PASS | HashMap lookup per element (O(1)); negligible overhead vs artifact size. |
| VII. Security-First | PASS | SEC review completed (Low risk). SEC-1 through SEC-5 requirements incorporated. |
| VIII. Error Handling | PASS | Uses existing `ForgeError` variants; no new error types needed. |
| IX. Observability | PASS | `tracing::debug!` for embedding counts; consistent with existing logging. |
| X. Simplicity | PASS | Minimal changes: 1 new module, 3 struct extensions, 2 builder modifications. No new abstractions. |
| XI. Current Dependencies | PASS | No new dependencies. All existing crates are current. |

## Project Structure

### Documentation (this feature)

```text
specs/017-traceability-embedding/
├── spec.md              # Feature specification (complete)
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── trace_embedding.rs   # Interface contract
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
src/oscal/
├── trace_embedding.rs       # NEW: constants, helpers, embed_trace_in_catalog()
├── parts.rs                 # MODIFIED: extend OscalProp with optional ns field
├── catalog.rs               # MODIFIED: extend OscalGroup with props/links fields
├── component_definition.rs  # MODIFIED: extend DocumentaryComponent with props field
├── implemented_requirements.rs  # MODIFIED: inject trace props/links during impl-req construction
├── back_matter.rs           # UNCHANGED (OscalLink already has href, rel, text)
├── metadata.rs              # UNCHANGED
└── mod.rs                   # MODIFIED: export new trace_embedding module

src/pipeline.rs              # MODIFIED: call embed_trace_in_catalog after catalog build
```

**Structure Decision**: Single crate, feature-organized within `src/oscal/`. The new `trace_embedding.rs` module owns all trace-specific constants, helper functions, and the catalog embedding function. Component Definition trace embedding is done inline during construction (in `implemented_requirements.rs` and `component_definition.rs`) because the source data is readily available at construction time.

## Phase 0: Research

All unknowns resolved during `/speckit.clarify` session and codebase exploration. See [research.md](research.md) for details.

### Key Research Findings

| Decision | Rationale | Alternatives Rejected |
|----------|-----------|----------------------|
| Reuse `OscalLink` from `back_matter.rs` for trace source links | It already has `href: String`, `rel: String`, `text: Option<String>` -- exact fields needed | Creating a separate `TraceSourceLink` type (unnecessary duplication) |
| Post-processing for Catalog, inline for Component Definition | Catalog has TraceLinkCollection available after build; Component Definition has source data at construction time | Pure post-processing for both (requires complex control-id → TraceLink mapping for impl-reqs); Pure inline for both (requires threading trace data through catalog builder, which already records TraceLinks) |
| Extend `OscalProp.ns` as `Option<String>` | OSCAL v1.2.0 specifies `ns` as optional; `skip_serializing_if = "Option::is_none"` preserves backward compat | Adding `ns` as required field (breaks all existing prop construction sites); Separate `NamespacedProp` type (unnecessary type proliferation) |
| Make `collect_requirements_with_section` `pub(crate)` in catalog.rs | Component-def builder needs section titles per requirement for trace props; function already exists but is private | Duplicating the function in implemented_requirements.rs (violates DRY) |
| Manual percent-encoding for link href (EC-6) | Only need to encode `%`, space, `#` in file paths; `url` crate's percent-encoding is transitively available but not re-exported | Full `percent-encoding` crate as direct dependency (overkill for 3 characters); No encoding (violates RFC 3986 per EC-6) |
| Replace `forge:source-line` prefix with namespaced `source-line` | Clarification Q1 explicitly chose clean replacement (no dual/legacy props) | Keeping both (complexity, confusion); Only adding new props (inconsistent naming) |

## Phase 1: Design & Contracts

### Data Model

See [data-model.md](data-model.md) for entity definitions. Key structural changes:

1. **`OscalProp`** (parts.rs:56-63) -- add `ns: Option<String>` field
2. **`OscalGroup`** (catalog.rs:45-54) -- add `props: Vec<OscalProp>` and `links: Vec<OscalLink>` fields
3. **`DocumentaryComponent`** (component_definition.rs:72-90) -- add `props: Vec<OscalProp>` field

### Contracts

See [contracts/trace_embedding.rs](contracts/trace_embedding.rs) for the full interface contract.

**Constants** (from AR-017):

```rust
pub const FORGE_TRACE_NS: &str = "https://forge.policy-forge.github.io/ns/trace";
pub const PROP_SOURCE_FILE: &str = "source-file";
pub const PROP_SOURCE_SECTION: &str = "source-section";
pub const PROP_SOURCE_LINE: &str = "source-line";
pub const LINK_REL_SOURCE: &str = "source";
```

**Helper functions**:

```rust
/// Build 3 namespaced trace props for a source location.
pub fn build_trace_props(
    source_file: &str,
    section_title: &str,
    line_number: usize,
) -> Vec<OscalProp>;

/// Build 1 source link with href "<file>#line=<n>".
pub fn build_trace_link(
    source_file: &str,
    line_number: usize,
) -> OscalLink;

/// Percent-encode special characters in a file path for use in link href.
/// Encodes: '%' -> %25, ' ' -> %20, '#' -> %23 (per RFC 3986 EC-6).
fn encode_href_path(path: &str) -> String;
```

**Embedding functions**:

```rust
/// Walk catalog groups and controls, inject trace props/links from TraceLinkCollection.
/// Uses trace_links.by_oscal_element(control.uuid) for O(1) lookup.
/// Groups receive source-section prop (S-1).
/// Controls receive 3 props + 1 link (M-1, M-2).
pub fn embed_trace_in_catalog(
    catalog: &mut OscalCatalog,
    trace_links: &TraceLinkCollection,
);
```

**Struct modifications**:

```rust
// parts.rs -- OscalProp gains optional ns
pub struct OscalProp {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,
    pub value: String,
}

// catalog.rs -- OscalGroup gains props and links
pub struct OscalGroup {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub props: Vec<OscalProp>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<OscalLink>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub controls: Vec<OscalControl>,
}

// component_definition.rs -- DocumentaryComponent gains props
pub struct DocumentaryComponent {
    pub uuid: String,
    #[serde(rename = "type")]
    pub component_type: String,
    pub title: String,
    pub description: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub props: Vec<OscalProp>,
    #[serde(rename = "control-implementations")]
    pub control_implementations: Vec<serde_json::Value>,
}
```

### Implementation Flow

#### Catalog Trace Embedding (post-processing)

```text
pipeline.rs::run_catalog_pipeline()
  1. prepare_document(input_path) -> doc_with_ids
  2. TraceLinkCollection::new()
  3. build_catalog(&doc_with_ids, Some(&mut trace_links)) -> catalog
     - Records TraceLink per control: oscal_element_id = stable_id
     - Controls start with props: vec![] (old forge:source-line removed)
  4. NEW: embed_trace_in_catalog(&mut catalog, &trace_links)
     - For each group:
       a. Derive source-section from first child control's trace link section_title
       b. Add source-section prop to group.props (S-1)
     - For each control:
       a. trace_links.by_oscal_element(control.uuid) -> TraceLink
       b. build_trace_props(file, section, line) -> 3 props
       c. build_trace_link(file, line) -> 1 link
       d. Append to control.props and control.links
  5. Assemble envelope with annotated catalog groups
  6. Serialize and output
```

#### Component Definition Trace Embedding (inline)

```text
pipeline.rs::run_component_pipeline()
  1. prepare_document(input_path) -> doc_with_ids
  2. build_component_definition(&doc_with_ids, Some(source_profile), None)
     a. Build DocumentaryComponent with NEW props: vec![source-file prop] (M-5)
     b. build_control_implementations(document, source_profile)
        - For each section + requirement:
          i. collect_requirements_with_section(section) for section titles
          ii. map_requirement_to_implemented(req, control_id, index, source_file, section_title)
              - Build JSON with trace props + links injected (M-3, M-4)
  3. Serialize and output
```

### Key File Changes

| File | Change Type | Description |
|------|-------------|-------------|
| `src/oscal/trace_embedding.rs` | NEW | Constants, helpers, `embed_trace_in_catalog()` |
| `src/oscal/parts.rs` | MODIFY | Add `ns: Option<String>` to `OscalProp` |
| `src/oscal/catalog.rs` | MODIFY | Add `props`/`links` to `OscalGroup`; make `collect_requirements_with_section` `pub(crate)`; remove `forge:source-line` from `build_control_props` |
| `src/oscal/component_definition.rs` | MODIFY | Add `props` to `DocumentaryComponent`; populate source-file prop at construction |
| `src/oscal/implemented_requirements.rs` | MODIFY | Accept source info in `map_requirement_to_implemented`; inject trace props/links into JSON; use `collect_requirements_with_section` |
| `src/oscal/mod.rs` | MODIFY | Export `trace_embedding` module |
| `src/pipeline.rs` | MODIFY | Call `embed_trace_in_catalog` in catalog pipeline |

### Testing Strategy (from AR-017)

| Layer | Test | Coverage Target | Req IDs |
|-------|------|-----------------|---------|
| Unit | `build_trace_props()` returns 3 props with correct names, values, namespace | 100% | M-6, SEC-4 |
| Unit | `build_trace_link()` returns link with correct href format and rel | 100% | M-2, M-4, SEC-3 |
| Unit | `encode_href_path()` encodes spaces, unicode, `#` | 100% | EC-6 |
| Unit | `embed_trace_in_catalog()` annotates all controls with 3 props + 1 link | 90% | M-1, M-2 |
| Unit | `embed_trace_in_catalog()` annotates groups with source-section prop | 90% | S-1, S-2 |
| Unit | Component source-file prop on DocumentaryComponent | 100% | M-5 |
| Unit | Impl-req trace props/links in JSON output | 90% | M-3, M-4 |
| Unit | No trace data in any remarks field | 100% | M-7, SEC-1, SEC-2 |
| Unit | All prop name strings use constants (no raw literals) | Code review | SEC-5 |
| Integration | Full catalog generation with trace embedding | Key paths | SC-001, SC-002 |
| Integration | Full component generation with trace embedding | Key paths | SC-003, SC-004, SC-005 |
| Integration | Bidirectional traceability verification | Key paths | M-8, SC-008 |

### Implementation Guardrails (from AR-017)

- **DO NOT** place any trace metadata in `remarks` fields (PRD M-7, parent M-11)
- **DO NOT** use unnamespaced prop names -- all FORGE props must include `ns: FORGE_TRACE_NS` (PRD M-6)
- **DO NOT** use raw string literals for prop names or link rel -- use the defined constants (SEC-5)
- **DO NOT** combine multiple data points in a single prop value -- one prop per datum (OSCAL idiom)
- **MUST** annotate every control in Catalog output with 3 props + 1 link (PRD M-1, M-2)
- **MUST** annotate every implemented-requirement in Component Definition output with 3 props + 1 link (PRD M-3, M-4)
- **MUST** annotate the documentary component itself with source-file prop (PRD M-5)

### Security Requirements (from SEC-017)

| SEC ID | Requirement | Verification |
|--------|-------------|--------------|
| SEC-1 | No policy document content (prose) in trace props; only file paths, section titles, line numbers | Unit test |
| SEC-2 | Trace metadata shall not appear in OSCAL `remarks` fields | Unit test |
| SEC-3 | Source file paths with special characters properly escaped in link href values | Unit test |
| SEC-4 | All FORGE trace props must use the FORGE namespace | Unit test |
| SEC-5 | All prop name strings must use shared constants (no raw literals) | Code review |
