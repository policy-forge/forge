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
static URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https?://[^\s\)\]>,;]+").expect("URL regex must compile")
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
        assert_eq!(
            citations[0].url.as_deref(),
            Some("https://example.com/policy")
        );
        assert_eq!(citations[0].text, "https://example.com/policy");
        assert_eq!(
            citations[0].source_requirement_id.as_deref(),
            Some("req-1")
        );
        assert_eq!(text, "Access must comply with requirements");
    }

    // AC-2: Multiple URLs in one requirement
    #[test]
    fn us1_multiple_urls_extracted() {
        let (text, citations) = extract_citations_from_text(
            "req-1",
            "See https://a.com and https://b.com for details",
        )
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
        let (text, citations) = extract_citations_from_text(
            "req-1",
            "Requirements (https://example.com) apply",
        )
        .unwrap();

        assert_eq!(citations.len(), 1);
        assert_eq!(
            citations[0].url.as_deref(),
            Some("https://example.com")
        );
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
        let (text, citations) = extract_citations_from_text(
            "req-1",
            "Users must authenticate before access",
        )
        .unwrap();

        assert!(citations.is_empty());
        assert_eq!(text, "Users must authenticate before access");
    }

    // EC-2: Whitespace normalization after stripping
    #[test]
    fn us1_whitespace_normalized_after_stripping() {
        let (text, _) = extract_citations_from_text(
            "req-1",
            "Access  https://example.com  requirements",
        )
        .unwrap();

        assert!(!text.contains("  "));
        assert_eq!(text, "Access requirements");
    }

    // HTTP URL (not just HTTPS)
    #[test]
    fn us1_http_url_extracted() {
        let (text, citations) = extract_citations_from_text(
            "req-1",
            "See http://example.com for details",
        )
        .unwrap();

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
}
