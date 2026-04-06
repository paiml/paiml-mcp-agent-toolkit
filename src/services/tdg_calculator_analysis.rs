// Analysis, explanation, recommendations, and distribution for TDGCalculator

impl TDGCalculator {
    /// Analyze directory and generate TDG summary
    pub async fn analyze_directory(&self, path: &Path) -> Result<TDGSummary> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let files = self.discover_files(path).await?;
        let scores = self.calculate_batch(files.clone()).await?;

        let mut critical_files = 0;
        let mut warning_files = 0;
        let mut tdg_values: Vec<f64> = Vec::with_capacity(scores.len());

        for score in &scores {
            tdg_values.push(score.value);
            match score.severity {
                TDGSeverity::Critical => critical_files += 1,
                TDGSeverity::Warning => warning_files += 1,
                TDGSeverity::Normal => {}
            }
        }

        // Sort for percentile calculation
        tdg_values.sort_by(|a, b| a.total_cmp(b));

        let average_tdg = if tdg_values.is_empty() {
            0.0
        } else {
            tdg_values.iter().sum::<f64>() / tdg_values.len() as f64
        };

        let p95_tdg = self.percentile(&tdg_values, 0.95);
        let p99_tdg = self.percentile(&tdg_values, 0.99);

        // Find hotspots
        let mut indexed_scores: Vec<(usize, &TDGScore, PathBuf)> = scores
            .iter()
            .enumerate()
            .zip(files.iter())
            .map(|((idx, score), path)| (idx, score, path.clone()))
            .collect();

        indexed_scores.sort_by(|a, b| b.1.value.total_cmp(&a.1.value));

        let hotspots = indexed_scores
            .iter()
            .take(10)
            .map(|(_, score, path)| {
                let primary_factor = self.identify_primary_factor(&score.components);
                TDGHotspot {
                    path: path.display().to_string(),
                    tdg_score: score.value,
                    primary_factor,
                    estimated_hours: self.estimate_refactoring_hours(score.value),
                }
            })
            .collect();

        let estimated_debt_hours = scores
            .iter()
            .map(|s| self.estimate_refactoring_hours(s.value))
            .sum();

