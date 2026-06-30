//! Benchmarks do Safe-Core AGI
//!
//! Para executar: cargo bench --workspace

use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_hashes(c: &mut Criterion) {
    let mut group = c.benchmark_group("Hashing");
    group.bench_function("dummy", |b| {
        b.iter(|| {
            std::hint::black_box(1 + 1)
        })
    });
    group.finish();
}

criterion_group!(
    benches,
    benchmark_hashes,
);
criterion_main!(benches);
