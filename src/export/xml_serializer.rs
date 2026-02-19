//! OSCAL XML serialization: converts OSCAL model structs to valid OSCAL v1.2.0 XML.
//!
//! All XML is produced via `quick_xml::Writer` — no string concatenation (SEC-5).
//! Text content is XML-escaped automatically by `BytesText` (SEC-3).
//! Attributes are escaped automatically by `push_attribute()` (SEC-4).

use std::io::Write;

use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use tracing::debug;

use crate::error::ForgeError;
use crate::oscal::back_matter::{BackMatter, BackMatterResource, OscalLink};
use crate::oscal::catalog::{OscalCatalog, OscalControl, OscalGroup};
use crate::oscal::component_definition::{ComponentDefinition, DocumentaryComponent};
use crate::oscal::parts::{OscalPart, OscalProp};

/// OSCAL XML namespace URI — applied to root element only.
pub const OSCAL_NS: &str = "http://csrc.nist.gov/ns/oscal/1.0";

/// XML indentation: 2 spaces per level.
const INDENT_SIZE: usize = 2;

// ─── Error Mapping ───────────────────────────────────────────────────────

/// Map an I/O error (from `quick_xml::Writer::write_event`) to `ForgeError::Serialization`.
#[allow(clippy::needless_pass_by_value)]
fn map_xml_err(e: std::io::Error) -> ForgeError {
    ForgeError::Serialization(format!("XML write error: {e}"))
}

/// Map a UTF-8 conversion error to `ForgeError::Serialization`.
#[allow(clippy::needless_pass_by_value)]
fn map_utf8_err(e: std::string::FromUtf8Error) -> ForgeError {
    ForgeError::Serialization(format!("XML UTF-8 conversion error: {e}"))
}

// ─── Helper Functions ────────────────────────────────────────────────────

/// Write OSCAL metadata elements in XSD order.
///
/// Writes: `<metadata>` → title, last-modified, version, oscal-version → `</metadata>`
fn write_metadata<W: Write>(
    writer: &mut Writer<W>,
    title: &str,
    last_modified: &str,
    version: &str,
    oscal_version: &str,
) -> Result<(), ForgeError> {
    writer.write_event(Event::Start(BytesStart::new("metadata"))).map_err(map_xml_err)?;

    write_text_element(writer, "title", title)?;
    write_text_element(writer, "last-modified", last_modified)?;
    write_text_element(writer, "version", version)?;
    write_text_element(writer, "oscal-version", oscal_version)?;

    writer.write_event(Event::End(BytesEnd::new("metadata"))).map_err(map_xml_err)?;

    Ok(())
}

/// Write a single OSCAL prop element.
///
/// Writes: `<prop name="..." value="..." [ns="..."] />`
fn write_prop<W: Write>(writer: &mut Writer<W>, prop: &OscalProp) -> Result<(), ForgeError> {
    let mut elem = BytesStart::new("prop");
    elem.push_attribute(("name", prop.name.as_str()));
    elem.push_attribute(("value", prop.value.as_str()));
    if let Some(ns) = &prop.ns {
        elem.push_attribute(("ns", ns.as_str()));
    }
    writer.write_event(Event::Empty(elem)).map_err(map_xml_err)?;
    Ok(())
}

/// Write a single OSCAL param element (WI-34).
///
/// Writes: `<param id="...">` → `<label>`, `<constraint>*`, `<value>*` → `</param>`
fn write_param<W: Write>(
    writer: &mut Writer<W>,
    param: &crate::oscal::catalog::OscalParam,
) -> Result<(), ForgeError> {
    let mut elem = BytesStart::new("param");
    elem.push_attribute(("id", param.id.as_str()));
    writer.write_event(Event::Start(elem)).map_err(map_xml_err)?;

    write_text_element(writer, "label", &param.label)?;

    // XSD order: constraint* before value* (oscal-control-common-parameter-ASSEMBLY)
    // <description> is MarkupMultilineDatatype — content must be wrapped in <p>
    for constraint in &param.constraints {
        writer.write_event(Event::Start(BytesStart::new("constraint"))).map_err(map_xml_err)?;
        writer.write_event(Event::Start(BytesStart::new("description"))).map_err(map_xml_err)?;
        write_text_element(writer, "p", &constraint.description)?;
        writer.write_event(Event::End(BytesEnd::new("description"))).map_err(map_xml_err)?;
        writer.write_event(Event::End(BytesEnd::new("constraint"))).map_err(map_xml_err)?;
    }

    for value in &param.values {
        write_text_element(writer, "value", value)?;
    }

    writer.write_event(Event::End(BytesEnd::new("param"))).map_err(map_xml_err)?;
    Ok(())
}

/// Write a single OSCAL link element.
///
/// When `link.text` is `None`: emit self-closing `<link href="..." rel="..." />`
/// using `Event::Empty`.
/// When `link.text` is `Some(t)`: emit `<link href="..." rel="..."><text>t</text></link>`
/// using `Event::Start` + text child + `Event::End`.
fn write_link<W: Write>(writer: &mut Writer<W>, link: &OscalLink) -> Result<(), ForgeError> {
    let mut elem = BytesStart::new("link");
    elem.push_attribute(("href", link.href.as_str()));
    elem.push_attribute(("rel", link.rel.as_str()));

    match &link.text {
        None => {
            writer.write_event(Event::Empty(elem)).map_err(map_xml_err)?;
        }
        Some(text) => {
            writer.write_event(Event::Start(elem)).map_err(map_xml_err)?;
            write_text_element(writer, "text", text)?;
            writer.write_event(Event::End(BytesEnd::new("link"))).map_err(map_xml_err)?;
        }
    }
    Ok(())
}

/// Write a single OSCAL part element with children in XSD order.
///
/// Writes: `<part id="..." name="...">` → prop*, `<p>prose</p>`, part* → `</part>`
///
/// # Recursion
///
/// Recurses for nested sub-parts (position 4). Practical policy documents
/// rarely exceed 5–10 levels of nesting; the default stack can accommodate
/// thousands of levels safely.
fn write_part<W: Write>(writer: &mut Writer<W>, part: &OscalPart) -> Result<(), ForgeError> {
    let mut elem = BytesStart::new("part");
    elem.push_attribute(("id", part.id.as_str()));
    elem.push_attribute(("name", part.name.as_str()));
    writer.write_event(Event::Start(elem)).map_err(map_xml_err)?;

    // Props in XSD order (position 2)
    for prop in &part.props {
        write_prop(writer, prop)?;
    }

    // Prose as <p> element (position 3 — blockElementGroup)
    if !part.prose.is_empty() {
        write_text_element(writer, "p", &part.prose)?;
    }

    // Nested parts (position 4)
    for sub_part in &part.parts {
        write_part(writer, sub_part)?;
    }

    writer.write_event(Event::End(BytesEnd::new("part"))).map_err(map_xml_err)?;
    Ok(())
}

