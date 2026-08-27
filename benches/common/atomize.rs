//! Atomization benchmark fixture construction.

use forge::model::PolicyRequirement;

/// Build an un-enriched requirement matching normal parser output.
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
        parameters_extracted: false,
    }
}
