use super::*;
use std::collections::HashMap;

// Code analysis prompt
/// Code analysis prompt.
pub struct CodeAnalysisPrompt;

impl Default for CodeAnalysisPrompt {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeAnalysisPrompt {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self
    }
}

// Refactoring prompt
/// Refactoring prompt.
pub struct RefactoringPrompt;

impl Default for RefactoringPrompt {
    fn default() -> Self {
        Self::new()
    }
}

impl RefactoringPrompt {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self
    }
}

// Quality assessment prompt
/// Quality assessment prompt.
pub struct QualityAssessmentPrompt;

impl Default for QualityAssessmentPrompt {
    fn default() -> Self {
        Self::new()
    }
}

impl QualityAssessmentPrompt {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self
    }
}

// Repository health scoring prompt
/// Repo score prompt.
pub struct RepoScorePrompt;

impl Default for RepoScorePrompt {
    fn default() -> Self {
        Self::new()
    }
}

impl RepoScorePrompt {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
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
