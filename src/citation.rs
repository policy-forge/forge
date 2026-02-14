//! Citation extraction — detects and extracts citations from policy requirement text.
//!
//! Implements WI-8 (Citation and Reference Extraction) as a pipeline enrichment step.
//! Detects inline URLs, bibliographic references (NIST SP, ISO, RFC, FIPS),
//! scheme-less URLs (`www.` prefix), and internal cross-references (Section, Appendix, Table).
//!
//! # Pipeline Position
//!
//! WI-7 (UUID) → **WI-8 (Citations)** → WI-9 (Catalog) → WI-12 (Back Matter)
//!
//! # Design Decisions
//!
//! - **R-1**: Citation IDs use UUID v5 (deterministic, idempotent)
//! - **R-2**: No `validated` field — `back_matter` handles URL validation at OSCAL layer
//! - **R-3**: `&mut PolicyDocument` enrichment pattern (consistent with WI-7)
//! - **R-5**: Priority order: URL > scheme-less URL > bibliographic > cross-ref
//! - **R-8**: All patterns use `LazyLock<Regex>` (compiled once, RE2-style)
//! - **R-9**: Prose cleanup: strip → collapse whitespace → trim → normalize punctuation

use std::ops::Range;
use std::sync::LazyLock;

use regex::Regex;
use uuid::Uuid;

use crate::error::ForgeError;
use crate::model::{Citation, PolicyDocument, PolicySection};
use crate::uuid::FORGE_NAMESPACE_UUID;

// T010: URL pattern — matches http:// or https:// followed by non-whitespace, non-delimiter chars.
// SAFETY: static regex — panics only if regex literal is invalid
static URL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"https?://[^\s\)\]>,;]+").expect("URL regex must compile"));

// T016: Bibliographic pattern — NIST SP, ISO, RFC, FIPS with optional Rev and Section suffixes.
// SAFETY: static regex — panics only if regex literal is invalid
static BIBLIO_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:NIST\s+SP|ISO|RFC|FIPS)\s+[\d]+[-\w.]*(?:\s+Rev\.?\s*\d+)?(?:,?\s+Section\s+[\w.-]+)?")
        .expect("Bibliographic regex must compile")
});

// T026: Scheme-less URL pattern — matches www. prefix (R-7).
// SAFETY: static regex — panics only if regex literal is invalid
static SCHEMELESS_URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\bwww\.[^\s\)\]>,;]+").expect("Scheme-less URL regex must compile")
});

// T030: Cross-reference pattern — Section X.Y, Appendix A, Table N (SEC-4).
// SAFETY: static regex — panics only if regex literal is invalid
static CROSSREF_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:Section|Appendix|Table)\s+[\dA-Z]+(?:\.\d+)*\b")
        .expect("Cross-reference regex must compile")
});

/// Extract citations from all requirements in a `PolicyDocument`.
///
/// Walks the full section tree recursively. For each `PolicyRequirement`:
/// 1. Detects URL, bibliographic, and cross-reference patterns in `text`
/// 2. Creates `Citation` objects for each match
/// 3. Strips matched text from `text` and normalizes whitespace
/// 4. Populates the `citations` field with extracted citations
///
/// # Errors
///
/// Returns `ForgeError::Parse` if regex pattern compilation fails
/// (should not happen with static patterns).
#[tracing::instrument(level = "debug", skip(document))]
pub fn extract_citations(document: &mut PolicyDocument) -> Result<(), ForgeError> {
    for section in &mut document.sections {
        extract_citations_from_section(section)?;
    }
    Ok(())
}

/// Recursively process a section and its children for citation extraction.
fn extract_citations_from_section(section: &mut PolicySection) -> Result<(), ForgeError> {
    for req in &mut section.requirements {
        // Skip already-processed requirements (idempotency, S-3)
        if !req.citations.is_empty() {
            continue;
        }
        let requirement_id = req.stable_id.as_deref().unwrap_or_else(|| {
            tracing::warn!(
                source_line = req.source_line,
                "Requirement missing stable_id, citation IDs may collide"
            );
            ""
        });
        let (cleaned_text, citations) = extract_citations_from_text(requirement_id, &req.text)?;
        tracing::debug!(
            requirement_id = requirement_id,
            citation_count = citations.len(),
            "Extracted citations from requirement"
        );
        req.text = cleaned_text;
        req.citations = citations;
    }
    for child in &mut section.children {
        extract_citations_from_section(child)?;
    }
    Ok(())
}

