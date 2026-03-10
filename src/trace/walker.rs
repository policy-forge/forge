use crate::error::ForgeError;

use super::extractor::extract_trace_metadata;
use super::report::{ArtifactType, ElementType, TraceEntry};

/// Detect whether a parsed JSON value is a Catalog or Component Definition.
///
/// # Errors
///
/// Returns `ForgeError::TraceUnsupportedArtifact` if neither top-level key is found.
pub fn detect_artifact_type(json: &serde_json::Value) -> Result<ArtifactType, ForgeError> {
    if json.get("catalog").is_some() {
        Ok(ArtifactType::Catalog)
    } else if json.get("component-definition").is_some() {
        Ok(ArtifactType::ComponentDefinition)
    } else {
        Err(ForgeError::TraceUnsupportedArtifact {
            detail: "Expected top-level key 'catalog' or 'component-definition'".to_string(),
        })
    }
}

/// Walk a Catalog's groups and controls, extracting trace entries.
///
/// Yields: groups (`element_type` "group") then controls (`element_type` "control")
/// within each group. Parts are excluded.
#[must_use]
pub fn walk_catalog_elements(catalog: &serde_json::Value) -> Vec<TraceEntry> {
    let mut entries = Vec::new();

    let Some(groups) = catalog.get("groups").and_then(|g| g.as_array()) else {
        return entries;
    };

    for group in groups {
        let group_id =
            group.get("id").and_then(|v| v.as_str()).unwrap_or("unknown-group").to_string();
        let group_trace = extract_trace_metadata(group);
        entries.push(TraceEntry {
            element_id: group_id,
            element_type: ElementType::Group,
            trace: group_trace,
        });

        if let Some(controls) = group.get("controls").and_then(|c| c.as_array()) {
            for control in controls {
                let control_id = control
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown-control")
                    .to_string();
                let control_trace = extract_trace_metadata(control);
                entries.push(TraceEntry {
                    element_id: control_id,
                    element_type: ElementType::Control,
                    trace: control_trace,
                });
            }
        }
    }

    entries
}

