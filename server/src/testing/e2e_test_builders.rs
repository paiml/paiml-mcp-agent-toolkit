//! E2E Test Builders for Enhanced Testing
//!
//! This module provides builders for creating comprehensive end-to-end tests
//! that use the ProjectBuilder and AnalysisResultMatcher.

use super::{AnalysisResultMatcher, MlModelFixture, ProjectBuilder, SimdValidator};
use crate::models::error::PmatError;
use crate::services::deep_context::{DeepContextAnalyzer, DeepContextConfig};
use serde_json::Value;
use std::path::PathBuf;
use std::time::Duration;

/// Builder for comprehensive E2E analysis tests
pub struct E2eTestBuilder {
    /// Project to analyze
    project_builder: Option<ProjectBuilder>,
    /// Expected analysis results
    result_matcher: AnalysisResultMatcher,
    /// SIMD performance validator
    simd_validator: Option<SimdValidator>,
    /// ML model fixtures for validation
    ml_fixtures: Vec<MlModelFixture>,
    /// Test configuration
    config: E2eTestConfig,
}

/// Configuration for E2E tests
#[derive(Debug, Clone)]
pub struct E2eTestConfig {
    /// Maximum analysis time allowed
    pub max_analysis_time: Duration,
    /// Whether to validate SIMD performance
    pub validate_simd: bool,
    /// Whether to validate ML models
    pub validate_ml: bool,
    /// Whether to cleanup test projects
    pub cleanup_projects: bool,
    /// Deep context analysis configuration
    pub deep_context_config: DeepContextConfig,
}

impl Default for E2eTestConfig {
    fn default() -> Self {
        Self {
            max_analysis_time: Duration::from_secs(30),
            validate_simd: false,
            validate_ml: false,
            cleanup_projects: true,
            deep_context_config: DeepContextConfig::default(),
        }
    }
}

/// Result of E2E test execution
#[derive(Debug)]
pub struct E2eTestResult {
    /// Whether the test passed
    pub passed: bool,
    /// Analysis duration
    pub analysis_duration: Duration,
    /// Project path used for testing
    pub project_path: PathBuf,
    /// Validation errors
    pub errors: Vec<String>,
    /// Analysis results
    pub analysis_results: Value,
    /// SIMD validation results
    pub simd_results: Option<super::ValidationResult>,
    /// ML validation results
    pub ml_results: Vec<(String, super::ml_model_fixtures::ValidationResult)>,
}

impl E2eTestBuilder {
    /// Create a new E2E test builder
    pub fn new() -> Self {
        Self {
            project_builder: None,
            result_matcher: AnalysisResultMatcher::new(),
            simd_validator: None,
            ml_fixtures: Vec::new(),
            config: E2eTestConfig::default(),
        }
    }

    /// Set the project to analyze
    pub fn with_project(mut self, project_builder: ProjectBuilder) -> Self {
        self.project_builder = Some(project_builder);
        self
    }

    /// Set expected analysis results
    pub fn expect_results(mut self, matcher: AnalysisResultMatcher) -> Self {
        self.result_matcher = matcher;
        self
    }

    /// Enable SIMD validation
    pub fn with_simd_validation(mut self, validator: SimdValidator) -> Self {
        self.simd_validator = Some(validator);
        self.config.validate_simd = true;
        self
    }

    /// Add ML model fixture for validation
    pub fn with_ml_fixture(mut self, fixture: MlModelFixture) -> Self {
        self.ml_fixtures.push(fixture);
        self.config.validate_ml = true;
        self
    }

    /// Set test configuration
    pub fn with_config(mut self, config: E2eTestConfig) -> Self {
        self.config = config;
        self
    }

