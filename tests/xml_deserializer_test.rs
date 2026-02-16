//! Unit tests for OSCAL XML deserialization.
//!
//! Tests verify that `deserialize_catalog_from_xml` and
//! `deserialize_component_from_xml` correctly round-trip through
//! the XML serializer and back.

use uuid::Uuid;

use forge::error::ForgeError;
use forge::export::xml_deserializer::{
    deserialize_catalog_from_xml, deserialize_component_from_xml,
};
use forge::export::xml_serializer::{
    serialize_catalog_to_xml, serialize_component_definition_to_xml,
};
use forge::oscal::back_matter::{
    BackMatter, BackMatterResource, OscalLink, Prop, ResourceCitation, Rlink,
};
use forge::oscal::catalog::{OscalCatalog, OscalControl, OscalGroup, OscalMetadata};
use forge::oscal::component_definition::{
    ComponentDefinition, ComponentDefinitionMetadata, DocumentaryComponent,
};
use forge::oscal::parts::{OscalPart, OscalProp};

// ── Test Helpers ─────────────────────────────────────────

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

fn test_catalog() -> OscalCatalog {
    OscalCatalog {
        uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        metadata: test_metadata(),
        groups: vec![OscalGroup {
            id: "access-control".to_string(),
            title: "Access Control".to_string(),
            props: vec![OscalProp {
                name: "source-section".to_string(),
                value: "Section 3".to_string(),
                ns: Some("https://forge.policy-forge.github.io/ns/trace".to_string()),
            }],
            links: vec![
                OscalLink {
                    href: "#ref-001".to_string(),
                    rel: "reference".to_string(),
                    text: None,
                },
                OscalLink {
                    href: "#ref-002".to_string(),
                    rel: "reference".to_string(),
                    text: Some("NIST SP 800-53".to_string()),
                },
            ],
            controls: vec![OscalControl {
                id: "POL-AC-001".to_string(),
                uuid: "should-not-round-trip".to_string(),
                title: "All users must use MFA.".to_string(),
                links: vec![OscalLink {
                    href: "#ref-001".to_string(),
                    rel: "reference".to_string(),
                    text: None,
                }],
                parts: vec![OscalPart {
                    id: "POL-AC-001_smt".to_string(),
                    name: "statement".to_string(),
                    prose: "All users must use MFA.".to_string(),
                    parts: vec![OscalPart {
                        id: "POL-AC-001_smt.a".to_string(),
                        name: "item".to_string(),
                        prose: "Sub-item A.".to_string(),
                        parts: vec![],
                        props: vec![],
                    }],
                    props: vec![OscalProp {
                        name: "label".to_string(),
                        value: "a.".to_string(),
                        ns: None,
                    }],
                }],
                props: vec![OscalProp {
                    name: "label".to_string(),
                    value: "AC-1".to_string(),
                    ns: None,
                }],
            }],
        }],
        back_matter: Some(BackMatter {
            resources: vec![BackMatterResource {
                uuid: Uuid::parse_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890").unwrap(),
                title: "NIST SP 800-53".to_string(),
                description: Some("Referenced standard".to_string()),
                citation: Some(ResourceCitation { text: "NIST SP 800-53 Rev 5".to_string() }),
                rlinks: vec![Rlink {
                    href: "https://nvd.nist.gov/800-53".to_string(),
                    media_type: None,
                }],
                props: vec![],
            }],
        }),
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
            props: vec![OscalProp {
                name: "source-file".to_string(),
                value: "policy.md".to_string(),
                ns: Some("https://forge.policy-forge.github.io/ns/trace".to_string()),
            }],
            control_implementations: vec![],
        }],
        back_matter: None,
    }
}

// ══════════════════════════════════════════════════════════
// T023: deserialize_catalog_from_xml round-trips a catalog
// ══════════════════════════════════════════════════════════

