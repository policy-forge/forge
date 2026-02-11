// Contract: Structural Extraction — Headings
// This file defines the type contracts for the heading extraction feature.
// It is NOT compiled code — it is a design artifact that specifies the
// public API before implementation (Constitution Principle III).

use crate::ForgeError;

/// A node in the section hierarchy tree, representing one Markdown heading
/// and its associated content.
///
/// # Examples
///
/// ```
/// use forge::parse::SectionNode;
///
/// let node = SectionNode {
///     title: "Access Control".to_string(),
///     heading_level: 2,
///     source_line: 10,
///     body_text: Some("This section covers access control requirements.".to_string()),
///     children: vec![],
/// };
///
/// assert_eq!(node.heading_level, 2);
/// assert_eq!(node.source_line, 10);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct SectionNode {
    /// Heading title text (e.g., "Access Control" from "## Access Control").
    /// May be empty for headings with no text content.
    pub title: String,

    /// Heading level: 1 for H1, 2 for H2, ..., 6 for H6.
    pub heading_level: u8,

    /// Source line number in the original document (1-based).
    pub source_line: usize,

    /// Text content between this heading and the next heading (if any).
    /// `None` if no body content exists between this heading and the next.
    pub body_text: Option<String>,

    /// Child sections (headings at deeper levels nested under this one).
    pub children: Vec<SectionNode>,
}

/// Extract a hierarchical section tree from Markdown content.
///
/// Parses the content using pulldown-cmark's event-based parser, identifies
/// heading events, and constructs a tree where deeper headings are children
/// of shallower ones using a stack-based single-pass O(n) algorithm.
///
/// # Arguments
///
/// * `content` - Raw Markdown content string.
///
/// # Returns
///
/// A vector of top-level `SectionNode` trees. Multiple H1 headings produce
/// multiple top-level entries. An empty document or a document with no
/// headings returns an empty vector.
///
/// # Errors
///
/// Returns `ForgeError::Parse` if the content cannot be parsed.
/// In practice, pulldown-cmark accepts any UTF-8 string, so this error
/// is unlikely for valid input.
///
/// # Algorithm
///
/// Uses the stack-based tree construction algorithm from AR-003:
/// 1. Initialize empty stack: `Vec<(u8, SectionNode)>`
/// 2. Initialize empty root list: `Vec<SectionNode>`
/// 3. For each `(event, range)` from `pulldown_cmark::Parser::new(content).into_offset_iter()`:
///    - `Start(Heading { level, .. })`: pop stack to level < N, push new node
///    - Text/Code/SoftBreak between headings: accumulate into current node's body_text
///    - Other events: skip
/// 4. Drain stack (pop all remaining, attaching to parents or root list)
/// 5. Return root list
///
/// # Security Requirements
///
/// - SEC-1: Handles all heading level combinations without panicking
/// - SEC-2: Returns empty Vec for documents with no headings
/// - SEC-3: Uses explicit stack (Vec), not call-stack recursion
/// - SEC-4: Tree depth bounded by heading levels (max 6)
pub fn extract_sections(content: &str) -> Result<Vec<SectionNode>, ForgeError>;
