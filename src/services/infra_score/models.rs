#![cfg_attr(coverage_nightly, coverage(off))]
//! Data models for pmat infra-score
//! Implements the scoring system defined in docs/specifications/components/infra-score.md
//!
//! 5 dimensions, 100 points total. Hard cutoff: <90 = auto-fail.

use crate::services::normalized_score::NormalizedScore;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

/// Maximum possible raw points for Infra Score (no bonuses — strict 100)
pub const INFRA_SCORE_MAX_POINTS: f64 = 100.0;

/// Overall infrastructure score result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfraScore {
    /// Raw score (0-100)
    pub total_score: f64,
    pub grade: InfraGrade,
    pub auto_fail: bool,
    pub categories: InfraCategoryScores,
    pub recommendations: Vec<InfraRecommendation>,
    pub metadata: InfraScoreMetadata,
}

/// Infra-specific grade with hard cutoff semantics
/// A+ (95-100), A (90-94), B (80-89) AUTO-FAIL, C (60-79) AUTO-FAIL, D (40-59) AUTO-FAIL, F (0-39) AUTO-FAIL
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum InfraGrade {
    APlus, // 95-100
    A,     // 90-94
    B,     // 80-89 — AUTO-FAIL
    C,     // 60-79 — AUTO-FAIL
    D,     // 40-59 — AUTO-FAIL
    F,     // 0-39  — AUTO-FAIL
}

/// Category scores (100 points total)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfraCategoryScores {
    pub workflow_architecture: InfraCategoryScore, // 25 points
    pub build_reliability: InfraCategoryScore,     // 25 points
    pub quality_pipeline: InfraCategoryScore,      // 20 points
    pub deployment_release: InfraCategoryScore,    // 15 points
    pub supply_chain: InfraCategoryScore,          // 15 points
    pub provable_contracts: InfraCategoryScore,    // 10 points (bonus)
}

/// Individual category score (mirrors repo_score CategoryScore)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfraCategoryScore {
    pub score: f64,
    pub max_score: f64,
    pub percentage: f64,
    pub checks: Vec<InfraCheck>,
    pub findings: Vec<InfraFinding>,
}

/// Individual check result (e.g., WA-01, BR-03)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfraCheck {
    pub id: String,     // "WA-01", "BR-03", etc.
    pub name: String,   // Human-readable name
    pub score: f64,     // Earned points
    pub max_score: f64, // Maximum possible
    pub passed: bool,
    pub evidence: Vec<String>,
}

/// Finding with severity (mirrors repo_score Finding)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfraFinding {
    pub severity: InfraSeverity,
    pub check_id: String,
    pub message: String,
    pub location: Option<String>,
    pub impact_points: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum InfraSeverity {
    Pass,    // Check passed
    Warning, // Partial compliance
    Fail,    // Check failed
    Info,    // Informational
}

/// Recommendation for improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfraRecommendation {
    pub priority: InfraPriority,
    pub check_id: String,
    pub title: String,
    pub description: String,
    pub impact_points: f64,
    pub estimated_effort: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum InfraPriority {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Metadata about the scoring run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfraScoreMetadata {
    pub timestamp: String,
    pub repository_path: PathBuf,
    pub git_branch: Option<String>,
    pub git_commit: Option<String>,
    pub pmat_version: String,
    pub spec_version: String,
    pub execution_time_ms: u64,
}

// --- impl blocks ---

impl NormalizedScore for InfraScore {
    fn raw(&self) -> f64 {
        self.total_score
    }

    fn max_raw(&self) -> f64 {
        INFRA_SCORE_MAX_POINTS
    }
}

impl fmt::Display for InfraScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fail_marker = if self.auto_fail { " [AUTO-FAIL]" } else { "" };
        write!(
            f,
            "Infra Score: {:.1}/100 ({}){}",
            self.total_score,
            self.grade.as_str(),
            fail_marker,
        )
    }
}

impl InfraGrade {
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s >= 95.0 => InfraGrade::APlus,
            s if s >= 90.0 => InfraGrade::A,
            s if s >= 80.0 => InfraGrade::B,
            s if s >= 60.0 => InfraGrade::C,
            s if s >= 40.0 => InfraGrade::D,
            _ => InfraGrade::F,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            InfraGrade::APlus => "A+",
            InfraGrade::A => "A",
            InfraGrade::B => "B",
            InfraGrade::C => "C",
            InfraGrade::D => "D",
            InfraGrade::F => "F",
        }
    }

    /// Returns true if this grade is an auto-fail (<90)
    pub fn is_auto_fail(&self) -> bool {
        !matches!(self, InfraGrade::APlus | InfraGrade::A)
    }
}

