use super::*;
use std::collections::HashMap;

// Code analysis prompt
pub struct CodeAnalysisPrompt;

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
