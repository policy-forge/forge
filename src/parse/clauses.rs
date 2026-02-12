//! Markdown structural extraction — clauses, tables, and paragraphs.
//!
//! Extracts list items, tables, and standalone paragraphs from Markdown
//! content using pulldown-cmark's event-based parser with GFM table support.
//!
//! # Security Requirements (SEC-004)
//!
//! - SEC-1: Handle documents with no lists/tables without error
//! - SEC-2: Empty table cells produce empty strings
//! - SEC-3: Strip inline formatting to plain text
//! - SEC-4: Use `u8` for `nesting_depth` with saturation
//! - SEC-5: Single-pass O(n) event-based extraction
//! - SEC-6: Event-based parsing only (no regex)
//! - SEC-7: Enable GFM table support via `Options::ENABLE_TABLES`

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use serde::Serialize;

use crate::ForgeError;

use super::{build_line_starts, offset_to_line};

// ── Types ──────────────────────────────────────────────────────────────

/// Distinguishes ordered (numbered) from unordered (bullet) lists.
/// *(AR-004: `ListType` component)*
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ListType {
    /// Numbered list (1. 2. 3.)
    Ordered,
    /// Bullet list (- or * or +)
    Unordered,
}

/// A single list item extracted from a Markdown document.
/// *(AR-004: `ExtractedListItem` component)*
///
/// # Examples
///
/// ```
/// use forge::parse::{ExtractedListItem, ListType};
///
/// let item = ExtractedListItem {
///     text: "All systems must enforce MFA".to_string(),
///     source_line: 5,
///     nesting_depth: 0,
///     list_type: ListType::Ordered,
/// };
///
/// assert_eq!(item.nesting_depth, 0);
/// assert_eq!(item.list_type, ListType::Ordered);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExtractedListItem {
    /// Full text content of the list item. All text paragraphs within the
    /// item are concatenated (space-separated). Inline formatting is stripped
    /// to plain text *(SEC-3)*. Code blocks and blockquotes within the item
    /// are excluded *(EC-8)*.
    pub text: String,

    /// 1-based line number of the list item in the original document *(M-4)*.
    pub source_line: usize,

    /// 0-based nesting depth. 0 = top-level, 1 = nested one level, etc.
    /// Uses `u8` for bounded overflow safety; saturates at 255 *(SEC-4)*.
    pub nesting_depth: u8,

    /// Whether this item belongs to an ordered or unordered list *(M-1, M-2)*.
    pub list_type: ListType,
}

/// A table extracted from a Markdown document (GFM table syntax).
/// *(AR-004: `ExtractedTable` component; SEC-7 requires `ENABLE_TABLES`)*
///
/// # Examples
///
/// ```
/// use forge::parse::ExtractedTable;
///
/// let table = ExtractedTable {
///     headers: vec!["Role".to_string(), "Access Level".to_string()],
///     rows: vec![
///         vec!["Admin".to_string(), "Full".to_string()],
///         vec!["User".to_string(), "Read-only".to_string()],
///     ],
///     source_line: 42,
/// };
///
/// assert_eq!(table.headers.len(), 2);
/// assert_eq!(table.rows.len(), 2);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExtractedTable {
    /// Column header texts from the table header row.
    /// Inline formatting is stripped to plain text *(SEC-3)*.
    pub headers: Vec<String>,

    /// Data rows. Each row is a vector of cell strings matching header order.
    /// Empty cells are represented as empty strings *(SEC-2, EC-5)*.
    pub rows: Vec<Vec<String>>,

    /// 1-based line number of the table start in the original document *(M-4)*.
    pub source_line: usize,
}

/// A paragraph block extracted from Markdown content.
/// Only standalone paragraphs (not inside list items or tables) are captured.
/// *(AR-004: `ExtractedParagraph` component; spec S-2)*
///
/// # Examples
///
/// ```
/// use forge::parse::ExtractedParagraph;
///
/// let para = ExtractedParagraph {
///     text: "This section covers access control requirements.".to_string(),
///     source_line: 12,
/// };
///
/// assert_eq!(para.source_line, 12);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExtractedParagraph {
    /// Text content of the paragraph. Inline formatting stripped.
    pub text: String,

    /// 1-based line number of the paragraph start *(M-4)*.
    pub source_line: usize,
}

/// Container for all structural elements extracted from a document.
/// *(AR-004: `ExtractedContent` component)*
///
/// # Examples
///
/// ```
/// use forge::parse::ExtractedContent;
///
/// let content = ExtractedContent {
///     list_items: vec![],
///     tables: vec![],
///     paragraphs: vec![],
/// };
///
/// assert!(content.list_items.is_empty());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExtractedContent {
    /// All list items found in the document, in document order *(M-1, M-2)*.
    pub list_items: Vec<ExtractedListItem>,

    /// All tables found in the document, in document order *(M-3)*.
    pub tables: Vec<ExtractedTable>,

    /// Standalone paragraph text blocks, in document order *(S-2)*.
    pub paragraphs: Vec<ExtractedParagraph>,
}

