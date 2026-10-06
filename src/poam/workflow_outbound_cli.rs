//! Confined local handoff from one explicit native/authoring pair and held selection.
//!
//! Valid output is a complete local request with no remote planning or mutation.

use std::io::Write as _;
use std::path::{Component, Path, PathBuf};

use super::{manifest, workflow_outbound};
use crate::ForgeError;
use crate::cli::PoamOutboundArgs;

/// Held real selection bytes and identity, independent of parsed operation metadata.
struct SelectionOriginal {
    /// Resolved original bundle directory, used only for actual file revalidation.
    root: PathBuf,
    /// One explicit confined JSON filename beside the authoring manifest.
    relative: PathBuf,
    /// Every original selection byte, including whitespace and final newline.
    bytes: Vec<u8>,
    /// Actual platform file identity from the existing confined single-link reader.
    identity: (u64, u64),
}

impl SelectionOriginal {
    /// Reject changed bytes, replaced identity or an unsafe original immediately before output.
    fn verify(&self) -> Result<(), ForgeError> {
        let (bytes, identity) = crate::linkage::read_confined_local_file(
            &self.root,
            &self.relative,
            manifest::MAX_MANIFEST_BYTES,
        )
        .map_err(|_| error("outbound selection is missing or unsafe at recheck"))?;
        if bytes != self.bytes || identity != self.identity {
            return Err(error("outbound selection original generation changed"));
        }
        Ok(())
    }
}

/// Emit one complete local handoff after validating and rechecking all eight actual originals.
///
/// A valid handoff returns success regardless of schedule action. Explicit intent
/// never authorizes a remote target, dry-run, apply or native closure. File output
/// uses the existing platform-qualified no-replacement publisher; stdout is portable.
/// # Errors
/// Refuses invalid or stale originals, unsupported native/workflow/selection data,
/// unsafe or aliased destinations, existing outputs and publication/write failure.
pub fn execute(args: &PoamOutboundArgs) -> Result<(), ForgeError> {
    let root = args
        .output_root
        .canonicalize()
        .map_err(|_| error("cannot resolve explicit outbound output directory"))?;
    if !root.is_dir() {
        return Err(error("outbound output root must be an existing directory"));
    }
    let selection = capture_selection(&args.manifest, &args.selection)?;
    let manifest_name = args
        .manifest
        .file_name()
        .ok_or_else(|| error("outbound authoring filename is required"))?;
    // Use the same resolved bundle for selection, authoring and native capture.
    let captured_manifest = selection.root.join(manifest_name);
    let prepared = workflow_outbound::prepare(
        &captured_manifest,
        &args.native,
        &selection.bytes,
        &args.as_of,
        args.due_soon_days,
    )?;
    prepared.reject_input_aliases(&[selection.identity])?;
    if let Some(path) = &args.report {
        destination(&root, path)?;
        let complete = root.join(path);
        if prepared
            .input_paths()
            .chain(std::iter::once(selection.root.join(&selection.relative)))
            .any(|original| paths_alias(&complete, &original))
        {
            return Err(error("outbound output aliases an actual captured input"));
        }
    }
    selection.verify()?;
    prepared.verify_inputs()?;
    match &args.report {
        Some(path) => crate::authoring::output::publish_new_file(&root, path, prepared.report())
            .map_err(|_| {
                error("cannot publish new outbound output; destination may exist or be unsafe")
            }),
        None => std::io::stdout()
            .lock()
            .write_all(prepared.report())
            .map_err(|_| error("cannot write complete outbound JSON report")),
    }
}

