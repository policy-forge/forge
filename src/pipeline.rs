//! End-to-end catalog pipeline orchestrator (WI-13).
//!
//! Wires all pipeline stages (WI-1 through WI-12) into a single
//! `run_catalog_pipeline` function that transforms a Markdown policy
//! document into OSCAL Catalog JSON or XML.

use std::{path::Path, sync::OnceLock};

use crate::error::ForgeError;
use crate::model::PolicyDocument;
use crate::types::{OutputFormat, Strategy};

/// The serialized OSCAL artifact content produced by a pipeline run.
#[derive(Debug)]
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

/// A secondary artifact produced alongside the primary output.
#[derive(Debug)]
pub struct SecondaryOutput {
    /// Suggested filename for this artifact.
    pub filename: String,
    /// Serialized content.
    pub content: String,
}

/// Validate a serializable OSCAL envelope against the appropriate JSON schema.
///
/// Serializes the envelope to JSON, validates against the specified OSCAL schema,
/// and returns the serialized JSON string on success. On validation failure,
/// returns `ForgeError::SchemaValidation`.
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
        const MAX_REPORTED_ERRORS: usize = 5;
        let errors = report.errors();
        let details = errors
            .iter()
            .take(MAX_REPORTED_ERRORS)
            .map(|error| format!("{error:?}"))
            .collect::<Vec<_>>()
            .join("; ");
        let omitted = errors.len().saturating_sub(MAX_REPORTED_ERRORS);
        let suffix = if omitted == 0 {
            String::new()
        } else {
            format!("; {omitted} additional error(s) omitted")
        };
        return Err(ForgeError::SchemaValidation(format!(
            "{} validation error(s) in generated {label}: {details}{suffix}",
            errors.len()
        )));
    }
    Ok(json)
}

/// Compile and cache the embedded OSCAL 1.2.3 Assessment Plan schema.
fn assessment_plan_validator() -> Result<&'static jsonschema::Validator, ForgeError> {
    static ASSESSMENT_PLAN_VALIDATOR: OnceLock<jsonschema::Validator> = OnceLock::new();

    if let Some(validator) = ASSESSMENT_PLAN_VALIDATOR.get() {
        return Ok(validator);
    }

    let schema: serde_json::Value =
        serde_json::from_str(include_str!("../schemas/oscal_assessment-plan_schema.json"))
            .map_err(|error| {
                ForgeError::SchemaValidation(format!(
                    "failed to parse embedded OSCAL 1.2.3 assessment-plan schema: {error}"
                ))
            })?;
    let validator = jsonschema::validator_for(&schema).map_err(|error| {
        ForgeError::SchemaValidation(format!(
            "failed to compile embedded OSCAL 1.2.3 assessment-plan schema: {error}"
        ))
    })?;
    let _ = ASSESSMENT_PLAN_VALIDATOR.set(validator);

    ASSESSMENT_PLAN_VALIDATOR.get().ok_or_else(|| {
        ForgeError::SchemaValidation(
            "compiled assessment-plan schema validator was unavailable after initialization"
                .to_string(),
        )
    })
}

