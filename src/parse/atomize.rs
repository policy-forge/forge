//! Requirement atomization — splits compound policy statements into atomic requirements.
//!
//! This module implements heuristic-based splitting of compound policy statements
//! (e.g., "Systems must enforce MFA and must require complex passwords") into
//! individual atomic requirements suitable for independent OSCAL control mapping.

use std::sync::LazyLock;

use regex::Regex;
use sha2::{Digest, Sha256};

use tracing::{debug, warn};

use crate::ForgeError;
use crate::model::{PolicyDocument, PolicyRequirement, PolicySection};

/// Maximum number of atomic requirements that can be produced from a single compound statement.
/// If exceeded, the statement is preserved as-is and a warning is logged (SEC-5, FR-010).
const MAX_SPLITS_PER_REQUIREMENT: usize = 50;

/// Compiled regex pattern for detecting conjunction + normative verb boundaries.
/// Pattern: `\b(and|or)\s+(must|shall|should|will)\b` (case-sensitive).
static SPLIT_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(and|or)\s+(must|shall|should|will)\b").expect("SPLIT_PATTERN regex is valid")
});

/// Compiled regex pattern for finding the first normative verb in a statement.
static FIRST_VERB_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(must|shall|should|will)\b").expect("FIRST_VERB_PATTERN regex is valid")
});

/// Result of atomizing a single policy requirement.
#[derive(Debug, Clone, PartialEq)]
pub struct AtomizationResult {
    /// The atomic requirements produced (1 if already atomic, N if split).
    pub requirements: Vec<PolicyRequirement>,
    /// Whether the original statement was split.
    pub was_split: bool,
    /// The original compound text (if split).
    pub original_text: Option<String>,
}

/// Generate a preliminary stable ID for an atomic requirement.
///
/// Uses SHA-256 hash (hex-encoded) of `"{text}|{source_line}|{atom_index}"`.
/// Will be replaced by UUID v5 in WI-7.
///
/// # Examples
///
/// ```
/// use forge::parse::preliminary_id;
///
/// let id = preliminary_id("Systems must enforce MFA", 42, 0);
/// assert_eq!(id.len(), 64);
/// assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
///
/// // Deterministic
/// let id2 = preliminary_id("Systems must enforce MFA", 42, 0);
/// assert_eq!(id, id2);
/// ```
#[must_use]
pub fn preliminary_id(text: &str, source_line: usize, atom_index: usize) -> String {
    let input = format!("{text}|{source_line}|{atom_index}");
    let hash = Sha256::digest(input.as_bytes());
    format!("{hash:x}")
}

/// Extract the shared subject from a compound statement.
///
/// Returns the text before `first_verb_pos`, trimmed. Returns `None` if empty.
fn extract_subject(text: &str, first_verb_pos: usize) -> Option<String> {
    let subject = text[..first_verb_pos].trim();
    if subject.is_empty() { None } else { Some(subject.to_string()) }
}

/// Reconstruct a complete sentence by prepending the shared subject to a clause fragment.
///
/// If the clause already starts with the shared subject (case-insensitive), it is returned as-is.
fn reconstruct_clause(shared_subject: &str, clause: &str) -> String {
    let trimmed = clause.trim();
    if trimmed.to_lowercase().starts_with(&shared_subject.to_lowercase()) {
        trimmed.to_string()
    } else {
        format!("{shared_subject} {trimmed}")
    }
}

