//! Unified analysis service implementing the Service trait
//!
//! Provides various code analysis capabilities through a unified interface

use super::service_base::{Service, ServiceMetrics, ValidationError};
use crate::services::dead_code_analyzer::DeadCodeAnalyzer;
use crate::services::satd_detector::SATDDetector;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Input for analysis operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisInput {
    pub operation: AnalysisOperation,
    pub path: PathBuf,
    pub options: AnalysisOptions,
}

/// Available analysis operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisOperation {
    Complexity,
    Satd,
    DeadCode,
    All,
}

/// Options for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisOptions {
    pub max_complexity: Option<u32>,
    pub include_tests: bool,
    pub parallel: bool,
    pub format: OutputFormat,
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            max_complexity: Some(20),
            include_tests: false,
            parallel: true,
            format: OutputFormat::Json,
        }
    }
}

/// Output format for results
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Json,
    Text,
    Markdown,
}

/// Output from analysis operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisOutput {
    pub operation: AnalysisOperation,
    pub results: AnalysisResults,
    pub summary: AnalysisSummary,
}

/// Analysis results container
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnalysisResults {
    Complexity(ComplexityResults),
    Satd(SatdResults),
    DeadCode(DeadCodeResults),
    Combined(CombinedResults),
}

/// Complexity analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityResults {
    pub total_files: usize,
    pub average_complexity: f64,
    pub max_complexity: u32,
    pub violations: Vec<ComplexityViolation>,
}

/// SATD analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatdResults {
    pub total_files: usize,
    pub total_satd: usize,
    pub violations: Vec<SatdViolation>,
}

/// Dead code analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeResults {
    pub total_files: usize,
    pub dead_code_count: usize,
    pub dead_code_percentage: f64,
    pub unused_items: Vec<UnusedItem>,
}

/// Combined results from all analyses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombinedResults {
    pub complexity: ComplexityResults,
    pub satd: SatdResults,
    pub dead_code: DeadCodeResults,
}

/// Summary of analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSummary {
    pub files_analyzed: usize,
    pub total_issues: usize,
    pub critical_issues: usize,
    pub duration_ms: u64,
}

/// Individual complexity violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityViolation {
    pub file: String,
    pub function: String,
    pub complexity: u32,
    pub line: usize,
}

/// Individual SATD violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatdViolation {
    pub file: String,
    pub line: usize,
    pub comment: String,
    pub category: String,
}

/// Individual unused item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnusedItem {
    pub file: String,
    pub item: String,
    pub item_type: String,
    pub line: usize,
}

/// Unified analysis service
pub struct AnalysisService {
    metrics: Arc<RwLock<ServiceMetrics>>,
    satd_detector: SATDDetector,
}

impl AnalysisService {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(ServiceMetrics::default())),
            satd_detector: SATDDetector::new(),
        }
    }

    async fn analyze_complexity(
        &self,
        _path: &PathBuf,
        _options: &AnalysisOptions,
    ) -> Result<ComplexityResults> {
        // Implementation would call the actual complexity analyzer
        // This is a simplified version
        Ok(ComplexityResults {
            total_files: 10,
            average_complexity: 5.5,
            max_complexity: 15,
            violations: vec![],
        })
    }

    async fn analyze_satd(
        &self,
        path: &PathBuf,
        _options: &AnalysisOptions,
    ) -> Result<SatdResults> {
        // Use the actual SATD detector
        let results = self
            .satd_detector
            .analyze_project(path, true)
            .await
            .map_err(|e| anyhow::anyhow!("SATD analysis failed: {}", e))?;

        // Convert TechnicalDebt to SatdViolation
        let violations: Vec<SatdViolation> = results
            .items
            .into_iter()
            .map(|debt| SatdViolation {
                file: debt.file.to_string_lossy().to_string(),
                line: debt.line as usize,
                comment: debt.text,
                category: format!("{:?}", debt.category),
            })
            .collect();

        Ok(SatdResults {
            total_files: results.total_files_analyzed,
            total_satd: violations.len(),
            violations,
        })
    }

    async fn analyze_dead_code(
        &self,
        path: &PathBuf,
        options: &AnalysisOptions,
    ) -> Result<DeadCodeResults> {
        use crate::models::dead_code::DeadCodeAnalysisConfig;

        let config = DeadCodeAnalysisConfig {
            include_unreachable: true, // Include all dead code
            include_tests: options.include_tests,
            min_dead_lines: 1, // Include even single-line dead code
        };

        // Create a new analyzer instance (DeadCodeAnalyzer doesn't implement Clone)
        let mut analyzer = DeadCodeAnalyzer::new(DeadCodeAnalyzer::DEFAULT_CAPACITY);
        let analysis_result = analyzer.analyze_with_ranking(path, config).await?;

        // Convert ranked files to unused items
        let unused_items: Vec<UnusedItem> = analysis_result
            .ranked_files
            .into_iter()
            .flat_map(|file| {
                file.items
                    .into_iter()
                    .map(move |item| UnusedItem {
                        file: file.path.clone(),
                        item: item.name.clone(),
                        line: item.line as usize,
                        item_type: format!("{:?}", item.item_type),
                    })
            })
            .collect();

        let total_files = analysis_result.summary.total_files_analyzed;
        let dead_code_count = unused_items.len();
        let dead_code_percentage = if total_files > 0 {
            (dead_code_count as f64 / total_files as f64) * 100.0
        } else {
            0.0
        };

        Ok(DeadCodeResults {
            total_files,
            dead_code_count,
            dead_code_percentage,
            unused_items,
        })
    }
}

