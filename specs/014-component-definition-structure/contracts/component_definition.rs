//! Type contracts for OSCAL Component Definition builder (WI-14).
//!
//! These types define the API surface for the Component Definition builder.
//! Implementation follows after test-first development.

use serde::Serialize;

use crate::error::ForgeError;
use crate::model::PolicyDocument;
use crate::oscal::back_matter::BackMatter;

// ─── Constants ──────────────────────────────────────────────────────────

/// Default title when PolicyDocument has no title (empty string).
pub const DEFAULT_COMPONENT_TITLE: &str = "Untitled Policy Document";

// ─── Structs ────────────────────────────────────────────────────────────

/// JSON envelope producing `{"component-definition": {...}}` at the top level.
#[derive(Debug, Clone, Serialize)]
pub struct ComponentDefinitionEnvelope {
    /// The OSCAL Component Definition.
    #[serde(rename = "component-definition")]
    pub component_definition: ComponentDefinition,
}

/// OSCAL Component Definition root structure.
#[derive(Debug, Clone, Serialize)]
pub struct ComponentDefinition {
    /// Document-level UUID (v4, unique per generation).
    pub uuid: String,

    /// OSCAL metadata block (reused from WI-11 pattern).
    pub metadata: ComponentDefinitionMetadata,

    /// Documentary components (exactly one for this WI).
    pub components: Vec<DocumentaryComponent>,

    /// Back matter containing reference resources (WI-12).
    #[serde(rename = "back-matter", skip_serializing_if = "Option::is_none")]
    pub back_matter: Option<BackMatter>,
}

/// OSCAL metadata for the Component Definition.
///
/// Fields mapped from the shared `assemble_metadata` return value.
/// Uses String types for JSON serialization consistency with the Catalog pattern.
#[derive(Debug, Clone, Serialize)]
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

/// A documentary component of type "policy" within the Component Definition.
#[derive(Debug, Clone, Serialize)]
pub struct DocumentaryComponent {
    /// Deterministic UUID v5 (from COMPONENT_NAMESPACE + title + version).
    pub uuid: String,

    /// Component type -- always "policy" for documentary components.
    #[serde(rename = "type")]
    pub component_type: String,

    /// Component title (from PolicyDocument title or default).
    pub title: String,

    /// Component description (template format).
    pub description: String,

    /// Control implementations placeholder (empty for WI-14; populated by WI-15).
    #[serde(rename = "control-implementations")]
    pub control_implementations: Vec<serde_json::Value>,
}

// ─── Builder Function ───────────────────────────────────────────────────

/// Build an OSCAL Component Definition from a PolicyDocument.
///
/// Produces a `ComponentDefinitionEnvelope` with:
/// - Document-level UUID (v4) and metadata (via WI-11 `assemble_metadata`)
/// - One documentary component (type: "policy") with UUID (v5), title, description
/// - Empty `control-implementations` placeholder (populated by WI-15)
/// - Optional back matter (via WI-12 `generate_back_matter`)
///
/// # Arguments
/// * `document` - The parsed PolicyDocument from the domain model
///
/// # Errors
/// Returns `ForgeError::ComponentDefinitionBuild` if back matter generation fails.
pub fn build_component_definition(
    document: &PolicyDocument,
) -> Result<ComponentDefinitionEnvelope, ForgeError>;

/// Generate a deterministic UUID v5 for the documentary component.
///
/// Uses `COMPONENT_NAMESPACE` and the PolicyDocument's title + version
/// as the hash input. Same title + version always produces the same UUID.
fn generate_component_uuid(title: &str, version: &str) -> uuid::Uuid;
