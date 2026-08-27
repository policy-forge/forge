//! Pipeline performance benchmarks (WI-24).
//!
//! Benchmarks the full catalog pipeline and individual pipeline stages
//! using the 50-page synthetic policy fixture (~150KB, ~200 requirements).
//!
//! # Usage
//!
//! ```bash
//! # Full pipeline benchmark
//! cargo bench --bench pipeline_benchmark -- full_pipeline
//!
//! # Per-stage benchmarks
//! cargo bench --bench pipeline_benchmark -- pipeline_stages
//!
//! # All benchmarks
//! cargo bench --bench pipeline_benchmark
//! ```

use std::hint::black_box;
use std::path::Path;
use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};

/// Path to the committed 50-page synthetic policy fixture.
const FIXTURE_PATH: &str = "tests/fixtures/synthetic-50page-policy.md";

/// Maximum file size for ingest (10 MB).
const MAX_SIZE_BYTES: u64 = 10 * 1024 * 1024;

// ─── Full Pipeline Helper ───────────────────────────────────────────────

/// Runs the full catalog pipeline and returns the validated serialized JSON string.
///
/// Mirrors `src/pipeline.rs::run_catalog_pipeline` but captures the JSON
/// string instead of writing to file/stdout. Composes public API functions
/// in the same order as the production pipeline.
fn run_full_catalog_pipeline(fixture_path: &Path) -> Result<String, forge::ForgeError> {
    // Step 1: Ingest
    let ingested = forge::ingest::ingest_file(fixture_path, MAX_SIZE_BYTES)?;
    let content = ingested.reconstruct_content();

    // Step 2: Parse
    let sections = forge::parse::extract_sections(&content)?;
    let clauses = forge::parse::extract_clauses(&content)?;

    // EC-6: mirror the production no-structure guard (src/pipeline.rs).
    let has_clause_structure = !clauses.list_items.is_empty() || !clauses.tables.is_empty();
    if sections.is_empty() && !has_clause_structure {
        return Err(forge::ForgeError::NoStructureDetected { path: fixture_path.to_path_buf() });
    }

    // Step 3: Assemble + Atomize + IDs + Citations
    let document = forge::model::assemble_document(&ingested, &sections, &clauses)?;
    let atomized = forge::parse::atomize_document(&document)?;
    let doc = forge::uuid::assign_stable_ids(atomized);
    let doc = forge::citation::extract_citations(doc)?;
    // Steps 7c/7d: modality annotation and parameter extraction, same order as
    // the production pipeline so the timed stages match a real run.
    let doc = forge::parse::annotate_modalities(doc)?;
    let mut doc = doc;
    forge::parameter::extract_parameters(&mut doc)?;

    // Step 4: Catalog assembly
    let envelope = build_catalog_envelope(&doc)?;

    // Step 5: Production serializes, parses to a value, then validates schema
    // and semantic invariants before returning the JSON payload.
    let json = serde_json::to_string_pretty(&envelope)
        .map_err(|error| forge::ForgeError::Serialization(error.to_string()))?;
    let value = serde_json::from_str(&json)
        .map_err(|error| forge::ForgeError::Serialization(error.to_string()))?;
    let report = forge::validate::run_full_validation(
        "generated benchmark catalog",
        &value,
        forge::OscalModelType::Catalog,
    )
    .map_err(|error| forge::ForgeError::SchemaValidation(error.to_string()))?;
    if !report.is_valid() {
        return Err(forge::ForgeError::SchemaValidation(format!(
            "{} validation error(s) in generated benchmark catalog",
            report.errors().len()
        )));
    }

    Ok(json)
}

// ─── Full Pipeline Benchmark ────────────────────────────────────────────

/// Benchmark the full catalog pipeline end-to-end.
///
/// Measures ingest → parse → assemble → atomize → IDs → citations →
/// modality annotation → parameter extraction → catalog → traces → metadata →
/// `back_matter` → serialize → validate.
///
/// Uses extended measurement time (10s) for stable results on potentially
/// multi-second pipeline runs.
fn bench_full_pipeline(c: &mut Criterion) {
    let fixture_path = Path::new(FIXTURE_PATH);
    assert!(
        fixture_path.exists(),
        "Synthetic fixture must exist at {FIXTURE_PATH} — run fixture generator first"
    );

    let ingested = forge::ingest::ingest_file(fixture_path, MAX_SIZE_BYTES)
        .expect("benchmark fixture ingestion must succeed");
    let content = ingested.reconstruct_content();
    let sections = forge::parse::extract_sections(&content)
        .expect("benchmark fixture section parsing must succeed");
    let clauses = forge::parse::extract_clauses(&content)
        .expect("benchmark fixture clause parsing must succeed");
    let document = forge::model::assemble_document(&ingested, &sections, &clauses)
        .expect("benchmark fixture assembly must succeed");
    let atomized = forge::parse::atomize_document(&document)
        .expect("benchmark fixture atomization must succeed");
    assert!(
        atomized.sections.iter().any(|section| !section.requirements.is_empty()),
        "fixture produced no atomized requirements"
    );

    let mut group = c.benchmark_group("full_pipeline");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("catalog_50page", |b| {
        b.iter(|| {
            let result = run_full_catalog_pipeline(black_box(fixture_path));
            black_box(result).expect("Pipeline must not fail during benchmark")
        });
    });

    group.finish();
}

// ─── Helpers ────────────────────────────────────────────────────────────