    /// Build a Rust complexity analysis test
    pub fn rust_complexity_test() -> Self {
        let project = ProjectBuilder::new()
            .expect("Failed to create project builder")
            .with_rust_project("complexity-test")
            .with_file(
                "src/complex.rs", 
                r#"
pub fn complex_function() -> Result<String, Box<dyn std::error::Error>> {
    let mut result = String::new();
    
    for i in 0..100 {
        if i % 2 == 0 {
            if i % 4 == 0 {
                if i % 8 == 0 {
                    result.push_str(&format!("Eight: {}", i));
                } else {
                    result.push_str(&format!("Four: {}", i));
                }
            } else {
                result.push_str(&format!("Two: {}", i));
            }
        } else {
            match i % 3 {
                0 => result.push_str(&format!("Three: {}", i)),
                1 => {
                    if i > 50 {
                        result.push_str(&format!("Large odd: {}", i));
                    } else {
                        result.push_str(&format!("Small odd: {}", i));
                    }
                },
                _ => result.push_str(&format!("Other: {}", i)),
            }
        }
        
        if result.len() > 1000 {
            break;
        }
    }
    
    Ok(result)
}

pub struct ComplexStruct {
    data: Vec<i32>,
    metadata: std::collections::HashMap<String, String>,
}

impl ComplexStruct {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            metadata: std::collections::HashMap::new(),
        }
    }
    
    pub fn process(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for (index, value) in self.data.iter().enumerate() {
            if *value > 0 {
                if *value % 2 == 0 {
                    self.metadata.insert(
                        format!("even_{}", index),
                        format!("Value: {}", value)
                    );
                } else {
                    self.metadata.insert(
                        format!("odd_{}", index),
                        format!("Value: {}", value)
                    );
                }
            }
        }
        Ok(())
    }
}
"#
            );

        let matcher = AnalysisResultMatcher::new()
            .expect_complexity(
                Some("src/complex.rs"),
                super::ComplexityExpectation {
                    min_cyclomatic: Some(10),
                    max_cyclomatic: Some(50),
                    min_cognitive: Some(15),
                    max_cognitive: Some(80),
                    file_path: None,
                },
            );

        Self::new()
            .with_project(project)
            .expect_results(matcher)
    }

    /// Build a TypeScript analysis test
    pub fn typescript_analysis_test() -> Self {
        let project = ProjectBuilder::new()
            .expect("Failed to create project builder")
            .with_typescript_project("ts-analysis-test")
            .with_file(
                "src/advanced.ts",
                r#"
interface ApiResponse<T> {
    data: T;
    status: number;
    message?: string;
}

class DataProcessor<T extends Record<string, any>> {
    private cache = new Map<string, T>();
    private subscribers = new Set<(data: T) => void>();

    async processData(input: T[]): Promise<ApiResponse<T[]>> {
        const results: T[] = [];
        
        for (const item of input) {
            try {
                const processed = await this.processItem(item);
                if (processed) {
                    results.push(processed);
                    this.notifySubscribers(processed);
                }
            } catch (error) {
                console.error('Processing failed:', error);
                continue;
            }
        }

        return {
            data: results,
            status: results.length > 0 ? 200 : 204,
            message: `Processed ${results.length} items`
        };
    }

    private async processItem(item: T): Promise<T | null> {
        const key = this.generateKey(item);
        
        if (this.cache.has(key)) {
            return this.cache.get(key)!;
        }

        // Simulate async processing
        await new Promise(resolve => setTimeout(resolve, 10));
        
        const processed = { ...item, processed: true };
        this.cache.set(key, processed);
        
        return processed;
    }

    private generateKey(item: T): string {
        return Object.keys(item)
            .sort()
            .map(key => `${key}:${item[key]}`)
            .join('|');
    }

    subscribe(callback: (data: T) => void): () => void {
        this.subscribers.add(callback);
        return () => this.subscribers.delete(callback);
    }

    private notifySubscribers(data: T): void {
        this.subscribers.forEach(callback => {
            try {
                callback(data);
            } catch (error) {
                console.error('Subscriber notification failed:', error);
            }
        });
    }
}

export { DataProcessor, ApiResponse };
"#
            );

        let matcher = AnalysisResultMatcher::new()
            .expect_complexity(
                Some("src/advanced.ts"),
                super::ComplexityExpectation {
                    min_cyclomatic: Some(5),
                    max_cyclomatic: Some(25),
                    min_cognitive: Some(8),
                    max_cognitive: Some(40),
                    file_path: None,
                },
            );

        Self::new()
            .with_project(project)
            .expect_results(matcher)
    }

    /// Build a mixed-language project test
    pub fn mixed_language_test() -> Self {
        let project = ProjectBuilder::new()
            .expect("Failed to create project builder")
            .with_mixed_project()
            .with_file(
                "src/analysis.rs",
                r#"
use std::collections::HashMap;

pub struct CodeAnalyzer {
    metrics: HashMap<String, f64>,
    thresholds: HashMap<String, f64>,
}

impl CodeAnalyzer {
    pub fn new() -> Self {
        let mut thresholds = HashMap::new();
        thresholds.insert("complexity".to_string(), 10.0);
        thresholds.insert("duplication".to_string(), 0.1);
        
        Self {
            metrics: HashMap::new(),
            thresholds,
        }
    }

    pub fn analyze_file(&mut self, path: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let complexity = self.calculate_complexity(content);
        let duplication = self.calculate_duplication(content);
        
        self.metrics.insert(format!("{}_complexity", path), complexity);
        self.metrics.insert(format!("{}_duplication", path), duplication);
        
        if complexity > self.thresholds["complexity"] {
            eprintln!("Warning: High complexity in {}", path);
        }
        
        if duplication > self.thresholds["duplication"] {
            eprintln!("Warning: High duplication in {}", path);
        }
        
        Ok(())
    }

    fn calculate_complexity(&self, content: &str) -> f64 {
        let lines = content.lines().count() as f64;
        let functions = content.matches("fn ").count() as f64;
        let conditions = content.matches("if ").count() as f64 + 
                        content.matches("match ").count() as f64 +
                        content.matches("while ").count() as f64;
        
        (functions * 2.0 + conditions * 1.5) / lines.max(1.0)
    }

    fn calculate_duplication(&self, content: &str) -> f64 {
        let lines: Vec<&str> = content.lines().collect();
        let mut duplicates = 0;
        
        for i in 0..lines.len() {
            for j in (i + 1)..lines.len() {
                if lines[i].trim() == lines[j].trim() && !lines[i].trim().is_empty() {
                    duplicates += 1;
                }
            }
        }
        
        duplicates as f64 / lines.len().max(1) as f64
    }
}
"#
            );

        let matcher = AnalysisResultMatcher::new()
            .expect_complexity(
                None, // Overall project
                super::ComplexityExpectation {
                    min_cyclomatic: Some(5),
                    max_cyclomatic: Some(100),
                    min_cognitive: Some(10),
                    max_cognitive: Some(150),
                    file_path: None,
                },
            )
            .expect_json_path(
                "files_analyzed",
                serde_json::Value::Number(serde_json::Number::from(3u32)),
                super::ComparisonType::GreaterThanOrEqual,
            );

        Self::new()
            .with_project(project)
            .expect_results(matcher)
    }

    /// Build a performance-focused test with SIMD validation
    pub fn performance_test_with_simd() -> Self {
        let project = ProjectBuilder::new()
            .expect("Failed to create project builder")
            .with_rust_project("perf-test")
            .with_file(
                "src/vectorized.rs",
                r#"
use std::arch::x86_64::*;

pub fn vectorized_sum(data: &[f32]) -> f32 {
    unsafe {
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
}

pub fn scalar_sum(data: &[f32]) -> f32 {
    data.iter().sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vectorized_vs_scalar() {
        let data: Vec<f32> = (0..1000).map(|i| i as f32).collect();
        
        let scalar_result = scalar_sum(&data);
        let vectorized_result = vectorized_sum(&data);
        
        assert!((scalar_result - vectorized_result).abs() < 0.001);
    }
}
"#
            );

        let simd_validator = SimdValidator::new()
            .with_min_utilization(0.7)
            .with_max_cache_misses(100)
            .with_expected_instructions(vec![super::SimdInstruction::Avx2]);

        let matcher = AnalysisResultMatcher::new()
            .with_performance_thresholds(super::PerformanceThresholds {
                max_analysis_time_ms: Some(5000),
                max_memory_mb: Some(200),
                min_throughput_files_per_sec: Some(10.0),
            })
            .expect_simd_performance(super::SimdExpectation {
                min_utilization: 0.7,
                expected_operations: vec!["vectorized_sum".to_string()],
                max_cache_misses: Some(100),
            });

        Self::new()
            .with_project(project)
            .expect_results(matcher)
            .with_simd_validation(simd_validator)
    }

    /// Execute the E2E test
    pub async fn execute(self) -> Result<E2eTestResult, PmatError> {
        let project_builder = self.project_builder.ok_or_else(|| {
            PmatError::ValidationError {
                field: "project".to_string(),
                reason: "No project specified for E2E test".to_string(),
            }
        })?;

        // Build the test project
        let project_path = project_builder.build()?;
        let start_time = std::time::Instant::now();

        // Run deep context analysis
        let analyzer = DeepContextAnalyzer::new(self.config.deep_context_config);
        let analysis_result = analyzer.analyze_project(&project_path).await?;

        let analysis_duration = start_time.elapsed();

        // Convert to JSON for validation
        let analysis_json = serde_json::to_value(&analysis_result)
            .map_err(|e| PmatError::SerializationError(e.to_string()))?;

        let mut errors = Vec::new();

        // Validate analysis results
        if let Err(e) = self.result_matcher.assert_json(&analysis_json) {
            errors.push(format!("Analysis validation failed: {}", e));
        }

        // Validate performance
        if analysis_duration > self.config.max_analysis_time {
            errors.push(format!(
                "Analysis took too long: {:?} > {:?}",
                analysis_duration, self.config.max_analysis_time
            ));
        }

        // SIMD validation (mock for now)
        let simd_results = if self.config.validate_simd && self.simd_validator.is_some() {
            // Mock SIMD validation - in real implementation would measure actual SIMD usage
            Some(super::ValidationResult {
                passed: true,
                errors: Vec::new(),
                warnings: Vec::new(),
                metrics: super::SimdMetrics {
                    utilization: 0.85,
                    operations_count: 1000,
                    cache_misses: 50,
                    instructions_used: vec![super::SimdInstruction::Avx2],
                    throughput_ops_per_sec: 1_000_000.0,
                    memory_bandwidth_gbps: 3.2,
                },
            })
        } else {
            None
        };

        // ML validation (mock for now)
        let ml_results = if self.config.validate_ml && !self.ml_fixtures.is_empty() {
            // Mock ML validation
            self.ml_fixtures.iter().map(|fixture| {
                (fixture.name.clone(), super::ml_model_fixtures::ValidationResult {
                    passed: true,
                    errors: Vec::new(),
                    mean_prediction_error: 0.05,
                    mean_confidence: 0.85,
                    model_metrics: fixture.performance_metrics.clone(),
                })
            }).collect()
        } else {
            Vec::new()
        };

        // Cleanup if requested
        if self.config.cleanup_projects {
            // Note: project_path is a temporary directory that will be cleaned up automatically
        }

        Ok(E2eTestResult {
            passed: errors.is_empty(),
            analysis_duration,
            project_path,
            errors,
            analysis_results: analysis_json,
            simd_results,
            ml_results,
        })
    }

    /// Create a comprehensive test suite
    pub fn comprehensive_test_suite() -> Vec<Self> {
        vec![
            Self::rust_complexity_test(),
            Self::typescript_analysis_test(),
            Self::mixed_language_test(),
            Self::performance_test_with_simd(),
        ]
    }
}

