// Contract: XML Deserialization Functions
// Location: src/export/xml_deserializer.rs (new file)
// PRD Traceability: Prerequisite for M-1, M-3 (JSON-XML-JSON round-trip)
// AR Reference: Gap identified — WI-26 delivered serialization only

use crate::error::ForgeError;
use crate::oscal::catalog::CatalogEnvelope;
use crate::oscal::component_definition::ComponentDefinitionEnvelope;

/// Deserialize an OSCAL Catalog from an XML string.
///
/// Uses quick-xml serde integration to deserialize XML into the typed
/// CatalogEnvelope struct. Handles OSCAL namespace and XML declaration.
///
/// # Errors
/// Returns `ForgeError::Serialization` if XML parsing or deserialization fails.
pub fn deserialize_catalog_from_xml(xml: &str) -> Result<CatalogEnvelope, ForgeError>;

/// Deserialize an OSCAL Component Definition from an XML string.
///
/// Uses quick-xml serde integration to deserialize XML into the typed
/// ComponentDefinitionEnvelope struct.
///
/// # Errors
/// Returns `ForgeError::Serialization` if XML parsing or deserialization fails.
pub fn deserialize_component_from_xml(xml: &str) -> Result<ComponentDefinitionEnvelope, ForgeError>;
