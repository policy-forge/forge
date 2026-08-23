//! `forge config check` — read-only validation of project configuration.
//!
//! Validates config selection, parsing, file-safety constraints, path
//! resolution, and cross-field settings without running conversion,
//! validation, or external tools (PRD 051 M-13 / AC-17).

use std::path::Path;

use crate::config;
use crate::error::ForgeError;
use crate::types::Strategy;

/// Execute `forge config check [--config <path>]`.
///
/// Prints a diagnostic report to stdout and exits 0 on success; failures
/// return [`ForgeError::Config`] (exit code 3) without side effects.
///
/// # Errors
///
/// Returns [`ForgeError::Config`] when selection, parsing, or validation fails.
pub fn execute(explicit: Option<&Path>) -> Result<(), ForgeError> {
    let selected = config::load_selected(explicit)?;

    let Some(project) = selected else {
        println!("OK: no project configuration selected");
        println!(
            "  discovery anchor: {}",
            std::env::current_dir().map_err(ForgeError::Io)?.display()
        );
        println!("  built-in defaults apply (schema-version 1 contract not in use)");
        return Ok(());
    };

    // Cross-field constraints (M-13): reject configurations that could never
    // run successfully, before printing any report.
    if let Some(convert) = &project.convert {
        config::validate_cross_field(convert)?;
    }

    println!("OK: {}", project.describe());

    if let Some(convert) = &project.convert {
        println!("project root: {}", project.project_root.display());
        println!("[convert]");
        print_setting("strategy", convert.strategy.as_ref().map(Strategy::as_str));
        print_setting("format", convert.format.as_ref().map(|f| output_format_label(*f)));
        print_setting(
            "output",
            convert.output.as_ref().map(|p| relative_or_self(&project.project_root, p)).as_deref(),
        );
        print_setting("max-size-mb", convert.max_size_mb.map(|v| v.to_string()).as_deref());
        print_setting("jobs", convert.jobs.map(|v| v.to_string()).as_deref());
        print_setting("summary", convert.summary.map(|v| v.to_string()).as_deref());
        print_setting(
            "source-profile",
            convert
                .source_profile
                .as_ref()
                .map(|p| relative_or_self(&project.project_root, p))
                .as_deref(),
        );
        print_setting(
            "stable-id-baseline",
            convert
                .stable_id_baseline
                .as_ref()
                .map(|p| relative_or_self(&project.project_root, p))
                .as_deref(),
        );
        print_setting("to", convert.to.as_ref().map(crate::types::OutputType::as_str));
        print_setting("import-ssp", convert.import_ssp.as_deref());
    }

    if let Some(validate) = &project.validate {
        println!("[validate]");
        print_setting(
            "format",
            validate.format.as_ref().map(|f| match *f {
                crate::cli::ValidateOutputFormat::Text => "text",
                crate::cli::ValidateOutputFormat::Json => "json",
            }),
        );
        print_setting(
            "output",
            validate.output.as_ref().map(|p| relative_or_self(&project.project_root, p)).as_deref(),
        );
        print_setting(
            "timeout-seconds",
            validate.timeout_seconds.map(|v| v.to_string()).as_deref(),
        );
        print_setting("schema-type", validate.schema_type.as_ref().map(schema_type_label));
    }

    println!(
        "note: checked-in configuration makes option resolution reproducible; generated \
         OSCAL artifacts still embed runtime UUID/timestamp metadata and are not yet \
         byte-reproducible"
    );

    Ok(())
}

fn print_setting(key: &str, value: Option<&str>) {
    match value {
        Some(v) => println!("  {key} = {v}"),
        None => println!("  {key}: not set"),
    }
}

fn output_format_label(format: crate::types::OutputFormat) -> &'static str {
    match format {
        crate::types::OutputFormat::Json => "json",
        crate::types::OutputFormat::Xml => "xml",
        crate::types::OutputFormat::Yaml => "yaml",
    }
}

fn schema_type_label(schema_type: &crate::cli::SchemaType) -> &'static str {
    match schema_type {
        crate::cli::SchemaType::Catalog => "catalog",
        crate::cli::SchemaType::ComponentDefinition => "component-definition",
    }
}

/// Prefer project-root-relative display for diagnostics.
fn relative_or_self(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map_or_else(|_| path.display().to_string(), |rel| rel.display().to_string())
}
