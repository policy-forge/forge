//! OSCAL XML deserialization: converts OSCAL XML strings to model structs.
//!
//! Uses `quick_xml::de::from_str()` with dedicated XML deserialization structs
//! that handle XML attribute/element naming differences from JSON.
//!
//! XXE prevention: quick-xml's default parser does NOT process DTDs or expand entities (SEC-1).

use serde::Deserialize;
use uuid::Uuid;

use crate::error::ForgeError;
use crate::oscal::back_matter::{
    BackMatter, BackMatterResource, OscalLink, Prop, ResourceCitation, Rlink,
};
use crate::oscal::catalog::{
    CatalogEnvelope, OscalCatalog, OscalControl, OscalGroup, OscalMetadata, OscalParam,
    OscalParamConstraint,
};
use crate::oscal::component_definition::{
    ComponentDefinition, ComponentDefinitionEnvelope, ComponentDefinitionMetadata,
    DocumentaryComponent,
};
use crate::oscal::implemented_requirements::{ControlImplementation, ImplementedRequirement};
use crate::oscal::parts::{OscalPart, OscalProp};

// ─── XML Deserialization Structs ─────────────────────────────────────────
//
// These structs mirror the OSCAL model types but use quick-xml serde
// conventions: `@attr` for XML attributes, singular element names for
// repeated children, and `<p>` wrapper handling.

#[derive(Deserialize)]
struct XmlCatalog {
    #[serde(rename = "@uuid")]
    uuid: String,
    metadata: XmlMetadata,
    #[serde(default, rename = "group")]
    groups: Vec<XmlGroup>,
    #[serde(default, rename = "back-matter")]
    back_matter: Option<XmlBackMatter>,
}

#[derive(Deserialize)]
struct XmlComponentDefinition {
    #[serde(rename = "@uuid")]
    uuid: String,
    metadata: XmlMetadata,
    #[serde(default, rename = "component")]
    components: Vec<XmlComponent>,
    #[serde(default, rename = "back-matter")]
    back_matter: Option<XmlBackMatter>,
}

#[derive(Deserialize)]
struct XmlMetadata {
    title: String,
    #[serde(rename = "last-modified")]
    last_modified: String,
    version: String,
    #[serde(rename = "oscal-version")]
    oscal_version: String,
}

#[derive(Deserialize)]
struct XmlGroup {
    #[serde(rename = "@id")]
    id: String,
    title: String,
    #[serde(default, rename = "prop")]
    props: Vec<XmlProp>,
    #[serde(default, rename = "link")]
    links: Vec<XmlLink>,
    #[serde(default, rename = "control")]
    controls: Vec<XmlControl>,
}

#[derive(Deserialize)]
struct XmlControl {
    #[serde(rename = "@id")]
    id: String,
    title: String,
    #[serde(default, rename = "param")]
    params: Vec<XmlParam>,
    #[serde(default, rename = "prop")]
    props: Vec<XmlProp>,
    #[serde(default, rename = "link")]
    links: Vec<XmlLink>,
    #[serde(default, rename = "part")]
    parts: Vec<XmlPart>,
}

/// OSCAL `<param>` element within a control.
///
/// XSD order: `<label>`, `<constraint>*`, `<value>*`.
#[derive(Deserialize)]
struct XmlParam {
    #[serde(rename = "@id")]
    id: String,
    #[serde(default)]
    label: String,
    #[serde(default, rename = "constraint")]
    constraints: Vec<XmlParamConstraint>,
    #[serde(default, rename = "value")]
    values: Vec<String>,
}

/// OSCAL `<constraint>` element within a `<param>`.
///
/// Contains `<description><p>text</p></description>` (markup-multiline).
#[derive(Deserialize)]
struct XmlParamConstraint {
    #[serde(default)]
    description: Option<XmlDescription>,
}

#[derive(Deserialize)]
struct XmlPart {
    #[serde(default, rename = "@id")]
    id: Option<String>,
    #[serde(rename = "@name")]
    name: String,
    #[serde(default, rename = "prop")]
    props: Vec<XmlProp>,
    /// Prose content wrapped in `<p>` elements in OSCAL XML.
    /// Multiple `<p>` nodes are preserved and joined with newlines.
    #[serde(default, rename = "p")]
    paragraphs: Vec<String>,
    #[serde(default, rename = "part")]
    parts: Vec<XmlPart>,
}

