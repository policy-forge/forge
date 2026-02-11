# Feature Specification: Structural Extraction — Clauses & Tables

**Feature Branch**: `004-structural-extraction-clauses`
**Created**: 2026-02-11
**Status**: Draft
**Input**: PRD docs/PRD/004-prd-structural-extraction-clauses.md (WI-4)
**Depends On**: WI-2 (002-markdown-ingestion), WI-3 (003-structural-extraction-headings)

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Extract Ordered & Unordered List Items (Priority: P1)

A compliance engineer has a policy document with numbered clauses (e.g., "1. All systems must enforce MFA", "2. Access reviews shall be conducted quarterly") that represent individual policy requirements. FORGE must extract each numbered list item as a candidate requirement with its full text and source line number, so the pipeline can convert them into OSCAL controls.

> As a compliance engineer, I want FORGE to extract numbered list items from my policy so that each clause becomes a candidate OSCAL control.

**Why this priority**: Numbered lists are the most common format for policy requirements across compliance frameworks (NIST, ISO 27001, SOC 2). They directly map to OSCAL control statements and are the highest-value extraction target.

**Independent Test**: Parse a Markdown document with numbered lists and verify each item is extracted with its text, list type (ordered), nesting depth, and source line number.

**Acceptance Scenarios**:

1. **Given** a section with 5 numbered list items, **When** extracting clauses, **Then** 5 ordered list items are produced, each with correct full text and source line number. *(AC-1 → M-1, M-4)*
2. **Given** a nested numbered list (1. item → a. sub-item → i. sub-sub-item), **When** extracting, **Then** nested items are captured with nesting depths 0, 1, and 2 respectively. *(AC-5 → M-5)*
3. **Given** a section with 3 bullet list items, **When** extracting clauses, **Then** 3 unordered list items are produced with correct text and source line numbers. *(AC-2 → M-2, M-4)*

---

### User Story 2 — Extract Table Content (Priority: P2)

A policy document contains tables with structured policy data such as roles and responsibilities, control applicability matrices, or requirement summaries. FORGE must extract the table structure (headers, rows, cells) so that tabular policy data is preserved for downstream OSCAL mapping.

> As a compliance engineer, I want FORGE to extract table content from my policy so that tabular policy data is preserved in the OSCAL output.

**Why this priority**: Tables are common in policies for role matrices, control mappings, and requirement summaries. They contain important structured data that must be preserved, though they are secondary to list-based requirements.

**Independent Test**: Parse a Markdown document with a table and verify the table structure (headers, data rows, cell content) is preserved with source line number.

**Acceptance Scenarios**:

1. **Given** a Markdown table with 3 columns and 5 data rows, **When** extracting clauses, **Then** the table structure is preserved with all 3 column headers and all 5 rows of cell values. *(AC-3 → M-3)*
2. **Given** a table with a header row, **When** extracting, **Then** column headers are distinguished from data rows. *(AC-3 → M-3)*
3. **Given** a table at line 42 of the source, **When** extracting, **Then** the table records source_line=42. *(AC-4 → M-4)*

---

### User Story 3 — Associate Clauses with Parent Sections (Priority: P3)

> **Note**: Deferred to WI-5 (domain model assembly) per AR-004 implementation guardrails: "DO NOT try to associate list items with sections in this module." Source line numbers on every extracted element provide the data WI-5 needs for line-range-based section association.

