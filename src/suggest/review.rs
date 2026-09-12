//! Crate-level `forge suggest review`: turn human decisions into the record.
//!
//! FORGE never invents a reviewer, a time or a rationale. The operator supplies
//! a closed `forge.suggest-dispositions/1` document and FORGE binds it to the
//! exact bundle: the manifest must decide this bundle digest and task, every
//! record must name a suggestion in the bundle, and every record must cite the
//! digest of the content that was quarantined. What is published is that
//! document, unchanged.

use std::collections::BTreeSet;
use std::path::Path;

use crate::ForgeError;
use crate::cli::AuthorReportFormat;

use super::bundle::SuggestionsBundle;
use super::disposition::{DispositionManifest, DispositionStatus, MAX_DISPOSITIONS_BYTES};
use super::shared;

/// Artifact name of the published disposition record.
pub const DISPOSITIONS_ARTIFACT: &str = "dispositions.json";

/// Arguments for one review run.
pub struct ReviewArgs<'a> {
    /// The quarantine bundle the decisions are about.
    pub bundle: &'a Path,
    /// The operator's closed disposition document.
    pub decisions: &'a Path,
    /// New directory beneath the bundle's directory.
    pub output_dir: &'a Path,
    /// Stdout format.
    pub format: AuthorReportFormat,
}

/// Bind an operator's decisions to their bundle and publish the record.
///
/// Returns `true` while at least one suggestion is still undecided, which the
/// CLI maps to the action-required exit code.
///
/// # Errors
/// Returns an authoring error for an unreadable or invalid input, a manifest
/// that decides a different bundle or task, a record that names a suggestion
/// outside the bundle, a record whose content digest is not the quarantined
/// one, or publication failure.
pub fn execute(args: &ReviewArgs<'_>) -> Result<bool, ForgeError> {
    let bundle_bytes = crate::io::read_bounded(args.bundle, super::bundle::MAX_BUNDLE_BYTES)?;
    let bundle = SuggestionsBundle::parse(&bundle_bytes)?;
    let manifest = DispositionManifest::parse(&crate::io::read_bounded(
        args.decisions,
        MAX_DISPOSITIONS_BYTES,
    )?)?;

    let bundle_sha256 = crate::hashing::sha256_hex(&bundle_bytes);
    if manifest.bundle_id != bundle.bundle_id {
        return Err(shared::error(
            "the disposition manifest decides a different bundle identifier",
        ));
    }
    if manifest.bundle_sha256 != bundle_sha256 {
        return Err(shared::error(
            "the disposition manifest must cite the digest of exactly this bundle",
        ));
    }
    if manifest.task != bundle.task {
        return Err(shared::error(
            "the disposition manifest declares a different task than the bundle",
        ));
    }

    let quarantined: std::collections::BTreeMap<&str, &str> = bundle
        .suggestions
        .iter()
        .map(|suggestion| (suggestion.suggestion_id.as_str(), suggestion.content_sha256.as_str()))
        .collect();
    for (index, record) in manifest.records.iter().enumerate() {
        let content = quarantined.get(record.suggestion_id.as_str()).ok_or_else(|| {
            shared::error(format!(
                "dispositions.records[{index}] decides a suggestion that is not in this bundle"
            ))
        })?;
        if *content != record.original_sha256 {
            return Err(shared::error(format!(
                "dispositions.records[{index}].original_sha256 is not the quarantined content digest"
            )));
        }
    }

    let decided: BTreeSet<&str> =
        manifest.records.iter().map(|record| record.suggestion_id.as_str()).collect();
    let undecided = bundle
        .suggestions
        .iter()
        .filter(|suggestion| !decided.contains(suggestion.suggestion_id.as_str()))
        .count();

    let root = args
        .bundle
        .parent()
        .map(std::fs::canonicalize)
        .transpose()
        .map_err(|cause| shared::error(format!("cannot resolve the bundle directory: {cause}")))?
        .ok_or_else(|| shared::error("--bundle must name a file in a directory"))?;
    let manifest_json = serde_json::to_vec_pretty(&manifest).map_err(|cause| {
        shared::error(format!("cannot encode the disposition manifest: {cause}"))
    })?;
    // A record FORGE publishes must pass FORGE's own contract first.
    DispositionManifest::parse(&manifest_json)?;
    crate::authoring::output::publish(
        &root,
        args.output_dir,
        &[crate::authoring::output::OutputArtifact {
            relative_path: DISPOSITIONS_ARTIFACT.to_string(),
            bytes: manifest_json.clone(),
        }],
    )?;

    let stdout = match args.format {
        AuthorReportFormat::Json => String::from_utf8(manifest_json)
            .map_err(|cause| shared::error(format!("disposition JSON encoding: {cause}")))?,
        AuthorReportFormat::Text => {
            render_summary(&manifest, bundle.suggestions.len(), undecided, args)
        }
    };
    crate::cli::output::write_output(&stdout, None)
        .map_err(|cause| shared::error(format!("cannot write the review summary: {cause}")))?;
    Ok(undecided > 0)
}

/// The text summary for one review.
fn render_summary(
    manifest: &DispositionManifest,
    total: usize,
    undecided: usize,
    args: &ReviewArgs<'_>,
) -> String {
    use std::fmt::Write as _;
    let count = |status: DispositionStatus| {
        manifest.records.iter().filter(|record| record.status == status).count()
    };
    let mut summary = String::new();
    let _ = writeln!(summary, "reviewed {} of {total} suggestion(s)", manifest.records.len());
    let _ = writeln!(
        summary,
        "decisions: accept-as-is {} accept-edited {} reject {} expired {}",
        count(DispositionStatus::AcceptAsIs),
        count(DispositionStatus::AcceptEdited),
        count(DispositionStatus::Reject),
        count(DispositionStatus::Expired)
    );
    let _ = writeln!(summary, "undecided: {undecided}");
    let _ = writeln!(summary, "output: {}", args.output_dir.display());
    let _ = writeln!(summary, "next: forge suggest promote --bundle ...");
    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_disposition_artifact_keeps_its_name() {
        assert_eq!(DISPOSITIONS_ARTIFACT, "dispositions.json");
    }
}
