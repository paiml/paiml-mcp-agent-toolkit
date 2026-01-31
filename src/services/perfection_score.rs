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
//!
//! PMAT-454: All output normalized to 0-100 scale

use crate::models::tdg::TDGConfig;
use crate::services::normalized_score::NormalizedScore;
use crate::services::popper_score::orchestrator::PopperOrchestrator;
use crate::services::repo_score::aggregator::ScoreAggregator;
use crate::services::repo_score::scorers::ScorerConfig;
use crate::services::rust_project_score::models::ScoringMode;
use crate::services::rust_project_score::orchestrator::RustProjectScoreOrchestrator;
use crate::services::tdg_calculator::TDGCalculator;
use serde::{Deserialize, Serialize};
use std::fmt;
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
        // PMAT-454: Use normalized percentage (0-100) for grading
        let normalized = (score / f64::from(MAX_PERFECTION_SCORE)) * 100.0;
        match normalized as u16 {
            95..=100 => "A+".to_string(),
            90..=94 => "A".to_string(),
            85..=89 => "A-".to_string(),
            80..=84 => "B+".to_string(),
            70..=79 => "B".to_string(),
            60..=69 => "C".to_string(),
            50..=59 => "D".to_string(),
            _ => "F".to_string(),
        }
    }
}

impl NormalizedScore for PerfectionScoreResult {
    fn raw(&self) -> f64 {
        self.total_score
    }

    fn max_raw(&self) -> f64 {
        f64::from(MAX_PERFECTION_SCORE)
    }
}

impl fmt::Display for PerfectionScoreResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Perfection Score: {:.1}/100 ({}) [raw: {:.0}/{}]",
            self.normalized(),
            self.grade,
            self.total_score,
            MAX_PERFECTION_SCORE
        )
    }
}

impl PerfectionScoreResult {
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
        let has_mutants_results =
            project_path.join(".mutants").exists() || project_path.join("server/.mutants").exists();
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
        // Normalized percentage grading (score/200 * 100)
        // 190/200 = 95% → A+, 180/200 = 90% → A, etc.
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(198.0), "A+"); // 99%
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(190.0), "A+"); // 95%
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(184.0), "A");  // 92%
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(174.0), "A-"); // 87%
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(164.0), "B+"); // 82%
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(150.0), "B");  // 75%
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(130.0), "C");  // 65%
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(110.0), "D");  // 55%
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(80.0), "F");   // 40%
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
        assert_eq!(result.grade, "B"); // 149.5/200 = 74.75% → B range (70-79%)
    }

    #[tokio::test]
    #[ignore] // Times out in coverage builds (>120s)
    async fn test_calculator_fast_mode() {
        let calc = PerfectionScoreCalculator::new().fast_mode(true);
        let result = calc.calculate(Path::new(".")).await.unwrap();

        // Should have all 8 categories
        assert_eq!(result.categories.len(), 8);
        // Score should be in valid range
        assert!(result.total_score >= 0.0 && result.total_score <= 200.0);
    }
}

