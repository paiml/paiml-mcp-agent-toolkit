// Tests for popper_score models.
// Included by models.rs - shares parent module scope (no `use` imports here).

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // RED TESTS - Falsifiability Gateway (Annotation 2)
    // ========================================================================

    #[test]
    fn test_gateway_fails_when_falsifiability_below_15() {
        let mut score = PopperScore::new();
        score.categories.falsifiability.earned = 14.0; // Below threshold
        score.categories.reproducibility.earned = 25.0;
        score.categories.transparency.earned = 20.0;
        score.categories.statistical_rigor.earned = 15.0;
        score.categories.historical_integrity.earned = 10.0;

        score.calculate();

        assert!(!score.gateway_passed);
        assert_eq!(score.normalized_score, 0.0);
        assert_eq!(score.grade, PopperGrade::InsufficientFalsifiability);
    }

    #[test]
    fn test_gateway_passes_when_falsifiability_at_15() {
        let mut score = PopperScore::new();
        score.categories.falsifiability.earned = 15.0; // At threshold
        score.categories.reproducibility.earned = 20.0;
        score.categories.transparency.earned = 15.0;
        score.categories.statistical_rigor.earned = 10.0;
        score.categories.historical_integrity.earned = 8.0;

        score.calculate();

        assert!(score.gateway_passed);
        assert!(score.normalized_score > 0.0);
    }

    #[test]
    fn test_gateway_passes_when_falsifiability_above_15() {
        let mut score = PopperScore::new();
        score.categories.falsifiability.earned = 20.0;
        score.categories.reproducibility.earned = 20.0;
        score.categories.transparency.earned = 15.0;
        score.categories.statistical_rigor.earned = 10.0;
        score.categories.historical_integrity.earned = 8.0;

        score.calculate();

        assert!(score.gateway_passed);
        assert!(score.normalized_score > 70.0);
    }

    // ========================================================================
    // RED TESTS - Score Normalization (Annotation 8)
    // ========================================================================

    #[test]
    fn test_normalization_without_ml() {
        let mut score = PopperScore::new();
        score.categories.falsifiability.earned = 22.0;
        score.categories.reproducibility.earned = 23.0;
        score.categories.transparency.earned = 17.0;
        score.categories.statistical_rigor.earned = 12.0;
        score.categories.historical_integrity.earned = 8.0;
        // ML is N/A by default

        score.calculate();

        // Expected: 82/95 = 86.3%
        assert!(score.gateway_passed);
        assert_eq!(score.max_available, 95.0); // 100 - 5 for N/A ML
        assert!((score.normalized_score - 86.3).abs() < 0.5);
        assert_eq!(score.grade, PopperGrade::AMinus);
    }

    #[test]
    fn test_normalization_with_ml() {
        let mut score = PopperScore::new();
        score.categories.falsifiability.earned = 22.0;
        score.categories.reproducibility.earned = 23.0;
        score.categories.transparency.earned = 17.0;
        score.categories.statistical_rigor.earned = 12.0;
        score.categories.historical_integrity.earned = 8.0;
        score.categories.ml_reproducibility.mark_applicable();
        score.categories.ml_reproducibility.earned = 4.0;

        score.calculate();

        // Expected: 86/100 = 86%
        assert!(score.gateway_passed);
        assert_eq!(score.max_available, 100.0);
        assert_eq!(score.normalized_score, 86.0);
        assert_eq!(score.grade, PopperGrade::AMinus);
    }

    // ========================================================================
    // RED TESTS - Grade Assignment (Annotation 9)
    // ========================================================================

    #[test]
    fn test_grade_thresholds() {
        assert_eq!(
            PopperGrade::from_normalized_score(100.0),
            PopperGrade::APlus
        );
        assert_eq!(PopperGrade::from_normalized_score(95.0), PopperGrade::APlus);
        assert_eq!(PopperGrade::from_normalized_score(94.9), PopperGrade::A);
        assert_eq!(PopperGrade::from_normalized_score(90.0), PopperGrade::A);
        assert_eq!(
            PopperGrade::from_normalized_score(89.9),
            PopperGrade::AMinus
        );
        assert_eq!(
            PopperGrade::from_normalized_score(85.0),
            PopperGrade::AMinus
        );
        assert_eq!(PopperGrade::from_normalized_score(84.9), PopperGrade::BPlus);
        assert_eq!(PopperGrade::from_normalized_score(80.0), PopperGrade::BPlus);
        assert_eq!(PopperGrade::from_normalized_score(79.9), PopperGrade::B);
        assert_eq!(PopperGrade::from_normalized_score(70.0), PopperGrade::B);
        assert_eq!(PopperGrade::from_normalized_score(69.9), PopperGrade::C);
        assert_eq!(PopperGrade::from_normalized_score(60.0), PopperGrade::C);
        assert_eq!(PopperGrade::from_normalized_score(59.9), PopperGrade::D);
        assert_eq!(PopperGrade::from_normalized_score(50.0), PopperGrade::D);
        assert_eq!(PopperGrade::from_normalized_score(49.9), PopperGrade::F);
        assert_eq!(PopperGrade::from_normalized_score(0.0), PopperGrade::F);
    }

    #[test]
    fn test_grade_meets_standards() {
        assert!(PopperGrade::APlus.meets_standards());
        assert!(PopperGrade::A.meets_standards());
        assert!(PopperGrade::AMinus.meets_standards());
        assert!(!PopperGrade::BPlus.meets_standards());
        assert!(!PopperGrade::B.meets_standards());
        assert!(!PopperGrade::C.meets_standards());
        assert!(!PopperGrade::D.meets_standards());
        assert!(!PopperGrade::F.meets_standards());
        assert!(!PopperGrade::InsufficientFalsifiability.meets_standards());
    }

    #[test]
    fn test_grade_interpretation_not_science_removed() {
        // Verify we use "Insufficient Rigor" not "NOT SCIENCE" (Annotation 9)
        let f_interpretation = PopperGrade::F.interpretation();
        assert!(!f_interpretation.contains("NOT SCIENCE"));
        assert!(f_interpretation.contains("Insufficient Rigor"));
    }

    // ========================================================================
    // RED TESTS - Category Scores
    // ========================================================================

    #[test]
    fn test_category_total_excludes_na() {
        let scores = PopperCategoryScores::default();
        // Default has ML as N/A, so max should be 95
        assert_eq!(scores.total_available(), 95.0);
    }

    #[test]
    fn test_category_total_includes_ml_when_applicable() {
        let mut scores = PopperCategoryScores::default();
        scores.ml_reproducibility.mark_applicable();
        assert_eq!(scores.total_available(), 100.0);
    }

    #[test]
    fn test_sub_score_accumulation() {
        let mut category = PopperCategoryScore::new("Test", 0.0, 25.0);

        category.add_sub_score(PopperSubScore::new("T1", "Test 1", 5.0, 8.0, "First test"));
        category.add_sub_score(PopperSubScore::new(
            "T2",
            "Test 2",
            7.0,
            10.0,
            "Second test",
        ));

        assert_eq!(category.earned, 12.0);
        assert_eq!(category.sub_scores.len(), 2);
    }

    // ========================================================================
    // RED TESTS - Findings
    // ========================================================================

    #[test]
    fn test_finding_creation() {
        let positive = PopperFinding::positive("Good test coverage");
        assert_eq!(positive.severity, FindingSeverity::Positive);

        let warning = PopperFinding::warning("Missing documentation", 2.0);
        assert_eq!(warning.severity, FindingSeverity::Warning);
        assert_eq!(warning.impact, 2.0);

        let critical = PopperFinding::critical("No tests", 10.0);
        assert_eq!(critical.severity, FindingSeverity::Critical);
        assert_eq!(critical.impact, 10.0);
    }

    // ========================================================================
    // RED TESTS - Recommendations
    // ========================================================================

    #[test]
    fn test_recommendation_with_command() {
        let rec = PopperRecommendation::new(
            "Testing",
            "Add mutation testing",
            RecommendationPriority::High,
            5.0,
        )
        .with_command("cargo mutants");

        assert_eq!(rec.command, Some("cargo mutants".to_string()));
    }
}
