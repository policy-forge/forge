//! Export pipeline performance benchmarks (T047).
//!
//! Benchmarks `export_artifact()` converting a large OSCAL JSON catalog (~500KB+)
//! to XML format through the full deserialize-validate-serialize pipeline.
//!
//! Target: < 1 second per export (SC-005, Constitution VI).
//!
//! # Usage
//!
//! ```bash
//! cargo bench --bench export_bench
//! ```

use std::hint::black_box;
use std::path::Path;

use criterion::{Criterion, criterion_group, criterion_main};
use forge::DEFAULT_MAX_SIZE_BYTES;

/// Path to the committed 50-page synthetic policy fixture.
const FIXTURE_PATH: &str = "tests/fixtures/synthetic-50page-policy.md";

/// Build a large OSCAL catalog JSON string from the synthetic fixture.
///
/// Runs the full forge pipeline (ingest → parse → atomize → catalog) and
/// serializes to JSON, producing a 500KB+ artifact for realistic benchmarks.
fn build_catalog_json(fixture_path: &Path) -> String {
    let ingested = forge::ingest::ingest_file(fixture_path, DEFAULT_MAX_SIZE_BYTES)
        .expect("bench setup: ingest_file failed");
    let content = ingested.reconstruct_content();
    let sections =
        forge::parse::extract_sections(&content).expect("bench setup: extract_sections failed");
    let clauses =
        forge::parse::extract_clauses(&content).expect("bench setup: extract_clauses failed");
    let document = forge::model::assemble_document(&ingested, &sections, &clauses)
        .expect("bench setup: assemble_document failed");
    let atomized =
        forge::parse::atomize_document(&document).expect("bench setup: atomize_document failed");
    let doc = forge::uuid::assign_stable_ids(atomized);
    let doc =
        forge::citation::extract_citations(doc).expect("bench setup: extract_citations failed");

    let mut trace_links = forge::TraceLinkCollection::new();
    let mut catalog = forge::oscal::build_catalog(&doc, Some(&mut trace_links))
        .expect("bench setup: build_catalog failed");
    forge::oscal::trace_embedding::embed_trace_in_catalog(&mut catalog, &trace_links);

    let metadata = forge::oscal::assemble_metadata(&doc.metadata, None)
        .expect("bench setup: assemble_metadata failed");
    let citations = forge::oscal::component_definition::collect_all_citations(&doc.sections);
    let (back_matter_resources, _) = forge::oscal::generate_back_matter(&citations)
        .expect("bench setup: generate_back_matter failed");
    let back_matter = if back_matter_resources.is_empty() {
        None
    } else {
        Some(forge::BackMatter { resources: back_matter_resources })
    };

    let oscal_catalog = forge::oscal::OscalCatalog {
        uuid: metadata.uuid.to_string(),
        metadata: forge::oscal::catalog::OscalMetadata {
            title: metadata.title,
            last_modified: metadata.last_modified.to_rfc3339(),
            version: metadata.version,
            oscal_version: metadata.oscal_version,
        },
        controls: vec![],
        groups: catalog.groups,
        back_matter,
    };

    let envelope = forge::oscal::catalog::CatalogEnvelope { catalog: oscal_catalog };
    serde_json::to_string_pretty(&envelope).expect("bench setup: serialize catalog JSON failed")
}

fn bench_export_pipeline(c: &mut Criterion) {
    let fixture_path = Path::new(FIXTURE_PATH);
    assert!(
        fixture_path.exists(),
        "benchmark fixture missing: {FIXTURE_PATH} (commit it or fix FIXTURE_PATH)"
    );

    // Pre-compute the large catalog JSON and write to a temp file.
    let catalog_json = build_catalog_json(fixture_path);
    let json_size_kb = catalog_json.len() / 1024;
    assert!(
        json_size_kb >= 500,
        "export benchmark fixture regressed below 500 KiB: {json_size_kb} KiB"
    );
    eprintln!("export benchmark input: {json_size_kb} KiB");

    let temp_dir =
        tempfile::TempDir::new().expect("bench setup: create temporary directory failed");
    let json_path = temp_dir.path().join("large-catalog.json");
    std::fs::write(&json_path, &catalog_json).expect("bench setup: write catalog fixture failed");

    let mut group = c.benchmark_group("export_pipeline");

    group.bench_function("catalog_json_to_xml", |b| {
        b.iter(|| {
            forge::cli::export::export_artifact(
                black_box(&json_path),
                forge::cli::OutputFormat::Xml,
                None,
            )
            .expect("catalog JSON export benchmark must succeed");
        });
    });

    group.bench_function("catalog_json_to_yaml", |b| {
        b.iter(|| {
            forge::cli::export::export_artifact(
                black_box(&json_path),
                forge::cli::OutputFormat::Yaml,
                None,
            )
            .expect("catalog JSON export benchmark must succeed");
        });
    });

    group.finish();
}

criterion_group!(benches, bench_export_pipeline);
criterion_main!(benches);
