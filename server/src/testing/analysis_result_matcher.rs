//! Analysis Result Matcher for Test Assertions
//!
//! This module provides a fluent API for testing analysis results with support
//! for complex assertions across different analysis types.

use crate::models::error::PmatError;
use crate::services::deep_context::DeepContextResult;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

/// Builder for creating analysis result assertions
pub struct AnalysisResultMatcher {
    /// Expected complexity metrics
    complexity_expectations: HashMap<String, ComplexityExpectation>,
    /// Expected dead code patterns
    dead_code_expectations: Vec<DeadCodeExpectation>,
    /// Expected duplicate code patterns
    duplicate_expectations: Vec<DuplicateExpectation>,
    /// Expected defect predictions
    defect_expectations: Vec<DefectExpectation>,
    /// Expected graph metrics
    graph_expectations: HashMap<String, GraphExpectation>,
    /// Expected SIMD performance metrics
    simd_expectations: Option<SimdExpectation>,
    /// Expected ML model metrics
    ml_expectations: Option<MlExpectation>,
    /// General JSON path expectations
    json_path_expectations: Vec<JsonPathExpectation>,
    /// Performance thresholds
    performance_thresholds: PerformanceThresholds,
}

/// Complexity metric expectations
#[derive(Debug, Clone)]
pub struct ComplexityExpectation {
    pub min_cyclomatic: Option<u32>,
    pub max_cyclomatic: Option<u32>,
    pub min_cognitive: Option<u32>,
    pub max_cognitive: Option<u32>,
    pub file_path: Option<PathBuf>,
}

/// Dead code detection expectations
#[derive(Debug, Clone)]
pub struct DeadCodeExpectation {
    pub expected_count: Option<usize>,
    pub min_count: Option<usize>,
    pub max_count: Option<usize>,
    pub should_contain_file: Option<PathBuf>,
    pub should_not_contain_file: Option<PathBuf>,
    pub expected_line_ranges: Vec<(usize, usize)>,
}

/// Duplicate code detection expectations
#[derive(Debug, Clone)]
pub struct DuplicateExpectation {
    pub expected_pairs: Option<usize>,
    pub min_similarity: Option<f64>,
    pub max_similarity: Option<f64>,
    pub clone_type: Option<String>,
    pub expected_files: Vec<PathBuf>,
}

/// Defect prediction expectations
#[derive(Debug, Clone)]
pub struct DefectExpectation {
    pub file_path: PathBuf,
    pub min_score: Option<f64>,
    pub max_score: Option<f64>,
    pub min_confidence: Option<f64>,
    pub expected_components: Vec<String>,
}

/// Graph metrics expectations
#[derive(Debug, Clone)]
pub struct GraphExpectation {
    pub metric_name: String,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub expected_nodes: Option<usize>,
    pub expected_edges: Option<usize>,
}

/// SIMD performance expectations
#[derive(Debug, Clone)]
pub struct SimdExpectation {
    pub min_utilization: f64,
    pub expected_operations: Vec<String>,
    pub max_cache_misses: Option<u64>,
}

/// ML model performance expectations
#[derive(Debug, Clone)]
pub struct MlExpectation {
    pub max_inference_time_ms: u64,
    pub min_accuracy: Option<f64>,
    pub expected_model_version: Option<String>,
}

/// JSON path-based expectations
#[derive(Debug, Clone)]
pub struct JsonPathExpectation {
    pub path: String,
    pub expected_value: Value,
    pub comparison: ComparisonType,
}

/// Comparison types for JSON path assertions
#[derive(Debug, Clone)]
pub enum ComparisonType {
    Equals,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Contains,
    NotContains,
    Exists,
    NotExists,
}

/// Performance threshold expectations
#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    pub max_analysis_time_ms: Option<u64>,
    pub max_memory_mb: Option<u64>,
    pub min_throughput_files_per_sec: Option<f64>,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_analysis_time_ms: Some(30000), // 30 seconds default
            max_memory_mb: Some(500),         // 500MB default
            min_throughput_files_per_sec: Some(1.0), // 1 file/sec minimum
        }
    }
}

