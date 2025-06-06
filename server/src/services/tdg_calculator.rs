use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use dashmap::DashMap;
use tokio::sync::Semaphore;

use crate::models::tdg::{
    RecommendationType, TDGAnalysis, TDGBucket, TDGComponents, TDGConfig, TDGDistribution,
    TDGHotspot, TDGRecommendation, TDGScore, TDGSeverity, TDGSummary,
};
use crate::services::file_discovery::ProjectFileDiscovery;
use crate::services::lightweight_provability_analyzer::{
    FunctionId, LightweightProvabilityAnalyzer,
};

/// Technical Debt Gradient Calculator
/// Primary service for calculating TDG scores to replace defect probability
pub struct TDGCalculator {
    config: TDGConfig,
    /// Simple cache for TDG scores
    cache: Arc<DashMap<PathBuf, TDGScore>>,
    semaphore: Arc<Semaphore>,
    /// Lightweight provability analyzer
    provability_analyzer: Arc<LightweightProvabilityAnalyzer>,
}

impl TDGCalculator {
    pub fn new() -> Self {
        Self::with_config(TDGConfig::default())
    }

    pub fn with_config(config: TDGConfig) -> Self {
        Self {
            config,
            cache: Arc::new(DashMap::new()),
            semaphore: Arc::new(Semaphore::new(num_cpus::get() * 2)),
            provability_analyzer: Arc::new(LightweightProvabilityAnalyzer::new()),
        }
    }

    /// Calculate TDG score for a single file
    pub async fn calculate_file(&self, path: &Path) -> Result<TDGScore> {
        // Check cache first
        if let Some(cached) = self.cache.get(&path.to_path_buf()) {
            return Ok(cached.clone());
        }

        let score = self.calculate_file_uncached(path).await?;
        self.cache.insert(path.to_path_buf(), score.clone());
        Ok(score)
    }

    /// Calculate TDG score without caching
    async fn calculate_file_uncached(&self, path: &Path) -> Result<TDGScore> {
        // Gather all metrics in parallel
        let (complexity, churn, coupling, duplication, provability) = tokio::try_join!(
            self.calculate_complexity_factor(path),
            self.calculate_churn_factor(path),
            self.calculate_coupling_factor(path),
            self.calculate_duplication_factor(path),
            self.calculate_provability_factor(path),
        )?;

        let domain_risk = self.calculate_domain_risk(path).await?;

        // Calculate weighted TDG value
        let components = TDGComponents {
            complexity,
            churn,
            coupling,
            domain_risk,
            duplication,
        };

        let value = self.calculate_weighted_tdg(&components, provability);
        let severity = TDGSeverity::from(value);

        Ok(TDGScore {
            value,
            components,
            severity,
            percentile: 0.0, // Will be calculated in batch analysis
            confidence: self.calculate_confidence(&components),
        })
    }

    /// Calculate TDG scores for multiple files with parallelization
    pub async fn calculate_batch(&self, files: Vec<PathBuf>) -> Result<Vec<TDGScore>> {
        let tasks: Vec<_> = files
            .into_iter()
            .map(|file| {
                let calculator = self.clone();
                tokio::spawn(async move {
                    let _permit = calculator.semaphore.acquire().await?;
                    calculator.calculate_file(&file).await
                })
            })
            .collect();

        let mut results = Vec::with_capacity(tasks.len());
        for task in tasks {
            results.push(task.await??);
        }

        // Calculate percentiles
        self.calculate_percentiles(&mut results);

        Ok(results)
    }

    /// Analyze directory and generate TDG summary
    pub async fn analyze_directory(&self, path: &Path) -> Result<TDGSummary> {
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
        tdg_values.sort_by(|a, b| a.partial_cmp(b).unwrap());

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

        indexed_scores.sort_by(|a, b| b.1.value.partial_cmp(&a.1.value).unwrap());

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
        let score = self.calculate_file(path).await?;
        let explanation = self.generate_explanation(&score);
        let recommendations = self.generate_recommendations(&score, path).await?;

        Ok(TDGAnalysis {
            score,
            explanation,
            recommendations,
        })
    }

