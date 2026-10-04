//! POA&M source foundation and explicitly authored nonterminal workflow.
//!
//! Source-only checks establish input integrity. The separate workflow mode checks
//! authored ownership, history and schedules and generates native POA&M JSON; it
//! authenticates no actor and refuses undisposed completion/risk-acceptance history.

#![deny(missing_docs)]

/// Stable UUID v5 identities from immutable plan, item and milestone keys.
pub mod identity;
/// Closed and bounded unselected foundation manifest.
pub mod manifest;
/// Bounded, content-minimizing source-only inventories.
pub mod report;
/// Confined capture, exact companion binding and explicit source selection.
pub mod source;
/// Bounded author-supplied item, milestone and nonterminal assertion workflow.
pub mod workflow;
/// Bounded read-only declaration comparisons with complete stable-identity changes.
pub mod workflow_baseline;
/// Confined authored workflow commands with explicit dates and new-file publication.
pub mod workflow_cli;
/// Typed native POA&M projection used by the authored workflow producer.
mod workflow_model;

/// Complete escaped static HTML from the internally prepared portfolio report.
pub mod html;
/// Read-only connector-neutral local outbound intent from an actual native workflow original.
pub mod workflow_outbound;

/// Confined local outbound CLI with held selection and complete original generation rechecks.
pub mod workflow_outbound_cli;

/// Explicit supplied-native portfolio and separately qualified authoring preview.
pub mod portfolio;

/// Explicit paired-native portfolio CLI and separate new-file report publication.
pub mod portfolio_cli;

use std::io::Write as _;
use std::path::{Component, Path, PathBuf};

use serde_json::Value;

use crate::ForgeError;
use crate::assessment_results::manifest::{ArtifactManifest, ContextManifest, DocumentManifest};
use crate::cli::{AuthorReportFormat, PoamInitArgs};
use crate::hashing::sha256_hex;
use crate::json_strict::{self, Limits};

/// Original confined file generation and parsed native root used to prepare the scaffold.
struct CapturedInput {
    /// Normalized path relative to the fixed bundle root.
    relative: PathBuf,
    /// Complete original bytes retained for exact revalidation.
    bytes: Vec<u8>,
    /// File identity tuple returned by the held-handle confinement reader.
    identity: (u64, u64),
    /// Duplicate-safe parsed root from the original captured bytes.
    value: Value,
}

/// Scaffold an exact source manifest without selecting any remediation work.
///
/// # Errors
/// Returns exit-2 errors for unsafe, invalid, stale or inconsistent inputs,
/// unsupported metadata/source format, or a destination that cannot be published.
pub fn execute_init(args: &PoamInitArgs) -> Result<(), ForgeError> {
    let root = args.root.canonicalize().map_err(|_| error("cannot resolve bundle root"))?;
    let mut captures = Vec::new();
    let mut total = 0_u64;
    for (kind, path) in [
        ("assessment-results", &args.assessment_results),
        ("assessment-plan", &args.assessment_plan),
        ("system-security-plan", &args.ssp),
        ("profile", &args.profile),
        ("catalog", &args.catalog),
    ] {
        let remaining = (100 * 1024 * 1024_u64).saturating_sub(total);
        let input = capture(&root, path, kind, crate::io::MAX_FILE_SIZE.min(remaining))?;
        total += input.bytes.len() as u64;
        if total > 100 * 1024 * 1024 {
            return Err(error("source captures exceed the 100 MiB aggregate limit"));
        }
        captures.push(input);
    }
    let ar = pin(&captures[0], "assessment-results", path_text(&captures[0].relative)?)?;
    let ap_href = required(&captures[0].value, "/assessment-results/import-ap/href")?;
    let ap = pin(&captures[1], "assessment-plan", ap_href)?;
    let ssp_href = required(&captures[1].value, "/assessment-plan/import-ssp/href")?;
    let ssp = pin(&captures[2], "system-security-plan", ssp_href)?;
    let profile_href = required(&captures[2].value, "/system-security-plan/import-profile/href")?;
    let profile = pin(&captures[3], "profile", profile_href)?;
    let imports = captures[3]
        .value
        .pointer("/profile/imports")
        .and_then(Value::as_array)
        .ok_or_else(|| error("Profile must declare exactly one Catalog import"))?;
    if imports.len() != 1 {
        return Err(error("Profile must declare exactly one Catalog import"));
    }
    let catalog_href = required(&captures[3].value, "/profile/imports/0/href")?;
    let catalog = pin(&captures[4], "catalog", catalog_href)?;
    let draft = manifest::PoamManifest {
        schema_version: manifest::MANIFEST_SCHEMA_VERSION.to_string(),
        document: DocumentManifest {
            key: args.document_key.clone(),
            title: args.title.clone(),
            version: args.document_version.clone(),
            last_modified: args.last_modified.clone(),
        },
        source: manifest::SourceManifest {
            assessment_results: ar,
            result: manifest::ResultIdentity {
                uuid: args.result_uuid.clone(),
                key: args.result_key.clone(),
            },
            context: ContextManifest {
                assessment_plan: ap,
                ssp,
                profile,
                catalog,
                evidence_index: None,
            },
        },
        roles: Vec::new(),
        parties: Vec::new(),
        items: Vec::new(),
    };
    let rendered = json_line(&draft)?;
    let checked = manifest::parse(&rendered)?;
    let prepared = source::load(&root.join(".forge-poam-scaffold.json"), &checked.source)?;
    prepared.verify_inputs()?;
    verify_captures(&root, &captures)?;
    if let Some(output) = &args.output {
        // Keeping the manifest at the bundle root preserves all pinned relative paths.
        if output.components().count() != 1
            || !matches!(output.components().next(), Some(Component::Normal(_)))
        {
            return Err(error("--output must be a new manifest filename in --root"));
        }
        if output.extension().and_then(std::ffi::OsStr::to_str) != Some("json") {
            return Err(error("--output must use the .json extension"));
        }
        crate::authoring::output::publish_new_file(&root, output, &rendered)
            .map_err(|_| error("cannot publish new manifest; destination may exist or be unsafe"))
    } else {
        write_stdout(&rendered)
    }
}

