use criterion::{criterion_group, criterion_main, Criterion};
use pmat::services::dead_code_analyzer::DeadCodeAnalyzer;
use std::hint::black_box;

/// Benchmark dead code analysis with minimal setup (10K capacity)
fn bench_dead_code_10k(c: &mut Criterion) {
    c.bench_function("dead_code_analysis_10k", |b| {
        b.iter(|| {
            let analyzer = black_box(DeadCodeAnalyzer::new(10_000));
            black_box(analyzer);
        });
    });
}

/// Benchmark dead_code analysis with medium setup (50K capacity) - realistic large project
fn bench_dead_code_50k(c: &mut Criterion) {
    c.bench_function("dead_code_analysis_50k", |b| {
        b.iter(|| {
            let analyzer = black_box(DeadCodeAnalyzer::new(50_000));
            black_box(analyzer);
        });
    });
}

/// Benchmark dead code analysis with large setup (100K capacity) - stress test
#[cfg(feature = "simd")]
fn bench_dead_code_100k_simd(c: &mut Criterion) {
    c.bench_function("dead_code_analysis_100k_simd", |b| {
        b.iter(|| {
            let analyzer = black_box(DeadCodeAnalyzer::new(100_000));
            black_box(analyzer);
        });
    });
}

#[cfg(feature = "simd")]
criterion_group!(
    benches,
    bench_dead_code_10k,
    bench_dead_code_50k,
    bench_dead_code_100k_simd
);

#[cfg(not(feature = "simd"))]
criterion_group!(benches, bench_dead_code_10k, bench_dead_code_50k);

criterion_main!(benches);