        Ok(TDGSummary {
            total_files: scores.len(),
            critical_files,
            warning_files,
            average_tdg,
            p95_tdg,
            p99_tdg,
            estimated_debt_hours,
            hotspots,
        })
    }

    /// Generate detailed TDG analysis with recommendations
    pub async fn analyze_path(&self, path: &Path) -> Result<TDGAnalysis> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let score = self.calculate_file(path).await?;
        let explanation = self.generate_explanation(&score);
        let recommendations = self.generate_recommendations(&score, path).await?;

        Ok(TDGAnalysis {
            score,
            explanation,
            recommendations,
        })
    }

    /// Identify primary contributing factor
    fn identify_primary_factor(&self, components: &TDGComponents) -> String {
        let mut factors = [
            (
                components.complexity * self.config.complexity_weight,
                "High Complexity",
            ),
            (
                components.churn * self.config.churn_weight,
                "Frequent Changes",
            ),
            (
                components.coupling * self.config.coupling_weight,
                "High Coupling",
            ),
            (
                components.domain_risk * self.config.domain_risk_weight,
                "Domain Risk",
            ),
            (
                components.duplication * self.config.duplication_weight,
                "Code Duplication",
            ),
        ];

        factors.sort_by(|a, b| b.0.total_cmp(&a.0));
        factors[0].1.to_string()
    }

    /// Estimate refactoring hours based on TDG score
    fn estimate_refactoring_hours(&self, tdg_score: f64) -> f64 {
        // Empirical formula: hours = base * multiplier^tdg
        let base_hours: f64 = 2.0;
        let multiplier: f64 = 1.8;

        base_hours * multiplier.powf(tdg_score)
    }

    /// Generate human-readable explanation
    fn generate_explanation(&self, score: &TDGScore) -> String {
        let mut explanation = format!(
            "Code Quality Gradient: {:.2} ({})\n\n",
            score.value,
            score.severity.as_str()
        );

        explanation.push_str("Component Breakdown:\n");

        let components = [
            (
                score.components.complexity,
                "Complexity",
                self.config.complexity_weight,
            ),
            (
                score.components.churn,
                "Code Churn",
                self.config.churn_weight,
            ),
            (
                score.components.coupling,
                "Coupling",
                self.config.coupling_weight,
            ),
            (
                score.components.domain_risk,
                "Domain Risk",
                self.config.domain_risk_weight,
            ),
            (
                score.components.duplication,
                "Duplication",
                self.config.duplication_weight,
            ),
        ];

        for (value, name, weight) in components {
            let contribution = value * weight;
            explanation.push_str(&format!(
                "- {name}: {value:.2} (contributes {contribution:.2} to total)\n"
            ));
        }

        explanation.push_str(&format!("\nConfidence: {:.0}%", score.confidence * 100.0));

        explanation
    }

    /// Generate actionable recommendations
    async fn generate_recommendations(
        &self,
        score: &TDGScore,
        _path: &Path,
    ) -> Result<Vec<TDGRecommendation>> {
        debug_assert!(_path.exists(), "_path must exist: {}", _path.display());
        let mut recommendations = Vec::new();

        // Complexity recommendations
        if score.components.complexity > 3.0 {
            recommendations.push(TDGRecommendation {
                recommendation_type: RecommendationType::ReduceComplexity,
                action: "Extract complex logic into smaller, focused functions".to_string(),
                expected_reduction: score.components.complexity
                    * 0.3
                    * self.config.complexity_weight,
                estimated_hours: 4.0,
                priority: 5,
            });
        }

        // Churn recommendations
        if score.components.churn > 3.0 {
            recommendations.push(TDGRecommendation {
                recommendation_type: RecommendationType::StabilizeChurn,
                action: "Add comprehensive tests to stabilize frequently changing code".to_string(),
                expected_reduction: score.components.churn * 0.4 * self.config.churn_weight,
                estimated_hours: 8.0,
                priority: 4,
            });
        }

        // Coupling recommendations
        if score.components.coupling > 3.0 {
            recommendations.push(TDGRecommendation {
                recommendation_type: RecommendationType::ReduceCoupling,
                action: "Introduce abstractions to reduce direct dependencies".to_string(),
                expected_reduction: score.components.coupling * 0.35 * self.config.coupling_weight,
                estimated_hours: 6.0,
                priority: 3,
            });
        }

        // Duplication recommendations
        if score.components.duplication > 2.0 {
            recommendations.push(TDGRecommendation {
                recommendation_type: RecommendationType::RemoveDuplication,
                action: "Extract duplicated code into shared utilities".to_string(),
                expected_reduction: score.components.duplication
                    * 0.5
                    * self.config.duplication_weight,
                estimated_hours: 3.0,
                priority: 2,
            });
        }

        // Sort by priority
        recommendations.sort_by_key(|r| std::cmp::Reverse(r.priority));

        Ok(recommendations)
    }

    /// Discover files for analysis
    async fn discover_files(&self, path: &Path) -> Result<Vec<PathBuf>> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        // Use the proper file discovery service with external dependency filtering
        let discovery = ProjectFileDiscovery::new(path.to_path_buf());
        discovery.discover_files()
    }

    /// Generate TDG distribution for visualization
    #[must_use]
    pub fn calculate_distribution(&self, scores: &[TDGScore]) -> TDGDistribution {
        debug_assert!(!scores.is_empty(), "scores must not be empty");
        let bucket_size = 0.5;
        let max_value = 5.0;
        let num_buckets = (max_value / bucket_size) as usize;

        let mut buckets = Vec::with_capacity(num_buckets);

        for i in 0..num_buckets {
            let min = i as f64 * bucket_size;
            let max = (i + 1) as f64 * bucket_size;

            let count = scores
                .iter()
                .filter(|s| s.value >= min && s.value < max)
                .count();

            let percentage = if scores.is_empty() {
                0.0
            } else {
                (count as f64 / scores.len() as f64) * 100.0
            };

            buckets.push(TDGBucket {
                min,
                max,
                count,
                percentage,
            });
        }

        TDGDistribution {
            buckets,
            total_files: scores.len(),
        }
    }
}