/// Atomize a single policy requirement.
///
/// Detects compound statements containing conjunctions ("and", "or") paired with
/// normative verbs ("must", "shall", "should", "will") and splits them into
/// individual atomic requirements.
///
/// # Errors
///
/// Returns `ForgeError::Parse` if regex matching fails.
///
/// # Panics
///
/// Panics if the regex engine returns a capture match without group 0 or group 2,
/// which cannot happen with the `SPLIT_PATTERN` regex.
///
/// # Examples
///
/// ```
/// use forge::model::PolicyRequirement;
/// use forge::parse::atomize_requirement;
///
/// let req = PolicyRequirement {
///     stable_id: None,
///     text: "Systems must enforce MFA and must require complex passwords".to_string(),
///     source_line: 42,
///     nesting_depth: 0,
///     atom_index: 0,
///     parent_text: None,
/// };
///
/// let result = atomize_requirement(&req).unwrap();
/// assert!(result.was_split);
/// assert_eq!(result.requirements.len(), 2);
/// ```
pub fn atomize_requirement(
    requirement: &PolicyRequirement,
) -> Result<AtomizationResult, ForgeError> {
    let text = &requirement.text;

    // Empty or whitespace-only text: preserve as-is (EC-7)
    if text.trim().is_empty() {
        return Ok(AtomizationResult {
            requirements: vec![PolicyRequirement {
                stable_id: Some(preliminary_id(text, requirement.source_line, 0)),
                text: text.clone(),
                source_line: requirement.source_line,
                nesting_depth: requirement.nesting_depth,
                atom_index: 0,
                parent_text: None,
            }],
            was_split: false,
            original_text: None,
        });
    }

    // Find all conjunction + normative verb boundaries
    let match_count = SPLIT_PATTERN.find_iter(text).count();

    if match_count == 0 {
        // No compound pattern detected: preserve as-is (FR-003, M-3)
        return Ok(AtomizationResult {
            requirements: vec![PolicyRequirement {
                stable_id: Some(preliminary_id(text, requirement.source_line, 0)),
                text: text.clone(),
                source_line: requirement.source_line,
                nesting_depth: requirement.nesting_depth,
                atom_index: 0,
                parent_text: None,
            }],
            was_split: false,
            original_text: None,
        });
    }

    // Check max split count (SEC-5, FR-010, EC-9)
    let split_count = match_count + 1;
    if split_count > MAX_SPLITS_PER_REQUIREMENT {
        warn!(
            source_line = requirement.source_line,
            split_count = split_count,
            max = MAX_SPLITS_PER_REQUIREMENT,
            "Requirement would produce too many splits; preserved as-is"
        );
        return Ok(AtomizationResult {
            requirements: vec![PolicyRequirement {
                stable_id: Some(preliminary_id(text, requirement.source_line, 0)),
                text: text.clone(),
                source_line: requirement.source_line,
                nesting_depth: requirement.nesting_depth,
                atom_index: 0,
                parent_text: None,
            }],
            was_split: false,
            original_text: None,
        });
    }

    // Find position of the first normative verb in the text to extract shared subject
    let shared_subject =
        FIRST_VERB_PATTERN.find(text).and_then(|m| extract_subject(text, m.start()));

    // Split text at conjunction + normative verb boundaries.
    // For each match like "and must", we take text before the conjunction as a clause,
    // then the next clause starts at the normative verb (capture group 2).
    let mut clauses = Vec::with_capacity(split_count);
    let mut last_end = 0;

    for cap in SPLIT_PATTERN.captures_iter(text) {
        let full_match = cap.get(0).unwrap();
        let verb = cap.get(2).unwrap();

        // Clause: text from last_end to start of conjunction
        let clause = text[last_end..full_match.start()].trim();
        if !clause.is_empty() {
            clauses.push(clause.to_string());
        }

        // Next clause starts at the normative verb
        last_end = verb.start();
    }

    // Final clause: from last verb position to end of text
    let final_clause = text[last_end..].trim();
    if !final_clause.is_empty() {
        clauses.push(final_clause.to_string());
    }

    // Reconstruct clauses with shared subject
    let mut requirements = Vec::with_capacity(clauses.len());
    for (index, clause) in clauses.iter().enumerate() {
        let full_text = if let Some(ref subject) = shared_subject {
            reconstruct_clause(subject, clause)
        } else {
            clause.clone()
        };

        requirements.push(PolicyRequirement {
            stable_id: Some(preliminary_id(&full_text, requirement.source_line, index)),
            text: full_text,
            source_line: requirement.source_line,
            nesting_depth: requirement.nesting_depth,
            atom_index: index,
            parent_text: Some(text.clone()),
        });
    }

    Ok(AtomizationResult { requirements, was_split: true, original_text: Some(text.clone()) })
}

