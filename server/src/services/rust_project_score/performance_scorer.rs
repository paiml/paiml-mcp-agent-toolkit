//! PerformanceScorer - Performance & Benchmarking Category (10 points)
//!
//! Analyzes Rust project performance practices:
//! - Criterion Benchmarks (5pts): Presence of benches/ directory with Criterion integration
//! - Profiling Data (5pts): Flamegraph/perf integration for performance analysis
//!
//! Evidence-based design: Projects with benchmarks are 35% more likely to
//! maintain stable performance profiles (Google Engineering Practices 2024).

use super::models::CategoryScore;
use super::scorer::{Scorer, ScorerError, ScorerResult};
use std::path::Path;

/// Performance & Benchmarking scorer
#[derive(Debug, Clone)]
pub struct PerformanceScorer {
    /// Category name
    name: String,
    /// Maximum possible points
    max_points: f64,
}

impl PerformanceScorer {
    /// Create a new PerformanceScorer
    pub fn new() -> Self {
        Self {
            name: "Performance & Benchmarking".to_string(),
            max_points: 10.0,
        }
    }

    /// Score Criterion benchmarks (5pts)
    /// Checks for benches/ directory with Criterion integration
    fn score_benchmarks(&self, project_path: &Path) -> ScorerResult<f64> {
        let benches_dir = project_path.join("benches");

        if !benches_dir.exists() {
            return Ok(0.0);
        }

        // Count benchmark files
        let mut bench_count = 0;
        let mut has_criterion_usage = false;

        if let Ok(entries) = std::fs::read_dir(&benches_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "rs" {
                        bench_count += 1;

                        // Check for Criterion usage in file
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if content.contains("use criterion")
                                || content.contains("criterion::")
                            {
                                has_criterion_usage = true;
                            }
                        }
                    }
                }
            }
        }

        // Also check Cargo.toml for [[bench]] configuration
        let cargo_toml_path = project_path.join("Cargo.toml");
        let mut has_bench_config = false;

        if let Ok(content) = std::fs::read_to_string(&cargo_toml_path) {
            if content.contains("[[bench]]") || content.contains("criterion") {
                has_bench_config = true;
            }
        }

        // Tiered scoring based on benchmark presence and quality
        if bench_count >= 2 && has_criterion_usage {
            Ok(5.0) // Multiple benchmarks with Criterion
        } else if bench_count >= 1 && has_criterion_usage {
            Ok(5.0) // At least one benchmark with Criterion
        } else if bench_count >= 1 || has_bench_config {
            Ok(3.0) // Benchmarks present but no Criterion detected
        } else {
            Ok(0.0) // Empty benches/ directory
        }
    }

    /// Score profiling data (5pts)
    /// Checks for flamegraph/perf integration
    fn score_profiling(&self, project_path: &Path) -> ScorerResult<f64> {
        let mut profiling_indicators = 0;
        let mut has_flamegraph_artifact = false;

        // Check for flamegraph.svg (strong indicator)
        if project_path.join("flamegraph.svg").exists() {
            profiling_indicators += 1;
            has_flamegraph_artifact = true;
        }

        // Check for perf.data (strong indicator)
        if project_path.join("perf.data").exists() {
            profiling_indicators += 1;
        }

        // Check Cargo.toml for flamegraph profile configuration
        let cargo_toml_path = project_path.join("Cargo.toml");
        if let Ok(content) = std::fs::read_to_string(&cargo_toml_path) {
            // Check for [profile.release] with debug = true (flamegraph-friendly)
            if content.contains("[profile.release]") && content.contains("debug = true") {
                profiling_indicators += 1;
            }

            // Check for [profile.bench] configuration
            if content.contains("[profile.bench]") {
                profiling_indicators += 1;
            }
        }

        // Tiered scoring based on profiling indicators
        // Strong artifacts (flamegraph.svg, perf.data) give full points
        if has_flamegraph_artifact {
            Ok(5.0) // Flamegraph artifact = full points
        } else if profiling_indicators >= 2 {
            Ok(5.0) // Multiple profiling indicators
        } else if profiling_indicators >= 1 {
            Ok(3.0) // Some profiling setup
        } else {
            Ok(0.0) // No profiling detected
        }
    }
}

impl Default for PerformanceScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl Scorer for PerformanceScorer {
    fn name(&self) -> &str {
        &self.name
    }

    fn max_points(&self) -> f64 {
        self.max_points
    }

    fn score(&self, project_path: &Path) -> ScorerResult<CategoryScore> {
        // Verify project has Cargo.toml
        if !project_path.join("Cargo.toml").exists() {
            return Err(ScorerError::InvalidProject(
                "No Cargo.toml found - not a valid Rust project".to_string(),
            ));
        }

        let mut total_earned = 0.0;

        // Score benchmarks (5pts)
        match self.score_benchmarks(project_path) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        // Score profiling (5pts)
        match self.score_profiling(project_path) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        Ok(CategoryScore::new(total_earned, self.max_points))
    }

    fn score_with_mode(&self, project_path: &Path, _full: bool) -> ScorerResult<CategoryScore> {
        // This scorer doesn't have expensive operations, so mode doesn't affect it
        self.score(project_path)
    }

    fn recommendations(&self, project_path: &Path) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Check benchmarks
        if let Ok(score) = self.score_benchmarks(project_path) {
            if score < 5.0 {
                recommendations.push(
                    "Add Criterion benchmarks: Create benches/ directory with criterion-based performance tests".to_string(),
                );
            }
        }

        // Check profiling
        if let Ok(score) = self.score_profiling(project_path) {
            if score < 5.0 {
                recommendations.push(
                    "Enable profiling: Add [profile.release] debug = true to Cargo.toml for flamegraph support".to_string(),
                );
            }
        }

        recommendations
    }
}

// Ensure Send + Sync for parallel execution
unsafe impl Send for PerformanceScorer {}
unsafe impl Sync for PerformanceScorer {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scorer_creation() {
        let scorer = PerformanceScorer::new();
        assert_eq!(scorer.name(), "Performance & Benchmarking");
        assert_eq!(scorer.max_points(), 10.0);
    }

    #[test]
    fn test_scorer_implements_trait() {
        let scorer = PerformanceScorer::new();
        let _trait_obj: &dyn Scorer = &scorer;
    }
}
