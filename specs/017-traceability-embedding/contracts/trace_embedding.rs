//! Interface contract for WI-17: Traceability Embedding.
//!
//! This file defines the public API surface for the `trace_embedding` module.
//! Implementation follows this contract exactly.

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

/// Build 3 namespaced trace props for a source location.
///
/// Returns props in order: source-file, source-section, source-line.
/// All props have `ns: Some(FORGE_TRACE_NS)`.
///
/// # Arguments
/// * `source_file` - Path to the source policy file (preserved as-is per clarification Q2)
/// * `section_title` - Hierarchical section path (S-2)
/// * `line_number` - 1-based line number in the source file
///
/// # Contract
/// - Returns exactly 3 props
/// - All props have `ns == Some(FORGE_TRACE_NS)` (SEC-4)
/// - Prop names use module constants (SEC-5)
/// - `source-line` value is the string representation of `line_number`
#[must_use]
pub fn build_trace_props(
    source_file: &str,
    section_title: &str,
    line_number: usize,
) -> Vec<OscalProp> {
    // Implementation: construct 3 OscalProp instances with FORGE_TRACE_NS namespace
    todo!()
}

/// Build 1 source link with href `"<encoded_file>#line=<n>"`.
///
/// # Arguments
/// * `source_file` - Path to the source policy file
/// * `line_number` - 1-based line number
///
/// # Contract
/// - Returns exactly 1 `OscalLink`
/// - `rel` is `LINK_REL_SOURCE` ("source")
/// - `href` format: `"<percent_encoded_path>#line=<line_number>"`
/// - `text` is `None`
/// - File path in href is percent-encoded via `encode_href_path` (SEC-3, EC-6)
#[must_use]
pub fn build_trace_link(source_file: &str, line_number: usize) -> OscalLink {
    // Implementation: encode path, format href, construct OscalLink
    todo!()
}

/// Percent-encode special characters in a file path for use in link href.
///
/// Encodes ALL RFC 3986 reserved characters, spaces, ASCII controls, and
/// non-ASCII (UTF-8 percent-bytes) per EC-6. Shipped: encode_href_path in
/// src/oscal/trace_embedding.rs.
///
/// # Contract
/// - `%` -> `%25` (input is never treated as already-encoded)
/// - Spaces become `%20`, not `+`; `#` becomes `%23`
/// - Reserved set (`:/?#[]@!$&'()*+,;=`) is fully escaped so a filename
///   cannot alter the path, query, or fragment of the generated link
/// - Non-ASCII becomes UTF-8 percent-bytes (`café.md` -> `caf%C3%A9.md`):
///   URIs are ASCII; raw UTF-8 would make an IRI, not a URI
/// - ASCII control characters are percent-encoded, never emitted raw
/// - Unreserved characters (alphanumerics, `-._~`) pass through unchanged
/// - Empty input returns empty string; output is bounded to 4096 bytes
///   (truncation warns, never splits a percent-escape)
fn encode_href_path(path: &str) -> String {
    // Implementation: sequential replacement with % first
    todo!()
}

// ─── Embedding Functions ─────────────────────────────────────────────────

/// Walk catalog groups and controls, inject trace props/links from TraceLinkCollection.
///
/// For each **group**:
/// - Derive `source-section` from first child control's trace link `section_title` (S-1)
/// - Add `source-section` prop to `group.props`
/// - If no child controls have trace links, skip the prop (EC-4)
///
/// For each **control**:
/// - Look up `trace_links.by_oscal_element(control.uuid)` -> `Option<&TraceLink>`
/// - If found: remove existing FORGE_TRACE_NS props and LINK_REL_SOURCE links from
///   the target element, then `build_trace_props(file, section, line)` -> 3 props
///   and `build_trace_link(file, line)` -> 1 link (when it fits the href budget)
/// - Idempotent: re-embedding replaces prior FORGE trace annotations instead of
///   duplicating them (group `source-section` prop is likewise replaced)
/// - If not found: log `tracing::debug!` and skip (no error — supports partial trace data)
///
/// # Arguments
/// * `catalog` - Mutable reference to the catalog to annotate
/// * `trace_links` - The trace link collection built during catalog generation
///
/// # Contract
/// - Every control with a matching trace link gets 3 props + 1 link (M-1, M-2)
/// - Every group with traceable controls gets 1 source-section prop (S-1)
/// - No trace data appears in any `remarks` field (M-7, SEC-1, SEC-2)
/// - All prop names use module constants (SEC-5)
/// - Logs count of annotated controls/groups at `tracing::debug!` level
pub fn embed_trace_in_catalog(
    catalog: &mut OscalCatalog,
    trace_links: &TraceLinkCollection,
) {
    // Implementation: iterate groups -> controls, look up trace links, inject props/links
    todo!()
}