impl InfraCategoryScores {
    /// Total base score (100 points max, excluding bonus)
    pub fn total(&self) -> f64 {
        self.workflow_architecture.score
            + self.build_reliability.score
            + self.quality_pipeline.score
            + self.deployment_release.score
            + self.supply_chain.score
    }

    /// Total including provable contracts bonus (110 max)
    pub fn total_with_bonus(&self) -> f64 {
        self.total() + self.provable_contracts.score
    }
}

impl Default for InfraCategoryScores {
    fn default() -> Self {
        Self {
            workflow_architecture: InfraCategoryScore::empty(25.0),
            build_reliability: InfraCategoryScore::empty(25.0),
            quality_pipeline: InfraCategoryScore::empty(20.0),
            deployment_release: InfraCategoryScore::empty(15.0),
            supply_chain: InfraCategoryScore::empty(15.0),
            provable_contracts: InfraCategoryScore::empty(10.0),
        }
    }
}

impl InfraCategoryScore {
    pub fn empty(max_score: f64) -> Self {
        Self {
            score: 0.0,
            max_score,
            percentage: 0.0,
            checks: vec![],
            findings: vec![],
        }
    }

    pub fn new(max_score: f64, checks: Vec<InfraCheck>, findings: Vec<InfraFinding>) -> Self {
        let score: f64 = checks.iter().map(|c| c.score).sum();
        let percentage = if max_score > 0.0 {
            (score / max_score) * 100.0
        } else {
            0.0
        };

        Self {
            score,
            max_score,
            percentage,
            checks,
            findings,
        }
    }
}

impl InfraCheck {
    pub fn pass(id: &str, name: &str, max_score: f64, evidence: Vec<String>) -> Self {
        debug_assert!(!id.is_empty(), "id must not be empty");
        debug_assert!(!name.is_empty(), "name must not be empty");
        Self {
            id: id.to_string(),
            name: name.to_string(),
            score: max_score,
            max_score,
            passed: true,
            evidence,
        }
    }

    pub fn fail(id: &str, name: &str, max_score: f64, evidence: Vec<String>) -> Self {
        debug_assert!(!id.is_empty(), "id must not be empty");
        debug_assert!(!name.is_empty(), "name must not be empty");
        Self {
            id: id.to_string(),
            name: name.to_string(),
            score: 0.0,
            max_score,
            passed: false,
            evidence,
        }
    }

    pub fn partial(
        id: &str,
        name: &str,
        score: f64,
        max_score: f64,
        evidence: Vec<String>,
    ) -> Self {
        debug_assert!(!id.is_empty(), "id must not be empty");
        debug_assert!(!name.is_empty(), "name must not be empty");
        Self {
            id: id.to_string(),
            name: name.to_string(),
            score,
            max_score,
            passed: score >= max_score,
            evidence,
        }
    }
}

