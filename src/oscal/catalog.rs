//! OSCAL Catalog builder: maps [`PolicyDocument`] to OSCAL Catalog JSON.
//!
//! Converts the domain model (`PolicyDocument` → `PolicySection` →
//! `PolicyRequirement`) to OSCAL Catalog structures (`OscalCatalog` →
//! `OscalGroup` → `OscalControl`).

use std::collections::HashMap;

use serde::Serialize;
use tracing::debug;

use crate::error::ForgeError;
use crate::model::{PolicyDocument, PolicyRequirement, PolicySection};

// ─── OSCAL Structs ──────────────────────────────────────────────────────

/// JSON envelope producing `{"catalog": {...}}` at the top level.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Serialize)]
pub struct CatalogEnvelope {
    /// The OSCAL Catalog.
    pub catalog: OscalCatalog,
}

/// OSCAL Catalog root structure.
///
/// Metadata and UUID are placeholders — populated by WI-11.
#[derive(Debug, Clone, Serialize)]
pub struct OscalCatalog {
    /// Placeholder UUID.
    pub uuid: String,
    /// Placeholder metadata (WI-11).
    pub metadata: OscalMetadata,
    /// Groups mapped from `PolicySection`s.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<OscalGroup>,
}

/// OSCAL Group mapped from a [`PolicySection`].
#[derive(Debug, Clone, Serialize)]
pub struct OscalGroup {
    /// Slugified section title (e.g., `"access-control"`).
    pub id: String,
    /// Section title verbatim.
    pub title: String,
    /// Controls mapped from requirements.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub controls: Vec<OscalControl>,
}

/// OSCAL Control mapped from a [`PolicyRequirement`].
#[derive(Debug, Clone, Serialize)]
pub struct OscalControl {
    /// Control ID following `POL-{ABBR}-{NNN}` pattern.
    pub id: String,
    /// UUID copied from `PolicyRequirement.stable_id`.
    pub uuid: String,
    /// Derived title (first sentence, 120-char cap).
    pub title: String,
}

/// Placeholder metadata — fully implemented in WI-11.
#[derive(Debug, Clone, Serialize)]
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

// ─── Builder ────────────────────────────────────────────────────────────

/// Build an OSCAL Catalog from a [`PolicyDocument`].
///
/// Pure function: reads domain model, produces OSCAL struct.
///
/// # Errors
///
/// Returns [`ForgeError::CatalogBuild`] if any
/// `PolicyRequirement.stable_id` is `None`.
pub fn build_catalog(document: &PolicyDocument) -> Result<OscalCatalog, ForgeError> {
    let mut group_id_counts: HashMap<String, usize> = HashMap::new();
    let mut abbrev_counts: HashMap<String, usize> = HashMap::new();
    let mut groups = Vec::new();

    for (idx, section) in document.sections.iter().enumerate() {
        let group_id = resolve_group_id(&section.title, idx, &mut group_id_counts);

        let abbreviation = resolve_abbreviation(&section.title, &mut abbrev_counts);

        let requirements = collect_requirements(section);
        let mut controls = Vec::with_capacity(requirements.len());

        for (req_idx, req) in requirements.iter().enumerate() {
            let stable_id = req.stable_id.as_ref().ok_or_else(|| {
                let preview: String = req.text.chars().take(60).collect();
                ForgeError::CatalogBuild(format!(
                    "Requirement missing stable_id in section '{}': '{preview}'",
                    section.title,
                ))
            })?;

            controls.push(OscalControl {
                id: generate_control_id(&abbreviation, req_idx, "POL"),
                uuid: stable_id.clone(),
                title: derive_control_title(&req.text),
            });
        }

        groups.push(OscalGroup { id: group_id, title: section.title.clone(), controls });
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
        groups,
    })
}

/// Resolve a group ID with collision tracking.
fn resolve_group_id(title: &str, index: usize, counts: &mut HashMap<String, usize>) -> String {
    let base = generate_group_id(title);
    if base.is_empty() {
        return format!("group-{index}");
    }
    let count = counts.entry(base.clone()).or_insert(0);
    *count += 1;
    if *count == 1 { base } else { format!("{base}-{count}") }
}

