//! Explicit nonterminal POA&M CLI preparation and publication adapters.
//!
//! The captured manifest directory is the fixed native relative-link base. Artifact
//! and report destinations must be new single filenames there; no href is rewritten.
//! Separate output publication is deliberately not a two-file atomic transaction.

use std::io::Write as _;
use std::path::{Component, Path, PathBuf};

use serde_json::Value;

use super::{manifest, workflow, workflow_baseline};
use crate::ForgeError;
use crate::cli::{AuthorReportFormat, PoamBaselineArgs, PoamBuildArgs};

/// Original declaration generation retained independently of the producer's five sources.
struct Declaration {
    /// Exact normalized descendant used by the confined reader for every recheck.
    relative: PathBuf,
    /// Complete original bytes; no parsed/reserialized hash stands in for this input.
    bytes: Vec<u8>,
    /// Held-reader original file identity; same-byte replacement remains stale.
    identity: (u64, u64),
}

/// Actual authoring inputs captured before off-store preparation.
struct Inputs {
    /// Canonical manifest directory and immutable base of all generated native hrefs.
    root: PathBuf,
    /// Exact current manifest identity and original bytes.
    manifest: Declaration,
    /// Optional explicitly supplied prior authoring bytes; no history trust is inferred.
    baseline: Option<Declaration>,
}

/// Prepare valid authored work, publish a new native JSON, then emit its minimized schedule.
///
/// # Errors
/// Rejects invalid/stale/unsafe declarations, undisposed closure, any source drift,
/// relocation or alias destinations, existing files, and actual publication/output
/// failures. If a later report publication fails, the already-created artifact can
/// remain; the command returns invalid exit2 and never promises rollback.
pub fn execute_build(args: &PoamBuildArgs) -> Result<bool, ForgeError> {
    let inputs = capture_inputs(&args.manifest, args.baseline.as_deref())?;
    let forbidden = declared_input_paths(&inputs)?;
    validate_destinations(
        &inputs.root,
        &forbidden,
        Some(&args.output),
        args.report.as_deref(),
        args.format,
    )?;
    let prepared = prepare(&inputs, &args.as_of, args.due_soon_days)?;
    let report = render_report(prepared.schedule(), args.format)?;
    verify_inputs(&inputs, &prepared)?;
    publish(&inputs.root, &args.output, prepared.artifact())?;
    verify_inputs(&inputs, &prepared)?;
    emit_report(&inputs.root, args.report.as_deref(), &report)?;
    Ok(prepared.review_required())
}

/// Check explicit authored workflow and exact sources without publishing a native artifact.
///
/// A report file is created only for an explicit new report filename; otherwise the
/// minimized report goes to stdout. The boolean is a valid schedule/review action,
/// never an invalid-input or closure-authority flag.
/// # Errors
/// Rejects invalid/unsafe/stale declarations, unsupported terminal assertions, bad
/// explicit dates or intervals, existing/relocated report destinations and I/O failure.
pub fn execute_check(
    path: &Path,
    as_of: &str,
    due_soon_days: u16,
    baseline: Option<&Path>,
    report: Option<&Path>,
    format: AuthorReportFormat,
) -> Result<bool, ForgeError> {
    let inputs = capture_inputs(path, baseline)?;
    let forbidden = declared_input_paths(&inputs)?;
    validate_destinations(&inputs.root, &forbidden, None, report, format)?;
    let prepared = prepare(&inputs, as_of, due_soon_days)?;
    let rendered = render_report(prepared.schedule(), format)?;
    verify_inputs(&inputs, &prepared)?;
    emit_report(&inputs.root, report, &rendered)?;
    Ok(prepared.review_required())
}

