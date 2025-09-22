use super::analyzer::{AnalyzerModule, Metrics};
use super::{ModuleError, PmatModule};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[async_trait]
pub trait ValidatorModule: Send + Sync {
    async fn validate(&self, metrics: &Metrics, thresholds: &Thresholds) -> ValidationResult;
    async fn validate_code(&self, code: &str, thresholds: &Thresholds) -> ValidationResult;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thresholds {
    pub max_complexity: u32,
    pub max_functions: usize,
    pub max_lines: usize,
    pub min_test_coverage: f64,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            max_complexity: 10,
            max_functions: 50,
            max_lines: 500,
            min_test_coverage: 0.8,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub passed: bool,
    pub violations: Vec<Violation>,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub rule: String,
    pub severity: Severity,
    pub message: String,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Clone)]
pub struct ValidatorImpl {
    analyzer: Option<Arc<dyn AnalyzerModule>>,
    strict_mode: bool,
}

impl ValidatorImpl {
    pub fn new() -> Self {
        Self {
            analyzer: None,
            strict_mode: false,
        }
    }

    pub fn with_analyzer(mut self, analyzer: Arc<dyn AnalyzerModule>) -> Self {
        self.analyzer = Some(analyzer);
        self
    }

    pub fn strict(mut self) -> Self {
        self.strict_mode = true;
        self
    }

    fn check_thresholds(&self, metrics: &Metrics, thresholds: &Thresholds) -> Vec<Violation> {
        let mut violations = Vec::new();

        if metrics.complexity > thresholds.max_complexity {
            violations.push(Violation {
                rule: "complexity".to_string(),
                severity: if self.strict_mode {
                    Severity::Error
                } else {
                    Severity::Warning
                },
                message: format!(
                    "Complexity {} exceeds maximum {}",
                    metrics.complexity, thresholds.max_complexity
                ),
                location: None,
            });
        }

        if metrics.functions > thresholds.max_functions {
            violations.push(Violation {
                rule: "functions".to_string(),
                severity: Severity::Warning,
                message: format!(
                    "Function count {} exceeds maximum {}",
                    metrics.functions, thresholds.max_functions
                ),
                location: None,
            });
        }

        if metrics.lines_of_code > thresholds.max_lines {
            violations.push(Violation {
                rule: "lines".to_string(),
                severity: Severity::Info,
                message: format!(
                    "Line count {} exceeds maximum {}",
                    metrics.lines_of_code, thresholds.max_lines
                ),
                location: None,
            });
        }

        violations
    }
}

#[async_trait]
impl ValidatorModule for ValidatorImpl {
    async fn validate(&self, metrics: &Metrics, thresholds: &Thresholds) -> ValidationResult {
        let violations = self.check_thresholds(metrics, thresholds);
        let has_errors = violations.iter().any(|v| v.severity == Severity::Error);

        let score = if violations.is_empty() {
            100.0
        } else {
            let penalty = violations.len() as f64 * 10.0;
            (100.0 - penalty).max(0.0)
        };

        ValidationResult {
            passed: !has_errors,
            violations,
            score,
        }
    }

    async fn validate_code(&self, code: &str, thresholds: &Thresholds) -> ValidationResult {
        if let Some(analyzer) = &self.analyzer {
            match analyzer.analyze(code).await {
                Ok(metrics) => self.validate(&metrics, thresholds).await,
                Err(e) => ValidationResult {
                    passed: false,
                    violations: vec![Violation {
                        rule: "analysis".to_string(),
                        severity: Severity::Error,
                        message: format!("Analysis failed: {}", e),
                        location: None,
                    }],
                    score: 0.0,
                },
            }
        } else {
            ValidationResult {
                passed: false,
                violations: vec![Violation {
                    rule: "configuration".to_string(),
                    severity: Severity::Error,
                    message: "No analyzer configured".to_string(),
                    location: None,
                }],
                score: 0.0,
            }
        }
    }
}

#[async_trait]
impl PmatModule for ValidatorImpl {
    type Input = (Metrics, Thresholds);
    type Output = ValidationResult;

    fn name(&self) -> &'static str {
        "ValidatorModule"
    }

    async fn initialize(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    async fn process(&self, input: Self::Input) -> Result<Self::Output, ModuleError> {
        Ok(self.validate(&input.0, &input.1).await)
    }

    async fn shutdown(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validator_passes() {
        let validator = ValidatorImpl::new();

        let metrics = Metrics {
            complexity: 5,
            lines_of_code: 100,
            functions: 10,
            classes: 2,
            imports: 5,
        };

        let thresholds = Thresholds::default();
        let result = validator.validate(&metrics, &thresholds).await;

        assert!(result.passed);
        assert!(result.violations.is_empty());
        assert_eq!(result.score, 100.0);
    }

    #[tokio::test]
    async fn test_validator_fails_complexity() {
        let validator = ValidatorImpl::new().strict();

        let metrics = Metrics {
            complexity: 15,
            lines_of_code: 100,
            functions: 10,
            classes: 2,
            imports: 5,
        };

        let thresholds = Thresholds::default();
        let result = validator.validate(&metrics, &thresholds).await;

        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].severity, Severity::Error);
    }
}
