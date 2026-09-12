//! Crate-level `forge suggest promote`: propose, never approve.
//!
//! Promotion turns accepted suggestions into a proposed downstream patch. The
//! proposal is validated with the destination's own contract validator before
//! it is published, records its own unapproved status, and is written beside
//! the destination rather than into it: applying the patch stays a human act.

use std::path::{Path, PathBuf};

use crate::ForgeError;
use crate::authoring::manifest::{HumanClause, PROJECT_SCHEMA_VERSION, PinnedFile, Review};
use crate::cli::AuthorReportFormat;

use super::bundle::{Suggestion, SuggestionsBundle};
use super::disposition::{DispositionManifest, DispositionRecord};
use super::promotion::{
    Destination, DestinationKind, MAX_PATCH_BYTES, PatchArtifact, PromotionEntry,
    PromotionProposal, PromotionStatus,
};
use super::review;
use super::shared;
use super::task::TaskKind;

/// Artifact name of the promotion proposal.
pub const PROMOTION_ARTIFACT: &str = "promotion.json";
/// Artifact name of the proposed destination document.
pub const PROPOSED_PATCH_ARTIFACT: &str = "promotion/proposed-project.json";
/// Directory holding one proposed clause file per promoted suggestion.
pub const ENTRY_DIRECTORY: &str = "promotion";

/// Arguments for one promote run.
pub struct PromoteArgs<'a> {
    /// The quarantine bundle the decisions were made about.
    pub bundle: &'a Path,
    /// The prepared request the bundle was validated from, so a proposed clause
    /// cites the gaps that request selected.
    pub request: &'a Path,
    /// The disposition record published by `forge suggest review`.
    pub dispositions: &'a Path,
    /// The destination `forge.author-project/1` document, relative to the working directory.
    pub destination: &'a Path,
    /// New directory beneath the destination's directory.
    pub output_dir: &'a Path,
    /// Stdout format.
    pub format: AuthorReportFormat,
}

/// Propose a downstream patch for the accepted suggestions.
///
/// Returns `true` when nothing was accepted, which the CLI maps to the
/// action-required exit code.
///
/// # Errors
/// Returns an authoring error for an unreadable or invalid input, a disposition
/// record that does not bind this bundle, an unsupported task, a destination
/// that is not portable, a proposed document the destination contract rejects,
/// or publication failure.
pub fn execute(args: &PromoteArgs<'_>) -> Result<bool, ForgeError> {
    let bundle_bytes = crate::io::read_bounded(args.bundle, super::bundle::MAX_BUNDLE_BYTES)?;
    let bundle = SuggestionsBundle::parse(&bundle_bytes)?;
    // Read once: the bytes that `bind` authorises are the bytes the proposal
    // cites, so a later read cannot disagree with what was validated.
    let dispositions_bytes =
        crate::io::read_bounded(args.dispositions, super::disposition::MAX_DISPOSITIONS_BYTES)?;
    let manifest = DispositionManifest::parse(&dispositions_bytes)?;
    review::bind(&bundle, &manifest, &bundle_bytes)?;

    // The request supplies the gap references the destination requires. FORGE
    // never invents one: a clause is proposed only for a section that selected
    // at least one gap.
    let request_bytes = crate::io::read_bounded(args.request, super::request::MAX_REQUEST_BYTES)?;
    let request = super::request::SuggestRequest::parse(&request_bytes)?;
    if crate::hashing::sha256_hex(&request_bytes) != bundle.request.sha256 {
        return Err(shared::error(
            "the request does not match the bundle; pass the request the bundle was validated from",
        ));
    }

    if bundle.task.kind != TaskKind::PolicyDrafting {
        return Err(shared::error(
            "promotion of the mapping-candidate task has no selected destination semantics yet; the owner \
             deferred task selection on 2026-09-12, so only a policy-drafting bundle can be promoted",
        ));
    }
    shared::relative_path("promote --destination", &portable(args.destination)?)?;

    let promoted = accepted(&bundle, &manifest);
    if promoted.is_empty() {
        // Every other exit path honours the requested format; a JSON caller must
        // not receive prose.
        let outcome = serde_json::json!({
            "schema_version": "forge.suggest-promotion-outcome/1",
            "outcome": "nothing-to-promote",
            "accepted": 0,
        });
        let message = match args.format {
            AuthorReportFormat::Json => format!(
                "{}\n",
                serde_json::to_string(&outcome).map_err(|cause| shared::error(format!(
                    "cannot encode the outcome: {cause}"
                )))?
            ),
            AuthorReportFormat::Text => {
                "forge suggest promote: no suggestion has an accept-as-is or \
                 accept-edited disposition, so there is nothing to propose.\n"
                    .to_string()
            }
        };
        crate::cli::output::write_output(&message, None)
            .map_err(|cause| shared::error(format!("cannot write the promote summary: {cause}")))?;
        return Ok(true);
    }

    let destination_path = args.destination.to_string_lossy().into_owned();
    let destination_bytes = crate::io::read_bounded(args.destination, MAX_PATCH_BYTES)?;
    let mut project =
        crate::authoring::manifest::parse_project(&destination_bytes).map_err(|cause| {
            shared::error(format!(
                "the destination is not a valid {PROJECT_SCHEMA_VERSION} document: {cause}"
            ))
        })?;

    let (entries, mut artifacts) = propose_entries(&request, &promoted, &mut project)?;

    let patch_bytes = serde_json::to_vec_pretty(&project)
        .map_err(|cause| shared::error(format!("cannot encode the proposed project: {cause}")))?;
    if patch_bytes.len() as u64 > MAX_PATCH_BYTES {
        return Err(shared::error(format!(
            "the proposed project exceeds the {MAX_PATCH_BYTES} byte limit"
        )));
    }
    // The destination's own validator must accept the proposed document.
    crate::authoring::manifest::parse_project(&patch_bytes).map_err(|cause| {
        shared::error(format!(
            "the proposed destination document does not pass its own contract: {cause}"
        ))
    })?;
    artifacts.push(crate::authoring::output::OutputArtifact {
        relative_path: PROPOSED_PATCH_ARTIFACT.to_string(),
        bytes: patch_bytes.clone(),
    });

    let (proposal, proposal_json) = assemble_proposal(
        &ProposalContext {
            bundle: &bundle,
            bundle_bytes: &bundle_bytes,
            dispositions_bytes: &dispositions_bytes,
            destination_path,
            destination_bytes: &destination_bytes,
            patch_bytes: &patch_bytes,
        },
        entries,
    )?;
    artifacts.push(crate::authoring::output::OutputArtifact {
        relative_path: PROMOTION_ARTIFACT.to_string(),
        bytes: proposal_json.clone(),
    });

    let root = shared::document_root(args.destination, "--destination")?;
    crate::authoring::output::publish(&root, args.output_dir, &artifacts)?;

    let stdout = match args.format {
        AuthorReportFormat::Json => String::from_utf8(proposal_json)
            .map_err(|cause| shared::error(format!("promotion JSON encoding: {cause}")))?,
        AuthorReportFormat::Text => render_summary(&proposal, args),
    };
    crate::cli::output::write_output(&stdout, None)
        .map_err(|cause| shared::error(format!("cannot write the promote summary: {cause}")))?;
    Ok(false)
}

