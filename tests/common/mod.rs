#![allow(dead_code)]
pub mod fixture_generator;

use std::path::{Path, PathBuf};

use forge::model::{DocumentMetadata, PolicyDocument, PolicyRequirement, PolicySection};

/// Maximum file size for ingest in tests (10 MB).
pub const MAX_SIZE_BYTES: u64 = 10 * 1024 * 1024;

/// Log a skip message and return `true` if the fixture file does not exist.
///
/// Callers should `return` when this returns `true`.
pub fn skip_if_missing(path: &Path) -> bool {
    if path.exists() {
        false
    } else {
        eprintln!("Skipping test: fixture not found at {}", path.display());
        true
    }
}

pub fn make_req(text: &str, source_line: usize) -> PolicyRequirement {
    PolicyRequirement {
        stable_id: None,
        text: text.to_string(),
        source_line,
        nesting_depth: 0,
        atom_index: 0,
        parent_text: None,
        citations: vec![],
        modality: None,
        parameters: vec![],
    }
}

pub fn make_section(title: &str, requirements: Vec<PolicyRequirement>) -> PolicySection {
    PolicySection {
        title: title.to_string(),
        heading_level: 1,
        source_line: 1,
        body_text: None,
        children: vec![],
        requirements,
    }
}

pub fn make_doc(title: &str, sections: Vec<PolicySection>) -> PolicyDocument {
    PolicyDocument {
        id: "test".to_string(),
        metadata: DocumentMetadata {
            title: title.to_string(),
            version: "0.0.0".to_string(),
            author: None,
            date: None,
            source_path: PathBuf::from("test.md"),
            content_hash: None,
        },
        sections,
    }
}