#[async_trait::async_trait]
impl Service for AnalysisService {
    type Input = AnalysisInput;
    type Output = AnalysisOutput;
    type Error = anyhow::Error;

    async fn process(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let start = std::time::Instant::now();

        let results = match input.operation {
            AnalysisOperation::Complexity => {
                let complexity = self.analyze_complexity(&input.path, &input.options).await?;
                AnalysisResults::Complexity(complexity)
            }
            AnalysisOperation::Satd => {
                let satd = self.analyze_satd(&input.path, &input.options).await?;
                AnalysisResults::Satd(satd)
            }
            AnalysisOperation::DeadCode => {
                let dead_code = self.analyze_dead_code(&input.path, &input.options).await?;
                AnalysisResults::DeadCode(dead_code)
            }
            AnalysisOperation::All => {
                let complexity = self.analyze_complexity(&input.path, &input.options).await?;
                let satd = self.analyze_satd(&input.path, &input.options).await?;
                let dead_code = self.analyze_dead_code(&input.path, &input.options).await?;

                AnalysisResults::Combined(CombinedResults {
                    complexity,
                    satd,
                    dead_code,
                })
            }
        };

        let duration = start.elapsed();
        let mut metrics = self.metrics.write().await;
        metrics.record_request(duration, true);

        // Calculate summary
        let (files_analyzed, total_issues, critical_issues) = match &results {
            AnalysisResults::Complexity(c) => (
                c.total_files,
                c.violations.len(),
                c.violations.iter().filter(|v| v.complexity > 20).count(),
            ),
            AnalysisResults::Satd(s) => (s.total_files, s.violations.len(), s.violations.len()),
            AnalysisResults::DeadCode(d) => (d.total_files, d.unused_items.len(), 0),
            AnalysisResults::Combined(c) => (
                c.complexity.total_files,
                c.complexity.violations.len()
                    + c.satd.violations.len()
                    + c.dead_code.unused_items.len(),
                c.complexity
                    .violations
                    .iter()
                    .filter(|v| v.complexity > 20)
                    .count()
                    + c.satd.violations.len(),
            ),
        };

        Ok(AnalysisOutput {
            operation: input.operation,
            results,
            summary: AnalysisSummary {
                files_analyzed,
                total_issues,
                critical_issues,
                duration_ms: duration.as_millis() as u64,
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

        if let Some(max) = input.options.max_complexity {
            if max == 0 || max > 100 {
                return Err(ValidationError::InvalidValue {
                    field: "max_complexity".to_string(),
                    reason: "Must be between 1 and 100".to_string(),
                });
            }
        }

        Ok(())
    }

    fn metrics(&self) -> ServiceMetrics {
        // Return a clone of current metrics
        self.metrics.blocking_read().clone()
    }

    fn name(&self) -> &str {
        "AnalysisService"
    }
}

impl Default for AnalysisService {
    fn default() -> Self {
        Self::new()
    }
}
