//! Fixture determinism test (EC-1, WI-24).
//!
//! Verifies that `generate_synthetic_policy()` produces byte-identical output
//! across invocations, ensuring benchmark results are reproducible and the
//! committed fixture stays in sync with the generator.

mod common;

use std::path::Path;

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

    // ── One plain-text standard reference per generated subsection ──
    let standard_reference_count =
        first.matches("The requirements in this section are aligned with ").count();
    assert_eq!(
        standard_reference_count, 40,
        "Expected one standard reference for each of the 40 generated subsections"
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

#[test]
fn generated_section_references_match_target_headings() {
    let policy = common::fixture_generator::generate_synthetic_policy();

    for (reference, heading) in [
        ("Section 2 of this policy", "## 2. Data Protection"),
        ("Section 1.1 for account review procedures", "### 1.1. User Account Management"),
        ("Section 8.3 for backup procedures", "### 8.3. Backup and Restoration Procedures"),
        ("Section 2.4 of this policy", "### 2.4. Data Disposal"),
        (
            "Section 7.1 for personnel clearance requirements",
            "### 7.1. Background Screening and Verification",
        ),
        ("Section 2.4 and must be documented", "### 2.4. Data Disposal"),
        ("Section 3.1", "### 3.1. Incident Detection and Reporting"),
        ("Section 1.3 for authentication requirements", "### 1.3. Authorization and Privileges"),
    ] {
        assert!(policy.contains(reference), "missing cross-reference: {reference}");
        assert!(policy.contains(heading), "missing target heading for {reference}: {heading}");
    }
}

/// Verify the committed fixture file stays in sync with `generate_synthetic_policy()`.
///
/// Prevents silent drift between the generator, committed fixture, and benchmark input.
#[test]
fn committed_fixture_matches_generator() {
    let fixture_path = Path::new("tests/fixtures/synthetic-50page-policy.md");
    assert!(fixture_path.exists(), "Committed fixture must exist at {}", fixture_path.display());

    let generated = common::fixture_generator::generate_synthetic_policy();

    if std::env::var_os("UPDATE_SYNTHETIC_FIXTURE").is_some() {
        std::fs::write(fixture_path, &generated)
            .expect("Should update the committed synthetic fixture");
    }

    let committed = std::fs::read_to_string(fixture_path)
        .expect("Should be able to read the committed fixture file");
    assert_eq!(
        committed,
        generated,
        "Committed fixture at {} has drifted from generate_synthetic_policy() output. \
         Regenerate the fixture to bring them back in sync.",
        fixture_path.display()
    );
}