// ── Functions ──────────────────────────────────────────────────────────

/// Extract list items, tables, and paragraphs from Markdown content.
///
/// Uses pulldown-cmark with GFM table extension *(SEC-7)* to parse the
/// event stream. Tracks nesting depth for list items *(SEC-4)* and
/// preserves table structure. Uses event-based parsing, not regex *(SEC-6)*.
///
/// # Arguments
///
/// * `content` - Raw Markdown content string.
///
/// # Returns
///
/// An `ExtractedContent` containing all list items, tables, and paragraphs
/// in document order. Returns empty `ExtractedContent` if the document has
/// no lists, tables, or paragraphs *(SEC-1)*.
///
/// # Errors
///
/// Returns `ForgeError::Parse` if extraction fails.
///
/// # Panics
///
/// This function does not intentionally panic. A list item encountered
/// outside of a list context is treated as invalid and is ignored (no
/// list item is emitted); this should never happen with well-formed
/// pulldown-cmark events.
///
/// # Algorithm (AR-004)
///
/// Single-pass O(n) event-based extraction *(SEC-5)*:
/// 1. Enable `Options::ENABLE_TABLES` for GFM table support
/// 2. Track list nesting with `Vec<ListType>` stack (depth counter)
/// 3. Accumulate text within list items (paragraphs only, per EC-8)
/// 4. Track table state (header vs data rows, cell accumulation)
/// 5. Capture standalone paragraphs (not inside items or tables)
/// 6. Strip inline formatting by processing only Text/Code/SoftBreak events
/// 7. Map byte offsets to 1-based line numbers via line-starts lookup table
#[allow(clippy::too_many_lines)]
pub fn extract_clauses(content: &str) -> Result<ExtractedContent, ForgeError> {
    // Enable GFM table support (SEC-7)
    let options = Options::ENABLE_TABLES;
    let parser = Parser::new_ext(content, options).into_offset_iter();

    // Build line-starts lookup table for O(log n) line number mapping (M-4)
    let line_starts = build_line_starts(content);

    // Initialize result vectors
    let mut list_items: Vec<ExtractedListItem> = Vec::new();
    let mut tables: Vec<ExtractedTable> = Vec::new();
    let mut paragraphs: Vec<ExtractedParagraph> = Vec::new();

    // List extraction state (T017)
    let mut list_type_stack: Vec<ListType> = Vec::new(); // Depth tracking via stack
    let mut item_stack: Vec<(String, usize)> = Vec::new(); // Stack of (text, source_line) for nested items
    let mut exclude_depth: usize = 0; // T019: Track CodeBlock/BlockQuote nesting

    // Table extraction state (T028)
    let mut in_table = false;
    let mut current_table_source_line: usize = 0;
    let mut table_headers: Vec<String> = Vec::new();
    let mut table_rows: Vec<Vec<String>> = Vec::new();
    let mut current_row: Vec<String> = Vec::new();
    let mut current_cell: String = String::new();

    // Paragraph extraction state (T035)
    let mut in_standalone_paragraph = false;
    let mut para_text = String::new();
    let mut para_source_line: usize = 0;
    // Process events (SEC-5: single-pass O(n))
    for (event, range) in parser {
        match event {
            // T017: List start/end — push/pop list type for depth tracking
            Event::Start(Tag::List(start_num)) => {
                let list_type =
                    if start_num.is_some() { ListType::Ordered } else { ListType::Unordered };
                list_type_stack.push(list_type);
            }
            Event::End(TagEnd::List(_)) => {
                list_type_stack.pop();
            }

            // T017: Item start/end — accumulate text within items
            Event::Start(Tag::Item) => {
                // Push a new item onto the stack
                let source_line = offset_to_line(range.start, &line_starts);
                item_stack.push((String::new(), source_line));
            }
            Event::End(TagEnd::Item) => {
                // Pop and save the completed item
                if let Some((text, source_line)) = item_stack.pop() {
                    // T016: Exclude empty items (only whitespace)
                    let trimmed = text.trim();
                    if !trimmed.is_empty() && !list_type_stack.is_empty() {
                        let list_type = *list_type_stack.last().unwrap();
                        // T017: nesting_depth with saturation at 255 (SEC-4)
                        let depth = list_type_stack.len() - 1;
                        #[allow(clippy::cast_possible_truncation)]
                        let nesting_depth = depth.min(255) as u8;

                        list_items.push(ExtractedListItem {
                            text: trimmed.to_string(),
                            source_line,
                            nesting_depth,
                            list_type,
                        });
                    }
                }
            }

            // T019: Exclude code blocks and blockquotes within items (EC-8)
            Event::Start(Tag::CodeBlock(_) | Tag::BlockQuote(_)) if !item_stack.is_empty() => {
                exclude_depth += 1;
            }
            Event::End(TagEnd::CodeBlock | TagEnd::BlockQuote(_)) if !item_stack.is_empty() => {
                exclude_depth = exclude_depth.saturating_sub(1);
            }

            // T018: Text accumulation within list items (SEC-3: strip formatting)
            // Accumulate text into the current (deepest) item
            Event::Text(text) if !item_stack.is_empty() && exclude_depth == 0 => {
                if let Some((item_text, _)) = item_stack.last_mut() {
                    item_text.push_str(&text);
                }
            }
            Event::Code(code) if !item_stack.is_empty() && exclude_depth == 0 => {
                if let Some((item_text, _)) = item_stack.last_mut() {
                    item_text.push_str(&code);
                }
            }
            Event::SoftBreak if !item_stack.is_empty() && exclude_depth == 0 => {
                if let Some((item_text, _)) = item_stack.last_mut() {
                    item_text.push(' '); // T018: SoftBreak → space
                }
            }
            Event::HardBreak if !item_stack.is_empty() && exclude_depth == 0 => {
                if let Some((item_text, _)) = item_stack.last_mut() {
                    item_text.push(' '); // T018: HardBreak → space
                }
            }
            Event::End(TagEnd::Paragraph) if !item_stack.is_empty() && exclude_depth == 0 => {
                if let Some((item_text, _)) = item_stack.last_mut() {
                    // Add space between paragraphs if text doesn't already end with whitespace
                    let last_is_whitespace =
                        item_text.chars().next_back().is_some_and(char::is_whitespace);
                    if !last_is_whitespace {
                        item_text.push(' '); // T018: Paragraph boundary → space
                    }
                }
            }

            // T028: Table extraction — track table/head/row/cell events
            Event::Start(Tag::Table(_)) => {
                in_table = true;
                current_table_source_line = offset_to_line(range.start, &line_starts);
                table_headers.clear();
                table_rows.clear();
                current_row.clear();
                current_cell.clear();
            }
            Event::End(TagEnd::Table) => {
                tables.push(ExtractedTable {
                    headers: std::mem::take(&mut table_headers),
                    rows: std::mem::take(&mut table_rows),
                    source_line: current_table_source_line,
                });
                in_table = false;
            }
            Event::Start(Tag::TableHead | Tag::TableRow) => {
                current_row.clear();
            }
            Event::End(TagEnd::TableHead) => {
                table_headers.clone_from(&current_row);
                current_row.clear();
            }
            Event::End(TagEnd::TableRow) => {
                table_rows.push(std::mem::take(&mut current_row));
            }
            Event::Start(Tag::TableCell) => {
                current_cell.clear();
            }
            Event::End(TagEnd::TableCell) => {
                current_row.push(current_cell.trim().to_string());
                current_cell.clear();
            }
            // Text/Code within table cells (SEC-3: inline formatting stripped)
            Event::Text(text) if in_table => {
                current_cell.push_str(&text);
            }
            Event::Code(code) if in_table => {
                current_cell.push_str(&code);
            }
            Event::SoftBreak if in_table => {
                current_cell.push(' ');
            }

            // T035: Standalone paragraph capture (S-2, SEC-3)
            Event::Start(Tag::Paragraph) if item_stack.is_empty() && !in_table => {
                in_standalone_paragraph = true;
                para_text.clear();
                para_source_line = offset_to_line(range.start, &line_starts);
            }
            Event::End(TagEnd::Paragraph) if in_standalone_paragraph => {
                let trimmed = para_text.trim().to_string();
                if !trimmed.is_empty() {
                    paragraphs
                        .push(ExtractedParagraph { text: trimmed, source_line: para_source_line });
                }
                in_standalone_paragraph = false;
            }
            Event::Text(text) if in_standalone_paragraph => para_text.push_str(&text),
            Event::Code(code) if in_standalone_paragraph => para_text.push_str(&code),
            Event::SoftBreak if in_standalone_paragraph => para_text.push(' '),

            // Ignore all other events (formatting tags, etc.)
            _ => {}
        }
    }

    // Sort items by source_line to maintain document order.
    // Necessary because nested items complete (and are pushed) before their parent items.
    // When End(Item) events fire, deeply nested children are finalized first, which can
    // reorder the list_items vector. Sorting by source_line restores original document order.
    list_items.sort_by_key(|item| item.source_line);

    Ok(ExtractedContent { list_items, tables, paragraphs })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Phase 2: Foundational Tests ───────────────────────────────────

    /// T005: Empty document returns empty ExtractedContent (SEC-1, EC-1)
    #[test]
    fn test_empty_document_returns_empty_content() {
        let result = extract_clauses("").unwrap();
        assert!(result.list_items.is_empty(), "Expected no list items");
        assert!(result.tables.is_empty(), "Expected no tables");
        assert!(result.paragraphs.is_empty(), "Expected no paragraphs");
    }

    /// T006: Document with only headings returns empty ExtractedContent (SEC-1)
    #[test]
    fn test_document_with_only_headings_returns_empty_content() {
        let md = "# Title\n\n## Section\n\n### Subsection\n";
        let result = extract_clauses(md).unwrap();
        assert!(result.list_items.is_empty(), "Headings should not produce list items");
        assert!(result.tables.is_empty(), "Headings should not produce tables");
        assert!(result.paragraphs.is_empty(), "Headings should not produce paragraphs");
    }

    // ── Phase 3: User Story 1 — List Extraction Tests (TDD) ─────────

    /// T009: Ordered list with 3 items (AC-1, M-1, M-4)
    #[test]
    fn test_ordered_list_three_items() {
        let md = "1. First requirement\n2. Second requirement\n3. Third requirement\n";
        let result = extract_clauses(md).unwrap();
        assert_eq!(result.list_items.len(), 3, "Expected 3 list items");

        assert_eq!(result.list_items[0].text, "First requirement");
        assert_eq!(result.list_items[0].list_type, ListType::Ordered);
        assert_eq!(result.list_items[0].nesting_depth, 0);
        assert_eq!(result.list_items[0].source_line, 1);

        assert_eq!(result.list_items[1].text, "Second requirement");
        assert_eq!(result.list_items[1].list_type, ListType::Ordered);
        assert_eq!(result.list_items[1].source_line, 2);

        assert_eq!(result.list_items[2].text, "Third requirement");
        assert_eq!(result.list_items[2].source_line, 3);
    }

    /// T010: Unordered list with 3 items (AC-2, M-2, M-4)
    #[test]
    fn test_unordered_list_three_items() {
        let md = "- First item\n- Second item\n- Third item\n";
        let result = extract_clauses(md).unwrap();
        assert_eq!(result.list_items.len(), 3, "Expected 3 list items");

        for (i, item) in result.list_items.iter().enumerate() {
            assert_eq!(item.list_type, ListType::Unordered, "Item {} should be unordered", i);
            assert_eq!(item.nesting_depth, 0, "Item {} should have depth 0", i);
        }

        assert_eq!(result.list_items[0].text, "First item");
        assert_eq!(result.list_items[0].source_line, 1);
        assert_eq!(result.list_items[1].text, "Second item");
        assert_eq!(result.list_items[2].text, "Third item");
    }

    /// T011: Nested list with 3 levels (AC-5, M-5)
    #[test]
    fn test_nested_list_three_levels() {
        let md = "\
1. Top level ordered
   - Nested unordered
      1. Deeply nested ordered
";
        let result = extract_clauses(md).unwrap();
        assert_eq!(result.list_items.len(), 3, "Expected 3 nested items");

        assert_eq!(result.list_items[0].text, "Top level ordered");
        assert_eq!(result.list_items[0].list_type, ListType::Ordered);
        assert_eq!(result.list_items[0].nesting_depth, 0);

        assert_eq!(result.list_items[1].text, "Nested unordered");
        assert_eq!(result.list_items[1].list_type, ListType::Unordered);
        assert_eq!(result.list_items[1].nesting_depth, 1);

        assert_eq!(result.list_items[2].text, "Deeply nested ordered");
        assert_eq!(result.list_items[2].list_type, ListType::Ordered);
        assert_eq!(result.list_items[2].nesting_depth, 2);
    }

    /// T012: Mixed list types in same document (M-1, M-2)
    #[test]
    fn test_mixed_list_types() {
        let md = "\
1. Ordered item
2. Another ordered

- Unordered item
- Another unordered
";
        let result = extract_clauses(md).unwrap();
        assert_eq!(result.list_items.len(), 4, "Expected 4 items");

        assert_eq!(result.list_items[0].list_type, ListType::Ordered);
        assert_eq!(result.list_items[1].list_type, ListType::Ordered);
        assert_eq!(result.list_items[2].list_type, ListType::Unordered);
        assert_eq!(result.list_items[3].list_type, ListType::Unordered);
    }

    /// T013: Deeply nested list (6 levels) for depth saturation (EC-2, SEC-4)
    #[test]
    fn test_deeply_nested_list_six_levels() {
        let md = "\
1. Level 0
   - Level 1
      1. Level 2
         - Level 3
            1. Level 4
               - Level 5
";
        let result = extract_clauses(md).unwrap();
        assert_eq!(result.list_items.len(), 6, "Expected 6 nested items");

        for (i, item) in result.list_items.iter().enumerate() {
            assert_eq!(item.nesting_depth, i as u8, "Item {} should have depth {}", i, i);
        }
    }

    /// T014: Inline formatting stripped to plain text (EC-4, SEC-3)
    #[test]
    fn test_inline_formatting_stripped() {
        let md = "- Item with **bold** and *italic* and `code` and [link](url)\n";
        let result = extract_clauses(md).unwrap();
        assert_eq!(result.list_items.len(), 1);

        let text = &result.list_items[0].text;
        // Should contain plain text without markup
        assert!(text.contains("bold"), "Should contain 'bold'");
        assert!(text.contains("italic"), "Should contain 'italic'");
        assert!(text.contains("code"), "Should contain 'code'");
        assert!(text.contains("link"), "Should contain 'link'");

        // Should NOT contain markup characters
        assert!(!text.contains("**"), "Should not contain bold markers");
        assert!(!text.contains("["), "Should not contain link brackets");
        assert!(!text.contains("]("), "Should not contain link syntax");
    }

    /// T015: Multi-paragraph list item with code block exclusion (EC-8)
    #[test]
    fn test_multi_paragraph_list_item() {
        let md = "\
- First paragraph.

  Second paragraph.

  ```
  code block should be excluded
  ```

  Third paragraph.
";
        let result = extract_clauses(md).unwrap();
        assert_eq!(result.list_items.len(), 1);

        let text = &result.list_items[0].text;
        assert!(text.contains("First paragraph"), "Should contain first paragraph");
        assert!(text.contains("Second paragraph"), "Should contain second paragraph");
        assert!(text.contains("Third paragraph"), "Should contain third paragraph");
        assert!(!text.contains("code block"), "Should NOT contain code block content");
    }

    /// T016: Empty list item excluded
    #[test]
    fn test_empty_list_item_excluded() {
        let md = "- Item with text\n-    \n- Another item\n";
        let result = extract_clauses(md).unwrap();
        // Empty item (only whitespace) should not be included
        assert_eq!(result.list_items.len(), 2, "Empty items should be excluded");
        assert_eq!(result.list_items[0].text, "Item with text");
        assert_eq!(result.list_items[1].text, "Another item");
    }

    // ── Phase 4: User Story 2 — Table Extraction Tests (TDD) ────────

    /// T022: Basic table with 3 columns and 5 data rows (AC-3, M-3, M-4)
    #[test]
    fn test_basic_table_three_columns_five_rows() {
        let md = "\
| Role   | Access Level | Department |
|--------|--------------|------------|
| Admin  | Full         | IT         |
| Editor | Write        | Content    |
| Viewer | Read-only    | General    |
| Audit  | Log access   | Compliance |
| Guest  | None         | External   |
";
        let result = extract_clauses(md).unwrap();
        assert_eq!(result.tables.len(), 1, "Expected 1 table");

        let table = &result.tables[0];
        assert_eq!(table.headers.len(), 3, "Expected 3 column headers");
        assert_eq!(table.headers[0], "Role");
        assert_eq!(table.headers[1], "Access Level");
        assert_eq!(table.headers[2], "Department");

        assert_eq!(table.rows.len(), 5, "Expected 5 data rows");
        assert_eq!(table.rows[0], vec!["Admin", "Full", "IT"]);
        assert_eq!(table.rows[1], vec!["Editor", "Write", "Content"]);
        assert_eq!(table.rows[2], vec!["Viewer", "Read-only", "General"]);
        assert_eq!(table.rows[3], vec!["Audit", "Log access", "Compliance"]);
        assert_eq!(table.rows[4], vec!["Guest", "None", "External"]);
    }

    /// T023: Table at a known line position records correct source_line (AC-4, M-4)
    #[test]
    fn test_table_source_line() {
        let md = "\
# Policy

Some introductory text.

| Control | Status   |
|---------|----------|
| AC-1    | Active   |
| AC-2    | Inactive |
";
        let result = extract_clauses(md).unwrap();
        assert_eq!(result.tables.len(), 1, "Expected 1 table");
        // Table starts at line 5 (after heading on line 1, blank, text on line 3, blank)
        assert_eq!(result.tables[0].source_line, 5, "Table should start at line 5");
    }

    /// T024: Table with empty cells produces empty strings (EC-5, SEC-2)
    #[test]
    fn test_table_empty_cells() {
        let md = "\
| Name  | Value |
|-------|-------|
| Alpha |       |
|       | Beta  |
";
        let result = extract_clauses(md).unwrap();
        assert_eq!(result.tables.len(), 1, "Expected 1 table");

        let table = &result.tables[0];
        assert_eq!(table.rows.len(), 2, "Expected 2 data rows");
        // First row: "Alpha" and empty
        assert_eq!(table.rows[0][0], "Alpha");
        assert_eq!(table.rows[0][1], "", "Empty cell should be empty string");
        // Second row: empty and "Beta"
        assert_eq!(table.rows[1][0], "", "Empty cell should be empty string");
        assert_eq!(table.rows[1][1], "Beta");
    }

    /// T025: Header-only table (no data rows) produces empty rows vec (EC-3)
    #[test]
    fn test_table_header_only_no_data_rows() {
        let md = "\
| Header A | Header B |
|----------|----------|
";
        let result = extract_clauses(md).unwrap();
        assert_eq!(result.tables.len(), 1, "Expected 1 table");

        let table = &result.tables[0];
        assert_eq!(table.headers.len(), 2, "Expected 2 headers");
        assert_eq!(table.headers[0], "Header A");
        assert_eq!(table.headers[1], "Header B");
        assert!(table.rows.is_empty(), "Header-only table should have no data rows");
    }

    /// T026: Inline formatting in table cells is stripped to plain text (SEC-3)
    #[test]
    fn test_table_inline_formatting_stripped() {
        let md = "\
| Feature       | Description          |
|---------------|----------------------|
| **Bold text** | *Italic text*        |
| `code span`   | [link text](http://x) |
";
        let result = extract_clauses(md).unwrap();
        assert_eq!(result.tables.len(), 1, "Expected 1 table");

        let table = &result.tables[0];
        assert_eq!(table.rows.len(), 2, "Expected 2 data rows");

        // Bold should be stripped
        assert_eq!(table.rows[0][0], "Bold text");
        // Italic should be stripped
        assert_eq!(table.rows[0][1], "Italic text");
        // Code should be stripped
        assert_eq!(table.rows[1][0], "code span");
        // Link should be stripped to link text only
        assert_eq!(table.rows[1][1], "link text");
    }

    /// T027: Multiple tables in one document are all extracted in order (M-3)
    #[test]
    fn test_multiple_tables() {
        let md = "\
| Col A | Col B |
|-------|-------|
| A1    | B1    |

Some text between tables.

| X | Y | Z |
|---|---|---|
| 1 | 2 | 3 |
| 4 | 5 | 6 |
";
        let result = extract_clauses(md).unwrap();
        assert_eq!(result.tables.len(), 2, "Expected 2 tables");

        // First table: 2 columns, 1 data row
        let t1 = &result.tables[0];
        assert_eq!(t1.headers, vec!["Col A", "Col B"]);
        assert_eq!(t1.rows.len(), 1);
        assert_eq!(t1.rows[0], vec!["A1", "B1"]);

        // Second table: 3 columns, 2 data rows
        let t2 = &result.tables[1];
        assert_eq!(t2.headers, vec!["X", "Y", "Z"]);
        assert_eq!(t2.rows.len(), 2);
        assert_eq!(t2.rows[0], vec!["1", "2", "3"]);
        assert_eq!(t2.rows[1], vec!["4", "5", "6"]);

        // Document order: first table should have smaller source_line
        assert!(
            t1.source_line < t2.source_line,
            "Tables should be in document order: {} < {}",
            t1.source_line,
            t2.source_line
        );
    }

    // ── Phase 5: User Story 4 — Paragraph Extraction Tests (TDD) ────

    /// T031: Standalone paragraph produces ExtractedParagraph (S-2)
    #[test]
    fn test_standalone_paragraph() {
        let md = "This is a standalone paragraph about access control.\n";
        let result = extract_clauses(md).unwrap();
        assert_eq!(result.paragraphs.len(), 1, "Expected 1 paragraph");
        assert_eq!(
            result.paragraphs[0].text,
            "This is a standalone paragraph about access control."
        );
        assert_eq!(result.paragraphs[0].source_line, 1);
    }

    /// T032: Paragraph text inside a list item is NOT captured as standalone
    #[test]
    fn test_paragraph_not_captured_inside_list() {
        let md = "\
- First list item
- Second list item

This paragraph is standalone.
";
        let result = extract_clauses(md).unwrap();
        // Only the standalone paragraph outside the list should appear
        assert_eq!(result.paragraphs.len(), 1, "Expected 1 standalone paragraph");
        assert_eq!(result.paragraphs[0].text, "This paragraph is standalone.");
        // List item text must not leak into paragraphs
        for para in &result.paragraphs {
            assert!(
                !para.text.contains("list item"),
                "List item text should not appear as a paragraph: {:?}",
                para.text
            );
        }
    }

    /// T033: Paragraph with inline formatting produces plain text (SEC-3)
    #[test]
    fn test_paragraph_with_inline_formatting() {
        let md =
            "A paragraph with **bold**, *italic*, `code`, and [a link](https://example.com).\n";
        let result = extract_clauses(md).unwrap();
        assert_eq!(result.paragraphs.len(), 1, "Expected 1 paragraph");

        let text = &result.paragraphs[0].text;
        // Plain text content should be present
        assert!(text.contains("bold"), "Should contain 'bold'");
        assert!(text.contains("italic"), "Should contain 'italic'");
        assert!(text.contains("code"), "Should contain 'code'");
        assert!(text.contains("a link"), "Should contain 'a link'");

        // Markdown formatting should be stripped
        assert!(!text.contains("**"), "Should not contain bold markers");
        assert!(!text.contains("["), "Should not contain link brackets");
        assert!(!text.contains("]("), "Should not contain link syntax");
    }

    /// T034: Multiple paragraphs in document order (S-2)
    #[test]
    fn test_multiple_paragraphs() {
        let md = "\
First paragraph of the document.

Second paragraph follows.

Third and final paragraph.
";
        let result = extract_clauses(md).unwrap();
        assert_eq!(result.paragraphs.len(), 3, "Expected 3 paragraphs");

        assert_eq!(result.paragraphs[0].text, "First paragraph of the document.");
        assert_eq!(result.paragraphs[1].text, "Second paragraph follows.");
        assert_eq!(result.paragraphs[2].text, "Third and final paragraph.");

        // Verify document order (source lines are ascending)
        assert!(
            result.paragraphs[0].source_line < result.paragraphs[1].source_line,
            "Paragraphs should be in document order"
        );
        assert!(
            result.paragraphs[1].source_line < result.paragraphs[2].source_line,
            "Paragraphs should be in document order"
        );
    }

    // ── Phase 6: Integration & Edge Case Tests ──────────────────────

    /// T038: Full document with all element types extracted together
    #[test]
    fn test_full_document_extraction() {
        let md = "\
# Security Policy

This document defines access control requirements.

1. All users must authenticate via MFA
2. Service accounts require key rotation

| Role   | Access |
|--------|--------|
| Admin  | Full   |
| Viewer | Read   |

Additional compliance notes apply.

- Annual review required
- Exceptions need CISO approval
";
        let result = extract_clauses(md).unwrap();

        // Paragraphs: 2 standalone (intro + compliance note)
        assert_eq!(result.paragraphs.len(), 2, "Expected 2 standalone paragraphs");
        assert!(result.paragraphs[0].text.contains("access control requirements"));
        assert!(result.paragraphs[1].text.contains("compliance notes"));

        // Ordered list items: 2
        let ordered: Vec<_> =
            result.list_items.iter().filter(|i| i.list_type == ListType::Ordered).collect();
        assert_eq!(ordered.len(), 2, "Expected 2 ordered list items");
        assert!(ordered[0].text.contains("MFA"));
        assert!(ordered[1].text.contains("key rotation"));

        // Unordered list items: 2
        let unordered: Vec<_> =
            result.list_items.iter().filter(|i| i.list_type == ListType::Unordered).collect();
        assert_eq!(unordered.len(), 2, "Expected 2 unordered list items");

        // Table: 1 with 2 headers and 2 rows
        assert_eq!(result.tables.len(), 1, "Expected 1 table");
        assert_eq!(result.tables[0].headers, vec!["Role", "Access"]);
        assert_eq!(result.tables[0].rows.len(), 2);
    }

    /// T039: Interleaved lists, tables, and paragraphs in document order
    #[test]
    fn test_document_with_lists_and_tables_mixed() {
        let md = "\
Intro paragraph.

- Bullet one
- Bullet two

| H1 | H2 |
|----|----|
| A  | B  |

Middle paragraph.

1. Numbered one
2. Numbered two

| X | Y |
|---|---|
| 1 | 2 |

Closing paragraph.
";
        let result = extract_clauses(md).unwrap();

        assert_eq!(result.paragraphs.len(), 3, "Expected 3 paragraphs");
        assert_eq!(result.paragraphs[0].text, "Intro paragraph.");
        assert_eq!(result.paragraphs[1].text, "Middle paragraph.");
        assert_eq!(result.paragraphs[2].text, "Closing paragraph.");

        assert_eq!(result.list_items.len(), 4, "Expected 4 list items");
        assert_eq!(result.list_items[0].list_type, ListType::Unordered);
        assert_eq!(result.list_items[1].list_type, ListType::Unordered);
        assert_eq!(result.list_items[2].list_type, ListType::Ordered);
        assert_eq!(result.list_items[3].list_type, ListType::Ordered);

        assert_eq!(result.tables.len(), 2, "Expected 2 tables");
        assert_eq!(result.tables[0].headers, vec!["H1", "H2"]);
        assert_eq!(result.tables[1].headers, vec!["X", "Y"]);

        // Verify document order across all element types
        assert!(result.paragraphs[0].source_line < result.list_items[0].source_line);
        assert!(result.list_items[1].source_line < result.tables[0].source_line);
        assert!(result.tables[0].source_line < result.paragraphs[1].source_line);
        assert!(result.paragraphs[1].source_line < result.list_items[2].source_line);
        assert!(result.tables[1].source_line < result.paragraphs[2].source_line);
    }

    /// T040: Code block inside list item is excluded from text (EC-8)
    #[test]
    fn test_list_item_with_code_block_excluded() {
        let md = "\
- Configure the system:

  ```bash
  sudo apt install nginx
  ```

  Then verify the installation.
";
        let result = extract_clauses(md).unwrap();
        assert_eq!(result.list_items.len(), 1, "Expected 1 list item");

        let text = &result.list_items[0].text;
        assert!(text.contains("Configure the system"), "Should have item text");
        assert!(text.contains("verify the installation"), "Should have trailing paragraph");
        assert!(!text.contains("sudo"), "Code block content must be excluded");
        assert!(!text.contains("nginx"), "Code block content must be excluded");
        assert!(!text.contains("bash"), "Code block language tag must be excluded");
    }

    /// T041: Nesting depth saturates at 255 (SEC-4)
    #[test]
    fn test_nesting_depth_saturates_at_255() {
        // Build a Markdown list nested 260 levels deep
        let mut md = String::new();
        for i in 0..260 {
            let indent = "   ".repeat(i);
            if i % 2 == 0 {
                md.push_str(&format!("{indent}1. Level {i}\n"));
            } else {
                md.push_str(&format!("{indent}- Level {i}\n"));
            }
        }

        let result = extract_clauses(&md).unwrap();
        assert!(!result.list_items.is_empty(), "Should have list items");

        // Find the maximum nesting depth
        let max_depth = result.list_items.iter().map(|i| i.nesting_depth).max().unwrap();
        assert_eq!(max_depth, 255, "Nesting depth should saturate at 255 (u8 max)");

        // Verify items beyond depth 255 still have nesting_depth == 255 (saturation)
        let deep_items: Vec<_> =
            result.list_items.iter().filter(|i| i.nesting_depth == 255).collect();
        assert!(deep_items.len() > 1, "Multiple items beyond depth 255 should all saturate at 255");
    }

    /// T042: Restarted numbered lists are independent sets
    #[test]
    fn test_restarted_numbered_list() {
        let md = "\
1. First set item A
2. First set item B

Some separator text.

1. Second set item A
2. Second set item B
3. Second set item C
";
        let result = extract_clauses(md).unwrap();
        assert_eq!(result.list_items.len(), 5, "Expected 5 total list items");

        // All should be ordered at depth 0
        for item in &result.list_items {
            assert_eq!(item.list_type, ListType::Ordered);
            assert_eq!(item.nesting_depth, 0);
        }

        // First set
        assert_eq!(result.list_items[0].text, "First set item A");
        assert_eq!(result.list_items[1].text, "First set item B");

        // Second set (after separator)
        assert_eq!(result.list_items[2].text, "Second set item A");
        assert_eq!(result.list_items[3].text, "Second set item B");
        assert_eq!(result.list_items[4].text, "Second set item C");

        // Separator text should be a standalone paragraph
        assert!(result.paragraphs.iter().any(|p| p.text == "Some separator text."));
    }

    /// T043: Mixed bullet markers (-, *, +) all produce Unordered
    #[test]
    fn test_mixed_bullet_markers() {
        let md = "\
- Dash item
* Star item
+ Plus item
";
        let result = extract_clauses(md).unwrap();
        assert_eq!(result.list_items.len(), 3, "Expected 3 list items");

        for (i, item) in result.list_items.iter().enumerate() {
            assert_eq!(
                item.list_type,
                ListType::Unordered,
                "Item {} ('{}') should be Unordered regardless of marker",
                i,
                item.text
            );
            assert_eq!(item.nesting_depth, 0, "All items should be top-level");
        }

        assert_eq!(result.list_items[0].text, "Dash item");
        assert_eq!(result.list_items[1].text, "Star item");
        assert_eq!(result.list_items[2].text, "Plus item");
    }
}
