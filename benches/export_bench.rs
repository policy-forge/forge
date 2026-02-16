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

/// Path to the committed 50-page synthetic policy fixture.
const FIXTURE_PATH: &str = "tests/fixtures/synthetic-50page-policy.md";

/// Maximum file size for ingest (10 MB).
const MAX_SIZE_BYTES: u64 = 10 * 1024 * 1024;

/// Build a large OSCAL catalog JSON string from the synthetic fixture.
///
/// Runs the full forge pipeline (ingest → parse → atomize → catalog) and
/// serializes to JSON, producing a 500KB+ artifact for realistic benchmarks.
fn build_catalog_json(fixture_path: &Path) -> String {
    let ingested = forge::ingest::ingest_file(fixture_path, MAX_SIZE_BYTES).unwrap();
    let content = ingested.reconstruct_content();
    let sections = forge::parse::extract_sections(&content).unwrap();
    let clauses = forge::parse::extract_clauses(&content).unwrap();
    let document = forge::model::assemble_document(&ingested, &sections, &clauses).unwrap();
    let atomized = forge::parse::atomize_document(&document).unwrap();
    let doc = forge::uuid::assign_stable_ids(atomized);
    let doc = forge::citation::extract_citations(doc).unwrap();

    let mut trace_links = forge::TraceLinkCollection::new();
    let mut catalog = forge::oscal::build_catalog(&doc, Some(&mut trace_links)).unwrap();
    forge::oscal::trace_embedding::embed_trace_in_catalog(&mut catalog, &trace_links);

    let metadata = forge::oscal::assemble_metadata(&doc.metadata, None).unwrap();
    let citations = doc.collect_citations();
    let (back_matter_resources, _) = forge::oscal::generate_back_matter(&citations).unwrap();
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
        groups: catalog.groups,
        back_matter,
    };

    let envelope = forge::oscal::catalog::CatalogEnvelope { catalog: oscal_catalog };
    serde_json::to_string_pretty(&envelope).unwrap()
}

fn bench_export_pipeline(c: &mut Criterion) {
    let fixture_path = Path::new(FIXTURE_PATH);
    if !fixture_path.exists() {
        tracing::warn!(fixture = %FIXTURE_PATH, "Skipping export benchmark: fixture not found");
        return;
    }

    // Pre-compute the large catalog JSON and write to a temp file
    let catalog_json = build_catalog_json(fixture_path);
    let json_size_kb = catalog_json.len() / 1024;

    let temp_dir = tempfile::TempDir::new().unwrap();
    let json_path = temp_dir.path().join("large-catalog.json");
    std::fs::write(&json_path, &catalog_json).unwrap();

    let mut group = c.benchmark_group("export_pipeline");

    let xml_output = temp_dir.path().join("out.xml");
    group.bench_function(&format!("json_to_xml_{}kb", json_size_kb), |b| {
        b.iter(|| {
            forge::cli::export::export_artifact(
                black_box(&json_path),
                forge::cli::OutputFormat::Xml,
                Some(black_box(&xml_output)),
            )
            .unwrap();
        });
    });

    let yaml_output = temp_dir.path().join("out.yaml");
    group.bench_function(&format!("json_to_yaml_{}kb", json_size_kb), |b| {
        b.iter(|| {
            forge::cli::export::export_artifact(
                black_box(&json_path),
                forge::cli::OutputFormat::Yaml,
                Some(black_box(&yaml_output)),
            )
            .unwrap();
        });
    });

    group.finish();
}

criterion_group!(benches, bench_export_pipeline);
criterion_main!(benches);
