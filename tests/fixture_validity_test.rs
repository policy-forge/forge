mod common;

use std::path::Path;

/// EC-5 (WI-24): Verify the committed synthetic fixture produces valid OSCAL JSON
/// output when run through the full catalog pipeline.
#[test]
fn fixture_produces_valid_oscal_output() {
    // Read the committed 50-page synthetic fixture
    let fixture_path = Path::new("tests/fixtures/synthetic-50page-policy.md");
    assert!(fixture_path.exists(), "Synthetic fixture must exist at {}", fixture_path.display());

    // Step 1: Ingest file
    let ingested = forge::ingest::ingest_file(fixture_path, 10 * 1024 * 1024)
        .expect("ingest_file should succeed on synthetic fixture");

    // Step 2: Reconstruct content
    let content = ingested.reconstruct_content();

    // Step 3: Extract sections
    let sections =
        forge::parse::extract_sections(&content).expect("extract_sections should succeed");

    // Step 4: Extract clauses
    let clauses = forge::parse::extract_clauses(&content).expect("extract_clauses should succeed");

    // Step 5: Assemble document
    let document = forge::model::assemble_document(&ingested, &sections, &clauses)
        .expect("assemble_document should succeed");

    // Step 6: Atomize document
    let atomized =
        forge::parse::atomize_document(&document).expect("atomize_document should succeed");

    // Step 7: Assign stable IDs
    let mut doc = atomized;
    forge::uuid::assign_stable_ids(&mut doc);

    // Step 7b: Extract citations
    forge::citation::extract_citations(&mut doc).expect("extract_citations should succeed");

    // Step 8: Build catalog with trace link capture
    let mut trace_links = forge::TraceLinkCollection::new();
    let mut catalog = forge::oscal::build_catalog(&doc, Some(&mut trace_links))
        .expect("build_catalog should succeed");

    // Step 8b: Embed trace props/links into catalog
    forge::oscal::trace_embedding::embed_trace_in_catalog(&mut catalog, &trace_links);

    // Step 9: Assemble metadata
    let real_metadata = forge::oscal::assemble_metadata(&doc.metadata, None)
        .expect("assemble_metadata should succeed");

    // Step 10: Generate back matter
    let (back_matter_resources, _resource_map) =
        forge::oscal::generate_back_matter(&[]).expect("generate_back_matter should succeed");

    // Step 11: Construct CatalogEnvelope
    let back_matter = if back_matter_resources.is_empty() {
        None
    } else {
        Some(forge::BackMatter { resources: back_matter_resources })
    };

    let envelope = forge::oscal::CatalogEnvelope {
        catalog: forge::oscal::OscalCatalog {
            uuid: real_metadata.uuid.to_string(),
            metadata: forge::oscal::catalog::OscalMetadata {
                title: real_metadata.title,
                last_modified: real_metadata.last_modified.to_rfc3339(),
                version: real_metadata.version,
                oscal_version: real_metadata.oscal_version,
            },
            groups: catalog.groups,
            back_matter,
        },
    };

    // Step 12: Serialize to JSON
    let json_string =
        serde_json::to_string_pretty(&envelope).expect("Serialization to JSON should succeed");

    // ─── Assertions ─────────────────────────────────────────────────────

    // Assert: JSON output is valid (can be parsed as serde_json::Value)
    let json: serde_json::Value =
        serde_json::from_str(&json_string).expect("Output must be valid JSON");

    // Assert: JSON contains a "catalog" key
    assert!(json.get("catalog").is_some(), "JSON output must contain a 'catalog' key");

    let catalog_value = &json["catalog"];

    // Assert: catalog contains "groups" with at least 1 group
    let groups = catalog_value["groups"].as_array().expect("catalog must contain a 'groups' array");
    assert!(!groups.is_empty(), "catalog.groups must contain at least 1 group");

    // Assert: count total controls across all groups — at least 100 (sanity check for scale)
    let total_controls: usize = groups
        .iter()
        .filter_map(|group| group["controls"].as_array())
        .map(std::vec::Vec::len)
        .sum();

    assert!(
        total_controls >= 100,
        "Synthetic fixture should produce at least 100 controls for scale testing, got {total_controls}"
    );
}