/// EXTREME TDD coverage tests for perfection_score module
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use proptest::prelude::*;
    use std::fs;
    use tempfile::TempDir;

    // ============================================================================
    // Test Fixture Helpers
    // ============================================================================

    /// Create a test fixture with configurable project structure
    fn create_test_project(
        readme: bool,
        changelog: bool,
        docs: bool,
        contributing: bool,
        benches: bool,
        mutants_toml: bool,
        mutants_dir: bool,
    ) -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        if readme {
            fs::write(root.join("README.md"), "# Test Project").unwrap();
        }
        if changelog {
            fs::write(root.join("CHANGELOG.md"), "# Changelog").unwrap();
        }
        if docs {
            fs::create_dir(root.join("docs")).unwrap();
        }
        if contributing {
            fs::write(root.join("CONTRIBUTING.md"), "# Contributing").unwrap();
        }
        if benches {
            fs::create_dir(root.join("benches")).unwrap();
        }
        if mutants_toml {
            fs::write(root.join("mutants.toml"), "[mutants]").unwrap();
        }
        if mutants_dir {
            fs::create_dir(root.join(".mutants")).unwrap();
        }

        temp_dir
    }

    /// Create a Rust project fixture with test files
    fn create_rust_project_fixture(test_count: usize, source_files: usize) -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create src directory
        fs::create_dir(root.join("src")).unwrap();

        // Create source files with tests
        for i in 0..source_files {
            let mut content = format!("// Source file {}\n", i);
            let tests_in_file = if i < test_count { 1 } else { 0 };
            for j in 0..tests_in_file {
                content.push_str(&format!("\n#[test]\nfn test_{}_{} () {{}}\n", i, j));
            }
            fs::write(root.join("src").join(format!("mod_{}.rs", i)), content).unwrap();
        }

        // Create Cargo.toml
        fs::write(
            root.join("Cargo.toml"),
            r#"[package]
name = "test_project"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();

        temp_dir
    }

    /// Create coverage metrics cache file
    fn create_coverage_cache(temp_dir: &TempDir, coverage: f64) {
        let metrics_dir = temp_dir.path().join(".pmat-metrics");
        fs::create_dir_all(&metrics_dir).unwrap();
        let cache_content = format!(r#"{{"coverage": {}}}"#, coverage);
        fs::write(metrics_dir.join("coverage.json"), cache_content).unwrap();
    }

    // ============================================================================
    // CategoryWeights Tests
    // ============================================================================

    #[test]
    fn test_category_weights_default_values() {
        let weights = CategoryWeights::default();
        assert_eq!(weights.tdg, 40);
        assert_eq!(weights.repo_score, 30);
        assert_eq!(weights.rust_score, 30);
        assert_eq!(weights.popper_score, 25);
        assert_eq!(weights.test_coverage, 25);
        assert_eq!(weights.mutation, 20);
        assert_eq!(weights.documentation, 15);
        assert_eq!(weights.performance, 15);
    }

    #[test]
    fn test_category_weights_clone() {
        let weights = CategoryWeights::default();
        let cloned = weights;
        assert_eq!(weights.tdg, cloned.tdg);
        assert_eq!(weights.repo_score, cloned.repo_score);
    }

    #[test]
    fn test_category_weights_debug() {
        let weights = CategoryWeights::default();
        let debug_str = format!("{:?}", weights);
        assert!(debug_str.contains("CategoryWeights"));
        assert!(debug_str.contains("40"));
    }

    // ============================================================================
    // CategoryScore Tests
    // ============================================================================

    #[test]
    fn test_category_score_new_zero_score() {
        let score = CategoryScore::new("Test", 0.0, 40);
        assert_eq!(score.name, "Test");
        assert_eq!(score.raw_score, 0.0);
        assert_eq!(score.max_points, 40);
        assert_eq!(score.earned_points, 0.0);
        assert_eq!(score.grade, "F");
        assert!(score.details.is_none());
    }

    #[test]
    fn test_category_score_new_fifty_percent() {
        let score = CategoryScore::new("Test", 50.0, 40);
        assert_eq!(score.earned_points, 20.0);
        assert_eq!(score.grade, "F");
    }

    #[test]
    fn test_category_score_new_hundred_percent() {
        let score = CategoryScore::new("Test", 100.0, 40);
        assert_eq!(score.earned_points, 40.0);
        assert_eq!(score.grade, "A+");
    }

    #[test]
    fn test_category_score_with_details() {
        let score = CategoryScore::new("Test", 75.0, 30).with_details("Some details");
        assert_eq!(score.details, Some("Some details".to_string()));
    }

    #[test]
    fn test_category_score_grade_a_plus() {
        let score = CategoryScore::new("Test", 97.0, 10);
        assert_eq!(score.grade, "A+");
        let score = CategoryScore::new("Test", 100.0, 10);
        assert_eq!(score.grade, "A+");
    }

    #[test]
    fn test_category_score_grade_a() {
        let score = CategoryScore::new("Test", 93.0, 10);
        assert_eq!(score.grade, "A");
        let score = CategoryScore::new("Test", 96.0, 10);
        assert_eq!(score.grade, "A");
    }

    #[test]
    fn test_category_score_grade_a_minus() {
        let score = CategoryScore::new("Test", 90.0, 10);
        assert_eq!(score.grade, "A-");
        let score = CategoryScore::new("Test", 92.0, 10);
        assert_eq!(score.grade, "A-");
    }

    #[test]
    fn test_category_score_grade_b_plus() {
        let score = CategoryScore::new("Test", 87.0, 10);
        assert_eq!(score.grade, "B+");
        let score = CategoryScore::new("Test", 89.0, 10);
        assert_eq!(score.grade, "B+");
    }

    #[test]
    fn test_category_score_grade_b() {
        let score = CategoryScore::new("Test", 83.0, 10);
        assert_eq!(score.grade, "B");
        let score = CategoryScore::new("Test", 86.0, 10);
        assert_eq!(score.grade, "B");
    }

    #[test]
    fn test_category_score_grade_b_minus() {
        let score = CategoryScore::new("Test", 80.0, 10);
        assert_eq!(score.grade, "B-");
        let score = CategoryScore::new("Test", 82.0, 10);
        assert_eq!(score.grade, "B-");
    }

    #[test]
    fn test_category_score_grade_c_plus() {
        let score = CategoryScore::new("Test", 77.0, 10);
        assert_eq!(score.grade, "C+");
        let score = CategoryScore::new("Test", 79.0, 10);
        assert_eq!(score.grade, "C+");
    }

    #[test]
    fn test_category_score_grade_c() {
        let score = CategoryScore::new("Test", 73.0, 10);
        assert_eq!(score.grade, "C");
        let score = CategoryScore::new("Test", 76.0, 10);
        assert_eq!(score.grade, "C");
    }

    #[test]
    fn test_category_score_grade_c_minus() {
        let score = CategoryScore::new("Test", 70.0, 10);
        assert_eq!(score.grade, "C-");
        let score = CategoryScore::new("Test", 72.0, 10);
        assert_eq!(score.grade, "C-");
    }

    #[test]
    fn test_category_score_grade_d_plus() {
        let score = CategoryScore::new("Test", 67.0, 10);
        assert_eq!(score.grade, "D+");
        let score = CategoryScore::new("Test", 69.0, 10);
        assert_eq!(score.grade, "D+");
    }

    #[test]
    fn test_category_score_grade_d() {
        let score = CategoryScore::new("Test", 63.0, 10);
        assert_eq!(score.grade, "D");
        let score = CategoryScore::new("Test", 66.0, 10);
        assert_eq!(score.grade, "D");
    }

    #[test]
    fn test_category_score_grade_d_minus() {
        let score = CategoryScore::new("Test", 60.0, 10);
        assert_eq!(score.grade, "D-");
        let score = CategoryScore::new("Test", 62.0, 10);
        assert_eq!(score.grade, "D-");
    }

    #[test]
    fn test_category_score_grade_f() {
        let score = CategoryScore::new("Test", 59.0, 10);
        assert_eq!(score.grade, "F");
        let score = CategoryScore::new("Test", 0.0, 10);
        assert_eq!(score.grade, "F");
    }

    #[test]
    fn test_category_score_serialization() {
        let score = CategoryScore::new("Test", 85.0, 25);
        let json = serde_json::to_string(&score).unwrap();
        assert!(json.contains("\"name\":\"Test\""));
        assert!(json.contains("\"raw_score\":85.0"));
        assert!(json.contains("\"max_points\":25"));
    }

    #[test]
    fn test_category_score_deserialization() {
        let json = r#"{"name":"Test","raw_score":85.0,"max_points":25,"earned_points":21.25,"grade":"B","details":null}"#;
        let score: CategoryScore = serde_json::from_str(json).unwrap();
        assert_eq!(score.name, "Test");
        assert_eq!(score.raw_score, 85.0);
        assert_eq!(score.max_points, 25);
    }

    // ============================================================================
    // PerfectionScoreResult Tests
    // ============================================================================

    #[test]
    fn test_perfection_score_result_empty_categories() {
        let result = PerfectionScoreResult::new(vec![]);
        assert_eq!(result.total_score, 0.0);
        assert_eq!(result.max_score, MAX_PERFECTION_SCORE);
        assert_eq!(result.grade, "F");
        assert_eq!(result.recommendations.len(), 1);
        assert!(result.recommendations[0].contains("All categories are healthy"));
    }

    #[test]
    fn test_perfection_score_result_perfect_score() {
        let categories = vec![
            CategoryScore::new("TDG", 100.0, 40),
            CategoryScore::new("Repo", 100.0, 30),
            CategoryScore::new("Rust", 100.0, 30),
            CategoryScore::new("Popper", 100.0, 25),
            CategoryScore::new("Coverage", 100.0, 25),
            CategoryScore::new("Mutation", 100.0, 20),
            CategoryScore::new("Docs", 100.0, 15),
            CategoryScore::new("Performance", 100.0, 15),
        ];
        let result = PerfectionScoreResult::new(categories);
        assert_eq!(result.total_score, 200.0);
        assert_eq!(result.grade, "A+");
    }

    #[test]
    fn test_perfection_score_result_with_target() {
        let categories = vec![CategoryScore::new("TDG", 80.0, 40)];
        let result = PerfectionScoreResult::new(categories).with_target(100);
        assert!(result.target_gap.is_some());
        assert_eq!(result.target_gap.unwrap(), 100.0 - 32.0);
    }

    #[test]
    fn test_perfection_score_result_recommendations_critical() {
        let categories = vec![
            CategoryScore::new("TDG", 50.0, 40), // 50% - critical
        ];
        let result = PerfectionScoreResult::new(categories);
        assert!(result
            .recommendations
            .iter()
            .any(|r| r.contains("critical")));
    }

    #[test]
    fn test_perfection_score_result_recommendations_needs_attention() {
        let categories = vec![
            CategoryScore::new("TDG", 70.0, 40), // 70% - needs attention
        ];
        let result = PerfectionScoreResult::new(categories);
        assert!(result
            .recommendations
            .iter()
            .any(|r| r.contains("needs attention")));
    }

    #[test]
    fn test_perfection_score_result_recommendations_healthy() {
        let categories = vec![
            CategoryScore::new("TDG", 90.0, 40), // 90% - healthy
        ];
        let result = PerfectionScoreResult::new(categories);
        assert!(result.recommendations.iter().any(|r| r.contains("healthy")));
    }

    #[test]
    fn test_overall_grade_boundary_a_plus() {
        // 95%+ = A+: 190/200=95%, 200/200=100%
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(190.0), "A+");
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(200.0), "A+");
    }

    #[test]
    fn test_overall_grade_boundary_a() {
        // 90-94% = A: 180/200=90%, 188/200=94%
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(180.0), "A");
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(188.0), "A");
    }

    #[test]
    fn test_overall_grade_boundary_f() {
        // <50% = F: 0/200=0%, 98/200=49%
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(0.0), "F");
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(98.0), "F");
    }

    #[test]
    fn test_perfection_score_result_serialization() {
        let categories = vec![CategoryScore::new("TDG", 80.0, 40)];
        let result = PerfectionScoreResult::new(categories);
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"total_score\""));
        assert!(json.contains("\"max_score\""));
        assert!(json.contains("\"grade\""));
    }

    // ============================================================================
    // PerfectionScoreCalculator Tests
    // ============================================================================

    #[test]
    fn test_calculator_new() {
        let calc = PerfectionScoreCalculator::new();
        assert!(!calc.fast_mode);
        assert_eq!(calc.weights.tdg, 40);
    }

    #[test]
    fn test_calculator_default() {
        let calc = PerfectionScoreCalculator::default();
        assert!(!calc.fast_mode);
    }

    #[test]
    fn test_calculator_fast_mode_setter() {
        let calc = PerfectionScoreCalculator::new().fast_mode(true);
        assert!(calc.fast_mode);

        let calc = PerfectionScoreCalculator::new().fast_mode(false);
        assert!(!calc.fast_mode);
    }

    #[tokio::test]
    async fn test_get_documentation_score_all_docs() {
        let temp_dir = create_test_project(true, true, true, true, false, false, false);
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_documentation_score(temp_dir.path()).await;
        assert_eq!(score, 100.0); // 40 + 20 + 25 + 15 = 100
    }

    #[tokio::test]
    async fn test_get_documentation_score_readme_only() {
        let temp_dir = create_test_project(true, false, false, false, false, false, false);
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_documentation_score(temp_dir.path()).await;
        assert_eq!(score, 40.0);
    }

    #[tokio::test]
    async fn test_get_documentation_score_no_docs() {
        let temp_dir = create_test_project(false, false, false, false, false, false, false);
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_documentation_score(temp_dir.path()).await;
        assert_eq!(score, 0.0);
    }

    #[tokio::test]
    async fn test_get_documentation_score_lowercase_readme() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("readme.md"), "# Test").unwrap();
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_documentation_score(temp_dir.path()).await;
        assert_eq!(score, 40.0);
    }

    #[tokio::test]
    async fn test_get_performance_score_with_benches() {
        let temp_dir = create_test_project(false, false, false, false, true, false, false);
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_performance_score(temp_dir.path()).await;
        assert_eq!(score, 80.0); // 50 base + 30 for benches
    }

    #[tokio::test]
    async fn test_get_performance_score_with_criterion() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"[dev-dependencies]