/// Extract citations from a single requirement's text.
///
/// Lower-level function that performs pattern matching and text cleanup.
/// Returns the cleaned text and a vector of extracted citations.
///
/// # Arguments
///
/// * `requirement_id` - The `stable_id` of the source requirement
/// * `text` - The requirement text to scan for citations
///
/// # Returns
///
/// A tuple of `(cleaned_text, extracted_citations)`.
///
/// # Errors
///
/// Returns `ForgeError::Parse` if regex matching encounters an error.
pub fn extract_citations_from_text(
    requirement_id: &str,
    text: &str,
) -> Result<(String, Vec<Citation>), ForgeError> {
    let mut citations = Vec::new();
    let mut matched_ranges: Vec<Range<usize>> = Vec::new();

    // US1: URL matches (highest priority)
    for m in URL_REGEX.find_iter(text) {
        let raw = m.as_str();
        // Trim trailing sentence punctuation (periods are valid inside URLs but not at end)
        let url_text = raw.trim_end_matches('.').to_string();
        let trimmed_len = raw.len() - url_text.len();
        let citation_id = generate_citation_id(requirement_id, &url_text);
        citations.push(Citation {
            id: citation_id,
            text: url_text.clone(),
            url: Some(url_text),
            source_requirement_id: Some(requirement_id.to_string()),
        });
        matched_ranges.push(m.start()..(m.end() - trimmed_len));
    }

    // US3: Scheme-less URL matches (skip if overlapping with full URL matches)
    for m in SCHEMELESS_URL_REGEX.find_iter(text) {
        let range = m.start()..m.end();
        if overlaps_any(&range, &matched_ranges) {
            continue;
        }
        let raw = m.as_str();
        let url_text = raw.trim_end_matches('.').to_string();
        let trimmed_len = raw.len() - url_text.len();
        let citation_id = generate_citation_id(requirement_id, &url_text);
        citations.push(Citation {
            id: citation_id,
            text: url_text.clone(),
            url: Some(url_text),
            source_requirement_id: Some(requirement_id.to_string()),
        });
        matched_ranges.push(m.start()..(m.end() - trimmed_len));
    }

    // US2: Bibliographic matches (skip if overlapping with URL matches)
    for m in BIBLIO_REGEX.find_iter(text) {
        let range = m.start()..m.end();
        if overlaps_any(&range, &matched_ranges) {
            continue;
        }
        let ref_text = m.as_str().to_string();
        let citation_id = generate_citation_id(requirement_id, &ref_text);
        citations.push(Citation {
            id: citation_id,
            text: ref_text,
            url: None,
            source_requirement_id: Some(requirement_id.to_string()),
        });
        matched_ranges.push(range);
    }

    // US4: Cross-reference matches (skip if overlapping with any prior matches)
    for m in CROSSREF_REGEX.find_iter(text) {
        let range = m.start()..m.end();
        if overlaps_any(&range, &matched_ranges) {
            continue;
        }
        let ref_text = m.as_str().to_string();
        let citation_id = generate_citation_id(requirement_id, &ref_text);
        citations.push(Citation {
            id: citation_id,
            text: ref_text,
            url: None,
            source_requirement_id: Some(requirement_id.to_string()),
        });
        matched_ranges.push(range);
    }

    let cleaned = strip_matches(text, &matched_ranges);
    let cleaned = normalize_prose(&cleaned);

    Ok((cleaned, citations))
}

/// Generate a deterministic citation ID using UUID v5.
///
/// Uses `FORGE_NAMESPACE_UUID` namespace with input `"{requirement_id}:{citation_text}"`.
#[must_use]
pub fn generate_citation_id(requirement_id: &str, citation_text: &str) -> String {
    let input = format!("{requirement_id}:{citation_text}");
    Uuid::new_v5(&FORGE_NAMESPACE_UUID, input.as_bytes()).to_string()
}

