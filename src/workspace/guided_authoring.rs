//! PRD 062 S-2: PRD 061 artifacts for the workspace, produced by the standard confined loader.
//!
//! The author project, authoring pack and human clause files arrive as exact request bytes.
//! Baseline files are copied only from registered resources into one private directory, and
//! the PRD 061 loader runs there unchanged, so each artifact equals the matching CLI output.
use super::contract::{Error, Result};
use super::preparation::{ProgressUpdate, Stage, WorkControl, WorkError};
use super::services::{Item, Snapshot};
use crate::ForgeError;
use crate::authoring::manifest::{MAX_CLAUSE_BYTES, MAX_MANIFEST_BYTES, MAX_TOTAL_BYTES};
use serde_json::{Value, json};
use std::io::Write as _;
use std::path::{Path, PathBuf};

/// Bound on explicitly supplied human clause files, mirroring the public request schema.
const MAX_SUPPLIED_CLAUSES: usize = 64;

/// One single-file PRD 061 artifact.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ArtifactKind {
    /// The drafting plan (`forge.authoring-plan/1`).
    Plan,
    /// The span-level provenance graph of a build.
    Provenance,
    /// One policy's Markdown skeleton.
    Skeleton,
}

impl ArtifactKind {
    /// Wire name of the artifact.
    #[must_use]
    pub(crate) const fn wire(self) -> &'static str {
        match self {
            Self::Plan => "plan",
            Self::Provenance => "provenance",
            Self::Skeleton => "skeleton",
        }
    }

    /// Effect operation type of the single-file write.
    #[must_use]
    pub(crate) const fn operation(self) -> &'static str {
        match self {
            Self::Plan => "authoring-plan-write",
            Self::Provenance => "authoring-provenance-write",
            Self::Skeleton => "authoring-skeleton-write",
        }
    }
}

/// Exact artifact bytes, the registered inputs they consumed, and a safe client summary.
pub(crate) struct PreparedAuthoring<'a> {
    /// Effect operation type for the preview.
    pub kind: &'static str,
    /// Exact bytes to confirm and write.
    pub bytes: Vec<u8>,
    /// Registered resources whose bytes reached the artifact; pinned into the preview.
    pub inputs: Vec<&'a Item>,
    /// Artifact kind, gap counts and per-policy draft states. No paths, names or excerpts.
    pub summary: Value,
}

/// Prepare one PRD 061 artifact from the request documents and the registered baseline.
///
/// # Errors
/// Returns a stable contract error for unsafe or oversized supplied documents, an unknown
/// policy, an engine rejection, or a workspace interruption.
pub(crate) fn prepare<'a>(
    snapshot: &'a Snapshot,
    request: &Value,
    control: &mut dyn WorkControl,
) -> Result<PreparedAuthoring<'a>> {
    let kind = match text(request, "artifact_kind")? {
        "plan" => ArtifactKind::Plan,
        "provenance" => ArtifactKind::Provenance,
        "skeleton" => ArtifactKind::Skeleton,
        _ => return Err(Error::invalid()),
    };
    let project_path = text(request, "author_project_path")?;
    let project = text(request, "author_project")?.as_bytes();
    let pack = text(request, "authoring_pack")?.as_bytes();
    let policy_key = request.get("policy_key").and_then(Value::as_str);
    let clauses = request
        .get("human_clauses")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .map(|item| {
                    Ok((text(item, "path")?.to_owned(), text(item, "content")?.as_bytes().to_vec()))
                })
                .collect::<Result<Vec<_>>>()
        })
        .transpose()?
        .unwrap_or_default();
    control
        .checkpoint(Stage::CopyInputs, ProgressUpdate::Unchanged)
        .map_err(WorkError::into_error)?;
    let directory = tempfile::tempdir().map_err(|_| internal())?;
    let items: Vec<&'a Item> = snapshot.items.iter().collect();
    let manifest = materialize(
        directory.path(),
        project_path,
        project,
        pack,
        &clauses,
        items.iter().map(|item| (item.registration.path.as_str(), item.captured.bytes.as_slice())),
    )?;
    control
        .checkpoint(Stage::CopyInputs, ProgressUpdate::Unchanged)
        .map_err(WorkError::into_error)?;
    control
        .checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)
        .map_err(WorkError::into_error)?;
    let (loaded, plan) =
        crate::authoring::load_project(&manifest).map_err(|cause| engine_error(&cause))?;
    let bytes = match kind {
        ArtifactKind::Plan => {
            crate::authoring::report::render_json(&plan).map_err(|cause| engine_error(&cause))?
        }
        ArtifactKind::Provenance | ArtifactKind::Skeleton => {
            let rendered = crate::authoring::render::render(&loaded, &plan)
                .map_err(|cause| engine_error(&cause))?;
            if kind == ArtifactKind::Provenance {
                rendered.provenance
            } else {
                let key = policy_key.ok_or_else(Error::invalid)?;
                rendered
                    .policies
                    .into_iter()
                    .find(|policy| policy.policy_key == key)
                    .ok_or_else(|| rejected("The plan has no policy with the requested key."))?
                    .markdown
            }
        }
    };
    // Only files the loader actually read are pinned; supplied documents are bound by the
    // artifact's own hashes and cannot share a path with a registered resource.
    let inputs = loaded
        .inputs
        .iter()
        .filter_map(|fingerprint| {
            items.iter().copied().find(|item| item.registration.path == fingerprint.path)
        })
        .collect();
    let mut summary = json!({
        "artifact_kind": kind.wire(),
        "gap_counts": serde_json::to_value(&plan.counts).map_err(|_| Error::invalid())?,
        "policies": plan
            .policies
            .iter()
            .map(|policy| json!({"policy_key": policy.policy_key, "state": policy.state.as_str()}))
            .collect::<Vec<_>>(),
    });
    if kind == ArtifactKind::Skeleton {
        summary["policy_key"] = json!(policy_key);
    }
    Ok(PreparedAuthoring { kind: kind.operation(), bytes, inputs, summary })
}

