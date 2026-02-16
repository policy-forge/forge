//! OSCAL XML deserialization: converts OSCAL XML strings back to typed model structs.
//!
//! Provides the inverse of `xml_serializer.rs` — parses XML produced by
//! `serialize_catalog_to_xml` and `serialize_component_definition_to_xml`
//! back into `CatalogEnvelope` and `ComponentDefinitionEnvelope`.
//!
//! All XML is parsed via `quick_xml::Reader` — manual event-based parsing
//! for consistency with the Writer-based serializer (no serde).

use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::reader::Reader;
use uuid::Uuid;

use crate::error::ForgeError;
use crate::oscal::back_matter::{
    BackMatter, BackMatterResource, OscalLink, Prop, ResourceCitation, Rlink,
};
use crate::oscal::catalog::{
    CatalogEnvelope, OscalCatalog, OscalControl, OscalGroup, OscalMetadata,
};
use crate::oscal::component_definition::{
    ComponentDefinition, ComponentDefinitionEnvelope, ComponentDefinitionMetadata,
    DocumentaryComponent,
};
use crate::oscal::parts::{OscalPart, OscalProp};

// ─── Error Helpers ───────────────────────────────────────────────────────

/// Map a `quick_xml::Error` to `ForgeError::Serialization`.
#[allow(clippy::needless_pass_by_value)]
fn map_read_err(e: quick_xml::Error) -> ForgeError {
    ForgeError::Serialization(format!("XML read error: {e}"))
}

/// Create a `ForgeError::Serialization` from a message string.
fn xml_err(msg: impl Into<String>) -> ForgeError {
    ForgeError::Serialization(msg.into())
}

// ─── Attribute Extraction ────────────────────────────────────────────────

/// Extract a named attribute value from a `BytesStart` element.
///
/// Returns `None` if the attribute is not present. Returns an error if
/// the attribute value cannot be decoded as UTF-8.
fn get_attr(
    start: &quick_xml::events::BytesStart<'_>,
    name: &str,
) -> Result<Option<String>, ForgeError> {
    for attr_result in start.attributes() {
        let attr = attr_result.map_err(|e| xml_err(format!("attribute error: {e}")))?;
        if attr.key == QName(name.as_bytes()) {
            let value = attr
                .unescape_value()
                .map_err(|e| xml_err(format!("attribute unescape error: {e}")))?;
            return Ok(Some(value.into_owned()));
        }
    }
    Ok(None)
}

/// Extract a required named attribute value from a `BytesStart` element.
///
/// Returns an error if the attribute is not found.
fn require_attr(
    start: &quick_xml::events::BytesStart<'_>,
    name: &str,
    element: &str,
) -> Result<String, ForgeError> {
    get_attr(start, name)?
        .ok_or_else(|| xml_err(format!("missing '{name}' attribute on <{element}>")))
}

// ─── Text Content Helpers ────────────────────────────────────────────────

/// Read the text content of a simple element and consume its closing tag.
///
/// Assumes the `Start` event has already been consumed. Reads `Text` events
/// (accumulating them) until the matching `End` event is found.
fn read_text_content(reader: &mut Reader<&[u8]>, tag: &str) -> Result<String, ForgeError> {
    let mut content = String::new();
    loop {
        match reader.read_event().map_err(map_read_err)? {
            Event::Text(e) => {
                let text = e.unescape().map_err(|e| xml_err(format!("unescape error: {e}")))?;
                content.push_str(&text);
            }
            Event::End(e) if e.name() == QName(tag.as_bytes()) => {
                return Ok(content);
            }
            Event::Eof => {
                return Err(xml_err(format!("unexpected EOF inside <{tag}>")));
            }
            _ => {
                // Skip comments, processing instructions, etc.
            }
        }
    }
}

/// Read the text inside a `<p>` element that is wrapped in an outer element.
///
/// E.g., for `<description><p>text</p></description>`, call this after
/// consuming the `<description>` start event. Returns the text from within
/// `<p>` tags, then consumes the closing `</outer>` tag.
fn read_markup_content(reader: &mut Reader<&[u8]>, outer_tag: &str) -> Result<String, ForgeError> {
    let mut content = String::new();
    loop {
        match reader.read_event().map_err(map_read_err)? {
            Event::Start(e) if e.name() == QName(b"p") => {
                content = read_text_content(reader, "p")?;
            }
            Event::End(e) if e.name() == QName(outer_tag.as_bytes()) => {
                return Ok(content);
            }
            Event::Eof => {
                return Err(xml_err(format!("unexpected EOF inside <{outer_tag}>")));
            }
            _ => {}
        }
    }
}

