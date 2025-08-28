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
    pub fn new(registry: Arc<ServiceRegistry>) -> Self {
        Self { registry }
    }

    /// Perform complexity analysis on a project
    pub async fn analyze_project(
        &self,
        request: ComplexityAnalysisRequest,
    ) -> Result<ComplexityAnalysisResult> {
        // This is a facade implementation that would orchestrate:
        // 1. Language detection if not specified
        // 2. File discovery and filtering
        // 3. Complexity analysis using appropriate service
        // 4. Result aggregation and formatting

        // For now, return a mock result to establish the interface
        Ok(ComplexityAnalysisResult {
            total_files: 1,
            violations: vec![ComplexityViolation {
                file_path: request.path.display().to_string(),
                function_name: "example_function".to_string(),
                line_number: 42,
                complexity: 15,
                complexity_type: "cyclomatic".to_string(),
            }],
            average_complexity: 8.5,
            max_complexity: 15,
            summary: format!("Analyzed {} with {} violations", request.path.display(), 1),
        })
    }

    /// Analyze a single file for complexity
    pub async fn analyze_file<P: AsRef<Path>>(
        &self,
        path: P,
        language: Option<&str>,
    ) -> Result<ComplexityAnalysisResult> {
        let request = ComplexityAnalysisRequest {
            path: path.as_ref().to_path_buf(),
            language: language.map(|s| s.to_string()),
            include_tests: true,
            max_complexity_threshold: Some(20),
            output_format: ComplexityOutputFormat::Detailed,
        };

        self.analyze_project(request).await
    }

    /// Get complexity thresholds for different languages
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
}
