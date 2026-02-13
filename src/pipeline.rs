//! End-to-end catalog pipeline orchestrator (WI-13).
//!
//! Wires all pipeline stages (WI-1 through WI-12) into a single
//! `run_catalog_pipeline` function that transforms a Markdown policy
//! document into OSCAL Catalog JSON.

use std::path::Path;

use crate::error::ForgeError;

/// Writes JSON output to a file or stdout.
///
/// # Arguments
/// * `json` - The serialized JSON string
/// * `output_path` - If Some, writes to file (validates parent dir exists); if None, prints to stdout
///
/// # Errors
/// * `ForgeError::Validation` if parent directory does not exist
/// * `ForgeError::Io` if file write fails
pub fn write_output(json: &str, output_path: Option<&Path>) -> Result<(), ForgeError> {
    match output_path {
        None => {
            println!("{json}");
            Ok(())
        }
        Some(path) => {
            if let Some(parent) = path.parent()
                && !parent.as_os_str().is_empty()
                && !parent.exists()
            {
                return Err(ForgeError::Validation(format!(
                    "Output directory '{}' does not exist",
                    parent.display()
                )));
            }
            std::fs::write(path, json)?;
            Ok(())
        }
    }
}

/// Orchestrates the full catalog pipeline: ingest → parse → normalize → map → serialize → output.
///
/// # Arguments
/// * `input_path` - Path to the Markdown policy document
/// * `output_path` - Optional output file path; if None, writes JSON to stdout
/// * `max_size_bytes` - Maximum allowed input file size in bytes
///
/// # Errors
/// * `Err(ForgeError)` if any pipeline stage fails
pub fn run_catalog_pipeline(
    input_path: &Path,
    output_path: Option<&Path>,
    max_size_bytes: u64,
) -> Result<(), ForgeError> {
    // Step 1: Ingest file
    let ingested = crate::ingest::ingest_file(input_path, max_size_bytes)?;

    // Step 2: Reconstruct content
    let content = ingested.reconstruct_content();

    // EC-2: Reject empty files
    if content.trim().is_empty() {
        return Err(ForgeError::Validation(
            "Input file is empty — no content to process".to_string(),
        ));
    }

    // Step 3: Extract sections
    let sections = crate::parse::extract_sections(&content)?;

    // Step 4: Extract clauses
    let clauses = crate::parse::extract_clauses(&content)?;

    // Step 5: Assemble document
    let document = crate::model::assemble_document(&ingested, &sections, &clauses)?;

    // Step 6: Atomize document
    let atomized = crate::parse::atomize_document(&document)?;

    // Step 7: Assign stable IDs (mutates)
    let mut doc_with_ids = atomized;
    crate::uuid::assign_stable_ids(&mut doc_with_ids);

    // Step 8: Build catalog
    let catalog = crate::oscal::build_catalog(&doc_with_ids)?;

    // Step 9: Assemble metadata
    let real_metadata = crate::oscal::assemble_metadata(&doc_with_ids.metadata, None)?;

    // Step 10: Generate back matter (empty citations stub per D2)
    let (back_matter_resources, _resource_map) = crate::oscal::generate_back_matter(&[])?;

    // Step 11: Assemble CatalogEnvelope with real metadata mapped to placeholder fields
    let back_matter = if back_matter_resources.is_empty() {
        None
    } else {
        Some(crate::oscal::BackMatter { resources: back_matter_resources })
    };

    let envelope = crate::oscal::CatalogEnvelope {
        catalog: crate::oscal::OscalCatalog {
            uuid: real_metadata.uuid.to_string(),
            metadata: crate::oscal::catalog::OscalMetadata {
                title: real_metadata.title,
                last_modified: real_metadata.last_modified.to_rfc3339(),
                version: real_metadata.version,
                oscal_version: real_metadata.oscal_version,
            },
            groups: catalog.groups,
            back_matter,
        },
    };

    // Step 12: Serialize to pretty JSON
    let json = serde_json::to_string_pretty(&envelope)
        .map_err(|e| ForgeError::Serialization(e.to_string()))?;

    // Step 13: Write output
    write_output(&json, output_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn write_output_none_prints_to_stdout() {
        // Should not error when writing to stdout
        let result = write_output("{}", None);
        assert!(result.is_ok());
    }

    #[test]
    fn write_output_some_creates_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("output.json");
        let result = write_output(r#"{"test": true}"#, Some(&path));
        assert!(result.is_ok());
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, r#"{"test": true}"#);
    }

    #[test]
    fn write_output_nonexistent_parent_dir_returns_error() {
        let path = std::path::Path::new("/nonexistent/dir/output.json");
        let result = write_output("{}", Some(path));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("does not exist"),
            "Expected validation error about nonexistent dir, got: {err}"
        );
    }
}
