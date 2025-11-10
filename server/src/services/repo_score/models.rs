// Data models for pmat repo-score
// Implements the scoring system defined in docs/specifications/repo-score-spec.md

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Overall repository score result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoScore {
    pub total_score: f64,          // 0-100 base score
    pub bonus_points: f64,         // 0-10 bonus
    pub final_score: f64,          // total + bonus (max 110)
    pub grade: Grade,              // A+, A, A-, B+, etc.
    pub categories: CategoryScores,
    pub bonus: BonusScores,
    pub recommendations: Vec<Recommendation>,
    pub metadata: ScoreMetadata,
}

/// Letter grade assignment
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Grade {
    APlus,   // 95-110
    A,       // 90-94
    AMinus,  // 85-89
    BPlus,   // 80-84
    B,       // 70-79
    C,       // 60-69
    D,       // 50-59
    F,       // 0-49
}

impl Grade {
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s >= 95.0 => Grade::APlus,
            s if s >= 90.0 => Grade::A,
            s if s >= 85.0 => Grade::AMinus,
            s if s >= 80.0 => Grade::BPlus,
            s if s >= 70.0 => Grade::B,
            s if s >= 60.0 => Grade::C,
            s if s >= 50.0 => Grade::D,
            _ => Grade::F,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Grade::APlus => "A+",
            Grade::A => "A",
            Grade::AMinus => "A-",
            Grade::BPlus => "B+",
            Grade::B => "B",
            Grade::C => "C",
            Grade::D => "D",
            Grade::F => "F",
        }
    }
}

/// Category scores (base 100 points)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryScores {
    pub documentation: CategoryScore,           // 20 points
    pub precommit_hooks: CategoryScore,         // 20 points
    pub repository_hygiene: CategoryScore,      // 10 points
    pub build_test_automation: CategoryScore,   // 25 points
    pub continuous_integration: CategoryScore,  // 20 points
    pub pmat_compliance: CategoryScore,         // 5 points
}

impl CategoryScores {
    pub fn total(&self) -> f64 {
        self.documentation.score
            + self.precommit_hooks.score
            + self.repository_hygiene.score
            + self.build_test_automation.score
            + self.continuous_integration.score
            + self.pmat_compliance.score
    }
}

impl Default for CategoryScores {
    fn default() -> Self {
        Self {
            documentation: CategoryScore::default_with_max(20.0),
            precommit_hooks: CategoryScore::default_with_max(20.0),
            repository_hygiene: CategoryScore::default_with_max(10.0),
            build_test_automation: CategoryScore::default_with_max(25.0),
            continuous_integration: CategoryScore::default_with_max(20.0),
            pmat_compliance: CategoryScore::default_with_max(5.0),
        }
    }
}

/// Individual category score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryScore {
    pub score: f64,           // Earned points
    pub max_score: f64,       // Maximum possible
    pub percentage: f64,      // score/max_score * 100
    pub status: ScoreStatus,  // Pass, Warning, Fail
    pub subcategories: Vec<SubcategoryScore>,
    pub findings: Vec<Finding>,
}

impl CategoryScore {
    fn default_with_max(max_score: f64) -> Self {
        Self {
            score: 0.0,
            max_score,
            percentage: 0.0,
            status: ScoreStatus::Fail,
            subcategories: vec![],
            findings: vec![],
        }
    }

    pub fn new(score: f64, max_score: f64, subcategories: Vec<SubcategoryScore>, findings: Vec<Finding>) -> Self {
        let percentage = if max_score > 0.0 {
            (score / max_score) * 100.0
        } else {
            0.0
        };

        let status = if percentage >= 90.0 {
            ScoreStatus::Pass
        } else if percentage >= 70.0 {
            ScoreStatus::Warning
        } else {
            ScoreStatus::Fail
        };

        Self {
            score,
            max_score,
            percentage,
            status,
            subcategories,
            findings,
        }
    }
}

/// Subcategory breakdown (e.g., A1, A2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubcategoryScore {
    pub id: String,           // "A1", "A2", etc.
    pub name: String,         // "README Accuracy"
    pub score: f64,
    pub max_score: f64,
    pub findings: Vec<Finding>,
}

/// Bonus points (0-10 max)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonusScores {
    pub property_tests: BonusItem,      // +3 max
    pub fuzzing: BonusItem,             // +2 max
    pub mutation_testing: BonusItem,    // +2 max
    pub living_docs: BonusItem,         // +3 max
}

