use crate::error::ForgeError;
use crate::types::OscalModelType;
use crate::validate;

use super::extractor::extract_trace_metadata;
use super::report::{ElementType, TraceEntry};

/// Detect whether a parsed JSON value is a Catalog or Component Definition.
///
/// Delegates to [`validate::detect_model_type`] for core detection, then applies
/// trace-specific validation: rejects `Profile` artifacts and verifies the
/// top-level value is a JSON object.
///
/// # Errors
///
/// Returns `ForgeError::TraceUnsupportedArtifact` if the model type is not
/// recognized, is a Profile, or the top-level value is not a JSON object.
pub fn detect_artifact_type(json: &serde_json::Value) -> Result<OscalModelType, ForgeError> {
    let model_type =
        validate::detect_model_type(json).map_err(|_| ForgeError::TraceUnsupportedArtifact {
            detail:
                "Expected top-level key 'catalog' or 'component-definition' with an object value"
                    .to_string(),
        })?;

    if model_type == OscalModelType::Profile {
        return Err(ForgeError::TraceUnsupportedArtifact {
            detail: "Profile artifacts are not supported for traceability".to_string(),
        });
    }

    // Verify the top-level value is a JSON object (stricter than detect_model_type)
    let key = model_type.as_str();
    if !json.get(key).is_some_and(serde_json::Value::is_object) {
        return Err(ForgeError::TraceUnsupportedArtifact {
            detail: format!("Expected top-level key '{key}' with an object value"),
        });
    }

    // Reject ambiguous artifacts with multiple supported top-level keys
    let has_catalog = json.get("catalog").is_some_and(serde_json::Value::is_object);
    let has_compdef = json.get("component-definition").is_some_and(serde_json::Value::is_object);
    if has_catalog && has_compdef {
        return Err(ForgeError::TraceUnsupportedArtifact {
            detail: "Ambiguous artifact: both 'catalog' and 'component-definition' keys present"
                .to_string(),
        });
    }

    Ok(model_type)
}

/// Walk a Catalog's groups and controls, extracting trace entries.
///
/// Recursively traverses nested groups and controls per the OSCAL catalog model.
/// Parts are excluded.
#[must_use]
pub fn walk_catalog_elements(catalog: &serde_json::Value) -> Vec<TraceEntry> {
    let mut entries = Vec::new();

    if let Some(groups) = catalog.get("groups").and_then(|g| g.as_array()) {
        for group in groups {
            walk_group(group, &mut entries);
        }
    }

    // OSCAL allows controls directly under catalog (no group parent)
    if let Some(controls) = catalog.get("controls").and_then(|c| c.as_array()) {
        for control in controls {
            walk_control(control, &mut entries);
        }
    }

    entries
}

/// Recursively walk a group: emit its entry, then recurse into child groups and controls.
fn walk_group(group: &serde_json::Value, entries: &mut Vec<TraceEntry>) {
    let group_id = group.get("id").and_then(|v| v.as_str()).unwrap_or("unknown-group").to_string();
    entries.push(TraceEntry {
        element_id: group_id,
        element_type: ElementType::Group,
        trace: extract_trace_metadata(group),
    });

    if let Some(child_groups) = group.get("groups").and_then(|g| g.as_array()) {
        for child in child_groups {
            walk_group(child, entries);
        }
    }

    if let Some(controls) = group.get("controls").and_then(|c| c.as_array()) {
        for control in controls {
            walk_control(control, entries);
        }
    }
}

/// Recursively walk a control: emit its entry, then recurse into child controls.
fn walk_control(control: &serde_json::Value, entries: &mut Vec<TraceEntry>) {
    let control_id =
        control.get("id").and_then(|v| v.as_str()).unwrap_or("unknown-control").to_string();
    entries.push(TraceEntry {
        element_id: control_id,
        element_type: ElementType::Control,
        trace: extract_trace_metadata(control),
    });

    // OSCAL allows nested controls (e.g. control enhancements)
    if let Some(children) = control.get("controls").and_then(|c| c.as_array()) {
        for child in children {
            walk_control(child, entries);
        }
    }
}

/// Walk a Component Definition's components and capabilities,
/// extracting implemented-requirement trace entries from both contexts.
#[must_use]
pub fn walk_compdef_elements(compdef: &serde_json::Value) -> Vec<TraceEntry> {
    let mut entries = Vec::new();

    if let Some(components) = compdef.get("components").and_then(|c| c.as_array()) {
        for component in components {
            collect_impl_requirements(component, &mut entries);
        }
    }

    // OSCAL component-definitions may also group implementations under capabilities
    if let Some(capabilities) = compdef.get("capabilities").and_then(|c| c.as_array()) {
        for capability in capabilities {
            collect_impl_requirements(capability, &mut entries);
        }
    }

    entries
}

/// Collect implemented-requirements from a component or capability's control-implementations.
fn collect_impl_requirements(container: &serde_json::Value, entries: &mut Vec<TraceEntry>) {
    let Some(control_impls) = container.get("control-implementations").and_then(|ci| ci.as_array())
    else {
        return;
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
        assert!(matches!(detect_artifact_type(&json), Ok(OscalModelType::Catalog)));
    }

    #[test]
    fn detect_component_definition() {
        let json = json!({ "component-definition": { "uuid": "456" } });
        assert!(matches!(detect_artifact_type(&json), Ok(OscalModelType::ComponentDefinition)));
    }

    #[test]
    fn detect_unsupported_type() {
        let json = json!({ "profile": { "uuid": "789" } });
        assert!(matches!(
            detect_artifact_type(&json),
            Err(ForgeError::TraceUnsupportedArtifact { .. })
        ));
    }

    #[test]
    fn detect_rejects_non_object_catalog() {
        let json = json!({ "catalog": null });
        assert!(matches!(
            detect_artifact_type(&json),
            Err(ForgeError::TraceUnsupportedArtifact { .. })
        ));

        let json = json!({ "catalog": "not-an-object" });
        assert!(matches!(
            detect_artifact_type(&json),
            Err(ForgeError::TraceUnsupportedArtifact { .. })
        ));

        let json = json!({ "catalog": [1, 2, 3] });
        assert!(matches!(
            detect_artifact_type(&json),
            Err(ForgeError::TraceUnsupportedArtifact { .. })
        ));
    }

    #[test]
    fn detect_rejects_ambiguous_artifact() {
        let json = json!({ "catalog": { "uuid": "1" }, "component-definition": { "uuid": "2" } });
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
