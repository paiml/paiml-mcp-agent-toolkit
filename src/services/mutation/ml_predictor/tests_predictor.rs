#![cfg_attr(coverage_nightly, coverage(off))]
//! Tests for SurvivabilityPredictor training, prediction, and edge cases.

#[cfg(test)]
mod tests {
    use super::super::predictor::SurvivabilityPredictor;
    use super::super::types::TrainingData;
    use crate::services::mutation::{Mutant, MutantStatus, MutationOperatorType, SourceLocation};

    fn create_test_mutant(source: &str, operator: MutationOperatorType) -> Mutant {
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

    fn create_training_data(count: usize, kill_rate: f64) -> Vec<TrainingData> {
        (0..count)
            .map(|i| {
                let mutant = create_test_mutant(
                    &format!("fn test{}() {{ x + y }}", i),
                    MutationOperatorType::ArithmeticReplacement,
                );
                TrainingData {
                    mutant,
                    was_killed: (i as f64 / count as f64) < kill_rate,
                    test_failures: if (i as f64 / count as f64) < kill_rate {
                        vec!["test".to_string()]
                    } else {
                        vec![]
                    },
                    execution_time_ms: 100,
                }
            })
            .collect()
    }

    // ==================== SurvivabilityPredictor Tests ====================

    #[test]
    fn test_predictor_new() {
        let predictor = SurvivabilityPredictor::new();
        assert!(!predictor.is_trained());
    }

    #[test]
    fn test_predictor_default() {
        let predictor = SurvivabilityPredictor::default();
        assert!(!predictor.is_trained());
    }

    #[test]
    fn test_predictor_train_empty_fails() {
        let mut predictor = SurvivabilityPredictor::new();
        let result = predictor.train(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_predictor_predict_untrained_fails() {
        let predictor = SurvivabilityPredictor::new();
        let mutant = create_test_mutant("x + y", MutationOperatorType::ArithmeticReplacement);
        let result = predictor.predict(&mutant);
        assert!(result.is_err());
    }

    #[test]
    fn test_predictor_feature_importance_untrained_fails() {
        let predictor = SurvivabilityPredictor::new();
        let result = predictor.feature_importance();
        assert!(result.is_err());
    }

    #[test]
    fn test_predictor_train_small_sample() {
        let mut predictor = SurvivabilityPredictor::new();
        let data = create_training_data(5, 0.5);

        // Should succeed but use statistical baseline
        let result = predictor.train(&data);
        assert!(result.is_ok());
        assert!(predictor.is_trained());
    }

    #[test]
    fn test_predictor_train_and_predict() {
        let mut predictor = SurvivabilityPredictor::new();
        let data = create_training_data(30, 0.6); // More samples than features (18)

        let result = predictor.train(&data);
        assert!(result.is_ok());

        let mutant = create_test_mutant("x + y", MutationOperatorType::ArithmeticReplacement);
        let prediction = predictor.predict(&mutant);
        assert!(prediction.is_ok());

        let pred = prediction.unwrap();
        assert!(pred.kill_probability >= 0.0 && pred.kill_probability <= 1.0);
        assert!(pred.confidence >= 0.0 && pred.confidence <= 1.0);
    }

    #[test]
    fn test_predictor_predict_with_explanation() {
        let mut predictor = SurvivabilityPredictor::new();
        let data = create_training_data(20, 0.7);
        predictor.train(&data).unwrap();

        let mutant = create_test_mutant("x + y", MutationOperatorType::ArithmeticReplacement);
        let result = predictor.predict_with_explanation(&mutant);
        assert!(result.is_ok());

        let (pred, explanation) = result.unwrap();
        assert!(pred.kill_probability >= 0.0);
        assert!(!explanation.is_empty());
        assert!(explanation.contains("Kill probability"));
    }

    #[test]
    fn test_predictor_prioritize_mutants() {
        let mut predictor = SurvivabilityPredictor::new();
        let data = create_training_data(20, 0.6);
        predictor.train(&data).unwrap();

        let mutants = vec![
            create_test_mutant("simple", MutationOperatorType::ArithmeticReplacement),
            create_test_mutant(
                "if x > 0 { for i in 0..10 { y += i; } }",
                MutationOperatorType::ConditionalReplacement,
            ),
        ];

        let result = predictor.prioritize_mutants(&mutants);
        assert!(result.is_ok());

        let prioritized = result.unwrap();
        assert_eq!(prioritized.len(), 2);

        // Should be sorted by kill probability descending
        assert!(prioritized[0].1.kill_probability >= prioritized[1].1.kill_probability);
    }

    #[test]
    fn test_predictor_update() {
        let mut predictor = SurvivabilityPredictor::new();
        let initial_data = create_training_data(20, 0.5);
        predictor.train(&initial_data).unwrap();

        let new_data = create_training_data(5, 0.8);
        let result = predictor.update(&new_data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_predictor_update_untrained() {
        let mut predictor = SurvivabilityPredictor::new();
        let data = create_training_data(20, 0.5);

        // Update should train if not trained
        let result = predictor.update(&data);
        assert!(result.is_ok());
        assert!(predictor.is_trained());
    }

    #[test]
    fn test_predictor_feature_importance() {
        let mut predictor = SurvivabilityPredictor::new();
        let data = create_training_data(20, 0.5);
        predictor.train(&data).unwrap();

        let importance = predictor.feature_importance();
        assert!(importance.is_ok());

        let imp = importance.unwrap();
        assert!(!imp.is_empty());

        // All values should be non-negative
        for value in imp.values() {
            assert!(*value >= 0.0);
        }
    }

    #[test]
    fn test_predictor_cross_validate_empty_fails() {
        let predictor = SurvivabilityPredictor::new();
        let result = predictor.cross_validate(&[], 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_predictor_cross_validate_insufficient_folds() {
        let predictor = SurvivabilityPredictor::new();
        let data = create_training_data(20, 0.5);
        let result = predictor.cross_validate(&data, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_predictor_cross_validate() {
        let predictor = SurvivabilityPredictor::new();
        let data = create_training_data(100, 0.5);
        let result = predictor.cross_validate(&data, 5);

        assert!(result.is_ok());
        let accuracy = result.unwrap();
        assert!((0.0..=1.0).contains(&accuracy));
    }

    #[test]
    fn test_predictor_save_and_load() {
        use tempfile::tempdir;

        let mut predictor = SurvivabilityPredictor::new();
        let data = create_training_data(20, 0.5);
        predictor.train(&data).unwrap();

        let dir = tempdir().unwrap();
        let path = dir.path().join("model.bin");

        // Save
        let save_result = predictor.save(&path);
        assert!(save_result.is_ok());

        // Load
        let load_result = SurvivabilityPredictor::load(&path);
        assert!(load_result.is_ok());

        let loaded = load_result.unwrap();
        assert!(loaded.is_trained());
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_predictor_different_operators() {
        let mut predictor = SurvivabilityPredictor::new();

        // Create training data with different operators
        let data: Vec<TrainingData> = [
            MutationOperatorType::ArithmeticReplacement,
            MutationOperatorType::RelationalReplacement,
            MutationOperatorType::ConditionalReplacement,
            MutationOperatorType::StatementDeletion,
        ]
        .iter()
        .enumerate()
        .flat_map(|(i, op)| {
            (0..5).map(move |j| TrainingData {
                mutant: create_test_mutant(&format!("code_{}{}", i, j), op.clone()),
                was_killed: j % 2 == 0,
                test_failures: vec![],
                execution_time_ms: 100,
            })
        })
        .collect();

        predictor.train(&data).unwrap();

        // Predict for each operator type
        for op in [
            MutationOperatorType::ArithmeticReplacement,
            MutationOperatorType::RelationalReplacement,
        ] {
            let mutant = create_test_mutant("test", op);
            let result = predictor.predict(&mutant);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_predictor_unseen_operator() {
        let mut predictor = SurvivabilityPredictor::new();

        // Train only on ArithmeticReplacement
        let data: Vec<TrainingData> = (0..20)
            .map(|i| TrainingData {
                mutant: create_test_mutant(
                    &format!("code_{}", i),
                    MutationOperatorType::ArithmeticReplacement,
                ),
                was_killed: i % 2 == 0,
                test_failures: vec![],
                execution_time_ms: 100,
            })
            .collect();

        predictor.train(&data).unwrap();

        // Predict for unseen operator
        let mutant = create_test_mutant("test", MutationOperatorType::BitwiseReplacement);
        let result = predictor.predict(&mutant);
        assert!(result.is_ok());

        // Should have lower confidence for unseen operator
        let pred = result.unwrap();
        assert!(pred.confidence <= 0.8);
    }

    #[test]
    fn test_complex_source_feature_extraction() {
        use super::super::types::MutantFeatures;

        let complex_source = r#"
            fn complex() -> Result<i32, Error> {
                let mut result = 0;
                for i in 0..100 {
                    if i % 2 == 0 && i > 10 {
                        match get_value(i)? {
                            Some(v) => result += v,
                            None => continue,
                        }
                    }
                }
                assert!(result > 0);
                Ok(result)
            }
        "#;

        let mutant =
            create_test_mutant(complex_source, MutationOperatorType::ArithmeticReplacement);
        let features = MutantFeatures::from_mutant(&mutant);

        assert!(features.has_loops);
        assert!(features.has_conditionals);
        assert!(features.has_error_handling);
        assert!(features.has_assertions);
        assert!(features.has_arithmetic);
        assert!(features.has_comparisons);
        assert!(features.has_logical_ops);
        assert!(features.nesting_depth >= 2);
        assert!(features.token_count > 10);
    }
}
