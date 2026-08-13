#![cfg_attr(coverage_nightly, coverage(off))]
/// Property-based and edge case tests for perfection_score module
#[cfg(test)]
mod property_tests {
    use super::super::calculator::PerfectionScoreCalculator;
    use super::super::types::{
        CategoryScore, CategoryWeights, PerfectionScoreResult, MAX_PERFECTION_SCORE,
    };
    use proptest::prelude::*;
    use std::fs;
    use tempfile::TempDir;

    // ============================================================================
    // Property-Based Tests
    // ============================================================================

    proptest! {
        /// Property: Earned points are always proportional to raw score and max points
        #[test]
        fn prop_earned_points_proportional(raw_score in 0.0f64..=100.0, max_points in 1u16..=100) {
            let score = CategoryScore::new("Test", raw_score, max_points);
            let expected = (raw_score / 100.0) * f64::from(max_points);
            prop_assert!((score.earned_points - expected).abs() < 0.001);
        }

        /// Property: Grade is always one of the valid grades
        #[test]
        fn prop_grade_is_valid(raw_score in 0.0f64..=100.0) {
            let score = CategoryScore::new("Test", raw_score, 40);
            let valid_grades = ["A+", "A", "A-", "B+", "B", "B-", "C+", "C", "C-", "D+", "D", "D-", "F"];
            prop_assert!(valid_grades.contains(&score.grade.as_str()));
        }

        /// Property: Total score is sum of earned points
        #[test]
        fn prop_total_score_is_sum(
            s1 in 0.0f64..=100.0,
            s2 in 0.0f64..=100.0,
            s3 in 0.0f64..=100.0
        ) {
            let categories = vec![
                CategoryScore::new("A", s1, 40),
                CategoryScore::new("B", s2, 30),
                CategoryScore::new("C", s3, 30),
            ];
            let result = PerfectionScoreResult::new(categories.clone());
            let expected: f64 = categories.iter().map(|c| c.earned_points).sum();
            prop_assert!((result.total_score - expected).abs() < 0.001);
        }

        /// Property: Score is always in valid range
        #[test]
        fn prop_score_in_valid_range(raw_score in 0.0f64..=100.0) {
            let score = CategoryScore::new("Test", raw_score, 40);
            prop_assert!(score.earned_points >= 0.0);
            prop_assert!(score.earned_points <= 40.0);
        }

        /// Property: Target gap calculation is correct
        #[test]
        fn prop_target_gap_correct(score in 0.0f64..=100.0, target in 0u16..=200) {
            let categories = vec![CategoryScore::new("Test", score, 100)];
            let result = PerfectionScoreResult::new(categories).with_target(target);
            let expected_gap = f64::from(target) - (score / 100.0) * 100.0;
            prop_assert!((result.target_gap.unwrap() - expected_gap).abs() < 0.001);
        }

        /// Property: Category weights always sum to MAX_PERFECTION_SCORE
        #[test]
        fn prop_weights_sum_to_max(_dummy in 0u8..1) {
            let weights = CategoryWeights::default();
            let sum = weights.tdg
                + weights.repo_score
                + weights.rust_score
                + weights.popper_score
                + weights.test_coverage
                + weights.mutation
                + weights.documentation
                + weights.performance;
            prop_assert_eq!(sum, MAX_PERFECTION_SCORE);
        }

        /// Property: Overall grade is monotonic with score
        #[test]
        fn prop_grade_monotonic(score1 in 0.0f64..=200.0, score2 in 0.0f64..=200.0) {
            let grade1 = PerfectionScoreResult::calculate_overall_grade(score1);
            let grade2 = PerfectionScoreResult::calculate_overall_grade(score2);

            // Define grade ordering
            fn grade_value(grade: &str) -> u8 {
                match grade {
                    "A+" => 12, "A" => 11, "A-" => 10,
                    "B+" => 9, "B" => 8, "B-" => 7,
                    "C+" => 6, "C" => 5, "C-" => 4,
                    "D+" => 3, "D" => 2, "D-" => 1,
                    "F" => 0,
                    _ => 0,
                }
            }

            if score1 > score2 + 1.0 {
                prop_assert!(grade_value(&grade1) >= grade_value(&grade2));
            }
        }

        /// Property: Category score with details preserves all fields
        #[test]
        fn prop_with_details_preserves_fields(
            raw_score in 0.0f64..=100.0,
            max_points in 1u16..=100
        ) {
            let score = CategoryScore::new("Test", raw_score, max_points);
            let score_with_details = score.clone().with_details("some details");

            prop_assert_eq!(score.name, score_with_details.name);
            prop_assert_eq!(score.raw_score, score_with_details.raw_score);
            prop_assert_eq!(score.max_points, score_with_details.max_points);
            prop_assert_eq!(score.earned_points, score_with_details.earned_points);
            prop_assert_eq!(score.grade, score_with_details.grade);
            prop_assert!(score_with_details.details.is_some());
        }

        /// Property: Recommendations always generated for low scores
        #[test]
        fn prop_recommendations_for_low_scores(raw_score in 0.0f64..50.0) {
            let categories = vec![CategoryScore::new("Critical", raw_score, 100)];
            let result = PerfectionScoreResult::new(categories);
            prop_assert!(!result.recommendations.is_empty());
            prop_assert!(result.recommendations.iter().any(|r| r.contains("critical")));
        }

        /// Property: High scoring categories get healthy message
        #[test]
        fn prop_healthy_for_high_scores(raw_score in 85.0f64..=100.0) {
            let categories = vec![CategoryScore::new("Good", raw_score, 100)];
            let result = PerfectionScoreResult::new(categories);
            prop_assert!(result.recommendations.iter().any(|r| r.contains("healthy")));
        }
    }

