// Contract: Round-Trip Test Helpers
// Location: tests/round_trip_test.rs (round-trip helper functions used by integration tests)
// PRD Traceability: M-1 (JSON-XML-JSON), M-2 (JSON-YAML-JSON), S-1 (XML-YAML-XML)

use serde_json::Value;
use crate::error::ForgeError;

/// Round-trip helper: JSON → XML → JSON (via typed model).
///
/// 1. Deserialize JSON Value to CatalogEnvelope
/// 2. Serialize catalog to XML via serialize_catalog_to_xml (WI-26)
/// 3. Deserialize XML back to CatalogEnvelope via quick-xml serde
/// 4. Serialize back to serde_json::Value
///
/// PRD M-1, M-3
pub fn round_trip_catalog_json_xml_json(json_input: &Value) -> Result<Value, ForgeError>;

/// Round-trip helper: JSON → YAML → JSON (via typed model).
///
/// 1. Deserialize JSON Value to CatalogEnvelope
/// 2. Serialize to YAML via serialize_to_yaml (WI-27)
/// 3. Deserialize YAML back to CatalogEnvelope via deserialize_from_yaml
/// 4. Serialize back to serde_json::Value
///
/// PRD M-2, M-4
pub fn round_trip_catalog_json_yaml_json(json_input: &Value) -> Result<Value, ForgeError>;

/// Round-trip helper: JSON → XML → JSON for Component Definition.
///
/// Same as catalog variant but uses ComponentDefinitionEnvelope.
///
/// PRD M-3
pub fn round_trip_component_json_xml_json(json_input: &Value) -> Result<Value, ForgeError>;

/// Round-trip helper: JSON → YAML → JSON for Component Definition.
///
/// Same as catalog variant but uses ComponentDefinitionEnvelope.
///
/// PRD M-4
pub fn round_trip_component_json_yaml_json(json_input: &Value) -> Result<Value, ForgeError>;

/// Round-trip helper: XML → YAML → XML for Catalog (via internal model).
///
/// 1. Deserialize XML to CatalogEnvelope via quick-xml serde
/// 2. Serialize to YAML via serialize_to_yaml
/// 3. Deserialize YAML back to CatalogEnvelope via deserialize_from_yaml
/// 4. Serialize back to XML via serialize_catalog_to_xml
///
/// PRD S-1
pub fn round_trip_catalog_xml_yaml_xml(xml_input: &str) -> Result<String, ForgeError>;
