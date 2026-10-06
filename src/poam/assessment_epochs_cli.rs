//! Original-bound two-file epoch publication and optional complete report stdout.

use std::io::Write;
use std::path::Path;

use crate::ForgeError;
use crate::assessment_results::epoch_report::EpochViewFormat;
use crate::cli::{AssessmentResultsFailOn, AssessmentResultsReportFormat};

use super::assessment_epochs::{self, PreparedEpochAppend};

/// Prepare all outputs, recheck their originals, and publish a new epoch pair.
///
/// Both destination arguments are relative portable JSON filenames on the
/// request's captured parent. The report file always contains closed JSON.
/// A selected view reaches stdout only after both files have been published.
/// # Errors
/// Returns a fixed error for invalid preparation, aliases, changed originals,
/// unsupported publication, or output failure. Late failures can leave complete
/// files or partial stdout; this operation does not roll back published files.
pub(crate) fn execute(
    request: &Path,
    output: &Path,
    report: &Path,
    view_format: Option<&AssessmentResultsReportFormat>,
    fail_on: &AssessmentResultsFailOn,
) -> Result<bool, ForgeError> {
    validate_output(output)?;
    validate_output(report)?;
    if paths_alias(output, report) {
        return Err(error("epoch outputs must have distinct portable filenames"));
    }
    let prepared = if let Some(format) = view_format {
        let format = match format {
            AssessmentResultsReportFormat::Json => EpochViewFormat::Json,
            AssessmentResultsReportFormat::Text => EpochViewFormat::Text,
            AssessmentResultsReportFormat::Html => EpochViewFormat::Html,
        };
        assessment_epochs::prepare_with_view(request, output, report, Some(format))?
    } else {
        assessment_epochs::prepare(request, output, report)?
    };
    for destination in [output, report] {
        let destination = prepared.root().join(destination);
        if prepared.input_paths().any(|original| paths_alias(&destination, &original)) {
            return Err(error("epoch output aliases an original input"));
        }
    }
    publish_prepared(
        &prepared,
        output,
        report,
        fail_on,
        &mut std::io::stdout().lock(),
        |root, relative, bytes| {
            crate::authoring::output::publish_new_file(root, relative, bytes)
                .map_err(|_| error("cannot publish a new complete epoch output"))
        },
    )
}

/// Consume actual held preparation through every publication and stdout gate.
///
/// The producer owns the sealed bytes and original proof. The production caller
/// supplies the real no-replace publisher; tests can exercise late publisher or
/// writer failures while retaining the same actual proof and output buffers.
pub(super) fn publish_prepared(
    prepared: &PreparedEpochAppend,
    output: &Path,
    report: &Path,
    fail_on: &AssessmentResultsFailOn,
    writer: &mut impl Write,
    mut publisher: impl FnMut(&Path, &Path, &[u8]) -> Result<(), ForgeError>,
) -> Result<bool, ForgeError> {
    prepared
        .verify_inputs()
        .map_err(|_| error("epoch original generations changed or are unsafe"))?;
    publisher(prepared.root(), output, prepared.native_bytes())?;
    prepared
        .verify_inputs()
        .map_err(|_| error("epoch original generations changed or are unsafe"))?;
    publisher(prepared.root(), report, prepared.report_json())?;
    if let Some(view) = prepared.view_bytes() {
        prepared
            .verify_inputs()
            .map_err(|_| error("epoch original generations changed or are unsafe"))?;
        writer.write_all(view).map_err(|_| error("cannot write the complete epoch report view"))?;
        writer.flush().map_err(|_| error("cannot flush the complete epoch report view"))?;
    }
    Ok(prepared.review_required() && matches!(fail_on, AssessmentResultsFailOn::Any))
}

/// Consume the actual publisher's filename profile before any input capture.
fn validate_output(output: &Path) -> Result<(), ForgeError> {
    let text =
        output.to_str().ok_or_else(|| error("epoch output filename must be portable UTF8"))?;
    crate::authoring::output::validate_relative(text)
        .map_err(|_| error("epoch output filename must satisfy the publisher profile"))?;
    if output.components().count() != 1
        || output.extension().and_then(|part| part.to_str()) != Some("json")
    {
        return Err(error("epoch output must be a new single JSON filename"));
    }
    Ok(())
}

/// Compare complete native path components with conservative ASCII case folding.
fn paths_alias(left: &Path, right: &Path) -> bool {
    left.components().count() == right.components().count()
        && left.components().zip(right.components()).all(|(left, right)| {
            left.as_os_str()
                .as_encoded_bytes()
                .eq_ignore_ascii_case(right.as_os_str().as_encoded_bytes())
        })
}

/// Keep caller prose and private native or filesystem causes out of diagnostics.
fn error(reason: &'static str) -> ForgeError {
    ForgeError::AssessmentResultsBuild(reason.to_string())
}
