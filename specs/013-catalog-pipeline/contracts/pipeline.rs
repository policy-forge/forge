// Contract: End-to-End Catalog Pipeline (WI-13)
//
// These signatures define the public interface for the pipeline orchestrator.
// Implementation must match these signatures exactly.

use std::path::Path;
use crate::error::ForgeError;

/// Orchestrates the full catalog pipeline: ingest → parse → normalize → map → serialize → output.
///
/// # Arguments
/// * `input_path` - Path to the Markdown policy document
/// * `output_path` - Optional output file path; if None, writes JSON to stdout
/// * `max_size_bytes` - Maximum allowed input file size in bytes
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(ForgeError)` if any pipeline stage fails
///
/// # Pipeline Stages
/// 1. ingest_file(input_path, max_size_bytes) → IngestedDocument
/// 2. reconstruct_content() → String
/// 3. extract_sections(&content) → Vec<SectionNode>
/// 4. extract_clauses(&content) → ExtractedContent
/// 5. assemble_document(&ingested, &sections, &clauses) → PolicyDocument
/// 6. atomize_document(&document) → PolicyDocument (new copy)
/// 7. assign_stable_ids(&mut document) → () (mutates)
/// 8. build_catalog(&document) → OscalCatalog
/// 9. assemble_metadata(&doc.metadata, None) → metadata::OscalMetadata
/// 10. generate_back_matter(&[]) → BackMatter (empty, WI-8 stub)
/// 11. Assemble CatalogEnvelope with real metadata mapped to placeholder fields
/// 12. serde_json::to_string_pretty(&envelope) → String
/// 13. write_output(&json, output_path)
pub fn run_catalog_pipeline(
    input_path: &Path,
    output_path: Option<&Path>,
    max_size_bytes: u64,
) -> Result<(), ForgeError> {
    todo!("See src/pipeline.rs for implementation")
}

/// Writes JSON output to a file or stdout.
///
/// # Arguments
/// * `json` - The serialized JSON string
/// * `output_path` - If Some, writes to file (validates parent dir exists); if None, prints to stdout
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(ForgeError::Io)` if file write fails
/// * `Err(ForgeError::Validation)` if parent directory does not exist
pub fn write_output(json: &str, output_path: Option<&Path>) -> Result<(), ForgeError> {
    todo!("See src/pipeline.rs for implementation")
}