impl AnalysisResultMatcher {
    /// Create a new analysis result matcher
    pub fn new() -> Self {
        Self {
            complexity_expectations: HashMap::new(),
            dead_code_expectations: Vec::new(),
            duplicate_expectations: Vec::new(),
            defect_expectations: Vec::new(),
            graph_expectations: HashMap::new(),
            simd_expectations: None,
            ml_expectations: None,
            json_path_expectations: Vec::new(),
            performance_thresholds: PerformanceThresholds::default(),
        }
    }

    /// Expect complexity metrics for a specific file or overall
    pub fn expect_complexity(mut self, file: Option<&str>, expectation: ComplexityExpectation) -> Self {
        let key = file.unwrap_or("overall").to_string();
        self.complexity_expectations.insert(key, expectation);
        self
    }

    /// Expect dead code patterns
    pub fn expect_dead_code(mut self, expectation: DeadCodeExpectation) -> Self {
        self.dead_code_expectations.push(expectation);
        self
    }

    /// Expect duplicate code patterns
    pub fn expect_duplicates(mut self, expectation: DuplicateExpectation) -> Self {
        self.duplicate_expectations.push(expectation);
        self
    }

    /// Expect defect predictions
    pub fn expect_defects(mut self, expectation: DefectExpectation) -> Self {
        self.defect_expectations.push(expectation);
        self
    }

    /// Expect graph metrics
    pub fn expect_graph_metric(mut self, metric_name: &str, expectation: GraphExpectation) -> Self {
        self.graph_expectations.insert(metric_name.to_string(), expectation);
        self
    }

    /// Expect SIMD performance metrics
    pub fn expect_simd_performance(mut self, expectation: SimdExpectation) -> Self {
        self.simd_expectations = Some(expectation);
        self
    }

    /// Expect ML model performance
    pub fn expect_ml_performance(mut self, expectation: MlExpectation) -> Self {
        self.ml_expectations = Some(expectation);
        self
    }

    /// Expect JSON path values
    pub fn expect_json_path(mut self, path: &str, expected: Value, comparison: ComparisonType) -> Self {
        self.json_path_expectations.push(JsonPathExpectation {
            path: path.to_string(),
            expected_value: expected,
            comparison,
        });
        self
    }

    /// Set performance thresholds
    pub fn with_performance_thresholds(mut self, thresholds: PerformanceThresholds) -> Self {
        self.performance_thresholds = thresholds;
        self
    }

    /// Assert that a deep context result matches all expectations
    pub fn assert_deep_context(&self, result: &DeepContextResult) -> Result<(), AssertionError> {
        // Convert to JSON for unified processing
        let json_result = serde_json::to_value(result)
            .map_err(|e| AssertionError::SerializationError(e.to_string()))?;

        self.assert_json(&json_result)
    }

    /// Assert that a JSON result matches all expectations
    pub fn assert_json(&self, result: &Value) -> Result<(), AssertionError> {
        let mut errors = Vec::new();

        // Check complexity expectations
        for (key, expectation) in &self.complexity_expectations {
            if let Err(e) = self.assert_complexity_expectation(result, key, expectation) {
                errors.push(e);
            }
        }

        // Check dead code expectations
        for expectation in &self.dead_code_expectations {
            if let Err(e) = self.assert_dead_code_expectation(result, expectation) {
                errors.push(e);
            }
        }

        // Check duplicate expectations
        for expectation in &self.duplicate_expectations {
            if let Err(e) = self.assert_duplicate_expectation(result, expectation) {
                errors.push(e);
            }
        }

        // Check defect expectations
        for expectation in &self.defect_expectations {
            if let Err(e) = self.assert_defect_expectation(result, expectation) {
                errors.push(e);
            }
        }

        // Check graph expectations
        for (key, expectation) in &self.graph_expectations {
            if let Err(e) = self.assert_graph_expectation(result, key, expectation) {
                errors.push(e);
            }
        }

        // Check SIMD expectations
        if let Some(ref expectation) = self.simd_expectations {
            if let Err(e) = self.assert_simd_expectation(result, expectation) {
                errors.push(e);
            }
        }

        // Check ML expectations
        if let Some(ref expectation) = self.ml_expectations {
            if let Err(e) = self.assert_ml_expectation(result, expectation) {
                errors.push(e);
            }
        }

        // Check JSON path expectations
        for expectation in &self.json_path_expectations {
            if let Err(e) = self.assert_json_path_expectation(result, expectation) {
                errors.push(e);
            }
        }

        // Check performance thresholds
        if let Err(e) = self.assert_performance_thresholds(result) {
            errors.push(e);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(AssertionError::MultipleErrors(errors))
        }
    }

