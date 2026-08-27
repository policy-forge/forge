//! Semantic validation for OSCAL artifacts (WI-20).
//!
//! Detects logical inconsistencies beyond JSON Schema compliance:
//! - Orphaned back-matter links (PRD M-3)
//! - Missing required references (PRD M-4)

use std::collections::HashSet;

use serde_json::Value;

use super::OscalModelType;
use super::error_types::{ValidationError, ValidationErrorCategory};
use super::formatter::{pointer_to_json_path, truncate_value};

/// Semantic validator for OSCAL artifacts (PRD M-3, M-4).
pub struct SemanticValidator;

impl SemanticValidator {
    /// Run all semantic validation checks on an OSCAL artifact.
    ///
    /// Returns a list of semantic `ValidationError`s.
    /// Does NOT follow external URLs (SEC-5).
    #[must_use]
    pub fn validate(&self, json: &Value, model_type: OscalModelType) -> Vec<ValidationError> {
        let mut errors = check_orphaned_links(json, model_type);
        errors.extend(check_missing_references(json, model_type));
        errors
    }
}

/// Collect resource UUIDs from the model's own back-matter.
///
/// Reads only the root key matching `model_type` (F0784): a Profile (or any
/// other type) whose links target its own back-matter must not be judged
/// against an empty resource set harvested from unrelated root keys.
fn collect_resource_uuids(json: &Value, model_type: OscalModelType) -> HashSet<String> {
    let mut uuids = HashSet::new();

    if let Some(root) = json.get(model_type.as_str())
        && let Some(resources) = root.pointer("/back-matter/resources")
        && let Some(arr) = resources.as_array()
    {
        for resource in arr {
            if let Some(uuid) = resource.get("uuid").and_then(Value::as_str) {
                uuids.insert(uuid.to_string());
            }
        }
    }

    uuids
}

/// Check for orphaned back-matter links (PRD M-3).
///
/// Finds `href` values starting with `#` that reference UUIDs
/// not present in `back-matter.resources[].uuid`.
/// Does NOT follow external URLs (SEC-5).
fn check_orphaned_links(json: &Value, model_type: OscalModelType) -> Vec<ValidationError> {
    let resource_uuids = collect_resource_uuids(json, model_type);
    let mut errors = Vec::new();

    // Recursively walk JSON tree looking for href fields starting with "#"
    walk_for_orphaned_links(json, "$", &resource_uuids, &mut errors);

    errors
}

/// Maximum recursion depth for JSON tree walking (`DoS` protection).
const MAX_WALK_DEPTH: usize = 100;

/// Recursively walk the JSON tree tracking path, looking for orphaned `href` references.
fn walk_for_orphaned_links(
    value: &Value,
    current_path: &str,
    resource_uuids: &HashSet<String>,
    errors: &mut Vec<ValidationError>,
) {
    walk_for_orphaned_links_inner(value, current_path, resource_uuids, errors, 0);
}

fn walk_for_orphaned_links_inner(
    value: &Value,
    current_path: &str,
    resource_uuids: &HashSet<String>,
    errors: &mut Vec<ValidationError>,
    depth: usize,
) {
    if depth > MAX_WALK_DEPTH {
        tracing::trace!(path = %current_path, depth, max = MAX_WALK_DEPTH, "max walk depth exceeded; skipping further traversal");
        return;
    }
    match value {
        Value::Object(map) => {
            // Check if this object has an href that starts with "#"
            // SEC-5: do NOT follow external URLs (non-# hrefs)
            if let Some(href_value) = map.get("href")
                && let Some(href_str) = href_value.as_str()
                && let Some(uuid) = href_str.strip_prefix('#')
                && !resource_uuids.contains(uuid)
            {
                errors.push(ValidationError {
                    category: ValidationErrorCategory::Semantic,
                    path: format!("{current_path}.href"),
                    message: format!(
                        "orphaned link: reference #{uuid} not found in back-matter resources"
                    ),
                    expected: "referenced resource exists in back-matter".to_string(),
                    actual: truncate_value(&format!("#{uuid}"), 100),
                });
            }

            // Recurse into all child values
            for (key, child) in map {
                let child_path = format!("{current_path}.{key}");
                walk_for_orphaned_links_inner(
                    child,
                    &child_path,
                    resource_uuids,
                    errors,
                    depth + 1,
                );
            }
        }
        Value::Array(arr) => {
            for (i, child) in arr.iter().enumerate() {
                let child_path = format!("{current_path}[{i}]");
                walk_for_orphaned_links_inner(
                    child,
                    &child_path,
                    resource_uuids,
                    errors,
                    depth + 1,
                );
            }
        }
        _ => {}
    }
}

