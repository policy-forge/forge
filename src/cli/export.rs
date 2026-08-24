//! Export subcommand: converts OSCAL artifacts between JSON, XML, and YAML formats.
//!
//! Pipeline: read → detect format → deserialize → validate → serialize → write.

use std::path::Path;

use tracing::{debug, info};

use super::OutputFormat;
use crate::error::ForgeError;
use crate::oscal::catalog::CatalogEnvelope;
use crate::oscal::component_definition::ComponentDefinitionEnvelope;

/// Wrapper enum for deserialized OSCAL models during the export pipeline.
///
/// Used internally to carry either a Catalog or Component Definition
/// through deserialize → validate → serialize stages.
#[derive(Debug)]
pub enum OscalModel {
    /// An OSCAL Catalog envelope (wraps [`CatalogEnvelope`]).
    Catalog(CatalogEnvelope),
    /// An OSCAL Component Definition envelope (wraps [`ComponentDefinitionEnvelope`]).
    Component(ComponentDefinitionEnvelope),
}

/// Detect the OSCAL format of an input file from its file extension.
///
/// # Errors
/// - `ForgeError::ExportUnsupportedExtension` if extension is unrecognized
/// - `ForgeError::ExportNoExtension` if no extension is present
pub fn detect_format(path: &Path) -> Result<OutputFormat, ForgeError> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| ForgeError::ExportNoExtension { path: path.to_path_buf() })?;

    let ext_lower = ext.to_lowercase();
    match ext_lower.as_str() {
        "json" => Ok(OutputFormat::Json),
        "xml" => Ok(OutputFormat::Xml),
        "yaml" | "yml" => Ok(OutputFormat::Yaml),
        _ => Err(ForgeError::ExportUnsupportedExtension { extension: ext_lower }),
    }
}

/// Deserialize an OSCAL artifact from a string in the specified format.
///
/// Detects the OSCAL model type (Catalog vs `ComponentDefinition`) and
/// deserializes into the appropriate envelope struct.
///
/// # Errors
/// - `ForgeError::ExportInvalidOscal` if input is not a valid OSCAL artifact
/// - `ForgeError::Serialization` if format-specific parsing fails
pub fn deserialize_oscal(content: &str, format: &OutputFormat) -> Result<OscalModel, ForgeError> {
    debug!(format = ?format, "Deserializing OSCAL artifact");
    match format {
        OutputFormat::Json => deserialize_from_json(content),
        OutputFormat::Xml => deserialize_from_xml(content),
        OutputFormat::Yaml => deserialize_from_yaml_format(content),
    }
}

/// Deserialize an OSCAL artifact from JSON.
fn deserialize_from_json(content: &str) -> Result<OscalModel, ForgeError> {
    let value: serde_json::Value = serde_json::from_str(content)
        .map_err(|e| ForgeError::ExportInvalidOscal { detail: format!("Invalid JSON: {e}") })?;

    let model_type = crate::validate::detect_model_type(&value).map_err(|_| {
        ForgeError::ExportInvalidOscal {
            detail: "JSON does not contain a recognized OSCAL root key ('catalog' or 'component-definition')".to_string(),
        }
    })?;

    match model_type {
        crate::validate::OscalModelType::Catalog => {
            let envelope: CatalogEnvelope =
                serde_json::from_value(value).map_err(|e| ForgeError::ExportInvalidOscal {
                    detail: format!("Failed to parse OSCAL catalog: {e}"),
                })?;
            Ok(OscalModel::Catalog(envelope))
        }
        crate::validate::OscalModelType::ComponentDefinition => {
            let envelope: ComponentDefinitionEnvelope =
                serde_json::from_value(value).map_err(|e| ForgeError::ExportInvalidOscal {
                    detail: format!("Failed to parse OSCAL component-definition: {e}"),
                })?;
            Ok(OscalModel::Component(envelope))
        }
        crate::validate::OscalModelType::Profile => Err(ForgeError::ExportInvalidOscal {
            detail: "Export of OSCAL Profile documents is not yet supported".to_string(),
        }),
    }
}

