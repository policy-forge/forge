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

/// Build production catalog JSON from the synthetic fixture.
///
/// Setup is intentionally outside the timed export loop, but it uses the
/// production pipeline rather than a hand-maintained envelope mirror (F0031).
fn build_catalog_json(fixture_path: &Path) -> String {
    forge::pipeline::run_catalog_pipeline(
        fixture_path,
        DEFAULT_MAX_SIZE_BYTES,
        &forge::cli::OutputFormat::Json,
        None,
    )
    .expect("bench setup: production catalog pipeline failed")
    .content
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
