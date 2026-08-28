// Tests for CodeAnalysis, Refactoring, and QualityAssessment prompts

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;

    // ============================================================
    // CodeAnalysisPrompt Tests
    // ============================================================

    #[test]
    fn test_code_analysis_prompt_new() {
        let prompt = CodeAnalysisPrompt::new();
        // Should create without panic
        let _ = prompt;
    }

    #[test]
    #[allow(clippy::default_constructed_unit_structs)]
    fn test_code_analysis_prompt_default() {
        // `X::default()` on a unit struct is what clippy's
        // default_constructed_unit_structs wants rewritten to `X` -- but the
        // `Default` impl IS the subject here, so rewriting it would delete
        // the test. Allowed locally, and given something that can actually
        // fail: `Default` and `new()` must produce the same prompt.
        let prompt = CodeAnalysisPrompt::default();
        assert_eq!(
            prompt.metadata().name,
            CodeAnalysisPrompt::new().metadata().name
        );
    }

    #[test]
    fn test_code_analysis_prompt_metadata() {
        let prompt = CodeAnalysisPrompt::new();
        let metadata = prompt.metadata();

        assert_eq!(metadata.name, "code_analysis");
        assert!(metadata.description.is_some());
        assert!(metadata
            .description
            .as_ref()
            .unwrap()
            .contains("quality issues"));

        // Check arguments
        let args = metadata.arguments.as_ref().unwrap();
        assert_eq!(args.len(), 2);

        // Check language argument
        let lang_arg = &args[0];
        assert_eq!(lang_arg.name, "language");
        assert!(lang_arg.description.is_some());
        assert_eq!(lang_arg.required, Some(true));

        // Check focus argument
        let focus_arg = &args[1];
        assert_eq!(focus_arg.name, "focus");
        assert!(focus_arg.description.is_some());
        assert_eq!(focus_arg.required, Some(false));
    }

    #[tokio::test]
    async fn test_code_analysis_prompt_get_with_language() {
        let prompt = CodeAnalysisPrompt::new();
        let mut args = HashMap::new();
        args.insert("language".to_string(), "rust".to_string());

        let messages = prompt.get(Some(args)).await.unwrap();

        assert_eq!(messages.len(), 2);

        // Check system message contains language
        let system_msg = &messages[0];
        assert_eq!(system_msg.role, "system");
        match &system_msg.content {
            PromptContent::Text(text) => {
                assert!(text.contains("rust"));
                assert!(text.contains("code quality expert"));
            }
            _ => panic!("Expected Text content"),
        }

        // Check user message
        let user_msg = &messages[1];
        assert_eq!(user_msg.role, "user");
        match &user_msg.content {
            PromptContent::Text(text) => {
                assert!(text.contains("analyze"));
            }
            _ => panic!("Expected Text content"),
        }
    }

    #[tokio::test]
    async fn test_code_analysis_prompt_get_without_arguments() {
        let prompt = CodeAnalysisPrompt::new();

        let messages = prompt.get(None).await.unwrap();

        assert_eq!(messages.len(), 2);

        // Should default to "unknown" language
        let system_msg = &messages[0];
        match &system_msg.content {
            PromptContent::Text(text) => {
                assert!(text.contains("unknown"));
            }
            _ => panic!("Expected Text content"),
        }
    }

    #[tokio::test]
    async fn test_code_analysis_prompt_get_with_empty_arguments() {
        let prompt = CodeAnalysisPrompt::new();
        let args = HashMap::new();

        let messages = prompt.get(Some(args)).await.unwrap();

        // Should default to "unknown" language
        let system_msg = &messages[0];
        match &system_msg.content {
            PromptContent::Text(text) => {
                assert!(text.contains("unknown"));
            }
            _ => panic!("Expected Text content"),
        }
    }

    #[tokio::test]
    async fn test_code_analysis_prompt_various_languages() {
        let prompt = CodeAnalysisPrompt::new();

        for lang in ["python", "javascript", "go", "java", "c++"] {
            let mut args = HashMap::new();
            args.insert("language".to_string(), lang.to_string());

            let messages = prompt.get(Some(args)).await.unwrap();
            let system_msg = &messages[0];

            match &system_msg.content {
                PromptContent::Text(text) => {
                    assert!(text.contains(lang), "Should contain language: {}", lang);
                }
                _ => panic!("Expected Text content"),
            }
        }
    }

    // ============================================================
    // RefactoringPrompt Tests
    // ============================================================

    #[test]
    fn test_refactoring_prompt_new() {
        let prompt = RefactoringPrompt::new();
        let _ = prompt;
    }

    #[test]
    #[allow(clippy::default_constructed_unit_structs)]
    fn test_refactoring_prompt_default() {
        // `X::default()` on a unit struct is what clippy's
        // default_constructed_unit_structs wants rewritten to `X` -- but the
        // `Default` impl IS the subject here, so rewriting it would delete
        // the test. Allowed locally, and given something that can actually
        // fail: `Default` and `new()` must produce the same prompt.
        let prompt = RefactoringPrompt::default();
        assert_eq!(
            prompt.metadata().name,
            RefactoringPrompt::new().metadata().name
        );
    }

    #[test]
    fn test_refactoring_prompt_metadata() {
        let prompt = RefactoringPrompt::new();
        let metadata = prompt.metadata();

        assert_eq!(metadata.name, "refactoring");
        assert!(metadata.description.is_some());
        assert!(metadata
            .description
            .as_ref()
            .unwrap()
            .contains("refactoring"));

        // Check arguments
        let args = metadata.arguments.as_ref().unwrap();
        assert_eq!(args.len(), 1);

        let pattern_arg = &args[0];
        assert_eq!(pattern_arg.name, "pattern");
        assert!(pattern_arg.description.is_some());
        assert_eq!(pattern_arg.required, Some(true));
    }

    #[tokio::test]
    async fn test_refactoring_prompt_get_with_pattern() {
        let prompt = RefactoringPrompt::new();
        let mut args = HashMap::new();
        args.insert("pattern".to_string(), "extract_method".to_string());

        let messages = prompt.get(Some(args)).await.unwrap();

        assert_eq!(messages.len(), 1);

        let system_msg = &messages[0];
        assert_eq!(system_msg.role, "system");
        match &system_msg.content {
            PromptContent::Text(text) => {
                assert!(text.contains("extract_method"));
                assert!(text.contains("refactoring expert"));
            }
            _ => panic!("Expected Text content"),
        }
    }

    #[tokio::test]
    async fn test_refactoring_prompt_get_without_arguments() {
        let prompt = RefactoringPrompt::new();

        let messages = prompt.get(None).await.unwrap();

        assert_eq!(messages.len(), 1);

        // Should default to "general" pattern
        let system_msg = &messages[0];
        match &system_msg.content {
            PromptContent::Text(text) => {
                assert!(text.contains("general"));
            }
            _ => panic!("Expected Text content"),
        }
    }

    #[tokio::test]
    async fn test_refactoring_prompt_various_patterns() {
        let prompt = RefactoringPrompt::new();

        for pattern in [
            "extract_method",
            "inline_variable",
            "rename",
            "move_field",
            "encapsulate",
        ] {
            let mut args = HashMap::new();
            args.insert("pattern".to_string(), pattern.to_string());

            let messages = prompt.get(Some(args)).await.unwrap();
            let system_msg = &messages[0];

            match &system_msg.content {
                PromptContent::Text(text) => {
                    assert!(
                        text.contains(pattern),
                        "Should contain pattern: {}",
                        pattern
                    );
                }
                _ => panic!("Expected Text content"),
            }
        }
    }

    // ============================================================
    // QualityAssessmentPrompt Tests
    // ============================================================

    #[test]
    fn test_quality_assessment_prompt_new() {
        let prompt = QualityAssessmentPrompt::new();
        let _ = prompt;
    }

    #[test]
    #[allow(clippy::default_constructed_unit_structs)]
    fn test_quality_assessment_prompt_default() {
        // `X::default()` on a unit struct is what clippy's
        // default_constructed_unit_structs wants rewritten to `X` -- but the
        // `Default` impl IS the subject here, so rewriting it would delete
        // the test. Allowed locally, and given something that can actually
        // fail: `Default` and `new()` must produce the same prompt.
        let prompt = QualityAssessmentPrompt::default();
        assert_eq!(
            prompt.metadata().name,
            QualityAssessmentPrompt::new().metadata().name
        );
    }

    #[test]
    fn test_quality_assessment_prompt_metadata() {
        let prompt = QualityAssessmentPrompt::new();
        let metadata = prompt.metadata();

        assert_eq!(metadata.name, "quality_assessment");
        assert!(metadata.description.is_some());
        assert!(metadata.description.as_ref().unwrap().contains("quality"));

        // No arguments for this prompt
        assert!(metadata.arguments.is_none());
    }

    #[tokio::test]
    async fn test_quality_assessment_prompt_get() {
        let prompt = QualityAssessmentPrompt::new();

        let messages = prompt.get(None).await.unwrap();

        assert_eq!(messages.len(), 1);

        let system_msg = &messages[0];
        assert_eq!(system_msg.role, "system");
        match &system_msg.content {
            PromptContent::Text(text) => {
                assert!(text.contains("quality assessor"));
                assert!(text.contains("best practices"));
            }
            _ => panic!("Expected Text content"),
        }
    }

    #[tokio::test]
    async fn test_quality_assessment_prompt_ignores_arguments() {
        let prompt = QualityAssessmentPrompt::new();
        let mut args = HashMap::new();
        args.insert("ignored".to_string(), "value".to_string());

        // Should not error even with arguments provided
        let messages = prompt.get(Some(args)).await.unwrap();
        assert_eq!(messages.len(), 1);
    }
}
