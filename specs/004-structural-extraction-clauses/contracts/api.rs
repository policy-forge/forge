// Contract: Structural Extraction — Clauses & Tables (WI-4)
//
// ⚠️  SPECIFICATION-ONLY FILE — NOT COMPILED
// This file is documentation of the API contract. The actual implementation
// resides in src/parse/clauses.rs. The `use crate::ForgeError;` import and
// `todo!()` placeholder are intentional per contract-first development.
//
// This file defines the PUBLIC API contract for the clause/table extraction
// module. Types and function signatures defined here BEFORE implementation.
// Naming follows AR-004 Interface Definitions.
//
// Location: src/parse/clauses.rs (re-exported from src/parse/mod.rs)
// Dependencies: serde::Serialize, crate::ForgeError
// Security: SEC-004 requirements traced inline

use serde::Serialize;
use crate::ForgeError;  // ⚠️ Note: This is spec-only; won't compile standalone

// ── Types ──────────────────────────────────────────────────────────────

/// Distinguishes ordered (numbered) from unordered (bullet) lists.
/// *(AR-004: ListType component)*
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ListType {
    /// Numbered list (1. 2. 3.)
    Ordered,
    /// Bullet list (- or * or +)
    Unordered,
}

/// A single list item extracted from a Markdown document.
/// *(AR-004: ExtractedListItem component)*
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
/// *(AR-004: ExtractedTable component; SEC-7 requires ENABLE_TABLES)*
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
/// *(AR-004: ExtractedParagraph component; spec S-2)*
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
/// *(AR-004: ExtractedContent component)*
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
pub fn extract_clauses(content: &str) -> Result<ExtractedContent, ForgeError> {
    todo!("Implementation follows contract-first development")
}
