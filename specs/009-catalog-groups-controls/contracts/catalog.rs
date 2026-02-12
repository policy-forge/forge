// Interface Contract: OSCAL Catalog Groups and Controls (WI-9)
//
// This file defines the type signatures and function contracts for the Catalog builder.
// It is a reference document — the actual implementation lives in src/oscal/catalog.rs.
//
// DO NOT compile this file directly; it is a design artifact.

use serde::Serialize;

// ─── Error Variant ──────────────────────────────────────────────────────────

// Added to ForgeError in src/error.rs:
//
// #[error("Catalog build error: {0}")]
// CatalogBuild(String),

// ─── OSCAL Structs ──────────────────────────────────────────────────────────

/// JSON envelope: produces `{"catalog": {...}}` at the top level.
#[derive(Debug, Clone, Serialize)]
pub struct CatalogEnvelope {
    pub catalog: OscalCatalog,
}

/// OSCAL Catalog root structure.
/// Metadata and UUID are placeholders — populated by WI-11.
#[derive(Debug, Clone, Serialize)]
pub struct OscalCatalog {
    pub uuid: String,
    pub metadata: OscalMetadata,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<OscalGroup>,
}

/// OSCAL Group mapped from a PolicySection.
#[derive(Debug, Clone, Serialize)]
pub struct OscalGroup {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub controls: Vec<OscalControl>,
}

/// OSCAL Control mapped from a PolicyRequirement.
#[derive(Debug, Clone, Serialize)]
pub struct OscalControl {
    pub id: String,
    pub uuid: String,
    pub title: String,
}

/// Placeholder metadata — fully implemented in WI-11.
#[derive(Debug, Clone, Serialize)]
pub struct OscalMetadata {
    pub title: String,
    #[serde(rename = "last-modified")]
    pub last_modified: String,
    pub version: String,
    #[serde(rename = "oscal-version")]
    pub oscal_version: String,
}

// ─── Builder Function ───────────────────────────────────────────────────────

/// Build an OSCAL Catalog from a PolicyDocument.
///
/// Pure function: reads domain model, produces OSCAL struct. No side effects.
///
/// # Errors
///
/// Returns `ForgeError::CatalogBuild` if:
/// - Any `PolicyRequirement.stable_id` is `None` (SEC-1, M-6)
/// - Control ID collision cannot be resolved (should not happen with numeric suffix)
///
/// # Algorithm
///
/// 1. For each top-level PolicySection in document.sections:
///    a. Generate group ID via `generate_group_id(section.title)`
///    b. Generate section abbreviation via `generate_section_abbreviation(section.title)`
///    c. Resolve abbreviation collisions (numeric suffix: AC → AC2 → AC3)
///    d. Recursively collect all requirements (section + children)
///    e. For each requirement (indexed 0..N):
///       - Validate stable_id is Some
///       - Generate control ID via `generate_control_id(abbreviation, index, "POL")`
///       - Derive control title via `derive_control_title(requirement.text)`
///       - Create OscalControl { id, uuid: stable_id, title }
///    f. Create OscalGroup { id, title: section.title, controls }
/// 2. Assemble OscalCatalog with placeholder uuid, placeholder metadata, and groups
/// 3. Log at DEBUG: group count, control count, any collisions resolved
pub fn build_catalog(document: &PolicyDocument) -> Result<OscalCatalog, ForgeError>;

// ─── Helper Functions ───────────────────────────────────────────────────────

/// Slugify a section title into a group ID.
///
/// - Lowercase
/// - Replace non-alphanumeric with hyphens
/// - Collapse consecutive hyphens
/// - Trim leading/trailing hyphens
///
/// # Examples
/// - "Access Control Policies" → "access-control-policies"
/// - "Data Protection & Privacy" → "data-protection-privacy"
fn generate_group_id(section_title: &str) -> String;

/// Derive a section abbreviation from the title.
///
/// - Split into words
/// - Remove stop words: ["a", "an", "and", "the", "of", "for", "in", "to"]
/// - Take first character of each remaining word, uppercase
/// - If empty result, use first 2 chars of title uppercased
///
/// # Examples
/// - "Access Control" → "AC"
/// - "Incident Response and Recovery" → "IRR"
fn generate_section_abbreviation(section_title: &str) -> String;

/// Generate a control ID from abbreviation and index.
///
/// Pattern: {prefix}-{abbreviation}-{NNN}
/// Index is 0-based internally, displayed as 1-based.
/// Zero-pad to 3 digits; extend naturally if >999.
///
/// # Examples
/// - generate_control_id("AC", 0, "POL") → "POL-AC-001"
/// - generate_control_id("DP", 4, "POL") → "POL-DP-005"
fn generate_control_id(
    section_abbreviation: &str,
    requirement_index: usize,
    prefix: &str,
) -> String;

/// Derive a control title from requirement text.
///
/// 1. Find first sentence (up to first '.', '!', or '?')
/// 2. If no sentence-ending punctuation, use full text
/// 3. Trim whitespace
/// 4. If > 120 chars, truncate to 120 and append "..."
///
/// # Examples
/// - "Systems shall require MFA. Additional info." → "Systems shall require MFA."
/// - "All access must be logged" → "All access must be logged"
fn derive_control_title(requirement_text: &str) -> String;

/// Recursively collect all requirements from a section and its children.
///
/// Preserves order: section's own requirements first, then children's
/// requirements in depth-first order.
fn collect_requirements(section: &PolicySection) -> Vec<&PolicyRequirement>;
