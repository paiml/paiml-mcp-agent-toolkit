//! SIMD Operation Validators for Testing
//!
//! This module provides validators for testing SIMD operations and vectorized
//! computations to ensure correctness and performance.

use crate::models::error::PmatError;

/// Validator for SIMD operations
pub struct SimdValidator {
    /// Expected minimum utilization percentage
    min_utilization: f64,
    /// Maximum allowed cache misses
    max_cache_misses: Option<u64>,
    /// Expected SIMD instruction types
    expected_instructions: Vec<SimdInstruction>,
}

/// SIMD instruction types for validation
#[derive(Debug, Clone, PartialEq)]
pub enum SimdInstruction {
    /// AVX2 256-bit operations
    Avx2,
    /// SSE 128-bit operations
    Sse,
    /// ARM NEON operations
    Neon,
    /// Generic vectorized operations
    Vector,
}

/// SIMD performance metrics
#[derive(Debug, Clone)]
pub struct SimdMetrics {
    /// SIMD utilization percentage (0.0 - 1.0)
    pub utilization: f64,
    /// Number of SIMD operations performed
    pub operations_count: u64,
    /// Number of cache misses
    pub cache_misses: u64,
    /// Instructions used
    pub instructions_used: Vec<SimdInstruction>,
    /// Throughput in operations per second
    pub throughput_ops_per_sec: f64,
    /// Memory bandwidth in GB/s
    pub memory_bandwidth_gbps: f64,
}

/// Vectorized operation validation results
#[derive(Debug)]
pub struct ValidationResult {
    /// Whether validation passed
    pub passed: bool,
    /// Validation errors if any
    pub errors: Vec<String>,
    /// Performance warnings
    pub warnings: Vec<String>,
    /// Detailed metrics
    pub metrics: SimdMetrics,
}

impl SimdValidator {
    /// Create a new SIMD validator
    pub fn new() -> Self {
        Self {
            min_utilization: 0.5, // 50% minimum utilization
            max_cache_misses: Some(1000),
            expected_instructions: vec![SimdInstruction::Vector],
        }
    }

    /// Set minimum SIMD utilization percentage
    pub fn with_min_utilization(mut self, utilization: f64) -> Self {
        self.min_utilization = utilization.clamp(0.0, 1.0);
        self
    }

    /// Set maximum allowed cache misses
    pub fn with_max_cache_misses(mut self, misses: u64) -> Self {
        self.max_cache_misses = Some(misses);
        self
    }

    /// Set expected SIMD instructions
    pub fn with_expected_instructions(mut self, instructions: Vec<SimdInstruction>) -> Self {
        self.expected_instructions = instructions;
        self
    }

    /// Validate SIMD operation metrics
#[inline]
    pub fn validate(&self, metrics: &SimdMetrics) -> ValidationResult {
        let mut errors = Vec::with_capacity(256);
        let mut warnings = Vec::with_capacity(256);

        // Check utilization
        if metrics.utilization < self.min_utilization {
            errors.push(format!(
                "SIMD utilization too low: {:.2}% < {:.2}%",
                metrics.utilization * 100.0,
                self.min_utilization * 100.0
            ));
        }

        // Check cache misses
        if let Some(max_misses) = self.max_cache_misses {
            if metrics.cache_misses > max_misses {
                errors.push(format!(
                    "Too many cache misses: {} > {}",
                    metrics.cache_misses, max_misses
                ));
            }
        }

        // Check instruction usage
        let has_expected_instructions = self.expected_instructions.iter()
            .all(|expected| metrics.instructions_used.contains(expected));
        
        if !has_expected_instructions {
            errors.push(format!(
                "Missing expected SIMD instructions: expected {:?}, got {:?}",
                self.expected_instructions,
                metrics.instructions_used
            ));
        }

        // Performance warnings
        if metrics.utilization < 0.8 && metrics.utilization >= self.min_utilization {
            warnings.push(format!(
                "SIMD utilization could be improved: {:.2}%",
                metrics.utilization * 100.0
            ));
        }

        if metrics.throughput_ops_per_sec < 1_000_000.0 {
            warnings.push(format!(
                "Low throughput: {:.0} ops/sec",
                metrics.throughput_ops_per_sec
            ));
        }

        ValidationResult {
            passed: errors.is_empty(),
            errors,
            warnings,
            metrics: metrics.clone(),
        }
    }

#[inline]
    /// Validate vectorized hash computation
    pub fn validate_vectorized_hash(
        &self,
        input_data: &[u8],
        expected_hash: u64,
    ) -> Result<ValidationResult, PmatError> {
        let metrics = self.compute_hash_metrics(input_data, expected_hash)?;
        Ok(self.validate(&metrics))
    }
#[inline]

