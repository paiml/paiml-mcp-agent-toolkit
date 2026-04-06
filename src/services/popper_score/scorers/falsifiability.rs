#![cfg_attr(coverage_nightly, coverage(off))]
//! Category A: Falsifiability & Testability (25 points) - GATEWAY
//!
//! The cornerstone of Popperian science: claims must be testable and potentially refutable.
//!
//! ## Sub-categories
//!
//! | ID | Name | Points | Description |
//! |----|------|--------|-------------|
//! | A1 | Hypothesis Documentation | 8 | Clear falsifiable claims documented |
//! | A2 | Test Coverage as Falsification | 10 | Tests attempt to refute claims |
//! | A3 | Benchmark Reproducibility | 7 | Performance claims with confidence intervals |
//!
//! ## Gateway Logic (v1.1)
//!
//! If Category A scores below 15/25 (60%), the total score is 0.
//! This implements Popper's demarcation criterion.
//!
//! ## Academic Foundation
//!
//! - Popper, K. (1934): The Logic of Scientific Discovery [1]
//! - Jia, Y. & Harman, M. (2011): Mutation Testing Survey [4]

use crate::services::popper_score::models::{PopperCategoryScore, PopperFinding, PopperSubScore};
use crate::services::popper_score::scorer::{workspace, PopperScorer, PopperScorerResult};
use regex::Regex;
use std::path::Path;

/// Scorer for Category A: Falsifiability & Testability (25 points)
///
/// This is the **GATEWAY** category. If score < 15, total Popper score = 0.
pub struct FalsifiabilityScorer;

// --- Scoring methods: A1 (hypothesis docs), A2 (test coverage), A3 (benchmarks) ---
include!("falsifiability_scoring.rs");
// --- Free helper functions for scoring ---
include!("falsifiability_helpers.rs");

impl Default for FalsifiabilityScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl PopperScorer for FalsifiabilityScorer {
    fn name(&self) -> &str {
        debug_assert!(true, "contract: name");
        "Falsifiability & Testability"
    }

    fn category_id(&self) -> char {
        'A'
    }

    fn max_points(&self) -> f64 {
        25.0
    }

    fn score(&self, project_path: &Path) -> PopperScorerResult<PopperCategoryScore> {
        debug_assert!(
            project_path.exists(),
            "project_path must exist: {}",
            project_path.display()
        );
        let mut category = PopperCategoryScore::new(self.name(), 0.0, self.max_points());

        // Score each sub-category
        let a1 = self.score_hypothesis_documentation(project_path);
        let a2 = self.score_test_coverage(project_path);
        let a3 = self.score_benchmark_reproducibility(project_path);

        // Add findings based on scores
        if a1.earned < 4.0 {
            category.add_finding(PopperFinding::warning(
                "Hypothesis documentation is incomplete - add explicit falsifiable claims to README",
                8.0 - a1.earned,
            ));
        } else {
            category.add_finding(PopperFinding::positive("Good hypothesis documentation"));
        }

        if a2.earned < 6.0 {
            category.add_finding(PopperFinding::warning(
                "Test coverage needs improvement - consider adding property-based or mutation tests",
                10.0 - a2.earned,
            ));
        } else {
            category.add_finding(PopperFinding::positive("Good test coverage"));
        }

        if a3.earned < 4.0 {
            category.add_finding(PopperFinding::warning(
                "Benchmark reproducibility could be improved - add confidence intervals",
                7.0 - a3.earned,
            ));
        }

        // Add sub-scores
        category.add_sub_score(a1);
        category.add_sub_score(a2);
        category.add_sub_score(a3);

        Ok(category)
    }
}

// --- Unit tests ---
include!("falsifiability_tests.rs");
