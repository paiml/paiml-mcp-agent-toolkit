#![allow(unused)]
//! Defect Prediction Analysis Facade
//!
//! Provides a simplified interface for defect prediction and risk analysis.

#![cfg_attr(coverage_nightly, coverage(off))]
use crate::services::service_registry::ServiceRegistry;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
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

/// Risk metrics for a file, each normalised to 0.0-1.0.
///
/// `None` means pmat did not measure the factor for this file — not that it
/// measured zero. Four of the five were compile-time constants
/// (`churn_score = 0.3`, `coupling_score = 0.2`, `duplication_score = 0.1`,
/// `confidence = 0.75`), each tagged "would be calculated from …", so
/// `defect_probability` was a function of file length alone while five named
/// "ML-based" metrics were presented as measurements (GH #657). churn was
/// reported as 0.3 even for directories with no git repository at all.
///
/// See `contracts/pmat-no-fabrication-v1.yaml`, equation `measured_or_absent`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRiskMetrics {
    pub complexity_score: Option<f32>,
    /// Derived from git history. `None` when the project has no repository, or
    /// when the file has no commits in the window.
    pub churn_score: Option<f32>,
    pub coupling_score: Option<f32>,
    pub size_score: Option<f32>,
    pub duplication_score: Option<f32>,
}

/// Facade for defect prediction analysis
#[derive(Clone)]
pub struct DefectPredictionFacade {
    registry: Arc<ServiceRegistry>,
}

impl DefectPredictionFacade {
    /// Create a new defect prediction facade
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(registry: Arc<ServiceRegistry>) -> Self {
        Self { registry }
    }