/// One request string field.
fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value.get(key).and_then(Value::as_str).ok_or_else(Error::invalid)
}

/// The request named something the PRD 061 services reject.
fn rejected(message: &'static str) -> Error {
    Error::new("validation-failed", message, false)
}

/// An unexpected local failure; the cause never leaves the process.
fn internal() -> Error {
    Error::new("internal-error", "The guided authoring preparation failed unexpectedly.", false)
}

/// Authored guidance for an engine rejection. Engine text is never retained
/// (`contract::Error`), so the common causes map to fixed messages.
fn engine_error(cause: &ForgeError) -> Error {
    let text = cause.to_string();
    rejected(if text.contains("does not match its exact byte pin") {
        "A supplied document or clause does not match the SHA-256 pin recorded for it. Supply the exact bytes the reviewer pinned."
    } else if text.contains("baseline fingerprints must match exactly") {
        "The author project, authoring pack and the registered gap report, framework and catalog fingerprints do not match. Re-pin the baseline after the reviewed change."
    } else if text.contains("does not exactly represent the complete current") {
        "The registered gap report is not the exact current analysis of the registered applicability manifest and framework. Re-run the analysis before authoring."
    } else {
        "The PRD 061 services rejected the authoring inputs. Check the author project, the authoring pack and the registered baseline files; forge author plan reports the detailed reason."
    })
}

/// Write the supplied documents and the registered baseline bytes into one private directory.
///
/// Every path is validated and every collision is found before the first write.
///
/// # Errors
/// Returns a stable contract error for an unsafe or colliding path, an oversized input, or an
/// author project that does not parse.
fn materialize<'a>(
    directory: &Path,
    project_path: &str,
    project: &[u8],
    pack: &[u8],
    clauses: &[(String, Vec<u8>)],
    registered: impl Iterator<Item = (&'a str, &'a [u8])>,
) -> Result<PathBuf> {
    if project.len() as u64 > MAX_MANIFEST_BYTES
        || pack.len() as u64 > MAX_MANIFEST_BYTES
        || clauses.len() > MAX_SUPPLIED_CLAUSES
        || clauses.iter().any(|(_, bytes)| bytes.len() as u64 > MAX_CLAUSE_BYTES)
    {
        return Err(Error::invalid());
    }
    let author =
        crate::authoring::manifest::parse_project(project).map_err(|cause| engine_error(&cause))?;
    let pack_path = author.authoring_pack.path.to_str().ok_or_else(Error::invalid)?;
    let mut supplied: Vec<(&str, &[u8])> = vec![(project_path, project), (pack_path, pack)];
    for (path, bytes) in clauses {
        supplied.push((path.as_str(), bytes.as_slice()));
    }
    for (path, _) in &supplied[1..] {
        crate::authoring::manifest::validate_local_path("supplied file", Path::new(path))
            .map_err(|cause| engine_error(&cause))?;
    }
    let registered: Vec<(&str, &[u8])> = registered.collect();
    let mut supplied_keys: Vec<String> = Vec::new();
    let mut total = 0usize;
    for (path, bytes) in &supplied {
        let key = path.to_ascii_lowercase();
        if supplied_keys.contains(&key) {
            return Err(Error::invalid());
        }
        supplied_keys.push(key);
        total = total.checked_add(bytes.len()).ok_or_else(oversized_baseline)?;
    }
    for (path, bytes) in &registered {
        if supplied_keys.contains(&path.to_ascii_lowercase()) {
            return Err(Error::invalid());
        }
        total = total.checked_add(bytes.len()).ok_or_else(oversized_baseline)?;
    }
    if total as u64 > MAX_TOTAL_BYTES {
        return Err(oversized_baseline());
    }
    // The confined reader walks root components without following symlinks.
    let root = directory.canonicalize().map_err(|_| internal())?;
    for (path, bytes) in supplied.iter().chain(registered.iter()) {
        let target = root.join(path);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|_| internal())?;
        }
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&target)
            .map_err(|_| internal())?;
        file.write_all(bytes).map_err(|_| internal())?;
    }
    Ok(root.join(project_path))
}

