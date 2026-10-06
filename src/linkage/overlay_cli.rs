//! Publish one complete fresh evidence overlay without altering its original target.

use std::path::Path;

use crate::ForgeError;

use super::overlay;

/// Prepare and recheck actual originals before one confined no-replacement publication.
///
/// All four arguments are explicit. The producer admits the selected model, output
/// parent, source generations, complete associations and preservation oracle.
/// # Errors
/// Returns fixed linkage errors for invalid or changed originals, input aliases,
/// unsupported publication and filesystem failures. A complete destination may
/// remain after a publisher durability failure; callers must never replace it.
pub(crate) fn execute(
    manifest: &Path,
    target_resource: &str,
    as_of: &str,
    output: &Path,
) -> Result<(), ForgeError> {
    let prepared = overlay::prepare(manifest, target_resource, as_of, output)?;
    let destination = prepared.root().join(output);
    if prepared.input_paths().any(|original| paths_alias(&destination, &original)) {
        return Err(ForgeError::Linkage("overlay output aliases an original input".into()));
    }
    prepared.verify_inputs().map_err(|_| {
        ForgeError::Linkage("overlay original generations changed or are unsafe".into())
    })?;
    crate::authoring::output::publish_new_file(prepared.root(), output, prepared.bytes())
        .map_err(|_| ForgeError::Linkage("cannot publish a new complete evidence overlay".into()))
}

/// Compare complete native path components using conservative ASCII case folding.
fn paths_alias(left: &Path, right: &Path) -> bool {
    left.components().count() == right.components().count()
        && left.components().zip(right.components()).all(|(left, right)| {
            left.as_os_str()
                .as_encoded_bytes()
                .eq_ignore_ascii_case(right.as_os_str().as_encoded_bytes())
        })
}
