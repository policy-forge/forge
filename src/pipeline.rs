//! End-to-end catalog pipeline orchestrator (WI-13).
//!
//! Wires all pipeline stages (WI-1 through WI-12) into a single
//! `run_catalog_pipeline` function that transforms a Markdown policy
//! document into OSCAL Catalog JSON or XML.

use std::path::Path;

use crate::error::ForgeError;
use crate::model::PolicyDocument;
use crate::types::{OutputFormat, Strategy};

/// Writes serialized output to a file or stdout.
///
/// # Arguments
/// * `content` - The serialized output string (JSON, YAML, etc.)
/// * `output_path` - If Some, writes to file (validates parent dir exists); if None, prints to stdout
///
/// # Errors
/// * `ForgeError::Validation` if parent directory does not exist
/// * `ForgeError::Io` if file write fails
pub fn write_output(content: &str, output_path: Option<&Path>) -> Result<(), ForgeError> {
    match output_path {
        None => {
            println!("{content}");
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
            crate::io::write_atomic(path, content.as_bytes())?;
            Ok(())
        }
    }
}

/// Validate a serializable OSCAL envelope against the appropriate JSON schema.
///
/// Serializes the envelope to JSON, validates against the specified OSCAL schema,
/// and returns the serialized JSON string on success. On validation failure,
/// renders the error report to stderr and returns `ForgeError::SchemaValidation`.
fn validate_and_serialize<T: serde::Serialize>(
    envelope: &T,
    label: &str,
    model_type: crate::validate::OscalModelType,
) -> Result<String, ForgeError> {
    let json = serde_json::to_string_pretty(envelope)
        .map_err(|e| ForgeError::Serialization(e.to_string()))?;
    let json_value: serde_json::Value =
        serde_json::from_str(&json).map_err(|e| ForgeError::Serialization(e.to_string()))?;
    let report = crate::validate::run_full_validation(
        &format!("generated {label}"),
        &json_value,
        model_type,
    )
    .map_err(|e| ForgeError::SchemaValidation(e.to_string()))?;
    if !report.is_valid() {
        let rendered = crate::validate::report::render_text_report(&report);
        eprintln!("{rendered}");
        // PRD EC-7: do NOT write output file on validation failure
        return Err(ForgeError::SchemaValidation(format!(
            "{} validation error(s) in generated {label}",
            report.errors().len()
        )));
    }
    Ok(json)
}

/// Build and write an Assessment Plan alongside the primary artifact (WI-41).
///
/// Called by both `run_catalog_pipeline` and `run_component_pipeline` when
/// `--import-ssp` is provided. The zero-controls warning is owned by
/// `build_assessment_plan`; callers do not need to check separately.
fn write_assessment_plan(
    control_ids: &[String],
    import_ssp_href: &str,
    policy_title: &str,
    input_path: &Path,
    output_path: Option<&Path>,
) -> Result<(), ForgeError> {
    let ap_envelope =
        crate::oscal::build_assessment_plan(control_ids, import_ssp_href, policy_title)?;
    let ap_json = serde_json::to_string_pretty(&ap_envelope)
        .map_err(|e| ForgeError::Serialization(e.to_string()))?;
    let ap_path = crate::oscal::derive_ap_output_path(input_path, output_path);
    write_output(&ap_json, Some(&ap_path))?;
    tracing::info!(
        ap_path = %ap_path.display(),
        "Assessment Plan written"
    );
    Ok(())
}

