//! Assembly functions for constructing the domain model from parsed structures.
//!
//! Converts parse-layer types (`SectionNode`, `ExtractedListItem`) into
//! domain-model types (`PolicySection`, `PolicyRequirement`) using line-range
//! association to map list items to their containing sections.

use std::path::Path;

use crate::ForgeError;
use crate::ingest::IngestedDocument;
use crate::parse::{ExtractedContent, ExtractedListItem, SectionNode};

use super::frontmatter::parse_frontmatter;
use super::{DocumentMetadata, PolicyDocument, PolicyRequirement, PolicySection};

/// Map a slice of `SectionNode`s and `ExtractedListItem`s into `PolicySection`s.
///
/// Associates each list item with its containing section by line range:
/// - For sibling sections at the same level, each section owns the line range
///   `[section.source_line, next_sibling.source_line)`.
/// - The last sibling owns `[section.source_line, usize::MAX)`.
/// - Children are processed recursively within their parent's range.
///
/// # Arguments
///
/// * `section_nodes` - Top-level (or sibling) section nodes from the parse layer.
/// * `list_items` - All list items that could belong to these sections or their children.
///
/// # Returns
///
/// A vector of `PolicySection`s with requirements populated from matching list items.
/// Returns a "Preamble" section if `section_nodes` is empty but `list_items` is not.
/// Returns an empty vector if both are empty.
pub(crate) fn map_sections(
    section_nodes: &[SectionNode],
    list_items: &[&ExtractedListItem],
) -> Vec<PolicySection> {
    if section_nodes.is_empty() {
        if list_items.is_empty() {
            return Vec::new();
        }
        // SEC-5: Don't silently drop orphaned list items
        let requirements: Vec<PolicyRequirement> = list_items
            .iter()
            .map(|item| PolicyRequirement {
                stable_id: None,
                text: item.text.clone(),
                source_line: item.source_line,
                nesting_depth: item.nesting_depth,
                atom_index: 0,
                parent_text: None,
                citations: vec![],
            })
            .collect();
        return vec![PolicySection {
            title: "Preamble".to_string(),
            heading_level: 0,
            source_line: list_items[0].source_line,
            body_text: None,
            children: vec![],
            requirements,
        }];
    }

    let first_heading_line = section_nodes[0].source_line;

    // Collect list items before the first heading into a Preamble section
    let preamble_items: Vec<&ExtractedListItem> =
        list_items.iter().copied().filter(|item| item.source_line < first_heading_line).collect();

    let mut result = Vec::new();

    if !preamble_items.is_empty() {
        let requirements: Vec<PolicyRequirement> = preamble_items
            .iter()
            .map(|item| PolicyRequirement {
                stable_id: None,
                text: item.text.clone(),
                source_line: item.source_line,
                nesting_depth: item.nesting_depth,
                atom_index: 0,
                parent_text: None,
                citations: vec![],
            })
            .collect();
        result.push(PolicySection {
            title: "Preamble".to_string(),
            heading_level: 0,
            source_line: preamble_items[0].source_line,
            body_text: None,
            children: vec![],
            requirements,
        });
    }

    result.extend(map_sections_recursive(section_nodes, list_items));
    result
}

/// Recursive implementation of section mapping (does not produce Preamble fallback).
fn map_sections_recursive(
    section_nodes: &[SectionNode],
    list_items: &[&ExtractedListItem],
) -> Vec<PolicySection> {
    if section_nodes.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::with_capacity(section_nodes.len());

    for (i, node) in section_nodes.iter().enumerate() {
        let range_start = node.source_line;
        let range_end = section_nodes.get(i + 1).map_or(usize::MAX, |next| next.source_line);

        // Filter list items within this section's line range.
        let items_in_range: Vec<&ExtractedListItem> = list_items
            .iter()
            .copied()
            .filter(|item| item.source_line >= range_start && item.source_line < range_end)
            .collect();

        // Recursively process children, passing only items within this section's range.
        let children = map_sections_recursive(&node.children, &items_in_range);

        // Items consumed by children should not also be requirements of the parent.
        // Collect all source lines owned by children's ranges.
        let child_owned: Vec<(usize, usize)> = build_child_ranges(&node.children);

        let requirements: Vec<PolicyRequirement> = items_in_range
            .iter()
            .filter(|item| !is_in_child_range(item.source_line, &child_owned))
            .map(|item| PolicyRequirement {
                stable_id: None,
                text: item.text.clone(),
                source_line: item.source_line,
                nesting_depth: item.nesting_depth,
                atom_index: 0,
                parent_text: None,
                citations: vec![],
            })
            .collect();

        result.push(PolicySection {
            title: node.title.clone(),
            heading_level: node.heading_level,
            source_line: node.source_line,
            body_text: node.body_text.clone(),
            children,
            requirements,
        });
    }

    result
}

