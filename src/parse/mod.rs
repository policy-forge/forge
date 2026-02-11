//! Markdown structural extraction — headings.
//!
//! Parses Markdown content to extract heading elements (H1-H6) into a
//! hierarchical `Vec<SectionNode>` tree using pulldown-cmark's event-based
//! parser with a stack-based single-pass O(n) algorithm.

use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};

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
    /// Heading title text (e.g., "Access Control" from `## Access Control`).
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
///    - Text/Code/SoftBreak between headings: accumulate into current node's `body_text`
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
pub fn extract_sections(content: &str) -> Result<Vec<SectionNode>, ForgeError> {
    let line_starts = build_line_starts(content);
    let mut stack: Vec<(u8, SectionNode)> = Vec::new();
    let mut roots: Vec<SectionNode> = Vec::new();
    let mut in_heading = false;
    let mut title_buf = String::new();
    let mut current_level: u8 = 0;
    let mut current_line: usize = 0;

    for (event, range) in Parser::new(content).into_offset_iter() {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                in_heading = true;
                title_buf.clear();
                current_level = heading_level_to_u8(level);
                current_line = offset_to_line(range.start, &line_starts);
            }
            Event::End(TagEnd::Heading(_)) => {
                in_heading = false;
                let node = SectionNode {
                    title: title_buf.clone(),
                    heading_level: current_level,
                    source_line: current_line,
                    body_text: None,
                    children: Vec::new(),
                };

                // Pop stack until top has level < current_level
                pop_to_parent(&mut stack, &mut roots, current_level);

                stack.push((current_level, node));
            }
            Event::Text(text) if in_heading => {
                title_buf.push_str(&text);
            }
            Event::Code(code) if in_heading => {
                title_buf.push_str(&code);
            }
            // Body text accumulation (between headings, per R-6)
            Event::Text(text) if !in_heading => {
                if let Some((_, node)) = stack.last_mut() {
                    node.body_text.get_or_insert_with(String::new).push_str(&text);
                }
            }
            Event::Code(code) if !in_heading => {
                if let Some((_, node)) = stack.last_mut() {
                    let body = node.body_text.get_or_insert_with(String::new);
                    body.push('`');
                    body.push_str(&code);
                    body.push('`');
                }
            }
            Event::SoftBreak | Event::HardBreak if !in_heading => {
                if let Some((_, node)) = stack.last_mut() {
                    node.body_text.get_or_insert_with(String::new).push('\n');
                }
            }
            _ => {}
        }
    }

    // Drain remaining stack
    drain_stack(&mut stack, &mut roots);

    for root in &mut roots {
        finalize_body(root);
    }

    Ok(roots)
}

/// Pop stack entries with level >= `current_level`, attaching them to their
/// parent or to roots.
fn pop_to_parent(
    stack: &mut Vec<(u8, SectionNode)>,
    roots: &mut Vec<SectionNode>,
    current_level: u8,
) {
    while stack.last().is_some_and(|&(top_level, _)| top_level >= current_level) {
        if let Some((_, popped)) = stack.pop() {
            if let Some((_, parent)) = stack.last_mut() {
                parent.children.push(popped);
            } else {
                roots.push(popped);
            }
        }
    }
}

/// Drain all remaining stack entries into roots, preserving parent-child order.
fn drain_stack(stack: &mut Vec<(u8, SectionNode)>, roots: &mut Vec<SectionNode>) {
    while let Some((_, popped)) = stack.pop() {
        if let Some((_, parent)) = stack.last_mut() {
            parent.children.push(popped);
        } else {
            roots.push(popped);
        }
    }
}

/// Trim trailing whitespace from `body_text`, converting empty bodies to `None`.
fn finalize_body(node: &mut SectionNode) {
    if let Some(ref mut body) = node.body_text {
        let trimmed = body.trim_end().to_string();
        if trimmed.is_empty() {
            node.body_text = None;
        } else {
            *body = trimmed;
        }
    }
    for child in &mut node.children {
        finalize_body(child);
    }
}

/// Convert pulldown-cmark's `HeadingLevel` enum to a `u8` (1-6).
fn heading_level_to_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

