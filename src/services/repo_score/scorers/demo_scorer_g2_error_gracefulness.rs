// DemoScorer G2: Error Gracefulness (3 points)
// Checks for proper error handling in demo code (no raw panics/unwraps in user-facing code)

impl DemoScorer {
    /// Score Error Gracefulness (G2: 3 points)
    async fn score_error_gracefulness(
        &self,
        repo_path: &Path,
        archetype: RepoArchetype,
    ) -> Result<SubcategoryScore> {
        debug_assert!(repo_path.exists(), "repo_path must exist: {}", repo_path.display());
        // Handle N/A state for cookbooks
        if archetype.g2_max_score().is_none() {
            return Ok(SubcategoryScore {
                id: "G2".to_string(),
                name: "Error Gracefulness (N/A for Cookbook)".to_string(),
                score: 0.0,
                max_score: 0.0,
                findings: vec![Finding {
                    severity: Severity::Info,
                    category: "Demo Quality".to_string(),
                    message: "G2 scoring not applicable for documentation-heavy repositories"
                        .to_string(),
                    location: None,
                    impact_points: 0.0,
                }],
            });
        }

        let max_score = archetype.g2_max_score().unwrap_or(3.0);
        let demo_files = self.find_demo_files(repo_path).await;

        if demo_files.is_empty() {
            return self.g2_no_demo_files(repo_path, max_score).await;
        }

        self.g2_analyze_demo_files(&demo_files, max_score).await
    }

    async fn g2_no_demo_files(
        &self,
        repo_path: &Path,
        max_score: f64,
    ) -> Result<SubcategoryScore> {
        debug_assert!(repo_path.exists(), "repo_path must exist: {}", repo_path.display());
        let has_error_section = check_readme_error_section(repo_path).await;
        let partial_score: f64 = if has_error_section { 2.0 } else { 1.5 };

        Ok(SubcategoryScore {
            id: "G2".to_string(),
            name: "Error Gracefulness".to_string(),
            score: partial_score.min(max_score),
            max_score,
            findings: vec![Finding {
                severity: Severity::Info,
                category: "Demo Quality".to_string(),
                message: if has_error_section {
                    "No demo files to analyze, but error handling documentation found".to_string()
                } else {
                    "No demo files found to analyze for error handling".to_string()
                },
                location: None,
                impact_points: 0.0,
            }],
        })
    }
}

async fn check_readme_error_section(repo_path: &Path) -> bool {
    debug_assert!(repo_path.exists(), "repo_path must exist: {}", repo_path.display());
    let readme_path = repo_path.join("README.md");
    if !readme_path.exists() {
        return false;
    }
    let Ok(content) = tokio::fs::read_to_string(&readme_path).await else {
        return false;
    };
    let error_section_patterns = [
        r"(?i)##?\s*error\s*handling",
        r"(?i)##?\s*troubleshoot",
        r"(?i)##?\s*common\s*(errors|issues|problems)",
    ];
    error_section_patterns.iter().any(|p| {
        regex::Regex::new(p)
            .map(|re| re.is_match(&content))
            .unwrap_or(false)
    })
}
