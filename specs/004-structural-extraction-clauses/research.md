# Research: Structural Extraction — Clauses & Tables

**Feature Branch**: `004-structural-extraction-clauses`
**Date**: 2026-02-11
**Sources**: AR-004, SEC-004, spec.md, constitution.md

## Research Summary

No NEEDS CLARIFICATION items in Technical Context. All technical decisions are constrained by the Architecture Review (AR-004 Option 1: Event-Based Pattern Matching with Depth Counter) and the Security Review (SEC-004).

---

## R-1: pulldown-cmark Event API for Lists and Tables

**Decision**: Use `Options::ENABLE_TABLES` with `Parser::new_ext(content, options)` to enable GFM table parsing. *(AR-004 selected option; SEC-7)*

**Rationale**: The existing heading parser uses `Parser::new(content)` (default options). For clause/table extraction, we need the `ENABLE_TABLES` option to receive `Table`, `TableHead`, `TableRow`, and `TableCell` events. The AR explicitly chose event-based parsing over regex (Option 3 rejected) to avoid ReDoS risk (SEC-6).

**Alternatives considered**:
- `Options::ENABLE_GFM` — rejected as overly broad; only table support is needed.
- Regex-based scanning (AR Option 3) — rejected; fails on code blocks, cannot handle lazy continuation, reinvents what pulldown-cmark does correctly. *(SEC-6 requires event-based parsing)*
- Section-aware extraction (AR Option 2) — rejected; breaks parallelism with WI-3, introduces coupling.

**API confirmed (pulldown-cmark 0.13)**:
- `Tag::List(Option<u64>)` — `Some(n)` for ordered (start number), `None` for unordered
- `Tag::Item` — list item boundary
- `Tag::Table(Vec<Alignment>)` — table start with column alignments
- `Tag::TableHead` — header row (contains `TableCell`s directly)
- `Tag::TableRow` — data row (contains `TableCell`s)
- `Tag::TableCell` — individual cell

---

## R-2: Line Number Mapping Utilities

**Decision**: Reuse `build_line_starts()` and `offset_to_line()` from the existing heading extraction module by promoting them to `pub(crate)` visibility.

**Rationale**: These functions are already tested (T007 test suite) and provide O(n) build + O(log n) lookup. Currently private (`fn`) in `parse/mod.rs`; need `pub(crate)` for the new `clauses` submodule to access them.

**Alternatives considered**:
- Copy into `clauses.rs` — rejected; violates DRY.
- Move to `parse::util` submodule — rejected as over-engineering for two small functions.

---

## R-3: Nesting Depth Tracking

**Decision**: Use a depth counter (not indentation parsing) per AR-004. Use `u8` type per SEC-4 with saturation on overflow (no panic). Track list type at each level with a `Vec<ListType>` stack.

**Rationale**: AR anti-pattern: "Don't track nesting by counting indentation whitespace." pulldown-cmark's `Start(List)`/`End(List)` event pairs are the authoritative nesting signals. SEC-4 mandates a bounded type to prevent pathological input from causing issues.

**Implementation**: `depth` = `list_type_stack.len()`. On `Start(Item)`, `nesting_depth` = `depth.saturating_sub(1).min(u8::MAX as usize) as u8`. This naturally saturates at 255 and safely handles empty stacks.

---

## R-4: Multi-Paragraph List Item Text Accumulation

**Decision**: Accumulate text from all `Paragraph` blocks within a list `Item`, separated by spaces. Exclude text from `CodeBlock` and `BlockQuote` blocks within the item. *(Spec clarification session 2026-02-11; EC-8)*

**Rationale**: Policy requirements are prose text. Code blocks and blockquotes within list items are rare in policy documents and would add noise.

**Implementation**: Track `exclude_depth` counter; increment on `Start(CodeBlock|BlockQuote)`, decrement on end. Only accumulate text when `exclude_depth == 0`.

---

## R-5: Inline Formatting Normalization

**Decision**: Rely on pulldown-cmark's event model for natural formatting stripping. *(AR-004 anti-pattern: "Don't preserve inline Markdown formatting in extracted text"; SEC-3)*

**Rationale**: pulldown-cmark separates content from formatting in its event stream:
- `**bold**` → `Text("bold")`
- `[link text](url)` → `Text("link text")` (URL is in `Start(Link)` tag)
- `` `code` `` → `Code("code")`
- `*italic*` → `Text("italic")`

No additional normalization logic needed.

---

## R-6: Parser Pass Strategy

**Decision**: Use a separate parser pass for clause extraction, independent of heading extraction (WI-3). Section association is deferred to WI-5 (domain model assembly). *(AR-004: "WI-3 and WI-4 are parallel"; AR guardrail: "DO NOT try to associate list items with sections in this module")*

**Rationale**: The AR decision driver #4 (Parallelism) requires this module to work independently of WI-3. Two passes over in-memory content is negligible cost. Section association (S-1) is handled downstream by WI-5 using source line ranges.

---

## R-7: ExtractedParagraph Type

**Decision**: Include `ExtractedParagraph` in the data model per AR-004 interface definition. This supports S-2 (paragraph text as section body content). *(AR-004 Component Overview; spec S-2)*

**Rationale**: The AR defines `ExtractedContent` with three collections: `list_items`, `tables`, and `paragraphs`. Paragraph text is captured separately from list items and tables, giving downstream consumers (WI-5) clean separation of content types.

**Constraint**: Only capture paragraphs that are NOT inside list items or table cells. These are standalone paragraphs between structural elements.

---

## R-8: Section Association Strategy

**Decision**: Do NOT associate extracted items with sections in this module. Defer to WI-5. *(AR-004 guardrail: explicit "DO NOT"; AR anti-pattern: "Don't try to associate list items with sections")*

**Rationale**: The AR's Decision Scope explicitly states "How extracted content is associated with sections — decided by the integration in WI-5." Coupling clause extraction to heading extraction would break the parallelism constraint. Source line numbers on every element provide the data WI-5 needs for line-range-based association.
