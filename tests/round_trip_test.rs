//! Round-trip fidelity integration tests for OSCAL multi-format conversion.
//!
//! Tests verify that OSCAL artifacts survive conversion between JSON, XML, and
//! YAML without data loss by comparing `serde_json::Value` trees using semantic
//! equivalence.
//!
//! PRD Requirements: M-1, M-2, M-3, M-4, M-5, M-6, M-8, S-1, S-2

use forge::export::xml_deserializer::{
    deserialize_catalog_from_xml, deserialize_component_from_xml,
};
use forge::export::xml_serializer::{
    serialize_catalog_to_xml, serialize_component_definition_to_xml,
};
use forge::export::yaml::{deserialize_from_yaml, serialize_to_yaml};
use forge::oscal::catalog::CatalogEnvelope;
use forge::oscal::component_definition::ComponentDefinitionEnvelope;
use forge::testing::assert_semantic_equivalence;

// ─── Fixture Loading ────────────────────────────────────────────────────

fn load_fixture(relative_path: &str) -> String {
    let path = format!("tests/fixtures/golden/{relative_path}");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to load fixture {path}: {e}"))
}

// ─── Normalization Helpers ──────────────────────────────────────────────

/// Create a normalized copy of a `ComponentDefinitionEnvelope` with
/// `control_implementations` cleared on each component. XML serialization
/// intentionally skips this field, so the original must be normalized before
/// comparison.
fn normalize_component_envelope(
    envelope: &ComponentDefinitionEnvelope,
) -> ComponentDefinitionEnvelope {
    use forge::oscal::component_definition::{ComponentDefinition, DocumentaryComponent};

    let components = envelope
        .component_definition
        .components
        .iter()
        .map(|comp| DocumentaryComponent { control_implementations: vec![], ..comp.clone() })
        .collect();

    ComponentDefinitionEnvelope {
        component_definition: ComponentDefinition {
            components,
            ..envelope.component_definition.clone()
        },
    }
}

// ─── Round-Trip Helpers ─────────────────────────────────────────────────

/// JSON → YAML → JSON round-trip for Catalog (T048).
///
/// Returns `(original_normalized, round_tripped)` as `serde_json::Value` pairs.
fn round_trip_catalog_json_yaml_json(json_str: &str) -> (serde_json::Value, serde_json::Value) {
    let envelope: CatalogEnvelope =
        serde_json::from_str(json_str).expect("deserialize fixture JSON");
    let original = serde_json::to_value(&envelope).expect("normalize original");

    let yaml = serialize_to_yaml(&envelope).expect("serialize to YAML");
    let rt_envelope: CatalogEnvelope = deserialize_from_yaml(&yaml).expect("deserialize from YAML");
    let round_tripped = serde_json::to_value(&rt_envelope).expect("serialize round-tripped");

    (original, round_tripped)
}

/// JSON → YAML → JSON round-trip for Component Definition (T049).
fn round_trip_component_json_yaml_json(json_str: &str) -> (serde_json::Value, serde_json::Value) {
    let envelope: ComponentDefinitionEnvelope =
        serde_json::from_str(json_str).expect("deserialize fixture JSON");
    let original = serde_json::to_value(&envelope).expect("normalize original");

    let yaml = serialize_to_yaml(&envelope).expect("serialize to YAML");
    let rt_envelope: ComponentDefinitionEnvelope =
        deserialize_from_yaml(&yaml).expect("deserialize from YAML");
    let round_tripped = serde_json::to_value(&rt_envelope).expect("serialize round-tripped");

    (original, round_tripped)
}

/// JSON → XML → JSON round-trip for Catalog (T060).
fn round_trip_catalog_json_xml_json(json_str: &str) -> (serde_json::Value, serde_json::Value) {
    let envelope: CatalogEnvelope =
        serde_json::from_str(json_str).expect("deserialize fixture JSON");
    let original = serde_json::to_value(&envelope).expect("normalize original");

    let xml = serialize_catalog_to_xml(&envelope.catalog).expect("serialize to XML");
    let rt_envelope = deserialize_catalog_from_xml(&xml).expect("deserialize from XML");
    let round_tripped = serde_json::to_value(&rt_envelope).expect("serialize round-tripped");

    (original, round_tripped)
}

