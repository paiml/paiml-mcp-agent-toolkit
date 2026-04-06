// ReadmeScorer accuracy analysis methods (A1: README Accuracy - 5 points)
// Checks for broken links, broken image references, and empty/missing README.

impl ReadmeScorer {
    /// Score README accuracy (A1: 5 points)
    /// Checks for broken links and valid image references
    async fn score_accuracy(&self, repo_path: &Path) -> Result<SubcategoryScore> {
        debug_assert!(repo_path.exists(), "repo_path must exist: {}", repo_path.display());
        let readme_path = repo_path.join("README.md");

        if !readme_path.exists() {
            return Ok(SubcategoryScore {
                id: "A1".to_string(),
                name: "README Accuracy".to_string(),
                score: 0.0,
                max_score: 5.0,
                findings: vec![Finding {
                    severity: Severity::Error,
                    category: "Documentation".to_string(),
                    message: "README.md not found".to_string(),
                    location: Some(readme_path.display().to_string()),
                    impact_points: -5.0,
                }],
            });
        }

        let content = tokio::fs::read_to_string(&readme_path).await?;
        let mut score: f64 = 5.0;
        let mut findings = vec![];

        // Check file is not empty
        if content.trim().is_empty() {
            return Ok(SubcategoryScore {
                id: "A1".to_string(),
                name: "README Accuracy".to_string(),
                score: 0.0,
                max_score: 5.0,
                findings: vec![Finding {
                    severity: Severity::Error,
                    category: "Documentation".to_string(),
                    message: "README.md is empty".to_string(),
                    location: Some(readme_path.display().to_string()),
                    impact_points: -5.0,
                }],
            });
        }

        // Check for broken image references
        let image_pattern =
            regex::Regex::new(r#"!\[[^\]]*\]\(([^)]+)\)|<img[^>]+src=["']([^"']+)["']"#)
                .expect("internal error");
        for cap in image_pattern.captures_iter(&content) {
            let img_path = cap
                .get(1)
                .or_else(|| cap.get(2))
                .map(|m| m.as_str())
                .unwrap_or("");

            // Skip external URLs
            if img_path.starts_with("http://") || img_path.starts_with("https://") {
                continue;
            }

            // Check if local file exists
            let full_path = repo_path.join(img_path);
            if !full_path.exists() {
                score -= 1.0;
                findings.push(Finding {
                    severity: Severity::Error,
                    category: "Documentation".to_string(),
                    message: format!("Broken image link: {}", img_path),
                    location: Some("README.md".to_string()),
                    impact_points: -1.0,
                });
            }
        }

        // Check for broken relative links
        let link_pattern = regex::Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").expect("internal error");
        for cap in link_pattern.captures_iter(&content) {
            let link_path = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            // Skip external URLs, anchors, and mailto
            if link_path.starts_with("http://")
                || link_path.starts_with("https://")
                || link_path.starts_with("#")
                || link_path.starts_with("mailto:")
            {
                continue;
            }

            // Remove anchor from path
            let clean_path = link_path.split('#').next().unwrap_or(link_path);
            if clean_path.is_empty() {
                continue;
            }

            let full_path = repo_path.join(clean_path);
            if !full_path.exists() {
                score -= 0.5;
                findings.push(Finding {
                    severity: Severity::Warning,
                    category: "Documentation".to_string(),
                    message: format!("Broken link: {}", link_path),
                    location: Some("README.md".to_string()),
                    impact_points: -0.5,
                });
            }
        }

        score = score.max(0.0);

        if findings.is_empty() {
            findings.push(Finding {
                severity: Severity::Success,
                category: "Documentation".to_string(),
                message: "All links and images valid".to_string(),
                location: Some("README.md".to_string()),
                impact_points: 0.0,
            });
        }

        Ok(SubcategoryScore {
            id: "A1".to_string(),
            name: "README Accuracy".to_string(),
            score,
            max_score: 5.0,
            findings,
        })
    }
}
