use super::*;
use std::collections::HashMap;

// Code analysis prompt
pub struct CodeAnalysisPrompt;

impl Default for CodeAnalysisPrompt {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeAnalysisPrompt {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl McpPrompt for CodeAnalysisPrompt {
    fn metadata(&self) -> PromptMetadata {
        PromptMetadata {
            name: "code_analysis".to_string(),
            description: Some("Analyze code for quality issues".to_string()),
            arguments: Some(vec![
                PromptArgument {
                    name: "language".to_string(),
                    description: Some("Programming language".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "focus".to_string(),
                    description: Some("Analysis focus area".to_string()),
                    required: Some(false),
                },
            ]),
        }
    }

    async fn get(
        &self,
        arguments: Option<HashMap<String, String>>,
    ) -> Result<Vec<PromptMessage>, McpError> {
        let lang = arguments
            .as_ref()
            .and_then(|args| args.get("language"))
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        Ok(vec![
            PromptMessage {
                role: "system".to_string(),
                content: PromptContent::Text(
                    format!("You are a code quality expert specializing in {}. Analyze the provided code for quality issues, complexity, and potential improvements.", lang)
                ),
            },
            PromptMessage {
                role: "user".to_string(),
                content: PromptContent::Text(
                    "Please analyze the following code and provide detailed feedback.".to_string()
                ),
            },
        ])
    }
}

// Refactoring prompt
pub struct RefactoringPrompt;

impl Default for RefactoringPrompt {
    fn default() -> Self {
        Self::new()
    }
}

impl RefactoringPrompt {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl McpPrompt for RefactoringPrompt {
    fn metadata(&self) -> PromptMetadata {
        PromptMetadata {
            name: "refactoring".to_string(),
            description: Some("Guide code refactoring".to_string()),
            arguments: Some(vec![PromptArgument {
                name: "pattern".to_string(),
                description: Some("Refactoring pattern to apply".to_string()),
                required: Some(true),
            }]),
        }
    }

    async fn get(
        &self,
        arguments: Option<HashMap<String, String>>,
    ) -> Result<Vec<PromptMessage>, McpError> {
        let pattern = arguments
            .as_ref()
            .and_then(|args| args.get("pattern"))
            .cloned()
            .unwrap_or_else(|| "general".to_string());

        Ok(vec![PromptMessage {
            role: "system".to_string(),
            content: PromptContent::Text(format!(
                "You are a refactoring expert. Apply the {} pattern to improve code quality.",
                pattern
            )),
        }])
    }
}

// Quality assessment prompt
pub struct QualityAssessmentPrompt;

impl Default for QualityAssessmentPrompt {
    fn default() -> Self {
        Self::new()
    }
}

impl QualityAssessmentPrompt {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl McpPrompt for QualityAssessmentPrompt {
    fn metadata(&self) -> PromptMetadata {
        PromptMetadata {
            name: "quality_assessment".to_string(),
            description: Some("Assess overall code quality".to_string()),
            arguments: None,
        }
    }

    async fn get(
        &self,
        _arguments: Option<HashMap<String, String>>,
    ) -> Result<Vec<PromptMessage>, McpError> {
        Ok(vec![
            PromptMessage {
                role: "system".to_string(),
                content: PromptContent::Text(
                    "You are a code quality assessor. Evaluate code against industry best practices and provide a comprehensive quality report.".to_string()
                ),
            },
        ])
    }
}

// Repository health scoring prompt
pub struct RepoScorePrompt;

impl Default for RepoScorePrompt {
    fn default() -> Self {
        Self::new()
    }
}

impl RepoScorePrompt {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl McpPrompt for RepoScorePrompt {
    fn metadata(&self) -> PromptMetadata {
        PromptMetadata {
            name: "repo_score".to_string(),
            description: Some(
                "Assess repository health with quantitative scoring (0-110 scale)".to_string(),
            ),
            arguments: Some(vec![
                PromptArgument {
                    name: "repository_path".to_string(),
                    description: Some("Path to repository to score".to_string()),
                    required: Some(false),
                },
                PromptArgument {
                    name: "output_format".to_string(),
                    description: Some("Output format: text, json, junit".to_string()),
                    required: Some(false),
                },
            ]),
        }
    }

