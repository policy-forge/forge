//! Modality detection enrichment pass (WI-33).
//!
//! Classifies each [`PolicyRequirement`] as normative (mandatory obligation)
//! or advisory (recommendation/option) by scanning its text for RFC 2119
//! modal verbs using word-boundary-anchored, case-insensitive regex patterns.
//!
//! The two static patterns are compiled exactly once via [`std::sync::LazyLock`]
//! (SEC-4, SEC-3: no nested quantifiers, no unbounded repetition).

use std::sync::LazyLock;

use regex::Regex;
use tracing::{debug, warn};

use crate::ForgeError;
use crate::model::{Modality, PolicyDocument, PolicyRequirement, PolicySection};

// ─── Static Patterns ────────────────────────────────────────────────────

/// Compiled regex for normative verb detection.
///
/// Matches: must, shall, will, required (case-insensitive, word-boundary-aware).
/// Pattern: `(?i)\b(must|shall|will|required)\b`
static NORMATIVE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(must|shall|will|required)\b")
        .expect("NORMATIVE_PATTERN is a valid static regex")
});

/// Compiled regex for advisory verb detection.
///
/// Matches: should, may, recommended, optional (case-insensitive, word-boundary-aware).
/// Pattern: `(?i)\b(should|may|recommended|optional)\b`
static ADVISORY_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(should|may|recommended|optional)\b")
        .expect("ADVISORY_PATTERN is a valid static regex")
});

// ─── Types ───────────────────────────────────────────────────────────────

/// Result of modality detection for a single policy requirement.
///
/// Carries the classification plus diagnostic metadata for logging.
/// Not part of the public API — used internally by [`annotate_modalities`].
#[derive(Debug)]
pub struct ModalityResult {
    /// The detected modality classification.
    pub modality: Modality,
    /// Verb strings matched in the requirement text.
    /// Empty when `is_default` is `true` (no verbs found).
    /// Contains verbs from both categories when `has_conflict` is `true`.
    pub matched_verbs: Vec<String>,
    /// `true` when no modality verb was detected; default (Normative) applied.
    pub is_default: bool,
    /// `true` when both normative and advisory verbs were detected.
    /// Normative classification applied (strongest wins).
    pub has_conflict: bool,
}

// ─── Public Functions ────────────────────────────────────────────────────

/// Detect modality for a single policy requirement.
///
/// Scans `requirement.text` against `NORMATIVE_PATTERN` and `ADVISORY_PATTERN`.
/// Returns a [`ModalityResult`] with the classification and diagnostic metadata.
///
/// Classification rules:
/// 1. Normative verbs only → `Modality::Normative`
/// 2. Advisory verbs only → `Modality::Advisory`
/// 3. Both found (conflict) → `Modality::Normative`, `has_conflict = true`
/// 4. Neither found (default) → `Modality::Normative`, `is_default = true`
#[must_use]
pub fn detect_modality(requirement: &PolicyRequirement) -> ModalityResult {
    let text = &requirement.text;

    let normative_hits: Vec<String> =
        NORMATIVE_PATTERN.captures_iter(text).map(|c| c[1].to_lowercase()).collect();

    let advisory_hits: Vec<String> =
        ADVISORY_PATTERN.captures_iter(text).map(|c| c[1].to_lowercase()).collect();

    let has_normative = !normative_hits.is_empty();
    let has_advisory = !advisory_hits.is_empty();

    match (has_normative, has_advisory) {
        (true, false) => ModalityResult {
            modality: Modality::Normative,
            matched_verbs: normative_hits,
            is_default: false,
            has_conflict: false,
        },
        (false, true) => ModalityResult {
            modality: Modality::Advisory,
            matched_verbs: advisory_hits,
            is_default: false,
            has_conflict: false,
        },
        (true, true) => {
            let mut combined = normative_hits;
            combined.extend(advisory_hits);
            ModalityResult {
                modality: Modality::Normative,
                matched_verbs: combined,
                is_default: false,
                has_conflict: true,
            }
        }
        (false, false) => ModalityResult {
            modality: Modality::Normative,
            matched_verbs: vec![],
            is_default: true,
            has_conflict: false,
        },
    }
}