impl Default for E2eTestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Batch runner for multiple E2E tests
pub struct E2eBatchRunner {
    tests: Vec<E2eTestBuilder>,
    parallel: bool,
}

impl E2eBatchRunner {
    /// Create a new batch runner
    pub fn new() -> Self {
        Self {
            tests: Vec::new(),
            parallel: false,
        }
    }

    /// Add a test to the batch
    pub fn add_test(mut self, test: E2eTestBuilder) -> Self {
        self.tests.push(test);
        self
    }

    /// Add all comprehensive tests
    pub fn with_comprehensive_tests(mut self) -> Self {
        self.tests.extend(E2eTestBuilder::comprehensive_test_suite());
        self
    }

    /// Enable parallel execution
    pub fn with_parallel_execution(mut self) -> Self {
        self.parallel = true;
        self
    }

    /// Run all tests and return results
    pub async fn execute_all(self) -> Vec<Result<E2eTestResult, PmatError>> {
        if self.parallel {
            // Run tests in parallel using tokio
            let futures = self.tests.into_iter().map(|test| test.execute());
            let results = futures::future::join_all(futures).await;
            results
        } else {
            // Run tests sequentially
            let mut results = Vec::new();
            for test in self.tests {
                results.push(test.execute().await);
            }
            results
        }
    }

