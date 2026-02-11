# Feature Specification: Structural Extraction — Headings

**Feature Branch**: `003-structural-extraction-headings`
**Created**: 2026-02-11
**Status**: Draft
**Input**: PRD docs/PRD/003-prd-structural-extraction-headings.md (WI-3)
**Depends On**: WI-2 (002-markdown-ingestion)

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Extract Section Hierarchy (Priority: P1)

A compliance engineer has a policy document with Markdown headings organizing sections (e.g., H1 "Access Control Policy", H2 "Authentication Requirements", H3 "Password Rules"). FORGE must parse these headings into a hierarchical tree so that the document structure is preserved for downstream OSCAL Catalog group generation.

> As a compliance engineer, I want FORGE to recognize the heading hierarchy in my Markdown policy so that sections map correctly to OSCAL Catalog groups.

**Why this priority**: Section hierarchy is the primary organizational structure that drives OSCAL group generation. Without it, all requirements would appear in a flat, unstructured list — making the OSCAL output unusable for compliance workflows.

**Independent Test**: Parse a Markdown document with nested headings (H1 through H3+) and verify the resulting tree has correct parent-child relationships, heading titles, levels, and source line numbers.

**Acceptance Scenarios**:

1. **Given** a Markdown document with H1 "Policy", H2 "Access Control", H3 "Passwords", **When** extracting structure, **Then** a tree is produced where "Access Control" is a child of "Policy" and "Passwords" is a child of "Access Control". *(AC-1, AC-3 → M-1, M-3)*
2. **Given** a heading "## Access Control" at line 10 of the source document, **When** extracting sections, **Then** the section node records title="Access Control", level=2, source_line=10. *(AC-2 → M-2)*
3. **Given** a Markdown document with 5 headings at various levels, **When** extracting sections, **Then** 5 section nodes are produced in the correct hierarchy. *(AC-1 → M-1)*

---

### User Story 2 — Handle Irregular Heading Levels (Priority: P2)

A policy document uses inconsistent heading levels — for example, jumping from H1 directly to H3 without an H2, or containing multiple H1 headings. FORGE must handle these gracefully without crashing or losing sections.

> As a compliance engineer, I want FORGE to handle imperfect heading structures so that real-world policy documents (which may not follow strict heading nesting) are still parsed correctly.

**Why this priority**: Real-world documents frequently have inconsistent heading levels. The parser must be robust, not fragile. If irregular documents cause failures, the tool is unusable for most real compliance documents.

**Independent Test**: Parse a document with skipped heading levels and multiple H1 headings, and verify a reasonable tree is still produced with no lost sections.

**Acceptance Scenarios**:

1. **Given** a document with H1 followed directly by H3 (skipping H2), **When** extracting structure, **Then** H3 is placed as a child of H1 (no crash or missing sections). *(AC-4 → M-4)*
2. **Given** a document with multiple H1 headings, **When** extracting sections, **Then** each H1 starts a new top-level section. *(EC-4 → M-1)*

---

### User Story 3 — Capture Section Body Content (Priority: P3)

A compliance engineer needs not only the heading hierarchy but also the text content between headings associated with each section, so downstream processing can identify requirements within each section.

> As a compliance engineer, I want FORGE to capture the body text under each heading so that I can see which content belongs to which section.

**Why this priority**: While the hierarchy is the primary structure, associating body content with sections enables downstream clause extraction (WI-4) and requirement identification (WI-6) to work within section boundaries.

**Independent Test**: Parse a document where headings have body text between them and verify each section node contains the correct body content.

**Acceptance Scenarios**:

1. **Given** a document with heading "## Scope" followed by two paragraphs of text before the next heading, **When** extracting sections, **Then** the "Scope" section node contains those two paragraphs as its body text. *(S-1)*
2. **Given** a heading with no content before the next heading, **When** extracting sections, **Then** the section node has no body text (empty or absent). *(S-1)*

---

### User Story 4 — Verify Section Tree via Debug Output (Priority: P4)

A developer or compliance engineer wants to inspect the extracted section tree to verify correctness during development and troubleshooting.

> As a developer, I want to view the extracted section tree as debug output so that I can verify the heading extraction is correct.

