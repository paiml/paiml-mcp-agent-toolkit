// DemoScorer G3: Visual Stability (2 points)
// Checks for consistent output formatting, progress indicators, structured output
// Based on Posnett et al. (2011) - avoids ecological fallacy

const RUST_RICH_LIBS: &[&str] = &[
    "indicatif", "console", "colored", "termcolor", "ratatui",
    "crossterm", "comfy-table", "prettytable", "dialoguer", "owo-colors",
];

const NODE_RICH_LIBS: &[&str] = &[
    "chalk", "ora", "ink", "blessed", "cli-table", "boxen", "figlet",
];

const PYTHON_RICH_LIBS: &[&str] = &["rich", "tqdm", "colorama", "click", "typer"];

async fn detect_manifest_libs(repo_path: &Path) -> Vec<&'static str> {
    debug_assert!(repo_path.exists(), "repo_path must exist: {}", repo_path.display());
    let mut detected: Vec<&str> = vec![];

    let cargo_toml = repo_path.join("Cargo.toml");
    if cargo_toml.exists() {
        if let Ok(content) = tokio::fs::read_to_string(&cargo_toml).await {
            detected.extend(RUST_RICH_LIBS.iter().filter(|p| content.contains(**p)));
        }
        return detected;
    }

    let package_json = repo_path.join("package.json");
    if package_json.exists() {
        if let Ok(content) = tokio::fs::read_to_string(&package_json).await {
            detected.extend(NODE_RICH_LIBS.iter().filter(|p| content.contains(**p)));
        }
        return detected;
    }

    let pyproject = repo_path.join("pyproject.toml");
    if pyproject.exists() {
        if let Ok(content) = tokio::fs::read_to_string(&pyproject).await {
            detected.extend(PYTHON_RICH_LIBS.iter().filter(|p| content.contains(**p)));
        }
    }

    detected
}

async fn check_structured_output(demo_files: &[PathBuf]) -> bool {
    let structured_patterns = [
        r#"println!\s*\(\s*"\s*\{"#,
        r#"eprintln!\s*\("#,
        r#"serde_json::to_string_pretty"#,
        r#"format!\s*\("#,
        r#"table\.add_row"#,
        r#"ProgressBar::new"#,
        r#"spinner"#,
    ];

    for file_path in demo_files {
        let Ok(content) = tokio::fs::read_to_string(file_path).await else {
            continue;
        };
        for pattern in structured_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if re.is_match(&content) {
                    return true;
                }
            }
        }
    }
    false
}

impl DemoScorer {
    /// Score Visual Stability (G3: 2 points)
    async fn score_visual_stability(&self, repo_path: &Path) -> Result<SubcategoryScore> {
        debug_assert!(repo_path.exists(), "repo_path must exist: {}", repo_path.display());
        let mut score: f64 = 0.0;
        let mut findings = vec![];

        let demo_files = self.find_demo_files(repo_path).await;
        let detected_libs = detect_manifest_libs(repo_path).await;

        // Genchi Genbutsu: Verify actual usage in src/ files (Posnett et al. 2011)
        let src_path = repo_path.join("src");
        let verified_usage = if !detected_libs.is_empty() && src_path.exists() {
            self.verify_library_usage(&src_path, &detected_libs).await
        } else {
            false
        };

        // Scoring: Manifest detection = 0.5, Verified usage = 1.0
        if !detected_libs.is_empty() {
            let (sev, pts, msg) = if verified_usage {
                (Severity::Success, 1.0, format!(
                    "Rich output libraries detected and verified in use: {}",
                    detected_libs.join(", ")
                ))
            } else {
                (Severity::Info, 0.5, format!(
                    "Rich output libraries in manifest but usage not verified: {}",
                    detected_libs.join(", ")
                ))
            };
            score += pts;
            findings.push(Finding {
                severity: sev,
                category: "Demo Quality".to_string(),
                message: msg,
                location: if verified_usage { Some("src/".to_string()) } else { None },
                impact_points: pts,
            });
        }

        if check_structured_output(&demo_files).await {
            score += 1.0;
            findings.push(Finding {
                severity: Severity::Success,
                category: "Demo Quality".to_string(),
                message: "Structured/formatted output detected in demos".to_string(),
                location: None,
                impact_points: 1.0,
            });
        }

        score = score.min(2.0_f64);

        if findings.is_empty() {
            findings.push(Finding {
                severity: Severity::Warning,
                category: "Demo Quality".to_string(),
                message: "Consider adding rich terminal output (indicatif, colored, etc.)".to_string(),
                location: None,
                impact_points: 0.0,
            });
        }

        Ok(SubcategoryScore {
            id: "G3".to_string(),
            name: "Visual Stability".to_string(),
            score,
            max_score: 2.0,
            findings,
        })
    }
}
