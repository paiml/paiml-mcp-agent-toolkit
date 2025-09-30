//! EXTREME TDD: TDG Score Normalization Tests
//!
//! These tests enforce that TDG scores are properly normalized to 0-100 range
//! regardless of component values, entropy contribution, or complexity analysis.

#[cfg(test)]
mod red_phase_tests {
    use crate::tdg::{TdgScore, Grade};

    /// RED TEST: TDG total score MUST always be in 0-100 range
    ///
    /// This test will FAIL until normalization is properly implemented in calculate_total()
    #[test]
    fn red_test_tdg_total_must_be_normalized_to_0_100() {
        let mut score = TdgScore::default();

        // Test 1: Default score should be in range
        score.calculate_total();
        assert!(
            score.total >= 0.0 && score.total <= 100.0,
            "Default TDG total {} is outside 0-100 range",
            score.total
        );

        // Test 2: All components at max should still normalize to 100
        score.structural_complexity = 25.0;
        score.semantic_complexity = 20.0;
        score.duplication_ratio = 20.0;
        score.coupling_score = 15.0;
        score.doc_coverage = 10.0;
        score.consistency_score = 10.0;
        score.entropy_score = 0.0; // Should not break normalization
        score.calculate_total();

        assert!(
            score.total >= 0.0 && score.total <= 100.0,
            "TDG total {} with max components is outside 0-100 range",
            score.total
        );
        assert!(
            (score.total - 100.0).abs() < 0.01,
            "TDG total {} should be ~100 when all components at max",
            score.total
        );

        // Test 3: All components at 0 should give 0
        score.structural_complexity = 0.0;
        score.semantic_complexity = 0.0;
        score.duplication_ratio = 0.0;
        score.coupling_score = 0.0;
        score.doc_coverage = 0.0;
        score.consistency_score = 0.0;
        score.entropy_score = 0.0;
        score.calculate_total();

        assert!(
            score.total >= 0.0 && score.total <= 100.0,
            "TDG total {} with zero components is outside 0-100 range",
            score.total
        );
        assert!(
            score.total.abs() < 0.01,
            "TDG total {} should be ~0 when all components at 0",
            score.total
        );
    }

    /// RED TEST: Entropy score MUST NOT break 0-100 normalization
    ///
    /// When entropy is added, the total must remain in 0-100 range
    #[test]
    fn red_test_entropy_must_not_break_normalization() {
        let mut score = TdgScore::default();

        // Test with various entropy values
        for entropy in [0.0, 5.0, 10.0, 15.0, 20.0, 50.0] {
            score.structural_complexity = 25.0;
            score.semantic_complexity = 20.0;
            score.duplication_ratio = 20.0;
            score.coupling_score = 15.0;
            score.doc_coverage = 10.0;
            score.consistency_score = 10.0;
            score.entropy_score = entropy;
            score.calculate_total();

            assert!(
                score.total >= 0.0 && score.total <= 100.0,
                "TDG total {} with entropy {} is outside 0-100 range",
                score.total,
                entropy
            );
        }
    }

    /// RED TEST: Component scores MUST be clamped to their weight limits
    ///
    /// Individual components should never exceed their designated weight
    #[test]
    fn red_test_components_must_respect_weight_limits() {
        // Try to set components beyond their limits
        let mut score = TdgScore {
            structural_complexity: 50.0, // Max should be ~25
            semantic_complexity: 40.0,   // Max should be ~20
            duplication_ratio: 40.0,     // Max should be ~20
            coupling_score: 30.0,        // Max should be ~15
            doc_coverage: 20.0,          // Max should be ~10
            consistency_score: 20.0,     // Max should be ~10
            entropy_score: 100.0,        // Should have a reasonable limit
            ..Default::default()
        };

        score.calculate_total();

        // After normalization, total should still be in range
        assert!(
            score.total >= 0.0 && score.total <= 100.0,
            "TDG total {} with excessive components is outside 0-100 range",
            score.total
        );
    }

    /// RED TEST: Grade MUST match the normalized score
    ///
    /// Grade calculation should work correctly after normalization
    #[test]
    fn red_test_grade_must_match_normalized_score() {
        let mut score = TdgScore::default();

        // Test various score levels
        let test_cases = vec![
            (100.0, Grade::APLus),
            (95.0, Grade::APLus),
            (90.0, Grade::A),
            (85.0, Grade::AMinus),
            (75.0, Grade::B),
            (60.0, Grade::C),
            (50.0, Grade::D),
            (40.0, Grade::F),
        ];

        for (target_total, expected_grade) in test_cases {
            // Set components to achieve target (simplified)
            let component_value = target_total / 6.0; // 6 main components
            score.structural_complexity = component_value * 25.0 / (100.0 / 6.0);
            score.semantic_complexity = component_value * 20.0 / (100.0 / 6.0);
            score.duplication_ratio = component_value * 20.0 / (100.0 / 6.0);
            score.coupling_score = component_value * 15.0 / (100.0 / 6.0);
            score.doc_coverage = component_value * 10.0 / (100.0 / 6.0);
            score.consistency_score = component_value * 10.0 / (100.0 / 6.0);
            score.entropy_score = 0.0;

            score.calculate_total();

            // Verify total is in range
            assert!(
                score.total >= 0.0 && score.total <= 100.0,
                "TDG total {} targeting {} is outside 0-100 range",
                score.total,
                target_total
            );

            // Grade should be correctly calculated from normalized score
            assert_eq!(
                score.grade, expected_grade,
                "Grade {:?} doesn't match expected {:?} for score {}",
                score.grade, expected_grade, score.total
            );
        }
    }

