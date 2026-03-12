//! Traceability Embedding: inject source provenance metadata into OSCAL elements.
//!
//! Provides constants, helper functions, and embedding functions that annotate
//! OSCAL Catalog controls/groups and Component Definition implemented-requirements
//! with source file, section, and line number trace props and links.

use crate::model::trace::TraceLinkCollection;
use crate::oscal::back_matter::OscalLink;
use crate::oscal::catalog::OscalCatalog;
use crate::oscal::parts::OscalProp;

// ─── Constants (AR-017, SEC-4, SEC-5) ────────────────────────────────────

/// FORGE trace namespace URI for all trace-related props (M-6).
pub const FORGE_TRACE_NS: &str = "https://forge.policy-forge.github.io/ns/trace";

/// Prop name for the source file path (M-1, M-3, M-5).
pub const PROP_SOURCE_FILE: &str = "source-file";

/// Prop name for the source section title (M-1, M-3, S-1, S-2).
pub const PROP_SOURCE_SECTION: &str = "source-section";

/// Prop name for the source line number (M-1, M-3).
pub const PROP_SOURCE_LINE: &str = "source-line";

/// Link rel value for source references (M-2, M-4).
pub const LINK_REL_SOURCE: &str = "source";

// ─── Helper Functions ────────────────────────────────────────────────────

/// Percent-encode special characters in a file path for use in link href.
///
/// Encodes: `%` -> `%25`, ` ` (space) -> `%20`, `#` -> `%23` (per RFC 3986 EC-6).
///
/// Only path-critical characters are encoded because inputs are local file paths
/// (not arbitrary URIs). Characters like `?`, `&`, `=` are not expected in file
/// paths and are left as-is to keep the encoding minimal and predictable.
///
/// `%` is encoded FIRST to avoid double-encoding.
/// Maximum file path length accepted for href encoding (4096 per POSIX `PATH_MAX`).
const MAX_HREF_PATH_LENGTH: usize = 4096;

fn encode_href_path(path: &str) -> String {
    // Guard against excessively long paths (DoS / allocation risk)
    // Use char boundary-safe truncation to avoid panic on non-ASCII UTF-8 paths.
    let safe_path = if path.len() > MAX_HREF_PATH_LENGTH {
        let mut end = MAX_HREF_PATH_LENGTH;
        while end > 0 && !path.is_char_boundary(end) {
            end -= 1;
        }
        &path[..end]
    } else {
        path
    };
    safe_path.replace('%', "%25").replace(' ', "%20").replace('#', "%23")
}

/// Build 3 namespaced trace props for a source location.
///
/// Returns props in order: source-file, source-section, source-line.
/// All props have `ns: Some(FORGE_TRACE_NS)`.
#[must_use]
pub fn build_trace_props(
    source_file: &str,
    section_title: &str,
    line_number: usize,
) -> Vec<OscalProp> {
    vec![
        OscalProp {
            name: PROP_SOURCE_FILE.to_string(),
            ns: Some(FORGE_TRACE_NS.to_string()),
            value: source_file.to_string(),
        },
        OscalProp {
            name: PROP_SOURCE_SECTION.to_string(),
            ns: Some(FORGE_TRACE_NS.to_string()),
            value: section_title.to_string(),
        },
        OscalProp {
            name: PROP_SOURCE_LINE.to_string(),
            ns: Some(FORGE_TRACE_NS.to_string()),
            value: line_number.to_string(),
        },
    ]
}

/// Build 1 source link with href `"<encoded_file>#line=<n>"` (relative URI reference per RFC 3986).
#[must_use]
pub fn build_trace_link(source_file: &str, line_number: usize) -> OscalLink {
    let encoded = encode_href_path(source_file);
    OscalLink {
        href: format!("{encoded}#line={line_number}"),
        rel: LINK_REL_SOURCE.to_string(),
        text: None,
    }
}

// ─── Embedding Functions ─────────────────────────────────────────────────

