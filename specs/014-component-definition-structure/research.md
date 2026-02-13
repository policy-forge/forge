# Research: OSCAL Component Definition Structure

**Phase 0 output** | **Date**: 2026-02-12

## R-1: Builder Pattern -- Typed Structs vs serde_json::Value

**Decision**: Use typed Rust structs with `#[derive(Serialize)]` (same pattern as Catalog builder).

**Rationale**: The AR and PRD both state "use serde_json::Value for consistency with the Catalog builder." However, examination of the actual codebase reveals the **Catalog builder uses typed structs**, not `serde_json::Value`:

- `CatalogEnvelope` -- `#[derive(Serialize)]` struct with `catalog` field
- `OscalCatalog` -- typed struct with `uuid`, `metadata`, `groups`, `back_matter`
- `OscalGroup` -- typed struct with `id`, `title`, `controls`
- `OscalControl` -- typed struct with `id`, `uuid`, `title`, `links`, `parts`, `props`

The `serde_json::json!` macro is NOT used anywhere in the Catalog builder. All JSON is produced via `serde_json::to_string_pretty(&envelope)` on typed structs.

Therefore, **"mirroring the Catalog builder pattern"** means using typed structs, not `serde_json::Value`. This provides:
- Compile-time enforcement of required fields
- Self-documenting code (struct fields describe the OSCAL model)
- Consistency with the actual codebase (not the AR's mistaken premise)
- IDE support (autocomplete, type checking)

**Alternatives considered**:
- `serde_json::Value` builder (AR recommendation) -- rejected because it contradicts the actual Catalog pattern
- Shared `OscalArtifactBuilder` trait -- rejected per constitution principle X (only 2 builders; premature abstraction)

## R-2: Metadata Assembly Reuse Pattern

**Decision**: Call `assemble_metadata` from `src/oscal/metadata.rs` and map the returned `OscalMetadata` fields into a `ComponentDefinitionMetadata` struct.

**Rationale**: The pipeline (`pipeline.rs:96-121`) shows the established pattern:
1. Call `assemble_metadata(&doc.metadata, None)` to get the proper `OscalMetadata` struct
2. Map the returned fields into the artifact's metadata

The final design (see `contracts/component_definition.rs`) uses a dedicated `ComponentDefinitionMetadata` struct with mapped fields from `OscalMetadata`, rather than embedding via `#[serde(flatten)]`. This gives explicit control over field serialization and keeps the Component Definition metadata contract self-contained.

**Alternatives considered**:
- Duplicate metadata assembly logic -- rejected per FR-007 (M-7)
- Embed `OscalMetadata` via `#[serde(flatten)]` -- considered initially but rejected in favor of explicit field mapping for clearer serialization control

## R-3: UUID v5 Namespace for Components

**Decision**: Define a new `COMPONENT_NAMESPACE` constant in `src/uuid.rs`, derived from `Uuid::new_v5(&FORGE_NAMESPACE_UUID, b"component")`.

**Rationale**: The existing pattern uses `BACK_MATTER_NAMESPACE` derived from `Uuid::new_v5(&FORGE_NAMESPACE_UUID, b"back-matter")`. The component namespace follows the same derivation pattern but with `"component"` as the label. This ensures component UUIDs never collide with back matter UUIDs or requirement stable IDs.

**Hash input**: `format!("{title}\0{version}\0{document_id}")` — null byte separators prevent collisions between ambiguous title/version boundaries, and the document ID (derived from filename) ensures uniqueness across different source files sharing the same title/version.

**Alternatives considered**:
- Reuse `FORGE_NAMESPACE_UUID` directly -- rejected (collision risk with requirement UUIDs)
- Random UUID v4 -- rejected per FR-004 (determinism requirement)

## R-4: Error Handling

**Decision**: Add `ComponentDefinitionBuild(String)` variant to `ForgeError` in `src/error.rs`.

**Rationale**: Mirrors the existing `CatalogBuild(String)` and `BackMatter(String)` patterns. The builder currently has limited failure modes (metadata assembly is infallible, back matter may fail). Returning `Result` maintains API consistency and allows future extensibility.

**Alternatives considered**:
- Reuse `CatalogBuild` -- rejected (misleading; separate artifact type)
- Make builder infallible -- rejected (back matter can fail; consistency)

## R-5: Component Description Format

**Decision**: Always use template: `"Documentary component representing the {title} policy document."` where `{title}` is the resolved title (defaults to `"Untitled Policy Document"` if empty).

**Rationale**: Confirmed in clarification Q3. Consistent across all Component Definitions.

## R-6: Serialization Key Names

**Decision**: Use `#[serde(rename = "...")]` for OSCAL-compliant JSON keys.

**Rationale**: OSCAL uses hyphenated keys (`component-definition`, `control-implementations`, `back-matter`, `last-modified`, `oscal-version`). Rust struct fields use snake_case. The same approach is used throughout the existing codebase (see `BackMatter` struct with `#[serde(rename_all = "kebab-case")]` and `OscalMetadata` with field-level renames).

Required renames for new structs:
- `ComponentDefinitionEnvelope.component_definition` -> `"component-definition"`
- `DocumentaryComponent.control_implementations` -> `"control-implementations"`