/// Validate and serialize an Assessment Plan envelope for secondary output.
///
/// Assessment Plans intentionally have no [`crate::validate::OscalModelType`]:
/// exposing one would broaden the public CLI schema options. This boundary performs
/// schema validation only because semantic validation is unavailable for this OSCAL root.
fn validate_and_serialize_assessment_plan<T: serde::Serialize>(
    envelope: &T,
) -> Result<String, ForgeError> {
    const MAX_REPORTED_ERRORS: usize = 5;

    let content = serde_json::to_string_pretty(envelope)
        .map_err(|error| ForgeError::Serialization(error.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|error| ForgeError::Serialization(error.to_string()))?;
    let errors = assessment_plan_validator()?
        .iter_errors(&json)
        .map(|error| error.to_string())
        .collect::<Vec<_>>();

    if errors.is_empty() {
        return Ok(content);
    }

    let details = errors.iter().take(MAX_REPORTED_ERRORS).cloned().collect::<Vec<_>>().join("; ");
    let omitted = errors.len().saturating_sub(MAX_REPORTED_ERRORS);
    let suffix = if omitted == 0 {
        String::new()
    } else {
        format!("; {omitted} additional error(s) omitted")
    };
    Err(ForgeError::SchemaValidation(format!(
        "{} schema validation error(s) in generated assessment plan: {details}{suffix}",
        errors.len()
    )))
}

/// Build a validated Assessment Plan secondary output.
fn assessment_plan_secondary_output(
    input_path: &Path,
    envelope: &crate::oscal::AssessmentPlanEnvelope,
) -> Result<SecondaryOutput, ForgeError> {
    Ok(SecondaryOutput {
        filename: derive_ap_filename(input_path),
        content: validate_and_serialize_assessment_plan(envelope)?,
    })
}

/// Build common conversion statistics after a successfully validated primary artifact.
fn conversion_statistics(
    sections_parsed: usize,
    requirements_extracted: usize,
    controls_generated: usize,
    strategy: Strategy,
) -> crate::summary::ConversionStatistics {
    crate::summary::ConversionStatistics {
        sections_parsed,
        requirements_extracted,
        controls_generated,
        validation_status: crate::summary::ValidationStatus::Passed,
        strategy,
        ..Default::default()
    }
}

/// Select the requested serialization format after JSON validation succeeds.
fn serialize_in_requested_format(
    format: OutputFormat,
    json: String,
    xml: impl FnOnce() -> Result<String, ForgeError>,
    yaml: impl FnOnce() -> Result<String, ForgeError>,
) -> Result<String, ForgeError> {
    match format {
        OutputFormat::Json => {
            tracing::info!("Serializing to JSON");
            Ok(json)
        }
        OutputFormat::Xml => {
            tracing::info!("Serializing to XML");
            xml()
        }
        OutputFormat::Yaml => {
            tracing::info!("Serializing to YAML");
            yaml()
        }
    }
}

/// Warn consistently when a prepared document used the metadata version fallback.
fn warn_if_default_document_version(doc: &PolicyDocument, input_path: &Path) {
    if doc.metadata.version == "0.0.0" {
        tracing::warn!(
            source = %input_path.display(),
            "document version not found; defaulting OSCAL metadata version to 0.0.0"
        );
    }
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

/// Orchestrates the full catalog pipeline: ingest → parse → normalize → map → serialize.
///
/// Returns a `PipelineOutput` containing the serialized content and metadata.
/// The caller is responsible for writing output to disk or stdout.
///
/// # Arguments
/// * `input_path` - Path to the Markdown policy document
/// * `max_size_bytes` - Maximum allowed input file size in bytes
/// * `format` - Output format (JSON, YAML, or XML)
/// * `import_ssp_href` - Optional SSP reference for assessment plan generation
///
/// # Errors
/// * `Err(ForgeError)` if any pipeline stage fails
pub fn run_catalog_pipeline(
    input_path: &Path,
    max_size_bytes: u64,
    format: &OutputFormat,
    import_ssp_href: Option<&str>,
) -> Result<PipelineOutput, ForgeError> {
    let document = prepare_document(input_path, max_size_bytes)?;
    run_catalog_pipeline_prepared(input_path, &document, *format, import_ssp_href)
}

/// Generate a catalog from a document already prepared by the shared pipeline stages.
///
/// This permits callers that need the prepared document for stable-ID comparison to
/// avoid re-ingesting and re-parsing the primary input.
pub(crate) fn run_catalog_pipeline_prepared(
    input_path: &Path,
    doc_with_ids: &PolicyDocument,
    format: OutputFormat,
    import_ssp_href: Option<&str>,
) -> Result<PipelineOutput, ForgeError> {
    use crate::summary::count_catalog_controls;
    warn_if_default_document_version(doc_with_ids, input_path);

    let sections_parsed = doc_with_ids.total_sections();
    let requirements_extracted = doc_with_ids.total_requirements();

    // Step 8: Build catalog (with trace link capture)
    let mut trace_links = crate::model::trace::TraceLinkCollection::new();
    let mut catalog = crate::oscal::build_catalog(doc_with_ids, Some(&mut trace_links))?;

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

    catalog.uuid = real_metadata.uuid.to_string();
    catalog.metadata = crate::oscal::catalog::OscalMetadata {
        title: real_metadata.title,
        last_modified: real_metadata.last_modified.to_rfc3339(),
        version: real_metadata.version,
        oscal_version: real_metadata.oscal_version,
    };
    catalog.back_matter = back_matter;
    let oscal_catalog = catalog;

    let controls_generated = count_catalog_controls(&oscal_catalog);

    // Step 12: Validate and serialize based on output format
    let envelope = crate::oscal::CatalogEnvelope { catalog: oscal_catalog };

    // Step 12b: Auto-validate OSCAL model (schema + semantic) (WI-20, PRD M-5)
    let json =
        validate_and_serialize(&envelope, "catalog", crate::validate::OscalModelType::Catalog)?;

    let stats = conversion_statistics(
        sections_parsed,
        requirements_extracted,
        controls_generated,
        Strategy::Catalog,
    );

    let content = serialize_in_requested_format(
        format,
        json,
        || crate::export::xml_serializer::serialize_catalog_to_xml(&envelope.catalog),
        || crate::export::yaml::serialize_to_yaml(&envelope),
    )?;

    // WI-41/WI-42: Build the assessment-plan secondary output when requested.
    let mut secondary_outputs = Vec::new();
    if let Some(href) = import_ssp_href {
        let control_ids =
            crate::oscal::catalog::collect_control_ids_from_catalog(&envelope.catalog);
        let mut ap_envelope = crate::oscal::build_assessment_plan(
            &control_ids,
            href,
            &envelope.catalog.metadata.title,
        )?;
        let requirements = collect_all_requirements(doc_with_ids);
        let tasks = crate::oscal::generate_assessment_tasks(&requirements);
        let subjects =
            crate::oscal::create_assessment_subjects(None, &envelope.catalog.metadata.title);
        crate::oscal::complete_assessment_plan(&mut ap_envelope, tasks, subjects)?;
        secondary_outputs.push(assessment_plan_secondary_output(input_path, &ap_envelope)?);
    }

    Ok(PipelineOutput { content, format, secondary_outputs, statistics: stats })
}

/// Derive the suggested filename for an assessment plan based on the input path.
fn derive_ap_filename(input_path: &Path) -> String {
    let ap_path = crate::oscal::derive_ap_output_path(input_path, None);
    ap_path.file_name().map_or_else(
        || "assessment-plan.json".to_owned(),
        |file| file.to_string_lossy().into_owned(),
    )
}

/// Recursively collect requirements in depth-first section order.
fn collect_all_requirements(doc: &PolicyDocument) -> Vec<&crate::model::PolicyRequirement> {
    fn collect_section<'a>(
        section: &'a crate::model::PolicySection,
        requirements: &mut Vec<&'a crate::model::PolicyRequirement>,
    ) {
        requirements.extend(section.requirements.iter());
        for child in &section.children {
            collect_section(child, requirements);
        }
    }
    let mut requirements = Vec::new();
    for section in &doc.sections {
        collect_section(section, &mut requirements);
    }
    requirements
}

/// Return a filename-only source label without exposing caller path components.
fn component_source_file_label(input_path: &Path) -> String {
    input_path.file_name().map_or_else(
        || "unknown-file".to_string(),
        |file_name| file_name.to_string_lossy().into_owned(),
    )
}

/// Orchestrates the full component pipeline: ingest → parse → normalize → map → serialize.
///
/// Returns a `PipelineOutput` containing the serialized content and metadata.
/// The caller is responsible for writing output to disk or stdout.
///
/// # Arguments
/// * `input_path` - Path to the Markdown policy document
/// * `max_size_bytes` - Maximum allowed input file size in bytes
/// * `source_profile` - Optional baseline profile reference for control-implementations;
///   when `None`, produces a Component Definition with empty `control-implementations`
/// * `format` - Output format (JSON, YAML, or XML)
/// * `import_ssp_href` - Optional SSP reference for assessment plan generation
///
/// # Errors
/// * `Err(ForgeError)` if any pipeline stage fails
pub fn run_component_pipeline(
    input_path: &Path,
    max_size_bytes: u64,
    source_profile: Option<&str>,
    format: &OutputFormat,
    import_ssp_href: Option<&str>,
) -> Result<PipelineOutput, ForgeError> {
    let document = prepare_document(input_path, max_size_bytes)?;
    run_component_pipeline_prepared(input_path, &document, source_profile, *format, import_ssp_href)
}

/// Generate a component definition from a document prepared by the shared pipeline stages.
pub(crate) fn run_component_pipeline_prepared(
    input_path: &Path,
    doc_with_ids: &PolicyDocument,
    source_profile: Option<&str>,
    format: OutputFormat,
    import_ssp_href: Option<&str>,
) -> Result<PipelineOutput, ForgeError> {
    // S-3: Pipeline stage progress logging (visible with --verbose)
    tracing::info!("Building component definition from prepared policy document");
    warn_if_default_document_version(doc_with_ids, input_path);

    let sections_parsed = doc_with_ids.total_sections();
    let requirements_extracted = doc_with_ids.total_requirements();

    tracing::info!(
        source_profile = source_profile.unwrap_or("<none>"),
        "Building component definition"
    );

    // Step 10: Build component definition with source_profile and source_file (WI-17)
    // SEC-1: Use filename-only to prevent absolute path leakage into OSCAL output
    let source_file_str = component_source_file_label(input_path);
    let envelope = crate::oscal::build_component_definition(
        doc_with_ids,
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

    let stats = conversion_statistics(
        sections_parsed,
        requirements_extracted,
        controls_generated,
        Strategy::Component,
    );

    let content = serialize_in_requested_format(
        format,
        json,
        || {
            crate::export::xml_serializer::serialize_component_definition_to_xml(
                &envelope.component_definition,
            )
        },
        || crate::export::yaml::serialize_to_yaml(&envelope),
    )?;

    // WI-41/WI-42: Build the assessment-plan secondary output when requested.
    let mut secondary_outputs = Vec::new();
    if let Some(href) = import_ssp_href {
        let control_ids =
            crate::oscal::component_definition::collect_control_ids_from_component_def(&envelope);
        let mut ap_envelope = crate::oscal::build_assessment_plan(
            &control_ids,
            href,
            &envelope.component_definition.metadata.title,
        )?;
        let requirements = collect_all_requirements(doc_with_ids);
        let tasks = crate::oscal::generate_assessment_tasks(&requirements);
        let subjects = crate::oscal::create_assessment_subjects(
            Some(&envelope.component_definition.uuid),
            &envelope.component_definition.metadata.title,
        );
        crate::oscal::complete_assessment_plan(&mut ap_envelope, tasks, subjects)?;
        secondary_outputs.push(assessment_plan_secondary_output(input_path, &ap_envelope)?);
    }

    Ok(PipelineOutput { content, format, secondary_outputs, statistics: stats })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use tempfile::TempDir;

    // --- US2: Pipeline integration tests (T019) ---

    #[test]
    fn empty_file_pipeline_returns_empty_input() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("empty.md");
        std::fs::write(&path, "").unwrap();
        let result = run_catalog_pipeline(&path, 10 * 1_048_576, &OutputFormat::Json, None);
        let ForgeError::EmptyInput { .. } = result.unwrap_err() else {
            panic!("Expected EmptyInput");
        };
    }

    #[test]
    fn structureless_file_pipeline_returns_no_structure_detected() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("flat.md");
        std::fs::write(&path, "Just plain text without any structure.\n").unwrap();
        let result = run_catalog_pipeline(&path, 10 * 1_048_576, &OutputFormat::Json, None);
        let ForgeError::NoStructureDetected { .. } = result.unwrap_err() else {
            panic!("Expected NoStructureDetected");
        };
    }

    // --- T034: Catalog pipeline auto-validation uses ValidationReport (WI-20) ---

    #[test]
    fn catalog_pipeline_valid_input_succeeds() {
        // T034: valid Markdown input → valid output (no validation errors)
        let fixture = std::path::Path::new("tests/fixtures/sample_policy.md");
        if !fixture.exists() {
            return; // Skip if fixture not available
        }
        let result = run_catalog_pipeline(fixture, 10 * 1_048_576, &OutputFormat::Json, None);
        assert!(
            result.is_ok(),
            "Catalog pipeline should succeed with valid input: {:?}",
            result.err()
        );
        // Pipeline now returns content instead of writing files
        let output = result.unwrap();
        assert!(!output.content.is_empty(), "Output content should not be empty");
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
        let result =
            run_component_pipeline(fixture, 10 * 1_048_576, None, &OutputFormat::Json, None);
        match &result {
            Ok(output) => {
                // If it succeeds, that's also fine — means schema accepts the output
                assert!(!output.content.is_empty(), "Output content should not be empty");
            }
            Err(ForgeError::SchemaValidation(msg)) => {
                // Verify error uses the new report-based format ("N validation error(s)")
                assert!(
                    msg.contains("validation error(s)"),
                    "SchemaValidation error should use report format, got: {msg}"
                );
            }
            Err(other) => {
                panic!("Expected Ok or SchemaValidation error, got: {other:?}");
            }
        }
    }

    #[test]
    fn assessment_plan_output_uses_input_derived_filename() {
        assert_eq!(derive_ap_filename(Path::new("policy.md")), "policy-assessment-plan.json");
    }

    #[test]
    fn valid_assessment_plan_is_emitted_as_secondary_output() {
        let envelope = crate::oscal::build_assessment_plan(
            &["ac-1".to_string()],
            "./ssp.json",
            "Example policy",
        )
        .expect("assessment plan should build");

        let output = assessment_plan_secondary_output(Path::new("policy.md"), &envelope)
            .expect("valid assessment plan should be emitted");

        assert_eq!(output.filename, "policy-assessment-plan.json");
        assert!(output.content.contains("\"assessment-plan\""));
    }

    #[test]
    fn schema_invalid_assessment_plan_mutation_is_rejected_before_output() {
        let mut envelope = crate::oscal::build_assessment_plan(
            &["ac-1".to_string()],
            "./ssp.json",
            "Example policy",
        )
        .expect("assessment plan should build");
        envelope.assessment_plan.uuid.clear();

        let result = assessment_plan_secondary_output(Path::new("policy.md"), &envelope);

        assert!(
            matches!(result, Err(ForgeError::SchemaValidation(_))),
            "schema-invalid assessment plans must not produce a secondary output"
        );
    }

    #[test]
    fn component_source_label_never_falls_back_to_full_path() {
        assert_eq!(component_source_file_label(Path::new("/")), "unknown-file");
        assert_eq!(component_source_file_label(Path::new("/private/policy.md")), "policy.md");
    }

    // --- T036: Auto-validation errors go to stderr only (SEC-7) ---
    // Note: stderr-only behavior is enforced by the pipeline returning Err()
    // on validation failure, preventing any output from being produced.
    // The CLI layer is responsible for rendering error messages to stderr.
}
