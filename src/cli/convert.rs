use std::path::Path;

use crate::ForgeError;
use crate::cli::{OutputFormat, Strategy};
use crate::ingest;

/// Convert an input file into a pretty-printed JSON document and write it to stdout.
///
/// The `input` file is ingested (subject to `max_size`) and the resulting document is
/// serialized to pretty JSON and printed to standard output.
///
/// # Parameters
///
/// - `input`: path to the file to convert.
/// - `_strategy`, `_format`, `_output`: currently unused and ignored by this implementation.
/// - `max_size`: maximum allowed input size in megabytes; values are converted to bytes.
///
/// # Errors
///
/// Returns `ForgeError` if ingestion fails or if the document cannot be serialized to JSON.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// // `execute` will ingest `example.bin` with a 10 MB limit and print JSON to stdout.
/// let _ = crate::cli::convert::execute(Path::new("example.bin"), None, &crate::cli::OutputFormat::Json, None, 10);
/// ```
pub fn execute(
    input: &Path,
    _strategy: Option<&Strategy>,
    _format: &OutputFormat,
    _output: Option<&Path>,
    max_size: u64,
) -> Result<(), ForgeError> {
    let max_size_bytes = max_size * 1024 * 1024;
    let doc = ingest::ingest_file(input, max_size_bytes)?;
    let json = serde_json::to_string_pretty(&doc).map_err(|e| ForgeError::Parse(e.to_string()))?;
    println!("{json}");
    Ok(())
}