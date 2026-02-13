//! OSCAL implemented-requirements builder: maps [`PolicyRequirement`]s to
//! OSCAL `implemented-requirement` JSON entries within a `control-implementations` array.
//!
//! Walks the [`PolicyDocument`]'s section tree depth-first (matching the Catalog
//! builder's traversal order) and produces one `implemented-requirement` per
//! [`PolicyRequirement`] with a deterministic UUID v5 and a `control-id` matching
//! the Catalog builder's scheme (e.g., `POL-AC-001`).

use std::collections::HashMap;

use serde_json::Value;
use uuid::Uuid;

use crate::error::ForgeError;
use crate::model::{PolicyDocument, PolicyRequirement};
use crate::oscal::catalog::{collect_requirements, generate_control_id, resolve_abbreviation};
use crate::uuid::{CONTROL_IMPL_NAMESPACE, IMPL_REQ_NAMESPACE};

/// Build the `control-implementations` JSON array for a Component Definition.
///
/// Walks the document's sections to generate control-ids consistent with the
/// Catalog builder (WI-9). Produces a JSON array containing one
/// `control-implementations` entry with the source profile reference and all
/// implemented-requirements mapped from [`PolicyRequirement`]s.
///
/// # Arguments
/// * `document` - `PolicyDocument` with sections and requirements
/// * `source_profile` - Value of the `--source-profile` CLI flag
///
/// # Returns
/// A `serde_json::Value` array containing one control-implementations entry.
///
/// # Errors
/// Returns [`ForgeError::ComponentDefinitionBuild`] if mapping fails.
pub fn build_control_implementations(
    document: &PolicyDocument,
    source_profile: &str,
) -> Result<Value, ForgeError> {
    let mut abbrev_counts: HashMap<String, usize> = HashMap::new();
    let mut implemented_requirements = Vec::new();
    let mut global_index: usize = 0;

    for section in &document.sections {
        let abbreviation = resolve_abbreviation(&section.title, &mut abbrev_counts);
        let requirements = collect_requirements(section);

        for (req_idx, req) in requirements.iter().enumerate() {
            let has_stable_id = req.stable_id.is_some();
            let control_id =
                derive_control_id_or_fallback(&abbreviation, req_idx, global_index, has_stable_id);
            let entry = map_requirement_to_implemented(req, &control_id, global_index);
            implemented_requirements.push(entry);
            global_index += 1;
        }
    }

    if implemented_requirements.is_empty() {
        tracing::warn!(
            "Document has zero requirements — control-implementations will have empty implemented-requirements array"
        );
    }

    let ci_uuid = generate_control_impl_uuid(source_profile, &document.metadata.title);
    let description =
        format!("Implementation narratives derived from {}.", document.metadata.title);

    let entry = serde_json::json!({
        "uuid": ci_uuid.to_string(),
        "source": source_profile,
        "description": description,
        "implemented-requirements": implemented_requirements,
    });

    Ok(serde_json::json!([entry]))
}

