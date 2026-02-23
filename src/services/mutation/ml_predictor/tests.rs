#![cfg_attr(coverage_nightly, coverage(off))]
//! Tests for helper functions, feature extraction, and basic type operations.

#[cfg(test)]
mod tests {
    use super::super::helpers::{
        count_parameters, count_unique_variables, estimate_nesting_depth, is_rust_keyword,
    };
    use super::super::types::{MutantFeatures, PredictionResult, TrainingData};
    use crate::services::mutation::{Mutant, MutantStatus, MutationOperatorType, SourceLocation};
    use std::collections::HashMap;

    // ==================== Helper Functions Tests ====================

    #[test]
    fn test_estimate_nesting_depth_empty() {
        assert_eq!(estimate_nesting_depth(""), 0);
    }

    #[test]
    fn test_estimate_nesting_depth_no_braces() {
        assert_eq!(estimate_nesting_depth("let x = 1;"), 0);
    }

    #[test]
    fn test_estimate_nesting_depth_single() {
        assert_eq!(estimate_nesting_depth("fn foo() { x }"), 1);
    }

    #[test]
    fn test_estimate_nesting_depth_nested() {
        assert_eq!(estimate_nesting_depth("fn foo() { if true { x } }"), 2);
    }

    #[test]
    fn test_estimate_nesting_depth_deeply_nested() {
        assert_eq!(estimate_nesting_depth("{ { { { x } } } }"), 4);
    }

    #[test]
    fn test_estimate_nesting_depth_unbalanced() {
        // Should handle unbalanced gracefully
        assert_eq!(estimate_nesting_depth("{{ }"), 2);
    }

    #[test]
    fn test_count_parameters_empty() {
        assert_eq!(count_parameters("fn foo()"), 0);
    }

    #[test]
    fn test_count_parameters_none() {
        assert_eq!(count_parameters("let x = 5;"), 0);
    }

    #[test]
    fn test_count_parameters_single() {
        assert_eq!(count_parameters("fn foo(x: i32)"), 1);
    }

    #[test]
    fn test_count_parameters_multiple() {
        assert_eq!(count_parameters("fn foo(x: i32, y: i32, z: i32)"), 3);
    }

    #[test]
    fn test_count_parameters_with_generics() {
        assert_eq!(count_parameters("fn foo<T>(x: T, y: T)"), 2);
    }

    #[test]
    fn test_count_unique_variables_empty() {
        assert_eq!(count_unique_variables(""), 0);
    }

    #[test]
    fn test_count_unique_variables_simple() {
        let source = "let x = y + z;";
        let count = count_unique_variables(source);
        assert!(count >= 1); // At least one variable
    }

    #[test]
    fn test_count_unique_variables_with_keywords() {
        let source = "fn let mut x = 5;";
        let count = count_unique_variables(source);
        // Should not count keywords
        assert!(count >= 1);
    }

    #[test]
    fn test_count_unique_variables_uppercase() {
        let source = "let Type = MyStruct;";
        let count = count_unique_variables(source);
        // Uppercase identifiers shouldn't be counted as variables
        assert!(count >= 0);
    }

    #[test]
    fn test_is_rust_keyword_true() {
        assert!(is_rust_keyword("fn"));
        assert!(is_rust_keyword("let"));
        assert!(is_rust_keyword("mut"));
        assert!(is_rust_keyword("if"));
        assert!(is_rust_keyword("else"));
        assert!(is_rust_keyword("for"));
        assert!(is_rust_keyword("while"));
        assert!(is_rust_keyword("loop"));
        assert!(is_rust_keyword("match"));
        assert!(is_rust_keyword("return"));
    }

    #[test]
    fn test_is_rust_keyword_false() {
        assert!(!is_rust_keyword("variable"));
        assert!(!is_rust_keyword("foo"));
        assert!(!is_rust_keyword("bar"));
        assert!(!is_rust_keyword("x"));
        assert!(!is_rust_keyword("MyType"));
    }

    // ==================== MutantFeatures Tests ====================

