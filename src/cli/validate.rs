use std::path::Path;
use std::time::Duration;

use tracing::info;

use crate::ForgeError;
use crate::cli::SchemaType;
use crate::cli::ValidateOutputFormat;
use crate::oscal_cli::OscalCliDetect;
use crate::oscal_cli::detector::PathDetector;
use crate::oscal_cli::invoker::ProcessInvoker;
use crate::round_trip::{
    DivergenceClass, OscalComparisonRules, RoundTripResult, compare_oscal_json,
    run_round_trip_chain, write_divergence_log,
};
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
    output: Option<&Path>,
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
    if report.is_valid() {
        let rendered = match format {
            ValidateOutputFormat::Text => {
                format!("Valid: {model_type} artifact passes all validation.\n")
            }
            ValidateOutputFormat::Json => validate::report::render_json_report(&report),
        };
        crate::cli::output::write_output(&rendered, output)?;
        Ok(())
    } else {
        let rendered = match format {
            ValidateOutputFormat::Text => validate::report::render_text_report(&report),
            ValidateOutputFormat::Json => validate::report::render_json_report(&report),
        };
        if let Some(path) = output {
            crate::cli::output::write_output(&rendered, Some(path))?;
        } else {
            eprintln!("{rendered}");
        }
        Err(ForgeError::SchemaValidation(format!(
            "{} validation error(s) in {model_type} artifact",
            report.errors().len()
        )))
    }
}

/// Execute the `forge validate --round-trip` subcommand.
///
/// Runs the oscal-cli round-trip conversion chain (JSON → XML → YAML → JSON),
/// compares the original artifact against the round-tripped result, and reports
/// any divergences.
///
/// # Errors
///
/// * `ForgeError::OscalCliNotFound` — oscal-cli not on PATH (exit 4)
/// * `ForgeError::OscalCliNotFunctional` — oscal-cli found but broken (exit 4)
/// * `ForgeError::RoundTripFailed` — unresolved divergences found (exit 1)
/// * `ForgeError::Validation` — input file issues
pub fn execute_round_trip(
    input: &Path,
    format: &ValidateOutputFormat,
    output: Option<&Path>,
    timeout_secs: u64,
    oscal_cli_path: Option<&Path>,
) -> Result<(), ForgeError> {
    // Step 1: Validate input is JSON and canonicalize (validates existence)
    match input.extension().and_then(|e| e.to_str()) {
        Some("json") => {}
        _ => {
            return Err(ForgeError::Validation(
                "Round-trip validation requires a JSON input file".to_string(),
            ));
        }
    }
    let canonical_input = input.canonicalize().map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => ForgeError::FileNotFound { path: input.to_path_buf() },
        std::io::ErrorKind::PermissionDenied => {
            ForgeError::PermissionDenied { path: input.to_path_buf() }
        }
        _ => ForgeError::Io(e),
    })?;

    // Step 2: Check file size (SEC-3)
    validate::check_file_size(&canonical_input).map_err(|e| match e {
        ValidateError::FileTooLarge { size_mb, limit_mb } => ForgeError::Validation(format!(
            "Artifact file is too large ({size_mb:.1}MB, limit: {limit_mb}MB)"
        )),
        ValidateError::FileRead { path, source } => ForgeError::Validation(format!(
            "Failed to read artifact file '{}': {source}",
            path.display()
        )),
        other => ForgeError::Validation(other.to_string()),
    })?;

    // Step 3: Read and parse original JSON
    let content = std::fs::read_to_string(&canonical_input).map_err(|e| {
        ForgeError::Validation(format!("Failed to read artifact file '{}': {e}", input.display()))
    })?;
    if content.trim().is_empty() {
        return Err(ForgeError::Validation(format!(
            "Artifact file '{}' is empty",
            input.display()
        )));
    }
    let original_json: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
        ForgeError::Validation(format!("Failed to parse '{}' as JSON: {e}", input.display()))
    })?;

    // Step 3: Detect oscal-cli
    let detector = match oscal_cli_path {
        Some(path) => PathDetector::with_path(path.to_path_buf()),
        None => PathDetector::new(),
    };
    let cli_info = detector.detect();

    if !cli_info.available {
        return Err(ForgeError::OscalCliNotFound);
    }
    if !cli_info.functional {
        return Err(ForgeError::OscalCliNotFunctional {
            path: cli_info.executable_path.unwrap_or_else(|| std::path::PathBuf::from("oscal-cli")),
            detail: "oscal-cli --version check failed (Java may be missing)".to_string(),
        });
    }

    info!(
        oscal_cli_path = %cli_info.executable_path.as_deref().unwrap_or(Path::new("unknown")).display(),
        oscal_cli_version = cli_info.version.as_deref().unwrap_or("unknown"),
        "Detected oscal-cli for round-trip validation"
    );

    let invoker = match cli_info.executable_path {
        Some(path) => ProcessInvoker::new(path),
        None => {
            return Err(ForgeError::OscalCliNotFunctional {
                path: std::path::PathBuf::from("oscal-cli"),
                detail: "oscal-cli reported as functional but no executable path provided"
                    .to_string(),
            });
        }
    };

    // Step 4: Run round-trip chain in a temp directory
    let temp_dir = tempfile::tempdir().map_err(ForgeError::Io)?;
    let timeout = Duration::from_secs(timeout_secs);

    let rt_json_path = run_round_trip_chain(&canonical_input, &invoker, temp_dir.path(), timeout)?;

    // Step 5: Parse round-tripped JSON and compare
    let rt_content = std::fs::read_to_string(&rt_json_path).map_err(ForgeError::Io)?;
    let rt_json: serde_json::Value = serde_json::from_str(&rt_content).map_err(|e| {
        ForgeError::Validation(format!(
            "round-tripped artifact parse failure for {}: {e}",
            rt_json_path.display()
        ))
    })?;

    let rules = OscalComparisonRules::default();
    let divergences = compare_oscal_json(&original_json, &rt_json, "", &rules);

    // Step 6: Build result
    let unresolved_count =
        divergences.iter().filter(|d| d.classification != DivergenceClass::Acceptable).count();
    let passed = unresolved_count == 0;

    let artifact_type = validate::detect_model_type(&original_json)
        .map_or_else(|_| "Unknown".to_string(), |m| m.to_string());

    let result =
        RoundTripResult { artifact_type, source_path: input.to_path_buf(), passed, divergences };

    // Step 7: Output results
    output_round_trip_results(&result, unresolved_count, output, format)?;

    if passed { Ok(()) } else { Err(ForgeError::RoundTripFailed(unresolved_count)) }
}

