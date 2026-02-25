// Tests for normalized_score module.
// Included by normalized_score.rs — no `use` imports or `#!` attributes.

#[cfg_attr(coverage_nightly, coverage(off))]
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

    #[test]
    fn test_grade_display() {
        assert_eq!(format!("{}", Grade::A), "A");
        assert_eq!(format!("{}", Grade::B), "B");
        assert_eq!(format!("{}", Grade::C), "C");
        assert_eq!(format!("{}", Grade::D), "D");
        assert_eq!(format!("{}", Grade::F), "F");
    }

    #[test]
    fn test_grade_description() {
        assert_eq!(Grade::A.description(), "A (Excellent)");
        assert_eq!(Grade::B.description(), "B (Good)");
        assert_eq!(Grade::C.description(), "C (Satisfactory)");
        assert_eq!(Grade::D.description(), "D (Needs Improvement)");
        assert_eq!(Grade::F.description(), "F (Failing)");
    }

    #[test]
    fn test_grade_min_score() {
        assert_eq!(Grade::A.min_score(), 90.0);
        assert_eq!(Grade::B.min_score(), 80.0);
        assert_eq!(Grade::C.min_score(), 70.0);
        assert_eq!(Grade::D.min_score(), 60.0);
        assert_eq!(Grade::F.min_score(), 0.0);
    }

    #[test]
    fn test_grade_partial_cmp() {
        assert!(Grade::A.partial_cmp(&Grade::B) == Some(std::cmp::Ordering::Greater));
        assert!(Grade::B.partial_cmp(&Grade::B) == Some(std::cmp::Ordering::Equal));
        assert!(Grade::C.partial_cmp(&Grade::B) == Some(std::cmp::Ordering::Less));
    }

    #[test]
    fn test_grade_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Grade::A);
        set.insert(Grade::B);
        assert!(set.contains(&Grade::A));
        assert!(set.contains(&Grade::B));
        assert!(!set.contains(&Grade::C));
    }

    #[test]
    fn test_aggregate_display() {
        let mut agg = AggregateScore::new("Combined Score");
        agg.add(SimpleScore::new(80.0, 100.0, "a"), 1.0);
        agg.add(SimpleScore::new(90.0, 100.0, "b"), 1.0);
        let display = format!("{}", agg);
        assert!(display.contains("Combined Score"));
        assert!(display.contains("85.0"));
    }

    #[test]
    fn test_aggregate_total_weight() {
        let mut agg = AggregateScore::new("Test");
        agg.add(SimpleScore::new(50.0, 100.0, "a"), 2.0);
        agg.add(SimpleScore::new(50.0, 100.0, "b"), 3.0);
        assert_eq!(agg.total_weight(), 5.0);
    }

    #[test]
    fn test_aggregate_negative_weight_clamped() {
        let mut agg = AggregateScore::new("Test");
        agg.add(SimpleScore::new(50.0, 100.0, "a"), -1.0); // Negative weight should be clamped to 0
        agg.add(SimpleScore::new(100.0, 100.0, "b"), 1.0);
        assert_eq!(agg.total_weight(), 1.0); // -1 clamped to 0 + 1 = 1
    }

    #[test]
    fn test_simple_score_raw_and_max() {
        let score = SimpleScore::new(75.0, 100.0, "Test");
        assert_eq!(score.raw(), 75.0);
        assert_eq!(score.max_raw(), 100.0);
    }

    #[test]
    fn test_from_percentage_clamps() {
        // Test clamping of extreme values
        let high = SimpleScore::from_percentage(150.0, "High");
        assert_eq!(high.raw(), 100.0);

        let low = SimpleScore::from_percentage(-50.0, "Low");
        assert_eq!(low.raw(), 0.0);
    }

    #[test]
    fn test_normalized_score_clone_box() {
        let score = SimpleScore::new(80.0, 100.0, "test");
        let boxed: Box<dyn NormalizedScoreClone> = Box::new(score);
        let cloned = boxed.clone();
        assert_eq!(cloned.normalized(), 80.0);
    }

    #[test]
    fn test_normalized_score_clone_debug() {
        let score = SimpleScore::new(80.0, 100.0, "test");
        let boxed: Box<dyn NormalizedScoreClone> = Box::new(score);
        let debug_str = format!("{:?}", boxed);
        assert!(debug_str.contains("NormalizedScore"));
        assert!(debug_str.contains("80.0"));
    }

    #[test]
    fn test_simple_score_debug() {
        let score = SimpleScore::new(75.0, 100.0, "Test");
        let debug_str = format!("{:?}", score);
        assert!(debug_str.contains("SimpleScore"));
        assert!(debug_str.contains("75.0"));
    }

    #[test]
    fn test_aggregate_score_debug() {
        let mut agg = AggregateScore::new("Test");
        agg.add(SimpleScore::new(50.0, 100.0, "a"), 1.0);
        let debug_str = format!("{:?}", agg);
        assert!(debug_str.contains("AggregateScore"));
        assert!(debug_str.contains("Test"));
    }

    #[test]
    fn test_simple_score_copy() {
        let score = SimpleScore::new(70.0, 100.0, "Test");
        let copied = score; // Copy (SimpleScore is Copy)
        assert_eq!(copied.normalized(), 70.0);
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