/// Deserialize an OSCAL artifact from XML.
///
/// Peeks at the root element name to determine model type before deserializing.
fn deserialize_from_xml(content: &str) -> Result<OscalModel, ForgeError> {
    use crate::export::xml_deserializer;

    // Determine model type from root element name
    let root_element = detect_xml_root_element(content)?;
    match root_element.as_str() {
        "catalog" => {
            let envelope = xml_deserializer::deserialize_catalog_from_xml(content)?;
            Ok(OscalModel::Catalog(envelope))
        }
        "component-definition" => {
            let envelope = xml_deserializer::deserialize_component_from_xml(content)?;
            Ok(OscalModel::Component(envelope))
        }
        "profile" => Err(ForgeError::ExportInvalidOscal {
            detail: "Export of OSCAL Profile documents is not yet supported".to_string(),
        }),
        _ => Err(ForgeError::ExportInvalidOscal {
            detail: format!(
                "XML root element '<{root_element}>' is not a recognized OSCAL type. \
                 Expected '<catalog>', '<component-definition>', or '<profile>'."
            ),
        }),
    }
}

/// Extract the root element name from XML content.
fn detect_xml_root_element(content: &str) -> Result<String, ForgeError> {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    let mut reader = Reader::from_str(content);
    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e) | Event::Empty(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                return Ok(name);
            }
            Ok(Event::Eof) => {
                return Err(ForgeError::ExportInvalidOscal {
                    detail: "XML has no root element".to_string(),
                });
            }
            Err(e) => {
                return Err(ForgeError::ExportInvalidOscal { detail: format!("Invalid XML: {e}") });
            }
            _ => {} // Skip declarations, comments, DTDs, etc.
        }
    }
}

/// Deserialize an OSCAL artifact from YAML.
fn deserialize_from_yaml_format(content: &str) -> Result<OscalModel, ForgeError> {
    // Parse to serde_json::Value via YAML, then detect model type
    let value: serde_json::Value = crate::export::yaml::deserialize_from_yaml(content)?;

    let model_type = crate::validate::detect_model_type(&value).map_err(|_| {
        ForgeError::ExportInvalidOscal {
            detail: "YAML does not contain a recognized OSCAL root key ('catalog' or 'component-definition')".to_string(),
        }
    })?;

    match model_type {
        crate::validate::OscalModelType::Catalog => {
            let envelope: CatalogEnvelope =
                serde_json::from_value(value).map_err(|e| ForgeError::ExportInvalidOscal {
                    detail: format!("Failed to parse OSCAL catalog from YAML: {e}"),
                })?;
            Ok(OscalModel::Catalog(envelope))
        }
        crate::validate::OscalModelType::ComponentDefinition => {
            let envelope: ComponentDefinitionEnvelope =
                serde_json::from_value(value).map_err(|e| ForgeError::ExportInvalidOscal {
                    detail: format!("Failed to parse OSCAL component-definition from YAML: {e}"),
                })?;
            Ok(OscalModel::Component(envelope))
        }
        crate::validate::OscalModelType::Profile => Err(ForgeError::ExportInvalidOscal {
            detail: "Export of OSCAL Profile documents is not yet supported".to_string(),
        }),
    }
}

/// Serialize an OSCAL model to a string in the specified format.
///
/// # Errors
/// - `ForgeError::Serialization` if serialization fails
pub fn serialize_oscal(model: &OscalModel, format: OutputFormat) -> Result<String, ForgeError> {
    debug!(format = ?format, "Serializing OSCAL model");
    match format {
        OutputFormat::Json => {
            let result = match model {
                OscalModel::Catalog(envelope) => serde_json::to_string_pretty(envelope),
                OscalModel::Component(envelope) => serde_json::to_string_pretty(envelope),
            };
            result.map_err(|e| ForgeError::Serialization(format!("JSON serialization failed: {e}")))
        }
        OutputFormat::Xml => match model {
            OscalModel::Catalog(envelope) => {
                crate::export::xml_serializer::serialize_catalog_to_xml(&envelope.catalog)
            }
            OscalModel::Component(envelope) => {
                crate::export::xml_serializer::serialize_component_definition_to_xml(
                    &envelope.component_definition,
                )
            }
        },
        OutputFormat::Yaml => match model {
            OscalModel::Catalog(envelope) => crate::export::yaml::serialize_to_yaml(envelope),
            OscalModel::Component(envelope) => crate::export::yaml::serialize_to_yaml(envelope),
        },
    }
}

