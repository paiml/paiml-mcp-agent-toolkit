#![allow(unused)]
#![cfg_attr(coverage_nightly, coverage(off))]
//! Complexity Analysis Facade
//!
//! Provides a simplified interface for complexity analysis operations.

use crate::services::service_registry::ServiceRegistry;
use anyhow::Result;
use serde::Serialize;
use std::path::Path;
use std::sync::Arc;

/// Request for complexity analysis
#[derive(Debug, Clone)]
pub struct ComplexityAnalysisRequest {
    pub path: std::path::PathBuf,
    pub language: Option<String>,
    pub include_tests: bool,
    pub max_complexity_threshold: Option<u32>,
    pub output_format: ComplexityOutputFormat,
}

/// Output format options for complexity analysis
#[derive(Debug, Clone)]
pub enum ComplexityOutputFormat {
    Json,
    Summary,
    Detailed,
}

/// Result of complexity analysis
#[derive(Debug, Clone, Serialize)]
pub struct ComplexityAnalysisResult {
    pub total_files: usize,
    pub violations: Vec<ComplexityViolation>,
    pub average_complexity: f64,
    pub max_complexity: u32,
    pub summary: String,
}

/// Individual complexity violation
#[derive(Debug, Clone, Serialize)]
pub struct ComplexityViolation {
    pub file_path: String,
    pub function_name: String,
    pub line_number: usize,
    pub complexity: u32,
    pub complexity_type: String,
}

/// Facade for complexity analysis operations
#[derive(Clone)]
pub struct ComplexityFacade {
    registry: Arc<ServiceRegistry>,
}

impl ComplexityFacade {
    /// Create a new complexity facade
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(registry: Arc<ServiceRegistry>) -> Self {
        Self { registry }
    }

    /// Perform complexity analysis on a project
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn analyze_project(
        &self,
        request: ComplexityAnalysisRequest,
    ) -> Result<ComplexityAnalysisResult> {
        // Wired to the same analyzer `pmat analyze complexity` uses.
        //
        // This returned a fixed `example_function` at line 42 with complexity
        // 15 for every project — two unrelated crates produced byte-identical
        // "analysis". `analyze comprehensive` reaches this facade through the
        // orchestrator, so that fabricated violation was what users and CI
        // gates actually saw.
        use crate::cli::analysis_utilities::analyze_project_files;
        use crate::services::complexity::{aggregate_results_with_thresholds, Violation};

        const MAX_CYCLOMATIC: u16 = 20;
        const MAX_COGNITIVE: u16 = 15;

        let file_metrics = analyze_project_files(
            &request.path,
            request.language.as_deref(),
            &[],
            MAX_CYCLOMATIC,
            MAX_COGNITIVE,
        )
        .await?;

        let total_files = file_metrics.len();
        let report = aggregate_results_with_thresholds(
            file_metrics,
            Some(MAX_CYCLOMATIC),
            Some(MAX_COGNITIVE),
        );

        let violations: Vec<ComplexityViolation> = report
            .violations
            .iter()
            .map(|v| {
                let (file, function, value, line, kind) = match v {
                    Violation::Error {
                        file,
                        function,
                        value,
                        line,
                        rule,
                        ..
                    }
                    | Violation::Warning {
                        file,
                        function,
                        value,
                        line,
                        rule,
                        ..
                    } => (file, function, *value, *line, rule),
                };
                ComplexityViolation {
                    file_path: file.clone(),
                    function_name: function.clone().unwrap_or_else(|| "<file>".to_string()),
                    line_number: line as usize,
                    complexity: value as u32,
                    complexity_type: kind.clone(),
                }
            })
            .collect();

        let max_complexity = violations.iter().map(|v| v.complexity).max().unwrap_or(0);
        let average_complexity = mean_cyclomatic(&report.files);

        Ok(ComplexityAnalysisResult {
            total_files,
            summary: format!(
                "Analyzed {} file(s) in {} with {} violation(s)",
                total_files,
                request.path.display(),
                violations.len()
            ),
            violations,
            average_complexity,
            max_complexity,
        })
    }

    /// Analyze a single file for complexity
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn analyze_file<P: AsRef<Path>>(
        &self,
        path: P,
        language: Option<&str>,
    ) -> Result<ComplexityAnalysisResult> {
        let request = ComplexityAnalysisRequest {
            path: path.as_ref().to_path_buf(),
            language: language.map(std::string::ToString::to_string),
            include_tests: true,
            max_complexity_threshold: Some(20),
            output_format: ComplexityOutputFormat::Detailed,
        };

        self.analyze_project(request).await
    }

    /// Get complexity thresholds for different languages
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_language_thresholds(&self, language: &str) -> ComplexityThresholds {
        match language {
            "rust" => ComplexityThresholds {
                warning: 15,
                error: 25,
                max_acceptable: 20,
            },
            "typescript" | "javascript" => ComplexityThresholds {
                warning: 10,
                error: 20,
                max_acceptable: 15,
            },
            "python" => ComplexityThresholds {
                warning: 12,
                error: 20,
                max_acceptable: 15,
            },
            _ => ComplexityThresholds {
                warning: 10,
                error: 20,
                max_acceptable: 15,
            },
        }
    }

