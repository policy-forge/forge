//! Integration tests for the atomization pipeline.
//!
//! Tests the public API of `atomize_document` and `atomize_requirement`
//! end-to-end, verifying the full pipeline from `PolicyDocument` input
//! to atomized `PolicyDocument` output.

use std::collections::HashSet;

use forge::parse::{atomize_document, atomize_requirement};

mod common;
use common::{make_doc, make_req, make_section};

// ===================================================================
// T014: Integration tests for atomize_document
// ===================================================================

#[test]
fn document_with_compound_and_atomic_increases_count() {
    // AC-8: document with 1 compound + 1 atomic → total count increases
    let doc = make_doc(
        "Integration Test Policy",
        vec![make_section(
            "Access Control",
            vec![
                make_req("Systems must enforce MFA and must require complex passwords", 10),
                make_req("All systems must enforce MFA", 20),
            ],
        )],
    );

    let original_count = doc.total_requirements();
    assert_eq!(original_count, 2);

    let result = atomize_document(&doc).unwrap();

    // 2 from compound + 1 atomic = 3 total
    assert_eq!(result.total_requirements(), 3);
    assert!(result.total_requirements() > original_count);
}

#[test]
fn split_requirements_have_sequential_atom_index_and_shared_source_line() {
    let doc = make_doc(
        "Test",
        vec![make_section(
            "S1",
            vec![make_req(
                "All employees must complete security training and must acknowledge the acceptable use policy or must request a waiver",
                42,
            )],
        )],
    );

    let result = atomize_document(&doc).unwrap();
    let reqs = &result.sections[0].requirements;

    assert_eq!(reqs.len(), 3);
    let original_text = "All employees must complete security training and must acknowledge the acceptable use policy or must request a waiver";
    let fragment_texts: HashSet<&str> = reqs.iter().map(|req| req.text.as_str()).collect();
    assert_eq!(fragment_texts.len(), reqs.len(), "split fragments must have distinct text");
    for (i, req) in reqs.iter().enumerate() {
        assert_eq!(req.atom_index, i);
        assert_eq!(req.source_line, 42);
        assert!(!req.text.trim().is_empty(), "split fragment must not be empty");
        assert_eq!(req.parent_text.as_deref(), Some(original_text));
    }
}

#[test]
fn atomic_requirement_preserved_unchanged_in_document() {
    let doc = make_doc(
        "Test",
        vec![make_section("S1", vec![make_req("Passwords must be at least 12 characters", 5)])],
    );

    let result = atomize_document(&doc).unwrap();
    let reqs = &result.sections[0].requirements;

    assert_eq!(reqs.len(), 1);
    assert_eq!(reqs[0].text, "Passwords must be at least 12 characters");
    assert_eq!(reqs[0].source_line, 5);
    assert_eq!(reqs[0].atom_index, 0);
    assert!(reqs[0].parent_text.is_none());
}

#[test]
fn empty_document_returned_unchanged_ec8() {
    // EC-8: empty document
    let doc = make_doc("Empty", vec![]);

    let result = atomize_document(&doc).unwrap();
    assert_eq!(result.metadata.title, "Empty");
    assert!(result.sections.is_empty());
    assert_eq!(result.total_requirements(), 0);
}

#[test]
fn document_with_empty_sections() {
    let doc = make_doc(
        "Test",
        vec![
            make_section("S1", vec![]),
            make_section("S2", vec![make_req("Systems must enforce MFA", 1)]),
        ],
    );

    let result = atomize_document(&doc).unwrap();
    assert_eq!(result.sections.len(), 2);
    assert_eq!(result.sections[0].requirements.len(), 0);
    assert_eq!(result.sections[1].requirements.len(), 1);
}

#[test]
fn atomize_requirement_preserves_text_byte_for_byte() {
    // SEC-9: atomic text preserved byte-for-byte
    let original_text = "Systems must encrypt and store data securely";
    let req = make_req(original_text, 7);
    let result = atomize_requirement(&req).unwrap();

    assert_eq!(result.requirements.len(), 1, "atomic requirement must remain a single requirement");
    assert_eq!(result.requirements[0].text, original_text);
}

#[test]
fn all_stable_ids_are_64_char_hex_in_document() {
    let doc = make_doc(
        "Test",
        vec![make_section(
            "S1",
            vec![
                make_req("Systems must enforce MFA and must require passwords", 1),
                make_req("All systems must enforce MFA", 2),
            ],
        )],
    );

    let result = atomize_document(&doc).unwrap();
    let mut stable_ids = HashSet::new();
    for section in &result.sections {
        for req in &section.requirements {
            let id = req.stable_id.as_deref().expect("stable_id should be set after atomization");
            assert_eq!(id.len(), 64, "ID length mismatch for: {}", req.text);
            assert!(
                id.chars().all(|c| c.is_ascii_hexdigit()),
                "Non-hex char in ID for: {}",
                req.text
            );
            assert!(stable_ids.insert(id), "Duplicate stable ID for: {}", req.text);
        }
    }
}

#[test]
fn multi_section_document_atomization() {
    let doc = make_doc(
        "Full Policy",
        vec![
            make_section(
                "Authentication",
                vec![
                    make_req("Systems must enforce MFA and must require complex passwords", 1),
                    make_req("All accounts must have unique passwords", 2),
                ],
            ),
            make_section(
                "Authorization",
                vec![make_req("Access must be role-based and must follow least privilege", 10)],
            ),
            make_section(
                "Audit",
                vec![make_req(
                    "The organization shall log all access and shall retain logs for 90 days",
                    20,
                )],
            ),
        ],
    );

    let result = atomize_document(&doc).unwrap();

    // Section 1: 2 from compound + 1 atomic = 3
    assert_eq!(result.sections[0].requirements.len(), 3);
    // Section 2: 1 compound → 2
    assert_eq!(result.sections[1].requirements.len(), 2);
    // Section 3: 1 compound → 2
    assert_eq!(result.sections[2].requirements.len(), 2);
    // Total: 7
    assert_eq!(result.total_requirements(), 7);
}
