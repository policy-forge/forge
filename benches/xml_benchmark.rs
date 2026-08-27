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

mod common;

use std::hint::black_box;
use std::path::Path;

use common::MAX_SIZE_BYTES;
use criterion::{Criterion, criterion_group, criterion_main};

/// Path to the committed 50-page synthetic policy fixture.
const FIXTURE_PATH: &str = "tests/fixtures/synthetic-50page-policy.md";

/// Prepare the shared document input for XML serialization benchmarks.
fn prepared_document(fixture_path: &Path) -> forge::model::PolicyDocument {
    let ingested =
        forge::ingest::ingest_file(fixture_path, MAX_SIZE_BYTES).expect("fixture ingestion failed");
    let content = ingested.reconstruct_content();
    let sections = forge::parse::extract_sections(&content).expect("section extraction failed");
    let clauses = forge::parse::extract_clauses(&content).expect("clause extraction failed");
    let document = forge::model::assemble_document(&ingested, &sections, &clauses)
        .expect("document assembly failed");
    let atomized = forge::parse::atomize_document(&document).expect("document atomization failed");
    let doc = forge::uuid::assign_stable_ids(atomized);
    let doc = forge::citation::extract_citations(doc).expect("citation extraction failed");
    let doc = forge::parse::annotate_modalities(doc).expect("modality annotation failed");
    let mut doc = doc;
    forge::parameter::extract_parameters(&mut doc).expect("parameter extraction failed");
    doc
}

/// Pre-process a fixture through the full pipeline up to catalog assembly.
fn build_catalog_from_fixture(fixture_path: &Path) -> forge::oscal::catalog::OscalCatalog {
    let doc = prepared_document(fixture_path);

    let mut trace_links = forge::TraceLinkCollection::new();
    let mut catalog =
        forge::oscal::build_catalog(&doc, Some(&mut trace_links)).expect("catalog assembly failed");
    forge::oscal::trace_embedding::embed_trace_in_catalog(&mut catalog, &trace_links);

    let metadata =
        forge::oscal::assemble_metadata(&doc.metadata, None).expect("metadata assembly failed");
    let citations = forge::oscal::component_definition::collect_all_citations(&doc.sections);
    let (back_matter_resources, _) =
        forge::oscal::generate_back_matter(&citations).expect("back-matter generation failed");
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
    let doc = prepared_document(fixture_path);
    let envelope = forge::oscal::build_component_definition(&doc, None, None, Some("test.md"))
        .expect("component-definition assembly failed");
    envelope.component_definition
}

fn bench_xml_serialization(c: &mut Criterion) {
    let fixture_path = Path::new(FIXTURE_PATH);
    assert!(fixture_path.exists(), "required benchmark fixture missing: {FIXTURE_PATH}");

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