/// Check exact source integrity and emit a deterministic minimized inventory.
///
/// # Errors
/// Returns an error for invalid manifest/source data, unsafe input, source drift
/// during checking, or failed report output. This does not validate plan workflow.
pub fn execute_source_check(path: &Path, format: AuthorReportFormat) -> Result<(), ForgeError> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let root = parent.canonicalize().map_err(|_| error("cannot resolve manifest directory"))?;
    let name = path.file_name().ok_or_else(|| error("manifest filename is required"))?;
    let relative = Path::new(name);
    let (bytes, identity) =
        crate::linkage::read_confined_local_file(&root, relative, manifest::MAX_MANIFEST_BYTES)
            .map_err(|_| error("manifest must be a bounded confined regular single-link file"))?;
    let manifest = manifest::parse(&bytes)?;
    let prepared = source::load(&root.join(relative), &manifest.source)?;
    let rendered = match format {
        AuthorReportFormat::Json => report::render_json(prepared.inventory())?.into_bytes(),
        AuthorReportFormat::Text => report::render_text(prepared.inventory())?.into_bytes(),
    };
    prepared.verify_inputs()?;
    let (current, current_identity) =
        crate::linkage::read_confined_local_file(&root, relative, manifest::MAX_MANIFEST_BYTES)
            .map_err(|_| error("manifest changed or became unsafe during checking"))?;
    if identity != current_identity || bytes != current {
        return Err(error("manifest changed during checking"));
    }
    write_stdout(&rendered)
}

/// Capture a confined single-link file and parse exactly one bounded native root.
fn capture(
    root: &Path,
    relative: &Path,
    kind: &str,
    max_bytes: u64,
) -> Result<CapturedInput, ForgeError> {
    let (bytes, identity) = crate::linkage::read_confined_local_file(root, relative, max_bytes)
        .map_err(|_| {
            error(format!("{kind} must be a bounded confined regular single-link file"))
        })?;
    let value = json_strict::parse_value(
        &bytes,
        kind,
        Limits { max_depth: 128, max_string_bytes: 1024 * 1024 },
    )
    .map_err(|_| error(format!("{kind} must contain bounded duplicate-free JSON")))?;
    if value.as_object().is_none_or(|object| object.len() != 1 || !object.contains_key(kind)) {
        return Err(error(format!("expected exactly one {kind} root")));
    }
    Ok(CapturedInput { relative: relative.to_path_buf(), bytes, identity, value })
}