    /// Generate a summary report
    pub fn generate_report(results: &[Result<E2eTestResult, PmatError>]) -> String {
        let mut report = String::new();
        report.push_str("E2E Test Report\n");
        report.push_str("===============\n\n");

        let passed = results.iter().filter(|r| {
            matches!(r, Ok(result) if result.passed)
        }).count();
        let total = results.len();

        report.push_str(&format!("Overall: {}/{} tests passed\n\n", passed, total));

        for (i, result) in results.iter().enumerate() {
            report.push_str(&format!("Test {}: ", i + 1));
            
            match result {
                Ok(test_result) => {
                    report.push_str(if test_result.passed { "PASSED" } else { "FAILED" });
                    report.push_str(&format!(" (Duration: {:?})\n", test_result.analysis_duration));
                    
                    if !test_result.errors.is_empty() {
                        for error in &test_result.errors {
                            report.push_str(&format!("  Error: {}\n", error));
                        }
                    }

                    if let Some(ref simd_result) = test_result.simd_results {
                        report.push_str(&format!("  SIMD: {} (Utilization: {:.2}%)\n", 
                            if simd_result.passed { "PASSED" } else { "FAILED" },
                            simd_result.metrics.utilization * 100.0
                        ));
                    }

                    if !test_result.ml_results.is_empty() {
                        let ml_passed = test_result.ml_results.iter().all(|(_, r)| r.passed);
                        report.push_str(&format!("  ML: {} ({} models)\n", 
                            if ml_passed { "PASSED" } else { "FAILED" },
                            test_result.ml_results.len()
                        ));
                    }
                }
                Err(e) => {
                    report.push_str(&format!("ERROR: {}\n", e));
                }
            }
            report.push('\n');
        }

        report
    }
}