/// JSON → XML → JSON round-trip for Component Definition (T061).
///
/// Normalizes `control-implementations` on the original since XML
/// serialization intentionally skips this field.
fn round_trip_component_json_xml_json(json_str: &str) -> (serde_json::Value, serde_json::Value) {
    let envelope: ComponentDefinitionEnvelope =
        serde_json::from_str(json_str).expect("deserialize fixture JSON");

    // Normalize: XML serializer skips control-implementations (immutable copy)
    let normalized = normalize_component_envelope(&envelope);
    let original = serde_json::to_value(&normalized).expect("normalize original");

    let xml = serialize_component_definition_to_xml(&normalized.component_definition)
        .expect("serialize to XML");
    let rt_envelope = deserialize_component_from_xml(&xml).expect("deserialize from XML");
    let round_tripped = serde_json::to_value(&rt_envelope).expect("serialize round-tripped");

    (original, round_tripped)
}

/// XML → YAML → XML round-trip for Catalog (T066).
///
/// Normalizes through JSON Value for comparison since XML string
/// comparison is not meaningful (whitespace, attribute order).
fn round_trip_catalog_xml_yaml_xml(json_str: &str) -> (serde_json::Value, serde_json::Value) {
    // Start: JSON → model → XML (establish starting XML)
    let envelope: CatalogEnvelope =
        serde_json::from_str(json_str).expect("deserialize fixture JSON");
    let starting_xml =
        serialize_catalog_to_xml(&envelope.catalog).expect("serialize to starting XML");

    // XML → model (original for comparison, normalized through XML)
    let xml_envelope =
        deserialize_catalog_from_xml(&starting_xml).expect("deserialize starting XML");
    let original = serde_json::to_value(&xml_envelope).expect("normalize original via XML model");

    // XML → model → YAML → model → XML → model (round-trip)
    let yaml = serialize_to_yaml(&xml_envelope).expect("serialize XML model to YAML");
    let yaml_envelope: CatalogEnvelope =
        deserialize_from_yaml(&yaml).expect("deserialize from YAML");
    let rt_xml = serialize_catalog_to_xml(&yaml_envelope.catalog).expect("serialize back to XML");
    let rt_envelope = deserialize_catalog_from_xml(&rt_xml).expect("deserialize round-tripped XML");
    let round_tripped = serde_json::to_value(&rt_envelope).expect("serialize round-tripped");

    (original, round_tripped)
}

/// XML → YAML → XML round-trip for Component Definition (T067).
fn round_trip_component_xml_yaml_xml(json_str: &str) -> (serde_json::Value, serde_json::Value) {
    let envelope: ComponentDefinitionEnvelope =
        serde_json::from_str(json_str).expect("deserialize fixture JSON");
    // Normalize: XML serializer skips control-implementations (immutable copy)
    let normalized = normalize_component_envelope(&envelope);
    let starting_xml = serialize_component_definition_to_xml(&normalized.component_definition)
        .expect("serialize to starting XML");

    let xml_envelope =
        deserialize_component_from_xml(&starting_xml).expect("deserialize starting XML");
    let original = serde_json::to_value(&xml_envelope).expect("normalize original via XML model");

    let yaml = serialize_to_yaml(&xml_envelope).expect("serialize XML model to YAML");
    let yaml_envelope: ComponentDefinitionEnvelope =
        deserialize_from_yaml(&yaml).expect("deserialize from YAML");
    let rt_xml = serialize_component_definition_to_xml(&yaml_envelope.component_definition)
        .expect("serialize back to XML");
    let rt_envelope =
        deserialize_component_from_xml(&rt_xml).expect("deserialize round-tripped XML");
    let round_tripped = serde_json::to_value(&rt_envelope).expect("serialize round-tripped");

    (original, round_tripped)
}

// ─── Assertion Helper ───────────────────────────────────────────────────