#[derive(Deserialize)]
struct XmlProp {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@value")]
    value: String,
    #[serde(default, rename = "@ns")]
    ns: Option<String>,
}

#[derive(Deserialize)]
struct XmlLink {
    #[serde(rename = "@href")]
    href: String,
    #[serde(default, rename = "@rel")]
    rel: Option<String>,
    #[serde(default)]
    text: Option<String>,
}

#[derive(Deserialize)]
struct XmlComponent {
    #[serde(rename = "@uuid")]
    uuid: String,
    #[serde(rename = "@type")]
    component_type: String,
    title: String,
    #[serde(default)]
    description: Option<XmlDescription>,
    #[serde(default, rename = "prop")]
    props: Vec<XmlProp>,
    #[serde(default, rename = "control-implementation")]
    control_implementations: Vec<XmlControlImplementation>,
}

#[derive(Deserialize)]
struct XmlControlImplementation {
    #[serde(rename = "@uuid")]
    uuid: String,
    #[serde(rename = "@source")]
    source: String,
    #[serde(default)]
    description: Option<XmlDescription>,
    #[serde(default, rename = "implemented-requirement")]
    implemented_requirements: Vec<XmlImplementedRequirement>,
}

#[derive(Deserialize)]
struct XmlImplementedRequirement {
    #[serde(rename = "@uuid")]
    uuid: String,
    #[serde(rename = "@control-id")]
    control_id: String,
    #[serde(default)]
    description: Option<XmlDescription>,
    #[serde(default, rename = "prop")]
    props: Vec<XmlProp>,
    #[serde(default, rename = "link")]
    links: Vec<XmlLink>,
}

/// Handles `<description><p>text</p></description>` markup-multiline format.
/// Multiple `<p>` elements are preserved and joined with newlines.
#[derive(Deserialize)]
struct XmlDescription {
    #[serde(default, rename = "p")]
    paragraphs: Vec<String>,
}

#[derive(Deserialize)]
struct XmlBackMatter {
    #[serde(default, rename = "resource")]
    resources: Vec<XmlResource>,
}

#[derive(Deserialize)]
struct XmlResource {
    #[serde(rename = "@uuid")]
    uuid: String,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    description: Option<XmlDescription>,
    #[serde(default, rename = "prop")]
    props: Vec<XmlBackMatterProp>,
    #[serde(default)]
    citation: Option<XmlCitation>,
    #[serde(default, rename = "rlink")]
    rlinks: Vec<XmlRlink>,
}

#[derive(Deserialize)]
struct XmlBackMatterProp {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@value")]
    value: String,
}

#[derive(Deserialize)]
struct XmlCitation {
    #[serde(default)]
    text: Option<String>,
}

#[derive(Deserialize)]
struct XmlRlink {
    #[serde(rename = "@href")]
    href: String,
    #[serde(default, rename = "@media-type")]
    media_type: Option<String>,
}

// ─── Conversion Functions ────────────────────────────────────────────────

fn convert_metadata(xml: XmlMetadata) -> OscalMetadata {
    OscalMetadata {
        title: xml.title,
        last_modified: xml.last_modified,
        version: xml.version,
        oscal_version: xml.oscal_version,
    }
}

fn convert_prop(xml: XmlProp) -> OscalProp {
    OscalProp { name: xml.name, value: xml.value, ns: xml.ns }
}

fn convert_link(xml: XmlLink) -> crate::oscal::back_matter::OscalLink {
    crate::oscal::back_matter::OscalLink {
        href: xml.href,
        rel: xml.rel.unwrap_or_default(),
        text: xml.text,
    }
}

fn convert_part(xml: XmlPart) -> OscalPart {
    let prose = if xml.paragraphs.is_empty() { String::new() } else { xml.paragraphs.join("\n") };
    OscalPart {
        id: xml.id.unwrap_or_default(),
        name: xml.name,
        prose,
        props: xml.props.into_iter().map(convert_prop).collect(),
        parts: xml.parts.into_iter().map(convert_part).collect(),
    }
}

fn convert_param_constraint(xml: XmlParamConstraint) -> OscalParamConstraint {
    let description = xml.description.map(|d| d.paragraphs.join("\n")).unwrap_or_default();
    OscalParamConstraint { description }
}

fn convert_param(xml: XmlParam) -> OscalParam {
    OscalParam {
        id: xml.id,
        label: xml.label,
        values: xml.values,
        constraints: xml.constraints.into_iter().map(convert_param_constraint).collect(),
    }
}

