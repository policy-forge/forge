//! Criterion benchmarks for parameter extraction (T040, WI-34).
//!
//! Measures `extract_parameters()` on a narrow synthetic `PolicyDocument` corpus with 500
//! requirements (a fixed mix of parameterized and non-parameterized text).
//!
//! The timings are a regression signal for this corpus; they do not establish p95 latency for
//! production policies or empirically prove linear-time behavior for all inputs.

use std::hint::black_box;
use std::path::PathBuf;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};

use forge::model::{DocumentMetadata, PolicyDocument, PolicyRequirement, PolicySection};
use forge::parameter::extract_parameters;

/// Parameterized requirement texts covering all four matcher types.
const PARAMETERIZED: &[&str] = &[
    "Users must change passwords within 30 days of provisioning.",
    "Sessions must expire after no more than 15 minutes of inactivity.",
    "Encryption keys must be rotated at least annually.",
    "MFA must require no fewer than 3 authentication factors.",
    "Audit logs must be reviewed at least quarterly.",
    "Passwords must be at least 12 characters in length.",
    "Backups must complete within 4 hours of the scheduled window.",
    "Certificates must be renewed every 90 days before expiration.",
    "Systems must scan for vulnerabilities at least weekly.",
    "Access reviews must occur after no more than 6 months.",
];

/// Non-parameterized requirement texts (no extractable patterns).
const PLAIN: &[&str] = &[
    "All users must authenticate before accessing systems.",
    "Access must follow the principle of least privilege.",
    "Role-based access control must be enforced across all services.",
    "Sensitive data must be classified before storage.",
    "Security incidents must be reported to the security team.",
    "Third-party integrations must undergo security review.",
    "Vendor access must be controlled and audited.",
];

/// Build a `PolicyRequirement` with the given text and a stable ID.
fn make_req(i: usize, text: &str) -> PolicyRequirement {
    PolicyRequirement {
        stable_id: Some(format!("bench-req-{i:04}")),
        text: text.to_string(),
        source_line: i + 1,
        nesting_depth: 0,
        atom_index: i,
        parent_text: None,
        citations: vec![],
        modality: None,
        parameters: vec![],
        parameters_extracted: false,
    }
}

/// Build a document with `n` requirements: every 3rd is parameterized, rest plain.
fn make_synthetic_document(n: usize) -> PolicyDocument {
    let requirements: Vec<PolicyRequirement> = (0..n)
        .map(|i| {
            if i % 3 == 0 {
                make_req(i, PARAMETERIZED[i % PARAMETERIZED.len()])
            } else {
                make_req(i, PLAIN[i % PLAIN.len()])
            }
        })
        .collect();

    PolicyDocument {
        id: "bench".to_string(),
        metadata: DocumentMetadata {
            title: "Benchmark Policy".to_string(),
            version: "0.0.0".to_string(),
            author: None,
            date: None,
            source_path: PathBuf::from("bench.md"),
            content_hash: None,
        },
        sections: vec![PolicySection {
            title: "Benchmark Section".to_string(),
            heading_level: 1,
            source_line: 1,
            body_text: None,
            children: vec![],
            requirements,
        }],
    }
}

/// Benchmark parameter extraction with fresh documents prepared outside the timed region.
fn bench_extract_parameters(c: &mut Criterion, name: &str, requirement_count: usize) {
    let doc = make_synthetic_document(requirement_count);

    c.bench_function(name, |b| {
        b.iter_batched(
            || doc.clone(),
            |mut document| {
                black_box(extract_parameters(&mut document))
                    .expect("parameter extraction benchmark must not fail");
            },
            BatchSize::SmallInput,
        );
    });
}

/// Benchmark the 500-requirement synthetic corpus.
fn bench_extract_parameters_500(c: &mut Criterion) {
    bench_extract_parameters(c, "extract_parameters/500_requirements", 500);
}

/// Benchmark the 100-requirement synthetic corpus.
fn bench_extract_parameters_100(c: &mut Criterion) {
    bench_extract_parameters(c, "extract_parameters/100_requirements", 100);
}

/// Benchmark one requirement as a baseline.
fn bench_extract_parameters_single(c: &mut Criterion) {
    bench_extract_parameters(c, "extract_parameters/1_requirement", 1);
}

criterion_group!(
    benches,
    bench_extract_parameters_500,
    bench_extract_parameters_100,
    bench_extract_parameters_single,
);
criterion_main!(benches);