/// Compare complete declarations and actual current sources without a native artifact.
///
/// A complete changed or refused comparison returns true for the valid exit-1
/// action state. It never converts a refusal observation into artifact admission.
/// Every declaration's original bytes and identity and all five native source
/// generations are rechecked before the single explicit report publication/stdout.
/// # Errors
/// Rejects missing/unsafe/unbounded inputs, malformed closed declarations or reopen
/// relations, invalid source capture/date, output aliases, unsupported publication
/// and complete-output bounds or I/O failure. No partial report earns valid credit.
pub fn execute_baseline(args: &PoamBaselineArgs) -> Result<bool, ForgeError> {
    let inputs = capture_inputs(&args.manifest, Some(&args.baseline))?;
    let prior =
        inputs.baseline.as_ref().ok_or_else(|| error("baseline declaration is required"))?;
    let reopens = args.reopens.as_deref().map(|path| capture(&inputs.root, path)).transpose()?;
    let current = workflow_baseline::parse_declaration(&inputs.manifest.bytes)?;
    let mut forbidden = vec![inputs.manifest.relative.clone(), prior.relative.clone()];
    if let Some(reopens) = &reopens {
        forbidden.push(reopens.relative.clone());
    }
    forbidden.extend([
        current.source.assessment_results.artifact,
        current.source.context.assessment_plan.artifact,
        current.source.context.ssp.artifact,
        current.source.context.profile.artifact,
        current.source.context.catalog.artifact,
    ]);
    validate_destinations(&inputs.root, &forbidden, None, args.report.as_deref(), args.format)?;
    let prepared = workflow_baseline::prepare(
        &inputs.root.join(&inputs.manifest.relative),
        &inputs.manifest.bytes,
        &prior.bytes,
        reopens.as_ref().map(|captured| captured.bytes.as_slice()),
        &args.as_of,
    )?;
    let report = render_baseline_report(prepared.report(), args.format)?;
    prepared.verify_inputs()?;
    recheck(&inputs.root, &inputs.manifest)?;
    recheck(&inputs.root, prior)?;
    if let Some(reopens) = &reopens {
        recheck(&inputs.root, reopens)?;
    }
    emit_report(&inputs.root, args.report.as_deref(), &report)?;
    Ok(prepared.review_required())
}

/// Preserve every comparison field in JSON or escaped ASCII text under the same bound.
fn render_baseline_report(bytes: &[u8], format: AuthorReportFormat) -> Result<Vec<u8>, ForgeError> {
    if bytes.len() > workflow::MAX_OUTPUT_BYTES {
        return Err(error("baseline report exceeds complete output bound"));
    }
    if matches!(format, AuthorReportFormat::Json) {
        return Ok(bytes.to_vec());
    }
    let value: Value = serde_json::from_slice(bytes)
        .map_err(|_| error("prepared baseline report is invalid JSON"))?;
    let text = serde_json::to_string_pretty(&value)
        .map_err(|_| error("baseline text rendering failed"))?;
    let mut output = String::new();
    for line in text.lines() {
        append_ascii(&mut output, line)?;
        append_newline(&mut output)?;
    }
    Ok(output.into_bytes())
}

