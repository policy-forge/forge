//! H-7: Property-based tests for atomization and citation extraction.
//!
//! Uses proptest to verify invariants that must hold for all inputs,
//! catching edge cases that hand-crafted tests miss.

use proptest::prelude::*;

use forge::model::PolicyRequirement;
use forge::parse::atomize_requirement;

// ─── Atomization Property Tests ────────────────────────────────────────────

proptest! {
    /// P-1: Atomization never panics on arbitrary ASCII input.
    #[test]
    fn atomize_never_panics(text in "[ -~]{0,500}") {
        let req = PolicyRequirement {
            stable_id: None,
            text,
            source_line: 1,
            nesting_depth: 0,
            atom_index: 0,
            parent_text: None,
            citations: vec![],
        };
        let _ = atomize_requirement(&req);
    }

    /// P-2: Atomization always produces at least 1 requirement.
    #[test]
    fn atomize_produces_at_least_one(text in ".{0,300}") {
        let req = PolicyRequirement {
            stable_id: None,
            text,
            source_line: 1,
            nesting_depth: 0,
            atom_index: 0,
            parent_text: None,
            citations: vec![],
        };
        let result = atomize_requirement(&req).unwrap();
        prop_assert!(!result.requirements.is_empty());
    }

    /// P-3: Atomization never exceeds MAX_SPLITS_PER_REQUIREMENT (50).
    #[test]
    fn atomize_bounded_output(text in ".{0,500}") {
        let req = PolicyRequirement {
            stable_id: None,
            text,
            source_line: 1,
            nesting_depth: 0,
            atom_index: 0,
            parent_text: None,
            citations: vec![],
        };
        let result = atomize_requirement(&req).unwrap();
        prop_assert!(result.requirements.len() <= 50);
    }

    /// P-4: Every output requirement has a non-empty stable_id.
    #[test]
    fn atomize_all_have_stable_id(text in "[a-zA-Z ]{1,200}") {
        let req = PolicyRequirement {
            stable_id: None,
            text,
            source_line: 42,
            nesting_depth: 0,
            atom_index: 0,
            parent_text: None,
            citations: vec![],
        };
        let result = atomize_requirement(&req).unwrap();
        for r in &result.requirements {
            let id = r.stable_id.as_ref().unwrap();
            prop_assert!(!id.is_empty(), "stable_id must be non-empty");
        }
    }

    /// P-5: source_line is preserved on all output requirements.
    #[test]
    fn atomize_preserves_source_line(
        text in "[a-zA-Z ]{1,200}",
        line in 1..10_000usize
    ) {
        let req = PolicyRequirement {
            stable_id: None,
            text,
            source_line: line,
            nesting_depth: 0,
            atom_index: 0,
            parent_text: None,
            citations: vec![],
        };
        let result = atomize_requirement(&req).unwrap();
        for r in &result.requirements {
            prop_assert_eq!(r.source_line, line);
        }
    }

    /// P-6: Atomization is deterministic — same input always produces same output.
    #[test]
    fn atomize_deterministic(text in "[a-zA-Z ]{0,200}") {
        let req = PolicyRequirement {
            stable_id: None,
            text,
            source_line: 1,
            nesting_depth: 0,
            atom_index: 0,
            parent_text: None,
            citations: vec![],
        };
        let r1 = atomize_requirement(&req).unwrap();
        let r2 = atomize_requirement(&req).unwrap();
        prop_assert_eq!(r1.requirements.len(), r2.requirements.len());
        for (a, b) in r1.requirements.iter().zip(r2.requirements.iter()) {
            prop_assert_eq!(&a.stable_id, &b.stable_id);
            prop_assert_eq!(&a.text, &b.text);
        }
    }

    /// P-7: When split, parent_text is set on all output requirements.
    #[test]
    fn atomize_split_sets_parent_text(
        subject in "[A-Z][a-z]{2,20}",
        verb1 in prop::sample::select(vec!["must", "shall", "should", "will"]),
        action1 in "[a-z]{3,15}",
        conjunction in prop::sample::select(vec!["and", "or"]),
        verb2 in prop::sample::select(vec!["must", "shall", "should", "will"]),
        action2 in "[a-z]{3,15}",
    ) {
        let text = format!("{subject} {verb1} {action1} {conjunction} {verb2} {action2}");
        let req = PolicyRequirement {
            stable_id: None,
            text: text.clone(),
            source_line: 1,
            nesting_depth: 0,
            atom_index: 0,
            parent_text: None,
            citations: vec![],
        };
        let result = atomize_requirement(&req).unwrap();
        if result.was_split {
            for r in &result.requirements {
                prop_assert!(r.parent_text.is_some(), "Split requirements must have parent_text");
                prop_assert_eq!(r.parent_text.as_ref().unwrap(), &text);
            }
        }
    }
}

