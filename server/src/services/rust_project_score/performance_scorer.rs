//! PerformanceScorer - Performance & Benchmarking Category (10 points)
//!
//! Based on "Learn from Rust Giants" specification (v2.0):
//! - Criterion benchmarks configured ([[bench]] sections): 5pts
//! - CI workflow for benchmark baselines: 3pts
//! - harness = false for custom bench harness: 2pts
//!
//! Academic Foundation:
//! - ICST 2024: Criterion-based CI reduces performance bugs by 67%
//! - Projects with automated performance regression detection ship 2.4x faster

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

    /// Score Criterion benchmarks configured in [[bench]] sections (5pts)
    /// Based on "Learn from Rust Giants" specification
    ///
    /// **Kaizen Round 4**: Cache-aware - uses FileCache if available for Cargo.toml
    /// **v2.0**: Simplified to match spec - checks for [[bench]] sections only
    fn score_benchmarks(
        &self,
        project_path: &Path,
        cache: Option<&FileCache>,
    ) -> ScorerResult<f64> {
        let cargo_toml_path = project_path.join("Cargo.toml");

        // Try cache first, fall back to filesystem
        let content_result = if let Some(cache) = cache {
            cache.get(&cargo_toml_path).map(|s| s.to_string()).ok_or(())
        } else {
            std::fs::read_to_string(&cargo_toml_path).map_err(|_| ())
        };

        if let Ok(content) = content_result {
            // Check for [[bench]] sections in Cargo.toml
            if content.contains("[[bench]]") {
                return Ok(5.0);
            }
        }

        Ok(0.0)
    }

    /// Score CI workflow for benchmark baselines (3pts)
    /// Checks for .github/workflows with benchmark automation
    fn score_benchmark_ci(&self, project_path: &Path) -> ScorerResult<f64> {
        let workflows_dir = project_path.join(".github/workflows");
        if !workflows_dir.exists() {
            return Ok(0.0);
        }

        // Check for benchmark workflow files
        if let Ok(entries) = std::fs::read_dir(&workflows_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "yml" || ext == "yaml" {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            // Check for benchmark-related keywords
                            if content.contains("cargo bench") ||
                               content.contains("benchmark") ||
                               content.contains("bench-baseline") {
                                return Ok(3.0);
                            }
                        }
                    }
                }
            }
        }

        Ok(0.0)
    }

    /// Score harness = false for custom bench harness (2pts)
    /// Checks [[bench]] sections for harness = false
    fn score_custom_harness(&self, project_path: &Path, cache: Option<&FileCache>) -> ScorerResult<f64> {
        let cargo_toml_path = project_path.join("Cargo.toml");

        // Try cache first, fall back to filesystem
        let content_result = if let Some(cache) = cache {
            cache.get(&cargo_toml_path).map(|s| s.to_string()).ok_or(())
        } else {
            std::fs::read_to_string(&cargo_toml_path).map_err(|_| ())
        };

        if let Ok(content) = content_result {
            // Check for harness = false in [[bench]] sections
            if content.contains("[[bench]]") && content.contains("harness = false") {
                return Ok(2.0);
            }
        }

        Ok(0.0)
    }

    /// Internal scoring logic that accepts optional cache
    ///
    /// **Kaizen Round 4**: Cache-aware scoring implementation
    /// **v2.0**: Aligned with "Learn from Rust Giants" specification
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

        // Score benchmarks - [[bench]] sections configured (5pts)
        match self.score_benchmarks(project_path, cache) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        // Score benchmark CI workflow (3pts)
        match self.score_benchmark_ci(project_path) {
            Ok(score) => total_earned += score,
            Err(e) => return Err(e),
        }

        // Score custom harness (2pts)
        match self.score_custom_harness(project_path, cache) {
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

        // Check [[bench]] sections (no cache - backward compatibility)
        if let Ok(score) = self.score_benchmarks(project_path, None) {
            if score < 5.0 {
                recommendations.push(
                    "Add [[bench]] sections: Configure benchmark targets in Cargo.toml with Criterion".to_string(),
                );
            }
        }

        // Check benchmark CI workflow
        if let Ok(score) = self.score_benchmark_ci(project_path) {
            if score < 3.0 {
                recommendations.push(
                    "Add benchmark CI: Create .github/workflows with 'cargo bench' for automated performance testing".to_string(),
                );
            }
        }

        // Check custom harness
        if let Ok(score) = self.score_custom_harness(project_path, None) {
            if score < 2.0 {
                recommendations.push(
                    "Use custom harness: Add 'harness = false' to [[bench]] sections for Criterion integration".to_string(),
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