/// Capture manifest and explicit prior through existing bounded held-identity primitives.
fn capture_inputs(path: &Path, baseline: Option<&Path>) -> Result<Inputs, ForgeError> {
    let parent =
        path.parent().filter(|part| !part.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    let root = parent.canonicalize().map_err(|_| error("cannot resolve manifest directory"))?;
    let name = path.file_name().ok_or_else(|| error("manifest filename is required"))?;
    let manifest = capture(&root, Path::new(name))?;
    let baseline = baseline.map(|relative| capture(&root, relative)).transpose()?;
    Ok(Inputs { root, manifest, baseline })
}

/// Retain original raw declaration bytes and exact confined file identity, not just a digest.
fn capture(root: &Path, relative: &Path) -> Result<Declaration, ForgeError> {
    let (bytes, identity) =
        crate::linkage::read_confined_local_file(root, relative, manifest::MAX_MANIFEST_BYTES)
            .map_err(|_| {
                error("declaration must be a bounded confined regular single-link file")
            })?;
    Ok(Declaration { relative: relative.to_path_buf(), bytes, identity })
}

/// Consume the frozen producer against the actual original manifest base and optional prior bytes.
fn prepare(
    inputs: &Inputs,
    as_of: &str,
    due_soon_days: u16,
) -> Result<workflow::PreparedWorkflow, ForgeError> {
    workflow::prepare(
        &inputs.root.join(&inputs.manifest.relative),
        &inputs.manifest.bytes,
        as_of,
        due_soon_days,
        inputs.baseline.as_ref().map(|prior| prior.bytes.as_slice()),
    )
}

/// Reconcile every current/baseline declaration and all five source generations before output.
fn verify_inputs(inputs: &Inputs, prepared: &workflow::PreparedWorkflow) -> Result<(), ForgeError> {
    prepared.verify_inputs()?;
    recheck(&inputs.root, &inputs.manifest)?;
    if let Some(prior) = &inputs.baseline {
        recheck(&inputs.root, prior)?;
    }
    Ok(())
}

/// Require original bytes and file identity; a same-byte replacement is still refused.
fn recheck(root: &Path, original: &Declaration) -> Result<(), ForgeError> {
    let current = capture(root, &original.relative)?;
    if current.identity != original.identity || current.bytes != original.bytes {
        return Err(error("authoring declaration identity or bytes changed during preparation"));
    }
    Ok(())
}

/// Collect complete declared source paths plus actual manifest/prior capture paths before any output.
///
/// This is collision preflight, not source authority: the producer still captures
/// and verifies the complete native closure and every exact selection tuple.
fn declared_input_paths(inputs: &Inputs) -> Result<Vec<PathBuf>, ForgeError> {
    let manifest = workflow::parse(&inputs.manifest.bytes)?;
    let mut paths = vec![inputs.manifest.relative.clone()];
    if let Some(prior) = &inputs.baseline {
        paths.push(prior.relative.clone());
    }
    paths.extend([
        manifest.source.assessment_results.artifact,
        manifest.source.context.assessment_plan.artifact,
        manifest.source.context.ssp.artifact,
        manifest.source.context.profile.artifact,
        manifest.source.context.catalog.artifact,
    ]);
    Ok(paths)
}

/// Validate all explicit destinations and reject known partial-output conflicts before preparation.
fn validate_destinations(
    root: &Path,
    forbidden: &[PathBuf],
    artifact: Option<&Path>,
    report: Option<&Path>,
    format: AuthorReportFormat,
) -> Result<(), ForgeError> {
    if let Some(path) = artifact {
        destination(root, path, "json")?;
        input_collision(path, forbidden)?;
    }
    if let Some(path) = report {
        destination(
            root,
            path,
            match format {
                AuthorReportFormat::Json => "json",
                AuthorReportFormat::Text => "txt",
            },
        )?;
        input_collision(path, forbidden)?;
    }
    if let (Some(artifact), Some(report)) = (artifact, report) {
        if artifact
            .as_os_str()
            .to_string_lossy()
            .eq_ignore_ascii_case(&report.as_os_str().to_string_lossy())
        {
            return Err(error("artifact and report destinations must be distinct new filenames"));
        }
    }
    Ok(())
}

/// Reject case-folded source/manifest/baseline collisions before any platform write is attempted.
fn input_collision(output: &Path, inputs: &[PathBuf]) -> Result<(), ForgeError> {
    let name = output.to_str().ok_or_else(|| error("output filename must be UTF8"))?;
    if inputs
        .iter()
        .any(|input| input.to_str().is_some_and(|input| input.eq_ignore_ascii_case(name)))
    {
        return Err(error("output filename aliases a declared source, manifest or baseline"));
    }
    Ok(())
}

/// Bind every emitted href to the manifest root by refusing relocation instead of inventing rewrites.
fn destination(root: &Path, relative: &Path, extension: &str) -> Result<(), ForgeError> {
    let text = relative.to_str().ok_or_else(|| error("output filename must be UTF8"))?;
    crate::authoring::output::validate_relative(text)
        .map_err(|_| error("output filename must satisfy the portable publisher profile"))?;
    if relative.components().count() != 1
        || !matches!(relative.components().next(), Some(Component::Normal(_)))
        || relative.as_os_str() != Path::new(text).file_name().unwrap_or_default()
        || text.contains(['/', '\\'])
        || text.chars().any(char::is_control)
        || text.trim() != text
        || text.is_empty()
        || relative.extension().and_then(std::ffi::OsStr::to_str) != Some(extension)
    {
        return Err(error(
            "output must be a new correctly suffixed filename in the manifest directory",
        ));
    }
    match std::fs::symlink_metadata(root.join(relative)) {
        Err(cause) if cause.kind() == std::io::ErrorKind::NotFound => Ok(()),
        _ => Err(error("output destination already exists or cannot be qualified")),
    }
}

/// Delegate actual no-replacement confinement/publication without treating preflight as authority.
fn publish(root: &Path, relative: &Path, bytes: &[u8]) -> Result<(), ForgeError> {
    crate::authoring::output::publish_new_file(root, relative, bytes).map_err(|_| {
        error("cannot publish new workflow output; destination may exist or be unsafe")
    })
}

/// Emit one already-prepared complete report only to the explicit destination or standard output.
fn emit_report(root: &Path, path: Option<&Path>, bytes: &[u8]) -> Result<(), ForgeError> {
    match path {
        Some(path) => publish(root, path, bytes),
        None => std::io::stdout()
            .lock()
            .write_all(bytes)
            .map_err(|_| error("cannot write minimized workflow report")),
    }
}

/// Render complete minimized schedule data; text escapes every authored key into ASCII.
fn render_report(bytes: &[u8], format: AuthorReportFormat) -> Result<Vec<u8>, ForgeError> {
    if bytes.len() > workflow::MAX_OUTPUT_BYTES {
        return Err(error("schedule exceeds output bound"));
    }
    if matches!(format, AuthorReportFormat::Json) {
        return Ok(bytes.to_vec());
    }
    let value: Value =
        serde_json::from_slice(bytes).map_err(|_| error("prepared schedule is invalid JSON"))?;
    let mut output = String::new();
    for name in [
        "schema_version",
        "as_of",
        "due_soon_days",
        "source_objects",
        "items",
        "milestones",
        "overdue",
        "due_soon",
        "blocked",
        "boundary",
    ] {
        let field = value.get(name).ok_or_else(|| error("prepared schedule field is missing"))?;
        append_ascii(&mut output, name)?;
        append_ascii(&mut output, ": ")?;
        append_ascii(
            &mut output,
            &serde_json::to_string(field).map_err(|_| error("schedule serialization failed"))?,
        )?;
        append_newline(&mut output)?;
    }
    let rows = value
        .get("rows")
        .and_then(Value::as_array)
        .ok_or_else(|| error("prepared schedule rows are missing"))?;
    for row in rows {
        append_ascii(
            &mut output,
            &serde_json::to_string(row).map_err(|_| error("schedule row serialization failed"))?,
        )?;
        append_newline(&mut output)?;
    }
    Ok(output.into_bytes())
}

/// Append escaped printable ASCII while enforcing the complete output bound before growth.
fn append_ascii(output: &mut String, text: &str) -> Result<(), ForgeError> {
    for character in text.chars() {
        for escaped in character.escape_default() {
            if output.len() == workflow::MAX_OUTPUT_BYTES {
                return Err(error("text schedule exceeds output bound"));
            }
            output.push(escaped);
        }
    }
    Ok(())
}

/// Append only an intentional report line delimiter inside the same complete byte limit.
fn append_newline(output: &mut String) -> Result<(), ForgeError> {
    if output.len() == workflow::MAX_OUTPUT_BYTES {
        return Err(error("text schedule exceeds output bound"));
    }
    output.push('\n');
    Ok(())
}

/// Preserve the existing invalid-input/publication exit2 contract without exposing private paths.
fn error(message: impl Into<String>) -> ForgeError {
    ForgeError::PoamBuild(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Declaration rechecks reject changed bytes and same-byte replacement identity.
    #[test]
    fn declaration_recheck_requires_original_identity_and_bytes() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let path = Path::new("plan.json");
        std::fs::write(root.join(path), b"{}").unwrap();
        let original = capture(&root, path).unwrap();
        assert!(recheck(&root, &original).is_ok());
        std::fs::write(root.join(path), b"changed").unwrap();
        assert!(recheck(&root, &original).is_err());
        std::fs::write(root.join(path), b"{}").unwrap();
        assert!(recheck(&root, &original).is_ok());
        std::fs::rename(root.join(path), root.join("retained-original.json")).unwrap();
        std::fs::write(root.join(path), b"{}").unwrap();
        assert!(recheck(&root, &original).is_err());
    }

    /// Explicit baseline paths remain confined and capture bounds are whole-file, not prefix checks.
    #[test]
    fn baseline_and_manifest_capture_do_not_read_outside_the_root() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        std::fs::write(root.join("plan.json"), b"{}").unwrap();
        assert!(
            capture_inputs(&root.join("plan.json"), Some(Path::new("../outside.json"))).is_err()
        );
        assert!(capture_inputs(&root.join("plan.json"), Some(&root.join("plan.json"))).is_err());
        let large = std::fs::File::create(root.join("large.json")).unwrap();
        large.set_len(manifest::MAX_MANIFEST_BYTES + 1).unwrap();
        assert!(capture(&root, Path::new("large.json")).is_err());
    }

    /// Destination preflight refuses relocated paths, aliases, existing files and wrong report suffixes.
    #[test]
    fn output_placement_preserves_href_base_and_no_replacement() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        assert!(
            validate_destinations(
                &root,
                &[],
                Some(Path::new("native.json")),
                Some(Path::new("report.json")),
                AuthorReportFormat::Json
            )
            .is_ok()
        );
        for path in [
            "nested/native.json",
            "./native.json",
            "../native.json",
            "native.xml",
            "native\\name.json",
            "NUL.json",
            "COM1.json",
            "bad:name.json",
            "unicode-é.json",
        ] {
            assert!(destination(&root, Path::new(path), "json").is_err(), "{path}");
        }
        assert!(
            validate_destinations(
                &root,
                &[],
                Some(Path::new("same.json")),
                Some(Path::new("SAME.json")),
                AuthorReportFormat::Json
            )
            .is_err()
        );
        std::fs::write(root.join("existing.json"), b"sentinel").unwrap();
        assert!(destination(&root, Path::new("existing.json"), "json").is_err());
        assert_eq!(std::fs::read(root.join("existing.json")).unwrap(), b"sentinel");
        assert!(
            validate_destinations(
                &root,
                &[],
                None,
                Some(Path::new("report.json")),
                AuthorReportFormat::Text
            )
            .is_err()
        );
        let long_name = format!("{}.json", "x".repeat(124));
        assert!(destination(&root, Path::new(&long_name), "json").is_err());
        for input in ["poam.json", "prior.json", "assessment-results.json", "ssp.json"] {
            let candidate = input.to_ascii_uppercase().replace(".JSON", ".json");
            assert!(input_collision(Path::new(&candidate), &[PathBuf::from(input)]).is_err());
        }
    }

    /// Text cannot inject terminal controls or Unicode bidi markers, and expansion never returns a prefix.
    #[test]
    fn minimized_text_escapes_authored_keys_and_enforces_whole_output_limit() {
        let value = serde_json::json!({"schema_version":"forge.poam-schedule/1","as_of":"2026-02-06", "due_soon_days":7,
            "source_objects":3,"items":1,"milestones":0,"overdue":1,"due_soon":0,"blocked":0,
            "boundary":"assertions only","rows":[{"item_key":"key\u{202e}\u{1b}[31m","milestone_key":null}]});
        let bytes = serde_json::to_vec(&value).unwrap();
        let text = render_report(&bytes, AuthorReportFormat::Text).unwrap();
        assert!(text.is_ascii());
        assert!(!text.contains(&0x1b));
        assert!(String::from_utf8(text).unwrap().contains("\\u{202e}"));
        let mut almost_full = "x".repeat(workflow::MAX_OUTPUT_BYTES - 1);
        assert!(append_ascii(&mut almost_full, "\u{202e}").is_err());
        assert_eq!(almost_full.len(), workflow::MAX_OUTPUT_BYTES);
        assert!(append_newline(&mut almost_full).is_err());
    }

    /// Symlink declaration inputs and outputs remain refused by the actual existing confinement/publication seams.
    #[cfg(unix)]
    #[test]
    fn symlink_and_hardlink_declarations_are_not_capture_authority() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        std::fs::write(root.join("real.json"), b"{}").unwrap();
        std::os::unix::fs::symlink("real.json", root.join("link.json")).unwrap();
        assert!(capture(&root, Path::new("link.json")).is_err());
        std::fs::hard_link(root.join("real.json"), root.join("hard.json")).unwrap();
        assert!(capture(&root, Path::new("hard.json")).is_err());
        assert!(destination(&root, Path::new("link.json"), "json").is_err());
    }
}