#[test]
fn test_catalog_round_trip() {
    let catalog = test_catalog();
    let xml = serialize_catalog_to_xml(&catalog).unwrap();
    let envelope = deserialize_catalog_from_xml(&xml).unwrap();
    let rt = &envelope.catalog;

    assert_eq!(rt.uuid, catalog.uuid);
    assert_eq!(rt.groups.len(), catalog.groups.len());

    let orig_group = &catalog.groups[0];
    let rt_group = &rt.groups[0];
    assert_eq!(rt_group.id, orig_group.id);
    assert_eq!(rt_group.title, orig_group.title);
    assert_eq!(rt_group.controls.len(), orig_group.controls.len());

    let orig_ctrl = &orig_group.controls[0];
    let rt_ctrl = &rt_group.controls[0];
    assert_eq!(rt_ctrl.id, orig_ctrl.id);
    assert_eq!(rt_ctrl.title, orig_ctrl.title);

    // uuid is not serialized in XML, so it should be empty after round-trip
    assert_eq!(rt_ctrl.uuid, "");

    // Back matter round-trips
    assert!(rt.back_matter.is_some());
    let rt_bm = rt.back_matter.as_ref().unwrap();
    let orig_bm = catalog.back_matter.as_ref().unwrap();
    assert_eq!(rt_bm.resources.len(), orig_bm.resources.len());
    assert_eq!(rt_bm.resources[0].uuid, orig_bm.resources[0].uuid);
}

// ══════════════════════════════════════════════════════════
// T024: deserialize_catalog_from_xml preserves metadata
// ══════════════════════════════════════════════════════════

#[test]
fn test_catalog_preserves_metadata() {
    let catalog = test_catalog();
    let xml = serialize_catalog_to_xml(&catalog).unwrap();
    let envelope = deserialize_catalog_from_xml(&xml).unwrap();
    let rt = &envelope.catalog;

    assert_eq!(rt.uuid, "550e8400-e29b-41d4-a716-446655440000");
    assert_eq!(rt.metadata.title, "Test Policy");
    assert_eq!(rt.metadata.last_modified, "2026-02-15T12:00:00Z");
    assert_eq!(rt.metadata.version, "1.0");
    assert_eq!(rt.metadata.oscal_version, "1.2.0");
}

// ══════════════════════════════════════════════════════════
// T025: deserialize_catalog_from_xml preserves group/control structure
// ══════════════════════════════════════════════════════════

#[test]
fn test_catalog_preserves_group_and_control_structure() {
    let catalog = test_catalog();
    let xml = serialize_catalog_to_xml(&catalog).unwrap();
    let envelope = deserialize_catalog_from_xml(&xml).unwrap();
    let rt = &envelope.catalog;

    // Group
    let group = &rt.groups[0];
    assert_eq!(group.id, "access-control");
    assert_eq!(group.title, "Access Control");

    // Group props
    assert_eq!(group.props.len(), 1);
    assert_eq!(group.props[0].name, "source-section");
    assert_eq!(group.props[0].value, "Section 3");
    assert_eq!(group.props[0].ns.as_deref(), Some("https://forge.policy-forge.github.io/ns/trace"));

    // Group links
    assert_eq!(group.links.len(), 2);
    assert_eq!(group.links[0].href, "#ref-001");
    assert_eq!(group.links[0].rel, "reference");
    assert!(group.links[0].text.is_none());
    assert_eq!(group.links[1].href, "#ref-002");
    assert_eq!(group.links[1].text.as_deref(), Some("NIST SP 800-53"));

    // Control
    let ctrl = &group.controls[0];
    assert_eq!(ctrl.id, "POL-AC-001");
    assert_eq!(ctrl.title, "All users must use MFA.");

    // Control props
    assert_eq!(ctrl.props.len(), 1);
    assert_eq!(ctrl.props[0].name, "label");
    assert_eq!(ctrl.props[0].value, "AC-1");

    // Control links
    assert_eq!(ctrl.links.len(), 1);
    assert_eq!(ctrl.links[0].href, "#ref-001");

    // Parts (including nested)
    assert_eq!(ctrl.parts.len(), 1);
    let part = &ctrl.parts[0];
    assert_eq!(part.id, "POL-AC-001_smt");
    assert_eq!(part.name, "statement");
    assert_eq!(part.prose, "All users must use MFA.");

    // Part props
    assert_eq!(part.props.len(), 1);
    assert_eq!(part.props[0].name, "label");
    assert_eq!(part.props[0].value, "a.");

    // Nested sub-parts
    assert_eq!(part.parts.len(), 1);
    assert_eq!(part.parts[0].id, "POL-AC-001_smt.a");
    assert_eq!(part.parts[0].name, "item");
    assert_eq!(part.parts[0].prose, "Sub-item A.");

    // Back-matter resource details
    let bm = rt.back_matter.as_ref().unwrap();
    let resource = &bm.resources[0];
    assert_eq!(resource.uuid.to_string(), "a1b2c3d4-e5f6-7890-abcd-ef1234567890");
    assert_eq!(resource.title, "NIST SP 800-53");
    assert_eq!(resource.description.as_deref(), Some("Referenced standard"));
    assert_eq!(resource.citation.as_ref().unwrap().text, "NIST SP 800-53 Rev 5");
    assert_eq!(resource.rlinks.len(), 1);
    assert_eq!(resource.rlinks[0].href, "https://nvd.nist.gov/800-53");
}