/// Replace matched byte ranges with spaces, preserving surrounding text.
fn strip_matches(text: &str, ranges: &[Range<usize>]) -> String {
    if ranges.is_empty() {
        return text.to_string();
    }

    let mut sorted = ranges.to_vec();
    sorted.sort_by_key(|r| r.start);

    let mut result = String::with_capacity(text.len());
    let mut last_end = 0;

    for range in &sorted {
        if range.start > last_end {
            result.push_str(&text[last_end..range.start]);
        }
        result.push(' ');
        last_end = range.end;
    }
    if last_end < text.len() {
        result.push_str(&text[last_end..]);
    }

    result
}

/// Normalize prose after citation stripping: remove artifacts, collapse whitespace.
fn normalize_prose(text: &str) -> String {
    let mut result = text.to_string();

    // Remove orphaned parentheses left after URL extraction
    result = result.replace("( )", "");
    result = result.replace("()", "");

    // Collapse consecutive spaces
    while result.contains("  ") {
        result = result.replace("  ", " ");
    }

    // Normalize punctuation artifacts from stripping
    result = result.replace(", ,", ",");
    result = result.replace(",,", ",");
    result = result.replace(", .", ".");
    result = result.replace(",.", ".");
    result = result.replace(" ,", ",");
    result = result.replace(" .", ".");

    result.trim().to_string()
}

/// Check if a byte range overlaps with any existing matched ranges (R-5 priority).
fn overlaps_any(range: &Range<usize>, existing: &[Range<usize>]) -> bool {
    existing.iter().any(|r| range.start < r.end && r.start < range.end)
}

#[cfg(test)]
mod tests {
    use super::*;

    // === T009: US1 URL Extraction Tests ===

    // AC-1: Single URL extraction
    #[test]
    fn us1_single_url_extracted() {
        let (text, citations) = extract_citations_from_text(
            "req-1",
            "Access must comply with https://example.com/policy requirements",
        )
        .unwrap();

        assert_eq!(citations.len(), 1);
        assert_eq!(citations[0].url.as_deref(), Some("https://example.com/policy"));
        assert_eq!(citations[0].text, "https://example.com/policy");
        assert_eq!(citations[0].source_requirement_id.as_deref(), Some("req-1"));
        assert_eq!(text, "Access must comply with requirements");
    }

    // AC-2: Multiple URLs in one requirement
    #[test]
    fn us1_multiple_urls_extracted() {
        let (text, citations) =
            extract_citations_from_text("req-1", "See https://a.com and https://b.com for details")
                .unwrap();

        assert_eq!(citations.len(), 2);
        assert_eq!(citations[0].url.as_deref(), Some("https://a.com"));
        assert_eq!(citations[1].url.as_deref(), Some("https://b.com"));
        assert!(!text.contains("https://"));
        assert_eq!(text, "See and for details");
    }

    // EC-4: URL in parentheses extracted without parens
    #[test]
    fn us1_url_in_parentheses_extracted_without_parens() {
        let (text, citations) =
            extract_citations_from_text("req-1", "Requirements (https://example.com) apply")
                .unwrap();

        assert_eq!(citations.len(), 1);
        assert_eq!(citations[0].url.as_deref(), Some("https://example.com"));
        assert_eq!(text, "Requirements apply");
    }

    // EC-5: Duplicate URLs produce separate Citations
    #[test]
    fn us1_duplicate_urls_produce_separate_citations() {
        let (_, citations) = extract_citations_from_text(
            "req-1",
            "See https://example.com and also https://example.com",
        )
        .unwrap();

        assert_eq!(citations.len(), 2);
        assert_eq!(citations[0].url, citations[1].url);
    }

    // EC-1: No citations text unchanged
    #[test]
    fn us1_no_citations_text_unchanged() {
        let (text, citations) =
            extract_citations_from_text("req-1", "Users must authenticate before access").unwrap();

        assert!(citations.is_empty());
        assert_eq!(text, "Users must authenticate before access");
    }

    // EC-2: Whitespace normalization after stripping
    #[test]
    fn us1_whitespace_normalized_after_stripping() {
        let (text, _) =
            extract_citations_from_text("req-1", "Access  https://example.com  requirements")
                .unwrap();

        assert!(!text.contains("  "));
        assert_eq!(text, "Access requirements");
    }

    // HTTP URL (not just HTTPS)
    #[test]
    fn us1_http_url_extracted() {
        let (text, citations) =
            extract_citations_from_text("req-1", "See http://example.com for details").unwrap();

        assert_eq!(citations.len(), 1);
        assert_eq!(citations[0].url.as_deref(), Some("http://example.com"));
        assert_eq!(text, "See for details");
    }

