//! Empty authoring-pack scaffolding from a validated framework inventory.
//!
//! S-1 seeds a `forge.authoring-pack/1` template bound to one author project's
//! exact framework and report fingerprints. The scaffold deliberately creates no
//! topics, questions, assignments or families, and it never fabricates reviewer
//! provenance: the required `reviewers` collection and every `content_rights`
//! reviewer field are emitted empty. The result is therefore intentionally not a
//! valid pack until a human reviewer supplies those fields, mirroring the
//! `forge mapping init` fail-until-attested convention. No approval evidence,
//! lifecycle state, assignment or inferred value is created.

use std::path::Path;

use crate::ForgeError;

use super::error;
use super::input;
use super::manifest::{
    self, AuthorProject, AuthoringPack, ContentRights, MAX_MANIFEST_BYTES, PACK_SCHEMA_VERSION,
    Review,
};
use super::output;

/// Scaffold pack version. A reviewer replaces it when the reviewed pack is versioned.
pub const SCAFFOLD_PACK_VERSION: &str = "0.1.0";

/// Build the empty pack bound to an already validated project.
///
/// Deterministic by construction: the key and baseline come from the input
/// project, and no wall-clock time, locale, environment or path is consulted.
#[must_use]
pub fn empty_pack(project: &AuthorProject) -> AuthoringPack {
    AuthoringPack {
        schema_version: PACK_SCHEMA_VERSION.to_owned(),
        pack_key: project.project_key.clone(),
        version: SCAFFOLD_PACK_VERSION.to_owned(),
        baseline: project.baseline.clone(),
        reviewers: Vec::new(),
        content_rights: ContentRights {
            source_label: String::new(),
            statement: String::new(),
            review: Review {
                reviewer_key: String::new(),
                reviewed_at: String::new(),
                rationale: String::new(),
            },
        },
        topics: Vec::new(),
        policy_families: Vec::new(),
        questions: Vec::new(),
        control_assignments: Vec::new(),
        family_assignments: Vec::new(),
    }
}

/// Serialize a scaffold pack as deterministic pretty JSON with a trailing newline.
///
/// # Errors
/// Returns an authoring error if serialization fails or the bounded manifest
/// limit would be exceeded.
pub fn render(pack: &AuthoringPack) -> Result<Vec<u8>, ForgeError> {
    let mut bytes = serde_json::to_vec_pretty(pack)
        .map_err(|cause| error(format!("scaffold serialization failed: {cause}")))?;
    bytes.push(b'\n');
    if bytes.len() as u64 > MAX_MANIFEST_BYTES {
        return Err(error("scaffold pack exceeds the 2 MiB manifest limit"));
    }
    Ok(bytes)
}

/// Write an empty pack to the project's pinned `authoring_pack` path.
///
/// The project and framework inventory must already be valid, exactly regenerated
/// inputs. The destination is never replaced and no assignment is created.
///
/// # Errors
/// Returns an authoring error for invalid or stale inputs, an unsafe destination,
/// an existing destination, or publication failure.
pub fn execute(manifest_path: &Path) -> Result<(), ForgeError> {
    let prepared = input::prepare_inventory(manifest_path)?;
    let destination = prepared.project.authoring_pack.path.clone();
    manifest::validate_local_path("authoring_pack.path", &destination)?;
    let bytes = render(&empty_pack(&prepared.project))?;
    // Revalidate every captured source through the confined traversal before publishing.
    prepared.verify_inputs()?;
    output::publish_new_file(&prepared.root, &destination, &bytes)
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    fn project() -> AuthorProject {
        serde_json::from_value(json!({
            "schema_version": "forge.author-project/1",
            "project_key": "synthetic-project",
            "project_root": ".",
            "baseline": {
                "framework_sha256": "a".repeat(64),
                "report_sha256": "b".repeat(64)
            },
            "applicability_manifest": {
                "path": "applicability.json",
                "expected_sha256": "c".repeat(64)
            },
            "gap_report": {"path": "gap-report.json", "expected_sha256": "b".repeat(64)},
            "authoring_pack": {"path": "pack.json", "expected_sha256": "d".repeat(64)},
            "as_of": "2026-09-08T00:00:00Z",
            "reviewers": [{"key": "human", "name": "Synthetic Reviewer"}],
            "baseline_review": {
                "reviewer_key": "human",
                "reviewed_at": "2026-09-01T00:00:00Z",
                "rationale": "Explicit synthetic fixture decision."
            },
            "policies": [],
            "answers": [],
            "deferrals": [],
            "human_clauses": []
        }))
        .unwrap()
    }

    #[test]
    fn empty_pack_binds_the_project_baseline_and_creates_no_records() {
        let project = project();
        let pack = empty_pack(&project);
        assert_eq!(pack.schema_version, PACK_SCHEMA_VERSION);
        assert_eq!(pack.pack_key, project.project_key);
        assert_eq!(pack.version, SCAFFOLD_PACK_VERSION);
        assert_eq!(pack.baseline, project.baseline);
        assert!(pack.reviewers.is_empty());
        assert!(pack.topics.is_empty());
        assert!(pack.policy_families.is_empty());
        assert!(pack.questions.is_empty());
        assert!(pack.control_assignments.is_empty());
        assert!(pack.family_assignments.is_empty());
        assert_eq!(pack.content_rights.review.reviewer_key, "");
        assert_eq!(pack.content_rights.review.reviewed_at, "");
        assert_eq!(pack.content_rights.review.rationale, "");
    }

    #[test]
    fn rendered_scaffold_is_deterministic_and_closed() {
        let first = render(&empty_pack(&project())).unwrap();
        let second = render(&empty_pack(&project())).unwrap();
        assert_eq!(first, second);
        assert!(first.ends_with(b"\n"));
        let value: Value = serde_json::from_slice(&first).unwrap();
        let mut keys: Vec<&str> = value.as_object().unwrap().keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            [
                "baseline",
                "content_rights",
                "control_assignments",
                "family_assignments",
                "pack_key",
                "policy_families",
                "questions",
                "reviewers",
                "schema_version",
                "topics",
                "version",
            ]
        );
    }

    #[test]
    fn rendered_scaffold_requires_reviewer_provenance_before_use() {
        // The required provenance fields are present but empty, so the scaffold is
        // intentionally rejected until a reviewer supplies them.
        let bytes = render(&empty_pack(&project())).unwrap();
        assert!(manifest::parse_pack(&bytes).is_err());
    }
}
