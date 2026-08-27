use std::collections::HashMap;

use serde_json::Value;

use super::types::{ArtifactType, ControlSnapshot};

/// Extract control snapshots from an OSCAL artifact JSON value.
///
/// Dispatches to the appropriate extractor based on [`ArtifactType`],
/// producing a map of control ID → [`ControlSnapshot`].
///
/// For Catalogs, walks groups recursively (including nested groups).
/// For Component Definitions, walks components and capabilities.
#[must_use]
pub fn extract_controls(
    json: &Value,
    artifact_type: &ArtifactType,
) -> HashMap<String, ControlSnapshot> {
    match artifact_type {
        ArtifactType::Catalog => extract_catalog_controls(json),
        ArtifactType::ComponentDefinition => extract_component_def_controls(json),
    }
}

const EMPTY_ARRAY: &[Value] = &[];

fn extract_catalog_controls(json: &Value) -> HashMap<String, ControlSnapshot> {
    let mut map = HashMap::new();
    let controls = json
        .pointer("/catalog/controls")
        .and_then(Value::as_array)
        .map_or(EMPTY_ARRAY, Vec::as_slice);
    collect_controls(controls, &mut map);
    let groups = json
        .pointer("/catalog/groups")
        .and_then(Value::as_array)
        .map_or(EMPTY_ARRAY, Vec::as_slice);
    collect_controls_from_groups(groups, &mut map);
    map
}

fn collect_controls_from_groups(groups: &[Value], map: &mut HashMap<String, ControlSnapshot>) {
    for group in groups {
        if let Some(controls) = group.get("controls").and_then(Value::as_array) {
            collect_controls(controls, map);
        }
        if let Some(nested) = group.get("groups").and_then(Value::as_array) {
            collect_controls_from_groups(nested, map);
        }
    }
}

fn collect_controls(controls: &[Value], map: &mut HashMap<String, ControlSnapshot>) {
    for control in controls {
        let Some(id) = control.get("id").and_then(Value::as_str) else {
            continue;
        };
        let uuid = control.get("uuid").and_then(Value::as_str).unwrap_or("").to_string();
        let title = control.get("title").and_then(Value::as_str).map(String::from);
        let parts_prose = collect_statement_prose(control);
        if map.contains_key(id) {
            tracing::warn!(
                control_id = id,
                "Duplicate control-id in catalog; last occurrence wins"
            );
        }
        map.insert(
            id.to_string(),
            ControlSnapshot {
                control_id: id.to_string(),
                uuid,
                title,
                description: None,
                parts_prose,
            },
        );
    }
}

fn collect_statement_prose(control: &Value) -> Vec<String> {
    let Some(parts) = control.get("parts").and_then(Value::as_array) else {
        return vec![];
    };
    parts
        .iter()
        .filter(|p| p.get("name").and_then(Value::as_str) == Some("statement"))
        .filter_map(|p| p.get("prose").and_then(Value::as_str))
        .map(String::from)
        .collect()
}

fn extract_component_def_controls(json: &Value) -> HashMap<String, ControlSnapshot> {
    let mut map = HashMap::new();
    let components = json
        .pointer("/component-definition/components")
        .and_then(Value::as_array)
        .map_or(EMPTY_ARRAY, Vec::as_slice);
    for component in components {
        collect_impl_requirements_from_container(component, &mut map);
    }

    // OSCAL component-definitions may also group implementations under capabilities
    let capabilities = json
        .pointer("/component-definition/capabilities")
        .and_then(Value::as_array)
        .map_or(EMPTY_ARRAY, Vec::as_slice);
    for capability in capabilities {
        collect_impl_requirements_from_container(capability, &mut map);
    }

    map
}

