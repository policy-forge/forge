use std::path::Path;

use forge::ForgeError;
use forge::ingest::ingest_file;
use forge::model::PolicySection;
use forge::model::assemble_document;
use forge::parse::{extract_clauses, extract_sections};

fn count_and_verify(sections: &[PolicySection], total: &mut usize) {
    for section in sections {
        assert!(section.source_line >= 1, "Section source_line must be >= 1");
        for req in &section.requirements {
            assert!(req.source_line >= 1, "Requirement source_line must be >= 1");
            assert!(!req.text.is_empty(), "Requirement text must not be empty");
            assert!(req.stable_id.is_none(), "stable_id should be None (WI-7 not run)");
            *total += 1;
        }
        count_and_verify(&section.children, total);
    }
}

/// Full pipeline integration test: ingest -> extract sections -> extract clauses -> assemble.
#[test]
fn full_pipeline_produces_valid_policy_document() -> Result<(), ForgeError> {
    let input = Path::new("tests/fixtures/sample_policy.md");
    let ingested = ingest_file(input, 10 * 1024 * 1024)?;

    // Reconstruct content from lines
    let content = ingested.reconstruct_content();

    let sections = extract_sections(&content)?;
    let clauses = extract_clauses(&content)?;
    let document = assemble_document(&ingested, &sections, &clauses)?;

    // Verify metadata from frontmatter
    assert_eq!(document.metadata.title, "Sample Security Policy");
    assert_eq!(document.metadata.version, "1.0.0");
    assert_eq!(document.metadata.author.as_deref(), Some("Policy Team"));

    // Verify sections exist
    assert!(!document.sections.is_empty(), "Document should have sections");

    // Verify requirements exist and have valid source_lines
    let mut total_requirements = 0;
    count_and_verify(&document.sections, &mut total_requirements);

    assert_eq!(
        document.total_requirements(),
        total_requirements,
        "total_requirements() helper must match manual count"
    );

    assert!(total_requirements > 0, "Document should have requirements");

    // Verify content_hash is present (from IngestedDocument.fingerprint)
    assert!(document.metadata.content_hash.is_some());

    // Verify document id is derived from filename
    assert_eq!(document.id, "sample_policy");

    Ok(())
}
