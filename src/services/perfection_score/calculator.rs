#![cfg_attr(coverage_nightly, coverage(off))]
//! Perfection Score Calculator implementation.

use crate::models::tdg::TDGConfig;
use crate::services::popper_score::orchestrator::PopperOrchestrator;
use crate::services::repo_score::aggregator::ScoreAggregator;
use crate::services::repo_score::scorers::ScorerConfig;
use crate::services::rust_project_score::models::ScoringMode;
use crate::services::rust_project_score::orchestrator::RustProjectScoreOrchestrator;
use crate::services::tdg_calculator::TDGCalculator;
use std::path::Path;
use std::time::Duration;

use super::types::{CategoryScore, CategoryWeights, PerfectionScoreResult};

/// Perfection Score Calculator
pub struct PerfectionScoreCalculator {
    pub(super) weights: CategoryWeights,
    pub(super) fast_mode: bool,
}

impl Default for PerfectionScoreCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl PerfectionScoreCalculator {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            weights: CategoryWeights::default(),
            fast_mode: false,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Fast mode.
    pub fn fast_mode(mut self, fast: bool) -> Self {
        self.fast_mode = fast;
        self
    }

    /// Calculate perfection score for a project.
    ///
    /// Categories 1-4 (TDG, repo-score, rust-project-score, popper-score) run in
    /// parallel via `tokio::join!`. The entire calculation is wrapped in a 120-second
    /// timeout to prevent runaway CPU usage from unbounded `git log` subprocesses.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub async fn calculate(&self, project_path: &Path) -> anyhow::Result<PerfectionScoreResult> {
        match tokio::time::timeout(Duration::from_secs(120), self.calculate_inner(project_path))
            .await
        {
            Ok(result) => result,
            Err(_elapsed) => {
                eprintln!("⚠️  Perfection score calculation timed out after 120s");
                // Return a partial result with timeout details
                let categories = vec![
                    CategoryScore::new("Technical Debt Grade", 0.0, self.weights.tdg)
                        .with_details("Timed out"),
                    CategoryScore::new("Repository Health", 0.0, self.weights.repo_score)
                        .with_details("Timed out"),
                    CategoryScore::new("Rust Project Quality", 0.0, self.weights.rust_score)
                        .with_details("Timed out"),
                    CategoryScore::new("Popperian Falsifiability", 0.0, self.weights.popper_score)
                        .with_details("Timed out"),
                    CategoryScore::new("Test Coverage", 0.0, self.weights.test_coverage)
                        .with_details("Timed out"),
                    CategoryScore::new("Mutation Testing", 0.0, self.weights.mutation)
                        .with_details("Timed out"),
                    CategoryScore::new("Documentation", 0.0, self.weights.documentation)
                        .with_details("Timed out"),
                    CategoryScore::new("Performance", 0.0, self.weights.performance)
                        .with_details("Timed out"),
                ];
                Ok(PerfectionScoreResult::new(categories))
            }
        }
    }

    /// Inner calculation logic, called within the timeout wrapper.
    async fn calculate_inner(&self, project_path: &Path) -> anyhow::Result<PerfectionScoreResult> {
        // Categories 1-4 are expensive and independent — run in parallel
        let (tdg_score, repo_score, rust_score, popper_score) = tokio::join!(
            self.get_tdg_score(project_path),
            self.get_repo_score(project_path),
            self.get_rust_project_score(project_path),
            self.get_popper_score(project_path),
        );

        let mut categories = vec![
            CategoryScore::new("Technical Debt Grade", tdg_score, self.weights.tdg),
            CategoryScore::new("Repository Health", repo_score, self.weights.repo_score),
            CategoryScore::new("Rust Project Quality", rust_score, self.weights.rust_score),
            CategoryScore::new(
                "Popperian Falsifiability",
                popper_score,
                self.weights.popper_score,
            ),
        ];

        // Categories 5-8 are cheap filesystem checks — run sequentially

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

    pub(super) async fn get_tdg_score(&self, project_path: &Path) -> f64 {
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

    pub(super) async fn get_repo_score(&self, project_path: &Path) -> f64 {
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

    pub(super) async fn get_rust_project_score(&self, project_path: &Path) -> f64 {
        // Rust Project Score: raw points, normalize to 0-100 using the
        // orchestrator-reported max (289 in v3.0). Never hardcode the scale —
        // it grows as scorers are added (was 134, now 289).
        let orchestrator = RustProjectScoreOrchestrator::new();
        let mode = if self.fast_mode {
            ScoringMode::Quick
        } else {
            ScoringMode::Fast
        };

        match orchestrator.score_with_mode(project_path, mode) {
            Ok(score) => normalize_rps_percentage(score.total_earned, score.total_possible),
            Err(e) => {
                eprintln!("⚠️  Rust project score failed: {}", e);
                50.0 // Default on error
            }
        }
    }

    pub(super) async fn get_popper_score(&self, project_path: &Path) -> f64 {
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

    pub(super) async fn get_coverage_score(&self, project_path: &Path) -> f64 {
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

    pub(super) async fn get_mutation_score(&self, project_path: &Path) -> f64 {
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

    pub(super) async fn get_documentation_score(&self, project_path: &Path) -> f64 {
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

    pub(super) async fn get_performance_score(&self, project_path: &Path) -> f64 {
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

/// Convert raw rust-project-score points to a 0-100 percentage.
///
/// `total_earned` is raw points out of `total_possible` (289 in RPS v3.0).
/// Feeding raw points directly into `CategoryScore::new` (which expects a
/// 0-100 percentage) inflated the category past its max — e.g. raw 184
/// scaled to 55.2/30 and clamped the total at 200/200 A+.
pub(super) fn normalize_rps_percentage(total_earned: f64, total_possible: f64) -> f64 {
    if total_possible > 0.0 {
        ((total_earned / total_possible) * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    }
}