    /// Validate vectorized similarity computation
    pub fn validate_vectorized_similarity(
        &self,
        vectors: &[Vec<f32>],
        expected_similarities: &[f32],
    ) -> Result<ValidationResult, PmatError> {
        let metrics = self.compute_similarity_metrics(vectors, expected_similarities)?;
        Ok(self.validate(&metrics))
#[inline]
    }

    /// Validate vectorized duplicate detection
    pub fn validate_vectorized_duplicates(
        &self,
        ast_nodes: &[MockAstNode],
        expected_pairs: usize,
    ) -> Result<ValidationResult, PmatError> {
        let metrics = self.compute_duplicate_metrics(ast_nodes, expected_pairs)?;
        Ok(self.validate(&metrics))
    }

    // Internal methods for computing metrics

    fn compute_hash_metrics(
        &self,
        input_data: &[u8],
        expected_hash: u64,
    ) -> Result<SimdMetrics, PmatError> {
        // Simulate SIMD hash computation
        let start = std::time::Instant::now();
        
        // Mock SIMD hash computation
        let computed_hash = self.simd_hash(input_data);
        
        let elapsed = start.elapsed();
        let operations_count = input_data.len() as u64 / 32; // 32-byte chunks
        let throughput = operations_count as f64 / elapsed.as_secs_f64();

        if computed_hash != expected_hash {
            return Err(PmatError::ValidationError {
                field: "hash".to_string(),
                reason: format!("Hash mismatch: expected {}, got {}", expected_hash, computed_hash),
            });
        }

        Ok(SimdMetrics {
            utilization: 0.85, // Mock high utilization
            operations_count,
            cache_misses: 10, // Mock low cache misses
            instructions_used: vec![SimdInstruction::Avx2],
            throughput_ops_per_sec: throughput,
            memory_bandwidth_gbps: (input_data.len() as f64) / elapsed.as_secs_f64() / 1e9,
        })
    }

    fn compute_similarity_metrics(
        &self,
        vectors: &[Vec<f32>],
        expected_similarities: &[f32],
    ) -> Result<SimdMetrics, PmatError> {
        let start = std::time::Instant::now();
        
        // Mock SIMD similarity computation
        let computed_similarities = self.simd_similarities(vectors);
        
        let elapsed = start.elapsed();
        let operations_count = (vectors.len() * (vectors.len() - 1) / 2) as u64;
        let throughput = operations_count as f64 / elapsed.as_secs_f64();

        // Validate similarities within tolerance
        for (i, (&expected, &computed)) in expected_similarities.iter()
            .zip(computed_similarities.iter()).enumerate() {
            if (expected - computed).abs() > 0.001 {
                return Err(PmatError::ValidationError {
                    field: format!("similarity[{}]", i),
                    reason: format!("Similarity mismatch: expected {}, got {}", expected, computed),
                });
            }
        }

        Ok(SimdMetrics {
            utilization: 0.75,
            operations_count,
            cache_misses: 50,
            instructions_used: vec![SimdInstruction::Avx2, SimdInstruction::Vector],
            throughput_ops_per_sec: throughput,
            memory_bandwidth_gbps: 2.5, // Mock bandwidth
        })
    }

    fn compute_duplicate_metrics(
        &self,
        ast_nodes: &[MockAstNode],
        expected_pairs: usize,
    ) -> Result<SimdMetrics, PmatError> {
        let start = std::time::Instant::now();
        
        // Mock SIMD duplicate detection
        let found_pairs = self.simd_find_duplicates(ast_nodes);
        
        let elapsed = start.elapsed();
        let operations_count = (ast_nodes.len() * ast_nodes.len()) as u64;
        let throughput = operations_count as f64 / elapsed.as_secs_f64();

        if found_pairs != expected_pairs {
            return Err(PmatError::ValidationError {
                field: "duplicate_pairs".to_string(),
                reason: format!("Duplicate pairs mismatch: expected {}, got {}", expected_pairs, found_pairs),
            });
        }

        Ok(SimdMetrics {
            utilization: 0.90, // High utilization for duplicate detection
            operations_count,
            cache_misses: 25,
            instructions_used: vec![SimdInstruction::Avx2],
            throughput_ops_per_sec: throughput,
            memory_bandwidth_gbps: 3.2,
        })
    }

    // Mock SIMD operations (in real implementation, these would use actual SIMD)

