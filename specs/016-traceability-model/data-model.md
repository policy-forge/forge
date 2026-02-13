# Data Model: Traceability Model (WI-16)

**Phase**: 1 — Design & Contracts
**Date**: 2026-02-13

## Entities

### SourceLocation

Source location of a policy requirement in the original document.

| Field | Type | Constraints | Source |
|-------|------|-------------|--------|
| `file_path` | `PathBuf` | Required. Path to the source policy file. | `DocumentMetadata.source_path` |
| `section_title` | `String` | Required. Empty string if no parent section (EC-4). | `PolicySection.title` |
| `line_number` | `usize` | Required. 1-based line number. | `PolicyRequirement.source_line` |

**Derives**: `Debug`, `Clone`, `PartialEq`, `Eq`, `Serialize`, `Deserialize`

### TraceLink

A single mapping from a policy requirement to an OSCAL element.

| Field | Type | Constraints | Source |
|-------|------|-------------|--------|
| `requirement_stable_id` | `String` | Required. UUID from WI-7 (or placeholder). | `PolicyRequirement.stable_id.unwrap()` |
| `oscal_json_path` | `String` | Required. Dot-notation logical path (e.g., `catalog.groups[0].controls[2]`). | Computed at generation time |
| `oscal_element_id` | `String` | Required. Unique across entire collection (one-to-one reverse constraint). | Generated control/impl-req UUID |
| `source_location` | `SourceLocation` | Required. | Composed from domain model fields |

**Derives**: `Debug`, `Clone`, `Serialize`, `Deserialize`

### TraceLinkCollection

Append-only aggregation container with dual-index bidirectional lookup.

| Field | Type | Visibility | Notes |
|-------|------|------------|-------|
| `links` | `Vec<TraceLink>` | `private` | Canonical store. Insertion-order preserved. |
| `by_requirement` | `HashMap<String, Vec<TraceLink>>` | `private` | Forward index: requirement_stable_id -> grouped Vec of TraceLinks (enables `&[TraceLink]` return from lookup) |
| `by_oscal_element` | `HashMap<String, usize>` | `private` | Reverse index: oscal_element_id -> index into `links` |

**Derives**: `Debug`, `Default`

**Public API**:

| Method | Signature | Returns | Notes |
|--------|-----------|---------|-------|
| `new()` | `fn new() -> Self` | `TraceLinkCollection` | Empty collection via `Default` |
| `record()` | `fn record(&mut self, link: TraceLink) -> Result<(), TraceError>` | `Ok(())` or `Err(DuplicateElement)` | Rejects duplicate `oscal_element_id`. Clones link into `by_requirement` grouped Vec, appends original to `links` Vec, updates `by_oscal_element` reverse index. |
| `by_requirement()` | `fn by_requirement(&self, stable_id: &str) -> &[TraceLink]` | Slice of matching links | Returns `&[]` from grouped `by_requirement` HashMap if not found (EC-2). Contiguous slice enabled by grouped `Vec<TraceLink>` storage. |
| `by_oscal_element()` | `fn by_oscal_element(&self, element_id: &str) -> Option<&TraceLink>` | `Some(&TraceLink)` or `None` | `None` if not found (EC-3) |
| `iter()` | `fn iter(&self) -> impl Iterator<Item = &TraceLink>` | Iterator | Insertion-order iteration (S-2) |
| `len()` | `fn len(&self) -> usize` | Count | Number of trace links (S-3) |
| `is_empty()` | `fn is_empty(&self) -> bool` | Bool | True if no trace links (S-3) |

### TraceError

Error types for traceability operations.

| Variant | Fields | Message Template |
|---------|--------|------------------|
| `DuplicateElement` | `element_id: String` | `"Duplicate OSCAL element ID: {element_id} already recorded"` |

**Derives**: `Debug`, `thiserror::Error`

## Relationships

```
TraceLinkCollection 1 ──contains──o..* TraceLink
TraceLink           1 ──has──────── 1 SourceLocation

PolicyRequirement   1 ──maps-to──o..* TraceLink   (forward: one-to-many)
OscalElement        1 ──maps-to──── 1 TraceLink   (reverse: one-to-one)
```

## Validation Rules

- `oscal_element_id` must be unique across the entire `TraceLinkCollection` (enforced by `record()`)
- `line_number` must be >= 1 (0 is not a valid 1-based line number, but no runtime check -- trusted input from upstream pipeline)
- `section_title` may be empty string (edge case EC-4: requirement outside any heading)
- TraceLink instances are immutable after creation
- TraceLinkCollection is append-only during generation, read-only afterward

## State Lifecycle

TraceLink has no state transitions -- it is a data record created once and never modified.

TraceLinkCollection lifecycle:
1. **Created**: `TraceLinkCollection::new()` -- empty
2. **Populating**: `record()` calls during generation (append-only writes)
3. **Sealed**: After generation completes, collection is read-only (lookups, iteration)

No explicit state field -- lifecycle enforced by ownership semantics (mutable borrow during generation, shared borrow afterward).
