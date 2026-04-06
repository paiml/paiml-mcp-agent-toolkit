// DemoScorer G1: Time-to-Interaction (3 points)
// Checks for quick-start guides, simple examples, and fast demo execution

fn check_readme_patterns(content: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|pattern| {
        regex::Regex::new(pattern)
            .map(|re| re.is_match(content))
            .unwrap_or(false)
    })
}

impl DemoScorer {
    /// Score Time-to-Interaction (G1: 3 points)
    async fn score_time_to_interaction(&self, repo_path: &Path) -> Result<SubcategoryScore> {
        debug_assert!(repo_path.exists(), "repo_path must exist: {}", repo_path.display());
        let mut score: f64 = 0.0;
        let mut findings = vec![];

        // Check for examples directory
        let examples_dirs = ["examples", "demos", "demo", "samples"];
        let mut has_examples = false;
        for dir in examples_dirs {
            if repo_path.join(dir).exists() {
                has_examples = true;
                score += 1.0;
                findings.push(Finding {
                    severity: Severity::Success,
                    category: "Demo Quality".to_string(),
                    message: format!("{} directory found", dir),
                    location: Some(dir.to_string()),
                    impact_points: 1.0,
                });
                break;
            }
        }

        if !has_examples {
            findings.push(Finding {
                severity: Severity::Warning,
                category: "Demo Quality".to_string(),
                message: "No examples/ or demos/ directory found".to_string(),
                location: None,
                impact_points: 0.0,
            });
        }

        // Check for quick-start in README
        let readme_path = repo_path.join("README.md");
        if readme_path.exists() {
            if let Ok(content) = tokio::fs::read_to_string(&readme_path).await {
                let quick_start_patterns = [
                    r"(?i)##?\s*quick\s*start",
                    r"(?i)##?\s*getting\s*started",
                    r"(?i)##?\s*try\s*it\s*(out|now)",
                    r"(?i)##?\s*5[\s-]minute",
                    r"(?i)##?\s*tldr",
                ];

                if check_readme_patterns(&content, &quick_start_patterns) {
                    score += 1.0;
                    findings.push(Finding {
                        severity: Severity::Success,
                        category: "Demo Quality".to_string(),
                        message: "Quick-start section found in README".to_string(),
                        location: Some("README.md".to_string()),
                        impact_points: 1.0,
                    });
                }

                let one_liner_patterns = [
                    r"```(?:bash|sh)?\n(?:cargo install|pip install|npm install|npx)[^\n]+\n```",
                    r"```(?:bash|sh)?\n[^\n]{1,80}\n```",
                ];

                if check_readme_patterns(&content, &one_liner_patterns) {
                    score += 1.0;
                    findings.push(Finding {
                        severity: Severity::Success,
                        category: "Demo Quality".to_string(),
                        message: "One-liner install/run command found".to_string(),
                        location: Some("README.md".to_string()),
                        impact_points: 1.0,
                    });
                }
            }
        }

        score = score.min(3.0_f64);

        if findings.is_empty() {
            findings.push(Finding {
                severity: Severity::Warning,
                category: "Demo Quality".to_string(),
                message: "No quick-start documentation found".to_string(),
                location: None,
                impact_points: 0.0,
            });
        }

        Ok(SubcategoryScore {
            id: "G1".to_string(),
            name: "Time-to-Interaction".to_string(),
            score,
            max_score: 3.0,
            findings,
        })
    }
}
