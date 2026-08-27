//! OSCAL Component Definition builder: maps [`crate::model::PolicyDocument`] to OSCAL
//! Component Definition JSON.
//!
//! Converts the domain model to an OSCAL Component Definition structure
//! containing a single documentary component of type "policy".

use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ForgeError;
use crate::model::trace::TraceLinkCollection;
use crate::model::{Citation, DocumentMetadata, PolicyDocument, PolicySection};
use crate::oscal::back_matter::BackMatter;
use crate::oscal::metadata::assemble_metadata;
use crate::uuid::{BACK_MATTER_NAMESPACE, COMPONENT_NAMESPACE};

// ─── Constants ──────────────────────────────────────────────────────────

/// Default title when `PolicyDocument` has no title (empty string).
pub const DEFAULT_COMPONENT_TITLE: &str = "Untitled Policy Document";

// ─── Structs ────────────────────────────────────────────────────────────

/// JSON envelope producing `{"component-definition": {...}}` at the top level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentDefinitionEnvelope {
    /// The OSCAL Component Definition.
    #[serde(rename = "component-definition")]
    pub component_definition: ComponentDefinition,
}

/// OSCAL Component Definition root structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentDefinition {
    /// Document-level UUID (v4, unique per generation).
    pub uuid: String,

    /// OSCAL metadata block (reused from WI-11 pattern).
    pub metadata: ComponentDefinitionMetadata,

    /// Documentary components (exactly one for this WI).
    pub components: Vec<DocumentaryComponent>,

    /// Capabilities grouping control implementations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<Capability>,

    /// Back matter containing reference resources (WI-12).
    #[serde(default, rename = "back-matter", skip_serializing_if = "Option::is_none")]
    pub back_matter: Option<BackMatter>,
}

/// OSCAL metadata for the Component Definition.
///
/// Fields mapped from the shared `assemble_metadata` return value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentDefinitionMetadata {
    /// Document title from PolicyDocument.metadata.title.
    pub title: String,

    /// ISO 8601 UTC timestamp of artifact generation.
    #[serde(rename = "last-modified")]
    pub last_modified: String,

    /// Document version from PolicyDocument.metadata.version.
    pub version: String,

    /// OSCAL specification version -- always "1.2.0".
    #[serde(rename = "oscal-version")]
    pub oscal_version: String,
}

/// OSCAL Capability within a Component Definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Unique identifier for this capability.
    pub uuid: String,
    /// Capability name.
    pub name: String,
    /// Capability description.
    pub description: String,
    /// Control implementations under this capability.
    #[serde(default, rename = "control-implementations", skip_serializing_if = "Vec::is_empty")]
    pub control_implementations: Vec<crate::oscal::implemented_requirements::ControlImplementation>,
}

/// A documentary component of type "policy" within the Component Definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentaryComponent {
    /// Deterministic UUID v5 (from `COMPONENT_NAMESPACE` + title + version + document ID).
    pub uuid: String,

    /// Component type -- always "policy" for documentary components.
    #[serde(rename = "type")]
    pub component_type: String,

    /// Component title (from `PolicyDocument` title or default).
    pub title: String,

    /// Component description (template format).
    pub description: String,

    /// Component-level properties (e.g., source-file for traceability).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub props: Vec<crate::oscal::parts::OscalProp>,

    /// Control implementations (empty for WI-14; populated by WI-15).
    #[serde(default, rename = "control-implementations", skip_serializing_if = "Vec::is_empty")]
    pub control_implementations: Vec<crate::oscal::implemented_requirements::ControlImplementation>,
}

// ─── Builder Function ───────────────────────────────────────────────────