/// Write round-trip validation results to the appropriate output destination.
fn output_round_trip_results(
    result: &RoundTripResult,
    unresolved_count: usize,
    output: Option<&Path>,
    format: &ValidateOutputFormat,
) -> Result<(), ForgeError> {
    match output {
        Some(output_path) => {
            write_divergence_log(result, output_path)?;
            if result.passed {
                tracing::info!("round-trip validation passed");
            } else {
                eprintln!(
                    "FAIL: {unresolved_count} unresolved divergence(s). Details written to: {}",
                    output_path.display()
                );
            }
        }
        None => match format {
            ValidateOutputFormat::Text => {
                render_round_trip_text(result, unresolved_count)?;
            }
            ValidateOutputFormat::Json => {
                let json = serde_json::to_string_pretty(result)
                    .map_err(|e| ForgeError::Serialization(e.to_string()))?;
                crate::cli::output::write_output(&json, None)?;
            }
        },
    }
    Ok(())
}

/// Render a human-readable round-trip validation summary to stdout/stderr.
fn render_round_trip_text(
    result: &RoundTripResult,
    unresolved_count: usize,
) -> Result<(), ForgeError> {
    if result.passed {
        let mut msg = format!(
            "PASS: round-trip validation of {} artifact '{}' succeeded\n",
            result.artifact_type,
            result.source_path.display()
        );
        if !result.divergences.is_empty() {
            use std::fmt::Write as _;
            let _ =
                writeln!(msg, "  ({} acceptable divergence(s) noted)", result.divergences.len());
        }
        crate::cli::output::write_output(&msg, None)?;
    } else {
        eprintln!(
            "FAIL: round-trip validation of {} artifact '{}' — {} unresolved divergence(s)",
            result.artifact_type,
            result.source_path.display(),
            unresolved_count
        );
        for d in &result.divergences {
            let marker = match d.classification {
                DivergenceClass::ForgeFix => "FORGE-FIX",
                DivergenceClass::OscalCliDiff => "OSCAL-CLI",
                DivergenceClass::Acceptable => "ACCEPT",
            };
            eprintln!("  [{marker}] {}: {}", d.json_path, d.description);
        }
    }
    Ok(())
}
