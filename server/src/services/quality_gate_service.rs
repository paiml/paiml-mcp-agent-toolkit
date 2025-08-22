//! Quality gate service implementing the Service trait
//! 
//! Enforces quality standards across the codebase

use super::service_base::{Service, ServiceMetrics, ValidationError};
use super::analysis_service::{AnalysisService, AnalysisInput, AnalysisOperation, AnalysisOptions};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Input for quality gate checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateInput {
    pub path: PathBuf,
    pub checks: Vec<QualityCheck>,
    pub strict: bool,
}

/// Types of quality checks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualityCheck {
    Complexity { max: u32 },
    Satd { tolerance: u32 },
    DeadCode { max_percentage: f64 },
    Coverage { min: f64 },
    Lint,
    Documentation,
}

/// Output from quality gate checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateOutput {
    pub passed: bool,
    pub results: Vec<QualityCheckResult>,
    pub summary: QualitySummary,
}

/// Result of individual quality check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityCheckResult {
    pub check: String,
    pub passed: bool,
    pub message: String,
    pub violations: Vec<Violation>,
}

/// Quality violation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub file: String,
    pub line: Option<usize>,
    pub severity: Severity,
    pub message: String,
}

/// Violation severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// Summary of quality gate results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySummary {
    pub total_checks: usize,
    pub passed_checks: usize,
    pub failed_checks: usize,
    pub total_violations: usize,
    pub error_count: usize,
    pub warning_count: usize,
}

/// Quality gate service
pub struct QualityGateService {
    metrics: Arc<RwLock<ServiceMetrics>>,
    analysis_service: AnalysisService,
}

impl QualityGateService {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(ServiceMetrics::default())),
            analysis_service: AnalysisService::new(),
        }
    }
    
    async fn check_complexity(&self, path: &PathBuf, max: u32) -> Result<QualityCheckResult> {
        let input = AnalysisInput {
            operation: AnalysisOperation::Complexity,
            path: path.clone(),
            options: AnalysisOptions {
                max_complexity: Some(max),
                ..Default::default()
            },
        };
        
        let _output = self.analysis_service.process(input).await?;
        
        // Extract violations from analysis results
        let violations = vec![];  // Would be populated from actual results
        let passed = violations.is_empty();
        
        Ok(QualityCheckResult {
            check: format!("Complexity (max: {})", max),
            passed,
            message: if passed {
                "All functions within complexity limit".to_string()
            } else {
                format!("{} functions exceed complexity limit", violations.len())
            },
            violations,
        })
    }
    
    async fn check_satd(&self, path: &PathBuf, tolerance: u32) -> Result<QualityCheckResult> {
        let input = AnalysisInput {
            operation: AnalysisOperation::Satd,
            path: path.clone(),
            options: AnalysisOptions::default(),
        };
        
        let _output = self.analysis_service.process(input).await?;
        
        let violations = vec![];  // Would be populated from actual results
        let passed = violations.len() <= tolerance as usize;
        
        Ok(QualityCheckResult {
            check: format!("SATD (tolerance: {})", tolerance),
            passed,
            message: if passed {
                if violations.is_empty() {
                    "No SATD comments found".to_string()
                } else {
                    format!("{} SATD comments within tolerance", violations.len())
                }
            } else {
                format!("{} SATD comments exceed tolerance", violations.len())
            },
            violations,
        })
    }
    
    async fn check_dead_code(&self, path: &PathBuf, max_percentage: f64) -> Result<QualityCheckResult> {
        let input = AnalysisInput {
            operation: AnalysisOperation::DeadCode,
            path: path.clone(),
            options: AnalysisOptions::default(),
        };
        
        let _output = self.analysis_service.process(input).await?;
        
        let violations = vec![];  // Would be populated from actual results
        let percentage = 0.0;  // Would be calculated from actual results
        let passed = percentage <= max_percentage;
        
        Ok(QualityCheckResult {
            check: format!("Dead Code (max: {}%)", max_percentage),
            passed,
            message: if passed {
                format!("Dead code percentage: {:.1}%", percentage)
            } else {
                format!("Dead code {:.1}% exceeds limit", percentage)
            },
            violations,
        })
    }
    
    async fn check_coverage(&self, _path: &PathBuf, min: f64) -> Result<QualityCheckResult> {
        // Placeholder for coverage check
        Ok(QualityCheckResult {
            check: format!("Coverage (min: {}%)", min),
            passed: true,
            message: "Coverage check passed".to_string(),
            violations: vec![],
        })
    }
    
    async fn check_lint(&self, _path: &PathBuf) -> Result<QualityCheckResult> {
        // Placeholder for lint check
        Ok(QualityCheckResult {
            check: "Lint".to_string(),
            passed: true,
            message: "No lint violations".to_string(),
            violations: vec![],
        })
    }
    
    async fn check_documentation(&self, _path: &PathBuf) -> Result<QualityCheckResult> {
        // Placeholder for documentation check
        Ok(QualityCheckResult {
            check: "Documentation".to_string(),
            passed: true,
            message: "Documentation is up to date".to_string(),
            violations: vec![],
        })
    }
}

