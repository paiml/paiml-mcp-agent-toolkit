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
        // Higher grade = better, so A > B > C > D > F
        // We compare by min_score which gives the correct ordering
        self.min_score()
            .partial_cmp(&other.min_score())
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl Grade {
    /// Convert a normalized score (0-100) to a grade.
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
    pub fn new(raw: f64, max: f64, name: &'static str) -> Self {
        assert!(max > 0.0, "max must be positive");
        Self {
            raw: raw.max(0.0),
            max,
            name,
        }
    }

    /// Create from a percentage (0-100).
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

/// Aggregate multiple scores into a weighted normalized score.
#[derive(Debug, Clone)]
pub struct AggregateScore {
    components: Vec<(Box<dyn NormalizedScoreClone>, f64)>, // (score, weight)
    name: String,
}

/// Helper trait for cloning boxed NormalizedScore.
pub trait NormalizedScoreClone: NormalizedScore {
    fn clone_box(&self) -> Box<dyn NormalizedScoreClone>;
}

impl<T: NormalizedScore + Clone + 'static> NormalizedScoreClone for T {
    fn clone_box(&self) -> Box<dyn NormalizedScoreClone> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn NormalizedScoreClone> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl fmt::Debug for dyn NormalizedScoreClone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NormalizedScore({:.1})", self.normalized())
    }
}

impl AggregateScore {
    /// Create a new aggregate score.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            components: Vec::new(),
            name: name.into(),
        }
    }

    /// Add a component score with weight.
    pub fn add<S: NormalizedScoreClone + 'static>(&mut self, score: S, weight: f64) {
        self.components.push((Box::new(score), weight.max(0.0)));
    }

    /// Get the total weight.
    pub fn total_weight(&self) -> f64 {
        self.components.iter().map(|(_, w)| w).sum()
    }
}

impl NormalizedScore for AggregateScore {
    fn raw(&self) -> f64 {
        let total_weight = self.total_weight();
        if total_weight <= 0.0 {
            return 0.0;
        }
        self.components
            .iter()
            .map(|(score, weight)| score.normalized() * weight)
            .sum::<f64>()
            / total_weight
    }

    fn max_raw(&self) -> f64 {
        100.0 // Aggregates are already normalized
    }
}

impl fmt::Display for AggregateScore {
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

#[cfg(test)]
mod tests {
    use super::*;

    // =============================================================================
    // RED PHASE: Tests that define expected behavior
    // =============================================================================

    #[test]
    fn test_normalized_score_always_0_to_100() {
        // Any score must return normalized value in [0, 100]
        let score = SimpleScore::new(50.0, 100.0, "test");
        assert!(score.normalized() >= 0.0);
        assert!(score.normalized() <= 100.0);
    }

    #[test]
    fn test_normalized_score_106_scale() {
        // Rust Project Score uses 106-point scale
        let score = SimpleScore::new(95.0, 106.0, "Rust Project");
        let normalized = score.normalized();
        assert!(normalized >= 0.0 && normalized <= 100.0);
        // 95/106 * 100 ≈ 89.6
        assert!((normalized - 89.6).abs() < 0.1);
    }

    #[test]
    fn test_normalized_score_110_scale() {
        // Repo Score uses 110-point scale
        let score = SimpleScore::new(99.0, 110.0, "Repo");
        let normalized = score.normalized();
        assert!(normalized >= 0.0 && normalized <= 100.0);
        // 99/110 * 100 = 90.0
        assert!((normalized - 90.0).abs() < 0.1);
    }

    #[test]
    fn test_normalized_score_200_scale() {
        // Perfection Score uses 200-point scale
        let score = SimpleScore::new(180.0, 200.0, "Perfection");
        let normalized = score.normalized();
        assert!(normalized >= 0.0 && normalized <= 100.0);
        // 180/200 * 100 = 90.0
        assert_eq!(normalized, 90.0);
    }

    #[test]
    fn test_normalized_clamps_negative() {
        // Negative raw scores should clamp to 0
        let score = SimpleScore::new(-10.0, 100.0, "test");
        assert_eq!(score.normalized(), 0.0);
    }

    #[test]
    fn test_normalized_clamps_overflow() {
        // Scores exceeding max should clamp to 100
        let score = SimpleScore::new(150.0, 100.0, "test");
        assert_eq!(score.normalized(), 100.0);
    }

    #[test]
    fn test_grade_boundaries() {
        assert_eq!(Grade::from_score(100.0), Grade::A);
        assert_eq!(Grade::from_score(90.0), Grade::A);
        assert_eq!(Grade::from_score(89.9), Grade::B);
        assert_eq!(Grade::from_score(80.0), Grade::B);
        assert_eq!(Grade::from_score(79.9), Grade::C);
        assert_eq!(Grade::from_score(70.0), Grade::C);
        assert_eq!(Grade::from_score(69.9), Grade::D);
        assert_eq!(Grade::from_score(60.0), Grade::D);
        assert_eq!(Grade::from_score(59.9), Grade::F);
        assert_eq!(Grade::from_score(0.0), Grade::F);
    }

