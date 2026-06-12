#![cfg_attr(coverage_nightly, coverage(off))]
//! Types for the Perfection Score service.

use crate::services::normalized_score::NormalizedScore;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Maximum possible perfection score
pub const MAX_PERFECTION_SCORE: u16 = 200;

/// Category weights for the 200-point scale
#[derive(Debug, Clone, Copy)]
pub struct CategoryWeights {
    pub tdg: u16,           // 40 pts (20%)
    pub repo_score: u16,    // 30 pts (15%)
    pub rust_score: u16,    // 30 pts (15%)
    pub popper_score: u16,  // 25 pts (12.5%)
    pub test_coverage: u16, // 25 pts (12.5%)
    pub mutation: u16,      // 20 pts (10%)
    pub documentation: u16, // 15 pts (7.5%)
    pub performance: u16,   // 15 pts (7.5%)
}

impl Default for CategoryWeights {
    fn default() -> Self {
        Self {
            tdg: 40,
            repo_score: 30,
            rust_score: 30,
            popper_score: 25,
            test_coverage: 25,
            mutation: 20,
            documentation: 15,
            performance: 15,
        }
    }
}

/// Individual category score (0-100 normalized to category weight)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryScore {
    pub name: String,
    pub raw_score: f64,     // Original 0-100 score
    pub max_points: u16,    // Max points for this category
    pub earned_points: f64, // Normalized to category weight
    pub grade: String,      // Letter grade for this category
    pub details: Option<String>,
}

impl CategoryScore {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(name: &str, raw_score: f64, max_points: u16) -> Self {
        let max = f64::from(max_points);
        // raw_score is a 0-100 percentage; clamp so a mis-scaled upstream
        // score (e.g. raw points instead of a percentage) can never earn
        // more than the category max or go negative.
        let earned_points = ((raw_score / 100.0) * max).clamp(0.0, max);
        let grade = Self::calculate_grade(raw_score.clamp(0.0, 100.0));
        Self {
            name: name.to_string(),
            raw_score,
            max_points,
            earned_points,
            grade,
            details: None,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// With details.
    pub fn with_details(mut self, details: &str) -> Self {
        self.details = Some(details.to_string());
        self
    }

    fn calculate_grade(score: f64) -> String {
        // Standard academic grading scale (F-A)
        match score as u8 {
            97..=100 => "A+".to_string(),
            93..=96 => "A".to_string(),
            90..=92 => "A-".to_string(),
            87..=89 => "B+".to_string(),
            83..=86 => "B".to_string(),
            80..=82 => "B-".to_string(),
            77..=79 => "C+".to_string(),
            73..=76 => "C".to_string(),
            70..=72 => "C-".to_string(),
            67..=69 => "D+".to_string(),
            63..=66 => "D".to_string(),
            60..=62 => "D-".to_string(),
            _ => "F".to_string(),
        }
    }
}

/// Complete perfection score result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfectionScoreResult {
    pub total_score: f64,
    pub max_score: u16,
    pub grade: String,
    pub categories: Vec<CategoryScore>,
    pub recommendations: Vec<String>,
    pub target_gap: Option<f64>,
}

impl PerfectionScoreResult {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(categories: Vec<CategoryScore>) -> Self {
        let total_score: f64 = categories
            .iter()
            .map(|c| c.earned_points)
            .sum::<f64>()
            .clamp(0.0, f64::from(MAX_PERFECTION_SCORE));
        let max_score = MAX_PERFECTION_SCORE;
        let grade = Self::calculate_overall_grade(total_score);
        let recommendations = Self::generate_recommendations(&categories);

        Self {
            total_score,
            max_score,
            grade,
            categories,
            recommendations,
            target_gap: None,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// With target.
    pub fn with_target(mut self, target: u16) -> Self {
        self.target_gap = Some(f64::from(target) - self.total_score);
        self
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    /// Calculate overall grade.
    pub fn calculate_overall_grade(score: f64) -> String {
        // PMAT-454: Use normalized percentage (0-100) for grading
        let normalized = (score / f64::from(MAX_PERFECTION_SCORE)) * 100.0;
        match normalized as u16 {
            95..=100 => "A+".to_string(),
            90..=94 => "A".to_string(),
            85..=89 => "A-".to_string(),
            80..=84 => "B+".to_string(),
            70..=79 => "B".to_string(),
            60..=69 => "C".to_string(),
            50..=59 => "D".to_string(),
            _ => "F".to_string(),
        }
    }

    fn generate_recommendations(categories: &[CategoryScore]) -> Vec<String> {
        let mut recs = Vec::new();

        for cat in categories {
            let percentage = (cat.earned_points / f64::from(cat.max_points)) * 100.0;
            if percentage < 60.0 {
                recs.push(format!(
                    "🔴 {} is critical ({:.0}%) - prioritize improvement",
                    cat.name, percentage
                ));
            } else if percentage < 80.0 {
                recs.push(format!(
                    "🟡 {} needs attention ({:.0}%)",
                    cat.name, percentage
                ));
            }
        }

        if recs.is_empty() {
            recs.push("✅ All categories are healthy!".to_string());
        }

        recs
    }
}

impl NormalizedScore for PerfectionScoreResult {
    fn raw(&self) -> f64 {
        self.total_score
    }

    fn max_raw(&self) -> f64 {
        f64::from(MAX_PERFECTION_SCORE)
    }
}

impl fmt::Display for PerfectionScoreResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Perfection Score: {:.1}/100 ({}) [raw: {:.0}/{}]",
            self.normalized(),
            self.grade,
            self.total_score,
            MAX_PERFECTION_SCORE
        )
    }
}