/// Build line ranges `[start, end)` for each child section.
fn build_child_ranges(children: &[SectionNode]) -> Vec<(usize, usize)> {
    children
        .iter()
        .enumerate()
        .map(|(i, child)| {
            let start = child.source_line;
            let end = children.get(i + 1).map_or(usize::MAX, |next| next.source_line);
            (start, end)
        })
        .collect()
}

/// Check whether a line falls within any of the given child ranges.
fn is_in_child_range(line: usize, ranges: &[(usize, usize)]) -> bool {
    ranges.iter().any(|&(start, end)| line >= start && line < end)
}

/// Extract filename stem from a path, used as fallback document ID and title.
fn filename_stem(path: &Path) -> String {
    path.file_stem().and_then(|s| s.to_str()).unwrap_or("untitled").to_string()
}

/// Assemble a `PolicyDocument` from ingestion and extraction outputs.
///
/// Bridges the three extraction outputs into a unified domain model:
/// - `IngestedDocument` provides file path, content hash, raw content
/// - `Vec<SectionNode>` provides heading hierarchy
/// - `ExtractedContent` provides list items, tables, paragraphs
///
/// # Arguments
///
/// * `ingested` - Reference to the ingested document (from WI-2)
/// * `sections` - Section hierarchy tree (from WI-3)
/// * `clauses` - Extracted content (from WI-4)
///
/// # Returns
///
/// * `Ok(PolicyDocument)` - Complete domain model with all sections and requirements
/// * `Err(ForgeError)` - Fatal error preventing assembly
///
/// # Errors
///
/// Returns `ForgeError::Parse` if section tree structure is invalid.
/// Does NOT error on malformed YAML frontmatter (warns to stderr, uses fallback).
pub fn assemble_document(
    ingested: &IngestedDocument,
    sections: &[SectionNode],
    clauses: &ExtractedContent,
) -> Result<PolicyDocument, ForgeError> {
    let content = ingested.reconstruct_content();
    let frontmatter = parse_frontmatter(&content);

    // Resolve title: frontmatter -> first H1 heading -> filename stem
    let title = frontmatter
        .as_ref()
        .and_then(|fm| fm.title.clone())
        .or_else(|| sections.iter().find(|s| s.heading_level == 1).map(|s| s.title.clone()))
        .unwrap_or_else(|| filename_stem(&ingested.source_path));

    // Resolve version: frontmatter -> "0.0.0"
    let version = frontmatter
        .as_ref()
        .and_then(|fm| fm.version.clone())
        .unwrap_or_else(|| "0.0.0".to_string());

    // Author and date come from frontmatter only
    let author = frontmatter.as_ref().and_then(|fm| fm.author.clone());
    let date = frontmatter.as_ref().and_then(|fm| fm.date.clone());

    let metadata = DocumentMetadata {
        title,
        version,
        author,
        date,
        source_path: ingested.source_path.clone(),
        content_hash: Some(ingested.fingerprint.clone()),
    };

    let item_refs: Vec<&ExtractedListItem> = clauses.list_items.iter().collect();
    let mapped_sections = map_sections(sections, &item_refs);

    let id = filename_stem(&ingested.source_path);

    Ok(PolicyDocument { id, metadata, sections: mapped_sections })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::ingest::{IngestedDocument, SourceLine};
    use crate::parse::ListType;

    fn section(title: &str, level: u8, line: usize) -> SectionNode {
        SectionNode {
            title: title.to_string(),
            heading_level: level,
            source_line: line,
            body_text: None,
            children: vec![],
        }
    }

    fn section_with_children(
        title: &str,
        level: u8,
        line: usize,
        children: Vec<SectionNode>,
    ) -> SectionNode {
        SectionNode {
            title: title.to_string(),
            heading_level: level,
            source_line: line,
            body_text: None,
            children,
        }
    }

    fn list_item(text: &str, line: usize, depth: u8) -> ExtractedListItem {
        ExtractedListItem {
            text: text.to_string(),
            source_line: line,
            nesting_depth: depth,
            list_type: ListType::Unordered,
        }
    }

    // ── T006: map_sections tests ─────────────────────────────────────

    #[test]
    fn flat_three_sections_produce_three_policy_sections() {
        let nodes = vec![
            section("Introduction", 1, 1),
            section("Scope", 1, 10),
            section("References", 1, 20),
        ];

        let result = map_sections(&nodes, &[]);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].title, "Introduction");
        assert_eq!(result[0].heading_level, 1);
        assert_eq!(result[0].source_line, 1);
        assert_eq!(result[1].title, "Scope");
        assert_eq!(result[1].heading_level, 1);
        assert_eq!(result[1].source_line, 10);
        assert_eq!(result[2].title, "References");
        assert_eq!(result[2].heading_level, 1);
        assert_eq!(result[2].source_line, 20);
    }

    #[test]
    fn nested_h1_h2_h3_produces_correct_tree() {
        let nodes = vec![section_with_children(
            "Policy",
            1,
            1,
            vec![section_with_children("Access Control", 2, 5, vec![section("Passwords", 3, 10)])],
        )];

        let result = map_sections(&nodes, &[]);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "Policy");
        assert_eq!(result[0].heading_level, 1);
        assert_eq!(result[0].children.len(), 1);

        let h2 = &result[0].children[0];
        assert_eq!(h2.title, "Access Control");
        assert_eq!(h2.heading_level, 2);
        assert_eq!(h2.children.len(), 1);

        let h3 = &h2.children[0];
        assert_eq!(h3.title, "Passwords");
        assert_eq!(h3.heading_level, 3);
        assert!(h3.children.is_empty());
    }

    #[test]
    fn section_with_no_matching_list_items_has_empty_requirements() {
        // Two sections: first owns [1, 10), second owns [10, MAX)
        // Items only in the second section's range
        let nodes = vec![section("First", 1, 1), section("Second", 1, 10)];
        let items = [list_item("Belongs to second only", 15, 0)];
        let item_refs: Vec<&ExtractedListItem> = items.iter().collect();

        let result = map_sections(&nodes, &item_refs);

        assert_eq!(result.len(), 2);
        assert!(
            result[0].requirements.is_empty(),
            "Section with no matching list items should have empty requirements (EC-2)"
        );
        assert_eq!(result[1].requirements.len(), 1);
    }

    #[test]
    fn list_items_associated_with_correct_parent_section_by_line_range() {
        // Two sibling sections: lines 1-9 and lines 10+
        let nodes = vec![section("First", 1, 1), section("Second", 1, 10)];
        let items = [
            list_item("Belongs to First", 3, 0),
            list_item("Also belongs to First", 7, 0),
            list_item("Belongs to Second", 12, 0),
            list_item("Also belongs to Second", 15, 1),
        ];
        let item_refs: Vec<&ExtractedListItem> = items.iter().collect();

        let result = map_sections(&nodes, &item_refs);

        assert_eq!(result.len(), 2);

        // First section owns lines [1, 10)
        assert_eq!(result[0].requirements.len(), 2);
        assert_eq!(result[0].requirements[0].text, "Belongs to First");
        assert_eq!(result[0].requirements[0].source_line, 3);
        assert_eq!(result[0].requirements[1].text, "Also belongs to First");
        assert_eq!(result[0].requirements[1].source_line, 7);

        // Second section owns lines [10, MAX)
        assert_eq!(result[1].requirements.len(), 2);
        assert_eq!(result[1].requirements[0].text, "Belongs to Second");
        assert_eq!(result[1].requirements[0].source_line, 12);
        assert_eq!(result[1].requirements[1].text, "Also belongs to Second");
        assert_eq!(result[1].requirements[1].nesting_depth, 1);
    }

    #[test]
    fn empty_sections_vec_returns_empty_vec() {
        let result = map_sections(&[], &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn orphaned_list_items_produce_preamble_section() {
        let items = [list_item("Req one", 5, 0), list_item("Req two", 8, 0)];
        let item_refs: Vec<&ExtractedListItem> = items.iter().collect();
        let result = map_sections(&[], &item_refs);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "Preamble");
        assert_eq!(result[0].heading_level, 0);
        assert_eq!(result[0].requirements.len(), 2);
        assert_eq!(result[0].requirements[0].text, "Req one");
    }

    #[test]
    fn preamble_items_before_first_heading_preserved() {
        let nodes = vec![section("Introduction", 1, 10)];
        let items = [
            list_item("Before heading 1", 3, 0),
            list_item("Before heading 2", 5, 0),
            list_item("After heading", 15, 0),
        ];
        let item_refs: Vec<&ExtractedListItem> = items.iter().collect();
        let result = map_sections(&nodes, &item_refs);

        // Should have Preamble + Introduction
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].title, "Preamble");
        assert_eq!(result[0].heading_level, 0);
        assert_eq!(result[0].requirements.len(), 2);
        assert_eq!(result[0].requirements[0].text, "Before heading 1");
        assert_eq!(result[0].requirements[1].text, "Before heading 2");
        assert_eq!(result[1].title, "Introduction");
        assert_eq!(result[1].requirements.len(), 1);
        assert_eq!(result[1].requirements[0].text, "After heading");
    }

    // ── Additional edge case tests ───────────────────────────────────

    #[test]
    fn stable_id_is_none_for_all_requirements() {
        let nodes = vec![section("Policy", 1, 1)];
        let items = [list_item("Req A", 3, 0), list_item("Req B", 5, 1)];
        let item_refs: Vec<&ExtractedListItem> = items.iter().collect();

        let result = map_sections(&nodes, &item_refs);

        for req in &result[0].requirements {
            assert!(req.stable_id.is_none(), "stable_id must be None (SEC-7)");
        }
    }

    #[test]
    fn source_line_preserved_from_section_node_to_policy_section() {
        let nodes = vec![section("First", 2, 42), section("Second", 2, 99)];

        let result = map_sections(&nodes, &[]);

        assert_eq!(result[0].source_line, 42, "source_line must be preserved (SEC-4)");
        assert_eq!(result[1].source_line, 99, "source_line must be preserved (SEC-4)");
    }

    #[test]
    fn body_text_preserved_from_section_node() {
        let node = SectionNode {
            title: "Scope".to_string(),
            heading_level: 2,
            source_line: 5,
            body_text: Some("This section defines scope.".to_string()),
            children: vec![],
        };

        let result = map_sections(&[node], &[]);

        assert_eq!(result[0].body_text.as_deref(), Some("This section defines scope."));
    }

    #[test]
    fn child_section_items_not_duplicated_in_parent_requirements() {
        // Parent at line 1 with child at line 10
        let nodes = vec![section_with_children("Parent", 1, 1, vec![section("Child", 2, 10)])];

        let items = [
            list_item("Parent req", 5, 0), // In parent range, before child
            list_item("Child req", 12, 0), // In child range
        ];
        let item_refs: Vec<&ExtractedListItem> = items.iter().collect();

        let result = map_sections(&nodes, &item_refs);

        // Parent should only have the item at line 5
        assert_eq!(result[0].requirements.len(), 1);
        assert_eq!(result[0].requirements[0].text, "Parent req");

        // Child should have the item at line 12
        assert_eq!(result[0].children[0].requirements.len(), 1);
        assert_eq!(result[0].children[0].requirements[0].text, "Child req");
    }

    #[test]
    fn nesting_depth_carried_from_list_item_to_requirement() {
        let nodes = vec![section("Policy", 1, 1)];
        let items = [
            list_item("Top level", 3, 0),
            list_item("Nested once", 4, 1),
            list_item("Nested twice", 5, 2),
        ];
        let item_refs: Vec<&ExtractedListItem> = items.iter().collect();

        let result = map_sections(&nodes, &item_refs);

        assert_eq!(result[0].requirements.len(), 3);
        assert_eq!(result[0].requirements[0].nesting_depth, 0);
        assert_eq!(result[0].requirements[1].nesting_depth, 1);
        assert_eq!(result[0].requirements[2].nesting_depth, 2);
    }

    // ── T007: assemble_document tests ────────────────────────────────

    fn make_ingested(filename: &str, content: &str) -> IngestedDocument {
        let lines: Vec<SourceLine> = content
            .lines()
            .enumerate()
            .map(|(i, text)| SourceLine { number: i + 1, text: text.to_string() })
            .collect();

        IngestedDocument {
            source_path: PathBuf::from(filename),
            fingerprint: "abc123def456".to_string(),
            lines,
        }
    }

    #[test]
    fn assemble_with_yaml_frontmatter_populates_metadata() {
        let content = "---\ntitle: Security Policy\nversion: \"1.0\"\nauthor: Jane\ndate: 2026-01-15\n---\n\n# Access Control\n\n- Users must authenticate\n";
        let ingested = make_ingested("policy.md", content);

        let sections = vec![SectionNode {
            title: "Access Control".to_string(),
            heading_level: 1,
            source_line: 8,
            body_text: None,
            children: vec![],
        }];

        let clauses = ExtractedContent {
            list_items: vec![ExtractedListItem {
                text: "Users must authenticate".to_string(),
                source_line: 10,
                nesting_depth: 0,
                list_type: ListType::Unordered,
            }],
            tables: vec![],
            paragraphs: vec![],
        };

        let doc = assemble_document(&ingested, &sections, &clauses).unwrap();

        assert_eq!(doc.metadata.title, "Security Policy");
        assert_eq!(doc.metadata.version, "1.0");
        assert_eq!(doc.metadata.author.as_deref(), Some("Jane"));
        assert_eq!(doc.metadata.date.as_deref(), Some("2026-01-15"));
        assert_eq!(doc.id, "policy");
        assert_eq!(doc.sections.len(), 1);
        assert_eq!(doc.sections[0].requirements.len(), 1);
    }

    #[test]
    fn assemble_no_frontmatter_falls_back_to_h1_and_default_version() {
        let content = "# Access Control Policy\n\nSome body text.\n";
        let ingested = make_ingested("policy.md", content);

        let sections = vec![SectionNode {
            title: "Access Control Policy".to_string(),
            heading_level: 1,
            source_line: 1,
            body_text: Some("Some body text.".to_string()),
            children: vec![],
        }];

        let clauses = ExtractedContent { list_items: vec![], tables: vec![], paragraphs: vec![] };

        let doc = assemble_document(&ingested, &sections, &clauses).unwrap();

        assert_eq!(doc.metadata.title, "Access Control Policy");
        assert_eq!(doc.metadata.version, "0.0.0");
        assert!(doc.metadata.author.is_none());
        assert!(doc.metadata.date.is_none());
    }

    #[test]
    fn assemble_no_frontmatter_no_headings_falls_back_to_filename() {
        let content = "Just some plain text.\n";
        let ingested = make_ingested("my-policy.md", content);

        let sections = vec![];
        let clauses = ExtractedContent { list_items: vec![], tables: vec![], paragraphs: vec![] };

        let doc = assemble_document(&ingested, &sections, &clauses).unwrap();

        assert_eq!(doc.metadata.title, "my-policy");
        assert_eq!(doc.metadata.version, "0.0.0");
        assert_eq!(doc.id, "my-policy");
    }

    #[test]
    fn assemble_empty_document_returns_ok_with_empty_sections() {
        let content = "";
        let ingested = make_ingested("empty.md", content);

        let doc = assemble_document(
            &ingested,
            &[],
            &ExtractedContent { list_items: vec![], tables: vec![], paragraphs: vec![] },
        )
        .unwrap();

        assert!(doc.sections.is_empty());
        assert_eq!(doc.metadata.title, "empty");
        assert_eq!(doc.metadata.version, "0.0.0");
    }

    #[test]
    fn assemble_malformed_yaml_returns_ok_with_fallback() {
        let content = "---\ntitle: [broken\nversion: : : bad\n---\n\n# Fallback Title\n";
        let ingested = make_ingested("fallback.md", content);

        let sections = vec![SectionNode {
            title: "Fallback Title".to_string(),
            heading_level: 1,
            source_line: 6,
            body_text: None,
            children: vec![],
        }];

        let clauses = ExtractedContent { list_items: vec![], tables: vec![], paragraphs: vec![] };

        let doc = assemble_document(&ingested, &sections, &clauses).unwrap();

        assert_eq!(doc.metadata.title, "Fallback Title");
        assert_eq!(doc.metadata.version, "0.0.0");
    }

    #[test]
    fn assemble_three_sections_ten_requirements() {
        let content = "---\ntitle: Full Policy\nversion: \"2.0\"\n---\n\n# Section A\n\n- Req 1\n- Req 2\n- Req 3\n\n# Section B\n\n- Req 4\n- Req 5\n- Req 6\n- Req 7\n\n# Section C\n\n- Req 8\n- Req 9\n- Req 10\n";
        let ingested = make_ingested("full.md", content);

        let sections = vec![
            SectionNode {
                title: "Section A".to_string(),
                heading_level: 1,
                source_line: 6,
                body_text: None,
                children: vec![],
            },
            SectionNode {
                title: "Section B".to_string(),
                heading_level: 1,
                source_line: 12,
                body_text: None,
                children: vec![],
            },
            SectionNode {
                title: "Section C".to_string(),
                heading_level: 1,
                source_line: 19,
                body_text: None,
                children: vec![],
            },
        ];

        let mut items = Vec::new();
        for (i, line) in [
            (8, "Req 1"),
            (9, "Req 2"),
            (10, "Req 3"),
            (14, "Req 4"),
            (15, "Req 5"),
            (16, "Req 6"),
            (17, "Req 7"),
            (21, "Req 8"),
            (22, "Req 9"),
            (23, "Req 10"),
        ] {
            items.push(ExtractedListItem {
                text: line.to_string(),
                source_line: i,
                nesting_depth: 0,
                list_type: ListType::Unordered,
            });
        }

        let clauses = ExtractedContent { list_items: items, tables: vec![], paragraphs: vec![] };

        let doc = assemble_document(&ingested, &sections, &clauses).unwrap();

        assert_eq!(doc.metadata.title, "Full Policy");
        assert_eq!(doc.metadata.version, "2.0");
        assert_eq!(doc.sections.len(), 3);

        let total_reqs: usize = doc.sections.iter().map(|s| s.requirements.len()).sum();
        assert_eq!(total_reqs, 10);

        assert_eq!(doc.sections[0].requirements.len(), 3);
        assert_eq!(doc.sections[1].requirements.len(), 4);
        assert_eq!(doc.sections[2].requirements.len(), 3);
    }

    #[test]
    fn assemble_content_hash_from_fingerprint() {
        let ingested = make_ingested("test.md", "# Title\n");
        let sections = vec![SectionNode {
            title: "Title".to_string(),
            heading_level: 1,
            source_line: 1,
            body_text: None,
            children: vec![],
        }];
        let clauses = ExtractedContent { list_items: vec![], tables: vec![], paragraphs: vec![] };
        let doc = assemble_document(&ingested, &sections, &clauses).unwrap();

        assert_eq!(doc.metadata.content_hash.as_deref(), Some("abc123def456"));
        assert_eq!(doc.metadata.source_path, PathBuf::from("test.md"));
    }

    // ── T012: Section source_line traceability ───────────────────────

    #[test]
    fn section_source_lines_match_section_node_source_lines() {
        let content = "---\ntitle: Trace Test\nversion: \"1.0\"\n---\n\n# Section at 10\n\nBody.\n\n## Section at 20\n\nBody.\n\n### Section at 35\n\nBody.\n";
        let ingested = make_ingested("trace.md", content);

        let sections = vec![SectionNode {
            title: "Section at 10".to_string(),
            heading_level: 1,
            source_line: 10,
            body_text: None,
            children: vec![SectionNode {
                title: "Section at 20".to_string(),
                heading_level: 2,
                source_line: 20,
                body_text: None,
                children: vec![SectionNode {
                    title: "Section at 35".to_string(),
                    heading_level: 3,
                    source_line: 35,
                    body_text: None,
                    children: vec![],
                }],
            }],
        }];

        let clauses = ExtractedContent { list_items: vec![], tables: vec![], paragraphs: vec![] };

        let doc = assemble_document(&ingested, &sections, &clauses).unwrap();

        assert_eq!(doc.sections[0].source_line, 10, "H1 source_line mismatch");
        assert_eq!(doc.sections[0].children[0].source_line, 20, "H2 source_line mismatch");
        assert_eq!(
            doc.sections[0].children[0].children[0].source_line, 35,
            "H3 source_line mismatch"
        );
    }

    // ── T013: Requirement source_line traceability ───────────────────

    #[test]
    fn requirement_source_lines_match_list_item_source_lines() {
        let content = "# Policy\n\n- Req at 15\n\n## Sub\n\n- Req at 22\n- Req at 30\n";
        let ingested = make_ingested("req-trace.md", content);

        let sections = vec![SectionNode {
            title: "Policy".to_string(),
            heading_level: 1,
            source_line: 1,
            body_text: None,
            children: vec![SectionNode {
                title: "Sub".to_string(),
                heading_level: 2,
                source_line: 5,
                body_text: None,
                children: vec![],
            }],
        }];

        let clauses = ExtractedContent {
            list_items: vec![
                ExtractedListItem {
                    text: "Req at 15".to_string(),
                    source_line: 15,
                    nesting_depth: 0,
                    list_type: ListType::Unordered,
                },
                ExtractedListItem {
                    text: "Req at 22".to_string(),
                    source_line: 22,
                    nesting_depth: 0,
                    list_type: ListType::Unordered,
                },
                ExtractedListItem {
                    text: "Req at 30".to_string(),
                    source_line: 30,
                    nesting_depth: 0,
                    list_type: ListType::Unordered,
                },
            ],
            tables: vec![],
            paragraphs: vec![],
        };

        let doc = assemble_document(&ingested, &sections, &clauses).unwrap();

        // Req at 15 belongs to parent (line range [1, 5))
        // Wait — actually line 15 > 5, so it goes to "Sub" section [5, MAX)
        // Let me adjust: parent owns [1, 5), child owns [5, MAX)
        // Line 15, 22, 30 all > 5, so they go to Sub

        let sub_reqs = &doc.sections[0].children[0].requirements;
        assert_eq!(sub_reqs.len(), 3);
        assert_eq!(sub_reqs[0].source_line, 15, "First req source_line mismatch");
        assert_eq!(sub_reqs[1].source_line, 22, "Second req source_line mismatch");
        assert_eq!(sub_reqs[2].source_line, 30, "Third req source_line mismatch");
    }

    // ── T014: Data completeness — no silent drops ────────────────────

    #[test]
    fn assemble_preserves_all_sections_and_requirements_without_drops() {
        // Count output sections recursively
        fn count_sections(sections: &[super::super::PolicySection]) -> usize {
            sections.iter().map(|s| 1 + count_sections(&s.children)).sum()
        }

        fn count_requirements(sections: &[super::super::PolicySection]) -> usize {
            sections.iter().map(|s| s.requirements.len() + count_requirements(&s.children)).sum()
        }

        let content = "# S1\n\n- R1\n- R2\n\n# S2\n\n- R3\n\n## S2.1\n\n- R4\n- R5\n";
        let ingested = make_ingested("complete.md", content);

        let sections = vec![
            SectionNode {
                title: "S1".to_string(),
                heading_level: 1,
                source_line: 1,
                body_text: None,
                children: vec![],
            },
            SectionNode {
                title: "S2".to_string(),
                heading_level: 1,
                source_line: 6,
                body_text: None,
                children: vec![SectionNode {
                    title: "S2.1".to_string(),
                    heading_level: 2,
                    source_line: 10,
                    body_text: None,
                    children: vec![],
                }],
            },
        ];

        let clauses = ExtractedContent {
            list_items: vec![
                ExtractedListItem {
                    text: "R1".to_string(),
                    source_line: 3,
                    nesting_depth: 0,
                    list_type: ListType::Unordered,
                },
                ExtractedListItem {
                    text: "R2".to_string(),
                    source_line: 4,
                    nesting_depth: 0,
                    list_type: ListType::Unordered,
                },
                ExtractedListItem {
                    text: "R3".to_string(),
                    source_line: 8,
                    nesting_depth: 0,
                    list_type: ListType::Unordered,
                },
                ExtractedListItem {
                    text: "R4".to_string(),
                    source_line: 12,
                    nesting_depth: 0,
                    list_type: ListType::Unordered,
                },
                ExtractedListItem {
                    text: "R5".to_string(),
                    source_line: 13,
                    nesting_depth: 0,
                    list_type: ListType::Unordered,
                },
            ],
            tables: vec![],
            paragraphs: vec![],
        };

        let input_section_count = 3; // S1, S2, S2.1
        let input_item_count = 5; // R1-R5

        let doc = assemble_document(&ingested, &sections, &clauses).unwrap();

        let output_section_count = count_sections(&doc.sections);
        let output_req_count = count_requirements(&doc.sections);

        assert_eq!(
            output_section_count, input_section_count,
            "Section count mismatch: some sections were silently dropped (SEC-5)"
        );
        assert_eq!(
            output_req_count, input_item_count,
            "Requirement count mismatch: some requirements were silently dropped (SEC-5)"
        );
    }
}
