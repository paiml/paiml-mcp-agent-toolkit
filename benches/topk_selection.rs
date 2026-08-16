//! Top-K Selection Benchmarks (Phase 5, Task 5.4)
//!
//! Compares heap-based vs Arrow-based (trueno-db) Top-K selection.
//!
//! **Target**: Validate 5-28x speedup claims from specification.
//!
//! Requires the `analytics-simd` feature.
//! Run with: cargo bench --bench topk_selection --features analytics-simd

#[cfg(feature = "analytics-simd")]
mod bench {
    use criterion::{criterion_group, BenchmarkId, Criterion, Throughput};
    use pmat::services::analytics_top_k::{select_top_k, select_top_k_arrow, TopKSelector};
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
    fn bench_arrow_topk(c: &mut Criterion) {
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
    fn bench_topk_comparison(c: &mut Criterion) {
        let mut group = c.benchmark_group("topk_comparison");

        for size in [1_000, 10_000, 100_000, 1_000_000] {
            group.throughput(Throughput::Elements(size as u64));

            let data = generate_dataset(size);
            let selector = TopKSelector::new(100);

            group.bench_with_input(BenchmarkId::new("heap", size), &data, |b, data| {
                b.iter(|| {
                    let result = selector.select(black_box(data));
                    black_box(result);
                });
            });

            group.bench_with_input(BenchmarkId::new("arrow", size), &data, |b, data| {
                b.iter(|| {
                    let result = select_top_k_arrow(black_box(data), 100).unwrap();
                    black_box(result);
                });
            });
        }

        group.finish();
    }

    criterion_group!(
        benches,
        bench_heap_topk,
        bench_arrow_topk,
        bench_unified_topk,
        bench_topk_comparison
    );
}

// `criterion_main!` expands to `fn main()`, which cargo requires at CRATE level
// for a `harness = false` bench. It used to sit inside `mod bench`, so with
// `analytics-simd` enabled it defined `bench::main` and the crate had no `main`
// at all:
//
//     $ cargo bench --bench topk_selection --features analytics-simd
//     error[E0601]: `main` function not found in crate `topk_selection`
//
// That is the exact command this file's own header tells you to run, so this
// benchmark had never executed once under the feature it requires — and its
// stated job is to "validate 5-28x speedup claims from specification". Those
// claims have never been measured by it. Only the `not(analytics-simd)` arm
// compiled, and that arm is the empty stub, so `cargo bench` was quietly
// benchmarking nothing.
//
// Found because `cargo clippy --all-targets --features full` aborts here (#1011);
// `--lib` alone never builds bench targets, the same blind spot as #1005.
#[cfg(feature = "analytics-simd")]
criterion::criterion_main!(bench::benches);

#[cfg(not(feature = "analytics-simd"))]
fn main() {
    // This benchmark requires the analytics-simd feature.
    // Run with: cargo bench --bench topk_selection --features analytics-simd
}