/// Accepted suggestions with the decision that accepted them, in bundle order.
fn accepted<'a>(
    bundle: &'a SuggestionsBundle,
    manifest: &'a DispositionManifest,
) -> Vec<(&'a Suggestion, &'a DispositionRecord)> {
    bundle
        .suggestions
        .iter()
        .filter_map(|suggestion| {
            manifest
                .records
                .iter()
                .find(|record| {
                    record.suggestion_id == suggestion.suggestion_id && record.status.promotes()
                })
                .map(|record| (suggestion, record))
        })
        .collect()
}

/// The clause text to propose: the reviewer's edit when there is one.
fn promoted_text(
    suggestion: &Suggestion,
    record: &DispositionRecord,
) -> Result<String, ForgeError> {
    let clause = suggestion
        .body
        .drafting
        .as_ref()
        .ok_or_else(|| shared::error("a policy-drafting bundle must carry draft clauses"))?;
    if let Some(edited) = &record.edited {
        let edited = edited
            .drafting
            .as_ref()
            .ok_or_else(|| shared::error("an edited acceptance must edit the draft clause"))?;
        return Ok(edited.draft_text.clone());
    }
    Ok(clause.draft_text.clone())
}

/// A proposed clause file is the clause text and exactly one trailing newline.
fn clause_file(text: &str) -> Vec<u8> {
    let mut bytes = text.as_bytes().to_vec();
    if !bytes.ends_with(b"\n") {
        bytes.push(b'\n');
    }
    bytes
}

/// What one proposal binds together; grouped so the assembler stays readable.
struct ProposalContext<'a> {
    bundle: &'a SuggestionsBundle,
    bundle_bytes: &'a [u8],
    dispositions_bytes: &'a [u8],
    destination_path: String,
    destination_bytes: &'a [u8],
    patch_bytes: &'a [u8],
}

