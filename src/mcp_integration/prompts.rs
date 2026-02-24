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

// McpPrompt trait implementations
include!("prompts_impls.rs");

// Tests: CodeAnalysis, Refactoring, QualityAssessment prompts
include!("prompts_tests.rs");

// Tests: RepoScore prompt and edge cases
include!("prompts_tests_repo_score.rs");
