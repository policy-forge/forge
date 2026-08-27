//! OSCAL implemented-requirements builder: maps [`crate::model::PolicyRequirement`]s to
//! OSCAL `implemented-requirement` JSON entries within a `control-implementations` array.
//!
//! Walks the [`crate::model::PolicyDocument`]'s section tree depth-first (matching the Catalog
//! builder's traversal order) and produces one `implemented-requirement` per
//! [`crate::model::PolicyRequirement`] with a deterministic UUID v5 and a `control-id` matching
//! the Catalog builder's scheme (e.g., `POL-AC-001`).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ForgeError;
use crate::model::{PolicyDocument, PolicyRequirement};
use crate::oscal::back_matter::OscalLink;
use crate::oscal::catalog::{
    collect_requirements_with_section, generate_control_id, resolve_abbreviation,
};
use crate::oscal::parts::OscalProp;
use crate::oscal::trace_embedding::{build_trace_link, build_trace_props};
use crate::uuid::{CONTROL_IMPL_NAMESPACE, IMPL_REQ_NAMESPACE};

// ─── Typed Structs ──────────────────────────────────────────────────────

/// A single control-implementation entry within a Component Definition.
///
/// Contains the source profile reference and all implemented requirements
/// derived from the policy document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlImplementation {
    /// Deterministic UUID v5 for the control-implementation entry.
    pub uuid: String,

    /// Source profile reference (e.g., `"./baseline.json"`).
    pub source: String,

    /// Human-readable description of the implementation.
    pub description: String,

    /// The implemented requirements mapped from policy requirements.
    #[serde(rename = "implemented-requirements")]
    pub implemented_requirements: Vec<ImplementedRequirement>,
}

/// A single implemented-requirement within a control-implementation.
///
/// Maps a [`PolicyRequirement`] to its OSCAL representation with
/// traceability metadata (props and links).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementedRequirement {
    /// Deterministic UUID v5 for the implemented-requirement.
    pub uuid: String,

    /// Control ID matching the Catalog builder's scheme (e.g., `"POL-AC-001"`).
    #[serde(rename = "control-id")]
    pub control_id: String,

    /// Implementation narrative (raw requirement text or placeholder).
    pub description: String,

    /// Trace properties (source-file, source-section, source-line, modality).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub props: Vec<OscalProp>,

    /// Source links for traceability.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<OscalLink>,
}