**Why this priority**: Debug output is essential for development-time verification but does not affect end-user functionality.

**Independent Test**: Run section extraction on a sample document and verify a human-readable tree representation is produced.

**Acceptance Scenarios**:

1. **Given** a document with a heading hierarchy, **When** extracting sections with debug output enabled, **Then** a human-readable tree representation showing nesting, titles, levels, and line numbers is produced. *(S-2)*

---

### Edge Cases

- **EC-1** (M-1): When a document has no headings, an empty section list is returned.
- **EC-2** (M-4): When a document starts with H3 (no preceding H1 or H2), H3 becomes a top-level section.
- **EC-3** (M-1): When a heading has no text (empty `##`), the section is created with an empty title.
- **EC-4** (M-3): When multiple H1 headings exist, each is a separate top-level section.
- **EC-5** (S-1): When body text exists before the first heading, it is either discarded or associated with a synthetic root (assumption: discarded — see Assumptions).
- **EC-6** (M-4): When a document has only one heading, a single-node tree is returned.

## Requirements *(mandatory)*

### Functional Requirements

**Must Have (M) — MVP, launch blockers:**

- **M-1**: The system MUST extract all Markdown headings (H1–H6) from ingested content and produce a hierarchical section tree. *(Traces to: Parent PRD M-1)*
- **M-2**: Each extracted section MUST include the heading title text, heading level (1–6), and source line number (1-based). *(Traces to: Parent PRD M-1, M-10)*
- **M-3**: The section tree MUST represent parent-child relationships where a heading at level N is a child of the nearest preceding heading at level N-1 or lower. *(Traces to: Parent PRD M-1)*
- **M-4**: The system MUST handle documents with irregular heading nesting (skipped levels) without failure or lost sections. *(Traces to: Parent PRD M-1)*

**Should Have (S) — High value, not blocking:**

- **S-1**: The system SHOULD capture the text content between headings (section body) and associate it with the appropriate section.
- **S-2**: The section tree SHOULD be representable as debug output for verification during development.

**Could Have (C) — Nice to have, if time permits:**

- **C-1**: The system COULD detect and warn about heading-level inconsistencies (e.g., H1 to H3 skip) in diagnostic output.

**Won't Have (W) — Explicitly deferred:**

- **W-1**: Extraction of lists, tables, and clauses — *Deferred to WI-4 (004-structural-extraction-clauses).*
- **W-2**: Mapping sections to OSCAL groups — *Deferred to WI-9.*

### Key Entities

- **Section Node**: Represents a single heading-delimited section in the document. Attributes: heading title (text), heading level (1–6), source line number (1-based), body text (optional), and an ordered list of child section nodes.
- **Section Tree**: The complete hierarchical representation of a document's structure. Contains zero or more top-level section nodes, each of which may have nested children to arbitrary depth (up to 6 levels).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of Markdown headings (H1–H6) in a document are correctly identified and extracted into section nodes.
- **SC-002**: 100% of parent-child relationships in the section tree match the heading-level nesting of the source document.
- **SC-003**: Source line numbers recorded in each section node match the actual line positions in the original Markdown file with 100% accuracy.
- **SC-004**: Documents with irregular heading nesting (skipped levels, multiple H1s, headings starting above H1) are parsed without errors, and no sections are lost.
- **SC-005**: All 25 example policy documents in the `example_data/` directory are parsed successfully, producing valid section trees.

## Assumptions

- [A-1] Policy documents use Markdown headings (`#` through `######`) as the primary organizational structure.
- [A-2] The ingested content from WI-2 provides accurate line-numbered content for cross-referencing source positions.
- [A-3] Non-heading content between sections belongs to the section defined by the immediately preceding heading.
- [A-4] Text that appears before the first heading in a document is discarded (not associated with any section node), since it has no heading context.
- [A-5] Headings inside fenced code blocks are not treated as structural headings (the Markdown parser handles this distinction).

## Dependencies

- **Requires**: WI-1 (001-project-scaffolding), WI-2 (002-markdown-ingestion) — ingested content with line numbers
- **Blocks**: WI-5 (005-domain-model) — section tree feeds into the PolicySection domain model
- **Parallel**: WI-4 (004-structural-extraction-clauses) — can be developed concurrently
