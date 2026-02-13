# Data Model: Component Implemented Requirements (WI-15)

## Entities

### ControlImplementation (new — OSCAL output structure)

Groups implemented-requirements under a single source baseline reference.

| Field | Type | Source | Validation | Notes |
|-------|------|--------|------------|-------|
| uuid | String | UUID v5(CONTROL_IMPL_NAMESPACE, `"{source_profile}\0{policy_title}"`) | Non-empty, valid UUID | Deterministic per baseline+document pair |
| source | String | `--source-profile` CLI flag | Non-empty (SEC-3, SEC-4) | Stored as-is, never fetched (href reference) |
| description | String | Generated: `"Implementation narratives derived from {policy_title}."` | Non-empty | References policy document title (FR-004, S-2) |
| implemented-requirements | Vec | Mapped from PolicyRequirements | May be empty (FR-013) | One entry per PolicyRequirement |

### ImplementedRequirement (new — OSCAL output structure)

Maps a single PolicyRequirement to a control-id with implementation narrative.

| Field | Type | Source | Validation | Notes |
|-------|------|--------|------------|-------|
| uuid | String | UUID v5(IMPL_REQ_NAMESPACE, `"{stable_id}\0{text}\0{index}"`) | Non-empty, valid UUID | Deterministic; index ensures uniqueness for EC-5 |
| control-id | String | Derived from section context via `generate_control_id` | Non-empty | Matches Catalog control.id (e.g., `POL-AC-001`) |
| description | String | `requirement.text` (raw, no transformation) | Non-empty or placeholder (FR-014) | FR-008: raw text at P1 scope |

### PolicyRequirement (existing — domain model, read-only)

| Field | Type | Used By WI-15 | Notes |
|-------|------|---------------|-------|
| stable_id | Option<String> | UUID seed + fallback detection | UUID string from WI-7; None triggers EC-2 fallback |
| text | String | description field + UUID seed | Raw requirement prose |
| source_line | usize | Not directly used | Available for future traceability (WI-16/17) |
| nesting_depth | u8 | Not used | Structural metadata |
| atom_index | usize | UUID seed (index component) | Ensures uniqueness for split requirements |
| parent_text | Option<String> | Not used | Original text before atomization |
| citations | Vec<Citation> | Not used (WI-12 handles) | Extracted citation references |

### PolicySection (existing — domain model, read-only)

| Field | Type | Used By WI-15 | Notes |
|-------|------|---------------|-------|
| title | String | Section abbreviation for control-id | Used by generate_section_abbreviation |
| requirements | Vec<PolicyRequirement> | Iterated for mapping | Direct children |
| children | Vec<PolicySection> | Recursed for deep requirements | Subsections |

## Relationships

```
PolicyDocument 1──* PolicySection (sections tree)
PolicySection  1──* PolicyRequirement (requirements)
PolicySection  1──* PolicySection (children — recursive)

DocumentaryComponent 1──* ControlImplementation (control_implementations)
ControlImplementation 1──* ImplementedRequirement (implemented-requirements)

PolicyRequirement 1──1 ImplementedRequirement (maps to — via build_control_implementations)
```

## Key Invariants

1. **Completeness** (FR-011): `len(implemented-requirements) == len(all PolicyRequirements in document)`
2. **Determinism** (FR-002, FR-006): Same inputs produce same UUIDs across all runs
3. **Uniqueness** (FR-012, EC-5): Two requirements with identical text but different positions receive distinct UUIDs (index in seed prevents collision)
4. **Control-ID Consistency**: Control-ids match what `build_catalog` would generate for the same document
5. **No Data Loss**: Every PolicyRequirement produces exactly one ImplementedRequirement; empty text produces placeholder (FR-014)

## Edge Case Behaviors

| Condition | Behavior | Requirement |
|-----------|----------|-------------|
| Zero PolicyRequirements | Empty `implemented-requirements` array + `tracing::warn!` | FR-013, EC-1 |
| Missing stable_id | Fallback control-id: `REQ-{zero-padded global index}` | EC-2 |
| Empty requirement text | Placeholder description: `"No implementation narrative available."` | FR-014, EC-3 |
| Empty `--source-profile` | Error before building | SEC-4, EC-4 |
| Identical text, different positions | Distinct UUIDs (index in seed) | FR-012, EC-5 |
