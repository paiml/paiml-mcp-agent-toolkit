#![cfg_attr(coverage_nightly, coverage(off))]

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(
    Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
/// Grade.
pub enum Grade {
    APLus,
    A,
    AMinus,
    BPlus,
    B,
    BMinus,
    CPlus,
    #[default]
    C,
    CMinus,
    D,
    F,
}

impl Grade {
    /// Returns `true` if this grade is at least as good as `threshold`.
    ///
    /// `Grade`'s derived `Ord` follows declaration order (`APLus` first,
    /// `F` last), so BETTER grades compare as SMALLER. Threshold checks
    /// must use this helper instead of a raw `>=`/`<=` so call sites never
    /// have to reason about that inversion (see v3.18.2 quality_gate fix).
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "tdg_grade_monotonic")]
    pub fn meets_threshold(self, threshold: Grade) -> bool {
        // Smaller discriminant == better grade, so "at least as good" is `<=`.
        self <= threshold
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    /// From score.
    pub fn from_score(score: f32) -> Self {
        match score {
            s if s >= 95.0 => Grade::APLus,
            s if s >= 90.0 => Grade::A,
            s if s >= 85.0 => Grade::AMinus,
            s if s >= 80.0 => Grade::BPlus,
            s if s >= 75.0 => Grade::B,
            s if s >= 70.0 => Grade::BMinus,
            s if s >= 65.0 => Grade::CPlus,
            s if s >= 60.0 => Grade::C,
            s if s >= 55.0 => Grade::CMinus,
            s if s >= 50.0 => Grade::D,
            _ => Grade::F,
        }
    }
}

impl std::fmt::Display for Grade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Grade::APLus => write!(f, "A+"),
            Grade::A => write!(f, "A"),
            Grade::AMinus => write!(f, "A-"),
            Grade::BPlus => write!(f, "B+"),
            Grade::B => write!(f, "B"),
            Grade::BMinus => write!(f, "B-"),
            Grade::CPlus => write!(f, "C+"),
            Grade::C => write!(f, "C"),
            Grade::CMinus => write!(f, "C-"),
            Grade::D => write!(f, "D"),
            Grade::F => write!(f, "F"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
/// Category classification for metric.
pub enum MetricCategory {
    StructuralComplexity,
    SemanticComplexity,
    Duplication,
    Coupling,
    Documentation,
    Consistency,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Penalty attribution.
pub struct PenaltyAttribution {
    pub source_metric: MetricCategory,
    pub amount: f32,
    pub applied_to: HashSet<MetricCategory>,
    pub issue: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// All grades from best to worst (declaration order).
    const BEST_TO_WORST: [Grade; 11] = [
        Grade::APLus,
        Grade::A,
        Grade::AMinus,
        Grade::BPlus,
        Grade::B,
        Grade::BMinus,
        Grade::CPlus,
        Grade::C,
        Grade::CMinus,
        Grade::D,
        Grade::F,
    ];

    #[test]
    fn test_derived_ord_makes_better_grades_smaller() {
        // Documents the inversion that meets_threshold exists to hide:
        // declaration order means APLus < F under the derived Ord.
        assert!(Grade::APLus < Grade::F);
        assert!(Grade::AMinus < Grade::BPlus);
    }

    #[test]
    fn test_meets_threshold_better_grade_passes() {
        // A- is better than B+, so it must meet a B+ threshold
        assert!(Grade::AMinus.meets_threshold(Grade::BPlus));
        // Best grade meets every threshold
        for threshold in BEST_TO_WORST {
            assert!(Grade::APLus.meets_threshold(threshold));
        }
    }

    #[test]
    fn test_meets_threshold_worse_grade_fails() {
        // C is worse than B+, so it must NOT meet a B+ threshold
        assert!(!Grade::C.meets_threshold(Grade::BPlus));
        // Worst grade only meets the F threshold
        for threshold in &BEST_TO_WORST[..BEST_TO_WORST.len() - 1] {
            assert!(!Grade::F.meets_threshold(*threshold));
        }
        assert!(Grade::F.meets_threshold(Grade::F));
    }

    #[test]
    fn test_meets_threshold_exhaustive_direction() {
        // For every pair: grade meets threshold iff it sits at or before
        // the threshold in best-to-worst order.
        for (gi, grade) in BEST_TO_WORST.iter().enumerate() {
            for (ti, threshold) in BEST_TO_WORST.iter().enumerate() {
                assert_eq!(
                    grade.meets_threshold(*threshold),
                    gi <= ti,
                    "{grade} vs threshold {threshold}"
                );
            }
        }
    }
}
