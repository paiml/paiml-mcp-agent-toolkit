//! RustProjectScore Orchestrator
//!
//! Aggregates all 10 category scorers into a unified project score.
//!
//! Categories (159 points total):
//! - Rust Tooling Compliance (25pts)
//! - Code Quality (26pts)
//! - Testing Excellence (20pts)
//! - Documentation (15pts)
//! - Performance & Benchmarking (10pts)
//! - Dependency Health (12pts)
//! - Formal Verification (8pts)
//! - Known Defects (20pts)
//! - GPU/SIMD Quality (10pts) - v2.2
//! - Build Performance (15pts) - NEW in v2.3

use super::build_perf_scorer::BuildPerfScorer;
use super::code_quality_scorer::CodeQualityScorer;
use super::dependency_scorer::DependencyScorer;
use super::documentation_scorer::DocumentationScorer;
use super::formal_verification_scorer::FormalVerificationScorer;
use super::gpu_simd_scorer::GpuSimdScorer;
use super::known_defects_scorer::KnownDefectsScorer;
use super::models::*;
use super::performance_scorer::PerformanceScorer;
use super::rust_tooling_scorer::RustToolingScorer;
use super::scorer::{Scorer, ScorerError, ScorerResult};
use super::testing_scorer::TestingScorer;
use indicatif::ProgressBar;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::Path;

/// Rust Project Score specification version
/// This tracks the scoring methodology version, not the PMAT binary version
pub const SPEC_VERSION: &str = "2.3";

/// Orchestrates all 10 category scorers to produce unified project score
pub struct RustProjectScoreOrchestrator {
    /// All 10 category scorers
    scorers: Vec<Box<dyn Scorer>>,
}

impl RustProjectScoreOrchestrator {
    /// Create a new orchestrator with all 10 scorers
    pub fn new() -> Self {
        let scorers: Vec<Box<dyn Scorer>> = vec![
            Box::new(RustToolingScorer::new()),
            Box::new(CodeQualityScorer::new()),
            Box::new(TestingScorer::new()),
            Box::new(DocumentationScorer::new()),
            Box::new(PerformanceScorer::new()),
            Box::new(DependencyScorer::new()),
            Box::new(FormalVerificationScorer::new()),
            Box::new(KnownDefectsScorer::new()),
            Box::new(GpuSimdScorer::new()),
            Box::new(BuildPerfScorer::new()),
        ];

        Self { scorers }
    }

    /// Get orchestrator name with spec version
    pub fn name(&self) -> String {
        format!("Rust Project Score v{}", SPEC_VERSION)
    }

    /// Get maximum possible points (159)
    pub fn max_points(&self) -> f64 {
        159.0
    }

    /// Get all scorer names
    pub fn scorer_names(&self) -> Vec<&str> {
        self.scorers.iter().map(|s| s.name()).collect()
    }

    /// Get maximum points by category
    pub fn max_points_by_category(&self) -> HashMap<&str, f64> {
        self.scorers
            .iter()
            .map(|s| (s.name(), s.max_points()))
            .collect()
    }

    /// Calculate grade from score and max
    pub fn calculate_grade(&self, earned: f64, _max: f64) -> Grade {
        Grade::from_score(earned, _max)
    }

    /// Score a Rust project with fast mode (default, <60 seconds)
    ///
    /// Runs all 10 category scorers and aggregates results
    pub fn score(&self, project_path: &Path) -> ScorerResult<ProjectScore> {
        self.score_with_mode(project_path, ScoringMode::default())
    }