// ══════════════════════════════════════════════════════════
// T026: deserialize_component_from_xml round-trips
// ══════════════════════════════════════════════════════════

#[test]
fn test_component_round_trip() {
    let comp_def = test_component_def();
    let xml = serialize_component_definition_to_xml(&comp_def).unwrap();
    let envelope = deserialize_component_from_xml(&xml).unwrap();
    let rt = &envelope.component_definition;

    assert_eq!(rt.uuid, comp_def.uuid);
    assert_eq!(rt.metadata.title, comp_def.metadata.title);
    assert_eq!(rt.metadata.last_modified, comp_def.metadata.last_modified);
    assert_eq!(rt.metadata.version, comp_def.metadata.version);
    assert_eq!(rt.metadata.oscal_version, comp_def.metadata.oscal_version);
    assert_eq!(rt.components.len(), comp_def.components.len());

    let orig_comp = &comp_def.components[0];
    let rt_comp = &rt.components[0];
    assert_eq!(rt_comp.uuid, orig_comp.uuid);
    assert_eq!(rt_comp.component_type, orig_comp.component_type);
    assert_eq!(rt_comp.title, orig_comp.title);
    assert_eq!(rt_comp.description, orig_comp.description);

    // control_implementations should be empty (not serialized in XML)
    assert!(rt_comp.control_implementations.is_empty());

    // Back matter is None
    assert!(rt.back_matter.is_none());
}

// ══════════════════════════════════════════════════════════
// T027: deserialize_component_from_xml preserves component fields
// ══════════════════════════════════════════════════════════

#[test]
fn test_component_preserves_fields() {
    let comp_def = test_component_def();
    let xml = serialize_component_definition_to_xml(&comp_def).unwrap();
    let envelope = deserialize_component_from_xml(&xml).unwrap();
    let rt_comp = &envelope.component_definition.components[0];

    assert_eq!(rt_comp.uuid, "770e8400-e29b-41d4-a716-446655440000");
    assert_eq!(rt_comp.component_type, "policy");
    assert_eq!(rt_comp.title, "Test Policy");
    assert_eq!(
        rt_comp.description,
        "Documentary component representing the Test Policy policy document."
    );

    // Props preserved
    assert_eq!(rt_comp.props.len(), 1);
    assert_eq!(rt_comp.props[0].name, "source-file");
    assert_eq!(rt_comp.props[0].value, "policy.md");
    assert_eq!(
        rt_comp.props[0].ns.as_deref(),
        Some("https://forge.policy-forge.github.io/ns/trace")
    );
}

// ══════════════════════════════════════════════════════════
// T028: deserialize_catalog_from_xml returns error on invalid XML
// ══════════════════════════════════════════════════════════

