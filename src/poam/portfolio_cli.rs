//! Explicit paired-native portfolio reports and separate static HTML publication.
//!
//! Every plan has a supplied native artifact beside its authoring companion. Native
//! hrefs remain inert original-bundle text even when reports use a different root.
//! JSON and HTML publication are separate operations and have no shared rollback.

use std::io::Write as _;
use std::path::{Component, Path, PathBuf};

use super::{html, portfolio};
use crate::ForgeError;
use crate::cli::PoamPortfolioArgs;

/// Prepare every explicitly paired plan and emit complete JSON and optional static HTML.
///
/// All output destinations and complete rendered bounds are checked before any
/// publication. Every held original and complete source admission is rechecked
/// immediately before each output. The first output can remain when a later
/// publication fails; a valid-looking earlier output does not change the exit2.
/// # Errors
/// Refuses incomplete/unbounded pairs, invalid/stale source or native inputs,
/// unsupported assertions/dates, unsafe or aliased output paths, existing files,
/// complete rendering bounds and I/O failure. File publication fails closed on
/// platforms without the existing no-replacement publisher; stdout remains supported.
pub fn execute(args: &PoamPortfolioArgs) -> Result<bool, ForgeError> {
    if args.manifest.is_empty()
        || args.manifest.len() > portfolio::MAX_PLANS
        || args.manifest.len() != args.native.len()
    {
        return Err(error("portfolio requires 1–32 manifests and one native filename for each"));
    }
    let root = args
        .output_root
        .canonicalize()
        .map_err(|_| error("cannot resolve explicit portfolio output directory"))?;
    if !root.is_dir() {
        return Err(error("portfolio output root must be an existing directory"));
    }
    let inputs: Vec<portfolio::Input> = args
        .manifest
        .iter()
        .zip(&args.native)
        .map(|(manifest, native)| portfolio::Input {
            manifest: manifest.clone(),
            native_artifact: Some(native.clone()),
        })
        .collect();
    let prepared = portfolio::prepare_native_portfolio(&inputs, &args.as_of, args.due_soon_days)?;
    let original_paths: Vec<PathBuf> = prepared.input_paths().collect();
    validate_destinations(&root, &original_paths, args.report.as_deref(), args.html.as_deref())?;
    let html_bytes = args.html.as_ref().map(|_| html::render(prepared.report())).transpose()?;
    prepared.verify_inputs()?;
    match &args.report {
        Some(path) => publish(&root, path, prepared.json())?,
        None => std::io::stdout()
            .lock()
            .write_all(prepared.json())
            .map_err(|_| error("cannot write complete portfolio JSON report"))?,
    }
    if let (Some(path), Some(bytes)) = (&args.html, &html_bytes) {
        prepared.verify_inputs()?;
        publish(&root, path, bytes)?;
    }
    Ok(prepared.review_required())
}

/// Qualify both new files and compare full paths against every actual held original.
fn validate_destinations(
    root: &Path,
    originals: &[PathBuf],
    report: Option<&Path>,
    html: Option<&Path>,
) -> Result<(), ForgeError> {
    for (path, extension) in [(report, "json"), (html, "html")] {
        if let Some(path) = path {
            destination(root, path, extension)?;
            let complete = root.join(path);
            if originals.iter().any(|original| paths_alias(&complete, original)) {
                return Err(error(
                    "portfolio output aliases a captured authoring, native or source input",
                ));
            }
        }
    }
    if let (Some(report), Some(html)) = (report, html) {
        if paths_alias(&root.join(report), &root.join(html)) {
            return Err(error("portfolio output filenames must be distinct"));
        }
    }
    Ok(())
}

/// Compare lossless full path components with conservative ASCII case folding.
///
/// No display/lossy conversion or relative-basename comparison can erase a root
/// distinction. Existing canonical input/output roots fix the absolute placement.
fn paths_alias(left: &Path, right: &Path) -> bool {
    left.components().count() == right.components().count()
        && left.components().zip(right.components()).all(|(left, right)| {
            left.as_os_str()
                .as_encoded_bytes()
                .eq_ignore_ascii_case(right.as_os_str().as_encoded_bytes())
        })
}

/// Require one new portable ASCII filename under the explicitly resolved output root.
fn destination(root: &Path, path: &Path, extension: &str) -> Result<(), ForgeError> {
    let text = path.to_str().ok_or_else(|| error("portfolio output filename must be UTF8"))?;
    crate::authoring::output::validate_relative(text).map_err(|_| {
        error("portfolio output filename must satisfy the portable publisher profile")
    })?;
    if path.components().count() != 1
        || !matches!(path.components().next(), Some(Component::Normal(_)))
        || text.contains(['/', '\\'])
        || path.extension().and_then(std::ffi::OsStr::to_str) != Some(extension)
    {
        return Err(error("portfolio output must be one correctly suffixed filename"));
    }
    match std::fs::symlink_metadata(root.join(path)) {
        Err(cause) if cause.kind() == std::io::ErrorKind::NotFound => Ok(()),
        _ => Err(error("portfolio output already exists or cannot be qualified")),
    }
}

/// Consume the existing confined no-replacement single-file publisher without changing native hrefs.
fn publish(root: &Path, path: &Path, bytes: &[u8]) -> Result<(), ForgeError> {
    crate::authoring::output::publish_new_file(root, path, bytes).map_err(|_| {
        error("cannot publish new portfolio output; destination may exist or be unsafe")
    })
}

/// Keep invalid input/publication distinct from valid schedule action without private paths.
fn error(message: &'static str) -> ForgeError {
    ForgeError::PoamBuild(message.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Full root distinctions survive lossless comparison, while ASCII-case aliases are rejected.
    #[test]
    fn full_output_paths_keep_roots_and_case_aliases() {
        assert!(paths_alias(
            Path::new("/bundle/é/NATIVE.json"),
            Path::new("/bundle/é/native.json")
        ));
        assert!(!paths_alias(
            Path::new("/other/é/native.json"),
            Path::new("/bundle/é/native.json")
        ));
        assert!(!paths_alias(
            Path::new("/bundle/é/native.json"),
            Path::new("/bundle/e/native.json")
        ));
        assert!(!paths_alias(Path::new("native.json"), Path::new("/bundle/native.json")));
    }

    /// A missing case-variant destination still collides with actual absolute captured input metadata.
    #[test]
    fn every_output_is_preflighted_against_complete_original_paths() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let other = tempfile::tempdir().unwrap();
        let originals = vec![root.join("native.json")];
        let error = validate_destinations(
            &root,
            &originals,
            Some(Path::new("NATIVE.json")),
            Some(Path::new("fresh.html")),
        )
        .unwrap_err();
        assert!(error.to_string().contains("aliases a captured"));
        assert!(
            validate_destinations(
                &root,
                &[other.path().join("native.json")],
                Some(Path::new("native.json")),
                None
            )
            .is_ok()
        );
        assert!(
            validate_destinations(
                &root,
                &[],
                Some(Path::new("fresh.json")),
                Some(Path::new("NUL.html"))
            )
            .is_err()
        );
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 0);
    }
}