    fn assert_complexity_expectation(&self, result: &Value, key: &str, expectation: &ComplexityExpectation) -> Result<(), AssertionError> {
        let complexity_data = self.extract_complexity_data(result, key)?;

        if let Some(min) = expectation.min_cyclomatic {
            if complexity_data.cyclomatic < min {
                return Err(AssertionError::ComplexityTooLow {
                    file: key.to_string(),
                    metric: "cyclomatic".to_string(),
                    expected: min,
                    actual: complexity_data.cyclomatic,
                });
            }
        }

        if let Some(max) = expectation.max_cyclomatic {
            if complexity_data.cyclomatic > max {
                return Err(AssertionError::ComplexityTooHigh {
                    file: key.to_string(),
                    metric: "cyclomatic".to_string(),
                    expected: max,
                    actual: complexity_data.cyclomatic,
                });
            }
        }

        if let Some(min) = expectation.min_cognitive {
            if complexity_data.cognitive < min {
                return Err(AssertionError::ComplexityTooLow {
                    file: key.to_string(),
                    metric: "cognitive".to_string(),
                    expected: min,
                    actual: complexity_data.cognitive,
                });
            }
        }

        if let Some(max) = expectation.max_cognitive {
            if complexity_data.cognitive > max {
                return Err(AssertionError::ComplexityTooHigh {
                    file: key.to_string(),
                    metric: "cognitive".to_string(),
                    expected: max,
                    actual: complexity_data.cognitive,
                });
            }
        }

        Ok(())
    }

    fn assert_dead_code_expectation(&self, result: &Value, expectation: &DeadCodeExpectation) -> Result<(), AssertionError> {
        let dead_code_data = self.extract_dead_code_data(result)?;

        if let Some(expected) = expectation.expected_count {
            if dead_code_data.count != expected {
                return Err(AssertionError::CountMismatch {
                    category: "dead_code".to_string(),
                    expected,
                    actual: dead_code_data.count,
                });
            }
        }

        if let Some(min) = expectation.min_count {
            if dead_code_data.count < min {
                return Err(AssertionError::CountTooLow {
                    category: "dead_code".to_string(),
                    expected: min,
                    actual: dead_code_data.count,
                });
            }
        }

        if let Some(max) = expectation.max_count {
            if dead_code_data.count > max {
                return Err(AssertionError::CountTooHigh {
                    category: "dead_code".to_string(),
                    expected: max,
                    actual: dead_code_data.count,
                });
            }
        }

        Ok(())
    }

    fn assert_duplicate_expectation(&self, result: &Value, expectation: &DuplicateExpectation) -> Result<(), AssertionError> {
        let duplicate_data = self.extract_duplicate_data(result)?;

        if let Some(expected) = expectation.expected_pairs {
            if duplicate_data.pair_count != expected {
                return Err(AssertionError::CountMismatch {
                    category: "duplicate_pairs".to_string(),
                    expected,
                    actual: duplicate_data.pair_count,
                });
            }
        }

        Ok(())
    }

