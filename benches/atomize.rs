//! Criterion benchmarks for the atomization pipeline.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use forge::model::{PolicyDocument, PolicyRequirement, PolicySection};
use forge::parse::{atomize_document, atomize_requirement, preliminary_id};

// NOTE: mirrors tests/common/mod.rs — kept local since benches cannot import test modules
fn make_req(text: &str, source_line: usize) -> PolicyRequirement {
    PolicyRequirement {
        stable_id: String::new(),
        text: text.to_string(),
        source_line,
        atom_index: 0,
        parent_text: None,
    }
}

fn bench_atomize_compound(c: &mut Criterion) {
    let req = make_req("Systems must enforce MFA and must require complex passwords", 1);
    c.bench_function("atomize_requirement/compound_2part", |b| {
        b.iter(|| atomize_requirement(black_box(&req)).unwrap());
    });
}

fn bench_atomize_atomic(c: &mut Criterion) {
    let req = make_req("All systems must enforce MFA", 1);
    c.bench_function("atomize_requirement/atomic_passthrough", |b| {
        b.iter(|| atomize_requirement(black_box(&req)).unwrap());
    });
}

fn bench_atomize_document_100(c: &mut Criterion) {
    let mut requirements = Vec::with_capacity(100);
    for i in 0..50 {
        requirements
            .push(make_req("Systems must enforce MFA and must require complex passwords", i + 1));
    }
    for i in 50..100 {
        requirements.push(make_req("All systems must enforce MFA", i + 1));
    }
    let doc = PolicyDocument {
        title: "Benchmark Policy".to_string(),
        sections: vec![PolicySection { heading: "Controls".to_string(), requirements }],
    };
    c.bench_function("atomize_document/100_mixed_requirements", |b| {
        b.iter(|| atomize_document(black_box(&doc)).unwrap());
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