fn assert_round_trip_equivalent(original: &serde_json::Value, round_tripped: &serde_json::Value) {
    let result = assert_semantic_equivalence(original, round_tripped);
    if !result.is_equivalent {
        let diff_report: Vec<String> = result
            .differences
            .iter()
            .map(|d| {
                format!(
                    "  {} — {} (expected: {:?}, actual: {:?})",
                    d.path, d.description, d.expected, d.actual
                )
            })
            .collect();
        panic!(
            "Round-trip fidelity failure: {} differences found:\n{}",
            result.differences.len(),
            diff_report.join("\n")
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Phase 3: JSON → YAML → JSON Round-Trip (US2) — PRD M-2, M-4, M-8, S-2
// ═══════════════════════════════════════════════════════════════════════

mod yaml_round_trip {
    use super::*;

    // ── Catalog: JSON → YAML → JSON ────────────────────────

    // T035
    #[test]
    fn test_catalog_json_yaml_json_small() {
        let json = load_fixture("small/expected-catalog.json");
        let (original, round_tripped) = round_trip_catalog_json_yaml_json(&json);
        assert_round_trip_equivalent(&original, &round_tripped);
    }

    // T036
    #[test]
    fn test_catalog_json_yaml_json_medium() {
        let json = load_fixture("medium/expected-catalog.json");
        let (original, round_tripped) = round_trip_catalog_json_yaml_json(&json);
        assert_round_trip_equivalent(&original, &round_tripped);
    }

    // T037 (PRD S-4: 50+ controls)
    #[test]
    fn test_catalog_json_yaml_json_complex() {
        let json = load_fixture("complex/expected-catalog.json");
        let (original, round_tripped) = round_trip_catalog_json_yaml_json(&json);
        assert_round_trip_equivalent(&original, &round_tripped);
    }

    // ── Component Definition: JSON → YAML → JSON ───────────

    // T038
    #[test]
    fn test_component_json_yaml_json_small() {
        let json = load_fixture("small/expected-component-definition.json");
        let (original, round_tripped) = round_trip_component_json_yaml_json(&json);
        assert_round_trip_equivalent(&original, &round_tripped);
    }

    // T039
    #[test]
    fn test_component_json_yaml_json_medium() {
        let json = load_fixture("medium/expected-component-definition.json");
        let (original, round_tripped) = round_trip_component_json_yaml_json(&json);
        assert_round_trip_equivalent(&original, &round_tripped);
    }

    // T040
    #[test]
    fn test_component_json_yaml_json_complex() {
        let json = load_fixture("complex/expected-component-definition.json");
        let (original, round_tripped) = round_trip_component_json_yaml_json(&json);
        assert_round_trip_equivalent(&original, &round_tripped);
    }

    // ── YAML Type Coercion Edge Cases (PRD M-8, S-2) ───────

    // T041: Boolean-like strings must remain strings (PRD M-8, EC-6)
    #[test]
    fn test_yaml_preserves_boolean_like_strings() {
        let envelope = make_catalog_with_prop_values(&[
            "true", "false", "yes", "no", "on", "off", "True", "False", "YES", "NO",
        ]);
        let original = serde_json::to_value(&envelope).unwrap();
        let yaml = serialize_to_yaml(&envelope).unwrap();
        let rt: CatalogEnvelope = deserialize_from_yaml(&yaml).unwrap();
        let round_tripped = serde_json::to_value(&rt).unwrap();
        assert_round_trip_equivalent(&original, &round_tripped);
    }

    // T042: Numeric strings must remain strings (PRD M-8, EC-7)
    #[test]
    fn test_yaml_preserves_numeric_strings() {
        let envelope = make_catalog_with_prop_values(&["10", "3.14", "1.0", "0", "007", "1e5"]);
        let original = serde_json::to_value(&envelope).unwrap();
        let yaml = serialize_to_yaml(&envelope).unwrap();
        let rt: CatalogEnvelope = deserialize_from_yaml(&yaml).unwrap();
        let round_tripped = serde_json::to_value(&rt).unwrap();
        assert_round_trip_equivalent(&original, &round_tripped);
    }

    // T043: Null-like strings must remain strings
    #[test]
    fn test_yaml_preserves_null_like_strings() {
        let envelope = make_catalog_with_prop_values(&["null", "Null", "NULL", "~"]);
        let original = serde_json::to_value(&envelope).unwrap();
        let yaml = serialize_to_yaml(&envelope).unwrap();
        let rt: CatalogEnvelope = deserialize_from_yaml(&yaml).unwrap();
        let round_tripped = serde_json::to_value(&rt).unwrap();
        assert_round_trip_equivalent(&original, &round_tripped);
    }

    // T044: ISO 8601 timestamp strings remain strings (EC-3)
    #[test]
    fn test_yaml_preserves_timestamp_strings() {
        let envelope =
            make_catalog_with_prop_values(&["2026-09-08T10:00:00Z", "2026-01-01", "12:30:00"]);
        let original = serde_json::to_value(&envelope).unwrap();
        let yaml = serialize_to_yaml(&envelope).unwrap();
        let rt: CatalogEnvelope = deserialize_from_yaml(&yaml).unwrap();
        let round_tripped = serde_json::to_value(&rt).unwrap();
        assert_round_trip_equivalent(&original, &round_tripped);
    }

    // T045: UUID strings remain strings (EC-4)
    #[test]
    fn test_yaml_preserves_uuid_strings() {
        let envelope = make_catalog_with_prop_values(&[
            "550e8400-e29b-41d4-a716-446655440000",
            "00000000-0000-0000-0000-000000000000",
        ]);
        let original = serde_json::to_value(&envelope).unwrap();
        let yaml = serialize_to_yaml(&envelope).unwrap();
        let rt: CatalogEnvelope = deserialize_from_yaml(&yaml).unwrap();
        let round_tripped = serde_json::to_value(&rt).unwrap();
        assert_round_trip_equivalent(&original, &round_tripped);
    }

    // T046: Empty arrays survive YAML round-trip (EC-2)
    #[test]
    fn test_yaml_preserves_empty_arrays() {
        use forge::oscal::catalog::{OscalCatalog, OscalGroup, OscalMetadata};

        let envelope = CatalogEnvelope {
            catalog: OscalCatalog {
                uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
                metadata: OscalMetadata {
                    title: "Empty arrays test".to_string(),
                    last_modified: "2026-02-15T12:00:00Z".to_string(),
                    version: "1.0".to_string(),
                    oscal_version: "1.2.0".to_string(),
                },
                controls: vec![],
                groups: vec![OscalGroup {
                    id: "test".to_string(),
                    title: "Test".to_string(),
                    props: vec![],
                    links: vec![],
                    controls: vec![], // empty array
                    groups: vec![],
                }],
                back_matter: None,
            },
        };
        let original = serde_json::to_value(&envelope).unwrap();
        let yaml = serialize_to_yaml(&envelope).unwrap();
        let rt: CatalogEnvelope = deserialize_from_yaml(&yaml).unwrap();
        let round_tripped = serde_json::to_value(&rt).unwrap();
        assert_round_trip_equivalent(&original, &round_tripped);
    }

    // T047: Deeply nested objects (5+ levels) survive YAML round-trip (EC-5)
    #[test]
    fn test_yaml_preserves_deeply_nested() {
        use forge::oscal::catalog::{OscalCatalog, OscalControl, OscalGroup, OscalMetadata};
        use forge::oscal::parts::OscalPart;

        let envelope = CatalogEnvelope {
            catalog: OscalCatalog {
                uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
                metadata: OscalMetadata {
                    title: "Deep nesting test".to_string(),
                    last_modified: "2026-02-15T12:00:00Z".to_string(),
                    version: "1.0".to_string(),
                    oscal_version: "1.2.0".to_string(),
                },
                controls: vec![],
                groups: vec![OscalGroup {
                    id: "deep".to_string(),
                    title: "Deep".to_string(),
                    props: vec![],
                    links: vec![],
                    controls: vec![OscalControl {
                        id: "POL-D-001".to_string(),
                        uuid: String::new(),
                        title: "Deep control".to_string(),
                        links: vec![],
                        params: vec![],
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
                                    parts: vec![OscalPart {
                                        id: "smt.a.1.i".to_string(),
                                        name: "item".to_string(),
                                        prose: "Level 4".to_string(),
                                        props: vec![],
                                        parts: vec![OscalPart {
                                            id: "smt.a.1.i.A".to_string(),
                                            name: "item".to_string(),
                                            prose: "Level 5".to_string(),
                                            props: vec![],
                                            parts: vec![],
                                        }],
                                    }],
                                }],
                            }],
                        }],
                        props: vec![],
                    }],
                    groups: vec![],
                }],
                back_matter: None,
            },
        };
        let original = serde_json::to_value(&envelope).unwrap();
        let yaml = serialize_to_yaml(&envelope).unwrap();
        let rt: CatalogEnvelope = deserialize_from_yaml(&yaml).unwrap();
        let round_tripped = serde_json::to_value(&rt).unwrap();
        assert_round_trip_equivalent(&original, &round_tripped);
    }

    // T075: Array ordering preserved through YAML round-trip (PRD M-6)
    #[test]
    fn test_yaml_preserves_array_ordering() {
        use forge::oscal::catalog::{OscalCatalog, OscalControl, OscalGroup, OscalMetadata};
        use forge::oscal::parts::OscalPart;

        let envelope = CatalogEnvelope {
            catalog: OscalCatalog {
                uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
                metadata: OscalMetadata {
                    title: "Array ordering test".to_string(),
                    last_modified: "2026-02-15T12:00:00Z".to_string(),
                    version: "1.0".to_string(),
                    oscal_version: "1.2.0".to_string(),
                },
                controls: vec![],
                groups: vec![OscalGroup {
                    id: "ac".to_string(),
                    title: "Access Control".to_string(),
                    props: vec![],
                    links: vec![],
                    groups: vec![],
                    controls: vec![
                        OscalControl {
                            id: "POL-AC-001".to_string(),
                            uuid: String::new(),
                            title: "First control".to_string(),
                            links: vec![],
                            params: vec![],
                            parts: vec![OscalPart {
                                id: "POL-AC-001_smt".to_string(),
                                name: "statement".to_string(),
                                prose: "First".to_string(),
                                props: vec![],
                                parts: vec![],
                            }],
                            props: vec![],
                        },
                        OscalControl {
                            id: "POL-AC-002".to_string(),
                            uuid: String::new(),
                            title: "Second control".to_string(),
                            links: vec![],
                            params: vec![],
                            parts: vec![OscalPart {
                                id: "POL-AC-002_smt".to_string(),
                                name: "statement".to_string(),
                                prose: "Second".to_string(),
                                props: vec![],
                                parts: vec![],
                            }],
                            props: vec![],
                        },
                        OscalControl {
                            id: "POL-AC-003".to_string(),
                            uuid: String::new(),
                            title: "Third control".to_string(),
                            links: vec![],
                            params: vec![],
                            parts: vec![OscalPart {
                                id: "POL-AC-003_smt".to_string(),
                                name: "statement".to_string(),
                                prose: "Third".to_string(),
                                props: vec![],
                                parts: vec![],
                            }],
                            props: vec![],
                        },
                    ],
                }],
                back_matter: None,
            },
        };
        let original = serde_json::to_value(&envelope).unwrap();
        let yaml = serialize_to_yaml(&envelope).unwrap();
        let rt: CatalogEnvelope = deserialize_from_yaml(&yaml).unwrap();
        let round_tripped = serde_json::to_value(&rt).unwrap();
        assert_round_trip_equivalent(&original, &round_tripped);

        // Explicitly verify control ordering
        assert_eq!(rt.catalog.groups[0].controls[0].id, "POL-AC-001");
        assert_eq!(rt.catalog.groups[0].controls[1].id, "POL-AC-002");
        assert_eq!(rt.catalog.groups[0].controls[2].id, "POL-AC-003");
    }

    // ── Edge-Case Helper ────────────────────────────────────

    /// Build a catalog with props containing the given values for testing
    /// YAML type coercion.
    fn make_catalog_with_prop_values(values: &[&str]) -> CatalogEnvelope {
        use forge::oscal::catalog::{OscalCatalog, OscalControl, OscalGroup, OscalMetadata};
        use forge::oscal::parts::{OscalPart, OscalProp};

        let props: Vec<OscalProp> = values
            .iter()
            .enumerate()
            .map(|(i, v)| OscalProp {
                name: format!("test-prop-{i}"),
                value: (*v).to_string(),
                ns: None,
            })
            .collect();

        CatalogEnvelope {
            catalog: OscalCatalog {
                uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
                metadata: OscalMetadata {
                    title: "Type coercion test".to_string(),
                    last_modified: "2026-02-15T12:00:00Z".to_string(),
                    version: "1.0".to_string(),
                    oscal_version: "1.2.0".to_string(),
                },
                controls: vec![],
                groups: vec![OscalGroup {
                    id: "test".to_string(),
                    title: "Test".to_string(),
                    props: vec![],
                    links: vec![],
                    controls: vec![OscalControl {
                        id: "POL-T-001".to_string(),
                        uuid: String::new(),
                        title: "Test control".to_string(),
                        links: vec![],
                        params: vec![],
                        parts: vec![OscalPart {
                            id: "POL-T-001_smt".to_string(),
                            name: "statement".to_string(),
                            prose: "Test".to_string(),
                            props,
                            parts: vec![],
                        }],
                        props: vec![],
                    }],
                    groups: vec![],
                }],
                back_matter: None,
            },
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Phase 4: JSON → XML → JSON Round-Trip (US1) — PRD M-1, M-3, M-5, M-6
// ═══════════════════════════════════════════════════════════════════════

mod xml_round_trip {
    use super::*;

    // ── Catalog: JSON → XML → JSON ─────────────────────────

    // T052
    #[test]
    fn test_catalog_json_xml_json_small() {
        let json = load_fixture("small/expected-catalog.json");
        let (original, round_tripped) = round_trip_catalog_json_xml_json(&json);
        assert_round_trip_equivalent(&original, &round_tripped);
    }

    // T053
    #[test]
    fn test_catalog_json_xml_json_medium() {
        let json = load_fixture("medium/expected-catalog.json");
        let (original, round_tripped) = round_trip_catalog_json_xml_json(&json);
        assert_round_trip_equivalent(&original, &round_tripped);
    }

    // T054 (PRD S-4: 50+ controls)
    #[test]
    fn test_catalog_json_xml_json_complex() {
        let json = load_fixture("complex/expected-catalog.json");
        let (original, round_tripped) = round_trip_catalog_json_xml_json(&json);
        assert_round_trip_equivalent(&original, &round_tripped);
    }

    // ── Component Definition: JSON → XML → JSON ────────────

    // T055
    #[test]
    fn test_component_json_xml_json_small() {
        let json = load_fixture("small/expected-component-definition.json");
        let (original, round_tripped) = round_trip_component_json_xml_json(&json);
        assert_round_trip_equivalent(&original, &round_tripped);
    }

    // T056
    #[test]
    fn test_component_json_xml_json_medium() {
        let json = load_fixture("medium/expected-component-definition.json");
        let (original, round_tripped) = round_trip_component_json_xml_json(&json);
        assert_round_trip_equivalent(&original, &round_tripped);
    }

    // T057
    #[test]
    fn test_component_json_xml_json_complex() {
        let json = load_fixture("complex/expected-component-definition.json");
        let (original, round_tripped) = round_trip_component_json_xml_json(&json);
        assert_round_trip_equivalent(&original, &round_tripped);
    }

    // ── XML-Specific Edge Cases ─────────────────────────────

    // T058: Array ordering preserved through XML round-trip (PRD M-6)
    #[test]
    fn test_xml_preserves_array_ordering() {
        use forge::oscal::catalog::{OscalCatalog, OscalControl, OscalGroup, OscalMetadata};
        use forge::oscal::parts::OscalPart;

        let envelope = CatalogEnvelope {
            catalog: OscalCatalog {
                uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
                metadata: OscalMetadata {
                    title: "XML array ordering test".to_string(),
                    last_modified: "2026-02-15T12:00:00Z".to_string(),
                    version: "1.0".to_string(),
                    oscal_version: "1.2.0".to_string(),
                },
                controls: vec![],
                groups: vec![OscalGroup {
                    id: "ac".to_string(),
                    title: "Access Control".to_string(),
                    props: vec![],
                    links: vec![],
                    groups: vec![],
                    controls: vec![
                        OscalControl {
                            id: "POL-AC-001".to_string(),
                            uuid: String::new(),
                            title: "First".to_string(),
                            links: vec![],
                            params: vec![],
                            parts: vec![OscalPart {
                                id: "smt1".to_string(),
                                name: "statement".to_string(),
                                prose: "First".to_string(),
                                props: vec![],
                                parts: vec![],
                            }],
                            props: vec![],
                        },
                        OscalControl {
                            id: "POL-AC-002".to_string(),
                            uuid: String::new(),
                            title: "Second".to_string(),
                            links: vec![],
                            params: vec![],
                            parts: vec![OscalPart {
                                id: "smt2".to_string(),
                                name: "statement".to_string(),
                                prose: "Second".to_string(),
                                props: vec![],
                                parts: vec![],
                            }],
                            props: vec![],
                        },
                        OscalControl {
                            id: "POL-AC-003".to_string(),
                            uuid: String::new(),
                            title: "Third".to_string(),
                            links: vec![],
                            params: vec![],
                            parts: vec![OscalPart {
                                id: "smt3".to_string(),
                                name: "statement".to_string(),
                                prose: "Third".to_string(),
                                props: vec![],
                                parts: vec![],
                            }],
                            props: vec![],
                        },
                    ],
                }],
                back_matter: None,
            },
        };
        let original = serde_json::to_value(&envelope).unwrap();
        let xml = serialize_catalog_to_xml(&envelope.catalog).unwrap();
        let rt = deserialize_catalog_from_xml(&xml).unwrap();
        let round_tripped = serde_json::to_value(&rt).unwrap();
        assert_round_trip_equivalent(&original, &round_tripped);

        // Explicitly verify control ordering
        assert_eq!(rt.catalog.groups[0].controls[0].id, "POL-AC-001");
        assert_eq!(rt.catalog.groups[0].controls[1].id, "POL-AC-002");
        assert_eq!(rt.catalog.groups[0].controls[2].id, "POL-AC-003");
    }

    // T059: Deeply nested objects (5+ levels) survive XML round-trip (EC-5)
    #[test]
    fn test_xml_preserves_deeply_nested() {
        use forge::oscal::catalog::{OscalCatalog, OscalControl, OscalGroup, OscalMetadata};
        use forge::oscal::parts::OscalPart;

        let envelope = CatalogEnvelope {
            catalog: OscalCatalog {
                uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
                metadata: OscalMetadata {
                    title: "XML deep nesting test".to_string(),
                    last_modified: "2026-02-15T12:00:00Z".to_string(),
                    version: "1.0".to_string(),
                    oscal_version: "1.2.0".to_string(),
                },
                controls: vec![],
                groups: vec![OscalGroup {
                    id: "deep".to_string(),
                    title: "Deep".to_string(),
                    props: vec![],
                    links: vec![],
                    controls: vec![OscalControl {
                        id: "POL-D-001".to_string(),
                        uuid: String::new(),
                        title: "Deep control".to_string(),
                        links: vec![],
                        params: vec![],
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
                                    parts: vec![OscalPart {
                                        id: "smt.a.1.i".to_string(),
                                        name: "item".to_string(),
                                        prose: "Level 4".to_string(),
                                        props: vec![],
                                        parts: vec![OscalPart {
                                            id: "smt.a.1.i.A".to_string(),
                                            name: "item".to_string(),
                                            prose: "Level 5".to_string(),
                                            props: vec![],
                                            parts: vec![],
                                        }],
                                    }],
                                }],
                            }],
                        }],
                        props: vec![],
                    }],
                    groups: vec![],
                }],
                back_matter: None,
            },
        };
        let original = serde_json::to_value(&envelope).unwrap();
        let xml = serialize_catalog_to_xml(&envelope.catalog).unwrap();
        let rt = deserialize_catalog_from_xml(&xml).unwrap();
        let round_tripped = serde_json::to_value(&rt).unwrap();
        assert_round_trip_equivalent(&original, &round_tripped);

        // Verify deepest level preserved (5 levels of nesting)
        let deep_part =
            &rt.catalog.groups[0].controls[0].parts[0].parts[0].parts[0].parts[0].parts[0];
        assert_eq!(deep_part.id, "smt.a.1.i.A");
        assert_eq!(deep_part.prose, "Level 5");
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Phase 5: XML → YAML → XML Round-Trip (US3) — PRD S-1
// ═══════════════════════════════════════════════════════════════════════