fn convert_control(xml: XmlControl) -> OscalControl {
    OscalControl {
        id: xml.id,
        // OSCAL XML controls don't carry a uuid attribute; OscalControl.uuid is
        // #[serde(skip_serializing, default)] so this empty value never appears in output.
        uuid: String::new(),
        title: xml.title,
        props: xml.props.into_iter().map(convert_prop).collect(),
        links: xml.links.into_iter().map(convert_link).collect(),
        params: xml.params.into_iter().map(convert_param).collect(),
        parts: xml.parts.into_iter().map(convert_part).collect(),
    }
}

fn convert_group(xml: XmlGroup) -> OscalGroup {
    OscalGroup {
        id: xml.id,
        title: xml.title,
        props: xml.props.into_iter().map(convert_prop).collect(),
        links: xml.links.into_iter().map(convert_link).collect(),
        controls: xml.controls.into_iter().map(convert_control).collect(),
        groups: vec![],
    }
}

fn convert_back_matter(xml: XmlBackMatter) -> BackMatter {
    BackMatter { resources: xml.resources.into_iter().map(convert_resource).collect() }
}

fn convert_resource(xml: XmlResource) -> BackMatterResource {
    let uuid = Uuid::try_parse(&xml.uuid).unwrap_or_else(|_| Uuid::new_v4());
    let description = xml.description.map(|d| d.paragraphs.join("\n"));
    BackMatterResource {
        uuid,
        title: xml.title.unwrap_or_default(),
        description,
        citation: xml.citation.and_then(|c| c.text.map(|t| ResourceCitation { text: t })),
        rlinks: xml
            .rlinks
            .into_iter()
            .map(|r| Rlink { href: r.href, media_type: r.media_type })
            .collect(),
        props: xml.props.into_iter().map(|p| Prop { name: p.name, value: p.value, ns: None }).collect(),
    }
}

fn convert_catalog(xml: XmlCatalog) -> OscalCatalog {
    OscalCatalog {
        uuid: xml.uuid,
        metadata: convert_metadata(xml.metadata),
        controls: vec![],
        groups: xml.groups.into_iter().map(convert_group).collect(),
        back_matter: xml.back_matter.map(convert_back_matter),
    }
}

fn convert_component(xml: XmlComponent) -> Result<DocumentaryComponent, ForgeError> {
    let description = xml.description.map(|d| d.paragraphs.join("\n")).unwrap_or_default();
    let control_implementations = xml
        .control_implementations
        .into_iter()
        .map(convert_control_implementation)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(DocumentaryComponent {
        uuid: xml.uuid,
        component_type: xml.component_type,
        title: xml.title,
        description,
        props: xml.props.into_iter().map(convert_prop).collect(),
        control_implementations,
    })
}

fn convert_control_implementation(
    xml: XmlControlImplementation,
) -> Result<ControlImplementation, ForgeError> {
    Uuid::try_parse(&xml.uuid).map_err(|e| ForgeError::ExportInvalidOscal {
        detail: format!("Invalid UUID in control-implementation: '{uuid}' — {e}", uuid = xml.uuid),
    })?;
    let description = xml.description.map(|d| d.paragraphs.join("\n")).unwrap_or_default();
    let implemented_requirements = xml
        .implemented_requirements
        .into_iter()
        .map(convert_implemented_requirement)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ControlImplementation {
        uuid: xml.uuid,
        source: xml.source,
        description,
        implemented_requirements,
    })
}

fn convert_implemented_requirement(
    xml: XmlImplementedRequirement,
) -> Result<ImplementedRequirement, ForgeError> {
    Uuid::try_parse(&xml.uuid).map_err(|e| ForgeError::ExportInvalidOscal {
        detail: format!(
            "Invalid UUID in implemented-requirement: '{uuid}' — {e}",
            uuid = xml.uuid
        ),
    })?;
    let description = xml.description.map(|d| d.paragraphs.join("\n")).unwrap_or_default();
    Ok(ImplementedRequirement {
        uuid: xml.uuid,
        control_id: xml.control_id,
        description,
        props: xml.props.into_iter().map(convert_prop).collect(),
        links: xml.links.into_iter().map(convert_link).collect(),
    })
}

