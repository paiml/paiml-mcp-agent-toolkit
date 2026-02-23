#![cfg_attr(coverage_nightly, coverage(off))]

#[cfg(test)]
mod tests {
    use super::super::calculator::PerfectionScoreCalculator;
    use super::super::types::{
        CategoryScore, CategoryWeights, PerfectionScoreResult, MAX_PERFECTION_SCORE,
    };
    use std::path::Path;

    #[test]
    fn test_category_weights_sum_to_200() {
        let weights = CategoryWeights::default();
        let sum = weights.tdg
            + weights.repo_score
            + weights.rust_score
            + weights.popper_score
            + weights.test_coverage
            + weights.mutation
            + weights.documentation
            + weights.performance;
        assert_eq!(sum, MAX_PERFECTION_SCORE);
    }

    #[test]
    fn test_category_score_calculation() {
        let score = CategoryScore::new("Test", 80.0, 40);
        assert_eq!(score.earned_points, 32.0);
        assert_eq!(score.grade, "B-"); // 80 is in B- range (80-82)
    }

    #[test]
    fn test_overall_grade_thresholds() {
        // Normalized percentage grading (score/200 * 100)
        // 190/200 = 95% → A+, 180/200 = 90% → A, etc.
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(198.0), "A+"); // 99%
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(190.0), "A+"); // 95%
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(184.0), "A"); // 92%
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(174.0), "A-"); // 87%
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(164.0), "B+"); // 82%
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(150.0), "B"); // 75%
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(130.0), "C"); // 65%
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(110.0), "D"); // 55%
        assert_eq!(PerfectionScoreResult::calculate_overall_grade(80.0), "F"); // 40%
    }

    #[test]
    fn test_perfection_score_result() {
        let categories = vec![
            CategoryScore::new("TDG", 80.0, 40),         // 32 pts
            CategoryScore::new("Repo", 70.0, 30),        // 21 pts
            CategoryScore::new("Rust", 75.0, 30),        // 22.5 pts
            CategoryScore::new("Popper", 65.0, 25),      // 16.25 pts
            CategoryScore::new("Coverage", 90.0, 25),    // 22.5 pts
            CategoryScore::new("Mutation", 60.0, 20),    // 12 pts
            CategoryScore::new("Docs", 70.0, 15),        // 10.5 pts
            CategoryScore::new("Performance", 85.0, 15), // 12.75 pts
        ];
        let result = PerfectionScoreResult::new(categories);

        // Total: 32 + 21 + 22.5 + 16.25 + 22.5 + 12 + 10.5 + 12.75 = 149.5
        assert!((result.total_score - 149.5).abs() < 0.01);
        assert_eq!(result.grade, "B"); // 149.5/200 = 74.75% → B range (70-79%)
    }

    #[tokio::test]
    #[ignore] // Times out in coverage builds (>120s)
    async fn test_calculator_fast_mode() {
        let calc = PerfectionScoreCalculator::new().fast_mode(true);
        let result = calc.calculate(Path::new(".")).await.unwrap();

        // Should have all 8 categories
        assert_eq!(result.categories.len(), 8);
        // Score should be in valid range
        assert!(result.total_score >= 0.0 && result.total_score <= 200.0);
    }
}