/// Build an OSCAL Component Definition from a `PolicyDocument`.
///
/// Produces a `ComponentDefinitionEnvelope` with:
/// - Document-level UUID (v4) and metadata (via WI-11 `assemble_metadata`)
/// - One documentary component (type: "policy") with UUID (v5), title, description
/// - Empty `control-implementations` placeholder (populated by WI-15)
/// - Optional back matter (via WI-12 `generate_back_matter`)
///
/// The `trace_links` parameter is retained for API compatibility but is currently ignored.
/// Pass `None` for backward compatibility.
///
/// # Errors
///
/// Returns the back-matter error variant when citation resource generation fails.
pub fn build_component_definition(
    document: &PolicyDocument,
    source_profile: Option<&str>,
    _trace_links: Option<&mut TraceLinkCollection>,
    source_file: Option<&str>,
) -> Result<ComponentDefinitionEnvelope, ForgeError> {
    // Step 1: Resolve title and version defaults
    let title = resolve_title(&document.metadata.title);
    let version = resolve_version(&document.metadata.version);

    // Step 2: Assemble metadata via WI-11 with resolved defaults
    let resolved_meta = DocumentMetadata {
        title: title.clone(),
        version: version.clone(),
        ..document.metadata.clone()
    };
    let assembled = assemble_metadata(&resolved_meta, None)?;

    // Step 3: Build control-implementations (WI-15) when source_profile is provided
    let control_implementations = match source_profile {
        Some(profile) => crate::oscal::implemented_requirements::build_control_implementations(
            document,
            profile,
            source_file,
        )?,
        None => vec![],
    };

    // Step 4: Build documentary component
    let component_uuid = generate_component_uuid(&title, &version, &document.id);
    let description = format!("Documentary component representing the {title} policy document.");

    let component_props =
        source_file.filter(|path| !path.is_empty()).map_or_else(Vec::new, |path| {
            vec![crate::oscal::parts::OscalProp {
                name: crate::oscal::trace_embedding::PROP_SOURCE_FILE.to_string(),
                ns: Some(crate::oscal::trace_embedding::FORGE_TRACE_NS.to_string()),
                value: path.to_string(),
            }]
        });

    let component = DocumentaryComponent {
        uuid: component_uuid.to_string(),
        component_type: "policy".to_string(),
        title: title.clone(),
        description,
        props: component_props,
        control_implementations,
    };

    // Step 5: Collect citations and generate back matter (WI-12 reuse)
    let all_citations = collect_all_citations(&document.sections);
    let back_matter = if all_citations.is_empty() {
        None
    } else {
        let (resources, _resource_map) =
            crate::oscal::back_matter::generate_back_matter(&all_citations).map_err(|error| {
                match error {
                    ForgeError::BackMatter(detail) => ForgeError::BackMatter(format!(
                        "while generating component-definition back matter: {detail}"
                    )),
                    other => other,
                }
            })?;

        if resources.is_empty() { None } else { Some(BackMatter { resources }) }
    };

    // Step 6: Map metadata fields from assembled metadata
    let metadata = ComponentDefinitionMetadata {
        title: assembled.title,
        last_modified: assembled.last_modified.to_rfc3339(),
        version: assembled.version,
        oscal_version: assembled.oscal_version,
    };

    // Step 7: Assemble envelope
    Ok(ComponentDefinitionEnvelope {
        component_definition: ComponentDefinition {
            uuid: assembled.uuid.to_string(),
            metadata,
            components: vec![component],
            capabilities: vec![],
            back_matter,
        },
    })
}

/// Resolve a title string, defaulting to `DEFAULT_COMPONENT_TITLE` if empty.
fn resolve_title(title: &str) -> String {
    if title.is_empty() { DEFAULT_COMPONENT_TITLE.to_string() } else { title.to_string() }
}

/// Resolve a version string, defaulting to `"0.0.0"` if empty.
fn resolve_version(version: &str) -> String {
    if version.is_empty() { "0.0.0".to_string() } else { version.to_string() }
}

/// Generate a deterministic UUID v5 for the documentary component.
///
/// Uses title, version, and document ID as inputs to ensure uniqueness
/// across different source documents that may share the same title/version.
fn generate_component_uuid(title: &str, version: &str, document_id: &str) -> Uuid {
    let input = format!("{title}\0{version}\0{document_id}");
    Uuid::new_v5(&COMPONENT_NAMESPACE, input.as_bytes())
}

/// Maximum recursion depth for section tree traversal (`DoS` protection).
const MAX_SECTION_DEPTH: usize = 50;

/// Maximum number of unique citations to collect (`DoS` protection).
const MAX_CITATIONS: usize = 10_000;

/// Recursively collect citations that produce unique back-matter resources.
///
/// Deduplicates by the same UUID identity used by `generate_back_matter`, rather
/// than the citation ID, to prevent duplicate OSCAL resource UUIDs. Enforces
/// depth and count bounds to prevent unbounded memory allocation.
#[must_use]
pub fn collect_all_citations(sections: &[PolicySection]) -> Vec<Citation> {
    let mut citations = Vec::new();
    let mut seen_resource_uuids = HashSet::new();
    for section in sections {
        collect_citations_from_section(section, &mut citations, &mut seen_resource_uuids, 0);
    }
    citations
}

fn citation_resource_uuid(citation: &Citation) -> Uuid {
    let normalized = crate::uuid::normalize_for_hashing(&citation.text);
    let hash_input = match &citation.url {
        Some(url) => format!("{normalized}\n{url}"),
        None => normalized,
    };
    Uuid::new_v5(&BACK_MATTER_NAMESPACE, hash_input.as_bytes())
}