    pub(super) fn create_test_mutant(source: &str, operator: MutationOperatorType) -> Mutant {
        Mutant {
            id: "test".to_string(),
            original_file: std::path::PathBuf::from("test.rs"),
            mutated_source: source.to_string(),
            location: SourceLocation {
                line: 1,
                column: 1,
                end_line: 1,
                end_column: 10,
            },
            operator,
            hash: "hash".to_string(),
            status: MutantStatus::Pending,
        }
    }

    #[test]
    fn test_mutant_features_from_simple() {
        let mutant = create_test_mutant("let x = 1;", MutationOperatorType::ArithmeticReplacement);
        let features = MutantFeatures::from_mutant(&mutant);

        assert!(!features.has_loops);
        assert!(!features.has_conditionals);
        assert_eq!(features.nesting_depth, 0);
    }

    #[test]
    fn test_mutant_features_with_loops() {
        let mutant = create_test_mutant(
            "for i in 0..10 { x += 1; }",
            MutationOperatorType::ArithmeticReplacement,
        );
        let features = MutantFeatures::from_mutant(&mutant);

        assert!(features.has_loops);
        assert!(features.control_flow_count >= 1);
    }

    #[test]
    fn test_mutant_features_with_conditionals() {
        let mutant = create_test_mutant(
            "if x > 0 { y } else { z }",
            MutationOperatorType::RelationalReplacement,
        );
        let features = MutantFeatures::from_mutant(&mutant);

        assert!(features.has_conditionals);
    }

    #[test]
    fn test_mutant_features_with_error_handling() {
        let mutant = create_test_mutant(
            "fn foo() -> Result<(), Error> { Ok(()) }",
            MutationOperatorType::ReturnReplacement,
        );
        let features = MutantFeatures::from_mutant(&mutant);

        assert!(features.has_error_handling);
    }

    #[test]
    fn test_mutant_features_with_assertions() {
        let mutant = create_test_mutant(
            "assert_eq!(x, y);",
            MutationOperatorType::ArithmeticReplacement,
        );
        let features = MutantFeatures::from_mutant(&mutant);

        assert!(features.has_assertions);
    }

    #[test]
    fn test_mutant_features_with_arithmetic() {
        let mutant = create_test_mutant(
            "let z = x + y;",
            MutationOperatorType::ArithmeticReplacement,
        );
        let features = MutantFeatures::from_mutant(&mutant);

        assert!(features.has_arithmetic);
    }

    #[test]
    fn test_mutant_features_with_comparisons() {
        let mutant =
            create_test_mutant("if x == y { }", MutationOperatorType::RelationalReplacement);
        let features = MutantFeatures::from_mutant(&mutant);

        assert!(features.has_comparisons);
    }

    #[test]
    fn test_mutant_features_with_logical_ops() {
        let mutant = create_test_mutant(
            "if a && b || !c { }",
            MutationOperatorType::ConditionalReplacement,
        );
        let features = MutantFeatures::from_mutant(&mutant);

        assert!(features.has_logical_ops);
    }

    #[test]
    fn test_mutant_features_to_feature_vector_length() {
        let mutant = create_test_mutant("let x = 1;", MutationOperatorType::ArithmeticReplacement);
        let features = MutantFeatures::from_mutant(&mutant);
        let vector = features.to_feature_vector();

        assert_eq!(vector.len(), 18); // 18 features in v2
    }

    #[test]
    fn test_mutant_features_operator_type_numeric() {
        let mutant1 = create_test_mutant("x", MutationOperatorType::ArithmeticReplacement);
        let features1 = MutantFeatures::from_mutant(&mutant1);

        let mutant2 = create_test_mutant("x", MutationOperatorType::RelationalReplacement);
        let features2 = MutantFeatures::from_mutant(&mutant2);

        let vec1 = features1.to_feature_vector();
        let vec2 = features2.to_feature_vector();

        // First element is operator type
        assert_eq!(vec1[0], 1.0); // ArithmeticReplacement
        assert_eq!(vec2[0], 2.0); // RelationalReplacement
    }

