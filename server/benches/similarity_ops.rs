//! Benchmarks for similarity operations (cosine similarity, entropy)
//!
//! Run with: cargo bench --bench similarity_ops --features simd
//!
//! Expected results:
//! - SIMD should be 2-8x faster than scalar for large vectors
//! - Speedup increases with vector size due to better cache utilization

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

/// Scalar implementation of cosine similarity
fn cosine_similarity_scalar(v1: &[f64], v2: &[f64]) -> f64 {
    if v1.len() != v2.len() || v1.is_empty() {
        return 0.0;
    }

    let mut dot_product = 0.0;
    let mut norm1 = 0.0;
    let mut norm2 = 0.0;

    for i in 0..v1.len() {
        dot_product += v1[i] * v2[i];
        norm1 += v1[i] * v1[i];
        norm2 += v2[i] * v2[i];
    }

    if norm1 > 0.0 && norm2 > 0.0 {
        dot_product / (norm1.sqrt() * norm2.sqrt())
    } else {
        0.0
    }
}

/// SIMD implementation using Trueno
#[cfg(feature = "simd")]
fn cosine_similarity_simd(v1: &[f64], v2: &[f64]) -> f64 {
    use trueno::Vector;

    if v1.len() != v2.len() || v1.is_empty() {
        return 0.0;
    }

    // Convert f64 to f32 for Trueno
    let v1_f32: Vec<f32> = v1.iter().map(|&x| x as f32).collect();
    let v2_f32: Vec<f32> = v2.iter().map(|&x| x as f32).collect();

    let vec1 = Vector::from_slice(&v1_f32);
    let vec2 = Vector::from_slice(&v2_f32);

    let dot = match vec1.dot(&vec2) {
        Ok(d) => d as f64,
        Err(_) => return 0.0,
    };

    let norm1 = match vec1.norm_l2() {
        Ok(n) => n as f64,
        Err(_) => return 0.0,
    };

    let norm2 = match vec2.norm_l2() {
        Ok(n) => n as f64,
        Err(_) => return 0.0,
    };

    if norm1 > 0.0 && norm2 > 0.0 {
        dot / (norm1 * norm2)
    } else {
        0.0
    }
}

/// Scalar implementation of entropy
fn entropy_scalar(probabilities: &[f64]) -> f64 {
    let mut entropy = 0.0;
    for &p in probabilities {
        if p > 0.0 {
            entropy -= p * p.log2();
        }
    }
    entropy
}

/// SIMD implementation of entropy using Trueno's log2()
#[cfg(feature = "simd")]
fn entropy_simd(probabilities: &[f64]) -> f64 {
    use trueno::Vector;

    if probabilities.is_empty() {
        return 0.0;
    }

    // Convert to f32, replacing zeros with 1.0 (so log2(1) = 0)
    let probs_f32: Vec<f32> = probabilities
        .iter()
        .map(|&p| if p > 0.0 { p as f32 } else { 1.0 })
        .collect();

    let probs_vec = Vector::from_slice(&probs_f32);

    // Compute log2(p) for all elements
    let log2_vec = match probs_vec.log2() {
        Ok(v) => v,
        Err(_) => return entropy_scalar(probabilities),
    };

    // Compute p * log2(p)
    let p_log_p = match probs_vec.mul(&log2_vec) {
        Ok(v) => v,
        Err(_) => return entropy_scalar(probabilities),
    };

    // Sum and negate: H = -Σ p * log2(p)
    match p_log_p.sum() {
        Ok(sum) => -(sum as f64),
        Err(_) => entropy_scalar(probabilities),
    }
}

/// Benchmark cosine similarity at different vector sizes
fn bench_cosine_similarity(c: &mut Criterion) {
    let sizes = [100, 500, 1000, 5000, 10000];

    let mut group = c.benchmark_group("cosine_similarity");

    for size in sizes {
        // Generate test vectors
        let v1: Vec<f64> = (0..size).map(|i| (i as f64).sin()).collect();
        let v2: Vec<f64> = (0..size).map(|i| (i as f64).cos()).collect();

        // Benchmark scalar
        group.bench_with_input(BenchmarkId::new("scalar", size), &size, |b, _| {
            b.iter(|| cosine_similarity_scalar(black_box(&v1), black_box(&v2)))
        });

        // Benchmark SIMD (only when feature is enabled)
        #[cfg(feature = "simd")]
        group.bench_with_input(BenchmarkId::new("simd", size), &size, |b, _| {
            b.iter(|| cosine_similarity_simd(black_box(&v1), black_box(&v2)))
        });
    }

    group.finish();
}