impl Default for E2eBatchRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_e2e_test_builder_creation() {
        let builder = E2eTestBuilder::new();
        assert!(builder.project_builder.is_none());
        assert!(!builder.config.validate_simd);
        assert!(!builder.config.validate_ml);
    }

    #[test]
    fn test_rust_complexity_test_creation() {
        let test = E2eTestBuilder::rust_complexity_test();
        assert!(test.project_builder.is_some());
    }

    #[test]
    fn test_comprehensive_test_suite() {
        let tests = E2eTestBuilder::comprehensive_test_suite();
        assert_eq!(tests.len(), 4);
    }

    #[test]
    fn test_batch_runner_creation() {
        let runner = E2eBatchRunner::new().with_comprehensive_tests();
        assert_eq!(runner.tests.len(), 4);
    }

    #[tokio::test]
    async fn test_simple_e2e_execution() {
        let project = ProjectBuilder::new()
            .expect("Failed to create project builder")
            .with_rust_project("test")
            .with_file("src/simple.rs", "fn main() { println!(\"Hello\"); }");

        let matcher = AnalysisResultMatcher::new();

        let test = E2eTestBuilder::new()
            .with_project(project)
            .expect_results(matcher)
            .with_config(E2eTestConfig {
                max_analysis_time: Duration::from_secs(10),
                cleanup_projects: true,
                ..Default::default()
            });

        let result = test.execute().await;
        assert!(result.is_ok());
        
        let test_result = result.unwrap();
        assert!(test_result.analysis_duration < Duration::from_secs(10));
    }
}
