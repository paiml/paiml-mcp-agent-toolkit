// McpPrompt trait implementations for all prompt types

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