/// Benchmark to show f32 vs f64 overhead from Trueno's f32 requirement
#[cfg(feature = "simd")]
fn bench_conversion_overhead(c: &mut Criterion) {
    let sizes = [1000, 10000];

    let mut group = c.benchmark_group("conversion_overhead");

    for size in sizes {
        let v1: Vec<f64> = (0..size).map(|i| (i as f64).sin()).collect();

        // Benchmark f64 to f32 conversion
        group.bench_with_input(BenchmarkId::new("f64_to_f32", size), &size, |b, _| {
            b.iter(|| {
                let _: Vec<f32> = black_box(&v1).iter().map(|&x| x as f32).collect();
            })
        });
    }

    group.finish();
}

/// Benchmark for batch cosine similarity (multiple vector pairs)
fn bench_batch_cosine_similarity(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_cosine_similarity");

    let num_pairs = 100;
    let vector_size = 1000;

    // Generate test vector pairs
    let pairs: Vec<(Vec<f64>, Vec<f64>)> = (0..num_pairs)
        .map(|i| {
            let v1: Vec<f64> = (0..vector_size)
                .map(|j| ((i * vector_size + j) as f64).sin())
                .collect();
            let v2: Vec<f64> = (0..vector_size)
                .map(|j| ((i * vector_size + j) as f64).cos())
                .collect();
            (v1, v2)
        })
        .collect();

    // Benchmark scalar batch
    group.bench_function("scalar_batch_100x1000", |b| {
        b.iter(|| {
            let mut results = Vec::with_capacity(num_pairs);
            for (v1, v2) in &pairs {
                results.push(cosine_similarity_scalar(black_box(v1), black_box(v2)));
            }
            results
        })
    });

    // Benchmark SIMD batch
    #[cfg(feature = "simd")]
    group.bench_function("simd_batch_100x1000", |b| {
        b.iter(|| {
            let mut results = Vec::with_capacity(num_pairs);
            for (v1, v2) in &pairs {
                results.push(cosine_similarity_simd(black_box(v1), black_box(v2)));
            }
            results
        })
    });

    group.finish();
}

/// Benchmark for entropy calculation
fn bench_entropy(c: &mut Criterion) {
    let mut group = c.benchmark_group("entropy");

    let sizes = [100, 500, 1000, 5000, 10000];

    for size in sizes {
        // Generate valid probability distribution
        let raw: Vec<f64> = (0..size).map(|i| i as f64 + 1.0).collect();
        let sum: f64 = raw.iter().sum();
        let probs: Vec<f64> = raw.iter().map(|&x| x / sum).collect();

        // Benchmark scalar
        group.bench_with_input(BenchmarkId::new("scalar", size), &size, |b, _| {
            b.iter(|| entropy_scalar(black_box(&probs)))
        });

        // Benchmark SIMD (only when feature is enabled)
        #[cfg(feature = "simd")]
        group.bench_with_input(BenchmarkId::new("simd", size), &size, |b, _| {
            b.iter(|| entropy_simd(black_box(&probs)))
        });
    }

    group.finish();
}

/// Benchmark for batch entropy calculations
fn bench_batch_entropy(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_entropy");

    let num_batches = 100;
    let vector_size = 1000;

    // Generate test probability distributions
    let batches: Vec<Vec<f64>> = (0..num_batches)
        .map(|i| {
            let raw: Vec<f64> = (0..vector_size)
                .map(|j| ((i * vector_size + j) % 100 + 1) as f64)
                .collect();
            let sum: f64 = raw.iter().sum();
            raw.iter().map(|&x| x / sum).collect()
        })
        .collect();

    // Benchmark scalar batch
    group.bench_function("scalar_batch_100x1000", |b| {
        b.iter(|| {
            let mut results = Vec::with_capacity(num_batches);
            for probs in &batches {
                results.push(entropy_scalar(black_box(probs)));
            }
            results
        })
    });

    // Benchmark SIMD batch
    #[cfg(feature = "simd")]
    group.bench_function("simd_batch_100x1000", |b| {
        b.iter(|| {
            let mut results = Vec::with_capacity(num_batches);
            for probs in &batches {
                results.push(entropy_simd(black_box(probs)));
            }
            results
        })
    });

    group.finish();
}

#[cfg(feature = "simd")]
criterion_group!(
    benches,
    bench_cosine_similarity,
    bench_conversion_overhead,
    bench_batch_cosine_similarity,
    bench_entropy,
    bench_batch_entropy
);

#[cfg(not(feature = "simd"))]
criterion_group!(
    benches,
    bench_cosine_similarity,
    bench_batch_cosine_similarity,
    bench_entropy,
    bench_batch_entropy
);

criterion_main!(benches);
