//! Publish complete local evidence inspection without native or closure authority.

use std::io::Write as _;
use std::path::{Component, Path, PathBuf};

use super::workflow_evidence;
use crate::ForgeError;
use crate::cli::PoamEvidenceArgs;

/// Inspect every declared closure assertion and publish a complete bounded report.
///
/// Return whether the valid inspection requires review. Terminal declarations
/// always remain unadmitted; observed hashes and dates convey no sufficiency.
/// # Errors
/// Refuse invalid, unsafe, stale or unbounded originals, input aliases, an existing
/// report destination, unsupported publication or incomplete output writes.
pub fn execute(args: &PoamEvidenceArgs) -> Result<bool, ForgeError> {
    let root = plan_root(&args.manifest)?;
    let bindings = binding_path(&root, &args.links)?;
    let name = args
        .manifest
        .file_name()
        .ok_or_else(|| error("evidence inspection needs a plan filename"))?;
    let plan = root.join(name);
    if let Some(report) = &args.report {
        destination(&root, report)?;
        let complete = root.join(report);
        if paths_alias(&complete, &plan) || paths_alias(&complete, &bindings) {
            return Err(error("evidence report aliases an explicit declaration"));
        }
    }
    let prepared = workflow_evidence::prepare_with_report(
        &plan,
        &bindings,
        &args.as_of,
        args.report.as_deref(),
    )
    .map_err(|_| error("evidence inspection inputs could not be qualified"))?;
    if let Some(report) = &args.report {
        let complete = root.join(report);
        if prepared.input_paths().any(|original| paths_alias(&complete, &original)) {
            return Err(error("evidence report aliases an actual input observation"));
        }
    }
    prepared
        .verify_inputs()
        .map_err(|_| error("evidence inspection original generations changed or are unsafe"))?;
    match &args.report {
        Some(report) => {
            crate::authoring::output::publish_new_file(&root, report, prepared.report())
                .map_err(|_| error("cannot publish new complete evidence inspection report"))?;
        }
        None => std::io::stdout()
            .lock()
            .write_all(prepared.report())
            .map_err(|_| error("cannot write complete evidence inspection JSON"))?,
    }
    Ok(prepared.review_required())
}

/// Resolve the actual plan directory once before sealed original capture.
fn plan_root(manifest: &Path) -> Result<PathBuf, ForgeError> {
    let parent =
        manifest.parent().filter(|path| !path.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let root = std::path::absolute(parent)
        .map_err(|_| error("evidence inspection plan directory is unavailable"))?;
    crate::linkage::fresh::qualify_root(&root)
        .map_err(|_| error("evidence inspection plan root is unsafe or unavailable"))?;
    Ok(root)
}

/// Resolve the explicitly plan-relative companion without cwd substitution or path normalization.
fn binding_path(root: &Path, relative: &Path) -> Result<PathBuf, ForgeError> {
    let text = relative.to_str().ok_or_else(|| error("evidence companion path must be UTF8"))?;
    if text.is_empty()
        || text.len() > 4096
        || text.contains(['\\', ':', '?', '#'])
        || text.chars().any(char::is_control)
        || text.split('/').any(|part| {
            part.is_empty() || part == "." || part == ".." || part.ends_with(['.', ' '])
        })
        || relative.components().any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(error("evidence companion must be a normalized plan-relative descendant"));
    }
    Ok(root.join(relative))
}

/// Preflight one absent portable JSON filename before any inspection capture.
fn destination(root: &Path, relative: &Path) -> Result<(), ForgeError> {
    let text = relative.to_str().ok_or_else(|| error("evidence report filename must be UTF8"))?;
    crate::authoring::output::validate_relative(text)
        .map_err(|_| error("evidence report filename is outside the portable publisher profile"))?;
    if relative.components().count() != 1
        || !matches!(relative.components().next(), Some(Component::Normal(_)))
        || text.contains(['/', '\\'])
        || relative.extension().and_then(std::ffi::OsStr::to_str) != Some("json")
    {
        return Err(error("evidence report must be one correctly suffixed JSON filename"));
    }
    match std::fs::symlink_metadata(root.join(relative)) {
        Err(cause) if cause.kind() == std::io::ErrorKind::NotFound => Ok(()),
        _ => Err(error("evidence report already exists or cannot be qualified")),
    }
}