    /// Calculate complexity factor (normalized 0-5)
    async fn calculate_complexity_factor(&self, path: &Path) -> Result<f64> {
        // Simplified complexity calculation based on file content
        let content = tokio::fs::read_to_string(path).await?;
        let lines: Vec<&str> = content.lines().collect();

        // Basic complexity heuristics
        let mut complexity = 0;
        let mut nesting_level = 0;

        for line in &lines {
            let trimmed = line.trim();
            // Count control flow statements
            if trimmed.starts_with("if ")
                || trimmed.starts_with("elif ")
                || trimmed.starts_with("while ")
                || trimmed.starts_with("for ")
                || trimmed.starts_with("match ")
                || trimmed.starts_with("case ")
            {
                complexity += 1 + nesting_level;
            }

            // Track nesting
            nesting_level += trimmed.matches('{').count();
            nesting_level = nesting_level.saturating_sub(trimmed.matches('}').count());
        }

        // Normalize to 0-5 scale
        let normalized = (complexity as f64 / 25.0).min(5.0);
        Ok(normalized)
    }

    /// Calculate churn factor based on git history
    async fn calculate_churn_factor(&self, path: &Path) -> Result<f64> {
        // Simplified churn calculation - would integrate with git
        // For now, return a default value based on file age
        match tokio::fs::metadata(path).await {
            Ok(metadata) => {
                // Newer files tend to have higher churn
                if let Ok(modified) = metadata.modified() {
                    if let Ok(elapsed) = modified.elapsed() {
                        let days_old = elapsed.as_secs() / 86400;
                        // Files modified recently have higher churn factor
                        if days_old < 7 {
                            Ok(3.0)
                        } else if days_old < 30 {
                            Ok(2.0)
                        } else if days_old < 90 {
                            Ok(1.0)
                        } else {
                            Ok(0.5)
                        }
                    } else {
                        Ok(1.0)
                    }
                } else {
                    Ok(1.0)
                }
            }
            Err(_) => Ok(1.0),
        }
    }

    /// Calculate coupling factor
    async fn calculate_coupling_factor(&self, path: &Path) -> Result<f64> {
        // This would integrate with DAG analysis
        // For now, use a simplified metric based on imports
        let content = tokio::fs::read_to_string(path).await?;
        let import_count = self.count_imports(&content);

        // Normalize: >15 imports indicates high coupling
        Ok((import_count as f64 / 15.0).min(5.0))
    }

    /// Calculate code duplication factor
    async fn calculate_duplication_factor(&self, path: &Path) -> Result<f64> {
        // Simplified duplication detection
        let content = tokio::fs::read_to_string(path).await?;
        let lines: Vec<&str> = content
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with("//") && !l.starts_with("#"))
            .collect();

        if lines.len() < 10 {
            return Ok(0.0);
        }

        // Count duplicate lines (very basic)
        let mut line_counts = HashMap::new();
        for line in &lines {
            if line.len() > 10 {
                // Only count substantial lines
                *line_counts.entry(*line).or_insert(0) += 1;
            }
        }

        let duplicate_lines: usize = line_counts
            .values()
            .filter(|&&count| count > 1)
            .map(|&count| count - 1)
            .sum();

        let duplication_percentage = (duplicate_lines as f64 / lines.len() as f64) * 100.0;