    /// Perform defect prediction analysis on a project
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn analyze_project(
        &self,
        request: DefectPredictionRequest,
    ) -> Result<DefectPredictionResult> {
        // Discover source files to analyze
        let files = self.discover_files(&request).await?;

        // One git pass for the whole project rather than per file.
        let churn = Self::churn_by_path(&request.project_path).await;

        // Analyze each file for defect probability
        let mut predictions = Vec::new();
        for file_path in files.iter().take(request.top_files * 2) {
            // Analyze more than needed for filtering
            if let Ok(prediction) = self.analyze_file(file_path, &request, churn.as_ref()).await {
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
            .filter_map(std::result::Result::ok)
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
                        Some("rs" | "py" | "js" | "ts" | "cpp" | "c" | "java")
                    ) {
                        files.push(path.to_path_buf());
                    }
                }
            }
        }

        Ok(files)
    }

    /// Per-file churn from the project's git history, keyed by absolute path.
    ///
    /// `None` when the project has no repository — a directory with no git
    /// history has no churn to report, and reporting one anyway is what made
    /// `churn_score` 0.3 for a tree `analyze churn` correctly refuses to
    /// analyse (GH #657).
    async fn churn_by_path(project_path: &Path) -> Option<std::collections::HashMap<PathBuf, f32>> {
        let analysis =
            crate::services::git_analysis::GitAnalysisService::analyze_code_churn(project_path, 90)
                .ok()?;
        Some(
            analysis
                .files
                .into_iter()
                .map(|f| (f.path, f.churn_score))
                .collect(),
        )
    }

    /// Weighted defect probability over the factors that were actually
    /// measured, with the weights renormalised across them.
    ///
    /// The old formula multiplied three literals by their weights and added
    /// them in unconditionally, so ~55% of every score was a constant.
    fn weighted_probability(metrics: &FileRiskMetrics) -> Option<f32> {
        // (value, weight) — the published model weights.
        let factors = [
            (metrics.complexity_score, 0.30_f32),
            (metrics.churn_score, 0.25),
            (metrics.coupling_score, 0.20),
            (metrics.size_score, 0.15),
            (metrics.duplication_score, 0.10),
        ];

        let mut weighted = 0.0_f32;
        let mut total_weight = 0.0_f32;
        for (value, weight) in factors {
            if let Some(value) = value {
                weighted += value * weight;
                total_weight += weight;
            }
        }

        if total_weight <= 0.0 {
            return None;
        }
        Some((weighted / total_weight).clamp(0.0, 1.0))
    }

    /// Analyze a single file for defect probability
    async fn analyze_file(
        &self,
        file_path: &PathBuf,
        request: &DefectPredictionRequest,
        churn: Option<&std::collections::HashMap<PathBuf, f32>>,
    ) -> Result<FilePrediction> {
        let lines = tokio::fs::read_to_string(file_path).await?.lines().count();

        // Skip files below minimum line threshold
        if lines < request.min_lines {
            return Err(anyhow::anyhow!("File too small"));
        }

        // Complexity, coupling and duplication come from the TDG factor
        // calculators, which read the file. They were 0.6-from-LOC, 0.2 and 0.1.
        // TDG factors are on a 0-5 scale; the risk metrics are 0-1.
        let tdg = crate::services::tdg_calculator::TDGCalculator::new()
            .with_project_root(request.project_path.clone());
        let tdg_score = tdg.calculate_file(file_path).await.ok();
        let normalise = |v: f64| ((v / 5.0) as f32).clamp(0.0, 1.0);

        let complexity_score = tdg_score
            .as_ref()
            .map(|s| normalise(s.components.complexity));
        let coupling_score = tdg_score.as_ref().map(|s| normalise(s.components.coupling));
        let duplication_score = tdg_score
            .as_ref()
            .map(|s| normalise(s.components.duplication));

        // Real git churn, absent where there is no repository.
        let churn_score = churn.map(|map| map.get(file_path).copied().unwrap_or(0.0));

        #[allow(clippy::cast_precision_loss)]
        let size_score = Some((lines as f32 / 1000.0).min(1.0));

        let metrics = FileRiskMetrics {
            complexity_score,
            churn_score,
            coupling_score,
            size_score,
            duplication_score,
        };

        let defect_probability = Self::weighted_probability(&metrics)
            .ok_or_else(|| anyhow::anyhow!("no risk factor could be measured for this file"))?;

        // Determine risk level
        let risk_level = match defect_probability {
            p if p >= 0.8 => RiskLevel::Critical,
            p if p >= 0.6 => RiskLevel::High,
            p if p >= 0.4 => RiskLevel::Medium,
            _ => RiskLevel::Low,
        };

        // Confidence is the share of the model's weight that was actually
        // measured — the honest version of the old constant 0.75.
        let measured_weight = [
            (metrics.complexity_score, 0.30_f32),
            (metrics.churn_score, 0.25),
            (metrics.coupling_score, 0.20),
            (metrics.size_score, 0.15),
            (metrics.duplication_score, 0.10),
        ]
        .into_iter()
        .filter(|(value, _)| value.is_some())
        .map(|(_, weight)| weight)
        .sum::<f32>();
        let confidence = measured_weight.clamp(0.0, 1.0);

        // Identify contributing factors
        let mut contributing_factors = Vec::new();
        if complexity_score.is_some_and(|v| v > 0.7) {
            contributing_factors.push("High complexity".to_string());
        }
        if churn_score.is_some_and(|v| v > 0.5) {
            contributing_factors.push("Frequent changes".to_string());
        }
        if size_score.is_some_and(|v| v > 0.7) {
            contributing_factors.push("Large file size".to_string());
        }
        if duplication_score.is_some_and(|v| v > 0.5) {
            contributing_factors.push("Duplicated code".to_string());
        }
        if churn_score.is_none() {
            contributing_factors
                .push("Churn not measured (no git history for this project)".to_string());
        }

        Ok(FilePrediction {
            file_path: file_path.display().to_string(),
            defect_probability,
            risk_level,
            confidence,
            metrics,
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
            "Analyzed {total_files_analyzed} files: {high_risk_files} high risk, {medium_risk_files} medium risk, {low_risk_files} low risk"
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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

#[cfg_attr(coverage_nightly, coverage(off))]
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

    fn request_for(path: &Path) -> DefectPredictionRequest {
        DefectPredictionRequest {
            project_path: path.to_path_buf(),
            confidence_threshold: 0.0,
            min_lines: 1,
            include_low_confidence: true,
            high_risk_only: false,
            include_recommendations: false,
            include: None,
            exclude: None,
            top_files: 50,
        }
    }

    /// GH #657: churn/coupling/duplication/complexity were the same four
    /// constants for every file — {0.3}, {0.2}, {0.1}, and complexity derived
    /// only from LOC — so only file length moved the answer. Two files that
    /// differ in content must be able to differ in more than size.
    #[tokio::test]
    async fn risk_metrics_vary_with_file_content() {
        let dir = tempfile::TempDir::new().unwrap();

        // EXACTLY the same line count, very different content: one trivial,
        // one nested and import-heavy. Under the old constants these were
        // indistinguishable — complexity was `lines / 100` and coupling and
        // duplication were the literals 0.2 and 0.1 — so equal length meant
        // equal risk no matter what the code said.
        let mut simple = String::from("// a\n// b\n// c\n");
        let mut gnarly = String::from(
            "use std::collections::HashMap;\nuse std::path::PathBuf;\nuse std::sync::Arc;\n",
        );
        for i in 0..60 {
            simple.push_str(&format!("pub fn s{i}() -> i32 {{ {i} }}\n"));
            gnarly.push_str(&format!(
                "pub fn g{i}(x: i32) -> i32 {{ if x > 0 {{ for _ in 0..x {{ if x % 2 == 0 {{ return x; }} }} }} x }}\n"
            ));
        }
        assert_eq!(
            simple.lines().count(),
            gnarly.lines().count(),
            "the fixtures must differ only in content, not in length"
        );
        std::fs::write(dir.path().join("simple.rs"), &simple).unwrap();
        std::fs::write(dir.path().join("gnarly.rs"), &gnarly).unwrap();

        let facade = DefectPredictionFacade::new(Arc::new(ServiceRegistry::new()));
        let result = facade
            .analyze_project(request_for(dir.path()))
            .await
            .expect("analysis");

        assert_eq!(result.predictions.len(), 2, "both files must be analysed");

        let distinct_complexity: std::collections::BTreeSet<_> = result
            .predictions
            .iter()
            .filter_map(|p| p.metrics.complexity_score.map(|v| format!("{v:.4}")))
            .collect();
        let distinct_coupling: std::collections::BTreeSet<_> = result
            .predictions
            .iter()
            .filter_map(|p| p.metrics.coupling_score.map(|v| format!("{v:.4}")))
            .collect();

        assert!(
            distinct_complexity.len() > 1 || distinct_coupling.len() > 1,
            "complexity and coupling were both constant across two very different \
             files: complexity={distinct_complexity:?} coupling={distinct_coupling:?}"
        );
    }

    /// GH #657: churn was reported as 0.3 for a directory with NO git
    /// repository — a tree `analyze churn` correctly refuses to analyse.
    #[tokio::test]
    async fn churn_is_not_measured_without_a_git_repository() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("lib.rs"),
            "pub fn a() -> i32 { 1 }\npub fn b() -> i32 { 2 }\n",
        )
        .unwrap();

        let facade = DefectPredictionFacade::new(Arc::new(ServiceRegistry::new()));
        let result = facade
            .analyze_project(request_for(dir.path()))
            .await
            .expect("analysis");

        assert_eq!(result.predictions.len(), 1);
        assert_eq!(
            result.predictions[0].metrics.churn_score, None,
            "no git repository means churn was not measured; it used to report 0.3"
        );
        assert!(
            result.predictions[0].confidence < 1.0,
            "confidence must drop when a factor is missing; it used to be a flat 0.75"
        );
    }

    /// The probability must use only the factors that were measured — the old
    /// formula folded three literals in unconditionally.
    #[test]
    fn weighted_probability_renormalises_over_measured_factors() {
        let only_size = FileRiskMetrics {
            complexity_score: None,
            churn_score: None,
            coupling_score: None,
            size_score: Some(0.5),
            duplication_score: None,
        };
        let p = DefectPredictionFacade::weighted_probability(&only_size).unwrap();
        assert!(
            (p - 0.5).abs() < 1e-6,
            "one measured factor of 0.5 must give 0.5, got {p}"
        );

        let nothing = FileRiskMetrics {
            complexity_score: None,
            churn_score: None,
            coupling_score: None,
            size_score: None,
            duplication_score: None,
        };
        assert_eq!(
            DefectPredictionFacade::weighted_probability(&nothing),
            None,
            "with nothing measured there is no probability to report"
        );
    }
}