/// Build a lookup table of byte offsets where each line starts.
///
/// Line 1 starts at byte 0. Each subsequent line starts at the byte
/// after a `\n` character.
fn build_line_starts(content: &str) -> Vec<usize> {
    let mut starts = vec![0]; // Line 1 starts at byte 0
    for (i, byte) in content.bytes().enumerate() {
        if byte == b'\n' {
            starts.push(i + 1);
        }
    }
    starts
}

/// Convert a byte offset to a 1-based line number using the line-starts table.
///
/// Uses binary search (`partition_point`) for O(log n) lookup.
fn offset_to_line(offset: usize, line_starts: &[usize]) -> usize {
    line_starts.partition_point(|&start| start <= offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── T007: offset_to_line unit tests ──────────────────────────────

    #[test]
    fn offset_to_line_returns_1_at_offset_0() {
        let starts = build_line_starts("hello\nworld\n");
        assert_eq!(offset_to_line(0, &starts), 1);
    }

    #[test]
    fn offset_to_line_correct_for_multi_line() {
        // "hello\nworld\nfoo"
        // Line 1: bytes 0-5, Line 2: bytes 6-11, Line 3: bytes 12-14
        let content = "hello\nworld\nfoo";
        let starts = build_line_starts(content);
        assert_eq!(offset_to_line(0, &starts), 1); // 'h' in "hello"
        assert_eq!(offset_to_line(3, &starts), 1); // 'l' in "hello"
        assert_eq!(offset_to_line(6, &starts), 2); // 'w' in "world"
        assert_eq!(offset_to_line(12, &starts), 3); // 'f' in "foo"
    }

    #[test]
    fn offset_to_line_at_newline_boundary() {
        let content = "hello\nworld\n";
        let starts = build_line_starts(content);
        // Offset 5 is the '\n' itself — still part of line 1
        assert_eq!(offset_to_line(5, &starts), 1);
        // Offset 6 is 'w' — start of line 2
        assert_eq!(offset_to_line(6, &starts), 2);
    }

    #[test]
    fn offset_to_line_at_end_of_content() {
        let content = "hello\nworld";
        let starts = build_line_starts(content);
        // Offset 10 is 'd' — last char, still line 2
        assert_eq!(offset_to_line(10, &starts), 2);
    }

    #[test]
    fn offset_to_line_empty_content() {
        let content = "";
        let starts = build_line_starts(content);
        // Only entry is [0]. offset 0 should return line 1.
        assert_eq!(offset_to_line(0, &starts), 1);
    }

    #[test]
    fn heading_level_to_u8_all_levels() {
        assert_eq!(heading_level_to_u8(HeadingLevel::H1), 1);
        assert_eq!(heading_level_to_u8(HeadingLevel::H2), 2);
        assert_eq!(heading_level_to_u8(HeadingLevel::H3), 3);
        assert_eq!(heading_level_to_u8(HeadingLevel::H4), 4);
        assert_eq!(heading_level_to_u8(HeadingLevel::H5), 5);
        assert_eq!(heading_level_to_u8(HeadingLevel::H6), 6);
    }

    // ── T008: single H1 heading ─────────────────────────────────────

    #[test]
    fn single_h1_produces_one_root_node() {
        let sections = extract_sections("# Policy\n").unwrap();
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].title, "Policy");
        assert_eq!(sections[0].heading_level, 1);
        assert_eq!(sections[0].source_line, 1);
        assert!(sections[0].children.is_empty());
    }

    // ── T009: H1 followed by H2 ─────────────────────────────────────

    #[test]
    fn h1_followed_by_h2_produces_nested_tree() {
        let md = "# Policy\n\n## Access Control\n";
        let sections = extract_sections(md).unwrap();
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].title, "Policy");
        assert_eq!(sections[0].children.len(), 1);
        assert_eq!(sections[0].children[0].title, "Access Control");
        assert_eq!(sections[0].children[0].heading_level, 2);
    }

    // ── T010: H1 → H2 → H3 three-level nesting ─────────────────────

    #[test]
    fn h1_h2_h3_produces_three_level_tree() {
        let md = "# Policy\n\n## Access Control\n\n### Passwords\n";
        let sections = extract_sections(md).unwrap();
        assert_eq!(sections.len(), 1);
        let h1 = &sections[0];
        assert_eq!(h1.title, "Policy");
        assert_eq!(h1.children.len(), 1);
        let h2 = &h1.children[0];
        assert_eq!(h2.title, "Access Control");
        assert_eq!(h2.children.len(), 1);
        let h3 = &h2.children[0];
        assert_eq!(h3.title, "Passwords");
        assert_eq!(h3.heading_level, 3);
    }

    // ── T011: heading title and line number accuracy (AC-2) ─────────

    #[test]
    fn heading_records_correct_title_level_and_line() {
        let md = "Some preamble\n\n## Access Control\n";
        let sections = extract_sections(md).unwrap();
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].title, "Access Control");
        assert_eq!(sections[0].heading_level, 2);
        assert_eq!(sections[0].source_line, 3);
    }

    // ── T012: five headings at various levels (AC-1) ────────────────

    #[test]
    fn five_headings_at_various_levels_in_correct_hierarchy() {
        let md = "\
# Policy
## Scope
## Access Control
### Authentication
### Authorization
";
        let sections = extract_sections(md).unwrap();
        // One root (H1 "Policy") with two H2 children
        assert_eq!(sections.len(), 1);
        let root = &sections[0];
        assert_eq!(root.title, "Policy");
        assert_eq!(root.children.len(), 2);
        assert_eq!(root.children[0].title, "Scope");
        assert_eq!(root.children[0].children.len(), 0);
        assert_eq!(root.children[1].title, "Access Control");
        assert_eq!(root.children[1].children.len(), 2);
        assert_eq!(root.children[1].children[0].title, "Authentication");
        assert_eq!(root.children[1].children[1].title, "Authorization");
    }

    // ── T015: H1 → H3 skip places H3 as child of H1 (AC-4, SEC-1) ──

    #[test]
    fn h1_to_h3_skip_places_h3_as_child_of_h1() {
        let md = "# Policy\n\n### Deep Section\n";
        let sections = extract_sections(md).unwrap();
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].title, "Policy");
        assert_eq!(sections[0].children.len(), 1);
        assert_eq!(sections[0].children[0].title, "Deep Section");
        assert_eq!(sections[0].children[0].heading_level, 3);
    }

    // ── T016: multiple H1s produce separate top-level sections (EC-4) ─

    #[test]
    fn multiple_h1s_produce_separate_roots() {
        let md = "# First\n\n# Second\n\n# Third\n";
        let sections = extract_sections(md).unwrap();
        assert_eq!(sections.len(), 3);
        assert_eq!(sections[0].title, "First");
        assert_eq!(sections[1].title, "Second");
        assert_eq!(sections[2].title, "Third");
    }

    // ── T017: document starting with H3 (EC-2, SEC-1) ───────────────

    #[test]
    fn document_starting_with_h3_makes_it_top_level() {
        let md = "### Deep Start\n\n### Another\n";
        let sections = extract_sections(md).unwrap();
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].title, "Deep Start");
        assert_eq!(sections[0].heading_level, 3);
        assert_eq!(sections[1].title, "Another");
    }

    // ── T018: no headings returns empty Vec (EC-1, SEC-2) ───────────

    #[test]
    fn no_headings_returns_empty_vec() {
        let md = "Just some text.\n\nMore text.\n";
        let sections = extract_sections(md).unwrap();
        assert!(sections.is_empty());
    }

    // ── T019: empty heading text (EC-3) ─────────────────────────────

    #[test]
    fn empty_heading_produces_empty_title() {
        let md = "## \n";
        let sections = extract_sections(md).unwrap();
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].title, "");
        assert_eq!(sections[0].heading_level, 2);
    }

    // ── T020: single heading produces single-node tree (EC-6) ───────

    #[test]
    fn single_heading_produces_single_node() {
        let md = "## Only One\n";
        let sections = extract_sections(md).unwrap();
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].title, "Only One");
        assert!(sections[0].children.is_empty());
    }

    // ── T021: H1 → H3 → H2 pops H3 and makes H2 sibling ──────────

    #[test]
    fn h1_h3_h2_pops_h3_makes_h2_sibling() {
        let md = "# Root\n\n### Deep\n\n## Sibling\n";
        let sections = extract_sections(md).unwrap();
        assert_eq!(sections.len(), 1);
        let root = &sections[0];
        assert_eq!(root.title, "Root");
        assert_eq!(root.children.len(), 2);
        assert_eq!(root.children[0].title, "Deep");
        assert_eq!(root.children[0].heading_level, 3);
        assert_eq!(root.children[1].title, "Sibling");
        assert_eq!(root.children[1].heading_level, 2);
    }

    // ── T024: body text with two paragraphs (S-1 scenario 1) ────────

    #[test]
    fn heading_with_body_captures_paragraphs() {
        let md = "## Scope\n\nFirst paragraph.\n\nSecond paragraph.\n";
        let sections = extract_sections(md).unwrap();
        assert_eq!(sections.len(), 1);
        let body = sections[0].body_text.as_deref().unwrap();
        assert!(body.contains("First paragraph."), "body: {body:?}");
        assert!(body.contains("Second paragraph."), "body: {body:?}");
    }

    // ── T025: heading with no body has body_text None (S-1 scenario 2)

    #[test]
    fn heading_with_no_body_has_none() {
        let md = "## First\n## Second\n";
        let sections = extract_sections(md).unwrap();
        // "First" has no content before "Second"
        assert_eq!(sections[0].body_text, None);
    }

    // ── T026: text before first heading is discarded (EC-5, A-4) ────

    #[test]
    fn text_before_first_heading_is_discarded() {
        let md = "This is preamble text.\n\n# Title\n";
        let sections = extract_sections(md).unwrap();
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].title, "Title");
        // Preamble text should not appear in any section
        assert_eq!(sections[0].body_text, None);
    }

    // ── T027: inline code in body is preserved ──────────────────────

    #[test]
    fn inline_code_in_body_is_preserved() {
        let md = "## Config\n\nUse `foo` for setup.\n";
        let sections = extract_sections(md).unwrap();
        let body = sections[0].body_text.as_deref().unwrap();
        assert!(body.contains("`foo`"), "body should contain backtick-wrapped code: {body:?}");
    }

    // ── T030: Debug output includes all fields ──────────────────────

    #[test]
    fn debug_output_includes_all_fields() {
        let md = "# Policy\n\n## Scope\n\nSome body.\n";
        let sections = extract_sections(md).unwrap();
        let debug = format!("{sections:?}");
        assert!(debug.contains("title"), "debug should contain 'title': {debug}");
        assert!(debug.contains("heading_level"), "debug should contain 'heading_level': {debug}");
        assert!(debug.contains("source_line"), "debug should contain 'source_line': {debug}");
        assert!(debug.contains("body_text"), "debug should contain 'body_text': {debug}");
        assert!(debug.contains("children"), "debug should contain 'children': {debug}");
        assert!(debug.contains("Policy"), "debug should contain heading title: {debug}");
        assert!(debug.contains("Scope"), "debug should contain child title: {debug}");
    }

    // ── T031: pretty debug output is readable ───────────────────────

    #[test]
    fn pretty_debug_output_is_indented_and_readable() {
        let md = "# Root\n\n## Child\n\n### Grandchild\n";
        let sections = extract_sections(md).unwrap();
        let pretty = format!("{sections:#?}");
        // Pretty-printed Debug should have indented nesting
        assert!(pretty.contains("Root"), "should contain 'Root'");
        assert!(pretty.contains("Child"), "should contain 'Child'");
        assert!(pretty.contains("Grandchild"), "should contain 'Grandchild'");
        // Should have newlines (pretty formatting)
        assert!(pretty.contains('\n'), "pretty debug should have newlines");
        // Should show nested children structure
        assert!(pretty.contains("children:"), "should show children field");
    }

    // ── T033: integration test — all 25 example policy documents (SC-005) ─

    #[test]
    fn all_example_policy_documents_produce_non_empty_sections() {
        let example_dir = std::path::Path::new("example_data");
        assert!(example_dir.exists(), "example_data/ directory must exist");

        let mut md_count = 0;
        for entry in std::fs::read_dir(example_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "md") {
                md_count += 1;
                let content = std::fs::read_to_string(&path)
                    .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));
                let sections = extract_sections(&content)
                    .unwrap_or_else(|e| panic!("Failed to parse {}: {e}", path.display()));
                assert!(!sections.is_empty(), "Expected non-empty sections for {}", path.display());
            }
        }
        assert_eq!(md_count, 25, "Expected 25 .md files in example_data/");
    }
}