    #[test]
    fn test_grade_ordering() {
        assert!(Grade::A > Grade::B);
        assert!(Grade::B > Grade::C);
        assert!(Grade::C > Grade::D);
        assert!(Grade::D > Grade::F);
    }

    #[test]
    fn test_meets_threshold() {
        let score = SimpleScore::new(85.0, 100.0, "test");
        assert!(score.meets_threshold(80.0));
        assert!(score.meets_threshold(85.0));
        assert!(!score.meets_threshold(90.0));
    }

    #[test]
    fn test_aggregate_score_weighted() {
        let mut agg = AggregateScore::new("Combined");
        agg.add(SimpleScore::new(80.0, 100.0, "a"), 1.0);
        agg.add(SimpleScore::new(100.0, 100.0, "b"), 1.0);
        // (80 + 100) / 2 = 90
        assert_eq!(agg.normalized(), 90.0);
    }

    #[test]
    fn test_aggregate_score_different_weights() {
        let mut agg = AggregateScore::new("Weighted");
        agg.add(SimpleScore::new(100.0, 100.0, "heavy"), 3.0);
        agg.add(SimpleScore::new(0.0, 100.0, "light"), 1.0);
        // (100*3 + 0*1) / 4 = 75
        assert_eq!(agg.normalized(), 75.0);
    }

    #[test]
    fn test_aggregate_empty() {
        let agg = AggregateScore::new("Empty");
        assert_eq!(agg.normalized(), 0.0);
    }

    #[test]
    fn test_display_format() {
        let score = SimpleScore::new(85.0, 100.0, "Test Score");
        let display = format!("{}", score);
        assert!(display.contains("85.0/100"));
        assert!(display.contains("B"));
    }

    #[test]
    fn test_from_percentage() {
        let score = SimpleScore::from_percentage(75.5, "Coverage");
        assert_eq!(score.normalized(), 75.5);
        assert_eq!(score.grade(), Grade::C);
    }

    #[test]
    fn test_zero_max_returns_zero() {
        // Edge case: max=0 should return 0, not panic/NaN
        // This shouldn't happen via new(), but test the trait method directly
        struct ZeroMax;
        impl NormalizedScore for ZeroMax {
            fn raw(&self) -> f64 {
                50.0
            }
            fn max_raw(&self) -> f64 {
                0.0
            }
        }
        impl fmt::Display for ZeroMax {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "ZeroMax")
            }
        }
        let z = ZeroMax;
        assert_eq!(z.normalized(), 0.0);
    }

    // =============================================================================
    // Property-based tests for score normalization
    // =============================================================================

    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn normalized_always_in_range(raw in 0.0f64..1000.0, max in 1.0f64..500.0) {
                let score = SimpleScore::new(raw, max, "prop");
                let normalized = score.normalized();
                prop_assert!(normalized >= 0.0, "normalized {} < 0", normalized);
                prop_assert!(normalized <= 100.0, "normalized {} > 100", normalized);
            }

            #[test]
            fn grade_monotonic(score1 in 0.0f64..100.0, score2 in 0.0f64..100.0) {
                let g1 = Grade::from_score(score1);
                let g2 = Grade::from_score(score2);
                if score1 > score2 {
                    prop_assert!(g1 >= g2);
                } else if score1 < score2 {
                    prop_assert!(g1 <= g2);
                }
            }

            #[test]
            fn aggregate_bounded(
                s1 in 0.0f64..100.0,
                s2 in 0.0f64..100.0,
                w1 in 0.1f64..10.0,
                w2 in 0.1f64..10.0
            ) {
                let mut agg = AggregateScore::new("test");
                agg.add(SimpleScore::from_percentage(s1, "a"), w1);
                agg.add(SimpleScore::from_percentage(s2, "b"), w2);
                let result = agg.normalized();
                prop_assert!(result >= 0.0 && result <= 100.0);
                // Result should be between min and max component
                let min = s1.min(s2);
                let max = s1.max(s2);
                prop_assert!(result >= min - 0.001 && result <= max + 0.001);
            }

            #[test]
            fn threshold_consistent(score in 0.0f64..100.0, threshold in 0.0f64..100.0) {
                let s = SimpleScore::from_percentage(score, "test");
                let meets = s.meets_threshold(threshold);
                if score >= threshold {
                    prop_assert!(meets);
                } else {
                    prop_assert!(!meets);
                }
            }
        }
    }
}