/// Build the closed proposal and prove it passes its own contract.
fn assemble_proposal(
    context: &ProposalContext<'_>,
    entries: Vec<PromotionEntry>,
) -> Result<(PromotionProposal, Vec<u8>), ForgeError> {
    let proposal = PromotionProposal {
        schema_version: super::promotion::SCHEMA_VERSION.to_string(),
        bundle_id: context.bundle.bundle_id.clone(),
        bundle_sha256: crate::hashing::sha256_hex(context.bundle_bytes),
        dispositions_sha256: crate::hashing::sha256_hex(context.dispositions_bytes),
        destination: Destination {
            kind: DestinationKind::AuthoringProject,
            path: context.destination_path.clone(),
            expected_sha256: crate::hashing::sha256_hex(context.destination_bytes),
        },
        patch: PatchArtifact {
            artifact: PROPOSED_PATCH_ARTIFACT.to_string(),
            sha256: crate::hashing::sha256_hex(context.patch_bytes),
            bytes: u64::try_from(context.patch_bytes.len()).map_err(|_| {
                shared::error("the proposed project length does not fit the contract")
            })?,
        },
        status: PromotionStatus::ProposedUnapproved,
        entries,
    };
    let json = serde_json::to_vec_pretty(&proposal)
        .map_err(|cause| shared::error(format!("cannot encode the promotion proposal: {cause}")))?;
    PromotionProposal::parse(&json)?;
    Ok((proposal, json))
}

/// Build one proposed clause per promoted suggestion and extend the project.
fn propose_entries(
    request: &super::request::SuggestRequest,
    promoted: &[(&Suggestion, &DispositionRecord)],
    project: &mut crate::authoring::manifest::AuthorProject,
) -> Result<(Vec<PromotionEntry>, Vec<crate::authoring::output::OutputArtifact>), ForgeError> {
    let mut entries = Vec::new();
    let mut artifacts = Vec::new();
    for (suggestion, record) in promoted {
        let clause =
            suggestion.body.drafting.as_ref().ok_or_else(|| {
                shared::error("a policy-drafting bundle must carry draft clauses")
            })?;
        let text = promoted_text(suggestion, record)?;
        let gap_ids = section_gaps(request, &clause.policy_key, &clause.topic_key)?;
        // Paths derive from the full suggestion identity, so a later promotion
        // cannot reuse a path an earlier, applied proposal already pinned.
        let relative = format!("{ENTRY_DIRECTORY}/entry-{}.md", suggestion.suggestion_id);
        let bytes = clause_file(&text);
        let bytes_len = u64::try_from(bytes.len()).map_err(|_| {
            shared::error("a proposed clause file length does not fit the contract")
        })?;
        let sha256 = crate::hashing::sha256_hex(&bytes);
        project.human_clauses.push(HumanClause {
            key: clause_key(&suggestion.suggestion_id),
            policy_key: clause.policy_key.clone(),
            topic_key: clause.topic_key.clone(),
            gap_ids,
            answer_refs: Vec::new(),
            source: PinnedFile { path: PathBuf::from(&relative), expected_sha256: sha256.clone() },
            review: Review {
                reviewer_key: record.reviewer_key.clone(),
                reviewed_at: record.decided_as_of.clone(),
                rationale: record.rationale.clone(),
            },
        });
        artifacts.push(crate::authoring::output::OutputArtifact {
            relative_path: relative.clone(),
            bytes,
        });
        entries.push(PromotionEntry {
            suggestion_id: suggestion.suggestion_id.clone(),
            contract: PROJECT_SCHEMA_VERSION.to_string(),
            artifact: relative,
            sha256,
            bytes: bytes_len,
        });
    }
    Ok((entries, artifacts))
}

/// The gaps the prepared request selected for one clause's section.
fn section_gaps(
    request: &super::request::SuggestRequest,
    policy_key: &str,
    topic_key: &str,
) -> Result<Vec<String>, ForgeError> {
    let section = request
        .task
        .drafting_sections
        .iter()
        .find(|section| section.policy_key == policy_key && section.topic_key == topic_key)
        .ok_or_else(|| {
            shared::error(format!(
                "the request supplies no drafting section for '{policy_key}/{topic_key}'"
            ))
        })?;
    if section.gap_ids.is_empty() {
        return Err(shared::error(format!(
            "subsection '{policy_key}/{topic_key}' selected no gap, and the destination requires a gap reference"
        )));
    }
    Ok(section.gap_ids.clone())
}

/// A stable clause key derived from the suggestion identifier.
fn clause_key(suggestion_id: &str) -> String {
    format!("suggested-{suggestion_id}")
}

/// The portable relative spelling of the destination document.
///
/// Windows operators write `dir\\project.json`; the contract is `/`-separated and
/// rejects backslashes, so the supplied spelling is normalized before it is
/// validated and recorded. Absolute paths and `..` are still refused.
fn portable(path: &Path) -> Result<String, ForgeError> {
    path.to_str()
        .map(|text| text.replace('\\', "/"))
        .ok_or_else(|| shared::error("promote --destination must be UTF-8"))
}