    // Punctuation normalization after URL stripping
    #[test]
    fn us1_punctuation_normalized_after_stripping() {
        let (text, _) = extract_citations_from_text(
            "req-1",
            "Controls per https://example.com, and more requirements.",
        )
        .unwrap();

        assert!(!text.contains(", ,"));
        assert!(!text.contains(",,"));
        assert_eq!(text, "Controls per, and more requirements.");
    }

    // Trailing period not captured in URL
    #[test]
    fn us1_trailing_period_not_captured() {
        let (text, citations) =
            extract_citations_from_text("req-1", "See https://example.com.").unwrap();

        assert_eq!(citations.len(), 1);
        assert_eq!(citations[0].url.as_deref(), Some("https://example.com"));
        assert_eq!(text, "See.");
    }

    // T011: Citation ID generation tests
    #[test]
    fn citation_id_deterministic() {
        let id1 = generate_citation_id("req-1", "https://example.com");
        let id2 = generate_citation_id("req-1", "https://example.com");
        assert_eq!(id1, id2);
    }

    #[test]
    fn citation_id_different_for_different_citation_text() {
        let id1 = generate_citation_id("req-1", "https://a.com");
        let id2 = generate_citation_id("req-1", "https://b.com");
        assert_ne!(id1, id2);
    }

    #[test]
    fn citation_id_different_for_different_requirements() {
        let id1 = generate_citation_id("req-1", "https://example.com");
        let id2 = generate_citation_id("req-2", "https://example.com");
        assert_ne!(id1, id2);
    }

    #[test]
    fn citation_id_is_valid_uuid() {
        let id = generate_citation_id("req-1", "https://example.com");
        assert!(uuid::Uuid::parse_str(&id).is_ok());
    }

    // === T015: US2 Bibliographic Extraction Tests ===

    // AC-6: NIST SP with Rev and Section suffix
    #[test]
    fn us2_nist_sp_with_rev_and_section() {
        let (text, citations) = extract_citations_from_text(
            "req-1",
            "Controls shall align with NIST SP 800-53 Rev 5, Section AC-2",
        )
        .unwrap();

        assert_eq!(citations.len(), 1);
        assert_eq!(citations[0].text, "NIST SP 800-53 Rev 5, Section AC-2");
        assert!(citations[0].url.is_none());
        assert_eq!(citations[0].source_requirement_id.as_deref(), Some("req-1"));
        assert_eq!(text, "Controls shall align with");
    }

    // ISO standard number
    #[test]
    fn us2_iso_standard() {
        let (text, citations) =
            extract_citations_from_text("req-1", "Must comply with ISO 27001 requirements")
                .unwrap();

        assert_eq!(citations.len(), 1);
        assert_eq!(citations[0].text, "ISO 27001");
        assert!(citations[0].url.is_none());
        assert_eq!(text, "Must comply with requirements");
    }

    // RFC number
    #[test]
    fn us2_rfc_number() {
        let (text, citations) =
            extract_citations_from_text("req-1", "Follow RFC 2119 keyword conventions").unwrap();

        assert_eq!(citations.len(), 1);
        assert_eq!(citations[0].text, "RFC 2119");
        assert!(citations[0].url.is_none());
        assert_eq!(text, "Follow keyword conventions");
    }

    // FIPS number
    #[test]
    fn us2_fips_number() {
        let (text, citations) =
            extract_citations_from_text("req-1", "Encryption must meet FIPS 140-2 standards")
                .unwrap();

        assert_eq!(citations.len(), 1);
        assert_eq!(citations[0].text, "FIPS 140-2");
        assert!(citations[0].url.is_none());
        assert_eq!(text, "Encryption must meet standards");
    }

    // Multiple standards in one requirement
    #[test]
    fn us2_multiple_standards_separate_citations() {
        let (text, citations) = extract_citations_from_text(
            "req-1",
            "Comply with NIST SP 800-53 Rev 5 and ISO 27001 standards",
        )
        .unwrap();

        assert_eq!(citations.len(), 2);
        assert_eq!(citations[0].text, "NIST SP 800-53 Rev 5");
        assert_eq!(citations[1].text, "ISO 27001");
        assert!(citations[0].url.is_none());
        assert!(citations[1].url.is_none());
        assert_eq!(text, "Comply with and standards");
    }

