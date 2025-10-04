//! Cross-Validation Tests for ML Predictor

#[cfg(test)]
mod cross_validation_tests {
    use crate::services::mutation::{
        Mutant, MutantStatus, MutationOperatorType, SourceLocation, SurvivabilityPredictor,
        TrainingData,
    };

    fn create_test_mutant() -> Mutant {
        Mutant {
            id: "test_1".to_string(),
            original_file: std::path::PathBuf::from("test.rs"),
            mutated_source: "fn test(a: i32) -> i32 { a - 1 }".to_string(),
            location: SourceLocation {
                line: 10,
                column: 5,
                end_line: 10,
                end_column: 15,
            },
            operator: MutationOperatorType::ArithmeticReplacement,
            hash: "test_hash".to_string(),
            status: MutantStatus::Pending,
        }
    }

    fn create_diverse_training_data() -> Vec<TrainingData> {
        let mut data = Vec::new();

        // Add diverse mutants with different outcomes
        for i in 0..20 {
            let operator = if i % 2 == 0 {
                MutationOperatorType::ArithmeticReplacement
            } else {
                MutationOperatorType::RelationalReplacement
            };

            let was_killed = i < 15; // 75% kill rate

            data.push(TrainingData {
                mutant: Mutant {
                    operator,
                    ..create_test_mutant()
                },
                was_killed,
                test_failures: vec![],
                execution_time_ms: 100,
            });
        }

        data
    }

    #[test]
    fn test_cross_validate_basic() {
        let predictor = SurvivabilityPredictor::new();
        let training_data = create_diverse_training_data();

        // 5-fold cross-validation
        let accuracy = predictor.cross_validate(&training_data, 5);
        assert!(accuracy.is_ok());

        let acc = accuracy.unwrap();
        println!("Cross-validation accuracy: {:.2}%", acc * 100.0);

        // Accuracy should be reasonable (> 0.5 for this test data)
        assert!(acc >= 0.5, "Expected accuracy >= 50%, got {:.2}%", acc * 100.0);
        assert!(acc <= 1.0, "Accuracy should not exceed 100%");
    }

    #[test]
    fn test_cross_validate_with_perfect_data() {
        let predictor = SurvivabilityPredictor::new();

        // Create data where all ArithmeticReplacement are killed
        let mut data = Vec::new();
        for _i in 0..20 {
            data.push(TrainingData {
                mutant: Mutant {
                    operator: MutationOperatorType::ArithmeticReplacement,
                    ..create_test_mutant()
                },
                was_killed: true,
                test_failures: vec![],
                execution_time_ms: 100,
            });
        }

        let accuracy = predictor.cross_validate(&data, 4).unwrap();
        println!("Perfect data accuracy: {:.2}%", accuracy * 100.0);

        // Should achieve high accuracy on perfectly separable data
        assert!(accuracy >= 0.8, "Expected high accuracy on perfect data");
    }

    #[test]
    fn test_cross_validate_insufficient_folds() {
        let predictor = SurvivabilityPredictor::new();
        let training_data = create_diverse_training_data();

        // k_folds must be at least 2
        let result = predictor.cross_validate(&training_data, 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least 2"));
    }

    #[test]
    fn test_cross_validate_empty_data() {
        let predictor = SurvivabilityPredictor::new();

        let result = predictor.cross_validate(&[], 5);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_cross_validate_too_many_folds() {
        let predictor = SurvivabilityPredictor::new();

        // Only 5 samples, trying 10 folds
        let small_data: Vec<_> = create_diverse_training_data().into_iter().take(5).collect();

        let result = predictor.cross_validate(&small_data, 10);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not enough samples"));
    }
}
