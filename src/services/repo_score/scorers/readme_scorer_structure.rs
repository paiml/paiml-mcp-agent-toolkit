// ReadmeScorer structure and comprehensiveness analysis methods
// A2: README Comprehensiveness (5 points) - Required sections present
// A3: Professional Structure (5 points) - Hero image, ToC, centered header, no bot patterns

impl ReadmeScorer {
    /// Score README comprehensiveness (A2: 5 points)
    async fn score_comprehensiveness(&self, repo_path: &Path) -> Result<SubcategoryScore> {
        debug_assert!(repo_path.exists(), "repo_path must exist: {}", repo_path.display());
        let readme_path = repo_path.join("README.md");

        if !readme_path.exists() {
            return Ok(SubcategoryScore {
                id: "A2".to_string(),
                name: "README Comprehensiveness".to_string(),
                score: 0.0,
                max_score: 5.0,
                findings: vec![],
            });
        }

        let content = tokio::fs::read_to_string(&readme_path).await?;

        // Required sections (1.0 points each, 5 sections = 5 points)
        let required_sections = vec![
            (
                "Project Description",
                vec![
                    r"(?i)##\s*(overview|about|description)",
                    r"(?i)#\s+[^#\n]+\n\n[^#]", // Project title followed by description
                ],
            ),
            ("Installation", vec![r"(?i)##\s*install(ation)?"]),
            (
                "Usage",
                vec![r"(?i)##\s*(usage|getting\s+started|quick\s*start)"],
            ),
            (
                "License",
                vec![r"(?i)##\s*license", r"(?i)\bMIT\b", r"(?i)\bApache\b"],
            ),
            (
                "Contributing",
                vec![r"(?i)##\s*contribut(ing|e)", r"(?i)CONTRIBUTING\.md"],
            ),
        ];

        let mut score = 0.0;
        let mut findings = vec![];

        for (section_name, patterns) in required_sections {
            let mut found = false;
            for pattern in patterns {
                if let Ok(re) = regex::Regex::new(pattern) {
                    if re.is_match(&content) {
                        found = true;
                        break;
                    }
                }
            }

            if found {
                score += 1.0;
                findings.push(Finding {
                    severity: Severity::Success,
                    category: "Documentation".to_string(),
                    message: format!("{} section found", section_name),
                    location: Some("README.md".to_string()),
                    impact_points: 1.0,
                });
            } else {
                findings.push(Finding {
                    severity: Severity::Warning,
                    category: "Documentation".to_string(),
                    message: format!("{} section missing", section_name),
                    location: Some("README.md".to_string()),
                    impact_points: 0.0,
                });
            }
        }

        Ok(SubcategoryScore {
            id: "A2".to_string(),
            name: "README Comprehensiveness".to_string(),
            score,
            max_score: 5.0,
            findings,
        })
    }

