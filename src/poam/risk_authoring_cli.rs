//! Complete caller-authored risk workflow output after actual original revalidation.

use std::io::Write as _;
use std::path::Path;

use crate::ForgeError;

use super::risk_authoring;

/// Prepare the complete workflow and recheck actual originals before output.
///
/// The optional destination is a new single JSON filename on the original source
/// base. Without it, write one fully prepared bounded JSON document plus LF.
/// Caller review assertions authenticate no actor or remediation authority.
/// # Errors
/// Returns fixed POA&M errors for invalid inputs, aliases, changed originals or
/// output failures. An OS stdout error may leave a partial stream; a publisher
/// durability error may leave a complete file. Either returns invalid exit 2.
pub(crate) fn execute(
    scaffold: &Path,
    authoring: &Path,
    as_of: &str,
    output: Option<&Path>,
) -> Result<(), ForgeError> {
    if let Some(output) = output {
        validate_output(output)?;
    }
    let prepared = risk_authoring::prepare(scaffold, authoring, as_of, output)?;
    if let Some(output) = output {
        let destination = prepared.root().join(output);
        if prepared.input_paths().any(|original| paths_alias(&destination, &original)) {
            return Err(error("risk authoring output aliases an original input"));
        }
    }
    prepared
        .verify_inputs()
        .map_err(|_| error("risk authoring original generations changed or are unsafe"))?;
    if let Some(output) = output {
        crate::authoring::output::publish_new_file(prepared.root(), output, prepared.bytes())
            .map_err(|_| error("cannot publish a new complete risk workflow"))
    } else {
        std::io::stdout()
            .lock()
            .write_all(prepared.bytes())
            .map_err(|_| error("cannot write the complete risk workflow to stdout"))
    }
}

/// Consume the actual publisher's portable filename rule before any original capture.
fn validate_output(output: &Path) -> Result<(), ForgeError> {
    let text =
        output.to_str().ok_or_else(|| error("risk output filename must be portable UTF8"))?;
    crate::authoring::output::validate_relative(text)
        .map_err(|_| error("risk output filename must satisfy the publisher profile"))?;
    if output.components().count() != 1
        || output.extension().and_then(|part| part.to_str()) != Some("json")
    {
        return Err(error("risk output must be a new single JSON filename"));
    }
    Ok(())
}

/// Compare all native path components using conservative ASCII case folding.
fn paths_alias(left: &Path, right: &Path) -> bool {
    left.components().count() == right.components().count()
        && left.components().zip(right.components()).all(|(left, right)| {
            left.as_os_str()
                .as_encoded_bytes()
                .eq_ignore_ascii_case(right.as_os_str().as_encoded_bytes())
        })
}

/// Keep caller prose and private filesystem/native causes out of diagnostics.
fn error(reason: &'static str) -> ForgeError {
    ForgeError::PoamBuild(reason.to_string())
}