    async fn get(
        &self,
        arguments: Option<HashMap<String, String>>,
    ) -> Result<Vec<PromptMessage>, McpError> {
        let repo_path = arguments
            .as_ref()
            .and_then(|args| args.get("repository_path"))
            .cloned()
            .unwrap_or_else(|| ".".to_string());

        let format = arguments
            .as_ref()
            .and_then(|args| args.get("output_format"))
            .cloned()
            .unwrap_or_else(|| "text".to_string());

        Ok(vec![
            PromptMessage {
                role: "system".to_string(),
                content: PromptContent::Text(format!(
                    r#"You are a repository health assessment expert using PMAT's repo-score system.

**Repository Scoring System (0-110 scale):**
- **100 base points** across 6 categories (A-F)
- **10 bonus points** for advanced quality practices

**Categories (100 base points):**

1. **Documentation (A): 20 points**
   - A1: README Accuracy (10 pts) - File exists, not empty, valid markdown
   - A2: Comprehensiveness (10 pts) - Overview, Install, Usage, License, Contributing

2. **Pre-commit Hooks (B): 20 points**
   - B1: Hook Present (10 pts) - .git/hooks/pre-commit exists & executable
   - B2: Performance (10 pts) - Fast execution, quality checks

3. **Repository Hygiene (C): 10 points**
   - C1: No Cruft Files (5 pts) - No temp files, build artifacts
   - C2: No Team Files (5 pts) - No .idea/, .vscode/

4. **Build & Test (D): 25 points**
   - D1: Makefile Present (5 pts) - Valid Makefile exists
   - D2: Required Targets (15 pts) - test-fast, test, lint, coverage
   - D3: Performance (5 pts) - Optimized fast targets

5. **CI/CD (E): 20 points**
   - E1: Workflows Present (10 pts) - .github/workflows/ with YAML files
   - E2: Configured (10 pts) - Valid structure, testing, linting

6. **PMAT Compliance (F): 5 points**
   - F1: Config Present (2.5 pts) - .pmat-gates.toml exists & valid
   - F2: No Violations (2.5 pts) - Quality gates defined

**Bonus Features (+10 points):**
- Property-based testing (proptest) → +3 points
- Fuzzing (cargo-fuzz) → +2 points
- Mutation testing (cargo-mutants) → +2 points
- Living documentation (mdBook) → +3 points

**Grading Scale:**
- A+ (95-110): Exceptional (includes bonus)
- A (90-94): Excellent
- A- (85-89): PMAT standard (minimum for production)
- B+ (80-84): Good
- B (70-79): Acceptable
- C (60-69): Needs improvement
- D (50-59): Poor
- F (0-49): Failing

**Score Status per Category:**
- ✅ Pass: ≥90% of max score
- ⚠️  Warning: 70-89% of max score
- ❌ Fail: <70% of max score

**Usage:**
```bash
# Score repository
pmat repo-score {}

# Output formats
pmat repo-score {} --format {}
```

**Key Features:**
- Graceful degradation (missing components score 0, not error)
- Partial credit (e.g., non-executable hook: 5/10 points)
- Prioritized recommendations (Critical → High → Medium → Low)
- Evidence-based findings with locations
- Git context extraction (branch, commit)

**Recommendations System:**
- 🔴 CRITICAL: Blocking issues (README, Makefile)
- 🟠 HIGH: Important quality (Pre-commit, CI/CD)
- 🟡 MEDIUM: Nice-to-have (Hygiene, PMAT config)
- 🟢 LOW: Enhancements (Bonus features)

Provide comprehensive repository health assessment with actionable recommendations."#,
                    repo_path, repo_path, format
                )),
            },
            PromptMessage {
                role: "user".to_string(),
                content: PromptContent::Text(format!(
                    "Please assess the repository health at: {}",
                    repo_path
                )),
            },
        ])
    }
}

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
    fn test_code_analysis_prompt_default() {
        let prompt = CodeAnalysisPrompt::default();
        // Default should work same as new
        let _ = prompt;
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
    fn test_refactoring_prompt_default() {
        let prompt = RefactoringPrompt::default();
        let _ = prompt;
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
                    assert!(text.contains(pattern), "Should contain pattern: {}", pattern);
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
    fn test_quality_assessment_prompt_default() {
        let prompt = QualityAssessmentPrompt::default();
        let _ = prompt;
    }

    #[test]
    fn test_quality_assessment_prompt_metadata() {
        let prompt = QualityAssessmentPrompt::new();
        let metadata = prompt.metadata();

        assert_eq!(metadata.name, "quality_assessment");
        assert!(metadata.description.is_some());
        assert!(metadata
            .description
            .as_ref()
            .unwrap()
            .contains("quality"));

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

    // ============================================================
    // RepoScorePrompt Tests
    // ============================================================

    #[test]
    fn test_repo_score_prompt_new() {
        let prompt = RepoScorePrompt::new();
        let _ = prompt;
    }

    #[test]
    fn test_repo_score_prompt_default() {
        let prompt = RepoScorePrompt::default();
        let _ = prompt;
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
        assert!(metadata
            .description
            .as_ref()
            .unwrap()
            .contains("0-110"));

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
                assert!(text.contains("0-110"));
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
                // Check all scoring categories are mentioned
                assert!(text.contains("Documentation"));
                assert!(text.contains("Pre-commit Hooks"));
                assert!(text.contains("Repository Hygiene"));
                assert!(text.contains("Build & Test"));
                assert!(text.contains("CI/CD"));
                assert!(text.contains("PMAT Compliance"));

                // Check grading scale
                assert!(text.contains("A+"));
                assert!(text.contains("Exceptional"));
                assert!(text.contains("Failing"));

                // Check bonus features
                assert!(text.contains("Property-based testing"));
                assert!(text.contains("Fuzzing"));
                assert!(text.contains("Mutation testing"));
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

        assert_eq!(names.len(), unique_names.len(), "All prompt names should be unique");
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
