//! Core data models for Rust Project Score v1.1
//!
//! This module defines the core types for the 106-point scoring system
//! with 6 categories following the evidence-based specification.
//!
//! Evidence-based design from 15 peer-reviewed papers (2022-2025)

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// RustProjectScore - Main Score Container
// ============================================================================

/// Comprehensive Rust project quality score (v1.1)
///
/// Total score: 0-106 points across 6 categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustProjectScore {
    /// Total score (0-106 points)
    pub total_score: f64,

    /// Letter grade (A+ to F)
    pub grade: Grade,

    /// Breakdown by category
    pub categories: CategoryScores,

    /// Actionable recommendations
    pub recommendations: Vec<Recommendation>,

    /// Metadata (timestamp, project, version)
    pub metadata: ScoreMetadata,

    /// Score velocity (Kaizen tracking) - NEW in v1.1
    pub velocity: Option<ScoreVelocity>,
}

impl RustProjectScore {
    /// Create a new score with zero values
    pub fn new() -> Self {
        Self {
            total_score: 0.0,
            grade: Grade::F,
            categories: CategoryScores::default(),
            recommendations: Vec::new(),
            metadata: ScoreMetadata::new("unknown".to_string(), "1.1.0".to_string()),
            velocity: None,
        }
    }
}

impl Default for RustProjectScore {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Grade - Letter Grade Enum
// ============================================================================

/// Letter grade based on percentage of total possible points
///
/// Thresholds:
/// - A+ : 95-106 (89.6%+)
/// - A  : 90-94  (84.9%-89.5%)
/// - A- : 85-89  (80.2%-84.8%)
/// - B+ : 80-84  (75.5%-80.1%)
/// - B  : 70-79  (66.0%-75.4%)
/// - C  : 60-69
/// - D  : 50-59
/// - F  : 0-49
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Grade {
    APlus,
    A,
    AMinus,
    BPlus,
    B,
    C,
    D,
    F,
}

impl Grade {
    /// Calculate grade from score and max possible points
    pub fn from_score(score: f64, _max: f64) -> Self {
        if score >= 95.0 {
            Grade::APlus
        } else if score >= 90.0 {
            Grade::A
        } else if score >= 85.0 {
            Grade::AMinus
        } else if score >= 80.0 {
            Grade::BPlus
        } else if score >= 70.0 {
            Grade::B
        } else if score >= 60.0 {
            Grade::C
        } else if score >= 50.0 {
            Grade::D
        } else {
            Grade::F
        }
    }
}

impl fmt::Display for Grade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Grade::APlus => write!(f, "A+"),
            Grade::A => write!(f, "A"),
            Grade::AMinus => write!(f, "A-"),
            Grade::BPlus => write!(f, "B+"),
            Grade::B => write!(f, "B"),
            Grade::C => write!(f, "C"),
            Grade::D => write!(f, "D"),
            Grade::F => write!(f, "F"),
        }
    }
}

// ============================================================================
// CategoryScores - 6 Scoring Categories
// ============================================================================

/// Six scoring categories (106 points total)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryScores {
    /// Rust tooling compliance (25pts)
    pub rust_tooling: CategoryScore,

    /// Code quality (26pts)
    pub code_quality: CategoryScore,

    /// Testing excellence (20pts)
    pub testing: CategoryScore,

    /// Documentation (15pts)
    pub documentation: CategoryScore,

    /// Performance & benchmarking (10pts)
    pub performance: CategoryScore,

    /// Dependency health (12pts)
    pub dependencies: CategoryScore,
}

impl CategoryScores {
    /// Calculate total score across all categories
    pub fn total(&self) -> f64 {
        self.rust_tooling.earned
            + self.code_quality.earned
            + self.testing.earned
            + self.documentation.earned
            + self.performance.earned
            + self.dependencies.earned
    }
}

impl Default for CategoryScores {
    fn default() -> Self {
        Self {
            rust_tooling: CategoryScore::new(0.0, 25.0),
            code_quality: CategoryScore::new(0.0, 26.0),
            testing: CategoryScore::new(0.0, 20.0),
            documentation: CategoryScore::new(0.0, 15.0),
            performance: CategoryScore::new(0.0, 10.0),
            dependencies: CategoryScore::new(0.0, 12.0),
        }
    }
}

// ============================================================================
// CategoryScore - Individual Category Metrics
// ============================================================================

/// Score for an individual category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryScore {
    /// Points earned
    pub earned: f64,

    /// Maximum possible points
    pub max: f64,
}

impl CategoryScore {
    /// Create a new category score
    pub fn new(earned: f64, max: f64) -> Self {
        Self { earned, max }
    }

    /// Calculate percentage (0-100)
    pub fn percentage(&self) -> f64 {
        if self.max == 0.0 {
            0.0
        } else {
            (self.earned / self.max) * 100.0
        }
    }

    /// Check if category has perfect score
    pub fn is_perfect(&self) -> bool {
        (self.earned - self.max).abs() < 0.01
    }
}

// ============================================================================
// ScoreVelocity - Kaizen Continuous Improvement Tracking (NEW in v1.1)
// ============================================================================

/// Kaizen: Continuous improvement tracking
///
/// Tracks score changes over time to encourage incremental progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreVelocity {
    /// Current score
    pub current: f64,

    /// Previous score (from baseline)
    pub previous: f64,

    /// Change in points
    pub delta: f64,

    /// Change as percentage
    pub delta_percent: f64,

    /// Days since baseline
    pub days_elapsed: u64,

    /// Points per day improvement rate
    pub points_per_day: f64,

    /// Most improved category
    pub most_improved: Option<String>,

    /// Projected days to next grade
    pub days_to_next_grade: Option<u64>,
}

impl ScoreVelocity {
    /// Calculate velocity from previous and current scores
    ///
    /// # Arguments
    /// * `previous` - Previous baseline score
    /// * `current` - Current score
    /// * `days` - Days elapsed since baseline
    pub fn calculate(previous: f64, current: f64, days: u64) -> Self {
        let delta = current - previous;
        let delta_percent = if previous == 0.0 {
            0.0
        } else {
            (delta / previous) * 100.0
        };

        let points_per_day = if days == 0 { 0.0 } else { delta / days as f64 };

        Self {
            current,
            previous,
            delta,
            delta_percent,
            days_elapsed: days,
            points_per_day,
            most_improved: None,
            days_to_next_grade: None,
        }
    }
}

// ============================================================================
// Recommendation - Actionable Improvement Suggestions
// ============================================================================

/// Priority level for recommendations
///
/// Ordered from highest to lowest priority for sorting
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Actionable recommendation for improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Category this recommendation applies to
    pub category: String,

    /// Description of the recommendation
    pub description: String,

    /// Priority level
    pub priority: RecommendationPriority,

    /// Potential points to gain
    pub potential_points: f64,
}

impl Recommendation {
    /// Create a new recommendation
    pub fn new(
        category: String,
        description: String,
        priority: RecommendationPriority,
        potential_points: f64,
    ) -> Self {
        Self {
            category,
            description,
            priority,
            potential_points,
        }
    }
}

// ============================================================================
// ScoreMetadata - Project Information
// ============================================================================

/// Metadata about the scoring analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreMetadata {
    /// Timestamp of analysis
    pub timestamp: String,

    /// Project name
    pub project_name: String,

    /// Specification version
    pub version: String,
}

impl ScoreMetadata {
    /// Create new metadata
    pub fn new(project_name: String, version: String) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            project_name,
            version,
        }
    }
}