fn convert_component_definition(
    xml: XmlComponentDefinition,
) -> Result<ComponentDefinition, ForgeError> {
    let components = xml
        .components
        .into_iter()
        .map(convert_component)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ComponentDefinition {
        uuid: xml.uuid,
        metadata: ComponentDefinitionMetadata {
            title: xml.metadata.title,
            last_modified: xml.metadata.last_modified,
            version: xml.metadata.version,
            oscal_version: xml.metadata.oscal_version,
        },
        components,
        capabilities: vec![],
        back_matter: xml.back_matter.map(convert_back_matter),
    })
}

// ─── Public API ──────────────────────────────────────────────────────────

/// Deserialize an OSCAL Catalog from an XML string.
///
/// Parses the XML `<catalog>` root element using dedicated XML deserialization
/// structs, then converts to the shared `CatalogEnvelope` model.
///
/// # Errors
/// Returns `ForgeError::Serialization` if XML parsing fails.
pub fn deserialize_catalog_from_xml(xml: &str) -> Result<CatalogEnvelope, ForgeError> {
    let xml_catalog: XmlCatalog = quick_xml::de::from_str(xml).map_err(|e| {
        ForgeError::Serialization(format!("XML catalog deserialization failed: {e}"))
    })?;
    Ok(CatalogEnvelope { catalog: convert_catalog(xml_catalog) })
}