    /// Score a Rust project with configurable mode
    ///
    /// # Arguments
    /// * `project_path` - Path to Rust project
    /// * `mode` - Scoring mode (Quick/<10s, Fast/<60s, Full/<5m)
    ///
    /// # Performance Targets
    /// - Quick mode: <10 seconds - Filesystem only
    /// - Fast mode (default): <60 seconds - Skip expensive cargo operations
    /// - Full mode: <5 minutes (300 seconds) - Complete analysis
    pub fn score_with_mode(
        &self,
        project_path: &Path,
        mode: ScoringMode,
    ) -> ScorerResult<ProjectScore> {
        // Verify project has Cargo.toml
        if !project_path.join("Cargo.toml").exists() {
            return Err(ScorerError::InvalidProject(
                "No Cargo.toml found - not a valid Rust project".to_string(),
            ));
        }

        // Verify path exists
        if !project_path.exists() {
            return Err(ScorerError::InvalidProject(format!(
                "Path does not exist: {}",
                project_path.display()
            )));
        }

        // Verify path is a directory
        if !project_path.is_dir() {
            return Err(ScorerError::InvalidProject(format!(
                "Path is not a directory: {}",
                project_path.display()
            )));
        }

        // ═══════════════════════════════════════════════════════════════
        // Kaizen Round 4: Create FileCache once, share across all scorers
        // ═══════════════════════════════════════════════════════════════
        // BEFORE: Each scorer read filesystem independently (22 walks!)
        // AFTER: Single filesystem walk, cache shared (3x faster)
        let file_cache = match FileCache::populate(project_path) {
            Ok(cache) => {
                let (files, bytes) = cache.stats();
                if mode == ScoringMode::Full {
                    // Only show cache stats in full mode (verbose)
                    eprintln!("📦 Cached {} files ({} KB)", files, bytes / 1024);
                }
                Some(cache)
            }
            Err(e) => {
                eprintln!("⚠️  FileCache failed: {}, using direct filesystem reads", e);
                None
            }
        };

        // Run all scorers and collect results
        // **Kaizen Round 5**: Parallel scorer execution for 2-3x speedup
        let mut category_map: HashMap<String, CategoryScore> = HashMap::new();
        let mut all_recommendations: Vec<String> = Vec::new();

        // Create progress spinner (simpler for parallel execution)
        let pb = ProgressBar::new_spinner();
        pb.set_message(format!(
            "Analyzing {} categories in parallel...",
            self.scorers.len()
        ));
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        // Run scorers in parallel using rayon
        let results: Result<Vec<_>, ScorerError> = self
            .scorers
            .par_iter()
            .map(|scorer| {
                let category_score =
                    scorer.score_with_cache(project_path, mode, file_cache.as_ref())?;
                let recommendations = scorer.recommendations(project_path);
                Ok((scorer.name().to_string(), category_score, recommendations))
            })
            .collect();

        pb.finish_with_message("✅ Analysis complete");

        // Unpack parallel results
        let results = results?;
        for (name, score, recs) in results {
            category_map.insert(name, score);
            all_recommendations.extend(recs);
        }

        // Build CategoryScores struct
        let categories = category_map.clone();

        // Calculate total earned
        let total_earned: f64 = category_map.values().map(|cs| cs.earned).sum();

        // Calculate percentage
        let percentage = (total_earned / self.max_points()) * 100.0;

        // Calculate grade
        let grade = self.calculate_grade(total_earned, self.max_points());

        Ok(ProjectScore {
            total_earned,
            total_possible: self.max_points(),
            percentage,
            grade,
            categories,
            recommendations: all_recommendations,
        })
    }
}

impl Default for RustProjectScoreOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for RustProjectScoreOrchestrator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RustProjectScoreOrchestrator")
            .field("scorer_count", &self.scorers.len())
            .field("max_points", &self.max_points())
            .finish()
    }
}

/// Project score result from orchestrator
#[derive(Debug, Clone)]
pub struct ProjectScore {
    /// Total points earned
    pub total_earned: f64,

    /// Total possible points (159) - 10 categories
    pub total_possible: f64,

    /// Percentage (0-100)
    pub percentage: f64,

    /// Letter grade
    pub grade: Grade,

    /// Scores by category
    pub categories: HashMap<String, CategoryScore>,

    /// Recommendations
    pub recommendations: Vec<String>,
}

// Ensure Send + Sync for parallel execution
unsafe impl Send for RustProjectScoreOrchestrator {}
unsafe impl Sync for RustProjectScoreOrchestrator {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestrator_creation() {
        let orch = RustProjectScoreOrchestrator::new();
        assert_eq!(orch.name(), format!("Rust Project Score v{}", SPEC_VERSION));
        assert_eq!(orch.max_points(), 159.0);
    }

    #[test]
    fn test_scorer_count() {
        let orch = RustProjectScoreOrchestrator::new();
        assert_eq!(orch.scorers.len(), 10);
    }

    #[test]
    fn test_formal_verification_scorer_present() {
        let orch = RustProjectScoreOrchestrator::new();
        let names: Vec<&str> = orch.scorer_names();
        assert!(names.contains(&"Formal Verification"));
    }

    #[test]
    fn test_known_defects_scorer_present() {
        let orch = RustProjectScoreOrchestrator::new();
        let names: Vec<&str> = orch.scorer_names();
        assert!(names.contains(&"Known Defects"));
    }

    #[test]
    fn test_gpu_simd_scorer_present() {
        let orch = RustProjectScoreOrchestrator::new();
        let names: Vec<&str> = orch.scorer_names();
        assert!(names.contains(&"GPU/SIMD Quality"));
    }
}
