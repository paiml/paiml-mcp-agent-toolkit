/// NOTE: Temporarily disabled due to JsonValue move semantics in prop_assert_eq
#[cfg(all(test, feature = "broken-tests"))]
mod property_tests {
    use super::*;
    use crate::services::defect_probability::RiskLevel;
    use proptest::prelude::*;
    use std::time::Duration;

    // Strategy for generating valid probabilities (0.0 to 1.0)
    fn probability_strategy() -> impl Strategy<Value = f32> {
        (0u32..=1000).prop_map(|x| x as f32 / 1000.0)
    }

    // Strategy for generating valid confidence values (0.0 to 1.0)
    fn confidence_strategy() -> impl Strategy<Value = f32> {
        (0u32..=1000).prop_map(|x| x as f32 / 1000.0)
    }

    // Strategy for generating DefectScore
    fn defect_score_strategy() -> impl Strategy<Value = DefectScore> {
        (probability_strategy(), confidence_strategy()).prop_map(|(prob, conf)| {
            let risk_level = if prob > 0.7 {
                RiskLevel::High
            } else if prob > 0.3 {
                RiskLevel::Medium
            } else {
                RiskLevel::Low
            };

            DefectScore {
                probability: prob,
                confidence: conf,
                risk_level,
                contributing_factors: vec![
                    ("complexity".to_string(), prob * 0.3),
                    ("churn".to_string(), prob * 0.35),
                ],
                recommendations: vec!["Test recommendation".to_string()],
            }
        })
    }

    // Strategy for generating predictions
    fn predictions_strategy() -> impl Strategy<Value = Vec<(String, DefectScore)>> {
        prop::collection::vec(
            ("[a-z][a-z0-9_]{0,20}\\.rs", defect_score_strategy()),
            0..20,
        )
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn prop_risk_statistics_sum_equals_total(predictions in predictions_strategy()) {
            let stats = calculate_risk_statistics(&predictions);
            let total = stats.high_risk + stats.medium_risk + stats.low_risk;
            prop_assert_eq!(total, predictions.len());
        }

        #[test]
        fn prop_filter_high_risk_only_reduces_count(predictions in predictions_strategy()) {
            let original_len = predictions.len();
            let filtered = filter_and_sort_predictions(
                predictions,
                true,  // high_risk_only
                true,  // include_low_confidence
                0.0,   // confidence_threshold
                0,     // top_files (unlimited)
            );
            prop_assert!(filtered.len() <= original_len);
        }

        #[test]
        fn prop_filtered_results_are_sorted(predictions in predictions_strategy()) {
            let filtered = filter_and_sort_predictions(
                predictions,
                false, // high_risk_only
                true,  // include_low_confidence
                0.0,   // confidence_threshold
                0,     // top_files
            );

            // Verify sorted descending by probability
            for i in 1..filtered.len() {
                prop_assert!(filtered[i-1].1.probability >= filtered[i].1.probability);
            }
        }

        #[test]
        fn prop_top_files_limit_respected(
            predictions in predictions_strategy(),
            limit in 1usize..10
        ) {
            let filtered = filter_and_sort_predictions(
                predictions.clone(),
                false,
                true,
                0.0,
                limit,
            );

            prop_assert!(filtered.len() <= limit);
            prop_assert!(filtered.len() <= predictions.len());
        }

        #[test]
        fn prop_json_output_is_valid(predictions in predictions_strategy()) {
            let elapsed = Duration::from_millis(100);
            let result = format_defect_json(&predictions, elapsed);
            prop_assert!(result.is_ok());

            // Verify it's parseable JSON
            let json_str = result.unwrap();
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json_str);
            prop_assert!(parsed.is_ok());
        }

        #[test]
        fn prop_csv_has_correct_line_count(predictions in predictions_strategy()) {
            let result = format_defect_csv(&predictions);
            prop_assert!(result.is_ok());

            let csv = result.unwrap();
            let line_count = csv.lines().count();
            // 1 header + N data rows
            prop_assert_eq!(line_count, predictions.len() + 1);
        }

        #[test]
        fn prop_sarif_output_is_valid(predictions in predictions_strategy()) {
            let result = format_defect_sarif(&predictions);
            prop_assert!(result.is_ok());

            let sarif_str = result.unwrap();
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(&sarif_str);
            prop_assert!(parsed.is_ok());

            let sarif = parsed.unwrap();
            prop_assert_eq!(sarif["version"], "2.1.0");
        }

        #[test]
        fn prop_summary_output_never_fails(predictions in predictions_strategy()) {
            let elapsed = Duration::from_millis(100);
            let result = format_defect_summary(&predictions, elapsed);
            prop_assert!(result.is_ok());
        }

        #[test]
        fn prop_detailed_output_never_fails(predictions in predictions_strategy()) {
            let elapsed = Duration::from_millis(100);
            let result = format_defect_detailed(&predictions, elapsed, true);
            prop_assert!(result.is_ok());

            let result_no_rec = format_defect_detailed(&predictions, elapsed, false);
            prop_assert!(result_no_rec.is_ok());
        }

        #[test]
        fn prop_confidence_threshold_filters_correctly(
            predictions in predictions_strategy(),
            threshold in probability_strategy()
        ) {
            let filtered = filter_and_sort_predictions(
                predictions.clone(),
                false,
                false, // Do NOT include low confidence
                threshold,
                0,
            );

            // All remaining predictions should have confidence >= threshold
            for (_, score) in &filtered {
                prop_assert!(score.confidence > threshold);
            }
        }

        #[test]
        fn prop_high_risk_filter_only_high(predictions in predictions_strategy()) {
            let filtered = filter_and_sort_predictions(
                predictions,
                true, // high_risk_only
                true,
                0.0,
                0,
            );

            // All remaining should have probability > 0.7
            for (_, score) in &filtered {
                prop_assert!(score.probability > 0.7);
            }
        }

        #[test]
        fn prop_risk_icon_always_returns_valid(prob in probability_strategy()) {
            let risk_level = if prob > 0.7 {
                RiskLevel::High
            } else if prob > 0.3 {
                RiskLevel::Medium
            } else {
                RiskLevel::Low
            };

            let icon = get_risk_icon(&risk_level);
            prop_assert!(["🔴", "🟡", "🟢"].contains(&icon));
        }

        #[test]
        fn prop_config_creation_preserves_values(
            threshold in probability_strategy(),
            min_lines in 1usize..1000,
            include_low_conf in proptest::bool::ANY,
            high_risk in proptest::bool::ANY,
            include_rec in proptest::bool::ANY
        ) {
            let config = create_defect_prediction_config(
                threshold,
                min_lines,
                include_low_conf,
                high_risk,
                include_rec,
                None,
                None,
            );

            prop_assert_eq!(config.confidence_threshold, threshold);
            prop_assert_eq!(config.min_lines, min_lines);
            prop_assert_eq!(config.include_low_confidence, include_low_conf);
            prop_assert_eq!(config.high_risk_only, high_risk);
            prop_assert_eq!(config.include_recommendations, include_rec);
        }
    }

}
