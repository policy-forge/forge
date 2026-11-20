// Contract: End-to-End Catalog Pipeline (WI-13)
//
// Signature-only declarations of the shipped public interface. The pipeline
// returns serialized output; writing to disk/stdout is the caller's job
// (src/cli/output.rs::write_output), not a pipeline stage.

use std::path::Path;
use crate::error::ForgeError;
use crate::types::OutputFormat;

/// One additional serialized artifact produced alongside the primary output.
pub struct SecondaryOutput {
    /// Suggested filename for this artifact.
    pub filename: String,
    /// Serialized content.
    pub content: String,
}

/// Primary serialized output plus format, secondary artifacts and statistics.
pub struct PipelineOutput {
    /// Serialized content (JSON, XML, or YAML string).
    pub content: String,
    /// The format of the content.
    pub format: OutputFormat,
    /// Optional secondary artifacts (e.g., assessment plan).
    pub secondary_outputs: Vec<SecondaryOutput>,
    /// Conversion statistics (requirements extracted, controls generated, etc.).
    pub statistics: crate::summary::ConversionStatistics,
}

/// Orchestrates the full catalog pipeline: ingest → parse → enrich → map → serialize.
///
/// # Arguments
/// * `input_path` - Path to the Markdown policy document
/// * `max_size_bytes` - Maximum allowed input file size in bytes
/// * `format` - Output format (JSON, YAML, or XML)
/// * `import_ssp_href` - Optional SSP reference for assessment plan generation
///
/// # Returns
/// * `Ok(PipelineOutput)` on success; the caller writes it to disk or stdout
/// * `Err(ForgeError)` if any pipeline stage fails (all recoverable)
///
/// # Pipeline Stages
/// 1. ingest_file(input_path, max_size_bytes) → IngestedDocument
/// 2. reconstruct_content() → String
/// 3. extract_sections(&content) → Vec<SectionNode>
/// 4. extract_clauses(&content) → ExtractedContent
/// 5. assemble_document(&ingested, &sections, &clauses) → PolicyDocument
/// 6. atomize_document(&document) → PolicyDocument (new copy)
/// 7. assign_stable_ids(document) → PolicyDocument (functional transformation)
/// 7b. extract_citations(document) → PolicyDocument (WI-8)
/// 7c. annotate_modalities(document) → PolicyDocument (WI-33)
/// 7d. extract_parameters(&mut document) → () (WI-34)
/// 8. build_catalog(&document, Some(&mut trace_links)) → OscalCatalog
/// 8b. embed_trace_in_catalog(&mut catalog, &trace_links) (WI-17)
/// 9. assemble_metadata(&doc.metadata, None) → metadata::OscalMetadata
/// 10. generate_back_matter(&citations) → BackMatter (from WI-8 citations)
/// 11. Assemble CatalogEnvelope
/// 12. Validate and serialize in the requested format
pub fn run_catalog_pipeline(
    input_path: &Path,
    max_size_bytes: u64,
    format: &OutputFormat,
    import_ssp_href: Option<&str>,
) -> Result<PipelineOutput, ForgeError>;

// Output writing lives in src/cli/output.rs::write_output, outside this
// contract: it validates the parent directory and returns ForgeError::Io /
// ForgeError::Validation rather than panicking.
