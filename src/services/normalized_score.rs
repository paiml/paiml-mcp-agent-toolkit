#![cfg_attr(coverage_nightly, coverage(off))]
//! Normalized Score System (PMAT-454)
//!
//! All PMAT scoring systems MUST output values in the 0-100 range.
//! This module provides the trait and utilities to ensure consistent scoring.
//!
//! # Design Principles
//! - All scores are normalized to 0.0-100.0 range
//! - Raw scores can use any internal scale (106, 110, 200 points)
//! - `normalized()` method always returns 0-100
//! - Clamping ensures no out-of-range values

use std::fmt;

/// Trait for all scoring systems in PMAT.
///
/// Implementors MUST ensure `normalized()` returns values in [0.0, 100.0].
pub trait NormalizedScore: fmt::Display {
    /// Returns the raw score value (internal scale).
    fn raw(&self) -> f64;

    /// Returns the maximum possible raw score.
    fn max_raw(&self) -> f64;

    /// Returns the normalized score in 0-100 range.
    ///
    /// # Guarantees
    /// - Always returns a value in [0.0, 100.0]
    /// - Values are clamped if raw calculation exceeds bounds
    fn normalized(&self) -> f64 {
        let max = self.max_raw();
        if max <= 0.0 {
            return 0.0;
        }
        let normalized = (self.raw() / max) * 100.0;
        normalized.clamp(0.0, 100.0)
    }

    /// Returns the letter grade based on normalized score.
    fn grade(&self) -> Grade {
        Grade::from_score(self.normalized())
    }

    /// Returns true if score meets the given threshold (0-100).
    fn meets_threshold(&self, threshold: f64) -> bool {
        self.normalized() >= threshold.clamp(0.0, 100.0)
    }
}

/// Universal letter grades for all scoring systems.
/// Ordering: A > B > C > D > F (higher grade = better)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Grade {
    /// 90-100: Excellent
    A,
    /// 80-89: Good
    B,
    /// 70-79: Satisfactory
    C,
    /// 60-69: Needs Improvement
    D,
    /// 0-59: Failing
    F,
}

impl PartialOrd for Grade {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Grade {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.min_score()
            .partial_cmp(&other.min_score())
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl Grade {
    /// Convert a normalized score (0-100) to a grade.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s >= 90.0 => Grade::A,
            s if s >= 80.0 => Grade::B,
            s if s >= 70.0 => Grade::C,
            s if s >= 60.0 => Grade::D,
            _ => Grade::F,
        }
    }

    /// Returns the minimum score for this grade.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    pub fn min_score(&self) -> f64 {
        match self {
            Grade::A => 90.0,
            Grade::B => 80.0,
            Grade::C => 70.0,
            Grade::D => 60.0,
            Grade::F => 0.0,
        }
    }

    /// Returns the grade as a string with description.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn description(&self) -> &'static str {
        match self {
            Grade::A => "A (Excellent)",
            Grade::B => "B (Good)",
            Grade::C => "C (Satisfactory)",
            Grade::D => "D (Needs Improvement)",
            Grade::F => "F (Failing)",
        }
    }
}

impl fmt::Display for Grade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Grade::A => write!(f, "A"),
            Grade::B => write!(f, "B"),
            Grade::C => write!(f, "C"),
            Grade::D => write!(f, "D"),
            Grade::F => write!(f, "F"),
        }
    }
}

/// Helper struct for creating normalized scores from raw values.
#[derive(Debug, Clone, Copy)]
pub struct SimpleScore {
    raw: f64,
    max: f64,
    name: &'static str,
}

impl SimpleScore {
    /// Create a new simple score.
    ///
    /// # Panics
    /// Panics if max <= 0.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(raw: f64, max: f64, name: &'static str) -> Self {
        assert!(max > 0.0, "max must be positive");
        Self {
            raw: raw.max(0.0),
            max,
            name,
        }
    }

    /// Create from a percentage (0-100).
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn from_percentage(pct: f64, name: &'static str) -> Self {
        Self {
            raw: pct.clamp(0.0, 100.0),
            max: 100.0,
            name,
        }
    }
}

impl NormalizedScore for SimpleScore {
    fn raw(&self) -> f64 {
        self.raw
    }

    fn max_raw(&self) -> f64 {
        self.max
    }
}

impl fmt::Display for SimpleScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {:.1}/100 ({})",
            self.name,
            self.normalized(),
            self.grade()
        )
    }
}

// Aggregate scoring: AggregateScore, NormalizedScoreClone trait
include!("normalized_score_aggregate.rs");

// Tests
include!("normalized_score_tests.rs");