mod xml_yaml_xml_round_trip {
    use super::*;

    // T063
    #[test]
    fn test_catalog_xml_yaml_xml_small() {
        let json = load_fixture("small/expected-catalog.json");
        let (original, round_tripped) = round_trip_catalog_xml_yaml_xml(&json);
        assert_round_trip_equivalent(&original, &round_tripped);
    }

    // T064
    #[test]
    fn test_catalog_xml_yaml_xml_medium() {
        let json = load_fixture("medium/expected-catalog.json");
        let (original, round_tripped) = round_trip_catalog_xml_yaml_xml(&json);
        assert_round_trip_equivalent(&original, &round_tripped);
    }

    // T076 (PRD S-4: large fixture)
    #[test]
    fn test_catalog_xml_yaml_xml_complex() {
        let json = load_fixture("complex/expected-catalog.json");
        let (original, round_tripped) = round_trip_catalog_xml_yaml_xml(&json);
        assert_round_trip_equivalent(&original, &round_tripped);
    }

    // T065
    #[test]
    fn test_component_xml_yaml_xml_small() {
        let json = load_fixture("small/expected-component-definition.json");
        let (original, round_tripped) = round_trip_component_xml_yaml_xml(&json);
        assert_round_trip_equivalent(&original, &round_tripped);
    }

