//! Crate-level `forge author reuse` entry point.
//!
//! Retrieval only: this function captures operator-supplied sources, ranks
//! their blocks against the unresolved authoring sections, and emits a report
//! of pointers. It never writes into the project, plan or pack.

use std::path::{Component, Path, PathBuf};

use crate::ForgeError;
use crate::cli::AuthorReportFormat;
use crate::hashing::sha256_hex;

use super::capture::capture;
use super::corpus::{Corpus, MAX_CORPUS_BYTES};
use super::rank::{MAX_CANDIDATE_LIMIT, RankOptions, SectionQuery, rank_sections};
use super::report::{ReuseCandidate, ReuseInput, ReuseReport, ReuseSection, ReuseSpan};

fn error(message: impl Into<String>) -> ForgeError {
    ForgeError::Authoring(message.into())
}

/// Produce a deterministic reuse report for one authoring project and corpus.
///
/// Writes the selected format to stdout and, when `output_dir` is given,
/// publishes a new generation beneath the project root containing `reuse.json`
/// and `reuse.txt`. Returns whether any section still lacks a candidate or has
/// only low-confidence candidates.
///
/// # Errors
///
/// Returns an authoring error for invalid arguments, invalid input contracts,
/// capture failures, unsafe output, or publication failure.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub fn execute(
    manifest: &Path,
    corpus_path: &Path,
    format: &AuthorReportFormat,
    output_dir: Option<&Path>,
    max_candidates: usize,
    min_score: f64,
    include_draft: bool,
    html: bool,
) -> Result<bool, ForgeError> {
    if max_candidates == 0 || max_candidates > MAX_CANDIDATE_LIMIT {
        return Err(error(format!("--max-candidates must be between 1 and {MAX_CANDIDATE_LIMIT}")));
    }
    if !min_score.is_finite() || min_score < 0.0 {
        return Err(error("--min-score must be a finite non-negative number"));
    }
    let seed = crate::authoring::reuse_seed(manifest)?;
    let (root, manifest_name) = corpus_location(corpus_path)?;
    let corpus_bytes = read_manifest(&root, &manifest_name)?;
    let corpus = Corpus::parse(&corpus_bytes)?;
    let captured = capture(&root, &corpus)?;

    let mut queries = Vec::new();
    let mut targets = Vec::new();
    for policy in &seed.plan.policies {
        for section in &policy.sections {
            if section.state == crate::authoring::model::DraftState::HumanDraftPresent {
                continue;
            }
            queries.push(SectionQuery::from_section(section, &seed.prompts));
            targets.push((policy.policy_key.as_str(), section));
        }
    }
    let ranked = rank_sections(
        &corpus,
        &captured,
        &queries,
        &RankOptions { include_draft, max_candidates, min_score },
    )?;

    let sections = targets
        .iter()
        .zip(ranked)
        .map(|((policy_key, section), candidates)| {
            let candidates = candidates
                .into_iter()
                .map(|candidate| {
                    Ok(ReuseCandidate {
                        source_key: candidate.source_key,
                        source_path: candidate
                            .source_path
                            .to_str()
                            .ok_or_else(|| error("candidate source path must be UTF-8"))?
                            .to_owned(),
                        source_sha256: candidate.source_sha256,
                        span: ReuseSpan { start: candidate.start, end: candidate.end },
                        score: candidate.score,
                        reasons: candidate.reasons,
                        low_confidence: candidate.low_confidence,
                    })
                })
                .collect::<Result<Vec<_>, ForgeError>>()?;
            Ok(ReuseSection {
                policy_key: (*policy_key).to_owned(),
                topic_key: section.topic_key.clone(),
                title: section.title.clone(),
                state: section.state,
                gap_ids: section.gap_ids.clone(),
                control_ids: section.control_ids.clone(),
                candidates,
            })
        })
        .collect::<Result<Vec<_>, ForgeError>>()?;

    let report = ReuseReport::new(
        seed.plan.project_key.clone(),
        seed.plan.as_of.clone(),
        sha256_hex(&corpus_bytes),
        merge_inputs(&seed.plan.provenance.inputs, &captured),
        sections,
    );

    let json = report.render_json()?;
    let text = report.render_text();
    if html && output_dir.is_none() {
        return Err(error("HTML output requires --output-dir"));
    }
    let mut artifacts = vec![
        crate::authoring::output::OutputArtifact {
            relative_path: "reuse.json".to_owned(),
            bytes: json.clone(),
        },
        crate::authoring::output::OutputArtifact {
            relative_path: "reuse.txt".to_owned(),
            bytes: text.clone().into_bytes(),
        },
    ];
    if html {
        artifacts.push(crate::authoring::output::OutputArtifact {
            relative_path: "reuse.html".to_owned(),
            bytes: crate::authoring::html::render_closed_bounded(
                "Authoring reuse",
                &json,
                super::report::REUSE_SCHEMA_VERSION,
                super::report::MAX_REPORT_BYTES,
            )?,
        });
    }
    // Every captured input is revalidated after rendering and immediately
    // before any output side effect — stdout and publication alike — so a
    // source changed during rendering or ranking is never reported or written.
    verify_sources(&seed.captures, &captured, &root, &manifest_name, &corpus_bytes)?;
    if let Some(destination) = output_dir {
        crate::authoring::output::publish(&seed.root, destination, &artifacts)?;
    }
    let stdout = match format {
        AuthorReportFormat::Text => text.as_str(),
        AuthorReportFormat::Json => std::str::from_utf8(&json)
            .map_err(|cause| error(format!("reuse report JSON encoding: {cause}")))?,
    };
    crate::cli::output::write_output(stdout, None)
        .map_err(|cause| error(format!("cannot write reuse report: {cause}")))?;
    Ok(report.action_required())
}

