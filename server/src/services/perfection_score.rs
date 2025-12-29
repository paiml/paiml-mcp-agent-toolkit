//! Perfection Score Service (master-plan-pmat-work-system.md)
//!
//! Aggregates 8 quality metrics into a unified 200-point score:
//! - TDG (40 pts)
//! - Repo Score (30 pts)
//! - Rust Project Score (30 pts)
//! - Popper Score (25 pts)
//! - Test Coverage (25 pts)
//! - Mutation Score (20 pts)
//! - Documentation (15 pts)
//! - Performance (15 pts)

use crate::models::tdg::TDGConfig;
use crate::services::popper_score::orchestrator::PopperOrchestrator;
use crate::services::repo_score::aggregator::ScoreAggregator;
use crate::services::repo_score::scorers::ScorerConfig;
use crate::services::rust_project_score::models::ScoringMode;
use crate::services::rust_project_score::orchestrator::RustProjectScoreOrchestrator;
use crate::services::tdg_calculator::TDGCalculator;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Maximum possible perfection score
pub const MAX_PERFECTION_SCORE: u16 = 200;

/// Category weights for the 200-point scale
#[derive(Debug, Clone, Copy)]
pub struct CategoryWeights {
    pub tdg: u16,           // 40 pts (20%)
    pub repo_score: u16,    // 30 pts (15%)
    pub rust_score: u16,    // 30 pts (15%)
    pub popper_score: u16,  // 25 pts (12.5%)
    pub test_coverage: u16, // 25 pts (12.5%)
    pub mutation: u16,      // 20 pts (10%)
    pub documentation: u16, // 15 pts (7.5%)
    pub performance: u16,   // 15 pts (7.5%)
}

impl Default for CategoryWeights {
    fn default() -> Self {
        Self {
            tdg: 40,
            repo_score: 30,
            rust_score: 30,
            popper_score: 25,
            test_coverage: 25,
            mutation: 20,
            documentation: 15,
            performance: 15,
        }
    }
}

/// Individual category score (0-100 normalized to category weight)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryScore {
    pub name: String,
    pub raw_score: f64,     // Original 0-100 score
    pub max_points: u16,    // Max points for this category
    pub earned_points: f64, // Normalized to category weight
    pub grade: String,      // Letter grade for this category
    pub details: Option<String>,
}

impl CategoryScore {
    pub fn new(name: &str, raw_score: f64, max_points: u16) -> Self {
        let earned_points = (raw_score / 100.0) * f64::from(max_points);
        let grade = Self::calculate_grade(raw_score);
        Self {
            name: name.to_string(),
            raw_score,
            max_points,
            earned_points,
            grade,
            details: None,
        }
    }

    pub fn with_details(mut self, details: &str) -> Self {
        self.details = Some(details.to_string());
        self
    }

    fn calculate_grade(score: f64) -> String {
        // Standard academic grading scale (F-A)
        match score as u8 {
            97..=100 => "A+".to_string(),
            93..=96 => "A".to_string(),
            90..=92 => "A-".to_string(),
            87..=89 => "B+".to_string(),
            83..=86 => "B".to_string(),
            80..=82 => "B-".to_string(),
            77..=79 => "C+".to_string(),
            73..=76 => "C".to_string(),
            70..=72 => "C-".to_string(),
            67..=69 => "D+".to_string(),
            63..=66 => "D".to_string(),
            60..=62 => "D-".to_string(),
            _ => "F".to_string(),
        }
    }
}

/// Complete perfection score result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfectionScoreResult {
    pub total_score: f64,
    pub max_score: u16,
    pub grade: String,
    pub categories: Vec<CategoryScore>,
    pub recommendations: Vec<String>,
    pub target_gap: Option<f64>,
}

impl PerfectionScoreResult {
    pub fn new(categories: Vec<CategoryScore>) -> Self {
        let total_score: f64 = categories.iter().map(|c| c.earned_points).sum();
        let max_score = MAX_PERFECTION_SCORE;
        let grade = Self::calculate_overall_grade(total_score);
        let recommendations = Self::generate_recommendations(&categories);

        Self {
            total_score,
            max_score,
            grade,
            categories,
            recommendations,
            target_gap: None,
        }
    }

    pub fn with_target(mut self, target: u16) -> Self {
        self.target_gap = Some(f64::from(target) - self.total_score);
        self
    }