/// Annotate all requirements in a document with modality classifications.
///
/// Iterates over every [`PolicyRequirement`] in every section (including nested
/// children), calls [`detect_modality`] on each, and sets `requirement.modality`.
///
/// Emits `tracing::warn!` for:
/// - Requirements with no modality verb detected (`is_default = true`)
/// - Requirements with conflicting normative/advisory verbs (`has_conflict = true`)
///
/// Emits `tracing::debug!` for matched verbs on every requirement.
///
/// # Errors
///
/// Currently infallible — modality detection cannot fail (static patterns).
/// Returns `Result` for pipeline consistency with other enrichment passes.
pub fn annotate_modalities(mut document: PolicyDocument) -> Result<PolicyDocument, ForgeError> {
    for section in &mut document.sections {
        annotate_section(section);
    }
    Ok(document)
}

/// Recursively annotate all requirements in a section and its children.
fn annotate_section(section: &mut PolicySection) {
    for req in &mut section.requirements {
        let result = detect_modality(req);

        if result.is_default || result.has_conflict {
            let preview: String = req.text.chars().take(120).collect();
            if result.is_default {
                warn!(text = preview, "No modality verb detected — defaulting to Normative");
            } else {
                warn!(
                    text = preview,
                    verbs = ?result.matched_verbs,
                    "Conflicting normative/advisory verbs — Normative wins"
                );
            }
        }

        debug!(
            verbs = ?result.matched_verbs,
            modality = ?result.modality,
            "Modality detected"
        );

        req.modality = Some(result.modality);
    }

    for child in &mut section.children {
        annotate_section(child);
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::PolicyRequirement;

    fn req(text: &str) -> PolicyRequirement {
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

    // ── T008: normative verbs → Modality::Normative ──────────────────

    #[test]
    fn detect_modality_normative_verbs() {
        for verb in &["must", "shall", "will", "required"] {
            let r = req(&format!("Organizations {verb} implement encryption."));
            let result = detect_modality(&r);
            assert_eq!(
                result.modality,
                Modality::Normative,
                "Expected Normative for verb '{verb}'"
            );
            assert!(
                !result.matched_verbs.is_empty(),
                "Expected non-empty matched_verbs for '{verb}'"
            );
            assert!(!result.is_default, "is_default should be false for '{verb}'");
            assert!(!result.has_conflict, "has_conflict should be false for '{verb}'");
        }
    }

    // ── T009: advisory verbs → Modality::Advisory ────────────────────

    #[test]
    fn detect_modality_advisory_verbs() {
        for verb in &["should", "may", "recommended", "optional"] {
            let r = req(&format!("Organizations {verb} implement logging."));
            let result = detect_modality(&r);
            assert_eq!(result.modality, Modality::Advisory, "Expected Advisory for verb '{verb}'");
            assert!(
                !result.matched_verbs.is_empty(),
                "Expected non-empty matched_verbs for '{verb}'"
            );
            assert!(!result.is_default, "is_default should be false for '{verb}'");
            assert!(!result.has_conflict, "has_conflict should be false for '{verb}'");
        }
    }

    // ── T010: case-insensitive matching ──────────────────────────────

    #[test]
    fn detect_modality_case_insensitive() {
        let cases = [
            ("MUST enable TLS", Modality::Normative),
            ("SHALL verify identity", Modality::Normative),
            ("Should implement logging", Modality::Advisory),
            ("MAY conduct reviews", Modality::Advisory),
        ];
        for (text, expected) in &cases {
            let r = req(text);
            let result = detect_modality(&r);
            assert_eq!(result.modality, *expected, "Expected {expected:?} for text '{text}'");
            assert!(!result.matched_verbs.is_empty(), "Should have matched verbs for '{text}'");
        }
    }

    // ── T011: word boundary prevents false positives ──────────────────

    #[test]
    fn detect_modality_word_boundary() {
        let non_matching = [
            "customize the configuration",
            "dismay at the results",
            "shouldering the burden",
            "willful neglect",
        ];
        for text in &non_matching {
            let r = req(text);
            let result = detect_modality(&r);
            assert!(
                result.matched_verbs.is_empty(),
                "Expected no verb matches for '{text}', got: {:?}",
                result.matched_verbs
            );
        }
    }

    // ── T012: annotate_modalities populates all requirements ──────────

    #[test]
    fn annotate_modalities_all_requirements_populated() {
        use crate::model::{DocumentMetadata, PolicySection};

        let section = PolicySection {
            title: "Test".to_string(),
            heading_level: 1,
            source_line: 1,
            body_text: None,
            children: vec![],
            requirements: vec![
                req("Organizations must implement MFA."),
                req("Systems should log failed attempts."),
                req("Encrypt all data at rest."),
            ],
        };

        let doc = PolicyDocument {
            id: "test".to_string(),
            metadata: DocumentMetadata::default(),
            sections: vec![section],
        };

        let annotated = annotate_modalities(doc).unwrap();
        for section in &annotated.sections {
            for req in &section.requirements {
                assert!(
                    req.modality.is_some(),
                    "All requirements should have Some(modality) after annotate_modalities"
                );
            }
        }
    }

    // ── T024: no verb → default Normative, is_default=true ───────────

    #[test]
    fn detect_modality_no_verb_defaults_normative() {
        let r = req("Encrypt all data at rest using approved algorithms.");
        let result = detect_modality(&r);
        assert_eq!(result.modality, Modality::Normative);
        assert!(result.is_default, "is_default should be true when no verb found");
        assert!(result.matched_verbs.is_empty(), "matched_verbs should be empty for default case");
        assert!(!result.has_conflict);
    }

    // ── T025: conflict → Normative wins, has_conflict=true ───────────

    #[test]
    fn detect_modality_conflict_normative_wins() {
        let r = req("Systems must implement and should log all events.");
        let result = detect_modality(&r);
        assert_eq!(result.modality, Modality::Normative);
        assert!(result.has_conflict, "has_conflict should be true when both verb types found");
        assert!(!result.is_default);
        assert!(
            result.matched_verbs.contains(&"must".to_string()),
            "matched_verbs should contain 'must'"
        );
        assert!(
            result.matched_verbs.contains(&"should".to_string()),
            "matched_verbs should contain 'should'"
        );
    }

    // ── T026: is_default and has_conflict are mutually exclusive ─────

    #[test]
    fn invariant_is_default_and_has_conflict_mutually_exclusive() {
        let inputs = [
            "Organizations must implement MFA.",
            "Systems should log attempts.",
            "Encrypt all data at rest.",
            "Systems must implement and should log.",
            "MUST enable TLS.",
            "MAY conduct reviews.",
            "customize the configuration",
        ];
        for text in &inputs {
            let r = req(text);
            let result = detect_modality(&r);
            assert!(
                !(result.is_default && result.has_conflict),
                "Invariant violated for '{text}': is_default={} and has_conflict={} are both true",
                result.is_default,
                result.has_conflict
            );
        }
    }

    // ── T036: negated normative verbs stay normative ──────────────────

    #[test]
    fn detect_modality_negated_normative_verbs() {
        let r = req("Organizations must not share passwords.");
        let result = detect_modality(&r);
        assert_eq!(result.modality, Modality::Normative);
        assert!(
            result.matched_verbs.contains(&"must".to_string()),
            "Expected 'must' in matched_verbs"
        );
        assert!(!result.is_default);
        assert!(!result.has_conflict);
    }

    // ── T037: "required" as adjective still classifies normative ─────

    #[test]
    fn detect_modality_required_as_adjective() {
        let r = req("Use the required configuration settings.");
        let result = detect_modality(&r);
        assert_eq!(result.modality, Modality::Normative);
        assert!(!result.matched_verbs.is_empty());
        assert!(!result.is_default);
    }

    // ── nested children sections are annotated recursively ────────────

    #[test]
    fn annotate_modalities_recurses_into_children() {
        use crate::model::{DocumentMetadata, PolicySection};

        let child_section = PolicySection {
            title: "Child".to_string(),
            heading_level: 2,
            source_line: 2,
            body_text: None,
            children: vec![],
            requirements: vec![req("Systems must encrypt data in transit.")],
        };

        let parent_section = PolicySection {
            title: "Parent".to_string(),
            heading_level: 1,
            source_line: 1,
            body_text: None,
            children: vec![child_section],
            requirements: vec![],
        };

        let doc = PolicyDocument {
            id: "test".to_string(),
            metadata: DocumentMetadata::default(),
            sections: vec![parent_section],
        };

        let annotated = annotate_modalities(doc).unwrap();
        for section in &annotated.sections {
            for child in &section.children {
                for r in &child.requirements {
                    assert!(
                        r.modality.is_some(),
                        "Requirements in nested children should be annotated"
                    );
                }
            }
        }
    }
}
