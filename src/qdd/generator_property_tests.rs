//! Property-based tests for QDD generator module

#[cfg(test)]
mod property_tests_coverage {
    use crate::qdd::{
        generator_ast::AstBuilder, generator_core::QualityCodeGenerator,
        generator_doc::DocGenerator, generator_test::TestGenerator, CodeType, CreateSpec,
        Parameter, QualityProfile,
    };
    use proptest::prelude::*;

    // =========================================================================
    // Property-Based Tests for Complexity Estimation
    // =========================================================================

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
            code.push_str("}");

            let metrics = generator.calculate_metrics(&code, "").unwrap();
            prop_assert_eq!(
                metrics.satd_count as usize,
                todo_count,
                "SATD count should match TODO occurrences"
            );
        }
    }

    // =========================================================================
    // Property-Based Tests for Quality Score
    // =========================================================================

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

    // =========================================================================
    // Property-Based Tests for Code Generation
    // =========================================================================

    proptest! {
        #[test]
        fn generated_function_contains_function_name(
            name in "[a-z][a-z0-9_]{0,20}"
        ) {
            let profile = QualityProfile::standard();
            let builder = AstBuilder::new(profile);

            let spec = CreateSpec {
                code_type: CodeType::Function,
                name: name.clone(),
                purpose: "Test purpose".to_string(),
                inputs: vec![],
                outputs: Parameter {
                    name: "result".to_string(),
                    param_type: "()".to_string(),
                    description: None,
                },
            };

            let code = builder.build_function(&spec).unwrap();
            prop_assert!(
                code.contains(&format!("pub fn {}", name)),
                "Generated code should contain function name: {}",
                name
            );
        }

        #[test]
        fn generated_tests_contain_test_function_name(
            name in "[a-z][a-z0-9_]{0,15}"
        ) {
            let profile = QualityProfile::standard();
            let generator = TestGenerator::new(profile);

            let spec = CreateSpec {
                code_type: CodeType::Function,
                name: name.clone(),
                purpose: "Test".to_string(),
                inputs: vec![],
                outputs: Parameter {
                    name: "result".to_string(),
                    param_type: "()".to_string(),
                    description: None,
                },
            };

            let tests = generator.generate_for_function("", &spec).unwrap();
            prop_assert!(
                tests.contains(&format!("test_{}", name)),
                "Generated tests should contain test function name: test_{}",
                name
            );
        }

        #[test]
        fn documentation_contains_purpose(
            purpose in "[A-Za-z][A-Za-z0-9 ]{0,50}"
        ) {
            let profile = QualityProfile::standard();
            let generator = DocGenerator::new(profile);

            let spec = CreateSpec {
                code_type: CodeType::Function,
                name: "test_fn".to_string(),
                purpose: purpose.clone(),
                inputs: vec![],
                outputs: Parameter {
                    name: "result".to_string(),
                    param_type: "()".to_string(),
                    description: None,
                },
            };

            let docs = generator.generate_for_function("", &spec).unwrap();
            prop_assert!(
                docs.contains(&purpose),
                "Documentation should contain purpose: {}",
                purpose
            );
        }

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