    /// Check if complexity violations exceed thresholds
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn validate_complexity(
        &self,
        result: &ComplexityAnalysisResult,
        language: &str,
    ) -> ValidationResult {
        let thresholds = self.get_language_thresholds(language);

        let warnings = result
            .violations
            .iter()
            .filter(|v| v.complexity >= thresholds.warning && v.complexity < thresholds.error)
            .count();

        let errors = result
            .violations
            .iter()
            .filter(|v| v.complexity >= thresholds.error)
            .count();

        ValidationResult {
            passed: errors == 0,
            warnings,
            errors,
            max_complexity: result.max_complexity,
            threshold_exceeded: result.max_complexity > thresholds.max_acceptable,
        }
    }
}

/// Mean cyclomatic complexity over every analysed function, including class
/// methods — the same population `summary.total_functions` counts.
///
/// `average_complexity` used to be assigned `summary.median_cyclomatic`, so a
/// crate of nine trivial functions plus one function of cyclomatic 81 published
/// an "average" of 1.0 where the mean is 9.0. A median published under the name
/// `average` is a mislabelled number, not an approximation of one.
fn mean_cyclomatic(files: &[crate::services::complexity::FileComplexityMetrics]) -> f64 {
    let mut sum: u64 = 0;
    let mut count: u64 = 0;

    for file in files {
        for func in &file.functions {
            sum += u64::from(func.metrics.cyclomatic);
            count += 1;
        }
        for class in &file.classes {
            for method in &class.methods {
                sum += u64::from(method.metrics.cyclomatic);
                count += 1;
            }
        }
    }

    if count == 0 {
        0.0
    } else {
        sum as f64 / count as f64
    }
}

/// Complexity thresholds for different severity levels
#[derive(Debug, Clone)]
pub struct ComplexityThresholds {
    pub warning: u32,
    pub error: u32,
    pub max_acceptable: u32,
}

/// Result of complexity validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub passed: bool,
    pub warnings: usize,
    pub errors: usize,
    pub max_complexity: u32,
    pub threshold_exceeded: bool,
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::service_registry::ServiceRegistry;

    #[tokio::test]
    async fn test_complexity_facade_creation() {
        let registry = Arc::new(ServiceRegistry::new());
        let facade = ComplexityFacade::new(registry);

        // Test basic facade functionality
        let thresholds = facade.get_language_thresholds("rust");
        assert_eq!(thresholds.warning, 15);
        assert_eq!(thresholds.error, 25);
    }

    #[tokio::test]
    async fn test_complexity_validation() {
        let registry = Arc::new(ServiceRegistry::new());
        let facade = ComplexityFacade::new(registry);

        let result = ComplexityAnalysisResult {
            total_files: 1,
            violations: vec![ComplexityViolation {
                file_path: "test.rs".to_string(),
                function_name: "test_fn".to_string(),
                line_number: 1,
                complexity: 30,
                complexity_type: "cyclomatic".to_string(),
            }],
            average_complexity: 30.0,
            max_complexity: 30,
            summary: "Test".to_string(),
        };

        let validation = facade.validate_complexity(&result, "rust");
        assert!(!validation.passed);
        assert_eq!(validation.errors, 1);
        assert!(validation.threshold_exceeded);
    }

    #[test]
    fn test_mean_cyclomatic_is_the_mean_not_the_median() {
        use crate::services::complexity::{
            ComplexityMetrics, FileComplexityMetrics, FunctionComplexity,
        };

        fn metrics(cyclomatic: u16) -> ComplexityMetrics {
            ComplexityMetrics {
                cyclomatic,
                cognitive: 0,
                nesting_max: 0,
                lines: 1,
                halstead: None,
            }
        }

        // The cx2 repro: nine trivial functions plus one of cyclomatic 81.
        // Median 1.0, mean 9.0 — `average_complexity` published the median.
        let mut functions: Vec<FunctionComplexity> = (0..9)
            .map(|i| FunctionComplexity {
                name: format!("triv{i}"),
                line_start: i,
                line_end: i,
                metrics: metrics(1),
            })
            .collect();
        functions.push(FunctionComplexity {
            name: "nasty".to_string(),
            line_start: 10,
            line_end: 90,
            metrics: metrics(81),
        });

        let file = FileComplexityMetrics {
            path: "src/lib.rs".to_string(),
            total_complexity: metrics(90),
            functions,
            classes: vec![],
        };

        let mean = mean_cyclomatic(std::slice::from_ref(&file));
        assert!(
            (mean - 9.0).abs() < f64::EPSILON,
            "expected the mean 9.0, got {mean}"
        );
    }

    #[test]
    fn test_mean_cyclomatic_counts_class_methods_and_handles_empty() {
        use crate::services::complexity::{
            ClassComplexity, ComplexityMetrics, FileComplexityMetrics, FunctionComplexity,
        };

        let m = |c: u16| ComplexityMetrics {
            cyclomatic: c,
            cognitive: 0,
            nesting_max: 0,
            lines: 1,
            halstead: None,
        };

        assert_eq!(mean_cyclomatic(&[]), 0.0);

        let file = FileComplexityMetrics {
            path: "src/lib.rs".to_string(),
            total_complexity: m(9),
            functions: vec![FunctionComplexity {
                name: "free".to_string(),
                line_start: 1,
                line_end: 2,
                metrics: m(3),
            }],
            classes: vec![ClassComplexity {
                name: "C".to_string(),
                line_start: 3,
                line_end: 9,
                metrics: m(6),
                methods: vec![FunctionComplexity {
                    name: "method".to_string(),
                    line_start: 4,
                    line_end: 5,
                    metrics: m(7),
                }],
            }],
        };

        assert_eq!(mean_cyclomatic(std::slice::from_ref(&file)), 5.0);
    }
}
