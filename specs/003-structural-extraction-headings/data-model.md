# Data Model: Structural Extraction — Headings

**Feature Branch**: `003-structural-extraction-headings`
**Date**: 2026-02-11

## Entities

### SectionNode

Represents a single heading-delimited section in a Markdown document.

| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| `title` | `String` | Heading title text (e.g., "Access Control" from `## Access Control`). May be empty for headings with no text (EC-3). | None — empty string is valid |
| `heading_level` | `u8` | Heading level: 1 for H1, 2 for H2, ..., 6 for H6 | Must be 1-6 (enforced by `HeadingLevel` enum at parse time) |
| `source_line` | `usize` | 1-based line number in the original Markdown document | Must be >= 1 |
| `body_text` | `Option<String>` | Text content between this heading and the next heading. `None` if no body content exists. | None — any string content is valid |
| `children` | `Vec<SectionNode>` | Child sections (headings at deeper levels nested under this one). Empty if no children. | Recursive; depth bounded to 6 levels (H1 through H6), enforced by `HeadingLevel` enum |

**Derives**: `Debug`, `Clone`, `PartialEq`

### Relationships

```text
SectionNode (root list)
├── SectionNode (child)
│   ├── SectionNode (grandchild)
│   │   └── ...
│   └── SectionNode (grandchild)
└── SectionNode (child)
    └── ...
```

- A `Vec<SectionNode>` represents the forest of top-level sections (multiple H1s produce multiple roots).
- Each `SectionNode` contains `Vec<SectionNode>` children, forming a tree.
- Maximum depth: 6 levels (H1 through H6), enforced by the `HeadingLevel` enum.
- A document with no headings produces an empty `Vec<SectionNode>`.

### State Transitions

N/A — `SectionNode` is an immutable data structure produced once by `extract_sections`. No state transitions.

## Entity Traceability

| Entity | PRD Requirement | AR Component | SEC Requirement |
|--------|-----------------|--------------|-----------------|
| SectionNode.title | M-2 (heading title text) | SectionNode struct | — |
| SectionNode.heading_level | M-2 (heading level 1-6) | SectionNode struct | SEC-1 (all levels without panic) |
| SectionNode.source_line | M-2 (source line number) | Offset-to-Line Converter | — |
| SectionNode.body_text | S-1 (section body content) | Stack-Based Builder | — |
| SectionNode.children | M-3 (parent-child relationships) | Stack-Based Builder | SEC-4 (bounded depth) |
| Vec\<SectionNode\> (root) | M-1 (extract all headings) | extract_sections function | SEC-2 (empty Vec for no headings) |

## Upstream/Downstream

**Upstream** (consumed from WI-2):
- `IngestedDocument.lines` — not directly used by `extract_sections`; the function takes raw `&str` content
- The raw Markdown content string is the input

**Downstream** (consumed by future WIs):
- WI-5 (Domain Model): `Vec<SectionNode>` will be converted to `PolicySection` structs
- WI-4 (Clause Extraction): May operate on `SectionNode.body_text` content
- WI-9 (OSCAL Groups): Section tree maps to OSCAL Catalog group hierarchy
