#![cfg_attr(coverage_nightly, coverage(off))]
//! Data models for pmat repo-score
//! Implements the scoring system defined in docs/specifications/components/repo-health.md
//!
//! PMAT-454: All scores normalized to 0-100 for display

use crate::services::normalized_score::NormalizedScore;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

/// Maximum possible raw points for Repo Score (base 100 + 10 bonus)
pub const REPO_SCORE_MAX_POINTS: f64 = 110.0;

/// Overall repository score result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoScore {
    /// Raw score (0-110 with bonuses)
    pub total_score: f64,
    pub grade: Grade, // A+, A, A-, B+, etc.
    pub categories: CategoryScores,
    pub recommendations: Vec<Recommendation>,
    pub metadata: ScoreMetadata,
}

/// Letter grade assignment (PMAT-454: uses normalized 0-100)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Grade {
    APlus,  // 95-100
    A,      // 90-94
    AMinus, // 85-89
    BPlus,  // 80-84
    B,      // 70-79
    C,      // 60-69
    D,      // 50-59
    F,      // 0-49
}

/// Category scores (base 100 points)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryScores {
    pub documentation: CategoryScore,          // 20 points
    pub precommit_hooks: CategoryScore,        // 20 points
    pub repository_hygiene: CategoryScore,     // 10 points
    pub build_test_automation: CategoryScore,  // 25 points
    pub continuous_integration: CategoryScore, // 20 points
    pub pmat_compliance: CategoryScore,        // 5 points
}

/// Individual category score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryScore {
    pub score: f64,          // Earned points
    pub max_score: f64,      // Maximum possible
    pub percentage: f64,     // score/max_score * 100
    pub status: ScoreStatus, // Pass, Warning, Fail
    pub subcategories: Vec<SubcategoryScore>,
    pub findings: Vec<Finding>,
}

/// Subcategory breakdown (e.g., A1, A2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubcategoryScore {
    pub id: String,   // "A1", "A2", etc.
    pub name: String, // "README Accuracy"
    pub score: f64,
    pub max_score: f64,
    pub findings: Vec<Finding>,
}

/// Bonus points (0-10 max)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonusScores {
    pub property_tests: BonusItem,   // +3 max
    pub fuzzing: BonusItem,          // +2 max
    pub mutation_testing: BonusItem, // +2 max
    pub living_docs: BonusItem,      // +3 max
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Bonus item.
pub struct BonusItem {
    pub points: f64,
    pub max_points: f64,
    pub detected: bool,
    pub evidence: Vec<String>,
}

/// Finding (positive or negative)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub severity: Severity,
    pub category: String,
    pub message: String,
    pub location: Option<String>, // File path or line number
    pub impact_points: f64,       // Points lost/gained
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
/// Severity level classification for severity.
pub enum Severity {
    Success, // Green - criterion met
    Warning, // Yellow - partial compliance
    Error,   // Red - criterion failed
    Info,    // Blue - informational
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
/// Status of score operation.
pub enum ScoreStatus {
    Pass,    // >=90% of max
    Warning, // 70-89% of max
    Fail,    // <70% of max
}

/// Recommendation for improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub priority: Priority,
    pub category: String,
    pub title: String,
    pub description: String,
    pub impact_points: f64,       // Potential score improvement
    pub estimated_effort: String, // "15 minutes", "2 hours", "1 week"
    pub commands: Vec<String>,    // Shell commands to execute
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
/// Priority level for priority.
pub enum Priority {
    Critical, // Blocks production readiness
    High,     // Important for quality
    Medium,   // Nice to have
    Low,      // Minor improvement
}

/// Metadata about the scoring run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreMetadata {
    pub timestamp: String, // ISO 8601
    pub repository_path: PathBuf,
    pub git_branch: Option<String>,
    pub git_commit: Option<String>,
    pub pmat_version: String,
    pub spec_version: String, // "1.0.0"
    pub execution_time_ms: u64,
}

// --- impl blocks and methods ---
include!("models_impls.rs");

// --- tests ---
include!("models_tests.rs");

// ── Kani proof harnesses (GH-276) ────────────────────────────────────────────
// See kani/README.md for run instructions.
#[cfg(kani)]
mod kani_proofs {
    use super::{CategoryScore, Grade, ScoreStatus};

    /// `Grade::from_score` is total on bounded finite f64 inputs.
    #[kani::proof]
    fn grade_from_score_total() {
        let s: f64 = kani::any();
        kani::assume(s.is_finite());
        kani::assume((-1000.0..=1000.0).contains(&s));
        let g = Grade::from_score(s);
        let _ok = matches!(
            g,
            Grade::APlus
                | Grade::A
                | Grade::AMinus
                | Grade::BPlus
                | Grade::B
                | Grade::C
                | Grade::D
                | Grade::F
        );
        assert!(_ok);
    }

    /// Scores >= 95 classify as A+. Band-boundary safety proof.
    #[kani::proof]
    fn score_ge_95_is_a_plus() {
        let s: f64 = kani::any();
        kani::assume(s.is_finite());
        kani::assume((95.0..=1000.0).contains(&s));
        assert!(matches!(Grade::from_score(s), Grade::APlus));
    }

    /// `CategoryScore::new` percentage invariant:
    /// when `max_score > 0`, the computed percentage equals `score/max * 100`.
    /// (Kani handles f64 arithmetic exactly for finite operands.)
    #[kani::proof]
    fn category_score_percentage_invariant() {
        let score: f64 = kani::any();
        let max_score: f64 = kani::any();
        kani::assume(score.is_finite() && max_score.is_finite());
        kani::assume((0.0..=1000.0).contains(&score));
        kani::assume((1.0..=1000.0).contains(&max_score));
        let cs = CategoryScore::new(score, max_score, Vec::new(), Vec::new());
        // Percentage is exactly score/max*100 for positive max_score.
        let expected = (score / max_score) * 100.0;
        assert!(cs.percentage == expected);
        // Status monotonically follows percentage thresholds.
        let status_matches = match cs.status {
            ScoreStatus::Pass => expected >= 90.0,
            ScoreStatus::Warning => expected >= 70.0 && expected < 90.0,
            ScoreStatus::Fail => expected < 70.0,
        };
        assert!(status_matches);
    }

    /// When `max_score == 0`, the percentage is 0 (division-by-zero guard).
    #[kani::proof]
    fn category_score_zero_max_is_safe() {
        let score: f64 = kani::any();
        kani::assume(score.is_finite());
        kani::assume((0.0..=1000.0).contains(&score));
        let cs = CategoryScore::new(score, 0.0, Vec::new(), Vec::new());
        assert!(cs.percentage == 0.0);
        assert!(matches!(cs.status, ScoreStatus::Fail));
    }
}