/// The text summary for one promotion.
fn render_summary(proposal: &PromotionProposal, args: &PromoteArgs<'_>) -> String {
    use std::fmt::Write as _;
    let mut summary = String::new();
    let _ = writeln!(summary, "proposed {} clause(s)", proposal.entries.len());
    let _ = writeln!(
        summary,
        "destination: {} ({})",
        proposal.destination.path,
        proposal.destination.kind.contract()
    );
    let _ = writeln!(summary, "proposed document: {}", proposal.patch.artifact);
    let _ = writeln!(summary, "status: {}", proposal.status.as_str());
    let _ = writeln!(summary, "output: {}", args.output_dir.display());
    let _ = writeln!(
        summary,
        "apply by hand, then run forge author plan/build; this proposal approves nothing"
    );
    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suggest::task::drafting::DraftClause;

    fn drafting_clause(text: &str) -> DraftClause {
        DraftClause {
            policy_key: "access-policy".to_string(),
            topic_key: "access-topic".to_string(),
            question_key: None,
            heading: None,
            draft_text: text.to_string(),
            citations: vec![crate::suggest::Citation {
                unit_id: "unit-0001".to_string(),
                quote: None,
            }],
            assumptions: Vec::new(),
            unresolved_questions: Vec::new(),
        }
    }

    #[test]
    fn clause_keys_are_portable_and_derived_from_the_identifier() {
        // The full identifier, so two suggestions cannot collide on a prefix.
        let key = clause_key("3f2b1a4c-5d6e-4f70-8a91-b2c3d4e5f607");
        assert_eq!(key, "suggested-3f2b1a4c-5d6e-4f70-8a91-b2c3d4e5f607");
        assert!(key.len() <= 64);
        assert_ne!(key, clause_key("3f2b1a4c-0000-4000-8000-000000000000"));
        assert!(
            key.bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        );
    }

    #[test]
    fn a_clause_must_map_to_a_requested_section_that_selected_a_gap() {
        let request = crate::suggest::request::fixture_request();
        // The fixture selects one gap, so its own section resolves.
        assert_eq!(
            section_gaps(&request, "access-policy", "access-control").unwrap(),
            vec!["gap-0001".to_string()]
        );
        assert!(section_gaps(&request, "invented-policy", "invented-topic").is_err());

        // A requested section that selected no gap cannot become a clause.
        let mut gapless = crate::suggest::request::fixture_request_json();
        gapless["task"]["drafting_sections"][0]["gap_ids"] = serde_json::json!([]);
        let gapless =
            crate::suggest::SuggestRequest::parse(&serde_json::to_vec(&gapless).unwrap()).unwrap();
        assert!(section_gaps(&gapless, "access-policy", "access-control").is_err());
    }

    #[test]
    fn the_destination_spelling_is_normalized_to_portable_separators() {
        assert_eq!(portable(Path::new("dir\\project.json")).unwrap(), "dir/project.json");
        assert_eq!(portable(Path::new("project.json")).unwrap(), "project.json");
        // Escaping is refused later by the contract validator, not here.
        assert_eq!(portable(Path::new("dir/../project.json")).unwrap(), "dir/../project.json");
    }

    #[test]
    fn a_clause_file_always_ends_in_exactly_one_newline() {
        assert_eq!(clause_file("plain"), b"plain\n");
        assert_eq!(clause_file("already\n"), b"already\n");
        assert_eq!(clause_file("two\n\n"), b"two\n\n");
    }

    #[test]
    fn an_edited_acceptance_proposes_the_edit_and_an_untouched_one_the_original() {
        let suggestion = crate::suggest::bundle::Suggestion {
            suggestion_id: "3f2b1a4c-5d6e-4f70-8a91-b2c3d4e5f607".to_string(),
            content_sha256: "a".repeat(64),
            evidence_support: crate::suggest::EvidenceSupport::High,
            body: crate::suggest::SuggestionBody {
                mapping: None,
                drafting: Some(drafting_clause("the original text")),
            },
        };
        let record = |edited: Option<crate::suggest::SuggestionBody>| DispositionRecord {
            suggestion_id: suggestion.suggestion_id.clone(),
            status: if edited.is_some() {
                crate::suggest::DispositionStatus::AcceptEdited
            } else {
                crate::suggest::DispositionStatus::AcceptAsIs
            },
            reviewer_key: "human".to_string(),
            decided_as_of: "2026-09-12T00:00:00Z".to_string(),
            rationale: "Reviewed.".to_string(),
            original_sha256: suggestion.content_sha256.clone(),
            edited,
            edited_sha256: None,
        };
        assert_eq!(promoted_text(&suggestion, &record(None)).unwrap(), "the original text");
        let edited = crate::suggest::SuggestionBody {
            mapping: None,
            drafting: Some(drafting_clause("the reviewed text")),
        };
        assert_eq!(promoted_text(&suggestion, &record(Some(edited))).unwrap(), "the reviewed text");
    }
}
