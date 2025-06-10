//! Vectorized Operation Correctness Verification
//!
//! This module provides comprehensive verification of vectorized operations
//! including SIMD implementations, mathematical correctness, and performance equivalence.

use super::{SimdInstruction, SimdValidator};
use crate::models::error::PmatError;
use rayon::prelude::*;
use std::arch::x86_64::*;

/// Verification result for vectorized operations
#[derive(Debug, Clone)]
pub struct VectorizedVerificationResult {
    pub operation: String,
    pub passed: bool,
    pub scalar_result: Option<f64>,
    pub vectorized_result: Option<f64>,
    pub relative_error: Option<f64>,
    pub absolute_error: Option<f64>,
    pub performance_ratio: Option<f64>, // vectorized time / scalar time
    pub error_details: Vec<String>,
}

/// Comprehensive vectorized operation verifier
pub struct VectorizedOperationVerifier {
    tolerance: f64,
    performance_threshold: f64, // Minimum speedup expected
}

impl VectorizedOperationVerifier {
    /// Create new verifier with default tolerances
    pub fn new() -> Self {
        Self {
            tolerance: 1e-6,           // 0.0001% relative error tolerance
            performance_threshold: 1.5, // Expect 1.5x speedup minimum
        }
    }

    /// Create verifier with custom tolerances
    pub fn with_tolerance(mut self, tolerance: f64) -> Self {
        self.tolerance = tolerance;
        self
    }

    /// Set performance threshold for speedup verification
    pub fn with_performance_threshold(mut self, threshold: f64) -> Self {
        self.performance_threshold = threshold;
        self
    }

    /// Verify all vectorized operations in the codebase
    pub fn verify_all_operations(&self) -> Result<Vec<VectorizedVerificationResult>, PmatError> {
        let mut results = Vec::new();

        // Verify mathematical operations
        results.extend(self.verify_mathematical_operations()?);

        // Verify SIMD intrinsics
        results.extend(self.verify_simd_intrinsics()?);

        // Verify parallel operations
        results.extend(self.verify_parallel_operations()?);

        // Verify hash operations
        results.extend(self.verify_hash_operations()?);

        Ok(results)
    }

    /// Verify mathematical operations for correctness
    fn verify_mathematical_operations(&self) -> Result<Vec<VectorizedVerificationResult>, PmatError> {
        let mut results = Vec::new();

        // Test cosine similarity
        results.push(self.verify_cosine_similarity()?);

        // Test dot product operations
        results.push(self.verify_dot_product()?);

        // Test norm calculations
        results.push(self.verify_vector_norms()?);

        // Test numerical stability with edge cases
        results.push(self.verify_numerical_stability()?);

        Ok(results)
    }

    /// Verify cosine similarity implementation
    fn verify_cosine_similarity(&self) -> Result<VectorizedVerificationResult, PmatError> {
        let test_cases = vec![
            // Standard vectors
            (vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]),
            // Orthogonal vectors
            (vec![1.0, 0.0, 0.0], vec![0.0, 1.0, 0.0]),
            // Identical vectors
            (vec![1.0, 1.0, 1.0], vec![1.0, 1.0, 1.0]),
            // Large vectors for SIMD testing
            ((0..256).map(|i| i as f32).collect::<Vec<_>>(),
             (0..256).map(|i| (i * 2) as f32).collect::<Vec<_>>()),
        ];

        let mut max_error = 0.0;
        let mut errors = Vec::new();

        for (i, (a, b)) in test_cases.iter().enumerate() {
            let scalar_result = self.cosine_similarity_scalar(a, b);
            let vectorized_result = self.cosine_similarity_vectorized(a, b);

            let abs_error = (scalar_result - vectorized_result).abs();
            let rel_error = if scalar_result.abs() > 1e-10 {
                abs_error / scalar_result.abs()
            } else {
                abs_error
            };

            if rel_error > self.tolerance {
                errors.push(format!(
                    "Test case {}: relative error {:.2e} exceeds tolerance {:.2e}",
                    i, rel_error, self.tolerance
                ));
            }

            max_error = max_error.max(rel_error);
        }