/// The registered baseline exceeds the PRD 061 aggregate input budget.
fn oversized_baseline() -> Error {
    Error::new(
        "payload-too-large",
        "The supplied documents and registered resources exceed the authoring input budget.",
        false,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authoring::load_project;
    use crate::authoring::manifest::MAX_MANIFEST_BYTES;
    use crate::authoring::report::render_json;

    const BASELINE: [&str; 3] = ["applicability.json", "framework.json", "gap-report.json"];

    fn example(name: &str) -> Vec<u8> {
        std::fs::read(format!("{}/examples/authoring/{name}", env!("CARGO_MANIFEST_DIR")))
            .expect("checked-in example")
    }

    fn count_files(directory: &Path) -> usize {
        std::fs::read_dir(directory).map_or(0, |entries| {
            entries
                .flatten()
                .map(|entry| {
                    let path = entry.path();
                    if path.is_dir() { count_files(&path) } else { 1 }
                })
                .sum()
        })
    }

    fn baseline() -> Vec<(String, Vec<u8>)> {
        BASELINE.iter().map(|name| ((*name).to_owned(), example(name))).collect()
    }

    #[test]
    fn supplied_paths_are_refused_before_anything_is_written() {
        for path in [
            "../escape.md",
            "/tmp/escape.md",
            "a/../../escape.md",
            "C:\\escape.md",
            "pack.json",
            "project.json",
            "gap-report.json",
        ] {
            let directory = tempfile::tempdir().unwrap();
            let files = baseline();
            let result = materialize(
                directory.path(),
                "project.json",
                &example("project.json"),
                &example("pack.json"),
                &[(path.to_owned(), b"clause".to_vec())],
                files.iter().map(|(path, bytes)| (path.as_str(), bytes.as_slice())),
            );
            assert!(result.is_err(), "{path} must be refused");
            assert_eq!(count_files(directory.path()), 0, "{path} left a file behind");
        }
    }

    #[test]
    fn an_oversized_project_document_is_refused_before_anything_is_written() {
        let directory = tempfile::tempdir().unwrap();
        let oversized = vec![b' '; usize::try_from(MAX_MANIFEST_BYTES).expect("fits usize") + 1];
        let files = baseline();
        let result = materialize(
            directory.path(),
            "project.json",
            &oversized,
            &example("pack.json"),
            &[],
            files.iter().map(|(path, bytes)| (path.as_str(), bytes.as_slice())),
        );
        assert!(result.is_err());
        assert_eq!(count_files(directory.path()), 0);
    }

    #[test]
    fn snapshot_inputs_and_plan_equal_a_direct_load_of_the_checked_in_example() {
        let directory = tempfile::tempdir().unwrap();
        let files = baseline();
        let manifest = materialize(
            directory.path(),
            "project.json",
            &example("project.json"),
            &example("pack.json"),
            &[("clause.md".to_owned(), example("clause.md"))],
            files.iter().map(|(path, bytes)| (path.as_str(), bytes.as_slice())),
        )
        .unwrap();
        let (snapshot_loaded, snapshot_plan) = load_project(&manifest).unwrap();
        let direct = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/authoring/project.json");
        let (direct_loaded, direct_plan) = load_project(&direct).unwrap();
        assert_eq!(render_json(&snapshot_plan).unwrap(), render_json(&direct_plan).unwrap());
        assert_eq!(
            serde_json::to_value(&snapshot_loaded.inputs).unwrap(),
            serde_json::to_value(&direct_loaded.inputs).unwrap()
        );
        assert_eq!(snapshot_plan.counts, direct_plan.counts);
    }
}