/// Build a `CatalogEnvelope` from a processed document.
///
/// Shared between per-stage `catalog_assembly` benchmark and serialization
/// pre-computation to avoid code duplication. Passes actual extracted
/// citations into back-matter generation to match production pipeline behavior.
///
/// Returns `Result` to propagate errors instead of panicking on recoverable
/// failures.
fn build_catalog_envelope(
    doc: &forge::model::PolicyDocument,
) -> Result<forge::oscal::CatalogEnvelope, forge::ForgeError> {
    let mut trace_links = forge::TraceLinkCollection::new();
    let mut catalog = forge::oscal::build_catalog(doc, Some(&mut trace_links))?;
    forge::oscal::trace_embedding::embed_trace_in_catalog(&mut catalog, &trace_links);
    let metadata = forge::oscal::assemble_metadata(&doc.metadata, None)?;
    let citations = forge::oscal::component_definition::collect_all_citations(&doc.sections);
    let (back_matter_resources, _) = forge::oscal::generate_back_matter(&citations)?;
    let back_matter = if back_matter_resources.is_empty() {
        None
    } else {
        Some(forge::BackMatter { resources: back_matter_resources })
    };
    Ok(forge::oscal::CatalogEnvelope {
        catalog: forge::oscal::OscalCatalog {
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
        },
    })
}

// ─── Per-Stage Benchmarks ───────────────────────────────────────────────

/// Benchmark each pipeline stage independently.
///
/// Pre-computes input for each stage outside the benchmark loop so only
/// the stage under measurement runs inside `b.iter()`. Uses `black_box()`
/// on all inputs and outputs to prevent compiler optimization.
///
/// Stages:
/// 1. ingest — `ingest_file()` + `reconstruct_content()`
/// 2. parse — `extract_sections()` + `extract_clauses()`
/// 3. atomize — `assemble_document()` + `atomize_document()` + `assign_stable_ids()` + `extract_citations()`
/// 4. `catalog_assembly` — `build_catalog()` + trace + metadata + `back_matter` + envelope
/// 5. serialization — `serde_json::to_string_pretty()`
fn bench_per_stage(c: &mut Criterion) {
    let fixture_path = Path::new(FIXTURE_PATH);
    assert!(fixture_path.exists(), "Synthetic fixture must exist at {FIXTURE_PATH}");

    let mut group = c.benchmark_group("pipeline_stages");

    // ── Stage 1: Ingest ──
    group.bench_function("ingest", |b| {
        b.iter(|| {
            let ingested =
                forge::ingest::ingest_file(black_box(fixture_path), MAX_SIZE_BYTES).unwrap();
            let content = ingested.reconstruct_content();
            black_box(content)
        });
    });

    // Pre-compute ingest output for parse stage
    let ingested = forge::ingest::ingest_file(fixture_path, MAX_SIZE_BYTES).unwrap();
    let content = ingested.reconstruct_content();

    // ── Stage 2: Parse ──
    group.bench_function("parse", |b| {
        b.iter(|| {
            let sections = forge::parse::extract_sections(black_box(&content)).unwrap();
            let clauses = forge::parse::extract_clauses(black_box(&content)).unwrap();
            black_box((sections, clauses))
        });
    });

    // Pre-compute parse output for atomize stage
    let sections = forge::parse::extract_sections(&content).unwrap();
    let clauses = forge::parse::extract_clauses(&content).unwrap();
    assert!(!sections.is_empty(), "fixture produced no sections");
    assert!(!clauses.list_items.is_empty(), "fixture produced no list-item requirements");

    // ── Stage 3: Atomize ──
    group.bench_function("atomize", |b| {
        b.iter(|| {
            let document = forge::model::assemble_document(
                black_box(&ingested),
                black_box(&sections),
                black_box(&clauses),
            )
            .unwrap();
            let atomized = forge::parse::atomize_document(black_box(&document)).unwrap();
            let doc = forge::uuid::assign_stable_ids(atomized);
            let doc = forge::citation::extract_citations(doc).unwrap();
            black_box(doc)
        });
    });

    let document = forge::model::assemble_document(&ingested, &sections, &clauses).unwrap();
    assert!(document.total_requirements() > 0, "fixture assembled no requirements");
    let atomized = forge::parse::atomize_document(&document).unwrap();
    let doc_for_catalog = forge::uuid::assign_stable_ids(atomized);
    let doc_for_catalog = forge::citation::extract_citations(doc_for_catalog).unwrap();

    // ── Stage 4: Catalog Assembly ──
    group.bench_function("catalog_assembly", |b| {
        b.iter(|| {
            let envelope = build_catalog_envelope(black_box(&doc_for_catalog)).unwrap();
            black_box(envelope)
        });
    });

    // Pre-compute catalog envelope for serialization stage
    let envelope = build_catalog_envelope(&doc_for_catalog).unwrap();

    // ── Stage 5: Serialization (JSON) ──
    group.bench_function("serialization_json", |b| {
        b.iter(|| {
            let json = serde_json::to_string_pretty(black_box(&envelope)).unwrap();
            black_box(json)
        });
    });

    // ── Stage 5b: Serialization (YAML) — WI-27 ──
    group.bench_function("serialization_yaml", |b| {
        b.iter(|| {
            let yaml = forge::export::yaml::serialize_to_yaml(black_box(&envelope)).unwrap();
            black_box(yaml)
        });
    });

    group.finish();
}

criterion_group!(benches, bench_full_pipeline, bench_per_stage);
criterion_main!(benches);
