# Data Model: Structural Extraction — Clauses & Tables

**Feature Branch**: `004-structural-extraction-clauses`
**Date**: 2026-02-11
**Sources**: AR-004 Interface Definitions, SEC-004 Security Requirements

## Entities

### ListType (Enum)

Distinguishes ordered (numbered) from unordered (bullet) list items.
*(AR-004: Component Overview — ListType)*

| Variant     | Description                        |
|-------------|------------------------------------|
| `Ordered`   | Numbered list item (e.g., `1.`)    |
| `Unordered` | Bullet list item (e.g., `-`, `*`)  |

**Derives**: `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Serialize`

**Mapping from pulldown-cmark**: `Tag::List(Some(_))` → `Ordered`, `Tag::List(None)` → `Unordered`

---

### ExtractedListItem (Struct)

Represents a single list item extracted from the document.
*(AR-004: Interface Definitions)*

| Field           | Type       | Description                                                                 |
|-----------------|------------|-----------------------------------------------------------------------------|
| `text`          | `String`   | Full-text content (all text paragraphs concatenated, excluding code blocks and blockquotes). Inline formatting stripped to plain text. *(SEC-3)* |
| `source_line`   | `usize`    | 1-based line number of the list item in the original document. *(M-4, SEC-5)* |
| `nesting_depth` | `u8`       | 0-based nesting depth. 0 = top-level. Saturates at 255 for deeply nested lists. *(M-5, SEC-4)* |
| `list_type`     | `ListType` | Whether the item belongs to an ordered or unordered list. *(M-1, M-2)* |

**Derives**: `Debug`, `Clone`, `PartialEq`, `Serialize`

**Validation rules**:
- `text` MUST NOT be empty (items with no text after normalization are excluded).
- `source_line` MUST be >= 1. *(M-4)*
- `nesting_depth` uses `u8` for bounded overflow safety. *(SEC-4)*
- `list_type` MUST match the enclosing list's type.

---

### ExtractedTable (Struct)

Represents a complete table extracted from the document.
*(AR-004: Interface Definitions; SEC-7 requires GFM table support)*

| Field         | Type               | Description                                                       |
|---------------|--------------------|-------------------------------------------------------------------|
| `headers`     | `Vec<String>`      | Column header texts from the table header row. Inline formatting stripped. *(SEC-3)* |
| `rows`        | `Vec<Vec<String>>` | Data rows, each row a vector of cell strings. Empty cells are empty strings. *(EC-5, SEC-2)* |
| `source_line` | `usize`            | 1-based line number of the table start in the original document. *(M-4)* |

**Derives**: `Debug`, `Clone`, `PartialEq`, `Serialize`

**Validation rules**:
- `headers` MAY be empty (degenerate table).
- `rows` MAY be empty (header-only table per EC-3).
- Each row in `rows` MUST have the same number of cells as `headers`. Empty cells MUST be `""` (empty String), never omitted. *(SEC-2)*
- `source_line` MUST be >= 1.

---

### ExtractedParagraph (Struct)

A standalone paragraph block extracted from Markdown content (not inside list items or tables).
*(AR-004: Interface Definitions; spec S-2)*

| Field         | Type     | Description                                          |
|---------------|----------|------------------------------------------------------|
| `text`        | `String` | Text content of the paragraph. Inline formatting stripped. |
| `source_line` | `usize`  | 1-based line number of the paragraph start. *(M-4)* |

**Derives**: `Debug`, `Clone`, `PartialEq`, `Serialize`

---

### ExtractedContent (Struct)

The collection of all extracted elements from a document.
*(AR-004: Interface Definitions)*

| Field        | Type                        | Description                                           |
|--------------|-----------------------------|-------------------------------------------------------|
| `list_items` | `Vec<ExtractedListItem>`    | All list items extracted, in document order. *(M-1, M-2)* |
| `tables`     | `Vec<ExtractedTable>`       | All tables extracted, in document order. *(M-3)* |
| `paragraphs` | `Vec<ExtractedParagraph>`   | Standalone paragraph text blocks, in document order. *(S-2)* |

**Derives**: `Debug`, `Clone`, `PartialEq`, `Serialize`

**Invariants**:
- All three vectors preserve document order (by `source_line`).
- All three vectors MAY be empty (document with no relevant content). *(SEC-1)*
- No cross-references between elements. Section association is handled by WI-5.

---

## Relationships

```text
ExtractedContent
├── list_items: Vec<ExtractedListItem>
│   └── list_type: ListType
├── tables: Vec<ExtractedTable>
└── paragraphs: Vec<ExtractedParagraph>
```

- `ExtractedContent` **contains** 0..n of each element type.
- `ExtractedListItem` **references** `ListType` (by value, Copy type).
- No reference to `SectionNode` (WI-3). Section association deferred to WI-5. *(AR-004 guardrail)*

---

## State Transitions

N/A — All types are immutable value objects created during a single extraction pass. No lifecycle transitions.

---

## Security Traceability

| Data Element | SEC Requirement | Verification |
|---|---|---|
| `nesting_depth: u8` | SEC-4 — bounded type, saturate on overflow | Unit test with 6+ nesting levels |
| Empty `ExtractedContent` | SEC-1 — handle no lists/tables without error | Unit test: empty document |
| Empty table cells | SEC-2 — produce empty strings, no panics | Unit test: table with empty cells |
| Stripped inline formatting | SEC-3 — clean text without malformed output | Unit test: bold, links, code spans |
| Event-based parsing | SEC-6 — no regex | Code review |
| `Options::ENABLE_TABLES` | SEC-7 — GFM table support | Code review + table extraction tests |