        // Normalize: >30% duplication is critical
        Ok((duplication_percentage / 30.0).min(5.0))
    }

    /// Calculate domain-specific risk factors
    async fn calculate_domain_risk(&self, path: &Path) -> Result<f64> {
        let mut risk: f64 = 0.0;

        // Security-sensitive paths
        if path.to_string_lossy().contains("auth")
            || path.to_string_lossy().contains("crypto")
            || path.to_string_lossy().contains("security")
        {
            risk += 2.0;
        }

        // Database/persistence layer
        if path.to_string_lossy().contains("database")
            || path.to_string_lossy().contains("migration")
        {
            risk += 1.5;
        }

        // External API integrations
        if path.to_string_lossy().contains("api") || path.to_string_lossy().contains("integration")
        {
            risk += 1.0;
        }

        Ok(risk.min(5.0))
    }

    /// Calculate weighted TDG value from components
    fn calculate_weighted_tdg(&self, components: &TDGComponents, provability_factor: f64) -> f64 {
        let base_weighted = components.complexity * self.config.complexity_weight
            + components.churn * self.config.churn_weight
            + components.coupling * self.config.coupling_weight
            + components.domain_risk * self.config.domain_risk_weight
            + components.duplication * self.config.duplication_weight;

        // Apply provability factor (higher provability reduces TDG)
        let adjusted = base_weighted * (1.0 - provability_factor * 0.2);

        // Ensure result is in 0-5 range
        adjusted.clamp(0.0, 5.0)
    }

    /// Calculate confidence level based on data availability
    fn calculate_confidence(&self, components: &TDGComponents) -> f64 {
        let mut confidence = 1.0;

        // Reduce confidence for zero values (likely missing data)
        if components.churn == 0.0 {
            confidence *= 0.8;
        }
        if components.coupling == 0.0 {
            confidence *= 0.9;
        }
        if components.duplication == 0.0 {
            confidence *= 0.95;
        }

        confidence
    }

    /// Calculate percentiles for a batch of scores
    fn calculate_percentiles(&self, scores: &mut [TDGScore]) {
        let mut values: Vec<f64> = scores.iter().map(|s| s.value).collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        for score in scores.iter_mut() {
            let position = values
                .binary_search_by(|&v| v.partial_cmp(&score.value).unwrap())
                .unwrap_or_else(|i| i);
            score.percentile = (position as f64 / values.len() as f64) * 100.0;
        }
    }

    /// Calculate specific percentile value
    fn percentile(&self, sorted_values: &[f64], percentile: f64) -> f64 {
        if sorted_values.is_empty() {
            return 0.0;
        }

        let index = (sorted_values.len() as f64 * percentile) as usize;
        let index = index.min(sorted_values.len() - 1);
        sorted_values[index]
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

        factors.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
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
            "Technical Debt Gradient: {:.2} ({})\n\n",
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
        // Use the proper file discovery service with external dependency filtering
        let discovery = ProjectFileDiscovery::new(path.to_path_buf());
        discovery.discover_files()
    }

    fn count_imports(&self, content: &str) -> usize {
        let patterns = [
            r"^use\s+",             // Rust
            r"^import\s+",          // Python/JS/TS
            r"^from\s+.*\s+import", // Python
            r"^require\(",          // Node.js
        ];

        content
            .lines()
            .filter(|line| {
                patterns
                    .iter()
                    .any(|p| regex::Regex::new(p).unwrap().is_match(line.trim()))
            })
            .count()
    }

    /// Calculate provability factor using lightweight analysis
    async fn calculate_provability_factor(&self, path: &Path) -> Result<f64> {
        // Extract function information from file
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let func_id = FunctionId {
            file_path: path.to_string_lossy().to_string(),
            function_name: file_name.to_string(),
            line_number: 1,
        };

        let summaries = self
            .provability_analyzer
            .analyze_incrementally(&[func_id])
            .await;

        if let Some(summary) = summaries.first() {
            Ok(summary.provability_score)
        } else {
            Ok(0.0) // Default to no provability
        }
    }

    /// Generate TDG distribution for visualization
    pub fn calculate_distribution(&self, scores: &[TDGScore]) -> TDGDistribution {
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

impl Default for TDGCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for TDGCalculator {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            cache: self.cache.clone(),
            semaphore: self.semaphore.clone(),
            provability_analyzer: self.provability_analyzer.clone(),
        }
    }
}

impl Default for TDGComponents {
    fn default() -> Self {
        Self {
            complexity: 0.0,
            churn: 0.0,
            coupling: 0.0,
            domain_risk: 0.0,
            duplication: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_tdg_calculation() {
        let calculator = TDGCalculator::new();
        let temp_dir = TempDir::new().unwrap();

        let test_file = temp_dir.path().join("test.rs");
        tokio::fs::write(
            &test_file,
            r#"
            fn complex_function(x: i32) -> i32 {
                if x > 0 {
                    if x > 10 {
                        if x > 20 {
                            x * 2
                        } else {
                            x + 10
                        }
                    } else {
                        x + 5
                    }
                } else {
                    0
                }
            }
        "#,
        )
        .await
        .unwrap();

        let score = calculator.calculate_file(&test_file).await.unwrap();

        assert!(score.value > 0.0);
        assert!(score.value <= 5.0);
        assert!(score.components.complexity > 0.0);
    }

    #[test]
    fn test_tdg_distribution() {
        let calculator = TDGCalculator::new();

        let scores = vec![
            TDGScore {
                value: 0.5,
                components: TDGComponents::default(),
                severity: TDGSeverity::Normal,
                percentile: 0.0,
                confidence: 1.0,
            },
            TDGScore {
                value: 1.8,
                components: TDGComponents::default(),
                severity: TDGSeverity::Warning,
                percentile: 0.0,
                confidence: 1.0,
            },
            TDGScore {
                value: 3.2,
                components: TDGComponents::default(),
                severity: TDGSeverity::Critical,
                percentile: 0.0,
                confidence: 1.0,
            },
        ];

        let distribution = calculator.calculate_distribution(&scores);

        assert_eq!(distribution.total_files, 3);
        assert!(!distribution.buckets.is_empty());

        let total_percentage: f64 = distribution.buckets.iter().map(|b| b.percentage).sum();
        assert!((total_percentage - 100.0).abs() < 0.01);
    }
}
