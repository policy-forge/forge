use std::path::Path;

use serde::Serialize;

use crate::ForgeError;
use crate::cli::{OutputFormat, Strategy};
use crate::ingest;
use crate::parse;

#[derive(Serialize)]
struct ConvertOutput<'a> {
    document: &'a ingest::IngestedDocument,
    sections: Vec<parse::SectionNode>,
    policy_document: crate::model::PolicyDocument,
}

/// Execute the convert subcommand.
///
/// # Errors
///
/// Returns `ForgeError` if the conversion fails.
pub fn execute(
    input: &Path,
    _strategy: Option<&Strategy>,
    _format: &OutputFormat,
    _output: Option<&Path>,
    max_size: u64,
) -> Result<(), ForgeError> {
    let max_size_bytes = max_size
        .checked_mul(1024 * 1024)
        .ok_or_else(|| ForgeError::Validation("--max-size value is too large".to_string()))?;
    let doc = ingest::ingest_file(input, max_size_bytes)?;

    // Reconstruct content from ingested lines for section extraction
    let content = doc.reconstruct_content();
    let sections = parse::extract_sections(&content)?;
    let clauses = parse::extract_clauses(&content)?;

    let policy_doc = crate::model::assemble_document(&doc, &sections, &clauses)?;

    let section_count = policy_doc.sections.len();
    let req_count = policy_doc.total_requirements();

    let output = ConvertOutput { document: &doc, sections, policy_document: policy_doc };
    let json =
        serde_json::to_string_pretty(&output).map_err(|e| ForgeError::Parse(e.to_string()))?;
    println!("{json}");
    eprintln!("Assembled: {section_count} sections, {req_count} requirements");
    Ok(())
}
