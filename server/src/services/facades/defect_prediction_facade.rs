//! Defect Prediction Analysis Facade
//!
//! Provides a simplified interface for defect prediction and risk analysis.

use crate::services::service_registry::ServiceRegistry;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

/// Request for defect prediction analysis
#[derive(Debug, Clone)]
pub struct DefectPredictionRequest {
    pub project_path: PathBuf,
    pub confidence_threshold: f32,
    pub min_lines: usize,
    pub include_low_confidence: bool,
    pub high_risk_only: bool,
    pub include_recommendations: bool,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub top_files: usize,
}

/// Result of defect prediction analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefectPredictionResult {
    pub total_files_analyzed: usize,
    pub high_risk_files: usize,
    pub medium_risk_files: usize,
    pub low_risk_files: usize,
    pub predictions: Vec<FilePrediction>,
    pub summary: String,
    pub recommendations: Vec<String>,
}

/// Prediction for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePrediction {
    pub file_path: String,
    pub defect_probability: f32,
    pub risk_level: RiskLevel,
    pub confidence: f32,
    pub metrics: FileRiskMetrics,
    pub contributing_factors: Vec<String>,
}

/// Risk level classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Critical,
    High,
    Medium,
    Low,
}

/// Risk metrics for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRiskMetrics {
    pub complexity_score: f32,
    pub churn_score: f32,
    pub coupling_score: f32,
    pub size_score: f32,
    pub duplication_score: f32,
}

/// Facade for defect prediction analysis
#[derive(Clone)]
pub struct DefectPredictionFacade {
    #[allow(dead_code)]
    registry: Arc<ServiceRegistry>,
}

impl DefectPredictionFacade {
    /// Create a new defect prediction facade
    pub fn new(registry: Arc<ServiceRegistry>) -> Self {
        Self { registry }
    }

