// Tests for RepoScorePrompt and edge cases / integration

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod repo_score_tests {
    use super::*;

    // ============================================================
    // RepoScorePrompt Tests
    // ============================================================

    #[test]
    fn test_repo_score_prompt_new() {
        let prompt = RepoScorePrompt::new();
        let _ = prompt;
    }

    #[test]
    #[allow(clippy::default_constructed_unit_structs)]
    fn test_repo_score_prompt_default() {
        // `X::default()` on a unit struct is what clippy's
        // default_constructed_unit_structs wants rewritten to `X` -- but the
        // `Default` impl IS the subject here, so rewriting it would delete
        // the test. Allowed locally, and given something that can actually
        // fail: `Default` and `new()` must produce the same prompt.
        let prompt = RepoScorePrompt::default();
        assert_eq!(
            prompt.metadata().name,
            RepoScorePrompt::new().metadata().name
        );
    }

    #[test]
    fn test_repo_score_prompt_metadata() {
        let prompt = RepoScorePrompt::new();
        let metadata = prompt.metadata();

        assert_eq!(metadata.name, "repo_score");
        assert!(metadata.description.is_some());
        assert!(metadata
            .description
            .as_ref()
            .unwrap()
            .contains("repository health"));
        // GH #685: the scale is 0-100. The six categories sum to exactly 100 and
        // `BonusScores` is never added to `total_score`, so "0-110" advertised a
        // range no run can reach. Derived from the scorers rather than hardcoded,
        // so a category max change fails here instead of drifting silently.
        let max_points: f64 = {
            let c = crate::services::repo_score::models::CategoryScores::default();
            c.documentation.max_score
                + c.precommit_hooks.max_score
                + c.repository_hygiene.max_score
                + c.build_test_automation.max_score
                + c.continuous_integration.max_score
                + c.pmat_compliance.max_score
        };
        let scale = format!("0-{} scale", max_points as u32);
        let description = metadata.description.as_ref().unwrap();
        assert!(
            description.contains(&scale),
            "description {description:?} must advertise the {scale} repo-score emits"
        );
        assert!(!description.contains("0-110"));

        // Check arguments
        let args = metadata.arguments.as_ref().unwrap();
        assert_eq!(args.len(), 2);

        // repository_path argument
        let path_arg = &args[0];
        assert_eq!(path_arg.name, "repository_path");
        assert!(path_arg.description.is_some());
        assert_eq!(path_arg.required, Some(false));

        // output_format argument
        let format_arg = &args[1];
        assert_eq!(format_arg.name, "output_format");
        assert!(format_arg.description.is_some());
        assert_eq!(format_arg.required, Some(false));
    }

    #[tokio::test]
    async fn test_repo_score_prompt_get_with_all_arguments() {
        let prompt = RepoScorePrompt::new();
        let mut args = HashMap::new();
        args.insert("repository_path".to_string(), "/path/to/repo".to_string());
        args.insert("output_format".to_string(), "json".to_string());

        let messages = prompt.get(Some(args)).await.unwrap();

        assert_eq!(messages.len(), 2);

        // Check system message
        let system_msg = &messages[0];
        assert_eq!(system_msg.role, "system");
        match &system_msg.content {
            PromptContent::Text(text) => {
                assert!(text.contains("/path/to/repo"));
                assert!(text.contains("json"));
                assert!(text.contains("repository health"));
                // GH #685: 0-100, the range repo-score actually emits.
                assert!(text.contains("(0-100 scale)"));
                assert!(!text.contains("0-110"));
            }
            _ => panic!("Expected Text content"),
        }

        // Check user message
        let user_msg = &messages[1];
        assert_eq!(user_msg.role, "user");
        match &user_msg.content {
            PromptContent::Text(text) => {
                assert!(text.contains("/path/to/repo"));
            }
            _ => panic!("Expected Text content"),
        }
    }

    #[tokio::test]
    async fn test_repo_score_prompt_get_without_arguments() {
        let prompt = RepoScorePrompt::new();

        let messages = prompt.get(None).await.unwrap();

        assert_eq!(messages.len(), 2);

        // Should default to "." path and "text" format
        let system_msg = &messages[0];
        match &system_msg.content {
            PromptContent::Text(text) => {
                // Default path is "."
                assert!(text.contains("pmat repo-score ."));
                // Default format is "text"
                assert!(text.contains("--format text"));
            }
            _ => panic!("Expected Text content"),
        }

        let user_msg = &messages[1];
        match &user_msg.content {
            PromptContent::Text(text) => {
                assert!(text.contains("."));
            }
            _ => panic!("Expected Text content"),
        }
    }

    #[tokio::test]
    async fn test_repo_score_prompt_get_with_partial_arguments() {
        let prompt = RepoScorePrompt::new();
        let mut args = HashMap::new();
        args.insert("repository_path".to_string(), "/custom/path".to_string());
        // output_format not provided, should default to "text"

        let messages = prompt.get(Some(args)).await.unwrap();

        let system_msg = &messages[0];
        match &system_msg.content {
            PromptContent::Text(text) => {
                assert!(text.contains("/custom/path"));
                assert!(text.contains("--format text"));
            }
            _ => panic!("Expected Text content"),
        }
    }

    #[tokio::test]
    async fn test_repo_score_prompt_various_formats() {
        let prompt = RepoScorePrompt::new();

        for format in ["text", "json", "junit"] {
            let mut args = HashMap::new();
            args.insert("output_format".to_string(), format.to_string());

            let messages = prompt.get(Some(args)).await.unwrap();
            let system_msg = &messages[0];

            match &system_msg.content {
                PromptContent::Text(text) => {
                    assert!(
                        text.contains(&format!("--format {}", format)),
                        "Should contain format: {}",
                        format
                    );
                }
                _ => panic!("Expected Text content"),
            }
        }
    }

    #[tokio::test]
    async fn test_repo_score_prompt_content_includes_scoring_categories() {
        let prompt = RepoScorePrompt::new();
        let messages = prompt.get(None).await.unwrap();

        let system_msg = &messages[0];
        match &system_msg.content {
            PromptContent::Text(text) => {
                // Every category is named, and named with the point budget its
                // scorer actually awards. The prompt used to claim 20 for
                // Documentation and 10 for Repository Hygiene when the scorers
                // award 15 and 15 (GH #685: do not advertise unmeasured points).
                let c = crate::services::repo_score::models::CategoryScores::default();
                for (name, max) in [
                    ("Documentation (A)", c.documentation.max_score),
                    ("Pre-commit Hooks (B)", c.precommit_hooks.max_score),
                    ("Repository Hygiene (C)", c.repository_hygiene.max_score),
                    ("Build & Test (D)", c.build_test_automation.max_score),
                    ("CI/CD (E)", c.continuous_integration.max_score),
                    ("PMAT Compliance (F)", c.pmat_compliance.max_score),
                ] {
                    let claim = format!("{name}: {} points", max as u32);
                    assert!(
                        text.contains(&claim),
                        "prompt must describe the category as {claim:?}"
                    );
                }

                // Check grading scale
                assert!(text.contains("A+"));
                assert!(text.contains("Exceptional"));
                assert!(text.contains("Failing"));

                // Bonus features are named, but explicitly as NOT scored: the
                // model defines BonusScores and nothing ever adds it to
                // total_score, so the prompt must not sell them as points.
                assert!(text.contains("Property-based testing"));
                assert!(text.contains("Fuzzing"));
                assert!(text.contains("Mutation testing"));
                assert!(text.contains("**Not scored:**"));
                // The two stale claims of a 110-point ceiling.
                assert!(!text.contains("0-110"));
                assert!(!text.contains("95-110"));
            }
            _ => panic!("Expected Text content"),
        }
    }

    // ============================================================
    // Edge Cases and Integration
    // ============================================================

    #[tokio::test]
    async fn test_all_prompts_return_valid_messages() {
        // CodeAnalysisPrompt
        let cap = CodeAnalysisPrompt::new();
        let cap_msgs = cap.get(None).await.unwrap();
        assert!(!cap_msgs.is_empty());
        for msg in &cap_msgs {
            assert!(!msg.role.is_empty());
        }

        // RefactoringPrompt
        let rp = RefactoringPrompt::new();
        let rp_msgs = rp.get(None).await.unwrap();
        assert!(!rp_msgs.is_empty());
        for msg in &rp_msgs {
            assert!(!msg.role.is_empty());
        }

        // QualityAssessmentPrompt
        let qap = QualityAssessmentPrompt::new();
        let qap_msgs = qap.get(None).await.unwrap();
        assert!(!qap_msgs.is_empty());
        for msg in &qap_msgs {
            assert!(!msg.role.is_empty());
        }

        // RepoScorePrompt
        let rsp = RepoScorePrompt::new();
        let rsp_msgs = rsp.get(None).await.unwrap();
        assert!(!rsp_msgs.is_empty());
        for msg in &rsp_msgs {
            assert!(!msg.role.is_empty());
        }
    }

    #[test]
    fn test_all_prompts_have_unique_names() {
        let names = vec![
            CodeAnalysisPrompt::new().metadata().name,
            RefactoringPrompt::new().metadata().name,
            QualityAssessmentPrompt::new().metadata().name,
            RepoScorePrompt::new().metadata().name,
        ];

        let mut unique_names = names.clone();
        unique_names.sort();
        unique_names.dedup();

        assert_eq!(
            names.len(),
            unique_names.len(),
            "All prompt names should be unique"
        );
    }

    #[tokio::test]
    async fn test_prompts_handle_special_characters_in_arguments() {
        let prompt = CodeAnalysisPrompt::new();
        let mut args = HashMap::new();
        args.insert("language".to_string(), "c++/c#".to_string());

        // Should not panic
        let messages = prompt.get(Some(args)).await.unwrap();
        assert!(!messages.is_empty());
    }

    #[tokio::test]
    async fn test_prompts_handle_empty_string_arguments() {
        let prompt = RefactoringPrompt::new();
        let mut args = HashMap::new();
        args.insert("pattern".to_string(), "".to_string());

        let messages = prompt.get(Some(args)).await.unwrap();
        // Should use empty string, not default
        let system_msg = &messages[0];
        match &system_msg.content {
            PromptContent::Text(text) => {
                assert!(text.contains("Apply the  pattern")); // Empty pattern
            }
            _ => panic!("Expected Text content"),
        }
    }

    #[tokio::test]
    async fn test_prompts_handle_unicode_arguments() {
        let prompt = CodeAnalysisPrompt::new();
        let mut args = HashMap::new();
        args.insert("language".to_string(), "日本語".to_string());

        let messages = prompt.get(Some(args)).await.unwrap();
        let system_msg = &messages[0];
        match &system_msg.content {
            PromptContent::Text(text) => {
                assert!(text.contains("日本語"));
            }
            _ => panic!("Expected Text content"),
        }
    }
}
