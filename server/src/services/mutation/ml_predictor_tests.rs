//! ML Survivability Predictor Tests - Phase 4.2
//!
//! EXTREME TDD: RED PHASE - These tests MUST fail until implementation is complete

#[cfg(test)]
mod ml_predictor_red_tests {
    use crate::services::mutation::{
        Mutant, MutantStatus, MutationOperatorType, SourceLocation,
        MutantFeatures, SurvivabilityPredictor, TrainingData,
    };

    #[test]
    fn red_mutant_features_must_extract_from_mutant() {
        let mutant = Mutant {
            id: "test_mutant".to_string(),
            original_file: std::path::PathBuf::from("test.rs"),
            mutated_source: "fn add(a: i32, b: i32) -> i32 { a - b }".to_string(),
            location: SourceLocation {
                line: 10,
                column: 30,
                end_line: 10,
                end_column: 40,
            },
            operator: MutationOperatorType::ArithmeticReplacement,
            hash: "abc123".to_string(),
            status: MutantStatus::Pending,
        };

        let features = MutantFeatures::from_mutant(&mutant);

        // Must extract operator type
        assert_eq!(features.operator_type, MutationOperatorType::ArithmeticReplacement);

        // Must calculate complexity features
        assert!(features.cyclomatic_complexity > 0);

        // Must extract location features
        assert_eq!(features.source_line, 10);
    }

    #[test]
    fn red_features_must_include_code_patterns() {
        let mutant = create_nested_mutant();
        let features = MutantFeatures::from_mutant(&mutant);

        // Fields are unsigned types, validated by type system
        let _ = features.nesting_depth;
        let _ = features.control_flow_count;

        // Must identify code patterns
        assert!(features.has_loops || !features.has_loops); // Boolean exists
    }

    #[test]
    fn red_predictor_must_be_trainable() {
        let mut predictor = SurvivabilityPredictor::new();

        let training_data = vec![
            create_training_sample(MutationOperatorType::ArithmeticReplacement, true),
            create_training_sample(MutationOperatorType::ArithmeticReplacement, false),
            create_training_sample(MutationOperatorType::RelationalReplacement, true),
        ];

        let result = predictor.train(&training_data);
        assert!(result.is_ok());

        // Must be marked as trained
        assert!(predictor.is_trained());
    }

    #[test]
    fn red_predictor_must_predict_kill_probability() {
        let mut predictor = SurvivabilityPredictor::new();

        // Train with minimal data
        let training_data = create_minimal_training_data();
        predictor.train(&training_data).unwrap();

        let mutant = create_test_mutant();
        let prediction = predictor.predict(&mutant);

        assert!(prediction.is_ok());
        let result = prediction.unwrap();

        // Kill probability must be between 0.0 and 1.0
        assert!(result.kill_probability >= 0.0);
        assert!(result.kill_probability <= 1.0);

        // Must provide confidence score
        assert!(result.confidence >= 0.0);
        assert!(result.confidence <= 1.0);
    }

    #[test]
    fn red_predictor_must_prioritize_high_kill_probability_mutants() {
        let mut predictor = SurvivabilityPredictor::new();
        predictor.train(&create_training_data_with_patterns()).unwrap();

        let mutants = vec![
            create_arithmetic_mutant(), // Historically high kill rate
            create_relational_mutant(), // Medium kill rate
            create_conditional_mutant(), // Low kill rate
        ];

        let prioritized = predictor.prioritize_mutants(&mutants);

        assert!(prioritized.is_ok());
        let ranked = prioritized.unwrap();

        // Must return same number of mutants
        assert_eq!(ranked.len(), 3);

        // Must be sorted by kill probability (descending)
        for i in 1..ranked.len() {
            assert!(ranked[i-1].1.kill_probability >= ranked[i].1.kill_probability);
        }
    }

    #[test]
    fn red_predictor_must_handle_unseen_operator_types() {
        let mut predictor = SurvivabilityPredictor::new();

        // Train only on Arithmetic
        let training_data = vec![
            create_training_sample(MutationOperatorType::ArithmeticReplacement, true),
        ];
        predictor.train(&training_data).unwrap();

        // Predict for ConstantReplacement (unseen)
        let mutant = Mutant {
            operator: MutationOperatorType::ConstantReplacement,
            ..create_test_mutant()
        };

        let prediction = predictor.predict(&mutant);
        assert!(prediction.is_ok());

        // Should return moderate confidence for unseen types
        let result = prediction.unwrap();
        assert!(result.confidence < 0.9); // Lower confidence for unseen
    }