/// Check for missing required references (PRD M-4).
///
/// For Component Definitions: walk `implemented-requirements` and check
/// `control-id` fields are non-empty strings with at least one alphanumeric character.
/// For Catalog: no-op (return empty vec).
fn check_missing_references(json: &Value, model_type: OscalModelType) -> Vec<ValidationError> {
    match model_type {
        OscalModelType::ComponentDefinition => check_component_control_ids(json),
        OscalModelType::Catalog | OscalModelType::Profile | OscalModelType::Mapping => vec![],
    }
}

/// Validate control-id fields in a component definition.
fn check_component_control_ids(json: &Value) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    let Some(root) = json.get("component-definition") else {
        return errors;
    };

    let Some(components) = root.get("components").and_then(Value::as_array) else {
        return errors;
    };

    for (comp_idx, component) in components.iter().enumerate() {
        let Some(control_impls) =
            component.get("control-implementations").and_then(Value::as_array)
        else {
            continue;
        };

        for (ci_idx, ctrl_impl) in control_impls.iter().enumerate() {
            let Some(impl_reqs) =
                ctrl_impl.get("implemented-requirements").and_then(Value::as_array)
            else {
                continue;
            };

            for (req_idx, req) in impl_reqs.iter().enumerate() {
                let base_path = pointer_to_json_path(&format!(
                    "/component-definition/components/{comp_idx}/control-implementations/{ci_idx}/implemented-requirements/{req_idx}/control-id"
                ));

                match req.get("control-id") {
                    Some(Value::String(control_id)) => {
                        // Valid control-ids must have at least one alphanumeric character
                        if !control_id.chars().any(char::is_alphanumeric) {
                            errors.push(ValidationError {
                                category: ValidationErrorCategory::Semantic,
                                path: base_path,
                                message: format!(
                                    "invalid control-id: \"{control_id}\" contains no alphanumeric characters"
                                ),
                                expected: "non-empty string with at least one alphanumeric character".to_string(),
                                actual: truncate_value(&format!("\"{control_id}\""), 100),
                            });
                        }
                    }
                    Some(_) => {
                        errors.push(ValidationError {
                            category: ValidationErrorCategory::Semantic,
                            path: base_path,
                            message: "control-id must be a string".to_string(),
                            expected: "non-empty string with at least one alphanumeric character"
                                .to_string(),
                            actual: "non-string value".to_string(),
                        });
                    }
                    None => {
                        errors.push(ValidationError {
                            category: ValidationErrorCategory::Semantic,
                            path: base_path,
                            message: "missing control-id in implemented-requirement".to_string(),
                            expected: "non-empty string with at least one alphanumeric character"
                                .to_string(),
                            actual: "field not present".to_string(),
                        });
                    }
                }
            }
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- T019: check_orphaned_links tests ---

    #[test]
    fn orphaned_link_detected() {
        let json: Value = serde_json::from_str(
            r##"{
                "catalog": {
                    "metadata": { "title": "Test" },
                    "groups": [{
                        "links": [{ "href": "#orphaned-uuid" }]
                    }],
                    "back-matter": {
                        "resources": [
                            { "uuid": "existing-uuid", "title": "Resource" }
                        ]
                    }
                }
            }"##,
        )
        .unwrap();

        let errors = check_orphaned_links(&json, OscalModelType::Catalog);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].category, ValidationErrorCategory::Semantic);
        assert!(errors[0].message.contains("orphaned-uuid"));
        assert!(errors[0].path.contains("href"));
    }

    #[test]
    fn valid_links_no_errors() {
        let json: Value = serde_json::from_str(
            r##"{
                "catalog": {
                    "groups": [{
                        "links": [{ "href": "#valid-uuid" }]
                    }],
                    "back-matter": {
                        "resources": [
                            { "uuid": "valid-uuid", "title": "Resource" }
                        ]
                    }
                }
            }"##,
        )
        .unwrap();

        let errors = check_orphaned_links(&json, OscalModelType::Catalog);
        assert!(errors.is_empty());
    }

    #[test]
    fn no_back_matter_links_with_hash_all_orphaned() {
        // PRD EC-3: no back-matter section but links with #uuid hrefs → all reported
        let json: Value = serde_json::from_str(
            r##"{
                "catalog": {
                    "groups": [{
                        "links": [
                            { "href": "#uuid-1" },
                            { "href": "#uuid-2" }
                        ]
                    }]
                }
            }"##,
        )
        .unwrap();

        let errors = check_orphaned_links(&json, OscalModelType::Catalog);
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn multiple_orphaned_links_all_reported() {
        let json: Value = serde_json::from_str(
            r##"{
                "catalog": {
                    "groups": [
                        { "links": [{ "href": "#orphan-1" }] },
                        { "links": [{ "href": "#orphan-2" }, { "href": "#orphan-3" }] }
                    ],
                    "back-matter": { "resources": [] }
                }
            }"##,
        )
        .unwrap();

        let errors = check_orphaned_links(&json, OscalModelType::Catalog);
        assert_eq!(errors.len(), 3);
    }

    #[test]
    fn mapping_back_matter_resource_satisfies_local_link() {
        let resource_uuid = "11111111-1111-4111-8111-111111111111";
        let json = serde_json::json!({
            "mapping-collection": {
                "links": [{"href": format!("#{resource_uuid}")}],
                "back-matter": {
                    "resources": [{"uuid": resource_uuid}]
                }
            }
        });

        assert!(check_orphaned_links(&json, OscalModelType::Mapping).is_empty());
    }

    // ── F0784: profile back-matter links must resolve against the profile root ──

    #[test]
    fn profile_back_matter_resource_satisfies_local_link() {
        let resource_uuid = "22222222-2222-4222-8222-222222222222";
        let json = serde_json::json!({
            "profile": {
                "links": [{"href": format!("#{resource_uuid}")}],
                "back-matter": {
                    "resources": [{"uuid": resource_uuid}]
                }
            }
        });

        assert!(
            check_orphaned_links(&json, OscalModelType::Profile).is_empty(),
            "profile-internal links must not be flagged orphaned (F0784)"
        );
    }

    #[test]
    fn no_links_no_errors() {
        // PRD EC-4: artifact with no links → no errors
        let json: Value = serde_json::from_str(
            r#"{
                "catalog": {
                    "metadata": { "title": "Test" },
                    "groups": [{ "title": "Group 1" }]
                }
            }"#,
        )
        .unwrap();

        let errors = check_orphaned_links(&json, OscalModelType::Catalog);
        assert!(errors.is_empty());
    }

    #[test]
    fn external_urls_not_followed() {
        // SEC-5: do not follow external URLs
        let json: Value = serde_json::from_str(
            r#"{
                "catalog": {
                    "groups": [{
                        "links": [
                            { "href": "https://example.com/resource" },
                            { "href": "http://internal/doc" }
                        ]
                    }]
                }
            }"#,
        )
        .unwrap();

        // External URLs should not produce errors
        let errors = check_orphaned_links(&json, OscalModelType::Catalog);
        assert!(errors.is_empty());
    }

    #[test]
    fn max_walk_depth_prevents_stack_overflow() {
        // Build a JSON structure deeper than MAX_WALK_DEPTH with an orphaned link at the bottom
        let mut json = serde_json::json!({"href": "#deep-orphan"});
        for _ in 0..=MAX_WALK_DEPTH + 10 {
            json = serde_json::json!({"child": json});
        }
        let wrapper =
            serde_json::json!({"catalog": {"back-matter": {"resources": []}, "data": json}});

        let errors = check_orphaned_links(&wrapper, OscalModelType::Catalog);
        // The deeply nested orphaned link should NOT be found (depth guard stops traversal)
        assert!(
            errors.is_empty(),
            "Should not find orphaned link beyond MAX_WALK_DEPTH, found {} errors",
            errors.len()
        );
    }

    // --- T020: check_missing_references tests ---

    #[test]
    fn empty_control_id_detected() {
        let json: Value = serde_json::from_str(
            r#"{
                "component-definition": {
                    "components": [{
                        "type": "software",
                        "title": "Test",
                        "control-implementations": [{
                            "uuid": "ci-1",
                            "source": "profile.json",
                            "implemented-requirements": [{
                                "uuid": "req-1",
                                "control-id": ""
                            }]
                        }]
                    }]
                }
            }"#,
        )
        .unwrap();

        let errors = check_missing_references(&json, OscalModelType::ComponentDefinition);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].category, ValidationErrorCategory::Semantic);
        assert!(errors[0].message.contains("control-id"));
    }

    #[test]
    fn valid_control_ids_no_errors() {
        let json: Value = serde_json::from_str(
            r#"{
                "component-definition": {
                    "components": [{
                        "type": "software",
                        "title": "Test",
                        "control-implementations": [{
                            "uuid": "ci-1",
                            "source": "profile.json",
                            "implemented-requirements": [{
                                "uuid": "req-1",
                                "control-id": "ac-1"
                            }]
                        }]
                    }]
                }
            }"#,
        )
        .unwrap();

        let errors = check_missing_references(&json, OscalModelType::ComponentDefinition);
        assert!(errors.is_empty());
    }

    #[test]
    fn no_implemented_requirements_no_errors() {
        let json: Value = serde_json::from_str(
            r#"{
                "component-definition": {
                    "components": [{
                        "type": "software",
                        "title": "Test"
                    }]
                }
            }"#,
        )
        .unwrap();

        let errors = check_missing_references(&json, OscalModelType::ComponentDefinition);
        assert!(errors.is_empty());
    }

    #[test]
    fn catalog_skips_check_gracefully() {
        let json: Value =
            serde_json::from_str(r#"{ "catalog": { "metadata": { "title": "Test" } } }"#).unwrap();

        let errors = check_missing_references(&json, OscalModelType::Catalog);
        assert!(errors.is_empty());
    }

    #[test]
    fn whitespace_only_control_id_detected() {
        let json: Value = serde_json::from_str(
            r#"{
                "component-definition": {
                    "components": [{
                        "type": "software",
                        "title": "Test",
                        "control-implementations": [{
                            "uuid": "ci-1",
                            "source": "profile.json",
                            "implemented-requirements": [{
                                "uuid": "req-1",
                                "control-id": "   "
                            }]
                        }]
                    }]
                }
            }"#,
        )
        .unwrap();

        let errors = check_missing_references(&json, OscalModelType::ComponentDefinition);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn punctuation_only_control_id_detected() {
        let json: Value = serde_json::from_str(
            r#"{
                "component-definition": {
                    "components": [{
                        "type": "software",
                        "title": "Test",
                        "control-implementations": [{
                            "uuid": "ci-1",
                            "source": "profile.json",
                            "implemented-requirements": [{
                                "uuid": "req-1",
                                "control-id": "---"
                            }]
                        }]
                    }]
                }
            }"#,
        )
        .unwrap();

        let errors = check_missing_references(&json, OscalModelType::ComponentDefinition);
        assert_eq!(errors.len(), 1);
    }

    // --- SemanticValidator::validate tests ---

    #[test]
    fn validator_combines_both_checks() {
        let validator = SemanticValidator;
        let json: Value = serde_json::from_str(
            r##"{
                "component-definition": {
                    "components": [{
                        "type": "software",
                        "title": "Test",
                        "links": [{ "href": "#orphan" }],
                        "control-implementations": [{
                            "uuid": "ci-1",
                            "source": "profile.json",
                            "implemented-requirements": [{
                                "uuid": "req-1",
                                "control-id": ""
                            }]
                        }]
                    }]
                }
            }"##,
        )
        .unwrap();

        let errors = validator.validate(&json, OscalModelType::ComponentDefinition);
        // Should have at least 1 orphaned link + 1 missing reference
        assert!(errors.len() >= 2);
        assert!(errors.iter().any(|e| e.message.contains("orphaned")));
        assert!(errors.iter().any(|e| e.message.contains("control-id")));
    }
}