/// Write OSCAL back-matter element with resource children.
///
/// Writes: `<back-matter>` → resource* → `</back-matter>`
fn write_back_matter<W: Write>(
    writer: &mut Writer<W>,
    back_matter: &BackMatter,
) -> Result<(), ForgeError> {
    writer.write_event(Event::Start(BytesStart::new("back-matter"))).map_err(map_xml_err)?;

    for resource in &back_matter.resources {
        write_resource(writer, resource)?;
    }

    writer.write_event(Event::End(BytesEnd::new("back-matter"))).map_err(map_xml_err)?;
    Ok(())
}

/// Write a single OSCAL resource element with children in XSD order.
///
/// Writes: `<resource uuid="...">` → title?, description?, prop*, citation?, rlink* → `</resource>`
fn write_resource<W: Write>(
    writer: &mut Writer<W>,
    resource: &BackMatterResource,
) -> Result<(), ForgeError> {
    let mut elem = BytesStart::new("resource");
    elem.push_attribute(("uuid", resource.uuid.to_string().as_str()));
    writer.write_event(Event::Start(elem)).map_err(map_xml_err)?;

    // Title (position 1)
    if !resource.title.is_empty() {
        write_text_element(writer, "title", &resource.title)?;
    }

    // Description (position 2) — OSCAL XSD requires markup-multiline (block elements like <p>)
    if let Some(desc) = &resource.description {
        write_markup_element(writer, "description", desc)?;
    }

    // Props (position 3)
    for prop in &resource.props {
        let mut prop_elem = BytesStart::new("prop");
        prop_elem.push_attribute(("name", prop.name.as_str()));
        prop_elem.push_attribute(("value", prop.value.as_str()));
        writer.write_event(Event::Empty(prop_elem)).map_err(map_xml_err)?;
    }

    // Citation (position 5)
    if let Some(citation) = &resource.citation {
        writer.write_event(Event::Start(BytesStart::new("citation"))).map_err(map_xml_err)?;
        write_text_element(writer, "text", &citation.text)?;
        writer.write_event(Event::End(BytesEnd::new("citation"))).map_err(map_xml_err)?;
    }

    // Rlinks (position 6)
    for rlink in &resource.rlinks {
        let mut rlink_elem = BytesStart::new("rlink");
        rlink_elem.push_attribute(("href", rlink.href.as_str()));
        if let Some(media_type) = &rlink.media_type {
            rlink_elem.push_attribute(("media-type", media_type.as_str()));
        }
        writer.write_event(Event::Empty(rlink_elem)).map_err(map_xml_err)?;
    }

    writer.write_event(Event::End(BytesEnd::new("resource"))).map_err(map_xml_err)?;
    Ok(())
}

/// Write a single OSCAL control element with children in XSD order.
///
/// Writes: `<control id="...">` → title, param*, prop*, link*, part* → `</control>`
///
/// Note: `uuid` field is NOT serialized (OSCAL catalog XSD does not allow uuid on controls).
fn write_control<W: Write>(
    writer: &mut Writer<W>,
    control: &OscalControl,
) -> Result<(), ForgeError> {
    let mut elem = BytesStart::new("control");
    elem.push_attribute(("id", control.id.as_str()));
    writer.write_event(Event::Start(elem)).map_err(map_xml_err)?;

    // Title (position 1)
    write_text_element(writer, "title", &control.title)?;

    // Params (position 2, before props per OSCAL schema ordering)
    for param in &control.params {
        write_param(writer, param)?;
    }

    // Props (position 3)
    for prop in &control.props {
        write_prop(writer, prop)?;
    }

    // Links (position 4)
    for link in &control.links {
        write_link(writer, link)?;
    }

    // Parts (position 5)
    for part in &control.parts {
        write_part(writer, part)?;
    }

    writer.write_event(Event::End(BytesEnd::new("control"))).map_err(map_xml_err)?;
    Ok(())
}

/// Write a single OSCAL group element with children in XSD order.
///
/// Writes: `<group id="...">` → title, prop*, link*, control* → `</group>`
fn write_group<W: Write>(writer: &mut Writer<W>, group: &OscalGroup) -> Result<(), ForgeError> {
    let mut elem = BytesStart::new("group");
    elem.push_attribute(("id", group.id.as_str()));
    writer.write_event(Event::Start(elem)).map_err(map_xml_err)?;

    // Title (position 1)
    write_text_element(writer, "title", &group.title)?;

    // Props (position 3)
    for prop in &group.props {
        write_prop(writer, prop)?;
    }

    // Links (position 4)
    for link in &group.links {
        write_link(writer, link)?;
    }

    // Controls (position 7)
    for control in &group.controls {
        write_control(writer, control)?;
    }

    writer.write_event(Event::End(BytesEnd::new("group"))).map_err(map_xml_err)?;
    Ok(())
}

/// Write a single OSCAL component element with children in XSD order.
///
/// Writes: `<component uuid="..." type="...">` → title, description, prop* → `</component>`
///
/// Note: `control_implementations` is SKIPPED for WI-26 (complex nested structure).
fn write_component<W: Write>(
    writer: &mut Writer<W>,
    component: &DocumentaryComponent,
) -> Result<(), ForgeError> {
    let mut elem = BytesStart::new("component");
    elem.push_attribute(("uuid", component.uuid.as_str()));
    elem.push_attribute(("type", component.component_type.as_str()));
    writer.write_event(Event::Start(elem)).map_err(map_xml_err)?;

    // Title (position 1)
    write_text_element(writer, "title", &component.title)?;

    // Description (position 2) — OSCAL XSD requires markup-multiline (block elements like <p>)
    write_markup_element(writer, "description", &component.description)?;

    // Props (position 4)
    for prop in &component.props {
        write_prop(writer, prop)?;
    }

    writer.write_event(Event::End(BytesEnd::new("component"))).map_err(map_xml_err)?;
    Ok(())
}

/// Write a simple text element: `<tag>content</tag>`.
fn write_text_element<W: Write>(
    writer: &mut Writer<W>,
    tag: &str,
    content: &str,
) -> Result<(), ForgeError> {
    writer.write_event(Event::Start(BytesStart::new(tag))).map_err(map_xml_err)?;
    writer.write_event(Event::Text(BytesText::new(content))).map_err(map_xml_err)?;
    writer.write_event(Event::End(BytesEnd::new(tag))).map_err(map_xml_err)?;
    Ok(())
}

