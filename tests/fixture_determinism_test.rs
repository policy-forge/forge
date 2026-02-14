//! Fixture determinism test (EC-1, WI-24).
//!
//! Verifies that `generate_synthetic_policy()` produces byte-identical output
//! across invocations, ensuring benchmark results are reproducible.

mod common;

/// EC-1: Two calls to `generate_synthetic_policy()` produce byte-identical output.
///
/// Also validates basic fixture properties:
/// - Non-empty output
/// - Contains expected H1 heading
/// - Contains approximately 200 requirements (numbered list items)
#[test]
fn determinism_generates_identical_output() {
    let first = common::fixture_generator::generate_synthetic_policy();
    let second = common::fixture_generator::generate_synthetic_policy();

    // ── Byte-identical determinism (EC-1) ──
    assert_eq!(
        first, second,
        "generate_synthetic_policy() must produce byte-identical output across invocations"
    );

    // ── Non-empty ──
    assert!(!first.is_empty(), "Generated fixture must not be empty");

    // ── Contains expected H1 heading ──
    assert!(
        first.contains("# Comprehensive Information Security Policy"),
        "Generated fixture must contain the expected H1 heading"
    );

    // ── Approximately 200 requirements (numbered list items) ──
    // Requirements are emitted as numbered Markdown list items (e.g., "1. The organization shall...")
    let requirement_count = first
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            // Match numbered list items: "1. ...", "2. ...", etc.
            trimmed.chars().next().is_some_and(|c| c.is_ascii_digit()) && trimmed.contains(". ")
        })
        .count();

    assert!(
        (180..=220).contains(&requirement_count),
        "Expected ~200 requirements (numbered list items), found {requirement_count}"
    );
}
