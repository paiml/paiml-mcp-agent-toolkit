#![cfg_attr(coverage_nightly, coverage(off))]
//! Category F: ML/AI Reproducibility (5 points) - CONDITIONAL
//!
//! Modern science standards for machine learning projects.
//! This category is N/A for non-ML projects.
//!
//! ## Sub-categories
//!
//! | ID | Name | Points | Description |
//! |----|------|--------|-------------|
//! | F1 | Random Seed Fixing | 2 | Deterministic training |
//! | F2 | Model Versioning | 2 | DVC, MLflow, or equivalent |
//! | F3 | Dataset Documentation | 1 | Data provenance |
//!
//! ## N/A Handling
//!
//! If a project is determined to be non-ML:
//! - Returns `is_applicable = false`
//! - Score is excluded from normalization denominator
//! - This prevents "free points" for non-ML projects
//!
//! ## Academic Foundation
//!
//! - NeurIPS ML Reproducibility Checklist [25]
//! - MLCommons Benchmarking [26]
//! - Pineau et al. (2021): ICLR Guidelines [27]

use crate::services::popper_score::models::{PopperCategoryScore, PopperFinding, PopperSubScore};
use crate::services::popper_score::scorer::{PopperScorer, PopperScorerResult};
use std::path::Path;

/// Patterns that indicate a project uses ML/AI
const ML_INDICATORS: &[&str] = &[
    "torch",
    "tensorflow",
    "keras",
    "sklearn",
    "scikit-learn",
    "pytorch",
    "jax",
    "huggingface",
    "transformers",
    "model",
    "neural",
    "training",
    "inference",
    "dataset",
    "ml",
    "machine learning",
    "deep learning",
    "llm",
    "embedding",
];

/// Scorer for Category F: ML/AI Reproducibility (5 points)
///
/// This is a **conditional** category - N/A for non-ML projects.
pub struct MLReproducibilityScorer;

impl MLReproducibilityScorer {
    /// Create a new ML reproducibility scorer
    pub fn new() -> Self {
        Self
    }
}

impl Default for MLReproducibilityScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl PopperScorer for MLReproducibilityScorer {
    fn name(&self) -> &str {
        "ML/AI Reproducibility"
    }

    fn category_id(&self) -> char {
        'F'
    }

    fn max_points(&self) -> f64 {
        5.0
    }

    fn score(&self, project_path: &Path) -> PopperScorerResult<PopperCategoryScore> {
        let is_ml = self.is_ml_project(project_path);

        let mut category = PopperCategoryScore::new(self.name(), 0.0, self.max_points());
        category.is_applicable = is_ml;

        if !is_ml {
            category.add_finding(PopperFinding::info(
                "Non-ML project - Category F excluded from scoring",
            ));
            return Ok(category);
        }

        // Score each sub-category
        let f1 = self.score_random_seed_fixing(project_path);
        let f2 = self.score_model_versioning(project_path);
        let f3 = self.score_dataset_documentation(project_path);

        // Add findings based on scores
        if f1.earned < 1.0 {
            category.add_finding(PopperFinding::warning(
                "Random seed not fixed - training may not be reproducible",
                2.0 - f1.earned,
            ));
        }

        if f2.earned < 1.0 {
            category.add_finding(PopperFinding::warning(
                "Model versioning missing - consider using DVC or MLflow",
                2.0 - f2.earned,
            ));
        }

        if f3.earned < 1.0 {
            category.add_finding(PopperFinding::warning(
                "Dataset documentation missing - add a data card or README",
                1.0 - f3.earned,
            ));
        }

        if f1.earned + f2.earned + f3.earned >= 4.0 {
            category.add_finding(PopperFinding::positive(
                "Strong ML reproducibility practices",
            ));
        }

        // Add sub-scores
        category.add_sub_score(f1);
        category.add_sub_score(f2);
        category.add_sub_score(f3);

        Ok(category)
    }
}

// Free helper functions for ML detection and file reading
include!("ml_reproducibility_helpers.rs");
// Scoring methods: is_ml_project, score_random_seed_fixing, score_model_versioning,
// score_dataset_documentation
include!("ml_reproducibility_scoring.rs");

// Unit tests
include!("ml_reproducibility_tests.rs");
