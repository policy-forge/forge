// Contract: Performance Benchmark API (WI-24)
//
// This file defines the interfaces for the benchmark suite.
// It is a design artifact — not compiled code.
// Implementation in benches/pipeline_benchmark.rs must conform to these signatures.

// ─── Fixture Generator ─────────────────────────────────────────────────

/// Generate a deterministic 50-page synthetic Markdown policy document.
///
/// # Content
/// - YAML frontmatter (title, version, author, date — all hardcoded)
/// - 10 H2 sections (Access Control, Data Protection, Incident Response, etc.)
/// - ~40 H3 subsections (3-5 per H2)
/// - ~200 numbered policy requirements (normative language: "shall", "must")
/// - ~20 compound statements ("must X and must Y")
/// - ~30 citations/references ("[NIST SP 800-53 AC-2]", "See Section 3.2")
/// - ~10 tables (role-responsibility matrices, retention schedules)
/// - ~25,000 words / ~150,000 characters
///
/// # Determinism
/// No randomness, no system time, no RNG. Byte-identical across invocations.
pub fn generate_synthetic_policy() -> String;

// ─── Benchmark Functions ────────────────────────────────────────────────

/// Full catalog pipeline benchmark.
///
/// Measures: ingest → parse → assemble → atomize → UUIDs → citations →
///           catalog → traces → metadata → back matter → serialize
///
/// Configuration: 100 samples, 10s measurement time
/// Input: tests/fixtures/synthetic-50page-policy.md (committed static file)
/// Output: criterion report (mean, std dev, throughput)
pub fn bench_full_pipeline(c: &mut Criterion);

/// Per-stage benchmarks with pre-computed inputs.
///
/// Stages:
///   1. ingest    — ingest_file() + reconstruct_content()
///   2. parse     — extract_sections() + extract_clauses()
///   3. atomize   — assemble_document() + atomize_document() + assign_stable_ids() + extract_citations()
///   4. catalog_assembly — build_catalog() + embed_trace + assemble_metadata + back_matter + envelope
///   5. serialization    — serde_json::to_string_pretty()
///
/// Note: validation stage omitted (src/validate/mod.rs is empty stub)
pub fn bench_per_stage(c: &mut Criterion);

// ─── Pipeline Helper ────────────────────────────────────────────────────

/// Runs the full catalog pipeline and returns the JSON string.
///
/// Mirrors src/pipeline.rs::run_catalog_pipeline but without file I/O output.
/// Composes public API functions in the same order as the production pipeline:
///
/// 1. forge::ingest::ingest_file(path, 10MB)
/// 2. IngestedDocument::reconstruct_content()
/// 3. forge::parse::extract_sections(&content)
/// 4. forge::parse::extract_clauses(&content)
/// 5. forge::model::assemble_document(&ingested, &sections, &clauses)
/// 6. forge::parse::atomize_document(&document)
/// 7. forge::uuid::assign_stable_ids(&mut doc)
/// 8. forge::citation::extract_citations(&mut doc)
/// 9. forge::oscal::build_catalog(&doc, Some(&mut trace_links))
/// 10. forge::oscal::trace_embedding::embed_trace_in_catalog(&mut catalog, &trace_links)
/// 11. forge::oscal::assemble_metadata(&doc.metadata, None)
/// 12. forge::oscal::generate_back_matter(&[])
/// 13. CatalogEnvelope assembly
/// 14. serde_json::to_string_pretty(&envelope)
pub fn run_full_catalog_pipeline(
    fixture_path: &std::path::Path,
) -> Result<String, forge::ForgeError>;

// ─── Cargo.toml Entry ───────────────────────────────────────────────────

// [[bench]]
// name = "pipeline_benchmark"
// harness = false