#[test]
fn test_catalog_invalid_xml_returns_serialization_error() {
    let invalid_xml = "this is not xml at all";
    let result = deserialize_catalog_from_xml(invalid_xml);
    // No <catalog> element found, metadata will be missing
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, ForgeError::Serialization(_)),
        "Expected ForgeError::Serialization, got: {err:?}"
    );
}

#[test]
fn test_catalog_malformed_xml_returns_error() {
    let malformed_xml = "<?xml version=\"1.0\"?><catalog uuid=\"x\"><metadata><title>T</title>";
    let result = deserialize_catalog_from_xml(malformed_xml);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ForgeError::Serialization(_)));
}

#[test]
fn test_component_invalid_xml_returns_error() {
    let result = deserialize_component_from_xml("<broken>");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ForgeError::Serialization(_)));
}

// ══════════════════════════════════════════════════════════
// Additional edge-case tests
// ══════════════════════════════════════════════════════════

#[test]
fn test_catalog_no_back_matter_round_trips() {
    let catalog = OscalCatalog {
        uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        metadata: test_metadata(),
        groups: vec![OscalGroup {
            id: "test".to_string(),
            title: "Test".to_string(),
            props: vec![],
            links: vec![],
            controls: vec![test_control("POL-T-001", "Test control.")],
        }],
        back_matter: None,
    };
    let xml = serialize_catalog_to_xml(&catalog).unwrap();
    let envelope = deserialize_catalog_from_xml(&xml).unwrap();
    assert!(envelope.catalog.back_matter.is_none());
}

#[test]
fn test_catalog_empty_groups_round_trips() {
    let catalog = OscalCatalog {
        uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        metadata: test_metadata(),
        groups: vec![],
        back_matter: None,
    };
    let xml = serialize_catalog_to_xml(&catalog).unwrap();
    let envelope = deserialize_catalog_from_xml(&xml).unwrap();
    assert!(envelope.catalog.groups.is_empty());
}

#[test]
fn test_resource_with_props_round_trips() {
    let catalog = OscalCatalog {
        uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        metadata: test_metadata(),
        groups: vec![],
        back_matter: Some(BackMatter {
            resources: vec![BackMatterResource {
                uuid: Uuid::parse_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890").unwrap(),
                title: "Bad ref".to_string(),
                description: None,
                citation: None,
                rlinks: vec![Rlink { href: "not-a-url".to_string(), media_type: None }],
                props: vec![Prop {
                    name: "url-status".to_string(),
                    value: "unvalidated".to_string(),
                }],
            }],
        }),
    };
    let xml = serialize_catalog_to_xml(&catalog).unwrap();
    let envelope = deserialize_catalog_from_xml(&xml).unwrap();
    let bm = envelope.catalog.back_matter.as_ref().unwrap();
    assert_eq!(bm.resources[0].props.len(), 1);
    assert_eq!(bm.resources[0].props[0].name, "url-status");
    assert_eq!(bm.resources[0].props[0].value, "unvalidated");
}

#[test]
fn test_rlink_with_media_type_round_trips() {
    let catalog = OscalCatalog {
        uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        metadata: test_metadata(),
        groups: vec![],
        back_matter: Some(BackMatter {
            resources: vec![BackMatterResource {
                uuid: Uuid::parse_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890").unwrap(),
                title: "PDF Guide".to_string(),
                description: None,
                citation: None,
                rlinks: vec![Rlink {
                    href: "https://example.com/guide.pdf".to_string(),
                    media_type: Some("application/pdf".to_string()),
                }],
                props: vec![],
            }],
        }),
    };
    let xml = serialize_catalog_to_xml(&catalog).unwrap();
    let envelope = deserialize_catalog_from_xml(&xml).unwrap();
    let bm = envelope.catalog.back_matter.as_ref().unwrap();
    assert_eq!(bm.resources[0].rlinks[0].media_type.as_deref(), Some("application/pdf"));
}