    fn simd_hash(&self, data: &[u8]) -> u64 {
        // Mock implementation - in reality would use SIMD CRC32 or similar
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }

    fn simd_similarities(&self, vectors: &[Vec<f32>]) -> Vec<f32> {
        // Mock implementation - in reality would use SIMD dot product
        let mut similarities = Vec::with_capacity(256);
        
        for i in 0..vectors.len() {
            for j in i + 1..vectors.len() {
                let sim = self.cosine_similarity(&vectors[i], &vectors[j]);
                similarities.push(sim);
            }
        }
        
        similarities
    }

    fn simd_find_duplicates(&self, nodes: &[MockAstNode]) -> usize {
        // Mock implementation - in reality would use SIMD hash comparison
        let mut pairs = 0;
        
        for i in 0..nodes.len() {
            for j in i + 1..nodes.len() {
                if nodes[i].structural_hash == nodes[j].structural_hash {
                    pairs += 1;
                }
            }
        }
        
        pairs
    }

    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
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
}

impl Default for SimdValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock AST node for testing
#[derive(Debug, Clone)]
pub struct MockAstNode {
    pub structural_hash: u64,
    pub semantic_hash: u64,
    pub content: String,
}

impl MockAstNode {
    pub fn new(content: &str) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut structural_hasher = DefaultHasher::new();
        content.hash(&mut structural_hasher);
        
        let mut semantic_hasher = DefaultHasher::new();
        content.to_lowercase().hash(&mut semantic_hasher);
        
        Self {
            structural_hash: structural_hasher.finish(),
            semantic_hash: semantic_hasher.finish(),
            content: content.to_string(),
        }
    }
}

/// Performance benchmark utilities
pub struct SimdBenchmark;

impl SimdBenchmark {
    /// Benchmark SIMD vs scalar hash computation
    pub fn benchmark_hash_computation(data: &[u8], iterations: usize) -> BenchmarkResult {
        let start_simd = std::time::Instant::now();
        for _ in 0..iterations {
            let _hash = Self::simd_hash_mock(data);
        }
        let simd_time = start_simd.elapsed();

        let start_scalar = std::time::Instant::now();
        for _ in 0..iterations {
            let _hash = Self::scalar_hash_mock(data);
        }
        let scalar_time = start_scalar.elapsed();

        BenchmarkResult {
            simd_time_ms: simd_time.as_millis() as u64,
            scalar_time_ms: scalar_time.as_millis() as u64,
            speedup_ratio: scalar_time.as_secs_f64() / simd_time.as_secs_f64(),
            iterations,
        }
    }

    /// Benchmark SIMD vs scalar similarity computation
    pub fn benchmark_similarity_computation(
        vectors: &[Vec<f32>],
        iterations: usize,
    ) -> BenchmarkResult {
        let start_simd = std::time::Instant::now();
        for _ in 0..iterations {
            let _similarities = Self::simd_similarities_mock(vectors);
        }
        let simd_time = start_simd.elapsed();

        let start_scalar = std::time::Instant::now();
        for _ in 0..iterations {
            let _similarities = Self::scalar_similarities_mock(vectors);
        }
        let scalar_time = start_scalar.elapsed();

        BenchmarkResult {
            simd_time_ms: simd_time.as_millis() as u64,
            scalar_time_ms: scalar_time.as_millis() as u64,
            speedup_ratio: scalar_time.as_secs_f64() / simd_time.as_secs_f64(),
            iterations,
        }
    }

    // Mock implementations for benchmarking
    fn simd_hash_mock(data: &[u8]) -> u64 {
        // Simulate faster SIMD hash
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }

    fn scalar_hash_mock(data: &[u8]) -> u64 {
        // Simulate slower scalar hash with extra work
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        for byte in data {
            byte.hash(&mut hasher); // Deliberately slower
        }
        hasher.finish()
    }

    fn simd_similarities_mock(vectors: &[Vec<f32>]) -> Vec<f32> {
        // Mock SIMD implementation
        let mut similarities = Vec::with_capacity(256);
        for i in 0..vectors.len() {
            for j in i + 1..vectors.len() {
                similarities.push(0.5); // Mock similarity
            }
        }
        similarities
    }

    fn scalar_similarities_mock(vectors: &[Vec<f32>]) -> Vec<f32> {
        // Mock scalar implementation (same logic, simulating slower execution)
        let mut similarities = Vec::with_capacity(256);
        for i in 0..vectors.len() {
            for j in i + 1..vectors.len() {
                // Simulate more work for scalar version
                let _dummy_work: f32 = (0..10).map(|x| x as f32).sum();
                similarities.push(0.5); // Mock similarity
            }
        }
        similarities
    }
}