/// Map a single [`PolicyRequirement`] to an OSCAL implemented-requirement JSON entry.
///
/// # Arguments
/// * `requirement` - A single `PolicyRequirement` from the domain model
/// * `control_id` - The pre-computed control-id for this requirement
/// * `global_index` - The requirement's global index (for UUID seed uniqueness)
///
/// # Returns
/// A `serde_json::Value` object with `uuid`, `control-id`, and `description` fields.
fn map_requirement_to_implemented(
    requirement: &PolicyRequirement,
    control_id: &str,
    global_index: usize,
) -> Value {
    let stable_id = requirement.stable_id.as_deref().unwrap_or("no-stable-id");

    let uuid = generate_impl_req_uuid(stable_id, &requirement.text, global_index);

    let description = if requirement.text.is_empty() {
        "No implementation narrative available.".to_string()
    } else {
        requirement.text.clone()
    };

    serde_json::json!({
        "uuid": uuid.to_string(),
        "control-id": control_id,
        "description": description,
    })
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
/// Seed format: `"{stable_id}\0{text}\0{index}"`
///
/// The index ensures uniqueness when two requirements have identical text (EC-5).
///
/// # Arguments
/// * `stable_id` - The requirement's stable ID (UUID string)
/// * `text` - The requirement text
/// * `index` - Global atom index for uniqueness
fn generate_impl_req_uuid(stable_id: &str, text: &str, index: usize) -> Uuid {
    let seed = format!("{stable_id}\0{text}\0{index}");
    Uuid::new_v5(&IMPL_REQ_NAMESPACE, seed.as_bytes())
}

/// Derive a control-id for a requirement, with fallback for missing `stable_id`.
///
/// Normal case: uses section-based generation via [`generate_control_id`] producing
/// `POL-{ABBR}-{NNN}` format matching the Catalog builder.
///
/// Fallback (EC-2): `REQ-{zero-padded global_index}` when `stable_id` is `None`.
///
/// # Arguments
/// * `abbreviation` - The section abbreviation (e.g., "AC")
/// * `req_index_in_section` - Zero-based index of the requirement within its section
/// * `global_index` - Zero-based global index across all sections
/// * `has_stable_id` - Whether the requirement has a `stable_id`
fn derive_control_id_or_fallback(
    abbreviation: &str,
    req_index_in_section: usize,
    global_index: usize,
    has_stable_id: bool,
) -> String {
    if has_stable_id {
        generate_control_id(abbreviation, req_index_in_section, "POL")
    } else {
        tracing::warn!(global_index, "Requirement missing stable_id — using fallback control-id");
        format!("REQ-{:03}", global_index + 1)
    }
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
        assert_ne!(uuid1, uuid2, "Different source_profile must produce different UUID");
        assert_ne!(uuid1, uuid3, "Different policy_title must produce different UUID");
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

    // ─── T005: derive_control_id_or_fallback Tests [US1] ─────────────────

    #[test]
    fn control_id_normal_case() {
        let id = derive_control_id_or_fallback("AC", 0, 0, true);
        assert_eq!(id, "POL-AC-001");
    }

    #[test]
    fn control_id_normal_case_second_req() {
        let id = derive_control_id_or_fallback("DP", 2, 5, true);
        assert_eq!(id, "POL-DP-003");
    }

    #[test]
    fn control_id_fallback_no_stable_id() {
        let id = derive_control_id_or_fallback("AC", 0, 0, false);
        assert_eq!(id, "REQ-001");
    }

    #[test]
    fn control_id_fallback_global_index() {
        let id = derive_control_id_or_fallback("AC", 0, 4, false);
        assert_eq!(id, "REQ-005");
    }

    // ─── T006: map_requirement_to_implemented Tests [US1] ────────────────

    #[test]
    fn map_requirement_has_required_fields() {
        let req = make_req("All users must authenticate.", Some("uuid-1"), 0);
        let result = map_requirement_to_implemented(&req, "POL-AC-001", 0);

        assert!(result.get("uuid").is_some(), "Must have uuid field");
        assert!(result.get("control-id").is_some(), "Must have control-id field");
        assert!(result.get("description").is_some(), "Must have description field");
    }

    #[test]
    fn map_requirement_uses_raw_text() {
        let req = make_req("All users must authenticate.", Some("uuid-1"), 0);
        let result = map_requirement_to_implemented(&req, "POL-AC-001", 0);

        assert_eq!(
            result["description"], "All users must authenticate.",
            "FR-008: description must be raw requirement text"
        );
    }

    #[test]
    fn map_requirement_control_id_matches() {
        let req = make_req("Test requirement.", Some("uuid-1"), 0);
        let result = map_requirement_to_implemented(&req, "POL-AC-001", 0);

        assert_eq!(result["control-id"], "POL-AC-001");
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

        let result = build_control_implementations(&doc, "./baseline.json").unwrap();
        let arr = result.as_array().expect("Must be a JSON array");
        assert_eq!(arr.len(), 1, "Must produce exactly one control-implementations entry");
    }

    #[test]
    fn build_control_impl_has_required_fields() {
        let doc = make_doc(vec![make_section(
            "Access Control",
            vec![make_req("R1.", Some("id1"), 0)],
            vec![],
        )]);

        let result = build_control_implementations(&doc, "./baseline.json").unwrap();
        let entry = &result[0];

        assert!(entry.get("uuid").is_some(), "Must have uuid");
        assert!(entry.get("source").is_some(), "Must have source");
        assert!(entry.get("description").is_some(), "Must have description");
        assert!(
            entry.get("implemented-requirements").is_some(),
            "Must have implemented-requirements"
        );
    }

    #[test]
    fn build_control_impl_source_matches_profile() {
        let doc = make_doc(vec![make_section(
            "Access Control",
            vec![make_req("R1.", Some("id1"), 0)],
            vec![],
        )]);

        let result = build_control_implementations(&doc, "./baselines/nist.json").unwrap();
        assert_eq!(result[0]["source"], "./baselines/nist.json");
    }

    #[test]
    fn build_control_impl_description_pattern() {
        let doc = make_doc(vec![make_section(
            "Access Control",
            vec![make_req("R1.", Some("id1"), 0)],
            vec![],
        )]);

        let result = build_control_implementations(&doc, "./baseline.json").unwrap();
        assert_eq!(
            result[0]["description"],
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

        let result = build_control_implementations(&doc, "./baseline.json").unwrap();
        let impl_reqs = result[0]["implemented-requirements"].as_array().unwrap();
        assert_eq!(
            impl_reqs.len(),
            5,
            "Must have one implemented-requirement per PolicyRequirement"
        );
    }

    // ─── T018: Zero requirements edge case (EC-1, FR-013) ────────────────

    #[test]
    fn zero_requirements_produces_empty_impl_reqs() {
        let doc = make_doc(vec![make_section("Empty Section", vec![], vec![])]);
        let result = build_control_implementations(&doc, "./baseline.json").unwrap();
        let impl_reqs = result[0]["implemented-requirements"].as_array().unwrap();
        assert!(impl_reqs.is_empty(), "Zero requirements must produce empty array");
    }

    // ─── T019: Empty requirement text (EC-3, FR-014, SEC-5) ──────────────

    #[test]
    fn empty_text_produces_placeholder_description() {
        let req = make_req("", Some("uuid-1"), 0);
        let result = map_requirement_to_implemented(&req, "POL-AC-001", 0);
        assert_eq!(
            result["description"], "No implementation narrative available.",
            "Empty text must produce placeholder description"
        );
    }

    // ─── T020: Missing stable_id fallback (EC-2) ─────────────────────────

    #[test]
    fn missing_stable_id_produces_fallback_control_id() {
        let doc = make_doc(vec![make_section(
            "Access Control",
            vec![make_req("R1.", None, 0), make_req("R2.", None, 0)],
            vec![],
        )]);

        let result = build_control_implementations(&doc, "./baseline.json").unwrap();
        let impl_reqs = result[0]["implemented-requirements"].as_array().unwrap();
        assert_eq!(impl_reqs[0]["control-id"], "REQ-001");
        assert_eq!(impl_reqs[1]["control-id"], "REQ-002");
    }

    // ─── T022: Identical text, different positions produce distinct UUIDs (EC-5) ──

    #[test]
    fn identical_text_different_positions_distinct_uuids() {
        let req1 = make_req("Same requirement text.", Some("id-1"), 0);
        let req2 = make_req("Same requirement text.", Some("id-2"), 1);

        let result1 = map_requirement_to_implemented(&req1, "POL-AC-001", 0);
        let result2 = map_requirement_to_implemented(&req2, "POL-AC-002", 1);

        assert_ne!(
            result1["uuid"], result2["uuid"],
            "Requirements with identical text but different positions must have distinct UUIDs"
        );
    }

    // ─── T023: No remarks in output (SEC-1, SEC-2) ──────────────────────

    #[test]
    fn no_remarks_key_in_output() {
        let req = make_req("All users must authenticate.", Some("uuid-1"), 0);
        let result = map_requirement_to_implemented(&req, "POL-AC-001", 0);
        let json = serde_json::to_string(&result).unwrap();

        assert!(!json.contains("\"remarks\""), "Output must not contain 'remarks' key");
        assert!(result.get("description").is_some(), "Must have 'description' field");
    }
}