A compliance engineer needs list items and tables to be associated with the section they belong to (from WI-3's heading hierarchy), so that the document's organizational structure is preserved when mapping to OSCAL.

> As a compliance engineer, I want extracted clauses and tables to be linked to their parent section so that I know which policy section each requirement belongs to.

**Why this priority**: Section association is high-value for traceability and OSCAL mapping but builds on the heading hierarchy from WI-3 and is not strictly required for basic extraction to function.

**Independent Test**: Parse a document with headings and list items under those headings, and verify each extracted clause references its parent section.

**Acceptance Scenarios** *(Informational — deferred to WI-5)*:

1. **Given** a heading "## Authentication" followed by 3 numbered list items, **When** extracting with section context *(WI-5)*, **Then** each list item is associated with the "Authentication" section. *(W-1)*
2. **Given** a heading with a table below it, **When** extracting with section context *(WI-5)*, **Then** the table is associated with that heading's section. *(W-1)*

---

### User Story 4 — Capture Paragraph Text as Section Body (Priority: P4)

A policy document has paragraph text within sections that provides context for the requirements but is not a requirement itself. This text should be captured as section body content for reference.

> As a compliance engineer, I want FORGE to capture paragraph text under headings as section body so that contextual descriptions are preserved alongside the extracted requirements.

**Why this priority**: Paragraph text is informational context, not candidate requirements. Capturing it is valuable for completeness but is lower priority than extracting the actual requirement structures (lists, tables).

**Independent Test**: Parse a document with paragraphs between headings and verify paragraph content is captured as section body text, separate from list items and tables.

**Acceptance Scenarios**:

1. **Given** a section with a paragraph followed by a numbered list, **When** extracting, **Then** the paragraph text is captured as section body content and the list items are captured as candidate requirements. *(S-2)*
2. **Given** a section with only paragraph text (no lists or tables), **When** extracting, **Then** paragraph text is captured as section body and no candidate requirements are produced from that section. *(S-2, EC-1)*

---

### Edge Cases

- **EC-1** (M-1, M-2): When a section contains no lists or tables (only paragraphs), no candidate requirements are produced from that section.
- **EC-2** (M-5): When list items are deeply nested (4+ levels), all nesting levels are preserved.
- **EC-3** (M-3): When a table has only a header row and no data rows, the table is extracted with headers and an empty rows collection.
- **EC-4** (M-1, M-2): When a list item contains inline Markdown formatting (bold, links, code spans), the text content is preserved with formatting stripped or normalized to plain text.
- **EC-5** (M-3): When a table cell is empty, an empty string is recorded for that cell.
- **EC-6** (M-1): When a numbered list restarts numbering within the same section, each contiguous list is treated as a separate list block.
- **EC-7** (M-2): When a bullet list uses mixed markers (`-`, `*`, `+`), all items are recognized as unordered list items.
- **EC-8** (M-1, M-2): When a list item contains multiple paragraphs (continuation content), all text paragraphs are concatenated into the item's full text; non-text blocks (code blocks, blockquotes) within the list item are excluded from the extracted text.

## Requirements *(mandatory)*

### Functional Requirements

**Must Have (M) — MVP, launch blockers:**

- **M-1**: The system MUST extract ordered (numbered) list items from Markdown content, producing candidate requirement objects with full text and source line numbers. *(Traces to: Parent PRD M-1)*
- **M-2**: The system MUST extract unordered (bullet) list items from Markdown content with full text and source line numbers. *(Traces to: Parent PRD M-1)*
- **M-3**: The system MUST extract Markdown tables, preserving header row, data rows, and cell content. *(Traces to: Parent PRD M-1)*
- **M-4**: All extracted elements MUST include their source line number (1-based) for downstream traceability. *(Traces to: Parent PRD M-10)*
- **M-5**: The system MUST handle nested list items, preserving the nesting depth (0-based). *(Traces to: Parent PRD M-1)*

**Should Have (S) — High value, not blocking:**

- **S-2**: Paragraph text within sections SHOULD be captured as section body content, separate from candidate requirements.

**Could Have (C) — Nice to have, if time permits:**

- **C-1**: The system COULD distinguish between list items that appear to be requirements (contain normative verbs like "must", "shall", "should") and informational items.

**Won't Have (W) — Explicitly deferred:**

- **W-1**: Section association — *Deferred to WI-5 (domain model assembly) per AR-004 guardrails. Source line numbers on extracted elements enable line-range-based section association in WI-5.*
- **W-2**: Compound statement splitting — *Deferred to WI-6 (requirement atomization).*
- **W-3**: Normative vs advisory classification — *Deferred to WI-33.*

### Key Entities

- **Extracted List Item**: Represents a single list item from the document. Attributes: full text content (all text paragraphs within the list item concatenated, excluding non-text blocks such as code blocks and blockquotes), source line number (1-based), nesting depth (0-based, where 0 is top-level), and list type (ordered or unordered).
- **Extracted Table**: Represents a complete table from the document. Attributes: column headers (list of strings), data rows (each row is a list of cell strings), and source line number (1-based).
- **Extracted Paragraph**: Represents a standalone paragraph block (not inside list items or tables). Attributes: text content (inline formatting stripped), source line number (1-based). *(AR-004; S-2)*
- **Extracted Content**: The collection of all extracted elements from a document, containing zero or more list items, zero or more tables, and zero or more paragraphs.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of ordered list items in a document are correctly extracted with full text, list type, nesting depth, and source line number.
- **SC-002**: 100% of unordered list items in a document are correctly extracted with full text, list type, nesting depth, and source line number.
- **SC-003**: 100% of Markdown tables in a document are extracted with correct header columns, data rows, cell content, and source line number.
- **SC-004**: Source line numbers recorded for all extracted elements match the actual line positions in the original Markdown file with 100% accuracy.
- **SC-005**: Nested list items at any depth (tested up to 6 levels) are extracted with correct nesting depth values.

## Assumptions

- [A-1] Policy requirements are primarily expressed as list items (numbered or bulleted) within sections.
- [A-2] Tables may contain requirement-like content but are treated as structured data, not individual requirements.
- [A-3] The Markdown parser correctly identifies list and table events (pulldown-cmark with GFM table extension).
- [A-4] Ingested content from WI-2 provides accurate content for parsing, with line number information available.
- [A-5] Inline Markdown formatting within list items and table cells should be normalized to plain text for extraction.

## Dependencies

- **Requires**: WI-1 (001-project-scaffolding), WI-2 (002-markdown-ingestion) — ingested content
- **Requires**: WI-3 (003-structural-extraction-headings) — heading hierarchy for section association (S-1)
- **Blocks**: WI-5 (005-domain-model) — extracted clauses and tables feed the domain model
- **External**: None

## Clarifications

### Session 2026-02-11

- Q: When a list item contains multiple paragraphs (continuation content), what should "full text" include? → A: All text paragraphs concatenated, excluding non-text blocks (code blocks, blockquotes).