    fn assert_defect_expectation(&self, result: &Value, expectation: &DefectExpectation) -> Result<(), AssertionError> {
        let defect_data = self.extract_defect_data(result, &expectation.file_path)?;

        if let Some(min) = expectation.min_score {
            if defect_data.score < min {
                return Err(AssertionError::ScoreTooLow {
                    file: expectation.file_path.to_string_lossy().to_string(),
                    metric: "defect_score".to_string(),
                    expected: min,
                    actual: defect_data.score,
                });
            }
        }

        if let Some(max) = expectation.max_score {
            if defect_data.score > max {
                return Err(AssertionError::ScoreTooHigh {
                    file: expectation.file_path.to_string_lossy().to_string(),
                    metric: "defect_score".to_string(),
                    expected: max,
                    actual: defect_data.score,
                });
            }
        }

        Ok(())
    }

    fn assert_graph_expectation(&self, result: &Value, key: &str, expectation: &GraphExpectation) -> Result<(), AssertionError> {
        let graph_data = self.extract_graph_data(result, key)?;

        if let Some(min) = expectation.min_value {
            if graph_data.value < min {
                return Err(AssertionError::ScoreTooLow {
                    file: "graph".to_string(),
                    metric: key.to_string(),
                    expected: min,
                    actual: graph_data.value,
                });
            }
        }

        if let Some(max) = expectation.max_value {
            if graph_data.value > max {
                return Err(AssertionError::ScoreTooHigh {
                    file: "graph".to_string(),
                    metric: key.to_string(),
                    expected: max,
                    actual: graph_data.value,
                });
            }
        }

        Ok(())
    }

    fn assert_simd_expectation(&self, result: &Value, expectation: &SimdExpectation) -> Result<(), AssertionError> {
        let simd_data = self.extract_simd_data(result)?;

        if simd_data.utilization < expectation.min_utilization {
            return Err(AssertionError::ScoreTooLow {
                file: "simd".to_string(),
                metric: "utilization".to_string(),
                expected: expectation.min_utilization,
                actual: simd_data.utilization,
            });
        }

        Ok(())
    }

    fn assert_ml_expectation(&self, result: &Value, expectation: &MlExpectation) -> Result<(), AssertionError> {
        let ml_data = self.extract_ml_data(result)?;

        if ml_data.inference_time_ms > expectation.max_inference_time_ms {
            return Err(AssertionError::PerformanceTooSlow {
                metric: "ml_inference_time".to_string(),
                expected_ms: expectation.max_inference_time_ms,
                actual_ms: ml_data.inference_time_ms,
            });
        }

        Ok(())
    }

    fn assert_json_path_expectation(&self, result: &Value, expectation: &JsonPathExpectation) -> Result<(), AssertionError> {
        let actual_value = self.extract_json_path_value(result, &expectation.path)?;

        match expectation.comparison {
            ComparisonType::Equals => {
                if actual_value != expectation.expected_value {
                    return Err(AssertionError::ValueMismatch {
                        path: expectation.path.clone(),
                        expected: expectation.expected_value.clone(),
                        actual: actual_value,
                    });
                }
            }
            ComparisonType::Exists => {
                // Value exists if we got here without error
            }
            ComparisonType::NotExists => {
                return Err(AssertionError::UnexpectedValue {
                    path: expectation.path.clone(),
                    value: actual_value,
                });
            }
            // TODO: Implement other comparison types
            _ => {
                return Err(AssertionError::UnsupportedComparison {
                    comparison: format!("{:?}", expectation.comparison),
                });
            }
        }

        Ok(())
    }

    fn assert_performance_thresholds(&self, result: &Value) -> Result<(), AssertionError> {
        let performance_data = self.extract_performance_data(result)?;

        if let Some(max_time) = self.performance_thresholds.max_analysis_time_ms {
            if performance_data.analysis_time_ms > max_time {
                return Err(AssertionError::PerformanceTooSlow {
                    metric: "analysis_time".to_string(),
                    expected_ms: max_time,
                    actual_ms: performance_data.analysis_time_ms,
                });
            }
        }

        if let Some(max_memory) = self.performance_thresholds.max_memory_mb {
            if performance_data.memory_mb > max_memory {
                return Err(AssertionError::MemoryTooHigh {
                    expected_mb: max_memory,
                    actual_mb: performance_data.memory_mb,
                });
            }
        }

        Ok(())
    }