/// Build exact byte, root and declared-version pins from an actual captured input.
fn pin(input: &CapturedInput, kind: &str, href: String) -> Result<ArtifactManifest, ForgeError> {
    Ok(ArtifactManifest {
        artifact: input.relative.clone(),
        href,
        expected_sha256: sha256_hex(&input.bytes),
        root_uuid: required(&input.value, &format!("/{kind}/uuid"))?,
        document_version: required(&input.value, &format!("/{kind}/metadata/version"))?,
        oscal_version: required(&input.value, &format!("/{kind}/metadata/oscal-version"))?,
    })
}

/// Read one nonempty string from an explicit native source field.
fn required(value: &Value, path: &str) -> Result<String, ForgeError> {
    value
        .pointer(path)
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .map(str::to_string)
        .ok_or_else(|| error(format!("required source field {path} is missing")))
}

/// Require a UTF-8 source path without inventing a replacement spelling.
fn path_text(path: &Path) -> Result<String, ForgeError> {
    path.to_str().map(str::to_string).ok_or_else(|| error("artifact path must be UTF-8"))
}

/// Require every original confined file identity and byte sequence before publication.
fn verify_captures(root: &Path, inputs: &[CapturedInput]) -> Result<(), ForgeError> {
    for input in inputs {
        let (bytes, identity) = crate::linkage::read_confined_local_file(
            root,
            &input.relative,
            crate::io::MAX_FILE_SIZE,
        )
        .map_err(|_| error("source changed or became unsafe before scaffold publication"))?;
        if identity != input.identity || bytes != input.bytes {
            return Err(error("source changed before scaffold publication"));
        }
    }
    Ok(())
}

/// Serialize the explicit scaffold with one deterministic trailing newline.
fn json_line(value: &impl serde::Serialize) -> Result<Vec<u8>, ForgeError> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|_| error("foundation JSON serialization failed"))?;
    bytes.push(b'\n');
    Ok(bytes)
}

/// Emit only the already-prepared foundation bytes to standard output.
fn write_stdout(bytes: &[u8]) -> Result<(), ForgeError> {
    std::io::stdout().lock().write_all(bytes).map_err(|_| error("cannot write foundation output"))
}

/// Construct the typed foundation failure without granting source or workflow approval.
fn error(message: impl Into<String>) -> ForgeError {
    ForgeError::PoamBuild(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that capture rejects wrong or multiple native roots.
    #[test]
    fn capture_rejects_wrong_or_multiple_native_roots() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let path = Path::new("source.json");
        for bytes in [
            br#"{"catalog":{}}"#.as_slice(),
            br#"{"assessment-results":{},"catalog":{}}"#.as_slice(),
        ] {
            std::fs::write(root.join(path), bytes).unwrap();
            let error = capture(&root, path, "assessment-results", 1024).err().unwrap();
            assert!(error.to_string().contains("expected exactly one assessment-results root"));
        }
    }

    /// Verify that scaffold capture revalidation requires original bytes and file identity.
    #[test]
    fn scaffold_capture_revalidation_requires_original_bytes_and_file_identity() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let path = Path::new("source.json");
        let original = br#"{"assessment-results":{}}"#;
        std::fs::write(root.join(path), original).unwrap();
        let input = capture(&root, path, "assessment-results", 1024).unwrap();
        let snapshot = [input];
        assert!(verify_captures(&root, &snapshot).is_ok());
        std::fs::write(root.join(path), br#"{"assessment-results":{"changed":true}}"#).unwrap();
        assert!(
            verify_captures(&root, &snapshot).unwrap_err().to_string().contains("source changed")
        );
        std::fs::write(root.join(path), original).unwrap();
        assert!(verify_captures(&root, &snapshot).is_ok());
        std::fs::rename(root.join(path), root.join("preserved-original.json")).unwrap();
        std::fs::write(root.join(path), original).unwrap();
        assert!(
            verify_captures(&root, &snapshot).unwrap_err().to_string().contains("source changed")
        );
        std::fs::remove_file(root.join(path)).unwrap();
        assert!(
            verify_captures(&root, &snapshot).unwrap_err().to_string().contains("became unsafe")
        );
    }
}

/// Complete read-only closure-reference joins from actual sealed local evidence captures.
pub mod workflow_evidence;
/// New-file publication and portable stdout for complete local evidence inspection.
pub mod workflow_evidence_cli;
