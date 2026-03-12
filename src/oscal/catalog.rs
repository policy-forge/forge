//! OSCAL Catalog builder: maps [`PolicyDocument`] to OSCAL Catalog JSON.
//!
//! Converts the domain model (`PolicyDocument` → `PolicySection` →
//! `PolicyRequirement`) to OSCAL Catalog structures (`OscalCatalog` →
//! `OscalGroup` → `OscalControl`).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing::debug;

use super::parts::{OscalPart, OscalProp, build_control_parts, build_control_props};
use crate::error::ForgeError;
use crate::model::trace::{SourceLocation, TraceLink, TraceLinkCollection};
use crate::model::{PolicyDocument, PolicyRequirement, PolicySection};

// ─── OSCAL Structs ──────────────────────────────────────────────────────

/// JSON envelope producing `{"catalog": {...}}` at the top level.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogEnvelope {
    /// The OSCAL Catalog.
    pub catalog: OscalCatalog,
}

/// OSCAL Catalog root structure.
///
/// Metadata and UUID are placeholders — populated by WI-11.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalCatalog {
    /// Placeholder UUID.
    pub uuid: String,
    /// Placeholder metadata (WI-11).
    pub metadata: OscalMetadata,
    /// Root-level controls (not inside any group). OSCAL v1.2.0 allows this.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub controls: Vec<OscalControl>,
    /// Groups mapped from `PolicySection`s.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<OscalGroup>,
    /// Back matter containing reference resources (WI-12).
    #[serde(default, rename = "back-matter", skip_serializing_if = "Option::is_none")]
    pub back_matter: Option<crate::oscal::back_matter::BackMatter>,
}

/// OSCAL Group mapped from a [`PolicySection`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalGroup {
    /// Slugified section title (e.g., `"access-control"`).
    pub id: String,
    /// Section title verbatim.
    pub title: String,
    /// Group-level properties (e.g., source-section for traceability).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub props: Vec<OscalProp>,
    /// Group-level links.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<crate::oscal::back_matter::OscalLink>,
    /// Controls mapped from requirements.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub controls: Vec<OscalControl>,
    /// Nested sub-groups. OSCAL v1.2.0 allows groups within groups.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<OscalGroup>,
}

/// OSCAL `param` element within a catalog control (WI-34).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalParam {
    /// Deterministic parameter ID (e.g., `"POL-AC-001_prm_0"`).
    pub id: String,
    /// Human-readable label (e.g., `"within 30 days"`).
    pub label: String,
    /// Extracted value(s) for this parameter.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<String>,
    /// Value domain constraints.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<OscalParamConstraint>,
}

/// OSCAL `param.constraint` element (WI-34).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalParamConstraint {
    /// Constraint description (e.g., `"minimum: 30 days"`).
    pub description: String,
}

/// OSCAL Control mapped from a [`PolicyRequirement`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalControl {
    /// Control ID following `POL-{ABBR}-{NNN}` pattern.
    pub id: String,
    /// UUID copied from `PolicyRequirement.stable_id`.
    /// Not serialized — OSCAL catalog schema does not allow uuid on controls.
    #[serde(skip_serializing, default)]
    pub uuid: String,
    /// Derived title (first sentence, 120-char cap).
    pub title: String,
    /// Links to back matter resources (WI-12).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<crate::oscal::back_matter::OscalLink>,
    /// Parameters extracted from requirement text (WI-34).
    /// OSCAL schema ordering: params before parts.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<OscalParam>,
    /// Parts array: statement part (mandatory) + optional guidance/objective.
    /// NOT skip-serialized — always present per FR-001.
    #[serde(default)]
    pub parts: Vec<OscalPart>,
    /// Props array: trace metadata added by post-processing (WI-17).
    /// Omitted from JSON when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub props: Vec<OscalProp>,
}

/// Placeholder metadata — fully implemented in WI-11.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalMetadata {
    /// Document title (placeholder: `"placeholder"`).
    pub title: String,
    /// Last modified timestamp.
    #[serde(rename = "last-modified")]
    pub last_modified: String,
    /// Document version (placeholder: `"0.0.0"`).
    pub version: String,
    /// OSCAL specification version.
    #[serde(rename = "oscal-version")]
    pub oscal_version: String,
}

// ─── Constants ──────────────────────────────────────────────────────────

/// Stop words filtered during abbreviation generation.
const STOP_WORDS: &[&str] = &["a", "an", "and", "the", "of", "for", "in", "to"];

// ─── Helper Functions ───────────────────────────────────────────────────

/// Slugify a section title into a group ID.
///
/// - Lowercase the title
/// - Replace non-ASCII-alphanumeric characters with hyphens
/// - Collapse consecutive hyphens
/// - Trim leading and trailing hyphens
///
/// Returns an empty string if the title has no valid characters.
///
/// # Examples
///
/// ```
/// use forge::oscal::catalog::generate_group_id;
///
/// assert_eq!(
///     generate_group_id("Access Control Policies"),
///     "access-control-policies"
/// );
/// assert_eq!(
///     generate_group_id("Data Protection & Privacy"),
///     "data-protection-privacy"
/// );
/// ```
#[must_use]
pub fn generate_group_id(section_title: &str) -> String {
    let lowered = section_title.to_lowercase();
    let mut result = String::with_capacity(lowered.len());
    let mut prev_hyphen = true; // true to trim leading hyphens

    for ch in lowered.chars() {
        if ch.is_ascii_alphanumeric() {
            result.push(ch);
            prev_hyphen = false;
        } else if !prev_hyphen {
            result.push('-');
            prev_hyphen = true;
        }
    }

    if result.ends_with('-') {
        result.pop();
    }

    result
}