/// Extract control snapshots from a component or capability's control-implementations.
fn collect_impl_requirements_from_container(
    container: &Value,
    map: &mut HashMap<String, ControlSnapshot>,
) {
    let cis = container
        .get("control-implementations")
        .and_then(Value::as_array)
        .map_or(EMPTY_ARRAY, Vec::as_slice);
    for ci in cis {
        let irs = ci
            .get("implemented-requirements")
            .and_then(Value::as_array)
            .map_or(EMPTY_ARRAY, Vec::as_slice);
        for ir in irs {
            let Some(control_id) = ir.get("control-id").and_then(Value::as_str) else {
                continue;
            };
            let uuid = ir.get("uuid").and_then(Value::as_str).unwrap_or("").to_string();
            let description = ir.get("description").and_then(Value::as_str).map(String::from);

            if let Some(snapshot) = map.get_mut(control_id) {
                tracing::warn!(
                    control_id,
                    "Duplicate control-id in component definition; aggregating descriptions"
                );
                if let Some(description) = description {
                    if let Some(existing) = &mut snapshot.description {
                        existing.push('\n');
                        existing.push_str(&description);
                    } else {
                        snapshot.description = Some(description);
                    }
                }
            } else {
                map.insert(
                    control_id.to_string(),
                    ControlSnapshot {
                        control_id: control_id.to_string(),
                        uuid,
                        title: None,
                        description,
                        parts_prose: vec![],
                    },
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_catalog_json(controls: &[(&str, &str, &str)]) -> Value {
        let controls_json: Vec<_> = controls
            .iter()
            .map(|(id, title, prose)| {
                serde_json::json!({
                    "id": id,
                    "title": title,
                    "parts": [{"name": "statement", "id": format!("{id}_smt"), "prose": prose}]
                })
            })
            .collect();
        serde_json::json!({
            "catalog": {
                "uuid": "test-uuid",
                "metadata": {"title": "Test", "last-modified": "2026-01-01T00:00:00Z",
                             "version": "1.0", "oscal-version": "1.2.0"},
                "groups": [{"id": "test", "title": "Test", "controls": controls_json}]
            }
        })
    }

    #[test]
    fn extract_catalog_flat_group() {
        let json = make_catalog_json(&[
            ("POL-AC-001", "Access Control", "Users shall authenticate."),
            ("POL-AC-002", "Password Policy", "Passwords shall be complex."),
        ]);
        let result = extract_controls(&json, &ArtifactType::Catalog);
        assert_eq!(result.len(), 2);
        let snap = &result["POL-AC-001"];
        assert_eq!(snap.control_id, "POL-AC-001");
        assert_eq!(snap.title.as_deref(), Some("Access Control"));
        assert_eq!(snap.parts_prose, vec!["Users shall authenticate."]);
    }

    #[test]
    fn extract_catalog_nested_groups() {
        let json = serde_json::json!({
            "catalog": {
                "uuid": "test-uuid",
                "metadata": {"title": "Test", "last-modified": "2026-01-01T00:00:00Z",
                             "version": "1.0", "oscal-version": "1.2.0"},
                "groups": [{
                    "id": "outer", "title": "Outer",
                    "groups": [{
                        "id": "inner", "title": "Inner",
                        "controls": [{
                            "id": "POL-NEST-001",
                            "title": "Nested Control",
                            "parts": [{"name": "statement", "id": "s1", "prose": "Nested prose."}]
                        }]
                    }]
                }]
            }
        });
        let result = extract_controls(&json, &ArtifactType::Catalog);
        assert_eq!(result.len(), 1);
        assert!(result.contains_key("POL-NEST-001"));
    }

    #[test]
    fn extract_catalog_empty_groups() {
        let json = serde_json::json!({
            "catalog": {
                "uuid": "test-uuid",
                "metadata": {"title": "Test", "last-modified": "2026-01-01T00:00:00Z",
                             "version": "1.0", "oscal-version": "1.2.0"},
                "groups": []
            }
        });
        let result = extract_controls(&json, &ArtifactType::Catalog);
        assert!(result.is_empty());
    }

    #[test]
    fn extract_catalog_missing_id_skipped() {
        let json = serde_json::json!({
            "catalog": {
                "uuid": "test-uuid",
                "metadata": {"title": "Test", "last-modified": "2026-01-01T00:00:00Z",
                             "version": "1.0", "oscal-version": "1.2.0"},
                "groups": [{"id": "g1", "title": "G1", "controls": [
                    {"title": "No ID", "parts": []},
                    {"id": "POL-VALID", "title": "Valid", "parts": []}
                ]}]
            }
        });
        let result = extract_controls(&json, &ArtifactType::Catalog);
        assert_eq!(result.len(), 1);
        assert!(result.contains_key("POL-VALID"));
    }

    // --- US3: Component Definition extractor tests (T023) ---

    fn make_component_def_json(reqs: &[(&str, &str, &str)]) -> Value {
        let reqs_json: Vec<_> = reqs
            .iter()
            .map(|(cid, uuid, desc)| {
                serde_json::json!({"uuid": uuid, "control-id": cid, "description": desc})
            })
            .collect();
        serde_json::json!({
            "component-definition": {
                "uuid": "cd-uuid",
                "metadata": {"title": "Test", "last-modified": "2026-01-01T00:00:00Z",
                             "version": "1.0", "oscal-version": "1.2.0"},
                "components": [{
                    "uuid": "comp-uuid", "type": "policy", "title": "Test",
                    "description": "Test",
                    "control-implementations": [{
                        "uuid": "ci-uuid", "source": "./baseline.json",
                        "description": "Test",
                        "implemented-requirements": reqs_json
                    }]
                }]
            }
        })
    }

    #[test]
    fn extract_component_def_populated() {
        let json = make_component_def_json(&[
            ("POL-AC-001", "uuid-1", "Impl desc 1"),
            ("POL-AC-002", "uuid-2", "Impl desc 2"),
        ]);
        let result = extract_controls(&json, &ArtifactType::ComponentDefinition);
        assert_eq!(result.len(), 2);
        let snap = &result["POL-AC-001"];
        assert_eq!(snap.uuid, "uuid-1");
        assert_eq!(snap.description.as_deref(), Some("Impl desc 1"));
        assert!(snap.title.is_none());
        assert!(snap.parts_prose.is_empty());
    }

    #[test]
    fn extract_component_def_multiple_control_implementations() {
        let json = serde_json::json!({
            "component-definition": {
                "uuid": "cd-uuid",
                "metadata": {"title": "Test", "last-modified": "2026-01-01T00:00:00Z",
                             "version": "1.0", "oscal-version": "1.2.0"},
                "components": [{
                    "uuid": "comp-uuid", "type": "policy", "title": "Test",
                    "description": "Test",
                    "control-implementations": [
                        {
                            "uuid": "ci-1", "source": "./b1.json", "description": "CI1",
                            "implemented-requirements": [
                                {"uuid": "u1", "control-id": "POL-AC-001", "description": "D1"}
                            ]
                        },
                        {
                            "uuid": "ci-2", "source": "./b2.json", "description": "CI2",
                            "implemented-requirements": [
                                {"uuid": "u2", "control-id": "POL-AC-002", "description": "D2"}
                            ]
                        }
                    ]
                }]
            }
        });
        let result = extract_controls(&json, &ArtifactType::ComponentDefinition);
        assert_eq!(result.len(), 2);
        assert!(result.contains_key("POL-AC-001"));
        assert!(result.contains_key("POL-AC-002"));
    }

    #[test]
    fn extract_component_def_missing_control_id_skipped() {
        let json = serde_json::json!({
            "component-definition": {
                "uuid": "cd-uuid",
                "metadata": {"title": "Test", "last-modified": "2026-01-01T00:00:00Z",
                             "version": "1.0", "oscal-version": "1.2.0"},
                "components": [{
                    "uuid": "comp-uuid", "type": "policy", "title": "Test",
                    "description": "Test",
                    "control-implementations": [{
                        "uuid": "ci-uuid", "source": "./b.json", "description": "CI",
                        "implemented-requirements": [
                            {"uuid": "u1", "description": "No control-id"},
                            {"uuid": "u2", "control-id": "POL-VALID", "description": "Has ID"}
                        ]
                    }]
                }]
            }
        });
        let result = extract_controls(&json, &ArtifactType::ComponentDefinition);
        assert_eq!(result.len(), 1);
        assert!(result.contains_key("POL-VALID"));
    }

    #[test]
    fn extract_component_def_from_capabilities() {
        let json = serde_json::json!({
            "component-definition": {
                "uuid": "cd-uuid",
                "metadata": {"title": "Test", "last-modified": "2026-01-01T00:00:00Z",
                             "version": "1.0", "oscal-version": "1.2.0"},
                "components": [],
                "capabilities": [{
                    "uuid": "cap-uuid",
                    "name": "Encryption",
                    "description": "Encryption capability",
                    "control-implementations": [{
                        "uuid": "ci-uuid", "source": "./b.json", "description": "CI",
                        "implemented-requirements": [
                            {"uuid": "u1", "control-id": "POL-ENC-001", "description": "Encrypt data"}
                        ]
                    }]
                }]
            }
        });
        let result = extract_controls(&json, &ArtifactType::ComponentDefinition);
        assert_eq!(result.len(), 1);
        assert!(result.contains_key("POL-ENC-001"));
        assert_eq!(result["POL-ENC-001"].description.as_deref(), Some("Encrypt data"));
    }

    #[test]
    fn extract_component_def_components_and_capabilities_combined() {
        let json = serde_json::json!({
            "component-definition": {
                "uuid": "cd-uuid",
                "metadata": {"title": "Test", "last-modified": "2026-01-01T00:00:00Z",
                             "version": "1.0", "oscal-version": "1.2.0"},
                "components": [{
                    "uuid": "comp-uuid", "type": "policy", "title": "Test",
                    "description": "Test",
                    "control-implementations": [{
                        "uuid": "ci-1", "source": "./b.json", "description": "CI",
                        "implemented-requirements": [
                            {"uuid": "u1", "control-id": "POL-AC-001", "description": "From component"}
                        ]
                    }]
                }],
                "capabilities": [{
                    "uuid": "cap-uuid",
                    "name": "Cap",
                    "description": "Capability",
                    "control-implementations": [{
                        "uuid": "ci-2", "source": "./b.json", "description": "CI",
                        "implemented-requirements": [
                            {"uuid": "u2", "control-id": "POL-ENC-001", "description": "From capability"}
                        ]
                    }]
                }]
            }
        });
        let result = extract_controls(&json, &ArtifactType::ComponentDefinition);
        assert_eq!(result.len(), 2);
        assert!(result.contains_key("POL-AC-001"));
        assert!(result.contains_key("POL-ENC-001"));
    }

    #[test]
    fn extract_component_def_empty_components() {
        let json = serde_json::json!({
            "component-definition": {
                "uuid": "cd-uuid",
                "metadata": {"title": "Test", "last-modified": "2026-01-01T00:00:00Z",
                             "version": "1.0", "oscal-version": "1.2.0"},
                "components": []
            }
        });
        let result = extract_controls(&json, &ArtifactType::ComponentDefinition);
        assert!(result.is_empty());
    }

    #[test]
    fn extract_catalog_deeply_nested() {
        let json = serde_json::json!({
            "catalog": {
                "uuid": "test-uuid",
                "metadata": {"title": "Test", "last-modified": "2026-01-01T00:00:00Z",
                             "version": "1.0", "oscal-version": "1.2.0"},
                "groups": [{
                    "id": "l1", "title": "L1",
                    "groups": [{
                        "id": "l2", "title": "L2",
                        "groups": [{
                            "id": "l3", "title": "L3",
                            "controls": [{"id": "DEEP-001", "title": "Deep", "parts": []}]
                        }]
                    }]
                }]
            }
        });
        let result = extract_controls(&json, &ArtifactType::Catalog);
        assert_eq!(result.len(), 1);
        assert!(result.contains_key("DEEP-001"));
    }

    #[test]
    fn extract_catalog_root_level_controls() {
        let json = serde_json::json!({
            "catalog": {
                "controls": [{
                    "id": "ROOT-001",
                    "uuid": "root-uuid",
                    "title": "Root control",
                    "parts": []
                }]
            }
        });

        let result = extract_controls(&json, &ArtifactType::Catalog);
        assert!(result.contains_key("ROOT-001"));
    }

    #[test]
    fn extract_component_def_aggregates_duplicate_control_descriptions() {
        let json = serde_json::json!({
            "component-definition": {
                "components": [
                    {
                        "control-implementations": [{
                            "implemented-requirements": [{
                                "control-id": "AC-1",
                                "uuid": "first",
                                "description": "First implementation"
                            }]
                        }]
                    },
                    {
                        "control-implementations": [{
                            "implemented-requirements": [{
                                "control-id": "AC-1",
                                "uuid": "second",
                                "description": "Second implementation"
                            }]
                        }]
                    }
                ]
            }
        });

        let result = extract_controls(&json, &ArtifactType::ComponentDefinition);
        assert_eq!(
            result["AC-1"].description.as_deref(),
            Some("First implementation\nSecond implementation")
        );
    }
}
