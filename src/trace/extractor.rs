use crate::oscal::trace_embedding::{
    FORGE_TRACE_NS, PROP_SOURCE_FILE, PROP_SOURCE_LINE, PROP_SOURCE_SECTION,
};

use super::report::TraceMetadata;

/// Extract trace metadata from an OSCAL element's `props` array (`serde_json::Value`).
///
/// Scans `element["props"]` for props with `ns == FORGE_TRACE_NS`.
/// Returns `Some(TraceMetadata)` if at least `source-section` is found.
/// Returns `None` if no trace props exist.
///
/// Groups may have trace metadata without a concrete source line.
#[must_use]
pub fn extract_trace_metadata(element: &serde_json::Value) -> Option<TraceMetadata> {
    let props = element.get("props")?.as_array()?;

    let mut source_file = None;
    let mut source_section = None;
    let mut source_line = None;

    for prop in props {
        if prop.get("ns").and_then(serde_json::Value::as_str) != Some(FORGE_TRACE_NS) {
            continue;
        }
        let (Some(name), Some(value)) = (
            prop.get("name").and_then(serde_json::Value::as_str),
            prop.get("value").and_then(serde_json::Value::as_str),
        ) else {
            continue;
        };

        match name {
            PROP_SOURCE_FILE if source_file.is_none() => source_file = Some(value.to_string()),
            PROP_SOURCE_SECTION if source_section.is_none() => {
                source_section = Some(value.to_string());
            }
            PROP_SOURCE_LINE if source_line.is_none() => {
                source_line = value.parse::<usize>().ok().filter(|&line| line > 0);
            }
            _ => {}
        }
    }

    // Must have at least source-section to be considered mapped.
    let source_section = source_section.filter(|section| !section.is_empty())?;

    Some(TraceMetadata {
        source_file: source_file.unwrap_or_default(),
        source_section,
        source_line,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extract_control_with_all_three_props() {
        let element = json!({
            "id": "POL-AC-001",
            "props": [
                { "name": "source-file", "ns": FORGE_TRACE_NS, "value": "policy.md" },
                { "name": "source-section", "ns": FORGE_TRACE_NS, "value": "Access Control" },
                { "name": "source-line", "ns": FORGE_TRACE_NS, "value": "42" }
            ]
        });
        let meta = extract_trace_metadata(&element).unwrap();
        assert_eq!(meta.source_file, "policy.md");
        assert_eq!(meta.source_section, "Access Control");
        assert_eq!(meta.source_line, Some(42));
    }

    #[test]
    fn extract_element_with_no_props_returns_none() {
        let element = json!({ "id": "POL-AC-001" });
        assert!(extract_trace_metadata(&element).is_none());
    }

    #[test]
    fn extract_group_with_only_source_section() {
        let element = json!({
            "id": "access-control",
            "props": [
                { "name": "source-section", "ns": FORGE_TRACE_NS, "value": "Access Control" }
            ]
        });
        let meta = extract_trace_metadata(&element).unwrap();
        assert_eq!(meta.source_section, "Access Control");
        assert_eq!(meta.source_line, None);
        assert_eq!(meta.source_file, "");
    }

    #[test]
    fn extract_element_with_non_trace_props_returns_none() {
        let element = json!({
            "id": "POL-AC-001",
            "props": [
                { "name": "label", "ns": "https://other.ns", "value": "AC-1" }
            ]
        });
        assert!(extract_trace_metadata(&element).is_none());
    }

    #[test]
    fn malformed_trace_props_are_skipped() {
        let element = json!({
            "props": [
                { "name": "source-section", "ns": FORGE_TRACE_NS, "value": 42 },
                { "name": 42, "ns": FORGE_TRACE_NS, "value": "Access Control" }
            ]
        });
        assert!(extract_trace_metadata(&element).is_none());
    }

    #[test]
    fn first_valid_trace_values_win_over_later_malformed_duplicates() {
        let element = json!({
            "props": [
                { "name": "source-section", "ns": FORGE_TRACE_NS, "value": "Access Control" },
                { "name": "source-section", "ns": FORGE_TRACE_NS, "value": 42 },
                { "name": "source-line", "ns": FORGE_TRACE_NS, "value": "42" },
                { "name": "source-line", "ns": FORGE_TRACE_NS, "value": "not-a-number" }
            ]
        });
        let meta = extract_trace_metadata(&element).unwrap();
        assert_eq!(meta.source_section, "Access Control");
        assert_eq!(meta.source_line, Some(42));
    }

    #[test]
    fn empty_source_section_is_not_mapped() {
        let element = json!({
            "props": [{ "name": "source-section", "ns": FORGE_TRACE_NS, "value": "" }]
        });

        assert_eq!(extract_trace_metadata(&element), None);
    }
}
