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
    let report = generate_trace_report(artifact, source)?;
    let table = format_trace_table(&report);

    match output {
        Some(path) => {
            crate::io::write_atomic(path, table.as_bytes())?;
            eprintln!("Trace report written to {}", path.display());
        }
        None => {
            print!("{table}");
        }
    }

    Ok(())
}