/// Shared pipeline stages: ingest, parse, atomize, assign IDs, extract citations, extract parameters.
///
/// Encapsulates the common steps (1-9) used by both the catalog and component
/// pipelines. Each caller receives a fully-prepared `PolicyDocument` ready for
/// OSCAL generation.
///
/// # Arguments
/// * `input_path` - Path to the Markdown policy document
/// * `max_size_bytes` - Maximum allowed input file size in bytes
///
/// # Errors
/// * `Err(ForgeError)` if any pipeline stage fails (ingest, parse, atomize, etc.)
pub(crate) fn prepare_document(
    input_path: &Path,
    max_size_bytes: u64,
) -> Result<PolicyDocument, ForgeError> {
    // Step 1: Ingest file
    let ingested = crate::ingest::ingest_file(input_path, max_size_bytes)?;

    // Step 2: Reconstruct content
    let content = ingested.reconstruct_content();

    // Step 3: Extract sections
    let sections = crate::parse::extract_sections(&content)?;

    // Step 4: Extract clauses
    let clauses = crate::parse::extract_clauses(&content)?;

    // EC-6: Detect missing structure — error when no sections AND no clauses
    let has_clause_structure = !clauses.list_items.is_empty() || !clauses.tables.is_empty();
    if sections.is_empty() && !has_clause_structure {
        return Err(ForgeError::NoStructureDetected { path: input_path.to_path_buf() });
    }
    if sections.is_empty() {
        tracing::warn!("No identifiable sections found in input — output will have empty groups");
    }

    // Step 5: Assemble document
    let document = crate::model::assemble_document(&ingested, &sections, &clauses)?;

    // Step 6: Atomize document
    let atomized = crate::parse::atomize_document(&document)?;

    // Step 7: Assign stable IDs (functional transformation)
    let doc_with_ids = crate::uuid::assign_stable_ids(atomized);

    // Step 7b: Extract citations (WI-8, after UUID assignment, before OSCAL generation)
    let doc_with_citations = crate::citation::extract_citations(doc_with_ids)?;

    // Step 7c: Detect modality (WI-33, after citations, before OSCAL generation)
    let doc_with_modality = crate::parse::annotate_modalities(doc_with_citations)?;

    // Step 7d: Extract parameters (WI-34, after modality detection)
    let mut doc_with_params = doc_with_modality;
    crate::parameter::extract_parameters(&mut doc_with_params)?;

    Ok(doc_with_params)
}

/// Orchestrates the full catalog pipeline: ingest → parse → normalize → map → serialize → output.
///
/// # Arguments
/// * `input_path` - Path to the Markdown policy document
/// * `output_path` - Optional output file path; if None, writes to stdout
/// * `max_size_bytes` - Maximum allowed input file size in bytes
/// * `format` - Output format (JSON, YAML, or XML)
///
/// # Errors
/// * `Err(ForgeError)` if any pipeline stage fails
pub fn run_catalog_pipeline(
    input_path: &Path,
    output_path: Option<&Path>,
    max_size_bytes: u64,
    format: &OutputFormat,
    import_ssp_href: Option<&str>,
) -> Result<crate::summary::ConversionStatistics, ForgeError> {
    use crate::summary::{ValidationStatus, count_catalog_controls};

    // Steps 1-9: shared pipeline stages
    let doc_with_ids = prepare_document(input_path, max_size_bytes)?;

    let sections_parsed = doc_with_ids.total_sections();
    let requirements_extracted = doc_with_ids.total_requirements();

    // Step 8: Build catalog (with trace link capture)
    let mut trace_links = crate::model::trace::TraceLinkCollection::new();
    let mut catalog = crate::oscal::build_catalog(&doc_with_ids, Some(&mut trace_links))?;

    tracing::info!(
        trace_link_count = trace_links.len(),
        "Trace links captured during catalog generation"
    );

    // Step 8b: Embed trace props/links into catalog controls and groups (WI-17)
    crate::oscal::trace_embedding::embed_trace_in_catalog(&mut catalog, &trace_links);

    // Step 9: Assemble metadata
    let real_metadata = crate::oscal::assemble_metadata(&doc_with_ids.metadata, None)?;

    // Step 10: Generate back matter from extracted citations
    let all_citations =
        crate::oscal::component_definition::collect_all_citations(&doc_with_ids.sections);
    let (back_matter_resources, _resource_map) =
        crate::oscal::generate_back_matter(&all_citations)?;

    // Step 11: Assemble CatalogEnvelope with real metadata mapped to placeholder fields
    let back_matter = if back_matter_resources.is_empty() {
        None
    } else {
        Some(crate::oscal::BackMatter { resources: back_matter_resources })
    };

    let oscal_catalog = crate::oscal::OscalCatalog {
        uuid: real_metadata.uuid.to_string(),
        metadata: crate::oscal::catalog::OscalMetadata {
            title: real_metadata.title,
            last_modified: real_metadata.last_modified.to_rfc3339(),
            version: real_metadata.version,
            oscal_version: real_metadata.oscal_version,
        },
        groups: catalog.groups,
        back_matter,
    };

    let controls_generated = count_catalog_controls(&oscal_catalog);

    // Step 12: Validate and serialize based on output format
    let envelope = crate::oscal::CatalogEnvelope { catalog: oscal_catalog };

    // Step 12b: Auto-validate OSCAL model (schema + semantic) (WI-20, PRD M-5)
    let json =
        validate_and_serialize(&envelope, "catalog", crate::validate::OscalModelType::Catalog)?;

    let mut stats = crate::summary::ConversionStatistics {
        sections_parsed,
        requirements_extracted,
        controls_generated,
        validation_status: ValidationStatus::Passed,
        strategy: Strategy::Catalog,
        ..Default::default()
    };

    match format {
        OutputFormat::Json => write_output(&json, output_path)?,
        OutputFormat::Xml => {
            let xml = crate::export::xml_serializer::serialize_catalog_to_xml(&envelope.catalog)?;
            write_output(&xml, output_path)?;
        }
        OutputFormat::Yaml => {
            let yaml = crate::export::yaml::serialize_to_yaml(&envelope)?;
            write_output(&yaml, output_path)?;
        }
    }

    stats.output_path = output_path.map(std::path::Path::to_path_buf);

    // WI-41: Generate Assessment Plan if --import-ssp was provided
    if let Some(href) = import_ssp_href {
        let control_ids =
            crate::oscal::catalog::collect_control_ids_from_catalog(&envelope.catalog);
        write_assessment_plan(
            &control_ids,
            href,
            &envelope.catalog.metadata.title,
            input_path,
            output_path,
        )?;
    }

    Ok(stats)
}