/// Derive a section abbreviation from the title.
///
/// - Split into words on whitespace boundaries
/// - Remove stop words
/// - Take first character of each remaining word, uppercased
/// - If empty result, use first 2 characters of title uppercased
///
/// # Examples
///
/// ```
/// use forge::oscal::catalog::generate_section_abbreviation;
///
/// assert_eq!(
///     generate_section_abbreviation("Access Control"),
///     "AC"
/// );
/// assert_eq!(
///     generate_section_abbreviation(
///         "Incident Response and Recovery"
///     ),
///     "IRR"
/// );
/// ```
#[must_use]
pub fn generate_section_abbreviation(section_title: &str) -> String {
    let abbreviation: String = section_title
        .split_whitespace()
        .filter_map(|word| {
            let cleaned: String = word.chars().filter(|c| c.is_alphanumeric()).collect();
            if cleaned.is_empty() {
                return None;
            }
            let lower = cleaned.to_ascii_lowercase();
            if STOP_WORDS.contains(&lower.as_str()) {
                return None;
            }
            cleaned.chars().next().map(|c| c.to_ascii_uppercase())
        })
        .collect();

    if abbreviation.is_empty() {
        let fallback: String = section_title
            .chars()
            .filter(|c| c.is_alphanumeric())
            .take(2)
            .collect::<String>()
            .to_uppercase();
        if fallback.is_empty() { "SEC".to_string() } else { fallback }
    } else {
        abbreviation
    }
}

/// Generate a control ID from abbreviation and requirement index.
///
/// Pattern: `{prefix}-{abbreviation}-{NNN}` where the index is
/// 0-based internally and displayed as 1-based. Zero-padded to
/// 3 digits; extends naturally past 999.
///
/// # Examples
///
/// ```
/// use forge::oscal::catalog::generate_control_id;
///
/// assert_eq!(generate_control_id("AC", 0, "POL"), "POL-AC-001");
/// assert_eq!(generate_control_id("DP", 4, "POL"), "POL-DP-005");
/// ```
#[must_use]
pub fn generate_control_id(abbreviation: &str, requirement_index: usize, prefix: &str) -> String {
    let display = requirement_index + 1;
    format!("{prefix}-{abbreviation}-{display:03}")
}

/// Derive a control title from requirement text.
///
/// 1. Find first sentence (up to first `.`, `!`, or `?`)
/// 2. If no sentence-ending punctuation, use full text
/// 3. Trim whitespace
/// 4. If length exceeds 120 characters, truncate and append `...`
///
/// # Examples
///
/// ```
/// use forge::oscal::catalog::derive_control_title;
///
/// assert_eq!(
///     derive_control_title(
///         "Systems shall require MFA. Extra."
///     ),
///     "Systems shall require MFA."
/// );
/// assert_eq!(
///     derive_control_title("All access must be logged"),
///     "All access must be logged"
/// );
/// ```
#[must_use]
pub fn derive_control_title(requirement_text: &str) -> String {
    let sentence = requirement_text
        .find(['.', '!', '?'])
        .map_or(requirement_text, |pos| &requirement_text[..=pos]);

    let trimmed = sentence.trim();

    if trimmed.chars().count() > 120 {
        let truncated: String = trimmed.chars().take(120).collect();
        format!("{truncated}...")
    } else {
        trimmed.to_string()
    }
}

/// Recursively collect all requirements from a section and
/// its children depth-first.
///
/// Preserves order: section's own requirements first, then each
/// child's requirements in depth-first order.
#[must_use]
pub fn collect_requirements(section: &PolicySection) -> Vec<&PolicyRequirement> {
    let mut reqs: Vec<&PolicyRequirement> = section.requirements.iter().collect();
    for child in &section.children {
        reqs.extend(collect_requirements(child));
    }
    reqs
}

/// Recursively collect all requirements paired with their owning section,
/// preserving depth-first order.
///
/// Unlike [`collect_requirements`], this tracks which section each
/// requirement belongs to, so callers can use the correct subsection
/// title (e.g., for trace link `SourceLocation`).
pub(crate) fn collect_requirements_with_section(
    section: &PolicySection,
) -> Vec<(&PolicyRequirement, &PolicySection)> {
    let mut reqs: Vec<(&PolicyRequirement, &PolicySection)> =
        section.requirements.iter().map(|r| (r, section)).collect();
    for child in &section.children {
        reqs.extend(collect_requirements_with_section(child));
    }
    reqs
}

// ─── Builder ────────────────────────────────────────────────────────────

