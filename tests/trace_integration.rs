//! Integration tests for traceability model (WI-16).
//!
//! T018: Catalog trace capture
//! T023: Component definition trace capture

use std::path::PathBuf;

use forge::model::trace::TraceLinkCollection;
use forge::model::{DocumentMetadata, PolicyDocument, PolicyRequirement, PolicySection};

// ── Test Helpers ────────────────────────────────────────────────────────

fn test_requirement(text: &str, stable_id: &str, line: usize) -> PolicyRequirement {
    PolicyRequirement {
        stable_id: Some(stable_id.to_string()),
        text: text.to_string(),
        source_line: line,
        nesting_depth: 0,
        atom_index: 0,
        parent_text: None,
        citations: vec![],
    }
}

fn test_section(
    title: &str,
    reqs: Vec<PolicyRequirement>,
    children: Vec<PolicySection>,
) -> PolicySection {
    PolicySection {
        title: title.to_string(),
        heading_level: 2,
        source_line: 1,
        body_text: None,
        children,
        requirements: reqs,
    }
}

fn test_document(sections: Vec<PolicySection>) -> PolicyDocument {
    PolicyDocument {
        id: "test-policy".to_string(),
        metadata: DocumentMetadata {
            title: "Test Security Policy".to_string(),
            version: "1.0".to_string(),
            author: None,
            date: None,
            source_path: PathBuf::from("policies/security.md"),
            content_hash: None,
        },
        sections,
    }
}

// ── T018: Catalog trace capture ────────────────────────────────────────

#[test]
fn catalog_trace_one_link_per_control() {
    let doc = test_document(vec![
        test_section(
            "Access Control",
            vec![
                test_requirement("Users must authenticate.", "uuid-ac-1", 10),
                test_requirement("MFA is required.", "uuid-ac-2", 15),
            ],
            vec![],
        ),
        test_section(
            "Data Protection",
            vec![test_requirement("Encrypt at rest.", "uuid-dp-1", 30)],
            vec![],
        ),
    ]);

    let mut trace_links = TraceLinkCollection::new();
    let catalog = forge::oscal::build_catalog(&doc, Some(&mut trace_links)).unwrap();

    // One TraceLink per control
    let total_controls: usize = catalog.groups.iter().map(|g| g.controls.len()).sum();
    assert_eq!(trace_links.len(), total_controls);
    assert_eq!(trace_links.len(), 3);
}

#[test]
fn catalog_trace_json_path_format() {
    let doc = test_document(vec![
        test_section(
            "Access Control",
            vec![
                test_requirement("Auth.", "uuid-ac-1", 10),
                test_requirement("MFA.", "uuid-ac-2", 15),
            ],
            vec![],
        ),
        test_section(
            "Data Protection",
            vec![test_requirement("Encrypt.", "uuid-dp-1", 30)],
            vec![],
        ),
    ]);

    let mut trace_links = TraceLinkCollection::new();
    forge::oscal::build_catalog(&doc, Some(&mut trace_links)).unwrap();

    // Verify dot-notation path format
    let paths: Vec<&str> = trace_links.iter().map(|l| l.oscal_json_path.as_str()).collect();
    assert_eq!(paths[0], "catalog.groups[0].controls[0]");
    assert_eq!(paths[1], "catalog.groups[0].controls[1]");
    assert_eq!(paths[2], "catalog.groups[1].controls[0]");
}

#[test]
fn catalog_trace_source_location_fields() {
    let doc = test_document(vec![test_section(
        "Access Control",
        vec![test_requirement("Auth required.", "uuid-ac-1", 42)],
        vec![],
    )]);

    let mut trace_links = TraceLinkCollection::new();
    forge::oscal::build_catalog(&doc, Some(&mut trace_links)).unwrap();

    let link = trace_links.by_oscal_element("uuid-ac-1").unwrap();
    assert_eq!(link.source_location.file_path, PathBuf::from("policies/security.md"));
    assert_eq!(link.source_location.section_title, "Access Control");
    assert_eq!(link.source_location.line_number, 42);
    assert_eq!(link.requirement_stable_id, "uuid-ac-1");
}

#[test]
fn catalog_trace_none_is_backward_compatible() {
    let doc = test_document(vec![test_section(
        "Access Control",
        vec![test_requirement("Auth.", "uuid-1", 10)],
        vec![],
    )]);

    // None trace_links — backward compatible
    let catalog = forge::oscal::build_catalog(&doc, None).unwrap();
    assert_eq!(catalog.groups.len(), 1);
}

// ── Nested section trace accuracy ────────────────────────────────────

#[test]
fn catalog_trace_nested_section_uses_subsection_title() {
    // Parent section "Access Control" with a child section "Password Policy".
    // Requirements in the child should trace back to "Password Policy",
    // not to the parent "Access Control".
    let child = test_section(
        "Password Policy",
        vec![test_requirement("Passwords must be 12+ chars.", "uuid-pw-1", 20)],
        vec![],
    );
    let parent = test_section(
        "Access Control",
        vec![test_requirement("Users must authenticate.", "uuid-ac-1", 10)],
        vec![child],
    );
    let doc = test_document(vec![parent]);

    let mut trace_links = TraceLinkCollection::new();
    forge::oscal::build_catalog(&doc, Some(&mut trace_links)).unwrap();

    assert_eq!(trace_links.len(), 2);

    // Parent requirement traces to parent section title
    let parent_links = trace_links.by_requirement("uuid-ac-1");
    assert_eq!(parent_links.len(), 1);
    assert_eq!(parent_links[0].source_location.section_title, "Access Control");

    // Child requirement traces to child section title, not the parent
    let child_links = trace_links.by_requirement("uuid-pw-1");
    assert_eq!(child_links.len(), 1);
    assert_eq!(child_links[0].source_location.section_title, "Password Policy");
}

// ── T023: Component definition trace capture ───────────────────────────

#[test]
fn component_def_trace_empty_when_no_implemented_requirements() {
    let doc = test_document(vec![test_section(
        "Access Control",
        vec![test_requirement("Auth.", "uuid-1", 10)],
        vec![],
    )]);

    let mut trace_links = TraceLinkCollection::new();
    let envelope = forge::oscal::build_component_definition(&doc, Some(&mut trace_links)).unwrap();

    // WI-15 not merged: no implemented-requirements → empty trace collection
    assert!(trace_links.is_empty());
    assert_eq!(envelope.component_definition.components.len(), 1);
}

#[test]
fn component_def_trace_none_is_backward_compatible() {
    let doc = test_document(vec![]);

    // None trace_links — backward compatible
    let envelope = forge::oscal::build_component_definition(&doc, None).unwrap();
    assert_eq!(envelope.component_definition.components.len(), 1);
}
