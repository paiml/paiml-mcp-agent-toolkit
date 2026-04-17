#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn complexity_is_never_zero(code in ".*") {
            let profile = QualityProfile::standard();
            let generator = QualityCodeGenerator::new(profile);

            let complexity = generator.estimate_complexity(&code);
            prop_assert!(complexity >= 1, "Complexity must be at least 1");
        }

        #[test]
        fn complexity_increases_with_control_flow(
            base_count in 0usize..5,
            if_count in 0usize..5,
            match_count in 0usize..3,
            for_count in 0usize..3,
            while_count in 0usize..3
        ) {
            let profile = QualityProfile::standard();
            let generator = QualityCodeGenerator::new(profile);

            let mut code = String::new();
            for _ in 0..if_count { code.push_str("if x {} "); }
            for _ in 0..match_count { code.push_str("match x {} "); }
            for _ in 0..for_count { code.push_str("for x in {} "); }
            for _ in 0..while_count { code.push_str("while x {} "); }
            for _ in 0..base_count { code.push_str("let x = 1; "); }

            let expected_min = 1 + if_count as u32 + match_count as u32
                + for_count as u32 + while_count as u32;
            let complexity = generator.estimate_complexity(&code);

            prop_assert!(
                complexity >= expected_min,
                "Complexity {} should be >= {} for code with {} ifs, {} matches, {} fors, {} whiles",
                complexity, expected_min, if_count, match_count, for_count, while_count
            );
        }

        #[test]
        fn satd_count_matches_todo_occurrences(todo_count in 0usize..10) {
            let profile = QualityProfile::standard();
            let generator = QualityCodeGenerator::new(profile);

            let mut code = String::from("fn foo() { ");
            for i in 0..todo_count {
                code.push_str(&format!("// TODO: item {}\n", i));
            }
            code.push('}');

            let metrics = generator.calculate_metrics(&code, "").unwrap();
            prop_assert_eq!(
                metrics.satd_count as usize,
                todo_count,
                "SATD count should match TODO occurrences"
            );
        }
    }

    proptest! {
        #[test]
        fn quality_score_is_bounded(
            complexity in 1u32..50,
            coverage in 0u32..=100,
            tdg in 0u32..20,
            satd_count in 0u32..10
        ) {
            use crate::qdd::core::QualityMetrics;

            let metrics = QualityMetrics {
                complexity,
                cognitive_complexity: complexity,
                coverage,
                tdg,
                satd_count,
                dead_code_percentage: 0,
                has_doctests: false,
                has_property_tests: false,
            };

            let score = metrics.calculate_score();
            prop_assert!(score >= 0.0, "Score must be >= 0: {}", score);
            prop_assert!(score <= 100.0, "Score must be <= 100: {}", score);
        }

        #[test]
        fn higher_coverage_gives_higher_score(
            complexity in 1u32..10,
            coverage1 in 0u32..50,
            coverage2 in 50u32..=100
        ) {
            use crate::qdd::core::QualityMetrics;

            let metrics1 = QualityMetrics {
                complexity,
                cognitive_complexity: complexity,
                coverage: coverage1,
                tdg: 5,
                satd_count: 0,
                dead_code_percentage: 0,
                has_doctests: false,
                has_property_tests: false,
            };

            let metrics2 = QualityMetrics {
                complexity,
                cognitive_complexity: complexity,
                coverage: coverage2,
                tdg: 5,
                satd_count: 0,
                dead_code_percentage: 0,
                has_doctests: false,
                has_property_tests: false,
            };

            let score1 = metrics1.calculate_score();
            let score2 = metrics2.calculate_score();

            prop_assert!(
                score2 >= score1,
                "Higher coverage ({}) should give higher or equal score ({} >= {})",
                coverage2, score2, score1
            );
        }

        #[test]
        fn lower_complexity_gives_higher_score(
            complexity1 in 15u32..25,
            complexity2 in 1u32..10
        ) {
            use crate::qdd::core::QualityMetrics;

            let metrics1 = QualityMetrics {
                complexity: complexity1,
                cognitive_complexity: complexity1,
                coverage: 80,
                tdg: 5,
                satd_count: 0,
                dead_code_percentage: 0,
                has_doctests: false,
                has_property_tests: false,
            };

            let metrics2 = QualityMetrics {
                complexity: complexity2,
                cognitive_complexity: complexity2,
                coverage: 80,
                tdg: 5,
                satd_count: 0,
                dead_code_percentage: 0,
                has_doctests: false,
                has_property_tests: false,
            };

            let score1 = metrics1.calculate_score();
            let score2 = metrics2.calculate_score();

            prop_assert!(
                score2 >= score1,
                "Lower complexity ({}) should give higher or equal score ({} >= {})",
                complexity2, score2, score1
            );
        }
    }

    proptest! {
        #[test]
        fn enhance_with_features_preserves_base_code(
            base in "[a-zA-Z0-9 ]{1,50}",
            features_count in 0usize..5
        ) {
            let profile = QualityProfile::standard();
            let generator = QualityCodeGenerator::new(profile);

            let features: Vec<String> = (0..features_count)
                .map(|i| format!("feature_{}", i))
                .collect();

            let enhanced = generator.enhance_with_features(&base, &features).unwrap();
            prop_assert!(
                enhanced.contains(&base),
                "Enhanced code should preserve base code"
            );
        }
    }
}