/// Walk catalog groups and controls, inject trace props/links from `TraceLinkCollection`.
///
/// For each **control**: looks up `trace_links.by_oscal_element(control.uuid)`.
/// If found, appends 3 trace props and 1 source link.
///
/// For each **group**: derives `source-section` from first child control's
/// trace link `section_title` and adds it to `group.props`. Skips groups
/// with no traceable child controls (EC-4).
pub fn embed_trace_in_catalog(catalog: &mut OscalCatalog, trace_links: &TraceLinkCollection) {
    let mut annotated_controls = 0usize;
    let mut annotated_groups = 0usize;

    for group in &mut catalog.groups {
        let mut group_section_title: Option<String> = None;

        for control in &mut group.controls {
            if let Some(trace) = trace_links.by_oscal_element(&control.uuid) {
                let loc = &trace.source_location;
                // SEC-1: Use filename-only to prevent absolute path leakage into
                // OSCAL output (consistent with component pipeline at pipeline.rs:205).
                let file = loc.file_path.file_name().map_or_else(
                    || "unknown-file".to_string(),
                    |f| f.to_string_lossy().into_owned(),
                );

                let props = build_trace_props(&file, &loc.section_title, loc.line_number);
                control.props.extend(props);

                let link = build_trace_link(&file, loc.line_number);
                control.links.push(link);

                annotated_controls += 1;

                if group_section_title.is_none() {
                    group_section_title = Some(loc.section_title.clone());
                }
            } else {
                tracing::debug!(control_uuid = %control.uuid, "No trace link found for control — skipping");
            }
        }

        if let Some(section_title) = group_section_title {
            group.props.push(OscalProp {
                name: PROP_SOURCE_SECTION.to_string(),
                ns: Some(FORGE_TRACE_NS.to_string()),
                value: section_title,
            });
            annotated_groups += 1;
        }
    }

    tracing::debug!(annotated_controls, annotated_groups, "Trace embedding complete for catalog");
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── T006: encode_href_path tests ─────────────────────────────────

    #[test]
    fn encode_percent() {
        assert_eq!(encode_href_path("100%done"), "100%25done");
    }

    #[test]
    fn encode_space() {
        assert_eq!(encode_href_path("my file.md"), "my%20file.md");
    }

    #[test]
    fn encode_hash() {
        assert_eq!(encode_href_path("file#1.md"), "file%231.md");
    }

    #[test]
    fn encode_empty_string() {
        assert_eq!(encode_href_path(""), "");
    }

    #[test]
    fn encode_normal_path_unchanged() {
        assert_eq!(encode_href_path("policies/access-control.md"), "policies/access-control.md");
    }

    #[test]
    fn encode_combined_special_chars() {
        assert_eq!(encode_href_path("my file#1%.md"), "my%20file%231%25.md");
    }

    #[test]
    fn encode_percent_first_avoids_double_encoding() {
        // If we encoded space first, "a b" -> "a%20b", then encoding % -> "a%2520b" (wrong)
        // Correct: encode % first (no-op here), then space -> "a%20b"
        assert_eq!(encode_href_path("a b"), "a%20b");
        // With actual percent: "a%b" -> "a%25b" (% encoded), no further changes
        assert_eq!(encode_href_path("a%b"), "a%25b");
        // With percent AND space: "a% b" -> "a%25 b" (% encoded) -> "a%25%20b" (space encoded)
        assert_eq!(encode_href_path("a% b"), "a%25%20b");
    }

    // ── T007: build_trace_props tests ────────────────────────────────

    #[test]
    fn build_trace_props_returns_three_props() {
        let props = build_trace_props("policy.md", "Access Control", 42);
        assert_eq!(props.len(), 3);
    }

    #[test]
    fn build_trace_props_order() {
        let props = build_trace_props("policy.md", "Access Control", 42);
        assert_eq!(props[0].name, PROP_SOURCE_FILE);
        assert_eq!(props[1].name, PROP_SOURCE_SECTION);
        assert_eq!(props[2].name, PROP_SOURCE_LINE);
    }

    #[test]
    fn build_trace_props_all_have_ns() {
        let props = build_trace_props("policy.md", "Access Control", 42);
        for prop in &props {
            assert_eq!(prop.ns, Some(FORGE_TRACE_NS.to_string()));
        }
    }

    #[test]
    fn build_trace_props_values_match_inputs() {
        let props = build_trace_props("policy.md", "Access Control", 42);
        assert_eq!(props[0].value, "policy.md");
        assert_eq!(props[1].value, "Access Control");
        assert_eq!(props[2].value, "42");
    }

    #[test]
    fn build_trace_props_uses_constants() {
        let props = build_trace_props("f", "s", 1);
        assert_eq!(props[0].name, "source-file");
        assert_eq!(props[1].name, "source-section");
        assert_eq!(props[2].name, "source-line");
    }

    #[test]
    fn build_trace_props_line_number_as_string() {
        let props = build_trace_props("f", "s", 999);
        assert_eq!(props[2].value, "999");
    }

    // ── T008: build_trace_link tests ─────────────────────────────────

    #[test]
    fn build_trace_link_rel_is_source() {
        let link = build_trace_link("policy.md", 42);
        assert_eq!(link.rel, LINK_REL_SOURCE);
    }

    #[test]
    fn build_trace_link_text_is_none() {
        let link = build_trace_link("policy.md", 42);
        assert!(link.text.is_none());
    }

    #[test]
    fn build_trace_link_href_format() {
        let link = build_trace_link("policy.md", 42);
        assert_eq!(link.href, "policy.md#line=42");
    }

    #[test]
    fn build_trace_link_special_chars_encoded() {
        let link = build_trace_link("my file#1%.md", 10);
        assert_eq!(link.href, "my%20file%231%25.md#line=10");
    }

    // ── T010: embed_trace_in_catalog tests ───────────────────────────

    use std::path::PathBuf;

    use crate::model::trace::{SourceLocation, TraceLink, TraceLinkCollection};
    use crate::oscal::catalog::{OscalCatalog, OscalControl, OscalGroup, OscalMetadata};

    fn test_metadata() -> OscalMetadata {
        OscalMetadata {
            title: "Test".to_string(),
            last_modified: "2026-01-01T00:00:00Z".to_string(),
            version: "1.0".to_string(),
            oscal_version: "1.2.0".to_string(),
        }
    }

    fn test_control(id: &str, uuid: &str) -> OscalControl {
        OscalControl {
            id: id.to_string(),
            uuid: uuid.to_string(),
            title: "Test control".to_string(),
            links: vec![],
            params: vec![],
            parts: vec![],
            props: vec![],
        }
    }

    fn test_catalog(groups: Vec<OscalGroup>) -> OscalCatalog {
        OscalCatalog {
            uuid: "cat-uuid".to_string(),
            metadata: test_metadata(),
            controls: vec![],
            groups,
            back_matter: None,
        }
    }

    fn test_trace_link(element_id: &str, file: &str, section: &str, line: usize) -> TraceLink {
        TraceLink {
            requirement_stable_id: format!("req-{element_id}"),
            oscal_json_path: "catalog.groups[0].controls[0]".to_string(),
            oscal_element_id: element_id.to_string(),
            source_location: SourceLocation {
                file_path: PathBuf::from(file),
                section_title: section.to_string(),
                line_number: line,
            },
        }
    }

    #[test]
    fn embed_trace_annotates_controls_with_matching_trace_links() {
        let mut catalog = test_catalog(vec![OscalGroup {
            id: "access-control".to_string(),
            title: "Access Control".to_string(),
            props: vec![],
            links: vec![],
            controls: vec![
                test_control("POL-AC-001", "uuid-1"),
                test_control("POL-AC-002", "uuid-2"),
            ],
        }]);

        let mut tl = TraceLinkCollection::new();
        tl.record(test_trace_link("uuid-1", "policy.md", "Access Control", 10)).unwrap();
        tl.record(test_trace_link("uuid-2", "policy.md", "Access Control", 25)).unwrap();

        embed_trace_in_catalog(&mut catalog, &tl);

        // Each control should have 3 trace props
        for ctrl in &catalog.groups[0].controls {
            assert_eq!(ctrl.props.len(), 3, "Control {} should have 3 trace props", ctrl.id);
            assert_eq!(ctrl.props[0].name, PROP_SOURCE_FILE);
            assert_eq!(ctrl.props[1].name, PROP_SOURCE_SECTION);
            assert_eq!(ctrl.props[2].name, PROP_SOURCE_LINE);
            for prop in &ctrl.props {
                assert_eq!(prop.ns, Some(FORGE_TRACE_NS.to_string()));
            }
        }

        // Each control should have 1 source link
        for ctrl in &catalog.groups[0].controls {
            assert_eq!(ctrl.links.len(), 1, "Control {} should have 1 source link", ctrl.id);
            assert_eq!(ctrl.links[0].rel, LINK_REL_SOURCE);
        }

        // Verify specific values
        assert_eq!(catalog.groups[0].controls[0].props[2].value, "10");
        assert_eq!(catalog.groups[0].controls[1].props[2].value, "25");
        assert_eq!(catalog.groups[0].controls[0].links[0].href, "policy.md#line=10");
        assert_eq!(catalog.groups[0].controls[1].links[0].href, "policy.md#line=25");
    }

    #[test]
    fn embed_trace_skips_controls_without_matching_trace_links() {
        let mut catalog = test_catalog(vec![OscalGroup {
            id: "access-control".to_string(),
            title: "Access Control".to_string(),
            props: vec![],
            links: vec![],
            controls: vec![
                test_control("POL-AC-001", "uuid-1"),
                test_control("POL-AC-002", "uuid-no-match"),
            ],
        }]);

        let mut tl = TraceLinkCollection::new();
        tl.record(test_trace_link("uuid-1", "policy.md", "Access Control", 10)).unwrap();
        // uuid-no-match has no trace link

        embed_trace_in_catalog(&mut catalog, &tl);

        // First control annotated
        assert_eq!(catalog.groups[0].controls[0].props.len(), 3);
        assert_eq!(catalog.groups[0].controls[0].links.len(), 1);

        // Second control NOT annotated
        assert!(catalog.groups[0].controls[1].props.is_empty());
        assert!(catalog.groups[0].controls[1].links.is_empty());
    }

    #[test]
    fn embed_trace_adds_group_source_section_from_first_child() {
        let mut catalog = test_catalog(vec![OscalGroup {
            id: "access-control".to_string(),
            title: "Access Control".to_string(),
            props: vec![],
            links: vec![],
            controls: vec![
                test_control("POL-AC-001", "uuid-1"),
                test_control("POL-AC-002", "uuid-2"),
            ],
        }]);

        let mut tl = TraceLinkCollection::new();
        tl.record(test_trace_link("uuid-1", "policy.md", "3.1 Access Control", 10)).unwrap();
        tl.record(test_trace_link("uuid-2", "policy.md", "3.1 Access Control", 25)).unwrap();

        embed_trace_in_catalog(&mut catalog, &tl);

        // Group should have source-section prop derived from first child
        assert_eq!(catalog.groups[0].props.len(), 1);
        assert_eq!(catalog.groups[0].props[0].name, PROP_SOURCE_SECTION);
        assert_eq!(catalog.groups[0].props[0].ns, Some(FORGE_TRACE_NS.to_string()));
        assert_eq!(catalog.groups[0].props[0].value, "3.1 Access Control");
    }

    #[test]
    fn embed_trace_skips_group_prop_when_no_traceable_children() {
        let mut catalog = test_catalog(vec![OscalGroup {
            id: "empty-group".to_string(),
            title: "Empty Group".to_string(),
            props: vec![],
            links: vec![],
            controls: vec![test_control("POL-EG-001", "uuid-no-match")],
        }]);

        let tl = TraceLinkCollection::new(); // empty — no trace links

        embed_trace_in_catalog(&mut catalog, &tl);

        // Group should have NO props (EC-4)
        assert!(catalog.groups[0].props.is_empty());
    }

    #[test]
    fn embed_trace_handles_empty_catalog() {
        let mut catalog = test_catalog(vec![]);
        let tl = TraceLinkCollection::new();
        embed_trace_in_catalog(&mut catalog, &tl);
        assert!(catalog.groups.is_empty());
    }

    // ── T021: No trace data in remarks (Catalog) — SEC-1, SEC-2, M-7 ──

    use crate::oscal::test_utils::collect_remarks;

    #[test]
    fn catalog_output_has_no_trace_data_in_remarks() {
        // Build a catalog with trace embedding applied, serialize, check remarks
        let mut catalog = test_catalog(vec![OscalGroup {
            id: "ac".to_string(),
            title: "Access Control".to_string(),
            props: vec![],
            links: vec![],
            controls: vec![test_control("POL-AC-001", "uuid-1")],
        }]);

        let mut tl = TraceLinkCollection::new();
        tl.record(test_trace_link("uuid-1", "policies/security.md", "Access Control", 42)).unwrap();
        embed_trace_in_catalog(&mut catalog, &tl);

        let json_str = serde_json::to_string_pretty(&catalog).unwrap();
        let json_val: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        let mut remarks = Vec::new();
        collect_remarks(&json_val, &mut remarks);

        let trace_patterns = ["policies/security.md", "Access Control", "42", "source-file"];
        for remark in &remarks {
            for pattern in &trace_patterns {
                assert!(
                    !remark.contains(pattern),
                    "Remarks must not contain trace data '{pattern}'. Found in: {remark}"
                );
            }
        }
    }

    #[test]
    fn embed_trace_multiple_groups() {
        let mut catalog = test_catalog(vec![
            OscalGroup {
                id: "ac".to_string(),
                title: "Access Control".to_string(),
                props: vec![],
                links: vec![],
                controls: vec![test_control("POL-AC-001", "uuid-ac1")],
            },
            OscalGroup {
                id: "dp".to_string(),
                title: "Data Protection".to_string(),
                props: vec![],
                links: vec![],
                controls: vec![test_control("POL-DP-001", "uuid-dp1")],
            },
        ]);

        let mut tl = TraceLinkCollection::new();
        tl.record(test_trace_link("uuid-ac1", "policy.md", "Access Control", 10)).unwrap();
        tl.record(test_trace_link("uuid-dp1", "policy.md", "Data Protection", 50)).unwrap();

        embed_trace_in_catalog(&mut catalog, &tl);

        // Both groups annotated
        assert_eq!(catalog.groups[0].controls[0].props.len(), 3);
        assert_eq!(catalog.groups[1].controls[0].props.len(), 3);
        assert_eq!(catalog.groups[0].props[0].value, "Access Control");
        assert_eq!(catalog.groups[1].props[0].value, "Data Protection");
    }
}
