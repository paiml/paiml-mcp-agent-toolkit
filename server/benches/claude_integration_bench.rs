// Performance benchmarks for Claude integration
// Uses Criterion for statistical analysis

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use pmat::claude_integration::{AnalysisResult, TwoTierCache};
use std::sync::Arc;

/// Benchmark cache performance with realistic access patterns
fn benchmark_cache_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("claude_cache");

    // Configure for statistical significance
    group.sample_size(100);

    // Benchmark L1 cache hit
    group.bench_function("l1_cache_hit", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let cache = Arc::new(TwoTierCache::new());

        // Pre-populate cache
        rt.block_on(async {
            cache
                .get_with_loader("test_key", || async {
                    AnalysisResult {
                        complexity: 10,
                        cognitive_complexity: 8,
                        satd_count: 0,
                        ..Default::default()
                    }
                })
                .await;
        });

        b.to_async(&rt).iter(|| async {
            let result = cache
                .get_with_loader("test_key", || async {
                    // This should never be called due to cache hit
                    AnalysisResult::default()
                })
                .await;
            black_box(result);
        });
    });

    // Benchmark cache miss with loader
    group.bench_function("cache_miss_with_load", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let cache = Arc::new(TwoTierCache::new());
        let mut counter = 0u64;

        b.to_async(&rt).iter(|| {
            let cache = Arc::clone(&cache);
            let key = format!("unique_key_{}", counter);
            counter += 1;

            async move {
                let result = cache
                    .get_with_loader(&key, || async {
                        // Simulate analysis work
                        AnalysisResult {
                            complexity: 15,
                            cognitive_complexity: 10,
                            satd_count: 2,
                            ..Default::default()
                        }
                    })
                    .await;
                black_box(result);
            }
        });
    });

    // Benchmark with different cache sizes
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let cache = Arc::new(TwoTierCache::new());

            // Pre-populate with `size` entries
            rt.block_on(async {
                for i in 0..size {
                    cache
                        .get_with_loader(&format!("key_{}", i), || async {
                            AnalysisResult::default()
                        })
                        .await;
                }
            });

            b.to_async(&rt).iter(|| {
                let cache = Arc::clone(&cache);
                async move {
                    // Access random key (Zipfian-like pattern)
                    let key_idx = (size / 10).max(1); // Access hot keys
                    let result = cache
                        .get_with_loader(&format!("key_{}", key_idx), || async {
                            AnalysisResult::default()
                        })
                        .await;
                    black_box(result);
                }
            });
        });
    }

    group.finish();
}

/// Benchmark hash function performance
fn benchmark_hash_function(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_function");

    let test_keys = vec![
        "short",
        "medium_length_key_here",
        "very_long_key_with_lots_of_characters_that_represents_a_file_path_or_similar",
    ];

    for key in test_keys {
        group.bench_with_input(BenchmarkId::from_parameter(key.len()), key, |b, key| {
            let cache = TwoTierCache::new();
            b.iter(|| {
                black_box(cache.hash_key(key));
            });
        });
    }

    group.finish();
}

/// Benchmark concurrent cache access
fn benchmark_concurrent_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_cache");

    for thread_count in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(thread_count),
            thread_count,
            |b, &threads| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let cache = Arc::new(TwoTierCache::new());

                b.to_async(&rt).iter(|| {
                    let cache = Arc::clone(&cache);
                    async move {
                        let mut handles = Vec::new();

                        for i in 0..threads {
                            let cache = Arc::clone(&cache);
                            let handle = tokio::spawn(async move {
                                cache
                                    .get_with_loader(&format!("key_{}", i % 10), || async {
                                        AnalysisResult::default()
                                    })
                                    .await;
                            });
                            handles.push(handle);
                        }

                        for handle in handles {
                            handle.await.unwrap();
                        }
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark memory allocation patterns
fn benchmark_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");

    group.bench_function("analysis_result_creation", |b| {
        b.iter(|| {
            let result = AnalysisResult {
                complexity: black_box(15),
                cognitive_complexity: black_box(10),
                satd_count: black_box(2),
                ..Default::default()
            };
            black_box(result);
        });
    });

    group.bench_function("arc_clone_overhead", |b| {
        let result = Arc::new(AnalysisResult::default());
        b.iter(|| {
            let cloned = Arc::clone(&result);
            black_box(cloned);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_cache_performance,
    benchmark_hash_function,
    benchmark_concurrent_access,
    benchmark_memory_allocation
);
criterion_main!(benches);