/// Skip an element and all its children.
///
/// Assumes the `Start` event for `tag` has already been consumed.
fn skip_element(reader: &mut Reader<&[u8]>, tag: &str) -> Result<(), ForgeError> {
    let mut depth: usize = 1;
    loop {
        match reader.read_event().map_err(map_read_err)? {
            Event::Start(_) => depth += 1,
            Event::End(e) => {
                depth -= 1;
                if depth == 0 {
                    debug_assert_eq!(
                        e.name(),
                        QName(tag.as_bytes()),
                        "mismatched close tag when skipping <{tag}>"
                    );
                    return Ok(());
                }
            }
            Event::Eof => return Err(xml_err(format!("unexpected EOF while skipping <{tag}>"))),
            _ => {}
        }
    }
}

// ─── Element Parsers ─────────────────────────────────────────────────────

/// Parse `<metadata>` children into an `OscalMetadata`.
fn parse_metadata(reader: &mut Reader<&[u8]>) -> Result<OscalMetadata, ForgeError> {
    let mut title = String::new();
    let mut last_modified = String::new();
    let mut version = String::new();
    let mut oscal_version = String::new();

    loop {
        match reader.read_event().map_err(map_read_err)? {
            Event::Start(e) => {
                let tag_name = e.name();
                match tag_name.as_ref() {
                    b"title" => title = read_text_content(reader, "title")?,
                    b"last-modified" => {
                        last_modified = read_text_content(reader, "last-modified")?;
                    }
                    b"version" => version = read_text_content(reader, "version")?,
                    b"oscal-version" => {
                        oscal_version = read_text_content(reader, "oscal-version")?;
                    }
                    other => {
                        let name = String::from_utf8_lossy(other).to_string();
                        skip_element(reader, &name)?;
                    }
                }
            }
            Event::End(e) if e.name() == QName(b"metadata") => break,
            Event::Eof => return Err(xml_err("unexpected EOF inside <metadata>")),
            _ => {}
        }
    }

    Ok(OscalMetadata { title, last_modified, version, oscal_version })
}

/// Parse a `<prop name="..." value="..." [ns="..."] />` element.
///
/// Handles both self-closing (`Event::Empty`) and open/close forms.
/// The `start` parameter is the `BytesStart` from the event.
fn parse_oscal_prop(start: &quick_xml::events::BytesStart<'_>) -> Result<OscalProp, ForgeError> {
    let name = require_attr(start, "name", "prop")?;
    let value = require_attr(start, "value", "prop")?;
    let ns = get_attr(start, "ns")?;
    Ok(OscalProp { name, ns, value })
}

/// Parse a `<prop name="..." value="..." />` element into a back-matter `Prop`.
fn parse_back_matter_prop(start: &quick_xml::events::BytesStart<'_>) -> Result<Prop, ForgeError> {
    let name = require_attr(start, "name", "prop")?;
    let value = require_attr(start, "value", "prop")?;
    Ok(Prop { name, value })
}

/// Parse a `<link>` element (self-closing or with `<text>` child).
fn parse_link_from_empty(
    start: &quick_xml::events::BytesStart<'_>,
) -> Result<OscalLink, ForgeError> {
    let href = require_attr(start, "href", "link")?;
    let rel = require_attr(start, "rel", "link")?;
    Ok(OscalLink { href, rel, text: None })
}

/// Parse a `<link>` element that has children (i.e., `Event::Start`).
fn parse_link_with_children(
    reader: &mut Reader<&[u8]>,
    start: &quick_xml::events::BytesStart<'_>,
) -> Result<OscalLink, ForgeError> {
    let href = require_attr(start, "href", "link")?;
    let rel = require_attr(start, "rel", "link")?;
    let mut text = None;

    loop {
        match reader.read_event().map_err(map_read_err)? {
            Event::Start(e) if e.name() == QName(b"text") => {
                text = Some(read_text_content(reader, "text")?);
            }
            Event::Start(e) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                skip_element(reader, &name)?;
            }
            Event::End(e) if e.name() == QName(b"link") => break,
            Event::Eof => return Err(xml_err("unexpected EOF inside <link>")),
            _ => {}
        }
    }

    Ok(OscalLink { href, rel, text })
}

