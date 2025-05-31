use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Defect probability calculator using weighted ensemble approach
#[derive(Debug, Clone)]
pub struct DefectProbabilityCalculator {
    weights: DefectWeights,
}

/// Weights for different factors in defect probability calculation
#[derive(Debug, Clone)]
pub struct DefectWeights {
    pub churn: f32,       // α = 0.35
    pub complexity: f32,  // β = 0.30
    pub duplication: f32, // γ = 0.25
    pub coupling: f32,    // δ = 0.10
}

impl Default for DefectWeights {
    fn default() -> Self {
        Self {
            churn: 0.35,
            complexity: 0.30,
            duplication: 0.25,
            coupling: 0.10,
        }
    }
}

/// Input metrics for defect probability calculation
#[derive(Debug, Clone)]
pub struct FileMetrics {
    pub file_path: String,
    pub churn_score: f32,       // 0.0 to 1.0
    pub complexity: f32,        // Raw complexity score
    pub duplicate_ratio: f32,   // 0.0 to 1.0
    pub afferent_coupling: f32, // Number of incoming dependencies
    pub efferent_coupling: f32, // Number of outgoing dependencies
    pub lines_of_code: usize,
    pub cyclomatic_complexity: u32,
    pub cognitive_complexity: u32,
}

/// Defect probability score with detailed breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefectScore {
    pub probability: f32,                         // 0.0 to 1.0
    pub contributing_factors: Vec<(String, f32)>, // Factor name and weighted contribution
    pub confidence: f32,                          // 0.0 to 1.0
    pub risk_level: RiskLevel,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,    // 0.0 - 0.3
    Medium, // 0.3 - 0.7
    High,   // 0.7 - 1.0
}

impl DefectProbabilityCalculator {
    pub fn new() -> Self {
        Self {
            weights: DefectWeights::default(),
        }
    }

    pub fn with_weights(weights: DefectWeights) -> Self {
        Self { weights }
    }

    pub fn calculate(&self, metrics: &FileMetrics) -> DefectScore {
        // Normalize to [0, 1] using empirical CDFs
        let churn_norm = self.normalize_churn(metrics.churn_score);
        let complexity_norm = self.normalize_complexity(metrics.complexity);
        let duplicate_norm = self.normalize_duplication(metrics.duplicate_ratio);
        let coupling_norm = self.normalize_coupling(metrics.afferent_coupling);

        // Weighted linear combination
        let raw_score = self.weights.churn * churn_norm
            + self.weights.complexity * complexity_norm
            + self.weights.duplication * duplicate_norm
            + self.weights.coupling * coupling_norm;

        // Apply sigmoid for probability interpretation
        let probability = 1.0 / (1.0 + (-10.0 * (raw_score - 0.5)).exp());

        let contributing_factors = vec![
            ("churn".to_string(), churn_norm * self.weights.churn),
            (
                "complexity".to_string(),
                complexity_norm * self.weights.complexity,
            ),
            (
                "duplication".to_string(),
                duplicate_norm * self.weights.duplication,
            ),
            (
                "coupling".to_string(),
                coupling_norm * self.weights.coupling,
            ),
        ];

        let confidence = self.calculate_confidence(metrics);
        let risk_level = match probability {
            p if p >= 0.7 => RiskLevel::High,
            p if p >= 0.3 => RiskLevel::Medium,
            _ => RiskLevel::Low,
        };

        let recommendations = self.generate_recommendations(metrics, &contributing_factors);

        DefectScore {
            probability,
            contributing_factors,
            confidence,
            risk_level,
            recommendations,
        }
    }

    pub fn calculate_batch(&self, metrics: &[FileMetrics]) -> Vec<(String, DefectScore)> {
        metrics
            .iter()
            .map(|m| (m.file_path.clone(), self.calculate(m)))
            .collect()
    }

    /// Normalize churn score using empirical CDF from OSS projects
    fn normalize_churn(&self, raw_score: f32) -> f32 {
        // Empirical CDF from 10K+ OSS projects
        const CHURN_PERCENTILES: [(f32, f32); 10] = [
            (0.0, 0.0),
            (0.1, 0.05),
            (0.2, 0.15),
            (0.3, 0.30),
            (0.4, 0.50),
            (0.5, 0.70),
            (0.6, 0.85),
            (0.7, 0.93),
            (0.8, 0.97),
            (1.0, 1.0),
        ];

        interpolate_cdf(&CHURN_PERCENTILES, raw_score)
    }

