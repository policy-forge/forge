//! Contract definitions for WI-8 Citation Extraction.
//!
//! These types and function signatures define the public API contract
//! before implementation begins (Constitution Principle III).

use crate::error::ForgeError;
use crate::model::{Citation, PolicyDocument};

/// Extract citations from all requirements in a PolicyDocument.
///
/// Walks the full section tree recursively. For each PolicyRequirement:
/// 1. Detects URL, bibliographic, and cross-reference patterns in `text`
/// 2. Creates Citation objects for each match
/// 3. Strips matched text from `text` and normalizes whitespace
/// 4. Populates the `citations` field with extracted citations
///
/// # Arguments
///
/// * `document` - Mutable reference to the PolicyDocument to process
///
/// # Errors
///
/// Returns `ForgeError::Parse` if regex pattern compilation fails
/// (should not happen with static patterns).
pub fn extract_citations(document: &mut PolicyDocument) -> Result<(), ForgeError>;

/// Extract citations from a single requirement's text.
///
/// Lower-level function that performs pattern matching and text cleanup.
/// Returns the cleaned text and a vector of extracted citations.
///
/// # Arguments
///
/// * `requirement_id` - The stable_id of the source requirement (used for Citation.source_requirement_id)
/// * `text` - The requirement text to scan for citations
///
/// # Returns
///
/// A tuple of (cleaned_text, extracted_citations).
///
/// # Errors
///
/// The `Result` wrapper is retained for API stability, but this function
/// never returns `Err`: the static patterns are compiled once at
/// initialization and `find_iter` on a compiled `regex::Regex` is infallible.
pub fn extract_citations_from_text(
    requirement_id: &str,
    text: &str,
) -> Result<(String, Vec<Citation>), ForgeError>;

/// Generate a deterministic citation ID from requirement ID, occurrence
/// ordinal and citation text.
///
/// Uses UUID v5 with FORGE_NAMESPACE_UUID namespace.
/// Input: "{requirement_id}:{occurrence}:{citation_text}"
///
/// The per-text occurrence ordinal (0-based, counted within one requirement)
/// keeps repeated identical citations distinct instead of colliding on one
/// UUID (F0298).
///
/// # Arguments
///
/// * `requirement_id` - The stable_id of the source requirement
/// * `citation_text` - The extracted citation text
/// * `occurrence` - Ordinal of this citation text within the requirement
///
/// # Returns
///
/// A deterministic UUID v5 string.
pub fn generate_citation_id(requirement_id: &str, citation_text: &str, occurrence: usize) -> String;
