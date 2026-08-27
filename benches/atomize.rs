//! Criterion benchmarks for the atomization pipeline.

#[path = "common/atomize.rs"]
mod atomize_fixture;

use std::hint::black_box;
use std::path::PathBuf;

use atomize_fixture::make_req;
use criterion::{Criterion, criterion_group, criterion_main};
use forge::model::{DocumentMetadata, PolicyDocument, PolicySection};
use forge::parse::{atomize_document, atomize_requirement, preliminary_id};

fn bench_atomize_compound(c: &mut Criterion) {
    let req = make_req("Systems must enforce MFA and must require complex passwords", 1);
    c.bench_function("atomize_requirement/compound_2part", |b| {
        b.iter(|| {
            atomize_requirement(black_box(&req))
                .expect("atomize_requirement/compound_2part: fixture must atomize")
        });
    });
}

fn bench_atomize_atomic(c: &mut Criterion) {
    let req = make_req("All systems must enforce MFA", 1);
    c.bench_function("atomize_requirement/atomic_passthrough", |b| {
        b.iter(|| {
            atomize_requirement(black_box(&req))
                .expect("atomize_requirement/atomic_passthrough: fixture must atomize")
        });
    });
}

fn bench_atomize_document_100(c: &mut Criterion) {
    let mut requirements = Vec::with_capacity(100);
    for i in 0..100 {
        let text = if i % 2 == 0 {
            format!("System {i} must enforce MFA and must require complex passwords")
        } else {
            format!("System {i} should notify administrators of policy violations")
        };
        let mut requirement = make_req(&text, i + 1);
        requirement.nesting_depth = [0_u8, 1, 2][i % 3];
        requirement.stable_id = Some(format!("bench-req-{i}"));

        if i % 10 == 0 {
            requirement.citations.push(forge::model::Citation {
                id: format!("citation-{i}"),
                text: format!("NIST SP 800-53 AC-{i}"),
                url: Some(format!("https://example.test/controls/{i}")),
                source_requirement_id: requirement.stable_id.clone().map(Into::into),
            });
        }
        if i % 15 == 0 {
            requirement.parameters.push(forge::model::PolicyParameter {
                id: format!("bench-req-{i}_prm_0"),
                requirement_id: format!("bench-req-{i}").into(),
                label: "within 30 days".to_string(),
                value: "30 days".to_string(),
                parameter_type: forge::model::ParameterType::TimeWindow,
                constraint: None,
            });
            requirement.parameters_extracted = true;
        }
        requirements.push(requirement);
    }
    let doc = PolicyDocument {
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
            title: "Controls".to_string(),
            heading_level: 1,
            source_line: 1,
            body_text: None,
            children: vec![],
            requirements,
        }],
    };
    c.bench_function("atomize_document/100_mixed_requirements", |b| {
        b.iter(|| {
            atomize_document(black_box(&doc))
                .expect("atomize_document/100_mixed_requirements: fixture must atomize")
        });
    });
}

fn bench_preliminary_id(c: &mut Criterion) {
    c.bench_function("preliminary_id/throughput", |b| {
        b.iter(|| {
            preliminary_id(black_box("Systems must enforce MFA"), black_box(42), black_box(0))
        });
    });
}

criterion_group!(
    benches,
    bench_atomize_compound,
    bench_atomize_atomic,
    bench_atomize_document_100,
    bench_preliminary_id,
);
criterion_main!(benches);