    // Helper methods for data extraction

    fn extract_complexity_data(&self, result: &Value, key: &str) -> Result<ComplexityData, AssertionError> {
        // Navigate JSON structure to find complexity data
        let path = if key == "overall" {
            "complexity.overall"
        } else {
            &format!("complexity.files.{}", key)
        };

        let complexity_obj = self.extract_json_path_value(result, path)?;
        
        Ok(ComplexityData {
            cyclomatic: complexity_obj.get("cyclomatic")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            cognitive: complexity_obj.get("cognitive")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
        })
    }

    fn extract_dead_code_data(&self, result: &Value) -> Result<DeadCodeData, AssertionError> {
        let dead_code_obj = self.extract_json_path_value(result, "dead_code")?;
        
        Ok(DeadCodeData {
            count: dead_code_obj.get("total_dead_functions")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as usize,
        })
    }

    fn extract_duplicate_data(&self, result: &Value) -> Result<DuplicateData, AssertionError> {
        let duplicate_obj = self.extract_json_path_value(result, "duplicates")?;
        
        Ok(DuplicateData {
            pair_count: duplicate_obj.get("total_pairs")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as usize,
        })
    }

    fn extract_defect_data(&self, result: &Value, file_path: &PathBuf) -> Result<DefectData, AssertionError> {
        let file_str = file_path.to_string_lossy();
        let path = format!("defect_predictions.{}", file_str);
        let defect_obj = self.extract_json_path_value(result, &path)?;
        
        Ok(DefectData {
            score: defect_obj.get("score")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            confidence: defect_obj.get("confidence")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
        })
    }

    fn extract_graph_data(&self, result: &Value, metric_name: &str) -> Result<GraphData, AssertionError> {
        let path = format!("graph_metrics.{}", metric_name);
        let graph_obj = self.extract_json_path_value(result, &path)?;
        
        Ok(GraphData {
            value: graph_obj.get("value")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
        })
    }

    fn extract_simd_data(&self, result: &Value) -> Result<SimdData, AssertionError> {
        let simd_obj = self.extract_json_path_value(result, "performance.simd")?;
        
        Ok(SimdData {
            utilization: simd_obj.get("utilization")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
        })
    }

    fn extract_ml_data(&self, result: &Value) -> Result<MlData, AssertionError> {
        let ml_obj = self.extract_json_path_value(result, "performance.ml")?;
        
        Ok(MlData {
            inference_time_ms: ml_obj.get("inference_time_ms")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
        })
    }

    fn extract_performance_data(&self, result: &Value) -> Result<PerformanceData, AssertionError> {
        let perf_obj = self.extract_json_path_value(result, "performance")?;
        
        Ok(PerformanceData {
            analysis_time_ms: perf_obj.get("analysis_time_ms")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            memory_mb: perf_obj.get("memory_mb")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
        })
    }

    fn extract_json_path_value(&self, result: &Value, path: &str) -> Result<Value, AssertionError> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = result;

        for part in parts {
            current = current.get(part).ok_or_else(|| AssertionError::PathNotFound {
                path: path.to_string(),
            })?;
        }

        Ok(current.clone())
    }
}

impl Default for AnalysisResultMatcher {
    fn default() -> Self {
        Self::new()
    }
}

// Helper data structures for extracted data

#[derive(Debug)]
struct ComplexityData {
    cyclomatic: u32,
    cognitive: u32,
}

#[derive(Debug)]
struct DeadCodeData {
    count: usize,
}

#[derive(Debug)]
struct DuplicateData {
    pair_count: usize,
}

#[derive(Debug)]
struct DefectData {
    score: f64,
    confidence: f64,
}

