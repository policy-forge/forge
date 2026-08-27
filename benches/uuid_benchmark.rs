use criterion::{Criterion, criterion_group, criterion_main};
use forge::uuid::{generate_stable_id, normalize_for_hashing};

const PADDED_SAMPLE: &str = "  All  users  must  use  multi-factor  authentication  ";
const LONG_SAMPLE: &str = "Organizations shall implement comprehensive security controls including but not limited to multi-factor authentication, role-based access control, encryption at rest and in transit, continuous monitoring, incident response procedures, and regular security assessments to ensure compliance with applicable regulatory requirements";

fn bench_normalize_for_hashing(c: &mut Criterion) {
    let text = PADDED_SAMPLE;
    c.bench_function("normalize_for_hashing", |b| {
        b.iter(|| normalize_for_hashing(std::hint::black_box(text)));
    });
}

fn bench_generate_stable_id(c: &mut Criterion) {
    let text = PADDED_SAMPLE;
    c.bench_function("generate_stable_id", |b| {
        b.iter(|| generate_stable_id(std::hint::black_box(text)));
    });
}

fn bench_generate_stable_id_long_text(c: &mut Criterion) {
    let text = LONG_SAMPLE;
    c.bench_function("generate_stable_id_long", |b| {
        b.iter(|| generate_stable_id(std::hint::black_box(text)));
    });
}

criterion_group!(
    benches,
    bench_normalize_for_hashing,
    bench_generate_stable_id,
    bench_generate_stable_id_long_text
);
criterion_main!(benches);