/// Parse a `<part>` element and its children recursively.
fn parse_part(
    reader: &mut Reader<&[u8]>,
    start: &quick_xml::events::BytesStart<'_>,
) -> Result<OscalPart, ForgeError> {
    let id = require_attr(start, "id", "part")?;
    let name = require_attr(start, "name", "part")?;
    let mut prose = String::new();
    let mut parts = Vec::new();
    let mut props = Vec::new();

    loop {
        match reader.read_event().map_err(map_read_err)? {
            Event::Start(e) => {
                let tag_name = e.name();
                match tag_name.as_ref() {
                    b"p" => {
                        prose = read_text_content(reader, "p")?;
                    }
                    b"part" => {
                        parts.push(parse_part(reader, &e)?);
                    }
                    b"prop" => {
                        props.push(parse_oscal_prop(&e)?);
                        // Consume the closing </prop> tag
                        skip_element(reader, "prop")?;
                    }
                    other => {
                        let tag = String::from_utf8_lossy(other).to_string();
                        skip_element(reader, &tag)?;
                    }
                }
            }
            Event::Empty(e) => {
                let tag_name = e.name();
                if tag_name.as_ref() == b"prop" {
                    props.push(parse_oscal_prop(&e)?);
                }
                // Self-closing elements like <p/> are rare but harmless to ignore
            }
            Event::End(e) if e.name() == QName(b"part") => break,
            Event::Eof => return Err(xml_err("unexpected EOF inside <part>")),
            _ => {}
        }
    }

    Ok(OscalPart { id, name, prose, parts, props })
}

/// Parse a `<control>` element and its children.
fn parse_control(
    reader: &mut Reader<&[u8]>,
    start: &quick_xml::events::BytesStart<'_>,
) -> Result<OscalControl, ForgeError> {
    let id = require_attr(start, "id", "control")?;
    let mut title = String::new();
    let mut links = Vec::new();
    let mut parts = Vec::new();
    let mut props = Vec::new();

    loop {
        match reader.read_event().map_err(map_read_err)? {
            Event::Start(e) => {
                let tag_name = e.name();
                match tag_name.as_ref() {
                    b"title" => title = read_text_content(reader, "title")?,
                    b"link" => links.push(parse_link_with_children(reader, &e)?),
                    b"part" => parts.push(parse_part(reader, &e)?),
                    b"prop" => {
                        props.push(parse_oscal_prop(&e)?);
                        skip_element(reader, "prop")?;
                    }
                    other => {
                        let tag = String::from_utf8_lossy(other).to_string();
                        skip_element(reader, &tag)?;
                    }
                }
            }
            Event::Empty(e) => {
                let tag_name = e.name();
                match tag_name.as_ref() {
                    b"prop" => props.push(parse_oscal_prop(&e)?),
                    b"link" => links.push(parse_link_from_empty(&e)?),
                    _ => {}
                }
            }
            Event::End(e) if e.name() == QName(b"control") => break,
            Event::Eof => return Err(xml_err("unexpected EOF inside <control>")),
            _ => {}
        }
    }

    Ok(OscalControl {
        id,
        uuid: String::new(), // XML serializer does not emit uuid on controls
        title,
        links,
        parts,
        props,
    })
}

/// Parse a `<group>` element and its children.
fn parse_group(
    reader: &mut Reader<&[u8]>,
    start: &quick_xml::events::BytesStart<'_>,
) -> Result<OscalGroup, ForgeError> {
    let id = require_attr(start, "id", "group")?;
    let mut title = String::new();
    let mut props = Vec::new();
    let mut links = Vec::new();
    let mut controls = Vec::new();

    loop {
        match reader.read_event().map_err(map_read_err)? {
            Event::Start(e) => {
                let tag_name = e.name();
                match tag_name.as_ref() {
                    b"title" => title = read_text_content(reader, "title")?,
                    b"control" => controls.push(parse_control(reader, &e)?),
                    b"link" => links.push(parse_link_with_children(reader, &e)?),
                    b"prop" => {
                        props.push(parse_oscal_prop(&e)?);
                        skip_element(reader, "prop")?;
                    }
                    other => {
                        let tag = String::from_utf8_lossy(other).to_string();
                        skip_element(reader, &tag)?;
                    }
                }
            }
            Event::Empty(e) => {
                let tag_name = e.name();
                match tag_name.as_ref() {
                    b"prop" => props.push(parse_oscal_prop(&e)?),
                    b"link" => links.push(parse_link_from_empty(&e)?),
                    _ => {}
                }
            }
            Event::End(e) if e.name() == QName(b"group") => break,
            Event::Eof => return Err(xml_err("unexpected EOF inside <group>")),
            _ => {}
        }
    }

    Ok(OscalGroup { id, title, props, links, controls })
}