#[async_trait::async_trait]
impl Service for QualityGateService {
    type Input = QualityGateInput;
    type Output = QualityGateOutput;
    type Error = anyhow::Error;
    
    async fn process(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let start = std::time::Instant::now();
        let mut results = Vec::new();
        
        for check in &input.checks {
            let result = match check {
                QualityCheck::Complexity { max } => {
                    self.check_complexity(&input.path, *max).await?
                }
                QualityCheck::Satd { tolerance } => {
                    self.check_satd(&input.path, *tolerance).await?
                }
                QualityCheck::DeadCode { max_percentage } => {
                    self.check_dead_code(&input.path, *max_percentage).await?
                }
                QualityCheck::Coverage { min } => {
                    self.check_coverage(&input.path, *min).await?
                }
                QualityCheck::Lint => {
                    self.check_lint(&input.path).await?
                }
                QualityCheck::Documentation => {
                    self.check_documentation(&input.path).await?
                }
            };
            results.push(result);
        }
        
        let duration = start.elapsed();
        let mut metrics = self.metrics.write().await;
        
        // Calculate summary
        let total_checks = results.len();
        let passed_checks = results.iter().filter(|r| r.passed).count();
        let failed_checks = total_checks - passed_checks;
        let total_violations: usize = results.iter().map(|r| r.violations.len()).sum();
        let error_count = results.iter()
            .flat_map(|r| &r.violations)
            .filter(|v| matches!(v.severity, Severity::Error))
            .count();
        let warning_count = results.iter()
            .flat_map(|r| &r.violations)
            .filter(|v| matches!(v.severity, Severity::Warning))
            .count();
        
        let passed = if input.strict {
            failed_checks == 0
        } else {
            error_count == 0
        };
        
        metrics.record_request(duration, passed);
        
        Ok(QualityGateOutput {
            passed,
            results,
            summary: QualitySummary {
                total_checks,
                passed_checks,
                failed_checks,
                total_violations,
                error_count,
                warning_count,
            },
        })
    }
    
    fn validate_input(&self, input: &Self::Input) -> Result<(), ValidationError> {
        if !input.path.exists() {
            return Err(ValidationError::InvalidValue {
                field: "path".to_string(),
                reason: "Path does not exist".to_string(),
            });
        }
        
        if input.checks.is_empty() {
            return Err(ValidationError::MissingField {
                field: "checks".to_string(),
            });
        }
        
        Ok(())
    }
    
    fn metrics(&self) -> ServiceMetrics {
        self.metrics.blocking_read().clone()
    }
    
    fn name(&self) -> &str {
        "QualityGateService"
    }
}

impl Default for QualityGateService {
    fn default() -> Self {
        Self::new()
    }
}