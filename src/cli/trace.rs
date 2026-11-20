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
    let source = read_excerpt_source(
        &report.source_path,
        crate::io::MAX_FILE_SIZE,
        report.source_line_count,
    )?;
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

/// Re-read the traced source for opt-in excerpts without trusting it blindly.
///
/// The report already counted the source's lines when it was built, so the read is bounded
/// by the same limit the trace itself uses, and a file whose line count has since changed is
/// refused instead of excerpting lines the report never saw. Anything that is not a regular
/// file is refused before it is opened (opening a FIFO blocks and a device never ends); a
/// symlink to a regular file is followed, exactly as the trace read itself does.
fn read_excerpt_source(
    path: &Path,
    max_bytes: u64,
    expected_lines: usize,
) -> Result<String, ForgeError> {
    use std::io::Read as _;
    let describe = |error: &std::io::Error| {
        ForgeError::Io(std::io::Error::new(
            error.kind(),
            format!("Failed to read source '{}' for excerpts: {error}", path.display()),
        ))
    };
    let not_regular = || {
        ForgeError::Io(std::io::Error::other(format!(
            "Source '{}' must be a regular file to excerpt it",
            path.display()
        )))
    };
    if !std::fs::metadata(path).map_err(|error| describe(&error))?.is_file() {
        return Err(not_regular());
    }
    let file = std::fs::File::open(path).map_err(|error| describe(&error))?;
    // Re-check on the opened handle so a swap after the path check is still caught.
    let metadata = file.metadata().map_err(|error| describe(&error))?;
    if !metadata.is_file() {
        return Err(not_regular());
    }
    let too_large = |size_bytes: u64| ForgeError::FileTooLarge {
        path: path.to_path_buf(),
        size_bytes,
        limit_bytes: max_bytes,
    };
    if metadata.len() > max_bytes {
        return Err(too_large(metadata.len()));
    }
    let mut source = String::new();
    let read = file
        .take(max_bytes.saturating_add(1))
        .read_to_string(&mut source)
        .map_err(|error| describe(&error))?;
    let read = u64::try_from(read).unwrap_or(u64::MAX);
    if read > max_bytes {
        // The file grew after it was opened; report the larger of what was seen.
        return Err(too_large(metadata.len().max(read)));
    }
    if source.lines().count() != expected_lines {
        return Err(ForgeError::Io(std::io::Error::other(format!(
            "Source '{}' changed since the trace report was generated; refusing to excerpt it",
            path.display()
        ))));
    }
    Ok(source)
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

    /// Excerpts re-read the source only within the trace bound and only if it still matches the report.
    #[test]
    fn excerpt_source_is_bounded_and_must_match_the_traced_line_count() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("policy.md");
        std::fs::write(&source, "one\ntwo\nthree\n").unwrap();

        assert_eq!(read_excerpt_source(&source, 1024, 3).unwrap(), "one\ntwo\nthree\n");
        // Exactly at the byte bound is accepted; one byte under the file size is refused.
        assert!(read_excerpt_source(&source, 14, 3).is_ok());
        let too_large = read_excerpt_source(&source, 13, 3).unwrap_err();
        // The error reports the file's real size, not the capped number of bytes read.
        assert!(
            matches!(too_large, ForgeError::FileTooLarge { size_bytes: 14, limit_bytes: 13, .. }),
            "{too_large}"
        );
        // A source that gained or lost lines since the report was built is refused.
        for expected in [2, 4] {
            let changed = read_excerpt_source(&source, 1024, expected).unwrap_err();
            assert!(changed.to_string().contains("changed since the trace report"), "{changed}");
        }
        // Anything but a regular file is refused before it is opened.
        let directory_error = read_excerpt_source(directory.path(), 1024, 0).unwrap_err();
        assert!(directory_error.to_string().contains("regular file"), "{directory_error}");
        #[cfg(unix)]
        {
            let fifo = directory.path().join("pipe.md");
            let path = std::ffi::CString::new(fifo.to_str().unwrap()).unwrap();
            // SAFETY: `path` is a valid NUL-terminated path and mkfifo only creates a node.
            #[allow(unsafe_code)]
            let created = unsafe { libc::mkfifo(path.as_ptr(), 0o600) };
            assert_eq!(created, 0);
            // A FIFO would block `open` forever with no writer; it must be refused up front.
            let fifo_error = read_excerpt_source(&fifo, 1024, 0).unwrap_err();
            assert!(fifo_error.to_string().contains("regular file"), "{fifo_error}");
        }
        // A missing source names the path.
        let missing = directory.path().join("missing.md");
        let error = read_excerpt_source(&missing, 1024, 0).unwrap_err();
        assert!(error.to_string().contains(&missing.display().to_string()), "{error}");
    }
}
