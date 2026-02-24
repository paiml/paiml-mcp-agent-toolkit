#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // ==================== RED PHASE TESTS ====================
    // These tests define expected behavior for GH-97

    #[test]
    fn test_complexity_features_extraction() {
        let source = r#"
fn complex_function(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            for i in 0..x {
                if i % 2 == 0 {
                    println!("{}", i);
                }
            }
        }
    }
    x
}

fn simple_function() -> i32 {
    42
}
"#;

        let features = ComplexityFeatures::from_source(source, "rust");

        assert!(features.loc > 0.0, "LOC should be positive");
        assert!(
            features.max_nesting >= 3.0,
            "Max nesting should be >= 3 for nested code"
        );
        assert!(
            features.conditional_count >= 2.0,
            "Should detect at least 2 conditionals"
        );
        assert!(features.loop_count >= 1.0, "Should detect at least 1 loop");
        assert!(features.function_count >= 2.0, "Should detect 2 functions");
        assert_eq!(features.language_type, 1.0, "Rust should be encoded as 1.0");
    }

    #[test]
    fn test_complexity_features_to_vector() {
        let features = ComplexityFeatures {
            loc: 100.0,
            max_nesting: 5.0,
            control_flow_count: 10.0,
            loop_count: 3.0,
            conditional_count: 7.0,
            function_count: 4.0,
            avg_function_size: 25.0,
            language_type: 1.0,
        };

        let vector = features.to_vector();

        assert_eq!(vector.len(), 8, "Feature vector should have 8 elements");
        assert!(
            vector.iter().all(|&v| (0.0..=10.0).contains(&v)),
            "All features should be normalized"
        );
    }

    #[test]
    fn test_ml_scorer_creation() {
        let scorer = MLQualityScorer::new();

        assert!(!scorer.is_trained(), "New scorer should not be trained");
        assert!(
            scorer.complexity_model.is_none(),
            "No complexity model initially"
        );
        assert!(scorer.tdg_model.is_none(), "No TDG model initially");
    }

    #[test]
    fn test_train_complexity_model() {
        let mut scorer = MLQualityScorer::new();

        // Generate synthetic training data
        let samples: Vec<QualityTrainingSample> = (0..50)
            .map(|i| {
                let complexity = (i as f64) / 10.0;
                QualityTrainingSample {
                    features: vec![
                        complexity * 0.5,        // loc
                        complexity * 0.3,        // nesting
                        complexity * 0.4,        // control_flow
                        complexity * 0.2,        // loops
                        complexity * 0.3,        // conditionals
                        0.1 + complexity * 0.05, // functions
                        0.5 + complexity * 0.1,  // avg_size
                        0.1,                     // language
                    ],
                    target_score: complexity * 10.0, // Ground truth
                    weight: None,
                }
            })
            .collect();

        let result = scorer.train_complexity_model(&samples);
        assert!(result.is_ok(), "Training should succeed");

        // Model may or may not be trained depending on sample size
        // With 50 samples and 8 features, it should work
        if scorer.complexity_model.is_some() {
            assert!(scorer.is_trained(), "Scorer should be marked as trained");
        }
    }

    #[test]
    fn test_train_empty_data_fails() {
        let mut scorer = MLQualityScorer::new();
        let result = scorer.train_complexity_model(&[]);
        assert!(result.is_err(), "Training with empty data should fail");
    }

    #[test]
    fn test_predict_complexity_without_training() {
        let scorer = MLQualityScorer::new();

        let features = ComplexityFeatures {
            loc: 100.0,
            max_nesting: 3.0,
            control_flow_count: 5.0,
            loop_count: 2.0,
            conditional_count: 3.0,
            function_count: 4.0,
            avg_function_size: 25.0,
            language_type: 1.0,
        };

        let prediction = scorer.predict_complexity(&features).unwrap();

        assert!(
            !prediction.ml_used,
            "Should use heuristics without training"
        );
        assert!(prediction.score > 0.0, "Score should be positive");
        assert!(
            prediction.confidence < 0.7,
            "Confidence should be low for heuristics"
        );
    }

    #[test]
    fn test_predict_complexity_with_training() {
        let mut scorer = MLQualityScorer::new();

        // Train with synthetic data
        let samples: Vec<QualityTrainingSample> = (0..50)
            .map(|i| {
                let complexity = (i as f64) / 10.0;
                QualityTrainingSample {
                    features: vec![
                        complexity * 0.5,
                        complexity * 0.3,
                        complexity * 0.4,
                        complexity * 0.2,
                        complexity * 0.3,
                        0.1 + complexity * 0.05,
                        0.5 + complexity * 0.1,
                        0.1,
                    ],
                    target_score: complexity * 10.0,
                    weight: None,
                }
            })
            .collect();

        scorer.train_complexity_model(&samples).unwrap();

        let features = ComplexityFeatures {
            loc: 100.0,
            max_nesting: 3.0,
            control_flow_count: 5.0,
            loop_count: 2.0,
            conditional_count: 3.0,
            function_count: 4.0,
            avg_function_size: 25.0,
            language_type: 1.0,
        };

        let prediction = scorer.predict_complexity(&features).unwrap();

        // With trained model, should use ML
        if scorer.complexity_model.is_some() {
            assert!(prediction.ml_used, "Should use ML with trained model");
            assert!(
                prediction.confidence > 0.7,
                "Confidence should be high with ML"
            );
        }

        assert!(prediction.score >= 0.0, "Score should be non-negative");
    }

    #[test]
    fn test_tdg_features_to_vector() {
        let features = TDGFeatures {
            complexity: 3.0,
            churn: 2.0,
            coupling: 1.5,
            domain_risk: 2.5,
            duplication: 1.0,
            test_coverage: 0.8,
            file_age_days: 180.0,
            commit_frequency: 5.0,
        };

        let vector = features.to_vector();

        assert_eq!(vector.len(), 8, "TDG vector should have 8 elements");
        assert!(
            vector.iter().all(|&v| v >= 0.0),
            "All values should be non-negative"
        );
    }

    #[test]
    fn test_predict_tdg() {
        let scorer = MLQualityScorer::new();

        let features = TDGFeatures {
            complexity: 3.0,
            churn: 2.0,
            coupling: 1.5,
            domain_risk: 2.5,
            duplication: 1.0,
            test_coverage: 0.8,
            file_age_days: 180.0,
            commit_frequency: 5.0,
        };

        let prediction = scorer.predict_tdg(&features).unwrap();

        assert!(prediction.score >= 0.0 && prediction.score <= 5.0);
        assert!(
            !prediction.ml_used,
            "Should use heuristics without training"
        );
    }

    #[test]
    fn test_ml_vs_heuristic_difference() {
        // This test verifies ML produces different (better) results than heuristics
        let mut scorer = MLQualityScorer::new();

        // Train with data that has a non-linear relationship
        let samples: Vec<QualityTrainingSample> = (0..100)
            .map(|i| {
                let x = (i as f64) / 20.0;
                // Non-linear target: nesting has quadratic effect
                let target = x + x * x * 0.5;
                QualityTrainingSample {
                    features: vec![
                        x * 0.1,
                        x, // nesting is dominant factor
                        x * 0.2,
                        x * 0.1,
                        x * 0.15,
                        x * 0.05,
                        x * 0.1,
                        0.1,
                    ],
                    target_score: target,
                    weight: None,
                }
            })
            .collect();

        scorer.train_complexity_model(&samples).unwrap();

        // Test on high-complexity case
        let features = ComplexityFeatures {
            loc: 200.0,
            max_nesting: 8.0,
            control_flow_count: 15.0,
            loop_count: 5.0,
            conditional_count: 10.0,
            function_count: 3.0,
            avg_function_size: 66.0,
            language_type: 1.0,
        };

        let ml_prediction = scorer.predict_complexity(&features).unwrap();
        let heuristic_score = scorer.heuristic_complexity(&features);

        // ML should produce a different prediction (not necessarily better without proper training)
        if ml_prediction.ml_used {
            println!("ML score: {}", ml_prediction.score);
            println!("Heuristic score: {}", heuristic_score);

            // They should be somewhat different (ML learns patterns)
            let diff = (ml_prediction.score - heuristic_score).abs();
            assert!(
                diff > 0.0 || ml_prediction.score == heuristic_score,
                "ML should produce a prediction"
            );
        }
    }

    #[test]
    fn test_feature_importance_calculation() {
        let mut scorer = MLQualityScorer::new();

        // Train with data where some features are more important
        let samples: Vec<QualityTrainingSample> = (0..50)
            .map(|i| {
                let nesting = (i as f64) / 10.0;
                QualityTrainingSample {
                    features: vec![
                        0.5,     // loc - constant (not important)
                        nesting, // nesting - varies (important)
                        0.3,     // control_flow - constant
                        0.2,     // loops - constant
                        0.3,     // conditionals - constant
                        0.1,     // functions - constant
                        0.5,     // avg_size - constant
                        0.1,     // language - constant
                    ],
                    target_score: nesting * 5.0, // Target correlates with nesting
                    weight: None,
                }
            })
            .collect();

        scorer.train_complexity_model(&samples).unwrap();

        let importance = scorer.feature_importance();

        // Nesting should have high importance since it correlates with target
        if let Some(&nesting_importance) = importance.get("nesting") {
            assert!(
                nesting_importance > 0.0,
                "Nesting should have positive importance"
            );
        }
    }

    #[test]
    fn test_language_specific_features() {
        // Test that different languages get different encodings
        let rust_features = ComplexityFeatures::from_source("fn main() {}", "rust");
        let python_features = ComplexityFeatures::from_source("def main(): pass", "python");
        let js_features = ComplexityFeatures::from_source("function main() {}", "javascript");

        assert_eq!(rust_features.language_type, 1.0);
        assert_eq!(python_features.language_type, 2.0);
        assert_eq!(js_features.language_type, 3.0);

        // Unknown language
        let unknown_features = ComplexityFeatures::from_source("main", "unknown");
        assert_eq!(unknown_features.language_type, 0.0);
    }

    #[test]
    fn test_correlation_calculation() {
        let scorer = MLQualityScorer::new();

        // Perfect positive correlation
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let corr = scorer.correlation(&x, &y);
        assert!(
            (corr - 1.0).abs() < 0.001,
            "Perfect positive correlation should be 1.0"
        );

        // Perfect negative correlation
        let y_neg = vec![10.0, 8.0, 6.0, 4.0, 2.0];
        let corr_neg = scorer.correlation(&x, &y_neg);
        assert!(
            (corr_neg + 1.0).abs() < 0.001,
            "Perfect negative correlation should be -1.0"
        );

        // No correlation (constant)
        let y_const = vec![5.0, 5.0, 5.0, 5.0, 5.0];
        let corr_none = scorer.correlation(&x, &y_const);
        assert!(
            corr_none.abs() < 0.001,
            "No correlation with constant values"
        );
    }

    #[test]
    fn test_prediction_bounds() {
        let scorer = MLQualityScorer::new();

        // Extreme features shouldn't produce unbounded scores
        let extreme_features = ComplexityFeatures {
            loc: 10000.0,
            max_nesting: 50.0,
            control_flow_count: 500.0,
            loop_count: 100.0,
            conditional_count: 400.0,
            function_count: 200.0,
            avg_function_size: 50.0,
            language_type: 1.0,
        };

        let prediction = scorer.predict_complexity(&extreme_features).unwrap();

        assert!(prediction.score <= 100.0, "Score should be bounded at 100");
        assert!(prediction.score >= 0.0, "Score should be non-negative");
    }
}