    /// RED TEST: High complexity values must be properly bounded
    ///
    /// Even with extreme penalty deductions, score should stay in range
    #[test]
    fn red_test_extreme_complexity_stays_in_bounds() {
        // Simulate extreme penalty scenario where all components go negative
        let mut score = TdgScore {
            structural_complexity: -10.0,  // Penalties exceeded starting value
            semantic_complexity: -5.0,
            duplication_ratio: -8.0,
            coupling_score: -3.0,
            doc_coverage: -2.0,
            consistency_score: -4.0,
            entropy_score: -1.0,
            ..Default::default()
        };

        score.calculate_total();

        // Even with negative components, total must be clamped to 0-100
        assert!(
            score.total >= 0.0 && score.total <= 100.0,
            "TDG total {} with negative components is outside 0-100 range",
            score.total
        );
        assert!(
            score.total >= 0.0,
            "TDG total {} should be clamped to minimum of 0",
            score.total
        );
    }

    /// RED TEST: Entropy contribution must be balanced with other metrics
    ///
    /// Entropy should have proportional weight, not dominate the score
    #[test]
    fn red_test_entropy_weight_is_balanced() {
        // Set all non-entropy components to perfect
        let mut score = TdgScore {
            structural_complexity: 25.0,
            semantic_complexity: 20.0,
            duplication_ratio: 20.0,
            coupling_score: 15.0,
            doc_coverage: 10.0,
            consistency_score: 10.0,
            ..Default::default()
        };

        // Test with max entropy
        score.entropy_score = 100.0; // Unreasonably high
        score.calculate_total();

        // Total should still be in range (entropy should be weighted/clamped)
        assert!(
            score.total >= 0.0 && score.total <= 100.0,
            "TDG total {} with max entropy is outside 0-100 range",
            score.total
        );

        // Entropy should not contribute more than ~10-15% of total
        let total_without_entropy = 100.0;
        let max_acceptable_total = 115.0; // Allow some buffer

        assert!(
            score.total <= max_acceptable_total,
            "TDG total {} suggests entropy has too much weight (base was {})",
            score.total,
            total_without_entropy
        );
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;
    use crate::tdg::TdgScore;

    proptest! {
        /// Property: Any combination of component values must produce total in 0-100
        #[test]
        fn prop_any_components_produce_valid_total(
            structural in -50.0f32..150.0,
            semantic in -50.0f32..150.0,
            duplication in -50.0f32..150.0,
            coupling in -50.0f32..150.0,
            doc in -50.0f32..150.0,
            consistency in -50.0f32..150.0,
            entropy in -50.0f32..150.0,
        ) {
            let mut score = TdgScore {
                structural_complexity: structural,
                semantic_complexity: semantic,
                duplication_ratio: duplication,
                coupling_score: coupling,
                doc_coverage: doc,
                consistency_score: consistency,
                ..Default::default()
            };
            score.entropy_score = entropy;

            score.calculate_total();

            prop_assert!(
                score.total >= 0.0 && score.total <= 100.0,
                "TDG total {} is outside 0-100 range with components: struct={}, sem={}, dup={}, coup={}, doc={}, cons={}, ent={}",
                score.total, structural, semantic, duplication, coupling, doc, consistency, entropy
            );
        }

        /// Property: Grade must always be valid for the total score
        #[test]
        fn prop_grade_always_matches_total(
            structural in 0.0f32..30.0,
            semantic in 0.0f32..25.0,
            duplication in 0.0f32..25.0,
            coupling in 0.0f32..20.0,
            doc in 0.0f32..15.0,
            consistency in 0.0f32..15.0,
        ) {
            let mut score = TdgScore {
                structural_complexity: structural,
                semantic_complexity: semantic,
                duplication_ratio: duplication,
                coupling_score: coupling,
                doc_coverage: doc,
                consistency_score: consistency,
                ..Default::default()
            };
            score.entropy_score = 0.0;

            score.calculate_total();

            // Verify grade matches the score range
            let expected_grade = crate::tdg::Grade::from_score(score.total);
            prop_assert_eq!(score.grade, expected_grade);
        }
    }
}