/// Hold every original selection byte beside the explicit manifest before native capture.
fn capture_selection(
    manifest_path: &Path,
    relative: &Path,
) -> Result<SelectionOriginal, ForgeError> {
    if relative.components().count() != 1
        || !matches!(relative.components().next(), Some(Component::Normal(_)))
        || relative.extension().and_then(std::ffi::OsStr::to_str) != Some("json")
        || relative.to_str().is_none_or(|text| text.contains(['\\', ':']))
    {
        return Err(error(
            "outbound selection must be one local JSON filename beside its manifest",
        ));
    }
    let parent = manifest_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let root =
        parent.canonicalize().map_err(|_| error("outbound authoring directory is unavailable"))?;
    let (bytes, identity) =
        crate::linkage::read_confined_local_file(&root, relative, manifest::MAX_MANIFEST_BYTES)
            .map_err(|_| error("actual outbound selection is missing, unsafe or unbounded"))?;
    // The consumed preparation parses these exact bytes before native source capture.
    Ok(SelectionOriginal { root, relative: relative.into(), bytes, identity })
}

/// Require one new portable JSON filename under the explicit existing output root.
fn destination(root: &Path, path: &Path) -> Result<(), ForgeError> {
    let text = path.to_str().ok_or_else(|| error("outbound output filename must be UTF8"))?;
    crate::authoring::output::validate_relative(text).map_err(|_| {
        error("outbound output filename must satisfy the portable publisher profile")
    })?;
    if path.components().count() != 1
        || !matches!(path.components().next(), Some(Component::Normal(_)))
        || text.contains(['/', '\\'])
        || path.extension().and_then(std::ffi::OsStr::to_str) != Some("json")
    {
        return Err(error("outbound output must be one correctly suffixed JSON filename"));
    }
    match std::fs::symlink_metadata(root.join(path)) {
        Err(cause) if cause.kind() == std::io::ErrorKind::NotFound => Ok(()),
        _ => Err(error("outbound output already exists or cannot be qualified")),
    }
}

/// Compare complete lossless path components while conservatively refusing ASCII-case aliases.
fn paths_alias(left: &Path, right: &Path) -> bool {
    left.components().count() == right.components().count()
        && left.components().zip(right.components()).all(|(left, right)| {
            left.as_os_str()
                .as_encoded_bytes()
                .eq_ignore_ascii_case(right.as_os_str().as_encoded_bytes())
        })
}

/// Keep local handoff refusal at exit2 without private paths or captured content.
fn error(message: &'static str) -> ForgeError {
    ForgeError::PoamBuild(message.into())
}

#[cfg(test)]
mod tests {
    //! Real selection-file generation controls confer no native or remote authority.
    use super::*;

    /// Capture a real closed selection without constructing a native workflow proof.
    fn selected() -> (tempfile::TempDir, SelectionOriginal) {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("selected.json"),
            br#"{"schema_version":"forge.poam-outbound-selection/1","operations":[{"item_key":"work","intent":"create"}]}"#).unwrap();
        let captured =
            capture_selection(&directory.path().join("authoring.json"), Path::new("selected.json"))
                .unwrap();
        (directory, captured)
    }

    /// Selection recheck rejects literal byte drift after an authentic positive capture.
    #[test]
    fn selected_original_bytes_are_rechecked_without_decoded_substitution() {
        let (directory, captured) = selected();
        captured.verify().unwrap();
        let mut changed = captured.bytes.clone();
        changed.push(b'\n');
        std::fs::write(directory.path().join("selected.json"), changed).unwrap();
        assert!(
            captured
                .verify()
                .unwrap_err()
                .to_string()
                .contains("outbound selection original generation changed")
        );
    }

    /// Retaining the old inode makes same-byte replacement a distinct real file generation.
    #[test]
    fn selected_original_identity_is_rechecked_even_when_bytes_match() {
        let (directory, captured) = selected();
        captured.verify().unwrap();
        std::fs::rename(
            directory.path().join("selected.json"),
            directory.path().join("retained-old-selection.json"),
        )
        .unwrap();
        std::fs::write(directory.path().join("selected.json"), &captured.bytes).unwrap();
        assert!(
            captured
                .verify()
                .unwrap_err()
                .to_string()
                .contains("outbound selection original generation changed")
        );
    }
}