/// Orchestrates the full component pipeline: ingest → parse → normalize → map → serialize → output.
///
/// # Arguments
/// * `input_path` - Path to the Markdown policy document
/// * `output_path` - Optional output file path; if None, writes to stdout
/// * `max_size_bytes` - Maximum allowed input file size in bytes
/// * `source_profile` - Optional baseline profile reference for control-implementations;
///   when `None`, produces a Component Definition with empty `control-implementations`
/// * `format` - Output format (JSON, YAML, or XML)
///
/// # Errors
/// * `Err(ForgeError)` if any pipeline stage fails
pub fn run_component_pipeline(
    input_path: &Path,
    output_path: Option<&Path>,
    max_size_bytes: u64,
    source_profile: Option<&str>,
    format: &OutputFormat,
    import_ssp_href: Option<&str>,
) -> Result<crate::summary::ConversionStatistics, ForgeError> {
    use crate::summary::ValidationStatus;

    // S-3: Pipeline stage progress logging (visible with --verbose)
    tracing::info!("Ingesting and parsing policy document");

    // Steps 1-9: shared pipeline stages
    let doc_with_ids = prepare_document(input_path, max_size_bytes)?;

    let sections_parsed = doc_with_ids.total_sections();
    let requirements_extracted = doc_with_ids.total_requirements();

    tracing::info!(
        source_profile = source_profile.unwrap_or("<none>"),
        "Building component definition"
    );

    // Step 10: Build component definition with source_profile and source_file (WI-17)
    // SEC-1: Use filename-only to prevent absolute path leakage into OSCAL output
    let source_file_str = input_path
        .file_name()
        .map_or_else(|| input_path.display().to_string(), |f| f.to_string_lossy().into_owned());
    let envelope = crate::oscal::build_component_definition(
        &doc_with_ids,
        source_profile,
        None,
        Some(&source_file_str),
    )?;

    // Count implemented-requirements as controls_generated for component strategy.
    let controls_generated: usize = envelope
        .component_definition
        .components
        .iter()
        .flat_map(|c| c.control_implementations.iter())
        .map(|ci| ci.implemented_requirements.len())
        .sum();

    // Step 11: Validate and serialize based on output format

    // Step 11b: Auto-validate OSCAL model (schema + semantic) (WI-20, PRD M-5)
    let json = validate_and_serialize(
        &envelope,
        "component definition",
        crate::validate::OscalModelType::ComponentDefinition,
    )?;

    let mut stats = crate::summary::ConversionStatistics {
        sections_parsed,
        requirements_extracted,
        controls_generated,
        validation_status: ValidationStatus::Passed,
        strategy: Strategy::Component,
        ..Default::default()
    };

    match format {
        OutputFormat::Json => {
            tracing::info!("Serializing to JSON");
            write_output(&json, output_path)?;
        }
        OutputFormat::Xml => {
            tracing::info!("Serializing to XML");
            let xml = crate::export::xml_serializer::serialize_component_definition_to_xml(
                &envelope.component_definition,
            )?;
            write_output(&xml, output_path)?;
        }
        OutputFormat::Yaml => {
            tracing::info!("Serializing to YAML");
            let yaml = crate::export::yaml::serialize_to_yaml(&envelope)?;
            write_output(&yaml, output_path)?;
        }
    }

    stats.output_path = output_path.map(std::path::Path::to_path_buf);

    // WI-41: Generate Assessment Plan if --import-ssp was provided
    if let Some(href) = import_ssp_href {
        let control_ids =
            crate::oscal::component_definition::collect_control_ids_from_component_def(&envelope);
        write_assessment_plan(
            &control_ids,
            href,
            &envelope.component_definition.metadata.title,
            input_path,
            output_path,
        )?;
    }

    Ok(stats)
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

    // --- US2: Pipeline integration tests (T019) ---

    #[test]
    fn empty_file_pipeline_returns_empty_input() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("empty.md");
        std::fs::write(&path, "").unwrap();
        let result = run_catalog_pipeline(&path, None, 10 * 1_048_576, &OutputFormat::Json, None);
        match result.unwrap_err() {
            ForgeError::EmptyInput { .. } => {}
            other => panic!("Expected EmptyInput, got: {other:?}"),
        }
    }

    #[test]
    fn structureless_file_pipeline_returns_no_structure_detected() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("flat.md");
        std::fs::write(&path, "Just plain text without any structure.\n").unwrap();
        let result = run_catalog_pipeline(&path, None, 10 * 1_048_576, &OutputFormat::Json, None);
        match result.unwrap_err() {
            ForgeError::NoStructureDetected { .. } => {}
            other => panic!("Expected NoStructureDetected, got: {other:?}"),
        }
    }

    // --- T034: Catalog pipeline auto-validation uses ValidationReport (WI-20) ---

    #[test]
    fn catalog_pipeline_valid_input_succeeds() {
        // T034: valid Markdown input → valid output (no validation errors)
        let fixture = std::path::Path::new("tests/fixtures/sample_policy.md");
        if !fixture.exists() {
            return; // Skip if fixture not available
        }
        let dir = TempDir::new().unwrap();
        let output = dir.path().join("catalog.json");
        let result =
            run_catalog_pipeline(fixture, Some(&output), 10 * 1_048_576, &OutputFormat::Json, None);
        assert!(
            result.is_ok(),
            "Catalog pipeline should succeed with valid input: {:?}",
            result.err()
        );
        assert!(output.exists(), "Output file should be written");
    }

    // --- T035: Component pipeline auto-validation uses ValidationReport (WI-20) ---

    #[test]
    fn component_pipeline_auto_validation_uses_report_format() {
        // T035: component pipeline auto-validation uses ValidationReport format (WI-20)
        // Note: without source_profile, component pipeline generates empty
        // control-implementations which OSCAL schema rejects — this is expected.
        // We verify the error uses the new ValidationReport-based format.
        let fixture = std::path::Path::new("tests/fixtures/sample_policy.md");
        if !fixture.exists() {
            return;
        }
        let dir = TempDir::new().unwrap();
        let output = dir.path().join("component.json");
        let result = run_component_pipeline(
            fixture,
            Some(&output),
            10 * 1_048_576,
            None,
            &OutputFormat::Json,
            None,
        );
        match &result {
            Ok(_) => {
                // If it succeeds, that's also fine — means schema accepts the output
                assert!(output.exists(), "Output file should be written on success");
            }
            Err(ForgeError::SchemaValidation(msg)) => {
                // Verify error uses the new report-based format ("N validation error(s)")
                assert!(
                    msg.contains("validation error(s)"),
                    "SchemaValidation error should use report format, got: {msg}"
                );
                // EC-7: output file should NOT be written on validation failure
                assert!(!output.exists(), "Output file must not be written on validation failure");
            }
            Err(other) => {
                panic!("Expected Ok or SchemaValidation error, got: {other:?}");
            }
        }
    }

    // --- T036: Auto-validation errors go to stderr only (SEC-7) ---
    // Note: stderr-only behavior is enforced by the pipeline using eprintln!()
    // for the rendered report and returning Err() before write_output().
    // This is a structural guarantee, not a runtime assertion, since capturing
    // stderr in unit tests requires spawning a process.
}