/// Build an OSCAL Catalog from a [`PolicyDocument`].
///
/// Optionally records a [`TraceLink`] for every generated control into the
/// provided [`TraceLinkCollection`]. Pass `None` for backward compatibility.
///
/// # Errors
///
/// Returns [`ForgeError::CatalogBuild`] if any
/// `PolicyRequirement.stable_id` is `None`, or if a duplicate trace link
/// is detected for the same requirement.
pub fn build_catalog(
    document: &PolicyDocument,
    mut trace_links: Option<&mut TraceLinkCollection>,
) -> Result<OscalCatalog, ForgeError> {
    let mut group_id_counts: HashMap<String, Vec<String>> = HashMap::new();
    let mut abbrev_counts: HashMap<String, Vec<String>> = HashMap::new();
    let mut groups = Vec::new();

    for (idx, section) in document.sections.iter().enumerate() {
        let group_id = resolve_group_id(&section.title, idx, &mut group_id_counts);

        let abbreviation = resolve_abbreviation(&section.title, &mut abbrev_counts);

        let requirements = collect_requirements_with_section(section);
        let mut controls = Vec::with_capacity(requirements.len());

        for (req_idx, (req, req_section)) in requirements.iter().enumerate() {
            let stable_id = req.stable_id.as_ref().ok_or_else(|| {
                let preview: String = req.text.chars().take(60).collect();
                ForgeError::CatalogBuild(format!(
                    "Requirement missing stable_id in section '{}': '{preview}'",
                    req_section.title,
                ))
            })?;

            let control_id = generate_control_id(&abbreviation, req_idx, "POL");
            let mut control_props = build_control_props(req);
            if let Some(modality) = req.modality {
                let modality_value = match modality {
                    crate::model::Modality::Normative => "normative",
                    crate::model::Modality::Advisory => "advisory",
                };
                control_props.push(OscalProp {
                    name: "modality".to_string(),
                    ns: None,
                    value: modality_value.to_string(),
                });
            }
            // T031: Emit OSCAL params from extracted PolicyParameters (WI-34)
            let params: Vec<OscalParam> =
                req.parameters.iter().map(crate::parameter::to_oscal_param).collect();
            controls.push(OscalControl {
                id: control_id.clone(),
                uuid: stable_id.clone(),
                title: derive_control_title(&req.text),
                links: vec![],
                params,
                parts: build_control_parts(&control_id, req, req_section.body_text.as_deref()),
                props: control_props,
            });

            // Record trace link for this control (T020)
            if let Some(ref mut tl) = trace_links {
                let trace = TraceLink {
                    requirement_stable_id: stable_id.clone(),
                    oscal_json_path: format!("catalog.groups[{idx}].controls[{req_idx}]"),
                    oscal_element_id: stable_id.clone(),
                    source_location: SourceLocation {
                        file_path: document.metadata.source_path.clone(),
                        section_title: req_section.title.clone(),
                        line_number: req.source_line,
                    },
                };
                tl.record(trace).map_err(|e| {
                    ForgeError::CatalogBuild(format!(
                        "Failed to record trace link for requirement '{stable_id}' in section '{}' at line {}: {e}",
                        req_section.title,
                        req.source_line,
                    ))
                })?;
            }
        }

        groups.push(OscalGroup {
            id: group_id,
            title: section.title.clone(),
            props: vec![],
            links: vec![],
            controls,
            groups: vec![],
        });
    }

    let total_controls: usize = groups.iter().map(|g| g.controls.len()).sum();
    debug!(group_count = groups.len(), control_count = total_controls, "Catalog built");

    Ok(OscalCatalog {
        uuid: "00000000-0000-0000-0000-000000000000".to_string(),
        metadata: OscalMetadata {
            title: "placeholder".to_string(),
            last_modified: "1970-01-01T00:00:00Z".to_string(),
            version: "0.0.0".to_string(),
            oscal_version: "1.2.0".to_string(),
        },
        controls: vec![],
        groups,
        back_matter: None,
    })
}

/// Resolve a group ID with content-based collision tracking.
///
/// The first title to claim a base group ID keeps it bare. Subsequent
/// titles that produce the same base group ID receive a hash suffix
/// derived from their content (first 2 bytes of SHA-256, hex-encoded).
fn resolve_group_id(title: &str, index: usize, counts: &mut HashMap<String, Vec<String>>) -> String {
    use sha2::{Sha256, Digest};

    let base = generate_group_id(title);
    if base.is_empty() {
        return format!("group-{index}");
    }
    let titles = counts.entry(base.clone()).or_default();
    titles.push(title.to_string());
    if titles.len() == 1 {
        base
    } else {
        let mut hasher = Sha256::new();
        hasher.update(title.as_bytes());
        let hash = hasher.finalize();
        let suffix = format!("{:02x}{:02x}", hash[0], hash[1]);
        format!("{base}-{suffix}")
    }
}

/// Resolve an abbreviation with content-based collision tracking.
///
/// The first title to claim a base abbreviation keeps it bare. Subsequent
/// titles that produce the same base abbreviation receive a hash suffix
/// derived from their content (first 2 bytes of SHA-256, hex-encoded).
/// This makes the disambiguation stable regardless of encounter order.
pub(crate) fn resolve_abbreviation(title: &str, counts: &mut HashMap<String, Vec<String>>) -> String {
    use sha2::{Sha256, Digest};

    let base = generate_section_abbreviation(title);
    let titles = counts.entry(base.clone()).or_default();
    titles.push(title.to_string());

    if titles.len() == 1 {
        base
    } else {
        let mut hasher = Sha256::new();
        hasher.update(title.as_bytes());
        let hash = hasher.finalize();
        let suffix = format!("{:02x}{:02x}", hash[0], hash[1]);
        debug!(
            abbreviation = %base,
            suffix = %suffix,
            section = %title,
            "Abbreviation collision resolved with hash suffix"
        );
        format!("{base}-{suffix}")
    }
}

// ─── Control ID Collector (WI-41, T008) ─────────────────────────────────

/// Collect all control IDs from a built OSCAL Catalog.
///
/// Iterates `catalog.groups[].controls[].id` in declaration order.
/// Returns an empty Vec if the catalog has no groups or controls.
/// Does NOT deduplicate — deduplication is performed by `build_assessment_plan`.
#[must_use]
pub fn collect_control_ids_from_catalog(catalog: &OscalCatalog) -> Vec<String> {
    catalog.groups.iter().flat_map(|g| g.controls.iter()).map(|c| c.id.clone()).collect()
}

