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
use crate::model::{Citation, PolicyDocument};
use crate::uuid::FORGE_NAMESPACE_UUID;

// T010: URL pattern — matches http:// or https:// followed by non-whitespace, non-delimiter chars.
static URL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"https?://[^\s\)\]>,;]+").expect("URL regex must compile"));

// T016: Bibliographic pattern — NIST SP, ISO, RFC, FIPS with optional Rev and Section suffixes.
static BIBLIO_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:NIST\s+SP|ISO|RFC|FIPS)\s+[\d]+[-\w.]*(?:\s+Rev\.?\s*\d+)?(?:,?\s+Section\s+[\w.-]+)?")
        .expect("Bibliographic regex must compile")
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
pub fn extract_citations(_document: &mut PolicyDocument) -> Result<(), ForgeError> {
    todo!("T020: Implement document-level citation extraction")
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
        let url_text = m.as_str().to_string();
        let citation_id = generate_citation_id(requirement_id, &url_text);
        citations.push(Citation {
            id: citation_id,
            text: url_text.clone(),
            url: Some(url_text),
            source_requirement_id: Some(requirement_id.to_string()),
        });
        matched_ranges.push(m.start()..m.end());
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
}
