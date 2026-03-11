use crate::oscal::trace_embedding::{
    FORGE_TRACE_NS, PROP_SOURCE_FILE, PROP_SOURCE_LINE, PROP_SOURCE_SECTION,
};

use super::report::TraceMetadata;

/// Strip ASCII control characters (0x00-0x1F, excluding 0x0A newline and 0x09 tab)
/// from a string. Used to prevent terminal escape injection (SEC-5).
#[must_use]
pub fn strip_control_chars(s: &str) -> String {
    s.chars()
        .filter(|&c| {
            let b = c as u32;
            // Keep everything except control chars 0x00-0x1F and DEL 0x7F, but preserve tab (0x09) and newline (0x0A)
            (b >= 0x20 && b != 0x7F) || b == 0x09 || b == 0x0A
        })
        .collect()
}

/// Extract trace metadata from an OSCAL element's `props` array (`serde_json::Value`).
///
/// Scans `element["props"]` for props with `ns == FORGE_TRACE_NS`.
/// Returns `Some(TraceMetadata)` if at least `source-section` is found.
/// Returns `None` if no trace props exist.
///
/// For groups: may return `TraceMetadata` with `source_line == 0` (no line prop).
#[must_use]
pub fn extract_trace_metadata(element: &serde_json::Value) -> Option<TraceMetadata> {
    let props = element.get("props")?.as_array()?;

    let mut source_file = None;
    let mut source_section = None;
    let mut source_line = None;

    for prop in props {
        let ns = prop.get("ns").and_then(|v| v.as_str()).unwrap_or("");
        if ns != FORGE_TRACE_NS {
            continue;
        }
        let name = prop.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let value = prop.get("value").and_then(|v| v.as_str()).unwrap_or("");

        match name {
            PROP_SOURCE_FILE => source_file = Some(value.to_string()),
            PROP_SOURCE_SECTION => source_section = Some(value.to_string()),
            PROP_SOURCE_LINE => source_line = value.parse::<usize>().ok(),
            _ => {}
        }
    }

    // Must have at least source-section to be considered mapped
    let section = source_section?;

    Some(TraceMetadata {
        source_file: source_file.unwrap_or_default(),
        source_section: section,
        source_line: source_line.unwrap_or(0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // T006: Unit tests for extract_trace_metadata()

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
        assert_eq!(meta.source_line, 42);
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
        assert_eq!(meta.source_line, 0);
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
    fn extract_element_with_unparseable_line_returns_none_line() {
        let element = json!({
            "id": "POL-AC-001",
            "props": [
                { "name": "source-file", "ns": FORGE_TRACE_NS, "value": "policy.md" },
                { "name": "source-section", "ns": FORGE_TRACE_NS, "value": "Access Control" },
                { "name": "source-line", "ns": FORGE_TRACE_NS, "value": "not-a-number" }
            ]
        });
        let meta = extract_trace_metadata(&element).unwrap();
        assert_eq!(meta.source_line, 0);
    }

    // T007: Unit tests for strip_control_chars()

    #[test]
    fn strip_ansi_escape() {
        let input = "Hello\x1b[31m World\x1b[0m";
        let result = strip_control_chars(input);
        assert_eq!(result, "Hello[31m World[0m");
    }

    #[test]
    fn strip_preserves_tab_and_newline() {
        let input = "Hello\tWorld\nNext line";
        let result = strip_control_chars(input);
        assert_eq!(result, "Hello\tWorld\nNext line");
    }

    #[test]
    fn strip_clean_string_unchanged() {
        let input = "Hello World";
        let result = strip_control_chars(input);
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn strip_empty_string() {
        let result = strip_control_chars("");
        assert_eq!(result, "");
    }

    #[test]
    fn strip_del_character() {
        let input = "Hello\x7FWorld";
        let result = strip_control_chars(input);
        assert_eq!(result, "HelloWorld");
    }
}
