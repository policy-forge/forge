//! XML serialization performance benchmarks (T056).
//!
//! Benchmarks `serialize_catalog_to_xml` and `serialize_component_definition_to_xml`
//! against the 50-page synthetic policy fixture (~150KB, ~200 requirements).
//!
//! Target: < 50ms per serialization (plan.md performance budget).
//!
//! # Usage
//!
//! ```bash
//! cargo bench --bench xml_benchmark
//! ```

use std::hint::black_box;
use std::path::Path;

use criterion::{Criterion, criterion_group, criterion_main};

/// Path to the committed 50-page synthetic policy fixture.
const FIXTURE_PATH: &str = "tests/fixtures/synthetic-50page-policy.md";

/// Maximum file size for ingest (10 MB).
///
/// Duplicated here because benchmarks cannot use the `tests/common` module.
const MAX_SIZE_BYTES: u64 = 10 * 1024 * 1024;

/// Pre-process a fixture through the full pipeline up to catalog assembly.
fn build_catalog_from_fixture(fixture_path: &Path) -> forge::oscal::catalog::OscalCatalog {
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

    forge::oscal::OscalCatalog {
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
    }
}

/// Pre-process a fixture through the component definition pipeline.
fn build_component_def_from_fixture(
    fixture_path: &Path,
) -> forge::oscal::component_definition::ComponentDefinition {
    let ingested = forge::ingest::ingest_file(fixture_path, MAX_SIZE_BYTES).unwrap();
    let content = ingested.reconstruct_content();
    let sections = forge::parse::extract_sections(&content).unwrap();
    let clauses = forge::parse::extract_clauses(&content).unwrap();
    let document = forge::model::assemble_document(&ingested, &sections, &clauses).unwrap();
    let atomized = forge::parse::atomize_document(&document).unwrap();
    let doc = forge::uuid::assign_stable_ids(atomized);
    let doc = forge::citation::extract_citations(doc).unwrap();

    let envelope =
        forge::oscal::build_component_definition(&doc, None, None, Some("test.md")).unwrap();
    envelope.component_definition
}

fn bench_xml_serialization(c: &mut Criterion) {
    let fixture_path = Path::new(FIXTURE_PATH);
    if !fixture_path.exists() {
        tracing::warn!(fixture = %FIXTURE_PATH, "Skipping XML benchmark: fixture not found");
        return;
    }

    let mut group = c.benchmark_group("xml_serialization");

    // Pre-compute the catalog and component definition
    let catalog = build_catalog_from_fixture(fixture_path);
    let component_def = build_component_def_from_fixture(fixture_path);

    group.bench_function("serialize_catalog_to_xml", |b| {
        b.iter(|| {
            let xml = forge::export::xml_serializer::serialize_catalog_to_xml(black_box(&catalog))
                .unwrap();
            black_box(xml)
        });
    });

    group.bench_function("serialize_component_definition_to_xml", |b| {
        b.iter(|| {
            let xml = forge::export::xml_serializer::serialize_component_definition_to_xml(
                black_box(&component_def),
            )
            .unwrap();
            black_box(xml)
        });
    });

    group.finish();
}

criterion_group!(benches, bench_xml_serialization);
criterion_main!(benches);