/// Write a markup-multiline element: `<tag><p>content</p></tag>`.
///
/// OSCAL XSD defines description as markup-multiline, which requires
/// block-level elements (e.g., `<p>`) rather than bare text content.
/// Skips the element entirely when `content` is empty, consistent with
/// `write_part` and `write_resource` empty-content handling.
fn write_markup_element<W: Write>(
    writer: &mut Writer<W>,
    tag: &str,
    content: &str,
) -> Result<(), ForgeError> {
    if content.is_empty() {
        return Ok(());
    }
    writer.write_event(Event::Start(BytesStart::new(tag))).map_err(map_xml_err)?;
    writer.write_event(Event::Start(BytesStart::new("p"))).map_err(map_xml_err)?;
    writer.write_event(Event::Text(BytesText::new(content))).map_err(map_xml_err)?;
    writer.write_event(Event::End(BytesEnd::new("p"))).map_err(map_xml_err)?;
    writer.write_event(Event::End(BytesEnd::new(tag))).map_err(map_xml_err)?;
    Ok(())
}

// ─── Shared Serialization Helpers ────────────────────────────────────────

/// Create a new XML writer with standard indentation and write the XML declaration.
fn create_xml_writer(buf: &mut Vec<u8>) -> Result<Writer<&mut Vec<u8>>, ForgeError> {
    let mut writer = Writer::new_with_indent(buf, b' ', INDENT_SIZE);
    writer
        .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
        .map_err(map_xml_err)?;
    Ok(writer)
}

/// Write the opening root element with OSCAL namespace and uuid attribute.
fn write_root_start<W: Write>(
    writer: &mut Writer<W>,
    tag: &str,
    uuid: &str,
) -> Result<(), ForgeError> {
    let mut root = BytesStart::new(tag);
    root.push_attribute(("xmlns", OSCAL_NS));
    root.push_attribute(("uuid", uuid));
    writer.write_event(Event::Start(root)).map_err(map_xml_err)?;
    Ok(())
}

/// Finalize the XML document by converting the buffer to a UTF-8 string.
fn finish_xml_document(buf: Vec<u8>) -> Result<String, ForgeError> {
    String::from_utf8(buf).map_err(map_utf8_err)
}

// ─── Public Serialization Functions ──────────────────────────────────────

/// Serialize an OSCAL Catalog to a valid OSCAL v1.2.0 XML string.
///
/// Produces a complete XML document with:
/// - XML declaration (`<?xml version="1.0" encoding="UTF-8"?>`)
/// - Root `<catalog>` element with OSCAL namespace and uuid attribute
/// - Child elements in XSD-prescribed order: metadata, group*, back-matter?
///
/// # Arguments
/// * `catalog` — Reference to the `OscalCatalog` to serialize
///
/// # Errors
/// * `ForgeError::Serialization` — If XML writing or UTF-8 conversion fails
///
/// # Security
/// * All text content is XML-escaped via quick-xml (SEC-3)
/// * No DTD or entity declarations are emitted (SEC-1)
/// * No string concatenation is used for XML construction (SEC-5)
pub fn serialize_catalog_to_xml(catalog: &OscalCatalog) -> Result<String, ForgeError> {
    debug!(artifact_type = "catalog", "Starting XML serialization");

    let mut buf = Vec::new();
    let mut writer = create_xml_writer(&mut buf)?;

    write_root_start(&mut writer, "catalog", &catalog.uuid)?;
    write_metadata(
        &mut writer,
        &catalog.metadata.title,
        &catalog.metadata.last_modified,
        &catalog.metadata.version,
        &catalog.metadata.oscal_version,
    )?;

    for group in &catalog.groups {
        write_group(&mut writer, group)?;
    }

    if let Some(bm) = &catalog.back_matter {
        write_back_matter(&mut writer, bm)?;
    }

    writer.write_event(Event::End(BytesEnd::new("catalog"))).map_err(map_xml_err)?;

    let xml = finish_xml_document(buf)?;
    debug!(artifact_type = "catalog", "XML serialization complete");
    Ok(xml)
}

/// Serialize an OSCAL Component Definition to a valid OSCAL v1.2.0 XML string.
///
/// Produces a complete XML document with:
/// - XML declaration (`<?xml version="1.0" encoding="UTF-8"?>`)
/// - Root `<component-definition>` element with OSCAL namespace and uuid attribute
/// - Child elements in XSD-prescribed order: metadata, component*, back-matter?
///
/// # Arguments
/// * `component_def` — Reference to the `ComponentDefinition` to serialize
///
/// # Errors
/// * `ForgeError::Serialization` — If XML writing or UTF-8 conversion fails
pub fn serialize_component_definition_to_xml(
    component_def: &ComponentDefinition,
) -> Result<String, ForgeError> {
    debug!(artifact_type = "component-definition", "Starting XML serialization");

    let mut buf = Vec::new();
    let mut writer = create_xml_writer(&mut buf)?;

    write_root_start(&mut writer, "component-definition", &component_def.uuid)?;
    write_metadata(
        &mut writer,
        &component_def.metadata.title,
        &component_def.metadata.last_modified,
        &component_def.metadata.version,
        &component_def.metadata.oscal_version,
    )?;

    for component in &component_def.components {
        write_component(&mut writer, component)?;
    }

    if let Some(bm) = &component_def.back_matter {
        write_back_matter(&mut writer, bm)?;
    }

    writer.write_event(Event::End(BytesEnd::new("component-definition"))).map_err(map_xml_err)?;

    let xml = finish_xml_document(buf)?;
    debug!(artifact_type = "component-definition", "XML serialization complete");
    Ok(xml)
}