    // T077: OSCAL namespace preserved through XML → YAML → XML (PRD US3-AC2)
    #[test]
    fn test_xml_yaml_xml_preserves_namespace() {
        use forge::oscal::catalog::{OscalCatalog, OscalControl, OscalGroup, OscalMetadata};
        use forge::oscal::parts::OscalPart;

        let envelope = CatalogEnvelope {
            catalog: OscalCatalog {
                uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
                metadata: OscalMetadata {
                    title: "Namespace test".to_string(),
                    last_modified: "2026-02-15T12:00:00Z".to_string(),
                    version: "1.0".to_string(),
                    oscal_version: "1.2.0".to_string(),
                },
                controls: vec![],
                groups: vec![OscalGroup {
                    id: "ns-test".to_string(),
                    title: "Namespace Test".to_string(),
                    props: vec![],
                    links: vec![],
                    groups: vec![],
                    controls: vec![OscalControl {
                        id: "POL-NS-001".to_string(),
                        uuid: String::new(),
                        title: "Namespace control".to_string(),
                        links: vec![],
                        params: vec![],
                        parts: vec![OscalPart {
                            id: "smt".to_string(),
                            name: "statement".to_string(),
                            prose: "Namespace test".to_string(),
                            props: vec![],
                            parts: vec![],
                        }],
                        props: vec![],
                    }],
                }],
                back_matter: None,
            },
        };

        // Serialize to XML (starting point)
        let starting_xml = serialize_catalog_to_xml(&envelope.catalog).unwrap();
        assert!(
            starting_xml.contains(r#"xmlns="http://csrc.nist.gov/ns/oscal/1.0""#),
            "Starting XML must have OSCAL namespace"
        );

        // Round-trip: XML → model → YAML → model → XML
        let xml_model = deserialize_catalog_from_xml(&starting_xml).unwrap();
        let yaml = serialize_to_yaml(&xml_model).unwrap();
        let yaml_model: CatalogEnvelope = deserialize_from_yaml(&yaml).unwrap();
        let rt_xml = serialize_catalog_to_xml(&yaml_model.catalog).unwrap();

        // Verify namespace is still present
        assert!(
            rt_xml.contains(r#"xmlns="http://csrc.nist.gov/ns/oscal/1.0""#),
            "Round-tripped XML must preserve OSCAL namespace"
        );

        // Verify semantic equivalence via model normalization
        let original = serde_json::to_value(&xml_model).unwrap();
        let rt_model = deserialize_catalog_from_xml(&rt_xml).unwrap();
        let round_tripped = serde_json::to_value(&rt_model).unwrap();
        assert_round_trip_equivalent(&original, &round_tripped);
    }
}