/// Revalidate the project snapshot, the corpus documents and the manifest.
fn verify_sources(
    project_captures: &crate::authoring::input::CaptureSet,
    captured_corpus: &super::capture::CapturedCorpus,
    root: &Path,
    manifest_name: &Path,
    corpus_bytes: &[u8],
) -> Result<(), ForgeError> {
    project_captures.verify()?;
    captured_corpus.verify()?;
    if read_manifest(root, manifest_name)? != corpus_bytes {
        return Err(error("reuse corpus manifest changed after capture"));
    }
    Ok(())
}

/// Merge project pin fingerprints with captured corpus documents, path-sorted.
fn merge_inputs(
    project_inputs: &[crate::authoring::model::InputFingerprint],
    captured: &super::capture::CapturedCorpus,
) -> Vec<ReuseInput> {
    let mut inputs: Vec<ReuseInput> = project_inputs
        .iter()
        .map(|input| ReuseInput {
            role: input.role.clone(),
            path: input.path.clone(),
            sha256: input.sha256.clone(),
            byte_length: input.byte_length,
        })
        .collect();
    for document in &captured.documents {
        inputs.push(ReuseInput {
            role: format!("reuse-document-{}", document.key),
            path: document.path.to_string_lossy().into_owned(),
            sha256: document.sha256.clone(),
            byte_length: u64::try_from(document.bytes.len()).unwrap_or(u64::MAX),
        });
    }
    inputs.sort_by(|left, right| (&left.role, &left.path).cmp(&(&right.role, &right.path)));
    inputs
}

/// Read the corpus manifest through the confined, no-follow, bounded reader.
///
/// The manifest is resolved exactly like every other authoring input: a
/// regular file below an absolute normalized root, never a symlink, device,
/// FIFO or directory, and never more than [`MAX_CORPUS_BYTES`] bytes.
fn read_manifest(root: &Path, name: &Path) -> Result<Vec<u8>, ForgeError> {
    let (bytes, _identity) = crate::linkage::read_confined_local_file(root, name, MAX_CORPUS_BYTES)
        .map_err(|cause| error(format!("cannot read the reuse corpus manifest: {cause}")))?;
    Ok(bytes)
}

/// The corpus manifest's absolute parent directory and file name.
///
/// The parent directory anchors every document path, and the name keeps the
/// confined read a normalized descendant.
fn corpus_location(path: &Path) -> Result<(PathBuf, PathBuf), ForgeError> {
    let base = if path.is_absolute() {
        PathBuf::new()
    } else {
        std::env::current_dir()
            .map_err(|cause| error(format!("cannot determine current directory: {cause}")))?
    };
    let mut absolute = base;
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => return Err(error("corpus path must not contain '..'")),
            _ => absolute.push(component.as_os_str()),
        }
    }
    let root = absolute
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| error("corpus manifest must have a parent directory"))?;
    let name = absolute
        .file_name()
        .map(PathBuf::from)
        .ok_or_else(|| error("corpus manifest must name a file"))?;
    Ok((root, name))
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::hashing::sha256_hex;

    /// A changed source must be rejected by the shared verification step that
    /// both the stdout and publication paths invoke after rendering.
    #[test]
    fn changed_sources_are_rejected_before_output() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let original = b"# A\n\nbody\n";
        std::fs::write(root.join("doc.md"), original).unwrap();
        std::fs::write(root.join("corpus.json"), b"{}").unwrap();
        let mut project_captures = crate::authoring::input::CaptureSet::new(root.clone());
        let bytes = project_captures
            .read("reuse-document-doc", Path::new("doc.md"), Some(&sha256_hex(original)), 1024)
            .unwrap();
        let captured_corpus = super::super::capture::CapturedCorpus {
            root: root.clone(),
            documents: vec![super::super::capture::CapturedDocument {
                key: "doc".to_owned(),
                path: PathBuf::from("doc.md"),
                sha256: sha256_hex(&bytes),
                bytes,
            }],
        };
        let manifest = Path::new("corpus.json");
        assert!(
            verify_sources(&project_captures, &captured_corpus, &root, manifest, b"{}").is_ok()
        );

        std::fs::write(root.join("doc.md"), b"# A\n\nchanged\n").unwrap();
        assert!(
            verify_sources(&project_captures, &captured_corpus, &root, manifest, b"{}").is_err()
        );

        std::fs::write(root.join("doc.md"), original).unwrap();
        assert!(
            verify_sources(
                &project_captures,
                &captured_corpus,
                &root,
                manifest,
                b"{\"changed\":true}"
            )
            .is_err()
        );
    }
}