criterion = "0.5"
"#,
        )
        .unwrap();
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_performance_score(temp_dir.path()).await;
        assert_eq!(score, 70.0); // 50 base + 20 for criterion
    }

    #[tokio::test]
    async fn test_get_performance_score_with_both() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir(temp_dir.path().join("benches")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"[dev-dependencies]
criterion = "0.5"
"#,
        )
        .unwrap();
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_performance_score(temp_dir.path()).await;
        assert_eq!(score, 100.0); // 50 base + 30 benches + 20 criterion = 100 (capped)
    }

    #[tokio::test]
    async fn test_get_mutation_score_with_mutants_config() {
        let temp_dir = create_test_project(false, false, false, false, false, true, false);
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_mutation_score(temp_dir.path()).await;
        assert_eq!(score, 70.0); // 50 base + 20 for config
    }

    #[tokio::test]
    async fn test_get_mutation_score_with_mutants_dir() {
        let temp_dir = create_test_project(false, false, false, false, false, false, true);
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_mutation_score(temp_dir.path()).await;
        assert_eq!(score, 70.0); // 50 base + 20 for results dir
    }

    #[tokio::test]
    async fn test_get_mutation_score_with_all_indicators() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("mutants.toml"), "[mutants]").unwrap();
        fs::create_dir(temp_dir.path().join(".mutants")).unwrap();
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"[dev-dependencies]
cargo-mutants = "1.0"
"#,
        )
        .unwrap();
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_mutation_score(temp_dir.path()).await;
        assert_eq!(score, 100.0); // 50 + 20 + 20 + 10 = 100 (capped)
    }

    #[tokio::test]
    async fn test_get_coverage_score_from_cache() {
        let temp_dir = TempDir::new().unwrap();
        create_coverage_cache(&temp_dir, 85.5);
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_coverage_score(temp_dir.path()).await;
        assert_eq!(score, 85.5);
    }

    #[tokio::test]
    async fn test_get_coverage_score_heuristic() {
        let temp_dir = create_rust_project_fixture(10, 5);
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_coverage_score(temp_dir.path()).await;
        // Score based on test density heuristic
        assert!(score >= 50.0 && score <= 95.0);
    }

    #[tokio::test]
    async fn test_get_coverage_score_empty_project() {
        let temp_dir = TempDir::new().unwrap();
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_coverage_score(temp_dir.path()).await;
        assert_eq!(score, 70.0); // Default moderate estimate
    }

    // ============================================================================
    // Property-Based Tests
    // ============================================================================

    proptest! {
        /// Property: Earned points are always proportional to raw score and max points
        #[test]
        fn prop_earned_points_proportional(raw_score in 0.0f64..=100.0, max_points in 1u16..=100) {
            let score = CategoryScore::new("Test", raw_score, max_points);
            let expected = (raw_score / 100.0) * f64::from(max_points);
            prop_assert!((score.earned_points - expected).abs() < 0.001);
        }

        /// Property: Grade is always one of the valid grades
        #[test]
        fn prop_grade_is_valid(raw_score in 0.0f64..=100.0) {
            let score = CategoryScore::new("Test", raw_score, 40);
            let valid_grades = ["A+", "A", "A-", "B+", "B", "B-", "C+", "C", "C-", "D+", "D", "D-", "F"];
            prop_assert!(valid_grades.contains(&score.grade.as_str()));
        }

        /// Property: Total score is sum of earned points
        #[test]
        fn prop_total_score_is_sum(
            s1 in 0.0f64..=100.0,
            s2 in 0.0f64..=100.0,
            s3 in 0.0f64..=100.0
        ) {
            let categories = vec![
                CategoryScore::new("A", s1, 40),
                CategoryScore::new("B", s2, 30),
                CategoryScore::new("C", s3, 30),
            ];
            let result = PerfectionScoreResult::new(categories.clone());
            let expected: f64 = categories.iter().map(|c| c.earned_points).sum();
            prop_assert!((result.total_score - expected).abs() < 0.001);
        }

        /// Property: Score is always in valid range
        #[test]
        fn prop_score_in_valid_range(raw_score in 0.0f64..=100.0) {
            let score = CategoryScore::new("Test", raw_score, 40);
            prop_assert!(score.earned_points >= 0.0);
            prop_assert!(score.earned_points <= 40.0);
        }

        /// Property: Target gap calculation is correct
        #[test]
        fn prop_target_gap_correct(score in 0.0f64..=100.0, target in 0u16..=200) {
            let categories = vec![CategoryScore::new("Test", score, 100)];
            let result = PerfectionScoreResult::new(categories).with_target(target);
            let expected_gap = f64::from(target) - (score / 100.0) * 100.0;
            prop_assert!((result.target_gap.unwrap() - expected_gap).abs() < 0.001);
        }

        /// Property: Category weights always sum to MAX_PERFECTION_SCORE
        #[test]
        fn prop_weights_sum_to_max(_dummy in 0u8..1) {
            let weights = CategoryWeights::default();
            let sum = weights.tdg
                + weights.repo_score
                + weights.rust_score
                + weights.popper_score
                + weights.test_coverage
                + weights.mutation
                + weights.documentation
                + weights.performance;
            prop_assert_eq!(sum, MAX_PERFECTION_SCORE);
        }

        /// Property: Overall grade is monotonic with score
        #[test]
        fn prop_grade_monotonic(score1 in 0.0f64..=200.0, score2 in 0.0f64..=200.0) {
            let grade1 = PerfectionScoreResult::calculate_overall_grade(score1);
            let grade2 = PerfectionScoreResult::calculate_overall_grade(score2);

            // Define grade ordering
            fn grade_value(grade: &str) -> u8 {
                match grade {
                    "A+" => 12, "A" => 11, "A-" => 10,
                    "B+" => 9, "B" => 8, "B-" => 7,
                    "C+" => 6, "C" => 5, "C-" => 4,
                    "D+" => 3, "D" => 2, "D-" => 1,
                    "F" => 0,
                    _ => 0,
                }
            }

            if score1 > score2 + 1.0 {
                prop_assert!(grade_value(&grade1) >= grade_value(&grade2));
            }
        }

        /// Property: Category score with details preserves all fields
        #[test]
        fn prop_with_details_preserves_fields(
            raw_score in 0.0f64..=100.0,
            max_points in 1u16..=100
        ) {
            let score = CategoryScore::new("Test", raw_score, max_points);
            let score_with_details = score.clone().with_details("some details");

            prop_assert_eq!(score.name, score_with_details.name);
            prop_assert_eq!(score.raw_score, score_with_details.raw_score);
            prop_assert_eq!(score.max_points, score_with_details.max_points);
            prop_assert_eq!(score.earned_points, score_with_details.earned_points);
            prop_assert_eq!(score.grade, score_with_details.grade);
            prop_assert!(score_with_details.details.is_some());
        }

        /// Property: Recommendations always generated for low scores
        #[test]
        fn prop_recommendations_for_low_scores(raw_score in 0.0f64..50.0) {
            let categories = vec![CategoryScore::new("Critical", raw_score, 100)];
            let result = PerfectionScoreResult::new(categories);
            prop_assert!(result.recommendations.len() > 0);
            prop_assert!(result.recommendations.iter().any(|r| r.contains("critical")));
        }

        /// Property: High scoring categories get healthy message
        #[test]
        fn prop_healthy_for_high_scores(raw_score in 85.0f64..=100.0) {
            let categories = vec![CategoryScore::new("Good", raw_score, 100)];
            let result = PerfectionScoreResult::new(categories);
            prop_assert!(result.recommendations.iter().any(|r| r.contains("healthy")));
        }
    }

    // ============================================================================
    // Edge Case Tests
    // ============================================================================

    #[test]
    fn test_category_score_extreme_values() {
        // Very small max points
        let score = CategoryScore::new("Test", 50.0, 1);
        assert_eq!(score.earned_points, 0.5);

        // Very large max points
        let score = CategoryScore::new("Test", 50.0, u16::MAX);
        assert!((score.earned_points - (f64::from(u16::MAX) / 2.0)).abs() < 1.0);
    }

    #[test]
    fn test_category_score_floating_point_precision() {
        // Test that floating point precision is reasonable
        let score = CategoryScore::new("Test", 33.333333, 100);
        assert!((score.earned_points - 33.333333).abs() < 0.0001);
    }

    #[test]
    fn test_perfection_score_result_with_negative_target_gap() {
        let categories = vec![CategoryScore::new("TDG", 100.0, 100)];
        let result = PerfectionScoreResult::new(categories).with_target(50);
        assert!(result.target_gap.unwrap() < 0.0);
    }

    #[test]
    fn test_max_perfection_score_constant() {
        assert_eq!(MAX_PERFECTION_SCORE, 200);
    }

    #[test]
    fn test_category_score_name_special_characters() {
        let score = CategoryScore::new("Test-Category_123", 80.0, 40);
        assert_eq!(score.name, "Test-Category_123");
    }

    #[test]
    fn test_category_score_empty_name() {
        let score = CategoryScore::new("", 80.0, 40);
        assert_eq!(score.name, "");
    }

    #[test]
    fn test_perfection_score_result_many_categories() {
        let categories: Vec<CategoryScore> = (0..100)
            .map(|i| CategoryScore::new(&format!("Category{}", i), 80.0, 2))
            .collect();
        let result = PerfectionScoreResult::new(categories);
        // Use epsilon comparison for floating point (100 * 1.6 earned points)
        assert!(
            (result.total_score - 160.0).abs() < 0.001,
            "Expected ~160.0, got {}",
            result.total_score
        );
    }

    #[tokio::test]
    async fn test_get_performance_score_workspace_structure() {
        let temp_dir = TempDir::new().unwrap();
        // Create server/benches structure (workspace-aware)
        fs::create_dir_all(temp_dir.path().join("server/benches")).unwrap();
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_performance_score(temp_dir.path()).await;
        assert_eq!(score, 80.0); // 50 base + 30 for benches
    }

    #[tokio::test]
    async fn test_get_mutation_score_server_mutants_toml() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir(temp_dir.path().join("server")).unwrap();
        fs::write(temp_dir.path().join("server/mutants.toml"), "[mutants]").unwrap();
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_mutation_score(temp_dir.path()).await;
        assert_eq!(score, 70.0); // 50 base + 20 for config
    }

    #[tokio::test]
    async fn test_get_coverage_score_workspace_cache() {
        let temp_dir = TempDir::new().unwrap();
        // Create server/.pmat-metrics/coverage.json
        let metrics_dir = temp_dir.path().join("server/.pmat-metrics");
        fs::create_dir_all(&metrics_dir).unwrap();
        fs::write(metrics_dir.join("coverage.json"), r#"{"coverage": 92.5}"#).unwrap();
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_coverage_score(temp_dir.path()).await;
        assert_eq!(score, 92.5);
    }

    #[tokio::test]
    async fn test_get_coverage_score_with_tokio_tests() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            r#"
            #[tokio::test]
            async fn test1() {}
            #[tokio::test]
            async fn test2() {}
            "#,
        )
        .unwrap();
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_coverage_score(temp_dir.path()).await;
        // Should detect tokio::test annotations
        assert!(score >= 50.0);
    }

    #[tokio::test]
    async fn test_get_documentation_score_partial() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("README.md"), "# Test").unwrap();
        fs::write(temp_dir.path().join("CHANGELOG.md"), "# Changes").unwrap();
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_documentation_score(temp_dir.path()).await;
        assert_eq!(score, 60.0); // 40 + 20
    }

    // ============================================================================
    // Calculator Integration Tests (using temp dirs to avoid slow external services)
    // ============================================================================

    #[tokio::test]
    async fn test_calculator_fast_mode_mutation_default() {
        let temp_dir = TempDir::new().unwrap();
        let calc = PerfectionScoreCalculator::new().fast_mode(true);

        // In fast mode, mutation score should be 50.0 (default credit)
        let result = calc.calculate(temp_dir.path()).await.unwrap();

        let mutation_cat = result
            .categories
            .iter()
            .find(|c| c.name == "Mutation Testing")
            .unwrap();
        assert_eq!(mutation_cat.raw_score, 50.0);
        assert!(mutation_cat
            .details
            .as_ref()
            .is_some_and(|d| d.contains("fast mode")));
    }

    #[test]
    fn test_category_weights_copy_trait() {
        let weights = CategoryWeights::default();
        let copy = weights; // Copy
        assert_eq!(weights.tdg, copy.tdg);
    }

    // ============================================================================
    // Serialization Round-Trip Tests
    // ============================================================================

    #[test]
    fn test_category_score_serde_roundtrip() {
        let score = CategoryScore::new("Test", 75.5, 40).with_details("Test details");
        let json = serde_json::to_string(&score).unwrap();
        let deserialized: CategoryScore = serde_json::from_str(&json).unwrap();

        assert_eq!(score.name, deserialized.name);
        assert_eq!(score.raw_score, deserialized.raw_score);
        assert_eq!(score.max_points, deserialized.max_points);
        assert_eq!(score.earned_points, deserialized.earned_points);
        assert_eq!(score.grade, deserialized.grade);
        assert_eq!(score.details, deserialized.details);
    }

    #[test]
    fn test_perfection_score_result_serde_roundtrip() {
        let categories = vec![
            CategoryScore::new("TDG", 80.0, 40),
            CategoryScore::new("Repo", 75.0, 30),
        ];
        let result = PerfectionScoreResult::new(categories).with_target(150);
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: PerfectionScoreResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result.total_score, deserialized.total_score);
        assert_eq!(result.max_score, deserialized.max_score);
        assert_eq!(result.grade, deserialized.grade);
        assert_eq!(result.categories.len(), deserialized.categories.len());
        assert_eq!(result.target_gap, deserialized.target_gap);
    }
}