/// Parse a `<resource>` element and its children.
fn parse_resource(
    reader: &mut Reader<&[u8]>,
    start: &quick_xml::events::BytesStart<'_>,
) -> Result<BackMatterResource, ForgeError> {
    let uuid_str = require_attr(start, "uuid", "resource")?;
    let uuid = Uuid::try_parse(&uuid_str)
        .map_err(|e| xml_err(format!("invalid UUID in <resource>: {e}")))?;

    let mut title = String::new();
    let mut description = None;
    let mut citation = None;
    let mut rlinks = Vec::new();
    let mut props = Vec::new();

    loop {
        match reader.read_event().map_err(map_read_err)? {
            Event::Start(e) => {
                let tag_name = e.name();
                match tag_name.as_ref() {
                    b"title" => title = read_text_content(reader, "title")?,
                    b"description" => {
                        description = Some(read_markup_content(reader, "description")?);
                    }
                    b"citation" => {
                        citation = Some(parse_citation(reader)?);
                    }
                    b"prop" => {
                        props.push(parse_back_matter_prop(&e)?);
                        skip_element(reader, "prop")?;
                    }
                    b"rlink" => {
                        rlinks.push(parse_rlink_from_start(&e)?);
                        skip_element(reader, "rlink")?;
                    }
                    other => {
                        let tag = String::from_utf8_lossy(other).to_string();
                        skip_element(reader, &tag)?;
                    }
                }
            }
            Event::Empty(e) => {
                let tag_name = e.name();
                match tag_name.as_ref() {
                    b"prop" => props.push(parse_back_matter_prop(&e)?),
                    b"rlink" => rlinks.push(parse_rlink_from_start(&e)?),
                    _ => {}
                }
            }
            Event::End(e) if e.name() == QName(b"resource") => break,
            Event::Eof => return Err(xml_err("unexpected EOF inside <resource>")),
            _ => {}
        }
    }

    Ok(BackMatterResource { uuid, title, description, citation, rlinks, props })
}

/// Parse a `<citation>` element: `<citation><text>...</text></citation>`.
fn parse_citation(reader: &mut Reader<&[u8]>) -> Result<ResourceCitation, ForgeError> {
    let mut text = String::new();

    loop {
        match reader.read_event().map_err(map_read_err)? {
            Event::Start(e) if e.name() == QName(b"text") => {
                text = read_text_content(reader, "text")?;
            }
            Event::Start(e) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                skip_element(reader, &tag)?;
            }
            Event::End(e) if e.name() == QName(b"citation") => break,
            Event::Eof => return Err(xml_err("unexpected EOF inside <citation>")),
            _ => {}
        }
    }

    Ok(ResourceCitation { text })
}

/// Parse an `<rlink>` element from its `BytesStart` (attributes only).
fn parse_rlink_from_start(start: &quick_xml::events::BytesStart<'_>) -> Result<Rlink, ForgeError> {
    let href = require_attr(start, "href", "rlink")?;
    let media_type = get_attr(start, "media-type")?;
    Ok(Rlink { href, media_type })
}

/// Parse `<back-matter>` and its children.
fn parse_back_matter(reader: &mut Reader<&[u8]>) -> Result<BackMatter, ForgeError> {
    let mut resources = Vec::new();

    loop {
        match reader.read_event().map_err(map_read_err)? {
            Event::Start(e) if e.name() == QName(b"resource") => {
                resources.push(parse_resource(reader, &e)?);
            }
            Event::Start(e) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                skip_element(reader, &tag)?;
            }
            Event::End(e) if e.name() == QName(b"back-matter") => break,
            Event::Eof => return Err(xml_err("unexpected EOF inside <back-matter>")),
            _ => {}
        }
    }

    Ok(BackMatter { resources })
}

/// Parse a `<component>` element and its children.
fn parse_component(
    reader: &mut Reader<&[u8]>,
    start: &quick_xml::events::BytesStart<'_>,
) -> Result<DocumentaryComponent, ForgeError> {
    let uuid = require_attr(start, "uuid", "component")?;
    let component_type = require_attr(start, "type", "component")?;
    let mut title = String::new();
    let mut description = String::new();
    let mut props = Vec::new();

    loop {
        match reader.read_event().map_err(map_read_err)? {
            Event::Start(e) => {
                let tag_name = e.name();
                match tag_name.as_ref() {
                    b"title" => title = read_text_content(reader, "title")?,
                    b"description" => {
                        description = read_markup_content(reader, "description")?;
                    }
                    b"prop" => {
                        props.push(parse_oscal_prop(&e)?);
                        skip_element(reader, "prop")?;
                    }
                    other => {
                        let tag = String::from_utf8_lossy(other).to_string();
                        skip_element(reader, &tag)?;
                    }
                }
            }
            Event::Empty(e) => {
                if e.name() == QName(b"prop") {
                    props.push(parse_oscal_prop(&e)?);
                }
            }
            Event::End(e) if e.name() == QName(b"component") => break,
            Event::Eof => return Err(xml_err("unexpected EOF inside <component>")),
            _ => {}
        }
    }

    Ok(DocumentaryComponent {
        uuid,
        component_type,
        title,
        description,
        props,
        control_implementations: vec![], // Not serialized in XML
    })
}