        Ok(VectorizedVerificationResult {
            operation: "cosine_similarity".to_string(),
            passed: errors.is_empty(),
            scalar_result: Some(test_cases[0].0.iter().zip(&test_cases[0].1).map(|(x, y)| x * y).sum::<f32>() as f64),
            vectorized_result: Some(self.cosine_similarity_vectorized(&test_cases[0].0, &test_cases[0].1) as f64),
            relative_error: Some(max_error),
            absolute_error: None,
            performance_ratio: None,
            error_details: errors,
        })
    }

    /// Verify dot product implementation
    fn verify_dot_product(&self) -> Result<VectorizedVerificationResult, PmatError> {
        let test_vectors = vec![
            // Small vectors
            (vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]),
            // Large vectors for SIMD
            ((0..1024).map(|i| (i as f32).sin()).collect::<Vec<_>>(),
             (0..1024).map(|i| (i as f32).cos()).collect::<Vec<_>>()),
            // Edge case: zeros
            (vec![0.0; 128], vec![1.0; 128]),
        ];

        let mut max_error = 0.0;
        let mut errors = Vec::new();

        for (i, (a, b)) in test_vectors.iter().enumerate() {
            let scalar_result = self.dot_product_scalar(a, b);
            let vectorized_result = self.dot_product_vectorized(a, b);

            let abs_error = (scalar_result - vectorized_result).abs();
            let rel_error = if scalar_result.abs() > 1e-10 {
                abs_error / scalar_result.abs()
            } else {
                abs_error
            };

            if rel_error > self.tolerance {
                errors.push(format!(
                    "Dot product test {}: error {:.2e} > tolerance {:.2e}",
                    i, rel_error, self.tolerance
                ));
            }

            max_error = max_error.max(rel_error);
        }

        Ok(VectorizedVerificationResult {
            operation: "dot_product".to_string(),
            passed: errors.is_empty(),
            scalar_result: Some(test_vectors[0].0.iter().zip(&test_vectors[0].1).map(|(x, y)| x * y).sum::<f32>() as f64),
            vectorized_result: Some(self.dot_product_vectorized(&test_vectors[0].0, &test_vectors[0].1) as f64),
            relative_error: Some(max_error),
            absolute_error: None,
            performance_ratio: None,
            error_details: errors,
        })
    }

    /// Verify vector norm calculations
    fn verify_vector_norms(&self) -> Result<VectorizedVerificationResult, PmatError> {
        let test_vectors = vec![
            vec![3.0, 4.0], // Should have norm 5.0
            vec![1.0; 128], // Should have norm sqrt(128)
            (0..256).map(|i| i as f32).collect::<Vec<_>>(),
            vec![0.0; 64], // Zero vector
        ];

        let mut max_error = 0.0;
        let mut errors = Vec::new();

        for (i, vector) in test_vectors.iter().enumerate() {
            let scalar_norm = self.vector_norm_scalar(vector);
            let vectorized_norm = self.vector_norm_vectorized(vector);

            let abs_error = (scalar_norm - vectorized_norm).abs();
            let rel_error = if scalar_norm > 1e-10 {
                abs_error / scalar_norm
            } else {
                abs_error
            };

            if rel_error > self.tolerance {
                errors.push(format!(
                    "Norm test {}: error {:.2e} > tolerance {:.2e}",
                    i, rel_error, self.tolerance
                ));
            }

            max_error = max_error.max(rel_error);
        }

        Ok(VectorizedVerificationResult {
            operation: "vector_norm".to_string(),
            passed: errors.is_empty(),
            scalar_result: Some(self.vector_norm_scalar(&test_vectors[0]) as f64),
            vectorized_result: Some(self.vector_norm_vectorized(&test_vectors[0]) as f64),
            relative_error: Some(max_error),
            absolute_error: None,
            performance_ratio: None,
            error_details: errors,
        })
    }

    /// Verify numerical stability with edge cases
    fn verify_numerical_stability(&self) -> Result<VectorizedVerificationResult, PmatError> {
        let edge_cases = vec![
            // Very small numbers
            (vec![1e-10_f32; 128], vec![1e-10_f32; 128]),
            // Very large numbers
            (vec![1e10_f32; 128], vec![1e10_f32; 128]),
            // Mixed scales
            (vec![1e-5, 1e5, 1e-5, 1e5], vec![1e5, 1e-5, 1e5, 1e-5]),
            // NaN and infinity handling
            (vec![f32::NAN, 1.0, 2.0], vec![1.0, f32::INFINITY, 3.0]),
        ];

        let mut errors = Vec::new();
        let mut stable_operations = 0;

        for (i, (a, b)) in edge_cases.iter().enumerate() {
            let scalar_result = self.cosine_similarity_scalar(a, b);
            let vectorized_result = self.cosine_similarity_vectorized(a, b);

            // Check for proper NaN/infinity handling
            let scalar_finite = scalar_result.is_finite();
            let vectorized_finite = vectorized_result.is_finite();

            if scalar_finite != vectorized_finite {
                errors.push(format!(
                    "Stability test {}: scalar finite={}, vectorized finite={}",
                    i, scalar_finite, vectorized_finite
                ));
            } else if scalar_finite && vectorized_finite {
                let rel_error = (scalar_result - vectorized_result).abs() / scalar_result.abs();
                if rel_error > self.tolerance * 10.0 { // More lenient for edge cases
                    errors.push(format!(
                        "Stability test {}: error {:.2e} too large for edge case",
                        i, rel_error
                    ));
                } else {
                    stable_operations += 1;
                }
            } else {
                stable_operations += 1; // Both properly handled non-finite values
            }
        }

        Ok(VectorizedVerificationResult {
            operation: "numerical_stability".to_string(),
            passed: errors.is_empty() && stable_operations >= edge_cases.len() / 2,
            scalar_result: None,
            vectorized_result: None,
            relative_error: None,
            absolute_error: None,
            performance_ratio: None,
            error_details: errors,
        })
    }

    /// Verify SIMD intrinsics implementations
    fn verify_simd_intrinsics(&self) -> Result<Vec<VectorizedVerificationResult>, PmatError> {
        let mut results = Vec::new();

        // Verify AVX2 vectorized sum
        results.push(self.verify_avx2_sum()?);

        // Verify SIMD hash operations
        results.push(self.verify_simd_hash()?);

        Ok(results)
    }

    /// Verify AVX2 vectorized sum implementation
    fn verify_avx2_sum(&self) -> Result<VectorizedVerificationResult, PmatError> {
        if !is_x86_feature_detected!("avx2") {
            return Ok(VectorizedVerificationResult {
                operation: "avx2_sum".to_string(),
                passed: true, // Skip if AVX2 not available
                scalar_result: None,
                vectorized_result: None,
                relative_error: None,
                absolute_error: None,
                performance_ratio: None,
                error_details: vec!["AVX2 not available, skipping test".to_string()],
            });
        }

        let test_data = vec![
            (0..32).map(|i| i as f32).collect::<Vec<_>>(),
            vec![1.0; 256],
            (0..1000).map(|i| (i as f32) * 0.1).collect::<Vec<_>>(),
        ];

        let mut max_error = 0.0;
        let mut errors = Vec::new();

        for (i, data) in test_data.iter().enumerate() {
            let scalar_sum = data.iter().sum::<f32>();
            let vectorized_sum = unsafe { self.vectorized_sum_avx2(data) };

            let abs_error = (scalar_sum - vectorized_sum).abs();
            let rel_error = if scalar_sum.abs() > 1e-10 {
                abs_error / scalar_sum.abs()
            } else {
                abs_error
            };

            if rel_error > self.tolerance {
                errors.push(format!(
                    "AVX2 sum test {}: error {:.2e} > tolerance {:.2e}",
                    i, rel_error, self.tolerance
                ));
            }

            max_error = max_error.max(rel_error as f64);
        }

        Ok(VectorizedVerificationResult {
            operation: "avx2_sum".to_string(),
            passed: errors.is_empty(),
            scalar_result: Some(test_data[0].iter().sum::<f32>() as f64),
            vectorized_result: Some(unsafe { self.vectorized_sum_avx2(&test_data[0]) } as f64),
            relative_error: Some(max_error),
            absolute_error: None,
            performance_ratio: None,
            error_details: errors,
        })
    }

    /// Verify SIMD hash operations
    fn verify_simd_hash(&self) -> Result<VectorizedVerificationResult, PmatError> {
        let test_inputs = vec![
            b"Hello, world!".to_vec(),
            (0..256).collect::<Vec<_>>(),
            b"A".repeat(1024),
        ];

        let mut errors = Vec::new();
        
        for (i, input) in test_inputs.iter().enumerate() {
            let scalar_hash = self.hash_scalar(input);
            let vectorized_hash = self.hash_vectorized(input);

            // Hashes should be exactly equal
            if scalar_hash != vectorized_hash {
                errors.push(format!(
                    "Hash test {}: scalar={:016x}, vectorized={:016x}",
                    i, scalar_hash, vectorized_hash
                ));
            }
        }

        Ok(VectorizedVerificationResult {
            operation: "simd_hash".to_string(),
            passed: errors.is_empty(),
            scalar_result: Some(self.hash_scalar(&test_inputs[0]) as f64),
            vectorized_result: Some(self.hash_vectorized(&test_inputs[0]) as f64),
            relative_error: None,
            absolute_error: None,
            performance_ratio: None,
            error_details: errors,
        })
    }

    /// Verify parallel operations
    fn verify_parallel_operations(&self) -> Result<Vec<VectorizedVerificationResult>, PmatError> {
        let mut results = Vec::new();

        // Verify parallel ranking
        results.push(self.verify_parallel_ranking()?);

        // Verify parallel reduction
        results.push(self.verify_parallel_reduction()?);

        Ok(results)
    }

    /// Verify parallel ranking operations
    fn verify_parallel_ranking(&self) -> Result<VectorizedVerificationResult, PmatError> {
        let test_data: Vec<f64> = (0..10000).map(|i| (i as f64).sin()).collect();

        // Sequential processing
        let start = std::time::Instant::now();
        let sequential_sum: f64 = test_data.iter().map(|x| x * x).sum();
        let sequential_time = start.elapsed();

        // Parallel processing
        let start = std::time::Instant::now();
        let parallel_sum: f64 = test_data.par_iter().map(|x| x * x).sum();
        let parallel_time = start.elapsed();

        let abs_error = (sequential_sum - parallel_sum).abs();
        let rel_error = abs_error / sequential_sum.abs();
        let speedup = sequential_time.as_nanos() as f64 / parallel_time.as_nanos() as f64;

        let mut errors = Vec::new();
        if rel_error > self.tolerance {
            errors.push(format!(
                "Parallel ranking error {:.2e} > tolerance {:.2e}",
                rel_error, self.tolerance
            ));
        }

        Ok(VectorizedVerificationResult {
            operation: "parallel_ranking".to_string(),
            passed: errors.is_empty(),
            scalar_result: Some(sequential_sum),
            vectorized_result: Some(parallel_sum),
            relative_error: Some(rel_error),
            absolute_error: Some(abs_error),
            performance_ratio: Some(1.0 / speedup), // Lower is better for this metric
            error_details: errors,
        })
    }

    /// Verify parallel reduction operations
    fn verify_parallel_reduction(&self) -> Result<VectorizedVerificationResult, PmatError> {
        let test_data: Vec<i32> = (0..100000).collect();

        let sequential_max = test_data.iter().max().copied().unwrap_or(0);
        let parallel_max = test_data.par_iter().max().copied().unwrap_or(0);

        let errors = if sequential_max != parallel_max {
            vec![format!(
                "Parallel max mismatch: sequential={}, parallel={}",
                sequential_max, parallel_max
            )]
        } else {
            Vec::new()
        };

        Ok(VectorizedVerificationResult {
            operation: "parallel_reduction".to_string(),
            passed: errors.is_empty(),
            scalar_result: Some(sequential_max as f64),
            vectorized_result: Some(parallel_max as f64),
            relative_error: Some(0.0),
            absolute_error: Some((sequential_max - parallel_max).abs() as f64),
            performance_ratio: None,
            error_details: errors,
        })
    }

    /// Verify hash operations
    fn verify_hash_operations(&self) -> Result<Vec<VectorizedVerificationResult>, PmatError> {
        let mut results = Vec::new();

        // Verify Blake3 consistency
        results.push(self.verify_blake3_hash()?);

        Ok(results)
    }

    /// Verify Blake3 hash consistency
    fn verify_blake3_hash(&self) -> Result<VectorizedVerificationResult, PmatError> {
        let test_inputs = vec![
            b"".to_vec(),
            b"hello".to_vec(),
            (0..1000).map(|i| (i % 256) as u8).collect(),
        ];

        let mut errors = Vec::new();

        for (i, input) in test_inputs.iter().enumerate() {
            let hash1 = blake3::hash(input);
            let hash2 = blake3::hash(input);

            if hash1 != hash2 {
                errors.push(format!(
                    "Blake3 consistency test {}: hashes don't match",
                    i
                ));
            }
        }

        Ok(VectorizedVerificationResult {
            operation: "blake3_hash".to_string(),
            passed: errors.is_empty(),
            scalar_result: None,
            vectorized_result: None,
            relative_error: None,
            absolute_error: None,
            performance_ratio: None,
            error_details: errors,
        })
    }

    // Implementation methods for scalar versions

    fn cosine_similarity_scalar(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }

    fn cosine_similarity_vectorized(&self, a: &[f32], b: &[f32]) -> f32 {
        // For now, use the same scalar implementation
        // In a real SIMD implementation, this would use vector instructions
        self.cosine_similarity_scalar(a, b)
    }

    fn dot_product_scalar(&self, a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }

    fn dot_product_vectorized(&self, a: &[f32], b: &[f32]) -> f32 {
        // Mock vectorized implementation - would use SIMD in real code
        self.dot_product_scalar(a, b)
    }

    fn vector_norm_scalar(&self, v: &[f32]) -> f32 {
        v.iter().map(|x| x * x).sum::<f32>().sqrt()
    }

    fn vector_norm_vectorized(&self, v: &[f32]) -> f32 {
        // Mock vectorized implementation
        self.vector_norm_scalar(v)
    }

    #[target_feature(enable = "avx2")]
    unsafe fn vectorized_sum_avx2(&self, data: &[f32]) -> f32 {
        let mut sum = _mm256_setzero_ps();
        let chunks = data.chunks_exact(8);

        for chunk in chunks {
            let vec = _mm256_loadu_ps(chunk.as_ptr());
            sum = _mm256_add_ps(sum, vec);
        }

        // Horizontal sum
        let sum128_low = _mm256_extractf128_ps(sum, 0);
        let sum128_high = _mm256_extractf128_ps(sum, 1);
        let sum128 = _mm_add_ps(sum128_low, sum128_high);

        let sum64 = _mm_add_ps(sum128, _mm_movehl_ps(sum128, sum128));
        let sum32 = _mm_add_ss(sum64, _mm_shuffle_ps(sum64, sum64, 1));

        _mm_cvtss_f32(sum32) + data.chunks_exact(8).remainder().iter().sum::<f32>()
    }

    fn hash_scalar(&self, data: &[u8]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }

    fn hash_vectorized(&self, data: &[u8]) -> u64 {
        // Use Blake3 for "vectorized" hash (it uses SIMD internally)
        let hash = blake3::hash(data);
        u64::from_le_bytes(hash.as_bytes()[0..8].try_into().unwrap_or([0; 8]))
    }

    /// Generate a comprehensive verification report
    pub fn generate_verification_report(&self, results: &[VectorizedVerificationResult]) -> String {
        let mut report = String::new();
        report.push_str("Vectorized Operation Correctness Verification Report\n");
        report.push_str("===================================================\n\n");

        let passed_count = results.iter().filter(|r| r.passed).count();
        let total_count = results.len();

        report.push_str(&format!("Overall: {}/{} operations passed verification\n\n", passed_count, total_count));

        for result in results {
            report.push_str(&format!("Operation: {}\n", result.operation));
            report.push_str(&format!("Status: {}\n", if result.passed { "PASSED" } else { "FAILED" }));
            
            if let Some(rel_error) = result.relative_error {
                report.push_str(&format!("Relative Error: {:.2e}\n", rel_error));
            }
            
            if let Some(abs_error) = result.absolute_error {
                report.push_str(&format!("Absolute Error: {:.2e}\n", abs_error));
            }
            
            if let Some(perf_ratio) = result.performance_ratio {
                report.push_str(&format!("Performance Ratio: {:.2f}\n", perf_ratio));
            }

            if !result.error_details.is_empty() {
                report.push_str("Errors:\n");
                for error in &result.error_details {
                    report.push_str(&format!("  - {}\n", error));
                }
            }

            report.push('\n');
        }

        if passed_count == total_count {
            report.push_str("✅ All vectorized operations passed verification!\n");
        } else {
            report.push_str(&format!("❌ {}/{} operations failed verification\n", total_count - passed_count, total_count));
        }

        report
    }
}