#[derive(Debug)]
struct GraphData {
    value: f64,
}

#[derive(Debug)]
struct SimdData {
    utilization: f64,
}

#[derive(Debug)]
struct MlData {
    inference_time_ms: u64,
}

#[derive(Debug)]
struct PerformanceData {
    analysis_time_ms: u64,
    memory_mb: u64,
}

/// Assertion errors for test failures
#[derive(Debug)]
pub enum AssertionError {
    ComplexityTooHigh {
        file: String,
        metric: String,
        expected: u32,
        actual: u32,
    },
    ComplexityTooLow {
        file: String,
        metric: String,
        expected: u32,
        actual: u32,
    },
    CountMismatch {
        category: String,
        expected: usize,
        actual: usize,
    },
    CountTooHigh {
        category: String,
        expected: usize,
        actual: usize,
    },
    CountTooLow {
        category: String,
        expected: usize,
        actual: usize,
    },
    ScoreTooHigh {
        file: String,
        metric: String,
        expected: f64,
        actual: f64,
    },
    ScoreTooLow {
        file: String,
        metric: String,
        expected: f64,
        actual: f64,
    },
    ValueMismatch {
        path: String,
        expected: Value,
        actual: Value,
    },
    UnexpectedValue {
        path: String,
        value: Value,
    },
    PathNotFound {
        path: String,
    },
    PerformanceTooSlow {
        metric: String,
        expected_ms: u64,
        actual_ms: u64,
    },
    MemoryTooHigh {
        expected_mb: u64,
        actual_mb: u64,
    },
    UnsupportedComparison {
        comparison: String,
    },
    SerializationError(String),
    MultipleErrors(Vec<AssertionError>),
}

