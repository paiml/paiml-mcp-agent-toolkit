//! Benchmarks for TDG calculator operations (variance, Gini coefficient)
//!
//! Run with: cargo bench --bench tdg_ops --features simd
//!
//! Expected results:
//! - SIMD should be 2-4x faster than scalar for variance calculations
//! - Gini coefficient benefits from SIMD mul() + sum() operations

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

/// Scalar implementation of variance
fn variance_scalar(values: &[u32]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let sum: u32 = values.iter().sum();
    let mean = f64::from(sum) / values.len() as f64;
    let squared_diff_sum: f64 = values
        .iter()
        .map(|&c| (f64::from(c) - mean).powi(2))
        .sum();
    squared_diff_sum / values.len() as f64
}

/// SIMD implementation of variance using Trueno
#[cfg(feature = "simd")]
fn variance_simd(values: &[u32]) -> f64 {
    use trueno::Vector;

    if values.is_empty() {
        return 0.0;
    }

    // Convert u32 to f32 for Trueno
    let values_f32: Vec<f32> = values.iter().map(|&x| x as f32).collect();
    let vec = Vector::from_slice(&values_f32);

    match vec.variance() {
        Ok(var) => var as f64,
        Err(_) => 0.0,
    }
}

/// Scalar implementation of Gini coefficient
fn gini_scalar(values: &[u32]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let sum: u32 = values.iter().sum();
    if sum == 0 {
        return 0.0;
    }

    let mut sorted = values.to_vec();
    sorted.sort_unstable();

    let n = sorted.len();
    let gini_sum: f64 = sorted
        .iter()
        .enumerate()
        .map(|(i, &value)| (2.0 * (i + 1) as f64 - n as f64 - 1.0) * f64::from(value))
        .sum();

    gini_sum / (n as f64 * f64::from(sum))
}

/// SIMD implementation of Gini coefficient using Trueno
#[cfg(feature = "simd")]
fn gini_simd(values: &[u32]) -> f64 {
    use trueno::Vector;

    if values.is_empty() {
        return 0.0;
    }

    let sum: u32 = values.iter().sum();
    if sum == 0 {
        return 0.0;
    }

    let mut sorted = values.to_vec();
    sorted.sort_unstable();

    let n = sorted.len();

    // Create weights vector: (2*(i+1) - n - 1) for i in 0..n
    let weights: Vec<f32> = (0..n)
        .map(|i| 2.0 * (i + 1) as f32 - n as f32 - 1.0)
        .collect();
    let values_f32: Vec<f32> = sorted.iter().map(|&x| x as f32).collect();

    let weights_vec = Vector::from_slice(&weights);
    let values_vec = Vector::from_slice(&values_f32);

    // Element-wise multiply and sum
    match weights_vec.mul(&values_vec) {
        Ok(product) => match product.sum() {
            Ok(gini_sum) => (gini_sum as f64) / (n as f64 * f64::from(sum)),
            Err(_) => gini_scalar(values),
        },
        Err(_) => gini_scalar(values),
    }
}

/// Benchmark variance at different vector sizes
fn bench_variance(c: &mut Criterion) {
    let sizes = [100, 500, 1000, 5000, 10000];

    let mut group = c.benchmark_group("variance");

    for size in sizes {
        // Generate test values (complexity scores)
        let values: Vec<u32> = (0..size).map(|i| (i % 100 + 1) as u32).collect();

        // Benchmark scalar
        group.bench_with_input(BenchmarkId::new("scalar", size), &size, |b, _| {
            b.iter(|| variance_scalar(black_box(&values)))
        });

        // Benchmark SIMD (only when feature is enabled)
        #[cfg(feature = "simd")]
        group.bench_with_input(BenchmarkId::new("simd", size), &size, |b, _| {
            b.iter(|| variance_simd(black_box(&values)))
        });
    }

    group.finish();
}

/// Benchmark Gini coefficient at different vector sizes
fn bench_gini(c: &mut Criterion) {
    let sizes = [100, 500, 1000, 5000, 10000];

    let mut group = c.benchmark_group("gini_coefficient");

    for size in sizes {
        // Generate test values (complexity scores with some variation)
        let values: Vec<u32> = (0..size).map(|i| ((i * 7 + 3) % 100 + 1) as u32).collect();

        // Benchmark scalar
        group.bench_with_input(BenchmarkId::new("scalar", size), &size, |b, _| {
            b.iter(|| gini_scalar(black_box(&values)))
        });

        // Benchmark SIMD (only when feature is enabled)
        #[cfg(feature = "simd")]
        group.bench_with_input(BenchmarkId::new("simd", size), &size, |b, _| {
            b.iter(|| gini_simd(black_box(&values)))
        });
    }

    group.finish();
}

/// Benchmark batch operations (multiple variance calculations)
fn bench_batch_variance(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_variance");

    let num_batches = 100;
    let vector_size = 1000;

    // Generate test value batches
    let batches: Vec<Vec<u32>> = (0..num_batches)
        .map(|i| {
            (0..vector_size)
                .map(|j| ((i * vector_size + j) % 100 + 1) as u32)
                .collect()
        })
        .collect();

    // Benchmark scalar batch
    group.bench_function("scalar_batch_100x1000", |b| {
        b.iter(|| {
            let mut results = Vec::with_capacity(num_batches);
            for values in &batches {
                results.push(variance_scalar(black_box(values)));
            }
            results
        })
    });

    // Benchmark SIMD batch
    #[cfg(feature = "simd")]
    group.bench_function("simd_batch_100x1000", |b| {
        b.iter(|| {
            let mut results = Vec::with_capacity(num_batches);
            for values in &batches {
                results.push(variance_simd(black_box(values)));
            }
            results
        })
    });

    group.finish();
}

/// Benchmark to show f32 conversion overhead for variance
#[cfg(feature = "simd")]
fn bench_conversion_overhead(c: &mut Criterion) {
    let sizes = [1000, 10000];

    let mut group = c.benchmark_group("variance_conversion_overhead");

    for size in sizes {
        let values: Vec<u32> = (0..size).map(|i| (i % 100 + 1) as u32).collect();

        // Benchmark u32 to f32 conversion
        group.bench_with_input(BenchmarkId::new("u32_to_f32", size), &size, |b, _| {
            b.iter(|| {
                let _: Vec<f32> = black_box(&values).iter().map(|&x| x as f32).collect();
            })
        });
    }

    group.finish();
}

#[cfg(feature = "simd")]
criterion_group!(
    benches,
    bench_variance,
    bench_gini,
    bench_batch_variance,
    bench_conversion_overhead
);

#[cfg(not(feature = "simd"))]
criterion_group!(benches, bench_variance, bench_gini, bench_batch_variance);

criterion_main!(benches);