    // ============================================================================
    // Edge Case Tests
    // ============================================================================

    #[test]
    fn test_category_score_extreme_values() {
        // Very small max points
        let score = CategoryScore::new("Test", 50.0, 1);
        assert_eq!(score.earned_points, 0.5);

        // Very large max points
        let score = CategoryScore::new("Test", 50.0, u16::MAX);
        assert!((score.earned_points - (f64::from(u16::MAX) / 2.0)).abs() < 1.0);
    }

    #[test]
    fn test_category_score_floating_point_precision() {
        // Test that floating point precision is reasonable
        let score = CategoryScore::new("Test", 33.333333, 100);
        assert!((score.earned_points - 33.333333).abs() < 0.0001);
    }

    #[test]
    fn test_perfection_score_result_with_negative_target_gap() {
        let categories = vec![CategoryScore::new("TDG", 100.0, 100)];
        let result = PerfectionScoreResult::new(categories).with_target(50);
        assert!(result.target_gap.unwrap() < 0.0);
    }

    #[test]
    fn test_max_perfection_score_constant() {
        assert_eq!(MAX_PERFECTION_SCORE, 200);
    }

    #[test]
    fn test_category_score_name_special_characters() {
        let score = CategoryScore::new("Test-Category_123", 80.0, 40);
        assert_eq!(score.name, "Test-Category_123");
    }

    #[test]
    fn test_category_score_empty_name() {
        let score = CategoryScore::new("", 80.0, 40);
        assert_eq!(score.name, "");
    }

    #[test]
    fn test_perfection_score_result_many_categories() {
        let categories: Vec<CategoryScore> = (0..100)
            .map(|i| CategoryScore::new(&format!("Category{}", i), 80.0, 2))
            .collect();
        let result = PerfectionScoreResult::new(categories);
        // Use epsilon comparison for floating point (100 * 1.6 earned points)
        assert!(
            (result.total_score - 160.0).abs() < 0.001,
            "Expected ~160.0, got {}",
            result.total_score
        );
    }

    /// The workspace-aware lookups still work — they just look for *results*
    /// now instead of empty directories (#938).
    #[tokio::test]
    async fn test_get_performance_score_workspace_structure() {
        let temp_dir = TempDir::new().unwrap();
        let change = temp_dir.path().join("server/target/criterion/parse/change");
        fs::create_dir_all(&change).unwrap();
        fs::write(
            change.join("estimates.json"),
            r#"{"mean":{"point_estimate":-0.10}}"#,
        )
        .unwrap();

        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_performance_score(temp_dir.path()).await.unwrap();
        assert_eq!(score, 100.0, "the only benchmark got faster");

        // An empty server/benches directory is not a benchmark run.
        let empty = TempDir::new().unwrap();
        fs::create_dir_all(empty.path().join("server/benches")).unwrap();
        assert!(calc.get_performance_score(empty.path()).await.is_err());
    }

    #[tokio::test]
    async fn test_get_mutation_score_server_mutants_toml() {
        let temp_dir = TempDir::new().unwrap();
        let out = temp_dir.path().join("server/mutants.out");
        fs::create_dir_all(&out).unwrap();
        fs::write(out.join("caught.txt"), "a\nb\n").unwrap();
        fs::write(out.join("missed.txt"), "c\n").unwrap();

        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_mutation_score(temp_dir.path()).await.unwrap();
        assert!((score - (2.0 / 3.0 * 100.0)).abs() < 0.001, "score {score}");

        // A config file alone is not a mutation run.
        let configured = TempDir::new().unwrap();
        fs::create_dir(configured.path().join("server")).unwrap();
        fs::write(configured.path().join("server/mutants.toml"), "[mutants]").unwrap();
        assert!(calc.get_mutation_score(configured.path()).await.is_err());
    }

