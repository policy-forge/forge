use uuid::Uuid;

use crate::model::{PolicyDocument, PolicySection};

/// Fixed namespace UUID for all FORGE content-addressed identifier generation.
///
/// This is a project-specific UUID v4 generated once and hardcoded to ensure
/// global uniqueness of FORGE's identifier namespace.
///
/// # Breaking Change Warning
///
/// Changing this value will change **ALL** generated `stable_id` values across
/// the entire FORGE ecosystem. Any change requires a documented migration path.
pub const FORGE_NAMESPACE_UUID: Uuid = Uuid::from_bytes([
    0x71, 0x43, 0xFD, 0xCB, 0xD7, 0xC4, 0x40, 0xC5, 0xBC, 0xA6, 0x51, 0xCD, 0x54, 0x83, 0xA1, 0x3B,
]);

/// Normalize text for stable ID generation.
///
/// Trims leading/trailing whitespace and collapses internal whitespace runs
/// to a single space. Uses Rust's [`str::split_whitespace`] which handles
/// Unicode whitespace characters (NBSP, em space, etc.), tabs, and newlines.
///
/// # Arguments
///
/// * `text` - Raw text to normalize
///
/// # Returns
///
/// Normalized string with single-space separation. Empty string if input
/// is empty or contains only whitespace.
///
/// # Examples
///
/// ```
/// use forge::uuid::normalize_for_hashing;
///
/// assert_eq!(normalize_for_hashing("  foo   bar  "), "foo bar");
/// assert_eq!(normalize_for_hashing("foo\t\nbar"), "foo bar");
/// assert_eq!(normalize_for_hashing("   "), "");
/// assert_eq!(normalize_for_hashing(""), "");
/// ```
#[tracing::instrument(level = "trace")]
pub fn normalize_for_hashing(text: &str) -> String {
    let mut result = String::new();
    for word in text.split_whitespace() {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(word);
    }
    result
}

/// Generate a deterministic UUID v5 from text content.
///
/// The text is normalized before hashing to ensure whitespace-insensitivity.
/// Uses [`FORGE_NAMESPACE_UUID`] as the namespace parameter.
///
/// This function accepts any string input, not just `PolicyRequirement` text,
/// enabling reuse for other content-addressed identifiers (PRD S-2).
///
/// # Arguments
///
/// * `text` - Raw text to generate UUID from (will be normalized internally)
///
/// # Returns
///
/// A deterministic [`Uuid`] (v5) derived from the normalized text.
///
/// # Determinism Guarantee
///
/// Same text produces the same UUID, always. Different substantive text produces
/// a different UUID. Whitespace-only differences are normalized away.
///
/// # Examples
///
/// ```
/// use forge::uuid::generate_stable_id;
///
/// let uuid1 = generate_stable_id("All users must use MFA");
/// let uuid2 = generate_stable_id("All users must use MFA");
/// assert_eq!(uuid1, uuid2);
///
/// // Whitespace-only changes produce the same UUID
/// let uuid3 = generate_stable_id("  All  users  must  use  MFA  ");
/// assert_eq!(uuid1, uuid3);
/// ```
#[tracing::instrument(level = "trace")]
pub fn generate_stable_id(text: &str) -> Uuid {
    let normalized = normalize_for_hashing(text);
    Uuid::new_v5(&FORGE_NAMESPACE_UUID, normalized.as_bytes())
}

/// Populate `stable_id` on all [`PolicyRequirement`]s in a [`PolicyDocument`].
///
/// Walks the full section tree recursively, generating a UUID v5 for each
/// requirement's text and setting `stable_id = Some(uuid.to_string())`.
///
/// After this function returns, no `PolicyRequirement` in the document will
/// have `stable_id = None`.
///
/// # Arguments
///
/// * `document` - Mutable reference to the `PolicyDocument` to process
///
/// # Examples
///
/// ```
/// use forge::uuid::assign_stable_ids;
/// use forge::model::{PolicyDocument, PolicySection, PolicyRequirement};
///
/// let mut doc = PolicyDocument {
///     sections: vec![PolicySection {
///         title: "Section 1".to_string(),
///         requirements: vec![PolicyRequirement {
///             text: "All users must use MFA".to_string(),
///             source_line: 1,
///             nesting_depth: 0,
///             stable_id: None,
///         }],
///         children: vec![],
///     }],
/// };
///
/// assign_stable_ids(&mut doc);
/// assert!(doc.sections[0].requirements[0].stable_id.is_some());
/// ```
#[tracing::instrument(level = "debug", skip(document))]
pub fn assign_stable_ids(document: &mut PolicyDocument) {
    for section in &mut document.sections {
        assign_stable_ids_to_section(section);
    }
}

