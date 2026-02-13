# Data Model: Citation and Reference Extraction (WI-8)

## Entities

### Citation (existing — `src/model/mod.rs:128`)

Already defined. No structural changes needed.

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Citation {
    pub id: String,                           // UUID v5 deterministic
    pub text: String,                         // Citation display text
    pub url: Option<String>,                  // URL if present; None for bibliographic/cross-ref
    pub source_requirement_id: Option<String>, // stable_id of source PolicyRequirement
}
```

**Change required**: Add `PartialEq` derive (currently only `Debug, Clone, Serialize`).

### PolicyRequirement (modified — `src/model/mod.rs:102`)

Add `citations` field:

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PolicyRequirement {
    pub stable_id: Option<String>,
    pub text: String,
    pub source_line: usize,
    pub nesting_depth: u8,
    pub atom_index: usize,
    pub parent_text: Option<String>,
    pub citations: Vec<Citation>,  // NEW — populated by WI-8
}
```

**Default**: `citations: vec![]` (empty until citation extraction runs).

## Relationships

```
PolicyDocument 1──* PolicySection 1──* PolicyRequirement 1──* Citation
```

- A `PolicyDocument` has many `PolicySection`s
- A `PolicySection` has many `PolicyRequirement`s
- A `PolicyRequirement` has many `Citation`s (0..N)
- Each `Citation.source_requirement_id` links back to its parent `PolicyRequirement.stable_id`

## Validation Rules

| Field | Rule | Source |
|-------|------|--------|
| Citation.id | Non-empty, deterministic UUID v5 | M-3 |
| Citation.text | Non-empty (citation display text or URL text) | M-3 |
| Citation.url | Valid URL or malformed URL string; None for non-URL citations | M-1, M-5 |
| Citation.source_requirement_id | Must match parent PolicyRequirement.stable_id (if assigned) | M-4 |
| PolicyRequirement.citations | Empty vec before WI-8; populated after extraction | M-6 |

## State Transitions

```
PolicyRequirement
  [Initial: citations = vec![]]
      ↓ WI-8 citation extraction
  [Enriched: citations = vec![Citation, ...], text = cleaned prose]
```

No complex lifecycle — citation extraction is a single-pass enrichment.