fn collect_citations_from_section(
    section: &PolicySection,
    citations: &mut Vec<Citation>,
    seen_resource_uuids: &mut HashSet<Uuid>,
    depth: usize,
) {
    if depth > MAX_SECTION_DEPTH {
        tracing::warn!(depth, "Section tree depth exceeds safety limit, stopping traversal");
        return;
    }
    if citations.len() >= MAX_CITATIONS {
        tracing::warn!(
            count = citations.len(),
            "Citation count exceeds safety limit, stopping collection"
        );
        return;
    }
    for req in &section.requirements {
        for citation in &req.citations {
            if citations.len() >= MAX_CITATIONS {
                return;
            }
            // Invalid IDs must reach the generator so it can return its typed error.
            if citation.id.is_empty()
                || seen_resource_uuids.insert(citation_resource_uuid(citation))
            {
                citations.push(citation.clone());
            }
        }
    }
    for child in &section.children {
        collect_citations_from_section(child, citations, seen_resource_uuids, depth + 1);
    }
}

// ─── Control ID Collector (WI-41, T009) ─────────────────────────────────

/// Collect all control IDs from a built Component Definition.
///
/// Iterates both `components[].control_implementations[].implemented_requirements[].control_id`
/// and `capabilities[].control_implementations[].implemented_requirements[].control_id`.
/// Returns an empty Vec if the Component Definition has no implemented requirements.
/// Does NOT deduplicate — deduplication is performed by `build_assessment_plan`.
#[must_use]
pub fn collect_control_ids_from_component_def(
    envelope: &ComponentDefinitionEnvelope,
) -> Vec<String> {
    let from_components = envelope
        .component_definition
        .components
        .iter()
        .flat_map(|c| c.control_implementations.iter())
        .flat_map(|ci| ci.implemented_requirements.iter())
        .map(|ir| ir.control_id.clone());

    let from_capabilities = envelope
        .component_definition
        .capabilities
        .iter()
        .flat_map(|cap| cap.control_implementations.iter())
        .flat_map(|ci| ci.implemented_requirements.iter())
        .map(|ir| ir.control_id.clone());

    from_components.chain(from_capabilities).collect()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::model::{Citation, DocumentMetadata, PolicyRequirement, PolicySection};

    // ─── Test Helpers ────────────────────────────────────────────────────

    fn test_document(title: &str, version: &str) -> PolicyDocument {
        PolicyDocument {
            id: "test-doc".to_string(),
            metadata: DocumentMetadata {
                title: title.to_string(),
                version: version.to_string(),
                author: None,
                date: None,
                source_path: PathBuf::from("test.md"),
                content_hash: None,
            },
            sections: vec![],
        }
    }

    fn test_document_with_sections(
        title: &str,
        version: &str,
        sections: Vec<PolicySection>,
    ) -> PolicyDocument {
        PolicyDocument {
            id: "test-doc".to_string(),
            metadata: DocumentMetadata {
                title: title.to_string(),
                version: version.to_string(),
                author: None,
                date: None,
                source_path: PathBuf::from("test.md"),
                content_hash: None,
            },
            sections,
        }
    }

    fn test_section(title: &str, reqs: Vec<PolicyRequirement>) -> PolicySection {
        PolicySection {
            title: title.to_string(),
            heading_level: 1,
            source_line: 1,
            body_text: None,
            requirements: reqs,
            children: vec![],
        }
    }

    fn test_requirement_with_citation(text: &str, citation: Citation) -> PolicyRequirement {
        PolicyRequirement {
            text: text.to_string(),
            source_line: 1,
            nesting_depth: 0,
            stable_id: Some("test-uuid".to_string()),
            atom_index: 0,
            parent_text: None,
            citations: vec![citation],
            modality: None,
            parameters: vec![],
            parameters_extracted: false,
        }
    }

    // ─── T006: Happy Path Root Key and Metadata ─────────────────────────

    #[test]
    fn test_happy_path_root_key_and_metadata() {
        let doc = test_document("Corporate Security Policy", "2.0");
        let envelope = build_component_definition(&doc, None, None, None).unwrap();
        let json = serde_json::to_string_pretty(&envelope).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();

        // (1) Root key is "component-definition"
        assert!(v.get("component-definition").is_some(), "Root key must be 'component-definition'");

        let cd = &v["component-definition"];

        // (2) metadata.title matches PolicyDocument title
        assert_eq!(cd["metadata"]["title"], "Corporate Security Policy");

        // (3) metadata.version matches PolicyDocument version
        assert_eq!(cd["metadata"]["version"], "2.0");

        // (4) metadata.oscal-version uses the shared current baseline.
        assert_eq!(cd["metadata"]["oscal-version"], crate::oscal::metadata::OSCAL_VERSION);

        // (5) metadata.last-modified is present and non-empty
        let last_modified = cd["metadata"]["last-modified"].as_str().unwrap();
        assert!(!last_modified.is_empty(), "last-modified must be non-empty");

        // (6) uuid is present and non-empty
        let uuid = cd["uuid"].as_str().unwrap();
        assert!(!uuid.is_empty(), "uuid must be non-empty");
    }

    // ─── T007: Documentary Component Structure ──────────────────────────

    #[test]
    fn test_documentary_component_structure() {
        let doc = test_document("Corporate Security Policy", "2.0");
        let envelope = build_component_definition(&doc, None, None, None).unwrap();
        let json = serde_json::to_string_pretty(&envelope).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();

        let components = v["component-definition"]["components"].as_array().unwrap();

        // (1) Exactly 1 component
        assert_eq!(components.len(), 1);

        let comp = &components[0];

        // (2) type is "policy"
        assert_eq!(comp["type"].as_str(), Some("policy"));

        // (3) title matches PolicyDocument title
        assert_eq!(comp["title"].as_str(), Some("Corporate Security Policy"));

        // (4) description matches template
        assert_eq!(
            comp["description"].as_str(),
            Some(
                "Documentary component representing the Corporate Security Policy policy document."
            )
        );

        // (5) uuid is present and non-empty
        let uuid = comp["uuid"].as_str().unwrap();
        assert!(!uuid.is_empty(), "Component uuid must be non-empty");
    }

    // ─── T008: Component UUID Determinism ───────────────────────────────

    #[test]
    fn test_component_uuid_determinism() {
        let doc1 = test_document("Corporate Security Policy", "2.0");
        let doc2 = test_document("Corporate Security Policy", "2.0");

        let env1 = build_component_definition(&doc1, None, None, None).unwrap();
        let env2 = build_component_definition(&doc2, None, None, None).unwrap();

        // (1) Same input produces same component UUID
        assert_eq!(
            env1.component_definition.components[0].uuid,
            env2.component_definition.components[0].uuid,
        );

        // (2) Different title produces different UUID
        let doc_diff_title = test_document("Different Policy", "2.0");
        let env_diff = build_component_definition(&doc_diff_title, None, None, None).unwrap();
        assert_ne!(
            env1.component_definition.components[0].uuid,
            env_diff.component_definition.components[0].uuid,
        );

        // (3) Different version produces different UUID
        let doc_diff_version = test_document("Corporate Security Policy", "3.0");
        let env_diff_v = build_component_definition(&doc_diff_version, None, None, None).unwrap();
        assert_ne!(
            env1.component_definition.components[0].uuid,
            env_diff_v.component_definition.components[0].uuid,
        );

        // (4) Component UUID differs from document-level UUID
        assert_ne!(env1.component_definition.components[0].uuid, env1.component_definition.uuid,);
    }

    // ─── T009: Edge Cases ───────────────────────────────────────────────

    #[test]
    fn test_empty_title_defaults() {
        let doc = test_document("", "1.0");
        let envelope = build_component_definition(&doc, None, None, None).unwrap();
        let comp = &envelope.component_definition.components[0];

        // EC-1, SEC-3: component title defaults to "Untitled Policy Document"
        assert_eq!(comp.title, DEFAULT_COMPONENT_TITLE);

        // Metadata title also defaults
        assert_eq!(envelope.component_definition.metadata.title, DEFAULT_COMPONENT_TITLE);

        // Description uses default title
        assert_eq!(
            comp.description,
            "Documentary component representing the Untitled Policy Document policy document."
        );
    }

    #[test]
    fn test_empty_version_defaults() {
        let doc = test_document("Test Policy", "");
        let envelope = build_component_definition(&doc, None, None, None).unwrap();

        // EC-2, SEC-4: metadata.version defaults to "0.0.0"
        assert_eq!(envelope.component_definition.metadata.version, "0.0.0");
    }

    #[test]
    fn test_empty_sections_produces_valid_output() {
        // EC-3: No sections or requirements — still valid
        let doc = test_document("Minimal Policy", "1.0");
        let envelope = build_component_definition(&doc, None, None, None).unwrap();

        assert_eq!(envelope.component_definition.components.len(), 1);
        assert_eq!(envelope.component_definition.components[0].component_type, "policy");
    }

    // ─── T010: Security Requirements ────────────────────────────────────

    #[test]
    fn test_no_remarks_in_output() {
        // SEC-2: No "remarks" key anywhere in JSON
        let doc = test_document("Corporate Security Policy", "2.0");
        let envelope = build_component_definition(&doc, None, None, None).unwrap();
        let json = serde_json::to_string_pretty(&envelope).unwrap();

        assert!(!json.contains("\"remarks\""), "JSON must not contain 'remarks' key");
    }

    #[test]
    fn test_no_extra_data_beyond_policy_document() {
        // SEC-1: Only PolicyDocument-derived and OSCAL-convention fields
        let doc = test_document("Corporate Security Policy", "2.0");
        let envelope = build_component_definition(&doc, None, None, None).unwrap();
        let json = serde_json::to_string_pretty(&envelope).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();

        let cd = v["component-definition"].as_object().unwrap();
        let allowed_root_keys = ["uuid", "metadata", "components", "back-matter"];
        for key in cd.keys() {
            assert!(allowed_root_keys.contains(&key.as_str()), "Unexpected root key: {key}");
        }

        let metadata = cd["metadata"].as_object().unwrap();
        let allowed_meta_keys = ["title", "last-modified", "version", "oscal-version"];
        for key in metadata.keys() {
            assert!(allowed_meta_keys.contains(&key.as_str()), "Unexpected metadata key: {key}");
        }
    }

    // ─── T011: Should Have Requirements ─────────────────────────────────

    #[test]
    fn test_empty_control_implementations() {
        // S-1: control-implementations is omitted when empty (skip_serializing_if)
        let doc = test_document("Corporate Security Policy", "2.0");
        let envelope = build_component_definition(&doc, None, None, None).unwrap();
        let json = serde_json::to_string_pretty(&envelope).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();

        let component = &v["component-definition"]["components"][0];
        assert!(
            component.get("control-implementations").is_none(),
            "control-implementations should be omitted when empty"
        );
    }

    #[test]
    fn test_back_matter_included_with_citations() {
        // S-2: Back matter appears when PolicyDocument has citations
        let citation = Citation {
            id: "cite-1".to_string(),
            text: "NIST SP 800-53".to_string(),
            url: Some("https://example.com/sp800-53".to_string()),
            source_requirement_id: Some("req-1".into()),
        };
        let doc = test_document_with_sections(
            "Corporate Security Policy",
            "2.0",
            vec![test_section(
                "Access Control",
                vec![test_requirement_with_citation("All users must auth.", citation)],
            )],
        );
        let envelope = build_component_definition(&doc, None, None, None).unwrap();
        assert!(
            envelope.component_definition.back_matter.is_some(),
            "Back matter should be present when citations exist"
        );
    }

    #[test]
    fn test_back_matter_deduplicates_citations() {
        // Same citation referenced by multiple requirements should produce one resource
        let citation1 = Citation {
            id: "cite-1".to_string(),
            text: "NIST SP 800-53".to_string(),
            url: Some("https://example.com/sp800-53".to_string()),
            source_requirement_id: Some("req-1".into()),
        };
        let citation2 = Citation {
            id: "cite-1".to_string(),
            text: "NIST SP 800-53".to_string(),
            url: Some("https://example.com/sp800-53".to_string()),
            source_requirement_id: Some("req-2".into()),
        };
        let doc = test_document_with_sections(
            "Corporate Security Policy",
            "2.0",
            vec![test_section(
                "Access Control",
                vec![
                    test_requirement_with_citation("All users must auth.", citation1),
                    test_requirement_with_citation("All admins must use MFA.", citation2),
                ],
            )],
        );
        let envelope = build_component_definition(&doc, None, None, None).unwrap();
        let bm = envelope.component_definition.back_matter.as_ref().unwrap();
        assert_eq!(bm.resources.len(), 1, "Duplicate citations should produce one resource");
    }

    #[test]
    fn test_back_matter_deduplicates_distinct_citation_ids_with_same_resource_identity() {
        let citation1 = Citation {
            id: "cite-1".to_string(),
            text: "NIST SP 800-53".to_string(),
            url: Some("https://example.com/sp800-53".to_string()),
            source_requirement_id: Some("req-1".into()),
        };
        let citation2 = Citation {
            id: "cite-2".to_string(),
            text: "NIST SP 800-53".to_string(),
            url: Some("https://example.com/sp800-53".to_string()),
            source_requirement_id: Some("req-2".into()),
        };
        let doc = test_document_with_sections(
            "Corporate Security Policy",
            "2.0",
            vec![test_section(
                "Access Control",
                vec![
                    test_requirement_with_citation("All users must authenticate.", citation1),
                    test_requirement_with_citation("All administrators must use MFA.", citation2),
                ],
            )],
        );

        let envelope = build_component_definition(&doc, None, None, None).unwrap();
        let back_matter = envelope.component_definition.back_matter.as_ref().unwrap();
        assert_eq!(back_matter.resources.len(), 1);
    }

    #[test]
    fn test_back_matter_error_preserves_its_variant_and_context() {
        let citation = Citation {
            id: String::new(),
            text: "NIST SP 800-53".to_string(),
            url: None,
            source_requirement_id: None,
        };
        let doc = test_document_with_sections(
            "Corporate Security Policy",
            "2.0",
            vec![test_section(
                "Access Control",
                vec![test_requirement_with_citation("All users must authenticate.", citation)],
            )],
        );

        let error = build_component_definition(&doc, None, None, None).unwrap_err();
        assert!(matches!(
            error,
            ForgeError::BackMatter(detail) if detail.contains("component-definition back matter")
        ));
    }

    #[test]
    fn test_back_matter_absent_without_citations() {
        // S-2: Back matter absent when no citations
        let doc = test_document("Corporate Security Policy", "2.0");
        let envelope = build_component_definition(&doc, None, None, None).unwrap();
        assert!(
            envelope.component_definition.back_matter.is_none(),
            "Back matter should be absent when no citations"
        );

        // Also verify it's omitted from JSON
        let json = serde_json::to_string_pretty(&envelope).unwrap();
        assert!(
            !json.contains("\"back-matter\""),
            "back-matter key should not appear in JSON when absent"
        );
    }

    // ─── T014: Metadata Consistency with Catalog (US2) ──────────────────

    #[test]
    fn test_metadata_consistency_with_catalog() {
        // AC-6, M-7: Both artifacts use assemble_metadata from WI-11
        let doc = test_document("Corporate Security Policy", "2.0");
        let cd_envelope = build_component_definition(&doc, None, None, None).unwrap();

        // Build catalog metadata via the same assemble_metadata path
        let cat_metadata = assemble_metadata(&doc.metadata, None).unwrap();

        let cd_meta = &cd_envelope.component_definition.metadata;

        // (1) Both use the shared current OSCAL baseline.
        assert_eq!(cd_meta.oscal_version, crate::oscal::metadata::OSCAL_VERSION);
        assert_eq!(cat_metadata.oscal_version, crate::oscal::metadata::OSCAL_VERSION);

        // (2) Both have matching title
        assert_eq!(cd_meta.title, cat_metadata.title);

        // (3) Both have matching version
        assert_eq!(cd_meta.version, cat_metadata.version);

        // (4) Both have non-empty last-modified timestamps
        assert!(!cd_meta.last_modified.is_empty());
    }

    // ─── T015: Version Default Consistency (US2) ────────────────────────

    #[test]
    fn test_version_default_consistency() {
        // EC-2, US2-AC-2: Empty version defaults to "0.0.0"
        let doc = test_document("Test Policy", "");
        let envelope = build_component_definition(&doc, None, None, None).unwrap();
        assert_eq!(envelope.component_definition.metadata.version, "0.0.0");
    }

    #[test]
    fn test_empty_version_metadata_consistency_with_assemble_metadata() {
        // The builder resolves defaults BEFORE calling assemble_metadata,
        // so both the builder and assemble_metadata see the same "0.0.0" version.
        let doc = test_document("Test Policy", "");
        let cd_envelope = build_component_definition(&doc, None, None, None).unwrap();
        let cd_meta = &cd_envelope.component_definition.metadata;

        // Builder defaults empty version to "0.0.0" and passes it to assemble_metadata
        assert_eq!(cd_meta.version, "0.0.0");

        // Verify consistency: build with same empty version doc and compare
        // assemble_metadata (with resolved defaults) against ComponentDefinitionMetadata
        let resolved_meta =
            DocumentMetadata { version: "0.0.0".to_string(), ..doc.metadata.clone() };
        let catalog_metadata = assemble_metadata(&resolved_meta, None).unwrap();
        assert_eq!(catalog_metadata.version, cd_meta.version);
    }

    #[test]
    fn test_component_uuid_differs_by_document_id() {
        // Same title/version but different document IDs should produce different UUIDs
        let doc1 = PolicyDocument {
            id: "doc-alpha".to_string(),
            metadata: DocumentMetadata {
                title: "Same Policy".to_string(),
                version: "1.0".to_string(),
                author: None,
                date: None,
                source_path: PathBuf::from("test.md"),
                content_hash: None,
            },
            sections: vec![],
        };
        let doc2 = PolicyDocument { id: "doc-beta".to_string(), ..doc1.clone() };

        let env1 = build_component_definition(&doc1, None, None, None).unwrap();
        let env2 = build_component_definition(&doc2, None, None, None).unwrap();

        assert_ne!(
            env1.component_definition.components[0].uuid,
            env2.component_definition.components[0].uuid,
            "Different document IDs should produce different component UUIDs"
        );
    }

    // ─── T015: DocumentaryComponent source-file prop (WI-17) ─────────────

    #[test]
    fn test_documentary_component_source_file_prop() {
        let doc = test_document("Test Policy", "1.0");
        let envelope =
            build_component_definition(&doc, None, None, Some("policies/security.md")).unwrap();
        let json = serde_json::to_string_pretty(&envelope).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();

        let comp = &v["component-definition"]["components"][0];
        let props = comp["props"].as_array().expect("Must have props array");
        assert_eq!(props.len(), 1, "Must have exactly 1 source-file prop");

        assert_eq!(props[0]["name"].as_str(), Some("source-file"));
        assert_eq!(props[0]["ns"].as_str(), Some("https://forge.policy-forge.github.io/ns/trace"));
        assert_eq!(props[0]["value"].as_str(), Some("policies/security.md"));
    }

    #[test]
    fn test_documentary_component_no_props_without_source_file() {
        let doc = test_document("Test Policy", "1.0");
        let envelope = build_component_definition(&doc, None, None, None).unwrap();
        let json = serde_json::to_string_pretty(&envelope).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();

        let comp = &v["component-definition"]["components"][0];
        // When no source_file, props should be absent (skip_serializing_if = "Vec::is_empty")
        assert!(comp.get("props").is_none(), "No props when source_file is None");
    }

    // ─── T012: build_component_definition with source_profile (WI-15) ────

    #[test]
    fn test_none_source_profile_empty_control_implementations() {
        let doc = test_document("Test Policy", "1.0");
        let envelope = build_component_definition(&doc, None, None, None).unwrap();
        let comp = &envelope.component_definition.components[0];
        assert!(
            comp.control_implementations.is_empty(),
            "None source_profile must produce empty control-implementations"
        );
    }

    #[test]
    fn test_some_source_profile_populated_control_implementations() {
        let doc = test_document_with_sections(
            "Test Policy",
            "1.0",
            vec![test_section(
                "Access Control",
                vec![PolicyRequirement {
                    text: "Users shall authenticate.".to_string(),
                    source_line: 1,
                    nesting_depth: 0,
                    stable_id: Some("uuid-1".to_string()),
                    atom_index: 0,
                    parent_text: None,
                    citations: vec![],
                    modality: None,
                    parameters: vec![],
                    parameters_extracted: false,
                }],
            )],
        );
        let envelope =
            build_component_definition(&doc, Some("./baseline.json"), None, Some("test.md"))
                .unwrap();
        let comp = &envelope.component_definition.components[0];
        assert!(
            !comp.control_implementations.is_empty(),
            "Some source_profile must produce populated control-implementations"
        );

        let ci = &comp.control_implementations[0];
        assert_eq!(ci.source, "baseline.json");
    }

    #[test]
    fn test_source_profile_without_source_file_omits_requirement_trace_data() {
        let doc = test_document_with_sections(
            "Test Policy",
            "1.0",
            vec![test_section(
                "Access Control",
                vec![PolicyRequirement {
                    text: "Users shall authenticate.".to_string(),
                    source_line: 1,
                    nesting_depth: 0,
                    stable_id: Some("uuid-1".to_string()),
                    atom_index: 0,
                    parent_text: None,
                    citations: vec![],
                    modality: None,
                    parameters: vec![],
                    parameters_extracted: false,
                }],
            )],
        );

        let envelope =
            build_component_definition(&doc, Some("./baseline.json"), None, None).unwrap();
        let implemented_requirement = &envelope.component_definition.components[0]
            .control_implementations[0]
            .implemented_requirements[0];
        assert!(implemented_requirement.props.is_empty());
        assert!(implemented_requirement.links.is_empty());
    }

    // ─── T022: No trace data in remarks (Component Definition) — SEC-1, SEC-2, M-7 ──

    use crate::oscal::test_utils::collect_remarks;

    #[test]
    fn test_no_trace_data_in_remarks_component_definition() {
        let doc = test_document_with_sections(
            "Test Policy",
            "1.0",
            vec![test_section(
                "Access Control",
                vec![PolicyRequirement {
                    text: "Users shall authenticate.".to_string(),
                    source_line: 42,
                    nesting_depth: 0,
                    stable_id: Some("uuid-1".to_string()),
                    atom_index: 0,
                    parent_text: None,
                    citations: vec![],
                    modality: None,
                    parameters: vec![],
                    parameters_extracted: false,
                }],
            )],
        );
        let envelope = build_component_definition(
            &doc,
            Some("./baseline.json"),
            None,
            Some("policies/security.md"),
        )
        .unwrap();

        let json_str = serde_json::to_string_pretty(&envelope).unwrap();
        let json_val: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        let mut remarks = Vec::new();
        collect_remarks(&json_val, &mut remarks);

        let trace_patterns =
            ["policies/security.md", "Access Control", "42", "source-file", "source-line"];
        for remark in &remarks {
            for pattern in &trace_patterns {
                assert!(
                    !remark.contains(pattern),
                    "Remarks must not contain trace data '{pattern}'. Found in: {remark}"
                );
            }
        }
    }

    // ─── T009: collect_control_ids_from_component_def ──────────────────

    #[test]
    fn collect_control_ids_from_component_def_basic() {
        use crate::oscal::implemented_requirements::{
            ControlImplementation, ImplementedRequirement,
        };

        let envelope = ComponentDefinitionEnvelope {
            component_definition: ComponentDefinition {
                uuid: "test-uuid".into(),
                metadata: ComponentDefinitionMetadata {
                    title: "Test".into(),
                    last_modified: "2026-01-01T00:00:00Z".into(),
                    version: "1.0.0".into(),
                    oscal_version: "1.2.0".into(),
                },
                capabilities: vec![],
                components: vec![DocumentaryComponent {
                    uuid: "comp-uuid".into(),
                    component_type: "policy".into(),
                    title: "Test Component".into(),
                    description: "Desc".into(),
                    props: vec![],
                    control_implementations: vec![ControlImplementation {
                        uuid: "ci-uuid".into(),
                        source: "./profile.json".into(),
                        description: "impl".into(),
                        implemented_requirements: vec![
                            ImplementedRequirement {
                                uuid: "ir1".into(),
                                control_id: "ac-1".into(),
                                description: "d1".into(),
                                props: vec![],
                                links: vec![],
                            },
                            ImplementedRequirement {
                                uuid: "ir2".into(),
                                control_id: "ac-2".into(),
                                description: "d2".into(),
                                props: vec![],
                                links: vec![],
                            },
                        ],
                    }],
                }],
                back_matter: None,
            },
        };

        let ids = collect_control_ids_from_component_def(&envelope);
        assert_eq!(ids, vec!["ac-1", "ac-2"]);
    }

    #[test]
    fn collect_control_ids_from_component_def_empty() {
        let envelope = ComponentDefinitionEnvelope {
            component_definition: ComponentDefinition {
                uuid: "test-uuid".into(),
                metadata: ComponentDefinitionMetadata {
                    title: "Test".into(),
                    last_modified: "2026-01-01T00:00:00Z".into(),
                    version: "1.0.0".into(),
                    oscal_version: "1.2.0".into(),
                },
                capabilities: vec![],
                components: vec![DocumentaryComponent {
                    uuid: "comp-uuid".into(),
                    component_type: "policy".into(),
                    title: "Test Component".into(),
                    description: "Desc".into(),
                    props: vec![],
                    control_implementations: vec![],
                }],
                back_matter: None,
            },
        };

        let ids = collect_control_ids_from_component_def(&envelope);
        assert!(ids.is_empty());
    }

    // ── collect_control_ids includes capabilities ─

    #[test]
    fn collect_control_ids_from_capabilities() {
        use crate::oscal::implemented_requirements::{
            ControlImplementation, ImplementedRequirement,
        };

        let envelope = ComponentDefinitionEnvelope {
            component_definition: ComponentDefinition {
                uuid: "test-uuid".into(),
                metadata: ComponentDefinitionMetadata {
                    title: "Test".into(),
                    last_modified: "2026-01-01T00:00:00Z".into(),
                    version: "1.0.0".into(),
                    oscal_version: "1.2.0".into(),
                },
                capabilities: vec![Capability {
                    uuid: "cap-uuid".into(),
                    name: "Encryption".into(),
                    description: "Encryption capability".into(),
                    control_implementations: vec![ControlImplementation {
                        uuid: "ci-uuid".into(),
                        source: "./baseline.json".into(),
                        description: "CI".into(),
                        implemented_requirements: vec![ImplementedRequirement {
                            uuid: "ir-1".into(),
                            control_id: "enc-1".into(),
                            description: "Encrypt data".into(),
                            props: vec![],
                            links: vec![],
                        }],
                    }],
                }],
                components: vec![DocumentaryComponent {
                    uuid: "comp-uuid".into(),
                    component_type: "policy".into(),
                    title: "Test Component".into(),
                    description: "Desc".into(),
                    props: vec![],
                    control_implementations: vec![ControlImplementation {
                        uuid: "ci-uuid-2".into(),
                        source: "./baseline.json".into(),
                        description: "CI".into(),
                        implemented_requirements: vec![ImplementedRequirement {
                            uuid: "ir-2".into(),
                            control_id: "ac-1".into(),
                            description: "Access control".into(),
                            props: vec![],
                            links: vec![],
                        }],
                    }],
                }],
                back_matter: None,
            },
        };

        let ids = collect_control_ids_from_component_def(&envelope);
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"ac-1".to_string()));
        assert!(ids.contains(&"enc-1".to_string()));
    }

    // ── Task 10: Capability struct and capabilities field ─

    #[test]
    fn component_definition_round_trips_capabilities() {
        let json = r#"{
            "component-definition": {
                "uuid": "test-uuid",
                "metadata": {
                    "title": "Test",
                    "last-modified": "2026-01-01T00:00:00Z",
                    "version": "1.0",
                    "oscal-version": "1.2.0"
                },
                "components": [],
                "capabilities": [{
                    "uuid": "cap-uuid",
                    "name": "Encryption Capability",
                    "description": "Provides data-at-rest encryption"
                }]
            }
        }"#;
        let envelope: ComponentDefinitionEnvelope = serde_json::from_str(json).unwrap();
        assert_eq!(envelope.component_definition.capabilities.len(), 1);
        assert_eq!(envelope.component_definition.capabilities[0].name, "Encryption Capability");
        let reserialized = serde_json::to_string(&envelope).unwrap();
        let re_parsed: ComponentDefinitionEnvelope = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(re_parsed.component_definition.capabilities.len(), 1);
    }
}
