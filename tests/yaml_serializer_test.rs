//! Unit tests for YAML serialization module (WI-27, T003).

use serde::{Deserialize, Serialize};

use forge::export::{deserialize_from_yaml, serialize_to_yaml};

/// Simple test struct for serialization.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestModel {
    title: String,
    version: String,
    count: u32,
}

#[test]
fn serialize_simple_struct_to_yaml() {
    let model =
        TestModel { title: "Test Policy".to_string(), version: "1.0".to_string(), count: 42 };
    let yaml = serialize_to_yaml(&model).expect("serialization should succeed");
    assert!(yaml.contains("title:"), "YAML should contain 'title:' key");
    assert!(yaml.contains("Test Policy"), "YAML should contain the title value");
    assert!(yaml.contains("version:"), "YAML should contain 'version:' key");
    assert!(yaml.contains("count:"), "YAML should contain 'count:' key");
}

#[test]
fn deserialize_yaml_string_to_value() {
    let yaml = "title: Test Policy\nversion: '1.0'\ncount: 42\n";
    let value: serde_json::Value =
        deserialize_from_yaml(yaml).expect("deserialization should succeed");
    assert_eq!(value["title"], "Test Policy");
    assert_eq!(value["version"], "1.0");
    assert_eq!(value["count"], 42);
}

#[test]
fn round_trip_serialize_then_deserialize_produces_equivalent_value() {
    let model = serde_json::json!({
        "catalog": {
            "uuid": "550e8400-e29b-41d4-a716-446655440000",
            "metadata": {
                "title": "Round Trip Test",
                "version": "2.0"
            },
            "groups": [
                {"id": "g1", "title": "Group 1"}
            ]
        }
    });

    let yaml = serialize_to_yaml(&model).expect("serialization should succeed");
    let deserialized: serde_json::Value =
        deserialize_from_yaml(&yaml).expect("deserialization should succeed");

    assert_eq!(model, deserialized, "Round-trip should produce equivalent Value");
}

#[test]
fn deserialize_invalid_yaml_returns_serialization_error() {
    let invalid_yaml = "{{{{invalid yaml::::";
    let result: Result<serde_json::Value, _> = deserialize_from_yaml(invalid_yaml);
    let err = result.expect_err("invalid YAML should return error");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("Serialization error") || err_msg.contains("YAML deserialization failed"),
        "Error should be Serialization variant, got: {err_msg}"
    );
}

#[test]
fn serialize_json_value_to_yaml() {
    let value = serde_json::json!({
        "name": "test",
        "nested": {"key": "value"},
        "list": [1, 2, 3]
    });
    let yaml = serialize_to_yaml(&value).expect("serialization should succeed");
    assert!(yaml.contains("name:"), "YAML should contain 'name:' key");
    assert!(yaml.contains("nested:"), "YAML should contain 'nested:' key");
    assert!(!yaml.starts_with('{'), "YAML output should not start with JSON brace");
}