    // === T019/T023: US5 Document-Level Extraction Tests ===

    use crate::model::{DocumentMetadata, PolicyRequirement, PolicySection};
    use std::path::PathBuf;

    fn make_test_doc(sections: Vec<PolicySection>) -> crate::model::PolicyDocument {
        crate::model::PolicyDocument {
            id: "test-doc".to_string(),
            metadata: DocumentMetadata {
                title: "Test Policy".to_string(),
                version: "1.0.0".to_string(),
                author: None,
                date: None,
                source_path: PathBuf::from("test.md"),
                content_hash: None,
            },
            sections,
        }
    }

    fn make_req(stable_id: &str, text: &str) -> PolicyRequirement {
        PolicyRequirement {
            stable_id: Some(stable_id.to_string()),
            text: text.to_string(),
            source_line: 1,
            nesting_depth: 0,
            atom_index: 0,
            parent_text: None,
            citations: vec![],
        }
    }

    fn make_section(title: &str, reqs: Vec<PolicyRequirement>) -> PolicySection {
        PolicySection {
            title: title.to_string(),
            heading_level: 2,
            source_line: 1,
            body_text: None,
            children: vec![],
            requirements: reqs,
        }
    }

    // Multiple sections each with requirements
    #[test]
    fn us5_multiple_sections_with_requirements() {
        let mut doc = make_test_doc(vec![
            make_section(
                "Access Control",
                vec![make_req("req-1", "Comply with https://example.com/access policy")],
            ),
            make_section("Encryption", vec![make_req("req-2", "Must meet FIPS 140-2 standards")]),
        ]);

        extract_citations(&mut doc).unwrap();

        assert_eq!(doc.sections[0].requirements[0].citations.len(), 1);
        assert_eq!(
            doc.sections[0].requirements[0].citations[0].url.as_deref(),
            Some("https://example.com/access")
        );
        assert_eq!(doc.sections[0].requirements[0].text, "Comply with policy");

        assert_eq!(doc.sections[1].requirements[0].citations.len(), 1);
        assert_eq!(doc.sections[1].requirements[0].citations[0].text, "FIPS 140-2");
        assert_eq!(doc.sections[1].requirements[0].text, "Must meet standards");
    }

    // Nested subsections
    #[test]
    fn us5_nested_subsections() {
        let child = PolicySection {
            title: "MFA Requirements".to_string(),
            heading_level: 3,
            source_line: 10,
            body_text: None,
            children: vec![],
            requirements: vec![make_req("req-nested", "Per NIST SP 800-63 use MFA")],
        };

        let parent = PolicySection {
            title: "Authentication".to_string(),
            heading_level: 2,
            source_line: 5,
            body_text: None,
            children: vec![child],
            requirements: vec![make_req(
                "req-parent",
                "See https://auth.example.com for guidelines",
            )],
        };

        let mut doc = make_test_doc(vec![parent]);
        extract_citations(&mut doc).unwrap();

        // Parent requirement
        assert_eq!(doc.sections[0].requirements[0].citations.len(), 1);
        assert_eq!(
            doc.sections[0].requirements[0].citations[0].url.as_deref(),
            Some("https://auth.example.com")
        );

        // Child requirement
        assert_eq!(doc.sections[0].children[0].requirements[0].citations.len(), 1);
        assert_eq!(doc.sections[0].children[0].requirements[0].citations[0].text, "NIST SP 800-63");
    }

    // EC-7: Empty document with zero requirements
    #[test]
    fn us5_empty_document_no_error() {
        let mut doc = make_test_doc(vec![]);
        let result = extract_citations(&mut doc);
        assert!(result.is_ok());
        assert!(doc.sections.is_empty());
    }

    // Mixed citation types (URL + bibliographic in same requirement)
    #[test]
    fn us5_mixed_citation_types_in_same_requirement() {
        let mut doc = make_test_doc(vec![make_section(
            "Compliance",
            vec![make_req(
                "req-mixed",
                "Per NIST SP 800-53 Rev 5 and https://example.com/controls apply",
            )],
        )]);

        extract_citations(&mut doc).unwrap();

        let citations = &doc.sections[0].requirements[0].citations;
        assert_eq!(citations.len(), 2);

        // URL comes first (highest priority)
        assert!(citations[0].url.is_some());
        assert_eq!(citations[0].url.as_deref(), Some("https://example.com/controls"));

        // Bibliographic second
        assert!(citations[1].url.is_none());
        assert_eq!(citations[1].text, "NIST SP 800-53 Rev 5");
    }