#[test]
fn test_component_with_back_matter_round_trips() {
    let comp_def = ComponentDefinition {
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
            description: "Test description.".to_string(),
            props: vec![],
            control_implementations: vec![],
        }],
        back_matter: Some(BackMatter {
            resources: vec![BackMatterResource {
                uuid: Uuid::parse_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890").unwrap(),
                title: "Ref".to_string(),
                description: None,
                citation: None,
                rlinks: vec![],
                props: vec![],
            }],
        }),
    };
    let xml = serialize_component_definition_to_xml(&comp_def).unwrap();
    let envelope = deserialize_component_from_xml(&xml).unwrap();
    assert!(envelope.component_definition.back_matter.is_some());
    let bm = envelope.component_definition.back_matter.as_ref().unwrap();
    assert_eq!(bm.resources.len(), 1);
    assert_eq!(bm.resources[0].title, "Ref");
}

#[test]
fn test_xml_escaped_content_round_trips() {
    let catalog = OscalCatalog {
        uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        metadata: OscalMetadata {
            title: "<script>alert('xss')</script>".to_string(),
            last_modified: "2026-02-15T12:00:00Z".to_string(),
            version: "1.0".to_string(),
            oscal_version: "1.2.0".to_string(),
        },
        groups: vec![OscalGroup {
            id: "test".to_string(),
            title: "Test & <Demo>".to_string(),
            props: vec![],
            links: vec![],
            controls: vec![OscalControl {
                id: "POL-T-001".to_string(),
                uuid: String::new(),
                title: "Control with \"quotes\" & entities".to_string(),
                links: vec![],
                parts: vec![OscalPart {
                    id: "POL-T-001_smt".to_string(),
                    name: "statement".to_string(),
                    prose: "Prose with <html> & special ]]> chars".to_string(),
                    parts: vec![],
                    props: vec![],
                }],
                props: vec![],
            }],
        }],
        back_matter: None,
    };
    let xml = serialize_catalog_to_xml(&catalog).unwrap();
    let envelope = deserialize_catalog_from_xml(&xml).unwrap();
    let rt = &envelope.catalog;

    assert_eq!(rt.metadata.title, "<script>alert('xss')</script>");
    assert_eq!(rt.groups[0].title, "Test & <Demo>");
    assert_eq!(rt.groups[0].controls[0].title, "Control with \"quotes\" & entities");
    assert_eq!(rt.groups[0].controls[0].parts[0].prose, "Prose with <html> & special ]]> chars");
}

#[test]
fn test_deeply_nested_parts_round_trip() {
    let catalog = OscalCatalog {
        uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        metadata: test_metadata(),
        groups: vec![OscalGroup {
            id: "test".to_string(),
            title: "Test".to_string(),
            props: vec![],
            links: vec![],
            controls: vec![OscalControl {
                id: "POL-T-001".to_string(),
                uuid: String::new(),
                title: "Deep".to_string(),
                links: vec![],
                parts: vec![OscalPart {
                    id: "smt".to_string(),
                    name: "statement".to_string(),
                    prose: "Level 1".to_string(),
                    props: vec![],
                    parts: vec![OscalPart {
                        id: "smt.a".to_string(),
                        name: "item".to_string(),
                        prose: "Level 2".to_string(),
                        props: vec![],
                        parts: vec![OscalPart {
                            id: "smt.a.1".to_string(),
                            name: "item".to_string(),
                            prose: "Level 3".to_string(),
                            props: vec![],
                            parts: vec![],
                        }],
                    }],
                }],
                props: vec![],
            }],
        }],
        back_matter: None,
    };
    let xml = serialize_catalog_to_xml(&catalog).unwrap();
    let envelope = deserialize_catalog_from_xml(&xml).unwrap();
    let part = &envelope.catalog.groups[0].controls[0].parts[0];

    assert_eq!(part.prose, "Level 1");
    assert_eq!(part.parts[0].prose, "Level 2");
    assert_eq!(part.parts[0].parts[0].prose, "Level 3");
    assert_eq!(part.parts[0].parts[0].id, "smt.a.1");
}