impl BonusScores {
    pub fn total(&self) -> f64 {
        self.property_tests.points
            + self.fuzzing.points
            + self.mutation_testing.points
            + self.living_docs.points
    }
}

impl Default for BonusScores {
    fn default() -> Self {
        Self {
            property_tests: BonusItem {
                points: 0.0,
                max_points: 3.0,
                detected: false,
                evidence: vec![],
            },
            fuzzing: BonusItem {
                points: 0.0,
                max_points: 2.0,
                detected: false,
                evidence: vec![],
            },
            mutation_testing: BonusItem {
                points: 0.0,
                max_points: 2.0,
                detected: false,
                evidence: vec![],
            },
            living_docs: BonusItem {
                points: 0.0,
                max_points: 3.0,
                detected: false,
                evidence: vec![],
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub location: Option<String>,  // File path or line number
    pub impact_points: f64,        // Points lost/gained
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity {
    Success,   // ✅ Green - criterion met
    Warning,   // ⚠️  Yellow - partial compliance
    Error,     // ❌ Red - criterion failed
    Info,      // ℹ️  Blue - informational
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ScoreStatus {
    Pass,      // ≥90% of max
    Warning,   // 70-89% of max
    Fail,      // <70% of max
}

/// Recommendation for improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub priority: Priority,
    pub category: String,
    pub title: String,
    pub description: String,
    pub impact_points: f64,        // Potential score improvement
    pub estimated_effort: String,  // "15 minutes", "2 hours", "1 week"
    pub commands: Vec<String>,     // Shell commands to execute
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Priority {
    Critical,  // Blocks production readiness
    High,      // Important for quality
    Medium,    // Nice to have
    Low,       // Minor improvement
}

// Manual PartialOrd/Ord implementation for correct ordering
// Critical > High > Medium > Low
impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Priority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_rank = match self {
            Priority::Critical => 4,
            Priority::High => 3,
            Priority::Medium => 2,
            Priority::Low => 1,
        };
        let other_rank = match other {
            Priority::Critical => 4,
            Priority::High => 3,
            Priority::Medium => 2,
            Priority::Low => 1,
        };
        self_rank.cmp(&other_rank)
    }
}

/// Metadata about the scoring run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreMetadata {
    pub timestamp: String,          // ISO 8601
    pub repository_path: PathBuf,
    pub git_branch: Option<String>,
    pub git_commit: Option<String>,
    pub pmat_version: String,
    pub spec_version: String,       // "1.0.0"
    pub execution_time_ms: u64,
}

impl ScoreMetadata {
    pub fn new(repository_path: PathBuf) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            repository_path,
            git_branch: None,
            git_commit: None,
            pmat_version: env!("CARGO_PKG_VERSION").to_string(),
            spec_version: "1.0.0".to_string(),
            execution_time_ms: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grade_from_score_a_plus() {
        let scores = vec![95.0, 100.0, 105.0, 110.0];
        for score in scores {
            let grade = Grade::from_score(score);
            assert_eq!(grade, Grade::APlus);
            assert_eq!(grade.as_str(), "A+");
        }
    }

    #[test]
    fn test_grade_from_score_a() {
        let scores = vec![90.0, 91.5, 93.9, 94.9];
        for score in scores {
            let grade = Grade::from_score(score);
            assert_eq!(grade, Grade::A);
            assert_eq!(grade.as_str(), "A");
        }
    }

    #[test]
    fn test_grade_from_score_a_minus() {
        let scores = vec![85.0, 87.0, 89.9];
        for score in scores {
            let grade = Grade::from_score(score);
            assert_eq!(grade, Grade::AMinus);
            assert_eq!(grade.as_str(), "A-");
        }
    }

    #[test]
    fn test_grade_from_score_b_plus() {
        let scores = vec![80.0, 82.0, 84.9];
        for score in scores {
            let grade = Grade::from_score(score);
            assert_eq!(grade, Grade::BPlus);
        }
    }

    #[test]
    fn test_grade_from_score_b() {
        let scores = vec![70.0, 75.0, 79.9];
        for score in scores {
            let grade = Grade::from_score(score);
            assert_eq!(grade, Grade::B);
        }
    }

    #[test]
    fn test_grade_from_score_c() {
        let scores = vec![60.0, 65.0, 69.9];
        for score in scores {
            let grade = Grade::from_score(score);
            assert_eq!(grade, Grade::C);
        }
    }

    #[test]
    fn test_grade_from_score_d() {
        let scores = vec![50.0, 55.0, 59.9];
        for score in scores {
            let grade = Grade::from_score(score);
            assert_eq!(grade, Grade::D);
        }
    }

    #[test]
    fn test_grade_from_score_f() {
        let scores = vec![0.0, 25.0, 49.9];
        for score in scores {
            let grade = Grade::from_score(score);
            assert_eq!(grade, Grade::F);
        }
    }

    #[test]
    fn test_grade_boundary_values() {
        assert_eq!(Grade::from_score(95.0), Grade::APlus);
        assert_eq!(Grade::from_score(94.99), Grade::A);
        assert_eq!(Grade::from_score(90.0), Grade::A);
        assert_eq!(Grade::from_score(89.99), Grade::AMinus);
    }

    #[test]
    fn test_category_scores_total() {
        let scores = CategoryScores {
            documentation: CategoryScore::new(18.0, 20.0, vec![], vec![]),
            precommit_hooks: CategoryScore::new(18.0, 20.0, vec![], vec![]),
            repository_hygiene: CategoryScore::new(8.0, 10.0, vec![], vec![]),
            build_test_automation: CategoryScore::new(22.0, 25.0, vec![], vec![]),
            continuous_integration: CategoryScore::new(18.0, 20.0, vec![], vec![]),
            pmat_compliance: CategoryScore::new(5.0, 5.0, vec![], vec![]),
        };

        // 18 + 18 + 8 + 22 + 18 + 5 = 89
        assert_eq!(scores.total(), 89.0);
    }

    #[test]
    fn test_category_scores_max_total() {
        let scores = CategoryScores {
            documentation: CategoryScore::new(20.0, 20.0, vec![], vec![]),
            precommit_hooks: CategoryScore::new(20.0, 20.0, vec![], vec![]),
            repository_hygiene: CategoryScore::new(10.0, 10.0, vec![], vec![]),
            build_test_automation: CategoryScore::new(25.0, 25.0, vec![], vec![]),
            continuous_integration: CategoryScore::new(20.0, 20.0, vec![], vec![]),
            pmat_compliance: CategoryScore::new(5.0, 5.0, vec![], vec![]),
        };

        assert_eq!(scores.total(), 100.0);
    }

    #[test]
    fn test_bonus_scores_total() {
        let bonus = BonusScores {
            property_tests: BonusItem { points: 3.0, max_points: 3.0, detected: true, evidence: vec![] },
            fuzzing: BonusItem { points: 2.0, max_points: 2.0, detected: true, evidence: vec![] },
            mutation_testing: BonusItem { points: 2.0, max_points: 2.0, detected: true, evidence: vec![] },
            living_docs: BonusItem { points: 0.0, max_points: 3.0, detected: false, evidence: vec![] },
        };

        assert_eq!(bonus.total(), 7.0);
    }

    #[test]
    fn test_bonus_scores_max_total() {
        let bonus = BonusScores {
            property_tests: BonusItem { points: 3.0, max_points: 3.0, detected: true, evidence: vec![] },
            fuzzing: BonusItem { points: 2.0, max_points: 2.0, detected: true, evidence: vec![] },
            mutation_testing: BonusItem { points: 2.0, max_points: 2.0, detected: true, evidence: vec![] },
            living_docs: BonusItem { points: 3.0, max_points: 3.0, detected: true, evidence: vec![] },
        };

        assert_eq!(bonus.total(), 10.0);
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Critical > Priority::High);
        assert!(Priority::High > Priority::Medium);
        assert!(Priority::Medium > Priority::Low);
    }

    #[test]
    fn test_score_status_pass() {
        let score = CategoryScore::new(18.0, 20.0, vec![], vec![]);
        assert_eq!(score.percentage, 90.0);
        assert_eq!(score.status, ScoreStatus::Pass);
    }

    #[test]
    fn test_score_status_warning() {
        let score = CategoryScore::new(16.0, 20.0, vec![], vec![]);
        assert_eq!(score.percentage, 80.0);
        assert_eq!(score.status, ScoreStatus::Warning);
    }

    #[test]
    fn test_score_status_fail() {
        let score = CategoryScore::new(10.0, 20.0, vec![], vec![]);
        assert_eq!(score.percentage, 50.0);
        assert_eq!(score.status, ScoreStatus::Fail);
    }
}