/// Resolve an abbreviation with collision tracking.
fn resolve_abbreviation(title: &str, counts: &mut HashMap<String, usize>) -> String {
    let base = generate_section_abbreviation(title);
    let count = counts.entry(base.clone()).or_insert(0);
    *count += 1;
    if *count == 1 {
        base
    } else {
        debug!(
            abbreviation = %base,
            count = *count,
            section = %title,
            "Abbreviation collision resolved"
        );
        format!("{base}{count}")
    }
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
        let cat = build_catalog(&d).unwrap();
        assert_eq!(cat.groups.len(), 3);
        assert_eq!(cat.groups[0].id, "access-control");
        assert_eq!(cat.groups[0].title, "Access Control");
        assert_eq!(cat.groups[1].id, "data-protection");
        assert_eq!(cat.groups[2].id, "incident-response");
    }

    #[test]
    fn catalog_zero_sections() {
        let d = doc(vec![]);
        let cat = build_catalog(&d).unwrap();
        assert!(cat.groups.is_empty());
    }

    #[test]
    fn catalog_section_empty_controls() {
        let d = doc(vec![sec("Empty", vec![])]);
        let cat = build_catalog(&d).unwrap();
        assert_eq!(cat.groups.len(), 1);
        assert!(cat.groups[0].controls.is_empty());
    }

    // ── T008a: group ID collision ───────────────────────

    #[test]
    fn catalog_group_id_collision() {
        let d = doc(vec![sec("Data Protection", vec![]), sec("Data Protection!", vec![])]);
        let cat = build_catalog(&d).unwrap();
        assert_eq!(cat.groups[0].id, "data-protection");
        assert_eq!(cat.groups[1].id, "data-protection-2");
    }

    #[test]
    fn catalog_empty_title_fallback() {
        let d = doc(vec![sec("", vec![])]);
        let cat = build_catalog(&d).unwrap();
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
        let cat = build_catalog(&d).unwrap();
        assert_eq!(cat.groups[0].controls[0].id, "POL-AC-001");
        assert_eq!(cat.groups[1].controls[0].id, "POL-AC2-001");
        assert_eq!(cat.groups[2].controls[0].id, "POL-AC3-001");
    }

    // ── T017: missing stable_id error ───────────────────

    #[test]
    fn catalog_missing_stable_id() {
        let d = doc(vec![sec("Test", vec![req_no_id("No ID requirement")])]);
        let err = build_catalog(&d).unwrap_err();
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
        let cat = build_catalog(&d).unwrap();

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
        let cat = build_catalog(&d).unwrap();
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
        assert_eq!(ctrl["uuid"], "uuid-1");
    }

    // ── T023: JSON round-trip ───────────────────────────

    #[test]
    fn json_round_trip() {
        let d = doc(vec![
            sec("Access Control", vec![req("Auth required.", "u1"), req("MFA required.", "u2")]),
            sec("Data Protection", vec![req("Encrypt data.", "u3")]),
        ]);
        let cat = build_catalog(&d).unwrap();
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
            groups: vec![],
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

        let cat = build_catalog(&d).unwrap();

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
        let cat2 = build_catalog(&d).unwrap();
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

        let cat = build_catalog(&d).unwrap();

        // Verify collision resolution
        assert_eq!(cat.groups[0].controls[0].id, "POL-AC-001");
        assert_eq!(cat.groups[1].controls[0].id, "POL-AC2-001");
        assert_eq!(cat.groups[2].controls[0].id, "POL-AC3-001");
        assert_eq!(cat.groups[3].controls[0].id, "POL-AC4-001");
        assert_eq!(cat.groups[4].controls[0].id, "POL-AC5-001");

        // All 10 IDs unique
        let mut ids: Vec<&str> =
            cat.groups.iter().flat_map(|g| g.controls.iter().map(|c| c.id.as_str())).collect();
        let count = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), count);
        assert_eq!(count, 10);
    }
}