    /// Score professional structure (A3: 5 points)
    /// Detects professional README patterns vs bot-generated/stream-of-consciousness
    async fn score_professional_structure(&self, repo_path: &Path) -> Result<SubcategoryScore> {
        debug_assert!(repo_path.exists(), "repo_path must exist: {}", repo_path.display());
        let readme_path = repo_path.join("README.md");

        if !readme_path.exists() {
            return Ok(SubcategoryScore {
                id: "A3".to_string(),
                name: "Professional Structure".to_string(),
                score: 0.0,
                max_score: 5.0,
                findings: vec![],
            });
        }

        let content = tokio::fs::read_to_string(&readme_path).await?;
        let mut score = 0.0;
        let mut findings = vec![];

        // 1. Hero image present (1.5 points)
        let has_hero_image = self.check_hero_image(repo_path, &content).await;
        if has_hero_image {
            score += 1.5;
            findings.push(Finding {
                severity: Severity::Success,
                category: "Documentation".to_string(),
                message: "Hero image present".to_string(),
                location: Some("README.md".to_string()),
                impact_points: 1.5,
            });
        } else {
            findings.push(Finding {
                severity: Severity::Warning,
                category: "Documentation".to_string(),
                message: "Hero image missing (add docs/hero.svg or image at top of README)"
                    .to_string(),
                location: Some("README.md".to_string()),
                impact_points: 0.0,
            });
        }

        // 2. Centered header (0.5 points)
        let has_centered_header =
            content.contains("<p align=\"center\">") || content.contains("<h1 align=\"center\">");
        if has_centered_header {
            score += 0.5;
            findings.push(Finding {
                severity: Severity::Success,
                category: "Documentation".to_string(),
                message: "Professional centered header".to_string(),
                location: Some("README.md".to_string()),
                impact_points: 0.5,
            });
        } else {
            findings.push(Finding {
                severity: Severity::Info,
                category: "Documentation".to_string(),
                message: "Consider using centered header for professional look".to_string(),
                location: Some("README.md".to_string()),
                impact_points: 0.0,
            });
        }

        // 3. Table of Contents (1.0 point)
        let has_toc = regex::Regex::new(r"(?i)##?\s*(table\s+of\s+contents|contents|toc)")
            .map(|re| re.is_match(&content))
            .unwrap_or(false)
            || content.matches("](#").count() >= 4; // Multiple anchor links indicate ToC

        if has_toc {
            score += 1.0;
            findings.push(Finding {
                severity: Severity::Success,
                category: "Documentation".to_string(),
                message: "Table of Contents present".to_string(),
                location: Some("README.md".to_string()),
                impact_points: 1.0,
            });
        } else {
            findings.push(Finding {
                severity: Severity::Warning,
                category: "Documentation".to_string(),
                message: "Table of Contents missing".to_string(),
                location: Some("README.md".to_string()),
                impact_points: 0.0,
            });
        }

        // 4. Features section with bullet points or table (0.5 points)
        let has_features_section = regex::Regex::new(r"(?i)##\s*features")
            .map(|re| re.is_match(&content))
            .unwrap_or(false);
        if has_features_section {
            score += 0.5;
            findings.push(Finding {
                severity: Severity::Success,
                category: "Documentation".to_string(),
                message: "Features section present".to_string(),
                location: Some("README.md".to_string()),
                impact_points: 0.5,
            });
        }

        // 5. No stream-of-consciousness patterns (1.5 points)
        // Bot-generated READMEs often have release notes, changelogs, or version history at the top
        let first_50_lines: String = content.lines().take(50).collect::<Vec<_>>().join("\n");
        let stream_of_consciousness_patterns = [
            r"(?i)##?\s*(current\s+release|what'?s\s+new|v?\d+\.\d+\.\d+\s*[-:])",
            r"(?i)##?\s*(changelog|release\s+notes|version\s+history)",
            r"(?i)##?\s*(latest\s+bug\s+fixes|recent\s+changes)",
            r"(?i)\*\*major\s+feature\*\*",
            r"(?i)##?\s*previous\s+release",
        ];

        let mut is_stream_of_consciousness = false;
        for pattern in stream_of_consciousness_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if re.is_match(&first_50_lines) {
                    is_stream_of_consciousness = true;
                    break;
                }
            }
        }

        if !is_stream_of_consciousness {
            score += 1.5;
            findings.push(Finding {
                severity: Severity::Success,
                category: "Documentation".to_string(),
                message: "Professional structure (no release notes at top)".to_string(),
                location: Some("README.md".to_string()),
                impact_points: 1.5,
            });
        } else {
            findings.push(Finding {
                severity: Severity::Warning,
                category: "Documentation".to_string(),
                message: "Stream-of-consciousness pattern detected: release notes/changelog at top of README appears bot-generated. Move to CHANGELOG.md".to_string(),
                location: Some("README.md".to_string()),
                impact_points: 0.0,
            });
        }

        Ok(SubcategoryScore {
            id: "A3".to_string(),
            name: "Professional Structure".to_string(),
            score,
            max_score: 5.0,
            findings,
        })
    }

    /// Check if a hero image is present and valid
    async fn check_hero_image(&self, repo_path: &Path, content: &str) -> bool {
        debug_assert!(repo_path.exists(), "repo_path must exist: {}", repo_path.display());
        // Check for hero.svg in standard locations
        let hero_paths = [
            "docs/hero.svg",
            "docs/hero.png",
            ".github/hero.svg",
            ".github/hero.png",
            "assets/hero.svg",
            "assets/hero.png",
        ];

        for path in hero_paths {
            if repo_path.join(path).exists() {
                return true;
            }
        }

        // Check if README starts with an image (within first 20 lines)
        let first_20_lines: String = content.lines().take(20).collect::<Vec<_>>().join("\n");
        regex::Regex::new(r#"<img[^>]+src=|!\[[^\]]*\]\("#)
            .map(|re| re.is_match(&first_20_lines))
            .unwrap_or(false)
    }
}
