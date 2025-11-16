//! RustProjectScore Orchestrator
//!
//! Aggregates all 6 category scorers into a unified project score.
//!
//! Categories (106 points total):
//! - Rust Tooling Compliance (25pts)
//! - Code Quality (26pts)
//! - Testing Excellence (20pts)
//! - Documentation (15pts)
//! - Performance & Benchmarking (10pts)
//! - Dependency Health (12pts)

use super::code_quality_scorer::CodeQualityScorer;
use super::dependency_scorer::DependencyScorer;
use super::documentation_scorer::DocumentationScorer;
use super::models::*;
use super::performance_scorer::PerformanceScorer;
use super::rust_tooling_scorer::RustToolingScorer;
use super::scorer::{Scorer, ScorerError, ScorerResult};
use super::testing_scorer::TestingScorer;
use std::collections::HashMap;
use std::path::Path;

/// Orchestrates all 6 category scorers to produce unified project score
pub struct RustProjectScoreOrchestrator {
    /// All 6 category scorers
    scorers: Vec<Box<dyn Scorer>>,
}

impl RustProjectScoreOrchestrator {
    /// Create a new orchestrator with all 6 scorers
    pub fn new() -> Self {
        let scorers: Vec<Box<dyn Scorer>> = vec![
            Box::new(RustToolingScorer::new()),
            Box::new(CodeQualityScorer::new()),
            Box::new(TestingScorer::new()),
            Box::new(DocumentationScorer::new()),
            Box::new(PerformanceScorer::new()),
            Box::new(DependencyScorer::new()),
        ];

        Self { scorers }
    }

    /// Get orchestrator name
    pub fn name(&self) -> &str {
        "Rust Project Score v1.1"
    }

    /// Get maximum possible points (106)
    pub fn max_points(&self) -> f64 {
        106.0
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

    /// Score a Rust project
    ///
    /// Runs all 6 category scorers and aggregates results
    pub fn score(&self, project_path: &Path) -> ScorerResult<ProjectScore> {
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

        // Run all scorers and collect results
        let mut category_map: HashMap<String, CategoryScore> = HashMap::new();
        let mut all_recommendations: Vec<String> = Vec::new();

        for scorer in &self.scorers {
            let category_score = scorer.score(project_path)?;
            let recommendations = scorer.recommendations(project_path);

            category_map.insert(scorer.name().to_string(), category_score);
            all_recommendations.extend(recommendations);
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

    /// Total possible points (106)
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
        assert_eq!(orch.name(), "Rust Project Score v1.1");
        assert_eq!(orch.max_points(), 106.0);
    }

    #[test]
    fn test_scorer_count() {
        let orch = RustProjectScoreOrchestrator::new();
        assert_eq!(orch.scorers.len(), 6);
    }
}
