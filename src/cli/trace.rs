use std::path::Path;

use crate::error::ForgeError;
use crate::trace::formatter::format_trace_table;
use crate::trace::generate_trace_report;

/// Execute the `forge trace` subcommand.
///
/// Validates inputs, generates report, formats table, outputs to stdout or file.
///
/// # Errors
///
/// Returns `ForgeError` if artifact reading, parsing, or output writing fails.
pub fn execute(artifact: &Path, source: &Path, output: Option<&Path>) -> Result<(), ForgeError> {
    execute_with_composition_provenance(artifact, source, output, None)
}

/// Execute trace reporting with an optional PRD 059 composition provenance bridge.
///
/// # Errors
///
/// Returns [`ForgeError`] when trace inputs, composition provenance, or output cannot be
/// validated, read, rendered, or written.
pub fn execute_with_composition_provenance(
    artifact: &Path,
    source: &Path,
    output: Option<&Path>,
    composition_provenance: Option<&Path>,
) -> Result<(), ForgeError> {
    let report = generate_trace_report(artifact, source)
        .map_err(|error| contextualize_trace_input_error(error, artifact, source))?;
    let mut table = format_trace_table(&report);
    if let Some(provenance) = composition_provenance {
        table.push_str(&crate::policy::format_composition_trace_origins(
            provenance, source, &report,
        )?);
    }

    crate::cli::output::write_output(&table, output)?;
    if let Some(path) = output {
        eprintln!("Trace report written to {}", path.display());
    }

    Ok(())
}

fn contextualize_trace_input_error(
    error: ForgeError,
    artifact: &Path,
    source: &Path,
) -> ForgeError {
    match error {
        ForgeError::Parse(detail) => ForgeError::Parse(format!(
            "Failed to generate trace report from artifact '{}' and source '{}': {detail}",
            artifact.display(),
            source.display()
        )),
        ForgeError::Io(source_error) => ForgeError::Io(std::io::Error::new(
            source_error.kind(),
            format!(
                "Failed to read trace inputs artifact '{}' and source '{}': {source_error}",
                artifact.display(),
                source.display()
            ),
        )),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_utf8_source_error_names_source_path() {
        let directory = tempfile::tempdir().unwrap();
        let artifact = directory.path().join("catalog.json");
        let source = directory.path().join("policy.md");
        std::fs::write(&artifact, r#"{"catalog": {"groups": []}}"#).unwrap();
        std::fs::write(&source, [0xff]).unwrap();

        let error = execute(&artifact, &source, None)
            .expect_err("a non-UTF-8 source must fail trace generation");

        assert!(error.to_string().contains(&source.display().to_string()), "{error}");
    }
}
