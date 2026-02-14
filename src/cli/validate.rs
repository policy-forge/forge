use std::path::Path;

use crate::ForgeError;
use crate::cli::{SchemaType, ValidateOutputFormat};
use crate::validate::{self, OscalModelType, ValidateError};

/// Convert CLI `SchemaType` to validation module `OscalModelType`.
fn schema_type_to_model_type(schema_type: &SchemaType) -> OscalModelType {
    match schema_type {
        SchemaType::Catalog => OscalModelType::Catalog,
        SchemaType::ComponentDefinition => OscalModelType::ComponentDefinition,
    }
}

/// Execute the validate subcommand.
///
/// Uses `run_full_validation()` for enhanced error reporting (WI-20).
/// On valid: prints "Valid" to stdout + exit 0.
/// On invalid: renders report to stderr + returns error (exit non-zero).
///
/// # Errors
///
/// Returns `ForgeError` if the validation fails.
pub fn execute(
    input: &Path,
    schema_type: Option<&SchemaType>,
    format: &ValidateOutputFormat,
) -> Result<(), ForgeError> {
    // Step 1: Check file size (SEC-3)
    validate::check_file_size(input).map_err(|e| match e {
        ValidateError::FileTooLarge { size_mb, limit_mb } => ForgeError::Validation(format!(
            "Artifact file is too large ({size_mb:.1}MB, limit: {limit_mb}MB)"
        )),
        ValidateError::FileRead { path, source } => ForgeError::Validation(format!(
            "Failed to read artifact file '{}': {source}",
            path.display()
        )),
        other => ForgeError::Validation(other.to_string()),
    })?;

    // Step 2: Read file
    let content = std::fs::read_to_string(input).map_err(|e| {
        ForgeError::Validation(format!("Failed to read artifact file '{}': {e}", input.display()))
    })?;

    // Step 3: Check empty file (SEC-5)
    if content.trim().is_empty() {
        return Err(ForgeError::Validation(format!(
            "Artifact file '{}' is empty",
            input.display()
        )));
    }

    // Step 4: Parse JSON (SEC-4)
    let json: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
        ForgeError::Validation(format!("Failed to parse '{}' as JSON: {e}", input.display()))
    })?;

    // Step 5: Determine model type (auto-detect or override)
    let model_type = match schema_type {
        Some(st) => schema_type_to_model_type(st),
        None => {
            validate::detect_model_type(&json).map_err(|e| ForgeError::Validation(e.to_string()))?
        }
    };

    // Step 6: Run full validation (schema + semantic) via WI-20 orchestrator
    let artifact_path = input.display().to_string();
    let report = validate::run_full_validation(&artifact_path, &json, model_type)
        .map_err(|e| ForgeError::SchemaValidation(e.to_string()))?;

    // Step 7: Render report and exit
    if report.is_valid {
        println!("Valid: {model_type} artifact passes all validation.");
        Ok(())
    } else {
        let rendered = match format {
            ValidateOutputFormat::Text => validate::report::render_text_report(&report),
            ValidateOutputFormat::Json => validate::report::render_json_report(&report),
        };
        eprintln!("{rendered}");
        Err(ForgeError::SchemaValidation(format!(
            "{} validation error(s) in {model_type} artifact",
            report.errors.len()
        )))
    }
}