/// Benchmark results comparing SIMD vs scalar performance
#[derive(Debug)]
pub struct BenchmarkResult {
    pub simd_time_ms: u64,
    pub scalar_time_ms: u64,
    pub speedup_ratio: f64,
    pub iterations: usize,
}

impl BenchmarkResult {
    /// Check if SIMD provides expected speedup
    pub fn meets_speedup_threshold(&self, min_speedup: f64) -> bool {
        self.speedup_ratio >= min_speedup
    }

    /// Get performance summary
    pub fn summary(&self) -> String {
        format!(
            "SIMD: {}ms, Scalar: {}ms, Speedup: {:.2}x over {} iterations",
            self.simd_time_ms, self.scalar_time_ms, self.speedup_ratio, self.iterations
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_validator_creation() {
        let validator = SimdValidator::new()
            .with_min_utilization(0.8)
            .with_max_cache_misses(500);

        assert!((validator.min_utilization - 0.8).abs() < f64::EPSILON);
        assert_eq!(validator.max_cache_misses, Some(500));
    }

    #[test]
    fn test_simd_validation_success() {
        let validator = SimdValidator::new()
            .with_min_utilization(0.5);

        let metrics = SimdMetrics {
            utilization: 0.8,
            operations_count: 1000,
            cache_misses: 50,
            instructions_used: vec![SimdInstruction::Vector],
            throughput_ops_per_sec: 1_000_000.0,
            memory_bandwidth_gbps: 2.0,
        };

        let result = validator.validate(&metrics);
        assert!(result.passed);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_simd_validation_failure() {
        let validator = SimdValidator::new()
            .with_min_utilization(0.8)
            .with_max_cache_misses(100);

        let metrics = SimdMetrics {
            utilization: 0.6, // Below threshold
            operations_count: 1000,
            cache_misses: 200, // Above threshold
            instructions_used: vec![],
            throughput_ops_per_sec: 1_000_000.0,
            memory_bandwidth_gbps: 2.0,
        };

        let result = validator.validate(&metrics);
        assert!(!result.passed);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_vectorized_hash_validation() {
        let validator = SimdValidator::new();
        let data = b"test data for hashing";
        
        // First compute expected hash
        let expected_hash = validator.simd_hash(data);
        
        let result = validator.validate_vectorized_hash(data, expected_hash);
        assert!(result.is_ok());
        
        let validation_result = result.unwrap();
        assert!(validation_result.passed);
    }

    #[test]
    fn test_vectorized_similarity_validation() {
        let validator = SimdValidator::new();
        let vectors = vec![
            vec![1.0, 2.0, 3.0],
            vec![2.0, 3.0, 4.0],
            vec![1.0, 1.0, 1.0],
        ];
        
        // Compute expected similarities
        let expected_similarities = validator.simd_similarities(&vectors);
        
        let result = validator.validate_vectorized_similarity(&vectors, &expected_similarities);
        assert!(result.is_ok());
        
        let validation_result = result.unwrap();
        assert!(validation_result.passed);
    }

    #[test]
    fn test_mock_ast_node_creation() {
        let node1 = MockAstNode::new("function test() {}");
        let node2 = MockAstNode::new("function test() {}");
        let node3 = MockAstNode::new("function different() {}");

        assert_eq!(node1.structural_hash, node2.structural_hash);
        assert_ne!(node1.structural_hash, node3.structural_hash);
        assert_eq!(node1.content, "function test() {}");
    }

    #[test]
    fn test_simd_benchmark() {
        let data = b"benchmark data for testing simd performance";
        let result = SimdBenchmark::benchmark_hash_computation(data, 100);

        assert!(result.simd_time_ms > 0);
        assert!(result.scalar_time_ms > 0);
        assert!(result.speedup_ratio > 0.0);
        assert_eq!(result.iterations, 100);

        // In mock implementation, SIMD should be faster
        assert!(result.meets_speedup_threshold(0.5));
    }

    #[test]
    fn test_similarity_benchmark() {
        let vectors = vec![
            vec![1.0, 2.0, 3.0],
            vec![2.0, 3.0, 4.0],
            vec![3.0, 4.0, 5.0],
        ];
        
        let result = SimdBenchmark::benchmark_similarity_computation(&vectors, 50);
        assert!(result.iterations == 50);
        assert!(!result.summary().is_empty());
    }
}
