//! Integration tests for the atomization pipeline.
//!
//! Tests the public API of `atomize_document` and `atomize_requirement`
//! end-to-end, verifying the full pipeline from PolicyDocument input
//! to atomized PolicyDocument output.

use forge::model::{PolicyDocument, PolicySection};
use forge::parse::{atomize_document, atomize_requirement};

mod common;
use common::make_req;

// ===================================================================
// T014: Integration tests for atomize_document
// ===================================================================

#[test]
fn document_with_compound_and_atomic_increases_count() {
    // AC-8: document with 1 compound + 1 atomic → total count increases
    let doc = PolicyDocument {
        title: "Integration Test Policy".to_string(),
        sections: vec![PolicySection {
            heading: "Access Control".to_string(),
            requirements: vec![
                make_req("Systems must enforce MFA and must require complex passwords", 10),
                make_req("All systems must enforce MFA", 20),
            ],
        }],
    };

    let original_count = doc.total_requirement_count();
    assert_eq!(original_count, 2);

    let result = atomize_document(&doc).unwrap();

    // 2 from compound + 1 atomic = 3 total
    assert_eq!(result.total_requirement_count(), 3);
    assert!(result.total_requirement_count() > original_count);
}

#[test]
fn split_requirements_have_sequential_atom_index_and_shared_source_line() {
    let doc = PolicyDocument {
        title: "Test".to_string(),
        sections: vec![PolicySection {
            heading: "S1".to_string(),
            requirements: vec![make_req(
                "All employees must complete security training and must acknowledge the acceptable use policy or must request a waiver",
                42,
            )],
        }],
    };

    let result = atomize_document(&doc).unwrap();
    let reqs = &result.sections[0].requirements;

    assert_eq!(reqs.len(), 3);
    for (i, req) in reqs.iter().enumerate() {
        assert_eq!(req.atom_index, i);
        assert_eq!(req.source_line, 42);
        assert!(req.parent_text.is_some());
    }
}

#[test]
fn atomic_requirement_preserved_unchanged_in_document() {
    let doc = PolicyDocument {
        title: "Test".to_string(),
        sections: vec![PolicySection {
            heading: "S1".to_string(),
            requirements: vec![make_req("Passwords must be at least 12 characters", 5)],
        }],
    };

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
    let doc = PolicyDocument { title: "Empty".to_string(), sections: vec![] };

    let result = atomize_document(&doc).unwrap();
    assert_eq!(result.title, "Empty");
    assert!(result.sections.is_empty());
    assert_eq!(result.total_requirement_count(), 0);
}

#[test]
fn document_with_empty_sections() {
    let doc = PolicyDocument {
        title: "Test".to_string(),
        sections: vec![
            PolicySection { heading: "S1".to_string(), requirements: vec![] },
            PolicySection {
                heading: "S2".to_string(),
                requirements: vec![make_req("Systems must enforce MFA", 1)],
            },
        ],
    };

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

    assert_eq!(result.requirements[0].text, original_text);
}

#[test]
fn all_stable_ids_are_64_char_hex_in_document() {
    let doc = PolicyDocument {
        title: "Test".to_string(),
        sections: vec![PolicySection {
            heading: "S1".to_string(),
            requirements: vec![
                make_req("Systems must enforce MFA and must require passwords", 1),
                make_req("All systems must enforce MFA", 2),
            ],
        }],
    };

    let result = atomize_document(&doc).unwrap();
    for section in &result.sections {
        for req in &section.requirements {
            assert_eq!(req.stable_id.len(), 64, "ID length mismatch for: {}", req.text);
            assert!(
                req.stable_id.chars().all(|c| c.is_ascii_hexdigit()),
                "Non-hex char in ID for: {}",
                req.text
            );
        }
    }
}

#[test]
fn multi_section_document_atomization() {
    let doc = PolicyDocument {
        title: "Full Policy".to_string(),
        sections: vec![
            PolicySection {
                heading: "Authentication".to_string(),
                requirements: vec![
                    make_req("Systems must enforce MFA and must require complex passwords", 1),
                    make_req("All accounts must have unique passwords", 2),
                ],
            },
            PolicySection {
                heading: "Authorization".to_string(),
                requirements: vec![make_req(
                    "Access must be role-based and must follow least privilege",
                    10,
                )],
            },
            PolicySection {
                heading: "Audit".to_string(),
                requirements: vec![make_req(
                    "The organization shall log all access and shall retain logs for 90 days",
                    20,
                )],
            },
        ],
    };

    let result = atomize_document(&doc).unwrap();

    // Section 1: 2 from compound + 1 atomic = 3
    assert_eq!(result.sections[0].requirements.len(), 3);
    // Section 2: 1 compound → 2
    assert_eq!(result.sections[1].requirements.len(), 2);
    // Section 3: 1 compound → 2
    assert_eq!(result.sections[2].requirements.len(), 2);
    // Total: 7
    assert_eq!(result.total_requirement_count(), 7);
}