/// Validate an OSCAL model using full validation (schema + semantic checks).
///
/// Serializes the model to JSON Value, then validates using `run_full_validation()`
/// which includes both JSON schema validation and semantic checks (orphaned
/// back-matter links, missing references, etc.).
///
/// # Errors
/// - `ForgeError::SchemaValidation` if validation fails
pub fn validate_oscal_model(model: &OscalModel) -> Result<(), ForgeError> {
    debug!("Validating OSCAL model (schema + semantic)");
    let (json_value, model_type) = match model {
        OscalModel::Catalog(envelope) => {
            let value = serde_json::to_value(envelope).map_err(|e| {
                ForgeError::Serialization(format!(
                    "Failed to serialize model to JSON for validation: {e}"
                ))
            })?;
            (value, crate::validate::OscalModelType::Catalog)
        }
        OscalModel::Component(envelope) => {
            let value = serde_json::to_value(envelope).map_err(|e| {
                ForgeError::Serialization(format!(
                    "Failed to serialize model to JSON for validation: {e}"
                ))
            })?;
            (value, crate::validate::OscalModelType::ComponentDefinition)
        }
    };

    let report = crate::validate::run_full_validation("export-artifact", &json_value, model_type)
        .map_err(|e| ForgeError::SchemaValidation(e.to_string()))?;

    if report.is_valid() {
        info!("Export validation passed for {model_type}");
        Ok(())
    } else {
        let error_messages: Vec<String> =
            report.errors().iter().map(|e| e.message.clone()).collect();
        Err(ForgeError::SchemaValidation(format!(
            "{} validation error(s): {}",
            report.errors().len(),
            error_messages.join("; ")
        )))
    }
}

/// Execute the full export pipeline: read → detect → deserialize → validate → serialize → write.
///
/// # Errors
/// Returns `ForgeError` if any pipeline stage fails.
pub fn export_artifact(
    input_path: &Path,
    target_format: OutputFormat,
    output: Option<&Path>,
) -> Result<(), ForgeError> {
    info!(input = %input_path.display(), target = ?target_format, "Starting export");

    // Step 1: Check file exists
    if !input_path.exists() {
        return Err(ForgeError::FileNotFound { path: input_path.to_path_buf() });
    }

    // Step 2: Guard against oversized files before reading
    crate::io::check_file_size(input_path, crate::io::MAX_FILE_SIZE)?;

    // Step 3: Read input file (read bytes first for actionable encoding errors)
    let bytes = std::fs::read(input_path)?;
    let content = String::from_utf8(bytes).map_err(|_| ForgeError::ExportInvalidOscal {
        detail: format!(
            "File '{}' is not valid UTF-8 text. OSCAL artifacts must be UTF-8 encoded.",
            input_path.display()
        ),
    })?;

    // Step 4: Check non-empty
    if content.is_empty() {
        return Err(ForgeError::ExportEmptyInput { path: input_path.to_path_buf() });
    }

    // Step 5: Detect input format
    let source_format = detect_format(input_path)?;
    info!(source = ?source_format, target = ?target_format, "Format detection complete");

    // Step 6: Deserialize to internal model
    let model = deserialize_oscal(&content, &source_format)?;
    debug!("Deserialization complete");

    // Step 7: Validate model against OSCAL schema
    validate_oscal_model(&model)?;
    debug!("Validation passed");

    // Step 8: Serialize to target format
    let output_content = serialize_oscal(&model, target_format)?;
    debug!("Serialization complete");

    // Step 9: Write output
    crate::cli::output::write_output(&output_content, output)?;
    info!("Export complete");

    Ok(())
}