// ─── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oscal::back_matter::{
        BackMatter, BackMatterResource, Prop, ResourceCitation, Rlink,
    };
    use crate::oscal::catalog::OscalMetadata;
    use crate::oscal::component_definition::ComponentDefinitionMetadata;

    // ── Test helpers ──────────────────────────────────────

    fn test_metadata() -> OscalMetadata {
        OscalMetadata {
            title: "Test Policy".to_string(),
            last_modified: "2026-02-15T12:00:00Z".to_string(),
            version: "1.0".to_string(),
            oscal_version: "1.2.0".to_string(),
        }
    }

    fn test_control(id: &str, title: &str) -> OscalControl {
        OscalControl {
            id: id.to_string(),
            uuid: "skip-me".to_string(),
            title: title.to_string(),
            links: vec![],
            params: vec![],
            parts: vec![OscalPart {
                id: format!("{id}_smt"),
                name: "statement".to_string(),
                prose: title.to_string(),
                parts: vec![],
                props: vec![],
            }],
            props: vec![],
        }
    }

    fn test_group(id: &str, title: &str, controls: Vec<OscalControl>) -> OscalGroup {
        OscalGroup {
            id: id.to_string(),
            title: title.to_string(),
            props: vec![],
            links: vec![],
            controls,
        }
    }

    fn test_catalog() -> OscalCatalog {
        OscalCatalog {
            uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            metadata: test_metadata(),
            groups: vec![test_group(
                "access-control",
                "Access Control",
                vec![test_control("POL-AC-001", "All users must use MFA.")],
            )],
            back_matter: None,
        }
    }

    fn test_component_def() -> ComponentDefinition {
        ComponentDefinition {
            uuid: "660e8400-e29b-41d4-a716-446655440000".to_string(),
            metadata: ComponentDefinitionMetadata {
                title: "Test Policy".to_string(),
                last_modified: "2026-02-15T12:00:00Z".to_string(),
                version: "1.0".to_string(),
                oscal_version: "1.2.0".to_string(),
            },
            components: vec![DocumentaryComponent {
                uuid: "770e8400-e29b-41d4-a716-446655440000".to_string(),
                component_type: "policy".to_string(),
                title: "Test Policy".to_string(),
                description: "Documentary component representing the Test Policy policy document."
                    .to_string(),
                props: vec![],
                control_implementations: vec![],
            }],
            back_matter: None,
        }
    }

    // ══════════════════════════════════════════════════════
    // T004: write_metadata unit tests
    // ══════════════════════════════════════════════════════

    #[test]
    fn test_write_metadata_xsd_order() {
        let mut buf = Vec::new();
        let mut writer = Writer::new_with_indent(&mut buf, b' ', INDENT_SIZE);
        write_metadata(&mut writer, "Test Title", "2026-02-15T12:00:00Z", "1.0", "1.2.0").unwrap();
        let xml = String::from_utf8(buf).unwrap();

        assert!(xml.contains("<metadata>"));
        assert!(xml.contains("<title>Test Title</title>"));
        assert!(xml.contains("<last-modified>2026-02-15T12:00:00Z</last-modified>"));
        assert!(xml.contains("<version>1.0</version>"));
        assert!(xml.contains("<oscal-version>1.2.0</oscal-version>"));
        assert!(xml.contains("</metadata>"));

        // Verify XSD order: title before last-modified before version before oscal-version
        let title_pos = xml.find("<title>").unwrap();
        let lm_pos = xml.find("<last-modified>").unwrap();
        let ver_pos = xml.find("<version>").unwrap();
        let oscal_pos = xml.find("<oscal-version>").unwrap();
        assert!(title_pos < lm_pos);
        assert!(lm_pos < ver_pos);
        assert!(ver_pos < oscal_pos);
    }

    // ══════════════════════════════════════════════════════
    // T006: write_prop unit tests
    // ══════════════════════════════════════════════════════

    #[test]
    fn test_write_prop_basic() {
        let mut buf = Vec::new();
        let mut writer = Writer::new_with_indent(&mut buf, b' ', INDENT_SIZE);
        let prop = OscalProp { name: "label".to_string(), value: "AC-1".to_string(), ns: None };
        write_prop(&mut writer, &prop).unwrap();
        let xml = String::from_utf8(buf).unwrap();
        assert!(xml.contains(r#"<prop name="label" value="AC-1"/>"#));
    }

    #[test]
    fn test_write_prop_with_ns() {
        let mut buf = Vec::new();
        let mut writer = Writer::new_with_indent(&mut buf, b' ', INDENT_SIZE);
        let prop = OscalProp {
            name: "source-file".to_string(),
            value: "policy.md".to_string(),
            ns: Some("https://forge.policy-forge.github.io/ns/trace".to_string()),
        };
        write_prop(&mut writer, &prop).unwrap();
        let xml = String::from_utf8(buf).unwrap();
        assert!(xml.contains(r#"name="source-file""#));
        assert!(xml.contains(r#"value="policy.md""#));
        assert!(xml.contains(r#"ns="https://forge.policy-forge.github.io/ns/trace""#));
        // Self-closing element
        assert!(xml.contains("/>"));
    }

    // ══════════════════════════════════════════════════════
    // T008: write_link unit tests
    // ══════════════════════════════════════════════════════

    #[test]
    fn test_write_link_self_closing() {
        let mut buf = Vec::new();
        let mut writer = Writer::new_with_indent(&mut buf, b' ', INDENT_SIZE);
        let link =
            OscalLink { href: "#uuid-123".to_string(), rel: "reference".to_string(), text: None };
        write_link(&mut writer, &link).unwrap();
        let xml = String::from_utf8(buf).unwrap();
        assert!(xml.contains(r##"<link href="#uuid-123" rel="reference"/>"##));
        // Must be self-closing (no </link>)
        assert!(!xml.contains("</link>"));
    }

    #[test]
    fn test_write_link_with_text() {
        let mut buf = Vec::new();
        let mut writer = Writer::new_with_indent(&mut buf, b' ', INDENT_SIZE);
        let link = OscalLink {
            href: "#uuid-456".to_string(),
            rel: "reference".to_string(),
            text: Some("NIST SP 800-53".to_string()),
        };
        write_link(&mut writer, &link).unwrap();
        let xml = String::from_utf8(buf).unwrap();
        assert!(xml.contains(r##"<link href="#uuid-456" rel="reference">"##));
        assert!(xml.contains("<text>NIST SP 800-53</text>"));
        assert!(xml.contains("</link>"));
    }

    #[test]
    fn test_write_link_with_empty_text() {
        let mut buf = Vec::new();
        let mut writer = Writer::new_with_indent(&mut buf, b' ', INDENT_SIZE);
        let link = OscalLink {
            href: "#uuid-789".to_string(),
            rel: "reference".to_string(),
            text: Some(String::new()),
        };
        write_link(&mut writer, &link).unwrap();
        let xml = String::from_utf8(buf).unwrap();
        // Empty text still emits the <text> child element (not self-closing link)
        assert!(xml.contains("<text>"));
        assert!(xml.contains("</link>"));
    }

    // ══════════════════════════════════════════════════════
    // T010: write_part unit tests
    // ══════════════════════════════════════════════════════

    #[test]
    fn test_write_part_basic() {
        let mut buf = Vec::new();
        let mut writer = Writer::new_with_indent(&mut buf, b' ', INDENT_SIZE);
        let part = OscalPart {
            id: "POL-AC-001_smt".to_string(),
            name: "statement".to_string(),
            prose: "All users must use MFA.".to_string(),
            parts: vec![],
            props: vec![],
        };
        write_part(&mut writer, &part).unwrap();
        let xml = String::from_utf8(buf).unwrap();
        assert!(xml.contains(r#"<part id="POL-AC-001_smt" name="statement">"#));
        assert!(xml.contains("<p>All users must use MFA.</p>"));
        assert!(xml.contains("</part>"));
    }

    #[test]
    fn test_write_part_with_props_and_nested() {
        let mut buf = Vec::new();
        let mut writer = Writer::new_with_indent(&mut buf, b' ', INDENT_SIZE);
        let part = OscalPart {
            id: "POL-AC-001_smt".to_string(),
            name: "statement".to_string(),
            prose: "Parent statement.".to_string(),
            parts: vec![OscalPart {
                id: "POL-AC-001_smt.a".to_string(),
                name: "item".to_string(),
                prose: "Sub-item A.".to_string(),
                parts: vec![],
                props: vec![],
            }],
            props: vec![OscalProp { name: "label".to_string(), value: "a.".to_string(), ns: None }],
        };
        write_part(&mut writer, &part).unwrap();
        let xml = String::from_utf8(buf).unwrap();

        // Verify XSD order: props before prose before nested parts
        let prop_pos = xml.find(r"<prop ").unwrap();
        let prose_pos = xml.find("<p>Parent statement.</p>").unwrap();
        let nested_pos = xml.find(r#"<part id="POL-AC-001_smt.a""#).unwrap();
        assert!(prop_pos < prose_pos);
        assert!(prose_pos < nested_pos);
    }

    #[test]
    fn test_write_part_empty_prose_skipped() {
        let mut buf = Vec::new();
        let mut writer = Writer::new_with_indent(&mut buf, b' ', INDENT_SIZE);
        let part = OscalPart {
            id: "POL-AC-001_smt".to_string(),
            name: "statement".to_string(),
            prose: String::new(),
            parts: vec![],
            props: vec![],
        };
        write_part(&mut writer, &part).unwrap();
        let xml = String::from_utf8(buf).unwrap();
        assert!(!xml.contains("<p>"), "Empty prose should not produce <p> element");
    }

    // ══════════════════════════════════════════════════════
    // T012: write_back_matter and write_resource unit tests
    // ══════════════════════════════════════════════════════

    #[test]
    fn test_write_resource_full() {
        let mut buf = Vec::new();
        let mut writer = Writer::new_with_indent(&mut buf, b' ', INDENT_SIZE);
        let resource = BackMatterResource {
            uuid: uuid::Uuid::parse_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890").unwrap(),
            title: "NIST SP 800-53".to_string(),
            description: Some("Referenced standard".to_string()),
            props: vec![],
            citation: Some(ResourceCitation { text: "NIST SP 800-53 Rev 5".to_string() }),
            rlinks: vec![Rlink {
                href: "https://nvd.nist.gov/800-53".to_string(),
                media_type: None,
            }],
        };
        write_resource(&mut writer, &resource).unwrap();
        let xml = String::from_utf8(buf).unwrap();

        assert!(xml.contains(r#"<resource uuid="a1b2c3d4-e5f6-7890-abcd-ef1234567890">"#));
        assert!(xml.contains("<title>NIST SP 800-53</title>"));
        assert!(xml.contains("<p>Referenced standard</p>"));
        assert!(xml.contains("<citation>"));
        assert!(xml.contains("<text>NIST SP 800-53 Rev 5</text>"));
        assert!(xml.contains("</citation>"));
        assert!(xml.contains(r#"<rlink href="https://nvd.nist.gov/800-53"/>"#));
        assert!(xml.contains("</resource>"));

        // Verify XSD order: title before description before citation before rlinks
        let title_pos = xml.find("<title>NIST").unwrap();
        let desc_pos = xml.find("<description>").unwrap();
        let citation_pos = xml.find("<citation>").unwrap();
        let rlink_pos = xml.find("<rlink").unwrap();
        assert!(title_pos < desc_pos);
        assert!(desc_pos < citation_pos);
        assert!(citation_pos < rlink_pos);
    }

    #[test]
    fn test_write_resource_with_props() {
        let mut buf = Vec::new();
        let mut writer = Writer::new_with_indent(&mut buf, b' ', INDENT_SIZE);
        let resource = BackMatterResource {
            uuid: uuid::Uuid::parse_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890").unwrap(),
            title: "Bad ref".to_string(),
            description: None,
            props: vec![Prop { name: "url-status".to_string(), value: "unvalidated".to_string() }],
            citation: None,
            rlinks: vec![Rlink { href: "not-a-url".to_string(), media_type: None }],
        };
        write_resource(&mut writer, &resource).unwrap();
        let xml = String::from_utf8(buf).unwrap();
        assert!(xml.contains(r#"<prop name="url-status" value="unvalidated"/>"#));
    }

    #[test]
    fn test_write_resource_with_media_type() {
        let mut buf = Vec::new();
        let mut writer = Writer::new_with_indent(&mut buf, b' ', INDENT_SIZE);
        let resource = BackMatterResource {
            uuid: uuid::Uuid::parse_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890").unwrap(),
            title: "PDF Guide".to_string(),
            description: None,
            props: vec![],
            citation: None,
            rlinks: vec![Rlink {
                href: "https://example.com/guide.pdf".to_string(),
                media_type: Some("application/pdf".to_string()),
            }],
        };
        write_resource(&mut writer, &resource).unwrap();
        let xml = String::from_utf8(buf).unwrap();
        assert!(xml.contains(
            r#"<rlink href="https://example.com/guide.pdf" media-type="application/pdf"/>"#
        ));
    }

    #[test]
    fn test_write_back_matter() {
        let mut buf = Vec::new();
        let mut writer = Writer::new_with_indent(&mut buf, b' ', INDENT_SIZE);
        let bm = BackMatter {
            resources: vec![BackMatterResource {
                uuid: uuid::Uuid::parse_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890").unwrap(),
                title: "Ref".to_string(),
                description: None,
                props: vec![],
                citation: None,
                rlinks: vec![],
            }],
        };
        write_back_matter(&mut writer, &bm).unwrap();
        let xml = String::from_utf8(buf).unwrap();
        assert!(xml.contains("<back-matter>"));
        assert!(xml.contains("<resource"));
        assert!(xml.contains("</back-matter>"));
    }

    // ══════════════════════════════════════════════════════
    // T014: error mapping unit tests
    // ══════════════════════════════════════════════════════

    #[test]
    fn test_map_xml_err() {
        let err = map_xml_err(std::io::Error::other("test"));
        assert!(matches!(err, ForgeError::Serialization(_)));
        assert!(err.to_string().contains("XML write error"));
    }

    #[test]
    fn test_map_utf8_err() {
        let bad_bytes = vec![0xFF, 0xFE];
        let utf8_err = String::from_utf8(bad_bytes).unwrap_err();
        let err = map_utf8_err(utf8_err);
        assert!(matches!(err, ForgeError::Serialization(_)));
        assert!(err.to_string().contains("XML UTF-8 conversion error"));
    }

    // ══════════════════════════════════════════════════════
    // T016: write_control unit tests
    // ══════════════════════════════════════════════════════

    #[test]
    fn test_write_control_xsd_order() {
        let mut buf = Vec::new();
        let mut writer = Writer::new_with_indent(&mut buf, b' ', INDENT_SIZE);
        let control = OscalControl {
            id: "POL-AC-001".to_string(),
            uuid: "should-not-appear".to_string(),
            title: "All users must use MFA.".to_string(),
            props: vec![OscalProp {
                name: "label".to_string(),
                value: "AC-1".to_string(),
                ns: None,
            }],
            links: vec![OscalLink {
                href: "#uuid-ref".to_string(),
                rel: "reference".to_string(),
                text: None,
            }],
            params: vec![],
            parts: vec![OscalPart {
                id: "POL-AC-001_smt".to_string(),
                name: "statement".to_string(),
                prose: "All users must use MFA.".to_string(),
                parts: vec![],
                props: vec![],
            }],
        };
        write_control(&mut writer, &control).unwrap();
        let xml = String::from_utf8(buf).unwrap();

        // ID is attribute
        assert!(xml.contains(r#"<control id="POL-AC-001">"#));
        // UUID is NOT serialized
        assert!(!xml.contains("should-not-appear"));
        // Verify XSD order: title → props → links → parts
        let title_pos = xml.find("<title>").unwrap();
        let prop_pos = xml.find(r"<prop ").unwrap();
        let link_pos = xml.find(r"<link ").unwrap();
        let part_pos = xml.find(r"<part ").unwrap();
        assert!(title_pos < prop_pos);
        assert!(prop_pos < link_pos);
        assert!(link_pos < part_pos);
    }

    #[test]
    fn test_write_control_uuid_not_serialized() {
        let mut buf = Vec::new();
        let mut writer = Writer::new_with_indent(&mut buf, b' ', INDENT_SIZE);
        let control = test_control("POL-AC-001", "MFA required.");
        write_control(&mut writer, &control).unwrap();
        let xml = String::from_utf8(buf).unwrap();
        assert!(!xml.contains("skip-me"), "Control uuid must not be serialized");
        assert!(!xml.contains(r"uuid="), "No uuid attribute on control");
    }

    // ══════════════════════════════════════════════════════
    // T017: write_group unit tests
    // ══════════════════════════════════════════════════════

    #[test]
    fn test_write_group_xsd_order() {
        let mut buf = Vec::new();
        let mut writer = Writer::new_with_indent(&mut buf, b' ', INDENT_SIZE);
        let group = OscalGroup {
            id: "access-control".to_string(),
            title: "Access Control".to_string(),
            props: vec![OscalProp {
                name: "source-section".to_string(),
                value: "Section 3".to_string(),
                ns: None,
            }],
            links: vec![OscalLink {
                href: "#ref-1".to_string(),
                rel: "reference".to_string(),
                text: None,
            }],
            controls: vec![test_control("POL-AC-001", "MFA required.")],
        };
        write_group(&mut writer, &group).unwrap();
        let xml = String::from_utf8(buf).unwrap();

        assert!(xml.contains(r#"<group id="access-control">"#));
        // Verify XSD order: title → props → links → controls
        let title_pos = xml.find("<title>Access Control</title>").unwrap();
        let prop_pos = xml.find(r"<prop ").unwrap();
        let link_pos = xml.find(r"<link ").unwrap();
        let control_pos = xml.find(r"<control ").unwrap();
        assert!(title_pos < prop_pos);
        assert!(prop_pos < link_pos);
        assert!(link_pos < control_pos);
    }

    // ══════════════════════════════════════════════════════
    // T018: serialize_catalog_to_xml unit tests
    // ══════════════════════════════════════════════════════

    #[test]
    fn test_serialize_catalog_xml_declaration() {
        let catalog = test_catalog();
        let xml = serialize_catalog_to_xml(&catalog).unwrap();
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    }

    #[test]
    fn test_serialize_catalog_namespace() {
        let catalog = test_catalog();
        let xml = serialize_catalog_to_xml(&catalog).unwrap();
        assert!(xml.contains(r#"xmlns="http://csrc.nist.gov/ns/oscal/1.0""#));
    }

    #[test]
    fn test_serialize_catalog_uuid_attribute() {
        let catalog = test_catalog();
        let xml = serialize_catalog_to_xml(&catalog).unwrap();
        assert!(xml.contains(r#"uuid="550e8400-e29b-41d4-a716-446655440000""#));
    }

    #[test]
    fn test_serialize_catalog_complete_structure() {
        let catalog = test_catalog();
        let xml = serialize_catalog_to_xml(&catalog).unwrap();

        assert!(xml.contains("<catalog"));
        assert!(xml.contains("<metadata>"));
        assert!(xml.contains("<title>Test Policy</title>"));
        assert!(xml.contains("<group"));
        assert!(xml.contains("<control"));
        assert!(xml.contains("<part"));
        assert!(xml.contains("</catalog>"));
    }

    #[test]
    fn test_serialize_catalog_with_back_matter() {
        let mut catalog = test_catalog();
        catalog.back_matter = Some(BackMatter {
            resources: vec![BackMatterResource {
                uuid: uuid::Uuid::parse_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890").unwrap(),
                title: "NIST SP 800-53".to_string(),
                description: None,
                props: vec![],
                citation: None,
                rlinks: vec![Rlink {
                    href: "https://nvd.nist.gov/800-53".to_string(),
                    media_type: None,
                }],
            }],
        });
        let xml = serialize_catalog_to_xml(&catalog).unwrap();
        assert!(xml.contains("<back-matter>"));
        assert!(xml.contains("<resource"));
        assert!(xml.contains("</back-matter>"));
    }

    #[test]
    fn test_serialize_catalog_no_back_matter_when_none() {
        let catalog = test_catalog();
        let xml = serialize_catalog_to_xml(&catalog).unwrap();
        assert!(!xml.contains("<back-matter>"), "No back-matter when None");
    }

    // ══════════════════════════════════════════════════════
    // T019: write_component unit tests
    // ══════════════════════════════════════════════════════

    #[test]
    fn test_write_component_xsd_order() {
        let mut buf = Vec::new();
        let mut writer = Writer::new_with_indent(&mut buf, b' ', INDENT_SIZE);
        let component = DocumentaryComponent {
            uuid: "comp-uuid-123".to_string(),
            component_type: "policy".to_string(),
            title: "Security Policy".to_string(),
            description: "A policy document.".to_string(),
            props: vec![OscalProp {
                name: "source-file".to_string(),
                value: "policy.md".to_string(),
                ns: Some("https://forge.policy-forge.github.io/ns/trace".to_string()),
            }],
            control_implementations: vec![],
        };
        write_component(&mut writer, &component).unwrap();
        let xml = String::from_utf8(buf).unwrap();

        assert!(xml.contains(r#"<component uuid="comp-uuid-123" type="policy">"#));
        // Verify XSD order: title → description → props
        let title_pos = xml.find("<title>Security Policy</title>").unwrap();
        let desc_pos = xml.find("<description>").unwrap();
        assert!(xml.contains("<description>\n"), "description must use markup-multiline format");
        assert!(xml.contains("<p>A policy document.</p>"), "description must wrap content in <p>");
        let prop_pos = xml.find(r"<prop ").unwrap();
        assert!(title_pos < desc_pos);
        assert!(desc_pos < prop_pos);
    }

    #[test]
    fn test_write_component_control_implementations_skipped() {
        let mut buf = Vec::new();
        let mut writer = Writer::new_with_indent(&mut buf, b' ', INDENT_SIZE);
        let component = DocumentaryComponent {
            uuid: "comp-uuid".to_string(),
            component_type: "policy".to_string(),
            title: "Title".to_string(),
            description: "Desc".to_string(),
            props: vec![],
            control_implementations: vec![serde_json::json!({"source": "test"})],
        };
        write_component(&mut writer, &component).unwrap();
        let xml = String::from_utf8(buf).unwrap();
        assert!(
            !xml.contains("control-implementations"),
            "control_implementations must be skipped in WI-26"
        );
    }

    // ══════════════════════════════════════════════════════
    // T020: serialize_component_definition_to_xml unit tests
    // ══════════════════════════════════════════════════════

    #[test]
    fn test_serialize_component_def_xml_declaration() {
        let cd = test_component_def();
        let xml = serialize_component_definition_to_xml(&cd).unwrap();
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    }

    #[test]
    fn test_serialize_component_def_namespace() {
        let cd = test_component_def();
        let xml = serialize_component_definition_to_xml(&cd).unwrap();
        assert!(xml.contains(r#"xmlns="http://csrc.nist.gov/ns/oscal/1.0""#));
    }

    #[test]
    fn test_serialize_component_def_uuid_attribute() {
        let cd = test_component_def();
        let xml = serialize_component_definition_to_xml(&cd).unwrap();
        assert!(xml.contains(r#"uuid="660e8400-e29b-41d4-a716-446655440000""#));
    }

    #[test]
    fn test_serialize_component_def_complete_structure() {
        let cd = test_component_def();
        let xml = serialize_component_definition_to_xml(&cd).unwrap();

        assert!(xml.contains("<component-definition"));
        assert!(xml.contains("<metadata>"));
        assert!(xml.contains("<component"));
        assert!(xml.contains(r#"type="policy""#));
        assert!(xml.contains("</component-definition>"));
    }

    // ══════════════════════════════════════════════════════
    // T041: SEC-1 — no DTD declarations
    // ══════════════════════════════════════════════════════

    #[test]
    fn test_no_dtd_in_output() {
        let catalog = test_catalog();
        let xml = serialize_catalog_to_xml(&catalog).unwrap();
        assert!(!xml.contains("<!DOCTYPE"), "Output must not contain DOCTYPE");
        assert!(!xml.contains("<!ENTITY"), "Output must not contain ENTITY");
    }

    // ══════════════════════════════════════════════════════
    // T042: SEC-2, SEC-3 — adversarial input escaping
    // ══════════════════════════════════════════════════════

    #[test]
    fn test_xml_escaping_in_text_content() {
        let catalog = OscalCatalog {
            uuid: "test-uuid".to_string(),
            metadata: OscalMetadata {
                title: "<script>alert('xss')</script>".to_string(),
                last_modified: "2026-01-01T00:00:00Z".to_string(),
                version: "1.0".to_string(),
                oscal_version: "1.2.0".to_string(),
            },
            groups: vec![test_group(
                "test",
                "Test",
                vec![OscalControl {
                    id: "test-ctrl".to_string(),
                    uuid: "u".to_string(),
                    title: "&entity; injection".to_string(),
                    links: vec![],
                    params: vec![],
                    parts: vec![OscalPart {
                        id: "test_smt".to_string(),
                        name: "statement".to_string(),
                        prose: "]]> CDATA escape & <!-- comment -->".to_string(),
                        parts: vec![],
                        props: vec![],
                    }],
                    props: vec![],
                }],
            )],
            back_matter: None,
        };
        let xml = serialize_catalog_to_xml(&catalog).unwrap();

        // Script tags must be escaped
        assert!(!xml.contains("<script>"), "Script tags must be escaped");
        assert!(xml.contains("&lt;script&gt;"));
        // Entity references must be escaped
        assert!(xml.contains("&amp;entity;"));
        // CDATA end must be escaped
        assert!(xml.contains("]]&gt;"));
        // Comment markers must be escaped
        assert!(xml.contains("&lt;!-- comment --&gt;"));
    }

    // ══════════════════════════════════════════════════════
    // T043: SEC-4 — attribute escaping
    // ══════════════════════════════════════════════════════

    #[test]
    fn test_attribute_escaping() {
        let catalog = OscalCatalog {
            uuid: r#"test"uuid<"#.to_string(),
            metadata: test_metadata(),
            groups: vec![],
            back_matter: None,
        };
        let xml = serialize_catalog_to_xml(&catalog).unwrap();
        // Quotes and < in attributes must be escaped
        assert!(!xml.contains(r#"uuid="test"uuid<""#), "Attributes must be escaped");
        assert!(xml.contains("&quot;") || xml.contains("&lt;"));
    }

    // ══════════════════════════════════════════════════════
    // T044: SEC-6 — namespace isolation
    // ══════════════════════════════════════════════════════

    #[test]
    fn test_namespace_isolation() {
        let catalog = OscalCatalog {
            uuid: "test-uuid".to_string(),
            metadata: test_metadata(),
            groups: vec![test_group(
                "test",
                "xmlns=http://evil.com",
                vec![test_control("test-ctrl", "xmlns=attack")],
            )],
            back_matter: None,
        };
        let xml = serialize_catalog_to_xml(&catalog).unwrap();

        // User content with xmlns= must NOT create actual namespace declarations.
        // Verify only the root <catalog> element has xmlns attribute.
        assert!(
            xml.contains(r#"<catalog xmlns="http://csrc.nist.gov/ns/oscal/1.0""#),
            "Root element must have OSCAL namespace"
        );
        // No child elements should have xmlns attributes — only text content
        assert!(
            !xml.contains(r"<group xmlns="),
            "Group must not get xmlns attribute from user content"
        );
        assert!(
            !xml.contains(r"<control xmlns="),
            "Control must not get xmlns attribute from user content"
        );
        // User content is preserved as text (not as XML structure)
        assert!(xml.contains("xmlns=http://evil.com"));
        assert!(xml.contains("xmlns=attack"));
    }

    // ══════════════════════════════════════════════════════
    // T045: EC-1 — empty groups
    // ══════════════════════════════════════════════════════

    #[test]
    fn test_empty_group_produces_valid_xml() {
        let catalog = OscalCatalog {
            uuid: "test-uuid".to_string(),
            metadata: test_metadata(),
            groups: vec![OscalGroup {
                id: "empty-group".to_string(),
                title: "Empty Group".to_string(),
                props: vec![],
                links: vec![],
                controls: vec![],
            }],
            back_matter: None,
        };
        let xml = serialize_catalog_to_xml(&catalog).unwrap();
        assert!(xml.contains(r#"<group id="empty-group">"#));
        assert!(xml.contains("<title>Empty Group</title>"));
        assert!(xml.contains("</group>"));
    }

    // ══════════════════════════════════════════════════════
    // T046: EC-5 — deeply nested parts
    // ══════════════════════════════════════════════════════

    #[test]
    fn test_deeply_nested_parts() {
        let catalog = OscalCatalog {
            uuid: "test-uuid".to_string(),
            metadata: test_metadata(),
            groups: vec![test_group(
                "test",
                "Test",
                vec![OscalControl {
                    id: "deep-ctrl".to_string(),
                    uuid: "u".to_string(),
                    title: "Deep control".to_string(),
                    links: vec![],
                    params: vec![],
                    parts: vec![OscalPart {
                        id: "deep_smt".to_string(),
                        name: "statement".to_string(),
                        prose: "Level 1".to_string(),
                        parts: vec![OscalPart {
                            id: "deep_smt.a".to_string(),
                            name: "item".to_string(),
                            prose: "Level 2".to_string(),
                            parts: vec![OscalPart {
                                id: "deep_smt.a.1".to_string(),
                                name: "item".to_string(),
                                prose: "Level 3".to_string(),
                                parts: vec![],
                                props: vec![],
                            }],
                            props: vec![],
                        }],
                        props: vec![],
                    }],
                    props: vec![],
                }],
            )],
            back_matter: None,
        };
        let xml = serialize_catalog_to_xml(&catalog).unwrap();
        assert!(xml.contains("Level 1"));
        assert!(xml.contains("Level 2"));
        assert!(xml.contains("Level 3"));
        assert!(xml.contains(r#"id="deep_smt.a.1""#));
    }

    // ══════════════════════════════════════════════════════
    // T047: EC-6 — 2-space indentation
    // ══════════════════════════════════════════════════════

    #[test]
    fn test_two_space_indentation() {
        let catalog = test_catalog();
        let xml = serialize_catalog_to_xml(&catalog).unwrap();
        // Metadata should be indented 2 spaces from catalog
        assert!(xml.contains("\n  <metadata>"));
        // Title in metadata should be 4 spaces
        assert!(xml.contains("\n    <title>"));
    }

    // ══════════════════════════════════════════════════════
    // T048: EC-7 — rlink with media-type
    // ══════════════════════════════════════════════════════

    #[test]
    fn test_rlink_media_type_attribute() {
        let mut catalog = test_catalog();
        catalog.back_matter = Some(BackMatter {
            resources: vec![BackMatterResource {
                uuid: uuid::Uuid::parse_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890").unwrap(),
                title: "PDF".to_string(),
                description: None,
                props: vec![],
                citation: None,
                rlinks: vec![Rlink {
                    href: "https://example.com/doc.pdf".to_string(),
                    media_type: Some("application/pdf".to_string()),
                }],
            }],
        });
        let xml = serialize_catalog_to_xml(&catalog).unwrap();
        assert!(xml.contains(r#"media-type="application/pdf""#));
    }

    // ══════════════════════════════════════════════════════
    // T055: EC-3 — namespace-prefixed prop names
    // ══════════════════════════════════════════════════════

    #[test]
    fn test_ns_prefixed_prop_name() {
        let mut buf = Vec::new();
        let mut writer = Writer::new_with_indent(&mut buf, b' ', INDENT_SIZE);
        let prop = OscalProp {
            name: "ns:custom-name".to_string(),
            value: "custom-value".to_string(),
            ns: None,
        };
        write_prop(&mut writer, &prop).unwrap();
        let xml = String::from_utf8(buf).unwrap();
        assert!(xml.contains(r#"<prop name="ns:custom-name" value="custom-value"/>"#));
        // Must not create spurious namespace declarations
        assert!(!xml.contains("xmlns:ns"), "Must not create spurious xmlns:ns declarations");
    }

    // ══════════════════════════════════════════════════════
    // T034: Insta snapshot tests for regression detection
    // ══════════════════════════════════════════════════════

    #[test]
    fn snapshot_catalog_xml() {
        let mut catalog = test_catalog();
        // Add back-matter for more complete snapshot
        catalog.back_matter = Some(BackMatter {
            resources: vec![BackMatterResource {
                uuid: uuid::Uuid::parse_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890").unwrap(),
                title: "NIST SP 800-53".to_string(),
                description: Some("Referenced standard".to_string()),
                props: vec![],
                citation: Some(ResourceCitation { text: "NIST SP 800-53 Rev 5".to_string() }),
                rlinks: vec![Rlink {
                    href: "https://nvd.nist.gov/800-53".to_string(),
                    media_type: None,
                }],
            }],
        });
        let xml = serialize_catalog_to_xml(&catalog).unwrap();
        insta::assert_snapshot!("catalog_xml", xml);
    }

    #[test]
    fn snapshot_component_definition_xml() {
        let cd = test_component_def();
        let xml = serialize_component_definition_to_xml(&cd).unwrap();
        insta::assert_snapshot!("component_definition_xml", xml);
    }
}