fn assign_stable_ids_to_section(section: &mut PolicySection) {
    for requirement in &mut section.requirements {
        let normalized = normalize_for_hashing(&requirement.text);
        let uuid = Uuid::new_v5(&FORGE_NAMESPACE_UUID, normalized.as_bytes());
        tracing::debug!(
            normalized_text = %normalized,
            uuid = %uuid,
            "UUID generated"
        );
        requirement.stable_id = Some(uuid.to_string());
    }
    for child in &mut section.children {
        assign_stable_ids_to_section(child);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{PolicyDocument, PolicyRequirement, PolicySection};
    use uuid::Version;

    // === Phase 3: User Story 1 — Deterministic IDs Across Runs ===

    // T005: Basic normalization
    #[test]
    fn test_normalize_for_hashing_basic() {
        assert_eq!(normalize_for_hashing("  foo   bar  "), "foo bar");
        assert_eq!(normalize_for_hashing("already clean"), "already clean");
        // Idempotency: normalizing an already-normalized string returns the same result
        let normalized = normalize_for_hashing("  foo   bar  ");
        assert_eq!(normalize_for_hashing(&normalized), normalized);
    }

    // T006: Determinism (AC-1, M-4)
    #[test]
    fn test_generate_stable_id_determinism() {
        let text = "All users must use multi-factor authentication";
        let uuid1 = generate_stable_id(text);
        let uuid2 = generate_stable_id(text);
        assert_eq!(uuid1, uuid2);
    }

    // T007: UUID v5 format (AC-5)
    #[test]
    fn test_generate_stable_id_uuid_v5_format() {
        let uuid = generate_stable_id("All users must use multi-factor authentication");
        assert_eq!(uuid.get_version(), Some(Version::Sha1));
        assert_eq!(uuid.get_variant(), uuid::Variant::RFC4122);
    }

    // T011: All requirements populated (AC-4, M-3)
    #[test]
    fn test_assign_stable_ids_all_populated() {
        let mut doc = PolicyDocument {
            sections: vec![
                PolicySection {
                    title: "Section 1".to_string(),
                    requirements: vec![
                        make_req("Requirement A", 1, 0),
                        make_req("Requirement B", 2, 0),
                    ],
                    children: vec![PolicySection {
                        title: "Section 1.1".to_string(),
                        requirements: vec![make_req("Requirement C", 3, 1)],
                        children: vec![],
                    }],
                },
                PolicySection {
                    title: "Section 2".to_string(),
                    requirements: vec![],
                    children: vec![],
                },
            ],
        };

        assign_stable_ids(&mut doc);

        // All requirements should have stable_id populated
        assert!(doc.sections[0].requirements[0].stable_id.is_some());
        assert!(doc.sections[0].requirements[1].stable_id.is_some());
        assert!(doc.sections[0].children[0].requirements[0].stable_id.is_some());
    }

    // T012: Nested sections (EC-5)
    #[test]
    fn test_assign_stable_ids_nested_sections() {
        let mut doc = PolicyDocument {
            sections: vec![PolicySection {
                title: "Level 0".to_string(),
                requirements: vec![make_req("L0 Req", 1, 0)],
                children: vec![PolicySection {
                    title: "Level 1".to_string(),
                    requirements: vec![make_req("L1 Req", 2, 1)],
                    children: vec![PolicySection {
                        title: "Level 2".to_string(),
                        requirements: vec![make_req("L2 Req", 3, 2)],
                        children: vec![],
                    }],
                }],
            }],
        };

        assign_stable_ids(&mut doc);

        // Level 0
        assert!(doc.sections[0].requirements[0].stable_id.is_some());
        // Level 1
        assert!(doc.sections[0].children[0].requirements[0].stable_id.is_some());
        // Level 2
        assert!(doc.sections[0].children[0].children[0].requirements[0].stable_id.is_some());
    }

    // === Phase 4: User Story 2 — Whitespace Normalization ===

    // T015: Extra spaces (AC-2)
    #[test]
    fn test_whitespace_resilience_extra_spaces() {
        let clean = generate_stable_id("Users must change passwords every 90 days");
        let messy = generate_stable_id("  Users  must  change  passwords  every  90  days  ");
        assert_eq!(clean, messy);
    }

    // T016: Trailing newline (AC-2)
    #[test]
    fn test_whitespace_resilience_trailing_newline() {
        let without = generate_stable_id("Requirement text");
        let with_newline = generate_stable_id("Requirement text\n");
        assert_eq!(without, with_newline);
    }

    // T017: Mixed tabs and newlines (EC-2)
    #[test]
    fn test_whitespace_resilience_mixed_tabs_newlines() {
        let clean = generate_stable_id("foo bar");
        let mixed = generate_stable_id("foo\t\n\nbar");
        assert_eq!(clean, mixed);
    }

    // T018: Empty text (EC-1, SEC-1)
    #[test]
    fn test_edge_case_empty_text() {
        let uuid = generate_stable_id("");
        assert_eq!(uuid.get_version(), Some(Version::Sha1));
        assert_eq!(normalize_for_hashing(""), "");
    }

    // T019: Whitespace-only input (EC-1)
    #[test]
    fn test_edge_case_whitespace_only() {
        let empty_uuid = generate_stable_id("");
        let whitespace_uuid = generate_stable_id("   ");
        assert_eq!(empty_uuid, whitespace_uuid);
        assert_eq!(normalize_for_hashing("   "), "");
    }

    // T020: Unicode whitespace (EC-3, SEC-2)
    #[test]
    fn test_edge_case_unicode_whitespace() {
        // Non-breaking space (\u{00A0}) and em space (\u{2003})
        let ascii = generate_stable_id("foo bar");
        let nbsp = generate_stable_id("foo\u{00A0}bar");
        let em_space = generate_stable_id("foo\u{2003}bar");
        assert_eq!(ascii, nbsp);
        assert_eq!(ascii, em_space);
    }

    // === Phase 5: User Story 3 — Substantive Change Detection ===

    // T022: Substantive change (AC-3, M-5)
    #[test]
    fn test_substantive_change_different_uuid() {
        let original = generate_stable_id("All users must use MFA");
        let changed = generate_stable_id("All administrators must use MFA");
        assert_ne!(original, changed);
    }

    // T023: Numeric change (AC-3)
    #[test]
    fn test_substantive_change_numeric() {
        let original = generate_stable_id("Passwords must be at least 8 characters");
        let changed = generate_stable_id("Passwords must be at least 12 characters");
        assert_ne!(original, changed);
    }

    // T024: No false collisions (EC-4)
    #[test]
    fn test_no_false_collisions() {
        let texts = [
            "All users must use multi-factor authentication",
            "Passwords must be at least 12 characters",
            "Systems must be patched within 30 days",
            "Access logs must be retained for 90 days",
            "Encryption must use AES-256",
            "Network traffic must be monitored continuously",
        ];
        let uuids: Vec<Uuid> = texts.iter().map(|t| generate_stable_id(t)).collect();
        // Every UUID should be unique
        for i in 0..uuids.len() {
            for j in (i + 1)..uuids.len() {
                assert_ne!(uuids[i], uuids[j], "Collision between texts[{i}] and texts[{j}]");
            }
        }
    }

    // === Helper ===

    fn make_req(text: &str, line: usize, depth: u8) -> PolicyRequirement {
        PolicyRequirement {
            text: text.to_string(),
            source_line: line,
            nesting_depth: depth,
            stable_id: None,
        }
    }
}