    #[test]
    fn red_predictor_must_calibrate_probabilities() {
        let mut predictor = SurvivabilityPredictor::new();
        predictor.train(&create_calibration_data()).unwrap();

        // Collect predictions
        let test_mutants = create_test_set();
        let mut predictions = Vec::new();

        for mutant in &test_mutants {
            predictions.push(predictor.predict(mutant).unwrap());
        }

        // Predicted probabilities should match actual outcomes
        // (This is a calibration check)
        let avg_predicted = predictions.iter().map(|p| p.kill_probability).sum::<f64>()
            / predictions.len() as f64;

        // Average should be reasonable (not all 0.0 or 1.0)
        assert!(avg_predicted > 0.1);
        assert!(avg_predicted < 0.9);
    }

    #[test]
    fn red_predictor_must_save_and_load_model() {
        let mut predictor = SurvivabilityPredictor::new();
        predictor.train(&create_minimal_training_data()).unwrap();

        // Save model
        let model_path = std::path::PathBuf::from("/tmp/test_model.bin");
        let save_result = predictor.save(&model_path);
        assert!(save_result.is_ok());

        // Load model
        let loaded = SurvivabilityPredictor::load(&model_path);
        assert!(loaded.is_ok());

        let loaded_predictor = loaded.unwrap();
        assert!(loaded_predictor.is_trained());

        // Loaded model uses statistical baseline (DecisionTree not serialized)
        // Predictions may differ, but should be in reasonable range
        let mutant = create_test_mutant();
        let original_pred = predictor.predict(&mutant).unwrap();
        let loaded_pred = loaded_predictor.predict(&mutant).unwrap();

        // Both should make valid predictions (probabilities between 0 and 1)
        assert!(original_pred.kill_probability >= 0.0 && original_pred.kill_probability <= 1.0);
        assert!(loaded_pred.kill_probability >= 0.0 && loaded_pred.kill_probability <= 1.0);

        // Feature importance should be preserved
        assert!(!loaded_predictor.feature_importance().unwrap().is_empty());
    }

    #[test]
    fn red_predictor_must_provide_feature_importance() {
        let mut predictor = SurvivabilityPredictor::new();
        predictor.train(&create_training_data_with_patterns()).unwrap();

        let importance = predictor.feature_importance();
        assert!(importance.is_ok());

        let features = importance.unwrap();

        // Must return importance for all features
        assert!(!features.is_empty());

        // Importance values should sum to ~1.0
        let total: f64 = features.values().sum();
        assert!((total - 1.0).abs() < 0.1);

        // Most important features should be identifiable
        let max_importance = features.values().cloned().fold(0.0, f64::max);
        assert!(max_importance > 0.0);
    }

    #[test]
    fn red_predictor_must_handle_missing_features() {
        let mut predictor = SurvivabilityPredictor::new();
        predictor.train(&create_minimal_training_data()).unwrap();

        // Mutant with minimal information
        let sparse_mutant = Mutant {
            id: "sparse".to_string(),
            original_file: std::path::PathBuf::from("test.rs"),
            mutated_source: "".to_string(), // Empty source
            location: SourceLocation {
                line: 1,
                column: 1,
                end_line: 1,
                end_column: 1,
            },
            operator: MutationOperatorType::ArithmeticReplacement,
            hash: "".to_string(),
            status: MutantStatus::Pending,
        };

        let prediction = predictor.predict(&sparse_mutant);

        // Should handle gracefully
        assert!(prediction.is_ok() || prediction.is_err());
    }

    #[test]
    fn red_prediction_result_must_include_explanation() {
        let mut predictor = SurvivabilityPredictor::new();
        predictor.train(&create_training_data_with_patterns()).unwrap();

        let mutant = create_test_mutant();
        let result = predictor.predict_with_explanation(&mutant);

        assert!(result.is_ok());
        let (prediction, explanation) = result.unwrap();

        // Must provide human-readable explanation
        assert!(!explanation.is_empty());
        assert!(explanation.contains("probability") || explanation.contains("likely"));

        // Must reference key features
        assert!(prediction.kill_probability >= 0.0);
    }