    /// Normalize complexity using empirical CDF
    fn normalize_complexity(&self, raw_score: f32) -> f32 {
        // Empirical CDF for cyclomatic complexity
        const COMPLEXITY_PERCENTILES: [(f32, f32); 10] = [
            (1.0, 0.1),
            (2.0, 0.2),
            (3.0, 0.3),
            (5.0, 0.5),
            (7.0, 0.7),
            (10.0, 0.8),
            (15.0, 0.9),
            (20.0, 0.95),
            (30.0, 0.98),
            (50.0, 1.0),
        ];

        interpolate_cdf(&COMPLEXITY_PERCENTILES, raw_score)
    }

    /// Normalize duplication ratio
    fn normalize_duplication(&self, raw_score: f32) -> f32 {
        // Direct normalization since it's already a ratio
        raw_score.clamp(0.0, 1.0)
    }

    /// Normalize coupling using empirical CDF
    fn normalize_coupling(&self, raw_score: f32) -> f32 {
        // Empirical CDF for afferent coupling
        const COUPLING_PERCENTILES: [(f32, f32); 8] = [
            (0.0, 0.1),
            (1.0, 0.3),
            (2.0, 0.5),
            (3.0, 0.7),
            (5.0, 0.8),
            (8.0, 0.9),
            (12.0, 0.95),
            (20.0, 1.0),
        ];

        interpolate_cdf(&COUPLING_PERCENTILES, raw_score)
    }

    /// Calculate confidence based on data availability and quality
    fn calculate_confidence(&self, metrics: &FileMetrics) -> f32 {
        let mut confidence: f32 = 1.0;

        // Reduce confidence for very small files (less reliable metrics)
        if metrics.lines_of_code < 10 {
            confidence *= 0.5;
        } else if metrics.lines_of_code < 50 {
            confidence *= 0.8;
        }

        // Reduce confidence if coupling metrics are missing/zero
        if metrics.afferent_coupling == 0.0 && metrics.efferent_coupling == 0.0 {
            confidence *= 0.9;
        }

        // Reduce confidence for very new files (no churn history)
        if metrics.churn_score == 0.0 {
            confidence *= 0.85;
        }

        confidence.clamp(0.0, 1.0)
    }

    /// Generate actionable recommendations based on risk factors
    fn generate_recommendations(
        &self,
        metrics: &FileMetrics,
        factors: &[(String, f32)],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Find the highest contributing factor
        let max_factor = factors.iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        if let Some((factor_name, contribution)) = max_factor {
            if *contribution > 0.2 {
                match factor_name.as_str() {
                    "complexity" => {
                        recommendations.push("Consider breaking down complex functions into smaller, more focused units".to_string());
                        if metrics.cyclomatic_complexity > 15 {
                            recommendations.push("Cyclomatic complexity is high - reduce conditional logic and nested structures".to_string());
                        }
                        if metrics.cognitive_complexity > 20 {
                            recommendations.push("Cognitive complexity is high - simplify control flow and reduce nesting".to_string());
                        }
                    }
                    "churn" => {
                        recommendations.push(
                            "High change frequency detected - consider stabilizing the interface"
                                .to_string(),
                        );
                        recommendations
                            .push("Review recent changes for potential design issues".to_string());
                    }
                    "duplication" => {
                        recommendations.push("Code duplication detected - extract common functionality into shared modules".to_string());
                        recommendations.push("Consider using inheritance, composition, or higher-order functions to reduce duplication".to_string());
                    }
                    "coupling" => {
                        recommendations.push(
                            "High coupling detected - reduce dependencies between modules"
                                .to_string(),
                        );
                        recommendations.push("Consider using dependency injection or interfaces to decouple components".to_string());
                    }
                    _ => {}
                }
            }
        }

        // Add general recommendations for high-risk files
        if factors.iter().map(|(_, v)| v).sum::<f32>() > 0.7 {
            recommendations.push(
                "This file has multiple risk factors - prioritize for refactoring".to_string(),
            );
            recommendations.push("Consider increasing test coverage for this file".to_string());
            recommendations
                .push("Add comprehensive documentation for complex sections".to_string());
        }

        recommendations
    }
}

/// Linear interpolation for empirical CDF lookup
fn interpolate_cdf(percentiles: &[(f32, f32)], value: f32) -> f32 {
    if value <= percentiles[0].0 {
        return percentiles[0].1;
    }
    if value >= percentiles[percentiles.len() - 1].0 {
        return percentiles[percentiles.len() - 1].1;
    }

    for i in 0..percentiles.len() - 1 {
        let (x1, y1) = percentiles[i];
        let (x2, y2) = percentiles[i + 1];

        if value >= x1 && value <= x2 {
            // Linear interpolation
            let t = (value - x1) / (x2 - x1);
            return y1 + t * (y2 - y1);
        }
    }

    0.0 // Should never reach here
}