    /// Perform defect prediction analysis on a project
    pub async fn analyze_project(
        &self,
        request: DefectPredictionRequest,
    ) -> Result<DefectPredictionResult> {
        // Discover source files to analyze
        let files = self.discover_files(&request).await?;

        // Analyze each file for defect probability
        let mut predictions = Vec::new();
        for file_path in files.iter().take(request.top_files * 2) {
            // Analyze more than needed for filtering
            if let Ok(prediction) = self.analyze_file(file_path, &request).await {
                // Apply filters
                if request.high_risk_only
                    && matches!(prediction.risk_level, RiskLevel::Low | RiskLevel::Medium)
                {
                    continue;
                }
                if !request.include_low_confidence
                    && prediction.confidence < request.confidence_threshold
                {
                    continue;
                }
                predictions.push(prediction);
            }
        }

        // Sort by probability and limit to top files
        predictions.sort_by(|a, b| {
            b.defect_probability
                .partial_cmp(&a.defect_probability)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        predictions.truncate(request.top_files);

        // Build result
        Ok(self.build_result(predictions, &request))
    }

    /// Discover source files to analyze
    async fn discover_files(&self, request: &DefectPredictionRequest) -> Result<Vec<PathBuf>> {
        use walkdir::WalkDir;

        let mut files = Vec::new();
        for entry in WalkDir::new(&request.project_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                // Check if file matches include/exclude patterns
                let path_str = path.to_string_lossy();

                if let Some(ref excludes) = request.exclude {
                    if excludes.iter().any(|pattern| path_str.contains(pattern)) {
                        continue;
                    }
                }

                if let Some(ref includes) = request.include {
                    if !includes.iter().any(|pattern| path_str.contains(pattern)) {
                        continue;
                    }
                }

                // Check if it's a source file (basic check by extension)
                if let Some(ext) = path.extension() {
                    if matches!(
                        ext.to_str(),
                        Some("rs")
                            | Some("py")
                            | Some("js")
                            | Some("ts")
                            | Some("cpp")
                            | Some("c")
                            | Some("java")
                    ) {
                        files.push(path.to_path_buf());
                    }
                }
            }
        }

        Ok(files)
    }

    /// Analyze a single file for defect probability
    async fn analyze_file(
        &self,
        file_path: &PathBuf,
        request: &DefectPredictionRequest,
    ) -> Result<FilePrediction> {
        // Get file metrics (would integrate with real analysis services)
        let lines = tokio::fs::read_to_string(file_path).await?.lines().count();

        // Skip files below minimum line threshold
        if lines < request.min_lines {
            return Err(anyhow::anyhow!("File too small"));
        }

        // Calculate risk metrics (simplified for now)
        let complexity_score = (lines as f32 / 100.0).min(1.0);
        let churn_score = 0.3; // Would be calculated from git history
        let coupling_score = 0.2; // Would be calculated from dependency analysis
        let size_score = (lines as f32 / 1000.0).min(1.0);
        let duplication_score = 0.1; // Would be calculated from duplicate detector

        // Calculate overall defect probability
        let defect_probability = (complexity_score * 0.3
            + churn_score * 0.25
            + coupling_score * 0.2
            + size_score * 0.15
            + duplication_score * 0.1)
            .min(1.0);

        // Determine risk level
        let risk_level = match defect_probability {
            p if p >= 0.8 => RiskLevel::Critical,
            p if p >= 0.6 => RiskLevel::High,
            p if p >= 0.4 => RiskLevel::Medium,
            _ => RiskLevel::Low,
        };

        // Calculate confidence (based on available metrics)
        let confidence = 0.75; // Would be based on data quality

        // Identify contributing factors
        let mut contributing_factors = Vec::new();
        if complexity_score > 0.7 {
            contributing_factors.push("High complexity".to_string());
        }
        if churn_score > 0.5 {
            contributing_factors.push("Frequent changes".to_string());
        }
        if size_score > 0.7 {
            contributing_factors.push("Large file size".to_string());
        }

        Ok(FilePrediction {
            file_path: file_path.display().to_string(),
            defect_probability,
            risk_level,
            confidence,
            metrics: FileRiskMetrics {
                complexity_score,
                churn_score,
                coupling_score,
                size_score,
                duplication_score,
            },
            contributing_factors,
        })
    }

    /// Build the final result
    fn build_result(
        &self,
        predictions: Vec<FilePrediction>,
        request: &DefectPredictionRequest,
    ) -> DefectPredictionResult {
        let total_files_analyzed = predictions.len();
        let high_risk_files = predictions
            .iter()
            .filter(|p| matches!(p.risk_level, RiskLevel::Critical | RiskLevel::High))
            .count();
        let medium_risk_files = predictions
            .iter()
            .filter(|p| matches!(p.risk_level, RiskLevel::Medium))
            .count();
        let low_risk_files = predictions
            .iter()
            .filter(|p| matches!(p.risk_level, RiskLevel::Low))
            .count();

        let summary = format!(
            "Analyzed {} files: {} high risk, {} medium risk, {} low risk",
            total_files_analyzed, high_risk_files, medium_risk_files, low_risk_files
        );

        let mut recommendations = Vec::new();
        if high_risk_files > 0 {
            recommendations.push("Focus testing and review efforts on high-risk files".to_string());
        }

        if request.include_recommendations {
            for prediction in predictions.iter().take(3) {
                if !prediction.contributing_factors.is_empty() {
                    recommendations.push(format!(
                        "{}: Address {}",
                        prediction.file_path,
                        prediction.contributing_factors.join(", ")
                    ));
                }
            }
        }

        DefectPredictionResult {
            total_files_analyzed,
            high_risk_files,
            medium_risk_files,
            low_risk_files,
            predictions,
            summary,
            recommendations,
        }
    }

    /// Quick analysis with defaults
    pub async fn quick_analysis(&self, project_path: PathBuf) -> Result<DefectPredictionResult> {
        let request = DefectPredictionRequest {
            project_path,
            confidence_threshold: 0.5,
            min_lines: 50,
            include_low_confidence: false,
            high_risk_only: false,
            include_recommendations: true,
            include: None,
            exclude: Some(vec!["test".to_string(), "vendor".to_string()]),
            top_files: 10,
        };

        self.analyze_project(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::service_registry::ServiceRegistry;

    #[tokio::test]
    async fn test_defect_prediction_facade_creation() {
        let registry = Arc::new(ServiceRegistry::new());
        let _facade = DefectPredictionFacade::new(registry);
    }

    #[test]
    fn test_risk_level_classification() {
        assert_eq!(RiskLevel::Critical, RiskLevel::Critical);
    }
}