/// CLI handler for `forge export`.
///
/// # Errors
/// Returns `ForgeError` if any stage of the export pipeline fails.
pub fn execute(
    input: &Path,
    format: &OutputFormat,
    output: Option<&Path>,
) -> Result<(), ForgeError> {
    export_artifact(input, *format, output)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    // ══════════════════════════════════════════════════════
    // T004: detect_format tests
    // ══════════════════════════════════════════════════════

    #[test]
    fn detect_format_json() {
        let path = PathBuf::from("catalog.json");
        assert!(matches!(detect_format(&path).unwrap(), OutputFormat::Json));
    }

    #[test]
    fn detect_format_xml() {
        let path = PathBuf::from("catalog.xml");
        assert!(matches!(detect_format(&path).unwrap(), OutputFormat::Xml));
    }

    #[test]
    fn detect_format_yaml() {
        let path = PathBuf::from("catalog.yaml");
        assert!(matches!(detect_format(&path).unwrap(), OutputFormat::Yaml));
    }

    #[test]
    fn detect_format_yml() {
        let path = PathBuf::from("catalog.yml");
        assert!(matches!(detect_format(&path).unwrap(), OutputFormat::Yaml));
    }

    #[test]
    fn detect_format_case_insensitive() {
        assert!(matches!(detect_format(&PathBuf::from("file.JSON")).unwrap(), OutputFormat::Json));
        assert!(matches!(detect_format(&PathBuf::from("file.XML")).unwrap(), OutputFormat::Xml));
        assert!(matches!(detect_format(&PathBuf::from("file.YAML")).unwrap(), OutputFormat::Yaml));
        assert!(matches!(detect_format(&PathBuf::from("file.YML")).unwrap(), OutputFormat::Yaml));
        assert!(matches!(detect_format(&PathBuf::from("file.Json")).unwrap(), OutputFormat::Json));
    }

    #[test]
    fn detect_format_unsupported_extension() {
        let path = PathBuf::from("catalog.txt");
        let err = detect_format(&path).unwrap_err();
        assert!(matches!(err, ForgeError::ExportUnsupportedExtension { .. }));
        assert!(err.to_string().contains(".txt"));
    }

    #[test]
    fn detect_format_no_extension() {
        let path = PathBuf::from("catalog");
        let err = detect_format(&path).unwrap_err();
        assert!(matches!(err, ForgeError::ExportNoExtension { .. }));
    }

    // ══════════════════════════════════════════════════════
    // T010: deserialize_oscal JSON catalog test
    // ══════════════════════════════════════════════════════

    #[test]
    fn deserialize_json_catalog() {
        let content = include_str!("../../tests/fixtures/export/catalog.json");
        let model = deserialize_oscal(content, &OutputFormat::Json).unwrap();
        assert!(matches!(model, OscalModel::Catalog(_)));
    }

    #[test]
    fn deserialize_json_component() {
        let content = include_str!("../../tests/fixtures/export/component.json");
        let model = deserialize_oscal(content, &OutputFormat::Json).unwrap();
        assert!(matches!(model, OscalModel::Component(_)));
    }

    // ══════════════════════════════════════════════════════
    // T011: export_artifact JSON catalog → XML
    // ══════════════════════════════════════════════════════

    #[test]
    fn export_json_catalog_to_xml() {
        let content = include_str!("../../tests/fixtures/export/catalog.json");
        let model = deserialize_oscal(content, &OutputFormat::Json).unwrap();
        let xml = serialize_oscal(&model, OutputFormat::Xml).unwrap();
        assert!(xml.contains("<catalog"));
        assert!(xml.contains("xmlns"));
    }

    // ══════════════════════════════════════════════════════
    // T017: JSON component → XML
    // ══════════════════════════════════════════════════════

    #[test]
    fn export_json_component_to_xml() {
        let content = include_str!("../../tests/fixtures/export/component.json");
        let model = deserialize_oscal(content, &OutputFormat::Json).unwrap();
        let xml = serialize_oscal(&model, OutputFormat::Xml).unwrap();
        assert!(xml.contains("<component-definition"));
        assert!(xml.contains("xmlns"));
    }

    // ══════════════════════════════════════════════════════
    // T014: validate_oscal_model
    // ══════════════════════════════════════════════════════

    #[test]
    fn validate_valid_catalog_model() {
        let content = include_str!("../../tests/fixtures/export/catalog.json");
        let model = deserialize_oscal(content, &OutputFormat::Json).unwrap();
        assert!(validate_oscal_model(&model).is_ok());
    }

    #[test]
    fn validate_valid_component_model() {
        let content = include_str!("../../tests/fixtures/export/component.json");
        let model = deserialize_oscal(content, &OutputFormat::Json).unwrap();
        assert!(validate_oscal_model(&model).is_ok());
    }

    #[test]
    fn supported_patch_declarations_validate_and_are_preserved() {
        let legacy = include_str!("../../tests/fixtures/legacy/v1.2.0/catalog/catalog.json");
        for version in ["1.2.0", "1.2.1", "1.2.2", "1.2.3"] {
            let content = legacy.replace(
                "\"oscal-version\": \"1.2.0\"",
                &format!("\"oscal-version\": \"{version}\""),
            );
            let model = deserialize_oscal(&content, &OutputFormat::Json).unwrap();
            validate_oscal_model(&model)
                .unwrap_or_else(|error| panic!("{version} should be supported: {error}"));
            let xml = serialize_oscal(&model, OutputFormat::Xml).unwrap();
            assert!(
                xml.contains(&format!("<oscal-version>{version}</oscal-version>")),
                "export must preserve declaration {version}"
            );
        }
    }

    #[test]
    fn unsupported_declaration_fails_with_declared_and_schema_versions() {
        let legacy = include_str!("../../tests/fixtures/legacy/v1.2.0/catalog/catalog.json");
        let content =
            legacy.replace("\"oscal-version\": \"1.2.0\"", "\"oscal-version\": \"1.3.0\"");
        let model = deserialize_oscal(&content, &OutputFormat::Json).unwrap();
        let error = validate_oscal_model(&model).expect_err("1.3.0 must be rejected");
        let message = error.to_string();
        assert!(message.contains("unsupported OSCAL version declaration '1.3.0'"));
        assert!(message.contains("available schema baseline is 1.2.3"));
    }

    // ══════════════════════════════════════════════════════
    // T015: export_artifact end-to-end with file
    // ══════════════════════════════════════════════════════

    #[test]
    fn export_artifact_json_to_xml_stdout() {
        let input = PathBuf::from("tests/fixtures/export/catalog.json");
        let result = export_artifact(&input, OutputFormat::Xml, None);
        assert!(result.is_ok());
    }

    #[test]
    fn export_artifact_json_to_xml_file() {
        let input = PathBuf::from("tests/fixtures/export/catalog.json");
        let dir = tempfile::TempDir::new().unwrap();
        let output = dir.path().join("catalog.xml");
        let result = export_artifact(&input, OutputFormat::Xml, Some(&output));
        assert!(result.is_ok());
        let content = std::fs::read_to_string(&output).unwrap();
        assert!(content.contains("<catalog"));
    }

    // ══════════════════════════════════════════════════════
    // T018: export_artifact JSON catalog → YAML (US2)
    // ══════════════════════════════════════════════════════

    #[test]
    fn export_json_catalog_to_yaml() {
        let content = include_str!("../../tests/fixtures/export/catalog.json");
        let model = deserialize_oscal(content, &OutputFormat::Json).unwrap();
        let yaml = serialize_oscal(&model, OutputFormat::Yaml).unwrap();
        assert!(yaml.contains("catalog:"));
    }

    // ══════════════════════════════════════════════════════
    // T019: export_artifact JSON component → YAML (US2)
    // ══════════════════════════════════════════════════════

    #[test]
    fn export_json_component_to_yaml() {
        let content = include_str!("../../tests/fixtures/export/component.json");
        let model = deserialize_oscal(content, &OutputFormat::Json).unwrap();
        let yaml = serialize_oscal(&model, OutputFormat::Yaml).unwrap();
        assert!(yaml.contains("component-definition:"));
    }

    // ══════════════════════════════════════════════════════
    // T030: Invalid OSCAL JSON → ExportInvalidOscal (US4)
    // ══════════════════════════════════════════════════════

    #[test]
    fn deserialize_invalid_oscal_json() {
        let content = r#"{"some_key": "some_value"}"#;
        let err = deserialize_oscal(content, &OutputFormat::Json).unwrap_err();
        assert!(matches!(err, ForgeError::ExportInvalidOscal { .. }));
    }

    // ══════════════════════════════════════════════════════
    // T031: Empty input file → ExportEmptyInput (US4)
    // ══════════════════════════════════════════════════════

    #[test]
    fn export_artifact_empty_file() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("empty.json");
        std::fs::write(&path, "").unwrap();
        let err = export_artifact(&path, OutputFormat::Xml, None).unwrap_err();
        assert!(matches!(err, ForgeError::ExportEmptyInput { .. }));
    }

    // ══════════════════════════════════════════════════════
    // T032: Non-existent file → FileNotFound (US4)
    // ══════════════════════════════════════════════════════

    #[test]
    fn export_artifact_nonexistent_file() {
        let path = PathBuf::from("nonexistent.json");
        let err = export_artifact(&path, OutputFormat::Xml, None).unwrap_err();
        assert!(matches!(err, ForgeError::FileNotFound { .. }));
    }

    // ══════════════════════════════════════════════════════
    // T033: Unrecognized extension → ExportUnsupportedExtension (US4)
    // ══════════════════════════════════════════════════════

    #[test]
    fn export_artifact_unsupported_extension() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("catalog.txt");
        std::fs::write(&path, "content").unwrap();
        let err = export_artifact(&path, OutputFormat::Xml, None).unwrap_err();
        assert!(matches!(err, ForgeError::ExportUnsupportedExtension { .. }));
    }

    // ══════════════════════════════════════════════════════
    // T034: No file extension → ExportNoExtension (US4)
    // ══════════════════════════════════════════════════════

    #[test]
    fn export_artifact_no_extension() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("catalog");
        std::fs::write(&path, "content").unwrap();
        let err = export_artifact(&path, OutputFormat::Xml, None).unwrap_err();
        assert!(matches!(err, ForgeError::ExportNoExtension { .. }));
    }

    // ══════════════════════════════════════════════════════
    // T045: Format mismatch (EC-2) (US4)
    // ══════════════════════════════════════════════════════

    #[test]
    fn export_artifact_format_mismatch() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("catalog.json");
        // Write XML content to a .json file
        std::fs::write(&path, "<?xml version=\"1.0\"?><catalog/>").unwrap();
        let err = export_artifact(&path, OutputFormat::Xml, None).unwrap_err();
        // Should fail during deserialization with descriptive error
        assert!(
            matches!(err, ForgeError::ExportInvalidOscal { .. }),
            "Expected ExportInvalidOscal, got: {err:?}"
        );
    }

    // ══════════════════════════════════════════════════════
    // T023: XML catalog → JSON (US3)
    // ══════════════════════════════════════════════════════

    #[test]
    fn export_xml_catalog_to_json() {
        let input = PathBuf::from("tests/fixtures/export/catalog.xml");
        let dir = tempfile::TempDir::new().unwrap();
        let output = dir.path().join("catalog.json");
        let result = export_artifact(&input, OutputFormat::Json, Some(&output));
        assert!(result.is_ok(), "XML→JSON export failed: {:?}", result.unwrap_err());
        let content = std::fs::read_to_string(&output).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(json.get("catalog").is_some());
        assert_eq!(json["catalog"]["uuid"], "a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d");
    }

    // ══════════════════════════════════════════════════════
    // T024: YAML catalog → JSON (US3)
    // ══════════════════════════════════════════════════════

    #[test]
    fn export_yaml_catalog_to_json() {
        let input = PathBuf::from("tests/fixtures/export/catalog.yaml");
        let dir = tempfile::TempDir::new().unwrap();
        let output = dir.path().join("catalog.json");
        let result = export_artifact(&input, OutputFormat::Json, Some(&output));
        assert!(result.is_ok(), "YAML→JSON export failed: {:?}", result.unwrap_err());
        let content = std::fs::read_to_string(&output).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(json.get("catalog").is_some());
        assert_eq!(json["catalog"]["uuid"], "a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d");
    }

    // ══════════════════════════════════════════════════════
    // T028: Comprehensive format-pair tests (18 cases) are in
    // tests/export_format_pairs.rs to keep this file under 400 lines.
    // ══════════════════════════════════════════════════════

    // ══════════════════════════════════════════════════════
    // T038: --output file writing test
    // ══════════════════════════════════════════════════════

    #[test]
    fn export_artifact_output_to_file() {
        let dir = tempfile::TempDir::new().unwrap();
        let output = dir.path().join("output.xml");
        let input = PathBuf::from("tests/fixtures/export/catalog.json");
        let result = export_artifact(&input, OutputFormat::Xml, Some(&output));
        assert!(result.is_ok());
        assert!(output.exists());
        let content = std::fs::read_to_string(&output).unwrap();
        assert!(content.contains("<catalog"));
    }

    // ══════════════════════════════════════════════════════
    // T039: Same-format normalization (EC-3)
    // ══════════════════════════════════════════════════════

    #[test]
    fn export_same_format_json_to_json() {
        let input = PathBuf::from("tests/fixtures/export/catalog.json");
        let dir = tempfile::TempDir::new().unwrap();
        let output = dir.path().join("normalized.json");
        let result = export_artifact(&input, OutputFormat::Json, Some(&output));
        assert!(result.is_ok());
        let content = std::fs::read_to_string(&output).unwrap();
        let value: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(value.get("catalog").is_some());
    }
}