    fn calculate_overall_grade(score: f64) -> String {
        // Standard academic grading scale (F-A) for 200-point scale
        match score as u16 {
            194..=200 => "A+".to_string(),
            186..=193 => "A".to_string(),
            180..=185 => "A-".to_string(),
            174..=179 => "B+".to_string(),
            166..=173 => "B".to_string(),
            160..=165 => "B-".to_string(),
            154..=159 => "C+".to_string(),
            146..=153 => "C".to_string(),
            140..=145 => "C-".to_string(),
            134..=139 => "D+".to_string(),
            126..=133 => "D".to_string(),
            120..=125 => "D-".to_string(),
            _ => "F".to_string(),
        }
    }

    fn generate_recommendations(categories: &[CategoryScore]) -> Vec<String> {
        let mut recs = Vec::new();

        for cat in categories {
            let percentage = (cat.earned_points / f64::from(cat.max_points)) * 100.0;
            if percentage < 60.0 {
                recs.push(format!(
                    "🔴 {} is critical ({:.0}%) - prioritize improvement",
                    cat.name, percentage
                ));
            } else if percentage < 80.0 {
                recs.push(format!(
                    "🟡 {} needs attention ({:.0}%)",
                    cat.name, percentage
                ));
            }
        }

        if recs.is_empty() {
            recs.push("✅ All categories are healthy!".to_string());
        }

        recs
    }
}

/// Perfection Score Calculator
pub struct PerfectionScoreCalculator {
    weights: CategoryWeights,
    fast_mode: bool,
}

impl Default for PerfectionScoreCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl PerfectionScoreCalculator {
    pub fn new() -> Self {
        Self {
            weights: CategoryWeights::default(),
            fast_mode: false,
        }
    }

    pub fn fast_mode(mut self, fast: bool) -> Self {
        self.fast_mode = fast;
        self
    }

    /// Calculate perfection score for a project
    pub async fn calculate(&self, project_path: &Path) -> anyhow::Result<PerfectionScoreResult> {
        let mut categories = Vec::new();

        // 1. TDG Score (40 pts)
        let tdg_score = self.get_tdg_score(project_path).await;
        categories.push(CategoryScore::new(
            "Technical Debt Grade",
            tdg_score,
            self.weights.tdg,
        ));

        // 2. Repo Score (30 pts)
        let repo_score = self.get_repo_score(project_path).await;
        categories.push(CategoryScore::new(
            "Repository Health",
            repo_score,
            self.weights.repo_score,
        ));

        // 3. Rust Project Score (30 pts)
        let rust_score = self.get_rust_project_score(project_path).await;
        categories.push(CategoryScore::new(
            "Rust Project Quality",
            rust_score,
            self.weights.rust_score,
        ));

        // 4. Popper Score (25 pts)
        let popper_score = self.get_popper_score(project_path).await;
        categories.push(CategoryScore::new(
            "Popperian Falsifiability",
            popper_score,
            self.weights.popper_score,
        ));

        // 5. Test Coverage (25 pts)
        let coverage_score = self.get_coverage_score(project_path).await;
        categories.push(CategoryScore::new(
            "Test Coverage",
            coverage_score,
            self.weights.test_coverage,
        ));

        // 6. Mutation Score (20 pts) - Skip in fast mode
        let mutation_score = if self.fast_mode {
            50.0 // Default credit in fast mode
        } else {
            self.get_mutation_score(project_path).await
        };
        categories.push(
            CategoryScore::new("Mutation Testing", mutation_score, self.weights.mutation)
                .with_details(if self.fast_mode {
                    "Skipped (fast mode)"
                } else {
                    ""
                }),
        );

        // 7. Documentation (15 pts)
        let doc_score = self.get_documentation_score(project_path).await;
        categories.push(CategoryScore::new(
            "Documentation",
            doc_score,
            self.weights.documentation,
        ));

        // 8. Performance (15 pts)
        let perf_score = self.get_performance_score(project_path).await;
        categories.push(CategoryScore::new(
            "Performance",
            perf_score,
            self.weights.performance,
        ));

        Ok(PerfectionScoreResult::new(categories))
    }