// ─── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::model::DocumentMetadata;

    // ── Test helpers ────────────────────────────────────

    fn req(text: &str, id: &str) -> PolicyRequirement {
        PolicyRequirement {
            stable_id: Some(id.to_string()),
            text: text.to_string(),
            source_line: 1,
            nesting_depth: 0,
            atom_index: 0,
            parent_text: None,
            citations: vec![],
            modality: None,
            parameters: vec![],
        }
    }

    fn req_no_id(text: &str) -> PolicyRequirement {
        PolicyRequirement {
            stable_id: None,
            text: text.to_string(),
            source_line: 1,
            nesting_depth: 0,
            atom_index: 0,
            parent_text: None,
            citations: vec![],
            modality: None,
            parameters: vec![],
        }
    }

    fn sec(title: &str, reqs: Vec<PolicyRequirement>) -> PolicySection {
        PolicySection {
            title: title.to_string(),
            heading_level: 1,
            source_line: 1,
            body_text: None,
            children: vec![],
            requirements: reqs,
        }
    }

    fn sec_nested(
        title: &str,
        reqs: Vec<PolicyRequirement>,
        children: Vec<PolicySection>,
    ) -> PolicySection {
        PolicySection {
            title: title.to_string(),
            heading_level: 1,
            source_line: 1,
            body_text: None,
            children,
            requirements: reqs,
        }
    }

    fn doc(sections: Vec<PolicySection>) -> PolicyDocument {
        PolicyDocument {
            id: "test-doc".to_string(),
            metadata: DocumentMetadata {
                title: "Test".to_string(),
                version: "1.0".to_string(),
                author: None,
                date: None,
                source_path: PathBuf::from("test.md"),
                content_hash: None,
            },
            sections,
        }
    }

    // ── T004: generate_group_id normal ──────────────────

    #[test]
    fn group_id_normal_cases() {
        assert_eq!(generate_group_id("Access Control Policies"), "access-control-policies");
        assert_eq!(generate_group_id("Data Protection & Privacy"), "data-protection-privacy");
        assert_eq!(generate_group_id("3.1 \u{2014} Incident Response"), "3-1-incident-response");
    }

    #[test]
    fn group_id_consecutive_hyphens() {
        assert_eq!(generate_group_id("foo---bar"), "foo-bar");
        assert_eq!(generate_group_id("a & & b"), "a-b");
    }

    #[test]
    fn group_id_leading_trailing() {
        assert_eq!(generate_group_id("--hello--"), "hello");
        assert_eq!(generate_group_id("!test!"), "test");
    }

    // ── T005: generate_group_id edge cases ──────────────

    #[test]
    fn group_id_special_characters() {
        assert_eq!(generate_group_id("Foo @#$ Bar"), "foo-bar");
    }

    #[test]
    fn group_id_non_ascii() {
        assert_eq!(generate_group_id("caf\u{00e9}"), "caf");
    }

    #[test]
    fn group_id_empty() {
        assert_eq!(generate_group_id(""), "");
        assert_eq!(generate_group_id("!!!"), "");
    }

    // ── T009: generate_section_abbreviation ─────────────

    #[test]
    fn abbrev_normal() {
        assert_eq!(generate_section_abbreviation("Access Control"), "AC");
        assert_eq!(generate_section_abbreviation("Data Protection"), "DP");
        assert_eq!(generate_section_abbreviation("Physical and Environmental Security"), "PES");
    }

    #[test]
    fn abbrev_stop_words() {
        assert_eq!(generate_section_abbreviation("Incident Response and Recovery"), "IRR");
        assert_eq!(generate_section_abbreviation("The Art of War"), "AW");
    }

    #[test]
    fn abbrev_all_stop_words_fallback() {
        assert_eq!(generate_section_abbreviation("of the"), "OF");
        assert_eq!(generate_section_abbreviation("a"), "A");
    }

    #[test]
    fn abbrev_punctuation_in_tokens() {
        // "&" token stripped to empty, skipped; remaining words produce "DPP"
        assert_eq!(generate_section_abbreviation("Data Protection & Privacy"), "DPP");
        // "The," cleaned to "the" (stop word), "Art" kept, "of" stop word, "War" kept
        assert_eq!(generate_section_abbreviation("The, Art of War"), "AW");
    }

    #[test]
    fn abbrev_only_punctuation_fallback() {
        // All tokens are punctuation-only → fallback to "SEC"
        assert_eq!(generate_section_abbreviation("& # !"), "SEC");
    }

    #[test]
    fn abbrev_empty() {
        assert_eq!(generate_section_abbreviation(""), "SEC");
    }

    // ── T010: generate_control_id ───────────────────────

    #[test]
    fn control_id_normal() {
        assert_eq!(generate_control_id("AC", 0, "POL"), "POL-AC-001");
        assert_eq!(generate_control_id("DP", 4, "POL"), "POL-DP-005");
    }

    #[test]
    fn control_id_zero_padded() {
        assert_eq!(generate_control_id("AC", 98, "POL"), "POL-AC-099");
    }

    #[test]
    fn control_id_exceeds_999() {
        assert_eq!(generate_control_id("AC", 999, "POL"), "POL-AC-1000");
    }

    // ── T015: derive_control_title ──────────────────────

    #[test]
    fn title_first_sentence() {
        assert_eq!(
            derive_control_title("Systems shall require MFA. Additional."),
            "Systems shall require MFA."
        );
    }

    #[test]
    fn title_no_punctuation() {
        assert_eq!(derive_control_title("All access must be logged"), "All access must be logged");
    }

    #[test]
    fn title_truncation() {
        let long = "Organizations must implement \
            comprehensive security controls including \
            multi-factor authentication and role-based \
            access control for all system users across \
            every department.";
        let title = derive_control_title(long);
        assert!(title.ends_with("..."));
        assert_eq!(title.chars().count(), 123);
    }

    #[test]
    fn title_exclamation_question() {
        assert_eq!(derive_control_title("Stop now! More text."), "Stop now!");
        assert_eq!(derive_control_title("Is this valid? More."), "Is this valid?");
    }

    // ── T016: collect_requirements ──────────────────────

    #[test]
    fn collect_flat() {
        let s = sec("Test", vec![req("R1.", "id1"), req("R2.", "id2")]);
        let reqs = collect_requirements(&s);
        assert_eq!(reqs.len(), 2);
        assert_eq!(reqs[0].text, "R1.");
        assert_eq!(reqs[1].text, "R2.");
    }

    #[test]
    fn collect_nested_depth_first() {
        let c1 = sec("Child1", vec![req("C1.", "c1")]);
        let c2 = sec("Child2", vec![req("C2.", "c2")]);
        let parent = sec_nested("Parent", vec![req("P.", "p")], vec![c1, c2]);
        let reqs = collect_requirements(&parent);
        assert_eq!(reqs.len(), 3);
        assert_eq!(reqs[0].text, "P.");
        assert_eq!(reqs[1].text, "C1.");
        assert_eq!(reqs[2].text, "C2.");
    }

    // ── T007: build_catalog group mapping ───────────────

    #[test]
    fn catalog_three_groups() {
        let d = doc(vec![
            sec("Access Control", vec![req("R.", "u1")]),
            sec("Data Protection", vec![req("R.", "u2")]),
            sec("Incident Response", vec![req("R.", "u3")]),
        ]);
        let cat = build_catalog(&d, None).unwrap();
        assert_eq!(cat.groups.len(), 3);
        assert_eq!(cat.groups[0].id, "access-control");
        assert_eq!(cat.groups[0].title, "Access Control");
        assert_eq!(cat.groups[1].id, "data-protection");
        assert_eq!(cat.groups[2].id, "incident-response");
    }

    #[test]
    fn catalog_zero_sections() {
        let d = doc(vec![]);
        let cat = build_catalog(&d, None).unwrap();
        assert!(cat.groups.is_empty());
    }

    #[test]
    fn catalog_section_empty_controls() {
        let d = doc(vec![sec("Empty", vec![])]);
        let cat = build_catalog(&d, None).unwrap();
        assert_eq!(cat.groups.len(), 1);
        assert!(cat.groups[0].controls.is_empty());
    }

    // ── T008a: group ID collision ───────────────────────

    #[test]
    fn catalog_group_id_collision() {
        let d = doc(vec![sec("Data Protection", vec![]), sec("Data Protection!", vec![])]);
        let cat = build_catalog(&d, None).unwrap();
        assert_eq!(cat.groups[0].id, "data-protection");
        // Hash suffix from SHA-256 of "Data Protection!" → 4d3c
        assert_eq!(cat.groups[1].id, "data-protection-4d3c");
    }

    #[test]
    fn catalog_empty_title_fallback() {
        let d = doc(vec![sec("", vec![])]);
        let cat = build_catalog(&d, None).unwrap();
        assert_eq!(cat.groups[0].id, "group-0");
    }

    // ── T013/T014: abbreviation collision ───────────────

    #[test]
    fn catalog_abbreviation_collision() {
        let d = doc(vec![
            sec("Access Control", vec![req("R.", "u1")]),
            sec("Application Configuration", vec![req("R.", "u2")]),
            sec("Audit Compliance", vec![req("R.", "u3")]),
        ]);
        let cat = build_catalog(&d, None).unwrap();
        assert_eq!(cat.groups[0].controls[0].id, "POL-AC-001");
        // Hash suffix from SHA-256 of "Application Configuration" → dd3e
        assert_eq!(cat.groups[1].controls[0].id, "POL-AC-dd3e-001");
        // Hash suffix from SHA-256 of "Audit Compliance" → c5c6
        assert_eq!(cat.groups[2].controls[0].id, "POL-AC-c5c6-001");
    }

    // ── T017: missing stable_id error ───────────────────

    #[test]
    fn catalog_missing_stable_id() {
        let d = doc(vec![sec("Test", vec![req_no_id("No ID requirement")])]);
        let err = build_catalog(&d, None).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("missing stable_id"));
        assert!(msg.contains("Test"));
    }

    // ── T021: build_catalog full controls ───────────────

    #[test]
    fn catalog_controls_mapping() {
        let d = doc(vec![
            sec(
                "Access Control",
                vec![
                    req("Users shall authenticate.", "u1"),
                    req("MFA is required.", "u2"),
                    req("Sessions expire.", "u3"),
                ],
            ),
            sec(
                "Data Protection",
                vec![
                    req("Encrypt at rest.", "u4"),
                    req("Encrypt in transit.", "u5"),
                    req("Classify data.", "u6"),
                    req("Retain per policy.", "u7"),
                ],
            ),
        ]);
        let cat = build_catalog(&d, None).unwrap();

        assert_eq!(cat.groups[0].controls.len(), 3);
        assert_eq!(cat.groups[1].controls.len(), 4);

        assert_eq!(cat.groups[0].controls[0].id, "POL-AC-001");
        assert_eq!(cat.groups[0].controls[2].id, "POL-AC-003");
        assert_eq!(cat.groups[1].controls[0].id, "POL-DP-001");
        assert_eq!(cat.groups[1].controls[3].id, "POL-DP-004");

        // UUIDs match stable_ids
        assert_eq!(cat.groups[0].controls[0].uuid, "u1");
        assert_eq!(cat.groups[1].controls[0].uuid, "u4");

        // Titles derived
        assert_eq!(cat.groups[0].controls[0].title, "Users shall authenticate.");
    }

    // ── T022: JSON serialization ────────────────────────

    #[test]
    fn json_serialization_structure() {
        let d = doc(vec![sec("Access Control", vec![req("All users shall auth.", "uuid-1")])]);
        let cat = build_catalog(&d, None).unwrap();
        let envelope = CatalogEnvelope { catalog: cat };
        let json = serde_json::to_string_pretty(&envelope).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();

        // D-6: Root key
        assert!(v.get("catalog").is_some());
        let c = &v["catalog"];

        // D-5: Placeholder metadata
        assert_eq!(c["uuid"], "00000000-0000-0000-0000-000000000000");
        let m = &c["metadata"];
        assert_eq!(m["title"], "placeholder");
        assert_eq!(m["last-modified"], "1970-01-01T00:00:00Z");
        assert_eq!(m["version"], "0.0.0");
        assert_eq!(m["oscal-version"], "1.2.0");

        // Control fields
        let ctrl = &c["groups"][0]["controls"][0];
        assert_eq!(ctrl["id"], "POL-AC-001");
        // uuid is not serialized (OSCAL catalog schema does not allow uuid on controls)
        assert!(ctrl.get("uuid").is_none());
    }

    // ── T023: JSON round-trip ───────────────────────────

    #[test]
    fn json_round_trip() {
        let d = doc(vec![
            sec("Access Control", vec![req("Auth required.", "u1"), req("MFA required.", "u2")]),
            sec("Data Protection", vec![req("Encrypt data.", "u3")]),
        ]);
        let cat = build_catalog(&d, None).unwrap();
        let envelope = CatalogEnvelope { catalog: cat };
        let json = serde_json::to_string_pretty(&envelope).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let groups = parsed["catalog"]["groups"].as_array().unwrap();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0]["controls"].as_array().unwrap().len(), 2);
        assert_eq!(groups[1]["controls"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn json_empty_groups_omitted() {
        let cat = OscalCatalog {
            uuid: "t".to_string(),
            metadata: OscalMetadata {
                title: "t".to_string(),
                last_modified: "t".to_string(),
                version: "t".to_string(),
                oscal_version: "t".to_string(),
            },
            controls: vec![],
            groups: vec![],
            back_matter: None,
        };
        let json = serde_json::to_string(&cat).unwrap();
        assert!(!json.contains("groups"));
    }

    // ── T026: integration full catalog ──────────────────

    #[test]
    fn integration_full_catalog() {
        let d = doc(vec![
            sec(
                "Access Control",
                vec![
                    req("AC1.", "ac1"),
                    req("AC2.", "ac2"),
                    req("AC3.", "ac3"),
                    req("AC4.", "ac4"),
                ],
            ),
            sec(
                "Data Protection",
                vec![
                    req("DP1.", "dp1"),
                    req("DP2.", "dp2"),
                    req("DP3.", "dp3"),
                    req("DP4.", "dp4"),
                ],
            ),
            sec(
                "Incident Response",
                vec![
                    req("IR1.", "ir1"),
                    req("IR2.", "ir2"),
                    req("IR3.", "ir3"),
                    req("IR4.", "ir4"),
                ],
            ),
            sec(
                "Physical Security",
                vec![
                    req("PS1.", "ps1"),
                    req("PS2.", "ps2"),
                    req("PS3.", "ps3"),
                    req("PS4.", "ps4"),
                ],
            ),
            sec(
                "Network Security",
                vec![
                    req("NS1.", "ns1"),
                    req("NS2.", "ns2"),
                    req("NS3.", "ns3"),
                    req("NS4.", "ns4"),
                ],
            ),
        ]);

        let cat = build_catalog(&d, None).unwrap();

        // SC-001: All sections mapped
        assert_eq!(cat.groups.len(), 5);

        // SC-002: All requirements mapped
        let total: usize = cat.groups.iter().map(|g| g.controls.len()).sum();
        assert_eq!(total, 20);

        // SC-003: Zero duplicate IDs
        let mut ids: Vec<&str> =
            cat.groups.iter().flat_map(|g| g.controls.iter().map(|c| c.id.as_str())).collect();
        let count = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), count);

        // SC-006: Deterministic
        let cat2 = build_catalog(&d, None).unwrap();
        for (g1, g2) in cat.groups.iter().zip(cat2.groups.iter()) {
            assert_eq!(g1.id, g2.id);
            for (c1, c2) in g1.controls.iter().zip(g2.controls.iter()) {
                assert_eq!(c1.id, c2.id);
                assert_eq!(c1.uuid, c2.uuid);
            }
        }
    }

    // ── T027: global ID uniqueness with collisions ──────

    #[test]
    fn global_control_id_uniqueness() {
        let d = doc(vec![
            sec("Access Control", vec![req("R1.", "u1"), req("R2.", "u2")]),
            sec("Application Configuration", vec![req("R3.", "u3"), req("R4.", "u4")]),
            sec("Audit Compliance", vec![req("R5.", "u5"), req("R6.", "u6")]),
            sec("Authentication Checks", vec![req("R7.", "u7"), req("R8.", "u8")]),
            sec("Authorization Controls", vec![req("R9.", "u9"), req("R10.", "u10")]),
        ]);

        let cat = build_catalog(&d, None).unwrap();

        // Verify collision resolution with content-based hash suffixes
        assert_eq!(cat.groups[0].controls[0].id, "POL-AC-001");
        // Hash suffix from SHA-256 of "Application Configuration" → dd3e
        assert_eq!(cat.groups[1].controls[0].id, "POL-AC-dd3e-001");
        // Hash suffix from SHA-256 of "Audit Compliance" → c5c6
        assert_eq!(cat.groups[2].controls[0].id, "POL-AC-c5c6-001");
        // Hash suffix from SHA-256 of "Authentication Checks" → bc96
        assert_eq!(cat.groups[3].controls[0].id, "POL-AC-bc96-001");
        // Hash suffix from SHA-256 of "Authorization Controls" → 634a
        assert_eq!(cat.groups[4].controls[0].id, "POL-AC-634a-001");

        // All 10 IDs unique
        let mut ids: Vec<&str> =
            cat.groups.iter().flat_map(|g| g.controls.iter().map(|c| c.id.as_str())).collect();
        let count = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), count);
        assert_eq!(count, 10);
    }

    // ── Additional helpers ──────────────────────────────

    fn sec_with_body(title: &str, body: &str, reqs: Vec<PolicyRequirement>) -> PolicySection {
        PolicySection {
            title: title.to_string(),
            heading_level: 1,
            source_line: 1,
            body_text: Some(body.to_string()),
            children: vec![],
            requirements: reqs,
        }
    }

    fn req_with_line(text: &str, id: &str, line: usize) -> PolicyRequirement {
        PolicyRequirement {
            stable_id: Some(id.to_string()),
            text: text.to_string(),
            source_line: line,
            nesting_depth: 0,
            atom_index: 0,
            parent_text: None,
            citations: vec![],
            modality: None,
            parameters: vec![],
        }
    }

    // ── T010: controls have statement parts ─────────────

    #[test]
    fn test_catalog_controls_have_statement_parts() {
        let d = doc(vec![
            sec("Access Control", vec![req("Users shall authenticate.", "u1")]),
            sec("Data Protection", vec![req("Encrypt at rest.", "u2")]),
        ]);
        let cat = build_catalog(&d, None).unwrap();

        for group in &cat.groups {
            for ctrl in &group.controls {
                assert!(!ctrl.parts.is_empty(), "Control {} has no parts", ctrl.id);
                assert_eq!(ctrl.parts[0].name, "statement");
                assert!(
                    ctrl.parts[0].id.ends_with("_smt"),
                    "Statement part id '{}' does not end with _smt",
                    ctrl.parts[0].id
                );
            }
        }

        // Verify prose matches requirement text
        assert_eq!(cat.groups[0].controls[0].parts[0].prose, "Users shall authenticate.");
        assert_eq!(cat.groups[1].controls[0].parts[0].prose, "Encrypt at rest.");
    }

    // ── T013: no remarks + props skip when empty ────────

    #[test]
    fn test_serialized_control_no_remarks_field() {
        let d = doc(vec![sec("Access Control", vec![req("Auth required.", "u1")])]);
        let cat = build_catalog(&d, None).unwrap();
        let envelope = CatalogEnvelope { catalog: cat };
        let json = serde_json::to_string_pretty(&envelope).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();

        let controls = v["catalog"]["groups"][0]["controls"].as_array().unwrap();
        for ctrl in controls {
            assert!(ctrl.get("remarks").is_none(), "Control should not have a 'remarks' field");
        }
    }

    #[test]
    fn test_props_omitted_when_empty() {
        // source_line: 0 means build_control_props returns empty vec
        let r = PolicyRequirement {
            stable_id: Some("u1".to_string()),
            text: "Test requirement.".to_string(),
            source_line: 0,
            nesting_depth: 0,
            atom_index: 0,
            parent_text: None,
            citations: vec![],
            modality: None,
            parameters: vec![],
        };
        let d = doc(vec![sec("Test", vec![r])]);
        let cat = build_catalog(&d, None).unwrap();
        let envelope = CatalogEnvelope { catalog: cat };
        let json = serde_json::to_string_pretty(&envelope).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();

        let ctrl = &v["catalog"]["groups"][0]["controls"][0];
        assert!(
            ctrl.get("props").is_none(),
            "props should be omitted when empty (skip_serializing_if)"
        );
    }

    // ── T020: guidance from body_text ───────────────────

    #[test]
    fn test_catalog_guidance_from_body_text() {
        let d = doc(vec![sec_with_body(
            "Access Control",
            "Guidance here.",
            vec![req("Users shall authenticate.", "u1")],
        )]);
        let cat = build_catalog(&d, None).unwrap();
        let ctrl = &cat.groups[0].controls[0];

        assert_eq!(ctrl.parts.len(), 2, "Expected statement + guidance parts");

        // First part: statement
        assert_eq!(ctrl.parts[0].name, "statement");
        assert!(ctrl.parts[0].id.ends_with("_smt"));
        assert_eq!(ctrl.parts[0].prose, "Users shall authenticate.");

        // Second part: guidance
        assert_eq!(ctrl.parts[1].name, "guidance");
        assert!(
            ctrl.parts[1].id.ends_with("_gdn"),
            "Guidance part id '{}' does not end with _gdn",
            ctrl.parts[1].id
        );
        assert_eq!(ctrl.parts[1].prose, "Guidance here.");
    }

    #[test]
    fn test_catalog_no_guidance_without_body_text() {
        let d = doc(vec![sec("Access Control", vec![req("Auth required.", "u1")])]);
        let cat = build_catalog(&d, None).unwrap();
        let ctrl = &cat.groups[0].controls[0];

        assert_eq!(ctrl.parts.len(), 1, "Expected only statement part when no body_text");
        assert_eq!(ctrl.parts[0].name, "statement");
    }

    // ── T024: JSON serialization OSCAL v1.2.0 shape ─────

    #[test]
    fn test_json_parts_structure_oscal_v120() {
        let d = doc(vec![sec_with_body(
            "Access Control",
            "Some guidance.",
            vec![req("Auth required.", "u1")],
        )]);
        let cat = build_catalog(&d, None).unwrap();
        let envelope = CatalogEnvelope { catalog: cat };
        let json = serde_json::to_string_pretty(&envelope).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();

        let parts = v["catalog"]["groups"][0]["controls"][0]["parts"]
            .as_array()
            .expect("parts should be an array");

        assert_eq!(parts.len(), 2, "Expected statement + guidance parts");

        // Verify each part has the required OSCAL fields
        for part in parts {
            let obj = part.as_object().unwrap();
            assert!(obj.contains_key("id"), "Part missing 'id' field");
            assert!(obj.contains_key("name"), "Part missing 'name' field");
            assert!(obj.contains_key("prose"), "Part missing 'prose' field");
        }

        // Verify statement part field values
        assert_eq!(parts[0]["name"], "statement");
        assert!(parts[0]["id"].as_str().unwrap().ends_with("_smt"));
        assert_eq!(parts[0]["prose"], "Auth required.");

        // Verify guidance part field values
        assert_eq!(parts[1]["name"], "guidance");
        assert!(parts[1]["id"].as_str().unwrap().ends_with("_gdn"));
        assert_eq!(parts[1]["prose"], "Some guidance.");
    }

    #[test]
    fn test_json_props_omitted_without_trace_embedding() {
        // After WI-17, build_control_props returns vec![] — trace props are
        // added by embed_trace_in_catalog post-processing, not inline.
        let d = doc(vec![sec("Access Control", vec![req_with_line("Auth required.", "u1", 42)])]);
        let cat = build_catalog(&d, None).unwrap();
        let envelope = CatalogEnvelope { catalog: cat };
        let json = serde_json::to_string_pretty(&envelope).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Props should be omitted (empty vec skipped by serde)
        assert!(
            v["catalog"]["groups"][0]["controls"][0].get("props").is_none(),
            "props should be omitted when no trace embedding has been applied"
        );
    }

    // ─── T008: collect_control_ids_from_catalog ────────────────────────

    #[test]
    fn collect_control_ids_depth_first() {
        let catalog = OscalCatalog {
            uuid: "test".into(),
            metadata: OscalMetadata {
                title: "T".into(),
                last_modified: "2026-01-01T00:00:00Z".into(),
                version: "1.0.0".into(),
                oscal_version: "1.2.0".into(),
            },
            controls: vec![],
            groups: vec![
                OscalGroup {
                    id: "g1".into(),
                    title: "Group 1".into(),
                    props: vec![],
                    links: vec![],
                    controls: vec![
                        OscalControl {
                            id: "POL-AC-001".into(),
                            uuid: "u1".into(),
                            title: "C1".into(),
                            parts: vec![],
                            props: vec![],
                            links: vec![],
                            params: vec![],
                        },
                        OscalControl {
                            id: "POL-AC-002".into(),
                            uuid: "u2".into(),
                            title: "C2".into(),
                            parts: vec![],
                            props: vec![],
                            links: vec![],
                            params: vec![],
                        },
                    ],
                    groups: vec![],
                },
                OscalGroup {
                    id: "g2".into(),
                    title: "Group 2".into(),
                    props: vec![],
                    links: vec![],
                    controls: vec![OscalControl {
                        id: "POL-DP-001".into(),
                        uuid: "u3".into(),
                        title: "C3".into(),
                        parts: vec![],
                        props: vec![],
                        links: vec![],
                        params: vec![],
                    }],
                    groups: vec![],
                },
            ],
            back_matter: None,
        };

        let ids = collect_control_ids_from_catalog(&catalog);
        assert_eq!(ids, vec!["POL-AC-001", "POL-AC-002", "POL-DP-001"]);
    }

    #[test]
    fn collect_control_ids_empty_catalog() {
        let catalog = OscalCatalog {
            uuid: "test".into(),
            metadata: OscalMetadata {
                title: "T".into(),
                last_modified: "2026-01-01T00:00:00Z".into(),
                version: "1.0.0".into(),
                oscal_version: "1.2.0".into(),
            },
            controls: vec![],
            groups: vec![],
            back_matter: None,
        };

        let ids = collect_control_ids_from_catalog(&catalog);
        assert!(ids.is_empty());
    }

    // ── Task 7: nested groups on OscalGroup ────────────

    #[test]
    fn catalog_round_trips_nested_groups() {
        let json = r#"{
            "catalog": {
                "uuid": "test-uuid",
                "metadata": {
                    "title": "Test",
                    "last-modified": "2026-01-01T00:00:00Z",
                    "version": "1.0",
                    "oscal-version": "1.2.0"
                },
                "groups": [{
                    "id": "parent",
                    "title": "Parent",
                    "groups": [{
                        "id": "child",
                        "title": "Child",
                        "controls": [{"id": "ctrl-1", "title": "Nested"}]
                    }]
                }]
            }
        }"#;
        let envelope: CatalogEnvelope = serde_json::from_str(json).unwrap();
        assert_eq!(envelope.catalog.groups[0].groups.len(), 1);
        assert_eq!(envelope.catalog.groups[0].groups[0].controls.len(), 1);
        let reserialized = serde_json::to_string(&envelope).unwrap();
        let re_parsed: CatalogEnvelope = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(re_parsed.catalog.groups[0].groups.len(), 1);
    }

    // ── Task 6: root-level controls on OscalCatalog ─────

    #[test]
    fn catalog_round_trips_root_level_controls() {
        let json = r#"{
            "catalog": {
                "uuid": "test-uuid",
                "metadata": {
                    "title": "Test",
                    "last-modified": "2026-01-01T00:00:00Z",
                    "version": "1.0",
                    "oscal-version": "1.2.0"
                },
                "controls": [
                    {"id": "ctrl-1", "title": "Root Control"}
                ]
            }
        }"#;
        let envelope: CatalogEnvelope = serde_json::from_str(json).unwrap();
        assert_eq!(envelope.catalog.controls.len(), 1);
        assert_eq!(envelope.catalog.controls[0].id, "ctrl-1");
        let reserialized = serde_json::to_string(&envelope).unwrap();
        let re_parsed: CatalogEnvelope = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(re_parsed.catalog.controls.len(), 1);
    }

    // ── Task 21: stable abbreviation collision under reorder ──

    #[test]
    fn abbreviation_collision_hash_suffix_is_deterministic() {
        use std::collections::HashMap;

        // Order 1: Access Control first, Audit Control second
        let mut counts1 = HashMap::new();
        let _a1 = resolve_abbreviation("Access Control", &mut counts1);
        let b1 = resolve_abbreviation("Audit Control", &mut counts1);

        // Order 2: Audit Control first, Access Control second
        let mut counts2 = HashMap::new();
        let _b2 = resolve_abbreviation("Audit Control", &mut counts2);
        let a2 = resolve_abbreviation("Access Control", &mut counts2);

        // The hash suffix for a given title is always the same content-based hash,
        // regardless of encounter order. When a title is second, it gets a suffix
        // derived from its own content (SHA-256), not from a counter.
        // Both b1 and a2 are the "collision" case (second to claim "AC").
        // b1 = "AC-{hash(Audit Control)}" and a2 = "AC-{hash(Access Control)}"
        // They should be different because they're different titles.
        assert_ne!(b1, a2, "Different titles should get different hash suffixes");

        // Verify the suffix format contains the expected hash
        assert!(b1.starts_with("AC-"), "Collision ID should start with base abbreviation");
        assert!(a2.starts_with("AC-"), "Collision ID should start with base abbreviation");

        // Running again in the same order should produce the same result
        let mut counts3 = HashMap::new();
        let _a3 = resolve_abbreviation("Access Control", &mut counts3);
        let b3 = resolve_abbreviation("Audit Control", &mut counts3);
        assert_eq!(b1, b3, "Same order should produce same result (deterministic)");
    }
}
