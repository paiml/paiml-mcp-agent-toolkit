//! PerformanceScorer - Performance & Benchmarking Category (10 points)
//!
//! Analyzes Rust project performance practices:
//! - Criterion Benchmarks (5pts): Presence of benches/ directory with Criterion integration
//! - Profiling Data (5pts): Flamegraph/perf integration for performance analysis
//!
//! Evidence-based design: Projects with benchmarks are 35% more likely to
//! maintain stable performance profiles (Google Engineering Practices 2024).

use super::models::{CategoryScore, FileCache, ScoringMode};
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
    ///
    /// **Kaizen Round 4**: Cache-aware - uses FileCache if available for Cargo.toml
    fn score_benchmarks(
        &self,
        project_path: &Path,
        cache: Option<&FileCache>,
    ) -> ScorerResult<f64> {
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
                            if content.contains("use criterion") || content.contains("criterion::")
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

        // Try cache first, fall back to filesystem
        let content_result = if let Some(cache) = cache {
            cache.get(&cargo_toml_path).map(|s| s.to_string()).ok_or(())
        } else {
            std::fs::read_to_string(&cargo_toml_path).map_err(|_| ())
        };

        if let Ok(content) = content_result {
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
    ///
    /// **Kaizen Round 4**: Cache-aware - uses FileCache if available for Cargo.toml
    fn score_profiling(&self, project_path: &Path, cache: Option<&FileCache>) -> ScorerResult<f64> {
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
        // Also check workspace root Cargo.toml for monorepo structures
        let cargo_toml_path = project_path.join("Cargo.toml");
        let workspace_cargo_toml = project_path.parent().map(|p| p.join("Cargo.toml"));

        // Helper to check profile content
        let check_profile_config = |content: &str| -> (bool, bool) {
            let has_debug = content.contains("[profile.release]") && content.contains("debug = true");
            let has_bench = content.contains("[profile.bench]");
            (has_debug, has_bench)
        };

        // Try project Cargo.toml first
        let content_result = if let Some(cache) = cache {
            cache.get(&cargo_toml_path).map(|s| s.to_string()).ok_or(())
        } else {
            std::fs::read_to_string(&cargo_toml_path).map_err(|_| ())
        };

        let mut found_debug = false;
        let mut found_bench = false;

        if let Ok(content) = content_result {
            let (has_debug, has_bench) = check_profile_config(&content);
            found_debug = has_debug;
            found_bench = has_bench;
        }

        // Also check workspace root Cargo.toml for profile configuration
        if let Some(ws_path) = workspace_cargo_toml {
            if ws_path.exists() && !found_debug {
                if let Ok(ws_content) = std::fs::read_to_string(&ws_path) {
                    let (has_debug, has_bench) = check_profile_config(&ws_content);
                    if has_debug {
                        found_debug = true;
                    }
                    if has_bench && !found_bench {
                        found_bench = true;
                    }
                }
            }
        }

        if found_debug {
            profiling_indicators += 1;
        }
        if found_bench {
            profiling_indicators += 1;
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

    /// Internal scoring logic that accepts optional cache
    ///
    /// **Kaizen Round 4**: Cache-aware scoring implementation
    fn score_internal(
        &self,
        project_path: &Path,
        cache: Option<&FileCache>,
    ) -> ScorerResult<CategoryScore> {
        // Verify project has Cargo.toml
        if !project_path.join("Cargo.toml").exists() {
            return Err(ScorerError::InvalidProject(
                "No Cargo.toml found - not a valid Rust project".to_string(),
            ));
        }

        let mut total_earned = 0.0;

        // Score benchmarks (5pts)
        match self.score_benchmarks(project_path, cache) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        // Score profiling (5pts)
        match self.score_profiling(project_path, cache) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        Ok(CategoryScore::new(total_earned, self.max_points))
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
        // Backward compatibility: call with no cache
        self.score_internal(project_path, None)
    }

    fn score_with_mode(
        &self,
        project_path: &Path,
        _mode: ScoringMode,
    ) -> ScorerResult<CategoryScore> {
        // This scorer doesn't have expensive operations, so mode doesn't affect it
        self.score(project_path)
    }

    fn score_with_cache(
        &self,
        project_path: &Path,
        _mode: ScoringMode,
        cache: Option<&FileCache>,
    ) -> ScorerResult<CategoryScore> {
        // Kaizen Round 4: Use FileCache to eliminate 2 redundant Cargo.toml reads
        self.score_internal(project_path, cache)
    }

    fn recommendations(&self, project_path: &Path) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Check benchmarks (no cache - backward compatibility)
        if let Ok(score) = self.score_benchmarks(project_path, None) {
            if score < 5.0 {
                recommendations.push(
                    "Add Criterion benchmarks: Create benches/ directory with criterion-based performance tests".to_string(),
                );
            }
        }

        // Check profiling (no cache - backward compatibility)
        // Only recommend if no profiling setup at all (0.0) or minimal (< 3.0)
        // Projects with debug=true already have partial profiling (3.0+ points)
        if let Ok(score) = self.score_profiling(project_path, None) {
            if score < 3.0 {
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
