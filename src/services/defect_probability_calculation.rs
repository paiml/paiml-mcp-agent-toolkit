impl DefectProbabilityCalculator {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            weights: DefectWeights::default(),
        }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// With weights.
    pub fn with_weights(weights: DefectWeights) -> Self {
        Self { weights }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Calculate.
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

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    /// Calculate batch.
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

        if let Some((factor_name, contribution)) = factors.iter().max_by(|a, b| a.1.total_cmp(&b.1)) {
            if *contribution > 0.2 {
                add_factor_recommendations(&mut recommendations, factor_name, metrics);
            }
        }

        if factors.iter().map(|(_, v)| v).sum::<f32>() > 0.7 {
            recommendations.push("This file has multiple risk factors - prioritize for refactoring".to_string());
            recommendations.push("Consider increasing test coverage for this file".to_string());
            recommendations.push("Add comprehensive documentation for complex sections".to_string());
        }

        recommendations
    }
}

fn add_factor_recommendations(recommendations: &mut Vec<String>, factor_name: &str, metrics: &FileMetrics) {
    match factor_name {
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
            recommendations.push("High change frequency detected - consider stabilizing the interface".to_string());
            recommendations.push("Review recent changes for potential design issues".to_string());
        }
        "duplication" => {
            recommendations.push("Code duplication detected - extract common functionality into shared modules".to_string());
            recommendations.push("Consider using inheritance, composition, or higher-order functions to reduce duplication".to_string());
        }
        "coupling" => {
            recommendations.push("High coupling detected - reduce dependencies between modules".to_string());
            recommendations.push("Consider using dependency injection or interfaces to decouple components".to_string());
        }
        _ => {}
    }
}

impl Default for DefectProbabilityCalculator {
    fn default() -> Self {
        Self::new()
    }
}