// ─── Public API ──────────────────────────────────────────────────────────

/// Deserialize an OSCAL Catalog from an XML string.
///
/// Uses `quick_xml::Reader` with manual event-based parsing to reconstruct
/// a `CatalogEnvelope` from XML produced by `serialize_catalog_to_xml`.
///
/// # Errors
///
/// Returns `ForgeError::Serialization` if XML parsing or deserialization fails.
pub fn deserialize_catalog_from_xml(xml: &str) -> Result<CatalogEnvelope, ForgeError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut uuid = String::new();
    let mut metadata = None;
    let mut groups = Vec::new();
    let mut back_matter = None;

    loop {
        match reader.read_event().map_err(map_read_err)? {
            Event::Start(e) if e.name() == QName(b"catalog") => {
                uuid = require_attr(&e, "uuid", "catalog")?;
                // Parse children of <catalog>
                loop {
                    match reader.read_event().map_err(map_read_err)? {
                        Event::Start(child) => {
                            let tag_name = child.name();
                            match tag_name.as_ref() {
                                b"metadata" => metadata = Some(parse_metadata(&mut reader)?),
                                b"group" => groups.push(parse_group(&mut reader, &child)?),
                                b"back-matter" => {
                                    back_matter = Some(parse_back_matter(&mut reader)?);
                                }
                                other => {
                                    let tag = String::from_utf8_lossy(other).to_string();
                                    skip_element(&mut reader, &tag)?;
                                }
                            }
                        }
                        Event::End(e) if e.name() == QName(b"catalog") => break,
                        Event::Eof => {
                            return Err(xml_err("unexpected EOF inside <catalog>"));
                        }
                        _ => {}
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    let meta = metadata.ok_or_else(|| xml_err("missing <metadata> element in <catalog>"))?;

    Ok(CatalogEnvelope { catalog: OscalCatalog { uuid, metadata: meta, groups, back_matter } })
}

/// Deserialize an OSCAL Component Definition from an XML string.
///
/// Uses `quick_xml::Reader` with manual event-based parsing to reconstruct
/// a `ComponentDefinitionEnvelope` from XML produced by
/// `serialize_component_definition_to_xml`.
///
/// # Errors
///
/// Returns `ForgeError::Serialization` if XML parsing or deserialization fails.
pub fn deserialize_component_from_xml(
    xml: &str,
) -> Result<ComponentDefinitionEnvelope, ForgeError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut uuid = String::new();
    let mut metadata = None;
    let mut components = Vec::new();
    let mut back_matter = None;

    loop {
        match reader.read_event().map_err(map_read_err)? {
            Event::Start(e) if e.name() == QName(b"component-definition") => {
                uuid = require_attr(&e, "uuid", "component-definition")?;
                // Parse children
                loop {
                    match reader.read_event().map_err(map_read_err)? {
                        Event::Start(child) => {
                            let tag_name = child.name();
                            match tag_name.as_ref() {
                                b"metadata" => metadata = Some(parse_metadata(&mut reader)?),
                                b"component" => {
                                    components.push(parse_component(&mut reader, &child)?);
                                }
                                b"back-matter" => {
                                    back_matter = Some(parse_back_matter(&mut reader)?);
                                }
                                other => {
                                    let tag = String::from_utf8_lossy(other).to_string();
                                    skip_element(&mut reader, &tag)?;
                                }
                            }
                        }
                        Event::End(e) if e.name() == QName(b"component-definition") => break,
                        Event::Eof => {
                            return Err(xml_err("unexpected EOF inside <component-definition>"));
                        }
                        _ => {}
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    let meta =
        metadata.ok_or_else(|| xml_err("missing <metadata> element in <component-definition>"))?;

    let cd_metadata = ComponentDefinitionMetadata {
        title: meta.title,
        last_modified: meta.last_modified,
        version: meta.version,
        oscal_version: meta.oscal_version,
    };

    Ok(ComponentDefinitionEnvelope {
        component_definition: ComponentDefinition {
            uuid,
            metadata: cd_metadata,
            components,
            back_matter,
        },
    })
}