    #[tokio::test]
    async fn test_get_coverage_score_workspace_cache() {
        let temp_dir = TempDir::new().unwrap();
        // Create server/.pmat-metrics/coverage.json
        let metrics_dir = temp_dir.path().join("server/.pmat-metrics");
        fs::create_dir_all(&metrics_dir).unwrap();
        fs::write(metrics_dir.join("coverage.json"), r#"{"coverage": 92.5}"#).unwrap();
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_coverage_score(temp_dir.path()).await.unwrap();
        assert_eq!(score, 92.5);
    }

    #[tokio::test]
    async fn test_get_coverage_score_with_tokio_tests() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir(temp_dir.path().join("src")).unwrap();
        fs::write(
            temp_dir.path().join("src/lib.rs"),
            r#"
            #[tokio::test]
            async fn test1() {}
            #[tokio::test]
            async fn test2() {}
            "#,
        )
        .unwrap();
        let calc = PerfectionScoreCalculator::new();
        // Two `#[tokio::test]` attributes are two tests, not a coverage figure.
        assert!(calc.get_coverage_score(temp_dir.path()).await.is_err());
    }

    #[tokio::test]
    async fn test_get_documentation_score_partial() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("README.md"), "# Test").unwrap();
        fs::write(temp_dir.path().join("CHANGELOG.md"), "# Changes").unwrap();
        let calc = PerfectionScoreCalculator::new();
        let score = calc.get_documentation_score(temp_dir.path()).await;
        assert_eq!(score, 60.0); // 40 + 20
    }

    // ============================================================================
    // Calculator Integration Tests (using temp dirs to avoid slow external services)
    // ============================================================================

    #[tokio::test]
    async fn test_calculator_fast_mode_mutation_default() {
        let temp_dir = TempDir::new().unwrap();
        let calc = PerfectionScoreCalculator::new().fast_mode(true);

        // In fast mode mutation testing is not run at all, so the category is
        // reported as not measured and excluded from the denominator.
        //
        // This used to assert `raw_score == 50.0` with the comment "default
        // credit in fast mode" — a flat half-credit handed to a measurement
        // that never ran (and identical for a real repo, an empty directory and
        // a nonexistent path), while the category's own details string admitted
        // "Skipped (fast mode)". Unearned points inside a number presented as a
        // grade are exactly the defect; the category now scores nothing and
        // costs nothing.
        let result = calc.calculate(temp_dir.path()).await.unwrap();

        let mutation_cat = result
            .categories
            .iter()
            .find(|c| c.name == "Mutation Testing")
            .unwrap();
        assert_eq!(mutation_cat.raw_score, 0.0);
        assert_eq!(
            mutation_cat.earned_points, 0.0,
            "a check that never ran cannot earn points"
        );
        assert_eq!(
            mutation_cat.max_points, 0,
            "a check that never ran must leave the denominator"
        );
        assert_eq!(mutation_cat.grade, "N/A");
        let details = mutation_cat
            .details
            .as_ref()
            .expect("skipped category must say why");
        assert!(
            details.contains("Not measured") && details.contains("--fast"),
            "unexpected details: {details}"
        );
    }

    #[test]
    fn test_category_weights_copy_trait() {
        let weights = CategoryWeights::default();
        let copy = weights; // Copy
        assert_eq!(weights.tdg, copy.tdg);
    }

    // ============================================================================
    // Serialization Round-Trip Tests
    // ============================================================================

    #[test]
    fn test_category_score_serde_roundtrip() {
        let score = CategoryScore::new("Test", 75.5, 40).with_details("Test details");
        let json = serde_json::to_string(&score).unwrap();
        let deserialized: CategoryScore = serde_json::from_str(&json).unwrap();

        assert_eq!(score.name, deserialized.name);
        assert_eq!(score.raw_score, deserialized.raw_score);
        assert_eq!(score.max_points, deserialized.max_points);
        assert_eq!(score.earned_points, deserialized.earned_points);
        assert_eq!(score.grade, deserialized.grade);
        assert_eq!(score.details, deserialized.details);
    }

    #[test]
    fn test_perfection_score_result_serde_roundtrip() {
        let categories = vec![
            CategoryScore::new("TDG", 80.0, 40),
            CategoryScore::new("Repo", 75.0, 30),
        ];
        let result = PerfectionScoreResult::new(categories).with_target(150);
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: PerfectionScoreResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result.total_score, deserialized.total_score);
        assert_eq!(result.max_score, deserialized.max_score);
        assert_eq!(result.grade, deserialized.grade);
        assert_eq!(result.categories.len(), deserialized.categories.len());
        assert_eq!(result.target_gap, deserialized.target_gap);
    }
}