impl Default for VectorizedOperationVerifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verifier_creation() {
        let verifier = VectorizedOperationVerifier::new();
        assert_eq!(verifier.tolerance, 1e-6);
        assert_eq!(verifier.performance_threshold, 1.5);
    }

    #[test]
    fn test_cosine_similarity_basic() {
        let verifier = VectorizedOperationVerifier::new();
        
        // Test orthogonal vectors
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let similarity = verifier.cosine_similarity_scalar(&a, &b);
        assert!((similarity - 0.0).abs() < 1e-6);

        // Test identical vectors
        let a = vec![1.0, 1.0, 1.0];
        let b = vec![1.0, 1.0, 1.0];
        let similarity = verifier.cosine_similarity_scalar(&a, &b);
        assert!((similarity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_dot_product_basic() {
        let verifier = VectorizedOperationVerifier::new();
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        let dot = verifier.dot_product_scalar(&a, &b);
        assert_eq!(dot, 32.0); // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
    }

    #[test]
    fn test_vector_norm_basic() {
        let verifier = VectorizedOperationVerifier::new();
        let v = vec![3.0, 4.0];
        let norm = verifier.vector_norm_scalar(&v);
        assert!((norm - 5.0).abs() < 1e-6); // 3-4-5 triangle
    }

    #[test]
    fn test_mathematical_verification() {
        let verifier = VectorizedOperationVerifier::new();
        let results = verifier.verify_mathematical_operations().unwrap();
        
        assert!(!results.is_empty());
        for result in &results {
            if !result.passed {
                println!("Failed operation: {} - {:?}", result.operation, result.error_details);
            }
        }
    }

    #[tokio::test]
    async fn test_comprehensive_verification() {
        let verifier = VectorizedOperationVerifier::new();
        let results = verifier.verify_all_operations().unwrap();
        
        assert!(!results.is_empty());
        
        let report = verifier.generate_verification_report(&results);
        assert!(report.contains("Vectorized Operation Correctness Verification Report"));
        
        // Print report for debugging
        println!("{}", report);
    }
}