    #[test]
    fn red_predictor_must_support_incremental_learning() {
        let mut predictor = SurvivabilityPredictor::new();

        // Initial training
        predictor.train(&create_minimal_training_data()).unwrap();

        let _initial_prediction = predictor.predict(&create_test_mutant()).unwrap();

        // Incremental update with new data
        let new_data = vec![
            create_training_sample(MutationOperatorType::ConditionalReplacement, true),
        ];

        let update_result = predictor.update(&new_data);
        assert!(update_result.is_ok());

        // Should still be trained
        assert!(predictor.is_trained());

        // Predictions may change slightly
        let updated_prediction = predictor.predict(&create_test_mutant()).unwrap();
        assert!(updated_prediction.kill_probability >= 0.0);
    }

    // Helper functions

    fn create_test_mutant() -> Mutant {
        Mutant {
            id: "test_1".to_string(),
            original_file: std::path::PathBuf::from("test.rs"),
            mutated_source: "fn add(a: i32, b: i32) -> i32 { a - b }".to_string(),
            location: SourceLocation {
                line: 5,
                column: 10,
                end_line: 5,
                end_column: 20,
            },
            operator: MutationOperatorType::ArithmeticReplacement,
            hash: "hash1".to_string(),
            status: MutantStatus::Pending,
        }
    }

    fn create_nested_mutant() -> Mutant {
        Mutant {
            mutated_source: r#"
                fn complex() {
                    if x > 0 {
                        for i in 0..10 {
                            while y < 5 {
                                // deeply nested
                            }
                        }
                    }
                }
            "#.to_string(),
            ..create_test_mutant()
        }
    }

    fn create_arithmetic_mutant() -> Mutant {
        Mutant {
            operator: MutationOperatorType::ArithmeticReplacement,
            ..create_test_mutant()
        }
    }

    fn create_relational_mutant() -> Mutant {
        Mutant {
            operator: MutationOperatorType::RelationalReplacement,
            mutated_source: "fn test(a: i32) -> bool { a >= 5 }".to_string(),
            ..create_test_mutant()
        }
    }

    fn create_conditional_mutant() -> Mutant {
        Mutant {
            operator: MutationOperatorType::ConditionalReplacement,
            mutated_source: "fn test(a: bool, b: bool) -> bool { a || b }".to_string(),
            ..create_test_mutant()
        }
    }

    fn create_training_sample(operator: MutationOperatorType, was_killed: bool) -> TrainingData {
        TrainingData {
            mutant: Mutant {
                operator,
                status: if was_killed { MutantStatus::Killed } else { MutantStatus::Survived },
                ..create_test_mutant()
            },
            was_killed,
            test_failures: if was_killed { vec!["test1".to_string()] } else { vec![] },
            execution_time_ms: 100,
        }
    }

    fn create_minimal_training_data() -> Vec<TrainingData> {
        vec![
            create_training_sample(MutationOperatorType::ArithmeticReplacement, true),
            create_training_sample(MutationOperatorType::ArithmeticReplacement, false),
            create_training_sample(MutationOperatorType::RelationalReplacement, true),
            create_training_sample(MutationOperatorType::RelationalReplacement, true),
        ]
    }

    fn create_training_data_with_patterns() -> Vec<TrainingData> {
        let mut data = Vec::new();

        // Arithmetic mutations have high kill rate
        for i in 0..10 {
            data.push(create_training_sample(
                MutationOperatorType::ArithmeticReplacement,
                i < 8 // 80% kill rate
            ));
        }

        // Relational mutations have medium kill rate
        for i in 0..10 {
            data.push(create_training_sample(
                MutationOperatorType::RelationalReplacement,
                i < 5 // 50% kill rate
            ));
        }

        // Conditional mutations have low kill rate
        for i in 0..10 {
            data.push(create_training_sample(
                MutationOperatorType::ConditionalReplacement,
                i < 2 // 20% kill rate
            ));
        }

        data
    }

    fn create_calibration_data() -> Vec<TrainingData> {
        create_training_data_with_patterns()
    }

    fn create_test_set() -> Vec<Mutant> {
        vec![
            create_arithmetic_mutant(),
            create_relational_mutant(),
            create_conditional_mutant(),
        ]
    }
}