/// Walk a Component Definition's components, control-implementations,
/// and implemented-requirements, extracting trace entries.
#[must_use]
pub fn walk_compdef_elements(compdef: &serde_json::Value) -> Vec<TraceEntry> {
    let mut entries = Vec::new();

    let Some(components) = compdef.get("components").and_then(|c| c.as_array()) else {
        return entries;
    };

    for component in components {
        let Some(control_impls) =
            component.get("control-implementations").and_then(|ci| ci.as_array())
        else {
            continue;
        };

        for impl_block in control_impls {
            let Some(impl_reqs) =
                impl_block.get("implemented-requirements").and_then(|ir| ir.as_array())
            else {
                continue;
            };

            for req in impl_reqs {
                let control_id = req
                    .get("control-id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown-requirement")
                    .to_string();
                let trace = extract_trace_metadata(req);
                entries.push(TraceEntry {
                    element_id: control_id,
                    element_type: ElementType::ImplementedRequirement,
                    trace,
                });
            }
        }
    }

    entries
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oscal::trace_embedding::FORGE_TRACE_NS;
    use serde_json::json;

    const NS: &str = FORGE_TRACE_NS;

    // T012: detect_artifact_type tests

    #[test]
    fn detect_catalog() {
        let json = json!({ "catalog": { "uuid": "123" } });
        assert!(matches!(detect_artifact_type(&json), Ok(ArtifactType::Catalog)));
    }

    #[test]
    fn detect_component_definition() {
        let json = json!({ "component-definition": { "uuid": "456" } });
        assert!(matches!(detect_artifact_type(&json), Ok(ArtifactType::ComponentDefinition)));
    }

    #[test]
    fn detect_unsupported_type() {
        let json = json!({ "profile": { "uuid": "789" } });
        assert!(matches!(
            detect_artifact_type(&json),
            Err(ForgeError::TraceUnsupportedArtifact { .. })
        ));
    }

    // T013: walk_catalog_elements tests

    #[test]
    fn walk_catalog_groups_and_controls() {
        let catalog = json!({
            "groups": [
                {
                    "id": "access-control",
                    "title": "Access Control",
                    "props": [
                        { "name": "source-section", "ns": NS, "value": "Access Control" }
                    ],
                    "controls": [
                        {
                            "id": "POL-AC-001",
                            "props": [
                                { "name": "source-file", "ns": NS, "value": "policy.md" },
                                { "name": "source-section", "ns": NS, "value": "Access Control" },
                                { "name": "source-line", "ns": NS, "value": "10" }
                            ]
                        },
                        {
                            "id": "POL-AC-002",
                            "props": [
                                { "name": "source-file", "ns": NS, "value": "policy.md" },
                                { "name": "source-section", "ns": NS, "value": "Access Control" },
                                { "name": "source-line", "ns": NS, "value": "25" }
                            ]
                        }
                    ]
                },
                {
                    "id": "data-protection",
                    "title": "Data Protection",
                    "props": [
                        { "name": "source-section", "ns": NS, "value": "Data Protection" }
                    ],
                    "controls": [
                        {
                            "id": "POL-DP-001",
                            "props": [
                                { "name": "source-file", "ns": NS, "value": "policy.md" },
                                { "name": "source-section", "ns": NS, "value": "Data Protection" },
                                { "name": "source-line", "ns": NS, "value": "50" }
                            ]
                        }
                    ]
                }
            ]
        });

        let entries = walk_catalog_elements(&catalog);
        assert_eq!(entries.len(), 5); // 2 groups + 3 controls

        assert_eq!(entries[0].element_type, ElementType::Group);
        assert_eq!(entries[0].element_id, "access-control");

        assert_eq!(entries[1].element_type, ElementType::Control);
        assert_eq!(entries[1].element_id, "POL-AC-001");

        assert_eq!(entries[2].element_type, ElementType::Control);
        assert_eq!(entries[2].element_id, "POL-AC-002");

        assert_eq!(entries[3].element_type, ElementType::Group);
        assert_eq!(entries[3].element_id, "data-protection");

        assert_eq!(entries[4].element_type, ElementType::Control);
        assert_eq!(entries[4].element_id, "POL-DP-001");
    }

    #[test]
    fn walk_empty_catalog() {
        let catalog = json!({ "groups": [] });
        let entries = walk_catalog_elements(&catalog);
        assert_eq!(entries.len(), 0);
    }

    #[test]
    fn walk_catalog_no_groups_key() {
        let catalog = json!({ "uuid": "123" });
        let entries = walk_catalog_elements(&catalog);
        assert_eq!(entries.len(), 0);
    }

    // T014: walk_compdef_elements tests

    #[test]
    fn walk_compdef_implemented_requirements() {
        let compdef = json!({
            "components": [
                {
                    "uuid": "comp-1",
                    "type": "policy",
                    "control-implementations": [
                        {
                            "uuid": "ci-1",
                            "implemented-requirements": [
                                {
                                    "uuid": "ir-1",
                                    "control-id": "POL-AC-001",
                                    "props": [
                                        { "name": "source-file", "ns": NS, "value": "policy.md" },
                                        { "name": "source-section", "ns": NS, "value": "Access Control" },
                                        { "name": "source-line", "ns": NS, "value": "10" }
                                    ]
                                },
                                {
                                    "uuid": "ir-2",
                                    "control-id": "POL-AC-002",
                                    "props": [
                                        { "name": "source-file", "ns": NS, "value": "policy.md" },
                                        { "name": "source-section", "ns": NS, "value": "Access Control" },
                                        { "name": "source-line", "ns": NS, "value": "25" }
                                    ]
                                },
                                {
                                    "uuid": "ir-3",
                                    "control-id": "POL-DP-001",
                                    "props": [
                                        { "name": "source-file", "ns": NS, "value": "policy.md" },
                                        { "name": "source-section", "ns": NS, "value": "Data Protection" },
                                        { "name": "source-line", "ns": NS, "value": "50" }
                                    ]
                                }
                            ]
                        }
                    ]
                }
            ]
        });

        let entries = walk_compdef_elements(&compdef);
        assert_eq!(entries.len(), 3);

        for entry in &entries {
            assert_eq!(entry.element_type, ElementType::ImplementedRequirement);
        }
        assert_eq!(entries[0].element_id, "POL-AC-001");
        assert_eq!(entries[1].element_id, "POL-AC-002");
        assert_eq!(entries[2].element_id, "POL-DP-001");
    }

    #[test]
    fn walk_compdef_empty_components() {
        let compdef = json!({ "components": [] });
        let entries = walk_compdef_elements(&compdef);
        assert_eq!(entries.len(), 0);
    }
}