    // Integration test: end-to-end with multiple citation types
    #[test]
    fn us5_integration_end_to_end() {
        let mut doc = make_test_doc(vec![
            make_section(
                "Section 1",
                vec![
                    make_req("req-1", "No citations here"),
                    make_req("req-2", "See https://example.com/policy for details"),
                ],
            ),
            make_section(
                "Section 2",
                vec![make_req(
                    "req-3",
                    "Comply with ISO 27001 and NIST SP 800-53 Rev 5, Section AC-2",
                )],
            ),
        ]);

        extract_citations(&mut doc).unwrap();

        // req-1: no citations
        assert!(doc.sections[0].requirements[0].citations.is_empty());
        assert_eq!(doc.sections[0].requirements[0].text, "No citations here");

        // req-2: one URL citation
        assert_eq!(doc.sections[0].requirements[1].citations.len(), 1);
        assert!(!doc.sections[0].requirements[1].text.contains("https://"));

        // req-3: two bibliographic citations
        assert_eq!(doc.sections[1].requirements[0].citations.len(), 2);
        assert_eq!(doc.sections[1].requirements[0].citations[0].text, "ISO 27001");
        assert_eq!(
            doc.sections[1].requirements[0].citations[1].text,
            "NIST SP 800-53 Rev 5, Section AC-2"
        );
    }

    // === T025: US3 Scheme-less URL Detection Tests ===

    // EC-3: www.example.com/policy extracted as Citation with url field
    #[test]
    fn us3_schemeless_url_extracted() {
        let (text, citations) =
            extract_citations_from_text("req-1", "See www.example.com/policy for details").unwrap();

        assert_eq!(citations.len(), 1);
        assert_eq!(citations[0].url.as_deref(), Some("www.example.com/policy"));
        assert_eq!(citations[0].text, "www.example.com/policy");
        assert_eq!(text, "See for details");
    }

    // Scheme-less URL in parentheses
    #[test]
    fn us3_schemeless_url_in_parentheses() {
        let (text, citations) =
            extract_citations_from_text("req-1", "Reference (www.example.com) applies").unwrap();

        assert_eq!(citations.len(), 1);
        assert_eq!(citations[0].url.as_deref(), Some("www.example.com"));
        assert_eq!(text, "Reference applies");
    }

    // Scheme-less URL alongside full URL
    #[test]
    fn us3_schemeless_alongside_full_url() {
        let (_, citations) = extract_citations_from_text(
            "req-1",
            "See https://example.com and www.other.com for details",
        )
        .unwrap();

        assert_eq!(citations.len(), 2);
        assert_eq!(citations[0].url.as_deref(), Some("https://example.com"));
        assert_eq!(citations[1].url.as_deref(), Some("www.other.com"));
    }

    // === T029: US4 Cross-Reference Detection Tests ===

    // AC-7: Section 3.2 extracted
    #[test]
    fn us4_section_reference_extracted() {
        let (text, citations) = extract_citations_from_text(
            "req-1",
            "This control supplements Section 3.2 requirements",
        )
        .unwrap();

        assert_eq!(citations.len(), 1);
        assert_eq!(citations[0].text, "Section 3.2");
        assert!(citations[0].url.is_none());
        assert_eq!(text, "This control supplements requirements");
    }

    // Appendix A extracted
    #[test]
    fn us4_appendix_reference_extracted() {
        let (text, citations) =
            extract_citations_from_text("req-1", "See Appendix A for definitions").unwrap();

        assert_eq!(citations.len(), 1);
        assert_eq!(citations[0].text, "Appendix A");
        assert!(citations[0].url.is_none());
        assert_eq!(text, "See for definitions");
    }

    // Table 2 extracted
    #[test]
    fn us4_table_reference_extracted() {
        let (text, citations) =
            extract_citations_from_text("req-1", "As shown in Table 2 below").unwrap();

        assert_eq!(citations.len(), 1);
        assert_eq!(citations[0].text, "Table 2");
        assert!(citations[0].url.is_none());
        assert_eq!(text, "As shown in below");
    }

