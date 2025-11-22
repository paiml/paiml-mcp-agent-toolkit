//! Top-K Selection Benchmarks (Phase 5, Task 5.4)
//!
//! Compares heap-based vs Arrow-based (trueno-db) Top-K selection.
//!
//! **Target**: Validate 5-28x speedup claims from specification.
//!
//! # Benchmark Groups
//!
//! 1. Small datasets (1K rows) - warmup
//! 2. Medium datasets (10K, 100K rows) - typical workload
//! 3. Large datasets (1M rows) - stress test
//!
//! # Expected Results
//!
//! - Small (1K): Heap competitive (conversion overhead dominates)
//! - Medium (10-100K): Arrow 2-5x faster (SIMD kicks in)
//! - Large (1M): Arrow 5-28x faster (full SIMD/GPU utilization)

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use pmat::services::analytics_top_k::TopKSelector;
use std::hint::black_box;

/// Generate dataset for benchmarking
fn generate_dataset(size: usize) -> Vec<i64> {
    (0..size as i64).rev().collect() // Worst case: descending order
}

/// Benchmark heap-based Top-K selection (baseline)
fn bench_heap_topk(c: &mut Criterion) {
    let mut group = c.benchmark_group("topk_heap");

    for size in [1_000, 10_000, 100_000, 1_000_000] {
        group.throughput(Throughput::Elements(size as u64));

        let data = generate_dataset(size);
        let selector = TopKSelector::new(100);

        group.bench_with_input(BenchmarkId::from_parameter(size), &data, |b, data| {
            b.iter(|| {
                let result = selector.select(black_box(data));
                black_box(result);
            });
        });
    }

    group.finish();
}

/// Benchmark Arrow-based Top-K selection (trueno-db)
#[cfg(feature = "analytics-simd")]
fn bench_arrow_topk(c: &mut Criterion) {
    use pmat::services::analytics_top_k::select_top_k_arrow;

    let mut group = c.benchmark_group("topk_arrow");

    for size in [1_000, 10_000, 100_000, 1_000_000] {
        group.throughput(Throughput::Elements(size as u64));

        let data = generate_dataset(size);

        group.bench_with_input(BenchmarkId::from_parameter(size), &data, |b, data| {
            b.iter(|| {
                let result = select_top_k_arrow(black_box(data), 100).unwrap();
                black_box(result);
            });
        });
    }

    group.finish();
}

/// Benchmark unified Top-K selection with automatic backend selection
fn bench_unified_topk(c: &mut Criterion) {
    use pmat::services::analytics_top_k::select_top_k;

    let mut group = c.benchmark_group("topk_unified");

    for size in [1_000, 10_000, 100_000, 1_000_000] {
        group.throughput(Throughput::Elements(size as u64));

        let data = generate_dataset(size);

        group.bench_with_input(BenchmarkId::from_parameter(size), &data, |b, data| {
            b.iter(|| {
                let result = select_top_k(black_box(data), 100).unwrap();
                black_box(result);
            });
        });
    }

    group.finish();
}

/// Compare heap vs Arrow at various scales
#[cfg(feature = "analytics-simd")]
fn bench_topk_comparison(c: &mut Criterion) {
    use pmat::services::analytics_top_k::select_top_k_arrow;

    let mut group = c.benchmark_group("topk_comparison");

    for size in [1_000, 10_000, 100_000, 1_000_000] {
        group.throughput(Throughput::Elements(size as u64));

        let data = generate_dataset(size);
        let selector = TopKSelector::new(100);

        group.bench_with_input(
            BenchmarkId::new("heap", size),
            &data,
            |b, data| {
                b.iter(|| {
                    let result = selector.select(black_box(data));
                    black_box(result);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("arrow", size),
            &data,
            |b, data| {
                b.iter(|| {
                    let result = select_top_k_arrow(black_box(data), 100).unwrap();
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

#[cfg(feature = "analytics-simd")]
criterion_group!(
    benches,
    bench_heap_topk,
    bench_arrow_topk,
    bench_unified_topk,
    bench_topk_comparison
);

#[cfg(not(feature = "analytics-simd"))]
criterion_group!(benches, bench_heap_topk, bench_unified_topk);

criterion_main!(benches);
