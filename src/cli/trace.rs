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
    execute_with_composition_provenance(artifact, source, output, None, false)
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
    include_source_excerpts: bool,
) -> Result<(), ForgeError> {
    let report = generate_trace_report(artifact, source)
        .map_err(|error| contextualize_trace_input_error(error, artifact, source))?;
    let mut table = format_trace_table(&report);
    if let Some(provenance) = composition_provenance {
        table.push_str(&crate::policy::format_composition_trace_origins(
            provenance, source, &report,
        )?);
    }
    if include_source_excerpts {
        table.push_str(&format_source_excerpts(&report)?);
    }

    crate::cli::output::write_output(&table, output)?;
    if let Some(path) = output {
        eprintln!("Trace report written to {}", path.display());
    }

    Ok(())
}

/// M-13 opt-in bounded source excerpts for resolved trace locations. Only the
/// explicit `--include-source-excerpts` flag reaches this; the default report
/// stays IDs/hashes only. Each excerpt is one bounded source line, untrusted.
fn format_source_excerpts(
    report: &crate::trace::report::TraceReport,
) -> Result<String, ForgeError> {
    use std::fmt::Write as _;
    let source = std::fs::read_to_string(&report.source_path).map_err(|error| {
        ForgeError::Io(std::io::Error::new(
            error.kind(),
            format!(
                "Failed to read source '{}' for excerpts: {error}",
                report.source_path.display()
            ),
        ))
    })?;
    let lines: Vec<&str> = source.lines().collect();
    let mut out = String::from("\nSource excerpts (opt-in; untrusted source content):\n");
    let mut count = 0;
    for entry in &report.entries {
        let Some(trace) = &entry.trace else { continue };
        let Some(line) = trace.source_line else { continue };
        if !crate::trace::resolver::validate_line_reference(Some(line), lines.len()) {
            continue;
        }
        if count >= 100 {
            break;
        }
        count += 1;
        let raw = lines[line - 1];
        let mut end = raw.len().min(500);
        while !raw.is_char_boundary(end) {
            end -= 1;
        }
        let marker = if end < raw.len() { " …" } else { "" };
        // Untrusted source bytes must not inject terminal escapes (SEC-5).
        let excerpt = crate::sanitize::strip_control_chars(&raw[..end]);
        let _ = writeln!(out, "  {} (line {line}): {excerpt}{marker}", entry.element_id);
    }
    Ok(out)
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
    fn source_excerpts_strip_terminal_escapes() {
        use crate::trace::report::{ElementType, TraceEntry, TraceMetadata, TraceReport};
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("policy.md");
        std::fs::write(&source, "safe\u{1b}[31mred\u{7}text\n").unwrap();
        let report = TraceReport {
            artifact_path: directory.path().join("catalog.json"),
            source_path: source,
            artifact_type: crate::types::OscalModelType::Catalog,
            entries: vec![TraceEntry {
                element_id: "c-1".into(),
                element_type: ElementType::Control,
                trace: Some(TraceMetadata {
                    source_file: "policy.md".into(),
                    source_section: "Section".into(),
                    source_line: Some(1),
                }),
            }],
            source_stale: false,
            source_line_count: 1,
        };
        let rendered = format_source_excerpts(&report).unwrap();
        assert!(!rendered.contains('\u{1b}'), "ESC must not reach the terminal: {rendered:?}");
        assert!(!rendered.contains('\u{7}'), "BEL must not reach the terminal: {rendered:?}");
        assert!(rendered.contains("safe"), "visible text survives");
    }

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