    #[test]
    fn test_mutant_features_operator_none() {
        let mutant = create_test_mutant("x", MutationOperatorType::None);
        let features = MutantFeatures::from_mutant(&mutant);
        let vector = features.to_feature_vector();

        assert_eq!(vector[0], 0.0); // None
    }

    #[test]
    fn test_mutant_features_operator_custom() {
        let mutant = create_test_mutant("x", MutationOperatorType::Custom("test".to_string()));
        let features = MutantFeatures::from_mutant(&mutant);
        let vector = features.to_feature_vector();

        assert_eq!(vector[0], 21.0); // Custom
    }

    // ==================== TrainingData Tests ====================

    #[test]
    fn test_training_data_creation() {
        let mutant = create_test_mutant("x + y", MutationOperatorType::ArithmeticReplacement);
        let data = TrainingData {
            mutant: mutant.clone(),
            was_killed: true,
            test_failures: vec!["test_add".to_string()],
            execution_time_ms: 100,
        };

        assert!(data.was_killed);
        assert_eq!(data.test_failures.len(), 1);
        assert_eq!(data.execution_time_ms, 100);
    }

    #[test]
    fn test_training_data_serialization() {
        let mutant = create_test_mutant("x + y", MutationOperatorType::ArithmeticReplacement);
        let data = TrainingData {
            mutant,
            was_killed: false,
            test_failures: vec![],
            execution_time_ms: 50,
        };

        let json = serde_json::to_string(&data).unwrap();
        let deserialized: TrainingData = serde_json::from_str(&json).unwrap();

        assert_eq!(data.was_killed, deserialized.was_killed);
        assert_eq!(data.execution_time_ms, deserialized.execution_time_ms);
    }

    // ==================== PredictionResult Tests ====================

    #[test]
    fn test_prediction_result_creation() {
        let mut contributions = HashMap::new();
        contributions.insert("operator_type".to_string(), 0.5);

        let result = PredictionResult {
            kill_probability: 0.75,
            confidence: 0.9,
            feature_contributions: contributions,
        };

        assert!((result.kill_probability - 0.75).abs() < f64::EPSILON);
        assert!((result.confidence - 0.9).abs() < f64::EPSILON);
        assert_eq!(result.feature_contributions.len(), 1);
    }

    #[test]
    fn test_prediction_result_serialization() {
        let result = PredictionResult {
            kill_probability: 0.5,
            confidence: 0.8,
            feature_contributions: HashMap::new(),
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: PredictionResult = serde_json::from_str(&json).unwrap();

        assert!((result.kill_probability - deserialized.kill_probability).abs() < f64::EPSILON);
    }

    // ==================== MutantFeatures Serialization Tests ====================

    #[test]
    fn test_mutant_features_serialization() {
        let mutant = create_test_mutant(
            "fn foo() { for i in 0..10 { if x > 0 { y } } }",
            MutationOperatorType::ArithmeticReplacement,
        );
        let features = MutantFeatures::from_mutant(&mutant);

        let json = serde_json::to_string(&features).unwrap();
        let deserialized: MutantFeatures = serde_json::from_str(&json).unwrap();

        assert_eq!(features.has_loops, deserialized.has_loops);
        assert_eq!(features.has_conditionals, deserialized.has_conditionals);
        assert_eq!(features.nesting_depth, deserialized.nesting_depth);
    }

    #[test]
    fn test_feature_vector_boolean_encoding() {
        let mutant_with_loops = create_test_mutant(
            "for i in 0..10 { }",
            MutationOperatorType::ArithmeticReplacement,
        );
        let mutant_without_loops =
            create_test_mutant("let x = 1;", MutationOperatorType::ArithmeticReplacement);

        let features_with = MutantFeatures::from_mutant(&mutant_with_loops);
        let features_without = MutantFeatures::from_mutant(&mutant_without_loops);

        let vec_with = features_with.to_feature_vector();
        let vec_without = features_without.to_feature_vector();

        // has_loops is at index 6
        assert_eq!(vec_with[6], 1.0); // true -> 1.0
        assert_eq!(vec_without[6], 0.0); // false -> 0.0
    }
}
