// Tests for Popper score handlers (fixtures and handler tests)
// Included by popper_score_handlers.rs — do NOT add `use` imports here.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::popper_score::{
        AnalysisStatus, PopperAnalysis, PopperCategoryScore, PopperCategoryScores, PopperGrade,
        PopperMetadata, PopperRecommendation, PopperSubScore, RecommendationPriority,
    };
    use tempfile::TempDir;

    /// Create a test PopperScore with gateway passed
    fn create_test_score_passed() -> PopperScore {
        let mut categories = PopperCategoryScores {
            falsifiability: PopperCategoryScore::new("Falsifiability & Testability", 20.0, 25.0),
            ..Default::default()
        };
        categories.falsifiability.add_sub_score(PopperSubScore::new(
            "A1",
            "Test Coverage",
            8.0,
            10.0,
            "Unit test coverage",
        ));
        categories.falsifiability.add_sub_score(PopperSubScore::new(
            "A2",
            "Claims",
            12.0,
            15.0,
            "Testable claims",
        ));
        categories.reproducibility =
            PopperCategoryScore::new("Reproducibility Infrastructure", 18.0, 25.0);
        categories.transparency = PopperCategoryScore::new("Transparency & Openness", 15.0, 20.0);
        categories.statistical_rigor = PopperCategoryScore::new("Statistical Rigor", 10.0, 15.0);
        categories.historical_integrity =
            PopperCategoryScore::new("Historical Integrity", 7.0, 10.0);
        // ML stays N/A

        let recommendations = vec![
            PopperRecommendation::new(
                "Testing",
                "Add mutation testing",
                RecommendationPriority::High,
                5.0,
            )
            .with_command("cargo mutants"),
            PopperRecommendation::new(
                "Documentation",
                "Improve README",
                RecommendationPriority::Medium,
                2.0,
            ),
            PopperRecommendation::new(
                "Infrastructure",
                "Add CI pipeline",
                RecommendationPriority::Critical,
                8.0,
            ),
            PopperRecommendation::new(
                "Testing",
                "Add benchmarks",
                RecommendationPriority::Low,
                1.0,
            ),
        ];

        PopperScore {
            raw_score: 70.0,
            max_available: 95.0,
            normalized_score: 73.7,
            grade: PopperGrade::B,
            gateway_passed: true,
            categories,
            recommendations,
            metadata: PopperMetadata::new("test-project".to_string()),
            analysis: PopperAnalysis {
                falsifiability_status: AnalysisStatus::Pass,
                reproducibility_status: AnalysisStatus::Partial,
                scrutiny_status: AnalysisStatus::Partial,
                methodology_status: AnalysisStatus::Pass,
                validation_status: AnalysisStatus::Fail,
                verdict: "Good scientific practices with room for improvement.".to_string(),
            },
        }
    }

    /// Create a test PopperScore with gateway failed
    fn create_test_score_failed() -> PopperScore {
        let mut categories = PopperCategoryScores {
            falsifiability: PopperCategoryScore::new("Falsifiability & Testability", 10.0, 25.0),
            ..Default::default()
        };
        categories.reproducibility =
            PopperCategoryScore::new("Reproducibility Infrastructure", 5.0, 25.0);
        categories.transparency = PopperCategoryScore::new("Transparency & Openness", 5.0, 20.0);
        categories.statistical_rigor = PopperCategoryScore::new("Statistical Rigor", 3.0, 15.0);
        categories.historical_integrity =
            PopperCategoryScore::new("Historical Integrity", 2.0, 10.0);

        PopperScore {
            raw_score: 0.0,
            max_available: 95.0,
            normalized_score: 0.0,
            grade: PopperGrade::InsufficientFalsifiability,
            gateway_passed: false,
            categories,
            recommendations: vec![PopperRecommendation::new(
                "Falsifiability",
                "Add testable claims",
                RecommendationPriority::Critical,
                25.0,
            )],
            metadata: PopperMetadata::new("failing-project".to_string()),
            analysis: PopperAnalysis {
                falsifiability_status: AnalysisStatus::Fail,
                reproducibility_status: AnalysisStatus::Fail,
                scrutiny_status: AnalysisStatus::Fail,
                methodology_status: AnalysisStatus::Fail,
                validation_status: AnalysisStatus::Fail,
                verdict: "Gateway failed - insufficient falsifiability.".to_string(),
            },
        }
    }

    // ========================================================================
    // Handler Tests
    // ========================================================================

    #[tokio::test]
    async fn test_handler_invalid_path() {
        let result = handle_popper_score(
            Path::new("/nonexistent/path"),
            &RepoScoreOutputFormat::Text,
            false,
            false,
            None,
        )
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_handler_not_a_directory() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("file.txt");
        std::fs::write(&file_path, "not a directory").unwrap();

        let result =
            handle_popper_score(&file_path, &RepoScoreOutputFormat::Text, false, false, None).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a directory"));
    }

    #[tokio::test]
    async fn test_handler_empty_project() {
        let temp = TempDir::new().unwrap();

        let result = handle_popper_score(
            temp.path(),
            &RepoScoreOutputFormat::Text,
            false,
            false,
            None,
        )
        .await;

        // Should succeed but show gateway failure
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handler_json_output() {
        let temp = TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("README.md"),
            "# Test\n\nSuccess criteria: Tests pass.",
        )
        .unwrap();

        let result = handle_popper_score(
            temp.path(),
            &RepoScoreOutputFormat::Json,
            false,
            false,
            None,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handler_markdown_output() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("README.md"), "# Test Project").unwrap();

        let result = handle_popper_score(
            temp.path(),
            &RepoScoreOutputFormat::Markdown,
            false,
            false,
            None,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handler_yaml_output() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("README.md"), "# Test Project").unwrap();

        let result = handle_popper_score(
            temp.path(),
            &RepoScoreOutputFormat::Yaml,
            false,
            false,
            None,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handler_verbose_output() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("README.md"), "# Test").unwrap();

        let result = handle_popper_score(
            temp.path(),
            &RepoScoreOutputFormat::Text,
            true, // verbose
            false,
            None,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handler_failures_only_output() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("README.md"), "# Test").unwrap();

        let result = handle_popper_score(
            temp.path(),
            &RepoScoreOutputFormat::Text,
            false,
            true, // failures_only
            None,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handler_output_to_file() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("README.md"), "# Test").unwrap();
        let output_path = temp.path().join("score.txt");

        let result = handle_popper_score(
            temp.path(),
            &RepoScoreOutputFormat::Text,
            false,
            false,
            Some(&output_path),
        )
        .await;

        assert!(result.is_ok());
        assert!(output_path.exists());
    }

    // Include format-specific tests
    include!("popper_score_tests_format.rs");

    /// PMAT-688: on the 3.39.0 tree `--failures-only` was read in two places
    /// only (`verbose && !failures_only` and `failures_only && gateway_passed`),
    /// so the swept `popper-score --failures-only` reproduced the baseline byte
    /// for byte. The flag is the same display filter its siblings implement
    /// (`repo-score`, `infra-score`, `rust-project-score`): a row at or above
    /// the pass line is not shown, everything else is untouched, and the
    /// recommendations — the actionable failures — stay even when the gateway
    /// passed.
    #[test]
    fn failures_only_text_keeps_only_failing_checks() {
        // Falsifiability 20/25 sits on the 80% pass line; the other four are below it.
        let score = create_test_score_passed();
        let all = format_text(&score, false, false);
        let failing = format_text(&score, false, true);
        assert!(all.contains("Falsifiability & Testability"), "{all}");
        assert!(
            !failing.contains("Falsifiability & Testability"),
            "a category at the pass line is not a failing check:\n{failing}"
        );
        for kept in ["Reproducibility Infrastructure", "Summary", "Verdict", "Recommendations"] {
            assert!(failing.contains(kept), "{kept} must survive the filter:\n{failing}");
        }
    }

    #[test]
    fn failures_only_verbose_keeps_only_failing_sub_scores() {
        // A1 8/10 and A2 12/15 both sit on the pass line inside a passing category;
        // B (18/25) has no sub-scores, so add one that fails. `add_sub_score`
        // adds the sub-score's points to the category, so it must earn 0 for
        // B to stay below the line.
        let mut score = create_test_score_passed();
        score
            .categories
            .reproducibility
            .add_sub_score(PopperSubScore::new("B1", "Lockfile", 0.0, 10.0, "no lockfile"));
        let all = format_text(&score, true, false);
        let failing = format_text(&score, true, true);
        assert!(all.contains("A1") && all.contains("B1"), "{all}");
        assert!(!failing.contains("A1"), "a passing sub-score is not a failure:\n{failing}");
        assert!(failing.contains("B1"), "a failing sub-score stays:\n{failing}");
    }

    #[test]
    fn failures_only_markdown_keeps_only_failing_checks() {
        let score = create_test_score_passed();
        let all = format_markdown(&score, false, false);
        let failing = format_markdown(&score, false, true);
        assert!(all.contains("Falsifiability & Testability"), "{all}");
        assert!(!failing.contains("Falsifiability & Testability"), "{failing}");
        assert!(failing.contains("Reproducibility Infrastructure"), "{failing}");
        assert!(failing.contains("Verdict"), "{failing}");
    }
}