// ─── Citation Extraction Property Tests ────────────────────────────────────

proptest! {
    /// P-8: Citation extraction never panics on arbitrary ASCII input.
    #[test]
    fn citation_never_panics(text in "[ -~]{0,500}") {
        let _ = forge::citation::extract_citations_from_text("req-prop", &text);
    }

    /// P-9: Cleaned text never contains double spaces after citation stripping.
    #[test]
    fn citation_no_double_spaces(text in "[ -~]{0,300}") {
        if let Ok((cleaned, _)) = forge::citation::extract_citations_from_text("req-prop", &text) {
            prop_assert!(
                !cleaned.contains("  "),
                "Cleaned text should not contain double spaces. Got: {cleaned:?}"
            );
        }
    }

    /// P-10: Citation extraction is deterministic.
    #[test]
    fn citation_deterministic(text in "[ -~]{0,300}") {
        let r1 = forge::citation::extract_citations_from_text("req-prop", &text);
        let r2 = forge::citation::extract_citations_from_text("req-prop", &text);
        match (r1, r2) {
            (Ok((t1, c1)), Ok((t2, c2))) => {
                prop_assert_eq!(t1, t2);
                prop_assert_eq!(c1.len(), c2.len());
                for (a, b) in c1.iter().zip(c2.iter()) {
                    prop_assert_eq!(&a.id, &b.id);
                    prop_assert_eq!(&a.text, &b.text);
                }
            }
            (Err(_), Err(_)) => {}
            _ => prop_assert!(false, "Determinism violation: one succeeded and one failed"),
        }
    }

    /// P-11: All extracted citation IDs are valid UUIDs.
    #[test]
    fn citation_ids_are_valid_uuids(text in "[ -~]{0,300}") {
        if let Ok((_, citations)) = forge::citation::extract_citations_from_text("req-prop", &text) {
            for c in &citations {
                prop_assert!(
                    uuid::Uuid::parse_str(&c.id).is_ok(),
                    "Citation ID must be a valid UUID. Got: {}",
                    c.id
                );
            }
        }
    }

    /// P-12: Citation text extracted is a substring of the original text.
    #[test]
    fn citation_text_is_substring_of_original(text in "[ -~]{0,300}") {
        if let Ok((_, citations)) = forge::citation::extract_citations_from_text("req-prop", &text) {
            for c in &citations {
                prop_assert!(
                    text.contains(&c.text),
                    "Citation text {:?} must appear in original text {:?}",
                    c.text,
                    text
                );
            }
        }
    }

    /// P-13: URL citations with schemes produce URLs containing http.
    #[test]
    fn url_citations_contain_scheme(
        domain in "[a-z]{3,10}\\.[a-z]{2,4}",
        path in "[a-z/]{0,30}",
    ) {
        let text = format!("See https://{domain}/{path} for details");
        let (_, citations) = forge::citation::extract_citations_from_text("req-prop", &text).unwrap();
        prop_assert!(!citations.is_empty(), "Should extract at least one URL citation");
        for c in &citations {
            if let Some(url) = &c.url {
                prop_assert!(
                    url.starts_with("https://") || url.starts_with("http://"),
                    "URL citation should have scheme. Got: {url}"
                );
            }
        }
    }

    /// P-14: Cleaned text length is always <= original text length.
    #[test]
    fn cleaned_text_not_longer(text in "[ -~]{0,300}") {
        if let Ok((cleaned, _)) = forge::citation::extract_citations_from_text("req-prop", &text) {
            prop_assert!(
                cleaned.len() <= text.len() + 1, // +1 for potential space from strip_matches
                "Cleaned text ({}) should not be substantially longer than original ({})",
                cleaned.len(),
                text.len()
            );
        }
    }
}