/// Proposed baseline adapter rendering controls; native CLI controls use real five-file fixtures.
#[cfg(test)]
mod baseline_adapter_tests {
    use super::*;

    /// Complete JSON preserves every row and the text representation escapes authored Unicode.
    #[test]
    fn baseline_renderer_preserves_all_fields_and_escapes_text() {
        let value = serde_json::json!({"schema_version":"forge.poam-baseline/1",
            "rows":[{"item_key":"key\u{202e}\u{1b}","presence":"removed"}],
            "artifact_validated":false,"refusal_observations":["removed-identity"],
            "items":{"previous":1,"current":0,"removed":1},"milestones":{"previous":2,"current":0,"removed":2}});
        let bytes = serde_json::to_vec(&value).unwrap();
        assert_eq!(render_baseline_report(&bytes, AuthorReportFormat::Json).unwrap(), bytes);
        let text = render_baseline_report(&bytes, AuthorReportFormat::Text).unwrap();
        assert!(text.is_ascii());
        assert!(!text.contains(&0x1b));
        let text = String::from_utf8(text).unwrap();
        for expected in ["removed-identity", "artifact_validated", "milestones", "\\u{202e}"] {
            assert!(text.contains(expected), "{expected}");
        }
    }

    /// Complete bounds fail before returning bytes, including Unicode expansion in text.
    #[test]
    fn baseline_renderer_refuses_raw_and_expanded_whole_output_limits() {
        assert!(
            render_baseline_report(
                &vec![b' '; workflow::MAX_OUTPUT_BYTES + 1],
                AuthorReportFormat::Json
            )
            .is_err()
        );
        let value = serde_json::json!({"key":"é".repeat(workflow::MAX_OUTPUT_BYTES / 3)});
        let bytes = serde_json::to_vec(&value).unwrap();
        assert!(bytes.len() < workflow::MAX_OUTPUT_BYTES);
        assert!(render_baseline_report(&bytes, AuthorReportFormat::Text).is_err());
    }
}