// ─── OscalProp Extension Contract ────────────────────────────────────────

// The following change is made to parts.rs::OscalProp:
//
// BEFORE:
//   pub struct OscalProp {
//       pub name: String,
//       pub value: String,
//   }
//
// AFTER:
//   pub struct OscalProp {
//       pub name: String,
//       #[serde(default, skip_serializing_if = "Option::is_none")]
//       pub ns: Option<String>,
//       pub value: String,
//   }
//
// `default` is mandatory, not optional: skip_serializing_if governs OUTPUT
// only. Without it, serde demands the `ns` key on input and every
// pre-existing artifact lacking `ns` fails to parse ("missing field `ns`").
// All existing construction sites must add `ns: None`.

// ─── OscalGroup Extension Contract ──────────────────────────────────────

// The following change is made to catalog.rs::OscalGroup:
//
// BEFORE:
//   pub struct OscalGroup {
//       pub id: String,
//       pub title: String,
//       pub controls: Vec<OscalControl>,
//   }
//
// AFTER:
//   pub struct OscalGroup {
//       pub id: String,
//       pub title: String,
//       #[serde(default, skip_serializing_if = "Vec::is_empty")]
//       pub props: Vec<OscalProp>,
//       #[serde(default, skip_serializing_if = "Vec::is_empty")]
//       pub links: Vec<OscalLink>,
//       #[serde(default, skip_serializing_if = "Vec::is_empty")]
//       pub controls: Vec<OscalControl>,
//   }
// `default` on each new Vec keeps pre-extension artifacts parseable.

// ─── DocumentaryComponent Extension Contract ────────────────────────────

// The following change is made to component_definition.rs::DocumentaryComponent:
//
// BEFORE:
//   pub struct DocumentaryComponent {
//       pub uuid: String,
//       pub component_type: String,
//       pub title: String,
//       pub description: String,
//       pub control_implementations: Vec<serde_json::Value>,
//   }
//
// AFTER:
//   pub struct DocumentaryComponent {
//       pub uuid: String,
//       pub component_type: String,
//       pub title: String,
//       pub description: String,
//       #[serde(default, skip_serializing_if = "Vec::is_empty")]
//       pub props: Vec<OscalProp>,
//       pub control_implementations: Vec<serde_json::Value>,
//   }
// `default` keeps pre-extension components parseable (see OscalProp above).

// ─── Implemented Requirements Trace Contract ────────────────────────────

// The following changes are made to implemented_requirements.rs:
//
// 1. `map_requirement_to_implemented` gains two new parameters:
//      fn map_requirement_to_implemented(
//          requirement: &PolicyRequirement,
//          control_id: &str,
//          global_index: usize,
//          source_file: &str,         // NEW
//          section_title: &str,       // NEW
//      ) -> Value
//
// 2. The JSON output includes `props` and `links` arrays:
//      serde_json::json!({
//          "uuid": uuid.to_string(),
//          "control-id": control_id,
//          "description": description,
//          "props": build_trace_props(source_file, section_title, requirement.source_line),
//          "links": [build_trace_link(source_file, requirement.source_line)],
//      })
//
// 3. `build_control_implementations` uses `collect_requirements_with_section`
//    (made pub(crate)) instead of `collect_requirements` to get section titles.
//
// 4. `build_component_definition` passes `input_path.display().to_string()`
//    as source_file for the DocumentaryComponent's source-file prop.