/// Atomize all requirements in a `PolicyDocument`.
///
/// Replaces compound `PolicyRequirement`s with their atomic parts.
///
/// # Errors
///
/// Returns `ForgeError::Parse` if any individual requirement atomization fails.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use forge::model::{DocumentMetadata, PolicyDocument, PolicySection, PolicyRequirement};
/// use forge::parse::atomize_document;
///
/// let doc = PolicyDocument {
///     id: "test".to_string(),
///     metadata: DocumentMetadata {
///         title: "Test Policy".to_string(),
///         version: "0.0.0".to_string(),
///         author: None,
///         date: None,
///         source_path: PathBuf::from("test.md"),
///         content_hash: None,
///     },
///     sections: vec![PolicySection {
///         title: "Section 1".to_string(),
///         heading_level: 1,
///         source_line: 1,
///         body_text: None,
///         children: vec![],
///         requirements: vec![PolicyRequirement {
///             stable_id: None,
///             text: "Systems must enforce MFA and must require complex passwords".to_string(),
///             source_line: 1,
///             nesting_depth: 0,
///             atom_index: 0,
///             parent_text: None,
///         }],
///     }],
/// };
///
/// let result = atomize_document(&doc).unwrap();
/// assert_eq!(result.sections[0].requirements.len(), 2);
/// ```
pub fn atomize_document(document: &PolicyDocument) -> Result<PolicyDocument, ForgeError> {
    let mut new_sections = Vec::with_capacity(document.sections.len());
    let mut split_count: usize = 0;
    let mut preserved_count: usize = 0;

    for section in &document.sections {
        let mut new_requirements = Vec::new();

        for requirement in &section.requirements {
            let result = atomize_requirement(requirement)?;
            if result.was_split {
                split_count += 1;
            } else {
                preserved_count += 1;
            }
            new_requirements.extend(result.requirements);
        }

        new_sections.push(PolicySection {
            title: section.title.clone(),
            heading_level: section.heading_level,
            source_line: section.source_line,
            body_text: section.body_text.clone(),
            children: section.children.clone(),
            requirements: new_requirements,
        });
    }

    let total_after: usize = new_sections.iter().map(|s| s.requirements.len()).sum();
    debug!(
        total = total_after,
        split = split_count,
        preserved = preserved_count,
        "Atomization complete"
    );

    Ok(PolicyDocument {
        id: document.id.clone(),
        metadata: document.metadata.clone(),
        sections: new_sections,
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::model::DocumentMetadata;

    // Helper to create a PolicyRequirement for testing
    fn make_req(text: &str, source_line: usize) -> PolicyRequirement {
        PolicyRequirement {
            stable_id: None,
            text: text.to_string(),
            source_line,
            nesting_depth: 0,
            atom_index: 0,
            parent_text: None,
        }
    }

    fn test_metadata() -> DocumentMetadata {
        DocumentMetadata {
            title: "Test Policy".to_string(),
            version: "0.0.0".to_string(),
            author: None,
            date: None,
            source_path: PathBuf::from("test.md"),
            content_hash: None,
        }
    }

    fn make_section(title: &str, requirements: Vec<PolicyRequirement>) -> PolicySection {
        PolicySection {
            title: title.to_string(),
            heading_level: 1,
            source_line: 1,
            body_text: None,
            children: vec![],
            requirements,
        }
    }

    fn make_doc(sections: Vec<PolicySection>) -> PolicyDocument {
        PolicyDocument { id: "test".to_string(), metadata: test_metadata(), sections }
    }

    // ===================================================================
    // T009: US1 — Two-part compound splitting tests
    // ===================================================================

    #[test]
    fn compound_two_parts_and_must_ac1() {
        // AC-1: "Systems must enforce MFA and must require complex passwords"
        let req = make_req("Systems must enforce MFA and must require complex passwords", 42);
        let result = atomize_requirement(&req).unwrap();

        assert!(result.was_split);
        assert_eq!(result.requirements.len(), 2);
        assert_eq!(result.requirements[0].text, "Systems must enforce MFA");
        assert_eq!(result.requirements[1].text, "Systems must require complex passwords");
        // AC-7: source_line preserved
        assert_eq!(result.requirements[0].source_line, 42);
        assert_eq!(result.requirements[1].source_line, 42);
        // parent_text set
        assert_eq!(
            result.requirements[0].parent_text,
            Some("Systems must enforce MFA and must require complex passwords".to_string())
        );
        assert_eq!(result.requirements[1].parent_text, result.requirements[0].parent_text);
        // was_split and original_text
        assert_eq!(
            result.original_text,
            Some("Systems must enforce MFA and must require complex passwords".to_string())
        );
    }

    #[test]
    fn compound_two_parts_and_shall_ac2() {
        // AC-2: shared subject "The organization" with "shall"
        let req = make_req(
            "The organization shall review access logs and shall revoke inactive accounts within 30 days",
            10,
        );
        let result = atomize_requirement(&req).unwrap();

        assert!(result.was_split);
        assert_eq!(result.requirements.len(), 2);
        assert_eq!(result.requirements[0].text, "The organization shall review access logs");
        assert_eq!(
            result.requirements[1].text,
            "The organization shall revoke inactive accounts within 30 days"
        );
        assert_eq!(result.requirements[0].source_line, 10);
        assert_eq!(result.requirements[1].source_line, 10);
    }

    // ===================================================================
    // T010: US1 — Multi-conjunction, mixed conjunctions, "or" splitting
    // ===================================================================

    #[test]
    fn compound_three_parts_ac3() {
        // AC-3: Three-part with mixed "and"/"or"
        let req = make_req(
            "All employees must complete security training and must acknowledge the acceptable use policy or must request a waiver",
            5,
        );
        let result = atomize_requirement(&req).unwrap();

        assert!(result.was_split);
        assert_eq!(result.requirements.len(), 3);
        assert_eq!(result.requirements[0].text, "All employees must complete security training");
        assert_eq!(
            result.requirements[1].text,
            "All employees must acknowledge the acceptable use policy"
        );
        assert_eq!(result.requirements[2].text, "All employees must request a waiver");
        // Sequential atom_index
        assert_eq!(result.requirements[0].atom_index, 0);
        assert_eq!(result.requirements[1].atom_index, 1);
        assert_eq!(result.requirements[2].atom_index, 2);
    }

    #[test]
    fn compound_shall_verb_ec6() {
        // EC-6: "shall" works the same as "must"
        let req = make_req("The team shall document procedures and shall train all staff", 20);
        let result = atomize_requirement(&req).unwrap();

        assert!(result.was_split);
        assert_eq!(result.requirements.len(), 2);
        assert_eq!(result.requirements[0].text, "The team shall document procedures");
        assert_eq!(result.requirements[1].text, "The team shall train all staff");
    }

    #[test]
    fn compound_or_conjunction_ec2() {
        // EC-2: "or" conjunction with normative verbs
        let req =
            make_req("Systems must implement MFA or must use certificate-based authentication", 15);
        let result = atomize_requirement(&req).unwrap();

        assert!(result.was_split);
        assert_eq!(result.requirements.len(), 2);
        assert_eq!(result.requirements[0].text, "Systems must implement MFA");
        assert_eq!(
            result.requirements[1].text,
            "Systems must use certificate-based authentication"
        );
    }

    #[test]
    fn compound_mixed_conjunctions_fr008() {
        // FR-008/S-2: mixed "and" + "or" conjunctions
        let req = make_req(
            "Systems must implement logging and must enable alerts or must notify administrators",
            30,
        );
        let result = atomize_requirement(&req).unwrap();

        assert!(result.was_split);
        assert_eq!(result.requirements.len(), 3);
        assert_eq!(result.requirements[0].atom_index, 0);
        assert_eq!(result.requirements[1].atom_index, 1);
        assert_eq!(result.requirements[2].atom_index, 2);
    }

    // ===================================================================
    // T011: US1 — Max split count enforcement
    // ===================================================================

    #[test]
    fn max_split_count_exceeded_ec9() {
        // EC-9: >50 splits → preserved as-is
        let mut text = "Systems must do task0".to_string();
        for i in 1..=51 {
            text.push_str(&format!(" and must do task{i}"));
        }
        let req = make_req(&text, 99);
        let result = atomize_requirement(&req).unwrap();

        // 52 parts > 50 max → preserved as-is
        assert!(!result.was_split);
        assert_eq!(result.requirements.len(), 1);
        assert_eq!(result.requirements[0].text, text);
    }

    // ===================================================================
    // T015: US2 — Atomic statement preservation
    // ===================================================================

    #[test]
    fn atomic_statement_preserved_ac3() {
        // AC-3 (US2): "All systems must enforce MFA" → unchanged
        let req = make_req("All systems must enforce MFA", 10);
        let result = atomize_requirement(&req).unwrap();

        assert!(!result.was_split);
        assert_eq!(result.requirements.len(), 1);
        assert_eq!(result.requirements[0].text, "All systems must enforce MFA");
        assert_eq!(result.requirements[0].atom_index, 0);
        assert!(result.requirements[0].parent_text.is_none());
        assert!(result.original_text.is_none());
    }

    #[test]
    fn atomic_statement_preserved_ac4() {
        // AC-4 (US2): "Passwords must be at least 12 characters" → unchanged
        let req = make_req("Passwords must be at least 12 characters", 20);
        let result = atomize_requirement(&req).unwrap();

        assert!(!result.was_split);
        assert_eq!(result.requirements.len(), 1);
        assert_eq!(result.requirements[0].text, "Passwords must be at least 12 characters");
    }

    // ===================================================================
    // T016: US2 — Non-splitting edge cases
    // ===================================================================

    #[test]
    fn no_split_and_without_normative_verb_ec1() {
        // EC-1: "and" without normative verb after conjunction
        let req = make_req("Systems must encrypt and store data securely", 5);
        let result = atomize_requirement(&req).unwrap();

        assert!(!result.was_split);
        assert_eq!(result.requirements.len(), 1);
        assert_eq!(result.requirements[0].text, "Systems must encrypt and store data securely");
    }

    #[test]
    fn no_split_logging_and_monitoring_ec5() {
        // EC-5: "and" in non-splitting context
        let req = make_req("Systems must implement logging and monitoring", 6);
        let result = atomize_requirement(&req).unwrap();

        assert!(!result.was_split);
        assert_eq!(result.requirements.len(), 1);
        assert_eq!(result.requirements[0].text, "Systems must implement logging and monitoring");
    }

    #[test]
    fn empty_text_preserved_ec7() {
        // EC-7: empty text
        let req = make_req("", 1);
        let result = atomize_requirement(&req).unwrap();

        assert!(!result.was_split);
        assert_eq!(result.requirements.len(), 1);
        assert_eq!(result.requirements[0].text, "");
    }

    #[test]
    fn whitespace_only_preserved_ec7() {
        // EC-7: whitespace-only
        let req = make_req("   ", 1);
        let result = atomize_requirement(&req).unwrap();

        assert!(!result.was_split);
        assert_eq!(result.requirements.len(), 1);
        assert_eq!(result.requirements[0].text, "   ");
    }

    #[test]
    fn uppercase_normative_verbs_not_split_ec10() {
        // EC-10: case-sensitive matching — uppercase "MUST" not matched
        let req = make_req("Systems MUST enforce MFA and MUST require passwords", 7);
        let result = atomize_requirement(&req).unwrap();

        assert!(!result.was_split);
        assert_eq!(result.requirements.len(), 1);
    }

    // ===================================================================
    // T018: US3 — Preliminary ID determinism
    // ===================================================================

    #[test]
    fn preliminary_id_deterministic_ac6() {
        // AC-6: same input → same output
        let id1 = preliminary_id("Systems must enforce MFA", 42, 0);
        let id2 = preliminary_id("Systems must enforce MFA", 42, 0);
        assert_eq!(id1, id2);
    }

    #[test]
    fn preliminary_id_is_64_char_hex() {
        let id = preliminary_id("Systems must enforce MFA", 42, 0);
        assert_eq!(id.len(), 64);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn preliminary_id_deterministic_via_atomize_ac5() {
        // AC-5: atomize same statement twice → same IDs
        let req = make_req("Systems must enforce MFA and must require complex passwords", 42);
        let result1 = atomize_requirement(&req).unwrap();
        let result2 = atomize_requirement(&req).unwrap();

        assert_eq!(result1.requirements.len(), result2.requirements.len());
        for (r1, r2) in result1.requirements.iter().zip(result2.requirements.iter()) {
            assert_eq!(r1.stable_id, r2.stable_id);
        }
    }

    // ===================================================================
    // T019: US3 — Preliminary ID uniqueness
    // ===================================================================

    #[test]
    fn preliminary_id_different_atom_index() {
        let id1 = preliminary_id("text", 42, 0);
        let id2 = preliminary_id("text", 42, 1);
        assert_ne!(id1, id2);
    }

    #[test]
    fn preliminary_id_different_source_line() {
        let id1 = preliminary_id("text", 42, 0);
        let id2 = preliminary_id("text", 43, 0);
        assert_ne!(id1, id2);
    }

    #[test]
    fn compound_split_produces_unique_ids() {
        let req = make_req("Systems must enforce MFA and must require complex passwords", 42);
        let result = atomize_requirement(&req).unwrap();
        assert_eq!(result.requirements.len(), 2);
        assert_ne!(result.requirements[0].stable_id, result.requirements[1].stable_id);
    }

    // ===================================================================
    // T020: US3 — ID integration verification
    // ===================================================================

    #[test]
    fn all_requirements_have_nonempty_stable_id() {
        // Compound
        let req = make_req("Systems must enforce MFA and must require complex passwords", 42);
        let result = atomize_requirement(&req).unwrap();
        for r in &result.requirements {
            let id = r.stable_id.as_ref().expect("stable_id should be Some");
            assert!(!id.is_empty());
            assert_eq!(id.len(), 64);
        }

        // Atomic
        let req2 = make_req("All systems must enforce MFA", 10);
        let result2 = atomize_requirement(&req2).unwrap();
        let id2 = result2.requirements[0].stable_id.as_ref().expect("stable_id should be Some");
        assert!(!id2.is_empty());
        assert_eq!(id2.len(), 64);
    }

    #[test]
    fn non_split_requirement_uses_atom_index_zero() {
        let req = make_req("All systems must enforce MFA", 10);
        let result = atomize_requirement(&req).unwrap();

        let expected_id = preliminary_id("All systems must enforce MFA", 10, 0);
        assert_eq!(result.requirements[0].stable_id, Some(expected_id));
    }

    // ===================================================================
    // atomize_document tests (T013, T014 equivalent — unit level)
    // ===================================================================

    #[test]
    fn atomize_document_empty_ec8() {
        // EC-8: empty document → unchanged
        let doc = make_doc(vec![]);
        let result = atomize_document(&doc).unwrap();
        assert_eq!(result.sections.len(), 0);
        assert_eq!(result.total_requirements(), 0);
    }

    #[test]
    fn atomize_document_mixed_compound_and_atomic_ac8() {
        // AC-8: document with compound + atomic requirements
        let doc = make_doc(vec![make_section(
            "Access Control",
            vec![
                make_req("Systems must enforce MFA and must require complex passwords", 1),
                make_req("All systems must enforce MFA", 2),
            ],
        )]);

        let original_count = doc.total_requirements();
        let result = atomize_document(&doc).unwrap();

        // 2 from compound + 1 atomic = 3 total (was 2)
        assert_eq!(result.total_requirements(), 3);
        assert!(result.total_requirements() >= original_count);

        // Verify the split requirements
        let reqs = &result.sections[0].requirements;
        assert_eq!(reqs[0].text, "Systems must enforce MFA");
        assert_eq!(reqs[1].text, "Systems must require complex passwords");
        assert_eq!(reqs[2].text, "All systems must enforce MFA");

        // Source lines preserved
        assert_eq!(reqs[0].source_line, 1);
        assert_eq!(reqs[1].source_line, 1);
        assert_eq!(reqs[2].source_line, 2);

        // Atomic requirement unchanged
        assert!(reqs[2].parent_text.is_none());
    }

    #[test]
    fn atomize_document_preserves_section_structure() {
        let doc = make_doc(vec![
            make_section("S1", vec![make_req("Systems must enforce MFA", 1)]),
            make_section("S2", vec![make_req("Users must use passwords and must enable 2FA", 5)]),
        ]);

        let result = atomize_document(&doc).unwrap();
        assert_eq!(result.sections.len(), 2);
        assert_eq!(result.sections[0].title, "S1");
        assert_eq!(result.sections[1].title, "S2");
        assert_eq!(result.sections[0].requirements.len(), 1);
        assert_eq!(result.sections[1].requirements.len(), 2);
    }

    #[test]
    fn atomize_document_metadata_preserved() {
        let doc = make_doc(vec![]);
        let result = atomize_document(&doc).unwrap();
        assert_eq!(result.id, "test");
        assert_eq!(result.metadata.title, "Test Policy");
    }

    // ===================================================================
    // Helper function tests
    // ===================================================================

    #[test]
    fn extract_subject_returns_trimmed_subject() {
        let s = extract_subject("Systems must enforce MFA", 8);
        assert_eq!(s, Some("Systems".to_string()));
    }

    #[test]
    fn extract_subject_returns_none_for_empty() {
        let s = extract_subject("must enforce MFA", 0);
        assert_eq!(s, None);
    }

    #[test]
    fn extract_subject_trims_whitespace() {
        let s = extract_subject("  The organization  shall do X", 20);
        assert_eq!(s, Some("The organization".to_string()));
    }

    #[test]
    fn reconstruct_clause_prepends_subject() {
        let c = reconstruct_clause("Systems", "must enforce MFA");
        assert_eq!(c, "Systems must enforce MFA");
    }

    #[test]
    fn reconstruct_clause_no_duplication() {
        let c = reconstruct_clause("Systems", "Systems must enforce MFA");
        assert_eq!(c, "Systems must enforce MFA");
    }

    #[test]
    fn reconstruct_clause_case_insensitive_check() {
        let c = reconstruct_clause("Systems", "systems must enforce MFA");
        assert_eq!(c, "systems must enforce MFA");
    }

    // ===================================================================
    // Additional should/will verb tests
    // ===================================================================

    #[test]
    fn compound_should_verb() {
        let req = make_req("Teams should review code and should write tests", 15);
        let result = atomize_requirement(&req).unwrap();
        assert!(result.was_split);
        assert_eq!(result.requirements.len(), 2);
        assert_eq!(result.requirements[0].text, "Teams should review code");
        assert_eq!(result.requirements[1].text, "Teams should write tests");
    }

    #[test]
    fn compound_will_verb() {
        let req =
            make_req("The department will implement controls and will monitor compliance", 25);
        let result = atomize_requirement(&req).unwrap();
        assert!(result.was_split);
        assert_eq!(result.requirements.len(), 2);
        assert_eq!(result.requirements[0].text, "The department will implement controls");
        assert_eq!(result.requirements[1].text, "The department will monitor compliance");
    }

    // ===================================================================
    // T021: Adversarial ReDoS resistance tests (SC-007, SEC-4)
    // ===================================================================

    #[test]
    fn redos_long_repetitive_input() {
        // 10KB+ repetitive string — must complete without hanging
        let repeated = "Systems must do task ".repeat(500); // ~10KB
        let req = make_req(&repeated, 1);
        let start = std::time::Instant::now();
        let result = atomize_requirement(&req).unwrap();
        let elapsed = start.elapsed();

        // Must complete in under 1 second (regex crate linear-time guarantee)
        assert!(elapsed.as_secs() < 1, "ReDoS: took {:?}", elapsed);
        // This won't match "and must" or "or must" so should be preserved
        assert!(!result.was_split);
    }

    #[test]
    fn redos_deeply_nested_conjunctions() {
        // Many conjunction-verb pairs — linear time
        let mut text = "Systems must do task0".to_string();
        for i in 1..200 {
            text.push_str(&format!(" and must do task{i}"));
        }
        let req = make_req(&text, 1);
        let start = std::time::Instant::now();
        let result = atomize_requirement(&req).unwrap();
        let elapsed = start.elapsed();

        assert!(elapsed.as_secs() < 1, "ReDoS: took {:?}", elapsed);
        // 200 conjunction-verb pairs → 201 parts > MAX_SPLITS_PER_REQUIREMENT → preserved as-is
        assert!(!result.was_split);
    }

    #[test]
    fn redos_pathological_whitespace() {
        // Many spaces between conjunction and verb
        let text = format!("Systems must X and{}must Y", " ".repeat(1000));
        let req = make_req(&text, 1);
        let start = std::time::Instant::now();
        let result = atomize_requirement(&req).unwrap();
        let elapsed = start.elapsed();

        assert!(elapsed.as_secs() < 1, "ReDoS: took {:?}", elapsed);
        assert!(result.was_split);
        assert_eq!(result.requirements.len(), 2);
    }

    // ===================================================================
    // T022: Unicode edge case tests (SEC-4)
    // ===================================================================

    #[test]
    fn unicode_zero_width_space_between_and_must() {
        // Zero-width space (\u{200B}) between "and" and "must" — should NOT match regex
        let text = "Systems must X and\u{200B} must Y";
        let req = make_req(text, 1);
        let result = atomize_requirement(&req).unwrap();
        // Zero-width space breaks the \s+ match, so no split
        // (or it may still split if \s matches ZWS — either way, no panic)
        assert!(result.requirements.len() >= 1);
    }

    #[test]
    fn unicode_bidi_override_no_panic() {
        // Bidirectional override characters — must not panic
        let text = "Systems must enforce MFA \u{202E}and must\u{202C} require passwords";
        let req = make_req(text, 1);
        let result = atomize_requirement(&req);
        assert!(result.is_ok());
    }

    #[test]
    fn unicode_multibyte_chars_preserved() {
        // Multi-byte UTF-8 characters in requirement text
        let text = "Systèmes must enforce MFA and must require contrôle d'accès";
        let req = make_req(text, 1);
        let result = atomize_requirement(&req).unwrap();

        assert!(result.was_split);
        assert_eq!(result.requirements.len(), 2);
        // The accented characters should be preserved
        assert!(result.requirements[0].text.contains("Systèmes"));
        assert!(result.requirements[1].text.contains("contrôle d'accès"));
    }

    #[test]
    fn unicode_emoji_in_text() {
        // Emoji in requirement text — should not cause panic
        let text = "Systems 🔒 must enforce MFA and must require passwords 🔑";
        let req = make_req(text, 1);
        let result = atomize_requirement(&req).unwrap();
        assert!(result.was_split);
        assert_eq!(result.requirements.len(), 2);
    }

    // ===================================================================
    // T026: Targeted mutation-killing tests
    // ===================================================================

    #[test]
    fn split_count_is_matches_plus_one() {
        // Kills mutant: replace `match_count + 1` with `match_count * 1`
        // 50 conjunctions → match_count=50 → split_count should be 51 (>50 max) → rejected
        // With mutant (*1), split_count=50, which is NOT >50, so it would incorrectly split
        let mut text = "Systems must do task0".to_string();
        for i in 1..51 {
            text.push_str(&format!(" and must do task{i}"));
        }
        // 50 conjunctions → 51 parts → exceeds MAX_SPLITS_PER_REQUIREMENT (50)
        let req = make_req(&text, 1);
        let result = atomize_requirement(&req).unwrap();
        assert!(!result.was_split, "51 parts should exceed max of 50");
        assert_eq!(result.requirements.len(), 1);
    }

    #[test]
    fn exactly_max_splits_allowed() {
        // Kills mutant: replace `>` with `>=` in split_count > MAX_SPLITS_PER_REQUIREMENT
        // Exactly 50 splits (49 conjunctions + 1) should be allowed, not rejected
        let mut text = "Systems must do task0".to_string();
        for i in 1..50 {
            text.push_str(&format!(" and must do task{i}"));
        }
        // 49 conjunctions → 50 parts = exactly MAX_SPLITS_PER_REQUIREMENT → should split
        let req = make_req(&text, 1);
        let result = atomize_requirement(&req).unwrap();
        assert!(result.was_split);
        assert_eq!(result.requirements.len(), 50);
    }
}