impl std::fmt::Display for AssertionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssertionError::ComplexityTooHigh { file, metric, expected, actual } => {
                write!(f, "Complexity too high in {}: {} expected <= {}, got {}", file, metric, expected, actual)
            }
            AssertionError::ComplexityTooLow { file, metric, expected, actual } => {
                write!(f, "Complexity too low in {}: {} expected >= {}, got {}", file, metric, expected, actual)
            }
            AssertionError::CountMismatch { category, expected, actual } => {
                write!(f, "Count mismatch for {}: expected {}, got {}", category, expected, actual)
            }
            AssertionError::CountTooHigh { category, expected, actual } => {
                write!(f, "Count too high for {}: expected <= {}, got {}", category, expected, actual)
            }
            AssertionError::CountTooLow { category, expected, actual } => {
                write!(f, "Count too low for {}: expected >= {}, got {}", category, expected, actual)
            }
            AssertionError::ScoreTooHigh { file, metric, expected, actual } => {
                write!(f, "Score too high for {} in {}: expected <= {}, got {}", metric, file, expected, actual)
            }
            AssertionError::ScoreTooLow { file, metric, expected, actual } => {
                write!(f, "Score too low for {} in {}: expected >= {}, got {}", metric, file, expected, actual)
            }
            AssertionError::ValueMismatch { path, expected, actual } => {
                write!(f, "Value mismatch at {}: expected {}, got {}", path, expected, actual)
            }
            AssertionError::UnexpectedValue { path, value } => {
                write!(f, "Unexpected value at {}: {}", path, value)
            }
            AssertionError::PathNotFound { path } => {
                write!(f, "Path not found: {}", path)
            }
            AssertionError::PerformanceTooSlow { metric, expected_ms, actual_ms } => {
                write!(f, "Performance too slow for {}: expected <= {}ms, got {}ms", metric, expected_ms, actual_ms)
            }
            AssertionError::MemoryTooHigh { expected_mb, actual_mb } => {
                write!(f, "Memory usage too high: expected <= {}MB, got {}MB", expected_mb, actual_mb)
            }
            AssertionError::UnsupportedComparison { comparison } => {
                write!(f, "Unsupported comparison type: {}", comparison)
            }
            AssertionError::SerializationError(msg) => {
                write!(f, "Serialization error: {}", msg)
            }
            AssertionError::MultipleErrors(errors) => {
                writeln!(f, "Multiple assertion errors:")?;
                for (i, error) in errors.iter().enumerate() {
                    writeln!(f, "  {}. {}", i + 1, error)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for AssertionError {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_complexity_assertion_success() {
        let matcher = AnalysisResultMatcher::new()
            .expect_complexity(
                Some("test.rs"),
                ComplexityExpectation {
                    min_cyclomatic: Some(1),
                    max_cyclomatic: Some(10),
                    min_cognitive: None,
                    max_cognitive: Some(20),
                    file_path: None,
                },
            );

        let result = json!({
            "complexity": {
                "files": {
                    "test.rs": {
                        "cyclomatic": 5,
                        "cognitive": 15
                    }
                }
            }
        });

        assert!(matcher.assert_json(&result).is_ok());
    }

    #[test]
    fn test_complexity_assertion_failure() {
        let matcher = AnalysisResultMatcher::new()
            .expect_complexity(
                Some("test.rs"),
                ComplexityExpectation {
                    min_cyclomatic: Some(1),
                    max_cyclomatic: Some(10),
                    min_cognitive: None,
                    max_cognitive: Some(10),
                    file_path: None,
                },
            );

        let result = json!({
            "complexity": {
                "files": {
                    "test.rs": {
                        "cyclomatic": 15,
                        "cognitive": 25
                    }
                }
            }
        });

        assert!(matcher.assert_json(&result).is_err());
    }

    #[test]
    fn test_dead_code_assertion() {
        let matcher = AnalysisResultMatcher::new()
            .expect_dead_code(DeadCodeExpectation {
                expected_count: Some(3),
                min_count: None,
                max_count: None,
                should_contain_file: None,
                should_not_contain_file: None,
                expected_line_ranges: vec![],
            });

        let result = json!({
            "dead_code": {
                "total_dead_functions": 3
            }
        });

        assert!(matcher.assert_json(&result).is_ok());
    }

    #[test]
    fn test_json_path_assertion() {
        let matcher = AnalysisResultMatcher::new()
            .expect_json_path("analysis.version", json!("1.0.0"), ComparisonType::Equals);

        let result = json!({
            "analysis": {
                "version": "1.0.0"
            }
        });

        assert!(matcher.assert_json(&result).is_ok());
    }

    #[test]
    fn test_performance_threshold_assertion() {
        let matcher = AnalysisResultMatcher::new()
            .with_performance_thresholds(PerformanceThresholds {
                max_analysis_time_ms: Some(1000),
                max_memory_mb: Some(100),
                min_throughput_files_per_sec: Some(5.0),
            });

        let result = json!({
            "performance": {
                "analysis_time_ms": 500,
                "memory_mb": 50
            }
        });

        assert!(matcher.assert_json(&result).is_ok());
    }

    #[test]
    fn test_multiple_expectations() {
        let matcher = AnalysisResultMatcher::new()
            .expect_complexity(
                Some("test.rs"),
                ComplexityExpectation {
                    min_cyclomatic: Some(1),
                    max_cyclomatic: Some(10),
                    min_cognitive: None,
                    max_cognitive: Some(20),
                    file_path: None,
                },
            )
            .expect_dead_code(DeadCodeExpectation {
                max_count: Some(5),
                expected_count: None,
                min_count: None,
                should_contain_file: None,
                should_not_contain_file: None,
                expected_line_ranges: vec![],
            })
            .expect_json_path("analysis.status", json!("complete"), ComparisonType::Equals);

        let result = json!({
            "complexity": {
                "files": {
                    "test.rs": {
                        "cyclomatic": 5,
                        "cognitive": 15
                    }
                }
            },
            "dead_code": {
                "total_dead_functions": 2
            },
            "analysis": {
                "status": "complete"
            }
        });

        assert!(matcher.assert_json(&result).is_ok());
    }
}