impl InfraScoreMetadata {
    pub fn new(repository_path: PathBuf) -> Self {
        debug_assert!(
            repository_path.exists(),
            "repository_path must exist: {}",
            repository_path.display()
        );
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            repository_path,
            git_branch: None,
            git_commit: None,
            pmat_version: env!("CARGO_PKG_VERSION").to_string(),
            spec_version: "0.1.0".to_string(),
            execution_time_ms: 0,
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infra_grade_from_score() {
        assert_eq!(InfraGrade::from_score(100.0), InfraGrade::APlus);
        assert_eq!(InfraGrade::from_score(95.0), InfraGrade::APlus);
        assert_eq!(InfraGrade::from_score(94.0), InfraGrade::A);
        assert_eq!(InfraGrade::from_score(90.0), InfraGrade::A);
        assert_eq!(InfraGrade::from_score(89.0), InfraGrade::B);
        assert_eq!(InfraGrade::from_score(80.0), InfraGrade::B);
        assert_eq!(InfraGrade::from_score(79.0), InfraGrade::C);
        assert_eq!(InfraGrade::from_score(60.0), InfraGrade::C);
        assert_eq!(InfraGrade::from_score(59.0), InfraGrade::D);
        assert_eq!(InfraGrade::from_score(40.0), InfraGrade::D);
        assert_eq!(InfraGrade::from_score(39.0), InfraGrade::F);
        assert_eq!(InfraGrade::from_score(0.0), InfraGrade::F);
    }

    #[test]
    fn test_infra_grade_auto_fail() {
        assert!(!InfraGrade::APlus.is_auto_fail());
        assert!(!InfraGrade::A.is_auto_fail());
        assert!(InfraGrade::B.is_auto_fail());
        assert!(InfraGrade::C.is_auto_fail());
        assert!(InfraGrade::D.is_auto_fail());
        assert!(InfraGrade::F.is_auto_fail());
    }

    #[test]
    fn test_infra_grade_as_str() {
        assert_eq!(InfraGrade::APlus.as_str(), "A+");
        assert_eq!(InfraGrade::A.as_str(), "A");
        assert_eq!(InfraGrade::B.as_str(), "B");
        assert_eq!(InfraGrade::C.as_str(), "C");
        assert_eq!(InfraGrade::D.as_str(), "D");
        assert_eq!(InfraGrade::F.as_str(), "F");
    }

    #[test]
    fn test_infra_category_scores_total() {
        let scores = InfraCategoryScores {
            workflow_architecture: InfraCategoryScore {
                score: 20.0,
                max_score: 25.0,
                percentage: 80.0,
                checks: vec![],
                findings: vec![],
            },
            build_reliability: InfraCategoryScore {
                score: 22.0,
                max_score: 25.0,
                percentage: 88.0,
                checks: vec![],
                findings: vec![],
            },
            quality_pipeline: InfraCategoryScore {
                score: 18.0,
                max_score: 20.0,
                percentage: 90.0,
                checks: vec![],
                findings: vec![],
            },
            deployment_release: InfraCategoryScore {
                score: 12.0,
                max_score: 15.0,
                percentage: 80.0,
                checks: vec![],
                findings: vec![],
            },
            supply_chain: InfraCategoryScore {
                score: 10.0,
                max_score: 15.0,
                percentage: 66.7,
                checks: vec![],
                findings: vec![],
            },
            provable_contracts: InfraCategoryScore {
                score: 5.0,
                max_score: 10.0,
                percentage: 50.0,
                checks: vec![],
                findings: vec![],
            },
        };
        assert!((scores.total() - 82.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_infra_check_pass() {
        let check = InfraCheck::pass(
            "WA-01",
            "Reusable workflow",
            5.0,
            vec!["found uses: org/.github".to_string()],
        );
        assert!(check.passed);
        assert!((check.score - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_infra_check_fail() {
        let check = InfraCheck::fail(
            "WA-01",
            "Reusable workflow",
            5.0,
            vec!["no reusable workflow found".to_string()],
        );
        assert!(!check.passed);
        assert!((check.score - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_infra_check_partial() {
        let check = InfraCheck::partial(
            "BR-01",
            "CI success rate",
            3.0,
            5.0,
            vec!["7/10 passed".to_string()],
        );
        assert!(!check.passed);
        assert!((check.score - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_infra_category_score_new() {
        let checks = vec![
            InfraCheck::pass("WA-01", "Reusable", 5.0, vec![]),
            InfraCheck::fail("WA-02", "Self-hosted", 5.0, vec![]),
        ];
        let cat = InfraCategoryScore::new(25.0, checks, vec![]);
        assert!((cat.score - 5.0).abs() < f64::EPSILON);
        assert!((cat.percentage - 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_infra_score_display() {
        let score = InfraScore {
            total_score: 92.0,
            grade: InfraGrade::A,
            auto_fail: false,
            categories: InfraCategoryScores::default(),
            recommendations: vec![],
            metadata: InfraScoreMetadata::new(PathBuf::from("/tmp/test")),
        };
        let display = format!("{}", score);
        assert!(display.contains("92.0"));
        assert!(display.contains("A"));
        assert!(!display.contains("AUTO-FAIL"));
    }

    #[test]
    fn test_infra_score_display_auto_fail() {
        let score = InfraScore {
            total_score: 75.0,
            grade: InfraGrade::C,
            auto_fail: true,
            categories: InfraCategoryScores::default(),
            recommendations: vec![],
            metadata: InfraScoreMetadata::new(PathBuf::from("/tmp/test")),
        };
        let display = format!("{}", score);
        assert!(display.contains("AUTO-FAIL"));
    }

    #[test]
    fn test_normalized_score_trait() {
        let score = InfraScore {
            total_score: 85.0,
            grade: InfraGrade::B,
            auto_fail: true,
            categories: InfraCategoryScores::default(),
            recommendations: vec![],
            metadata: InfraScoreMetadata::new(PathBuf::from("/tmp/test")),
        };
        assert!((score.raw() - 85.0).abs() < f64::EPSILON);
        assert!((score.max_raw() - 100.0).abs() < f64::EPSILON);
        assert!((score.normalized() - 85.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_metadata_new() {
        let meta = InfraScoreMetadata::new(PathBuf::from("/tmp/repo"));
        assert_eq!(meta.repository_path, PathBuf::from("/tmp/repo"));
        assert_eq!(meta.spec_version, "0.1.0");
        assert!(meta.git_branch.is_none());
        assert!(meta.git_commit.is_none());
    }
}
