use std::path::Path;
use std::process;

use crate::ForgeError;
use crate::cli::SchemaType;
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
/// # Errors
///
/// Returns `ForgeError` if the validation fails.
pub fn execute(input: &Path, schema_type: Option<&SchemaType>) -> Result<(), ForgeError> {
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

    // Step 6: Validate artifact
    let result = validate::validate_artifact(&json, model_type)
        .map_err(|e| ForgeError::SchemaValidation(e.to_string()))?;

    // Step 7: Format output and exit
    if result.is_valid {
        println!("Valid: {model_type} artifact passes schema validation.");
        Ok(())
    } else {
        eprintln!("Invalid: {} error(s) found in {model_type} artifact:", result.errors.len());
        for (i, error) in result.errors.iter().enumerate() {
            let path_info =
                error.instance_path.as_deref().map_or(String::new(), |p| format!(" at {p}"));
            eprintln!("  {}: {}{path_info}", i + 1, error.message);
        }
        process::exit(1);
    }
}
