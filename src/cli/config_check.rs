//! `forge config check` — read-only validation of project configuration.
//!
//! Validates config selection, parsing, file-safety constraints, path
//! resolution, and cross-field settings without running conversion,
//! validation, or external tools (PRD 051 M-13 / AC-17).

use std::fmt::Write as _;
use std::path::Path;

use crate::cli::output::write_output;
use crate::config;
use crate::error::ForgeError;
use crate::types::Strategy;

/// Execute `forge config check`.
///
/// Writes a diagnostic report to stdout (via [`crate::cli::output::write_output`])
/// and exits 0 on success. Validation, selection, parsing, and schema failures
/// return [`ForgeError::Config`] (exit code 3). The sole exception is an
/// unreadable working directory on the no-config discovery path, which returns
/// [`ForgeError::Io`] (exit code 1).
///
/// # Errors
///
/// Returns [`ForgeError::Config`] when selection, parsing, or validation fails,
/// and [`ForgeError::Io`] when the working directory cannot be read.
pub fn execute(explicit: Option<&Path>) -> Result<(), ForgeError> {
    let selected = config::load_selected(explicit)?;

    let mut report = String::new();

    let Some(project) = selected else {
        let cwd = std::env::current_dir().map_err(ForgeError::Io)?;
        let _ = writeln!(report, "OK: no project configuration selected");
        let _ = writeln!(report, "  discovery anchor: {}", cwd.display());
        let _ =
            writeln!(report, "  built-in defaults apply (schema-version 1 contract not in use)");
        return write_output(&report, None);
    };

    // Cross-field constraints (M-13): reject configurations that could never
    // run successfully, before printing any report.
    if let Some(convert) = &project.convert {
        config::validate_cross_field(convert)?;
    }

    let _ = writeln!(report, "OK: {}", project.describe());

    if let Some(convert) = &project.convert {
        let _ = writeln!(report, "project root: {}", project.project_root.display());
        let _ = writeln!(report, "[convert]");
        print_setting(&mut report, "strategy", convert.strategy.as_ref().map(Strategy::as_str));
        print_setting(
            &mut report,
            "format",
            convert.format.as_ref().map(|f| output_format_label(*f)),
        );
        print_setting(
            &mut report,
            "output",
            convert.output.as_ref().map(|p| relative_or_self(&project.project_root, p)).as_deref(),
        );
        print_setting(
            &mut report,
            "max-size-mb",
            convert.max_size_mb.map(|v| v.to_string()).as_deref(),
        );
        print_setting(&mut report, "jobs", convert.jobs.map(|v| v.to_string()).as_deref());
        print_setting(&mut report, "summary", convert.summary.map(|v| v.to_string()).as_deref());
        print_setting(
            &mut report,
            "source-profile",
            convert
                .source_profile
                .as_ref()
                .map(|p| relative_or_self(&project.project_root, p))
                .as_deref(),
        );
        print_setting(
            &mut report,
            "stable-id-baseline",
            convert
                .stable_id_baseline
                .as_ref()
                .map(|p| relative_or_self(&project.project_root, p))
                .as_deref(),
        );
        print_setting(&mut report, "to", convert.to.as_ref().map(crate::types::OutputType::as_str));
        print_setting(&mut report, "import-ssp", convert.import_ssp.as_deref());
    }

    if let Some(validate) = &project.validate {
        let _ = writeln!(report, "[validate]");
        print_setting(
            &mut report,
            "format",
            validate.format.as_ref().map(|f| match *f {
                crate::cli::ValidateOutputFormat::Text => "text",
                crate::cli::ValidateOutputFormat::Json => "json",
            }),
        );
        print_setting(
            &mut report,
            "output",
            validate.output.as_ref().map(|p| relative_or_self(&project.project_root, p)).as_deref(),
        );
        print_setting(
            &mut report,
            "timeout-seconds",
            validate.timeout_seconds.map(|v| v.to_string()).as_deref(),
        );
        print_setting(
            &mut report,
            "schema-type",
            validate.schema_type.as_ref().map(schema_type_label),
        );
    }

    let _ = writeln!(
        report,
        "note: checked-in configuration makes option resolution reproducible; generated \
         OSCAL artifacts still embed runtime UUID/timestamp metadata and are not yet \
         byte-reproducible"
    );

    write_output(&report, None)
}

fn print_setting(report: &mut String, key: &str, value: Option<&str>) {
    match value {
        Some(v) => {
            let _ = writeln!(report, "  {key} = {v}");
        }
        None => {
            let _ = writeln!(report, "  {key}: not set");
        }
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