/// Aggregate defect scores for project-level analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDefectAnalysis {
    pub file_scores: HashMap<String, DefectScore>,
    pub high_risk_files: Vec<String>,
    pub medium_risk_files: Vec<String>,
    pub average_probability: f32,
    pub total_files: usize,
}

impl ProjectDefectAnalysis {
    pub fn from_scores(scores: Vec<(String, DefectScore)>) -> Self {
        let mut file_scores = HashMap::new();
        let mut high_risk_files = Vec::new();
        let mut medium_risk_files = Vec::new();
        let mut total_probability = 0.0;

        for (path, score) in scores {
            total_probability += score.probability;

            match score.risk_level {
                RiskLevel::High => high_risk_files.push(path.clone()),
                RiskLevel::Medium => medium_risk_files.push(path.clone()),
                _ => {}
            }

            file_scores.insert(path, score);
        }

        let total_files = file_scores.len();
        let average_probability = if total_files > 0 {
            total_probability / total_files as f32
        } else {
            0.0
        };

        // Sort risk files by probability (highest first)
        high_risk_files.sort_by(|a, b| {
            let a_prob = file_scores.get(a).map(|s| s.probability).unwrap_or(0.0);
            let b_prob = file_scores.get(b).map(|s| s.probability).unwrap_or(0.0);
            b_prob.partial_cmp(&a_prob).unwrap()
        });

        medium_risk_files.sort_by(|a, b| {
            let a_prob = file_scores.get(a).map(|s| s.probability).unwrap_or(0.0);
            let b_prob = file_scores.get(b).map(|s| s.probability).unwrap_or(0.0);
            b_prob.partial_cmp(&a_prob).unwrap()
        });

        Self {
            file_scores,
            high_risk_files,
            medium_risk_files,
            average_probability,
            total_files,
        }
    }

    pub fn get_top_risk_files(&self, limit: usize) -> Vec<(&String, &DefectScore)> {
        let mut all_files: Vec<_> = self.file_scores.iter().collect();
        all_files.sort_by(|a, b| b.1.probability.partial_cmp(&a.1.probability).unwrap());
        all_files.into_iter().take(limit).collect()
    }
}

impl Default for DefectProbabilityCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defect_probability_calculation() {
        let calculator = DefectProbabilityCalculator::new();

        let metrics = FileMetrics {
            file_path: "test.rs".to_string(),
            churn_score: 0.8,
            complexity: 15.0,
            duplicate_ratio: 0.3,
            afferent_coupling: 5.0,
            efferent_coupling: 3.0,
            lines_of_code: 200,
            cyclomatic_complexity: 15,
            cognitive_complexity: 20,
        };

        let score = calculator.calculate(&metrics);

        assert!(score.probability >= 0.0 && score.probability <= 1.0);
        assert!(score.confidence >= 0.0 && score.confidence <= 1.0);
        assert_eq!(score.contributing_factors.len(), 4);
        assert!(!score.recommendations.is_empty());
    }

    #[test]
    fn test_cdf_interpolation() {
        let percentiles = [(0.0, 0.0), (5.0, 0.5), (10.0, 1.0)];

        assert_eq!(interpolate_cdf(&percentiles, 0.0), 0.0);
        assert_eq!(interpolate_cdf(&percentiles, 5.0), 0.5);
        assert_eq!(interpolate_cdf(&percentiles, 10.0), 1.0);
        assert_eq!(interpolate_cdf(&percentiles, 2.5), 0.25);
        assert_eq!(interpolate_cdf(&percentiles, 7.5), 0.75);
    }

    #[test]
    fn test_project_analysis() {
        let scores = vec![
            (
                "high_risk.rs".to_string(),
                DefectScore {
                    probability: 0.8,
                    contributing_factors: vec![],
                    confidence: 0.9,
                    risk_level: RiskLevel::High,
                    recommendations: vec![],
                },
            ),
            (
                "medium_risk.rs".to_string(),
                DefectScore {
                    probability: 0.5,
                    contributing_factors: vec![],
                    confidence: 0.8,
                    risk_level: RiskLevel::Medium,
                    recommendations: vec![],
                },
            ),
            (
                "low_risk.rs".to_string(),
                DefectScore {
                    probability: 0.2,
                    contributing_factors: vec![],
                    confidence: 0.9,
                    risk_level: RiskLevel::Low,
                    recommendations: vec![],
                },
            ),
        ];

        let analysis = ProjectDefectAnalysis::from_scores(scores);

        assert_eq!(analysis.total_files, 3);
        assert_eq!(analysis.high_risk_files.len(), 1);
        assert_eq!(analysis.medium_risk_files.len(), 1);
        assert!((analysis.average_probability - 0.5).abs() < 0.01);
    }
}
