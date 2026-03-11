# Research: Traceability Report (WI-38)

## R-1: WI-17 Trace Metadata Format (Verified)

**Decision**: Use the existing WI-17 trace prop/link constants from `src/oscal/trace_embedding.rs`.

**Findings**: WI-17 embeds trace metadata using:
- **Namespace**: `https://forge.policy-forge.github.io/ns/trace` (`FORGE_TRACE_NS`)
- **Props on controls**: `source-file` (filename only), `source-section` (section title), `source-line` (1-based line number as string)
- **Props on groups**: `source-section` only (derived from first child control's section title)
- **Links on controls**: `rel: "source"`, `href: "<encoded_file>#line=<n>"`
- **Props on parts**: None (WI-17 does not trace parts)
- **Props on CompDef components**: `source-file` only (at component level)
- **CompDef implemented-requirements**: Stored as `Vec<serde_json::Value>` with trace props/links embedded per requirement

**Rationale**: Reusing constants ensures naming consistency between embedding (WI-17) and extraction (WI-38). The extractor matches `ns == FORGE_TRACE_NS && name == PROP_SOURCE_*`.

**Alternatives considered**: Defining new extraction-specific constants (rejected — creates naming drift risk).

---

## R-2: OSCAL Artifact Parsing Strategy

**Decision**: Parse OSCAL artifact files as `serde_json::Value` and detect type from top-level key.

**Findings**: OSCAL artifacts on disk use envelope format:
- Catalog: `{"catalog": {"uuid": "...", "metadata": {...}, "groups": [...]}}`
- Component Definition: `{"component-definition": {"uuid": "...", "metadata": {...}, "components": [...]}}`

The existing typed structs (`OscalCatalog`, `ComponentDefinition`) are *builder outputs* — they produce OSCAL from `PolicyDocument`. They can be used for deserialization, but `serde_json::Value` is more robust for reading arbitrary OSCAL JSON (may include additional fields not in our structs).

**Type detection algorithm**:
1. Parse file as `serde_json::Value`
2. If top-level object has key `"catalog"` → Catalog
3. If top-level object has key `"component-definition"` → ComponentDefinition
4. Otherwise → unsupported artifact type error

**Rationale**: `serde_json::Value` avoids tight coupling to our builder structs and gracefully handles OSCAL JSON from external tools.

**Alternatives considered**: Deserialize into typed structs (`CatalogEnvelope`, `ComponentDefinitionEnvelope`) — works but may fail on OSCAL files with extra fields or different versions.

---

## R-3: Catalog Element Walking

**Decision**: Walk `catalog.groups[]` → yield group + walk `group.controls[]` → yield control. Skip parts.

**Findings from codebase** (`src/oscal/catalog.rs`):
- `OscalGroup`: has `id`, `title`, `props`, `links`, `controls`
- `OscalControl`: has `id`, `uuid` (skip_serializing), `title`, `links`, `params`, `parts`, `props`
- WI-17 adds 3 props + 1 link to controls; 1 prop to groups

**Walk order**: Groups first (in array order), then controls within each group. This matches the embedding order in `embed_trace_in_catalog`.

**Element ID for controls**: Use `control.id` (e.g., `"POL-AC-001"`) as the display ID in the report (human-readable). The `uuid` field is internal and not serialized.

**Element ID for groups**: Use `group.id` (e.g., `"access-control"`).

---

## R-4: Component Definition Element Walking

**Decision**: Walk `component-definition.components[]` → walk `component.control-implementations[]` → walk `implemented-requirements[]` → yield each implemented-requirement.

**Findings from codebase** (`src/oscal/component_definition.rs`, `src/oscal/implemented_requirements.rs`):
- `DocumentaryComponent`: has `uuid`, `component_type`, `title`, `description`, `props`, `control_implementations: Vec<serde_json::Value>`
- `control_implementations` is stored as raw JSON — each entry has `implemented-requirements` array
- Each implemented-requirement has `uuid`, `control-id`, `description`, `props` (with trace metadata), `links`

**Element ID**: Use `control-id` from each implemented-requirement (e.g., `"POL-AC-001"`).

**Element type**: `"implemented-requirement"`.

**Note**: Components themselves are not yielded as trace entries — they represent the policy document, not individual requirements. Only implemented-requirements carry per-element trace metadata.

---

## R-5: Source Staleness Detection

**Decision**: Compare source file `mtime` against OSCAL `metadata.last-modified` timestamp.

**Findings**:
- Catalog metadata: `last_modified: String` (ISO 8601 format, e.g., `"2026-01-15T10:30:00Z"`)
- CompDef metadata: `last_modified: String` (same format)
- `chrono` crate is already in Cargo.toml — use `DateTime::parse_from_rfc3339` for the OSCAL timestamp and `std::fs::metadata().modified()` for the source file mtime
- If OSCAL `metadata.last-modified` is missing or unparseable, skip the staleness check (no warning)
- If source file mtime > OSCAL last-modified → emit warning

**Rationale**: mtime is a reasonable heuristic for local CLI usage. The warning text should note it's a heuristic ("source file may have been modified since conversion").

**Alternatives considered**: SHA-256 hash comparison (rejected — WI-17 does not embed source hash; would require WI-17 change).

---

## R-6: Table Formatting Strategy

**Decision**: Two-pass `format!` approach — first pass calculates max column widths, second pass renders padded rows.

**Findings**:
- Existing FORGE codebase uses `format!` macros for all formatted output (no table crate)
- AR-038 specifies manual column alignment with fixed-width padding
- Column order: OSCAL Element ID, Element Type, Source Section, Source Line
- Separator: `---` dashes matching column width
- Summary section appended after the table

**Format example**:
```
OSCAL Element ID  Element Type    Source Section     Source Line
----------------  --------------  -----------------  -----------
access-control    group           Access Control     —
POL-AC-001        control         Access Control     10
POL-AC-002        control         Access Control     25
POL-DP-001        control         Data Protection    50
POL-DP-002        control         [unmapped]         [unmapped]

Summary: 5 elements, 4 mapped, 1 unmapped (80.0% coverage)
```

**Unmapped elements**: Show `[unmapped]` in Source Section and Source Line columns.

**Groups with partial data**: Show section title, `—` for Source Line.

---

## R-7: Error Handling Strategy

**Decision**: Add trace-specific error variants to `ForgeError`.

**New variants needed**:
- `TraceArtifactNotFound { path }` → exit code 1
- `TraceSourceNotFound { path }` → exit code 1
- `TraceParseError { detail }` → exit code 2 (invalid JSON)
- `TraceUnsupportedType { detail }` → exit code 2 (not catalog or component-definition)

**Rationale**: Follows existing error categorization in `src/error.rs`. Reuse `ForgeError::FileNotFound` for file existence checks (already provides the right message). Add new variants only for trace-specific errors (parse failures, unsupported type).

**Revised decision**: Actually, we can reuse existing variants:
- File not found → `ForgeError::FileNotFound` (already exists)
- Invalid JSON → `ForgeError::Parse` (already exists, generic parse error)
- Unsupported type → new `ForgeError::TraceUnsupportedArtifact { detail }` (exit code 2)

This minimizes new error variants while keeping trace-specific messaging.

---

## R-8: Control Character Stripping (SEC-5)

**Decision**: Strip ASCII control characters 0x00-0x1F (excluding 0x0A newline and 0x09 tab) from source-derived strings before embedding in the report.

**Implementation**: A small utility function `strip_control_chars(s: &str) -> String` that filters bytes. Applied to source section titles and any other source-derived content rendered in the table.

**Rationale**: Prevents terminal escape sequence injection (ANSI codes) when report is displayed via stdout. Minimal implementation cost.