    // EC-6: Lowercase "section" NOT matched
    #[test]
    fn us4_lowercase_section_not_matched() {
        let (text, citations) =
            extract_citations_from_text("req-1", "This section covers authentication requirements")
                .unwrap();

        assert!(citations.is_empty());
        assert_eq!(text, "This section covers authentication requirements");
    }

    // Cross-ref alongside URL in same requirement
    #[test]
    fn us4_crossref_alongside_url() {
        let (_, citations) = extract_citations_from_text(
            "req-1",
            "Per Section 3.2 and https://example.com/controls requirements",
        )
        .unwrap();

        assert_eq!(citations.len(), 2);
        // URL first (higher priority)
        assert_eq!(citations[0].url.as_deref(), Some("https://example.com/controls"));
        // Cross-ref second
        assert_eq!(citations[1].text, "Section 3.2");
        assert!(citations[1].url.is_none());
    }

    // === T033: Idempotency Test (S-3) ===

    #[test]
    fn idempotency_second_pass_unchanged() {
        let mut doc = make_test_doc(vec![make_section(
            "Policy",
            vec![
                make_req(
                    "req-1",
                    "Comply with https://example.com/policy and NIST SP 800-53 Rev 5",
                ),
                make_req("req-2", "See www.example.com and Section 3.2 for details"),
            ],
        )]);

        // First pass
        extract_citations(&mut doc).unwrap();
        let first_text = doc.sections[0].requirements[0].text.clone();
        let first_citations = doc.sections[0].requirements[0].citations.clone();
        let second_text = doc.sections[0].requirements[1].text.clone();
        let second_citations = doc.sections[0].requirements[1].citations.clone();

        // Second pass
        extract_citations(&mut doc).unwrap();

        // Text and citations should be identical after second pass
        assert_eq!(doc.sections[0].requirements[0].text, first_text);
        assert_eq!(doc.sections[0].requirements[0].citations, first_citations);
        assert_eq!(doc.sections[0].requirements[1].text, second_text);
        assert_eq!(doc.sections[0].requirements[1].citations, second_citations);
    }

    // === T034: Performance Test (SEC-6) ===

    #[test]
    fn performance_1000_requirements_under_1_second() {
        let reqs: Vec<PolicyRequirement> = (0..1000)
            .map(|i| {
                make_req(
                    &format!("req-{i}"),
                    &format!(
                        "Requirement {i}: comply with https://example.com/policy-{i}, \
                         per NIST SP 800-53 Rev 5 and www.standards.org/ref-{i}, \
                         see Section 3.{} for details",
                        i % 10
                    ),
                )
            })
            .collect();

        let mut doc = make_test_doc(vec![make_section("All Requirements", reqs)]);

        let start = std::time::Instant::now();
        extract_citations(&mut doc).unwrap();
        let elapsed = start.elapsed();

        assert!(
            elapsed.as_secs() < 1,
            "Citation extraction took {elapsed:?} (>1s) for 1000 requirements"
        );

        // Verify all requirements were processed
        for req in &doc.sections[0].requirements {
            assert!(!req.citations.is_empty(), "Requirement should have citations");
        }
    }

    // === T035: ReDoS Resistance Tests (SEC-1) ===

    #[test]
    fn redos_resistance_long_url_like_string() {
        let pathological = "https://".to_string() + &"a".repeat(10_000);
        let start = std::time::Instant::now();
        let result = extract_citations_from_text("req-1", &pathological);
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        assert!(elapsed.as_secs() < 1, "URL regex took {elapsed:?} on pathological input");
    }

    #[test]
    fn redos_resistance_long_biblio_like_string() {
        let pathological = "NIST SP ".to_string() + &"1-2-3-4-5-".repeat(2_000);
        let start = std::time::Instant::now();
        let result = extract_citations_from_text("req-1", &pathological);
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        assert!(
            elapsed.as_secs() < 1,
            "Bibliographic regex took {elapsed:?} on pathological input"
        );
    }

    #[test]
    fn redos_resistance_deeply_nested_parens() {
        let pathological = "(".repeat(1_000) + "https://example.com" + &")".repeat(1_000);
        let start = std::time::Instant::now();
        let result = extract_citations_from_text("req-1", &pathological);
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        assert!(elapsed.as_secs() < 1, "Regex took {elapsed:?} on deeply nested parens");
    }
}