/// Compare full lossless path components with conservative ASCII case folding.
fn paths_alias(left: &Path, right: &Path) -> bool {
    left.components().count() == right.components().count()
        && left.components().zip(right.components()).all(|(left, right)| {
            left.as_os_str()
                .as_encoded_bytes()
                .eq_ignore_ascii_case(right.as_os_str().as_encoded_bytes())
        })
}

/// Emit a fixed redacted stage refusal without paths, author text or underlying errors.
fn error(message: &'static str) -> ForgeError {
    ForgeError::PoamBuild(message.into())
}

#[cfg(test)]
mod tests {
    //! Destination admission controls run without detached preparation or output authority.
    use super::*;

    /// Existing regular output is preserved before source preparation.
    #[test]
    fn existing_output_is_preserved_and_portable_new_name_is_admitted() {
        let root = tempfile::tempdir().unwrap();
        let sentinel = root.path().join("inspection.json");
        std::fs::write(&sentinel, b"OWNED").unwrap();
        assert!(destination(root.path(), Path::new("inspection.json")).is_err());
        assert_eq!(std::fs::read(sentinel).unwrap(), b"OWNED");
        assert!(destination(root.path(), Path::new("new-inspection.json")).is_ok());
        assert!(!root.path().join("new-inspection.json").exists());
    }

    /// Complete component comparison refuses folded aliases without prefix string confusion.
    #[test]
    fn folded_component_aliases_preserve_complete_path_boundaries() {
        assert!(paths_alias(Path::new("/safe/PLAN.json"), Path::new("/SAFE/plan.json")));
        assert!(!paths_alias(Path::new("/safe/plan.json"), Path::new("/safe/plan.json-child")));
        assert!(!paths_alias(Path::new("/safe/plan.json"), Path::new("/safe/child/plan.json")));
    }

    /// A nested companion resolves from the declared plan root and rejects ambiguous spelling.
    #[test]
    fn companion_uses_plan_root_and_refuses_non_descendant_spellings() {
        let root = Path::new("/declared/plan");
        assert_eq!(
            binding_path(root, Path::new("nested/links.json")).unwrap(),
            root.join("nested/links.json")
        );
        for name in [
            "",
            "/links.json",
            "../links.json",
            "nested//links.json",
            "./links.json",
            "nested/links.json ",
            "https:links.json",
        ] {
            assert!(binding_path(root, Path::new(name)).is_err());
        }
    }

    /// A caller-supplied symlink ancestor cannot be hidden by canonicalization.
    #[cfg(unix)]
    #[test]
    fn linked_plan_root_is_refused_before_input_capture() {
        let owned = tempfile::tempdir().unwrap();
        let actual = owned.path().canonicalize().unwrap();
        std::fs::create_dir(actual.join("real")).unwrap();
        std::os::unix::fs::symlink(actual.join("real"), actual.join("alias")).unwrap();
        assert!(plan_root(&actual.join("real/plan.json")).is_ok());
        assert!(plan_root(&actual.join("alias/plan.json")).is_err());
        assert!(!actual.join("real/plan.json").exists());
    }

    /// Unsafe suffixes, nested paths and reserved portable names are refused before file output.
    #[test]
    fn unsupported_destinations_are_refused_without_creation() {
        let root = tempfile::tempdir().unwrap();
        for name in
            ["../new.json", "nested/new.json", "new.txt", "NEW.JSON", "CON.json", "new\\name.json"]
        {
            assert!(destination(root.path(), Path::new(name)).is_err(), "{name}");
        }
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
    }
}