    async fn get_tdg_score(&self, project_path: &Path) -> f64 {
        // TDG score: 0-5 scale where 0 = excellent, 5 = critical
        // Convert to 0-100 scale where 100 = excellent
        let config = TDGConfig::default();
        let calculator = TDGCalculator::with_config(config);

        match calculator.analyze_directory(project_path).await {
            Ok(summary) => {
                // Convert TDG scale (0-5, lower is better) to 0-100 (higher is better)
                // TDG 0 -> 100, TDG 2.5 -> 50, TDG 5 -> 0
                let normalized = 100.0 - (summary.average_tdg * 20.0);
                normalized.clamp(0.0, 100.0)
            }
            Err(e) => {
                eprintln!("⚠️  TDG calculation failed: {}", e);
                // Fall back to repo score as proxy
                self.get_repo_score(project_path).await
            }
        }
    }

    async fn get_repo_score(&self, project_path: &Path) -> f64 {
        // Repo Score: 0-100 scale
        let aggregator = ScoreAggregator::new();
        let config = ScorerConfig {
            verbose: false,
            timeout_seconds: 60,
            skip_slow_checks: self.fast_mode,
            deep: !self.fast_mode,
        };

        match aggregator.aggregate(project_path, &config).await {
            Ok(score) => score.total_score,
            Err(e) => {
                eprintln!("⚠️  Repo score failed: {}", e);
                50.0 // Default on error
            }
        }
    }

    async fn get_rust_project_score(&self, project_path: &Path) -> f64 {
        // Rust Project Score: 0-134 scale, normalize to 0-100
        let orchestrator = RustProjectScoreOrchestrator::new();
        let mode = if self.fast_mode {
            ScoringMode::Quick
        } else {
            ScoringMode::Fast
        };

        match orchestrator.score_with_mode(project_path, mode) {
            Ok(score) => {
                // Normalize 134-point scale to 100-point scale
                (score.total_earned / 134.0) * 100.0
            }
            Err(e) => {
                eprintln!("⚠️  Rust project score failed: {}", e);
                50.0 // Default on error
            }
        }
    }

    async fn get_popper_score(&self, project_path: &Path) -> f64 {
        // Popper Score: 0-100 scale
        let orchestrator = PopperOrchestrator::new();

        match orchestrator.score(project_path) {
            Ok(result) => result.normalized_score,
            Err(e) => {
                eprintln!("⚠️  Popper score failed: {}", e);
                50.0 // Default on error
            }
        }
    }

