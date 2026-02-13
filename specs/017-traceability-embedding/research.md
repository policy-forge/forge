# Phase 0: Research — Traceability Embedding (WI-17)

**Date**: 2026-02-13
**Status**: Complete — all unknowns resolved

## Research Summary

All technical unknowns were resolved through codebase exploration and cross-referencing with AR-017, SEC-017, and the OSCAL v1.2.0 specification. No external research was required — all decisions are grounded in existing code patterns and NIST guidance.

## Decisions

### D-1: OscalLink Reuse for Trace Source Links

- **Decision**: Reuse `OscalLink` from `back_matter.rs` (lines 73-84) for trace source links.
- **Rationale**: `OscalLink` already provides `href: String`, `rel: String`, `text: Option<String>` — exactly the fields needed for `rel: "source"` trace links. No new type needed.
- **Alternatives rejected**: Creating a separate `TraceSourceLink` type (unnecessary duplication, increases maintenance burden).

### D-2: Post-Processing for Catalog, Inline for Component Definition

- **Decision**: Catalog trace embedding uses post-processing (`embed_trace_in_catalog` after `build_catalog`); Component Definition uses inline injection during construction.
- **Rationale**:
  - Catalog: `TraceLinkCollection` is available after `build_catalog` with matching element IDs (`stable_id`). Post-processing cleanly separates concerns — catalog building vs. trace annotation.
  - Component Definition: Source data (file path, section title, line number) is readily available at construction time in `build_control_implementations` and `build_component_definition`. The `_trace_links` parameter is already accepted but unused (TODO at line 154 of `component_definition.rs`).
- **Alternatives rejected**: Pure post-processing for both (requires complex control-id-to-TraceLink mapping for impl-reqs since their UUIDs differ from stable_ids); Pure inline for both (requires threading trace data through catalog builder, which is already recording TraceLinks).

### D-3: OscalProp Extension with Optional `ns` Field

- **Decision**: Extend `OscalProp` (parts.rs:56-63) with `ns: Option<String>`, using `#[serde(skip_serializing_if = "Option::is_none")]`.
- **Rationale**: OSCAL v1.2.0 specifies `ns` as optional on property elements. `skip_serializing_if` preserves backward compatibility — existing props without `ns` serialize identically. The clarification session (Q1) explicitly chose this approach.
- **Alternatives rejected**: Adding `ns` as required field (breaks all existing prop construction sites — at least 5 locations); Separate `NamespacedProp` type (unnecessary type proliferation, inconsistent with OSCAL's single `property` model).

### D-4: `collect_requirements_with_section` Visibility

- **Decision**: Make `collect_requirements_with_section` in `catalog.rs:277` `pub(crate)` instead of private.
- **Rationale**: `implemented_requirements.rs` needs section titles per requirement for trace props. The function already exists (private) and implements the correct depth-first traversal pairing requirements with their owning sections.
- **Alternatives rejected**: Duplicating the function in `implemented_requirements.rs` (violates DRY); Adding section title to `collect_requirements` return type (changes public API, breaks existing callers).

### D-5: Manual Percent-Encoding for Link href (EC-6)

- **Decision**: Implement a simple `encode_href_path` function that encodes `%` → `%25`, space → `%20`, `#` → `%23` in file path components of link hrefs.
- **Rationale**: Only 3 characters need encoding per RFC 3986 for file-path-style hrefs. The `url` crate's `percent-encoding` is transitively available (via the `url` dependency in Cargo.toml) but is not re-exported and would need explicit feature activation. The encoding is trivial and well-tested.
- **Alternatives rejected**: Full `percent-encoding` crate as direct dependency (overkill for 3 characters, violates Constitution XI — no new dependencies); No encoding (violates RFC 3986 per EC-6, fails SEC-3).

### D-6: Clean Replacement of `forge:source-line`

- **Decision**: Replace existing `forge:source-line` prefix-based prop with namespaced `source-line` prop. No dual/legacy props retained.
- **Rationale**: Clarification Q1 explicitly chose clean replacement. The `build_control_props` function (parts.rs:194) currently creates `forge:source-line` — this will effectively become a no-op (returning `vec![]`) since trace props are now added by `embed_trace_in_catalog` post-processing.
- **Alternatives rejected**: Keeping both old and new props (complexity, confusion, doubles prop count); Only adding new props without removing old (inconsistent naming, violates M-6 namespace requirement).

### D-7: OscalGroup Extension with `props` and `links`

- **Decision**: Add `props: Vec<OscalProp>` and `links: Vec<OscalLink>` to `OscalGroup` (catalog.rs:45-54), both with `#[serde(skip_serializing_if = "Vec::is_empty")]`.
- **Rationale**: Clarification Q3 chose structural consistency with `OscalControl` (which already has both fields). Groups need `props` for S-1 (source-section) and `links` for future extensibility. Empty-skip serialization preserves backward compatibility.
- **Alternatives rejected**: Only adding `props` (inconsistent with OscalControl structure, limits future extensibility).

### D-8: DocumentaryComponent Extension with `props`

- **Decision**: Add `props: Vec<OscalProp>` to `DocumentaryComponent` (component_definition.rs:72-90) with `#[serde(skip_serializing_if = "Vec::is_empty")]`.
- **Rationale**: M-5 requires `source-file` prop on the documentary component. Empty-skip serialization preserves backward compatibility.
- **Alternatives rejected**: Using a separate metadata field (not OSCAL-compliant); Adding source-file to description text (violates M-7, SEC-2 — no trace data in remarks/descriptions).

## Integration Points

### Pipeline Integration (pipeline.rs)

- **Catalog pipeline** (line 115): Insert `embed_trace_in_catalog(&mut catalog, &trace_links)` between `build_catalog` and envelope assembly. The `catalog` variable must become mutable.
- **Component pipeline** (line 184): Pass `input_path` display string to `build_component_definition` for source-file prop. The `_trace_links` parameter can remain `None` since component def trace embedding is inline.

### TraceLinkCollection Usage

- `by_oscal_element(control.uuid)` returns `Option<&TraceLink>` with `source_location: SourceLocation { file_path, section_title, line_number }` — all fields needed for the 3 trace props + 1 trace link.
- Control UUIDs in the catalog match `oscal_element_id` in TraceLinkCollection (both use `stable_id`).

### Existing Type Compatibility

- `OscalLink` (back_matter.rs:73-84): Already has `PartialEq` via derive — no changes needed.
- `back_matter::Prop` (back_matter.rs:90-96): Separate type from `parts::OscalProp` — these are NOT unified. The back_matter `Prop` is used only for resource-level annotations (e.g., `url-status`).