/// Deserialize an OSCAL Component Definition from an XML string.
///
/// Parses the XML `<component-definition>` root element using dedicated XML
/// deserialization structs, then converts to the shared `ComponentDefinitionEnvelope` model.
///
/// # Errors
/// Returns `ForgeError::Serialization` if XML parsing fails.
pub fn deserialize_component_from_xml(
    xml: &str,
) -> Result<ComponentDefinitionEnvelope, ForgeError> {
    let xml_cd: XmlComponentDefinition = quick_xml::de::from_str(xml).map_err(|e| {
        ForgeError::Serialization(format!("XML component-definition deserialization failed: {e}"))
    })?;
    let component_definition = convert_component_definition(xml_cd)?;
    Ok(ComponentDefinitionEnvelope { component_definition })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ══════════════════════════════════════════════════════
    // T022: XML deserialization unit tests
    // ══════════════════════════════════════════════════════

    #[test]
    fn deserialize_catalog_xml_fixture() {
        let xml = include_str!("../../tests/fixtures/export/catalog.xml");
        let result = deserialize_catalog_from_xml(xml);
        assert!(result.is_ok(), "Catalog XML deser failed: {:?}", result.unwrap_err());
        let envelope = result.unwrap();
        assert_eq!(envelope.catalog.uuid, "a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d");
        assert_eq!(envelope.catalog.metadata.title, "Test Catalog");
        assert_eq!(envelope.catalog.metadata.oscal_version, "1.2.0");
        assert_eq!(envelope.catalog.groups.len(), 1);
        assert_eq!(envelope.catalog.groups[0].id, "access-control");
        assert_eq!(envelope.catalog.groups[0].controls.len(), 1);
        assert_eq!(envelope.catalog.groups[0].controls[0].id, "POL-AC-001");
    }

    #[test]
    fn deserialize_component_xml_fixture() {
        let xml = include_str!("../../tests/fixtures/export/component.xml");
        let result = deserialize_component_from_xml(xml);
        assert!(result.is_ok(), "Component XML deser failed: {:?}", result.unwrap_err());
        let envelope = result.unwrap();
        assert_eq!(envelope.component_definition.uuid, "b2c3d4e5-f6a7-4b8c-9d0e-1f2a3b4c5d6e");
        assert_eq!(envelope.component_definition.metadata.title, "Test Component Definition");
        assert_eq!(envelope.component_definition.components.len(), 1);
        assert_eq!(envelope.component_definition.components[0].component_type, "policy");
        assert_eq!(envelope.component_definition.components[0].title, "Test Policy");
    }

    #[test]
    fn deserialize_catalog_xml_preserves_prose() {
        let xml = include_str!("../../tests/fixtures/export/catalog.xml");
        let envelope = deserialize_catalog_from_xml(xml).unwrap();
        let control = &envelope.catalog.groups[0].controls[0];
        assert_eq!(control.parts.len(), 1);
        assert_eq!(control.parts[0].name, "statement");
        assert_eq!(control.parts[0].prose, "All users must authenticate using MFA.");
    }

    #[test]
    fn deserialize_component_xml_preserves_description() {
        let xml = include_str!("../../tests/fixtures/export/component.xml");
        let envelope = deserialize_component_from_xml(xml).unwrap();
        assert_eq!(
            envelope.component_definition.components[0].description,
            "A test policy document."
        );
    }

    // ══════════════════════════════════════════════════════
    // T029: XXE prevention test (SEC-1)
    // ══════════════════════════════════════════════════════

    #[test]
    fn xxe_prevention_no_entity_expansion() {
        let malicious_xml = r#"<?xml version="1.0"?>
<!DOCTYPE foo [
  <!ENTITY xxe "INJECTED">
]>
<catalog xmlns="http://csrc.nist.gov/ns/oscal/1.0" uuid="test">
  <metadata>
    <title>&xxe;</title>
    <last-modified>2026-01-01T00:00:00Z</last-modified>
    <version>1.0</version>
    <oscal-version>1.2.0</oscal-version>
  </metadata>
</catalog>"#;

        // quick-xml should either error or not expand the entity
        let result = deserialize_catalog_from_xml(malicious_xml);
        match result {
            Ok(envelope) => {
                // If parsing succeeds, entity must NOT have been expanded
                assert_ne!(
                    envelope.catalog.metadata.title, "INJECTED",
                    "XXE entity expansion detected — security vulnerability!"
                );
            }
            Err(_) => {
                // Rejecting the document is also acceptable (safe behavior)
            }
        }
    }

    // ══════════════════════════════════════════════════════
    // T068: XML round-trip for control-implementations
    // ══════════════════════════════════════════════════════

    #[test]
    fn xml_round_trip_control_implementations() {
        use crate::export::xml_serializer::serialize_component_definition_to_xml;
        use crate::oscal::component_definition::ComponentDefinitionMetadata;

        let original = ComponentDefinitionEnvelope {
            component_definition: ComponentDefinition {
                uuid: "660e8400-e29b-41d4-a716-446655440000".to_string(),
                metadata: ComponentDefinitionMetadata {
                    title: "Round-Trip Test".to_string(),
                    last_modified: "2026-03-12T00:00:00Z".to_string(),
                    version: "1.0".to_string(),
                    oscal_version: "1.2.0".to_string(),
                },
                components: vec![DocumentaryComponent {
                    uuid: "770e8400-e29b-41d4-a716-446655440000".to_string(),
                    component_type: "policy".to_string(),
                    title: "Test Policy".to_string(),
                    description: "A test policy.".to_string(),
                    props: vec![],
                    control_implementations: vec![ControlImplementation {
                        uuid: "880e8400-e29b-41d4-a716-446655440000".to_string(),
                        source: "./baseline.json".to_string(),
                        description: "Implementation narratives.".to_string(),
                        implemented_requirements: vec![
                            ImplementedRequirement {
                                uuid: "990e8400-e29b-41d4-a716-446655440000".to_string(),
                                control_id: "POL-AC-001".to_string(),
                                description: "All users must authenticate.".to_string(),
                                props: vec![OscalProp {
                                    name: "source-file".to_string(),
                                    value: "policy.md".to_string(),
                                    ns: Some(
                                        "https://forge.policy-forge.github.io/ns/trace"
                                            .to_string(),
                                    ),
                                }],
                                links: vec![OscalLink {
                                    href: "policy.md#line=10".to_string(),
                                    rel: "source".to_string(),
                                    text: None,
                                }],
                            },
                            ImplementedRequirement {
                                uuid: "aa0e8400-e29b-41d4-a716-446655440000".to_string(),
                                control_id: "POL-AC-002".to_string(),
                                description: "Passwords must be 12+ characters.".to_string(),
                                props: vec![],
                                links: vec![],
                            },
                        ],
                    }],
                }],
                capabilities: vec![],
                back_matter: None,
            },
        };

        // Serialize to XML
        let xml =
            serialize_component_definition_to_xml(&original.component_definition).unwrap();

        // Deserialize back
        let round_tripped = deserialize_component_from_xml(&xml).unwrap();
        let rt = &round_tripped.component_definition;

        // Verify top-level fields
        assert_eq!(rt.uuid, original.component_definition.uuid);
        assert_eq!(rt.metadata.title, "Round-Trip Test");

        // Verify component
        assert_eq!(rt.components.len(), 1);
        let comp = &rt.components[0];
        assert_eq!(comp.uuid, "770e8400-e29b-41d4-a716-446655440000");
        assert_eq!(comp.title, "Test Policy");
        assert_eq!(comp.description, "A test policy.");

        // Verify control-implementations survived
        assert_eq!(comp.control_implementations.len(), 1);
        let ci = &comp.control_implementations[0];
        assert_eq!(ci.uuid, "880e8400-e29b-41d4-a716-446655440000");
        assert_eq!(ci.source, "./baseline.json");
        assert_eq!(ci.description, "Implementation narratives.");

        // Verify implemented-requirements survived
        assert_eq!(ci.implemented_requirements.len(), 2);

        let ir0 = &ci.implemented_requirements[0];
        assert_eq!(ir0.uuid, "990e8400-e29b-41d4-a716-446655440000");
        assert_eq!(ir0.control_id, "POL-AC-001");
        assert_eq!(ir0.description, "All users must authenticate.");
        assert_eq!(ir0.props.len(), 1);
        assert_eq!(ir0.props[0].name, "source-file");
        assert_eq!(ir0.props[0].value, "policy.md");
        assert_eq!(
            ir0.props[0].ns.as_deref(),
            Some("https://forge.policy-forge.github.io/ns/trace")
        );
        assert_eq!(ir0.links.len(), 1);
        assert_eq!(ir0.links[0].href, "policy.md#line=10");
        assert_eq!(ir0.links[0].rel, "source");

        let ir1 = &ci.implemented_requirements[1];
        assert_eq!(ir1.uuid, "aa0e8400-e29b-41d4-a716-446655440000");
        assert_eq!(ir1.control_id, "POL-AC-002");
        assert_eq!(ir1.description, "Passwords must be 12+ characters.");
        assert!(ir1.props.is_empty());
        assert!(ir1.links.is_empty());
    }

    #[test]
    fn xml_deser_control_implementation_invalid_uuid_errors() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<component-definition xmlns="http://csrc.nist.gov/ns/oscal/1.0" uuid="b2c3d4e5-f6a7-4b8c-9d0e-1f2a3b4c5d6e">
  <metadata>
    <title>Test</title>
    <last-modified>2026-01-01T00:00:00Z</last-modified>
    <version>1.0</version>
    <oscal-version>1.2.0</oscal-version>
  </metadata>
  <component uuid="c3d4e5f6-a7b8-4c9d-ae1f-2a3b4c5d6e7f" type="policy">
    <title>Test</title>
    <control-implementation uuid="NOT-A-VALID-UUID" source="./profile.json">
      <description><p>Test</p></description>
    </control-implementation>
  </component>
</component-definition>"#;

        let result = deserialize_component_from_xml(xml);
        assert!(result.is_err(), "Invalid UUID in control-implementation should error");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Invalid UUID"),
            "Error should mention invalid UUID, got: {err}"
        );
    }

    #[test]
    fn xml_deser_implemented_requirement_invalid_uuid_errors() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<component-definition xmlns="http://csrc.nist.gov/ns/oscal/1.0" uuid="b2c3d4e5-f6a7-4b8c-9d0e-1f2a3b4c5d6e">
  <metadata>
    <title>Test</title>
    <last-modified>2026-01-01T00:00:00Z</last-modified>
    <version>1.0</version>
    <oscal-version>1.2.0</oscal-version>
  </metadata>
  <component uuid="c3d4e5f6-a7b8-4c9d-ae1f-2a3b4c5d6e7f" type="policy">
    <title>Test</title>
    <control-implementation uuid="880e8400-e29b-41d4-a716-446655440000" source="./p.json">
      <description><p>Test</p></description>
      <implemented-requirement uuid="BAD-UUID" control-id="AC-1">
        <description><p>Test</p></description>
      </implemented-requirement>
    </control-implementation>
  </component>
</component-definition>"#;

        let result = deserialize_component_from_xml(xml);
        assert!(result.is_err(), "Invalid UUID in implemented-requirement should error");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Invalid UUID"),
            "Error should mention invalid UUID, got: {err}"
        );
    }

    #[test]
    fn xml_deser_component_without_control_implementations() {
        let xml = include_str!("../../tests/fixtures/export/component.xml");
        let result = deserialize_component_from_xml(xml).unwrap();
        let comp = &result.component_definition.components[0];
        assert!(
            comp.control_implementations.is_empty(),
            "Component without control-implementations should deserialize with empty vec"
        );
    }
}