    async fn get_coverage_score(&self, project_path: &Path) -> f64 {
        // Coverage: Check .pmat-metrics cache or run estimation
        // Look for cached coverage data in multiple locations (workspace-aware)
        let cache_paths = [
            project_path.join(".pmat-metrics").join("coverage.json"),
            project_path.join("server/.pmat-metrics/coverage.json"),
        ];

        for metrics_file in &cache_paths {
            if metrics_file.exists() {
                if let Ok(content) = std::fs::read_to_string(metrics_file) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(coverage) = json.get("coverage").and_then(|v| v.as_f64()) {
                            return coverage;
                        }
                    }
                }
            }
        }

        // Count #[test] and #[cfg(test)] in Rust files for better heuristic
        let mut test_count = 0;
        let mut source_count = 0;

        for entry in walkdir::WalkDir::new(project_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
        {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                source_count += 1;
                test_count += content.matches("#[test]").count();
                test_count += content.matches("#[tokio::test]").count();
            }
        }

        // Better heuristic: ratio of tests to source files + absolute test count
        if source_count > 0 {
            let test_density = (test_count as f64 / source_count as f64).min(5.0);
            let base_score = 50.0 + (test_count as f64 * 0.1).min(25.0);
            return (base_score + test_density * 5.0).min(95.0);
        }

        // Full mode: would run cargo llvm-cov but that's expensive
        // Default to moderate estimate
        70.0
    }

    async fn get_mutation_score(&self, project_path: &Path) -> f64 {
        // Check for mutation testing setup indicators
        let mut score: f64 = 50.0; // Base score

        // Check for mutants.toml (cargo-mutants config)
        let has_mutants_config = project_path.join("mutants.toml").exists()
            || project_path.join("server/mutants.toml").exists();
        if has_mutants_config {
            score += 20.0;
        }

        // Check for .mutants/ directory (mutation test results)
        let has_mutants_results = project_path.join(".mutants").exists()
            || project_path.join("server/.mutants").exists();
        if has_mutants_results {
            score += 20.0;
        }

        // Check for cargo-mutants in dev-dependencies
        let has_mutants_dep = walkdir::WalkDir::new(project_path)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name() == "Cargo.toml")
            .any(|e| {
                std::fs::read_to_string(e.path())
                    .map(|s| s.contains("cargo-mutants") || s.contains("mutants"))
                    .unwrap_or(false)
            });
        if has_mutants_dep {
            score += 10.0;
        }

        score.min(100.0)
    }

    async fn get_documentation_score(&self, project_path: &Path) -> f64 {
        // Check for common documentation files
        let has_readme =
            project_path.join("README.md").exists() || project_path.join("readme.md").exists();
        let has_changelog = project_path.join("CHANGELOG.md").exists();
        let has_docs_dir = project_path.join("docs").exists();
        let has_contributing = project_path.join("CONTRIBUTING.md").exists();

        let mut score: f64 = 0.0;
        if has_readme {
            score += 40.0;
        }
        if has_changelog {
            score += 20.0;
        }
        if has_docs_dir {
            score += 25.0;
        }
        if has_contributing {
            score += 15.0;
        }

        score.min(100.0)
    }

    async fn get_performance_score(&self, project_path: &Path) -> f64 {
        // Check for performance-related files (handle both standalone and workspace projects)
        let has_benches = project_path.join("benches").exists()
            || project_path.join("server/benches").exists()
            || project_path.join("crates").exists()
                && walkdir::WalkDir::new(project_path.join("crates"))
                    .max_depth(2)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .any(|e| e.path().ends_with("benches") && e.path().is_dir());

        // Check for criterion in any Cargo.toml (workspace-aware)
        let has_criterion = walkdir::WalkDir::new(project_path)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name() == "Cargo.toml")
            .any(|e| {
                std::fs::read_to_string(e.path())
                    .map(|s| s.contains("criterion"))
                    .unwrap_or(false)
            });

        let mut score: f64 = 50.0; // Base score
        if has_benches {
            score += 30.0;
        }
        if has_criterion {
            score += 20.0;
        }

        score.min(100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_weights_sum_to_200() {
        let weights = CategoryWeights::default();
        let sum = weights.tdg
            + weights.repo_score
            + weights.rust_score
            + weights.popper_score
            + weights.test_coverage
            + weights.mutation
            + weights.documentation
            + weights.performance;
        assert_eq!(sum, MAX_PERFECTION_SCORE);
    }

    #[test]
    fn test_category_score_calculation() {
        let score = CategoryScore::new("Test", 80.0, 40);
        assert_eq!(score.earned_points, 32.0);
        assert_eq!(score.grade, "B-"); // 80 is in B- range (80-82)
    }

    #[test]
    fn test_overall_grade_thresholds() {
        // Standard academic grading scale (F-A) for 200-point scale
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(198.0), "A+"); // 194-200
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(190.0), "A");  // 186-193
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(182.0), "A-"); // 180-185
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(176.0), "B+"); // 174-179
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(170.0), "B");  // 166-173
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(162.0), "B-"); // 160-165
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(156.0), "C+"); // 154-159
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(150.0), "C");  // 146-153
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(142.0), "C-"); // 140-145
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(136.0), "D+"); // 134-139
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(130.0), "D");  // 126-133
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(122.0), "D-"); // 120-125
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(100.0), "F");  // 0-119
    }

    #[test]
    fn test_perfection_score_result() {
        let categories = vec![
            CategoryScore::new("TDG", 80.0, 40),         // 32 pts
            CategoryScore::new("Repo", 70.0, 30),        // 21 pts
            CategoryScore::new("Rust", 75.0, 30),        // 22.5 pts
            CategoryScore::new("Popper", 65.0, 25),      // 16.25 pts
            CategoryScore::new("Coverage", 90.0, 25),    // 22.5 pts
            CategoryScore::new("Mutation", 60.0, 20),    // 12 pts
            CategoryScore::new("Docs", 70.0, 15),        // 10.5 pts
            CategoryScore::new("Performance", 85.0, 15), // 12.75 pts
        ];
        let result = PerfectionScoreResult::new(categories);

        // Total: 32 + 21 + 22.5 + 16.25 + 22.5 + 12 + 10.5 + 12.75 = 149.5
        assert!((result.total_score - 149.5).abs() < 0.01);
        assert_eq!(result.grade, "C"); // 149.5 is in C range (146-153)
    }

    #[tokio::test]
    async fn test_calculator_fast_mode() {
        let calc = PerfectionScoreCalculator::new().fast_mode(true);
        let result = calc.calculate(Path::new(".")).await.unwrap();

        // Should have all 8 categories
        assert_eq!(result.categories.len(), 8);
        // Score should be in valid range
        assert!(result.total_score >= 0.0 && result.total_score <= 200.0);
    }
}