/// Build the `control-implementations` array for a Component Definition.
///
/// Walks the document's sections to generate control-ids consistent with the
/// Catalog builder (WI-9). Produces a `Vec<ControlImplementation>` containing
/// one entry with the source profile reference and all implemented-requirements
/// mapped from [`PolicyRequirement`]s.
///
/// # Arguments
/// * `document` - `PolicyDocument` with sections and requirements
/// * `source_profile` - Value of the `--source-profile` CLI flag
/// * `source_file` - Optional path to the source policy file for trace props
///
/// # Returns
/// A `Vec<ControlImplementation>` containing one control-implementations entry.
///
/// # Errors
/// Returns [`ForgeError::ComponentDefinitionBuild`] when a requirement lacks the
/// stable ID required to reference its corresponding catalog control.
pub fn build_control_implementations(
    document: &PolicyDocument,
    source_profile: &str,
    source_file: Option<&str>,
) -> Result<Vec<ControlImplementation>, ForgeError> {
    let mut abbrev_counts: HashMap<String, Vec<String>> = HashMap::new();
    let mut implemented_requirements = Vec::new();
    // Occurrence counter per (stable_id, text) so UUIDs derive from content
    // and stay stable when unrelated requirements are inserted or deleted
    // (F0585); the counter only disambiguates genuinely identical pairs.
    let mut occurrences: HashMap<(String, String), usize> = HashMap::new();

    for section in &document.sections {
        let abbreviation = resolve_abbreviation(&section.title, &mut abbrev_counts);
        let requirements = collect_requirements_with_section(section);

        for (req_idx, (req, req_section)) in requirements.iter().enumerate() {
            let stable_id = req.stable_id.as_deref().ok_or_else(|| {
                let preview: String = req.text.chars().take(60).collect();
                ForgeError::ComponentDefinitionBuild(format!(
                    "Requirement missing stable_id in section '{}': '{preview}'",
                    req_section.title,
                ))
            })?;
            let control_id = generate_control_id(&abbreviation, req_idx, "POL");
            let normalized_text = crate::uuid::normalize_for_hashing(&req.text);
            let occurrence = occurrences
                .entry((stable_id.to_string(), normalized_text))
                .and_modify(|count| *count += 1)
                .or_insert(0);
            let entry = map_requirement_to_implemented(
                req,
                stable_id,
                &control_id,
                *occurrence,
                source_file,
                &req_section.title,
            );
            implemented_requirements.push(entry);
        }
    }

    if implemented_requirements.is_empty() {
        tracing::warn!(
            "Document has zero requirements — control-implementations will have empty implemented-requirements array"
        );
    }

    let title = if document.metadata.title.is_empty() {
        crate::oscal::component_definition::DEFAULT_COMPONENT_TITLE
    } else {
        &document.metadata.title
    };
    let sanitized_source = crate::io::sanitize_artifact_path(std::path::Path::new(source_profile));
    let ci_uuid = generate_control_impl_uuid(source_profile, title);
    let description = format!("Implementation narratives derived from {title}.");

    Ok(vec![ControlImplementation {
        uuid: ci_uuid.to_string(),
        source: sanitized_source,
        description,
        implemented_requirements,
    }])
}

/// Map a single [`PolicyRequirement`] to a typed [`ImplementedRequirement`].
///
/// # Arguments
/// * `requirement` - A single `PolicyRequirement` from the domain model
/// * `stable_id` - Validated stable ID for the requirement
/// * `control_id` - The pre-computed control-id for this requirement
/// * `occurrence` - Ordinal of this (`stable_id`, `text`) pair within the document
/// * `source_file` - Optional path to the source policy file for trace props
/// * `section_title` - Section title containing the requirement (for trace props)
fn map_requirement_to_implemented(
    requirement: &PolicyRequirement,
    stable_id: &str,
    control_id: &str,
    occurrence: usize,
    source_file: Option<&str>,
    section_title: &str,
) -> ImplementedRequirement {
    let uuid = generate_impl_req_uuid(stable_id, &requirement.text, occurrence);

    let description = if requirement.text.is_empty() {
        "No implementation narrative available.".to_string()
    } else {
        requirement.text.clone()
    };

    let source_file = source_file.filter(|path| !path.is_empty());
    let mut props = source_file.map_or_else(Vec::new, |path| {
        build_trace_props(path, section_title, requirement.source_line)
    });
    if let Some(modality) = requirement.modality {
        let modality_value = match modality {
            crate::model::Modality::Normative => "normative",
            crate::model::Modality::Advisory => "advisory",
        };
        props.push(OscalProp {
            name: "modality".to_string(),
            ns: None,
            value: modality_value.to_string(),
        });
    }
    let links = source_file
        .map_or_else(Vec::new, |path| vec![build_trace_link(path, requirement.source_line)]);

    ImplementedRequirement {
        uuid: uuid.to_string(),
        control_id: control_id.to_string(),
        description,
        props,
        links,
    }
}

/// Generate a deterministic UUID v5 for a control-implementation entry.
///
/// Seed format: `"{source_profile}\0{policy_title}"`
///
/// # Arguments
/// * `source_profile` - The baseline profile href reference
/// * `policy_title` - The policy document title
fn generate_control_impl_uuid(source_profile: &str, policy_title: &str) -> Uuid {
    let seed = format!("{source_profile}\0{policy_title}");
    Uuid::new_v5(&CONTROL_IMPL_NAMESPACE, seed.as_bytes())
}

/// Generate a deterministic UUID v5 for an implemented-requirement entry.
///
/// Seed format: `"{stable_id}\0{normalized_text}\0{occurrence}"`
///
/// `occurrence` is the per-document ordinal of this exact (`stable_id`,
/// normalized text) pair: 0 for unique content (the normal case), incremented
/// only for genuine duplicates — so inserting or deleting an unrelated
/// requirement does not re-roll the UUIDs of everything after it (F0585).
///
/// # Arguments
/// * `stable_id` - The requirement's stable ID (UUID string)
/// * `text` - The requirement text; normalized before UUID derivation
/// * `occurrence` - Ordinal of this (`stable_id`, normalized text) pair within the document
fn generate_impl_req_uuid(stable_id: &str, text: &str, occurrence: usize) -> Uuid {
    let normalized_text = crate::uuid::normalize_for_hashing(text);
    let seed = format!("{stable_id}\0{normalized_text}\0{occurrence}");
    Uuid::new_v5(&IMPL_REQ_NAMESPACE, seed.as_bytes())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use uuid::Version;

    use super::*;
    use crate::model::{DocumentMetadata, PolicyDocument, PolicyRequirement, PolicySection};

    // ─── Test Helpers ────────────────────────────────────────────────────

    fn test_metadata() -> DocumentMetadata {
        DocumentMetadata {
            title: "Corporate Security Policy".to_string(),
            version: "1.0".to_string(),
            author: None,
            date: None,
            source_path: PathBuf::from("test.md"),
            content_hash: None,
        }
    }

    fn make_req(text: &str, stable_id: Option<&str>, atom_index: usize) -> PolicyRequirement {
        PolicyRequirement {
            text: text.to_string(),
            source_line: 1,
            nesting_depth: 0,
            stable_id: stable_id.map(String::from),
            atom_index,
            parent_text: None,
            citations: vec![],
            modality: None,
            parameters: vec![],
            parameters_extracted: false,
        }
    }

    fn make_section(
        title: &str,
        reqs: Vec<PolicyRequirement>,
        children: Vec<PolicySection>,
    ) -> PolicySection {
        PolicySection {
            title: title.to_string(),
            heading_level: 1,
            source_line: 1,
            body_text: None,
            requirements: reqs,
            children,
        }
    }

    fn make_doc(sections: Vec<PolicySection>) -> PolicyDocument {
        PolicyDocument { id: "test-doc".to_string(), metadata: test_metadata(), sections }
    }

    // ─── T004: UUID Generation Helper Tests [US3] ────────────────────────

    #[test]
    fn control_impl_uuid_determinism() {
        let uuid1 = generate_control_impl_uuid("./baseline.json", "Corporate Security Policy");
        let uuid2 = generate_control_impl_uuid("./baseline.json", "Corporate Security Policy");
        assert_eq!(uuid1, uuid2, "Same inputs must produce same UUID");
    }

    #[test]
    fn control_impl_uuid_different_inputs() {
        let uuid1 = generate_control_impl_uuid("./baseline.json", "Corporate Security Policy");
        let uuid2 = generate_control_impl_uuid("./other.json", "Corporate Security Policy");
        let uuid3 = generate_control_impl_uuid("./baseline.json", "Different Policy");
        let uuid4 = generate_control_impl_uuid(
            "./baselines/production/baseline.json",
            "Corporate Security Policy",
        );
        let uuid5 =
            generate_control_impl_uuid("./vendor/baseline.json", "Corporate Security Policy");
        assert_ne!(uuid1, uuid2, "Different source_profile must produce different UUID");
        assert_ne!(uuid1, uuid3, "Different policy_title must produce different UUID");
        assert_ne!(uuid4, uuid5, "Distinct original profile paths must remain distinct UUIDs");
    }

    #[test]
    fn control_impl_uuid_is_v5() {
        let uuid = generate_control_impl_uuid("./baseline.json", "Test");
        assert_eq!(uuid.get_version(), Some(Version::Sha1));
        assert_eq!(uuid.get_variant(), uuid::Variant::RFC4122);
    }

    #[test]
    fn impl_req_uuid_determinism() {
        let uuid1 = generate_impl_req_uuid("stable-id-1", "Requirement text", 0);
        let uuid2 = generate_impl_req_uuid("stable-id-1", "Requirement text", 0);
        assert_eq!(uuid1, uuid2, "Same inputs must produce same UUID");
    }

    #[test]
    fn impl_req_uuid_different_inputs() {
        let uuid1 = generate_impl_req_uuid("stable-id-1", "Requirement text", 0);
        let uuid2 = generate_impl_req_uuid("stable-id-2", "Requirement text", 0);
        let uuid3 = generate_impl_req_uuid("stable-id-1", "Different text", 0);
        let uuid4 = generate_impl_req_uuid("stable-id-1", "Requirement text", 1);
        assert_ne!(uuid1, uuid2, "Different stable_id must produce different UUID");
        assert_ne!(uuid1, uuid3, "Different text must produce different UUID");
        assert_ne!(uuid1, uuid4, "Different index must produce different UUID");
    }

    #[test]
    fn impl_req_uuid_is_v5() {
        let uuid = generate_impl_req_uuid("id", "text", 0);
        assert_eq!(uuid.get_version(), Some(Version::Sha1));
        assert_eq!(uuid.get_variant(), uuid::Variant::RFC4122);
    }

    // ─── T005: Catalog-compatible control ID generation [US1] ─────────────

    #[test]
    fn control_id_normal_case() {
        assert_eq!(generate_control_id("AC", 0, "POL"), "POL-AC-001");
    }

    #[test]
    fn control_id_normal_case_second_req() {
        assert_eq!(generate_control_id("DP", 2, "POL"), "POL-DP-003");
    }

    // ─── T006: map_requirement_to_implemented Tests [US1] ────────────────

    #[test]
    fn map_requirement_has_required_fields() {
        let req = make_req("All users must authenticate.", Some("uuid-1"), 0);
        let result = map_requirement_to_implemented(
            &req,
            "uuid-1",
            "POL-AC-001",
            0,
            Some("test.md"),
            "Access Control",
        );

        assert!(!result.uuid.is_empty(), "Must have uuid field");
        assert!(!result.control_id.is_empty(), "Must have control-id field");
        assert!(!result.description.is_empty(), "Must have description field");
    }

    #[test]
    fn map_requirement_uses_raw_text() {
        let req = make_req("All users must authenticate.", Some("uuid-1"), 0);
        let result = map_requirement_to_implemented(
            &req,
            "uuid-1",
            "POL-AC-001",
            0,
            Some("test.md"),
            "Access Control",
        );

        assert_eq!(
            result.description, "All users must authenticate.",
            "FR-008: description must be raw requirement text"
        );
    }

    #[test]
    fn map_requirement_control_id_matches() {
        let req = make_req("Test requirement.", Some("uuid-1"), 0);
        let result = map_requirement_to_implemented(
            &req,
            "uuid-1",
            "POL-AC-001",
            0,
            Some("test.md"),
            "Access Control",
        );

        assert_eq!(result.control_id, "POL-AC-001");
    }

    // ─── T007: build_control_implementations Tests [US1] ─────────────────

    #[test]
    fn build_control_impl_returns_single_entry() {
        let doc = make_doc(vec![
            make_section(
                "Access Control",
                vec![make_req("R1.", Some("id1"), 0), make_req("R2.", Some("id2"), 0)],
                vec![],
            ),
            make_section(
                "Data Protection",
                vec![
                    make_req("R3.", Some("id3"), 0),
                    make_req("R4.", Some("id4"), 0),
                    make_req("R5.", Some("id5"), 0),
                ],
                vec![],
            ),
        ]);

        let result =
            build_control_implementations(&doc, "./baseline.json", Some("test.md")).unwrap();
        assert_eq!(result.len(), 1, "Must produce exactly one control-implementations entry");
    }

    #[test]
    fn build_control_impl_has_required_fields() {
        let doc = make_doc(vec![make_section(
            "Access Control",
            vec![make_req("R1.", Some("id1"), 0)],
            vec![],
        )]);

        let result =
            build_control_implementations(&doc, "./baseline.json", Some("test.md")).unwrap();
        let entry = &result[0];

        assert!(!entry.uuid.is_empty(), "Must have uuid");
        assert!(!entry.source.is_empty(), "Must have source");
        assert!(!entry.description.is_empty(), "Must have description");
        // implemented_requirements field always exists on the struct
    }

    #[test]
    fn build_control_impl_source_uses_filename_only() {
        let doc = make_doc(vec![make_section(
            "Access Control",
            vec![make_req("R1.", Some("id1"), 0)],
            vec![],
        )]);

        let result =
            build_control_implementations(&doc, "./baselines/nist.json", Some("test.md")).unwrap();
        assert_eq!(result[0].source, "nist.json");
    }

    #[test]
    fn control_implementation_source_uses_filename_only() {
        let doc = make_doc(vec![make_section(
            "Access Control",
            vec![make_req("R1.", Some("id1"), 0)],
            vec![],
        )]);

        let result =
            build_control_implementations(&doc, "/absolute/path/to/baseline.json", Some("test.md"))
                .unwrap();
        assert_eq!(result[0].source, "baseline.json");
        assert!(!result[0].source.contains('/'));
    }

    #[test]
    fn build_control_impl_description_pattern() {
        let doc = make_doc(vec![make_section(
            "Access Control",
            vec![make_req("R1.", Some("id1"), 0)],
            vec![],
        )]);

        let result =
            build_control_implementations(&doc, "./baseline.json", Some("test.md")).unwrap();
        assert_eq!(
            result[0].description,
            "Implementation narratives derived from Corporate Security Policy."
        );
    }

    #[test]
    fn build_control_impl_requirement_count_matches() {
        let doc = make_doc(vec![
            make_section(
                "Access Control",
                vec![make_req("R1.", Some("id1"), 0), make_req("R2.", Some("id2"), 0)],
                vec![],
            ),
            make_section(
                "Data Protection",
                vec![
                    make_req("R3.", Some("id3"), 0),
                    make_req("R4.", Some("id4"), 0),
                    make_req("R5.", Some("id5"), 0),
                ],
                vec![],
            ),
        ]);

        let result =
            build_control_implementations(&doc, "./baseline.json", Some("test.md")).unwrap();
        assert_eq!(
            result[0].implemented_requirements.len(),
            5,
            "Must have one implemented-requirement per PolicyRequirement"
        );
    }

    // ─── T018: Zero requirements edge case (EC-1, FR-013) ────────────────

    #[test]
    fn zero_requirements_produces_empty_impl_reqs() {
        let doc = make_doc(vec![make_section("Empty Section", vec![], vec![])]);
        let result =
            build_control_implementations(&doc, "./baseline.json", Some("test.md")).unwrap();
        assert!(
            result[0].implemented_requirements.is_empty(),
            "Zero requirements must produce empty array"
        );
    }

    // ─── T019: Empty requirement text (EC-3, FR-014, SEC-5) ──────────────

    #[test]
    fn empty_text_produces_placeholder_description() {
        let req = make_req("", Some("uuid-1"), 0);
        let result = map_requirement_to_implemented(
            &req,
            "uuid-1",
            "POL-AC-001",
            0,
            Some("test.md"),
            "Access Control",
        );
        assert_eq!(
            result.description, "No implementation narrative available.",
            "Empty text must produce placeholder description"
        );
    }

    // ─── T020: Missing stable_id fallback (EC-2) ─────────────────────────

    #[test]
    fn missing_stable_id_returns_component_definition_error() {
        let doc =
            make_doc(vec![make_section("Access Control", vec![make_req("R1.", None, 0)], vec![])]);

        let error =
            build_control_implementations(&doc, "./baseline.json", Some("test.md")).unwrap_err();
        assert!(matches!(error, ForgeError::ComponentDefinitionBuild(detail)
            if detail.contains("missing stable_id") && detail.contains("Access Control")));
    }

    // ─── T022: Identical text, different positions produce distinct UUIDs (EC-5) ──

    #[test]
    fn identical_text_different_positions_distinct_uuids() {
        let req1 = make_req("Same requirement text.", Some("id-1"), 0);
        let req2 = make_req("Same requirement text.", Some("id-2"), 1);

        let result1 = map_requirement_to_implemented(
            &req1,
            "id-1",
            "POL-AC-001",
            0,
            Some("test.md"),
            "Access Control",
        );
        let result2 = map_requirement_to_implemented(
            &req2,
            "id-2",
            "POL-AC-002",
            1,
            Some("test.md"),
            "Access Control",
        );

        assert_ne!(
            result1.uuid, result2.uuid,
            "Requirements with identical text but different positions must have distinct UUIDs"
        );
    }

    // ─── T014: Trace props and links in implemented-requirement (WI-17) ──

    #[test]
    fn map_requirement_has_trace_props() {
        let req = PolicyRequirement {
            text: "All users must authenticate.".to_string(),
            source_line: 42,
            nesting_depth: 0,
            stable_id: Some("uuid-1".to_string()),
            atom_index: 0,
            parent_text: None,
            citations: vec![],
            modality: None,
            parameters: vec![],
            parameters_extracted: false,
        };
        let result = map_requirement_to_implemented(
            &req,
            "uuid-1",
            "POL-AC-001",
            0,
            Some("policies/security.md"),
            "Access Control",
        );

        assert_eq!(result.props.len(), 3, "Must have exactly 3 trace props");

        assert_eq!(result.props[0].name, "source-file");
        assert_eq!(
            result.props[0].ns.as_deref(),
            Some("https://forge.policy-forge.github.io/ns/trace")
        );
        assert_eq!(result.props[0].value, "policies/security.md");

        assert_eq!(result.props[1].name, "source-section");
        assert_eq!(
            result.props[1].ns.as_deref(),
            Some("https://forge.policy-forge.github.io/ns/trace")
        );
        assert_eq!(result.props[1].value, "Access Control");

        assert_eq!(result.props[2].name, "source-line");
        assert_eq!(
            result.props[2].ns.as_deref(),
            Some("https://forge.policy-forge.github.io/ns/trace")
        );
        assert_eq!(result.props[2].value, "42");
    }

    #[test]
    fn map_requirement_has_trace_link() {
        let req = PolicyRequirement {
            text: "Encrypt at rest.".to_string(),
            source_line: 99,
            nesting_depth: 0,
            stable_id: Some("uuid-2".to_string()),
            atom_index: 0,
            parent_text: None,
            citations: vec![],
            modality: None,
            parameters: vec![],
            parameters_extracted: false,
        };
        let result = map_requirement_to_implemented(
            &req,
            "uuid-2",
            "POL-DP-001",
            0,
            Some("policies/security.md"),
            "Data Protection",
        );

        assert_eq!(result.links.len(), 1, "Must have exactly 1 source link");
        assert_eq!(result.links[0].rel, "source");
        assert_eq!(result.links[0].href, "policies%2Fsecurity.md#line=99");
    }

    // ─── T023: No remarks in output (SEC-1, SEC-2) ──────────────────────

    #[test]
    fn no_remarks_key_in_output() {
        let req = make_req("All users must authenticate.", Some("uuid-1"), 0);
        let result = map_requirement_to_implemented(
            &req,
            "uuid-1",
            "POL-AC-001",
            0,
            Some("test.md"),
            "Access Control",
        );
        let json = serde_json::to_string(&result).unwrap();

        assert!(!json.contains("\"remarks\""), "Output must not contain 'remarks' key");
        assert!(!result.description.is_empty(), "Must have 'description' field");
    }
}
